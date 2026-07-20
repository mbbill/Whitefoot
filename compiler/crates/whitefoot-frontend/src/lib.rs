#![forbid(unsafe_code)]
#![deny(missing_docs)]

//! Lossless, resource-bounded frontend components for Whitefoot v0.8.
//!
//! The current entry point performs byte-preserving lexical partitioning. A
//! complete lexical result is not a language-acceptance verdict: parsing,
//! canonical formatting, semantic checking, and normative diagnostics remain
//! separate stages.

mod outcome;
mod scanner;
mod token;

pub use outcome::{
    LexCompilerFailure, LexLimit, LexLimits, LexOutcome, LexResourceFailure, LexStorage,
    LexedBundle, SourceIssue, SourceIssueKind,
};
pub use scanner::lex_v0_8;
pub use token::{Lexeme, Token, TokenId, TokenKind, Trivia, TriviaKind};

#[cfg(test)]
mod tests;
