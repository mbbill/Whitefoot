# Batch 0082 — world-region I/O and completion backends

Branch: `codex/io-model-completion`, from `main` at
`eab81a335addfb0ae060735771d4e98891dec2ea`.

Status: COMPLETE ON THE WORK BRANCH. Phase A started 2026-08-25; v0.37 was
activated as ordinary work-branch work on 2026-08-25, and Phases A through C
closed on 2026-08-25.

## Charter

The owner directed this branch to execute the now-retired handoff formerly at
`research/investigations/io-model/HANDOFF.md` from Phase A through the final
phase. Its completion boundary included the specification migration, compiler
and conformance migration, macOS prototype and measurement, production macOS,
Linux, and Windows completion backends, cross-host harness, CI coverage, the
deterministic `--io-ledger` audit surface, scripted schedule evidence, and
canonical repository verification. Every item is complete.

No revision from this branch may enter `main` without the owner's approval of
the exact revision. This record does not authorize such a merge.

## Binding material read before implementation

The implementation pass read these sources in the order fixed by the handoff:

1. `AGENTS.md`;
2. `docs/constitution.md`, including W3 and theorem T3;
3. `research/investigations/io-model/DESIGN.md`, revision 2;
4. all three reports in `research/investigations/io-model/reviews/`;
5. `docs/current-plan.md` W2 and the active v0.36 rules [FN-7], [EFF-1..5],
   [PAR-1], [PAR-2], [TRAP-1], and [SYS-1..14];
6. `docs/done/0081-loan-column.md`.

T3 controls the yield direction: a correct program never loses permission in
order to stabilize a defective execution's observables. The world-window rule
must widen the erroneous-execution promise and must not restore the overruled
trap-free gate.

## Fixed implementation boundaries

- Phase A uses DESIGN section 3e option 1: every operation formerly carrying
  `external` joins one conservative global world-order domain, preserving the
  v0.36 order promise.
- Different capability values never prove world disjointness. Missing origin,
  alias, projection, or target evidence denies overlap.
- `blocks` becomes trusted completion/blocking metadata for every target
  action, including compiler-derived release and transitive user wrappers.
- The language exposes completion semantics only. kqueue readiness, io_uring,
  and IOCP remain backend and trusted-base choices.
- Phase C covers macOS, Linux, and Windows against one shared C contract.
- No active source, test, tool, or build path may depend on `archive/`.
- Phase B measurements are evidence rather than a gate. Favorable or adverse
  results are recorded honestly and implementation proceeds directly to Phase
  C; adverse evidence is highlighted in the final merge packet.
- Before delivering any document that directs another agent, mechanically
  scan it for `approval`, `owner`, `批准`, `present`, `wait`, and `decide`.
  Every match must have a stated non-pausing reason, followed by a literal
  reader pass asking where the most literal implementer could stop.

## Flagged decisions

The owner approved the recommended selections D1 through D5 on 2026-08-25
and directed that future DESIGN-section flagged decisions adopt their
recommended selection on the work branch. They take effect in `main` only
through the owner's eventual approval of the final exact revision.

- **D1, conservative-first ordering:** every former-`external` action writes
  the command-wide world-order region.
- **D2, erroneous execution:** a selected schedule may choose the false claim
  and the pre-abort world-effect prefix; T3 forbids a trap-free permission
  gate.
- **D3, worker mapping:** `WF_WORKERS=0` is sequential with no compute pool,
  `WF_WORKERS=1` is overlapped with one compute lane and no stealing worker,
  and larger values name the compute-lane count. The flagged observables are
  false-claim selection, pre-abort output, and the 48 B/level overlapped stack
  record versus the 16 B/level sequential record.
- **D4, retired words:** bare `external` and `blocks` remain reserved after
  their effect alternatives are removed.
- **D5, provenance vocabulary:** the independent PRV class is renamed
  `boundary-derived` in this batch without changing verdicts.

## Phase A evidence

### Parent behavior reproduced

The branch was created from the byte-exact v0.36 `main` revision
`eab81a335addfb0ae060735771d4e98891dec2ea`; every parent reproduction below
uses that revision. Commit `13a93bdf` adds only the candidate and batch record.
Commit `fee33565` then performs the initial work-branch v0.37 activation and
archives the outgoing v0.36 bytes while also removing the last handoff pause.
That commit is the compiler/conformance comparison origin because it changes
neither compiler nor corpus bytes, but its active specification is already the
initial v0.37 text. The implementation later completes and rehashes those
active specification bytes together with the compiler and corpus migration.

- Case-insensitive word-boundary counts in `spec/kernel-spec.md` reproduce the
  review exactly: 136 `external` occurrences, 31 `blocks` occurrences, on 117
  physical lines.
- The same search under `tests/conformance/cases/` names exactly 42 `.wf`
  files.
- `cargo test --locked --offline system_effects`: 10 passed, 0 failed.
- `cargo test --locked --offline semantic::tests::permission`: 28 passed,
  0 failed. This pins both adjacent boundaries: an `external` row is denied by
  the current row gate, while a claim-bearing closure remains eligible under
  T3.
- `cargo test --test conformance --locked --offline -- --ignored --nocapture`:
  `Pass=500  Skip=1`, including the same-sink EFF-5 runtime witness and the
  seven release/effect-row verdict records.

### Candidate and ledgers

The now-retired `research/investigations/io-model/SPEC-CANDIDATE.md` contained
the complete non-authoritative delta against v0.36 under the recommended
selections. Its research state is preserved by commit `13a93bdf`; its selected
content is carried by the active specification and the ledgers below. It
included:

- D1 through D5 with consequences, including all three observable
  `WF_WORKERS=1` mapping deltas;
- an explicit disposition for all 163 distinct `K` anchors cited by the
  specification sweep and all sixteen final amendments;
- all fifteen kinded operation signatures, all eight release rows, exact
  declaration counts, origin/alias rules, operation points, outcomes,
  progress, abort, qualification, and target-action rules;
- a one-to-one ledger for the 42 source cases and the exact seven
  verdict-sensitive manifest records, with no proposed verdict change, plus
  an explicit inventory of the 27 files that actually write an `external`
  effect row; and
- the nineteen additional tracked `.wf` workloads outside conformance that
  use world-bearing system types or operations, plus the one deliberate
  `open_read` name-collision control that receives no syntax rewrite;
- compiler work sizing that isolates capability world-vector representation
  and EFF-2 world projection as the two substantial pieces, followed by their
  syntax, catalog, release, permission, entry, provenance, lowering, and
  activation satellites.

The draft remained research material through commit `13a93bdf`; D1 through
D5 were then selected. Activation was ordinary work-branch work. Commits
`b2a5d409` and `fee33565` make the continuous sequence explicit: adverse data
and flagged decisions are recorded while work continues, and the only
repository gate is the final exact merge revision.

### Phase A acceptance audit

Mechanical checks over the completed draft establish:

- sweep coverage is 163 candidate anchors for 163 review anchors, with no
  missing or extra ID;
- the conformance table is 42 candidate IDs for the 42 case files found by the
  parent atom search, with no missing or extra case;
- stripping string literals identifies exactly the separately enumerated 27
  source files that really write an `external` row;
- the broader world-family search identifies exactly the separately
  enumerated nineteen `.wf` workloads outside conformance;
- parsing the candidate operation block yields fifteen operations, fifty-four
  operation region parameters, and thirty-eight value parameters; with
  sixteen nominals, fourteen nominal world parameters, forty-two constructors,
  and sixty-seven fields, the declared preorder total is 246; and
- `make repository-invariants` passes, and both new documents contain no
  trailing whitespace, personal home path, TODO, TBD, or FIXME marker.

The flagged selections are complete. Activation, implementation, and
verification proceed continuously on this branch.

### Post-implementation semantic audit

The completed implementation audit found and closed three specification-facing
gaps rather than treating the first green implementation as final:

- `reads` and `writes` now consume an already established kind without
  inventing one. A complete resolved-unit prepass collects direct anchors
  before propagating user-call edges, reports the first conflicting occurrence
  in canonical source order, rejects unanchored cycles, and preserves FN-2,
  SYS-2, FN-7, and EFF-1 ownership for their wrong-kind occurrences.
- Unequal world-region declarations no longer act as a disjointness proof.
  World/world read, write, and release footprints conservatively conflict in
  the absence of the TCB minting or checked generativity fact v0.37 does not
  provide.
- The first current-corpus rerun caught a diagnostic-order regression in
  `x-eff-dup-reads-effect`: the prepass reported a later FN-2 call-shape error
  before the declaration's duplicate EFF-1 row. Effect-row formation now
  remains on its one ordinary parser path in the prepass timeline, and a unit
  regression pins EFF-1 as the earlier result.

The added normative sentence changes the final active v0.37 bytes to SHA-256
`6ace763ae2c2d20127f9218ed93ef8865312f68e62d40a23dbc4757d40160c6b`.
The generated identity, activation chain, compiler literal, and all six prose
anchors name those exact bytes; the outgoing v0.36 archive remains unchanged.

The old/new verdict comparison is keyed by stable case ID and never feeds new
syntax to the old compiler:

- `runner.py verdicts fee335654d9dea027f4636bbad448d57a4e84d08`
  reports 0 moved, 0 removed, and 0 added among 501 declarations;
- the detached `fee33565` compiler with its own v0.36 corpus reaches
  `Pass=500  Skip=1`;
- the source diff contains the 42 ledgered cases plus the two D5-only
  `prv3-neg-external-claim*` corrections, while all seven verdict-sensitive
  manifest rows retain their expectation and rule; and
- after the diagnostic-order repair, the current compiler with the migrated
  corpus also reaches `Pass=500  Skip=1`, including the same-sink runtime case
  whose declared observable is the exact `AABB` byte sequence.

Because both independent endpoint runs reach every nonpending declaration and
the declaration map is unchanged, their actual verdicts equal the same
case-ID-keyed map. The comparison therefore establishes no verdict drift
without depending on source compatibility across the language versions.

## Phase B evidence

### Prototype and controlled workload

The prototype became the production shared contract rather than a separate
source-shaped path. `compiler/src/backend/completion/` contains caller-owned
generation-tagged frames, preallocated intrusive MPSC nodes, release/acquire
publication, a fixed four-thread blocking disk pool, executing-lane mailbox
affinity, progress-then-rescan and announce-then-recheck parking, bounded
helping, submission rollback, and loan states that remain in flight through
terminal completion. Native `openat`, `read`, `write`, `fstat`, `close`, and
macOS directory-enumeration adapters use that contract; a nested adapter on a
disk worker performs its one native attempt directly rather than deadlocking
the fixed pool.

The controlled directory corpus contains 5,461 directories and 4,096 empty
leaf files. Every run starts inside that tree, while its `out` entry is a
symbolic link to a directory outside the traversed tree. The measured parent
v0.36 compiler is built from `fee33565`; the current compiler is built from
this worktree. An independent release rebuild at the true v0.36 `main` origin
`eab81a33` emits byte-identical directory-walk LLVM to the `fee33565` compiler:
SHA-256 `643fe3d632fc1019398ca8a40c83f1db517aa90cad65844656579f0d5a3c5284`
for the sequential module and
`b9971c5a01a391176cd12c96e6d366e3dc0d6cfb3e54e54c360a0b26f75fcf40`
for `--par`. The timing baseline therefore names the same executable program
as the byte-exact main origin despite the earlier branch-only specification
activation. Sequential and `--par` executables are measured with the existing
native `research/investigations/proof-derived-parallelism/bench/timeit.zsh`
interleaved min/max timer for nine rounds per cell.

An earlier trial started from the benchmark parent, so the timer's output
files were themselves discovered by later traversals; changing output sizes
and hashes exposed the mistake. That trial is excluded in full. The corrected
experiment has 72/72 successful runs and exactly one full output SHA-256:
`583d8775eda989cf1d4159f046e2274864bdd662e24398fcdfbc98d4030fae50`.

| Executable | Best | Worst | Spread |
|---|---:|---:|---:|
| v0.36 sequential | 0.9196 s | 1.0239 s | 11.3% |
| v0.36 `--par`, W0 | 0.9492 s | 0.9996 s | 5.3% |
| v0.36 `--par`, W1 | 0.9488 s | 1.0181 s | 7.3% |
| v0.36 `--par`, W4 | 0.9304 s | 0.9973 s | 7.2% |
| v0.37 sequential | 1.1955 s | 1.6186 s | 35.4% |
| v0.37 `--par`, W0 | 1.1787 s | 1.4670 s | 24.5% |
| v0.37 `--par`, W1 | 1.1863 s | 1.2377 s | 4.3% |
| v0.37 `--par`, W4 | 1.1911 s | 1.3328 s | 11.9% |

The result is adverse and material. At the best observed times, v0.37 is
30.0% slower sequentially and 24.2%, 25.0%, and 28.0% slower at W0, W1, and
W4 respectively than the corresponding v0.36 executable. Within v0.37, W1 is
0.6% slower and W4 is 1.1% slower than W0. The current `--par-ledger` and
`--io-ledger` explain the shape: every world-bearing system/wrapper call in
`dir_walk.wf` has `lowering=sequential`; only unrelated pure stored-byte pairs
are permitted. This workload therefore exposes no I/O overlap and pays only
the adapter/service cost. The old caveated 2.83x observation is not
reproduced and must not be used as a current performance claim.

### Completion-service overhead and sanitizer evidence

The harness measures 128 generation-reuse completion round trips against 128
condition-variable round trips. Eleven final-code repetitions give paired
ratios rather than a ratio selected from separate best values:

| Host | Paired median | Range | Median completion | Median condvar |
|---|---:|---:|---:|---:|
| macOS kqueue | 1.917x | 1.079x–2.592x | 5,648 ns | 2,960 ns |
| Linux io_uring | 0.659x | 0.109x–0.895x | 13,367 ns | 25,438 ns |

The macOS readiness-served path is slower and noisy; the Linux completion
park path is faster in this microbenchmark. Every mixed-load run makes compute
progress and observes completion before draining the ten-million-step backlog.

ThreadSanitizer found two defects during the adversarial pass: the kqueue
waiter read a non-atomic lane-initialized flag, and a completion thread reread
`owner_lane` after publishing a reusable frame. The repair makes initialization
atomic and captures the owner before the release publication; the shared
state-machine harness then runs with zero macOS TSan reports. macOS and Linux
ASan/UBSan runs pass. GCC TSan cannot initialize inside the Colima VM because
it rejects the io_uring mmap address layout before `main`; this is recorded as
a tool/environment limit rather than reported as runtime evidence.

No uncovered first-version loan shape remains in the implemented surface:
borrowed buffers, capability owners, aggregate releases, submission failure,
late/duplicate publication classification, frame reuse, and nested disk-worker
adapters all have direct tests. Cancellation and hidden-reaper ownership are
explicitly outside v0.37; normal and recoverable exits wait for terminal state.

## Phase C evidence

### Shared contract and backend implementations

- **macOS:** kqueue carries per-lane EVFILT_USER wake hints through one waiter
  thread to lane conditions; disk work never executes on that waiter.
- **Linux:** every executing lane owns one io_uring and eventfd. POLL_ADD
  multishot is used when accepted; `-EINVAL` falls back to one-shot, and every
  consumed terminal CQE explicitly rearms it. A test-only forced-one-shot build
  observes more than one poll arm (155 in the latest local strict run), while
  the default multishot build observes one arm per lane (4).
- **Windows:** every executing lane owns one IOCP port, and queued completion
  status is the same lossless park/wake endpoint used by the shared mailbox
  state machine.

The harness starts three concurrent submitter lanes and proves distinct owner
endpoints, races 257 frames from four disk workers into one mailbox, crosses
the 64-node drain bound, reuses one frame for 128 generations, injects one
submission failure, classifies one stale and one duplicate publication, forces
a real park, bounds help depth at one, and tests mixed compute/completion
progress. Strict local C11 builds use `-Wall -Wextra -Wpedantic -Werror`.
macOS and both Linux modes run successfully; the Windows sources cross-compile
with the same warning policy to a PE32+ x86-64 executable. The final source
audit found that the Windows timing helper multiplied the process-lifetime QPC
tick count by one billion before division, allowing an old-enough host counter
to wrap and corrupt only the reported timing. It now converts the quotient and
remainder separately. The repaired source passes the macOS harness, both Linux
modes, and the strict Windows cross-compile; hosted Windows execution remains
the matrix item below.

The compiler embeds the shared/platform sources and headers and links them
whenever emitted LLVM names a native completion adapter or the parallel ABI.
All executable integration-test paths use one shared runtime-link helper after
the native conformance adapter exposed and drove the repair of an omitted
completion unit. A real program test actualizes a world-bearing wrapper beside
pure compute and executes W0/W1/W4. D3 tests pin the sequential W0 mapping,
the one-compute-lane completion-capable W1 mapping, and larger compute-lane
counts. Optimized program integration is 56/56 green; the default and `--par`
whole-corpus paths both link and execute.

Target qualification remains exact and per triple. The v0.37 review note pins
the unchanged fifteen semantic IDs and target guarantees, validates each IR
target-action record against its catalog row, and maps only already-qualified
native macOS/Linux operations to completion adapters. IOCP CI coverage does
not add Windows compiler qualification.

### Hosted completion matrix

GitHub Actions run
[`32920412577`](https://github.com/mbbill/Whitefoot/actions/runs/32920412577)
tested exact implementation revision
`0bb7b97b83e3a7286cac812bd0e6d295aca00add` after it was pushed to
`codex/io-model-completion`. The run started at 2026-08-26 01:49:31 UTC and
completed successfully at 01:49:56 UTC. All three independent jobs succeeded:

| Host | Job | Result | Completed UTC |
|---|---:|---|---|
| Ubuntu | `98032760765` | success | 01:49:42 |
| Windows | `98032760835` | success | 01:49:55 |
| macOS | `98032760876` | success | 01:49:46 |

The Ubuntu job strictly compiled and executed both the normal io_uring path
and `WF_IO_TEST_FORCE_ONESHOT`; the macOS job strictly compiled and executed
the kqueue path; the Windows job strictly compiled and executed the IOCP path.
Each build used C11 with `-Wall -Wextra -Wpedantic -Werror`. The subsequent
closure revision changes only project records and removes the superseded
handoff and candidate; the runtime/compiler bytes exercised by this matrix are
unchanged.

## Closure

Phases A, B, and C are complete. The cross-host matrix is green, the adverse
Phase B measurements remain prominent and unsoftened, the conformance boundary
is exact, and D1 through D5 are recorded as flagged decisions. The superseded
handoff and candidate draft are deleted with this closure, and this batch
record moves to `docs/done/`.

The final merge packet supplies the closure commit identity and its canonical
`make check` result; embedding a commit's own identity here would be
self-referential. No branch revision has been merged into `main`.

The required agent-document scan covered this record and
`docs/current-plan.md`. Every stop-word match is either the final merge rule,
the mandated scan vocabulary itself, a runtime identifier such as `waiter` or
`owner_lane`, or an English substring in a semantic term such as `ownership`
or `representation`. A broader literal-reader pass found no instruction to
pause during branch work. The final merge packet gives the line-by-line match
classification.
