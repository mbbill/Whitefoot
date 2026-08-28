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

The runner tables come from `.github/workflows/io-hosts.yml`: `bench-linux`
runs `linux-bench.sh` on Linux hardware with no container in the way, and
`bench-linux-read` and `bench-macos-read` run `read-bench.sh` on both hosts.
Every one of them uploads its table as a job artifact and prints it to the job
summary.

Two claims in the dated sections below have since been retired by a later
measurement, and each section says so where it stands. Batch 0084's "a thread
pool beats io_uring only where opens dominate" was retired by batch 0090.
Batch 0084's and 0086's "overlap is worth about two times on a program that
exposes width" was a macOS reading taken on a machine whose endpoint-security
stack charges 116 us for an `openat`; batch 0092 re-ran that workload on an
ordinary macOS system and found the completion build 1.20 times *slower* than
its own sequential build. Neither retirement touches what those sections
measured; both change what may be concluded from it.

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
2.8 times on Linux. Both halves of that sentence were later retired: batch
0090 for Linux, on hardware rather than in a container, and batch 0092 for
macOS, on a host without this machine's endpoint-security stack. What is
measured here stands; what may be concluded from it is in those sections. The depth-matched Linux comparison batch 0084 reported —
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
make -C research/experiments/io-completion-bench bench-read   # macOS, local
make -C research/experiments/io-completion-bench linux-read   # Linux container
```

The recorded tables below come from GitHub-hosted runners, not from the
maintainer's machine, and both runner jobs execute the same
`research/experiments/io-completion-bench/read-bench.sh`. The reason is in the
two sections above: the maintainer's macOS machine runs a corporate
endpoint-security stack that charges 116 us for an `openat` and reads the
benchmark tree behind the benchmark's back, and it is shared with whatever
else the maintainer is doing. A hosted runner has neither problem, and a
hosted macOS runner gives this project its first macOS numbers taken on an
ordinary system. The local table is kept below all the same, labelled
provisional, because it is the only table taken on hardware anyone here owns.

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
measured. Each cache state is created rather than inherited: `make
read-uncache` regenerates the tree through a descriptor that does not populate
the cache and flushes it, and `make read-warm` reads every block of every file
back in through plain ones. The uncached tables run first, because they are
the ones the design question turns on. `read_baseline probe-uncached` then
times sixteen positioned reads in each of the eight files immediately before
and immediately after every table, refusing the label unless all but ten per
cent of them cost more than 40 us. The threshold sits in the gap between the
two populations this host keeps far apart: 6 to 20 us from the unified buffer
cache, about 134 us from the device. `probe-warm` is the same check inverted.
`research/experiments/io-completion-bench/README.md` describes it, and the
per-file medians are printed beside every table below.

### Method

Every line is checked to publish the same bytes before it may report a time.
Timing is by whole pass, not by line: the runner runs the entire plan, then
runs it again, reversing the plan's order on every other pass, for two
unrecorded passes and then seven recorded ones, and reports each line's median
across passes with the observed minimum and maximum beside it and the median
child user and system CPU beside those.

Passes rather than all of one line's runs and then all of the next's, because
these hosts drift. A shared runner's disk, its neighbours, and a laptop's
thermal state all change over the minutes a table takes, and a grouped
schedule turns that drift into a difference between lines: the line that ran
first is measured against a different machine from the line that ran last.
Interleaving spreads the drift across every line, and reversing alternate
passes cancels the residue of position within a pass, so a systematic
first-in-pass or last-in-pass cost lands on every line equally. The local
provisional table at the end of this section predates that change and was
taken line by line; that is one of the reasons it is labelled provisional.

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

### The two runners

```text
Linux runner (io-hosts / bench-linux-read)
  kernel   Linux 6.17.0-1022-azure #22-Ubuntu SMP x86_64
  cpus     4 x Intel Xeon Platinum 8370C @ 2.80GHz
  memory   16372440 kB
  storage  ext4 on /dev/sda1, 87G free, io_uring enabled
  load     0.41 at the start of the job

macOS runner (io-hosts / bench-macos-read)
  kernel   Darwin 23.6.0 RELEASE_ARM64_VMAPPLE, macOS 14.8.7
  cpus     3 x Apple M1 (virtual)
  memory   7516192768 B
  storage  APFS on /dev/disk3s5, 39Gi free
```

Two facts about these hosts decide how the tables below read. The Linux runner
has four cores and 16 GB of RAM, so its page cache holds the whole 512 MiB
tree; the macOS runner has three cores, 7 GB, and is a virtual machine, so
underneath its own page cache there is a host cache it cannot address.

### Linux runner

The 4 KiB table is the cleanest measurement in this document. Its probe
confirmed the uncached label immediately before *and* immediately after the
table, its lines sit within three per cent of their own minimum and maximum
across seven interleaved passes, and every read in it reached the device.

```text
== read-heavy 4 KiB, uncached (WF_IO_NOCACHE=1) ==
probe before: 1 of 128 sampled reads at or below 40 us; per-file medians 160.7..205.6 us
probe after:  2 of 128 sampled reads at or below 40 us; per-file medians 183.3..226.6 us
line                                median_ms     min_ms     max_ms    user_ms     sys_ms
N.direct                              2993.34    2873.66    3107.02       6.06     290.92
N.pool1                               2961.06    2920.30    3107.88       7.04     284.33
N.pool2                               1637.26    1555.87    1724.28      11.52     323.06
N.pool4                               1478.38    1458.56    1502.02      13.89     370.97
N.pool8                               1487.68    1451.40    1492.35      17.31     349.34
N.uring4                              1472.85    1465.07    1493.92       5.74     212.64
N.uring8                              1495.06    1464.92    1512.50       5.71     192.06
N.uring32                             1459.84    1436.19    1496.39       3.10     169.06
S.narrow                              3133.06    3016.24    3196.67      87.40     358.14
S.wide8                               3071.27    3038.07    3158.54      86.35     349.46
C.narrow.default                      3058.12    2994.33    3166.44      84.23     332.49
C.wide8.default                       1463.43    1449.46    1487.15      51.03     202.49
C.narrow.h0                           3091.61    2991.24    3213.13      85.80     356.62
C.narrow.h1                           3045.64    2992.26    3227.07      84.49     346.98
C.narrow.h4                           3076.18    3007.10    3161.55      84.03     353.51
C.wide8.h0                            1478.72    1460.47    1494.94      45.67     211.17
C.wide8.h1                            1479.94    1463.21    1497.83      54.16     193.79
C.wide8.h2                            1472.13    1467.43    1494.96      48.96     199.77
C.wide8.h4                            1490.88    1486.04    1494.20      46.30     207.10
C.wide8.h8                            1479.01    1456.57    1497.86      50.01     200.90
```

**The eight-wide Whitefoot program lands on the hand-written 32-deep io_uring
pipeline**: 1463.43 against 1459.84 ms, a quarter of a per cent apart, with
the ring's own minimum and maximum straddling C's median. It is 2.10 times
faster than the same source built `--no-overlap`, and it beats every thread
pool at every size. This is the first table in this investigation where a
Whitefoot program reaches a native completion path rather than approaching
one, and the reason is visible in the two lines above it: `N.direct` at
2993.34 ms is what this work costs with one read outstanding, and `N.uring32`
at 1459.84 ms is what it costs with thirty-two. The whole distance between
those two numbers is device wait, and eight reads stated consecutively in
Whitefoot source recover all of it.

The 64 KiB table is a *cold-start* table rather than an uncached one, and the
probes say so: the tree was not resident when the table began (per-file
medians 158.9..201.6 us) and was resident when it ended (8.7..9.7 us). At
64 KiB the plan reads 2 GiB over a 512 MiB tree on a host with 16 GB of RAM,
so each line warms the cache as it goes. Every line starts cold all the same,
because `POSIX_FADV_DONTNEED` runs on each of the eight opens and evicts what
the previous line left, and every line covers the identical read schedule; the
self-warming is therefore the same constant in all of them.

```text
== read-heavy 64 KiB, cold start (WF_IO_NOCACHE=1) ==
probe before: 1 of 128 sampled reads at or below 40 us; per-file medians 158.9..201.6 us
probe after:  127 of 128 at or below 40 us; per-file medians 8.7..9.7 us  (label refused)
line                                median_ms     min_ms     max_ms    user_ms     sys_ms
N.direct                              1663.80    1583.81    1719.63      39.52     458.77
N.pool1                               1632.37    1594.96    1787.15      36.52     463.14
N.pool2                               1112.12    1085.11    1316.44      34.38     480.58
N.pool4                               1257.55    1241.68    1296.34      36.66     553.96
N.pool8                               1278.13    1201.83    1285.35      44.86     597.45
N.uring4                              1343.45    1280.84    1366.35      35.51     467.43
N.uring8                              1274.99    1271.51    1300.51      29.93     463.80
N.uring32                             1294.89    1108.55    1305.25      33.75     483.03
S.narrow                              1680.88    1627.11    1784.59      86.60     466.68
S.wide8                               1751.03    1717.21    1851.10      83.52     518.37
C.narrow.default                      1664.13    1597.66    1769.51      82.09     454.74
C.wide8.default                       1228.53    1215.99    1262.12      76.19     506.00
C.narrow.h0                           1652.34    1611.55    1775.53      86.65     451.24
C.narrow.h1                           1635.68    1578.10    1882.25      75.07     444.67
C.narrow.h4                           1643.15    1614.12    1782.87      84.07     443.37
C.wide8.h0                            1258.58    1222.52    1300.79      77.14     513.57
C.wide8.h1                            1285.70    1275.10    1302.49      75.66     494.17
C.wide8.h2                            1280.44    1269.72    1297.00      82.13     484.69
C.wide8.h4                            1282.37    1252.44    1292.18      75.22     502.41
C.wide8.h8                            1296.50    1262.90    1333.53      77.18     527.25
```

C at eight wide is 1.43 times faster than S at eight wide, 1.04 times faster
than an eight-thread pool, 1.04 times faster than an eight-deep io_uring
pipeline, and 1.10 times slower than the best native line in the table, which
is the two-thread pool. The pool curve is the explanation: on four cores, two
threads is this workload's optimum and more threads cost more than they buy.
C's eight-wide shape asks for eight in flight, so it is compared honestly
against the pool at eight and against the ring at eight, and it beats both.

Warm, both windows, on the same runner:

```text
== read-heavy 64 KiB, page cache warm ==       == read-heavy 4 KiB, page cache warm ==
line                     median_ms  sys_ms     line                     median_ms  sys_ms
N.direct                    275.02  240.78     N.direct                     29.74   26.12
N.pool2                     141.13  241.95     N.pool2                      16.27   25.54
N.pool4                      96.42  338.91     N.pool4                      11.06   35.76
N.pool8                      99.00  330.33     N.pool8                      12.95   32.95
N.uring8                    289.00  254.77     N.uring8                     30.21   26.80
N.uring32                   324.71  292.12     N.uring32                    29.98   27.98
S.narrow                    294.03  218.37     S.narrow                     74.05   37.06
S.wide8                     334.53  264.99     S.wide8                      76.17   39.27
C.narrow.default            295.05  217.93     C.narrow.default             74.88   38.44
C.wide8.default             328.38  265.20     C.wide8.default              71.69   34.15
C.wide8.h0                  327.88  258.04     C.wide8.h0                   74.01   38.67
C.wide8.h8                  331.17  261.92     C.wide8.h8                   71.43   37.06
```

With the tree in memory there is no wait to overlap and the ring proves it:
`N.uring32` is no faster than `N.direct` at 4 KiB and 18 per cent slower at
64 KiB. C stays level with S — 1.02 and 1.06 times faster — which is the
correct behaviour for a lowering that has nothing to hide, and every line is
three to six times slower than the best pool because a pool still has four
cores to copy memory with.

The warm 4 KiB column also prices the framework itself. `N.direct` folds
32,768 cached 4 KiB reads in 29.74 ms; `S.narrow`, doing exactly the same
reads in Whitefoot with no overlap lowering at all, takes 74.05 ms. That is
about 1.4 us per read of Whitefoot per-read path, and it is the floor both S
and C stand on.

### macOS runner

The first macOS reading this project has that is not also a reading of an
endpoint-security stack.

```text
== read-heavy 64 KiB, cold start ==            == read-heavy 4 KiB, cold start ==
line                     median_ms  sys_ms     line                     median_ms  sys_ms
N.direct                   2345.41  478.17     N.direct                   1971.16  489.92
N.pool2                    1120.59  507.67     N.pool2                    1100.27  509.47
N.pool4                     853.27  562.11     N.pool4                     808.40  508.87
N.pool8                     772.34  639.97     N.pool8                     532.07  592.59
S.narrow                   2045.43  435.44     S.narrow                   1663.83  437.36
S.wide8                    2108.61  473.90     S.wide8                    1736.79  450.31
C.narrow.default           1952.50  456.61     C.narrow.default           1889.77  442.69
C.wide8.default            1220.68  755.87     C.wide8.default            1100.57  698.82
C.wide8.h0                 1681.74  423.80     C.wide8.h0                 1611.22  447.46
C.wide8.h2                 1252.02  654.21     C.wide8.h2                 1160.21  684.76
C.wide8.h4                 1048.26  709.79     C.wide8.h4                  962.59  664.09
C.wide8.h8                  940.47  975.94     C.wide8.h8                  793.93  903.04

== read-heavy 64 KiB, page cache warm ==       == read-heavy 4 KiB, page cache warm ==
line                     median_ms  sys_ms     line                     median_ms  sys_ms
N.direct                    169.00  131.35     N.direct                     33.10   25.88
N.pool2                      97.04  153.23     N.pool2                      20.18   31.00
N.pool4                      73.05  169.41     N.pool4                      15.36   33.02
N.pool8                      71.08  165.87     N.pool8                      15.18   32.60
S.narrow                    152.57  115.21     S.narrow                     31.20   23.67
S.wide8                     166.31  129.30     S.wide8                      32.88   25.89
C.narrow.default            153.01  115.65     C.narrow.default             31.34   23.83
C.wide8.default             211.58  301.48     C.wide8.default              94.72  129.39
C.wide8.h0                  175.21  129.14     C.wide8.h0                   41.84   26.28
C.wide8.h2                  241.57  310.70     C.wide8.h2                   98.04  109.67
C.wide8.h8                  205.61  299.55     C.wide8.h8                  118.48  202.19
```

Cold, C behaves as it does on Linux: 1.73 and 1.58 times faster than S at
eight wide, and faster still with helpers named explicitly (`C.wide8.h8` is
2.24 and 2.19 times faster than S). It does not reach the native pool here —
1.58 and 2.07 times slower than `N.pool8` at the default helper count, 1.22
and 1.49 times slower at `h8`.

Warm, **C is slower than S on this host**, by 1.27 times at 64 KiB and 2.88
times at 4 KiB, and the system-time column says where it goes: 301.48 against
129.30 ms, and 129.39 against 25.89. `C.wide8.h0` — the same program with the
helper pool switched off — costs 41.84 ms against C's default 94.72 and S's
32.88, so the overhead is the helper handoff, not the completion state
machine. On Linux the same comparison costs nothing: warm 4 KiB `C.wide8`
system time is 34.15 ms against S's 39.27. Whatever this is, it is Darwin's
half of the runtime and not the model.

The macOS spreads are wide where the Linux ones are tight — `C.wide8.h4` cold
at 4 KiB has a maximum of 12395 ms against a median of 962 — and the reason is
the next section.

### What the probes said, and what a label means on each host

```text
table                        probe before          probe after
Linux  64 KiB uncached       confirmed  159..202us refused   8.7..9.7 us
Linux   4 KiB uncached       confirmed  161..206us confirmed  183..227 us
Linux  64 KiB warm           confirmed  7.6..8.2us confirmed  7.4..8.0 us
Linux   4 KiB warm           confirmed  7.3..8.3us confirmed  7.3..8.0 us
macOS  64 KiB uncached       confirmed  76..316 us refused   37.0..55.5 us
macOS   4 KiB uncached       confirmed  43..275 us refused   38..203 us
macOS  64 KiB warm           confirmed  5.5..6.0us confirmed  4.0..5.0 us
macOS   4 KiB warm           confirmed  4.0..5.0us confirmed  4.0..5.0 us
```

Every table began in the state it claims. Three ended somewhere else, and the
two reasons are different.

On Linux the cause is known and bounded: `POSIX_FADV_DONTNEED` evicts at open
and cannot stop a later read populating, so a 2 GiB read schedule over a
512 MiB tree ends with the tree resident. The 4 KiB schedule moves 128 MiB and
does not.

On macOS the cause is outside the guest. `F_NOCACHE` is a mode of the
descriptor, so no read in those tables entered this kernel's unified buffer
cache — and the after-probe still found 37 to 55 us reads where the
before-probe found 76 to 316 and the warm probe finds 4 to 6. Forty
microseconds is neither a memory copy nor an SSD round trip; it is a cache
below the guest, in the hypervisor or the host, which no flag available to a
process inside the virtual machine can bypass. So the macOS cold tables warm
as they run and there is no host call that would stop them.

This is exactly the drift the interleaved schedule was built for. The runner
runs the whole plan as a pass and reverses the order every other pass, so a
tree that gets cheaper over the minutes a table takes gets cheaper for every
line by the same amount instead of rewarding whichever line ran last. It does
not remove the drift; it stops the drift from becoming a ranking.

### The local machine, provisional

The maintainer's macOS laptop, taken before the interleaved schedule existed,
line by line, at a one-minute load average of 2.5, medians of nine. Only the
medians were kept. It is recorded because it is the only reading here from
hardware anyone on this project owns, and it is labelled provisional because
its method is the older one and its host reads the benchmark tree behind the
benchmark's back.

```text
                          64 KiB uncached      64 KiB warm
N.direct                          4378 ms            160 ms
N.pool2                           2439                88
N.pool8                           1211                44
S.narrow                          4450               136
S.wide8                           4447               145
C.narrow.default                  4496               128
C.wide8.default                   1885 (sys 484)     164 (sys 455)
C.wide8.h2                        2602               141
C.wide8.h8                        1886               159
```

It agrees with the runners on the shape of the result — C at eight wide is
2.36 times faster than S at eight wide when the reads reach the device, and
1.13 times slower than it when they do not — and disagrees with them on the
absolute cost of a read: 134 us here against 160 to 200 us on the Linux runner
and 76 to 316 on the macOS one.

### Against the standing bar

The bar this investigation has carried since batch 0084: C at least as fast as
S on every workload, and within 10 per cent of N wherever N is a native
completion path or a fairly sized thread pool.

```text
workload / host                cache      C vs S         C vs N best      C vs N at width 8   bar
read 4 KiB / Linux runner      uncached   2.10x faster   1.00x of uring32 1.02x faster        met, both halves
read 64 KiB / Linux runner     cold       1.43x faster   1.10x slower     1.04x faster        met on S, missed N by 0.5
read 4 KiB / Linux runner      warm       1.06x faster   6.48x slower     --                  met on S, missed N
read 64 KiB / Linux runner     warm       1.02x faster   3.41x slower     --                  met on S, missed N
read 4 KiB / macOS runner      cold       1.58x faster   2.07x slower     2.07x slower        met on S, missed N
read 64 KiB / macOS runner     cold       1.73x faster   1.58x slower     1.58x slower        met on S, missed N
read 4 KiB / macOS runner      warm       2.88x slower   6.24x slower     --                  missed, both halves
read 64 KiB / macOS runner     warm       1.27x slower   2.98x slower     --                  missed, both halves
many files / macOS runner      warm       1.20x slower   2.99x slower     --                  missed, both halves
```

Read the table by its cache column and it says one thing twice.

**Where there are waits, the model works.** Every cold or uncached row has C
faster than S by 1.4 to 2.1 times, and the single row where the reads are
device reads from beginning to end has C level with a hand-written 32-deep
io_uring pipeline. The bar's second half is met there too, and on Linux it is
met at the width the source actually asks for: against a pool of eight and a
ring of eight, C at eight wide is the faster line.

**Where there are no waits, the model costs.** Every warm row has C at best
level with S and at worst 2.88 times slower, and the cost is system time in
the Darwin helper handoff: 5.0 times S's system time on the macOS warm 4 KiB
row, against 0.87 times S's on the Linux one. The same program with helpers
switched off is 2.3 times faster than the default on that row.

The system-time ratio is the cleanest single statement of where the model
stands. On Linux, C at eight wide spends **0.58 times** the system time of S
at eight wide on the uncached 4 KiB table and **0.98 times** on the 64 KiB
one: the overlap does not cost kernel work, it saves it, because eight reads
outstanding is fewer round trips than eight reads in sequence. On macOS the
same ratios are **1.55** and **1.60** cold, and **5.00** warm. Nothing about
the model changes between those two hosts; the adapter does.

### The many-files workload without an endpoint-security stack

The macOS runner also ran the workload batches 0084 and 0086 measured, whose
every previous macOS reading came from a machine that charges 116 us for an
`openat`. Here one open, read, close and fold of a small file costs 17.2 us
end to end (`N.direct`, 141.06 ms for 8192 files).

```text
line                                median_ms     min_ms     max_ms    user_ms     sys_ms
N.direct                               141.06     140.69     150.29      71.12      69.46
N.pool2                                 80.32      79.22      80.78      74.57      83.28
N.pool4                                 58.21      57.56      59.45      77.41      89.58
N.pool8                                 57.95      57.01      58.40      77.49      90.25
S.narrow                               143.75     143.52     146.89      71.86      71.50
S.wide                                 144.10     143.85     144.79      71.73      71.92
S.wide8                                144.28     143.89     145.05      71.70      72.06
C.narrow.default                       144.31     143.61     152.25      71.89      71.89
C.wide.default                         216.89     214.23     218.83     103.22     165.00
C.wide8.default                        173.16     172.17     174.04      93.03     140.50
C.wide.w0.h0                           148.61     148.40     151.67      75.63      72.50
C.wide.w0.h1                           211.99     209.51     226.69      96.40     125.73
C.wide.w0.h2                           222.35     221.60     244.37     101.56     158.69
C.wide.w0.h4                           217.29     215.53     235.04     103.31     167.68
C.wide8.h0                             149.36     148.74     150.58      76.06      72.70
C.wide8.h1                             199.79     198.89     201.63      92.72     129.22
C.wide8.h2                             178.66     177.43     179.51      93.32     136.72
C.wide8.h4                             174.27     173.51     178.10      94.02     143.89
```

Batch 0084 recorded this workload on macOS with C 2.05 times **faster** than
S. On a macOS host without the endpoint-security stack, C is 1.20 times
**slower** at eight wide and 1.51 times slower at four, and 2.99 times slower
than the best pool. The 2.05x was the 116 us `openat`: a wait that large is
worth overlapping whatever the handoff costs, and once it drops to 17 us the
handoff is all that is left. This is the same conclusion batch 0090 reached on
Linux hardware, now reached on macOS, and it retires the last table in this
document that credited the overlap lowering with a win on the many-files
workload.

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

### The Linux runner

`bench-linux-read`, run
[33130875022](https://github.com/mbbill/Whitefoot/actions/runs/33130875022/job/98719847684)
at commit `6ac36126`. Kernel `6.17.0-1022-azure`, `x86_64`, 4 CPUs, Intel Xeon
Platinum 8370C, 16 GiB, tree on the runner's own `ext4` disk (`/dev/sda1`),
`kernel.io_uring_disabled=0`, one-minute load average 0.41 at the start.
Seven recorded passes after two warm-ups.

Both uncached tables were confirmed uncached before they ran: the probe found
every file answering at 159 to 202 us against a 7.6 us page-cache hit, with
one of 128 sampled reads on the wrong side of the threshold.

```text
                  64 KiB uncached                4 KiB uncached
line             median    min    max  sys      median    min    max  sys
N.direct        1663.80 1583.81 1719.63 459    2993.34 2873.66 3107.02 291
N.pool1         1632.37 1594.96 1787.15 463    2961.06 2920.30 3107.88 284
N.pool2         1112.12 1085.11 1316.44 481    1637.26 1555.87 1724.28 323
N.pool4         1257.55 1241.68 1296.34 554    1478.38 1458.56 1502.02 371
N.pool8         1278.13 1201.83 1285.35 597    1487.68 1451.40 1492.35 349
N.uring4        1343.45 1280.84 1366.35 467    1472.85 1465.07 1493.92 213
N.uring8        1274.99 1271.51 1300.51 464    1495.06 1464.92 1512.50 192
N.uring32       1294.89 1108.55 1305.25 483    1459.84 1436.19 1496.39 169
S.narrow        1680.88 1627.11 1784.59 467    3133.06 3016.24 3196.67 358
S.wide8         1751.03 1717.21 1851.10 518    3071.27 3038.07 3158.54 349
C.narrow.default 1664.13 1597.66 1769.51 455   3058.12 2994.33 3166.44 332
C.wide8.default 1228.53 1215.99 1262.12 506    1463.43 1449.46 1487.15 202
C.narrow.h0     1652.34 1611.55 1775.53 451    3091.61 2991.24 3213.13 357
C.narrow.h1     1635.68 1578.10 1882.25 445    3045.64 2992.26 3227.07 347
C.narrow.h4     1643.15 1614.12 1782.87 443    3076.18 3007.10 3161.55 354
C.wide8.h0      1258.58 1222.52 1300.79 514    1478.72 1460.47 1494.94 211
C.wide8.h1      1285.70 1275.10 1302.49 494    1479.94 1463.21 1497.83 194
C.wide8.h2      1280.44 1269.72 1297.00 485    1472.13 1467.43 1494.96 200
C.wide8.h4      1282.37 1252.44 1292.18 502    1490.88 1486.04 1494.20 207
C.wide8.h8      1296.50 1262.90 1333.53 527    1479.01 1456.57 1497.86 201

                  64 KiB warm                    4 KiB warm
line             median    min    max  sys      median    min    max  sys
N.direct         275.02  269.20  277.28 241      29.74  28.71  30.28  26
N.pool1          276.24  272.88  286.55 245      30.36  29.66  31.65  26
N.pool2          141.13  138.20  143.41 242      16.27  15.67  16.50  26
N.pool4           96.42   95.37   98.00 339      11.06  10.66  11.26  36
N.pool8           99.00   96.83  103.20 330      12.95  11.96  13.40  33
N.uring4         284.65  275.67  292.47 251      31.35  30.57  33.31  30
N.uring8         289.00  284.76  297.66 255      30.21  29.85  31.76  27
N.uring32        324.71  316.66  331.43 292      29.98  28.85  31.09  28
S.narrow         294.03  283.08  300.17 218      74.05  73.24  77.12  37
S.wide8          334.53  332.09  342.59 265      76.17  75.80  81.70  39
C.narrow.default 295.05  288.77  302.70 218      74.88  73.30  76.62  38
C.wide8.default  328.38  323.48  338.35 265      71.69  69.22  77.88  34
C.narrow.h0      290.42  280.83  301.20 225      75.06  74.09  83.22  35
C.narrow.h1      291.32  281.63  308.35 221      75.44  74.46  76.02  36
C.narrow.h4      292.30  287.97  308.71 221      75.20  73.52  76.01  37
C.wide8.h0       327.88  321.42  342.97 258      74.01  70.68  78.62  39
C.wide8.h1       328.49  322.42  341.22 256      72.84  70.37  78.65  37
C.wide8.h2       331.12  323.01  337.07 257      71.68  70.57  78.25  34
C.wide8.h4       325.27  320.07  341.14 254      71.49  70.48  78.39  35
C.wide8.h8       331.17  322.95  422.78 262      71.43  70.37  72.47  37
```

On this host `C.*.default` is the ring with no helpers: `bridge.c` starts a
target that has a native completion path at zero, because a ready ring already
carries the transfer without a thread handoff. The pinned lines say that
policy is right here. Every `C.wide8.h1` through `h8` line sits about four per
cent behind `C.wide8.default` in the 64 KiB table and about one per cent
behind it at 4 KiB, and none of them beats it; a helper on this host can only
take work the ring already has. `C.narrow.default` lands on `S.narrow` to
within one per cent in both tables, which is what a program with no stated
width should do — there is nothing for the lowering to overlap.

The probe after the 64 KiB uncached table refused the uncached label, and that
is reported rather than treated as a fault. Linux has no per-descriptor
non-populating mode: `POSIX_FADV_DONTNEED` evicts at the open and nothing
after it, so every line starts from a cold tree — the pre-table probe checks
exactly that — and then warms the tree as it reads. Reading 2 GiB over a
512 MiB tree leaves most of it resident, which is what the second probe saw.
The 4 KiB table reads 128 MiB and its second probe still passed. Every line
pays the same self-warming, because every line traverses the same schedule
through the same policy.

### The macOS runner

`bench-macos-read`, run
[33130875022](https://github.com/mbbill/Whitefoot/actions/runs/33130875022/job/98719847615)
at the same commit. Darwin 23.6.0, macOS 14.8.7, Apple M1 (virtual), 3 CPUs,
7 GiB, APFS. Seven recorded passes after two warm-ups.

This host is noisy in a way the Linux runner is not — its reported one-minute
load average at the start of the job was 26.21, and the minima and maxima
below are three to four times apart on the uncached lines where the Linux
runner's are within five per cent. Read the uncached medians as the shape of
an ordering, not as a measurement to two figures.

```text
                  64 KiB uncached                4 KiB uncached
line             median    min    max  sys      median    min     max  sys
N.direct        2345.41 1677.37 2948.05 478   1971.16 1384.67 2783.78 490
N.pool1         1957.22 1524.92 2969.60 440   1894.80 1369.52 2729.63 469
N.pool2         1120.59  992.46 1795.42 508   1100.27  895.68 1910.56 509
N.pool4          853.27  689.77 1234.53 562    808.40  575.77 1196.95 509
N.pool8          772.34  570.53  891.76 640    532.07  381.54  814.07 593
S.narrow        2045.43 1567.83 3009.86 435   1663.83 1395.20 3311.46 437
S.wide8         2108.61 1517.62 2448.46 474   1736.79 1401.80 3168.64 450
C.narrow.default 1952.50 1629.75 2255.74 457  1889.77 1379.83 3141.18 443
C.wide8.default 1220.68  909.44 1462.66 756   1100.57  799.56 1523.61 699
C.narrow.h0     1692.08 1428.76 2722.92 413   1806.40 1379.28 2383.97 468
C.narrow.h1     1721.31 1441.47 2088.63 414   1684.41 1373.31 2186.81 462
C.narrow.h4     1719.52 1424.33 1986.09 410   1620.28 1379.79 2796.36 454
C.wide8.h0      1681.74 1459.60 2082.95 424   1611.22 1421.07 2656.38 447
C.wide8.h1      2147.41 1458.44 2577.45 683   1473.34 1446.82 2313.73 620
C.wide8.h2      1252.02 1064.98 2057.00 654   1160.21  971.59 2007.93 685
C.wide8.h4      1048.26  759.53 1389.55 710    962.59  705.95 12395.31 664
C.wide8.h8       940.47  621.30 1351.73 976    793.93  516.60 8665.33 903

                  64 KiB warm                    4 KiB warm
line             median    min    max  sys      median   min    max  sys
N.direct         169.00  168.32  174.18 131      33.10  32.91  37.85  26
N.pool1          168.63  168.30  173.52 131      32.97  32.70  36.17  26
N.pool2           97.04   94.50  110.70 153      20.18  20.01  20.43  31
N.pool4           73.05   72.64  104.57 169      15.36  15.20  15.45  33
N.pool8           71.08   70.44  103.24 166      15.18  15.07  15.36  33
S.narrow         152.57  151.97  174.32 115      31.20  31.12  31.47  24
S.wide8          166.31  165.63  189.29 129      32.88  32.71  33.56  26
C.narrow.default 153.01  151.99  174.19 116      31.34  31.22  31.59  24
C.wide8.default  211.58  210.93  217.35 301      94.72  94.11  95.66 129
C.narrow.h0      151.94  151.48  152.79 115      31.30  31.12  31.38  24
C.narrow.h1      152.20  151.41  156.74 115      31.24  31.06  31.43  24
C.narrow.h4      152.25  151.36  169.46 115      31.41  31.07  31.82  24
C.wide8.h0       175.21  173.88  178.65 129      41.84  41.54  42.55  26
C.wide8.h1       274.64  274.42  288.16 261      89.38  88.81  90.93  77
C.wide8.h2       241.57  236.08  242.27 311      98.04  96.00  99.41 110
C.wide8.h4       211.97  211.05  218.06 297      98.50  94.84  99.56 146
C.wide8.h8       205.61  204.54  234.67 300     118.48 113.49 120.11 202
```

Both uncached tables were confirmed uncached before they ran and refused
after, and here the refusal means something the Linux one does not. `F_NOCACHE`
is a mode of the descriptor, so no line in this table can have populated the
page cache. What the after-probe found was not page-cache hits either: this
host answers a resident 64 KiB read in 5.5 to 6.0 us, as the warm probes show,
and the after-probe's per-file medians were 37 to 55 us — an order of
magnitude above a cache hit, and about half what the same files cost before
the table, where the medians ran 75 to 315 us. The blocks are not in this kernel's cache; they
are answered faster by something under it, which on a virtualized runner is
the host's own disk cache. The 40 us threshold was calibrated against a
physical machine where the two populations are 6 us and 134 us, and it does
not separate three populations. So the uncached macOS medians are a table
whose device got faster as it ran, and every line ran through the same
warming, in both directions, because the passes alternate.

### The many-files workload on an ordinary macOS system

The same job also re-ran the workload of batches 0084 and 0086 — open a small
file, read it, close it, 8,192 times — because every previous macOS reading of
it came from a machine whose endpoint-security stack charges 116 us for an
`openat`. Warm cache, seven recorded passes after two warm-ups.

```text
line             median    min    max   user    sys
N.direct         141.06  140.69 150.29  71.12  69.46
N.pool2           80.32   79.22  80.78  74.57  83.28
N.pool4           58.21   57.56  59.45  77.41  89.58
N.pool8           57.95   57.01  58.40  77.49  90.25   best N
N.pool10          58.38   57.23  60.41  77.54  90.82
S.narrow         143.75  143.52 146.89  71.86  71.50
S.wide           144.10  143.85 144.79  71.73  71.92
S.wide8          144.28  143.89 145.05  71.70  72.06
C.narrow.default 144.31  143.61 152.25  71.89  71.89
C.wide.default   216.89  214.23 218.83 103.22 165.00
C.wide8.default  173.16  172.17 174.04  93.03 140.50
C.wide.w0.h0     148.61  148.40 151.67  75.63  72.50
C.wide8.h0       149.36  148.74 150.58  76.06  72.70   best C
C.wide8.h1       199.79  198.89 201.63  92.72 129.22
C.wide8.h2       178.66  177.43 179.51  93.32 136.72
C.wide8.h4       174.27  173.51 178.10  94.02 143.89
C.wide8.h6       178.69  176.66 189.25  96.06 151.50
```

The whole table is one number restated. `N.direct` opens, reads and closes
8,192 files in 141.06 ms — 17.2 us each — against 1138.51 ms, 139 us each, on
the maintainer's M4. The runner is a virtual M1 with three cores against a
physical M4 with ten, and it does this workload eight times faster, because
the operation being measured was never the open: it was the hook on the open.

With that hook gone, the two-times figure goes with it. On the maintainer's
machine `C.wide8` finished 2.07 times faster than `S.wide8`. Here it finishes
**1.20 times slower** — 173.16 against 144.28 — and the best C line, at zero
helpers, is still 3.5 per cent slower than the sequential build. That is the
same sign and nearly the same size as batch 0090's finding on Linux hardware.

The CPU columns say where it goes: `C.wide8.default` spends 140.50 ms of
system time against `S.wide8`'s 72.06 for the same 8,192 opens, because the
default helper policy hands operations to a pool that has nothing to wait for.
`C.wide8.h0` spends 72.70 and lands within 3.5 per cent of S.

### Against the standing bar

The bar this investigation has carried since batch 0084: C at least as fast as
S on every workload, and within 10 per cent of the best native shape at
matched width. `C` is the shipped default; the best helper count is noted
where it differs.

```text
workload / host                       C vs S          C vs N at width 8    bar
read 64 KiB uncached / Linux runner   1.43x faster    1.04x faster         met
read 4 KiB uncached  / Linux runner   2.10x faster    1.02x faster         met
read 64 KiB warm     / Linux runner   1.02x faster    3.32x slower         missed on N
read 4 KiB warm      / Linux runner   1.06x faster    5.54x slower         missed on N
read 64 KiB uncached / macOS runner   1.73x faster    1.58x slower         missed on N
read 4 KiB uncached  / macOS runner   1.58x faster    2.07x slower         missed on N
read 64 KiB warm     / macOS runner   1.27x slower    2.98x slower         missed, both
read 4 KiB warm      / macOS runner   2.88x slower    6.24x slower         missed, both
many files (warm)    / macOS runner   1.20x slower    2.99x slower         missed, both
```

**The uncached Linux rows are the first time this project has met both halves
of the bar.** At matched width the completion build is not within 10 per cent
of the best hand-written native shape; it is faster than it. `C.wide8.default`
finishes 1228.53 ms against `N.pool8`'s 1278.13 and against a raw io_uring
pipeline's 1274.99 at depth 8 and 1294.89 at depth 32, and at 4 KiB it lands
on `N.uring32` to within a fifth of a per cent. Against the same program
compiled with no overlap lowering it is 1.43 and 2.10 times faster. This is
the workload the model was designed for and the first one measured where the
operations genuinely wait.

The one caveat on that row is that `N.pool2` — two threads, not eight — is the
fastest native line in the 64 KiB uncached table at 1112.12 ms, so against the
*best* native shape at any width C is 10.5 per cent behind rather than 4 per
cent ahead. At 4 KiB there is no such gap: `N.uring32` is 1459.84 against C's
1463.43.

**Warm, the lowering costs what it always cost.** With every read served from
memory there is no wait to overlap, and what remains is the submission, the
token and the join. On Linux that is within two per cent either way. On macOS
it is 1.27x at 64 KiB and 2.88x at 4 KiB, and the CPU columns name the cause:
`C.wide8.default` warm spends 301 ms of system time against `S.wide8`'s 129,
because the default policy grows a helper pool for operations that never wait.
`C.wide8.h0` — the same program with the pool switched off — costs 175.21
against S's 166.31, five per cent. The distance is the pool, not the lowering.

**The system-time ratio is the clearest single reading.** On the Linux 4 KiB
uncached table `C.wide8.default` spends 202 ms of system CPU where `S.wide8`
spends 349 and `N.direct` spends 291, and it finishes 2.1 times faster than S:
one ring submission carries eight reads that the sequential build enters the
kernel eight times for. On macOS the ratio runs the other way — 756 against
474 uncached, 301 against 129 warm — because the bounded POSIX adapter has no
ring: it still issues one `pread` per read, now from several threads, so the
helper pool buys overlap by spending system time rather than by saving it.
Whether the overlap is worth that is exactly the uncached/warm split above:
uncached it buys 1.73x, warm it costs 1.27x.

**With waits and without is the whole finding.** Every earlier program-level
table in this document was taken with a warm cache, and the two conclusions it
reached — "overlap is worth about two times on macOS" and "overlap costs three
per cent on Linux hardware" — were both statements about a workload with
almost no waits in it. The first of those does not survive this batch: on a
macOS system without an endpoint-security hook, the many-files workload puts C
1.20 times *slower* than S, the same sign as Linux. What replaces it is the
uncached read table, where the same lowering, on the same host, on a workload
whose time really is spent waiting, reaches and passes the best native shape
at matched width.

### This machine, provisional

Kept because it is the only table taken on hardware anyone here owns, and
labelled provisional because three things about it fall short of the tables
above. It was taken line by line rather than in alternating passes over the
whole plan, so drift across the minutes it took is inside it. It was taken at
a one-minute load average of 2.5 on a machine shared with everything else the
maintainer was doing. And only the medians survive: the run's minima, maxima
and CPU columns were not kept, apart from the two CPU readings noted below.
Nine recorded runs per line, 64 KiB window.

```text
line                 uncached (ms)   warm (ms)
N.direct                     4378         160
N.pool2                      2439          88
N.pool8                      1211          44
S.narrow                     4450         136
S.wide8                      4447         145
C.narrow.default             4496         128
C.wide8.default              1885         164
C.wide8.h2                   2602         141
C.wide8.h8                   1886         159
```

`C.wide8.default` uncached spent 67 ms of user CPU and 484 ms of system CPU
against its 1885 ms of wall time, and warm it spent 455 ms of system CPU
against 164 ms of wall.

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
