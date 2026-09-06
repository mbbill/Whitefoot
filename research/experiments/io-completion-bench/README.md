# io-completion-bench

Program-level measurement of the unified-state completion I/O model against
the best hand-written native shape and against Whitefoot's own sequential
build.

## What it serves

`research/investigations/io-model/RESULTS.md` held only C-level
microbenchmarks of the completion core — round-trip cost, cached `pread`
delta, park/wake parity. Those numbers cannot answer the question the design
stands or falls on: whether a whole Whitefoot program that does real I/O
reaches the best native performance. This bundle answers that question and
supplies the evidence section that RESULTS.md now carries.

It is removed when the completion model stops being an open performance
question — when the numbers are stable, the bar is settled, and no further
runtime or lowering change is being measured against them.

## The three lines

Every line publishes the same bytes; a line that publishes anything else
cannot report a time.

- **N** — the best hand-written native C shape. `baseline.c`: a single-threaded
  blocking loop, a pthread pool over a striped index range, and on Linux a raw
  `io_uring` read pipeline (`uring_baseline.h`, kernel ABI directly, no
  liburing). Compiled `-O2` with no handicap.
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

The control test `research/investigations/io-model/NETWORK.md` section 6 asks
for, and the bar it sets: the reference is the fastest existing solution
regardless of language, because the target is first place, and the gap is the
batch's result rather than something to hide.

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
  filled, so the data is not copied on either side of the exchange. Exhaustion
  is real and is handled rather than avoided: a receive that finds no buffer
  answers `-ENOBUFS`, and that connection waits for a buffer to come back
  instead of spinning on a re-arm.
- **one ring per core.** Each thread owns its ring, its buffer ring and its own
  `SO_REUSEPORT` listening socket, so a connection is accepted, received and
  echoed on one thread with nothing shared on the path.
- **a ring the kernel need not interrupt.** `IORING_SETUP_SINGLE_ISSUER` with
  `IORING_SETUP_DEFER_TASKRUN` says one thread submits and the same thread
  waits, which lets the kernel defer completion work to the moment that thread
  asks for it. It is worth about a quarter of the small-message rate here, and
  it is what puts the io_uring line ahead of the epoll line at 64 and 1024
  connections instead of level with it.

`--sqpoll` adds `IORING_SETUP_SQPOLL` and is off by default. The development
host admits it; whether another host does is what `uring_echo --sqpoll` says
there, since a kernel that refuses the flag refuses it at `io_uring_setup` and
the server reports that and exits. The protocol does not run it, because a
poll thread per ring costs a core each and on a four-core host it loses to the
default by a third. It cannot be combined with the deferred task work above,
since there the submitting task is the kernel's own.

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
and `uring_echo` are written against Linux interfaces, and the workload's
point is the fastest shape that kernel offers.

The current scheduler experiments and their controls are described in
[`SCHEDULER-EXPERIMENT.md`](../../investigations/io-model/SCHEDULER-EXPERIMENT.md).
`scheduler-checkpoint`, `scheduler-footprint`, `scheduler-paced`,
`scheduler-chunks`, `scheduler-canonical`, `scheduler-stackful` and
`scheduler-stackful-paced`, `scheduler-nodelay`, `scheduler-owner` and
`scheduler-owner-paced`, `scheduler-memory`, `scheduler-dispatch` and
`scheduler-dispatch-paced` run their respective Linux cohorts. The measured
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

This bundle is deliberately not reachable from the repository's canonical
`make check`. It generates a large tree and runs for minutes, so correctness
builds do not depend on a performance host. The dedicated `io-bench` workflow
owns the Windows qualification and the exploratory Linux/macOS tables.
Generated trees, binaries, and raw output stay in the selected scratch
directory; durable results retain the host identity and raw artifact beside
their summarized table.
