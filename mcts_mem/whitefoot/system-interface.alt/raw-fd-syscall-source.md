- System access appears in source as raw syscall numbers and integer file descriptors with a thin native ABI and complete OS reach.
- Descriptor identity is an ordinary copyable integer resolved through the process-global descriptor table; close is a manual call.

## Moves

- 2026-08-05 (8f7055fc) replaced by [[system-interface]]: raw syscalls and integer fds in source expose forgeable identities, an implicit global fd table, manual close, weak effect precision, poor Windows portability, and an unchecked pointer wall; they remain permitted only inside compiler-owned target code (sourced)
