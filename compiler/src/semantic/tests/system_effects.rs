//! [EFF-2] world-row projection and compiler-derived release attribution.
//!
//! The exhibited row is the union of the syntactic contribution and the
//! release contribution: the effect rows of every compiler-derived release
//! that may run on a normal control-flow edge, scoped by [STOR-3] to the
//! system resource families whose [SYS-5] contract fixes a nonempty row.

use crate::{SemanticIssueKind, SemanticLocation, SemanticOutcome, SemanticRule, TargetAction};

use super::{assert_rule, with_semantics};

const RELEASE_FIX: &str =
    "declare the world effects of every resource this function may release, or move the owner out";

fn assert_complete(source: &[u8]) {
    with_semantics(source, |outcome| {
        assert!(
            matches!(outcome, SemanticOutcome::Complete(_)),
            "expected acceptance, got {outcome:?}"
        );
    });
}

/// Asserts an EFF-2 release-attributed rejection at the function's effects
/// node, rendering the owner whose release contributed the world access.
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

const CANONICAL_ACCEPT: &[u8] = b"fn release_read_file['q, 'h, 'c, 'f](file: own ReadFile<'q, 'h, 'c, 'f>) -> result: own unit writes('q 'h) {\n  return unit;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n";

const CANONICAL_REJECT: &[u8] = b"fn release_read_file['q, 'h, 'c, 'f](file: own ReadFile<'q, 'h, 'c, 'f>) -> result: own unit pure {\n  return unit;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n";

#[test]
fn the_canonical_release_case_holds_exactly() {
    // A nongeneric function whose only parameter is `own ReadFile` and whose
    // complete body is exactly `return unit;` exhibits `writes('q 'h)`:
    // its whole row is the instantiated release contribution of that
    // parameter's compiler-derived close attempt [EFF-2, STOR-3, SYS-5].
    assert_complete(CANONICAL_ACCEPT);
    // Declaring `pure` is an undeclared-but-exhibited rejection at that
    // function's `effects` node, rendering the owning parameter.
    assert_release_mismatch(CANONICAL_REJECT, "file", b"pure");
}

const BORROWED_ACCEPT: &[u8] = b"fn touch_read_file['b, 'q, 'h, 'c, 'f](file: &'b ReadFile<'q, 'h, 'c, 'f>) -> result: own unit pure {\n  return unit;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n";

const BORROWED_REJECT: &[u8] = b"fn touch_read_file['b, 'q, 'h, 'c, 'f](file: &'b ReadFile<'q, 'h, 'c, 'f>) -> result: own unit writes('q 'h) {\n  return unit;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n";

#[test]
fn a_borrowed_resource_parameter_contributes_no_release_row() {
    // The exact contrast with the canonical case above, and the whole reason a
    // helper may touch a system value without inheriting its owner's row: the
    // release contribution collects compiler-derived *releases*, and only an
    // owner has one [EFF-2, STOR-3]. The same body under a borrowed parameter
    // is therefore exactly `pure`.
    assert_complete(BORROWED_ACCEPT);
    // And exactly `pure`: declaring the owner's row over a borrow is
    // declared-but-unexhibited. No release contributed the accesses, so this
    // is the ordinary mismatch rather than the release-attributed diagnostic.
    assert_rule(
        BORROWED_REJECT,
        SemanticRule::Eff2,
        SemanticIssueKind::EffectMismatch,
    );
}

#[test]
fn over_declaring_a_world_row_rejects_likewise() {
    // The opposite direction: `Args` releases by logical consume with the
    // empty row [SYS-5, SYS-9], and `args_count` has no world access, so a
    // declared command-order write is declared-but-unexhibited. The Output
    // parameter anchors the world kind but its source-detach release is pure.
    // No release contributed the mismatching access, so this is the
    // ordinary EFF-2 mismatch, not the release-attributed diagnostic.
    assert_rule(
        b"fn count_arguments['q, 'o](args: own Args, output: own Output<'q, 'o>) -> result: own u64 writes('q) {\n  region 'a {\n    let total = args_count<'a>(args: &'a args);\n    return total;\n  }\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Eff2,
        SemanticIssueKind::EffectMismatch,
    );
}

#[test]
fn an_immutable_borrowing_helper_is_exactly_pure() {
    // Reading through a borrow of a current-function own root contributes no
    // enclosing region effect [EFF-2], `args_count` is total with the empty
    // row [SYS-2], and `Args` releases with the empty row [SYS-5], so `pure`
    // is exact rather than merely permitted.
    assert_complete(
        b"fn count_arguments(args: own Args) -> result: own u64 pure {\n  region 'a {\n    let total = args_count<'a>(args: &'a args);\n    return total;\n  }\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
    );
}

const CONDITIONAL_UNION_ACCEPT: &[u8] = b"fn dispose_open_outcome['q, 'h, 'c, 'f](outcome: own Result<ReadFile<'q, 'h, 'c, 'f>, IoError>) -> result: own unit writes('q 'h) {\n  match outcome {\n    Ok(value: file) => {\n      return unit;\n    }\n    Err(error: problem) => {\n      return unit;\n    }\n  }\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n";

const CONDITIONAL_UNION_REJECT: &[u8] = b"fn dispose_open_outcome['q, 'h, 'c, 'f](outcome: own Result<ReadFile<'q, 'h, 'c, 'f>, IoError>) -> result: own unit pure {\n  match outcome {\n    Ok(value: file) => {\n      return unit;\n    }\n    Err(error: problem) => {\n      return unit;\n    }\n  }\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n";

#[test]
fn a_release_on_one_match_arm_contributes_its_row() {
    // The release contribution is the union over every normal edge of the
    // conservative structural graph [FN-1]: only the `Ok` arm ever holds a
    // `ReadFile`, and `IoError` has no release action [SYS-5], yet the
    // one-arm release still contributes `writes('q 'h)`.
    assert_complete(CONDITIONAL_UNION_ACCEPT);
    // Running on only some paths never weakens the contribution: omitting
    // the row is an undeclared-but-exhibited rejection naming the arm
    // binder whose release contributed it.
    assert_release_mismatch(CONDITIONAL_UNION_REJECT, "file", b"pure");
}

#[test]
fn a_pure_contract_member_cannot_bind_a_world_release_function() {
    // [FN-3] compares kinded rows and capability vectors after positional
    // alpha-renaming: a pure member cannot bind a function whose release
    // writes two world facets.
    assert_rule(
        b"contract Disposer {\n  fn dispose['cq, 'ch, 'cc, 'cf](file: own ReadFile<'cq, 'ch, 'cc, 'cf>) -> result: own unit pure;\n}\n\nconform u64: Disposer {\n  dispose = release_read_file;\n}\n\nfn release_read_file['q, 'h, 'c, 'f](file: own ReadFile<'q, 'h, 'c, 'f>) -> result: own unit writes('q 'h) {\n  return unit;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Fn3,
        SemanticIssueKind::IncompatibleConformanceFunction,
    );
    // The alpha-renamed world row and vector bind when both are exact.
    assert_complete(
        b"contract Disposer {\n  fn dispose['cq, 'ch, 'cc, 'cf](file: own ReadFile<'cq, 'ch, 'cc, 'cf>) -> result: own unit writes('cq 'ch);\n}\n\nconform u64: Disposer {\n  dispose = release_read_file;\n}\n\nfn release_read_file['q, 'h, 'c, 'f](file: own ReadFile<'q, 'h, 'c, 'f>) -> result: own unit writes('q 'h) {\n  return unit;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
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
    // boxed `ReadFile` with its instantiated world row, so the row is
    // exhibited through the indirection.
    assert_complete(
        b"fn stash['q, 'h, 'c, 'f](file: own ReadFile<'q, 'h, 'c, 'f>) -> result: own unit writes('q 'h), allocates(heap) {\n  let boxed = box_new(move file);\n  return unit;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
    );
}

#[test]
fn world_rows_keep_eff1_canonical_order_and_multiplicity() {
    // Reads then writes is the fixed category order, and each category occurs
    // at most once [EFF-1].
    assert_rule(
        b"fn probe['q, 'o](output: own Output<'q, 'o>) -> result: own unit writes('q), reads('q) {\n  return unit;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Eff1,
        SemanticIssueKind::InvalidEffectRow,
    );
    assert_rule(
        b"fn probe['q, 'o](output: own Output<'q, 'o>) -> result: own unit writes('q), writes('o) {\n  return unit;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Eff1,
        SemanticIssueKind::InvalidEffectRow,
    );
    assert_rule(
        b"fn probe['q, 'o](output: own Output<'q, 'o>) -> result: own unit traps, writes('q) {\n  return unit;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Eff1,
        SemanticIssueKind::InvalidEffectRow,
    );
}

#[test]
fn user_calls_substitute_world_rows_positionally() {
    // A user call substitutes world formals directly through the capability
    // vector and exposes the resulting row at its caller [EFF-2].
    assert_complete(
        b"fn release_read_file['rq, 'rh, 'rc, 'rf](file: own ReadFile<'rq, 'rh, 'rc, 'rf>) -> result: own unit writes('rq 'rh) {\n  return unit;\n}\n\nfn forward['q, 'h, 'c, 'f](file: own ReadFile<'q, 'h, 'c, 'f>) -> result: own unit writes('q 'h) {\n  release_read_file<'q, 'h, 'c, 'f>(file: move file);\n  return unit;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
    );
    assert_rule(
        b"fn release_read_file['rq, 'rh, 'rc, 'rf](file: own ReadFile<'rq, 'rh, 'rc, 'rf>) -> result: own unit writes('rq 'rh) {\n  return unit;\n}\n\nfn forward['q, 'h, 'c, 'f](file: own ReadFile<'q, 'h, 'c, 'f>) -> result: own unit pure {\n  release_read_file<'q, 'h, 'c, 'f>(file: move file);\n  return unit;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Eff2,
        SemanticIssueKind::EffectMismatch,
    );
}

#[test]
fn target_actions_close_transitively_over_calls_and_releases() {
    let source = b"fn close_file['q, 'h, 'c, 'f](file: own ReadFile<'q, 'h, 'c, 'f>) -> result: own unit writes('q 'h) {\n  return unit;\n}\n\nfn forward['q, 'h, 'c, 'f](file: own ReadFile<'q, 'h, 'c, 'f>) -> result: own unit writes('q 'h) {\n  close_file<'q, 'h, 'c, 'f>(file: move file);\n  return unit;\n}\n\nfn outer['q, 'h, 'c, 'f](file: own ReadFile<'q, 'h, 'c, 'f>) -> result: own unit writes('q 'h) {\n  forward<'q, 'h, 'c, 'f>(file: move file);\n  return unit;\n}\n\nfn local(value: own u64) -> result: own u64 pure {\n  return value;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n";
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(program) = outcome else {
            panic!("target-action fixture must be accepted: {outcome:?}");
        };
        for name in ["close_file", "forward", "outer"] {
            let function = program
                .data
                .functions
                .iter()
                .find(|function| function.name == name)
                .unwrap_or_else(|| panic!("missing function {name}"));
            assert_eq!(
                function.target_action,
                TargetAction::COMPLETION,
                "{name} must inherit the native-close completion action"
            );
        }
        let local = program
            .data
            .functions
            .iter()
            .find(|function| function.name == "local")
            .expect("local function");
        assert_eq!(local.target_action, TargetAction::INLINE);
    });
}
