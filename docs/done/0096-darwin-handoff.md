# Batch 0096 — what one file operation costs on Darwin

Branch: `batch/0096-darwin-handoff`, from `main` at `b2e2e267` with
`batch/0093-gate-budget` merged at `d9925ae6` for its on-demand `io-bench`
workflow. Deliverables: the runtime change in `compiler/`, the tests that pin
it, the macOS section of `research/investigations/io-model/RESULTS.md`, this
record.

## Charter

Batch 0092 measured the completion model on hosted runners for the first time
and left one result standing on macOS: **where there are waits the model
works, and where there are none it costs.** Warm, the eight-wide read program
was 2.88 times slower than its own sequential build at 4 KiB and 1.27 times
slower at 64 KiB, with five times the system time; cold, it trailed a
same-width thread pool by 1.6 to 2.1 times with about twice the kernel time.
On Linux the same programs cost nothing extra — 0.58 times the sequential
build's system time on the uncached 4 KiB table — so the thing to fix was
Darwin's half of the runtime and not the model.

Darwin has no kernel completion facility for regular files, so the mechanism
is the bounded helper pool in `file_adapter.c` and `bridge.c`. Its
per-operation cost is what this batch is about.

## What one operation is made of

Before changing anything, the path was instrumented behind `WF_IO_TRACE` and
the read tables were run on `macos-14` with the eight-wide program traced at
four helper configurations. The instrumentation reads a clock at each stage
boundary and is removed with this record; the numbers below are its own, so
they are a decomposition rather than a second measurement of the wall time.

Medians of nine passes, run
[33150416900](https://github.com/mbbill/Whitefoot/actions/runs/33150416900),
per operation, nanoseconds. That run is marked failed because its
`bench-linux-read` job was cancelled; `bench-macos-read`, which is where these
numbers come from, completed in 13m50s. The stage lines were re-derived from
its uploaded artifact for this record rather than copied from the job summary:

```text
stage              warm 4K     warm 4K     cold 64K     what it is
                   h0          h3=default  h8
claim                   43           70           98    take a free slot
submit                  84        1,280        2,967    queue lock, two slot
                                                        transitions, notify
wake latency             —*      10,791       38,533    enqueue to helper
execute              1,079        1,869      168,117    the host call
publish                 55        1,467        4,334    result, event, notify
drain                  222          302          584    find the event
consume                 53          101          176    result out, slot free
park (amortised)         0        1,779        6,158    announce sleep, wake
```

\* The trace does record a figure here — 6,491 ns — but with no helper there is
no helper to wake: `helper_exec=0, scheduler_exec=28,679` says every operation
ran on the joining scheduler, so what that stage times is queue residency
between submit and join, not a cross-thread wake. It is left out rather than
put in a column whose other two entries mean something else.

and the counts that explain them, per operation:

```text
                   warm 4K h0   warm 4K h3   cold 64K h8
slot probes drained      51.86        32.57         35.69
join turns                2.08         1.37          1.44
helper sleeps             0.00         0.42          0.99
helper wakes issued       0.00         1.00          1.00
parks                     0.00         0.15          0.08
```

Read it in two halves.

**With no helper the path costs about 450 ns an operation, and half of it is
looking for the completion.** A drain has no idea where an event is, so it
compared and exchanged its way across a sixteen-slot window of a sixty-four
slot array; consecutive drains took disjoint windows, so finding one event
cost fifty-two slot probes. The rest is five uncontended mutex round trips —
claim, two transitions at submit, publish, consume — at about 45 ns each on
this host. Against a warm 4 KiB read of 1.1 us that is a 40 per cent tax, and
it is exactly the 41.78 ms the pinned zero-helper line cost against the
sequential build's 32.80 ms.

**With helpers the path costs about 5 us an operation, and almost all of it is
system calls.** Submission rose from 84 ns to 1,280 and publication from 55 to
1,467 — neither does more work than before; both are contending. Every
publication took a process-wide wake lock, every submission took it again, and
every consumption a third time, so a program crossed one global mutex three
times per operation. On Darwin a contended `pthread_mutex` is a system call.
On top of that each submission signalled a condition variable *while holding
the queue lock*, so the woken helper's first act was to block on the lock the
signaller still held, and the joining scheduler announced sleep and was woken
again for one operation in seven.

None of this is the completion model. It is what the Darwin adapter charges to
cross a thread.

## What shipped

### The wake lock is taken only when there is a sleeper

`wf_completion_notify_scheduler` raised the epoch and then took the wake lock
to look for a parked scheduler. The lock was what ordered "raise the epoch,
then look for a sleeper" against a scheduler's "announce sleep, then look at
the epoch". Both pairs are sequentially consistent now, which is the same
exclusion Dekker's algorithm gets without a lock: both sides cannot read the
old value, so either the publisher sees a sleeper and wakes it, or the
scheduler sees the new epoch and does not sleep. The two park paths — the
core's own and the Linux target's external `epoll` wait — name that order
explicitly at their increment and recheck, because the fast path is only
correct while both do.

*The number that kept it.* Publication cost 55 ns an operation with no helper
and 1,467 with three — it does no more work in the second case, it contends.
Every publication took this process-wide lock, every submission took it again
and every consumption a third time, so one program crossed one global mutex
three times per operation, and on Darwin a contended `pthread_mutex` is a
system call.

### A drain that knows when not to look, and one that is asked by name

Two changes, neither of which alters what a sweep does when it runs:

- it returns immediately when the durable ready-event count is zero, so a
  scheduler that has nothing to harvest stops probing slots to find that out.
  The count is durable — a publisher raises it before announcing the epoch —
  so a scheduler that reads zero and parks is parking against the epoch it
  snapshotted before the call;
- a token owner can drain **its own** operation's event by name
  (`wf_completion_drain_token`), which is what the three join loops now do
  before they look at anything else. A joining thread knows which operation it
  is waiting for; making it say so is cheaper than a sweep and is the shape
  the join already had.

A third was built and removed. A publication named its own slot so the next
drain could try that slot before sweeping, which turned the common case — one
event, taken by the next drain — into a single compare-exchange. It is under
"What was tried and removed" below, with the bound it broke.

*The number that kept them.* With no helper the whole path cost about 450 ns
an operation and 222 of those were the drain: a sweep has no idea where an
event is, so it compared and exchanged across a sixteen-slot window of a
sixty-four slot array, and consecutive drains took disjoint windows — 51.86
slot probes to find one event.

### One kilobyte less inside the queue lock

A queue entry holds an open's path bytes, and moving an entry out copied the
whole record — a kilobyte of path storage for every read and write as well,
inside the one lock every submission and every execution has to take. Only an
open needs its path, and now only an open pays for it.

*The number that kept it.* Submission cost 84 ns an operation with no helper
and 1,280 with three, inside the one lock every submission and every execution
has to take. The copy is a fixed part of that, paid by reads and writes that
have no path at all.

### The wake goes outside the lock, and only to a sleeper

The enqueue decides under the queue lock whether a helper is actually asleep —
that lock is the one a sleeper counts itself under, so the answer is exact
rather than a guess — and the submission issues the signal after unlocking. A
helper that woke on its own in between only makes the signal spurious, which
its predicate loop already tolerates.

*The number that kept it.* One wake was issued per operation — `helper wakes
issued 1.00` — and each was signalled while the signaller still held the queue
lock, so the woken helper's first act was to block on it. Measured beside a
wake latency of 10,791 ns an operation warm and 38,533 cold.

### A bounded look before a joining scheduler sleeps

Announcing sleep and being woken is two system calls, paid by the waiter and
by whichever thread publishes. A helper pool only exists when the adapter has
measured operations that wait, and those waits end while the joining thread
has nothing else to do, so it reads the ready-event count for a bounded window
before announcing sleep. It is a bound on wasted CPU, not a latency target: a
wait longer than the window still ends in a sleep.

*The number that kept it.* Parks cost 1,779 ns an operation amortised warm and
6,158 cold, at 0.15 and 0.08 parks per operation — the joining scheduler
announced sleep and was woken again for about one operation in seven.

### The helper policy: none until something waits, and bounded by operations

Both ends of the old policy were wrong, and the measurement says so.

It started at one helper. Queue depth was the only evidence growth had, and
depth says a program stated independent work — not that the work waits on
anything. A program reading a warm page cache states the same eight
independent reads as one reading a device and exposes the same queue, but each
of its reads is finished before a woken helper is scheduled. The adapter now
measures what its own host calls cost, from a sample of executions, and growth
needs both facts: a queue that has outrun the pool **and** a measured wait
above the threshold at which a handoff pays for itself.

Its ceiling was the machine's core count. A helper inside a host call holds no
CPU, so what bounds useful I/O concurrency is how many operations a program
can have outstanding — the bridge's operation bound. Sizing by cores capped
the three-core runner at three outstanding reads for a program that states
eight, which is a device left idle rather than a machine kept busy.

*The number that kept it.* On the after run the demand-driven default is
within two per cent of its own pinned eight-helper line on both cold tables —
591.82 ms against 585.05 at 64 KiB, 489.75 against 478.59 at 4 KiB — where in
batch 0092 the default trailed `C.wide8.h8` by 1.30 and 1.39 times. The policy
is no longer what limits the cold rows. Warm, it grows no pool at all, which
is the other half of the same rule.

### A positioned read the submitting thread would run itself is not submitted

This is the change that decides the warm tables, and it is the design's own
direct specialisation rather than a new mechanism.

When the bounded adapter holds no helper, has nothing queued, and has measured
its own operations as not waiting, a submitted read is executed by the very
thread that submitted it — at its join, after a queue crossing, a claim, four
slot transitions and a drain. There is no wait to overlap and no other thread
to overlap it on. The bridge declines the submission, and the caller takes the
ordinary direct call the emitter already emits for a refused one: the same
host call with none of the machinery. It is a throughput event of the same
class as a full queue and takes the same already-emitted route.

Two limits make it safe rather than merely fast.

Only a **positioned** transfer is declined. An offset is meaningful only on a
seekable object, and the typed opens that produce one admit nothing but a
regular file, so a positioned read waits on storage. A non-positioned read or
write may be waiting on something another part of the same program has to do —
a pipe the program itself must drain — and running one where it was stated
could stall the thread that would unblock it. That is exactly what
`independent_io_reaches_the_second_operation_before_the_first_unblocks` pins,
and it writes to a pipe; those keep the queue.

And a written `WF_IO_HELPERS` declines nothing: it pins the route with the
count. That is what makes a pinned line of a measurement a measurement of the
completion path rather than of the policy that may decline it.

The measurement keeps running while the policy is declining, because every
direct execution is timed by the same adapter. A program whose reads start
waiting is submitting again within a few operations, which is what keeps a
cold program from being demoted into its own sequential build.

*The number that kept it.* This is the change that decides the warm tables,
and `C.wide8.h0` is the control for it: the same program, same width, on the
completion path with the pool pinned off — and therefore never declined,
because a written `WF_IO_HELPERS` pins the route as well as the count. Warm at
4 KiB it costs 40.57 ms against `S.wide8`'s 32.65, which is the 24 per cent
the machinery charges an operation with nothing to overlap. With the policy
free to decline, the same program costs 33.57. In milliseconds over the
sequential build that is 7.92 before and 0.92 after: the policy removes 88 per
cent of what the machinery was charging. And the cold tables show it does not
remove the overlap along with the cost — the same binary is 2.84 times faster
than `S.wide8` on the cold 4 KiB table.

## What was tried and removed

**A spin before a helper sleeps.** The symmetric idea to the joining
scheduler's: a helper that has just finished one request is often a fraction
of a microsecond from the next, and sleeping across that gap costs two system
calls. Built and measured locally: it removed only a quarter of the helper
sleeps on the eight-wide warm program while eight helpers spinning drove
submission from 313 ns to 2,510 ns an operation through queue-lock contention.
On a three-core runner that is worse still — eight spinners on three cores is
not idle CPU, it is the CPU the reads need. Removed. The joining scheduler's
spin stayed because there is exactly one of it.

**A drain hint: the last publication naming its own slot.** The attribution
table above says the pool-off path spent half its time looking for the
completion — fifty-two slot probes an operation to find one event — so a
publication stored its slot and the next drain tried it before sweeping.
Built, and removed after the contract probes said what it cost.

`wf_completion_drain` promises to drain at most `scan_budget` slots, and the
hint was not a reordering inside that bound but an escape from it: the named
slot is wherever the last publication happened, and the drain took it whether
or not it lay in the window the budget describes. Two probes say what the
bound is for. The writer-scheduler probe publishes on slot 16 of 17 and drains
with a budget of 16, requiring zero — a bounded scan that stops short of a
token must not make that token's dependent writer frame runnable — and the
hint drained the slot and released the frame. The native adapter probe failed
from the other side: with two events outstanding it drove them to terminal in
turn, and a drain answering with the newest published rather than the one the
sweep would have met left the named token unconsumed. Making the hint legal
would mean hinting only into the window about to be scanned, which is the
sweep it existed to skip.

What settles it is that the cost it targeted had already been removed by this
batch's own direct specialisation. Fifty-two probes an operation was the
zero-helper warm read path, and that path no longer reaches the bridge at all:
a positioned read with no helper, nothing queued and no measured wait is
executed where it was stated. The sweeps that remain have the zero-ready-events
early return in front of them. So the hint was an unmeasured saving on a path
its own measurement had deleted, bought with a stated bound.

**Making the queue lock shorter by moving the slot transitions out of it.**
A submission holds the queue lock across two slot-lock round trips because the
core's handoff protocol is begin, reserve, accept, expose. Reserving an entry
without publishing it needs a ring in which entries are filled out of order and
consumed in order, which is a multi-producer ring with its own sequence
numbers. It was rejected without building it: the measured convoy is the wake
inside the lock and the kilobyte copy, both of which came out for a few lines
each.

## The numbers

Full tables and host details are in
`research/investigations/io-model/RESULTS.md`. Every number here is from a
GitHub-hosted runner through the `io-bench` workflow, never from a maintainer
machine, and before and after are separate runs on separate draws of the
`macos-14` label — so **ratios within a run are the evidence and absolute
milliseconds across runs are not**. On this pair the warm and many-files
tables happen to be closely matched draw to draw and the cold tables are not,
which the section below states line by line rather than assuming either way.


### The macOS runner, before and after

**Before** is the batch-0092 macOS-runner section of
`research/investigations/io-model/RESULTS.md`. **After** is run
[33155821397](https://github.com/mbbill/Whitefoot/actions/runs/33155821397),
job `bench-macos-read`, at commit `266acf4f` — the final runtime of this
branch, with the drain hint already removed. Both are `macos-14`, Apple M1
(Virtual), three CPUs, 7 GiB; medians of nine interleaved passes, milliseconds.

```text
                          before (0092)              after (33155821397)
line                cold64  cold4  warm64 warm4  cold64  cold4 warm64  warm4
N.direct           2345.41 1971.16 169.00 33.10 1472.85 1372.47 176.24 33.04
N.pool8             772.34  532.07  71.08 15.18  424.58  381.86  80.13 15.03
S.narrow           2045.43 1663.83 152.57 31.20 1488.30 1370.55 159.86 31.30
S.wide8            2108.61 1736.79 166.31 32.88 1487.24 1392.83 172.94 32.65
C.narrow.default   1952.50 1889.77 153.01 31.34 1516.75 1382.75 160.51 31.64
C.wide8.default    1220.68 1100.57 211.58 94.72  591.82  489.75 173.97 33.57
C.wide8.h0         1681.74 1611.22 175.21 41.84 1452.09 1432.86 180.33 40.57
C.wide8.h8          940.47  793.93 205.61 118.48 585.05  478.59 224.20 83.33
```

The many-files workload, same two runs, same units:

```text
line                 before (0092)   after (33155821397)
N.direct                    141.06                141.84
N.pool8                      57.95                 58.10
S.wide8                     144.28                145.01
C.narrow.default            144.31                144.83
C.wide8.default             173.16                148.23
C.wide8.h0                  149.36                149.42
```

The two draws are close on the lines whose code did not change — `N.direct`
warm 4 KiB is 33.10 against 33.04, `S.wide8` warm 4 KiB 32.88 against 32.65,
`N.pool8` many-files 57.95 against 58.10 — so the read-heavy warm and
many-files halves of this comparison are unusually well matched for two
separate draws of a hosted label. The cold halves are not: this draw's whole
cold table is faster than 0092's on every line, native baselines included
(`N.pool8` cold 4 KiB 381.86 against 532.07), which is the hypervisor-level
caching the 0092 section documents. **Ratios within a run are the evidence
there, and absolute milliseconds across runs are not.**

### Against the bar

The bar is the owner's: on the macOS runner's read-heavy tables, warm
`C.wide8` not slower than `S.wide8`; cold `C.wide8` within ten per cent of
`N.pool8`; many-files `C` not slower than `S`; and Linux must not regress.

```text
row                        before      after     bar          met
warm 64 KiB  C/S           1.27x       1.006x    <= 1.00x     0.6% over
warm  4 KiB  C/S           2.88x       1.028x    <= 1.00x     2.8% over
cold 64 KiB  C/N.pool8     1.58x       1.394x    <= 1.10x     no
cold  4 KiB  C/N.pool8     2.07x       1.283x    <= 1.10x     no
many-files   C/S           1.20x       1.022x    <= 1.00x     2.2% over
```

**The two warm rows and the many-files row land within three per cent of a bar
they missed by 27, 188 and 20 per cent.** None of the three is met on a strict
reading — C is 0.6, 2.8 and 2.2 per cent slower than S rather than not slower
— and that residue is the honest answer to what the completion path still
costs a program with nothing to overlap. The line that says where it went is
`C.wide8.h0`: the same program on the completion path with the pool pinned off
and therefore never declined costs 40.57 ms warm at 4 KiB against S's 32.65,
which is the 24 per cent the machinery charges. In milliseconds over `S.wide8`
that is 7.92; with the policy free to decline it is 0.92, so 88 per cent of the
charge is gone and what remains is the operations the policy does not decline —
the opens and the closes, which keep the queue.

**Neither cold row is met, and the cold 64 KiB row is where the two draws
disagree.** In the earlier run on this branch
([33153717709](https://github.com/mbbill/Whitefoot/actions/runs/33153717709),
commit `96bb4778`) the same row read 817.47 against 808.79, which is within
one per cent and would have passed. That run is not the one to read it from:
its `N.pool8` cold 64 KiB line has a median of 808.79 against a minimum of
445.55, an 1.8-fold spread inside a single line, while this run's spread on
the same line is 417.49 to 453.58 — under nine per cent. The bar is read from
the tighter draw, which fails it, rather than from the draw that happens to
pass because the baseline it is measured against was noisy.

What the cold rows do say is that the demand-driven policy now finds the
helpers the work needs: `C.wide8.default` and `C.wide8.h8` are within two per
cent of each other on both cold tables (591.82 against 585.05, and 489.75
against 478.59), where in 0092 the default trailed its own pinned eight-helper
line by 1.30 and 1.39 times. The policy is no longer the limit; the remaining
distance to `N.pool8` is. Against its own sequential build the same program is
2.51 and 2.84 times faster cold, where in 0092 it was 1.73 and 1.58.

The stage-level attribution for the cold miss is in the table at the top of
this record, measured on the same runner label: at eight helpers the path
charges about 5 us an operation against a 168 us host call, of which 38.5 us
is wake latency alone — the time between a submission enqueueing work and a
helper being scheduled to run it. On a three-core runner, eight helpers is
more threads than cores, and that latency is the scheduler rather than the
adapter. Nothing in this batch's change set addresses it.


## Tests

Every case below is an ordinary compiler or harness test. No conformance case,
manifest, verdict, adapter, or collection wiring is added, changed, or removed
by this batch.

- **The growth rule is decided, not sampled.** The helper policy now turns on a
  measurement, so a case that sleeps and hopes would test the machine. The
  adapter reads its clock through a named seam of the same class as
  `WF_COMPLETION_PREAD` and `WF_COMPLETION_POLL`; the harness build names it and
  two cases script it. `test_pool_stays_empty_when_operations_do_not_wait`
  scripts warm work and requires no helper to appear;
  `test_pool_grows_when_operations_wait` scripts waiting work and requires
  growth to the cap and no further.
- **The process-wide budget case keeps only what it can assert.** How many
  helpers a program *has* depends on whether this harness's own temporary-file
  operations happened to wait, which is a property of the machine. The ceiling
  is not, so the ceiling is what it asserts, and it says so in place.
- **Both harness defines are named once in the Makefile.** The sanitizer build
  had drifted to a different set and was therefore exercising a different
  runtime from the one the plain build tested. That is a test-integrity fix,
  not a convenience: two builds under one target name must run one runtime.
- **The bridge cases assert a pinned route.** Under the demand-driven policy the
  bridge declines a positioned read once it has measured this host's reads as
  not waiting, so whether a case sees the submitted route would depend on what
  the cases before it happened to execute. `main` writes `WF_IO_HELPERS` when
  the invocation did not name one, which pins the route with the count, and
  every bridge case then asserts a route that is fixed rather than sampled.
- **What that pinning leaves out is covered where it can be decided.** The
  declining policy itself is exercised by the two scripted-clock cases above;
  end to end, by `independent_io_reaches_the_second_operation_before_the_first_unblocks`,
  whose fourth arm removes `WF_IO_HELPERS` from the environment entirely; and by
  `compiler/tests/programs`, which builds and runs whole compiled programs —
  `wfgrep` and `raw_deflate` read real files — with nothing pinned, so a
  declined positioned read that returned the wrong bytes would fail them.
- **The contract's boundedness is pinned by two probes, and they earned their
  keep this batch.** `writer_scheduler_probe.c` publishes on slot 16 of 17 and
  drains with a budget of 16, requiring zero events and an unready frame;
  `native_adapter_probe.c` drives two outstanding operations to terminal in
  turn. Both are unchanged by this batch. Both caught the drain hint, which is
  why it is in "What was tried and removed" rather than in "What shipped".

## Judgment calls

- **The drain hint was removed rather than made legal.** It could have been kept
  by hinting only into the window the sweep was about to scan, which is the
  sweep it existed to skip, or by widening `scan_budget`'s meaning, which is
  editing the promise to fit the code. Neither is worth an optimisation whose
  own motivation — fifty-two slot probes an operation on the pool-off warm read
  path — was deleted earlier in this same batch by the direct specialisation.
  The two probes that caught it are the reason the choice was easy: they state
  what the bound is for.
- **The harness helper went back to the sweep it was asserting.** When the hint
  landed, `drain_and_consume_file` was changed to drain its token by name
  because a sweep could no longer be relied on to answer with the right event.
  With the hint gone the sweep is deterministic again, and draining by name
  would have made its `event.token.slot == token.slot` assertion tautological.
  Restoring `drain_exact` restores what that case covers.
- **Only a positioned transfer is declined.** This is a liveness argument, not a
  performance one. An offset is meaningful only on a seekable object and the
  typed opens that produce one admit nothing but a regular file, so a positioned
  read waits on storage and on nothing the program itself must do. A
  non-positioned read or write may be waiting on a pipe the same program has to
  drain, and running one where it was stated could stall the very thread that
  would unblock it. That is exactly what
  `independent_io_reaches_the_second_operation_before_the_first_unblocks` pins,
  and it writes to a pipe.
- **A written `WF_IO_HELPERS` declines nothing.** A written setting is an
  instruction about how to run, so the runtime stops choosing. That is also what
  makes a pinned line of a measurement a measurement of the completion path
  rather than of the policy that may decline it — without it, `C.wide8.h0` and
  `C.wide8.default` would not be measuring the same thing.
- **The helper ceiling is the bridge's operation bound, not the core count.** A
  helper inside a host call holds no CPU, so what bounds useful I/O concurrency
  is how many operations a program can have outstanding. Sizing by cores capped
  the three-core runner at three outstanding reads for a program that states
  eight, which is a device left idle rather than a machine kept busy.
- **The wake fast path is Dekker's exclusion, not a lock.** `notify_scheduler`
  raised the epoch and then took the process-wide wake lock to look for a parked
  scheduler; the lock ordered that against a scheduler's "announce sleep, then
  read the epoch". Both pairs are sequentially consistent now, so both sides
  cannot read the old value: either the publisher sees the sleeper and wakes it,
  or the scheduler sees the new epoch and does not sleep. Both park paths — the
  core's own and the Linux target's external `epoll` wait — name that order
  explicitly at their increment and recheck, because the fast path is correct
  only while both do.
- **Stage attribution shipped as scaffolding and was removed with the record.**
  The numbers above are the instrumentation's own clock readings, so they are a
  decomposition of the path rather than a second measurement of the wall time,
  and the record says so where they appear. The instrumentation is gone from the
  tree; the run id is how it stays checkable.

## Not done

- **A stage decomposition of the shipped path.** The attribution table measures
  the path as it was before this batch changed it. Re-instrumenting the shipped
  runtime would have needed another pair of `io-bench` runs to say anything the
  wall-clock tables do not already say, and the tables are what the bar is read
  from.
- **The cold 4 KiB row.** It is the one row that misses its bar by more than a
  little, and the stage table says why: at eight helpers on a three-core runner
  the path charges about 5 us an operation against a 168 us host call, but the
  wake latency alone is 38.5 us and the pool is bounded by the runner's ability
  to schedule eight threads on three cores. This is a scheduling question on a
  small machine, not a completion-model question, and nothing in this batch's
  change set addresses it.
- **Windows.** The IOCP adapter is untouched. `completion-windows` links and
  passes as before; none of the Darwin helper-path work applies to it.
- **A unit-level case for a declined positioned read's bytes.** The declining
  path is covered end to end by `compiler/tests/programs` rather than by a
  harness case of its own, because the harness pins the route for every
  invocation so that its other cases can assert one. A case that unpinned the
  route inside the harness would be sampling the policy, which is the thing the
  scripted-clock cases were written to stop doing.
- **The many-files workload.** It is recorded and it is still slower under C
  than under S. This batch narrowed it and did not close it; the reason is the
  one batch 0092 reached, that a 17 us `openat` is not a wait worth a handoff.

## Approval classes

- **No specification bytes.** `spec/kernel-spec.md` is untouched;
  `make spec-digest-sync` reports the live prose quoting the chain tail at
  v0.38 (`5a43c7638bd5839d77829836518374f9a169eb953d9c1edbd66b87815aedfb2d`).
- **No conformance content.** No conformance case, manifest, verdict, adapter,
  runner, or collection wiring is added, modified, deleted, or renamed. The six
  red `sys14-*` and `accept-*` cases on Linux are the pre-existing
  `directory_next` gap recorded by batch 0094, unchanged by this batch: the run
  reports `Pass=503 Fail=6 Skip=1` with every failure
  `TargetQualification(MissingMapping(Operation(12)))`.
- **`make approval-history-integrity`** reports existing main records an exact
  prefix.
- Everything else here is compiler implementation, ordinary tests, workflow
  wiring inherited from batch 0093, and this record.
