# Batch 0086 — opens and closes as completion operations

Branch: `batch/0086-open-handout`, from `batch/0084-io-perf` at `8f06cbd6`.
Deliverables: the runtime and qualification change in `compiler/`, the
eight-wide program in `research/experiments/io-completion-bench/`, the
program-level section of `research/investigations/io-model/RESULTS.md`, this
record.

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

## What one file operation actually costs

Before choosing what to move, the two host calls in question were measured
directly, warm cache, on the same trees the workload uses.

```text
                       one openat      one close
macOS M4 host            ~100 us         ~17 us
Linux container           0.85 us        0.45 us
```

The macOS host runs a corporate endpoint-security stack that hooks file
operations, which is why its per-call numbers are two orders of magnitude
above the container's. That is the machine every line in this report is
measured on, so the comparison stays fair; it also means macOS is entirely
open-and-close bound while Linux is not.

These two numbers decide the rest of the batch. On macOS a program that opens
and releases 8,192 files spends about 140 ms of its writer thread closing them
one after another, which is worth taking off that thread. On Linux the same
8,192 closes cost under 4 ms, and the whole open-plus-close budget is about
11 ms of a program that runs for over a hundred — so on Linux the prize is not
the syscall but the bridge round trip wrapped around it.

## The numbers

Full tables and host details are in
`research/investigations/io-model/RESULTS.md`. Both platforms were measured
with the batch-0084 protocol — two warm-ups, recorded runs, medians, every
line checked to publish the same bytes — with one change: the base commit and
the branch were built as separate binaries and run inside **one** plan, so
before and after see the same machine, the same tree and the same page cache.
`S.wide.before` against `S.wide.after` is a built-in control: the sequential
build's direct path is untouched on macOS, so whatever separates those two
lines is measurement noise and nothing else.

PLACEHOLDER_MACOS_TABLE

PLACEHOLDER_LINUX_TABLE

### Against the bar

PLACEHOLDER_BAR

## The mechanism, per backend

### Linux: an open is three ring operations, not one

`IORING_OP_OPENAT` carries the open. Its typed outcome needs the kind of the
object that was actually opened — `open_read` and `open_file` refuse anything
but a regular file, `open_directory` anything but a directory — and there is
no open flag that asks the kernel for that refusal without changing which
error class the writer sees. So the kind check is a second ring operation:
`IORING_OP_STATX` of the descriptor the open produced, named by the empty path
with `AT_EMPTY_PATH`, so it inspects that descriptor and never resolves the
path a second time. When the check refuses, the descriptor is disposed of by a
third, `IORING_OP_CLOSE`, on the same ring. No stage blocks a scheduler
thread, and the adapter's existing retry-pending machinery carries the stages
without a second mechanism: an entry keeps its ring slot across all three.

A `close` is `IORING_OP_CLOSE` directly.

The typed outcome is unchanged because the rule that decides it moved to one
place. `wf_file_kind_outcome` now lives in the shared typed file contract, and
both the bounded POSIX adapter and the ring adapter answer with it, so a FIFO
is refused with the same discriminator whether the open ran on a helper
thread, on the scheduler itself, or in the kernel. The generated program's
open mapper is a switch on that discriminator with an `abort` on anything
outside the set, so a target that answered differently would be a fail-stop
defect rather than a wrong program.

### Every backend: one queue, one lock, one wake

The bridge kept a helper pool of its own — its own mutex, condition variable
and epoch — layered over a file adapter that already had all three. Every
submission therefore took the adapter's queue lock to enqueue, the bridge's
target lock to bump an epoch, the queue lock again to ask how deep the queue
was, and the core's wake lock to announce, then broadcast to every helper. The
adapter's own helper loop, the one that takes from the head of the queue while
a scheduler takes from the tail, was dead code in the shipped configuration.

The bridge pool is gone. One enqueue, under the queue lock it already holds,
appends the entry, signals exactly one helper, and grows the pool if needed.

### Every backend: the pool grows on the evidence that matters

Growth used to fire when the queue held more requests than there were helpers
to take them. That measures how far behind the pool has already fallen, and on
a program that submits a short run of independent operations the queue drains
fast enough that it almost never fires: the eight-wide program left the pool
at its initial size and ran about 1.8x slower than the same program with
helpers pinned at six. Growth now fires when a submission finds no helper
waiting for it, which is the condition that actually means "this program
exposed width the pool cannot absorb". The ceiling is unchanged: the machine's
own CPU count, and a written `WF_IO_HELPERS` still pins the count exactly.

### Every backend: a release is target work

A `[SYS-5]` release is one best-effort close whose diagnostic the language
discards, on a resource the writer has already given up. Nothing observes the
outcome and no frame waits for it, so it has no reason to occupy the writer's
thread. It is now handed to the target wherever a helper exists to take it,
and is the same single direct close everywhere else — which is exactly the
zero-helper configuration and the ring-backed Linux default, where the measured
close costs 0.45 us and a hand-over would only add a queue entry.

A disposal is deliberately not a completion operation. It claims no operation
slot, makes no ownership transition, and publishes nothing, because there is
no result to transfer and no owner to wake; it takes a queue entry and no
more. A full queue refuses it and the caller makes the attempt itself, since
waiting for capacity would put back exactly the serialization the hand-over
removes.

### Windows

Nothing to move. `CreateFile` has no overlapped form, so an open on Windows is
a blocking call whatever carries it, and the IOCP adapter continues to carry
transfers only. The contract additions are header-level and the Windows units
do not include them; `completion-windows-cross` links and imports the same
IOCP and overlapped-file facilities it did before.

## What is and is not handed out

Verified against `completion_file_operation` in `backend/emitter/system.rs`,
which is the whole set the lowering can hand out:

```text
open_read, open_file, open_directory, open_directory_source   handed out
read_at                                                        handed out
write_once                                                     handed out
directory_next                                                 handed out
release (close)                    not a call; target-side disposal
reserve_file, relative_path, arg_get, args_count, host_*       never suspends
```

A release is not an IR system call at all — it is a compiler-derived action at
scope exit whose result is discarded — so it cannot be an overlap-group
member. Taking it off the writer's thread is a target decision, which is where
this batch put it.

PLACEHOLDER_TESTS

PLACEHOLDER_JUDGMENT

PLACEHOLDER_NOTDONE
