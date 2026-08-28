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
per operation, nanoseconds:

```text
stage              warm 4K     warm 4K     cold 64K     what it is
                   h0          h3=default  h8
claim                   43           70           98    take a free slot
submit                  84        1,280        2,967    queue lock, two slot
                                                        transitions, notify
wake latency             —       10,791       38,533    enqueue to helper
execute              1,079        1,869      168,117    the host call
publish                 55        1,467        4,334    result, event, notify
drain                  222          302          584    find the event
consume                 53          101          176    result out, slot free
park (amortised)         0        1,779        6,158    announce sleep, wake
```

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

### One kilobyte less inside the queue lock

A queue entry holds an open's path bytes, and moving an entry out copied the
whole record — a kilobyte of path storage for every read and write as well,
inside the one lock every submission and every execution has to take. Only an
open needs its path, and now only an open pays for it.

### The wake goes outside the lock, and only to a sleeper

The enqueue decides under the queue lock whether a helper is actually asleep —
that lock is the one a sleeper counts itself under, so the answer is exact
rather than a guess — and the submission issues the signal after unlocking. A
helper that woke on its own in between only makes the signal spurious, which
its predicate loop already tolerates.

### A bounded look before a joining scheduler sleeps

Announcing sleep and being woken is two system calls, paid by the waiter and
by whichever thread publishes. A helper pool only exists when the adapter has
measured operations that wait, and those waits end while the joining thread
has nothing else to do, so it reads the ready-event count for a bounded window
before announcing sleep. It is a bound on wasted CPU, not a latency target: a
wait longer than the window still ends in a sleep.

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
milliseconds across runs are not**. The two draws differed by about a third on
every line, native baselines included.

