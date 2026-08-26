//! Compiler-owned target execution metadata and its call-graph fixed point.

use crate::{SemanticOutcome, TargetAction};

use super::with_semantics;

#[test]
fn system_catalog_actions_have_one_shot_ownership_completion() {
    for operation in crate::SYSTEM_OPERATIONS {
        let expected = matches!(
            operation.spelling,
            "open_read"
                | "read_at"
                | "write_once"
                | "open_directory"
                | "open_directory_source"
                | "directory_next"
                | "open_file"
        );
        assert_eq!(
            operation.target_action.may_suspend(),
            expected,
            "unexpected target action for {}",
            operation.spelling
        );
        assert_eq!(
            operation.target_action.completion,
            if expected {
                crate::TargetCompletion::OwnershipComplete
            } else {
                crate::TargetCompletion::CallReturn
            }
        );
        let milestones = operation.target_action.milestones;
        assert_eq!(milestones.result_ready, operation.target_action.completion);
        assert_eq!(
            milestones.payload_released,
            operation.target_action.completion
        );
        assert_eq!(
            milestones.authority_released,
            operation.target_action.completion
        );
        assert_eq!(milestones.terminal, operation.target_action.completion);
    }
}

#[test]
fn authority_catalog_names_the_supplier_family_and_fragment() {
    use crate::{
        SystemAuthorityAttribution as Attribution, SystemAuthorityFamily as Family,
        SystemAuthorityFragment as Fragment, SystemAuthorityPairRelation as Relation,
    };

    let expected = [
        (
            "args_count",
            Family::Invocation,
            Fragment::InvocationSnapshot,
            Relation::Free,
        ),
        (
            "arg_get",
            Family::Invocation,
            Fragment::InvocationSnapshot,
            Relation::Free,
        ),
        (
            "open_read",
            Family::DirectoryRead,
            Fragment::DirectoryLookup,
            Relation::Free,
        ),
        (
            "read_at",
            Family::ReadFile,
            Fragment::FileRandomRead,
            Relation::Free,
        ),
        (
            "write_once",
            Family::Output,
            Fragment::OutputSequence,
            Relation::Ordered(Attribution::OutputBytes),
        ),
        (
            "open_directory",
            Family::DirectoryRead,
            Fragment::DirectoryLookup,
            Relation::Free,
        ),
        (
            "open_directory_source",
            Family::DirectoryRead,
            Fragment::DirectoryLookup,
            Relation::Free,
        ),
        (
            "directory_next",
            Family::DirectorySource,
            Fragment::DirectoryCursor,
            Relation::Ordered(Attribution::DirectoryEntries),
        ),
        (
            "open_file",
            Family::DirectoryRead,
            Fragment::DirectoryLookup,
            Relation::Free,
        ),
    ];
    for operation in crate::SYSTEM_OPERATIONS {
        let selected = expected
            .iter()
            .find(|(name, _, _, _)| *name == operation.spelling);
        match (operation.authority, selected) {
            (None, None) => {}
            (Some(authority), Some((_, family, fragment, relation))) => {
                assert_eq!(authority.parameter, 0);
                assert_eq!(authority.family, *family);
                assert_eq!(authority.fragment, *fragment);
                assert_eq!(
                    crate::system_authority_pair_relation(*family, *fragment, *fragment),
                    Some(*relation)
                );
            }
            pair => panic!("authority mismatch for {}: {pair:?}", operation.spelling),
        }
    }
}

#[test]
fn tcp_model_witness_has_pair_owned_relations() {
    use crate::{
        SystemAuthorityAttribution as Attribution,
        SystemAuthorityFamily::Tcp,
        SystemAuthorityFragment::{TcpControl, TcpReceive, TcpSend},
        SystemAuthorityPairRelation::{Exclusive, Free, Ordered},
        system_authority_pair_relation,
    };
    assert_eq!(
        system_authority_pair_relation(Tcp, TcpReceive, TcpReceive),
        Some(Ordered(Attribution::TcpReceiveBytes))
    );
    assert_eq!(
        system_authority_pair_relation(Tcp, TcpSend, TcpSend),
        Some(Ordered(Attribution::TcpSendBytes))
    );
    assert_eq!(
        system_authority_pair_relation(Tcp, TcpReceive, TcpSend),
        Some(Free)
    );
    assert_eq!(
        system_authority_pair_relation(Tcp, TcpSend, TcpReceive),
        Some(Free)
    );
    for fragment in [TcpReceive, TcpSend, TcpControl] {
        assert_eq!(
            system_authority_pair_relation(Tcp, TcpControl, fragment),
            Some(Exclusive)
        );
        assert_eq!(
            system_authority_pair_relation(Tcp, fragment, TcpControl),
            Some(Exclusive)
        );
    }
}

#[test]
fn current_family_tables_are_total_and_whole_resource_is_exclusive() {
    use crate::{
        SystemAuthorityAttribution as Attribution, SystemAuthorityFamily as Family,
        SystemAuthorityFragment as Fragment, SystemAuthorityPairRelation as Relation,
        system_authority_pair_relation,
    };
    for (family, fragment, self_relation) in [
        (
            Family::Invocation,
            Fragment::InvocationSnapshot,
            Relation::Free,
        ),
        (
            Family::DirectoryRead,
            Fragment::DirectoryLookup,
            Relation::Free,
        ),
        (Family::ReadFile, Fragment::FileRandomRead, Relation::Free),
        (
            Family::Output,
            Fragment::OutputSequence,
            Relation::Ordered(Attribution::OutputBytes),
        ),
        (
            Family::DirectorySource,
            Fragment::DirectoryCursor,
            Relation::Ordered(Attribution::DirectoryEntries),
        ),
    ] {
        assert_eq!(
            system_authority_pair_relation(family, fragment, fragment),
            Some(self_relation)
        );
        for pair in [
            (Fragment::WholeResource, Fragment::WholeResource),
            (Fragment::WholeResource, fragment),
            (fragment, Fragment::WholeResource),
        ] {
            assert_eq!(
                system_authority_pair_relation(family, pair.0, pair.1),
                Some(Relation::Exclusive)
            );
        }
    }
    assert_eq!(
        system_authority_pair_relation(
            Family::Output,
            Fragment::OutputSequence,
            Fragment::DirectoryCursor,
        ),
        None
    );
}

#[test]
fn combining_release_rows_from_multiple_families_is_unknown() {
    let close = |family| crate::SystemReleaseRow {
        target_action: crate::TargetAction::MAY_SUSPEND,
        capability_write: true,
        authority: crate::SystemReleaseAuthority::Known(crate::SystemAuthorityFacet {
            family,
            fragment: crate::SystemAuthorityFragment::WholeResource,
        }),
    };
    let combined = close(crate::SystemAuthorityFamily::DirectoryRead)
        .union(close(crate::SystemAuthorityFamily::ReadFile));
    assert_eq!(combined.authority, crate::SystemReleaseAuthority::Unknown);
}

#[test]
fn repeated_uses_deduplicate_by_parameter_family_and_fragment_only() {
    let source = br#"fn twice(output: own Output) -> result: own unit writes(output), allocates(heap) {
  let bytes = buffer_new(1_u64, 65_u8);
  region 'o {
    region 's {
      let first = write_once<'o, 's>(output: &'o output, source: &'s bytes, start: 0_u64, end: 1_u64);
      let second = write_once<'o, 's>(output: &'o output, source: &'s bytes, start: 0_u64, end: 1_u64);
    }
  }
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(program) = outcome else {
            panic!("repeated authority-use fixture must check: {outcome:?}");
        };
        let [usage] = program.data.functions[0].authority_summary.uses.as_slice() else {
            panic!("two same-fragment calls must retain one deduplicated use");
        };
        assert_eq!(usage.parameter, 0);
        assert_eq!(usage.family, crate::SystemAuthorityFamily::Output);
        assert_eq!(
            usage.fragment,
            crate::SystemAuthorityFragment::OutputSequence
        );
    });
}

#[test]
fn may_suspend_propagates_through_calls_and_derived_releases() {
    let source = br#"fn compute() -> result: own unit pure {
  return unit;
}

fn close_file(file: own ReadFile) -> result: own unit writes(file) {
  return unit;
}

fn forward(file: own ReadFile) -> result: own unit writes(file) {
  close_file(file: move file);
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  let done = compute();
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(program) = outcome else {
            panic!("target-action fixture must check: {outcome:?}");
        };
        let action = |name: &str| {
            program
                .data
                .functions
                .iter()
                .find(|function| function.name == name)
                .unwrap_or_else(|| panic!("missing function {name}"))
                .target_action
        };
        assert_eq!(action("compute"), TargetAction::INLINE);
        assert_eq!(action("main"), TargetAction::INLINE);
        assert_eq!(action("close_file"), TargetAction::MAY_SUSPEND);
        assert_eq!(action("forward"), TargetAction::MAY_SUSPEND);
    });
}

#[test]
fn residual_projection_releases_contribute_to_target_action() {
    let source = br#"struct Holder {
  file: ReadFile;
  data: buffer<u8>;
}

fn take_data(holder: own Holder) -> result: own buffer<u8> writes(holder) {
  return move holder.data;
}

fn forward(holder: own Holder) -> result: own buffer<u8> writes(holder) {
  return take_data(holder: move holder);
}

fn consume(data: own buffer<u8>) -> result: own unit pure {
  return unit;
}

fn pass_data(holder: own Holder) -> result: own unit writes(holder) {
  consume(data: move holder.data);
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(program) = outcome else {
            panic!("residual-release target-action fixture must check: {outcome:?}");
        };
        let action = |name: &str| {
            program
                .data
                .functions
                .iter()
                .find(|function| function.name == name)
                .unwrap_or_else(|| panic!("missing function {name}"))
                .target_action
        };
        assert_eq!(action("take_data"), TargetAction::MAY_SUSPEND);
        assert_eq!(action("forward"), TargetAction::MAY_SUSPEND);
        assert_eq!(action("pass_data"), TargetAction::MAY_SUSPEND);
        assert_eq!(action("consume"), TargetAction::INLINE);
    });
}
