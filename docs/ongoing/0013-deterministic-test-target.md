# 0013 — Deterministic test target

Live coordination record. It reports how authorized work is being carried
out; it is not authority and it cannot expand the scope it cites.

- **Status:** `WAITING` — the target column and fault-injection case 1 of 4
  are complete and green; cases 2–4 wait on task 0012's landed operation
  rows (see Dependencies).
- **Authority:** `ACTIVE` `docs/current-plan.md` Work item 2, fifth bullet
  ("the deterministic test implementation"). Implements dossier §6.10's
  deterministic-test-implementation paragraph.
- **Owner:** executor agent `exec-0013`
- **Workspace:** branch `worktree-agent-af48fca5bfd684c6a`
- **Base revision:** `eca0078` (`docs: close task 0011 and complete the
  v0.19 corpus pin`)

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

- Completed (`979f2c5`): the second target column. `SystemTarget` carries
  a `HostFacilities` column; `HostFacilities::DeterministicTest` exists
  only in a test build, so no `whitefootc` compilation can select it. The
  `DirectoryRead`/`ReadFile` release row and the bootstrap's
  directory-open facility now come from that column instead of a fixed
  libc name, and native emission is byte-identical (a test asserts the
  two modules differ by exactly the two symbol substitutions).
  `compiler/src/backend/tests/deterministic_target.rs` holds the scripted
  host, the link-and-run harness, and the first fault-injection case.
- Completed: fault-injection case 1 of 4 — a release close that reports
  `EINTR` is attempted exactly once and never retried, with a success
  control, plus evidence that a program reaching no host object emits
  byte-identically on both columns.
- Current: **waiting on task 0012.** The remaining three cases (mid-stream
  `ReadFailed`, forced short write, close/writeback-only failure) all run
  through `read_once`/`write_once`, whose rows are `NotImplemented` on
  both target columns and stop as `UnsupportedSystemInterface`. Writing
  that lowering here would duplicate task 0012's written scope, create a
  second lowering path for the same semantic IDs, and pre-empt the
  operation-row shapes this task was told to adopt — so this task stops at
  the boundary rather than working around it.
- Next, once task 0012 lands: rebase; extend `HostFacilities` with its
  directory-relative-open, read, and write facilities (the trap writer's
  own `@write` stays native on both columns, so a forced short write can
  never truncate a `DIAG-3` record); add the matching `wf_test_openat`/
  `wf_test_read`/`wf_test_write` functions and their scripts; land the
  three remaining cases. The work is mechanical — one accessor and one
  scripted facility per operation.

## Scope and expected touch set

- `compiler/src/backend/qualification.rs` (second target column: host
  facilities, deterministic-target constructor, per-target operation and
  release rows)
- `compiler/src/backend/emitter/system.rs` (read the host facility symbol
  from the qualification instead of a fixed libc name; native emission
  byte-identical)
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
overlap, not merely textual. Integration order is fixed: **0012 lands
first; this task rebases onto it before landing** and adopts 0012's final
operation-row shapes for those three operations, adding only the
deterministic host-facility column beside them. This task changes no
native emission.

Task 0016 depends on this task for four fault-injection cases:
close-`EINTR` (one attempt, never retried) — **available now**; and
mid-stream `ReadFailed`, a forced short write, and an output sink that
fails only at close or writeback — **not yet available**, each gated on
task 0012's `read_once`/`write_once` rows.

The surface task 0016 consumes is
`compiler/src/backend/tests/deterministic_target.rs`:
`HostScript::new().closes(&[HostOutcome::Fail(HostError::Interrupted)])`,
`run_on_deterministic_host(source, &script, arguments)`, and the returned
`DeterministicRun`'s `output`, `trace()`, and `attempts(facility)`.
`emit_for_deterministic_target(source)` returns the module for a
codegen-shape inspection without running it.

## Validation

`make -C compiler check`; unit tests proving the fake target reproduces
each forced condition (an exact short-read count, a write failure at a
chosen call, a close failure) with the same source-visible outcome shape
the real target's contract requires.

## Stop condition

The dossier §6.10 proportionality clause is binding: if closing a case
requires a general simulator, artifact replay, or a mock framework beyond
the scripted state these contract tests consume, stop and report rather
than build it. A blocker or plan defect stops the task with an honest
report per `docs/WORKFLOW.md`'s executor lane.

## Closure

Lead review lands the change; the record moves to `docs/done/` with its
number unchanged in the integration change.
