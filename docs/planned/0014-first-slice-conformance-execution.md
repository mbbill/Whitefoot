# 0014 — First-slice conformance execution

**Planned task.** Decomposed from `docs/current-plan.md` Work item 2 (wave 7
of 8; task 9 of 11; runs concurrently with task 0015). Not yet claimed —
claiming fills in `Status`, `Owner`, workspace, and `Base revision` and
moves this file unchanged in number to `docs/ongoing/` per
`docs/WORKFLOW.md`. This record authorizes nothing beyond Work item 2
itself; if `docs/current-plan.md` is replaced before this task is claimed,
delete this file unless the new plan explicitly retains it.

- **Authority:** `ACTIVE` `docs/current-plan.md` Work item 2, sixth bullet
  ("first-slice conformance execution"), and the plan's Verification bullet
  ("first-slice conformance cases pass through the normal command path").
  Implements dossier §12.2's required test list. Claimable only while
  `docs/current-plan.md` remains `ACTIVE`.

## Goal

Add the dossier §12.2 case catalog to `tests/conformance/cases/*.wf` and
`tests/conformance/manifest.jsonl`; extend the manifest schema and
`runner.py`'s structural validation so a case can describe the runtime
arrangement it needs (a fixture-file list, argv bytes, an optional stdin
body, and a redirection description) and so a target-qualification-failure
case can carry the `unsupported` verdict `runner.py`'s own docstring
already names but `validate_manifest` does not yet accept; and give the
corpus the native execution adapter its own scaffolding already anticipates
(`tests/conformance/runner.py`'s own comment: "A later entrance-gated
integration may install a named Rust adapter") so the "runnable" v0.18
cases actually compile and run through `whitefootc`.

## Direction and invariants

- Cover exactly the subset of §12.2's list expressible as a self-contained
  `tests/conformance` case once the schema extension below exists: entry/
  name visibility; `Args`/`HostString`; the non-symlink path cases;
  `run-sysdir-open-notfound` and `run-sysdir-open-isdirectory` (not
  `open-permissiondenied` — see the exclusion below); the
  empty/short/exact/multichunk file-shape cases (not the changing-file
  case); the basic-write and same-sink-redirect cases; the
  effect-attribution flagship pair and its three companion cases; and the
  exit-code cases.
- **Explicitly out of this task's `tests/conformance` scope:** every
  fault-injection case (`open-permissiondenied`'s privilege dependence,
  close-`EINTR` no-retry behavior, mid-stream `ReadFailed`, a short host
  write, a broken pipe, and an output sink that fails only at close or
  writeback) and every filesystem arrangement beyond a plain fixture file
  (the symlink-policy witness, the changing-file witness). These need
  either task 0013's deterministic test target, real process-spawning or
  piping the single-`.wf`-file corpus model cannot express even with the
  schema extension below, or platform/privilege-conditional setup that
  does not belong in a portable corpus fixture. This is **an OS-level
  integration-test lane outside `tests/conformance`** — do not promise any
  of these eight cases as corpus fixtures. Task 0015 (`wfgrep`) owns the
  broken-pipe, symlink-policy, and changing-file cases; task 0016 owns the
  remaining five (fault-injection needing task 0013). `open-permissiondenied`
  and the symlink case remain possibly re-includable in this corpus once
  the schema below exists, since both are just a chmod/symlink fixture
  away and the first slice targets macOS/Linux only; if a future claimant
  of this task pulls the symlink case back into the corpus, task 0015
  drops it from its own scope in the same change.
- This task must extend `tests/conformance/manifest.jsonl`'s schema —
  today exactly `{id, rules, expect, status, reason?, doc}` — and
  `runner.py`'s `validate_manifest`, adding: a fixture-file list (path
  plus content, sufficient for the file-shape and redirect-target cases);
  an argv byte list (byte-level, not text-restricted, so a non-UTF-8
  argument fixture is expressible); an optional stdin body (added for
  schema completeness; no first-slice operation reads stdin, so no
  in-scope v0.18 case uses this field yet); and a redirection description
  (route stdout/stderr to one named sink). `runner.py`'s role stays
  structural validation of these new fields — never their execution, which
  is the Rust adapter's job, preserving the existing Python/Rust boundary.
- `runner.py`'s own docstring already states the verdict space as
  `("accept",) | ("reject", rule) | ("run", exit) | ("trap",) |
  ("unsupported", why)`, but `validate_manifest`'s `expectation_fields`
  only recognizes `accept`/`reject`/`run`/`trap`. This task must add
  `unsupported` acceptance, since a target-qualification-failure case (for
  example, a test double target lacking the argv-backing guarantee)
  otherwise has nowhere in the schema to go.
- The execution adapter is a Rust integration test, not a Python addition:
  `tests/conformance/runner.py`'s existing Python stays scoped to corpus
  structure and coverage checks per its own docstring, consistent with the
  project's standing rule against re-implementing compiler behavior in
  Python.
- **Shared harness cross-link with task 0015.**
  `compiler/tests/programs/support.rs`'s current `compile_and_run` is a
  bare `Command::new(&executable).output()` with no argv, cwd,
  fixture-file, or redirection support — this task must extend it (or add
  a sibling helper) to consume the same fixture/argv/redirection
  description the manifest schema now carries. Task 0015 needs the same
  extension; whichever of the two lands first should build it generally
  enough for the other to reuse without a second incompatible helper. This
  record and task 0015's record cross-link the dependency; land in one
  order per `docs/WORKFLOW.md`'s semantic-overlap rule and rebase the
  later one onto the extension the earlier one lands.
- The Route C same-spelling collision policy is deterministic rejection
  (neither name resolves), per task 0007 — treat it as settled.

## Method

Design the manifest schema extension first (fixture list, argv, stdin,
redirection, plus the `unsupported` expectation kind) and land the
`validate_manifest` changes for it before porting cases, since every
runtime case below depends on that surface existing. Then port the dossier
§12.2 groups A/B/C(non-symlink)/D(partial)/E(partial)/F(partial)/G/I into
`.wf` case files and manifest entries following the existing
`id`/`rules`/`expect`/`status`/`doc` schema plus the new fields,
substituting the real `spec/kernel-spec-v0.18.md` rule IDs for any
placeholder used during drafting. Build the native adapter as a Cargo
integration test (for example `compiler/tests/conformance.rs`) that reads
`manifest.jsonl`, compiles each "runnable" case via `whitefoot::compile`,
and for a `run`-verdict case executes it with the harness's new
argv/fixture/redirection support and checks the exit code; for
`reject`/`trap`/`accept`/`unsupported` verdicts, checks the compiler or
qualification outcome directly without executing.

## Scope and expected touch set

- `tests/conformance/manifest.jsonl` (schema extension: fixture list,
  argv, stdin, redirection, `unsupported` expectation; then new case
  entries)
- `tests/conformance/runner.py` (`validate_manifest` accepts the new
  optional fields and the `unsupported` expectation kind; its "no active
  adapter" docstring updated once the Rust adapter exists; its
  structure/coverage checks otherwise untouched)
- `tests/conformance/cases/*.wf` (new files, per family mnemonics:
  `sysentry`, `sysname`, `sysarg`, `syshost`, `syspath`, `sysdir`,
  `sysfile`, `sysout`, `sysrelease`, `syseff`, `sysexit`)
- `compiler/tests/programs/support.rs` (argv/cwd/fixture-file/redirection
  extension to `compile_and_run`, consuming the manifest schema's new
  fields — shared with task 0015, see the cross-link above)
- New: `compiler/tests/conformance.rs` (the native adapter)

## Dependencies and integration order

- **Prerequisite (harness lane):** task 0017 owns the manifest/runner schema
  extension, the `unsupported` verdict, the v0.18 corpus pin and coverage
  annotations, and the compile-time case lane; this task consumes them and
  owns runtime execution.

Depends on task 0012 (real native I/O execution is needed for the
run-verdict cases). Cross-links with task 0015 on the shared
`support.rs` harness extension — land whichever lands first; the other
rebases onto it. Runs concurrently with task 0015 (wave 7). Task 0016
depends on this task.

## Validation

`make check` (both the compiler gate and the repository conformance
gate); `runner.py`'s structural checks accept the new
fixture/argv/redirection fields and the `unsupported` expectation kind
without executing anything; every new case's actual verdict matches its
manifest `expect`; the flagship effect-attribution pair
(`accept-sysrelease-return-unit-declared` /
`reject-syseff-return-unit-omitted`) both pass. A claimed task lands only
through lead review per the executor lane in `docs/WORKFLOW.md`.

## Done-when

The §12.2 cases achievable with real OS fixtures are in the corpus,
execute through the normal `whitefootc` path via the new native adapter,
and pass; `make check` green.
