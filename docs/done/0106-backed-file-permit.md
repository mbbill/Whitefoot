# Batch 0106 — the backed `FilePermit` (specification candidate v0.42)

Branch: `io/t4-resource-relations` (PR #13), from `main` at `e2c7c0ca`.
Deliverables: the v0.42 candidate specification and everything derived from
it, the credit-backed permit in the runtime, the deletion of the descriptor
retirement ledger on every route and target, the rewritten corpus, and this
record. The plan is `docs/current-plan.md`, Batch 1; the ruling it implements
is constitution T4.

## Why

Under v0.41 a `FilePermit` promised nothing the host could run out of. A
`close` produced nothing the checker could see, so a staged pipeline
overlapped it with a later `open`, the host answered `EMFILE` for a schedule
the sequential program never produces, and the runtime hid that answer with a
process-wide descriptor retirement ledger: a refused open registered as a
waiter, awards were handed out in registration order, and a blocking direct
call held its thread while it waited for another engine to give a descriptor
back. T4 names that whole apparatus as the symptom of one missing API
relation. The relation is now on the API.

## 1. Language delta ([SYS-10], [SYS-11], [SYS-14], tables)

`spec/kernel-spec.md` is `Status: CANDIDATE v0.42 supersedes v0.41 <digest>`.

- `reserve_file(&uniq FileFactory) -> Result<FilePermit, IoError>`. The
  factory has a real capacity, at most the descriptors the target provides;
  a permit is one unit of it. Its `Err(ResourceExhausted)` is the program's
  own source-order outcome and arrives before any submission.
- `close_read(own ReadFile) -> FilePermit`, `close_directory(own
  DirectoryRoot) -> FilePermit`, `close_directory_source(own DirectorySource)
  -> FilePermit`: explicit closes return the credit as a fresh permit. Derived
  release still closes and returns nothing.
- Writer spellings +3, system operations +3 (META-5 delta), outcome and
  target-contract tables updated, derivation ledger rows SYS-10/11/14 and the
  v0.42 candidate paragraph appended.

## 2. Compiler

- Catalog: `reserve_file` returns the `Result`; three `SystemOperation`
  entries for the closes (`TargetAction::MAY_SUSPEND`, result `FilePermit`).
- Emitter: `reserve_file` calls `wf__file_reserve()` and answers the `Result`
  shape (`Ok` with the erased bit, `Err` with a `ResourceExhausted` of origin
  none); each close calls the target's native close and then
  `wf__file_credit_return()`.
- Staged pipelines ([PAR-3]) admit a prologue gate: a `match` on a
  never-suspending operation whose one continuing arm holds the cut and whose
  other arms leave by `return` or `break` reading only carried bindings. An
  exiting arm with nothing in flight leaves at once; otherwise it drains first
  and leaves through a pending-exit edge (`compiler/src/lowering/builder/loops.rs`).
- Qualification review pinned to v0.42; ABI symbols 16–18 for the closes.

## 3. Runtime

- `wf_floor.c` / `wf_floor_windows.c`: one process-wide credit counter,
  initialised from `RLIMIT_NOFILE` less a 64-descriptor runtime reserve
  (ceiling 2^20) on POSIX and a fixed 4096 on Windows; `wf__file_reserve`
  decrements or refuses, `wf__file_credit_return` increments. No heap, no
  wait.
- The descriptor retirement ledger is deleted on every route and target:
  `wf_completion_retirement_*`, `wf_completion_descriptor_returns`,
  `wf_completion_operation_accepted/retired`, the waiter struct and state
  enum, and the announce endpoint (`contract.h`, `runtime.c`);
  `wf_bridge_retire_and_retry_direct` and the direct executor's ledger
  entry (`bridge.c`); `wf_file_retire_and_retry`, the owed-work callback and
  the `may_run_owed_work` plumbing (`file_adapter.c`); the ring's
  `RETRY_HELD` entry state, `wf_linux_resolve_retry_held_locked`, and the
  re-attempt arm of its completion handler (`linux_io_uring.c`); the Windows
  ledger section, the open-order tickets, the resource-attempt lock, the
  three observation points, the took/returned/refused resource plumbing of
  the open worker, and the blocking adapter's retry and gate
  (`windows_completion.c`, `windows_runtime.c`, `windows_blocking.c`,
  `windows_iocp.c`, `windows_bridge.c`). The exported counters
  `wf__completion_open_exhaustion_retries/waits` are gone with them.
- An open that still meets `EMFILE`/`ENFILE` (an external process filling the
  system table, or the reserve exceeded by the runtime's own descriptors)
  publishes `ResourceExhausted` at once.

Slice 3 alone removed 6,539 lines against 120 added across 24 files.

## 4. Evidence retired with the mechanism

These tested the ledger and nothing else, and are deleted rather than kept
green against a mechanism that no longer exists:

- `retirement_interleave_probe.c` and its `completion-retirement-interleave-test`
  target; `windows_resource_attempt_probe.c` and its cross-build block with the
  three `WF_WINDOWS_*_POINT` defines.
- Nineteen harness tests (`harness.c`): the ledger's award order, charge and
  promise accounting, the owed-work read, standing aside, the endpoint wakes,
  the retry on each route, the ring's close-return count, and the three
  bridge schedules that parked a read on one engine while an open was refused
  on the other. The `openat` and `poll` observation gates they armed are
  reduced to plain pass-throughs.
- `windows_blocking_probe.c` is rewritten: it no longer scripts a refused
  open, a retirement waiter, or two opens crossing a ticket. It keeps the
  adapter's own proof — a write parked on a worker while an open and a
  directory job run beside it, every lease cleared before publication, and
  statistics that close — and the cross-build import check no longer demands
  the ticket's condition variable.

## 5. Corpus and conformance

- 26 conformance cases and the Rust fixtures move to the `Result` idiom; new
  case `run-sysfile-close-returns-permit.wf` (a permit returned by
  `close_read` opens again).
- `tests/programs` and `research/experiments`: 29 programs rewritten; the
  ones that fail to compile do so on pre-existing, unrelated baseline issues
  verified against `main`.

## 6. Gates

- `make -C compiler completion-test`: core-read probe, default-route probe,
  harness under `WF_IO_HELPERS=0/1/4` and `WF_IO_NOCACHE=1`, link boundary —
  PASS.
- `make -C compiler completion-windows-cross` (zig, x86-64): PASS (not
  runtime-qualified, as before).
- Linux translation units (`linux_io_uring.c`, `bridge.c`, `runtime.c`,
  `file_adapter.c`, `harness.c`, `writer_scheduler.c`, `native_contract.c`)
  compile under `zig cc -target x86_64-linux-gnu -Werror -Wpedantic`.
- `cargo test --profile gate --lib`: 1491 passed. `make conformance-run`:
  Pass=502, Xfail=1 (the recorded `ent5-neg-callee-uniq-buffer-replace-kills-length`),
  Skip=1. `make snapshot-run`: Pass=491, Flip=0.
- Canonical `make check` stages on the branch: `repository-invariants`,
  `approval-history-integrity`, `spec-append-only`, `spec-digest-sync`,
  `conformance`, `compiler` (`== WHITEFOOT COMPILER GATE GREEN ==`),
  `research-tests`, `conformance-run`, `snapshot-run` all green;
  `spec-candidate-integrity` reports the declared v0.42 candidate over the
  recorded v0.41. The one stage that refuses is `spec-archive-integrity`,
  by design: a CANDIDATE status is valid branch work and not a merge-ready
  ACTIVE identity. It turns green at activation (archive the v0.41 bytes,
  install the ACTIVE v0.42 record), which is the merge step and not branch
  work.

## Approval classes

Specification (v0.42 candidate; the ACTIVE record is written at merge),
conformance evidence (26 modified cases, one added case, manifest), compiler,
runtime, and documentation.
