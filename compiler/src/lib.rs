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
    Architecture, COMPLETION_BRIDGE_HEADER, COMPLETION_BRIDGE_SOURCE, COMPLETION_CONTRACT_HEADER,
    COMPLETION_FILE_ADAPTER_HEADER, COMPLETION_FILE_ADAPTER_SOURCE, COMPLETION_FILE_POSIX_HEADER,
    COMPLETION_FILE_POSIX_SOURCE, COMPLETION_FILE_WINDOWS_SOURCE, COMPLETION_LINUX_IO_URING_HEADER,
    COMPLETION_LINUX_IO_URING_SOURCE, COMPLETION_RUNTIME_SOURCE, COMPLETION_WAIT_HOST_SOURCE,
    COMPLETION_WAIT_WINDOWS_SOURCE, COMPLETION_WINDOWS_IOCP_HEADER, COMPLETION_WINDOWS_IOCP_SOURCE,
    FLOOR_RUNTIME_SOURCE, FLOOR_STACK_BYTES, FLOOR_WINDOWS_RUNTIME_SOURCE, SCHED_CORE_HEADER,
    SCHED_CORE_SOURCE, SCHED_ENTRY_HEADER, SCHED_ENTRY_SOURCE, SCHED_PRIM_HEADER,
    SCHED_PRIM_HOST_SOURCE, SCHED_PRIM_WINDOWS_SOURCE, SCHED_SWITCH_HEADER, WINDOWS_RUNTIME_HEADER,
    WINDOWS_RUNTIME_SOURCE, module_requires_completion_runtime, module_requires_parallel_runtime,
    stack_ledger,
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
