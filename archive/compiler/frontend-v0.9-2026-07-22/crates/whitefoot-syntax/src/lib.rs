#![forbid(unsafe_code)]
#![deny(missing_docs)]

//! Resource-bounded canonical syntax construction for Whitefoot v0.9.
//!
//! The implementation classifies every formed token against the complete
//! context-free terminal set, constructs one private iterative LL(2)
//! derivation, finalizes its topology and source binding, and audits exact
//! FORM-2 bytes from that same tree before publishing [`CanonicalSyntaxUnit`].
//! This crate does not perform semantic checking or create portable artifacts.

mod classifier;
mod outcome;
mod parser;

pub use classifier::classify_terminals_v0_9;
pub use outcome::{
    ClassifiedBundle, ClassifiedToken, TerminalCompilerFailure, TerminalInvocationFailure,
    TerminalIssue, TerminalIssueOwner, TerminalLimit, TerminalLimits, TerminalOutcome,
    TerminalResourceFailure, TerminalStorage,
};
pub use parser::{
    BundleSourceExtent, CanonicalCompilerFailure, CanonicalIssue, CanonicalLimit, CanonicalLimits,
    CanonicalLocation, CanonicalOutcome, CanonicalResourceFailure, CanonicalStorage,
    CanonicalSyntaxUnit, ExpectedTerminalsV0_9, FinalizeCompilerFailure, FinalizeLimit,
    FinalizeLimits, FinalizeOutcome, FinalizeResourceFailure, FinalizeStorage, FinalizedBundle,
    NodePath, ParseCompilerFailure, ParseInvocationFailure, ParseLimit, ParseLimits, ParseOutcome,
    ParseResourceFailure, ParseStorage, ParsedBundle, SyntaxCoordinate, SyntaxIssue,
    SyntaxRuleV0_9, audit_canonical_v0_9, finalize_v0_9, parse_v0_9,
};

#[cfg(test)]
mod tests;
