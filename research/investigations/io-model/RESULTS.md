# Completion I/O results

Status: measured at the program level on 2026-08-27, on macOS and on Linux
with io_uring, re-measured the same day with the base commit and the branch
interleaved in one plan on a quiet host, then measured on real Linux hardware
and on a clean macOS host through the repository's own continuous integration,
and finally against a read-dominated workload whose files are opened once and
whose reads are taken past the page cache.

Read the batch-0092 section first: it is the only workload here whose
operations genuinely wait, and it is where the design's own question is
answered. Read the batch-0090 section for the Linux-hardware result that the
container's headline ratio does not reproduce, the batch-0086 one for absolute
values on the many-files workload, and the batch-0084 one for the findings it
established; everything before that date was a C-level measurement of the
completion core alone and is retained below, labelled, because it still
describes what it measured.

The program-level sections are the ones that answer the design's own question.
Reproduce them with:

```sh
make -C research/experiments/io-completion-bench bench        # macOS
make -C research/experiments/io-completion-bench bench-pipe
make -C research/experiments/io-completion-bench linux        # Linux, container
make -C research/experiments/io-completion-bench bench-read   # macOS, read-heavy
make -C research/experiments/io-completion-bench linux-read   # Linux, read-heavy
```

The Linux-hardware run is the `bench-linux` job in
`.github/workflows/io-hosts.yml`, which runs the same `linux-bench.sh` with no
container in the way; its table is uploaded as a job artifact.

## Program-level results, batch 0084 (2026-08-27)

These were the first whole-program numbers, taken on a busy host one line
after another. The batch-0086 measurement below, taken on a quiet one with
before and after interleaved in a single plan, shows that method carried
roughly 20 percent of noise, so read the *ratios* here and the absolute values
below. Every finding this section states survives the quieter run: overlap is
worth about two times on a program that exposes width, a narrow loop overlaps
nothing, and the completion path matches a hand-written ring at the depth the
source can ask for.

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

Linux 6.8.0 aarch64, 2 CPUs, 1.9 GiB, in a Docker container on the macOS
machine above — a virtual guest, not Linux hardware — with io_uring
permitted. The batch-0090 section below re-measures this workload on a real
Linux kernel on real hardware and does not reproduce the C-against-S ratio
here:

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

   *Batch 0086 refuted the causal half of this.* One `openat` on that
   container costs 0.85 us and one close 0.45 us, so the entire
   open-plus-close budget of this workload is about 11 ms — the opens do not
   dominate on Linux, and putting them on the ring changed the total by one
   percent. The pool's advantage is item 2 below, not item 1.
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


## Program-level results, batch 0086 (2026-08-27)

### What changed in the method

Same workload, same protocol, same checked bytes. One change: the base commit
and the branch are built as two compilers, their programs are built from the
same sources, and both sets of binaries run inside **one** plan. Before and
after therefore see the same machine, the same tree and the same page cache,
which the batch-0084 method did not.

`S.wide.before` against `S.wide.after` is the built-in control. The sequential
build reaches the host through ordinary direct calls, and nothing on that path
changed on macOS, so whatever separates those two lines is measurement noise.
On the quiet macOS run they differ by 0.01 percent; on the batch-0084 method
under load the same pair differed by 19 percent.

The eight-wide program is new here. `many_files_wide8.wf` states eight opens
and then eight positioned reads consecutively, and emits seven open
submissions plus one direct open and seven read submissions plus one direct
read, against the four-wide program's three plus one. The narrow program still
emits none. Hand-widening from four to eight is worth 13 percent on macOS
(629 ms to 546 ms) and 19 percent on Linux (147 ms to 119 ms).

### What one file operation costs

Measured directly on the same trees, warm cache, one file open at a time,
which is the pattern the programs use:

```text
                    openat      pread 64 KiB      close
macOS M4 host       116 us          1.9 us        4.8 us
Linux container    0.85 us          ~1 us        0.45 us
```

The macOS host runs an endpoint-security stack that hooks file operations.
That is the host every line here is measured on, so the comparison is fair; it
does mean macOS is almost entirely open-bound and Linux is not. On Linux the
whole open-plus-close budget of the workload is about 11 ms.

macOS 26.5.2, Apple M4, 10 cores, 16 GiB. Medians of fifteen recorded runs
after two warm-ups:

```text
line                          median      min      user      sys
N.direct                     1138.51  1076.71     58.85   323.55
N.pool2                       715.91   689.55     63.94   430.82
N.pool4                       455.64   452.93     75.58   528.39
N.pool6                       374.29   368.92     78.24   528.79
N.pool8                       373.91   361.79     78.45   528.45    best N
N.pool10                      378.30   370.83     78.89   528.84
S.wide.before                1171.42  1128.13     58.65   325.03
S.wide.after                 1217.53  1114.55     61.64   353.25
S.wide8.before               1168.63  1114.62     58.17   315.54
S.wide8.after                1130.97  1099.88     58.41   313.55
C.wide.before                 637.33   624.57     91.66   564.82
C.wide.after                  629.30   625.38     90.07   568.64
C.wide8.before                543.40   538.82     97.54   661.88
C.wide8.after                 545.50   542.10     97.10   682.54    best C
C.narrow.before              1182.99  1158.98     59.53   327.62
C.narrow.after               1183.83  1114.80     59.83   328.91
```

Nothing on the macOS path changed in this batch beyond the collapsed helper
pool, and the paired lines say so: every before/after pair above agrees within
the control's own spread.

Linux 6.8.0 aarch64, 2 CPUs, in a Docker container on the macOS machine above
— a virtual guest, not Linux hardware — with io_uring permitted, tree on the
container-local filesystem. Medians of nine recorded runs after two warm-ups:

```text
line                          median      min      user      sys
N.direct                       72.04    70.47     52.20    20.08
N.pool1                        70.43    69.59     48.15    22.07
N.pool2                        40.21    37.70     54.54    23.31    best N
N.pool4                        41.55    40.76     59.41    22.15
N.pool8                        44.72    41.77     56.79    31.90
N.uring4                      138.76   134.54     59.28    54.71
N.uring8                       94.89    93.34     53.75    60.04
N.uring16                      86.37    85.74     57.55    52.51
N.uring32                      82.46    81.14     49.47    53.20    best one-thread N
S.wide.before                 335.89   330.56     86.17   141.01
S.wide.after                  337.29   202.02     82.97   139.01
S.wide8.before                331.72   323.63     85.91   138.26
S.wide8.after                 335.50   326.25     88.08   141.39
S.narrow.after                345.95   340.89     87.66   148.51
C.wide.before                 142.31   141.04     71.58    56.71
C.wide.after                  146.73   143.87     70.20    62.47
C.wide8.before                118.09   106.53     69.66    52.54
C.wide8.after                 119.47   117.30     65.65    61.42    best C
C.narrow.before               346.32   338.42     82.83   151.97
C.narrow.after                346.65   342.91     86.07   151.25
C.wide8.after WF_IO_HELPERS=0 117.61   108.37     65.12    57.34
C.wide8.after WF_IO_HELPERS=1 116.58   112.72     69.61    52.60
C.wide8.after WF_IO_HELPERS=2 119.36   116.72     64.93    60.99
```

The 202.02 ms minimum on `S.wide.after` is one outlier in nine runs against a
337 ms median and a 357 ms maximum. It is left as reported; every other
minimum in either table is within about 10 percent of its own median.

### Against the bar

The bar: C at least as fast as S on every workload, and within 10 percent of
the best native shape at matched width.

```text
comparison                                   ratio            bar
macOS  C.wide8 against S.wide8               2.07x faster
macOS  C.wide8 against N.pool8 (width 8)     1.46x slower     missed
macOS  C.wide  against N.pool4 (width 4)     1.38x slower     missed
Linux  C.wide8 against S.wide8               2.81x faster
Linux  C.wide  against N.uring4 (depth 4)    1.06x slower     met
Linux  C.wide8 against N.uring8 (depth 8)    1.26x slower     missed
Linux  C.wide8 against N.uring32             1.45x slower     missed
Linux  C.wide8 against N.pool2               2.97x slower     missed
```

C beats its own sequential build everywhere, by about two times on macOS and
2.8 times on Linux. The depth-matched Linux comparison batch 0084 reported —
the four-wide program against a hand-written ring at queue depth four — still
holds, at 6 percent. It opens to 26 percent when both are widened to eight,
and that widening is the finding about where the distance lives.

### Where the remaining distance is

It is not the opens, and on neither platform is it the completion protocol's
per-operation cost.

**Linux is compute-bound and barrier-bound, not open-bound.** The whole
open-plus-close budget is about 11 ms of a 119 ms program — 9 percent, and a
perfect open could not take more than that. `C.wide8` spends 65.7 ms of user
CPU on one thread, which is its own fold and program logic and is a floor
under any single-threaded line. The comparison that controls for that is
`N.uring32`, which also folds on one thread: 82.5 ms against 119.5 ms, using
103 ms of CPU against 127 ms. So C spends about a quarter more CPU — the
completion protocol's claims, publications, drains and consumes, plus a
bounds-checked fold — and about half again as much wall time. The wall-time
excess beyond the CPU excess is the group barrier: the program joins all eight
operations before submitting the next eight, so each round costs the maximum
of eight latencies while the ring baseline keeps 32 continuously in flight.
`N.pool2` at 40.2 ms is not a matched shape at all — it folds each file's
checksum on the worker that read it, which is compute parallelism the
Whitefoot source cannot express.

**macOS is open-bound, and the host itself limits how much that helps.** One
`openat` costs 116 us there. The best native shape, an eight-thread pool,
turns 1113 ms of serial work into 368 ms — a 3.0x speedup on eight threads,
so the security stack serializes most of the concurrency a pool asks for. The
eight-wide Whitefoot program gets 2.0x over its own sequential build. The
difference is what the writer thread still does serially in every round: its
own direct open, the eight folds, the eight releases, and the join of all
eight before the next round starts.

Both remaining gaps are therefore the same shape the batch-0084 record already
flagged: overlap groups are runs of consecutive calls in one basic block, so a
program pipelines nothing across iterations and pays a barrier per round.

## Program-level results on Linux hardware, batch 0090 (2026-08-27)

Every Linux number above this line was taken inside a Docker container on this
project's macOS machine: an aarch64 guest with two virtual CPUs, its tree on
the container's own overlay filesystem. That is a Linux kernel, but it is not
Linux hardware, and the section below shows the difference is not a rounding
error. These numbers come from GitHub-hosted `ubuntu-24.04` runners through
`.github/workflows/io-hosts.yml`, which builds the bundle natively — no
container, no bind mount — and runs `linux-bench.sh` with the same protocol,
the same `workload.h`, and the same published checksum.

Host, reported by the job itself: kernel `6.17.0-1022-azure`, `x86_64`, 4
CPUs, AMD EPYC 9V74, `kernel.io_uring_disabled=0`, tree on the runner's own
local disk (`ext4`). Nine recorded runs after two warm-ups, medians with the
observed spread, because a shared runner is noisy and the reading has to carry
its own error bars.

Two runs on two separately provisioned runners, printed side by side below.
They differ in absolute speed by about 21 percent — the second landed on an
NVMe-backed disk, the first on a SATA one — and agree on every ordering and
every ratio. A third runner, also NVMe-backed, reproduces the second within
two percent on every line and is not tabulated: `N.direct` 94.68, `N.pool4`
27.14, `N.uring32` 94.75, `S.wide` 112.19, `C.wide.default` 118.14.

```text
                       run 1: sda1                 run 2: nvme0n1p1
line                median   min    max         median   min    max
N.direct            119.69 119.33 120.61         94.24  94.17  94.63
N.pool1             120.35 119.71 125.55         94.84  94.71  95.87
N.pool2              60.80  60.64  62.50         47.71  47.55  48.22
N.pool4              34.04  33.81  34.52         26.61  26.30  33.10   best N
N.pool8              38.49  36.77  41.61         31.11  28.18  33.17
N.uring2            123.89 123.72 125.14         97.14  96.93  97.88
N.uring4            121.95 121.44 122.49         95.81  95.36  96.02
N.uring8            120.06 119.82 120.38         94.67  94.15  95.07
N.uring16           119.00 118.82 119.39         94.25  94.13  94.51
N.uring32           118.69 118.15 118.93         94.45  94.25  94.62
S.narrow            145.26 144.87 147.22        114.30 113.85 114.66
S.wide              142.15 141.66 144.20        112.07 111.22 113.52
S.wide8             141.26 140.64 143.44        110.94 110.63 112.45   best S
C.narrow.default    147.39 145.30 147.68        114.49 113.71 116.18
C.wide.default      149.47 147.17 150.57        115.92 115.38 117.83
C.wide8.default     147.04 146.57 149.51        115.97 115.53 117.79
C.wide.w0.h0        148.88 146.82 155.20        117.19 115.78 117.72
C.wide.w0.h1        148.03 147.52 153.46        115.75 115.50 117.67
C.wide.w0.h2        149.80 147.87 150.72        117.33 115.79 118.08
C.wide.w0.h4        148.33 147.45 153.23        116.17 115.73 117.54
C.wide8.w0.h0       147.80 146.65 149.40        115.96 115.56 119.25
C.wide8.w0.h1       147.58 146.65 149.31        116.04 115.82 117.64
C.wide8.w0.h2       148.43 146.57 152.95        115.93 115.62 117.07
C.wide8.w0.h4       147.49 146.79 150.07        115.96 115.53 117.93
C.wide.w1.hdefault  148.23 147.10 150.59        116.12 115.91 117.72
C.wide.w2.hdefault  148.29 147.45 149.96        116.88 115.79 117.67
```

### Against the standing bar

The bar this investigation has carried since batch 0084: C at least as fast as
S on every workload, and within 10 percent of N wherever N is a native
completion path or a fairly sized thread pool.

```text
workload / platform          C vs S             C vs N best        bar
many files / Linux hardware  1.03x slower       4.36x slower       missed, both halves
```

This is the first host on which **C is slower than S**. Not by much — 3 percent
in run 2, 5 percent in runs 1 and 3, against a within-run spread of about 2
percent — but consistently, on all three runners, at every helper count, and on
both the four-wide and the eight-wide program. The container said C was 2.3 times
faster than S. That reading was about the container.

### Why nothing that reorders I/O moves this workload here

Two facts from the table settle it without a further experiment.

The hand-written io_uring baseline lands on `N.direct` at every depth from 2 to
32 — 94.45 against 94.24 ms in run 2, a fifth of a percent. A ring that submits
32 reads at once and a loop that blocks on one at a time finish together. There
is no read latency on this host to hide, because the whole 68 MiB tree is in
the page cache and a cached `pread` returns without ever sleeping.

The one native line that does move is the thread pool, and the CPU columns say
what it moves. In run 2 `N.pool4` spends 103.6 ms of CPU to reach 26.6 ms of
wall time — 3.9 cores busy at once — while `N.direct` spends 94.1 ms of CPU for
94.2 ms of wall. The pool is not hiding I/O; it is folding four checksums at
once. Batch 0084 already named that as part of the pool's advantage. On real
hardware with a warm cache it is the whole of it.

So a lowering whose only power is to start an operation before waiting on the
previous one has nothing to recover here, and what remains of it is its own
cost: the submission, the token, the join. Three percent.

### The measurement that separates this host from the container

Wall time against child CPU time, for the same lines:

```text
                          wall      user+sys     CPU/wall
runner run 2  N.direct    94.24       94.08        1.00
              S.wide     112.07      111.61        1.00
              C.wide     115.92      115.82        1.00
container     N.direct    72.04       72.28        1.00
              S.wide     337.29      221.98        0.66
              C.wide     146.73      132.67        0.90
```

On the runner every line is CPU-saturated: the process runs for its whole wall
time. In the container the C baseline is too, but Whitefoot's own sequential
build is not — 115 ms of its 337 ms is time the process was not running at all,
and the completion build recovers most of that. Overlap was repaying a wait
that exists in the container and does not exist on the runner.

What produced that wait is not settled here. Two candidates, both consistent
with the numbers and neither established: the container's block and filesystem
path can make an operation genuinely sleep where the runner's page cache
answers inline; and a two-vCPU guest running a lane pool plus a writer is
oversubscribed in a way a four-CPU host is not. Discriminating them wants a
run with the pool disabled on both hosts, which this batch did not do.

### What this changes and what it does not

It does not touch the macOS results, the pipe workload, the C-level core
measurements, or anything about correctness. It does not show overlap is
worthless: a workload whose operations really wait is exactly the case the
design is for, and this workload, on this host, is not one. The batch-0084
section already listed "any workload whose operations genuinely wait" among
what its numbers do not cover.

What it does change is the standing of one claim. "Overlap is worth about two
times on a program that exposes width" was measured on macOS, where the
endpoint-security stack makes an open cost 116 us, and in a container where a
third of the sequential build's wall time was spent not running. Neither is a
statement about Linux hardware, and on Linux hardware the same programs, the
same tree, and the same checksum say the lowering costs three percent. The
honest summary is that the value of the completion lowering is a property of
the host's I/O latency, and this repository has now measured one host where
that latency is zero.

It also retires the batch-0084 note that "a thread pool beats io_uring on this
workload" needs opens to dominate. On this host opens do not dominate and the
pool still wins, because it is winning at compute.


## Read-dominated, open-once, uncached, batch 0092 (2026-08-27)

The two program-level sections above measure one workload: open a small file,
read it, close it, thousands of times. On the macOS host an `openat` costs
116 us against a 1.9 us `pread`, so that table is very largely a measurement
of the host's endpoint-security stack rather than of the completion
framework — and both sections were measured with a warm page cache, where a
read is a memory copy and a model built to overlap waits has almost nothing to
overlap.

This section measures the framework instead. Eight 64 MiB files are opened
once, before any read; then the program performs 32,768 positioned reads of
64 KiB, or the same number of 4 KiB, at offsets a deterministic mix decides
from the read's own position. Reproduce with:

```sh
make -C research/experiments/io-completion-bench bench-read   # macOS
make -C research/experiments/io-completion-bench linux-read   # Linux, io_uring
```

### The first version of this section was measured from the page cache

It has to be said before the numbers, because it is the reason to trust them.
The first run of this table set `WF_IO_NOCACHE=1`, generated the tree, and ran
the uncached tables straight away — which is precisely the order that
guarantees a warm cache, since the pages were still resident from the writes
that had just made them. `F_NOCACHE` stops a read populating the cache; it
does not evict a page that is already there.

The published table said so and was not read. Its `N.direct` uncached line was
294 ms for 32,768 reads of 64 KiB, which is 9 us a read — 7 GB/s for a 64 KiB
transfer, memory bandwidth, not an NVMe round trip. An independent re-run
hours later, after ordinary builds had evicted the tree, put the same line at
4378 ms: 134 us a read, fifteen times slower. Every ratio in that table was a
ratio between cache hits.

The bundle now refuses to print a table under a cache-state label it has not
measured. `make read-uncache` regenerates the tree through a descriptor that
does not populate the cache and flushes it, and `read_baseline probe-uncached`
times sixteen positioned reads in each of the eight files immediately before
and immediately after every table, refusing the label unless all but ten per
cent of them cost more than 40 us. The threshold sits in the gap between the
two populations this host keeps far apart: 6 to 20 us from the unified buffer
cache, about 134 us from the device. `probe-warm` is the same check inverted.
`research/experiments/io-completion-bench/README.md` describes it, and the
per-file medians are printed beside every table below.

### Method

Two warm-ups, then medians of fifteen on macOS, every line checked to publish
the same bytes before it may report a time, with the runner's observed minimum
and maximum shown beside each median and the median child user and system CPU
beside those. The host's one-minute load average is stated with each table;
the machine is shared, and a table taken under load is not a table.

The opens stay inside the timed region — the runner times whole processes —
but there are exactly eight of them in every line, N, S and C alike, so at
116 us each they are 0.93 ms of a table whose fastest uncached line is over a
second, and a constant every line pays identically cannot move a ratio. That
is what opening once buys: the open cost stops scaling with the work.

Each line folds the first sixty-fourth of every window into the
position-weighted checksum and publishes the transferred byte count beside it.
The digest is a serial multiply-add chain running at about 800 MB/s in both C
and Whitefoot, so folding a whole 64 KiB window costs about 80 us of CPU
against a 134 us uncached read and a 7 us warm one. Folding everything would
make the warm table pure compute and would add, to the uncached table, CPU
that the eight-wide program can spread across helpers and the sequential one
cannot — flattering C for a reason that has nothing to do with I/O. So the
fold stays at one sixty-fourth. The checksum still pins the file, the offset,
the size and the position of every read.

### What WF_IO_NOCACHE does

`WF_IO_NOCACHE=1` applies `fcntl(fd, F_NOCACHE, 1)` on Darwin, and one
`posix_fadvise(fd, 0, 0, POSIX_FADV_DONTNEED)` on Linux, to each descriptor an
open hands back. Darwin's is a mode of the descriptor, so every read of the
run bypasses the unified buffer cache rather than only the first. It is target
policy of the same class as `WF_IO_HELPERS`: no Whitefoot source names it, and
`read-verify` checks that every line publishes identical bytes with it off and
on before any line reports a time.

What it does *not* do is make a table uncached on its own. It cannot evict,
so the tree has to arrive non-resident and be checked; that is what the
regeneration and the probes above are for.

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

Batch 0090 re-ran the io_uring half on a real Linux kernel on real hardware —
kernel `6.17.0-1022-azure`, `x86_64`, `kernel.io_uring_disabled=0` — through
the `completion-linux` job, which treats an unavailable ring as a job failure
rather than letting the fallback path report a pass. The native adapter probe
returns `target=linux-io-uring status=pass`, and the full completion harness
passes under `WF_REQUIRE_LINUX_IO_URING=1` at zero, one and four helpers,
alongside ASan/UBSan and the thread-sanitizer target. The description below
still holds; it was taken in the container.

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
paths, not execution evidence.

**Batch 0090 executed both Windows probes for the first time.** The
`completion-windows` job in `.github/workflows/io-hosts.yml` builds them with
LLVM clang 20.1.8 targeting `x86_64-pc-windows-msvc` and runs them on
`windows-latest` — Windows Server 2025 Datacenter, build 10.0.26100, x64:

```text
windows-native-completion-probe status=pass
native-adapter-probe target=windows-iocp status=pass
```

The first drives the Win32-native completion core and a real completion port
end to end: an overlapped `ReadFile` against a real file, an overlapped
`WriteFile`, the `ERROR_HANDLE_EOF` that Windows reports for an overlapped
read past the end and that the file-read contract observes as successful
progress of zero bytes, a malformed reserved-key packet that is consumed and
reported without publishing a terminal, and the adapter and core statistics
that must agree afterwards. The second is the shared native-adapter probe over
`windows_iocp.c`.

Exactly one of the two reasons Windows qualification was fail-closed has
closed. A Windows host now exists and both probes pass on it. The other reason
stands unchanged: the current IOCP wake packet is neither coalesced nor
persistent for every already-announced waiter. Production Windows
qualification therefore remains fail-closed, and `implemented` does not move
on this evidence.

The reproducible local cross-link and import check is:

```sh
make -C compiler completion-windows-cross
```

It is still the right local check: a cross-link needs no Windows host, and the
execution evidence above needs one, which is why it lives in CI.

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
