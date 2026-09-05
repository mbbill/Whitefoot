# Current Plan: back the file permit, then park on miss

Status: IN PROGRESS on `io/t4-resource-relations` (PR #13). The previous
plan, source-carried proof, is IMPLEMENTED AND ACTIVATED as v0.40 and is
recorded in `docs/done/` and the v0.41 activation.

The active language authority is the specification at `spec/kernel-spec.md`;
its version and digest are the chain tail in `governance/APPROVALS.md`. This
branch carries the backed-permit amendment as one change: the amended file,
the archive of the outgoing bytes, the appended approval record, and the
regenerated identity module, so the branch is merge-ready the moment its gate
is green and the owner's merge approval of that exact revision is the
activation. Nothing merges to `main` until the owner approves the exact
revision and canonical `make check` passes on that revision. This document
records technical direction and sequencing; it grants no permission and adds
no workflow gate.

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
   The port is slice 3's Windows twins bullet and stays there. The emitter
   half of (c), the one lowering, is next.
3. **Runtime** (design §7): the core replaces both parallel runtimes and both
   writer schedulers; the bridge, adapters and rings find the record by
   address and lose their pools, capacity waits and tokens; the I/O joins park
   the stack; the floor's entry runs `wf__main_body` on a pool stack selected
   at link time; the Windows twins in `wf_floor_windows.c` and
   `windows_bridge.c`.
4. **Replay, the remaining measurements, docs, record** (design §11 item 24,
   §12): the enumerator's recorder replays a run's data and completion order;
   park cost and per-frame record growth measured; `LOOP-PIPELINE.md` §3.4
   and the roadmap's two stackless items edited in place; batch record.

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

## After batch 2: the order the owner agreed on 2026-09-04

The surface is validated first and widened one API at a time, and every new
API passes the same T4 question the permit passed: the resource is a value in
the signature, and every dependency between it and an existing resource is
one the checker can see.

3. **The first API that waits: `command.stdin` read.** One operation, one
   resource, the same completion path. It is the first workload on which the
   park-on-miss scheduler faces a real wait, so the first one whose numbers
   say anything about the scheduler. Pipes and sockets are its kin and follow.
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
