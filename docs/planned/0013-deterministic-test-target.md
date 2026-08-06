# 0013 — Deterministic test target

**Planned task.** Decomposed from `docs/current-plan.md` Work item 2 (wave 6
of 8; task 8 of 11; runs concurrently with task 0012). Not yet claimed —
claiming fills in `Status`, `Owner`, workspace, and `Base revision` and
moves this file unchanged in number to `docs/ongoing/` per
`docs/WORKFLOW.md`. This record authorizes nothing beyond Work item 2
itself; if `docs/current-plan.md` is replaced before this task is claimed,
delete this file unless the new plan explicitly retains it.

- **Authority:** `ACTIVE` `docs/current-plan.md` Work item 2, fifth bullet
  ("the deterministic test implementation"). Implements dossier §6.10's
  deterministic-test-implementation paragraph. Claimable only while
  `docs/current-plan.md` remains `ACTIVE`.

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
macOS/Linux target selection in `compiler/src/backend/target.rs`) and its
own small emitter/interpreter module that answers each semantic ID against
caller-configured in-memory state (a fake file's bytes; a script of
"return this outcome on call N") rather than a real fd. Keep the
configuration surface minimal — enough for the task 0016 cases, not a
general mock framework. Whether the fake host lives inside the compiled
test artifact or purely in the test process is an open engineering choice
for the executor; either shape must satisfy the semantic-ID contract
above.

## Scope and expected touch set

- `compiler/src/backend/target.rs` (second target registration)
- New: a small emitter/interpreter module under `compiler/src/backend/`,
  or a `compiler/tests/`-side harness component, per the executor's
  placement choice above
- New tests exercising the fake target directly: forced short read,
  forced write failure, forced close failure.

## Dependencies and integration order

Depends on task 0011 (qualification-table mechanism) and task 0010 (the
semantic IDs and checked-IR shape it must mirror). Does not depend on task
0012's actual native code — it is an independent sibling implementation of
the same IDs, so it runs concurrently with task 0012 (wave 6) once task
0011 lands. Task 0016 depends on this task.

## Validation

`make -C compiler check`; unit tests proving the fake target reproduces
each forced condition (an exact short-read count, a write failure at a
chosen call, a close failure) with the same source-visible outcome shape
the real target's contract requires. A claimed task lands only through
lead review per the executor lane in `docs/WORKFLOW.md`.

## Done-when

The deterministic test target is registered and exercises the specific
hostile conditions task 0016 needs; `make -C compiler check` green.
