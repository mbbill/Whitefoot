use crate::{ClaimBoundaryResultDetail, SemanticIssueKind, SemanticOutcome, SemanticRule};

use super::with_semantics;

const REVIEW: &str = "premises: this fixture exposes one call-result dependency to the claim-locality judgment\\nderivation: the predicate reads the named carrier whose value descends from that call result\\nconclusion: the named predicate is asserted by this claim\\nchecker gap: CLM-1 must reject before lifecycle or residuality can use this predicate\\nconsumers: the following partial operation is present only to make the old unsound claim load-bearing";

const LOCAL_REVIEW: &str = "premises: the predicate reads only values produced inside the current function from its own parameters and local control\\nderivation: this fixture intentionally leaves that local theorem to the executed claim rather than importing behavior from another function\\nconclusion: the named predicate is asserted by this claim\\nchecker gap: this fixture checks that claim locality does not reject genuinely local runtime values\\nconsumers: the following partial operation consumes the predicate established at this exact point";

fn assert_complete(source: &[u8]) {
    with_semantics(source, |outcome| {
        assert!(
            matches!(outcome, SemanticOutcome::Complete(_)),
            "expected the local or summary-consuming fixture to check, got {outcome:?}"
        );
    });
}

fn assert_user_call_locality(
    source: &[u8],
    claim: &str,
    component: u32,
    carrier: &str,
    callee: &str,
) {
    with_semantics(source, |outcome| {
        let SemanticOutcome::SourceIssue { issue } = outcome else {
            panic!("expected a CLM-1 claim-locality rejection, got {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Clm1);
        let SemanticIssueKind::NonLocalClaim(detail) = issue.kind() else {
            panic!("expected a non-local-claim payload, got {:?}", issue.kind());
        };
        assert_eq!(detail.name, claim);
        assert_eq!(detail.component, component);
        assert_eq!(detail.carrier, carrier);
        let ClaimBoundaryResultDetail::UserCall {
            declaration: _,
            callee: actual,
        } = &detail.boundary
        else {
            panic!("expected a user-call boundary, got {:?}", detail.boundary);
        };
        assert_eq!(actual, callee);
        assert_eq!(
            detail.mechanical_fix,
            "publish the required cross-function relation as an exact verified ensures clause on the callee and remove this caller claim"
        );
        assert!(!detail.boundary_call.components().is_empty());
    });
}

fn assert_system_call_locality(
    source: &[u8],
    claim: &str,
    component: u32,
    carrier: &str,
    operation: &str,
) {
    with_semantics(source, |outcome| {
        let SemanticOutcome::SourceIssue { issue } = outcome else {
            panic!("expected a CLM-1 system-result locality rejection, got {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Clm1);
        let SemanticIssueKind::NonLocalClaim(detail) = issue.kind() else {
            panic!("expected a non-local-claim payload, got {:?}", issue.kind());
        };
        assert_eq!(detail.name, claim);
        assert_eq!(detail.component, component);
        assert_eq!(detail.carrier, carrier);
        let ClaimBoundaryResultDetail::SystemCall {
            declaration_ordinal,
            operation: actual,
        } = &detail.boundary
        else {
            panic!("expected a system-call boundary, got {:?}", detail.boundary);
        };
        assert_eq!(actual, operation);
        let expected_ordinal = crate::SYSTEM_OPERATIONS
            .iter()
            .position(|candidate| candidate.spelling == operation)
            .expect("the tested operation is in the system catalog");
        assert_eq!(usize::from(*declaration_ordinal), expected_ordinal);
        assert_eq!(
            detail.mechanical_fix,
            "use the system operation's specified fact or typed outcome, or branch on the returned value; do not claim an unstated system-result property"
        );
        assert!(!detail.boundary_call.components().is_empty());
    });
}

#[test]
fn a_caller_cannot_claim_an_unpublished_callee_result_bound() {
    let source = format!(
        r#"fn clamp_three(value: own u64) -> result: own u64 pure {{
  return imin(value, 3_u64);
}}

fn read(values: own array<u8, 4>, input: own u64) -> result: own u8 traps {{
  let bounded = clamp_three(value: input);
  claim reviewed_bound: ilt(bounded, 4_u64) because "{REVIEW}";
  return values[bounded];
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_user_call_locality(
        source.as_bytes(),
        "reviewed_bound",
        0,
        "bounded",
        "clamp_three",
    );
}

#[test]
fn copying_a_call_result_does_not_launder_claim_authority() {
    let source = format!(
        r#"fn clamp_three(value: own u64) -> result: own u64 pure {{
  return imin(value, 3_u64);
}}

fn read(values: own array<u8, 4>, input: own u64) -> result: own u8 traps {{
  let returned = clamp_three(value: input);
  let copied = returned;
  claim reviewed_bound: ilt(copied, 4_u64) because "{REVIEW}";
  return values[copied];
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_user_call_locality(
        source.as_bytes(),
        "reviewed_bound",
        0,
        "copied",
        "clamp_three",
    );
}

#[test]
fn a_wrapping_identity_operation_does_not_launder_claim_authority() {
    let source = format!(
        r#"fn clamp_three(value: own u64) -> result: own u64 pure {{
  return imin(value, 3_u64);
}}

fn read(values: own array<u8, 4>, input: own u64) -> result: own u8 traps {{
  let returned = clamp_three(value: input);
  let laundered = returned +wrap 0_u64;
  claim reviewed_bound: ilt(laundered, 4_u64) because "{REVIEW}";
  return values[laundered];
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_user_call_locality(
        source.as_bytes(),
        "reviewed_bound",
        0,
        "laundered",
        "clamp_three",
    );
}

#[test]
fn a_later_nonlocal_conjunct_reports_its_canonical_component() {
    let source = format!(
        r#"fn clamp_three(value: own u64) -> result: own u64 pure {{
  return imin(value, 3_u64);
}}

fn need(flag: own Bool) -> result: own unit pure contract {{
  requires flag;
}} {{
  return unit;
}}

fn probe(local: own u64, input: own u64) -> result: own unit traps {{
  let local_unknown = ixor(local, 123_u64);
  let first = ilt(local_unknown, 8_u64);
  let returned = clamp_three(value: input);
  let second = ilt(returned, 4_u64);
  let both = band(first, second);
  claim reviewed_pair: both because "{REVIEW}";
  need(flag: both);
  return unit;
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_user_call_locality(
        source.as_bytes(),
        "reviewed_pair",
        1,
        "returned",
        "clamp_three",
    );
}

#[test]
fn the_earliest_boundary_witness_selects_its_support_carrier() {
    let source = format!(
        r#"fn first(value: own u64) -> result: own u64 pure {{
  return value;
}}

fn second(value: own u64) -> result: own u64 pure {{
  return value;
}}

fn need(flag: own Bool) -> result: own unit pure contract {{
  requires flag;
}} {{
  return unit;
}}

fn probe(left: own u64, right: own u64) -> result: own unit traps {{
  let earlier = first(value: right);
  let later = second(value: left);
  let ordered = ilt(later, earlier);
  claim reviewed_order: ordered because "{REVIEW}";
  need(flag: ordered);
  return unit;
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_user_call_locality(source.as_bytes(), "reviewed_order", 0, "earlier", "first");
}

#[test]
fn an_uninstantiated_generic_body_uses_the_same_claim_locality_rule() {
    let source = format!(
        r#"fn identity<T: Int>(value: own T) -> result: own T pure {{
  return value;
}}

fn unused<T: Int>(value: own T) -> result: own T traps {{
  let returned = identity<T>(value: value);
  claim reviewed_identity: ieq(returned, value) because "{REVIEW}";
  return returned;
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_user_call_locality(
        source.as_bytes(),
        "reviewed_identity",
        0,
        "returned",
        "identity",
    );
}

#[test]
fn a_system_call_result_is_never_local_claim_authority() {
    let source = format!(
        r#"command fn main(command.args as args: own Args) -> status: own ExitStatus reads(args), allocates(heap), traps {{
  region 'arguments {{
    let position = args_count<'arguments>(args: &'arguments args);
    let values = buffer_new(4_u64, 0_u8);
    let room = len(values);
    claim reviewed_position: ilt(position, room) because "{REVIEW}";
    let selected = values[position];
  }}
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_system_call_locality(
        source.as_bytes(),
        "reviewed_position",
        0,
        "position",
        "args_count",
    );
}

#[test]
fn a_direct_result_payload_keeps_the_call_boundary() {
    let source = format!(
        r#"fn hidden(value: own u64) -> result: own Result<u64, u64> pure {{
  let bounded = imin(value, 3_u64);
  return Ok<u64, u64>(value: bounded);
}}

fn read(values: own array<u8, 4>, input: own u64) -> result: own u8 traps {{
  match hidden(value: input) {{
    Ok(value: payload) => {{
      claim reviewed_payload: ilt(payload, 4_u64) because "{REVIEW}";
      return values[payload];
    }}
    Err(error: problem) => {{
      return 0_u8;
    }}
  }}
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_user_call_locality(
        source.as_bytes(),
        "reviewed_payload",
        0,
        "payload",
        "hidden",
    );
}

#[test]
fn a_result_tag_cannot_be_laundered_through_a_value_match() {
    let source = format!(
        r#"fn hidden(value: own u64) -> result: own Result<u64, u64> pure {{
  return Ok<u64, u64>(value: value);
}}

fn need_zero(value: own u64) -> result: own unit pure contract {{
  requires ieq(value, 0_u64);
}} {{
  return unit;
}}

fn probe(input: own u64) -> result: own unit traps {{
  let picked = match hidden(value: input) {{
    Ok(value: payload) => {{
      give 0_u64;
    }}
    Err(error: problem) => {{
      give 1_u64;
    }}
  }}
  claim reviewed_tag: ieq(picked, 0_u64) because "{REVIEW}";
  need_zero(value: picked);
  return unit;
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_user_call_locality(source.as_bytes(), "reviewed_tag", 0, "picked", "hidden");
}

#[test]
fn a_call_result_stored_in_a_struct_field_remains_nonlocal() {
    let source = format!(
        r#"struct Pair {{
  boundary: u64;
  local: u64;
}}

fn clamp_three(value: own u64) -> result: own u64 pure {{
  return imin(value, 3_u64);
}}

fn read(values: own array<u8, 4>, input: own u64, local: own u64) -> result: own u8 traps {{
  let returned = clamp_three(value: input);
  let pair = Pair(boundary: returned, local: local);
  let reloaded = pair.boundary;
  claim reviewed_field: ilt(reloaded, 4_u64) because "{REVIEW}";
  return values[reloaded];
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_user_call_locality(
        source.as_bytes(),
        "reviewed_field",
        0,
        "reloaded",
        "clamp_three",
    );
}

#[test]
fn a_local_struct_sibling_is_not_tainted_by_another_field() {
    let source = format!(
        r#"struct Pair {{
  boundary: u64;
  local: u64;
}}

fn opaque(value: own u64) -> result: own u64 pure {{
  return value;
}}

fn read(values: own array<u8, 4>, input: own u64, local: own u64) -> result: own u8 traps {{
  let returned = opaque(value: input);
  let pair = Pair(boundary: returned, local: local);
  claim local_sibling: ilt(pair.local, 4_u64) because "{LOCAL_REVIEW}";
  return values[pair.local];
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_complete(source.as_bytes());
}

#[test]
fn a_local_box_field_is_not_tainted_by_a_boundary_sibling() {
    let source = format!(
        r#"struct Pair {{
  boundary: u64;
  local: u64;
}}

fn opaque(value: own u64) -> result: own u64 pure {{
  return value;
}}

fn read(values: own array<u8, 4>, input: own u64, local: own u64) -> result: own u8 allocates(heap), traps {{
  let returned = opaque(value: input);
  let pair = Pair(boundary: returned, local: local);
  let holder = box_new(move pair);
  let observed = deref(holder).local;
  claim local_box_sibling: ilt(observed, 4_u64) because "{LOCAL_REVIEW}";
  return values[observed];
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_complete(source.as_bytes());
}

#[test]
fn a_direct_local_box_projection_is_not_tainted_by_a_boundary_sibling() {
    let source = format!(
        r#"struct Pair {{
  boundary: u64;
  local: u64;
}}

fn opaque(value: own u64) -> result: own u64 pure {{
  return value;
}}

fn read(values: own array<u8, 4>, input: own u64, local: own u64) -> result: own u8 allocates(heap), traps {{
  let returned = opaque(value: input);
  let pair = Pair(boundary: returned, local: local);
  let holder = box_new(move pair);
  claim local_box_projection: ilt(deref(holder).local, 4_u64) because "{LOCAL_REVIEW}";
  return values[deref(holder).local];
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_complete(source.as_bytes());
}

#[test]
fn the_length_of_a_returned_buffer_keeps_the_call_boundary() {
    let source = format!(
        r#"fn make_buffer() -> result: own buffer<u8> allocates(heap) {{
  return buffer_new(4_u64, 0_u8);
}}

fn read() -> result: own u8 allocates(heap), traps {{
  let returned = make_buffer();
  let length = len(returned);
  claim reviewed_length: ieq(length, 4_u64) because "{REVIEW}";
  return returned[3_u64];
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_user_call_locality(
        source.as_bytes(),
        "reviewed_length",
        0,
        "length",
        "make_buffer",
    );
}

#[test]
fn a_returned_box_cannot_hide_its_dereferenced_payload() {
    let source = format!(
        r#"fn boxed_three(value: own u64) -> result: own box<u64> allocates(heap) {{
  let bounded = imin(value, 3_u64);
  return box_new(bounded);
}}

fn read(values: own array<u8, 4>, input: own u64) -> result: own u8 allocates(heap), traps {{
  let holder = boxed_three(value: input);
  let observed = deref(holder);
  claim reviewed_box: ilt(observed, 4_u64) because "{REVIEW}";
  return values[observed];
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_user_call_locality(
        source.as_bytes(),
        "reviewed_box",
        0,
        "observed",
        "boxed_three",
    );
}

#[test]
fn a_returned_borrow_holder_cannot_hide_its_referent() {
    let source = format!(
        r#"fn relay['r](value: &'r u64) -> result: &'r u64 pure {{
  return &'r deref(value);
}}

fn read(values: own array<u8, 4>, index: own u64) -> result: own u8 traps {{
  region 'view {{
    let holder = relay<'view>(value: &'view index);
    let observed = deref(holder);
    claim reviewed_borrow: ilt(observed, 4_u64) because "{REVIEW}";
    return values[observed];
  }}
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_user_call_locality(source.as_bytes(), "reviewed_borrow", 0, "observed", "relay");
}

#[test]
fn a_value_if_join_does_not_launder_a_call_result() {
    let source = format!(
        r#"fn clamp_three(value: own u64) -> result: own u64 pure {{
  return imin(value, 3_u64);
}}

fn read(values: own array<u8, 4>, input: own u64, direct: own Bool) -> result: own u8 traps {{
  let returned = clamp_three(value: input);
  let picked = if direct {{
    give returned;
  }} else {{
    give returned +wrap 0_u64;
  }}
  claim reviewed_join: ilt(picked, 4_u64) because "{REVIEW}";
  return values[picked];
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_user_call_locality(
        source.as_bytes(),
        "reviewed_join",
        0,
        "picked",
        "clamp_three",
    );
}

#[test]
fn a_call_result_used_as_value_if_control_taints_the_delivery() {
    let source = format!(
        r#"fn hidden_true() -> result: own Bool pure {{
  return True();
}}

fn need_zero(value: own u64) -> result: own unit pure contract {{
  requires ieq(value, 0_u64);
}} {{
  return unit;
}}

fn probe() -> result: own unit traps {{
  let condition = hidden_true();
  let picked = if condition {{
    give 0_u64;
  }} else {{
    give 1_u64;
  }}
  claim reviewed_control: ieq(picked, 0_u64) because "{REVIEW}";
  need_zero(value: picked);
  return unit;
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_user_call_locality(
        source.as_bytes(),
        "reviewed_control",
        0,
        "picked",
        "hidden_true",
    );
}

#[test]
fn a_call_result_used_as_if_control_taints_written_values() {
    let source = format!(
        r#"fn hidden_true() -> result: own Bool pure {{
  return True();
}}

fn need_zero(value: own u64) -> result: own unit pure contract {{
  requires ieq(value, 0_u64);
}} {{
  return unit;
}}

fn probe() -> result: own unit traps {{
  let condition = hidden_true();
  let chosen = 2_u64;
  if condition {{
    set chosen = 0_u64;
  }} else {{
    set chosen = 1_u64;
  }}
  claim reviewed_write: ieq(chosen, 0_u64) because "{REVIEW}";
  need_zero(value: chosen);
  return unit;
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_user_call_locality(
        source.as_bytes(),
        "reviewed_write",
        0,
        "chosen",
        "hidden_true",
    );
}

#[test]
fn storing_and_reloading_a_call_result_does_not_launder_it() {
    let source = format!(
        r#"fn clamp_three(value: own u64) -> result: own u64 pure {{
  return imin(value, 3_u64);
}}

fn read(values: own array<u8, 4>, input: own u64) -> result: own u8 traps {{
  let returned = clamp_three(value: input);
  let storage = array_new<u64, 1>(0_u64);
  set storage[0_u64] = returned;
  let reloaded = storage[0_u64];
  claim reviewed_storage: ilt(reloaded, 4_u64) because "{REVIEW}";
  return values[reloaded];
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_user_call_locality(
        source.as_bytes(),
        "reviewed_storage",
        0,
        "reloaded",
        "clamp_three",
    );
}

/// The delivered value is selected and stays non-local; `cursor` is not, and
/// no arm writes it, so the continuation's claim over it is local.  v0.38
/// refused this one on the continuation's control frame.
#[test]
fn a_delivery_with_a_returning_arm_leaves_an_untouched_local_alone() {
    let source = format!(
        r#"fn hidden_true() -> result: own Bool pure {{
  return True();
}}

fn read(values: own array<u8, 4>, cursor: own u64, leave: own Bool) -> result: own u8 traps {{
  let condition = hidden_true();
  let ignored = if condition {{
    if leave {{
      return 0_u8;
    }} else {{
      give 0_u64;
    }}
  }} else {{
    give 1_u64;
  }}
  claim reviewed_delivery_continuation: ilt(cursor, 4_u64) because "{LOCAL_REVIEW}";
  return values[cursor];
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_complete(source.as_bytes());
}

#[test]
fn nested_exhaustive_delivery_does_not_taint_the_receiver_continuation() {
    let source = format!(
        r#"fn hidden_true() -> result: own Bool pure {{
  return True();
}}

fn read(values: own array<u8, 4>, cursor: own u64, choose: own Bool) -> result: own u8 traps {{
  let picked = if choose {{
    let condition = hidden_true();
    if condition {{
      give 0_u64;
    }} else {{
      give 1_u64;
    }}
  }} else {{
    give 2_u64;
  }}
  claim local_after_nested_delivery: ilt(cursor, 4_u64) because "{LOCAL_REVIEW}";
  return values[cursor];
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_complete(source.as_bytes());
}

#[test]
fn an_exact_local_overwrite_clears_an_earlier_call_result() {
    let source = format!(
        r#"fn opaque(value: own u64) -> result: own u64 pure {{
  return value;
}}

fn read(values: own array<u8, 4>, input: own u64, local: own u64) -> result: own u8 traps {{
  let slot = opaque(value: input);
  set slot = local;
  claim local_after_overwrite: ilt(slot, 4_u64) because "{LOCAL_REVIEW}";
  return values[slot];
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_complete(source.as_bytes());
}

#[test]
fn a_future_boundary_assignment_does_not_taint_an_earlier_local_claim() {
    let source = format!(
        r#"fn opaque(value: own u64) -> result: own u64 pure {{
  return value;
}}

fn read(values: own array<u8, 4>, input: own u64, local: own u64) -> result: own u8 traps {{
  let future = opaque(value: input);
  let cursor = local;
  claim local_before_write: ilt(cursor, 4_u64) because "{LOCAL_REVIEW}";
  let selected = values[cursor];
  set cursor = future;
  return selected;
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_complete(source.as_bytes());
}

#[test]
fn an_ordinary_parameter_remains_valid_local_claim_input() {
    let source = format!(
        r#"fn read(values: own array<u8, 4>, cursor: own u64) -> result: own u8 traps {{
  claim local_parameter: ilt(cursor, 4_u64) because "{LOCAL_REVIEW}";
  return values[cursor];
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_complete(source.as_bytes());
}

#[test]
fn a_loop_local_residual_is_not_mistaken_for_a_call_result() {
    let source = format!(
        r#"fn read(values: own array<u8, 4>, input: own u64, leave: own Bool) -> result: own u8 traps {{
  loop @again {{
    let cursor = ixor(input, 17_u64);
    claim local_loop: ilt(cursor, 4_u64) because "{LOCAL_REVIEW}";
    let selected = values[cursor];
    if leave {{
      return selected;
    }} else {{
      break @again;
    }}
  }}
  return values[0_u64];
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_complete(source.as_bytes());
}

#[test]
fn an_exact_ensures_does_not_authorize_a_caller_restatement_claim() {
    let source = format!(
        r#"fn identity(value: own i32) -> result: own i32 pure contract {{
  ensures ieq(result, value);
}} {{
  return value;
}}

fn need_same(left: own i32, right: own i32) -> result: own unit pure contract {{
  requires ieq(left, right);
}} {{
  return unit;
}}

fn caller(value: own i32) -> result: own unit traps {{
  let returned = identity(value: value);
  claim restated_summary: ieq(returned, value) because "{REVIEW}";
  need_same(left: returned, right: value);
  return unit;
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_user_call_locality(
        source.as_bytes(),
        "restated_summary",
        0,
        "returned",
        "identity",
    );
}

#[test]
fn a_weaker_ensures_does_not_authorize_a_stronger_caller_claim() {
    let source = format!(
        r#"fn identity(value: own i32) -> result: own i32 pure contract {{
  ensures ile(result, value);
}} {{
  return value;
}}

fn need_same(left: own i32, right: own i32) -> result: own unit pure contract {{
  requires ieq(left, right);
}} {{
  return unit;
}}

fn caller(value: own i32) -> result: own unit traps {{
  let returned = identity(value: value);
  claim strengthened_summary: ieq(returned, value) because "{REVIEW}";
  need_same(left: returned, right: value);
  return unit;
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_user_call_locality(
        source.as_bytes(),
        "strengthened_summary",
        0,
        "returned",
        "identity",
    );
}

#[test]
fn verified_exact_and_weak_ensures_are_consumed_without_caller_claims() {
    let source = br#"fn exact(value: own i32) -> result: own i32 pure contract {
  ensures ieq(result, value);
} {
  return value;
}

fn weak(value: own i32) -> result: own i32 pure contract {
  ensures ile(result, value);
} {
  return value;
}

fn need_same(left: own i32, right: own i32) -> result: own unit pure contract {
  requires ieq(left, right);
} {
  return unit;
}

fn need_ordered(left: own i32, right: own i32) -> result: own unit pure contract {
  requires ile(left, right);
} {
  return unit;
}

fn caller(value: own i32) -> result: own unit pure {
  let exact_result = exact(value: value);
  need_same(left: exact_result, right: value);
  let weak_result = weak(value: value);
  need_ordered(left: weak_result, right: value);
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_complete(source);
}

#[test]
fn a_concrete_generic_call_reports_only_source_stable_identities() {
    let source = format!(
        r#"fn identity<T: Int>(value: own T) -> result: own T pure {{
  return value;
}}

fn need_same(left: own i32, right: own i32) -> result: own unit pure contract {{
  requires ieq(left, right);
}} {{
  return unit;
}}

fn caller(value: own i32) -> result: own unit traps {{
  let returned = identity<i32>(value: value);
  claim reviewed_concrete: ieq(returned, value) because "{REVIEW}";
  need_same(left: returned, right: value);
  return unit;
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );

    let render_issue = || {
        let mut rendered = None;
        with_semantics(source.as_bytes(), |outcome| {
            let SemanticOutcome::SourceIssue { issue } = outcome else {
                panic!("the concrete generic claim must reject: {outcome:?}");
            };
            assert_eq!(issue.rule(), SemanticRule::Clm1);
            let SemanticIssueKind::NonLocalClaim(detail) = issue.kind() else {
                panic!(
                    "expected the structured locality issue, got {:?}",
                    issue.kind()
                );
            };
            assert_eq!(detail.name, "reviewed_concrete");
            assert_eq!(detail.carrier, "returned");
            let ClaimBoundaryResultDetail::UserCall {
                declaration: _,
                callee,
            } = &detail.boundary
            else {
                panic!("expected a user-call boundary, got {:?}", detail.boundary);
            };
            assert_eq!(callee, "identity");
            rendered = Some(format!("{issue:#?}"));
        });
        rendered.expect("one deterministic locality issue")
    };

    let expected = render_issue();
    for scratch_identity in ["$instance$", "FunctionId(", "NominalId(", "BindingId("] {
        assert!(
            !expected.contains(scratch_identity),
            "concrete generic locality diagnostic leaked {scratch_identity}: {expected}"
        );
    }
    for _ in 0..2 {
        assert_eq!(render_issue(), expected);
    }
}

#[test]
fn named_constant_array_operations_do_not_break_a_local_residual_claim() {
    let source = format!(
        r#"const values: array<u8, 4> =[1_u8, 2_u8, 3_u8, 4_u8];

fn read(cursor: own u64) -> result: own u8 traps {{
  let length = len(values);
  let first = values[0_u64];
  region 'view {{
    let view = slice_of(&'view values);
    let view_length = len(view);
  }}
  claim local_constant_root: ilt(cursor, length) because "{LOCAL_REVIEW}";
  return values[cursor];
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_complete(source.as_bytes());
}

#[test]
fn a_local_enum_tag_is_not_tainted_by_its_boundary_payload() {
    let source = format!(
        r#"fn opaque(value: own u64) -> result: own u64 pure {{
  return value;
}}

fn read(values: own array<u8, 4>, input: own u64, cursor: own u64) -> result: own u8 traps {{
  let returned = opaque(value: input);
  let local_tag = Ok<u64, u64>(value: returned);
  match local_tag {{
    Ok(value: ignored_boundary_payload) => {{
      claim local_tag_control: ilt(cursor, 4_u64) because "{LOCAL_REVIEW}";
      return values[cursor];
    }}
    Err(error: ignored_local_payload) => {{
      return 0_u8;
    }}
  }}
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_complete(source.as_bytes());
}

/// v0.39 narrowed [CLM-1]: standing on a boundary-selected edge is not itself
/// a selection.  The condition chose which arm runs and no operand of the
/// predicate, so a claim over an ordinary parameter inside the arm is local.
/// Under v0.38 this fixture was refused with carrier `cursor`.
#[test]
fn a_local_claim_inside_a_selected_arm_is_admitted() {
    let source = format!(
        r#"fn hidden_true() -> result: own Bool pure {{
  return True();
}}

fn read(values: own array<u8, 4>, cursor: own u64) -> result: own u8 traps {{
  let condition = hidden_true();
  if condition {{
    claim reviewed_arm_control: ilt(cursor, 4_u64) because "{LOCAL_REVIEW}";
    return values[cursor];
  }} else {{
    return 0_u8;
  }}
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_complete(source.as_bytes());
}

/// One arm returns, so the continuation is reached only through the other
/// edge.  `cursor` is untouched on that edge, so both incoming reaching
/// definitions of it are the same definition and the selector chose nothing
/// about it.  Under v0.38 the continuation itself carried the witness.
#[test]
fn a_partial_arm_continuation_leaves_an_untouched_local_alone() {
    let source = format!(
        r#"fn hidden_true() -> result: own Bool pure {{
  return True();
}}

fn read(values: own array<u8, 4>, cursor: own u64) -> result: own u8 traps {{
  let condition = hidden_true();
  if condition {{
    return 0_u8;
  }} else {{
    let still_local = cursor;
  }}
  claim reviewed_continuation_control: ilt(cursor, 4_u64) because "{LOCAL_REVIEW}";
  return values[cursor];
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_complete(source.as_bytes());
}

/// The same narrowing through a nested branch: no arm writes `cursor`, so no
/// merge selects it however the arms leave.
#[test]
fn a_nested_partial_return_leaves_an_untouched_local_alone() {
    let source = format!(
        r#"fn hidden_true() -> result: own Bool pure {{
  return True();
}}

fn read(values: own array<u8, 4>, cursor: own u64, leave: own Bool) -> result: own u8 traps {{
  let condition = hidden_true();
  if condition {{
    if leave {{
      return 0_u8;
    }} else {{
      let kept = 0_u8;
    }}
  }} else {{
    let skipped = 0_u8;
  }}
  claim reviewed_nested_return: ilt(cursor, 4_u64) because "{LOCAL_REVIEW}";
  return values[cursor];
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_complete(source.as_bytes());
}

/// A claim component whose supports are a named const and a literal reads
/// nothing a boundary returned.  v0.38 refused it on the occurrence's control
/// authority alone, which is exactly the position clause v0.39 repealed.  With
/// that clause gone the occurrence is local and reaches the next judgment, so
/// this fixture now fails [CLM-2] as a redundant claim rather than [CLM-1] as
/// a non-local one.  That is the honest verdict for a component the checker
/// decides on its own: a control-only [CLM-1] rejection no longer exists.
#[test]
fn a_local_named_const_component_reaches_the_redundancy_judgment() {
    let source = format!(
        r#"const four: u64 = 4_u64;

fn hidden_true() -> result: own Bool pure {{
  return True();
}}

fn probe() -> result: own unit traps {{
  let condition = hidden_true();
  if condition {{
    claim reviewed_constant_control: ieq(four, 4_u64) because "{LOCAL_REVIEW}";
    return unit;
  }} else {{
    return unit;
  }}
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    with_semantics(source.as_bytes(), |outcome| {
        let SemanticOutcome::SourceIssue { issue } = outcome else {
            panic!("expected the local component to reach CLM-2, got {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Clm2);
    });
}

#[test]
fn a_returned_unique_holder_survives_a_local_referent_overwrite() {
    let source = format!(
        r#"fn passthru['r](value: &uniq 'r u64) -> result: &uniq 'r u64 pure {{
  return &uniq 'r deref(value);
}}

fn read(values: own array<u8, 4>, cursor: own u64, local: own u64) -> result: own u8 traps {{
  region 'write {{
    let holder = passthru<'write>(value: &uniq 'write cursor);
    set deref(holder) = local;
    let observed = deref(holder);
    claim reviewed_overwritten_holder: ilt(observed, 4_u64) because "{REVIEW}";
    return values[observed];
  }}
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_user_call_locality(
        source.as_bytes(),
        "reviewed_overwritten_holder",
        0,
        "observed",
        "passthru",
    );
}

#[test]
fn an_intermediate_reborrow_holder_cannot_launder_a_boundary_result() {
    use super::super::claim_locality::{BoundaryResultKind, ClaimAuthorityAnalysis};
    use super::super::entailment::FunctionEntailment;
    use super::super::model::{
        BindingId, CheckedBodyDisposition, CheckedExpression, CheckedFunction, CheckedMode,
        CheckedParameter, CheckedResultBorrow, CheckedResultStateOrigin, CheckedStatement,
        CheckedType, CheckedValue, FunctionId, IntegerType,
    };
    use crate::{DeclarationId, NodePath};

    fn path(component: u32) -> NodePath {
        NodePath {
            components: vec![component],
        }
    }

    let integer = CheckedType::Integer(IntegerType::U64);
    let call = path(1);
    let claim = path(4);
    let function = CheckedFunction {
        id: FunctionId(0),
        declaration: DeclarationId::from_index(0).expect("the test declaration fits"),
        name: "probe".to_owned(),
        symbol: "probe".to_owned(),
        deny_claims_marker: None,
        parameters: vec![CheckedParameter {
            name: "actual".to_owned(),
            declaration: DeclarationId::from_index(1).expect("the test declaration fits"),
            node_path: path(0),
            binding: BindingId(0),
            mode: CheckedMode::Own,
            ty: integer,
            slice_origins: Vec::new(),
        }],
        result_mode: CheckedMode::Own,
        result: CheckedType::Unit,
        result_state_origin: CheckedResultStateOrigin::NoState,
        slice_return_ceiling: Vec::new(),
        declared_traps: true,
        declared_allocates_heap: false,
        declared_state_writes: Vec::new(),
        target_action: crate::TargetAction::INLINE,
        requirements: Vec::new(),
        postconditions: Vec::new(),
        body: vec![
            CheckedStatement::Let {
                node_path: path(1),
                binding: BindingId(1),
                value: CheckedExpression::UserCall {
                    function: FunctionId(1),
                    call: call.clone(),
                    argument_nodes: Vec::new(),
                    arguments: Vec::new(),
                    goal_arguments: Vec::new(),
                    goal_regions: Vec::new(),
                    requirements: Vec::new(),
                    result: integer,
                    slice_origins: Vec::new(),
                    result_borrow: Some(CheckedResultBorrow {
                        binding: BindingId(0),
                        fields: Vec::new(),
                    }),
                },
            },
            CheckedStatement::Let {
                node_path: path(2),
                binding: BindingId(2),
                value: CheckedExpression::ReborrowAddressed {
                    carrier: path(2),
                    binding: BindingId(1),
                    ty: integer,
                },
            },
            CheckedStatement::Let {
                node_path: path(3),
                binding: BindingId(3),
                value: CheckedExpression::DerefAddressed {
                    carrier: path(3),
                    binding: BindingId(2),
                    ty: integer,
                },
            },
            CheckedStatement::Claim {
                name: "reviewed_reborrow".to_owned(),
                predicate: "True()".to_owned(),
                justification: super::super::model::ClaimJustification {
                    raw: String::new(),
                    premises: String::new(),
                    derivation: String::new(),
                    conclusion: String::new(),
                    checker_gap: String::new(),
                    consumers: String::new(),
                },
                condition: CheckedExpression::Constant(CheckedValue::Bool(true)),
                site: super::super::model::ClaimSite {
                    rule_id: "CLM-1",
                    message: "reviewed_reborrow".to_owned(),
                    function: "probe".to_owned(),
                    node_path: claim.clone(),
                },
            },
        ],
        body_disposition: CheckedBodyDisposition::default(),
        entailment: FunctionEntailment::default(),
    };

    let analysis = ClaimAuthorityAnalysis::analyze(&function, &[])
        .expect("the synthetic checked holder chain must analyze");
    let witness = analysis
        .witness(&claim, BindingId(3), &[], false)
        .expect("the intermediate call-result holder must remain visible");
    assert_eq!(witness.kind, BoundaryResultKind::UserCall(FunctionId(1)));
    assert_eq!(witness.call, call);
}

#[test]
fn a_whole_box_replace_keeps_the_owner_shape_well_typed() {
    let source = format!(
        r#"struct Pair {{
  value: u64;
}}

fn read(values: own array<u8, 4>, cursor: own u64) -> result: own u8 allocates(heap), traps {{
  let first_value = Pair(value: 0_u64);
  let first = box_new(move first_value);
  let second_value = Pair(value: 1_u64);
  let second = box_new(move second_value);
  let old = replace first = move second;
  let observed = deref(first).value;
  claim local_after_owner_replace: ilt(cursor, 4_u64) because "{LOCAL_REVIEW}";
  return values[cursor];
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_complete(source.as_bytes());
}

/// The endpoint selects how many iterations run and, at the loop head, which
/// definition of a loop-carried value arrives.  It selects no operand of a
/// claim over an untouched parameter, so the body's claim is local under
/// v0.39; v0.38 refused it on the body's endpoint control frame.
#[test]
fn a_counted_endpoint_leaves_an_untouched_local_in_the_body_alone() {
    let source = format!(
        r#"fn endpoint(value: own u64) -> result: own u64 pure {{
  return value;
}}

fn read(values: own array<u8, 4>, input: own u64, cursor: own u64) -> result: own u8 traps {{
  let upper = endpoint(value: input);
  for @items item in 0_u64..upper {{
    claim reviewed_counted_body: ilt(cursor, 4_u64) because "{LOCAL_REVIEW}";
    return values[cursor];
  }}
  return 0_u8;
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_complete(source.as_bytes());
}

#[test]
fn a_counted_endpoint_does_not_taint_an_untouched_post_loop_local() {
    let source = format!(
        r#"fn endpoint(value: own u64) -> result: own u64 pure {{
  return value;
}}

fn read(values: own array<u8, 4>, input: own u64, cursor: own u64) -> result: own u8 traps {{
  let upper = endpoint(value: input);
  for @items item in 0_u64..upper {{
    let observed = item;
  }}
  claim local_after_counted: ilt(cursor, 4_u64) because "{LOCAL_REVIEW}";
  return values[cursor];
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_complete(source.as_bytes());
}

/// A body that returns makes the post-loop path endpoint-dependent, but not
/// the value of a parameter no iteration wrote.
#[test]
fn a_counted_endpoint_leaves_a_post_loop_local_alone_when_the_body_returns() {
    let source = format!(
        r#"fn endpoint(value: own u64) -> result: own u64 pure {{
  return value;
}}

fn read(values: own array<u8, 4>, input: own u64, cursor: own u64) -> result: own u8 traps {{
  let upper = endpoint(value: input);
  for @items item in 0_u64..upper {{
    return 0_u8;
  }}
  claim reviewed_counted_exit: ilt(cursor, 4_u64) because "{LOCAL_REVIEW}";
  return values[cursor];
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_complete(source.as_bytes());
}

/// The same with a partial return inside the body.
#[test]
fn a_counted_endpoint_leaves_a_post_loop_local_alone_with_a_partial_return() {
    let source = format!(
        r#"fn endpoint(value: own u64) -> result: own u64 pure {{
  return value;
}}

fn read(values: own array<u8, 4>, input: own u64, cursor: own u64, stop: own Bool) -> result: own u8 traps {{
  let upper = endpoint(value: input);
  for @items item in 0_u64..upper {{
    if stop {{
      return 0_u8;
    }} else {{
      let observed = item;
    }}
  }}
  claim reviewed_partial_counted_exit: ilt(cursor, 4_u64) because "{LOCAL_REVIEW}";
  return values[cursor];
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_complete(source.as_bytes());
}

/// The same with a propagating body, whose implicit error edge leaves the
/// function.
#[test]
fn a_counted_endpoint_leaves_a_post_loop_local_alone_with_propagation() {
    let source = format!(
        r#"fn endpoint(value: own u64) -> result: own u64 pure {{
  return value;
}}

fn read(values: own array<u8, 4>, input: own u64, cursor: own u64) -> result: own Result<u8, Overflow> traps {{
  let upper = endpoint(value: input);
  for @items item in 0_u64..upper {{
    let ignored = propagate cursor +checked 1_u64;
  }}
  claim reviewed_propagating_counted_exit: ilt(cursor, 4_u64) because "{LOCAL_REVIEW}";
  let observed = values[cursor];
  return Ok<u8, Overflow>(value: observed);
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_complete(source.as_bytes());
}

/// A boundary-selected break decides whether the loop is left, not the value
/// of a parameter no iteration wrote.
#[test]
fn an_ordinary_loop_leaves_an_untouched_local_alone_after_a_selected_break() {
    let source = format!(
        r#"fn hidden_true() -> result: own Bool pure {{
  return True();
}}

fn read(values: own array<u8, 4>, cursor: own u64) -> result: own u8 traps {{
  let condition = hidden_true();
  loop @again {{
    if condition {{
      break @again;
    }} else {{
      return 0_u8;
    }}
  }}
  claim reviewed_break_control: ilt(cursor, 4_u64) because "{LOCAL_REVIEW}";
  return values[cursor];
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_complete(source.as_bytes());
}

#[test]
fn exhaustive_breaks_to_one_loop_do_not_taint_its_continuation() {
    let source = format!(
        r#"fn hidden_true() -> result: own Bool pure {{
  return True();
}}

fn read(values: own array<u8, 4>, cursor: own u64) -> result: own u8 traps {{
  let condition = hidden_true();
  loop @once {{
    if condition {{
      break @once;
    }} else {{
      break @once;
    }}
  }}
  claim local_after_exhaustive_break: ilt(cursor, 4_u64) because "{LOCAL_REVIEW}";
  return values[cursor];
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_complete(source.as_bytes());
}

#[test]
fn exhaustive_breaks_with_local_prefixes_do_not_taint_their_continuation() {
    let source = format!(
        r#"fn hidden_true() -> result: own Bool pure {{
  return True();
}}

fn read(values: own array<u8, 4>, cursor: own u64) -> result: own u8 traps {{
  let condition = hidden_true();
  loop @once {{
    if condition {{
      let harmless = 0_u8;
      break @once;
    }} else {{
      break @once;
    }}
  }}
  claim local_after_exhaustive_break_prefix: ilt(cursor, 4_u64) because "{LOCAL_REVIEW}";
  return values[cursor];
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_complete(source.as_bytes());
}

#[test]
fn an_outer_call_result_reports_its_own_boundary_event() {
    let source = format!(
        r#"fn first(value: own u64) -> result: own u64 pure {{
  return value;
}}

fn second(value: own u64) -> result: own u64 pure {{
  return value;
}}

fn read(values: own array<u8, 4>, input: own u64) -> result: own u8 traps {{
  let inner = first(value: input);
  let outer = second(value: inner);
  claim reviewed_outer_result: ilt(outer, 4_u64) because "{REVIEW}";
  return values[outer];
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_user_call_locality(
        source.as_bytes(),
        "reviewed_outer_result",
        0,
        "outer",
        "second",
    );
}

#[test]
fn earlier_claim_formation_precedes_later_claim_locality() {
    let source = format!(
        r#"fn opaque(value: own u64) -> result: own u64 pure {{
  return value;
}}

fn probe(left: own Bool, right: own Bool, input: own u64) -> result: own unit traps {{
  let unsupported = bxor(left, right);
  claim early_formation: unsupported because "{LOCAL_REVIEW}";
  let returned = opaque(value: input);
  claim later_locality: ilt(returned, 4_u64) because "{REVIEW}";
  return unit;
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    with_semantics(source.as_bytes(), |outcome| {
        let SemanticOutcome::SourceIssue { issue } = outcome else {
            panic!("the earlier formation failure must reject: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Clm1);
        let SemanticIssueKind::InvalidClaim(detail) = issue.kind() else {
            panic!(
                "expected an invalid-claim formation payload: {:?}",
                issue.kind()
            );
        };
        assert_eq!(detail.name, "early_formation");
        assert_eq!(detail.classification, "unsupported canonical formation");
    });
}

#[test]
fn a_claim_function_resolves_fields_through_an_opaque_unique_struct_parameter() {
    let source = format!(
        r#"struct Pool {{
  values: buffer<u64>;
  count: u64;
}}

fn write['r](pool: &uniq 'r Pool, seed: own u64, witnesses: own array<u64, 4>) -> result: own u64 reads(pool.values, pool.count), writes(pool.values), traps {{
  let slot = deref(pool).count;
  let room = len(deref(pool).values);
  let bounded = seed % 4_u64;
  let reviewed = ilt(bounded, 4_u64);
  claim local_buffer_field_walk: reviewed because "{LOCAL_REVIEW}";
  let observed = witnesses[bounded];
  let slot_in_room = ilt(slot, room);
  if slot_in_room {{
    set deref(pool).values[slot] = observed;
  }} else {{
    let unchanged = observed;
  }}
  return observed;
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_complete(source.as_bytes());
}

/// The narrowing keeps the selection the join performs.  One arm writes the
/// place and the other does not, so the reconvergence has two different
/// reaching definitions of it and the call result chose which one arrives.
#[test]
fn a_write_on_one_arm_only_is_selected_at_the_join() {
    let source = format!(
        r#"fn hidden_true() -> result: own Bool pure {{
  return True();
}}

fn read(values: own array<u8, 4>, unused: own u64) -> result: own u8 traps {{
  let cursor = 3_u64;
  let condition = hidden_true();
  if condition {{
    set cursor = 0_u64;
  }} else {{
    let untouched = 0_u8;
  }}
  claim reviewed_one_arm_write: ilt(cursor, 4_u64) because "{REVIEW}";
  return values[cursor];
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_user_call_locality(
        source.as_bytes(),
        "reviewed_one_arm_write",
        0,
        "cursor",
        "hidden_true",
    );
}

/// A definition formed after that same join, from literals alone, is a new
/// reaching definition no edge chose, so it is local although the write above
/// it is not.  This is the differential-fuzz shape at unit scale.
#[test]
fn a_local_definition_after_a_selected_join_is_local() {
    let source = format!(
        r#"fn hidden_true() -> result: own Bool pure {{
  return True();
}}

fn read(values: own array<u8, 4>, unused: own u64) -> result: own u8 traps {{
  let cursor = 3_u64;
  let condition = hidden_true();
  if condition {{
    set cursor = 0_u64;
  }} else {{
    let untouched = 0_u8;
  }}
  let position = 2_u64 % 4_u64;
  claim reviewed_post_join_local: ilt(position, 4_u64) because "{LOCAL_REVIEW}";
  return values[position];
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_complete(source.as_bytes());
}

/// Loop-carried state a boundary-selected iteration updates keeps the
/// endpoint's witness: the loop head and the loop exit each merge the
/// definition written before the loop with the one an iteration wrote.
#[test]
fn a_counted_endpoint_selects_loop_carried_state() {
    let source = format!(
        r#"fn endpoint(value: own u64) -> result: own u64 pure {{
  return imin(value, 2_u64);
}}

fn read(values: own array<u8, 4>, input: own u64) -> result: own u8 traps {{
  let upper = endpoint(value: input);
  let cursor = 0_u64;
  for @steps step in 0_u64..upper {{
    set cursor = 1_u64;
  }}
  claim reviewed_carried_update: ilt(cursor, 4_u64) because "{REVIEW}";
  return values[cursor];
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_user_call_locality(
        source.as_bytes(),
        "reviewed_carried_update",
        0,
        "cursor",
        "endpoint",
    );
}

/// The same for an ordinary loop whose break the boundary result selects: the
/// loop exit merges the entry definition with the one the body wrote.
#[test]
fn an_ordinary_loop_selects_state_its_iterations_wrote() {
    let source = format!(
        r#"fn hidden_true() -> result: own Bool pure {{
  return True();
}}

fn read(values: own array<u8, 4>, unused: own u64) -> result: own u8 traps {{
  let cursor = 3_u64;
  let condition = hidden_true();
  loop @again {{
    if condition {{
      break @again;
    }} else {{
      set cursor = 0_u64;
    }}
  }}
  claim reviewed_loop_write: ilt(cursor, 4_u64) because "{REVIEW}";
  return values[cursor];
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_user_call_locality(
        source.as_bytes(),
        "reviewed_loop_write",
        0,
        "cursor",
        "hidden_true",
    );
}

/// A matching binder is the payload its own tag selected and stays non-local
/// in the arm that introduced it.
#[test]
fn a_matching_binder_is_selected_by_its_own_tag() {
    let source = format!(
        r#"fn measure(value: own u64) -> result: own Result<u64, Overflow> pure {{
  return value +checked 1_u64;
}}

fn read(values: own array<u8, 4>, input: own u64) -> result: own u8 traps {{
  match measure(value: input) {{
    Ok(value: measured) => {{
      claim reviewed_selected_payload: ilt(measured, 4_u64) because "{REVIEW}";
      return values[measured];
    }}
    Err(error: overflow) => {{
      return 0_u8;
    }}
  }}
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_user_call_locality(
        source.as_bytes(),
        "reviewed_selected_payload",
        0,
        "measured",
        "measure",
    );
}

/// A value construct whose own selector is local delivers a local value even
/// inside a boundary-selected arm: the selection that chose the delivery is
/// the inner condition, and the outer edge chose no operand of it.
#[test]
fn a_local_delivery_inside_a_selected_arm_is_local() {
    let source = format!(
        r#"fn hidden_true() -> result: own Bool pure {{
  return True();
}}

fn read(values: own array<u8, 4>, choose: own Bool) -> result: own u8 traps {{
  let condition = hidden_true();
  if condition {{
    let picked = if choose {{
      give 0_u64;
    }} else {{
      give 1_u64;
    }}
    claim reviewed_inner_delivery: ilt(picked, 4_u64) because "{LOCAL_REVIEW}";
    return values[picked];
  }} else {{
    return 0_u8;
  }}
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_complete(source.as_bytes());
}
