# 0012 — Native I/O lowering

Live coordination record. It reports how authorized work is being carried
out; it is not authority and cannot expand Work item 2.

- **Status:** `IN PROGRESS`
- **Owner:** executor agent `exec-0012`
- **Workspace:** worktree `agent-a3e638de31b82da80`, branch
  `worktree-agent-a3e638de31b82da80`
- **Base revision:** `eca0078` (`docs: close task 0011 and complete the
  v0.19 corpus pin`)
- **Authority:** same Work item 2 bullet as task 0011 ("target-qualification
  table plus the static native macOS/Linux lowering"). Implements the active
  `spec/kernel-spec-v0.19.md`'s `PATH-2` (directory-relative resolution);
  `SYS-8` (one-attempt transfer semantics for `read_once`/`write_once`),
  `SYS-10`/`SYS-11` (`DirectoryRead`/`ReadFile` contracts and release), and
  `SYS-7` (`IoError` representation, including the two-field inline detail);
  under `QUAL-3`'s emitted shape, `TRAP-1`, and `EFF-5`; and dossier
  §7.1/§7.2/§7.3/§9's `open_read`/`read_once`/`write_once`/release rows.
  (The planned record was written against v0.18; v0.19 is the active
  version and carries these rules unchanged.) Claimable only while
  `docs/current-plan.md` remains `ACTIVE`.

## Goal

Implement native lowering for `open_read`, `read_once`, `write_once`, and
the `DirectoryRead`/`ReadFile`/`Output` release actions: one direct libc
open-relative call and `IoError` mapping for `open_read`; bounds-checked,
at-most-one-host-transfer lowering for `read_once`/`write_once` with the
exact `ReadBytes`/`ReadEnd`/`ReadFailed` and `Ok`/`Err` outcome mapping;
one native close attempt (never retried after an ambiguous `EINTR`) for
`DirectoryRead`/`ReadFile` release; and a no-op logical detach for
`Output` release.

## Direction and invariants

- `open_read` resolves against the target's own directory-relative
  facility (for example `openat`) — never string-prefix concatenation onto
  an ambient cwd (`PATH-2`).
- Every buffer range is validated before any target call or destination
  write; a zero-length range never reports `ReadEnd` and never issues a
  host transfer; a nonempty read never returns `ReadBytes(0)`; a write
  never returns `Ok(0)` (a host zero-write is `Err(WriteZero)`).
- One source `read_once`/`write_once` maps to at most one host transfer
  attempt; a reported interruption returns `Interrupted` rather than being
  silently retried.
- A consuming close invalidates the handle on success and error; the
  lowering must never retry a numeric fd after `close` reports `EINTR`.
- `Output` release is logical detach only — no close, no flush, no target
  call; OS process teardown owns descriptor close.

## Method

Extend the `backend/emitter/system.rs` module task 0011 introduces with
the I/O operation cluster, reusing task 0011's qualification-table
registration pattern. Map the closed 30-class `IoError` set to native
errno values (or platform equivalents) via one cold mapper function, with
no wildcard arm and no silent narrowing. Per `SYS-7`, each `IoError`
variant carries two declared fields, not one: fill `code: u32` with the
value-preserving native error code and `origin: u8` with a target-owned
discriminator identifying which native facility produced it (zero when the
target supplies no value for a field); a target unable to represent its
detail in these two fields maps the class to `Other` instead.

## Scope and expected touch set

- `compiler/src/backend/emitter/system.rs` (I/O operation lowering,
  sibling to task 0011's argument/path lowering)
- `compiler/src/backend/target.rs` (additional qualification-table entries
  for the I/O semantic IDs)
- New file under `compiler/src/backend/tests/` for I/O lowering shape
  assertions

## Progress

- Done: claimed at base `eca0078`; read `docs/WORKFLOW.md`, the governing
  `SYS-7`/`SYS-8`/`SYS-9`–`SYS-12`/`PATH-2`/`QUAL-3`/`TRAP-1`/`EFF-5`
  rules, and task 0011's landed surfaces.
- Current: native emission for `open_read`, `read_once`, and `write_once`
  plus the `IoError` mapping table, and removal of the last
  `UnsupportedSystemInterface` stop.
- Next: §9.1 optimized-module inspection, gates, record closure hand-off.

## Dependencies and integration order

Depends on task 0011 (shares the qualification-table mechanism and
module); 0011 is terminal (`DONE`, landed at `61936d6`, closed by
`eca0078`). Runs concurrently with task 0013 (wave 6) — neither depends on
the other, but both touch the backend qualification and target surfaces
(`compiler/src/backend/qualification.rs`, `compiler/src/backend/target.rs`,
`compiler/src/backend/emitter/system.rs`). Lead-granted overlap
authorization, recorded here as the agreed integration order: **this task
lands first; task 0013 refreshes its base onto it, rereads the changed
qualification surface, rebases, and reruns its gates.** Tasks 0014, 0015,
and 0016 depend on this task.

## Validation

`make -C compiler check`; compiled-and-run tests for open/read/write
against real files (the fixture corpus task 0014 adds); codegen inspection
confirming no allocation, copy, or lock on the hot transfer path. A
claimed task lands only through lead review per the executor lane in
`docs/WORKFLOW.md`.

## Stop condition

A blocker, plan defect, or specification/compiler discrepancy stops the
task with an honest report and reproduction evidence rather than a
workaround, a weakened check, or a narrowed deliverable.

## Closure

Move this record to `docs/done/` with a final status, the landed commits,
the canonical evidence, and any remaining follow-up links, in the same
integration change.

## Done-when

`open_read`/`read_once`/`write_once` and their release actions lower to
the required native shape and pass a minimal compiled-and-run smoke test
(for example, reading a small fixture file to completion); `make -C
compiler check` green.
