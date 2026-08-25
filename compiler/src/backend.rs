//! Conservative textual LLVM emission for the active Whitefoot specification.

mod emitter;
mod qualification;
mod stack_ledger;
mod target;

#[cfg(test)]
mod tests;

pub use emitter::{
    BackendFailure, COMPLETION_CONTRACT_HEADER, COMPLETION_PLATFORM_FILE_NAME,
    COMPLETION_PLATFORM_HEADER, COMPLETION_PLATFORM_SOURCE, COMPLETION_RUNTIME_SOURCE,
    FLOOR_RUNTIME_SOURCE, FLOOR_STACK_BYTES, PARALLEL_RUNTIME_SOURCE, emit_llvm,
    module_requires_completion_runtime, module_requires_parallel_runtime,
};
pub use stack_ledger::stack_ledger;
