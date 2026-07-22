#![forbid(unsafe_code)]
#![deny(missing_docs)]

//! The Whitefoot research compiler.
//!
//! The crate currently contains the v0.10 source frontend. The lower-level
//! stages remain visible while the resolver is built directly on the
//! canonical syntax tree; they are private implementation APIs, not protocols.

mod lexer;
mod source;
mod spec;
mod syntax;

pub use lexer::*;
pub use source::*;
pub use spec::*;
pub use syntax::grammar::*;
pub use syntax::terminal::*;
pub use syntax::*;
