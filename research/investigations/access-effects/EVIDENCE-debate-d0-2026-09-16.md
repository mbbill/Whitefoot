# Evidence: the D0 requirements debate, 2026-09-16

The three drafts (A: safety and formal adequacy first; B: performance and
systems practice first; C: the AI writer and cost first), the six critiques
(soundness and cost lenses per draft) and the three judge reports behind
[VERDICT-D0.md](VERDICT-D0.md). Agent output reviewed by the primary agent;
evidence, not decisions. Remove with the verdict it supports.


# File: draft-A.md

# Draft A: revised requirement list (safety and formal adequacy first)

Stance: every hazard the constitution names has an owner row; every row is
stated so that its soundness is a theorem lemma and its independence from the
other rows is a claim the theorem can check; every "meta-constraint" that is a
chosen mechanism is named as one and kept as the recorded decision it is. No
mechanism is selected here.

Sources read: MECHANISM-MAP §2, §3, §7, §8; constitution; why-whitefoot lines
1–140; merged/D0.md (22 families); merged/REQ.md (23 families);
discriminating-programs.md (P1–P10 and the corpus clause).

## 0. Hazard owners (constitution, Safety section)

| Constitution hazard or duty | Owner row(s) |
|---|---|
| undefined behavior | M6 (theorem), composed from R1, R5, R11, R14 |
| memory corruption | R1, R2, R3, R8 |
| data races | R5 |
| uninitialized reads | R1 (i) |
| silent overflow | R11 (new) |
| any other unproved partial operation | R11 (new) |
| machine-verifiable resource bounds (where budgets are explicit) | R13 (new, out of R2) |
| execution model and external conditions defined | R14 (new) |
| expected input and environment failures have defined behavior | R12 (new) |
| external interaction through ordinary objects | R10 |
| compatibility and evolution | M11 (new) |
| machine verification before acceptance; no writer escape | M1, M2, M3 |
| practical iteration at scale (termination alone insufficient) | M10 (new) |

## 1. Revised list

Column key: (a) what must hold; (b) the least a checker must know; (c) where
checked; (d) not its job; (e) acceptance test. Programs are P1–P10 from
discriminating-programs.md; P11–P14 are proposed additions named in place.
Status marks: KEPT, RESTATED, SPLIT, MERGED, RECLASSIFIED, ADDED.

### 1.1 Hazard and capability rows R1–R15

| Row | (a) must hold | (b) checker needs | (c) where | (d) not its job | (e) acceptance test | Status |
|---|---|---|---|---|---|---|
| **R1 Sequential memory safety** | (i) no read of storage holding no value; (ii) no access to storage that has ended by any event the model admits: free, reallocation, scope release, relocating move, or an R14-admitted external actor; (iii) no access under a stale layout. All three are properties of the storage, never of the pointer. | For the storage an access names: live, initialized in the selected part, layout current; the complete storage-ending event set (map §8 Q1), including write-once transitions. | Each access, against the named storage's state at that point. | Two pointers to one storage; sequential aliased writes; overlap (R5); who releases (R2); what an external actor may do (R14). | P4 (second `c.read()` legal iff push proved non-reallocating; read after `free(v)` refused), P7, P2 (pointer into b2 refused after b4 reuses its bytes), P3 (removal invalidates every path to m). Theorem: preservation lemma — every access in any execution of an accepted program names Init storage under its current layout. Proposed P11: write-once lazy-init cache is accepted, or R14 excludes it explicitly. | SPLIT internally into (i)(ii)(iii) as three lemma obligations under one row (THE-D0-02 adopted in this form; RAD-D0-06 refused) |
| **R2 Resource lifecycle accounting** | Every allocation and external resource is released exactly once on every path, including typed-failure paths (R12); linear obligations are discharged; a buffer loaned to a non-program agent (DMA, io_uring) is an obligation with a program-visible discharge event. | Which binding holds which release obligation; definiteness on every path including failure paths; for an escrowed buffer, the completion event that discharges it. | Consuming operations, scope exits, joins; static decisions (a runtime flag is an unstated failure edge under M2). | Read/write permission; how much is held (R13); value multiplicity (R3). | P2 (three blocks, exactly one free each, arena never held exclusively), P8 (no drop flag; rejection names the path whose state differs). Theorem: trace-counting invariant — each resource has exactly one release event per acquisition. Proposed P12: DMA escrow with completion token. | RESTATED: bounded-memory clause moved to R13; failure-path and escrow clauses added |
| **R3 Value classes and transfer** | Copy values duplicate freely with no obligation; affine values move at most once and may take a compiler-derived release; linear values are consumed exactly once. Move transfers value with obligations; take leaves a hole; put fills one; replace exchanges. | Each value's class; each transfer's source and destination storage. | At the transfer. | Pointer validity: a value leaving storage is an event on that storage's state that R1 consumes. | P7 (take/put/replace through aliases). Theorem: multiplicity-preservation lemma, stated and proved separately from R1's state lemma; M6 composes them. | KEPT; merges into R2 or R1 refused (THE-D0-03, RAD-D0-05, RAD-D0-06) |
| **R4 Licensed facts for the backend** | Soundness: every fact the lowering uses without a guard (two accesses disjoint, storage unwritten over a span, loaded value invariant, call removable) is true in every execution the semantics admits for the accepted program. Sufficiency: the source can license per-access and per-span facts, not only per-parameter ones. Bound: a writer-stated physical demand (R8: erasing store, constant-time operation) is never elided by a licensed fact. | Distinctness by identity, field, proved index, proved range; write footprints over spans; the set of demand-protected operations. | At lowering, from checked facts of the accepted program. Whether the backend may add facts from its own sound analysis is a mechanism decision (RAD-D0-25); the requirement binds only what is used. | Safety of the source (R1, R5, R11); profitability (M9). A missing fact costs speed, never correctness. | P5 (eight columns distinct per load, no guards), P1 (two parts `noalias`). Theorem: fact licensing — Facts(P) ⊆ Truths(⟦P⟧). Measurement: M9 floor on the locked corpus. | RESTATED: soundness clause made explicit; "never re-derived" reclassified as mechanism; demand bound added |
| **R5 Data-race freedom and permitted overlap** | A data race is unrepresentable. Two statements or iterations may overlap only when the checker has established non-interference of their footprints, or a law (e.g. associativity of a named reduction operator) that R14's model admits for that construct. Every permitted overlap's observable behavior is one R14's stated level for that construct admits. | Read and write footprints per statement over storages; distinctness between footprints; data dependencies; for reductions, the operator law and its discharge. | At each overlap construct: window, counted loop, spawn/join, acquire/release. | Sequential safety (R1); choosing the level (R14); acceptance (M5: permission never changes acceptance). | P1 (two writes parallel), P3 (parallel map by node distinctness), P5 (PAR-2 overlap), P6 (overlap from range arithmetic), P10. Theorem: DRF-guarantee shape — permitted overlap is observably equivalent to the R14 level. Corpus: wfgrep, zlib kernels, compute-bench, accumulators keep their PAR-1/PAR-2 overlap or the loss is stated. | RESTATED: level parameterized by R14; reductions admitted under a proved law (SYS-REQ-05) |
| **R6 Frame: proof-fact retention** | Retired as a row. Soundness half ("no fact survives an operation that may have falsified it") is M6 (ii), a checker-soundness condition. Precision half ("every other fact survives") is checker precision, not a requirement. | — | — | — | Precision is tested by R1 and R5 programs at the precision they demand: P1 `read v[k]` with the k∉{i,j} fact, P4 second read after a proved non-reallocating push. | RECLASSIFIED; number retained so nothing is dropped silently |
| **R7 Compositional call judgement** | A call is judged from a stated interface of the callee; every accepted callee body satisfies its interface; the interface vocabulary can state, per storage: hole, ended, replaced backing, returned interior pointer, fresh allocation, entry and exit states, footprint. A declaration whose interface no caller can use is rejected at the declaration (HIS-REQ-06). | Storage identities nameable at the interface; entry and exit states per identity; footprints per identity; existential or fresh results. | Caller: at the call. Callee: once at its exits. Declaration: at the declaration. | Whether the interface is written or inferred-and-pinned (mechanism: FN-1 is one choice, see Premises); recheck cost (M10); trusted-import rules (M3). | P9 (drain, reserve, pick each visible in the interface), P6 (helper's interface alone tells the caller what it writes). Theorem: compositionality — (body ⊢ I) ∧ (caller judged by I) ⇒ M6 adequacy for the whole. | RESTATED: FN-1 reclassified as mechanism; declaration-side obligation added |
| **R8 Storage placement, relocation and physical demands** | Every value lives in a stated placement; every operation that ends, moves or replaces storage — including relocation by a non-program actor R14 admits — is a storage-ending event R1 consumes and R7 can state; a writer may demand alignment, layout (SoA/AoS, cache-line), address space, and erasure of a placement, and every accepted lowering honors the demand. | Which operations end or replace which storage; bytes-vs-handle travel; the demand vocabulary and which demands bind the backend. | At the relocating operation; at the placement declaration. | Who may access; the ordering model (R14); alias facts (R4). | P4 (push may reallocate), P3 (storage reuse for a later insert), P2 (address reuse). Test: a demanded alignment or erasing store is present in the lowered IR after optimization. Proposed P13: storage relocated by a compacting third party. | RESTATED: widened to non-program relocation and physical demands (SYS-D0-01, SYS-D0-04, SYS-D0-09, SYS-REQ-04 merged) |
| **R9 Shared mutation among several holders** | Capability requirement: several long-lived pointers to one storage, each writing occasionally, is an accepted shape — sequentially with nothing beyond R1; across threads with a synchronization discipline that transfers the storage's state facts at acquire and release and costs only at the write. | Sequentially: nothing beyond R1. Concurrently: which primitive holds the state facts while no holder does; what is known before and after acquire. | Each access; acquire and release. | Forbidding aliasing to strengthen R4 (that trade is R4's and local to coinciding identities); choosing the determinism level (R14). | P7 sequential; P10 concurrent: synchronization cost only at the write, the facts known before and after acquire, and the determinism promise named. | RESTATED as capability; hazards owned by R1 (sequential) and R5 (concurrent) |
| **R10 External resources as ordinary objects** | Files, sockets, mappings, devices obey the same state, effect and proof rules as memory; no rule of the acceptance relation is conditioned on whether an implementation crosses the host boundary. | One identity per external resource; its states; which operations transition them; what an external actor may change between operations (supplied by R14). | Each operation, through ordinary contracts. | Host mechanisms or scheduling categories. | Theorem shape: the acceptance relation has no premise mentioning host crossing. Proposed P14: a descriptor as three identities (close-obligation wrapper, open-file description with cursor, foreign contents) with two wrappers sharing one description. | KEPT |
| **R11 Partial operations and overflow** | Every operation with a domain (index, division, narrowing, arithmetic under the overflow behavior chosen in its name, cast, hardware operation with preconditions) executes only inside its domain; silent overflow is unrepresentable; outside the domain the source either wrote a typed outcome (M2) or the program is rejected. | The domain of each partial operation; a discharge for it (a fixed automatic family or explicit steps). | Each partial operation. | Memory state (R1); choosing the discharge family (mechanism under M1). | P1 `read v[k]`: rejected unless k≠i ∧ k≠j is proved, diagnostic names that fact; P5 `f`, `g` overflow behavior chosen per operation. Theorem: progress lemma — no accepted program reaches a stuck state. | ADDED: constitution hazard with no owner in the map |
| **R12 Failure paths and typed outcomes** | Expected input and environment failures (allocation failure, short read or write, interruption, device error) have defined behavior: each is a typed outcome on a source-visible path; every R2 obligation discharges on the failure path; a context that must not fail in a given way (no-allocate, no-block) can state it and the checker enforces it. | The outcome type of each fallible operation; the failure edges each path adds to R2's join; the context's failure prohibitions. | At the fallible operation and at every join the outcome creates. | The outcome's spelling or routing mechanism (CALL-6 is one); unexpected failure outside R14's model. | P2 `alloc(A, 32)` on exhaustion: the outcome path frees nothing twice and leaves b1, b3 owned; P8 loop-exit obligations on every exit. Theorem: R2's counting invariant holds on every outcome path. | ADDED |
| **R13 Resource bounds and termination where demanded** | Where a use demands it, heap, stack, arena and other hardware-resource usage is provably bounded; where a licensed fact presumes termination (a removable call, EFF-3 purity), termination is proved; deadlock freedom and deadlines are demand-conditional in the same form. Unintended nontermination otherwise may remain (constitution). | A cost semantics per resource; a deterministic bound derivation; a termination-witness form. | At the demanding declaration; at each licensed fact that presumes termination. | Making bounds mandatory for every program; the accounting mechanism (envelope, extent items). | P2: arena total ≤ S provable; P6: helper marked removable only with a termination witness. Measurement: the derived bound equals or exceeds the measured peak on the corpus. | ADDED: moved out of R2's conditional clause; AI-REQ-11's unconditional form refused |
| **R14 Stated execution model and external conditions** | The specification states the model every safety theorem rests on: memory ordering and synchronization semantics; the determinism level promised per overlap construct (source-order equality, a named weaker level, or a reduction law); what non-program actors (allocator, OS, DMA device, debugger, another thread) may do to which storage and when; the runtime's rights over ended storage; the trusted external conditions. Anything outside is excluded, never undefined. | The enumerated assumptions; a per-storage qualifier for externally writable storage, spelled over interference ("another agent may write it without an edge in this program's order"), never over origin. | In the specification; M6's theorem takes exactly these assumptions and no others. | Selecting the level or the model; host mechanism categories (R10). | Theorem form: M6's statement lists no assumption absent from R14. P10: the determinism promise is named. P2: allocator metadata written inside freed b2 is admitted or excluded. A debugger is admitted as a non-interfering reader. | ADDED: constitution duty with no owner |
| **R15 Abstraction** | R1–R14 hold for generic, higher-order, nominal-abstracted, dynamically dispatched and module-separated code; identity, state and effect facts cross abstraction boundaries; instantiation terminates. | Identity and effect parameters on abstractions; the bound vocabulary; an instantiation-termination criterion. | At instantiation and at every abstraction boundary. | Signature size (M8 measures it); monomorphization strategy (mechanism). | P3 (pool generic over the node type), P9 `pick` (higher-order result carrying permission), P6 helper as a closure, P5 kernel generic over the column type. Theorem: M6 is proved for the language with abstraction, not for a first-order fragment. | ADDED |

### 1.2 Meta-constraints M1–M11

| Row | (a) must hold | (b) checker needs | (c) where | (d) not its job | (e) acceptance test | Status |
|---|---|---|---|---|---|---|
| **M1 Specified deterministic acceptance** | Acceptance is a total function of the source bytes and the specification, computed by a specified terminating procedure; no timeout, machine speed, solver state, iteration budget or portfolio order selects it; every accepted program's discharge is reproducible from the source alone (explicit steps are the certificate). | The specification of each admitted automatic family as a decision procedure with a stated bound (M10). | Specification; conformance corpus on two machines. | Forbidding a specification-fixed terminating fixpoint as such; "no SMT" (HIS-D0-02, adopted) is the chosen mechanism serving this row and M8. | Byte-identical verdicts and diagnostics across machines and runs; no acceptance-affecting knob exists in the compiler. | RESTATED |
| **M2 No unstated failure edge** | An accepted program has no executable failure edge (trap, abort, UB, silent wraparound) the source did not write; every condition that can be false at runtime is a typed outcome with source-visible control flow, visible at the operation's interface; the checker never inserts a check. | The failure edges of the lowering and their mapping to source outcomes. | At lowering (every abort edge maps to a written outcome) and at the interface. | Banning writer-written branches; pricing a written check (M9). Vale, Mezzo, CHERI, generational handles stay excluded: their check is a failure edge absent from source and interface. HIS-D0-04, -05, -07, -10 refusals stand. | P1 `read v[k]` with no runtime check; P4 `free(v); c.read()` refused, not trapped; P8 no drop flag. Theorem: abort edges of the lowered program ⊆ image of source outcomes. | RESTATED: "no runtime check" named as the over-broad mechanism reading |
| **M3 Enumerated, writer-closed trusted base** | Every assumption the checker does not establish (linked adapters, host declarations, axioms, R14's conditions) is enumerable from the source tree; each entry carries a machine-checked boundary obligation or is a named axiom; no writer-reachable construct adds an entry; no discharge is a bare assume. | The trusted-base list and each entry's obligation. | At the boundary declaration; the list is generated, never hand-kept. | Semantic non-vacuity (a weak `requires` or a defaulted result is a logic error the constitution leaves to the requirement author); where the base is hosted. | The tool prints the base; for the corpus it contains only adapters and R14 conditions; P9 and P10 add no entry. | RESTATED: "no unsafe keyword" named as the mechanism |
| **M4 Writer regularity** | One spelling per construct; no special cases; every rule stated once; every diagnostic local to the rule and tree location it cites. | The spelling inventory; the rule cross-reference. | Specification lint; diagnostics. | Token budget (M8); repairability (M7). | A spec-level lint finds one spelling per construct; each rejection cites one rule and one location. | SPLIT: budget → M8, repair → M7 |
| **M5 Acceptance independent of scheduling** | Whether a program is accepted never depends on parallel permission, cost model, target or schedule; a failed permission leaves the program sequential; the sequential semantics is the reference. | Nothing new: the acceptance relation must not mention the permission judgement. | Specification; theorem over the acceptance relation. | Performance (M9); the determinism level (R14). | Theorem: Accept(P) is invariant under removing all permission facts. Corpus: every current program stays accepted with permissions stripped. | SPLIT: "accepted shapes are the fast shapes" → M9 |
| **M6 Soundness as a theorem, with evidence** | The specification carries (i) an operational semantics; (ii) an adequacy theorem: accepted programs reach no UB, corruption, race, uninitialized read, overflow or unproved partial state, under exactly R14's assumptions, and a fact retained across an operation is true after it (R6's soundness half); (iii) an erasure theorem: lowering ignores proof terms except through R4's licensed facts, and observable behavior equals the semantics; (iv) a checker-defect channel that is compiler evidence, not a build mode: an exhaustive small-model check, translation validation, or a differential oracle (HIS-D0-16 stands: one observable behavior). | The semantics; the theorem statements; the oracle. | `research/experiments` small-model check before a candidate is selected; a formal model later (map §8 Q8). | Proving freedom from logic errors; a debug build. | Theorem shapes as stated; the access-state exhaustive check extended to every discriminating program; a seeded checker bug is caught by the oracle before any miscompile. | ADDED |
| **M7 Repairable rejection with machine-readable diagnostics** | Every rejection cites the rule, the tree location, and the missing fact or the restructuring it demands; the fix is applicable from the diagnostic alone; diagnostics are deterministic, byte-stable and parseable; no rule leaves the writer without a route (HIS-D0-15). | Per rule, its repair form. | Diagnostic emission; conformance corpus. | Guaranteeing that a repair loop terminates (measured under M10). | P1 `read v[k]` names k≠i and k≠j; P8 names the differing path. Measurement: blind-writer repair-from-diagnostic rate. | ADDED (split from M4) |
| **M8 Teaching and retrieval budget** | The taught surface (specification, pattern cards, and any prelude the writer must re-declare) fits a stated token budget K, addressable per rule; a construct's card is retrievable in one lookup. | Measured token counts per revision (META-5 practice). | Per specification amendment. | Choosing K here (measure it; 48k claimed vs ~130k measured means the constraint is unmet today); the checker. | Tokens ≤ K; re-declared prelude lines per file → 0. | ADDED (split from M4; prelude and retrieval merged) |
| **M9 Measured performance floor** | On a locked corpus, the first checker-green artifact of a fixed writer with no hints meets a stated ratio against a named baseline with all checks in place; the obvious shape is within a stated factor of the restructured shape. | Corpus, baseline, ratio, protocol. | `research/experiments`, per candidate. | Acceptance (M5); promising speedup from permission (HIS-D0-20 refused). | Reproduce (1.65x, 1.10x) in the current compiler; P5 obvious loop within the stated factor of the hand-restructured loop. | ADDED (M5's fast-shape clause made testable) |
| **M10 Checking cost, scaling and edit stability** | Each admitted automatic family has a stated worst-case work bound polynomial in a named measure (statements, identities, explicit steps); whole-project check time and per-edit recheck scale as stated; an edit rechecks only its stated dependency cone. | The bound per family; the dependency-cone definition. | Specification (bound) plus measurement. | Budget-selected acceptance (M1 forbids it); claiming more annotations reduce cost (HIS-D0-21 refused). | Measurement: proof-use-cost `growing` 30→163→1689 ms for N=16→32→64 (~N^2.9) is compared against the stated bound; a one-token edit in P9's caller reopens no callee proof. | ADDED |
| **M11 Evolution with stated loss** | A specification revision states the set of previously accepted corpus programs it rejects and a mechanical migration for each, or states the loss; explicit steps survive a revision that does not change their rule. | The conformance-corpus verdict diff per revision. | Per amendment. | Backward compatibility as a hard rule (the constitution lets it yield). | Verdict diff reported; migration applied mechanically to the corpus. | ADDED (conditional) |

Row count: 15 R (R6 retained as a reclassified marker) + 11 M = 26.

## 2. Premises

| Premise | Classification | Serves | If dropped |
|---|---|---|---|
| **FN-1** signature-only checking | Mechanism, not a requirement. | R7 (one way to realise a stated interface) and M10 (bounded recheck: the cone stops at the interface). | Whole-program inference becomes admissible; R7 still requires an interface for the compositionality theorem, so an inferred interface must be pinned and printed; M10's cone must be re-stated without the interface as its boundary, else per-edit recheck is unbounded. |
| **PROG-1** closed world | Mechanism, not unspent (RAD-D0-14 refused). | M3: with every body in the tree, the trusted base is adapters and R14 conditions only; M6: the adequacy theorem quantifies over a closed program. | Separately compiled interfaces become trusted-base entries that M3 must enumerate with a boundary obligation; M6 gains an assumption ("imported interfaces are honored"); FN-1's annotation cost then buys separate compilation, which today it does not (SYS-D0-16's double cost is real and is a mechanism cost, not a requirement conflict). |
| **M1** deterministic, budget-free checking | Requirement (restated: specified total deterministic acceptance). "No SMT" is the adopted mechanism serving it. | M6 (the acceptance relation is a function the theorem can quantify over); M7; M8. | Acceptance becomes implementation-defined; the theorem loses its object; a fixed-portfolio deterministic search with certificates would still satisfy the restated row, which is why the row names the mechanism separately. |
| **M2** no runtime safety check | Requirement (restated: no unstated failure edge). "No runtime check" is the mechanism reading. | M3 (a trap is an unenumerated failure edge); M6 (iii) erasure; R12 (failures are typed outcomes). | Vale/Mezzo/CHERI-shaped designs re-enter; every interface gains an implicit failure edge the caller cannot read; M6's erasure theorem must then account for inserted checks. |
| **M3** no unsafe | Requirement (restated: enumerated, writer-closed trusted base). "No unsafe keyword" is the mechanism. | M6 (the theorem's assumptions are exactly M3's list plus R14); the why-whitefoot floor argument. | The floor argument fails (a stuck writer uses the escape); M6's theorem acquires per-site assumptions no tool enumerates. |
| **M5** fast shapes only | Split: acceptance-independence is a requirement (M5); "accepted shapes are the fast shapes" is a goal, kept as measured floor M9; "sequential rather than rejected" is M5's operational statement. | M5 serves M6 (permission facts are not in the acceptance relation, so the theorem is schedule-free); M9 serves the constitution's performance objective. | Without M5, acceptance depends on the cost model and the theorem must quantify over schedules; without M9, the floor claim is unmeasured and why-whitefoot §1 is unsupported. |

Reclassified in this draft: FN-1 (mechanism), PROG-1 (mechanism, spent by M3/M6), M1's no-SMT clause (mechanism), M2's no-runtime-check clause (mechanism), M3's no-unsafe-keyword clause (mechanism), M5's fast-shapes clause (goal → measured M9).

## 3. Disposition table

### 3.1 D0.md families (22)

| Family | Disposition | Reason |
|---|---|---|
| Keep the map's list verbatim | refuse | §7's couplings already contradict the claimed independence, and the constitution's overflow, failure-behavior and execution-model duties have no owner in the verbatim list; freezing it freezes the gaps. |
| Refactor the R1–R3 state ladder | adopt in part (R1 split internally); merges refused | Three named lemma obligations under one row give each hazard its own theorem without three vocabularies; R2/R3 and R3/R1 merges are refused because the multiplicity lemma and the state lemma compose in M6 only if separate, and R2's join rule has no R3 counterpart (Clean's obligation-vs-uniqueness distinction). |
| Interference rows are one fact (R4, R5, R9) | refuse merge; adopt as M6 coupling; R9 restated | R5 is a safety row and cannot inherit R4's tolerance for a missing fact; M6's theorem must derive both from one distinctness relation, which states the coupling without collapsing the exactness directions; R9 becomes a capability row (P7, P10). |
| R6 relocated | reclassify as mechanism | Soundness half is M6 (ii); precision half is checker precision tested by P1/P4; the R7-framing destination is refused because intra-procedural kills must not need a signature-shaped statement per write. |
| R4's standing and the optimizer's authority | adopt restated (R4) | Deleting R4 (RAD-D0-01) is refused: a fact the backend uses is a soundness surface and the constitution ranks performance high; re-derivation (RAD-D0-25) is a mechanism because the requirement binds what is used, not who derived it. |
| R7 / FN-1 / PROG-1 reclassified | adopt (R7 restated; FN-1, PROG-1 mechanisms) | Signature-only judgement is one way to realise a stated interface; PROG-1 is not unspent (RAD-D0-14 refused): it is what keeps M3's trusted base to adapters; SYS-D0-16's double cost is answered in Premises as a mechanism cost. |
| M1 restated as reproducible acceptance | adopt (M1 restated) | The requirement is a specified total deterministic acceptance function with M10's bound; no-SMT (HIS-D0-02) stays the adopted mechanism; a specification-fixed fixpoint qualifies, an order-dependent portfolio does not. |
| M2 restated as no trap and no unchosen cost | adopt as "no unstated failure edge" (M2) | Keeps Vale/Mezzo/CHERI excluded because their check is an interface-invisible failure edge, so MAP §5's disqualifications survive the restatement; the cost of a written check goes to M9, not M2; the HIS-D0 refusals stand. |
| M3's boundary: escapes and non-vacuous discharge | adopt (M3 restated); AI-D0-08 refused | Enumerated trusted base with a boundary obligation per entry (RAD-D0-10); bare assume stays refused (HIS-D0-06); non-vacuity is a logic-error property the constitution assigns to the requirement author, not to the checker. |
| Failure paths and typed outcomes as an owned requirement | adopt as R12 | Without an owner "typed outcome" is an escape valve; the row owns outcome typing, no-allocate contexts, and discharge of R2 obligations on failed paths; CALL-6 routing is a mechanism. |
| Writer cost model: M4 restated and split | adopt split (M4, M7, M8) | Each sub-requirement gets its own test; prelude (AI-D0-10) and retrieval (AI-D0-17) fold into M8's budget; one-spelling lint (AI-D0-20) is M4's acceptance test; HIS-D0-21 refusal stands. |
| Repairable rejection and machine-readable diagnostics | adopt as M7 | Makes "mechanical fix" a rule property; repair-loop termination is measured under M10 rather than promised. |
| Checking cost and latency as a requirement | adopt as M10 | Answers SYS-D0-15: M1 gets a bound via M10 and PROG-1 remains a mechanism; proof stability under edit (RAD-D0-20) is M10's cone clause. |
| M5 and performance guarantees | adopt split (M5, M9); RAD-D0-12 reclassified; SYS-D0-13 merged into R14 | Acceptance-independence is checkable as a theorem, the floor only by measurement; a schedule language is a mechanism; the determinism menu belongs to the execution model; HIS-D0-20 refusal stands. |
| Resource bounds, termination and time | adopt as R13 | Demand-conditional as the constitution words it; termination is mandatory only where a licensed fact presumes it (EFF-3), which makes it an R4 soundness matter, not a global liveness promise. |
| Abstraction carries identity and effect facts | adopt as R15 | A candidate scored on a first-order fragment may be unwritable under generics; signature-size cost is measured under M8. |
| Evolution and migration | adopt as M11 (conditional) | The constitution lets compatibility yield, so the requirement is stated loss plus mechanical migration plus certificate survival, all checkable on the corpus. |
| Admission policy for missing requirements | adopt THE-D0-06 for hazard-owner rows, THE-D0-07 for capability rows | Every constitutional hazard gets an owner now; capability rows enter when a discriminating program blocks; HIS-D0-12/13 refusals of frequency stand. |
| Form of the requirement statement | adopt THE-D0-08 (M6) and THE-D0-09 (column e); THE-D0-10 and AI-D0-21 refused | Theorems make independence checkable and programs make precision checkable; the lattice is kept as §7's coupling table, not as the form; deleting a requirement for want of a program makes the list hostage to the suite. |
| Hardware placement, ordering, address spaces and secret erasure | merge into R8, R14, R4 (split) | Layout, address space and erasure demands are placement demands (R8); ordering is execution-model vocabulary (R14); protection of a demand from elision is R4's bound clause; they are demands on lowering, not hazards, so no new row. |
| Observability, debug builds and a checker-soundness oracle | merge into M6 (iv) and R14 | The oracle is compiler evidence, never a build mode, so HIS-D0-16 stands and HIS-D0-17 stays superseded; a foreign reader is an R14 non-interfering actor. |
| Storage shapes outside the state model | merge as acceptance tests: R1 (P11), R2/R10 (P12), R8/R14 (P13), R14 (allocator writes) | Each shape must be admitted or excluded by R14, never left undefined; none is a new hazard, each is a program the existing rows must answer. |

### 3.2 REQ.md families (23)

| Family | Disposition | Reason |
|---|---|---|
| Checking cost as a requirement | adopt as M10 | A stated bound distinct from budget-freedom; the measured N^2.9 growth is the first number the bound must be set against. |
| Abstraction as a requirement | adopt as R15 | Includes instantiation termination (HIS-REQ-07); the theorem must be proved for the abstracted language. |
| Resource bounds as a requirement | adopt as R13; AI-REQ-11 refused | The constitution conditions bounds on explicit budgets, so the unconditional form overreads it; R2's conditional clause moves out so a candidate cannot satisfy R2 while missing the constitution. |
| Failure, partiality and typed outcomes | adopt as R12 (failure) and R11 (partiality, overflow) | Two hazards, two owners; the R2 join interaction is R12's acceptance test. |
| M4 decomposition: teachability, spec size, auditability | adopt (M4, M7, M8) | Human auditability (THE-REQ-05) is served by M7's cited diagnostics and M8's measured size; META-5 counting is M8's test. |
| Repair, edit stability, evolution | adopt across M7, M10, M11 | Three properties with three tests; OWN-8's named restructuring is M7's repair form. |
| Standard library and prelude cost | merge into M8 | The re-declared prelude is the largest measured token cost and counts against the budget. |
| Layout and memory ordering | merge into R8 and R14 | Layout is a placement demand; ordering is execution-model vocabulary R5's race-freedom theorem rests on. |
| Third-party mutation of storage | merge into R8 and R14 | Relocation by a non-program actor is a storage-ending event; allocator rights over Gone storage are an R14 statement. |
| R5 scope: reductions and determinism level | adopt into R5 and R14 | Reductions are admitted under a proved operator law; the level is named per construct in R14 rather than fixed by R5. |
| Checker observability | merge into M6 (iv) | The channel is a soundness oracle in compiler evidence; erasure and no-trap are preserved because the oracle is not in the accepted program. |
| R4 and R5 are one relation | refuse as merge; adopt as M6 coupling | One distinctness relation, two exactness directions; merging would make a missing R4 fact an R5 rejection. |
| R4 and R6 are precision policies | adopt for R6; partial for R4 | R6 is reclassified; R4's soundness clause stays a requirement; HIS-REQ-02 is honored: the channel is compiler detail, the licensing is language. |
| R6 not independent of R4 and R7 | adopt | R6 retired; the kill rule follows the contract vocabulary and is checked once under M6 (ii). |
| R7 / FN-1 versus PROG-1 | adopt reclassification | R7 restated as compositional judgement; HIS-REQ-06's declaration-side obligation adopted into R7 (a). |
| R9 dependence on R1, R4, R5 | adopt | R9 restated as a capability with hazards owned by R1/R5; HIS-REQ-01/10/12 record that today's mechanism over-serves, and the requirement is not weakened to match it. |
| R2, R3, R8 and R1 lifecycle dependence | refuse merges; adopt the storage-ending set as a shared coupling | The join rule is not duplicated because R3 has none; the storage-ending set is stated once (R1 (b)) and checked under M6. |
| R1 bundles three properties | adopt as internal split | Three lemma obligations under one row; whether one vocabulary serves all three is the candidate's to justify. |
| M1 "no SMT" is a mechanism | adopt | Restated as reproducible, specified acceptance; the adopted no-SMT decision is named as the mechanism serving it. |
| M2 "no runtime check" is a mechanism | adopt | Restated as no unstated failure edge; THE-REQ-17's trap-vs-conditional distinction is the restatement itself. |
| M3 "no unsafe escape" hole | adopt (enumerated trusted base); AI-REQ-04 refused as a checker requirement | The trusted adapter at R10 is an admitted, enumerated exception with a boundary obligation; a weakened `requires` is a logic error outside the checker's charge. |
| M5 is a goal, not a checkable property | adopt split (M5, M9) | The checkable part is acceptance-independence; the goal becomes a measured floor with a protocol nobody had assigned. |
| Erasure as a checkable requirement | adopt as M6 (iii) | An erasure theorem with a validation obligation, since a constitutional premise with no theorem is an unchecked assumption of every other row. |

Families dispositioned: 22 + 23 = 45 of 45.

## 4. What this draft costs

| Accepted cost | Who would resist | Why this draft accepts it |
|---|---|---|
| 26 rows instead of 15, with a theorem form (M6) and an operational semantics required before any candidate is selected. | A velocity or writer-cost priority: taught vocabulary grows, D13 is delayed, and the spec-size budget (M8) is already unmet. | An unowned hazard is an unaudited hazard; a requirement without a theorem shape cannot be shown independent of its neighbours, so the §7 couplings would remain prose. |
| R5 stays strict and separate from R4: a missing distinctness fact is a rejection for overlap, never a tolerated slowdown, and no reduction overlaps without a proved law. | A performance priority: merging R4/R5 and admitting compiler-side recovery would take free wins and halve the vocabulary. | Race freedom is a safety row and inherits no tolerance; the coupling is stated where it is checkable (M6), not by collapsing rows. |
| M2 restated so that any interface-invisible failure edge is refused, keeping generational, epoch and CHERI-style designs out even where the check would be cheap; M3's non-vacuity is left to the requirement author. | A pragmatic priority: cheap dynamic boundaries would admit gradual designs and shrink proof burden; an oracle for weakened specs would catch reward-hacking writers. | The trap surface must be enumerable from signatures for the theorem to have a fixed assumption set; semantic correctness is outside the safety boundary the constitution draws, and pretending the checker owns it would make the list unsatisfiable. |


# File: draft-B.md

# Draft B — requirement list, performance and systems practice first

Stance: a requirement earns its row by naming a fact a backend or a scheduler
consumes, a hazard a real kernel, database or HPC program hits, or a cost the
compiler must not silently pay. Every row carries a **Price**: what it forbids
the compiler from doing. A row with no price and no consumer is a mechanism or
a policy, and is marked so. No mechanism is selected here.

Notation: `(a)` what must hold, `(b)` the least a checker must know, `(c)`
where it is checked, `(d)` not its job, `(e)` acceptance test. `P1..P10` are
the discriminating programs. "Kept", "restated", "split", "merged",
"reclassified" and "new" say what happened to the map's row.

## 1. Revised list

### R1 Sequential memory safety — restated

- (a) No read of storage holding no value; no program access to storage that
  has ended (freed, reallocated, released at scope exit, vacated by a
  relocating move, reclaimed by a third party per R19); no access under a
  stale layout. Ended storage is *inaccessible to the program*, not inert: the
  allocator (R21) or a foreign agent (R19) may write it.
- (b) For the storage an access names: live, initialized in the selected part,
  layout current. Properties of the storage, not of the pointer. Address reuse
  never revives an ended identity.
- (c) At each access, against the named storage's state at that point.
- (d) Not its job: preventing two pointers to one storage; preventing
  sequential aliased writes; parallel overlap; what a non-program actor does
  to ended bytes.
- (e) P2 (pointer into freed `b2` refused after `b4` reuses its bytes), P4
  (`c.read()` after `free(v)` refused), P7 (`read(p)` after `free A` refused
  forever), P8 (no drop flag). Theorem shape: progress for every accepted
  access under a store model where ended addresses may be reused.
- Price: forbids nothing at runtime; forbids the compiler from treating `Gone`
  bytes as unobservable garbage it may write freely (R21 owns that).
- Change: THE-D0-02's three-way split refused (see disposition). The "Gone is
  inert" reading is dropped: SLUB free pointers and tcache `next` live in
  exactly that storage.

### R2 Resource lifecycle accounting — split

- (a) Every allocation and external resource is released exactly once on
  every path, including typed-outcome paths (R14); linear obligations are
  discharged. (The bounded-memory clause moves to R15.)
- (b) Which binding holds the release obligation of which storage; whether it
  is definitely discharged on every path, including each outcome arm.
- (c) At consuming operations, scope exits, joins, and every outcome branch.
  Release decisions are static.
- (d) Not its job: who may read or write; how much memory the program uses.
- (e) P2 (four frees, no double), P8 (conditional release and loop exits with
  the writer's own branches carrying state), P2 under R14: a failed `alloc`
  outcome arm still discharges obligations acquired before it.
- Price: forbids the compiler from inserting a drop flag, an epilogue
  cleanup, or an unwinder; the release schedule is the source's.

### R3 Value classes and transfer — kept

- (a) Copy values duplicate freely; affine values move at most once and may
  take a compiler-derived release; linear values are consumed exactly once.
  Move transfers value plus obligations; take leaves a hole; put fills; replace
  exchanges. Whether an aggregate travels as bytes or as a handle is a stated,
  ABI-visible property of the class (R11).
- (b) Each value's class; per transfer, its source and destination storage;
  bytes-or-handle per class.
- (c) At the transfer.
- (d) Not its job: pointer validity. A value leaving `'a` is an event on
  `'a`'s state (hole or end) that R1 consumes.
- (e) P7 (take leaves a hole another alias may fill; replace reads new), P3
  (link writes through several paths to one node legal sequentially).
- Price: forbids the compiler from choosing byte-copy versus handle-pass per
  call site on its own; the class fixes the calling convention.

### R4 Aliasing, footprint and frame facts for the backend — merged (R6 in), restated

- (a) The backend receives, without guards, facts it may lower to target
  metadata: two accesses never overlap; a storage is not written during a
  span; a loaded value is invariant over a span; a callee's summary says
  which argument storages it reads, writes, ends, or returns pointers into.
  The same write-footprint fact decides which checker facts survive a write
  or a call (former R6): exactly the facts whose support lies in the written
  footprint are dropped.
- (b) Distinctness of storages by identity, field, proved index or proved
  range; read and write footprints per operation and per span; callee
  footprints per argument storage. This is the same relation R5 reads; R4
  tolerates a missing "distinct" (loses speed), R5 does not (loses
  permission).
- (c) At lowering, from retained checked facts; intra-procedurally at every
  write and call for fact retention.
- (d) Not its job: safety. A missing fact costs speed, never correctness.
  Not its job to forbid the backend's own analysis: the backend may *add*
  facts by analysis; it may never *drop* a stated fact, and acceptance never
  depends on what analysis finds.
- (e) P5: eight columns lowered with per-load distinctness, vectorized with no
  runtime overlap guard; measurement: the emitted IR carries the stated
  facts (`noalias`/scope, `readonly`, `memory`, `initializes` or their
  equivalents) for every P5/P6 load and call, and wfgrep and zlib-core-kernels
  lose no overlap that PAR-1/PAR-2 permit today. Frame half: after `write(s.y)`
  the fact on `s.x` survives (ladder Step 1).
- Price: forbids the compiler from *requiring* re-derivation to reach the
  stated facts, and from emitting a fact the source did not check. Does not
  forbid extra analysis (M6).

### R5 Data-race freedom and parallel permission — restated

- (a) Two statements, iterations or threads may overlap only when their
  footprints do not interfere under the memory model R12 names; a data race is
  unrepresentable. Overlap on a shared accumulator is admitted when the
  operator is declared associative and the permission names its determinism
  level. Every permission judgment states which of a closed set of determinism
  levels it guarantees (at least: source-order equality; associative
  reordering with bitwise-reproducible result; associative reordering with
  unspecified result order). A level weaker than source-order equality is
  admitted only where written.
- (b) Read and write footprints per statement over storages; distinctness
  between footprints (the R4 relation, read exactly); data dependencies;
  the operator's declared law for a reduction; sync edges (R12).
- (c) PAR-1 windows and PAR-2 loops today; a thread construct at spawn and
  join; a reduction at its combine point.
- (d) Not its job: sequential safety; choosing the schedule, tile, or worker
  count (M6). Permission never changes acceptance (a failed permission proof
  leaves the program sequential, M5).
- (e) P1 (two writes overlap; `v[k]` read needs `k≠i,j` or a branch), P5
  (PAR-2 iteration overlap), P6 (overlap from range arithmetic and the
  helper's signature alone), P10 (level promised or not), compute-bench
  accumulator snapshot cases at each level. Theorem shape: race-free under
  R12's model implies the promised level.
- Price: forbids the compiler from reordering across a stated sync edge, and
  from picking a level weaker than the one written. Forbids nothing about
  how it schedules within the permitted set.

### R6 Frame: proof-fact retention — merged into R4

- The soundness half ("drop at least the facts whose support the write
  reached") is R4(a)'s write-footprint fact consumed by the checker instead
  of the backend. The precision half ("drop no more") is a cost, priced under
  R13 (edit stability) and M7 (a spurious kill must be repairable from the
  diagnostic). No content dropped; the taught vocabulary loses one row.

### R7 Bounded, checked call summaries — restated (FN-1 reclassified)

- (a) A call site's judgment, and the backend's facts at that call, depend
  only on a bounded summary of the callee; the summary is checked once
  against the body and states, per argument storage: entry state required,
  exit state left (hole, ended, replaced backing, returned pointer into an
  argument, fresh allocation), read/write/end footprints, outcomes (R14),
  resource effects (R15), and ordering edges (R12). A declaration whose
  summary no caller can use is itself an error (declaration-side obligation).
  Whether the summary is writer-written or derived and pinned is a mechanism.
- (b) Storage identities nameable in summaries; entry and exit states per
  identity; footprints per identity; composition of summaries under
  instantiation (R16).
- (c) At the call for the caller; once at the returns for the callee.
- (d) Not its job: fixing whether summaries are inferred; fixing compilation
  units; forbidding the backend a whole-program view it can get (M6).
- (e) P9 (`drain`, `reserve`, `pick` each visible in the summary; caller
  judges by summary alone), P6 (the helper's summary alone tells the caller
  what it writes). Measurement: recheck after a one-line body edit touches
  only the callee and callers whose summary changed (R13).
- Price: forbids the compiler from making acceptance of a caller depend on
  a callee body. Does not forbid the backend from inlining or LTO.

### R8 Storage placement and relocation by the program — restated

- (a) A value lives somewhere: frame, heap store, arena, slab, inline in a
  container, or a placed address (R18). Program operations that relocate or
  replace storage (relocating move, reallocating push, arena reset,
  `mremap` issued by the program) are contract-visible events on the source
  storage; interior pointers name the storage that existed before the event.
  Relocation the program did not issue is R19.
- (b) Which operations end or replace which storage; whether aggregates
  travel as bytes or handles (R3); the backing identity of a container.
- (c) At the relocating operation, as a state event (R1) and a summary clause
  (R7).
- (d) Not its job: who may access; layout width or alignment (R11); address
  space (R18).
- (e) P4 (validity of `c.read()` after `push` decided by `len0 < cap0`, not by
  a borrow), P2 (arena blocks exclusively owned, metadata written only at
  alloc/free), P3 (pool storage reuse invalidates every path to `m`).
- Price: forbids the compiler from relocating storage on its own (no moving
  GC, no compiler-introduced copy that changes an identity's address while an
  interior pointer names it) unless R19's fixup channel is used.

### R9 Shared mutation among several long-lived holders — restated

- (a) Several long-lived pointers to one storage, each writing occasionally.
  Sequentially this is R1 and nothing more (clause moved). Across threads
  each write is synchronized (R12); the cost of synchronization lands on the
  write, never on the hold, never on a read that does not need it; the
  checker states what it knows of the storage's contents facts after acquire
  and after release. Global mutable state is one instance, not a separate
  rule.
- (b) Concurrently: which primitive holds the storage's state facts while no
  thread holds them; the ordering edges of acquire and release; the
  determinism level promised (R5).
- (c) At acquire and release; at the write.
- (d) Not its job: forbidding aliasing to strengthen R4; that trade is R4's
  and is local to the storages whose identities coincide (R9 is the negation
  of R4's exclusivity on exactly those storages).
- (e) P10: two writers increment occasionally, a third reads; measurement:
  zero instructions on the hold, one synchronization per write, reader cost
  stated; P3 sequential link writes need no judgment. Theorem shape: a
  holder that never writes contributes no ordering edge.
- Price: forbids the compiler from inserting a lock, a fence, or a
  generational check on a hold or a read the source did not synchronize.

### R10 External resources as ordinary objects — kept

- (a) Files, sockets, mappings, devices, descriptors obey the same state,
  effect, ordering and proof rules as memory. An external role grants no
  source-language exception; the trusted adapters that implement it belong to
  M3's enumerated base.
- (b) One identity per external resource; its states; which operations
  transition them; what an external actor may change between operations
  (a foreign-interference fact, R19).
- (c) At each operation, through ordinary summaries (R7).
- (d) Not its job: host mechanisms or scheduling.
- (e) A descriptor case: wrapper with close obligation (R2), open-file cursor
  state, contents any agent may write (R19) — all three judged by ordinary
  rules; an EINTR/short-read outcome (R14) on the same path.
- Price: forbids a compiler-special path for I/O values.

### R11 Layout and ABI — new

- (a) The writer can state, and the checker knows, for every type and every
  storage: size, alignment, field order and padding, SoA/AoS shape,
  cacheline placement, and the representation a foreign boundary sees. A
  stated layout is a checked fact the backend must honor and may consume
  (vectorization legality, `align(n)`, contiguity). Layout currency (R1) is
  judged against this layout.
- (b) A deterministic layout function per type; alignment per storage;
  contiguity per container backing; which fields share a cacheline.
- (c) At type definition; at the foreign boundary; at pointer formation.
- (d) Not its job: aliasing (R4), address space (R18), erasure (R20).
- (e) P5: columns declared contiguous and aligned; emitted loads carry the
  alignment; the loop vectorizes with no peel for alignment. zlib-core-kernels:
  a stated `align(64)` on the hot table reaches the IR. Foreign boundary: a
  struct's C layout is checked, not assumed.
- Price: forbids the compiler from reordering fields, padding, or choosing
  SoA/AoS where a layout is stated. Where none is stated the compiler is
  free (M6).

### R12 Memory ordering — new

- (a) The language names the memory model against which race freedom (R5)
  and shared mutation (R9) are stated. Every cross-thread or cross-agent
  access carries its ordering; synchronization primitives are declared by
  their acquire/release edges; the happens-before relation is a checker
  fact. Where R19 storage is involved, the ordering includes the foreign
  agent's edges (DMA barriers, `__iomem` ordering).
- (b) An ordering vocabulary; the sync edges of each primitive; the program
  order; the model's rule for what a race-free program observes.
- (c) At each atomic or synchronizing operation; at spawn/join; at R19
  hand-off points.
- (d) Not its job: choosing fences per target; sequential code.
- (e) P10 (stated orderings; checker facts before/after acquire). Theorem
  shape: DRF-guarantee for the named model. Measurement: a seqlock or
  phase/epoch pattern written in source emits exactly the writer's barriers.
- Price: forbids the compiler from reordering across a stated edge and from
  strengthening or weakening an ordering. Forbids nothing on unordered
  private accesses.

### R13 Checking cost, scaling law and edit stability — new

- (a) Each automatic family has a stated complexity bound in source size and
  written proof steps (a fixed polynomial degree, published per family);
  total check time on the corpus is measured against that law; an edit
  reopens only the proofs whose support it touched (the R4 write-footprint
  relation run over the edit). Termination alone is insufficient.
- (b) The cost of each derivation step; the support of each retained fact;
  the dependency between summaries (R7).
- (c) Per family, at specification time (the bound is part of the spec); per
  release, as a measured scaling law; per edit, as an incremental recheck.
- (d) Not its job: reproducibility (M1); runtime performance (M5).
- (e) Measurement: proof-use-cost `growing` at N=16/32/64 must fit the
  published law (today 30→163→1689 ms is not quadratic); a one-token change
  in P9's `drain` body rechecks `drain` and its direct callers only. Theorem
  shape: fact support is a function of footprints, not of position.
- Price: forbids a checker family whose worst case exceeds its published
  bound; forbids per-statement proof forms with an unnamed superlinear
  price. Forbids nothing of the backend.

### R14 Failure paths and typed outcomes — new

- (a) Every partial operation whose domain is not proved yields a typed
  outcome with real control flow; environment failures (allocation failure,
  short read, EINTR, timeout, device error) are outcomes with defined
  behavior; a context may declare that it does not allocate or does not
  block, and the checker refuses an allocating or blocking call there. On
  each outcome arm R2's obligations still discharge.
- (b) Each operation's outcome type; each context's allocation/blocking
  effect; the join rule for obligations across arms.
- (c) At the operation; at the context boundary; at each arm's exit.
- (d) Not its job: preventing the failure; the trap ban (M2) or the
  discharge-by-weakening ban (M3).
- (e) P2 with `alloc` allowed to fail: the failing arm frees `b1`, `b3`; a
  `GFP_ATOMIC`-shaped context refusing a sleeping allocation; a partial write
  loop over a socket carrying the short-write outcome.
- Price: forbids the compiler from eliding a written outcome arm or from
  inserting one; forbids an allocation the source did not write in a
  no-allocate context.

### R15 Resource bounds, termination and deadlines — new (takes R2's bound clause)

- (a) Where a bound is promised, the checker proves it: peak memory by
  tangible resource (heap, stack depth, arena extent, lanes, descriptors),
  allocation count, and, where declared, termination, deadlock freedom (lock
  order) and a deadline. Unpromised programs are unaffected.
- (b) A cost semantics per operation for each resource; a deterministic
  bound derivation; a lock-order relation; a termination measure supplied by
  the writer.
- (c) At the declaration promising the bound; per call through R7 summaries.
- (d) Not its job: performance (M5); failure behavior (R14).
- (e) P2 with a declared arena extent: `b4` fits or the program is refused;
  an embedded shape with a declared stack depth; two locks acquired in
  conflicting orders refused where deadlock freedom is promised. Measurement:
  the derived bound equals the measured peak on wfgrep.
- Price: forbids the compiler from introducing allocation, stack growth or
  blocking where a bound is promised (no compiler-inserted heap temporaries).

### R16 Abstraction carries identity, effect and layout facts — new

- (a) Generics, nominals, closures and dynamic dispatch carry the facts of
  R4, R5, R7, R11, R12 through instantiation and through indirect calls;
  a candidate that holds on the first-order fragment must hold under
  abstraction with a summary size bounded in the number of identity and
  effect parameters. Instantiation terminates by a syntactic rule.
- (b) Summary composition under substitution; how a closure or vtable slot
  carries its summary; a bound on summary size; the instantiation
  termination rule.
- (c) At instantiation; at each indirect call against the slot's summary.
- (d) Not its job: choosing monomorphization versus dictionaries.
- (e) P6's helper made generic over element type and slice identity; P3's
  parallel map through a closure argument retains node distinctness; P9's
  `pick` returning through a trait object still names its permission.
  Measurement: summary tokens per generic parameter stays linear on the
  corpus.
- Price: forbids the compiler from losing a stated fact at an indirect
  call; permits it to specialize.

### R17 Write-once-then-frozen storage — new

- (a) A storage may transition `Uninit → Init` exactly once under a stated
  protocol (lazy init, memoization, static key patching, plan cache) and
  thereafter be read by many holders and threads with no synchronization;
  the transition point is a contract-visible event; reads after it yield
  invariant-load facts (R4).
- (b) The protocol's transition operation; that no write exists after the
  transition; the sync edge of the transition (R12).
- (c) At the transition; at each read (state must be `Init` or the read is
  in the initializing arm).
- (d) Not its job: caches that are rewritten (those are R9); eviction (R19).
- (e) A `OnceLock`-shaped cell read from P10's third thread with zero cost
  after init; a memo table in P3's traversal. Measurement: post-init reads
  emit an invariant load, no guard.
- Price: forbids the compiler from assuming `Init` from birth and from
  inserting a guard on post-init reads.

### R18 Address spaces — new

- (a) A storage identity carries its address space (user, kernel, device
  register, DMA-coherent, GPU local/shared/global, persistent). A pointer is
  dereferenced only in a context that can reach its space; copies across
  spaces are explicit operations with their own summaries (R7). Cross-space
  distinctness is a backend fact only where the target guarantees it (the
  catalog's LLVM claim is unverified and is not assumed).
- (b) The space of each identity; reachability per execution context; which
  operations cross.
- (c) At pointer formation and dereference; at the crossing operation.
- (d) Not its job: aliasing within a space (R4); ordering (R12).
- (e) A kernel copy-from-user shape: a `__user` dereference in kernel context
  refused; a GPU kernel over P5's columns with columns placed in global and
  a tile in shared memory, distinctness stated per space. Measurement: the
  emitted IR carries the space.
- Price: forbids the compiler from assuming cross-space non-aliasing on its
  own; forbids it from moving storage across spaces.

### R19 Third-party relocation, reclamation and loans to non-program agents — new

- (a) Storage that a non-program agent may write, relocate, reclaim, or
  complete asynchronously (DMA, io_uring registered buffers, a moving
  collector, `mremap` by a runtime, buffer-pool eviction, device registers,
  shared mappings) is qualified by that interference; no contents fact about
  it survives an operation the agent could have interleaved; a loan to the
  agent is a linear obligation with a program-visible completion point (the
  completion token or the unmap); a relocation the agent performs is either
  unreachable by program pointers at that point or arrives through a
  contract-visible fixup.
- (b) Which identities are foreign-writable; the agent's edges (R12); the
  loan's completion operation; the relocation fixup channel if any.
- (c) At every operation on a qualified identity; at loan and completion.
- (d) Not its job: the program's own relocations (R8); allocator reuse (R21).
- (e) A `dma_map_single`/`dma_unmap_single` pair: reads of the buffer between
  map and unmap retain no contents facts, the unmap discharges the loan; an
  io_uring SQE with a registered buffer whose CQE is the completion point;
  an evicted LeanStore page whose swizzled pointer must be refused after
  eviction. Measurement: extent facts survive, contents facts do not.
- Price: forbids the compiler from caching a load or deleting a store on
  foreign-writable storage; forbids nothing elsewhere.

### R20 Secret erasure and constant-time discipline — new

- (a) A store the writer marks as erasing is executed even when its value is
  never read; a region the writer marks constant-time has no
  secret-dependent branch, memory index, or variable-latency operation
  after optimization; both survive every check-removal and optimization a
  proof authorizes.
- (b) Which identities hold secrets; which stores are must-execute; which
  regions are constant-time; a target's variable-latency operation set.
- (c) At the marked store; at lowering of the marked region (a backend-side
  check that the emitted code kept the property).
- (d) Not its job: unmarked code; cryptographic correctness.
- (e) A key buffer zeroed before `free` in P2's shape: the store is present
  in the emitted code. Measurement: an `explicit_bzero`-shaped store is not
  dead-store-eliminated; a marked comparison compiles branch-free.
- Price: forbids dead-store elimination on marked stores and
  branch-introducing or table-introducing transforms inside marked regions.
  Forbids nothing elsewhere.

### R21 Allocator contract — new

- (a) Allocation, reuse and release are ordinary contracts (R7, R10): an
  allocator may write metadata inside ended storage (free lists, poison);
  address reuse yields a fresh identity (R1); an allocation returns storage
  distinct from every live storage (an R4 fact); an allocator declares its
  context effects (may sleep, may fail: R14) and its bound behavior (R15).
  Writer-defined allocators (arenas, slabs, pools) obey the same contract.
- (b) The allocator's summary: distinctness of results, ended-storage
  writes, outcomes, effects.
- (c) At each alloc/free through the summary; once at the allocator's body.
- (d) Not its job: who may access a block (R1); layout of a block (R11).
- (e) P2: metadata written only at alloc and free, nobody holds `A`
  exclusively between operations, `b4` is a fresh identity in reused bytes;
  P3's pool reuse; a slab whose free pointer lives in the freed object.
  Measurement: `noalias`-return on `alloc` reaches the IR.
- Price: forbids the compiler from assuming ended storage is unwritten and
  from assuming any allocation property the summary does not state.

### R22 Fact-emission audit trail — new

- (a) For every accepted program the compiler can emit, beside the one
  binary and without changing it, the set of facts it handed the backend and
  the checks it removed under proof, in a machine-readable form a foreign
  reader (sanitizer-parity run, translation validator, debugger, profiler)
  can consume. There is one observable program behavior and no build mode;
  the trail is metadata, not a second build.
- (b) Which retained facts were consumed at which lowering points.
- (c) At lowering, as a side output.
- (d) Not its job: catching checker bugs itself; runtime instrumentation.
- (e) Measurement: for P5 the trail lists each per-load distinctness fact and
  each removed bounds check; a sanitizer run under a deliberately broken
  checker rule flags the removed check. Theorem shape: the trail is exactly
  the difference between the facts-off and facts-on lowering of the same
  source, which is how M9's erasure is checked.
- Price: forbids nothing at runtime; obliges the compiler to keep a record.

### M1 Reproducible, input-determined acceptance — restated (no-SMT clause reclassified)

- (a) The accept/reject verdict is a function of the source bytes and the
  specification version only. No solver state, timeout, machine speed, or
  work budget selects acceptance; every automatic family runs to its
  specified completion under its R13 bound; harder proofs are written
  finite steps the checker verifies without rediscovery. "No SMT" is the
  adopted mechanism (HIS-D0-02) serving this; a fixed, terminating,
  certificate-producing fixpoint would also satisfy it.
- (b) Nothing beyond the family definitions and the written steps.
- (c) Every acceptance decision.
- (d) Not its job: latency (R13); repairability (M7).
- (e) Theorem shape: verdict(source, spec) is a total function. Measurement:
  the same corpus on two machines and two load levels yields byte-identical
  verdicts and diagnostics.
- Price: forbids budget-selected acceptance; forbids nothing of the backend.

### M2 No unchosen runtime cost, no trap — restated

- (a) Accepted programs contain no compiler-inserted safety check, trap,
  guard or branch; every runtime test is writer-written control flow with a
  typed outcome (R14); the compiler may not insert a check to make a program
  accepted. A writer-written retry loop, seqlock read, generational handle
  comparison or version check is ordinary code and is not a violation; a
  design that *requires* such a check to reach safety is, because the cost is
  not chosen per site.
- (b) Nothing beyond the emitted instruction stream's accounting to source.
- (c) At lowering.
- (d) Not its job: deciding which writer-written checks are cheap.
- (e) P1: no runtime check at the `v[k]` read; the writer branches or proves.
  Measurement: emitted instructions are accounted for by source constructs;
  a diff of the IR against a facts-off lowering (R22) contains no added
  guards. Vale-style generational references and Swift dynamic exclusivity
  remain excluded, on this reading, because the check is not writer-chosen.
- Price: forbids the compiler from inserting any check; this is the
  performance floor's precondition, not a cost to it.

### M3 Closed, enumerated trusted base; non-vacuous discharge — restated

- (a) No writer-accessible escape: the trusted base (adapters, linked
  definitions, the allocator's foreign half, R20's backend check) is
  enumerated outside writer-reachable source, each entry carries the
  obligation it assumes, and a change to it is a change to the trusted base,
  not to a program. An obligation may not be discharged by weakening the
  specification, returning a default outside the domain, or assuming without
  check. "No `unsafe` keyword" is the mechanism serving this.
- (b) The enumerated base; for each discharge, that it proves the stated
  obligation rather than a weakened one.
- (c) At the base's boundary; at each discharge.
- (d) Not its job: semantic correctness of the base's own bodies.
- (e) A `byte_at` that returns `0` outside range is refused as a discharge;
  P9's `pick` cannot weaken its postcondition to make a caller check. The
  trusted base of the current compiler can be listed on one page.
- Price: forbids the compiler from trusting any source-level assertion; every
  R4 fact traces to a proof or to an enumerated base entry, which is what
  makes the facts non-defeasible (unlike Rust's `noalias` under `unsafe`).

### M4 Writer cost: regularity, budget, shared surface — restated

- (a) The writer is an AI: verbosity is cheap; irregularity, special cases,
  non-local rules and re-declared surface are expensive. Concretely: one
  spelling per construct, checked in the specification; the taught working
  set (specification plus cards) fits a stated token budget K, measured per
  amendment; the shared library surface is not re-declared per file. The
  closed pattern catalog is a mechanism serving this.
- (b) Token counts per amendment; a spelling-uniqueness check; the prelude's
  per-file bill.
- (c) Per specification amendment.
- (d) Not its job: repairability (M7); latency (R13).
- (e) Measurement: spec plus cards ≤ K tokens (today ≈130k against a claimed
  48k, so the row is currently failed and K must be chosen); the 110-line
  prelude (30–60% of each corpus file) is gone or shared; zero constructs
  with two spellings.
- Price: forbids nothing of the compiler; may forbid a fact channel whose
  vocabulary breaks the budget, which is the one place this row and R4/R11/R12
  collide and M6 adjudicates.

### M5 Measured performance floor; sequential fallback — restated

- (a) The program a checker accepts with no extra writer work runs, on the
  corpus and the discriminating programs, within a stated factor of the best
  C or Rust implementation of the same program on the same workload; the
  default-accepted shape is the fast shape. A failed parallel-permission
  proof leaves the program sequential rather than rejected (a chosen
  degradation policy, recorded as such).
- (b) Nothing in the checker; a reference implementation and a workload per
  corpus program.
- (c) Per release, by measurement.
- (d) Not its job: guaranteeing that proved permission is profitable
  (refused as HIS-D0-20); choosing schedules.
- (e) Measurement: P5, P6, wfgrep, zlib-core-kernels, compute-bench against
  reference implementations; the floor results (1.65x, 1.10x) are the shape
  of the evidence. Every overlap PAR-1/PAR-2 permit today remains permitted
  or the loss is stated.
- Price: forbids a design whose default-accepted program is off the fast
  shape; forbids nothing of the backend.

### M6 Backend freedom is priced — new

- (a) Every requirement states what it forbids the compiler from doing.
  A requirement may forbid only what a stated fact, ordering, layout, or
  marked property requires; where the source states nothing, the backend is
  free to analyze, specialize, inline, reorder, choose schedule, tile and
  worker count, and to use the closed world (PROG-1) it is given. A
  restriction on the compiler with no named ground is a defect of the row.
- (b) Nothing; this is a rule about rows.
- (c) At each amendment to this list.
- (d) Not its job: selecting any optimization.
- (e) Every row above has a Price line; a row without one fails this test.
  Measurement: for P5, the backend applies its own alias analysis on top of
  stated facts and loses nothing the source stated.
- Price: none; it is the pricing rule.

### M7 Repairable, machine-readable rejection — new

- (a) Every rejection cites one rule, one tree location, and where the fix is
  local, a mechanical repair; diagnostics are deterministic, byte-stable and
  parseable; no restriction leaves the writer without an accessible route; a
  rejection of a sound program names the restructuring it implies.
- (b) The rule and the location; the repair form.
- (c) At each rejection.
- (d) Not its job: acceptance; a bounded number of repair rounds.
- (e) P1's `v[k]` read: the diagnostic names the missing fact `k≠i ∧ k≠j`;
  P8: the diagnostic names the path whose state differs. Measurement: the
  corpus's rejections are repaired from the diagnostic alone by a writer
  without the example corpus (blind-writer OWN-6 fails this today).
- Price: forbids a rule whose only fix is non-local and unnamed.

### M8 Evolution and migration — new

- (a) A revision of the specification, of a summary vocabulary, or of a proof
  form ships with a mechanical migration; certificates and archived
  specifications survive it or their loss is stated.
- (b) The diff between vocabularies; the migration's totality.
- (c) Per amendment.
- (d) Not its job: preserving compatibility against the constitution's
  permission to break it.
- (e) The `&`/`&uniq` retirement in the map's open question 9 comes with a
  rewrite that re-accepts the corpus; a P9 signature vocabulary change
  migrates every corpus signature mechanically.
- Price: forbids nothing of the compiler.

### M9 Erasure is checkable — new

- (a) Proofs and checker facts contribute only facts to lowering; the
  emitted program is a function of the erased program plus the fact set
  (R22's trail); no proof term, certificate, or invariant reaches runtime.
- (b) Nothing beyond R22's trail.
- (c) At lowering.
- (d) Not its job: which facts are emitted (R4).
- (e) Theorem shape: lower(source) = lower(erase(source), facts(source)).
  Measurement: stripping every `invariant` and `use` from wfgrep changes the
  binary only by the facts the trail lists.
- Price: forbids the compiler from emitting proof-carrying runtime data.

Row count: R1–R22 (R6 as a merged stub) and M1–M9, 31 rows.

## 2. Premises

| Premise | Classification | Serves | If dropped |
|---|---|---|---|
| FN-1 signature-only checking | Mechanism (reclassified). The backend does not care whether a call-site summary was written or derived; it needs a summary. | R7 (bounded summary), R13 (edit locality), M7 (a rejection is local to a summary) | Infer-and-pin summaries become admissible under PROG-1; R7's guarantee (caller acceptance independent of callee body) must then be re-established by pinning; the emitted facts do not change. |
| PROG-1 closed world | Mechanism (reclassified), unspent by the checker, spent by the backend. Nothing in R1–R10 needs it; whole-program devirtualization, cross-declaration `noalias`, full LTO and R16 monomorphization consume it for free. | R4, R16, M5 through M6's backend freedom | Separate compilation appears: the summary of R7 becomes the only cross-unit fact channel and must carry everything the backend needs (LLVM's ThinLTO summary shape); libraries and incremental builds open; whole-program inference options close. |
| M1 deterministic, budget-free checking | Requirement (kept) with its "no SMT" clause reclassified as the adopted mechanism. | M7 (a flaky rejection is unrepairable), R13, the constitution's practical-iteration clause | If reproducibility itself were dropped, verdicts could vary by machine and the audit trail (R22) and the floor (M5) lose their evidence base. If only "no SMT" were dropped, a fixed-portfolio certificate-producing search would qualify; nothing downstream changes. |
| M2 no runtime safety check | Requirement (kept, restated as no unchosen cost, no trap). | Constitution Safety; M5 (the floor's precondition); R4 (facts are unconditional, not guarded) | Vale/Mezzo/Swift-exclusivity designs become admissible; every R4 fact becomes conditional on a guard the backend must keep; the floor is renegotiated per check. |
| M3 no unsafe | Requirement (restated as closed, enumerated trusted base and non-vacuous discharge) with "no `unsafe` keyword" as its mechanism. | R4's facts being non-defeasible; the constitution's floor; M9 | Every emitted fact becomes trust-dependent, as Rust's `noalias` is under `unsafe`; the performance argument of why-whitefoot collapses first, safety second. |
| M5 fast shapes only | Requirement (restated as a measured floor); "failed permission leaves the program sequential" is a chosen degradation policy and is named as such. | Constitution Performance | Nothing forbids a design whose default-accepted shape is `Rc<RefCell>`-like; the language stays safe and stops being a reason not to use Rust. |

## 3. Disposition table

### D0.md families (22)

| Family | Disposition | Reason |
|---|---|---|
| Keep the map's list verbatim | Refuse | §7's coupling table already shows the rows are not independent; freezing them also freezes the absence of layout, ordering, address space and third-party rows that every covered systems program needs. |
| Refactor the R1–R3 state ladder | Refuse the split and the folds; adopt one narrowing (R2's bound clause → R15) | Three state vocabularies give the backend nothing; folding R2 into R3 loses the obligation-versus-uniqueness distinction R14's outcome arms need; folding R3 into R1 loses the ABI-visible bytes-or-handle class. |
| Interference rows are one fact (R4, R5, R9) | Merge the *fact base* (R4(b) = R5(b), stated once), refuse merging the rows | The consumers read the relation in opposite exactness directions: a missing "distinct" costs R4 speed and costs R5 permission, so one row would make a missing fact a rejection and break M5's sequential fallback; R9's "occasional write, long-lived holder" keeps its row because its cost rule (pay only at the write) is a backend-visible requirement. |
| R6 relocated | Merge into R4 (soundness half) and R13 (precision half) | Kill is R4's write-footprint fact run backwards (SYS-D0-18); relocating into R7 would force a summary-shaped clause per intra-procedural write; leaving it checker-internal would hide the cost driver, so the precision half is priced under R13 and M7. |
| R4's standing and the optimizer's authority | Adopt RAD-D0-25 (backend may add facts by analysis); refuse RAD-D0-01 (delete R4) | CompCert is correct without alias metadata but slow on P5; deleting R4 concedes the floor; forbidding re-derivation gives up free wins, so R4(d) now says analysis may add, never subtract, and acceptance never depends on it. |
| R7 / FN-1 / PROG-1 reclassified | Adopt: R7 restated as bounded checked summary; FN-1 and PROG-1 reclassified as mechanisms | A backend treats the signature as a cache of whole-program analysis; both premises are load-bearing for different consumers (FN-1 for R13 locality, PROG-1 for R4/R16 whole-program facts), and the draft says so rather than picking one; HIS-D0-08/09 remain adopted decisions, now placed. |
| M1 restated as reproducible acceptance | Adopt; "no SMT" named as the adopted mechanism | A fixed, terminating fixpoint that produces a certificate satisfies reproducibility; the recorded ground (no solver state, timeout, machine speed, or budget) is kept verbatim as M1(a). |
| M2 restated as no trap and no unchosen cost | Adopt, with the literal "no compiler-inserted check" kept as the enforceable form | The performance reading needs the literal ban (an inserted check is unchosen cost by definition); writer-written retry loops and version comparisons are ordinary code; Vale and Mezzo remain excluded because their check is not chosen per site. |
| M3's boundary: escapes and non-vacuous discharge | Adopt RAD-D0-10 and AI-D0-08 into M3; HIS-D0-06 stays refused | The trusted base already exists (adapters, linked definitions) and must be enumerated and audited; discharge by weakening is an escape in all but name; a bare assume is refused because it makes R4 facts defeasible. |
| Failure paths and typed outcomes as an owned requirement | Adopt as R14 | Without a row, "typed outcome" is an unowned valve; kernels need `GFP_ATOMIC`-shaped no-allocate contexts and the R2 join rule must run over outcome arms. |
| Writer cost model: M4 restated and split | Adopt restated M4 with numbers; refuse the split into separate rows; HIS-D0-21 stays refused | Teachability, budget and prelude share one acceptance test (tokens per amendment); the 48k versus 130k gap makes the row currently failed, which is why it needs a number, not three rows; the catalog is a mechanism. |
| Repairable rejection and machine-readable diagnostics | Adopt as M7 (merging AI-D0-19's M6 into it) | Verifier-in-the-loop training needs this and it is true by accident today; a rule whose only fix is non-local is a rule the writer cannot use. |
| Checking cost and latency as a requirement | Adopt as R13 with a published per-family bound and edit locality | M1 says nothing about a two-hour accepting run; proof-use-cost's superlinear curve is a measured failure; per-statement proof options must pay a named price. |
| M5 and performance guarantees | Adopt AI-D0-18 (measured floor) as M5; SYS-D0-13's determinism menu folded into R5; RAD-D0-24 partially adopted as M6's pricing rule; refuse RAD-D0-12 (schedule language) as a mechanism; HIS-D0-20 stays refused | A floor is testable and a rule about one mechanism is not; a determinism menu is a requirement on what permission promises; a published cost model is a mechanism, but "each row prices the backend" is the requirement-level form of it. |
| Resource bounds, termination and time | Adopt as R15 | Constitution's bound clause is unconditional where promised; kernels and embedded targets need stack and no-block bounds; deadlock freedom is a lock-order fact R12 already needs. |
| Abstraction carries identity and effect facts | Adopt as R16 | Signature explosion under generics is where a first-order winner becomes unwritable; the backend also needs facts through indirect calls or loses them at every vtable. |
| Evolution and migration | Adopt as M8 | No row owns migration; a summary vocabulary will change and the corpus must re-accept mechanically. |
| Admission policy for missing requirements | Adopt "a named real-system program or constitutional clause grounds a row"; refuse frequency (HIS-D0-12/13 stay refused) and refuse "only what a current experiment blocks on" | THE-D0-07 is exactly the failure that selects a mechanism on a first-order fragment; the corpus was not written to represent real programs; every added row above names its program or clause. |
| Form of the requirement statement | Adopt the discriminating-program suite as each row's (e); refuse adequacy theorems as the sole form; refuse the lattice as a form; refuse AI-D0-21's deletion rule | A test per row is checkable now; theorems come with the soundness plan (open question 8) not before a mechanism; the lattice's one finding (R9 negates R4) is recorded in R9(d). |
| Hardware placement, ordering, address spaces and secret erasure | Adopt all four as R11, R12, R18, R20 | Every covered systems program is shaped by layout, ordering and address space; erasure's collision with proof-authorized DSE is resolved by pricing (marked stores are exempt from DSE), not by omission; the LLVM cross-space claim stays flagged unverified in R18. |
| Observability, debug builds and a checker-soundness oracle | Adopt as R22 (audit trail) under HIS-D0-16 (one behavior, no build modes); HIS-D0-17 stays superseded | A foreign reader gets the emitted fact set as metadata beside the one binary; that is sanitizer-parity without a facts-off build. |
| Storage shapes outside the state model | Adopt: R17 (write-once), R19 (loans and third-party relocation), R21 (allocator writes in ended storage) | Each names a real kernel or database shape (static keys, io_uring, SLUB) the current model misdescribes; R1's "Gone is inert" is dropped rather than defended. |

### REQ.md families (23)

| Family | Disposition | Reason |
|---|---|---|
| Checking cost as a requirement | Adopt as R13 | Budget-freedom and a bound are different properties; the measured 30→163→1689 ms curve fails any quadratic law. |
| Abstraction as a requirement | Adopt as R16 | Summary size under instantiation and instantiation termination are live decisions no row owns. |
| Resource bounds as a requirement | Adopt as R15 (taking R2's clause) | R2's "where promised" was weaker than the constitution and bounded memory is a whole accounting system. |
| Failure, partiality and typed outcomes | Adopt as R14 | Outcome arms multiply the paths R2 must discharge on; environment failure has constitutional standing. |
| M4 decomposition: teachability, spec size, auditability | Merge into M4 with a numeric K and per-amendment counts | The three sub-constraints share one measurement; META-5 already counts per amendment; auditability is served by M7's parseable diagnostics and R22's trail rather than a separate row. |
| Repair, edit stability, evolution | Split: repair → M7, edit stability → R13, evolution → M8 | Edit stability is a checker-cost property keyed on footprints; repair and migration are writer-facing and separately tested. |
| Standard library and prelude cost | Merge into M4 | The prelude is the largest measured token cost and belongs with the budget it breaks. |
| Layout and memory ordering | Adopt as R11 and R12 | A backend and a scheduler consume both directly; R5 promised race freedom with no ordering vocabulary to state it against. |
| Third-party mutation of storage | Adopt as R19 and R21; R1 restated | "Inert Gone storage" is false for every production allocator; relocation by a runtime or the OS needs a fixup channel or a handle rule. |
| R5 scope: reductions and determinism level | Merge into R5 | HPC reductions are the common parallel shape; a named determinism level lets a candidate promise less than source-order equality only where written. |
| Checker observability | Adopt as R22 | Full erasure and no trap leave no channel; a fact trail is one that costs nothing at runtime. |
| R4 and R5 are one relation | Merge the relation (R4(b) = R5(b)); refuse merging the rows | Same distinctness fact, opposite exactness direction; stating the relation once removes the silent retune while keeping M5's fallback. |
| R4 and R6 are precision policies | Refuse for R4; adopt for R6's precision half (→ R13) | R4 is a requirement because the floor depends on it and the constitution ranks runtime performance high; HIS-REQ-02's "what reaches LLVM is an implementation detail" is the position this draft reverses, with M6 as the ground. |
| R6 not independent of R4 and R7 | Adopt (R6 merged into R4) | The contract vocabulary chooses the kill rule; the draft keeps one footprint relation for both. |
| R7 / FN-1 versus PROG-1 | Adopt the reclassification; add HIS-REQ-06's declaration-side obligation to R7(a) | The requirement is a bounded, checked summary with local recheck; both premises are mechanisms with distinct consumers. |
| R9 dependence on R1, R4, R5 | Adopt: sequential clause deleted from R9, concurrent clause kept with a cost rule, R9(d) records the negation of R4's exclusivity | HIS-REQ-01/12 show OWN-5 does what R1(d) disclaims; the draft's R1(d) stays as written and the exclusivity trade is R4's, local to coinciding identities; global mutable state is one R9 instance. |
| R2, R3, R8 and R1 lifecycle dependence | Refuse the merges; adopt the shared join rule as a coupling note in R2(c)/R14 | Clean's one differing part (last reference versus obligation) is exactly what outcome arms and allocator contracts need kept separate; arenas and slabs are R21 instances, not a reason to merge R2 and R8. |
| R1 bundles three properties | Refuse the split; adopt the observation by moving layout *definition* to R11 while layout *currency* stays an R1 state | Initialization and end share one state vocabulary and one check point; layout as a backend fact has its own row. |
| M1 "no SMT" is a mechanism | Adopt | Reproducible acceptance is the requirement; a fixed-portfolio certificate search would satisfy it; the adopted no-SMT decision is named as the mechanism. |
| M2 "no runtime check" is a mechanism | Adopt the "no unchosen cost, no trap" reading; keep the literal ban as its enforceable form | The distinction the family asks for (safety trap versus writer conditional) is drawn in M2(a); CHERI and seqlock retry are writer code, generational handles are not. |
| M3 "no unsafe escape" hole | Adopt: trusted base enumerated, weakened discharge refused | D11's trusted adapter becomes an enumerated base entry with its obligation, not a violation. |
| M5 is a goal, not a checkable property | Adopt measured floor; RAD-REQ-19's shape-versus-schedule tension answered by M6 (schedule is backend freedom) | Measurement is the only validation; the emitter's missing alias promises are an R4 acceptance failure, not an M5 wording defect. |
| Erasure as a checkable requirement | Adopt as M9 | A premise with no theorem is unspent; R22's trail gives it a test. |

Families dispositioned: 45 of 45.

## 4. What this draft costs

1. **Nine backend-facing rows the writer-cost priority would resist.** R11,
   R12, R18, R19, R20, R21 and the ordering/space/layout clauses in R5, R7,
   R10 each add vocabulary to summaries and to the taught set; M4 is already
   failed at 130k tokens against 48k, and this draft makes it harder to pass.
   The draft accepts that because every covered kernel, database and HPC
   program is shaped by all of them and a row absent from the requirement
   list becomes a mechanism chosen blind.
2. **Optimizer authority over re-derivation (R4(d), M6).** Letting the
   backend add facts by its own analysis on top of stated ones weakens the
   purity of "the source is the only fact channel" that a theory priority
   wants for adequacy theorems and that an AI-writer priority wants so the
   writer's stated facts are the whole story. The draft accepts it because
   forbidding free wins prices a restriction no requirement grounds, and R22's
   trail keeps the added facts auditable.
3. **A determinism menu in R5 and target-dependent semantics in R12 and
   R18.** Admitting associative reductions with a weaker-than-source-order
   level, and pinning a memory model and address-space rules into the
   language, reopens CAP-1's single guarantee and imports target facts the
   spec must now state; a theory priority resists the loss of one semantics,
   and a writer priority resists the extra rule cards. The draft accepts it
   because HPC and kernel code cannot be written without them and a
   candidate that fixes source-order equality for everything refuses every
   reduction in the corpus.


# File: draft-C.md

# Draft C — the requirement list, priced for the AI writer

Stance: a requirement that no writer loop can test is a wish. Every row below
has an acceptance test that a checker, a measurement script, or a blind-writer
run over `discriminating-programs.md` can execute, and a price in tokens and
repair rounds. Hazard rows are kept where their repair messages differ, merged
where the writer states one fact for two consumers, and the meta-constraints
are rebuilt as cost constraints with the chosen mechanisms named as such.
Nothing here selects a mechanism; M1/M2/M3/M5 and FN-1/PROG-1 are named as the
mechanisms currently recorded as adopted and are left where they are.

Price columns are estimates to be replaced by the measurements the rows
themselves demand. "Spec" is tokens the rule costs in the taught spec plus
cards; "program" is tokens per use in source; "rounds" is expected
rejection-to-green iterations per violation, with "local" meaning the fix
lands at the cited tree location.

## 1. Revised list

Column key: (a) what must hold, (b) least a checker must know, (c) where
checked, (d) not its job, (e) acceptance test, Price.

### Hazard rows

| # | Name | (a) what must hold | (b) least a checker must know | (c) where checked | (d) not its job | (e) acceptance test | Price (spec / program / rounds) |
|---|---|---|---|---|---|---|---|
| R1 | Sequential memory safety (kept, whole) | No read of storage holding no value; no access to storage whose identity has ended (freed, reallocated, released, relocated); no access under a stale layout. An ended identity is terminal for that identity; the bytes may belong to another live identity (an arena, a pool). | For the storage an access names: live, initialized in the selected part, layout current. Properties of storage, not of the pointer. | At each access, against the named storage's state at that point. | Two pointers to one storage; sequential aliased writes; parallel overlap; who released it. | P4 (`c.read()` after `push` accepted iff validity provable; after `free` refused), P7 (hole read refused, free permanent), P8 (rejection names the differing path), P2 (`b2` pointers refused after reuse by `b4`). Theorem shape: no accepted program evaluates a read at `Uninit` or `Gone`. Diagnostic must name which of the three sub-properties failed. | ~2k / 0 extra beyond state-changing ops / 1, local |
| R2 | Resource lifecycle accounting (kept; bounds clause moved to R13) | Every allocation and external resource is released exactly once; linear obligations discharged on every path; release decisions static. | Which binding holds the release obligation of which storage; whether it is definitely discharged on every path; the join rule (stated once here, reused by R3). | Consuming ops, every scope exit, every join. | Who may read or write; the memory bound (R13). | P2 (each block freed exactly once; `A` never held exclusively between ops), P8 (no runtime drop flag: count of release-selecting runtime bits in lowered IR = 0; rejection names the path). | ~1.5k / 1 release op per resource / 1, local |
| R3 | Value classes and transfer (kept) | Copy duplicates freely; affine moves at most once with derived release; linear consumed exactly once. Move transfers with obligations; take leaves a hole; put fills; replace exchanges. | Each value's class; each transfer's source and destination storage. | At the transfer. | Pointer validity: a value leaving `'a` is an event on `'a`'s state that R1 consumes. | P7 (take/put/replace through aliases: hole visible to `q`, fill by `q` restores `p`). Diagnostic distinguishes "used twice" from "never released" (R2). | ~1k / 1 keyword per transfer / 1, local |
| R4 | Aliasing facts for the optimizer — **merged into R5** | — | — | — | — | Carried as R5's second consumer and its P5 measurement. | — |
| R5 | Interference facts: race freedom, parallel permission, optimizer distinctness (**merged R4 into R5**) | Two statements or iterations overlap only when footprints do not interfere; a data race is unrepresentable. The same footprint and distinctness facts are handed to the backend without guards. A missing fact costs speed for both consumers, never correctness or acceptance (with M5). Each permission construct states the determinism level it promises. When threads arrive the row names what the checker knows before and after acquire. | Read/write footprints per statement over storages; distinctness by identity, field, proved index, proved range; data dependencies; per construct, its determinism promise. | Permission windows and loops today; spawn/join later; at lowering for the backend, from retained facts. | Sequential safety (R1); forbidding aliasing to gain facts (that trade is local to storages the writer let coincide, R9). | P1 (two element writes may run in parallel; `v[k]` read diagnostic names the missing fact), P5 (eight columns distinct; measurement: per-load alias fact emitted for every column load, zero guards), P6 (overlap permitted from range arithmetic), P10 (what is known across acquire; determinism stated). Corpus: every PAR-1/PAR-2 overlap accepted today stays accepted or the loss is listed. | ~3k / footprint clause per signature, index proof where two footprints share a container / 1–2, local (a missing distinctness fact is one cited proof step) |
| R6 | Frame: predictable fact retention (kept; **reclassified** as a writer-cost floor, not a hazard) | After a write or call exactly the facts whose support may have changed are dropped; every other fact survives; the writer can compute which from the footprint it wrote. | Written footprint of each operation, keyed as facts are keyed. | At commits and call boundaries. | Safety; permission. It decides how many facts the writer must re-prove. | P3 (invariant "next/prev point to live nodes" stated once and reused across four link writes and a removal). Measurement: re-proofs of facts whose support was untouched = 0 over P1–P9. Diagnostic on a killed fact names the killing operation. | ~1k / 0 / 0 when precise; each imprecision is one extra round |
| R7 | Contract locality and completeness (**restated**; FN-1 named as the current mechanism) | What a caller needs of a callee is available at the call site in a bounded, writer-readable form: what the callee needs of each storage on entry and leaves on exit (hole, ended, replaced backing, returned interior pointer, fresh allocation, footprint). A declaration whose result no caller can use is itself an error. | Storage identities nameable in contracts; entry/exit states per identity; footprints per identity. | At the call for the caller; once at the returns for the callee. | Prescribing signature-only versus summary-derived contracts (that is the mechanism); inferring from bodies is a cost question under M7/M8. | P9 (`drain`, `reserve`, `pick` each visible in the signature; caller judged from it; body checked once). Measurement: bytes the caller check reads outside the callee's contract = 0 under the current mechanism. | ~2.5k / 1 clause per state-changing effect per signature / 1, local at the signature |
| R8 | Storage placement and relocation (kept; identity-end clarified) | A value lives somewhere (frame, heap, arena, inline). Some operations relocate or replace storage; interior pointers name the storage before relocation; identity end is an event on the identity, not the bytes. | Which operations end or replace which storage; whether aggregates travel as bytes or handles. | At the relocating operation, as an R1 state event and an R7 contract clause. | Who may access; layout width and alignment (not owned at this decision point). | P4 (`push` may reallocate: second read legal iff "still valid" carried), P3 (pool slot reuse invalidates every path to `m`), P2 (arena bytes reused by `b4`). | ~1.5k / 1 contract clause per relocating op / 1, local |
| R9 | Several holders is a writable shape (**restated** as a shape-admission row) | Several long-lived pointers to one storage, each writing occasionally, is a writable shape. Sequentially it costs nothing beyond R1 and R6; across threads the cost lands on the write, never on the hold. | Sequentially: nothing beyond R1/R6. Concurrently: which primitive holds the storage's state facts while no thread holds them. | Sequentially each access; concurrently acquire and release. | Forbidding aliasing to strengthen R5's optimizer facts; global bans. | P3 (back edges through several paths accepted sequentially), P7 (two aliases to one slot), P10 (sync cost only at the write). Negative test: a design that rejects P3's sequential link updates fails R9. | ~0.5k / 0 sequentially; 1 acquire per write concurrently / 1 |
| R10 | External resources as ordinary objects (kept) | Files, sockets, mappings, devices obey the same state, effect and proof rules as memory; what an external actor may change between operations is a stated contract clause. | One identity per resource; its states; which ops transition them; what may change between ops. | Each operation, through ordinary contracts. | Host mechanisms; scheduling. | Descriptor program: open, short read, close-once. Measurement: rules in the spec that mention the host boundary = 0. | 0 extra spec / contract clauses as for memory / 1 |
| R11 | Abstraction survival (**added**) | R1–R10 hold under generics, nominals, closures and dynamic dispatch; identity and effect facts pass through abstraction; instantiation terminates by a stated syntactic rule. | Identity and effect parameters on nominals and callables; a bound vocabulary; the instantiation termination rule. | At instantiation and at every abstract call. | Choosing monomorphization versus dictionaries. | P3 and P5 written generic over the element type accepted with the same parallel permissions. Measurement: signature token count grows linearly in the number of identity/effect parameters, with a stated constant. | ~2k / 1 parameter per abstracted identity or effect / 1–2 |
| R12 | Typed outcomes and environment failure (**added**) | Every condition that can be false at runtime (allocation failure, short read, interruption, index outside a proved range) is a typed outcome with real control flow; R2/R3 obligations discharge on every outcome arm. | The outcome type of each partial or external operation; the obligation state on each arm. | At the operation and at each arm's join. | Deciding the propagation syntax; where allocation may not occur (a future effect axis under R13). | P2 with `alloc` returning failure: obligations on `b1`, `b3` discharged on the failure arm; P8 loop exits. Measurement: partial operations without a proof or a typed outcome in accepted programs = 0. | ~1.5k / 1 arm per fallible op / 1, local |
| R13 | Resource bounds where promised (**added**, opt-in as the constitution states) | Where a program declares a memory, stack or time budget or a termination promise, the bound is machine-verifiable; an unpromised program pays nothing. | A cost semantics for the promised resource; the budget declaration; the deterministic bound derivation. | At the declaration's scope end. | Making every program bounded; deadlock freedom before threads exist. | P2 with a declared byte budget for `A`; wfgrep with a stack-depth promise. Measurement: token cost of the promise per program and rounds to green. | ~2k when used, 0 otherwise / budget clause + proof steps / 2–3 |

### Meta-constraints

| # | Name | (a) what must hold | (b) least a checker must know | (c) where checked | (d) not its job | (e) acceptance test | Price |
|---|---|---|---|---|---|---|---|
| M1 | Reproducible, explainable acceptance (**restated**; "fixed families, no solver" is the adopted mechanism, HIS-D0-02) | The same source yields the same verdict and the same diagnostic bytes on any machine at any speed; every verdict is derivable from spec plus source by a reader; no state outside the source selects acceptance. | Nothing beyond the spec's stated derivation. | Every check. | Bounding wall-clock time (M7). | Run the checker on the corpus on two machines and under a 10x CPU throttle: identical verdict and diagnostic bytes. Spec derivability: each AUTO family's completion is stated in the spec. | ~0 extra / 0 / 0 |
| M2 | No unwritten failure edge, no unchosen runtime cost (**restated**; "no runtime check" is the adopted mechanism, HIS-D0-03) | An accepted program contains no trap and no branch the writer did not write in source; every runtime condition is a typed outcome (R12) with source control flow. | Nothing; a property of lowering. | Lowering. | Deciding which cheap checks would be acceptable (none are, under the adopted mechanism). | P1: the only branch at the `v[k]` read is the writer's. Measurement: runtime branches in lowered IR without a source branch = 0 across P1–P9. | 0 / branches the writer writes / 1 |
| M3 | Closed, enumerated trusted base (**restated**; "no unsafe keyword" is the adopted mechanism) | The set of definitions trusted without proof is finite, listed in one place, and unreachable from writer-authored code; no writer-authored construct can assert a fact without a checked discharge. | The trusted list. | At every discharge. | Preventing a vacuous but sound discharge (a logic error the constitution allows to remain; the owner's tests own it). | Measurement: writer-reachable assume-like constructs in the spec = 0; trusted-base entries listed and counted per amendment. | ~0.3k / 0 / 0 |
| M4 | Teachability and regularity (**split** from the old M4; budget went to M10, repair to M6) | One spelling per construct to the byte; each rule teachable as one card of at most K tokens; no rule has a special case another rule must mention. | Nothing; a spec property. | Spec check in `make check`. | The total budget (M10); diagnostics (M6). | Spelling lint over the grammar: alternative spellings = 0. Card size: max card ≤ K (K set by measurement of blind-writer success per card size). | Bounded by K per rule / 0 / 0 |
| M5 | Permission-independent acceptance and a measured floor (**restated**; "accepted shapes are the fast shapes" is the goal) | Removing every permission annotation changes no verdict; a failed permission leaves the program sequential. The obvious shape's cost is a published measured floor on locked workloads, not a slogan. | Nothing; a theorem shape plus a measurement. | Theorem: acceptance is invariant under permission erasure. Measurement: floor ratio per corpus. | A schedule language; a published backend cost model (mechanisms, refused here). | Theorem shape on the checker; measurement: first-green artifact on P5/P6-shaped locked workloads within a stated ratio of the reference, restated per compiler revision. | 0 / 0 / 0 |
| M6 | Repairable rejection (**added**) | Every rejection is fixable from the diagnostic alone: one rule cited, one tree location, and where a mechanical fix exists, its form; diagnostics are deterministic, byte-stable and parseable; no restriction leaves the writer without a taught route. | The rule, the location, the missing fact. | Every diagnostic. | Making every program accepted; repairing logic errors. | Blind-writer loop over P1–P10 with no corpus access: rounds per rejection ≤ N (N set by the first measurement), fix lands at the cited location; diagnostics byte-equal across runs and parse under one grammar. | ~0.5k / 0 / this row sets the bound on every other row's rounds |
| M7 | Checking latency and its scaling law (**added**) | Compile time is a stated function of program size and writer proof steps: at most quadratic in proof steps per unit, near-linear in units, with the constants published and re-measured per revision. Termination alone is insufficient. | The complexity of each automatic family, stated in the spec. | `make check` scaling series. | Selecting acceptance (M1). | Scaling series on the proof-use-cost shape (N = 16, 32, 64, 128): fitted exponent ≤ 2 in proof steps; whole-corpus wall time under a stated ceiling. | 0 / 0 / 0; violation is a compiler defect |
| M8 | Edit stability (**added**) | A one-token edit rechecks a bounded dependent set and reopens only proofs whose support the edit touched; certificates for untouched units survive. | Dependency granularity of the check. | Incremental check. | Deciding the unit (function, module, program). | Measurement: for each P1–P9, mutate one token, count units rechecked and proofs reopened; ratio to touched support ≤ stated constant. | 0 / 0 / 0 |
| M9 | Evolution and migration (**added**) | A spec, signature or proof-form revision ships with a mechanical migration; certificates and archives survive or the count of manual edits is stated; META-5 token and spelling deltas are counted per amendment. | The delta. | Per amendment, on the corpus. | Preserving backward compatibility for its own sake (the constitution lets it yield). | Migrate the conformance corpus vN→vN+1 with the shipped tool: manual edits = 0 or the count is in the PR; META-5 counts present. | ~0 / 0 / migration rounds counted per revision |
| M10 | Spec budget and retrieval (**added**; absorbs the prelude cost) | Spec plus cards fit a stated budget B tokens, measured in `make check`; every rule is addressable by ID so a writer can retrieve one card; the fixed per-file cost (prelude, re-declared library) is bounded at fraction f of a file and measured. | Nothing; counts. | `make check`. | Choosing how the prelude is delivered (injection, import: mechanism). | Token count ≤ B (today 130k against a 48k claim: the row starts red); rule-ID index exists; prelude fraction over the corpus ≤ f (today 30–60%: red). | This row prices every other row |
| M11 | Erasure and checker-defect channel (**added**) | Proofs and permissions are erased before lowering; the executable is a function of the erased program plus retained facts; the compiler's own tests can withhold retained facts and compare behavior. One observable behavior; no writer-visible build mode. | Which facts are retained. | Theorem shape on lowering; differential run in the compiler's test suite only. | Debuggers, profilers, foreign readers (deferred). | Theorem shape: lowering commutes with proof erasure. Measurement: facts-withheld differential run over the corpus agrees; any disagreement is a checker defect with a channel to surface. | 0 / 0 / 0 |

Rows: 23 live (12 R, 11 M) plus the R4 tombstone.

Split/merge/reclassify marks: R4→R5 merged; R6 reclassified (cost floor); R7 restated (FN-1 named as mechanism); R9 restated (shape admission); R2's bounds clause moved to R13; old M4 split into M4/M6/M10; M1/M2/M3/M5 restated with their mechanisms named.

## 2. Premises

| Premise | Classification | Requirement it serves | What changes if dropped |
|---|---|---|---|
| FN-1 signature-only checking | Mechanism serving R7 (contract locality), M6 (local diagnostics), M8 (edit stability), M7 (per-unit cost). Recorded ADOPTED (HIS-D0-08); not re-selected here. | R7, M6, M8, M7 | Whole-program inference becomes legal under PROG-1; R7 is then satisfied by "infer, print, pin" summaries; M6 must then bound non-local rejections (a body edit rejecting a distant caller) and M8 must state the recheck set for inference. The requirements survive; the price columns of R7/M6/M8 are re-measured. |
| PROG-1 closed world | Unspent premise at the requirement level. No row above needs it; R13 (bounds) and M11 (erasure) are easier with it, not dependent on it. Recorded ADOPTED (HIS-D0-09); its grounds must be restated against M7/M8. | None directly; eases R13, M11 | Separate compilation and libraries become admissible; a declaration-only interface becomes a fact-loss surface R7 must price (HIS-REQ-06); M7's scaling law must be stated per unit rather than per program. |
| M1 deterministic budget-free checking | The requirement is reproducible, explainable acceptance (M1 restated). "Fixed terminating families plus explicit certificates, no solver" is the adopted mechanism (HIS-D0-02). | M1, M6 (byte-stable diagnostics), M7 | A fixed-iteration Datalog or Houdini fixpoint with certificates would satisfy M1 restated; M7 must then bound the fixpoint's cost; explainability needs the fixpoint's derivation printed. |
| M2 no runtime safety check | The requirement is no unwritten failure edge and no unchosen cost (M2 restated). "No runtime check or trap" is the adopted mechanism (HIS-D0-03). | M2, M5 floor, R12 | Generational or epoch checks become admissible if every check is a source-visible typed outcome; the floor measurement (M5) must then include the check cost; the disqualification of Vale/Mezzo in MAP §5 becomes a cost argument rather than a rule. |
| M3 no unsafe | The requirement is a closed, enumerated trusted base (M3 restated). "No unsafe keyword" is the adopted mechanism. | M3, M6 (a stuck writer has no escape to reach for) | An escape with a machine-checked boundary obligation moves the trusted base into source; M3's list grows per use and someone audits it; M6's "no route left" clause becomes "the route is the escape", which the writer loop will take. |
| M5 fast shapes only | A goal. The checkable residue is M5 restated: permission-independent acceptance plus a measured floor. "Accepted shapes are the fast shapes" is neither a requirement nor a mechanism; it is what the floor measurement tests. | M5 | Nothing at the requirement level; a design free to reject a slow shape must then pay M6 (a route exists) and M4 (one more rule). |

## 3. Disposition table

Family names verbatim. Disposition: adopt / merge into <row> / reclassify as mechanism / refuse.

### D0.md families

| Family | Disposition | Reason |
|---|---|---|
| Keep the map's list verbatim | Refuse | The list's own §7 couplings contradict the independence it claims, and verbatim rows have no acceptance test or price; every row is kept only after gaining (e) and a cost. |
| Refactor the R1–R3 state ladder | Refuse (R1 kept whole; R2, R3 kept separate; join rule stated once in R2) | Three state vocabularies cost three cards where one serves; the R2/R3 merge loses the "never released" vs "used twice" repair message, which is the writer's one signal; bounded memory gets its own home in R13. |
| Interference rows are one fact (R4, R5, R9) | Merge R4 into R5; R9 kept as a shape row | The writer states one footprint for two consumers, halving the taught vocabulary; the "missing fact rejects" objection is void because M5 restated makes acceptance permission-independent, so a missing fact costs speed for both consumers; R9 survives because P3/P7/P10 must stay writable. |
| R6 relocated | Adopt as reclassification (R6 kept, reclassified as a writer-cost floor) | Hiding it in R4/R5 or the checker hides a measured repair-round driver; folding into R7 forces a signature-shaped clause per local write, which costs tokens; P3's "stated once and reused" is an executable test only if R6 has its own row. |
| R4's standing and the optimizer's authority | Merge into R5 (deletion refused); re-derivation authority reclassified as a compiler decision | Deleting R4 concedes the constitution's performance ranking and the P5 measurement; whether the backend may re-derive is a `design/compiler` decision that no writer-facing requirement constrains beyond M11 (retained facts are sound). |
| R7 / FN-1 / PROG-1 reclassified | Adopt (R7 restated as contract locality; FN-1 mechanism; PROG-1 unspent) | Recheck cost per edit is the real requirement (M8) and it is testable; FN-1 stays adopted because its price columns (local diagnostics) are the best measured, but as a mechanism whose grounds are M6/M8, not a row of its own. |
| M1 restated as reproducible acceptance | Adopt | Reproducibility and byte-stable diagnostics are what the writer loop consumes; "no solver" is the adopted mechanism and stays adopted; the restatement makes M1 testable by a two-machine run. |
| M2 restated as no trap and no unchosen cost | Adopt | Stating it removes the appearance that all branching is banned, which P1's `v[k]` branch needs; the cost rule for "cheap enough checks" is refused: under the adopted mechanism none are, and the floor measurement (M5) is where a rival would have to argue. |
| M3's boundary: escapes and non-vacuous discharge | Adopt the boundary (M3 restated); refuse non-vacuity as a requirement | A closed listed trusted base is countable; non-vacuity is semantic, the constitution leaves logic errors to remain, and a spec rule against weakening cannot be checked in principle without an oracle the owner's tests already are. |
| Failure paths and typed outcomes as an owned requirement | Adopt as R12 | Typed outcome is M2's pressure valve and had no discipline; the join interaction with R2 is exactly why the row is priced at one arm per fallible operation; "may not allocate here" is deferred to R13's opt-in effect axis. |
| Writer cost model: M4 restated and split | Adopt as M4 / M6 / M10 | Each sub-requirement gets its own test (spelling lint, card size, token count, prelude fraction); K, B and f are set by measurement, and the row starts red on today's 130k spec, which is the point. |
| Repairable rejection and machine-readable diagnostics | Adopt as M6 | It is the precondition for the verifier-in-the-loop; the infinite-loop risk is handled by bounding rounds N and measuring, and "no route" (HIS-D0-15) becomes a clause of the row. |
| Checking cost and latency as a requirement | Adopt as M7 and M8 | M1 says nothing about a two-hour green run; a stated exponent and a recheck ratio are measurable in `make check` and price the per-statement options directly. |
| M5 and performance guarantees | Adopt the measured floor and permission independence (M5 restated); reclassify schedule language and cost-model exposure as mechanisms; refuse a determinism menu here but require R5 to state a level | A floor is testable, a slogan is not; a schedule language is two artifacts to teach; a determinism level per construct (P10) is a requirement clause, whether a menu exists is a mechanism. |
| Resource bounds, termination and time | Adopt as R13, opt-in | The constitution text is conditional ("for uses with explicit resource budgets"), so the opt-in form matches it and costs unpromised programs nothing; termination and deadlines join the same opt-in row; deadlock freedom waits for threads. |
| Abstraction carries identity and effect facts | Adopt as R11 | Selecting on the first-order fragment is the known failure; signature explosion is the cost, so the row's test is linear signature growth, which a candidate must show before selection. |
| Evolution and migration | Adopt as M9 | The constitution names evolution and no row owned migration; a corpus migration with a manual-edit count is a runnable test each amendment already half performs via META-5. |
| Admission policy for missing requirements | Adopt THE-D0-07 with two named exceptions (writer-cost rows and R11 admitted on measured failure); refuse frequency as a veto | Lazy admission keeps the spec inside M10, but abstraction and the writer-cost rows are admitted now because their absence already changed decisions (HIS-REQ-09, blind-writer §6.2–6.3). |
| Form of the requirement statement | Adopt the discriminating-program-plus-measurement form; refuse adequacy theorems as the primary form; refuse the lattice as a form | Programs and measurements are what a writer loop runs; theorem shapes are used only where no program discriminates (M5 independence, M11 erasure); the lattice's R9-negates-R4 reading is recorded in R9(d) without adopting the form. |
| Hardware placement, ordering, address spaces and secret erasure | Refuse at this decision point (ordering vocabulary noted as an R5 obligation when threads arrive) | No P1–P10 program and no current experiment fails without them; secret erasure collides with M11 and needs its own investigation; adding four rows now spends M10 budget on unmeasured needs. |
| Observability, debug builds and a checker-soundness oracle | Merge the oracle into M11; refuse foreign-reader observability | A compiler-test differential run is not a build mode, so HIS-D0-16 holds; debugger and profiler support is deferred under the admission policy because the AI writer's loop is the compile check. |
| Storage shapes outside the state model | Merge allocator-writes-in-Gone into R1/R8 (identity end is not byte end, which P2 requires); refuse caches, escrow loans and third-party relocation as rows | P2 forces the identity/bytes clarification; the other three shapes are added to the discriminating suite when a corpus program needs them, per the admission policy. |

### REQ.md families

| Family | Disposition | Reason |
|---|---|---|
| Checking cost as a requirement | Adopt as M7 | The measured 30→163→1689 ms series is the acceptance test's shape; a stated exponent distinguishes budget-freedom from usability. |
| Abstraction as a requirement | Adopt as R11 | Same as above; the instantiation-termination rule is a stated syntactic rule the writer can be taught. |
| Resource bounds as a requirement | Adopt as R13, opt-in; refuse the unconditional reading | The constitution's own sentence is conditional, so R2's clause was consistent with it; the accounting system (envelope, lanes) is mechanism and is not selected. |
| Failure, partiality and typed outcomes | Adopt as R12 | Outcome arms multiply the paths on which R2 discharges, so the row is priced at one arm per fallible operation and tested on P2 and P8. |
| M4 decomposition: teachability, spec size, auditability | Adopt as M4 / M10; refuse a separate human-auditability row | The measured 130k-vs-48k gap is the test; owner auditability of diffs is served by M9's META-5 counts and M4's one spelling rather than a row nobody can measure. |
| Repair, edit stability, evolution | Adopt as M6 / M8 / M9 | Three rows because three different measurements: rounds per rejection, units rechecked per token, manual edits per migration. |
| Standard library and prelude cost | Merge into M10 | The prelude is the largest measured per-file cost, so it is priced under the same budget row as the spec; delivery is a mechanism. |
| Layout and memory ordering | Refuse (R8 notes width and alignment as not owned; R5 notes ordering when threads arrive) | No discriminating program needs them yet; adding them now spends budget on requirements with no test. |
| Third-party mutation of storage | Merge allocator writes into R1/R8; refuse relocation by a third party | P2 requires the arena's metadata writes and byte reuse; OS or collector relocation has no program in the suite. |
| R5 scope: reductions and determinism level | Adopt as an R5 clause (determinism level stated per construct) | P10 asks whether determinism is promised; fixing source-order equality for every construct is a mechanism choice the row must not make. |
| Checker observability | Merge into M11 | A differential run inside the compiler's tests is the channel; it is not a source-visible mode. |
| R4 and R5 are one relation | Adopt (R4 merged into R5) | One footprint vocabulary; the "mechanism change for R4 retunes R5" concern is now a feature, since the writer states the fact once. |
| R4 and R6 are precision policies | Adopt for R4's imprecision clause (now R5(a)); reclassify R6 as a cost floor rather than deleting it | HIS-REQ-02's "what reaches LLVM is an implementation detail" holds; what the writer must re-prove after a kill is not a detail, it is rounds. |
| R6 not independent of R4 and R7 | Refuse the fold | Coupling is recorded in R6(b) (keyed as facts are keyed); a fold would force signature-shaped clauses on local writes. |
| R7 / FN-1 versus PROG-1 | Adopt (R7 restated; FN-1 mechanism; PROG-1 unspent; declaration-side obligation added to R7) | Bounded recheck (M8) is the requirement; FN-1 stays adopted on its measured local-diagnostic price, not as a row. |
| R9 dependence on R1, R4, R5 | Adopt the restatement (R9 as shape admission) | HIS-REQ-01/12 show today's OWN-5 rejects the shape, which is why R9 must be a requirement on the option space; R1(d) describes the requirement, OWN-5 the current mechanism. |
| R2, R3, R8 and R1 lifecycle dependence | Refuse the merges; join rule stated once in R2 | Clean's one differing part (obligation vs last reference) is the difference between two repair messages. |
| R1 bundles three properties | Refuse the split | One vocabulary, one card; the acceptance test enumerates the three sub-properties and the diagnostic must name which failed. |
| M1 "no SMT" is a mechanism | Adopt (M1 restated; no-SMT named as adopted mechanism) | A certificate-bearing fixed portfolio would satisfy M1 restated and M7 would then price it; the mechanism is not re-selected here. |
| M2 "no runtime check" is a mechanism | Adopt (M2 restated; no-check named as adopted mechanism) | Distinguishes a safety trap from writer-written control flow, which P1 needs; the literal reading's disqualifications become floor arguments under M5. |
| M3 "no unsafe escape" hole | Adopt the trusted-base clause; refuse a weakened-contract gate | The trusted base is listable and countable; a gate against weakened `requires` is semantic and belongs to the owner's tests. |
| M5 is a goal, not a checkable property | Adopt (M5 restated with a theorem shape and a measured floor) | The unassigned D13 measurement obligation becomes the row's test; what the writer may demand of the backend beyond the floor is refused as a cost-model mechanism. |
| Erasure as a checkable requirement | Adopt as M11 | An erasure theorem shape plus a differential run makes the premise testable and gives checker defects their only channel. |

Families: 22 (D0) + 23 (REQ) = 45, all dispositioned.

## 4. What this draft costs

1. **R4 disappears as a row.** Merging optimizer distinctness into R5 spends the "a missing fact costs speed, never correctness" wording as a clause rather than a row; a performance-first drafter would keep the optimizer's consumer separately visible so its emitted-fact count is never lost behind the scheduler's judgement. The P5 measurement is kept, but under R5's name.
2. **Three needed-by-kernels families are refused now**: layout and ordering, address spaces and secret erasure, and third-party relocation and escrow loans. A systems-first drafter would admit them today; this draft admits them only when a discriminating program fails without them, because every unmeasured row spends M10's budget and adds a card.
3. **M6, M7, M8, M10 start red against the current compiler and spec** (130k tokens, 30–60% prelude, no measured repair-round bound, a superlinear proof-cost series). A theory-first drafter would refuse to put unmet measurements in the requirement list; this draft insists on them because a requirement without a runnable test is the thing the writer loop cannot use.


# File: critique-A-soundness.md

# Critique of draft A — lens: SOUNDNESS AND COMPLETENESS OF THE LIST

Reviewed: `/private/tmp/whitefoot-mechanism-research/d0/draft-A.md` against
MECHANISM-MAP §2/§3/§7/§8, the constitution, why-whitefoot 1–140,
merged/D0.md, merged/REQ.md and discriminating-programs.md.

Severity key: **fatal** = the list would select a wrong or unsound design;
**serious** = a requirement is wrong, missing, non-independent or
unenforceable, or a disposition reason does not engage the family's
recorded tensions; **minor** = wording.

## 1. Findings

| # | Sev | Row / family | Finding | One-sentence fix |
|---|---|---|---|---|
| F1 | fatal | M2; R2 (c); D0 "M2 restated"; REQ "M2 no runtime check" | The restated M2 ("no unstated failure edge") drops the family's other half, "no cost the writer did not choose" (AI-D0-16), sending it to M9, a corpus-level floor that cannot refuse a single program. A drop flag, a generational compare that never traps on a correct program, or a Vale-style check wrapped as a typed outcome at the interface is then not a failure edge and is not refused by M2's text. The row rescues itself only by the clause "the checker never inserts a check", which is the literal "no runtime check" mechanism reading the same row claims to have reclassified; and R2 (c) then justifies "no drop flag" by calling a drop flag "an unstated failure edge under M2", which is false (a drop flag never fails). The acceptance test "P8 no drop flag" therefore does not follow from the row's own (a). A candidate with hidden runtime checks could satisfy the list as written. | State M2 as two checkable clauses — (i) no executable failure edge absent from source and interface, (ii) no executable branch, load or store in the accepted program that the source did not write — and derive the drop-flag and generational refusals from (ii), deleting the "unstated failure edge" justification in R2 (c). |
| F2 | fatal | R1, R13, M2, R12; constitution "memory corruption", "environment failures have defined behavior" | Stack exhaustion has no owner. M2 forbids the guard-page trap (a failure edge the source did not write), R13 makes stack bounds demand-conditional ("where a use demands it"), R12 lists heap allocation failure but not stack, and R1 forbids corruption. An accepted recursive program with no demanded bound therefore has no defined behavior at stack overflow: the list admits undefined behavior on a path the constitution names as a hazard, and a candidate can satisfy every row while leaving it undefined. | Give stack (and any other implicitly consumed resource: file-descriptor table, address space) an unconditional owner: either R13 makes stack depth a mandatory bound for every accepted program, or R12 names stack exhaustion as an expected environment failure with a typed outcome, and §0 lists it. |
| F3 | fatal | M5, R5, R14, M6 (iii), "Observability" family (HIS-D0-16), "M5 and performance" family (SYS-D0-13) | Three rows contradict each other on what the reference behavior is. M5 says "the sequential semantics is the reference" and "a failed permission leaves the program sequential"; the draft also keeps HIS-D0-16 "one observable behavior"; but R5 and R14 admit a "named weaker level" per overlap construct, under which the sequential result is one admitted outcome, not the reference, and observable behavior is a set. M6 (iii) "observable behavior equals the semantics" is then stated over an undefined object. Further, M5's "leaves the program sequential" is false for the spawn/join construct R5 itself lists: a racy spawn is rejected, not sequentialized. A judge applying M5 refuses every candidate that honors R14's level; a judge applying R14 loses the theorem M5 exists for. | Restate M5 as: acceptance never mentions the permission judgement, and for implicit overlap constructs (PAR-1/PAR-2) a refused permission yields the sequential program; state separately in R14 that the reference semantics of a construct at a weaker level is the set its level admits, and retire or restate HIS-D0-16 to "one specified behavior set", so M6 (iii) has an object. |
| S1 | serious | §0 hazard owners; R4 | The UB owner list "R1, R5, R11, R14" and the memory-corruption list "R1, R2, R3, R8" omit R4 and M6 (iii). A false licensed fact (a wrong `noalias`, a wrongly removed call) is the canonical miscompile path to corruption and UB; the table that claims every hazard has an owner leaves the backend-facing hazard unowned. | Add R4 (soundness clause) and M6 (iii) to the UB and memory-corruption owner rows. |
| S2 | serious | R11; R1; "Failure, partiality and typed outcomes" | (i) R11's acceptance test, P1 `read v[k]` refused unless k∉{i,j}, is an R1 hole-read case (the two parts were taken out), not a domain case, so it cannot discriminate R11 from R1; (ii) R11's theorem "progress lemma — no accepted program reaches a stuck state" is the global progress half of M6 and subsumes R1's lemma, so the two rows are not independent as the draft's stance requires; (iii) "arithmetic under the overflow behavior chosen in its name" is why-whitefoot's per-operation-name mechanism written into (a). | Give R11 a domain test (division by a runtime divisor, a narrowing cast, an index against a runtime length), move the progress lemma to M6 with R11 contributing the partial-operation clause, and replace "chosen in its name" with "chosen per operation site". |
| S3 | serious | R14 | R14 is a requirement on the specification, not a per-program property, and it is M6's assumption list rather than an independent row; its (b) column fixes a mechanism — "a per-storage qualifier ... spelled over interference, never over origin" is the map §5 Part D `foreign` qualifier — and its (e) column decides a model question ("a debugger is admitted as a non-interfering reader") rather than testing one. | Keep R14 as the enumerated assumption set only; move the qualifier's spelling and the debugger's admission to the open questions or to the candidate's mechanism sheet; make (e) "M6's statement lists no assumption absent from R14 and every P-program's external actors are named". |
| S4 | serious | R4 (sufficiency and bound clauses); M6 (iii); REQ "R4 and R6 are precision policies" (HIS-REQ-02) | (i) The sufficiency clause "the source can license per-access and per-span facts, not only per-parameter ones" is a language-level statement about the fact channel's granularity; the family cites the owner's ruling (design/log.md:168) that what reaches the backend "is an implementation detail the language never sees", and the disposition claims to honor it while the row contradicts it. (ii) The bound clause says a demand "is never elided by a licensed fact", but dead-store elimination of an erasing store needs no licensed fact; the clause misattributes the hazard and leaves the actual elision path unbound. (iii) No row defines the observation model (timing, erased stores) under which M6 (iii)'s "observable behavior" and a constant-time demand are judged. | Restate sufficiency as a measured property under M9 (P5's per-load facts as a floor test), restate the bound as "a demanded store, alignment or constant-time property is preserved by every lowering", and add the observation model to R14 so that erasure and timing demands are meaningful to M6 (iii). |
| S5 | serious | R12; R10 (d); "Failure paths and typed outcomes" | R12 admits a context that "must not fail in a given way (no-allocate, no-block)" and makes the checker enforce it. "No-block" is not a failure; it is the host-mechanism effect category (`blocks`) that R10 (d) and effects.md refuse, so the row re-admits through R12 what the list refuses in R10 without engaging that tension. | Limit R12's context clause to failure outcomes the context forbids (no-allocate as "the allocation-failure outcome is unreachable here"), and send blocking, latency and scheduling to R13's demand-conditional form or refuse them with a reason. |
| S6 | serious | R10; M3; REQ "M3 hole" (THE-REQ-14) | R10's theorem shape says the acceptance relation has no premise mentioning host crossing, yet M3 lists "linked adapters, host declarations" in the trusted base with a per-entry boundary obligation, and the disposition calls the trusted adapter "an admitted, enumerated exception". The boundary obligation is a proof rule that distinguishes a function by host crossing; the two rows contradict and the constitution's "no ... proof ... rule distinguishes" clause is violated by the draft's own admission. | State in M3 that the boundary obligation attaches to every trusted-base entry regardless of origin (an axiom, a linked definition, a host adapter alike), so R10's theorem holds, or record the adapter exception as a constitutional amendment rather than an "admitted exception". |
| S7 | serious | M3; "M3's boundary" family (AI-D0-08, AI-REQ-04) | (i) M3 (a) forbids a bare assume but allows "a named axiom" without saying who may declare one; if a writer may, the hole HIS-D0-06 closed reopens under a new name. (ii) The refusal of non-vacuity leans on "logic errors are the requirement author's" but never engages the floor argument the same row invokes: a stuck writer uses the escape, and a weakened `requires` or a defaulted result is exactly the escape a stuck AI writer reaches for. | Say that axioms enter only through the specification or an enumerated adapter declaration, never a writer-reachable form, and either own non-vacuity as a checkable obligation (e.g. a contract the writer authored may not weaken one the requirement author fixed) or state why the floor argument stops at the semantic boundary. |
| S8 | serious | Constitution "retaining control over software objectives and key tradeoffs", "reducing dependence on repeated human inspection"; M4 family (THE-REQ-05) | No row owns the human-facing half of delegation: what a person must read (a signature, a contract, a stated tradeoff) to trust delegated code without reading its body. THE-REQ-05 is folded into M7 and M8, which measure rejections and spec size; neither says the delegated interface is auditable, and the family's recorded tension "verbosity cheap for the writer can be prohibitive for the owner reading diffs" is not engaged. | Add a requirement (or a clause of R7) that the stated interface is sufficient for a reader who never sees the body, with a measured test (owner review time or diff size per change on the corpus). |
| S9 | serious | R9 (concurrent clause); P10 | "Costs only at the write" and "a synchronization discipline that transfers state facts at acquire and release" are cost and mechanism constraints inside a capability row; they select against ordinary locks (which cost at every acquire, including reads) and toward epoch/RCU shapes the map's Part D itself recommends against as a language model. This is a mechanism selection, which the ground rules forbid at D0. | Keep R9's capability statement ("several long-lived writable holders across threads is an accepted shape") and move "cost only at the write" to M9 as a measured property of P10. |
| S10 | serious | R5; REQ "R5 scope" (SYS-REQ-05, SYS-REQ-14) | R5 conflates a proved operator law (an algebraic property a checker discharges) with a determinism level (nondeterminism the spec admits). Floating-point addition has no associativity law, so under the row as written a float reduction is either refused or admitted by a false law; the level is the thing that admits it, and the row's (b) asks for "the operator law and its discharge" that does not exist. | Split: overlap by disjoint footprints (R5), overlap by a proved law (R5 with a discharge), and overlap under a named weaker level with no law (R14), each with its own program. |
| S11 | serious | R1 (b), (e); "Storage shapes outside the state model" (SYS-D0-07) | (i) R1 (b) "including write-once transitions" is a state mechanism written into the least-a-checker-needs column. (ii) The P4 test hardened to "legal iff push proved non-reallocating" fixes a capacity-conditional contract; a handle-only candidate (why-whitefoot §10) satisfies R1 and fails the test, so the test discriminates mechanisms, not the requirement. (iii) Proposed P11 "or R14 excludes it explicitly" misuses R14: a memoizing cache is an internal program shape, not an external condition, and the disposition does not engage the family's recorded tension that why-whitefoot §5 makes the absence a performance argument. | Remove the write-once clause from (b); restore P4's candidate-relative wording ("legal iff the candidate can carry validity across push"); make P11 a capability program with a stated pass/fail under R1 alone. |
| S12 | serious | M10 | "Polynomial in a named measure" does not discriminate the constitution's duty ("practical iteration at the target project's scale; termination alone is insufficient"); the measured N^2.9 already satisfies it, and cubic work in statements is infeasible for a kernel. The bound has no degree, no scale point and no wall-clock criterion. | State the bound as a degree per family plus a whole-project criterion (a stated project size and check time) that the measurement compares against. |
| S13 | serious | M6 (iv); "Observability" family (SYS-D0-06, RAD-D0-23) | (i) M6 (iv) names a menu of mechanisms (small-model check, translation validation, differential oracle) inside a requirement row; (ii) its test "a seeded checker bug is caught by the oracle before any miscompile" is not checkable in principle (it is a mutation score at best); (iii) profilers, dumps and a debugger reading an erased, optimized program (RAD-D0-23, SYS-D0-06) are dispositioned by one phrase ("an R14 non-interfering actor") with no statement of what such a reader may observe. | State (iv) as "the specification names a checker-defect evidence channel outside the accepted program" with a measurable test (seeded-defect detection rate), and add to R14 what a non-interfering reader may observe. |
| S14 | serious | Disposition tables (members never named) | Family-level dispositions exist for all 45 families, but these members receive no disposition and their reasons are not engaged: RAD-D0-24 (cost-model exposure), HIS-D0-14 (closed pattern catalog as writer contract, ADOPTED), HIS-D0-19 (obvious shape is the fast shape, FLOATED), SYS-D0-18 (R6 is R4 run backwards), AI-D0-13 (R6 as sub-requirement of R4/R5), HIS-REQ-10 (global mutable state ban unacknowledged by R9; R9's restated capability does not say whether a global writable holder is in it), SYS-REQ-11 (seqlock retry loops: writer-written, so admitted under restated M2, but unstated). | Add one line per member to §3 with adopt/merge/refuse and the tension it answers. |
| S15 | serious | R13 (termination clause); R4 | Termination "where a licensed fact presumes it (a removable call, EFF-3 purity)" is the same obligation as R4's soundness clause ("call removable ... true in every execution"), so the two rows are not independent, and the disposition says so ("an R4 soundness matter"). The P6 test names a helper that writes its slice as "removable", which no candidate can mark removable, so the test cannot discriminate. | Keep termination-for-removal inside R4's soundness clause, leave R13 with demand-conditional bounds, deadlines and deadlock, and give R13 a termination test on a program that actually demands it. |
| S16 | serious | M1 | The row forbids "portfolio order" from selecting acceptance while the disposition says a specification-fixed fixed-order portfolio qualifies; the row and its reason disagree, and "explicit steps are the certificate" fixes a mechanism inside (a). | Say "no order, budget or state outside the specification selects acceptance" and move the certificate form to (b). |
| S17 | serious | R8 | Two kinds of requirement share one row with unrelated tests: storage-ending events (a state theorem feeding R1) and physical demands (a backend-conformance check on lowered IR). The demand half is a capability with no hazard, so the row is not one lemma, and the "Hardware placement" family's recorded tensions — "R5 promises race freedom while fence appears nowhere", address-space non-aliasing being target-dependent and unverified — are not engaged (ordering is sent to R14 without a per-construct vocabulary; address space is adopted with no ground). | Split R8 into R8a (storage-ending and relocation events) and R8b (placement and physical demands), and state in R8b which demands are portable and which are target-conditional. |
| m1 | minor | R6 | A row with dashes in every column is counted among "26 rows"; it states nothing that must hold and nothing a checker needs, so by the ground rules it is not a requirement. | Keep the reclassification note in §3 and drop R6 from the row count. |
| m2 | minor | R4, R8 | R4 lists "constant-time operation" as an R8 demand; R8 (a) lists alignment, layout, address space and erasure only. | Add constant time to R8's demand vocabulary or remove it from R4. |
| m3 | minor | R15; "Admission policy" family | R15 is the quantification domain of M6 ("proved for the language with abstraction"), not an independent row; and its admission names no blocking program, which is the THE-D0-07 ground the draft adopted for capability rows. | Either cite the blocking program (P3 generic pool as written) or fold R15 into M6's statement of scope. |
| m4 | minor | R7 | "A declaration whose interface no caller can use" has no definition of "can use". | Define it as "the interface's entry conditions are unsatisfiable or its exit conditions unusable by any call site" or cite the SPEC-DELTA rule. |
| m5 | minor | M8 | A budget K with no value and no owner for choosing it is an unfilled slot, not a requirement. | Name who fixes K, when, and the interim figure the row is tested against. |
| m6 | minor | R3; "Refactor the R1–R3 state ladder" | The refusal of RAD-D0-06 is circular ("compose in M6 only if separate"); take and put are events on storage state, so the multiplicity lemma cannot be stated without R1's state and the claimed separation is asserted, not shown. | State the multiplicity lemma's signature (what it quantifies over) so its separation from the state lemma is visible. |
| m7 | minor | "Interference rows are one fact" | The reason prescribes "one distinctness relation" inside M6's theorem, which is a mechanism statement. | Say "M6 must state how R4's and R5's facts relate" and leave the relation's form to the candidate. |
| m8 | minor | R8; "Storage shapes" family (SYS-D0-10) | Treating third-party relocation as a storage-ending event answers the shape by killing every interior pointer; the family's recorded tension ("mutually exclusive without a fixup channel") is not engaged. | Add one line: interior pointers into relocatable storage are refused unless the candidate supplies a fixup form, and P13 tests that. |
| m9 | minor | R9 (sequential clause) | "Nothing beyond R1" contradicts the draft's own R4 (d) note that coinciding holders weaken licensed facts; the clause should say the hazard needs nothing beyond R1 and the cost lands on R4. | Reword as "hazard: R1 only; facts: R4 weakens where holders coincide". |

## 2. Families with no disposition

None. Every one of the 22 D0.md families and the 23 REQ.md families appears
in §3.1 or §3.2 of the draft with an operation named. The gaps are at member
level (S14) and in reasons that do not engage the recorded tensions (F1, F3,
S4, S5, S6, S7, S11, S17).

## 3. Mechanisms found inside requirement rows

R1 (b) write-once transitions; R2 (c) drop-flag-as-failure-edge; R9 "costs
only at the write"; R11 "chosen in its name"; R14 (b) "spelled over
interference, never over origin" and the debugger admission; R4 sufficiency
(per-load channel); M1 "explicit steps are the certificate"; M2 "the checker
never inserts a check"; M6 (iv) the oracle menu; disposition reason "one
distinctness relation".

## 4. Hazards or duties from the constitution with no owner

Stack and other implicitly consumed resources at exhaustion (F2); false
backend facts as a memory-corruption path in §0 (S1); the human-auditable
delegated interface (S8); the observation model under which erasure,
constant time and determinism are judged (S4, F3).


# File: critique-A-cost.md

# Critique of draft A — lens: COST AND FEASIBILITY

Refuter brief: find rows no deterministic checker could enforce, rows whose
acceptance test is unmeasurable, rows that contradict measured evidence in the
repository, rows that forbid the fast shapes the constitution wants, and rows
that silently assume a mechanism. Default: report when uncertain.

Evidence consulted beyond the briefed files: `spec/kernel-spec.md:22` (heap
and stack exhaustion are explicitly outside the outcome model today),
`compiler/src/backend/emitter.rs:4` (emits no alias promises),
`research/experiments/scoped-alias-channel/RESULTS.md` (democ-era, retired
prototype), `research/experiments/default-floor/RESULTS.md` (1.65x, 1.10x
replicated), `research/experiments/blind-writer/2026-08-28/REPORT.md` §6.2–6.3
(34/41 `len()` rebinds, 120 `region` blocks, 110-line prelude, three
coordinate-only diagnostics), `EVIDENCE-option-enumeration-2026-09-16.md:25-33`
(proof-use-cost `growing` 30→163→1689 ms ≈ N^2.5–N^3.4; spec 533 KB ≈ 130k
tokens vs 48k claimed), `design/language.md` decisions 2, 3, 5, 6 (no
exponential work; one observable behaviour; writer trials are not a language
ground; closed unit), `spec/kernel-spec.md:1929` (EFF-3), `:2403` (CAP-1).

## Findings

| # | Severity | Row / family | Finding | One-sentence fix |
|---|---|---|---|---|
| 1 | fatal | M2, M5→M9, R2(c); families "M2 restated", "M5 and performance guarantees", "R2, R3, R8 and R1 lifecycle dependence" | Draft A restates M2 as "no unstated failure edge" and moves the "no cost the writer did not choose" half into M9, a two-benchmark corpus ratio. A runtime drop flag, a reference count, or a Perceus-style reuse counter never traps, so it is not a failure edge; under this list they are admissible so long as the locked corpus meets the (unstated) ratio. The map's own R2/R3 tables disqualify drop flags ("M2 violated; WF refused it in LIV-1") and RC ("M5 violated for ordinary data"). R2(c) tries to keep the exclusion by asserting "a runtime flag is an unstated failure edge under M2", which is false: a flag is hidden state, not an edge. Result: the list admits a design the map rejects and the constitution's performance ranking argues against, and the only per-program cost rule left is a corpus aggregate. | Keep a per-program rule in M2 or a new M-row: "no lowered bookkeeping, check, or branch exists on a path the source does not name" (the exclusion of hidden state as well as hidden edges), and correct R2(c) to cite that rule, not "failure edge". |
| 2 | fatal | M2, R12, R13, R14; families "Failure paths and typed outcomes", "Resource bounds, termination and time" | M2's theorem "abort edges of the lowered program ⊆ image of source outcomes" is false for every accepted program that can exhaust its stack, and the current specification says so explicitly (`kernel-spec.md:22`: heap exhaustion, stack exhaustion and OS quotas "may stop execution without a Whitefoot value, status, or cleanup guarantee"). Draft A deletes that carve-out without routing it: R13 makes stack bounds demand-conditional, R12 does not list stack exhaustion, R14 says everything outside the model is "excluded, never undefined" but nothing in R14 excludes stack exhaustion. The only consistent reading is that stack bounding is mandatory for every program, i.e. unbounded recursion is refused by the requirement list itself — a major design consequence selected silently, contradicting R13's stated stance. | State in R14 which resource exhaustions are outside the model (and therefore permitted abort edges, with a defined stop), or make R13's stack bound unconditional; either way say which, in M2's (d). |
| 3 | serious | M8; families "Writer cost model", "Standard library and prelude cost" | The acceptance test "re-declared prelude lines per file → 0" is unreachable while PROG-1 (one closed unit, no import: `design/language.md:6`) is retained, unless the compiler or specification injects a prelude. That is a mechanism choice (a blessed prelude or a module system), selected silently by an acceptance test. Also K is left unchosen ("not its job"), so "Tokens ≤ K" is unmeasurable; and the draft's own additions (R14 memory model, M6 operational semantics and theorems, R11–R15) push the surface further past the 130k measured against 48k claimed without reconciling. | Replace "→ 0" with "counted against K", state K or the procedure that fixes it, and note that R14/M6 additions are charged to M8's budget. |
| 4 | serious | R9; family "R9 dependence on R1, R4, R5" | R9 is restated as a *capability* row demanding that "several long-lived pointers ... across threads with a synchronization discipline ... costs only at the write" be an accepted shape. (i) The only discriminating program is P10, marked "(future)"; elevating it makes every candidate unevaluable until threads, locks, atomics and an R14 memory model exist. (ii) "Costs only at the write" is a runtime cost profile fixed by requirement: it forbids seqlock/retry readers and reader-side epoch entry, which are the read-mostly fast shapes; the map's part D "recommended against" protocol-governed shared regions on exactly this cost ground. (iii) The live tree has no shared-memory threads at all (HIS-REQ-10). | Keep R9's sequential half; move the concurrent half to R14/R5 as "when a thread construct exists, the synchronization primitive's cost placement is a named level", with P10 as a future test, not a must-hold. |
| 5 | serious | R6 retired; families "R6 relocated", "R4 and R6 are precision policies", "R6 not independent of R4 and R7" | The largest measured writer tax is fact-kill imprecision at call boundaries: 34 of 41 `len()` rebinds exist only because ENT-5 kills a length fact when a callee's row names the whole parameter (blind-writer §6.2 item 1). Draft A demotes precision to "checker precision, not a requirement" and tests it only by P1 (`read v[k]`) and P4 (post-push read) — neither has a call whose footprint is coarser than its write. The row that owned the measured cost driver is gone and nothing measures it. | Add to R7 or M9 an acceptance test: "a call whose interface writes elements of `v` leaves `len(v)` facts standing" (P9 `reserve`/`drain` variant), or restore R6's precision half as a measured requirement. |
| 6 | serious | R8, R4 bound clause, M6(iii); family "Hardware placement, ordering, address spaces and secret erasure" | "Every accepted lowering honors the demand" for erasure and "constant-time operation" is unenforceable by a deterministic WF checker: LLVM register allocation and spills copy secrets to places no IR store reaches, dead-store elimination removes the erasing store, and constant-time needs a leakage model neither R14 nor the semantics states. The acceptance test ("present in the lowered IR after optimization") does not test the property claimed. Worse, M6(iii) says observable behaviour equals the semantics, and erasure of dead storage is unobservable in any semantics R14 states, so the theorem cannot even express the demand. R4 lists "constant-time operation" as a demand while R8's vocabulary omits it. | Narrow R8's demand set to what an IR check can witness (alignment, layout, address space, presence of a volatile erasing store) and mark constant-time and register-level erasure as out of scope until R14 carries a leakage model. |
| 7 | serious | M3 test, M6(iii); families "M3's boundary", "Erasure as a checkable requirement" | M3's acceptance test says the generated trusted base "for the corpus contains only adapters and R14 conditions". LLVM, the emitter, the allocator, the loader and the OS are trusted and unlisted. Either they enter the enumeration (then the test as written fails) or they do not (then M6(iii)'s "observable behaviour equals the semantics" is an assumption no tool enumerates, which is exactly what M3 forbids). Translation validation of LLVM is not a deterministic family this project can run. | Add the backend and runtime to M3's enumerated base as named axioms with their obligation ("translation validation or differential oracle, M6(iv)"), and restate M6(iii) as "lowering ignores proof terms except through R4" without the observable-behaviour equality clause. |
| 8 | serious | M6(i)(ii); family "Form of the requirement statement" | M6 makes the *specification* carry an operational semantics and an adequacy theorem for the full language with abstraction (R15) before any candidate is selected. The spec is 130k tokens of prose; a prose theorem is unproved, and a mechanised one for a language with R14 memory model, R11 partial operations and R15 generics costs more than any candidate. The repository's priority order puts "the next end-to-end experiment" first and `CLAUDE.md` says work is not planned in a document up front. The draft admits D13 is delayed but not by how much; the realistic artefact (the access-state small-model check) already exists and is what the draft's (iv) names. | Require M6(i)(ii) in the form the project can run — a bounded small-model check per discriminating program with fixed prior criteria — and make the full theorem a stated later obligation, not a precondition of selection. |
| 9 | serious | M7, M9; families "Repairable rejection", "M5 and performance guarantees" | Both acceptance tests are fixed-writer trials ("blind-writer repair-from-diagnostic rate"; "first checker-green artifact of a fixed writer with no hints"). `design/language.md` decision 5 rules that "a trial in which a model writes programs under some assistance describes that model and that assistance rather than a ceiling on the language" and is not a ground for a language choice. A fixed writer is also unreproducible across model retirements. M7's "the fix is applicable from the diagnostic alone" is a property of the writer, not of the rule. | Restate M7's test as a rule property checkable without a writer ("each rejection's `mechanical_fix` or missing-fact payload is non-empty and names a tree location"), and cite M9's writer trial as evidence, not as the acceptance criterion. |
| 10 | serious | M10; family "Checking cost and latency as a requirement" | "An edit rechecks only its stated dependency cone" selects incremental, cone-based rechecking — a compiler architecture — as a language requirement; under the retained closed unit the current checker has no such cone and P9's "one-token edit reopens no callee proof" cannot be measured. The polynomial bound's degree is "as stated" but never stated, so the measured N^2.9 for `growing` is compared against nothing; the adopted decision (`design/language.md:2`) is "never exponential", which the draft neither adopts nor tightens. | State the bound (or the procedure that fixes it: e.g. "≤ quadratic in explicit steps, ≤ linear in statements") and restate the cone clause mechanism-free as a per-edit latency bound. |
| 11 | serious | R1 (a), (b); families "Refactor the R1–R3 state ladder", "R1 bundles three properties" | "All three are properties of the storage, never of the pointer" is the per-storage-state decomposition of map §6, stated as a requirement. A borrow-duration mechanism refuses use-after-free by a property of the pointer; excluding it here selects the mechanism family before D1. (P7 already fixes this for the *programs*; the requirement row need not repeat it.) Likewise "(b) ... including write-once transitions" imports the storage-shapes mechanism. | Keep (a) as the three prohibitions; move "property of the storage" to the acceptance test (P7) where it already lives, and drop "write-once transitions" from (b). |
| 12 | serious | R12; family "Failure paths and typed outcomes" | R12 lists allocation failure as an *expected* failure with a typed outcome. Combined with M2 this makes every `box`/`buffer` construction a two-armed outcome or a proved-bounded arena — a whole-corpus rewrite, a real design fork (Linux `GFP_*` vs Rust abort-on-OOM), and a per-file token cost. The current spec deliberately leaves heap exhaustion outside the outcome model (`kernel-spec.md:22`). §4 "what this draft costs" does not name it. | Either name allocation failure in §4 as an accepted cost with its P2 test, or move it to R14's excluded-exhaustion set with a defined stop, leaving R12 to the failures the constitution calls "expected input and environment failures". |
| 13 | serious | R13; families "Resource bounds, termination and time" | (i) "Where a licensed fact presumes termination (EFF-3 purity), termination is proved" requires a termination witness for every `pure` function used for dedup or reordering — a new deterministic family that does not exist, with unmeasured writer cost; the acceptance test names P6's helper, which writes its slice and is never removable. (ii) "Deadlines" as a demand-conditional checkable property needs a WCET model of LLVM output on a named target; no deterministic checker in this project can enforce it. | Scope R13 to memory/stack bounds and a termination witness only where EFF-3 removes a call (not where it reorders), drop "deadlines" until R14 states a timing model, and pick a program that has a removable call. |
| 14 | serious | R15; family "Abstraction as a requirement" | R15 requires R1–R14 to hold for "dynamically dispatched and module-separated code". The retained PROG-1 excludes modules, and the language has no dynamic dispatch; a requirement that quantifies over constructs the language refuses adds no obligation but does add a theorem surface (M6 "proved for the language with abstraction"). Its acceptance tests name generic variants of P3, P5, P6, P9 that are not in the discriminating-programs file. | Restrict R15 to the abstraction forms the language admits (generics, closures, nominal identity parameters) and add the generic program variants to the suite by name. |
| 15 | minor | R2, R8, R10, R1; family "Storage shapes outside the state model" | Acceptance tests cite P11–P14, which exist only as one-line proposals in the draft; until written to discriminating-programs.md the tests are unmeasurable. | Write P11–P14 into the suite in the same change or mark those cells "pending". |
| 16 | minor | M9 test cell | "Reproduce (1.65x, 1.10x) in the current compiler" is the *default-floor* result, already replicated; the open result (HIS-REQ-04) is the obvious-shape 1.6x from why-whitefoot §3, and P5's "obvious loop within the stated factor" depends on an alias channel the current emitter does not emit. The cell conflates the two and states no factor. | Name the two measurements separately, state the factor, and note that the P5 half is unmeasurable until the alias channel is rebuilt. |
| 17 | minor | M1 | "No ... iteration budget ... selects acceptance" coexists with "forbidding a specification-fixed terminating fixpoint" being out of scope; a fixpoint that terminates by a fixed iteration cap *is* an iteration budget. | Say "a fixpoint bounded by lattice height, not by an iteration cap". |
| 18 | minor | M4 | "No special cases" is unmeasurable, and "a spec-level lint finds one spelling per construct" presumes a lint and a formal notion of "construct" that FORM-2 does not provide (D0 notes this). | Restate as "the grammar admits one production per construct name in a stated construct inventory". |
| 19 | minor | M6(iv) | "The access-state exhaustive check extended to every discriminating program" is infeasible as *exhaustive* for P3 (pool graph plus parallel map) and P10 (threads); the experiment's own criteria say not to infer bounds beyond the fragment. | Say "bounded model check with stated depth" rather than exhaustive. |
| 20 | minor | M3 | "The list is generated, never hand-kept" is a tooling mechanism, not a requirement. | Move to (b) as "derivable from the source tree". |
| 21 | minor | R7 (a) | "A declaration whose interface no caller can use is rejected" generalises HIS-REQ-06 (a result no caller can use under provenance) into an undecidable usability judgment. | Restrict to the provenance case: "a result whose declared origin no caller can name". |
| 22 | minor | R11 (a) | "Arithmetic under the overflow behavior chosen in its name" is the current mechanism (per-operation overflow spelling) stated as the requirement. | Say "under an explicitly selected overflow behaviour". |
| 23 | minor | R8 (a) | "Address space" is merged from SYS-D0-09, which the catalog marks an unverified LLVM claim, without carrying the flag. | Keep the unverified mark or drop address space until verified. |
| 24 | minor | R14 vs M8 | R14 requires the spec to state memory ordering and synchronization semantics (LKMM-scale text); M8 charges it to no budget line. | Add "charged to M8" in R14 (d). |
| 25 | minor | M11 | "A mechanical migration for each" rejected corpus program is a per-amendment tooling commitment on a spec that is at v0.57; the recurring cost is not in §4. | Allow "or states the loss" to cover the tool as well, and list the cost in §4. |
| 26 | minor | §3 member-level silence | Families are all dispositioned, but some members inside adopted families receive no word: RAD-D0-24 (published cost model), HIS-D0-19 (obvious shape is the fast shape, FLOATED), RAD-REQ-19 (M5 presumes shape not schedule — M9's "obvious shape" test still presumes it), SYS-REQ-13 (M5 unenforceable while the emitter states no alias promises — M9 inherits this and does not say so), THE-REQ-13 (no hidden control flow — implicit in M2). | Add one clause per member or state "subsumed by X" explicitly. |

## Families not dispositioned

None at family level: all 22 D0.md families and all 23 REQ.md families appear
in §3.1/§3.2 with a disposition. The member-level silences are finding 26.

## Cost summary

Draft A trades the writer's and the project's cost for formal completeness and
says so in §4, but §4 under-counts: it omits the every-allocation outcome
(finding 12), the mandatory stack bound the list silently implies (finding 2),
the termination-witness family (finding 13), the memory-model text in the spec
(finding 24), and the per-amendment migration tool (finding 25). Its
measured-evidence handling is weakest where the evidence is strongest: the
biggest measured writer tax loses its owner (finding 5), the spec-size overrun
is acknowledged and then made worse (finding 3), and the retired alias channel
is treated as reproducible (finding 16). The two fatal findings are both ways
the list, read literally, would admit or force a design the repository has
already rejected or never chosen.


# File: critique-B-soundness.md

# Critique of draft B — lens: soundness and completeness of the list

Draft: `/private/tmp/whitefoot-mechanism-research/d0/draft-B.md`.
Lens: rows that are mechanisms in disguise; pairs that are not independent;
constitution hazards with no owner row; families dispositioned with a reason
that does not engage the family's recorded tensions; acceptance tests that
cannot discriminate.

Severity: **fatal** = the list would select a wrong or unsound design;
**serious** = a requirement is wrong, missing, or unenforceable; **minor** =
wording.

## Findings

| # | Severity | Row / family | Finding | One-sentence fix |
|---|---|---|---|---|
| F1 | fatal | R14(a); D0 "M2 restated" members HIS-D0-04, HIS-D0-05, HIS-D0-07, HIS-D0-10 | R14(a) says "every partial operation whose domain is not proved yields a typed outcome with real control flow". That is global prove-or-handle as law (HIS-D0-05, REFUSED) and Result-everywhere (HIS-D0-04, REFUSED); the constitution and the premise say an unproved partial operation is *rejected*, never converted. The four refused rivals in the M2 family receive no disposition, and R14 silently adopts one of them. | Restate R14(a) as "a partial operation the writer chose to write in outcome form yields a typed outcome; an unproved domain is a rejection", and disposition HIS-D0-04/05/07/10 explicitly. |
| F2 | fatal | R4 (R6 merged in); D0 "R6 relocated"; REQ "R6 not independent of R4 and R7" | R4 now carries the frame's soundness half ("exactly the facts whose support lies in the written footprint are dropped") while R4(d) still says "a missing fact costs speed, never correctness". A missing *write* fact under-kills and lets R1/R2 proofs run on stale facts: that is a correctness failure. The frame needs a may-write over-approximation; the optimizer needs a must-distinct under-approximation. One relation read in opposite exactness directions is exactly the reason the draft refuses the R4/R5 merge, yet it accepts the same defect for R4/R6. A candidate scored under merged R4 can under-kill and be graded "slow, not wrong". | Either keep R6 as a soundness row with its own (d) ("a missing write fact is a rejection"), or split R4(d) into the two exactness directions and drop "never correctness" for the frame half. |
| F3 | fatal | R5(a)/(b); REQ "R5 scope: reductions" | Overlap on a shared accumulator "is admitted when the operator is *declared* associative". A declared, unproved algebraic law is a writer-accessible assumption (M3) and the why-whitefoot premise requires laws to be *checked*. Also, associativity alone licenses only re-bracketing; "unspecified result order" under arbitrary scheduling needs commutativity too. As written the list would admit an unsound reduction design. | Change "declared" to "proved (or an enumerated trusted-base entry)" and name the law per level: associative for fixed-tree levels, associative and commutative for unspecified order. |
| F4 | fatal | R5(a) "a data race is unrepresentable" vs R19(a)/R12 | Foreign-writable storage (DMA, device registers, shared mappings) is written by an agent with no happens-before edge in the program. Under any C11/LKMM-style model R12 names, a plain load racing such a write is a data race, i.e. UB. R19 only says "no contents fact survives", which fixes precision, not the race. No row gives accesses to foreign-writable storage a defined non-racing semantics (volatile/relaxed-atomic class). | Add to R12 or R19: accesses to foreign-writable identities are of a declared access class with defined (unordered but non-UB) semantics, and R5's DRF theorem excludes that class explicitly. |
| F5 | fatal | M2(a)/(e); D0 "M2 restated", REQ "M2 no runtime check", THE-REQ-17 | The discriminating criterion "a design that *requires* such a check to reach safety is a violation because the cost is not chosen per site" is not checkable: Vale's generational check written as a library `deref` returning a typed outcome is "writer-written control flow" and passes M2(a) verbatim. M2(e) "emitted instructions are accounted for by source constructs" is satisfied by every compiler's lowering (spills and prologues included). M2 as stated neither excludes nor admits dynamic-check designs on a checkable ground, so the list can select one by accident. | State M2 as a property of the *language rules*: no rule of acceptance is satisfied by the presence of a runtime comparison (safety never depends on a branch), and give a test on the rule text rather than on the instruction stream. |
| F6 | fatal | R17(a)/(e) | "Thereafter be read by many holders and threads with no synchronization" and "post-init reads emit an invariant load, no guard". A reader is guard-free only if it can prove it happens-after the transition (R12); `OnceLock::get` does an acquire load on every call for exactly that reason. R17 as written selects an unsound write-once cell, or, if the acquire load is required, its acceptance test cannot be met. | Restate R17(a): reads are guard-free exactly where the checker holds a happens-before fact for the transition; otherwise the read is in the initializing arm or carries the writer's acquire. Name the happens-before fact in (b). |
| S1 | serious | M6; premise table (PROG-1 "unspent by the checker") | M6 is a rule about how rows are written ("(b) Nothing; this is a rule about rows"), not a property a checker can verify of a candidate design; it is a methodology, and it silently makes two design choices: the backend is free wherever the source is silent (schedule/tile/worker count are never the writer's, which answers RAD-REQ-19 by fiat), and the backend consumes PROG-1. The premise table then claims PROG-1 is a droppable mechanism whose loss "opens separate compilation", while M6, R16 (monomorphization, syntactic instantiation termination) and R13 (enumerate callers to recheck) all spend it — including on the checker side. | Move M6 to a preface on how rows are written; record "backend free where the source is silent" and "PROG-1 is spent by R13/R16/backend" as named chosen positions, and fix the premise table's "unspent by the checker". |
| S2 | serious | R7(a)/(d)/Price; R13(e) | R7(a) says "the backend's facts at that call depend only on a bounded summary" while (d) and the Price permit inlining, LTO and a whole-program view (M6). Both cannot hold. Further, the Price "caller acceptance never depends on a callee body" is admitted by the premise table to hold only if summaries are pinned, i.e. only under a mechanism this draft says it does not select; and R13(e) ("a one-token body edit rechecks `drain` and its direct callers") presumes derived summaries, contradicting R7's own price (a body edit that leaves a written summary unchanged rechecks the callee only). | Delete "and the backend's facts at that call" from R7(a); state the price mechanism-independently ("a caller is judged by a summary that does not change when the body changes without changing the summary") and align R13(e) with it. |
| S3 | serious | R13(a)/(e); REQ "Checking cost" | "A fixed polynomial degree, published per family" is self-certified: a family may publish a cubic law and pass, so the test cannot fail any design. The catalog's recorded tension was "cost must be at most quadratic in writer proof steps and today it is not"; the draft neither bounds the degree nor says who chooses it. R13(a) also selects a mechanism (support-keyed incremental recheck via "the R4 write-footprint relation run over the edit"). | Fix the bound in the requirement (e.g. degree ≤ 2 in written proof steps, stated as the requirement not the family's choice) and restate edit stability as "recheck cost is bounded by the size of the edit's dependency cone" without naming the relation. |
| S4 | serious | M1(a) vs R11, R18, R20 | M1 says the verdict is "a function of the source bytes and the specification version only", but R11 (size and alignment per type, checked layout currency), R18 (cross-space distinctness "where the target guarantees it") and R20 (a target's variable-latency operation set) make acceptance target-dependent. The draft's own §4 admits it "imports target facts the spec must now state" without amending M1. | Add the target description (as a named, versioned input) to M1(a)'s function arguments, or make the target-dependent clauses of R11/R18/R20 backend-only facts that never touch acceptance. |
| S5 | serious | M1(a) vs disposition of THE-REQ-12 | M1(a) says harder proofs are "written finite steps the checker verifies without rediscovery", while the same row and the REQ disposition say "a fixed-portfolio certificate search would satisfy it". A portfolio search that finds a certificate inside the compiler *is* rediscovery; one outside it is a writer tool and not the checker. The row admits and forbids the same thing. | Say which: "a certificate-producing search may exist as a writer-side tool; the checker verifies only written steps and fixed families." |
| S6 | serious | R22(e), M9(e), M2(e); D0 "Observability" (HIS-D0-16/17) | Three acceptance tests need artifacts the draft says it forecloses: R22's "sanitizer run under a deliberately broken checker rule" is an instrumented build; M9's "stripping every `invariant` and `use` from wfgrep" produces a *rejected* program that has no lowering unless a check-skipping mode exists; M2's "diff of the IR against a facts-off lowering" is the superseded HIS-D0-17 pair. The disposition says HIS-D0-17 stays superseded and HIS-D0-16 is honored, but the tests reinstate both. | Either admit a test-harness-only facts-off lowering and say it is outside the language's observable behavior, or replace the three tests with ones on the trail alone (e.g. every emitted `noalias` traces to a listed fact). |
| S7 | serious | R22 (a)/(b); REQ "Checker observability" | R22 is a specific mechanism (a machine-readable fact trail as side output) standing in for the requirement (a checker defect has a channel to surface before miscompilation). M9(b) then depends entirely on it ("Nothing beyond R22's trail"), so a premise (erasure) becomes checkable only through a chosen artifact. R22(a) also speaks of "the checks it removed under proof", presuming an insert-then-remove model that M2 denies exists. | Restate R22 as "every fact the backend consumed and every safety-relevant transform a proof authorized is auditable by a foreign reader" and name the trail as the candidate mechanism. |
| S8 | serious | R3(a)/Price; map §7 coupling "`own` aggregates passed as handles" | "Whether an aggregate travels as bytes or as a handle is a stated, ABI-visible property of the class" and "forbids the compiler from choosing byte-copy versus handle-pass per call site" turns a map §7 *coupling choice* into a requirement. No hazard grounds it: R8's price already forbids relocation while an interior pointer names the storage; where none does, passing a small aggregate in registers is harmless and the ban costs M5 (no SROA/register passing). It fails M6's own test ("a restriction with no named ground is a defect of the row"). | Drop the calling-convention clause from R3 and let R8's price ("no relocation while an interior pointer names the storage") be the only restriction. |
| S9 | serious | R2 Price vs R3(a) | R2's price "forbids the compiler from inserting a drop flag, an epilogue cleanup, or an unwinder" forbids what R3(a) permits ("affine values may take a compiler-derived release") and what LIV-1's unconditional scope-exit release is. The ban is on *conditional* (flag-dependent) cleanup, not on derived static release. | Reword: "forbids a release whose execution depends on runtime state the source did not branch on". |
| S10 | serious | R1 Price | "Forbids the compiler from treating `Gone` bytes as unobservable garbage it may write freely" forbids stack-slot colouring and register-spill reuse of dead frames, standard transforms with no hazard behind them (the program cannot access ended storage by R1(a)). The price restricts the backend on no ground, failing M6's test. | Price R1 only on the checker: the compiler may not *assume* ended storage unwritten by others (that is R21's fact); it may reuse ended storage the program cannot reach. |
| S11 | serious | R12 Price / (e) | "Forbids the compiler from strengthening or weakening an ordering" and "emits exactly the writer's barriers" forbid the only correct lowering on strong targets (a C11 release store on x86 is a plain store; a redundant fence may be elided; ARM may need a stronger primitive than written). The price should be on the observable happens-before, not on the fence set. | Reword: "may not weaken the stated happens-before; may implement it with any target primitive". |
| S12 | serious | R5(a) level 2 vs R5 Price and M6 | "Associative reordering with bitwise-reproducible result" is a fixed reduction tree (CNR/ReproBLAS); it *is* a schedule commitment. R5's price says it "forbids nothing about how it schedules within the permitted set" and M6 says schedule is backend freedom; both are contradicted by the level. | Price level 2: "fixes the combine order the backend may use"; or drop level 2 and keep two levels. |
| S13 | serious | R14(a) "does not block"; R10(d); D0 "Failure paths" | A "does not block"/"may sleep" context effect is an effect category over a host mechanism, which the live effects decision rejected ("mechanism categories") and the draft's own R10(d) keeps ("Not its job: host mechanisms or scheduling"). The draft adopts the category without engaging that refusal. | Either state blocking as an ordinary resource/ordering fact (R12/R15) or name the reintroduction of a mechanism category as a position that reverses a live decision. |
| S14 | serious | R21 (b)/(e); P2, P3 | R21 requires "an allocation returns storage distinct from every live storage" and "allocator may write metadata inside ended storage", and (c) says the allocator body is checked once against its summary. For writer-defined arenas and pools (P2, P3) that check needs an invariant over the free list/ended blocks, which (b) does not list; R21(b) names only the summary. The P2 test ("`b4` is a fresh identity in reused bytes") cannot be met without saying what the checker minimally needs to verify the body. Also R21 is a composition of R1 (fresh identity on reuse), R4 (distinct result), R7 (summary), R14, R15, so it is not an independent row. | Add to R21(b) the invariant class over ended storage the checker must hold to verify an allocator body, or reclassify R21 as a worked instance of R1/R4/R7 like R10. |
| S15 | serious | REQ "R4 and R6 are precision policies" (HIS-REQ-02) | The draft reverses a live owner ruling ("what the compiler hands LLVM is an implementation detail the language never sees", `design/log.md:168`, `design/language.md:3`) "with M6 as the ground". M6 is the draft's own new row, so the reversal is grounded on itself and is not named as a pending amendment. | Mark the R4 standing as an amendment awaiting the owner, with the constitution's performance clause and the why-whitefoot measurements as ground, not M6. |
| S16 | serious | REQ "R9 dependence" member HIS-REQ-10; R9(a) | "Global mutable state is one instance, not a separate rule" reverses the live ban on global mutable state (`design/language/ownership.md:5`) without engaging its recorded reason ("with no shared-memory threads there is nothing a global lock would guard"). | Either keep the ban until threads exist and say so in R9, or record the reversal and the reason it no longer holds. |
| S17 | serious | M4 (a)/(e); D0 "Writer cost model" (AI-D0-01, AI-D0-17), REQ "M4 decomposition" (THE-REQ-05) | The refusal to split rests on "teachability, budget and prelude share one acceptance test (tokens per amendment)", but M4(b)/(e) list three different measurements (a spelling-uniqueness lint, a per-file prelude bill, a spec+cards token count). K is unchosen, so the row is unenforceable today by the draft's own admission. AI-D0-17 (retrieval, not window, is the scarce resource; rule-addressable cards beat a shorter monolith) and THE-REQ-05 (owner auditability of diffs, which M7 diagnostics and R22 trail do not address) receive no engaged reason. | Either split M4 into the three measured rows or give one composite metric with a chosen K; disposition AI-D0-17 and THE-REQ-05 on their own terms. |
| S18 | serious | R9(a)/(e) | "The cost lands on the write, never on the hold, never on a read that does not need it": under R12's model every read racing a write needs an atomic or protocol read, so "a read that does not need it" is undefined; "reader cost stated" is not a test. R9 is also R5+R12 applied to one shape plus a cost rule, so its independence is only the cost rule, which M5 could own. | Define the read class in (a) and give a measurable reader bound (e.g. one acquire load, no lock), or fold the cost rule into M5 and delete the row. |
| S19 | serious | Acceptance tests on programs outside the suite (R17, R18, R19, R20, R21, R10, R14, R15) | The ground rules fix P1..P10 as the programs every candidate must handle. Eight rows anchor their (e) on unlisted programs (`OnceLock` cell, copy-from-user, `dma_map_single`, key-zeroing, slab free pointer, descriptor with EINTR, `GFP_ATOMIC` context, declared stack depth). A candidate evaluated against the suite cannot be discriminated by them. | Add the named shapes as P11..P18 in `discriminating-programs.md` or restate the tests over P1..P10. |
| S20 | serious | Constitution Safety; no owner row | Stack exhaustion is memory corruption or a trap; R15 bounds stack depth only "where promised", and unpromised programs have no owner, while M2 forbids a guard-page trap. Also "the execution model and external conditions on which safety depends" (constitution) has no owner beyond R12's memory model; the hardware/OS assumptions (MMU, no foreign write outside a loaned extent) sit nowhere, and M3's base enumerates adapters, not assumptions. | Add an owner clause: either R15 makes stack bound unconditional (a default bound proved for every program) or R14 gives stack exhaustion a defined outcome; add an "external conditions" clause to M3's enumerated base. |
| S21 | serious | Backend-conformance tests presented as discriminators (R4(e), R11(e), R17(e), R18(e), R21(e), M6(e)) | "Vectorized with no guard", "no peel for alignment", "emits an invariant load", "IR carries the space", "`noalias`-return reaches the IR" test the *compiler's* lowering, not the requirement list's power to separate candidate ownership designs; any candidate can be lowered to emit them. They cannot fail a design. | Restate each as a fact the *checker* must be able to state at the named program point; keep the IR checks as compiler tests elsewhere. |
| S22 | serious | M5 (a)/(e); REQ "M5 is a goal" | "Within a stated factor of the best C or Rust implementation" names no factor and no fixed reference; "best implementation" is not an oracle and "no extra writer work" is undefined. The (e) cites the retired-prototype results (1.65x, 1.10x) which are beats, not a factor. The row is still not checkable in principle, which was the family's complaint. | Fix a reference implementation and workload per corpus program and a numeric factor in the row, or state M5 as the measurement obligation on D13 with the reference set named. |
| S23 | serious | M3 Price vs R4(d)/M6 | M3's price says "every R4 fact traces to a proof or to an enumerated base entry, which is what makes the facts non-defeasible", but R4(d) and M6 let the backend *add* facts by its own analysis, which trace to neither. Either the analysis-derived facts are outside M3's claim (and R22 must mark them) or the re-derivation authority contradicts M3. | Say that backend-derived facts are the backend's own trust domain, distinguished in R22's trail from checker facts, and narrow M3's price to checker-emitted facts. |
| S24 | serious | D0 "Resource bounds" member RAD-D0-22 (termination for EFF-3); R15 | RAD-D0-22 is not dispositioned. R4 emits `readonly`/`pure`-derived facts that let the backend delete or reorder a call; a non-terminating pure call deleted changes behavior. R15 makes termination "where declared", so an undeclared, non-terminating pure callee is a soundness hole in R4. | Either R4 requires termination as part of a deletable-call fact, or R15 makes termination a precondition of any fact that licenses call deletion. |
| S25 | serious | R17 independence; D0 "Storage shapes" tension | R17 is R1's `Uninit→Init` transition plus R4's invariant-over-span plus R12's edge; the only new content is "exactly once, then frozen", which is a state protocol (a mechanism). The catalog's recorded tension — why-whitefoot §5 makes the *absence* of such cells a performance argument — is not engaged. | Fold R17 into R1/R4 as a named state protocol candidate, and answer the §5 tension explicitly. |
| S26 | serious | R19(a) "a loan to the agent is a linear obligation with a program-visible completion point (the completion token or the unmap)" | This selects the escrow-plus-linear-token mechanism the catalog lists as a *need* ("needs an escrow state plus a linear completion token"); a requirement would say only that the loan has a discharge point the checker can see. Also the catalog's tension that R2 assumes program-ordered discharge is only half answered: the completion point is where the program *observes* completion, which must be stated. | Restate as "the loan's completion is a program-observed event that discharges the obligation" and name the token as a candidate mechanism. |
| M1 | minor | R4(a) vs R6 stub | R4(a) says "*exactly* the facts ... are dropped" (the precision half), while the R6 stub says the precision half moved to R13/M7. | Change "exactly" to "at least" in R4(a). |
| M2 | minor | R9(a) vs REQ disposition | The disposition says "sequential clause deleted from R9" but R9(a) retains "Sequentially this is R1 and nothing more" — and with R6 merged into R4, the sequential case also rests on R4's frame half, which the clause drops. | Delete the sentence or say "R1 and R4's frame half". |
| M3 | minor | R11(a) vs Price | "(a) the checker knows, for every type and every storage: size, alignment ..." contradicts "where none is stated the compiler is free" (then the checker does not know). | Restrict (a) to stated layouts. |
| M4 | minor | R13(e) | "30→163→1689 ms is not quadratic" is correct, but "must fit the published law" is vacuous while the law is family-chosen (see S3). | Depends on S3. |
| M5 | minor | R15(e) | "The derived bound equals the measured peak on wfgrep" would fail every sound over-approximation; a bound is ≥ the peak. | "bounds the measured peak within a stated slack". |
| M6 | minor | R1(e), R3(e), R4(e) | P8 "(no drop flag)" is an R2 test; P3 "link writes through several paths legal sequentially" is an R1(d)/R9 test, not R3; "wfgrep and zlib lose no overlap PAR-1/PAR-2 permit" is an R5 test. Misfiled tests weaken each row's (e). | Move each test to the row it exercises. |
| M7 | minor | R16(a)/(d) | "Instantiation terminates by a syntactic rule" names the current cycle-check mechanism; (d) disowns monomorphization-versus-dictionaries while (a) presumes instantiation. | "Instantiation terminates by a fixed rule"; make (a) neutral between specialization and dictionaries. |
| M8 | minor | R15 disposition (AI-REQ-11) | The reason says "R2's 'where promised' was weaker than the constitution", but the constitution is itself conditional ("for uses with explicit resource budgets"); the family tension (a candidate can satisfy R2 and not the constitution) is not resolved by moving the clause. | Say the constitution and R15 are both conditional and state what "promised" means. |
| M9 | minor | M7(d) | "Not its job: a bounded number of repair rounds" disowns the family's recorded infinite-repair-loop risk; nobody then owns it. | Add "a repair does not introduce a new rejection of the same rule at the same location" or assign the risk. |
| M10 | minor | D0 "Admission policy" (THE-D0-06) | The draft in effect adopts THE-D0-06 (adds every missing row) under a different label; the member is not named, and the tension that its bundled rows overlap seven families is not engaged. | Name THE-D0-06 as adopted in substance and say the overlap is resolved by the row split above. |
| M11 | minor | R11(e) "a struct's C layout is checked, not assumed" | Checked against what oracle? A C ABI description is a trusted-base entry (M3), not a checker fact. | Say the foreign layout is an enumerated base entry the checker compares against. |
| M12 | minor | Constitution "silent overflow" | No row owns arithmetic partiality; R14 only owns the outcome-arm side once an operation is partial. The ownership scope may exclude it, but then the list should say so. | Add a scope note or a stub row for partial-operation admission. |
| M13 | minor | D0 "Checking cost" member SYS-D0-15 (M1 budget-free vs closed world) | Not engaged: closed-world checks (R16 instantiation, R13 caller enumeration) are where unbounded runs bite, and R13's bound is per family, not per program. | State that R13's bound is in program size including instantiations. |

## Families not dispositioned

At the family level, every one of the 22 D0.md families and 23 REQ.md families
appears in the draft's disposition table with a disposition and a reason. None
is missing.

Member-level gaps inside dispositioned families (the reason names the family
but never the member, or contradicts it):

- D0 "M2 restated as no trap and no unchosen cost": HIS-D0-04 (Result-everywhere, REFUSED), HIS-D0-05 (global prove-or-handle, REFUSED), HIS-D0-07 (implicit retained checks, REFUSED), HIS-D0-10 (exceptions/effect handlers, REFUSED) — none dispositioned, and R14(a) adopts HIS-D0-05 (F1).
- D0 "Writer cost model": AI-D0-17 (retrieval as the scarce resource) — not engaged (S17).
- D0 "Resource bounds, termination and time": RAD-D0-22 (termination for EFF-3) — not engaged (S24).
- D0 "Checking cost and latency": SYS-D0-15 (M1 budget-free vs closed world) — not engaged (M13).
- D0 "Admission policy": THE-D0-06 — adopted in substance, unnamed (M10).
- REQ "M4 decomposition": THE-REQ-05 (human auditability) — reason does not engage (S17).
- REQ "R9 dependence": HIS-REQ-10 (global mutable state ban) — reversed without engaging the recorded reason (S16).
- REQ "M2 no runtime check": THE-REQ-17 (safety trap vs writer conditional undistinguished) — the draft claims to draw the line but the line is not checkable (F5).
- REQ "R4 and R6 are precision policies": HIS-REQ-02 — reversed on a self-made ground (S15).

## Summary of the lens

- Mechanisms in disguise: R13's support-keyed recheck and per-family bound,
  R17's write-once protocol, R19's linear completion token, R22's fact trail,
  R3's per-class calling convention, R14's blocking/allocation context effect,
  R5's closed three-level menu, M6's "backend free where silent" stance.
- Pairs not independent: R4/R6 (opposite exactness directions), R17/R1+R4+R12,
  R21/R1+R4+R7, R9/R5+R12, M9/R22.
- Constitution hazards with no owner: stack exhaustion outside a promised
  bound; the execution model and external hardware/OS conditions; silent
  overflow (scope note missing).
- Tests that cannot discriminate: M2(e), M5(e), R13(e) as self-published,
  the IR-inspection tests (R4, R11, R17, R18, R21, M6), tests on programs
  outside the fixed suite, and the three tests that need a facts-off or
  instrumented build.


# File: critique-B-cost.md

# Critique of draft B — lens COST (cost and feasibility)

Draft: `/private/tmp/whitefoot-mechanism-research/d0/draft-B.md`.
Lens: rows no deterministic checker could enforce; rows whose acceptance test
is unmeasurable; rows that contradict measured repository evidence; rows that
forbid the fast shapes the constitution wants; rows that silently assume a
mechanism. Default: report when uncertain.

Evidence consulted beyond the briefing set:
`research/experiments/proof-use-cost/baseline-2026-09-14.tsv` (growing:
31 ms / 1452 lines at N=16, 164 ms / 2812 lines at N=32, 1690 ms / 5532 lines
at N=64); `compiler/src/backend/emitter.rs:1-5` ("emits no overflow or alias
promises"); `research/experiments/scoped-alias-channel/RESULTS.md` (retired
democ channel: Rust's obvious shape is vectorized by loop versioning with 29
runtime guards and ties at n ≥ 32; the fact channel's durable wins are short
trips and 17x code size; whitefoot-control without facts and without
versioning runs 2.1 ns/element against 0.4); `research/experiments/checked-law-channel/RESULTS.md`
(laws are closed OP-8 table facts; undischargeable laws are hard rejects);
`research/experiments/default-floor/RESULTS.md:95-110` (1.65x/1.10x were
measured on the retired compiler and do not establish "that the current
proof/facts channels caused either win"); `spec/kernel-spec.md` at 533,492
bytes; `research/experiments/blind-writer/2026-08-28/REPORT.md` §6.2–6.3.

## Findings

| # | Severity | Row / family | Finding | Fix (one sentence) |
|---|---|---|---|---|
| F1 | fatal | M2, M6, R4(d) | M2's "enforceable form" bans every compiler-inserted "guard or branch" and its test says emitted instructions must be "accounted for by source constructs"; M6 simultaneously frees the backend to "analyze, specialize, inline, reorder". A real backend inserts branches in loop versioning, unroll epilogues, memcpy expansion, switch lowering and select expansion, so the literal test fails every LLVM `-O2` program (unenforceable) or, if honoured, forbids the very loop versioning the scoped-alias-channel measurement shows is how a missing alias fact recovers parity at long trips. Under the literal ban a missing R4 fact is a ~5x cliff (2.1 vs 0.4 ns/element measured), not the "costs speed" R4(d) describes. The list as written selects a design with a performance cliff wherever a fact is absent and gives no criterion to tell a safety trap from a cost-choice guard. | Restate M2 as "no inserted check whose failure has language semantics (trap, abort, outcome) and no check required to reach safety"; move cost-choice guards (versioning, peeling) to M6 as priced backend freedom, and make M2's test a check on the WF→IR boundary, not on emitted instructions. |
| F2 | fatal | R5(a)/(b) | Overlap on a shared accumulator is admitted when "the operator is declared associative"; R5(b) needs only "the operator's declared law". A declaration that selects observable behavior (reordering) without a proof is a writer-accessible escape in all but name (M3): a false law changes results with no trap and no diagnostic. The repository's own checked-law-channel evidence uses a closed table of laws that are facts by construction and hard-rejects undischargeable laws; a deterministic checker cannot prove associativity of an arbitrary user operator without induction or search (M1). | Replace "declared" with "checked": the law is a fixed-table fact or a written finite proof, and an unproved law refuses the reduction rather than the program. |
| F3 | fatal | R12, R17, R9, M2(e) | Three rows jointly presuppose that a seqlock read ("emits exactly the writer's barriers", "seqlock read ... is ordinary code") and a post-init read of a `OnceLock` cell in another thread at "zero cost" and "with no synchronization" are expressible while R5 says "a data race is unrepresentable" under R12's model. Under any DRF-only model (C11, LKMM's DRF core) a seqlock's data reads are racy, and a reader thread sees `Init` only through an acquire edge from the transition; the rows are mutually inconsistent and no candidate can satisfy all three tests. The list would select an inconsistent design or one whose memory model admits benign races, which R5 forbids. | Either R12 names a model with racy-read semantics and R5's "unrepresentable" is restated to "no racy write is unordered", or the seqlock test is dropped and R17's zero-cost read is conditioned on a checker-visible happens-before from the transition. |
| S1 | serious | R13 | "A fixed polynomial degree, published per family" forbids nothing: degree 4 is a published bound, so a cubic checker passes and the growing curve (exponents ≈2.4 then ≈3.4 between doublings) "fits". The row was adopted to answer "practical iteration at the target project's scale" and does not: no degree, no latency ceiling at a stated unit size, no memory bound. The scaling law is also confounded — source lines double along with N in the same measurement, so a two-variable law cannot be fitted from three points. | Name a degree (at most quadratic in unit size) or a wall-clock ceiling per unit size and per edit, and separate the source-size and proof-step variables in the measurement. |
| S2 | serious | R13(a), (e) | "An edit reopens only the proofs whose support it touched (the R4 write-footprint relation run over the edit)" names the incremental algorithm (support-keyed kill over footprints) and assumes a persistent checker state across edits, neither of which the current compiler has and neither of which is a requirement; the (e) "rechecks `drain` and its direct callers only" also contradicts R7(e) ("only callers whose summary changed"). | State edit locality as a bound on recheck scope (a function of the edit's summary delta), drop the named relation, and align the R13 and R7 tests. |
| S3 | serious | R4(e), R21(e) | The acceptance test is "the emitted IR carries the stated facts (`noalias`/scope, `readonly`, `memory`, `initializes`)". The emitter states it "emits no overflow or alias promises" and open question 15 says the channel must be built under either model; the test therefore measures a compiler capability that does not exist and cannot rank candidates at this decision point. Naming LLVM attributes also silently binds the requirement to one backend. | Split the test: at selection time "the candidate can state per-load distinctness for every P5/P6 access from source facts alone"; the IR measurement is a later compiler obligation, stated backend-neutrally. |
| S4 | serious | R4 (ground), M5(e) | The disposition grounds R4 on "CompCert is correct without alias metadata but slow on P5"; the retired channel measurement says LLVM reaches parity on P5's shape at n ≥ 32 by versioning, with the win at short trips and code size. M5's floor cites 1.65x/1.10x, whose own claim boundary says nothing about the facts channel causing the win and was measured on the retired compiler. Both rows overstate the evidence they cite. | Cite the measured wins as short-trip speed and code size, and mark the floor numbers as retired-compiler evidence that must be re-measured on the current compiler before they can test M5. |
| S5 | serious | M5(a),(e) | "Within a stated factor of the best C or Rust implementation" is unmeasurable: no factor is stated, "best" has no bound on expert effort (default-floor explicitly disclaims "beats expert Rust"), and P1–P10 have no reference implementations or workloads. The row also makes "the default-accepted shape is the fast shape" a property of a design, which only a corpus can exhibit. | State the factor, fix the reference as a named public library path per corpus program (as default-floor does), and restrict the fast-shape clause to "the obvious source of each P5/P6/corpus program is accepted and reaches the stated factor". |
| S6 | serious | M4 | The budget K is unchosen and the row is admitted as "currently failed"; the draft then adds twelve R rows and four M rows to a specification already at 533 KB (~130k tokens vs the 48k claim) and says M6 "adjudicates" the collision, but M6 is a rule about backend freedom and says nothing about vocabulary size. The list therefore contains a known-failed row, no number to fail against, and no rule to settle the collision it creates. "Tokens" is also unmeasurable without a named tokenizer. | Choose K (or bytes under a named tokenizer), and add to M4 the adjudication rule: a new row's vocabulary must fit within K or displace an existing row, decided per amendment. |
| S7 | serious | R7(a) | "A declaration whose summary no caller can use is itself an error" is enforceable only by enumerating all call sites (spending PROG-1 in the checker, which §2 says is "unspent by the checker") or by a satisfiability check on the summary (search, M1). The row's premise table and its (a) clause contradict each other. | Restate the clause as "a summary whose entry condition is syntactically unsatisfiable is an error" (a fixed check), or record that R7 spends PROG-1 in the checker. |
| S8 | serious | M3 price vs R4(d)/M6 | M3's price says "every R4 fact traces to a proof or to an enumerated base entry" while R4(d)/M6 let the backend add facts by its own analysis; those facts trace to LLVM's alias analysis, so LLVM must be an enumerated base entry "carrying the obligation it assumes", which cannot be listed on one page (M3(e)) and makes "one page" an unmeasurable criterion. | Distinguish stated facts (must trace) from backend-derived facts (trusted as part of the backend entry), and replace "one page" with an enumerated list checked by a test. |
| S9 | serious | R3 price | "The class fixes the calling convention" and forbids "byte-copy versus handle-pass per call site" forbids argument promotion, register passing of small aggregates and SROA across internal calls under a closed world; the fact the requirement needs is address stability of the storage while it is named (R8), not the convention. This forbids a fast shape M6 says may be forbidden only with a named ground. | Price R3 as "the storage's address does not change while any pointer names it"; leave the internal convention to M6 and the foreign boundary to R11. |
| S10 | serious | R11(a),(b) | R11(b) requires "a deterministic layout function per type", which fixes layout for every type, so the price's "where none is stated the compiler is free" is empty: field reordering and padding choice are forbidden everywhere (the Rust default is reordering). "The checker knows size for every type" is also target-dependent. | Require a layout function only where a layout is stated, an interior pointer is formed, or a foreign boundary is crossed; elsewhere the layout is the backend's. |
| S11 | serious | R12 price | "Forbids the compiler from ... strengthening ... an ordering" is unimplementable on TSO targets where every load is acquire and every store is release; the property is about observable behavior under the model, not instruction choice. | Price R12 as "the emitted program's behaviors are a subset of the model's behaviors for the stated orderings". |
| S12 | serious | R20 | The constant-time half needs a secret-tracking analysis of the *emitted* code against a per-target variable-latency set; no deterministic source checker can enforce it and LLVM gives no such guarantee, so enforcement requires a CT-preserving backend or a machine-code verifier, i.e. a large trusted-base entry the row does not name. | Keep secret erasure (checkable: the marked store is emitted); split constant-time into a row that names its target model and its trusted backend entry, or defer it. |
| S13 | serious | R15 | Deadlines require a WCET model of the target; the checker cannot derive one, and the (e) "the derived bound equals the measured peak on wfgrep" is unpassable because a sound static bound is conservative. | Drop deadlines or make them a target-model entry; change the test to "measured peak never exceeds the derived bound, ratio stated". |
| S14 | serious | R22, M9 | R22 defines the trail as "the difference between the facts-off and facts-on lowering", reintroducing the superseded HIS-D0-17 pair as a definition while claiming "no build mode"; "which retained facts were consumed at which lowering points" is unobservable inside LLVM. M9's test "stripping every `invariant` and `use` from wfgrep" produces a rejected program, so it cannot be run without a checker-bypass mode. | Define the trail as the fact set and retained/removed checks at the WF→IR boundary, and test M9 by lowering the checked program with and without that fact set at that boundary. |
| S15 | serious | M7(a) | "A rejection of a sound program names the restructuring it implies" is unenforceable: soundness of a rejected program is exactly the residue a no-search checker cannot decide. The (e) blind-writer measurement names no protocol or model. | Scope the clause to rejections under stated precision rules (reject-when-unsure), and cite the blind-writer PROTOCOL as the measurement. |
| S16 | serious | M8 | An unconditional "mechanical migration ships with every revision" makes the constitution's conditional clause ("once real projects have compatibility needs, weigh...") unconditional and is violated by every current amendment; it is also work no current experiment needs (CLAUDE.md priority 5). | Make M8 conditional: a revision states the migration or the loss for the retained corpus; a mechanical tool is required only once a project depends on compatibility. |
| S17 | serious | R11, R12, R17, R18, R19, R20, R21 | Seven rows' acceptance tests name programs outside P1–P10 and outside the corpus (`dma_map_single`, io_uring, LeanStore eviction, GPU kernel, `OnceLock`, `explicit_bzero`, copy-from-user). The debate's ground rule fixes the discriminating set; a candidate cannot be evaluated on programs that exist only as names. | Add pseudocode P11–P17 to the discriminating set for every named shape, or mark those (e) tests deferred. |
| S18 | serious | R19(e) | Refusing a swizzled pointer "after eviction" presumes the checker knows when a runtime buffer-pool decision happened; LeanStore detects it at runtime (tag bit plus optimistic latch), which M2 forbids unless writer-written, so the test silently assumes a linear pin-token mechanism. | State the test as "a page pointer is usable only while a held pin obligation is in scope". |
| S19 | serious | R16(a),(e), R18(a) | "Summary size bounded in the number of identity and effect parameters" and "a storage identity carries its address space" presuppose the map's §6 identity-parameter decomposition; a region-parameter (DPJ) or permission-based candidate has no such parameters to be measured on. The (e) "summary tokens per generic parameter stays linear on the corpus" is unmeasurable: the corpus has no identity- or effect-polymorphic generics. | Bound summary size in "the candidate's abstraction parameters" and define the test on P6/P3/P9 made generic, not the corpus. |
| S20 | serious | R14(a), R21(a) | "A context may declare that it does not allocate or does not block" and "an allocator declares its context effects" name a context-effect mechanism inside a requirement. | State the fact needed: "at each call the checker knows whether the callee may allocate or block and whether the calling context permits it". |
| S21 | serious | R4 (disposition) | The draft reverses an owner ruling (`design/log.md:168`: what reaches LLVM is an implementation detail) with M6 as the ground; a tree ruling is not reversed by a draft row, and the acceptance test on IR attributes couples the requirement list to one backend. | Record the reversal as a pending amendment beside the tree and keep R4's requirement-level test backend-neutral. |
| S22 | serious | R2 price | "The release schedule is the source's" forbids the backend from moving a release earlier (live-range and peak-memory reduction) even where R4/R21 effect facts make the move unobservable; M6 says a restriction without a named ground is a defect. | Price R2 as "no release inserted or removed; a release may move only where the summary facts make it unobservable". |
| M1 | minor | M6 | M6 is a form rule about rows ("(b) Nothing; this is a rule about rows"); its test is vacuous since a row passes by writing "Price: none", which M4, M7, M8, M9 and R22 do. | Move M6 to the "Form of the requirement statement" disposition as an authoring rule, not an M row. |
| M2 | minor | R1 price | "Forbids the compiler from treating Gone bytes as unobservable garbage it may write freely" forbids nothing a backend does (spills go to the frame, not to freed heap); a phantom price written to satisfy M6. | State the real price: DSE of stores to storage about to end remains legal; nothing else is forbidden. |
| M3 | minor | R9(a),(e) | "Never on a read that does not need it" and "a holder that never writes contributes no ordering edge" are wrong under R12 for P10's reader (a relaxed atomic load is still an ordering); "reader cost stated" is not a test. | Say "a reader pays the model's minimum for its ordering, no more" and give the reader a measured instruction count. |
| M4 | minor | R9 | "Global mutable state is one instance, not a separate rule" silently lifts a live ban (`design/language/ownership.md:5`) without naming it as a reversal, unlike the draft's explicit treatment of HIS-REQ-02. | Name the reversal in the disposition table as a pending amendment. |
| M5 | minor | R5(a) | The middle determinism level, "associative reordering with bitwise-reproducible result", is impossible for floating point under free reordering and identical to level three for modular integers; it needs a fixed reduction tree to mean anything. | Define the level as "a fixed reduction order independent of worker count" or drop it. |
| M6 | minor | R5(e) | "Compute-bench accumulator snapshot cases at each level" do not exist; only source-order cases exist today. | Mark the test as to-be-written. |
| M7 | minor | R16(a) | "Instantiation terminates by a syntactic rule" is the current mechanism (HIS-REQ-07's cycle check) stated as a requirement. | Say "instantiation terminates by a fixed rule". |
| M8 | minor | R18 | GPU local/shared/global spaces lie outside the constitution's named targets (kernels, compilers, browsers, embedded) and the row's (e) invents a GPU program; the admission rule the draft adopts requires a named real-system program. | Keep user/kernel/device/DMA spaces; defer GPU spaces until a target program grounds them. |
| M9 | minor | Disposition table | HIS-D0-19 ("obvious shape is the fast shape", FLOATED) is adopted verbatim into M5(a) without being named; THE-D0-06 ("add all ten missing requirements") is in effect adopted (twelve rows) while the table refuses only THE-D0-07. | Name both members' dispositions. |
| M10 | minor | M3(e) | "The trusted base of the current compiler can be listed on one page" is not a criterion (no page size) and is likely false once LLVM, the host adapters and the emitter's abort edge are counted. | Replace with "the trusted base is an enumerated list under test". |
| M11 | minor | R14 / R2 | Discharging R2 on every outcome arm multiplies checker paths per arm; the row does not note the R13 exposure. | Add "path count per outcome arm is bounded by the arm count" to R13's inputs. |
| M12 | minor | R11(b) | "Size ... for every type" needs pointer width and alignment per target; the checker's knowledge is target-parametric, which the row does not say. | State that layout facts are per target and the spec names the parameters. |

## Families not dispositioned

Every D0.md family (22) and every REQ.md family (23) receives a disposition
line in the draft's §3. None is missing at family level. Member-level silent
dispositions are M9 above (HIS-D0-19 adopted without name; THE-D0-06 adopted
in effect without name). Within "Writer cost model", AI-D0-17 (retrieval is
the scarce resource) and AI-D0-02 (one card per rule) are not addressed by
the merged M4 text; within "Checking cost", SYS-D0-15 (M1 budget-free vs
closed world) has no sentence. These are member gaps, not family gaps.

## Summary for the judge

The draft is the most backend-aware of the three, and its Price discipline is
the right instinct, but four of its prices forbid fast shapes (M2 literal,
R3 convention, R11 layout-everywhere, R12 no-strengthening), three rows are
jointly unsatisfiable under R5's race rule (R12/R17/R9 seqlock and
zero-cost reads), R5 admits a declared law as an escape the repository's
own law channel refuses, and R13, M4 and M5 carry no number to fail against.
Seven rows are tested on programs outside the fixed discriminating set.


# File: critique-C-soundness.md

# Critique of draft C — lens: SOUNDNESS AND COMPLETENESS OF THE LIST

Read against: `merged/D0.md` (22 families), `merged/REQ.md` (23 families),
MECHANISM-MAP §2/§3/§7/§8, the constitution, why-whitefoot §0–§2,
`discriminating-programs.md`, and `spec/kernel-spec.md` EFF-3 / SCOPE-3.

Severity: **fatal** = the list would select a wrong or unsound design;
**serious** = a requirement is wrong, missing or unenforceable; **minor** =
wording. "Family" names are verbatim from D0.md / REQ.md.

## 1. Findings

| # | Sev | Row / family | Finding | One-sentence fix |
|---|---|---|---|---|
| F1 | fatal | R5 (merged R4); D0 "Interference rows are one fact"; REQ "R4 and R5 are one relation" | The merge is grounded on "the missing-fact-rejects objection is void because M5 makes acceptance permission-independent", but R5 itself extends to `spawn/join` and P10, and a spawn is not a permission that can be erased leaving the same program: sequentialising a blocking producer/consumer changes meaning (deadlock), which M2 forbids. So for the concurrent half a missing distinctness fact must reject, and the D0 tension "R4 tolerates imprecision, R5 does not" is unresolved, not void. A candidate that silently sequentialises a spawn on a missing fact would satisfy R5(a) as written. | Split R5(a)'s "missing fact costs speed, never acceptance" clause so it is scoped to erasable permission constructs (PAR-1/PAR-2) and state that at a non-erasable construct (spawn) a missing fact rejects; or keep the optimizer consumer as its own clause with its own exactness direction. |
| F2 | fatal | M2, R12, premises table; D0 "M2 restated as no trap and no unchosen cost"; REQ "M2 'no runtime check' is a mechanism" | M2 restated ("every runtime condition is a typed outcome with source control flow") is satisfied by Result-everywhere (HIS-D0-04 REFUSED), implicit retained checks surfaced as outcomes (HIS-D0-07 REFUSED), and Vale/Mezzo generational checks written as `match`. The draft's own premises row says so ("generational or epoch checks become admissible"). The only discriminator left is M5's floor with an unset ratio, which cannot separate a well-predicted check from no check. Two recorded owner refusals are thereby un-refused without a row saying so. | Either keep the literal no-check clause as the adopted mechanism inside M2's (a) with its ground cited, or add to M2 a checkable clause that names what distinguishes a writer-chosen outcome from a laundered safety check (e.g. the check's condition must be a source expression the writer could have proved instead), and record HIS-D0-04/07 as still refused. |
| F3 | serious | R6; REQ "R4 and R6 are precision policies"; D0 "R6 relocated" | R6 is reclassified "writer-cost floor, not a hazard", (d) says "Safety: not its job", yet R6(a)'s "every other fact survives" has a soundness half: a fact surviving whose support did change (a stale `state = Init` across a free) is exactly what makes R1's per-access state check accept a use-after-free. R1(c) judges "against the named storage's state at that point" and that state is only current if kills are sound. The pair is not independent in the direction the draft states. | Split R6 into a soundness clause (no fact survives an operation that may have changed its support — owned by R1 or stated in R6 as a hazard) and the cost clause (no fact whose support was untouched is dropped), and give the soundness half a negative test (a design whose frame under-kills must fail P7/P2, not merely R6's re-proof count). |
| F4 | serious | M3, R10; D0 "M3's boundary: escapes and non-vacuous discharge"; REQ "M3 'no unsafe escape' hole" (THE-REQ-14 unengaged) | M3 says the trusted base is "unreachable from writer-authored code" and R10 requires "rules that mention the host boundary = 0", while SCOPE-3 trusts "linked function definitions" and every I/O primitive enters through a declaration whose contract someone writes. Either that declaration is writer-authored (violating M3 as stated) or a distinguished declaration form exists (violating the constitution's "no declaration ... distinguishes ... by whether its implementation crosses the host boundary" and R10's test). No design can satisfy M3 + R10 + the constitution as written; the family's "trusted adapter at R10 is either a violation or an admitted exception" is not answered. | State in M3 where a trusted contract may be authored and by whom (owner-fixed interface vs. writer), and make R10's test count trusted declarations rather than grep for the words "host boundary". |
| F5 | serious | M3 (d); REQ "M3 'no unsafe escape' hole" (AI-REQ-04); constitution objective 1 | Refusing non-vacuity as "uncheckable without an oracle" ignores the checkable half: separation of authority. If the same writer authors the `requires`/`ensures` and the body, weakening a contract is an escape that returns every contract to human inspection, which the constitution's first objective exists to reduce. "The owner's tests own it" makes a constitutional hazard ownerless. | Add a clause (to M3 or a new row) that an owner-fixed interface contract cannot be weakened by writer edits, with the test "diff of contract strength between the owner's interface and the accepted source = 0", and refuse only the general non-vacuity claim. |
| F6 | serious | R5 acceptance test; REQ "R4 and R6 are precision policies" (HIS-REQ-02) | The draft adopts HIS-REQ-02 ("what reaches LLVM is an implementation detail the language never sees") and simultaneously makes "per-load alias fact emitted for every column load, zero guards" R5's acceptance test. A language requirement whose test is an IR-metadata count contradicts the adopted reading; and since today's emitter emits nothing, every candidate starts equally red, so the test cannot discriminate candidates at selection time. | Restate the optimizer test at the language level (the source carries a checkable distinctness fact for every pair of column accesses in P5, and nothing in the language forbids its retention to lowering), and move the emitted-count to `design/compiler`. |
| F7 | serious | M2 test vs. D0 "R4's standing and the optimizer's authority" (RAD-D0-25) | RAD-D0-25 is dispositioned "re-derivation is a compiler decision no writer-facing requirement constrains beyond M11", but M2 restated forbids "any branch the writer did not write", which forbids loop versioning and guarded re-derivation outright. The two dispositions contradict. Separately, "runtime branches in lowered IR without a source branch = 0" fails on every real lowering (loop rotation, switch expansion, `select` legalisation, stack probes) or needs the very safety-check classification M2 restated claims to have removed. | Decide whether guarded re-derivation is an unchosen cost (then say so under M2 and RAD-D0-25) and define M2's metric as "no branch whose condition is a safety predicate the checker did not discharge", not raw branch counts. |
| F8 | serious | M11; D0 "Observability, debug builds and a checker-soundness oracle"; REQ "Erasure as a checkable requirement" | (i) M11 is a mechanism in disguise: "erased before lowering; executable is a function of the erased program plus retained facts" names the erasure-plus-retained-facts architecture; the requirement is that proofs affect cost only, never meaning. (ii) The facts-withheld differential run detects only a *false retained fact the backend exploited on the test inputs*; a wrongly accepted R1 violation with no retained fact behind it is invisible to it. The family tension "the facts are erased before the bug is observable" is not engaged; SYS-D0-06 is only half merged. | Restate M11 as "the meaning of an accepted program is independent of its proofs and permissions" with the erasure architecture named as the adopted mechanism, and state honestly that the differential channel covers retained-fact unsoundness only; put the acceptance-unsoundness oracle (small-model check, MAP §8 item 8) in the row or refuse it explicitly. |
| F9 | serious | Premises table (PROG-1), R7 | PROG-1 is declared "unspent; no row above needs it", but R7's new clause "a declaration whose result no caller can use is itself an error" is decidable only by seeing every caller, i.e. under the closed world. M3's "listed in one place, unreachable from writer code" and M11's whole-program erasure theorem also assume it. PROG-1 is spent by the draft's own rows. | Either mark PROG-1 as spent by R7's declaration-side obligation (and M3/M11) or reword the R7 clause as a property of the declaration alone ("no possible caller"). |
| F10 | serious | R9; REQ "R9 dependence on R1, R4, R5" (HIS-REQ-10 unengaged) | R9(a) "Sequentially it costs nothing beyond R1 and R6" and (b) "nothing beyond R1/R6" are claims true of the per-storage-state mechanism, not a requirement: a GhostCell-style brand or a typestate-with-permissions design writes P3/P7 at the cost of a token, and would fail R9 as written for a mechanism reason. Also the family's HIS-REQ-10 (global mutable storage ban, whose ground is parallel permission itself) is never mentioned: R9 does not say whether a global is a "holder". | Restate R9(a) as the shape admission only (P3/P7 sequential accepted; P10's cost lands on the write) and move "nothing beyond R1/R6" to the price column as the identity model's estimate; add a clause or explicit exclusion for global storage. |
| F11 | serious | R11; D0 "Abstraction carries identity and effect facts"; REQ "Abstraction as a requirement" | (i) "instantiation terminates by a stated syntactic rule" presupposes instantiation (monomorphization), which (d) says the row does not choose. (ii) The test "signature token count grows linearly in the number of identity/effect parameters" is trivially true of any syntax; the explosion the family names is in the *number* of parameters a signature needs as reachable storages grow (location polymorphism, existential packing), which the test never measures. | Reword (a) as "generic and higher-order definitions are checked once for all instances and instantiation is decidable", and make the test "parameters per signature over P3/P5 generic variants ≤ stated function of the storages the callee touches". |
| F12 | serious | R13; D0 "Resource bounds, termination and time" (RAD-D0-22 unengaged) | Termination is placed in the opt-in R13 ("an unpromised program pays nothing"), but EFF-3 licenses reordering of `pure` calls with equal arguments; reordering a diverging pure call earlier changes the observable prefix, so under M2/M11 the optimizer consumer needs termination wherever purity licenses reordering or deletion. Termination is not purely opt-in and the family named this tension. | Add to R13 (or R5's optimizer clause) that any optimization licensed by an effect fact that assumes termination requires either a termination promise on the callee or a licence that preserves divergence. |
| F13 | serious | Constitution "the specification must define the execution model and external conditions on which its safety guarantees depend"; D0 "Hardware placement, ordering..."; REQ "Layout and memory ordering" | R5 promises "a data race is unrepresentable" and now owns P10's acquire semantics, but memory ordering is refused "until threads arrive". Race freedom without a stated memory model is not a checkable claim, and the constitution makes the execution-model statement mandatory. This constitutional hazard has no owner row. | Add a clause to R5 (or a deferred row with a stated trigger) that the execution model and ordering vocabulary on which race freedom depends is stated in the spec before any thread construct is accepted; keep layout/ABI refused. |
| F14 | serious | Constitution "silent overflow"; R12, M2 | The draft reaches beyond ownership (R12, R13, M9, M10) yet no row owns arithmetic overflow, which the constitution names beside data races and uninitialised reads; R12(a)'s list of runtime conditions omits it and M2 depends on R12. | State in R12 that overflow is in scope (a typed outcome or a proved domain per operation) or state explicitly that overflow is outside this list's scope and which spec rule owns it. |
| F15 | serious | R8; REQ "R2, R3, R8 and R1 lifecycle dependence" (SYS-REQ-10 unengaged) | R8's (c) is "as an R1 state event and an R7 contract clause", its tests (P4, P3, P2) are all also R1's tests, and "whether aggregates travel as bytes or handles" is a representation mechanism. By the criterion the draft used to fold R4 into R5 (one fact, two consumers), R8 is a sub-clause of R1 plus R7 with no independent discriminator. The family's SYS-REQ-10 (R2 and R8 one decision for arenas/slabs) is not engaged. | Either restate R8 as the completeness requirement on the storage-ending/replacing set (MAP §8 Q1: "every member is contract-visible") with a test that enumerates the set against P2/P3/P4/P9, or fold it into R1/R7 and say so. |
| F16 | serious | All M-rows with thresholds (M4 K, M5 ratio, M6 N, M7 constants, M8 constant, M10 B and f) | Every threshold is "set by the first measurement". A test whose bound is fixed by its own first run passes by construction and thereafter detects compiler regressions only; it cannot discriminate between candidate designs at D-selection time, which is what a requirement list is for. | For each such row state the ground for a provisional bound now (why-whitefoot's 48k, the proof-use-cost series, the default-floor 1.65x/1.10x) and mark the measured re-set as an M9 amendment, not the initial value. |
| F17 | serious | M9 | "manual edits = 0 or the count is in the PR" is satisfied by reporting any number; the row is unfalsifiable. | Bound the count (or the count relative to corpus size) or drop the "or" clause. |
| F18 | serious | §1 as a whole; D0 "Keep the map's list verbatim" | The draft refuses the verbatim list because MAP §7 couplings contradict independence, then presents 23 rows with no coupling table: identity granularity (reaches R1, R5, R6, M4/M10), distinct-by-default with call-site discharge (R5, M5, soundness), storage state as term (M1/M7 cost, R1/R2 definiteness), the loan channel (MAP §8 Q14) are dropped silently. A reader cannot check the independence the list implies. | Carry MAP §7 forward as a section of the draft, re-keyed to the new row numbers, and mark each coupled pair in the rows' (d) columns. |
| F19 | serious | REQ "R5 scope: reductions and determinism level" (SYS-REQ-05 unengaged) | The disposition covers only the determinism-level clause; associative shared accumulators (reductions) are never mentioned, no reduction program is in the suite, and R5(a) "overlap only when footprints do not interfere" still excludes them by construction. | Either add a reduction clause to R5 (interfering footprints admitted under a stated operator law and determinism level) with a P-program, or refuse SYS-REQ-05 explicitly with a reason. |
| F20 | serious | R2 test | "count of release-selecting runtime bits in lowered IR = 0" cannot be evaluated: a drop flag and a writer's own boolean are indistinguishable in IR without a source-provenance rule, the same defect as M2's branch count. | Define the metric as "no runtime value not bound in source selects a release" and test it by source-to-IR provenance, not an IR count. |
| F21 | minor | R12(a) | "Every condition that can be false at runtime ... is a typed outcome" omits the proof route (P1: "legal iff k provably ... else the writer must branch"); "index outside a proved range" is not a runtime condition. | Reword: "is either discharged by proof before execution or a typed outcome with source control flow". |
| F22 | minor | R1 theorem shape | "no accepted program evaluates a read at `Uninit` or `Gone`" is stated in the checker's fact vocabulary, i.e. the identity/state mechanism's; a requirement theorem should be over the operational semantics (read of unallocated or unwritten memory). | Restate over memory events, keep the state names as the map's illustrative notation. |
| F23 | minor | M4; D0 "Writer cost model" (HIS-D0-14, HIS-D0-21 unengaged) | The draft names M1/M2/M3/M5's mechanisms but not M4's: "one spelling per construct to the byte" and the closed pattern catalog (HIS-D0-14 ADOPTED) are mechanisms serving regularity; HIS-D0-21 (more annotations make checking cheaper, REFUSED) bears on M7 vs. per-statement options and is not mentioned. The grammar-level spelling lint cannot see semantic duplicates (two constructs with one meaning), which the family flags as FORM-2's gap. | Name one-spelling and the pattern catalog as adopted mechanisms under M4 and state the semantic-duplicate limit of the lint. |
| F24 | minor | M7 | "At most quadratic in proof steps" has no stated ground (today's series is N^2.5–3.4; why 2?), and "near-linear in units" is not a checkable bound. | Give the ground for the exponent or mark it provisional; replace "near-linear" with a stated bound. |
| F25 | minor | M1 | "every verdict is derivable from spec plus source by a reader" is not checkable in principle; the proxy test (each AUTO family's completion stated) is fine but the (a) clause should say that. | Reword (a) to the proxy. |
| F26 | minor | R5 test, P1 | P1's `v[k]` diagnostic is a sequential hole-read precision question (ladder Step 2, R1 + M6), not interference; assigning it to R5 means the test cannot separate R5 from R1 on that program. | Cite P1's parallel-writes clause under R5 and the `v[k]` diagnostic under R1/M6. |
| F27 | minor | R7 test | "bytes the caller check reads outside the callee's contract = 0 under the current mechanism" is mechanism-relative: under an infer-and-pin rival the summary is the contract and the count is 0 by definition, so the test cannot discriminate. | Test the size and readability of what the caller must read (tokens per call site) instead. |
| F28 | minor | R10, R11, R13 tests | The descriptor program, the generic P3/P5 variants and "wfgrep with a stack-depth promise" are not in `discriminating-programs.md`; a row's test must be a program the suite carries. | Add them to the suite as P11–P13. |
| F29 | minor | R5, D0 "Resource bounds" (SYS-D0-11) | "Deadlock freedom waits for threads" while R5 already states what is known "before and after acquire" for P10; the two deferrals are inconsistent. | Defer both to the same trigger or admit both. |
| F30 | minor | M5, R5 determinism clause | Permission erasure ("changes no verdict") and a construct promising a weaker-than-source-order determinism level interact: the sequential run is one member of the promised result set. Stated nowhere. | Add one sentence to M5: erasure preserves acceptance and yields a result within the construct's promised determinism level. |
| F31 | minor | D0 "Storage shapes outside the state model" (SYS-D0-07) | Caches/lazy init are refused "until a corpus program needs them" without engaging the family's tension that why-whitefoot §5 makes their absence a performance argument. | State that the refusal is a deliberate performance stance, or add a write-once program to the suite. |

## 2. Families not dispositioned

Every one of the 22 D0.md and 23 REQ.md family names appears in the draft's
§3 with a verdict. None is wholly missing. The following families are
dispositioned with a reason that silently drops a member or does not engage a
recorded tension (details in the findings cited):

| Family | Source | Member or tension dropped | Finding |
|---|---|---|---|
| R5 scope: reductions and determinism level | REQ | SYS-REQ-05 reductions never mentioned | F19 |
| R9 dependence on R1, R4, R5 | REQ | HIS-REQ-10 global mutable state ban | F10 |
| R2, R3, R8 and R1 lifecycle dependence | REQ | SYS-REQ-10 R2/R8 one decision for arenas; R8 not discussed | F15 |
| M3 "no unsafe escape" hole | REQ | THE-REQ-14 trusted adapter at R10; AI-REQ-04 authority separation | F4, F5 |
| Interference rows are one fact (R4, R5, R9) | D0 | "R4 tolerates imprecision, R5 does not" answered only for erasable permissions | F1 |
| M2 restated as no trap and no unchosen cost | D0 | HIS-D0-04 / HIS-D0-07 refusals silently reopened | F2 |
| R4's standing and the optimizer's authority | D0 | RAD-D0-25 disposition contradicts M2 | F7 |
| Writer cost model: M4 restated and split | D0 | HIS-D0-14 (pattern catalog ADOPTED), HIS-D0-21 (REFUSED) | F23 |
| Resource bounds, termination and time | D0 | RAD-D0-22 termination for EFF-3 | F12 |
| Observability, debug builds and a checker-soundness oracle | D0 | "facts are erased before the bug is observable" (SYS-D0-06) | F8 |
| Storage shapes outside the state model | D0 | SYS-D0-07 performance tension | F31 |
| Keep the map's list verbatim | D0 | MAP §7 couplings refused as ground, then dropped from the list | F18 |

## 3. Summary of the lens

- **Mechanisms in disguise**: M11 (erasure architecture), R9(a)/(b) ("nothing
  beyond R1/R6"), R11(a) (instantiation), R8(b) (bytes vs handles), M4
  (one-spelling and the pattern catalog unnamed), R1's theorem vocabulary.
- **Pairs not independent as stated**: R6 soundness half ↔ R1; R8 ↔ R1/R7;
  R7's declaration clause ↔ PROG-1; M2 ↔ RAD-D0-25's re-derivation authority;
  R5's missing-fact clause ↔ M5 only for erasable constructs.
- **Constitutional hazards without an owner row**: execution model / memory
  ordering under race freedom; silent overflow; dependence on human inspection
  when contracts are writer-weakenable.
- **Acceptance tests that cannot discriminate**: every threshold set by its
  own first measurement (M4–M8, M10); R5's emitted-fact count; M2's and R2's
  IR counts; R11's linear-token test; R7's mechanism-relative byte count; M9's
  "or the count is in the PR".


# File: critique-C-cost.md

# Critique of draft C — lens COST (cost and feasibility)

Refuter brief: find rows no deterministic checker could enforce, rows whose
acceptance test is unmeasurable, rows that contradict the measured evidence
(compile-latency scaling, spec size, the retired alias channel, the
blind-writer taxes), rows that forbid the fast shapes the constitution wants,
and rows that silently assume a mechanism. Default: report when uncertain.

Evidence consulted beyond the briefed files: `research/experiments/proof-use-cost/*.tsv`
(baseline 2026-09-14 and paired rows 2026-09-15), `scoped-alias-channel/RESULTS.md`,
`effect-attrs-channel/RESULTS.md`, `blind-writer/2026-08-28/REPORT.md` §6–7,
`default-floor/RESULTS.md`, `proof-certificate-architecture/CHECKING-COST.md`,
`design/language.md`, `design/language/parallelism.md`,
`design/language/checks-and-proofs.md`, `spec/kernel-spec.md` (PROG-1, PRE-1,
PAR-2, line 1902, line 2102), `compiler/src/backend/emitter.rs`.

## Findings

| # | Severity | Row / family | Finding | Fix (one sentence) |
|---|---|---|---|---|
| F1 | fatal | R5 (merged R4+R5); REQ "R5 scope: reductions and determinism level" | R5(a) "two statements or iterations overlap only when footprints do not interfere" forbids every reduction: PAR-2 today recombines the loop accumulator across overlapping iterations (spec [PAR-2]), the checked-law channel measured the reduction shape, and the draft's own corpus test ("every PAR-2 overlap accepted today stays accepted") is contradicted by its own (a) clause. The reductions member SYS-REQ-05 is silently dropped from the family the draft marks "adopt". | Add to R5(a) "or the interference is a stated recombinable accumulation under a checked law", and disposition SYS-REQ-05 explicitly. |
| F2 | fatal | R1 (identity-end clarification) with R5(b) | R1 now says "the bytes may belong to another live identity (an arena, a pool)" while R5(b) hands the backend "distinctness by identity" without guards; in P2 the arena's storage `S` and the live block `b1` are two live identities over overlapping bytes, so identity inequality read as `noalias` is a false fact the optimizer may act on. The map's own soundness condition (§5: distinct identity parameters are not distinct until discharged, sub-identities included) is absent from the row. | State in R5(b) that distinctness is a proved disjointness relation over a containment structure of identities, never inequality of identity names, and add P2 `S` vs `b1` as a negative alias test. |
| F3 | fatal | M2 (restated) | M2(a) "no branch the writer did not write in source" and its test "runtime branches in lowered IR without a source branch = 0" forbid ordinary fast lowering: vectorizer remainder loops, loop rotation, LLVM loop versioning, select lowering all add branches with no source branch, and the scoped-alias-channel measurement shows versioning is exactly how the backend recovers a missing fact ("a missing fact costs speed", R5). The emitter today already emits a defensive abort edge on enum discriminants and an allocation-refusal abort (`emitter.rs`), so the count is red now and unmeasurable in optimized IR. | Restate M2 as "no failure edge (trap, abort, unwind) and no runtime check whose outcome selects between two source-visible behaviors; backend control flow that preserves the one observable behavior is unconstrained", and measure trap/abort edges, not branches. |
| S1 | serious | M6, M4 (K), M5 (floor), and every row's "rounds" price | The acceptance tests are runs of a blind-writer model: rounds ≤ N with "N set by the first measurement", card size K "set by measurement of blind-writer success", floor ratio from a "first-green artifact". Each is model-dependent and non-reproducible, contradicting M1's own two-machine reproducibility standard and the recorded tree decision that a writer trial "describes that model and that assistance rather than a ceiling on the language" (`design/language.md`). Setting the bound from the first run makes the test unfalsifiable on that run. Since M6 "sets the bound on every other row's rounds", every price column inherits the defect. | Keep the checker-side, model-free halves (byte-stable, parseable diagnostics; one rule and one location per rejection; a spelling lint) as the acceptance tests, and demote model runs to descriptive evidence with the model and protocol pinned. |
| S2 | serious | R6 (reclassified cost floor) | Test "re-proofs of facts whose support was untouched = 0" is undecidable under FN-1: the blind-writer tax 1 (34 re-bound `len` facts) exists precisely because a signature naming the whole buffer cannot say the length was untouched; the checker cannot know the support was untouched without either reading bodies (forbidden by R7's mechanism) or a finer footprint vocabulary (identity granularity, decision D1). The row therefore either has no measurable test or silently selects the granularity mechanism. | Restate the test as "facts killed whose support the operation's stated footprint does not name = 0" (precision relative to the stated footprint) and record the granularity trade as a cost to measure, not a zero. |
| S3 | serious | R7 (restated) | Test "bytes the caller check reads outside the callee's contract = 0 under the current mechanism" measures FN-1, not contract locality; an "infer, print, pin" candidate fails it by construction, so the acceptance test selects the mechanism the draft says it does not select. | Replace with a mechanism-neutral test: "the caller's verdict is a function of the source of the call site plus a bounded artifact whose token count is stated" and measure that artifact's size. |
| S4 | serious | R12 (added) | R12(a) "every condition that can be false at runtime … is a typed outcome" is the global prove-or-handle law the tree refused (HIS-D0-05: the unprovable-but-true residue is large under a no-search checker), and naming allocation failure as a typed outcome reverses the recorded treatment of heap refusal as a trusted-base limit that aborts with a resource record (spec line 2102, `emitter.rs` HEAP_RECORD) without saying the row starts red or that a decision is being reopened. | Scope R12(a) to "every operation the specification classifies as fallible", list allocation refusal as an open reclassification with its current abort semantics named, and state the row's current status as red. |
| S5 | serious | R13 (added, opt-in) | "time budget", "deadline" and "stack-depth promise" are not checkable by a deterministic source checker under M11: stack bytes and timing are decided by the backend after erasure (inlining, register allocation, target), `docs/ideas.md` records that timing bounds "require different evidence", and stack exhaustion today is a runtime abort record, not a source proof. Only heap bytes via allocation contracts have a checker-side derivation. | Restrict R13 to resources with a source-level cost semantics (allocation bytes, declared recursion depth as a count, termination) and refuse time and machine stack bytes at this decision point. |
| S6 | serious | M7 (added) | (i) "at most quadratic in proof steps" tightens the tree's recorded bound ("never exponential") with no stated ground; a spec-fixed family such as transitive closure over premises is cubic worst-case, so the row would forbid an admitted AUTO family and calls that a "compiler defect". (ii) "near-linear in units" is vacuous under PROG-1 (one unit). (iii) The cited series 30→163→1689 ms is unpinned (CHECKING-COST: "no pinned input bundle") and stale: the current branch measures growing-64 ≈ 120 ms and growing-128 ≈ 734 ms (paired rows 2026-09-15), a 64→128 exponent ≈ 2.6, still red. (iv) A wall-clock ceiling inside `make check` is a machine-speed gate, the thing M1 forbids for acceptance and that a two-machine run would flake on. | State the bound per named automatic family in operation counts over spec-defined quantities (premises, entries), cite a pinned bundle, and drop the wall-clock ceiling in favor of a fitted-exponent regression on operation counts. |
| S7 | serious | M8 (added) | "certificates for untouched units survive" and "count units rechecked and proofs reopened" presuppose certificates and an incremental checker; neither is adopted (proof-certificate-architecture is an investigation; `docs/ideas.md`: "Stop before building cache infrastructure if the cost is elsewhere"). The measurement cannot be run today and the row silently selects a mechanism. | Restate M8 as "the set of declarations whose verdict can change under a one-token edit is defined by the language and bounded by a stated function of the edit's syntactic reach" and measure the size of that set on the checker, without assuming caching. |
| S8 | serious | M9 (added) | A standing "mechanical migration ships with every revision" contradicts the constitution's conditional ("once real projects have compatibility needs") and the tree decision that migration cost "is not a ground" before adoption; at v0.57 amendment frequency it prices tooling per amendment that no experiment needs, and "certificates and archives survive" again assumes certificates. | Make M9 opt-in like R13 ("where a compatibility promise is declared") and keep only the META-5 delta counts as the standing test. |
| S9 | serious | Premises table (PROG-1 "unspent") | Spec line 1902 computes the reach-a-store closure "from signatures alone" because "the unit being closed [PROG-1], that transitive closure is exact"; that exactness serves R2/R5 today, and the tree grounds PROG-1 on "a module is a fact-loss surface", which is R7's ground. PROG-1 is spent, and dropping it would re-price R5's exactness and R7's declaration-side obligation, which the table does not say. | Record PROG-1 as spent by R7 (contract completeness) and by R5's exact transitive footprint closure, and price its removal on those rows. |
| S10 | serious | R9 (restated), price "0 sequentially" | "Sequentially it costs nothing beyond R1 and R6" contradicts the map §7 row "sequential aliased writes allowed: R4 where identities coincide, R6 kill precision, M5" and the measured alias channel, whose entire delta came from exclusivity-derived disjointness; where two long-lived writers coincide the optimizer fact is lost and the kill set widens. The price column hides the one cost R9 has. | Price R9 as "loss of the merged R5 distinctness fact and widened R6 kills for the coinciding storages" and require P3/P7 to report the emitted-fact count under the shape. |
| S11 | serious | R5 acceptance test (P10) and family "Hardware placement, ordering …" | R5's test includes P10, marked "(future)", and the clause "what the checker knows before and after acquire", while the draft refuses any ordering vocabulary; the test cannot be run for any candidate now, and when threads arrive it cannot be stated without the vocabulary the draft refused. | Move P10 to a deferred clause with the ordering vocabulary named as its precondition, or admit a minimal ordering row now. |
| S12 | serious | M5 (restated) | "Removing every permission annotation changes no verdict" assumes permission is annotation-based; today permission is derived from `for` and window constructs, so the theorem is vacuous for derived-permission candidates and non-vacuous only for annotation candidates, which biases the row toward one mechanism family. | Restate as "no permission judgment is an input to any acceptance verdict" (the parallelism tree's own wording) and test it on the checker's dependency graph. |
| S13 | serious | Disposition table (member-level silent drops) | Every family carries a disposition, but three members are dropped without a word: SYS-REQ-05 (reductions; see F1), THE-REQ-14 (the trusted adapter at R10 is either an M3 trusted-base entry or a violation; M3's list and R10's "0 extra spec" never say which), HIS-REQ-10 (the global-mutable-state ban's ground is parallel permission, which R9's "global bans" refusal touches without naming). | Add one line per member to the family's disposition. |
| S14 | serious | M10 and all Price columns | B, K, f and N are unstated, so "starts red" verdicts and every price are against unnamed thresholds; the R-row spec prices sum to ≈18.5k tokens for the ownership subsystem alone against the 48k whole-spec claim the draft cites, and the draft never checks the sum against B. | State B (or the derivation rule for B), and add a "sum of row prices ≤ subsystem share of B" test to M10. |
| m1 | minor | M1 | "every verdict is derivable from spec plus source by a reader" has no measurement; the two-machine, 10x-throttle byte-equality test is the enforceable part. | Drop the reader clause or make it "each AUTO family's completion is stated in the spec" only. |
| m2 | minor | R2 test | "release-selecting runtime bits in lowered IR = 0" is unmeasurable after optimization (a drop flag is an ordinary `i1` phi). | Measure at the checker: every release decision is static (LIV-1), not in IR. |
| m3 | minor | R1 price | "0 extra beyond state-changing ops" ignores the `len0 < cap0` (or equivalent) proof the writer must carry for every interior pointer held across a reallocating operation (P4, map §3 Step 3). | Price one proof step per interior pointer per relocating call. |
| m4 | minor | R10 | "0 extra spec" is contradicted by the map's foreign identity qualifier ("one qualifier", §5); the "descriptor program" is not in `discriminating-programs.md`. | Price the qualifier and add the descriptor program to the suite or cite an existing one. |
| m5 | minor | R11 | "signature token count grows linearly in the number of identity/effect parameters" is trivially true and on the wrong axis; explosion is parameters growing with fields × call depth; "instantiation terminates by a stated syntactic rule" names the live HIS-REQ-07 mechanism. | Measure parameters per signature as a function of type nesting on generic P3/P5, and leave the termination rule's form open. |
| m6 | minor | M10 prelude clause | "today 30–60%: red" is the 2026-08-28 measurement, taken before PRE-1 (spec §14, "Prelude (normative, counted)") existed; whether the tax survives PRE-1 is unmeasured. | Re-measure the prelude fraction against the current spec before calling the row red. |
| m7 | minor | M11 | The facts-withheld differential run is vacuous today (the emitter "emits no overflow or alias promises") and, once a channel exists, costs a second corpus build per `make check` that the row does not price. | Note the vacuity and price the second build. |
| m8 | minor | M4 | "no rule has a special case another rule must mention" has no lint defined; the tree's "no exception clause" decision is the checkable form. | Cite that decision and define the lint (e.g. no rule text contains a cross-rule exception). |
| m9 | minor | Price "rounds" column | "1, local" on nine rows discriminates nothing and is unmeasured; with M6's N unset it is a guess presented as a price. | Mark rounds as TBD until a pinned measurement exists. |
| m10 | minor | "Form of the requirement statement" (AI-D0-21) | The draft adopts programs-plus-measurement but does not say whether a row with no program in the suite (R10, R13 partly) is deleted or tolerated, which is AI-D0-21's whole content. | State the rule: a row without a suite program is provisional and named as such. |
| m11 | minor | M7(b) | "the complexity of each automatic family, stated in the spec" is a spec-form mechanism; the checks-and-proofs tree records structural ceilings (4096 entries), not complexity statements. | Move the statement to the design tree's compiler side, or record it as a pending amendment. |

## Families the draft failed to disposition

At family granularity: none. All 22 D0.md families and all 23 REQ.md families
appear by name in section 3 with a disposition.

Partial dispositions (a member dropped without a line):

- REQ "R5 scope: reductions and determinism level" — SYS-REQ-05 (reductions) undispositioned; the adopted R5(a) forbids it (F1).
- REQ "M3 'no unsafe escape' hole" — THE-REQ-14 (trusted adapter at R10) undispositioned (S13).
- REQ "R9 dependence on R1, R4, R5" — HIS-REQ-10 (global mutable state ban) undispositioned (S13).
- D0 "Form of the requirement statement" — AI-D0-21's deletion rule not stated (m10).

## Summary

The draft's stance (every row has a runnable test and a price) is the right
stance, and it fails on its own terms: the model-dependent tests (S1) make the
"rounds" price on every row unmeasurable; three rows forbid shapes the
constitution and the measured evidence want (F1 reductions, F3 backend
control flow, S10 the alias cost R9 hides); one pair of rows admits a false
alias fact (F2); and four meta rows (M7, M8, M9, R13) demand measurements no
current checker can produce or silently assume certificates and incremental
units. The cheapest repair is to keep the checker-side tests, delete the
model-dependent thresholds, and state B.


# File: judge-1.md

# Judge 1 — language designer's reading of drafts A, B, C

Role: the person who has to live with these rows for years. I weigh a draft by
whether its rows will still be true, independent and testable after the
mechanism under them changes twice, and by whether its mistakes are the
repairable kind (a clause) or the structural kind (a row that should not
exist, a ruling reversed on a self-made ground).

Read: MECHANISM-MAP §2/§3/§7/§8, constitution, why-whitefoot 1–140,
merged/D0.md, merged/REQ.md, discriminating-programs.md, drafts A/B/C, six
critiques.

## 1. Scores

Scale 0–10 per dimension; total is the plain sum (max 50).

| Draft | Completeness | Independence | Constitution | Checkability | Cost realism | Total |
|---|---|---|---|---|---|---|
| A | 7 | 6 | 7 | 5 | 4 | 29 |
| B | 6 | 3 | 4 | 4 | 3 | 20 |
| C | 5 | 5 | 5 | 5 | 6 | 26 |

### Completeness (against the 22 + 23 families)

All three disposition every family by name; none is missing at family level.
The differences are at member level and at the level of constitutional
duties.

- **A (7).** The §0 hazard-owner table is the only place in the three drafts
  where every constitutional hazard and duty is mapped to a row: overflow
  (R11), environment failure (R12), bounds (R13), execution model (R14),
  evolution (M11). Two duties still have no owner: stack exhaustion (both
  critiques, correctly, call this fatal) and the human-auditable interface
  (THE-REQ-05 folded into M7/M8 without engaging the "owner reading diffs"
  tension). Member silences are mostly floated or superseded items
  (RAD-D0-24, HIS-D0-19, SYS-D0-18, AI-D0-13, HIS-REQ-10, SYS-REQ-11).
- **B (6).** Widest adoption, thinnest engagement. It adopts THE-D0-06 in
  substance without naming it, silently adopts HIS-D0-05 (global
  prove-or-handle, REFUSED) inside R14(a), never dispositions HIS-D0-04/07/10,
  AI-D0-17, RAD-D0-22, SYS-D0-15, THE-REQ-05, and reverses HIS-REQ-10 without
  stating the recorded reason. Silent overflow has no owner. Breadth is not
  completeness when the reasons do not touch the recorded tensions.
- **C (5).** Its refusals of hardware, foreign-reader observability and three
  storage shapes are *reasoned* refusals under an admission policy, which is
  legitimate; I do not dock for that. I dock for the members it drops while
  marking a family "adopt": SYS-REQ-05 (reductions — R5(a) as written forbids
  the PAR-2 accumulator the corpus already runs), THE-REQ-14, HIS-REQ-10,
  RAD-D0-22, and for two constitutional duties with no owner: the execution
  model on which "a data race is unrepresentable" rests, and silent overflow.
  Dropping MAP §7 without carrying a coupling table forward is a completeness
  loss of a different kind: the list can no longer show the independence it
  claims.

### Independence of the rows

- **A (6).** The theorem-lemma form is the right instrument for checking
  independence and A is the only draft that uses it. R6 retired cleanly (its
  soundness half to M6(ii), its precision half to programs). R4/R5 kept apart
  with the coupling stated in M6. Real defects: R11's progress lemma subsumes
  R1's; R13's termination clause is R4's soundness clause; R14 is M6's
  assumption list wearing a row number; R15 is M6's quantification domain;
  R8 fuses a state theorem with a backend-conformance demand. All of these
  are fold-or-split repairs, not a wrong structure.
- **B (3).** Thirty-one rows and the overlap is structural. R17 is R1 + R4 +
  R12; R21 is R1 + R4 + R7 + R14 + R15; R9 is R5 + R12 plus a cost rule; M9
  depends entirely on R22. The R4/R6 merge is the one I call disqualifying
  for a base: the frame needs a may-write over-approximation and the
  optimizer needs a must-distinct under-approximation, and B accepts for
  R4/R6 the exact defect it refuses for R4/R5. Under merged R4 an under-kill
  is graded "slow, not wrong".
- **C (5).** The R4→R5 merge is argued from M5 erasure, which holds for
  PAR-1/PAR-2 and fails for spawn (a sequentialised producer/consumer can
  deadlock). R6's "safety: not its job" hides that a stale fact surviving a
  free is what lets R1 accept a use-after-free. R8 has no discriminator of
  its own by C's own merge criterion. Fewer rows than B and cleaner, but the
  two merges it does make are the two that matter most.

### Fidelity to the constitution and the recorded rulings

- **A (7).** Keeps every HIS refusal standing and says so; names FN-1,
  PROG-1, no-SMT, no-runtime-check, no-unsafe and fast-shapes as mechanisms
  or goals while keeping the requirement each serves. Two real breaches: the
  M2 restatement sends "no cost the writer did not choose" to a corpus
  aggregate, so a drop flag or a reference count is admissible by the text
  (LIV-1 and the map's R2 table refuse both); and R9's "costs only at the
  write" selects a synchronisation shape at D0. S6 (R10's theorem vs M3's
  adapter obligation) is a genuine constitutional tension A admits rather
  than hides.
- **B (4).** Reverses a live owner ruling (`design/log.md:168`, what reaches
  LLVM is an implementation detail) "with M6 as the ground", where M6 is B's
  own new row. Lifts the global-mutable-state ban in a subordinate clause.
  Adopts a refused rival (HIS-D0-05) in R14. "Declared associative" is a
  writer-accessible assumption, the thing why-whitefoot's premise ("checked
  algebraic laws") exists to exclude. On the credit side B honours
  "performance over ease" most faithfully and its R21 is the best reading
  of "external interaction through ordinary objects" for allocators.
- **C (5).** Reopens HIS-D0-04/07 by restating M2 so that Result-everywhere
  and a generational check written as a `match` pass. Its R1 identity/bytes
  clarification plus R5(b) identity distinctness hands the backend a false
  `noalias` on P2 (S vs b1). Model-dependent thresholds contradict
  `design/language.md` decision 5. Leaves the execution-model duty unowned.
  In its favour: the admission policy tracks the repository's priority order
  honestly, and "starts red" is the most truthful sentence in any draft.

### Checkability of acceptance tests

- **A (5).** Every row has a P-program or a theorem shape; the theorem
  shapes are checkable in principle, which the ground rules ask for. But M7
  and M9 rest on a fixed-writer trial, K and the M10 degree are unstated,
  M6(iv)'s "seeded bug is caught" is a mutation score at best, R8's
  erasure/constant-time demand cannot be witnessed in IR, and P11–P14 exist
  only as one-liners.
- **B (4).** IR-attribute counts cannot fail a design (any candidate lowers
  to them); seven or eight rows test on programs outside the fixed suite;
  R22/M9/M2 tests need the facts-off build B says it forecloses; R13's
  degree is self-published; M5 names no factor; R15's "equals the measured
  peak" fails every sound bound.
- **C (5).** The stance is right and the execution undercuts it: every
  threshold is "set by the first measurement", which passes by construction;
  rounds are guesses; R2 and M2 count IR bits and branches that no
  provenance rule distinguishes; R7's byte-count test is 0 by definition
  under the rival mechanism; M9's "or the count is in the PR" is
  unfalsifiable. What C gets right and A/B do not: the two-machine 10x
  throttle byte-identity test for M1, the spelling lint, META-5 counts, and
  the negative test on R9.

### Cost realism against measured evidence

- **A (4).** Acknowledges 130k vs 48k, then charges nothing for R14's memory
  model text, M6's operational semantics and adequacy theorem before any
  candidate is selected, R12's every-allocation outcome (a corpus rewrite
  the current spec deliberately avoids at kernel-spec.md:22), or R13's
  termination-witness family. Retires R6 and with it the owner of the
  largest measured writer tax (34 of 41 `len()` rebinds from ENT-5 kills at
  call boundaries). Treats the retired alias channel as reproducible.
- **B (3).** Adds twelve R rows and four M rows to a spec that already fails
  M4, and says M6 adjudicates a vocabulary collision M6 says nothing about.
  Four prices forbid fast shapes the constitution wants (M2 literal vs loop
  versioning, R3 convention vs SROA, R11 layout-everywhere, R12
  no-strengthening on TSO). Overstates its evidence: CompCert "slow on P5"
  against a measurement that shows parity at n ≥ 32; 1.65x/1.10x as facts
  channel proof when the result's own boundary disclaims that.
- **C (6).** The only draft whose every row carries a price, that refuses
  unmeasured rows, and that states which rows are red now. Its numbers are
  stale (the current branch measures growing-64 ≈ 120 ms, growing-128 ≈
  734 ms), its prelude fraction predates PRE-1, the row prices sum to ~18.5k
  without being checked against B, and M8 assumes certificates that are an
  investigation, not a decision. Still the most honest cost accounting.

## 2. Best base and why

**Base: Draft A.**

A language designer keeps the structure that will still be true after the
mechanism changes. A's skeleton — constitutional hazard-owner table,
(a)–(e) rows, premise table with "if dropped" consequences, disposition
tables, an honest §4 — is that structure. Its fatal findings are clause-level
(M2's second clause, a stack-exhaustion owner, the M5/R14 reference-behaviour
contradiction) and every critique of A supplies a one-sentence fix.

B's best content is its instinct (price every row; layout, ordering,
allocator, foreign storage are real) but its rows are mechanism-laden and it
reverses two owner rulings on a ground it invented; it cannot be the base
without being rewritten. C's best content is its discipline (price columns,
admission policy, runnable M1/M4 tests, R9 as shape admission) and that
discipline grafts cleanly onto A; but its two merges (R4→R5, R6 demoted)
are the two I would not sign.

## 3. What to graft onto A

From B:

1. A "Price" line per row stating what the row forbids the compiler from
   doing, as an authoring rule in a preface (not as an M row), with B's four
   over-broad prices corrected (M2 literal, R3 convention, R11 layout
   everywhere, R12 fence set → observable happens-before).
2. R1's correction that ended storage is *inaccessible to the program*, not
   inert; P2 requires it (SLUB free pointers, tcache `next`), and B's R21
   allocator contract as a worked instance under R1/R4/R7/R12, not a row.
3. R7's summary vocabulary list (outcomes, resource effects, ordering edges)
   into A's R7 interface vocabulary.
4. A closed set of determinism levels with the law/level split from A's own
   critique S10: overlap by disjoint footprints, overlap under a *proved or
   table-enumerated* law, overlap under a named weaker level with no law
   (float). "Declared" never appears.
5. B-soundness F4's fix: accesses to foreign-writable identities have a
   declared non-UB access class excluded from R5's DRF theorem; put it in
   A's R14.
6. B's honest premise-table analysis that PROG-1 is spent by the backend
   (devirtualisation, cross-declaration `noalias`, monomorphisation) as well
   as by M3/M6.

From C:

7. Price columns (spec tokens / program tokens / rounds) on every row, with
   every threshold given a *provisional grounded value* now (48k, the pinned
   proof-use-cost series, 1.65x/1.10x) and re-set only by a recorded M11
   amendment — never "set by the first measurement".
8. Restore R6's precision half as a measured writer-cost requirement, tested
   relative to the operation's *stated* footprint (C-cost S2's wording), with
   the 34/41 `len()` rebind tax as the current red value. The soundness half
   stays in M6(ii) as A has it.
9. R9 as a shape-admission row with a negative test (a design that rejects
   P3's sequential link updates fails R9); move "costs only at the write" to
   M9 as a measured property of P10.
10. M1's two-machine, 10x-CPU-throttle, byte-identical verdict-and-diagnostic
    test.
11. M4/M8 split with the spelling lint, the rule-ID retrieval index
    (AI-D0-17), and the prelude fraction re-measured against PRE-1.
12. M5 restated in the parallelism tree's own words: "no permission judgment
    is an input to any acceptance verdict", tested on the checker's
    dependency graph, not on annotation removal.
13. The admission-policy rule made explicit: a row whose test names no
    program in the suite is provisional and marked so (AI-D0-21's content
    without its deletion clause).
14. M11-style honesty on the checker-defect channel: the facts-withheld
    differential covers retained-fact unsoundness only; acceptance
    unsoundness needs the small-model check, named as such.

From the critiques, applied to A regardless of source:

15. M2 as two clauses: (i) no executable failure edge absent from source and
    interface; (ii) no safety check, bookkeeping state, or outcome-selecting
    branch the source did not write; backend control flow that preserves the
    one specified behaviour is unconstrained. Derive the drop-flag,
    reference-count and generational-handle refusals from (ii). Fix R2(c)'s
    false "failure edge" justification.
16. An owner for stack exhaustion and other implicitly consumed resources.
17. Move R4's sufficiency clause (per-load channel granularity) to M9 as a
    measured property, so A stops contradicting `design/log.md:168` while
    still claiming it honours it.
18. Split R8 into storage-ending events (R8a, a state theorem) and physical
    demands (R8b, a lowering conformance check), and narrow R8b to what an IR
    check can witness; defer constant time until R14 carries a leakage model.
19. Restate M6(i)(ii) in the form the project can run — a bounded small-model
    check per discriminating program with prior criteria — with the full
    theorem as a stated later obligation, not a precondition of selection.
20. Carry MAP §7 forward as a coupling section re-keyed to the new row
    numbers; every draft dropped it.
21. Name the members each disposition silently adopts or reverses (THE-D0-06,
    HIS-D0-19, HIS-REQ-10, RAD-D0-22, SYS-D0-15, THE-REQ-05, THE-REQ-14,
    SYS-REQ-05).

## 4. Disagreements the owner should decide personally

These are not synthesis questions; each either touches a live ruling, reopens
a refused rival, or is a design fork with a real corpus cost.

1. **R4's standing.** Is a backend-consumed alias fact a language
   requirement (A restated; B reverses `design/log.md:168` on its own
   ground; C merges it into R5)? This is a live owner ruling and only the
   owner can amend it.
2. **Optimizer authority over re-derivation (RAD-D0-25).** B lets the
   backend add facts; A calls it a mechanism; C sends it to
   `design/compiler`. Coupled: is guarded loop versioning an "unchosen
   cost" under M2? The scoped-alias-channel measurement says it is how a
   missing fact recovers parity.
3. **M2's boundary.** Literal "no compiler-inserted check" (B, C as adopted
   mechanism) versus "no unstated failure edge plus no unwritten
   bookkeeping" (A after graft 15). Decides whether Vale/generational/
   seqlock-shaped designs are refused by rule or priced by floor, and whether
   HIS-D0-03/04/07 stay refused.
4. **Allocation failure.** Typed outcome on every allocation (all three
   drafts) versus the current spec's abort-with-record outside the outcome
   model (kernel-spec.md:22, line 2102). A corpus rewrite and the Linux
   `GFP_*` versus abort-on-OOM fork; no draft names the cost.
5. **Stack exhaustion.** Mandatory stack bound for every program, a defined
   abort outside the model, or a typed outcome. The list silently implies
   the first unless the owner says otherwise.
6. **Admission of hardware rows and storage shapes.** Now (B), on a blocking
   program (C), or merged into R8/R14 (A) — against an M8 budget already
   failed at ~130k tokens versus 48k claimed. Only the owner can weigh
   kernel-scope ambition against the budget.
7. **Determinism-level menu.** One source-order guarantee (CAP-1,
   HIS-D0-16) versus a per-construct level (all three drafts, differently).
   Reopens CAP-1 and decides how float reductions are ever admitted.
8. **Requirement form.** Operational semantics plus adequacy theorem before
   selection (A's M6) versus programs plus measurements (B, C). This sets
   D13's delay and is a priority call under the repository's own order.
9. **Global mutable state ban (HIS-REQ-10).** B lifts it in a clause; A and
   C are silent. Its recorded ground is parallel permission itself.
10. **PROG-1.** Unspent (C), spent by M3/M6 (A), spent by the backend (B),
    and whether separate compilation is ever a goal.
11. **Writer trials as acceptance tests.** A's M7/M9 and C's rounds column
    versus `design/language.md` decision 5 (a trial describes a model, not a
    ceiling on the language).
12. **R6 frame precision.** Retired (A), merged into R4 (B), kept as a
    measured cost floor (C). The owner of the largest measured writer tax.
13. **Contract-authority separation (AI-REQ-04).** All three refuse
    non-vacuity as a checker duty; C-soundness F5 argues the
    owner-fixed-interface half is checkable and constitutional (objective 1).
14. **Threshold ownership.** Who fixes K, B, N, the floor ratio and the
    scaling exponent, on what ground, and by what amendment procedure.


# File: judge-2.md

# Judge 2 — compiler implementer building a deterministic checker

Read: draft-A, draft-B, draft-C; all six critiques; D0.md (22 families),
REQ.md (23 families); MECHANISM-MAP §2/§3/§7/§8; constitution; why-whitefoot
1–140; discriminating-programs.md. Repository evidence re-checked by hand:
`compiler/src/backend/emitter.rs:1-5` ("emits no overflow or alias promises",
"keeps a defensive abort edge for enum discriminants"); `spec/kernel-spec.md`
at 533,492 bytes; SCOPE-3 ("heap exhaustion, stack exhaustion, operating-system
quotas ... may stop execution without a Whitefoot value"); proof-use-cost
`baseline-2026-09-14.tsv` growing 31→169→1693 ms at N=16/32/64 (lines
1452/2812/5532), `context-baseline` and `paired-rows-2026-09-15` showing the
current branch faster in absolute terms but still superlinear (growing-64 ≈
173 ms, growing-128 ≈ 1158 ms on the context baseline); `design/language.md`
decisions 2 (never exponential), 3 (one observable behavior; what reaches
LLVM is an implementation detail), 5 (writer trials are not a language
ground), 6 (one closed unit).

## The lens I score from

I have to turn each row into one of three things: a rule the checker refuses
on, a compiler test in `make check`, or a measurement under
`research/experiments/`. A row that becomes none of those is a wish. A pair of
rows that pull my implementation in opposite directions is a bug factory. And
a row whose acceptance test names an artifact that does not exist (an alias
channel, a certificate store, a facts-off build, a P11) is not red, it is
unmeasurable, which is worse because it cannot fail a candidate.

Scores are 0–10 on: completeness against the 45 families; independence of
rows; constitutional fidelity; checkability of each row's (e); cost realism
against measured evidence.

## Draft A (safety and formal adequacy first)

**Completeness 8.** All 45 families dispositioned with reasons that engage
the recorded tensions more often than B or C. Only draft with a
constitution-to-row hazard table (§0), and the only one that owns silent
overflow (R11), execution model (R14), and failure behavior (R12) as separate
rows. Member-level silences (RAD-D0-24, HIS-D0-14/19, SYS-D0-18, AI-D0-13,
HIS-REQ-10, SYS-REQ-11) and stack exhaustion with no owner (both critiques
fatal, and SCOPE-3 confirms today's spec leaves it out) keep this off 9.

**Independence 6.** R4/R5 kept separate with the coupling named at M6: that
is exactly what I need, because my PAR judgement rejects on a missing fact
and my emitter merely drops metadata on one. R6 retired cleanly. But R11's
progress lemma subsumes R1's, R13's termination clause duplicates R4's
soundness clause, R14 is M6's assumption list rather than a row, R15 is M6's
quantification domain, and R8 bundles storage-ending events with
lowering-conformance demands. Half the added rows lean on M6.

**Constitutional fidelity 7.** Best mapping of duties to owners; keeps every
HIS refusal standing by name; names FN-1, PROG-1, no-SMT, no-check,
no-unsafe, fast-shapes as mechanisms. Two real defects: M2 restated as "no
unstated failure edge" loses the "no unchosen cost" half, so a drop flag or
RC counter is admissible in the row text (R2(c)'s "a runtime flag is an
unstated failure edge" is false; a flag never fails). And M6 makes an
operational semantics plus adequacy theorem a precondition of selection,
against the repository's priority order (next experiment first, no plan
document up front). R9's "costs only at the write" is a mechanism selection
against ordinary locks inside a capability row.

**Checkability 5.** The P-program tests are the best-chosen of the three:
P7 for R3, P9 for R7, P2 for R2, P4/P7/P2/P3 for R1 each separate the row
from its neighbours. Against that: theorem shapes are not tests I can run;
M6(i)(ii) needs a formal model this project does not have; P11–P14 exist
only as one-liners; M7 and M9 are fixed-writer trials (decision 5 says these
describe the model, not the language); M8's K is unchosen; M10 says
"polynomial" with no degree, so the measured N^2.9 satisfies it. Erasure and
constant-time demands "present in the lowered IR after optimization" cannot
be witnessed for register spills.

**Cost realism 5.** Acknowledges 130k vs 48k and the N^2.9 series, then
adds R14 memory-model text, M6 semantics, and five R rows without charging
them. Under-counts four costs §4 never names: allocation failure as a typed
outcome (whole-corpus rewrite; SCOPE-3 leaves heap exhaustion out today), a
silently mandatory stack bound, a termination-witness family for `pure`, a
per-amendment migration tool. Treats the retired alias channel as
reproducible in M9.

**Total 31.**

## Draft B (performance and systems practice first)

**Completeness 7.** All 45 families dispositioned; broadest coverage of
systems shapes (layout, ordering, address space, erasure, third-party,
write-once, allocator, trail). But R14(a) adopts global prove-or-handle
(HIS-D0-05, REFUSED) and Result-everywhere (HIS-D0-04, REFUSED) without
dispositioning either; silent overflow has no owner; stack exhaustion has no
owner; THE-D0-06 is adopted in substance and unnamed.

**Independence 4.** R4 absorbs R6 while keeping "a missing fact costs speed,
never correctness": an under-kill is a correctness failure. As the person who
would implement one relation read in two exactness directions, this is the
defect I would refuse to ship. R17 = R1 + R4 + R12; R21 = R1 + R4 + R7 +
R14 + R15; R9 = R5 + R12 + a cost rule; M9 depends wholly on R22's trail;
R7(a) ("backend facts depend only on the summary") contradicts M6 (inline,
LTO). R12/R17/R9 are jointly unsatisfiable under R5's race rule for the
seqlock and the zero-sync post-init read.

**Constitutional fidelity 5.** The instinct (every row prices the backend) is
right, and B is the only draft that says what a row forbids the compiler
from doing. But: R5 admits a *declared* associative law, which is a writer
escape (M3) and contradicts the checked-law channel; R17's guard-free reads
are unsound without a happens-before fact; acceptance becomes
target-dependent (R11, R18, R20) with M1 unamended; HIS-REQ-02
(design/log.md:168) is reversed on M6, a ground the draft made itself; the
global-mutable-state ban is lifted in a subordinate clause; four prices
forbid fast shapes (R1 Gone bytes, R3 calling convention, R11 layout
everywhere, R12 no-strengthening on TSO).

**Checkability 4.** Most (e) cells are IR-inspection tests on a channel the
emitter does not emit, named as LLVM attributes; eight rows test on programs
outside P1–P10; R22, M9 and M2(e) need the facts-off build HIS-D0-17
superseded; R13's bound is self-published (a cubic passes); M5 states no
factor; R7's "no caller can use" needs whole-program enumeration or search.

**Cost realism 5.** Cites the measured series correctly and is the most
backend-aware draft. Then: 31 rows against a spec already 2.7x over its
claimed budget, with K unchosen and M6 offered as the adjudicator although
M6 says nothing about vocabulary size; constant-time needs a leakage model
and a CT-preserving backend the row does not name; deadlines need WCET;
R22's "which retained facts were consumed at which lowering points" is
unobservable inside LLVM; overstates the retired channel (versioning reaches
parity at n ≥ 32; the durable wins are short trips and code size).

**Total 25.**

## Draft C (priced for the AI writer)

**Completeness 6.** All 45 families named. Three members dropped silently
and one of them matters: SYS-REQ-05 reductions, and R5(a) as written forbids
the PAR-2 accumulator overlap the spec accepts today, contradicting its own
corpus test. THE-REQ-14 and HIS-REQ-10 unanswered. Refuses the execution
model and ordering vocabulary although the constitution makes stating it
mandatory, and refuses layout with R8 noting "not owned". No owner for silent
overflow.

**Independence 5.** Honest about the R4/R5 merge, but the merge only holds
for erasable permission constructs; a spawn cannot be sequentialised on a
missing fact (deadlock is a meaning change), so the exactness direction
returns at P10. R6's soundness half is R1's precondition and the draft says
"safety: not its job". R8 is R1 + R7 by the draft's own fold criterion. R7's
declaration-side clause spends PROG-1, which the premises table calls
unspent. MAP §7 couplings dropped entirely.

**Constitutional fidelity 5.** M1/M3/M5 mechanisms named well; R13 opt-in
matches the constitution's conditional sentence exactly (the only draft to
read it correctly). But M2 restated is satisfied by Result-everywhere and by
Vale checks written as `match`, un-refusing HIS-D0-04/07, and the premises
table admits it; R12(a) is global prove-or-handle (HIS-D0-05); every
model-dependent threshold contradicts decision 5; R1's theorem is in the
identity/state mechanism's vocabulary.

**Checkability 6.** The only draft whose M1 (two machines, 10x throttle,
byte-identical verdicts and diagnostics), M4 (spelling lint), M10 (token
count, rule-ID index, prelude fraction in `make check`) I could wire this
week. Against that, F16 is decisive for selection: K, B, f, N, M7's
constants, M8's constant are all "set by the first measurement", so six M
rows pass by construction on their first run. IR branch count = 0 fails on
the emitter's existing abort edges; R2's IR-bit count is invisible after
optimization; R6's test is undecidable under FN-1; R7's test selects FN-1;
M8 presumes certificates and an incremental checker; the rounds column is
guesswork.

**Cost realism 6.** Best handling of writer cost: sums row prices (≈18.5k
for the subsystem), starts M6/M7/M8/M10 red on purpose, prices the prelude.
But cites the unpinned series as if stable; M7's quadratic bound would
forbid a cubic-worst-case admitted family (closure over premises) and its
wall-clock ceiling in `make check` is the machine-speed gate M1 forbids; M8
assumes caching infrastructure ideas.md says not to build; R13's stack
bytes and time are decided after erasure; R9 priced "0 sequentially" hides
the one alias cost the shape has.

**Total 28.**

## Verdict

Best base: **Draft A.** It has the fewest distinct fatal findings (three:
M2's lost cost half, stack exhaustion, the M5/R14 reference-semantics
contradiction), the only hazard-owner table, R4 and R5 kept separate, the
best-chosen P-program tests, and the most complete premise reclassification.
Its two structural weaknesses (theorem form as precondition; unrunnable
thresholds) are exactly what C supplies, and its missing backend discipline
is what B supplies once B's phantom prices are removed.

Grafts, in order of value to the checker:

1. M2 in the two-clause form the critiques converge on: (i) no executable
   failure edge (trap, abort, unwind) absent from source and interface;
   (ii) no runtime check required to reach safety and no bookkeeping state
   the source did not bind; backend control flow that preserves the one
   observable behavior is unconstrained. Tested at the WF→IR boundary, not
   by counting IR branches. This keeps Vale/Mezzo/generational designs
   excluded by rule and keeps loop versioning legal.
2. From C: the checker-side runnable tests (two-machine/throttled
   byte-equality for M1; spelling lint for M4; token count, rule-ID index
   and prelude fraction in `make check` for M8) replacing A's writer-trial
   criteria in M7/M9; writer runs demoted to evidence with model and protocol
   pinned.
3. From C: R13's opt-in wording, restricted to resources with a source-level
   cost semantics (allocation bytes, declared recursion depth as a count,
   termination); time and machine-stack bytes refused at this point.
4. From B: a Price line per row stating what it forbids the compiler from
   doing, as an authoring rule in the disposition (not an M row), with the
   four phantom prices dropped (R1 Gone bytes, R3 convention, R11 layout
   everywhere, R12 no-strengthening) and R2's price reworded to "no release
   whose execution depends on runtime state the source did not branch on".
5. From B: R1's "ended storage is inaccessible to the program, not inert;
   address reuse never revives an identity", and the allocator contract as
   a worked instance of R1/R4/R7 against P2/P3 (not a row).
6. From B (with B-soundness F3's fix): R5's reduction clause with a
   *checked* law (fixed-table fact or written proof) and the law named per
   level (associative for a fixed tree; associative and commutative for
   unspecified order), and a reduction program added to the suite.
7. From A-cost finding 2 / B-soundness S20: R14 states the excluded
   exhaustion set with a defined stop (matching SCOPE-3 today), so stack
   exhaustion has an owner without silently mandating a bound.
8. From B: memory-ordering vocabulary as a clause of A's R14 with an
   explicit trigger ("stated before any thread construct is accepted"),
   priced on the WF→IR happens-before, never on the fence set.
9. From C-cost S7 fix: edit stability stated as "the set of declarations
   whose verdict can change under a one-token edit is bounded by a stated
   function of the edit's syntactic reach", measured on the checker without
   assuming certificates or caching.
10. From B-cost S7: R7's declaration-side obligation restricted to a fixed
    check (syntactically unsatisfiable entry condition, or the provenance
    case), not "no caller can use".
11. From C: R9 reduced to shape admission (P3/P7 sequential accepted; P10
    deferred with its trigger), cost moved to the price line; and an explicit
    clause on whether a global is a holder (HIS-REQ-10).
12. From A-soundness S10: split "overlap by a proved law" from "overlap
    under a named weaker level with no law" so float reductions are admitted
    by level, never by a false law.
13. M10's bound stated as degree per family in operation counts over
    spec-defined quantities plus a project-size criterion, cited against a
    pinned bundle; never a wall-clock gate in `make check`.
14. Carry MAP §7 forward re-keyed to the new row numbers (C-soundness F18);
    none of the three drafts kept it.
15. Member-level lines for RAD-D0-24, HIS-D0-14, HIS-D0-19, SYS-D0-18,
    AI-D0-13, RAD-D0-22, SYS-D0-15, THE-D0-06, HIS-REQ-10, SYS-REQ-11,
    THE-REQ-14, AI-D0-17, THE-REQ-05.

## Disagreements the owner should decide personally

These are places where the drafts take positions a synthesizer cannot
average, or where a live tree ruling is touched.

1. **R4 and R5: separate rows (A, B) or one row (C).** Concretely: does a
   missing distinctness fact at a non-erasable construct (spawn/join, P10)
   reject, or leave the program sequential? C's merge is sound only for
   PAR-1/PAR-2.
2. **Optimizer re-derivation authority and R4's standing.** B adopts
   RAD-D0-25 (backend may add facts by analysis, never drop) and reverses
   design/log.md:168 (what reaches LLVM is an implementation detail); A calls
   it a mechanism; C calls it a `design/compiler` decision. This is a live
   ruling and only the owner can reverse it.
3. **M2's enforceable form.** Literal no-inserted-check (B), no-unstated-
   failure-edge (A), no-unwritten-branch (C). The real question: do cheap
   writer-visible dynamic checks (generational, epoch, seqlock retry) stay
   excluded by rule, or move to a floor argument under M9? A and C's texts
   un-refuse HIS-D0-04/07 by accident.
4. **Allocation failure as a typed outcome.** All three drafts make it one
   (A R12, B R14, C R12); the live spec (SCOPE-3) leaves heap and stack
   exhaustion outside the outcome model with an abort record. This is a
   whole-corpus consequence and a real design fork (GFP_* vs abort-on-OOM);
   none of the drafts names it in §4.
5. **Admission policy for systems rows.** B admits layout, ordering, address
   space, erasure, third-party, write-once, allocator and trail now; A admits
   constitutional-hazard rows now and capability rows on a blocking program;
   C admits lazily with two exceptions. The spec is at ~130k tokens against a
   48k claim, so this is a budget decision.
6. **Form: adequacy theorem and operational semantics before selection
   (A's M6) versus program suite plus measurement (B, C).** This sets how
   long D13 waits and whether the small-model check is the artifact or a
   stand-in.
7. **PROG-1: spent (A by M3/M6, B by the backend and R16) or unspent (C).**
   Decision 6 (one closed unit) is live; the owner must say which premise is
   load-bearing, since every "infer and pin" option depends on it.
8. **Determinism levels.** B fixes a closed three-level menu in R5; A names
   a level per construct in R14; C requires a level to be stated but refuses
   a menu. CAP-1 fixes one guarantee today, and B's middle level is a
   schedule commitment.
9. **Global mutable state and R9's concurrent half.** B lifts the ban
   (design/language/ownership.md:5) in a clause; A and C do not mention it;
   A's "costs only at the write" selects against locks. The owner should say
   whether R9 is a capability requirement now or a deferred shape with P10's
   trigger.
10. **Who fixes K/B and the tokenizer, and the interim value.** All three
    leave the budget unfilled; C's first-measurement rule makes it
    unfalsifiable; the interim number (48k claimed vs 130k measured) is a
    choice, not a derivation.
11. **Non-vacuity and authority separation (AI-REQ-04, AI-D0-08).** A and C
    refuse a checker obligation; C-soundness F5 argues the checkable half is
    "an owner-fixed interface contract cannot be weakened by writer edits".
    Whether the checker owns that is a constitutional reading.
12. **Stack exhaustion: mandatory bound for every program, or an excluded
    exhaustion with a defined stop.** Graft 7 above picks the second because
    it matches SCOPE-3; the owner may want the first for embedded targets.


# File: judge-3.md

# Judge 3 — the AI writer that must satisfy the list in a repair loop

Read: MECHANISM-MAP §2/§3/§7/§8, constitution, why-whitefoot 1–140,
merged/D0.md (22 families), merged/REQ.md (23 families),
discriminating-programs.md, draft-A/B/C, all six critiques. Verified in the
repository before scoring: `compiler/src/backend/emitter.rs:1-5` (no alias
promises, defensive abort edge), `spec/kernel-spec.md` SCOPE-3 (heap and
stack exhaustion outside the outcome model), spec size 533,492 bytes,
`research/experiments/proof-use-cost/baseline-2026-09-14.tsv` (growing
31/164/1690 ms) and `paired-rows-2026-09-15.tsv` (growing-64 candidate
≈120 ms, so the baseline series is stale on this branch),
`research/experiments/default-floor/RESULTS.md` claim boundary ("does not
establish that the current proof/facts channels caused either win"),
`design/language.md` decisions 2/3/5/6, `design/log.md:168`, blind-writer
REPORT §6.2 (34 of 41 `len` rebinds, 110-line prelude), and `[PRE-1]` in the
active spec (the prelude tax C calls "red" predates PRE-1 and is unmeasured
against it).

My lens: I am the writer who receives a rejection and must fix it from the
diagnostic, inside a token budget, without a second run reopening every
proof. A row helps me only if its test can fail a candidate before I am
handed that candidate, and if what it costs me (tokens, rounds, re-proofs)
is named against a number that exists.

## 1. Scores (0–10)

| Draft | Completeness | Independence | Constitutional fidelity | Checkability | Cost realism | Total |
|---|---|---|---|---|---|---|
| A | 7.5 | 6 | 7 | 5 | 4 | 29.5 |
| B | 6.5 | 4 | 5 | 5 | 4 | 24.5 |
| C | 6 | 5 | 6 | 6 | 6 | 29 |

The A/C margin is inside my error bar; the base choice below rests on
structure, not on the half-point.

### Completeness

All three disposition every family at family level (both critiques of each
draft confirm it). The difference is what happens below the family line and
whether the reason engages the recorded tension.

- **A 7.5.** The §0 hazard-owner table is the only device in the three
  drafts that checks the constitution's hazard list against the row list,
  and it is what found R11 (overflow), R12 (failure paths) and R14
  (execution model) as unowned. Its member-level silences (RAD-D0-24,
  HIS-D0-14, HIS-D0-19, SYS-D0-18, AI-D0-13, HIS-REQ-10, SYS-REQ-11,
  RAD-REQ-19, SYS-REQ-13, THE-REQ-13) are informational members none of
  which A's rows contradict. What it leaves unowned is real: stack
  exhaustion (both A critiques, fatal), the human-auditable interface
  (S8), the observation model for erasure and timing (S4).
- **B 6.5.** Broadest coverage in rows, and the only draft that answers the
  hardware, storage-shape and observability families with content rather
  than a merge. But the M2 family's four recorded refusals (HIS-D0-04/05/07/10)
  get no line and R14(a) adopts one of them; HIS-REQ-02 and the global
  mutable-state ban are reversed on the draft's own new row as the ground;
  THE-D0-06 is adopted in substance and unnamed. Silent overflow has no
  owner beyond a minor scope note.
- **C 6.** Refusing the hardware and third-party families under a stated
  admission policy is a legitimate disposition. Dropping SYS-REQ-05
  (reductions) while R5(a) "overlap only when footprints do not interfere"
  forbids what PAR-2 permits today is not: both C critiques rate it fatal
  and the draft's own corpus test contradicts its own clause. C also drops
  the MAP §7 coupling table it cites as the ground for refusing the verbatim
  list (F18), THE-REQ-14, HIS-REQ-10 and the R2/R8 arena member.

### Independence

- **A 6.** Keeps R4 and R5 apart with the coupling stated in M6, which is the
  reading both other drafts' critiques converge on (a missing "distinct"
  costs R4 speed and R5 permission; one row makes it a rejection). Its
  overlaps are theorem-level: R11's progress lemma subsumes R1's, R13's
  termination-for-removal is R4's soundness clause, R14 is M6's assumption
  list, R15 is M6's quantification domain, R8 bundles a state theorem with a
  backend-conformance check. All are restatement fixes, not structural ones.
- **B 4.** The R4/R6 merge is the structural defect: the frame needs a
  may-write over-approximation, the optimizer a must-distinct
  under-approximation, and R4(d) "never correctness" now covers the frame
  half, so an under-killing candidate is graded slow rather than wrong.
  R17, R19, R21 are compositions of R1/R4/R7/R12 with a mechanism as their
  only new content; R12/R17/R9 are jointly unsatisfiable with R5's "a data
  race is unrepresentable" (seqlock and zero-cost post-init reads); R7(a)
  and M6 contradict on whether the backend's facts depend only on a summary.
- **C 5.** Fewer rows, fewer overlaps, but the R4-into-R5 merge grounds
  itself on M5's erasability, which holds for PAR-1/PAR-2 and fails for
  spawn/join and P10 (C-soundness F1). R6's soundness half is R1's (F3); R8
  fails C's own one-fact-two-consumers criterion (F15); R9(a) "nothing
  beyond R1/R6" is the identity model's estimate written as a requirement
  (F10, S10).

### Constitutional fidelity

- **A 7.** Every constitutional hazard has a named owner, the M3 refusals
  stand, and M5 is split so the theorem-shaped half is separable from the
  goal. It loses points where its restatements contradict each other on the
  reference behavior (M5 sequential reference vs R14 weaker levels vs
  HIS-D0-16 one behavior: F3), where M3's per-entry boundary obligation
  distinguishes a function by host crossing against the constitution's own
  clause (S6), and where M6(i)(ii) demands a full operational semantics and
  adequacy theorem before selection, which the repository's priority order
  does not support (A-cost 8).
- **B 5.** Two direct violations: R5 admits a *declared* unproved algebraic
  law (the constitution requires machine proof, and the checked-law channel
  hard-rejects undischargeable laws), and R14(a) is global prove-or-handle
  as law, a recorded refusal. Reversing two owner rulings with the draft's
  own M6 as ground is a procedure failure under `design/skill`. The
  performance-clause reading is the strongest of the three and the
  foreign-writable storage gap (F4) is a genuine constitutional hazard B
  found and then left undefined.
- **C 6.** The "harness for AI agents" objective is taken seriously in every
  row. But race freedom is promised with the execution-model duty refused
  "until threads arrive" (F13, a constitutional sentence with no owner),
  silent overflow has no owner (F14), and M2 restated is satisfied by
  Result-everywhere and by generational checks written as `match`, which
  un-refuses HIS-D0-04/07 without a row saying so (both C critiques, fatal).

### Checkability

- **A 5.** Column (e) names a program and a property for every row, but
  many properties are theorem shapes over a semantics that does not exist
  (M6), four tests cite P11–P14 that exist only as one-line proposals, M8's
  K and M10's degree are unstated, M7 and M9 are fixed-writer trials
  against `design/language.md` decision 5, and M6(iv)'s "a seeded bug is
  caught" is a mutation score at best.
- **B 5.** The most concrete tests, and the least discriminating: IR
  attribute counts test the compiler's lowering (which today emits nothing),
  eight rows test on programs outside P1–P10, three tests need the
  facts-off build the draft says it forecloses, R13's per-family published
  bound is self-certified, M5 names no factor and no reference.
- **C 6.** The only draft whose M1 test I could run tomorrow (two machines,
  10x throttle, byte-equal verdicts and diagnostics), whose M4 spelling lint
  and M10 token count live in `make check`, and whose per-row diagnostic
  requirements ("names which sub-property failed", "distinguishes used-twice
  from never-released") are what I actually consume. Docked because every
  threshold is "set by the first measurement", which cannot fail a candidate
  at selection time (C-soundness F16), because the rounds column is
  model-dependent (C-cost S1), and because the R2 and M2 IR-count tests are
  not evaluable after optimization.

### Cost realism

- **A 4.** Acknowledges 130k vs 48k, then adds a memory model, an
  operational semantics and R11–R15 without charging them to M8. Retires the
  row that owned the largest measured writer tax (ENT-5 kills behind a
  call: 34 of 41 `len` rebinds) and tests precision only on P1/P4, neither
  of which has a coarse-footprint call. Treats the retired-prototype alias
  numbers as reproducible. Makes every allocation a two-armed outcome
  against the live SCOPE-3 carve-out without listing it in §4.
- **B 4.** Twelve R rows and four M rows onto a known-failed M4 with no K.
  Four prices forbid fast shapes the measured evidence wants: the literal M2
  ban forbids the loop versioning that the scoped-alias-channel result shows
  recovering parity when a fact is missing (a ~5x cliff, not "costs
  speed"); R3's per-class convention forbids register passing and SROA;
  R11's layout-function-per-type forbids reordering everywhere; R12's
  no-strengthening is unimplementable on TSO. Cites 1.65x/1.10x beyond
  their claim boundary. Unconditional migration tooling per amendment at
  v0.57.
- **C 6.** The right stance and the only draft that prices rows, marks rows
  red against the current compiler, and refuses unmeasured rows on a stated
  policy. Docked for: M7's "at most quadratic" tightening the tree's
  "never exponential" with no ground while a cubic fixed family (transitive
  closure) exists; the cited series being stale (this branch measures
  growing-64 ≈ 120 ms) and unpinned; a wall-clock ceiling inside `make
  check` being the machine-speed gate M1 forbids; M8 assuming certificates
  and an incremental checker that are investigations, not decisions; the
  prelude "red" predating PRE-1; and R9's price hiding the one cost it has
  (the merged distinctness fact and widened kills where holders coincide).

## 2. Best base: draft A

Grounds, in order of weight:

1. **Its defects are restatements; C's and B's are structural.** A's three
   fatals (M2 loses the "no unchosen cost" half; stack exhaustion unowned;
   M5/R14/HIS-D0-16 disagree on the reference behavior) are each fixed by
   rewriting one row's (a) and (d). C's lead fatal (R4 merged into R5) and
   B's (R6 merged into R4, R5's declared law, R14 adopting HIS-D0-05,
   R12/R17/R9 jointly unsatisfiable) require restoring or deleting rows.
2. **The §0 hazard-owner table is the completeness instrument the debate's
   ground rules ask for** ("nothing may be dropped silently"). Neither B nor
   C has one, and both left a constitutional hazard unowned that the table
   would have caught.
3. **R1–R10 keep the map's numbering**, so the owner can read the list
   against MECHANISM-MAP §2 line by line. B renumbers by insertion and C by
   merge.
4. **Premises table is the most careful** on FN-1 (mechanism serving R7 and
   M10) and PROG-1 (spent by M3 and M6, not unspent as C claims and as both
   C critiques refute).
5. **The writer-cost split (M4/M7/M8) matches what I consume**: regularity,
   repair and budget have different tests and different failure modes; B's
   single M4 with one K cannot tell me which one I broke.

A is not a good list as written; it is the best scaffold. The grafts below
are what make it one, and most of them come from C.

## 3. Grafts

From draft C:

- Price column per row (spec tokens / program tokens / rounds), with each
  threshold given a provisional grounded value now (48k claim, 130k
  measured, proof-use-cost series pinned to a bundle, default-floor
  ratios) and the "set by first measurement" clause deleted.
- R6 restored as a measured precision row (writer-cost floor), with the
  test restated relative to the stated footprint: "facts killed whose
  support the operation's stated footprint does not name = 0", and the
  blind-writer `len` rebind count as its red baseline.
- M1 acceptance test: two machines and a 10x CPU throttle, byte-identical
  verdicts and diagnostics; drop A's "portfolio order" sentence and C's
  unmeasurable "derivable by a reader" clause.
- M5 stated as "no permission judgment is an input to any acceptance
  verdict" (mechanism-neutral, not annotation erasure), plus one sentence
  that erasure yields a result inside the construct's promised level.
- Admission policy as C states it: THE-D0-07 with named exceptions
  (hazard-owner rows now, writer-cost rows now, abstraction now), frequency
  refused; every row without a suite program marked provisional.
- Per-row diagnostic obligations in (e): the diagnostic names which
  sub-property of R1 failed; R3's "used twice" is distinguishable from R2's
  "never released"; a killed fact names the killing operation.
- Naming M4's adopted mechanisms (one spelling to the byte; the closed
  pattern catalog, HIS-D0-14) and HIS-D0-21's refusal in the M4 row.
- "Starts red" status per row against the current compiler and spec.

From draft B:

- A "Price: what this forbids the compiler from doing" line on every row,
  kept as an authoring rule in the form disposition, not as an M row, and
  with the four over-pricings (M2 literal branch ban, R3 convention, R11
  layout everywhere, R12 no-strengthening) replaced by the critiques'
  one-sentence fixes.
- R1 restated: ended storage is inaccessible to the program, not inert;
  address reuse never revives an identity; allocator metadata in ended
  storage is admitted (P2, P3 need it).
- Foreign-writable storage gets a declared access class with defined
  non-racing semantics, and R5's race theorem excludes that class
  explicitly (B-soundness F4) — placed in A's R14.
- A reductions clause in R5 under a *proved or enumerated-base* law, with
  the level naming which law (associative for a fixed tree; associative and
  commutative for unspecified order); B's three-level menu carried as the
  owner-decision placeholder, not adopted.
- Allocator contract as a worked instance and acceptance test of
  R1/R4/R7 (P2, P3), not a row.
- Loan to a non-program agent restated as "a program-observed completion
  event discharges the obligation", the token named as a candidate
  mechanism.
- The premises table's two-consumer reading of FN-1 and PROG-1 (checker
  locality vs backend whole-program facts).
- R2's price reworded: "no release whose execution depends on runtime
  state the source did not branch on".

From the critiques of A (repairs the base needs before any graft):

- M2 as two clauses: (i) no executable failure edge absent from source and
  interface; (ii) no lowered bookkeeping, check or branch whose outcome
  selects between two source-visible behaviors on a path the source does
  not name; backend control flow that preserves the one observable
  behavior is unconstrained. Derive the drop-flag and generational
  refusals from (ii); delete R2(c)'s "unstated failure edge" justification.
- Stack exhaustion and other implicitly consumed resources given an owner
  (R13 unconditional default bound, R12 typed outcome, or R14 exclusion
  with a defined stop), and M2(d) saying which.
- M5 restated as acceptance-independence for erasable constructs; R14
  states the reference of a construct at a weaker level is the set its
  level admits; HIS-D0-16 restated to "one specified behavior set".
- R11 given a domain test (runtime divisor, narrowing cast, index against
  runtime length); the progress lemma moved to M6; "chosen in its name"
  replaced by "chosen per operation site".
- R14 reduced to the enumerated assumption set; the `foreign` qualifier's
  spelling and the debugger's admission moved to open questions.
- M10 given a degree per family and a project-scale criterion; the cone
  clause restated as a per-edit recheck bound without naming the algorithm.
- M6(i)(ii) required in the runnable form (bounded small-model check per
  discriminating program with prior criteria), the theorem a later
  obligation.
- P11–P14 written into discriminating-programs.md in the same change, or
  the cells marked pending.
- M7 and M9 tests restated as rule properties checkable without a writer
  model (non-empty `mechanical_fix`/missing-fact payload naming a tree
  location); writer trials cited as evidence only.
- R9's concurrent clause reduced to the shape admission; "costs only at the
  write" moved to M9 as a measured P10 property.
- R13 scoped to resources with a source-level cost semantics; deadlines and
  machine-stack bytes deferred until R14 carries a timing model;
  termination-for-removal kept inside R4's soundness clause.
- One disposition line per silent member (S14 of A-soundness, 26 of A-cost).

## 4. Disagreements the owner should decide personally

These are places where the drafts take opposite positions on grounds that
are the owner's to weigh, or where all three drafts silently reverse a live
ruling. A synthesizer should not pick.

1. **R4's standing.** B reverses `design/log.md:168` ("what the compiler
   hands LLVM is an implementation detail the language never sees") and
   makes backend facts a language requirement with IR tests; A keeps a
   soundness clause and reclassifies the channel as mechanism; C adopts the
   ruling and then tests on an IR count. Whether alias facts are a language
   requirement, and whether the backend may add facts by its own analysis
   (RAD-D0-25), is an owner ruling already on the tree.
2. **R4 and R5: two rows or one.** A and B keep them separate (opposite
   exactness directions); C merges them on M5's erasability. The critiques
   of C show the merge fails at spawn/join; the owner should say whether
   P10-style constructs are in scope of "a missing fact costs speed".
3. **Admission policy for backend-facing rows.** B admits layout, memory
   ordering, address spaces, erasure, write-once, loans and allocator
   contracts now (THE-D0-06 in substance); A merges them into R8/R14; C
   refuses them until a discriminating program fails. This decides whether
   the requirement list is a systems-language list or an ownership-subsystem
   list, and how much M8 budget it spends.
4. **Determinism level and reductions.** B fixes a closed three-level menu
   in R5; A parameterizes the level by R14 with a proved law; C requires
   each construct to state a level and (by omission) forbids reductions.
   CAP-1 fixes one guarantee today; admitting a weaker level reopens it.
5. **M2's reading.** Literal "no compiler-inserted check or branch" (B, C's
   test) versus "no unstated failure edge" (A) versus "no unchosen cost".
   The reading decides whether loop versioning and guarded re-derivation
   are permitted, which the scoped-alias-channel measurement makes a
   performance question, and whether Vale/Mezzo/CHERI stay excluded by rule
   or by floor measurement.
6. **Allocation failure.** All three drafts make allocation failure a typed
   outcome on a source-visible path, reversing SCOPE-3's temporary carve-out
   (heap exhaustion may stop execution with no value or cleanup). None says
   so. This is a whole-corpus shape change and a `GFP_*`-versus-abort fork.
7. **Stack exhaustion.** No draft owns it; the consistent readings are a
   mandatory stack bound for every program, a typed outcome, or an excluded
   exhaustion with a defined stop. Each is a language decision.
8. **The trusted adapter at the host boundary.** M3's per-entry boundary
   obligation distinguishes a definition by host crossing, against the
   constitution's "no declaration, type, ownership, effect, proof,
   diagnostic or semantic rule distinguishes"; all three drafts leave the
   contradiction (THE-REQ-14) unresolved. Either the adapter is an
   enumerated exception or the constitutional clause is amended.
9. **Global mutable state.** B lifts the live ban (`design/language/ownership.md:5`)
   as "one R9 instance"; A and C are silent. The ban's recorded ground is
   parallel permission itself.
10. **Writer-model trials as acceptance tests.** A (M7, M9) and C (M5, M6,
    every rounds price) use blind-writer runs as acceptance criteria;
    `design/language.md` decision 5 says such a trial describes the model
    and the assistance, not the language. Keeping them as evidence only
    leaves M7 and the floor with checker-side tests alone.
11. **Form and timing.** A requires an operational semantics and an adequacy
    theorem (M6) before a candidate is selected; B and C select on programs
    and measurements and defer the theorem to the soundness plan. This is
    the cost-of-D13 decision and it is the owner's priority call.
12. **PROG-1: spent or unspent.** A says spent by M3/M6; B says spent by the
    backend and unspent by the checker; C says unspent. Both C critiques and
    B-soundness show the checker spends it (R7's declaration-side clause,
    R5's exact transitive closure, R16 instantiation). The owner should
    state which premise is load-bearing, since every "infer, print, pin"
    option in later decisions depends on the answer.
13. **Authority separation on contracts.** B adopts non-vacuity (no discharge
    by weakening); A and C refuse it as uncheckable. C-soundness F5 names a
    checkable half: an owner-fixed interface contract that writer edits may
    not weaken. Whether that clause enters M3 decides how much of
    "reducing dependence on repeated human inspection" the list owns.

