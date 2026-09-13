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
    mod stream;
    mod support;
    mod text;
    // Directory-walking programs link ordinary prelude implementations on
    // every host this repository gates on.
    mod traversal;
    mod wfgrep;
    mod wide_scan;
}
