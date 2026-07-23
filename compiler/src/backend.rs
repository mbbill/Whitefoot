//! Conservative textual LLVM emission for exact Whitefoot v0.14.

mod emitter;

#[cfg(test)]
mod tests;

pub use emitter::{BackendFailure, emit_llvm_v0_14};
