#![forbid(unsafe_code)]
#![deny(missing_docs)]

//! The Whitefoot research compiler.
//!
//! The crate contains one path for the active specification, from ordered sources through the
//! frontend and direct resolver into semantic and ownership checking, a
//! target-independent typed control-flow IR, conservative textual LLVM, and a
//! host compiler executable. These stages remain evolvable implementation
//! APIs, not stable protocols.

mod backend;
mod driver;
mod lexer;
mod lowering;
mod resolution;
mod semantic;
mod source;
mod spec;
mod syntax;

pub use driver::*;
pub use lexer::*;
pub use resolution::*;
pub use source::*;
pub use spec::*;
pub use syntax::grammar::*;
pub use syntax::terminal::*;
pub use syntax::*;

pub(crate) use backend::*;
pub(crate) use lowering::*;
pub(crate) use semantic::*;
