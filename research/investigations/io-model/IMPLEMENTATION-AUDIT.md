# Completion I/O implementation audit

Status: historical implementation evidence. This audit compares experimental
revision `d822851c276438df92b0c730b807524ec42aec9f` with the design which
preceded the owner-confirmed unified-state revision of 2026-08-26. Its runtime
measurements and findings about writer-code helpers remain evidence. Its
positive references to capability roots, family fragments, and ordered
attribution are superseded by `FIRST-PRINCIPLES.md` and `DESIGN.md`; they are
not requirements for the current rebuild.

## The deciding example

The smallest useful test is two independent reads:

```whitefoot
let left = read_once(file: &uniq 'lf left_file, destination: &uniq 'lb left_bytes, start: 0_u64, end: left_end);
let right = read_once(file: &uniq 'rf right_file, destination: &uniq 'rb right_bytes, start: 0_u64, end: right_end);
```

The owned file capabilities and destination buffers are distinct. Nothing in
the second operation needs the first result. The compiler should therefore
submit both operations as soon as their arguments and authority are ready.

The experimental branch cannot prove that separation. It adds a command-wide
world region, treats every two world accesses as possibly aliased, and leaves
the calls sequential. Even when an overlap is actualized through a user
wrapper, the runtime sends the complete wrapper to a fixed blocking worker.
The worker can then execute writer computation, matches, calls, and claims in
addition to the target operation.

That behavior fails both sides of the example. Independent operations do not
overlap by default, and a helper that should only report target completion
becomes a second executor of Whitefoot code.

## What the branch actually implements

The branch contains seven file, directory, and output operations that may
wait. Each ordinary operation is wrapped in a stack-local `wf_io_frame`, put
on one unbounded queue, executed by one of four blocking worker threads, and
waited for by the submitting lane.

The files named `kqueue.c`, `io_uring.c`, and `iocp.c` do not submit those I/O
operations to kqueue, io_uring, or IOCP. They transport synthetic wake hints:

```text
file operation
    -> fixed blocking worker performs open/read/write/close
    -> generic completion mailbox
    -> EVFILT_USER, eventfd poll, or posted IOCP wake packet
    -> submitting lane's native-stack wait
```

Linux contains no `IORING_OP_READ`, `IORING_OP_WRITE`, or corresponding file
submission. Windows associates no file or socket handle with its completion
port and issues no overlapped operation. macOS registers an `EVFILT_USER`
wake rather than file readiness or asynchronous file work.

The branch's canonical `make check` passes. That proves that its compiler,
specification, and tests agree with one another. It does not prove that the
runtime is a native completion backend or that the chosen authority model is
correct.

## Measured result

The branch's own batch record reports that the final implementation is about
30.0 percent slower for its sequential comparison and 24.2 to 28.0 percent
slower in the recorded worker configurations. The same record reports no
useful world-call overlap. A fresh run of `make check` on the unmodified audit
worktree passed 1,277 library tests and 500 conformance cases. Its completion
harness measured the completion path at about 2.26 times its condition-variable
comparison in that run.

These numbers apply only to that branch, host, and harness. They are enough to
reject the implementation as Whitefoot's default fast shape. They do not yet
compare the clean rebuild with native blocking and native completion I/O.

## Disposition

| Experimental part | Disposition | Reason |
|---|---|---|
| Writer-visible world regions and `Output<'world>`-style identities | Discard | They duplicate ownership identity, fail to prove independent capabilities separate, and globally serialize the deciding example. |
| Command-wide world order | Discard | Family authority and attribution determine real order. Unrelated logical capabilities have no language-level cross-root order. |
| `wf__par_publish_io` whole-wrapper dispatch | Discard | A target helper must not execute writer code. |
| Fixed-lane native-stack `wf_io_wait` as the common lowering | Discard | It occupies a lane and cannot resume a compact continuation on another eligible lane. |
| Fixed four-thread unbounded blocking queue | Discard as common architecture | It imposes one host strategy on every target and has no admission or backpressure boundary. |
| Blocking worker as a target fallback | Repair and retain narrowly | A target may need a bounded helper for operations without native completion, but the helper executes only a typed target adapter. |
| Caller-owned stable operation storage | Retain | It avoids per-operation allocation and keeps borrowed payloads alive until their declared release milestone. |
| Release/acquire result publication | Retain | It establishes the required result and payload visibility edge. |
| Bounded completion draining and ready-frame publication | Retain after repair | The clean runtime scans fixed slots through `event_pending` and uses a fixed mutex-protected ready ring; it keeps the bounded principle without retaining the experimental intrusive MPSC shape or one fixed owner lane. |
| Announce, recheck, then park | Retain | It closes the lost-wake window when compute work and completions share one sleeping decision. |
| Generation field | Repair | A target token must capture the generation and validate it before writing any result. Reading the frame's current generation at completion does not stop an old publisher. |
| Compiler-owned target-action summary | Retain after repair | Dispatch and milestones belong to trusted operation contracts, not writer effect syntax. A transitive summary selects suspension lowering, not a worker on which to run a whole wrapper. |
| Deterministic I/O ledger | Retain after repair | It should report capability roots, authority fragments, milestones, and actualized suspension instead of world-region vectors. |
| Synthetic platform wake code | Retain only as wake evidence | It may help implement a scheduler wake endpoint, but it is not evidence of native file completion. |
| Trap-latch submission promises | Discard | Correct operations may not pay a trap-specific gate for an impossible false claim. The branch does not implement the promised gate, which is preferable to adding it. |

## Clean rebuild boundary

The rebuild starts from `main` plus the settled first-principles record. It
does not reverse-edit the 163-file experimental diff. The first implemented
specialization has the following boundary:

1. Source removes `external` and `blocks`. Both spellings become ordinary
   identifiers. Ordinary memory effects continue to name borrow regions.
2. System operation contracts add compiler-owned suspension, authority,
   capacity, attribution, and milestone facts. User functions receive exact
   derived summaries through the closed call graph.
3. Direct system calls participate in the existing proof-derived overlap
   judgment. Their authority projects to concrete capability places and
   family fragments, never to one global world object.
4. A finite one-shot operation uses bounded stable storage and reports at
   least result-ready, one loan-released fact per retained argument, and
   terminal milestones. The first file specialization may publish them together,
   but the common state cannot collapse them into one unnamed bit.
5. A helper fallback executes only a target adapter. Writer continuations run
   on scheduler lanes.
6. Pure compute links no completion runtime and carries no completion sidecar.
7. A false claim adds no submission check, metadata field, queue transition,
   wake, or permission denial on the correct path.

The first specialization is complete only when independent direct operations
overlap, same-resource conflicts still obey their family authority, a single
thread can submit and make progress, completion-before-wait causes no wake,
stale publication is rejected before result storage changes, and the
repository-wide `make check` passes. Native Linux, Windows, and macOS target
adapters are then measured separately against matching direct blocking
implementations.
