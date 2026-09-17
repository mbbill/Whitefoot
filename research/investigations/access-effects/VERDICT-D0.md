# Verdict D0: the requirement list, settled by debate

Research date: 2026-09-16. This is the synthesized verdict of the D0 debate
(three drafts from different priorities, six adversarial critiques, three
judges, one synthesizer; the full record is
[EVIDENCE-debate-d0-2026-09-16.md](EVIDENCE-debate-d0-2026-09-16.md)). It is
the record of the settled requirement list, the premises reclassified, the
disposition of every family in [OPTIONS.md](OPTIONS.md) sections D0 and REQ
with its reason, and the sixteen decisions only the owner can take. Nothing
here amends the live design tree; owner decisions are carried as pending
amendments. Supersede in place.


Synthesis of drafts A, B, C (`d0/draft-*.md`), six critiques (`d0/critique-*.md`)
and three judge reports (`d0/judge-*.md`) over MECHANISM-MAP §2/§3/§7/§8, the
constitution, why-whitefoot 1–140, `merged/D0.md` (22 families), `merged/REQ.md`
(23 families) and `discriminating-programs.md`. Base: draft A (all three
judges). Grafts: every graft two or more judges named. Where judges disagreed
the majority position is in the row and the minority is a "Disputed" note.
Nothing is selected: adopted mechanisms are named as mechanisms where a premise
contained one, and every live-tree ruling a row touches is an owner decision
(§7), not a reversal.

## 0. Row form and authoring rules

- Each row: (a) what must hold; (b) the least a checker must know; (c) where
  checked; (d) not its job; (e) acceptance test over P1–P10 and the pending
  programs of §3. A row whose (e) names only a pending program is
  **provisional** and marked so (AI-D0-21 without its deletion clause).
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
- Status marks against MAP §2: KEPT, RESTATED, SPLIT, MERGED, RECLASSIFIED,
  RESTORED, ADDED.

## 1. Hazard and duty owners (constitution → rows)

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
| external interaction through ordinary objects | R10; adapter obligation in M3 (THE-REQ-14: owner decision 15) |
| machine verification before acceptance; no writer escape | M1, M2, M3 |
| practical iteration at scale; termination alone insufficient | M10 |
| compatibility and evolution | M11 |
| delegation with retained control; less repeated human inspection | R7 (interface sufficiency, provisional), M7, M8; contract authority: owner decision 14 |

## 2. Settled list

### R1 Sequential memory safety — RESTATED

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
- Note: "a property of the storage, never of the pointer" is the map §6
  decomposition; it lives in the P7 test, not in (a), so a borrow-duration
  candidate is refused by the program, not by the row. "Write-once
  transitions" removed from (b) (a state mechanism).

### R2 Resource lifecycle accounting — RESTATED

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
  differs); pending P18 (P2 with a failing `alloc` arm, owner decision 5);
  pending P12 (DMA escrow completion). Metric: no runtime value not bound in
  source selects a release, tested by source-to-IR provenance, never by an IR
  bit count.
- Price: no release inserted or removed; no release whose execution depends on
  runtime state the source did not branch on; a release may move only where
  R4/R7 facts make the move unobservable.
- Cost: ~1.5k / one release op per resource / 1, local. Status: green (LIV-1,
  PROV-6). Outcome arms multiply the paths this row discharges on; charged to
  M10.

### R3 Value classes and transfer — KEPT

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

### R4 Licensed facts for the backend — RESTATED

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
  analysis (RAD-D0-25: owner decision 3); which IR attributes carry a fact
  (`design/compiler`).
- (e) P5: the source carries a checkable distinctness fact for every pair of
  column accesses, and nothing in the language forbids its retention to
  lowering; P1: the two taken parts are distinct; P2 negative: no distinctness
  between `S` and `b1`. Theorem shape Facts(P) ⊆ Truths(⟦P⟧), exercised by
  M6(i). Emitted-attribute counts are compiler tests under `design/compiler`,
  not this row's test: the emitter emits no alias promises today.
- Price: forbids emitting a fact the source did not check and requiring
  re-derivation to reach a stated fact; says nothing about extra analysis
  pending owner decision 3.
- Cost: 0 spec beyond R5's footprint vocabulary / 0 / 0. Status:
  unmeasurable (no channel; MAP §8 Q15).
- Disputed: standing as a language requirement touches `design/log.md:168`
  (owner decision 1); B reverses that ruling and tests on IR attributes
  (recorded as a pending amendment, not adopted); C merges R4 into R5 on M5's
  erasability, which fails at spawn/join (owner decision 2). Majority (A, B;
  all three judges): separate rows, soundness clause kept, the per-load
  sufficiency clause moved to M9 as a measured property.

### R5 Data-race freedom and permitted overlap — RESTATED

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
  (CAP-1 fixes one guarantee; owner decision 8).
- Disputed: B's closed three-level menu inside R5, with its middle level a
  schedule commitment; C's merge with R4. Majority: the grounds here, the
  level set in R14.

### R6 Frame: proof-fact retention — RESTORED as a measured precision row

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
  writer tax. Majority: restored; owner decision 13.

### R7 Compositional call judgement — RESTATED (FN-1 reclassified)

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
  (MAP §8 Q5).

### R8a Storage-ending and relocation events — SPLIT from R8

- (a) Every value lives in a stated placement (frame, heap, arena, slab,
  inline in a container). Every operation that ends, moves or replaces
  storage — relocating move, reallocating push, arena reset, scope exit,
  `own` parameter passing, container insertion, enum payload move, return,
  and relocation by an R14-admitted non-program actor — is a storage-ending
  event R1 consumes and R7 can state; the set is complete and
  contract-visible (MAP §8 Q1). Interior pointers name the storage that
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

### R8b Placement and physical demands — SPLIT from R8 (provisional)

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
  provisional (no demand in P1–P10); owner decision 7.

### R9 Shared mutation among several holders — RESTATED as shape admission

- (a) Several long-lived pointers to one storage, each writing occasionally,
  is a writable shape. Sequentially the hazard is R1's alone; licensed facts
  weaken under R4 where holders coincide. Across threads the shape is
  admitted only once a thread construct and R14(i)'s ordering vocabulary
  exist (trigger); what the checker knows before and after acquire is then
  R14's to state. Whether a global writable storage is a holder is not
  decided here: the live ban (`design/language/ownership.md:5`, grounded on
  parallel permission itself) stands until the owner rules (owner decision
  10).
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

### R10 External resources as ordinary objects — KEPT

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
  for memory / 1. Status: provisional (P14 pending); THE-REQ-14 is owner
  decision 15.

### R11 Partial operations and overflow — ADDED

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

### R12 Failure paths and typed outcomes — ADDED (scoped)

- (a) Every operation the specification classifies as fallible (expected
  input and environment failure: short read or write, interruption, device
  error; allocation failure pending owner decision 5) yields a typed outcome
  on a source-visible path; every R2 obligation discharges on each arm; a
  context may forbid a failure outcome (no-allocate as "the
  allocation-failure outcome is unreachable here") and the checker enforces
  it. An unproved domain is a rejection (R11): this row is not global
  prove-or-handle (HIS-D0-04, HIS-D0-05, HIS-D0-07, HIS-D0-10 stay refused). Blocking and latency are
  not failures (R10(d): host mechanism categories refused).
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

### R13 Resource bounds where demanded — ADDED (opt-in)

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
  Status: provisional; stack exhaustion's owner is R14(v) (owner decision 6).

### R14 Stated execution model and external conditions — ADDED

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
  fixes one level); owner decisions 6 and 8.

### R15 Abstraction — ADDED (scoped)

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

### M1 Specified deterministic acceptance — RESTATED

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

### M2 No unstated failure edge, no unchosen cost — RESTATED (two clauses)

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
  enforceable form, which fails every `-O2` lowering (B-cost F1); owner
  decision 4 on whether cheap writer-visible dynamic checks stay excluded by
  rule (this text) or move to a floor argument.

### M3 Enumerated, writer-closed trusted base — RESTATED

- (a) Every assumption the checker does not establish (linked adapters and
  definitions, host declarations, the backend, runtime, loader and OS as
  named axioms, R14's conditions) is derivable from the source tree as a
  list; each entry carries a machine-checked boundary obligation or the named
  evidence obligation standing in for it (M6(iv) for the backend); the
  obligation attaches to every entry regardless of origin, so R10's theorem
  holds (interim; owner decision 15); axioms enter only through the
  specification or an enumerated declaration, never a writer-reachable form;
  no discharge is a bare assume (HIS-D0-06 refused). "No `unsafe` keyword" is
  the mechanism.
- (b) The list and each entry's obligation; which emitted facts trace to a
  proof and which to a base entry (backend-derived facts, if owner decision 3
  admits them, belong to the backend entry).
- (c) At the boundary declaration; per amendment.
- (d) Semantic non-vacuity in general (a weak `requires` or a defaulted
  result is a logic error the constitution leaves to the requirement author);
  the owner-fixed-interface half (a contract the owner fixed cannot be
  weakened by writer edits) is a provisional clause pending owner decision 14.
- (e) The tool prints the base; for the corpus it holds adapters, the
  backend/runtime axioms and R14 conditions only; P9 and P10 add no entry;
  the blind-writer `byte_at` returning `0` outside its range is refused as a
  discharge only if the owner adopts the interface clause.
- Price: forbids trusting any source-level assertion; every checker-emitted
  fact traces to a proof or a listed entry.
- Cost: ~0.3k / 0 / 0. Status: red (no generated list).

### M4 Writer regularity — SPLIT (budget → M8, repair → M7)

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

### M5 Acceptance independent of permission — RESTATED

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

### M6 Soundness evidence — ADDED

- (a) (i) A bounded small-model check per discriminating program with stated
  depth and prior criteria (the access-state experiment's form), run before a
  candidate is selected; the operational semantics and the adequacy theorem
  (accepted programs reach no UB, corruption, race, uninitialized read,
  overflow or unproved partial state under exactly R14's assumptions; global
  progress) are a stated later obligation (owner decision 9). (ii) Frame
  soundness: a fact retained across an operation is true after it; a design
  whose frame under-kills fails P7/P2, not merely R6's count. (iii) Erasure:
  the meaning of an accepted program is independent of its proofs and
  permissions; lowering ignores proof terms except through R4's licensed
  facts; observable behavior lies in R14(ii)'s specified behavior set. (iv) A
  checker-defect evidence channel outside the accepted program, never a build
  mode (HIS-D0-16 restated; HIS-D0-17 stays superseded): the facts-withheld
  differential covers retained-fact unsoundness only; acceptance unsoundness
  needs the small-model check; measured as a seeded-defect detection rate.
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

### M7 Repairable rejection — ADDED (split from M4)

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
  protocol pinned; decision 5).
- (e) P1 `v[k]` names `k≠i ∧ k≠j`; P8 names the differing path; R1 names the
  failed sub-property; R3 and R2 are distinguishable; R6 names the killer.
  Metric: payload non-empty for 100% of corpus rejections. Evidence: the
  blind-writer repair-from-diagnostic rate (OWN-6 fails today: three
  coordinate-only diagnostics).
- Price: forbids a rule whose only fix is non-local and unnamed.
- Cost: ~0.5k / 0 / sets the local-fix bound for every row. Status: red.

### M8 Teaching and retrieval budget — ADDED (split from M4; prelude and retrieval merged)

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
- Cost: this row prices every other row. Status: red; owner decision 16
  fixes K, the tokenizer, f and the share.

### M9 Measured performance floor — ADDED (M5's fast-shape clause made testable)

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
  unmeasurable until an alias channel exists (SYS-REQ-13; MAP §8 Q15).
- Price: forbids a design whose default-accepted shape is off the floor;
  nothing of the backend.
- Cost: 0 / 0 / 0. Status: unmeasurable on the current compiler.

### M10 Checking cost, scaling and edit stability — ADDED

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

### M11 Evolution with stated loss — ADDED (conditional)

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

## 3. Pending discriminating programs

To be written as pseudocode into `discriminating-programs.md`; a row whose (e)
names only one of these is provisional until then.

| Program | Shape | Rows |
|---|---|---|
| P11 | write-once-then-frozen cache (lazy init, memo table); pass/fail stated under R1 alone; why-whitefoot §5's stance that the absence of such cells is a performance argument is recorded, so admission is a capability question | R1, R4, R14 |
| P12 | DMA escrow: map, agent completes, unmap discharges the loan; which contents facts survive between map and unmap | R2, R14(iv) |
| P13 | storage relocated by a compacting third party while an interior pointer exists; refused unless a fixup form is supplied | R8a |
| P14 | descriptor as three identities (close-obligation wrapper, open-file description with cursor, foreign contents); two wrappers share one description; a short-read arm | R10, R12, R2 |
| P15 | one reduction under a checked law (fixed tree and unspecified order), one float reduction under a named level | R5, R14(ii) |
| P16 | P3 generic over the node type; P5 over the column type; P6's helper as a closure; P9 `pick` through a nominal | R15 |
| P17 | division by a runtime divisor; narrowing cast; index against a runtime length | R11, M7 |
| P18 | P2 with a failing `alloc` arm and a declared byte budget for `A` | R12, R13, R2 (owner decision 5) |
| P19 | a declared depth count with a termination promise | R13 |
| P10 | unchanged, with its trigger made explicit: a thread construct and R14(i) | R5, R9, R14, M9 |

## 4. Couplings carried forward (MAP §7 re-keyed)

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
| C11 | Locks holding identity state | R5, R9, M2, R14(i) | cost lands on the write; two calls on one lock deny each other under PAR-1 (MAP §8 Q11) unless a weaker level exists |
| C12 | Per-storage state as the primary mechanism | R1, R2, R3, M1 | affine-replacement.md's three costs: vacancy flow and definiteness paid deliberately, the third vanishes because pointers carry no permission |
| C13 | Modes retired instead of refined | R4, R9, M4 | identity plus footprints answers LEX-1's two-axis questions without a second axis on pointers |
| C14 | The loan channel (MAP §8 Q14) | R5, R7, M5 | whether a call may claim exclusivity beyond its row; PAR-1's source-equivalence ground is what changes |
| C15 | Outcome arms multiply R2's join paths | R2, R12, M10 | path count per arm bounded by the arm count |
| C16 | Level set ↔ sequential fallback ↔ erasure | R14(ii), M5, M6(iii), R5 | the sequential result is one member of a weaker level's admitted set |
| C17 | Foreign access class ↔ race theorem ↔ retry loops | R14(iv), R5, M2, R10 | seqlock-shaped reads are legal only in the declared class; outside it a plain load racing a foreign write is a race |
| C18 | Trusted base ↔ host boundary ↔ backend-derived facts | M3, R10, R4, M6(iv) | the obligation attaches regardless of origin; backend-derived facts, if admitted, belong to the backend's entry and are marked apart from checker facts |

## 5. Premises

| Premise | Classification | Serves | Reason; what changes if dropped |
|---|---|---|---|
| FN-1 signature-only checking | Mechanism (HIS-D0-08 remains an adopted decision, now placed) | R7 (one way to realize a stated interface), M10 (edit locality: the reach function stops at the interface), M7 (local diagnostics) | The backend and the caller need an interface, not its authorship. Dropped: whole-program inference becomes admissible under PROG-1; R7 still requires a pinned, printed interface for compositionality; M10's reach function is restated without the interface as its boundary. |
| PROG-1 closed world | Mechanism, spent (majority A and B; C's "unspent" is refuted by C's own R7 clause). Two-consumer reading: the checker spends it in M3 (the base is enumerable from the tree), M6 (closed quantification), R5's exact transitive footprint closure (spec line 1902), R7's declaration-side check and R15 instantiation; the backend spends it in devirtualization, cross-declaration facts and monomorphization (M9). HIS-D0-09 remains adopted. | M3, M6, R5, R7, R15, M9 | Dropped: separately compiled interfaces become M3 entries with obligations; M6 gains an "imported interfaces are honored" assumption; R7's interface becomes the only cross-unit fact channel; FN-1's annotation cost would then buy separate compilation, which today it does not (SYS-D0-16's double cost is a mechanism cost, not a requirement conflict). Owner decision 11. |
| M1 deterministic, budget-free checking | Requirement (restated as specified total deterministic acceptance). "No SMT" is the adopted mechanism (HIS-D0-02). | M6 (the acceptance relation is a function the theorem quantifies over), M7, M8 | A specification-fixed fixpoint bounded by lattice height qualifies; a writer-side certificate search qualifies; an in-checker portfolio does not. Dropped: acceptance becomes implementation-defined and the theorem loses its object. |
| M2 no runtime safety check | Requirement (restated as two clauses). "No runtime check or trap" is the adopted mechanism reading (HIS-D0-03); clause (ii) is what keeps HIS-D0-04, HIS-D0-05, HIS-D0-07, HIS-D0-10 refused. | M3 (a trap is an unenumerated failure edge), M6(iii), R12, M9's precondition | Dropped: Vale/Mezzo/CHERI-shaped designs re-enter; every interface gains an implicit failure edge; every R4 fact becomes guard-conditional. Owner decision 4 on the literal form. |
| M3 no unsafe | Requirement (restated as enumerated, writer-closed trusted base). "No `unsafe` keyword" is the mechanism. | M6 (assumptions are exactly M3's list plus R14), R4's facts being non-defeasible, the why-whitefoot floor argument | Dropped: a stuck writer uses the escape; M6's theorem acquires per-site assumptions no tool enumerates; R4 facts become trust-dependent as Rust's `noalias` is under `unsafe`. |
| M5 fast shapes only | Split. Acceptance-independence is the requirement (M5); "accepted shapes are the fast shapes" is a goal measured under M9 (HIS-D0-18, HIS-D0-19); "sequential rather than rejected" is a chosen degradation policy for erasable constructs, named as such. | M5 serves M6 (the theorem is schedule-free); M9 serves the constitution's performance objective | Dropped: acceptance depends on the cost model and the theorem quantifies over schedules; without M9 the floor claim is unmeasured and why-whitefoot §1 unsupported. |

Premises reclassified: 6 (FN-1 and PROG-1 wholly to mechanism; M1, M2, M3
keep a requirement with the mechanism clause split out and named; M5 split
into requirement, goal and policy).

## 6. Dispositions

This table is the record of refused alternatives. "Refuse" means the family's
primary proposal is refused; parts adopted are named. Member lines cover every
member a draft or critique found silently adopted, reversed or dropped.

### 6.1 D0.md families (22)

| Family | Disposition | Reason (engaging the stated tensions); member lines |
|---|---|---|
| Keep the map's list verbatim | Refuse | §7's couplings contradict the independence the family assumes, and the constitution's overflow, failure-behavior, execution-model and practical-iteration duties have no owner in the verbatim list; freezing it freezes the gaps. THE-D0-01 refused; HIS-D0-01 (FLOATED) refused. |
| Refactor the R1–R3 state ladder | Adopt in part (R1 one row with three named sub-properties, THE-D0-02 in that form); THE-D0-03, RAD-D0-05, RAD-D0-06 refused | Three state vocabularies cost three cards where one serves, and layout definition has its own row (R8b), so the split's one gain is taken without it; folding R2 into R3 loses Clean's obligation-versus-uniqueness distinction that R12's outcome arms need and R2's join rule has no R3 counterpart; folding R3 into R1 loses the multiplicity lemma's separate signature; bounded memory gets R13; copy/affine/linear stays a type-level class in R3. |
| Interference rows are one fact (R4, R5, R9) | Refuse the row merge; adopt the shared fact base as coupling C2; R9 restated | One relation is read in opposite exactness directions: a missing "distinct" costs R4 speed but must reject R5 at spawn/join (P10), so one row makes a missing fact a rejection and breaks M5's fallback; the taught vocabulary is halved by stating the relation once (C2), not by merging rows; the "occasional write, long-lived holder" shape survives as R9's negative test. AI-D0-12, RAD-D0-07, SYS-D0-17 adopted as C2; THE-D0-05 refused; RAD-D0-04 adopted for hazard ownership (R1 sequential, R5 concurrent) while R9 stays a capability row. Owner decision 2. |
| R6 relocated | Refuse relocation; R6 restored as a measured precision row; soundness half to M6(ii) | Keeping it separate costs one row; removing it hides the measured cost driver (34/41 `len()` rebinds); folding into R7 forces a signature-shaped clause per local write; folding into R4 (SYS-D0-18) merges a may-write over-approximation with a must-distinct under-approximation and grades an under-kill "slow, not wrong"; identity granularity (D1) is the knob, recorded as C1. AI-D0-13 refused; RAD-D0-03 half-adopted (precision is checker-side and still measured as a requirement); SYS-D0-18 refused; THE-D0-04 refused. Owner decision 13. |
| R4's standing and the optimizer's authority | Adopt R4 restated (soundness clause); RAD-D0-01 refused; RAD-D0-25 to owner decision 3 | CompCert's correctness without metadata does not answer the constitution's performance ranking, and deleting R4 concedes the why-whitefoot measurements (retired; to be re-measured under M9); forbidding re-derivation gives up the loop-versioning parity the scoped-alias-channel result measured, allowing it dilutes the fact channel's purpose; the row binds only what the lowering uses, so the fork is the owner's. Owner decision 1. |
| R7 / FN-1 / PROG-1 reclassified | Adopt: R7 restated; FN-1 and PROG-1 mechanisms; PROG-1 spent | The closed world makes whole-program inference legal while FN-1 forbids it: resolved by naming both as mechanisms with distinct consumers (SYS-D0-16 adopted); recheck cost per edit is M10; spending PROG-1 forecloses separate compilation only if the owner says so (decision 11); "infer and pin" stays open at later decisions because R7 binds the interface, not its authorship. AI-D0-14 adopt (modular cost → M10); RAD-D0-02, RAD-D0-13 adopt; RAD-D0-14 (unspent) refused; HIS-D0-08, HIS-D0-09 remain adopted, placed as mechanisms. |
| M1 restated as reproducible acceptance | Adopt | Reproducible, specified acceptance is the requirement; a specification-fixed fixpoint bounded by lattice height qualifies (the "it is a solver" tension is answered: what is forbidden is state or order outside the specification, not iteration); a fixed-iteration cap does not qualify; no-SMT stays the adopted mechanism. AI-D0-15, RAD-D0-08 adopt; HIS-D0-02 stands. |
| M2 restated as no trap and no unchosen cost | Adopt as the two-clause form | Stating it removes the appearance that all branching is banned (P1's branch) without a "cheap enough" cost rule: clause (ii) draws the line at safety predicates the checker did not discharge, so Vale, Mezzo and CHERI stay excluded by rule and MAP §5's disqualifications survive; the slogan erodes in wording only. AI-D0-16 adopted in part (the unpredicted-branch half → clause (ii); the cost half → M9); RAD-D0-09, SYS-D0-14 adopt as reclassification; HIS-D0-03 stands; HIS-D0-04, HIS-D0-05, HIS-D0-07, HIS-D0-10 stay refused by (ii). Owner decision 4. |
| M3's boundary: escapes and non-vacuous discharge | Adopt RAD-D0-10 (enumerated entries with obligations); HIS-D0-06 stays refused; AI-D0-08 refused in its general form, its owner-fixed-interface half provisional | The trusted base moves into an enumerated list someone audits: that audit is the per-entry obligation, cheaper than an unenumerated base; safety survives a weakened contract but semantics do not, and no oracle exists for semantics in general; the checkable half is owner decision 14. |
| Failure paths and typed outcomes as an owned requirement | Adopt as R12 (scoped) | Without a row "typed outcome" is an unowned valve; the context rule is representable as "this outcome is unreachable here", so the representable/unrepresentable split is closed; linearity on failed paths is R2's join over arms (C15). AI-D0-07 adopt; SYS-D0-03 adopt in part (allocation failure → owner decision 5; bounded allocation → R13); RAD-D0-16 adopt. |
| Writer cost model: M4 restated and split | Adopt the split M4 / M7 / M8 | Each sub-requirement has its own test (lint, payload, token count); K is set provisionally at 48k so the measured 130k is red rather than unmeasured; rule-addressable cards beat a shorter monolith, so M8 requires the ID index (AI-D0-17) and D13's shape follows; the prelude is charged to M8 and re-measured against PRE-1; the spelling lint's FORM-2 gap is stated as its limit. AI-D0-01, AI-D0-02, AI-D0-10, AI-D0-17, AI-D0-20, RAD-D0-11 adopt; HIS-D0-14 named as the adopted mechanism in M4; HIS-D0-21 stays refused. |
| Repairable rejection and machine-readable diagnostics | Adopt as M7 | "Mechanical fix" becomes a rule property (non-empty payload naming a location) rather than a writer property; the infinite-repair-loop risk is owned by the no-reintroduction clause; the machine-readable form is required. AI-D0-03, RAD-D0-19, AI-D0-19 adopt; HIS-D0-15 stays refused, as a clause. |
| Checking cost and latency as a requirement | Adopt as M10 | M1 forbids budgets selecting acceptance and says nothing of a two-hour run; a degree ceiling against a pinned bundle is the bound distinct from budget-freedom; per-statement options (D7/D12) carry a named price; a one-token edit is bounded by syntactic reach without assuming caching; PROG-1 stays because the bound is in program size including instantiations. AI-D0-04, AI-D0-05, RAD-D0-18, RAD-D0-20, SYS-D0-05 adopt; SYS-D0-15 answered; HIS-D0-11 adopted and provisionally tightened. |
| M5 and performance guarantees | Adopt the split M5 / M9; RAD-D0-12 and RAD-D0-24 reclassified as mechanisms; SYS-D0-13 → R14(ii) | A floor is testable, a rule about one mechanism is not; a schedule language is two artifacts to check; the Price line is the requirement-level residue of a published cost model; the determinism menu belongs to the execution model, where CAP-1's single guarantee is reopened only by the owner (decision 8). AI-D0-18 adopt; HIS-D0-18 adopted as measured; HIS-D0-19 adopted as a measured property, never a rule; HIS-D0-20 stays refused. |
| Resource bounds, termination and time | Adopt as R13, opt-in and scoped | The constitution's clause is conditional, so R2's "where promised" was consistent with it and the unconditional reading overreads; termination-for-removal is R4's soundness matter (RAD-D0-22 adopted there); deadlines need a timing model R14 lacks; deadlock freedom waits for the thread trigger; M2's silence on nontermination is answered by the constitution's own allowance. AI-D0-09 adopt conditional; RAD-D0-15 adopt; RAD-D0-22 adopt into R4; SYS-D0-11 adopt in part. |
| Abstraction carries identity and effect facts | Adopt as R15, scoped | Signature explosion is where a first-order winner becomes unwritable, so the test measures parameters per interface against storages touched; dynamic dispatch and modules are not quantified over because the language lacks them. AI-D0-06, RAD-D0-21 adopt. |
| Evolution and migration | Adopt as M11, conditional | The constitution lets compatibility yield; a standing migration tool per amendment at v0.57 is work no experiment needs; the standing test is the verdict diff and the META-5 counts; certificates survive where a rule is unchanged. AI-D0-11 adopted conditionally; RAD-D0-17 adopt. |
| Admission policy for missing requirements | Adopt: THE-D0-06 for hazard-owner rows, writer-cost rows and abstraction (named as such); THE-D0-07 for capability rows; HIS-D0-12, HIS-D0-13 stay refused | The two members contradict only when applied to one row class; hazards enter now because an unowned hazard is unaudited and late discovery of abstraction would select on a fragment; capability rows wait for a program because each costs M8 budget; frequency sets priority, never capability. Owner decision 7. |
| Form of the requirement statement | Adopt THE-D0-09 (programs as (e)) with measurements; THE-D0-08 in runnable form (bounded small-model check now, theorem later); THE-D0-10 refused as form (kept as §4); AI-D0-21 adopted without its deletion clause | Theorem shapes make independence checkable, but a full semantics before selection is the D13 delay the priority order refuses (owner decision 9); test-shaped rows are under-specified between programs, which the theorem shapes and the coupling table cover; the lattice's finding (R9 negates R4) is recorded in R9(d); a row without a program is provisional, not deleted. |
| Hardware placement, ordering, address spaces and secret erasure | Merge in part: SYS-D0-01 → R8b; SYS-D0-02 → R14(i) with trigger; SYS-D0-09 → R8b with the unverified flag kept; SYS-D0-04 half (erasing store → R8b; constant time deferred to R14(vi)) | Layout demands are witnessable in IR; ordering must exist before race freedom is checkable, hence a trigger rather than a row; the erasure-versus-optimization collision is resolved by pricing (a marked store is exempt from DSE); constant time needs a leakage model and a CT-preserving backend entry that no deterministic source checker supplies. Owner decision 7. |
| Observability, debug builds and a checker-soundness oracle | Merge: SYS-D0-06 → M6(iv); RAD-D0-23 → open question B; HIS-D0-16 stands, restated as "one specified behavior set"; HIS-D0-17 stays superseded; B's R22 fact trail named as a candidate mechanism | The channel is compiler evidence outside the accepted program, never a build mode; the facts-withheld differential covers retained-fact unsoundness only, so the small-model check is named beside it; a foreign reader sees erased, optimized code unless R14 states what it may observe. |
| Storage shapes outside the state model | Merge as pending programs and clauses: SYS-D0-07 → P11 under R1 (why-whitefoot §5's performance stance recorded); SYS-D0-08 → R2's completion-event clause plus P12; SYS-D0-10 → R8a's fixup clause plus P13; SYS-D0-12 → R1/R14(iii), a worked instance on P2/P3 | None is a new hazard; each is a program the existing rows must answer or R14 must exclude; the escrow-plus-token form and the write-once protocol are candidate mechanisms, named, not requirements; "Gone is inert" is dropped rather than defended (SLUB, tcache). |

### 6.2 REQ.md families (23)

| Family | Disposition | Reason; member lines |
|---|---|---|
| Checking cost as a requirement | Adopt as M10 | The measured series is the first number the ceiling is set against; HIS-REQ-09 (cost already changed decisions) is the ground for admitting the row now. AI-REQ-01, RAD-REQ-04, SYS-REQ-06, THE-REQ-04 adopt. |
| Abstraction as a requirement | Adopt as R15 | HIS-REQ-07's live decisions (monomorphization, bounds, instantiation termination) are recorded, not re-selected. AI-REQ-09, RAD-REQ-05, THE-REQ-01 adopt. |
| Resource bounds as a requirement | Adopt as R13; AI-REQ-11 refused | The constitution is conditional ("for uses with explicit resource budgets"); a candidate satisfies both by proving what is promised; HIS-REQ-05's accounting system is a mechanism, not selected. RAD-REQ-01, THE-REQ-03 adopt. |
| Failure, partiality and typed outcomes | Adopt as R12 (failure) and R11 (partiality, overflow) | Two hazards, two owners; the R2 join interaction is C15 and R12's test; HIS-REQ-08 outcome typing is a live decision, not re-selected. AI-REQ-10, RAD-REQ-02, SYS-REQ-02, THE-REQ-02 adopt. |
| M4 decomposition: teachability, spec size, auditability | Adopt as M4 / M7 / M8; THE-REQ-05 → R7's provisional interface-sufficiency clause plus M8 | "Verbosity cheap for the writer, prohibitive for the owner reading diffs" is answered by the interface-sufficiency clause and its diff-size test, not by spec size alone; HIS-REQ-11's per-amendment META-5 counting is M8/M11's test; AI-REQ-03's number is K, provisional at 48k. AI-REQ-02, RAD-REQ-08 adopt. |
| Repair, edit stability, evolution | Adopt across M7, M10, M11 | Three properties, three tests; OWN-8's named restructuring is M7's payload for precision-rule rejections. RAD-REQ-03, RAD-REQ-06, RAD-REQ-07 adopt. |
| Standard library and prelude cost | Merge into M8 | The largest measured per-file cost counts against the budget; delivery is a mechanism; the fraction is re-measured against PRE-1. AI-REQ-08 adopt. |
| Layout and memory ordering | Merge into R8b (layout) and R14(i) (ordering, with trigger) | SYS-REQ-01: every covered systems program is shaped by both, but neither is a hazard; layout is a witnessable demand, ordering is the execution model race freedom rests on. |
| Third-party mutation of storage | Merge into R1 (ended storage not inert), R8a (relocation with a fixup form), R14(iii)/(iv) (actors' rights, foreign class) | SYS-REQ-03, SYS-REQ-04 adopt as clauses; R1's "inert Gone storage" widened to non-program actors; an interior pointer and third-party relocation are mutually exclusive without a fixup channel, which R8a states. |
| R5 scope: reductions and determinism level | Adopt: SYS-REQ-05 → R5 ground (ii) with P15; SYS-REQ-14 → R14(ii) | Associative shared accumulators are admitted under a checked law, never a declared one; the level is named per construct rather than fixed at source-order equality, which is owner decision 8. |
| Checker observability | Merge into M6(iv) | SYS-REQ-07 adopt: the channel is compiler evidence; erasure and no-trap are preserved because the channel is outside the accepted program; its coverage limit is stated. |
| R4 and R5 are one relation | Refuse as a row merge; adopt the relation as C2 | AI-REQ-06, RAD-REQ-15, SYS-REQ-08, THE-REQ-07: same distinctness fact, opposite exactness direction; stating the relation once removes the silent retune while keeping M5's fallback and R5's rejection at spawn/join. Disputed (C). Owner decision 2. |
| R4 and R6 are precision policies | Adopt for R6's precision as a measured cost (AI-REQ-07, RAD-REQ-10 in that form); refuse for R4's soundness clause; HIS-REQ-02 honored for the channel (sufficiency → M9), its standing → owner decision 1 | RAD-REQ-09's "a missing fact costs speed" is R4(d); what reaches LLVM stays a compiler detail; what the lowering may use without a guard is a soundness surface and stays a requirement. |
| R6 not independent of R4 and R7 | Refuse the fold; coupling recorded in C2 and R6(b) | SYS-REQ-09, THE-REQ-08: choosing a contract vocabulary largely chooses the kill rule, recorded as a coupling because a fold forces signature-shaped clauses on local writes. |
| R7 / FN-1 versus PROG-1 | Adopt the reclassification; HIS-REQ-06 in fixed-check form; RAD-REQ-11 → M10; THE-REQ-16 adopt (FN-1's grounds are M7 and M10); SYS-REQ-12 answered (both costs named; PROG-1 spent); RAD-REQ-20, AI-REQ-05 adopt | Opens "infer, print, pin" at later decisions without giving up a stated interface; WF's annotation cost without separate compilation is priced as a mechanism cost in §5. |
| R9 dependence on R1, R4, R5 | Adopt the shape-admission restatement; RAD-REQ-12, THE-REQ-10 adopted (hazards owned by R1/R5; the negation of R4's exclusivity recorded in R9(d)); HIS-REQ-01, HIS-REQ-12 recorded (OWN-5 over-serves; the requirement is not weakened to match it); HIS-REQ-10 → owner decision 10, the ban standing meanwhile | Restated as one axis with a per-storage choice; R1(d) describes the requirement and OWN-5 the current mechanism. |
| R2, R3, R8 and R1 lifecycle dependence | Refuse the merges; C7 stated once; SYS-REQ-10 answered (arenas and slabs are worked instances under R1/R2/R8a/R14(iii), not one row) | RAD-REQ-13, RAD-REQ-14, THE-REQ-09: Clean's one differing part (obligation versus last reference) is the difference between two repair messages, and the join rule is stated once in R2, so nothing is duplicated. |
| R1 bundles three properties | Adopt as an internal split with the diagnostic naming the failed sub-property; layout definition to R8b, layout currency stays in R1 | THE-REQ-11: whether one vocabulary serves all three is the candidate's to justify; the row does not prejudge it. |
| M1 "no SMT" is a mechanism | Adopt | RAD-REQ-16, THE-REQ-12: a writer-side certificate search qualifies, an in-checker portfolio does not; the mechanism is not re-selected. |
| M2 "no runtime check" is a mechanism | Adopt the two-clause form | THE-REQ-17's distinction is clause (ii); RAD-REQ-17, SYS-REQ-11: generational and epoch checks that safety depends on stay refused, a seqlock retry loop is admitted only over R14(iv)'s access class (C17); THE-REQ-13 adopt; HIS-REQ-03 stands. |
| M3 "no unsafe escape" hole | Adopt the enumerated base; RAD-REQ-18 adopt (obligation per entry); AI-REQ-04 general form refused, owner-fixed half → owner decision 14; THE-REQ-14 → owner decision 15 with the interim "obligation regardless of origin" | The trusted adapter at R10 is an ordinary entry under the interim, neither a violation nor an exception; the constitution's clause is preserved until the owner rules otherwise. |
| M5 is a goal, not a checkable property | Adopt the split M5 / M9 | RAD-REQ-19 answered (schedule is backend freedom by the Price rule; shape is measured); SYS-REQ-13 recorded (M9's P5 half is unmeasurable until a channel exists); THE-REQ-15 adopt (validated by measurement); HIS-REQ-04 recorded (obvious-shape reproduction open); AI-REQ-12 adopt. |
| Erasure as a checkable requirement | Adopt as M6(iii), with M6(iv)'s coverage limit | THE-REQ-06: a premise with no theorem is an unchecked assumption of every other row; the erasure architecture is named as the adopted mechanism and the property is "meaning independent of proofs". |

Families dispositioned: 22 + 23 = 45 of 45. Refused (primary proposal
refused): 6 — "Keep the map's list verbatim", "Interference rows are one
fact", "R6 relocated", "R4 and R5 are one relation", "R6 not independent of
R4 and R7", "R2, R3, R8 and R1 lifecycle dependence".

## 7. Owner decisions

Each touches a live ruling, reopens a recorded refusal, or is a design fork
with a corpus cost. The list above takes the interim position named; the owner
overrides by ruling under `design/skill/SKILL.md`.

| # | Question | Options and what each costs | Interim in this list |
|---|---|---|---|
| 1 | Is a backend-consumed alias fact a language requirement (R4), given `design/log.md:168` rules what reaches LLVM an implementation detail? | (a) Requirement with a soundness clause; the channel and its granularity stay compiler detail, measured under M9 — costs: sufficiency unmeasurable until a channel exists. (b) Language requirement with IR-attribute tests (B) — reverses the ruling, couples the list to one backend, needs a tree amendment. (c) Delete R4 (RAD-D0-01) — concedes the performance ranking and the floor evidence. | (a) |
| 2 | R4 and R5: two rows or one? Does a missing distinctness fact at a non-erasable construct (spawn/join, P10) reject, or leave the program sequential? | (a) Two rows with the relation stated once (C2) — costs one relation in two places. (b) One row (C) — halves the vocabulary, but the row still needs two exactness clauses because a spawn cannot be sequentialized (deadlock is a meaning change). | (a); rejection at non-erasable constructs |
| 3 | May the backend add facts by its own sound analysis on top of stated facts (RAD-D0-25), and is guarded loop versioning an unchosen cost? | (a) May add, never drop (B) — free wins; backend-derived facts become the backend's M3 entry and must be marked apart in M6(iv). (b) Forbidden — pure fact channel; gives up the versioning parity measured at 2.1 vs 0.4 ns/element. (c) A `design/compiler` decision (C) — the list binds only what is used. | (c); versioning is not an unchosen cost under M2 |
| 4 | M2's enforceable boundary. | (a) Two clauses (this list) — Vale/generational/epoch designs excluded by rule; backend control flow free; tested at the WF→IR boundary. (b) Literal "no compiler-inserted check or branch" (B) — fails every `-O2` lowering, forbids versioning. (c) "No unchosen cost" priced by the M9 floor (AI-D0-16 pure form) — un-refuses HIS-D0-04, HIS-D0-07; Vale and Mezzo re-enter as cost arguments. | (a) |
| 5 | Allocation failure: typed outcome or SCOPE-3's abort-with-record outside the model? | (a) Typed outcome on every allocation (all three drafts) — whole-corpus rewrite, `GFP_ATOMIC`-shaped contexts expressible, R2 arms multiply (C15). (b) Abort with a resource record, outside the outcome model (live SCOPE-3) — kernels cannot express failure handling. (c) Both: an opt-in fallible allocation form beside a default abort — a second spelling (M4 cost). | (b) in R14(v); R12's clause marked red |
| 6 | Stack exhaustion's owner. | (a) Excluded exhaustion with a defined stop (matches SCOPE-3) — no undefined path, no bound, embedded targets get nothing. (b) Mandatory depth bound for every program — refuses unbounded recursion; writer cost per program; embedded-friendly. (c) Typed outcome — needs a probe (M2(ii) conflict) or a static bound anyway. | (a) in R14(v) |
| 7 | Admission of hardware rows and storage shapes against an M8 budget already failed (~130k vs 48k). Is this a systems-language list or an ownership-subsystem list? | (a) Rows now (B: layout, ordering, address space, erasure, write-once, loans, allocator, trail) — about nine rows of vocabulary. (b) Merged into R8b/R14 with pending programs (this list) — demands admitted, protocols named as candidate mechanisms. (c) Only on a blocking program (C) — cheapest; risks selecting a mechanism on a first-order, single-threaded fragment. | (b) |
| 8 | Determinism levels: keep CAP-1's single source-order guarantee or admit a per-construct closed level set? | (a) Single guarantee (CAP-1, HIS-D0-16 as written) — every reduction not provable under a law is refused; float reductions refused permanently. (b) Closed set per construct in R14(ii) with HIS-D0-16 restated to "one specified behavior set" — reopens CAP-1; spec text; float reductions admitted by level. (c) B's three-level menu inside R5 with a bitwise-reproducible middle — a schedule commitment inside a requirement. | (b) |
| 9 | Requirement form and timing: operational semantics plus adequacy theorem before selection, or programs plus measurements with the theorem later? | (a) Theorem first (A's M6) — independence becomes checkable; D13 delayed by the cost of a mechanized semantics for a language with R14, R11 and R15. (b) Bounded small-model check per program now, theorem a stated later obligation — independence partly prose until then. | (b) |
| 10 | Global mutable state ban (`design/language/ownership.md:5`): keep until threads exist, or lift as an R9 instance? | (a) Keep — R9 does not say whether a global is a holder. (b) Lift (B) — needs a new ground, since the recorded ground is parallel permission itself. | (a) |
| 11 | PROG-1: spent (checker and backend) or unspent; is separate compilation ever a goal? | (a) Spent by M3/M6/R5/R7/R15 and the backend — infer-and-pin stays possible; separate compilation is a later cost. (b) Unspent (C) — opens libraries and incremental builds now; re-prices R5's exact closure, R7's declaration check and M3's enumeration. | (a) |
| 12 | Writer-model trials as acceptance criteria or evidence only? | (a) Evidence only, model and protocol pinned (decision 5) — M7 and M9 keep checker-side tests alone. (b) Acceptance criteria (A's M7/M9, C's rounds) — unreproducible across model retirements; conflicts with the tree. | (a) |
| 13 | R6 frame precision: restored measured row, retired, or merged into R4? | (a) Restored (judges 1 and 3) — one more row; owns the largest measured writer tax. (b) Retired (A, judge 2) — precision tested only by P1/P4, neither with a coarse-footprint call. (c) Merged into R4 (B) — opposite exactness directions. | (a) |
| 14 | Contract authority separation (AI-REQ-04): does the checker own "an owner-fixed interface contract cannot be weakened by writer edits"? | (a) Refuse all non-vacuity (A, C) — a stuck writer may weaken a contract; constitutional objective 1 partly unowned. (b) Owner-fixed half (C-soundness F5; provisional M3 clause) — needs an "owner-fixed" marker (a mechanism) and a contract-strength diff test. (c) Full non-vacuity (B) — uncheckable without an oracle. | (b) provisional |
| 15 | The trusted adapter at the host boundary (THE-REQ-14): ordinary M3 entry, enumerated exception, or constitutional amendment? | (a) The boundary obligation attaches to every entry regardless of origin — adapters are ordinary entries; the constitution's "no rule distinguishes" clause holds. (b) Enumerated exception — amends the clause in effect. (c) Amend the constitution. | (a) |
| 16 | Threshold ownership: who fixes K (48k provisional) and its tokenizer, f, the subsystem share of K, the M9 ratio and factor, the M10 degree ceiling, and by what procedure? | (a) Owner fixes each now. (b) Owner delegates a derivation rule per threshold. (c) Leave provisional; M11 governs every reset. | (c) with the provisional values named |

## 8. Changes to MECHANISM-MAP.md section 2

| # | Edit | Content |
|---|---|---|
| 1 | Add preface | §0's row form: (a)–(e) plus Price, Cost, Status; provisional marking; threshold rule; writer trials as evidence; admission policy |
| 2 | Add table before R1 | §1 hazard-and-duty owner table |
| 3 | Reword R1 | (a)(b)(d)(e) as §2; ended storage inaccessible not inert; address reuse never revives an identity; three named sub-properties; "property of storage" moved to the P7 test; write-once removed from (b) |
| 4 | Reword R2 | outcome arms; program-observed completion event for loans; release decisions static by M2(ii); bounded-memory clause moved to R13; Price line |
| 5 | Amend R3 | diagnostic clause (used-twice vs never-released); (d) refuses a calling-convention price |
| 6 | Retitle and reword R4 | "Licensed facts for the backend"; soundness clause; termination-for-removal; distinctness as a proved relation never name inequality; "never re-derived" reclassified (owner decision 3); Disputed note |
| 7 | Reword R5 | three overlap grounds; checked law never declared; erasable vs non-erasable constructs; foreign access class excluded |
| 8 | Reword R6 | precision only, tested relative to the stated footprint; soundness half moved to M6(ii); red value 34/41 |
| 9 | Retitle and reword R7 | "Compositional call judgement"; extended vocabulary; fixed declaration-side check; FN-1 named as mechanism; provisional interface-sufficiency clause |
| 10 | Split R8 | R8a storage-ending and relocation events (set completeness, fixup form); R8b placement and physical demands (provisional; constant time deferred) |
| 11 | Reword R9 | shape admission with negative test; sequential hazard R1 only; concurrent half on the thread trigger; global holders undecided (owner decision 10) |
| 12 | Amend R10 | adapter obligation lives in M3; trusted declarations counted; P14 pending |
| 13 | Add R11–R15 | as §2 |
| 14 | Replace the "Meta-constraints" block | M1–M11 in (a)–(e) form as §2, each naming its adopted mechanism |
| 15 | Section 3 note | Step 5's conflict table gains grounds (ii) and (iii); no other change to the ladder |
| 16 | Add section 2a | §5 premises table |
| 17 | Replace section 7 | §4 couplings C1–C18 |
| 18 | Amend section 8 | Q8 → M6(i) runnable form; Q11 → R14(ii) and owner decision 8; Q12 → R14(iv); Q14 → C14; Q15 → R4/M9 status; add A (interference qualifier spelling), B (what a non-interfering reader may observe; observation model), C (allocation-failure form), D (stack exhaustion), E (global holders) |
| 19 | Amend `discriminating-programs.md` | add P11–P19 pseudocode from §3; make P10's trigger explicit |
| 20 | Link | MAP §2 cites this file as the record of dispositions and refused alternatives; the owner-decision table is carried as pending amendments beside the tree |
