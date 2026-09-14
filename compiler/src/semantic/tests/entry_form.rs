//! [FN-7] ordinary function declarations and build-selected entry points.
//!
//! Entry names, arguments and results confer no source acceptance privilege.
//! Removed kind/label assertions are recorded in the C2 CASES register.

use super::{assert_rule, assert_rule_kind, with_semantics};
use crate::{SemanticIssueKind, SemanticOutcome, SemanticRule};

fn assert_complete(source: &[u8]) {
    with_semantics(source, |outcome| {
        assert!(
            matches!(outcome, SemanticOutcome::Complete(_)),
            "{outcome:?}"
        );
    });
}

#[test]
fn ordinary_prelude_calls_are_named_in_declared_order() {
    let declared = |callee: &str, parameters: &[&str]| SemanticIssueKind::InvalidNamedArguments {
        callee: callee.to_owned(),
        declared_parameters: parameters.iter().map(|name| (*name).to_owned()).collect(),
    };
    assert_rule(
        b"fn main() -> status: own ExitStatus pure {\n  return exit_status(value: 0_u8);\n}\n",
        SemanticRule::Gram11,
        declared("exit_status", &["code"]),
    );
    assert_rule(
        b"fn main() -> status: own ExitStatus pure {\n  return exit_status(0_u8);\n}\n",
        SemanticRule::Gram11,
        declared("exit_status", &["code"]),
    );
    assert_rule(
        b"fn main() -> status: own ExitStatus pure {\n  return exit_status();\n}\n",
        SemanticRule::Gram11,
        declared("exit_status", &["code"]),
    );
    assert_rule(
        b"fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8, extra: 0_u8);\n}\n",
        SemanticRule::Gram11,
        declared("exit_status", &["code"]),
    );
    with_semantics(
        b"fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        |outcome| {
            assert!(
                matches!(outcome, SemanticOutcome::Complete(_)),
                "the ordinary named call must check: {outcome:?}"
            );
        },
    );
}

#[test]
fn arg_get_calls_are_checked_by_the_same_general_rule() {
    let declared = SemanticIssueKind::InvalidNamedArguments {
        callee: "arg_get".to_owned(),
        declared_parameters: vec!["args".to_owned(), "position".to_owned()],
    };
    assert_rule(
        b"fn probe(args: &Args) -> result: own unit reads(args) {\n  let value = arg_get(args: args);\n  return unit;\n}\n\nfn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Gram11,
        declared.clone(),
    );
    assert_rule(
        b"fn probe(args: &Args) -> result: own unit reads(args) {\n  let value = arg_get(args: args, offset: 0_u64);\n  return unit;\n}\n\nfn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Gram11,
        declared,
    );
}

#[test]
fn a_bundle_without_a_build_selected_entry_is_ordinary_source() {
    assert_complete(b"fn helper() -> result: own unit pure {\n  return unit;\n}\n");
}

#[test]
fn main_has_ordinary_parameters_results_regions_and_generics() {
    for source in [
        &b"fn main() -> result: own unit pure {\n  return unit;\n}\n"[..],
        &b"fn main(value: own i32) -> result: own i32 pure {\n  return value;\n}\n"[..],
        &b"fn main<T: affine>(value: own T) -> result: own T pure {\n  return move value;\n}\n"[..],
        &b"fn main['s](heap: own Heap<'s>) -> result: own unit pure {\n  return unit;\n}\n"[..],
        &b"fn main(env: own Args, again: own Args) -> result: own unit pure {\n  return unit;\n}\n"[..],
        &b"fn main(args: own DirectoryRead) -> result: own DirectoryRead pure {\n  return move args;\n}\n"[..],
    ] {
        assert_complete(source);
    }
}

#[test]
fn a_main_requirement_is_an_ordinary_contract_not_a_launcher_promise() {
    assert_complete(include_bytes!(
        "../../../../tests/conformance/cases/fn8-neg-entry-contract.wf"
    ));
    let source = b"fn main(value: own u64) -> result: own u64 pure contract {\n  requires value < 4_u64;\n} {\n  return value;\n}\n\nfn call() -> result: own u64 pure {\n  return main(value: 4_u64);\n}\n";
    assert_rule_kind(source, SemanticRule::Fn8, |_| true);
}

#[test]
fn a_source_call_to_main_uses_the_ordinary_function_contract() {
    assert_complete(b"fn main(value: own u64) -> result: own u64 pure {\n  return value;\n}\n\nfn helper() -> result: own u64 pure {\n  return main(value: 7_u64);\n}\n");
    assert_complete(include_bytes!(
        "../../../../tests/conformance/cases/reject-sysentry-call-to-kind-entry.wf"
    ));
}

#[test]
fn unexhibited_main_effects_are_still_rejected_by_eff2() {
    for source in [
        &b"fn main['s](heap: own Heap<'s>) -> result: own unit allocates(heap) {\n  return unit;\n}\n"
            [..],
        &b"fn probe(args: own Args) -> result: own unit reads(args) {\n  return unit;\n}\n"[..],
    ] {
        assert_rule_kind(source, SemanticRule::Eff2, |kind| {
            matches!(kind, SemanticIssueKind::EffectMismatch { .. })
        });
    }
}

#[test]
fn prelude_inputs_are_a_normal_linear_struct() {
    assert_complete(
        b"fn relay(inputs: own Inputs) -> result: own Inputs pure {\n  return move inputs;\n}\n",
    );
    assert_rule_kind(
        b"fn discard(inputs: own Inputs) -> result: own unit pure {\n  return unit;\n}\n",
        SemanticRule::Prov6,
        |_| true,
    );
}
