//! Conservative textual LLVM emission for the active Whitefoot specification.

mod emitter;
mod target;

#[cfg(test)]
mod tests;

pub use emitter::{BackendFailure, emit_llvm};
