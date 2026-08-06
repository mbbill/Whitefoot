//! [FN-7] entry-form admission and the [GRAM-11] system-call argument rule.
//!
//! Every rejection here also pins the exact `SourceNode` [FN-7] names for it,
//! because that location table is normative [DIAG-1] and a rule that reports
//! the whole declaration for a one-parameter defect is not the same
//! diagnostic.

use crate::{
    SemanticIssueKind, SemanticLocation, SemanticOutcome, SemanticRule, UnsupportedSemanticFeature,
};

use super::super::model::CheckedEntryForm;
use super::{assert_rule, assert_unsupported, with_semantics};

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

fn invalid_kind(kind: &str) -> SemanticIssueKind {
    SemanticIssueKind::InvalidProgramKind {
        kind: kind.to_owned(),
        admitted_kinds: vec!["command".to_owned()],
    }
}

#[test]
fn the_unlabelled_entry_is_admitted_and_recorded_unchanged() {
    let source = b"fn main() -> own unit pure {\n  return unit;\n}\n";
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("the unlabelled entry must stay admitted: {outcome:?}");
        };
        match checked.entry_form() {
            CheckedEntryForm::Unlabelled => {}
            CheckedEntryForm::Command { inputs } => {
                panic!("an unlabelled entry must not record command inputs {inputs:?}");
            }
        }
    });
}

#[test]
fn the_unlabelled_entry_keeps_its_exact_four_effect_rows() {
    // [FN-7] admits exactly these four rows for this form. A body that does
    // not exhibit a declared row is [EFF-2]'s rejection, not FN-7's, so this
    // asserts only that the FN-7 judgment lets each one through.
    for row in [
        &b"pure"[..],
        &b"allocates(heap)"[..],
        &b"traps"[..],
        &b"allocates(heap), traps"[..],
    ] {
        let mut source = b"fn main() -> own unit ".to_vec();
        source.extend_from_slice(row);
        source.extend_from_slice(b" {\n  return unit;\n}\n");
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
fn the_unlabelled_entry_keeps_its_ordinary_callee_status() {
    // [FN-7] rejects a call only to a *kind-declaring* entry; the unlabelled
    // entry stays an ordinary callee [TYPE-6, OP-1].
    let source = b"fn helper() -> own unit pure {\n  main();\n  return unit;\n}\n\nfn main() -> own unit pure {\n  return unit;\n}\n";
    with_semantics(source, |outcome| {
        assert!(
            matches!(outcome, SemanticOutcome::Complete(_)),
            "a call to the unlabelled entry must remain admitted: {outcome:?}"
        );
    });
}

#[test]
fn a_missing_entry_is_the_one_bundle_root_rejection() {
    with_semantics(
        b"fn helper() -> own unit pure {\n  return unit;\n}\n",
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
        b"fn main<T>() -> own unit pure {\n  return unit;\n}\n",
        SemanticRule::Fn7,
        SemanticIssueKind::InvalidMain,
        b"<T>",
    );
    assert_rule_at(
        b"fn main ['a]() -> own unit pure {\n  return unit;\n}\n",
        SemanticRule::Fn7,
        SemanticIssueKind::InvalidMain,
        b"['a]",
    );
}

#[test]
fn the_unlabelled_entry_fixes_its_result_and_its_four_rows() {
    assert_rule_at(
        b"fn main() -> own i32 pure {\n  return 0_i32;\n}\n",
        SemanticRule::Fn7,
        SemanticIssueKind::InvalidEntryResult {
            required: "own unit",
        },
        b"own i32",
    );
    assert_rule_at(
        b"fn main(value: own i32) -> own unit pure {\n  return unit;\n}\n",
        SemanticRule::Fn7,
        SemanticIssueKind::InvalidMain,
        b"fn main(value: own i32) -> own unit pure {\n  return unit;\n}",
    );
}

#[test]
fn an_inadmissible_entry_row_outranks_the_unsupported_effect_category() {
    // The [DIAG-1] ordering this task repaired: an unsupported compiler
    // capability establishes no source violation, so an `external` or
    // `blocks` category must not mask the FN-7 rejection the checker can
    // already establish. `external` is not one of the unlabelled form's four
    // rows, so this source is rejected, not stopped.
    assert_rule_at(
        b"fn main() -> own unit external {\n  return unit;\n}\n",
        SemanticRule::Fn7,
        SemanticIssueKind::InvalidEntryEffects {
            admitted: "exactly one of `pure`, `allocates(heap)`, `traps`, `allocates(heap), traps`",
        },
        b"external",
    );
    assert_rule_at(
        b"fn main() -> own unit blocks {\n  return unit;\n}\n",
        SemanticRule::Fn7,
        SemanticIssueKind::InvalidEntryEffects {
            admitted: "exactly one of `pure`, `allocates(heap)`, `traps`, `allocates(heap), traps`",
        },
        b"blocks",
    );
    // The same category on a declaration FN-7 does not govern is [EFF-2]'s
    // judgment: a non-kind-declaring function can never exhibit it, so
    // declaring it is declared-but-unexhibited.
    assert_rule(
        b"fn probe() -> own unit external {\n  return unit;\n}\n\nfn main() -> own unit pure {\n  return unit;\n}\n",
        SemanticRule::Eff2,
        SemanticIssueKind::EffectMismatch,
    );
}

#[test]
fn the_kind_table_is_closed_at_its_program_kind_node() {
    for kind in ["service", "embedded", "daemon"] {
        let source = format!(
            "{kind} fn main() -> own ExitStatus pure {{\n  return exit_status(code: 0_u8);\n}}\n"
        );
        assert_rule_at(
            source.as_bytes(),
            SemanticRule::Fn7,
            invalid_kind(kind),
            kind.as_bytes(),
        );
    }
}

#[test]
fn only_the_entry_may_declare_a_program_kind() {
    assert_rule_at(
        b"command fn helper() -> own unit pure {\n  return unit;\n}\n\nfn main() -> own unit pure {\n  return unit;\n}\n",
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
    // The full-input entry exhibits `external, blocks` from the
    // `DirectoryRead` input's compiler-derived close attempt; every other
    // standard input's release row is empty [SYS-5].
    for source in [
        &b"command fn main() -> own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n"[..],
        &b"command fn main(command.args as args: own Args, command.cwd as cwd: own DirectoryRead, command.stdout as out: own Output, command.stderr as err: own Output) -> own ExitStatus external, blocks {\n  return exit_status(code: 0_u8);\n}\n"[..],
        // A subset in strictly increasing table-ordinal order, skipping rows.
        &b"command fn main(command.args as args: own Args, command.stderr as err: own Output) -> own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n"[..],
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
        b"command fn main(command.env as env: own Args) -> own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Fn7,
        invalid_label("command.env"),
        b"command.env as",
    );
    // Foreign kind prefix.
    assert_rule_at(
        b"command fn main(app.args as args: own Args) -> own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Fn7,
        invalid_label("app.args"),
        b"app.args as",
    );
    // Repeated row: distinct binders do not make it two inputs, because
    // ordinal identity selects the supplied value.
    assert_rule_at(
        b"command fn main(command.args as args: own Args, command.args as again: own Args) -> own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Fn7,
        invalid_label("command.args"),
        b"command.args as",
    );
    // Out of table-ordinal order.
    assert_rule_at(
        b"command fn main(command.cwd as cwd: own DirectoryRead, command.args as args: own Args) -> own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
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
        b"command fn main(command.stdout as out: own Output, command.stderr as err: own Output) -> own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        |outcome| {
            assert!(
                matches!(outcome, SemanticOutcome::Complete(_)),
                "the in-order two-output entry must check: {outcome:?}"
            );
        },
    );
    assert_rule_at(
        b"command fn main(command.stderr as err: own Output, command.stdout as out: own Output) -> own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Fn7,
        invalid_label("command.stdout"),
        b"command.stdout as",
    );
}

#[test]
fn a_selected_input_equals_its_row_at_the_complete_param_node() {
    assert_rule_at(
        b"command fn main(command.args as args: own DirectoryRead) -> own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
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
        b"command fn main(command.stderr as err: own Args) -> own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
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
        b"command fn main(value: own u8) -> own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
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
        b"fn helper(command.args as args: own Args) -> own unit pure {\n  return unit;\n}\n\ncommand fn main() -> own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Fn7,
        SemanticIssueKind::StandardInputLabelOutsideEntry {
            label: "command.args".to_owned(),
        },
        b"command.args as",
    );
    // In a unit whose entry is the unlabelled form, where no parameter of any
    // declaration may carry one.
    assert_rule_at(
        b"fn helper(app.input as value: own i32) -> own unit pure {\n  return unit;\n}\n\nfn main() -> own unit pure {\n  return unit;\n}\n",
        SemanticRule::Fn7,
        SemanticIssueKind::StandardInputLabelOutsideEntry {
            label: "app.input".to_owned(),
        },
        b"app.input as",
    );
    // In a `fn_sig`, which [FN-7] names separately.
    assert_rule_at(
        b"contract Sink {\n  fn emit(app.out as value: own i32) -> own unit pure;\n}\n\nfn main() -> own unit pure {\n  return unit;\n}\n",
        SemanticRule::Fn7,
        SemanticIssueKind::StandardInputLabelOutsideEntry {
            label: "app.out".to_owned(),
        },
        b"app.out as",
    );
}

#[test]
fn a_command_entry_fixes_its_own_exit_status_result() {
    assert_rule_at(
        b"command fn main() -> own unit pure {\n  return unit;\n}\n",
        SemanticRule::Fn7,
        SemanticIssueKind::InvalidEntryResult {
            required: "own ExitStatus",
        },
        b"own unit",
    );
    assert_rule_at(
        b"command fn main() -> own Args pure {\n  return unit;\n}\n",
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
        b"command fn main() -> own ExitStatus pure {\n  return main();\n}\n",
        SemanticRule::Fn7,
        SemanticIssueKind::CallToKindDeclaringEntry {
            entry: "main".to_owned(),
        },
        b"main()",
    );
    assert_rule_at(
        b"fn helper() -> own unit pure {\n  main();\n  return unit;\n}\n\ncommand fn main() -> own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
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
        b"command fn main() -> own ExitStatus pure {\n  return exit_status(value: 0_u8);\n}\n",
        SemanticRule::Gram11,
        declared("exit_status", &["code"]),
    );
    // Positional operands are not admitted for a system operation.
    assert_rule(
        b"command fn main() -> own ExitStatus pure {\n  return exit_status(0_u8);\n}\n",
        SemanticRule::Gram11,
        declared("exit_status", &["code"]),
    );
    // Missing parameter.
    assert_rule(
        b"command fn main() -> own ExitStatus pure {\n  return exit_status();\n}\n",
        SemanticRule::Gram11,
        declared("exit_status", &["code"]),
    );
    // Extra parameter.
    assert_rule(
        b"command fn main() -> own ExitStatus pure {\n  return exit_status(code: 0_u8, extra: 0_u8);\n}\n",
        SemanticRule::Gram11,
        declared("exit_status", &["code"]),
    );
    // The correctly spelled call is admitted here and the system semantic
    // path types it completely, so the unit checks.
    with_semantics(
        b"command fn main() -> own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
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
    // [SYS-2] names `arg_get`'s second parameter `index`, which [FORM-3]
    // excludes from IDENT, so no complete legal `arg_get` call is writable in
    // v0.18 (recorded at task 0007 closure; task 0018 carries the v0.19
    // rename that unblocks the positive case, and this assertion's spelling
    // belongs to that amendment's derived-material sweep). The general
    // GRAM-11 rule still rejects every incomplete or misspelled call, which
    // is the coverage this defect leaves reachable.
    let declared = SemanticIssueKind::InvalidNamedArguments {
        callee: "arg_get".to_owned(),
        declared_parameters: vec!["args".to_owned(), "index".to_owned()],
    };
    assert_rule(
        b"fn probe ['a](args: &'a Args) -> own unit reads('a) {\n  let value: own u64 = arg_get<'a>(args: args);\n  return unit;\n}\n\ncommand fn main() -> own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Gram11,
        declared.clone(),
    );
    assert_rule(
        b"fn probe ['a](args: &'a Args) -> own unit reads('a) {\n  let value: own u64 = arg_get<'a>(args: args, offset: 0_u64);\n  return unit;\n}\n\ncommand fn main() -> own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Gram11,
        declared,
    );
}
