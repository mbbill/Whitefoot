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
pub mod spec_identity;
mod syntax;

/// The parallel runtime a module that hands work out must be linked against,
/// and the predicate that decides whether one must.
pub use backend::{
    FLOOR_RUNTIME_SOURCE, PARALLEL_RUNTIME_SOURCE, module_requires_parallel_runtime,
};
pub use driver::*;
pub use lexer::*;
/// The compile-time choice of whether the backend actualizes the permission
/// judgment's overlap groups.
pub use lowering::OverlapLowering;
pub use resolution::*;
pub use source::*;
pub use spec::*;
pub use syntax::grammar::*;
pub use syntax::terminal::*;
pub use syntax::*;

pub(crate) use backend::*;
pub(crate) use lowering::*;
pub(crate) use semantic::*;
