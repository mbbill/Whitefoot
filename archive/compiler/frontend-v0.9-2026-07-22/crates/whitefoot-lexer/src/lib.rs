#![forbid(unsafe_code)]
#![deny(missing_docs)]

//! Lossless, resource-bounded lexical shapes for Whitefoot v0.9.
//!
//! The versioned entry points perform byte-preserving lexical partitioning. A
//! complete lexical result is not a language-acceptance verdict: parsing,
//! terminal classification, canonical formatting, semantic checking, and
//! normative diagnostics remain separate stages.

mod outcome;
mod scanner;
mod token;

pub use outcome::{
    LexCompilerFailure, LexLimit, LexLimits, LexOutcome, LexResourceFailure, LexStorage,
    LexedBundle, SourceIssue, SourceIssueKind,
};
pub use scanner::{lex_v0_8, lex_v0_9};
pub use token::{Lexeme, Token, TokenId, TokenKind, Trivia, TriviaKind};

#[cfg(test)]
mod tests;
