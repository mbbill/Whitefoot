//! Native runtime source assets used by the build driver.
//! These files are linked implementations, with no source-checking metadata.

/// The C representation of the ordinary prelude library and launcher values.
pub const ORDINARY_VALUES_HEADER: &str = include_str!("ordinary_values.h");
/// Ordinary native function definitions linked through the regular call ABI.
pub const ORDINARY_VALUES_SOURCE: &str = include_str!("ordinary_values.c");
/// Ordinary linked view definitions using the shared Whitefoot callable ABI.
pub const ORDINARY_VALUES_LLVM: &str = include_str!("ordinary_values.ll");

/// The finite completion core contract embedded in the compiler.
pub const COMPLETION_CONTRACT_HEADER: &str = include_str!("completion/contract.h");
/// The typed file-adapter contract embedded in the compiler.
pub const COMPLETION_FILE_ADAPTER_HEADER: &str = include_str!("completion/file_adapter.h");
/// The compiler-owned file-completion bridge contract embedded in the compiler.
pub const COMPLETION_BRIDGE_HEADER: &str = include_str!("completion/bridge.h");
/// The target-guarded Linux io_uring adapter contract embedded in the compiler.
pub const COMPLETION_LINUX_IO_URING_HEADER: &str = include_str!("completion/linux_io_uring.h");
/// The Windows IOCP ring's contract embedded in the compiler.
pub const COMPLETION_WINDOWS_IOCP_HEADER: &str = include_str!("completion/windows_iocp.h");
/// The typed file-adapter's POSIX leaf contract embedded in the compiler.
pub const COMPLETION_FILE_POSIX_HEADER: &str = include_str!("completion/file_posix.h");
/// The address vocabulary every socket engine shares, on every platform.
pub const COMPLETION_SOCKET_ADDRESS_HEADER: &str = include_str!("completion/socket_address.h");
/// The finite completion core implementation embedded in the compiler.
pub const COMPLETION_RUNTIME_SOURCE: &str = include_str!("completion/runtime.c");
/// The host's one wait set, which `runtime.c` and the file adapter sleep on.
pub const COMPLETION_WAIT_HOST_SOURCE: &str = include_str!("completion/wait_host.c");
/// Windows's one wait set, the twin of the above.
pub const COMPLETION_WAIT_WINDOWS_SOURCE: &str = include_str!("completion/wait_windows.c");
/// The typed file-adapter implementation embedded in the compiler.
pub const COMPLETION_FILE_ADAPTER_SOURCE: &str = include_str!("completion/file_adapter.c");
/// The file adapter's POSIX host leaf: one host call per request kind.
pub const COMPLETION_FILE_POSIX_SOURCE: &str = include_str!("completion/file_posix.c");
/// The file adapter's Windows host leaf, the twin of the above.
pub const COMPLETION_FILE_WINDOWS_SOURCE: &str = include_str!("completion/file_windows.c");
/// The compiler-owned file-completion bridge embedded in the compiler.
pub const COMPLETION_BRIDGE_SOURCE: &str = include_str!("completion/bridge.c");
/// The target-guarded Linux io_uring adapter embedded in the compiler.
pub const COMPLETION_LINUX_IO_URING_SOURCE: &str = include_str!("completion/linux_io_uring.c");
/// The target-guarded Windows IOCP ring embedded in the compiler.
pub const COMPLETION_WINDOWS_IOCP_SOURCE: &str = include_str!("completion/windows_iocp.c");

/// The scheduler core's contract embedded in the compiler.
///
/// The completion record begins with a `wf_sched_record` and every publication
/// goes through `wf_sched_complete`, so a link that carries the completion
/// runtime carries the core beside it
/// (`research/investigations/io-model/PARK-ON-MISS.md` §5, §7).
pub const SCHED_CORE_HEADER: &str = include_str!("sched/core.h");
/// The scheduler core embedded in the compiler.
pub const SCHED_CORE_SOURCE: &str = include_str!("sched/core.c");
/// The seven primitives the core reaches shared state through.
pub const SCHED_PRIM_HEADER: &str = include_str!("sched/prim.h");
/// The host's implementation of those primitives.
pub const SCHED_PRIM_HOST_SOURCE: &str = include_str!("sched/prim_host.c");
/// Windows's implementation of the same set, the twin of the above.
pub const SCHED_PRIM_WINDOWS_SOURCE: &str = include_str!("sched/prim_windows.c");
/// The one stack switch, shared by the host primitives and the enumerator.
pub const SCHED_SWITCH_HEADER: &str = include_str!("sched/switch.h");
/// The platform layer over the core: its one instance, the startup policy and
/// the emitted module's `wf__par_*` ABI (design §7's platform layer).
pub const SCHED_ENTRY_HEADER: &str = include_str!("sched/entry.h");
/// That layer's implementation, which replaces `par_runtime.c`.
pub const SCHED_ENTRY_SOURCE: &str = include_str!("sched/entry.c");

/// Windows host primitives used by ordinary linked function definitions.
pub const WINDOWS_RUNTIME_HEADER: &str = include_str!("windows_runtime.h");
/// Windows implementations of those private host primitives.
pub const WINDOWS_RUNTIME_SOURCE: &str = include_str!("windows_runtime.c");
