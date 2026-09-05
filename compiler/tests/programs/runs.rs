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

/// [BLK-2, PROV-1, MSR-1] the store-backed half of the two runs at execution:
/// one bump extent reserved in the entry's own frame, the proved take and the
/// checked take over it, and the refusal a take the extent cannot hold gets.
///
/// The program checks the store as well as the runs it hands out. It reads the
/// extent's own measures before and after each take, so a cursor that advanced
/// by the wrong `advance<T>(count)` reports a nonzero code; it observes that a
/// refused take leaves the cursor exactly where it was, which is the relation
/// the `None` arm publishes; and it fills a taken run through the boundary row
/// and reads the window back, so a descriptor pointing at the wrong byte of the
/// extent is visible rather than silent.
#[test]
fn a_bump_extent_hands_out_runs_and_refuses_the_one_it_cannot_hold() {
    let llvm = compile_program("arena_workspace.wf");
    // The extent is one frame reservation at its own written alignment, and
    // the take is pointer arithmetic inside it: no allocation call is emitted
    // [BLK-2, STOR-1].
    assert!(llvm.contains("getelementptr inbounds i8, ptr"));
    assert!(!llvm.contains("call ptr @malloc"));

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

/// [BLK-1, PROV-6, FORM-8] the one-level lift at execution: a frame-resident
/// run whose elements are store-backed runs, and the block pool 3.L.4 writes
/// over it.
///
/// The program is the design's pool with its two nominals removed, which is
/// exactly what this version cannot spell: a source nominal generic over its
/// store region has no instantiation, so `BlockPool['s]` and `Lease['s]` wait,
/// and the free list is the bare `FixedVector<Vector<'s, u8>, 8>` the struct
/// would have held. What it does exercise is everything the lift was for —
/// eight arena-backed runs carved into one run of runs, a lease taken off the
/// back boundary and a block returned to it, and both pool operations generic
/// over the store, which needs the formal region a run's *element* names to be
/// determined by the actual.
///
/// The exit code is the program's own report: it reads the free list's length
/// after the carve, the leased block's own `room_of`, and the length the
/// release leaves, so an element slot laid out or addressed wrongly reports a
/// nonzero code rather than passing quietly.
#[test]
fn a_run_of_store_backed_runs_is_a_block_pool() {
    let llvm = compile_program("block_pool.wf");
    // One slot holds a whole four-word descriptor, so the run's storage is
    // eight of them and the element load is that aggregate [BLK-1, OP-9].
    assert!(llvm.contains("[8 x { ptr, i64, i64, i64 }]"));
    // The blocks come out of the extent's own frame reservation: no
    // allocation call is emitted for a bump take [BLK-2, STOR-1].
    assert!(!llvm.contains("call ptr @malloc"));

    let output = compile_and_run(&llvm);
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}
