#![forbid(unsafe_code)]

use memchr::memmem::Finder;
use wf_literal_line_ceiling::literal_line;

#[path = "../../harness_common.rs"]
mod harness_common;

fn main() {
    harness_common::main_with(
        |needle| Finder::new(needle).into_owned(),
        |finder, haystack, needle| literal_line(finder, haystack, needle),
    );
}
