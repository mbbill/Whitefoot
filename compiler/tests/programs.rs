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
    /// The directory-walking flagships, both of which enumerate a directory in
    /// every case they have.
    ///
    /// Host-limited to a target with an approved [SYS-14] directory-enumeration
    /// row, the same limit `backend/tests.rs` states over the cost census:
    /// `backend/qualification.rs` deliberately gives Linux no such row, because
    /// `getdents64` writes no per-entry name length and the portable record the
    /// emitted shim fills needs one, so qualification reports
    /// `MissingMapping(Operation(12))` rather than pretending the facility is
    /// there. `dir_walk.wf` and `wfgrep.wf` do not compile on such a target, so
    /// there is no program here to run. The limit is the target table's, not
    /// these modules': the day a Linux enumeration row lands, these two lines
    /// go.
    #[cfg(target_os = "macos")]
    mod traversal;
    #[cfg(target_os = "macos")]
    mod wfgrep;
    mod wide_scan;
}
