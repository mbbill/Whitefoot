#![forbid(unsafe_code)]

#[path = "support/link.rs"]
mod link_support;

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
    mod traversal;
    mod wfgrep;
    mod wide_scan;
}
