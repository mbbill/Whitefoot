# The Whitefoot I/O model — foundation record, revision 2

Status: DESIGN, pre-implementation. Revision 2 folds in the three
adversarial reviews of 2026-08-25 (`spec-sweeper`, `runtime-attacker`,
`blindspot-scout`, reports under the operator's scratch directory
`wf-io/out/`; every load-bearing anchor re-verified by the lead). It
authorizes no execution and changes no rule; every language change it
sketches lands, if at all, through its own specification batch with owner
approval at merge.

Revision 1's two defects, named rather than smoothed over: it claimed
distinct capability values imply world disjointness (false — the spec's
own stdout/stderr same-sink conformance witness refutes it), and it
claimed the per-contact-point narrowing of [EFF-5] "becomes a theorem"
(false — v0.36's order promise spans different resources, so any
narrowing is a versioned semantic decision, not a derivation).

Owner's chartering frame, verbatim (2026-08-24): "我希望外部io设计最好不要被
posix限制了,其实最好不要被任何传统系统API限制思维。我设计wf的初衷之一就是
自下而上构建完整的软件系统,包括操作系统。所以语言是在操作系统以下的。"

## 1. First principles: what external interaction is

A program is a state transformer on memory it owns. External interaction
is contact with state it does not own, and only two physical facts
distinguish that from computation: the outside world has its own clock,
and the outside world's state is shared, so order out there is a real
observable. Everything reducible to these facts fits two primitives:

- **submit** — the program, on its own clock, deposits a request into
  shared state (a device register write, a DMA descriptor, a queue entry).
- **complete** — the world, on its clock, announces an outcome (an
  interrupt, a completion-queue entry, a flag).

The bottom of the stack (NVMe queue pairs, NIC descriptor rings) and the
top (io_uring, IOCP) independently converged on paired queues in shared
memory; only the POSIX middle still carries the 1970s shape. Of the three
constraints that shaped that shape — protection domains, the blocking
thread, interrupt-as-preemption — the first dissolves for proven code
(its *containment* function survives, §8), the second was a
representation choice, and the third reduces to completion delivery (an
interrupt runs no user code; it wakes an executor — Embassy's shape).

One amendment from review: **not every completion answers a submit.**
Signals, device hot-unplug, file-change notification, and child exit
arrive unrequested. The model therefore has a third element: an **event
source** — a capability armed by the program, with a contract fixing
arming, pending/coalescing, drop, and fatal behavior. Writers still call
ordinary operations; no handler or callback construct exists.

## 2. One mode: completion; readiness is a backend

Readiness (select/poll/epoll/kqueue) delivers state, not results; it
exists because POSIX `read` is indivisible; it cannot express ordinary
file reads at all (a regular file is always "ready" and the read still
blocks on the disk — disqualifying for wfgrep's disk-bound workload). A
completion surface can be served from a readiness host by a runtime that
performs the middle steps; the reverse is contorted. The language-facing
model is completion; readiness, thread pools, and bare-metal interrupts
are host backends.

## 3. The language surface: the world becomes regions — under proof, not
by capability equality

No new construct; the writer's program stays sequential; overlap is
permission read off proofs, as [PAR-1] did for compute. The refinement
replaces the `external`/`blocks` row atoms with world-region vocabulary.
The sweep of all 136 `external` / 31 `blocks` occurrences in the active
spec found a mechanical rewrite for most and a missing semantic rule
behind the rest; those rules are this section.

### 3a. Two region kinds; identity lives in the capability's type

Memory regions are [OWN-3] lexical lifetimes. World regions are effect
and alias identities. They are distinct kinds: `&uniq 'b Output<'w>`
carries memory-loan region `'b` and world region `'w`, and neither
substitutes for the other. `own` stays a payload-free mode; each
capability family declares the world-region vector its *type* carries,
and system nominal identity keys on the family plus that vector. (The
main representation cost of the whole design; nothing in `CheckedMode`
carries this today.)

### 3b. Disjointness is proven, never inferred from values

> Different capability values are never by themselves evidence of world
> disjointness. Two world regions are disjoint only when a TCB minting
> rule or a checked generativity derivation proves that every state facet
> the two footprints name cannot alias. Absent that proof, they overlap.

Consequences fixed now, not deferred: stdout and stderr are conservatively
may-alias (the spec's own same-sink witness); separate opens, equal or
distinct paths, and hard links prove nothing about file content
disjointness; `dup` preserves every source region; a handle's lifetime
region is distinct from the persistent object's region; a
capability-producing operation may mint a fresh result region only for
state its contract proves fresh, and a compile-time identity stands for a
may-alias class across executions unless separation is proven. On a
Whitefoot OS the arbiter mints capabilities it can prove disjoint — the
full-stack ambition is what eventually makes the type-level story sound
outright; on POSIX the near-term policy is conservative: one file-object
alias domain per target unless proven otherwise, network connections
fresh only when the TCB mints them, all `Output` values may-alias.

### 3c. World reads are not free; consuming reads are writes

Two monotonic-clock samples overlapped can return t2 < t1 — no
source-order execution produces that. So:

> Read/read overlap is admitted only when the operation's contract proves
> source-order result attribution under overlap. An operation that
> advances a cursor, consumes input, samples an ordered sequence, or
> otherwise changes future observations *writes* its world region.

Stdin consumption, entropy draws, `accept`, and cursor reads are
world-region writes; genuinely idempotent snapshot reads may share.

### 3d. Rows over world regions; does-ness stays checked

With the kinds in place, the row vocabulary extends: `reads('w)` /
`writes('w)` over world regions, [EFF-2]-checked in both directions
exactly as memory rows are, projected through call boundaries — which
requires extending the projection to world-region occurrences in
own-mode actuals, capability values nested in outcomes, and
compiler-derived releases (a release that closes a file *writes* its
handle-lifetime region; today's release rows carry `external, blocks`
and the migration must keep their conformance verdicts). The
possession/use split survives review intact: loans on capability values
order what could happen; world rows state what does.

### 3e. The order law is a versioned decision with a conservative first
step

v0.36's [EFF-5] orders external calls across *different* resources; any
per-region narrowing is a semantic weakening. Two honest migrations
exist: (1) first land the vocabulary with one conservative global
world-order domain joined to every former-`external` operation —
preserving v0.36 order exactly, zero semantic change — then narrow
family by family under evidence, each narrowing a flagged owner
decision; or (2) declare the weakening at once with a complete trace law
(what is ordered, at which linearization point — submission, completion,
or remote observation — with fence semantics; the current SYS-2
inventory contains no fence operation, so one must be added, not
presumed). **Recommendation: (1).** The eventual trace law must decide
the linearization point explicitly; "order" without it is not a law.

### 3f. Traps and world windows: the erroneous promise widens; permission
does not narrow

Both adversarial reviews found the same true fact from opposite ends:
once windows may contain world writes, two current sentences become
false — "no statement of a permitted overlap produces an external
effect at all" and "which claim the record names is the only thing a
schedule may select". Both reviews then proposed gating world-bearing
windows on trap-free closures. That disposition is rejected, on the
2026-08-23 claim ruling this project already made: a claim is a
reviewed always-true lemma, a trapping execution is a defective
program, and permission is never withheld from correct programs to
stabilize a defective execution's observables. The gate would tax
exactly the programs that can never trap.

The doctrine-consistent repair rewrites the erroneous-execution clause
instead: the schedule selects two things for a defective execution —
which false claim the single record names, and which world effects were
performed before the abort. Everything else stands: one complete
record, whole-process abort, no undefined behavior. The TCB obligations
land on the trap path alone, which correct programs never execute: no
new submission after the trap latch; already-submitted operations
retain their family semantics ([TRAP-1]'s existing already-started
clause, extended from "started" to "submitted" — the abort does not
wait for terminal states, because a defective program must still die
promptly); the diagnostic record is written through a TCB-serialized
single write that in-flight program output cannot split.

The nuance that made this look new, dissolved: compute-window
divergence at an abort was externally invisible (the process dies with
its memory), world-window divergence is visible (bytes in a file). But
the language never promised a defective program's partial output a
shape — a sequential defective program also leaves half its output
before trapping. Overlap changes which garbage a defective run leaves,
and garbage shape was never in the contract. Deterministic
reproduction, as before, is free at `WF_WORKERS=0`.

### 3g. Residue the deletion must also settle

`blocks` generalizes to trusted completion/blocking metadata on *every*
target action (operations, releases, close, waits), with a derived
transitive summary for user wrappers, so no backend routes a blocking
action onto a required compute lane. Whether the bare spellings stay
reserved words is a META-5 accepted-set choice. [PRV-1]'s
external-*input* provenance class is a homonym, untouched by effect
migration (rename to `boundary-derived` to avoid confusion). Gated FFI
signatures with unclassifiable world reach charge one conservative
top-world domain — absence of a footprint never implies purity. The
conformance migration is enumerable and enumerated: 42 case files
mention the atoms, 7 manifest records are verdict-sensitive, the
same-sink EFF-5 runtime witness must keep passing under conservative
aliasing, and no verdict changes silently.

## 4. The runtime: one scheduler, two work sources — as a written state
machine, not a slogan

The unification survives review; the sketch did not. A compute frame is
self-runnable and stealable; an I/O frame is runnable only by the world;
join's fallback is "run it" for compute and "wait for it" for I/O. What
the reviews add:

- **The park/wake state machine is the design.** Its one law: after
  *any* progress — a reaped CQE, a consumed wake hint, a completed
  frame — the lane returns to the top of the scheduling loop; it never
  parks on the heels of progress. (The sketched loop had a
  sleep-forever edge: reap the join target's completion, flip its flag,
  park anyway.) Parking follows announce-then-recheck, the discipline
  the existing condvar path already implements with its idle bit.
- **Wake plumbing, corrected.** `io_uring`'s registered eventfd notifies
  ring→fd, not fd→ring; a compute wake becomes a CQE only via a POLL_ADD
  on the eventfd, one-shot unless multishot, re-armed after every
  consumption. An eventfd read drains the whole count: one notification
  means "scan all sources to quiescence, re-arm, announce, scan once
  more, then park" — never "run one frame".
- **Ring affinity follows the executing lane.** A stolen frame submits
  on the thief's ring and joins on it; the compute slot's home-lane
  field must not be reused as ring identity, or completions land on a
  ring nobody waits on.
- **Completions never enter compute deques.** Chase-Lev has one
  producer; a completion writes results and a terminal state, full stop.
  Dependent continuations are the C stack below the join — that is what
  "no continuations" means operationally.
- **Join helping is bounded.** An unbounded steal-while-joining both
  inverts latency (the ready completion waits out an arbitrary stolen
  frame) and nests frames on one lane stack (the 0079 audit already
  established stolen calls do not start at stack bottom; the ledger
  cannot see the scheduler layer). First version: a joining lane drains
  its completions first, returns the instant its target is done, helps
  only frames of its own window, and takes at most one unrelated steal
  per round under a per-lane help-depth bound that the stack ledger
  names.
- **Fairness is stated, not assumed:** completion queues drained after
  at most a bounded number of compute frames and vice versa; batches
  bounded; a non-terminating frame voids latency guarantees and the
  limitation is documented.
- **The kqueue/waiter shape** keeps one never-blocking waiter (readiness,
  dispatch, enqueue, wake) over a disk pool of fixed depth; completion
  nodes are preallocated per frame so the mailbox can never be full;
  the mailbox is an MPSC with release/acquire publication,
  exactly-once terminal transitions, generation-tagged frames against
  ABA, enqueue linearized before wake, and announce-then-recheck
  consumers — each a named requirement, none assumed from the word
  "MPSC". (The runtime has already paid once for a plain shared word;
  the lane-count race of batch 0080 is the precedent.)
- **In-flight loans run to terminal state.** From submission
  linearization to the operation's terminal completion: the buffer
  neither moves, frees, nor is reused; the frame stays out of the free
  list; an exclusive input loan excludes the lane itself. A cancel
  request is not a terminal state (its CQE and the operation's are
  unordered; hardware operations may be uncancelable).
- **Abort teardown is target qualification.** Hosted Linux: ring
  teardown cancels what it can and holds references for what it cannot;
  nothing user-side drains queues in a signal handler. Bare metal: DMA
  may write after the CPU declares abort — the arbiter quiesces,
  quarantines, or declares reset-before-reuse; the floor's
  first-record-wins abort composes with this as TCB teardown, not
  language cleanup. v0.36's abandoned-continuation sentence does not
  cover asynchronous families; the I/O batch owes [TRAP-1] an
  already-submitted-work clause.
- **Cancellation, first version:** none at window exits. Every normal or
  recoverable exit edge first observes terminal states for all submitted
  operations of its window — the structure the current lowering already
  has (join after the last member, before any exit edge). "Stop
  waiting" without a terminal state is not sound; early-exit
  cancellation waits for a design that can transfer buffer ownership to
  a hidden reaper, and is deliberately not promised now.

## 5. The worlds, re-drawn on two axes

Review dissolved revision 1's single axis twice over. The honest
structure:

- **Execution axis:** sequential (source-order schedule, no overlap) or
  overlapped (permission-based).
- **World-provider axis:** live world, or — named now, built later — a
  recorded world for deterministic replay. `WF_WORKERS=0` alone is *not*
  a full determinism anchor; a live filesystem, clock, or network varies
  under it. Replay is a world backend, not a third lowering.

Within the execution axis, three concepts the current bootstrap conflates
must split: whether the overlapped world is selected; how many compute
lanes exist; whether an I/O backend is initialized. Current code maps
`WF_WORKERS<2` to "pool off" and the emitter defers a refused member to
after the last member — so revision 1's claim that W=1 keeps source
order was **false**; a W=1 overlapped world changes three observables
(which claim an erroneous execution records; published bytes when traps
and world writes could mix — narrowed by §3f's widened erroneous clause; stack resource
records, since overlapped clones spend 48 B/level against sequential
16 B/level on the 0079 measurement). Each is a flagged decision at the
batch that changes the mapping, with `WF_WORKERS=0` retained as the
sequential world and the compute-claim refusal made an explicit
`compute_lanes < 2` rule rather than a side effect of "pool off".
Embedded terminology corrected: `WF_WORKERS=0` means no overlap and no
worker pool, not "no I/O substrate"; a bare-metal build still carries
the minimal driver/executor its targets qualify.

## 6. What any first spec batch must fix per operation (the minimal
closure)

For every system operation (the current inventory is fifteen; the table
rewrites as a whole, not by suffix):

1. authority inputs and capability mint/alias relations;
2. world footprint and its commutativity class (§3c);
3. submission, linearization, loan-release, and completion points —
   "complete" names which of buffer-reuse, world-visibility, durability
   (§ blindspot B5);
4. outcome taxonomy: typed world outcome vs static rejection vs claim
   trap vs TCB resource death vs target defect, keyed by semantic
   source, never by errno spelling;
5. progress contract (may wait forever vs eventually completes; device
   removal completes everything; a lost completion is a target defect);
6. protection, memory-order, and lifetime obligations for target
   qualification;
7. a stable operation identity for ledgers and conformance.

Program kinds need root capability tables: today's `command` entry has
no stdin (the word appears zero times in the spec) and bare metal has no
args/cwd; each kind fixes its closed capability set and lifecycle.
Conformance runs on three tiers: a scripted deterministic world for
semantics across completion schedules; target qualification for real
backends; performance strictly outside conformance. An `--io-ledger`
sibling of `--par-ledger` shows sites, origin relations, footprints, and
grants/denials at compile time.

## 7. Structural backlog (shaped, not blocking)

From the blind-spot sweep, parked with owners: signal classification
(SIGPIPE is an outcome, SIGCHLD a child-lifecycle completion, fatal
faults stay with the floor); processes as capabilities (no writer fork/
in-place exec; atomic spawn returning a completion-required Child; wait
is a completion; ChildOutcome separate from ExitStatus); pipes with
arbiter-owned history and Rx/Tx facets; filesystem namespace operations
as multi-region footprints (rename writes two directories); file object
vs cursor facets (positioned reads want per-range footprints only where
targets prove them); mmap/MMIO never behind ordinary borrows (snapshot
or arbiter-proved exclusive mappings only — optimizer facts break on
externally mutable memory); clock domains (monotonic vs wall, opaque
instants, deadline races arbiter-linearized); entropy as one seed
operation feeding an owned local PRNG; network state machines
(NetworkAuthority, Listener, connection type states; accept writes the
listener's sequence region and mints fresh connection origins);
stream Rx/Tx facets with per-direction FIFO and half-close as consuming
transitions; datagram outcomes distinct from byte streams; resolver as
explicit capability; variable-sized results on caller buffers or owned
backing, never hidden allocation; visibility vs durability vs
crash-atomicity as distinct resource families (fsync commits a file,
not its directory entry). Distant, named: TLS placement, typed device
control without an ioctl escape hatch, cross-program leases, zero-copy
splice as dual-capability operations.

## 8. What a syscall was for; what protection keeps

Protection against bad access → replaced by proof; the gate becomes a
queue write. Portability → a library concern. Naming and authority →
affine capabilities, arbiter-minted on a Whitefoot OS. Multiplexing
shared devices → irreducible, a queue arbiter. And the owner's
correction stands as §7 of revision 1 stated: containment of *logic*
errors survives proof — the residual kernel is bootstrap, scheduler,
and an arbiter that keeps accounts (quotas, fairness, revocation, kill),
with affine capabilities making revocation tractable and "process"
surviving as the unit of containment and reclamation. Before a
Whitefoot OS exists, target qualification names the protection each
host actually provides (descriptor aliasing, generation-tagged handle
tables, quarantine of late completions).

## 9. Falsifiers and next experiments, cheapest first

1. **The migration ledger (paper).** Amendments §3a–§3g enumerate the
   spec surface (117 lines carrying the atoms; 42 conformance files; 7
   verdict-sensitive records; the release table; the entry canonical
   form). Write the conservative-first migration (§3e option 1) as a
   draft delta and check every RESISTS sentence from the sweep resolves
   against it. Any sentence still resisting is a missed rule.
2. **kqueue prototype on this machine.** Waiter + preallocated-node
   mailbox + bounded disk pool + the §4 state machine; re-measure the
   directory-walk 2.83x (recorded caveat: measuring-machine security
   daemon). Success gates any runtime batch.
3. **Unified-parking probe (Linux).** POLL_ADD-on-eventfd multishot,
   re-arm discipline, mixed load; verify no lost wakeup against the §4
   law, and measure against the condvar path.

## 10. Decisions queued for the owner

1. §3e: conservative-first order migration (recommended) vs declared
   weakening with a full trace law.
2. §3f: the widened erroneous-execution clause (schedule also selects a
   defective run's pre-abort world effects; [TRAP-1]'s already-started
   clause extends to submitted operations). Rewrites two [PAR-1]
   sentences; gates nothing; the rewrite carries constitution T3's
   direction clause into the rule text, so the next reader under
   pressure finds the yield direction beside the promise.
3. §5: the WF_WORKERS mapping change and its three observable deltas.
4. Whether `external`/`blocks` remain reserved spellings after deletion.
5. The provenance rename (`boundary-derived`) riding the same batch or
   a separate one.
