# Requirements and mechanisms: a map for the ownership redesign

Research date: 2026-09-16; revised on 2026-09-16 after the D0 verdict
([VERDICT-D0.md](VERDICT-D0.md)) settled the requirement list of section 2,
the owner taking the interim position on every owner decision. This is a
language-design investigation, not an amendment to the active specification
(v0.57) and not a claim of compiler support. It belongs to the access-effects
investigation beside [RESEARCH.md](RESEARCH.md) (literature and hypothesis),
[DESIGN.md](DESIGN.md) (two candidate systems), [CASES.md](CASES.md)
(incremental cases), [PROGRAMS.md](PROGRAMS.md) (the discriminating programs)
and [VERDICT-D0.md](VERDICT-D0.md) (the settled list with its dispositions).
It is the frame in which those candidates are evaluated; it selects nothing.
Supersede it in place when the analysis moves on.

## 1. Purpose

A Rust-style reference is one value that answers several unrelated questions
at once: may this be dereferenced now (memory safety), may anyone else write
while it is held (interference), what may the optimizer assume (aliasing),
which proved facts survive (frame), and who releases the storage (lifecycle).
Whitefoot inherited that coupling and tightened it. The coupling is why a change
to ownership drags regions, effects and parallel permission along, and why such
a change is hard to think through to completion.

This document states the requirements independently, records the least
information each one needs, maps every current mechanism to the requirements
it serves, lists the mechanism families known to serve each requirement, and
records the couplings a candidate design must respect.

## 2. Requirements, settled by the D0 verdict

This list was settled by the D0 debate on 2026-09-16 (three drafts, six
critiques, three judges, one synthesizer), with the owner taking the interim
position on every one of the sixteen owner decisions; each row below carries
that settlement. [VERDICT-D0.md](VERDICT-D0.md) is the record of the
dispositions, the refused alternatives and the owner-decision table, and
EVIDENCE-debate-d0-2026-09-16.md (`EVIDENCE-debate-d0-2026-09-16.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) is the
full debate record. The rows are copied from the verdict's section 2; a
"Disputed" note keeps the minority positions for the record, the majority
position in the row being the settled one. Nothing here amends the live design
tree.

### Row form and authoring rules

- Each row: (a) what must hold; (b) the least a checker must know; (c) where
  checked; (d) not its job; (e) acceptance test over P1–P10 and the pending
  programs P11–P19 of [PROGRAMS.md](PROGRAMS.md). A row whose (e) names only a
  pending program is **provisional** and marked so (AI-D0-21 without its
  deletion clause).
- **Price**: what the row forbids the compiler from doing. A restriction on
  the compiler with no named ground is a defect of the row (B's rule, kept as
  an authoring rule, not as an M row).
- **Cost**: provisional spec-token / program-token / repair-round estimate.
  Every threshold has a grounded provisional value now (K = 48k tokens from
  why-whitefoot §1; the proof-use-cost `growing` series pinned to a bundle;
  1.65x/1.10x from default-floor as retired-compiler evidence) and is reset
  only through an amendment recorded under M11, never "by the first
  measurement".
- **Status** against the current compiler and spec v0.57: green, red,
  unmeasurable, provisional.
- Writer-model trials are evidence with model and protocol pinned, never an
  acceptance criterion (`design/language.md` decision 5).
- Admission: hazard-owner rows, writer-cost rows and abstraction enter now;
  capability rows enter on a blocking program; corpus frequency is never a
  veto (HIS-D0-12, HIS-D0-13 stay refused).
- Status marks, in each row heading, against the pre-verdict list this
  section replaced (git history before 2026-09-16): KEPT, RESTATED, SPLIT,
  MERGED, RECLASSIFIED, RESTORED, ADDED.

### Hazard and duty owners (constitution → rows)

| Constitution hazard or duty | Owner rows |
|---|---|
| undefined behavior | M6, composed from R1, R4 (a false licensed fact), R5, R11, R14, M6(iii) |
| memory corruption | R1, R2, R3, R4 (soundness clause), R8a, M6(iii) |
| data races | R5 under R14's model and access classes |
| uninitialized reads | R1(i) |
| silent overflow; any other unproved partial operation | R11 |
| machine-verifiable resource bounds where budgets are explicit | R13 |
| execution model and external conditions defined | R14 |
| expected input and environment failures have defined behavior | R12; exhaustions outside the model: R14(v) excluded set with a defined stop |
| external interaction through ordinary objects | R10; adapter obligation in M3, THE-REQ-14 settled by the owner on 2026-09-16 (VERDICT-D0 decision 15) |
| machine verification before acceptance; no writer escape | M1, M2, M3 |
| practical iteration at scale; termination alone insufficient | M10 |
| compatibility and evolution | M11 |
| delegation with retained control; less repeated human inspection | R7 (interface sufficiency, provisional), M7, M8; contract authority settled by the owner on 2026-09-16 (VERDICT-D0 decision 14) |

### Requirement rows R1..R15

#### R1 Sequential memory safety (RESTATED)

- (a) (i) No read of storage holding no value; (ii) no program access to
  storage that has ended by any event the model admits (free, reallocation,
  scope release, relocating move, an R14-admitted non-program actor); (iii) no
  access under a stale layout. Ended storage is inaccessible to the program,
  not inert: an R14-admitted actor (allocator, foreign agent) may write it;
  address reuse never revives an ended identity.
- (b) For the storage an access names: live, initialized in the selected part,
  layout current; the complete storage-ending event set (R8a).
- (c) Each access, against the named storage's state at that point.
- (d) Two pointers to one storage; sequential aliased writes; overlap (R5);
  who releases (R2); what a non-program actor may do (R14); layout definition
  (R8b).
- (e) P4 (second `c.read()` legal iff the candidate carries validity across
  `push`; read after `free(v)` refused); P7 (a hole left through `p` is
  visible through `q`; free is permanent); P2 (pointer into `b2` refused after
  `b4` reuses its bytes); P3 (removal invalidates every path to `m`). The
  diagnostic names which of (i)(ii)(iii) failed. M6's small-model check is
  stated over memory events, not the checker's state vocabulary. Pending P11
  (write-once cache) as a capability program with a stated pass/fail under R1
  alone.
- Price: none on the backend. The compiler may reuse ended storage the program
  cannot reach (stack coloring, spill reuse); it may not assume ended storage
  is unwritten by others (R14(iii)).
- Cost: ~2k / one proof step per interior pointer held across a relocating
  call / 1, local. Status: green for P7 shapes; red for stored pointers (P4)
  under OWN-3/OWN-4.
- Note: "a property of the storage, never of the pointer" is the section 6
  decomposition; it lives in the P7 test, not in (a), so a borrow-duration
  candidate is refused by the program, not by the row. "Write-once
  transitions" removed from (b) (a state mechanism).

#### R2 Resource lifecycle accounting (RESTATED)

- (a) Every allocation and external resource is released exactly once on every
  path, including every typed-outcome arm (R12); linear obligations are
  discharged; a buffer loaned to a non-program agent (DMA, io_uring) is an
  obligation whose discharge is a program-observed completion event (a
  completion token is a candidate mechanism, not the requirement).
- (b) Which binding holds which release obligation; definiteness on every
  path including outcome arms; the join rule, stated once here and reused by
  R3; for a loaned buffer, the observed completion event.
- (c) Consuming operations, scope exits, joins, each outcome arm's exit.
  Release decisions are static by M2(ii) (no bookkeeping state the source did
  not bind), not by a "failure edge" argument.
- (d) Read/write permission; how much is held (R13); value multiplicity (R3).
- (e) P2 (four frees, no double; `A` never held exclusively between
  operations); P8 (no drop flag; the rejection names the path whose state
  differs); pending P18 (P2 with a failing `alloc` arm, under the
  allocation-failure form settled by the owner on 2026-09-16, VERDICT-D0
  decision 5); pending P12 (DMA escrow completion). Metric: no runtime value
  not bound in source selects a release, tested by source-to-IR provenance,
  never by an IR bit count.
- Price: no release inserted or removed; no release whose execution depends on
  runtime state the source did not branch on; a release may move only where
  R4/R7 facts make the move unobservable.
- Cost: ~1.5k / one release op per resource / 1, local. Status: green (LIV-1,
  PROV-6). Outcome arms multiply the paths this row discharges on; charged to
  M10.

#### R3 Value classes and transfer (KEPT)

- (a) Copy values duplicate freely with no obligation; affine values move at
  most once and may take a compiler-derived release; linear values are
  consumed exactly once. A move transfers a value with its obligations; a take
  leaves a hole; a put fills one; a replace exchanges.
- (b) Each value's class; each transfer's source and destination storage.
- (c) At the transfer.
- (d) Pointer validity (a value leaving `'a` is an event on `'a`'s state that
  R1 consumes); the calling convention (B's per-class bytes-or-handle price
  refused: the fact needed is address stability, which is R8a's).
- (e) P7 (take through `p` leaves a hole visible through `q`; put through `q`
  restores `p`; replace reads new). The diagnostic distinguishes "used twice"
  (R3) from "never released" (R2). The multiplicity lemma quantifies over
  transfers, separately from R1's state lemma; M6 composes them.
- Price: none.
- Cost: ~1k / one keyword per transfer / 1, local. Status: green.

#### R4 Licensed facts for the backend (RESTATED)

- (a) Soundness: every fact the lowering uses without a guard (two accesses
  disjoint; a storage unwritten over a span; a loaded value invariant over a
  span; a call removable or reorderable) is true in every execution R14's
  model admits for the accepted program; a fact licensing call removal or
  reordering carries the callee's termination (RAD-D0-22). A missing fact
  costs speed, never correctness.
- (b) Distinctness by identity, field, proved index or proved range, as a
  proved disjointness relation over a containment structure of identities,
  never inequality of identity names (P2: the arena `S` and the live block
  `b1` overlap); write footprints over spans; for a removable call, a
  termination witness. This is the relation R5 reads; R4 tolerates a missing
  "distinct", R5 does not (coupling C2).
- (c) At lowering, from checked facts of the accepted program.
- (d) Safety of the source (R1, R5, R11); profitability and channel
  granularity (M9); whether the backend may add facts from its own sound
  analysis (RAD-D0-25), a `design/compiler` decision settled by the owner on
  2026-09-16 (VERDICT-D0 decision 3); which IR attributes carry a fact
  (`design/compiler`).
- (e) P5: the source carries a checkable distinctness fact for every pair of
  column accesses, and nothing in the language forbids its retention to
  lowering; P1: the two taken parts are distinct; P2 negative: no distinctness
  between `S` and `b1`. Theorem shape Facts(P) ⊆ Truths(⟦P⟧), exercised by
  M6(i). Emitted-attribute counts are compiler tests under `design/compiler`,
  not this row's test: the emitter emits no alias promises today.
- Price: forbids emitting a fact the source did not check and requiring
  re-derivation to reach a stated fact; says nothing about extra analysis, a
  `design/compiler` decision settled by the owner on 2026-09-16 (VERDICT-D0
  decision 3).
- Cost: 0 spec beyond R5's footprint vocabulary / 0 / 0. Status:
  unmeasurable (no channel; section 8 Q15).
- Disputed: standing as a language requirement touches `design/log.md:168`,
  settled by the owner on 2026-09-16 as a requirement with a soundness clause
  whose channel stays compiler detail (VERDICT-D0 decision 1); B reverses
  that ruling and tests on IR attributes (recorded as a pending amendment,
  not adopted; superseded by decision 1); C merges R4 into R5 on M5's
  erasability, which fails at spawn/join, two rows settled by the owner on
  2026-09-16 (VERDICT-D0 decision 2). Majority (A, B; all three judges):
  separate rows, soundness clause kept, the per-load sufficiency clause moved
  to M9 as a measured property.

#### R5 Data-race freedom and permitted overlap (RESTATED)

- (a) A data race is unrepresentable on every access outside R14(iv)'s
  declared foreign access class. Two statements, iterations or threads may
  overlap only on one of three grounds: (i) their footprints are proved
  non-interfering; (ii) they interfere only on a recombinable accumulator
  whose operator law is a fixed-table fact or a written finite proof
  (associative for a fixed combine tree; associative and commutative for
  unspecified order); (iii) the construct carries a named weaker level from
  R14(ii)'s closed set, under which no law is claimed (float reductions).
  "Declared" appears nowhere: an undischarged law refuses the overlap, never
  the program. At an erasable construct (PAR-1, PAR-2) a missing fact leaves
  the program sequential (M5); at a non-erasable construct (spawn/join) a
  missing fact rejects.
- (b) Read and write footprints per statement over storages; the R4
  distinctness relation, read exactly; data dependencies; for (ii) the law and
  its discharge; for (iii) the level; when threads exist, the sync edges
  (R14(i)).
- (c) PAR-1 windows and PAR-2 loops today; spawn/join, acquire/release and a
  reduction's combine point when those constructs exist.
- (d) Sequential safety (R1); defining the level set (R14); acceptance (M5);
  schedule, tile and worker count (Price).
- (e) P1 (two writes overlap); P3 (parallel map by node distinctness); P5
  (PAR-2 overlap); P6 (overlap from range arithmetic and the helper's
  interface alone); compute-bench accumulator snapshot cases under ground
  (ii). Corpus: every PAR-1/PAR-2 overlap accepted today stays accepted or
  the loss is listed. P10 deferred to the thread trigger. Pending P15: one
  reduction under a checked law, one under a named float level.
- Price: forbids reordering across a stated sync edge and picking a level
  weaker than the one written; a fixed-tree level fixes the combine order;
  nothing else about scheduling.
- Cost: ~3k / one footprint clause per interface, one index proof where two
  footprints share a container / 1–2, local. Status: green for (i) on the
  corpus; red for (ii) (checked-law channel is an experiment) and (iii)
  (CAP-1 fixes one guarantee; the closed level set of R14(ii) settled by the
  owner on 2026-09-16, VERDICT-D0 decision 8).
- Disputed: B's closed three-level menu inside R5, with its middle level a
  schedule commitment; C's merge with R4. Majority: the grounds here, the
  level set in R14.

#### R6 Frame: proof-fact retention (RESTORED as a measured precision row)

- (a) Precision: after a write or a call, no fact is dropped whose support the
  operation's stated footprint does not name, and the writer can compute from
  the footprint which facts survive. Soundness (no fact survives an operation
  that may have falsified its support) is M6(ii), not this row.
- (b) The written footprint of each operation, keyed as facts are keyed.
- (c) At commits and call boundaries.
- (d) Safety (M6(ii)); permission (R5); choosing identity granularity
  (coupling C1).
- (e) P3 (the invariant "next/prev point to live nodes" stated once and reused
  across four link writes and a removal); a P9 `reserve`/`drain` variant: a
  call whose interface writes elements of `v` leaves `len(v)` facts standing.
  Metric: facts killed whose support the stated footprint does not name = 0
  over P1–P9. A killed fact's diagnostic names the killing operation. Current
  red value: 34 of 41 `len()` rebinds in the blind-writer corpus exist only
  because ENT-5 kills at a whole-parameter footprint.
- Price: none on the backend.
- Cost: ~1k / 0 when precise, one round per imprecision / 0–1. Status: red.
- Disputed: A and judge 2 retire the row (soundness to M6(ii), precision to
  P1/P4); judges 1 and 3 restore it as the owner of the largest measured
  writer tax. Majority: restored; settled by the owner on 2026-09-16
  (VERDICT-D0 decision 13).

#### R7 Compositional call judgement (RESTATED; FN-1 reclassified)

- (a) A call is judged from a stated interface of the callee; every accepted
  callee body satisfies its interface; the vocabulary can state, per storage:
  entry and exit states (hole, ended, replaced backing), a returned interior
  pointer, a fresh allocation, read/write/end footprints, outcomes (R12),
  resource effects (R13), ordering edges (R14 when threads exist). A
  declaration whose entry condition is syntactically unsatisfiable, or whose
  result names an origin no caller can name (the provenance case, HIS-REQ-06),
  is rejected at the declaration. Provisional clause (THE-REQ-05): the stated
  interface is sufficient for a reader who never sees the body.
- (b) Storage identities nameable at the interface; entry and exit states per
  identity; footprints per identity; existential or fresh results.
- (c) Caller at the call; callee once at its exits; declaration at the
  declaration.
- (d) Whether the interface is written or inferred-and-pinned (FN-1 is one
  mechanism); recheck cost (M10); trusted imports (M3); inferring anything
  from bodies.
- (e) P9 (`drain`, `reserve`, `pick` each visible in the interface; caller
  judged from it; body checked once); P6 (the helper's interface alone tells
  the caller what it writes). Mechanism-neutral metric: the caller's verdict
  is a function of the call site plus a bounded artifact whose token count is
  stated. Compositionality theorem shape under M6. Provisional test for the
  auditability clause: diff size per change on the corpus with no body opened.
- Price: caller acceptance never depends on a callee body beyond the
  interface; nothing about inlining or LTO.
- Cost: ~2.5k / one clause per state-changing effect per interface / 1, local
  at the interface. Status: green for today's rows; red for state clauses
  (section 8 Q5).

#### R8a Storage-ending and relocation events (SPLIT from R8)

- (a) Every value lives in a stated placement (frame, heap, arena, slab,
  inline in a container). Every operation that ends, moves or replaces
  storage (relocating move, reallocating push, arena reset, scope exit,
  `own` parameter passing, container insertion, enum payload move, return,
  and relocation by an R14-admitted non-program actor) is a storage-ending
  event R1 consumes and R7 can state; the set is complete and
  contract-visible (section 8 Q1). Interior pointers name the storage that
  existed before the event; an interior pointer into storage a non-program
  actor may relocate is refused unless the candidate supplies a
  contract-visible fixup form.
- (b) Which operations end or replace which storage; whether aggregates
  travel as bytes or handles (coupling C4, not a mandate); a container's
  backing identity.
- (c) At the relocating operation, as an R1 state event and an R7 interface
  clause.
- (d) Who may access; layout, alignment, address space (R8b); ordering (R14).
- (e) P4 (`push` may reallocate; the second read is decided by state and
  proof, not by a borrow that blocks `push`); P3 (pool slot reuse invalidates
  every path to `m`); P2 (arena bytes reused by `b4`); P9 `reserve`. Test: the
  storage-ending set enumerated against P2/P3/P4/P9. Pending P13: storage
  relocated by a compacting third party.
- Price: the compiler relocates no storage while an interior pointer names it
  (no moving collector, no address-changing copy) unless the fixup form is
  used; free otherwise.
- Cost: ~1.5k / one interface clause per relocating op / 1, local. Status:
  green for explicit moves; red for conditional reallocation contracts.

#### R8b Placement and physical demands (SPLIT from R8; provisional)

- (a) A writer may demand of a placement: alignment; layout (field order,
  padding, SoA/AoS, cache-line placement); the representation a foreign
  boundary sees; an address space (the LLVM cross-space non-aliasing claim is
  unverified and carried as a flag); and that a marked store executes even
  when its value is never read. Every accepted lowering honors a stated
  demand. Constant time and register-level erasure are out of scope until
  R14(vi) carries an observation model.
- (b) The demand vocabulary; which demands are portable and which
  target-conditional; target-conditional demands are backend facts that never
  touch acceptance (M1).
- (c) At the placement declaration; at lowering, as a conformance check on
  the IR.
- (d) Aliasing (R4); ordering (R14); any layout nobody stated.
- (e) P5 with a demanded column alignment: the alignment is present in the
  lowered IR after optimization; zlib-core-kernels `align(64)` on the hot
  table reaches the IR; pending: a key-buffer erasing store before `free` in
  P2's shape is present after optimization. A foreign layout is compared
  against an enumerated M3 entry, never assumed.
- Price: forbids reordering fields or choosing padding only where a layout is
  stated; forbids dead-store elimination of a marked store; forbids assuming
  cross-space non-aliasing; nothing where no demand is stated.
- Cost: ~1.5k when used, 0 otherwise / one demand per placement / 1. Status:
  provisional (no demand in P1–P10); admission as a merged row with pending
  programs settled by the owner on 2026-09-16 (VERDICT-D0 decision 7).

#### R9 Shared mutation among several holders (RESTATED as shape admission)

- (a) Several long-lived pointers to one storage, each writing occasionally,
  is a writable shape. Sequentially the hazard is R1's alone; licensed facts
  weaken under R4 where holders coincide. Across threads the shape is
  admitted only once a thread construct and R14(i)'s ordering vocabulary
  exist (trigger); what the checker knows before and after acquire is then
  R14's to state. Whether a global writable storage is a holder is not
  decided here: the live ban (`design/language/ownership.md:5`, grounded on
  parallel permission itself) stands, settled by the owner on 2026-09-16
  (VERDICT-D0 decision 10).
- (b) Sequentially: nothing beyond R1 and R6's precision (the identity
  model's estimate, not a rule). Concurrently: which primitive holds the
  storage's state facts while no thread holds them.
- (c) Sequentially each access; concurrently acquire and release.
- (d) Forbidding aliasing to strengthen R4 (that trade is R4's and local to
  coinciding identities; R9 is the negation of R4's exclusivity on exactly
  those storages); synchronization cost placement (M9, measured on P10);
  global-state policy.
- (e) P3 (four link writes through several paths accepted sequentially); P7
  (two aliases to one slot). Negative test: a design that rejects P3's
  sequential link updates fails R9. P10 deferred with its trigger; "sync cost
  only at the write" is an M9 measurement, not a clause.
- Price: forbids inserting a lock, fence or generational check on a hold or a
  read the source did not synchronize (an M2(ii) instance).
- Cost: ~0.5k / 0 sequentially / 1. The cost C's "0 sequentially" hid is
  named: loss of the R4 distinctness fact and widened R6 kills for the
  coinciding storages; P3/P7 report the retained-fact count under the shape.
  Status: red (OWN-5 rejects the shape; HIS-REQ-01, HIS-REQ-12).

#### R10 External resources as ordinary objects (KEPT)

- (a) Files, sockets, mappings, devices and descriptors obey the same state,
  effect and proof rules as memory; no rule of the acceptance relation is
  conditioned on whether an implementation crosses the host boundary; what an
  external actor may change between operations is an ordinary contract clause
  (the interference qualifier of R14(iv)).
- (b) One identity per external resource; its states; which operations
  transition them; what an external actor may change between operations.
- (c) Each operation, through ordinary contracts (R7).
- (d) Host mechanisms or scheduling categories (effects.md); where the trusted
  adapter lives (M3).
- (e) Theorem shape: the acceptance relation has no premise mentioning host
  crossing; metric: trusted declarations are counted under M3, not found by
  grepping "host boundary". Pending P14: a descriptor as three identities
  (close-obligation wrapper, open-file description with cursor, foreign
  contents) with two wrappers sharing one description and a short-read
  outcome (R12) on the same path.
- Price: forbids a compiler-special path for I/O values.
- Cost: 0 extra spec beyond one interference qualifier / contract clauses as
  for memory / 1. Status: provisional (P14 pending); THE-REQ-14 settled by
  the owner on 2026-09-16 (VERDICT-D0 decision 15): the adapter is an
  ordinary M3 entry.

#### R11 Partial operations and overflow (ADDED)

- (a) Every operation with a domain (index, division, narrowing, arithmetic
  under an overflow behavior explicitly selected per operation site, cast,
  hardware operation with preconditions) executes only inside its domain;
  silent overflow is unrepresentable; outside the domain the source either
  wrote a typed outcome (R12) or the program is rejected, never repaired with
  an executable fallback.
- (b) The domain of each partial operation; a discharge for it (a fixed
  automatic family or explicit steps).
- (c) Each partial operation.
- (d) Memory state (R1); the discharge family (a mechanism under M1); the
  global progress lemma (M6).
- (e) Pending P17: a division by a runtime divisor, a narrowing cast, an index
  against a runtime length, each accepted only with a proof or a written
  outcome, the diagnostic naming the missing fact. P5: `f` and `g` overflow
  behavior selected per site. (P1's `v[k]` is an R1 hole-read precision case,
  not this row's.)
- Price: none.
- Cost: 0 extra spec (live OP rules) / one proof or arm per partial site / 1,
  local. Status: green.

#### R12 Failure paths and typed outcomes (ADDED; scoped)

- (a) Every operation the specification classifies as fallible (expected
  input and environment failure: short read or write, interruption, device
  error; allocation failure settled by the owner on 2026-09-16 as an R14(v)
  exhaustion with a defined stop, VERDICT-D0 decision 5, so this row's
  allocation clause stays red) yields a typed outcome on a source-visible
  path; every R2 obligation discharges on each arm; a context may forbid a
  failure outcome (no-allocate as "the allocation-failure outcome is
  unreachable here") and the checker enforces it. An unproved domain is a
  rejection (R11): this row is not global prove-or-handle (HIS-D0-04,
  HIS-D0-05, HIS-D0-07, HIS-D0-10 stay refused). Blocking and latency are not
  failures (R10(d): host mechanism categories refused).
- (b) The outcome type of each fallible operation; the failure edges each arm
  adds to R2's join; the context's failure prohibitions.
- (c) At the fallible operation; at every join the outcome creates; at the
  context boundary.
- (d) The outcome's spelling or routing (CALL-6 is one mechanism; HIS-REQ-08
  is a live decision not re-selected here); exhaustions R14(v) excludes;
  preventing the failure.
- (e) P8 (obligations on every loop exit); pending P18 (P2 with a failing
  `alloc` arm frees nothing twice and leaves `b1`, `b3` owned); pending P14
  short-read arm. Metric: fallible operations in accepted programs with
  neither a proof nor an outcome arm = 0.
- Price: forbids eliding a written outcome arm or inserting one; forbids an
  allocation the source did not write in a no-allocate context.
- Cost: ~1.5k / one arm per fallible op / 1, local. Status: green for the
  outcome model; red on allocation failure (SCOPE-3 leaves heap exhaustion
  outside the outcome model with an abort record; the arm doubles every
  construction site, a corpus rewrite no draft priced).

#### R13 Resource bounds where demanded (ADDED; opt-in)

- (a) Where a use declares a budget or a promise, the checker proves it:
  allocation bytes per tangible resource (heap, arena extent, descriptors),
  a declared recursion depth as a count, and termination. Deadlock freedom
  (lock order) enters with the thread trigger; deadlines and machine-stack
  bytes are refused at this decision point (decided after erasure; R14 has no
  timing model). Unpromised programs pay nothing; unintended nontermination
  may remain (constitution).
- (b) A cost semantics per promised resource; a deterministic bound
  derivation; a termination measure supplied by the writer.
- (c) At the promising declaration; per call through R7 interfaces.
- (d) Making every program bounded (the constitution's clause is conditional:
  "for uses with explicit resource budgets"); the accounting mechanism
  (envelope, extent items: HIS-REQ-05, not selected); termination-for-removal
  (R4); performance (M9).
- (e) Pending P18: P2 with a declared byte budget for `A`, `b4` fits or the
  program is refused; pending P19: a declared depth count with a termination
  promise. Metric: the measured peak never exceeds the derived bound, slack
  ratio stated.
- Price: forbids introducing allocation or stack growth where a bound is
  promised (no compiler heap temporaries).
- Cost: ~2k when used, 0 otherwise / budget clause plus proof steps / 2–3.
  Status: provisional; stack exhaustion's owner is R14(v), settled by the
  owner on 2026-09-16 (VERDICT-D0 decision 6).

#### R14 Stated execution model and external conditions (ADDED)

- (a) The specification states the enumerated assumption set every safety
  theorem rests on, and nothing else: (i) the memory model and
  synchronization semantics, stated before any thread construct is accepted
  (trigger); (ii) a closed determinism-level set per overlap construct:
  source-order equality; proved-law reordering (fixed tree or unspecified
  order); a named weaker level with no law. The reference behavior of a
  construct at a weaker level is the set its level admits ("one specified
  behavior set", the restated HIS-D0-16); (iii) what non-program actors
  (allocator, OS, DMA device, another thread, a debugger) may do to which
  storage and when, including the runtime's rights over ended storage; (iv)
  foreign-writable identities carry a declared access class with defined,
  unordered, non-UB semantics, excluded from R5's race theorem; no contents
  fact about them survives an operation the agent could have interleaved;
  (v) the excluded exhaustion set (heap exhaustion, stack exhaustion, OS
  quotas), each with a defined stop matching SCOPE-3 today, so no accepted
  program has an undefined path at exhaustion; (vi) the observation model
  under which erasure and timing demands are judged (deferred; R8b's
  constant-time clause waits on it). Anything outside is excluded, never
  undefined.
- (b) The enumerated assumptions; a per-storage interference qualifier (its
  spelling is open question A); the level per construct; the access class per
  foreign identity.
- (c) In the specification; M6's theorem takes exactly these assumptions.
- (d) Selecting the model or a level; host mechanism categories (R10); a
  debugger's admission and what it may observe (open question B).
- (e) M6's statement lists no assumption absent from R14; every P-program's
  external actors are named; P2: allocator metadata written inside freed `b2`
  is admitted under (iii); P10 (deferred): the level is named; the emitter's
  abort edges on enum discriminants and allocation refusal are listed under
  (v) or removed.
- Price: forbids weakening a stated happens-before; the backend may implement
  it with any target primitive (the fence set is free; strengthening on TSO
  is legal).
- Cost: charged to M8; the ordering text is LKMM-scale and is the largest
  single item this list adds / 0 / 0. Status: red (no model stated; CAP-1
  fixes one level); (v)'s stack-exhaustion entry and (ii)'s closed level set
  settled by the owner on 2026-09-16 (VERDICT-D0 decisions 6 and 8).

#### R15 Abstraction (ADDED; scoped)

- (a) R1–R14 hold for generic, closure-carrying and nominal-parameterized
  code, the forms the language admits (dynamic dispatch and separately
  compiled modules are not in the language and are not quantified over);
  identity, state and effect facts cross abstraction boundaries; generic and
  higher-order definitions are checked once for all instances and
  instantiation is decidable by a fixed rule (HIS-REQ-07's cycle check is the
  current mechanism).
- (b) Identity and effect parameters on abstractions; the bound vocabulary;
  the decidability rule.
- (c) At instantiation and at every abstraction boundary.
- (d) Monomorphization versus dictionaries; signature size (M8 measures it).
- (e) Pending P16: P3 generic over the node type, P5 over the column type,
  P6's helper as a closure, P9 `pick` through a nominal. Metric: parameters
  per interface over generic P3/P5 ≤ a stated function of the storages the
  callee touches (not a linear-in-parameters tautology).
- Price: forbids losing a stated fact at an abstraction boundary; permits
  specialization.
- Cost: ~2k / one parameter per abstracted identity or effect / 1–2. Status:
  provisional (P16 pending); admitted now because a candidate scored on the
  first-order fragment may be unwritable under generics.

### Meta requirements M1..M11

#### M1 Specified deterministic acceptance (RESTATED)

- (a) Acceptance is a total function of the source bytes and the
  specification, computed by a specified terminating procedure; no order,
  budget, machine speed, solver state or state outside the specification
  selects it; a fixpoint terminates by lattice height, never by an iteration
  cap; every admitted automatic family runs to its specified completion
  under its M10 bound.
- (b) Each family as a decision procedure with a stated bound; the
  certificate form (written finite steps the checker verifies without
  rediscovery). A certificate-producing search may exist as a writer-side
  tool; the checker verifies only written steps and fixed families.
- (c) Every acceptance decision; the conformance corpus.
- (d) Latency (M10); repairability (M7). "No SMT" (HIS-D0-02, adopted) is
  the mechanism serving this row and M8.
- (e) The corpus on two machines and under a 10x CPU throttle yields
  byte-identical verdicts and diagnostics; no acceptance-affecting knob
  exists in the compiler.
- Price: forbids budget-selected acceptance; nothing of the backend.
- Cost: 0 / 0 / 0. Status: green (test to be wired).

#### M2 No unstated failure edge, no unchosen cost (RESTATED; two clauses)

- (a) (i) An accepted program has no executable failure edge (trap, abort,
  unwind, UB, silent wraparound) absent from source and interface; every
  runtime condition that can be false is a typed outcome with source-visible
  control flow (R12) or lies in R14(v)'s excluded exhaustion set with a
  defined stop. (ii) No rule of acceptance is satisfied by the presence of a
  runtime comparison: no check is required to reach safety, no bookkeeping
  state exists that the source did not bind, and no lowered branch selects
  between source-visible behaviors on a path the source does not name.
  Backend control flow that preserves the one specified behavior set (loop
  versioning, remainder loops, select lowering) is unconstrained by this row.
  Drop flags, reference counts, generational or epoch checks that safety
  depends on, and a Vale-style check wrapped as a library outcome are refused
  by (ii): the check's condition is a safety predicate the checker did not
  discharge. A writer-written retry loop whose reads are in R14(iv)'s access
  class is ordinary code.
- (b) The failure edges of the lowering and their mapping to source
  outcomes; the safety predicates each rule discharges.
- (c) At the WF→IR boundary (edges and safety predicates), never by counting
  IR branches; at the interface.
- (d) Pricing a written check (M9); which exhaustions are outside the model
  (R14(v)).
- (e) P1: the only branch at the `v[k]` read is the writer's; P4: `free(v);
  c.read()` refused, not trapped; P8: no drop flag, by (ii). Theorem shape:
  abort edges of the lowered program ⊆ image of source outcomes ∪ R14(v)'s
  defined stops. HIS-D0-03 stands; HIS-D0-04, HIS-D0-05, HIS-D0-07, HIS-D0-10 stay refused by (ii).
- Price: forbids inserting any check, trap or bookkeeping the source did not
  write; the floor's precondition.
- Cost: 0 / branches the writer writes / 1. Status: red until R14(v) lists
  the emitter's defensive abort edges.
- Disputed: B keeps the literal "no compiler-inserted check or branch" as the
  enforceable form, which fails every `-O2` lowering (B-cost F1); the
  two-clause form of this text, under which cheap writer-visible dynamic
  checks stay excluded by rule rather than moving to a floor argument, was
  settled by the owner on 2026-09-16 (VERDICT-D0 decision 4).

#### M3 Enumerated, writer-closed trusted base (RESTATED)

- (a) Every assumption the checker does not establish (linked adapters and
  definitions, host declarations, the backend, runtime, loader and OS as
  named axioms, R14's conditions) is derivable from the source tree as a
  list; each entry carries a machine-checked boundary obligation or the named
  evidence obligation standing in for it (M6(iv) for the backend); the
  obligation attaches to every entry regardless of origin, so R10's theorem
  holds, settled by the owner on 2026-09-16 (VERDICT-D0 decision 15); axioms
  enter only through the specification or an enumerated declaration, never a
  writer-reachable form; no discharge is a bare assume (HIS-D0-06 refused).
  "No `unsafe` keyword" is the mechanism.
- (b) The list and each entry's obligation; which emitted facts trace to a
  proof and which to a base entry; backend-derived facts, whose admission
  the owner settled on 2026-09-16 as a `design/compiler` decision (VERDICT-D0
  decision 3), belong to the backend entry if admitted.
- (c) At the boundary declaration; per amendment.
- (d) Semantic non-vacuity in general (a weak `requires` or a defaulted
  result is a logic error the constitution leaves to the requirement author);
  the owner-fixed-interface half (a contract the owner fixed cannot be
  weakened by writer edits) is a provisional clause, settled by the owner on
  2026-09-16 (VERDICT-D0 decision 14).
- (e) The tool prints the base; for the corpus it holds adapters, the
  backend/runtime axioms and R14 conditions only; P9 and P10 add no entry;
  the blind-writer `byte_at` returning `0` outside its range is refused as a
  discharge under the provisional interface clause (VERDICT-D0 decision 14).
- Price: forbids trusting any source-level assertion; every checker-emitted
  fact traces to a proof or a listed entry.
- Cost: ~0.3k / 0 / 0. Status: red (no generated list).

#### M4 Writer regularity (SPLIT; budget → M8, repair → M7)

- (a) One spelling per construct to the byte: the grammar admits one
  production per construct name in a stated inventory; no rule text contains
  a cross-rule exception; every rule is stated once. Adopted mechanisms
  serving this row: one spelling per construct (AI-D0-20) and the closed
  pattern catalog (HIS-D0-14). HIS-D0-21 stays refused.
- (b) The construct inventory; the rule cross-reference.
- (c) Specification lint in `make check`.
- (d) Token budget (M8); repairability (M7); semantic duplicates (two
  constructs with one meaning), which the grammar lint cannot see and which
  is stated as its limit.
- (e) Spelling lint: alternative spellings = 0; cross-rule exception clauses =
  0.
- Price: none.
- Cost: 0 / 0 / 0. Status: red (no lint).

#### M5 Acceptance independent of permission (RESTATED)

- (a) No permission judgment is an input to any acceptance verdict (the
  parallelism tree's wording). For an erasable construct (PAR-1, PAR-2) a
  refused permission yields the sequential program, whose result lies inside
  the construct's promised level (R14(ii)); a non-erasable construct
  (spawn/join) rejects on a missing fact (R5). "Sequential rather than
  rejected" is a chosen degradation policy, named as such.
- (b) Nothing new: the acceptance relation must not mention the permission
  judgment.
- (c) Specification; the checker's dependency graph.
- (d) Performance (M9); the level set (R14); schedules (Price rule).
- (e) Dependency-graph test: no acceptance verdict depends on a PAR judgment
  node; corpus: every program stays accepted with the PAR judgment disabled.
- Price: none.
- Cost: 0 / 0 / 0. Status: green (`design/language/parallelism.md`).

#### M6 Soundness evidence (ADDED)

- (a) (i) A bounded small-model check per discriminating program with stated
  depth and prior criteria (the access-state experiment's form), run before a
  candidate is selected; the operational semantics and the adequacy theorem
  (accepted programs reach no UB, corruption, race, uninitialized read,
  overflow or unproved partial state under exactly R14's assumptions; global
  progress) are a stated later obligation, settled by the owner on 2026-09-16
  (VERDICT-D0 decision 9). (ii) Frame soundness: a fact retained across an
  operation is true after it; a design whose frame under-kills fails P7/P2,
  not merely R6's count. (iii) Erasure: the meaning of an accepted program is
  independent of its proofs and permissions; lowering ignores proof terms
  except through R4's licensed facts; observable behavior lies in R14(ii)'s
  specified behavior set. (iv) A checker-defect evidence channel outside the
  accepted program, never a build mode (HIS-D0-16 restated; HIS-D0-17 stays
  superseded): the facts-withheld differential covers retained-fact
  unsoundness only; acceptance unsoundness needs the small-model check;
  measured as a seeded-defect detection rate.
- (b) The bounded model; R14's assumption list; which facts are retained.
- (c) `research/experiments` per candidate; the compiler's own tests for (iv).
- (d) Freedom from logic errors; a debug build; what a foreign reader may
  observe (open question B).
- (e) Every P1–P9 program under the bounded check; a seeded defect on P5's
  distinctness facts is detected by (iv) at a stated rate. Lemmas composed
  here: R1's state lemma, R3's multiplicity lemma, R4's fact licensing, R5's
  DRF guarantee, R7's compositionality, R11's partial-operation clause.
- Price: none.
- Cost: 0 spec / 0 / 0; a second corpus build per `make check` once a fact
  channel exists. Status: partial (access-state check exists for a fragment).

#### M7 Repairable rejection (ADDED; split from M4)

- (a) Every rejection cites one rule, one tree location and the missing fact
  or the restructuring it demands (for rejections under stated precision
  rules such as reject-when-unsure); its `mechanical_fix` or missing-fact
  payload is non-empty and names a tree location where a local fix exists;
  diagnostics are deterministic, byte-stable and parseable under one grammar;
  no restriction leaves the writer without a taught route (HIS-D0-15 stays
  refused); a repair applied as named does not reintroduce the same rejection
  at the same location.
- (b) Per rule, its repair form.
- (c) Every diagnostic.
- (d) Acceptance; bounding rounds by a writer model (evidence only, model and
  protocol pinned; `design/language.md` decision 5, settled by the owner on
  2026-09-16 as VERDICT-D0 decision 12).
- (e) P1 `v[k]` names `k≠i ∧ k≠j`; P8 names the differing path; R1 names the
  failed sub-property; R3 and R2 are distinguishable; R6 names the killer.
  Metric: payload non-empty for 100% of corpus rejections. Evidence: the
  blind-writer repair-from-diagnostic rate (OWN-6 fails today: three
  coordinate-only diagnostics).
- Price: forbids a rule whose only fix is non-local and unnamed.
- Cost: ~0.5k / 0 / sets the local-fix bound for every row. Status: red.

#### M8 Teaching and retrieval budget (ADDED; split from M4, prelude and retrieval merged)

- (a) The taught surface (specification, pattern cards, any prelude a file
  must re-declare) fits K tokens under a named tokenizer; every rule is
  addressable by ID so one card is retrievable in one lookup (AI-D0-17:
  retrieval, not the window, is the scarce resource); each card is at most a
  stated per-rule size; the fixed per-file cost is at most a fraction f of a
  file; a new row's vocabulary fits within K or displaces an existing row,
  decided per amendment; R14's model text, M6's semantics and R11–R15 are
  charged here.
- (b) Token counts per revision (META-5 practice); the rule-ID index; the
  prelude fraction.
- (c) `make check`, per amendment.
- (d) The checker; how the prelude is delivered (mechanism).
- (e) Tokens ≤ K with provisional K = 48k (why-whitefoot §1); today ≈130k
  (533,492 bytes): red. Rule-ID index exists. Prelude fraction ≤ f,
  re-measured against PRE-1 before the 30–60% figure is cited. Sum of row
  Cost estimates ≤ the ownership subsystem's stated share of K (≈18.5k
  today against an unstated share).
- Price: may forbid a fact vocabulary that breaks K; nothing of the backend.
- Cost: this row prices every other row. Status: red; K, the tokenizer, f and
  the share stay provisional at the values named here, with M11 governing
  every reset, settled by the owner on 2026-09-16 (VERDICT-D0 decision 16).

#### M9 Measured performance floor (ADDED; M5's fast-shape clause made testable)

- (a) On a locked corpus with a named public reference implementation and
  workload per program, the first checker-green artifact with no hints meets
  a stated ratio with all checks in place; the obvious shape of P5/P6 and of
  each corpus program is accepted and within a stated factor of the
  restructured shape (HIS-D0-19 adopted as a measured property, never as a
  rule); the source can license per-access and per-span facts (R4's former
  sufficiency clause), measured as P5 per-load facts; P10's "synchronization
  cost only at the write" is measured here when threads exist.
- (b) Corpus, references, ratio, factor, protocol.
- (c) `research/experiments`, per candidate and per release.
- (d) Acceptance (M5); promising speedup from permission (HIS-D0-20 stays
  refused); schedule languages and cost-model exposure (RAD-D0-12, RAD-D0-24:
  mechanisms).
- (e) Two measurements, named separately: (1) default-floor 1.65x
  (percent-encoding) and 1.10x (utf8parse), retired-compiler evidence whose
  own boundary disclaims that the fact channels caused the win, to be
  re-measured on the current compiler; (2) the obvious-shape factor
  (why-whitefoot §3's 1.6x, open per HIS-REQ-04). The P5 per-load half is
  unmeasurable until an alias channel exists (SYS-REQ-13; section 8 Q15).
- Price: forbids a design whose default-accepted shape is off the floor;
  nothing of the backend.
- Cost: 0 / 0 / 0. Status: unmeasurable on the current compiler.

#### M10 Checking cost, scaling and edit stability (ADDED)

- (a) Each admitted automatic family has a stated worst-case bound as a degree
  in operation counts over spec-defined quantities (premises, entries,
  identities, written steps), under a provisional ceiling: no family above
  cubic in its measure, total work per declaration at most quadratic in
  written proof steps; a family needing more is a recorded amendment naming
  why. Whole-project criterion: a pinned bundle at a stated project size with
  a fitted exponent, never a wall-clock gate in `make check` (M1). Edit
  stability: the set of declarations whose verdict can change under a
  one-token edit is bounded by a stated function of the edit's syntactic
  reach, measured on the checker without assuming certificates or caching.
  The bound is in program size including instantiations, so PROG-1 stays
  (SYS-D0-15). Termination alone is insufficient; never exponential
  (HIS-D0-11).
- (b) The bound per family; the reach function.
- (c) Specification (the bound); measurement against the pinned bundle.
- (d) Budget-selected acceptance (M1); "more annotations make checking
  cheaper" (HIS-D0-21 refused); the incremental architecture.
- (e) proof-use-cost `growing` pinned bundle N = 16…128: baseline 31→164→1690
  ms (N^2.5–3.4); current branch growing-64 ≈ 120–173 ms, growing-128 ≈
  734–1158 ms (exponent ≈ 2.6): red against the ceiling. A one-token edit in
  P9's caller changes the verdict set of a bounded declaration set. Outcome
  arms: path count per arm bounded by the arm count.
- Price: forbids a family whose worst case exceeds its stated degree;
  nothing of the backend.
- Cost: 0 / 0 / 0; a violation is a compiler defect. Status: red.

#### M11 Evolution with stated loss (ADDED; conditional)

- (a) A specification, interface-vocabulary or proof-form revision states the
  verdict diff over the retained corpus and either a migration or the loss; a
  mechanical migration tool is required only once a project declares a
  compatibility need (the constitution's clause is conditional); explicit
  steps survive a revision that does not change their rule; META-5 token and
  spelling deltas are counted per amendment; every threshold in this list (K,
  f, ratio, factor, degree) is reset only through an amendment recorded here.
- (b) The conformance-corpus verdict diff; the deltas.
- (c) Per amendment.
- (d) Backward compatibility as a hard rule.
- (e) Verdict diff present; META-5 counts present; where a tool ships, manual
  edits ≤ a stated count relative to corpus size (never "or the count is in
  the PR").
- Price: none.
- Cost: ~0 / 0 / migration rounds counted per revision. Status: green.

Row count: 16 R rows (R1–R7, R8a, R8b, R9–R15) + 11 M rows = 27.

## 2a. Premises

Premises the pre-verdict list carried as meta-constraints or as unstated
assumptions, reclassified by the verdict (its section 5). Six were
reclassified: FN-1 and PROG-1 wholly to mechanism; M1, M2, M3 keep a
requirement with the mechanism clause split out and named; M5 split into
requirement, goal and policy.

| Premise | Classification | Serves | Reason; what changes if dropped |
|---|---|---|---|
| FN-1 signature-only checking | Mechanism (HIS-D0-08 remains an adopted decision, now placed) | R7 (one way to realize a stated interface), M10 (edit locality: the reach function stops at the interface), M7 (local diagnostics) | The backend and the caller need an interface, not its authorship. Dropped: whole-program inference becomes admissible under PROG-1; R7 still requires a pinned, printed interface for compositionality; M10's reach function is restated without the interface as its boundary. |
| PROG-1 closed world | Mechanism, spent (majority A and B; C's "unspent" is refuted by C's own R7 clause). Two-consumer reading: the checker spends it in M3 (the base is enumerable from the tree), M6 (closed quantification), R5's exact transitive footprint closure (spec line 1902), R7's declaration-side check and R15 instantiation; the backend spends it in devirtualization, cross-declaration facts and monomorphization (M9). HIS-D0-09 remains adopted. | M3, M6, R5, R7, R15, M9 | Dropped: separately compiled interfaces become M3 entries with obligations; M6 gains an "imported interfaces are honored" assumption; R7's interface becomes the only cross-unit fact channel; FN-1's annotation cost would then buy separate compilation, which today it does not (SYS-D0-16's double cost is a mechanism cost, not a requirement conflict). Spent, settled by the owner on 2026-09-16 (VERDICT-D0 decision 11). |
| M1 deterministic, budget-free checking | Requirement (restated as specified total deterministic acceptance). "No SMT" is the adopted mechanism (HIS-D0-02). | M6 (the acceptance relation is a function the theorem quantifies over), M7, M8 | A specification-fixed fixpoint bounded by lattice height qualifies; a writer-side certificate search qualifies; an in-checker portfolio does not. Dropped: acceptance becomes implementation-defined and the theorem loses its object. |
| M2 no runtime safety check | Requirement (restated as two clauses). "No runtime check or trap" is the adopted mechanism reading (HIS-D0-03); clause (ii) is what keeps HIS-D0-04, HIS-D0-05, HIS-D0-07, HIS-D0-10 refused. | M3 (a trap is an unenumerated failure edge), M6(iii), R12, M9's precondition | Dropped: Vale/Mezzo/CHERI-shaped designs re-enter; every interface gains an implicit failure edge; every R4 fact becomes guard-conditional. The two-clause form settled by the owner on 2026-09-16 (VERDICT-D0 decision 4). |
| M3 no unsafe | Requirement (restated as enumerated, writer-closed trusted base). "No `unsafe` keyword" is the mechanism. | M6 (assumptions are exactly M3's list plus R14), R4's facts being non-defeasible, the why-whitefoot floor argument | Dropped: a stuck writer uses the escape; M6's theorem acquires per-site assumptions no tool enumerates; R4 facts become trust-dependent as Rust's `noalias` is under `unsafe`. |
| M5 fast shapes only | Split. Acceptance-independence is the requirement (M5); "accepted shapes are the fast shapes" is a goal measured under M9 (HIS-D0-18, HIS-D0-19); "sequential rather than rejected" is a chosen degradation policy for erasable constructs, named as such. | M5 serves M6 (the theorem is schedule-free); M9 serves the constitution's performance objective | Dropped: acceptance depends on the cost model and the theorem quantifies over schedules; without M9 the floor claim is unmeasured and why-whitefoot §1 unsupported. |

## 3. The hazard ladder

The ladder adds one feature at a time and asks what can go wrong and the least
a checker must know to refuse it. It fixes the minimal information for R1, R2,
R3 and R8 and shows where R4, R5 and R6 first appear. Notation: `ptr<'a, T>` is
a copyable pointer whose type names storage `'a`; `state('a)` is a checker fact
in `{Init, Uninit, Gone}`; owners are ordinary affine bindings. This notation
is illustrative and is not a surface-syntax proposal.

### Step 0: scalars, pointers, sequential execution

```text
(p, own_a) = alloc(10)
q = p
write(p, 1); x = read(q)     // legal, 1: sequential aliased access needs no judgment
v = take(p); read(q)         // reject: read of a hole.      needs state('a)
dispose own_a; read(q)       // reject: use after end.       needs state('a) = Gone
dispose own_a                // reject: double release.      needs an affine owner
```

Precision rather than safety: a write through any pointer to `'a` kills facts
about `'a`'s contents. ENT-5 already kills by support; the support becomes the
identity. No read/write conflict judgment appears at this step.

### Step 1: struct fields

```text
s: S in storage 'a; px = ptr_of(s.x)      // px: ptr<'a.x>
write(s.y, 1); read(px)                   // legal: 'a.x unchanged
v = take(s.x); read(px)                   // reject: 'a.x = Uninit; put restores it
t = move s; read(px)                      // reject: bytes moved to 'b, 'a = Gone, never restored

b = heap_box(...); pc = ptr_of(deref(b).x)
b2 = move b; read(pc)                     // legal: the box moved, its content storage did not
```

The distinction that matters is whether an operation relocates storage, not
whether the value is a struct or a container. A take is a value leaving
storage; a relocating move is the storage ending.

### Step 2: fixed array

```text
pi = ptr_of(a[i]); pj = ptr_of(a[j])
v = take(a[i])                            // state('a[i]) = Uninit
read(pj)                                  // requires state('a[j]) = Init: needs j != i, else reject
```

Index disjointness first appears here, for sequential state precision, before
any parallelism. Without holes it is not needed.

### Step 3: growable vector

```text
v: Vector in 'v; fact backing('v) = 'b
pe = ptr_of(v[i])                         // pe: ptr<'b[i]>
push(v, 5)                                // contract: len0 < cap0 => backing unchanged;
                                          //           else 'b = Gone, backing('v) = fresh
read(pe)                                  // requires state('b) != Gone: provable iff len0 < cap0
```

The container case is Step 0 plus one contract clause: push may end storage,
as free does. Rust invalidates every borrow unconditionally; here the proof of
`len0 < cap0` decides.

### Step 4: calls

Contracts state entry and exit states per identity and which storages end or
are replaced. No loans are needed.

### Step 5: parallel overlap

The read/write conflict table appears here and only here: two overlapping
statements may not write a storage the other reads or writes.

Settled R5 (section 2) admits overlap on three grounds; the conflict table
above is ground (i), and the ladder is otherwise unchanged.

| Ground | Overlap admitted when | Row |
|---|---|---|
| (i) | the footprints are proved non-interfering (the conflict table above) | R5(a)(i) |
| (ii) | the statements interfere only on a recombinable accumulator whose operator law is a fixed-table fact or a written finite proof | R5(a)(ii) |
| (iii) | the construct carries a named weaker level from R14(ii)'s closed set, under which no law is claimed | R5(a)(iii) |

### What each mechanism is for, by the ladder

| Mechanism | Hazard or goal it serves | First needed |
|---|---|---|
| Per-storage state (`Init`, `Uninit`, `Gone`) | hole read, use after end | Step 0 |
| Affine owner binding | double release, leak | Step 0 |
| Storage identity in the pointer type | state stays precise under aliasing; optimizer distinctness | Step 0 |
| Kill facts by identity on write | proof precision, not safety | Step 0 |
| Contract-visible storage-ending operations | relocation, reallocation, free | Steps 1 and 3 |
| Index and range disjointness proofs | state precision with holes in arrays | Step 2 |
| Read and write footprints | parallel permission only | Step 5 |
| Borrow modes, regions, exclusivity | no row above needs them | none |

The last row is the ladder's finding: borrowing is not the mechanism that
refuses any hazard once storage state and identity exist. Its remaining role
is the interference and optimizer facts of R4 and R5, which footprints over
identities can carry, and the default clean shape of M5, which section 7
records as an open coupling.

## 4. Today's mechanisms mapped to the requirements

The columns keep the map's original R1..R10 numbering from before the D0
verdict: the R8 column now covers R8a and R8b, and R11..R15 have no column.
The matrix was not rebuilt; it remains the reading of today's mechanisms
against the original rows.

P marks the requirement a rule primarily serves, S a secondary one. The
bucket column says which of the ladder's roles the mechanism plays:
**perm** exists only to keep permission-carrying references sound, **life**
is lifecycle accounting, **intf** is interference or optimizer facts,
**frame** is fact retention, **class** is value-class semantics, **proof**
is the deterministic proof discipline. Rule text is the authority; the
mapping is this document's reading of it.

| Mechanism | Rules | R1 | R2 | R3 | R4 | R5 | R6 | R7 | R8 | R9 | R10 | Bucket |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Copy or affine classification, explicit `move`, dead-root kill | OWN-1 | S | P | P | | | | | S | | | class |
| Borrow modes `&`, `&uniq` | OWN-2 | S | | | P | P | P | S | | P | | perm |
| Lexical regions, outlives, incomparable caller regions | OWN-3 | P | | | | | | S | | | | perm |
| Named-region liveness, region-checked storing and returning | OWN-4 | P | | | S | S | | S | | | | perm |
| Resolved-place exclusivity and suspension | OWN-5 | | | | P | P | P | | | P | | intf via perm |
| View origin sets; no slice-valued join | OWN-5, VIEW-1, VIEW-2 | S | | | P | P | | P | | | | perm |
| Holder resolution; statement-scoped and candidate-position child reborrow | OWN-6 | | | | S | S | | P | | | | perm |
| Overlap of resolved places; proved range disjointness | OWN-7 | S | | | P | P | P | | | | | intf |
| Reject when unsure | OWN-8 | | | | | | | | | | | proof |
| Optimizer noalias consequence | OWN-9 | | | | P | | | | | | | intf |
| Borrow-storage duration | OWN-10 | P | | | | | | | S | | | perm |
| Loop body as a region; per-iteration liveness agreement | OWN-11 | P | S | | | S | | | | | | perm |
| Call substitution; argument loans; effect projection | OWN-12 | | | | S | P | S | P | | | | intf |
| Match ownership; arm-scoped binder reborrows | OWN-13 | S | S | P | | | | | | | | class, perm |
| Returned reborrow; other reborrow forms deferred | OWN-14 | | | | | | | P | | | | perm |
| Join-checked liveness; unconditional scope-exit release | LIV-1 | S | P | | | | | | | | | life |
| One `set` commit; read-out; no observable hole | LIV-2 | P | S | S | | | | | | | | life |
| Store identity is a region; brand resolution | PROV-1 | | P | | | | | S | P | | | life |
| Linearity by capability presence; `dispose`; release graph | PROV-6 | | P | S | | | | | S | | | life |
| Borrow-free storage | STOR-5 | P | | | | | | | P | | | perm |
| Effect rows: three categories, formal-rooted static paths | EFF-1 | | | | P | P | P | P | | | P | intf |
| Exhibited-row exactness; call-boundary projection; reclamation contribution | EFF-2 | | | | S | P | P | P | | | | intf, frame |
| `pure` licensing | EFF-3 | | | | P | | | | | | | intf |
| No writer-visible capability category | CAP-1 | | | | | P | | | | | | intf |
| Window permission with loans and footprints | PAR-1 | | | | | P | | | | | | intf |
| Counted-loop permission: accumulator, affine element maps, adjacent ranges | PAR-2 | | | | | P | | | | | | intf |
| Support kill at commits and calls | ENT-5 | | | | | | P | | | | | frame |
| Routed relations at calls | CALL-6 | | | | | | S | P | | | | proof |
| Invariants and explicit certificates | ENT, section 15 | S | | | | S | | | | | | proof |

The full 44-row inventory with verbatim rule quotes, dependency lists and a
bucket per row is part A of
[EVIDENCE-mechanism-survey-2026-09-16.md](EVIDENCE-mechanism-survey-2026-09-16.md).
Its count: 19 rows exist only to keep permission-carrying references sound,
6 are lifecycle accounting, 6 interference, 4 fact maintenance, 4 value
class, 5 other.

Two readings of the table:

- Every **perm** row serves R1 only because the reference is also the
  permission: regions, named-region liveness, borrow-storage duration,
  loop-body regions and borrow-free storage exist to stop a permission-bearing
  reference from outliving its storage. Under per-storage state, the storage
  ending is the fact and no reference needs a duration.
- Every row that serves R4 and R5 does so through exclusivity and place
  overlap. The information those consumers actually read, and whether
  identity footprints carry it, is section 5's question.


## 5. Mechanism families per requirement

Numbering note: this section and section 6 were written against the
pre-verdict list and keep its names and numbers (R1..R10, M1..M5); read R8 as
R8a and R8b, and the meta-constraints as their successors M1..M11 in section
2. The families themselves are unchanged by the verdict.

For each requirement, the families known to serve it, the information each
needs, and its fit with M1 to M5. Representative works are named for
orientation; the survey with constraint scoring and citation status is in
[EVIDENCE-mechanism-survey-2026-09-16.md](EVIDENCE-mechanism-survey-2026-09-16.md)
part C, and RESEARCH.md keeps the primary-source table. Fit phrases are this
document's assessment.

### R1 Sequential memory safety

| Family | Representative | Information needed | Checked by | Fit | Cost |
|---|---|---|---|---|---|
| Permission-carrying references with regions or inferred lifetimes | Rust NLL, Oxide; WF today | reference durations, exclusivity | flow analysis over regions or NLL | M1 needs a fixed lifetime solution, which is why WF chose lexical regions; M4 poor: non-local diagnostics; M5 good | stored references, holes and split parts are inexpressible |
| Storage identity in the pointer type plus per-identity state | Alias Types 2000, L3 2005, Verus `PointsTo` 2023, DESIGN.md Candidate A | which storage each pointer names; state per storage; storage-ending events | type equality plus state facts at each access | M1: equality and finite states are fixed families; M2, M3 met; M4: local diagnostics | proof burden where identity is data-dependent: indices, containers, recursive structures |
| Stateful views over a linear context | ATS 2005, Cogent 2016 | view assertions per location, including uninitialized | linear type rules | M1 to M3 met; M4: a second small language | recursive structure needs dependent views |
| Typestate and path-sensitive property automata | Strom and Yemini 1986, Bierhoff and Aldrich 2007, ESP 2002 | finite state per tracked object; permission per reference | finite-state dataflow | M1 met; M5: safety facts only | precision collapses under aliasing without a permission system underneath |
| Region-based memory management | Tofte and Talpin 1997, Cyclone 2002 | region per allocation; outlives | region typing | M1 met; M5 poor: coarse lifetimes leak or copy | no per-object free inside a live region |
| Runtime-validated references | Vale generational references, Mezzo adoption | generation per allocation | runtime compare and trap | M2 violated | disqualified; a baseline only |

### R2 Resource lifecycle accounting

| Family | Representative | Information needed | Checked by | Fit | Cost |
|---|---|---|---|---|---|
| Affine owners with compiler-derived release, static at joins | WF OWN-1, LIV-1; Cogent; Austral | owner per storage; liveness agreement at joins | linear typing over structured control flow | all met; no drop flags | join disagreements are restructured by the writer |
| Linear obligations with explicit consumption where no capability is present | WF PROV-6; Wadler 1990; Linear Haskell 2018 | provider capability in scope | linear typing | all met | explicit routes only |
| Capability-indexed deallocation | Capability Calculus 2000 | linear capability per region, consumed by free | linear typing | all met | region granularity; per-object free needs L3-style locations |
| Runtime drop flags or ownership indicators | Rust MIR, MLIR ownership-based deallocation | none static | runtime flag | M2 violated; WF refused it in LIV-1 | a hidden bit the source could have stated |
| Reference counting with reuse | Perceus 2021 | inferred ownership | RC insertion | M5 violated for ordinary data | its reuse analysis is a useful precedent only |

### R3 Value classes and transfer

| Family | Representative | Information needed | Checked by | Fit | Cost |
|---|---|---|---|---|---|
| Copy, affine, linear classes with explicit `move` | WF today; Austral; Linear Haskell multiplicities | class per type | typing | all met; one spelling per meaning already | none |
| Uniqueness typing for in-place update | Clean 1996, Futhark 2017 | uniqueness per reference | attribute propagation | met; Futhark shows M5 gains for arrays | uniqueness is not an obligation; R2 needs its own rule |
| Explicit take and put on inline storage | Cogent 2016, ATS views | which slot holds a value, which is a hole | linear typing with slot state in the type | met; this is WF's hole-and-refill requirement with no runtime flag | slot state in types grows with nesting |
| Handle passing for aggregates, storage fixed for life | why-whitefoot section 10 pools; Hylo `inout` conventions | storage identity stable across calls | representation choice plus contracts | met; enables interior pointers across calls | contracts must say whether the callee ends the storage |
| Ghost permission separated from the runtime value | Verus `Tracked<PointsTo>`, L3 | pointer plus a separate linear permission | linear checking of the ghost term, erased | met for the linear part | permissions threaded by hand through every call |

### R4 Aliasing facts for the optimizer

| Family | Representative | Information needed | Checked by | Fit | Cost |
|---|---|---|---|---|---|
| Exclusive references lowered to `noalias` | Rust `&mut`; WF OWN-9; Stacked and Tree Borrows as the model | live exclusive loans | borrow checking | met; parameter granularity only; loaded pointers unmarked | the gap why-whitefoot section 5 measures |
| Per-location store typing | Alias Types, L3, Capability Calculus | a name per storage; distinct names are disjoint | syntax-directed | met; the strongest per-storage facts; per-span `alias.scope` possible | location polymorphism and existential packing at structure boundaries |
| Region and effect annotations on code | DPJ 2009, Lucassen and Gifford 1988, Regent | region partition; effect summary per call | effect subsumption and region disjointness | met if the index fragment is fixed | array partitioning needs arithmetic disjointness, where solvers usually enter |
| Uniqueness and purity | Clean, Futhark, Cogent | uniqueness; purity | syntax-directed | met for arrays | weak for pointer graphs |
| Proof-derived separation exported to the backend | RefinedRust 2024, Verus, Iris | separation assertions | tactics or SMT | M1 violated by the derivation; usable only with fixed fact forms and explicit steps | non-local failures |
| Compiler-side recovery | LLVM alias analysis, loop versioning | none | analysis under guards | contradicts M5's premise | guards and code bloat |

### R5 Data-race freedom and parallel permission

| Family | Representative | Information needed | Checked by | Fit | Cost |
|---|---|---|---|---|---|
| Loans plus effect footprints over resolved places | WF PAR-1, PAR-2 | loans, rows, place overlap, affine index images, range proofs | fixed judgment per window or loop | met today | loans restate what rows and identities say |
| Footprints over storage identities with distinctness facts | DPJ 2009, Regent's static part, the CSL parallel rule (O'Hearn 2007) | identity footprints; identity distinctness; index or range proofs | the same fixed judgment with identities as roots | met if identity parameters are distinct by default or proved | a proof where two footprints come from one container |
| Fractional or counting permissions | Boyland 2003, Bornat et al. 2005, Chalice, Viper | permission amount per location | entailment; SMT as deployed | M1 needs a fixed split discipline; the counting form is deterministic | verbose bookkeeping |
| Sendability by capability | Pony 2015, Rust `Send`/`Sync` | capability per type | typing | met; thread-level only | a capability lattice to teach |
| Runtime dependency tracking | Legion runtime | none static | runtime | M2 and M5 violated for permission | scheduler cost |
| Purity and uniqueness | Futhark, Cogent | purity; uniqueness | syntax-directed | met for data-parallel shapes | no general statement overlap over mutable structures |

### R6 Frame: proof-fact retention

| Family | Representative | Information needed | Checked by | Fit | Cost |
|---|---|---|---|---|---|
| Kill by written footprint keyed as facts are keyed | WF ENT-5; Dafny `modifies` as notation; Low* `modifies loc h0 h1` | write footprint per commit and call | deterministic kill | met; the key becomes the identity | precision follows identity granularity |
| Ownership-derived framing | Prusti 2019, Flux 2023, Creusot 2022 | the ownership structure already present | framing from types; only value facts to a solver | the framing half is deterministic; WF keeps that half and replaces the value half with explicit steps | shared state falls back to explicit permissions |
| Store typing as the frame | Alias Types, L3 | full store type before and after each operation | syntax-directed rewriting | met | unaffected memory is still named; needs abstraction for large modules |
| Separation-logic frame rule | Reynolds 2002, O'Hearn | footprint as a separating conjunct | structural when separation is syntactic | met when separation is by distinct names; search otherwise | inductive predicates need explicit fold and unfold |
| Implicit dynamic frames and region logic | Smans et al. 2009, Viper; Banerjee et al. 2008 | accessibility predicates or region expressions | verifier | M1 violated as deployed | solver |

### R7 Modular, signature-only checking

| Family | Representative | Information needed | Checked by | Fit | Cost |
|---|---|---|---|---|---|
| Store-type pre and post in the signature | Alias Types, Capability Calculus | store-in and store-out, including holes and vanished locations | syntax-directed subsumption at the call | met; the signature literally says which storage becomes a hole or ends | signatures grow; location polymorphism and existentials are mandatory |
| Requires and ensures over identities and states | Vault 2001, Mezzo `consumes`, Verus, DESIGN.md Candidate A | identity parameters; entry and exit states | signature check at call and return; routed states reuse CALL-6 | met | every callee that changes storage state must say so |
| Modifies and reads clauses as notation | Dafny, Low* | footprint sets, `fresh`, `old` | SMT frame axioms as deployed | the notation fits M4; the discharge violates M1 and must be replaced by a footprint algebra | none if the discharge is syntactic |
| Lifetime-parameterized signatures returning interior pointers | Rust elision, WF FN-1 provenance, OWN-14 | which parameter a result borrows from | region inference or provenance rules | met, but expresses durations only, never holes or frees | a rule per case |
| Prophecies or backward functions for mutable borrows | RustHorn 2020, Creusot, Aeneas 2022 | final value of a borrow | solver or translation | M1 poor; unnecessary when access checks state per operation | complexity |
| Existential identities for fresh results | Alias Types `exists`, DESIGN.md `exists P` | the pack | pack and unpack | met | an unpack step at the caller |

### R8 Storage placement and relocation

| Family | Representative | Information needed | Checked by | Fit | Cost |
|---|---|---|---|---|---|
| Storage classes with explicit placement rows | WF STOR-*, BLK-2 | placement per value | typing | met today | none |
| Relocation as a name change | L3 `realloc: cap(ρ) -> exists ρ'. cap(ρ')`; this document Steps 1 and 3 | old and new storage identity; the transfer between them | type rule plus contract clause | met; the most explainable form; makes push conditional on `len < cap` | every derived pointer must be re-derived after a relocation |
| Take and put on inline storage | Cogent, ATS | which inline slots are full or empty | linear typing | met; inline-in-container without a hidden tag | slot state multiplies with nesting |
| Projections for interior access | Hylo subscripts, Swift accessors | the projected path and its span | static exclusivity over paths | M2 partially: Swift falls back to dynamic checks for class storage | interior pointers may not escape the projection |
| Pinning | Rust `Pin` | a pinned flag | typing | expresses less than a state event | idiom burden |
| Handles instead of interior pointers | why-whitefoot section 10 | index plus bounds proof | bounds proofs | met; already taught | one indirection |

### R9 Shared mutation among several holders

| Family | Representative | Information needed | Checked by | Fit | Cost |
|---|---|---|---|---|---|
| Sequential: state per storage, aliasing allowed | Candidate A; L3 unrestricted pointers | state per identity | R1 and R6 | met; nothing further needed | facts weaken where identities coincide |
| Identity and permission separated by a brand | GhostCell 2021 | a brand shared by cells; one token whose borrow grants access to all of them | ordinary type check on the token | met at zero runtime cost | brand-coarse: no concurrency inside one brand |
| Lock owns the storage's state while unlocked | CSL resource invariants, Chalice monitors, Verus locks | lock invariant naming identities | acquire yields facts, release requires them | met when threads arrive; cost only at acquire | lock cost at the write, none for the hold |
| Ghost protocols over shared state | Iris STS, fictional separation (Jensen and Birkedal 2012), CAP 2010, Verus state-machine sharding | per-role transitions | ghost proofs | M1 needs a fixed protocol checker | proof burden; a second sub-language |
| Manifest sharing on channels | Balzer and Pfenning 2017 | acquire and release types | session typing | acquire is a typed outcome with real control flow, which matches M2 | interaction with R4 across an acquire is not worked out |
| Fine-grained access permissions | Bierhoff and Aldrich 2007 | permission per reference | typestate checking | expressible as footprints plus a read-only flag | five modes to teach |
| Interior mutability with runtime flags | Rust `RefCell`, `Cell` | none static | runtime flag | M2 violated; why-whitefoot rejects the hole | every fact conditional |

### R10 External resources as ordinary objects

| Family | Representative | Information needed | Checked by | Fit | Cost |
|---|---|---|---|---|---|
| External state as identities with declared states and opaque contents | Low* external state, VST external state, Iris external resources | identity per resource; state machine; opacity of contents | contracts | met; no special rule per resource | contracts per operation |
| Capability tokens for I/O | Effekt, Scala capture checking | capability in scope | typing | expressible as an identity in scope | a second vocabulary |
| Effect categories for host mechanisms | early WF `external`, `blocks` | mechanism labels | rows | rejected by effects.md: mechanisms, not state | none |

### The three decouplings the literature already made

Every design above that scores well on M1 to M5 makes one of three cuts,
and the cuts are what the hazard ladder found independently:

| Cut | Where it is made | What it buys | What it costs |
|---|---|---|---|
| Pointer from capability | Alias Types, L3, Verus `PointsTo` | copying a pointer stops implying R1 or R4 consequences; R7 signatures can say "ends `'a`, produces `'b`" for reallocation | the capability is threaded or looked up; recursive structures need packing |
| Identity from permission | GhostCell | R9 at zero runtime cost; R4 survives because the permission is still exclusive | permission granularity is the brand's |
| Effects from ownership | DPJ | R4 and R5 are checked from effects on code rather than ownership of values; the object model stays conventional | effect annotations restate information the type system nearly has |

The decomposition in section 6 makes all three cuts at once: the pointer
carries identity only, permission is the storage's state and the operation's
footprint, and interference is judged from effects over identities.

Two further findings from the survey bear on M1:

- In Prusti, Flux, Viper and implicit dynamic frames, the framing and
  permission half is decidable accounting while the value half goes to a
  solver. That split is where WF's automatic and explicit boundary belongs:
  framing is derived, value facts are explicit `use` steps.
- The `modifies` and `reads` clauses of Dafny and Low* are the best notation
  in the literature for "this call frees, reallocates or returns an interior
  pointer"; only their solver discharge is unusable. The clause form can be
  kept with a syntactic footprint discharge.

### What the consumers of ownership facts read today

Part B of the evidence file reads the specification and the compiler to find
what each consumer of ownership information actually consumes, and whether
identity footprints carry the same information. Condensed:

| Consumer | Reads today | Under identity footprints | Verdict |
|---|---|---|---|
| PAR-1 window footprints | effect rows projected onto resolved places; OWN-7 overlap; consumed `own` actuals; arena regions as a separate access kind | the same over identities; the arena region stays its own identity kind | equal |
| PAR-1 argument loans | the borrow mode of each actual, independent of the callee's row: a `pure` callee taking `&uniq` still holds an exclusive loan | no counterpart: a footprint is an access, a loan is an exclusion | the one difference, below |
| PAR-1 argument-expression reads | per-operand reads; address formation is not a read | forming `ptr<'a, T>` is manifestly not an access of `'a` | equal or cleaner |
| PAR-2 accumulator, affine element maps, adjacent-range partitions | one binding and a closed operator set; retained `ProvedAffineIndexMap` and `ProvedRangePartition` from discharged OP-4 proofs | the same proofs hung on identities; `RangeId` is already an identity in all but name | equal |
| PAR-2 exclusive-loan containment | every exclusive loan must be iteration-own or inside a partition | same difference as the loan row; but the model regains the shared-versus-uniq distinction that the checked tree erases today, which forces PAR-2 to refuse every non-view borrow of enclosing storage | difference plus a gain |
| ENT-5 fact kill | OWN-7 overlap of a projected write against fact support; an element-versus-descriptor bit; a per-parameter reach table from declared mode and type | identity footprints state element, descriptor and range reach directly | equal or better |
| EFF-2 call projection | static struct paths; a view actual projects through its origin set, which the compiler implements as at most one origin and fails closed on callee-returned views | substitution is ordinary type instantiation; a returned view carries its identity | strictly better |
| Backend storage reuse | a three-valued source mode plus IR liveness; its one assumption is that a result may alias a consumed input | identity equality states the sameness directly | equal or better |
| Backend alias and effect metadata | **nothing**: the current emitter states it "emits no overflow or alias promises", and no `noalias`, `alias.scope`, `readonly` or `memory` attribute is emitted anywhere; the why-whitefoot measurements are retired-prototype evidence | per-identity scopes on loaded pointers, per-tile scopes for PAR-2, `initializes` from a `writes`-only row, cross-procedural `noalias` on declarations | a channel to build, under either model |

Two consequences for this map:

- **Today the whole borrow apparatus buys exactly one backend effect: PAR-1
  and PAR-2 overlap actualization.** R4 is a requirement no current mechanism
  serves; the noalias story in why-whitefoot section 5 was measured on the
  retired prototype, which re-derived per-field scopes from resolved places at
  emission. A storage identity in the type is the first-class form of that
  re-derivation.
- **The one behavioral difference is the loan channel.** Today a `&uniq`
  actual denies overlap against any access of the same place even when the
  callee only reads; identity footprints would grant it. Execution is unaffected,
  since two reads do not race, but PAR-1's recorded ground is source
  equivalence, "an implementation may not construct a loan state the source
  checker refuses", and permission-judgment.md lists "treating exclusive borrows
  as writes" among its rejected alternatives. Under the decomposition there is
  no exclusive loan to preserve; whether a call may still claim exclusivity
  beyond its row is a decision to record, not a derivation.

Two soundness conditions the decomposition must state explicitly:

- Two distinct identity parameters may not be assumed distinct unless the call
  site discharged their distinctness, including sub-identities of one storage
  such as `'v[i]` and `'v[j]`. OWN-7 today takes the opposite default: formal
  origins "never establish that two actual sources are disjoint". The
  default-distinct choice in section 7 is sound only with that call-site
  obligation.
- A pointer derived from another pointer, today's reborrow, shares its
  parent's identity or names a sub-identity of it; it never mints a fresh one.
  OWN-9 forbids exactly the noalias pair a fresh identity would assert.

### Threads, locks and external resources under the decomposition

Part D of the evidence file studies what the decomposition needs when threads,
locks and external resources arrive. Its findings, condensed:

| Need | Mechanism | New vocabulary? | Runtime cost | Status |
|---|---|---|---|---|
| Lexically scoped fork and join | PAR-1 already is the CSL parallel rule with syntactic footprints; the window end moves to the join statement | none | fork and join edges | CONCURRENCY-CATALOG section 12 already maps `thread::scope` onto PAR-1 |
| Parallel width not written in source | a PAR-2 iteration-exclusive affine range loan `[c*i+b, c*(i+1)+b)`, proved by the arithmetic PAR-2 already runs | none | none | the catalog's own most recurring cost; a PAR-2 refinement, not a thread construct |
| A child that outlives its block | a linear task handle carrying the child's exit contract over the transferred identities; join consumes it | one linear type | a wait | Chalice `fork`/`join` tokens, Verus join handles; an unjoined child's obligations are an open question |
| Conditional states crossing a join | collapse `ite(c, s1, s2)` at any boundary whose condition the other side cannot name; publish the condition as a returned value when it is needed | none | none | the one place the identity model asks for something CSL does not; sound, untested for precision |
| A lock over identities no caller owns | an ordinary nominal declaring a custody set and an invariant; acquire adds the custody facts at the invariant and a linear guard; release checks the invariant and erases every fact about the custody identities | one nominal kind | one mutex per critical section | CSL resource invariants, Chalice monitors; the invariant's precision condition is unchecked against WF's fact model |
| Two lock-bearing calls actually overlapping | a second permission level: race freedom plus invariant preservation without PAR-1's source-order equality | yes, an interference category CAP-1 excludes | the mutex | a decision, not a derivation; DPJ and Regent decline it to keep determinism |
| Several holders, occasional writes, across threads, paying only at the write | protocol-governed shared regions with a stability judgment and a memory model | a second model | one atomic per write | recommended against; the phase and epoch pattern the catalog already measures beats a read-write lock on the read path at zero reader cost |
| Externally writable storage: shared mappings, device registers, DMA | a **foreign** identity qualifier: no contents fact is retained across any operation; extent facts stay ordinary | one qualifier | none | passes the constitution only if spelled over interference, "another agent may write it without an edge in this program's order", never over origin |
| File descriptors, sockets, anonymous mappings | nothing: a descriptor is three identities, the wrapper with the close obligation, the open-file description with the cursor, and the contents any agent may write | none | none | RESEARCH.md's I/O witness already separates these subjects |

The lock finding is the sharp one. The custody sketch works under the
decomposition, but two `alloc` calls on one lock both exhibit `writes(lock)`,
so PAR-1 denies their overlap: safe and useless. Letting them overlap needs a
permission whose guarantee is weaker than PAR-1's source-order equality. That
is a language decision the map records; it selects nothing.

## 6. A decomposition that keeps the requirements separate

Numbering note: as for section 5, this section keeps the pre-verdict row
names; the decomposition is one candidate and is being debated per decision
point.

The ladder and the table suggest one decomposition. It is [DESIGN.md](DESIGN.md)'s
Candidate A stated requirement by requirement, with two refinements from
[CASES.md](CASES.md): storage state at a join is a term over the captured
branch condition rather than a lattice value, and signatures carry entry and
exit states. Nothing here is selected.

| Requirement | Mechanism in the decomposition | Status relative to today |
|---|---|---|
| R1 | Pointer types name a storage identity; the checker holds `state(identity)` facts; every access checks the named storage's state; storage-ending operations are contract-visible | new: identity on pointers, per-identity state |
| R2 | Affine owner bindings, `dispose`, capability-by-provider linearity, release graph, static release decisions at joins | unchanged: OWN-1, LIV-1, LIV-2, PROV-6 |
| R3 | Copy, affine, linear classes; `move`, take, put, replace as state events on storage; `own` aggregates travel as handles so storage stays put; relocation only by explicit `move` into other storage | changed: handle passing; relocation set explicit |
| R4 | Distinctness from identities: fresh allocations distinct, distinct identity parameters distinct by default, fields and proved ranges distinct; write footprints over spans give per-load alias scopes | changed: facts from identity, not exclusivity; no alias fact is emitted today under either model (section 5) |
| R5 | PAR-1 and PAR-2 over footprints of identities; the loan clause retires; index and range proofs unchanged | changed: roots of footprints |
| R6 | ENT-5 kills by identity footprint; call projection substitutes identity arguments | changed: key of the kill |
| R7 | Identity parameters in signatures and nominals (today's `region_params` spelling, meaning identity only); `requires`/`ensures` over states; existential identities for fresh results; states routed by result variant as CALL-6 routes relations | new: state clauses |
| R8 | Storage classes unchanged; interior pointers name the backing identity; reallocation is a contract-declared replacement with a conditional exit state | new: backing identity |
| R9 | Sequential: nothing; concurrent: a lock holds an identity's state facts while unlocked (section 5, R9) | new when threads arrive |
| R10 | External resources are identities with declared states and an opaque-contents rule | new: external identities |

What this retires, what it keeps, what it adds:

| Retired | Kept | Added |
|---|---|---|
| OWN-2 modes as permission (a read-only pointer becomes an interface flag, not a loan) | OWN-1, OWN-8, LIV-1, LIV-2, PROV-6, ENT-5, CALL-6, certificates | identity parameters on pointer types |
| OWN-3 lexical regions and outlives; OWN-4 named-region liveness; OWN-10 borrow-storage duration | OWN-7 as footprint disjointness over identities | per-identity state facts, conditional at joins |
| OWN-5 exclusivity, suspension, view origin sets, no slice-valued join | EFF-1/2 exactness and projection, rooted at identities | contract clauses for entry and exit states, ended and replaced storage |
| OWN-6, OWN-13 binder borrows, OWN-14 reborrow family | PAR-1, PAR-2 structure and proofs | default-distinct identity parameters with declared aliasing |
| OWN-11 loop body as region; STOR-5 borrow-free storage | PROV-1 spelling `'s` with identity meaning only | handle passing for `own` aggregates; explicit relocation set |
| PAR-1 loan clause | | existential identities for fresh results |

## 7. Couplings and change impact

These are the places where one choice reaches several requirements, re-keyed
to the settled rows of section 2 by the D0 verdict (its section 4). A
candidate that changes a coupling's choice must re-check every row the
coupling names.

| # | Choice | Reaches | Why |
|---|---|---|---|
| C1 | Identity granularity: per allocation, field, element, proved range | R1 precision with holes, R4, R5, R6, M8 | one refinement vocabulary serves state, distinctness, footprints and kills |
| C2 | One distinctness relation, two exactness directions: R4(b) = R5(b), R6 keyed the same way | R4, R5, R6, M6 | R4 tolerates a missing "distinct" (speed); R5 rejects at a non-erasable construct; R6 needs a may-write over-approximation; a mechanism change for one silently retunes the others, so the relation is stated once |
| C3 | Distinct-by-default identity parameters with call-site discharge | R4, R5, M9, M8, M6 soundness | sound only if every call site discharges distinctness for every pair, sub-identities included; never inequality of identity names (P2 `S` vs `b1`); a proof where two arguments come from one container |
| C4 | `own` aggregates passed as handles | R8a, R7, backend convention | identity survives a call only if storage does not move; the convention itself is not priced (R3) |
| C5 | Sequential aliased writes allowed | R4 where identities coincide, R6 kill precision, R9, M9 | safety stays with state; facts weaken only for storages the writer let coincide |
| C6 | Storage state as a term over captured conditions | M10 cost in condition atoms, R1/R2 definiteness at scope exit and loop heads, M1 | replaces path-sensitive analysis with fact discharge; the definiteness rule replaces drop flags |
| C7 | The storage-ending set | R1 completeness, R7 vocabulary, R8a, R14(iii) | every member contract-visible, non-program actors included |
| C8 | Effect roots become identities | R5, R6, R7, R15 | struct-held pointers are nameable in rows only through identity parameters |
| C9 | Backing identity for containers | R7 push/reserve contracts, R8a, interior pointer types | an existential backing a contract may replace |
| C10 | Read-only interface on pointers | R4 readonly facts, API design | a type flag, never a loan or a duration |
| C11 | Locks holding identity state | R5, R9, M2, R14(i) | cost lands on the write; two calls on one lock deny each other under PAR-1 (section 8 Q11) unless a weaker level exists |
| C12 | Per-storage state as the primary mechanism | R1, R2, R3, M1 | affine-replacement.md's three costs: vacancy flow and definiteness paid deliberately, the third vanishes because pointers carry no permission |
| C13 | Modes retired instead of refined | R4, R9, M4 | identity plus footprints answers LEX-1's two-axis questions without a second axis on pointers |
| C14 | The loan channel (section 8 Q14) | R5, R7, M5 | whether a call may claim exclusivity beyond its row; PAR-1's source-equivalence ground is what changes |
| C15 | Outcome arms multiply R2's join paths | R2, R12, M10 | path count per arm bounded by the arm count |
| C16 | Level set ↔ sequential fallback ↔ erasure | R14(ii), M5, M6(iii), R5 | the sequential result is one member of a weaker level's admitted set |
| C17 | Foreign access class ↔ race theorem ↔ retry loops | R14(iv), R5, M2, R10 | seqlock-shaped reads are legal only in the declared class; outside it a plain load racing a foreign write is a race |
| C18 | Trusted base ↔ host boundary ↔ backend-derived facts | M3, R10, R4, M6(iv) | the obligation attaches regardless of origin; backend-derived facts, if admitted, belong to the backend's entry and are marked apart from checker facts |

## 8. Open questions before a candidate can be selected

1. Is the storage-ending set complete and contract-visible? Scope exit,
   `own` parameter passing, container insertion, enum payload moves and
   returns are the cases to check one by one.
2. What identity does an element or a backing carry: a path identity
   `'v[i]`, an existential backing unpacked at pointer formation, or both?
   Interior pointers into containers depend on the answer.
3. Distinct-by-default identity parameters: what the writer declares to allow
   aliasing, and what the call site must prove.
4. Effect rows over identities: grammar, exactness in both directions, and
   whether rows stay writer-written or become derived from contracts.
5. State terms: atom growth, definiteness at cut points, and whether one
   `ensures` clause can state a reallocating push.
6. Whether reads need anything at all in sequential code, or only writes
   and lifecycle events; the ladder says only writes and events.
7. Threads, locks, atomics and external identities: the minimal additions
   (section 5, R9 and R10).
8. A soundness plan, settled as M6(i)'s runnable form: a bounded small-model
   check per discriminating program with stated depth and prior criteria, in
   the [access-state](../../experiments/access-state/RESULTS.md) form, run
   before a candidate is selected; the operational semantics and the adequacy
   theorem are a stated later obligation, settled by the owner on 2026-09-16
   (VERDICT-D0 decision 9). Open: running the check for each candidate over
   P1..P9.
9. Migration: `&` and `&uniq` as sugar over pointer plus state clause; region
   syntax retirement; pattern cards.
10. Conditional states at a fork or join: the collapse rule for `ite` states whose
    condition the other side cannot name is sound but untested for precision.
11. A lock over identities no caller owns fits the decomposition, but two calls
    on one lock deny each other under PAR-1 (coupling C11); letting them
    overlap is a weaker determinism level. R14(ii) now carries a closed level
    set per overlap construct, settled by the owner on 2026-09-16 (VERDICT-D0
    decision 8), which reopens CAP-1's single guarantee. Open: the level a
    lock-bearing overlap names and how it is spelled.
12. Externally writable storage is owned by R14(iv): a foreign-writable
    identity carries a declared access class with defined, unordered, non-UB
    semantics, excluded from R5's race theorem and spelled over interference
    rather than origin (coupling C17). The class's spelling is open question
    A; whether an internal object shared with another thread carries the same
    class waits on the thread trigger.
13. A spawned child that is never joined holds obligations its parent cannot
    discharge; CAP-1 says nothing about obligation leaks across a thread
    boundary.
14. The loan channel is carried as coupling C14 (R5, R7, M5): whether a call
    may claim exclusivity on an identity beyond its row, as a `&uniq` actual
    does today, or whether the row is the complete interference contract.
    Execution is safe either way; PAR-1's recorded source-equivalence ground
    is what changes.
15. R4 has no current mechanism: the emitter states no alias promises, so R4's
    status is unmeasurable and M9's P5 per-load half is unmeasurable until an
    alias-fact channel exists, under either model; the identity model makes
    the prototype's per-field re-derivation a first-class fact. Whether the
    backend may add facts of its own on top of stated facts is a
    `design/compiler` decision (VERDICT-D0 decision 3).

Open questions the verdict added:

| # | Question | Rows | Settled ground and what remains open |
|---|---|---|---|
| A | The spelling of the per-storage interference qualifier: the declared access class of R14(iv), which R10's contracts and coupling C17 read | R14(b), R10, R5 | Spelled over interference, never origin (section 5, foreign identity row); the surface form and its interaction with contents facts are open |
| B | What a non-interfering reader (a debugger, a foreign agent) may observe of an accepted program, and the observation model R14(vi) under which erasure and timing demands are judged | R14(vi), M6(iii), R8b | Deferred by R14(vi); R8b's constant-time and register-level erasure clauses wait on it; a debugger's admission and what it may observe belong here (R14(d), M6(d)) |
| C | The allocation-failure form | R12, R14(v), R2, C15 | Settled by the owner on 2026-09-16 (VERDICT-D0 decision 5): abort with a resource record under R14(v), outside the outcome model, which leaves R12's allocation clause red; open is whether kernels and `GFP_ATOMIC`-shaped contexts ever get an opt-in fallible allocation form, a second spelling priced under M4 |
| D | Stack exhaustion | R14(v), R13 | Settled by the owner on 2026-09-16 (VERDICT-D0 decision 6): an excluded exhaustion with a defined stop matching SCOPE-3; no bound by default, so embedded targets get nothing unless they promise one; open is whether R13's opt-in depth count (P19) is enough for them |
| E | Global holders | R9, `design/language/ownership.md:5` | Settled by the owner on 2026-09-16 (VERDICT-D0 decision 10): the global mutable state ban stands; R9 does not say whether a global writable storage is a holder, which stays open until the thread trigger |

## 9. Relationship to the other files

[RESEARCH.md](RESEARCH.md) records the literature and the hypothesis;
[DESIGN.md](DESIGN.md) records Candidate A and Candidate B in full;
[CASES.md](CASES.md) records the incremental cases under Candidate A. This map
adds the requirement-level frame and the hazard ladder that justifies the
minimal information. CASES.md case B08 is refined here into the
state-as-term rule of section 6; nothing else in those files changes.
Section 2 cites [VERDICT-D0.md](VERDICT-D0.md) as the record of the
dispositions and refused alternatives behind the settled list; its
owner-decision table, on which the owner took the interim position on
2026-09-16, is carried as pending amendments beside the design tree until
entered under `design/skill/SKILL.md`, and [PROGRAMS.md](PROGRAMS.md) holds
the pending programs P11..P19 the rows cite.
