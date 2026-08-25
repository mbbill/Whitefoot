# Current Plan — world-region I/O completion

Status: EXECUTING on `codex/io-model-completion` as batch 0082.

This file records technical state and sequence only. Every branch step below
continues without an intermediate repository gate, including specification
activation, adverse-result handling, CI, and documentation closure. The only
repository stop is after the exact final revision passes canonical
`make check`, when that revision and its merge packet are submitted for the
single approval required to enter `main`.

Active language authority: v0.37 at `spec/kernel-spec.md`, SHA-256
`f772f2aec5e0da963c1cb9d8607a9e87cd3ad03cb71f3b6532451404d4d07bb5`.
It supersedes v0.36, whose exact bytes are archived as
`spec/kernel-spec-v0.36.md` at SHA-256
`fd57cfc4bfcf685f14b073c98e149c8a44a201dc79fbd76075ebd49a87995c62`.

## Objective

Replace the source `external`/`blocks` effect atoms with proof-carrying world
regions, preserve v0.36's global outside-world order conservatively, and make
host waits a completion work source selected from trusted target-action
metadata. The compiler must retain one ordinary semantic/lowering path,
facts-off acceptance, terminal in-flight loans, and the written-claim
discipline while supporting kqueue, io_uring, and IOCP behind one C contract.

## Completed technical sequence

1. **Phase A — specification and compiler migration.** v0.37 is active on the
   work branch. The compiler implements world-region kinds, capability world
   vectors, exact kinded EFF-2 projection, release contributions, conservative
   world alias/order identities, target-action metadata and transitive
   summaries, IR type erasure for region-only nominal instances, permission
   footprints, and deterministic `--io-ledger` output. Forty-four conformance
   sources and nineteen additional workloads are syntax-migrated; all
   conformance IDs, rules, expectations, and statuses are unchanged.
2. **Phase B — kqueue prototype and measurement.** The fixed disk pool,
   preallocated intrusive mailbox nodes, generation-tagged frames,
   release/acquire publication, executing-lane affinity,
   progress-then-rescan/announce-then-recheck parking, bounded helping, and
   terminal loan states are implemented and exercised. The controlled
   directory-walk result is adverse and therefore recorded rather than used as
   a stop: the workload exposes no actualizable world-I/O pair, and final
   v0.37 best times are 24.2%–30.0% slower than v0.36 because adapter overhead
   dominates.
3. **Phase C — shared backend matrix.** macOS uses kqueue plus one waiter,
   Linux uses per-lane io_uring rings with POLL_ADD-on-eventfd multishot and
   explicit one-shot rearming, and Windows uses per-lane IOCP ports. The shared
   harness covers submission/completion, MPSC publication, generation reuse,
   forced submission rollback, terminal loans, multi-lane affinity, bounded
   helping, mixed-load progress, and completion-vs-condvar timing. Native
   compiler qualification remains macOS/Linux only; the Windows C harness does
   not widen the reviewed target table.

## Evidence interpretation

- The v0.36 and v0.37 native adapters each reach `Pass=500 Skip=1`; a
  mechanical comparison of all case `id/rules/expect/status` fields is empty.
- Seventy-two interleaved directory-walk runs all exit successfully and
  publish the same SHA-256. Current W1 and W4 are respectively 0.6% and 1.1%
  slower than current W0 at the best observed time, matching the ledger's
  report that every world-I/O call remains sequential in this program.
- On eleven final-code repetitions, the paired completion/condvar ratio has a
  1.917x median on macOS (1.079x–2.592x) and a 0.659x median on Linux
  (0.109x–0.895x). These are service-path measurements, not an end-to-end
  speedup claim.
- macOS ThreadSanitizer found and drove the repair of one frame-reuse race and
  one kqueue initialization race; the repaired shared state machine runs with
  zero TSan reports. macOS and Linux ASan/UBSan runs are green. Linux TSan
  cannot start in the Colima VM because its runtime rejects the io_uring mmap
  address layout, before the harness begins.

## Remaining continuous sequence

1. Run the shared strict-C harness on GitHub-hosted macOS, Ubuntu, and Windows,
   including Linux's forced one-shot poll path.
2. Close the implementation record: remove superseded handoff/candidate
   material, move batch 0082 to `docs/done/`, and synchronize the README,
   roadmap, patterns, specification record, and compiler inventory.
3. Run the final adversarial diff, repository hygiene and document-literalism
   scans, then canonical `make check` on the exact committed revision.
4. Assemble one merge packet containing the exact revision, complete evidence,
   adverse measurements, conformance before/after boundary, and D1–D5 list.

## Flagged decisions carried by the final revision

- **D1:** preserve one command-wide world-order domain for every former
  `external` action.
- **D2:** a defective schedule may select both the false claim and its
  pre-abort world-effect prefix; theorem T3 forbids taxing correct programs
  with a trap-free permission gate.
- **D3:** `WF_WORKERS=0` selects the sequential world; `1` selects one compute
  lane plus completion overlap and no stealing worker; larger values name the
  compute-lane count.
- **D4:** `external` and `blocks` remain retired reserved words.
- **D5:** the provenance class is named `boundary-derived` with no verdict
  change.

## Preserved boundaries

- Backend names, event mechanisms, worker counts, and schedules are outside
  the language.
- Permission is not obligation. Missing origin, place, alias, kind, or target
  evidence denies overlap; an optimizer fact cannot change acceptance.
- Different capability values never prove world disjointness. The first batch
  keeps the conservative global order; any later narrowing needs an explicit
  trace law and new evidence.
- Completion publication never enters a compute deque. A submitted loan and
  its frame remain live through terminal state, and normal/recoverable exits
  observe all submitted operations before releasing their storage.
- No Windows compiler target row is added by this batch. Later world families,
  cancellation, deterministic replay, writer-visible parallel syntax, and
  world-order narrowing remain project-selected follow-up work.
