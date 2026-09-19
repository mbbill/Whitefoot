//! Acceptance canaries for the originating source-proof boundary. Every protected
//! operation below lacks an ordinary fact or a source proof for its domain,
//! so the checker must reject at the live rule that owns the operation.

use crate::{SemanticIssueKind, SemanticOutcome, SemanticRule};

use super::assert_rule_kind;

fn rejects_as(source: &[u8], rule: SemanticRule, expected: fn(&SemanticIssueKind) -> bool) {
    assert_rule_kind(source, rule, expected);
}

fn accepts(source: &[u8]) {
    super::with_semantics(source, |outcome| {
        assert!(
            matches!(outcome, SemanticOutcome::Complete(_)),
            "source proof must be accepted: {outcome:?}"
        );
    });
}

#[test]
fn an_unproved_array_bound_rejects_under_op4() {
    let source = br#"const values: Array<i32, 4> =[0_i32, 0_i32, 0_i32, 0_i32];

fn read(input: own u64) -> result: own i32 pure {
  let bounded = imin(input, 3_u64);
  return values[bounded];
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    rejects_as(source, SemanticRule::Op4, |kind| {
        matches!(kind, SemanticIssueKind::UndischargedBoundsObligation { .. })
    });
}

#[test]
fn an_unproved_exact_addition_domain_rejects_under_op2() {
    let source = br#"fn bump(input: own u64) -> result: own u64 pure {
  let bounded = imin(input, 100_u64);
  return bounded + 1_u64;
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    rejects_as(source, SemanticRule::Op2, |kind| {
        matches!(
            kind,
            SemanticIssueKind::UndischargedIntegerDomainObligation { .. }
        )
    });
}

#[test]
fn an_unproved_call_requirement_rejects_under_fn8() {
    let source = br#"fn need(index: own u64) -> result: own unit pure contract {
  requires index < 4_u64;
} {
  return unit;
}

fn caller(input: own u64) -> result: own unit pure {
  let bounded = imin(input, 3_u64);
  need(index: bounded);
  return unit;
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    rejects_as(source, SemanticRule::Fn8, |kind| {
        matches!(kind, SemanticIssueKind::UndischargedCallRequirement(_))
    });
}

#[test]
fn an_unproved_postcondition_rejects_under_fn9() {
    let source = br#"fn guarded() -> result: own i32 pure contract {
  ensures result == 1_i32;
} {
  let reviewed = 1_i32;
  let cursor = 0_u8;
  loop {
    if cursor == 3_u8 {
      break;
    } else {
      set reviewed = 1_i32;
      set cursor = cursor +wrap 1_u8;
    }
  }
  return reviewed;
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    rejects_as(source, SemanticRule::Fn9, |kind| {
        matches!(kind, SemanticIssueKind::UndischargedPostcondition(_))
    });
}

#[test]
fn an_unproved_loop_header_fact_rejects_under_inv1() {
    let source = br#"const values: Array<i32, 4> =[0_i32, 0_i32, 0_i32, 0_i32];

fn read(input: own u64) -> result: own unit pure {
  let bounded = imin(input, 3_u64);
  for (
    i in 0_u64..1_u64,
    invariant limit: bounded <= 3_u64
  ) {
    let value = values[bounded];
  }
  return unit;
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    rejects_as(source, SemanticRule::Inv1, |kind| {
        matches!(kind, SemanticIssueKind::UndischargedLoopInvariant { .. })
    });
}

#[test]
fn an_unproved_allocation_ceiling_rejects_under_op9() {
    // The acquiring row is `box_array_filled`, a runtime-capacity [OP-13]
    // construction, which carries the same [OP-9] allocation-fit obligation on
    // its own count that the retiring `buffer_new` did. Allocation is total
    // [STOR-8], so there is no fallible outcome to match and the refusal
    // reached here is the static ceiling alone.
    let source = br#"fn allocate(count: own u64) -> result: own unit pure {
  let values = box_array_filled::<u16>(count: count, value: 0_u16);
  return unit;
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    rejects_as(source, SemanticRule::Op9, |kind| {
        matches!(
            kind,
            SemanticIssueKind::UndischargedAllocationFitObligation { .. }
        )
    });
}

#[test]
fn unproved_prelude_endpoints_reject_under_fn8() {
    let source = br#"fn publish(factory: &HandleFactory, output: &OutputStream, source: &[u8], start: own u64, end: own u64) -> result: own unit reads(source), writes(factory), writes(output) {
  match write_once(factory: factory, output: output, source: source, start: start, end: end) {
    Ok(value: next) => {
    }
    Err(error: problem) => {
    }
  }
  return unit;
}

fn main(output: own OutputStream) -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    rejects_as(source, SemanticRule::Fn8, |kind| {
        matches!(kind, SemanticIssueKind::UndischargedCallRequirement(..))
    });
}

#[test]
fn an_external_index_needs_a_real_control_flow_fact() {
    let direct = br#"const bytes: Array<u8, 4> =[0_u8, 0_u8, 0_u8, 0_u8];

fn main(args: own Args) -> status: own ExitStatus reads(args) {
  let index = args_count(args: &args);
  let value = bytes[index];
  return exit_status(code: value);
}
"#;
    rejects_as(direct, SemanticRule::Op4, |kind| {
        matches!(kind, SemanticIssueKind::UndischargedBoundsObligation { .. })
    });

    let guarded = br#"const bytes: Array<u8, 4> =[0_u8, 0_u8, 0_u8, 0_u8];

fn main(args: own Args) -> status: own ExitStatus reads(args) {
  let index = args_count(args: &args);
  let spare = bytes.len;
  if index < spare {
    let value = bytes[index];
    return exit_status(code: value);
  } else {
    return exit_status(code: 0_u8);
  }
}
"#;
    accepts(guarded);
}

#[test]
fn an_external_call_actual_needs_a_real_control_flow_fact() {
    // An `own` parameter carries no effect entry at all in v0.60 [EFF-1], so
    // the reader's row is `pure`; the requirement and its measure operand are
    // unchanged apart from the [OP-15] member spelling.
    let function = r#"fn read_at_index(bytes: own Array<u8, 4>, index: own u64) -> result: own u8 pure contract {
  define spare = bytes.len;
  requires index < spare;
} {
  return bytes[index];
}

"#;
    let direct = format!(
        "{function}fn main(args: own Args) -> status: own ExitStatus reads(args) {{\n  let index = args_count(args: &args);\n  let bytes = array_filled::<u8, 4>(value: 0_u8);\n  let value = read_at_index(bytes: move bytes, index: index);\n  return exit_status(code: value);\n}}\n"
    );
    rejects_as(direct.as_bytes(), SemanticRule::Fn8, |kind| {
        matches!(kind, SemanticIssueKind::UndischargedCallRequirement(_))
    });

    let guarded = format!(
        "{function}fn main(args: own Args) -> status: own ExitStatus reads(args) {{\n  let index = args_count(args: &args);\n  let bytes = array_filled::<u8, 4>(value: 0_u8);\n  let spare = bytes.len;\n  if index < spare {{\n    let value = read_at_index(bytes: move bytes, index: index);\n    return exit_status(code: value);\n  }} else {{\n    return exit_status(code: 0_u8);\n  }}\n}}\n"
    );
    accepts(guarded.as_bytes());
}

#[test]
fn a_recursive_postcondition_cycle_cannot_publish_its_own_premise() {
    let source = br#"fn first(value: own i32) -> result: own i32 pure contract {
  ensures result == value;
} {
  let called = second(value: value);
  return called;
}

fn second(value: own i32) -> result: own i32 pure contract {
  ensures result == value;
} {
  let called = first(value: value);
  return called;
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    rejects_as(source, SemanticRule::Fn9, |kind| {
        matches!(kind, SemanticIssueKind::UndischargedPostcondition(_))
    });
}

#[test]
fn originating_proof_context_retains_acceptance_results_and_derivations() {
    let source = br#"fn increment(x: own u8, middle: own u8, left: own u64, left_limit: own u64, center: own u64, center_limit: own u64, right: own u64, right_limit: own u64) -> result: own u8 pure contract {
  requires x <= middle;
  requires middle <= 254_u8;
  requires left <= left_limit;
  requires center <= center_limit;
  requires right <= right_limit;
  ensures result <= 255_u8;
} {
  invariant combined: left + center + right <= left_limit + center_limit + right_limit {
    use (left <= left_limit);
    use (center <= center_limit);
    use (right <= right_limit);
  }
  invariant upper_bound: x <= 254_u8;
  let result = x + 1_u8;
  return result;
}

fn main() -> status: own ExitStatus pure {
  for (
    i in 0_u64..1_u64,
    invariant limit: i <= 1_u64
  ) {
  }
  let value = increment(x: 1_u8, middle: 2_u8, left: 1_u64, left_limit: 2_u64, center: 3_u64, center_limit: 4_u64, right: 5_u64, right_limit: 6_u64);
  return exit_status(code: 0_u8);
}
"#;
    super::with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("ordinary proof sources must check: {outcome:?}");
        };
        let mut saw_integer_domain = false;
        let mut saw_call_requirement = false;
        let mut saw_invariant = false;
        let mut saw_source_proof = false;
        let mut saw_postcondition = false;
        for function in &checked.data.functions {
            for obligation in &function.entailment.obligations {
                assert!(obligation.discharged);
                assert!(obligation.derivation.is_some());
                saw_integer_domain = true;
            }
            for call in &function.entailment.call_goals {
                assert!(matches!(
                    call.disposition,
                    super::super::entailment::CallGoalDisposition::Discharged
                ));
                assert!(call.derivation.is_some());
                saw_call_requirement = true;
            }
            for invariant in &function.entailment.loop_invariants {
                assert!(invariant.proof.discharged());
                saw_invariant = true;
            }
            for proof in &function.entailment.source_proofs {
                assert!(proof.check.discharged());
                saw_source_proof = true;
            }
            for proof in &function.entailment.postconditions {
                assert!(proof.aggregate.discharged);
                assert!(proof.aggregate.derivation.is_some());
                for exit in &proof.exits {
                    assert!(matches!(
                        exit.disposition,
                        super::super::entailment::PostconditionDisposition::Discharged
                    ));
                    assert!(exit.derivation.is_some());
                }
                saw_postcondition = true;
            }
        }
        assert!(saw_integer_domain);
        assert!(saw_call_requirement);
        assert!(saw_invariant);
        assert!(saw_source_proof);
        assert!(saw_postcondition);
    });
}
