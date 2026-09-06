# Current Plan: back the file permit, park on miss, then streams and TCP

Status: IN PROGRESS on `io/t4-resource-relations` (PR #13). Batch 1 (the
backed permit) and batch 2 (park on miss) are done on this branch; batch 3's
slice 1, streams and TCP, landed on 2026-09-05 and its slices 2 through 5 are
open. The previous plan, source-carried proof, is IMPLEMENTED AND ACTIVATED as
v0.40 and is recorded in `docs/done/` and the v0.41 activation.

The active language authority is the specification at `spec/kernel-spec.md`;
its version and digest are the chain tail in `governance/APPROVALS.md`. This
branch carries two amendments, each landed as one change: the amended file,
the archive of the outgoing bytes, the appended approval record, and the
regenerated identity module, so the branch is merge-ready the moment its gate
is green and the owner's merge approval of that exact revision is the
activation. Nothing merges to `main` until the owner approves the exact
revision and canonical `make check` passes on that revision. This document
records technical direction and sequencing; it grants no permission and adds
no workflow gate.

## Rules the owner has stated for this work

Folded here on 2026-09-05 from `docs/ongoing/HANDOFF-2026-09-04.md`, which is
deleted with batch 2's record: these are the standing rules for this branch
that `CLAUDE.md` does not already carry. They are technical direction, not a
workflow gate.

- **Keep implementing on PR #13 and do not merge until the owner approves the
  exact revision.** Merging a half-product is worse than a longer branch.
  Expect the design documents to be revised as the implementation reveals
  problems; fold each finding into this plan and into the batch record, the way
  `docs/done/0106-backed-file-permit.md` §7 does, and never edit a conformance
  verdict or delete a test to go green.
- **T4 is the test for every device API.** If overlap can invent an outcome the
  sequential program never produces, a resource is missing from the API, and
  the answer is on the API and never in the scheduler. The error is the
  system's, so the operation keeps it; the resource is ours, so it comes back
  where the checker can see it.
- **The runtime allocates nothing at run time.** Fixed capacities, one
  reservation at entry, deterministic refusal: no growth, and no per-operation
  blocking fallback.
- **Judge a runtime paradox by one observable:** no completed operation's
  continuation is ever buried. The state machine gets its exhaustive tests
  before it merges, not after.
- **Tests:** never plain `cargo test`; always `cargo test --profile gate`. The
  whole gate stays under five minutes per host, and its cost is cut by
  structure and never by weakening a check. Background a long run and read its
  log; a foreground wait stalls the session.
- **Wording:** repository text and replies say what is being done — walk the
  schedule, check the case, review — and never reach for hostile framing for
  any of them.
- **Reporting to the owner:** Chinese, code first. A concrete example with the
  real lines quoted verbatim and the file named, then the rule; no invented
  terminology, no em-dashes; and say what was verified by running against what
  was only read or compiled.

## Outcome

Two batches, in this order, both on the one branch so the design documents
can change while the implementation teaches us what they got wrong.

1. **Backed `FilePermit` (specification v0.45).** Constitution T4 says a
   finite resource a system operation consumes is an owned value drawn from a
   factory whose capacity never exceeds what the target provides. Today's
   `FilePermit` is proof-only ([SYS-10]: never returned, promises no
   descriptor, erased before the native ABI), so `close` produces nothing the
   checker sees, a pipeline overlaps it with a later `open`, the host answers
   `EMFILE` for a schedule the sequential program never produces, and the
   runtime hides that with the descriptor retirement ledger
   (`compiler/src/backend/completion/contract.h:409-786`,
   `runtime.c:1142-1786`, `bridge.c:1271-1330`, Windows twins). This batch
   makes the permit real and deletes the ledger.
2. **Park on miss** (`research/investigations/io-model/PARK-ON-MISS.md`,
   63 recorded decisions). One scheduler for compute hand-outs and I/O
   completions; a join that misses parks its stack; the completion record
   lives in the submitting frame; no per-operation blocking fallback. Its
   implementation is gated on the enumeration harness its §11 specifies.

## Batch 1: backed `FilePermit`

### Language delta ([SYS-10], [SYS-11], [SYS-14], tables)

- `FileFactory` carries a capacity fixed at program start: the descriptors
  the target provides to this program, less the runtime's own and the
  handles the entry already holds. The capacity is not a source constant;
  it is observed only through `reserve_file`'s outcome.
- `reserve_file(factory: &uniq FileFactory) -> Result<FilePermit, IoError>`;
  the exhaustion member is `ResourceExhausted`, the program's own source-order
  outcome. Effects unchanged; still no host call.
- Three explicit closes return the permit, total, discarding the close
  diagnostic exactly as derived release does today:
  `close_read(file: own ReadFile) -> FilePermit`,
  `close_directory(directory: own DirectoryRead) -> FilePermit`,
  `close_directory_source(source: own DirectorySource) -> FilePermit`.
  Target contract may-suspend, terminal, like the opens. Derived (implicit)
  release still closes and burns the credit: FIRST-PRINCIPLES §12's first
  disposition. [SYS-11]'s "declares no separate explicit-close operation"
  is replaced.
- Consequence stated in [SYS-10]: an open holding a permit cannot fail for
  want of a descriptor; `ResourceExhausted` on an open names only honest
  target exhaustion (a limit changed outside the program, such as the
  system-wide file table).
- Not in this batch, named as the later step: a proof-admitted `reserve_file`
  whose domain (credit available) the checker discharges from a factory
  counter carried as a numeric fact, making exhaustion unrepresentable.

### Compiler

- `compiler/src/resolution/catalog.rs`: `reserve_file` result
  `ok_nominal(FILE_PERMIT, IO_ERROR)`; three close operations; effect rows.
- `compiler/src/backend/emitter/system.rs`: `emit_reserve_file` becomes a
  call to the runtime's credit counter (`wf__file_reserve`) returning the
  `Result` shape (precedent: `emit_relative_path`); the close operations lower
  to the existing native close plus the credit return; the permit's ABI
  representation stays the erased bit.
- `compiler/src/backend/qualification.rs`: ABI symbols and release table.
- Pinned sentences, driver fixtures, and the Rust test corpora that embed
  `reserve_file` programs are rewritten to the `Result` shape.

### Runtime

- One process-wide credit counter set at completion-runtime start from the
  target's descriptor budget (POSIX `RLIMIT_NOFILE` less the runtime's own
  descriptors and the entry's handles; Windows a fixed default the target
  exceeds). `wf__file_reserve` decrements or refuses; the close path returns
  a credit after the native close.
- The retirement ledger, its award order, `wf_bridge_retire_and_retry_direct`,
  `wf_completion_retirement_*`, the Windows twins, `retirement_interleave_probe.c`
  and the harness tests that exist only for them are deleted. An open that
  still meets `EMFILE`/`ENFILE` returns `ResourceExhausted` at once.

### Conformance and corpus

- `tests/conformance/cases`: every case that calls `reserve_file` moves to the
  `Result` shape with its verdict re-derived; new cases: reserve exhaustion is
  `Err(ResourceExhausted)` in source order; `close_read` returns a usable
  permit; a permit-holding open never answers `ResourceExhausted` for the
  program's own descriptors. Manifest entries and rule citations updated.
- `tests/programs` and `research/experiments` programs (56 files, 108 call
  sites) rewritten.

### Activation

- Landed as one change under the no-candidate rule after `main` was merged
  into the branch on 2026-09-05: `spec/kernel-spec.md` declares
  `Status: ACTIVE v0.45` over v0.44's text (the branch's amendment was
  written as v0.42 over v0.41 before `main` activated v0.42 to v0.44 for
  region spelling, loop-body regions and the fact machinery); the v0.44
  bytes are archived, the `ACTIVE-SPEC:` record is appended in
  `governance/APPROVALS.md`, the derivation ledger carries the v0.45
  amendment, and `compiler/src/spec_identity.rs` is regenerated. The owner's
  merge approval of the exact revision is the activation.

### Slices

Slices 1–3 landed on `io/t4-resource-relations` (PR #13) on 2026-09-04;
slice 4 is in progress there.

1. Specification candidate + catalog + emitter + runtime counter (the tree
   compiles; old programs fail at the new signature). Done.
2. Corpus and fixture rewrite; conformance verdicts; new cases. Done.
3. Retirement ledger deletion and its tests; capacity accounting at start.
   Done: the ledger, its award order, the retire-and-retry paths on every
   route, the Windows open-order tickets and resource-attempt lock, and the
   two probes that scripted them are gone.
4. Docs (`docs/patterns.md` P9/P12, roadmap BOUND-1 current state, derivation
   ledger); canonical `make check` green; batch record in `docs/done/`.

### Decided 2026-09-04: a refused open hands its permit back

The owner's rule: the error is the system's, so the open keeps it; the
permit is ours, so it comes back where the checker can see it. The four opens
answer with `FileOpenOutcome`, `DirectoryOpenOutcome`, and
`SourceOpenOutcome` (`Opened(value: …)` / `OpenFailed(error: …, permit: …)`),
no count changes on a failure, and `propagate` no longer applies to an open.
Landed on the branch (`docs/done/0106-backed-file-permit.md` §7).

## Batch 2: park on miss

**Status 2026-09-05: done on this branch except the one open decision below.**
All four slices landed; the record is
[`docs/done/0107-park-on-miss.md`](done/0107-park-on-miss.md). What is not
settled is design §12 item 1, the compute-miss regression and its fallback,
which is the owner's decision and is stated at slice 4's status.

Sequenced after batch 1. The plan file is the specification of the work; its
§11 enumeration harness is the merge gate and the replay tool. The plan's
decisions file records why each mechanism is shaped as it is; implementation
findings that contradict it are folded back into the plan on this branch
before the code lands.

### Slices (started 2026-09-04 on `io/t4-resource-relations`)

0. **Measurements before a line of the compiler changes** (design §12). The
   decisive one first: the cost of one hand-written stack switch against the
   2.2 µs park-and-wake figure the tree measured, on this host; a switch that
   is not well under it removes the design's reason to exist. Home:
   `research/experiments/park-on-miss-switch-cost/`. The four-stage chain in
   C on io_uring runs on the Linux runner later in the batch; record growth
   per frame is reported from the stack ledger once slice 2 exists.
1. **The scheduler core and its enumerator** (design §5–§7.1, §11). One C unit
   that reaches shared state only through the seven primitives of §7.1, named
   in one header; a cargo test that compiles that unit against a replacement
   header and drives it with a controlled scheduler enumerating every
   interleaving of primitive steps for (T=1,S=2), (T=1,S=3), (T=2,S=3),
   (T=2,S=4), checking every §11 item at every step. This slice is the merge
   gate; nothing in slices 2–3 lands while it fails.

   **Status 2026-09-04, evening: the core, the enumerator and the gate exist;
   the enumeration passes at the four configurations.** Files, all under
   `compiler/src/backend/sched/`: `core.h`, `core.c` (the core), `prim.h`
   (the seven primitives, two implementations), `prim_host.c` (the host's),
   `switch.h` (the one stack switch, shared by the host primitives and the
   enumerator), `smoke.c` (real threads, `make -C compiler sched-smoke`),
   `enumerate.h` (what a schedule is written against), `enumerate.c` (the
   controlled scheduler, the checks, the searches), `schedules.c` (§10's
   schedules as scenarios). The cargo wrapper is
   `compiler/src/backend/tests/sched.rs`, one test per configuration, and
   `make -C compiler sched-enumerate` runs the same four sweeps under
   `completion-test`.

   The enumerator is the shape the previous status described: one OS
   thread; T simulated threads as coroutines on host stacks, each calling
   `wf_sched_run`; every primitive announces its step and switches to the
   controller, which chooses the next enabled process and lets it perform
   the step on resumption; the lock blocks while held, the park until a
   wake, a yield until another process has written shared state since the
   yielder last returned from one (the only spin that can observe anything
   new); a device process per submitted record whose one step completes it
   and bumps the epoch, while target progress drains every completed record
   deterministically, one announced step per record; the §11 invariants
   after every step (the state machine's edges and who may take each, no
   stack on two lists or executed by two threads, nothing committed or made
   READY on a stack still being executed, exactly one enqueue per park, I2
   at every deque write, I3 at every free-list pop, I4 at every step, no
   idle sleeper beside a READY stack, the posted status or a hand-out), and
   a terminal check for what liveness would hide (an entry that never
   returned, a completion never drained, a stack parked with nothing left to
   wake it, a spinner with nothing left to change what it spins on).

   Three searches were built; two remain and the second is the gate. Plain
   DFS with re-execution is the reference and is feasible at one thread
   only. Dynamic partial-order reduction with sleep sets over the objects
   each step touches needed 10^5 to 10^6 re-executions per schedule at two
   threads, did not finish the heavier schedules, and was removed once the
   third search existed. The explicit-state search checkpoints every state
   (the live bytes of every stack, the core, the device, the checker's
   bookkeeping, the schedule's state), digests it, never explores a state
   twice, takes a step that touches no shared object (an unlock, a switch,
   a yield's return, an owner's load of its own deque's bottom) alone, and
   explores a device completion only where a thread's next step can observe
   it (a progress pass, an epoch capture, a park, a blocked spin) or where
   no thread can move. What made it fit was canonical state: the wake epoch
   and the yield bookkeeping became per-thread flags, a wake reaches only a
   thread inside its capture window, and the digest leaves out what the
   protocol makes dead (the per-thread counters, a lane's steal seed, a
   running stack's saved pointer, a deque's cells past the one being
   claimed, a freed slot's record and frame, bookkeeping the checker has
   consumed); each of those had made equal states hash apart, by an order
   of magnitude in all. The core's counters are credited per explored step,
   as the difference each step made, because a terminal reached twice is
   pruned before it is counted. At one thread, where the full walk is
   feasible, the search reaches exactly the arms the full walk reaches on
   every schedule, and the cargo wrapper asserts that. The idle bitmap's
   read-modify-writes and the section-named lock (finding 7) were made for
   the reduction and stay: the first is the simpler core, the second costs
   the host nothing.

   **What the enumeration found in the core, each fixed in place and
   marked with the schedule that reached it:**

   1. *The entry thread's last pool stack was unresumable* (S20, T=2, S=3).
      After the status post the entry thread switches to its host stack
      and pushes the pool stack it left; a worker whose start came later
      popped that stack and switched into the `wf_prim_fail` after the exit
      switch. The far side of that switch now continues the loop like every
      other EMPTY switch, with the arriving thread's obligations.
   2. *A late publisher acted on a cancelled park* (S3, T=2, S=3). The
      publisher loaded the waiter, the parker read DONE and cancelled, ran on
      and finished, and the publisher then read a phase no park reaches; in
      other orders it would have notified the stack for a later park's
      event. A publisher now claims the registration with a
      compare-exchange before touching the waiter's phase, and a parker that
      finds DONE cancels only when it takes its registration back; when the
      publisher claimed first the parker switches away and is resumed
      through the ordinary arms. §6's cancel arm that consumes NOTIFIED
      (§11 item 7) is therefore unreachable, and the enumerator fails an
      execution that takes it; §6's sentence that a RUNNING waiter has
      nothing left to wake is obsolete.
   3. *The publisher touched a dead record* (S1, T=2, S=3, a crash). DONE
      was stored before the waiter was read, DONE lets the joiner return,
      and an I/O record is a block of the joiner's frame. The record now
      goes PENDING, COMPLETING (the publisher claims), DONE (the publisher's
      last touch); a joiner that meets COMPLETING waits for DONE; the same
      hole would have let a compute slot be released and re-acquired under
      a late publisher.
   4. *The in-place waiter slept past another thread's drain.* The I/O arm
      registers no stack, so a drain elsewhere stored DONE and woke nothing.
      The arm registers `WF_SCHED_WAITER_IN_PLACE`, and a publisher that
      claims the marker bumps the epoch.
   5. *The compute arm never drained* (S1, T=1, S=3). Its empty-handed turn
      only yielded; with the target held by a stack parked on I/O and no
      other thread to drain, it spun forever. The turn now runs target
      progress and yields only when there was nothing to drain.
   6. *A worker start found no stack* (S1, T=2, S=3). §5's floor argument
      assumed a worker takes its stack at its start; a start is scheduled
      by the host and can come after the entry thread has parked every free
      stack. `wf_sched_start_thread` takes the worker's stack on the thread
      that creates it, at creation, where at most one stack is parked;
      `wf_sched_run` for a worker finds it reserved. §11 item 21 is checked
      at that pop.
   7. *Primitive 1 gained the two read-modify-writes the idle bitmap uses,
      and primitive 5's lock names its section* (`prim.h`). The first turns
      five compare-exchange loops into one step each so two threads' bits
      commute; the second lets a critical section be classified as a read
      of its list when it pops an empty one. Both exist for the reduction;
      the host ignores the section.

   Observations that are not defects: with one thread an I/O completion
   that arrived before the join still costs a park, because only a drain
   stores DONE and only the idle step drains (S20's line one needs a second
   thread); the late third line is unreachable with one thread for the same
   reason. The state counts at two threads are the honest size of the
   interleaving space at primitive granularity, so the schedules were sized
   to it: S1's chain is three iterations and runs at one thread, where its
   depth assertion (the pool used to S minus T) and S9's slot exhaustion
   are reached at a few thousand states; S23 carries group C item 11's two
   arms with one body (J1 compute, J2 a read, a read of main's own) at the
   floor configurations, where those arms live; S5 asserts the depth at two
   threads (the sibling's stack and main's park together); S22 keeps its
   grandchild's read, because without it the crossed exhaustion at (2,4) is
   unreachable, which its coverage check said. S8 (the floor's overflow
   record), S10b (Windows) and S11 (retired) are not enumerable and are
   absent from `schedules.c`.

   The sweeps on this host (Linux, four cores, clang 18, `-O2`): (T=1, S=2)
   fifteen schedules in 0.04 s; (T=1, S=3) fifteen in 0.05 s; (T=2, S=3)
   eighteen in 9.8 s, the largest S22 at 2.6 million states; (T=2, S=4)
   seventeen in 35 s, S22 at 12.3 million states and about half a gigabyte
   for the table of states seen. The Makefile target runs the four together.

   Not in this slice: item 24's recorder of a run's data and completion
   order (the enumerator replays a printed sequence of picks, which is the
   same mechanism) and a sanitizer build of the enumerator (its coroutine
   stacks defeat the address sanitizer's stack model; the smoke runs clean
   under ASan and UBSan on Linux).
2. **Emitter** (design §8): the completion record as one opaque block of the
   submitting frame with its size and alignment as an ABI constant asserted on
   both sides; one lowering for every I/O operation, submit then join; the
   direct family, the inline arm, `stackless.rs` and the Windows verdict fork
   removed; compute join order reversed through one `compute_join_order`.

   **Sequencing decided 2026-09-05, after reading the bridge.** The one
   lowering (no inline arm, no direct family) is sound only once the
   runtime accepts every submission, and today's bridge refuses for
   throughput reasons (`wf_bridge_positioned_read_runs_on_caller`, a path
   that does not fit the pool's record, a target with no ring), answers
   pool exhaustion with a capacity park, and finds an operation by a token
   into a pool of 256 slots. Dropping the inline arm before the record is
   the frame's would turn every refusal into an abort or a capacity wait,
   which the owner's rule forbids. So the slice lands in three steps, each
   with the gate green: (a) `compute_join_order`; (b) the record block ABI
   (the token slot becomes an opaque block of the size and alignment the
   completion header states, asserted on both sides) together with the
   removal of `stackless.rs`, the writer-frame submit ABI and the
   `WF_PAR_WITH_WRITER_SCHEDULER` runtime variant, the runtime otherwise
   unchanged; (c) the one lowering, taken together with the first runtime
   step of slice 3, in which the bridge finds the record by address, loses
   its pools, capacity waits and refusals, and executes an operation with no
   kernel completion form inside its engine, publishing a completion like
   any other. Step (c) therefore opens slice 3 rather than closing slice 2.

   **Status 2026-09-05.** (a) landed as `bccf181`: `compute_join_order` in
   `emitter/parallel.rs`, consumed by `emit_overlap_joins`,
   `overlap_join_tail` and `block_exit_label`; a finding on the way, that a
   group with a completion member between compute members is handed out
   nowhere by today's emitter (its compute members are emitted as plain
   calls), so the completion half of the rule is pinned by the function's
   unit test only. (b) landed as `89688f9`: the record block
   (`WF_COMPLETION_RECORD_BYTES`, `WF_COMPLETION_RECORD_ALIGN` in
   `completion/contract.h`, mirrored in `native_completion_api.h`, asserted
   by the C units that store a record and by a Rust test that parses the
   header), `stackless.rs`, `tests/stackless.rs`, the writer-frame submit
   ABI and the `WF_PAR_WITH_WRITER_SCHEDULER` variant gone. The runtime half
   of (c) landed next (the commit after this note): `wf_completion_record`
   (120 bytes, reserved as 128 at 8) is the frame's block, `contract.h` is
   its header and `runtime.c` keeps only the wake epoch and the park; the
   slot pool, tokens, claims, milestones, drains, consumes, dependent
   frames and every capacity wait are deleted from the core, the bridge, the
   file adapter (whose queue is now an intrusive FIFO through the records)
   and the io_uring adapter (whose `user_data` is the record's address and
   whose depth is its own parameter, `WF_LINUX_IO_URING_DEPTH`); every
   submit ends in `wf_completion_record_complete`, which is
   `wf_sched_complete` on the record; the joins wait in place through the
   core's `WF_SCHED_WAITER_IN_PLACE` registration; the core's park and wake
   are the bridge's (`wf__sched_host_epoch`, `wf__sched_host_park`,
   `wf__sched_host_wake`, weak in `prim_host.c`, strong in the bridge), so
   one wake rings the ring's eventfd and the condition variable alike. The
   harness and the probes were brought to the record API, the tests of the
   deleted machinery retired with design §7 named at each site, and the
   properties that remain retested (one completion per submission under a
   race, an in-place registration claimed by its completion, a helper
   completion waking a waiting join, more operations outstanding than the
   old pool and the ring depth).

   Not done in that step: the Windows record port. The Windows units keep
   their own completion core and pools; the bridge header's Windows-only
   entries moved to `windows_completion.h`, and both platforms state the
   same reservation, but nothing Windows was compiled or run here (this
   host has no Windows toolchain, and the gate's matrix is Linux and macOS).
   The port is slice 3's Windows twins bullet and stays there.

   The emitter half of (c) landed next: every ordinary I/O wrapper in
   `emitter/system.rs` reserves the record in its own `alwaysinline` frame,
   submits and joins, and derived release closes through one shared
   `wf.sys.close` helper that does the same; the direct family, the
   qualification rows that named it and every `declare` of a `_direct` symbol
   are gone from the compiler, the bridge, the Windows runtime and the
   harness, and `module_requires_completion_runtime` is now "the module calls
   a submit". Two things the one lowering exposed and this step answered: the
   `open_file`/`open_directory` kind check and its close on mismatch left the
   wrapper for the runtime that already decides them from `expected_kind`, so
   no emitted code holds a `struct stat`; and a thread joining its own
   submission now runs that record itself when it is still queued
   (`wf_file_adapter_claim_own`), because with helpers pinned an ordinary
   write would otherwise wait behind an unrelated blocked one.
3. **Runtime** (design §7): the core replaces both parallel runtimes and both
   writer schedulers; the bridge, adapters and rings find the record by
   address and lose their pools, capacity waits and tokens; the I/O joins park
   the stack; the floor's entry runs `wf__main_body` on a pool stack selected
   at link time; the Windows twins in `wf_floor_windows.c` and
   `windows_bridge.c`.

   **Order and the Windows decision, 2026-09-05.** (i) The records-by-address
   step is in (`3acc3e9`, with the one lowering at `effe2e2`). (ii) An
   emitter defect found on the way, a mixed overlap group handing out none
   of its compute members, is fixed as its own commit: the gate was in
   `FunctionEmitter::new`, which dropped any group holding a completion
   member (deliberately, and a test pinned it), so `overlap_handed_out` was
   empty for every mixed run; the owner ruled on 2026-09-05 that I/O and
   compute members overlap in one group, so a submitting member needs no
   lane frame and is the one member allowed to suspend, the join site's
   joins run through `compute_join_order`, and `block_exit_label` follows.
   Two fixtures pin it, joined C2, IO, C1 and IO, C2, C1 at every worker
   count. (iii) The core becomes
   the runtime on POSIX: `sched/entry.c` carries the `wf__par_*` ABI and the
   pool policy over `core.c`, the floor runs the entry on a pool stack when
   the core is linked, every I/O join parks through `wf_sched_join`, and
   `par_runtime.c` with both writer schedulers retire. (iv) Windows is done
   as shared code, not as a second copy, which the owner ruled on 2026-09-05
   and which the design's core-and-platform-layer shape already implies:
   one core, one bridge over one record (`windows_bridge.c` folds into
   `bridge.c` with the adapter chosen per platform), the adapters behind one
   interface (io_uring, IOCP, the bounded pool), a `sched/prim_windows.c`
   beside `prim_host.c` (fibers for the switch, the completion port for the
   park and wake, `_beginthreadex` for threads), and `wf_floor_windows.c`
   reduced to what only Windows has (the exception-code classification, the
   fiber entry); `par_runtime_windows.c`, `writer_scheduler_windows.c` and
   `windows_completion.c` (its own record pool) go. The real Windows host
   in `.github/workflows/io-hosts.yml` (`completion-windows`) is the gate
   for that step; it had been red on this branch since `554f1d9` for a
   stale expectation line and is repaired at `7b9d41f`, so the steps it
   had been skipping since then run again from the next push.

   **Status 2026-09-05, step (iii) landed.** `compiler/src/backend/sched/`
   gains `entry.h` and `entry.c`, the platform layer of design §7 over
   `core.c`: the process's one `wf__sched_core`, the startup policy
   `par_runtime.c` had (`WF_WORKERS`, the machine's core count, the
   `WF_PAR_MAX_LANES` ceiling, a value below two meaning no workers) plus the
   stack count's own setting (`WF_STACKS`, else the thread count plus eight,
   raised to the core's floor of threads plus one, at
   `wf__floor_stack_bytes()` each), the worker threads with their reserved
   host stacks and the rendezvous that made "the pool started" mean "the pool
   can take work", and the `wf__par_*` module ABI as thin functions over the
   core. `wf_floor.c` tests one weak symbol, `wf__sched_entry_stack`, and when
   the core answers it the entry runs `wf__main_body` on a pool stack whose
   bottom is the scheduler loop, posts its status there and returns it on its
   own host stack; its two `pthread_create` fallbacks stay and are unreachable
   on that branch. The floor's per-thread attach is now the alternate signal
   stack and the host stack's bounds, and the per-stack half is
   `wf__floor_set_stack_bounds`, which the switch writes from the reservation
   record. Every I/O join runs the core's rule when the calling thread is on a
   pool stack and waits in place when it is not.
   `par_runtime.c`, `completion/writer_scheduler.c` and
   `completion/writer_scheduler.h` are deleted, the core is staged under
   `module_requires_parallel_runtime || module_requires_completion_runtime` at
   every link site, and the Windows units kept the retired writer ABI behind
   `#if defined(_WIN32)` in `completion/bridge.h` until step (iv) deleted them
   with the twins that used them.

   Two defects the step exposed and fixed, both of them consequences of the
   scheduler loop being the thing that sleeps. The bridge's wake seam
   (`wf__sched_host_epoch` / `_park` / `_wake`) answered "not mine" until the
   whole bridge had initialized, so a thread that parked before the program's
   first operation slept on `prim_host.c`'s own condition variable while every
   later wake went to the bridge's — a lost wake, and with no timeout in this
   design, a hang; the wake epoch now has its own `pthread_once`, taken by
   whichever of a seam call or the bridge's initializer arrives first. And the
   bridge's `atexit` shutdown now returns without destroying the ring when the
   pool is still running, because a detached worker may be asleep inside it.

   **Worker start, measured before chosen (2026-09-05).** The step's first
   cut started the workers eagerly at the core's entry; design §5 keeps the
   start lazy, under a `pthread_once` at the first lane acquisition. Both
   were built and timed on this host (4 cores), one `--emit-llvm` module
   linked against each runtime. io-completion-bench `many_files_wide` at the
   default configuration (FILES=8192, MAX_KIB=16, `WF_WORKERS` and
   `WF_IO_HELPERS` unset), best of 7: before step (iii) 0.1063 s, eager
   0.2851 s, lazy 0.1083 s; eager at `WF_WORKERS=1` 0.1074 s, so the cost is
   the idle workers' progress-pass-and-park traffic against the epoch every
   submit bumps. The lazy start shipped: a program that submits and never
   hands out runs on the entry thread's loop alone, and its parked stack is
   resumed by that loop when the completion arrives. The same pair on
   `tests/programs/par_layout.wf`, best of 9, before and after step (iii):
   W=1 1.5210 s and 1.5231 s, W=2 0.7791 s and 0.8301 s, W=4 0.4067 s and
   0.5711 s, W=8 0.4213 s and 0.8307 s; at W=4 lazy 0.5822 s against eager
   0.5711 s, so that regression is not the start but park-on-miss itself at
   a compute miss, which is §12's first item. Its bar is "within noise" and
   this misses it; the design's stated fallback is nested runs of
   never-suspends jobs at a miss, which needs the target-action bit at the
   hand-out. Slice 4 measures that against the park before either is chosen.
   **Status 2026-09-05, step (iv) landed.** Windows takes the same runtime
   every other target takes. What is shared is every piece of logic:
   `sched/core.c` and `sched/entry.c`, `completion/runtime.c`,
   `completion/file_adapter.c`, `completion/bridge.c` and
   `completion/contract.h`. What stays a platform leaf is only what calls the
   host: `sched/prim_windows.c` beside `sched/prim_host.c`,
   `completion/windows_iocp.c` beside `completion/linux_io_uring.c`,
   `completion/file_windows.c` beside `completion/file_posix.c`,
   `completion/wait_windows.c` beside `completion/wait_host.c`, and
   `wf_floor_windows.c` beside `wf_floor.c`. `windows_runtime.c` keeps the
   UTF-16 bootstrap, the NtCreateFile relative opens, the directory batch and
   the host calls the Windows file leaf makes; of its descriptor ledger only
   two facts survive, the resource class and whether the completion port has
   taken the handle, because those are the two the ring and the leaf cannot
   do without. Deleted: `par_runtime_windows.c` and its probe,
   `completion/writer_scheduler_windows.c`, `completion/windows_completion.c`
   and its header and probe, `completion/windows_bridge.c`,
   `completion/windows_blocking.c` with its header and probe,
   `completion/windows_bridge_capacity_probe.c`,
   `completion/windows_compiler_capacity_observer.c`,
   `completion/native_completion_api.h`, and
   `tests/programs/completion_windows_capacity.wf`, which exercised a slot
   capacity that no longer exists.

   `entry.c` carries one startup policy with no `#if` of its own: thread
   creation with a reserved host stack, the thread's attach and detach for the
   switch, the machine's core count, the once and the rendezvous are all
   behind `prim.h` now, the last two written over the core's own atomics and
   yield rather than over a second sleep mechanism. The switch on Windows is
   fibers, the reservation is address space with one committed page per slot
   for the core's state header, and `wf__floor_set_stack_bounds` is defined
   strongly in the Windows floor and does nothing, because that floor
   classifies an overflow by exception code. What that floor does need per
   stack, found on the real host on 2026-09-05 after the port landed, is its
   emergency stack on each pool fiber: Windows keeps a stack guarantee for the
   calling thread or fiber and a fiber takes it only when set from inside, so
   `wf_prim_fiber_main` runs the floor's attach as every pool fiber's first
   frame, and `entry.c` reaches that attach through `wf_prim_floor_attach`
   rather than naming it, because one link admits one weak default per symbol
   and a PE weak default satisfies only its own unit's references. Before it did, the io-hosts overflow proof passed on one push
   and segfaulted on the next with the same bytes, because a pool fiber
   overflowed with only what was left of the guard page under the handler;
   the job now runs each configuration five times. One rule now covers WF_WORKERS,
   WF_STACKS and WF_IO_HELPERS on both platforms: unset or empty is the
   caller's own default, an integer from 0 through the setting's ceiling is
   that number, and anything else is a configuration error that ends the run
   before the program body with one line on the diagnostic channel
   (`whitefoot scheduler: WF_WORKERS must be an integer from 0 through 64`),
   nothing on the output channel and a nonzero status. WF_WORKERS 0 or 1 is
   still the sequential opt-out the corpus and every reference build use.
   That rule replaces two: POSIX read a malformed value as the opt-out and
   clamped an oversized one, and Windows accepted only 2 through 64. It is a
   decision the owner can reverse. `WF_WINDOWS_BLOCKING_WORKERS` is gone;
   WF_IO_HELPERS is the one helper setting on both.

   The wake on Windows is the completion port and nothing else: the bridge
   answers the `wf__sched_host_epoch`, `_park` and `_wake` seam, its park is
   `GetQueuedCompletionStatus` on that port, and a wake is a
   `PostQueuedCompletionStatus` packet posted once per announced sleeper, so a
   ready stack and an I/O completion arrive on one queue. A reaping thread
   that finds a wake packet puts it back while a sleeper is announced, because
   a posted packet is consumable where the Linux eventfd is a level fact.

   The record grew from 128 bytes to 160, and 160 is the smallest multiple of
   sixteen that holds it on every platform: 48 of those bytes are the typed
   request, which gains the `descriptor_class` a Windows open needs, and 40
   are the ring's own state, which on Windows is the record's `OVERLAPPED` and
   the handle the request was issued on. That is 32 bytes per outstanding
   operation per frame, and it is §12's per-frame record growth item.
   `contract.h` names no host threading API at all now: the wait set the epoch
   sleeps on is an opaque block the platform's own unit fills.

   The `completion-windows` job proves, on the real host: the shipped default
   helper policy and the shipped routing through the same
   `bridge_default_probe.c` the POSIX gate runs, once on the port
   (`route=native-ring`) and once with the port refused
   (`route=posix-adapter`); the ring alone, through `native_adapter_probe.c`;
   the namespace facilities; that the whole staged runtime compiles under the
   gate's warning set; that a bridge with no engine terminates before a submit
   returns; the three compiler boundaries that were already there; that a real
   `--par` program grants lanes, read through `sched/grant_observer.c`, which
   is now one file both platforms link for that proof; and that an overflow on
   a pool stack that is a fiber writes exactly `{"resource":"stack"}` and
   nothing else, at the default worker count and at four.

   Verified here, with the mingw-w64 cross compiler and wine: that every
   Windows unit compiles clean under a second compiler's `-Werror`, that the
   link imports the real fiber, port, SRWLOCK and reservation entries, that
   the IOCP ring carries a real positioned read found by the record's own
   address, and that the default-route probe passes on both routes. The real
   host settled the rest: `completion-windows` is green at `427b9aa` on every
   step, including the floor on a pool fiber and a `--par` program stealing
   across four workers. It took seven rounds, and each found something the
   proxies could not: the MSVC C runtime deprecates `getenv` under `-Werror`,
   so the platform layer gained one setting read (`prim.h` P4); the driver's
   include-closure test needs every header staged on every platform; the
   host runtime must not reach the bridge, since a program that submits
   nothing links no bridge; binding a handle to the completion port is one
   critical section under the descriptor table's lock, because every lane
   offers its first record on one descriptor at once and a handle cannot
   leave a port, and the file leaf sets the low bit of its overlapped event
   so its reads post no packet to that port; and on the MSVC target a
   destructor attribute is a C-runtime terminator that runs after the
   streams are torn down, so the grant observer registers its report
   through `atexit`. Every Windows run step now prints both channels and the
   exit status before judging, and every fail-stop in the Windows link set
   names its site. The POSIX numbers did not move: the same
   `--emit-llvm` module linked against the previous commit's runtime and
   against this one reads io-completion-bench `many_files_wide` at 0.1088 s
   and 0.1095 s best of seven, and `par_layout.wf` best of nine at W=1
   1.5235 s and 1.5232 s, W=2 0.8218 s and 0.8205 s, W=4 0.5789 s and
   0.5821 s, W=8 0.8464 s and 0.8348 s.

4. **Replay, the remaining measurements, docs, record** (design §11 item 24,
   §12): the enumerator's recorder replays a run's data and completion order;
   park cost and per-frame record growth measured; `LOOP-PIPELINE.md` §3.4
   and the roadmap's two stackless items edited in place; batch record.

   **Status 2026-09-05, slice 4a.** The measurements exist and nothing is
   chosen. Home: `research/experiments/park-on-miss-measurements/`, with the
   method, every table, the exact commands and the bars; the runner is its
   `run.sh`, wired to `make -C compiler park-on-miss-measurements`, outside
   `check`. Each alternative is a compile-time `-D` read by the C unit and
   carried into every gate build by `compiler/Makefile`'s
   `SCHED_VARIANT_DEFINES`, so `completion-test` and the §11 enumerator judge a
   form on the terms they judge the shipped one. Host: Linux, four cores,
   `cc` GCC 13.3 for the gate and clang 18 `-O2` for every measured binary;
   this host's own condition-variable park-and-wake is 16.2 µs and its
   hand-written stack switch 10.6 ns, against the 2.2 µs and 10.4 ns the tree
   quotes from another machine.

   | § | form | number | bar | met |
   |---|---|---|---|---|
   | 12.1 | park and resume at a compute miss, shipped | par_layout W=1 1526.64 ms, W=2 824.13, W=4 587.82, W=8 856.48 (medians of 9); grid W=1 1561.19, W=2 778.10, W=4 398.14, W=8 399.42 | within noise of 0.4067 s at W=4 and 0.4213 s at W=8 | **no**, by 45 percent at W=4 and 103 percent at W=8 |
   | 12.1 | nested runs of never-suspends jobs | not measured | — | variant rejected: it must assume every compute hand-out never suspends, which S23 falsifies, and the enumerator finds a live-lock at one thread |
   | 12.2 | one park and one publish, lock-free | 4399 ns best, 6231 ns median, N=15 | 2.2 µs | **no** at 2.2 µs; 0.27× this host's own 16.2 µs |
   | 12.2 | the locked form of §6 | not measured | — | variant rejected: §11's invariants encode the lock-free state machine the locked form deliberately collapses |
   | 12.3 | the claim protocol's price | 4399 ns against 4502 ns best, 6231 against 6556 median | — | not separated at this host's spread; the variant aborts on `par_layout` W=4 with the S3 fault |
   | 12.4 | the in-place wait of the idle window | shipped many_files_wide 112.06 ms default, 112.28 ms at `WF_IO_HELPERS=4` | — | second column absent: variant rejected on S23's coverage assertion |
   | 12.4 | the four-stage chain, 1000 files, 8 threads | nested 119.43 ms / depth 8; compensation 124.13 / 15; stack switch 438.36 / 8; pipeline K=32 119.68 / 8 | park on miss not slower than the pipeline, and beats nested helping | **no**, and the run cannot test the bar: this host's reads do not wait enough for any shape to reach its own bound, and the switch shape sleeps on `prim_host.c`'s broadcast fallback rather than the bridge's ring park |
   | 12.5 | the memory orders | weak orders 3119 ns best, 4394 median against 4399 and 6231 | admitted only with a GenMC run | **worse than that**: the form passes the enumerator at all four configurations and then hangs `par_layout` deterministically at W≥2, which is §6's step-2/step-3 store-then-load pair on both sides |
   | 12.5 | the stack count at which the pool stops refusing | par_layout 12 at W=4, 20 at W=8; grid 8 at W=4, 16 at W=8; default is threads+8 | — | a refusal costs no measurable wall time (par_layout W=8: 827 ms with 274 317 refusals against 850 ms with none) |
   | 12.6 | the lane slot count | 2, 4, 8, 16 against the shipped 64, at W=4 and 8 | — | not separated between 4 and 64 on either program; 2 slots costs 12 percent on the grid at W=8 |
   | 12.6 | the per-thread ready list | not measured | — | variant rejected: it adds a shared word the enumerator does not model |
   | 12.6 | record memory per frame | 128 to 160 bytes: +128 completion_read_boundary, +96 wfgrep, +32 byte_string, dir_walk, par_layout, 0 elsewhere | — | exactly 32 bytes per outstanding operation a frame holds, and the deepest chain bound grows by the same |
   | 12.7 | the chain bound per hand-out entry | nineteen entries across nine programs, every one ≤ 80 bytes | — | today's corpus needs one stack class |

   The result that shapes the rest: five of the six behavioural switches
   cannot be measured. Four are rejected by the §11 enumerator, and the
   rejections are three different kinds — an invariant the form breaks
   (`WF_SCHED_NO_CLAIM`, and `WF_SCHED_NESTED_NEVER_SUSPENDS`'s live-lock), a
   state machine the enumerator does not have (`WF_SCHED_LOCKED_PARK`, whose
   two failures are exactly the two edges §6 says the locked fallback
   collapses), and a coverage assertion or an unmodelled word the form cannot
   meet (`WF_SCHED_PARK_AT_ONCE`, `WF_SCHED_THREAD_READY`). The fifth,
   `WF_SCHED_WEAK_ORDERS`, passes every gate and hangs a corpus program. So
   §12 items 1, 2 and 6's ready list are not answerable by measurement in
   their current state, and what each needs is named in the bundle: the
   target-action bit at the hand-out for item 1, a second invariant set in
   `enumerate.c` for the locked state machine, and a model of one word for the
   per-thread list. `tests/programs` holds no grid loop-split program —
   `mandelbrot_grid.wf` is a `loop` and not a counted `for`, so [PAR-2] grants
   it nothing — so the bundle supplies `programs/grid_split.wf` for the
   comparison item 1 asks for.

   **Status 2026-09-05, slice 4b. Slice 4 is closed except design §12 item 1.**
   The record is [`docs/done/0107-park-on-miss.md`](done/0107-park-on-miss.md).
   What closed here:

   - *The variants the measurement could not separate are deleted.* Under the
     rule below, `WF_SCHED_NESTED_NEVER_SUSPENDS`, `WF_SCHED_LOCKED_PARK`,
     `WF_SCHED_NO_CLAIM`, `WF_SCHED_PARK_AT_ONCE`, `WF_SCHED_WEAK_ORDERS` and
     `WF_SCHED_THREAD_READY` leave `core.c`, `core.h` and `bridge.c` with every
     `#if` and every field they added, and `compiler/Makefile` loses
     `SCHED_VARIANT_DEFINES`, `SCHED_ENUMERATE_VARIANT_DEFINES` and
     `COMPLETION_ENUMERATE_CFLAGS`; the shipped form is the only text again.
     `WF_SCHED_LANE_SLOTS` stays the `#if !defined` override of `core.h` it was
     before slice 4a, which the enumerator pins to 2. The experiment keeps
     every table as the record of 2026-09-05, each retired section opening with
     one sentence on why its form went, and `run.sh` keeps the lines it can
     still reproduce.
   - *Design §11 item 24, the recorder and the replay.* The enumerator records
     one walk of every schedule it sweeps — per step the process that stepped,
     and at a device step which record completed, in what order, and with what
     result head the stub delivered — and replays it from a fresh core, feeding
     the recorded picks and heads back and comparing the whole ordered sequence
     of the core's own transitions (every stack phase change and every lane
     free-list pop and push, each with the thread that made it), the statistics
     at the end, and the transition count. A fed head is checked at the join,
     so the wrong datum fails there. The pair runs before the search on every
     schedule and is reported in the `enumerate:` line as `replay_steps`,
     `replay_completions` and `replay_transitions`. Two perturbations of a
     scratch copy show the check is not vacuous: a head one greater than the
     stub's fails at the join, and a hidden static in `wf_sched_take_target`
     that alternates its preference across executions fails at the pick.
   - *One gate repaired.* `repository-invariants` had been red since slice 4a,
     because the experiment's `run.sh` named a home directory in its `ROOT`
     default; `ROOT` is derived from the script's own location now.
   - *Docs in place.* `LOOP-PIPELINE.md` §3.4 corrected for what the shipped
     emitter does now; `docs/roadmap.md`'s two stackless items and its Windows
     line rewritten to name what replaced them; and
     `docs/ongoing/HANDOFF-2026-09-04.md` deleted, its removal condition met,
     with what was still live in it folded into this file.

   **Open, and the owner's to decide: design §12 item 1, the compute-miss
   regression.** `par_layout` is 45 percent slower at four workers and 103
   percent at eight than before park on miss, and the bar is "within noise".
   The design's stated fallback is nested runs of never-suspends jobs at a
   miss, which needs the target-action bit at the hand-out; today's emitter
   marks none, and the variant built for slice 4a had to assume every compute
   hand-out never suspends, which S23 falsifies — it is a live-lock at one
   thread. So the fallback is not measurable until the emitter marks hand-outs,
   nothing here chooses between the two, and the grid loop-split program says
   the regression is the cost of a miss and not of the scheduler: it pays 4 ms
   of system time at four and eight workers against `par_layout`'s 239 and
   702 ms, because its loop split hands out large chunks and almost never
   misses.

   The owner's reading of it, 2026-09-05: the fallback is the colouring
   question (a function that may suspend against one that never does), and
   if that is what recovers the performance it is accepted, with the cost
   that the language's standard puts such a colour on the declaration; it is
   a larger change and needs research, and it is sequenced after the first
   API that waits. One fact for that research, from the specification as it
   stands: [FN-1] already gives every concrete function a compiler-derived
   target summary, `never-suspends` or `may-suspend`, "derived from exact
   system contracts and the finite concrete call graph, never written,
   inferred from a spelling, or weakened by a declaration", and [EFF-3]
   already consumes it. So the colour exists today as a derived fact the
   declaration must not carry, and the fallback's target-action bit would be
   that summary read at the hand-out, not a new spelling; what the research
   has to settle is whether a bit that is never written is enough for the
   writer to predict which joins park, and what the enumerator needs to
   check a miss that nests a never-suspends job under a may-suspend one.

   **Status 2026-09-05, the idle spin.** The Windows qualification bench now
   runs to its bar on the real host and fails two of them: `mixed-full` 1.0593
   and `mixed-total` 1.0578 against 0.95, with `compute` 0.3202 against 0.90
   and `io-warm` 0.9595 against 1.10 both met and `grants=1024`. What the two
   failing cohorts pay is wake latency: per iteration of
   `windows_runtime_mixed.wf` an idle worker has to be woken to steal the
   hand-out and the publisher has to be woken again when the stolen job makes
   its joiner's stack READY, and on a four-vCPU Windows VM each of those is a
   completion-port round trip. The retired runtime found both by spinning
   (`WF_PAR_SPIN_ROUNDS` 4096, `WF_PAR_YIELD_ROUNDS` 16) before it slept; the
   core parks the moment its last look misses.

   So `wf_sched_idle_step` now repeats its own looks before it parks, for
   `WF_SCHED_IDLE_SPIN_ROUNDS` rounds of a new `wf_prim_pause` and then
   `WF_SCHED_IDLE_YIELD_ROUNDS` rounds of `wf_prim_yield` (`sched/core.h`,
   `sched/prim.h`). **Where the spin sits is most of the result.** In front of
   the drain it is a disaster, because the drain is the only thing that
   delivers an I/O completion: sixteen rounds there cost `many_files_wide` 19
   percent and four thousand cost it forty-one times. After the drain and after
   the window's own last look, immediately before the park, the I/O line is
   flat across the whole sweep — a turn with work to find has already found it
   and never reaches the spin. The epoch capture does not move, so §6's
   lost-wake argument is untouched.

   The sweep is
   [`research/experiments/park-on-miss-measurements/README.md`](../research/experiments/park-on-miss-measurements/README.md)'s
   "§12 addendum: the idle spin" and `run.sh`'s new `spin` section: pause
   rounds 0, 16, 64, 256, 1024, 4096, 16384 against yield rounds 0, 16, 64, on
   the mixed program's three builds, `par_layout` and the grid at four worker
   counts, `many_files_wide`, and the park and publish round trip, with CPU
   beside wall on every line. The chosen constants against the shipped form, at
   N = 15 on this four-core Linux host:

   | line | `spin-0-0` | `spin-256-16` (chosen) | `spin-4096-16` |
   |---|---|---|---|
   | mixed, completion-only build | 256.53 ms | 252.63 | 244.39 |
   | mixed, unified `--par` build | 170.68 ms | 160.91 | 154.10 |
   | mixed, unified ÷ completion-only | 0.665 | 0.637 | 0.631 |
   | mixed unified, user CPU | 254.97 ms | 410.39 | 457.40 |
   | par_layout W=4 | 562.75 ms | 451.52 | 453.60 |
   | par_layout W=8 | 802.02 ms | 492.70 | 566.55 |
   | par_layout W=8, system CPU | 625.75 ms | 229.49 | 356.71 |
   | grid W=8 | 389.34 ms | 390.36 | 390.60 |
   | many_files_wide, best of 7 | 114.86 ms | 112.94 | 118.79 |
   | park and publish, settled median | 6320 ns | 4107 | 3662 |

   **256 and 16 chosen, by the numbers.** A look round costs about 43 ns here,
   so 256 of them is a window of about 11 µs against this host's own
   park-and-wake of 16.2 — the retired runtime's own floor (do not sleep to
   save less than the sleep costs) applied to the machine that was measured
   rather than to the 2.2 µs machine its comment was written on. Above it the
   sweep buys a further 4 percent on the mixed program and pays for it on
   `par_layout` at eight workers on four cores, where a pause-spin takes a core
   from a thread that had work, and in idle CPU. Nothing regresses at the
   chosen point: the two I/O-only mixed builds, the grid and `many_files_wide`
   are inside their own spread at every form of the grid.

   It also moves design §12 item 1, the open compute-miss regression: against
   the 406.7 ms and 421.3 ms the tree read before park on miss, `par_layout`'s
   45 percent at four workers and 103 at eight become 11 and 17. That is not an
   answer to the item — the fallback it asks for is still the colouring
   question above — but it is most of the gap, taken without one.

   **The real judge is the Windows job**, because every number here is a
   Linux VM whose park-and-wake is 16.2 µs and the failing bar is a Windows VM
   whose wake is a completion-port round trip, which is the case the spin
   exists for and the case this host cannot produce. Both constants are
   `#if !defined` overrides so the bench can sweep them where the bar is, and
   `sched-enumerate` pins them to one round and no yields — a spin round is a
   step there, not a delay, and the §11 search costs 9.4 times the states at
   (T=2,S=4) with one round and 24 times at (T=2,S=3) with two.

   **The judge ran on 6311482.** `bench-windows-qualified` met every bar:
   mixed-full 0.6920 and mixed-total 0.6878 against 0.95, with p10..p90 under
   0.70, io-warm 0.9502, compute 0.2820, `grants=1023`. That is the ratio this
   host gives. The run just before the spin, 2d455e5 on another runner, also
   met the bars without it at 0.86, with the completion-only reference itself
   12 percent slower than in the 1.06 record and in the spin's run, so the
   Windows VM moves between runners by more than the bar's margin; the spin's
   evidence is its 0.69 beside the 0.86 and 1.06 of the runs without it. The
   addendum's closing section holds both tables.

### Decided 2026-09-05: measured before chosen

The owner ruled that the locked form and the lock-free form of §6 are both
built in slice 3 over the one `prim.h`, and §12 chooses between them by
measurement; and that every other choice with a plausible cost is measured
the same way rather than argued. The choices the enumeration added to §12's
list, each measured as one park and one publish against the 2.2 microsecond
park-and-wake figure, and on the four-stage chain:

- the claim protocol: the compare-exchange on `record->waiter` that every
  completion with a registered waiter now pays, and the COMPLETING store
  before the DONE store that every completion pays;
- the in-place wait of the idle window against parking at once;
- the sequentially consistent orders the core uses today at the record and
  the phase, against the acquire and release orders the enumerator cannot
  tell apart (its model is sequentially consistent, so a weaker order is
  admitted only with the GenMC run the deferred note below names);
- the lane slot count and the ready list under the one mutex, against a
  per-thread ready list.

A choice the measurement cannot separate keeps the form the enumerator
checked.

Batch 2 is done with the surface the language has today: read-only files,
directory enumeration, and the two standard outputs. That surface reaches
every state of the scheduler through the enumeration harness and injected
stubs, and it cannot exercise a real wait: a cached read does not wait, and a
cold read waits briefly and uniformly.

## Batch 3: streams and TCP (specification v0.46)

**Status 2026-09-06, peer-bound requests are a helper's: landed on this
branch.** On the shared file adapter — every socket on Darwin and Windows
accept, and `WF_IO_NO_NATIVE_RING` on Linux — a request whose kind waits on a
peer (accept, receive, connect, send) now grows a helper on submission
whatever the measured verdict says, is skipped by a scheduler thread's
progress pass and by a pool stack's claim of its own record while a helper
exists, and is left out of the execution average; the pinned zero-helper
policy is unchanged, because there the waiting thread is the queue's only
engine. Before it, three workers of `tcp_fanout.wf` each sat inside a receive
from a silent peer with no thread left to accept the fourth connection, which
failed `four_peers_are_served_at_once_under_par_on_both_routes` on a
three-core runner; that case now pins `WF_WORKERS=3` so the property does not
depend on the host's core count. Adapter-route socket concurrency is therefore
`WF_BRIDGE_MAX_HELPERS`, and the readiness-driven adapter that would remove
that bound is the open design item recorded in
`research/investigations/io-model/NETWORK.md` §5.

**Status 2026-09-06, the staged hand-out: landed on this branch.** A staged
step whose call is a may-suspend *user* call is now handed to a compute lane,
so the fixed-trip accept loop of `tcp_fanout.wf` keeps four accepts in flight
at once in a `--par` build and four peers that connect before any of them
speaks are answered in the order they speak
(`four_peers_are_served_at_once_under_par_on_both_routes`). At the staged
point the emitter acquires a lane frame sized for `{ arguments..., result }`,
stores the arguments and publishes it with the same thunk the [PAR-1] compute
hand-out uses, so the callee runs on a pool stack and parks on its own I/O
without holding the loop; the frame's address is what the pipeline slot holds
for that iteration, exactly as a record's address is what it holds for a
system operation, and the exact drain joins the frame, reads the result out of
it, releases it, and runs the remainder in iteration order. A refused
acquisition runs the same call on the same operands where it is written and
leaves its answer in the same ring element, which is the permitted sequential
form and changes no observable; the sequential clone and the `--no-overlap`
build take exactly that edge and name no lane entry at all. The lowering side
is `lowering/builder/loops.rs`: the bounded-batch recognizer admits a staged
tail written as `let reported = call(...)` followed by the statements after
it, and the iteration's own compiler-derived releases ride the ring so a
per-iteration `buffer_new` is released in the drain, after the join, on the
value that iteration allocated. The window is the runtime's own answer through
`wf__completion_window` under a compiler ceiling of `WF_SCHED_LANE_SLOTS`,
because every iteration in flight holds one frame slot of the offering
thread's lane; the fanout's trip count of four is the binding term, so it gets
K = 4. Above that the bound is the stack pool — a parked callee holds a pool
stack, so `WF_STACKS`, defaulting to the thread count plus eight and capped by
`WF_SCHED_MAX_STACKS`, is how many can be parked at once, and a program past
it waits on PARK-ON-MISS.md §2's fourth line rather than failing.

**Status 2026-09-06, slice 3: landed on this branch.** Every TCP operation
runs on Windows, on both routes. `backend/qualification.rs` maps ordinals 22
through 28 on the Windows column to the same ABI symbols the native column
uses, because an operation's symbol is target-independent and which engine
runs it is a runtime routing fact; the emitter is unchanged.
`completion/file_windows.c` executes all six request kinds against Winsock
(`WSASocketW` with `WSA_FLAG_OVERLAPPED` and `WSA_FLAG_NO_HANDLE_INHERIT`,
`bind`, `listen(SOMAXCONN)`, `accept`, `connect`, `recv`, `send`, `shutdown`,
`closesocket`), and `completion/windows_iocp.c` carries the connect, the
receive and the send on the completion port with `ConnectEx`, `WSARecv` and
`WSASend` on the record's own `OVERLAPPED`, the connect's socket created and
bound to its family's wildcard in the submitting call and
`SO_UPDATE_CONNECT_CONTEXT` set on the completion path. `backend/
windows_runtime.c` holds the once-per-process `WSAStartup`, the socket open
and close, the ledger's new `WF_WINDOWS_DESCRIPTOR_CLASS_SOCKET` row, and the
normalization that makes a `WSAGetLastError` code and the completion port's
own Win32 code for one condition answer one [SYS-7] class. The address
vocabulary is shared rather than twinned: it moved out of `file_posix.h` into
`completion/socket_address.h`, which every engine on every platform includes.

The accept stays on the shared file adapter on this platform, by measurement
rather than preference: an `AcceptEx` address pair is
`2 * (sizeof(sockaddr_in6) + 16)` = 88 bytes that must live until the
operation completes, and the completion record is exactly 160 bytes with the
accept's union arm at the 40-byte ceiling `contract.h` asserts and the
ring-state block at 40 bytes already full of the `OVERLAPPED` and its handle.
The record may not grow and the runtime allocates nothing at run time, so
there is nowhere for those 88 bytes to go; refusing the accept on the port is
the same class of fact as the Linux ring's refusal of a listen.

`bridge_default_probe.c` gains a loopback round trip through the bridge's own
ABI — listen, connect, accept, send, receive and the four releases — which is
one text on every platform and is what `completion-windows-wine` and the POSIX
`completion-default-route-test` both run; `.github/workflows/io-hosts.yml`'s
`completion-windows` job gains a step that compiles `tcp_echo.wf` (plain and
`--par`) and `tcp_refused.wf` with the production driver and runs each on the
completion port and on the adapter against a `System.Net.Sockets` peer.
Winsock is the one import library a Windows link now names.

**Status 2026-09-05, slice 2: landed on this branch.** Every TCP operation
lowers and runs on POSIX. `backend/qualification.rs` maps ordinals 22 through
28 on the native column and leaves them unmapped on the Windows one, whose
completion-port route is slice 3, so a Windows submission is still refused at
qualification rather than at run time. The emitter has the seven wrappers in
the same submit-then-join shape as the opens and the reads —
`@wf.sys.tcp_listen.v1`, `@wf.sys.tcp_accept.v1`, `@wf.sys.tcp_connect.v1`,
`@wf.sys.receive_next.v1`, `@wf.sys.send_once.v1`,
`@wf.sys.close_connection.v1` and `@wf.sys.close_listener.v1` — with the three
outcome enums built exactly as `FileOpenOutcome` is. `wf_file_request` gains
six kinds (listen, accept, connect, receive, send, half-close) inside the
budget the record already had: the accept's peer record lives in the union
arm the accept alone uses, because twenty-four more bytes on the shared result
head would put the record past the 160-byte block an emitted frame reserves.
`file_posix.c` executes every kind, `linux_io_uring.c` carries accept,
connect, receive and send on the ring, and listen, bind and the half-close stay
on the adapter. `file_adapter.c` keeps the connection pair's own two-count, so
the second release of a pair is the one that closes the target's object and
spends the credit. `tests/programs/` gains `tcp_echo.wf`, `tcp_client.wf`,
`tcp_fanout.wf` and `tcp_refused.wf`, each run on both routes against a
`std::net` peer, and the five `systcp-*` conformance cases that expected
`unsupported` now expect `accept` — the subject moved from unsupported to
supported and no expectation about the language changed.

What the slice left open was the lowering rather than the program shape or the
judgment, and the staged hand-out below closes it. The one shape [PAR-3]
itself does not stage remains what it was: a loop whose stop condition is data
a remainder produced.

**Status 2026-09-05, slice 1: landed on this branch.** The amendment and
everything derived from it are in the tree: `spec/kernel-spec.md` declares
`Status: ACTIVE v0.46` over v0.45's text, the v0.45 bytes are archived, the
`ACTIVE-SPEC:` record is appended in `governance/APPROVALS.md`, the derivation
ledger carries the v0.46 section with rows for the four added rules, and
`compiler/src/spec_identity.rs` is regenerated. The design is
`research/investigations/io-model/NETWORK.md`, whose §8 records the owner's
decisions of 2026-09-05; the ruling it implements is constitution T4 applied to
every socket resource that document's §2 enumerates.

The language delta: `Output`, `FileFactory`, `FilePermit`, `command.files` and
`reserve_file` are respelled `OutputStream`, `HandleFactory`, `HandlePermit`,
`command.handles` and `reserve_handle`, because a listener and a connection
each draw one credit from the same capacity a file open draws from. [SYS-15]
adds `InputStream`, supplied at entry ordinal 5 as `command.stdin` and read by
`read_next`. [SYS-16] adds `SocketAddress` with two total pure constructors.
[SYS-17] adds `TcpListener` with `tcp_listen`, `tcp_accept`, `tcp_connect` and
`close_listener`. [SYS-18] adds `TcpConnection`, the first system-declared
struct, whose `receive: TcpReceive` and `send: TcpSend` fields are ordinary
places: two `&uniq` loans on disjoint fields coexist under [OWN-5], a partial
move kills the whole binding under [OWN-1], and no `split` or `join` exists.

### Slices (NETWORK.md §7)

1. **Amendment v0.46: the two renames, `command.stdin` and `read_next`, the
   types and operations of §4, conformance cases, corpus programs.** Done.
   `read_next` is end to end on POSIX: the runtime's existing unpositioned
   stream-read kind, routed to the io_uring ring as a read at offset -1 and to
   the shared file adapter's own `read` otherwise, with the Windows leaf
   reading a console or a redirected handle through one `ReadFile` at the
   handle's own position. `tests/programs/stdin_echo.wf` echoes its standard
   input on both shapes and both routes. The TCP operations are declared,
   checked, lowered and emitted, and refused at target qualification, because
   their runtime routes are slice 2.
2. **POSIX runtime: adapter route for every TCP kind, Linux ring route for
   accept, connect, receive and send; loopback tests in `tests/programs`.**
   Done. This is where a wait becomes a real ring wait for the first time.
3. **Windows: the completion-port route; the io-hosts job proves it.** Done.
   The port carries the connect, the receive and the send; the accept is the
   adapter's for the record-size reason the status above states.
4. **The control benchmark** against the io_uring and epoll references, in
   `io-completion-bench`, reported as a ratio to the io_uring reference.
5. **Batch record.**

### What slice 1 revealed

- **A latent inventory-decoding defect.** Lowering decoded a
  `CheckedConstructor::System` ordinal against `Inventory::ACTIVE` rather than
  against the inventory the unit was resolved under. The two agreed while
  every inventory state had the same nominal-record block, and stopped
  agreeing the moment a system struct added field records ahead of the
  constructor block. The checked program now carries its inventory and lowering
  reads that. The prefix-differential property is what caught it.
- **A field borrow of a system resource was unsupported.** `&uniq
  connection.receive` is the whole point of the two-field connection, and no
  checked borrow form carried a field path for a non-buffer type.
  `BorrowSystemResource` now carries one, and lowering projects the field
  without consuming the root. The loan machinery needed nothing: the borrow's
  own `place` already named the field, so [OWN-5] decides two loans on disjoint
  fields exactly as it does for a source struct.
- **A read into a buffer kills nothing the publish needs, if the publish is a
  helper.** `write_once` over the buffer a read filled needs
  `available <= len(chunk)`, and both halves of that are live only immediately
  after the read. A helper whose contract states the bound moves the obligation
  to that point; a second inner loop loses it. This is P22 in
  `docs/patterns.md`.

## After batch 2: the order the owner agreed on 2026-09-04

The surface is validated first and widened one API at a time, and every new
API passes the same T4 question the permit passed: the resource is a value in
the signature, and every dependency between it and an existing resource is
one the checker can see.

3. **The first API that waits: the network, with `command.stdin` as the
   first stream.** Revised by the owner on 2026-09-05: concurrency's real
   demand is the network, so the first waiting API is TCP over a loopback
   rather than standard input alone; a loopback is a controlled peer, which
   makes the control test easy. Standard input comes along as the first
   instance of the same stream design, and the standard streams are renamed
   `InputStream` and `OutputStream`. The proposal, the T4 accounting of
   every socket resource, the operations, the runtime routes, the control
   test and the owner decisions it needs are
   `research/investigations/io-model/NETWORK.md`; the compute-miss colouring
   question stays sequenced after it.
4. **File write and create.** The threshold for writing real programs (the
   compiler itself has to write files) and the second examination of the
   resource accounting: write handles, the namespace effect of a create, and
   their dependencies on directory reads, all expressed on the API the way the
   permit is.
5. Only then the rest of the roadmap's list: clock, timers, network,
   cancellation, namespace mutation.

### Recorded 2026-09-05, deferred until batch 2 is done: prove the core

The owner asked whether the scheduler core can be proved rather than only
enumerated, and decided to study it after batch 2 rather than now. The idea to
study, as discussed:

- Rewrite the core in Whitefoot. The stacks, records, lanes, ready list and
  free list become indices into fixed arrays instead of pointers, so every
  access is a bounds-checked operation the language already proves.
- Keep the minimal primitives in C with contracts: the atomics, the stack
  switch, sleep and wake, the reservation, the mutex, the yield. Their
  `requires`/`ensures` are the trust base, and the enumerator's replacement
  `prim.h` stays the executable check of the same contracts.
- Prefer the locked form (`PARK-ON-MISS.md` §6 fallback) for the proved
  version: under the one mutex the shared state is a sequential value, so the
  §11 invariants become monitor invariants that Whitefoot's sequential proofs
  can carry across every entry and exit. The lock-free deque and the claim
  protocol stay the measured alternative, not the proved one, unless §12 shows
  the locked form loses.
- Intermediate steps if the rewrite is too far: a GenMC or CDSChecker run of
  the C core for the weak-memory question the enumerator's sequentially
  consistent model does not ask, and a TLA+ model of the claim and park
  protocols checked by TLC.

Nothing in this note is a batch 2 requirement.
