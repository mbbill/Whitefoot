# 0013 — Deterministic test target

Live coordination record. It reports how authorized work is being carried
out; it is not authority and it cannot expand the scope it cites.

- **Status:** `IN PROGRESS` — complete and green; awaiting lead review.
- **Authority:** `ACTIVE` `docs/current-plan.md` Work item 2, fifth bullet
  ("the deterministic test implementation"). Implements dossier §6.10's
  deterministic-test-implementation paragraph.
- **Owner:** executor agent `exec-0013`
- **Workspace:** branch `worktree-agent-af48fca5bfd684c6a`
- **Base revision:** `0a47f54` (`docs: close task 0012; move the 0011
  record; refresh stale labels`) — rebased from `eca0078` once task 0012
  landed, per the integration order below.

## Goal

Register a second qualified "target" entry in task 0011's qualification
table that implements the same first-slice operation set
(`open_read`/`read_once`/`write_once`/releases, and the argument/path
cluster if useful) against an in-process fake host instead of libc, so
hostile conditions that are impractical to trigger deterministically
through real OS files and pipes — a close attempt that fails, a read that
returns a specific short count on demand, a write that fails only at a
chosen call — can be exercised in tests without real OS races.

## Direction and invariants

- Scoped exactly to what the first slice's own contract tests need
  (arguments, files, short reads, partial writes, redirects, and the
  specific failures task 0016 needs) — this is explicitly **not** a
  general simulator or an artifact-replay framework, and must not be
  extended speculatively for operations a later slice might add.
- Must satisfy the same semantic-ID contract (signature, outcomes,
  ownership transitions, effect row) as the real native target: it is a
  different implementation of the same specification, not a relaxed one.
  A test using it must observe the same source-visible behavior a real
  fixture would produce for the same forced condition.
- Test-only infrastructure: must not be reachable from a normal
  `whitefootc` compilation of a real command program, only from a test
  harness that selects it explicitly.

## Method

Add the second qualification-table entry (parallel to the real
macOS/Linux target selection) and its own small emitter/interpreter module
that answers each semantic ID against caller-configured in-memory state (a
fake file's bytes; a script of "return this outcome on call N") rather
than a real fd. Keep the configuration surface minimal — enough for the
task 0016 cases, not a general mock framework.

**Placement choice (executor).** The fake host lives inside the compiled
test artifact. `SystemTarget` gains a host-facility column; the
deterministic target's qualification rows name `wf_test_*` host symbols
where the native rows name libc ones, and the test harness links one
generated fake-host translation unit that answers those symbols from
scripted in-memory state. This keeps one emitted lowering under test —
the same wrapper bodies, the same outcome mapping, the same release
shape — so a forced condition is observed through real compiled code
rather than through a Rust model of it, and it keeps every scripting
surface outside the compiler.

## Progress

- Completed: the second target column. `SystemTarget` carries a
  `HostFacilities` column naming the five facilities that reach a real
  operating-system object — directory open, directory-relative file open,
  read, write, close. `HostFacilities::DeterministicTest` exists only in a
  test build, so no `whitefootc` compilation can select it. The release
  row, the bootstrap's directory open, and task 0012's three I/O rows now
  take their symbol from that column instead of a fixed libc name.
- Completed: native emission is unchanged. A program reaching no host
  object emits byte-identically on both columns; a program that does
  differs by exactly the symbol substitutions, asserted directly.
- Completed: all four fault-injection cases task 0016 consumes, each with
  a control that shows the forced condition is what changed the outcome:
  1. a release close reporting `EINTR` is attempted once and never
     retried, and the discarded diagnostic changes nothing the source
     sees;
  2. a mid-stream `ReadFailed` after a delivering read stops the drain as
     its own outcome, never as the end of input;
  3. a forced short write reports exactly the accepted count as `Ok(n)`,
     with one host attempt and no retry finishing the range;
  4. an output sink that fails only at close is never closed by its
     release, so the failure cannot reach the program [SYS-12].
- Completed: the `DIAG-3` hazard is closed by construction and by test.
  The trap-record writer keeps the native `@write` on both columns while
  `write_once` takes the column's symbol, so a scripted short write can
  never truncate a trap record; one module declares both.
- Completed: rebased onto task 0012 at `0a47f54` and adopted its
  `open_read`/`read_once`/`write_once` row shapes. The three host names it
  had fixed in `emitter/system.rs` now come from the column; its
  `SystemEmission.declarations` set became `BTreeSet<String>` and its
  `@write`/`@abort` dedupe against the trap prologue is preserved.
- Current: awaiting lead review.

## Scope and expected touch set

- `compiler/src/backend/qualification.rs` (second target column: host
  facilities, deterministic-target constructor, per-target operation and
  release rows)
- `compiler/src/backend/emitter/system.rs` (read the host facility symbol
  from the qualification instead of a fixed libc name; native emission
  byte-identical)
- `compiler/src/backend/emitter.rs` (a test-only `emit_llvm_for_target`)
- `compiler/src/backend/tests.rs` (one clang-plumbing path that can link
  one host translation unit)
- `compiler/src/backend/tests/system.rs` and `.../system_io.rs` (one
  visibility word each, so this module reuses their pipeline helper and
  their contract programs rather than copying them)
- New: `compiler/src/backend/tests/deterministic_target.rs` (the fake
  host, its script, the link-and-run harness, and the contract tests)

## Dependencies and integration order

Depends on task 0011 (qualification-table mechanism, `SystemTarget::probe`
and the guarantee-withholding path — both landed at `61936d6`) and task
0010 (the semantic IDs and checked-IR shape it mirrors).

**Cross-link — task 0012 (native I/O lowering), lead-granted overlap.**
Both tasks touch `compiler/src/backend/qualification.rs` and
`compiler/src/backend/emitter/system.rs`, and both are about the same
three `open_read`/`read_once`/`write_once` operation rows: semantic
overlap, not merely textual. The fixed integration order — **0012 lands
first; this task rebases onto it** — was followed: 0012 landed at
`2af4f8b`/`0a47f54` and this branch is rebased onto it, carrying its
commits as ancestors. This task changes no native emission.

Task 0016 depends on this task for four fault-injection cases:
close-`EINTR` (one attempt, never retried), mid-stream `ReadFailed`, a
forced short write, and an output sink that fails only at close or
writeback. All four are available.

The surface task 0016 consumes is
`compiler/src/backend/tests/deterministic_target.rs`:

- `HostScript::new()` with `.file(bytes)`, `.reads(&[..])`,
  `.writes(&[..])`, `.closes(&[..])`;
- `HostOutcome::{Succeed, Accept(n), Fail(HostError)}` and
  `HostError::{Interrupted, DeviceFailure}` — a non-negative entry caps
  the bytes one call may transfer, so `Accept(n)` is a short read or a
  partial write and `Succeed` is no cap;
- `run_on_deterministic_host(source, &script, arguments)`, returning a
  `DeterministicRun` with `output`, `trace()`, and `attempts(facility)`
  for `"open"`, `"openat"`, `"read"`, `"write"`, and `"close"`; and
- `emit_for_deterministic_target(source)` for a codegen-shape inspection
  without running it.

## Validation

Unit tests prove the fake target reproduces each forced condition (an
exact short-read count, a write accepted only in part, a mid-stream read
failure, a close failure) with the same source-visible outcome shape the
real target's contract requires. Two of the four reuse task 0012's own
contract programs unchanged, so the same source is exercised on both
columns.

Gates green by unpiped exit code on the rebased branch:
`make -C compiler check` (lib tests 423 → 427) and `make check`.

## Stop condition

The dossier §6.10 proportionality clause is binding: if closing a case
requires a general simulator, artifact replay, or a mock framework beyond
the scripted state these contract tests consume, stop and report rather
than build it. A blocker or plan defect stops the task with an honest
report per `docs/WORKFLOW.md`'s executor lane.

## Closure

Lead review lands the change; the record moves to `docs/done/` with its
number unchanged in the integration change.
