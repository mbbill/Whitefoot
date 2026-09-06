# Batch 0084 — program-level I/O performance

Branch: `batch/0084-io-perf`, from main at `0295399d`.
Deliverables: the measurement bundle at
`research/experiments/io-completion-bench/`, three runtime fixes and one
compiler option in `compiler/`, the program-level section of
`research/investigations/io-model/RESULTS.md`, this record.

## Charter

Spec v0.37 activated the unified-state completion I/O model on the evidence
of C-level microbenchmarks of the completion core: about 35 ns per round
trip, a cached 4 KiB `pread` costing 53–64 ns more through completion than
through a direct call, no helper crossover on cached reads, park/wake parity
with a condvar. No whole program had ever been measured. The owner's rule is
that if the completion design does not reach the best native performance on
real programs, the design is dead.

So: measure whole programs that do real I/O against the best hand-written
native shape and against Whitefoot's own sequential build, on macOS and on
Linux where the native completion path is io_uring; and where the numbers
miss, fix the runtime or the lowering and measure again.

## Method

Three lines per workload, every line checked to publish the same bytes on
every recorded run:

- **N** — the best hand-written native C shape, `-O2`, no handicap: a
  single-threaded blocking loop, a pthread pool over a striped range, and on
  Linux a raw io_uring read pipeline written against the kernel ABI.
- **S** — the Whitefoot program built with the new `whitefootc --no-overlap`,
  which emits the module a compiler with no overlap lowering emits. Every I/O
  call is an ordinary direct call.
- **C** — the same Whitefoot source built the way it ships.

C and S are one source compiled two ways, which is what makes the pair a
statement about the lowering rather than about two programs. The tree
definition and the folded checksum live once, in `workload.h`, shared by the
generator, the baselines, and the Whitefoot programs; the checksum is
position-weighted so a four-wide lane split and a one-at-a-time loop fold to
the same value.

Two warm-up runs then seven recorded runs per line; medians and min/max
reported, with child user and system CPU from `wait4`.

## What the language admits, and what that cost the workload plan

Three of the chartered workloads had to be reshaped, because the current
system surface does not express them. These are findings, not
substitutions:

- **There is no file-write API.** `write_once` takes an `Output`, and the
  only `Output` values in existence are `command.stdout` and
  `command.stderr` — the entry-input table in
  `compiler/src/semantic/check/entry_form.rs` is a closed set of five. A
  many-file read-then-write copy cannot be written at all. The
  read-many-independent-files half is what was measured.
- **`open_read` and `open_file` reject a non-regular file by design**
  (`file_adapter.c` checks `S_ISREG` against the operation's
  `expected_kind`, and a test pins the FIFO rejection). There is no way to
  read from a pipe, so the blocking-read half of the pipe relay is not
  expressible. The blocking-*write* half is, through two independent
  Outputs, and that is what was measured.
- **Directory traversal cannot overlap by construction.** `directory_next`
  takes `&uniq DirectorySource`, one advancing cursor, so successive
  listings of one directory serialize on the unique loan. That is the
  ownership model working as designed; it also means a directory walk's
  listing cost is inherently sequential and only the per-entry opens and
  reads can overlap. The measured workload opens files by generated name,
  which isolates exactly those.

## Workloads

1. **many-files** — open and read a generated tree of 8,192 files of mixed
   size (1–16 KiB, 68 MiB total), one positioned read each, folding a
   checksum. Two Whitefoot programs: `many_files_wide.wf` states four opens
   and then four positioned reads consecutively, and `many_files_narrow.wf`
   is the same work as the natural one-file-at-a-time loop.
2. **pipe-relay** — push two independent byte streams at two independent
   consumers, chunks larger than the pipe buffer so each write genuinely
   waits, consumers sleeping per read so the consumer sets the pace.

## The numbers

Full tables, protocol, and host details are in
`research/investigations/io-model/RESULTS.md`. Medians of seven recorded
runs after two warm-ups; 8,192 files, 1 to 16 KiB, 68 MiB, warm cache.

```text
                      macOS M4, 10 cores      Linux 6.8.0, 2 CPUs
N best                289.8 ms  (pool6)        30.5 ms  (pool2)
N native ring         not available            63.9 ms  (io_uring depth 32)
N ring at depth 4     not available           117.1 ms  (io_uring depth 4)
S best                974.8 ms  (wide)        292.0 ms  (wide)
C best / default      475.2 ms  (wide)        121.1 ms  (wide)
S narrow              989.4 ms                303.3 ms
C narrow              1069.1 ms               300.9 ms
```

Pipe relay, macOS, medians of seven: N.seq 394.8 ms, N.threads 389.9 ms,
S.relay 395.6 ms, C.relay 390.4 ms.

### Against the bar

The bar: C at least as fast as S on every workload, and within 10 percent of
N wherever N is a native completion path or a fairly sized thread pool.

```text
workload / platform      C vs S            C vs N                      bar
many files / macOS       2.05x faster      1.64x slower than pool6     missed
                                           1.30x slower than pool4
many files / Linux       2.41x faster      3.97x slower than pool2     missed
                                           1.89x slower than uring32
                                           1.03x slower than uring4    met
two streams / macOS      1.01x faster      1.00x of threads            met
```

C beats S on every workload. The 10 percent bar is met against the native
completion path at the concurrency the Whitefoot source can actually ask for,
and missed against every native shape that asks for more.

The pipe workload discriminates nothing: all four lines, including one thread
per stream, fall inside 1.5 percent. The pipe buffer already decouples the two
streams, so there is no serialization for overlap to recover. That is a null
result about this workload, not about overlap.

## Findings

### The overlap a program gets is decided by its source shape, not its I/O

`many_files_narrow.wf` is the shape a writer reaches for. It emits **zero**
submissions. The lowering forms overlap groups from runs of *consecutive*
calls in one basic block (`IrBuilder::completion_steps`, and
`has_later_independent_call` in `semantic/permission.rs`), so a loop body
holding one I/O call per iteration has no later independent call to overlap
with, ever. Measured, the narrow program is within noise of its own
sequential build on both platforms.

`many_files_wide.wf` is the same work hand-widened to four. It emits three
open submissions plus one direct open, and three read submissions plus one
direct read, and it is roughly twice as fast as its own sequential build.

This is the single largest determinant of program-level I/O performance in
the current system, and it is a language and lowering question rather than a
runtime one. It is flagged below rather than changed here.

### At matched depth the completion path equals hand-written io_uring

On Linux the deciding comparison is not C against the fastest thing a C
programmer can write, but C against the same concurrency a C programmer
would have to write by hand. The four-wide Whitefoot program and the raw
io_uring baseline at queue depth four land within a few percent of each
other. The remaining distance to the baseline's best number is the baseline
running at depth 32 — a depth the Whitefoot source cannot ask for, for the
reason above.

### A thread pool beats io_uring on this workload

The io_uring baseline is about twice as slow as the plain pthread pool on
the two-CPU Linux box, at every depth. Opens dominate a many-small-file
workload and the ring does not carry them, so each file still costs a
blocking `openat` on the submitting thread while the ring only saves on
cached reads that were already cheap. Whitefoot's own Linux adapter has the
same split — `linux_io_uring.c` submits `IORING_OP_READ` and
`IORING_OP_WRITE` and nothing else.

## What was fixed

### 1. A waiting scheduler busy-waited whenever a helper owned queued work

`wf__completion_file_join` refused to park while the target queue held
anything at all. With helpers, `wf_bridge_progress` deliberately does not let
a waiting scheduler execute an unrelated queued request, so the join loop
spun: no progress, no sleep, for exactly as long as a helper kept the queue
non-empty. The guard is now one named predicate,
`wf_bridge_target_work_needs_this_thread`, true only in the zero-helper
configuration where the waiting scheduler really is the target's engine.

Effect on the four-wide workload's default configuration: user CPU 266 ms
to 60 ms.

### 2. The default helper count was pinned at the worst measured value

An unset `WF_IO_HELPERS` meant one helper. One helper is right for a program
with one operation outstanding and wrong for a program that exposes width:
the submitting scheduler then waits on a queue only one thread is draining.
The policy now starts at one and grows by one, under the lock the submission
announcement already holds, only when the queue holds more requests than
there are helpers to take them, and never past the machine's own CPU count.
A written `WF_IO_HELPERS` still pins the count exactly, so every test that
names `0`, `1`, or `4` is unaffected.

Where a native completion path is ready — Linux with io_uring — the default
is now **zero** helpers. The ring already carries every transfer without a
handoff, so a helper can only serve the operations the ring does not take,
and those are exactly the ones a warm cache answers immediately.

### 3. No Linux link of any completion program compiled at all

`bridge.c` declared a union member spelled `linux`. `whitefootc` compiles the
runtime units with the host compiler's default dialect, which is a GNU
dialect, and that dialect predefines `linux` as `1`. Every attempt to link a
completion program on Linux failed in the C compiler. It shipped unnoticed
because the repository's own Linux probes compile with `-std=c11`, where the
macro is not defined, so the probe evidence in RESULTS.md was real and the
compiler was still broken. Renamed to `ring`, with a test that scans every
compiler-linked C unit for identifiers the host compiler predefines.

### 4. `whitefootc --no-overlap`

There was no way to compile one source into the program that reaches the host
through ordinary direct calls, so there was no way to measure the S line.
`OverlapLowering::Off` already existed internally; this exposes it. It is a
measurement switch, not a performance option — the default build is what
ships — and it refuses to be written together with `--par`, which selects the
opposite lowering.

## Judgment calls

- Chose `open_file`-by-generated-name over a directory walk for the
  many-files workload, so the measurement isolates opens and reads instead of
  mixing in a traversal that cannot overlap by construction. The traversal
  limitation is recorded above rather than benchmarked.
- Wrote the wide program at four-way width rather than eight. Four is enough
  to separate overlap from no overlap and to match against a depth-four
  native baseline; eight would have meant eight nested open matches by hand
  for a point the depth-matched comparison already makes.
- Ran the whole Linux pipeline inside one container rather than mounting the
  generated tree from the host, because a bind mount would have measured the
  host's file sharing rather than the kernel's I/O path.
- Kept `--security-opt seccomp=unconfined` in the documented Linux target
  rather than working around it: without it `io_uring_setup` is refused and
  both the native baseline and Whitefoot's own adapter silently fall back,
  which would have produced a confidently wrong table.
- Made the C and S lines one source compiled two ways rather than two
  sources, so no difference between them can be a difference of program.

## Flagged for a design decision

Each of these is a language or surface question, out of scope for a
performance batch, and each is stated with the concrete program that shows it.

1. **Overlap width is a source-shape property.** `many_files_narrow.wf` and
   `many_files_wide.wf` do identical work; the first emits zero submissions
   and measures like its own sequential build, the second emits three plus a
   direct one and is roughly twice as fast. Overlap groups are runs of
   consecutive calls in one basic block, so a loop with one I/O call per
   iteration can never overlap, and a writer who wants depth must hand-unroll
   and hand-nest the matches. The decision is whether the language should let
   a writer state independence across iterations, whether the lowering should
   find it, or whether hand-widening is the intended contract. This is the
   largest single determinant of program-level I/O performance in the current
   system.
2. **There is no file-write API.** The only `Output` values are
   `command.stdout` and `command.stderr`. No Whitefoot program can write a
   file, so the read-then-write copy workload does not exist.
3. **There is no way to read a pipe or any non-regular file.** `open_read`
   and `open_file` check `S_ISREG` and reject, by design and with a test. The
   blocking-read half of a relay is not expressible.
4. **io_uring carries reads and writes only.** Opens and closes never reach
   the ring, so on a many-small-file workload each file still costs a
   blocking `openat` on the submitting thread. This is where the Linux
   distance to a plain thread pool lives, and it is why the pool beats the
   ring here at every depth tested.
5. **Directory traversal cannot overlap.** `directory_next` takes
   `&uniq DirectorySource`. One cursor, so successive listings serialize.
   That is the ownership model working; it does mean a `wfgrep`-shaped walk's
   listing cost is inherently sequential.
6. **An unparsable `WF_IO_HELPERS` now means "unset".** It previously meant
   one helper. No test depended on it and treating garbage as absent matches
   what the value means, but it is a behavior change and is recorded here
   rather than buried.

## Not done

- **Uncached measurement.** `fcntl(F_NOCACHE)` and `posix_fadvise(DONTNEED)`
  are available to the C baseline, but the Whitefoot surface exposes no
  equivalent, so N and C could not have been compared on the same terms. Every
  file number here has a warm page cache, which is the case least favourable
  to completion and most favourable to a direct call.
- **A width-8 program.** The depth-matched Linux comparison (four-wide
  Whitefoot against io_uring at depth four) already isolates protocol cost
  from width, so an eight-wide program would have cost eight hand-nested open
  matches to restate a point already made.
- **Syscall and context-switch counts.** Wall time, and child user and system
  CPU, are reported. The CPU split already attributes the differences that
  mattered — the busy-wait showed up as user CPU, the helper handoff as system
  CPU — so no tracing tool was added.
- **More than two Linux CPUs.** The container VM has two. The Linux pool
  numbers saturate at two threads for that reason, and no conclusion about
  pool scaling on a larger Linux machine is claimed.
