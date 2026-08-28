# Batch 0096 — what one file operation costs on Darwin

Branch: `batch/0096-darwin-handoff`, from `main` at `b2e2e267` with
`batch/0093-gate-budget` merged at `d9925ae6` for its on-demand `io-bench`
workflow. Deliverables: the runtime change in `compiler/`, the tests that pin
it, the batch-0096 section of `research/investigations/io-model/RESULTS.md`,
this record.

## Charter

Batch 0092 measured the completion model on hosted runners for the first time
and left one result standing on macOS: **where there are waits the model
works, and where there are none it costs.** Warm, the eight-wide read program
was 2.88 times slower than its own sequential build at 4 KiB and 1.27 times
slower at 64 KiB, with five times the system time; cold, it trailed a
same-width thread pool by 1.6 to 2.1 times, at 1.18 times that pool's kernel
time and 1.55 to 1.60 times its own sequential build's (`C.wide8.default`
755.87 and 698.82 ms of system time against `N.pool8`'s 639.97 and 592.59 and
`S.wide8`'s 473.90 and 450.31).
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

Medians of nine traced passes, run
[33150416900](https://github.com/mbbill/Whitefoot/actions/runs/33150416900),
per operation, nanoseconds. Nine and not seven because the trace prints on the
two warm-up passes as well as the seven recorded ones, so it is a different
count from the wall-clock tables' and correct for this table only. That run's
conclusion is `cancelled`, not `failed`, because its `bench-linux-read` job
was cancelled; `bench-macos-read`, which is where these numbers come from,
completed in 13m50s. The stage lines were re-derived from its uploaded
artifact for this record rather than copied from the job summary:

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
helper sleeps             0.00         0.42          1.00
helper wakes issued       0.00         1.00          1.00
parks                     0.00         0.15          0.17
```

The last column's two right-hand figures were 0.99 and 0.08 in the first draft
of this record and are corrected here against the artifact: the medians of the
nine passes are 0.999 helper sleeps and 0.168 parks per operation. Nothing in
the argument turned on either.

Read it in two halves.

**With no helper the path costs about 450 ns an operation, and half of it is
looking for the completion.** A drain has no idea where an event is, so it
compared and exchanged its way across a sixteen-slot window of a sixty-four
slot array; consecutive drains took disjoint windows, so finding one event
cost fifty-two slot probes. The rest is five uncontended mutex round trips —
claim, two transitions at submit, publish, consume — at about 45 ns each on
this host. Against a warm 4 KiB read of 1.1 us that is a 40 per cent tax on
the host call. What the same configuration cost end to end is a smaller
number and a different measurement: the pinned zero-helper line at 41.78 ms
against the sequential build's 32.80 ms is 27.4 per cent, on run
[33149563172](https://github.com/mbbill/Whitefoot/actions/runs/33149563172)
— not on 33150416900, whose traced build reads the same pair at 47.09 against
37.44. The two figures are not the same quantity and the first draft of this
record equated them: 40 per cent is the path against one host call, 27.4 is
the whole program against the whole program, which also does work no
completion path touches.

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
6,158 cold, at 0.15 and 0.17 parks per operation — the joining scheduler
announced sleep and was woken again for about one operation in six or seven.
(0.08 was the first draft's figure for the second of those and is the same
stale number the counts table above corrects to 0.168.)

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
within 1.2 per cent of its own pinned eight-helper line at 64 KiB (591.82 ms
against 585.05) and 2.3 per cent at 4 KiB (489.75 against 478.59), where in
batch 0092 the default trailed `C.wide8.h8` by 1.30 and 1.39 times. Both are
inside the same table, so the cache-label problem recorded with those tables
does not reach this comparison. The policy
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
job `bench-macos-read`, at commit `266acf4f` — the measured runtime of this
branch, with the drain hint already removed. What the tip adds on top of it is
the correctness follow-up in "After the tables" below; the tables were not
re-measured against it, and the reason that is honest rather than convenient
is stated there. Both draws are `macos-14`, Apple M1
(Virtual), three CPUs, 7 GiB; medians of seven recorded interleaved passes
after two unrecorded warm-ups, milliseconds. (`io-bench.yml` sets
`ROUNDS: "7"`, `WARMUP: "2"`; the stage table at the top of this record says
nine because the trace also prints on the warm-ups.)

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

**And the after run's cold tables are not uncached tables.** Its own probes
refused the uncached label *before* each of them and confirmed it after —
`refusing the uncached label -- 93 of 128 sampled reads (72.7%) were at or
below 40.0 us` before the 64 KiB table and `120 of 128 (93.8%)` before the
4 KiB one, against `4 of 128 (3.1%)` and `3 of 128 (2.3%)` after — and the
artifact prints, beside each, `the label above is NOT confirmed: the probe
refused it before the table ran`. That is the reverse of 0092's direction,
where both cold tables were confirmed before and refused after. So each cold
line here is a mixture of resident and non-resident reads rather than a cold
one, which is why the two cold rows of the bar below are not read from *this*
draw. They are graded from a later one — run 33172323795, at this record's own
repair commit, the first on this branch whose probe confirms the uncached
label at both ends of both cold tables — and the section below tabulates all
five macOS draws' cache labels side by side instead of reading a cold ratio
from whichever one is at hand.

### Against the bar

The bar is the owner's: on the macOS runner's read-heavy tables, warm
`C.wide8` not slower than `S.wide8`; cold `C.wide8` within ten per cent of
`N.pool8`; many-files `C` not slower than `S`; and Linux must not regress.

```text
row                       before   after   bar          met         before is
warm 64 KiB  C/S          1.27x    1.006x  <= 1.00x     0.6% over   0092 macOS
warm  4 KiB  C/S          2.88x    1.028x  <= 1.00x     2.8% over   0092 macOS
cold 64 KiB  C/N.pool8    1.58x    1.394x  <= 1.10x     no          0092 macOS
cold  4 KiB  C/N.pool8    2.07x    1.283x  <= 1.10x     no          0092 macOS
many-files   C/S          1.20x    1.022x  <= 1.00x     2.2% over   0092 macOS
Linux warm 64 KiB C/S     0.98x    1.026x  no regress   unresolved  0092 Linux
Linux warm  4 KiB C/S     0.94x    1.055x  no regress   unresolved  0092 Linux
Linux many   C/S          1.041x   1.058x  no regress   unresolved  0090 Linux
                          1.045x
```

This is the same table as the one in the RESULTS section, row for row. The
`before is` column is there because the rows do not share a baseline: the five
macOS rows are against the batch-0092 macOS-runner section, the two Linux read
rows against its Linux-runner section, and the Linux many-files row has no
0092 reading at all — it is against batch 0090's **two** draws of that job,
1.041 and 1.045. Its 1.058 is worse than both, by 1.3 points on the worse of
them, against the roughly 2 per cent within-run spread batch 0090 reports for
that job; C being slower than S there is batch 0090's own finding and not new,
but two draws either side of a change cannot separate 1.058 from 1.045, so the
row is `unresolved` and not `yes`.

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

**Both cold rows are graded `no`, and the draw that reads them arrived last.**
For most of this batch neither row could be read at all: every macOS draw
failed a quality test, and the section below tabulated which. The repair
commit's own `io-bench` run then produced the draw the record kept saying was
owed — a macOS table whose uncached label its probe confirms *before and
after*, at both sizes — and on it the two rows read 1.477 and 1.557 against a
bar of 1.10. That is a miss, not a pass, and it is a wider miss than the
unlabelled mixture the `after` column above reports.

Rather than repeat a partial table here, the complete one is in
`research/investigations/io-model/RESULTS.md` under **"Every io-bench draw on
this branch"** — every `io-bench` run on this branch, every job, every table,
one row each, enumerated with `gh run list` — and beside it, under "Against
the standing bar", a nine-row `bench-macos-read` cold table giving each draw's
probe verdicts, its `C/N.pool8`, and the min..max of both lines that ratio is
taken from. There are nine macOS draws, not five. What they say about this
grade:

- `33172323795` at `261070c8` is the **only** draw on this branch that
  confirms the uncached label at both ends of *both* cold tables. Its medians
  are the grade: 1.477 at 64 KiB and 1.557 at 4 KiB.
- Of the eighteen macOS cold tables — nine draws at two sizes — four are
  confirmed at both ends: `261070c8`'s two, `33153717709`/`96bb4778`'s 64 KiB
  one at 1.011, which would pass, and `33151353052`/`4a748d6e`'s 4 KiB one at
  1.419, which would not. The other fourteen have a probe that refused the
  label at one end or both, so they are not readings of a cold device.

The graded draw is noisy: `C.wide8.default` runs 1044.77 to 14961.31 cold at
64 KiB and 560.66 to 4332.72 cold at 4 KiB, on a runner whose load average was
5.60 at the start — so the honest thing is to say what survives that noise and
what does not. At 64 KiB it survives: `C.wide8.default`'s *minimum* over
`N.pool8`'s median is still 1.34, outside the bar without needing the median
at all. At 4 KiB it does not survive as cleanly: C's minimum over `N.pool8`'s
median is 0.82, so the two lines' ranges overlap and only their medians
separate them. Neither row is met on any statistic that puts C ahead, so both
are graded `no`; the 4 KiB row is the weaker of the two gradings and this says
so — with `4a748d6e`'s confirmed 4 KiB table agreeing with the grade from the
other side at 1.419.

`96bb4778`'s 1.011 is the one confirmed reading that would pass, and its
baseline moves: `N.pool8` runs 445.55 to 898.44 around a median of 808.79, and
`C.wide8.default`'s own maximum on that line is 8252.57 against a median of
817.47. So the two confirmed-cold 64 KiB readings this branch has are 1.011
and 1.477, which is the range a single hosted runner label covers, and the
`no` grade rests on the newer of them being the one where both probes agree at
both sizes.

Run [33158144391](https://github.com/mbbill/Whitefoot/actions/runs/33158144391)
at commit `72e98cba` is a doc-only commit on the same runtime, taken after the
tables above were written. Its warm and many-files halves corroborate this
one — 1.002 warm at 64 KiB, 1.026 warm at 4 KiB, 1.019 many-files — and its
cold half is unreadable: both labels refused at both ends, `N.pool8` cold
64 KiB spanning 416.53 to 1075.57, and `C.wide8.default` cold 4 KiB reaching
26351.72 against a median of 960.86, which is the widest `C.wide8.default`
4 KiB spread of the nine.

The numbers reported above the bar are `33155821397`'s because its commit
`266acf4f` is the runtime the before/after comparison is about — not because
its halves are the tightest, which they are not: the draw table puts
`96bb4778` closer to 1 on macOS warm 4 KiB (1.0168 against 1.0282) and
`a06c53f9` closer on macOS many files (1.0144 against 1.0222). `266acf4f` is
not this branch's last runtime: the follow-up read under "What the follow-up
cost" changes `runtime.c`, `bridge.c`,
`file_adapter.c/.h` and `contract.h` after it, at `a06c53f9`, and this
record's own repairs change the runtime once more after that. **What was owed
is no longer a confirmed cold label — that arrived and the answer was a miss.
What is owed now is a confirmed cold table on a quiet runner, which would
narrow the 1.011-to-1.477 range rather than decide whether a cold table can be
read at all.**

What the cold tables do say, and this does not depend on the label because
both lines ran interleaved inside one table, is that the demand-driven policy
now finds the helpers the work needs: `C.wide8.default` and `C.wide8.h8` are
within 1.2 per cent of each other at 64 KiB (591.82 against 585.05) and 2.3
per cent at 4 KiB (489.75 against 478.59), where in 0092 the default trailed
its own pinned eight-helper line by 1.30 and 1.39 times. The policy is no
longer the limit. Against its own sequential build the same program is 2.51
and 2.84 times faster on those tables, where in 0092 it was 1.73 and 1.58.

**The Linux row is unresolved, and this record does not resolve it.** The
`bench-linux-read` job in the same run landed on a Xeon 8573C whose cold
tables span 1.26 to 4.98 times inside a single line, nineteen of the forty
lines past 2.5 — `S.narrow` cold 64 KiB runs 2478 to 11054 ms around a median
of 4258, and `N.direct` cold 64 KiB 2781 to 7661 around 6158 — so the cold
half of it cannot be read at all. Its warm half is tight and disagrees with the two prior Linux
draws, which agreed with each other: warm `C.wide8` over `S.wide8` is 1.026 and
1.055 here against 0.982/0.941 in batch 0092 and 0.984/0.946 at `96bb4778`.

The narrow lines are the control, and they do not exonerate the wide ones:

```text
warm C/S                  0092   96bb4778     this   delta
64 KiB  C.wide8/S.wide8  0.9816   0.9840   1.0261   +0.042
64 KiB  C.narrow/S.narrow 1.0035  1.0023   0.9969   -0.005
 4 KiB  C.wide8/S.wide8  0.9412   0.9460   1.0550   +0.109
 4 KiB  C.narrow/S.narrow 1.0112  1.0157   1.0514   +0.036
```

`C.narrow` and `S.narrow` are the same source with and without the completion
lowering and state no overlap width, so a host difference should move them with
the wide pair. At 64 KiB the narrow pair does not move at all while the wide
pair moves 4.2 points, and at 4 KiB the wide pair moves three times as far. The
movement is concentrated where the completion path does its work.

What cuts the other way is that there is no change to point at. The only
completion-source difference between `96bb4778`, which read 0.946, and
`266acf4f`, read here, is the removal of the `WF_IO_TRACE` instrumentation —
`git diff 96bb4778..266acf4f -- compiler/src/backend/completion/` is that and
nothing else — and removing instrumentation does not make a program slower. The
host also differs, though not in core count: all three Linux draws report four
CPUs. What differs is the processor and the disk — Xeon Platinum 8370C on
`sda1` for 0092, EPYC 7763 on `sda1` for `96bb4778`, Xeon Platinum 8573C on
`nvme0n1p1` here — and the wide lowering has more scheduling surface than the
narrow one. The many-files Linux job in the same run keeps 0090's ordering but
not its ratio: 1.058 against 0090's 1.041 and 1.045 is worse than both, which
is why that row is `unresolved` above rather than met.

So the honest reading is that one Linux draw on different hardware neither
confirms nor refutes the no-regression bar, and another draw is owed — and the
narrow control makes it a draw worth resolving rather than dismissing. That
draw arrived with the correctness follow-up and is read under "What the
follow-up cost" below; it agrees with the two readings before this one. What is
not in doubt is Linux correctness: `io-hosts` `completion-linux` is green on
this commit, including the required native io_uring adapter probe and the
harness under the address and undefined sanitizers, and the same targets pass
in a local Linux container. Thread sanitizer is a separate step and runs the
probes rather than the harness — the isolated core/read probe, and the
default-route bridge probe added in this batch's follow-up.

The stage-level attribution for the cold miss is in the table at the top of
this record, in its `cold 64K h8` column and measured on the same runner
label: at eight helpers the path charges about 5 us an operation against a
168 us host call, of which 38.5 us is wake latency alone — the time between a
submission enqueueing work and a helper being scheduled to run it. On a
three-core runner, eight helpers is more threads than cores, and that latency
is the host scheduler rather than the adapter. That column covers the cold
64 KiB row; there is no cold 4 KiB column, so the same account is inferred for
that row and not measured. Nothing in this batch's change set addresses
either.


## Tests

Every case below is an ordinary compiler or harness test. No conformance case,
manifest, verdict, adapter, or collection wiring is added, changed, or removed
by this batch.

- **The growth rule is decided, not sampled, and the cases reach the bounds
  they name.** The helper policy now turns on a measurement, so a case that
  sleeps and hopes would test the machine. The adapter reads its clock through
  a named seam of the same class as `WF_COMPLETION_PREAD` and
  `WF_COMPLETION_POLL`; the harness build names it and three cases script it.
  `test_pool_stays_empty_when_operations_do_not_wait` scripts warm work and
  requires no helper to appear.

  The other two need a queue the pool cannot empty, and getting that right is
  the whole of what makes them worth having. Growth is gated on
  `queue_count > held`, so a driver that waits for each request before
  submitting the next holds the queue at one entry and the pool at one helper
  whatever cap it was given — a case written against such a driver and
  asserting an upper bound above one asserts nothing, because the bound is
  never approached and deleting the code that enforces it changes no verdict.
  That is what the first versions of these two cases did. They now submit
  twenty reads of an *empty pipe* without waiting: a helper that takes one
  blocks in the host call and never returns for another, so the queue only
  deepens and the pool climbs until its cap stops it.
  `test_pool_grows_when_operations_wait` then requires exactly four helpers
  against a cap of four, and
  `test_helper_growth_stops_at_the_helper_storage` exactly two against a cap
  of eight and storage of two.
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
- **What that pinning leaves out is covered where it can be decided, and the
  corpus is not one of those places.** The declining policy itself is
  exercised by the scripted-clock cases above, by
  `independent_io_reaches_the_second_operation_before_the_first_unblocks`,
  whose fourth arm removes `WF_IO_HELPERS` from the environment entirely, and
  by `bridge_default_probe.c` below. It is *not* exercised by
  `compiler/tests/programs`: a decline is a zero returned from
  `wf__completion_file_pread_submit`, and counted over
  `whitefootc --emit-llvm` no corpus program emits that call at all —
  `wfgrep` and `raw_deflate_boundary` read their files through
  `wf__completion_file_pread_direct`, which is the route a decline falls back
  to and not the decision. "Not done" below carries the count.
- **The shipped default route has its own translation unit.**
  `bridge_default_probe.c` runs the bridge with `WF_IO_HELPERS` unset and
  refuses to run with it set, four lanes and sixteen thousand positioned reads
  against a fixture whose byte at offset `o` is `o % 251`, so that every read
  must deliver its own byte whichever route the bridge chose. It exists
  because pinning the route for the harness — which the bullet above explains
  and which is right for the harness — left the demand-driven policy covered
  by no bridge test at all. It reads the route it took from the runtime's own
  submission counters and asserts it: a submitted positioned read on either
  route, and on the POSIX-adapter route one of the policy's two branches —
  a declined read, or a started helper.

  Requiring specifically a decline was tried and is wrong, and the run that
  showed it is worth recording. Both branches turn on the same measurement:
  a read is left to its caller when the adapter measured no wait and holds no
  helper, and a helper is started when it measured one. Which branch a run
  gets is a property of how fast this host's reads are, not of the runtime.
  The same sanitized binary on the same machine declined 15 814 of 16 000
  reads with `helpers=0` when the host was quiet, and declined none while
  growing three helpers when it was loaded — because under a sanitizer on a
  busy machine a one-byte read really does cost more than the twenty
  microseconds the rule asks about, and the policy then correctly took its
  other branch. So the probe requires one branch or the other; requiring the
  decline would be requiring an idle host. Forcing the decline off in
  `bridge.c` still fails it, on every run, with
  `the demand-driven policy took neither branch in 16000 positioned reads`.

  `completion-test` and `completion-sanitize` run it, and
  `completion-default-route-tsan` gives `io-hosts` a thread sanitizer over the
  bridge.
- **The helper storage bounds the helper cap, and two cases say so.**
  `test_helper_growth_stops_at_the_helper_storage` tells
  `wf_file_adapter_init` it has two helpers of storage, asks for a cap of
  eight, and requires the pool to reach exactly two against the deep queue
  described above. Its array is deliberately longer than the two entries init
  is told about: the bound under test is the declared capacity, and the slack
  decides only whether a missing clamp is *reported* as a pool larger than its
  storage or executed as a write past the end of the frame, which ends the run
  before any check is reached. Deleting the clamp from
  `wf_file_adapter_set_helper_cap` makes it fail with
  `check failed: held == 2` at a measured `held` of eight.
  `test_helper_count_above_its_storage_is_refused` requires an initial count
  above the storage to be refused outright rather than clamped, because that
  one is threads to start now rather than a ceiling to grow towards.
- **The named drain's generation check has a case that fails without it.**
  `test_named_drain_refuses_a_recycled_slot` retires one operation on a
  one-slot runtime, lets a second reuse the slot, and requires the retired
  token to take nothing and the live owner to find its own event exactly once.
- **The contract's boundedness is pinned by two probes, and they earned their
  keep this batch.** `writer_scheduler_probe.c` publishes on slot 16 of 17 and
  drains with a budget of 16, requiring zero events and an unready frame;
  `native_adapter_probe.c` drives two outstanding operations to terminal in
  turn. Both are unchanged by this batch. Both caught the drain hint, which is
  why it is in "What was tried and removed" rather than in "What shipped".
- **Where each of these actually runs, including what canonical `make check`
  does not.** The gate's compiler stage list is `format lint test-partition
  test-unit test-sampling test-corpus docs spec completion-test`, so what
  `make check` runs of the C runtime is `completion-test` alone: the harness
  at `WF_IO_HELPERS` 0, 1 and 4, one `WF_IO_NOCACHE` arm, the core/read probe,
  and the default-route probe. Everything else is CI's.
  `completion-sanitize` — the harness and both probes under the address and
  undefined sanitizers — and the two thread-sanitizer arms,
  `completion-core-read-tsan` and `completion-default-route-tsan`, are wired
  into the `io-hosts` workflow's Linux job and into no gate stage.
  `make check` therefore runs **no sanitizer of any kind**, and the
  data-race evidence in this record is CI's rather than the local gate's.

  That is left as it is. The gate has a five-minute budget on every host, and
  sanitized builds of these sources would spend a large part of it re-proving
  on a laptop what a per-push job already proves on a real Linux kernel;
  adding them buys latency, not coverage. What it costs is that a race or an
  overrun introduced between pushes is caught by CI rather than before the
  push, which is the trade this record is stating rather than hiding.

## Judgment calls

- **The default-route probe requires a branch of the policy, not the decline.**
  The obvious assertion — on a host running the POSIX adapter, some positioned
  read must have been declined — was written first and then failed on this
  machine under the address sanitizer while other work was running: sixteen
  thousand reads, none declined, three helpers started. That is not a defect.
  A sanitized one-byte read on a loaded machine really does cost more than the
  twenty microseconds the growth rule asks about, so the adapter measured a
  wait and took the policy's *other* branch, which is the correct answer to
  what it measured. Requiring the decline would have been requiring the host
  to be idle. Requiring either branch keeps the assertion about the runtime,
  and the negative control — the decline forced off in `bridge.c` — still
  fails it on every run.
- **The helper-storage case is given more array than it declares.**
  `test_helper_growth_stops_at_the_helper_storage` hands
  `wf_file_adapter_init` an array of twenty and tells it there are two. That
  looks like a weakened test and is the opposite: the bound under test is the
  capacity the adapter was told about, and with the true array at two a
  missing clamp does not fail the case, it runs `pthread_create` past the end
  of the frame and takes the process with it — which is what the first attempt
  did, hanging under the address sanitizer instead of reporting. The slack
  turns an undefined execution into a `check failed: held == 2` at a measured
  eight. Nothing beyond `helpers[1]` is written in a passing run.
- **The two cold rows went from `not read` to `no`, and only because a draw
  arrived that could read them.** They were graded `no` first, on a draw whose
  uncached label its own probe refused before both cold tables; that grade
  claimed more than the measurement supported and was replaced by `not read`,
  which is what the artifact allowed. Pushing this repair then ran `io-bench`
  and produced a macOS draw that confirms the label at both ends of both cold
  tables — the thing the record had been calling owed — and on it the rows
  read 1.477 and 1.557 against a bar of 1.10. So the grade is `no` again, now
  on a table that is actually cold, and the route between the two gradings is
  in the record rather than a silent flip: `not read` was never a softer way of
  saying `no`, it was the absence of a reading, and the reading when it came
  was worse than the mixture it replaced.
- **The bar's `after` column is not re-read from the later draws, but its cold
  grades are.** Three further draws exist — `72e98cba`, `a06c53f9` and
  `261070c8` — and two of them would move rows in the program's favour.
  Reading a bar row's *number* from whichever draw flatters it is the error
  this record was corrected for, so the `after` column stays on the run its
  tables come from and every other draw is reported in full beside it. A
  *grade* is a different thing: the two cold rows had no grade at all for want
  of a cold table, and `261070c8`'s draw supplies one — against the program.
  Taking a grade from the draw that hurts, while refusing to take numbers from
  the draw that helps, is the asymmetry this record is willing to defend.
- **The follow-up's runtime changes are recorded here rather than under
  "What shipped".** They land after the measurement and none of them changes a
  route, a policy or a threshold, so putting them among the changes the tables
  price would suggest the tables priced them. The fourth macOS draw taken at
  `a06c53f9` is what prices them, and it is reported as its own reading rather
  than folded into the before/after tables.
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
  rather than of the policy that may decline it. Without it `C.wide8.h0` would
  be measuring whatever the policy happened to choose, and the pair
  `h0`-against-`default` that prices the machinery at 7.92 ms over `S.wide8`
  would not exist.
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
- **The two cold rows are read and missed, and why is not measured.** On the
  one macOS table whose uncached label its probe confirms at both ends — run
  33172323795 at the repair commit — they stand at 1.477 and 1.557 times
  `N.pool8` against a bar of 1.10. What this batch does not have is an
  attribution of the gap on that draw: the stage table has a column for one of
  the two rows only, `cold 64K h8`, and it was taken on the earlier runtime.
  There it says the path charges about 5 us an operation against a 168 us host
  call, of which the wake latency alone — enqueue to a helper being scheduled
  to run the work — is 38.5 us. On a three-core runner eight helpers is more
  threads than cores, so that latency is the host scheduler rather than the
  adapter. There is no cold 4 KiB column, so the same reading is inferred for
  that row rather than measured, and neither column is on the draw the grade
  now comes from; closing either would need its own instrumented run.
- **Windows.** The IOCP adapter is untouched. `completion-windows` links and
  passes as before; none of the Darwin helper-path work applies to it.
- **The many-files workload.** It is recorded and it is still slower under C
  than under S. This batch narrowed it and did not close it; the reason is the
  one batch 0092 reached, that a 17 us `openat` is not a wait worth a handoff.
- **An explanation of the 8573C Linux draw.** The draw this record's tables
  come from is on hardware the earlier ones were not on, its cold half is
  unusable, and its warm half disagrees with the two prior readings. A fourth
  draw taken at `a06c53f9` — see "What the follow-up cost" — agrees with those
  two rather than with it, so the no-regression bar is not refuted; but the
  8573C reading is still unexplained, and the bar table keeps `unresolved`
  rather than borrowing a grade from a different draw on different hardware.
- **A confirmed cold macOS table on a quiet runner.** The confirmed label
  arrived — run 33172323795 confirms it at both ends of both cold tables — but
  that runner's load average was 5.60 and `C.wide8.default` spans a factor of
  14 inside the 64 KiB line. The grade it supports is `no`, and at 64 KiB even
  C's minimum is outside the bar, so the grade does not depend on the noise;
  at 4 KiB the two lines' ranges overlap and only the medians separate them.
  What a quiet confirmed draw would settle is how far outside the bar the cold
  rows really are, not whether they are outside it.
- **The growth path observed in a real program on a maintainer machine.** That
  a pool appears when operations wait and does not when they do not is pinned
  by two harness cases with a scripted clock, deliberately, so that the rule is
  tested rather than the machine; the evidence that it fires on a real program
  is the runner's cold tables, where `C.wide8.default` lands on its own pinned
  eight-helper line. A warm macOS page cache is exactly the case where the rule
  is meant not to fire, so this host cannot supply that observation.
- **A corpus program that reaches submit and join.** Counted over `whitefootc
  --emit-llvm` for the 22 units of `CORPUS_UNITS` in
  `compiler/tests/programs/parallel.rs` — 25 `.wf` files, since each
  `raw_deflate_*` unit compiles four — in the default lowering and again under
  `--par`, 44 modules in all: not one emits a completion `*_submit` or
  `*_join` call; every completion call the corpus emits is a `*_direct` one;
  and the two lowerings emit the same set for every unit. Five units emit a
  completion call at all. `byte_string` and `par_layout` emit
  `wf__completion_file_write_direct` and nothing else; `dir_walk` emits
  `open_at_direct`, `close_direct`, `directory_next_direct` and
  `write_direct`; `raw_deflate_boundary` emits
  `open_at_direct`, `close_direct`, `pread_direct` and `write_direct`; and
  `wfgrep` emits those four plus `directory_next_direct`. The other seventeen
  units — 34 of the 44 modules — emit no completion call whatever. So the
  overlap-versus-`--no-overlap` differential over that corpus covers the new
  direct routing and not the submitted path, and nothing in
  `compiler/tests/programs` can observe a declined positioned read, because a
  decline is a refusal returned from `wf__completion_file_pread_submit` and no
  corpus program calls it. The programs that do exercise it are the bench
  programs in `research/experiments/io-completion-bench/programs/`, and that
  is where the differential has to be run until a corpus program states
  overlapped I/O.

## After the tables

Two independent verifiers read this branch at `20b92e09` after the runs above.
One examined the runtime against the completion contract and confirmed it; the
other checked every figure in this record against the job artifacts and
refuted the prose. Both sets of findings are answered in this batch, and the
runtime changes are listed here rather than in "What shipped" because they
land after the measurement and none of them is a performance change.

The repair those findings produced was then re-read by two more verifiers, at
`6948c94e`, and the same split came back: the runtime was confirmed with
problems, and the prose was refuted again on new figures. That second round is
what the rest of this section answers. Its two substantive findings about the
tests are the ones worth stating plainly, because both are the same defect in
different places — a check that passes without deciding anything.
`test_helper_growth_stops_at_the_helper_storage` could not fail: its driver
completed each read before submitting the next, so the queue never held more
than one entry, the pool never passed one helper, and deleting the clamp it
exists to guard left the whole harness green. `bridge_default_probe.c`
counted its route split and asserted nothing about it. Both are fixed above,
and both fixes were checked by breaking the thing they guard and watching them
fail. The figures the second round refuted are corrected in place, in this
record and in `RESULTS.md`, and the claims about coverage that could not be
supported are withdrawn rather than softened.

- **The named drain refuses a token whose slot was reused.**
  `wf_completion_drain_token` was the only token-named entry that did not
  compare the token's generation with the slot's, so a retired token could
  take the event of the operation that reused its slot, leaving the real owner
  waiting for an event that no longer exists. No emitted program reaches it —
  the join is emitted only under `submitted == 1` and the bridge passes a live
  unconsumed token — but it is a defence every sibling entry has. The check
  and the take now happen together under the slot's publication lock, behind a
  relaxed no-event load, so the join loop's usual answer costs one load rather
  than the compare-exchange it used to.
  `test_named_drain_refuses_a_recycled_slot` fails on the previous runtime.
- **The adapter's readiness is published rather than merely written.** Routing
  the `*_direct` entries through `wf_file_execute_timed` made them read the
  adapter's `initialized` field, and a direct entry runs no once-control, so
  that read raced `wf_file_adapter_init`'s write on the thread making the
  first submitted operation — a race ThreadSanitizer reports in a two-thread
  probe (`Write of size 4 ... wf_file_adapter_init` against
  `Previous read of size 4 ... wf_file_execute_timed`). The field is atomic
  now, published by one release store at the end of init and read with an
  acquire load; the flag is cleared first so that a record left half built by a
  failed `pthread_mutex_init` never reads as ready. That release/acquire pair
  is the whole of what excludes the race, and the record used to credit the
  field-by-field assignment as well — which was wrong. `atomic_init` is a
  plain write by definition, and clang at -O2 on aarch64 merges these into
  wide stores anyway: one 16-byte store over `mean_execute_ns` and
  `execute_ticks`, two more over the four statistics counters. A `memset`
  there would be no worse in kind. A reader can see any of those writes only
  if a record it already holds is initialized again underneath it, which no
  caller does — the bridge initializes once under a `pthread_once` — and a
  probe that violates that precondition on purpose draws the corresponding
  ThreadSanitizer report. The comment at `wf_file_adapter_init` now says this
  rather than the old claim. The bridge's readiness flags are the same class,
  read by `wf__completion_file_pread_submit` before it reaches any once, and
  are atomic for the same reason.
- **The helper cap is bounded by the helper storage.** The growth rule writes
  `helpers[held]`, so a cap above the array's length was a `pthread_create`
  past its end. The storage length is carried and the cap clamped to it. The
  case that guards it now reaches the bound: see "The helper storage bounds
  the helper cap" under "Tests".
- **A failed clock ends the join spin, and it matters on one route.**
  `wf_bridge_monotonic_ns` answers zero when `clock_gettime` fails, and the
  spin's only bound was a clock reading, so the deadline was never reached.
  What that costs is not the same on both routes, and the comment used to
  claim it was. On the native ring the join never ends: a submitted read
  becomes a ready event only when `wf_bridge_progress` reaps the completion
  queue, and this spin never calls it, so the loop's other exit cannot fire —
  forcing `clock_gettime` to fail stops the Linux io_uring route dead while
  the guarded build finishes in milliseconds. On the POSIX adapter a helper
  publishes the completion, the other exit fires on its own, and the unguarded
  build finishes at the same wall time as the guarded one on macOS. The guard
  ships either way; the comment now says which route it rescues.
- **The shipped default route is tested, and the probe now asserts the
  decision rather than counting it.** `harness.c`'s `main` pins
  `WF_IO_HELPERS` for any run that named none, and `completion-test` names 0,
  1 and 4, so the demand-driven policy was reachable from no bridge test in
  the tree. `bridge_default_probe.c` is that arm: sixteen thousand positioned
  reads across four lanes with `WF_IO_HELPERS` unset, each of which must
  deliver the byte at its offset whichever route the bridge chose. It reads
  the route it took from the runtime's own submission counters — the native
  ring's and the POSIX adapter's — and requires a submitted positioned read on
  either route, and on the adapter route one of the policy's two branches: a
  declined read or a started helper. "Tests" above says why the decline alone
  cannot be required, and quotes the two runs of one binary that show it.

  What the probe reaches on each host is uneven, and worth stating rather than
  averaging. On Linux with io_uring the ring takes every positioned read, so
  the adapter branch is never reached at all and only liveness and the byte
  check are covered there. On macOS a quiet host declines about 15 800 of the
  16 000 and grows no helper; a loaded one grows helpers and declines none.
  Growth to a *bound* is not what this probe decides in any case — four lanes
  each hold one read at a time, so the queue rarely outruns the pool. That is
  decided by `test_pool_grows_when_operations_wait` and
  `test_helper_growth_stops_at_the_helper_storage`, which drive a queue the
  pool cannot empty, and priced by the runners' cold tables, where
  `C.wide8.default` lands on its own pinned eight-helper line.

  It is wired to `completion-test` and `completion-sanitize`, and it gives
  `io-hosts` a thread-sanitizer run over the bridge, which the isolated
  core/read probe by construction cannot provide.
- **The adapter stops answering "usable" before its lock is destroyed.**
  `wf_file_adapter_shutdown` destroyed the condition variable and the mutex and
  only then stored zero into `initialized`, so over that window the acquire
  guard every reader passes still answered yes while the mutex behind it no
  longer existed — and a reader admitted through it takes that mutex, because
  the decline check reaches `wf_file_adapter_queued`. The store now comes
  first, and unconditionally: a record whose lock has been destroyed is not
  usable whatever the joins reported. The window that remains is the one the
  precondition already covers, a reader that passed the guard before the
  store.
- **Two things were written down rather than changed.** A submission signals
  the queue's condition variable *after* releasing the queue lock, so a
  shutdown concurrent with a submission destroys the variable the submitter is
  about to signal; closing that window costs either the wake-inside-the-lock
  this batch removed or a second lock on the submission path, and no caller
  has the overlap — the bridge's only shutdown is its `atexit` handler. The
  precondition is now stated at `wf_file_adapter_shutdown`, and it names the
  decline check beside the submission, because those are the two entries a
  delivered program reaches while holding nothing. And the decline check takes
  the queue lock for its "nothing queued" term, so every positioned read that
  reaches the question pays an uncontended lock and unlock; that is now stated
  at `wf_file_adapter_transfer_runs_on_caller` beside what it saves.

### What the follow-up cost

The argument first, because it is what the changes were chosen against: per
operation they add a handful of loads that became acquire loads where they
were plain ones — one on the direct route, a few more on the submit route's
policy question — against one *fewer* atomic read-modify-write on the named
drain's common answer, which the join loop asks on every turn. No route, no
policy and no threshold changes.

Then the measurement, because an argument about cost is not one. Pushing the
follow-up ran `io-bench` on it: run
[33165141309](https://github.com/mbbill/Whitefoot/actions/runs/33165141309) at
commit `a06c53f9` — the last commit of this branch that changed the runtime
when this section was written — same `macos-14` label, same script, and a
fourth separate draw. Against the tables above, which are `266acf4f`'s:

```text
row                       266acf4f   a06c53f9   delta
warm 64 KiB  C/S            1.006      1.0085   +0.002
warm  4 KiB  C/S            1.028      1.0309   +0.003
many-files   C/S            1.022      1.0144   -0.008
cold 64 KiB  C/N.pool8      1.394      1.302    -0.092
cold  4 KiB  C/N.pool8      1.283      1.229    -0.054
```

Two separate draws of a hosted label agree to three thousandths on both warm
rows, which is inside the noise either of them carries, and the cold rows and
the many-files row move in the program's favour. Nothing here is a claim that
the follow-up made anything faster — these are different draws, and the cold
tables of this one are labelled no better than the others — but a change that
cost the path anything measurable would not reproduce the warm rows to three
decimal places. The tables above stay at `266acf4f` because that is where they
were measured; this is the shipped runtime's own reading beside them.

Its cache labels leave it unable to carry a cold bar like the rest:
`probe before the table: refused; probe after it: refused` on both cold
tables. Its spreads are not uniformly better either, and on the line the bar
reads they are much worse. At 64 KiB it is the tighter draw: `N.pool8` runs
427.78 to 447.04 around 434.33, a factor of 1.05 against `266acf4f`'s 1.09
(417.49 to 453.58), and `C.wide8.default` 553.98 to 596.73 against 558.77 to
644.12, 1.08 against 1.15. At 4 KiB it is the looser one: `N.pool8` runs
379.22 to 413.11 around 399.02, a factor of 1.09 against `266acf4f`'s 1.01
(379.97 to 383.57), and `C.wide8.default` — the line the cold bar is read
from — spans 487.42 to 4996.77 around a median of 490.57, a factor of 10.25,
against `266acf4f`'s 469.00 to 508.36 at 1.08. So this draw is short of the
label and, at 4 KiB, of the tightness as well; what carries its cold reading
is the median, not the range.

**The same run is also the Linux read draw this record says is owed**, and it
is the better half of the news. `bench-linux-read` landed on an AMD EPYC 9V74
with the tree on NVMe, and its uncached 4 KiB table is confirmed at both ends
— `probe before the table: confirmed; probe after it: confirmed`. It is not
the only one: four of this branch's Linux cold 4 KiB tables carry that label,
and reading the same row from all four is what the claim rests on rather than
on one draw's exclusivity.

```text
run          commit    processor    C.wide8.default  S.wide8  N.pool8  N.uring32
33153717709  96bb4778  EPYC 7763            1479.87  4227.07  1479.56    1469.81
33155821397  266acf4f  Xeon 8573C           1514.79  8973.99  1435.03    1268.13
33165141309  a06c53f9  EPYC 9V74            1216.03  4108.74  1482.48    1448.85
33172323795  261070c8  Xeon 8370C           1465.26  3469.10  1481.78    1441.72
```

All four report four CPUs; the 7763 and the 8370C ran on `sda1`, the 8573C and
the 9V74 on `nvme0n1p1`.

The reading used is `33165141309`'s 1216.03 ms, and what makes it the reading
rather than the flattering one is that it is the *only* row of the four where
the eight-wide program is faster than every native line — 3.38 times its own
sequential build, 1.22 times an eight-thread pool and 1.19 times a
hand-written 32-deep io_uring pipeline. On the other three confirmed draws it
is not: `96bb4778` puts it level with the native pool and the io_uring
pipeline (1479.87 against 1479.56 and 1469.81), `261070c8` puts it a shade
ahead of the pool and a shade behind the ring (1465.26 against 1481.78 and
1441.72), and `266acf4f` puts it behind both (1514.79 against 1435.03 and
1268.13). So the honest claim across the four is that the completion program
is level with a hand-written native pipeline on this job on three Linux hosts
and ahead of it on the fourth, and ahead of its own sequential build by 2.4 to
5.9 times on all of them; the 1216.03 row is quoted because it is the draw the
follow-up was measured on, and it is quoted beside the three that do not reach
it.

And its warm half does not reproduce the reading this record could not
resolve:

```text
draw           commit    processor     warm 64  warm 4   narrow 64  narrow 4
0092           6ac36126  Xeon 8370C     0.982    0.941     1.004      1.011
earlier        96bb4778  EPYC 7763      0.984    0.946     1.002      1.016
this record    266acf4f  Xeon 8573C     1.026    1.055     0.997      1.051
follow-up      a06c53f9  EPYC 9V74      1.010    0.989     0.998      1.015
repair         261070c8  Xeon 8370C     1.015    1.000     0.996      0.999
```

Three of the five draws put warm `C.wide8` at or under `S.wide8` at 4 KiB and
a fourth sits on it exactly. The last row is the one that changes the
argument. It is on a **Xeon 8370C — batch 0092's own processor**, the one that
read 0.941 — and it reads 1.000. So the 4 KiB ratio moves from 0.941 to 1.000
on the same processor model between two draws, which means the spread across
these rows is not the hardware being different: it is what a hosted runner
gives this pair from one draw to the next. That removes the framing the
earlier rows invited, that the 8573C is an outlier to be explained. What is
owed is not an explanation of one machine; it is a reading of this pair that
does not move 6 points between draws, which needs repeated draws on one label
rather than one more draw on a new one. The bar table above keeps its
`unresolved` grade for exactly that reason.

The many-files job does not improve, and it is not the machine that separates
the readings: `bench-linux` ran on an AMD EPYC 7763 in both of this batch's
first two draws — the EPYC 9V74 above is the `bench-linux-read` job's host in
the same run, not this one's — on an EPYC 9V74 in both of batch 0090's, and on
an EPYC 9V45 in the repair draw. It reads `C/S` 128.73/120.94 = 1.064 at
`a06c53f9`, against 1.058 on the other 7763 draw, 1.050 at `261070c8`, and
1.041 and 1.045 in batch 0090. The pair that differs most is same-processor,
and five draws on three processors now put C between 4.1 and 6.4 per cent
slower than S on that job, which is batch 0090's own finding and not a new one.

### A draw of the repaired runtime

Pushing this repair ran `io-bench` a fifth time, at `261070c8`: run
[33172323795](https://github.com/mbbill/Whitefoot/actions/runs/33172323795),
all three jobs green. It is reported here in full because the record's own
standard is that no draw of these jobs may be left out, and because two of its
three halves answer questions the earlier draws could not.

Its `bench-macos-read` job is the draw this record kept saying was owed: the
uncached label is confirmed *before and after* both cold tables, the first
time on this branch. "Against the bar" above takes the two cold grades from
it. Its other rows, for completeness — warm `C/S` 0.989 at 64 KiB and 1.028 at
4 KiB, narrow control 0.968 and 1.013, many-files 146.59/144.37 = 1.015 —
agree with the tables this record reports to within the noise those tables
carry, on a runner whose load average was 5.60 at the start.

Its `bench-linux-read` job lands on a Xeon 8370C with the tree on `sda1`, and
its uncached 4 KiB table is confirmed at both ends as well, which makes four
such Linux tables on this branch. On it `C.wide8.default` reads 1465.26 ms
against `N.pool8`'s 1481.78, `N.uring32`'s 1441.72 and its own sequential
build's 3469.10 — 2.37 times faster than the sequential build, 1.1 per cent
faster than an eight-thread native pool, and 1.6 per cent behind a
hand-written 32-deep io_uring pipeline. That is the fourth reading of that row
and the third of the four where the completion program is level with or
ahead of the native pool.

Its `bench-linux` job is the many-files draw folded into the paragraph above.

Nothing in this run is folded into the before/after tables: those are
`266acf4f`'s and stay that way. What it changes is two grades and one owed
item.

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
