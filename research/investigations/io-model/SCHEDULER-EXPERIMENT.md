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
gate also checks the default policy. The gate runs its enumeration and
stream/continuation parts as separate jobs on Linux and macOS; production
default changes also reach the existing native
Windows checks. This does not qualify the experimental policies on Windows.

The two parts remain one local `scheduler-experiment` stage, reached by
`make check`. Root `scheduler-enumeration` runs the same three policies and
four configurations; `scheduler-streams` runs every existing native
continuation, stream/client and Go qualification. The benchmark's original
`scheduler-check` still runs its complete enumeration/native-check union.
The split addresses the Linux gate at `48bdbfea`,
[run 34115496718](https://github.com/mbbill/Whitefoot/actions/runs/34115496718):
although its individual steps report success, the platform annotation states
that the job exceeded eight minutes. The command spent 233.99 seconds in
enumeration including compilation and 196.68 seconds in later checks,
compared with 171.98 and 165.66 seconds at successful `cf998b56`. No directly
exercised test, compiler or gate input changed between these runs. Separate
jobs preserve the complete test union and the eight-minute ceiling; they do
not change the enumerator, its bounds or any acceptance/coverage assertion.
Their actual completion time requires a new CI observation.

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

### Direct LLVM pipeline feasibility

A separate M1/Clang 21 probe feeds the existing native coroutine servers
through an IR-only second compilation. The C++ frontend first emits LLVM
with `-O2 -S -emit-llvm -Xclang -disable-llvm-passes`, retaining
presplitcoroutine and coro.suspend. The ordinary C driver then consumes that
IR with `-O2 -x ir`, plus the existing C compatibility objects. This is a
toolchain probe using real echo/compute/quantum servers, not a new WF source
lowering. No repository runtime or source ABI is changed.

All three IR inputs split and link without a C++ runtime library; the
ordinary quantum binary's only dynamic dependency is libSystem. Ten stream
cases pass across one/four workers, echo backpressure and compute/quantum
normal/truncated requests. Allocation observations retain exactly one root
per connection, including the truncated case: child elision survives the
IR transport. Local observed root sizes are 176 bytes for echo and 256 for
quantum; these are not compared numerically to another target/toolchain's
frame layout. The existing suspended-nested-frame lifetime source also
passes through this path under ASan/UBSan: heap form allocates/frees 2048
frames, parent-contained form 1024, over 1024 destructions each. Sanitizer
attributes are emitted by the frontend before the IR-only compilation.

This establishes that the current host driver can lower coroutine IR
without introducing generated C++ source into WF's compilation path. It
does not establish a portable intrinsic dialect: local Clang 21 emits
`i1 @llvm.coro.end(ptr, i1, token)`, whereas the current upstream
[LLVM coroutine documentation](https://llvm.org/docs/Coroutines.html)
shows a void return. A future emitter must qualify the actual supported
toolchains. The documented presplit marker and coroutine intrinsics carry
compiler representation; they do not themselves require writer-visible
suspension syntax.

The unresolved work remains semantic and operational: generic WF CFG/call
lowering, stable frame-owned addresses, nested/recursive frame ownership,
completion-driven resumption, and draining kernel buffer loans before
destruction. The native readiness test has no outstanding kernel buffer
loan at EAGAIN and does not prove that completion cancellation is safe.
Keeping a parked root stack per connection merely to drive heap continuations
would retain the live stack-page cost, so that compatibility arrangement is
not selected as the intended memory representation.

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
not Linux timings. The native client-policy results follow.

### Native client-service results

Revision `f42ebdd8ca2425d9094a4e9afa9a6bb56fe3a30e` completed the
[native client panel](https://github.com/mbbill/Whitefoot/actions/runs/34069452611)
on AMD EPYC 9V74, four logical CPUs on two SMT cores, Linux 6.17 and Clang
20.1.2. Artifact 10000418135 retains all 630 rows. Independent analysis checks
18 placement/client/workload cells, all five forms and seven passes, every
raw client count, empty diagnostics, all 4800/24000 planned light requests,
and exact default LLVM identity for the client and three C server modes.
Budget-one yields equal heavy_count minus the 16 heavy peers in every row.
The canonical gate, host qualification and scheduler Windows checks pass.
The separate io-bench Windows job rejects its compute timings as unstable
after two complete cohorts; its Linux/macOS jobs pass. No stability threshold
is changed and that Windows performance table is not qualified.

Budget eight never exhausts in any of its 210 timed rows. Budget one yields
9473361 times over its 210 rows, so it is a real service-policy control.
It does not remove the split2 tail gap. Backlog-inclusive light p99 medians
in microseconds are:

| Placement / heavy steps / light arrivals per peer/s | WF balanced, original | WF balanced, budget 1 | Native elided, original | Native elided, budget 1 |
| --- | ---: | ---: | ---: | ---: |
| split1 / 0 / 100 | 292 | 191 | 252 | 168 |
| split1 / 2097152 / 100 | 1086 | 1093 | 917 | 916 |
| split1 / 2097152 / 500 | 1078 | 1125 | 995 | 995 |
| split2 / 0 / 100 | 1671 | 2048 | 181 | 162 |
| split2 / 2097152 / 100 | 1316 | 1270 | 690 | 673 |
| split2 / 2097152 / 500 | 1049 | 989 | 687 | 673 |

At split2/zero, WF balanced budget-one/original paired tail ratios range
0.4675..2.2673 with median 1.0694, and its rate ratio is 0.9875
(0.9682..1.0318). There is no consistent WF improvement. Against the same
budget-one native elided server, every WF rate pair loses (median 0.9426,
range 0.9288..0.9696) and every end-to-end tail pair loses (11.9627,
1.6319..20.5867). The original-client ratios are 0.9083 for rate and 9.2320
for tail. Their apparent convergence in throughput is partly a native loss:
native budget-one/original rate is 0.9661 (0.9438..0.9852), losing every
pass, with higher server and client CPU per exchange. It is not a WF speedup.

The zero-compute split2 dispatch-wait median remains 1472 us for WF balanced
versus 54 us for native under budget one. Post-dispatch medians are 175 and
125 us; their paired ratio is 1.6667 (1.0219..27.3455), still losing every
pass. Quantiles remain separate measurements, not additive components.
Restricting consecutive exchanges within a pump does not establish the
cause of dispatch waiting: serial per-peer request backlog, delayed client
service, scheduling and kernel execution are still different possibilities.

There are local improvements. Budget one lowers every split1/zero balanced
tail pair (median 0.6541) and native elided tail pair (0.6520), with rate
ranges crossing parity. Long-compute split2/100 still loses every balanced
tail pair to native, median 1.8277; heavy deadline capacity is essentially
unchanged, with one-second count granularity limiting small distinctions.
At split2/500 tail ranges cross parity under budget one, so the panel does
not support a universal tail ordering across all cells.

Keep the original client as the default. Retain budget one as a measured
control; budget eight's code-layout changes without an actual yield are
not evidence of service fairness. This rejects the proposed pump cap as
the explanation/fix for the large two-worker gap. It preserves experiment
28's attribution limit: backlog-inclusive latency still cannot all be
assigned to execution inside the server. The next same-host control changes
whether client and server share physical cores.

## Thirtieth experiment: main-thread allocation and heap padding

Experiment 27 leaves the one-worker ordinary heap unchanged after moving
buffer ownership into the handler, while two-worker heap residency falls
substantially. The WF entry thread executes worker zero. In the native
references every worker, including a one-worker server, is created with
pthread_create. This difference can select another libc arena even when
both handlers call calloc for the same initialized 65536-byte allocation.
It needs a controlled native comparison before changing WF initialization
or its container/storage contract.

`make scheduler-allocator` adds WF_BENCH_MAIN_WORKER=1 to the shared native
engine. All listeners still exist before any worker starts. Workers one
through N-1 are spawned normally, worker zero runs the same worker_main
on the process entry thread, and cleanup waits for every worker. The flag
defaults to zero. With zero, optimized C LLVM remains byte-identical to the
retained manual reference after removing only module/source filename lines,
for echo, computation and quantum modes. No connection state machine,
buffer size, ownership, initialization or send policy changes. Untimed
storage diagnostics identify the worker-start policy.

The second axis sets glibc.malloc.top_pad to 131072 or zero explicitly. The
[glibc 2.39 tunable documentation](https://sourceware.org/glibc/manual/2.39/html_node/Memory-Allocation-Tunables.html)
describes padding as extra heap growth and retained shrink space that saves
system calls; its default value is 131072 bytes. This is not the buffer size.
The [glibc implementation](https://raw.githubusercontent.com/bminor/glibc/glibc-2.39/malloc/malloc.c)
also disables dynamic threshold adjustment when setting top_pad, so both
padding policies use explicit settings. The 131072 control means the explicit default
numeric value, not an assertion of identity to an entirely unset allocator.
The experiment does not separately attribute dynamic threshold behavior.
The server binary's ELF interpreter is read from readelf, then its
[--list-tunables output](https://sourceware.org/glibc/manual/2.39/html_node/Tunables.html)
must report the requested value for each environment. Actual loader output
is retained. The timed load generator keeps its original environment;
untimed stream/residency launchers inherit the selected server setting.

Every server process disables THP through the existing checked launcher.
Split1 and split2 each have explicit-default and top0 cohorts. Eight forms
are measured: WF caller base, caller small, accepted-handler callee-small,
native shared-scratch epoll, private-calloc manual epoll on spawned/main
workers, and private-calloc stackful on spawned/main workers. All forms
use the same Clang 18 toolchain. Three echo cases (64/1024 peers with 64-byte
messages and 64 peers with 65536-byte messages), seven passes and two
warmups yield 672 timed rows. Form and cohort order alternate. Three
byte-checked live snapshots per cell yield 288 smaps/status records. The
original client is retained; fixed-arrival fairness is measured separately
by experiment 29. Pure compute/file timings are not repeated because this
panel attributes connection allocation, not a changed WF scheduler.

The existing native stream target retains its 32 cases and adds four
main-worker private-calloc cases: manual/stackful, each with one/four workers.
It verifies fragmented streams, short sends, backpressure, half-close,
cleanup and the observer's startup-policy value. All 36 pass locally on M1
through the existing epoll compatibility layer. The actual maintained WF
build block produces the three normal binaries and three observers with
the intended caller/callee source selection. All twelve WF stream cases
(three forms, normal/observed, one/four workers) pass locally. Native Linux
results follow below.

The decisive memory observation is the ordinary heap mapping while every
connection remains live, separately from total peak RSS. Moving worker zero
also changes pthread startup and stack reservation, so a rate difference
alone cannot identify calloc behavior. At one worker every accepted native
buffer necessarily uses that worker; at two workers listener distribution
can vary. Lower live memory does not establish a better performance frontier
unless CPU, throughput and tails support it. No runtime allocator default,
source initialization rule, container address guarantee or ABI is changed
by this experiment.

### Native allocator results

Revision 6ac4ebf4 completed
[the scheduler workflow](https://github.com/mbbill/Whitefoot/actions/runs/34070252329),
[the canonical gate](https://github.com/mbbill/Whitefoot/actions/runs/34070252296),
[host checks](https://github.com/mbbill/Whitefoot/actions/runs/34070252301) and
[the independent I/O benchmark workflow](https://github.com/mbbill/Whitefoot/actions/runs/34070252367).
Linux job 101586151119 ran on an AMD EPYC 7763 guest with four logical CPUs,
two reported SMT cores, Linux 6.17.0-1022-azure, Clang 18.1.3, glibc 2.39
and 4096-byte pages. Artifact 10000660487, `io-scheduler-allocator`, retains
the raw data. All 672 timed rows have their expected connection/trip byte
checks, seven passes per cell and empty normal-process diagnostics. All
288 live smaps sums match the table and report THP disabled, zero huge-page
residency and zero swap. Both loader top_pad readbacks match the requested
values. All three default native LLVM identities, 36 native stream cases,
12 WF allocation stream cases and 20 native storage-policy observers pass.

The large one-worker difference is reproduced by changing only which
native thread runs worker zero. The following are median live RSS values
in KiB with 1024 live connections exchanging 64-byte messages; they are
separate snapshots from timed maximum RSS.

| Form | Split1 top_pad=131072 | Split1 top_pad=0 | Split2 top_pad=131072 | Split2 top_pad=0 |
| --- | ---: | ---: | ---: | ---: |
| WF caller small | 53336 | 15052 | 53812 | 15480 |
| WF handler callee-small | 53320 | 15036 | 37108 | 15460 |
| Native manual calloc, spawned worker zero | 10040 | 9960 | 10112 | 9944 |
| Native manual calloc, main worker zero | 48064 | 9720 | 29136 | 9904 |
| Native stackful calloc, spawned worker zero | 14152 | 14068 | 14228 | 14060 |
| Native stackful calloc, main worker zero | 52160 | 13832 | 33640 | 14012 |

At split1, both WF buffer placements have an ordinary heap RSS of exactly
46516 KiB in all three explicit-default snapshots and 8212 KiB in all
three top0 snapshots. Native manual main-worker calloc changes from 46584
to 8240 KiB. Its total heap mapping size remains 65644 KiB under both
policies; the difference is physical residency, not logical allocation size.
The spawned native manual form keeps only 32 KiB in the ordinary heap and
most buffer residency in unnamed mappings. Native stackful main-worker
calloc similarly changes ordinary heap RSS from 46584 to 8256 KiB. This
reproduces the main-arena amplification without WF types, proof or task
lowering. Neither the initialized 65536-byte buffers nor the requirement to
preserve every returned byte was relaxed.

With two workers, handler allocation under explicit-default padding varies
with execution placement: callee-small live RSS spans 30672..43744 KiB,
with ordinary heap RSS 18356..34272 KiB. Under top0 its total live RSS spans
only 15448..15464 KiB even though ordinary-heap and unnamed-map shares still
vary. Caller small and callee-small then have almost equal residency. The
earlier caller/handler memory difference therefore does not require a new
container initialization or address contract to explain it. At 64 small
connections the same effect is smaller: split1 callee-small live RSS falls
5908 -> 3500 KiB, while main-worker native stackful falls 4668 -> 2264 KiB.

This is a memory explanation, not a recovered throughput frontier. All
ratios below pair the same pass; brackets contain the minimum and maximum
of seven ratios, not a confidence interval. CPU/exchange is the server's
reported user plus system time divided by verified round trips.

| Top0 comparison, 1024 small-message peers | Throughput ratio | CPU/exchange ratio | Timed peak RSS ratio |
| --- | --- | --- | --- |
| Split1 WF callee-small / native manual main | 0.9598 [0.9348, 0.9977] | 1.0723 [1.0405, 1.1024] | 1.5411 [1.5290, 1.5440] |
| Split1 WF callee-small / native stackful main | 0.9676 [0.9215, 0.9984] | 1.0476 [1.0286, 1.1024] | 1.0807 [1.0762, 1.0836] |
| Split2 WF callee-small / native manual main | 0.8490 [0.8090, 0.9155] | 1.1318 [1.1116, 1.1701] | 1.5637 [1.5482, 1.5863] |
| Split2 WF callee-small / native stackful main | 0.8561 [0.8184, 0.8957] | 1.1239 [1.1161, 1.1371] | 1.1054 [1.0978, 1.1214] |

Every rate and CPU pair in that table still loses. Tail ratios cross parity
in all four comparisons. The top0/explicit-default rate ranges also cross
parity for both compact WF forms at both worker counts; no repeatable rate
gain accompanies the large RSS reduction. The base WF split2/1024 form
actually loses every top0 rate pair, median 0.9856 [0.9574, 0.9978]. At 64
small-message peers, native rate variability prevents a consistent rate
ranking, while WF's split2 CPU cost still exceeds both native main-worker
forms in every pair.

Large-transfer timed peak RSS does not preserve the sparse live-snapshot
gain: split1 callee-small peaks at median 6916/6920 KiB under the two
padding policies. With top0, its rate exceeds native stackful main in every
split1 pair, median 1.0573 [1.0345, 1.0774], but CPU/exchange also exceeds it
in every pair, 1.3545 [1.3036, 1.3889], and tails cross parity. Split2 rate
ranges cross parity while WF CPU/exchange is still 1.5154
[1.3759, 1.6718] of that reference. These results cannot select a universal
winner from throughput alone.

Retain explicit top0 as an allocator control and the main-worker native
forms as an attribution control. Do not adopt a process-wide allocator
default from this panel: steady exchanges exclude connection establishment
from their rate interval, and other allocation sizes, churn and resource
pressure are unmeasured. The roughly 37.4-MiB one-worker heap reduction is
real and independently reproducible in native C. The remaining WF work is
continuation state and service cost, not removal of source initialization
or a claim that sequential source inherently requires the original RSS.

## Thirty-first experiment: separate client/server physical cores

The two-worker split2 cohort uses distinct logical CPUs, but on the measured
four-vCPU/two-core SMT hosts it puts a client and a server thread on each
physical core. The one-worker control uses different physical cores. The
client budget did not resolve the large difference, so `make scheduler-placement`
adds a same-host `separate2` cohort. It assigns the first core's two SMT
siblings to the two server workers and the second core's siblings to the
two client workers. Both roles retain two logical CPUs and two workers;
the runner verifies that this sibling topology exists before measuring.

The new panel retains split1 and split2, original and budget-one clients,
all five WF/native server forms, the three fixed-arrival cases, seven passes
and two warmups: 630 rows. Budget eight is omitted from this new comparison
because it never exhausted in experiment 29; its original experiment and
maintained qualification remain available. Form and cohort order alternate.
No server, compiler or runtime code changes for this placement control, and
all preceding completion, stream, coroutine lifetime, forced-client and
unchanged-default-LLVM checks remain enabled on Clang 20.

This changes where physical-core resources are shared, not how many workers
exist. It also changes each role from using two physical cores to sharing
one core internally. A result must therefore be reported as a placement
interaction, not proof of one kernel lock or of SMT being the sole cause.
Server rate, CPU/exchange, light dispatch/post-dispatch tails and heavy
deadline capacity are still required together. In particular, lower tails
bought by reducing the native reference's capacity do not establish a WF
improvement.

### Native placement results

Revision dce5ef95 completed
[the scheduler experiment](https://github.com/mbbill/Whitefoot/actions/runs/34071706036),
[the canonical gate](https://github.com/mbbill/Whitefoot/actions/runs/34071706024),
[host checks](https://github.com/mbbill/Whitefoot/actions/runs/34071706042) and
[the independent I/O benchmarks](https://github.com/mbbill/Whitefoot/actions/runs/34071706033).
Linux job 101590147030 used EPYC 7763, four logical CPUs/two reported SMT
cores, Linux 6.17.0-1022-azure and Clang 20.1.2. Artifact 10001161255,
`io-scheduler-coroutine-paced`, retains all 630 verified rows: 18 cells,
five forms, seven passes. Every planned light request, exact raw/deadline
count, budget-one yield count and empty normal-process diagnostic is checked.
All four unchanged-default LLVM comparisons pass. Split2 assigns server
0,2 / client 1,3; separate2 assigns server 0,1 / client 2,3. These are the
guest's reported core relationships, not verified host physical-CPU pinning.

Zero-compute p99 medians in microseconds show an interaction between client
policy and placement. They do not establish a placement-only cure.

| Placement / client budget | WF balanced light | Native elided light | WF dispatch | Native dispatch | WF post-dispatch | Native post-dispatch |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Split2 / original | 1790 | 172 | 1259 | 56 | 202 | 130 |
| Separate2 / original | 2037 | 177 | 1755 | 64 | 377 | 131 |
| Split2 / one | 2302 | 167 | 715 | 55 | 195 | 133 |
| Separate2 / one | 466 | 166 | 215 | 51 | 301 | 140 |

For the original client, separate2/split2 WF rate is 0.9943
[0.9366, 1.0277] and light tail 1.3571 [0.2251, 8.1459]. Neither improves
consistently. Native elided rate loses every placement pair, 0.9739
[0.9567, 0.9838], while CPU/exchange increases every pair, 1.0174
[1.0078, 1.0343]. A relative WF/native rate improvement here would partly
come from slowing the reference.

With the one-round client, separate2 improves every WF rate pair, 1.0242
[1.0022, 1.0633]. Its light-tail ratio is 0.1499 [0.0214, 1.1223], a large
median reduction with a range still crossing parity. Under separate2, WF
nevertheless loses every native-elided light-tail and post-dispatch pair:
2.7251 [2.0387, 8.4027] and 2.0680 [1.1286, 9.8490]. Rate is 0.9452
[0.9152, 1.2213] and CPU/exchange 1.1082 [0.9593, 1.1264]; one native
slow sample crosses those ranges. Separate quantiles cannot be added, and
post-dispatch still includes kernel and client handling, not just server code.

At 2097152 heavy steps and 100 light requests/second/peer, separate2 with
the one-round client gives WF/native light-tail ratio 1.8435
[1.0030, 2.3527] and post-dispatch ratio 1.9812 [1.0187, 2.3618], both
losing every pair. Heavy deadline capacity is 1.0000 [0.9647, 1.0021].
At 500 light requests/second/peer, the tail ranges cross parity, but WF
heavy capacity loses every pair, 0.9811 [0.9764, 0.9905], and CPU/exchange
also loses every pair, 1.0152 [1.0053, 1.0251]. Median heavy counts are
416/424. Thus moving the roles apart does not remove the mixed-load service
gap either. Client CPU/exchange falls about 10..20% with separate2 for
both WF and native long-compute cases, a measurable placement effect that
does not by itself identify the remaining server cost.

Keep separate2 and budget one as diagnostic controls. Reject physical-role
sharing as a sufficient explanation or standalone repair for the gap.
The original and forced-client cohorts must remain distinguishable, and
the relative amount of physical-core resource sharing is not an isolated
kernel-lock experiment. No scheduler or source default is selected here.

## Thirty-second experiment: combine measured dispatch and storage policies

`make scheduler-combined` combines existing policies after experiment 30
has separated the large calloc residency effect. It runs allocator echo
mode `combine`, using one unchanged accepted-handler WF module for all
four forms. `callee-small` retains the original global scheduling/ring
policy with compact stack metadata and configured-lane initialization.
`balanced` uses pinned owner rings and round-robin initial I/O dispatch
with the original storage policy. `balanced-small` adds both storage
optimizations to balanced; `quiet-small` additionally omits the running
owner's redundant self-wake. No completed-I/O service budget is added.
This separates the storage combination from the further wake change.

Every server has THP disabled and explicit glibc top_pad=0. The existing
two loader readbacks remain, but only zero is used in this timed panel.
This is a controlled server environment, not a proposed process-wide
default. Native controls are shared-scratch manual epoll, private-calloc
manual and stackful with worker zero on the main thread, and elided C++
coroutines with shared scratch or private calloc. The coroutine engine
retains its original spawned-worker startup. Both storage forms are kept
visible rather than assigning all native memory differences to continuations.

Split1 and split2 cross nine total forms with five echo cases: one, four,
64 and 1024 small-message peers, and 64 large-message peers. Seven passes
after two warmups give 630 rows. Underoccupied cases remain because the
dispatch and wake policies have different results there. Three snapshots
of all nine forms at 64/1024 small peers and 64 large peers give 162 live
smaps/status records. Form and cohort order alternate. Throughput, tail,
CPU/exchange and peak/live memory all remain separate selection grounds;
this echo panel cannot settle fixed-arrival mixed-load fairness.

Before timing, each combined policy runs the full existing completion
suite. The original small and balanced qualifications, all 36 native
stream cases, all 48 C++ stream cases, both sanitized coroutine-lifetime
checks and native LLVM identity checks remain enabled. Linux observations
require the exact memory/wake flags, evenly distributed initial calls,
actual owner rings and zero pinned resume migrations.

Locally, both new combined policies pass the complete M1 completion target,
including every existing schedule/configuration. The actual maintained WF
build block produces all four normal and four observed binaries; all 16
stream checks pass at one/four workers. All eight observers confirm the
selected storage/wake flags and pinned dispatch where requested. Executing
the actual configuration blocks against the captured guest topology produces
630 rows/162 snapshots for combine and preserves the prior allocator mode's
672 rows/288 snapshots. Native Linux results follow below.
This experiment changes no source signature, runtime ABI, container storage
contract or default runtime policy.

### Native combined results

Revision `5ca8a674`, scheduler run
[34073711463](https://github.com/mbbill/Whitefoot/actions/runs/34073711463),
finishes all sampling on an EPYC 9V74 guest with four logical CPUs, two
reported SMT cores, Linux 6.17 and clang 20.1.2. Its Linux job is **failed**:
the final summary still requested `base`, which the combined panel replaces
with `callee-small`. This is a reporting defect, not a missing timed sample.
The artifact contains all 630 rows in 90 complete seven-pass cells and all
162 live snapshots in 54 complete three-repetition cells. Raw client trip
counts and timing fields match every row; sample diagnostics are empty;
smaps sums match every snapshot with THP disabled and no huge pages or swap.
Both allocator readbacks and all three native LLVM identities agree.

The full four completion qualifications pass. Small retains its existing
17/17/20/19 enumerated schedules; the three owner policies run 18/18/21/20.
Both native-ring and fallback bridge routes pass, including the owner
four-thread bridge checks. All 36 C stream and 48 C++ stream cases pass,
as do both existing sanitized nested-destruction checks. All 16 WF
observations confirm the exact memory/wake flags; pinned forms distribute
initial calls evenly, use the requested owner rings and never migrate a
resumed stack. The same revision's native Windows scheduler check, gate
and io-hosts workflows pass. These results do not relabel the failed Linux
summary job as successful.

The summary now selects `callee-small` only for combine and retains `base`
elsewhere. Replaying its actual corrected block against the saved combined
and prior allocator data regenerates 90 and 96 groups; independently
computed same-pass ratios agree with every printed ratio. Removing one
paired control still fails. Measurements are not rerun or substituted to
repair this reporting error.

The useful combination survives. In split2, balanced-small versus
callee-small has rate ratios 1.1572 [1.1490, 1.1734] at 64 small-message
peers and 1.1315 [1.0976, 1.1472] at 1024; CPU/exchange improves every
pair, with medians 0.9574 and 0.9671. At 64 large-message peers rate also
improves every pair, 1.1515 [1.1277, 1.1716], but CPU worsens every pair,
1.1121 [1.1019, 1.1495]. Relative to balanced alone, adding compact metadata
and configured-lane initialization has throughput ranges crossing parity
in all ten cells. It therefore retains the measured dispatch gain without
establishing a further consistent throughput gain.

Live RSS medians in KiB, with identical explicit allocator/THP controls:

| Form | Split1, 64 small | Split2, 64 small | Split1, 1024 small | Split2, 1024 small |
| --- | ---: | ---: | ---: | ---: |
| Balanced | 27064 | 27204 | 34756 | 34892 |
| Balanced-small | 3528 | 3972 | 15064 | 15504 |
| Native manual, shared scratch | 1644 | 1652 | 1660 | 1676 |
| Native manual, main-worker calloc | 2008 | 2184 | 9724 | 9900 |
| Native stackful, main-worker calloc | 2260 | 2440 | 13832 | 14016 |
| Native elided, shared scratch | 3608 | 3628 | 3820 | 3840 |
| Native elided, private calloc | 4140 | 4192 | 12060 | 12096 |

Thus the combination removes most startup metadata residency while
preserving the separately attributed buffer-allocation improvement. The
remaining 1024-peer WF residency is about 11% above the native stackful
private-buffer control, 28% above elided private buffers, and much further
above the manual shared-scratch engine. Fixed process costs matter at
64 peers; this table cannot assign every difference to continuation frames.

For orientation, select the fastest native form by its cell's median rate
from the five controls, then retain that fixed form for all seven paired
ratios below. This is a descriptive comparison within this panel, not a
claim to have enumerated the fastest possible native implementation.

| Cohort/case | Fixed native reference | WF balanced-small/native rate | CPU/exchange |
| --- | --- | --- | --- |
| Split1, 64 small | Manual shared | 1.0137 [0.9945, 1.0155] | 1.0000 [1.0000, 1.0278] |
| Split1, 1024 small | Manual main calloc | 1.0107 [0.9937, 1.0392] | 1.0272 [0.9946, 1.0380] |
| Split2, 64 small | Elided private calloc | 1.0423 [0.9779, 1.3796] | 0.9927 [0.9571, 1.0385] |
| Split2, 1024 small | Elided shared | 0.9922 [0.9777, 1.0761] | 1.0586 [1.0304, 1.0682] |
| Split2, 64 large | Manual main calloc | 0.9536 [0.9351, 1.0118] | 1.1182 [1.1019, 1.1415] |

Higher-concurrency throughput is close to these native controls, with
crossing ranges, but CPU and memory still prevent a general win. Except
for split1/64-small versus manual shared (tail 0.9729 [0.9613, 0.9914]),
the table's p99 comparisons cross parity. Low-concurrency regressions remain
substantial: split2 balanced-small/callee-small rate loses every one-peer
pair (0.8655 median) and every four-peer pair (0.7338), while four-peer CPU
rises 1.4737 [1.4026, 1.4805]. The one-peer rate versus native elided private
is 0.6635 [0.6384, 0.7775], CPU 5.3529 [4.5500, 5.5625]. This is not a
throughput-optimal default across occupancies.

Quiet-small roughly halves split2 one-peer CPU relative to balanced-small
(0.5161 [0.5054, 0.5618]) but still uses 2.7059 [2.3000, 2.9375] times its
native control. Its rate improvement crosses parity there, and every
split2/1024 rate pair loses against balanced-small (0.9955 [0.9641, 0.9980]).
Keep wake omission as a control, not a universal addition. The surviving
result is that dispatch and storage optimizations compose; neither that
combination nor wake omission solves the low-occupancy, remaining CPU,
per-connection memory or previously measured mixed-load gaps.

## Thirty-third experiment: nested continuations retain real completion loans

The native readiness-coroutine lifetime check does not retain a buffer in a
pending kernel operation. `coroutine_completion.cpp` closes that specific
qualification gap using the existing WF receive-submit and join entries.
It is compiled once as C for the runtime seam and once as C++20 for nested
sequential coroutines. Only the bridge object's reference to
`wf_sched_complete` is redirected to a test coordinator, which still calls
the actual core publisher exactly once. Runtime sources, record layout,
generated-code ABI and container contracts are unchanged. The coordinator's
mutex and registration list are test instrumentation, not a proposed fast
scheduler. Replace this fixture with compiler-generated cases when general
continuation lowering can express the same protocol.

A parent owns an initialized 128-byte local buffer; a child receives into
that borrowed storage in a loop, using at most seven bytes per operation.
The parent's guard and byte-pattern checks require the buffer to remain at
one address until the child's last join returns. Four controlled orderings
cover immediate completion, completion before wait registration, publication
while registration is still returning, and completion after suspension.
The latter two include a logical early-exit request: the pending operation
must be joined before either coroutine frame is destroyed. This is draining,
not native cancellation, and requires the producer eventually to respond.
It does not solve cancellation of an uncooperative peer or destruction of
arbitrary suspended frames.

There are 512 individual cases and 16 groups of eight simultaneous roots.
All eight group producers remain gated until every root is suspended, so
the helper route has more outstanding loans than its four helper threads.
The roots must have distinct, stable buffer addresses. A single owning
thread resumes continuations; publication occurs on the real runtime's
completion threads. Under the coordinator lock, registration either observes
DONE and continues inline, or transfers ownership of its continuation node
to the pending/ready lists. Publication never accesses the completion record
after the real publisher stores DONE; dequeue removes its separate node
before allowing the frame to resume and die. The existing core's stackful
waiter handshake is not replaced or claimed to cover this new node.

`make completion-coroutine-check` runs heap and nested-frame-elided forms
under ASan/UBSan. The Linux canonical scheduler stage includes this check
alongside the existing C++ reference tests. Its toolchain has clang 20;
the macOS-14 canonical runner currently has Apple clang 15, so this
attribute-dependent fixture has the same Linux qualification boundary as
those reference tests. It also runs directly on the local Apple clang 21.
The dedicated Linux job requires at least 512 native-ring publications and
no helper publications, then explicitly forces the helper backend and
requires the inverse route counts. A fallback cannot qualify native I/O.
All previously maintained checks remain enabled.

Locally, both O2 forms pass all 640 cases, including 192 logical exits,
under ASan/UBSan and separately under ThreadSanitizer. Each run observes
384 registrations/dequeues, 128 pre-registration completions, 128
in-registration completions and 513 helper publications. Heap mode makes
1,280 frame allocations/frees; elided mode makes 640. Total requested frame
bytes are equal between forms: 307,200 under ASan/UBSan, 312,320 under TSan.
The allocation-count reduction is therefore not evidence of reduced total
frame bytes or a throughput improvement. Native Linux qualification follows
below. This fixture is lifetime evidence, not a WF stackless backend or a
performance result.

The first Linux qualification at `ea1d08be` reaches the final route guard
and fails it. Its bootstrap used pread on descriptor -1; the Linux native
adapter deliberately refuses negative transfer descriptors, so that one
initialization operation takes the helper route and violates the explicit
zero-helper requirement. The corrected bootstrap uses a checked byte read
from a real temporary file, which both adapters carry. It keeps the same
route requirements and reports both counters on failure. The old revision's
canonical Linux job separately confirms exactly 512 ring publications and
one helper publication; the specialized zero-helper job remains failed.

Revision `207b271e` passes the specialized native qualification in
[run 34075853738](https://github.com/mbbill/Whitefoot/actions/runs/34075853738),
job 101601679643. Under clang 20 and ASan/UBSan, both heap and elided forms
pass all 640 cases and 192 logical exits on each of the two actual backends.
Each native run has 513 ring publications and zero helper publications;
each forced-fallback run has 513 helper publications and zero ring
publications. Every run has 384 registrations/dequeues, 128 completions
before registration and 128 during registration. Native frame allocations
and frees are 1280/1280 for heap and 640/640 for elided, with 312320 total
requested bytes in either form. The workflow's Windows owner checks also
pass; they do not qualify this POSIX-only coroutine fixture on Windows.
The same corrected source passes both local sanitizer configurations again.

This closes the real-completion loan gap left by the readiness-only
coroutine probe: nested stackless frames can retain parent-owned addresses
through actual io_uring completion and scoped draining. It does not supply
a general WF coroutine ABI, a lock-free continuation-registration protocol,
cross-thread coroutine migration, native cancellation or a faster scheduler.

## Thirty-fourth experiment: compile sequential WF into continuations

The next representation probe is emitted by the real WF compiler. The
`--continuations --emit-llvm` mode uses the ordinary parsing, canonical,
semantic, proof and lowering path with overlap actualization off. Existing
derived `may_suspend` effects select coroutine representation across the
finite call graph, including recursive functions. It does not select by
function name or source shape, add a source annotation, or change the active
specification. Pure functions keep their ordinary representation.

`backend/emitter/continuation.rs` emits LLVM switched-resume intrinsics.
The existing target frame planner owns source storage, child result slots
and one opaque wait node per serial activation. A child receives its typed
arguments, a parent-owned result slot and a continuation handle. Symmetric
transfer enters the child and returns to its parent after source cleanup and
result publication. The parent destroys the completed child before reading
the result. LLVM may combine these nested frames under `coro_elide_safe`;
recursive activations can still require separate allocations. The cold
synchronous wrapper marks its root factory call `noinline`, preventing
root-frame elision into the host call stack. In one local optimized stdin
build this leaves a 1,296-byte heap root containing its elided child. This
is code-generation evidence, not a per-connection memory measurement.

Each mapped direct completion operation uses the existing submit, typed
outcome mapping and join/retirement implementation. A completed record takes
the ready path. Otherwise the generated body registers its wait node and
suspends; only completion publication makes it runnable. After resumption
the ordinary join retires the operation before source execution continues.
This first mode deliberately retains the serial source schedule, with no
independent sibling work issued across the await.

The experiment extends the preceding C publication coordinator, compiled
as C with `PROBE_WF_GENERATED`. Its host runs one root and has one progress
thread for actual completions. It neither depends on LLVM frame offsets nor
uses a C++ runtime. Its private interface is:

| Entry | Owner and meaning |
| --- | --- |
| `wf__continuation_record_done(record)` | C host; acquire observation of completion |
| `wf__continuation_prepare(waiter, record)` | C host; initialize an unregistered node |
| `wf__continuation_arm(waiter, frame)` | C host; register or observe completion, returning whether to suspend |
| `wf__continuation_run(frame)` | C host; drive this one root to completion |
| `wf__continuation_resume(frame)` | Generated LLVM; resume an opaque frame |
| `wf__continuation_finished(frame)` | Generated LLVM; test final suspension |

The node is 40 bytes aligned to 8 on the qualified targets, with matching
C static assertions. The coordinator lock retains the publication/register/
dequeue ownership protocol of experiment 33. It is a correctness host, not
a proposed low-overhead scheduler. It does not call the C++ fixture's file
bootstrap; the source program initializes the runtime through its own system
operations.

`make compiler-continuation-check` retains `completion-coroutine-check` as
a prerequisite and then compiles three WF programs. Existing
`completion_read_boundary.wf` covers successful file reads producing `AB`,
open failure (status 5), first-file EOF (status 8), and return before any
source I/O (status 1). Existing `stdin_echo.wf` and the new
`programs/continuation_stream.wf` each receive 0 and 65,673 bytes. The latter
reborrows its parent's 128-byte buffer through 17 recursive calls before
reading and publishing one chunk. The independent C oracle withholds input
until the root's initial resume has returned suspended, then checks every
output byte and the exact exit status. It also requires nonzero and equal
wait registrations/dequeues, with no unexpected diagnostics.

All of these maintained checks pass locally on Apple clang 21, with the
generated LLVM functions explicitly marked `sanitize_address` and the C
runtime compiled with ASan/UBSan. Passing `-fsanitize` to a clang LLVM-input
link alone does not instrument those generated functions, so the test adds
the LLVM function attribute before code generation. A separate local run of
the same maintained target also passes with ThreadSanitizer, including
`sanitize_thread` on the generated LLVM functions. The native Linux job
requires nonzero actual io_uring publications for both stream programs and
then repeats them with the helper backend explicitly forced. Those compiler
checks are wired into the Linux canonical scheduler stage; the independent
C++ fixture still requires its stricter all-ring or all-helper counters.
Revision `caa316ac` passes the native Linux qualification in
[run 34079265361](https://github.com/mbbill/Whitefoot/actions/runs/34079265361),
job 101611301155. Both generated stream programs pass their zero-byte and
65,673-byte cases on the native ring and forced helper backend. The plain
nonempty native run has 18 ring and 17 helper publications; the recursive
run has 515 ring and 514 helper publications. Those helpers are the existing
unpositioned `write_once` route, not silently substituted native reads.
The corresponding forced-helper runs have 35 and 1,029 helper publications
and no ring publications. All wait registration/dequeue counts match. The
independent nested C++ fixture also passes all four runs with the required
513-ring/zero-helper or 513-helper/zero-ring counts. The same revision's
canonical gate and host workflow pass, including the maintained Windows
runtime checks; the new continuation representation remains POSIX-only.

This has not integrated staged task publication, compute checkpoints,
asynchronous cleanup, Windows, concurrent roots, migration, or destruction
of arbitrary suspended frames. System operations without the existing direct
completion mapping and cleanup still use their qualified synchronous wrappers
and may block the host. The CLI therefore refuses native executable linking,
parallel/checkpoint modes and ledgers in this experiment instead of silently
claiming to provide them. There is no performance result for this WF mode.

The existing staged runtime ABI and container storage contract are unchanged:
acquire a lane, initialize arguments, publish, wait for join to return, read
the result, and release the lane. A task that retains a container or element
address requires the backing allocation to remain alive and address-stable
until that join returns. Copying a descriptor does not extend storage
lifetime, and observing DONE does not replace returning from join. No new
container pin, lease or release callback is introduced by this experimental
LLVM-to-host interface.

## Thirty-fifth experiment: network operations suspend the generated caller

A concurrent continuation executor cannot call the old synchronous
`tcp_accept` wrapper while other roots need its resumer. The network extension
therefore adds `tcp_listen`, `tcp_accept` and `tcp_connect` to the experimental
sequential await inventory. The ordinary hand-out inventory is unchanged:
the new selection authorizes no additional source overlap. All three use
the existing qualified runtime submits and typed outcome mappers. Accept's
three peer-address outputs are reserved by the normal target frame planner,
and the common retirement emitter now carries typed additional outputs for
both open outcomes and accept peer scalars. Its ordinary accept wrapper also
uses that retirement emitter, so the experiment does not maintain a second
error/resource mapping.

The existing `tcp_echo_server.wf` is compiled in continuation mode by
`compiler-continuation-check`. The native oracle withholds a connection until
the host has returned from resume with an accept node still pending; it then
withholds all input until the same observation witnesses a pending receive.
The host reads these nodes under the existing publication coordinator lock.
An old wrapper blocking the resumer cannot produce these witnesses. Empty
input and 65,673 pattern bytes exercise EOF, transfer and directional close
after the two gated suspensions. No timing threshold selects a pass.

The same target compiles existing `tcp_client.wf` and `tcp_refused.wf`.
A native peer checks the client's `ABCDEFGH`, withholds its reply until the
WF receive has suspended, then returns `abcdefgh` one byte at a time and
half-closes. The stdout byte oracle requires exactly that reply and status 0.
The refused program connects twice with the permit returned by the first
`ConnectionRefused`; both attempts must preserve the original status-0
expectation. A separate native listener retains the port during an occupied
listen check, which requires the echo server's original status 7 and no
stdout. Separate publication counters require each exercised connect, accept
and receive kind to use the selected native-ring or helper route, with zero
publications of that kind on the other route. One native receive therefore
cannot conceal a helper accept. Listen remains the existing helper operation
and is checked as such.

An initial test held a bound-but-not-listening socket to reserve the refusal
port. On this macOS host, the independent native connect reports
`ETIMEDOUT`, and the WF refusal fixture correctly returns 49 rather than
its required status 0. Using the released-port arrangement of the ordinary
network tests makes the independent native call report `ECONNREFUSED` and
both ordinary and continuation WF builds return 0. The oracle was corrected
to use that arrangement; neither the expected error class nor the permit
reuse requirement was relaxed. The occupied-listen case still retains its
actual listening socket throughout.

All new network cases and the existing file/pipe/recursive cases pass locally
with the generated LLVM functions and C runtime instrumented by ASan/UBSan
and separately by ThreadSanitizer. Eighty relevant ordinary backend tests
and eleven native network integration tests also pass. Revision
`d5037bfc191073fe49bc8ceb0625ad544b960e99` also passes the Linux native
[continuation qualification](https://github.com/mbbill/Whitefoot/actions/runs/34080707758/job/101615300619),
the [canonical gate](https://github.com/mbbill/Whitefoot/actions/runs/34080707729)
and [host qualification](https://github.com/mbbill/Whitefoot/actions/runs/34080707724).
The native TCP byte oracles pass at 0 and 65,673 bytes, with respectively 3
and 4 ring publications; the forced-helper runs have zero ring publications.
The native client and refused-connect cases each have 2 ring publications,
while their forced-helper counterparts have zero. In addition to these
aggregate counts, each exercised accept/connect/receive kind must satisfy
the oracle's selected-route requirement. Both occupied-listen routes report
one helper publication, as required by the existing listen implementation.
The earlier file, pipe, recursive-loan and independent C++ continuation
qualifications also pass in the same job. The host still owns one root.
Staged publication, compute checkpoints and asynchronous
cleanup are not yet integrated, so this is the network prerequisite for the
concurrent executor, not a concurrency or performance result. No source
signature, container layout or existing staged-runtime interface changes.

## Baseline matrix and evidence levels

This is the maintained comparison map for the I/O investigation. It supersedes
unqualified descriptions of the in-tree controls as the fastest possible
implementation; it does not change any recorded result above. A strong
reference is a measured Pareto candidate under a named workload, hardware and
resource budget. Beating the fastest measured candidate in one cell does not
establish that WF beats all implementations, or even that the server rather
than the load generator limits that cell.

Two comparisons answer different questions. The competitive comparison lets
each backend use its best qualified storage and scheduling policy within the
same resource envelope. The diagnostic comparison holds the engine, storage,
compiler or scheduler fixed while changing one factor. A 64 KiB io_uring
provided buffer and 64 KiB epoll worker scratch have the same transfer ceiling
but different ownership and residency. They are not matched storage. A native
control need not inherit a WF implementation limitation to be admitted.

Evidence levels are explicit: **audited** means source and configuration are
understood; **qualified** means the shared protocol oracle passes; **screened**
means same-host paired measurements exist; **confirmed** means tuning was
frozen before independent confirmation; **NIC-qualified** means a separate
machine and physical network repeat support the claim. Qualification and
screening apply to an exact revision/configuration, not the language or library
name. A known limitation remains visible until its experiment is complete.

| Implementation / backend | Source form, storage and scheduling | Role and expected source of performance | Evidence and remaining question |
| --- | --- | --- | --- |
| Native C epoll | Manual state machine; per-worker edge-triggered reactor and `SO_REUSEPORT`; 64 KiB shared scratch, bounded private spill on backpressure | Competitive readiness control: immediate recv/send, no ordinary per-operation allocation, local connection state | Screened on Linux loopback; 2 MiB streams, short sends and half-close qualified. Physical NIC and overload confirmation missing |
| Native C epoll with private storage | Arena, malloc or calloc per connection; main-thread worker variant | Diagnostic storage and allocator comparison; private backing remains owned through I/O | Screened and stream-qualified; these rows need not beat shared scratch to explain WF storage cost |
| Native C io_uring | Multishot accept/recv, provided buffers, per-worker rings/listeners, ordered vectored sends; SINGLE_ISSUER + DEFER_TASKRUN, no SQPOLL | Competitive completion control: batching, no receive submission per arrival, loaned receive buffers reused for send | Stream-qualified and screened at `475008b5`: 64 KiB improves both large-message cells; small-message intervals overlap. No independently confirmed or universal winner. Experiment 62 fixes a subsequently reproduced delayed-ENOBUFS lost rearm; corrected native qualification is pending, with earlier cohorts retained |
| Native C io_uring with immediate send | Same receive engine and loans, one nonblocking `sendmsg` attempt before ring fallback | Competitive hybrid candidate: avoids a submission/completion round trip when the socket accepts bytes immediately | Stream-qualified and screened at `0357259d`; 8 KiB hybrid severely regresses large messages; experiment 46 measures about 3.785x more send operations from lost application-level gathering, without short/EAGAIN retries. 64 KiB does not show the same loss. Shutdown wake-storage correction requalified at `0ebe924b`. The delayed-exhaustion correction in experiment 62 also applies to this path; corrected native qualification is pending |
| Native C stackful / C++ stackless | Same epoll engine; private or shared receive storage; stackful, heap coroutine, and compiler-elided coroutine forms | Diagnostic representation control: separates coroutine/frame allocation, storage and reactor cost | Screened and stream/lifetime-qualified at their recorded revisions; not independent mature runtime comparisons |
| WF stackful runtime | Sequential source, checked staged calls; shared or owner rings, source loans, compact stacks, dispatch/wake variants | Candidate language/runtime under test | Screened; candidate choices trade occupancy, CPU and throughput. No universal winning default selected |
| WF generated LLVM continuations | Sequential source, nested calls and recursion, completion-owned loans | Candidate to remove parked native-stack cost without signature coloring | Experiments 39/44/48 qualify the threaded, owner and batched-owner paths. At `fc69af15`, batching adds 28% / 27% paired throughput at 64 / 1024 small-message peers, with separate counters confirming aggregated ring submissions. Experiment 54 removes 97..98% of pending-list visits, with only 1.0%/1.8% median paired rate gains and reversals at 1024 peers; native CPU/trip still leads. Client headroom and multi-owner compute remain open. Experiments 52/54 qualify the unchanged sequential mixed protocol on both Linux completion routes |
| Go `net` | Goroutine per connection, sequential read/write loop; runtime netpoll and scheduler | External sequential-API baseline and runtime-preemption comparison | Go 1.27.1 release/race and both 64 KiB storage forms qualified (49/51). The fixed Linux screen (53) favors acceptor-heap/P1 for memory and tail latency within one server CPU; P4 oversubscription inflates p99. WF batch32 exceeds this Go candidate at 64/1024 small peers while native stays ahead. Large transfers approach the client ceiling; no universal optimum. Race moves both buffers to the heap |
| Rust sequential / Rayon CPU pool | Existing recursive `par_layout.wf` port with exact floating-point order and every node write; sibling `join`, calibrated grain and explicit pool width | Essential CPU-parallel reference; separates sequential code generation from parallel scheduling. Broader `par_iter`/`scope` and unbalanced workloads remain candidate rows | Checksum-qualified and independently confirmed after grain calibration on M1/Linux (experiment 40). Experiment 45's stack/batch control reduces the WF12/Rayon paired wall gap from 13.9% at one batch to 1.6% at sixteen, supporting substantial fixed costs. Experiments 47/50 find no stable gain from unused-ring or lane-initialization changes. Corrected traces locate early underutilization during computation; experiment 55 associates about 90% of WF runnable waits with runtime-issued yields. Experiment 57's complete pre-exit attribution places over 99.4% of early yield-associated off-CPU time in the idle scan; three post-exit probes remain explicitly unpaired. Experiment 59 finds no wall improvement from removing those rounds, with +1.24% short-task CPU. Experiment 61 also finds no useful gain from sequential tree building plus relocated lazy startup; decoded hot operations/constants agree despite relocated code/data. Retain both defaults; neither control measures ready-work availability or an OS scheduling cause. This is not an optimal WF CPU setup claim |
| Tokio I/O + bounded Rayon CPU offload | One current-thread I/O driver plus B-1 CPU workers, fixed 64-byte protocol, asynchronous bounded admission, one request/reply per connection | External mixed-load reference under one total execution budget; exposes CPU queue transfer, backpressure and light-request progress | Linux-qualified at `040bfc4b` (experiment 42), including saturation, errors, reset, partial input, half-close and slow output. Four short client smoke records are correctness evidence, not a performance ranking |
| Tokio | Fixed-worker multithread runtime and a separate per-core current-thread/reactor configuration | External mainstream async baseline; distinguish work stealing from reactor locality | Source candidate only; pin toolchain/lockfile, socket distribution and blocking-pool budget |
| Monoio | Per-core runtime, separately forced IoUringDriver and LegacyDriver | External completion/readiness comparison within one runtime family | Source candidate only; prohibit silent fusion fallback in backend-specific rows |
| Seastar | Per-core reactor and kernel TCP stack, explicit memory and polling settings | External high-performance locality and mixed-load scheduling baseline | Source candidate only; qualify chosen backend and reserve/count all CPU and memory resources |
| Native uring SQPOLL / fixed files / bundles / SEND_ZC | Separate explicit configurations, not silently enabled defaults | Later backend tuning envelope; upstream liburing proxy is implementation provenance, not an echo drop-in | Audited candidates, unmeasured here; SQPOLL CPU and zero-copy loan lifetime require separate qualification |

External architecture sources are [Go netpoll](https://go.dev/src/runtime/netpoll_epoll.go),
[Rayon](https://docs.rs/rayon/latest/rayon/) and its
[`join` contract](https://docs.rs/rayon/latest/rayon/fn.join.html),
[Tokio runtime](https://docs.rs/tokio/latest/tokio/runtime/),
[Monoio](https://github.com/monoio-rs/monoio), its
[legacy driver](https://github.com/monoio-rs/monoio/blob/master/docs/en/use-legacy-driver.md),
and the [Seastar tutorial](https://docs.seastar.io/master/tutorial.html).
Their reputation or published benchmarks select candidates for our matrix,
not winners of our workload. Each executable row must pin its actual source
revision, compiler and dependency lockfile.

| Workload class | Matrix axes and semantic equivalence | Current coverage / next discriminating evidence |
| --- | --- | --- |
| TCP closed-loop echo | 1/4/64/1024 peers × 64 B, 64 peers × 64 KiB; one outstanding request per peer; exact bytes and EOF | Current ten-cell split1/split2 screen. Add 4 KiB and pipeline depths 8/32 only after candidate screening |
| TCP streaming / backpressure | Continuous 2 MiB or larger, arbitrary fragmentation, short sends, slow readers, half-close; preserve order and bounded live storage | Existing epoll stream oracle; uring joins it below. Idle 10k peers, churn and reset/cancellation remain separate qualification |
| TCP fixed-arrival / mixed compute | Same recurrence and compute quantum; light paced requests alongside heavy work; below/near/above saturation | Existing paced mixed experiments cover selected controls. External candidates need scheduled-to-response p99/p99.9, goodput, missed deadlines, backlog and drain recovery |
| CPU-only parallelism and CPU offload | Sequential Rust vs Rayon; balanced/unbalanced recursive and data-parallel jobs; matched arithmetic, tuned grain and pool width. Mixed mode charges enqueue, completion transfer and bounded queues | Recursive `join`, stack/batch, unused-ring and initializer controls are measured; caller traces identify the idle-yield path, but the ordinary zero-yield control finds no improvement. Experiment 61 also finds no useful gain from the tree-build/lazy-start control. The 63-versus-15 layout fork difference remains; observed owner inline executions are not failed acquisitions. Ready-work availability and same-budget worker placement remain open. Tokio + Rayon mixed qualification exists, with timing pending client-capacity control. Both executors share one total budget |
| File reads | Open-once cache-hot vs cold buffered vs direct I/O; random/sequential; 4/64 KiB; QD 1/8/64; same offsets, bytes and checksum | Existing file experiments cover subsets. Extend native blocking/pread pool/uring comparisons; fio is a device-envelope cross-check, not an identical-program runtime row |
| File writes | Buffered accepted bytes vs fdatasync/fsync durability are distinct contracts; name batch size, flush cadence and directory durability | Broader matrix required; no current TCP result supports a write or durability claim |
| Dependent storage/network pipeline | Read → parse → request → write with the same dependency graph and compute work | Unmeasured; tests whether sequential-source overlap composes across stages |
| Pipes / child processes | Bounded pipes, producer/consumer backpressure, EOF and child exit ordering | Continuation correctness exists for pipes; matched throughput and helper-admission envelopes remain unmeasured |

File workload provenance is the [fio documentation](https://fio.readthedocs.io/en/latest/fio_doc.html).
Ephemeral CI storage and virtual block devices cannot certify a physical
NVMe-best claim. Linux epoll/io_uring loopback is the current network screen;
macOS helper qualification and local timings do not measure either Linux
backend. A native kqueue comparison on macOS and matched
[Windows IOCP](https://learn.microsoft.com/en-us/windows/win32/fileio/i-o-completion-ports)
comparison remain explicit platform rows, not inferred coverage.

| Resource / measurement axis | Required interpretation and current limitation |
| --- | --- |
| CPU placement | Record physical cores, SMT siblings, NUMA, cpuset and IRQ placement. Current split2 uses disjoint logical CPUs that may share physical cores; it is not a promise of two physical server cores. split1 separates physical cores |
| Thread and poll budgets | Count runtime workers, blocking helpers, io-wq and SQPOLL kernel threads. Process taskset and process CPU alone do not bound or account for a kernel polling thread. A competitive CPU budget is an upper bound: tuning may use fewer workers at low occupancy rather than forcing every contender to start all available workers |
| CPU cost | Existing `/usr/bin/time` `%U/%S` covers whole process lifetime with centisecond output: startup/drain and quantization matter in short cells. Separate steady-state CPU-ns/request, idle CPU and kernel CPU before fine low-load claims |
| Load generator | Occupied echo cells consume nearly all assigned client CPU, predominantly system time. Experiment 43's extra SMT worker does not establish spare capacity; experiment 58's readiness client remains opt-in after small-message regressions. Experiment 60 qualifies four distinct reported ARM cores: with the server fixed, two client workers improve paired median 64 KiB rates by 9–44% and reverse WF/epoll's same-host ranking, while 64-peer small messages regress with wider pools. This establishes client-worker/resource sensitivity, including changed connection partitioning and event batches, not an unrestricted server ceiling. Keep the default for small-message comparisons; verify capacity per workload and architecture before interpreting close rates |
| Latency / overload | Closed-loop echo p99 does not establish an overload SLO. Fixed-arrival latency begins at the intended send time and includes dispatch delay; report goodput, drops/deadlines, backlog and recovery |
| Memory | Record total reserved/provided bytes, live RSS/PSS and slope versus peers, socket/kernel memory, faults and allocations. Equal provided bytes does not imply equal total or resident memory. Keep THP and allocator readbacks with each panel |
| Mechanism evidence | Untimed observers: syscalls/submissions/CQEs, send/recv bytes, queue depth/exhaustion, context switches, task migration and frame allocations. An observer is not part of a timed binary |
| Statistical selection | Tune on calibration cells, freeze survivors and confirm independently. Keep paired samples and dispersion. Current seven alternating passes are screening evidence, not independent confirmation after tuning |

The near-term sequence is bounded: qualify the in-tree byte-stream controls,
run the ten-cell native screen, then add Go, Tokio and forced-backend Monoio to
that same screen. Rayon accompanies the CPU-only and bounded CPU-offload
comparisons, with a sequential Rust control. Surviving Pareto configurations advance to mixed load,
streaming/pipeline stress and real NIC measurements; Seastar is a targeted
mixed-load/locality control. This avoids an unbounded Cartesian product while
keeping omitted workloads and implementations visible.

## Thirty-sixth experiment: strengthen the native completion reference

The dedicated baseline branch starts at `d5037bfc`. It changes only native
references, their experiment harness/checks and this investigation; no compiler,
runtime ABI, specification or conformance rule changes. The recorded
`5ca8a6747ea7f26ce1700a62dd140e20778744bc` combined panel remains a historical
four-WF/five-native comparison. Its omission of uring isolated storage and
scheduler policies; it did not demonstrate epoll superiority.

Source audit found two concrete invariant defects in `uring_echo.c`: the fixed
64-buffer per-connection queue used payload size to bound TCP fragment count,
and `ring_enter` cleared pending submissions even when an interrupted, busy or
short enter had not consumed the published SQEs. The new queue uses one link
per provided buffer and keeps the send vector bounded at 64 entries. No node
is allocated per operation, and the total pending queue is bounded by actual
buffer ownership. Submission debt is reread from the shared SQ head after
enter. These are source-level defects; the earlier timing logs do not prove
that either failure occurred in an earlier measured sample.

Terminal buffer ownership follows CQE `F_BUFFER` even when no payload is
returned. A send failure aborts the measurement instead of closing and reusing
a descriptor that an outstanding multishot receive may still name. Ring
teardown precedes freeing its loan storage. This is an explicit fail-fast
error contract, not qualification of recoverable resets or cancellation.
A follow-up conservatively retains allocations and descriptors through process
exit when a run has failed: ring close alone is not used as evidence that a
native operation has stopped accessing its loan. Successful-run cleanup and
the running `475008b5` screening revision are unchanged.

`make -C research/experiments/io-completion-bench uring-check` adds the existing
four-peer, 2 MiB-per-peer backpressure/half-close oracle at one and four server
workers, for both buffer sizes. The server is built with ASan/UBSan, a small
send buffer, TCP_NODELAY readback and untimed configuration/byte counters.
The original epoll, stackful and C++ checks stay intact. Native execution is
Linux-only; local shell/YAML/diff checks cannot substitute for it.

At `475008b5`, `scheduler-native-baselines` selected a new panel with
`NATIVE_BASELINES=1`:
`callee-small` and `balanced-small` WF controls, the existing five competitive
manual/stackful/elided-C++ controls, and native uring at 8 and 64 KiB. Two CPU
cohorts × five echo cases × nine forms × seven passes give 630 timed records;
three repetitions of three live-storage cases give 162 snapshots. Two warm-up
passes, THP disabled, explicit glibc top_pad=0, byte checks and exit checks are
retained. This is a screening configuration, not a new production default.

The 64 KiB form divides the provided-buffer count by eight. For the previous
256..2048 buffers of 8 KiB, it uses 32..256 buffers of 64 KiB: both reserve
2..16 MiB of payload storage per worker, with the same 4096-entry submission
ring. Queue metadata and touched pages can differ and are not claimed equal.
The change isolates chunk size at a fixed payload-storage budget; it does not
prove either choice best. SQPOLL is excluded until its additional kernel CPU
has a controlled and accounted placement.

The [io_uring setup interface](https://man7.org/linux/man-pages/man2/io_uring_setup.2.html)
and [upstream liburing proxy](https://github.com/axboe/liburing/blob/master/examples/proxy.c)
identify later tuning candidates such as fixed descriptors, NO_SQARRAY and
receive/send bundles. They need explicit feature qualification and a new
paired result. SEND_ZC also changes when buffers may be recycled, and
[local-network deferred copies](https://www.kernel.org/doc/html/latest/networking/msg_zerocopy.html)
make a real NIC follow-up necessary before any zero-copy superiority claim.

Validation at `475008b519f3dd5b3df86d6113259ebe3e7a6c23`: all fourteen
canonical gate jobs passed. The
[Linux scheduler job](https://github.com/mbbill/Whitefoot/actions/runs/34082126596/job/101619282242)
passed ASan/UBSan `uring-check` in all four configurations, alongside every
previous maintained scheduler, stream and continuation check. Each uring case
received and sent exactly 8,388,608 bytes. The 8 KiB one-worker case reached
queue depth 152 and the four-worker case reached 248: the shared oracle
actually exercised queue depths exceeding the old 64-entry assumption. This
does not assert that an old recorded timing sample encountered that defect.

| Qualification configuration | Receive completions | Send completions | Buffer exhaustion events | Maximum per-connection queue |
| --- | ---: | ---: | ---: | ---: |
| 8 KiB, 1 worker | 1038 | 105 | 205 | 152 |
| 8 KiB, 4 workers, summed | 1045 | 65 | 56 | 248 |
| 64 KiB, 1 worker | 158 | 135 | 285 | 25 |
| 64 KiB, 4 workers, summed | 156 | 76 | 36 | 32 |

These counters describe a forced-small-send-buffer correctness fixture, not
a timing comparison. SO_REUSEPORT distributed four peers unevenly (one of
four workers remained idle in both shown cases), which is another reason to
observe actual placement instead of assuming per-worker load equality.
Both observed buffer variants also cross-compile to Linux-musl objects with
Zig 0.14 and strict C11 warnings; shell syntax, YAML, Make dry-run and diff
checks pass locally.

The separate Windows timing workflow at this revision did not qualify a
measurement: [job 101619282069](https://github.com/mbbill/Whitefoot/actions/runs/34082126614/job/101619282069)
stopped at `compute remained unstable after two complete cohorts`. Its Linux
and macOS timing jobs passed. This is distinct from the passing Windows owner
and completion correctness checks; no failed byte oracle or IOCP operation is
reported by this timing failure. Both complete 15-pair raw compute cohorts
remain in [artifact 10004121543](https://github.com/mbbill/Whitefoot/actions/runs/34082126614/artifacts/10004121543),
SHA256 `db0637fb5a6099d6fff0521cb699c61991a62b5aa7ddf4333e15adc29eed5e33`.

An independent branch revision `f0d633c1` encountered the same stability
failure in [job 101621961875](https://github.com/mbbill/Whitefoot/actions/runs/34083089083/job/101621961875).
Its [raw artifact 10004458999](https://github.com/mbbill/Whitefoot/actions/runs/34083089083/artifacts/10004458999)
has SHA256 `b2cc4db261b0d8f3e0b25bc122b7cb4d788789d59b05d72ddb3e6620cac0fb6d`.
Both report EPYC 7763, four logical processors, affinity `0xf`, four compute
batches per child and Windows image `win25-vs2026 20260824.214.3`; this does
not establish the same physical host. Recomputing each paired parallel/serial
wall ratio reproduces the refusal to publish a stable table:

| Revision, attempt | Median ratio | MAD / median | (p90 - p10) / median |
| --- | ---: | ---: | ---: |
| 475008b5, 1 | 0.2840 | 0.0372 | 0.1266 |
| 475008b5, 2 | 0.2859 | 0.0077 | 0.1940 |
| f0d633c1, 1 | 0.2935 | 0.0386 | 0.2536 |
| f0d633c1, 2 | 0.2874 | 0.0349 | 0.4570 |

Every cohort passes the 0.05 MAD limit but exceeds the 0.10 spread limit.
Serial median wall time stays near 4.74 s; parallel medians are 1.35--1.39 s,
with slow observations up to 1.82 s at `475008b5` and 2.09 s at `f0d633c1`.
Parallel process CPU remains approximately 5.0--5.25 s even in these slow
observations, while CPU/wall falls from roughly 3.8 to 2.5--2.8. This suggests
lost parallel overlap or pauses rather than a large increase in total work.
Aggregate process CPU cannot distinguish host descheduling from internal
load imbalance or parking; per-thread/off-CPU evidence or a controlled host
is needed before attributing the cause. The stability threshold is unchanged,
and these unqualified cohorts are not performance evidence for a Windows
continuation implementation.

### Native screen results at 475008b5

[Run 34082126598](https://github.com/mbbill/Whitefoot/actions/runs/34082126598)
completed successfully, including native measurement job `101619281846` and
the Windows owner checks. Its
[raw artifact](https://github.com/mbbill/Whitefoot/actions/runs/34082126598/artifacts/10004549653)
has SHA256 `9b8fa8b7da8184adb68ad340bb48ccf586fc2f55eeec902b712e288a159eec31`.
The host was an EPYC 7763 VM, Linux 6.17.0-1022-azure, Clang 20.1.2 and glibc
2.39, exposing two physical cores with two SMT threads each. split1 used
server CPU 0 and client CPU 2; split2 used server 0,2 and client 1,3, sharing
physical cores. These results do not reuse timing denominators from the
earlier EPYC 9V74 combined panel.

The artifact audit found all 630 raw rows, 90 groups with exactly passes 0..6,
and 162 live snapshots in 54 groups with repetitions 0..2. Raw client metrics,
trip/byte counts and process-resource files match each row; timed diagnostics
are empty. Every RSS/anonymous/huge/private-dirty/swap value was recomputed
from its smaps. THP was disabled, huge pages and swap were zero, and the actual
loader's top_pad readback was zero. These checks establish a complete screen,
not absence of all environmental bottlenecks.

The existing exchange-interval client CPU counters expose a material ceiling:
`(client_exchange_user_us + client_exchange_system_us) / exchange_us` is near
the full assigned client CPU budget in most occupied cells. The table uses
the same fixed highest-native-median selections as the rate table below;
values are CPU-seconds per exchange wall-second, median [minimum, maximum]
across seven samples, not percentages of the whole machine.

| Placement, peers × bytes | Native form | Client CPU / wall |
| --- | --- | --- |
| split1, 1 × 64 | epoll-calloc-main | 0.4820 [0.4794, 0.4865] |
| split1, 4 × 64 | uring | 0.9972 [0.9950, 0.9976] |
| split1, 64 × 64 | uring | 0.9999 [0.9999, 1.0000] |
| split1, 64 × 65536 | cpp-elide | 0.9998 [0.9991, 0.9999] |
| split1, 1024 × 64 | epoll-calloc-main | 0.9999 [0.9766, 0.9999] |
| split2, 1 × 64 | fiber-calloc-main | 0.5016 [0.4903, 0.5176] |
| split2, 4 × 64 | epoll | 1.3810 [1.0490, 1.8507] |
| split2, 64 × 64 | uring | 1.9811 [1.8815, 1.9945] |
| split2, 64 × 65536 | epoll | 1.9218 [1.7960, 1.9640] |
| split2, 1024 × 64 | cpp-elide | 1.9158 [1.8240, 1.9421] |

split1 assigns one client logical CPU; split2 assigns two and shares physical
cores with the server. These samples have little client headroom, so close
WF/native rates do not establish equal server saturation throughput. Client
CPU alone does not prove it is the only bottleneck. In the occupied split1
native selections, user CPU is only about 0.050--0.068 CPU and system CPU
0.932--0.950; the existing client already precomputes echo payloads before
connect, changes only the round byte, and verifies full responses with
`memcmp`. A claim that repeated payload construction or a byte-at-a-time
success-path verifier explains this ceiling would contradict the source.
Experiment43 tests more client execution resources on its own physical core
before attributing that ceiling to server code.

Ratios below pair the same pass and cell. Values are median [minimum, maximum]
across seven pairs; these ranges are not confidence intervals.

| 64 KiB buffer / 8 KiB buffer, at 64 peers × 64 KiB | Throughput ratio | Server CPU/trip ratio | p99 ratio |
| --- | --- | --- | --- |
| split1 | 1.1898 [1.0951, 1.2135] | 0.8372 [0.8136, 0.8913] | 0.5215 [0.3721, 0.8460] |
| split2 | 1.1841 [1.0328, 1.2708] | 0.9049 [0.8630, 0.9341] | 0.6059 [0.5468, 0.7842] |

The larger provided buffer improves throughput, CPU and p99 in every paired
large-message sample under both placements, at equal payload reservation.
All eight small-message throughput ranges cross 1.0, so this screen does not
select a faster small-message size. Reduced receive fragmentation is a
supported mechanism candidate, but the forced-backpressure qualification's
completion counts are not counts from these timed large-message samples.

For each cell, the next table selects the native form with the highest median
throughput from these seven candidates, then pairs both existing WF forms
against that fixed form. This is an in-sample screening selection. It is not a
claim that each winner is statistically separated from every close runner-up.
The final column is the `balanced-small` server CPU/trip ratio; the original
whole-process, centisecond-precision CPU-accounting limitations still apply.

| Placement, peers × bytes | Highest native median | Native k trips/s | `callee-small` / native rate | `balanced-small` / native rate | `balanced-small` / native CPU |
| --- | --- | ---: | --- | --- | --- |
| split1, 1 × 64 | epoll-calloc-main | 23.9 | 1.191 [1.079, 1.259] | 1.136 [1.076, 1.213] | 1.850 [1.750, 1.950] |
| split1, 4 × 64 | uring | 127.1 | 0.876 [0.863, 0.975] | 0.879 [0.861, 0.982] | 1.194 [1.161, 1.233] |
| split1, 64 × 64 | uring | 130.0 | 0.996 [0.987, 1.005] | 0.995 [0.987, 1.008] | 1.041 [1.010, 1.042] |
| split1, 64 × 65536 | cpp-elide | 31.0 | 0.700 [0.531, 1.053] | 0.695 [0.529, 1.046] | 1.656 [1.349, 1.910] |
| split1, 1024 × 64 | epoll-calloc-main | 126.7 | 0.976 [0.902, 0.983] | 0.985 [0.939, 0.994] | 1.049 [1.037, 1.085] |
| split2, 1 × 64 | fiber-calloc-main | 22.0 | 0.973 [0.869, 1.040] | 0.950 [0.773, 1.194] | 3.696 [3.391, 4.091] |
| split2, 4 × 64 | epoll | 86.6 | 0.877 [0.447, 1.060] | 0.673 [0.349, 0.825] | 2.230 [2.127, 3.163] |
| split2, 64 × 64 | uring | 186.7 | 0.840 [0.828, 0.905] | 0.982 [0.971, 1.051] | 1.085 [1.069, 1.102] |
| split2, 64 × 65536 | epoll | 36.1 | 0.823 [0.569, 0.887] | 0.790 [0.568, 0.922] | 1.696 [1.520, 1.868] |
| split2, 1024 × 64 | cpp-elide | 182.6 | 0.825 [0.813, 0.881] | 0.950 [0.867, 1.038] | 1.088 [1.054, 1.156] |

Uring has the highest native median in three small-message cells; epoll and
C++/stackful controls lead other cells. The split1 one-peer WF rate advantage
costs more CPU, while several occupied/large-message cells still have material
WF rate and CPU gaps. No representation or backend wins throughout this
matrix. In particular, treating the old epoll-only combined controls as the
complete native target would have omitted a relevant competitor.

Live RSS medians below retain three byte-checked snapshots per cell. Payload
reservation is equal between the two uring sizes, but the number of buffer
identities and touched pages differs. Native shared scratch also reserves a
per-descriptor spill arena whose pages need not be touched. The table measures
RSS, not equal virtual reservation or equal buffer-initialization semantics:
the WF source explicitly owns initialized 64 KiB buffers, while native uring
exposes only received prefixes from a shared malloc pool.

| Placement, peers × bytes | uring 8 KiB | uring 64 KiB | epoll shared | epoll calloc main | C++ elided calloc | WF balanced-small |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| split1, 64 × 64 | 2380 | 2244 | 1640 | 2008 | 4108 | 3524 |
| split1, 64 × 65536 | 4172 | 3304 | 1672 | 3828 | 5964 | 6092 |
| split1, 1024 × 64 | 7312 | 4192 | 1660 | 9720 | 12076 | 15060 |
| split2, 64 × 64 | 2800 | 2784 | 1660 | 2184 | 4192 | 3988 |
| split2, 64 × 65536 | 6524 | 4808 | 1724 | 4088 | 5956 | 6420 |
| split2, 1024 × 64 | 7780 | 5640 | 1672 | 9896 | 12060 | 15500 |

The next native ablation is the immediate-send policy below. Fixed submission
ring sizing, the minimum provided pool, polling/parking choices and real-NIC
behavior remain untuned dimensions. An initialized-prefix or target-owned
buffer loan API is a language-design hypothesis for explaining the storage
gap; this screen does not establish that memory safety itself imposes it.

## Thirty-seventh experiment: checked staged calls own independent continuations

Revision `767858bd` adds `--continuations --par --emit-llvm`. The existing
checked [PAR-3] issue/drain graph now publishes may-suspend user calls as
independent continuation roots. Source order still determines the drain and
result consumption; a completed later task retains its result and storage
until its own join. The serial continuation option remains available, and the
experimental `Staged` lowering policy leaves pure compute outlining off.
A recursive pure-compute control verifies that ordinary `--par` changes its
module while both continuation policies remain byte-identical to `Off`.
No source signature, proof rule or conformance expectation changes.

The issuer reserves one 32-byte task descriptor, a child-frame pointer and
the typed result per existing pipeline slot, using the normal target frame
planner. Published children do not carry the nested-call frame-elision hint:
successive loop iterations can remain live simultaneously. Nested calls
inside each task retain the existing symmetric-transfer and elision path.
Both activation representations now consume the same block emission path,
including issue-window initialization, carry storage and the exact drain.

The experimental C host has one owning resumer and a FIFO of newly published
roots. Each I/O waiter carries its owner task, making this experiment's
waiter 48 bytes rather than 40; the independent C++ fixture retains its
original layout. Native completion publication only makes a waiter ready.
After resume returns, the host checks the owning root, marks a completed task
done and publishes its issuer's join waiter. A finished child is destroyed
and its descriptor retired before the issuer loads the typed result. No
publisher retains a waiter node after the owner dequeues it. Registration,
dequeue, task completion and retirement counters must balance at host exit.
This is a test coordinator with a locked pending list, not the intended
performance scheduler.

The same experimental ABI supplies a bounded window, with a default of 64
and an explicit host-only override used by qualification. Zero IR bounds
mean no bound, as in `IrCompletionWindow`; source and compiler ceilings still
apply. An initial host implementation mistakenly treated the dynamic-span
sentinel zero as an empty range and returned a window of one. A live process
sample showed one pending receive and three idle helpers while the peer
waited for four suspended tasks. Correcting the host's interpretation made
the unchanged multi-batch protocol complete; no peer expectation or source
permission was relaxed.

`compiler-continuation-check` adds two unchanged source programs to its
instrumented builds:

- `tests/programs/tcp_fanout.wf` lends an iteration-owned scratch buffer to
  each of four staged callees. All four peers connect before any speaks,
  the host observes four pending receives, and the last connected peer must
  finish before the earlier three receive input.
- The research `tcp_echo_server.wf` repeats the same reverse-completion
  protocol over twelve connections with a four-task window, exercising
  three batches and reuse of the issuer's task/result slots.

Both cases pass on the local M1 helper route with generated LLVM functions
and runtime code instrumented by ASan/UBSan, and separately by ThreadSanitizer.
The first reports 4 tasks completed and retired; the second reports 12;
both report a peak of 4 retained child tasks. Every existing file, pipe,
recursive-loan, endpoint-outcome and independent C++ continuation case also
passes in those runs. The compiler library's 1,504 tests pass, and the
strengthened recursive pure-control test passes separately. Format, clippy
and diff checks pass. Revision `f0d633c1` passes the canonical gate
([run 34083089080](https://github.com/mbbill/Whitefoot/actions/runs/34083089080))
and actual Linux native-ring staged qualification
([job 101621961511](https://github.com/mbbill/Whitefoot/actions/runs/34083089056/job/101621961511)).
The borrowed-buffer case records 4 ring accepts and 4 ring receives; the
three-batch case records 12 ring accepts and 10 ring receives. Both record
zero accept/receive helpers, a peak of 4 tasks, and exact completion and
retirement totals of 4 and 12 respectively. The forced-helper repetitions
also pass, as do the retained independent C++ nested-loan cases. Immediate
receives need no pending native request, so receive counts need not equal
the number of peers. Windows checks for the unchanged stackful path and the
cross-platform host checks also pass; they do not qualify continuations on
Windows.

The separate Windows performance qualification at the same revision fails
its measurement-stability requirement
([job 101621961875](https://github.com/mbbill/Whitefoot/actions/runs/34083089083/job/101621961875)):
the compute pair remains unstable after both complete cohorts. The other
three `io-bench` jobs pass. This run supplies no qualified Windows timing
table and does not change the native continuation correctness result above.
Its stability threshold is retained.

The old `wf__par_*` runtime ABI and container storage contract are unchanged.
Only the experimental continuation-host interface adds publication, task
readiness/registration, retirement and window queries. Storage borrowed by a
child stays alive and address-stable through its join; a finished flag alone
does not permit reuse. The first test covers an issuer-owned buffer loan and
the second covers callee-owned buffers. Arbitrary suspended-frame destruction,
source cancellation, asynchronous cleanup, compute checkpoints, multiple
resumers and Windows are not qualified here. Other system wrappers and
cleanup can still block the owner. There is no performance result for this
concurrent WF representation yet.

## Thirty-eighth experiment: native completion receive with immediate send

The pure-ring native reference submits every send through io_uring. The WF
runtime already has an immediate nonblocking socket-transfer path, so pure
ring submission is not automatically the strongest competitive control.
`WF_BENCH_URING_INLINE_SEND=1` now selects one `sendmsg(MSG_DONTWAIT |
MSG_NOSIGNAL)` attempt per send arm while retaining multishot provided-buffer
receive. The default remains the pure-ring reference.

A successful syscall retires exactly the same FIFO byte prefix as a send CQE.
A short syscall rebuilds the remaining vector before submitting it, and EAGAIN
falls back to the existing ordered ring send. Buffers covered by a submitted
send remain owned until its completion; an appended receive cannot modify an
active vector. If an inline completion drains the last bytes after EOF, it
closes the connection immediately rather than waiting for a CQE that will
never exist. There is one attempt per arm, not an unbounded new userspace send
loop. Terminal failures retain native loan storage through process exit.

The existing `uring-check` now covers both send policies at both buffer sizes
and one/four workers. Along with exact bytes, ASan/UBSan, backpressure and
half-close, it asserts positive ring-send completions in the streaming case;
the hybrid forms must also report positive immediate-send bytes and attempts.
Pure-ring forms must report zero for those counters. `send_bytes` counts both
paths, while `sends` still counts successful ring-send CQEs.

`NATIVE_BASELINES=2` and the current `scheduler-native-baselines` target retain
all nine forms from experiment36 and add `uring-inline` and
`uring-64k-inline`. The same ten workload/placement cells, two warm-ups and
seven passes produce 770 timed records and 198 live snapshots. Qualification
runs before any timing. Buffer payload capacity remains matched between the
8/64 KiB variants; the pure-ring controls remain in the same-revision panel,
so no timing ratio depends on comparing different CI machines.

Strict Linux-musl cross-compilation passes for all eight combinations of
buffer size, send policy and observation. Shell/YAML parsing, Make dry-run and
diff checks pass locally. At `0357259d`, the
[Linux scheduler gate](https://github.com/mbbill/Whitefoot/actions/runs/34084400944/job/101625562596)
passed all eight native uring stream configurations under ASan/UBSan, along
with all existing scheduler, stream, coroutine and continuation checks.
Every uring case received and sent exactly 8,388,608 bytes. Hybrid counters
below sum four-worker rows where applicable and demonstrate actual immediate
and ring-fallback transfers under the shared small-send-buffer fixture:

| Hybrid buffer size, workers | Receive CQEs | Ring send CQEs | Inline attempts | Inline bytes | Maximum queue |
| --- | ---: | ---: | ---: | ---: | ---: |
| 8 KiB, 1 | 1039 | 95 | 131 | 905357 | 154 |
| 8 KiB, 4 | 1038 | 49 | 70 | 825485 | 240 |
| 64 KiB, 1 | 145 | 90 | 92 | 1704095 | 22 |
| 64 KiB, 4 | 152 | 32 | 35 | 645367 | 32 |

These are correctness-fixture counters, not hot-path timing measurements.
The successful-exit lifetime qualification remains limited as audited below.
The completed screen below preserves the earlier `475008b5` result rather
than combining denominators from different CI hosts.

The separate `0357259d` Windows timing
[job 101625562416](https://github.com/mbbill/Whitefoot/actions/runs/34084400899/job/101625562416)
again stopped at the compute stability gate, before publishing a qualified
table. Its [raw artifact 10005026530](https://github.com/mbbill/Whitefoot/actions/runs/34084400899/artifacts/10005026530)
has SHA256 `bb67959767c3d5b3d1db2f386ceb208dd1fd9200216240e547d3eb1e9aa64cea`.
The host is a Xeon Platinum 8370C, unlike the preceding EPYC Windows runs;
absolute timings therefore are not paired with those runs. The two attempt
median parallel/serial ratios are 0.3076 and 0.2925, MAD/median 0.0521 and
0.0078, and spread/median 0.4104 and 0.1489. The unchanged spread limit rejects
both cohorts. Windows owner checks passed, as did this revision's three
Linux/macOS io-bench jobs; the Windows timing failure is not represented as
a semantic failure or as an accepted performance result.

### Same-revision hybrid screen results at 0357259d

[Run 34084400987](https://github.com/mbbill/Whitefoot/actions/runs/34084400987)
and measurement job `101625562590` passed, as did all fourteen canonical gate
jobs and both io-hosts jobs. The
[raw artifact 10005481127](https://github.com/mbbill/Whitefoot/actions/runs/34084400987/artifacts/10005481127)
has SHA256 `0e02f6d15704b1dfd4098e00dafebf4ec56f545fcc3b2388fc8a49fc6ec9fae9`.
This host is EPYC 9V74, Linux 6.17.0-1022-azure, Clang 20.1.2, with two
physical cores and two SMT threads per core. split1 assigns server CPU 0 and
client CPU 2; split2 assigns server 0,2 and client 1,3. The preceding EPYC
7763 measurements are not paired with this host.

The raw audit verifies 770 rows in 110 complete seven-pass groups, 198 live
snapshots in complete three-repetition groups, every client byte/trip count,
client and server resource field, and empty timed diagnostics. Recomputed
smaps fields agree with every resident row; THP readback is disabled, huge
pages and swap are zero, and glibc top_pad readback is zero. The ratios below
pair the same pass/cell; brackets contain observed minimum and maximum,
not confidence intervals.

| Placement, peers × bytes | 8 KiB hybrid / pure rate | 8 KiB hybrid / pure CPU/trip | 64 KiB hybrid / pure rate | 64 KiB hybrid / pure CPU/trip |
| --- | --- | --- | --- | --- |
| split1, 1 × 64 | 1.0010 [0.9933, 1.0150] | 1.0000 [0.9375, 1.0000] | 0.9972 [0.9820, 1.0200] | 1.0000 [0.9375, 1.0000] |
| split1, 4 × 64 | 0.9913 [0.9506, 1.0079] | 1.0000 [1.0000, 1.0303] | 0.9966 [0.9898, 1.0687] | 1.0000 [0.9412, 1.0303] |
| split1, 64 × 64 | 0.9939 [0.9915, 1.0317] | 1.0189 [0.9908, 1.0189] | 0.9994 [0.9926, 1.0234] | 1.0093 [0.9907, 1.0189] |
| split1, 64 × 65536 | 0.4251 [0.4171, 0.4342] | 2.3529 [2.3193, 2.4174] | 0.9986 [0.9744, 1.0365] | 0.9800 [0.9505, 1.0100] |
| split1, 1024 × 64 | 1.0136 [0.9770, 1.1128] | 0.9891 [0.9297, 1.0263] | 1.0004 [0.9857, 1.0493] | 1.0107 [0.9730, 1.0273] |
| split2, 1 × 64 | 0.9957 [0.9318, 1.0823] | 0.9500 [0.8947, 1.1176] | 0.9805 [0.9456, 1.0373] | 1.0556 [1.0000, 1.0625] |
| split2, 4 × 64 | 0.9242 [0.6437, 1.2814] | 1.0208 [0.8909, 1.1739] | 0.9182 [0.5060, 1.2907] | 1.0909 [0.9434, 1.1739] |
| split2, 64 × 64 | 1.0218 [1.0041, 1.0280] | 1.0078 [0.9922, 1.0156] | 1.0240 [1.0141, 1.0493] | 1.0078 [0.9922, 1.0236] |
| split2, 64 × 65536 | 0.4604 [0.3927, 0.5094] | 2.2917 [2.2569, 2.3496] | 1.0125 [0.9204, 1.0390] | 0.9915 [0.9832, 1.0442] |
| split2, 1024 × 64 | 1.0389 [0.9981, 1.0914] | 1.0045 [0.9819, 1.0317] | 1.0152 [0.9669, 1.0992] | 1.0090 [0.9685, 1.0419] |

The 8 KiB hybrid loses more than half its large-message throughput and more
than doubles server CPU/trip in every pair under both placements. Its large
p99 ratios are 2.6075 [2.2807, 2.6778] at split1 and 2.4346 [2.1367, 2.8834]
at split2. In contrast, 64 KiB hybrid large-message rate and p99 ranges cross
1.0. Both sizes gain about 2% rate in all pairs at split2/64 small peers, but
their CPU and p99 ranges overlap the controls; other small-rate ranges cross
1.0. The default pure-ring policy is retained.

A source-supported mechanism hypothesis is lost aggregation: pure-ring send
remains in flight while later receive CQEs append buffers to its next vector,
whereas a successful immediate send drains each 8 KiB arrival before later
CQEs can accumulate. This could increase send operations and kernel TCP work.
The backpressure qualification above is a different workload and cannot
establish those operation counts here. The discriminating next observation
is receive/send/inline/EAGAIN and packet aggregation counts on this exact
64 KiB echo cell, before adding a deferred-batch send policy.

For the competitive view, each row fixes the native candidate with the
highest median rate among the nine controls. The same in-sample selection
and client-headroom limitations as experiment36 apply; this does not identify
a universally fastest backend or statistically separate every close form.

| Placement, peers × bytes | Highest native median | Native k trips/s | callee-small / native rate | balanced-small / native rate | balanced-small / native CPU/trip |
| --- | --- | ---: | --- | --- | --- |
| split1, 1 × 64 | epoll-calloc-main | 30.1 | 0.8406 [0.8223, 0.9069] | 0.8074 [0.7997, 0.8631] | 2.6250 [2.4706, 2.8000] |
| split1, 4 × 64 | fiber-calloc-main | 112.9 | 0.9596 [0.9109, 0.9679] | 0.9591 [0.9171, 0.9682] | 1.1143 [1.0571, 1.1176] |
| split1, 64 × 64 | uring | 117.1 | 1.0038 [1.0020, 1.0315] | 1.0039 [0.9977, 1.0281] | 1.0377 [1.0183, 1.0377] |
| split1, 64 × 65536 | epoll | 30.6 | 0.9670 [0.9053, 1.0344] | 0.9846 [0.9349, 1.0829] | 1.0594 [0.9596, 1.1111] |
| split1, 1024 × 64 | epoll | 107.5 | 0.9978 [0.9194, 1.0315] | 0.9996 [0.8867, 1.0305] | 1.0513 [1.0213, 1.1746] |
| split2, 1 × 64 | cpp-elide | 28.5 | 0.7761 [0.7194, 0.8324] | 0.7167 [0.6946, 0.7424] | 5.0556 [4.5500, 5.3529] |
| split2, 4 × 64 | epoll-calloc-main | 131.4 | 0.7351 [0.7245, 1.0054] | 0.5464 [0.5262, 0.7533] | 2.1961 [1.8966, 2.2353] |
| split2, 64 × 64 | uring-inline | 185.0 | 0.8804 [0.8746, 0.9126] | 1.0297 [1.0167, 1.0335] | 1.0465 [1.0308, 1.0547] |
| split2, 64 × 65536 | cpp-elide | 52.9 | 0.8319 [0.8243, 0.8798] | 0.9572 [0.9258, 0.9804] | 1.1667 [1.1261, 1.1835] |
| split2, 1024 × 64 | epoll | 175.5 | 0.8597 [0.7896, 0.9869] | 0.9488 [0.9154, 1.1096] | 1.0822 [1.0667, 1.1467] |

The source/storage distinction remains visible in live RSS medians (KiB,
three byte-checked snapshots). Immediate sending makes little difference to
these allocations; 64 KiB provided buffers reduce touched storage relative
to 8 KiB at the same reserved payload capacity. These are not equal
initialization contracts or equal virtual reservation across WF and native.

| Placement, peers × bytes | uring | uring-inline | uring-64k | uring-64k-inline | epoll | balanced-small |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| split1, 64 × 64 | 2376 | 2380 | 2248 | 2248 | 1640 | 3524 |
| split1, 64 × 65536 | 4172 | 4176 | 3252 | 3236 | 1700 | 5440 |
| split1, 1024 × 64 | 7312 | 7308 | 4192 | 4188 | 1660 | 15060 |
| split2, 64 × 64 | 2800 | 2800 | 2784 | 2792 | 1656 | 3984 |
| split2, 64 × 65536 | 6392 | 6396 | 4760 | 4700 | 1748 | 6392 |
| split2, 1024 × 64 | 7784 | 7780 | 5636 | 5636 | 1672 | 15548 |

### Successful-exit loan audit after the 0357259d screen was started

The running `0357259d` tree is left unchanged, but a source audit identifies
one remaining shutdown loan without an explicit completion proof: a worker
can observe global `finished` and exit before consuming its eventfd READ
CQE. The successful cleanup then frees `workers`, including the operation's
`wake_storage`. Ring close is not used as a cancellation/drain proof here,
and ASan cannot establish the lifetime of a later kernel write. Consequently,
the earlier stream oracle and screen qualify their observed protocol results,
not this complete shutdown lifetime.

The follow-up retains the worker-record allocation until process exit on
success, as failed runs already retain all loan storage. It adds no shutdown
scheduler and does not change the send/receive hot path. Other cleanup relies
on the following operation-specific boundaries in the qualified, clean
half-close protocol; arbitrary reset recovery remains outside this evidence:

| Allocation or resource | Successful-exit lifetime evidence |
| --- | --- |
| `workers`, containing eventfd `wake_storage` | Retained until process exit because a wake READ may remain pending when the worker exits |
| Connection table, embedded `msghdr` and send iovecs | Exactly one send can be in flight per connection; close waits for its CQE and any queued suffix, or follows a synchronous inline send. All accepted connections must close before successful cleanup |
| Provided payload buffers | Each send retires only its completed byte prefix. Clean EOF terminates multishot receive; close follows the last pending send, so successful cleanup has no active network buffer loan |
| Provided-buffer ring | No receive remains after all clean connection closes; the ring is closed before the userspace buffer-ring mapping is released |
| `loans` and `starved_list` | Userspace-only metadata, never passed as a kernel operation address; accessed only by joined worker threads |
| Submission/completion mappings | Kernel ring-owned mappings are unmapped by ring teardown; their syscall-managed backing lifetime is distinct from caller-owned operation buffers |
| Listening sockets | Remaining multishot accepts use null address/length pointers, so they do not borrow freed caller storage; no extra client connections are part of this fixed-count protocol |

This narrow lifetime correction does not retroactively turn an observed byte
failure into a pass or claim that the timing screen witnessed a use-after-free.
At `0ebe924b`, the existing full
[Linux scheduler qualification](https://github.com/mbbill/Whitefoot/actions/runs/34085705091/job/101629217631)
passes with this correction, including all eight ASan/UBSan uring stream
configurations. That separate branch leaves the earlier running screen
unchanged.

A separate source audit of the optional, unmeasured SQPOLL path found a
missing store-to-load barrier between publishing the SQ tail and reading
`IORING_SQ_NEED_WAKEUP`. The release store and acquire load alone do not
provide that ordering. The follow-up inserts a sequentially consistent fence,
matching the full-barrier protocol in
[upstream liburing `sq_ring_needs_enter`](https://github.com/axboe/liburing/blob/master/src/queue.c).
This is a source-protocol correction, not a reproduced SQPOLL stall or native
qualification. The current screen never enables SQPOLL; its kernel-thread
CPU placement/accounting and dedicated runtime qualification remain pending.

## Thirty-ninth experiment: measure generated staged WF against the native panel

The first continuation performance screen reuses the exact sequential
`tcp_echo_server.wf` source and the experiment 37 publication/join lowering.
`compiler-continuation-bench` builds the generated module at `-O2`, without
sanitizers, and links the existing experimental continuation coordinator.
Its compact scheduler metadata, configured-lane initialization and TCP_NODELAY
flags match the small stackful control. The timed and observed continuation
form is the same counter-enabled binary; observation only enables reports.
The unoptimized and optimized LLVM modules accompany the raw artifacts.

`CONTINUATION_SCREEN=1 NATIVE_BASELINES=1 EXPERIMENT=allocator` selects this
panel through `scheduler-bench.sh combine`, also wired to the
`codex/io-continuation-screen` CI branch. It includes `callee-small`,
`balanced-small`, generated `wf-coro`, and the seven native controls from
experiment 36. Only the split1 cohort runs: the server, including every
helper and progress thread, shares one logical CPU; the client uses a CPU
on a different physical core. The current continuation host has one resumer,
so a two-worker row would not compare the intended execution configuration.
The continuation entry is mechanically carried through the harness's
alternative-executor list but is always a WF candidate, never a native
frontier control.

Five existing echo cases cover 1, 4, 64 and 1024 small-message peers and
64 peers exchanging 64 KiB messages. Seven alternating passes after two
warmups yield 350 measured rows. Three live snapshots for each of the three
resident cases yield 90 records. Every form uses THP disabled and explicit
glibc `top_pad=0`, with the existing readback and raw resource accounting.
Throughput, latency, CPU and resident memory remain separate outcomes.

The host uses an explicit 1024-task window in this panel. Every connection
handler lives until peer EOF, and the client holds its peers until all
finish exchanging. A window below the fixed cohort's connection count would
admit only a prefix whose peers wait for the remainder, preventing progress.
This is a benchmark admission requirement, not a source-language guarantee
for arbitrary protocols. The runtime default stays 64 and the compiler's
current experiment ceiling stays 1024. The four-slot, twelve-task gated
oracle continues to test actual slot reuse before any timing.

Linux timing requires the full ASan/UBSan native/helper continuation suite,
the same-flags uninstrumented staged oracle, and the common four-peer 2 MiB
stream/backpressure/half-close oracle. Untimed 4/64-peer observations require
native accept and receive activity with no helpers for those operations,
balanced registration/dequeue, and exactly one completion and retirement
per task. Timed runs emit no reports. Existing reference qualifications and
legacy scheduler assertions remain intact.

The local M1 build, uninstrumented multi-batch helper oracle, and common
four-peer 2 MiB stream oracle pass. The large-stream run retires all four
tasks and balances 36 registrations/dequeues on the helper route.
The Apple Clang 21 optimized module requests a 42,552-byte issuer frame and
a 1,784-byte frame per connection, plus a separate 65,536-byte `calloc`
receive buffer per accepted connection. The nested send continuation is
embedded in the connection frame. These are allocation requests from the
emitted code, not resident-memory readings. The Linux Clang 20.1.2 optimized
module retains these same three allocation sizes. The host still has locked
pending-list lookup, a separate progress thread and synchronous cleanup;
the screen measures that implementation and must not be read as a limit
on the language or continuation representation. Its purpose is to locate
the next measured cost while comparing actual compiler-generated sequential
WF with stronger native references on the same host.

### Linux confirmation at `2147857e`

[Run 34084346275](https://github.com/mbbill/Whitefoot/actions/runs/34084346275)
completed successfully, including measurement job 101625413250, native
continuation job 101625413218 and Windows placement job 101625412997.
Artifact `io-scheduler-allocator` 10005292466 has ZIP SHA256
`f3fe8dada054e37d21847b717153e9ce4d6c94252262c57b16fa652150277ecb`.
It retains all 350 timing rows, 90 live memory snapshots, separate observer
runs, emitted modules and qualification logs. Every timing row was checked
against its original client and process-resource files: exact trip counts,
empty timed diagnostics and stdout, matching rate/latency/CPU fields. All
90 memory rows match the summed original `smaps`; every process status
confirms THP disabled, with zero huge anonymous pages and swap. No row was
removed. Printed exchange microseconds truncate the higher-resolution time
used for rate; recomputing rate allows that one-microsecond precision loss.

The host was AMD EPYC 7763, Linux 6.17.0-1022-azure, glibc 2.39 and Clang
20.1.2, with two physical cores and two SMT siblings per core exposed. The
entire server process used CPU 0; the single client thread used CPU 2 on
the other physical core. These are hosted, nonexclusive resources.

| Peers × payload | Callee WF rt/s | Balanced WF rt/s | Generated continuation rt/s | Continuation / callee rate, paired median [min, max] | Continuation / callee CPU/trip, paired median [min, max] |
| --- | ---: | ---: | ---: | --- | --- |
| 1 × 64 B | 29,110 | 27,913 | 22,131 | 0.760 [0.686, 0.805] | 1.143 [1.086, 1.257] |
| 4 × 64 B | 111,530 | 112,406 | 38,752 | 0.348 [0.314, 0.445] | 2.833 [2.239, 3.152] |
| 64 × 64 B | 129,216 | 129,311 | 48,247 | 0.375 [0.371, 0.385] | 2.634 [2.573, 2.650] |
| 1024 × 64 B | 122,117 | 121,684 | 32,357 | 0.276 [0.260, 0.285] | 3.494 [3.386, 3.678] |
| 64 × 64 KiB | 21,824 | 21,800 | 21,071 | 0.967 [0.957, 0.976] | 1.034 [1.020, 1.041] |

Rates are medians of seven passes. Ratios pair the same pass, rather than
dividing independently selected samples; brackets are observed ranges,
not confidence intervals. Server CPU is whole-process user plus system
time, recorded to 0.01 seconds, divided by completed trips.

| Peers × payload | Highest native median in this cohort | Native rt/s | Continuation / native rate, paired median [min, max] | Continuation / native CPU/trip, paired median [min, max] |
| --- | --- | ---: | --- | --- |
| 1 × 64 B | `fiber-calloc-main` | 23,938 | 0.922 [0.851, 0.972] | 2.000 [1.900, 2.200] |
| 4 × 64 B | `uring-64k` | 127,433 | 0.307 [0.289, 0.383] | 3.367 [2.861, 3.586] |
| 64 × 64 B | `uring-64k` | 130,479 | 0.371 [0.369, 0.376] | 2.732 [2.680, 2.771] |
| 1024 × 64 B | `epoll-calloc-main` | 127,325 | 0.259 [0.251, 0.277] | 3.812 [3.574, 3.938] |
| 64 × 64 KiB | `cpp-elide` | 34,222 | 0.616 [0.527, 0.976] | 1.798 [1.452, 1.913] |

This native selection describes the measured cohort; it is not an
independently calibrated and confirmed winner. In particular, the large
`cpp-elide` cell ranges from 21,680 to 39,839 rt/s. Its median cannot stand
for a stable universal target. Native shutdown loan qualifications also
retain the experiment 38 limitation: revision 214 predates the correction
that retains an outstanding eventfd READ's worker storage until process exit.

The useful diagnostic is the generated host's process scheduling cost.
Its voluntary plus involuntary switches per trip are 7.111, 3.745, 2.614,
3.750 and 3.152 in the table's case order; callee WF records 0.0251, 0.0101,
0.0013, 0.0012 and 0.0315. At 64 small-message peers, continuation CPU/trip
is 20.703 microseconds versus callee WF's 7.812, and median p99 is 2,720
versus 543 microseconds. At 1024 peers, p99 is 58,685 versus 9,708
microseconds. These counters locate a costly implementation path, but
cannot alone separate futex wakeups, ring submission, list lookup and other
kernel work. The next discriminating change is to drive target progress on
the sole resumer, removing the background-progress handoff while retaining
the source, frame ABI, publication lifetime and remaining coordinator logic.

Client exchange CPU divided by exchange wall time is about 0.998–1.000
for the selected native forms at 4/64/1024 peers and large payloads. Thus
their measured rate still has the client-headroom limitation identified
in experiment 36. The continuation small-message clients use only 0.522,
0.545 and 0.395 CPUs at 4/64/1024 peers, while its server uses approximately
one CPU. Its large-payload client is already at 0.999 CPU. These observations
support investigating the small-message server loss first; the nearly
equal large-payload rates do not establish equal server capacity.

Live resident memory, measured separately from timed peak RSS, is:

| Form | 64 × 64 B, KiB | 1024 × 64 B, KiB | 64 × 64 KiB, KiB |
| --- | ---: | ---: | ---: |
| Callee WF | 3,504 | 15,024 | 5,492 |
| Balanced WF | 3,524 | 15,060 | 6,212 |
| Generated continuation | 3,040 | 12,448 | 5,504 |
| `uring-64k` | 2,244 | 4,188 | 3,284 |
| `epoll-calloc-main` | 2,008 | 9,720 | 3,804 |
| `cpp-elide-calloc` | 4,132 | 12,104 | 5,912 |

Values are medians of three live snapshots. The continuation's large-message
range is 5,336–6,568 KiB. Its small-message storage saving does not compensate
for the measured execution loss under the stated performance priority.
Shared provided buffers, shared scratch and per-connection calloc retain
different storage strategies; this table does not erase that distinction.
The sequential source and continuation representation remain viable research
inputs; this particular two-thread coordinator is not a competitive executor.

## Fortieth experiment: independent Rust and Rayon CPU reference

The external baseline matrix now has an executable CPU-only row. The selected
workload is the existing `tests/programs/par_layout.wf`, already used by the
scheduler experiment's finite compute controls. It builds a depth-six box
tree with 127 nodes, fills an 8192-entry word-metric table, and performs 800
full-table layouts followed by 800 layouts using a 4096-entry prefix per
batch. Each layout writes the resolved result into every node. This is a
recursive shared-read/disjoint-write workload, not a newly selected integer
kernel made convenient for Rayon. The integer recurrence in
`windows_runtime_mixed.wf` and `compute_protocol.h` was inspected but is not
parallelized internally: each recurrence step depends on the preceding one.

`research/experiments/io-completion-bench/rayon-baseline/` is a standalone
native reference crate. It belongs to this comparison and is removed when
the comparison is replaced. It adds no dependency to the compiler. The lock
records Rayon 1.12.0, rayon-core 1.13.0, crossbeam-deque 0.8.8,
crossbeam-epoch 0.9.21, crossbeam-utils 0.8.23 and either 1.18.0. The release
profile uses one codegen unit and thin LTO, without fast-math or reassociation.

The sequential port preserves the floating-point expression order, explicit
fused multiply-adds, table traversal order, child-result grouping, node
writes and sequential repetition order. The Rayon port uses
[`rayon::join`](https://docs.rs/rayon/1.12.0/rayon/fn.join.html) for disjoint
sibling calls. Grain means the maximum leaf count of a subtree executed
sequentially. The word sum and repeated mutations of the same tree remain
sequential. The explicit pool is created once, and one `install` encloses the
whole invocation; there is no unnecessary pool entry per node or layout.
One-worker Rayon still creates a one-worker pool; ordinary sequential Rust
creates none. These are different useful startup controls. WF_WORKERS is the
existing WF execution-thread budget, including its calling execution thread.

`make rayon-check` performs the small deterministic qualification. The
complete sequential result matches the independent corpus expectation
`420a993efa7437a1 41fa962893d45299`. A second test compares all 127 written
node values, not just the root, over pool widths 1/2/4, grains 1/3/4/16/64,
and repeated full/empty/prefix-table walks. The canonical root
`research-tests` target calls this check; its tests use the optimized gate
profile with overflow checks and debug assertions. Formatting and clippy
with denied warnings are included. The gate's explicit fetch step includes
this separate lock file; subsequent builds remain `--locked --offline`.

`make -C research/experiments/io-completion-bench rayon-bench` runs the actual
comparison. It compiles the existing WF program with `--no-overlap` and
`--par`, checks their output at every requested worker count, then uses
`runner.c` to alternate native configurations in forward/reverse order.
The first cohort calibrates grains 1/4/16 at pool widths 1/2/4. The lowest
median grain for each width is frozen in `selected.tsv` before an independent
confirmation cohort is run. Selection uses the printed 0.01 ms medians; an
exact tie retains the first configured grain. It does not choose a lucky
confirmation sample as the reference. BATCHES defaults to one; WF's complete
argument count selects the same count using BATCHES minus one dummy argument.

The runner's optional `WF_BENCH_RAW` output retains every successful recorded
sample in order, with wall/user/system milliseconds before summary sorting.
Its ordinary output and correctness criterion are unchanged. A local
wrong-output negative control fails and emits no measured sample. The script
retains calibration/confirmation plans, raw TSVs, summaries, exact WF output,
the dependency lock, host details, source hash, compiler hash and binary
hashes in OUT. Only correctness belongs to the gate; timing selects no
source-language acceptance or performance pass/fail threshold. The separate
`io-scheduler` Rayon branch job runs this bounded panel instead of the larger
allocator/network timing panel.

CLANG configures only the timing runner's C compilation. The current
`whitefootc.rs::clang_executable` selects `/usr/bin/clang` for native linking
on both supported POSIX hosts. The harness records these two compiler roles
explicitly, plus the Rust source/profile/lock and compiler-CLI source hashes;
it does not infer the WF linker from an arbitrary CLANG override. The local
cohort below used the same `/usr/bin/clang` for both roles.

### Local qualification and indicative measurement

The first local cohort used MacBookPro18,3, arm64 Darwin 25.6.0, eight physical
and eight logical CPUs, 32 GiB RAM; no CPU affinity or exclusive host ownership
was established. Rust was 1.98.1 with LLVM 22.1.8; WF native linking used
Apple Clang 21.0.0. The WF source SHA-256 was
`7bfd197936acdd03fa1adf387e7038279874a3728c23be1f0d33f3eafc93df09`,
and the compiler binary SHA-256 was
`f3ecdb38d00945002c5a73e39f8b8a2417a547d9a8cb46c2ee4aae1c26409285`.
The implementation was based on f0d633c1; these local samples precede its
Rayon-reference commit and therefore describe that tested worktree.

All native tests, formatting and clippy passed. Compiled WF sequential and
parallel programs and every measured native invocation published the exact
expected bytes. Three calibration passes followed one warmup; five
confirmation passes followed a separate warmup. Calibration selected grains
16/1/1 for widths 1/2/4. The [70 raw samples](../../experiments/io-completion-bench/rayon-baseline/m1-2026-09-06.tsv)
retain both cohorts. Their counts, alternating execution order and every
reported wall/user/system summary were independently recomputed; agreement
is within the summary's printed precision.

| Confirmation form | Wall median ms | Wall min..max ms | User median ms | System median ms |
|---|---:|---:|---:|---:|
| Rust sequential | 1586.33 | 1582.19..1591.35 | 1559.46 | 18.85 |
| Rayon 1 worker, grain 16 | 1585.56 | 1583.18..1619.89 | 1558.38 | 18.68 |
| Rayon 2 workers, grain 1 | 834.45 | 833.04..837.89 | 1607.77 | 23.97 |
| Rayon 4 workers, grain 1 | 488.01 | 486.27..490.91 | 1695.81 | 77.83 |
| WF sequential | 1585.54 | 1583.51..1603.10 | 1556.43 | 20.02 |
| WF parallel, 1 worker | 1605.48 | 1600.86..1688.85 | 1558.27 | 33.07 |
| WF parallel, 2 workers | 848.64 | 844.38..855.29 | 1620.18 | 29.65 |
| WF parallel, 4 workers | 491.89 | 491.19..494.77 | 1712.20 | 95.93 |

Same-pass WF-parallel/Rayon wall ratios at widths 1/2/4 have medians
1.012758/1.018216/1.010128 and ranges
1.009084..1.042568 / 1.007744..1.023051 / 1.001996..1.013856.
All five pairs favor Rayon at each width. Corresponding total user+system
CPU ratio medians are 1.009477/1.011172/1.018964, with every pair above one.
The sequential medians are essentially equal; four-worker native speedup
over sequential Rust is about 3.25x. These small gaps are useful evidence of
a competitive independently implemented CPU reference, not evidence of
universal Rayon/WF rankings or scheduler-only causation.

These are complete-process timings including allocation, pool startup,
formatting and shutdown. The platform schedules heterogeneous laptop cores;
there is no separate kernel-time profile, sustained-load isolation, RSS
comparison, unbalanced-tree qualification or real I/O in this panel. Native
Rust and WF use different LLVM versions and different language frontends.
The matched operations make the workload comparable but do not erase those
code-generation differences. The Linux cohort below is a separate hosted
measurement, not a replication on the same hardware.

### Linux qualification and independent confirmation

[Run 34084372230, job 101625479691](https://github.com/mbbill/Whitefoot/actions/runs/34084372230/job/101625479691)
completed successfully at d79ffaf1c1028e88832852b91764dbbc9967e875.
Formatting, both correctness tests, clippy, compiled-WF byte qualification and
every timed invocation passed. The job took approximately three minutes,
including toolchain setup and builds. The separate
[canonical gate 34084372209](https://github.com/mbbill/Whitefoot/actions/runs/34084372209)
subsequently completed with all fourteen Linux/macOS jobs passing at the same
d79ffaf1 revision.

The host was an Ubuntu 24.04 Azure VM with kernel 6.17.0-1022-azure and an
AMD EPYC 9V74 model string. Its four allowed vCPUs, 0-3, were reported as two
cores with two SMT threads each, one socket and one NUMA node. This guest
topology does not establish four dedicated physical cores. Rust 1.98.0 used
LLVM 22.1.8. The job explicitly set CLANG=/usr/bin/clang, and that same path
is selected by whitefootc for native linking; it reported Ubuntu Clang
18.1.3. No affinity or exclusive physical-host allocation was established.

Three calibration passes selected grains 16/16/4 for widths 1/2/4, then five
independent confirmation passes used those frozen choices. Both cohorts had
their own warmup. The [70 raw samples](../../experiments/io-completion-bench/rayon-baseline/linux-2026-09-07.tsv)
retain all thirty calibration and forty confirmation observations. Counts,
alternating execution order, all printed wall/user/system summaries and the
four retained WF output files were independently checked successfully.

| Confirmation form | Wall median ms | Wall min..max ms | User median ms | System median ms |
|---|---:|---:|---:|---:|
| Rust sequential | 1320.81 | 1320.57..1330.02 | 1320.31 | 1.00 |
| Rayon 1 worker, grain 16 | 1321.48 | 1321.10..1324.43 | 1320.30 | 1.00 |
| Rayon 2 workers, grain 16 | 670.37 | 669.57..670.60 | 1324.29 | 15.00 |
| Rayon 4 workers, grain 4 | 354.04 | 353.33..354.20 | 1340.95 | 65.85 |
| WF sequential | 1329.70 | 1326.20..1332.26 | 1327.42 | 3.00 |
| WF parallel, 1 worker | 1356.16 | 1354.49..1359.79 | 1321.47 | 33.97 |
| WF parallel, 2 workers | 707.68 | 705.59..710.12 | 1343.37 | 35.97 |
| WF parallel, 4 workers | 425.54 | 423.84..429.29 | 1393.35 | 77.80 |

Ratios below are calculated per pass before taking their median and range;
they are not ratios of independently selected median samples. Total CPU
means the process's measured user plus system time. CPU/wall is average
running-thread equivalents during that complete process, not a measurement
of useful work or physical-core occupancy.

| Workers | WF/Rayon wall median [min,max] | WF/Rayon total CPU median [min,max] |
|---|---:|---:|
| 1 | 1.025992 [1.023958,1.029111] | 1.025329 [1.023379,1.028179] |
| 2 | 1.056016 [1.052503,1.058935] | 1.029694 [1.027706,1.030356] |
| 4 | 1.202192 [1.196595,1.214973] | 1.045739 [1.043629,1.049634] |

| Form | CPU/wall median [min,max] | Own sequential/form wall median [min,max] |
|---|---:|---:|
| Rust sequential | 0.999781 [0.999746,0.999807] | 1.000000 |
| Rayon 1 worker | 0.999884 [0.999864,0.999904] | 0.999614 [0.999404,1.004225] |
| Rayon 2 workers | 1.997914 [1.997783,1.997958] | 1.972625 [1.969231,1.984005] |
| Rayon 4 workers | 3.973502 [3.972294,3.975693] | 3.734575 [3.728268,3.757458] |
| WF sequential | 0.999783 [0.999762,0.999793] | 1.000000 |
| WF parallel, 1 worker | 0.999252 [0.998979,0.999324] | 0.979345 [0.978429,0.982377] |
| WF parallel, 2 workers | 1.948090 [1.941758,1.950716] | 1.878948 [1.867566,1.883669] |
| WF parallel, 4 workers | 3.454002 [3.434661,3.465551] | 3.124893 [3.102125,3.130760] |

Every pair favors Rayon in wall time and total CPU. At four workers WF takes
about 20% more wall time but only about 4.6% more CPU in this sample set; its
CPU/wall ratio is about 3.45 against Rayon's 3.97. This supports investigating
parallel utilization and additional CPU work separately. It does not yet
identify whether the missing concurrency comes from ready-work distribution,
parking, granularity or the program's critical path. The sequential median
gap is only about 0.7%, so differing backend versions do not explain away
the observed parallel loss. The next diagnostic experiment should retain
ordinary timing as its own cohort and add separate four-worker CPU/scheduler
observations. These hosted results do not establish a universal ranking.

The [complete artifact](https://github.com/mbbill/Whitefoot/actions/runs/34084372230/artifacts/10004859489)
contains plans, host metadata, raw samples, summaries, dependency lock and
native binaries. Its ZIP SHA-256 is
`203f614da02ae7946a9ff1ac7cb4db956f259816a5f1d7788d4d556709ea678c`.
The WF source hash is unchanged from the local cohort. The following hashes
identify the measured code and dependency profile; the source/profile hashes
were verified from the exact d79ffaf1 tree after downloading its artifact.

| Artifact or source | SHA-256 |
|---|---|
| whitefootc binary | `5c03c1893b63243bc51b5f3a9a396603cdf0ce8aaa25e2f1c45526f9b827a803` |
| WF sequential binary | `4f23a160e412fa00d90478b94668ac34efd6d7dd51bf1c23e001c4475ac23d3c` |
| WF parallel binary | `948ec36393b60f922cc4132c36b5cd1012fd4b8559a2523aa692a12f6e16db54` |
| Rust reference binary | `2a8289d8ca19fb74b0b035edb732cba533ab033c053379d36ae58c764b4d7d45` |
| whitefootc.rs | `66697813475573bedb116443f022284562c83938b6d705c61f03958e60566e67` |
| Rust reference main.rs | `86f4e33f5ec386cbcfbc111274dc8908fd9cd66a787eaffd1ee38cdf2e7be259` |
| Rust reference Cargo.toml | `7c11ba05d8e9bebc61bad049c5fc1150b17acb893bc2aec01535a4496aa4525b` |
| Rust reference Cargo.lock | `49a17a4292edea7a2e68dc2189c470404ffa5442170cf4e0fc21f8115b7e0637` |

The next mixed-I/O reference should use an async network/file driver plus a
bounded Rayon CPU pool, retaining the existing request framing and recurrence
result. Only independent requests or independent outer jobs may be offered
to the CPU pool. Charge enqueue, wakeup, completed-result transfer and queue
storage to the request, keep driver and CPU workers within one total CPU
budget, and measure paced light-request tails under heavy CPU work. Rayon
`join` is intended for CPU work; its documentation explicitly describes
blocking-I/O hazards. This panel does not qualify that transfer path and
does not claim that blocking I/O placed inside Rayon is the intended model.

## Forty-first experiment: attribute the WF/Rayon parallel utilization gap

The Linux confirmation in experiment 40 gives a concrete next question.
At four workers, WF uses about 4.6% more process CPU than Rayon but takes
about 20.2% longer. CPU time divided by elapsed time is about 3.45 for WF
and 3.97 for Rayon. Sequential execution differs by less than 1%, and the
same direction appears in all five paired samples. These measurements
separate extra CPU work from missing parallel overlap; they do not identify
whether the latter comes from runtime sleeping, the computation's critical
path, runnable tasks waiting for a CPU, or some combination.

`RAYON_PROFILE=1` extends the existing CPU harness after its independent
calibration and uninstrumented confirmation phases. The
`codex/io-compute-profile` CI branch installs Linux perf and selects this
mode. It retains the two sequential controls and both parallel forms at
1/2/4 workers. Each observation executes four identical source batches with
the same ordinary binary and that host's frozen Rayon grain. Separate
captures collect inherited `cpu-clock` samples at 999 Hz and Linux scheduler
switch/wakeup events. No profiled elapsed time enters the confirmation TSV.
The recorder has tracepoint privileges, but an exec wrapper returns the
workload to the original job user and records its actual PID. Program
stdout must match the corpus oracle and program stderr must remain empty;
recorder diagnostics have separate files.

CPU reports retain thread identity, DSO and symbol. The scheduler report
filters the recorded process and its threads, retaining switch-out state,
wakeups, migrations, runtime and runnable delay. As documented by
[Linux perf sched](https://man7.org/linux/man-pages/man1/perf-sched.1.html),
its wait interval and runnable scheduling delay are different quantities.
A sleeping worker and a worker runnable but not scheduled therefore require
different explanations. Raw perf data, decoded events, commands, PIDs,
versions and diagnostic files remain available for checking that distinction.
Capture loss must be audited before attributing a gap. Full scheduler
tracing adds work and can perturb the execution; a trace supplies mechanism
evidence, not a replacement throughput ranking or a precise decomposition
of the uninstrumented 20% difference.

Shell syntax, workflow YAML and diff checks pass locally. Linux capture and
attribution remain pending. This experiment changes no compiler or runtime
policy. The next policy change should follow the observed source of lost
overlap, then receive independent uninstrumented confirmation against both
the retained WF form and Rayon. Retire this optional capture mode when that
question has a measured answer.

The first CI attempt at `caafda17`
([job 101629327356](https://github.com/mbbill/Whitefoot/actions/runs/34085744798/job/101629327356))
installs perf 7.0's packages but stops before compiling or measuring: unlike
the package-list query, `dpkg-query -L` rejects a package-name wildcard.
The follow-up enumerates installed Linux tools packages first, then queries
those exact names, including the HWE package that owns the executable.
No workload verdict, measurement threshold or capture option changes.
The second attempt at `58f41f23`
([job 101630130995](https://github.com/mbbill/Whitefoot/actions/runs/34086022372/job/101630130995))
also stops during setup: the package lists contain a Python module directory
named `perf`, and an executable-bit test also accepts searchable directories.
Tool discovery now requires a regular executable file. Neither failed
setup produces workload measurements or profiling evidence.

Revision `0d15c7cc` completed all sixteen CPU/scheduler captures in
[run 34086189734](https://github.com/mbbill/Whitefoot/actions/runs/34086189734).
The raw-output, thread and sample-period audit is recorded with its resulting
fixed-cost control in experiment 45. Those instrumented four-batch captures
do not reproduce the ordinary single-batch utilization gap and therefore do
not establish excess parking as its cause.

## Forty-second experiment: async TCP with bounded Rayon CPU offload

The mixed reference is now an optional `mixed` feature and a separate
`mixed-rayon` binary in the existing standalone Rayon crate. It serves the
existing `compute_protocol.h` and `tcp_compute_server.wf` workload, without
changing the CPU-only layout binary, compiler dependencies or WF semantics.
This subsection owns the binary and `mixed-rayon-smoke.sh`; remove them when
this comparison is superseded. No comparative mixed-load winner is selected
by this implementation checkpoint.

The reference uses a single
[`Tokio current-thread runtime`](https://docs.rs/tokio/1.53.1/tokio/runtime/struct.Builder.html#method.new_current_thread)
for nonblocking TCP I/O and B-1 Rayon workers for a total execution-thread
budget B. The normal CLI is `mixed-rayon PORT CONNECTIONS --threads B
[--queue Q]`, with B at least two and Q defaulting to 2*(B-1). The driver
does not enter `ThreadPool::install`, execute a nonzero recurrence, call
`spawn_blocking`, resolve DNS or perform blocking filesystem I/O. There is
one async sequential request loop per accepted connection.

Every request is the same 64 bytes: big-endian u64 seed and round count,
followed by 48 ignored reserved bytes. Counts above 16777216 fail the
protocol. The dependent rotate/XOR/wrapping multiply/add recurrence is
sequential within a request. The 64-byte reply encodes the result's bits
least-significant first. Only independent requests enter the CPU pool.
Zero rounds return the seed directly and require no CPU-pool transfer;
the implementation never examines connection identity or a known heavy-peer
index to select execution behavior.

For a nonzero request the handler asynchronously acquires one of Q
[`Semaphore`](https://docs.rs/tokio/1.53.1/tokio/sync/struct.Semaphore.html)
permits before submitting an owned Rayon job. Q counts queued plus executing
CPU jobs, not merely workers. The job carries seed, count, a one-value
oneshot sender and the permit; it retains no socket or buffer reference.
Its result wakes the awaiting handler. The permit is released before any
response write, so a slow receiver cannot reserve CPU admission indefinitely.
The handler encodes and completely writes that result before reading another
frame. There is at most one partial/full request, pending admission or pending
reply per accepted connection; total handler storage is bounded by CONNECTIONS.
Semaphore admission is FIFO; no strict global FIFO claim is made about
Rayon's work scheduling.

Reads retain an explicit cursor so frame-boundary EOF succeeds while a
partial frame fails. A full frame followed by a write-half close is answered
before EOF is consumed. Short writes use Tokio's `write_all`. Any protocol
or I/O error stops new accepts even if fewer than CONNECTIONS peers arrived,
aborts and joins the remaining handlers, and waits to acquire all Q permits
before returning failure. Dropped receivers discard completed CPU values
safely; jobs remain finite and release their permits. Dropping ThreadPool is
not mistaken for a synchronous join. Normal completion waits for all accepted
connections to finish. The admission and lifetime bounds hold for arbitrary
valid seeds, counts and TCP segmentation, not just the load generator's shape.

Tokio 1.53.1 is exactly pinned with rt/net/io-util/sync features; no macros,
multithreaded Tokio scheduler or timer driver is requested. Socket2 0.6.5 is
an optional direct dependency used only in tests for safe socket-buffer and
RST configuration. The updated lock also records bytes 1.12.1, mio 1.2.3,
libc 0.2.189, pin-project-lite 0.2.17 and their platform dependencies; the
Rayon dependency versions from experiment 40 remain unchanged. Existing
canonical fetch wiring fetches this lock once; all checks/builds remain
locked and offline.

`make mixed-rayon-check` is wired into canonical `research-tests`. The
following local M1 checks passed with Rust 1.98.1, Apple Clang 21.0.0 and
the optimized gate profile:

| Qualification | Observed result |
|---|---|
| Three independently fixed C-oracle vectors, every request split at 1..63, coalesced frames and full-frame half close | Exact ordered response bits and final EOF |
| B=2/Q=1 and B=4/Q=6, more heavy requests than CPU admission slots, each doing 16777216 real recurrence steps | Every CPU worker active, admission full, at least two waiting handlers; zero-round replies completed while all CPU workers were active |
| Illegal count and truncated frame while another request computes, with fewer peers than planned CONNECTIONS | Failure stops admission, cancels handlers and drains CPU jobs |
| TCP RST during a CPU job | I/O failure, no surviving admission permit or unfinished CPU job |
| Forced 4096-byte socket buffers, 2048 pipelined replies and a receiver that initially does not read | Actual write future returned Pending; another connection completed, then every buffered reply drained correctly |
| Existing external `stream_check` compute and truncated modes, total threads 2 and 4 | All four runs passed |
| Formatting, all-feature clippy with denied warnings | Passed |

The five Rust tests completed in approximately 0.20 seconds on the local
host. Every server finish checks submitted=completed, zero active/inflight/
waiting counts, restored permits, inflight peak at most Q and active peak
at most B-1. The max-round seed-seven expected value
`c39350d53d849e85` was computed independently using the existing C
`compute_churn(7, COMPUTE_MAX_ROUNDS)` implementation before fixing the
literal in the Rust protocol test. Qualification socket limits and atomic
observers are not part of ordinary timing builds.

`make mixed-rayon-smoke` is a separate Linux-only caller using unchanged
`netload.c` with its service budget of eight. It performs four short
qualifications: B=2/4, ordinary/observed binary, 16 admitted connections,
1048576 heavy rounds on every fourth peer, and 100 planned light arrivals
per second per other peer for 500 ms. Every planned request is drained and
checked by netload's existing oracle. It records the ordinary output and
resource figures separately from `mixed-observe` counters and requires
observed light progress while all CPU workers are active. Host, toolchain,
source/lock/binary hashes and output files remain in OUT. Its dedicated CI
branch excludes the larger allocator and CPU-layout timing panels.

The [Linux qualification job](https://github.com/mbbill/Whitefoot/actions/runs/34086375759/job/101631101434)
passed at 040bfc4b08f1b4d164b50281212f2c053323964e. All five Rust tests
passed in 0.23 seconds, all four external C oracle invocations passed, and
all four ordinary/observed netload qualifications passed. Its separate
[canonical gate](https://github.com/mbbill/Whitefoot/actions/runs/34086375764)
also passed all fourteen Linux/macOS jobs. Every sample
completed all 600 planned light requests before its nominal deadline and
drained every heavy reply. The [four raw qualification rows](../../experiments/io-completion-bench/rayon-baseline/mixed-linux-2026-09-07.tsv)
retain the full client reports, process resources and observer fields;
ordinary rows leave observer fields empty.

| Observed total budget | CPU workers | Admission limit / peak | Submitted = completed | Light handled while all CPU workers active | Final active / inflight / acquiring |
|---|---:|---:|---:|---:|---|
| 2 threads | 1 | 2 / 2 | 256 | 599 | 0 / 0 / 0 |
| 4 threads | 3 | 6 / 5 | 756 | 582 | 0 / 0 / 0 |

The acquisition counter includes the short immediate-permit path as well as
suspended acquisitions; it is not itself a count of blocked requests. CPU
inflight includes completion handoff until the worker retires its permit.
The max-round Rust test separately proves admission saturation for B=4/Q=6;
the four-heavy-peer smoke need not fill that limit. These qualifications
establish observable light progress and bounded lifetimes, not an upper
bound on latency under arbitrary loads.

The hosted machine reported AMD EPYC 7763, four allowed vCPUs 0-3, two
guest cores with two SMT threads per core, Linux 6.17.0-1022-azure, Rust
1.98.0/LLVM 22.1.8 and Ubuntu Clang 18.1.3 for the C client. The complete
[artifact 10005404525](https://github.com/mbbill/Whitefoot/actions/runs/34086375759/artifacts/10005404525)
retains the qualification log, host, lock, binaries and raw files. Its ZIP
SHA-256 is `05a187b37b4fa659e65acece9391716cb346ee0a7a98c31d7e12cb2c8c71cac2`.
Every retained binary/source/lock hash and the four rows' lifecycle conditions
were independently checked after downloading it:

| Artifact or source | SHA-256 |
|---|---|
| Ordinary mixed binary | `d11ae51aac21f6303745e8ac8089f64e3844a9f2944bc8ff104b30178fed39a5` |
| Observed mixed binary | `53adce70a93c210107d25c151b8165111b51a4cbb8e453b425a393d96b588389` |
| netload binary | `68692d0a80033e2069eb7a00034e6b4ad68f1cae470ec5c0a89f600ce8a360a8` |
| mixed.rs | `9182022c836bdd581ee5e670e65065d4fe1f313894a1fc4414f33433ea24835f` |
| Cargo.toml | `6d681897a3b35a83f487782ad29edc5b41af3954335b57c5b4273bee7db5889b` |
| Cargo.lock | `f9b36940fad5eea154d9dcb3e683caeaf0d7831f1510ba5479d253093c6c99cf` |

These small shared-host smoke samples are not a fair performance
comparison: the client shares the unrestricted host CPU set and there is no
independent tuning or confirmation cohort. The next comparison should freeze
queue/driver settings, count the I/O thread within total budgets 2/4, give
the client separately established CPU capacity, and compare paced light
tails, heavy completions, drain and CPU/RSS against qualified native and WF
forms. Alternative I/O backends and richer CPU tasks remain separate rows.

## Forty-third experiment: client CPU headroom before server saturation claims

Experiment36's raw exchange CPU counters show the load generator occupying
nearly all its assigned CPU in the occupied echo cells. Source inspection
rules out a proposed redundant-payload optimization: `fill_message` already
prefills each echo payload before connecting, `begin_round` changes only its
round byte, and `pump` uses full-length `memcmp` on the success path. The
observed occupied split1 client CPU is predominantly system time. This first
phase changes client resources before modifying client code without a
demonstrated redundant operation.

`make scheduler-client-headroom` selects `CLIENT_HEADROOM=1` with the existing
native panel. One server worker remains on the same logical CPU in every
sample. The client uses either one thread on one hardware thread of another
physical core, or two threads allowed on that same core's two SMT siblings.
Topology is derived from the process's allowed CPUs and physical core/socket
identity; a machine without two available client siblings fails explicitly.
The client never uses the server CPU's sibling. The cohorts and complete
`lscpu` topology are retained with the artifact.

The panel keeps `callee-small`, `balanced-small`, `uring-64k`,
`uring-64k-inline`, `epoll` and `cpp-elide`. It reuses the 64-peer × 64 B
(2,000 trips/peer) and 64-peer × 64 KiB (500 trips/peer) cells, with two warmups
and seven recorded passes. Representation and client-placement order alternate
by pass, yielding 168 rows. All variants in a client-placement cohort use the
same client binary, payload sequence, single-outstanding-request protocol,
full byte validation and strict errors. This revision includes the native
successful-exit wake-storage correction; it does not overwrite the running
`0357259d` hybrid screen.

All existing completion, native stream, coroutine and uring qualification
remains before timing. The new CPU-resource panel omits only the separate
resident-memory measurements; prior panels and canonical checks retain their
coverage. Raw client exchange user/system CPU, server resources and latency
remain per sample. This phase does not change compiler/runtime interfaces or
claim the current WF stackful rows measure the newer generated continuation.

The discriminating comparison is each fixed server's paired rate, tail and
CPU cost with two versus one client workers, together with client exchange
CPU/wall. A rise in rate under unchanged server resources demonstrates that
the former end-to-end ceiling was sensitive to the generator. An unchanged
rate cannot establish server saturation: SMT contention inside the client
core may still cap both variants. Extra physical client cores or another
host remain the stronger follow-up, and loopback remains distinct from a
real NIC. Local shell parsing, Make dry-run, YAML parsing and diff checks
pass.

### Client placement results at 0ebe924b

[Run 34085705133](https://github.com/mbbill/Whitefoot/actions/runs/34085705133)
and measurement job `101629217722` passed. All fourteen canonical gate jobs
and both io-hosts jobs passed at this exact revision. The
[raw artifact 10005493034](https://github.com/mbbill/Whitefoot/actions/runs/34085705133/artifacts/10005493034)
has SHA256 `edb74ace6c9f6b23f32eb83c7aa7b86d23af529912f5c1270f494ceb6123de42`.
The host is EPYC 7763, Linux 6.17.0-1022-azure and Clang 20.1.2. Server CPU 0
and one worker are fixed; the client changes from CPU 2/one worker to CPU
2,3/two workers on the other physical core. Its absolute numbers are not
paired with experiment38's EPYC 9V74 host.

The separate Windows io-bench timing
[job 101629217220](https://github.com/mbbill/Whitefoot/actions/runs/34085705087/job/101629217220)
stopped at the same `compute remained unstable after two complete cohorts`
gate; its three Linux/macOS timing jobs and the Windows owner checks passed.
That unrelated unqualified Windows timing result is not folded into this
Linux client-placement comparison.

The raw audit verifies all 168 rows and 24 seven-pass groups, matching client
metrics, exact byte/trip totals and client/server resource files, and empty
timed diagnostics. The following ratios pair the same pass, payload and
server form with two versus one client workers. CPU cost is the unchanged
whole-server-process CPU/trip measure. The final column is the median client
exchange CPU/wall, in CPU-seconds per wall-second; client capacity is one
logical CPU in the first cohort and two SMT siblings in the second.

| Payload bytes | Form | Rate ratio | Server CPU/trip ratio | p99 ratio | Client CPU/wall, one → two |
| --- | --- | --- | --- | --- | --- |
| 64 | callee-small | 0.9772 [0.9506, 1.0384] | 1.0198 [0.9604, 1.0495] | 0.9972 [0.9259, 1.2367] | 0.982 → 1.347 |
| 64 | balanced-small | 0.9850 [0.9177, 1.0316] | 1.0194 [0.9800, 1.0947] | 1.0139 [0.9169, 1.0691] | 0.986 → 1.326 |
| 64 | uring-64k | 0.9714 [0.9007, 1.0414] | 1.0521 [1.0000, 1.1099] | 2.7021 [0.9594, 5.0501] | 1.000 → 1.541 |
| 64 | uring-64k-inline | 1.0488 [0.9726, 1.1201] | 0.9691 [0.9158, 1.0330] | 1.0076 [0.8701, 1.1590] | 1.000 → 1.666 |
| 64 | epoll | 0.9635 [0.9032, 1.0346] | 1.0408 [0.9592, 1.1087] | 1.0600 [1.0208, 1.2720] | 1.000 → 1.631 |
| 64 | cpp-elide | 0.9632 [0.9267, 1.0304] | 1.0303 [0.9794, 1.0816] | 1.0778 [1.0000, 1.5291] | 1.000 → 1.634 |
| 65536 | callee-small | 1.1024 [0.6260, 1.1209] | 0.8716 [0.8571, 1.5412] | 0.8707 [0.5707, 1.2034] | 0.999 → 1.989 |
| 65536 | balanced-small | 1.0900 [0.6284, 1.1168] | 0.8836 [0.8707, 1.5476] | 1.0396 [0.7486, 1.7703] | 0.999 → 1.988 |
| 65536 | uring-64k | 1.0149 [1.0021, 1.0433] | 0.9862 [0.9720, 1.0141] | 1.9541 [1.7651, 2.4880] | 1.000 → 1.819 |
| 65536 | uring-64k-inline | 1.0289 [1.0163, 1.0597] | 0.9793 [0.9510, 0.9930] | 1.0157 [0.9371, 1.2568] | 1.000 → 1.833 |
| 65536 | epoll | 0.7717 [0.5828, 1.1199] | 1.0111 [0.8208, 1.1750] | 0.9084 [0.6310, 1.5789] | 1.000 → 1.999 |
| 65536 | cpp-elide | 0.9413 [0.5791, 1.0955] | 0.9200 [0.8762, 1.1235] | 0.9322 [0.5551, 1.5517] | 1.000 → 1.999 |

Every small-message rate range includes 1.0. Both native uring
large-message rates improve in all pairs, but only by median 1.5% and 2.9%;
pure-ring p99 becomes 1.77--2.49 times worse. For the hybrid, absolute large
rate medians are 21,840 → 22,531 trips/s and server CPU 45.312 → 44.062
us/trip, while client CPU rises 45.778 → 81.795 us/trip. Thus the extra client
SMT worker is neither free nor proof that the client now has spare capacity.

The WF large-message median increases are not stable improvements: the
single-client samples have two distinct observed ranges. `callee-small`
has five samples around 21--22k trips/s and two around 36--38k;
`balanced-small` has four around 21--22k and three around 37--38k. The faster
samples also lower server CPU from roughly 46 to 26--28 us/trip and reduce
client system CPU. The two-client WF samples cluster around 23--24k trips/s
while occupying nearly two client logical CPUs. Epoll's single-client rates
range from 20.7k to 39.1k. These are retained observations, not discarded
outliers; aggregate counters do not establish their cause. Changing CPU work
per request warrants syscall/packetization evidence rather than simply
attributing the spread to host descheduling.

This sensitivity screen does not produce a stronger, demonstrably idle
generator for all forms. Future server claims need additional independent
physical client cores or a separate host, and the complete same-client
cohorts must remain the comparison units. On this available VM, the next
bounded diagnostic is send/receive/EAGAIN/readiness and transfer-size counts
for the variable large-message cell. No client verifier, source ABI, server
default or performance threshold is changed by these results.

## Forty-fourth experiment: let the continuation resumer drive target progress

Experiment 39 identifies a concrete implementation loss: the generated
continuation host uses several process context switches per echo trip and
is much slower than both stackful WF and native forms for concurrent small
messages. The first isolated change removes the handoff between its sole
resumer and background progress thread. It does not change the source,
compiler lowering, frames, task window, buffer policy, completion record or
the established `wf__par_*` interface used by other work.

`WF_CONTINUATION_OWNER_PROGRESS=1` selects the new path in the same host
binary; the default and explicit zero keep the threaded control. The source
still returns to its owning resumer on suspension. When the new-task queue
is empty, that owner captures the completion wake epoch, drives target
progress, and checks the locked ready queue. If empty, it parks against the
captured epoch. A helper publication between that check and park changes
the epoch and therefore prevents a lost wake. Progress runs outside the
coordinator lock because reaping may synchronously call the publication
bridge. The owner never resumes source from inside a publisher.

The existing pending-list search, ready queue, counters, condition broadcasts
and target notifications remain. Thus a win would isolate the value of
avoiding the inter-thread handoff; it would not establish that this still
unoptimized coordinator is the best executor. Helpers retain their existing
responsibilities and resource accounting. With no helper, target progress
can execute an adapter request synchronously, as its existing contract
already permits; this experiment does not introduce cancellation or general
nonblocking cleanup semantics.

All generated file, pipe, recursive, TCP accept/connect/refusal, occupied
port, fanout and multi-batch staged-window qualifications run at both
settings. The independent C++ completion-loan fixture keeps its threaded
publisher-ordering cases. Every generated observer reports its actual owner
mode, checked against the requested setting as well as existing exact bytes,
route counts and complete task retirement. The build without sanitizers
also runs the four-slot, twelve-task oracle at both settings.

`CONTINUATION_SCREEN=2 NATIVE_BASELINES=1 EXPERIMENT=allocator` extends the
experiment 39 panel with `wf-coro-owner`. The same generated binary supplies
both continuation rows. Four WF forms plus seven native forms, five cases,
seven alternating passes and three repetitions of three live memory cases
give 385 timing rows and 99 snapshots. All server threads still share CPU 0
and the client uses the other physical core. Native/helper ASan/UBSan and
the common 2 MiB backpressure/half-close oracle precede timing for both
continuation settings. The work branch is `codex/io-continuation-owner`.

The main comparison is owner versus threaded continuation within each
paired pass, retaining throughput, p99, server CPU/trip, process switches
and live RSS separately. Existing WF/native rows expose residual costs.
The known client capacity limit still prevents calling close rates a server
capacity frontier. The Linux comparison is recorded below.

Local M1 qualification passes the generated suite at both settings with
ASan/UBSan and ThreadSanitizer, the independent nested C++ lifetime suite,
the twelve-task/four-slot oracle without sanitizers, and the common four-peer
2 MiB stream/backpressure/half-close oracle. The latter retires all four
tasks in both modes, with 64/64 and 75/75 registrations/dequeues respectively.
These local runs use the helper route; they do not qualify Linux native
completion or supply a comparative timing result.

At `f72aacb893966b05796e7571c53f8f80b4e4806f`, the
[Linux continuation/loan job](https://github.com/mbbill/Whitefoot/actions/runs/34087992851/job/101635635389)
passed the generated suite at both owner settings, with native and forced
helper routes. The stream/recursive, accept/connect/refusal, occupied-port,
four-task fanout and twelve-task/four-slot cases all passed, including complete
task retirement. The independent C++ heap and elided fixtures each passed
their 640-case loan/drain oracle on both routes, retaining the explicit
before-arm and during-arm publication cases. Those C++ ordering fixtures
still use the progress thread; they are not owner-mode timing evidence.
[Artifact 10006228142](https://github.com/mbbill/Whitefoot/actions/runs/34087992851/artifacts/10006228142)
retains this qualification separately from the ongoing performance panel.
The Windows memory/placement job in the same workflow also passed; it does
not qualify generated continuations on Windows.

### Linux owner-progress confirmation at f72aacb8

The [measurement job](https://github.com/mbbill/Whitefoot/actions/runs/34087992851/job/101635635457)
completed successfully with all 385 timing rows and 99 live-memory snapshots.
[Artifact 10006435248](https://github.com/mbbill/Whitefoot/actions/runs/34087992851/artifacts/10006435248)
has ZIP SHA-256 `42dbb10f1d4a6b7a2355dbd2dfa012b85c015e1dc437eb1780516442c95320f3`.
Its host was AMD EPYC 7763, Linux 6.17.0-1022-azure and Clang 20.1.2, with
four guest vCPUs on two cores and two SMT threads per core. The single server
CPU was 0 and the single client CPU was 2. Both continuation forms use the
same binary, source, allocation policy and reporting counters; timed runs
disable reports and sanitizers, not the coordinator counters themselves.

Independent raw-data audit found all 55 groups had seven unique passes.
Every timing row matches its client output, server/client resource files,
roundtrip count and rate within the printed rate and truncated-microsecond
precision. Every selected server stdout/stderr and client stderr is empty.
All 99 RSS/anonymous/private-dirty/swap totals were recomputed from smaps;
every process reports THP disabled, zero anonymous huge pages and zero swap.
All samples remain in the result, including broad native large-message and
latency ranges. These are same-revision comparisons; earlier hosts' absolute
rates are not substituted into this panel.

| Peers x bytes | Threaded median trips/s | Owner median trips/s | Paired owner/threaded rate median [min, max] | Paired owner/threaded CPU/trip median |
|---|---:|---:|---|---:|
| 1 x 64 | 22774 | 21969 | 0.963 [0.889, 0.999] | 0.634 |
| 4 x 64 | 38338 | 95110 | 2.500 [2.422, 2.606] | 0.398 |
| 64 x 64 | 47149 | 93433 | 1.975 [1.964, 2.028] | 0.506 |
| 1024 x 64 | 31689 | 89144 | 2.774 [2.746, 2.858] | 0.368 |
| 64 x 65536 | 20977 | 21142 | 1.007 [0.994, 1.020] | 0.796 |

The occupied small-message results strongly support removing this
cross-thread handoff. At 64 peers the median server CPU/trip drops from
21.172 to 10.703 us, switches/trip from 2.743 to 0.000305, and p99 from
2569 to 750 us. At 1024 peers switches/trip drops from 3.861 to 0.002354
and p99 from 58957 to 12660 us. The single-peer case instead loses throughput
in every pair while using less CPU; eliminating a thread is not a universal
latency/throughput improvement. Large-message throughput intervals overlap
while CPU cost falls, and its client consumes approximately one full CPU.

| Peers x bytes | Paired owner/callee rate median [min, max] | Paired owner/callee CPU/trip median | Paired owner/uring-64k rate median | Paired owner/uring-64k CPU/trip median |
|---|---|---:|---:|---:|
| 1 x 64 | 0.759 [0.741, 0.776] | 0.714 | 0.960 | 1.200 |
| 4 x 64 | 0.832 [0.775, 0.873] | 1.167 | 0.739 | 1.400 |
| 64 x 64 | 0.719 [0.709, 0.738] | 1.374 | 0.716 | 1.417 |
| 1024 x 64 | 0.752 [0.728, 0.779] | 1.306 | 0.721 | 1.469 |
| 64 x 65536 | 0.978 [0.970, 0.986] | 0.811 | 0.974 | 0.823 |

`uring-64k` is a fixed reference in this table, not a per-sample winner.
The remaining occupied small-message loss is material. The owner's client
CPU/exchange ratio is about 0.875/0.877/0.859 at 4/64/1024 peers, below the
near-full client occupancy of the faster native controls. This supports
investigating residual server work, but does not establish spare client
capacity for a future faster server. In particular, the close large-message
rates cannot establish equal server capacity.

| Form | Median RSS KiB, 64 x 64 | 1024 x 64 | 64 x 65536 |
|---|---:|---:|---:|
| WF callee-small | 3484 | 15036 | 5512 |
| WF balanced-small | 3528 | 15092 | 6048 |
| WF threaded continuation | 3040 | 12448 | 6204 |
| WF owner continuation | 2968 | 12380 | 4956 |
| Native uring-64k | 2248 | 4188 | 3272 |
| Native epoll shared scratch | 1640 | 1664 | 1700 |
| Native C++ elided shared scratch | 3604 | 3820 | 3636 |

Memory remains a separate axis. Large-message threaded/owner samples span
5392-6568/4820-5040 KiB. Removing a thread does not explain that entire
difference: receive chunking and the touched portion of each calloc-backed
buffer also affect residency. Shared native scratch and WF per-connection
storage have different ownership, so these are competitive resource figures,
not matched-storage representation costs.

The remaining coordinator still calls target progress before taking each
ready waiter. Progress flushes deferred SQEs before reaping, so this can
prevent a group of ready continuations from combining their next submissions.
That mechanism is a source-code hypothesis, not a measured syscall count:
the current continuation report does not expose ring submission/enter totals.
The next bounded control will vary the number of ready resumptions allowed
between progress calls, retain an immediate progress step when no ready work
exists, and report the existing bridge counters in separate observations.
Registration, wake epochs, publication, source storage and the single owner
remain unchanged. Skipping self-wakes is not the first control: the existing
wake implementation already avoids eventfd writes when no sleeper is
announced, so the disappearance of switches alone does not identify excessive
wake syscalls in owner mode.

## Forty-fifth experiment: control CPU stack capacity and fixed costs

Experiment 40's CPU panel inherited `WF_STACKS=1100` from the many-connection
I/O panel. That is a valid capacity setting, but it was not independently
tuned for `par_layout.wf`. At four workers the runtime default is twelve
stacks: four worker stacks plus eight spare stacks. `wf_sched_init` reserves
the requested count, prepares every stack context and touches its metadata;
the POSIX primitive also protects a guard for every stack. The setting can
therefore affect both startup and the chance of taking a no-free-target
compute join path. Reducing it is not automatically a cost-free optimization.

The program also writes its final checksum through ordinary `write_once`.
Its emitted `wf__completion_file_write_submit` calls `wf_bridge_begin`, which
requires bridge initialization before dispatch. On Linux initialization first
attempts a native ring, even though unpositioned `WF_FILE_WRITE` itself uses
the typed file adapter. Both normal and observed WF binaries therefore
include completion initialization and output costs. Missing ring counters on
macOS do not prove the bridge was uninitialized. This is a separate possible
fixed cost; the stack/batch control does not attribute all startup or shutdown
time to stacks, and no output or runtime route is changed in that panel.

The separate four-batch perf audit at 0d15c7cc found no recorded lost-event or
throttle diagnostics and the expected execution threads. It did not reproduce
the ordinary single-batch utilization gap: raw scheduler switches in the
common worker lifetime gave WF/Rayon run-time-to-wall ratios 3.894/3.862.
WF had less blocked time in that capture. The perf CPU report denominator
included wrapper samples despite its PID filter, and the sched timehist final
summary omitted most of the workload; the audit instead filtered raw samples
by TGID and reconstructed switch/waking intervals. Kernel symbols were
unresolved and there were no callchains, so neither a specific kernel wait
nor all layout-symbol samples can be attributed precisely. This evidence
does not establish excessive WF parking as the cause of the timing gap.

The audited [CPU/perf job](https://github.com/mbbill/Whitefoot/actions/runs/34086189734/job/101630615030)
and [artifact 10005361082](https://github.com/mbbill/Whitefoot/actions/runs/34086189734/artifacts/10005361082)
retain separate CPU-clock and scheduler captures, ordinary confirmations,
commands, workload PIDs and exact outputs. The artifact ZIP SHA-256 is
`435f53a48bf743cbd20b277f5dc0b45ce004da072fa2b64e7f3fcb28634385ff`.
The CPU-clock audit selects raw records for TGID 3874 (WF: TIDs 3874-3877)
or 3891 (Rayon: caller 3891, workers 3892-3895), and sums their periods.
Each sample has period 1001001 ns, so the figures are sampled CPU estimates,
not exact instruction costs. The few wrapper records in another TGID are
excluded even though perf's displayed denominator included them.

| Four-batch CPU-clock capture | WF4 sampled CPU, ms | Rayon4 sampled CPU, ms |
|---|---:|---:|
| Total in workload TGID | 5228.23 | 5160.16 |
| Layout functions, including their inlined code | 4809.81 | 4818.82 |
| WF runtime / Rayon and Crossbeam functions | 106.11 | 56.06 |
| pthread mutex functions | 81.08 | 0.00 |
| WF pause / explicit sched_yield symbol | 53.05 | 26.03 |
| Unresolved kernel | 134.13 | 251.25 |
| Remaining symbols | 44.04 | 8.01 |

The zero entry means no samples in that category, not proof of no such work.
The separate scheduler trace targets WF PID 4022 with children 4023-4025,
and Rayon PID 4003 with workers 4004-4007. Excluding Rayon's waiting caller,
raw switch intervals were clipped to each pool's common lifetime, from its
last worker fork to its first worker exit. A switch out in state R remains
runnable; other states count as blocked until the recorded waking event,
then runnable until switch-in. Waking is wake intent, so these are conservative
state estimates rather than exact activation timestamps. Waking records while
the target was already running were ignored; no unmatched running-state
transitions occurred. Exit-boundary unknown intervals were under 0.02 ms.

| Four-batch scheduler capture | WF4 | Rayon4 workers |
|---|---:|---:|
| Common pool lifetime, ms | 1313.481 | 1333.479 |
| Summed running time, ms | 5114.989 | 5150.403 |
| Summed runnable delay, ms | 125.262 | 141.007 |
| Summed blocked time, ms | 13.657 | 42.497 |
| Running time / lifetime | 3.894 | 3.862 |

The scheduler captures even reverse the ordinary confirmation's wall-time
ordering. They are separate, instrumented four-batch executions and cannot
be spliced into a one-batch confirmation as if they described that sample.
WF's first scheduled main-thread interval to final worker fork spans 21.315
ms and its first worker exit to main exit spans 14.690 ms; Rayon's startup
window spans 1.811 ms. These windows include other startup/shutdown and host
activity and are not measured stack setup/destruction costs. They motivate
the following ordinary fixed-resource experiment, not a runtime conclusion.

`make rayon-resource-bench ROUNDS=5 WARMUP=1` executes a deliberately small
control before any runtime changes. It uses the same compiler/runtime and
`par_layout.wf` bytes as 0d15c7cc, the existing independent Rust implementation,
and the following fixed panel. Grain four is frozen from the prior four-worker
calibration; this cohort does not select it again.

| Computing-thread budget | Implementation | Stack capacity | Batches per invocation |
|---|---|---|---|
| 4 | Ordinary Rust/Rayon, grain 4 | Rayon defaults | 1, 4, 16 |
| 4 | Ordinary generated WF parallel | 12 | 1, 4, 16 |
| 4 | Same ordinary generated WF binary | 1100 | 1, 4, 16 |

One plan contains all nine forms, grouped by batch count; each pass reverses
the preceding pass's order. One complete warmup precedes five recorded passes
(45 samples). Every invocation must exit successfully and print the corpus
checksum; the ordinary runner compares it after trimming trailing CR/LF.
Whole-process wall/user/system CPU include pool setup,
tree setup and shutdown. WF uses the main thread plus three spawned workers;
Rayon uses four workers and a caller that waits for the pool. The host CPU
set/topology is recorded, but neither physical-core dedication nor affinity
is assumed. The panel has no concurrent load generator and uses no perf.

After timing, a separate binary links the existing `WF_SCHED_OBSERVE=1`
scheduler/grant observer. It runs each WF cell once, untimed, with the same
checksum and a byte-exact `cmp`, and confirms three spawned workers and
positive grants.
`exhausted_compute` counts join turns without an available target stack;
it is not a count of unique requests or a high-water mark. The runtime has
no existing peak-live-stack counter, so this panel cannot report that value.
Observed counters may perturb scheduling and cannot be assigned to the
ordinary samples. No runtime implementation, policy default, or compiler
lowering changes in this experiment.

OUT retains the ordinary raw `resource.tsv`, alternating plan, summaries,
exact observation output/counters, generated IR, binaries, observer command,
locked dependencies, actual compiler versions and source/binary hashes.
The dedicated `codex/io-cpu-resource-controls` CI branch runs only this CPU
timing panel in place of the large allocator screen; canonical correctness
remains enabled. Compare paired wall and CPU ratios within each batch count,
CPU/wall concurrency, and per-batch cost as duration grows. Those results can
separate capacity sensitivity from amortized fixed costs; hosted noise and
whole-process timing still limit a precise causal cost decomposition.

The local M1 smoke passed the existing two Rust correctness tests, fmt and
clippy, all nine ordinary byte checks, and all six observed byte/counter
checks. Its 12-stack/16-batch observation took fifty no-target compute-join
turns; every other observed cell reported zero. These counters establish
that the small-capacity resource path ran without changing result bytes,
not that the same turns occurred in ordinary samples. The one-pass local
times were visibly affected by shared-host load and are not ranking evidence.
The raw smoke remains in the scratch `whitefoot-rayon-resource-smoke` folder;
Linux confirmation is the separate five-pass cohort.

The [Linux resource-control job](https://github.com/mbbill/Whitefoot/actions/runs/34088350612/job/101636646204)
passed at 60073e1d680328df14c5ba40086b044af1f69982. The existing Rust tests,
fmt/clippy, all 54 ordinary invocations including warmup, and all six separate
observations passed. Its [canonical gate](https://github.com/mbbill/Whitefoot/actions/runs/34088350621)
also passed all fourteen Linux/macOS jobs. The [45 raw confirmation samples](../../experiments/io-completion-bench/rayon-baseline/resource-linux-2026-09-07.tsv)
retain their original order and CPU fields. Independently re-reading them
confirmed each plan's exact argument/batch mapping, all five alternating
orders, finite nonnegative times and every printed median/min/max/CPU summary.
The six observed stdout files matched the corpus bytes exactly. Ordinary
invocation stdout is checked by the runner but not separately retained;
its aggregate stderr records all six complete passes without failures.

| Batches | Form | Median wall, ms | Median user + system CPU, ms | Median CPU / wall |
|---:|---|---:|---:|---:|
| 1 | Rayon4, grain4 | 317.82 | 1261.07 | 3.970 |
| 1 | WF4, 12 stacks | 360.55 | 1290.84 | 3.580 |
| 1 | WF4, 1100 stacks | 385.37 | 1316.84 | 3.418 |
| 4 | Rayon4, grain4 | 1266.60 | 5035.66 | 3.976 |
| 4 | WF4, 12 stacks | 1316.29 | 5093.95 | 3.874 |
| 4 | WF4, 1100 stacks | 1342.03 | 5121.62 | 3.816 |
| 16 | Rayon4, grain4 | 5058.05 | 20130.51 | 3.980 |
| 16 | WF4, 12 stacks | 5136.13 | 20311.33 | 3.954 |
| 16 | WF4, 1100 stacks | 5169.81 | 20343.83 | 3.935 |

CPU is summed within each sample before taking its median, rather than adding
separately rounded median fields. Ratios below pair forms within the same
pass; a median ratio need not equal the ratio of the two reported medians.

| Batches | Paired form / Rayon4 | Wall ratio median [min, max] | CPU ratio median |
|---:|---|---|---:|
| 1 | WF4, 12 stacks | 1.1391 [1.1266, 1.1644] | 1.0245 |
| 1 | WF4, 1100 stacks | 1.2081 [1.1995, 1.2191] | 1.0440 |
| 4 | WF4, 12 stacks | 1.0402 [1.0035, 1.0534] | 1.0116 |
| 4 | WF4, 1100 stacks | 1.0574 [1.0243, 1.0649] | 1.0171 |
| 16 | WF4, 12 stacks | 1.0156 [1.0069, 1.0201] | 1.0090 |
| 16 | WF4, 1100 stacks | 1.0206 [1.0175, 1.0251] | 1.0101 |

The paired 1100-minus-12 stack differences were 22.31/27.20/29.60 ms wall
and 24.43/28.47/32.26 ms CPU at 1/4/16 batches. Every pair favored twelve
stacks. A roughly bounded absolute difference across sixteen times the work
is consistent with predominantly fixed capacity costs; it does not isolate
individual initialization syscalls or exclude scheduling effects. More
broadly, the original roughly 20% single-batch gap is not a sustained 20%
compute-parallelism loss in this workload. At sixteen batches the paired
gap is about 1.6% with twelve stacks, with CPU/wall near four for both forms.
Five passes on one hosted machine still do not establish a universal ranking
or prove that the remaining difference is entirely fixed.

All six Linux observations reported four scheduler threads, three spawned
workers, positive grants and zero `exhausted_compute`. That does not refute
the fifty turns observed on M1 or prove the ordinary Linux samples never took
that path. Each Linux observation also emitted a native ring report with
zero submissions, submission enters and completions. This is direct evidence
that the final output initialized a ring that carried none of this workload's
requests. It supports a separate existing `WF_IO_NO_NATIVE_RING` control as
the next attribution experiment, without selecting a future runtime default.

The host reported AMD EPYC 7763, four allowed vCPUs 0-3, two guest cores with
two SMT threads each and one NUMA node, Linux 6.17.0-1022-azure, Rust
1.98.0/LLVM 22.1.8 and Ubuntu Clang 18.1.3 for both the runner and actual WF
native link. No affinity or other host load was controlled. The
[complete artifact 10006272465](https://github.com/mbbill/Whitefoot/actions/runs/34088350612/artifacts/10006272465)
has ZIP SHA-256 `51672bddde34e7dc70bca7accb13e6f44145d1ad8360722c0393987669a58353`.
Twenty-six recorded artifact/source hashes were independently verified
against the retained files or exact git revision, along with the uploaded
Cargo.lock. The compiler executable's hash is recorded, but that executable
itself was not uploaded. Key retained SHA-256 values are:

| Retained file | SHA-256 |
|---|---|
| Ordinary WF parallel binary | `948ec36393b60f922cc4132c36b5cd1012fd4b8559a2523aa692a12f6e16db54` |
| Ordinary Rust/Rayon binary | `2a8289d8ca19fb74b0b035edb732cba533ab033c053379d36ae58c764b4d7d45` |
| Observed WF parallel binary | `414da59a4417a234fb73489fcef2fdc51a9a9f26b68bee57e4193c21d7467d31` |
| Generated WF parallel IR | `111320f992b02a384d5eb0c7705f67dbeda74c9bf42685e06bb362836588b7c8` |
| Original resource.tsv | `6c0b079bd1e726fdcf892e5e8b55d26e12a1e73540cbe17a1393cc5e9f401805` |

## Forty-sixth experiment: observe aggregation in the native send regression

Experiment38 measured a large regression when the 8 KiB provided-buffer
reference attempts an immediate send for each receive. This bounded follow-up
uses the exact 64-peer × 64 KiB echo cell, 500 round trips per peer, with the
unchanged `netload` payload sequence and full byte oracle. It keeps all four
8/64 KiB pure-ring/inline native forms, one server worker on one logical CPU,
and one client worker on a different physical core. It changes no native
default, client policy or compiler/runtime interface.

`scheduler-uring-diagnostic` selects `URING_DIAGNOSTIC=1` in the existing
native harness. Three recorded passes after one warmup yield twelve observed
records. Each record must receive and echo exactly 2,097,152,000 bytes. All
existing native stream qualification remains, including the eight ASan/UBSan
size/send-policy/worker configurations. Instrumented metadata goes to
`uring-diagnostic.tsv`, with counters in `uring-diagnostic-counters.tsv` and
separate per-pass observer/client/resource files. The diagnostic does not
publish an ordinary throughput ranking or add resident-memory measurements.

Existing counters provide receive CQEs/bytes, successful ring-send CQEs,
total sent bytes, immediate calls/bytes, buffer exhaustion and maximum queued
buffers. Observed builds additionally count each ring-send request's gathered
bytes and iovecs, each immediate call's requested bytes/iovecs, and immediate
successes, short transfers and EAGAIN results. Requested bytes may count a
short-send suffix more than once; completed bytes count bytes actually moved.
Consequently the following quantities answer separate questions:

| Quantity | Derivation |
| --- | --- |
| Received bytes per CQE | `receive_bytes / receives` |
| Ring bytes actually sent per successful CQE | `(send_bytes - inline_bytes) / sends`, when sends are nonzero |
| Requested ring aggregation | `ring_requested_bytes / ring_requests` and `ring_requested_vectors / ring_requests` |
| Immediate bytes actually sent per call | `inline_bytes / inline_attempts`, including zero-byte unsuccessful attempts |
| Requested immediate aggregation | `inline_requested_bytes / inline_attempts` and `inline_requested_vectors / inline_attempts` |
| Immediate fallback evidence | Successful, short and EAGAIN counts, plus any following ring requests |

The counter checks require matching transfer totals, one positive ring-send
CQE per completed ring request, requested bytes no smaller than actual bytes,
and consistent call/vector/result counts. They do not assume a requested
vector is fully transferred or that TCP preserves application message
boundaries. A difference in gathered bytes or call counts can support an
aggregation mechanism; proving packetization itself needs kernel or packet
evidence. Observation overhead can perturb the traffic and its timings, so
these records are not substitutes for experiment38's uninstrumented samples.

All eight strict Linux-musl size/send/observation builds pass locally. For
each of the four normal builds, optimized LLVM IR is identical before and
after these counters once source-file identity is excluded and debug info is
disabled. Shell parsing, Make dry-run, YAML parsing and diff checks pass.
The native run and audit below complete this bounded diagnostic on the
independent `codex/io-uring-aggregation-diagnostic` branch; measured remote
`0ebe924b` remains unchanged.

The exact measured revision is
[`f6ff40f740b90fbe5e8d25a0352d4dfce9257cbc`](https://github.com/mbbill/Whitefoot/commit/f6ff40f740b90fbe5e8d25a0352d4dfce9257cbc).
[Run 34088489262](https://github.com/mbbill/Whitefoot/actions/runs/34088489262),
measure job 101637039662, succeeds. Artifact `10006491666`
(`io-scheduler-allocator`) has SHA-256
`9955f1598c320b53ae8845a60ee1d8ff9f0951569fb08ff2a073b1c733f5db18`.
Its twelve metadata rows, twelve counter rows, all per-pass raw observer/client
files and resource records were reconciled. All have exactly 32,000 completed
round trips and the required byte totals. The measured host is an EPYC 9V74 VM,
Linux 6.17.0-1022-azure, Clang 20.1.2, glibc 2.39. Topology identifies CPU 0
(server) and CPU 2 (client) as different physical cores; each side has one
worker and one permitted logical CPU. SQPOLL remains excluded, THP disabled
and allocator top padding zero. Every observed server reports 4,096 ring
entries and exactly 2 MiB of provided payload storage: 256 × 8 KiB or
32 × 64 KiB. This is a new VM cohort; the matching processor model does not
make it pairable with experiment38.

The measure job also passes all eight native ASan/UBSan stream configurations
and their counter checks. The pressured inline tests actually encounter short
sends and EAGAIN before successful ring fallback; all four-connection streams
retain the complete 8 MiB byte oracle. These are separate from the unpressured
echo diagnostic below, where neither inline policy needs fallback. Linux and
Windows completion-host qualification also passes at this revision. Both
canonical scheduler jobs pass, including Linux job 101637040328 in
[gate run 34088489263](https://github.com/mbbill/Whitefoot/actions/runs/34088489263).
Two unrelated gate jobs remain queued when these results are recorded; this
is not a claim that the entire gate has completed.

The following are the **twelve raw counter records**, with constant receive
and send totals of 2,097,152,000 bytes per record omitted from the table.
“Send CQEs” counts positive ring-send completions; receive counts likewise
exclude terminal/error CQEs. Inline bytes are actual completed bytes, not the
sum of requested suffixes. Queue depth counts owned queued buffers, including
the prefix of any active send.

| Pass | Form | Receive CQEs | Send CQEs | Inline calls | Inline bytes | Max queue | ENOBUFS CQEs |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 0 | 8 KiB ring | 256000 | 67602 | 0 | 0 | 8 | 59027 |
| 0 | 8 KiB inline | 256000 | 0 | 256000 | 2097152000 | 1 | 0 |
| 0 | 64 KiB ring | 42889 | 42889 | 0 | 0 | 2 | 6274 |
| 0 | 64 KiB inline | 43389 | 0 | 43389 | 2097152000 | 1 | 0 |
| 1 | 8 KiB ring | 256000 | 67657 | 0 | 0 | 8 | 59675 |
| 1 | 8 KiB inline | 256000 | 0 | 256000 | 2097152000 | 1 | 0 |
| 1 | 64 KiB ring | 44207 | 44206 | 0 | 0 | 3 | 1738 |
| 1 | 64 KiB inline | 41778 | 0 | 41778 | 2097152000 | 1 | 0 |
| 2 | 8 KiB ring | 256000 | 67640 | 0 | 0 | 8 | 58959 |
| 2 | 8 KiB inline | 256000 | 0 | 256000 | 2097152000 | 1 | 0 |
| 2 | 64 KiB ring | 38938 | 38937 | 0 | 0 | 3 | 18172 |
| 2 | 64 KiB inline | 39869 | 0 | 39869 | 2097152000 | 1 | 0 |

Every inline call succeeds completely: all six records have zero short sends,
zero EAGAIN and zero ring fallback. Each has one requested iovec per inline
call, and requested bytes equal actual bytes. Every pure-ring record also
has requested bytes equal actual sent bytes and one positive send CQE per
request; short-send retries do not explain its aggregation. The 8 KiB ring
records each request 256,000 total iovecs. The 64 KiB ring records request
42,889, 44,207 and 38,938 iovecs respectively.

Derived values below are median [minimum, maximum] across the three records.
A ring request is a send operation, not an individual submission syscall:
multiple operations can share one `io_uring_enter` call.

| Form | Received bytes / CQE | Actual bytes / send operation | Requested iovecs / send | Send operations / round trip |
| --- | ---: | ---: | ---: | ---: |
| 8 KiB ring | 8192 [8192, 8192] | 31005 [30997, 31022] | 3.7847 [3.7838, 3.7869] | 2.1138 [2.1126, 2.1143] |
| 8 KiB inline | 8192 [8192, 8192] | 8192 [8192, 8192] | 1 [1, 1] | 8 [8, 8] |
| 64 KiB ring | 48897 [47439, 53859] | 48897 [47440, 53860] | 1.00002 [1, 1.00003] | 1.3403 [1.2168, 1.3814] |
| 64 KiB inline | 50198 [48334, 52601] | 50198 [48334, 52601] | 1 [1, 1] | 1.3056 [1.2459, 1.3559] |

This establishes an application-level aggregation difference. Both 8 KiB
forms receive exactly eight buffers per round trip. Pure ring's one pending
send permits later receives to accumulate; subsequent vectored sends average
3.785 buffers. Immediate success empties the queue on each receive, resulting
in eight separate `sendmsg` calls per round trip. The inline/ring send-operation
ratio is 3.7847 [3.7838, 3.7869]. It removes the ring-send CQEs and buffer
exhaustion yet performs substantially more sends. For 64 KiB, each receive
already carries roughly 47--54 kB, and both forms send almost every received
buffer separately; the same aggregation penalty is absent. This supports a
specific implementation mechanism, not a conclusion that immediate syscalls
or io_uring are inherently unsuitable.

The observed timing reproduces the large 8 KiB regression: median rate
27,760 → 11,579 trips/s and process CPU 35.94 → 86.25 us/trip. Paired rate
ratios are 0.4171 [0.4167, 0.4214], CPU ratios 2.400 [2.379, 2.400].
64 KiB paired rate ratios span 0.9815--1.0324. These instrumented figures are
context only, with three samples and process-lifetime centisecond CPU
accounting; experiment38 retains the uninstrumented timing evidence.
Client exchange CPU/wall is 0.884--0.886 for 8 KiB ring, 0.650--0.652 for
8 KiB inline, and 0.987--1.000 across 64 KiB forms. Whole-client CPU saturation
is therefore unsupported as the cause of this particular 8 KiB slowdown,
while the 64 KiB cohort still cannot establish a server ceiling.

The evidence counts **send operations, not TCP packets**, and does not assign
the entire CPU difference to syscall entry cost: kernel transmission,
packetization and scheduling effects remain unmeasured. A discriminating
future implementation would defer an immediate send until a bounded completion
batch has been collected, preserving stream ordering, short-send ownership
and fairness without waiting for an application message boundary. It would
need independent stream qualification followed by the same counters and
uninstrumented comparisons. No default or such optimization is changed here.

## Forty-seventh experiment: control unused native ring initialization

Experiment 45's generated WF layout has no concurrent I/O workload, yet its
final `write_once` requires the completion bridge. All six Linux observations
reported an initialized native ring carrying zero submissions, submission
enters and completions. The same panel's large short-run gap mostly amortized
away with longer work. A native-ring policy control can now test one candidate
fixed cost while retaining the algorithm and actual output path.

`make rayon-ring-bench ROUNDS=5 WARMUP=1` reuses `rayon-bench.sh` and its
ordinary C runner. It fixes WF_WORKERS=4, WF_STACKS=12 and Rayon width/grain
4/4. It pairs `WF_IO_NO_NATIVE_RING` unset and exactly `1` at one and sixteen
batches, alongside the ordinary Rayon reference at each batch count. The
harness first removes any inherited value, then sets `1` only for the disabled
row's child. Both WF rows execute the identical normal compiler-produced
binary. The existing runtime setting skips ring startup and keeps the typed
adapter; no runtime source, language rule, computation or output call changes.

The six forms share one forward/reverse plan. One warmup and five measured
passes produce thirty ordinary samples, with whole-process wall and child
user/system CPU including startup and output. The runner validates each
checksum after trimming trailing CR/LF; it does not archive those stdout
files. Four separate untimed `WF_SCHED_OBSERVE=1` executions archive and
exactly compare their output. On Linux, each default observation must have
one ring report with zero submissions/enters/completions, and each disabled
observation must have none. Both retain four scheduler threads, three spawned
workers and positive grants. A Linux host that cannot initialize the default
ring fails this particular experiment's qualification rather than silently
measuring two fallback rows. macOS can check command and checksum wiring but
does not qualify the native-ring distinction.

This is an attribution control, not a selection of a future runtime policy.
Disabling native ring availability also selects the existing adapter policy;
the paired difference is not a direct timer around io_uring_setup or a proof
that every affected instruction belongs to ring initialization. No native
network throughput claim follows from a ring that carried zero requests.
The comparison retains total CPU accounting, the waiting Rayon caller,
host/topology, actual compilers, source/binary hashes, plans, raw samples and
separate observations as in experiment 45. Neither CPU affinity nor host
dedication is assumed, and no perf capture is mixed into ordinary timings.
The isolated `codex/io-cpu-ring-controls` branch reuses the bounded CI resource
job while experiment 45 remains frozen at 60073e1d.

The local M1 one-pass smoke passed all six ordinary checksum validations and
all four separate observed stdout/counter checks. Its retained plan and raw
order were independently checked, including both argument counts and the
per-child native-ring override. Bash syntax and workflow YAML also passed.
These are command/protocol qualifications; the M1 has no native Linux ring,
and its shared-host timings are not performance evidence for this control.

The [Linux ring-control job](https://github.com/mbbill/Whitefoot/actions/runs/34090258664/job/101642113941)
passed at e71ec2781142b0b6f3dc8619dcd19a9541c0f9c9. The existing two Rust tests,
fmt/clippy, all 36 ordinary checksum checks including warmup and all four
separate observed checks passed. The [thirty original raw samples](../../experiments/io-completion-bench/rayon-baseline/ring-linux-2026-09-07.tsv)
were independently audited for exact plan/argument/environment mapping,
per-pass forward/reverse order, finite nonnegative times and all summary
statistics. Ordinary stdout remains runner-validated with trailing CR/LF
normalization; each separate observed output was independently compared
including its newline.

| Batches | Form | Median wall, ms | Median user + system CPU, ms | Median CPU / wall |
|---:|---|---:|---:|---:|
| 1 | Rayon4, grain4 | 354.26 | 1406.61 | 3.973 |
| 1 | WF4/12, default ring policy | 398.22 | 1442.17 | 3.625 |
| 1 | Same WF binary, native ring disabled | 397.82 | 1442.56 | 3.619 |
| 16 | Rayon4, grain4 | 5653.93 | 22472.02 | 3.977 |
| 16 | WF4/12, default ring policy | 5757.74 | 22717.04 | 3.949 |
| 16 | Same WF binary, native ring disabled | 5753.21 | 22710.95 | 3.948 |

| Batches | Paired comparison | Wall ratio median [min, max] | CPU ratio median |
|---:|---|---|---:|
| 1 | WF default / Rayon | 1.1241 [1.1182, 1.1390] | 1.0255 |
| 1 | WF disabled / Rayon | 1.1217 [1.1185, 1.1521] | 1.0252 |
| 1 | WF default / WF disabled | 0.9997 [0.9833, 1.0034] | 0.9999 |
| 16 | WF default / Rayon | 1.0166 [1.0144, 1.0229] | 1.0102 |
| 16 | WF disabled / Rayon | 1.0160 [1.0145, 1.0214] | 1.0106 |
| 16 | WF default / WF disabled | 1.0010 [0.9984, 1.0039] | 1.0003 |

Default-minus-disabled paired median differences were -0.123 ms wall and
-0.133 ms CPU at one batch, and +6.036 ms wall / +7.812 ms CPU at sixteen.
The direction changed across passes at both batch counts. This panel does
not support the hypothesis that unused ring initialization explains the
remaining roughly 40 ms short-run difference. The init path is real, but
its removal produced no stable material improvement here. The tiny observed
differences do not justify selecting a runtime default or a precise bound on
ring setup cost; other serial startup, computation, final joins, output or
shutdown costs still need attribution before another implementation change.

Mechanism qualification succeeded rather than silently comparing fallback
paths: both default observations emitted one ring report with zero
submissions/enters/completions; neither disabled observation emitted a ring
report. All four had four scheduler threads, three spawned workers, positive
grants and zero no-target compute-join turns. These are separate observed
executions, not scheduler traces of the ordinary timing samples.

The host reported AMD EPYC 9V74, four allowed vCPUs 0-3, two guest cores with
two SMT threads each and one NUMA node, Linux 6.17.0-1022-azure, Rust
1.98.0/LLVM 22.1.8 and Ubuntu Clang 18.1.3 for both the runner and WF native
link. This differs from experiment 45's reported CPU model; comparisons above
pair samples within experiment 47, not absolute times across those hosts.
The ordinary WF/Rayon binaries, observed WF binary and generated IR are
byte-identical to experiment 45's retained files. Twenty-six available
source/artifact hashes and the lock file were independently verified against
e71ec278; the compiler executable remains recorded by hash but not uploaded.

The [complete artifact 10006697428](https://github.com/mbbill/Whitefoot/actions/runs/34090258664/artifacts/10006697428)
has ZIP SHA-256 `26fc10efebf6848df0cccd8f9930c1b593fd423816f762dfeffa348aa419d96c`.
Its original resource.tsv has SHA-256
`bbd6a32cd57bf2f9b97af8c4163b706d621852f0203bda22de441c71b8fe9021`.
Plans, host metadata, commands, ordinary binaries and all separate output
and observation files remain there. No performance-policy change follows
from this negative attribution result.

### Remaining process costs: source and old timeline audit

This audit uses the unchanged runtime sources at e71ec278 and the retained
0d15c7cc artifact linked in experiment 45. The old perf executions have four
batches and 1100 WF stacks; they are not traces of experiment 47's one-batch,
twelve-stack samples. No runtime policy or timing harness changes follow from
this audit alone.

The experiment 47 one-batch paired WF-default-minus-Rayon differences are
43.963 ms wall and 35.857 ms aggregate CPU. The CPU difference divided by four
is only 8.964 ms, so extra CPU work at full four-way utilization does not
account for the entire wall difference. This arithmetic does not locate a
serial phase: waiting, runnable delay, serial CPU work and imperfect overlap
can all contribute. The longer samples and the existing four-batch profile
already caution against assigning the whole difference to sustained parking.

| Phase | Direct source or retained-trace evidence | What remains unknown |
|---|---|---|
| Core initialization | `sched/entry.c::wf__sched_start` calls `wf_sched_init` before the program body. Default `sched/core.c` clears the complete core, reserves all requested stacks, initializes every stack header/context, and initializes every configured lane's slots. | The old trace has no function boundary around this work. |
| Worker startup | The first acquisition starts three detached pthreads, reserves their pool stacks and yields until all report ready. There is no fixed rendezvous deadline. Old WF PID 4022 first runs at 386.379580 and the last worker forks at 386.400895: 21.315 ms wall, including 19.073 ms of main-thread CPU. | This interval begins before exec completes, also includes 1100-stack setup, and is not a direct pool-start timer. Several short D-state switch-outs are visible; their cause is not identified. |
| Computation and final output | Both kernels retain 800 full-table and 800 prefix-table tree folds per batch, resetting the same seed each batch and preserving sibling reduction order and all node writes. The final generated IR calls `wf__completion_file_write_submit`, then `wf__completion_file_join`, once. | Existing symbol samples cannot separate the last compute join, final output helper, or time until the body returns. |
| Return and shutdown | `wf__sched_entry_body` posts status only after the complete body returns. Only scheduler thread zero returns to its original host stack. Bounds restoration uses cached host bounds. POSIX workers are detached, with no runtime join; `wf_bridge_shutdown` leaves descriptors/mappings to the kernel while that pool is running. | There is no existing `exit_group` timestamp to separate user cleanup from kernel cleanup. |

The old WF scheduler trace is particularly informative at the end. Workers
4023 and 4025 finally switch out with X at 387.714376, and worker 4024 at
387.714377. Main 4022 runs continuously from 387.714064 until its final Z
switch at 387.729067. Thus the interval from the first final worker exits to
main's final switch is **14.691 ms continuously on CPU**, with the last worker
gone after its first microsecond. That interval is not parking or a timed
condition wait. Rayon caller 4003's last run interval is only
384.656045..384.656176, while its workers finish at
384.656162/.656168/.656196/.656460. These are separate instrumented executions,
not paired timing samples or proof that the same tail exists with twelve
stacks.

The CPU-clock captures cannot close that attribution gap. WF PID 3874's first
layout sample is 348.265825, and its last layout sample at 349.602022 is also
its last recorded sample. Twenty-eight samples precede layout: twenty-five
unresolved kernel samples, two libc samples and one `__mprotect`, totaling
28.028 ms of sampled CPU. There is no recorded post-layout CPU sample from
which to classify the final teardown. In
[upstream Linux v6.17 `do_exit`](https://github.com/torvalds/linux/blob/v6.17/kernel/exit.c#L842),
task perf events are removed before address-space and file cleanup. This
ordering explains why task-local CPU sampling can miss late exit work while
a system-wide scheduler trace continues to see it. The exact Azure patchset
has not been audited, and these CPU/scheduler captures are different
executions; neither missing symbols nor missing late samples identify the
actual cleanup routine.

The sequential controls also constrain attribution. The retained WF-seq ELF
enters `wf__floor_run` and the same core initialization. With the recorded
unset environment and four online CPUs it configures four lanes and twelve
stacks, but no parallel acquisition starts workers. Its old scheduler PID
3924 has no child thread, as does Rust-seq PID 3909. Their ordinary one-batch
median walls are 1177.291 and 1170.690 ms, respectively, and aggregate CPU
medians are 1177.051 and 1170.495 ms. Therefore the full-core clear is a real
removable cost, but it cannot simply be declared the entire roughly 40 ms
parallel difference. The sequential scheduler lifetimes are continuous
computation without a final function marker; their last switch intervals do
not independently measure teardown.

No fixed millisecond sleep was found on the successful core path. The
startup rendezvous yields until readiness; idle parks use completion/condition
events. The bridge's bounded join look is 10 microseconds, followed by event
waiting, not a 40 ms delay. `wf_floor.c` contains a fallback pthread join when
core entry fails, but the qualified core executions do not take that fallback.
This excludes an obvious written fixed delay, not all scheduler waiting.

### Minimal next control and coarse trace proposal

The stronger first policy control is `WF_SCHED_INIT_USED_LANES=1`, holding
`WF_SCHED_COMPACT_STACKS=0`. The retained Linux ELF identifies
`wf__sched_core` as exactly 20,478,544 bytes; `wf_sched_init` passes that same
size to memset. Its lane loop uses stride 0x4e020, or 319,520 bytes, so
clearing four rather than sixty-four skips 19,171,200 bytes
of unreachable lane storage without changing the core layout, stack geometry
or hot-path algorithm. Experiment 8 qualified this implementation policy;
the current candidate still needs its existing completion checks rerun.
The expected effect concerns touched memory at initialization and potentially
cleanup, not a prediction of the full observed wall difference.

Compact stacks change a different mechanism: they add 2048 aligned 128-byte
header cells to the core and defer raw context preparation to the first free
list pop. At twelve reserved stacks their immediately avoidable raw touches
are small compared with the unused lanes, and some may be needed during the
same computation. They also change a first-use path. Defer that axis while
isolating used-lane initialization; the 1100-stack evidence remains a separate
reason to revisit it. Neither policy's default changes.

The proposed ordinary panel fixes four WF workers, twelve stacks, the default
ring setting, and frozen Rayon width/grain 4/4. At batches 1 and 16 it compares
the ordinary compiler WF binary, a manually linked same-IR default control,
a same-IR used-lanes candidate, and Rayon. One warmup plus five alternating
passes gives forty recorded samples. The manual default retains the same
runtime source units, `/usr/bin/clang`, flags and link order as whitefootc;
retain both its full hash and its code/layout comparison with ordinary output.
This control prevents a manual-link difference from being credited to the
policy. Keep exact source/IR/tool hashes, ordinary runner checksum validation,
and separate exact-output/counter qualification for each storage form. Timing
contains no probes or observer, and no timing threshold becomes a gate.

A separate one-batch trace cohort should run the ordinary WF4/12, used-lanes
WF4/12, WF-seq and frozen Rayon4 forms twice in opposite orders. Use the
existing checksum runner as the traced command so collection outlives child
exit and includes the runner's successful `wait4` return. Record system-wide
`sched_process_exec`, `sched_process_fork`, `sched_process_exit`, `sched_switch`
and `sched_waking`, plus syscall entry/exit for `write` and `wait4`, and entry
for `exit_group`. Retain tracepoint schemas, actual PIDs/TIDs, binaries and
loss/error reports. `sched_process_exit` alone is not a completed-teardown
marker; final X/Z switches and parent reap delimit the later interval.

Add only entry uprobes at existing WF symbols: `wf__floor_run`,
`wf_sched_init`, `wf_sched_run`, `wf__main_body`,
`wf__completion_file_write_submit`, `wf__completion_file_join`, and
`wf_sched_post_status`. All are present in the retained ordinary ELF; verify
each new binary's actual symbols before registration. Use entry events rather
than a function-return pairing assumption across migrating stackful calls.
Do not probe recursive layout or the hot compute-join path. Main-thread
`wf_sched_run` entry follows successful initialization; worker entries plus
fork events bound worker arrival without relying on the inlined startup
helper or claiming an exact timer around its readiness loop.
Submission-to-status-post brackets final output and body cleanup; status-post
to exit-group brackets return/user shutdown; exit-group to final switch/reap
tests the kernel-exit hypothesis directly. The pre-submit body interval still
includes input construction and all final compute joins, so it is not named
pure arithmetic time. Missing required events or loss make attribution
incomplete and must not be silently replaced by the old samples.

This bounded pair of measurements tests a concrete removable footprint while
locating any residual startup or teardown cost. If the policy does not move
the relevant phase, retain that negative result before selecting another
runtime change. The trace is diagnostic evidence, not a replacement for the
ordinary paired performance panel.

## Forty-eighth experiment: batch ready continuations before target progress

Experiment 44 removes most coordinator context switches at occupied small
messages, but the owner form still spends about 37% more server CPU per trip
than the existing WF callee form at 64 peers. Its current `probe_take_ready`
flushes/reaps target I/O before every ready dequeue. The target progress
implementation first kicks deferred SQEs, so one-at-a-time progress can lose
submission aggregation even when other continuations are already ready.
This is a mechanism hypothesis selected from code and the remaining measured
loss, not a claim that submission counts were already measured in experiment 44.

`WF_CONTINUATION_PROGRESS_BATCH` selects a bounded owner-ready budget, default
one, accepted range 1-64. A value greater than one requires owner progress.
The comparison freezes one versus thirty-two rather than tuning from these
confirmation samples. After explicit target progress, the sole owner may
resume up to that many ready waiters before its next explicit progress call.
If the ready queue empties first, it immediately progresses again; it never
waits for a batch to fill. Once the budget is spent, progress occurs even if
ready work remains. The existing bounded new-task queue keeps its precedence.

The pending/ready locks, registration handshake, completion publication,
helper behavior, wake notifications, source bytes, frames, window and buffer
ownership are unchanged. Target progress still runs outside the coordinator
lock. Before a park, the owner captures the epoch, progresses and checks the
queue under its lock, retaining the helper-publication lost-wake protection.
A queued waiter remains owned until dequeue, and source resumes only after
publication/progress returns. No source-level preemption, cancellation,
worker identity guarantee or new `wf__par_*` interface follows from this flag.

The default one preserves the preceding owner's progress frequency. The
same ordinary binary implements the threaded control, owner-one and
owner-thirty-two; reports and sanitizers are disabled during timing but the
existing coordinator counters remain in all three. Their observer reports
include actual owner/batch settings. A separate
`WF_CONTINUATION_REPORT_BRIDGE=1` reads already-maintained bridge counters at
completion; it adds no per-operation counting. Linux preflight requires a
native ring, matching completed submissions, positive submission enters,
expected native socket routes and complete task retirement. It does not
assert that larger batches must produce fewer enters or better performance.

`CONTINUATION_SCREEN=3 NATIVE_BASELINES=1 EXPERIMENT=allocator` uses the
existing `scheduler-bench.sh combine` caller. It retains the previous eleven
forms and adds `wf-coro-batch32`: five WF candidates and seven fixed native
controls, five echo cases, seven alternating passes after two complete
warmups, yielding 420 ordinary timing rows and 108 live-memory snapshots.
The resource and byte contracts are the same split1 panel: one server CPU,
one client CPU on the other physical core, 1024 staged slots, exact payload
verification, THP disabled and explicit allocator setting. Threaded/owner
qualifications plus the common 2 MiB partial-send/backpressure/half-close
oracle run before timing for all three policies. Observation records remain
separate from ordinary timings; sample dispersion and client occupancy are
retained regardless of which form wins.

Local M1 ASan/UBSan and ThreadSanitizer qualifications pass the generated
file, pipe, recursive and TCP outcomes at all three settings, including
four-task fanout and twelve-task/four-slot retirement. The independent nested
C++ before-arm/during-arm and loan-drain fixtures still pass. These helper
runs establish local safety/behavior evidence, not Linux syscall behavior or
a performance result. The ordinary same-binary twelve-task/four-slot checks
and common four-peer 2 MiB stream oracle also pass all three policies, with
all tasks completed and retired. The isolated Linux branch is
`codex/io-continuation-batch`.

### Linux batching confirmation at fc69af15

Frozen revision
[`fc69af155ff6fe5cae535a2ca663f98004c87472`](https://github.com/mbbill/Whitefoot/commit/fc69af155ff6fe5cae535a2ca663f98004c87472)
completed [run 34093974793](https://github.com/mbbill/Whitefoot/actions/runs/34093974793),
including measurement job 101653329137, generated native/helper qualification
and Windows placement checks. Its only difference from the preceding tested
`f2ac5bc6` is two workflow comments to trigger the dedicated branch run.
Artifact `10008989325`, `io-scheduler-allocator`, has SHA-256
`e99d91e257caaeaecc548c65129eabeb6257df9edf4b2d8f96c79ca998b69cf3`.
The ZIP retains all ordinary samples, observations, generated IR, qualification
logs and residency maps. Independent audit reconciles all 420 timing rows in
60 seven-pass groups against their raw client and process-resource records,
exact trip counts, rate rounding intervals and alternating form order.
Every ordinary server stdout/stderr and client stderr is empty. All 108
snapshots reconcile with their smaps totals; THP is disabled, huge-page and
swap totals are zero, and every retained process status has CPU mask 0.
This panel does not retain a separate per-thread affinity census.

The host is an Intel Xeon Platinum 8370C VM running Linux 6.17.0-1022-azure,
Clang 20.1.2 and glibc 2.39. Its four logical CPUs represent two physical
cores; the server uses CPU 0 and the client CPU 2. The allocator panel sets
top_pad to zero. All three continuation modes use the same generated binary,
1024 staged slots and private initialized 64 KiB source buffers. Neither
the absolute rates nor the size of this panel's owner/threaded difference
should be transplanted to experiment 44's different AMD host.

Rates below are medians in verified round trips per second. Ratios are the
median and full range of seven same-pass batch32/owner-one comparisons,
not a ratio of independently selected samples.

| Peers / bytes | Owner-one rate | Batch32 rate | Paired rate ratio | Paired server CPU/trip ratio | Paired p99 ratio |
| --- | ---: | ---: | ---: | ---: | ---: |
| 1 / 64 | 30184 | 30116 | 0.999 [0.958, 1.036] | 1.067 [1.000, 1.071] | 1.064 [0.912, 1.255] |
| 4 / 64 | 144450 | 154859 | 1.075 [1.065, 1.112] | 0.926 [0.889, 0.962] | 0.959 [0.855, 0.980] |
| 64 / 64 | 142075 | 183267 | 1.283 [1.254, 1.323] | 0.778 [0.761, 0.793] | 0.797 [0.572, 0.840] |
| 1024 / 64 | 112075 | 142080 | 1.271 [1.261, 1.296] | 0.792 [0.779, 0.801] | 0.816 [0.650, 0.968] |
| 64 / 65536 | 38081 | 42005 | 1.108 [1.066, 1.144] | 0.911 [0.880, 0.933] | 0.889 [0.660, 1.151] |

At 64 small-message peers, median server CPU falls from 6.953 to 5.391 us
per trip and p99 from 526 to 397 us. Context switches are already rare in
both owner forms: 0.000289 and 0.000234 per trip. This gain is additional to
the earlier handoff removal. At 1024 peers, CPU falls from 9.033 to 7.178 us
and p99 from 10308 to 8505 us. One peer has no stable rate benefit and retains
the occupied-versus-idle policy tradeoff; the threaded form's median there
is 40111 versus batch32's 30116. No default is selected from this screen.

Independent untimed reports provide mechanism evidence at 64 peers and
2000 trips each. All nine continuation preflight/stream observations have
the expected policy, balanced waiters, exact task retirement and native
accept/receive routes. Completed submissions equal submissions in each.

| Mode | Ring submissions | Submission kicks | Submissions/kick | Runtime parks |
| --- | ---: | ---: | ---: | ---: |
| Threaded | 125993 | 55606 | 2.266 | 115717 |
| Owner-one | 127932 | 127869 | 1.0005 | 581 |
| Owner-thirty-two | 128047 | 4031 | 31.766 | 14 |

The adapter's `submission_enters` counts calls to its nonempty submission
kick before any EINTR retries; it is not every io_uring syscall, completed
byte count or TCP packet count. Owner-thirty-two preserves approximately the
same number of ring operations while cutting these kicks by 96.8% in this
observation. That directly confirms recovered submission aggregation. The
four-peer, 80-trip preflight gathers only 2.75 submissions/kick, and the
fragmented 2 MiB stream gathers about one: the policy progresses immediately
when readiness runs out. It does not manufacture a full batch by waiting.
These observations are separate executions and their times never enter the
ordinary ranking.

The stronger candidate still does not lead the small-message native cells.
Batch32/callee paired rates are 0.940 / 0.937 / 0.987 at 4 / 64 / 1024 peers;
against the fixed 64 KiB io_uring control they are 0.844 / 0.920 / 0.822.
At 64 peers the seven native median rates span 196807-203013, versus 183267
for batch32. At 1024 they span 161029-175022, versus 142080. These ranges
describe the complete preselected native set rather than selecting one
winning implementation as a proven frontier. The client consumes roughly
0.93 CPU for batch32 at 64 peers and nearly 1.0 for the faster native forms.
At 1024 the corresponding values are about 0.86 and 0.97-0.98. Additional
client headroom has not been demonstrated.

The large-message cell is especially unsuitable for a peak-throughput claim:
client occupancy is about 1.0 CPU, and batch32's seven rates span 32819-44304,
with p99 spanning 1640-2875 us. Its paired rate exceeds the fixed 64 KiB ring
and shared-buffer epoll controls in all seven passes, but that is only an
end-to-end result under this client limit. No samples are discarded. The
64-peer small cell is steadier: batch32 spans 170712-188933 trips/s and
p99 384-451 us; its owner-one paired gain is positive in every pass.

Median live RSS for owner-one versus batch32 is 3104/3100 KiB at 64 small
peers, 12508/12508 KiB at 1024 small peers, and 5760/5260 KiB at 64 large
peers. The large-cell three-snapshot ranges overlap (5476-6124 versus
5204-5648); no storage-representation change follows from that difference.
The existing WF callee form uses 3664/15156/6156 KiB for those three cells;
the explicit private-calloc epoll control uses 2136/9852/4268 KiB. Shared
epoll and provided-buffer io_uring have different storage contracts and
must not be described as equal-capacity memory controls.

This experiment supports batching progress in an occupied continuation
executor without changing sequential source, ownership rules or function
signatures. It does not settle the remaining coordinator cost, idle policy,
multi-owner execution, cancellation or compute fairness. Those are separate
mechanism and workload questions; seven same-host samples cannot establish
that a fixed budget of thirty-two is optimal.


## Forty-ninth experiment: qualify a sequential Go net reference

The external comparison now has a concrete normal sequential-API candidate:
`go_echo.go` runs one goroutine per accepted connection, with a private
initialized 64 KiB buffer, sequential `TCPConn.Read` and ordered `TCPConn.Write`.
The CLI accepts a port, total connection count and `--threads` adapter argument;
that argument sets GOMAXPROCS, not the number of OS threads. Each connection
owns its buffer until all bytes returned by a read have been written.
Bytes returned together with EOF are written before normal completion.
A read/write failure closes the listener and every accepted connection, joins
all handlers and exits unsuccessfully. No fixture identity, payload value or
message-size recognition changes the algorithm. There is one listener, with
the standard runtime scheduling its goroutines across execution processors.

The official [Go release index](https://go.dev/dl/?mode=json) and
[release tag](https://github.com/golang/go/tree/go1.27.1) identify Go 1.27.1;
the tag resolves to `862c888e612ac346c7c4d99c9392bdfd265f33b0`. The local
Darwin/arm64 official archive SHA-256 was verified as
`ee215d57e0ec269c60cc9ceca68e6bda321ba9ee5afe24f4b0988703c2d87d12`.
Canonical scheduler jobs and the dedicated qualification job install exactly
1.27.1. `go-check` requires that readback and sets `GOTOOLCHAIN=local`, with no
automatic toolchain fallback. There are no external Go modules; release uses
`CGO_ENABLED=0`, while the separately identified race build enables cgo for
the race runtime. Build metadata is retained. Only a release build may enter
future timing.

The downloaded release sources were inspected. Its
[Linux netpoll implementation](https://github.com/golang/go/blob/go1.27.1/src/runtime/netpoll_epoll.go)
uses epoll, edge-triggered registrations and an eventfd wakeup; initialization
failure does not select a different backend. Its
[Unix descriptor implementation](https://github.com/golang/go/blob/go1.27.1/src/internal/poll/fd_unix.go)
attempts reads/writes directly and waits through the runtime poller on EAGAIN,
retaining a partial-write offset. Thus a sequential socket call need not
occupy an OS thread while waiting. The reference explicitly enables TCP_NODELAY
and disables keepalive to match the native echo socket policy. Qualification
sets SO_SNDBUF to 4096 and reads back effective send/receive buffers and the
enabled no-delay flag. Darwin returns a nonzero no-delay value of 4 locally;
the boolean check accordingly requires nonzero, not a Linux-specific value.
[Go TCP socket methods](https://pkg.go.dev/net#TCPConn.SetNoDelay) describe these controls.

`go-check` is wired into root `scheduler-experiment` and therefore `make check`.
All older stream modes remain enabled. Each release/race × GOMAXPROCS 1/4
combination runs four cases, sixteen total:

- A quiet ordinary echo with the same four independent 2 MiB streams, complete
  byte checking, delayed readers and half-close; stdout/stderr must contain
  only the fixture's successful result.
- The same stream oracle with runtime observations and the small send buffer,
  exercising the runtime's short-write/backpressure handling.
- A generic abortive reset while three other connections wait for input.
  The reset must terminate all four handlers and produce the expected nonzero
  process result; a partial response cannot be treated as success.
- Sixty-four live connections, each exchanging and validating 64 KiB before
  the live snapshot and orderly half-close.

The new optional `WF_BENCH_PROCESS_DETAILS=1` residency snapshot records
each enumerated live thread's status/stat, process status/stat/io, smaps,
descriptor fdinfo and namespace socket totals. Existing callers keep their
old snapshots. Linux Go qualification requires epoll fdinfo entries and checks
every captured thread's affinity against the requested server CPU. Kernel
and socket namespace totals include other processes, including the client;
they are context, not CPU or memory attributed to this server. These live
snapshots do not claim an atomic census or a lifetime thread peak.

The dedicated `codex/io-go-baselines` job fixes all server OS threads to one
logical CPU and the fixture to a different physical core. GOMAXPROCS 4 under
that mask deliberately qualifies oversubscription; it does not allocate four
CPU cores. GOMAXPROCS controls Go execution concurrency, while the OS mask
limits the entire process, including runtime/GC threads and syscall execution.
The [runtime documentation](https://pkg.go.dev/runtime#GOMAXPROCS) distinguishes
the explicit setting from defaults derived from affinity/cgroup limits.
The job records topology, build/compiler versions and runtime source hashes,
plus host/kernel counters before and after qualification. Canonical jobs
also run these correctness cases, without making an affinity/timing claim.

Observed startup/exit records include GOMAXPROCS, live runtime threads,
goroutines, GC policy/cycles, allocation/memory/stack counts and GC pause totals.
`ReadMemStats` flushes the memory snapshot. Runtime CPU-class metrics remain
their documented estimates, not replacements for OS process/kernel CPU
accounting; in particular their total is tied to GOMAXPROCS and elapsed time.
[Runtime metric definitions](https://pkg.go.dev/runtime/metrics) state these
limits. There is no forced GC, disabled GC, buffer pool or custom scheduler
policy in the reference.

Local M1 qualification passes all sixteen release/race cases with the exact
toolchain. Go vet, gofmt, strict C compilation and Linux cross-compilation are
checked. Native Linux affinity/backend/resource qualification is recorded
below. This promotes Go from an unspecified candidate to an implemented,
protocol-qualified row; it does not measure its speed or
claim that this idiom is Go's best possible tuning.

The first Linux qualification at `62a3e885247f3890bb2dab1cf6e788a2e019ddc8`
fails in the harness after successful quiet and observed release/P1 stream
checks, before reset/race/residency. In
[run 34092017030](https://github.com/mbbill/Whitefoot/actions/runs/34092017030),
job 101647323881, GNU Make preserves the backslash/newline inside the recipe's
single-quoted multi-line jq program; jq reports a parse error. The local
Make 3.81 run had accepted that recipe. Artifact `10007191004` retains the
partial evidence, SHA-256
`5821ab8b07c06ee2e296afdad5c2734b92944420fa30e3761fa1bf5363ccb5ed`.
The correction places the unchanged complete jq predicate in a non-recipe
Make variable, where continuations become spaces. No assertion or fixture is
removed or relaxed, and the Go implementation is unchanged. Linux qualification
must run to completion on the corrected revision before its evidence status
is promoted.


The corrected native Linux qualification succeeds at
[`ccf667fd077c2f4a1c7ada24f6ed6c3c0a3ed1de`](https://github.com/mbbill/Whitefoot/commit/ccf667fd077c2f4a1c7ada24f6ed6c3c0a3ed1de),
[run 34092404824](https://github.com/mbbill/Whitefoot/actions/runs/34092404824),
job 101648493665. Artifact `10007332754`, `go-reference-qualification`,
has SHA-256
`5fe56e4105a812530a77254e958c8b25a27db04ef0a741c2645be805b5c52853`.
Independent raw audit reconciles all sixteen case logs, release/race build
metadata, every socket-option record and all four live process snapshots.
The qualification host is an Intel Xeon 6973P-C VM on Linux
6.17.0-1022-azure. CPU 0 is the entire server mask; CPU 2 is the fixture on a
different physical core. Every runtime observation reads `NumCPU=1` and the
requested GOMAXPROCS. GC remains enabled at GOGC 100 with the default memory
limit and no GODEBUG overrides.

All four captured thread sets have mask `0` on every thread, with counts
matching their process status. Each snapshot has exactly one epoll descriptor
with 65 registered targets: the 64 active peers and the runtime wakeup.
All socket readbacks show TCP_NODELAY enabled and an effective 8,192-byte
send buffer for the requested 4,096 bytes. Reset cases report the intended
connection-reset write error and successfully close the three other waiting
handlers. No byte mismatch, unexpected output or race report occurs.

These are single live qualification snapshots after all 64 peers have
completed a byte-checked 64 KiB exchange and while they remain open.
Race instrumentation is included only to show its distinct resource cost;
it is not a performance reference.

| Build | GOMAXPROCS | Captured live OS threads | RSS KiB | PSS KiB | GC cycles at exit |
| --- | ---: | ---: | ---: | ---: | ---: |
| Release, cgo off | 1 | 3 | 13288 | 13280 | 0 |
| Release, cgo off | 4 | 3 | 15336 | 15328 | 0 |
| Race, cgo on | 1 | 5 | 41904 | 40288 | 1 |
| Race, cgo on | 4 | 7 | 42372 | 40770 | 1 |

GOMAXPROCS therefore does not name the thread count or multiply this job's
CPU allocation. Go heap counters alone also cannot describe this reference's
storage. Release process TotalAlloc at exit is only 238,640/288,272 bytes for
P1/P4, while its 64 simultaneously live private buffers have a 4 MiB source
payload budget. With this exact Go toolchain and Linux/amd64 target, compiler
escape diagnostics say the 65,536-byte allocation does not escape.
Disassembly of the **actual CI release artifact** places it in `main.echo`'s
stack frame (`SUBQ $0x10040, SP`) and explicitly zeroes 65,536 bytes with
`REP STOSQ` before the read loop. It is goroutine stack storage, not missing
payload memory or eliminated initialization. The exit stack count is after
handlers have finished; live smaps captures their retained physical footprint.
These four snapshots are neither lifetime peaks nor a cross-implementation
memory ranking, and they do not establish optimal Go storage.

The two audited runtime sources match the local verified toolchain:
`netpoll_epoll.go` SHA-256
`cd94172ee133e4e522a8091818c6d82c9fdb94a9c263d5004feee114f76f9624`
and `internal/poll/fd_unix.go`
`675f74e8cbfc73170e08b43947c131706cd59e498d722555fe3b793618f5f162`.
The row is now protocol-qualified on native Linux as well as local macOS,
with release/race separation and observed Linux backend/CPU constraints.
The canonical gate at this revision is still running when these results are
recorded. No Go timing panel has run, no existing native/WF cohort is replaced,
and there is no claim of spare generator capacity or maximal Go performance.

## Fiftieth experiment: configured-lane initialization and process boundaries

Experiment 47's source and timeline audit selected one existing candidate:
`WF_SCHED_INIT_USED_LANES=1`, with compact stacks kept at zero. This experiment
implements the bounded control described above without editing runtime
sources, emitted IR, the workload, the output path or either default policy.
It does not presume that the candidate removes the full short-run difference.

`make rayon-startup-bench ROUNDS=5 WARMUP=1` reuses `rayon-bench.sh` with
`RESOURCE_CONTROLS=3`. At each of one and sixteen batches it runs four forms:
normal compiler WF, the same IR manually linked with explicit default macros,
the same manual link with used-lanes set to one, and frozen Rayon width/grain
4/4. WF has four workers and twelve stacks; the harness removes any inherited
native-ring override. One warmup and five alternating passes yield forty
ordinary samples. No fresh grain calibration selects these controls.

The manual link reproduces whitefootc's C source order and actual
`/usr/bin/clang` selection, C11, pthread, O2 and math-library flags, feeding the
same generated IR on stdin. Commands and binary hashes are retained, along
with a byte-identity result for manual default versus compiler default and
Linux symbol/disassembly dumps for all three WF forms. Both default forms
remain in every pass regardless of that result. Before timing, the used-lanes
candidate passes the entire existing `completion-test`, including the core
enumerations; neither its scope nor its expected verdicts change.

Every ordinary invocation validates the existing checksum with the runner's
trailing CR/LF normalization. Additional manual one-batch output files are
compared including their newline. Four separate observer runs cross default
and used-lanes at both batch counts, require the expected macro values, four
scheduler threads, three spawned workers and positive grants, and retain
exact stdout plus no-target compute-join counts. There is still no peak live
stack counter. These observations are not measurements of the ordinary
samples' scheduling events.

`CPU_PHASE_TRACE=1` adds the separately maintained `rayon-phase-trace.sh`
caller after the complete ordinary panel. It first records the current Linux
kernel/perf version and actual available tracepoints, checks every required
event and each binary's entry symbols, and retains their schemas and ELF
segments. `perf probe` resolves file offsets as required by the
[Linux uprobe interface](https://www.kernel.org/doc/html/latest/trace/uprobetracer.html).
Only the seven coarse function entries identified above are registered, in
an invocation-specific group removed on exit. No function-return probe or
hot recursive probe is added.

The trace cohort is ordinary WF4/12, used-lanes WF4/12, sequential WF4/12
configured without any parallel acquisition, and Rayon4/grain4, each at one
batch in forward then reverse order. A privileged system-wide recorder runs
the checksum runner as the normal job user. Tiny exec-in-place wrappers retain
runner and child PIDs; the child's stdout still goes directly to the runner's
normal pipe. Recording outlives child exit and includes parent wait4 return.
The wrapper/recorder executions and all their timing rows live under
`phase-trace/`, never in `resource.tsv`.

The trace captures scheduler exec/fork/exit/switch/waking events, write and
wait4 entry/exit, exit-group entry, and the WF entry probes. It uses the
monotonic clock and retains raw perf data, nanosecond-decoded records,
commands, PID files, event definitions, recorder/decode stderr and basic
milestone checks. Missing permissions, kernel events, symbols, failed capture,
decode errors or loss markers produce explicit `status=incomplete`, preserving
the ordinary panel. `status=captured` means eight checksum-validated records
with necessary markers, not completed causal analysis: exact thread membership,
marker counts, the returned wait4 PID and final state transitions must still
be independently audited. A function-entry interval includes any intervening
code and scheduling; probes themselves can perturb the execution. Neither
that interval nor a profiled process wall is substituted into the ranking.

The isolated `codex/io-cpu-startup-controls` branch reuses the resource CI job
with a twenty-minute cap and the bounded panel above. Experiment 47 remains
frozen on its existing branch. Current native event availability and complete
trace qualification are Linux results to be established, not inferred from
the prior kernel version or from the local M1 smoke.

The initial M1 smoke passed both Rust tests, fmt/clippy, the complete candidate
completion suite, all eight ordinary commands and all six independent manual
or observer stdout files. Re-reading the plan/raw data verified executable,
environment and exact batch mapping. Both one-batch observations reported
zero no-target compute-join turns; at sixteen batches default/used-lanes
reported ten/three, respectively, with correct bytes and flags. Those are
separate observations, not an explanation of the ordinary timing rows.
The compiler/manual-default Mach-O files differ, while their retained complete
disassemblies match after removing only the file-heading lines. The local
shared-load timings are not ranking evidence. The optional trace path correctly
reported `incomplete` because this machine has no Linux tracepoints.
A separate local invocation of the actual launch wrapper ran WF-seq and
Rayon through the existing runner, preserving both checksum payloads and
distinct runner/child PID records. It also verified the runner's expected
progress stderr, which the trace qualification permits exactly. This checks
wrapper/pipe wiring, not Linux event collection.

### Fiftieth measurement: ordinary controls and a contaminated trace

The [Linux job](https://github.com/mbbill/Whitefoot/actions/runs/34093661039/job/101652368768)
passed at 8dd5b44a4e090601c276431ff4f83da55e443c43. The candidate completion
suite, Rust tests/fmt/clippy, forty-eight ordinary checksum validations
including warmup, two manual stdout checks and four observer checks passed.
The [forty raw ordinary samples](../../experiments/io-completion-bench/rayon-baseline/startup-linux-2026-09-07.tsv)
were independently re-read for every command/environment/batch mapping,
reverse/forward pass order and finite nonnegative CPU/wall values. Ordinary
stdout remains runner-validated after trailing CR/LF normalization; the six
separate retained stdout files were independently compared including newline.

| Batches | Form | Median wall, ms | Median aggregate CPU, ms | Median CPU / wall |
|---:|---|---:|---:|---:|
| 1 | Rayon4, grain4 | 274.497 | 1090.805 | 3.975 |
| 1 | Compiler WF4/12 | 310.030 | 1118.880 | 3.607 |
| 1 | Manual default WF4/12 | 309.883 | 1118.993 | 3.611 |
| 1 | Manual used-lanes WF4/12 | 313.448 | 1119.552 | 3.573 |
| 16 | Rayon4, grain4 | 4372.349 | 17430.688 | 3.987 |
| 16 | Compiler WF4/12 | 4454.889 | 17607.348 | 3.952 |
| 16 | Manual default WF4/12 | 4463.918 | 17608.316 | 3.947 |
| 16 | Manual used-lanes WF4/12 | 4454.994 | 17604.266 | 3.952 |

| Batches | Within-pass comparison | Wall ratio median [min, max] | CPU ratio median |
|---:|---|---|---:|
| 1 | Compiler WF / Rayon | 1.129864 [1.115232, 1.136003] | 1.025279 |
| 1 | Manual used-lanes / manual default | 1.011730 [0.985687, 1.030206] | 1.000500 |
| 1 | Compiler WF / manual default | 1.005970 [0.967859, 1.007037] | 0.999775 |
| 16 | Compiler WF / Rayon | 1.018947 [1.011446, 1.022726] | 1.010254 |
| 16 | Manual used-lanes / manual default | 0.998001 [0.996936, 1.001039] | 0.999744 |
| 16 | Compiler WF / manual default | 0.998126 [0.996570, 1.004517] | 0.999592 |

Compiler and manual-default Linux binaries are byte-identical. Their paired
differences therefore also expose sampling variation, not a link-policy
effect. Used-lanes-minus-manual-default median differences were +3.634 ms
wall / +0.559 ms CPU at one batch, and -8.924 ms wall / -4.507 ms CPU at sixteen.
The sign changes across passes at both lengths. This result gives no stable
material short-run benefit and does not select a new default. The established
footprint saving is a different measurement. All four observer executions
reported the requested storage flags, four scheduler threads, three started
workers, positive grants and zero no-target compute-join turns. Their rings
reported zero submissions, submission enters and completions.

The trace preflight succeeded on the actual Linux 6.17.0-1022-azure kernel
with perf 7.0.14: all requested tracepoints and twenty-one function-entry
probes were available. Eight captures passed the harness's basic marker/PID
checks, but the independent audit found a recorder defect. System-wide write
events include perf's own writes to its output, producing a large stream of
self-recorded writes. In the first ordinary capture they account for 335,616
of 338,471 decoded lines, about 99.2%. Ordinary WF body-to-submit intervals
are 430.523/426.745 ms in these traces, while the separate ordinary unprobed
whole-process median is 310.030 ms. The trace is therefore **not qualified
for performance attribution**, despite no reported LOST/throttling marker.
The preceding ordinary panel ran before probe registration and remains valid.

| Capture | Decoded lines | Recorder write entry + exit events | Adjacent exact duplicate switch lines |
|---|---:|---:|---:|
| Pass 0, ordinary | 338471 | 335616 | 1 |
| Pass 0, used-lanes | 316619 | 314094 | 0 |
| Pass 0, sequential WF | 949600 | 948170 | 0 |
| Pass 0, Rayon | 350258 | 317108 | 1 |
| Pass 1, Rayon | 342404 | 309684 | 1 |
| Pass 1, sequential WF | 948293 | 947088 | 0 |
| Pass 1, used-lanes | 315577 | 312666 | 1 |
| Pass 1, ordinary | 320194 | 317530 | 0 |

The four duplicate records have identical nanosecond timestamps, CPU and
complete switch payload, with no intervening transition. Raw files remain
unchanged. For diagnostic reconstruction only, removing that second identical
copy permits consistent running-state transitions throughout all eight target
thread sets. No further unmatched transition was found. The duplicate source
has not been isolated. Some final worker records have a decoded header PID
of -1 while their switch payload identifies the actual departing thread;
reconstruction uses those payload IDs rather than dropping the records.

Fork/exec membership confirms four target threads for each parallel WF,
one for sequential WF and a waiting caller plus four Rayon workers. Each
required single-occurrence WF marker has exactly one target hit, with one
run-entry per configured executing thread. Output/join/status-post may occur
on different worker IDs, as the runtime permits. All eight runner wait4 return
values exactly identify their respective children. Final X/Z transitions for
every target thread are present before the successful reaps in these captures.
The twenty-one invocation-specific probes were removed successfully.

| Diagnostic capture | Init to main run, ms | Submit to status post, ms | Status post to exit-group, ms | Exit-group to successful reap, ms |
|---|---:|---:|---:|---:|
| Pass 0, ordinary | 1.700 | 0.240 | 0.030 | 0.541 |
| Pass 0, used-lanes | 0.702 | 0.201 | 0.029 | 0.572 |
| Pass 0, sequential WF | 1.769 | 0.177 | 0.070 | 0.464 |
| Pass 0, Rayon | — | — | — | 0.188 |
| Pass 1, Rayon | — | — | — | 0.207 |
| Pass 1, sequential WF | 1.697 | 0.197 | 0.078 | 0.435 |
| Pass 1, used-lanes | 0.326 | 0.214 | 0.027 | 0.590 |
| Pass 1, ordinary | 1.664 | 0.222 | 0.031 | 0.728 |

For example, pass 0 ordinary has main Z at 823.136163934, workers X at
823.136164730/.136172136/.136646825, and runner wait4 returning child 4284 at
823.136669684. There is no old 1100-stack trace's 14.691 ms main tail in this
contaminated twelve-stack execution. This is a record-level observation, not
a claim that ordinary short-run cost has now been attributed or excluded.
All phase values above retain the observer's severe workload perturbation.

The correction is a bounded trace-only recapture using the exact archived
8dd ordinary binaries, checking their hashes before execution. Perf's native
[per-event `--exclude-perf` filter](https://man7.org/linux/man-pages/man1/perf-record.1.html)
can suppress recorder-issued write entry/exit events. Global scheduler events
must remain intact so switches from the recorder to a target are not lost.
The new audit must verify that recorder writes disappeared, independently
check state consistency and match successful reaps; absence of LOST reports
is insufficient. It will not rerun or replace these forty ordinary samples.

The host reported AMD EPYC 9V74, four allowed vCPUs 0-3, two guest cores with
two SMT threads each, one NUMA node, Rust 1.98.0/LLVM 22.1.8 and Clang 18.1.3
for both the runner and native WF link. No affinity or host dedication was
controlled. The [complete artifact 10008007005](https://github.com/mbbill/Whitefoot/actions/runs/34093661039/artifacts/10008007005)
has ZIP SHA-256 `a390db0c1c26664db02157e40c7e4da64c9f6a7d76257008a2e208681069168c`.
Thirty unique available source/binary/artifact hashes and the lock file were
independently verified against 8dd; the compiler executable remains recorded
by hash but not uploaded. The ordinary WF, Rust and IR hashes match experiments
45/47. The used-lanes binary is
`0be6cbbb5579df3aaca4c81d497101668060dc87b0cca68aadea72ef5569e5ae`;
the original resource.tsv is
`74471f805e030668ef72c4fff098839356cd46f4f770c3cc2cfa74939c93f9fd`.

### Trace-only correction

The `codex/io-cpu-phase-recheck` branch preserves 8dd's frozen branch and uses
a ten-minute CI job to retrieve artifact 10008007005 directly. It verifies
the recorded ZIP digest, extracts only the three WF forms needed for tracing,
the Rayon binary, runner, IR, lock file and original host metadata, and checks
all five executable hashes plus IR before execution. No current compiler
build, Cargo dependency fetch, ordinary calibration or timing panel runs on
this path. The original host description is labeled `source-8dd-host.txt`;
the new trace records its own host and collector revision separately.
Its topology and allowed CPU mask are captured on the new host; old ordinary
timings are not combined with new trace intervals across hosted machines.

The recorder uses an exec-in-place wrapper to retain its PID, adding native
[`--exclude-perf`](https://github.com/torvalds/linux/blob/v6.17/tools/perf/Documentation/perf-record.txt#L202)
immediately after each write entry/exit event selector.
Scheduler events retain their system-wide scope, including switches from perf
to a target. Each decoded capture must contain zero recorder-issued write
events, in addition to the existing milestone and checksum checks. Failed
recording now leaves any partial root-owned data and recorder PID readable by
the job user for artifact retention. No global trace filters or unrelated
probe registrations are removed.

The wrapper/filter checks do not replace independent event auditing: the new
capture must still be checked for exact duplicate records, actual exec/fork
membership, consistent switch transitions, complete final X/Z events and a
successful wait4 return naming the recorded child. The retained old record
already exercises the new recorder-write detector, which finds all 335,616
self-write events in pass 0 ordinary. A local selective extraction verified
every fixed ZIP member/hash used by the new workflow. The old contaminated
trace and valid forty ordinary samples remain unchanged and separately
linked above.

### Corrected trace audit at c81ac477

Frozen revision
[`c81ac477261fa69065caba44a37c54f4258c6c6d`](https://github.com/mbbill/Whitefoot/commit/c81ac477261fa69065caba44a37c54f4258c6c6d)
completes [run 34096629728](https://github.com/mbbill/Whitefoot/actions/runs/34096629728),
CPU job 101661622687. Artifact `10009398989`, `io-cpu-phase-recheck`, has
SHA-256 `f117d23ff93ed900b6418b677e76223318012b8a66866060eaaf5e9507b0a44a`.
Independent audit rehashes all six replay files and reconciles all eight
capture commands and timing rows. Every runner uses the expected checksum
argument and emits only its single expected progress diagnostic. The pinned
runner checks the checksum after trimming trailing CR/LF; these captures
retain its summary, not a separate byte-exact workload stdout file.

Each trace contains 1136-3587 records, with no recorder-issued write entry or
exit events. Every script diagnostic and loss file is empty. There are no
adjacent duplicate scheduler records, unlike four original captures. Exec
and fork payloads identify four threads for each parallel WF process, one
for sequential WF, and a caller plus four Rayon workers. Every expected WF
entry probe appears once, except the run entry which appears once per
computing thread. Switch payload TIDs, including negative-header records,
reconstruct consistent running intervals and final X/Z switches for every
thread. Each parent wait4 return names its exact child and follows all final
thread switches. No duplicate folding or inferred missing exit is needed.

The new host is an AMD EPYC 9V74 VM with Linux 6.17.0-1022-azure and perf
7.0.14. All four logical CPUs are allowed, representing two SMT cores, with
no per-worker pinning. This is a new hosted machine: the following probe
intervals cannot be subtracted from experiment50's earlier ordinary samples
to allocate that panel's gap. There are two opposite-order observations per
form, not a new uninstrumented performance cohort.

| WF form / pass | Init entry to first run ms | Body entry to output submission ms | Submission to post-status ms | Post-status to exit_group ms | exit_group to parent reap ms |
| --- | ---: | ---: | ---: | ---: | ---: |
| Ordinary / 0 | 1.689 | 313.012 | 0.211 | 0.028 | 0.701 |
| Ordinary / 1 | 1.870 | 304.545 | 0.208 | 0.029 | 0.647 |
| Used-lanes / 0 | 0.701 | 306.825 | 0.199 | 0.044 | 0.541 |
| Used-lanes / 1 | 0.646 | 307.667 | 0.211 | 0.031 | 0.568 |
| Sequential / 0 | 1.682 | 1026.135 | 0.221 | 0.067 | 0.454 |
| Sequential / 1 | 1.776 | 1027.951 | 0.210 | 0.072 | 0.455 |

Here ordinary WF exec-to-exit intervals are 315.328/307.068 ms, used-lanes
308.170/308.908 ms, and Rayon 276.311/277.293 ms. Aggregate running intervals
through reap are about 1119.7/1120.9 ms for ordinary WF and 1093.9/1094.8 ms
for Rayon. The observed short-run difference lies overwhelmingly inside
the source-body interval, not a long initializer or an unobserved process
teardown. Neither a 1-2 ms initializer nor a sub-millisecond exit/reap
interval accounts for tens of milliseconds in these captures. The ordinary
used-lanes gain still changes sign between observations, consistent with
the negative ordinary-timing result rather than a default selection.

A further scheduler-interval audit narrows the next question. In the first
100 ms after exec, ordinary WF accumulates 273.5/295.6 ms of group running
time, and used-lanes 288.7/293.6 ms. Rayon accumulates about 393.2/397.0 ms
(including its mostly sleeping caller). Most early WF 10 ms windows thus
use about three logical CPUs, then approach four after roughly 100 ms;
Rayon uses close to four from the start. WF's early off-CPU intervals are
predominantly R-state switch-outs. Individual WF threads accrue 58.5-67.7 ms
of runnable off-CPU time in that first 100 ms, often switching away from
and returning to the same CPU. Sleeping intervals are much smaller.

This is occupancy evidence, not useful-compute attribution. R-state delay
does not by itself distinguish kernel placement from an explicit
`sched_yield`: the WF platform layer has yield calls, and this capture did
not record those syscalls or per-thread affinity readbacks. It also does not
show which ready task was locally available. A same-budget worker-placement
control and yield attribution now address a more concrete hypothesis than
another initializer/exit change. Keep the old trace rejection and forty
ordinary samples; no runtime default or language design follows from two
profiled timelines per form.


## Fifty-first experiment: compare Go buffer ownership

Experiment49 qualifies a normal sequential Go reference, but its release
goroutines retain large stacks for private 64 KiB buffers. That is not evidence
that the storage form is optimal for Go. This follow-up keeps Go 1.27.1, the
same initialized capacity, byte-stream contract and CPU budget, and compares
two ordinary ownership forms selected by `WF_BENCH_GO_BUFFER_OWNER`:

| Owner | Allocation and transfer | Handler's use |
| --- | --- | --- |
| `handler`, default | The handler creates its private buffer in a stack-owning wrapper | Calls the common sequential `echoBuffer` loop |
| `acceptor` | The accepting goroutine allocates the private buffer after accept and transfers its slice into the new goroutine | Calls the same `echoBuffer` loop with that supplied buffer |

There are no global keepers, unsafe conversions, buffer pools, alternate
reactors, io.Copy/splice calls or artificial escape functions. The heap form
moves real allocation work to the acceptor and exposes the allocation to GC;
those are costs of that normal Go ownership choice, not work excluded from
the comparison. Each connection still owns one 65,536-byte initialized buffer
until its ordered writes finish. Socket policy, the first-error cleanup and
joining behavior are shared. The frozen experiment49 remote remains at
`ccf667fd`; the new `codex/io-go-storage` branch contains this comparison.

The stack-owning wrapper is separate from the supplied-buffer loop. A
conditional 64 KiB allocation in that loop could force both policies to reserve
the same large fixed frame and invalidate the intended comparison. Instead,
`echoStack` is called only from `startEchoStack`'s goroutine, while
`startEchoHeap` allocates before starting its goroutine. Both call one
`echoBuffer` implementation. The common-loop refactoring changes the code
layout relative to experiment49; this experiment compares its two forms at
one revision rather than pairing their timings with the older binary.

Before recording live RSS, `go-check` builds both release and race binaries,
saves build metadata, compiler escape diagnostics and the actual disassembly,
and checks the release allocation sites: the handler buffer must not escape,
while the acceptor buffer must escape to the heap. On the pinned Linux/amd64
target it additionally checks a large initialized stack frame for the
stack-owning goroutine, small frames for the heap-owning goroutine and shared
loop, a heap allocation call, and both calls into the common loop. These are
requirements on this experimental storage control, not timing thresholds or
source-language acceptance rules. Other native targets retain their escape
checks and disassembly for inspection.

The local Linux/amd64 cross-build demonstrates the intended code generation:
the compiler inlines `echoStack` into `startEchoStack.func1`, whose frame is
`0x10028` bytes and whose initializer zeroes the 64 KiB buffer.
`startEchoHeap` calls `runtime.makeslice`; its handler wrapper has a 40-byte
frame, and the shared `echoBuffer` frame is 64 bytes. Both wrappers call that
same loop. Race instrumentation causes even the handler-created buffer to
escape in the locally inspected race build. Race memory and timing therefore
cannot stand in for the release storage comparison.

All sixteen experiment49 cases remain for **each** owner, thirty-two cases
in total: release/race × owner × GOMAXPROCS 1/4 × quiet stream, pressured
stream, reset-with-three-waiters cleanup, and live residency. All twenty-four
lifecycle cases complete before any of the eight live snapshots is attempted.
There is no early promotion after one variant's success. The same full 2 MiB
per-peer stream oracle, explicit error result and all-thread affinity/epoll
readbacks remain. Each live case retains 64 peers after checking 64 KiB per
peer. The Linux CI job still limits all server threads to one logical CPU,
uses another physical core for the fixture, and labels P4 as oversubscription
qualification. There is no performance panel.

Local M1 release/race qualification passes all thirty-two cases. Compiler
escape diagnostics, the Linux/amd64 frame predicate against the actual
cross-built binary, Go formatting/vet, shell/Make expansion and diff checks
pass. Native Linux qualification also succeeds at frozen revision
[`56102eac700427fe74e38b021fa8dfdb3e30df01`](https://github.com/mbbill/Whitefoot/commit/56102eac700427fe74e38b021fa8dfdb3e30df01),
[run 34094123069](https://github.com/mbbill/Whitefoot/actions/runs/34094123069),
job 101653778721. Artifact `10008168851`, `go-reference-qualification`, has
SHA-256 `a53efbed6c70d5937db663f1fec2e4d8d4d5578bf72e6c6521f0f12f6a1c3622`.
Independent raw audit reconciles all 32 logs and eight live snapshots with
the recorded build profiles, allocation sites and disassembly. All lifecycle
cases precede residency. There are no byte mismatches, unexpected output or
race reports; reset cases preserve the intended error and close the other
three blocked handlers.

The actual CI release artifact confirms the cross-build's representation:
the handler wrapper reserves `0x10028` stack bytes and explicitly zeroes
64 KiB, the acceptor calls `runtime.makeslice`, its handler reserves `0x28`
bytes, and both call the same `echoBuffer` with a `0x40`-byte frame. The actual
race artifact confirms **both** allocations escape to the heap; its handler
wrapper calls `runtime.makeslice` and has a `0x30`-byte frame. Similar race
memory therefore does not contradict the release allocation difference.

The Intel Xeon 6973P-C VM runs Linux 6.17.0-1022-azure, with CPU 0 as the
entire server mask and CPU 2 on a different physical core for the fixture.
All observed threads have mask `0`, their census matches process status,
and each live snapshot has one epoll descriptor with 65 registrations.
Every runtime observation reports NumCPU 1, the requested GOMAXPROCS,
GOGC 100 and no memory-limit or GODEBUG override. Effective socket send
buffers are 8,192 bytes and TCP_NODELAY is enabled. The two runtime source
hashes match experiment49's pinned toolchain. P4 still means oversubscribing
one allowed logical CPU, not a four-CPU performance comparison.

These are single live snapshots per cell on the **same host and revision**,
after 64 simultaneously open peers each finish a verified 64 KiB exchange:

| Build | Buffer owner | GOMAXPROCS | Live OS threads | RSS KiB | PSS KiB | GC cycles at exit |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| Release | Handler / stack | 1 | 3 | 13292 | 13284 | 0 |
| Release | Handler / stack | 4 | 3 | 15344 | 15336 | 0 |
| Release | Acceptor / heap | 1 | 3 | 11824 | 11816 | 1 |
| Release | Acceptor / heap | 4 | 7 | 11836 | 11828 | 1 |
| Race | Handler / heap | 1 | 4 | 41772 | 40156 | 1 |
| Race | Handler / heap | 4 | 7 | 42340 | 40724 | 1 |
| Race | Acceptor / heap | 1 | 4 | 41760 | 40144 | 1 |
| Race | Acceptor / heap | 4 | 7 | 42368 | 40752 | 1 |

The release heap form reduces these RSS snapshots by 1,468 KiB at P1 and
3,508 KiB at P4 (11.0% and 22.9%). It also allocates into the GC-managed heap:
exit TotalAlloc is 4,437,664/4,512,304 bytes, versus 240,688/290,640 for the
stack form, and one GC cycle occurs. Both still own the same 4 MiB aggregate
source buffer capacity. TotalAlloc excludes the stack payload, exit stack
counters follow handler teardown, and smaps covers the whole process;
none alone isolates live payload storage or lifetime peaks. The snapshots
do not quantify GC's timing cost, thread peaks, connection-churn behavior or
memory scaling beyond these 64 peers.

The heap form is a qualified lower-residency candidate in these cells; it
does not establish the fastest Go implementation. Keep both release forms
for a future fixed-resource calibration, since their throughput/CPU and GC
tradeoffs have not been measured. The canonical gate at this revision is
still queued/running when this evidence is recorded. No Go timing cohort,
pooling or splice experiment has run, and neither frozen experiment49 nor
any earlier native/WF measurements are replaced.

## Fifty-second experiment: qualify generated continuations on the mixed protocol

The current continuation comparison must eventually include computation
between socket operations. The existing `tcp_compute_server.wf` already
expresses that workload as ordinary sequential source: assemble a 64-byte
request, decode its seed and round count, evaluate the dependent recurrence,
encode its result and send the complete reply before reading again. This
experiment compiles those unchanged bytes with `--continuations --par` and
reuses the existing generated host and compiler path. It changes no runtime,
source language rule, proof, emitted representation policy or source workload.

The initial ordinary-binary smoke passed the common four-connection compute
oracle, but the old truncated-input invocation failed its final status check.
A diagnostic re-run established an ordinary process exit of exactly 9, which
is the source-defined truncated-request result. The fixture had required 1
for every program. Its error modes now accept an explicit optional expected
status in 1-255, retaining default 1 for existing callers. No arbitrary
nonzero code or signal is accepted. Truncated requests are still incomplete
17-byte frames; the fixture now half-closes and verifies that the server
closes without any response bytes before checking the exact exit status.
The added `oversized` mode sends a complete request with 16777217 rounds,
one beyond the existing protocol limit, and checks the same no-response
closure with WF's source-defined exit code 10.

`compiler-continuation-check` compiles the compute source alongside all
previous generated programs, with the same LLVM and C sanitizer settings.
At each threaded, owner-one and owner-thirty-two policy it runs:

- Four concurrent clients, each sending the three existing independently
  fixed seed/round/result vectors in one-byte fragments and checking all
  response bytes, then half-closing and requiring clean EOF.
- One truncated request with expected exit 9 and no response bytes.
- One oversized request with expected exit 10 and no response bytes.

The common oracle owns bytes and process results. Separate generated-host
reports must show the requested owner/batch policy, balanced positive waiter
registration/dequeue counts, exactly four or one completed and retired task,
and matching accept/receive routes. Fragmented compute requests must exercise
a suspended receive. The complete error input may already be queued when
the handler runs, so its receive may legitimately complete inline. Initial
local sanitizer runs exposed that case and rejected an overstrict new report
assertion even though byte checks and exact source error codes passed. Error
cases now admit inline receives while retaining route checks for every
reported deferred receive. The Linux native-ring invocation requires native
accepts and no helper receives; its fragmented compute case also requires
native receives. The forced-helper repetition requires no native ring routes.
Unexpected diagnostics fail the qualification. The same three
policies retain the existing file, pipe, recursive, socket, fanout and bounded
window tests. The isolated `codex/io-continuation-mixed` CI branch runs this
qualification without a new timing panel.

Pure `churn` remains an ordinary function called by the sole continuation
resumer. It has no new compute offload or checkpoint. Correct answers from
four clients therefore do not establish multicore computation, light-request
progress during long computation, mixed throughput, or cancellation of
sibling tasks on failure. The error cases intentionally have one connection;
they do not qualify a multi-connection abort/drain policy. Those distinctions
remain necessary before comparing a future unified WF runtime with the
bounded Tokio/Rayon mixed reference.

Local M1 ASan/UBSan and ThreadSanitizer runs pass the entire generated suite,
including all nine new mixed-protocol invocations in each build. The
existing nested C++ registration/loan fixtures also pass. All five
Tokio/Rayon tests, its four shared-oracle invocations, and all sixteen
existing Go release/race checks pass with the strengthened common fixture.
A negative check specifying exit 1 for WF's truncated-input result still
fails and reports actual exit 9; the explicit status argument does not
collapse source errors into generic success. After integrating experiment
51, both Go storage forms also pass all thirty-two local cases against this
same fixture.

Frozen revision
[`6a8c19c1b7846d1aaed9a6a95f5ecfcdf1852d91`](https://github.com/mbbill/Whitefoot/commit/6a8c19c1b7846d1aaed9a6a95f5ecfcdf1852d91)
passes Linux native/helper job 101660513221 in
[run 34096270201](https://github.com/mbbill/Whitefoot/actions/runs/34096270201).
The separate Windows placement job also passes; it does not execute generated
WF continuations. Artifact `10008815869`, `completion-continuations`, has
SHA-256 `f4e2575d387e0b849555bafec7248023c982f4bb00397fa9553811e36cf99cc6`.
An independent log audit verifies all eighteen new mixed cases in the exact
three-policy/two-route order, positive balanced waiter counts, the requested
policy and exact completed/retired task counts. Each fragmented compute case
records deferred receives on its required route. In this run, all truncated
and oversized inputs are read inline; their accepts still use the required
native/helper route, and the common oracle checks closure and exact exits.
The previous 66 generated stream invocations, three file-outcome suites and
four 640-case nested completion-loan fixtures also pass. The artifact retains
ordinary and instrumented compute IR. These results qualify protocol and
lifetime behavior, with the sole-resumer computation limitations above.

## Fifty-third experiment: sequential Go in the fixed echo screen

Experiments49/51 establish byte-stream, lifecycle and storage evidence for
Go's normal sequential `net` form. This experiment brings both qualified
storage choices and both runtime widths into the same controlled Linux echo
screen as experiment48. The new `codex/io-go-screen` branch starts at root
revision `96e6cd2d` and integrates the two experiment51 commits; earlier
qualification and measurement branches remain frozen. No Go loop, compiler,
runtime, source buffer capacity or protocol changes in this experiment.

`make scheduler-go-screen` selects `GO_SCREEN=1`, `CONTINUATION_SCREEN=3`,
`NATIVE_BASELINES=1` and the existing `combine`/`allocator` harness. It retains
all prior qualification, instrumentation boundaries and alternating order.
The candidate list is fixed before measuring:

| Group | Forms | Resource/storage interpretation |
| --- | --- | --- |
| WF stackful | `callee-small`, `balanced-small` | Existing private initialized source buffers and qualified runtime policies |
| WF generated continuations | `wf-coro`, `wf-coro-owner`, `wf-coro-batch32` | Existing threaded progress, sole-owner progress and 32-resumption batching control |
| Native C/C++ | `uring`, `uring-64k`, `epoll`, `epoll-calloc-main`, `fiber-calloc-main`, `cpp-elide`, `cpp-elide-calloc` | The same seven controls as experiment48; matched provided bytes for the two ring buffer sizes, other storage models remain explicit |
| Go release | `go-handler-p1`, `go-handler-p4`, `go-acceptor-p1`, `go-acceptor-p4` | Same sequential loop and initialized private 64 KiB capacity; handler stack versus acceptor heap, GOMAXPROCS 1/4 |

Every form runs on `split1-top0-no-thp`: one logical CPU for the entire server,
with the client on another physical core. All Go runtime/GC threads inherit
that server mask. P4 oversubscribes one allowed CPU; it is not a four-CPU row.
The existing THP-disable launch policy also applies to Go. The glibc top-pad
setting remains common but does not choose Go's cgo-disabled allocator.
Go uses GOGC 100, default memory/debug limits and TCP_NODELAY. Ordinary samples
explicitly clear the qualification send-buffer override and turn reporting
off. Race instrumentation never enters the measured binary.

The five unchanged cells are 1/4/64/1024 peers at 64 bytes and 64 peers at
65,536 bytes. Per-peer trip counts are 10,000/10,000/2,000/200/500. Sixteen
forms × five cells × seven recorded passes produce **560 ordinary rows**
after two complete warmups. The existing three live cases (64/1024 peers at
64 bytes and 64 peers at 65,536 bytes), repeated three times for every form,
produce **144 panel snapshots**. The harness checks these counts and retains
every sample, including startup/drain CPU, quantized process resource data,
client exchange CPU, latency and raw RSS maps. No configuration is selected
by tuning against these samples.

Before Go joins the panel, the same job reruns all 32 release/race tests at
the exact revision and within the chosen CPU mask. The eight live snapshots
from that qualification stay in its separate artifact directory and do not
count toward 144. Both release allocation sites and actual Linux/amd64 stack
frames must still match the experiment51 control. The panel copies the exact
qualified release binary; it does not rebuild with different flags. Observed
preflight runs verify Go version, buffer owner, runtime width, default GC
settings and effective socket options. Normal live panel runs additionally
capture every enumerated thread's status and epoll fdinfo, checking the one
CPU mask and one reactor with all peers plus its wakeup registered. These
snapshots remain thread censuses rather than lifetime peaks.

The artifact retains Go release/race build metadata, escape diagnostics,
disassembly and qualification logs; Go source and the release binary; source,
compiler/tool and panel-binary SHA-256 hashes; runtime backend source hashes;
all existing WF/native observed logs; and process/host resource snapshots.
Runtime GC observations are outside timing and do not substitute for OS CPU
accounting. Race-induced heap escape remains a qualification caveat, not a
release storage result.

This is a fixed comparison within the existing loopback/client envelope.
The earlier client-headroom limitation remains: similar throughput can mean
client saturation, and p99 is from closed-loop requests with one outstanding
request per peer, not an open-loop overload SLO. Neither matching a reference
nor beating it in these five cells establishes universal optimality. Local M1 checks pass all 32 Go release/race cases and four additional
quiet launches through the new configuration helper, including deliberately
inherited report/send-buffer/GC overrides that the helper must clear. The
observed-report predicate accepts all four frozen Linux release records and
rejects a mismatched owner. Shell/workflow parsing, Make expansion and diff
checks pass. The native Linux screen completes successfully as recorded below.

Native Linux results are from the frozen
[`7aa6191c927723870ca5548d0b94f65940d6c7f3`](https://github.com/mbbill/Whitefoot/commit/7aa6191c927723870ca5548d0b94f65940d6c7f3),
[run 34096288985](https://github.com/mbbill/Whitefoot/actions/runs/34096288985),
job 101660571115. Artifact `10009918888`, `io-scheduler-allocator`, has SHA-256
`2e3284d8f3c8a626c2d6155f35b5dc5cb8283c1a4a92ddb7fa3e96fde5783645`.
The host is an Intel Xeon Platinum 8370C VM, Linux 6.17.0-1022-azure, Clang
20.1.2. Server CPU 0 and client CPU 2 are on different physical cores.
Independent audit reconciles all 560 ordinary rows against their raw client
and resource files, verifies all 80 seven-pass groups, and recomputes all
144 smaps totals. Ordinary server/client diagnostic channels are empty,
trip counts match the five fixed cells, and THP/AnonHugePages/swap readbacks
match the recorded policy. No samples are removed.

The same audit separately verifies all 32 Go qualification cases and their
eight snapshots, actual release/race allocation diagnostics and frames.
All 36 normal Go panel thread censuses have mask `0`, counts matching their
process status, and one epoll descriptor with peers-plus-one registrations.
Normal observed sockets report NODELAY 1, SO_SNDBUF 2,626,560 and SO_RCVBUF
131,072: the qualification's requested 4,096-byte send buffer is absent.
The measured release binary is byte-identical both to this job's qualified
copy and to experiment51's release artifact, SHA-256
`dbe30414219c51057ebaae2624542aa6939ba3f29e584c84899d18ea95de36e8`.
Toolchain/backend source hashes also match. Source capacity, source code and
binary are fixed; host and THP conditions differ from experiment51, so its
memory numbers must not be spliced into these paired comparisons.

The table reports median roundtrips/s and median per-pass p99 in microseconds
for **every Go configuration**. It is a fixed screen, not a retuned winner:

| Peers × bytes | Handler P1, rate / p99 | Acceptor P1, rate / p99 | Handler P4, rate / p99 | Acceptor P4, rate / p99 |
| --- | ---: | ---: | ---: | ---: |
| 1 × 64 | 33077 / 42 | 33018 / 42 | 33152 / 42 | 33072 / 42 |
| 4 × 64 | 172549 / 48 | 171911 / 48 | 154683 / 51 | 153675 / 52 |
| 64 × 64 | 181875 / 685 | 183046 / 682 | 175206 / 3279 | 175525 / 3316 |
| 1024 × 64 | 153285 / 9284 | 157627 / 8573 | 151119 / 42838 | 157237 / 42816 |
| 64 × 65536 | 47537 / 1872 | 47977 / 1663 | 47721 / 13830 | 47505 / 12566 |

At P1 the two ownership forms have overlapping paired rate ranges in four
of five cells. Acceptor/handler at 64 small peers is 1.008 [1.002, 1.025];
its small throughput difference is secondary to the memory difference below.
P4 under the same one-CPU mask is a poor latency tradeoff here. For the
acceptor form, paired P4/P1 p99 is 4.869 [4.629, 4.953] at 64 small peers,
5.062 [3.636, 5.234] at 1024, and 7.556 [2.743, 9.069] for large messages.
The corresponding rate ratios are 0.961 [0.955, 0.966], 1.008 [0.977, 1.018]
and 0.991 [0.984, 1.009]. These are median and full ranges of seven same-pass
ratios, not confidence intervals. They concern oversubscription on one CPU,
not Go with four physical CPUs. This control gives no reason to prefer P4
for the next one-CPU comparison; all four original rows remain retained.

These same-host throughput medians place the Go candidates among the existing
references. The native/old-WF columns identify the highest median in that
family **in these samples**, rather than an independently selected optimum:

| Peers × bytes | Native form and rate | Existing WF form and rate | WF batch32 rate | Go acceptor P1 rate |
| --- | --- | --- | ---: | ---: |
| 1 × 64 | epoll-calloc-main: 35112 | callee-small: 49683 | 33239 | 33018 |
| 4 × 64 | epoll-calloc-main: 204502 | callee-small: 181811 | 172355 | 171911 |
| 64 × 64 | uring: 218940 | balanced-small: 215084 | 197841 | 183046 |
| 1024 × 64 | cpp-elide: 206494 | balanced-small: 192876 | 176921 | 157627 |
| 64 × 65536 | fiber-calloc-main: 47684 | callee-small: 48027 | 46744 | 47977 |

At 64 small peers, native uring / WF balanced-small / WF batch32 / Go
acceptor P1 use median 4.453 / 4.688 / 5.000 / 5.391 process CPU
microseconds per trip, with p99 325 / 334 / 363 / 682 microseconds.
WF batch32 / Go acceptor P1 paired rate is 1.089 [1.060, 1.090], CPU is
0.928 [0.900, 0.942], and p99 is 0.530 [0.516, 0.537]. At 1024 small peers
those ratios are 1.116 [1.073, 1.163], 0.915 [0.879, 0.946], and
0.868 [0.808, 0.945]. The current generated sequential WF form thus exceeds
this ordinary Go control in these occupied small-message cells; the native
controls and existing WF stackful policies still provide higher throughput.
One small peer exposes a different CPU tradeoff: callee-small reaches 49,683
trips/s at 20 CPU microseconds/trip versus the leading measured native's
35,112 at 11. That is not a cost-free win or proof that the sleeping/polling
policy of the native control is optimal for underoccupancy.

Large-message medians cluster around 47–48 thousand trips/s. Client exchange
CPU is approximately one full client CPU for these fast forms, as it is for
many occupied small-message native cells. WF batch32 / Go acceptor P1 large
rate is 0.974 [0.938, 1.046]; the panel cannot declare either a server-capacity
winner from that near-tie. Process CPU still includes startup/drain and is
centisecond-quantized. All CPU, switch, latency and client observations are
retained; none is reclassified as open-loop SLO or real-NIC evidence.

Normal live RSS medians below use three snapshots per cell, not the observed
qualification snapshots. Selected comparison rows are shown; all sixteen
forms and all 144 smaps/status records remain in the artifact:

| Form | 64 peers × 64 B, KiB | 1024 peers × 64 B, KiB | 64 peers × 64 KiB, KiB |
| --- | ---: | ---: | ---: |
| Go handler P1 | 7496 | 74588 | 7504 |
| Go handler P4 | 7616 | 74832 | 7584 |
| Go acceptor P1 | 4124 | 11956 | 6632 |
| Go acceptor P4 | 4352 | 12296 | 7116 |
| WF callee-small | 3620 | 15200 | 6296 |
| WF batch32 | 3100 | 12508 | 5792 |
| Native uring-64k | 2372 | 4320 | 3756 |
| Native epoll | 1768 | 1788 | 1832 |

Acceptor P1 retains much less physical memory than handler P1 at 1024 small
peers (11,956 versus 74,588 KiB), despite the same 64 MiB aggregate initialized
source capacity. Initialization semantics do not require identical eager
page touching. The exact stack wrapper explicitly zeroes 64 KiB; the pinned
Go runtime's `makeslice` requests initialized storage from `mallocgc`, whose
large-allocation path clears only when the span is not already known zero.
The different 64-byte/64-KiB residency patterns are consistent with different
page touching and allocator behavior as well as frame size. These snapshots
and source evidence do not quantitatively partition those causes. TotalAlloc
omits stack payload; race moves both buffers into the heap. Neither serves
as a replacement for these normal live memory measurements.

Go acceptor/P1 is consequently the strongest current one-CPU Go candidate
for follow-up within this screen's storage/latency criteria, while handler/P1
remains a useful representation control. No pooling, splice, socket-policy
retuning or broader backend search is implied. The result strengthens the
reference quality and narrows implementation tradeoffs among sequential-API
candidates. It does not identify an intrinsic cost of sequential user code,
settle the language model's ultimate performance or establish a universal
ranking.

At result-recording time all Linux canonical stages, including scheduler,
pass at this revision; three macOS stages remain queued. Native Linux/Windows
host checks pass. The separate io-bench Windows job 101660570781 refuses its
warm-I/O measurement at `windows-bench.ps1:612`: `io-warm` remains unstable
after two complete cohorts. That is a separate invalid timing table, not a
Go/Linux protocol failure; its refusal is preserved without a threshold
change or a rerun to obtain green.

## Fifty-fourth experiment: index pending continuation waiters

Experiment 48 recovers submission batching but leaves the generated candidate
behind native small-message controls. Every completion currently searches
the coordinator's single pending list while holding its mutex, including
inline completions that have no registered waiter. With many suspended
connections that work can grow with the number of unrelated waiters. The
remaining measured CPU gap and this concrete path select a lookup control;
they do not yet prove that lookup dominates the loss.

`WF_CONTINUATION_PENDING_BUCKETS` accepts powers of two from 1 through 1024,
default one. One bucket retains the original LIFO pending-list behavior.
The candidate hashes the completion-record address into 1024 buckets;
pointer equality still identifies a waiter within its collision list. The
static array occupies 8192 bytes on the supported 64-bit targets in both
modes, while which pages are touched may differ. There is no allocation,
table resize, load-factor rejection or new maximum number of waiters.
The fixed 1024 choice serves this screen's connection range; it is not
claimed to be optimal or a language-level resource rule.

Configuration is read before the host starts source or completion publishers
and stays fixed for that process. Registration and lookup use the same
function and the existing mutex. Publication removes the matching pending
node before invoking the real completion publisher, then enqueues its waiter
on the existing FIFO. It still never reads the record after DONE. The sole
resumer owns the queued frame until dequeue; compiler-generated joins and
retirement preserve the established storage lifetime. Task joins retain
their separate direct waiter link. Bucket membership changes no ready order,
completion-record fields, generated waiter/task layout, source bytes,
ownership/proof rule or `wf__par_*` entry point. Hash collision or address
layout cannot select source acceptance, completion validity or a failure.

`make scheduler-continuation-index` selects `CONTINUATION_SCREEN=4`,
`NATIVE_BASELINES=1` and the unchanged allocator/combine panel. It keeps all
twelve experiment48 forms and adds `wf-coro-index`. The selected comparison
is the same uninstrumented executable at owner progress and batch 32 with
one versus 1024 buckets. The threaded and owner-one controls retain one.
All six WF forms and seven native controls use the same one-server-CPU and
separate-client-core envelope, initialized buffer capacities, THP setting,
source workload and five cases. Seven alternating passes after two complete
warmups yield 455 ordinary rows; three repetitions of three live cases yield
117 snapshots. The harness checks these counts. Source, generated IR and
both continuation executables are hashed and retained in the CI artifact.

The new `PROBE_LOOKUP_OBSERVE` compile-time switch adds completion-lookup
calls, visited list nodes and maximum nodes visited in one lookup. The
ordinary executable is built with zero, so these counters and increments
are absent. A separate host object built with one links the identical
generated IR and runtime objects. Existing coordinator and bridge counters
remain in both builds, as in experiment48. All fourteen continuation
observations use the counted host: a common large stream and 4/64-peer
preflights for each of four modes, plus 1024-peer runs for the two batch32
lookup choices. The reported bucket count must match the requested mode;
waiters, routes, submitted/completed operations and task retirement retain
their existing checks. Observed time is excluded from ordinary tables.

The C++ fixture runs both frame-allocation forms at one, two and 1024
buckets. Its eight roots remain gated until every root suspends, so two
buckets necessarily exercise collision chains. All previous before-arm,
during-arm, cancellation-request-with-drain and exact allocation/free checks
remain. The generated sanitizer suite runs both one and 1024 buckets at each
of the three progress policies, including mixed-protocol exact error exits.
The normal and lookup-counted release executables each run the twelve-task,
four-slot retirement oracle at all six settings. No error case is converted
to generic success to accommodate the index.

Local M1 ASan/UBSan and ThreadSanitizer runs pass all six generated settings:
66 stream invocations, six file-outcome suites and eighteen mixed-protocol
checks per sanitizer, plus six 640-case C++ fixtures. The ordinary and
counted release builds each pass all six bounded-retirement settings. Linux
native/helper qualification and performance remain pending on the isolated
`codex/io-continuation-index` branch. Client headroom, single-peer idle
tradeoffs, multi-owner computation and fairness retain their earlier limits;
even a lookup win would only identify another implementation cost.

The frozen index source is `1192ef351cd22eff92ed07b0320543d44419f38e`.
Its [Linux native/helper qualification](https://github.com/mbbill/Whitefoot/actions/runs/34099556884/job/101670620592)
passes. Artifact `10010458552`, `completion-continuations`, has ZIP SHA-256
`45f1532c4a7b31f3ed6780dad3082d44221547a29bf6cf220c36b7950223a546`.
An independent raw-log audit checks all six exact bucket/progress settings,
132 generated stream cases, six file-outcome suites, 36 mixed-protocol cases
and twelve 640-case C++ nested loan fixtures. Mixed cases have balanced
waiter registration/dequeue, exact task completion/retirement and matching
native/helper routes; the C++ fixtures retain forced two-bucket collisions
and exact heap/elided allocation/free counts. The separate Windows placement
job also passes; it does not run generated WF continuation code. The timing
panel below remains separate from qualification observations.

### Frozen index performance result

The same run's [measurement job 101670620437](https://github.com/mbbill/Whitefoot/actions/runs/34099556884/job/101670620437)
passes. Artifact `10011087479`, `io-scheduler-allocator`, has ZIP SHA-256
`be485fc5fc662da44d902ad1dcecb833c1bb855d87cfc8411a8714d220ad1e5b`.
The host is EPYC 9V74, four logical CPUs/two SMT cores, Linux
6.17.0-1022-azure, Clang 20.1.2 and glibc 2.39. Server CPU 0 and client
CPU 2 belong to different physical cores. This is a different host from
experiments48/53; compare forms within this panel instead of pooling rates.

The independent audit verifies 455 ordinary rows, 65 complete seven-pass
groups, exact alternating form and fixed case order, all raw client and
process resource fields, trip counts, rate rounding intervals and empty
ordinary output/diagnostic channels. It also verifies all 117 smaps sums,
THP disabled, zero huge pages/swap and retained process affinity masks.
Five frozen source hashes and four retained executable/IR hashes match.
The artifact records the compiler executable's hash but does not retain that
executable, so it is not independently rehashed here. ELF symbol tables
confirm that all three lookup counters are absent from the ordinary binary
and present in the observed binary. No counter-derived sample enters timing.

The selected comparison holds owner progress and batch32 fixed. Rates and
CPU values below are ordinary medians; each ratio is the median of seven
same-pass index/one-bucket ratios, with the complete range in brackets.
CPU is whole-process user+system time per round trip, including startup/drain.

| Peers / bytes | One-bucket rate/s | Index rate/s | Paired rate ratio [min, max] | Paired CPU ratio [min, max] | Paired p99 ratio [min, max] |
|---|---:|---:|---|---|---|
| 1 / 64 | 35822.2 | 35770.2 | 1.000618 [0.987249, 1.005021] | 1.000000 [0.941176, 1.062500] | 1.000000 [0.923077, 1.027778] |
| 4 / 64 | 134370.6 | 133846.1 | 0.997269 [0.964683, 1.013273] | 1.034483 [1.000000, 1.035714] | 1.000000 [0.956522, 1.073171] |
| 64 / 64 | 147101.1 | 148657.2 | 1.009701 [1.005300, 1.014297] | 0.988506 [0.977011, 1.000000] | 0.978858 [0.950104, 1.008791] |
| 1024 / 64 | 136268.4 | 139792.6 | 1.018090 [0.990758, 1.037157] | 0.980263 [0.961039, 1.006579] | 0.954892 [0.732471, 1.214468] |
| 64 / 65536 | 42165.0 | 42721.7 | 1.005607 [0.957901, 1.077448] | 0.986667 [0.935897, 1.041667] | 0.998762 [0.920976, 1.514391] |

At 64 small peers the approximately 1% rate improvement has the same sign
in all seven pairs; median CPU falls 6.796875 to 6.718750 us/round trip.
The 1024-peer median CPU falls 7.519531 to 7.275391 us, but paired rate,
CPU and tail-latency ranges retain reversals. Low-peer and large-message
cells do not establish a repeatable gain. Their negative pairs are retained.

All fourteen separate continuation reports pass exact bucket/progress,
balanced waiter, native route, ring submission/completion and task-retirement
checks. The occupied batch32 observations explain what the index changes:

| Peers | Buckets | Lookup calls | Visited nodes | Nodes/call | Maximum nodes | SQEs/nonempty kick |
|---|---:|---:|---:|---:|---:|---:|
| 64 | 1 | 256258 | 4677075 | 18.2514 | 64 | 31.5198 |
| 64 | 1024 | 256258 | 134136 | 0.5234 | 2 | 31.6177 |
| 1024 | 1 | 413698 | 11658924 | 28.1822 | 1024 | 27.9419 |
| 1024 | 1024 | 413698 | 219886 | 0.5315 | 7 | 28.1641 |

Visited nodes fall about 97.1%/98.1% while submission batching stays similar.
Sub-unit nodes/call is valid because inline completion may find no registered
waiter. These counts show elimination of lookup work, not proportional CPU
or throughput savings. Hash collisions remain real in the measured index
case; equality lookup and lifetime checks still complete correctly.

The ordinary index's paired rate versus native `uring-64k` is 0.982409 at
64 small peers and 1.004297 at 1024, both with ranges crossing one. Its paired
CPU ratios are 1.048780 and 1.072464: native still does less measured CPU
work. Versus native epoll the paired rates are 0.992218 and 0.976348.
The stackful WF `callee-small` control also remains faster in these two
cells (index/callee 0.980513 and 0.963978). Thus a narrow near-tie with one
native form is not a frontier result.

The client complicates that comparison: at 64 small peers the index,
one-bucket batch32, stackful WF and fast native controls all use approximately
one client CPU. At 1024 small peers index uses 0.979 CPU/wall and native
epoll 0.984. Large native forms are again near one client CPU, while the
index's median is 0.919; server-specific transfer behavior can expose
different limiting work even for the same client. A faster or independently
scaled client is still required before treating clustered rates as server
ceilings. The complete large-message spread remains in the raw artifact.

Median live RSS at 64-small/1024-small/64-large peers is 3108/12512/5456 KiB
for one-bucket batch32 and 3112/12516/5040 KiB for the index. The small-message
difference is one 4 KiB page; both settings already contain the same static
8 KiB bucket array. Large-message page-touch variability is not evidence
that hashing changes frame representation or buffer capacity. This screen
supports a small lookup improvement under occupancy, not a default change
or the conclusion that pending lookup explains the remaining native gap.

## 55. Attribute early runnable waits before changing CPU placement

The corrected experiment50 traces show WF using roughly three logical CPUs
during its first 100 ms while Rayon uses roughly four on the same host.
Several WF threads spend 58.5..67.7 ms runnable but off CPU in that interval.
The scheduler's R state alone does not distinguish an explicit yield from
ordinary kernel scheduling. The platform implements `wf_prim_yield` with
`sched_yield`; the core calls it during idle looks, completion-publication
waits and exhausted-stack joins, and worker startup also uses it. Pinning
threads or changing the wait strategy before distinguishing these cases
would confound two different candidate explanations.

`codex/io-cpu-yield-attribution` therefore uses the existing coarse trace
script with `CPU_YIELD_TRACE=1`, adding syscall entry and exit events for
`sched_yield`. It keeps all eight captures: ordinary WF, used-lanes WF,
sequential WF and Rayon, one batch in each of two opposite orders. The
script retains the existing global scheduler, process, write, wait4 and
milestone events, recorder-write exclusion, event-format files, explicit
loss/incomplete status, thread IDs and checksum-runner diagnostics. No
recursive function probes or syscall call stacks are added. Traced time
does not enter an ordinary performance table.

CI restores the exact experiment50 executables and generated IR through
the smaller audited c81 recapture artifact `10009398989` (ZIP SHA-256
`f117d23ff93ed900b6418b677e76223318012b8a66866060eaaf5e9507b0a44a`).
It verifies all six individual file hashes again. The original source
revision remains `8dd5b44a4e090601c276431ff4f83da55e443c43`, with compiler,
runtime policy, WF source, Rayon grain and pool width unchanged. The new
host's topology and affinity envelope are recorded separately. There is no
rebuild, ordinary retiming, pinning or default change in this diagnostic.

The required result audit reconstructs the exec/fork thread set, pairs each
thread's yield entry with its successful exit, and intersects those intervals
with that thread's R-state off-CPU intervals. It reports the first 100 ms
and complete body separately, with per-thread and aggregate denominators.
An unbalanced syscall stream or lost scheduler record cannot support that
attribution. Absence of yields on a sequential or Rayon thread is valid.
Tracing every yield can itself change scheduling; the control can locate
observed waits, but cannot silently replace the ordinary timing evidence or
establish readiness of useful tasks. If large waits occur outside yields,
the next discriminating control is same-budget worker placement. If waits
overlap yields, locate the runtime waiting path before selecting a replacement.

Local checks cover shell/YAML syntax, branch selection, invalid mode rejection
and restoring the exact six files from the retained ZIP. The Linux capture
and independent attribution audit follow below.

### Frozen yield attribution result

Frozen revision `ab3879e99a0f9e3eb9e922acf1b487a6842bba9f` completed
[Linux CPU capture job 101678868378](https://github.com/mbbill/Whitefoot/actions/runs/34102171268/job/101678868378).
Artifact `10010930480`, `io-cpu-yield-attribution`, has ZIP SHA-256
`18138d2e4417004ec596e206f9c99fc9b469f53fabe0b3dfa72973f8afd55b5d`.
This host is AMD EPYC 7763, four logical CPUs on two SMT cores, Linux
6.17.0-1022-azure and perf 7.0.14, with allowed CPUs 0..3. It is a different
host from the c81 EPYC 9V74 capture; the six restored executable/IR hashes
match exactly, while absolute times from the two hosts are not pooled.

The independent audit accepts all eight complete streams: monotonically
ordered timestamps, no duplicate scheduler records, no lost/throttled data,
zero recorder write records, complete exec/fork thread sets, balanced running
intervals, unique WF milestones, terminal thread states and exact successful
parent reaps. Every workload yield entry has one matching zero-status exit;
there are no censored or unmatched calls. The runner's exact checksum
argument and sole expected stderr line are retained; as in experiment50,
the runner validates output after trimming trailing CR/LF. Its summary is
not a retained copy of workload stdout.

These are sums across all workload threads during the first 100 ms after
exec. CPU and off-CPU values are thread milliseconds, so four simultaneously
running threads can contribute 400 ms. Rayon includes its mostly sleeping
caller plus four workers; WF uses the main thread plus three workers.
Yield count covers the complete process, while the other columns cover the
first 100 ms only.

| Form/pass | Yield calls | Running ms | R-state off-CPU ms | R-state ms inside a yield | Share inside yield |
|---|---:|---:|---:|---:|---:|
| Ordinary WF / 0 | 1131 | 286.277 | 94.678 | 85.665 | 90.48% |
| Ordinary WF / 1 | 1165 | 291.437 | 93.964 | 84.897 | 90.35% |
| Used lanes WF / 0 | 977 | 294.611 | 96.927 | 89.871 | 92.72% |
| Used lanes WF / 1 | 770 | 291.642 | 101.443 | 90.979 | 89.68% |
| Rayon / 0 | 32280 | 398.260 | 0.143 | 0.022 | 15.53% |
| Rayon / 1 | 32142 | 396.819 | 1.040 | 0.932 | 89.60% |

Sequential WF makes no yield calls. Across each WF body-to-output-submit
interval, 89.22..90.53% of R-state off-CPU time lies inside an explicit
yield. The longest ordinary-WF gaps are 2.99..4.55 ms on the same CPU, fully
inside yield calls. Rayon's far larger number of yields mostly returns
without descheduling; its small denominator makes the last percentage alone
misleading. Removing yields based on their count is therefore not a justified
optimization. In these captures it is the placement and duration of WF's
yields that matter.

The first-100-ms utilization deficit recurs with syscall observation, so
the previous R-state gap is now associated with explicit yielding in a
second host capture. This does not prove how much time a new waiting policy
would recover: syscall trace overhead differs between executors, the kernel
still chooses which thread runs after a yield, and neither ready-task
availability nor the yielding call site was captured. The selected next
control must locate the runtime wait path and distinguish per-worker
placement from changing its backoff; changing initialization or source
effects is not supported by this result. No runtime default is selected.

## Fifty-sixth experiment: observe the client syscall and readiness work

This bounded diagnostic starts from 72fdd468 and preserves experiment 53's
frozen source and rows. That Xeon host's 64-peer, 64-KiB fast controls spend
about 2.4--2.8 client user microseconds and 18.4--19.0 client system
microseconds per completed round, with client exchange CPU/wall near one.
This is evidence of occupied client CPU on that host, not proof that every
server on another host reaches the same limit or that system time is all
syscall-entry overhead. TCP work, copying and synchronous loopback processing
can also be charged to the client. Experiment 43's added client SMT sibling
did not establish spare physical client capacity.

Source inspection rules out redundant payload generation as the next fix:
`fill_message` runs before connecting; only the first echo byte changes per
round; full `memcmp` is already the successful verification path. Per-round
allocation is absent and percentile sorting follows exchange timing and CPU
snapshots. Two round-trip clocks remain. The echo pattern repeats modulo 256
in peer, position and round, so the old comment promising detection of every
misrouting, reordering or stale reply was too strong. The comment is corrected
without changing a payload byte or the existing error policy.

`WF_NETLOAD_OBSERVE=1` adds worker-owned accounting, with separate admission
and exchange records. `netload_observe.h` owns the portable accounting and
`netload_observe_check.c` supplies an operation-trace check through
`make netload-observe-check`; both are removed with this diagnostic when the
client limitation is resolved or a replacement supersedes it. Ordinary
builds have neither the observer fields nor its statements. The diagnostic
compares ordinary optimized LLVM IR against 72fdd468, normalizing only the
module/source filename lines, and fails on any remaining difference.

The observation fields have these meanings:

| Fields | Scope and interpretation |
| --- | --- |
| send/recv calls, positive, short, again, zero, error | Each actual socket call; short means positive and smaller than that call's requested length. EINTR remains the existing fatal socket error. No probe or retry is removed. |
| send/recv requested, bytes, maximum | Requested bytes include repeated suffixes and unsuccessful probes; bytes sum only successful transfers. Maximum is the largest successful return, not an application message or TCP packet size. |
| send/recv size0..size8 | Disjoint successful-return bins: 1; 2..64; 65..512; 513..4096; 4097..8192; 8193..16384; 16385..32768; 32769..65536; above 65536 bytes. |
| pumps, verified, verified_bytes | Pump invocations and completed comparisons. This panel is ordinary echo with full byte validation inside timing; existing duration-compute recurrence verification retains its prior separate semantics. |
| polls, ready, empty, interrupted, poll_errors | Exchange/admission epoll calls only, excluding connection establishment. Ready counts positive calls; events counts the returned entries. |
| input, output, both, errors, hangups, read_hangups | Returned event flags, including skipped events; input/output overlap in both. The unchanged interest mask does not request RDHUP, so a zero read_hangups count cannot establish absence of peer half-close. |
| skipped, dispatched, maximum_batch, batch0..batch3 | Returned entries skipped or dispatched by the original loop; maximum returned batch; positive poll bins 1, 2..8, 9..64, 65..256. |

No observer operation calls a clock, allocates, changes the requested bytes
or retries I/O. The original send/receive/EPOLLET control flow, one outstanding
round and full comparison remain. Startup captures each worker's actual TID
and CPU mask; it does not change affinity. After exchange CPU/timing snapshots
and all thread joins, the main thread checks outcome and histogram counts,
send=receive=checked bytes, checked rounds and event/batch conservation, then
prints the records. The shell independently checks recorded rounds/bytes and
the requested worker mask. Any failure remains a failed sample. Instrumented
counter updates still consume CPU and can perturb scheduling; these rows are
not an uninstrumented performance ranking.

`make scheduler-client-diagnostic` fixes the existing `uring-64k` and `epoll`
controls, one server worker on one logical CPU, and one client worker on a
logical CPU of another physical core. The server's existing top_pad=0 and
THP-disabled launch policy stays fixed. Two cells are retained verbatim:
64 peers x 64 bytes x 2000 rounds and 64 peers x 65536 bytes x 500 rounds.
One warmup followed by three alternating passes produces twelve rows in
`client-diagnostic.tsv`. Existing ordinary client-service and native stream
qualifications precede capture. The new real-client check uses two workers,
service budgets 0/1/8 and both message sizes; two extra four-peer admission
checks establish the separate phase accounting before diagnostic rows.

Four additional executions, one per server/cell, use the same observed
client under `perf record -e cpu-clock -F 199 --call-graph dwarf,8192`.
They go to `client-profile.tsv`, not the twelve-row table. Recorder and client
inherit the same client CPU mask; no extra worker, polling thread, SMT sibling
or host is added. The profile covers whole client lifetime and inherited
threads, including startup and post-timing output; exchange `getrusage`
continues to report only the client's original exchange interval. The outer
process-resource row for these four samples also includes recorder costs.
Stack capture can perturb the workload and is used only for attribution.

The conditional CI job installs a concrete Linux perf executable, requests
per-process kernel sampling and visible kernel symbols, and fails explicitly
if the capability probe fails. Raw `perf.data`, reports, period-bearing stack
samples and recorder diagnostics are retained. Decoding requests
`--show-lost-events`; a separate raw-record dump exposes LOST_SAMPLES and
THROTTLE/UNTHROTTLE records as well. `client-profile-status.txt` records each
decoder's exit status, sample counts, kernel/user/unknown symbol lines,
unexpected recorder output, decoder diagnostic bytes and loss/throttling
markers. All decoder diagnostics and unexpected recorder messages make the
profile explicitly incomplete. So do recorded loss/throttling, missing or
mismatched raw/decoded sample counts, absent kernel/user symbols or unresolved
symbols. Report output uses zero percent cutoff so small unknown-symbol rows
cannot disappear under the presentation threshold. Normal recorder progress
messages are retained separately from unexpected diagnostics.

An incomplete decode retains every raw file and the twelve checked
unprofiled observation rows; it does not claim complete kernel attribution or
silently accept a user-only trace. A recorder/client execution failure records
an incomplete status and fails the run, preserving preceding artifacts.
`loss_checked_kernel_attribution` only certifies the stated decoding and
visibility checks; a finite 199-Hz sample and 8192-byte DWARF stack snapshot
cannot prove exhaustive instruction or call-chain coverage. Host topology, runtime
settings, source/tool/binary hashes and kernel CPU/softirq/socket snapshots
surround the panel. `client-diagnostic-counters.tsv` links all sixteen records
to their raw sample directories and identifies the four profiled records.

The first discriminator is calls, empty probes and received bytes per call,
together with sampled client user/kernel work. Many recv/EAGAIN/epoll round
trips would motivate a separately qualified batching candidate; dominant TCP
copy/stack work would instead limit that hypothesis and motivate independent
physical client resources. Neither a new client backend nor a server policy
is implemented here. Counter counts must not be called TCP packet counts.

Local M1 qualification passes the ASan/UBSan synthetic operation trace,
including full/fragmented transfers, unsuccessful probes, EOF/error outcome
classes, every size-bin boundary and deliberately broken conservation edges.
All three observed service-budget forms cross-compile for x86-64 Linux with
strict C11 warnings, and ordinary optimized Linux IR matches 72fdd468. Shell
syntax and workflow YAML checks pass. These local checks do not execute Linux
syscalls. The frozen Linux result below supplies socket/stream qualification
and diagnostic counters; complete profile attribution remains unqualified.
Synthetic decoder-output checks separately verify the status policy for clean,
lost, throttled, failed-decode, unknown-symbol, user-only and mismatched-sample
captures while preserving the preceding observation table. Invalid diagnostic
mode/experiment combinations fail at preflight; no timing threshold changes.

### Frozen client diagnostic result

Revision `bc119e9a201da127c401decefc0a6c8a24e0cc4b` completed
[run 34104363706](https://github.com/mbbill/Whitefoot/actions/runs/34104363706),
including Linux measurement job 101685856568 and Windows placement checks.
Artifact 10012364607 has ZIP SHA-256
`92434c7bd9d7ebfa55f7cbda508036074abb7f727718f2bb1e2e5353ec67d7bc`.
The host is an EPYC 7763 guest with four logical CPUs on two reported SMT
cores, Linux 6.17.0-1022-azure and Clang 20.1.2. Server CPU 0 and client
CPU 2 belong to different reported cores; all observed client affinity
readbacks match CPU 2. Do not combine its absolute times with experiment 53.

The independently replayed artifact audit checks all twelve
counter-instrumented rows without perf and all four separate profiled rows
against their raw client and process-resource output. Across the 22 observed
sample directories, twelve measurements, four profiles, four warmups and two
admission qualifications remain distinct. All 22 exchange counter records
and both admission records conserve operation outcomes, bytes, verified
rounds, size histograms, poll/event batches and
`pumps = peer_count + dispatched_events` for the fixed zero service budget.
Native Linux client-service/observer and io_uring stream qualifications pass.

Four recorded source hashes match the frozen git revision. The retained
baseline source matches 72fdd468, and both normalized ordinary IR files
reconstruct from their retained inputs and are equal. The artifact records
42 executable hashes and two tool hashes, but the upload omitted the
executables themselves. None of those binary hashes is independently
recomputed from an uploaded binary. Future client comparisons must retain
the selected executable files as well as their hashes.

Every unprofiled observed row makes exactly one successful full send per
round, with zero short sends or send EAGAIN. The receive side is also almost
entirely full-message transfers. Ranges below span all three passes; rates
from these instrumented executions are not ordinary performance rankings.

| Server / message | Successful recv / round | EAGAIN recv / round | Events / positive epoll wait |
| --- | ---: | ---: | ---: |
| epoll / 64 B | 1.000000 | 0.999977..1.000000 | 42.04..49.36 |
| uring-64k / 64 B | 1.000000 | 0.999883..0.999984 | 17.41..62.37 |
| epoll / 64 KiB | 1.000094..1.000406 | 1.000562..1.004688 | 62.89..62.92 |
| uring-64k / 64 KiB | 1.000125..1.000219 | 1.006656..1.007156 | 60.78..61.16 |

The client uses 0.9795..0.9999 CPU/wall across these rows. Large-message
system CPU per round is 23.030/23.398/41.075 us against epoll and
43.869/43.866/43.675 us against uring-64k in pass order. Nearly identical
syscall counts coexist with materially different CPU costs; epoll's third
pass also changes substantially. Counts alone do not identify the expensive
kernel path or explain that variability. Fragmentation and small readiness
batches are not the large-transfer explanation in this capture.

All four profile statuses are **incomplete: unknown_symbols**. The audit
matches all 960 decoded samples to raw records and report totals; each has
the same 5,025,125 period weight. It finds no lost/throttle records, decode
diagnostics or count mismatch. Unknown leaf-symbol weight is respectively
3.0612%, 2.0202%, 3.6496% and 2.7397% for epoll-small, uring-small,
epoll-large and uring-large. Most are userspace libc/vDSO addresses; two
individual kernel samples lack names in nf_conntrack/nf_tables. This small
leaf share does not establish complete stacks: 194/196 and 196/198 sampled
small-message callchains contain unknown frames, while the large captures
have 16/274 and 11/292 empty callchains and 257/274 and 280/292 chains with
only kernel addresses. Full caller-based
kernel attribution is not qualified, and the original failure policy stays.

The concrete next performance candidate is a separately qualified
readiness-aware client: avoid the roughly one speculative empty receive per
round while preserving partial bidirectional progress and edge-triggered
notifications. This is a testable hypothesis, not a promised speedup; the
remaining per-call TCP/copy costs may dominate. Compare ordinary candidate
and unchanged clients under the same server/client CPU budget, then use
separate counters to verify which work disappeared. A lower client CPU cost
does not itself prove spare capacity or a server performance frontier.

## 57. Locate the yield wait path in the same CPU executables

Experiment 55 places about 90% of the captured early WF runnable off-CPU
time inside runtime-issued `sched_yield` calls. These are compiler-owned
runtime operations; WF source has no `yield` construct. A yield while
awaiting a completing record is
different from a yield after an empty work search or during pool startup.
Changing their shared primitive before identifying the path would combine
different hypotheses. The next capture adds caller identity to the same
frozen executables, without rebuilding the workload or changing placement.

The `codex/io-cpu-yield-callers` CI route restores the same six SHA-verified
files through artifact 10009398989. It runs the same four forms in two
opposite orders with `CPU_YIELD_TRACE=1 CPU_YIELD_CALLER=1`. The latter adds
one entry uprobe at `wf_prim_yield` in each WF executable. On these x86-64
ELFs its first instruction jumps to libc's yield entry; no prologue has
changed the caller's stack. The probe records `$stack0:x64`, the return
address, alongside the event's implicit probe instruction pointer. This
uses the documented [Linux uprobe fetch argument interface](https://www.kernel.org/doc/html/latest/trace/uprobetracer.html).
Other host architectures are explicitly incomplete for this experiment.

Each event's return address is normalized within its own image as
`caller - probe_ip + ELF(wf_prim_yield)`. The audit must match that address to
the instruction immediately after a real call to `wf_prim_yield`, not merely
the nearest function name. The retained ordinary and used-lanes binaries
have these return sites, verified by local LLVM disassembly:

| Wait path | Ordinary return offset | Used-lanes return offset |
| --- | --- | --- |
| Join sees COMPLETING | `0x485c` | `0x485c` |
| Compute join exhausted targets and finds no work/progress | `0x4a66` | `0x4a66` |
| Park handshake sees COMPLETING | `0x4b8d` | `0x4b8d` |
| Idle scan's yield rounds | `0x4df5` | `0x4df5` |
| Generic once wait | `0x6362` | `0x6382` |
| Worker-start once wait, inlined into lane acquisition | `0x663a` | `0x665a` |
| Worker-ready rendezvous | `0x66d5` | `0x66f5` |
| Completion bridge join sees COMPLETING | `0x8be5` | `0x8c05` |

Full native disassembly, symbols, ELF segments and event schemas accompany
the raw captures. The normal decoded events still include scheduler states,
yield entry/exit, process membership, output milestones and successful reap.
A separate raw-record decode checks LOST, LOST_SAMPLES and throttle records;
decode errors retain explicit incomplete status. The final attribution must
pair each caller event with its own thread's next yield entry and successful
exit, reject missing/duplicate/faulted or unmapped records, and intersect
those intervals with the already-required complete scheduler timeline.
An absent yield in a control is a possible result, not a required probe hit.

The additional traps affect WF and may change its scheduling. This is a
wait-path diagnostic, not another ordinary timing panel or a measurement of
the earlier host's exact deficit. It does not yet observe whether useful
work is available during a wait. No waiting policy, affinity, runtime ABI,
source effect or compiler default changes here. The native result below
provides the pre-exit/body caller and timeline audit used to select the next
idle-wait comparison; its three unpaired post-exit probes remain an explicit
limit on full-capture pairing.

### Frozen caller result

The frozen trace revision is `01aabbb6c6a0ad0e621a9236c15b3a1425209ea0`;
[run 34106967917](https://github.com/mbbill/Whitefoot/actions/runs/34106967917)
and CPU job 101694101286 completed successfully. Artifact 10012917872 has
ZIP SHA-256
`d3ac4c1546584a0b29719dd23e563c51f9e14e67499e5b4f1d839ddf379d7cb8`.
This host reports EPYC 9V74, four logical CPUs on two SMT cores, Linux
6.17.0-1022-azure and perf 7.0.14. Its absolute intervals are not pooled with
experiment 55's EPYC 7763 capture.

All six executable/IR hashes match. The registered probe file offsets map
through each ELF's executable segment to exactly `wf_prim_yield`'s entry.
Across eight traces, 188,130 raw samples match decoded events with no
loss/throttle records, recorder-write feedback or duplicate scheduler
transitions. Workload thread sets, phase order, complete switch lifetimes,
normal group exit and exact successful parent reap all pass. All 75,939
entered yield syscalls return successfully; each of the 3,465 WF calls also
pairs with a same-thread entry probe whose normalized return matches one of
the actual call instructions above. Sequential WF makes no yield calls.

The original unconditional caller-pair criterion has one explicit limit:
three extra idle caller probes never enter their yield syscall. Two occur
in used-lanes pass 1 and one in ordinary pass 1, respectively 1,959/6,215
and 7,716 ns **after** the recorded `exit_group` entry and before their own
thread-exit records. They are interrupted by process exit and are retained
as unpaired post-exit probes, not counted as successful yields. Every caller
before `exit_group`, including all first-100-ms and body intervals, pairs
completely. No unmatched entered syscall or unexplained pre-exit caller is
accepted. Thus the pre-exit attribution is complete; unconditional pairing
of every probe in the full capture is not claimed.

The first 100 ms after exec identify the dominant wait path. Values are
sums across WF's four workload threads, in thread milliseconds:

| Form / pass | Running | R-state off CPU | R-state inside any yield | R-state inside idle-scan yield | Idle share of yield off-CPU |
| --- | ---: | ---: | ---: | ---: | ---: |
| Ordinary / 0 | 285.816 | 98.199 | 69.586 | 69.184 | 99.4221% |
| Ordinary / 1 | 284.424 | 104.262 | 93.099 | 92.915 | 99.8020% |
| Used lanes / 0 | 285.405 | 88.135 | 73.851 | 73.561 | 99.6062% |
| Used lanes / 1 | 292.944 | 96.991 | 79.679 | 79.535 | 99.8187% |

The matching Rayon captures use 396.881/396.290 thread ms in this window.
WF's startup rendezvous and COMPLETING handshakes account for less than
0.6% of yield-associated runnable off-CPU time. The expensive observed wait
path is the idle scan, not the completion-publication handshake or the
exhausted-compute fallback. This locates a path; it does not prove useful
work was available during each of its waits.

Instrumentation remains material: the interval from caller probe to syscall
entry itself reaches 2.477..3.064 ms in three WF captures, outside the yield
interval counted in the table. These are not uninstrumented elapsed costs
or evidence that removing a syscall would recover all reported off-CPU
time. The next ordinary control should retain the same source and core
algorithm while disabling only the idle scan's yield rounds, keeping the
initial pause/look rounds and other yield sites unchanged. Compare short
and amortized CPU workloads against the frozen Rayon grain, with CPU and
context-switch costs as well as wall time. A later same-budget placement
control remains useful; neither policy is selected as a runtime default.

## 58. Remove speculative client receive probes

Experiment 56's checked operation counters show approximately one empty
receive probe for each completed echo round. Large-message cells have nearly
one successful receive per round, so their client cost cannot be explained
by high application-level fragmentation alone. This experiment tests whether
avoiding that probe changes ordinary throughput or CPU use. It does not
assume that the probe dominates, nor that all server forms reach the same
client limit.

`WF_NETLOAD_READINESS=1` adds one per-connection read-event mask. A delivered
EPOLLIN remains actionable while a partial send finishes and while the client
drains a partial response. EAGAIN clears the data bit. A complete response
also clears only the data bit: the protocol has one outstanding request, so
the next valid response cannot already exist before its request is sent.
EPOLLERR, EPOLLHUP and requested EPOLLRDHUP remain sticky across completed
frames, admission and exchange. The event is recorded before skipping a peer
that is finished, queued or waiting for its next scheduled arrival. An
already delivered read edge is never discarded merely because a send just
completed. No request bytes, full `memcmp`, per-trip clock calls, source
semantics or runtime ABI change. The default flag is zero and its normalized
optimized LLVM IR must equal the frozen 72fdd468 client.

The fixed screen uses `CLIENT_READINESS=1 CONTINUATION_SCREEN=4
NATIVE_BASELINES=1`, allocator combine mode, five passes and one warmup. It
requires server CPU 0 and client CPU 2 to be on different physical cores;
each role still has one worker and one logical CPU. No SMT sibling, profiler
or extra client worker is added. Existing server page/allocator policies and
all existing native/continuation qualifications remain. This resource-focused
panel omits the separately studied resident-memory cohort; ordinary process
RSS and both client/server CPU observations still accompany every row.

| Axis | Frozen choices |
| --- | --- |
| Server | Native epoll; native pure-ring uring with 64 KiB provided buffers; WF callee-small; WF continuation with owner progress, batch 32 and 1024 pending buckets |
| Client | Ordinary default and opt-in read readiness, service budget zero |
| Requests | 1/4 peers × 10,000 rounds × 64 B; 64 peers × 2,000 × 64 B; 1024 peers × 200 × 64 B; 64 peers × 500 × 64 KiB |
| Order | Adjacent client pairs within each server/cell/pass, both client order and server order alternate |
| Ordinary evidence | 4 servers × 5 cells × 2 clients × 5 passes = 200 rows, plus 40 warmup runs excluded from that table |
| Separate observations | One client-observed run per server/cell/client = 40 rows, with worker affinity, byte/outcome/bin/batch conservation |

The exact ordinary client binaries used in the screen first pass the real
socket fixture. It reads and verifies complete requests independently and
tests 64 B/64 KiB echoes, admission, 8 MiB bidirectional fragmented streams,
EOF, reset, truncated and corrupted responses. Error cases require both
no published timing and the intended diagnostic: corruption cannot pass
merely because a later round encounters EOF. Request bytes are drained before
EOF/truncation to avoid an unintended reset. Two additional observer builds
require actual short sends, short receives and send EAGAIN under the large
backpressure fixture, and preserve separate admission/exchange byte counters.
The two client policies at service budgets 0/1/8 also run admitted, scheduled
compute checks with a bounded process-group watchdog. Their retained results
must distinguish successful execution from timeout or early capacity failure.

The scripted qualifier executes the actual `pump`/`exchange` code with strict
send/receive/event sequences and a controlled monotonic clock. It covers a
read edge delivered before a partial send completes, bidirectional partial
progress, rearming after EAGAIN, terminal-only events, buffered admission
followed by EOF, an event on an already finished peer, the ninth queued peer
after the eight-item FIFO drain, and a terminal event while a light peer is
waiting for its scheduled arrival. Each returned epoll batch contains distinct
peers. Removing the event-recording operation must make the trace fail.

Local M1 evidence consists of 28 ASan/UBSan actual-loop traces through
temporary syscall/type shims (9/10/9 at budgets 0/1/8), Linux cross-compilation,
the default IR comparison, and shell/workflow checks. These shims do not
execute or qualify Linux epoll semantics. The separate real Linux socket,
mutation and paired timing evidence is recorded below. The source and harness
preserve the client's existing full-byte but modulo-256 message-identity
limitation.

`client-readiness.tsv` retains all ordinary rows and
`client-readiness-observed.tsv` holds the forty separate instrumented runs.
`client-diagnostic-counters.tsv` points to every observed raw report. Eight
selected client/server executables are copied to `retained/`, checked against
the actual launched files, and hashed with source/tool identities before the
screen; hashes are checked again afterward. The artifact includes these
executables, every raw sample and qualification log. Retained binaries make
independent rehashing possible, correcting experiment 56's missing-executable
limitation. Counter deltas are operations and bytes, not TCP packet counts;
instrumented rates never substitute for ordinary paired timing. A faster
client would improve this host's measurement headroom, without establishing
unrestricted server capacity or a universal fastest implementation.

### Qualified Linux result at 063cbef4

The readiness client removes the measured empty receive probes, but it is
not a uniformly better load generator. At 64 peers × 64 B, native uring
throughput falls in all five ordinary pairs and p99 rises in all five. At
64 KiB, all four servers still approach one full client CPU and their paired
median rate changes range from -0.1% to +2.3%. Keep the policy opt-in; this
screen does not establish spare client capacity or change the default client.

Frozen revision `063cbef4d182fac8c600c8ac2b6168ed4d24c4f6` passed
[run 34110355837](https://github.com/mbbill/Whitefoot/actions/runs/34110355837),
including Linux measurement job `101704901081` and Windows placement checks.
The [raw artifact 10014715642](https://github.com/mbbill/Whitefoot/actions/runs/34110355837/artifacts/10014715642)
has SHA256 `98628bd0051372ce8cf2215f76647cf094caa161a5f121f14e8155980dd5706d`,
verified against the downloaded 1,634,662-byte archive. The host is an Intel
Xeon Platinum 8370C VM, Linux 6.17.0-1022-azure and Clang 20.1.2. CPU 0 is
server core 0 and CPU 2 is client core 1; their SMT siblings 1 and 3 are not
added to either budget. The observed client worker reports CPU mask `2` in
all forty panel records. The same-revision io-hosts and io-bench runs passed;
the canonical gate was cancelled, so this is not a full-gate claim.

The real Linux qualification contains 28 actual-loop traces (9/10/9 for
service budgets 0/1/8), the deliberately removed read-edge mutation rejected
with exit 2, 48 ordinary socket cases, 16 additional observer socket cases,
and six admitted, scheduled compute checks. These are additional to the
local shim evidence above. The twelve successful qualification counter phases
preserve admission/exchange separation and exact sent, received and verified
bytes. Both 8 MiB observer clients actually encounter short sends, short
receives and send EAGAIN in both phases. The readiness policy still drains
partial responses to EAGAIN; eliminating speculative probes after full frames
does not eliminate the necessary partial-stream EAGAIN outcomes.

The independent raw audit verifies 200 ordinary rows, 40 distinct retained
warmup directories and 40 separate observed rows, including adjacent client
pairs and alternating order. Raw client outputs equal the table's rate,
latency, exchange CPU and byte/trip fields; client/server resource files also
match. Every ordinary client/server diagnostic channel is empty. Observed
send/receive outcome sums, positive-size bins, byte totals, verified rounds,
poll batches, event dispatch and budget-zero pump identities conserve.
Normalized default LLVM IR equals the retained pre-change source's IR. All
eight retained executables are independently rehashed and match their launched
binary manifest entries; all seven recorded source hashes match the frozen
revision. The Clang and WFC executable hashes are identifiers only: those two
tools were not uploaded or independently rehashed. The end-of-run manifest
check passed; it does not replace the separate checks on uploaded bytes.

The table reports the median of five same-pass readiness/default ratios,
with the full paired rate minimum and maximum. CPU is client exchange
user + system CPU per completed trip, measured in the ordinary binaries.
It is not the observer run's timing. A rate ratio above 1 is faster; CPU and
p99 ratios below 1 are lower. These are one-host screening samples, without
post-result tuning or an independent confirmation cohort.

| Peers × bytes | Server | Rate ratio [min, max] | Client CPU/trip ratio | p99 ratio |
| --- | --- | --- | --- | --- |
| 1 × 64 | epoll | 0.9906 [0.9781, 0.9982] | 0.9889 | 1.0476 |
| 1 × 64 | uring-64k | 0.9995 [0.9676, 1.0211] | 0.9739 | 1.0000 |
| 1 × 64 | callee-small | 0.9626 [0.9528, 1.0517] | 0.9964 | 1.0571 |
| 1 × 64 | wf-coro-index | 1.0086 [0.9905, 1.0290] | 0.9767 | 0.9556 |
| 4 × 64 | epoll | 1.0198 [1.0091, 1.0240] | 0.9705 | 1.2121 |
| 4 × 64 | uring-64k | 1.0569 [1.0398, 1.0651] | 0.9450 | 1.0000 |
| 4 × 64 | callee-small | 0.9775 [0.9474, 0.9992] | 0.9823 | 0.9767 |
| 4 × 64 | wf-coro-index | 1.0102 [0.9949, 1.0141] | 0.9648 | 1.0000 |
| 64 × 64 | epoll | 0.9979 [0.9731, 0.9995] | 0.9935 | 1.0265 |
| 64 × 64 | uring-64k | 0.9648 [0.9539, 0.9831] | 0.9885 | 2.6023 |
| 64 × 64 | callee-small | 1.0111 [0.9957, 1.0259] | 0.9468 | 0.9802 |
| 64 × 64 | wf-coro-index | 1.0092 [1.0017, 1.1341] | 0.9361 | 1.0000 |
| 1024 × 64 | epoll | 0.9975 [0.9348, 1.0396] | 0.9617 | 1.0392 |
| 1024 × 64 | uring-64k | 1.1319 [1.0623, 1.1834] | 0.8699 | 0.8930 |
| 1024 × 64 | callee-small | 0.9913 [0.8169, 1.1380] | 0.9711 | 1.0266 |
| 1024 × 64 | wf-coro-index | 1.0684 [0.9039, 1.2028] | 0.9236 | 1.0231 |
| 64 × 65536 | epoll | 1.0225 [0.9935, 1.0375] | 0.9780 | 0.9481 |
| 64 × 65536 | uring-64k | 1.0096 [0.9786, 1.0347] | 0.9908 | 1.1427 |
| 64 × 65536 | callee-small | 1.0041 [0.9818, 1.0169] | 0.9963 | 0.9908 |
| 64 × 65536 | wf-coro-index | 0.9989 [0.9967, 1.0241] | 1.0001 | 1.0146 |

The 64-small uring result is a regression: rate ratios are 0.9539..0.9831,
and p99 ratios are 1.0251..3.1556, with no favorable pair in either metric.
Conversely, uring improves at 4 and 1024 small-message peers in all five rate
pairs. The 1024-peer WF cells are variable: indexed continuation rates span
0.9039..1.2028 and callee rates span 0.8169..1.1380. The indexed row's median
paired increase is 6.8%, but its unpaired absolute rate medians are only
165,186 → 165,750 trips/s. Retain both facts and all samples; neither metric
alone establishes a stable improvement. Epoll at four peers gains 2.0% rate
while its p99 is worse in all five pairs (1.1471..1.2500).

At 64 KiB, median client exchange CPU/wall is 0.996..1.000 before and
0.997..1.000 after the change across the four forms. Median client system
CPU remains about 19.0..19.6 microseconds/trip, versus 2.2..2.8 microseconds
of user CPU. Thus removing the empty probe does not solve the observed
large-transfer client limit. The artifact also retains server lifetime CPU,
RSS and context switches; centisecond `/usr/bin/time` CPU and startup/drain
cost prevent interpreting small server CPU differences as fine-grained
steady-state savings.

Separate counter observations confirm the intended operation change: all
sixteen small-message readiness cells have zero receive EAGAIN, versus
approximately one per completed round in their default controls. The four
large-message readiness cells have 0, 1, 3 and 1 receive EAGAIN outcomes for
epoll, uring, callee and indexed continuation respectively, across 32,000
rounds each; their controls have 32,194, 32,202, 32,654 and 33,031. All forty
panel samples have exactly one successful send per round and no send EAGAIN;
large-response positive receive counts remain close to one per round.

Reducing receive calls can also change event batching. The following are
one separate observer pair per row, not a latency profile or an explanation
of the ordinary timing by itself:

| 64 peers, server/payload | Receive calls/trip, default → ready | Polls/trip, default → ready | Events/ready poll, default → ready |
| --- | --- | --- | --- |
| epoll / 64 B | 2.0000 → 1.0000 | 0.0240 → 0.4025 | 41.75 → 2.48 |
| uring / 64 B | 2.0000 → 1.0000 | 0.1775 → 0.2332 | 5.63 → 4.29 |
| epoll / 64 KiB | 2.0062 → 1.0000 | 0.0159 → 0.0159 | 63.25 → 63.22 |
| uring / 64 KiB | 2.0063 → 1.0002 | 0.0208 → 0.1153 | 48.42 → 8.83 |

These are application operations and event batches, not TCP packet counts.
Observer instrumentation and changed scheduling can affect aggregation;
these rows do not isolate a kernel cause for the uring tail regression.
No incomplete experiment 56 stack profile is used to supply that missing
attribution. Extra independent physical client resources or another host,
and a qualified client engine comparison, remain possible next controls.
Neither is measured here. Existing throughput proximity between WF and native
servers therefore remains an end-to-end result with unresolved client limits.

### Qualification fixture capacity follow-up

The canonical scheduler [job 101708097217](https://github.com/mbbill/Whitefoot/actions/runs/34111354204/job/101708097217)
at `a1142e5fd06d38e95b09e38b8c8b7c50ac2bc1ed` reached its existing eight-minute
job limit during the first observer client's 8 MiB fixture, after all 48
ordinary socket cases had passed. Each completed fragmented case took about
12.25 seconds, similarly for both client policies and all three service
budgets. Even the ordinary 64 KiB echo fixture took about 85 ms for three
rounds and 127 ms with admission. These regular delays suggest interaction
between TCP acknowledgements, the 4 KiB server send-buffer request and the
fixture's blocking read-then-write loop; they do not prove that kernel cause.

The bounded follow-up changes only the fixture's requested server
`SO_SNDBUF` from 4096 to 65536 bytes. It retains all cases, three exchange
rounds plus admission, the 8 MiB payload, the initial one-byte and later
8191-byte fragment limits, full independent request verification, exact
error checks and all observed short-send/short-receive/send-EAGAIN assertions
in both phases. A faster fixture is acceptable only if those real-operation
assertions still pass. Benchmark client/server code, measured sample counts
and the gate timeout remain unchanged. The frozen 063cbef4 measurements above
retain their original fixture revision.

The native [fixture control at e424b800](https://github.com/mbbill/Whitefoot/actions/runs/34114500864)
passes on EPYC 7763, four logical CPUs on two reported SMT cores, Linux
6.17.0-1022-azure and Clang 18.1.3. Its full `client-readiness-check`, including
compilation, takes 10.77 seconds: all 28 actual-loop traces, the lost-edge
mutation rejection, 48 ordinary and 16 observed socket cases, and six paced
compute checks pass. The same-host old/new fixture comparison reuses identical
observed client binaries, with old/new order for the default client and
new/old for readiness. Each row includes all eight socket cases:

| Observed client | Old 4 KiB fixture wall seconds | New 64 KiB fixture wall seconds |
| --- | ---: | ---: |
| Default | 12.52 | 0.17 |
| Readiness | 12.54 | 0.19 |

These are one observation per fixture/client pair, not an ordinary throughput
panel. Both new fixtures still encounter partial sends and receives and send
EAGAIN in admission and exchange; their send-EAGAIN counts are respectively
634/1970 and 686/1955. All 36 retained counter phases conserve operation,
size-bin and byte totals, and all 96 socket-case results have the expected
order and shapes. The audit rehashes four retained comparison executables and
five source entries, including the old fixture; shared client dependencies
equal their frozen 063 bytes and the end manifest check passes. Artifact
10015715346 has ZIP SHA-256
`7212ebd66ed9e43a7c30e1eb169e83ab5531ed655525a14058f1b6ee53106204`.
The current Linux scheduler [gate job at cf998b56](https://github.com/mbbill/Whitefoot/actions/runs/34113936064/job/101716322437)
also passes with the same fixture correction and unchanged eight-minute
limit. This resolves the observed fixture runtime problem while retaining
coverage; it does not identify the kernel mechanism or improve a measured
benchmark client policy. Full gate status remains revision-specific.

## 59. Ordinary CPU control without idle-scan yields

Experiment 57 locates the dominant observed early yield wait in the runtime's
idle scan, but adds traps and does not record whether ready work exists.
This experiment tests the performance consequence with ordinary binaries.
The candidate uses the existing `WF_SCHED_IDLE_YIELD_ROUNDS=0u` build override
instead of the default 16. It retains 256 pause/look rounds, the epoch and
last-look wake protocol, and all COMPLETING, exhausted-compute and startup
yield sites. No WF source construct, runtime ABI or default changes.

`make rayon-idle-bench` reuses `rayon-bench.sh` with `RESOURCE_CONTROLS=4`.
It fixes four computing threads, twelve WF stacks and the earlier Rayon
grain-four calibration. The short and amortized cases use 1/16 source
batches. Each has four forms: ordinary compiler WF, same-IR manual default,
same-IR manual idle-yield0, and Rayon. Five alternating passes after one
warmup produce forty ordinary samples. The manual default retains a check
against the ordinary executable's bytes; if they differ, both controls and
their code/layout evidence remain rather than assuming equivalent links.
These are fresh builds on one host, not a new timing of the frozen 8dd
executables. Do not pool absolute times with experiments 45, 55 or 57.

Before any performance row, the candidate runs the full existing completion
test target with its build override. This includes real scheduler smoke,
completion/helper checks and the unchanged four interleaving enumerations.
The enumerator already fixes one pause/look round and zero yield rounds to
model the wake protocol; passing it does not simulate host scheduling or
establish that an arbitrary waiting policy is fastest. Normal and candidate
programs must produce the independent expected bytes. Four separate observed
executions check four workload threads, three started workers, real grants,
spin rounds 256, yield rounds 16/0, default storage flags and idle counters.
Observed counters remain separate from the performance rows.

The retained ordinary rows include wall, user/system CPU, voluntary and
involuntary context switches and maximum RSS. Commands, tools, emitted IR,
source hashes and executables accompany the results; Linux disassembly
supports checking which call path disappeared. `codex/io-cpu-idle-controls`
runs this panel on Linux without profiler probes or changed affinity. Lower
wall time must be considered with CPU cost and both batch lengths; a negative
result rejects this isolated candidate, not the ownership-based parallel
model. Available-work and placement questions remain distinct. The frozen
Linux result below does not select the zero-yield candidate as a default.

### Frozen idle-wait result

Revision `e0a05efcae42bee9a622e68fcf62b614c94121d8` completed
[run 34110933420](https://github.com/mbbill/Whitefoot/actions/runs/34110933420),
including CPU job 101706761849 and Windows placement qualification. Artifact
10014504854 has ZIP SHA-256
`6bbc94c2c1eb37270e0b4e493fad1a98ef3d000eec45e7cee1d4e160ccca3354`.
The host reports EPYC 9V74, four logical CPUs on two SMT cores, Linux
6.17.0-1022-azure, Clang 18.1.3 and Rust 1.98.0. Processes inherit CPUs 0-3;
the experiment adds no affinity. This is a shared hosted runner cohort.

The artifact audit recomputes all nine distinct retained executable/IR
hashes and 22 frozen source hashes. The compiler executable hash is recorded
but its file is not retained, so that hash is not independently recomputed.
Ordinary and manual-default WF executables are byte-identical. The freshly
built ordinary WF, sequential WF, Rayon and emitted IR also match the earlier
8dd artifacts byte-for-byte. Manual candidate/default commands differ only
in the yield-round value and output filename. Actual ELF instruction bytes
confirm eight default calls to `wf_prim_yield` and seven candidate calls:
only the idle-scan call at `0x4df0` disappears; the seven other sites retain
their offsets and targets.

All forty ordinary rows have the expected plan labels, five alternating
passes after one warmup and complete eight-column resource records. Summary
medians and ranges reconstruct from the raw rows. Native-ring and helper
completion routes, scheduler smoke, all four unchanged enumerations and five
full Linux completion-harness runs pass. The owner-ring-only probe is
inapplicable to this default-ring build and takes its existing status-77
path. Four separate observed runs
confirm the expected output, actual workers/grants, 256 spin rounds, 16/0
yield rounds and unchanged storage/progress settings.

The paired comparison uses candidate/manual-default ratios within each pass.
Positive deltas mean greater candidate cost; the range contains all five
pass deltas rather than a confidence interval.

| Work | Default / idle0 median wall ms | Paired wall delta, median (range) | Paired total CPU delta | Paired voluntary-switch delta |
| --- | ---: | ---: | ---: | ---: |
| 1 batch | 397.428 / 402.280 | +1.196% (-0.643..+2.739%) | +1.244% | +51.939% |
| 16 batches | 5757.071 / 5755.385 | +0.114% (-2.147..+1.068%) | +0.022% | +5.383% |

The long-row ratio of separate medians has a different sign from the median
of paired ratios; neither is a material speedup. The byte-identical
manual/ordinary short-task control itself ranges from -1.221% to +2.271%
across pairs, comparable to the small candidate wall effect. Short-task total CPU rises
in every pass, by 0.196..1.400%, while its voluntary switches rise by
45.643..78.411%. Involuntary switches vary in both directions. WF remains
behind Rayon: ordinary/idle0 paired wall gaps are respectively 12.422/13.552%
for one batch and 2.117/2.114% for sixteen. These gaps belong to this cohort,
not an aggregate of earlier runners or a claim about all CPU workloads.

The separate observed idle-wait counts rise from 21 to 356 for one batch
and from 162 to 371 for sixteen. Together with the ordinary voluntary-switch
increase, this is consistent with reaching the blocking wait more often
after shortening the idle window. The observed executions are instrumented
and are not the timing rows; they do not assign a wall-time cost to each park
or prove ready work was available. The result rejects simply removing these
yield rounds as the proposed performance fix for this panel. It does not
reject the ownership-based parallel model. Keep the default and distinguish
ready-work availability from OS placement before another waiting-policy
change; experiment 57's off-CPU intervals are not recoverable-time estimates.

## 60. Client capacity on reported independent ARM cores

Experiments 43 and 58 leave the occupied echo client's CPU limit unresolved.
Experiment 43 added a second client worker on the same physical core's SMT
sibling. Experiment 58 reduced speculative receive calls but retained one
client CPU; its small-message regressions keep readiness opt-in. This screen
keeps the original default/service0 client and tests additional reported
physical client cores on one Linux AArch64 host. It changes no server
algorithm, private buffer capacity, continuation batch/index policy or source
language/runtime ABI.

The existing `scheduler-bench.sh` harness, reached through
`make scheduler-client-capacity`, uses the conditional
`codex/io-client-capacity` workflow route. GitHub's
[standard runner documentation](https://docs.github.com/en/actions/reference/runners/github-hosted-runners)
advertises four vCPUs for public `ubuntu-24.04-arm` runners. That is an
availability hypothesis: the harness requires native AArch64, four allowed
reported package/core pairs, matching `lscpu --physical` and sysfs identities, each CPU in
its own sibling list, and no selected CPUs sharing a sibling list. It records
all allowed topology, selected sibling lists, page size, image/kernel/tool
versions, cgroup-v2 ancestry, effective cpusets, quota and throttling counters.
Every visible finite CPU quota must allow at least four CPUs. Admission fails
before compilation/qualification when these conditions cannot be checked.
Missing or unreadable non-root quota metadata fails admission. The actual
cgroup-v2 root has no `cpu.max` interface, as defined by the kernel's
[CPU interface documentation](https://www.kernel.org/doc/html/latest/admin-guide/cgroup-v2.html#cpu-interface-files);
that root absence is recorded explicitly. If a namespace root exposes
`cpu.max`, its value is checked. Initial/final topology and canonical selection
snapshots must match, including allowed CPUs and selected sibling identities;
changing quota/stat counters are retained separately from this equality check.
Virtual topology does not establish dedicated host cores, eliminate noisy
neighbors or reveal a hidden host quota.

| Axis | Frozen settings |
| --- | --- |
| Server | `epoll`, `uring-64k`, `callee-small`, `wf-coro-index`; one worker/resumer on the first selected CPU |
| Client | Default receive behavior, service budget 0; widths 1/2/3 use the next 1/2/3 selected CPUs, with no server sibling |
| Work | 64 peers × 2,000 trips × 64 B; 1,024 × 200 × 64 B; 64 × 500 × 64 KiB |
| Protocol | Fresh connections, one outstanding request per peer, original bytes and full `memcmp`, original two clocks; NODELAY and normal errors retained |
| Server storage/policy | Existing 64 KiB echo capacity; glibc top pad 0, THP disabled; native provided-pool storage and WF private storage remain distinct; indexed continuation owner/batch32/window1024 |
| CPU scope | All server OS threads inherit its one-CPU mask; all client workers inherit their whole selected pool; no per-worker pinning and no SQPOLL |
| Repeats | One warmup plus five ordinary passes; four servers × three cases × three widths = 36 rows per pass |
| Observation | One separate default observed-client execution per server/case/width = 36 rows and 72 exchange worker reports; ordinary servers, no profiler |

Within each server/case/pass, the three widths run adjacently. The warmup uses
1/2/3; ordinary width orders are frozen as 1/2/3, 3/2/1, 2/3/1, 1/3/2 and
3/1/2. Server order alternates forward/reverse. Thus each width occupies every
position once or twice before any result is inspected. Connection partitioning
remains the original contiguous assignment: 64 peers split 22/21/21 and
1,024 split 342/341/341 at width 3. Per-worker epoll ownership and aggregation
are part of this worker-count experiment; extra allowed CPUs are not isolated
from that existing partitioning change.

The normal native ring, large-stream, helper-control, continuation sanitizer,
client service/observer and full readiness qualification paths remain wired.
Unsupported io_uring or toolchain setup is a failed qualification, with no
substitute backend or timing-only success. The selected release default
client is copied from the exact qualified service0 binary. Its optimized IR
must equal the frozen pre-observer client. Before timing, two additional
observed admitted runs use four peers across three workers, at 64 B and
64 KiB. Their 2/1/1 ownership and both admission/exchange byte and round totals
are checked separately; the twelve phase reports are qualification evidence,
not part of the 72 panel reports.

A small retained launcher reads `/proc/<pid>/status` after `taskset`, checks the
actual mask, then immediately `exec`s the client or existing server launcher.
The PID/mask therefore belongs to the launched process; new threads inherit
that mask. Every observed client additionally reports its actual worker mask.
These records establish launch and observed worker affinity, not a continuous
trace of CPU residency or individual server-thread execution. The launcher
runs before the client's exchange clocks; its setup work is included in the
existing whole-process `/usr/bin/time` resource figures. Client exchange
`getrusage` still measures aggregate client process CPU across all workers.
Visible quota/stat metadata and kernel CPU/softirq/socket counters are retained
before and after the panel for audit; they do not assign kernel work to a
particular request.

Expected artifacts are 180 ordinary rows, 36 warmups preserved in sample
folders/logs, 36 separate observed rows, 72 worker reports, and the two uneven
smokes with twelve phase reports. `retained/` contains six selected ELF files:
the default ordinary/observed clients and four ordinary servers. The manifest
checks these copies against their used binaries and records selected source,
generated IR/source and launcher hashes before/after timing. Compiler/tool
hashes identify the installed tools; those tool executables are not uploaded
or independently rehashed by an artifact consumer. Existing generated-host
codegen artifacts and qualification logs remain available. This screen adds
no profiler capture and preserves all existing branch routes.

Evaluate width2/width1 and width3/width1 rate, tails, server CPU/trip and
aggregate client exchange CPU/trip **within each same-host server/case/pass**.
A repeatable rate increase with the server fixed exposes client-resource
sensitivity; a decrease falsifies the claim that widening this existing client
improves that cell. A flat response does not prove an unrestricted server
ceiling: worker imbalance, per-worker event batches, loopback/kernel limits,
VM contention and the measured server's own capacity may differ by form.
Inspect all five paired ratios and CPU/latency effects before adopting a wider
client for later panels. Do not combine absolute ARM rates with the earlier
x86 results or choose a new algorithm from these confirmation samples.

Local checks pass: the extracted actual driver produces the required counts,
orders and masks; eleven preflight negatives reject mismatched modes; synthetic
admission cases cover root absence, finite quotas, sibling/core disagreement
and initial/final topology changes. Four synthetic proc-readback cases check
the launcher's canonical mask and actual PID preservation through `exec`.
All 44 prior workflow branch routes/settings remain equivalent and 22 shell
steps parse. Strict Linux AArch64 cross-compilation links six native-reference,
client and fixture ELF files, and compiles the scheduler host/native ring units;
the two selected WF benchmark executables are not part of this cross-link set. The
default client optimized IR equals the frozen pre-observer client on that
target. `make static` and patch checks pass. These local checks use filesystem
shims for Linux admission and do not execute the Linux ELF files. They alone
do not qualify native topology/socket/ring behavior or performance; the native
run and its evidence limits are recorded below.

### First ARM admission: f2e0f460

The frozen `f2e0f460b86c66c31098b0f1c1b8ae2077bf79e1` allocator job
`101726879322` in run
[34117260936](https://github.com/mbbill/Whitefoot/actions/runs/34117260936)
failed before compilation or runtime qualification. Artifact `10016763576`
contains four admission files (1,123-byte ZIP, SHA-256
`b4106dcb17fb7062b49e253acaf093013adbc30372790371f10387113616e3bf`).
The guest reports AArch64, Linux `6.17.0-1022-azure`, image
`20260831.111.1`, 4 KiB pages and allowed CPUs 0..3. The retained default
`lscpu -b -p=CPU,CORE,SOCKET` rows are `0,0,0` through `3,3,0`, while the
first sysfs read reports CPU 0 as package 36/core 1, siblings `0`. The equality
guard returns status 2 at that first comparison; its stderr was empty.
Replaying the captured table and first CPU's fields through the frozen
admission source reproduces this exact stopping point. No later CPU's raw
IDs, quota checks, io_uring qualification or performance samples were captured.

This comparison mixed two ID namespaces. The
[util-linux manual](https://man7.org/linux/man-pages/man1/lscpu.1.html)
defines `--physical`/`-y` as selecting kernel-provided topology IDs while
leaving CPU logical numbers unchanged. The
[upstream implementation](https://github.com/util-linux/util-linux/blob/v2.39.3/sys-utils/lscpu.c)
uses `__fill_id` to choose between those IDs and logical topology-map indices.
Thus the retained mismatch exposes an admission implementation error; it
does not show shared physical cores. It also cannot establish that all four
raw pairs will qualify on this or the next guest.

The correction requests `--physical` only for the capacity panel, retaining
strict raw package/core equality, composite pair uniqueness, complete sibling
disjointness, quota checks and initial/final snapshots. Earlier panels keep
their original logical grouping. Admission now records the `lscpu` version
and ID mode before reading topology, and a mismatch names the CPU, expected
sysfs package/core and actual table rows. There is no fallback from missing or
inconsistent physical IDs. Synthetic sparse-ID cases exercise this distinction.
The subsequent corrected run is recorded below; the failed revision and its
admission evidence remain unchanged.

### Corrected ARM qualification and client-capacity results: d241cf7d

Frozen revision `d241cf7d59b0e2724e44046358f10fa32ef350ec` passes allocator
job `101729631995` and Windows placement job `101729631817` in
[run 34118128572](https://github.com/mbbill/Whitefoot/actions/runs/34118128572).
Artifact `10017548083` is a 1,846,028-byte ZIP with SHA-256
`14ddc221705cf7ab83402eeef0fb0ea3dcef09fe25e09bc5a11d61a701a16304`.
The archive digest and CRC, source revision, all six retained AArch64 ELFs,
eight selected source hashes, retained generated source/IR and launcher hashes
were independently checked. Each retained executable equals its used-binary
hash; final hash checks agree with the initial manifest. The Clang, Clang++ and WFC hashes identify
tools recorded by CI, whose executables are not uploaded or independently
rehashed. Default-client optimized IR matches the frozen pre-observer client.

The guest reports Linux `6.17.0-1022-azure`, image `20260831.111.1`, util-linux
2.39.3 and 4 KiB pages. Allowed CPUs 0/1/2/3 map to package 36, cores 1/2/3/4,
with singleton sibling lists 0/1/2/3. Initial/final selection and topology
bytes agree. The server uses CPU 0; client pools use 1, then 1/2, then 1/2/3.
All retained launch masks and observed worker masks match. Visible non-root
ancestor quotas are `max 100000`, effective cpusets are 0–3, and all recorded
initial/final throttling counts are zero; the root correctly lacks `cpu.max`.
These checks establish the guest's reported separation and visible quota
conditions, not dedicated underlying host cores or absence of hidden limits.

The artifact contains exactly **180 ordinary rows, 36 warmups, 36 separate
observed rows and 72 exchange worker reports**. Actual sample order follows
the frozen width/server rotation. The two additional admitted width-3 smokes
have the required 2/1/1 peer assignment and twelve distinct phase reports;
they are not panel rows. Every ordinary/observed sample is bound to its
cohort, form, peer count, message size, pass and launch masks. Worker round,
byte, syscall-outcome, size-histogram and event/pump identities pass, as do
server exit/status and quiet-output checks. The preserved raw client and
`/usr/bin/time` records retain rates, tails and resource observations for
every pass; observations do not enter the ordinary timing comparisons.

Native qualification includes eight uring stream configurations (8/64 KiB,
ring/inline sends, 1/4 workers), the actual 64 KiB provided-pool preflights,
epoll storage readbacks, compact-stack controls and indexed owner/batch32
continuations. Client qualification retains 28 actual-loop trace cases,
the deliberate lost-edge rejection, six paced cases and 64 real socket cases
(48 ordinary plus 16 observed). Twelve observed socket phases include the
8 MiB fragmented transfer, with actual short sends, short receives and send
EAGAIN still required. This native execution evidence is distinct from the
earlier local filesystem/scheduling shims. It is a successful specialized
screen, not a claim that this measurement revision ran the full canonical gate.

The following are medians of **five same-server, same-case, same-pass rate
ratios**, wider client divided by width 1; brackets contain all-five minimum
and maximum. Ratios of separately reported median rates are not substituted.

| Peers × bytes | Server | Width 2 / 1 rate [min, max] | Width 3 / 1 rate [min, max] |
| --- | --- | --- | --- |
| 64 × 64 B | epoll | 0.8850 [0.8761, 0.8891] | 0.8488 [0.8236, 0.8560] |
| 64 × 64 B | uring-64k | 0.9247 [0.9200, 0.9319] | 0.8513 [0.8235, 0.8689] |
| 64 × 64 B | callee-small | 0.8787 [0.7848, 0.9560] | 0.8529 [0.7220, 0.8820] |
| 64 × 64 B | wf-coro-index | 0.8092 [0.7684, 0.9420] | 0.7833 [0.7168, 0.8818] |
| 1024 × 64 B | epoll | 0.8997 [0.8792, 0.9591] | 0.7892 [0.7721, 0.9041] |
| 1024 × 64 B | uring-64k | 1.0355 [0.9898, 1.0441] | 0.9734 [0.9265, 0.9946] |
| 1024 × 64 B | callee-small | 0.6960 [0.6706, 0.7263] | 0.6498 [0.6184, 0.7146] |
| 1024 × 64 B | wf-coro-index | 0.7099 [0.6969, 1.0520] | 0.6690 [0.6548, 0.8220] |
| 64 × 64 KiB | epoll | 1.4364 [1.3970, 1.5755] | 1.3943 [1.3693, 1.5238] |
| 64 × 64 KiB | uring-64k | 1.1449 [1.1286, 1.2018] | 1.1716 [1.1639, 1.2145] |
| 64 × 64 KiB | callee-small | 1.0932 [1.0755, 1.1139] | 1.0651 [1.0176, 1.1285] |
| 64 × 64 KiB | wf-coro-index | 1.1179 [1.0970, 1.1185] | 1.0758 [1.0597, 1.1237] |

Every large-message wider/base rate pair improves, but width 3 is not
generally better than width 2. All four 64-peer small-message cells regress
in all five passes at both wider settings. At 1024 small peers, uring width 2
improves in four passes, with a median of only 3.55%; indexed width 2 improves
in one pass despite its 29.0% median regression. The remaining wider settings
in that cell regress in all five passes. These outcomes reject a blanket
wider-client default.

Large-message latency and CPU use also matter. The next table uses the same
paired ordinary ratios. Client CPU/trip sums exchange user and system time;
server CPU/trip is whole-process lifetime user plus system time. The last
column is the median aggregate client exchange CPU/wall at the wider setting;
each form's width-1 median is about 0.999.

| Server | Client width | p99 ratio | Client CPU/trip ratio | Server CPU/trip ratio | Client CPU/wall |
| --- | --- | --- | --- | --- | --- |
| epoll | 2 | 0.7048 | 1.0383 | 0.8594 | 1.4894 |
| epoll | 3 | 0.6982 | 1.0414 | 0.9206 | 1.4506 |
| uring-64k | 2 | 1.1133 | 1.0515 | 1.0152 | 1.2169 |
| uring-64k | 3 | 0.8130 | 1.0429 | 0.9851 | 1.2228 |
| callee-small | 2 | 0.9183 | 1.0291 | 0.9200 | 1.1243 |
| callee-small | 3 | 0.9242 | 1.0531 | 0.9467 | 1.1211 |
| wf-coro-index | 2 | 0.8979 | 1.0099 | 0.9859 | 1.1228 |
| wf-coro-index | 3 | 0.9002 | 1.0393 | 1.0141 | 1.1325 |

The large uring width-2 p99 ratio spans 0.8113–2.5755, worsening in three of
five passes despite every rate pair improving. Its small 64-peer width-2
p99 worsens in all five, median 2.7962 [2.1981, 2.8594]; width 3 also worsens
in all five, median 1.1502 [1.1258, 3.1509]. Thus higher allowed CPU count is
not a reliable tail-latency improvement. Whole-process server CPU includes
startup, launcher and drain, with centisecond output, so small differences
there do not resolve a steady-state per-request cost. Aggregate client CPU
above one demonstrates work using the broader pool; it is not a guarantee
that each worker has headroom or that all relevant kernel work is charged.

The client envelope changes even the apparent server ranking. For 64 peers
at 64 KiB, these are same-pass **server / epoll rate ratios at equal client
width**, again with all-five ranges:

| Server / epoll | Width 1 | Width 2 | Width 3 |
| --- | --- | --- | --- |
| uring-64k / epoll | 1.0420 [1.0103, 1.1074] | 0.8330 [0.8212, 0.8692] | 0.8777 [0.8613, 0.8884] |
| callee-small / epoll | 1.0813 [1.0543, 1.1608] | 0.8237 [0.7930, 0.8550] | 0.8130 [0.7872, 0.8679] |
| wf-coro-index / epoll | 1.0351 [1.0062, 1.1140] | 0.7905 [0.7837, 0.8320] | 0.8024 [0.7686, 0.8375] |

All five passes reverse each of these comparisons. Epoll's separate rate
medians are 40,246.9 / 57,954.2 / 56,053.1 trips/s at widths 1/2/3; indexed
WF's are 41,476.7 / 45,979.5 / 44,915.8. A one-client result slightly favoring
WF over epoll therefore cannot establish a server advantage: this unchanged
epoll implementation advances much further when this guest allows the
existing client to use another reported core. This is a within-ARM result;
it does not numerically revise earlier x86 cohorts or establish epoll as the
unrestricted best implementation.

This large cell retains uring's 32 provided 64 KiB buffers (2 MiB total)
versus WF's private initialized 64 KiB source capacity per peer (4 MiB at
64 peers). Capacity is not resident memory, and the storage models remain
different. Neither the provided-pool capacity nor the existing inline-send
candidate was varied under the wider client here. Their earlier screening
does not establish the best native configuration for this new client envelope;
no part of this result attributes uring's gap to pool capacity.

The separate observed rows expose another effect of increasing the worker
pool. Every row still uses exactly one send call per verified trip. At
64 KiB, receive calls/trip range from 2.0006 to 2.0963 and receive
EAGAIN/trip from 0.9993 to 1.0666. For epoll's large cell, events per nonempty
epoll wait fall 62.825 → 1.168 → 1.033 while waits/trip rise
0.01591 → 0.91031 → 1.00816 across widths 1/2/3. The native 64-peer small
cells likewise go from roughly 54–61 events per nonempty wait to roughly
1.35–1.41 at width 2. These are client epoll event batches and syscall counts,
not TCP packet counts. There is one instrumented observation per configuration;
it does not assign causal timing costs or replace the five ordinary pairs.

The measured variable combines more allowed client CPUs, more client workers
and a different connection partition across their epoll instances. The
server/client scheduling and loopback kernel interactions can consequently
change. The successful large-message sensitivity and ranking reversal are
strong evidence that the previous one-client envelope can mask differences
between these servers; they do not isolate a copy, syscall or scheduler cost,
nor prove a new server ceiling. Keep the unchanged default/service0 client,
retain the small-message regressions, and carry client width and architecture
as explicit comparison axes before declaring a performance frontier.

## 61. Control parallel tree initialization

Experiment 59's ordinary control gives no wall improvement from removing
idle-scan yield rounds. Keep those defaults. One remaining structural
mismatch precedes the measured layout work: the generated WF builder makes
63 parallel acquisition attempts while allocating its 127-node tree; the
Rayon reference builds that tree sequentially inside its existing pool.
WF starts its workers lazily at the first acquisition. The small node count
neither proves negligible cost nor attributes the short-run gap to allocation.

`make rayon-build-bench` selects `RESOURCE_CONTROLS=5` in the existing CPU
harness. Its candidate changes only `wf_main`'s call from `wf_build` to the
already-emitted `wf__par_seq_build`. The transform requires exactly one main,
both builder definitions and one applicable call, then reverses that scoped
substitution and compares the entire original IR byte-for-byte. All layout
functions, recursive acquisition fallbacks, source loops and runtime units
remain unchanged. This is an experimental executable control, not a compiler
rule, source-language change or selected lowering default.

The intervention also moves worker creation from the first tree-building
pair to the first layout pair. Its result therefore measures parallel
initialization and that lazy-start relocation together, not isolated malloc
cost. Both programs construct the complete tree and table before traversing
it; no work moves outside the measured process. Both execute all 800 full
8192-word layouts per batch before all 800 banded 4096-word layouts per
batch. The parent table scan still precedes child publication. WF still
attempts acquisition at all 63 branch nodes per layout, while Rayon grain
four uses 15 joins; that granularity difference is not changed here.

Four computing threads, twelve WF stacks and the frozen Rayon width/grain
4/4 are fixed. The 1/16-batch panel contains ordinary compiler WF, an unchanged
manual link, the manual sequential-build candidate and Rayon. One complete
warmup precedes five alternating passes: forty ordinary wall, user/system
CPU, context-switch and peak-RSS rows. The duplicate WF baseline records
whether ordinary/manual executables are byte-identical. Only an identical
pair exposes same-binary sampling variation; otherwise both links and their
code/layout evidence remain explicit controls.
The normal completion suite runs once with default runtime settings; no
redundant candidate runtime variant is introduced. Both manual executables
must print the independent exact checksum at one and sixteen batches before
timing. Four separate observed executions check those bytes, four scheduler
threads, three started workers, grants, default storage settings and the
unchanged 256 pause/look and 16 idle-yield rounds. Their counters are not
assigned to ordinary samples.

The artifact retains both IR files and their exact diff, every selected
executable including the compiler, link commands, plans, resource records,
qualification outputs, observer reports and source hashes including backend
headers. Linux objdump decodes the actual main/body calls and requires the
candidate to call the sequential builder without calling the parallel one.
Hot layout functions are compared after normalizing instruction addresses
and symbolic relocation displacements; any difference is retained explicitly
for attribution review rather than silently called equivalent. Original
symbols and disassembly remain available. Source-level node-write parity is
not a claim of identical executed stores: the retained WF layout IR already
omits node-output stores whose values are not published by this workload.
No additional stores are removed by this control.

The `codex/io-cpu-build-control` route runs this panel without phase probes,
new affinity, changed waiting defaults or a new calibration cohort. An
initialization effect would not prove ready work was available during earlier
yields. Distinguishing insufficient exposed work from OS scheduling delay
requires contemporaneous eligible tasks/READY continuations and per-thread
scheduling state; aggregate grants, failed acquisitions and idle counters
cannot establish that relation. Local command and checksum evidence and the
native Linux measurement are recorded separately when available.

Local M1 qualification passes the normal completion suite, all four existing
scheduler enumeration configurations, Rayon tests/fmt/clippy, both manual
batch-length checks and a one-pass/no-warmup smoke with eight ordinary rows
and four separate observations. The initial qualification attempt passed
explicit default yield macros into the enumerator's zero-round model and
failed on a macro redefinition; the harness now uses the unchanged normal
qualification flags. All nine ordinary artifact hashes, eleven full artifact
hashes and 34 source entries verify after the panel. The Mach-O ordinary and
manual-default files differ, while their complete disassemblies match after
removing the file-heading line. Local timings are command/mapping evidence,
not a performance comparison.

The exact IR transform rejects six malformed inputs, and the actual Bash
3.2 compiler-array checks preserve every prior mode's commands. All 45 prior
workflow branch admissions and CPU target selectors remain equivalent;
invalid mode/profile/phase combinations stop before host or compiler work.
Linux x86-64 cross-links using LLVM 22 for IR and Zig's Clang with glibc 2.36
for the C runtime contain the required main/body call replacement. Both hot
layout functions retain identical offset-normalized decoded instructions,
and the call guard rejects an unchanged baseline supplied as a candidate.
These ELF files are not executed locally and use different tools from the
native CI cohort. Its actual ELF checks, qualifications and forty-row ordinary
result follow below; measurement qualification is not the full repository
gate. `make static`, shell syntax and patch checks pass locally.

### Sixty-first measurement: no useful tree-build control gain

Frozen revision `dc88c43342af90518f552e9e7c1e3af6df37d36d` on
`codex/io-cpu-build-control` completed
[run 34121175541](https://github.com/mbbill/Whitefoot/actions/runs/34121175541)
successfully. The Linux CPU job and Windows completion job passed; the other
six workflow jobs were correctly skipped by their existing routes. The
[CPU artifact 10018435219](https://github.com/mbbill/Whitefoot/actions/runs/34121175541/artifacts/10018435219)
has ZIP SHA-256
`e273acb6d6f6fecf625dc09db9a0699db5d5766a5ccada500aaf58da628bb2df`,
matching the API digest. This is the isolated measurement workflow, not a
claim that the complete repository gate passed.

The host was an AMD EPYC 9V74 with four allowed logical CPUs, two cores and
two SMT threads per core, running Linux 6.17.0-1022-azure. Clang 18.1.3 and
Rust 1.98.0 built this cohort. No new affinity constraint or dedicated-host
assumption was introduced. The normal completion target passed five full
native harness executions, its read/default-route/pure-compute probes,
scheduler smoke and all four enumeration configurations (17/17/20/19
schedules). Both native-ring and forced POSIX-adapter bridge checks passed.
The default configuration's optional owner-ring probe is inapplicable;
this does not assert an additional owner-ring harness pass. Rayon fmt,
both Rust tests and clippy also passed.

All nine pre-timing artifact hashes, eleven final artifact hashes and 34
source entries verify against the retained files and frozen source revision.
The compiler executable, runner, selected WF/Rayon executables, both observed
executables and both IR files are retained. Manual link arguments differ
only in IR input and output path, and the complete IR differs at exactly
one line: the scoped build-call replacement. Both manual executables print
the exact independent checksum at both batch lengths before timing. The
runner validates each ordinary/warmup output after trimming trailing CR/LF;
those outputs are not individually retained. Four observer stdout files
and four manual qualification stdout files match the exact expected bytes.

The plan and raw data contain eight warmup invocations followed by forty
ordinary rows: five passes over the four forms at each of one/sixteen
batches. Warmup runs in plan order; recorded passes start reversed and then
alternate. Warmup timings are not recorded. Wall, aggregate user-plus-system
CPU, both context-switch counts and peak RSS are recomputed from these rows.
The following candidate/manual-default ratios pair the same recorded pass;
brackets are the five-pair minimum and maximum, not confidence intervals.

| Batches | Wall ratio | Total CPU ratio | Voluntary switches | Involuntary switches | Total switches | Peak RSS ratio |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 1.010129 [0.997610, 1.016587] | 1.000504 [0.999131, 1.005711] | 0.989983 [0.943511, 1.067797] | 1.220903 [0.746631, 1.745882] | 1.085294 [0.887619, 1.309659] | 1.021600 [0.966392, 1.034682] |
| 16 | 0.997319 [0.984265, 1.001698] | 0.999921 [0.998609, 1.001052] | 0.982386 [0.967097, 1.046800] | 1.026518 [0.795757, 1.078092] | 0.985290 [0.943315, 1.042668] | 0.988800 [0.908764, 1.029695] |

Manual-default/candidate median walls are 397.175/400.472 ms at one batch
and 5762.130/5749.324 ms at sixteen. The ordinary and manual-default ELF
files are byte-identical. Their ordinary/manual paired wall ratios are
0.999434 [0.991685, 1.016073] and 0.997114 [0.994291, 1.003531], respectively;
their total CPU ratios are 0.998861 and 0.999896. This same-binary spread
limits interpretation of small differences in this five-pass cohort.

For the separate competitive comparison, ordinary WF/Rayon paired wall
ratios remain 1.124169 [1.119751, 1.137607] at one batch and 1.021157
[1.014251, 1.035833] at sixteen, with total CPU ratios 1.025910 and 1.011659.
Sequential-build WF/Rayon wall ratios are 1.128644 [1.123860, 1.147869] and
1.019900 [1.013177, 1.025388]. The control does not close the short-run gap.
These are ordinary whole-process measurements of this workload and resource
budget; observer counters do not supply a WF/Rayon performance ranking.

Actual ELF disassembly confirms the default main/body has one call to
`wf_build`; the candidate has none and calls `wf__par_seq_build`. The
uploaded hot-layout check honestly reports differences. Independent decoding
of all 264 `wf_layout` and 275 `wf_layout_banded` instructions resolves those
differences: operations, registers, branch-relative targets and thunk names
agree after address normalization. For each function, all 31 RIP-relative
constant reads resolve to identical actual 8/16-byte operands in the ELF
load segments. Twenty-nine constant addresses move by -16 bytes; two stay
fixed. Both function starts move by -400 bytes. Thus matching decoded work
and constant values does not establish byte-identical candidate code/data
placement or remove a cache/layout effect from this executable control.

All four separate observations report four scheduler threads, three started
workers, positive steals, unchanged storage settings, 256 pause/look rounds
and 16 idle-yield rounds. The counter identities matter:
`grants` is the steal count, while `inline_runs` counts a published task
popped and executed by its owner at join. It is not failed acquisition.
The [scheduler source](../../../compiler/src/backend/sched/core.c) increments
these counters on the two successful deque-pop routes. Once running on a
core lane, `wf_sched_acquire` refuses only an oversized frame or an empty
lane free list; the ABI wrapper also refuses a caller without a core lane.

| Observed form | Batches | Steals | Owner inline executions | Sum |
| --- | ---: | ---: | ---: | ---: |
| Default | 1 | 6756 | 94107 | 100863 |
| Sequential build | 1 | 6644 | 94156 | 100800 |
| Default | 16 | 85992 | 1526871 | 1612863 |
| Sequential build | 16 | 85858 | 1526942 | 1612800 |

These sums account for every planned publication: 63 per layout times
1600 layouts per batch, plus the default builder's 63. The observations
provide no evidence of executed failed-acquire fallbacks. They do not show
how much eligible work existed at an idle turn, and their relaxed snapshots
are not a simultaneous queue/scheduling timeline for ordinary samples.

Selection: retain parallel tree initialization and the existing runtime
defaults. This bounded change to initialization plus lazy-start placement
shows no useful gain here. The remaining 63-versus-15 layout fork difference
is exercised, whereas treating owner inline executions as acquisition
failures would select an unsupported explanation. Neither ready-work
availability nor an OS scheduling cause has been measured by this control.

## 62. Native large-message candidates with a wider ARM client

Experiment 60 shows that the one-client envelope can mask server differences
and reverse their apparent ranking. The next native screen holds the server
CPU fixed while comparing the existing 64 KiB uring pure-ring and inline-send
paths with 32/64/128 provided buffers. Epoll and indexed WF remain same-host
anchors. A correctness issue in the existing exhaustion recovery path must
be resolved before this screen can produce timing evidence.

### Delayed exhaustion after a buffer return

The existing `d241cf7d` receive path parks a connection on `-ENOBUFS` and
depends on a later `buffer_return` to arm another receive. The
[multishot documentation](https://man7.org/linux/man-pages/man7/io_uring_multishot.7.html)
and the [maintainer's explanation](https://github.com/axboe/liburing/discussions/1362)
establish that exhaustion terminates the multishot operation; replenishing
the provided ring does not restart it. The concrete ordering defect follows
from this reference's CQE loop, not from the API documentation alone:

1. The kernel consumes the last buffers and queues positive receive CQEs
   followed by an exhaustion CQE.
2. Earlier CQEs retire sends and return those buffers before userspace consumes
   the exhaustion CQE. Inline sends can do this while handling the positive
   receive CQEs; a different connection's send CQE can do so in the ring path.
3. At those returns, the exhausted connection is not yet in the parked list.
   Processing its later exhaustion CQE then parks it despite the completed
   replenishment. With no later return, it has no event that arms a receive.

The correction records a buffer-return generation per worker and snapshots it
at every actual receive arm. The generation advances only after the returned
buffer is published. Exhaustion with a changed generation gets one new arm,
which captures the current generation; exhaustion without another return
parks. A different connection can consume the retry's available buffers, but
cannot cause repeated idle retries without new publications. The ordinary
connection and worker representations acquire these fields; this is a visible
correctness change, not an unchanged-IR claim against `d241cf7d`.

`uring_echo_check.c`, wired into the existing `uring-check` target, drives the
actual receive-arm, queue, buffer-return and complete CQE-handling functions.
Only kernel setup/enter and synchronous send outcomes are simulated. Its
transport traces distinguish returns before and after exhaustion, no-return
parking, a retry whose buffers another connection consumes, partial sends and
terminal buffer CQEs, for pure-ring/inline and ordinary/observed builds. These
deterministic traces complement the real Linux streaming/backpressure oracle;
they do not establish native kernel behavior or timing. An observed-only
`late_exhaustion_retries` count makes the corrected path visible in subsequent
diagnostics. The earlier measurement revision stays frozen.

The regression was run locally with ASan/UBSan in all four builds. The frozen
`d241cf7d` source with only the transport hooks added fails the return-before-
exhaustion trace in each build: two buffers are available, two connections
remain parked, no sends are pending and no retry was submitted. The corrected
source passes all seven traces in each build (28 total), and rejects an unknown
trace name. Two ordinary send modes also cross-link against real Linux
AArch64 headers. These are simulated CQE executions on macOS and cross-builds,
not a native Linux streaming qualification or a new performance result. The
new fixture belongs to the existing benchmark qualification and is removed
if the native reference is retired.

### Bounded native capacity and send-policy panel

The separate corrected baseline is `4ad6c37cec63b4750d272225328a05aa5cc51a1f`.
`scheduler-native-frontier` uses the existing scheduler harness, ARM admission
and full-byte client. The default `WF_BENCH_URING_BUFFER_COUNT=0` retains the
connection-derived byte budget. Explicit counts must be powers of two no
larger than 32768; the screen selects only 32/64/128. Buffer capacity and send
policy are the only native candidate axes after the shared rearm correction.

| Forms | Receive storage per worker | Send path |
| --- | --- | --- |
| `uring-64k-p32`, `uring-64k-p32-inline` | 32 × 64 KiB = 2 MiB | Pure ring / immediate nonblocking send with ordered ring fallback |
| `uring-64k-p64`, `uring-64k-p64-inline` | 64 × 64 KiB = 4 MiB | Same pair |
| `uring-64k-p128`, `uring-64k-p128-inline` | 128 × 64 KiB = 8 MiB | Same pair |
| `epoll` | Existing 64 KiB shared scratch with bounded private spill | Existing readiness loop |
| `wf-coro-index` | Existing initialized 64 KiB private source buffer per peer | Existing sole-owner, batch-32, indexed-waiter continuation |

Every form runs 64 peers × 500 trips × 64 KiB with client widths one and two,
plus 64 peers × 2000 trips × 64 B with width one as a small-message control.
The server remains one worker on one admitted reported core; client workers
use separate admitted cores. The client remains default/service 0, one
outstanding request per peer, both latency clocks and every-byte comparison.
The experiment does not enable readiness-aware receive or SQPOLL.
Five predetermined rotation/reversal passes yield **120 ordinary rows after
24 warmups**. Three separate observer passes yield **72 rows, 96 client worker
reports and 54 native uring reports**. Two additional three-worker admitted
smokes preserve uneven 2/1/1 peer partition checks and stay outside the panel.

Admission again requires four allowed distinct package/core IDs, matching raw
physical `lscpu`/sysfs topology and sibling lists, visible quota accounting,
actual launch and worker affinity, and unchanged topology/selection at the
end. These guest reports do not establish dedicated host cores. Width two is
a same-host sensitivity control, not a server-ceiling certificate: partition
of the client service loop, kernel work, shared-VM interference and remaining
client saturation can all affect the result. No cross-architecture ranking
or extrapolation from experiment 60's single observer rows is made.

`URING_POOL_CHECK=1` extends the existing eight Linux stream configurations
with the 64 KiB pools 2/32/64/128 at both send policies and one/four workers:
24 stream cases total, each retaining the 8 MiB byte oracle and existing
partial-transfer, backpressure and EOF checks. The two-buffer cases must
actually exhaust and rearm; deterministic source traces separately force the
delayed-CQE order. Qualification failure stops timing rather than removing a
candidate. Required io_uring support, all existing client/continuation/native
qualification, and the two admitted smokes remain in the path.

The observed build adds per-buffer ownership marks and counts acquisitions,
returns, terminal-buffer CQEs, live/peak loans, receive arms, parks, ordinary
and return-since-arm rearms, closed parked entries, peak parked count, queued
aggregation and existing sends/bytes/vectors. A loan starts when userspace
consumes its buffer-select CQE; unread kernel CQEs are outside that count.
The checks require acquires = receives + terminal = returns, zero final live
loans, peak loans within capacity, parks = rearms + closed, and exhaustion =
parks + return-since-arm retries. Each worker must provide its own complete
record; a missing field cannot inherit a previous worker's value. Observer
ownership fields change the loan-node layout, so these counters describe
instrumented executions, not ordinary timings. No loan-age clocks are added.

Ordinary zero-count optimized IR must match the corrected parent before the
timed capacity candidates are built. Eighteen selected ELF files are retained and hashed:
eight ordinary servers, eight observed servers and both clients. The existing
source/tool IDs, generated IR, whole-process CPU/RSS/tails, raw rows, settings
and affinity reports remain available. Provided bytes and loan-node capacity
are allocation capacities, not actual RSS; the existing whole-process RSS
measurement is retained without a new memory harness. Ordinary paired rates,
p99 and CPU/trip determine the bounded comparison; observed exhaustion or
send counts can motivate a later control but do not by themselves establish
why a candidate wins. This frozen screen ends after its audit; it neither
autotunes against the recorded samples nor promotes a global default.

Local validation cross-links 20 strict AArch64 Linux binaries (counts
0/2/32/64/128 × pure/inline × ordinary/observed). Four ordinary optimized-IR
comparisons, at 8/64 KiB and both send modes, equal the corrected parent;
invalid counts 3 and 32769 fail compilation. The full overlay also passes the
same 28 simulated ASan/UBSan traces. Source-extracted driver checks establish
the new row/order/mask/retention counts and preserve experiment 60's prior
180/36/36/72 counts; synthetic admission, observer-record mutation and all
46 prior workflow-route checks pass. These local checks validate compilation
and harness contracts, not Linux syscalls or performance. Native stream,
exhaustion, real-host admission, measurements and full gate remain pending.

## 63. Control layout fork depth

Experiment 61 rejects the proposed initialization control as a useful gain
in its cohort and clarifies the operative task count. All 63 publications
per layout are accounted for by steals plus owner inline executions; those
inline executions are not failed acquisitions. Replacing only the two
failed-acquire recursion edges with sequential calls therefore lacks an
exercised path in that evidence. The next bounded comparison changes the
layout's actual fork depth instead.

The selected candidate specializes four parallel levels of `wf_layout` and
`wf_layout_banded`, then calls their already-emitted sequential clones.
With 64 leaves, this gives 1+2+4+8 = 15 publications per layout, matching the
join count of frozen Rayon grain four; the default WF layout makes 63.
The control keeps the default parallel 127-node builder and its lazy worker
startup, all source arithmetic/loop bounds, frame layouts and runtime units.
No compiler rule, source-language behavior, runtime ABI or shared demand
signal changes. It is an executable experiment, not a selected lowering.

`make rayon-grain-bench` selects `RESOURCE_CONTROLS=6`. Its transform copies
each layout function and its matching thunk at depths one through four:
eight function copies and eight thunk copies. Copied right-child calls,
failed-acquire left-child calls and published thunks all enter the next
depth; the last level uses the existing sequential clone. Only two calls
in the original `wf_main` select the depth-four entries. Every original
function, thunk and builder definition remains byte-for-byte unchanged.
The saved candidate is independently read back. Each copied body must
reverse to the complete original template with exact substitution counts,
and the appended region must contain exactly those sixteen copies. Removing
that region and reversing the two main calls must restore the entire
original IR byte-for-byte before either candidate executable is compiled.
Malformed definitions, duplicate/missing calls, pre-existing specialization
or a changed copy fail qualification.

This control tests a grain strategy, not pure deque or atomic overhead.
The existing sequential clones evaluate left before right; the parallel
body publishes the left child and directly evaluates the right child before
its join or failed-acquire fallback. The cutoff therefore changes traversal
order and locality below four levels. Copying functions can also change
inlining, code size, addresses and instruction-cache behavior. The artifact
retains actual ELF symbols/disassembly and relevant call sites for independent
callgraph and publication-site inspection; identical hot-function topology
is neither required nor claimed. WF's already-omitted unpublished node stores
and the reference's source writes remain the executed-write parity limit
described in experiment 61.

The resource budget remains WF main plus three workers, twelve WF stacks,
and Rayon four workers with its sleeping caller. One/sixteen batches each
execute all 800 full layouts followed by all 800 banded layouts. Ordinary
compiler WF, same-IR manual default, manual depth-four candidate and Rayon
form eight entries. One complete warmup precedes five alternating passes,
giving forty ordinary wall, user/system CPU, voluntary/involuntary switch
and peak-RSS rows. Ordinary/manual binary identity is checked and retained;
only a byte-identical pair supports a same-binary variation statement.
The experiment keeps default 256 pause/look and 16 idle-yield rounds, uses
no phase profiler or new affinity, and does not tune against these samples.

The normal completion suite runs once because the runtime is unchanged.
Both manual programs must print the independent checksum at both batch
lengths before timing. Four separate observed executions check exact bytes,
four scheduler threads, three started workers, positive steals and unchanged
runtime settings. `grants` must equal `steals`; adding `inline_runs` must
give 100863/1612863 for the default and 24063/384063 for the candidate at
one/sixteen batches. These are 1600 layouts per batch times 63 or 15, plus
the unchanged builder's 63. An unchanged candidate cannot pass that check.
The observations qualify executed publications without assigning their
counters to ordinary timing rows or inferring globally available work.

The `codex/io-cpu-grain-control` CI route reuses the resource job and retains
the compiler, runner, selected ordinary and observed executables, both IR
files, exact transform/diff, source hashes, commands, plans, qualification
outputs and raw samples. Every earlier resource mode and workflow branch
keeps its existing route. Evaluate within-pass candidate/manual wall and
total CPU alongside same-binary variation, context switches, RSS and the
ordinary Rayon comparison. Fewer publications may reduce useful exposed
parallel work and worsen wall or utilization; a loss is a valid result.
Any gain would support this combined grain/traversal/specialization strategy
on this workload, not a claim about ready-work availability or OS delay.
Local qualification and native Linux results are recorded separately; no
native performance conclusion is available yet.

Local M1 qualification passes the normal completion suite (four full
harness executions), all four enumeration configurations, Rayon fmt/tests/
clippy, both manual batch-length checks, eight one-pass/no-warmup ordinary
rows and all four observed publication totals. Nine ordinary, eleven final
artifact and 34 source hashes verify; manual links differ only in IR input
and output path. Ordinary/manual Mach-O files differ, while their complete
disassemblies match apart from the file heading. These shared-host smoke
timings provide no performance ranking.

The actual transform rejects eight malformed original inputs and five
candidate mutations. Bash 3.2 checks preserve all six previous runner/manual
compiler modes; 46 previous workflow admissions, artifact names and target
commands remain unchanged. Invalid mode/profile/phase combinations stop
before host or compiler work. Linux x86-64 cross-links pass the actual ELF
call guard, which also rejects an unchanged baseline as the candidate.
LLVM 22 inlines the depth-four entries into the ABI body while retaining
the lower copied levels, thunks and sequential child calls. This is inspected
cross-code evidence, not native Linux execution or identical code topology;
the native CI toolchain and ordinary result remain to be audited. Shell
syntax, `make static` and patch checks pass locally.
