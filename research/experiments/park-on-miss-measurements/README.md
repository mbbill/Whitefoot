# Park on miss: the measurements of design §12 and the plan's added choices

`research/investigations/io-model/PARK-ON-MISS.md` §12 lists the measurements
that must exist before the design's choices are made, and
`docs/current-plan.md`'s "Decided 2026-09-05: measured before chosen" adds four
more: the claim protocol, the in-place wait of the idle window, the memory
orders, and the lane slot count with the ready list. This bundle is those
numbers.

**It chooses nothing.** Each alternative form is built behind a compile-time
`-D` switch read by the C unit, both forms are measured with one method, and
the numbers are reported. The coordinator takes them to the owner; slice 4b
deletes what loses and removes the switches with it.

## The rule this bundle was taken under

Every variant must pass `make completion-test`, including the §11 enumerator
under it, and `make format lint`, when built with its define. **A variant the
enumerator rejects is reported as rejected, with the schedule it found, and is
not measured**, because a number for a form the enumerator refuses is not a
number about a choice. One exception is stated in the plan and taken here: the
claim protocol's `-DWF_SCHED_NO_CLAIM` is a measurement variant only, and its
round trip is reported as the price of the protocol even though the enumerator
rejects it.

This bundle adds one test of its own, and it is the reason most of these tables
have one column. Admitting a form under the enumerator is not the same as the
form working: the enumerator's model is sequentially consistent and its
schedules are the ones `schedules.c` names. So every behavioural switch is also
run against two whole compiled programs at three worker counts under a timeout
— the **liveness** table below. Two forms that the enumerator admits or that
merely fail a coverage assertion do not survive it.

## The host

Linux 6.18.44-fc-v24, x86-64, four cores, 16 GiB. `cc` is GCC 13.3.0, which is
what `make completion-test` compiles the core with; every measured binary is
built by `/usr/bin/clang` 18.1.3 at `-O2`, which is the compiler `whitefootc`
links a program with. The tree is ext4 on a virtio device.

Its own reference primitives, from
`research/experiments/park-on-miss-switch-cost/` re-run here (the x86-64 arm
that bundle's caveats said the Linux runner would supply):

| operation | this host | the Darwin arm64 figure that bundle recorded |
|---|---|---|
| hand-written stack switch | 10.6 ns | 10.4 ns |
| `swapcontext` | 193.2 ns | 347 ns |
| condition-variable park-and-wake, two threads | 16217.9 ns | 872 ns |
| park-and-wake ÷ switch | 1526× | 84× |

The 2.2 µs park-and-wake figure §12 states its bars against is the tree's, from
another machine. This host's own park-and-wake is 16.2 µs, seven times that, so
every bar below is stated twice: against the design's 2.2 µs and against this
host's 16.2 µs.

## The switches, and the file that reads each

Every one is a `-D` define read by a C unit, never a run-time setting and never
a second file. Each site carries a comment naming this experiment and its §12
item.

| switch | §12 item | read by |
|---|---|---|
| `WF_SCHED_NESTED_NEVER_SUSPENDS` | 1 | `compiler/src/backend/sched/core.c` |
| `WF_SCHED_LOCKED_PARK` | 2 | `compiler/src/backend/sched/core.c`, `core.h` |
| `WF_SCHED_NO_CLAIM` | 3 | `compiler/src/backend/sched/core.c` |
| `WF_SCHED_PARK_AT_ONCE` | 4 | `compiler/src/backend/sched/core.c`, `compiler/src/backend/completion/bridge.c` |
| `WF_SCHED_WEAK_ORDERS` | 5 | `compiler/src/backend/sched/core.c` |
| `WF_SCHED_THREAD_READY` | 6 | `compiler/src/backend/sched/core.c`, `core.h` |
| `WF_SCHED_LANE_SLOTS` | 6 | `compiler/src/backend/sched/core.h` (already a constant of the core) |

`compiler/Makefile`'s `SCHED_VARIANT_DEFINES` carries one of them into every C
build the gate makes, so `completion-test` and the enumerator judge a form on
exactly the terms they judge the shipped one. The enumerator pins its own lane
slot count, so `SCHED_ENUMERATE_VARIANT_DEFINES` drops a lane-slot define from
that one build and from nothing else.

## Method

One method for every line: the io-completion-bench runner's discipline
(`../io-completion-bench/runner.c`). It runs the whole plan as a pass, over and
over — `WARMUP` unrecorded passes and then `ROUNDS` recorded ones, reversing
the plan's order on every other pass — so the two forms of a comparison are
interleaved and this host's drift lands on both. It reports the median across
passes with the observed minimum and maximum beside it, and it compares every
recorded run's bytes against one expected line, so a form that computed
something else cannot report a time at all. `ROUNDS` is 9 and `WARMUP` is 2
unless a table says otherwise.

The two programs that are not whole-process runs — the park micro-benchmark and
the four-stage chain — print one number per run, so `run.sh` repeats them
itself under the same discipline: interleaved, reversed on alternate passes,
best and median reported with N stated.

Everything below is one run of

    make -C compiler park-on-miss-measurements

on the host above, on 2026-09-05.

## 0. The gate, per form

`make -C compiler SCHED_VARIANT_DEFINES=<define> format lint` and
`make -C compiler SCHED_VARIANT_DEFINES=<define> completion-test`, which runs
`sched-enumerate` at (T=1,S=2), (T=1,S=3), (T=2,S=3) and (T=2,S=4).

| form | format + lint | completion-test | what the enumerator found |
|---|---|---|---|
| shipped | PASS | PASS | — |
| `WF_SCHED_NESTED_NEVER_SUSPENDS` | PASS | **FAIL** | S1, S9 and S23 at (1,2) and (1,3): "thread 0 spins with nothing left to change what it spins on"; S3 and S21 at (2,3) and (2,4) and S5 at (2,4): "not covered: no thief published to a parked parent", "no cancel from SUSPENDING on a compute record", "the sibling and its caller did not park to the pool's depth" |
| `WF_SCHED_LOCKED_PARK` | PASS | **FAIL** | every schedule at all four configurations (15, 15, 18, 17 failures): "stack N moved from phase 1 to 3 by a store, which the state machine has no edge for", and on the compute schedules "a record was stored DONE from state 1, not COMPLETING" |
| `WF_SCHED_NO_CLAIM` | PASS | **FAIL** | every schedule at all four configurations: "a record was stored DONE from state 1, not COMPLETING" |
| `WF_SCHED_PARK_AT_ONCE` | PASS | **FAIL** | S23 at (2,3) only: "not covered: no execution found a READY stack inside the I/O arm"; the other three configurations PASS |
| `WF_SCHED_WEAK_ORDERS` | PASS | PASS | — |
| `WF_SCHED_THREAD_READY` | PASS | **FAIL** | every schedule at all four configurations (15, 15, 18, 17 failures): "core: a load reached a word the enumerator does not model" |
| `WF_SCHED_LANE_SLOTS` 2, 4, 8, 16 | PASS | PASS | — |

Three of these rejections are not the same kind of thing, and the difference
matters to whoever reads them:

- **An invariant the form breaks.** `WF_SCHED_NO_CLAIM` stores DONE without the
  COMPLETING window, which is §11's own edge, and the liveness table below
  reproduces the resulting fault on this host. `WF_SCHED_NESTED_NEVER_SUSPENDS`
  at one thread reaches a live-lock: a compute hand-out whose callee does I/O
  does suspend, so the variant's assumption (see item 1) is false on S23, and
  with no third line to take, the compute arm spins on a target held by a
  parked stack.
- **A state machine the enumerator does not have.** `WF_SCHED_LOCKED_PARK` is
  §6's own locked fallback, which the design says collapses the five phases to
  RUNNING, SUSPENDED and READY and removes the COMPLETING window. The two
  failures above are exactly those two collapses: §11's checks encode the
  lock-free form's edges. Nothing here says the locked form has a hole; it says
  the enumerator as written cannot judge it. Giving §11 a second edge set is
  work, not a repair, and this slice did not do it.
- **A coverage assertion the form cannot meet.** `WF_SCHED_PARK_AT_ONCE`
  removes the idle window's look at the ready list, which is the arm S23's
  coverage check asserts is reached. No invariant is violated at any
  configuration.
- **A word the enumerator does not model.** `WF_SCHED_THREAD_READY` puts a new
  shared word (`thread->ready_own`) in the core, and the enumerator's replacement
  primitives model a fixed set of words. Again: not a hole, an unmodelled word.

## 1. Liveness, per form

Each behavioural form, both compute programs, three worker counts, one run
each, under a ten-second timeout. This is not a timing; it is whether a number
could be taken at all.

| form | par_layout W=1 | W=2 | W=4 | grid W=1 | W=2 | W=4 |
|---|---|---|---|---|---|---|
| shipped | ran | ran | ran | ran | ran | ran |
| `WF_SCHED_NESTED_NEVER_SUSPENDS` | ran | ran | ran | ran | ran | ran |
| `WF_SCHED_LOCKED_PARK` | ran | ran | ran | ran | ran | ran |
| `WF_SCHED_NO_CLAIM` | ran | **hung** | **abort** | ran | ran | ran |
| `WF_SCHED_PARK_AT_ONCE` | ran | ran | ran | ran | ran | ran |
| `WF_SCHED_WEAK_ORDERS` | ran | **hung** | **hung** | ran | ran | ran |
| `WF_SCHED_THREAD_READY` | ran | ran | ran | ran | ran | ran |

The abort is `whitefoot scheduler: a cancel that owned its registration found
its stack not SUSPENDING` — the enumerator's S3 finding, reproduced on a real
host by removing the claim.

The `WF_SCHED_WEAK_ORDERS` hang is the important one, because that form passes
the gate. It is deterministic. Beside the ten-second probe in the table,
`par_layout` at `WF_WORKERS=4` was run eight consecutive times under a
twenty-second timeout and finished none of them, while the shipped form
finished all eight; three further runs each at `WF_WORKERS=2` and
`WF_WORKERS=8` also finished none, and `WF_WORKERS=1` finished every time. The
shape is the one §6's steps 2 and 3 are written
around — the parker stores `record->waiter` and then loads `record->state`
while the publisher stores `state` and then loads `waiter` — which is a
store-then-load on both sides and the one ordering x86-64's own model does not
give away for free. Release and acquire leave both sides able to see the stale
value and the wake is lost. So the plan's deferred note is now stronger than it
was: the weaker orders are not merely unproved pending a GenMC run, they are
observably wrong here, and no GenMC run is needed to say so for this pair.

**Therefore the only forms measured below are the shipped one and the lane slot
counts.** Every other §12 item is reported as one column with its bar, and with
the reason its second column is absent.

## 2. §12 item 1 — park and resume at a compute miss

The design's bar: within noise of nested helping. The plan records the
regression this item exists for: `par_layout` W=4 0.4067 s before park-on-miss
and 0.5711 s after, W=8 0.4213 s and 0.8307 s.

Taken again on the current tree, and beside it the grid loop-split program this
bundle supplies (see the caveat below):

| line | median (ms) | min (ms) | max (ms) | user (ms) | sys (ms) |
|---|---|---|---|---|---|
| par_layout W=1 | 1526.64 | 1525.82 | 1541.53 | 1526.19 | 0.00 |
| par_layout W=2 | 824.13 | 821.83 | 839.08 | 1565.49 | 36.09 |
| par_layout W=4 | 587.82 | 583.38 | 608.62 | 1592.55 | 239.39 |
| par_layout W=8 | 856.48 | 838.61 | 901.32 | 1630.75 | 701.71 |
| grid W=1 | 1561.19 | 1558.70 | 1604.53 | 1560.41 | 0.00 |
| grid W=2 | 778.10 | 776.98 | 795.66 | 1549.57 | 3.99 |
| grid W=4 | 398.14 | 391.15 | 416.19 | 1555.66 | 3.99 |
| grid W=8 | 399.42 | 392.93 | 407.79 | 1553.74 | 3.98 |

N = 9 recorded passes after 2 warm-up passes, interleaved.

**The bar, and whether it is met.** The par_layout regression stands: 0.588 s at
W=4 against the 0.4067 s the tree measured before park-on-miss is 45 percent
slower, and 0.856 s at W=8 against 0.4213 s is twice as slow. Both are far
outside the spread of either measurement, so the bar "within noise" is missed,
by the same margin the plan already recorded. The system-time column says where
it goes: 239 ms of system time at W=4 and 702 ms at W=8, against 0 ms at W=1.
That is the park's sleep and wake, which the grid program at the same worker
counts does not pay at all (4 ms of system time) because its loop split hands
out large chunks and almost never misses — 26 parks per run against par_layout's
18 000.

**The second column is missing and this is why.** §12's stated fallback is to
run a never-suspends job nested at a compute miss, which needs a target-action
bit at the hand-out. Today's emitter marks none. The variant built here takes
the bit from the only fact available — that a compute hand-out of a group with
no completion member never suspends, which is `compute_join_order`'s
classification in `emitter/parallel.rs` — and therefore assumes *every* compute
hand-out never suspends. **That assumption is false**, and the enumerator says
so directly: S23 is a hand-out whose callee does I/O, the variant's compute join
then has no third line, and the compute arm spins on a target that only a parked
stack can move. So the fallback cannot be measured until the emitter marks
hand-outs, and what §12 item 1 asks for is not answerable in this slice. It is
not a close call: the form as built is a live-lock at one thread.

## 3. §12 item 2 — the lock-free handshake, and the locked form of §6

One park and one publish through `sched/core.c` over the real host primitives,
timed by `park_publish.c`: one thread joins an I/O record and misses, parks its
stack and switches to a free one; another thread publishes; the parked stack is
resumed and the join returns.

Two modes, because the answer differs and both are real:

- **settled** — the publisher waits for the parked stack to reach SUSPENDED, so
  the unit is one whole park followed by one whole publish and the resume finds
  the READY stack at the scheduler loop's first priority. No sleep is in it.
- **racing** — the publisher waits only for the registration, so it acts inside
  §6's own window and the run takes the NOTIFIED arm and the cancel arm as the
  host's timing falls.

| form | mode | best (ns) | median (ns) | N |
|---|---|---|---|---|
| shipped | settled | 4399 | 6231 | 15 |
| shipped | racing | 3660 | 5660 | 15 |
| `WF_SCHED_WEAK_ORDERS` | settled | 3119 | 4394 | 15 |
| `WF_SCHED_WEAK_ORDERS` | racing | 3086 | 3904 | 15 |
| `WF_SCHED_NO_CLAIM` | settled | 4502 | 6556 | 15 |
| `WF_SCHED_NO_CLAIM` | racing | — | — | faulted on every run |

**The bar.** §12 states it against the 2.2 µs park-and-wake figure. The shipped
form's park and publish is 4.40 µs at best and 6.23 µs at the median, so it is
twice the design's quoted figure and **misses that bar**; against this host's
own park-and-wake of 16.2 µs it is 0.27× and comfortably inside. Which reading
is right is the owner's call and not this bundle's; the two numbers are both
here so the call can be made on one host's arithmetic.

**The locked form has no row.** The enumerator rejects `WF_SCHED_LOCKED_PARK`
for the reason given in section 0 — §11's invariants encode the lock-free
state machine that §6's locked fallback deliberately does not have — so under
the rule this bundle was taken under it is not measured. §12 item 2 is therefore
unanswered, and what it needs is not a measurement but a second invariant set in
`enumerate.c` for the locked state machine. The form itself builds, passes
`sched-smoke`, and ran both programs at every worker count in the liveness
table, so the work that remains is the enumerator's and not the form's.

## 4. §12 item 3 — the claim protocol

`WF_SCHED_NO_CLAIM` removes the COMPLETING store and the compare-exchange on
`record->waiter`. The enumerator rejects it on every schedule at all four
configurations, with "a record was stored DONE from state 1, not COMPLETING",
which is §11's edge and the defect the plan's finding 3 records. The plan asks
for the number anyway, as the price of the protocol:

| form | mode | best (ns) | median (ns) | N |
|---|---|---|---|---|
| shipped | settled | 4399 | 6231 | 15 |
| `WF_SCHED_NO_CLAIM` | settled | 4502 | 6556 | 15 |

**The price of the claim protocol is not visible at this host's spread.** The
settled round trip is 4.40 µs against 4.50 µs at best and 6.23 µs against
6.56 µs at the median, and the two samples overlap across every pass. The
COMPLETING store and one uncontended compare-exchange are a few tens of
nanoseconds against a round trip of thousands.

What the variant does show is the fault, twice. In `racing` mode it aborts on
every run with `a cancel that owned its registration found its stack not
SUSPENDING`, and on `par_layout` at `WF_WORKERS=4` it aborts with the same line
— the enumerator's S3 finding, reproduced by a real host in under a second.

## 5. §12 item 4 — the in-place wait of the idle window

`WF_SCHED_PARK_AT_ONCE` removes the idle window's look at the ready list and
the bridge's bounded spin, so the window sleeps at once. The enumerator fails
it at (T=2,S=3) on S23's coverage assertion — "no execution found a READY stack
inside the I/O arm", which is the arm the variant removes — and passes the other
three configurations. Under this bundle's rule it is not measured.

The shipped form's own column, on the many-files workload at the default helper
policy and at four helpers:

| line | median (ms) | min (ms) | max (ms) | user (ms) | sys (ms) |
|---|---|---|---|---|---|
| many_files_wide, default | 112.06 | 110.49 | 116.11 | 71.50 | 39.99 |
| many_files_wide, `WF_IO_HELPERS=4` | 112.28 | 111.89 | 127.74 | 83.10 | 29.06 |

N = 9, warm-up 2, `FILES=8192`, `MAX_KIB=16`, checksum
`17098009301725298919 00000000000071024640`.

The `four_stage_chain` half of this item is section 7 below; the chain is a C
program and takes no `-D` of the core's, so the in-place wait cannot be varied
in it either.

## 6. §12 item 5 — the memory orders

`WF_SCHED_WEAK_ORDERS` replaces the sequentially consistent orders at the
record's state and waiter and at the stack's phase with acquire and release.
It passes `make format lint`, `make completion-test` and the §11 enumerator at
all four configurations, and its park and publish is faster than the shipped
form's on this host:

| form | mode | best (ns) | median (ns) | N |
|---|---|---|---|---|
| shipped | settled | 4399 | 6231 | 15 |
| `WF_SCHED_WEAK_ORDERS` | settled | 3119 | 4394 | 15 |
| shipped | racing | 3660 | 5660 | 15 |
| `WF_SCHED_WEAK_ORDERS` | racing | 3086 | 3904 | 15 |

**And the number is not usable, because the form hangs.** `par_layout` at
`WF_WORKERS=2` and above did not finish inside ten seconds in any run here, nor
inside twenty seconds in eight consecutive repeats taken separately, while the
shipped form finished every time. The plan's deferred note says a weaker order is admitted
only with a GenMC run because the enumerator's model is sequentially
consistent; this host has answered that question ahead of GenMC for the pair
that matters. The enumerator passing this form is exactly the blind spot the
note names, now with a witness.

## 7. §12 item 4 — the four-stage chain in C on io_uring

`../io-completion-bench/chain.c`, 1000 files, 8 threads, in the four shapes §12
names. One reaper thread drives the ring in every shape, so what the four
numbers compare is what a worker does at a join that missed and nothing else.
Every shape publishes the same fold (`02227017180865178071`), so a shape that
skipped work cannot report a time. Descriptors are opened once before the timed
region, `O_DIRECT`, so a read is a device round trip and not a page-cache hit.

| shape | wall median (ms) | wall min–max | dependent-stage peak, median | dependent-stage mean in flight, median | all operations, peak median |
|---|---|---|---|---|---|
| nested helping | 119.43 | 108.15–134.29 | 8 | 3.06 | 20 |
| thread compensation | 124.13 | 114.50–128.98 | 15 | 3.17 | 46 |
| stack switch (the shipped core) | 438.36 | 426.28–478.29 | 8 | 0.22 | 18 |
| staged pipeline, K = 32 | 119.68 | 108.14–124.12 | 8 | 1.99 | 27 |

N = 7 interleaved runs. "Dependent stage" is the *request* read, the one that
depends on the parse of the first read's bytes — the operation §0's diagram
shows collapsing to the thread count.

**What the table shows, and what it does not.** §0's claim is visible in three
rows: nested helping and the staged pipeline both hold the dependent stage at
about the thread count (8), and thread compensation, which spends OS threads to
keep continuations live, reaches about twice that. **The stack-switch shape does
not beat them here, on either axis**: its dependent depth is the same 8 and its
wall time is 3.7× the others.

Two things have to be said about that number before anyone reads a design
conclusion into it, and neither is the design.

- The park it sleeps on is `prim_host.c`'s fallback — one epoch on one mutex and
  condition variable, whose wake is a `pthread_cond_broadcast` to every sleeper.
  A shipped Whitefoot program does not use it: the bridge answers the
  `wf__sched_host_epoch`/`_park`/`_wake` seam and sleeps on the ring. Linking the
  bridge into this program means reproducing the file adapter, which this slice
  did not do. With about 3 000 parks per run and eight threads woken per
  publication, the herd is where the 320 ms goes; the shipped park in section 3
  costs 4.4 µs, and 3 000 of those is 13 ms, not 320.
- The bar §12 states — that park on miss is not slower than the pipeline on the
  pipeline's own program — cannot be tested on this host at all, because the
  reads do not wait enough for any shape to reach its own bound. The largest
  in-flight count any shape reached is 64 against a pipeline capacity of
  8 × 32 = 256 and a switch capacity of 72 stacks. What limits every row is the
  rate at which a worker can submit, not the depth its shape allows. A host
  whose reads cost hundreds of microseconds each, or a workload with a real
  wait, is what this item needs; the plan already says the language's current
  surface has no such wait, and this is the same wall from the other side.

## 8. §12 item 5 — the stack count at which the pool stops refusing

`WF_STACKS` swept from the core's floor (threads + 1) upward, with the core's
own `exhausted_compute` counter read at exit through `statistics_observer.c`.
No I/O refusal (`exhausted_io`) occurred anywhere in the sweep.

par_layout:

| W | WF_STACKS | wall (ms) | parks | exhausted_compute |
|---|---|---|---|---|
| 4 | 5 (floor) | 585 | 3325 | 54675 |
| 4 | 6 | 590 | 4862 | 21961 |
| 4 | 8 | 577 | 6804 | 5237 |
| 4 | 12 | 585 | 6541 | 0 |
| 4 | 16 | 581 | 7085 | 0 |
| 4 | 20 | 580 | 6450 | 0 |
| 4 | 28 | 580 | 6740 | 0 |
| 4 | 36 | 584 | 6733 | 0 |
| 8 | 9 (floor) | 827 | 5438 | 274317 |
| 8 | 10 | 837 | 9910 | 176735 |
| 8 | 12 | 840 | 16488 | 22495 |
| 8 | 16 | 837 | 18242 | 11 |
| 8 | 20 | 868 | 18097 | 0 |
| 8 | 24 | 858 | 17648 | 0 |
| 8 | 32 | 854 | 17592 | 0 |
| 8 | 40 | 850 | 18221 | 0 |

grid:

| W | WF_STACKS | wall (ms) | parks | exhausted_compute |
|---|---|---|---|---|
| 4 | 5 (floor) | 391 | 4 | 11222 |
| 4 | 6 | 399 | 8 | 20728 |
| 4 | 8 | 393 | 4 | 0 |
| 4 | 12 to 36 | 393–401 | 3–15 | 0 |
| 8 | 9 (floor) | 399 | 10 | 11145 |
| 8 | 10 | 397 | 12 | 17259 |
| 8 | 12 | 395 | 15 | 46371 |
| 8 | 16 | 400 | 26 | 0 |
| 8 | 20 to 40 | 395–402 | 26–35 | 0 |

**The smallest stack count at which the pool stops refusing** is 12 for
par_layout at W=4, 20 at W=8 (with 11 refusals still at 16), 8 for the grid at
W=4 and 16 at W=8. The shipped default is threads + 8, which is 12 at W=4 and
16 at W=8 — right at the answer for W=4 and one step under it at W=8.

**What a refusal costs** is the one clear result in this table: on wall time,
nothing measurable. par_layout at W=8 pays 274 317 refusals at the floor and
runs in 827 ms against 850 ms with no refusals at all; the grid pays 46 371 at
twelve stacks and runs in 395 ms against 398 ms. The refused arm runs the
target's own work on the joining thread (`inline_runs` rises by the same order),
which on a compute program is the work that had to happen anyway. What the
stack count buys is parks, not time: par_layout at W=8 goes from 5 438 parks at
the floor to 18 242 at sixteen stacks with no change in wall time either way.

## 9. §12 item 6 — the lane slot count and the ready list

`WF_SCHED_LANE_SLOTS` at 2, 4, 8 and 16 against the shipped 64, at the worker
counts where a lane is contended. Interleaved with the shipped form in one plan.

par_layout:

| line | median (ms) | min (ms) | max (ms) |
|---|---|---|---|
| shipped (64) W=4 | 583.43 | 574.21 | 624.29 |
| 2 slots W=4 | 572.33 | 568.17 | 585.08 |
| 4 slots W=4 | 575.82 | 568.89 | 595.80 |
| 8 slots W=4 | 581.30 | 575.14 | 590.91 |
| 16 slots W=4 | 578.19 | 570.78 | 583.39 |
| shipped (64) W=8 | 841.91 | 831.91 | 856.64 |
| 2 slots W=8 | 838.35 | 832.39 | 847.40 |
| 4 slots W=8 | 837.34 | 821.81 | 857.69 |
| 8 slots W=8 | 841.61 | 823.35 | 853.91 |
| 16 slots W=8 | 838.11 | 821.86 | 856.52 |

grid:

| line | median (ms) | min (ms) | max (ms) |
|---|---|---|---|
| shipped (64) W=4 | 395.34 | 389.87 | 401.09 |
| 2 slots W=4 | 394.82 | 393.10 | 441.73 |
| 4 slots W=4 | 397.07 | 392.60 | 409.69 |
| 8 slots W=4 | 391.99 | 385.49 | 401.69 |
| 16 slots W=4 | 394.13 | 390.58 | 397.04 |
| shipped (64) W=8 | 394.18 | 391.82 | 404.73 |
| 2 slots W=8 | 442.46 | 424.42 | 537.61 |
| 4 slots W=8 | 403.99 | 395.53 | 405.98 |
| 8 slots W=8 | 390.68 | 386.10 | 395.71 |
| 16 slots W=8 | 393.34 | 390.45 | 400.57 |

N = 9, warm-up 2, interleaved. **The measurement separates nothing between 4,
8, 16 and 64 slots on either program**: every median is inside every other
row's observed min–max. Two slots is the one row that is separated, and only on
the grid at W=8, where it costs 12 percent (442 ms against 394 ms) and widens
the spread to 424–538 ms — which is the loop splitter running out of slots to
offer into. Under the owner's rule a choice the measurement cannot separate
keeps the form the enumerator checked, and the enumerator checked all five.

**The per-thread ready list has no row.** `WF_SCHED_THREAD_READY` is rejected by
the enumerator with "a load reached a word the enumerator does not model" on
every schedule: the variant adds a shared word the enumerator's replacement
primitives have no model for. It builds, passes `sched-smoke`, and ran both
programs at every worker count in the liveness table, so what it needs is a
model of its word in `enumerate.c`, not a repair.

## 10. §12 item 6 — record memory per frame

`whitefootc --stack-ledger` over `tests/programs`, at the record size the tree
carried at `92b19e1` and at the current one. The "before" column is built from
a scratch copy of the compiler with the emitter's constant set to the value
`git show 92b19e1:` reports; the tree is untouched.

Record bytes: before 128 (`WF_COMPLETION_RECORD_BYTES` 128), after 160
(header 160).

**The growth per frame that holds a group**, read as the chain bound of
`wf__main_body`, which is the frame every one of these programs reserves its
records in:

| program | main at 128 | main at 160 | growth | records held |
|---|---|---|---|---|
| completion_read_boundary | 3216 | 3344 | 128 | 4 |
| wfgrep | 3680 | 3776 | 96 | 3 |
| byte_string | 720 | 752 | 32 | 1 |
| dir_walk | 608 | 640 | 32 | 1 |
| par_layout | 576 | 608 | 32 | 1 |
| every other program in `tests/programs` | — | — | 0 | 0 |

**The growth of the deepest chain bound** is the same number in every case: no
program in `tests/programs` has a deeper chain than the one through
`wf__main_body`, so the deepest bound grows by 128, 96, 32, 32, 32 bytes on
those five programs and by nothing on the other twenty-one. The growth is
exactly 32 bytes per outstanding operation the frame holds, which is what the
plan's step (iv) note predicted, and it is a bound on a 1 GiB stack.

## 11. §12 item 7 — the ledger's chain bound per hand-out entry

Across `tests/programs`, at the shipped record size, one row per hand-out
entry the emitter produced. No variant.

| program | hand-out entry | chain bound (bytes) |
|---|---|---|
| byte_string | `wf__par_thunk_0` | 80 |
| dir_walk | `wf__par_thunk_0` | 16 |
| dir_walk | `wf__par_thunk_1` | 80 |
| generic_instances | `wf__par_thunk_0` | 8 |
| generic_instances | `wf__par_thunk_1` | 8 |
| generic_nominals | `wf__par_thunk_0` | 8 |
| growable_vec | `wf__par_thunk_0` | 80 |
| par_layout | `wf__par_thunk_0` | 64 |
| par_layout | `wf__par_thunk_1` | 16 |
| par_layout | `wf__par_thunk_2` | 16 |
| recursive_tree | `wf__par_thunk_0` | 64 |
| recursive_tree | `wf__par_thunk_1` | 64 |
| recursive_tree | `wf__par_thunk_2` | 64 |
| recursive_tree | `wf__par_thunk_3` | 80 |
| sha256_abc | `wf__par_thunk_0` | 8 |
| sha256_abc | `wf__par_thunk_1` | 8 |
| wfgrep | `wf__par_thunk_0` | 8 |
| wfgrep | `wf__par_thunk_1` | 8 |
| wfgrep | `wf__par_thunk_2` | 8 |

Nineteen hand-out entries across nine programs, and every one of them is bounded
by 80 bytes. §5's later step asks how many stack classes real programs would
need; the answer this table gives is that today's corpus needs one, because the
whole population fits inside a single small class and the frames that are large
(`wf__main_body` at 3 776 bytes on wfgrep) are not hand-out entries at all.

## What this bundle holds

- `README.md` — this file: the method, every table, the commands and the bars.
- `run.sh` — every line above that a script can produce, in eight sections
  (`gates liveness park compute lanes io chain stacks ledger`).
- `park_publish.c` — the park and publish micro-benchmark of sections 3 to 6.
- `statistics_observer.c` — the core's counters at exit, for section 8. Linked
  beside an emitted module by `run.sh` and by nothing the driver stages.
- `programs/grid_split.wf` — the grid loop-split program section 2 measures.

`../io-completion-bench/chain.c` is section 7's program; it is there and not
here because it is built out of that bundle's ring plumbing, generator and
file-name format.

## Caveats, stated rather than implied

- **The grid loop-split program is this bundle's, not the corpus's.** §12 item 1
  asks for a grid loop-split program and `tests/programs` has none: its grid
  program, `mandelbrot_grid.wf`, is written with `loop` and not with a counted
  `for`, so [PAR-2] does not reach it and it is granted nothing. `programs/grid_split.wf`
  is a counted `for` over a 512-wide grid with one `+wrap` accumulator, whose
  per-point cost depends on where in the grid the point is, so an even split of
  the range is not an even split of the work. Its `@points` loop is granted
  ("permitted, eligible; one accumulator under +wrap") and it scales 1.56 s to
  0.39 s from one worker to four.
- **One host, and a shared one.** Every number here is from one four-core Linux
  virtual machine whose neighbours it does not control. The park micro-benchmark
  in particular spreads by a factor of two across passes (best 4.4 µs, median
  6.2 µs, worst passes above 10 µs), which is why best and median are both
  printed and why section 9 says the lane slot counts cannot be separated rather
  than picking the fastest median.
- **`make completion-test` compiles the core with GCC 13.3; every measured
  binary is clang 18.** The gate's own compiler is `cc`. A form admitted by one
  and measured under the other is a seam, and it is stated rather than closed.
- **The chain's stack-switch shape is the real core over the fallback park.**
  See section 7. It is the shipped `core.c` and the shipped `prim_host.c`, and
  it is not the shipped park, because the shipped park is the bridge's.
- **Nothing here was chosen.** No default moved, no losing form was deleted, and
  the shipped build compiles exactly the form the §11 enumerator checked:
  `SCHED_VARIANT_DEFINES` is empty in every gate run.

## Reproducing

    make -C compiler park-on-miss-measurements
    make -C compiler park-on-miss-measurements SECTIONS="park compute"

or the script directly, from anywhere:

    sh research/experiments/park-on-miss-measurements/run.sh
    sh research/experiments/park-on-miss-measurements/run.sh chain ledger

One form's gate, by hand:

    make -C compiler SCHED_VARIANT_DEFINES=-DWF_SCHED_WEAK_ORDERS format lint
    make -C compiler SCHED_VARIANT_DEFINES=-DWF_SCHED_WEAK_ORDERS completion-test

The chain alone:

    make -C research/experiments/io-completion-bench chain

The target is outside `make check` and nothing in the repository depends on
what it prints.

## Removal condition

This bundle is removed in slice 4b, with the compile-time switches it measures,
once the owner has chosen from these numbers. `programs/grid_split.wf` goes with
it unless the corpus adopts it; `chain.c` goes when §12's fourth item is
answered or retired.
