# 0095 — staged loop pipeline, Stage A prerequisites

Branch `batch/0095-loop-pipeline`, from `main` at `b2e2e267`.

The design is `research/investigations/io-model/LOOP-PIPELINE.md` §1, §3 and §7
"Batch 2", with its §9 probe results as the measurement of record, and
`research/investigations/io-model/FIRST-PRINCIPLES.md` §13-16 as the ownership
contract the runtime work has to keep.

Stage A lands the four prerequisites §3.9 and §7 name. **None of them changes a
permission verdict or a published byte**, and no writer-visible surface is
added anywhere: there is no attribute, environment variable, or source spelling
for a window, a depth, or a schedule. Stage B lands the judgment ([PAR-3] in
`compiler/src/semantic/staged_permission.rs`), the lowering, and the
measurement, and this record moves to `docs/done/` then.

## 1. `wf__completion_window(span, slot_bytes, ceiling)`

`compiler/src/backend/completion/bridge.c`. The runtime's own answer to how
many iterations of one loop it will carry in flight, asked once per loop entry
and never per iteration — the discipline `wf__par_split_budget` already keeps.

Each argument is a bound and none is a request; a zero places no bound. The
runtime's own bound is **half** its process-wide operation capacity, because a
loop owning every record would push the rest of the program onto the
capacity-wait path, where a full ring degrades to a blocking direct call
(§3.9 item 6). Half of 64 is 32, which is where the hand-written ceiling
program measured its optimum (§9.2). Where the ring is the target engine the
ring's capacity bounds it too; where a bounded helper pool is,
`WF_BRIDGE_MAX_HELPERS` does. A 4 MiB byte budget divided by `slot_bytes` is
the last bound, so the design's 16 MiB privatized buffer gets one.

**One is always a legal answer**: the runtime may carry one iteration in flight
and that is the schedule the sequential program already runs, so no answer this
query gives can make a correct program fail. That is why the weak fallback a
link without the completion unit gets returns one.

Stage A does not demonstrate it, and the record earlier said it did. The value
the emitter binds has no consumer yet: what carries and what drains is decided
entirely by the pipeline's block set, so answering one restricts nothing and
reproduces nothing. It is harmless here only because one call site still owns
one operation record, which keeps at most one operation of a site in flight
whatever the answer. The claim becomes checkable when Stage B's driver consumes
the answer, and checking it there is Stage B's evidence to produce.

The fallback is `COMPLETION_WINDOW_FALLBACK` in
`compiler/src/backend/emitter/completion.rs`, emitted only where a module
actually asks for a window — the same discipline `PARALLEL_SPLIT_BUDGET_FALLBACK`
already keeps, and the reason no module that stages no loop changes by a byte.

## 2. Deferred io_uring doorbell

`compiler/src/backend/completion/linux_io_uring.c`. Staging a submission-queue
entry is a store to a page the kernel already maps, so it costs no system call;
`io_uring_enter` is what tells the kernel the entry exists. Probe A measured
15,360 enters against 2,048 deferred, taking 15.6 ms of system CPU and 14.4 ms
of wall time off the eight-wide program — in a barrier-bound single-lane
program the saved kernel CPU is all on the critical path (§9.1).

The doorbell now rings from four places, and between them they are what makes
deferring safe:

- `wf_linux_io_uring_progress` kicks before it reaps or sleeps, and every join
  and park in the bridge reaches the kernel through it;
- the submission path kicks when the submission queue is full, before it
  declares a capacity wait, because a deferred queue fills with this thread's
  own backlog first;
- `wf_bridge_flush_target` kicks before every blocking direct host call and
  before the demoted-open path hands its caller back to one, so a submitted
  open can never sit unkicked behind a blocking `openat` (§3.9 item 3);
- the retire-and-retry path below rings the doorbell for the re-attempt it
  stages, while the thread that decided to stage it still owns the decision —
  including where the submission queue was full, which is the one case an
  earlier repair left staged behind a completion-only sleep.

`wf_linux_io_uring_statistics.submission_enters` counts them, and the distance
between it and `submissions` is the whole of what deferring buys.

## 3. Retire-and-retry on descriptor exhaustion

Design §2.10. A schedule that keeps several iterations in flight holds several
descriptors where source order holds one, so a host limit the sequential
program never reaches could turn a correct program's `Ok` into an
`Err(ResourceExhausted)`. [SYS-10] is explicit that a `FilePermit` promises no
native descriptor, and [PAR-1]'s exhaustion clause excuses only what an
implementation spends on overlapping — never the descriptors the program's own
opens consume.

The rule is about a *moment*, and it is one rule over one ledger.

The moment is the host attempt. A refused open must ask "has this runtime given
a descriptor back since I asked the host", not "is anything running now": an
operation that retired between the failing `openat` and the decision has
already made room, and a rule that read the present would answer from a world
where nothing is left in flight and publish an exhaustion that no longer
exists.

The ledger is process-wide, because a descriptor is. It holds two counts,
because the rule turns on two different facts and one count for both gets the
rule wrong. The first is how many descriptors this runtime has put back in the
host's table: a close completing, and an open whose descriptor the kind check
refused and the runtime disposed of. That is the only event that can make a
second host attempt answer differently from the first, so it is the only thing
that justifies spending the one re-attempt. The second is the in-flight count,
which every operation either engine accepts joins, blocking direct calls
included for as long as they execute, and which is what "still holding a
descriptor it could give back" is read from. An operation that ends without
returning anything — a read, a write, a status, a directory batch — moves the
second and not the first.

The ledger lives with the completion core (`contract.h`, `runtime.c`) because
the core is the one unit both target engines and the bridge link; the bounded
POSIX adapter and the Linux ring are qualified separately and share no other
code.

Every refused open, whichever engine attempted it, then follows four steps:

1. read the descriptor-return count before the host attempt;
2. on `EMFILE`/`ENFILE`, if that count has moved since, re-attempt once;
3. otherwise, while an operation is in flight anywhere else, wait: it may still
   give a descriptor back, so this refusal is not the answer yet;
4. otherwise nothing is left that could give one back, so this is the last
   moment at which a second attempt could see anything the first did not —
   re-attempt once here too, and publish what it says.

A returned descriptor is awarded in the order the waiters registered, which is
the order the publication rule already follows. Step 2 is what spends a refused
open's one re-attempt, so two refused opens must not spend theirs on the same
returned descriptor: while a waiter registered earlier is owed a re-attempt, a
later one's step 2 does not fire for that return. The earliest takes it and
re-attempts first, and only what it does not consume falls to the next — its
re-attempt fails, or the descriptor it takes is one the kind check refuses,
which disposes of it and is a return again. Source order gives the `Ok` to the
first open that asks for it, so an unordered award publishes the program's one
`Ok` at the wrong call. "Earliest" is registration order into the one ledger,
whichever route the open took: a ring entry registers where its refusal is
reaped, an adapter open and a direct call where their own attempt was refused.
A waiter keeps that place for as long as it waits — running the work it owes is
standing aside, not leaving.

That is the whole of the promise, and it is written out here because the
stronger reading of it is false and a program leaning on the stronger one would
be leaning on nothing. Registration order is the order the host refused the
opens, and the order the host is *asked* in is not this ledger's to decide: a
helper pool refuses one thread's queued opens in whatever order its helpers
reach them, and a scheduler visit deliberately takes the newest queued request
rather than the oldest, so at zero helpers a thread draining its own queue
attempts the last open it submitted first. Where one thread's opens are
attempted in the order it wrote them — one helper draining the queue from the
head, or the work a refused open runs as owed, which takes the head whichever
thread runs it — registration order is submission order and the first open the
thread wrote is the one that publishes the `Ok`. Across threads nothing is
promised and nothing can be; order between threads rests on the in-order commit
of outside-rooted writes the lowering performs, not on this ledger.

Step 4 attempts rather than publishes because a descriptor can come back from
outside this runtime: a thread of the program's own closing one while this
runtime carries the read that thread is answering. No ledger can see that
close, so the only honest moment to look again is the moment the ledger runs
out of reasons to wait.

Exactly one re-attempt per refused open, so an exhausted host cannot turn one
`open_file` call into unbounded work; if the second attempt also fails, that is
the outcome source-order execution produces and the program sees it. A retry is
counted exactly where the second host attempt is made, so a published refusal
never reads as one.

What this costs, stated as it is rather than as it would be convenient: the
*decision* is paid only on the exhausted path, but the ledger's counts are not.
`wf_completion_operation_accepted` and `wf_completion_operation_retired` sit on
every submission and every ending, and each is a sequentially consistent
read-modify-write plus a sequentially consistent load of the waiter count — two
atomics per operation on the ordinary path, whether or not an open is ever
refused. The wake past that load costs nothing where nothing is waiting, and
the give-up decision, its lock and its waiter order are reached by refused
opens only. T3 is about claim-related gates and holds unchanged: nothing here
is a claim and no claim's correct path gained a check. Whether those two
atomics are measurable beside a host call is not knowable on the host these
counts were taken on; the runners are what measure it.

Waiting terminates because a waiter counts itself out of "in flight anywhere
else": when every operation still in flight is a waiter, the earliest of them
answers, and leaving the waiter order hands that place, and the same answer, to
the next one. Two more things are counted out alongside the waiters, and
both are the same fact — an operation nobody may wait for, because waiting for
it would be waiting for the waiter. One is local: the operations this waiter's
own thread would have to run itself, which is the queue a drained sibling's
suspended caller still owes and is nothing at all for a held ring entry, since
that blocks no thread. The ledger asks the engine for that queue where it
decides, inside the lock every wake takes, instead of being handed a reading of
it taken earlier; the queue is a live fact, and a decision made on a stale
reading of it is the third deadlock recorded below. The other is about the
waiter itself: one whose thread has gone into another operation *stands aside*
for as long as it is there. It stays a registered waiter, because its operation
still cannot retire and that is what a registered waiter says; what it gives up
is the duty to publish, because the operation it is running may be an open
refused in its turn and registered behind it, which must be able to answer
rather than wait for the thread that is waiting for it. Its claim on a returned
descriptor it keeps: that claim is registration order, and standing aside is
not leaving.

How each route waits is mechanical, and it is the only per-route thing left.
The ring holds the refused entry — in a state no doorbell may stage — and asks
the ledger again on every progress pass; no thread blocks, and the doorbell for
a staged re-attempt rings immediately, including where the submission queue was
full, so a re-attempt never sits behind a completion-only sleep. The bounded
adapter's thread first runs the work its own queue still owes, oldest first,
which is the order the program wrote and the order that reproduces the
sequential outcome, and only then sleeps on the ledger's signal — and it never
sleeps while an item is still in that queue, because running that item is the
answer rather than a reason to wait for one. A blocking direct open waits by
driving the engines and parking on the runtime's own endpoint, because on a
target with a ring it may be the only thread that can reap the completion it is
waiting for.

Waiters therefore sleep in two different places, and that is a fact about the
ledger rather than about a route: a transition announces where it can leave
*another* waiter asleep on an answer that is no longer the best one available
to it — an operation accepted, an operation retired, a waiter registering, a
waiter leaving — and then in both places, on the ledger's own condition
variable and on the runtime endpoint. A thread never needs to wake itself, and
standing aside and coming back are silent: the thread that stands aside is
running rather than sleeping and comes back here to answer, and the waiter its
absence exists for registers afterwards and reads the ledger where it
registers. The ledger is given that
endpoint by the unit that owns the runtime, which is the bridge, and clears it
before the runtime is destroyed. A transition announced in one place only is a
stopped process, not a slow one, and it is recorded below as the fourth
deadlock.

### What this record said before, and what was false

Eight shapes of this were wrong before the rule was stated that way, and the
rule then stopped the process four times before it was right. None of the
twelve was caught by a test as written: the first two were found by the Stage A
verification, the next two by re-verification of the repair, the fifth by
reading the repair for the same class of defect, the sixth by a third
re-verification, the seventh and eighth by a fifth adversarial re-verification
of the fourth repair, and the four deadlocks by running the suite and the
verifiers' probes to the point where a stall is distinguishable from a slow
host.

- the ring re-staged the refused open at the end of the *same* reap pass, so
  the source order `close(held); open(path)` — which succeeds sequentially —
  lost the race to that close and published `Err(EMFILE)`, deterministically,
  at every helper setting;
- the adapter asked the host a second time only when it found queued work to
  drain. With several helpers each simultaneous open lands on its own helper
  and looks at an empty queue, so the descriptors a retirement would return —
  held by operations running on the *other* helpers — were never waited for and
  the refusal was published instead;
- **"an open the host refuses ... is not published while this runtime is still
  holding a descriptor it could give back" was false across engines.** Each
  engine asked only about its own operations. A program with a read in flight
  on the helper adapter and an open refused on the ring published `Err(EMFILE)`
  with no re-attempt at all, deterministically at zero, one and four helpers.
  That program is an ordinary one rather than a contrivance:
  `wf__completion_file_read_submit` has no ring route, so a read and an open
  submitted together are on two different engines by construction;
- **"Each of them then gets its one re-attempt" was false on the adapter
  route.** A refused open ran a *sibling* open out of its own queue and
  published that sibling's refusal with no second attempt at all, while
  spending its own on a close that had not finished. The source order
  `close(held); open(p); open(p)` produces exactly one `Ok`; that shape
  produced none, 81 times in 200 at four helpers and 74 in 200 at two on macOS,
  and 112 in 200 before the first repair;
- the give-up decision read the generation, then the in-flight and waiter
  counts. A retirement whose generation increment landed after the first read
  and whose in-flight decrement landed before the second was invisible to both,
  so the decision could publish a refusal a descriptor had already answered. It
  re-reads that count at that exit now, and the ledger's schedule point is
  named so a test can stand exactly between the two reads. Since step 4 became
  an attempt rather than a publication, that re-read no longer keeps the `Ok`
  by itself — the attempt does — and what it keeps is the ledger naming which
  fact ended the wait, which is what the schedule-point test observes;
- **"a descriptor came back" and "an operation ended" were the same count**,
  and they are not the same fact. A read finishing on a helper thread returns
  no descriptor, but it moved the generation, so every open the ring was
  holding spent its single re-attempt on it — before the close it was actually
  waiting for had finished — and then published `Err(EMFILE)`. Source order
  runs the read, then the close, then the opens, and one open succeeds; that
  shape produced none. The measurement is `close(held)` on the ring with a
  cross-engine read in flight and four opens racing, 1,000 repetitions per
  cell on an x86-64 Linux host with a real io_uring, the scratch file on
  `overlayfs` so that `IORING_OP_CLOSE` has a `flush` to run and is genuinely
  asynchronous.  Two independent runs of 1,000 repetitions per cell lost 7 and
  34 `Ok`s at zero helpers, 16 and 21 at one, 3 and 5 at two, and 7 and 28 at
  four; every cell with the read removed lost none, in both runs — the read is
  the whole discriminator. On a host whose close runs inline the
  kernel frees the descriptor before the opens it was staged with are
  attempted, so the schedule does not arise and the shape reports nothing;
- **"a close ended, so a descriptor came back" was false of a close the host
  refused.** Both engines answered yes for every close whatever the host said
  of it, so a close of a descriptor that was not open — `EBADF` — granted a
  refused open its single re-attempt for a return that never happened, and the
  refusal that open then published was the program's `Ok`. The shape is a full
  table with one close the host performs and one it refuses submitted together,
  four opens, and a second thread opening on the direct route, on `overlayfs`:
  9 and 10 lost `Ok`s per 2,000 repetitions at one and four helpers at the
  revision before this one, 13 and 5 per 2,000 at the one before that, and 0
  and 0 here. The commit that made this repair recorded that earlier revision
  as losing none on this shape; re-measured at 2,000 repetitions it does not,
  and this is the corrected count. Both answer sites read the outcome now: the
  shared predicate accepts a close only with `error_code` zero, and the ring
  only with a non-negative completion result, which is the fact the ring holds
  in its own entry;
- **"a descriptor came back, so this refused open may take it" was true of
  every waiter at once**, and the descriptor then went to whichever of them
  reached the host first. That is the earliest *submitted* refused open, and
  source order owes the `Ok` to the first open in source order that can use it.
  Three refused opens racing one returned descriptor — a companion whose kind
  check will refuse it submitted first, then two plain opens — published the
  `Ok` at the wrong call in 963 of 1,000 repetitions, identically at the three
  revisions before this one, with the one re-attempt per refused open spent in
  every case. It is the award that was unordered, not the kind check: the same
  companion in a shape where it is not itself refused loses nothing. The ledger
  already ordered *publication*; it orders the award the same way now, and the
  same shape publishes the owed `Ok` in 1,000 of 1,000;
- and the first version of *this* rule deadlocked, which is the failure mode a
  rule about waiting has. A refused open running work its own adapter owed read
  the size of that queue once, before it waited. The queue then grew — the
  program was still submitting — so the count of operations nobody could retire
  stopped matching the count in flight, and every helper waited forever for
  work every helper was waiting instead of running. Two things follow from
  that, and both are the rule rather than a patch on it: the queue is read
  afresh on every pass, because it is a live fact; and a waiting open is still
  one of its adapter's engines, so it leaves the waiter order, runs everything
  queued, and only then waits again. A submission wakes a waiter for exactly
  that reason.

  It was found by the verifiers' probe rather than by the suite, and the reason
  is worth keeping: the harness names the adapter's `openat` so a test can
  stand at a refusal, and that named call took a global lock on *every*
  refusal, which serialized the very schedule the rule is about. The
  observation no longer perturbs it — the armed flag is read without the lock —
  and with that changed the suite reproduces the deadlock too;
- and it stopped a second time, for the other half of the same reason. A held
  ring entry is released by whatever thread next asks the ledger, and on a
  target with a ring that thread is a scheduler parked on the completion
  endpoint. What wakes it is a publication. A retirement is recorded just
  after the publication it belongs to, so the woken thread could ask in the
  instant before the answer landed, find nothing, and park on a wake that was
  never coming a second time — twice in four hundred runs at one helper in the
  Linux container, as a hang rather than a wrong answer.

  Recording the retirement *before* the publication removes that, and is
  wrong: `test_open_exhaustion_waits_for_another_engine` then fails four times
  in four hundred, because a program is entitled to find an operation's result
  published once the retirement that released its own open has happened. The
  order stands, and the retirement announces itself on the scheduler's own
  endpoint instead — only where the ledger says something is waiting for one,
  so the ordinary path pays one atomic load and no wake at all.

- and it stopped a third time, on the last input the decision still read
  outside its own lock. A refused open handed the ledger the size of the queue
  it was answerable for, and the ledger then decided under the lock every wake
  has to take. A submission landing between that reading and that lock made the
  reading too small; the wake it sent had already passed by the time the waiter
  reached the condition variable, and the answer "something else is running"
  became permanent. One helper asleep in the ledger, work in its queue, the
  main thread parked in the join, and no CPU consumed by any of them over
  fifteen seconds of watching. It appeared once in a twenty-run TSan sweep at
  one helper on macOS and once more in two hundred and ten scripted repetitions
  of the same configuration.

  The rule is now that every input the decision reads is read where the
  decision is made. The ledger asks the engine for that queue through a
  callback it calls inside its own lock, and the callback takes no lock of its
  own — the adapter's queue count is atomic, so it needs none, and the order
  every submission takes cannot be inverted. Work arriving while a waiter is
  deciding is then either seen by that decision or announced after the waiter
  is already on the condition variable, which is the difference between a wake
  that is unlikely to be missed and one that cannot be. The same callback says
  the other half of the fact: a thread that will run that queue itself does not
  sleep while an item is in it, because running the item is the answer. The
  reading passed in as a parameter is gone.

- and it stopped a fourth time, on the half of the wake the ledger could not
  reach. Waiters sleep in two places, and every ledger transition broadcast on
  the condition variable only. A direct-route open evaluated its state while
  one operation was in flight elsewhere, answered "keep waiting", and parked on
  the runtime endpoint with no bound; a second waiter then registered, which
  counted one more operation out of "in flight anywhere else" and turned the
  parked waiter's answer into "attempt and publish", and `wait_begin` said so
  where that waiter could not hear it. Three threads, no CPU, indefinitely: one
  observed process sat at 0% for eleven minutes and fifty-eight seconds before
  it was killed, its ledger reading 102 returns, 2 in flight, 2 waiters, 1
  deferred, with the parked waiter first in the order and its own `seen` at
  102 — every input for `UNREACHABLE`, evaluated when the second waiter did not
  yet exist. `wait_end` and `defer_begin` had the same hole. Four stalls in
  twenty runs of the shape at one helper, and more at four; the same shape with
  that one park bounded to five milliseconds stalled none in twenty, which is
  what makes it a missed wake rather than progress that was genuinely
  impossible. The bound is not the repair — a rule that needs a timeout to
  finish is a rule that does not know when it is done. Every transition that
  can leave another waiter asleep on a stale answer announces in both places
  now; the shape is a probe of its own where an overlay can be mounted, and
  `test_a_registration_wakes_a_waiter_parked_on_the_endpoint` is the same
  schedule scripted, with nothing asynchronous in it, which the gate runs
  everywhere.

Each of the three routes is covered by a test that fails without the fix, and
so are the third exit, the deadlocks and the moment the queue is read:
`test_bridge_open_waits_for_the_other_engine`,
`test_bridge_one_of_two_opens_behind_a_close_succeeds`,
`test_bridge_open_behind_a_submitted_close_succeeds`,
`test_open_exhaustion_waits_for_another_engine`,
`test_a_retirement_between_the_ledger_reads_is_not_missed`,
`test_the_work_a_waiter_owes_is_read_where_it_decides`,
`test_an_ending_that_returns_nothing_grants_no_reattempt`,
`test_a_returned_descriptor_is_awarded_in_waiter_order`,
`test_a_waiter_that_stands_aside_keeps_its_claim`,
`test_a_waiter_behind_one_that_stands_aside_can_answer`,
`test_a_registration_wakes_a_waiter_parked_on_the_endpoint`,
`test_descriptor_return_follows_the_outcome`,
`test_a_ring_close_counts_a_return_only_when_it_ran` and
`test_bridge_every_record_holding_a_refused_open_publishes`.

Two of those are about the same predicate on the two engines that answer it,
and one without the other is coverage that looks complete and is not: the first
calls the shared inline answer, which is the bounded adapter's half, and a ring
that counted every close it completed passes it — and the whole suite — while
mis-accounting a return on every refused close. The second stands where only
the ring can answer, reading the process-wide return count across a close the
kernel performs and one it refuses. The fourth deadlock has
`test_a_registration_wakes_a_waiter_parked_on_the_endpoint`, which is the same
kind and needs nothing asynchronous from the host: it narrows the descriptor
table so the host refuses an open outright, holds one operation in the ledger
by hand so the refused direct call parks rather than publishes, and then
registers the second waiter from the test's own thread — the transition that
turns the first waiter's answer into "publish" and reaches it only on the
runtime endpoint. With `wait_begin`'s endpoint announcement removed the direct
call never returns and the test's bound fails the run at one and four helpers;
the shipped runtime publishes the refusal at both. `retirement_interleave_probe`
stays beside it as the racing shape, which the gate runs where an overlay can
be mounted.

A deadlock is now a failure mode of this suite, so the harness has a watchdog:
three hundred seconds, three orders of magnitude above what the whole suite
takes, and it names the test that stopped rather than leaving a build job to
time out with an empty log.

**What the runtime cannot do, and Stage B must.** §2.10 says the adapter
"completes every older slot in index order (which runs their compiler-derived
closes)". A compiler-derived close is writer code, and completion never invokes
writer code (FIRST-PRINCIPLES.md §14), so the runtime can only give back the
descriptors held by operations *it* still owns — a close already submitted, not
one the driver has not reached. That covers the pipeline's steady state, where a
slot's close is submitted when its read retires and is often still in flight
when the slot's next open is. It does not cover a window whose older slots have
not been driven that far, so the driver has to retire its own slots in index
order on a published `ResourceExhausted` before it treats one as the program's
answer. Stage B owns that half.

## 4. Back-edge-tolerant joins

`compiler/src/backend/emitter.rs` and `emitter/completion.rs`, opting in on
`IrFunction::completion_pipeline` (`compiler/src/lowering.rs`). One rule
decides both halves of the staged schedule:

> A block the pipeline names never joins. Every block it does not name joins
> everything outstanding, in hand-out order.

The first gives a loop's back edge the right to carry work across it — today's
unconditional `emit_all_completion_joins` in `emit_terminator` is the whole of
the round barrier §3.4 measures. The second is the drain, and it needs no
separate machinery: retiring every outstanding operation in hand-out order is
what a block has always done.

The one thing a straight-line walk gets wrong is that a carrying region has
several exits — the loop's normal exit and every typed exit out of its body —
and each of them must retire the operations that reach it.
`pipeline_outstanding` records what the region handed out, together with the
block that handed it out, and each exit is given exactly what the carrying
blocks that actually reach it left in flight. Which blocks those are is read
from the edges (`pipeline_feeder_blocks`), not from how far the walk has got, so
a drain retires the same window however the blocks around it are numbered.

That is both halves and for every drain, the first one the walk reaches
included. The walk arrives at an exit holding the straight-line simulation,
which on a branching region is both too little and too much: an operation
started on a branch it has not passed is missing, and one started on a
*sibling* branch that cannot reach this exit at all is still in hand. Adding the
first without removing the second would emit a join for an operation no path
through that block ever started, reading a token no target ever wrote. An
earlier repair seeded only the later drains and left the first with whatever the
walk had; `a_drain_retires_only_what_the_branches_that_reach_it_started` is the
test, and with the removal deleted it shows that exit joining two operations
where one branch reaches it.

Two properties of the descriptor are refused outright as
`BackendFailure::UnretiredCompletionOperation`, both before a line of the
function is emitted:

- a carrying set no exit leaves, which has a path on which an accepted
  operation is never joined at all;
- a carrying block that starts an operation and is numbered at or after one of
  its own drains. Blocks are emitted in index order, so that drain is walked
  into before the hand-out it must retire exists and would emit a bare
  terminator with the operation still owned by a target — silently, because a
  function carrying a pipeline is exempt from the straight-line check at the
  end of emission, exactly because a carrying block may legitimately be the
  last block emitted. A latch numbered after a typed exit it reaches is fine
  and stays admitted: a block that starts no operation leaves that exit nothing
  to be missing. Stage B has to produce a descriptor whose hand-outs precede
  the exits that retire them, or teach the emitter to write its blocks in an
  order that does; the refusal is what keeps the difference from being silent.

`emit_stackless_root` does not consult any of this, and does not need to:
`StacklessPlan::build` admits only a single-block function ending in a return,
and a staged loop has a loop. If Stage B ever widens either side, the two paths
have to be reconciled rather than left to disagree quietly.

What this does **not** yet carry is the per-slot storage index. One call site
still owns one operation record, so a site inside a carrying region that
submits again while its earlier operation is outstanding is refused rather than
handed the first operation's storage. That is the driver's work (§3.4, §3.6
item 2), and the driver is Stage B.

Neither does it carry the wait a K-slot ring needs. The rule above says a
carrying block never joins, which is right for the back edge and not enough for
the body: reusing slot *i* means waiting for exactly the one older operation
that holds it, and nothing else. Stage B has to add a selective
single-operation join for that — a primitive the rule admits, since it retires
one named operation rather than everything outstanding — and must not reach it
by exempting the carrying block from the rule.

## Evidence

- Harness (`compiler/src/backend/completion/harness.c`), run by
  `make -C compiler completion-test` at four helper settings and, on Linux,
  once more with `WF_REQUIRE_LINUX_IO_URING=1`:
  `test_completion_window_answers_at_the_boundaries`,
  `test_a_submitted_operation_is_kicked_before_it_waits`,
  `test_open_exhaustion_retires_owned_work_and_retries`,
  `test_open_exhaustion_waits_for_another_engine`,
  `test_bridge_open_exhaustion_is_retried_once`,
  `test_bridge_open_behind_a_submitted_close_succeeds`,
  `test_descriptor_return_follows_the_outcome`,
  `test_a_ring_close_counts_a_return_only_when_it_ran`.
- `compiler/src/backend/completion/retirement_interleave_probe.c`, run by the
  same target at one and four helpers: a refused open on every route at once,
  250 repetitions a run, with a watchdog of its own because the defect it
  stands for is a stopped process rather than a wrong answer. It mounts an
  `overlayfs` for its file, because the ring runs a close on an ordinary
  filesystem inline and the window then never opens, and where it cannot mount
  one it prints that and exits 77, which the Makefile treats as the skip it is.
  It is a probe rather than a harness case because it narrows its own
  `RLIMIT_NOFILE`, which the harness process cannot do without changing every
  case that runs after it.

  What that skip costs is worth saying plainly: mounting an `overlayfs` needs a
  privilege, so on a host or a job that lacks it the probe reports the skip and
  the schedule goes untested there. On the host these counts come from it
  passes 25 runs of 250 repetitions at one helper and 25 at four when run
  directly, and skipped inside canonical `make check`, which is run here under
  a wrapper that drops exactly the capabilities the mount needs. A process can
  also mount one unprivileged inside a user namespace of its own, which this
  kernel allows and some hardened ones refuse; that is machinery the probe does
  not have, and adding it would trade a certain skip for an uncertain one.
- Backend (`compiler/src/backend/tests/completion.rs`):
  `a_staged_loop_carries_completion_across_its_back_edge`,
  `the_window_fallback_is_emitted_only_where_a_module_asks_for_one`,
  `a_carrying_region_with_no_exit_is_refused`,
  `a_drain_emitted_before_the_hand_out_it_retires_is_refused`.
- Every test named above fails without the fix it covers, which is what makes
  it evidence rather than decoration. Each control is the shipped harness with
  one hunk removed, built into a scratch copy so no other agent's tree moves.
  `test_bridge_open_waits_for_the_other_engine`, with the ring restored to
  deciding from its own in-flight and held counts, fails at zero, one and four
  helpers in the Linux container on `value >= 0` — the open publishes
  `Err(EMFILE)` where source order produces the descriptor.
  `test_bridge_one_of_two_opens_behind_a_close_succeeds`, with owed work
  restored to running with no re-attempt of its own, fails at two and four
  helpers on macOS on `successes == 1` — both opens refused where source order
  produces one `Ok`.
  `test_a_retirement_between_the_ledger_reads_is_not_missed`, with the
  generation's re-read at the give-up exit deleted, fails at zero, one and four
  helpers on macOS: the decision answers `UNREACHABLE` where a descriptor had
  already come back.
  `a_drain_retires_only_what_the_branches_that_reach_it_started`, with the
  sibling removal deleted, reports the first exit joining two operations where
  one branch reaches it.
  `test_an_ending_that_returns_nothing_grants_no_reattempt`, with the two
  counts merged back into one, fails on the second of its three states: the
  read's ending answers `HAPPENED` where the close it is waiting for is still
  in flight.
  `test_the_work_a_waiter_owes_is_read_where_it_decides`, with the ledger
  reading that queue once when the waiter registers instead of asking for it
  where it decides, fails at zero, one and four helpers on macOS on
  `state == WF_RETIREMENT_UNREACHABLE`: the ledger waits for a retirement no
  operation can produce, which is the shape that stopped the process.
  `test_bridge_every_record_holding_a_refused_open_publishes`, with the owed
  queue read once instead of on every pass, stops at four helpers on macOS in
  three runs of five and the watchdog names it; the same control hangs the
  verifiers' `attack_probe` in six runs of six. Two other controls separate the
  cause from its neighbours: with a submission no longer waking a waiter, and
  with the owed queue run once instead of on every pass, the probe passes six
  of six — so the live read is the fix and the other two are the rule it needs
  to stay one.
  `test_a_waiter_that_stands_aside_keeps_its_claim` fails at one and four
  helpers against two different alternatives: a ledger that passes a waiter
  standing aside over for the award, which hands the descriptor to the later
  open, and one that lets a waiter standing aside keep the duty to publish,
  which leaves the later open waiting for the thread that is running it.
  `test_a_waiter_behind_one_that_stands_aside_can_answer` fails against the
  second of those on its own.
  `test_a_registration_wakes_a_waiter_parked_on_the_endpoint`, with
  `wait_begin`'s announcement on the runtime endpoint removed, fails at one and
  four helpers on `finished != 0`: the direct call is parked with nothing left
  to wake it, which is the fourth deadlock in a schedule that needs no
  asynchrony to reach.
  `test_bridge_open_waits_for_the_other_engine`, with the retirement no longer
  announcing itself on the scheduler's endpoint, stops six times in four
  hundred runs at one helper in the Linux container, where the shipped runtime
  stops none in four hundred. That control is a rate rather than a certainty
  because the window is the few instructions between a publication and the
  retirement it belongs to, which is also why the defect it stands for took
  four hundred runs to find rather than twenty.
  `test_open_exhaustion_waits_for_another_engine`, with the adapter's old
  `drained == 0` give-up restored, fails 30 of 30 runs at zero, one and four
  helpers on macOS.
  `test_bridge_open_behind_a_submitted_close_succeeds`, with the ring restored
  to `14c89cf3`, fails 21 of 150 runs at one helper and 21 of 150 at four in
  the Linux container, where the shipped ring fails 0 of 150 at each under the
  same load in the same minute. That one control is a rate and not a certainty
  because the kernel is free to run the submitted close before the open, in
  which case the open is never refused and the old code has nothing to get
  wrong.
  `test_a_returned_descriptor_is_awarded_in_waiter_order`, with the award's
  order and its ration removed so that a return that has come back answers
  every waiter, fails on the first thing it asks of the second waiter: that
  waiter answers `HAPPENED` for a descriptor owed to the one registered before
  it, which is the state that sends both to the host for the same descriptor.
  `test_descriptor_return_follows_the_outcome`, with the close's outcome
  removed from the shared predicate so that every close answers yes, fails on
  its second assertion: a close the host refused with `EBADF` answers that a
  descriptor came back.
  `test_a_ring_close_counts_a_return_only_when_it_ran`, with the same half
  removed from the ring's own answer — `completion_result >= 0` deleted, which
  is byte-identical to the revision before the repair there — fails under
  `WF_REQUIRE_LINUX_IO_URING=1` on the refused close's return count, where the
  shipped harness exits 0. That control is what the test exists for: before it
  was written, a harness with only the ring's half removed passed 25 of 25 runs
  at each of zero, one, two and four helpers and passed the
  `WF_REQUIRE_LINUX_IO_URING=1` run, while mis-accounting one return per
  repetition of the shape that submits both. The other test calls the shared
  inline answer directly, so it cannot see the ring's.
  `retirement_interleave_probe`, built against the revision before the ledger
  announced on both endpoints, publishes fewer opens than source order owes in
  10 of 30 runs at one helper and 10 of 30 at four, where the shipped runtime
  loses none in 25 of 25 at either count; the same shape under the verifiers'
  own probe stops the process outright, which is the failure the watchdog is
  for.
  `a_drain_emitted_before_the_hand_out_it_retires_is_refused` emits a module
  with the ordering check removed, and the loop's `break` exit in that module
  carries a bare `ret` with no `wf__completion_file_join` while the
  later-numbered typed exit has one.
- The two probes the verifiers wrote are kept and re-run rather than replaced:
  they are the shape a harness test cannot reach, a whole process narrowing its
  own `RLIMIT_NOFILE`. At this revision, at zero, one and four helpers on both
  hosts, `attack_probe` passes every case — including `A1` (`oks=1 errs=1`),
  `A3` (four refused together, four refusals, four re-attempts — one each,
  since step 4 attempts), `A4` (the
  cross-engine open, `value=8 error=0` on Linux where it was `-1/EMFILE`), `A5`
  (64 opens against a full table) and `A6` (300 repetitions, `lost_ok=0`) — and
  `verify_probe`'s deferred-doorbell and `close(held); open(path)` cases pass
  as before. The two-opens probe reports `lost_ok=0` in 200 repetitions at
  zero, one, two and four helpers on both hosts, against 81 in 200 at four and
  74 in 200 at two before this rule.
- Negative controls run by hand on Linux: restoring the immediate kick fails
  the doorbell test, and disabling the ring's retry fails the exhaustion test.
- `make -C compiler completion-tsan`, new here and wired into the io-hosts
  Linux job. `completion-core-read-tsan` links neither the bridge nor the ring
  by design, so the deferred doorbell's staging, the retire-and-retry hand-back
  and the readiness flag the flush reads had nothing checking them. It now
  names its hooks through `COMPLETION_DEFINES` and
  `COMPLETION_HARNESS_DEFINES` rather than a hand-written list of its own,
  which is what the comment on those variables already required of every build
  that links the harness. The hand-written list left out the scripted clock, so
  on a host fast enough for the unscripted one the growth rule's measured half
  read `SHORT` where the test scripts a millisecond, and the sanitizer build
  was running a different runtime from the ordinary one. That held at the
  previous revision too; it is a build-list defect, not a runtime one.
- What the sanitizers actually say, replacing an earlier claim in this record
  that the harness ran clean under the thread sanitizer on macOS at every
  helper setting. At `14c89cf3` it did not:
  `test_bridge_open_exhaustion_is_retried_once` failed 13 of 200 runs (6.5%) at
  `WF_IO_HELPERS=4` in the `completion-test` build and 15 of 20 under
  `completion-tsan` — a flake inside canonical `make check`, since
  `completion-test` is one of its prerequisites and runs the harness at four
  helpers.

  Two separate defects fed that one symptom, and it took both fixes to clear
  it. The first is the adapter's give-up above. The second is the test itself:
  it asserted that a re-attempt had happened, which is not a property of three
  independent opens under the corrected rule, because a helper pool that
  finishes each open before the next is submitted holds nothing at the moment
  of each refusal and publishing the refusal unchanged is then correct. That
  assertion is now the bound the rule does promise — at most one re-attempt per
  refused open — and the property it was reaching for is carried by the two
  deterministic tests above. Its own comment says so.

- The direct route's wait was a spin, and it was the waiting thread's own
  announcement that made it one. A blocking direct call refused an open waits by
  driving the engines and parking on the runtime's endpoint: it reads the wake
  epoch, asks the ledger, and parks on that epoch while the answer is still
  "keep waiting". Between the two reads it told the ledger it was going into the
  engines, and that announcement moved the very epoch it was about to park on,
  so every park returned at once. Instrumented at the park call, the revision
  before this one turned that loop 21,255 times over four cells of the
  interleave and cross-route shapes and reached a park with a live epoch **not
  once** — 21,127 stale, 0 fresh. Nothing stalled and nothing was lost; it burnt
  a core to do it, and a spin is not a wait.

  What the repair is derived from rather than patched around is the
  announcement rule, now stated in `contract.h` and at the announcement itself:
  a transition announces where it can leave *another* waiter asleep on an answer
  that is no longer the best one available to it, and then on both places a
  waiter can sleep. A thread never needs to wake itself. And a transition
  nothing sleeps through owes nothing — standing aside is one, because the
  thread that stands aside is running rather than sleeping and comes back to the
  ledger to answer, so every answer its absence creates is still reachable when
  it does; the waiter its absence exists for is registered by the operation it
  is running, after it, and reads the ledger where it registers. A waiter left
  asleep by a silent transition is one whose answer is "something else is still
  in flight", and that something announces when it retires.

  So both halves of standing aside are silent, the epoch is read after
  everything this thread does to the ledger and before the state it parks on,
  and the same instrumentation at this revision counts 745 parks that slept and
  11 that returned at once because another thread had moved the epoch, over
  1,406 turns of the loop — fifteen times fewer turns for the same work, with 0
  stalls and 0 losses in 50 runs of 50 repetitions at one and four helpers.

- A waiter that runs the work it owes keeps its place. The bounded adapter's
  refused open ran its queue between decisions by leaving the waiter order and
  registering again on the way back, which put it behind every open refused
  meanwhile — and the ledger then handed the returned descriptor to the later
  open, exactly the defect the award order exists to prevent. On a scripted
  shape where an adapter open registers first and a direct-route open second,
  the later one took the descriptor in 200 runs of 200 at one helper and 195 of
  200 at four; with the place kept it is the earlier one in 200 of 200 at both,
  and the same shape with the registrations the other way round holds at 0 of
  200 wrong where it was 9 and 4 wrong before.

  The mis-award is not only a wrong publication. In that same scripted shape at
  four helpers the previous revision stopped outright in one run of the sweep:
  its ledger read 154 returns, 2 in flight, 1 waiter and 154 awarded, with the
  adapter's waiter asleep on the ledger at `seen = 153` — the return it was owed
  had been spent by the waiter that took its place while it ran the work it
  owed, and the operation it then waited for was the one the test only releases
  after that open publishes. A descriptor awarded to the wrong open is a wake
  nobody is left to give.

- Standing aside is now a fact about the waiter, not a count beside it, and that
  removed a double count. A deferred operation used to be a separate tally that
  a decision subtracted from what is in flight; a blocking direct call was both
  a registered waiter and a deferred operation at once, so every other waiter
  subtracted it twice and could conclude that nothing else was running while
  something was. A waiter standing aside is now counted once, where every
  waiter is counted, and what it gives up is the duty to publish — because the
  operation it is running may be an open refused in its turn and registered
  behind it, which must be able to answer rather than wait for the thread that
  is waiting for it. What it does not give up is its claim on a returned
  descriptor: that claim is registration order, and standing aside is not
  leaving.

- Two shapes the fifth and sixth verifications recorded as shortfalls are not
  promises, and the record says which. `regrace` holds several descriptors,
  submits that many opens, then submits the closes, and asks for an `Ok` from
  every open; sequential execution runs the opens before the closes and refuses
  every one of them, so the shape asks for more than source order produces and
  its "shortfall" at both revisions measures nothing. `srcorder` asks a
  different and real question — given that one of several refused opens gets the
  descriptor, which one — and the answer is the promise stated above. At one
  helper, where a single thread's queued opens are refused from the head in the
  order it wrote them, it is the first of them in 1,000 runs of 1,000, with both
  waiters in the order together every time; at the revision before this one the
  two never coexisted at all in those 1,000 runs, because the first left the
  order before the second could register, so the shape had nothing to measure.
  Where the opens are not refused in the order they were written — four helpers
  racing for the queue, or a scheduler visit taking the newest queued request
  first — no order is promised, and none holds: at four helpers the publisher is
  a later-submitted open in 198 runs of 200, because the ledger honours the
  order the host refused them in and that is what the helpers decided.

- Repetition counts. These were measured on an x86-64 Linux host (kernel 6.18,
  real io_uring, GCC 14) at **this** revision, every racy shape with its file on
  an `overlayfs` so that `IORING_OP_CLOSE` is genuinely asynchronous:
  `completion-test` at zero, one and four helpers plus the `WF_IO_NOCACHE` arm,
  green; 200 harness runs each at zero, one, two and four helpers, 0 failures
  and 0 stalls; `completion-tsan` 20 runs at each of the same four, 0 failures
  and 0 stalls; `completion-sanitize` (ASan + UBSan) green at zero, one and
  four; the `WF_REQUIRE_LINUX_IO_URING=1` run green; `attack_probe` and
  `verify_probe` 25 runs each at zero, one, two and four helpers on both
  `overlayfs` and the host's own filesystem, 400 of 400 passing; the
  cross-engine-read shape at 1,000 repetitions per cell at the four helper
  counts with the read present and absent — 0 lost `Ok`s in all eight cells,
  against 7 to 34 per thousand with the read present before that rule; the
  refused-open-on-one-route, refused-close-on-the-other shape at 1,000
  repetitions at each of the four, 0 lost against 971 and 973 per thousand at
  the two revisions before the outcome-gated predicate; the failing-close shape
  at 2,000 repetitions at one and four helpers, 0 lost; the award shape — three
  refused opens racing one returned descriptor — at 1,000 repetitions at one
  and four helpers, 1,000 owed `Ok`s of 1,000 in both with 2,994 re-attempts
  for the 2,994 opens the host refused, against 37 and 44 `Ok`s before the
  award was ordered; the interleave shape at 50 runs of 50 repetitions at one
  and four helpers, 0 stalls and 0 losses, against 5 stalls in 20 runs at one
  helper and 19 in 20 at four at the revision before the ledger announced on
  both endpoints; and `retirement_interleave_probe` 25 runs at one and four
  helpers, all passing, plus one run under the thread sanitizer.

  One nonzero exit of the thread-sanitizer harness is on this record without an
  explanation: it appeared once in an early twenty-run sweep at two helpers
  during the fifth verification, with three jobs running at once, and never
  again in 240 further runs across the four helper counts, nor in the 80 runs
  above. The log was overwritten before it could be read and the sweep that
  produced it counted a 300-second timeout as a failure, so it cannot be told
  apart from load. It is recorded as unreproduced rather than as nothing.

  Two helpers is in the list because that is where the two-opens defect showed
  at 74 in 200 and the shipped suites do not run it; one helper matters because
  that is where three of the four missed wakes showed — one twice in four
  hundred, one once in twenty under TSan, and the fourth five times in twenty
  runs of its own shape, where four helpers showed it nineteen times in
  twenty. The earlier counts in this record were measured on
  macOS and in an aarch64 Linux container at earlier revisions; they are what
  those revisions did, and the macOS half of this revision is `io-hosts`' to
  re-measure, because the host that produced the numbers above has no macOS.
  That aarch64 container's TSan needs ASLR off (`setarch -R`) or it aborts on
  every run with `unexpected memory mapping` before reaching a test; that is
  the host, not the suite.
- `make -C research/experiments/io-completion-bench verify` on macOS: every line
  of every workload — the hand-written native shape, the `--no-overlap`
  sequential reference, and the shipping overlapped build — publishes
  `17098009301725298919 00000000000071024640`, `many_files_loop` among them.
  The runtime changes here are on the exhausted path; this is the check that
  they left the ordinary one publishing the same bytes.
- Toolchain coverage. `gate-linux` builds the harness with the host `cc`, which
  on `ubuntu-24.04` is GCC, while `io-hosts`'s `completion-linux` builds it with
  clang. A first push of this repair compiled on macOS clang and on container
  clang and still failed `gate-linux`, because GCC rejects a discarded `write`
  under `-Werror` where clang does not. The harness is now also built and run
  with `gcc` in the Linux container, which is the same diagnostic set the gate
  applies.
- IR identity: every `.wf` under `tests/programs`, `tests/codegen` and
  `tests/conformance/cases` compiled with `whitefootc --emit-llvm` at this
  revision and at `main` (`10b76c66`), under the default, `--par` and
  `--no-overlap` — 630 sources, 1,890 compilations each side, 807 of which
  emit a module, every one byte-identical and no compilation succeeding on one
  side and failing on the other.

## Status

- [x] item 1 — window query, weak fallback, harness boundaries. The claim about
      what an answer of one means is now stated as a property of the runtime,
      not as something Stage A demonstrates
- [x] item 2 — deferred doorbell and its four flush points
- [x] item 3 — retire-and-retry as one rule over one process-wide ledger of two
      counts — descriptors returned, and operations in flight — asked at the
      moment of the host attempt, with the two per-engine gates it replaces
      gone and a deterministic test for each route, for the one exit that could
      miss a return, for the ending that returns nothing, and for the outcome
      each engine answers the return question from; a returned descriptor
      awarded in waiter order, so source order decides which open publishes the
      `Ok`; and every ledger transition announced in both the places a waiter
      can sleep, with the shape that proves it wired in as a probe of its own
- [x] item 4 — carrying and draining, with every drain including the first
      given exactly what the blocks that reach it started, the out-of-order
      descriptor refused, and the IR-identity oracle re-run: 630 sources,
      3 passes, 1,890 compilations, 807 modules, 0 differences
- [x] `completion-test`, `completion-sanitize`, `completion-tsan`,
      `completion-core-read-tsan` — green on an x86-64 Linux host at this
      revision, with the repetition counts above; green on macOS and in the
      aarch64 container at the previous one
- [x] canonical `make check` — green at this revision on x86-64 Linux
      (`conformance adapter: Pass=509  Skip=1`)
- [x] `io-hosts` and `gate` run on the pushed branch. `completion-linux` is
      the one that matters most here: it runs the harness, the sanitizers and
      `completion-tsan` on a real x86-64 Linux kernel with io_uring, so both
      new exhaustion tests are exercised on the ring rather than only in the
      aarch64 container. The six [QUAL-1] conformance cases that kept
      `gate-linux` red are green since the Linux row landed in `main`

## The job that was red, and is not any more

`gate-linux` used to fail on six conformance cases, all reaching
`TargetQualification(MissingMapping(Operation(12)))`: [QUAL-1] gave Linux no
approved [SYS-14] directory-enumeration row, so every case that enumerates a
directory answered `Unsupported` rather than its declared verdict. It was never
this branch's — `git diff main` over `compiler/src/backend/qualification.rs` was
empty then and is empty now — and it went away when the row landed:
`batch/0094-linux-directory-row` is in `main` at `10b76c66`, which this branch
is merged up to. Canonical `make check` on this Linux host reports
`conformance adapter: Pass=509  Skip=1` and `== WHITEFOOT ALL TESTS GREEN ==`.
