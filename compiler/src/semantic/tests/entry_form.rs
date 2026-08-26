//! [FN-7] entry-form admission and the [GRAM-11] system-call argument rule.
//!
//! Every rejection here also pins the exact `SourceNode` [FN-7] names for it,
//! because that location table is normative [DIAG-1] and a rule that reports
//! the whole declaration for a one-parameter defect is not the same
//! diagnostic.

use crate::{SemanticIssueKind, SemanticLocation, SemanticOutcome, SemanticRule};

use super::{assert_rule, with_semantics};

/// Asserts the rule, premise, and the exact source bytes the location selects.
fn assert_rule_at(source: &[u8], rule: SemanticRule, kind: SemanticIssueKind, located: &[u8]) {
    with_semantics(source, |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("expected {rule:?}/{kind:?}, got {outcome:?}");
        };
        assert_eq!(issue.rule(), rule);
        assert_eq!(issue.kind(), &kind);
        let SemanticLocation::SourceNode(_, coordinate) = issue.location() else {
            panic!(
                "{rule:?} must use a source node, got {:?}",
                issue.location()
            );
        };
        let start = usize::try_from(coordinate.start().value()).expect("test offset fits usize");
        let end = usize::try_from(coordinate.end().value()).expect("test offset fits usize");
        assert_eq!(
            std::str::from_utf8(&source[start..end]),
            std::str::from_utf8(located)
        );
    });
}

fn invalid_label(label: &str) -> SemanticIssueKind {
    SemanticIssueKind::InvalidStandardInputLabel {
        label: label.to_owned(),
        declared_labels: vec![
            "command.args".to_owned(),
            "command.cwd".to_owned(),
            "command.stdout".to_owned(),
            "command.stderr".to_owned(),
        ],
    }
}

#[test]
fn an_unmarked_main_is_not_an_alternate_entry_form() {
    // SYS-3 makes these system names resolvable even without the marker, but
    // it does not weaken FN-7: this declaration is still not an entry.
    let source = b"fn main(command.args as args: own Args) -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n";
    assert_rule_at(
        source,
        SemanticRule::Fn7,
        SemanticIssueKind::InvalidMain,
        b"fn main(command.args as args: own Args) -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}",
    );
}

#[test]
fn the_no_input_command_entry_admits_every_effect_subset() {
    // With no formal capability there is no legal IDENT subject for reads or
    // writes. FN-7 admits every canonical subset of the remaining command
    // categories; an unexhibited admitted row is EFF-2's later judgment.
    for row in [
        &b"pure"[..],
        &b"allocates(heap)"[..],
        &b"traps"[..],
        &b"allocates(heap), traps"[..],
    ] {
        let mut source = b"command fn main() -> status: own ExitStatus ".to_vec();
        source.extend_from_slice(row);
        source.extend_from_slice(b" {\n  return exit_status(code: 0_u8);\n}\n");
        with_semantics(&source, |outcome| {
            if let SemanticOutcome::SourceIssue { issue, .. } = &outcome {
                assert_ne!(
                    issue.rule(),
                    SemanticRule::Fn7,
                    "FN-7 must admit the row {:?}: {outcome:?}",
                    std::str::from_utf8(row)
                );
            }
        });
    }
}

#[test]
fn the_command_entry_has_no_source_call_route() {
    let source = b"fn helper() -> result: own unit pure {\n  main();\n  return unit;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n";
    assert_rule_at(
        source,
        SemanticRule::Fn7,
        SemanticIssueKind::CallToKindDeclaringEntry {
            entry: "main".to_owned(),
        },
        b"main()",
    );
}

#[test]
fn a_missing_entry_is_the_one_bundle_root_rejection() {
    with_semantics(
        b"fn helper() -> result: own unit pure {\n  return unit;\n}\n",
        |outcome| {
            let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
                panic!("a unit with no entry must be rejected: {outcome:?}");
            };
            assert_eq!(issue.rule(), SemanticRule::Fn7);
            assert_eq!(issue.kind(), &SemanticIssueKind::MissingMain);
            assert!(
                matches!(issue.location(), SemanticLocation::BundleRoot(_)),
                "a missing entry has no offending declaration: {:?}",
                issue.location()
            );
        },
    );
}

#[test]
fn the_entry_is_nongeneric_and_declares_no_region_parameter() {
    assert_rule_at(
        b"command fn main<T>() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Fn7,
        SemanticIssueKind::InvalidMain,
        b"<T>",
    );
    assert_rule_at(
        b"command fn main['a]() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Fn7,
        SemanticIssueKind::InvalidMain,
        b"['a]",
    );
}

#[test]
fn a_missing_command_marker_outranks_legacy_signature_details() {
    assert_rule_at(
        b"fn main() -> result: own i32 pure {\n  return 0_i32;\n}\n",
        SemanticRule::Fn7,
        SemanticIssueKind::InvalidMain,
        b"fn main() -> result: own i32 pure {\n  return 0_i32;\n}",
    );
    assert_rule_at(
        b"fn main(value: own i32) -> result: own unit pure {\n  return unit;\n}\n",
        SemanticRule::Fn7,
        SemanticIssueKind::InvalidMain,
        b"fn main(value: own i32) -> result: own unit pure {\n  return unit;\n}",
    );
}

#[test]
fn admitted_but_unexhibited_entry_effects_reach_eff2() {
    // Allocation and trap remain admitted entry categories. These bodies do
    // not exhibit them, so they pass FN-7 and reject later under EFF-2.
    assert_rule(
        b"command fn main() -> status: own ExitStatus allocates(heap) {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Eff2,
        SemanticIssueKind::EffectMismatch,
    );
    assert_rule(
        b"command fn main() -> status: own ExitStatus traps {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Eff2,
        SemanticIssueKind::EffectMismatch,
    );
    // A legal capability subject on a non-entry declaration is likewise an
    // ordinary declared-but-unexhibited mismatch.
    assert_rule(
        b"fn probe(args: own Args) -> result: own unit reads(args) {\n  return unit;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Eff2,
        SemanticIssueKind::EffectMismatch,
    );
}

#[test]
fn only_the_entry_may_declare_a_program_kind() {
    assert_rule_at(
        b"command fn helper() -> result: own unit pure {\n  return unit;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Fn7,
        SemanticIssueKind::NonEntryProgramKind {
            function: "helper".to_owned(),
        },
        b"command",
    );
}

#[test]
fn an_admitted_command_entry_completes_semantic_checking() {
    // FN-7 admission succeeds for each of these, and the system semantic
    // path — [SYS-2] call typing and [EFF-2] attribution including the
    // release contribution — is implemented, so a `command` entry whose
    // declared row equals its exhibited row completes semantic checking.
    // The full-input entry exhibits `writes(cwd)` from DirectoryRead's
    // compiler-derived close; every other input release row is empty [SYS-5].
    for source in [
        &b"command fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n"[..],
        &b"command fn main(command.args as args: own Args, command.cwd as cwd: own DirectoryRead, command.stdout as out: own Output, command.stderr as err: own Output) -> status: own ExitStatus writes(cwd) {\n  return exit_status(code: 0_u8);\n}\n"[..],
        // A subset in strictly increasing table-ordinal order, skipping rows.
        &b"command fn main(command.args as args: own Args, command.stderr as err: own Output) -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n"[..],
    ] {
        with_semantics(source, |outcome| {
            assert!(
                matches!(outcome, SemanticOutcome::Complete(_)),
                "a row-exact command entry must check: {outcome:?}"
            );
        });
    }
}

#[test]
fn the_standard_input_table_is_closed_at_its_input_label_node() {
    // Unknown row.
    assert_rule_at(
        b"command fn main(command.env as env: own Args) -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Fn7,
        invalid_label("command.env"),
        b"command.env as",
    );
    // Repeated row: distinct binders do not make it two inputs, because
    // ordinal identity selects the supplied value.
    assert_rule_at(
        b"command fn main(command.args as args: own Args, command.args as again: own Args) -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Fn7,
        invalid_label("command.args"),
        b"command.args as",
    );
    // Out of table-ordinal order.
    assert_rule_at(
        b"command fn main(command.cwd as cwd: own DirectoryRead, command.args as args: own Args) -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Fn7,
        invalid_label("command.args"),
        b"command.args as",
    );
}

#[test]
fn two_inputs_of_one_type_remain_two_distinct_ordinals() {
    // `command.stdout` and `command.stderr` share one type; selecting them in
    // table order is admitted (and checks completely, since `Output`'s
    // logical source detach carries the empty release row), and selecting
    // them in reverse is not.
    with_semantics(
        b"command fn main(command.stdout as out: own Output, command.stderr as err: own Output) -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        |outcome| {
            assert!(
                matches!(outcome, SemanticOutcome::Complete(_)),
                "the in-order two-output entry must check: {outcome:?}"
            );
        },
    );
    assert_rule_at(
        b"command fn main(command.stderr as err: own Output, command.stdout as out: own Output) -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Fn7,
        invalid_label("command.stdout"),
        b"command.stdout as",
    );
}

#[test]
fn a_selected_input_equals_its_row_at_the_complete_param_node() {
    assert_rule_at(
        b"command fn main(command.args as args: own DirectoryRead) -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Fn7,
        SemanticIssueKind::InvalidStandardInput {
            label: "command.args".to_owned(),
            declared: "own Args",
        },
        b"command.args as args: own DirectoryRead",
    );
    // The label, not the written type, selects the row: `command.stderr`
    // written as `own Args` fails against row 3's `own Output`.
    assert_rule_at(
        b"command fn main(command.stderr as err: own Args) -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Fn7,
        SemanticIssueKind::InvalidStandardInput {
            label: "command.stderr".to_owned(),
            declared: "own Output",
        },
        b"command.stderr as err: own Args",
    );
}

#[test]
fn a_kind_declaring_entry_admits_no_unlabelled_value_parameter() {
    assert_rule_at(
        b"command fn main(value: own u8) -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Fn7,
        SemanticIssueKind::UnlabelledEntryParameter {
            parameter: "value".to_owned(),
        },
        b"value: own u8",
    );
}

#[test]
fn an_input_label_outside_the_entry_is_rejected_at_its_own_node() {
    // On another `fn_decl` of a kind-declaring unit.
    assert_rule_at(
        b"fn helper(command.args as args: own Args) -> result: own unit pure {\n  return unit;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Fn7,
        SemanticIssueKind::StandardInputLabelOutsideEntry {
            label: "command.args".to_owned(),
        },
        b"command.args as",
    );
    // Placement outranks whether the label would select a table row.
    assert_rule_at(
        b"fn helper(command.env as value: own i32) -> result: own unit pure {\n  return unit;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Fn7,
        SemanticIssueKind::StandardInputLabelOutsideEntry {
            label: "command.env".to_owned(),
        },
        b"command.env as",
    );
    // In a `fn_sig`, which [FN-7] names separately.
    assert_rule_at(
        b"contract Sink {\n  fn emit(command.stdout as value: own i32) -> result: own unit pure;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Fn7,
        SemanticIssueKind::StandardInputLabelOutsideEntry {
            label: "command.stdout".to_owned(),
        },
        b"command.stdout as",
    );
}

#[test]
fn a_command_entry_fixes_its_own_exit_status_result() {
    assert_rule_at(
        b"command fn main() -> result: own unit pure {\n  return unit;\n}\n",
        SemanticRule::Fn7,
        SemanticIssueKind::InvalidEntryResult {
            required: "own ExitStatus",
        },
        b"own unit",
    );
    assert_rule_at(
        b"command fn main() -> result: own Args pure {\n  return unit;\n}\n",
        SemanticRule::Fn7,
        SemanticIssueKind::InvalidEntryResult {
            required: "own ExitStatus",
        },
        b"own Args",
    );
}

#[test]
fn a_source_call_to_a_kind_declaring_entry_is_rejected_at_that_call() {
    assert_rule_at(
        b"command fn main() -> status: own ExitStatus pure {\n  return main();\n}\n",
        SemanticRule::Fn7,
        SemanticIssueKind::CallToKindDeclaringEntry {
            entry: "main".to_owned(),
        },
        b"main()",
    );
    assert_rule_at(
        b"fn helper() -> out: own unit pure {\n  main();\n  return unit;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Fn7,
        SemanticIssueKind::CallToKindDeclaringEntry {
            entry: "main".to_owned(),
        },
        b"main()",
    );
}

#[test]
fn system_operation_calls_are_named_in_declared_order() {
    let declared = |callee: &str, parameters: &[&str]| SemanticIssueKind::InvalidNamedArguments {
        callee: callee.to_owned(),
        declared_parameters: parameters.iter().map(|name| (*name).to_owned()).collect(),
    };
    // Misspelled parameter name.
    assert_rule(
        b"command fn main() -> status: own ExitStatus pure {\n  return exit_status(value: 0_u8);\n}\n",
        SemanticRule::Gram11,
        declared("exit_status", &["code"]),
    );
    // Positional operands are not admitted for a system operation.
    assert_rule(
        b"command fn main() -> status: own ExitStatus pure {\n  return exit_status(0_u8);\n}\n",
        SemanticRule::Gram11,
        declared("exit_status", &["code"]),
    );
    // Missing parameter.
    assert_rule(
        b"command fn main() -> status: own ExitStatus pure {\n  return exit_status();\n}\n",
        SemanticRule::Gram11,
        declared("exit_status", &["code"]),
    );
    // Extra parameter.
    assert_rule(
        b"command fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8, extra: 0_u8);\n}\n",
        SemanticRule::Gram11,
        declared("exit_status", &["code"]),
    );
    // The correctly spelled call is admitted here and the system semantic
    // path types it completely, so the unit checks.
    with_semantics(
        b"command fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        |outcome| {
            assert!(
                matches!(outcome, SemanticOutcome::Complete(_)),
                "the correctly spelled system call must check: {outcome:?}"
            );
        },
    );
}

#[test]
fn arg_get_calls_are_checked_by_the_same_general_rule() {
    // v0.19 renamed [SYS-2]'s `arg_get` parameter to `position` (the v0.18
    // spelling `index` was a fixed [GRAM-5] atom excluded from IDENT by
    // [FORM-3], so no complete legal call existed; task 0018). The general
    // GRAM-11 rule rejects every incomplete or misspelled call against the
    // repaired declared spelling.
    let declared = SemanticIssueKind::InvalidNamedArguments {
        callee: "arg_get".to_owned(),
        declared_parameters: vec!["args".to_owned(), "position".to_owned()],
    };
    assert_rule(
        b"fn probe['a](args: &'a Args) -> result: own unit reads('a) {\n  let value = arg_get<'a>(args: args);\n  return unit;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Gram11,
        declared.clone(),
    );
    assert_rule(
        b"fn probe['a](args: &'a Args) -> result: own unit reads('a) {\n  let value = arg_get<'a>(args: args, offset: 0_u64);\n  return unit;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Gram11,
        declared,
    );
}
