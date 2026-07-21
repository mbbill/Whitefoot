#![forbid(unsafe_code)]
#![deny(missing_docs)]

//! Independent static grammar extraction and strong-LL(2) evidence engine.

mod document;
mod ebnf;
mod emit;
mod float_contract;
mod grammar;
mod hash;
mod lexical;
mod ll2;
mod terminal;
mod wire;

use emit::{failure_report, success_report};
use wire::Frame;

/// Process one complete `WFGRAMV1` frame into one classified raw report.
///
/// Classified input, extraction, resource, and internal failures are encoded
/// in-band. The binary reserves nonzero process exits for failures outside the
/// engine contract, such as an unreadable standard stream.
pub fn process_frame(input: &[u8]) -> Vec<u8> {
    match Frame::parse(input).and_then(crate::emit::analyze) {
        Ok(report) => success_report(report),
        Err(failure) => failure_report(failure),
    }
}
