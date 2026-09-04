//! [EFF-2] state-parameter effects and release attribution.
//!
//! The exhibited row is the union of the syntactic contribution and the
//! release contribution: the effect rows of every compiler-derived release
//! that may run on a normal control-flow edge, scoped by [STOR-3] to the
//! system resource families whose [SYS-5] contract fixes a nonempty row.

use crate::{SemanticIssueKind, SemanticLocation, SemanticOutcome, SemanticRule};

use super::super::model::{CheckedResultStateOrigin, CheckedResultStatePath};
use super::{assert_rule, assert_rule_kind, with_semantics};

const RELEASE_FIX: &str = "declare the release effects of every resource this function may release, or move the owner out";

fn root(parameter: u32) -> CheckedResultStatePath {
    CheckedResultStatePath {
        result_fields: Vec::new(),
        result_variant: None,
        parameter,
        parameter_fields: Vec::new(),
    }
}

fn assert_complete(source: &[u8]) {
    with_semantics(source, |outcome| {
        assert!(
            matches!(outcome, SemanticOutcome::Complete(_)),
            "expected acceptance, got {outcome:?}"
        );
    });
}

/// Asserts an EFF-2 release-attributed rejection at the function's effects
/// node, rendering the owner whose release contributed the category.
fn assert_release_mismatch(source: &[u8], owner: &str, located: &[u8]) {
    with_semantics(source, |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("expected an EFF-2 release mismatch, got {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Eff2);
        assert_eq!(
            issue.kind(),
            &SemanticIssueKind::ReleaseEffectMismatch {
                owner: owner.to_owned(),
                mechanical_fix: RELEASE_FIX,
            }
        );
        let SemanticLocation::SourceNode(_, coordinate) = issue.location() else {
            panic!("EFF-2 must use a source node, got {:?}", issue.location());
        };
        let start = usize::try_from(coordinate.start().value()).expect("test offset fits usize");
        let end = usize::try_from(coordinate.end().value()).expect("test offset fits usize");
        assert_eq!(
            std::str::from_utf8(&source[start..end]),
            std::str::from_utf8(located),
            "the mismatch must locate the effects node"
        );
    });
}

const CANONICAL_ACCEPT: &[u8] = b"fn release_read_file(file: own ReadFile) -> result: own unit writes(file) {\n  return unit;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n";

const CANONICAL_REJECT: &[u8] = b"fn release_read_file(file: own ReadFile) -> result: own unit pure {\n  return unit;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n";

#[test]
fn the_canonical_release_case_holds_exactly() {
    // A nongeneric function whose only parameter is `own ReadFile` and whose
    // complete body is exactly `return unit;` exhibits `writes(file)`:
    // its whole row is the release contribution of that parameter's
    // compiler-derived close attempt on the function-return edge
    // [EFF-2, STOR-3, SYS-5].
    assert_complete(CANONICAL_ACCEPT);
    // Declaring `pure` is an undeclared-but-exhibited rejection at that
    // function's `effects` node, rendering the owning parameter.
    assert_release_mismatch(CANONICAL_REJECT, "file", b"pure");
}

const BORROWED_ACCEPT: &[u8] = b"fn touch_read_file['f](file: &'f ReadFile) -> result: own unit pure {\n  return unit;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n";

const BORROWED_REJECT: &[u8] = b"fn touch_read_file['f](file: &'f ReadFile) -> result: own unit writes(file) {\n  return unit;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n";

#[test]
fn a_borrowed_resource_parameter_contributes_no_release_row() {
    // The exact contrast with the canonical case above, and the whole reason a
    // helper may touch a system value without inheriting its owner's row: the
    // release contribution collects compiler-derived *releases*, and only an
    // owner has one [EFF-2, STOR-3]. The same body under a borrowed parameter
    // is therefore exactly `pure`.
    assert_complete(BORROWED_ACCEPT);
    // A shared loan cannot authorize a state transition. The signature is
    // rejected at EFF-1 before release attribution is considered.
    assert_rule_kind(BORROWED_REJECT, SemanticRule::Eff1, |kind| {
        matches!(kind, SemanticIssueKind::InvalidEffectRow { .. })
    });
}

#[test]
fn over_declaring_the_release_row_rejects_likewise() {
    // Preserve the old test's declared-but-unexhibited direction under the
    // state-row model: Args release is empty and the body performs no
    // state action, so `reads(args)` is an exact over-declaration.
    assert_rule_kind(
        b"fn ignore_arguments(args: own Args) -> result: own unit reads(args) {\n  return unit;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Eff2,
        |kind| matches!(kind, SemanticIssueKind::EffectMismatch { .. }),
    );
}

#[test]
fn file_reservation_and_open_project_only_their_explicit_inputs() {
    assert_complete(
        br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files) {
  region 'state {
    match reserve_file::<'state>(factory: &uniq 'state files) {
      Ok(value: permit) => {
        let opened = open_directory_source::<'state>(permit: move permit, directory: &'state cwd);
      }
      Err(error: spent) => {
        return exit_status(code: 8_u8);
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#,
    );
}

#[test]
fn file_reservation_projects_the_factory_without_an_open() {
    assert_complete(
        br#"command fn main(command.files as files: own FileFactory) -> status: own ExitStatus reads(files), writes(files) {
  region 'state {
    match reserve_file::<'state>(factory: &uniq 'state files) {
      Ok(value: permit) => {
      }
      Err(error: spent) => {
        return exit_status(code: 8_u8);
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#,
    );
}

#[test]
fn unused_file_authority_releases_by_logical_consume() {
    assert_complete(
        br#"fn discard_factory(factory: own FileFactory) -> result: own unit pure {
  return unit;
}

fn discard_permit(permit: own FilePermit) -> result: own unit pure {
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
    );
}

#[test]
fn an_immutable_borrowing_helper_names_only_the_snapshot_state() {
    // The local borrow region still does not escape into the row. The new
    // authority component is `reads(args)`, independently of that lifetime.
    assert_complete(
        b"fn count_arguments(args: own Args) -> result: own u64 reads(args) {\n  region 'a {\n    let total = args_count::<'a>(args: &'a args);\n    return total;\n  }\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
    );
}

const CONDITIONAL_UNION_ACCEPT: &[u8] = b"fn dispose_open_outcome(outcome: own Result<ReadFile, IoError>) -> result: own unit writes(outcome) {\n  match outcome {\n    Ok(value: file) => {\n      return unit;\n    }\n    Err(error: problem) => {\n      return unit;\n    }\n  }\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n";

const CONDITIONAL_UNION_REJECT: &[u8] = b"fn dispose_open_outcome(outcome: own Result<ReadFile, IoError>) -> result: own unit pure {\n  match outcome {\n    Ok(value: file) => {\n      return unit;\n    }\n    Err(error: problem) => {\n      return unit;\n    }\n  }\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n";

#[test]
fn a_release_on_one_match_arm_contributes_its_row() {
    // The release contribution is the union over every normal edge of the
    // conservative structural graph [FN-1]: only the `Ok` arm ever holds a
    // `ReadFile`, and `IoError` has no release action [SYS-5], yet the
    // one-arm release still contributes its exact state write.
    assert_complete(CONDITIONAL_UNION_ACCEPT);
    // Running on only some paths never weakens the contribution: omitting
    // the row is an undeclared-but-exhibited rejection naming the arm
    // binder whose release contributed it.
    assert_release_mismatch(CONDITIONAL_UNION_REJECT, "file", b"pure");
}

#[test]
fn a_pure_contract_member_cannot_bind_a_release_effectful_function() {
    // [FN-3] normalizes state identities and compares `external` and `blocks`
    // by presence: a `pure` member cannot bind a function that exhibits a
    // category only through release.
    assert_rule(
        b"contract Disposer {\n  fn dispose(file: own ReadFile) -> result: own unit pure;\n}\n\nconform u64: Disposer {\n  dispose = release_read_file;\n}\n\nfn release_read_file(file: own ReadFile) -> result: own unit writes(file) {\n  return unit;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Fn3,
        SemanticIssueKind::IncompatibleConformanceFunction,
    );
    // The same member row binds the same function when both declare the two
    // categories, so the presence comparison admits as well as rejects.
    assert_complete(
        b"contract Disposer {\n  fn dispose(item: own ReadFile) -> result: own unit writes(item);\n}\n\nconform u64: Disposer {\n  dispose = release_read_file;\n}\n\nfn release_read_file(file: own ReadFile) -> result: own unit writes(file) {\n  return unit;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
    );
}

#[test]
fn memory_reclamation_contributes_no_release_row() {
    // The [STOR-3] scope limit, as a facts-off-style regression: a
    // `buffer<T>` drop, a `box<T>` drop, and every frame-resident drop carry
    // the empty release row, so no pre-existing accepted program's legal row
    // changes — these v0.17-legal rows stay exact.
    assert_complete(
        b"fn consume(data: own buffer<u8>) -> result: own unit pure {\n  return unit;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
    );
    assert_complete(
        b"command fn main() -> status: own ExitStatus allocates(heap) {\n  let boxed = box_new(0_u64);\n  let stored = buffer_new(4_u64, 0_u8);\n  return exit_status(code: 0_u8);\n}\n",
    );
}

#[test]
fn release_attribution_is_transitive_over_owned_content() {
    // Release of a value is release of its components [SYS-5]: a
    // `box<ReadFile>` drop frees the box with the empty row and releases the
    // boxed `ReadFile` with its fixed state-release row, so the row is
    // exhibited through the indirection.
    assert_complete(
        b"fn stash(file: own ReadFile) -> result: own unit writes(file), allocates(heap) {\n  let boxed = box_new(move file);\n  return unit;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
    );
}

#[test]
fn live_effect_categories_keep_eff1_canonical_order_and_multiplicity() {
    // The replacement keeps the same canonical-order and multiplicity
    // coverage over the live categories: reads, writes, and allocates.
    assert_rule_kind(
        b"fn probe(file: own ReadFile) -> result: own unit allocates(heap), writes(file) {\n  return unit;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Eff1,
        |kind| matches!(kind, SemanticIssueKind::InvalidEffectRow { .. }),
    );
    assert_rule_kind(
        b"fn probe(file: own ReadFile) -> result: own unit writes(file), writes(file) {\n  return unit;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Eff1,
        |kind| matches!(kind, SemanticIssueKind::InvalidEffectRow { .. }),
    );
    assert_rule_kind(
        b"fn probe(file: own ReadFile) -> result: own unit writes(file), reads(file) {\n  return unit;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Eff1,
        |kind| matches!(kind, SemanticIssueKind::InvalidEffectRow { .. }),
    );
}

#[test]
fn user_calls_substitute_state_formals_to_actual_origins() {
    // A moved state keeps its direct formal origin, and the callee's
    // state subject projects back to that caller formal.
    assert_complete(
        b"fn release_read_file(file: own ReadFile) -> result: own unit writes(file) {\n  return unit;\n}\n\nfn forward(file: own ReadFile) -> result: own unit writes(file) {\n  let moved = move file;\n  release_read_file(file: move moved);\n  return unit;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
    );
    assert_rule_kind(
        b"fn release_read_file(file: own ReadFile) -> result: own unit writes(file) {\n  return unit;\n}\n\nfn forward(file: own ReadFile) -> result: own unit pure {\n  release_read_file(file: move file);\n  return unit;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Eff2,
        |kind| matches!(kind, SemanticIssueKind::EffectMismatch { .. }),
    );
}

const PASS_OUTPUT_PREFIX: &str = r#"fn pass_output(output: own Output) -> result: own Output pure {
  return move output;
}

"#;

fn pass_output_program(effects: &str) -> Vec<u8> {
    format!(
        "{PASS_OUTPUT_PREFIX}command fn main(command.stdout as out: own Output) -> status: own ExitStatus {effects} {{\n  let same = pass_output(output: move out);\n  let bytes = buffer_new(1_u64, 65_u8);\n  region 'o {{\n    region 's {{\n      let written = write_once::<'o, 's>(output: &uniq 'o same, source: &'s bytes, start: 0_u64, end: 1_u64);\n    }}\n  }}\n  return exit_status(code: 0_u8);\n}}\n"
    )
    .into_bytes()
}

#[test]
fn a_user_result_cannot_wash_an_output_formal_origin() {
    let accepted = pass_output_program("reads(out), writes(out), allocates(heap)");
    assert_complete(&accepted);
    let washed = pass_output_program("allocates(heap)");
    assert_rule_kind(&washed, SemanticRule::Eff2, |kind| {
        matches!(kind, SemanticIssueKind::EffectMismatch { .. })
    });
    with_semantics(&accepted, |outcome| {
        let SemanticOutcome::Complete(program) = outcome else {
            panic!("the pass-through program must check: {outcome:?}");
        };
        assert_eq!(
            program.data.functions[0].result_state_origin,
            CheckedResultStateOrigin::Finite {
                formals: vec![root(0)],
            }
        );
    });
}

#[test]
fn a_passed_through_read_file_close_keeps_its_release_write() {
    let accepted = br#"fn pass_file(file: own ReadFile) -> result: own ReadFile pure {
  return move file;
}

fn close_after_pass(file: own ReadFile) -> result: own unit writes(file) {
  let same = pass_file(file: move file);
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_complete(accepted);
    let rejected = br#"fn pass_file(file: own ReadFile) -> result: own ReadFile pure {
  return move file;
}

fn close_after_pass(file: own ReadFile) -> result: own unit pure {
  let same = pass_file(file: move file);
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_release_mismatch(rejected, "same", b"pure");
}

fn choose_output_program(effects: &str, delivered: bool) -> Vec<u8> {
    let chooser = if delivered {
        r#"fn choose_output(left: own Output, right: own Output, take_left: own Bool) -> result: own Output pure {
  let selected = if take_left {
    give move left;
  } else {
    give move right;
  }
  return move selected;
}

fn forward_choice(left: own Output, right: own Output, take_left: own Bool) -> result: own Output pure {
  return choose_output(left: move left, right: move right, take_left: take_left);
}

"#
    } else {
        r#"fn forward_choice(left: own Output, right: own Output, take_left: own Bool) -> result: own Output pure {
  if take_left {
    return move left;
  } else {
    return move right;
  }
}

"#
    };
    format!(
        "{chooser}command fn main(command.stdout as out: own Output, command.stderr as err: own Output) -> status: own ExitStatus {effects} {{\n  let flag = True();\n  let selected = forward_choice(left: move out, right: move err, take_left: flag);\n  let bytes = buffer_new(1_u64, 65_u8);\n  region 'o {{\n    region 's {{\n      let written = write_once::<'o, 's>(output: &uniq 'o selected, source: &'s bytes, start: 0_u64, end: 1_u64);\n    }}\n  }}\n  return exit_status(code: 0_u8);\n}}\n"
    )
    .into_bytes()
}

#[test]
fn a_control_flow_result_projects_to_every_possible_formal() {
    let accepted =
        choose_output_program("reads(out, err), writes(out, err), allocates(heap)", false);
    assert_complete(&accepted);
    let narrowed = choose_output_program("reads(out), writes(out), allocates(heap)", false);
    assert_rule_kind(&narrowed, SemanticRule::Eff2, |kind| {
        matches!(kind, SemanticIssueKind::EffectMismatch { .. })
    });
    with_semantics(&accepted, |outcome| {
        let SemanticOutcome::Complete(program) = outcome else {
            panic!("the finite-origin choice must check: {outcome:?}");
        };
        assert_eq!(
            program.data.functions[0].result_state_origin,
            CheckedResultStateOrigin::Finite {
                formals: vec![root(0), root(1)],
            }
        );
    });
}

#[test]
fn value_if_delivery_and_a_multihop_wrapper_preserve_the_same_formal() {
    let source = br#"fn delivered(output: own Output, first: own Bool) -> result: own Output pure {
  let selected = if first {
    give move output;
  } else {
    give move output;
  }
  return move selected;
}

fn delivered_wrapper(output: own Output, first: own Bool) -> result: own Output pure {
  return delivered(output: move output, first: first);
}

command fn main(command.stdout as out: own Output) -> status: own ExitStatus reads(out), writes(out), allocates(heap) {
  let flag = True();
  let selected = delivered_wrapper(output: move out, first: flag);
  let bytes = buffer_new(1_u64, 65_u8);
  region 'o {
    region 's {
      let written = write_once::<'o, 's>(output: &uniq 'o selected, source: &'s bytes, start: 0_u64, end: 1_u64);
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    assert_complete(source);
}

#[test]
fn a_recursive_pass_through_reaches_the_formal_fixed_point() {
    let source = br#"fn recursive_pass(output: own Output, stop: own Bool) -> result: own Output pure {
  if stop {
    return move output;
  } else {
    return recursive_pass(output: move output, stop: stop);
  }
}

command fn main(command.stdout as out: own Output) -> status: own ExitStatus reads(out), writes(out), allocates(heap) {
  let flag = True();
  let selected = recursive_pass(output: move out, stop: flag);
  let bytes = buffer_new(1_u64, 65_u8);
  region 'o {
    region 's {
      let written = write_once::<'o, 's>(output: &uniq 'o selected, source: &'s bytes, start: 0_u64, end: 1_u64);
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    assert_complete(source);
}

#[test]
fn a_mutually_recursive_pass_through_reaches_the_same_fixed_point() {
    let source = br#"fn mutual_a(output: own Output, stop: own Bool) -> result: own Output pure {
  if stop {
    return move output;
  } else {
    return mutual_b(output: move output, stop: stop);
  }
}

fn mutual_b(output: own Output, stop: own Bool) -> result: own Output pure {
  return mutual_a(output: move output, stop: stop);
}

command fn main(command.stdout as out: own Output) -> status: own ExitStatus reads(out), writes(out), allocates(heap) {
  let flag = True();
  let selected = mutual_b(output: move out, stop: flag);
  let bytes = buffer_new(1_u64, 65_u8);
  region 'o {
    region 's {
      let written = write_once::<'o, 's>(output: &uniq 'o selected, source: &'s bytes, start: 0_u64, end: 1_u64);
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    assert_complete(source);
}

#[test]
fn a_fresh_and_formal_result_join_remains_a_finite_origin_set() {
    let source = br#"fn choose_file['r, 'p](existing: own FileOpenOutcome, permit: own FilePermit, root: &'r DirectoryRead, path: &'p RelativePath, fresh: own Bool) -> result: own FileOpenOutcome reads(permit, root, path), writes(existing, permit) {
  if fresh {
    return open_read::<'r, 'p>(permit: move permit, root: root, path: path);
  } else {
    return move existing;
  }
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(program) = outcome else {
            panic!("fresh/formal origin union must check: {outcome:?}");
        };
        assert_eq!(
            program.data.functions[0].result_state_origin,
            CheckedResultStateOrigin::Finite {
                formals: vec![root(0)],
            }
        );
    });
}

#[test]
fn an_unclosed_recursive_origin_no_longer_creates_a_language_stop() {
    let source = br#"fn unclosed(output: own Output) -> result: own Output pure {
  return unclosed(output: move output);
}

command fn main(command.stdout as out: own Output) -> status: own ExitStatus pure {
  let result = unclosed(output: move out);
  return exit_status(code: 0_u8);
}
"#;
    assert_complete(source);
}

#[test]
fn a_multi_state_aggregate_releases_each_structural_leaf() {
    let source = br#"struct Pair {
  first: ReadFile;
  second: ReadFile;
}

fn pass_pair(pair: own Pair) -> result: own Pair pure {
  return move pair;
}

fn dispose(pair: own Pair) -> result: own unit writes(pair.first, pair.second) {
  let same = pass_pair(pair: move pair);
  return unit;
}

fn pack(first: own ReadFile, second: own ReadFile) -> result: own Pair pure {
  return Pair(first: move first, second: move second);
}

fn dispose_inputs(first: own ReadFile, second: own ReadFile) -> result: own unit writes(first, second) {
  let pair = pack(first: move first, second: move second);
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(program) = outcome else {
            panic!("two structural state leaves must survive an ordinary result move: {outcome:?}");
        };
        assert_eq!(
            program.data.functions[0].result_state_origin,
            CheckedResultStateOrigin::Finite {
                formals: vec![
                    CheckedResultStatePath {
                        result_fields: vec![0],
                        result_variant: None,
                        parameter: 0,
                        parameter_fields: vec![0],
                    },
                    CheckedResultStatePath {
                        result_fields: vec![1],
                        result_variant: None,
                        parameter: 0,
                        parameter_fields: vec![1],
                    },
                ],
            }
        );
    });
}

#[test]
fn ordinary_affine_types_and_embedded_copy_fields_preserve_result_identity() {
    let source = br#"struct Record {
  label: HostString;
  count: u64;
}

fn pass_host(value: own HostString) -> result: own HostString pure {
  return move value;
}

fn pass_path(value: own RelativePath) -> result: own RelativePath pure {
  return move value;
}

fn pass_factory(value: own FileFactory) -> result: own FileFactory pure {
  return move value;
}

fn pass_buffer(value: own buffer<u8>) -> result: own buffer<u8> pure {
  return move value;
}

fn pass_record(value: own Record) -> result: own Record pure {
  return move value;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(program) = outcome else {
            panic!("ordinary affine pass-throughs must check: {outcome:?}");
        };
        for name in ["pass_host", "pass_path", "pass_factory", "pass_buffer"] {
            let function = program
                .data
                .functions
                .iter()
                .find(|function| function.name == name)
                .unwrap_or_else(|| panic!("missing function {name}"));
            assert_eq!(
                function.result_state_origin,
                CheckedResultStateOrigin::Finite {
                    formals: vec![root(0)],
                },
                "{name} must use the ordinary owner-transfer route"
            );
        }
        let record = program
            .data
            .functions
            .iter()
            .find(|function| function.name == "pass_record")
            .expect("pass_record function");
        assert_eq!(
            record.result_state_origin,
            CheckedResultStateOrigin::Finite {
                formals: vec![
                    CheckedResultStatePath {
                        result_fields: vec![0],
                        result_variant: None,
                        parameter: 0,
                        parameter_fields: vec![0],
                    },
                    CheckedResultStatePath {
                        result_fields: vec![1],
                        result_variant: None,
                        parameter: 0,
                        parameter_fields: vec![1],
                    },
                ],
            },
            "the copy field remains a structural leaf inside an affine owner"
        );
    });
}

#[test]
fn replace_routes_the_old_field_and_residual_releases_independently() {
    let source = br#"struct Pair {
  first: ReadFile;
  second: ReadFile;
}

fn replace_first(pair: own Pair, replacement: own ReadFile) -> result: own ReadFile reads(pair.first), writes(pair.first, pair.second, replacement) {
  let previous = replace pair.first = move replacement;
  return move previous;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(program) = outcome else {
            panic!("field replacement must keep the old and residual owners distinct: {outcome:?}");
        };
        assert_eq!(
            program.data.functions[0].result_state_origin,
            CheckedResultStateOrigin::Finite {
                formals: vec![CheckedResultStatePath {
                    result_fields: Vec::new(),
                    result_variant: None,
                    parameter: 0,
                    parameter_fields: vec![0],
                }],
            }
        );
    });
}

#[test]
fn direct_aggregate_construction_releases_every_input_leaf() {
    let accepted = br#"struct Pair {
  first: ReadFile;
  second: ReadFile;
}

fn dispose(first: own ReadFile, second: own ReadFile) -> result: own unit writes(first, second) {
  let pair = Pair(first: move first, second: move second);
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_complete(accepted);
    let missing_second = br#"struct Pair {
  first: ReadFile;
  second: ReadFile;
}

fn dispose(first: own ReadFile, second: own ReadFile) -> result: own unit writes(first) {
  let pair = Pair(first: move first, second: move second);
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_release_mismatch(missing_second, "pair", b"writes(first)");
}

#[test]
fn match_payload_binders_receive_only_the_selected_constructor_payload() {
    let source = br#"enum Choice {
  Left(file: ReadFile);
  Right(file: ReadFile);
}

fn select_left(left: own ReadFile, unrelated: own ReadFile) -> result: own ReadFile writes(unrelated) {
  let choice = Left(file: move left);
  match move choice {
    Left(file: selected) => {
      return move selected;
    }
    Right(file: impossible) => {
      return move impossible;
    }
  }
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(program) = outcome else {
            panic!(
                "an unselected payload must not inherit the selected payload's origin: {outcome:?}"
            );
        };
        assert_eq!(
            program.data.functions[0].result_state_origin,
            CheckedResultStateOrigin::Finite {
                formals: vec![root(0)],
            }
        );
    });
}

#[test]
fn a_source_error_still_precedes_state_origin_analysis() {
    let source = br#"struct Pair {
  first: ReadFile;
  second: ReadFile;
}

fn broken(pair: own Pair) -> result: own unit pure {
  return 1_u64;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_rule(source, SemanticRule::Fn1, SemanticIssueKind::ReturnMismatch);
}

#[test]
fn an_unrelated_loop_does_not_destroy_a_formal_origin() {
    let source = br#"fn through_loop(output: own Output) -> result: own Output pure {
  loop @once {
    break @once;
  }
  return move output;
}

command fn main(command.stdout as out: own Output) -> status: own ExitStatus reads(out), writes(out), allocates(heap) {
  let selected = through_loop(output: move out);
  let bytes = buffer_new(1_u64, 65_u8);
  region 'o {
    region 's {
      let written = write_once::<'o, 's>(output: &uniq 'o selected, source: &'s bytes, start: 0_u64, end: 1_u64);
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    assert_complete(source);
}

#[test]
fn a_loop_break_join_retains_the_two_origins_an_update_can_select() {
    let source = br#"fn loop_choice['p](selected: own Result<ReadFile, IoError>, factory: own FileFactory, root: own DirectoryRead, path: &'p RelativePath, refresh: own Bool) -> result: own Result<ReadFile, IoError> reads(selected, factory, root, path), writes(selected, factory, root) {
  loop @once {
    if refresh {
      region 'reservation {
        match reserve_file::<'reservation>(factory: &uniq 'reservation factory) {
          Ok(value: permit) => {
            region 'lookup {
              match open_read::<'lookup, 'p>(permit: move permit, root: &'lookup root, path: path) {
                FileOpened(value: got) => {
                  let discarded = replace selected = Ok<ReadFile, IoError>(value: move got);
                }
                FileOpenFailed(error: problem, permit: refused) => {
                  let discarded = replace selected = Err<ReadFile, IoError>(error: move problem);
                }
              }
            }
          }
          Err(error: spent) => {
            return Err<ReadFile, IoError>(error: move spent);
          }
        }
      }
    }
    break @once;
  }
  return move selected;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(program) = outcome else {
            panic!("loop origin update must check: {outcome:?}");
        };
        assert_eq!(
            program.data.functions[0].result_state_origin,
            CheckedResultStateOrigin::Finite {
                formals: vec![root(0)],
            }
        );
    });
}

#[test]
fn an_optional_state_result_can_prove_that_its_only_route_is_absent() {
    let source = br#"fn no_file() -> result: own Result<ReadFile, IoError> pure {
  let problem = Other(code: 0_u32, origin: 0_u8);
  return Err<ReadFile, IoError>(error: move problem);
}

command fn main() -> status: own ExitStatus pure {
  let result = no_file();
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(program) = outcome else {
            panic!("a proved Err-only optional state must check: {outcome:?}");
        };
        assert_eq!(
            program.data.functions[0].result_state_origin,
            CheckedResultStateOrigin::Finite {
                formals: Vec::new(),
            }
        );
    });
}

#[test]
fn an_optional_result_projects_its_present_formal_and_keeps_its_absent_route() {
    let source = br#"fn maybe_output(output: own Output, present: own Bool) -> result: own Result<Output, IoError> pure {
  if present {
    return Ok<Output, IoError>(value: move output);
  } else {
    let problem = Other(code: 0_u32, origin: 0_u8);
    return Err<Output, IoError>(error: move problem);
  }
}

command fn main(command.stdout as out: own Output) -> status: own ExitStatus reads(out), writes(out), allocates(heap) {
  let flag = True();
  match maybe_output(output: move out, present: flag) {
    Ok(value: selected) => {
      let bytes = buffer_new(1_u64, 65_u8);
      region 'o {
        region 's {
          let written = write_once::<'o, 's>(output: &uniq 'o selected, source: &'s bytes, start: 0_u64, end: 1_u64);
        }
      }
    }
    Err(error: problem) => {
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(program) = outcome else {
            panic!("optional formal origin must check: {outcome:?}");
        };
        assert_eq!(
            program.data.functions[0].result_state_origin,
            CheckedResultStateOrigin::Finite {
                formals: vec![CheckedResultStatePath {
                    result_fields: vec![0],
                    result_variant: Some(0),
                    parameter: 0,
                    parameter_fields: Vec::new(),
                }],
            }
        );
    });
}

#[test]
fn a_copy_only_parameter_is_a_valid_path_but_must_be_exhibited() {
    assert_rule_kind(
        b"fn probe(value: own u64) -> result: own unit reads(value) {\n  return unit;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Eff2,
        |kind| matches!(kind, SemanticIssueKind::EffectMismatch { .. }),
    );
}

#[test]
fn external_and_blocks_are_ordinary_function_and_parameter_names() {
    assert_complete(
        b"fn external(blocks: own Args) -> result: own u64 reads(blocks) {\n  region 'a {\n    let total = args_count::<'a>(args: &'a blocks);\n    return total;\n  }\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
    );
}

#[test]
fn borrowing_one_owned_struct_field_projects_only_that_field_effect() {
    assert_complete(
        br#"struct Pair {
  first: buffer<u8>;
  second: buffer<u8>;
}

fn length['v](value: &'v buffer<u8>) -> result: own u64 reads(value) {
  return len(deref(value));
}

fn read_second(pair: own Pair) -> result: own unit reads(pair.second) {
  region 'second {
    let count = length::<'second>(value: &'second pair.second);
  }
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
    );
}
