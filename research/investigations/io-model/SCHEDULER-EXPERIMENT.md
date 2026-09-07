# Scheduler experiments

Question: does the choice of worker that resumes a parked stack materially
affect the network gap, without changing the source model or stack representation?
This experiment starts from the runtime merged as `2e84ff44`. The first
measurement does not select either local-queue policy as a replacement:
low-connection throughput regresses and higher-connection throughput is flat.
This result concerns queue preference under the existing shared runtime;
it does not decide a complete per-worker execution design.
The current branch replaces those unselected queue variants with the second
experiment below, which isolates idle progress and CPU placement. The first
experiment remains reproducible at its recorded revision.

The existing `RESULTS.md` third network series tested per-worker rings while
retaining the global ready queue. It did not test a complete per-worker
execution design. This first experiment separates queue locality from lock
sharding and ring ownership; those remain later independent variables.

## First experiment: forms at b714ced7

The same compiler emits each program once. The runtime is linked three times:

| policy | ready queues | destination | when local queue is empty |
| --- | --- | --- | --- |
| 0 | one global FIFO | queue 0 | no other ready queue |
| 1 | one FIFO per worker | worker that parked the stack | scan all other queues |
| 2 | one FIFO per worker | worker that enqueues the stack | scan all other queues |

All forms keep the existing shared mutex, shared wake epoch, shared Linux
ring, progress policy, lane deques, stack count and stack representation.
An empty-to-nonempty transition of the union of ready queues wakes through the
shared epoch. A count under the same mutex preserves the original wake policy
in all three forms; the enumerator checks it against actual list membership.
Every pop checks all queues under the mutex before concluding no ready
work exists. A ready stack remains globally available: this is a preference,
not pinning. Policy 2 includes the worker committing an early notification;
adapter helpers have no worker identity and use queue 0. It is not a promise
that one worker receives every completion of a connection.

The stack records its parking worker before publishing SUSPENDING. The
existing handshake publishes that field to the enqueue, and the queue mutex
publishes it to the pop. The field is never changed while the stack is parked.
The record claim, DONE-last discipline, stack phases and memory orders stay
the same. The enumerator snapshots the queue arrays in the core and the
parking field in the stack header. Its invariants now check every queue's
tail, disjoint membership, valid destination and the union of ready queues
when checking that all workers cannot sleep beside ready work.

## First experiment: validation and measurement at b714ced7

`make scheduler-experiment` is part of root `make check`. It runs all existing
schedules at (1,2), (1,3), (2,3), (2,4) for every policy. The original compiler
gate also checks the default policy. The new gate matrix stage runs this on
Linux and macOS; production default changes also reach the existing native
Windows checks. This does not qualify the experimental policies on Windows.

`make -C research/experiments/io-completion-bench scheduler-bench` runs on
Linux. `.github/workflows/io-scheduler.yml` runs it on one hosted runner:

- One compiler revision, one host, explicit equal network worker counts,
  two warm-up passes, seven measured passes with reversed alternate order.
- Echo: 1, 4, 64, 1024 connections at 64 bytes, and 64 connections at 64 KiB.
  The existing io_uring and epoll references run in the same cohort.
- Every network sample verifies every echoed byte and both process exits;
  unexpected output or diagnostics fail the measurement.
- Throughput, p50/p99 latency, server CPU per round trip, peak resident memory
  and OS context switches are retained per sample. Ratios use the global FIFO
  in the same pass, connection count and payload; ranges accompany medians.
- Separate untimed observed links report parks, resumes, cross-worker resumes
  and ring counters. Instrumentation is absent from the timed binaries.
- Existing `par_layout` and `windows_runtime_mixed` run at 2, 4, 8 workers,
  with their existing expected output bytes and the paired C timing harness.
  The latter uses warm positioned reads. It measures compute/I/O coexistence,
  not network tail latency under long CPU work; that workload is still needed
  before selecting an overall mixed-load design.

CI duration limits fail an incomplete experiment. They never decide compiler
acceptance or turn an incomplete sample into a performance result. Host
metadata, raw samples, diagnostics and observed counters are artifacts; copy
the decision-relevant results and run URL into this document before those
artifacts expire. A client-limited plateau cannot establish executor parity.

## First measurement: 2026-09-06

Measured revision: `b714ced72a8f3e613a708a47edbbf1ec27093a44` on
`codex/io-scheduler-experiments`.
[Measurement run](https://github.com/mbbill/Whitefoot/actions/runs/34026973217)
and [raw artifact](https://github.com/mbbill/Whitefoot/actions/runs/34026973217/artifacts/9987514023).
The host reports a Xeon Platinum 8573C, four logical CPUs presented as two
cores with two SMT threads each, Linux 6.17.0-1022-azure, and Clang 18.1.3.
The server and load generator share those CPUs, each with four workers and
without pinning. This is a hosted VM measurement, not a four-physical-core
server with an independent load generator. All WF forms use 1100 pool stacks.

Throughput ratios below are the median of seven candidate/global ratios
paired by pass. Brackets give their minimum and maximum, not confidence
intervals. They need not equal the ratio of the two throughput medians.
The cohort uses two warm-up passes followed by seven recorded passes; form
order reverses on alternate passes.

| connections / payload | global median round trips/s | parking-worker/global | enqueue-worker/global |
| --- | ---: | ---: | ---: |
| 1 / 64 B | 36,910 | 0.894 [0.866, 0.930] | 0.911 [0.891, 1.021] |
| 4 / 64 B | 133,282 | 0.851 [0.750, 1.028] | 0.809 [0.671, 1.361] |
| 64 / 64 B | 253,432 | 0.991 [0.975, 1.014] | 0.995 [0.967, 1.016] |
| 1024 / 64 B | 235,604 | 1.000 [0.974, 1.015] | 1.000 [0.973, 1.024] |
| 64 / 64 KiB | 54,701 | 1.005 [0.980, 1.332] | 0.998 [0.990, 1.312] |

At four connections, policy 1 loses six of seven pairs and policy 2 loses
four of seven. Policy 2 has considerable variation, so its 19% median loss
is not a precise prediction for another run. The large-payload maxima share
one slow global sample (41,249 round trips/s against its median 54,701);
they are not a repeatable candidate gain.

The same-cohort reference medians remain higher at multiple connections:

| connections / payload | io_uring round trips/s | epoll round trips/s |
| --- | ---: | ---: |
| 1 / 64 B | 31,488 | 32,175 |
| 4 / 64 B | 146,367 | 167,946 |
| 64 / 64 B | 298,079 | 299,506 |
| 1024 / 64 B | 328,355 | 315,590 |
| 64 / 64 KiB | 55,923 | 70,947 |

These references locate the gap on this host. They are different execution
designs, not a controlled change to WF's stack representation. Absolute rates
from the older development-host series must not be compared with these.

The low-connection regression also costs server CPU. Values include the
server process's whole lifetime, normalized by verified round trips; they
are not profiles of an individual queue operation.

| case | global CPU microseconds/trip | parking worker | enqueue worker |
| --- | ---: | ---: | ---: |
| 1 / 64 B | 82.500 | 95.500 | 93.000 |
| 4 / 64 B | 20.375 | 25.750 | 24.375 |
| 64 / 64 B | 7.969 | 8.047 | 8.047 |
| 1024 / 64 B | 8.643 | 8.691 | 8.643 |

At four connections, the reference CPU costs are 7.625 microseconds/trip
for io_uring and 7.250 for epoll. Global / parking / enqueue median p99
latencies are 127 / 139 / 143 microseconds, and OS context switches per
trip are 0.994 / 0.998 / 1.034. At 64 connections the p99 values are
680 / 683 / 677 microseconds. Median peak resident memory is about 32 MiB
at four connections and 78 MiB at 1024 for all three WF forms; this experiment
does not attribute that memory to necessary live continuation state.

### Did the preference reduce migration?

The separate observed binaries each ran once at four and 64 connections,
2000 trips per connection. A migration means the resuming worker differs
from the worker that parked that stack. These are diagnostic snapshots,
not repeated paired timing results. In particular, the four-connection
observed workload is shorter than the timed workload's 20,000 trips.

| connections | policy | resumes | cross-worker resumes | fraction |
| --- | --- | ---: | ---: | ---: |
| 4 | global | 662 | 190 | 28.7% |
| 4 | parking worker | 1,481 | 711 | 48.0% |
| 4 | enqueue worker | 2,484 | 1,218 | 49.0% |
| 64 | global | 114,017 | 63,064 | 55.3% |
| 64 | parking worker | 120,517 | 45,934 | 38.1% |
| 64 | enqueue worker | 119,761 | 77,239 | 64.5% |

Policy 1 reduces migration at 64 connections without a corresponding timed
throughput gain. At four connections neither policy even reduces migration
in the observed snapshot. Policy 2's destination is not persistent connection
affinity: a different worker can reap the next completion, and another worker
can steal the ready stack immediately.

The execution paths also vary in these snapshots. At four connections,
global / parking / enqueue perform 658 / 1482 / 2498 ring submissions and
15,351 / 14,527 / 13,511 immediate completions. At 64 connections their
ring-enter counts are 15,123 / 17,271 / 5651, while ring submissions are
114,181 / 120,730 / 119,981. A reduction in either migrations or enters alone
is therefore insufficient to predict the timed result. Instrumentation and
client/server timing can affect these counts; they do not establish the cause
of the low-connection regression.

### Compute, mixed load, and correctness

Median elapsed milliseconds, with every expected output verified:

| workload / workers | global | parking worker | enqueue worker |
| --- | ---: | ---: | ---: |
| compute / 2 | 3320.23 | 3321.55 | 3317.02 |
| compute / 4 | 1804.69 | 1808.58 | 1807.13 |
| compute / 8 | 1880.69 | 1923.65 | 1928.18 |
| mixed / 2 | 165.29 | 168.39 | 166.28 |
| mixed / 4 | 170.01 | 170.78 | 168.01 |
| mixed / 8 | 194.25 | 190.64 | 187.96 |

Compute is effectively unchanged at two and four workers; at eight workers
on four logical CPUs the candidates take about 2.3% and 2.5% longer. Their
system CPU time rises from 205 ms to 393 and 412 ms. The small warm-read mixed
differences do not select an overall mixed-load design: long compute lanes
competing with network completion latency have not been measured here.

On the measured revision, all three policies passed every existing enumerated
schedule at all four configurations locally. The complete
[repository gate](https://github.com/mbbill/Whitefoot/actions/runs/34026973188)
passed its Linux and macOS stages, including all three policy enumerators.
The [native I/O checks](https://github.com/mbbill/Whitefoot/actions/runs/34026973201)
passed on Linux and Windows for the production default. The Linux measurement
validated every echo and compute/mixed expected output, and all six observed
WF runs recorded native ring submissions and completions. Experimental
policy 1 and policy 2 have not been qualified on native Windows.

## Interpretation and removal

A gain accompanied by fewer cross-worker resumes supports locality as one
cost. A flat result does not reject per-worker execution: the shared mutex,
shared ring and eager stealing remain. Neither result decides stackful versus
stackless representation or whether source signatures need to change.

The first result rejects selecting queue preference alone as the performance
fix. It also corrects the earlier inference that placing work on the reaping
worker's queue necessarily keeps the connection on that worker. It neither
isolates the cost of the shared lock nor rules out persistent ownership of
connections, ready queues and completion engines by workers. Ownership/join
source semantics are unchanged in every form, so these data provide no reason
to introduce an async distinction in function signatures.

Before another scheduler redesign, the next comparison should account for
worker CPU, idle spinning/waking and server/load-generator CPU competition.
A locality redesign must demonstrate the intended ownership in counters,
then compare shared and sharded synchronization separately. The true mixed
network/long-compute workload remains necessary before choosing the runtime.

The narrower negative result is recorded in
`mcts_mem/whitefoot/system-interface.md`. The second experiment removes the
unselected ready-queue selectors and restores the original single-queue
enumerator invariants. No schedule or configuration is removed: the temporary
gate stage now checks the additional idle-progress transitions. The runner,
stage and workflow belong to this active investigation and are superseded in
place; remove unused variants at the next selection, and remove the temporary
gate stage and workflow if the investigation closes.

## Second experiment: idle progress and CPU placement

Question: how much of the network cost comes from workers repeatedly looking
for compute/ready work while new I/O completions wait undrained, and how much
comes from server/client CPU competition? The original idle window progresses
I/O once, then makes 256 spin looks and 16 yielding looks without progressing
I/O again. A completion arriving in that window must be drained by another
worker or wait for this worker to progress again. This is a hypothesis about
the cost, not a prediction that polling more often must win.

All forms use the original global FIFO, shared mutex, shared ring, lane and
stack representation, and the same emitted program. The old queue union count
is no longer needed. Each form is compared with a new baseline in its own
cohort, never with the first experiment's absolute rates.

| form | spin looks | yielding looks | additional progress interval |
| --- | ---: | ---: | ---: |
| base | 256 | 16 | none |
| sleep | 0 | 0 | none |
| short | 16 | 0 | none |
| spin | 256 | 0 | none |
| poll1 | 16 | 0 | every look |
| poll16 | 256 | 0 | every 16 looks |

The added progress pass stays inside the existing epoch-capture window. A
successful pass ends the idle registration and returns to the normal
scheduler loop, exactly as the original first progress pass does. Every
unsuccessful pass is followed by the same last look, and parking still uses
the original captured epoch. The canonical experiment stage enumerates every
existing schedule at (1,2), (1,3), (2,3), (2,4) with zero spin, one spin without
additional progress, and one spin with progress. The short windows cover the
protocol's transitions, not the timing of 256 physical pause instructions.

CPU placement is derived from the runner's actual allowed CPU list and
physical-core topology, recorded in `host.txt` and `cohorts.tsv`:

| cohort | server / client workers | CPU placement |
| --- | --- | --- |
| shared4 | 4 / 4 | both use all allowed logical CPUs |
| shared2 | 2 / 2 | both use all allowed logical CPUs |
| split2 | 2 / 2 | disjoint sets of two logical CPUs |
| split1 | 1 / 1 | one logical CPU each, on different physical cores |

Split2 can still share physical cores through SMT. Split1 avoids that
server/client sharing but tests only one server worker. These cohorts separate
specific resource conditions; comparing a two-worker and four-worker cohort
does not isolate affinity by itself. The io_uring and epoll references use the
same worker counts and CPU sets as WF within each cohort.

This screen uses 1, 4, 64 connections at 64 bytes, two warm-up passes and seven
recorded passes in alternating form order. The first series's 1024-connection
and large-payload measurements remain evidence; finalists must return to both
before selection. Network samples retain separate server and client CPU/RSS
and context-switch measurements, and every echo is still verified. Compute
and warm-read mixed workloads run for all six forms at 2, 4, 8 workers with
the established expected output bytes. The true network/long-compute mixed
workload is the third experiment below.

Separate untimed observed links record migration, idle steps, idle looks,
progress passes and waits, plus the existing ring counters. Observed builds
must report both the scheduler and native ring and exercise submissions and
completions. Idle counters can change while the exit observer reads them, so
all scheduler counter accesses now use relaxed atomic loads and stores. One
worker writes each counter; no atomic read-modify-write or scheduling edge is
needed. The snapshot is defined but is not simultaneous across workers, and
the enumerator still excludes diagnostic counters from its state digest.
Timed links compile out the extra idle counters and migration tracking.

Results are pending after the failed first attempt below. This experiment
leaves source function signatures and the default idle policy unchanged. The owner's broader
research instruction permits changing language design if measurements later
show a need; the current experiment does not assume that need in advance.

### Failed first attempt: one worker disabled I/O concurrency

The c088f4f0 run [34028842218](https://github.com/mbbill/Whitefoot/actions/runs/34028842218)
stalled before the timed cohort. Its gate and Linux/Windows host qualification
passed, which did not establish progress for this new protocol configuration.
The unchanged-runtime diagnostic e4a1d47f added sample logging and a 120-second
process deadline. [Run 34030120197](https://github.com/mbbill/Whitefoot/actions/runs/34030120197)
completed samples 1 through 62, then failed sample 63: split1/base, four
connections. Its [artifact](https://github.com/mbbill/Whitefoot/actions/runs/34030120197/artifacts/9988428245)
retains the reset on connection 1 and the server deadline. This is a failed
experiment, not a slow throughput sample; no timed policy ranking exists for
that revision.

The runtime normalized both `WF_WORKERS=0` and `1` to zero. The bootstrap then
selected the entire sequential clone, including connection handling. Netload
keeps every connection open until all exchanges finish: the sequential server
waits for the first peer's EOF while another peer waits for its response.
A native macOS two-peer reproduction independently confirmed that the second
response required the first EOF at one worker, but not at two.

The correction preserves a requested one worker and asks the bootstrap for
the minimum appropriate to its reachable lowering: one for staged I/O
hand-outs, two for compute-only hand-outs. Zero remains explicit sequential
opt-out. The query is an internal compiler/runtime ABI, with both target
bootstraps and the optional weak implementation changed together. Source
signatures are unchanged. The same native reproduction now verifies both
responses before either EOF. The maintained four-peer reverse-order test
also runs at both one and three workers, on both completion routes.

This is a configuration/lowering defect, not evidence that one worker can
support arbitrary I/O concurrency. A bounded window, exhausted frame slots,
or exhausted stacks can still postpone a callee whose peer is needed by
another callee. Optional compute scheduling and externally required I/O
progress are therefore distinct design obligations. The present measurements
reserve 1100 stacks and use at most 64 peers; they do not settle admission or
progress at capacity exhaustion.

Every network sample now has a 120-second external deadline. A deadline
fails the job and preserves diagnostics; it never produces a timing result
or changes compiler acceptance. Timing and resource collection run inside
the deadline wrapper. A 20-trip initial screen precedes the unchanged 2000-trip
idle observations and full timed passes. Client phase barriers now prevent
requests preceding the start timestamp or cleanup preceding the final CPU
snapshot. New baseline measurements are required after these harness changes.

The e3fa2a6a retry passed every four-peer screen, including split1, then
failed the first split1 observer check. [Its artifact](https://github.com/mbbill/Whitefoot/actions/runs/34031265140/artifacts/9988702498)
shows 8000 verified round trips, 8007 scheduler parks/resumes, and no ring
report. This was exit-observer ordering: without detached workers, bridge
shutdown destroys the engine before the constructor-registered observer runs.
The diagnostic report now remembers successful ring initialization separately
from current engine readiness. Atomic counters in static storage survive
teardown; the report reads those counters only, never destroyed descriptors
or mappings. The native-ring activity check remains mandatory. This retry
also reached no timed idle cohort.

A separate native macOS capacity probe with four open peers and one worker
verified 1, 2, 3, 4, 4, 4 responses before any EOF at stack counts 2, 3, 4, 5,
6, 9 respectively (500 ms observation window, correct response bytes in every
completed exchange). This is a concrete capacity witness, not a general
formula or a measured latency bound. Below five stacks, pending peers waited
while completed peers remained open, as the exhaustion path predicts.

## Third experiment: network service while connections compute

Question: does a long ordinary compute call prevent unrelated connections
from making progress, and how do throughput and short-request tail latency
trade off? `make -C research/experiments/io-completion-bench scheduler-mixed`
selects this mode of the same runner. CI runs idle and mixed in separate jobs;
each comparison uses one job's host and one CPU cohort, never absolute rates
from the other host.

`tcp_compute_server.wf` keeps sequential source inside each connection:
receive a complete request, compute, send a complete response, repeat. Its
outer accept loop uses the existing staged permission. Requests contain a
big-endian u64 seed and round count, followed by 48 reserved bytes. The
response contains the result's 64 bits as 64 bytes of zero or one. Each round
rotates the previous value left by 17, xors it with that value, multiplies
modulo 2^64 by 6364136223846793005, and adds 1442695040888963407 modulo 2^64.
The dependent recurrence cannot be replaced by independent parallel loop
iterations. TCP fragmentation is handled explicitly. The protocol refuses
more than 16777216 rounds as an ordinary input error.

Every fourth connection requests compute; other connections request zero
rounds. Seeds depend on the full connection and request indexes. The client
computes expected results before any connections are opened and verifies
every response byte. A separate Ruby calculation checked five C known answers
and native WF replies, including fragmented requests. Examples: seed zero
after one round is 1442695040888963407; seed 11400714819323198485 after seven
rounds is 2323064754341931374 and after 65536 rounds is 14034923464053623880.

The initial screen compares base, sleep, and poll1 with the epoll reference
performing the identical recurrence inline. It covers 4 and 64 connections,
0, 16384, 262144, and 2097152 rounds, under shared4 and split2 placement, with
two warm-up and seven alternating recorded passes. Separate observed builds
check real native-ring activity. The C protocol header, new WF program, and
conditional reference path belong to this comparison and are removed or
superseded with it.

The artifact retains aggregate and light/heavy p99 latency, each class's
exchange span, server CPU/RSS/switches, and client exchange CPU. Whole-process
client CPU includes preparing the oracle, so it is labeled lifetime CPU and
must not be interpreted as exchange cost. Server resource counters likewise
cover its lifetime, including startup and waiting for client preparation.
The mixed epoll reference uses 64-byte buffers, as the WF program does.
This is a finite closed-loop burst:
light connections may finish earlier than heavy ones, and the spans expose
that difference. It does not establish steady-state open-loop SLOs. Inline
epoll is a reference control, not an assertion of the best mixed-load design;
CPU offload, prioritization, full per-worker ownership, and continuation
representation remain candidates if losses justify them.

### First mixed-load results

Revision e3fa2a6a, [mixed job 101481087471](https://github.com/mbbill/Whitefoot/actions/runs/34031265140/job/101481087471),
[artifact 9988791681](https://github.com/mbbill/Whitefoot/actions/runs/34031265140/artifacts/9988791681).
The mixed job succeeded independently of the idle job's observer failure.
Its host was a four-logical-CPU Xeon Platinum 8573C VM, two physical cores with
SMT, Linux 6.17.0-1022-azure, clang 18.1.3. All 448 timed samples completed;
every case has seven paired passes, after two warm-up passes. Columns below
use the median of within-pass WF/inline-epoll throughput ratios and the
separate median light-request p99 values in microseconds.

| placement | peers | compute rounds | paired WF/C rate | WF light p99 | C light p99 |
| --- | ---: | ---: | ---: | ---: | ---: |
| shared4 | 64 | 0 | 0.801 | 917 | 2902 |
| shared4 | 64 | 16384 | 0.975 | 3153 | 3285 |
| shared4 | 64 | 262144 | 1.415 | 30457 | 36953 |
| shared4 | 64 | 2097152 | 1.401 | 203486 | 226662 |
| split2 | 64 | 0 | 0.865 | 471 | 323 |
| split2 | 64 | 262144 | 1.216 | 4664 | 5777 |
| split2 | 64 | 2097152 | 1.138 | 34581 | 39383 |
| shared4 | 4 | 2097152 | 0.999 | 126 | 4325 |

The heavy 64-peer shared4 paired ranges were 1.233..1.632 and 1.116..1.637;
the split2 ranges were 1.061..1.337 and 1.120..1.239. These are repeat ranges,
not confidence intervals. At four peers with heavy compute, the light-class
span was only 0.006 of the heavy span, so the low light p99 does not describe
continuous competing traffic. At 64 peers and the largest compute count,
that fraction was 0.875 in shared4 and 0.950 in split2.

Sleep and poll1 did not solve the mixed-load tail. At 64 peers and 2097152
rounds in shared4, their paired throughput relative to base was 1.004 and
0.992, while light p99 was 185366 and 217980 microseconds. At four peers and
zero compute, their paired rates were only 0.597 and 0.632 of base. A policy
that wins one placement cannot be selected for all loads from these readings.

This result supports keeping ownership-derived overlap as a useful source
model: in a compute-heavy case the current WF implementation outperformed
this native inline reference. It does not establish the best implementation.
The large tail can include both delayed first service of a connection and
worker occupancy after admission. Completion polling alone does not resolve
either CPU occupancy or fairness between ready stacks and queued callees.

## Fourth experiment: admission and cooperative compute reference

The next comparison separates those two causes. `scheduler-fairness` uses
the same mixed protocol and baseline WF code, with both fresh connections
and `netload --admit`. Admission sends one zero-compute request per connection,
verifies all responses, then releases the timed exchange through a barrier.
Every connection remains open. Admission time is reported separately; neither
the handshake nor its CPU work enters exchange timing or latency samples.

The C reference now has an optional explicit continuation form, compiled only
with `WF_BENCH_QUANTUM`. Each connection owns its value and remaining compute
count; each worker owns a FIFO of runnable connections. One turn executes at
most 1024, 16384, or 65536 recurrence steps, then enqueues the continuation
when work remains. The worker polls I/O between groups of at most eight
turns. A turn also yields after eight complete replies, so a stream of
zero-compute requests cannot bypass the scheduling boundary indefinitely.
No queue or continuation allocation occurs during the exchange. The original
inline reference remains a separate build of the same protocol path.

This is a candidate native implementation that WF must compete with, not a
new WF source requirement or a selected runtime policy. The measured controls
are base, inline epoll, and three native compute quanta, at 4/64 peers and
0/262144/2097152 rounds in shared4/split2. Both admission states receive two
warm-up and seven recorded passes. The summary key includes admission state,
so a fresh sample cannot be paired with an admitted baseline.

The distinction has precedents but no performance conclusion is imported.
[Seastar's documented scheduler](https://docs.seastar.io/master/tutorial.html#preemption-and-task-quota)
checks a task quota at explicit preemption points; long compute without such
points can starve its reactor. Its
[stackful thread form](https://docs.seastar.io/master/split/25.html) also keeps
sequential-looking code, with extra stack storage and cooperative yielding.
[Go's runtime](https://go.dev/src/runtime/preempt.go) instead supports
asynchronous safe points and signal-assisted preemption, with register state
and runtime-lock constraints. These are separate choices from proving that
two tasks may access disjoint state. A future WF lowering could insert its
own continuations or safe points without requiring user-written callbacks;
whether that cost is justified must be measured against these native forms.

The capacity witness also exposes a specification question independent of
these timing policies. PAR-3 explicitly permits an implementation to overlap
nothing, while interactive peer protocols can require independent callees to
be started before another callee completes. A required-concurrency scope with
backed admission, or a different progress contract, needs explicit design.
Silently increasing a fixed stack count cannot settle that semantic question.

### Fourth measurement: 4f951acc

[Run 34032286100](https://github.com/mbbill/Whitefoot/actions/runs/34032286100)
and [artifact 9989254493](https://github.com/mbbill/Whitefoot/actions/runs/34032286100/artifacts/9989254493)
contain 840 verified timed samples, all seven pairs per cell. The four-logical-
CPU Xeon 8573C VM again reports two physical cores with SMT, Linux 6.17 and
clang 18. This is a separate host cohort. Gate and native-host checks passed.
The separate Windows benchmark run 34032286063 failed its existing compute
timing-stability qualification after two cohorts; it did not report a wrong
result. That failure is retained, and the stability thresholds are unchanged.

At 64 connections and 2097152 recurrence steps per heavy request:

| placement / admission | WF rate/s | WF light p99 us | inline C light p99 us | q1024 light p99 us | q16384 light p99 us | q65536 light p99 us |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| shared4 / fresh | 3700 | 197970 | 261923 | 3100 | 3073 | 3588 |
| shared4 / admitted | 3697 | 162770 | 191536 | 3296 | 3053 | 3467 |
| split2 / fresh | 1997 | 32081 | 40263 | 442 | 796 | 1909 |
| split2 / admitted | 1998 | 32138 | 36067 | 358 | 780 | 2070 |

The admitted split2 WF/quantum throughput ratios, paired by pass, are 1.126
[1.003, 1.496], 1.118 [0.995, 1.361], and 1.119 [0.995, 1.245]. For admitted
shared4 they are 1.434 [1.201, 1.610], 1.430 [0.938, 2.076], and 1.400
[1.147, 2.361]. Brackets are sample minima/maxima, not confidence intervals.
WF is faster in the median here, but the explicit C continuations serve light
requests far sooner. Admission alone does not eliminate WF's tail, especially
in split2. With 262144 steps in admitted split2, light p99 is 4454 us for WF
versus 371, 789, and 2060 us for the three quanta: the effect is not confined
to the largest compute cost.

This is not yet a sustained-load tradeoff curve. In admitted split2 at the
largest compute cost, light/heavy active-span ratios are 0.942 for WF, but
0.008, 0.019, and 0.042 for the quantum references. Their finite light bursts
finish early, after which heavy requests run mostly alone. At four peers WF
also finishes the light class early (about 0.015 of the heavy span). Total
burst throughput rewards the distribution of remaining heavy work as well
as per-request efficiency: WF has shared work stealing, while each C worker
retains the connections assigned through its listener. Neither the low tails
nor the aggregate rate establishes a best sustained mixed-load design.

Selection: retain the admission control and add a common-duration closed-loop
comparison before selecting CPU checkpoints. Keep both class throughputs and
tails; a scheduler must not appear faster merely by serving more cheap work.
Compiler-inserted safe points and explicit native continuations remain
candidate implementations, not a source-level coloring requirement.

## Completed idle-policy comparison

Revision 06d93a46 on `codex/io-idle-retry` contains the e3fa2a6a experiment,
fe5f0656's report-only correction, and an idle-only workflow selection.
[Run 34031510418](https://github.com/mbbill/Whitefoot/actions/runs/34031510418)
succeeded; [artifact 9988994655](https://github.com/mbbill/Whitefoot/actions/runs/34031510418/artifacts/9988994655)
contains all 672 timed network samples, observed counters, and the compute
and warm-file mixed comparisons. Its host was a four-logical-CPU Xeon
Platinum 8573C VM with two physical cores and SMT, Linux 6.17.0-1022-azure,
clang 18.1.3. This is a new same-host baseline, not an absolute comparison
with either earlier VM.

| policy / workload | shared4 | shared2 | split2 | split1 |
| --- | ---: | ---: | ---: | ---: |
| poll16, one peer, paired rate/base | 1.154 | 1.553 | 1.549 | 0.794 |
| sleep, one peer, paired rate/base | 1.004 | 1.139 | 1.249 | 0.755 |
| poll16, four peers, paired rate/base | 1.019 | 1.143 | 1.109 | 1.008 |
| poll16, 64 peers, paired rate/base | 1.003 | 0.987 | 0.996 | 0.993 |

At 64 peers, none of the five idle variants materially closed the multi-worker
native-reference throughput gap: the base/uring paired ratios were about
0.77 in shared4/shared2 and 0.74 in split2. Split1 put all implementations near
212000 trips/s, with client exchange CPU near 4.7 microseconds/trip; one fully
used client CPU is a plausible ceiling there, so equal rates do not prove
equal server capacity. WF base server lifetime CPU was 4.766 microseconds/trip
there versus 4.453 for uring. At one peer in shared4, WF's low latency and
throughput advantage also spent much more CPU: base used 69 microseconds/trip,
uring 10. A blocking native control is not a sufficient best-latency reference
for a spinning runtime.

All compute and warm-file expected bytes agreed. The compute medians at four
workers were 1555.75 ms base, 1710.03 sleep, 1669.87 short, 1556.92 spin,
1669.54 poll1, and 1555.53 poll16. At eight workers on four logical CPUs they
were 1608.14, 2483.25, 2404.11, 2047.37, 2407.28, and 2007.09 ms respectively.
These are ratios of whole-program medians, not the network's within-pass
paired statistic. The yielding part of the original policy matters under
oversubscription; removing it is not a free network optimization.

Selection: retain the default 256-spin/16-yield policy for now. No tested
fixed replacement is selected. The 64-peer gap calls for measurements of
shared state, per-owner I/O and continuation costs; mixed-load tails call for
admission and CPU service fairness. A future adaptive policy must pay for its
own measurements and pass the pure-compute and oversubscribed controls, not
only the cell that motivated it. The idle variants and their extra gate
enumerations can be retired once the next scheduler change supersedes this
screen; the recorded counterexamples remain evidence.

## Fifth experiment: completing a record that is still private

The ready socket-transfer path currently pays the same `COMPLETING`, waiter
claim, and `DONE` publication as an engine completing on another thread.
However, this particular record is fresh, has not been offered to an engine,
and its caller cannot join until submit returns. The experiment enabled by
`WF_COMPLETION_LOCAL_INLINE=1` writes the same result head and releases `DONE`
directly. Socket transfers have no additional status-buffer publication.
The truly pending path retains the full scheduler protocol. Inline and
publication diagnostic counters are unchanged, so removing measurements
cannot explain any gain. The production default remains the original path
until this comparison selects a result.

`scheduler-inline` compares base and local with uring and epoll controls, all
four CPU placements, 1/4/64 peers at 64 bytes, 1024 peers at 64 bytes, and 64
peers at 64 KiB. It uses two warm-up and seven alternating timed passes,
plus mandatory observed native-ring activity. The maintained completion
harness and scheduler enumeration run with the experimental path enabled
before Linux timing. No source function signature or admission judgment
changes; this asks whether an unshared runtime record can avoid an
unnecessary synchronization protocol.

Native macOS links of both forms passed 128 independently checked fragmented
exchanges at four peers, with one and two workers, before any peer EOF. An
initial 16-peer baseline probe stalled despite 40 available stacks: this
host's bounded-helper route caps the staged window at eight. That probe
exceeded a separate capacity bound, so it is retained as a limitation rather
than attributed to the inline change. The correctness comparison uses four
peers within that window; the Linux measurements cover larger native-ring
windows explicitly.
The maintained macOS `completion-test` target also passed with the candidate
enabled: all core enumeration configurations, the default-route probe, helper
counts 0/1/4, the uncached case, and the pure-compute link boundary. Linux
qualification and paired timing completed in the following cohort.

### Fifth measurement: 95ba7202

[Run 34033148357](https://github.com/mbbill/Whitefoot/actions/runs/34033148357)
and [artifact 9989517524](https://github.com/mbbill/Whitefoot/actions/runs/34033148357/artifacts/9989517524)
contain 560 verified timed samples and the successful Linux completion harness,
native-adapter probe and core enumeration with the candidate enabled. This VM
reports AMD EPYC 7763, four logical CPUs on two physical cores with SMT, Linux
6.17.0-1022-azure and clang 18.1.3. Its absolute rates must not be compared with
the preceding Intel cohorts as though only the code changed.

Median within-pass local/base rate ratios:

| peers / bytes | shared4 | shared2 | split2 | split1 |
| --- | ---: | ---: | ---: | ---: |
| 1 / 64 | 1.008 | 1.003 | 1.001 | 1.005 |
| 4 / 64 | 0.989 | 1.001 | 1.005 | 1.011 |
| 64 / 64 | 1.006 | 1.017 | 1.007 | 1.005 |
| 1024 / 64 | 1.000 | 1.003 | 1.001 | 1.004 |
| 64 / 65536 | 0.991 | 1.022 | 1.002 | 1.000 |

The 64-peer shared4 improvement is small but consistent in these seven pairs,
1.001..1.009. Split2 at that size ranges 0.992..1.011. Large-payload ratios
are noisy, including 0.796..1.116 in shared2 and 0.585..1.110 in split1, so
their medians do not establish an improvement. At 64 peers, the local/uring
paired rate ratios remain 0.852 shared4 and 0.864 split2. Split2 server CPU
changes from 11.563 to 11.406 us/trip; shared4 remains 13.047 us/trip. Removing
the private record's shared completion handshake alone does not close the gap.

Selection: keep the production baseline while isolating CPU checkpoints.
The private completion simplification is a qualified small optimization
candidate, not evidence that source coloring is needed or that the entire
completion runtime should use the relaxed path. Only records still private
to their submitter support that argument; pending and published records retain
their cross-thread protocol.

## Sixth experiment: keeping both request classes active

`scheduler-sustain` uses the same five forms as the fairness experiment, but
`netload --admit --duration-ms 1000` keeps every connection issuing requests
until one common deadline, then drains the last request on each connection.
The client remains closed-loop with one outstanding request per connection;
this is not an open-loop overload or service-level test. Four and 64 peers,
zero/262144/2097152 compute steps, shared4/split2, two warm-up passes and seven
alternating recorded passes remain the controls.

The round-trip argument is a per-connection storage ceiling in duration mode,
100000 in these cohorts. Hitting it before the deadline fails the sample
instead of silently ending the light class. Before timing, the client reserves
and touches every page of latency and captured-result arrays. Each response
must contain 64 canonical
bits; after exchange timing and socket cleanup, client threads recompute every
request from its connection and request indices and compare every captured
value. No success table is printed until all comparisons pass. This permits
an unknown request count without making an oracle compete during exchange.
Client lifetime CPU includes verification; exchange CPU is separately sampled.

The table adds actual total and per-class counts, common duration, exchange
time and drain time. CPU/trip uses actual completed requests, not the storage
ceiling. Both class rates use the same exchange interval, including drain.
Class spans show whether the requested competition actually lasted. Aggregate
rates alone cannot select a winner when the cheap/heavy request mix differs.
The zero-compute controls measure the cost of this client protocol as well.

The same client C passed a native macOS protocol check through a temporary
kqueue/pthread-barrier compatibility shim (no timing comparison is taken from
that shim): the WF server completed 100 ms admitted exchanges at 0/4096/262144
compute steps with four peers and two workers, with both class spans exceeding
90 ms and their counts summing to the total. An independent socket fixture
confirmed that a wrong canonical value fails the deferred verification, a
non-bit response fails immediately, and reaching the sample capacity early
fails without printing a result. Its fixed-count control verified 100000
requests. Native Linux validation completed in the following cohort.

### Sixth measurement: 4e874daa

[Run 34033979883](https://github.com/mbbill/Whitefoot/actions/runs/34033979883)
and [artifact 9989792739](https://github.com/mbbill/Whitefoot/actions/runs/34033979883/artifacts/9989792739)
contain all 420 verified timed samples. This VM reports Xeon Platinum 8573C,
four logical CPUs on two physical cores with SMT, Linux 6.17 and clang 18.
The gate, native-host and full benchmark workflows also passed at this revision.
Both classes remain active for approximately the full one-second interval;
the drain is reported separately rather than hidden in a fixed requested rate.

At 64 peers with 2097152 recurrence steps per heavy request, class rates use
the same exchange interval, including drain. Values below are medians of
seven samples, not ratios computed by dividing unrelated cohorts:

| placement / form | light requests/s | heavy requests/s | light p99 us | heavy p99 us |
| --- | ---: | ---: | ---: | ---: |
| shared4 / WF | 3852 | 928 | 257914 | 266752 |
| shared4 / inline C | 6394 | 934 | 244140 | 444489 |
| shared4 / q1024 | 218769 | 246 | 3058 | 116116 |
| shared4 / q16384 | 147416 | 479 | 2865 | 65992 |
| shared4 / q65536 | 102112 | 639 | 3054 | 49124 |
| split2 / WF | 1530 | 478 | 33842 | 34239 |
| split2 / inline C | 1447 | 479 | 37804 | 37837 |
| split2 / q1024 | 267366 | 67 | 221 | 329514 |
| split2 / q16384 | 95141 | 296 | 680 | 65112 |
| split2 / q65536 | 34049 | 410 | 2099 | 48204 |

In split2, paired native heavy-rate/WF ratios are 1.003 [0.991, 1.011] inline,
0.141 [0.129, 0.153] q1024, 0.619 [0.593, 0.627] q16384, and 0.858
[0.756, 0.869] q65536. At 262144 steps the three quantum ratios are 0.134,
0.636, and 0.875. The smallest quantum buys rapid light service partly by
spending most CPU on that much larger flow of light responses. Aggregate
requests/s and CPU/request are therefore not comparable measures of useful
heavy work when the served mix changes. WF's finite-burst throughput advantage
does not establish a sustained heavy-throughput advantage over inline C.

In the zero-compute split2 control, WF serves about 245515 total requests/s
versus 320451 inline C, spending 7.29 versus 6.17 server CPU us/request. There
is still a per-request implementation gap when the service mix is the same.
In the long-compute split2 case the median drain is 31.2 ms WF, 35.0 ms inline,
39.0 ms q1024, 17.2 ms q16384, and 17.9 ms q65536.

Selection: compare the WF checkpoint prototype against this class-specific
tradeoff, not the finite-burst aggregate ranking. Another limitation must be
measured: a request-weighted class percentile can hide a slow connection that
contributes very few responses. The next client revision also reports each
class's minimum per-peer completion count and maximum per-peer p99. These
cannot be reconstructed from this cohort's retained aggregate latency fields.

## Seventh experiment: compiler-inserted cooperative loop checkpoints

The compiler experiment `--par --sched-quantum N` runs the same semantic
checker and lowering, then adds a checkpoint to natural-loop jump backedges.
The dominance test is shared with the existing loop cost estimator; a break
to an earlier-numbered exit block is not a backedge. One private i32 counter
per activation belongs to the ordinary target-validated frame plan. An
always-inline LLVM helper decrements it and calls the runtime every N ticks;
LLVM updates the loop phis when inlining its control flow. No source proof is
replaced by that counter, and no source signature acquires a suspension effect.
Default compilation adds neither a counter nor a checkpoint call.

The runtime checkpoint drains completions and pops one already-READY stack.
With none ready it returns. Otherwise the running stack marks itself NOTIFIED
and owes its enqueue to the target stack's existing far-side commit. Only after
its SP and registers have been saved does that commit publish READY and enqueue
it. Enqueuing before the switch would permit two workers to execute one stack.
This path owns its readiness without a completion record. The enumerator
checks that the yielding stack has exclusive running ownership, no registered
waiter or claimed registration, and no previous enqueue owed; all existing
record and stack checks remain active.

New schedule S24 places a child's I/O completion beside a resumed parent and
requires the checkpoint yield transition to be reached. It passed full state
enumeration at (threads, stacks) = (1,2), (1,3), (2,3), (2,4), including 1.15
and 2.81 million states in the two-thread configurations with zero bounded
executions. The maintained completion-test target also passed with S24 added
to every configuration. Thirty-one parallel backend tests and all twelve
compiler invocation tests passed, including native loop-result and unchanged
permission-ledger checks at intervals 1/3/16384 and one/two workers. The real
checkpointed compute server passed four-peer admitted duration exchanges at
0/262144/2097152 steps with one/two workers through the previously checked
macOS client shim; these are correctness checks, not Linux timing evidence.

`scheduler-checkpoint` compares base with intervals 1024/16384/65536, plus the
same inline and three cooperative C references. Network cases use the sixth
experiment's common-duration admitted protocol and both class rates/tails.
Untimed observed 64-peer runs must report actual checkpoint switches as well
as native-ring activity. The pure-compute and warm-file mixed programs return
as controls at two/four/eight workers, with their independent expected bytes.
Network baseline and checkpoint forms share idle policy, global queue, ring,
stack representation, and completion protocol.
The client additionally records minimum completed requests per connection and
the worst individual connection's p99 in each class. Sorting for these metrics
occurs after timing and verification. Local observed runs confirmed 2560/2944
checkpoint calls and 1859/1149 actual switches with one/two workers, after
correcting the observer's omitted summation of the two new counters. Its
initial zero report did not establish that the optimizer removed checkpoints;
the mandatory positive-counter check caught that reporting defect before CI.

This prototype is deliberately not a progress contract. It does not start an
unstarted callee when no stack is ready, cannot repair bounded admission, and
does not cover arbitrary recursive CPU work or an uninterrupted host call.
Iteration counts do not bound elapsed time when a loop body has variable cost.
It tests whether compiler-chosen checkpoints can recover the measured service
opportunity while preserving sequential source, and what they cost pure
computation and oversubscription. No checkpoint interval is selected yet.

### Seventh measurement: 62b626c1

[Run 34035341314](https://github.com/mbbill/Whitefoot/actions/runs/34035341314)
and [artifact 9990387383](https://github.com/mbbill/Whitefoot/actions/runs/34035341314/artifacts/9990387383)
contain all 672 verified timed network samples, the pure-compute/mixed
controls, and positive observed checkpoint switches for every candidate.
This host reports AMD EPYC 9V74, four logical CPUs on two physical cores with
SMT, Linux 6.17 and clang 18. The full gate and native-host workflows passed.
The separate Windows benchmark workflow failed its unchanged stability test:
`mixed-iocp` remained unstable after both 15-pair cohorts. Its artifact
9990046681 retains both attempts; no wrong-result failure was reported.
This is not an all-workflows-green revision.

At 64 peers and 2097152 compute steps, medians are:

| placement / form | light requests/s | heavy requests/s | light p99 us | worst light-peer p99 us | minimum light-peer count |
| --- | ---: | ---: | ---: | ---: | ---: |
| shared4 / WF | 3905 | 1116 | 258431 | 261869 | 11 |
| shared4 / WF cq1024 | 161097 | 47 | 576 | 618 | 3297 |
| shared4 / WF cq16384 | 118640 | 346 | 939 | 1020 | 2376 |
| shared4 / WF cq65536 | 84121 | 616 | 2691 | 2925 | 1358 |
| shared4 / C q1024 | 168416 | 282 | 4005 | 5771 | 1879 |
| shared4 / C q16384 | 113117 | 520 | 2999 | 3700 | 1292 |
| shared4 / C q65536 | 82576 | 679 | 3291 | 4386 | 676 |
| split2 / WF | 1832 | 573 | 27812 | 28712 | 39 |
| split2 / WF cq1024 | 182735 | 47 | 321 | 325 | 3884 |
| split2 / WF cq16384 | 98562 | 267 | 528 | 542 | 2093 |
| split2 / WF cq65536 | 43902 | 443 | 1153 | 1172 | 922 |
| split2 / C q1024 | 215928 | 54 | 275 | 285 | 3978 |
| split2 / C q16384 | 98758 | 295 | 637 | 661 | 1796 |
| split2 / C q65536 | 40148 | 459 | 1574 | 1623 | 767 |

In split2, paired WF/C heavy-rate ratios at matching intervals are 0.865
[0.594, 0.991], 0.903 [0.851, 0.911], and 0.968 [0.956, 0.978]. In shared4
they are 0.168 [0.158, 0.195], 0.664 [0.614, 0.720], and 0.920
[0.865, 1.262]. The shortest interval is particularly expensive for WF when
the client shares CPUs. The different light rates still prevent attributing
all heavy-rate differences to checkpoint overhead alone. Worst-peer tails
and minimum counts confirm that the WF light-class gains cover every peer,
not just the aggregate's busiest connections.

Pure-compute median milliseconds for base/cq1024/cq16384/cq65536 are
2108.31/2144.56/2124.96/2125.50 at two workers,
1162.02/1533.98/1450.12/1453.03 at four, and
1191.41/1573.42/1489.69/1490.41 at eight. Even the larger intervals lose
about 25% at four/eight workers, versus under 1% at two. Warm-file mixed
medians at four workers are 157.11/172.70/163.34/162.15 ms and at eight
172.01/205.12/179.73/174.98 ms. The current instrumentation cannot be
selected as a universal compute policy.

Selection: sequential source plus compiler-inserted checkpoints can recover
the missing service opportunities without a writer-visible suspension
annotation. This is an existence result for the tested loop shape, not the
desired performance frontier or a complete progress contract. Retain default
compilation without checkpoints while separating code-generation cost,
scheduler switching cost, and the fixed-arrival tradeoff below. Nothing in
this measurement establishes that source coloring would eliminate those costs.

## Eighth experiment: initialize only configured worker lanes

The scheduler reserves one fixed core containing 64 worker lanes, but the
baseline clears all of them even when only one or four workers are configured.
On the local arm64 ABI, the core is 20478032 bytes and each lane is 319520
bytes. This is an implementation startup cost, not a requirement of source
ownership or of stackful suspension.

`WF_SCHED_INIT_USED_LANES=1` preserves the structure layout and all capacity
limits. It clears the prefix, exactly `thread_count` lanes, and the trailing
status/idle fields. Initialization of every configured lane, slot, thread and
stack remains unchanged. Unconfigured lanes are unreachable: worker lookups
are bounded by the configured count and only initialized slots can be handed
out. The default remains zero during comparison.

A local one-shot initialization probe checked live metadata from both zeroed
and deliberately poisoned storage at 1/4/8/64 workers and 1100 stacks. Median
peak RSS from three measured launches after one warm-up, in KiB, was
38880/19264, 38880/20192, 38880/21440 and 38880/38896 for baseline/candidate.
The lack of a saving at 64 workers is expected. The maintained completion-test
target passed with the candidate enabled, including all S24 enumeration and
the threaded smoke test, which now begins with poisoned core storage.

The real WF compute-protocol server, built from the same emitted module and
runtime sources with only this define changed, then verified four admitted
peers for 100 ms at zero compute through the macOS client compatibility shim.
With 1100 stacks in both forms, median peak RSS over three measured launches
after one warm-up was 39152/19520 KiB at one worker and 39152/20432 KiB at
four workers. These establish the local memory effect, not a Linux throughput
result. macOS's 16 KiB pages also make touching 1100 stack headers expensive;
the remaining footprint is not all used-lane storage.

`scheduler-footprint` compares base/lanes with native uring/epoll controls in
the same four CPU placements and five echo cases as the inline experiment.
All WF samples retain 1100 stacks, the baseline idle and completion protocols,
and no compiler checkpoints. It runs the full candidate completion checks
before timing, then two warm-ups and seven alternating measured passes. The
define can be removed once this implementation choice is selected or rejected;
the existing investigation and runner own its evidence.

### Eighth measurement: 87585b8e

[Run 34036774493](https://github.com/mbbill/Whitefoot/actions/runs/34036774493)
and [artifact 9990555530](https://github.com/mbbill/Whitefoot/actions/runs/34036774493/artifacts/9990555530)
contain all 560 verified timed samples and the full candidate completion
checks. The host reports Xeon 6973P-C, four logical CPUs on two physical
cores with SMT, Linux 6.17 and clang 18. The gate and native-host workflows
passed. The separate Windows benchmark failed its unchanged `io-warm`
stability qualification after two cohorts, artifact 9990475456; this revision
does not have every workflow green.

At 64 peers and 64 bytes, median peak RSS in KiB changes from 35712 to 18672
in shared4, 36220 to 18460 in shared2, 35760 to 18472 in split2, and 35884
to 17972 in split1. At 1024 peers the corresponding baseline/candidate pairs
are 79204/62512, 79360/61984, 79732/61900 and 79228/61584. The remaining
per-connection storage cost is substantial even after removing the unused
worker initialization.

Paired candidate/base rate medians for 1/4/64/1024 peers with 64-byte payloads,
then 64 peers with 64 KiB payloads, are:

| placement | 1 | 4 | 64 | 1024 | 64 KiB |
| --- | ---: | ---: | ---: | ---: | ---: |
| shared4 | 0.998 | 0.922 | 1.024 | 1.002 | 0.996 |
| shared2 | 1.028 | 1.019 | 0.996 | 0.976 | 1.002 |
| split2 | 0.992 | 0.998 | 1.001 | 0.987 | 0.995 |
| split1 | 1.014 | 1.010 | 1.005 | 1.001 | 0.970 |

Shared4 at four peers spans 0.664..1.075; split2 at 64 peers spans
0.989..1.006, and split2 at 1024 spans 0.949..1.029. The memory saving is
clear, while these timings do not establish universal throughput equivalence
or improvement. Keep both initialization forms available and hold the
baseline initialization fixed during the independent loop-codegen experiment.

## Ninth experiment: fix the light request arrival schedule

`scheduler-paced` keeps 64 admitted peers and a one-second arrival interval,
with every fourth peer doing heavy computation. Each light peer receives
20/100/500 scheduled requests per second, giving 960/4800/24000 total light
arrivals independent of server speed. Peers are evenly staggered within one
period. Heavy peers continue the saturated closed-loop protocol. Both
262144 and 2097152 compute costs are measured, plus a zero-compute control at
100 light arrivals per second per peer, in shared4 and split2. The forms are
the baseline, three WF checkpoint intervals and four native C references.

`netload --light-per-second RATE` uses absolute scheduled times derived from
the common exchange origin. Each connection still permits one outstanding
wire request: later arrivals queue logically in the client. Every arrival
scheduled before the common deadline must eventually be sent and verified,
including backlog after that deadline. Capacity must cover all planned light
arrivals before the run starts. This is a fixed offered workload with client
queueing, not a claim to have tested arbitrary pipelined wire overload.

Light latency begins at its scheduled arrival, including client dispatch
delay and queueing behind previous requests. The result also reports dispatch
delay and wire-service p99 separately, all planned light requests, and each
class's completions before the deadline. The latter rates use the requested
interval, whereas the retained exchange rates include drain. Pending light
requests at the deadline and drain time expose unsatisfied demand. A slow
server cannot improve its percentile merely by suppressing planned requests.

The event wait uses the nanosecond-resolution timeout of
[epoll_pwait2](https://www.man7.org/linux/man-pages/man2/epoll_wait.2.html),
available on the Linux CI kernel and its glibc. This avoids an intentional
millisecond rounding floor; it does not promise nanosecond wake-up accuracy.
The client scans its assigned waiting connections for the next arrival and
records its own exchange CPU. No per-request allocation or checksum oracle
competes during the timed exchange.

Native macOS correctness checks through the temporary API shim covered
1/2 workers, 0/262144/2097152 compute steps, and rates 1/100/1000/100000.
They included peers with no arrival in a short interval and a deliberately
overloaded run that drained all 30000 planned light requests. An independent
socket fixture retained all 300 requests both with immediate responses and
with 5 ms response delays. In the delayed run only 39 light requests completed
before the 100 ms deadline; dispatch p99 exceeded 638 ms and total p99 exceeded
647 ms, so backlog was not omitted. A wrong canonical result failed deferred
verification without printing a result table. A known-arithmetic fixture
checked the 44-column raw table and 26-column summary, including separate
rate cohorts, actual-count CPU divisors, paired ratios and deadline backlog.

### Ninth measurement: 609e4437

[Run 34037031772](https://github.com/mbbill/Whitefoot/actions/runs/34037031772)
and [artifact 9991023131](https://github.com/mbbill/Whitefoot/actions/runs/34037031772/artifacts/9991023131)
contain all 784 verified timed samples. Every row retains exactly
`48 * light_per_second` completed and verified light arrivals. This VM reports
AMD EPYC 9V74, four logical CPUs on two physical cores with SMT, Linux 6.17
and clang 18. The gate and native-host workflows passed at this revision.
The separate full benchmark ran on parent 93d280f5 (the only intervening
change enabled this temporary CI branch) and its Windows qualification failed;
it is not represented as a successful qualification at 609e4437.

For 2097152 heavy compute steps on split2, heavy rates below count completions
inside the fixed one-second interval. Light tails include dispatch/backlog,
and all outstanding requests are subsequently drained and verified:

| light arrivals/s | form | heavy completions/s | light p99 us | worst light-peer p99 us | light pending at deadline |
| ---: | --- | ---: | ---: | ---: | ---: |
| 960 | WF base | 572 | 26988 | 28074 | 4 |
| 960 | WF cq1024 | 528 | 624 | 991 | 0 |
| 960 | WF cq16384 | 560 | 1272 | 1909 | 0 |
| 960 | C q1024 | 559 | 135 | 185 | 0 |
| 960 | C q16384 | 560 | 540 | 579 | 0 |
| 4800 | WF base | 571 | 635639 | 635930 | 3007 |
| 4800 | WF cq1024 | 496 | 733 | 989 | 0 |
| 4800 | WF cq16384 | 544 | 1039 | 1234 | 1 |
| 4800 | C q1024 | 536 | 130 | 170 | 0 |
| 4800 | C q16384 | 550 | 566 | 582 | 2 |
| 24000 | WF base | 572 | 941338 | 941338 | 22176 |
| 24000 | WF cq1024 | 400 | 293 | 356 | 1 |
| 24000 | WF cq16384 | 464 | 576 | 731 | 7 |
| 24000 | C q1024 | 470 | 113 | 156 | 1 |
| 24000 | C q16384 | 492 | 548 | 579 | 8 |

At 960/4800/24000 light arrivals per second, paired WF/C heavy-rate ratios at
interval 16384 are 1.000 [0.989, 1.002], 0.989 [0.978, 1.000], and 0.951
[0.924, 0.984]. Matching interval 1024 ratios are 0.945, 0.925, and 0.851.
At 262144 compute steps the 16384 ratios are 0.989, 0.982, and 0.949.
On shared4 with long compute, the 16384 ratios are 0.984, 0.981, and 0.945;
light p99 is approximately 2.6..3.3 ms and dispatch delay contributes
approximately 1.8..2.0 ms. That client/server CPU competition is visible in
the protocol instead of being mislabeled entirely as server service latency.

Selection: under bounded identical light demand, cooperative service can
retain most heavy capacity. The severe heavy-rate loss in the unpaced cohort
was partly the changed offered workload. There is still a real implementation
gap: WF can approach the native heavy rate while having materially worse light
tails, especially at low light rates and small quanta. Near-equal heavy rates
alone do not meet the performance goal. Keep the fixed-arrival protocol as a
control for further compiler and ready-queue changes; do not infer a universal
best interval from the non-monotone light tails in these cells.

## Tenth experiment: keep checkpoint bookkeeping out of the inner loop

The seventh cohort's larger-interval pure-compute loss did not require a
runtime checkpoint call. A local observed build of the same layout program
at four workers and interval 16384 reported zero calls and zero switches,
matching the source's 8192/4096-iteration leaf loops. Their activation-local
counters never expire. Optimized LLVM nevertheless retains the decrement
and conditional branch in every iteration. Increasing the interval alone
does not remove that code-generation cost.

`--par --sched-chunks N` performs a post-checking IR transformation of
unsigned unit-stride natural loops. The header must consist of its bound
comparison, that comparison's exhaustion edge must leave the natural loop,
the bound must be invariant through every forwarded block parameter, and
the latch must increment the tested index by one. No function or source
name selects the transformation. A loop driven by a completion pipeline is
left to that driver and the existing counter fallback.
Equality/inequality termination is also eligible when the initial index is
the unsigned constant zero, which proves it cannot start above its bound.
A dynamic-start equality loop may intentionally wrap; it keeps the fallback.

The transformed loop compares its index against
`min(upper, saturating_add(start, N))`. On exhaustion, it either takes the
original exit or checkpoints and starts the next chunk. The inner body,
its source operations, early exits, drops and carried values are retained.
An empty or reversed range stays empty; saturation avoids wrapping a chunk
limit near the u64 maximum. The transformation adds ordinary IR blocks and
SSA values, not an acceptance rule or a writer-visible effect. Other loops
retain the existing counter prototype and all progress-contract limitations.

The native boundary test compares both counted and ordinary natural loops
with independent wrapping-fold results at intervals 1/3/16384/u32::MAX and
one/two workers. It includes empty/reversed ranges, near-maximum indices,
early breaks and nested loops. A changing bound and reversed comparison
polarity must decline chunking while preserving their native results, as must
a dynamic-start equality loop that actually wraps through the u64 maximum.
The permission ledger is unchanged. All 32 parallel backend tests passed.
The extended equality-termination case, all 15 loop-splitting backend tests,
and all 12 command-line tests also passed. Native four-peer network exchanges
with the new recurrence lowering verified every byte at one/two workers and
reported 2286/2540 checkpoint calls with 1612/1211 actual switches. The calls
equal exactly 127 inter-chunk checks per completed heavy request. A positive
runtime count alone had initially exercised the counter fallback; the runner
now also requires the measured recurrence's emitted body to use chunk checks.

On the M1, a local one-batch four-worker check with one warm-up and three
alternating measured passes gave median wall times of 488.75 ms base,
496.45 ms counter16384 and 497.42 ms chunks16384, with identical independent
expected bytes. This host does not reproduce the Linux four-worker counter
loss and does not establish that chunking recovers it. The Linux experiment
therefore compares base, counter16384 and chunks1024/16384/65536 on one host,
plus the existing four native C references. It retains the three 64-peer
common-duration network cases, and pure-compute/warm-file controls at two,
four and eight workers. Untimed observations must confirm zero runtime calls
in both 16384 pure-compute forms and positive switches in the mixed network
forms. Default compilation and source progress semantics remain unchanged.

Measured revision `32220011`, Linux run
[`34039331365`](https://github.com/mbbill/Whitefoot/actions/runs/34039331365),
artifact `9991458745`, completed with 378 timed network samples. This runner
was an AMD EPYC 7763 with four logical CPUs on two physical SMT cores,
Linux 6.17 and clang 18. Both observed 16384 compute forms reported exactly
zero checkpoint calls and switches. The gate, host qualification, scheduler
experiment and all platform I/O benchmarks passed at this revision.

Pure-compute median wall times in milliseconds:

| Workers | Base | Counter 16384 | Chunks 1024 | Chunks 16384 | Chunks 65536 |
| --- | ---: | ---: | ---: | ---: | ---: |
| 2 | 2415.58 | 2448.58 | 2443.73 | 2416.06 | 2417.84 |
| 4 | 1347.32 | 1886.52 | 1519.90 | 1442.73 | 1442.81 |
| 8 | 1383.70 | 1937.28 | 1558.66 | 1483.83 | 1481.77 |

The counter costs about 40% at four/eight workers on this host. Chunking
reduces that excess to about 7%, recovering most, but not all, of the lost
compute performance. At two workers chunks16384 matches base within 0.1%.
The warm-file mixed control at four workers is 188.23/190.31/206.01/189.54/
192.13 ms in the same column order; at eight it is 193.00/204.45/230.68/
202.29/201.27 ms. A remaining pure-compute cost without any runtime calls
still points to generated code or its effects on execution, not switch cost.

With 64 peers and 2097152 compute steps, split2 chunks16384 sustains 207.6
heavy requests/s and light p99 780 us; counter16384 gives 206.6 and 718 us.
Their paired heavy-rate ratio is 1.006 [0.952, 1.059]. Shared4 gives 270.6
heavy requests/s for both, with light p99 1162/1125 us and paired heavy ratio
1.000 [0.965, 1.012]. There is no measured network gain from this compiler
change. Against matching native C quanta, the chunks1024/16384/65536 paired
heavy-rate ratios are 0.797/0.828/0.968 on split2 and 0.135/0.629/0.883 on
shared4. As before, these unpaced cells serve different amounts of light
work; they do not compare capacity at equal demand. The zero-compute split2
base sustains 155291 total requests/s versus inline C's approximately 192009.

Selection: retain chunking as an experimental lowering with materially less
compute damage than per-iteration counters. It has not reached the original
compute performance and has not removed the network implementation gap.
Use it in the next fixed-arrival queue experiment while keeping its interval
and emitted module identical within each runtime-policy comparison.

## Eleventh experiment: bounded preference for completion-ready stacks (retired)

The paced cohort leaves a latency gap even when WF retains almost the native
heavy rate. A completion-ready stack can wait in the same FIFO as heavy
stacks that voluntarily yielded at a checkpoint. Test this scheduling
opportunity independently of the compiler's interval and generated code.

At revision `6380a17a`, `WF_SCHED_COMPLETION_READY_BURST=B` defaulted to zero, the original single
FIFO. Positive B separates record-completion resumptions and voluntary
checkpoint resumptions into two FIFOs under the existing mutex. This is a
completion class, including compute joins, not an I/O-only classification.
When both classes remain ready, at most B completion pops precede a yielded
stack's turn. A yielded pop resets the budget. Each class preserves FIFO
order, and the union's empty-to-nonempty transition performs the wake.

The running owner writes its next readiness class before publishing its
park/yield phase. The existing phase handshake and queue mutex publish this
field; READY is still offered only after the far-side stack-switch commit.
The field occupies existing stack-header padding on the supported ABIs.
Enumeration now checks disjoint lists, phases, class membership, both tails,
the budget range and the union's sleeping/wake invariant. Existing waiter,
stack-ownership, switch and completion checks remain in force. Both B=1 and
B=8 passed the complete maintained completion suite on the M1, including all
four enumeration configurations with no bounded executions. Their S24
two-thread/four-stack searches visited 3012673 and 3421427 states respectively.
Native four/eight-peer fixed-arrival exchanges at one/two workers verified
every response for B=0/1/8. The eight-peer one-worker observations reported
589 preferred and 108 forced selections for B=1, and 593 preferred and one
forced selection for B=8. Thus the local check exercised the actual policy,
not merely its configuration. These kqueue client-shim runs establish native
correctness observations, not a Linux performance comparison.

`scheduler-priority` at that revision compared chunks1024/16384, each with B=0/1/8, and native
C at both matching quanta. The compiler-emitted modules must be byte-identical
between queue policies at each interval. Keep the zero-compute paced control,
262144 steps at 4800 total light arrivals/s, and 2097152 steps at 960/4800/
24000 arrivals/s. All 64 peers are admitted before the one-second interval;
every planned light request is retained and independently verified after
timing. Shared4 and split2 use two warm-ups and seven alternating passes.
The summary's paired reference is the single-FIFO chunk form at the matching
interval, including for each native C quantum. Pure-compute and warm-file
controls remain at two/four/eight workers. Untimed observations report both
readiness classes and actual preferred/forced selections. Neither a positive
checkpoint count nor a policy flag alone establishes that prioritization
changed a particular execution.

Measured Linux run
[`34040650208`](https://github.com/mbbill/Whitefoot/actions/runs/34040650208),
artifact `9992033955`, completed with 560 timed samples on a Xeon 6973P-C,
four logical CPUs on two physical SMT cores, Linux 6.17 and clang 18. Every
planned light arrival was retained and verified. The gate and host checks
passed. The separate Windows benchmark failed its unchanged compute-stability
criterion: attempt one had relative p90-p10 spread 0.1553, attempt two 0.4218,
against a 0.10 limit; parallel wall times ranged 1321..1723 and 1322..2041 ms.
It did not produce a qualified Windows table. That is not a wrong-byte report
or evidence establishing the cause of the instability.

Long-compute split2 results at interval 16384:

| Total light arrivals/s | Form | Heavy completed by deadline/s | Light p99 us | Worst peer p99 us |
| --- | --- | ---: | ---: | ---: |
| 960 | Single FIFO | 636 | 1271 | 3827 |
| 960 | B=1 | 639 | 1101 | 2417 |
| 960 | B=8 | 624 | 1077 | 2994 |
| 960 | Native C | 642 | 498 | 538 |
| 4800 | Single FIFO | 620 | 1181 | 2081 |
| 4800 | B=1 | 624 | 1009 | 1119 |
| 4800 | B=8 | 624 | 1049 | 1532 |
| 4800 | Native C | 638 | 497 | 510 |
| 24000 | Single FIFO | 560 | 487 | 533 |
| 24000 | B=1 | 560 | 1028 | 1228 |
| 24000 | B=8 | 560 | 993 | 1124 |
| 24000 | Native C | 598 | 482 | 514 |

At the highest arrival rate, the paired B=1/B=8 light-p99 ratios to single
FIFO are 2.218 [1.440, 4.277] and 2.093 [1.431, 3.109]. This regression occurs
in every paired pass. Paired heavy-rate medians are 1.000/1.007. At the lowest
rate, B=8 improves light p99 in every paired pass, ratio 0.859 [0.629, 0.887],
so the policy's effect depends on the offered load. At interval 1024 and the
highest rate, single FIFO/B=1/B=8 light p99 is 195/176/495 us with identical
median heavy rates of 416/s, versus native C's 101 us and 578/s. Shared4
retains substantial client dispatch delay and shows no consistent priority
gain. Untimed split2 64-peer observations at 16384 recorded 4089 preferred /
3548 forced selections for B=1, and 4093 / 388 for B=8; the policies were
actually exercised.

Pure-compute times at four workers and interval 16384 are 1339.59/1346.52/
1348.97 ms for B=0/1/8, and at eight workers 1385.49/1393.70/1388.94 ms.
The corresponding warm-file medians are 129.00/132.96/131.34 and 156.17/
160.56/159.39 ms, with wide ranges in several cells. No general control
improvement compensates for the high-arrival network regression.

Selection: retire this fixed completion-class preference and its experiment
target. Restore the single FIFO, retaining cooperative checkpoints and all
original protocol tests. Remove the extra readiness tag, FIFO, budget and
their class-specific enumeration assertions because those states no longer
exist; do not remove any original ownership, waiter, completion or wake
assertion. Revision `6380a17a` and this result retain the reproducible trial.
This rejects the tested policy, not every possible fairness policy or I/O
priority design. Move to the measured code-generation issue and shared
diagnostic writes rather than accumulating an unused runtime policy.

## Twelfth experiment: give the chunk loop a separate header

The remaining no-call compute cost has a concrete code-generation lead.
Local clang compilation of the emitted modules to x86 assembly unrolls the
original metric-table loop twice. The first chunk representation instead
forms its inner cycle around the checkpoint path and leaves the computation
at one iteration per branch. Its source header receives both the ordinary
latch and chunk-resume backedges. This is evidence about generated code, not
yet a causal Linux timing measurement.

The revised transformation gives the outer chunk loop its own header and
carried parameters. That header computes the chunk limit and enters the
original inner loop; its original latch remains the only inner backedge.
Exhaustion either exits with the original carried values or checkpoints and
returns to the outer header. The source body is not duplicated. Empty and
reversed ranges, saturating limits near u64::MAX, early exits and the counter
fallback retain their previous behavior. This agrees with LLVM's preference
for a preheader and single latch in its
[canonical loop forms](https://llvm.org/docs/LoopTerminology.html#loop-simplify-form).
The local x86 assembly now restores two-iteration unrolling in the computation
and places the checkpoint on the outer cycle.

All 32 parallel backend tests and 15 loop-splitting tests passed, as did
all-target clippy. The boundary test additionally checks that every inner
header has exactly one natural backedge.
The extended boundary case also passed with a uniquely borrowed buffer carried
through chunks: two passes mutate the same storage, the second exits early,
and their independent expected sums check that writes happen exactly once.
Its early-stop branch keeps the ordinary reduction splitter from replacing
the loop before the checkpoint pass, and the test requires actual chunking.
Native eight-peer fixed-arrival exchanges at one/two workers verified every
response and reported 3556/7112
checkpoint calls, exactly 127 per completed heavy request. The observed
four-worker metric-table program still reports zero calls and switches.
A one-warm-up/three-alternating-pass M1 check gives 490.65 ms original,
490.96 ms former chunks and 492.95 ms canonical chunks. This host again does
not reproduce the Linux regression; assembly shape alone is not a speedup.

`scheduler-canonical` rebuilds the former compiler at `6380a17a` from Git,
checks that its uninstrumented module matches the current compiler byte for
byte, and compares former/canonical chunk intervals 1024/16384 plus base on
one Linux host. All forms link the current C runtime with B=0, the original
idle policy, full lane initialization and the shared completion path. Native
C at the two matching quanta remains in the network cohort. Keep 64 peers,
the zero-compute fixed-arrival control, and long computation at 4800/24000
total light arrivals/s, with shared4/split2 placement. Pure-compute and
warm-file controls remain at two/four/eight workers. Both 16384 compute forms
must report zero calls; the artifact includes their generated assembly and
the baseline's assembly. There are two warm-ups and seven alternating passes.
The prior compiler build is temporary measurement machinery, removed when
this lowering question is settled; the active implementation is superseded
in place rather than keeping two compiler passes.

Measured revision `f6b80173`, Linux run
[`34041692801`](https://github.com/mbbill/Whitefoot/actions/runs/34041692801),
artifact `9992147472`, completed with 294 timed samples on a Xeon Platinum
8370C, four logical CPUs on two physical SMT cores, Linux 6.17 and clang 18.
Every planned light arrival was retained and verified. Host qualification
and every platform I/O benchmark passed. Thirteen gate jobs passed; the
macOS scheduler job hit the workflow's existing eight-minute limit during
the progress-policy two-thread/four-stack enumeration. That is an incomplete
gate, not a source rejection or a reported incorrect execution. No test was
disabled or narrowed; the next revision reruns the full gate.

The measured clang 18 assembly confirms the diagnosis: former chunks put
the checkpoint call in the inner cycle, while canonical chunks restore the
two-element metric-table loop with the call on the outer cycle. Both observed
16384 compute forms execute zero checkpoint calls and switches. Median wall
times in milliseconds:

| Workers | Base | Former 1024 | Former 16384 | Canonical 1024 | Canonical 16384 |
| --- | ---: | ---: | ---: | ---: | ---: |
| 2 | 2957.18 | 3268.20 | 3248.81 | 2985.18 | 2955.18 |
| 4 | 1609.99 | 1876.50 | 1764.58 | 1724.41 | 1616.22 |
| 8 | 1678.21 | 1953.73 | 1839.89 | 1793.98 | 1681.94 |

Canonical16384 is within 0.4% of base at each worker count, while the former
16384 form costs about 10% on this host. The smaller interval still has real
runtime opportunities in these leaf loops and remains slower at four/eight
workers. Warm-file medians at four workers are 160.80/185.71/165.80/186.57/
167.10 ms in table order, and at eight 180.57/235.96/189.53/234.08/190.40 ms.
There is no corresponding warm-file improvement from the topology change.

Long-compute split2 results:

| Light arrivals/s | Form | Heavy by deadline/s | Light p99 us | Worst peer p99 us |
| --- | --- | ---: | ---: | ---: |
| 4800 | Former 1024 | 400 | 1336 | 2163 |
| 4800 | Canonical 1024 | 416 | 245 | 388 |
| 4800 | Native C 1024 | 496 | 125 | 137 |
| 4800 | Former 16384 | 480 | 1312 | 3042 |
| 4800 | Canonical 16384 | 496 | 1109 | 1443 |
| 4800 | Native C 16384 | 504 | 588 | 615 |
| 24000 | Former 1024 | 336 | 430 | 718 |
| 24000 | Canonical 1024 | 352 | 631 | 961 |
| 24000 | Native C 1024 | 445 | 117 | 130 |
| 24000 | Former 16384 | 416 | 880 | 1605 |
| 24000 | Canonical 16384 | 432 | 1448 | 2002 |
| 24000 | Native C 16384 | 464 | 577 | 617 |

Paired canonical/former heavy-rate ratios on split2 are 1.040/1.033 at 4800
arrivals/s for intervals 1024/16384, and 1.048/1.022 at 24000. Against native
C they are 0.839/0.982 and 0.786/0.929 respectively. Shared4 canonical/former
ratios are 1.044/1.037 and 1.035/1.039. Improved compute capacity does not
uniformly improve tails; do not rank the change by heavy throughput alone.

Selection: keep the canonical topology in the experimental chunk lowering.
It removes the measured no-call compute penalty without changing source
signatures or permission judgments, and recovers some mixed-load capacity.
It does not select a universal quantum, close the network tail/CPU gap, or
establish backed admission or general progress. The former topology remains
only in the explicitly versioned comparison build while this result is used
to separate compiler cost from subsequent runtime changes.

## Thirteenth experiment: retain counters without one shared write location

Every bridge publication updates one global diagnostic counter; inline
operations update a second. These relaxed atomic RMWs do not select a
protocol action, but their shared cache lines can still be contested. Test
their placement before attributing the remaining I/O cost entirely to the
completion representation or language design.

`WF_COMPLETION_COUNTER_STRIPES` defaults to one, retaining the original two
atomic counters. A positive value up to 64 provides that many static pairs,
each 128 bytes apart and aligned accordingly. The M1 reports a 128-byte cache
line; this spacing also separates the measured x86 host's 64-byte lines.
Each host thread obtains a stripe on its first update using a relaxed ticket,
then keeps the index in TLS. Collisions and ticket wrap remain correct because
updates remain atomic RMWs. Counter observations sum relaxed loads, exactly
recovering the completed counts once writers quiesce. Publication order,
records, completion states, stack switching and public counter functions are
unchanged. No counter is disabled to make a candidate faster.

The final 16-stripe layout passed the complete maintained completion suite
on the M1, including its full enumeration and all count assertions. The
four-lane default-route probe also passed with two stripes, forcing shared
slots while checking all 16000 lane operations and its additional route
probes. The default remains one pending measurement.

`scheduler-counters` compares one and 16 stripes using identical emitted
modules and otherwise fixed runtime policies. The Linux cohort keeps all
four CPU placements, 1/4/64/1024 peers with 64-byte messages, and 64 peers
with 64-KiB messages, plus native io_uring and epoll references. Pure-compute
and warm-file controls remain at two/four/eight workers. Two warm-ups and
seven alternating passes give 560 timed network samples. Untimed observations
must show the selected stripe count, inline completions and actual native-ring
traffic. The complete 16-stripe suite and two-stripe collision probe run on
the measuring Linux host before sampling. This is a counter-contention
experiment, not evidence yet that the language or completion ABI must change.

### Counter results and retirement

Revision `9479d624aa8994db2872e95f6f1d0a5df62bb30f`, Linux run
[34043256139](https://github.com/mbbill/Whitefoot/actions/runs/34043256139),
artifact `9992567181` (`io-scheduler-counters`), completed on an EPYC 7763 VM
with four logical CPUs, two SMT cores and clang 18.1.3. All 560 timed samples,
the complete 16-stripe completion suite, the two-stripe collision probe and
native-ring observations passed. The repository gate and host matrix passed;
the Windows qualification job in run `34043256138` rejected two complete
compute cohorts as unstable, so that workflow provides no qualified Windows
performance table. Its Linux and macOS jobs passed.

| Placement | 64 peers: striped/base paired rate | 1024 peers: striped/base paired rate |
| --- | ---: | ---: |
| Shared four CPUs | 1.0030 | 0.9988 |
| Shared two CPUs | 0.9984 | 1.0023 |
| Split two server/two client CPUs | 1.0008 | 1.0004 |
| Split one server/one client CPU | 0.9995 | 0.9972 |

These are medians of seven within-pass ratios for 64-byte messages. Every
cell has paired samples on both sides of one. At 64 peers on split2, server
CPU is 11.250/11.172 us per trip for base/striped, while native io_uring is
9.844 us; throughput remains about 163k versus 192k trips/s. At 1024 peers,
base/striped/native rates are about 158k/156k/179k. Distributing the diagnostics
does not close this gap. The 64-KiB split2 paired rate ratio is 1.026, within
wide sample variation; its p99 remains about 41.5 ms in both WF forms. The
four-peer shared4 ratio is 0.958, not a general benefit either.

Pure-compute base/striped medians at two/four/eight workers are
2412.24/2415.77, 1337.60/1342.75 and 1375.84/1382.73 ms. Warm-file medians
are 177.25/178.16, 183.79/183.58 and 190.96/197.95 ms. Neither control
selects the added machinery.

Retire the stripe storage, ticket, TLS index, summation helpers, diagnostic
field and temporary comparison mode. Restore the original two atomic
counters and every increment and count assertion. The collision invocation
existed only to validate the retired multi-stripe representation; the full
original route probe and completion tests remain maintained. Reproduce this
experiment by checking out its measured revision. This result rejects a
specific diagnostic-contention explanation on the measured workload, not
all possible shared-cache effects.

## Fourteenth experiment: spread stack-top offsets

Every POSIX pool slot has the same page-aligned stride. Consequently every
stack header and initial stack pointer has the same page offset. Test whether
this layout contributes to the cost of cycling through many small connection
frames. No queue or completion protocol change is needed for this experiment.

`WF_SCHED_STACK_SPREAD_BYTES=4096` adds 4096 bytes to each slot's requested
reservation before the platform rounds it to pages. Stack index modulo 32
selects an offset in 128-byte steps from zero through 3968. Both the header
and initial stack pointer move down by that amount. The stack's upper bound
ends immediately after its header, so the existing enumerator snapshots and
digests still cover all live stack bytes and use the actual header. The
lower guard stays at the slot's original beginning. Additional reservation
space ensures that no stack loses usable depth; page rounding can add more
than 4096 bytes on the M1. The unoccupied suffix is not live stack storage.

The default is zero. A nonzero setting is rejected on Windows because
Windows fibers own separate stacks; moving their reservation metadata would
not test the intended execution-stack layout. This is a POSIX implementation
experiment with unchanged source signatures, frame lifetimes, stack phases,
mutex, ring, idle policy, counter implementation and compiler lowering. A
positive result alone would not distinguish cache conflicts from other
address-layout effects.

`scheduler-stacks` supersedes the retired counter comparison in the maintained
runner. It uses the same four CPU placements, five connection/payload cases,
native io_uring and epoll references, pure-compute and warm-file controls,
two warm-ups and seven alternating passes (560 timed network samples).
Observed links verify the selected stack layout and real ring traffic. The
complete completion suite runs with the candidate before Linux timing; the
host smoke additionally checks usable depth, header/frame bounds and distinct
successive stack offsets. The original interleaving invariants remain intact.
A native M1 eight-peer TCP check has already verified every reply in both
layouts at one and two workers; timing conclusions await Linux CI.
The complete candidate completion suite subsequently passed on the M1,
including every maintained schedule configuration with zero bounded states,
all helper configurations and the exact completion count checks. The default
layout also passed the amended host smoke. These are correctness results;
the M1 compatibility load client is not used for Linux performance claims.

The first Linux launch (`34044699217`, artifact `9992763699`) passed the
complete candidate suite and the io_uring, epoll and baseline correctness
runs, then stopped before starting the candidate. The benchmark dispatcher
still listed the retired `striped` form and rejected `spread` with exit 2.
Correct that form name and rerun; no timed cohort was produced by this launch.

### Stack-layout results and retirement

The corrected revision `bc748c302de39d8279019c075a45b78190bf5544` completed
Linux run [34045212729](https://github.com/mbbill/Whitefoot/actions/runs/34045212729),
artifact `9993142659` (`io-scheduler-stacks`), on an EPYC 9V74 VM with four
logical CPUs and clang 18.1.3. All 560 timed network samples and the complete
candidate suite passed. The repository gate and native host matrix passed.
The separate Windows qualification again rejected two unstable compute
cohorts; its Linux/macOS jobs passed, but there is no qualified Windows
performance table for this revision.

| Placement | 64 peers: spread/base paired rate | 1024 peers: spread/base paired rate |
| --- | ---: | ---: |
| Shared four CPUs | 1.0068 | 0.9993 |
| Shared two CPUs | 0.9987 | 1.0086 |
| Split two server/two client CPUs | 1.0000 | 0.9885 |
| Split one server/one client CPU | 1.0032 | 1.0002 |

These are medians of seven within-pass ratios with 64-byte messages. At
1024 peers on split2 all paired rates are below one (range 0.9625..0.9999),
and p99 changes from 7211 to 8083 us. At 64 peers on split2 p99 is 441/443 us.
The 64-KiB split2 paired rate median is 1.0512, but its range is
0.7573..1.1517 and p99 is 1680/1690 us; this does not select a gain.

Peak RSS increases: at 64 peers on split2, base/spread medians are
36248/39892 KiB; at 1024 they are 79220/85720 KiB. Other placements show
similar increases. The mechanism behind the extra residency is not isolated
by this experiment; do not infer a particular cache or page-table cause.
Pure-compute medians at two/four/eight workers are 2717.25/2718.54,
1499.22/1496.12 and 1550.98/1555.10 ms. Warm-file medians are
195.24/195.37, 201.54/202.18 and 217.68/217.94 ms. Neither control supplies
a compensating benefit.

Retire stack spreading, its extra reservation bytes, diagnostic field and
comparison mode. Keep the smoke test's general usable-depth and frame-bound
checks; remove only its retired-layout-specific offset assertion. The default
layout and original guard/reservation geometry remain, and the measured
revision preserves the experiment for reproduction. This result rejects this
particular offset policy; it does not compare stackful and stackless task
representations or show that their locality costs are identical.

## Fifteenth experiment: independently locked worker ready queues (retired)

Experiment 1 changed queue preference while retaining one mutex. Its negative
result therefore did not test independent queue synchronization. The current
candidate compares three forms: the original global FIFO, one FIFO per parking
worker under the original mutex, and those same worker FIFOs under independent
mutexes. `WF_SCHED_READY_SHARDS` selects 0, 1 or 2 respectively; zero remains
the default. The second and third forms have identical queue layout, routing,
scan order and wake rules, so their paired difference isolates the lock
assignment and native mutex storage more closely than the earlier experiment.

A running owner records its worker index before an I/O park's phase transfer,
and before a cooperative checkpoint's corresponding transfer. The existing
phase protocol publishes this field to the enqueuer. Enqueue appends to that
worker's FIFO; pop first checks its own FIFO, then scans every other configured
worker's FIFO. Stealing remains permitted, so this is parking-worker preference,
not persistent connection affinity. Both worker-queue forms wake the shared
epoch on each queue's empty-to-nonempty transition. There is no global ready
counter. Compared with the global FIFO this can wake more often; it is an
explicit part of both worker-queue controls, not an unreported constant.

Mutex zero always protects the free-stack list. In form 1 it also protects
all ready queues; in form 2 worker i's ready queue uses mutex i+1. Native mutexes
and queue heads/tails are separated by 128-byte alignment. The POSIX mutex
array is initialized once during reservation, before any worker can run, with
no once check in the hot lock path. Windows uses statically initialized SRW
locks. No core path holds two list locks. Stack reservations, completion
records and their ordering, ring topology, compute deques and idle policy
are unchanged. The experiment uses the original stack-top layout and keeps
used-lane initialization off in every timed form.

The enumerator's lock operation now names a mutex. Its state snapshot includes
every lock holder, and a lock step is enabled only when that mutex is free.
Every original phase, list, wake and lifetime check remains. Ready membership
is the disjoint union of every FIFO; the checker validates each tail, routing
to the recorded worker, absence of work on unconfigured queues, and the
original prohibition on sleeping with any ready work. No schedule or state
limit is narrowed to admit the candidate.

`scheduler-shards` is wired to the existing runner. Three WF forms and the
native io_uring/epoll references run at all four CPU placements, five
connection/payload cases, two warm-ups and seven alternating passes (700 timed
network samples). The pure-compute and warm-file controls retain two/four/eight
workers. Both worker-queue forms run the complete completion suite on the
measuring Linux host before sampling; observed runs verify the selected
policy and native-ring traffic. An additional Windows CI job links the same compiler-emitted mixed I/O/compute module against all three runtime forms, checks its independent expected output at two/four workers, requires real IOCP submissions to be fully reaped, and verifies actual park/resume activity and the selected queue policy. It is correctness evidence, not a Windows timing comparison. The one-worker Linux case is retained: independent
locks cannot assume that compute parallelism requires two workers for I/O.

The initial independent-lock implementation passed the complete M1 completion
suite before adding the shared-lock control. Native eight-peer echo and paced
mixed-compute checks subsequently passed every response in all three final
forms at one and two workers, with actual cooperative switches in the mixed
case. All three final policy-numbered completion suites then passed on the M1, including every maintained enumeration configuration with zero bounded states. The shared-lock and independent-lock forms both reach 4656736 states for S24 at two threads/four stacks; each list mutation remains one protected operation in this model, so physical lock contention is measured separately.

### Ready-queue results and retirement

Revision `2f9468788790ca466a53e88d3b4f14634fe9c4ad` completed
[Linux run 34046410559](https://github.com/mbbill/Whitefoot/actions/runs/34046410559),
artifact `9993544474`: 700 timed network samples on an AMD EPYC 9V74 VM with
four logical CPUs/two SMT cores and clang 18.1.3. Both candidate completion
suites passed before timing. The same run's native Windows job passed all
three policies at two/four workers with exact output, actual IOCP traffic and
park/resume activity. The revision's canonical gate (`34046410524`), host
qualification (`34046410580`) and cross-platform performance qualification
(`34046410531`) also completed successfully.

Seven-pass median paired throughput ratios to the original FIFO are:

| CPU placement | Shared-lock queues, 64 peers | Independent queues, 64 peers | Shared-lock queues, 1024 peers | Independent queues, 1024 peers |
| --- | ---: | ---: | ---: | ---: |
| shared4 | 0.9900 | 0.9810 | 0.9946 | 0.9866 |
| shared2 | 0.9976 | 1.0042 | 0.9999 | 0.9985 |
| split2 | 0.9972 | 0.9996 | 1.0072 | 1.0150 |
| split1 | 0.9991 | 0.9984 | 1.0008 | 1.0104 |

Each cell's paired range includes one. At split2/64, independently locking
the worker queues versus sharing their mutex gives 1.0003, range
0.9982..1.0053. Base/independent/epoll rates are 212646/213390/237208 trips/s,
with median server CPU costs 8.672/8.594/7.969 microseconds per trip. The
independent queues do not close that rate/cost gap. At one peer, independent
locks recover much of the shared-lock queues' regression: on shared4 their
paired ratio to shared-lock queues is 1.4948, but only 0.9387 to the original
FIFO. Splitting a lock can repair overhead introduced by extra queue scans
without improving the original design.

Pure-compute base/shared-lock/independent medians are 2109.40/2108.52/2106.87 ms
at two workers, 1165.16/1174.54/1151.87 at four, and
1188.86/1416.18/1225.91 at eight. The independent variant helps the four-worker
control modestly but regresses the oversubscribed one; the shared-lock scan
is especially costly there. Warm-file medians are 153.57/151.08/152.49,
157.41/162.53/162.45 and 171.56/159.51/157.94 ms respectively. These local
tradeoffs do not select either queue variant as the default.

Retire both variants and their extra mutex/enumerator state. Restore the
single FIFO and original primitives while retaining all original schedules,
checkpoint tests and stack-bound checks. This result is scoped to stealable
parking-worker queues with the existing shared completion engine and wake
protocol. It does not test persistent connection ownership, a fully local I/O
engine, or a different continuation representation. The measured revision
retains the exact prototype and Windows checks for reproduction.

### Continuation-lowering feasibility probe

A separate temporary LLVM probe tested whether a general backend facility can
replace the former restricted source-shaped continuation emitter. It uses a
loop with two suspension sites, branching, an addressed value retained across
suspension, and a parent calling a suspending child. Two hundred independently
checked executions with varying zero/nonzero trip counts passed on the M1,
using caller-owned 256/512-byte frame buffers without allocator calls. This is
an ABI/code-generation feasibility result, not a WF compiler path, native I/O
benchmark, or evidence that arbitrary frames fit those buffers.

The [LLVM coroutine documentation](https://llvm.org/docs/Coroutines.html)
describes splitting ordinary LLVM functions at suspension intrinsics and
retaining live state in coroutine frames. Its returned-continuation form can
use caller-provided storage and allocate when that storage is insufficient.
That offers a route to general CFGs without hand-spelling a state machine for
every source pattern, but requires a precise allocation/lifetime contract and
integration with the completion publication protocol.

The installed Apple clang 21.0.0 (`clang-2100.1.1.101`) differs from the upstream
18.x/21.x retcon test signatures: the fixed-signature `coro.id.retcon` call is
rejected, while a variadic declaration/call passes that check. The nested
probe then reaches a backend error about multiple defining `coro.begin` calls
unless the child is marked `noinline`. With that restriction the complete
probe runs correctly and lowers to ordinary ramp/resume functions. These
observations require toolchain qualification before using this ABI in WF;
they do not establish that source-visible suspension annotations are needed.
Current SCOPE-3 permits host resource exhaustion outside the source outcome
model, but a future backed-concurrency promise still needs explicit capacity
reasoning. No source rule or public ABI is changed by this temporary probe.

A follow-up probe used LLVM's switched-resume ABI, with the intrinsic forms
observed in this installed clang's own C++20 coroutine output. The same nested
loop/branch/addressed-value cases, plus cancellation while a child is suspended,
passed 200 independent checks with all allocations reclaimed. Unlike retcon,
this form needed neither the variadic-ID adjustment nor a `noinline` child.
At `-O2`, LLVM embedded the child's state in the parent's frame: the external
allocator was called once per parent activation, for 88 bytes, and no child
allocation remained. This resolves one feasibility concern on the M1; it is
still not a WF coroutine emitter, a multithreaded completion publication test,
or a throughput comparison. Normal completion and premature destruction were
both tested because a parent must retain a suspended child's borrowed storage
until that child has ceased accessing it.

## Sixteenth experiment: sequential functions on an owner-local event loop

The preceding queue experiments retain the shared completion engine and
stealable stacks. They cannot distinguish the cost of retaining ordinary
call stacks from the cost of sharing and migrating connection work. This
comparison changes the continuation representation inside the native epoll
reference, while keeping its connection ownership and event loop.

`WF_BENCH_STACKFUL` includes `epoll_stackful.h` from the existing reference.
Each accepted connection runs a sequential handler with ordinary nested
receive/send calls. Receive offsets, send offsets, recurrence values and
remaining loop iterations are automatic locals spanning any number of
suspensions. The handler uses the exact `sched/switch.h` context switch used
by WF. Each descriptor slot has a guarded 64 KiB stack, reserved before the
listeners start and prepared only when accepted. This size is sufficient for
this measured C program, not a general bound on WF call depth.

The original SO_REUSEPORT listeners, per-worker epoll instances, edge-triggered
draining, read scratch, pending-send buffers, owner FIFO and wake channels
remain. The new handler copies the unsent suffix out of shared receive scratch
before its first blocked send, so another connection can reuse that scratch
while it waits. A final switch returns to the owning event loop before the
descriptor is closed and available for reuse. No connection migrates and no
hot operation allocates. These are properties of this native reference, not
newly proved properties of the WF compiler.

The compute form preserves the manual handler's queue turns: a nonzero
request yields before the first chunk, and each unfinished chunk yields
again. Both forms use the same 16384-step quantum, at most eight completed
replies per turn, and polling between groups of at most eight FIFO turns.
The sequential handler's reply budget belongs to the current owner turn and
resets on every resume, including a resume inside a nested I/O call.

`scheduler-stackful` compares ordinary echo in the manual epoll and stackful
forms, plus WF and native io_uring, at all four CPU placements and five
connection/payload cases (560 timed rows). `scheduler-stackful-paced` compares
inline and 16384-step manual/stackful handlers, plus base WF and canonically
chunked WF, at all four placements. It retains a zero-compute control and
long-compute loads at 4800/24000 fixed light arrivals/s (504 timed rows).
Both use two warm-ups and seven alternating passes. The difference of interest
is stackful/manual under the same engine and quantum; a WF comparison still
includes engine and scheduler differences. CPU/request, peak RSS, class
capacity, scheduled-arrival tails and per-peer tails remain reported.

On the M1, a temporary kqueue compatibility layer allowed correctness-only
execution: echo, inline compute and quantum compute all passed every byte
with one/two/four configured workers. This layer is not shipped and supplies
no Linux performance evidence. The maintained `stream_check.c` then passed
all 20 fixture invocations through the exact `stackful-check` target: delayed
readers with forced small send buffers, 2 MiB independently patterned streams
per peer, half-close after the complete stream, byte-fragmented compute
requests with three fixed independent answers, and premature close within a
frame. Observed stackful echo had thousands of actual blocked sends; quantum
cases resumed both I/O and compute suspensions. The send-buffer override and
observer are absent from timed binaries. Linux `scheduler-check` runs these
checks in addition to every existing enumeration, and each measuring job runs
them before timing.

All three original manual forms produce byte-identical optimized LLVM on the
M1 before/after this change, excluding the module/source-file name lines.
The measurement runner repeats that comparison on its Linux clang against
`2f9468788790ca466a53e88d3b4f14634fe9c4ad` and saves the modules. The restored
baseline scheduler also passed the complete M1 completion suite after
experiment 15's retirement. This experiment does not yet implement a new WF
backend or select stackful versus stackless lowering; it measures whether an
ordinary sequential call representation can approach the hand-written state
machine when ownership and the I/O engine are local.

### Sixteenth result: ordinary call stacks do not impose the measured gap

Revision `001262a39276b6dda6f3aa1da794a41f94f4eb0e` completed all 560 echo and
504 paced rows in [run 34048405406](https://github.com/mbbill/Whitefoot/actions/runs/34048405406).
The two jobs each reported AMD EPYC 7763, four logical CPUs on two physical
cores, and clang 18.1.3. They are separate VMs; only within-job comparisons
are paired. Artifacts `9994002749` and `9994113353` retain raw samples,
observers, native checks and optimized modules. The canonical gate, host
qualification and all-platform I/O benchmark workflows also passed at this
revision. All 20 Linux stream checks passed, including actual blocked sends
and compute yields; the optimized manual reference modules were unchanged.

With one server CPU on a different physical core from the client, the median
paired stackful/manual echo throughput ratios were 0.9962, 0.9946, 0.9995 and
0.9972 at 1/4/64/1024 small-payload peers. At 64 peers the rates were
128833/128932 requests/s, p99 536/542 us and CPU/request 7.734/7.734 us.
At 1024 peers, rates were 125558/125730 and peak RSS 5804/1864 KiB; retaining
ordinary stacks costs resident memory even when capacity is nearly equal.
The large-payload ratio was 1.0192, with a wide 0.8927..1.6484 paired range.
Multi-worker cells also varied substantially: split2/64 small peers had a
0.9538 median ratio and 0.6938..1.2951 range. These samples do not resolve
small multi-worker representation differences or isolate their variability.

The fixed-arrival comparison is more decisive about compute suspension. With
long computation and split2 placement, the 16384-step sequential/manual
heavy-rate paired ratios were 1.0000 (0.9812..1.0191) at 4800 light arrivals/s
and 0.9976 (0.9951..1.0146) at 24000. Median heavy completions/s and scheduled
light p99 are:

| Light arrivals/s | WF base | WF chunks 16384 | Manual C 16384 | Sequential C 16384 |
| --- | --- | --- | --- | --- |
| 4800 | 502 / 685123 us | 464 / 1086 us | 480 / 680 us | 480 / 674 us |
| 24000 | 502 / 953385 us | 384 / 640 us | 415 / 729 us | 416 / 639 us |

Inline sequential/manual C both retained about 502 heavy completions/s while
allowing roughly 0.7..1.0-second light tails. Under split1 the two quantum
forms had exactly equal heavy capacity in every paired pass at both offered
rates. Shared2/shared4 quantum capacity medians were also within 0.4%; tails
were not uniformly equal or better. The sequential representation preserves
the capacity/latency tradeoff of the same owner-local engine in these cases.
This is evidence against attributing the existing WF runtime gap to ordinary
sequential call stacks alone. It does not select a general stackful backend,
prove an arbitrary call-depth bound, or establish universal performance.

The full WF runtime still has a measurable gap. In the echo job at split2/64
small peers, WF/manual C rates were 157541/184524 and CPU/request
11.562/10.391 us. At 1024 peers, WF/manual/sequential RSS was
79288/1852/5740 KiB. Both task storage and buffer policy differ, so this is
not an isolated measurement of stack memory. Keep the sequential native
reference while investigating persistent ownership in WF itself.

One further control was missing from every earlier echo comparison: the two
native C servers and the client enable `TCP_NODELAY`, while WF leaves the
host default. In this job split2/64 large-payload p99 was 41628 us for WF
and 1893/1848 us for manual/sequential C. That difference cannot yet be
assigned to the scheduler. The next comparison isolates the TCP option.

## Seventeenth experiment: align TCP packet coalescing policy

`WF_TCP_NODELAY=1` sets the option once when a POSIX listener or outgoing
socket is created. The default remains zero while the comparison is open.
This is a target packetization policy, with no new source annotation or
operation outcome. The default-route and Linux native-adapter probes read
back the option on the listener, connected socket and accepted socket; no
assumption of inheritance substitutes for a measuring-host check.

The first M1 probe failed because its new assertion required exactly one.
Darwin returned four: [XNU's TCP option getter](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/netinet/tcp_usrreq.c)
returns the nonzero flag bit for `TCP_NODELAY`. The probe now checks the
boolean property, rather than a Linux-specific representation. This was a
test defect, not evidence of failed inheritance. The candidate's complete
M1 completion suite passed after this correction, including the actual option
on all three sockets and every original scheduler enumeration.

`scheduler-nodelay` compares base WF, WF with the option enabled, native epoll
with it enabled/disabled, and native io_uring with it enabled/disabled. All six
forms use the same client (which retains `TCP_NODELAY`), four CPU placements,
1/4/64/1024 small-payload peers and 64 large-payload peers, two warm-ups and
seven alternating passes: 840 timed rows. Native option verification runs in
separate binaries before timing, reading back each accepted descriptor. The
WF candidate runs its full completion suite on the measuring host, and
observed network runs also report the selected policy and require actual
native ring submissions/completions. Neither verification calls nor observers
are present in timed native binaries. No TCP result is inferred before the
CI samples exist.

The emitted WF echo module passed every byte on the M1 under both policies,
with eight peers, one/four workers and 64/65536-byte payloads (eight cases).
The native epoll option readback and delayed-reader patterned stream passed
with both policies and one/four workers through the temporary kqueue shim.
All three default manual reference forms still produce the exact earlier
optimized LLVM. These local checks are correctness evidence only.

### Seventeenth result: the 41 ms tail is a TCP policy interaction

Revision `74e72d3cf9795f2ab285561c6d8b2a07c1aa5da1` completed all 840 timed
rows in [run 34050912298](https://github.com/mbbill/Whitefoot/actions/runs/34050912298),
on AMD EPYC 7763, four logical CPUs/two physical cores, Linux 6.17.0-1022-azure
and clang 18.1.3. Artifact `9995105413` retains every sample and option
verification. The full gate, host qualification and Linux/macOS benchmark
jobs passed. Windows warm-file qualification refused its table because two
complete cohorts remained unstable; no Windows performance result is used.

The same WF program/runtime, with only the socket option changed, produced:

| Placement, 64 peers, 64 KiB payload | WF default rate/s | WF NODELAY rate/s | Default p99 us | NODELAY p99 us | Paired rate ratio | Paired p99 ratio |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| shared4 | 31720 | 31573 | 6931 | 5295 | 0.9898 | 0.7895 |
| shared2 | 27029 | 28208 | 41667 | 2410 | 1.0304 | 0.0579 |
| split2 | 28390 | 29807 | 41625 | 2390 | 1.0680 | 0.0578 |
| split1 | 20711 | 22467 | 3485 | 3071 | 1.0453 | 0.5776 |

The shared2 p99 ratio was 0.0563..0.0657 in all seven paired passes. Split2
was 0.0497..0.9735: one default sample already had a short tail, but every
pair improved. This removes the approximately 41 ms tail without changing
ownership, source coloring, continuation representation or the scheduler.
It does not establish the exact packet-level cause, which was not traced.

The reverse native control confirms that this interaction also depends on
the engine's transfer pattern. Native io_uring without NODELAY ran at about
1562..1564 requests/s with a 41..42 ms p99 in every large-payload placement.
Enabling it increased paired throughput by 11.78x/14.41x/14.57x/14.98x on
split1/split2/shared2/shared4. Native epoll did not show that collapse; its
large-payload paired ranges were wide and did not select one policy. Packet
coalescing must therefore be controlled before attributing a tail to the
language or continuation model.

The small-payload WF gap remains. At 64 peers, NODELAY/default paired rates
were 0.9941/1.0030/1.0073/1.0010 across shared4/shared2/split2/split1, all with
ranges straddling one. At shared2/four peers every pair was slightly slower
(median 0.9869, range 0.9589..0.9950). This is not a universal packet-policy
win. For the following latency-oriented engine comparisons, retain NODELAY
on every contender and leave the general WF default unselected.

Even after alignment, split2/64 large-payload WF CPU/request was 64.375 us
versus native epoll's 39.375 us, with p99 2390/2273 us and rates
29807/32384. At split2/1024 small-payload peers, WF/native epoll rates were
143560/176812 and peak RSS 79256/1864 KiB. Correcting the transport policy
does not resolve the remaining execution or storage costs.

## Eighteenth experiment: persistent continuations and worker-owned rings

The sequential native handler in experiment 16 motivates a WF runtime
comparison that combines persistent execution ownership with a local engine.
The new comparison keeps the WF source and emitted modules unchanged. It
separates two changes: resume a suspended stack only on its parking worker,
and submit/progress I/O through that worker's own Linux ring. Initial compute
hand-outs remain stealable. This is a scheduler policy, not a new source
effect, a restriction on resource moves, or a proof of bounded progress.

The pinned policy uses the independently locked queues from experiment 15,
but stops stealing READY stacks. Every queue's empty-to-nonempty transition
still wakes the shared epoch. A temporary M1 prototype passed the complete
completion suite and six emitted echo/mixed-program checks with one/two/four
workers and no observed resume migrations. Its first enumeration run refused
the new policy because four coverage requirements demanded migrations that
pinning forbids. The policy-specific coverage now requires resumes and zero
foreign resumes; S17 also requires the entry continuation to post only while
its own worker is running. The original migration coverage remains intact
for the default policy. The enumerator additionally rejects every individual
READY-to-RUNNING transition on a different worker. Every original schedule,
terminal assertion, replay and exhaustion check remains enabled; the four
configured pinned searches completed with zero bounded executions. The
two-worker/four-stack S24 search explored 2108545 states.

`WF_IO_OWNER_RINGS=1` selects `completion/bridge_linux_owner.h`, a private
Linux bridge policy embedded and staged with the compiler runtime. It is
kept only for this engine comparison and is removed or superseded when that
comparison is settled. Each core worker initializes a ring once on its own
thread before its first native submission. Host callers outside the core
share a separate slot and retain the adapter's locks. Pure computation does
not create a ring. Existing operation records, completion publication,
typed fallback, deferred-doorbell flush points and shutdown discipline
remain. Diagnostic totals sum the rings and survive teardown. The prototype
retains the global wake epoch and adapter mutexes; it does not yet measure
targeted owner wakes, lock removal, SINGLE_ISSUER or DEFER_TASKRUN.

A global wake callback reaches each ring with announced sleepers. Each
eventfd's readable lifetime is now counted per ring under the shared wait
lock; another ring's sleeper must not leave this ring's notification
permanently readable. A native probe creates two rings on one epoch, parks
two threads on each, broadcasts once, and requires all four to wake and both
descriptors to be empty afterwards. This probe is required by each Linux
candidate using multiple rings, before any timing.

The work also exposed a registration race in the existing lazy bridge:
the wake callback and context were assigned without the lock used by their
readers, although workers could already be sleeping on the condition
variable. Registration now takes that lock. A maintained completion case
installs the native endpoint while a thread is already parked, refuses a
replacement, and checks that the original endpoint and sleeper are notified.
The M1 prototype and restored default both passed the complete completion
suite; all 36 compiler completion integration tests also passed after adding
the new header to runtime staging. These local results do not qualify the
Linux-only ring implementation.

`scheduler-owner` measures base, pinning alone, worker rings alone and the
combined owner policy against native epoll/io_uring: five echo cases, four
CPU placements, two warm-ups and seven alternating passes (840 timed network
rows), plus the existing compute and warm-file controls at two/four/eight
workers. `scheduler-owner-paced` compares base/chunked WF under the original
and combined policies against inline/quantum C (504 rows). Its two chunked
modules must be byte-identical. All these forms enable TCP_NODELAY so the
engine comparison uses the native references' packet policy. The three
runtime candidates run full completion suites on each measuring host;
observations require real native traffic, an owner ring when selected,
and no migration under pinning. A separate four-thread bridge probe forces
one first positioned read onto each actual core thread and requires four
native submissions, four rings, and correct offset-specific bytes. A separate Windows job
checks original/pinned execution with actual IOCP traffic and fixed output.
Defaults remain unchanged pending native qualification and measurements.

The integrated tree additionally passed 16 M1 emitted-program checks: echo
and long-compute/light-arrival workloads, all four policy configurations,
and one/four workers. Every byte matched, cooperative mixed cases actually
switched, and pinned configurations reported no resume migration. Darwin
uses the bounded adapter in every configuration, so these checks supply no
evidence about the new Linux ring code. The owner comparison is published
on `codex/io-owner-experiments` so its qualification can run while the
independent TCP policy measurement finishes on the preceding branch.

At `24b967fbfd5c64f9c48d3781e69e038ce4ce70db`, the canonical gate and host
qualification passed. In [run 34052295820](https://github.com/mbbill/Whitefoot/actions/runs/34052295820),
all three candidate Linux completion suites completed successfully, including
the two-ring/four-sleeper native probe for the ring candidates. The original
and candidate emitted programs passed the initial four-peer checks. The
Windows mixed program produced its fixed expected result at two/four workers
under both policies; original execution reported 1468/176 resume migrations,
while pinned execution reported zero in both cases with actual parks/resumes.
Artifacts `9994963374`, `9994965569` and `9994933984` retain those checks.

Both measuring jobs then stopped before timing because the new AWK observer
assertion put a continuation operator at the beginning of a line. The
expression is corrected, and an explicit two-ring-probe success marker is
required so future measuring logs show that the candidate check actually ran.
No performance conclusion is drawn from these preflight runs.

At `2c5d7f947e2b123dbce07ceb96de6e15e1f5a7e0`, the paced preflight
exposed a genuine distribution weakness: split2/chowner16384/64 peers
completed all bytes with 4,225 parks/resumes, zero migration, 15,360 checkpoint
switches, but zero stolen hand-outs and only one submitting ring. Pinning
preserved an initial single-thread placement instead of balancing it later.
The former requirement for two rings in every opportunistic 64-peer run was
therefore not a bridge invariant. It is replaced by the deterministic
four-thread bridge test above, while the preflight retains and prints the
actual ring/steal counts, including the unbalanced case. There is no retry
until distribution happens to improve, and timed runs retain this policy's
load-balancing cost. The two-ring/four-sleeper wake test remains required.

### First complete echo result

At `2c5d7f947e2b123dbce07ceb96de6e15e1f5a7e0`, the echo job
[completed](https://github.com/mbbill/Whitefoot/actions/runs/34052879865/job/101539509498)
all 840 timed rows plus compute/file controls. Its
[artifact](https://github.com/mbbill/Whitefoot/actions/runs/34052879865/artifacts/9995525626)
contains every seven-pass cell and preflight. This is an AMD EPYC 7763 host,
four logical CPUs/two SMT cores, Linux 6.17.0-1022-azure, clang 18.1.3.
Ratios below are medians of same-pass ratios, with their full ranges.

| Placement / small-payload peers | Pinned/base rate | Rings/base rate | Combined owner/base rate |
| --- | --- | --- | --- |
| shared2 / 64 | 0.5612 [0.5421, 0.6695] | 1.1350 [1.0949, 1.1495] | 0.5658 [0.4863, 0.6940] |
| shared2 / 1024 | 0.6121 [0.4627, 0.6566] | 1.1288 [1.0899, 1.1420] | 0.5820 [0.5219, 0.7446] |
| split2 / 64 | 0.5667 [0.4982, 0.6746] | 1.1390 [1.1284, 1.1478] | 0.7294 [0.5119, 1.0593] |
| split2 / 1024 | 0.5374 [0.4695, 0.5542] | 1.1228 [1.1136, 1.1329] | 0.5106 [0.4740, 0.6817] |
| shared4 / 64 | 0.7247 [0.5341, 0.8220] | 1.0178 [0.9926, 1.0461] | 0.9023 [0.5578, 1.0868] |
| shared4 / 1024 | 0.8657 [0.5084, 0.9268] | 1.1300 [1.0984, 1.1531] | 1.0699 [0.8009, 1.1469] |

Independent rings with migration improve two-worker throughput in every
paired 64/1024-peer sample, but worsen tails: split2 p99 ratios are 1.1522
[1.1200, 1.1772] and 1.3817 [1.1835, 1.6580]. At split2/1024, baseline,
rings, native io_uring and epoll median rates are 152706/170583/170642/179783;
p99 values are 8044/11114/6697/8559 us and CPU costs
11.963/11.768/10.400/10.645 us per actual exchange. Equal capacity against one
reference does not remove the CPU, tail or memory gap.

The same ring policy loses every paired one/four-peer throughput sample on
multi-worker placements: split2 ratios are 0.7760 and 0.8763, shared2 0.7479
and 0.8754, shared4 0.8658 and 0.7750. With one server worker, rings/base is
0.9986 and 0.9992 at 64/1024 peers, with ranges crossing one. Four-worker
64-peer tails also worsen strongly despite unresolved throughput change.
No fixed replacement is selected from the ring throughput wins.

Pinning is substantially worse at high small-payload concurrency. Initial
hand-outs remain stealable, but stealing does not imply a balanced assignment:
one worker can take nearly all connection handlers. In the untimed shared2
64-peer combined run, 64 hand-outs were stolen, no resume migrated, and the
runtime made 2013 kernel waits/2017 host wake writes, against the baseline's
136/139. The split2 combined counts are 1551/1532, against 166/80. These are
separate observations, not counters collected inside the timed runs, and do
not by themselves prove that wake cost explains all of the loss. Together
with the one-ring paced preflight they identify placement and wake policy as
necessary work before persistent ownership can be competitive. This rejects
the tested combination, not the owner-local engine demonstrated in experiment
16. A future owner policy must distribute initial I/O tasks deliberately or
permit an explicit rebalancing mechanism.

Large transfers do not give a general owner win. Split2 combined/base rate
is 0.9650 [0.8662, 0.9758], with p99 ratio 1.0886; combined CPU falls to
39.688 us/exchange from 65.625, close to epoll's 40.312, but throughput remains
27884 versus epoll 30742. With one worker, WF remains about 22k large exchanges/s
against epoll's median 39.9k, so a single shared queue lock cannot explain that
engine/workload gap. One epoll sample is slow; do not present its median as an
all-pass superiority claim.

Compute controls are unchanged at two workers (~2415 ms). At eight workers,
pinning/combined reduce median time from 1383.55 to 1348.21/1347.31 ms. Warm
file plus compute medians for base/pinned/rings/combined are
179.62/175.38/169.55/165.99 ms at two workers,
185.90/179.92/195.47/174.29 at four, and
193.53/172.48/214.08/171.65 at eight. The controls therefore also show a
workload-dependent tradeoff. The first paced job is the failed preflight
above; its subsequent qualified run is required before drawing a mixed-load
conclusion.

The exact 2c revision passed the canonical gate and host qualifications; its
broad Windows benchmark rejected an unstable compute cohort after two complete
attempts (the Linux and macOS benchmark jobs succeeded). The subsequent
`7421a2580eaa8741b58728d3cee68a3e50327852` revision, which changes qualification
and documentation rather than the measured runtime, passed gate, io-hosts and
all io-bench jobs. Keep the failed cohort visible rather than treating it as a
valid Windows performance table.

### Qualified repeat and fixed-arrival mixed load

[Run 34053944411](https://github.com/mbbill/Whitefoot/actions/runs/34053944411)
at `7421a2580eaa8741b58728d3cee68a3e50327852` completed both Linux cohorts,
the Windows check, and all gate/io-hosts/io-bench workflows. Every ring policy
passed the independent two-ring wake test and four-thread native bridge read.
The [echo artifact](https://github.com/mbbill/Whitefoot/actions/runs/34053944411/artifacts/9995817394)
has 840 rows on EPYC 7763. It reproduces the first result: split2 rings/base
paired rate is 1.1411 [1.1334, 1.1429] at 64 peers and
1.1214 [1.0849, 1.1338] at 1024, while p99 ratios are
1.1491 [1.1463, 1.1545] and 1.4139 [1.1912, 1.5840]. The combined owner/base
rates are 0.5890 [0.5299, 0.9262] and 0.4796 [0.4287, 0.5590]. Ring throughput
and tail tradeoffs, and the large pinning loss, survive the qualification fix.

The [paced artifact](https://github.com/mbbill/Whitefoot/actions/runs/34053944411/artifacts/9995745423)
has 504 rows on a different host, EPYC 9V74, with four logical CPUs/two SMT
cores. Compare forms within that job; do not compare its absolute rate with
the 7763 echo job. The table uses medians of **heavy completions before the
one-second deadline** and light p99 measured from scheduled arrival. All
requests are subsequently drained and byte-checked. Light offers are the
aggregate across 48 light peers; 16 heavy peers remain saturated at 2097152
compute rounds/request. `Chunk` and `chunk+owner` use identical 16384-step WF
modules; `native chunk` is the existing epoll reference at that quantum.

| Placement | Light offer/s | Chunk heavy/s | Chunk light p99 us | Chunk+owner heavy/s | Chunk+owner light p99 us | Native chunk heavy/s | Native light p99 us |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| shared2 | 4800 | 416 | 656 | 192 | 1112 | 417 | 687 |
| shared2 | 24000 | 352 | 662 | 144 | 1128 | 366 | 705 |
| shared4 | 4800 | 816 | 3114 | 192 | 1101 | 831 | 2875 |
| shared4 | 24000 | 656 | 2879 | 144 | 1154 | 698 | 3048 |
| split2 | 4800 | 416 | 651 | 192 | 1053 | 420 | 722 |
| split2 | 24000 | 352 | 753 | 128 | 1113 | 367 | 706 |
| split1 | 4800 | 208 | 1104 | 208 | 1039 | 208 | 915 |
| split1 | 24000 | 160 | 1029 | 160 | 1043 | 160 | 983 |

Every multi-worker paired heavy-capacity ratio is below one. Split2
chunk+owner/chunk ratios are 0.4615 [0.4615, 0.9952] and
0.3765 [0.3636, 0.9716] at the two offers. Shared4 medians are 0.2353 and
0.2195: better light latency there accompanies a roughly three-quarter loss
of heavy capacity, not a Pareto improvement. One-worker capacity is near
identical, consistent with assignment/wake interaction rather than an
intrinsic cost of the chunked ordinary callee.

Aggregate rate conceals this failure. At split2/24000, chunk and chunk+owner
report 23758.6/23683.7 total exchanges/s, while heavy deadline completions
fall from 352 to 128. CPU per completed request also falls when fewer expensive
requests complete; that is not evidence of better execution efficiency.
Unchunked base/owner have 0.72..1.03-second light tails in the multi-worker
long-compute cases. Ownership placement alone does not supply preemption.
The zero-compute split2 control also loses rate with pinning:
base/chunk/owner/chunk+owner/native inline/native chunk are
141248/140416/83346/85967/191212/192023 exchanges/s.

These results reject opportunistic initial stealing plus permanent affinity
as the next runtime choice. They strengthen the separate experiment in
deliberate initial distribution, while leaving dynamic load imbalance and
wake cost open. Equal numbers of connections need not have equal CPU demand.

## Nineteenth experiment: compact stack metadata and first-use contexts

A reserved stack currently receives both a state header and an initial switch
frame during core initialization. With 1100 reserved stacks this touches a
page in every slot even when a program uses only a few of them. The existing
used-lane experiment addresses a different cost: clearing all maximum lane
storage instead of only configured lanes.

`WF_SCHED_COMPACT_STACKS=1` stores state headers in contiguous, 128-byte-aligned
cells in the core. The raw stack reservation and guard geometry are unchanged;
the old header gap remains unused so usable depth is not silently reduced.
An unused EMPTY stack has a null context. Its first exclusive free-list pop
prepares that context after releasing the list mutex; recycled stacks retain
the context saved by their scheduler loop. On Windows the same primitive
creates a fiber on first use instead of creating every reserved fiber at
startup. This is runtime storage policy and changes neither source effects
nor the emitted module ABI.

The enumerator snapshots compact headers with the core and skips raw stack
bytes only for an unprepared EMPTY context. A later first use initializes that
context before reading it. Existing schedule invariants and completion checks
remain. The smoke test retains the original layout assertions for the default,
checks initially null contexts and compact header alignment for the candidate,
and checks prepared context bounds for the entry and each worker before use.
Its pointer alignment check is native-word alignment: x86-64's initial switch
frame is 56 bytes below a 16-byte-aligned top, unlike AArch64's 176-byte frame.

The integrated compact+used-lane candidate passed the full M1 completion suite
and all four enumerations (16/16/19/18 schedules, zero bounded executions).
All 36 compiler completion integration tests passed with the updated embedded
runtime. Both default and compact smoke geometries passed their final checks.
The earlier compact-only prototype also passed the full suite. A separate
three-pass, alternating M1 check used the same emitted echo module, two workers,
1100 reserved stacks, TCP_NODELAY, 64 verified exchanges per peer and
`/usr/bin/time -l` process peak RSS. These are memory readings, not Linux
throughput evidence:

| Peers | Original RSS bytes | Used lanes | Compact stacks | Both |
| --- | ---: | ---: | ---: | ---: |
| 1 | 40075264 | 20267008 | 22380544 | 2555904 |
| 4 | 40157184 | 20332544 | 22511616 | 2686976 |
| 8 | 40206336 | 20398080 | 22642688 | 2818048 |

All 36 runs completed byte validation; within-cell ranges are at most 32768
bytes. An attempted 64-peer local cohort stalled in the original fallback
before any candidate ran and was stopped. This is the previously identified
bounded-helper progress problem, not a passed high-concurrency check or a
compact-stack regression. Native Linux is needed for the high-peer readings.

`scheduler-memory` crosses base/used-lanes/compact/both against the same native
references: five echo cases, four placements, two warm-ups and seven paired
passes (840 rows), plus compute/file controls. Every candidate runs the full
completion suite before timing. Untimed observations require the selected
storage flags and actual Linux native traffic. The Windows job retains pinned
continuation checks and adds all memory forms with actual IOCP reads and fixed
output. Buffer sizes and zero initialization remain identical. This experiment
can remove idle-stack startup cost; it does not by itself establish efficient
storage for a thousand live connection buffers. Both storage flags remain
experimental and default to zero until native results are assessed.

The first native run at `41ff2330766bc07dce6697f7726930434021bb8f`
passed all three Linux candidate completion suites and all Windows memory/
pinned IOCP checks. Gate, io-hosts and all io-bench jobs also passed. The memory
measurement job stopped before timing: the network launcher omitted `compact`
and `small` from its WF executable cases and returned status 2 at the first
compact preflight. The launcher now dispatches both already-built candidates.
Its final control selection also now includes memory, so the already-built
compute/file controls actually run for that experiment.
No timing or memory conclusion is drawn from that incomplete job; its logs
remain in [run 34055042189](https://github.com/mbbill/Whitefoot/actions/runs/34055042189).

### Native memory results

The corrected revision `a88eedd0549b175db74775f6cc27e3f320c86150` passed all
four workflows, including [memory run 34057509092](https://github.com/mbbill/Whitefoot/actions/runs/34057509092).
Its [Linux artifact](https://github.com/mbbill/Whitefoot/actions/runs/34057509092/artifacts/9996807148)
contains all 840 timed rows, candidate completion qualifications and compute/file
controls. The host is EPYC 9V74, four logical CPUs/two SMT cores, Linux 6.17
and clang 18. Compare paired forms in this job, not absolute rates with the
7763 jobs. Median peak RSS in KiB shows the two independent startup costs:

| Placement / peers | Base | Used lanes | Compact stacks | Both | Native epoll |
| --- | ---: | ---: | ---: | ---: | ---: |
| split1 / 1 | 32768 | 13044 | 24536 | 6928 | 1980 |
| split1 / 64 | 35560 | 15904 | 27848 | 7956 | 1992 |
| split1 / 1024 | 79228 | 61580 | 79236 | 59588 | 1980 |
| split2 / 1 | 32824 | 13524 | 24596 | 8516 | 1980 |
| split2 / 64 | 36020 | 18504 | 27784 | 12064 | 1980 |
| split2 / 1024 | 79456 | 61932 | 79364 | 60028 | 1980 |

Used-lane initialization saves roughly 17..20 MiB, including at 1024 peers.
Compact metadata/first-use preparation saves about 8 MiB when few stacks are
used, but essentially nothing at 1024 live peers. Combining them lowers the
one-peer split1 reading from 32 MiB to 6.8 MiB; the remaining roughly 58 MiB
at 1024 peers still needs a live-context/buffer explanation. Reserving a stack
and making every reserved stack resident are different costs.

This is principally a memory/startup result. For split2 at 64/1024 small
peers, both/base paired throughput is 1.0015 [0.9933, 1.0223] and
0.9996 [0.9807, 1.0054]. Corresponding p99 ratios are
0.9826 [0.8125, 1.0030] and 0.9507 [0.7485, 1.1305]. One-worker small-peer
medians remain within about 0.6% of base. There are qualifications: compact
alone loses every split1 one-peer pair (0.9955 [0.9668, 0.9990]); shared2/1024
compact and combined medians are 0.9828 and 0.9875. The noisy shared4/four-peer
combined ratio is 0.8006 [0.7244, 1.1465], so the result is not an unconditional
Pareto improvement. No general network-throughput replacement is selected.

The finite-process controls expose startup savings more clearly.
Base/compact/both compute medians are 2107.21/2094.08/2090.95 ms at two
workers, 1164.92/1155.58/1154.04 at four, and 1193.49/1180.04/1182.43 at eight.
Warm file+compute medians are 151.36/136.01/135.09,
159.24/143.45/141.42 and 171.81/154.76/152.76 ms. Every compact file-control
sample is faster than every corresponding base sample; system CPU falls by
roughly 13..18 ms while user work stays similar. These finite programs include
startup; do not call that a 10% sustained server-throughput gain. Used lanes
alone leave the controls near baseline, consistent with a residency saving.

Keep the two storage policies available for a later combined-runtime comparison,
without confusing their substantial cold/reserved-memory improvement with the
still-large live-connection footprint. Buffers and their initialized-byte
contract were unchanged throughout this experiment.

## Twentieth experiment: deliberate initial I/O placement

Experiment 18 pins an accidental initial assignment. The next candidate gives
compiler-admitted staged may-suspend calls an explicit initial owner before
using that same pinned continuation/per-worker-ring policy. `WF_SCHED_IO_ROUND_ROBIN=1`
is experimental and requires independently locked pinned ready queues. Its
zero default delegates to the original publisher. No source annotation,
public function signature, callee body or proof acceptance rule changes.
The staged lowering now calls the internal `wf__par_publish_staged` entry;
ordinary compute hand-outs retain `wf__par_publish` and their Chase-Lev deques.
The fallback module still refuses lane acquisition when no runtime is linked.

Each publishing worker advances its own initial-owner cursor across the
actually started contiguous worker prefix. Startup publishes that prefix
before the first acquisition returns; a configured but uncreated worker is
not a dispatch destination. Each owner queue holds an intrusive FIFO of initial
calls alongside its ready continuations, under the existing queue mutex.
Starting a call and resuming a stack alternate when both are available. An
in-place I/O join never starts another initial call above its borrowed buffer;
an exhausted compute join can execute an assigned initial call without asking
for another stack. Both completion paths retain the original record protocol.
Free-list and incoming-list membership are exclusive, so their slot link shares
storage. A local ABI probe confirms the original 304-byte slot and 48-byte
frame offset in both policies. Counters record starts per owner, independently
of steals; the timed candidate also retains this once-per-start count.

The enumerator now checks incoming-list ownership, state, duplicates, cycles,
tails, simultaneous compute-deque membership, and sleeping with runnable
initial work. S25 joins two such calls in reverse order and requires actual
park/resume coverage. S26 constrains the available prefix to one and requires
both calls there; it tests prefix routing, not an injected native thread-create
failure. Existing schedules and replay checks remain. Full integrated M1
completion suites pass with default counts 17/17/20/19 and candidate counts
18/18/21/20, all with zero bounded executions. The real-thread smoke requires
40 calls on each of four workers and the original exact compute/I/O results.

Actual compiler emission, without LLVM text rewriting, passes 18 local socket
runs: echo and long-compute chunked server, base/owner/balanced, one/two/four
workers, eight peers. Each balanced run reports exactly 8/workers calls on
every worker, correct response bytes, no resume migration, and real checkpoint
switches for the mixed program. All 45 staged library checks, the staged TCP
program integration check, 33 parallel backend tests and 10 cost-shape tests
pass. These Darwin fallback checks establish correctness, not native Linux
throughput or IOCP behavior.

`scheduler-dispatch` compares base/rings/owner/balanced plus native io_uring
and epoll over the five echo cases and four placements (840 timed rows), with
compute/file controls. `scheduler-dispatch-paced` compares base/chunk/
chunk+owner/chunk+balanced plus native inline/chunked epoll at fixed light
arrivals (504 rows). All three chunked WF modules must be byte-identical.
Memory policies remain off, all forms reserve 1100 stacks, and TCP_NODELAY is
on. Each candidate runs the full completion suite before timing, including
independent two-ring wake and four-thread native submission probes. Untimed
network observations require the exact initial distribution and zero pinned
resume migrations. Each assigned handler performs its own native accept, so
these observations also require at least one ring per participating worker.
Windows retains all prior memory/pinning checks and additionally runs the
existing four-peer staged source through actual IOCP under opportunistic and
round-robin initial placement at one/two/four workers.

This policy equalizes call counts for a single producer, not CPU demand.
Different connection classes, periodic arrival order, multiple producers,
nested hand-outs and changing workload phases can still create imbalance.
The mixed result must therefore be judged by heavy deadline capacity and light
tails together. It tests whether removing the demonstrated initial-assignment
defect makes the owner-local policy competitive; it does not select permanent
affinity, the current global wake mechanism or the per-ring locking overhead.

The dispatch revision `b8c94ecd7a48b0fb235d20e821028ac90a51aba2` passed the
canonical gate and io-hosts. Its native Windows placement job also passed all
six staged socket runs and the retained memory/pinning IOCP checks in
[run 34058004685](https://github.com/mbbill/Whitefoot/actions/runs/34058004685).
The gate, io-hosts, ordinary benchmark and both Linux measurement jobs all
completed successfully. The echo artifact has 840 validated timed rows; its
Intel Xeon Platinum 8573C host exposes four logical CPUs on two SMT cores.
The fixed-arrival artifact has 504 rows on an AMD EPYC 7763 with the same
logical/physical CPU counts. Compare policies within each job, not absolute
rates between these different hosts. Both series use seven paired passes.

For split2 echo, deliberate distribution recovers the pinned policy's large
capacity loss. Selected medians follow; rates are exchanges/s and p99 is us.

| Case | Base rate / p99 | Pinned owner rate / p99 | Balanced rate / p99 | io_uring rate / p99 | epoll rate / p99 |
| --- | ---: | ---: | ---: | ---: | ---: |
| 64 peers, 64 bytes | 249630 / 305 | 171676 / 470 | 307263 / 234 | 318904 / 232 | 280708 / 261 |
| 1024 peers, 64 bytes | 225258 / 5976 | — | 274812 / 4673 | 271990 / 4434 | 281181 / 4598 |
| 64 peers, 64 KiB | 49089 / 1656 | — | 63146 / 1295 | 46862 / 2074 | 66230 / 1165 |

Paired balanced/base throughput ratios are 1.2248 (1.1646..1.4670),
1.2242 (1.1827..1.2463), and 1.2839 (1.2134..1.3401), respectively.
Balanced/owner ratios are 1.8148, 2.4332 and 1.7002. This is a repeatable
placement improvement in these cells, not a universal replacement. Split2
four-peer throughput loses every pair to base, ratio 0.7667
(0.7488..0.8262). Shared4/64 and /1024 improve median throughput but worsen
every paired p99, ratios 3.7026 (3.3220..5.3678) and 2.8969
(1.2368..4.0525). Their balanced median tails are 2705 and 16895 us.
Shared4 large transfers also worsen every tail pair, ratio 1.7418.

CPU and memory remain behind native references in important cells. At
split2/64, balanced uses 6.641 us of server CPU per exchange versus io_uring
5.938; RSS is 35780 versus 4696 KiB (epoll 1980). At 1024, balanced uses
7.324 us and 79380 KiB versus epoll 6.592 us and 1992 KiB. The experiment
deliberately leaves both memory policies off. Finite compute controls remain
near base at two workers and improve about 1% at four; the eight-worker
median is 1815 versus 1892 ms. File+compute medians are 159/160/166 ms versus
base 167/170/196 at two/four/eight workers; these include startup.

An independent untimed split2/64 observation starts exactly 32 handlers on
each owner and has zero pinned resume migrations. It reports 4641 ring
enters, 375 kernel waits and 366 wake writes, versus pinned owner's 5413,
14 and zero. The pinned run's 1769 scheduler parks mostly are not ring waits;
it uses one ring. Balanced uses two rings and 376 scheduler parks. These are
mechanism observations from separate executions, not counters measured during
the timed samples. They motivate the next wake test without establishing its
benefit in advance.

The fixed-arrival mixed test restores most heavy capacity but does not fix
light tails. The table uses heavy requests completed by the one-second
deadline, not completions during drain; light p99 includes client backlog.
Every row has 64 peers and 2097152 recurrence steps per heavy request.

| Placement / light arrivals per second | Chunk heavy / light p99 us | Chunk+owner | Chunk+balanced | Native quantum 16384 |
| --- | ---: | ---: | ---: | ---: |
| shared2 / 4800 | 464 / 1122 | 469 / 1018 | 472 / 1204 | 472 / 651 |
| shared2 / 24000 | 384 / 930 | 160 / 1112 | 410 / 1444 | 416 / 668 |
| shared4 / 4800 | 905 / 2966 | 463 / 1075 | 911 / 3889 | 925 / 3271 |
| shared4 / 24000 | 720 / 2911 | 160 / 1234 | 745 / 3990 | 776 / 3277 |
| split2 / 4800 | 464 / 1031 | 224 / 1007 | 466 / 1324 | 472 / 646 |
| split2 / 24000 | 384 / 742 | 153 / 1094 | 408 / 994 | 416 / 669 |
| split1 / 4800 | 224 / 994 | 225 / 999 | 226 / 994 | 240 / 865 |
| split1 / 24000 | 192 / 935 | 192 / 1055 | 192 / 1094 | 192 / 858 |

At split2, balanced/native heavy-capacity paired medians are 0.9873/0.9829
at the two arrival rates. Every corresponding light-tail pair loses, with
median ratios 1.8914/1.4490 and ranges 1.1921..17.2337/1.1077..11.5402.
Shared2 also loses every native tail pair, ratios 1.8698/2.1991. Shared4 at
24000 loses both every heavy-capacity pair (median 0.9617) and every light-tail
pair (1.2671). Relative to unpinned chunk, balanced's heavy-capacity gain at
split2/24000 is 1.0625 (1.0417..1.0833), while tail variation remains large.

The zero-compute, 4800-light-arrivals control is a sharper counterexample:
split2 balanced reaches 165449 exchanges/s versus chunk 127751 and native
quantum 192472, but light p99 is 6971 us versus 2137 and 171. Every paired
balanced/native tail ratio loses, median 41.2485 (7.7412..96.7836). Shared4
balanced/chunk tail also loses every pair, median 5.4753. A candidate cannot
be selected from heavy throughput or closed-loop echo alone.

Initial placement was a real implementation defect in the tested fixed-owner
policy. Removing it supports further owner-local work, but leaves service
fairness, wake cost, shared-SMT interference, memory and dynamic load balance
unresolved. The native quantum handler independently limits each service turn
to eight replies; WF's chunk option still applies a 16384 backedge counter to
unchunkable request loops. An operation-service budget is a distinct hypothesis
to test, not yet an explanation established by these measurements.

## Twenty-first experiment: omit a running owner's redundant wake

A pinned ready queue is consumed only by its owner. When that owner is already
executing the completion drain, pushing its own ready stack need not broadcast
a wake to every scheduler. The same argument covers an initial call published
to the current owner: its scheduler will inspect the queue before sleeping.
`WF_SCHED_LOCAL_WAKE=1` tests just this omission, retaining the same ready queue,
mutex, FIFO, initial placement, ring engine and record protocol. It requires
pinned queues. Other-thread and helper publication, in-place waiters and program
exit still use the original wake paths. No source or module ABI changes.

The predicate must identify an actual executing core thread. A helper's default
thread ordinal is also zero; comparing ordinals alone would lose wakes for the
entry worker. POSIX and Windows primitives now retain a thread-local core
pointer while `wf_sched_run` executes and clear it on return. The predicate
requires both that pointer and the owner ordinal to match. The enumerator
stores the same attachment in each actor. No shared counter, new scheduling
edge or per-request allocation is introduced; the default policy omits the
attachment calls and keeps all original wakes.

The isolated balanced candidate passes the full M1 completion suite and all
four enumerations (18/18/21/20 schedules, zero bounded executions), plus twelve
byte-checked echo/mixed runs with one/two/four workers under opportunistic and
round-robin initial placement. The integrated original-placement pinned policy
also passes its full suite (17/17/20/19 schedules). New real-thread smoke checks
require an unattached helper to differ from worker zero, require calls executing
on a pool stack to identify their current core, and require the returning host
thread to have cleared that identity. The smoke initially needed its primitive
header included; after that build fix these assertions pass. All 36 compiler
completion integration tests pass. Six additional integrated runs use the
benchmark's actual `quiet` linker policy, unchanged emitted echo/chunked modules,
exact initial distribution and the reported `local_wake=1` flag; all return
correct bytes and the mixed runs perform checkpoint switches.

`scheduler-wake` compares base/rings/balanced/quiet plus native io_uring/epoll
over the same 840-row echo cohort and compute/file controls. Its fixed-arrival
companion compares base/chunk/chunk+balanced/chunk+quiet plus native inline/
chunked epoll (504 rows); all chunked WF modules must be byte-identical. Every
candidate runs the full completion suite, two-ring wake probe and four-thread
native bridge probe before timing. The Windows staged socket check retains
both existing placement policies and adds round-robin plus local wake omission.
All memory flags stay off and all TCP options, stacks and workloads match the
previous cohort. Reduced wake traffic is a hypothesis until native measurements
show its throughput, tails and CPU consequences; the experimental default is
zero.

Revision `1776d1af928a502bfb75a16a111177ac3f5d25f6` passed the gate, io-hosts,
ordinary benchmark and all three jobs in
[run 34058705179](https://github.com/mbbill/Whitefoot/actions/runs/34058705179).
Both Linux jobs expose four logical CPUs on two SMT cores, AMD EPYC 7763.
Echo has 840 validated rows and paced load 504; every cell has seven paired
passes. The native Windows job passed the retained memory checks and nine
staged socket runs, including all three initial-placement/wake combinations
at one/two/four workers.

Omitting the local wake substantially helps the shared4 one-peer echo cell:
quiet/balanced throughput ratio 1.3841 (1.1475..1.5739), p99 ratio 0.6796
(0.6634..0.7553), and median server CPU/exchange 48 versus 195 us. Shared2
and split2 one-peer CPU also falls, 96 to 46 and 85 to 48 us, but their
throughput and tail paired ranges cross parity. With one server worker,
quiet/balanced throughput ratios across the five cases stay within 0.7% of
one in the paired medians. The shared4 one-peer win does not establish native
CPU efficiency: native references spend about 24..25 us per exchange there.

At higher concurrency the wake omission does not close the performance gap:

| Placement / echo case | Quiet/balanced paired rate median (range) | Paired p99 median (range) |
| --- | ---: | ---: |
| split2 / 4 peers, 64 bytes | 1.0091 (0.9949..1.0254) | 1.0000 (0.9783..1.0217) |
| split2 / 64 peers, 64 bytes | 0.9931 (0.9767..1.0087) | 1.0101 (0.9800..1.0975) |
| split2 / 1024 peers, 64 bytes | 1.0067 (0.9424..1.0266) | 1.0036 (0.7654..2.5436) |
| split2 / 64 peers, 64 KiB | 1.0203 (0.9731..1.1000) | 0.9856 (0.9331..1.0004) |
| shared4 / 64 peers, 64 bytes | 1.0320 (0.9893..1.0827) | 1.0936 (0.6271..1.2547) |
| shared4 / 1024 peers, 64 bytes | 1.0035 (0.9809..1.0604) | 1.0546 (0.8298..1.2714) |

Every split2 four-peer quiet/base throughput pair still loses, median 0.7961.
Shared4 quiet/base 64/1024-peer tails still lose every pair, median ratios
3.0204/1.4821. Native epoll's large-transfer capacity advantage also remains:
split1 quiet reaches 21880 exchanges/s versus epoll 30529, and split2 28900
versus 35850. Different host speeds and workload behavior preclude comparing
these absolute rates with experiment 20's Intel echo job.

Separate untimed split2/64 runs report 67 wake writes for balanced versus two
for quiet, yet similar ring-enter counts (10173/10416) and essentially equal
timed throughput. Shared4/four-peer observations reduce wake writes from 1462
to 12 and idle looks from 1374840 to 710393. These establish the targeted
mechanism was exercised; they do not make every reduced counter a timed win.
Finite compute quiet/balanced medians are 2417/2416, 1324/1327 and 1345/1344 ms
at two/four/eight workers. File+compute is 165/170, 175/175 and 168/169 ms.

Paced long-compute capacity is nearly unchanged relative to balanced on
split2: paired ratios 1.0021 (0.9915..1.0172) and 1.0000 (0.9657..1.0459)
at 4800/24000 light arrivals/s. Light-tail ratios 1.0091/0.8439 have broad
ranges crossing parity. Quiet/native quantum light tails lose every pair at
both rates, medians 1.9390/1.6495; heavy capacity ratios are 0.9894/0.9615.
Shared2/24000 also loses every native capacity pair and tail pair, ratios
0.9662/2.0507. Shared4/4800 loses every native capacity and tail pair,
0.9849/1.2417. The zero-compute split2 control still has 1827 us light p99
versus native 187, every paired tail ratio worse, median 8.4251
(1.8837..23.2841). The corresponding balanced median is 2429 us on this run;
its difference from experiment 20's 6971 us is not a paired treatment effect.

Local wake omission is a qualified optimization for underoccupied owner
pools, not the solution to high-concurrency capacity or service fairness. Keep
its default zero and preserve it for a later combined-policy comparison. The
next service-budget experiment keeps balanced's original wakes in all budget
candidates so that the two changes remain independently measurable.

## Twenty-second experiment: completed-I/O service and progress budgets

The zero-compute fixed-arrival tail gap survives initial distribution and
local wake omission. The native reference has separate compute quanta,
eight-reply service turns and periodic event polling. The WF chunk option
instead leaves ordinary request-loop backedges on its 16384 counter, and a
join that finds DONE can return without progressing other I/O. A ready chain
can also keep switching stacks without visiting the idle progress path.

`WF_SCHED_IO_QUANTUM` adds an opportunity after that many completed I/O joins
on one worker. Zero keeps the original implementation. The opportunity calls
the existing progress/checkpoint path: it drains completions and switches to
an already-ready stack if available, with the original far-side park commit.
The joined record remains DONE and owned by its live caller; its storage may
not be reused before the join returns and its normal retirement finishes.
Compute joins do not charge the budget. File, socket, empty and failed I/O
results all use the same rule; no program, function or protocol is recognized.
The current compiler module, source proof, public signatures and record/frame
ABI remain unchanged.

`WF_SCHED_IO_RESET_TURN=1` resets the worker's budget on every stack transfer.
Zero retains the count across transfers so an uninterrupted chain of different
ready continuations cannot continually reset its progress budget. The counter
is private worker state, included in the enumerator's core digest; it is not
a property of a migrating stack. Observations separately report the constants
and `io_checkpoints`; the latter is absent from the timed hot path and never
selects a scheduling action. This execution policy does not bound proof work.

The initial M1 probe used reset-on-transfer quanta 1/4/16. All 24 byte-checked
echo/mixed runs pass, with one/two/four workers and eight peers, including the
unchanged balanced policy. The two-worker echo observation has 16034/56/zero
I/O checkpoints for quanta 1/4/16; mixed quantum 4 also reports zero. This
rejects assuming that every tested connection runs many uninterrupted requests.
It motivates measuring persistent worker cadence separately, not selecting a
quantum from the local fallback's speed. The final native Linux sweep uses
per-turn 1, per-turn 16, and persistent-worker 16 beside balanced and base.

S27 exercises repeated joins of a registered DONE record while a child awaits
asynchronous completion, requires an actual cooperative switch, retains
addressed local response storage, and finally checks the ordinary delivered
head and retirement. Its first draft constructed an unregistered DONE record;
the enumerator correctly refused the unmodeled atomic load. The corrected
case registers a real operation and retains it until final join, without
loosening the enumerator or any existing assertion. Complete M1 suites for
quanta 1 and 4 pass 19/19/22/21 schedules at the four configurations, all with
zero bounded executions, and pass all remaining completion checks.

The persistent-worker 16 policy also passes the full M1 suite with the same
19/19/22/21 schedule counts and zero bounded executions. Eighteen final
byte-checked runs cover all three shipped candidates, echo/mixed and
one/two/four workers, using the actual benchmark linker policies. They verify
budget/reset constants, initial placement, no pinned migration, and positive
I/O opportunities in every per-turn-1/persistent-16 run. Two-worker persistent
16 reports 1002 I/O opportunities in echo and 83 in the mixed run. All 36
compiler completion integration tests pass. Native Linux performance and IOCP
qualification still require the CI jobs below; local fallback timings select
no policy.

The benchmark callers compare five WF forms and two native references over
980 echo rows plus compute/file controls, and 588 fixed-arrival mixed rows.
All chunked candidate modules must be byte-identical; all memory policies and
local wake omission stay off, with TCP_NODELAY and 1100 reserved stacks held
constant. Every candidate runs the complete completion suite plus independent
two-ring wake and four-thread native submission probes before timing. Untimed
network observations require exact initial placement, native rings, no pinned
resume migration, the expected budget constants and positive I/O checkpoint
counts for per-turn 1 and persistent-worker 16. Per-turn 16 is allowed to
report zero: failure to reach that budget is evidence, not a qualification
failure. Windows retains prior checks and adds all three budget policies to
the real IOCP staged socket program at one/two/four workers.

This is neither a deadline guarantee nor an admission fix. The checkpoint
does not create a stack or start an unstarted call, and the fixed-capacity
fallback's progress question remains.

### Native service-budget results

Revision `c35a6ef25e6dd1b36bf50f89290e297ad5fe8011` completed Linux echo job
`101561489585` ([artifact 9997988029](https://github.com/mbbill/Whitefoot/actions/runs/34061037722/artifacts/9997988029))
with all 980 rows and controls on an EPYC 9V74. Fixed-arrival job `101561489586`
([artifact 9997980041](https://github.com/mbbill/Whitefoot/actions/runs/34061037722/artifacts/9997980041))
completed all 588 rows on a Xeon Platinum 8573C. Both have four logical CPUs,
two physical cores with SMT, Linux 6.17 and clang 18.1.3. The following ratios
pair the same pass and CPU cohort; the two jobs are separate hosts. Gate,
io-hosts and io-bench succeeded on this revision.

The completed-I/O budgets do not solve the measured tail gap. Persistent
worker cadence adds actual progress opportunities but regresses high-load
echo relative to balanced:

| split2 echo | Policy / balanced | Paired rate median (range) | Paired p99 median (range) |
| --- | --- | ---: | ---: |
| 4 peers, 64 bytes | per-turn 1 | 1.1478 (1.0575..1.1811) | 1.0328 (1.0161..1.0484) |
| 64 peers, 64 bytes | persistent 16 | 0.9545 (0.9273..0.9696) | 1.7254 (1.2832..1.7663) |
| 1024 peers, 64 bytes | persistent 16 | 0.9361 (0.9219..0.9903) | 1.7336 (1.6052..1.8111) |
| 64 peers, 64 KiB | per-turn 1 | 1.0589 (0.9978..1.0930) | 1.5956 (1.5365..1.7155) |
| 64 peers, 64 KiB | persistent 16 | 1.0143 (0.9999..1.0412) | 1.5133 (1.0009..1.5799) |

At split2/four peers, even persistent 16 remains below base on every paired
rate, median 0.7815; improving this weak balanced case does not restore base
capacity. Per-turn 16 stays near balanced at high concurrency and does not
establish a distinct improvement. Untimed split2/64 observations explain why
that candidate can be inactive: per-turn 16 reports zero I/O checkpoints,
per-turn 1 reports 256258, and persistent 16 reports 16016. The last switches
to another ready stack 16011 times. Merely increasing progress/checkpoint
counts therefore does not imply better tails or capacity.

With fixed arrivals and long compute, split1 per-turn-1/balanced light p99
ratios are 1.4610 (1.4184..1.5891) at 4800 light requests/s and 1.6229
(1.4683..1.7821) at 24000. Persistent 16 also loses every pair, 1.3559
(1.3041..1.5012) and 1.4319 (1.3140..1.5045). Heavy deadline capacity stays
essentially equal to balanced. At split2, all three candidates lose every
native-quantum light-tail pair at both arrival rates. Per-turn 16's paired
ratios are 2.0841 (1.5616..2.8856) and 1.6605 (1.3462..4.4030), while
persistent 16 is 2.0385 (1.5544..2.6345) and 1.9981 (1.2263..3.3071).
Per-turn 1 is worse, 3.1506 and 2.3045 in the paired medians.

The zero-compute split2 control remains far from native light tails: median
p99 is 2144 us for balanced, 2725/2256/2192 for the three candidates and 87
for native quantum. Every native comparison loses; paired candidate/native
tail medians are 29.9451/26.5412/24.0879, with ranges 19.8736..117.9565,
4.7174..34.9885 and 13.4505..128.3793. These noisy ratios do not select a
budget from a single favorable median. Finite compute candidate/balanced
medians stay within about 0.4%; file+compute changes are small, about
142..151 ms across the candidates at two/four/eight workers.

Keep `WF_SCHED_IO_QUANTUM=0` as the default. The per-join opportunity is not a
substitute for understanding admission, event service and continuation order.
The compiled-loop chunk result from experiment 12 remains separate evidence.

Windows scheduler job `101561489440` timed out at its existing 20-minute job
limit. Base file+compute passed at two/four workers and pinned passed at two;
the pinned/four invocation produced neither output nor a final report. This
occurred with `io_quantum=0`, before the new budget-policy socket cases were
built or run. [Artifact 9997879897](https://github.com/mbbill/Whitefoot/actions/runs/34061037722/artifacts/9997879897)
retains those partial results. The separate passing io-hosts suite does not
qualify these unexecuted cases or explain the stall. Experiment 24 isolates a
candidate Windows notification defect before any runtime fix is selected.

## Twenty-third experiment: receive storage and sequential-handler residency

The memory gap remaining after lazy scheduler initialization does not identify
the cost of the source execution model. The native epoll reference shares a
worker receive scratch and touches each connection's private pending buffer
only when a send blocks. WF's echo instead owns a 64 KiB initialized buffer
per connection. A comparison that changes both the receive storage and the
execution representation cannot attribute their separate costs.

`WF_BENCH_RECEIVE_STORAGE` isolates four native echo policies, under both the
existing manual state machine and the ordinary nested C stackful handler:

| Policy | Receive destination | Storage lifetime |
| --- | --- | --- |
| 0, shared | Worker scratch; private spill after a blocked send | Existing preallocated arenas |
| 1, arena | Connection's private 64 KiB arena slice | Existing preallocated arena |
| 2, malloc | Connection's private 64 KiB allocation | Allocate on accept, free on close |
| 3, calloc | Connection's private zeroed 64 KiB allocation | Allocate on accept, free on close |

Every policy echoes only the initialized prefix returned by receive. Private
storage retains its unsent suffix across waits without copying; it is freed
after the handler returns to its owner. Normal close clears the pointer before
descriptor reuse, registration failure frees the allocation, and final failed
run cleanup frees any remaining allocations after worker joins. No policy
allocates per request. These are native C reference changes, not permission
for WF source to expose uninitialized storage. No container, source function,
proof rule, compiler buffer lowering or runtime interface changes here.

The original manual echo, compute and quantum reference optimized LLVM remains
identical to the experiment-16 baseline after removing module path headers.
The runner verifies this on its actual toolchain before timing. In particular,
the default echo buffer expression stays a direct worker-scratch access: an
initial local-pointer rewrite changed the optimized baseline and was replaced
before measurement. Native storage observations run separately from timing and
require the requested policy, transfer size and exact accepted/closed count.
The artifact retains optimized LLVM for all eight native representations and
WF, plus page size and libc version, to expose allocator lowering differences.

The maintained `stackful-check` expands echo to all four storage policies and
retains every compute, truncated-protocol and quantum check. Its 32 M1
kqueue-compatibility runs pass, including one/four workers, forced short
writes, byte checks and the existing wait/send-wait/quantum observations. The
actual runner's eight native build paths also pass independent byte checks,
and all three unchanged-reference codegen comparisons pass. These local runs
qualify the prototype; Linux epoll behavior still requires CI.

Two complete M1 residency batches use one server worker, two client threads,
100 exchanges of 64 bytes per connection, one/64/1024 connections, and both
execution representations. The second reverses storage-policy order; all 48
runs pass. `/usr/bin/time -l` peak RSS, converted from bytes to KiB, is below;
ranges contain the two observations, not confidence intervals. The host has
16 KiB pages and Apple clang 21.0.0. No local rate selects a Linux policy.

| Connections / representation | Shared KiB | Arena KiB | malloc KiB | calloc KiB |
| --- | ---: | ---: | ---: | ---: |
| 1 / manual | 1376 | 1392 | 1360..1376 | 1360 |
| 1 / stackful | 1392 | 1392 | 1376 | 1360 |
| 64 / manual | 1376 | 2400 | 2400 | 2400..2416 |
| 64 / stackful | 2400 | 3408 | 3424 | 3408 |
| 1024 / manual | 1424 | 17776..17792 | 18048 | 18032..18048 |
| 1024 / stackful | 17808 | 34176 | 34448 | 34416..34432 |

Private receive storage and sequential-handler stack residency each add about
one host page per live connection at 1024 connections. The malloc/calloc paths
have nearly equal RSS on this host. Local optimized WF LLVM folds the emitted
malloc plus zero-fill loop into a 65536-byte calloc call; that does not imply
explicitly touching every byte of every allocation. An earlier diagnostic
with explicit full-buffer memset had much higher RSS and is not a simulation
of that optimized allocation path. The Linux allocator comparison remains
unmeasured; neither eliminating zero initialization nor a container API change
is selected from the M1 observations.

`make scheduler-storage` runs 770 paired Linux echo rows: base and the combined
used-lane/compact-stack WF control, io_uring, and all eight native echo forms;
five connection/payload cases; split1/split2 placement; seven recorded passes
after warmup. Compute and file+compute controls remain. TCP_NODELAY is fixed
on, WF reserves 1100 stacks, and dispatch, local-wake and service-budget flags
remain at their original defaults. The small-memory control runs its full
completion suite before timing. All native stream checks run before timing,
and the existing Windows memory and staged-IOCP qualification remains wired.
Other placement and paced-load comparisons are deferred until this narrower
experiment can distinguish receive storage from execution representation.

An exploratory M1 C++20 coroutine variant of the same native engine passes
30 manual/stackful/coroutine protocol cases, including short writes, truncated
compute messages and one/four workers. Two complete local RSS batches, 36
byte-checked runs with the same one-worker, two-client-thread, 100-trip setup,
compare shared scratch and private arena storage. At 1024 connections, C++
manual/shared uses 1408..1424 KiB, stackful/shared 17808 KiB and
coroutine/shared 1600 KiB. With private arena receives, they use
17776..17792, 34192 and 17952..17968 KiB respectively. These results separate
the live stack-page cost from receive storage on this M1 host; they do not
qualify a WF lowering or establish native Linux throughput.

The coroutine prototype is not yet a strongest native reference. Its local
optimized LLVM retains a 72-byte root malloc per connection and a 104-byte
child malloc in each send-response call. The observed short-write echo case
performs 138 allocations/frees for four connections, and the mixed quantum
case 32/32. Correct nested lifetimes alone did not cause this compiler to
eliminate the child allocations. A packed-frame lowering still needs an
allocation-placement experiment and real network measurements before it can
replace the existing representation comparison. No C++ source or coroutine
library has been introduced into the WF compiler.

### Native receive-storage results

Revision `361cb9520f421b8a0562e2a8618844b960562c2b` completes all 770 rows in
Linux job `101564979286`, [artifact 9998207444](https://github.com/mbbill/Whitefoot/actions/runs/34062347023/artifacts/9998207444).
The host is a Xeon Platinum 8573C with four logical CPUs on two physical SMT
cores, Linux 6.17, clang 18.1.3, glibc 2.39 and 4096-byte base pages. All
expanded stream checks, memory-control qualification and native baseline
codegen comparisons pass. Its Windows scheduler qualification, canonical gate,
native-host qualification and program I/O benchmark workflows also pass.
The optimized WF module folds the 64 KiB allocation/fill into calloc here too.

Peak RSS medians expose large layout-dependent costs even with 64-byte wire
payloads. Values below are KiB; no row is drawn from another host:

| Placement / peers | Manual shared | Manual arena | Manual malloc | Manual calloc | Stackful shared | Stackful calloc | WF base | WF small |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| split1 / 64 | 1980 | 5940 | 1992 | 2220 | 1992 | 2476 | 35664 | 10000 |
| split1 / 1024 | 1980 | 67396 | 5804 | 10028 | 5932 | 14124 | 79312 | 59584 |
| split2 / 64 | 1980 | 5972 | 2220 | 2364 | 1980 | 2604 | 36292 | 11864 |
| split2 / 1024 | 1980 | 67424 | 5804 | 10028 | 5804 | 14124 | 79448 | 63300 |

At 1024 connections, private arena receive storage adds about 64 MiB to the
manual reference, whereas per-connection malloc adds about 4 MiB and calloc
about 8 MiB. Stackful execution adds about another 4 MiB with the latter two
policies. The arena's generated code contains no full receive-buffer clearing;
its much larger RSS needs page-mapping evidence before being attributed to
initialization. Transparent huge pages can amplify sparsely touched mappings
([kernel documentation](https://docs.kernel.org/admin-guide/mm/transhuge.html)),
but this run did not record THP controls or live smaps and does not establish
that as its cause. The initialization allocator thread also differs: WF
allocates scratch in the staged caller, while native malloc/calloc policies
allocate on the accepting worker. This experiment does not isolate that axis.

Memory layout is not performance-neutral, and stackful/manual parity from a
smaller case cannot be extrapolated to every storage policy. At split1/1024,
shared stackful/manual throughput wins every paired pass, median 1.0611
(1.0101..1.1220), while calloc stackful/manual loses every pass, 0.9014
(0.8628..0.9457). At split2/64 KiB, shared stackful/manual loses every pass,
0.9459 (0.9134..0.9894), and private arena/shared manual also loses every pass,
0.9270 (0.8655..0.9729). The corresponding large-transfer medians are
69376/s for manual shared, 64952/s for stackful shared and 64204/s for
stackful calloc. These are host/workload observations, not a universal cost of
ordinary sequential calls.

Most small-memory WF/base paired rate medians remain within 1.2% of parity;
the noisier split2/one-peer median is 1.0366 with a range crossing parity.
The remaining WF gap survives a matched private-calloc stackful reference:
small/reference split2 rates lose every pair at 64 peers, 1024 peers and the
64 KiB transfer, with paired medians 0.8787 (0.8134..0.9613), 0.8553
(0.7999..0.9047), and 0.7613 (0.7470..0.8534). The 64-peer and large-transfer
tails also lose every pair, 1.1907 (1.0421..1.7110) and 1.4338
(1.2485..1.4477). At 1024 peers small/reference RSS remains about 4.2..4.4
times higher in paired medians. Allocation semantics and stack representation
alone therefore do not yet account for the full runtime gap.

Finite compute small/base medians are 3334/3338, 1813/1820 and 1887/1891 ms
at two/four/eight workers. File+compute is 154/167, 160/171 and 182/195 ms,
with about 8..12 ms less system CPU in the first two cases. These retain the
startup observation from experiment 19. The next residency experiment holds
all connections open for snapshots and compares inherited versus disabled
per-process THP without changing the compiler's buffer semantics.

## Twenty-fourth experiment: Windows wake ownership under re-entry

The Windows timeout above makes progress qualification the immediate next
runtime question. `wf_windows_iocp_notify` posts one unaddressed IOCP packet
per announced sleeper. A polling reaper returns such a packet when sleepers
remain, but `wf_windows_iocp_park` consumes a received wake unconditionally as
its own. A newer park can capture the new epoch, enter the same port before an
old sleeper reaches its kernel wait, and take the older sleeper's packet. A
second such park can exhaust the remaining broadcast packets. Whether this
explains the particular CI stall is not yet established.

The native adapter regression gains `WF_WINDOWS_IOCP_WAKE_REPLAY` hooks that
exist only in test links. Two threads announce against one epoch and stop at
an event barrier immediately before their kernel waits. After publication,
the caller performs two zero-timeout parks against the new epoch, then releases
the old threads. Both old threads must receive actual wake packets. A finite
test watchdog keeps a failure diagnosable; a timeout is explicitly not a wake.
The probe reports the exact count received and cleans up both waiters and the
port before checking it. The same case is wired into the existing native
Windows adapter caller and its cross-build, retaining the original file I/O
qualification. Production and timed links contain no replay hooks.

This first revision adds the reproducer without changing the wake protocol.
Native Windows execution is required to confirm the proposed interleaving;
the M1 host has neither the Windows headers nor the Windows kernel. No timeout
is added to a runtime progress or source-acceptance path.

The baseline replay is now confirmed on native Windows:
`3821ce739d9414aa41d8d08123e2b66ce52bec23`, io-hosts job
[`101567339867`](https://github.com/mbbill/Whitefoot/actions/runs/34063211463/job/101567339867),
reports `wake-replay expected=2 received=0` and exits 1 at the exact wake-count
assertion. Its Linux companion passes. An independent storage-revision
Windows scheduler job `101564979181` passes the original full qualification,
so ordinary repeat success does not remove this deterministic counterexample.
The replay proves a notification defect, while attributing the earlier full
program stall to that defect still requires further execution evidence.

The repair candidate records native wait calls in an intrusive list protected
by the existing runtime wait lock. Each node lives in its active native park
call and records whether publication has already assigned it a notification.
Publication posts once per previously unnotified node. Until the notified
cohort drains, a new park uses the runtime condition instead of entering the
port; the last notified call to leave wakes that condition. Zero-timeout calls
return immediately without taking old packets. A polling reaper returns a
wake only while notified native waiters remain, allowing surplus packets to
drain after that cohort ends. Native completion packets still publish through
the original record path. No writer-frame layout or generated entry changes.

The regression also covers an intervening reaper and an actual new condition
waiter, requiring both old native wake packets and a real condition wake for
the new waiter. Test timeouts satisfy none of those assertions. The new branch
retains the existing Windows memory and all staged socket policy checks and
repeats the previously stalled pinned/four-worker file+compute case 32 times;
Linux storage timings continue on their original revision instead of being
rerun for a Windows-only change. The repair was sent to native CI; no
performance claim is made for its altered wake traffic.

Revision `04106a23e9df8c7c6aefac01e9a2594af180a4d1` now passes native Windows
io-hosts job [`101568658904`](https://github.com/mbbill/Whitefoot/actions/runs/34063697994/job/101568658904):
both `gate=0` and `gate=1` replays report `expected=2 received=2`, followed by
the original native file-I/O qualification. Scheduler job
[`101568658716`](https://github.com/mbbill/Whitefoot/actions/runs/34063697974/job/101568658716)
passes all 32 pinned/four-worker repetitions, all other memory cases and all
18 staged IOCP socket configurations. The Linux io-hosts companion and
canonical gate also pass; the program I/O benchmark is still running.
This closes the demonstrated lost-notification interleaving and provides
repeat evidence for the previously stalled workload; it does not establish
that no other scheduler progress defect exists.

## Twenty-fifth experiment: process page policy and live storage

The storage result leaves two distinct questions: why a sparsely used private
arena adds roughly its entire virtual size to RSS, and why WF small still
uses over four times the memory of native stackful calloc. The next experiment
uses the same normal server binaries with process THP disable set to zero or
one before exec. Zero permits the global policy; it does not force huge pages.
The policy survives exec according to the [kernel documentation](https://docs.kernel.org/admin-guide/mm/transhuge.html).
Global THP controls are recorded, including per-size settings, and never
written. Compiler buffer initialization, scheduler flags and source semantics
remain unchanged.

`make scheduler-pages` retains the storage code-generation comparison, all
32 native stream cases, and the small-memory completion qualification. It
records 588 paired timed echo rows: WF base/small; native manual shared,
arena, malloc and calloc; native stackful calloc; split1/split2 placement;
both process page policies; 64/1024 peers with 64-byte payloads and 64 peers
with 64-KiB payloads; seven passes after two warmups. Form and page-policy
ordering alternate between passes. Every timed server passes through the same
small exec launcher, which verifies the requested process policy with prctl.
The client runs on the original disjoint logical CPU set. Compute/file control
results remain those of the storage revision: this experiment changes only
the network process page-policy comparison and makes no new claim about those
other workloads.

An untimed resident mode of the existing independent stream checker opens
every connection, exchanges and verifies a peer-specific byte pattern, and
keeps all connections open until both smaps and status have been copied from
the server. It then half-closes each peer, requires EOF without extra bytes,
and checks server exit. Linux taskset execs the child in place on the server
CPU set; the checker retains the actual server PID. Snapshot failure or a
wrong THP_enabled value fails the run. A 30-second test watchdog diagnoses
stalls without selecting runtime progress behavior.

Three repetitions cover all eleven WF/native forms from the storage study,
including io_uring and both manual/stackful forms of each storage policy,
under both page policies, both placements and all three connection/payload
cases: 396 snapshots. `resident.tsv` sums RSS, anonymous memory, anonymous huge
pages, private dirty pages and swap from smaps; the full mappings and process
status remain in the artifact. The [proc documentation](https://docs.kernel.org/filesystems/proc.html)
defines these fields. Peak RSS from timed process lifetime and live RSS from
these snapshots answer different questions and must not be substituted for
each other. Even a demonstrated page amplification would not by itself
explain the allocation-thread difference between WF's staged caller and the
native accepting worker.

Local qualification passes strict C11 compilation, all 32 maintained native
protocol cases using the existing M1 compatibility bridge, and the actual
pages runner's eight native builds plus all three unchanged manual LLVM
comparisons. The final resident client passes manual/shared 64-peer small
echo, stackful/calloc 1024-peer small echo with four workers, and
stackful/arena 64-peer large echo with one worker, including clean EOF and
server exit. Portable qualification uses the explicit dash snapshot prefix;
it makes no claim about Linux prctl or proc files. The no-policy exec launcher,
workflow YAML, embedded Bash and runner Bash syntax checks pass. The Linux
policy readback, retained mappings and performance results follow below.

### Process page-policy results

Revision `80ffb2c214361f6e54110d6586ea5e1d8c34850d` completes all 588 timing
rows and 396 live snapshots in Linux job `101571861785`,
[artifact 9998953575](https://github.com/mbbill/Whitefoot/actions/runs/34064910559/artifacts/9998953575).
The host is an EPYC 9V74, four logical CPUs on two SMT cores, Linux 6.17,
clang 18.1.3, glibc 2.39 and 4096-byte base pages. Global THP is `always`,
2-MiB pages inherit it, and every smaller recorded THP size is `never`.
Independent analysis verifies every snapshot's process-policy readback and
recomputes all reported smaps sums. Every disabled snapshot has zero
AnonHugePages. Native Windows, the canonical gate, native-host checks and the
ordinary io-bench workflow all pass at this revision.

Live RSS medians below are KiB, over three snapshots with every connection
open after its checked exchange. Arrows compare process THP permitted to
disabled on the same host:

| Placement / peers / bytes | WF base | WF small | Manual arena | Stackful calloc | Native io_uring |
| --- | ---: | ---: | ---: | ---: | ---: |
| split1 / 64 / 64 | 35540 -> 29460 | 9984 -> 5928 | 5748 -> 1896 | 2520 -> 2520 | 2412 -> 2416 |
| split1 / 1024 / 64 | 79968 -> 73052 | 61536 -> 53392 | 67208 -> 5756 | 14156 -> 14152 | 12964 -> 7924 |
| split2 / 64 / 64 | 35680 -> 29596 | 11684 -> 6384 | 5780 -> 1912 | 2648 -> 2652 | 2836 -> 2836 |
| split2 / 1024 / 64 | 79268 -> 73212 | 61992 -> 53808 | 67240 -> 5772 | 14224 -> 14228 | 13424 -> 8380 |
| split2 / 64 / 65536 | 36372 -> 30296 | 12604 -> 6992 | 6004 -> 3964 | 4356 -> 4328 | 6556 -> 6240 |

The arena result now has direct mapping evidence. At split2/1024, the first
permitted snapshot has a 70660-KiB anonymous mapping with 65572 KiB resident,
including 65536 KiB of AnonHugePages. The corresponding disabled mapping is
4100 KiB resident, with no huge pages. The same roughly 60-MiB amplification
appears in every repetition and both placements. Per-connection malloc and
calloc receive storage have no anonymous huge pages in these snapshots;
their small-payload live RSS is essentially unchanged. This establishes the
page-policy cause of the arena's inflated sparse residency on this host; it
is not evidence that private buffers intrinsically require their full virtual
size in resident memory.

WF small still has a much larger non-THP allocation. Every base/small
split2/1024 snapshot contains a 65620-KiB ordinary heap mapping with 46572 KiB
resident and zero AnonHugePages, under both policies. The first small snapshot's
other large resident mappings include a sparsely initialized static region
and three sparse 1-GiB mappings; disabling THP reduces their page amplification
but leaves the heap unchanged. Native stackful calloc instead has worker
allocator mappings holding about 8.2 MiB resident in total, plus roughly one
stack page per connection. Allocation placement remains the next independent
question: WF allocates buffers in the staged caller; native calloc runs in the
accepting worker. These maps locate the remaining difference, but do not yet
prove which allocator path or initialization behavior caused it.

Peak lifetime RSS in the timed rows follows the same large effects. At
split2/1024, manual arena falls from 67304 to 5804 KiB; WF small from 63084 to
53752 KiB; stackful calloc stays near 14 MiB. The paired small/calloc RSS
ratio remains 3.8308 (3.7886..3.8445) with THP disabled. Small's paired
throughput ratio to that reference is 0.8795 (0.8436..0.9184), and CPU per
exchange is 1.0894 (1.0519..1.1211): every pass still loses both. At
split2/64-KiB transfers with THP disabled, every small/calloc throughput pair
also loses, 0.8625 (0.8470..0.8792), and every p99 pair worsens, 1.3204
(1.2773..1.6922). Lower page amplification does not close the runtime gap.

Disabling THP is not selected as a universal performance policy. Most
within-form paired throughput ranges cross parity, including every WF cell.
For manual arena at split2/64-KiB transfers, disabling it loses every rate
pair, median 0.9699 (0.8465..0.9835), and increases CPU per exchange, 1.0357
(1.0088..1.0654). Conversely, manual calloc wins every split2/64-peer rate
and tail pair under disable, even though its live snapshots show no THP and
similar memory; timed mappings were not captured and placement/noise remain
possible factors, so that win is not attributed to huge-page elimination.
The one-worker small/calloc reference also wins all permitted 1024-peer rate
and tail pairs (1.0305 and 0.9157 medians). Host/workload dependence remains
visible; none of these findings supports a universal representation winner.

## Twenty-sixth experiment: nested coroutine frames in the same native engine

The exploratory C++ coroutine control retained a heap allocation in each
nested send-response call. Before using it as a performance reference, test
whether the compiler can put the child state inside its parent. Clang's
[`coro_await_elidable` attribute](https://clang.llvm.org/docs/AttributeReference.html#coro-await-elidable)
allows this for directly awaited coroutine calls when the child cannot outlive
its caller. This is a lifetime promise, not a request to ignore a failed
allocation or abandon an unfinished child. The prototype already gives each
parent exclusive ownership of its nested task; destroying a suspended parent
first destroys the owned child and then frees the parent's storage.

On Apple clang 21 at O2, that hint removes the nested allocation sites from
the normal, unobserved LLVM. Echo changes from a 72-byte root plus a 104-byte
allocation per send-response call to a single 176-byte root. The quantum
compute form changes from an 88-byte root plus 64-byte receive and 104-byte
send children to one 256-byte root. These are this compiler's frame sizes,
not ABI constants or guaranteed sizes on Linux. The observed four-connection
backpressure check changes from 136..139 allocations to four; fragmented
compute and quantum cases change from 32 to four. Both direct heap and
parent-contained forms free every allocation.

The same native engine now compiles as C or C++20 instead of maintaining a
source copy for each language. C keeps its original atomic types and control
flow; C++ uses the standard atomic equivalents and explicit pointer casts.
`epoll_coroutine.h` supplies nested sequential handlers, selected at build time
alongside the existing manual and stackful handlers. A connection owns one
root handle and the currently suspended leaf handle. Only its owner worker
resumes that leaf. Normal completion destroys the root before closing the
connection; registration failure and worker failure also destroy owned frames
before freeing their receive storage. The existing accept, edge-triggered
polling, per-worker FIFO, eight-turn service budget, quantum and byte protocol
remain shared. The unelided form is retained as an allocation control.

`make coroutine-check` is wired into the Linux portion of the canonical
scheduler check. It runs 48 C++ protocol cases across manual, stackful, heap
coroutine and parent-contained coroutine forms: shared/private-calloc echo,
fragmented compute, quantum compute, truncated requests, and one/four workers.
It checks allocation/free balance, exactly one elided root per connection,
actual waits, short-send waits and quantum yields where the workload requires
them. The original 32 C cases remain wired. `coroutine_lifetime.cpp` also
creates and destroys 1024 parents while their receive child is suspended;
both allocation forms run under ASan/UBSan and require exact allocation and
free counts. The M1 shared-engine run passes all 48 cases and both sanitizer
runs, with 2048/2048 heap allocations/frees and 1024/1024 elided ones.

The lifetime check does not qualify destruction of a WF kernel I/O loan.
This native readiness handler suspends after recv returns EAGAIN and leaves
no buffer address borrowed by an outstanding kernel operation. A completion
backend still has to retain buffers and records until terminal completion,
respect join return before reuse, and handle cancellation without shortening
those lifetimes. Likewise, this reference does not yet implement nested WF
parallel hand-outs or prove bounded-capacity admission progress.

`make scheduler-coroutine` measures 910 paired echo rows on split1/split2:
WF base/small, native C io_uring/shared-epoll/calloc-epoll, and C++ manual,
stackful, heap-coroutine and parent-contained-coroutine forms crossed with
shared scratch/private calloc. It retains all five prior small/large echo
cases, seven recorded passes after two warmups, and WF compute/file controls.
`make scheduler-coroutine-paced` measures 336 paired fixed-arrival rows:
WF base/chunked/balanced-chunked, C manual quantum, and all four C++ quantum
representations, on the same two placements and three zero/long-compute
arrival cases. Heavy completions by deadline and light backlog-inclusive tails
remain separate metrics. Each CI job is its own host; the two jobs' timings
must not be combined into cross-host ratios.

Every timed form uses clang 20 in these jobs, including C and WF runtime
links. Ubuntu 24.04 [packages clang 20](https://documentation.ubuntu.com/ubuntu-for-developers/reference/availability/llvm/);
the workflow installs it and its sanitizer runtime. Canonical C checks keep
the usual C compiler, while the new C++ checks require a compiler supporting
the elision attribute (clang++-20 is selected when installed). Timed and
observed binaries are separate, their optimized LLVM is retained, the original
three C manual LLVM comparisons remain mandatory, and THP controls are
recorded without changing them. Native Linux results follow below.

This is evidence for a compact sequential implementation candidate, not a
selected WF lowering. The C++ control explicitly uses coroutine types and
co_await. Whether WF can infer an internal suspension representation while
retaining its current source signatures remains a compiler/ABI design task;
these measurements alone cannot decide that question or establish a universal
throughput, memory or tail-latency win.

The updated shared source also passes all 32 original C stream cases on M1,
and its default C echo/compute/quantum LLVM remains identical to the retained
manual reference after removing only module filename metadata. The actual
runner build blocks produce all twelve C++ normal forms, their twelve observed
companions and optimized modules. Sixteen four-worker protocol runs of those
unpressured observed companions pass, including exact elided allocation counts
and truncated-stream exits. Workflow YAML, embedded shell and runner syntax
checks pass. A registration failure explicitly retires both coroutine handles
and marks the connection inactive before returning to the event loop.

The first Linux coroutine qualification at `b5aceb4c` stops before running a
C++ case: clang 20 rejects the command's trailing `-x none` as an unused
argument under Werror. Both the canonical Linux scheduler job `101573950449`
and paced job `101573950304` report that exact error. The M1 compatibility
wrapper had appended object files after the flag, masking the invalid native
invocation. Remove the unnecessary trailing flag from both maintained build
commands; the local wrapper alone resets language selection immediately before
its own object inputs. No warning is suppressed and no check is removed.
There is no coroutine timing result from those failed jobs.
The corrected commands pass all 48 local protocol cases and both sanitizer
lifetime probes again. A direct compile-only invocation, without the local
wrapper's appended objects, also passes with Werror retained.

At corrected revision `2de6c00039243aee98554eabba5143f011991461`, the
[canonical gate](https://github.com/mbbill/Whitefoot/actions/runs/34066113699)
passes all fourteen Linux/macOS jobs. Linux scheduler job `101575057169`
passes the original C stream cases, all 48 C++ cases and both nested-frame
sanitizer probes (2048/2048 heap allocations/frees; 1024/1024 elided ones).
Native-host and Windows placement checks also pass. Echo and paced timing
jobs subsequently complete; their independently checked results follow.

### Native coroutine results

Revision `2de6c000` completes 910 echo rows in Linux job `101575042832`,
[artifact 9999338615](https://github.com/mbbill/Whitefoot/actions/runs/34066113672/artifacts/9999338615),
on an EPYC 9V74, four logical CPUs on two SMT cores, Linux 6.17 and clang
20.1.2. All ten workload/placement cells contain thirteen forms and seven
distinct passes. Independent analysis checks exact exchange counts, retained
client outputs, and all three unchanged C manual LLVM comparisons. The
native checks pass all 32 C and 48 C++ stream cases plus both sanitizer probes.

Elision is real on Linux too. The four-peer echo preflight changes 84 heap
allocations/frees (8576 bytes cumulatively) to four roots (672 bytes). Normal
LLVM uses a 64-byte heap root with a 104-byte child allocation per send, versus
one 168-byte root with the child contained inside it. The quantum preflight
changes 168 allocations/frees (14016 bytes) to four 248-byte roots (992 bytes).
These are frame allocation counts, separate from receive buffers and libc's
physical page decisions.

Peak RSS at 1024 small-packet peers shows the representation's memory value:

| Placement / receive storage | C++ manual KiB | C++ stackful KiB | C++ elided KiB | Paired elided/stackful RSS |
| --- | ---: | ---: | ---: | ---: |
| split1 / shared scratch | 4096 | 7904 | 4364 | 0.5521 (0.5506..0.5541) |
| split2 / shared scratch | 4240 | 7792 | 4340 | 0.5487 (0.5362..0.5633) |
| split1 / private calloc | 12012 | 16096 | 12256 | 0.7614 (0.7566..0.7632) |
| split2 / private calloc | 12024 | 16124 | 12256 | 0.7551 (0.7539..0.7632) |

The roughly four-MiB difference is consistent with about one resident stack
page per connection; these are peak measurements, separate from WF's much
larger caller heap. C++ manual itself has a higher
baseline RSS than the C manual executable; comparing the C++ forms within
one link environment avoids assigning that baseline to coroutine frames.
THP is permitted and global policy is always, so these peak values do not
replace the separately controlled live-mapping experiment.

There is no stable overall throughput winner among the native representations.
Every elided/manual and elided/stackful paired throughput range crosses one
in this echo panel, for both storage forms. Most split1 medians are close to
parity. Split2/four-peer samples vary widely even between C and C++ manual
controls; a large median there is not a reliable representation effect.
At split2/1024 shared scratch, elided/stackful CPU per exchange improves in
every pass, median 0.9885 (0.9500..0.9943), while the rate range still crosses
parity. At split2/large private calloc, elided/manual p99 improves every pass,
0.9046 (0.5593..0.9972); stackful/manual also improves every p99 pair there,
so that observation does not uniquely select coroutine lowering.

WF small remains behind the compact private-calloc reference at split2/1024:
paired rate 0.9012 (0.8182..0.9614), CPU/exchange 1.0989
(1.0743..1.1908), and RSS 5.0607 (4.8944..5.4126). All rate and CPU pairs
lose. At split2/64-KiB transfers every rate pair loses, 0.8393
(0.8189..0.8854), and every p99 pair worsens, 1.2691 (1.2365..1.3650).
Split1 is different: WF small wins every 64-peer small-packet rate pair
(1.0113 median) and every large-packet rate pair (1.0604), without uniformly
winning tails. Existing compute/file controls reproduce the smaller runtime's
startup savings, roughly 11..18 ms for compute and 16..21 ms for file/compute;
they do not measure a coroutine WF backend.

The paced job `101575042677` completes 336 rows,
[artifact 9999215441](https://github.com/mbbill/Whitefoot/actions/runs/34066113672/artifacts/9999215441),
on a different host: Xeon 8573C, four logical CPUs on two SMT cores, Linux
6.17 and clang 20.1.2. All six cells contain eight forms and seven passes;
deadline counts, raw clients and the C LLVM controls independently check.
Its ratios are formed only within that job.

| Placement / compute rounds / light arrivals per peer/s | Native elided light p99 us | WF balanced chunks p99 us | Paired WF/native p99 | Paired WF/native heavy capacity |
| --- | ---: | ---: | ---: | ---: |
| split1 / 0 / 100 | 152 | 196 | 1.2973 (1.1316..1.4354) | 0.9647 (0.9534..0.9750) |
| split2 / 0 / 100 | 105 | 1885 | 19.1619 (3.6040..132.6667) | 0.8772 (0.8378..0.9482) |
| split1 / 2097152 / 100 | 847 | 1017 | 1.1797 (1.0841..1.2353) | 1.0000 (1.0000..1.0000) |
| split2 / 2097152 / 100 | 673 | 1371 | 2.0876 (1.6835..5.0129) | 0.9825 (0.9782..1.0044) |
| split1 / 2097152 / 500 | 844 | 924 | 1.0922 (1.0012..2.2655) | 0.9231 (0.9231..0.9846) |
| split2 / 2097152 / 500 | 647 | 1169 | 1.7761 (1.0970..6.1543) | 0.9903 (0.9784..1.0048) |

All six WF balanced/native light-tail comparisons lose every pass. The
zero-compute split2 rate ratio is 0.8791 (0.8403..0.9490) and CPU/exchange
is 1.1432 (1.0644..1.1900). For long compute at split1/500, total rate can
look better while every heavy-capacity pair loses; that remains a tradeoff,
not a throughput win. Unchunked WF still delays light requests by hundreds
of milliseconds under long compute. Among native manual, stackful and both
coroutine forms, every elided/manual and elided/stackful rate range crosses
parity; their long-compute deadline capacities are generally close. Removing
child allocations is not the missing mixed-load service policy.

The evidence supports compact sequential continuations as a memory candidate,
without selecting them as a universal speed winner or establishing a general
WF lowering. Keep allocation placement and CPU attribution separate from
the eventual compiler/ABI decision. No source coloring or container runtime
interface is changed by this reference comparison.

## Twenty-seventh experiment: allocation inside the sequential handler

The page experiment locates 46572 KiB of resident memory in the WF process's
ordinary heap at 1024 small-packet connections, unchanged by disabling THP.
The native per-worker calloc form instead touches much less private storage.
One concrete source difference is allocation placement: the WF caller creates
each 65536-byte initialized buffer and lends it to `serve_one`; the native
handler allocates on the accepting worker. This suggests an allocator-path
experiment, but the prior snapshots alone do not establish that mechanism.

The existing `tcp_echo_server.wf` is superseded in place: `serve_one` owns its
scratch buffer inside the Accepted arm and declares `allocates(heap)`. The
caller obtains the permit and invokes the same sequential connection loop.
The scratch parameter and its length precondition disappear because the
callee now constructs that exact initialized buffer itself. No proof check
or initialization is weakened. The redundant caller region is removed after
the compiler rejects it under FORM-8. The outer loop still emits
`wf__par_publish_staged`; the read-to-EOF and partial-send loops stay sequential.

`make scheduler-allocation` compiles the retained caller source from
`2de6c00039243aee98554eabba5143f011991461` with the current compiler and links
the current runtime. Forms base/small use caller ownership; callee/callee-small
use accepted-handler ownership. Both source files, emitted LLVM and optimized
LLVM are retained in codegen. This changes allocation timing, descriptor
placement and the private helper signature together, so a win would support
the writer form without by itself isolating a particular libc operation.
It creates no fixed-worker guarantee or container-facing runtime interface.

The Linux panel keeps split1/split2, both per-process THP policies, 64 and 1024
small-packet peers, and 64 large-packet peers. Four native references remain:
io_uring, shared-scratch epoll, private-calloc epoll and private-calloc stackful.
The eight forms produce 672 timed rows over seven passes after two warmups,
alternating form and cohort order. Three live snapshots per cell produce 288
smaps/status records under the preceding byte-checked resident protocol.
All native storage qualification and unchanged C LLVM checks remain enabled.
Sixteen additional WF backpressure/EOF stream cases qualify both ownership
forms, both runtime sizes, one/two workers and both page policies before any
timing. Pure-compute/file timing is not repeated: those sources and the runtime
are unchanged, and this panel isolates connection-buffer ownership. Clang 18
is used for every timed form, matching the page experiment; ratios are still
formed only within this new job, never across hosts.

On M1, a temporary before-accept allocation variant and the selected
after-accept form both compile and pass eight observed stream cases in total.
The maintained runner's actual WF build block then produces all four normal
binaries and four observer companions. All sixteen normal/observed stream
cases pass with one/four workers, including 2-MiB streams, backpressure,
half-close and process exit. Both optimized sources retain an initialized
65536-byte calloc. Runner Bash and workflow YAML/embedded shell checks pass.
Native Linux allocation results follow below.

### Native allocation results

Revision `8b44b5f9270a2bb322ae7c9dfe775f736730a7bc` completed the
[allocation panel](https://github.com/mbbill/Whitefoot/actions/runs/34066923286)
on AMD EPYC 7763, four logical CPUs on two SMT cores, Linux 6.17, Clang 18.1.3,
glibc 2.39 and 4096-byte pages. Artifact 9999604332 retains every sample and
live mapping. Independent analysis checks exactly 12 workload/policy cells,
eight forms and seven passes, all 672 raw client exchange counts and empty
diagnostics. All 288 live snapshots have three repetitions, matching smaps
sums and the requested read-back THP policy; disabled samples have zero
AnonHugePages. The canonical gate and host qualification pass. The separate
io-bench Windows performance job fails its existing compute stability rule
after two cohorts; this is not a successful Windows performance panel and
its threshold is unchanged. The subsequent d72d0d25 gate, hosts, scheduler
and io-bench workflows all pass.

Moving the buffer into the accepted handler improves two-worker residency,
but neither one-worker residency nor throughput generally improves. Live
RSS medians in KiB, with THP disabled, are:

| Placement and workload | Caller small | Callee small | Native stackful calloc |
| --- | ---: | ---: | ---: |
| split1, 64 peers, 64 bytes | 5912 | 5896 | 2524 |
| split1, 1024 peers, 64 bytes | 53360 | 53344 | 14152 |
| split2, 64 peers, 64 bytes | 6368 | 4128 | 2648 |
| split2, 1024 peers, 64 bytes | 53808 | 32904 | 14232 |
| split2, 64 peers, 65536 bytes | 7164 | 6996 | 4412 |

The callee-small split2/1024 snapshots range from 31620 to 36152 KiB, so
allocation distribution varies even within this cell. Its ordinary heap RSS
is 19508/21076/25012 KiB versus the caller's 46572 KiB in all three samples;
another anonymous mapping holds 3924..4896 KiB. At split1, both forms retain
exactly 46516 KiB in the ordinary heap. The WF entry thread executes worker
zero, while the native reference creates every worker with pthread_create.
Thus moving allocation into a handler need not move it off the process main
thread. These observations support an allocation-path hypothesis; they do
not trace a particular libc branch or prove that another allocator wins.

The split2/1024 paired callee-small/caller-small throughput median is 0.9966
(0.9822..1.0241) with THP permitted, and 0.9904 (0.9735..0.9998) when disabled.
The latter loses every pass despite lower memory. Against native stackful
calloc in that disabled cell, callee-small throughput is 0.8621
(0.8145..0.9231), CPU/exchange 1.0856 (1.0822..1.1781), and peak RSS 2.1217
(2.0295..2.1668): all remain worse. At split1/large transfers the disabled
callee-small form beats native throughput by 4.17% in the paired median,
but spends 36.11% more CPU/exchange; it is not a performance frontier win.
Live snapshots and process-lifetime peak RSS are different measurements.

Retain the initialized handler-owned source as an experimental ownership
form, with the caller control still reproducible. No buffer initialization
is removed and no writer-visible pinning, suspension annotation, container
ABI or lifetime exception is introduced. A temporary native main-worker
control has passed 20 M1 stream cases; Linux allocator attribution remains
a follow-up, not an explanation established by those local checks.

## Twenty-eighth experiment: CPU attribution of the mixed-load gap

The completed coroutine panel makes representation alone an insufficient
explanation for WF's remaining mixed-load loss. `make scheduler-profile`
therefore uses the same qualified coroutine-paced sources, compiler, normal
binaries, two CPU placements and three fixed-arrival cases, and profiles four
forms: WF base, WF balanced chunks, native C manual quantum and native C++
elided quantum. All preceding completion, protocol, observer, lifetime and
unchanged-C-LLVM qualification remains enabled. No compiler or runtime
interface changes in this experiment.

Three profiles per cell produce 72 profiled rows. They have no benchmark
warmup panel and are written as profile.tsv/profile-summary.txt, distinct
from the preceding unprofiled measurements. The client still checks every
byte, retains every scheduled light request and reports heavy completions
by deadline. Profile-induced rate or tail changes are not used to replace
those unprofiled performance conclusions.

[perf record](https://man7.org/linux/man-pages/man1/perf-record.1.html) samples
cpu-clock at 999 Hz, inheriting collection into the server's worker threads.
It covers the whole server lifetime, including startup and drain. No hardware
PMU event or stack-unwinding support is assumed. Flat sampled instruction
pointers can locate CPU-consuming functions but cannot measure off-CPU queue
waits or reconstruct costs of inlined callees independently. Raw perf.data,
per-sample text and the complete
[perf report](https://man7.org/linux/man-pages/man1/perf-report.1.html) histogram
are retained with each checked client sample, along with the exact four ELF
binaries in codegen for later instruction-level inspection. Unknown symbols and diagnostics
are retained rather than filtered out; empty sample output fails the run.
A temporary generated wrapper execs the normal server and separates its
strict stderr check from recorder diagnostics.

Only the dedicated GitHub-hosted profiling job temporarily sets
perf_event_paranoid=-1 and kptr_restrict=0 for event access and kernel symbol
lookup, records the original values, and restores them on shell exit. It
uses the installed generic perf executable directly because the Azure kernel
may have a different version from Ubuntu's generic tools. Page policy is
recorded without modification. M1 can check runner/workflow shell syntax but
cannot qualify Linux perf collection; the native results follow.

### Native CPU profiles and client-dispatch attribution

Revision `d72d0d253838c1e0134cb7f3f97ea681af105b7f` completed all four
workflows. The
[profile job](https://github.com/mbbill/Whitefoot/actions/runs/34067754736)
ran on AMD EPYC 9V74, four logical CPUs on two SMT cores, Linux 6.17,
Clang 20.1.2 and perf 6.8.12. Artifact 9999564655 holds 72 profiles.
Independent parsing accounts for all 110784 raw sample events, matches each
report's summed sample periods, verifies zero lost samples and checks raw
client counts against the profile table. It does not discard unknown symbols.

For zero computation, median CPU sample shares are:

| Placement and form | Kernel | Program | libc |
| --- | ---: | ---: | ---: |
| split1, WF base | 93.71% | 3.73% | 2.46% |
| split1, WF balanced chunks | 93.32% | 4.11% | 2.64% |
| split1, C manual quantum | 95.41% | 1.60% | 2.79% |
| split1, C++ elided quantum | 95.61% | 1.69% | 2.59% |
| split2, WF base | 90.61% | 5.48% | 4.16% |
| split2, WF balanced chunks | 90.75% | 5.75% | 3.37% |
| split2, C manual quantum | 94.87% | 1.61% | 3.56% |
| split2, C++ elided quantum | 93.49% | 2.77% | 4.06% |

Columns are independently formed medians over three profiles, so they need
not sum to 100%. Kernel execution dominates this sampled workload. WF's
userspace contribution is spread across submission, ring progress, joins
and scheduler completion rather than one dominant function. With long
computation and 100 light arrivals/peer/s, program code takes 93.83%/96.86%
of WF balanced samples on split1/split2, versus 94.74%/96.61% for native
elided quantum. The inlined recurrence dominates that function's samples.
These flat profiles do not measure off-CPU waiting or prove that a rarely
sampled checkpoint is free or sufficiently fair.

The resource wrapper times perf and its child, so profile.tsv user/system
resource columns include recorder/wrapper costs. They must not be reported
as isolated server CPU/exchange. The cpu-clock event stream follows the
target and inherited workers. Neither those samples nor this host's profiled
latencies replace experiment 26's unprofiled comparisons.

Re-examining that earlier unprofiled panel identifies an attribution limit.
At zero-compute split2, WF balanced's end-to-end light p99 median is 1885 us
versus native elided's 105 us. Its dispatch-wait p99 is 1560 versus 28 us;
its post-dispatch p99 is 186 versus 86 us. The latter paired ratio is still
2.1628 (1.1429..16.9091), but the full end-to-end ratio of 19.1619 cannot all
be assigned to execution inside the server. These are separately computed
quantiles: subtracting or adding their medians is not a p99 decomposition.
Post-dispatch latency includes client send/receive work and kernel time as
well as the server.

The client pump repeatedly exchanges on one heavy connection until EAGAIN.
Light arrival times are checked outside that pump. A continuously ready
heavy connection can therefore delay the client's next arrival check even
with disjoint client/server logical CPUs. The existing paced wait already
uses epoll_pwait2 with a timespec; millisecond timeout rounding is not this
mechanism. Server burstiness may affect the client's dispatch behavior, and
backlog can also reflect a previous request's slow response. A bounded
client control is needed before selecting another server policy from this
particular tail. Long-compute split2 post-dispatch tails also lose every
pair, with median ratios 1.9946/1.7876 at 100/500 light arrivals/peer/s;
the client observation does not erase the remaining performance problem.

## Twenty-ninth experiment: bounded client service

`make scheduler-client-service` crosses the unchanged five server forms
(WF base, chunks, balanced chunks, C manual quantum and C++ elided quantum)
with three client policies on split1/split2. The original client has service
budget zero; its optimized LLVM must match the retained d72d0d25 source
exactly after removing only module/source filename lines. The candidates
limit one pump to eight completed round trips or a single round trip, and queue its continuation
in an owner-local intrusive FIFO. Queued work is driven without requiring
a new edge-triggered kernel event. The client polls readiness and checks
light arrivals between finite FIFO groups of at most eight turns.

The policy neither deletes planned light arrivals nor resets their due
timestamps. The next heavy request's timer starts before its queue wait,
and all light backlog and drain remain in the latency measurements. Positive
budget builds report their budget and yield count in every raw client row.
No server binary, runtime budget, source ABI or container interface changes.
The five forms, six policy/placement cohorts and three cases yield 630
timed rows over seven passes after two warmups. Both form and cohort order
alternate. All prior native, WF completion and observer qualification stays
enabled. The uncapped comparison remains a full measured policy, rather
than being replaced silently.

Budget one forces the FIFO path during byte/compute/paced qualification.
The maintained client-service-check runs small and 64-KiB echo exchanges
against the native engine, verifies counts and requires positive yields
for budget one; Linux scheduler-check reaches it through the canonical
gate. The measurement runner additionally verifies computed and paced
budget-one exchanges at both server worker counts before timing. On M1,
32 native loopback cases pass across budgets one/eight, manual/stackful
references, one/four workers and small/large/compute/paced workloads.
Strict C11 builds and the default-client LLVM comparison pass locally.
The maintained verifier's actual exchange/assertion functions also pass
four local cases against the original native manual server. Budget one
reports 796 yields over 800 small exchanges and 76 over 80 large exchanges;
budget eight reports zero in those small cases. This is why the timed panel
includes both budgets: one stays a forced control if eight never exhausts.
These are correctness checks through the local epoll compatibility layer,
not Linux timings. The native client-policy panel remains pending CI.
