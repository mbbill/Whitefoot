# 0014 — First-slice conformance execution

Live coordination record. It reports how authorized work is being carried
out; it is not authority and expands nothing.

- **Status:** `BLOCKED` — the runtime case lane and the native adapter are
  built and green; the corpus-wide gate lane is blocked by 123 pre-existing
  runnable cases whose disposition is outside this task. See
  [Owner-level finding](#owner-level-finding-the-corpus-wide-gate-lane).
- **Authority:** `ACTIVE` `docs/current-plan.md` Work item 2, sixth bullet
  ("first-slice conformance execution"), and the plan's Verification bullet
  ("first-slice conformance cases pass through the normal command path").
  Implements dossier §12.2's required test list. Claimable only while
  `docs/current-plan.md` remains `ACTIVE`.
- **Owner:** executor agent `exec-0014`
- **Base revision:** `6336311` (claimed at `0a47f54`; rebased after 0013
  and 0015 landed, gates rerun)
- **Workspace:** worktree `agent-a1ffd6a2ad751eb60`, branch
  `worktree-agent-a1ffd6a2ad751eb60`

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

## Progress

- Completed: claim; base refreshed to `0a47f54`; the native execution
  adapter (`compiler/tests/conformance.rs` plus `conformance/{adapter,
  corpus,json}.rs`) — it reads the same `manifest.jsonl` bytes `runner.py`
  reads, compiles each case through `whitefoot::compile`, realizes an
  `arrange` as a real invocation (fixtures, exact argument bytes through the
  raw route, an empty or supplied standard input, one-or-two-sink
  redirection), and reduces the outcome to one corpus verdict with the same
  match rule and the same `runnable`/`pending`/`xfail` axis.
- Completed: 22 additive runtime cases, all reaching their declared verdict
  through that adapter — argument count and index, the non-UTF-8 argument
  round trip and its text-route refusal, the three recoverable copy
  refusals with sentinel no-write witnesses, the three range traps, three
  path-construction outcomes, one open error class, the four file shapes,
  and the two output cases. Corpus rule coverage by case rose 90 → 100 of
  119; `runner.py` structural validation and its 18 self-tests pass, and the
  manifest diff is +22/−0 with no existing byte touched.
- Completed: the `arrange.argv` reconciliation (below) and the
  `make conformance-run` wiring onto the native adapter.
- Blocked: the corpus-wide gate lane. See the finding below. No status was
  flipped, no case excluded, no expectation weakened.

## Reconciliation: `arrange.argv` is the complete native vector

Task 0017's schema documented `arrange.argv` as "arguments after the program
name"; task 0011 ruled that `command.args` carries the complete native
vector including position 0 for [HOST-1] losslessness. Reconciled in the
schema's favour of losslessness: **`arrange.argv` is the complete native
argument vector, position 0 included**, so `argv[i]` is what the program
reads at position i and the vector's length is the count it reads. The
alternative would leave the vector's first element — an element a program
can count and read — unstated by the case that claims to fix its
invocation, and would make position 0 the harness's incidental choice of
build path rather than the case's own datum. The adapter therefore sets
position 0 explicitly. Recorded in `tests/conformance/runner.py`'s schema
comment; no existing manifest line carried an `arrange`, so nothing was
reinterpreted.

## Owner-level finding: the corpus-wide gate lane

Running the whole corpus through the adapter for the first time gives
`Pass=242 Fail=123 Skip=14`. All 22 cases this task adds pass; every one of
the 123 failures is pre-existing and falls into exactly four causes, none
of which this task is authorized to resolve:

- **A — 45 cases: a rejection carries no rule id.** The compiler rejects
  correctly, but `CompilationFailure::rule_id` is populated only for
  semantic stops, so a rejection at Resolution (24), Parsing (17), Lexing
  (3), or CanonicalSource (1) reaches the adapter as `reject` with no rule
  and cannot be compared to the case's declared rule. This is compiler
  diagnostic plumbing, outside this task's touch set.
- **B — 41 cases: the case unit declares no entry.** All 41 verified to
  contain no `fn main`; [FN-7]'s whole-unit judgment fires and the
  diagnostic cites FN-7, masking each case's own declared rule (and, for two
  `accept` cases, contradicting the expectation outright). Either the
  [DIAG-1] ordering is wrong or 41 protected case sources are incomplete
  units — both are decisions above an executor. Task 0017 flagged one
  instance (`own13-pos-borrow-match-live`) as latent; it is 41.
- **C — 35 cases: `runnable` overclaims.** The compiler stops as
  `SemanticUnsupported { feature: RegionsAndBorrows }`. This is exactly what
  the corpus's own `pending` axis is for, and per-case run evidence now
  exists, but flipping only this bucket leaves the lane red and splits one
  ruling across two changes.
- **D — 2 cases: genuine verdict divergence.**
  `gram5-pos-recursive-place-projection` expects `run 0` and is rejected
  citing TYPE-5; `type7-neg-propagate-box-holder` expects TYPE-7 and is
  rejected citing ERR-3.

Because B and D cannot be dispositioned by an executor, no move available
here makes the lane green. The corpus-wide test is therefore `#[ignore]`d
with the blocker written into its `#[ignore]` reason, and
`make conformance-run` drives it and reports the complete tally
(exit 2 today). `make check` stays green and unchanged in what it claims:
it exercises corpus structure and declared coverage, not verdicts.

## Not drafted, with reasons

- `run-sysdir-open-isdirectory` — not expressible as catalogued. Measured:
  `open_read` on a directory returns `Ok` on this target, because a
  read-only open of a directory succeeds natively; [SYS-7] fixes the class
  but the specification does not fix which operation surfaces `EISDIR`, so
  pinning either surface point would pin unfixed target behaviour. An
  `ENOTDIR` case (`plain.txt/inner`) is available and does map to a distinct
  class, but substituting it is a scope change for the lead, not the
  executor.
- `run-syspath-nontext-bytes-preserved` — drafted, then withdrawn: APFS on
  the development host refuses a non-UTF-8 filename with `EILSEQ`, so the
  fixture cannot be created and the case would be host-conditional. HOST-1
  losslessness is still witnessed by the argument round trip.
- `run-syspath-nul-rejected` — not expressible at all. [HOST-1] fixes the
  Unix code-unit family as `0x01..0xff`, so no argument fixture can carry a
  NUL and no first-slice operation constructs a host string otherwise.
- QUAL-level `unsupported` cases — still not expressible after 0013 landed.
  The native target qualifies for every semantic ID, and 0013's second
  qualified column is `HostFacilities::DeterministicTest` under `#[cfg(test)]`,
  reachable from crate-internal tests but not from the `whitefoot::compile`
  path the corpus drives. A corpus case cannot therefore select a target that
  withholds a guarantee, so the `unsupported` expectation 0017 added has no
  case yet.
- Standard-input absence — realized, not observable. The adapter supplies an
  empty standard input whenever `arrange.stdin` is absent, but [SYS-2]
  declares no operation that reads standard input, so no first-slice case
  can witness it.

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

**Cross-link outcome (0015 landed first; rebased onto `6336311`).** No
conflict and no shared edit: 0015 added `build_program`/`CompiledProgram`
to `compiler/tests/programs/support.rs` for its Rust-literal `wfgrep`
tests, and this task left `support.rs` untouched because arrangement
realization here is driven by the manifest's typed `Arrangement`, which is
where the corpus states an invocation. **One difference matters if the two
are ever merged:** `CompiledProgram::run` takes arguments as argv[1..],
while the corpus schema now fixes `arrange.argv` as the complete vector
including position 0 (see the reconciliation above). Unifying them means
picking one convention, which is a lead decision.

## Validation

`make check` (both the compiler gate and the repository conformance
gate); `runner.py`'s structural checks accept the new
fixture/argv/redirection fields and the `unsupported` expectation kind
without executing anything; every new case's actual verdict matches its
manifest `expect`; the flagship effect-attribution pair
(`accept-sysrelease-return-unit-declared` /
`reject-syseff-return-unit-omitted`) both pass. A claimed task lands only
through lead review per the executor lane in `docs/WORKFLOW.md`.

**Performed.** `make check` green by unpiped exit code (repository
invariants, spec append-only, `runner.py` 18 self-tests, coverage 119/119
with 100 by case, and the complete compiler gate: fmt, clippy
`-D warnings` over `--all-targets`, tests, docs, spec identity).
`make conformance-run` runs the adapter over the complete corpus and
reports `Pass=242 Fail=123 Skip=14`, exit 2 — 22/22 new cases pass, and
every failure is one of the four pre-existing causes above. The flagship
effect pair is `accept-sysrelease-return-unit-declared` /
`reject-syseff-return-unit-pure` (the corpus's landed spelling of the
omission direction); both pass through the adapter.

## Done-when

The §12.2 cases achievable with real OS fixtures are in the corpus,
execute through the normal `whitefootc` path via the new native adapter,
and pass; `make check` green.
