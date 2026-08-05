- A program's system access is exactly its declared typed entry inputs under a declared program kind; no ambient authority exists, unused inputs are omitted, and the unlabelled no-input entry remains admissible.
- Source distinguishes immutable values, shared capabilities without caller-visible cursors, and unique stateful resources, expressed with the existing `own`/`&`/`&uniq` modes and no new writer keywords.
- Parallel use of one logical service is an explicit family-defined construction — split, controller/port, or a scoped shared borrow of a Shareable capability; a cursor or output owner is never implicitly shared.
- Exact `external` and `blocks` categories extend the effect row from [[effects]]; sequential external calls keep source program order, and resource identity and alias facts live in checked IR rather than parameterized source effects.
- Synchronous hot I/O is one-attempt with operation-specific outcomes: a prelude `Result` instantiation for two-outcome operations, a bespoke enum beyond that, a closed portable error-class set, caller-owned initialized buffers, and at most one host transfer per call.
- Arguments and paths are lossless target-indexed host strings with explicit fallible UTF-8 conversion; the first slice's string and path values are zero-copy inline leases whose command-lifetime argv backing is a required target-qualification guarantee.
- Every resource family has a compiler-owned contract covering states, aliases, owner disposition on every outcome, concurrency, cleanup, and a cross-platform floor, and carries one of three completion policies: release-complete, explicitly abandonable, or completion-required; traps keep whole-process abort with no language cleanup.
- System operations carry target-independent semantic IDs bound by a static (spec version, ID, target, program kind) qualification table with direct native lowering; hot paths admit no per-call dispatch, handle-table lookup, target tag, or global lock.
- System names resolve from a distinct compiler-owned declaration domain; [[declaration-home]] fixes it.
- The selection governs the v0.18 candidate batch and every later family as additive true subsets; the v0.17 compiler has no system path yet.

## Facts

- 2026-08-05 rationale: the owner selected this architecture from the dossier's alternative table after a four-critic, 31-issue adversarial review resolved every issue by evidence with none escalated; the dossier and its decision record are the canonical evidence (research/investigations/system-capability-architecture/). (sourced)
- 2026-08-05 statement: WASI evidence shaped the selection without becoming the contract — deny-by-default preopened authority, unforgeable owned/borrowed resource handles, and separated clock/random capabilities survive review, while its Unicode-only paths, pollable composition failure, and missing caller-buffer route are recorded anti-lessons. (sourced)

## Moves

- 2026-08-05 (8f7055fc) replaced [[raw-fd-syscall-source]]: raw syscalls and integer fds in source expose forgeable identities, an implicit global fd table, manual close, weak effect precision, poor Windows portability, and an unchecked pointer wall; they remain permitted only inside compiler-owned target code (sourced)
- 2026-08-05 (8f7055fc) replaced [[ambient-system-functions]]: ambient system functions hide access and create inter-function channels against FN-7's no-global rationale; system use invisible in signatures cannot be narrowed, tested, or parallelized by ownership (sourced)
- 2026-08-05 (8f7055fc) replaced [[affine-process-object]]: one permanently retained affine Process object makes every operation contend for the same unique holder, falsely serializing files, output, networking, clocks, and workers; making it shared would need a central lock or hidden aliasing (sourced)
- 2026-08-05 (8f7055fc) replaced [[wasi-source-contract]]: a literal WASI source contract imports Unicode-only paths, no guaranteed caller-buffer or zero-copy route, async tied to Component Model costs, and an incomplete threads and process surface chosen for cross-language components rather than Whitefoot ownership; WASI remains a possible target implementation for operations it can supply (sourced)
