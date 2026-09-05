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

/// [BLK-1, MSR-3, MSR-5, SET-2] the fixed-run half of the container design's
/// own library, executing: the two constructions, the transposing removal, the
/// two checked boundary forms, and the drain that returns a wrapped window to
/// its origin.
///
/// Every one of the six is written in the design's spelling and proves its own
/// contract, so the test is evidence for the surface as much as for the
/// lowering: `take_at` needs the element-position `replace` and the arithmetic
/// requirement `at + 2_u64 <= len_of(vector)`; `try_place` and `try_take` need
/// a parameter's measure in an `ensures` to denote its entry datum; `vacant`
/// needs a run at an unbounded element type; and `rebase` needs the rebind that
/// carries `spare`'s measures onto `built`. The program checks what it built —
/// the order the drain preserves, the element the transposition moved, and the
/// head the drain leaves at zero — so a wrong physical slot reports a nonzero
/// code rather than passing quietly.
#[test]
fn the_fixed_run_library_proves_and_runs() {
    let llvm = compile_program("fixed_run_library.wf");
    // The element-position `replace` goes through the window, so the store's
    // offset is the same conditional subtract a read uses [BLK-1].
    assert!(llvm.contains("select i1"));

    let output = compile_and_run(&llvm);
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}
