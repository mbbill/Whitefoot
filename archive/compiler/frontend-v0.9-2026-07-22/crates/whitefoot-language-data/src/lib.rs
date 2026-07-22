#![forbid(unsafe_code)]
#![deny(missing_docs)]

//! Exact, immutable language tables for approved Whitefoot specifications.
//!
//! This crate names grammar predicates and their spelling languages. It does
//! not inspect source bundles, choose parser alternatives, or grant syntax or
//! semantic authority.

mod terminal;

pub use terminal::{
    ALL_FIXED_TERMINALS_V0_9, ALL_TERMINAL_PREDICATES_V0_9, FixedTerminalV0_9,
    TERMINAL_CONTRACT_SPEC_V0_9, TerminalPredicateV0_9, TerminalSetV0_9, is_digits_v0_9,
    is_identifier_v0_9, is_label_v0_9, is_literal_v0_9, is_operation_name_v0_9,
    is_region_identifier_v0_9, is_string_v0_9, is_type_identifier_v0_9,
};
