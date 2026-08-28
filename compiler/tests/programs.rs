#![forbid(unsafe_code)]

mod programs {
    mod binary;
    mod generics;
    mod hashing;
    mod heap;
    mod image;
    mod network;
    mod numerics;
    mod parallel;
    mod raw_deflate;
    mod signal;
    mod support;
    mod text;
    // The directory-walking flagships. Every case in these two modules that
    // builds a program builds one that enumerates a directory, and only a
    // target with an approved [SYS-14] directory-enumeration row compiles it —
    // the same limit `backend/tests.rs` states over the cost census.
    // `backend/qualification.rs` deliberately gives Linux no such row, because
    // `getdents64` writes no per-entry name length and the portable record the
    // emitted shim fills needs one, so qualification reports
    // `MissingMapping(Operation(12))` rather than pretending the facility is
    // there. `dir_walk.wf` and `wfgrep.wf` do not compile on such a target, so
    // there is no program to run.
    //
    // `wfgrep` names the host here because all twelve of its cases build the
    // program. `traversal` does not: five of its eight cases build
    // `dir_walk.wf` and carry the attribute themselves, and the other three
    // reject inline source at a numbered rule, which every stage reaches
    // before target qualification, so they run and say the same thing on
    // every host. An attribute on the module would hide those three too. The
    // limit is the target table's, not these modules': the day a Linux
    // enumeration row lands, the attribute below and the five in
    // `traversal.rs` go.
    mod traversal;
    #[cfg(target_os = "macos")]
    mod wfgrep;
    mod wide_scan;
}
