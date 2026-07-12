# Rust Writer's Guardrails

You are writing safe Rust (edition 2021, no external crates).

- `unsafe` is forbidden.
- No unchecked arithmetic where the task asks for checked/trapping behavior:
  use `checked_*` with explicit handling, or index/assert so violations panic.
- Errors the task calls recoverable are `Result` values; use `?` freely.
- The program must be a single self-contained `main.rs`.
- Success is exit code 0 with any output the task requires on stdout.
