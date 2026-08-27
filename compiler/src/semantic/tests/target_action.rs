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
            milestones.ownership_released,
            operation.target_action.completion
        );
        assert_eq!(milestones.terminal, operation.target_action.completion);
    }
}

#[test]
fn system_catalog_projects_one_unified_state_row() {
    let row = |name| {
        crate::SYSTEM_OPERATIONS
            .iter()
            .find(|operation| operation.spelling == name)
            .unwrap_or_else(|| panic!("missing system operation {name}"))
    };
    let write = row("write_once");
    assert_eq!(crate::operation_state_effects(write), (vec![0, 1], vec![0]));
    assert!(matches!(
        write.parameters[0].mode,
        crate::SystemParameterMode::UniqueBorrow(_)
    ));

    let next = row("directory_next");
    assert_eq!(
        crate::operation_state_effects(next),
        (vec![0, 1], vec![0, 1])
    );
    assert!(matches!(
        next.parameters[0].mode,
        crate::SystemParameterMode::UniqueBorrow(_)
    ));
}

#[test]
fn release_rows_union_state_transitions_without_a_second_identity_system() {
    let close = crate::SystemReleaseRow {
        target_action: crate::TargetAction::MAY_SUSPEND,
        state_write: true,
    };
    let combined = close.union(crate::SystemReleaseRow::EMPTY);
    assert!(combined.state_write);
    assert_eq!(combined.target_action, crate::TargetAction::MAY_SUSPEND);
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

fn take_data(holder: own Holder) -> result: own buffer<u8> writes(holder.file) {
  return move holder.data;
}

fn forward(holder: own Holder) -> result: own buffer<u8> writes(holder.file) {
  return take_data(holder: move holder);
}

fn consume(data: own buffer<u8>) -> result: own unit pure {
  return unit;
}

fn pass_data(holder: own Holder) -> result: own unit writes(holder.file) {
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
