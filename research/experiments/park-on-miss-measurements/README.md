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

**Status 2026-09-05, slice 4b: the switches are gone; every table below stands
as the record of the run of 2026-09-05.** Under the plan's rule that a choice
the measurement cannot separate keeps the form the §11 enumerator checked, the
six behavioural switches were deleted from `core.c`, `core.h` and `bridge.c`
with every `#if` and every field they added, and `compiler/Makefile` lost the
`SCHED_VARIANT_DEFINES` plumbing that carried them: five of them the enumerator
rejected — one of those, the claim-protocol variant, measured anyway under the
exception the plan states — `WF_SCHED_WEAK_ORDERS` passed every gate and hangs
`par_layout`, and the lane-slot sweep separated nothing between 4 and 64.
`WF_SCHED_LANE_SLOTS` remains what it was before the sweep, the `#if !defined`
override of `core.h`, so its sweep still runs. No number below was rewritten; each retired section
opens with one sentence saying that its form was removed at that change and
why, so the numbers stay readable. `run.sh` keeps every line it can still
reproduce and has lost the per-form gate table and the per-form liveness probe,
which were sweeps of forms that no longer exist. What is still open is design
§12 item 1, the compute-miss regression and its fallback, which is the owner's
decision (`docs/current-plan.md`, Batch 2).

**Added 2026-09-05, after that: the "§12 addendum: the idle spin" at the foot of
this file.** It is the one measurement here that judges a change to the core
rather than a form the core already had: the bounded spin the idle window now
makes before it parks. It chose the two constants `WF_SCHED_IDLE_SPIN_ROUNDS`
and `WF_SCHED_IDLE_YIELD_ROUNDS` that `sched/core.h` carries, and `run.sh`
gained a `spin` section that reproduces its sweep. It also moves §12 item 1's
open regression: `par_layout` at four workers reads 451 ms with the spin
against 563 without it, and at eight 493 against 802.

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

**Slice 4b removed every switch in that table but the last**, with the `#if`s
and the fields they added, for the verdicts recorded below; `WF_SCHED_LANE_SLOTS`
stays because it was a constant of `core.h` before the sweep and still is. The
table is kept as the record of what was built and where each form was read.

`compiler/Makefile`'s `SCHED_VARIANT_DEFINES` carried one of them into every C
build the gate made, so `completion-test` and the enumerator judged a form on
exactly the terms they judge the shipped one. The enumerator pins its own lane
slot count, so `SCHED_ENUMERATE_VARIANT_DEFINES` dropped a lane-slot define from
that one build and from nothing else. Both variables went with the switches;
the gate now compiles one form, which is the shipped one.

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

*Retired at slice 4b: every form in this table but the shipped one and the lane
slot counts was deleted from the core: five of them for the enumerator verdict
this table records, and `WF_SCHED_WEAK_ORDERS` for the hang section 1 records.
The table stands as what the enumerator said on 2026-09-05; the per-form gate
run that produced it is no longer in `run.sh`, because there is one form left
to run it on and `make completion-test` runs it.*

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

*Retired at slice 4b with the forms it probes: this table is the reason
`WF_SCHED_WEAK_ORDERS`, which passes every gate, was deleted rather than kept,
and the reason `WF_SCHED_NO_CLAIM`'s fault is recorded as reproduced on a real
host. The probe is gone from `run.sh` with the forms.*

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

*The shipped column below is still measured by `run.sh`'s `compute` section.
The second column's form, `WF_SCHED_NESTED_NEVER_SUSPENDS`, was deleted at
slice 4b: the enumerator finds it a live-lock at one thread, for the reason
this section gives. §12 item 1 stays open in `docs/current-plan.md` as the
owner's decision, and what it needs is the target-action bit at the hand-out,
not this variant.*

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

*Retired in part at slice 4b: the shipped row is still measured by `run.sh`'s
`park` section, and the `WF_SCHED_WEAK_ORDERS` and `WF_SCHED_NO_CLAIM` rows are
the record of forms the change deleted -- the first for the hang of section 1,
the second for the enumerator's verdict of section 0. `WF_SCHED_LOCKED_PARK`
was deleted with them, having never had a row; what §12 item 2 needs is a
second invariant set in `enumerate.c`, which this slice did not write.*

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

*Retired at slice 4b with `WF_SCHED_NO_CLAIM`, which the enumerator rejects on
every schedule at all four configurations and which faults on a real host. The
answer it was built for is in the table below and does not need the form again:
the claim protocol's price is not visible at this host's spread.*

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

*Retired at slice 4b with `WF_SCHED_PARK_AT_ONCE`, which fails S23's coverage
assertion at (T=2,S=3) -- the arm the variant removes is the arm that assertion
requires. The shipped form's own two rows below are still measured by `run.sh`'s
`io` section.*

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

*Retired at slice 4b with `WF_SCHED_WEAK_ORDERS`. It is the one form here that
passed every gate, and it is deleted for the hang this section records rather
than for a verdict: §6's store-then-load pair on both sides needs the
sequentially consistent orders the shipped form has. The GenMC run the plan
defers is still worth taking, and it has one fewer question to answer.*

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

*The lane slot counts below are still measured by `run.sh`'s `lanes` section:
`WF_SCHED_LANE_SLOTS` survived slice 4b as the `#if !defined` override of
`core.h` it was before the sweep, because the measurement separates nothing
between 4 and 64 and the shipped 64 is the count the enumerator checked.
`WF_SCHED_THREAD_READY` was deleted for the verdict recorded at the foot of
this section.*

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

## §12 addendum: the idle spin

*Taken 2026-09-05 on the host above, with the method above, and unlike every
section before it this one measures a change to the core rather than a form
the core already had: the bounded spin `wf_sched_idle_step` makes before it
parks, and the two constants of `compiler/src/backend/sched/core.h` that bound
it, `WF_SCHED_IDLE_SPIN_ROUNDS` and `WF_SCHED_IDLE_YIELD_ROUNDS`. `run.sh`'s
`spin` section is the sweep; `spin-0-0` in every table below is the core
without the spin, which is the form sections 2 to 9 measured.*

### The record that asked for it

The Windows qualification bench (`.github/workflows/io-bench.yml`, job
`bench-windows-qualified`, script
`../io-completion-bench/windows-bench.ps1`) now runs to its bar on the real
host and fails it:

| cohort | reference median ms | candidate median ms | paired candidate/reference | MAD | p10..p90 |
|---|---|---|---|---|---|
| compute | 4713.074 | 1508.833 | 0.3202 | 0.27% | 0.3186..0.3228 |
| io-warm | 206.564 | 199.785 | 0.9595 | 1.39% | 0.9380..0.9868 |
| mixed-iocp | 273.916 | 273.332 | 0.9975 | 0.21% | 0.9923..1.0060 |
| mixed-full | 273.686 | 291.099 | 1.0593 | 1.42% | 1.0443..1.0847 |
| mixed-total | 273.798 | 289.026 | 1.0578 | 0.56% | 1.0519..1.0788 |

with `grants=1024`. Compute at most 0.90 is met and io-warm at most 1.10 is
met; **mixed-full and mixed-total at most 0.95 are missed**. The unified
`--par` build of `../io-completion-bench/programs/windows_runtime_mixed.wf` is
6 percent slower than its completion-only build on a four-vCPU Windows VM,
with every one of the 1024 `compute_pair` hand-outs stolen.

The same three builds on this Linux host, at `WF_WORKERS=4`, three rounds
each: sequential 249 to 262 ms, completion-only 259 to 260 ms, unified 173 to
178 ms, a ratio of 0.67. So the design wins here and loses there, and what
differs is the host's wake latency. Per iteration of that program's loop the
pattern is: the main thread publishes `churn` L and runs `churn` R, about 200
microseconds; an idle worker has to be woken to steal L; the main thread's
join then misses, parks its stack, and the worker's completion has to wake the
main thread again. On Windows each of those two wakes is a
`GetQueuedCompletionStatus` round trip, which that VM makes long. The retired
runtime (`git show 92b19e1^:compiler/src/backend/par_runtime.c`, lines 120 to
137 and 476 to 492) found both of them with no kernel at all: an idle lane spun
`WF_PAR_SPIN_ROUNDS` 4096 rounds, yielded `WF_PAR_YIELD_ROUNDS` 16 times, and
only then slept on its condition variable.

### The change measured

`wf_sched_idle_step` repeats its own looks before it parks: the record a stack
waits on, the ready list, and -- on the scheduler loop's own turn -- this
thread's deque and a steal, for `WF_SCHED_IDLE_SPIN_ROUNDS` rounds of
`wf_prim_pause` and then `WF_SCHED_IDLE_YIELD_ROUNDS` rounds of
`wf_prim_yield`. The looks are the same calls the step already made, factored
into one function that the window's own last look and every spin round run.
Where in the step it goes is the next section, and it is most of the result.
The epoch capture does not move: it stays in front of the drain, so every look
the spin makes is a look inside the capture-to-park window §6 argues about.

`wf_prim_pause` is new, item 6 of `prim.h` beside the yield: `pause` on x86,
`isb` on aarch64, `YieldProcessor` on Windows, and nothing at all under the
enumerator, where a step that reaches no shared state is not a step.

### The method, and what "the spin" means in these tables

The bundle's method, unchanged: the io-completion-bench runner's discipline,
interleaved passes with alternating direction, medians over `ROUNDS` recorded
passes after `WARMUP` warm-up ones, every recorded run's bytes compared against
one expected line. `ROUNDS` is 9 for the sweep and 15 for the four-form
comparison at the end; the I/O line is 7 rounds, read as a best-of.

A form is named `spin-<pause rounds>-<yield rounds>`, and it is one build of
the whole runtime with those two constants defined. `spin-0-0` is the core
without the spin. Twenty-one forms: pause rounds 0, 16, 64, 256, 1024, 4096 and
16384 against yield rounds 0, 16 and 64. The coarse grid this sweep started
from was 0, 256, 1024, 4096, 16384; 16 and 64 were added after the first run
put the interesting region far below 256.

### Where the spin goes, which is most of the result

The first form built put the spin at the top of the idle window, in front of
everything: before the idle bit went up, before the epoch capture, and before
`wf_prim_progress`, which is the drain. It is the reading of "spin before you
park" that keeps the park's own window untouched, and on this host it is a
disaster, because **the drain is the only thing that delivers an I/O
completion**, so a spin in front of it delays every completion by its own
length. `many_files_wide`, with the spin there:

| form | median ms | user ms | sys ms |
|---|---|---|---|
| `spin-0-0` | 111.03 | 72.12 | 40.83 |
| `spin-0-16` | 210.66 | 107.91 | 103.63 |
| `spin-0-64` | 501.35 | 204.10 | 280.55 |
| `spin-16-0` | 132.56 | 85.84 | 47.20 |
| `spin-16-16` | 232.33 | 125.61 | 105.35 |
| `spin-16-64` | 523.14 | 224.58 | 284.02 |
| `spin-64-0` | 187.11 | 142.67 | 43.56 |
| `spin-64-16` | 284.96 | 187.67 | 105.08 |
| `spin-64-64` | 577.68 | 290.67 | 292.49 |
| `spin-256-0` | 398.38 | 344.12 | 48.26 |
| `spin-256-16` | 497.85 | 367.56 | 123.35 |
| `spin-256-64` | 789.15 | 510.09 | 279.59 |
| `spin-1024-0` | 1225.06 | 1167.83 | 44.01 |
| `spin-1024-16` | 1348.33 | 1207.98 | 131.80 |
| `spin-1024-64` | 1635.58 | 1337.54 | 288.20 |
| `spin-4096-0` | 4578.95 | 4529.90 | 48.00 |
| `spin-4096-16` | 4605.99 | 4514.83 | 120.07 |
| `spin-4096-64` | 5040.90 | 4716.02 | 316.06 |
| `spin-16384-0` | 17685.93 | 17596.00 | 84.00 |
| `spin-16384-16` | 18055.67 | 17887.59 | 143.99 |
| `spin-16384-64` | 18136.37 | 17797.95 | 352.08 |

and the mixed program's three builds, with the same spin in the same place:

| form | seq | iocp | full |
|---|---|---|---|
| `spin-0-0` | 257.99 | 255.53 | 172.14 |
| `spin-0-16` | 266.12 | 265.20 | 167.51 |
| `spin-0-64` | 289.07 | 291.49 | 172.45 |
| `spin-16-0` | 258.47 | 259.26 | 172.11 |
| `spin-16-16` | 266.01 | 267.18 | 168.92 |
| `spin-16-64` | 291.39 | 289.40 | 172.01 |
| `spin-64-0` | 262.33 | 263.59 | 170.27 |
| `spin-64-16` | 272.04 | 273.95 | 168.89 |
| `spin-64-64` | 296.67 | 296.74 | 179.46 |
| `spin-256-0` | 281.63 | 282.49 | 183.56 |
| `spin-256-16` | 289.46 | 290.04 | 190.87 |
| `spin-256-64` | 312.76 | 314.45 | 195.19 |
| `spin-1024-0` | 343.34 | 350.40 | 446.68 |
| `spin-1024-16` | 361.74 | 351.64 | 445.36 |
| `spin-1024-64` | 387.10 | 385.73 | 468.42 |
| `spin-4096-0` | 621.47 | 634.35 | 1883.37 |
| `spin-4096-16` | 637.36 | 635.96 | 1889.26 |
| `spin-4096-64` | 653.17 | 667.34 | 1884.18 |
| `spin-16384-0` | 1757.48 | 1743.90 | 7546.73 |
| `spin-16384-16` | 1780.22 | 1775.25 | 7594.82 |
| `spin-16384-64` | 1824.01 | 1766.87 | 8255.66 |

Sixteen rounds cost the I/O workload 19 percent and four thousand cost it
forty-one times. The unified mixed build at four workers reaches 8.6 seconds
against 0.17, because four workers spinning in front of the drain also
contend for the core's one mutex on every round: its system time goes from
71 ms to 15 seconds.

So the spin moved to where the second reading of the same sentence puts it:
**after the drain and after the window's own last look, immediately before
`wf_prim_park`**. The epoch capture stays where it was, in front of the drain,
so every look the spin makes is a look after the capture and §6's lost-wake
argument is the argument it was -- a publisher that acts after the capture
either moved the epoch, which makes the park return at once, or left a push one
of the spin's looks finds. What that placement buys is the whole difference: a
turn that had something to find has already found it at the drain and never
reaches the spin at all, so the I/O line stops paying for it. Every table below
is that placement.

### 1. The mixed program, the three builds the Windows bench compares

`../io-completion-bench/programs/windows_runtime_mixed.wf` at `WF_WORKERS=4`,
built `--no-overlap` (`seq`), plain (`iocp`) and `--par` (`full`), over two
16 MiB files, medians of 9. `full/iocp` is the cohort the Windows bar reads as
`mixed-full`.

| form | seq | iocp | full | full/iocp | full user | full sys |
|---|---|---|---|---|---|---|
| `spin-0-0` | 257.21 | 240.40 | 169.96 | 0.707 | 285.31 | 71.32 |
| `spin-0-16` | 256.91 | 256.54 | 161.18 | 0.628 | 262.94 | 88.62 |
| `spin-0-64` | 248.02 | 244.28 | 155.58 | 0.637 | 282.50 | 80.24 |
| `spin-16-0` | 256.15 | 254.54 | 170.31 | 0.669 | 267.60 | 85.19 |
| `spin-16-16` | 256.90 | 255.87 | 164.91 | 0.645 | 282.73 | 71.53 |
| `spin-16-64` | 245.47 | 256.33 | 157.48 | 0.614 | 295.41 | 71.72 |
| `spin-64-0` | 251.39 | 256.64 | 170.59 | 0.665 | 302.98 | 73.79 |
| `spin-64-16` | 242.26 | 256.57 | 161.95 | 0.631 | 274.18 | 82.33 |
| `spin-64-64` | 253.35 | 257.34 | 162.46 | 0.631 | 318.92 | 87.33 |
| `spin-256-0` | 257.35 | 249.77 | 165.37 | 0.662 | 343.58 | 104.85 |
| `spin-256-16` | 256.24 | 246.29 | 163.07 | 0.662 | 367.26 | 76.50 |
| `spin-256-64` | 247.73 | 251.32 | 158.28 | 0.630 | 398.83 | 102.21 |
| `spin-1024-0` | 246.98 | 254.60 | 153.77 | 0.604 | 432.30 | 103.02 |
| `spin-1024-16` | 254.52 | 256.34 | 156.16 | 0.609 | 434.60 | 94.88 |
| `spin-1024-64` | 256.76 | 253.84 | 156.14 | 0.615 | 427.87 | 91.69 |
| `spin-4096-0` | 243.55 | 256.04 | 152.94 | 0.597 | 435.53 | 92.34 |
| `spin-4096-16` | 256.73 | 253.29 | 155.98 | 0.616 | 452.91 | 110.06 |
| `spin-4096-64` | 257.08 | 258.50 | 164.12 | 0.635 | 435.81 | 111.72 |
| `spin-16384-0` | 253.80 | 255.87 | 154.48 | 0.604 | 449.79 | 112.45 |
| `spin-16384-16` | 253.44 | 254.71 | 159.21 | 0.625 | 453.24 | 111.38 |
| `spin-16384-64` | 256.62 | 249.40 | 154.75 | 0.620 | 443.76 | 102.73 |

The two reference builds are flat across the whole grid -- they hand out
nothing, so their idle turns end at the drain and never reach the spin -- and
the unified build falls from 169.96 ms to about 154, 10 percent, monotonically
in the pause rounds and then flat from 1024 up. Its user CPU rises with the
same curve, from 285 ms to about 450: the spin buys the wall time with a core.

### 2. §12 item 1's table, with the spin

`tests/programs/par_layout.wf --par` and `programs/grid_split.wf`, medians of 9,
`W=8 min` and the two CPU columns given because this host was noisy for these
two programs on the night of the sweep (the four-form table below is the same
comparison at N=15, and is the one to read).

par_layout:

| form | W=1 | W=2 | W=4 | W=8 | W=8 min | W=8 user | W=8 sys |
|---|---|---|---|---|---|---|---|
| `spin-0-0` | 1538.55 | 858.38 | 815.85 | 1007.85 | 805.82 | 1706.78 | 512.89 |
| `spin-0-16` | 1556.58 | 828.76 | 639.91 | 610.63 | 480.21 | 1628.47 | 100.42 |
| `spin-0-64` | 1534.33 | 970.92 | 813.66 | 751.10 | 448.59 | 1543.36 | 49.72 |
| `spin-16-0` | 1555.09 | 862.51 | 1144.36 | 979.12 | 786.76 | 1735.20 | 481.71 |
| `spin-16-16` | 1552.36 | 860.10 | 769.95 | 641.50 | 461.15 | 1592.21 | 104.21 |
| `spin-16-64` | 1561.51 | 895.54 | 778.44 | 613.37 | 453.51 | 1610.06 | 103.96 |
| `spin-64-0` | 1471.93 | 823.55 | 936.43 | 1062.48 | 668.32 | 1822.91 | 472.08 |
| `spin-64-16` | 1560.62 | 1000.14 | 908.37 | 727.04 | 454.50 | 1600.45 | 45.95 |
| `spin-64-64` | 1568.13 | 979.96 | 877.54 | 721.12 | 449.43 | 1613.57 | 51.73 |
| `spin-256-0` | 1556.45 | 892.67 | 969.14 | 845.96 | 549.30 | 1973.79 | 322.33 |
| `spin-256-16` | 1530.15 | 1055.57 | 833.71 | 797.92 | 477.76 | 1623.84 | 62.14 |
| `spin-256-64` | 1557.21 | 887.19 | 770.18 | 711.39 | 476.59 | 1624.40 | 100.10 |
| `spin-1024-0` | 1503.47 | 1142.44 | 1066.83 | 845.86 | 613.56 | 2094.91 | 331.20 |
| `spin-1024-16` | 1558.10 | 1076.17 | 882.33 | 716.67 | 550.99 | 1755.12 | 178.60 |
| `spin-1024-64` | 1547.79 | 801.29 | 632.73 | 662.90 | 512.84 | 1737.22 | 180.70 |
| `spin-4096-0` | 1549.16 | 814.16 | 639.41 | 752.16 | 545.44 | 1926.89 | 345.18 |
| `spin-4096-16` | 1514.59 | 810.91 | 648.11 | 775.30 | 697.62 | 1908.21 | 257.72 |
| `spin-4096-64` | 1561.01 | 808.12 | 646.60 | 750.54 | 587.39 | 1896.96 | 280.90 |
| `spin-16384-0` | 1552.89 | 793.60 | 626.38 | 749.73 | 570.89 | 1902.16 | 367.18 |
| `spin-16384-16` | 1460.31 | 815.12 | 660.44 | 807.08 | 529.57 | 2016.33 | 394.77 |
| `spin-16384-64` | 1541.11 | 807.44 | 642.02 | 766.82 | 564.93 | 2010.66 | 292.51 |

grid:

| form | W=1 | W=2 | W=4 | W=8 | W=8 min | W=8 user | W=8 sys |
|---|---|---|---|---|---|---|---|
| `spin-0-0` | 1519.46 | 769.86 | 632.27 | 706.78 | 380.55 | 1507.63 | 8.07 |
| `spin-0-16` | 1477.50 | 740.44 | 682.47 | 687.30 | 364.30 | 1453.29 | 7.96 |
| `spin-0-64` | 1456.37 | 745.38 | 734.51 | 659.00 | 367.73 | 1488.56 | 3.97 |
| `spin-16-0` | 1469.74 | 741.94 | 701.60 | 700.70 | 370.98 | 1471.41 | 4.01 |
| `spin-16-16` | 1485.59 | 780.26 | 709.94 | 692.83 | 384.47 | 1513.77 | 7.96 |
| `spin-16-64` | 1513.69 | 779.02 | 662.46 | 695.95 | 377.43 | 1501.02 | 7.28 |
| `spin-64-0` | 1439.40 | 746.88 | 611.37 | 669.82 | 359.83 | 1474.97 | 8.07 |
| `spin-64-16` | 1514.14 | 764.87 | 735.28 | 628.70 | 376.65 | 1514.97 | 4.00 |
| `spin-64-64` | 1499.05 | 761.62 | 720.26 | 703.44 | 381.86 | 1503.54 | 4.00 |
| `spin-256-0` | 1465.18 | 750.10 | 699.66 | 699.22 | 367.06 | 1481.14 | 3.98 |
| `spin-256-16` | 1524.86 | 769.86 | 689.42 | 687.70 | 391.36 | 1513.01 | 4.00 |
| `spin-256-64` | 1520.24 | 768.40 | 596.58 | 533.54 | 385.96 | 1488.22 | 7.97 |
| `spin-1024-0` | 1419.64 | 750.66 | 515.63 | 508.95 | 368.33 | 1467.31 | 8.03 |
| `spin-1024-16` | 1511.97 | 767.52 | 516.38 | 516.79 | 370.72 | 1457.07 | 4.01 |
| `spin-1024-64` | 1457.65 | 769.62 | 526.55 | 514.11 | 368.02 | 1514.26 | 0.00 |
| `spin-4096-0` | 1485.65 | 753.21 | 515.63 | 513.01 | 374.85 | 1482.39 | 7.99 |
| `spin-4096-16` | 1502.68 | 758.38 | 539.95 | 518.93 | 382.38 | 1509.59 | 8.05 |
| `spin-4096-64` | 1505.99 | 765.46 | 514.37 | 525.18 | 382.02 | 1515.45 | 8.06 |
| `spin-16384-0` | 1481.62 | 748.86 | 527.83 | 509.15 | 376.64 | 1484.34 | 16.00 |
| `spin-16384-16` | 1506.27 | 769.48 | 533.05 | 534.43 | 384.13 | 1489.33 | 15.84 |
| `spin-16384-64` | 1520.64 | 770.50 | 488.50 | 466.39 | 385.93 | 1508.60 | 15.94 |

### 3. The I/O line

io-completion-bench `many_files_wide` at the default helper policy, 8192 files,
best of 7.

| form | median ms | min ms | user ms | sys ms |
|---|---|---|---|---|
| `spin-0-0` | 115.56 | 113.38 | 72.22 | 44.13 |
| `spin-0-16` | 113.44 | 108.83 | 66.76 | 44.27 |
| `spin-0-64` | 115.54 | 100.54 | 68.27 | 44.20 |
| `spin-16-0` | 114.57 | 103.98 | 75.94 | 39.97 |
| `spin-16-16` | 113.07 | 104.18 | 69.82 | 36.97 |
| `spin-16-64` | 115.17 | 111.74 | 72.82 | 43.72 |
| `spin-64-0` | 112.53 | 107.05 | 75.18 | 39.91 |
| `spin-64-16` | 113.23 | 107.28 | 78.01 | 35.44 |
| `spin-64-64` | 112.84 | 108.59 | 73.75 | 40.23 |
| `spin-256-0` | 109.73 | 102.09 | 70.50 | 40.72 |
| `spin-256-16` | 112.57 | 104.88 | 81.72 | 32.62 |
| `spin-256-64` | 111.73 | 101.50 | 71.36 | 39.86 |
| `spin-1024-0` | 114.01 | 107.50 | 79.94 | 35.97 |
| `spin-1024-16` | 114.03 | 106.95 | 68.05 | 48.24 |
| `spin-1024-64` | 113.04 | 104.48 | 74.89 | 39.27 |
| `spin-4096-0` | 115.40 | 103.90 | 71.49 | 43.69 |
| `spin-4096-16` | 114.14 | 110.10 | 70.72 | 43.18 |
| `spin-4096-64` | 114.37 | 109.54 | 78.99 | 36.38 |
| `spin-16384-0` | 115.41 | 110.89 | 71.77 | 43.70 |
| `spin-16384-16` | 115.61 | 108.28 | 71.75 | 43.79 |
| `spin-16384-64` | 114.06 | 108.31 | 69.04 | 47.05 |

**Flat across the entire grid**, 16 384 pause rounds included: 109.73 to
115.61 ms against 115.56 for `spin-0-0`, inside the spread of every row. This
is the placement doing its work -- compare the same line in front of the drain,
which reached 17.7 seconds.

### 4. The park and publish round trip

`park_publish.c` through the shipped core over the host primitives, 50 000
round trips per pass, 15 interleaved passes, nanoseconds.

| form | settled best | settled median | racing best | racing median |
|---|---|---|---|---|
| `spin-0-0` | 1667 | 6343 | 4210 | 6100 |
| `spin-0-16` | 1391 | 1776 | 1576 | 2083 |
| `spin-0-64` | 1274 | 1872 | 1571 | 2265 |
| `spin-1024-0` | 2373 | 4472 | 1868 | 3762 |
| `spin-1024-16` | 2316 | 3789 | 2151 | 3410 |
| `spin-1024-64` | 2698 | 3660 | 2094 | 3470 |
| `spin-16-0` | 2256 | 3507 | 1884 | 3154 |
| `spin-16-16` | 2696 | 3635 | 2309 | 3370 |
| `spin-16-64` | 3065 | 3833 | 2581 | 3300 |
| `spin-16384-0` | 2459 | 3912 | 2409 | 3259 |
| `spin-16384-16` | 1510 | 4203 | 2368 | 3338 |
| `spin-16384-64` | 1641 | 3972 | 1725 | 3770 |
| `spin-256-0` | 2869 | 3990 | 2558 | 4049 |
| `spin-256-16` | 1446 | 3379 | 2154 | 3194 |
| `spin-256-64` | 2630 | 3421 | 1514 | 3424 |
| `spin-4096-0` | 2288 | 3921 | 2767 | 4120 |
| `spin-4096-16` | 2441 | 3850 | 2564 | 3356 |
| `spin-4096-64` | 2443 | 3501 | 2714 | 4078 |
| `spin-64-0` | 2515 | 3956 | 2245 | 3921 |
| `spin-64-16` | 2607 | 3563 | 2723 | 4423 |
| `spin-64-64` | 2044 | 4301 | 2372 | 3138 |

Every spin form is two to three times faster than `spin-0-0` on the median,
which is the whole mechanism in one number: the thread that would have slept
finds the publication by looking. The forms do not order cleanly among
themselves, and `spin-0-16` -- no pause rounds at all -- is the fastest of them
here.

### 5. The chosen constants against the shipped form, at N = 15

Four forms, the same lines, 15 recorded passes instead of 9, which is the
comparison the choice is made on.

| form | mixed seq | mixed iocp | mixed full | full/iocp | full user |
|---|---|---|---|---|---|
| `spin-0-0` | 256.45 | 256.53 | 170.68 | 0.665 | 254.97 |
| `spin-0-16` | 256.40 | 257.21 | 165.10 | 0.642 | 277.75 |
| `spin-256-16` | 256.73 | 252.63 | 160.91 | 0.637 | 410.39 |
| `spin-4096-16` | 256.15 | 244.39 | 154.10 | 0.631 | 457.40 |

| form | W=1 | W=2 | W=4 | W=8 | W=8 sys |
|---|---|---|---|---|---|
| `spin-0-0` | 1526.49 | 830.82 | 562.75 | 802.02 | 625.75 |
| `spin-0-16` | 1551.92 | 815.33 | 482.87 | 457.48 | 139.63 |
| `spin-256-16` | 1530.65 | 801.73 | 451.52 | 492.70 | 229.49 |
| `spin-4096-16` | 1554.64 | 800.06 | 453.60 | 566.55 | 356.71 |

| form | grid W=4 | grid W=8 | many_files_wide | park settled median |
|---|---|---|---|---|
| `spin-0-0` | 391.27 | 389.34 | 114.86 | 6320 |
| `spin-0-16` | 377.98 | 377.43 | 115.28 | 1980 |
| `spin-256-16` | 387.87 | 390.36 | 112.94 | 4107 |
| `spin-4096-16` | 386.61 | 390.60 | 118.79 | 3662 |

### 6. The enumerator

`make -C compiler completion-test`'s `sched-enumerate`, at the four
configurations §11 derives, before the spin and with it. The enumerate build
pins the spin to one round and no yields, and `compiler/Makefile` states why.

| configuration | states before | states with the spin | wall before | wall with |
|---|---|---|---|---|
| (T=1,S=2) | 4 881 | 5 042 | 0.03 s | 0.03 s |
| (T=1,S=3) | 7 048 | 7 284 | 0.05 s | 0.04 s |
| (T=2,S=3) | 56 322 113 | 329 036 936 | 20.6 s | 47.9 s |
| (T=2,S=4) | 434 643 223 | 4 180 757 235 | 46.9 s | 152.5 s |

Every schedule passes at every configuration, and the coverage assertions §11
makes are met. Two things had to be pinned to get there, and both are
statements about the enumerator's model rather than about the spin:

- **A second spin round costs the search 24 times the states at (T=2,S=3).**
  One round reaches both of the spin's arms -- a look that hits inside it, and
  the fall through it to the park -- so one round is what the enumerate build
  takes.
- **A yield round breaks S10a.** The enumerator makes a yield block until
  another process has written shared state, which is the only way a spin can
  observe anything in a sequentially consistent model. A yield in front of the
  park therefore forces every device completion ahead of the park, and the one
  thread never sleeps on the primitive at all; S10a asserts that it does, and
  fails with "not covered: the one thread never slept on the primitive" at one
  yield round with no pause rounds. The yield is a delay like the pause, and
  the looks around it are what §11 judges, so the enumerate build takes none.

### The choice, and why

**256 pause rounds and 16 yield rounds**, which is what
`compiler/src/backend/sched/core.h` now carries.

What the numbers say, line by line, at N = 15:

- Nothing regresses. The two I/O-only mixed builds, the grid at every worker
  count and `many_files_wide` are inside their own spread at every form of the
  grid, because the spin sits after the drain and a turn with work to find
  never reaches it.
- The unified mixed build, which is the build the Windows bar fails on, falls
  from 170.68 ms to 160.91 at the chosen form and 154.10 at 4096 rounds, and
  its ratio against the completion-only build from 0.665 to 0.637 and 0.631.
- `par_layout`, which is §12 item 1's open regression, falls from 562.75 ms to
  451.52 at four workers and from 802.02 to 492.70 at eight, with its system
  time at eight workers falling from 626 ms to 229. Against the 406.7 ms and
  421.3 ms the tree read before park on miss, the gap the plan records as
  45 and 103 percent becomes 11 and 17.
- The park and publish round trip falls from 6 320 ns to 4 107.

Why 256 rather than the retired runtime's 4096. A look round costs about 43
nanoseconds here -- the placement-A mixed table divides out to that -- so 256
rounds is a window of about 11 microseconds against this host's own
park-and-wake of 16.2. That is exactly the floor the retired runtime's comment
argues for, that a thread should not sleep to save less than the sleep costs,
applied to the machine that was measured rather than to the 2.2 microsecond
machine that comment was written on. Above it the sweep buys a further 4
percent on the mixed program and pays for it twice: `par_layout` at eight
workers on four cores goes back up (492.70 at 256 rounds, 566.55 at 4096, and
in the wider sweep 775.30), because a pause-spin on an oversubscribed pool
takes a core from a thread that had work, and the idle CPU rises from 410 ms to
457 on the mixed build. 16 yield rounds is the retired runtime's own count and
the sweep gives no reason to move it: the yield rounds are where most of
`par_layout`'s gain comes from and they cost nothing on any other line.

**The real judge is the Windows job**, not this host. Every number above is a
four-core Linux VM whose park-and-wake is 16.2 microseconds; the bar that is
failing is a four-vCPU Windows VM whose wake is a completion-port round trip,
which is the case the spin exists for and the case this host cannot produce.
The two constants are `#if !defined` overrides for exactly that reason: the
bench can sweep them where the bar is, and if that sweep names another point of
this grid, the number to change is one define and the record to update is this
section.

**The judge ran, 2026-09-05, on 6311482** (`bench-windows-qualified`, run
33998108257, a four-vCPU AMD EPYC 7763 Windows Server 2025 VM, 15 recorded
alternating pairs after 2 warm-ups), with the chosen 256 and 16:

| cohort | reference median ms | candidate median ms | paired candidate/reference | MAD | p10..p90 |
|---|---:|---:|---:|---:|---:|
| compute | 4720.349 | 1331.076 | 0.2820 | 0.68% | 0.2790..0.2853 |
| io-warm | 208.083 | 198.221 | 0.9502 | 2.18% | 0.9116..0.9767 |
| mixed-iocp | 274.971 | 275.100 | 1.0006 | 0.42% | 0.9942..1.0075 |
| mixed-full | 274.822 | 190.109 | 0.6920 | 0.40% | 0.6867..0.6976 |
| mixed-total | 275.373 | 190.010 | 0.6878 | 0.66% | 0.6820..0.6974 |

Every bar met, and the two that were missing are met with the whole p10..p90
band under 0.70: the unified build is 31 percent faster than its
completion-only build on that host, which is the ratio this host gives (0.637
at 256-16 in table 5). One caution on reading the motivating record against
this one. The run that immediately preceded the spin (2d455e5, run
33996963197, an EPYC 9V74 runner) also met the bars without it, at mixed-full
0.8603 and mixed-total 0.8582, with the completion-only reference itself at
307 ms against the 274 ms of the record above and the 274 ms of this run; so
the Windows VM's numbers move between runners by more than the bar's margin,
and the spin's own evidence is the 0.69 of its own run beside the 0.86 and the
1.06 of the two runs without it, not the bar alone.

### What this measurement says the next work is

The cost of a spin round here is a global mutex: every round takes the core's
one lock to look at the ready list (`wf_sched_ready_pop`). That is why a round
costs 43 nanoseconds rather than the handful of loads the retired runtime's
lane-local spin cost, why the pause rounds stop paying above about a thousand,
and why four spinning workers in front of the drain could reach 15 seconds of
system time in the placement-A table. A ready-list look that does not take the
mutex -- §12 item 6's per-thread ready list is one shape of it, and a lock-free
head hint is another -- would make every point of this grid cheaper and is the
thing to measure before anyone widens the window.

## What this bundle holds

- `README.md` — this file: the method, every table, the commands and the bars.
- `run.sh` — every line above that a script can still produce, in eight
  sections (`park compute lanes io chain stacks ledger spin`). The two sections
  that swept the compile-time forms, `gates` and `liveness`, went with the forms
  at slice 4b; their tables are sections 0 and 1 above. `spin` is the addendum's
  sweep: the one section whose forms are builds of the core with its two spin
  constants overridden, one of which is the form the tree now ships.
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
- **Nothing was chosen when these numbers were taken, and the choosing came
  after them.** No default moved on 2026-09-05, and the shipped build compiled
  exactly the form the §11 enumerator checked. Slice 4b then applied the plan's
  rule to this record: every losing form is deleted, `WF_SCHED_LANE_SLOTS` and
  the shipped 64 stay, and one item is left open for the owner — §12 item 1's
  compute-miss regression and its fallback.

- **The compute programs were noisy on the night of the sweep.** `par_layout`
  and the grid spread by a factor of two to three across the nine passes of the
  twenty-one-form sweep — mins around 440 ms at four workers against medians
  between 450 and 1150 — which is why the choice is made on the four-form table
  at N = 15 rather than on the sweep's own medians, and why the sweep's compute
  tables carry a `min` column. The mixed program, the I/O line and the park
  round trip were not noisy in the same way.
- **The sweep was taken from a copy of the tree, not from the working tree.**
  The measurements were run against the branch's commit `6884dd2`, exported to
  a scratch directory, because the working tree carried a large unrelated
  in-progress change while the sweep ran and its programs did not compile. The
  branch has since advanced over that commit -- specification v0.46's rename of
  the file API, and the floor attach behind `wf_prim_floor_attach` -- and the
  numbers were not retaken, because neither reaches the scheduler core's idle
  window. Every gate that judges the change was run on the later tree.

## Reproducing

    make -C compiler park-on-miss-measurements
    make -C compiler park-on-miss-measurements SECTIONS="park compute"

or the script directly, from anywhere:

    sh research/experiments/park-on-miss-measurements/run.sh
    sh research/experiments/park-on-miss-measurements/run.sh chain ledger

The addendum's sweep alone, which is the longest section here:

    make -C compiler park-on-miss-measurements SECTIONS=spin

Its grid is `SPIN_ROUNDS` and `SPIN_YIELDS`, and a shorter run is one of those:

    SPIN_ROUNDS="0 256" SPIN_YIELDS=16 \
        sh research/experiments/park-on-miss-measurements/run.sh spin

Sections 0 and 1 were taken with `SCHED_VARIANT_DEFINES`, which no longer
exists; the one form the tree now has is gated by `make -C compiler format
lint` and `make -C compiler completion-test` like any other change to the core.

The chain alone:

    make -C research/experiments/io-completion-bench chain

The target is outside `make check` and nothing in the repository depends on
what it prints.

## Removal condition

Slice 4b removed the compile-time switches; the bundle stays for what it still
measures and for the record of what was measured. It is removed when §12 item 1
is answered — the owner's open decision on the compute-miss regression and its
fallback, which needs the target-action bit at the hand-out — and §12 item 4's
chain bar is answered or retired, since those are the two items whose numbers
are not yet a conclusion. `programs/grid_split.wf` goes with it unless the
corpus adopts it; `chain.c` goes with §12's fourth item.
