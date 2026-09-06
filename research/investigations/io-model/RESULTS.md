# Completion I/O results

Status: measured at the program level on 2026-08-27, on macOS and on Linux
with io_uring, re-measured the same day with the base commit and the branch
interleaved in one plan on a quiet host, then measured on real Linux hardware
and on a clean macOS host through the repository's own continuous integration,
then against a read-dominated workload whose files are opened once and
whose reads are taken past the page cache, and finally re-measured on 2026-08-28
after the Darwin helper path's per-operation cost was rebuilt.

Read the batch-0108 section at the end for the network control test, the first
workload here whose peer decides when an operation completes. Of the file
workloads, read the batch-0096 section last and the batch-0092 section first.
Batch 0092 is the only file workload whose operations genuinely wait, and it
is where the design's own question is answered; batch 0096 is the same workload re-measured
after the Darwin adapter was changed, and it is the current reading of the
standing bar. Read the batch-0090 section for the Linux-hardware result that the
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

The runner tables come from `.github/workflows/io-bench.yml` (until batch 0093
they lived in `io-hosts.yml`): `bench-linux`
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
its own sequential build. Batch 0096 narrowed that 1.20 to 1.02 by removing
per-operation cost from the Darwin adapter, which changes the size of the gap
and not its sign: on this workload the completion build is still not faster
than the sequential one. Neither retirement touches what those sections
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

That "roughly two times" did not survive being taken off this pair of hosts.
Batch 0090 re-ran the Linux half on hardware instead of in a container and
found C 3 per cent *slower* than S; batch 0092 re-ran the macOS half on a
system without this machine's endpoint-security stack and found it 1.20 times
slower. Both rows above were measured on hosts that charge far more for a file
operation than an ordinary one does, and the ratio was reading that charge.
What overlap is worth where the operations genuinely wait is in the batch-0092
read-dominated section.

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
`.github/workflows/io-bench.yml` (then `io-hosts.yml`), which builds the bundle natively — no
container, no bind mount — and runs `linux-bench.sh` with the same protocol,
the same `workload.h`, and the same published checksum.

Host, reported by the job itself: kernel `6.17.0-1022-azure`, `x86_64`, 4
CPUs, AMD EPYC 9V74, `kernel.io_uring_disabled=0`, tree on the runner's own
local disk (`ext4`). Nine recorded runs after two warm-ups, medians with the
observed spread, because a shared runner is noisy and the reading has to carry
its own error bars.

Two runs on two separately provisioned runners — `io-hosts` runs 33114336424
and 33115297530 on the batch-0090 branch — printed side by side below. They
differ in absolute speed by about 21 percent — the second landed on an
NVMe-backed disk, the first on a SATA one — and agree on every ordering and
every ratio. A third runner, also NVMe-backed (run 33118248259), reproduces
the second within three percent on every line — within two on 23 of the 26,
the exceptions being `N.pool2` at +2.05 percent, `N.pool8` at +2.64 and
`S.narrow` at +2.09 — and is not tabulated beyond its headline lines:
`N.direct` 94.68, `N.pool4` 27.14, `N.uring32` 94.75, `S.wide` 112.19,
`C.wide.default` 118.14.

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

No table in this section now carries a cache-state label that was not
measured. Each state is created rather than inherited: the tree is regenerated
through a descriptor that does not populate the cache and flushed, so an
uncached table starts from nothing resident, and a warm table is warmed by a
full sequential pass over every block rather than by whatever the previous
table left behind. The uncached tables run first, because they are the ones
the design question turns on.

`read_baseline probe-uncached` then measures the claim. It times sixteen
positioned reads in each of the eight files immediately before and immediately
after every table and reports what share of them cost more than 40 us — a
threshold that sits in the gap between the two populations the maintainer's
machine keeps far apart, 6 to 20 us from the unified buffer cache and about
134 us from the device. `probe-warm` is the same check inverted.

What follows from a verdict differs by caller, deliberately. `make bench-read`
refuses to print a table the probe did not confirm, which is right on a machine
whose two populations are known. `read-bench.sh`, which produced every runner
table below, prints the verdict on the table's own label line instead, because
a hosted runner's storage is not known in advance and a labelled table teaches
more than an absent one. Three of the tables below were refused their label at
one end, and each says so where it stands.
`research/experiments/io-completion-bench/README.md` describes the machinery,
and the per-file medians are printed beside every table.

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
provisional table below predates that change and was taken line by line; that
is one of the reasons it is labelled provisional.

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

Both tables come from one run of `io-hosts`,
[33130875022](https://github.com/mbbill/Whitefoot/actions/runs/33130875022), at
commit `6ac36126`; each job prints its table to the job summary and uploads it
as an artifact.

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
table, no line's minimum or maximum is more than six per cent from its own
median across seven interleaved passes — seventeen of the twenty lines are
inside four per cent, and the three that are not are `N.pool1` at +4.96,
`N.pool2` at -4.97 and +5.31, and `C.narrow.h1` at +5.96 — and every read in
it reached the device.

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
probes say so. "Cold start" is this document's word for it; the job log calls
every `WF_IO_NOCACHE=1` table uncached, because that is what the run asked
for, and prints the probe verdict beside it so a reader can tell which of the
two it got. The verdicts here: the tree was not resident when the table began
(per-file medians 158.9..201.6 us) and was resident when it ended (8.7 to
9.7 us). At
64 KiB the plan reads 2 GiB over a 512 MiB tree on a host with 16 GB of RAM,
so each line warms the cache as it goes. Every line starts cold all the same,
because `POSIX_FADV_DONTNEED` runs on each of the eight opens and evicts what
the previous line left, and every line covers the identical read schedule; the
self-warming is therefore the same constant in all of them.

```text
== read-heavy 64 KiB, WF_IO_NOCACHE=1, cold start ==
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

### Reproduced on two further pairs of runners

The same workflow ran twice more, at commits `e2e4535d` (run
[33131934257](https://github.com/mbbill/Whitefoot/actions/runs/33131934257))
and `031df30e` (run
[33133182075](https://github.com/mbbill/Whitefoot/actions/runs/33133182075)),
each time on separately provisioned hosts: the second Linux job landed on an
AMD EPYC 7763 rather than the Intel Xeon 8370C above. Different silicon, and
every ordering holds.

```text
                                  run 1        run 2         run 3
Linux 64 KiB cold   C vs S      1.43x faster  1.58x faster  1.89x faster
                    C vs pool8  1.04x faster  1.00x of it   1.06x slower
Linux  4 KiB unc.   C vs S      2.10x faster  2.61x faster  3.29x faster
                    C vs uring32 1.00x of it  1.01x slower  1.00x of it
Linux 64 KiB warm   C vs S      1.02x faster  1.04x faster  1.01x faster
Linux  4 KiB warm   C vs S      1.06x faster  1.08x faster  1.12x faster
macOS 64 KiB cold   C vs S      1.73x faster  1.65x faster  1.64x faster
macOS  4 KiB cold   C vs S      1.58x faster  1.81x faster  1.81x faster
macOS 64 KiB warm   C vs S      1.27x slower  1.24x slower  1.28x slower
macOS  4 KiB warm   C vs S      2.88x slower  2.73x slower  2.90x slower
many files / macOS  C vs S      1.20x slower  1.17x slower  1.21x slower
                    N.direct    17.2 us/file  17.9 us/file  17.2 us/file
```

The C-against-S ratio grows from run to run, and the absolute numbers say why:

```text
Linux uncached, C.wide8.default   run 1     run 2     run 3    spread
  64 KiB                        1228.53   1261.38   1339.04     9.0%
   4 KiB                        1463.43   1471.43   1482.16     1.3%
Linux uncached, N.pool8
  64 KiB                        1278.13   1265.51   1267.32     1.0%
   4 KiB                        1487.68   1479.46   1484.21     0.6%
Linux uncached, S.wide8
  64 KiB                        1751.03   1998.54   2536.27    44.8%
   4 KiB                        3071.27   3834.41   4875.91    58.8%
```

**The completion build's cost is pinned to the native floor across three
different machines; the sequential build's is not.** C and the eight-thread
pool land within one to nine per cent of themselves on three separately
provisioned runners, while the same source built `--no-overlap` swings by 45
and 59 per cent. The mechanism is the one the design claims: a program with
one read outstanding pays 32,768 times whatever this runner's per-read latency
happens to be, and these runners' latencies differ by more than half. A
program with eight outstanding pays what the device will deliver, which varies
far less. Overlap is not only faster here; it is what makes the program's cost
a property of the storage rather than of the queue in front of it.

Two things in the later runs are worth more than the agreement.

Their macOS uncached probes refused the label **before** the tables ran, not
only after — run 3 refused at both ends of both tables — and the script
printed the refusal on the label line and went on: "treat the label as a claim
about what was asked for, not about what was measured". That is the behaviour
the runner jobs were built for, and it means the macOS cold rows above should
be read as what those runs say they are: a device that was already partly
answering from somewhere other than itself. The macOS ordering holds anyway,
on all three runs, at every window.

And the Linux `N.uring32` warm line moved from 324.71 ms in run 1 to 942.12 in
run 2 and 934.39 in run 3, with about 900 ms of system time in each. A 32-deep
ring over a warm page cache is doing nothing but paying for submissions, and
two of the three runners charged three times as much for them as the first.
It is the one line whose readings do not agree, and it is a native baseline
rather than a Whitefoot line.

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

## Darwin helper-path cost, batch 0096 (2026-08-28)

Batch 0092 left one result standing on macOS: where operations wait the
completion model works, and where they do not it costs. This section is the
re-measurement after batch 0096 rebuilt the Darwin helper path's
per-operation cost. It is the same workload, the same script and the same
runner label as the batch-0092 macOS section above, which is the **before**
for every number here.

- **before**: batch-0092 macOS-runner section above.
- **after**: run
  [33155821397](https://github.com/mbbill/Whitefoot/actions/runs/33155821397),
  commit `266acf4f`, branch `batch/0096-darwin-handoff`.
- **hosts**: `bench-macos-read` on `macos-14` — Darwin 23.6.0, macOS 14.8.7,
  Apple M1 (Virtual), 3 CPUs, 7516192768 B, load 5.29 at start;
  `bench-linux-read` on `ubuntu-24.04`.
- **passes**: two unrecorded warm-up passes and then seven recorded ones per
  table, interleaved, order reversed every other pass, medians of the seven
  reported. (`io-bench.yml` sets `ROUNDS: "7"`, `WARMUP: "2"` for both
  read-bench jobs. The stage-attribution table in
  `docs/done/0096-darwin-handoff.md` says nine, and that is a different count
  and correct for it: the trace prints on the warm-up passes too.) Every
  table's cache label is probed immediately before and after it and the
  verdict is printed beside it.

The two draws are separate draws of the `macos-14` label, so **ratios within a
run are the evidence and absolute milliseconds across runs are not.** They
happen to be well matched on the warm and many-files halves — `N.direct` warm
4 KiB 33.10 before against 33.04 after, `N.pool8` many-files 57.95 against
58.10 — and not matched on the cold halves, where this draw is faster on every
line including the native baselines.

### Every io-bench draw on this branch

Three rounds of this record miscounted its own draws — "the only", "four of",
"three of the five" — every time because a run had been left out of a count
taken in prose. This table is the repair, and the prose below no longer
counts: it cites rows.

It is not a selection. It is every `io-bench` run on
`batch/0096-darwin-handoff`, enumerated with

```text
gh run list --branch batch/0096-darwin-handoff --workflow io-bench.yml \
  --limit 50 --json databaseId,headSha,conclusion,createdAt
```

which answers nine, with every artifact of all nine downloaded and parsed.
Nine runs against ten tables apiece — `bench-linux` (many files),
`bench-linux-read` (uncached and warm at 64 KiB and 4 KiB), and
`bench-macos-read`, whose artifact carries a many-files half as well as the
same four read tables — would be ninety rows; two runs were cancelled part way
and produced eight and seven tables instead of ten, so there are **85 rows,
one per (run, job, table)**. The cancelled runs' completed tables are kept and
their run ids marked `*`: a cancelled `bench-linux-read` job does not make its
`bench-macos-read` sibling's tables less real, and dropping them is exactly the
omission this table exists to prevent.

`S.wide8` is the eight-wide program compiled `--no-overlap`; `C.wide8` is
`C.wide8.default`, the same source on the completion path with the shipped
helper policy; both are medians in milliseconds over that table's recorded
interleaved passes. `C/S` is the ratio the warm and many-files bars read,
`C/N8` the one the cold bar reads, and `fastN`/`C/fN` name the *fastest* native
line in that table and compare against it — so a `C/fN` below 1, and nothing
else, is a row where the Whitefoot program beat every native line. `label` is
that table's own cache probe before and after it, `conf` confirmed and `ref`
refused. `proc` is `E` for AMD EPYC, `X` for Intel Xeon Platinum, `M1v` for
Apple M1 (Virtual); every Linux runner in the table reports 4 CPUs and every
macOS runner 3. `load` is the one-minute load average the job read at its
start — the `bench-linux` host record prints none.

Ratios are comparable within a row. Milliseconds are not comparable across
rows, because every row is a separate draw of a hosted runner label.

```text
run          commit   proc   disk   load label      S.wide8  C.wide8    C/S  N.pool8   C/N8 fastN   fastN_ms   C/fN

-- bench-linux-read, uncached 64 KiB  (9 rows)
33149563172  caa66bad X8370C sda1   0.29 conf/ref   1766.60  1224.38 0.6931  1281.05 0.9558 pool2    1105.50 1.1075
33150416900* 34ac1ae2 E7763  sda1   0.38 conf/ref   1991.67  1244.17 0.6247  1277.80 0.9737 pool2    1142.59 1.0889
33151353052  4a748d6e E7763  sda1   0.30 conf/ref   1956.06  1263.14 0.6458  1270.42 0.9943 pool2    1153.59 1.0950
33153717709  96bb4778 E7763  sda1   1.95 conf/ref   2180.75  1281.59 0.5877  1272.73 1.0070 pool2    1186.10 1.0805
33155045849* 135abdf2 E9V74  nvme   0.33 conf/ref   2127.72  1364.23 0.6412  1341.81 1.0167 pool4    1226.53 1.1123
33155821397  266acf4f X8573C nvme   0.48 conf/ref   4628.62  3413.10 0.7374  1348.85 2.5304 pool8    1348.85 2.5304
33158144391  72e98cba E9V45  nvme   0.97 conf/ref   2269.59  1451.01 0.6393  1224.54 1.1849 pool8    1224.54 1.1849
33165141309  a06c53f9 E9V74  nvme   0.50 conf/ref   1587.44  1213.14 0.7642  1275.99 0.9507 pool2    1124.36 1.0790
33172323795  261070c8 X8370C sda1   0.56 conf/ref   1863.93  1235.62 0.6629  1275.88 0.9684 pool2    1115.67 1.1075

-- bench-linux-read, uncached 4 KiB  (8 rows)
33149563172  caa66bad X8370C sda1   0.29 conf/conf  3075.32  1455.21 0.4732  1488.00 0.9780 uring32  1456.44 0.9992
33150416900* 34ac1ae2 E7763  sda1   0.38 conf/conf  3747.70  1470.94 0.3925  1474.69 0.9975 uring32  1466.31 1.0032
33151353052  4a748d6e E7763  sda1   0.30 conf/conf  4496.24  1482.81 0.3298  1486.45 0.9976 uring32  1458.48 1.0167
33153717709  96bb4778 E7763  sda1   1.95 conf/conf  4227.07  1479.87 0.3501  1479.56 1.0002 uring32  1469.81 1.0068
33155821397  266acf4f X8573C nvme   0.48 conf/conf  8973.99  1514.79 0.1688  1435.03 1.0556 uring32  1268.13 1.1945
33158144391  72e98cba E9V45  nvme   0.97 conf/conf  5085.89  1249.62 0.2457  1474.52 0.8475 uring32  1457.10 0.8576
33165141309  a06c53f9 E9V74  nvme   0.50 conf/conf  4108.74  1216.03 0.2960  1482.48 0.8203 uring32  1448.85 0.8393
33172323795  261070c8 X8370C sda1   0.56 conf/conf  3469.10  1465.26 0.4224  1481.78 0.9889 uring32  1441.72 1.0163

-- bench-linux-read, warm 64 KiB  (7 rows)
33149563172  caa66bad X8370C sda1   0.29 conf/conf   343.24   343.12 0.9997   100.94 3.3992 pool4      97.76 3.5098
33151353052  4a748d6e E7763  sda1   0.30 conf/conf   285.81   289.01 1.0112    81.85 3.5310 pool4      78.29 3.6915
33153717709  96bb4778 E7763  sda1   1.95 conf/conf   289.92   285.27 0.9840    80.56 3.5411 pool8      80.56 3.5411
33155821397  266acf4f X8573C nvme   0.48 conf/conf   284.31   291.74 1.0261    80.72 3.6142 pool4      79.77 3.6573
33158144391  72e98cba E9V45  nvme   0.97 conf/conf   227.61   229.38 1.0078    74.34 3.0856 pool4      72.42 3.1674
33165141309  a06c53f9 E9V74  nvme   0.50 conf/conf   280.52   283.30 1.0099    81.55 3.4739 pool8      81.55 3.4739
33172323795  261070c8 X8370C sda1   0.56 conf/conf   327.23   332.08 1.0148   103.01 3.2238 pool4      98.95 3.3560

-- bench-linux-read, warm 4 KiB  (7 rows)
33149563172  caa66bad X8370C sda1   0.29 conf/conf    75.31    69.82 0.9271    13.54 5.1566 pool4      10.94 6.3821
33151353052  4a748d6e E7763  sda1   0.30 conf/conf    81.02    81.05 1.0004    15.25 5.3148 pool4      12.35 6.5628
33153717709  96bb4778 E7763  sda1   1.95 conf/conf    83.28    78.78 0.9460    15.49 5.0859 pool4      12.09 6.5161
33155821397  266acf4f X8573C nvme   0.48 conf/conf    50.89    53.69 1.0550     8.51 6.3090 pool4       8.16 6.5797
33158144391  72e98cba E9V45  nvme   0.97 conf/conf    61.51    63.98 1.0402    11.26 5.6821 pool4      10.63 6.0188
33165141309  a06c53f9 E9V74  nvme   0.50 conf/conf    72.82    72.00 0.9887    14.88 4.8387 pool4      12.44 5.7878
33172323795  261070c8 X8370C sda1   0.56 conf/conf    65.87    65.86 0.9998    12.72 5.1777 pool4      11.18 5.8909

-- bench-linux, many files  (9 rows)
33149563172  caa66bad E7763  sda1      - -           123.91   131.09 1.0579    33.66 3.8945 pool4      31.09 4.2165
33150416900* 34ac1ae2 E9V74  sda1      - -           142.25   147.70 1.0383    40.75 3.6245 pool4      34.21 4.3175
33151353052  4a748d6e E7763  sda1      - -           122.60   131.14 1.0697    35.42 3.7024 pool4      31.09 4.2181
33153717709  96bb4778 E7763  sda1      - -           122.83   130.17 1.0598    34.04 3.8240 pool4      30.92 4.2099
33155045849* 135abdf2 E7763  sda1      - -           121.93   128.64 1.0550    34.36 3.7439 pool4      30.84 4.1712
33155821397  266acf4f E7763  sda1      - -           122.49   129.55 1.0576    35.43 3.6565 pool4      30.84 4.2007
33158144391  72e98cba E7763  sda1      - -           121.61   128.91 1.0600    32.65 3.9482 pool4      30.35 4.2474
33165141309  a06c53f9 E7763  sda1      - -           120.94   128.73 1.0644    34.67 3.7130 pool4      29.76 4.3256
33172323795  261070c8 E9V45  nvme      - -            96.04   100.85 1.0501    26.92 3.7463 pool4      24.18 4.1708

-- bench-macos-read, uncached 64 KiB  (9 rows)
33149563172  caa66bad M1v    disk3  0.47 ref/conf   1615.36   938.79 0.5812   433.57 2.1653 pool8     433.57 2.1653
33150416900* 34ac1ae2 M1v    disk3 13.88 ref/conf   2124.56  1099.83 0.5177   752.66 1.4613 pool8     752.66 1.4613
33151353052  4a748d6e M1v    disk3  4.00 conf/ref   1784.93   897.95 0.5031   523.24 1.7161 pool8     523.24 1.7161
33153717709  96bb4778 M1v    disk3  1.10 conf/conf  1754.70   817.47 0.4659   808.79 1.0107 pool8     808.79 1.0107
33155045849* 135abdf2 M1v    disk3  0.63 ref/conf   1639.47   581.13 0.3545   429.81 1.3521 pool8     429.81 1.3521
33155821397  266acf4f M1v    disk3  5.29 ref/conf   1487.24   591.82 0.3979   424.58 1.3939 pool8     424.58 1.3939
33158144391  72e98cba M1v    disk3 10.74 ref/ref    1547.00   609.97 0.3943   555.63 1.0978 pool8     555.63 1.0978
33165141309  a06c53f9 M1v    disk3  8.81 ref/ref    1500.48   565.31 0.3768   434.33 1.3016 pool8     434.33 1.3016
33172323795  261070c8 M1v    disk3  5.60 conf/conf  2381.05  1150.39 0.4831   779.08 1.4766 pool8     779.08 1.4766

-- bench-macos-read, uncached 4 KiB  (9 rows)
33149563172  caa66bad M1v    disk3  0.47 ref/ref    1416.82   807.70 0.5701   383.68 2.1051 pool8     383.68 2.1051
33150416900* 34ac1ae2 M1v    disk3 13.88 ref/ref    1833.91  1165.03 0.6353   553.63 2.1043 pool8     553.63 2.1043
33151353052  4a748d6e M1v    disk3  4.00 conf/conf  1820.21   690.79 0.3795   486.70 1.4193 pool8     486.70 1.4193
33153717709  96bb4778 M1v    disk3  1.10 conf/ref   1714.06   675.60 0.3942   439.22 1.5382 pool8     439.22 1.5382
33155045849* 135abdf2 M1v    disk3  0.63 ref/ref    1812.50   545.98 0.3012   609.28 0.8961 pool8     609.28 0.8961
33155821397  266acf4f M1v    disk3  5.29 ref/conf   1392.83   489.75 0.3516   381.86 1.2825 pool8     381.86 1.2825
33158144391  72e98cba M1v    disk3 10.74 ref/ref    1672.36   960.86 0.5746   587.85 1.6345 pool8     587.85 1.6345
33165141309  a06c53f9 M1v    disk3  8.81 ref/ref    1428.18   490.57 0.3435   399.02 1.2294 pool8     399.02 1.2294
33172323795  261070c8 M1v    disk3  5.60 conf/conf  2172.94  1058.71 0.4872   679.86 1.5572 pool8     679.86 1.5572

-- bench-macos-read, warm 64 KiB  (9 rows)
33149563172  caa66bad M1v    disk3  0.47 conf/conf   167.84   214.64 1.2788    72.33 2.9675 pool8      72.33 2.9675
33150416900* 34ac1ae2 M1v    disk3 13.88 conf/conf   187.46   246.17 1.3132    78.44 3.1383 pool8      78.44 3.1383
33151353052  4a748d6e M1v    disk3  4.00 conf/conf   194.01   205.37 1.0586    86.22 2.3819 pool8      86.22 2.3819
33153717709  96bb4778 M1v    disk3  1.10 conf/conf   185.75   177.91 0.9578    77.45 2.2971 pool8      77.45 2.2971
33155045849* 135abdf2 M1v    disk3  0.63 conf/conf   166.61   167.61 1.0060    72.90 2.2992 pool8      72.90 2.2992
33155821397  266acf4f M1v    disk3  5.29 conf/conf   172.94   173.97 1.0060    80.13 2.1711 pool8      80.13 2.1711
33158144391  72e98cba M1v    disk3 10.74 conf/conf   167.56   167.92 1.0021    71.21 2.3581 pool8      71.21 2.3581
33165141309  a06c53f9 M1v    disk3  8.81 conf/conf   167.06   168.48 1.0085    71.58 2.3537 pool8      71.58 2.3537
33172323795  261070c8 M1v    disk3  5.60 conf/conf   199.10   196.90 0.9890    88.81 2.2171 pool8      88.81 2.2171

-- bench-macos-read, warm 4 KiB  (9 rows)
33149563172  caa66bad M1v    disk3  0.47 conf/conf    32.80    96.10 2.9299    15.20 6.3224 pool8      15.20 6.3224
33150416900* 34ac1ae2 M1v    disk3 13.88 conf/conf    37.44   113.93 3.0430    16.66 6.8385 pool8      16.66 6.8385
33151353052  4a748d6e M1v    disk3  4.00 conf/conf    44.11    52.72 1.1952    22.93 2.2992 pool8      22.93 2.2992
33153717709  96bb4778 M1v    disk3  1.10 conf/conf    37.61    38.24 1.0168    16.36 2.3374 pool4      16.23 2.3561
33155045849* 135abdf2 M1v    disk3  0.63 conf/conf    32.64    33.58 1.0288    15.09 2.2253 pool8      15.09 2.2253
33155821397  266acf4f M1v    disk3  5.29 conf/conf    32.65    33.57 1.0282    15.03 2.2335 pool8      15.03 2.2335
33158144391  72e98cba M1v    disk3 10.74 conf/conf    32.92    33.77 1.0258    15.14 2.2305 pool8      15.14 2.2305
33165141309  a06c53f9 M1v    disk3  8.81 conf/conf    32.66    33.67 1.0309    15.24 2.2093 pool8      15.24 2.2093
33172323795  261070c8 M1v    disk3  5.60 conf/conf    32.99    33.91 1.0279    15.25 2.2236 pool8      15.25 2.2236

-- bench-macos-read (many-files half), many files  (9 rows)
33149563172  caa66bad M1v    disk3  0.47 -           144.06   176.08 1.2223    58.42 3.0140 pool6      57.99 3.0364
33150416900* 34ac1ae2 M1v    disk3 13.88 -           160.03   229.84 1.4362    75.33 3.0511 pool10     74.78 3.0735
33151353052  4a748d6e M1v    disk3  4.00 -           174.24   183.43 1.0527    85.06 2.1565 pool10     75.68 2.4238
33153717709  96bb4778 M1v    disk3  1.10 -           155.18   164.10 1.0575    59.86 2.7414 pool8      59.86 2.7414
33155045849* 135abdf2 M1v    disk3  0.63 -           145.11   147.53 1.0167    58.84 2.5073 pool6      57.91 2.5476
33155821397  266acf4f M1v    disk3  5.29 -           145.01   148.23 1.0222    58.10 2.5513 pool6      57.68 2.5699
33158144391  72e98cba M1v    disk3 10.74 -           144.65   147.41 1.0191    58.34 2.5267 pool6      57.33 2.5713
33165141309  a06c53f9 M1v    disk3  8.81 -           145.44   147.54 1.0144    58.44 2.5246 pool6      58.05 2.5416
33172323795  261070c8 M1v    disk3  5.60 -           144.37   146.59 1.0154    58.07 2.5244 pool6      57.39 2.5543
```

The commits, in branch order, and what each one is:

```text
caa66bad  the merge base: the runtime this batch started from
34ac1ae2  the same runtime under the WF_IO_TRACE stage instrumentation
4a748d6e  the Darwin per-operation repair
96bb4778  a positioned read is left where it was stated when nothing waits
135abdf2  the stage instrumentation removed
266acf4f  the drain hint removed -- the runtime the before/after tables read
72e98cba  a record commit; its only source change is a comment in bridge.c
a06c53f9  the correctness follow-up: named-drain generation, atomic readiness,
          helper cap bounded by its storage, clock guard, shutdown ordering
261070c8  this record's repair round -- and, since everything after it on the
          branch changes only these two documents, the tip's runtime as well
```

What the table says without anyone counting in prose:

- **The cache labels are uniform everywhere except the macOS uncached
  tables.** All nine `bench-linux-read` uncached 64 KiB tables are
  `conf/ref` — confirmed before, refused after — so every one of them is a
  cold-start table rather than an uncached one. All eight uncached 4 KiB
  tables are `conf/conf`. So are all fourteen Linux warm tables and all
  eighteen macOS warm tables. The macOS uncached tables are the only ones that
  disagree with each other, and only `261070c8`'s draw confirms the label at
  both ends of *both* of them.
- **Four of the 85 rows have `C/fN` below 1.** Three are `bench-linux-read`
  uncached 4 KiB tables — `caa66bad` at 0.9992, `72e98cba` at 0.8576 and
  `a06c53f9` at 0.8393 — and the fourth is `135abdf2`'s macOS uncached 4 KiB
  table at 0.8961, in a cancelled run and on a table whose probe refused the
  uncached label at both ends. `caa66bad`'s margin is 1.23 ms in 1456, which
  is a tie by any honest reading of a hosted runner.
- **The macOS warm 4 KiB `C/S` column is where this batch's work shows.** It
  reads 2.9299 on the merge base and 3.0430 on the traced build of the same
  runtime, then 1.1952, 1.0168, 1.0288, 1.0282, 1.0258, 1.0309, 1.0279 — the
  repair landing at `4a748d6e` and holding across six later draws.
- **The Linux warm columns move a few points between draws and do not settle.**
  Over the seven draws that reached them, warm 64 KiB `C/S` spans 0.9840 to
  1.0261 and warm 4 KiB spans 0.9271 to 1.0550, with no ordering by processor.
- **Linux many-files `C/S` is between 1.0383 and 1.0697 on all nine draws**,
  seven of them on the same EPYC 7763.

### macOS runner, read-heavy

```text
                          before (0092)              after (33155821397)
line                cold64  cold4  warm64 warm4  cold64  cold4 warm64  warm4
N.direct           2345.41 1971.16 169.00 33.10 1472.85 1372.47 176.24 33.04
N.pool2            1120.59 1100.27  97.04 20.18  928.74  883.28 101.63 20.11
N.pool4             853.27  808.40  73.05 15.36  627.14  572.90  81.75 15.21
N.pool8             772.34  532.07  71.08 15.18  424.58  381.86  80.13 15.03
S.narrow           2045.43 1663.83 152.57 31.20 1488.30 1370.55 159.86 31.30
S.wide8            2108.61 1736.79 166.31 32.88 1487.24 1392.83 172.94 32.65
C.narrow.default   1952.50 1889.77 153.01 31.34 1516.75 1382.75 160.51 31.64
C.wide8.default    1220.68 1100.57 211.58 94.72  591.82  489.75 173.97 33.57
C.wide8.h0         1681.74 1611.22 175.21 41.84 1452.09 1432.86 180.33 40.57
C.wide8.h2         1252.02 1160.21 241.57 98.04 1054.36  971.97 246.49 75.37
C.wide8.h4         1048.26  962.59 211.97 98.50  746.67  658.61 219.11  71.25
C.wide8.h8          940.47  793.93 205.61 118.48 585.05  478.59 224.20 83.33
```

Two cells of the `before` column are not findable in the 0092 section this
column names: its macOS warm tables list no `C.wide8.h4` row at all. The
211.97 and 98.50 here are read from that run's own artifact — run
[33130875022](https://github.com/mbbill/Whitefoot/actions/runs/33130875022),
`bench-macos-read`, which does print the row — rather than from the prose
above, and they are the only two cells of this table for which that is true.

**Cache labels on the after run: each cold table's probe refused the uncached
label before the table ran and confirmed it after.** Both warm tables were
confirmed warm at both ends (0.0 per cent of sampled reads at or above 40 us;
the four probes report per-file medians of 4.0..5.0, 4.0..5.0, 4.0..5.0 and
5.0..6.0 us, so 4.0 to 6.0 across the run). The job's own words for the 64 KiB
cold table:

```text
read_baseline: refusing the uncached label -- 93 of 128 sampled reads (72.7%)
  were at or below 40.0 us, past the stated 10.0% bound; per-file medians
  36.5..44.0 us
probe: uncached confirmed -- 4 of 128 sampled reads (3.1%) were at or below
  40.0 us, within the stated 10.0% bound; per-file medians 47.5..51.0 us
table: 64 KiB uncached -- probe before the table: refused; probe after it:
  confirmed
table: 64 KiB uncached -- the label above is NOT confirmed: the probe refused
  it before the table ran, so read the per-file medians printed above and
  treat the label as a claim about what was asked for, not about what was
  measured
```

and for the 4 KiB one, the same four lines with 120 of 128 (93.8 per cent)
refused before and 3 of 128 (2.3 per cent) confirmed after.

That is the **opposite** direction from the 0092 macOS tables, whose probes
confirmed the label before each cold table and refused it after
(`probe before the table: confirmed; probe after it: refused`) — there the
tree started non-resident and each line warmed the cache as it read, here it
started resident and ended non-resident. So the cold tables of this draw are
neither uncached nor cold-start: each line is a mixture whose composition
depends on where in the table it ran, and the interleaved schedule does not
cancel that, because it reverses order between passes rather than restoring
the cache between them.

The batch-0092 section already recorded this happening — "Their macOS uncached
probes refused the label **before** the tables ran, not only after" — on the
runs it reproduced against; this section had it the other way round and is
corrected here.

What that costs is stated with the bar below rather than hidden here: it does
not touch the warm or many-files halves, and it means this draw's cold rows
are not a reading of a cold bar. The cold grades are taken from a later draw
that is one — run 33172323795, which confirms the label at both ends of both
cold tables — and it grades them `no`.

### macOS runner, many files

```text
line                 before (0092)   after (33155821397)
N.direct                    141.06                141.84
N.pool8                      57.95                 58.10
S.narrow                    143.75                144.48
S.wide8                     144.28                145.01
C.narrow.default            144.31                144.83
C.wide.default              216.89                147.27
C.wide8.default             173.16                148.23
C.wide8.h0                  149.36                149.42
C.wide8.h4                  174.27                173.62
```

### Against the standing bar

The bar: warm `C.wide8` not slower than `S.wide8`; cold `C.wide8` within ten
per cent of `N.pool8`; many-files `C` not slower than `S`; Linux must not
regress.

```text
row                       before   after   bar          met         before is
warm 64 KiB  C/S          1.27x    1.006x  <= 1.00x     0.6% over   0092 macOS
warm  4 KiB  C/S          2.88x    1.028x  <= 1.00x     2.8% over   0092 macOS
cold 64 KiB  C/N.pool8    1.58x    1.394x  <= 1.10x     no          0092 macOS
cold  4 KiB  C/N.pool8    2.07x    1.283x  <= 1.10x     no          0092 macOS
many-files   C/S          1.20x    1.022x  <= 1.00x     2.2% over   0092 macOS
Linux warm 64 KiB C/S     0.98x    1.026x  no regress   unresolved  0092 Linux
Linux warm  4 KiB C/S     0.94x    1.055x  no regress   unresolved  0092 Linux
Linux many   C/S          1.041x   1.058x  no regress   unresolved  0090 Linux
                          1.045x
```

The `before is` column is there because the five macOS rows and the three
Linux rows do not share a baseline. The Linux read rows are against the
batch-0092 Linux-runner section; the Linux many-files row has no 0092 reading
at all and is against batch 0090's **two** draws of that job, 1.041 and 1.045.

The two warm rows and the many-files row land within three per cent of a bar
they missed by 27, 188 and 20 per cent, and none is met on a strict reading.
`C.wide8.h0` says where the residue is: the same program on the completion
path with the pool pinned off — and therefore never declined, because a
written `WF_IO_HELPERS` pins the route as well as the count — costs 40.57 ms
warm at 4 KiB against S's 32.65. That 24 per cent is what the machinery
charges an operation with nothing to overlap. In milliseconds over `S.wide8`
that is 7.92; with the policy free to decline it is 0.92, so 88 per cent of the
charge is gone and what remains is the operations the policy does not decline,
which are the opens and closes.

**Both cold rows are graded `no`, and the draw that reads them arrived last.**
For most of this batch they had no grade: this draw's cold tables are the
mixture the probes described above, and a table whose own runner refused its
uncached label before it ran cannot grade a bar about cold reads. On that
mixture C is 1.394 and 1.283 times `N.pool8`, which is a statement about what
was measured and not a grade. Pushing this batch's repair then ran `io-bench`
at `261070c8`, whose `bench-macos-read` job is the only one on this branch to
confirm the uncached label *before and after* both cold tables. On it the two
rows read 1.477 and 1.557 against a bar of 1.10 — a miss, and a wider one than
the unlabelled mixture.

Not five draws, nine. Every `bench-macos-read` cold table on this branch, in
branch order, with its own probe verdicts, its own `C/N.pool8`, and the
min..max of both lines that ratio is taken from; `*` marks a run cancelled
later, as in the draw table above:

```text
-- uncached 64 KiB  (9 rows)
run          commit   label      C.wide8  N.pool8  C/N8   N.pool8 min..max     C.wide8 min..max
33149563172  caa66bad ref/conf    938.79   433.57 2.165   428.18..435.96     916.50..2968.47
33150416900* 34ac1ae2 ref/conf   1099.83   752.66 1.461   430.94..1104.90    903.81..1721.45
33151353052  4a748d6e conf/ref    897.95   523.24 1.716   495.10..1052.44    676.63..1749.48
33153717709  96bb4778 conf/conf   817.47   808.79 1.011   445.55..898.44     644.26..8252.57
33155045849* 135abdf2 ref/conf    581.13   429.81 1.352   425.23..487.68     576.47..665.93
33155821397  266acf4f ref/conf    591.82   424.58 1.394   417.49..453.58     558.77..644.12
33158144391  72e98cba ref/ref     609.97   555.63 1.098   416.53..1075.57    557.78..1176.62
33165141309  a06c53f9 ref/ref     565.31   434.33 1.302   427.78..447.04     553.98..596.73
33172323795  261070c8 conf/conf  1150.39   779.08 1.477   584.73..991.44    1044.77..14961.31

-- uncached 4 KiB  (9 rows)
run          commit   label      C.wide8  N.pool8  C/N8   N.pool8 min..max     C.wide8 min..max
33149563172  caa66bad ref/ref     807.70   383.68 2.105   379.88..387.60     806.44..826.43
33150416900* 34ac1ae2 ref/ref    1165.03   553.63 2.104   414.62..682.65     823.29..1450.92
33151353052  4a748d6e conf/conf   690.79   486.70 1.419   419.97..705.92     552.35..1142.65
33153717709  96bb4778 conf/ref    675.60   439.22 1.538   374.18..727.81     480.67..2181.79
33155045849* 135abdf2 ref/ref     545.98   609.28 0.896   380.07..797.56     488.81..15724.38
33155821397  266acf4f ref/conf    489.75   381.86 1.283   379.97..383.57     469.00..508.36
33158144391  72e98cba ref/ref     960.86   587.85 1.635   380.15..764.48     494.70..26351.72
33165141309  a06c53f9 ref/ref     490.57   399.02 1.229   379.22..413.11     487.42..4996.77
33172323795  261070c8 conf/conf  1058.71   679.86 1.557   587.93..801.67     560.66..4332.72
```

Two of the nine confirm the 64 KiB label at both ends and two confirm the
4 KiB one, and `261070c8` is in both pairs — it is the only draw that confirms
both tables at both ends, which is why the grade is taken from it. The other
two confirmed tables are `96bb4778`'s 64 KiB one at 1.011, which would pass,
and `4a748d6e`'s 4 KiB one at 1.419, which would not. Both confirmed 64 KiB
readings sit on lines that move: `96bb4778`'s `N.pool8` runs 445.55 to 898.44
around a median of 808.79 while its `C.wide8.default` reaches 8252.57 against
a median of 817.47, and `261070c8`'s `C.wide8.default` spans 1044.77 to
14961.31 cold at 64 KiB and 560.66 to 4332.72 cold at 4 KiB, on a runner whose
load average was 5.60 at the start — so its reading is confirmed-cold and
noisy at once. At 64 KiB the grade does not depend on that noise: C's
*minimum* over `N.pool8`'s median is 1.34, outside the bar without the median.
At 4 KiB it does: C's minimum over `N.pool8`'s median is 0.82, so the ranges
overlap and only the medians separate them. Neither row is met on any
statistic that puts C ahead, so both are `no`, and the 4 KiB grading is the
weaker of the two — with `4a748d6e`'s confirmed 4 KiB table agreeing with it
from the other side at 1.419.

The row this section reports its numbers from is `33155821397`, because its
commit `266acf4f` is the runtime the before/after comparison is about — not
because its halves are the tightest, which they are not: the draw table above
puts `96bb4778` closer to 1 on macOS warm 4 KiB (1.0168 against 1.0282) and
`a06c53f9` closer on macOS many files (1.0144 against 1.0222). The cold rows
of any of the nine are a reading of the runner's cache state as much as of the
program. `266acf4f` is not this branch's last runtime: `git diff --stat
266acf4f a06c53f9` changes `runtime.c`, `bridge.c`, `file_adapter.c/.h` and
`contract.h` alongside the harness, the probes, the `Makefile` and
`io-hosts.yml`, and this record's own repairs change the runtime once more
after `a06c53f9`. **What was owed — a macOS draw whose cold labels are
confirmed at both ends — arrived, and the answer was a miss. What is owed now
is such a draw on a quiet runner, which would narrow the 1.011-to-1.477 range
these two confirmed 64 KiB tables span rather than decide whether a cold table
can be read at all.**

What the cold rows do show, and this does not depend on the label because both
lines ran interleaved inside the same table, is the demand-driven helper
policy working: `C.wide8.default` is within 1.2 per cent of its own pinned
eight-helper line at 64 KiB (591.82 against 585.05) and 2.3 per cent at 4 KiB
(489.75 against 478.59), where in 0092 the default trailed `C.wide8.h8` by
1.30 and 1.39 times. Against its own sequential build the completion program
is 2.51 and 2.84 times faster on those tables, where in 0092 it was 1.73 and
1.58.

### What changed in the runtime

Batch 0096's changes are recorded in `docs/done/0096-darwin-handoff.md` with
the stage-level attribution that motivated each, and its "After the tables"
section records the correctness follow-up that landed after this run: the
tables here are read at commit `266acf4f`, and the tip adds a generation check
on the named drain, an atomically published adapter readiness flag, a helper
cap bounded by its storage, a clock guard on the join spin, an adapter that
stops reading as usable before its own lock is destroyed, and a gate arm that
runs the bridge on the shipped default helper policy. None of those changes a
route, a policy or a threshold, and the macOS draw taken at `a06c53f9` — the
last commit that changed the runtime before this record's own repairs, and the
eighth of the nine draws in the table above (run
[33165141309](https://github.com/mbbill/Whitefoot/actions/runs/33165141309)) —
reproduces the two warm rows of the bar to three decimal places: 1.0085
against 1.006 at 64 KiB, 1.0309 against 1.028 at 4 KiB, with many-files at
1.0144 and the two cold rows at 1.302 and 1.229. Its cold labels are refused
at both ends of both tables — the draw table above says which of the nine
confirm and which refuse — so it does not settle the cold bar either; the
handoff records it line by line. In short: the process-wide
wake lock is taken only when there is a sleeper; a drain returns immediately
when the durable ready-event count is zero and a token owner may drain its own
event by name; a queue entry no longer copies a kilobyte of path storage for a
read or a write; the helper wake is issued outside the queue lock and only to
a helper the lock says is asleep; a joining scheduler reads the ready count
for a bounded window before announcing sleep; the helper pool starts empty and
grows on a measured wait rather than on queue depth, bounded by
`WF_BRIDGE_MAX_HELPERS` (eight) rather than the core count; and a positioned read is executed
where it was stated when the adapter holds no helper, has nothing queued and
has measured its operations as not waiting.

### What this reading does not cover

Four limits, stated here so a later reader does not have to rediscover them.

- **The cold tables' cache state**, above: refused before, confirmed after, on
  both of them, in the draw these numbers come from.
- **The growth path is asserted, not observed on this host.** That a pool
  appears when operations wait, and does not when they do not, is pinned by
  two harness cases with a *scripted* clock
  (`test_pool_stays_empty_when_operations_do_not_wait`,
  `test_pool_grows_when_operations_wait`) — deliberately, so the rule is
  tested rather than the machine. The evidence that it also fires on a real
  program is the runner's cold tables, where `C.wide8.default` lands on its
  own pinned eight-helper line. No test on the maintainer's machine watches a
  real program grow a pool, because a warm macOS page cache is exactly the
  case where the rule is meant not to fire.
- **The corpus differential does not reach this path.** `whitefootc
  --emit-llvm` was run over the 22 units of `CORPUS_UNITS` in
  `compiler/tests/programs/parallel.rs` — 25 `.wf` files, since each
  `raw_deflate_*` unit compiles four — in the default lowering and again under
  `--par`, and every `wf__completion_` symbol of all 44 modules was listed.
  Not one module names a `*_submit` or `*_join` entry; the two lowerings emit
  the same set for every unit; and five units emit a completion call at all.
  `byte_string` and `par_layout` emit `write_direct` alone. `dir_walk` emits
  `open_at_direct`, `close_direct`, `directory_next_direct` and
  `write_direct`. `raw_deflate_boundary` emits `open_at_direct`,
  `close_direct`, `pread_direct` and `write_direct`. `wfgrep` emits those four
  and `directory_next_direct`. The other seventeen units — 34 of the 44
  modules — emit no completion call whatever. So that differential covers the
  direct routing this batch changed — which is worth having, since routing
  every `*_direct` entry through `wf_file_execute_timed` is one of its
  changes — and not the submitted path. The programs that do exercise submit
  and join are the bench programs in
  `research/experiments/io-completion-bench/programs/`, which is where the
  overlap-versus-`--no-overlap` differential has to be run.
- **The decline check costs a queue lock.** `wf_file_adapter_transfer_runs_on_caller`
  asks whether anything is queued, and that term takes the adapter's queue
  lock, so every positioned read that reaches the question pays one
  uncontended lock and unlock — against the queue crossing, claim, four slot
  transitions and drain it saves when the answer is yes.

### Linux runner, read-heavy — a draw that does not reproduce

`bench-linux-read` in the same run, on `ubuntu-24.04`: kernel
`6.17.0-1022-azure`, 4 CPUs, INTEL(R) XEON(R) PLATINUM 8573C, 16 GB, tree on
`nvme0n1p1` (ext4, non-rotational), `io_uring_disabled=0`, load 0.48 at start.
Seven recorded interleaved passes after two warm-ups, medians in
milliseconds, with the observed spread because this draw needs it.

```text
                    cold 64 KiB              cold 4 KiB
line             median     min     max   median     min      max
N.direct        6158.09 2781.77 7661.14 11356.66 6661.56 12219.16
N.pool8         1348.85 1167.15 3037.12  1435.03 1288.90  1628.03
N.uring32       1414.28 1116.78 1668.42  1268.13 1202.11  2446.87
S.narrow        4258.41 2478.62 11054.60 8454.51 6171.66 15366.55
S.wide8         4628.62 3645.02 8092.69  8973.99 5298.00 14097.85
C.narrow.default 5579.62 2748.91 9898.19 11129.62 6956.06 12735.69
C.wide8.default 3413.10 1706.37 4882.94  1514.79 1341.48  3200.83
C.wide8.h0      2394.95 1436.40 4715.79  2655.82 1954.60  3487.78
C.wide8.h8      3548.17 1349.15 4765.59  2460.87 1372.74  3495.35

                    warm 64 KiB              warm 4 KiB
line             median     min     max   median     min      max
N.direct         243.84  241.55  253.97    20.95   18.89    22.05
N.pool8           80.72   78.90   83.64     8.51    7.53     8.70
S.narrow         234.92  227.47  244.60    47.90   45.71    51.88
S.wide8          284.31  274.11  298.30    50.89   48.80    56.93
C.narrow.default 234.20  228.59  242.84    50.36   48.90    56.60
C.wide8.default  291.74  280.70  294.55    53.69   53.38    60.63
C.wide8.h0       281.88  276.57  292.72    53.00   51.56    55.66
C.wide8.h8       282.55  276.19  289.80    53.64   51.38    57.56
```

**The cold half of this draw cannot be read.** `N.direct` at 64 KiB spans 2781
to 7661 ms around a median of 6158 — a 2.75-fold range inside one line — and
`S.narrow` spans 2478 to 11054, a 4.46-fold one. Across all forty cold lines
of the two tables the maximum-over-minimum ratio runs from 1.26 (`N.pool8` at
4 KiB) to 4.98 (`C.wide8.h2` at 4 KiB), and nineteen of the forty exceed 2.5.
Batch 0092's Linux 4 KiB table — the cleanest measurement in this document —
held seventeen of its twenty lines' minima and maxima inside four per cent of
their own medians, the three exceptions reaching 4.96, 5.31 and 5.96. A
ranking taken from lines as wide as this draw's is a ranking of the runner.
The interleaved schedule protects against monotonic drift, not against this.

**The warm half is tight, and it does not reproduce the two earlier Linux
readings.** Warm `C.wide8.default` over `S.wide8` is 1.026 at 64 KiB and 1.055
at 4 KiB here, where batch 0092 measured 0.982 and 0.941 and the earlier run on
this branch ([33153717709](https://github.com/mbbill/Whitefoot/actions/runs/33153717709),
commit `96bb4778`) measured 0.984 and 0.946. On both earlier draws the
completion build was slightly faster than its own sequential build; on this one
it is slightly slower.

The narrow lines are the control for this. `C.narrow` and `S.narrow` compile
the same source with and without the completion lowering but state no overlap
width, so a host difference should move them with the wide pair and a change in
the overlap path should not. They do not move together:

```text
warm C/S                  0092   96bb4778     this   delta vs 96bb4778
64 KiB  C.wide8/S.wide8  0.9816   0.9840   1.0261         +0.042
64 KiB  C.narrow/S.narrow 1.0035  1.0023   0.9969         -0.005
 4 KiB  C.wide8/S.wide8  0.9412   0.9460   1.0550         +0.109
 4 KiB  C.narrow/S.narrow 1.0112  1.0157   1.0514         +0.036
```

At 64 KiB the narrow pair does not move at all while the wide pair moves 4.2
points; at 4 KiB the wide pair moves three times as far as the narrow one. So
whatever this is, it is concentrated in the lines that state width, which is
where the completion path does its work. That is what a change in the overlap
path would look like.

Cutting the other way: there is no such change to point at. The only difference
in completion sources between `96bb4778` — the commit of the 0.946 reading —
and `266acf4f`, measured here, is the removal of the `WF_IO_TRACE` stage
instrumentation that `96bb4778` still carried; `git diff 96bb4778..266acf4f --
compiler/src/backend/completion/` is that removal and nothing else. No drain,
submit, publish or policy code differs between the two commits, and removing
instrumentation does not make a program slower. The host also differs, though
not in the way first written here: every Linux runner in the draw table above
reports 4 CPUs, and so did 0092's, so core count is not what separates them. The processor and the disk are — Xeon
Platinum 8370C on `sda1` for 0092, EPYC 7763 on `sda1` for `96bb4778`, Xeon
Platinum 8573C on `nvme0n1p1` here —
and the 8573C's warm 4 KiB sequential line is 50.89 ms against the previous
draw's 83.28, which is a different machine by any reading. The wide lowering
has more scheduling surface than the narrow one, so a host that different can
move the wide pair further without any code being at fault.

**So this section does not claim Linux is unregressed, and it does not claim it
is regressed.** One draw on different hardware, whose cold half is unusable and
whose warm half is the top of both warm ranges the draw table above records, is
not a reading of the bar — but the narrow control makes it a draw worth
resolving rather than one worth dismissing. The correctness evidence is
separate and is not in doubt: `io-hosts` `completion-linux` is green on this
commit, including the required native io_uring adapter probe and the harness
under the address and undefined sanitizers. Thread sanitizer is a separate
step and runs the probes rather than the harness — the isolated core/read
probe, and now the default-route bridge probe. The Linux draw this owed is
below: "The Linux draw at `a06c53f9`".

### The Linux draw at `a06c53f9`

Pushing the correctness follow-up ran `io-bench` again, and its
`bench-linux-read` job is the draw this section says is owed: run
[33165141309](https://github.com/mbbill/Whitefoot/actions/runs/33165141309) on
an AMD EPYC 9V74, 4 CPUs, tree on `nvme0n1p1`, load 0.50 at start. Two things
make it worth reading on its own rather than as one more row of the table.

**Its uncached 4 KiB table is confirmed at both ends** — `probe before the
table: confirmed; probe after it: confirmed` — so it is a reading of a cold
device rather than of a cache. It is not the only such table on this branch,
and the count is not four: the draw table above lists **eight**
`bench-linux-read` uncached 4 KiB tables and every one of them is
`confirmed/confirmed`, so the row this section quotes is worth quoting only
beside the other seven. `266acf4f`'s cold half is the one this section calls
unreadable for its spreads, so its row below is a confirmed label around a
median, not a ranking.

```text
run          commit    processor   C.wide8.default  S.wide8  N.pool8  N.uring32   C/S  C/uring32
33149563172  caa66bad  Xeon 8370C          1455.21  3075.32  1488.00    1456.44 0.473      0.999
33150416900* 34ac1ae2  EPYC 7763           1470.94  3747.70  1474.69    1466.31 0.392      1.003
33151353052  4a748d6e  EPYC 7763           1482.81  4496.24  1486.45    1458.48 0.330      1.017
33153717709  96bb4778  EPYC 7763           1479.87  4227.07  1479.56    1469.81 0.350      1.007
33155821397  266acf4f  Xeon 8573C          1514.79  8973.99  1435.03    1268.13 0.169      1.195
33158144391  72e98cba  EPYC 9V45           1249.62  5085.89  1474.52    1457.10 0.246      0.858
33165141309  a06c53f9  EPYC 9V74           1216.03  4108.74  1482.48    1448.85 0.296      0.839
33172323795  261070c8  Xeon 8370C          1465.26  3469.10  1481.78    1441.72 0.422      1.016
```

`N.uring32` is the fastest native line on every one of the eight, so
`C/uring32` is also C against *every* native line of its table. All eight
runners report four CPUs; the 8370C and 7763 draws ran on `sda1`, the 8573C,
9V45 and 9V74 draws on `nvme0n1p1`. `34ac1ae2`'s run was cancelled after this
table completed and carries the draw table's `*`.

Three of the eight put the eight-wide Whitefoot program ahead of every native
line — 0.999, 0.858 and 0.839 — and this run's table is the furthest ahead
of
the three:

```text
line                        median     min      max
N.direct                   4914.02 3641.98  5119.62
N.pool8                    1482.48 1453.84  1487.17
N.uring32                  1448.85 1209.27  1498.67
S.narrow                   3997.94 3852.94  5228.31
S.wide8                    4108.74 3720.78  6790.88
C.narrow.default           4336.82 3731.27  6871.13
C.wide8.default            1216.03 1206.55  1864.42
C.wide8.h0                 1470.85 1203.64  1629.84
C.wide8.h8                 1490.76 1408.88  1742.53
```

`C.wide8.default` at 1216.03 ms is 3.38 times faster than its own sequential
build, 1.22 times faster than an eight-thread pool and 1.19 times faster than
a hand-written 32-deep io_uring pipeline. `72e98cba` reads almost as far
ahead — 1249.62 against `N.pool8`'s 1474.52 and `N.uring32`'s 1457.10, 1.18
and 1.17 times — and `caa66bad` is a tie rather than a lead, 1455.21 against
the ring's 1456.44, which is 1.23 ms in 1456. Of the remaining five, four sit
within 1.7 per cent behind the ring — `96bb4778` 1479.87 against 1469.81,
`34ac1ae2` 1470.94 against 1466.31, `261070c8` 1465.26 against 1441.72,
`4a748d6e` 1482.81 against 1458.48 — and the fifth, `266acf4f`, sits 19.4 per
cent behind it (1514.79 against 1268.13) on the cold half this record already
calls unreadable for its spreads. All eight beat their own sequential build by
a wide margin: `S/C` in table order is 2.11, 2.55, 3.03, 2.86, 5.92, 4.07,
3.38 and 2.37 times. So what eight confirmed Linux cold 4 KiB tables support
is that the completion program is level with a hand-written native pipeline on
this job and sometimes well ahead of it, and the 1216.03 reading is this
section's because this run is the follow-up's draw — it is also, as the table
shows, the lowest `C.wide8.default` of the eight. The 64 KiB table on the same
run is a cold-start table by its own probe (confirmed before, refused after)
and reads the same way less sharply:
1213.14 against `N.pool8`'s 1275.99 and `S.wide8`'s 1587.44.

**And its warm half does not reproduce the draw above.** Warm
`C.wide8/S.wide8` here is 1.010 at 64 KiB and 0.989 at 4 KiB, with the narrow
control at 0.998 and 1.015. Set beside the earlier readings:

This is every Linux warm draw on this branch, not a selection: the seven
`bench-linux-read` warm tables of the draw table above — the two cancelled
runs stopped before their warm halves — with batch 0092's draw on top. The
two further 0092-era draws under "Reproduced on two further pairs of runners"
above (runs 33131934257 and 33133182075) are on other commits and are not
repeated here.

```text
draw         run          commit    processor   disk  warm 64  warm 4  narrow 64  narrow 4
0092         33130875022  6ac36126  Xeon 8370C  sda1   0.9816  0.9412     1.0035    1.0112
merge base   33149563172  caa66bad  Xeon 8370C  sda1   0.9997  0.9271     0.9983    0.9940
repair       33151353052  4a748d6e  EPYC 7763   sda1   1.0112  1.0004     0.9941    0.9974
earlier      33153717709  96bb4778  EPYC 7763   sda1   0.9840  0.9460     1.0023    1.0157
this section 33155821397  266acf4f  Xeon 8573C  nvme   1.0261  1.0550     0.9969    1.0514
record       33158144391  72e98cba  EPYC 9V45   nvme   1.0078  1.0402     1.0092    1.0128
follow-up    33165141309  a06c53f9  EPYC 9V74   nvme   1.0099  0.9887     0.9984    1.0149
repair round 33172323795  261070c8  Xeon 8370C  sda1   1.0148  0.9998     0.9956    0.9990
```

Read the warm 4 KiB column down: 0.9412, 0.9271, 1.0004, 0.9460, 1.0550,
1.0402, 0.9887, 0.9998. Four readings are below one, two are on it to within
half a thousandth, and **two are above it by four points or more — 1.0550 on
the Xeon 8573C and 1.0402 on the EPYC 9V45**, which are different processors
on different disks. So the 8573C is not an outlier to be explained; it is the
larger of two. The 8370C rows say the same thing a second way: batch 0092 read
0.9412 on that processor model, this branch's merge base read 0.9271 on it,
and the repair round read 0.9998 on it — the ratio moves seven points across
three draws of one processor model, which is what a hosted runner gives this
pair from one draw to the next rather than a property of a machine. The warm
64 KiB column spans 0.9840 to 1.0261 over the same seven branch draws.
**Nothing across these draws refutes the no-regression bar, and the bar table
above keeps its `unresolved` grade all the same, because that table reads
`266acf4f` and every other draw is a different one.** What is owed is not an
explanation of one machine but a reading of this pair that does not move six
points between draws, which needs repeated draws on one label rather than one
more draw on a new one.

Every figure in the table above was recomputed from each run's own artifact:
328.38/334.53 and 71.69/76.17 for 0092, and for the seven branch rows the
`S.wide8`, `C.wide8.default`, `S.narrow` and `C.narrow.default` medians of
that run's warm tables — the same parse that produced the draw table above.

### Linux hardware, many files

`bench-linux` ran in every one of this branch's nine `io-bench` runs and
completed its table in all nine, including both runs that were cancelled
later. All nine are here, with batch 0090's two draws of the same job on top:

```text
draw          run          commit    processor   S.wide8  C.wide8.default    C/S
0090 run 1    33114336424  -         EPYC 9V74    141.26           147.04  1.0409
0090 run 2    33115297530  -         EPYC 9V74    110.94           115.97  1.0453
merge base    33149563172  caa66bad  EPYC 7763    123.91           131.09  1.0579
traced        33150416900* 34ac1ae2  EPYC 9V74    142.25           147.70  1.0383
repair        33151353052  4a748d6e  EPYC 7763    122.60           131.14  1.0697
left in place 33153717709  96bb4778  EPYC 7763    122.83           130.17  1.0598
trace removed 33155045849* 135abdf2  EPYC 7763    121.93           128.64  1.0550
drain removed 33155821397  266acf4f  EPYC 7763    122.49           129.55  1.0576
record        33158144391  72e98cba  EPYC 7763    121.61           128.91  1.0600
follow-up     33165141309  a06c53f9  EPYC 7763    120.94           128.73  1.0644
repair round  33172323795  261070c8  EPYC 9V45     96.04           100.85  1.0501
```

The ordering reproduces on every draw — C is slower than S here as it was in
both 0090 draws, which is what batch 0090 recorded and this batch does not
change. The nine branch ratios span 1.0383 to 1.0697 and the two 0090 ratios
sit inside that span, against the within-run spread of about 2 per cent batch
0090 reports for this job. Seven of the nine branch draws are on one processor
model, the EPYC 7763, and those seven alone span 1.0550 to 1.0697, so the
spread is not the hardware. Batch 0090 had a third runner as well, run
33118248259, which that section records only by its headline lines — `S.wide`
112.19 against `C.wide.default` 118.14, a ratio of 1.053 on the unpinned wide
pair rather than on the eight-wide one this table reads — so it is named here
and not tabulated. Eleven draws across a change that no one of them
can resolve is neither enough to call a regression nor enough to call the row
met, which is why the bar table above grades it `unresolved` rather than
`yes`. What it would take is repeated draws on one label, not another host.


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

At that 2026-08-27 revision, canonical `make check` reached the specification
archive gate and stopped with the expected statement that a `CANDIDATE` was
valid branch work but not a merge-ready `ACTIVE` identity. Every later
component was also invoked independently and passed. Activation and merge
remained separate owner-approved work; that expected stop was not an
implementation or test failure.

### Windows native qualification, 2026-08-30

The later `x86_64-pc-windows-msvc` implementation closes that production
execution gap for the current file-operation slice. Commit `f04e15c9` passed
native `io-hosts` run
[33305475906](https://github.com/mbbill/Whitefoot/actions/runs/33305475906).
The Windows job executed all of these boundaries on `windows-latest`:

- the completion core's multi-waiter protocol, the real IOCP adapter, and the
  Windows target-inventory contract marked implemented with native completion,
  no blocking helpers, and scheduler progress;
- cwd-relative, no-follow namespace open and directory enumeration;
- strict compilation of every runtime unit and a linked bridge-capacity probe
  that admits 64 asynchronous reads, returns the 65th to direct fallback
  without blocking, joins all admitted reads, and closes its ledger;
- the production `whitefootc.exe` compiling, linking, and running
  `completion_read_boundary.wf`, with UTF-16 arguments, two cwd-relative file
  opens, output `AB`, and `WF_REQUIRE_WINDOWS_IOCP=1` requiring at least one
  accepted IOCP submission and exact submission/publication closure;
- `host_string_bytes.wf` receiving one U+4241 UTF-16 code unit through the
  native `wmain` boundary and requiring `host_bytes_len == 2`, copy endpoint
  `2`, and the exact little-endian bytes `41 42`; and
- reference and `--par` builds of `par_layout.wf` producing identical bytes
  through the emitted COFF sequential world.

The first native attempt also exposed that Git's Windows checkout had changed
canonical `.wf` LF bytes to CRLF before the compiler saw them. The repository
now marks `*.wf -text`, preserving source identity rather than weakening
[FORM-2] or normalizing inside the compiler. This qualification establishes
correct native execution of the implemented Windows command, namespace, file,
and completion boundaries. It is not a Windows performance measurement and
does not claim a Windows compute worker pool.

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

At Batch 0090, exactly one of the two reasons Windows qualification was
fail-closed had closed: a Windows host existed and both probes passed on it.
The other reason still stood at that revision: its IOCP wake packet was neither
coalesced nor persistent for every already-announced waiter. That Batch 0090
evidence therefore did not move `implemented`. The later
[Windows native qualification](#windows-native-qualification-2026-08-30)
supersedes this historical boundary with the bounded persistent protocol and
production-program evidence.

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

## TCP echo control test, batch 0108 (2026-09-06)

The control test `NETWORK.md` §6 asks for, in
`research/experiments/io-completion-bench/`: `uring_echo.c` is the io_uring
reference (raw ABI, multishot accept and receive into a registered buffer
ring, one ring and one `SO_REUSEPORT` listener per thread, single-issuer
deferred task running), `epoll_echo.c` the second reference (one
edge-triggered epoll and listener per thread), `programs/tcp_echo_server.wf`
the Whitefoot line (a fixed-trip accept loop over the connection count the
invocation names, one parked callee per connection, built with `--par` and
run with `WF_STACKS=1100`), `netload.c` the one load generator, and
`linux-net-bench.sh` the protocol: warm-up and recorded passes in alternating
order, medians over the recorded passes, every echoed byte verified, no
timeout deciding anything. `io-bench.yml`'s Linux job runs it after the file
tables; the table below is the development host, Linux 6.18 on four cores,
`ROUNDS=3 WARMUP=1`.

```text
line                conns   bytes    trips     rt_per_s     p50_us     p99_us   connect_us   vs_uring   vs_epoll
uring.k1                1      64    20000      28538.2       33.0       63.0         81.0       1.00       0.98
epoll.k1                1      64    20000      29235.4       32.0       62.0         26.0       1.02       1.00
wf.k1                   1      64    20000      15354.3       48.0      234.0        101.0       0.54       0.53
uring.k64              64      64     2000     329876.3       64.0     2070.0        440.0       1.00       1.12
epoll.k64              64      64     2000     294618.1       51.0     3532.0        520.0       0.89       1.00
wf.k64                 64      64     2000      35993.9     1862.0     2792.0        636.0       0.11       0.12
uring.k1024          1024      64      200     343760.3     1481.0    10234.0       5370.0       1.00       1.04
epoll.k1024          1024      64      200     330230.5     1634.0    10224.0       3305.0       0.96       1.00
wf.k1024             1024      64      200      26782.6    38105.0    43023.0       3771.0       0.08       0.08
uring.k64.64k          64   65536      200      59161.2      696.0     4131.0        423.0       1.00       0.79
epoll.k64.64k          64   65536      200      74564.0      324.0     5362.0        501.0       1.26       1.00
wf.k64.64k             64   65536      200      18242.5     3302.0     8699.0        539.0       0.31       0.24

line                  bytes_per_s
uring.k64.64k        3877187281.9
epoll.k64.64k        4886625315.3
wf.k64.64k           1195542174.9
```

The Whitefoot line stays at 27 to 36 thousand round trips a second whatever
the connection count while the references reach 330 to 344 thousand, which is
the mark of one serial resource rather than of the number of peers; what is
serial in the runtime, read from the code and not yet measured apart, is one
ring under one submission lock and one completion lock for the whole pool and
one wake through the runtime's eventfd per completion. At one connection the
gap is about one park-and-wake per operation. The test also found the ring's
completion-queue overflow stall at 129 connections, fixed in the same slice;
`docs/done/0108-streams-and-tcp.md` §6 carries the reading of this table and
§5 the defect. 8192 connections in flight is outside the shapes and the stack
pool, and the table stops at 1024.

The runner's own reading, `io-bench.yml`'s Linux job on a6b31b5 (ubuntu-24.04,
four cores, kernel 6.8, `ROUNDS=3 WARMUP=1`), where both references are
slower than here and the Whitefoot line faster, so the ratios differ from this
host's by about a factor of two:

```text
line                conns   bytes    trips     rt_per_s     p50_us     p99_us   connect_us   vs_uring   vs_epoll
wf.k1                   1      64    20000      16179.3       60.0      116.0         89.0       0.80       0.71
uring.k64              64      64     2000     186447.5      135.0     2717.0        796.0       1.00       1.05
epoll.k64              64      64     2000     177846.4       89.0     4246.0       1021.0       0.95       1.00
wf.k64                 64      64     2000      50283.0     1280.0     2080.0       1198.0       0.27       0.28
uring.k1024          1024      64      200     193084.2     5090.0     9698.0       9489.0       1.00       1.03
epoll.k1024          1024      64      200     187009.6     5333.0     9886.0       8694.0       0.97       1.00
wf.k1024             1024      64      200      50056.8    20328.0    25674.0       9987.0       0.26       0.27
uring.k64.64k          64   65536      200      23527.3     2313.0     6528.0       1147.0       1.00       0.64
epoll.k64.64k          64   65536      200      36791.7     1032.0     5885.0        552.0       1.56       1.00
wf.k64.64k             64   65536      200      19670.4     2556.0     5420.0         34.0       0.84       0.53
```

The shape is the same on both hosts: the Whitefoot line is flat across the
connection counts while the references scale, and the ratio at one connection
is the cost of one park-and-wake per operation on that host.
