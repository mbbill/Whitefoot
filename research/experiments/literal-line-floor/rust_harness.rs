#![forbid(unsafe_code)]

extern crate wf_literal_line_control;

mod harness_common;

fn main() {
    harness_common::main_with(
        |_| (),
        |_, haystack, needle| wf_literal_line_control::literal_line(haystack, needle),
    );
}
