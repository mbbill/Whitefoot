# Completion I/O results

Status: the unified-state rebuild was validated and remeasured on 2026-08-27.
The later sections retain historical component measurements for comparison.
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
