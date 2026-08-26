# Current Plan — world-region I/O completion

Status: COMPLETE ON `codex/io-model-completion` as batch 0082.

This file records technical state only. The specification activation,
compiler and conformance migration, adverse-result handling, three-host CI,
and documentation closure are complete on the work branch. The branch has not
entered `main`; the only remaining repository action is the owner-approved
merge of the exact green revision named in the final merge packet.

Active language authority: v0.37 at `spec/kernel-spec.md`, SHA-256
`6ace763ae2c2d20127f9218ed93ef8865312f68e62d40a23dbc4757d40160c6b`.
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

## Completion evidence

- GitHub Actions run
  [`32920412577`](https://github.com/mbbill/Whitefoot/actions/runs/32920412577)
  succeeded on macOS, Ubuntu, and Windows at exact implementation revision
  `0bb7b97b83e3a7286cac812bd0e6d295aca00add`. Ubuntu exercised normal and
  forced-one-shot io_uring, macOS exercised kqueue, and Windows exercised
  IOCP. The closure revision changes only records and removes superseded
  research instructions.
- The v0.36 and v0.37 native adapters each reach `Pass=500 Skip=1`; all 501
  case `id/rules/expect/status` declarations compare equal.
- The active v0.37 specification has SHA-256
  `6ace763ae2c2d20127f9218ed93ef8865312f68e62d40a23dbc4757d40160c6b`,
  contains 137 covered rules, and has 29 unbroken activation links.
- The exact closure revision receives the canonical `make check` run reported
  in the final merge packet. This statement records the finished handoff
  boundary and does not create a separate repository gate.

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
