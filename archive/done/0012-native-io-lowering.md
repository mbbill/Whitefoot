# 0012 — Native I/O lowering

Frozen coordination history. This record reports how the task was carried
out; it is not authority.

- **Status:** `DONE` — landed on main at `2af4f8b` (ff), 2026-08-06
- **Authority (historical):** `ACTIVE` `docs/current-plan.md` Work item 2,
  wave 6; lead-authorized overlap with 0013 (this task landed first)

## Outcome

The last `UnsupportedSystemInterface` stop is removed: a complete first-slice
program compiles and runs end-to-end on the native macOS/Linux command
target. `open_read` is one direct `openat` against the capability's own
descriptor (no concatenation, no ambient cwd; the path pointer is the lease
itself — nothing allocated or copied). `read_once`/`write_once` implement
SYS-8 exactly: range validation traps first through the site's TrapSite
record, zero-length transfers issue no host call, one host transfer, exact
reported progress, observed-zero-read = ReadEnd, host zero-write =
WriteZero, BrokenPipe recoverable under the bootstrap normalization. SYS-7's
30-class mapping is one cold shared mapper with the two-field detail; Darwin
codes verified line-by-line against the SDK; the Linux column is the
asm-generic ABI, honestly recorded as unexercised on this host. §9.1
verified on the optimized module: wrappers fully inlined, exactly one host
call site per operation, one program-wide buffer allocation, no
memcpy/lock/stdio/indirect call, signal setup in the bootstrap only. All
seven accepting system conformance cases compile end-to-end (the 0011 flips
already covered promotion; nothing named 0012).

## Evidence and validation

- Landed commits: `7d8280d` (claim), `2af4f8b` (implementation). Both gates
  green by unpiped exit codes; lib tests 404 → 417 (system_io: 13); the
  lead's closure change also moved the 0011 record to done (missed at its
  closure) and refreshed two stale version labels.
- Honest evidence notes: WriteZero is emitted-shape-only evidence (no real
  host produces it for a nonempty request); UnexpectedEnd maps from no
  native code in v0.19.

## Follow-ups

- 0013 resumes: moves the three fixed host symbols behind HostFacilities at
  rebase, preserves the @write/@abort declaration dedupe (double declaration
  is a clang rejection), completes injection cases 2-4.
- 0014/0015 consume `test_directory()`/`build_executable()` and the
  system_io fixture/pipe/one-sink runners.
- Linux-column verification awaits a Linux host in the loop.
