//! Embedded translation units for the common completion contract.
//!
//! The compiler ships these exact bytes into every executable that names a
//! native completion adapter, and into every executable carrying the parallel
//! runtime because its I/O-frame publication entry shares the same contract.
//! The host-specific unit is selected while the trusted runtime is built; no
//! language value, semantic operation ID, or emitted dispatch table selects a
//! backend at run time.

/// The shared C contract included by the runtime and parallel scheduler.
pub const COMPLETION_CONTRACT_HEADER: &str = include_str!("../completion/contract.h");

/// Internal host-primitives interface shared by the common and platform units.
pub const COMPLETION_PLATFORM_HEADER: &str = include_str!("../completion/platform.h");

/// Submission, mailbox, generation, loan, and fixed disk-pool implementation.
pub const COMPLETION_RUNTIME_SOURCE: &str = include_str!("../completion/runtime.c");

/// The host backend source selected as a build/TCB decision.
#[cfg(target_os = "macos")]
pub const COMPLETION_PLATFORM_SOURCE: &str = include_str!("../completion/kqueue.c");

/// The file name assigned to the selected host backend in a transient build.
#[cfg(target_os = "macos")]
pub const COMPLETION_PLATFORM_FILE_NAME: &str = "completion_kqueue.c";

/// The host backend source selected as a build/TCB decision.
#[cfg(target_os = "linux")]
pub const COMPLETION_PLATFORM_SOURCE: &str = include_str!("../completion/io_uring.c");

/// The file name assigned to the selected host backend in a transient build.
#[cfg(target_os = "linux")]
pub const COMPLETION_PLATFORM_FILE_NAME: &str = "completion_io_uring.c";

/// The host backend source selected as a build/TCB decision.
#[cfg(target_os = "windows")]
pub const COMPLETION_PLATFORM_SOURCE: &str = include_str!("../completion/iocp.c");

/// The file name assigned to the selected host backend in a transient build.
#[cfg(target_os = "windows")]
pub const COMPLETION_PLATFORM_FILE_NAME: &str = "completion_iocp.c";

/// True when an emitted module calls one of the native completion adapters.
///
/// The reserved `wf__io_` prefix cannot be produced by a source IDENT. The
/// predicate therefore reads the same symbol family the linker supplies,
/// without inspecting source shape, function names, or operation spellings.
pub fn module_requires_completion_runtime(module: &str) -> bool {
    module.contains("@wf__io_")
}
