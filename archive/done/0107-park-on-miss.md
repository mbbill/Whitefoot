# Batch 0107 — park on miss (the scheduler core, the frame's record, one runtime)

Branch: `io/t4-resource-relations` (PR #13), continuing from batch 0106 on the
same branch. Deliverables: the scheduler core and the enumeration that gates
it, the completion record as a block of the submitting frame, one lowering for
every I/O operation, one runtime for both platforms, the §12 measurements, the
enumerator's recorder and replay, and this record. The plan is
`docs/current-plan.md`, Batch 2; the design is
`research/investigations/io-model/PARK-ON-MISS.md`, whose section numbers this
record uses.

## Why

Before this batch a Whitefoot program that overlapped I/O bought its overlap
from two different machines. Compute hand-outs ran on `par_runtime.c`'s worker
pool; completions ran on `completion/runtime.c`'s slot pool, reached by a token
into 256 slots, with a capacity wait when the pool was full, a refusal path
back to a blocking direct call, a writer scheduler beside it, and a stackless
continuation lowering in the emitter for the one shape it could transform. Each
of those was a bound the source could exceed and a second answer to the same
question. §3's principle names the observable that ties them together: no
completed operation's continuation may be buried behind something that is
waiting. The design's answer is one scheduler for both kinds of hand-out, a
join that parks its own stack when it misses, and a completion record that
lives in the submitting frame so there is no pool to size, refuse from, or
index.

## 1. The scheduler core and its enumerator (slice 1, `1b1047a`)

`compiler/src/backend/sched/` is the core (§5–§7.1): `core.h` and `core.c`, the
protocol; `prim.h`, the seven primitives it reaches shared state through, with
two implementations; `prim_host.c`, the host's; `switch.h`, the one stack
switch; `smoke.c`, the core on real threads under `make -C compiler
sched-smoke`.

The gate is not that a scheduler test passes; it is that the core is enumerable
and the enumeration passes (§11). `enumerate.c`, `enumerate.h` and
`schedules.c` compile `core.c` against a replacement `prim.h`: one OS thread,
the controller on the process stack, T simulated threads as coroutines each
calling `wf_sched_run`, a device process per submitted record, every primitive
announcing its step and performing it when chosen, the lock and the park and a
yield as wait states, and every §11 invariant checked after every step rather
than once at the end. The gate's search is explicit-state — every scheduling
point checkpointed, digested and never explored twice, an ample step taken
alone, a completion explored only where a thread can observe it — and the
re-executing walk of every interleaving is the reference at one thread, where
the two must reach the same arms and the cargo wrapper asserts they do. The
sweep is (T=1,S=2), (T=1,S=3), (T=2,S=3), (T=2,S=4), which §11 derives from
§5's floor S ≥ T + 1 and from the two schedules that need two stacks parked
while two threads run. Twenty-one of §10's schedules are scenarios in
`schedules.c`; S8, S10b and S11 are not enumerable and are absent.

The enumeration found six defects in the core, each fixed in place and each
recorded in `docs/current-plan.md` with the schedule and configuration that
reached it: the entry thread's last pool stack was unresumable; a late
publisher acted on a cancelled park (the claim protocol is the fix); the
publisher touched a dead record (PENDING, COMPLETING, DONE is the fix); the
in-place waiter slept past another thread's drain; the compute arm never
drained; and a worker start found no stack. Two of the seven primitives changed
for the search — the idle bitmap's read-modify-writes and the lock's named
section — and both stayed, because the first is the simpler core and the second
costs the host nothing.

## 2. Emitter (slice 2, `bccf181`, `89688f9`, `effe2e2`)

Design §8, landed in the three steps the plan sequenced after reading the
bridge, because the one lowering is sound only once the runtime accepts every
submission.

- `compute_join_order` (`emitter/parallel.rs`) is the one place a group's join
  order is decided (§4): compute members newest first, because the deque is
  Chase-Lev and the newest entry is the one the owner can reach; completion
  members exactly where they were published, because they hold no deque entry.
  `emit_overlap_joins`, `overlap_join_tail` and `block_exit_label` consume it.
- The completion record is an opaque block of the submitting frame, its size
  and alignment an ABI constant (`WF_COMPLETION_RECORD_BYTES`,
  `WF_COMPLETION_RECORD_ALIGN` in `completion/contract.h`) asserted by the C
  units that store a record and by a Rust test that parses the header. The
  stackless continuation lowering went with it: `emitter/stackless.rs`,
  `tests/stackless.rs`, the writer-frame submit ABI and the
  `WF_PAR_WITH_WRITER_SCHEDULER` runtime variant are deleted.
- Every ordinary I/O wrapper in `emitter/system.rs` reserves its record in its
  own `alwaysinline` frame, submits and joins; derived release closes through
  one shared `wf.sys.close` helper that does the same. The direct family, the
  qualification rows that named it and every `declare` of a `_direct` symbol
  are gone, and `module_requires_completion_runtime` is now "the module calls a
  submit".

Two things the one lowering exposed and this slice answered: the
`open_file`/`open_directory` kind check left the wrapper for the runtime that
already decides it from `expected_kind`, so no emitted code holds a `struct
stat`; and a thread joining its own submission runs that record itself when it
is still queued (`wf_file_adapter_claim_own`), because with helpers pinned an
ordinary write would otherwise wait behind an unrelated blocked one.

## 3. Runtime (slice 3, `3acc3e9`, `9051576`, `92b19e1`, `babf5c7`)

Design §7, in four steps.

(i) **Records by address.** `wf_completion_record` is the frame's block,
`contract.h` is its header, and the slot pool, tokens, claims, milestones,
drains, consumes, dependent frames and every capacity wait are deleted from the
core, the bridge, the file adapter (whose queue is now an intrusive FIFO
through the records) and the io_uring adapter (whose `user_data` is the
record's own address). Every submit ends in `wf_completion_record_complete`,
which is `wf_sched_complete` on the record.

(ii) **A mixed overlap group hands out both its kinds** (`9051576`). The gate
in `FunctionEmitter::new` dropped any group holding a completion member, so
`overlap_handed_out` was empty for every mixed run. Two fixtures pin the fix at
every worker count.

(iii) **The core becomes the POSIX runtime.** `sched/entry.c` and `entry.h` are
the platform layer over `core.c`: the process's one `wf__sched_core`, the
startup policy `par_runtime.c` had, the worker threads with their reserved host
stacks, and the `wf__par_*` module ABI as thin functions over the core. The
floor runs `wf__main_body` on a pool stack when the core is linked. `par_runtime.c`,
`completion/writer_scheduler.c` and its header are deleted.

(iv) **Windows takes the same runtime**, as shared code rather than a second
copy. What is shared is every piece of logic — `sched/core.c`, `sched/entry.c`,
`completion/runtime.c`, `file_adapter.c`, `bridge.c`, `contract.h`. What stays
a platform leaf is only what calls the host: `sched/prim_windows.c`,
`completion/windows_iocp.c`, `file_windows.c`, `wait_windows.c` and
`wf_floor_windows.c`. Deleted: `par_runtime_windows.c`,
`writer_scheduler_windows.c`, `windows_completion.c` and its header,
`windows_bridge.c`, `windows_blocking.c`, three probes,
`native_completion_api.h`, and `tests/programs/completion_windows_capacity.wf`,
which exercised a slot capacity that no longer exists. One rule now covers
`WF_WORKERS`, `WF_STACKS` and `WF_IO_HELPERS` on both platforms: unset is the
caller's default, an integer within the ceiling is that number, anything else
ends the run before the program body with one diagnostic line and a nonzero
status. The record grew from 128 bytes to 160, which is 32 bytes per
outstanding operation a frame holds.

## 4. The two defects the runtime step exposed

**A lost wake, and with no timeout in this design, a hang.** The bridge answers
the core's `wf__sched_host_epoch` / `_park` / `_wake` seam so that one wake
rings the ring's eventfd and the condition variable alike. That seam answered
"not mine" until the whole bridge had initialized, so a thread that parked
before the program's first operation slept on `prim_host.c`'s own condition
variable while every later wake went to the bridge's. Nothing would ever wake
it. The wake epoch now has its own `pthread_once`, taken by whichever of a seam
call or the bridge's initializer arrives first, so the seam has one owner from
the first park. It is the shape of defect this design invites and the reason it
has no timeout to hide one: a lost wake is a hang, and a hang is what a gate
can see.

**A ring destroyed under a sleeping worker.** The bridge's `atexit` shutdown
destroyed the ring at exit. With the scheduler loop being the thing that
sleeps, a detached worker may be asleep inside that ring when the entry thread
reaches exit, and the teardown pulled the ring out from under it. The shutdown
now returns without destroying the ring while the pool is still running. Both
defects are consequences of the same change — before this batch the thing that
slept was a bridge-owned waiter, and now it is the scheduler loop, which
outlives the frame that started it.

## 5. The seven Windows rounds

Step (iv) was verified here with the mingw-w64 cross compiler and wine — every
Windows unit clean under a second compiler's `-Werror`, the link importing the
real fiber, port, SRWLOCK and reservation entries, the IOCP ring carrying a
positioned read found by the record's own address, the default-route probe
passing on both routes — and the real host in `.github/workflows/io-hosts.yml`
(`completion-windows`) still took seven rounds to go green at `427b9aa`, each
finding something the proxies could not. The MSVC C runtime deprecates `getenv`
under `-Werror`, so the platform layer gained one setting read (`prim.h` P4).
The driver's include-closure test needs every header staged on every platform.
The host runtime must not reach the bridge, because a program that submits
nothing links no bridge (`58d39d6`). Binding a handle to the completion port is
one critical section under the descriptor table's lock, because every lane
offers its first record on one descriptor at once and a handle cannot leave a
port, and the file leaf sets the low bit of its overlapped event so its reads
post no packet to that port (`29a48ac`). On the MSVC target a destructor
attribute is a C-runtime terminator that runs after the streams are torn down,
so the grant observer registers its report through `atexit` (`0627bf2`,
`427b9aa`). Every Windows run step now prints both channels and the exit status
before judging, and every fail-stop in the Windows link set names its site
(`3683d84`). The lesson recorded for the next platform: a cross compiler and an
emulator answer whether the code is well formed and whether the protocol is
right; they answer nothing about the host's own C runtime, its lock discipline,
or the order its teardown runs in.

## 6. Measurements (slice 4a, `8e43c54`)

`research/experiments/park-on-miss-measurements/` holds the method, every table
of §12 and of the plan's four added choices, the exact commands and the bars,
taken on 2026-09-05 on this host (Linux, four cores, GCC 13.3 for the gate and
clang 18 at `-O2` for every measured binary). Its runner is `run.sh`, wired to
`make -C compiler park-on-miss-measurements`, outside `check`. One line per
bar:

- **§12.1, park and resume at a compute miss.** Bar: within noise of the
  pre-batch numbers. **Missed**, by 45 percent at four workers and 103 percent
  at eight on `par_layout`; the grid loop-split program pays nothing, because
  it hands out large chunks and almost never misses. Open (§9).
- **§12.2, one park and one publish.** 4.40 µs best and 6.23 µs median, against
  the design's quoted 2.2 µs park-and-wake and this host's own 16.2 µs.
  **Missed at 2.2 µs, comfortably inside at 16.2 µs**; which figure applies is
  the owner's call.
- **§12.3, the claim protocol's price.** 4.40 against 4.50 µs best, 6.23
  against 6.56 median: **not separated at this host's spread**. The variant
  that removes the claim also reproduces the enumerator's S3 fault on a real
  host in under a second.
- **§12.4, the in-place wait.** Shipped only: `many_files_wide` at 112.06 ms
  by default and 112.28 ms at four helpers. The variant that sleeps at once
  fails S23's coverage assertion, which is the arm it removes.
- **§12.4, the four-stage chain.** Bar: park on miss not slower than the staged
  pipeline on the pipeline's own program. **Missed, and the run cannot test the
  bar**: this host's reads do not wait enough for any shape to reach its own
  bound, and the shape measured sleeps on `prim_host.c`'s broadcast fallback
  rather than the bridge's ring park. Open (§9).
- **§12.5, the memory orders.** The weaker orders are faster (3.12 µs best) and
  **wrong here**: the form passes the enumerator at all four configurations and
  then hangs `par_layout` deterministically at two workers and above, which is
  §6's store-then-load pair on both sides.
- **§12.5, the stack count.** The pool stops refusing at twelve stacks at four
  workers and twenty at eight; the shipped default is threads + 8. **A refusal
  costs no measurable wall time** — 274 317 refusals against none is 827 ms
  against 850 ms.
- **§12.6, the lane slot count.** **Not separated between 4 and 64** on either
  program; two slots costs 12 percent on the grid at eight workers. Under the
  plan's rule the form the enumerator checked stays.
- **§12.6, record memory per frame.** **Exactly 32 bytes per outstanding
  operation a frame holds**, and the deepest chain bound grows by the same.
- **§12.7, the chain bound per hand-out entry.** Nineteen entries across nine
  programs, **every one at most 80 bytes**: today's corpus needs one stack
  class.

The result that shaped slice 4b: five of the six behavioural switches could not
be measured. Four are rejected by the §11 enumerator, in three different kinds
— an invariant the form breaks, a state machine the enumerator does not have,
and a coverage assertion or an unmodelled word the form cannot meet — and the
fifth passes every gate and hangs a corpus program.

## 7. Replay, and the close of the slice (slice 4b)

**The switches are gone.** Under the plan's rule that a choice the measurement
cannot separate keeps the form the §11 enumerator checked,
`WF_SCHED_NESTED_NEVER_SUSPENDS`, `WF_SCHED_LOCKED_PARK`, `WF_SCHED_NO_CLAIM`,
`WF_SCHED_PARK_AT_ONCE`, `WF_SCHED_WEAK_ORDERS` and `WF_SCHED_THREAD_READY` are
deleted from `core.c`, `core.h` and `bridge.c` with every `#if` and every field
they added, and `compiler/Makefile` loses `SCHED_VARIANT_DEFINES`,
`SCHED_ENUMERATE_VARIANT_DEFINES` and `COMPLETION_ENUMERATE_CFLAGS`.
`WF_SCHED_LANE_SLOTS` stays what it was before the sweep, the `#if !defined`
override of `core.h` that the enumerator pins to 2. The experiment keeps every
table as the record of 2026-09-05, with one sentence at the top of each retired
section saying that its form was removed here and why; `run.sh` keeps every
line it can still reproduce and loses the two sections that swept forms which
no longer exist.

**The recorder and the replay (§11 item 24).** The enumerator was already the
half of item 24 that controls every interleaving of primitive steps; it is now
the other half as well. On every schedule it walks, before the search, it takes
one recorded walk and then replays it from a fresh core. The recording is that
walk's external inputs — per step the process that stepped, and at a device
step which submitted record completed, its place in the completion order, and
the result head that record's stub delivered — beside what the walk made of
them. The replay chooses nothing: it takes the process the recording names and
requires the state to enable it, delivers the recorded head instead of asking
the stub, and compares as it goes. "Identical internal execution" is checked
concretely as the whole ordered sequence of the core's own transitions — every
stack phase change (begun, committed, notified, made READY, resumed, cancelled,
emptied, taken) and every lane free-list pop and push, each with the thread that
made it — plus the core's statistics at the end and the count of transitions,
with the first difference reported at the step that produced it. A fed datum is
not inert: `wf_enum_join_io` checks that the head delivered for a record is the
head its stub answers with, so a replay fed the wrong datum fails at the join.
The sizes are reported in every sweep's line as `replay_steps`,
`replay_completions` and `replay_transitions`. The check is not vacuous, and
that was demonstrated rather than assumed: on a scratch copy of the core and
the enumerator, feeding the replay a head one greater than the stub's fails at
the join (`an I/O join read the head 51ed0001 where its stub delivered
51ed0000`), and giving `wf_sched_take_target` a hidden static that alternates
its preference across executions fails at the pick (`replay: step 28 names
process 0, which the replayed state does not enable`). Neither perturbation is
in the tree.

**One gate repaired.** `repository-invariants` had been red since slice 4a:
the experiment's `run.sh` named a home directory in its `ROOT` default, which
that stage forbids in tracked content. `ROOT` is now derived from the script's
own location and is still overridable, and the stage passes.

**Docs.** `research/investigations/io-model/LOOP-PIPELINE.md` §3.4 is corrected
in place for what the shipped emitter does now (the join site through
`compute_join_order`, completion members joined where published, the frame's
record, and the stack park in place of the continuation transform it argued
against). `docs/roadmap.md`'s two stackless items and its Windows line are
rewritten in place to name what replaced them and to point here.
`docs/ongoing/HANDOFF-2026-09-04.md` is deleted, its removal condition met;
what was still live in it is folded into `docs/current-plan.md`.

## 8. Gates

Run on this host (Linux, four cores) at the revision this record lands on.

- `make -C compiler format lint`: PASS.
- `make -C compiler completion-test`: every stage PASS, including
  `sched-smoke` and `sched-enumerate` at (T=1,S=2), (T=1,S=3), (T=2,S=3) and
  (T=2,S=4), each sweep now also recording and replaying every schedule it
  walks.
- `cargo test --profile gate --lib` and `--bins`: pass.
- `make conformance-run`: Pass=520, Xfail=1, Skip=1. `make snapshot-run`:
  Pass=491, Flip=0.
- `make repository-invariants`, `make spec-append-only`,
  `make spec-archive-integrity`, `make spec-prose-integrity`: PASS. This batch
  changes no specification bytes and no conformance content.
- `make -C compiler park-on-miss-measurements`: runs end to end on what
  remains.
- Windows: `completion-windows` in `.github/workflows/io-hosts.yml` green at
  `427b9aa` on every step, including the floor on a pool fiber and a `--par`
  program stealing across four workers.

## 9. What is left open

- **§12 item 1, the compute-miss regression, is the owner's decision and stays
  open in `docs/current-plan.md`.** `par_layout` is 45 percent slower at four
  workers and 103 percent at eight, and the design's stated fallback — nested
  runs of never-suspends jobs at a miss — needs the target-action bit at the
  hand-out, which today's emitter marks nowhere. The variant built for the
  measurement had to assume every compute hand-out never suspends, and S23
  falsifies that: it is a live-lock at one thread. So the fallback is not
  measurable until the emitter marks hand-outs, and nothing here chooses
  between the two.
- **The first API that waits.** The plan sequences `command.stdin` read next,
  for the reason this batch could not settle its own numbers: the surface the
  language has today — read-only files, directory enumeration, two standard
  outputs — cannot exercise a real wait, because a cached read does not wait
  and a cold read waits briefly and uniformly. It is the first workload on
  which this scheduler faces one.
- **The chain bar (§12 item 4) is untestable without a waiting read.** The
  largest in-flight count any shape reached was 64 against a pipeline capacity
  of 256 and a switch capacity of 72 stacks: what limits every row is the rate
  at which a worker can submit, not the depth its shape allows. A host whose
  reads cost hundreds of microseconds, or the waiting API above, is what the
  item needs.
- **The GenMC run stays deferred**, and it has one fewer question to answer:
  the enumerator's model is sequentially consistent, and this host answered the
  one pair that matters ahead of it by hanging on the weaker orders. The wider
  question — whether the core can be proved rather than only enumerated — is
  the plan's own deferred note and is not a batch 2 requirement.
- **Two enumerator gaps are named rather than closed.** §11 as written cannot
  judge §6's locked fallback (it encodes the lock-free state machine's edges),
  and it models a fixed set of words, so a per-thread ready list is unmodelled.
  Both are work, not repairs, and neither blocks anything shipped.

## Approval classes

Compiler, runtime, documentation. No specification bytes and no conformance
content changed in this batch.
