//! Conservative textual LLVM emission for the active Whitefoot specification.

mod emitter;
mod qualification;
mod stack_ledger;
mod target;

#[cfg(test)]
mod tests;

pub use emitter::{
    BackendFailure, FLOOR_RUNTIME_SOURCE, FLOOR_STACK_BYTES, PARALLEL_RUNTIME_SOURCE, emit_llvm,
    module_requires_parallel_runtime,
};
pub use stack_ledger::stack_ledger;
