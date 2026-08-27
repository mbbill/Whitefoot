# Batch 0086 — opens and closes as completion operations

Branch: `batch/0086-open-handout`, from `batch/0084-io-perf` at `8f06cbd6`.
Deliverables: the runtime change in `compiler/`, the eight-wide program in
`research/experiments/io-completion-bench/`, the program-level section of
`research/investigations/io-model/RESULTS.md`, this record.

## Charter

Batch 0084 measured whole programs for the first time and left one diagnosis
on the table. On a many-small-file workload the opens dominate, and no open
ever reached a completion path: the Linux ring submitted `IORING_OP_READ` and
`IORING_OP_WRITE` and nothing else, so every file still cost a blocking
`openat` on the submitting scheduler, and on macOS every open crossed a helper
queue whose per-operation handoff was what separated the shipped build from a
plain thread pool. Closes were never on a completion path at all.

So: make opens and closes first-class completion operations on every backend,
cut the per-operation handoff cost, and measure again at a fair width.

## What one file operation costs

Before deciding what to move, the host calls in question were measured
directly, warm cache, on the same trees the workload uses, in the pattern a
program actually uses — one file open at a time, not the whole tree opened and
then closed, which measures a descriptor table 8,192 entries deep instead.

```text
                    openat      pread 64 KiB      close
macOS M4 host       116 us          1.9 us        4.8 us
Linux container    0.85 us          ~1 us        0.45 us
```

The macOS host runs a corporate endpoint-security stack that hooks file
operations, which is why its per-call numbers are two orders of magnitude
above the container's. Every line in this report is measured on that host, so
the comparison stays fair; it does mean macOS is almost entirely open-bound
while Linux is not open-bound at all. On Linux the whole open-plus-close
budget for the workload is about 11 ms of a program that runs for over a
hundred.

These numbers decided the batch. They are also the reason two of the three
things this batch set out to do turned out not to be worth doing.

## What shipped

### Linux: the path resolution is a ring operation

`IORING_OP_OPENAT` carries the open, and `IORING_OP_CLOSE` a close. The kind
check that makes an open typed — `open_read` and `open_file` refuse anything
but a regular file, `open_directory` anything but a directory — is one
`fstat` of the descriptor the kernel already produced, performed where the
completion is reaped. A descriptor the check refuses is disposed of by the
same single close the direct path makes.

That split is measured, not preferred, and the measurement is in "What was
tried and removed" below. What it buys on this host is not speed: it is that
the zero-helper configuration a ready ring selects no longer blocks its one
scheduler thread inside `openat`, where a slow path resolution would stall
every unrelated ready frame. It costs nothing to have that.

The typed outcome is unchanged because the rule that decides it moved to one
place. `wf_file_kind_outcome` lives in the shared typed file contract and both
adapters answer with it, so a FIFO is refused with the same discriminator
whether the open ran on a helper thread, on the scheduler, or in the kernel.
The generated program's open mapper is a switch on that discriminator with an
`abort` on anything outside the set, so a target that answered differently
would be a fail-stop defect rather than a wrong program.

### Every backend: one queue, one lock, one wake

The bridge kept a helper pool of its own — its own mutex, condition variable
and epoch — layered over a file adapter that already had all three. Every
submission took the adapter's queue lock to enqueue, the bridge's target lock
to bump an epoch, the queue lock again to ask how deep the queue was, and the
core's wake lock to announce, then broadcast to every helper. The adapter's
own helper loop, the one that takes from the head of the queue while a
scheduler takes from the tail, was dead code in the shipped configuration.

The bridge pool is gone. One enqueue, under the queue lock it already holds,
appends the entry, signals exactly one helper, and grows the pool when the
queue has outrun it. Measured at every pinned helper count on macOS, before
and after agree within one percent — this is a simplification, not a speedup,
and it is recorded as one.

### The eight-wide program

`many_files_wide8.wf` states eight opens and then eight positioned reads
consecutively. It emits seven open submissions plus one direct open and seven
read submissions plus one direct read, against the four-wide program's three
plus one; the narrow program still emits none. Hand-widening from four to
eight is worth 13 percent on macOS (629 ms to 546 ms) and 19 percent on Linux
(147 ms to 119 ms).

## What was tried and removed

Three changes were built, measured, and taken back out. Each is recorded here
with the number that refuted it, because the next person to have the same idea
should find the measurement rather than repeat it.

**A second ring round trip for the kind check.** `IORING_OP_STATX` of the
descriptor the open produced, named by the empty path with `AT_EMPTY_PATH`,
keeps the reaping thread free of host calls entirely. Two ring round trips
cost more than the open they wrap: the eight-wide program ran 152 ms against
116 ms for the bounded adapter it replaced, and the four-wide one 203 ms
against 140 ms. With the check done as one `fstat` instead, the same programs
run 119 ms and 141 ms. The whole 31-to-45 percent was the second round trip.

**Growing the helper pool when a submission finds no helper waiting.** This
looked like the right condition — a request queued while every helper is busy
is exactly the serialization helpers remove — and it never fires: a helper
that has been signalled but not yet scheduled still counts as waiting, so a
run of consecutive submissions sees an available helper every time. The
four-wide program ran 919 ms against 625 ms for the queue-depth rule it
replaced. Queue depth is a lagging signal, but it is a true one.

**Handing a resource release to a helper.** A `[SYS-5]` release is one
best-effort close whose diagnostic the language discards, on a resource the
writer has already given up: nothing observes the outcome and no frame waits,
so it has no reason to occupy the writer's thread. It was built as a disposal
that takes a queue entry and no operation slot, and it is neutral — before and
after agree within one percent at every pinned helper count. The 17 us per
close that had justified it was an artifact of the first microbenchmark, which
closed 8,192 descriptors held open at once; in the pattern a program uses the
same close costs 4.8 us against a 116 us open. Neutral mechanism does not earn
its place, so it went.

A fourth was rejected without building it. Deferring the ring kick would turn
a run of seven submissions into one `io_uring_enter` instead of seven, worth
perhaps 9 ms of the Linux program, but it means an operation is not handed to
the kernel when it is submitted. The four-wide and eight-wide programs submit
their opens and then make a *direct* blocking open before joining, so a
deferred kick would hold every submitted open behind that blocking one — which
is precisely the overlap the design promises and
`independent_io_reaches_the_second_operation_before_the_first_unblocks`
pins.

## The numbers

Full tables and host details are in
`research/investigations/io-model/RESULTS.md`. The batch-0084 protocol — two
warm-ups, recorded runs, medians, every line checked to publish the same bytes
— with one change: the base commit and the branch are built as two compilers
and both sets of binaries run inside **one** plan, so before and after see the
same machine, the same tree and the same page cache.

`S.wide.before` against `S.wide.after` is the control. The sequential build
reaches the host through ordinary direct calls and nothing on that path changed
on macOS, so whatever separates those two lines is measurement noise. Under
the batch-0084 method on a loaded host that pair differed by 19 percent; on
the quiet paired runs it differs by 0.01 to 4 percent.

macOS 26.5.2, Apple M4, 10 cores. Medians of fifteen after two warm-ups:

```text
line                    before     after      best-of
N.pool4                            455.64
N.pool6                            374.29
N.pool8                            373.91     best N
S.wide                 1171.42   1217.53     control
S.wide8                1168.63   1130.97     control
C.wide                  637.33    629.30
C.wide8                 543.40    545.50     best C
C.narrow               1182.99   1183.83
```

Linux 6.8.0 aarch64, 2 CPUs, io_uring permitted, tree on the container-local
filesystem. Medians of nine after two warm-ups:

```text
line                    before     after      best-of
N.pool2                             40.21     best N
N.uring4                           138.76
N.uring8                            94.89
N.uring32                           82.46     best one-thread N
S.wide                  335.89    337.29     control
S.wide8                 331.72    335.50     control
C.wide                  142.31    146.73
C.wide8                 118.09    119.47     best C
C.narrow                346.32    346.65
```

### Against the bar

The bar: C at least as fast as S on every workload, and within 10 percent of
the best native shape at matched width.

```text
comparison                                   ratio     bar
macOS  C.wide8 against S.wide8               2.07x faster
macOS  C.wide8 against N.pool8 (width 8)     1.46x slower   missed
macOS  C.wide  against N.pool4 (width 4)     1.38x slower   missed
Linux  C.wide8 against S.wide8               2.81x faster
Linux  C.wide  against N.uring4 (depth 4)    1.06x slower   met
Linux  C.wide8 against N.uring8 (depth 8)    1.26x slower   missed
Linux  C.wide8 against N.uring32             1.45x slower   missed
Linux  C.wide8 against N.pool2               2.97x slower   missed
```

C beats its own sequential build everywhere, by about two times on macOS and
2.8 times on Linux. The depth-matched Linux comparison batch 0084 reported —
the four-wide program against a hand-written ring at queue depth four — still
holds, at 6 percent. It opens to 26 percent when both are widened to eight,
and that widening is the whole finding about where the distance lives.

### Where the remaining distance is

It is not the opens, and it is not the completion protocol's per-operation
cost.

**Linux is compute-bound and barrier-bound.** The entire open-plus-close
budget of the workload is about 11 ms of a 119 ms program — 9 percent — so no
change to opens could have moved the total by more than that, and moving them
onto the ring moved it by one percent. `C.wide8` spends 65.7 ms of user CPU on
one thread, its own bounds-checked fold, and that is a floor under any
single-threaded line. Against `N.uring32`, which also folds on one thread, C
uses 127 ms of CPU against 103 ms and takes 119.5 ms against 82.5 ms: about a
quarter more CPU, about half again as much wall time. The excess wall beyond
the excess CPU is the group barrier — the program joins all eight operations
before submitting the next eight, so every round costs the maximum of eight
latencies while the ring baseline keeps 32 continuously in flight. That is
also why the depth-matched ratio is 1.06x at four and 1.26x at eight: a wider
group is a wider maximum. `N.pool2` is not a matched shape at all; it folds
each file on the worker that read it, which is compute parallelism the source
cannot express.

**macOS is open-bound, and the host limits how much that helps anyone.** One
`openat` costs 116 us. The best native shape turns 1138 ms of serial work into
374 ms — 3.0x on eight threads, so the endpoint-security hooks serialize most
of the concurrency a pool asks for. The eight-wide Whitefoot program gets
2.07x. The difference is what the writer thread still does serially in every
round: its own direct open, eight folds, eight releases, and the join of all
eight before the next round starts.

Both remaining gaps are the same shape the batch-0084 record flagged: overlap
groups are runs of consecutive calls in one basic block, so nothing pipelines
across iterations and every round pays a barrier.

## What is and is not handed out

Verified against `completion_file_operation` in `backend/emitter/system.rs`,
which is the whole set the lowering can hand out, and against the emitted
modules:

```text
open_read, open_file, open_directory, open_directory_source   handed out
read_at                                                        handed out
write_once                                                     handed out
directory_next                                                 handed out
release (close)                            not a call, never handed out
reserve_file, relative_path, arg_get, args_count, host_*       never suspends
```

A release is not an IR system call at all — it is a compiler-derived action at
scope exit whose result is discarded — so it can never be an overlap-group
member. `wf__completion_file_close_submit` and
`wf__completion_file_status_submit` exist in the bridge ABI and have no
emitter call site at all: the lowering has never handed out a close or a
status. They are exercised by the harness, and routing them to the ring keeps
the two adapters answering the same operations, but no emitted program's
behaviour changes with them.

## Tests added

Bridge-level, in `compiler/src/backend/completion/harness.c`, so every one of
them runs on both platforms and in all three helper configurations:

- `test_open_failure_classes_are_typed_outcomes` — a name that does not
  resolve, a directory opened as a regular file, a regular file opened as a
  directory, and the directory that does open as one, each through both the
  submitted and the direct path, each checked for its exact discriminator and
  for the refused descriptor being closed.
- `test_open_capacity_refuses_and_resubmits` — the whole operation capacity
  filled with opens, the next one honestly refused before a token is touched,
  every one joined and closed, then the refused request readmitted.
- `test_open_results_reach_every_independent_owner` — six threads each holding
  their own open at once, half joining immediately and half only after
  yielding, so both the already-terminal join and the park-and-wake join are
  covered.
- The existing typed-lifecycle test gained a close of a descriptor whose
  authority is already gone (typed `EBADF`, not a crash), the same submitted
  refusal of a directory that the direct path already covered, and — under
  `WF_REQUIRE_LINUX_IO_URING` — an assertion that the open and both closes
  really were ring submissions. Without that last one the whole test would
  pass on a silent fallback to the bounded adapter, which is exactly the
  regression the ring path exists to prevent.

Ring-level, in `compiler/src/backend/completion/native_adapter_probe.c`,
which links only the core and the ring adapter:

- `probe_open_and_close_cases` — a regular open and close, a double close, a
  name that does not resolve, a directory and a FIFO refused for a regular
  open with the descriptor disposed of on the ring, and a directory opened as
  a directory.
- `probe_open_capacity_case` — the adapter's bounded entries exhausted by
  opens, the third refused as `WAIT_CAPACITY` without taking ownership, and
  the same token readmitted once an entry returns.
- `probe_open_generation_cases` — a duplicate terminal on an open result
  refused as a duplicate, and a token copied before its slot was recycled
  refused as stale rather than overwriting the live open that now owns the
  slot. It builds a one-slot runtime so the recycled operation lands on
  exactly the storage the stale token names.

Compiler-level, in `compiler/src/backend/tests/completion.rs`:

- `a_native_ring_carries_opens_and_closes_under_one_kind_rule` — the ring
  submits `IORING_OP_OPENAT`, `IORING_OP_STATX` and `IORING_OP_CLOSE`; both
  adapters answer with the one shared kind rule; no stage of an open calls
  `openat`, `fstat`, `statx` or `close` on a scheduler thread; and the bridge
  offers an open to the ring before its bounded fallback.
- `an_unset_helper_setting_selects_a_bounded_demand_driven_pool` and
  `a_waiting_scheduler_parks_unless_it_is_itself_the_target_engine` were
  retargeted at the collapsed pool, and the first now also pins that one
  queued request wakes exactly one helper and that the bridge keeps no pool of
  its own.

One existing assertion changed meaning rather than target.
`test_process_wide_target_helper_budget` asserted that an unset
`WF_IO_HELPERS` yields exactly one helper. That was the fixed default before
growth existed, and it kept passing after growth shipped in batch 0084 only
because the growth rule fires rarely. It now asserts what the policy actually
promises: a written value pins the count exactly, an unset one stays between
one and the machine's own CPU count, and where a native ring carries the
operations it may be none.

## Judgment calls

- **Measured before and after in one run rather than two.** The first
  measurements were taken while other work saturated the host, and the noise
  floor was about 20 percent — wide enough to invent a result and wide enough
  to hide one. Building the base commit and the branch as separate binaries
  and interleaving them in one plan makes the comparison immune to that, and
  the `S.wide.before` against `S.wide.after` pair, whose code is identical on
  macOS, reports the residual noise directly. On the final quiet run those two
  lines differ by 0.01 percent.
- **Two changes proposed on the noisy numbers were removed after the quiet
  ones refuted them.** The helper-growth rule and the release hand-over are
  described under "What was tried and removed" above. Keeping either would
  have been keeping a mechanism that its own measurement does not support.
- **The kind check went on the ring rather than into a reap-time `fstat`.**
  A `fstat` of the descriptor at completion-reaping time would be one cheap
  syscall instead of a second ring round trip, and on Linux it would probably
  measure slightly better. It would also put a blocking host call back on the
  scheduler thread for every open, which is the property this batch exists to
  remove. The ring shape is the one the design asks for and the one that
  stays correct when the open is not answered from a warm cache.
- **The direct open stayed a blocking `openat`.** `wf__completion_file_pread_direct`
  goes through the ring on Linux, so an open could have followed it. It is
  left alone because the direct open is the last member of an overlap group,
  taken exactly when the ring already owns the group's other opens: a blocking
  call there runs in parallel with them and costs one syscall instead of a
  submit, an enter and a reap. It also keeps the `--no-overlap` line an
  honestly blocking reference.
- **A close of a negative descriptor falls back to the bounded adapter.** The
  ring refuses that request shape, and a refusal after the operation is
  claimed is a fail-stop defect rather than a writer-visible outcome. The
  bounded adapter answers it as an ordinary typed `EBADF`, which is what the
  language rule wants, so the bridge routes it there before claiming anything.
- **The Linux image gained the compiler-runtime package.** Without
  `libclang-rt-18-dev` the container can compile and run the ring adapter but
  cannot sanitize it, and this batch's whole point is new adapter code.
- **The eight-wide program was written by a one-shot generator, not by hand.**
  Eight nested open matches are mechanical and easy to get subtly wrong; the
  generator copied everything above `command fn main` from the four-wide
  program verbatim, so the two differ only in width. The generator was deleted
  after use and the emitted program is what ships.

## Not done

- **Windows opens and closes.** `CreateFile` has no overlapped form, so there
  is nothing to move; the IOCP adapter still carries transfers only, and
  `completion-windows-cross` links and imports the same facilities it did
  before. This is the honest refusal the charter allowed, not an omission.
- **`directory_next` on Linux.** It remains the Darwin-only qualified
  facility. `getdents64` has no io_uring opcode, and the traversal cursor
  cannot overlap with itself anyway because `directory_next` takes
  `&uniq DirectorySource`.
- **`write_once` on the ring.** Unchanged and deliberate: the ring's write
  request fixes an explicit offset, and `write_once` is an unpositioned
  `Output` operation whose stream semantics that would change.
- **A reap-time `fstat` variant of the kind check, measured.** The ring
  shape was chosen on the design principle rather than on a comparison, and
  the comparison was not run. It is the obvious next experiment if the
  remaining Linux distance ever matters.
- **Uncached measurement.** Unchanged from batch 0084: the Whitefoot surface
  exposes no `F_NOCACHE` or `posix_fadvise` equivalent, so N and C could not
  be compared on the same terms. Every file number here has a warm page
  cache.
- **More than two Linux CPUs.** The container VM has two, so no conclusion
  about pool scaling on a larger Linux machine is claimed.
