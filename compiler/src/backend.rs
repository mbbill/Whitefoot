//! Conservative textual LLVM emission for exact Whitefoot v0.15.

mod emitter;
mod target;

#[cfg(test)]
mod tests;

pub use emitter::{BackendFailure, emit_llvm_v0_15};
