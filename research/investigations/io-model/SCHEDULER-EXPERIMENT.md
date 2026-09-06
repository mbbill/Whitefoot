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
