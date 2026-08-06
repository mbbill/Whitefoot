//! [EFF-2] release attribution and the two payload-free categories.
//!
//! The exhibited row is the union of the syntactic contribution and the
//! release contribution: the effect rows of every compiler-derived release
//! that may run on a normal control-flow edge, scoped by [STOR-3] to the
//! system resource families whose [SYS-5] contract fixes a nonempty row.

use crate::{SemanticIssueKind, SemanticLocation, SemanticOutcome, SemanticRule};

use super::{assert_rule, with_semantics};

const RELEASE_FIX: &str = "declare the release effects of every resource this function may release, or move the owner out";

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

const CANONICAL_ACCEPT: &[u8] = b"fn release_read_file(file: own ReadFile) -> own unit external, blocks {\n  return unit;\n}\n\ncommand fn main() -> own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n";

const CANONICAL_REJECT: &[u8] = b"fn release_read_file(file: own ReadFile) -> own unit pure {\n  return unit;\n}\n\ncommand fn main() -> own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n";

#[test]
fn the_canonical_release_case_holds_exactly() {
    // A nongeneric function whose only parameter is `own ReadFile` and whose
    // complete body is exactly `return unit;` exhibits `external, blocks`:
    // its whole row is the release contribution of that parameter's
    // compiler-derived close attempt on the function-return edge
    // [EFF-2, STOR-3, SYS-5].
    assert_complete(CANONICAL_ACCEPT);
    // Declaring `pure` is an undeclared-but-exhibited rejection at that
    // function's `effects` node, rendering the owning parameter.
    assert_release_mismatch(CANONICAL_REJECT, "file", b"pure");
}

const BORROWED_ACCEPT: &[u8] = b"fn touch_read_file ['f](file: &'f ReadFile) -> own unit pure {\n  return unit;\n}\n\ncommand fn main() -> own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n";

const BORROWED_REJECT: &[u8] = b"fn touch_read_file ['f](file: &'f ReadFile) -> own unit external, blocks {\n  return unit;\n}\n\ncommand fn main() -> own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n";

#[test]
fn a_borrowed_resource_parameter_contributes_no_release_row() {
    // The exact contrast with the canonical case above, and the whole reason a
    // helper may touch a system value without inheriting its owner's row: the
    // release contribution collects compiler-derived *releases*, and only an
    // owner has one [EFF-2, STOR-3]. The same body under a borrowed parameter
    // is therefore exactly `pure`.
    assert_complete(BORROWED_ACCEPT);
    // And exactly `pure`: declaring the owner's row over a borrow is
    // declared-but-unexhibited. No release contributed the categories, so this
    // is the ordinary mismatch rather than the release-attributed diagnostic.
    assert_rule(
        BORROWED_REJECT,
        SemanticRule::Eff2,
        SemanticIssueKind::EffectMismatch,
    );
}

#[test]
fn over_declaring_the_release_row_rejects_likewise() {
    // The opposite direction: `Args` releases by logical consume with the
    // empty row [SYS-5, SYS-9], and `args_count`'s own row carries neither
    // category, so declaring `external, blocks` is declared-but-unexhibited.
    // No release contributed the mismatching categories, so this is the
    // ordinary EFF-2 mismatch, not the release-attributed diagnostic.
    assert_rule(
        b"fn count_arguments(args: own Args) -> own u64 external, blocks {\n  region 'a {\n    let total: own u64 = args_count<'a>(args: &'a args);\n    return total;\n  }\n}\n\ncommand fn main() -> own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
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
        b"fn count_arguments(args: own Args) -> own u64 pure {\n  region 'a {\n    let total: own u64 = args_count<'a>(args: &'a args);\n    return total;\n  }\n}\n\ncommand fn main() -> own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
    );
}

const CONDITIONAL_UNION_ACCEPT: &[u8] = b"fn dispose_open_outcome(outcome: own Result<ReadFile, IoError>) -> own unit external, blocks {\n  match outcome {\n    Ok(value: file) => {\n      return unit;\n    }\n    Err(error: problem) => {\n      return unit;\n    }\n  }\n}\n\ncommand fn main() -> own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n";

const CONDITIONAL_UNION_REJECT: &[u8] = b"fn dispose_open_outcome(outcome: own Result<ReadFile, IoError>) -> own unit pure {\n  match outcome {\n    Ok(value: file) => {\n      return unit;\n    }\n    Err(error: problem) => {\n      return unit;\n    }\n  }\n}\n\ncommand fn main() -> own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n";

#[test]
fn a_release_on_one_match_arm_contributes_its_row() {
    // The release contribution is the union over every normal edge of the
    // conservative structural graph [FN-1]: only the `Ok` arm ever holds a
    // `ReadFile`, and `IoError` has no release action [SYS-5], yet the
    // one-arm release still contributes `external, blocks`.
    assert_complete(CONDITIONAL_UNION_ACCEPT);
    // Running on only some paths never weakens the contribution: omitting
    // the row is an undeclared-but-exhibited rejection naming the arm
    // binder whose release contributed it.
    assert_release_mismatch(CONDITIONAL_UNION_REJECT, "file", b"pure");
}

#[test]
fn a_pure_contract_member_cannot_bind_a_release_effectful_function() {
    // [FN-3] normalizes a row to six capabilities and compares `external`
    // and `blocks` by presence exactly as `traps`: a `pure` member cannot
    // bind a function that exhibits a category only through release.
    assert_rule(
        b"contract Disposer {\n  fn dispose(file: own ReadFile) -> own unit pure;\n}\n\nconform u64: Disposer {\n  dispose = release_read_file;\n}\n\nfn release_read_file(file: own ReadFile) -> own unit external, blocks {\n  return unit;\n}\n\ncommand fn main() -> own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Fn3,
        SemanticIssueKind::IncompatibleConformanceFunction,
    );
    // The same member row binds the same function when both declare the two
    // categories, so the presence comparison admits as well as rejects.
    assert_complete(
        b"contract Disposer {\n  fn dispose(file: own ReadFile) -> own unit external, blocks;\n}\n\nconform u64: Disposer {\n  dispose = release_read_file;\n}\n\nfn release_read_file(file: own ReadFile) -> own unit external, blocks {\n  return unit;\n}\n\ncommand fn main() -> own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
    );
}

#[test]
fn memory_reclamation_contributes_no_release_row() {
    // The [STOR-3] scope limit, as a facts-off-style regression: a
    // `buffer<T>` drop, a `box<T>` drop, and every frame-resident drop carry
    // the empty release row, so no pre-existing accepted program's legal row
    // changes — these v0.17-legal rows stay exact.
    assert_complete(
        b"fn consume(data: own buffer<u8>) -> own unit pure {\n  return unit;\n}\n\nfn main() -> own unit pure {\n  return unit;\n}\n",
    );
    assert_complete(
        b"fn main() -> own unit allocates(heap), traps {\n  let boxed: own box<u64> = box_new<u64>(0_u64);\n  let stored: own buffer<u8> = buffer_new<u8>(4_u64, 0_u8);\n  return unit;\n}\n",
    );
}

#[test]
fn release_attribution_is_transitive_over_owned_content() {
    // Release of a value is release of its components [SYS-5]: a
    // `box<ReadFile>` drop frees the box with the empty row and releases the
    // boxed `ReadFile` with its fixed `external, blocks` row, so the row is
    // exhibited through the indirection.
    assert_complete(
        b"fn stash(file: own ReadFile) -> own unit allocates(heap), external, blocks {\n  let boxed: own box<ReadFile> = box_new<ReadFile>(move file);\n  return unit;\n}\n\ncommand fn main() -> own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
    );
}

#[test]
fn the_two_categories_keep_eff1_canonical_order_and_multiplicity() {
    // `external` and `blocks` take fixed positions between `allocates` and
    // `traps` [EFF-1]; out-of-order and repeated spellings are EFF-1
    // rejections before any EFF-2 judgment.
    assert_rule(
        b"fn probe() -> own unit blocks, external {\n  return unit;\n}\n\nfn main() -> own unit pure {\n  return unit;\n}\n",
        SemanticRule::Eff1,
        SemanticIssueKind::InvalidEffectRow,
    );
    assert_rule(
        b"fn probe() -> own unit external, external {\n  return unit;\n}\n\nfn main() -> own unit pure {\n  return unit;\n}\n",
        SemanticRule::Eff1,
        SemanticIssueKind::InvalidEffectRow,
    );
    assert_rule(
        b"fn probe() -> own unit traps, blocks {\n  return unit;\n}\n\nfn main() -> own unit pure {\n  return unit;\n}\n",
        SemanticRule::Eff1,
        SemanticIssueKind::InvalidEffectRow,
    );
}

#[test]
fn user_calls_transfer_the_two_categories_by_presence() {
    // A call to a function whose row includes `external` or `blocks`
    // syntactically exhibits that category at the caller [EFF-2]; the
    // categories are payload-free, so no region projection applies.
    assert_complete(
        b"fn release_read_file(file: own ReadFile) -> own unit external, blocks {\n  return unit;\n}\n\nfn forward(file: own ReadFile) -> own unit external, blocks {\n  release_read_file(file: move file);\n  return unit;\n}\n\ncommand fn main() -> own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
    );
    assert_rule(
        b"fn release_read_file(file: own ReadFile) -> own unit external, blocks {\n  return unit;\n}\n\nfn forward(file: own ReadFile) -> own unit pure {\n  release_read_file(file: move file);\n  return unit;\n}\n\ncommand fn main() -> own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Eff2,
        SemanticIssueKind::EffectMismatch,
    );
}
