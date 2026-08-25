# The Whitefoot I/O model — foundation record

Status: DESIGN, pre-implementation. Distilled from four owner design rounds
of 2026-08-24/25, superseding nothing; W2 of `docs/current-plan.md` is the
workstream this document grounds. It authorizes no execution and changes no
rule; every language change it sketches lands, if at all, through its own
specification batch with owner approval at merge.

Owner's chartering frame, verbatim (2026-08-24): "我希望外部io设计最好不要被
posix限制了,其实最好不要被任何传统系统API限制思维。我设计wf的初衷之一就是
自下而上构建完整的软件系统,包括操作系统。所以语言是在操作系统以下的" — and
the method: derive from first principles what syscalls and external I/O are
for, ask which constraints shaped today's designs, and check whether those
constraints survive a rebuilt lower stack, down to bare-metal register and
interrupt access.

## 1. First principles: what external interaction is

Strip every API away. A program is a state transformer on memory it owns.
External interaction is contact with state it does not own, and only two
physical facts distinguish that from computation:

1. The outside world has its own clock. Packet arrival, disk completion,
   timer expiry happen at times the program does not choose.
2. The outside world's state is shared. Effects on it are observable by
   third parties, which is why order out there is a real observable.

Everything reducible to these two facts fits two primitives:

- **submit** — the program, on its own clock, deposits a request into
  shared state (a device register write, a DMA descriptor, a queue entry).
- **complete** — the world, on its clock, announces that something
  happened (an interrupt, a completion-queue entry, a flag).

Evidence that this is the natural shape rather than a fashion: the bottom
of the stack (NVMe queue pairs, NIC descriptor rings) and the top
(io_uring, Windows IOCP) have independently converged on paired
submission/completion queues in shared memory. Only the middle — POSIX
`read`/`write` plus blocking threads — still carries the 1970s shape.

### The three constraints that shaped the old shape, audited

- **Protection domains.** The syscall is a controlled gate because
  untrusted code shares the machine. Whitefoot's premise (machine-checked
  memory and type safety, no unsafe escape) dissolves the *mechanism* for
  proven code — Singularity demonstrated software-isolated processes where
  the gate becomes a call. It does NOT dissolve the *function* of
  protection: see §7.
- **The blocking thread.** A suspended thread per outstanding operation is
  a representation choice, not physics; the async-I/O movement is the
  industry backing out of it. Whitefoot never adopted it and need not.
- **Interrupt versus poll.** On bare metal the world's clock arrives as an
  interrupt. Embedded Rust (Embassy) shows the clean reduction: the
  interrupt runs no user code, it only wakes an executor — an interrupt is
  a completion delivery. Same shape again.

Conclusion: design against the two queues, not against any system API.

## 2. One mode: completion, with readiness demoted to a backend

Readiness (select/poll/epoll/kqueue) delivers *state* ("this descriptor
would not block"); the operation and its copy remain the caller's,
performed synchronously afterward. It exists because POSIX `read` is
indivisible. Three structural defects: two trips per operation; the state
can lie by the time the operation runs; and it cannot express ordinary
file reads at all — a regular file is always "ready" and the read still
blocks on the disk, which is why epoll-era servers kept thread pools and
why wfgrep's disk-dominated workload cannot live on readiness.

Completion (io_uring/IOCP/NVMe/Embassy) delivers *results*: the operation
itself is submitted, the buffer is on loan while in flight, and the
completion carries the outcome. Disk, network, and timers are uniform;
batching is native. The subsumption is one-directional — a runtime can
serve a completion surface on a readiness host by performing the middle
steps itself, while the reverse is contorted — so the language-facing
model is completion, and readiness is one host backend among several.

## 3. The language surface: no async, no external row — the world becomes
regions

The writer-facing design adds no construct. The program stays sequential;
dataflow already records dependence; overlap is permission read off
proofs, exactly as [PAR-1] did for compute. Two existing mechanisms are
refined instead:

### 3a. Capabilities are already the marker

System resources are already affine capability values threaded through
signatures (`command.stdout as out: own Output`; no ambient authority
exists). The owner's observation: a globally-unique writable object whose
use requires holding it exclusively gives atomicity and per-object
ordering through the loan machinery of batch 0081 with no effect
annotation at all — `external`/`blocks` are effect *coloring* doing work
the type system does better.

### 3b. The objection that improved the design

Possession is not use: `fn discard(output: own Output, ...) -> unit` and
`fn emit(...)` that actually writes have identical signatures; ownership
flow cannot distinguish them, and the written byte outlives the
capability that wrote it. So the does-it-touch-the-world fact must live
on the signature and be machine-checked — but the recorder need not be a
dedicated atom. The repair: **give each capability a world-region
identity and let the existing row vocabulary speak about it.**

```
fn discard['o](output: own Output<'o>, ...) -> result: own unit pure
    // no 'o in the row: holds the key, provably never opens the door
fn emit['o](output: own Output<'o>, ...) -> result: own unit writes('o)
    // write_once's own signature carries writes('o); [EFF-2] projection
    // forces the caller to declare it; under-declaration already rejects
```

What each of `external`/`blocks`'s current jobs maps to:

| current job | new owner |
|---|---|
| overlap atomicity/order (condition 3's blanket refusal) | loans on capability values + world-region write footprints, judged by the 0081 machinery, same matrix |
| signature-level "does it touch the world" | `reads('o)`/`writes('o)` over world regions, [EFF-2]-checked |
| trap law ("no external effect after the record") | "no write to any world region after the record" |
| `blocks` (duration on the world's clock) | a TCB-known attribute of system operations; not a language fact |

Two dividends fall out. The per-contact-point order law stops being a
decree: two writes through one capability are a write/write conflict on
one world region (order preserved); through different capabilities they
are disjoint (overlap permitted) — EFF-5's narrowing becomes a theorem of
the footprint machinery. And world *reads* (stdin, clocks) become
ordinary `reads('o)`, unlocking read/read overlap that the blanket
refusal denies today.

### 3c. Honest costs and boundaries

- Own-mode parameters do not carry regions today; `Output<'o>` needs real
  representation work (the `allocates(arena 'r)` precedent is related but
  not identical). This is the main implementation cost.
- Capability identity approximates contact-point identity. On POSIX two
  capabilities can alias one file; near-term policy: reads overlap
  freely (a moving world races sequential execution too), network writes
  overlap across connections (the network's own contract is
  per-connection FIFO), file writes stay conservatively ordered. On a
  Whitefoot OS the queue arbiter mints capabilities it can *prove*
  disjoint — a capability becomes a loan issued by the world's owner —
  and the type-level reasoning becomes sound outright. The full-stack
  ambition is thus a soundness argument, not just a portability one.
- Cross-region ordering, where wanted, is an explicit fence written as an
  ordinary operation (fsync's shape), not new syntax.

## 4. The runtime: one scheduler, two work sources, no continuations

The owner's summary is the design: a compute frame is self-runnable and
worker-stealable; an I/O frame is runnable by nobody but the world.

```
              compute frame                  I/O frame
published to  own lane's deque               the world's submission queue
runnable by   owner (join fallback), any     only the world
              stealing worker
join fallback run it yourself                wait for it (completion queue)
steal end     oldest (largest subtree)       n/a — the world takes all
```

No suspendable stacks, no futures, no coloring: the hand-out protocol
(claim/publish/join/release) is reused with two substitutions — publish
submits the operation (SQE carrying the frame pointer), join checks a
completion flag and, when unset, *works instead of waiting* (steal loop),
parking only when nothing anywhere is runnable. A lane at a join is a
worker, not a waiter. Structured windows make this sufficient: frames are
preallocated, joins are local, completion is a flag flip.

Host shapes:

- **io_uring (Linux).** One queue pair per lane (mirroring
  deque-per-lane). The idle path becomes: pop own deque → steal → reap
  own CQ → park in `io_uring_enter`. The pool's compute wake is an
  eventfd registered in each ring, so compute arrival is *also* a
  completion — the condition variable disappears and each lane has
  exactly one blocking point, the completion queue. This is the literal
  form of "all I/O blocks at one point".
- **kqueue/degraded (macOS, generic POSIX).** One waiter thread runs the
  readiness loop (disk operations on a small blocking pool), posts
  completions to an MPSC mailbox, pokes the existing wake path; the lane
  idle path adds one drain step. Language and program unchanged.
- **Bare metal.** submit = device registers/DMA descriptor; complete =
  the interrupt service routine posting to the mailbox and waking the
  executor (Embassy's shape). No OS anywhere in the contract.

Scope honesty: in-flight depth equals what windows expose. wfgrep-class
workloads (bounded overlap of I/O with I/O and compute) fit; a
10k-connection server whose concurrency is unrelated to program structure
does not — that is the explicit-concurrency track's question, and it will
land on this substrate rather than beside it.

## 5. The two worlds, re-drawn

Today's boundary — sequential clones versus overlapped lowering, selected
once by worker count — asks a CPU question. I/O overlap's parallelism
lives in the world (disks, NICs), so it is profitable at one lane;
mapping "one lane" to "blocking clones" throws away exactly the profit.
The repair costs no third lowering:

- `WF_WORKERS=0` — the sequential world, unchanged: no runtime, strict
  source order, the deterministic reproduction anchor and the zero-runtime
  embedded build.
- `WF_WORKERS=1` — **meaning change, flagged as an owner decision at its
  merge**: the overlapped world with one compute lane. Single-threaded
  async: I/O overlaps, compute does not. Compute hand-outs self-erase
  through the existing refusal path (`wf__par_claim` returns NULL; a
  refused offer is the same call made inline), so the compute side costs
  one predictable check while the world side runs at full depth.
- `WF_WORKERS=N`/unset — the overlapped world at N lanes.

Single-lane determinism survives I/O overlap because the world runs no
writer code: program statements execute in source order on the one lane,
`claim` statements cannot enter windows, and completions only flip flags
consumed at joins. The re-drawn boundary is honestly named: not parallel
versus serial, but *the machine alone* versus *the machine plus the
world's clock*.

## 6. What a syscall was for, and what remains

- Protection against bad access → replaced by proof (the premise of the
  language). The gate becomes a queue write.
- Portability abstraction → a library concern, not a boundary concern.
- Naming and authority → affine capability values; already the language's
  shape; on a Whitefoot OS, minted by the arbiter, unforgeable by
  construction.
- Multiplexing shared devices → **irreducible**, but its shape is a queue
  arbiter, not a function-call interface.

## 7. Protection's second job survives (owner's correction, adopted)

Proof removes the *mechanism* of address-space isolation, not the
*function* of containment. A logically wrong program — proven
memory-safe, still looping, leaking, or flooding — must not sink its
neighbors. The residual kernel of a full-Whitefoot stack is therefore:
bootstrap, scheduler, and an **arbiter that keeps accounts** — quotas,
fairness, revocation, kill. Affine capabilities make revocation tractable
(the arbiter knows every key it minted; kill = reclaim). The process
survives as a concept: no longer an address space, but the unit of
containment and reclamation — which a trapping claim also needs, since an
abort is per-process by design. This is the inter-process face of the
exhaustion charter's principle: running out, or misbehaving, is a designed
event.

## 8. Falsifiers and next experiments, cheapest first

1. **Paper sweep (no code).** List every specification sentence that
   mentions `external` or `blocks` and rewrite each in world-region
   vocabulary. Any sentence that resists rewriting is a fourth job the
   refinement missed. Grep-bounded, one sitting.
2. **kqueue prototype on this machine.** Hand-split `list_once`/
   `read_once` into submit/complete inside the runtime, waiter thread +
   mailbox + idle-path drain, and re-measure the directory-walk workload
   whose recorded 2.83x carries a measurement-artifact caveat. Answers:
   the honest speedup; the cost of serving a completion surface from a
   readiness host; whether any in-flight-buffer shape escapes the loan
   machinery.
3. **Unified-parking probe (Linux).** Verify the eventfd-in-ring trick
   gives one blocking point without lost wakeups under mixed compute/I/O
   load, and measure it against the condvar path.

## 9. Open questions this record does not settle

- Cancellation: an effect submitted to the world cannot be unsubmitted;
  "cancel" is "stop waiting" plus a completed-or-not outcome — the
  abandoned-continuation semantics v0.36 just fixed, generalized. What a
  window exit owes an in-flight operation needs its own ruling.
- Partial completions and short reads/writes: does the operation
  vocabulary stay whole-operation (`read_once`) with the runtime looping,
  or do partials surface? (Leaning: whole-operation stays; partials are a
  POSIX-ism.)
- Timeouts as first-class completions (a timer is the purest submit/
  complete pair) and their interaction with `Result` routes.
- Backpressure: submission-queue depth is a resource ceiling in the TCB
  (WF_PAR_MAX_LANES's sibling); whether any program-visible signal is
  ever warranted.
- The world-region representation for own-mode capabilities (§3c) — the
  main design-to-spec gap.
