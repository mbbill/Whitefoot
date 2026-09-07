# io-completion-bench

Program-level measurement of the unified-state completion I/O model against
hand-written native candidates and Whitefoot's own sequential build.

## What it serves

`research/investigations/io-model/RESULTS.md` held only C-level
microbenchmarks of the completion core — round-trip cost, cached `pread`
delta, park/wake parity. Those numbers cannot answer the question the design
needs to address: how a whole Whitefoot program that does real I/O compares
with strong native implementations. This bundle supplies measurements within
each recorded workload, platform and resource budget, and the evidence section
that RESULTS.md now carries. Its current candidates do not establish the
fastest possible implementation; the maintained comparison matrix records
their qualification, tuning gaps and omitted workloads.

It is removed when the completion model stops being an open performance
question — when the numbers are stable, the bar is settled, and no further
runtime or lowering change is being measured against them.

## The three lines

Every line publishes the same bytes; a line that publishes anything else
cannot report a time.

- **N** — hand-written native C candidates. `baseline.c`: a single-threaded
  blocking loop, a pthread pool over a striped index range, and on Linux a raw
  `io_uring` read pipeline (`uring_baseline.h`, kernel ABI directly, no
  liburing). Compiled `-O2`; implementation and tuning limits remain part of
  the recorded evidence.
- **S** — the Whitefoot program built with `whitefootc --no-overlap`, which
  emits the module a compiler with no overlap lowering at all emits. Every I/O
  call is an ordinary direct call.
- **C** — the same Whitefoot source built the way it ships. The C and S lines
  are one source compiled two ways, so the pair is a statement about the
  lowering rather than about two programs.

## Workloads

### The many-files workload

`programs/many_files_wide.wf` opens and reads four independent generated files
per round, four opens and then four positioned reads written consecutively so
the lowering can overlap them. `programs/many_files_wide8.wf` is the same
shape hand-widened to eight, so the comparison against an eight-thread pool
and a deep io_uring baseline is made at a matched width.
`programs/many_files_narrow.wf` is the same work written as the natural
one-file-at-a-time loop, with its name and destination buffers hoisted above
the loop; it exists to measure what a writer gets who does not hand-widen, and
the answer is no overlap at all.
`programs/many_files_loop.wf` is that same one-file-at-a-time loop with the
name and destination buffers constructed inside the body, which is the form
[PAR-3]'s staged permission grants — `whitefootc --par-ledger` prints a granted
`PAR stage` verdict for its `@scan` loop and a denial naming `&'n name` for the
narrow program's. Its helper functions are byte-identical to the other
programs', so the pair isolates exactly the hoisting. Until the staged lowering
lands it runs sequentially and pays a per-iteration allocation, and it must
publish the same checksum as every other line.
`programs/pipe_relay.wf` pushes two independent byte streams at two
independent consumers through `command.stdout` and `command.stderr`.

The generated tree and the checksum are defined once, in `workload.h`, and
shared by the generator, the native baselines, and the Whitefoot programs. The
checksum is position-weighted so a four-wide lane split and a one-at-a-time
loop fold to the same value.

### The read-heavy workload

`programs/read_heavy_narrow.wf` and `programs/read_heavy_wide8.wf`, with their
`_4k` variants, answer the question the many-files workload cannot. That
workload opens a file per unit of work, and on the macOS host one `openat`
costs 116 us against a 1.9 us `pread`, so its table is mostly a measurement of
the host's endpoint-security stack. These four programs open eight 64 MiB
files once, before any read, and then perform tens of thousands of positioned
reads into them: 32,768 reads of 64 KiB, or the same number of 4 KiB, so
what the time is made of is reads. Both windows do the same number of reads,
because an uncached read on this host costs about the same at either size --
it is a device round trip, not a bandwidth question -- so equal read counts
keep the two tables comparable and roughly equal in wall time.

`read_heavy_narrow.wf` is the natural loop: one read per iteration into one
destination buffer, eight such loops so that each stays on one file.
`read_heavy_wide8.wf` states eight reads consecutively per round into eight
buffers, which is the shape the lowering can overlap. Read *k* of a run takes
its file and its window-aligned offset from *k* alone, so the narrow program,
the eight-wide one, and every native baseline traverse exactly the same list
and fold exactly the same value.

The 4 KiB eight-wide source is also the Windows qualification workload. Its
eight fixed file names arrive as command arguments and are copied through
`host_copy_bytes` before `open_file`: ten bytes on a one-byte target and twenty
UTF-16LE bytes on Windows. The direct and completion builds therefore receive
the same target-native component ranges without either a source fallback or a
runtime transcode.

The opens are inside the timed region — the runner times whole processes — but
there are exactly eight of them in every line, N, S, and C alike. At 116 us
each that is 0.93 ms against a table whose fastest line is over a hundred
milliseconds, and a constant every line pays identically cannot move a ratio.
This is what "open-once" buys: the open cost stops scaling with the work.

Each line folds the first sixty-fourth of every window rather than all of it,
and publishes the full transferred byte count beside the checksum. The reason
is in `workload.h`: the digest is a serial multiply-add chain running at about
800 MB/s, so folding a whole 64 KiB window costs about 80 us of CPU against a
134 us uncached read and a 7 us warm one. Folding everything would make the
warm table pure compute, and would add to the uncached table CPU that the
eight-wide program can spread across helpers and the sequential one cannot.

### The four-stage chain

`chain.c` is not a Whitefoot program and is not one of the three lines above.
It is the C program design §12's fourth item asks for: the
`read -> parse -> request -> write` chain of `PARK-ON-MISS.md` §0, on raw
io_uring, in the four shapes that item names — nested helping, thread
compensation, the stack switch, and the staged pipeline as it is lowered today
(one lane, K slots, the loop blocking on the oldest slot's join). It reports
the dependent stage's in-flight depth beside the wall time, because the claim
§0 makes is about depth and not about speed.

It lives here rather than in a home of its own because the ring plumbing,
the generated tree and the file-name format are this bundle's
(`uring_baseline.h`, `gen.c`, `workload.h`), and a second copy of them would be
a second thing to keep true. The shape it compares is driven from
`research/experiments/park-on-miss-measurements/run.sh`, which is where its
numbers are recorded.

    make -C research/experiments/io-completion-bench chain

One thing is deliberately the same in all four shapes: the ring is driven by
one reaper thread, so what the four numbers compare is what a worker does when
it joins an operation that has not completed, and nothing else. The stack
switch shape links `compiler/src/backend/sched/core.c` and drives it, so that
shape is the shipped scheduler rather than a model of it — with the one
difference its own numbers have to be read against, that the park it sleeps on
is `prim_host.c`'s fallback epoch condition variable and not the bridge's ring
park, because the bridge is not linked here.

Every file is opened once, before the timed region, for the reason the
read-heavy workload opens once: an `openat` of a cold inode costs more here
than the read that follows it. The descriptors are opened `O_DIRECT` where the
filesystem allows it, and the printed line says which it was, because on a
buffered tree a read does not wait and no shape can reach any depth.

### WF_IO_NOCACHE

`WF_IO_NOCACHE=1` is a target-policy knob of the same class as `WF_IO_HELPERS`
and `WF_WORKERS`: it is never a language surface, no Whitefoot source names
it, and it changes no byte any program publishes — the `read-verify` target
checks every line's bytes with it off and on. What it changes is where a
read's bytes come from. On Darwin the runtime applies `fcntl(fd, F_NOCACHE, 1)`
to each descriptor an open hands back, which is a mode of the descriptor: every
read through it bypasses the unified buffer cache for the life of the open. On
Linux it applies one `posix_fadvise(fd, 0, 0, POSIX_FADV_DONTNEED)`, which
evicts what is cached at the moment of the open. `O_DIRECT` is deliberately not
used, because its alignment constraints would change the program's own buffers
and so change what is being measured.

The knob lives in `compiler/src/backend/completion/file_adapter.h` and is
applied by both the bounded POSIX adapter and the Linux io_uring adapter, once
per descriptor an open hands back and never to one a kind check refused.
`workload.h` mirrors it for the native baselines, so N and C wait on the same
device rather than on two different cache states.

### Proving a table is uncached

The knob alone does not make a table uncached. `F_NOCACHE` stops a read
populating the page cache; it does not evict a page that is already resident,
so a table run over a tree that was just written, or just read by a warm
table, is served from memory however loudly it asks for the device. Batch
an early table was published that way once; the correction
records it.

Three things now stand between that mistake and the table.

`make read-uncache` regenerates the tree so nothing is resident: the generator
writes each file through a descriptor that does not populate the cache,
flushes it, and on Linux drops its pages. That such a descriptor really keeps
its traffic out of the cache is measured rather than assumed — three passes
over the same eight blocks of a freshly generated file cost 248, 230 and
315 us on this host, with no drift towards the 7 us a resident page would
cost — which is what lets the probe below observe residency without creating
it.

`make read-warm` is the other half: it reads every block of every file back in
through plain descriptors, so a warm table has the state it claims. A full
sequential pass rather than a rerun of the workload, because 32,768
pseudo-random reads would leave about two per cent of the blocks untouched.

`make read-settle` then waits the host out. Writing half a gigabyte does not
only put bytes on the device: something on this machine reads the new files
back. A probe run every fifteen seconds on an otherwise idle machine watches
residency walk from the first file towards the last over a minute or two and
then vanish as the pages age out, with no benchmark line running and every
line reading through the non-populating policy; `XprotectService` holds about
a tenth of a core throughout. So a fresh tree is left alone until the same
probe the table is gated on passes quietly, which is a wait for an outside
reader to finish rather than a softer threshold. The gate probe below is
unchanged and still refuses the table.

`read_baseline probe-uncached` then checks the claim rather than trusting it.
It times sixteen positioned reads in each of the eight files, through
descriptors that do not populate the cache and at offsets that differ on every
invocation, and refuses the label unless all but ten per cent of those reads
cost more than 40 us — the gap between a 6-to-20 us cache hit and a 134 us
device read on this host. It runs immediately before and immediately after
every table, which is what catches both the tree being resident when a table
starts and something making it resident while the table runs. `probe-warm` is
the same check in the other direction, so the warm tables are labelled by
measurement too.

### The TCP echo workload

The target in `research/investigations/io-model/NETWORK.md` section 6 is to
compete with the strongest implementations regardless of language. The native
programs here are measured candidates; their presence alone does not establish
that they are the fastest existing solutions. The maintained
[baseline matrix](../../investigations/io-model/SCHEDULER-EXPERIMENT.md#baseline-matrix-and-evidence-levels)
records what each comparison can establish and what remains unmeasured.

Three servers, one contract, one load generator. The contract is what makes
the three comparable and what keeps the generator from telling them apart:
a server takes `PORT` and `CONNECTIONS`, listens on `127.0.0.1:PORT`, echoes
every byte of every connection back to that connection, and exits zero once
`CONNECTIONS` connections have been accepted in total and every one of them
has closed. A server that echoes the wrong bytes, drops a byte, reorders one,
or leaves a connection unserved fails the run instead of reporting a fast
time.

- **`uring_echo`**: the io_uring reference, and the number every other line is
  a ratio to. Written against the kernel ABI directly rather than liburing, as
  the read baselines are.
- **`epoll_echo`**: the second reference, the shape most deployed servers
  still have: one epoll instance and one `SO_REUSEPORT` listener per thread,
  edge triggered, reading until `EAGAIN` and carrying a per-connection buffer
  for what a short write leaves behind.
- **`wf_echo`**: `programs/tcp_echo_server.wf` built with `--par`, the
  Whitefoot line. It runs with `WF_STACKS=1100` and otherwise the shipped
  defaults, because a parked callee holds a pool stack for as long as its
  connection lives and the widest case here holds 1024 connections at once.

What the io_uring reference does that a portable server cannot, which is what
the ratio is against:

- **multishot accept.** One `IORING_OP_ACCEPT` with `IORING_ACCEPT_MULTISHOT`
  per listener yields a completion per connection. There is no accept call and
  no submission per connection at all.
- **multishot receive.** One `IORING_OP_RECV` with `IORING_RECV_MULTISHOT` per
  connection yields a completion per arrival, so a server that is echoing does
  not submit a read between one message and the next.
- **a provided buffer ring.** `IORING_REGISTER_PBUF_RING` hands the kernel a
  ring of buffers and lets it choose the destination when the bytes arrive,
  rather than committing a buffer per connection before there is anything to
  put in it. The echo is then sent straight out of the buffer the kernel
  filled, avoiding an extra userspace copy; ordinary kernel socket copies
  still exist. Exhaustion
  is real and is handled rather than avoided: a receive that finds no buffer
  answers `-ENOBUFS`, and that connection waits for a buffer to come back
  instead of spinning on a re-arm.
- **one ring per core.** Each thread owns its ring, its buffer ring and its own
  `SO_REUSEPORT` listening socket, so a connection is accepted, received and
  echoed on one thread with nothing shared on the path.
- **a ring the kernel need not interrupt.** `IORING_SETUP_SINGLE_ISSUER` with
  `IORING_SETUP_DEFER_TASKRUN` says one thread submits and the same thread
  waits, which lets the kernel defer completion work to the moment that thread
  asks for it. The relative gain depends on the kernel, workload and resource
  budget; a previous host's result does not select the best configuration for
  a new panel.

`--sqpoll` adds `IORING_SETUP_SQPOLL` and is off by default. The development
host admits it; whether another host does is what `uring_echo --sqpoll` says
there, since a kernel that refuses the flag refuses it at `io_uring_setup` and
the server reports that and exits. The current paired protocol does not run
it: the extra kernel poll threads need explicit CPU placement and whole-system
accounting before they fit the same resource budget. Process CPU from
`/usr/bin/time` does not account for them. This configuration uses SQPOLL
instead of the deferred task-work pair.

Everything each server needs is sized from `CONNECTIONS` before the first
accept: the connection tables are indexed by descriptor, the buffer rings and
echo queues are fixed arrays, and no server allocates per operation.

`netload` is the one generator all three are measured with:

    netload PORT CONNECTIONS ROUNDTRIPS BYTES [--threads T]

It opens `CONNECTIONS` connections spread over `T` client threads, each with
its own epoll, and times two phases. The connect phase runs from the first
connect call until every connection is established, which is the
connections-per-second measure. The exchange phase then has every connection
perform `ROUNDTRIPS` round trips of a `BYTES`-byte message with all
connections active at once, which is the round-trips-per-second, the
bytes-per-second and the latency-distribution measure. Every echoed byte is
compared with the byte that was sent, and a refused connect, a peer that
closes mid-exchange, or one wrong byte prints one line and exits nonzero. The
whole result is one line of tab-separated `key=value` fields, and the latency
samples live in one array of `CONNECTIONS*ROUNDTRIPS` 32-bit microsecond
values allocated before the first connect.

The echo pattern varies by peer, byte position and round modulo 256. Every
returned byte is checked, but identical-pattern peer swaps, old rounds and
reordered 256-byte blocks can escape this oracle; it is not a unique message
identifier. The diagnostic work preserves these exact historical bytes.

`make scheduler-client-diagnostic` runs experiment 56: an optional
`WF_NETLOAD_OBSERVE=1` client records send/receive outcomes, transfer-size
histograms and exchange epoll event/batch counts. The ordinary client has no
observer fields or branches; its optimized IR must match the fixed baseline.
Two native servers and two 64-peer echo cases produce twelve unprofiled
observed rows plus four separate client stack captures. Reporting and byte/
round conservation checks happen after exchange timing and thread joins.
`make netload-observe-check` checks synthetic operation traces on POSIX;
Linux `client-observer-check` additionally runs the actual client with two
workers and service budgets 0/1/8. These are different evidence scopes.
The counters and their retirement condition are documented in
[`SCHEDULER-EXPERIMENT.md`](../../investigations/io-model/SCHEDULER-EXPERIMENT.md#fifty-sixth-experiment-observe-the-client-syscall-and-readiness-work).

`make scheduler-client-readiness` runs experiment 58's ordinary default versus
`WF_NETLOAD_READINESS=1` client pairs, with four fixed servers and five echo
cases. Five alternating passes produce 200 ordinary rows; forty separate
client observations test whether empty receive probes decrease. The optional
client preserves delivered read edges through partial I/O and retains terminal
events across frames. Linux `make client-readiness-check` runs strict traces
through the actual exchange body, rejects a deliberately lost edge, and checks
real fragmented/backpressured streams, admission, scheduled compute and exact
error reasons with service budgets 0/1/8. Default optimized IR must remain
identical. Selected executables, hashes and every raw sample are retained;
the evidence boundaries are in
[experiment 58](../../investigations/io-model/SCHEDULER-EXPERIMENT.md#58-remove-speculative-client-receive-probes).

`linux-net-bench.sh` is the protocol, and one protocol for every host that can
run it, as `read-bench.sh` is for the read tables. It builds the compiler and
the three tools from one worktree, checks that every server echoes what the
generator sent at four connections before any of them reports a time, and then
runs the plan: four cases -- one connection at 20000 round trips, 64 at 2000,
1024 at 200, all of 64-byte messages, and then 64 connections at 200 round
trips of 65536 bytes for the bytes-per-second line -- across every server
line. A run picks a port below the kernel's ephemeral range, starts the server
for exactly that case's connection count, waits for the listening socket to
exist, runs the generator, and requires the server's own exit status to be
zero. Nothing here is decided by a timeout: the only waits are for a port to
appear and for a child to exit, and the wait for the port ends early if the
server process is gone, with the server's diagnostic channel printed.

`WARMUP` unrecorded passes are followed by `ROUNDS` recorded ones, a pass
being every line of every case once, with alternate passes in reverse order,
for the reason `runner.c` states: a host drifts over the minutes a table
takes, and a grouped schedule turns that drift into a difference between
lines. The table reports the median of each measure over the recorded passes
and the ratio of each line's round-trip rate to the io_uring reference and to
the epoll one. `NET_LINES` names a subset of `uring epoll wf` when one server
cannot complete a run yet and the others still owe a table; the table names
the lines it holds.

## Reproducing

    make -C research/experiments/io-completion-bench verify       # bytes only
    make -C research/experiments/io-completion-bench bench        # macOS table
    make -C research/experiments/io-completion-bench bench-pipe
    make -C research/experiments/io-completion-bench linux        # Linux table

    make -C research/experiments/io-completion-bench read-verify  # bytes only
    make -C research/experiments/io-completion-bench bench-read   # macOS tables
    make -C research/experiments/io-completion-bench linux-read   # Linux tables

    make -C research/experiments/io-completion-bench net-tools    # the C tools
    make -C research/experiments/io-completion-bench net-verify   # bytes only
    make -C research/experiments/io-completion-bench linux-net    # the TCP table

The TCP targets are Linux-only, as `linux` and `linux-read` are: `epoll_echo`
and `uring_echo` are written against Linux interfaces to compare tuned
kernel-specific I/O paths.

The current scheduler experiments and their controls are described in
[`SCHEDULER-EXPERIMENT.md`](../../investigations/io-model/SCHEDULER-EXPERIMENT.md).
`scheduler-native-baselines` screens pure-ring and immediate-send io_uring at
8/64 KiB with an equal provided-byte budget, alongside the existing epoll,
stackful, C++ coroutine and WF controls. `uring-check` qualifies every native
uring configuration against the shared 2 MiB stream oracle before timing.
`scheduler-client-headroom` holds one server worker/CPU fixed and compares
one client hardware thread with both SMT siblings of a separate physical
core. It retains full byte verification and all qualification checks while
screening six server forms at 64 peers and 64 B/64 KiB. This isolates a
client resource limit; it does not compare servers using different clients.
`go-check` qualifies the external Go sequential `net` reference with exactly
Go 1.27.1 (`GO=/path/to/go` selects the binary), `jq` and the host C compiler.
It is reached by canonical `make check` through `scheduler-experiment`; the
gate installs that exact Go version only for its scheduler jobs. Release and
race builds independently qualify both `WF_BENCH_GO_BUFFER_OWNER=handler`
(default: handler-created stack buffer) and `acceptor` (acceptor-created heap
buffer transferred to the handler). They share the same sequential read/write
loop. Escape diagnostics and actual disassembly are retained; all lifecycle
checks finish before the live memory snapshots. Both forms run the full
existing 2 MiB byte-stream/half-close/slow
reader oracle, a reset that must close three other waiting handlers, and 64
live peers using private initialized 64 KiB buffers. Runtime/GC/socket
readbacks and optional Linux per-thread, epoll-fdinfo and memory snapshots are
qualification evidence. Experiment49's dedicated CI pins the whole server to
one logical CPU, including GC/runtime threads, and uses another physical core
for the client; `GOMAXPROCS=4` is deliberate oversubscription qualification.
No Go timing rank or four-CPU performance claim exists yet. The earlier
[experiment49](../../investigations/io-model/SCHEDULER-EXPERIMENT.md#forty-ninth-experiment-qualify-a-sequential-go-net-reference)
qualification remains frozen; [experiment51](../../investigations/io-model/SCHEDULER-EXPERIMENT.md#fifty-first-experiment-compare-go-buffer-ownership)
compares the two ordinary storage forms before selecting timing candidates.

`scheduler-go-screen` adds all four qualified release configurations
(handler/acceptor buffer ownership × GOMAXPROCS 1/4) to experiment48's fixed
five WF and seven native echo controls. It pins Go 1.27.1, runs all 32 Go
release/race qualification cases first, and copies that exact release binary
into the panel. Five cases and seven alternating passes after two warmups
produce 560 ordinary timing rows. Three resident cases repeated three times
produce 144 snapshots, separate from the eight Go qualification snapshots.
Every server thread shares one logical CPU; the client uses a different
physical core. Go reports and send-buffer overrides are disabled in ordinary
samples, GOGC is 100, and runtime limits stay at their defaults. Observations
retain the storage/P configuration, live thread affinity and actual epoll
registrations. Source, tool and binary hashes accompany the raw samples.
[Experiment53](../../investigations/io-model/SCHEDULER-EXPERIMENT.md#fifty-third-experiment-sequential-go-in-the-fixed-echo-screen)
owns this bounded comparison; no fresh tuning or server-capacity claim is
inferred from the unchanged loopback/client envelope.

`scheduler-uring-diagnostic` captures three observed repetitions of the
64-peer × 64 KiB cell for the four 8/64 KiB pure/inline uring forms, using one
server CPU and the unchanged client on another physical core. Its counters
distinguish requested bytes/iovecs from completed transfers; instrumented
metadata is kept outside the normal timing tables.
`scheduler-checkpoint`, `scheduler-footprint`, `scheduler-paced`,
`scheduler-chunks`, `scheduler-canonical`, `scheduler-stackful` and
`scheduler-stackful-paced`, `scheduler-nodelay`, `scheduler-owner` and
`scheduler-owner-paced`, `scheduler-memory`, `scheduler-dispatch` and
`scheduler-dispatch-paced`, `scheduler-wake` and `scheduler-wake-paced` run
their respective Linux cohorts. The measured
priority, counter-stripe, stack-offset and independently locked ready-queue
prototypes were retired; their results and exact revisions remain in the investigation. The canonical
comparison also builds the recorded prior compiler revision from local Git
history; use a checkout containing it.
The memory cohort crosses compact stack metadata/first-use contexts with
used-lane initialization, keeps 1,100 reserved stacks in every form, and
measures echo plus compute/file controls. The source, I/O protocol, payload
initialization, and TCP_NODELAY policy are identical across its WF forms.
The paced client requires Linux 5.11 or newer and
glibc with `epoll_pwait2` (2.35 or newer); it measures scheduled arrival
latency including client backlog, with heavy peers remaining closed-loop.
The stackful comparison keeps the native epoll engine and connection ownership,
replacing manual continuation fields with sequential functions on guarded
stacks. `stackful-check` verifies back pressure, fragmented compute frames and
premature close for both representations; Linux `scheduler-check` also runs it.
The comparison's small-send-buffer override and observer are correctness-only.
The TCP policy comparison measures WF, native epoll and native io_uring with
`TCP_NODELAY` off/on, including both small and large payloads. It reads back
accepted socket options outside timing and runs the full candidate completion
suite. The WF default remains unchanged pending this measurement.
The owner comparison separates pinned ready continuations from per-worker
Linux rings, then combines them. Its paced form retains the same emitted
16384-step chunk module and fixed light arrivals. Every candidate runs the
full completion suite before timing; native probes also check the lifetime
of two independent ring wake descriptors sharing one logical wake epoch.
The dispatch comparison assigns initial staged I/O calls round-robin before
pinning continuations. It checks actual per-worker starts separately from
steals, retains pure-compute stealing, and compares identical WF chunk modules
under different placement policies. Equal connection counts do not guarantee
equal CPU demand; the fixed-arrival cohort measures heavy deadline capacity
and light tails together.
The wake comparison keeps that initial placement and suppresses a ready-queue
wake only when the target owner is the current executing core thread. External
helpers, other workers, in-place waiters and process exit retain their wake
paths. The same candidate runs the full completion suite before timing.

The generated-WF continuation screen uses
`CONTINUATION_SCREEN=1 NATIVE_BASELINES=1 EXPERIMENT=allocator bash scheduler-bench.sh combine`.
It compares the actual sequential echo source compiled with
`--continuations --par` against two stackful WF forms and seven native
controls, with one server CPU and a client on another physical core.
`compiler-continuation-bench` builds the candidate without sanitizers and checks
its four-slot, twelve-task protocol; the Linux harness additionally requires
the native/helper ASan/UBSan suite and the common large-stream oracle before
timing. A 1024-task window admits all peers held open by the fixed benchmark
protocol. The same counter-enabled binary supplies observed and timed runs,
with reports disabled during timing. Seven passes over five cases produce
350 rows and three repetitions of the three residency cases produce 90
snapshots. Experiment 39 in
`research/investigations/io-model/SCHEDULER-EXPERIMENT.md` owns the results
and limitations of this temporary coordinator; retire this screen when the
next executor comparison supersedes it.

`CONTINUATION_SCREEN=2` adds `wf-coro-owner` to that same panel: the identical
generated binary runs with `WF_CONTINUATION_OWNER_PROGRESS=1`, driving target
progress on its sole resumer instead of a background thread. Both modes keep
the existing locked publication protocol and pass the complete generated
continuation suite. The eleven-form panel produces 385 timing rows and 99
live snapshots. Experiment 44 owns this isolated handoff comparison; no
source or completion-record ABI change is involved.

`CONTINUATION_SCREEN=3` additionally keeps `wf-coro-batch32`: the same owner
binary with `WF_CONTINUATION_PROGRESS_BATCH=32` instead of the default one.
It resumes at most that many already-ready waiters between explicit target
progress calls, and always progresses immediately when the ready queue is
empty. This gives 420 timing rows and 108 live snapshots. All three policies
run the generated lifetime/stream suite; separate observations enable
`WF_CONTINUATION_REPORT_BRIDGE=1` to retain existing ring submission, enter,
completion and wake counters. Experiment 48 owns this batching comparison.
Its audited Linux panel at `fc69af15` improves paired 64/1024-peer small-message
rate by 28%/27% over owner-one and confirms aggregated submissions in separate
reports. One peer has no rate gain; native controls still lead the occupied
small-message cells. Client capacity limits several cells, and no new default
or universal throughput result is selected.

`make scheduler-continuation-index` selects `CONTINUATION_SCREEN=4` with the
same native panel and adds `wf-coro-index`: owner progress, batch 32 and 1024
pending-waiter buckets versus the default single list. One normal binary
contains both choices. The 8 KiB bucket array has collision lists under the
existing lock; it adds no waiter-capacity bound or frame/record ABI field.
Thirteen forms produce 455 rows and 117 live snapshots at seven passes.
An additional host build enables lookup counters only for untimed observations,
including 64/1024-peer comparisons. Both binaries pass the common stream and
bounded-retirement oracle. Sanitizer qualification includes 1/1024 buckets
under all progress modes; the C++ fixture additionally uses two buckets with
eight simultaneously pending roots to force collisions. Experiment 54 owns
this candidate and its pending Linux result.

The generated continuation qualification also compiles the unchanged
`tcp_compute_server.wf` with `--continuations --par`. Experiment 52 reuses
the common fragmented compute oracle, then checks truncated and oversized
requests with the source's exact exit codes 9 and 10. Error-mode callers may
provide `stream_check SERVER MODE WORKERS ERROR_STATUS`; omitting the status
keeps the existing native/Go default of one. Invalid requests must close
without response bytes. All three progress policies check native/helper
routes, complete task retirement and sanitizer results. Pure computation
still runs on the sole resumer; this is protocol qualification, not a mixed
performance, CPU-offload or fairness result.
The frozen `6a8c19c1` Linux artifact passes all eighteen native/helper mixed
invocations and the previous generated/loan suites; M1 ASan/UBSan and TSan
each pass the nine local-route invocations. Experiment 52 records the exact
artifact identity and independent audit.

On native Windows, `windows-bench.ps1` owns a separate production
qualification:

    pwsh research/experiments/io-completion-bench/windows-bench.ps1 \
      -Root $PWD -Out $env:TEMP/whitefoot-windows-bench \
      -Rounds 15 -Warmup 2 -Enforce

It builds every contender from one compiler revision, generates the same eight
64 MiB deterministic files as the other read-heavy protocols, warms them with
a complete sequential pass, and pins every sampled child to one recorded
processor mask. `windows_runner.c` takes each sample with
`QueryPerformanceCounter` and `GetProcessTimes`; a nonzero exit, a byte of
stderr, or stdout different from the committed exact oracle invalidates the
sample before its time is reported. A sampled child or the untimed observer
that exceeds two minutes is terminated and invalidates the run.

Before any timed cohort, one sequential and one IOCP 4 KiB read-heavy sample
must both publish the exact oracle. This keeps a target-native path or fixture
failure from being discovered only after the compute cohort has completed.

The five alternating paired cohorts are compute (`par_layout.wf`, default
against `--par`), warm 4 KiB reads (`--no-overlap` against production IOCP),
the mixed program's sequential/IOCP control, and its IOCP-only/full compute
plus IOCP pair, followed by a direct sequential/full pair that prevents the
two component improvements from hiding a net mixed regression. The exact
mixed window is source-level
`read_at, compute_pair, read_at`; `compute_pair` contains the independent
`churn, churn` pair. Its fixed tree oracle is `17574306422404092952\n`.

Before timing, the script checks that the mixed contender is the thing it
claims to be, in two places rather than one.

The first is the emitted module. Every I/O operation has one lowering now,
submit and then join, so the window's overlap is visible in the module itself:
`@wf_main` submits the first read, calls `compute_pair` on this thread, runs
the source-last read through the always-inlined wrapper that submits and joins
in place, and only then joins the first read. The group hands none of its own
members to a compute lane, because its join site is itself a submitting member
and the emitter keeps the pure completion lowering for such a group; the
compute hand-out this cohort measures is one level down, in `compute_pair`,
whose `churn, churn` group acquires a lane, publishes into it, joins it, and
releases it once per iteration. The script pins both orders. They belong to the
emitter and not to the target, so the same shape reads out of a Linux
`--emit-llvm` of the same program.

The second is one observed link, the shipped runtime plus `grant_observer.c`,
which is `io-hosts.yml`'s `completion-windows` worker step applied to this
program. Correct bytes alone would also be produced by a pool that granted no
lane and by a run that never reached the completion port, so that link requires
all three: the program's exact oracle on the output channel, exactly one line
on the diagnostic channel, `grants=` and a positive count, and exit zero under
`WF_REQUIRE_WINDOWS_IOCP=1`, which is the runtime's own exit assertion that the
port carried at least one submission and reaped every submission it made. The
retired Windows probes counted worker starts, worker executions,
compute publications outstanding across the first read, IOCP inline and
dequeued completions, and accepted/published/consumed operations; the second
copy of the runtime they instrumented is gone, the core's steal count and the
required-ring assertion carry the verdict, and the publication-during-flight
property is pinned in the emitted order above instead of counted once per run.

The compute pair gives both builds three inert command arguments.
`par_layout.wf` counts the complete invocation vector, including the invoked
name, and therefore runs four identical batches in one process. Every batch
resets both fold seeds, so the exact 34-byte oracle is unchanged. This makes
the shorter parallel side about 1.4 seconds on the qualifying host and measures
one initialized pool's steady-state work instead of trying to stabilize a
sub-second process by weakening the spread bound or averaging repeated pool
startups. Ordinary argument-free corpus runs still execute one batch.

Each cohort records fifteen candidate/reference ratios, alternating order in
each pair, after two unrecorded warm-up pairs. A cohort with ratio MAD above
5% or p10-to-p90 width above 10% is repeated once and fails as an invalid
measurement environment if still unstable. The production bounds are compute
at most 0.90, warm IOCP at most 1.10, and full mixed at most 0.95 relative to
both its IOCP-only control and the fully sequential program. These are
same-host runtime qualifications. The host,
Windows build, CPU, processor mask, memory, power scheme, toolchain, revision,
and every raw sample ship with the table; a hosted VM is not treated as a
persistent cross-revision hardware baseline.

`linux` builds `linux.Dockerfile` and runs the whole pipeline inside one
container, because the generated tree must sit on a container-local
filesystem: measuring a bind mount would measure the host's file sharing
rather than the kernel's I/O path. It passes
`--security-opt seccomp=unconfined`, without which `io_uring_setup` is refused
and both the native baseline and Whitefoot's own Linux adapter silently fall
back.

`linux-read` runs the read-heavy tables in the same container for the same
reasons, and adds the `io_uring` baselines at depths 4, 8, and 32.

It does that by running `read-bench.sh`, which is one protocol for every host
that can run it: the container, the project's Linux runner, and its macOS
runner. Only paths and the host's own capabilities differ -- `ROOT`, `OUT`,
`CLANG` and `CARGO_TARGET_DIR` name the paths, and `uname -s` decides whether
the io_uring lines are in the plan. The `io-bench` workflow's
`bench-linux-read` and `bench-macos-read` jobs run exactly those bytes; that
workflow runs on demand and when the runtime or this bundle changes. Those
Linux and macOS tables judge nothing; the Windows paired protocol above is the
dedicated qualified gate.

The script differs from `bench-read` in one deliberate way. `bench-read`
refuses to print a table whose cache-state label the probe did not confirm,
which is the right rule on a machine whose two populations are known and far
apart. A hosted runner's storage is not known in advance, and its "device" may
be a host-cached network disk that answers faster than the threshold; there
the honest result is a table labelled by what was measured rather than no
table at all. So the script always runs the probe, always prints its per-file
medians and its verdict, and prints that verdict on the table's own label
line.

The two probes around a table are not worth the same on both hosts, and the
script says so where it runs them. Before the table each host is making the
same claim and the probe checks it. After it, only Darwin is: `F_NOCACHE` is a
mode of the descriptor, so a Darwin table that asked for uncached reads cannot
have populated the cache itself, and a refusal there means something outside
the benchmark made the tree resident. Linux has no per-descriptor mode; its
one lever evicts at open and nothing later, so every Linux line starts from a
cold tree and warms it as it reads. The Linux after-probe measures how far
that went and is reported as that, not as a verdict.

`FILES`, `MAX_KIB`, `ROUNDS`, `WARMUP`, `PIPE_ROUNDS`, and `PIPE_DELAY_US`
override the many-files shape. `FILES` or `MAX_KIB` change the checksum;
`make expected` prints the new one for `EXPECTED`. `READ_FILES`, `READ_KIB`,
`READS_64K`, `READS_4K`, and `READ_ROUNDS` do the same for the read-heavy
shape, and `make expected-read` prints the two checksums they imply.

`bench-read` prints four tables: 64 KiB and 4 KiB, each with the page cache
warm and with `WF_IO_NOCACHE=1`. Both cache states publish the same bytes,
which `read-verify` checks before any line reports a time, and each table's
cache state is checked by the probe above before and after it runs.
`READ_PROBES`, `READ_THRESHOLD_US`, and `READ_TOLERANCE_PERCENT` set that
check.

The large-tree timing protocols are deliberately separate from canonical
`make check`; the maintained scheduler/native/Go correctness targets remain
in that gate. The dedicated `io-bench` workflow
owns the Windows qualification and the exploratory Linux/macOS tables.
Generated trees, binaries, and raw output stay in the selected scratch
directory; durable results retain the host identity and raw artifact beside
their summarized table.

`make rayon-check` is the small canonical correctness check for an independent
Rust/Rayon port of `tests/programs/par_layout.wf`. It checks the corpus's exact
floating-point result bits and every mutated node across pool widths and
subtree grains. `rayon-baseline/Cargo.lock` pins its dependencies separately
from the compiler. Fetch them once with `cargo fetch --locked --manifest-path
rayon-baseline/Cargo.toml`; builds and checks then run offline.

`make rayon-bench` builds the WF sequential/parallel and native Rust controls,
qualifies their output, calibrates Rayon subtree grain, freezes one candidate
per pool width, and runs a separate alternating confirmation cohort.
`RAYON_THREADS="1 2 4"`, `RAYON_GRAINS="1 4 16"`, `BATCHES=1` and
`CALIBRATION_ROUNDS=3` select a bounded initial screen; `ROUNDS` and `WARMUP`
select confirmation passes. `OUT` selects the artifact directory. Calibration
and confirmation plans, raw per-pass TSVs, summaries, pinned dependencies,
host identity and compiler/binary hashes remain there. Times cover complete
processes, including tree and pool construction; no concurrent I/O or CPU
offload transfer is measured. Experiment 40 in
`research/investigations/io-model/SCHEDULER-EXPERIMENT.md` owns its findings and
the condition for retiring this reference.

On Linux, `RAYON_PROFILE=1` appends experiment 41's CPU and scheduler captures
after the ordinary confirmation panel. `PROFILE_PERF` names the installed
perf executable. The recorder uses noninteractive sudo for tracepoint access,
then runs each workload as the original user. Four source batches use the
same binaries and frozen grains; every capture checks the exact output.
Raw CPU samples, scheduler events, per-thread reports, PIDs and commands stay
under `OUT/profile`. These observations never enter the performance TSVs.
Inspect capture loss and observer overhead before using them for attribution.

`make rayon-resource-bench` is experiment 45's fixed CPU resource control:
four computing threads, Rayon grain four, WF stack counts 12/1100, and
1/4/16 workload batches. It bypasses calibration and retains nine forms in
one alternating plan. `ROUNDS=5 WARMUP=1` produces 45 ordinary samples;
separate untimed builds use the existing scheduler observer to expose
no-target compute-join turns. No counter currently reports peak live stacks.
The experiment changes process settings only, not runtime code or defaults.

`make rayon-ring-bench` reuses that caller for experiment 47: four computing
threads, twelve WF stacks, 1/16 batches, and `WF_IO_NO_NATIVE_RING` unset/1
against the frozen Rayon grain-four control. Five passes after one warmup
produce thirty ordinary samples. Four separate Linux observations require
an initialized but unused ring in the default route and no ring report in
the disabled route. This isolates a candidate fixed output-path cost; it
does not choose a runtime default. macOS can qualify the commands and
checksums, but cannot qualify the native Linux ring distinction.

`make rayon-startup-bench` is experiment 50: four computing threads, twelve
WF stacks and 1/16 batches, retaining ordinary compiler WF output, same-IR
manual default and used-lanes links, and frozen Rayon grain four. Five passes
after one warmup produce forty ordinary samples. The used-lanes candidate
runs the full existing completion suite; separate observations check the
storage flags and bytes. `CPU_PHASE_TRACE=1` additionally invokes
`rayon-phase-trace.sh` after all ordinary timing. It preflights Linux events
and entry symbols, then records eight one-batch process timelines including
the WF sequential control and parent reap. `PROFILE_PERF` selects perf.
Missing events, failed recording or lost data leave an explicit incomplete
trace status; even captured traces require independent attribution audit.
No probe-derived timing enters the ordinary ranking and no default changes.
The isolated `codex/io-cpu-phase-recheck` CI path downloads experiment 50's
exact retained binary artifact and verifies its ZIP and individual hashes
before an eight-trace-only recapture. It neither rebuilds nor retimes the
ordinary panel. Recorder-issued write events are explicitly filtered and
checked against the saved recorder PID; global scheduler events remain.
Experiment 55's `codex/io-cpu-yield-attribution` path restores those same
hashed binaries from the smaller audited recapture artifact and sets
`CPU_YIELD_TRACE=1`. This adds `sched_yield` syscall entry/exit events to the
eight captures. Reconstruct each workload thread's syscall intervals and
their overlap with runnable off-CPU intervals before attributing the early
utilization gap to a waiting policy or placement. No source, executable,
worker budget or ordinary timing changes; tracing can perturb scheduling.
Experiment 57's `codex/io-cpu-yield-callers` path additionally sets
`CPU_YIELD_CALLER=1`, requiring yield syscall capture and an x86-64 host.
An entry probe at `wf_prim_yield` records the return address from the first
stack word. Its probe IP and retained ELF symbol/disassembly allow the audit
to remove ASLR and identify the actual preceding call instruction. The eight
captures keep the same hashed binaries and resource budget. Extra WF probes
can affect scheduling and are not performance samples. Raw record dumps also
check LOST, LOST_SAMPLES and throttle records. Caller/syscall/thread pairing
and complete wait-path coverage must pass before drawing an attribution.

`make mixed-rayon-check` qualifies the optional `mixed-rayon` binary in the
same standalone crate. Its CLI is `mixed-rayon PORT CONNECTIONS --threads B
[--queue Q]`: B includes one current-thread Tokio I/O driver, leaving B-1
Rayon CPU workers. Q bounds queued plus running CPU jobs and defaults to
2*(B-1). Each connection retains one framed request/reply; a full CPU queue
asynchronously suspends its handler. Zero-round requests stay on the I/O
thread, and CPU admission is released before response writes. Protocol
errors terminate the sample after handlers and CPU jobs are drained.

The canonical gate calls its five lifecycle tests and the existing independent
`stream_check` compute/truncation oracle. The optional `mixed-observe` feature
adds queue/concurrency/lifecycle counters; ordinary builds omit them. Socket2
is used only by the Rust tests to force TCP backpressure and RST safely.
`make mixed-rayon-smoke` adds four Linux-only, half-second runs of the existing
paced `netload` client at total budgets 2/4 with ordinary/observed binaries.
Those are protocol qualifications on a shared host, not a performance ranking.
Experiment 42 owns this bin, its caller and their retirement condition.
