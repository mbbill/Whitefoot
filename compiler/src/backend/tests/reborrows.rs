use super::*;

#[test]
fn statement_scoped_child_reborrows_resume_their_parent() {
    let llvm = compile(include_bytes!(
        "../../../../tests/conformance/cases/x-child-reborrow-run.wf"
    ));
    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}
