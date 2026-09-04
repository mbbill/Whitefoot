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
    mod runs;
    mod signal;
    mod support;
    mod text;
    // The directory-walking flagships. Every case in these two modules that
    // builds a program builds one that enumerates a directory, so both need a
    // target with an approved [SYS-14] directory-enumeration row. Every triple
    // `backend/qualification.rs` recognizes now has one, so both modules build
    // and run on every host this repository gates on.
    mod traversal;
    mod wfgrep;
    mod wide_scan;
}
