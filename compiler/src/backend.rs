//! Conservative textual LLVM emission for the active Whitefoot specification.

mod emitter;
mod qualification;
mod target;

#[cfg(test)]
mod tests;

pub use emitter::{
    BackendFailure, FLOOR_RUNTIME_SOURCE, PARALLEL_RUNTIME_SOURCE, emit_llvm,
    module_requires_parallel_runtime,
};
