# 0015 — Sequential wfgrep program

**Planned task.** Decomposed from `docs/current-plan.md` Work item 2 (wave 7
of 8; task 10 of 11; runs concurrently with task 0014). Not yet claimed —
claiming fills in `Status`, `Owner`, workspace, and `Base revision` and
moves this file unchanged in number to `docs/ongoing/` per
`docs/WORKFLOW.md`. This record authorizes nothing beyond Work item 2
itself; if `docs/current-plan.md` is replaced before this task is claimed,
delete this file unless the new plan explicitly retains it.

- **Authority:** `ACTIVE` `docs/current-plan.md` Work item 2, seventh
  bullet ("the sequential `wfgrep` program"), and the plan's Done-when
  bullet ("the compiler compiles and runs the sequential `wfgrep` slice on
  macOS/Linux through the normal path, passing its correctness oracle").
  Implements dossier §10.1's witness trace. Claimable only while
  `docs/current-plan.md` remains `ACTIVE`.

## Goal

Implement the frozen sequential `wfgrep` command as a repository-owned
program witness (following the existing `compiler/tests/programs/*`
convention used for the other program families), exactly matching §10.1's
six-step trace: request `Args`/`cwd`/`stdout`/`stderr`; copy the pattern
via `host_copy_bytes` with no Unicode restriction; open the target path via
`relative_path` + `open_read`; reuse one initialized buffer across
`read_once` calls, matching only the returned prefix; append matches to
one reusable output batch flushed via `write_once`; return `ExitStatus` 0,
1, or 2. Give it a correctness oracle over fixture files spanning the
required shapes (empty/short/exact/multichunk).

**Three OS-mechanism cases owned by this task.** Per the lead's placement
ruling, this task also owns the three §12.2 cases that need real
process-spawning or filesystem arrangements beyond a static fixture — a
broken pipe, the symlink-policy witness, and the changing-file witness —
since they exercise the real compiled program against real, portable,
deterministic OS mechanisms, which is exactly what this task's end-to-end
harness already does. These are split off from task 0014's `tests/conformance`
corpus scope (which cannot express them even with its schema extension)
and from task 0016's fake-target scope (which is for genuine fault
injection, not real OS mechanisms).

## Direction and invariants

- No whole-file allocation, no per-byte call, no Unicode conversion of the
  pattern or path, no raw fd, no per-argument allocation, no per-match
  `write_once` call (batched output only).
- `ExitStatus` mapping: 0 = match found, 1 = no match, 2 = error, matching
  the dossier's fixed mapping wfgrep depends on and cross-checked against
  task 0014's `run-sysexit-code-*` cases.
- The oracle must be a genuine correctness check (byte-exact matched-line
  output compared against a trusted reference), not merely "runs to some
  exit code."
- **Broken pipe:** spawn the compiled wfgrep with stdout piped to a reader
  the test drops immediately (without reading), and assert `write_once`
  reports `IoError::BrokenPipe` rather than the process dying to the
  default `SIGPIPE` disposition — this is the direct behavioral check of
  the command bootstrap's one-time SIGPIPE-to-ignored normalization (task
  0011).
- **Symlink-policy witness:** place a real symlink inside the fixture
  directory pointing outside it, and confirm `open_read` follows it and
  reads the linked file, per the first slice's process-equivalent (not
  confined) `command.cwd` semantics.
- **Changing-file witness:** run wfgrep against a fixture, mutate the
  fixture's content between two separate invocations (or between two
  `read_once`-driven passes if the program structure allows it in one
  run), and confirm no invariant beyond "each `read_once` reports exactly
  what was present for that attempt" is assumed — this is a real, if
  approximate, analog of a size-changing file; do not attempt to interleave
  a mid-read mutation with the running process, which the dossier never
  requires.
- `open-permissiondenied` and the symlink case are flagged elsewhere (task
  0014's Direction and invariants) as possibly re-includable in task
  0014's `tests/conformance` corpus once its schema extension exists. If a
  future claimant of task 0014 does that, this task drops the
  symlink-policy witness from its own scope in the same change rather than
  duplicating it.

## Method

Write the wfgrep source as a new file under `compiler/tests/programs/`
(placement follows the existing flat-file convention there unless the
command-entry-using program family warrants its own subdirectory — an
executor placement choice, not a design decision, and does not block
planning). Extend `compiler/tests/programs/support.rs` with the same
argv/cwd/fixture-file harness surface task 0014 needs, reusing it if task
0014 lands first, and further extend it with process-spawning support
(piped stdio with a reader the test controls, for the broken-pipe case)
and a real-symlink fixture helper (for the symlink-policy witness) if
`support.rs` does not already have them from task 0014's extension.

## Scope and expected touch set

- A new wfgrep source file under `compiler/tests/programs/`
- `compiler/tests/programs/support.rs` (shared harness extension,
  cross-linked with task 0014; plus process-spawning/piped-stdio and
  real-symlink-fixture support for this task's three OS-level cases)
- New fixture files (empty/short/exact/multichunk content, plus a
  changing fixture and a fixture directory containing an outward-pointing
  symlink) and new integration tests: the oracle comparison, the
  broken-pipe case, the symlink-policy case, and the changing-file case.

## Dependencies and integration order

Depends on task 0012 (real native I/O is needed to run a real command)
and, for the broken-pipe case specifically, on task 0011's SIGPIPE
bootstrap normalization. Cross-links with task 0014 on the shared harness
extension — land whichever lands first; the other rebases onto it. Runs
concurrently with task 0014 (wave 7). Task 0016 depends on this task.

## Validation

`make -C compiler check`; the oracle comparison passes for every required
fixture shape; the broken-pipe test observes `BrokenPipe` and a clean
`ExitStatus`, never process termination by signal; the symlink test reads
the linked file's content; the changing-file test asserts only the
per-attempt `read_once` contract, not a stronger invariant; a smoke run on
both a macOS and a Linux target (or CI equivalent), per the current plan's
macOS/Linux scope. A claimed task lands only through lead review per the
executor lane in `docs/WORKFLOW.md`.

## Done-when

wfgrep compiles and runs through the normal path on macOS/Linux, passes
its correctness oracle across the required file shapes plus the
broken-pipe, symlink-policy, and changing-file cases, and (per task
0016's inspection, not this task's own verification) exhibits none of the
forbidden per-byte/per-match/allocation patterns; `make -C compiler check`
green.
