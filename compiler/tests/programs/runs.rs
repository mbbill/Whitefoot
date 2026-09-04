use super::support::{compile_and_run, compile_program};

/// [BLK-1, BLK-2, BLK-3] the two runs at execution: a formation row, the four
/// boundary operations over the back and the front, and the window subscript
/// at a wrapped window.
///
/// The exit code is the program's own report: it observes the queue's order,
/// the element a wrapped subscript reads, and the length the run is left with,
/// so a lowering that computed any physical slot wrongly reports a nonzero
/// code rather than passing quietly.
#[test]
fn a_run_is_a_queue_whose_window_wraps() {
    let llvm = compile_program("run_queue.wf");
    // The window subscript is the one conditional subtract [BLK-1] fixes,
    // and the boundary store goes through the run's own frame slot.
    assert!(llvm.contains("select i1"));
    assert!(llvm.contains("getelementptr inbounds { [4 x i8], i64, i64 }"));

    let output = compile_and_run(&llvm);
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}
