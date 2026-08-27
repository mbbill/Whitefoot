# Completion I/O results

Status: measured at the program level on 2026-08-27, on macOS and on Linux
with io_uring. Everything before that date was a C-level measurement of the
completion core alone; those numbers are retained below, labelled, because
they still describe what they measured.

The program-level section is the one that answers the design's own question.
Reproduce it with:

```sh
make -C research/experiments/io-completion-bench bench       # macOS
make -C research/experiments/io-completion-bench bench-pipe
make -C research/experiments/io-completion-bench linux       # Linux, io_uring
```

## Program-level results, 2026-08-27

### What is compared

Three lines, every one checked to publish the same bytes on every recorded
run:

```text
N   the best hand-written native C shape, -O2, no handicap
S   the Whitefoot program built with `whitefootc --no-overlap`
C   the same Whitefoot source built the way it ships
```

C and S are one source compiled two ways, so the pair is a statement about
the lowering and not about two programs. Protocol: two warm-up runs, seven
recorded runs, medians reported, child user and system CPU from `wait4`.

### Workload 1: many independent files

8,192 generated files of mixed size, 1 to 16 KiB, 68 MiB in total, each
opened by name and transferred with one positioned read, folded into one
position-weighted checksum. Warm page cache. Two Whitefoot programs over the
identical work: `many_files_wide.wf` states four opens and then four reads
consecutively; `many_files_narrow.wf` is the natural one-file-at-a-time loop.

macOS 26.5.2, Apple M4, 10 cores, 16 GiB:

```text
line                          median      user      sys
N.direct                     948.9 ms    45.4     253.9
N.pool4                      365.5 ms    69.6     390.5
N.pool6                      289.8 ms    56.9     375.6      best N
S.wide                       974.8 ms    46.5     252.9      best S
S.narrow                     989.4 ms    50.0     255.1
C.wide  (default policy)     475.2 ms    63.0     348.8      best C
C.wide  WF_IO_HELPERS=0     1018.9 ms    50.6     287.8
C.wide  WF_IO_HELPERS=2      647.5 ms    61.5     315.9
C.wide  WF_IO_HELPERS=4      479.3 ms    67.0     378.7
C.narrow (default policy)   1069.1 ms    48.9     304.0
```

Linux 6.8.0 aarch64, 2 CPUs, 1.9 GiB, in a container with io_uring permitted:

```text
line                          median      user      sys
N.direct                      56.5 ms    37.0      20.0
N.pool2                       30.5 ms    41.4      19.8      best N
N.pool4                       31.1 ms    39.7      20.8
N.uring4                     117.1 ms    50.8      46.1
N.uring8                      74.6 ms    45.1      53.1
N.uring32                     63.9 ms    44.9      36.7      best native ring
S.wide                       292.0 ms    60.5     123.4      best S
S.narrow                     303.3 ms    69.0     122.2
C.wide  (default policy)     121.1 ms    51.3      54.3      best C
C.wide  WF_IO_HELPERS=0      121.0 ms    51.6      53.7
C.wide  WF_IO_HELPERS=1      157.4 ms    63.1      81.7
C.wide  WF_IO_HELPERS=2      173.6 ms    64.2      84.2
C.narrow (default policy)    300.9 ms    65.3     130.6
```

### Workload 2: two independent output streams

64 rounds of 256 KiB onto `command.stdout` and `command.stderr`, chunks
larger than the pipe buffer so a write genuinely waits, two independent
consumers each sleeping 1 ms per read. macOS, medians of seven:

```text
N.seq        394.8 ms
N.threads    389.9 ms
S.relay      395.6 ms
C.relay      390.4 ms
```

All four lines fall inside 1.5 percent of each other, including the ideal
native shape of one thread per stream. The workload is entirely
consumer-bound: the pipe buffer already decouples the two streams, so a
producer that serializes its two writes loses nothing that overlap could
recover. This measures no gain and no loss; it does not measure the absence
of overlap. The semantic property — an independent operation running while
another is blocked — is pinned separately and executably by
`independent_io_reaches_the_second_operation_before_the_first_unblocks`.

### Against the bar

The bar: C at least as fast as S on every workload, and within 10 percent of
N wherever N is a native completion path or a fairly sized thread pool.

```text
workload / platform      C vs S              C vs N                    bar
many files / macOS       2.05x faster        1.64x slower than pool6   missed
                                             1.30x slower than pool4
many files / Linux       2.41x faster        3.97x slower than pool2   missed
                                             1.89x slower than uring32
                                             1.03x slower than uring4  met
two streams / macOS      1.01x faster        1.00x of threads          met
```

C beats S on every workload, so overlap is real and it is worth roughly two
times on a program that exposes width. The 10 percent bar is met against the
native completion path **at the concurrency the Whitefoot source can
actually ask for**, and missed against every native shape that asks for
more.

### Where the remaining distance is

It is not the completion protocol. On Linux the four-wide Whitefoot program
and the hand-written io_uring pipeline at queue depth four land 3.4 percent
apart, on the same kernel, the same tree, the same checksum. Widen the native
baseline to depth 32 and it pulls away to 63.9 ms; the Whitefoot source has
no way to ask for depth 32.

The overlap a program gets is decided by its source shape. The lowering forms
groups from runs of *consecutive* calls in one basic block
(`IrBuilder::completion_steps`, with `has_later_independent_call` in
`semantic/permission.rs`), so a loop body holding one I/O call per iteration
has no later independent call to overlap with and emits zero submissions.
That is exactly what `many_files_narrow.wf` does, and it measures within
noise of its own sequential build on both platforms — 1069 against 989 ms on
macOS, 301 against 303 ms on Linux. The four-wide program over identical
work is about twice as fast as its own sequential build on both.

Two further facts bound the native ceiling rather than the Whitefoot one:

1. A blocking thread pool beats io_uring on this workload at every depth
   tested — 30.5 ms against 63.9 ms on Linux. Opens dominate a
   many-small-file workload and the ring does not carry them, so each file
   still costs a blocking `openat` on the submitting thread. Whitefoot's own
   Linux adapter has the same split: `linux_io_uring.c` submits
   `IORING_OP_READ` and `IORING_OP_WRITE` and nothing else.
2. The pool baseline also folds each file's checksum on the worker that read
   it, so part of its advantage is compute parallelism the Whitefoot program
   does not have — its fold runs on the single writer thread. On Linux
   `N.pool2` uses 61 ms of CPU across two cores for a 30.5 ms wall time,
   while the four-wide C line uses 106 ms of CPU on essentially one.

### What this run changed in the runtime

Three defects were found by measuring and fixed on the same branch.

```text
                                    before          after
macOS default, user CPU             266 ms          60 ms
macOS default, wall                 838 ms         475 ms
Linux default, wall                 171 ms         121 ms
Linux link of a completion program  did not compile  compiles
```

1. A scheduler waiting in `wf__completion_file_join` refused to park while
   the target queue held anything at all. With helpers it cannot execute
   that queued work, so it spun for exactly as long as a helper kept the
   queue non-empty. The guard is now one named predicate true only in the
   zero-helper configuration, where the waiting scheduler really is the
   target's engine.
2. An unset `WF_IO_HELPERS` pinned one helper, which measured worst of every
   setting on a program with width. The policy now starts at one, grows by
   one only when the queue holds more requests than there are helpers to
   take them, and stops at the machine's CPU count — and starts at **zero**
   where a native ring is ready, because there the ring already carries
   every transfer and a helper can only add a handoff to the operations it
   does not take. A written `WF_IO_HELPERS` still pins the count exactly.
3. `bridge.c` declared a union member spelled `linux`. `whitefootc` compiles
   the runtime units with the host compiler's default dialect, which
   predefines `linux` as `1`, so every Linux link of a completion program
   failed in the C compiler. The repository's own Linux probes compile with
   `-std=c11`, where the macro is absent, which is why the adapter evidence
   below was real while the compiler was still broken.

### What these numbers do not cover

Cold storage, durable writes, network I/O, Windows IOCP execution, machines
with more than two Linux CPUs, and any workload whose operations genuinely
wait. Every file measurement here has a warm page cache, which is the case
least favourable to completion and most favourable to a direct call. The
uncached variant was not run: `fcntl(F_NOCACHE)` and `posix_fadvise` are
available to the C baseline but the Whitefoot surface exposes no equivalent,
so the two lines could not have been compared on the same terms.

## Historical C-core results

Everything below measures the completion core and its adapters directly,
before any program-level measurement existed. It remains evidence for what it
measured.

The owner-confirmed design of 2026-08-26 removed the former Ordered batch and
capability-root layers. The first historical result measures commit
`6dec866363cdaceaaa2e26ef57971ede79abf098`; the last historical rerun uses the
completed admission protocol, distinct admission/capacity notification paths,
and target adapters described below.


## Unified-state rebuild result, 2026-08-27

The final work-branch tree passes the complete executable validation below:

```text
default Rust library tests       1321 passed, 0 failed
gate all-target Rust tests       1412 passed, 0 failed, 1 costly adapter ignored
separate conformance adapter     502 Pass, 1 Skip
maintained programs              54 passed, 0 failed
macOS native helper policies     0, 1, and 4 helpers passed
ASan and UBSan                   passed
core/read hostile stress         200 of 200 passed
conformance structure/coverage   29 of 29; 137 of 137 rules
```

The Windows x86-64 completion and adapter probes strict-cross-link with the
required IOCP and overlapped-I/O imports. This is compile/link evidence, not a
Windows execution result, so Windows production qualification remains closed.

Canonical `make check` reaches the specification archive gate and stops with
the expected statement that a `CANDIDATE` is valid branch work but not a
merge-ready `ACTIVE` identity. Every later component was also invoked
independently and passed. Activation and merge remain separate owner-approved
work; this expected stop is not an implementation or test failure.

### Cleaned runtime measurement

The historical benchmark source was rebuilt against the final unified-state
runtime after the Ordered, family, root, and batch-admission machinery had been
deleted. Each of three complete runs performed its semantic self-check for
direct, zero-helper progress, and helper-park `pread`, `read`, and `write`
paths before recording timings. The measured source identities were:

```text
runtime.c       8042e1df711f01d1ea45d8bde12f25c13d97e8316b1f6e3c6430055f28940cde
contract.h      f54e31f452be2baa0ec3a471271193e577056c3df5724b410aec54318480b0a4
file_adapter.c  535fc63a6232df92fee218fa88d2402cff1703551576964af36e2387c3ea18f6
```

The generation-checked accepted-terminal round trip was stable at 35.594 to
36.299 ns/op. The inline-terminal samples were 33.337 to 41.295 ns/op; one run
was visibly bimodal, while the other two medians were 33.856 and 34.137 ns/op.

The cached sequential-read comparison was stable across all three runs:

```text
run                    1          2          3
direct read         410.619    411.346    423.864 ns/op
0-helper progress   468.023    481.394    494.929 ns/op
added cost           57.404     70.048     71.065 ns/op
relative cost          14.0%      17.0%      16.8%
```

Positioned reads and 64-byte writes had run-to-run dispersion large enough to
swamp or materially change the measured delta. No comparative number is
claimed for those two operations from this rerun. The full raw records remain
outside the repository:

```text
$WHITEFOOT_SCRATCH_ROOT/whitefoot-completion-perf/run-unified-state.txt
$WHITEFOOT_SCRATCH_ROOT/whitefoot-completion-perf/run-unified-state-2.txt
$WHITEFOOT_SCRATCH_ROOT/whitefoot-completion-perf/run-unified-state-3.txt
```

The measurement supports the selected lowering boundary. A call with no
independent work should use the direct or inline specialization. A call with
real independent work may pay roughly 57 to 71 ns on this cached-read host to
expose overlap. These numbers do not predict cold storage, durable writes,
network I/O, Linux io_uring throughput, or Windows IOCP execution.

## The deciding question

How much does the completion contract add when the host operation is already
fast enough that no overlap can repay it?

The matched cached-read comparison answers that question:

```text
direct pread                 366 to 380 ns/op
0-helper completion pread   430 to 439 ns/op
added cost                   53 to 64 ns/op, about 14 to 17 percent
```

This selects an inline or direct depth-one specialization. It does not select
a second writer-visible blocking API. The same normal call can use completion
when independent work is available and the direct specialization when there is
none.

## Host and method

The host was a Mac16,12 with an Apple M4, 10 cores, 16 GiB memory, macOS
26.5.2 build 25F84, and Apple clang 21.0.0. The process was not pinned to one
CPU.

Each I/O measurement used two warmup rounds and nine recorded rounds of 8,192
operations. Reads used a warmed 64 MiB scratch file and transferred 4 KiB.
Writes used distinct scratch files and transferred 64 bytes without `fsync`.
The core measurement used 100,000 complete round trips per round. The wake
experiment recorded 18,000 samples for each path. Every mode checked result
bytes, milestone bits, publication and consumption counts, and capacity-wait
counts.

Scratch sources and the 72-line raw result remain outside the repository:

```text
$WHITEFOOT_SCRATCH_ROOT/whitefoot-completion-perf/completion_bench.c
$WHITEFOOT_SCRATCH_ROOT/whitefoot-completion-perf/run-highres.txt
$WHITEFOOT_SCRATCH_ROOT/whitefoot-completion-perf/run-current.txt
$WHITEFOOT_SCRATCH_ROOT/whitefoot-completion-perf/run-admission-final.txt
$WHITEFOOT_SCRATCH_ROOT/whitefoot-completion-perf/run-final-rebuild-3.txt
$WHITEFOOT_SCRATCH_ROOT/whitefoot-completion-perf/run-final-consume-wait.txt
```

## Final integrated rerun

The same O3 source, host, warmup count, sample count, operation count, data
file, and byte checks were rerun after Ordered batch admission, adapter tags,
stackless dependencies, multi-waiter wake, and the admission/capacity split had
landed. This last freeze additionally includes drain-before-resume, bounded
free-group admission, an exact consume-wait handshake, and
one-waiter/multi-waiter wake selection. The measured
core bytes were `runtime.c`
`3ba660a59c768b0f1f9a69525acfbd7b7cd0d0d581cae231c6d304e6b219624f`
and `contract.h`
`d4373afc8d7592c99471484f9532079759d8030c7ed6bbb3388edcaca1882dce`.

```text
accepted claim, publish, drain, consume   35.854 ns/op
inline terminal                           33.776 ns/op
direct pread 4 KiB                       354.146 ns/op
completion-progress pread 4 KiB          418.798 ns/op
added pread cost                           64.652 ns/op, 18.3%
direct read 4 KiB                        407.257 ns/op
completion-progress read 4 KiB           468.374 ns/op
added read cost                            61.117 ns/op, 15.0%
direct write 64 B                       2,656.301 ns/op
completion-progress write 64 B          2,756.526 ns/op
added write cost                          100.225 ns/op, 3.8%
```

An intermediate correct implementation placed one global mutex around every
single claim solely to make multi-slot batch admission atomic. The ordinary
O2 harness then reported roughly 93–154 ns per round trip. That shape was
discarded: a short atomic admission gate now lets single claims retain their
independent slot scan, while a rare batch waits for in-progress single
claimers and obtains all requested slots or none. Reopening that gate has its
own counter and uses the shared wake path, selecting one signal for one parked
scheduler and a broadcast for several; it neither claims that slot capacity
was released nor wakes when no single claimant observed the closed gate. An
exact token waiter registers only across its final recheck-to-park window, so
an uncontended drain pays no second epoch publication. The final core is within
0.3 ns of the original clean-core range; the cached-pread delta is 64.7 ns, at
the edge of the original 53–64 ns band.

## Core cost

```text
accepted claim, publish, drain, consume   34.9 to 35.6 ns/op
inline terminal                           31.7 to 32.9 ns/op
```

The operation record, generation check, four milestone facts, ready event,
and slot reuse fit inside that round trip.

## Cached host operations

```text
operation      direct             0-helper completion      delta
pread 4 KiB    366 to 380 ns      430 to 439 ns             +14 to 17%
read 4 KiB     415 to 430 ns      472 to 476 ns             +10 to 15%
write 64 B     2.59 to 2.72 us    2.67 to 2.73 us           about 0 to 5%
```

The write delta is smaller than the observed buffered-write noise. These
numbers establish neither durable-write cost nor cold-file behavior.

One, two, and four helper threads were tested at depths two, four, and eight.
No cached-read crossover appeared. The best read configuration was one helper
at depth eight and still cost about 2.6 to 2.9 times direct calls. The closest
write configuration was two helpers at depth eight: 3.67 to 3.88 us/op against
2.67 to 2.92 us/op direct, about 26 to 45 percent slower. More helpers increased
shared-queue, handoff, and publication contention for these sub-microsecond
operations.

A complete depth-one helper plus park path cost about 4.2 to 4.3 us for read or
pread and about 8.25 us for write. A helper fallback therefore needs enough
real target waiting or overlap to repay thread handoff. It cannot be the
unconditional common path.

## Park and wake

```text
path                    median       p95
completion resume       1.625 us     2.083 us
raw pthread condvar     1.542 us     2.125 us
```

The completion median was about 5.4 percent higher and its measured p95 was
about 2.0 percent lower. Notification call time was 625 ns against 583 ns,
about 7.2 percent higher. These sub-microsecond differences are small enough
that the mechanism conclusion is parity rather than a stable win for either
path.

This is the matched comparison for the new wake epoch. The experimental
branch's approximately 2.25 times single-run wake result measured a different
kqueue relay and remains historical evidence about that discarded runtime,
not evidence about this core.

## Consequences for implementation

1. Pure computation links no completion code.
2. A depth-one fast host operation uses inline or direct completion.
3. A completion overlap run pays the core only when it has independent work to
   expose.
4. A blocking helper is target fallback for operations likely to wait. It is
   not a fixed common pool and its count is target policy.
5. Native io_uring and IOCP remain necessary experiments because cached helper
   results cannot predict their queue and completion costs.

## Native adapter evidence

The first native adapter probes were added after the timing run above, so they
establish correctness and target shape rather than comparative performance.

On Linux 6.8.0 aarch64, the raw io_uring probe executed two positioned reads
simultaneously, observed bounded-capacity refusal of a third operation,
resubmitted after capacity release, executed one positioned write, and checked
result bytes, milestones, and final file contents. The adapter submits real
`IORING_OP_READ` and `IORING_OP_WRITE` entries and requires
`IORING_FEAT_NODROP`; it does not poll an eventfd as a substitute for I/O.
The final bridge waits on one epoll set containing the io_uring fd and an
eventfd for compute, target, admission, and capacity publication. CQ readiness wakes directly through
the ring fd. A four-token staggered hostile test and 20/20 concurrent W4 runs
on the final runtime pin the broadcast rule: one waiter cannot drain the
eventfd while another
already-announced waiter still owns a private completion condition.

The Windows adapter and Win32-native completion core strict-cross-link as an
x86-64 PE. Its import table contains `CreateIoCompletionPort`,
`GetQueuedCompletionStatus`, `PostQueuedCompletionStatus`, `ReadFile`, and
`WriteFile`. A reserved-key null `OVERLAPPED` packet wakes an announced
scheduler after a compute, terminal, or capacity epoch change; it never names
an I/O operation and does not count as a published terminal. The cross-linked
probe covers bounded adapter capacity, real packet/wake interleaving, stale
generations, duplicate terminal rejection, product milestones, adapter tags,
batch claim atomicity, and scheduler-frame injection. These are compiled test
paths, not execution evidence. Production Windows qualification remains
fail-closed for two independent reasons: no Windows runner was available, and
the current IOCP wake packet is neither coalesced nor persistent for every
already-announced waiter. Both must close before `implemented` can change.

The reproducible local cross-link and import check is:

```sh
make -C compiler completion-windows-cross
```

macOS continues to use the bounded typed helper fallback for regular files.
Both zero-helper scheduler progress and one-helper execution pass the shared
harness. Target helpers receive no writer function pointer.

## Reproduction

```sh
REPO_ROOT=/path/to/Whitefoot
WHITEFOOT_SCRATCH_ROOT=/path/to/scratch

clang -std=c11 -O3 -DNDEBUG -Wall -Wextra -Werror -pthread \
  -I"$REPO_ROOT/compiler/src/backend/completion" \
  "$WHITEFOOT_SCRATCH_ROOT/whitefoot-completion-perf/completion_bench.c" \
  "$REPO_ROOT/compiler/src/backend/completion/runtime.c" \
  "$REPO_ROOT/compiler/src/backend/completion/file_adapter.c" \
  -o "$WHITEFOOT_SCRATCH_ROOT/whitefoot-completion-perf/completion_bench"

"$WHITEFOOT_SCRATCH_ROOT/whitefoot-completion-perf/completion_bench" \
  "$WHITEFOOT_SCRATCH_ROOT/whitefoot-completion-perf/data.bin" \
  "$WHITEFOOT_SCRATCH_ROOT/whitefoot-completion-perf/write" \
  "$WHITEFOOT_SCRATCH_ROOT/whitefoot-completion-perf/run-highres.txt"
```

The timing results do not cover cold or durable I/O, real network traffic,
native io_uring throughput, Windows IOCP execution, scheduler contention in a
complete Whitefoot program, or continuation-frame resume cost. Cached
operations produced no helper crossover; that observation does not imply that
high-latency operations have none.
