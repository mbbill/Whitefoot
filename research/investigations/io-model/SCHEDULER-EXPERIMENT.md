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
