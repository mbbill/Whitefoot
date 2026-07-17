# Systems-Performance Coverage — Consolidated Design Dossier

Status: RESEARCH SYNTHESIS for owner review. This document assembles the whole
systems-performance-coverage design in one place so it can be read deliberately
and the five pending decisions taken. It authorizes nothing: every production
language, specification, checker, compiler, runtime, standard-library, and
teaching change remains separately gated (D15). All paths below are relative to
`optimizer-language-research/implementation/systems-performance-coverage/` unless
noted. Numbers are indicative research measurements, not deploy-target results;
their limits are stated in §7.

---

## 1. The goal and the shape of the answer

**The objective (D15, `../../notes/user-directives.md`).** xlang must be usable
for general systems programming at top-tier performance. It does NOT need to let
you rewrite existing software line by line. It must provide, for MOST
systems-programming scenarios, at least ONE blessed way of writing whose
performance reaches or exceeds the best existing implementation. The owner's
calibration: if a single container forced everywhere were optimal in every
scenario, that would be a correct answer; since that is impossible, the language
offers `n` forms, and `n` must stay SMALL — because the second binding
requirement is that the specification stay compact enough to fit in a model
context window. So the target is an elegant, concise way to express most systems
demands at high performance, under the unchanged safety theorems (no
writer-accessible unsafe, proof-only check elision, checker feasibility, AI
writability).

**The shape of the answer: a three-tier architecture**
(`DESIGN-COMPARISON-AND-RECOMMENDATION.md`). The design that survived adversarial
comparison places every capability in exactly one of three tiers:

1. **A narrow language core** — checker/compiler rules only (ownership, regions,
   effects, the loan/freeze judgment, brands, capability predicates). Not
   library-implementable by definition; it is the language itself.
2. **Sealed built-in parts** — a small set of trusted-internals kernels whose
   implementations users cannot reach (uninitialized typed storage, atomics,
   syscalls, work-stealing). This is the same idea as Rust's lang-items /
   `core`-intrinsics or Go's runtime: the compiler blesses a fixed set of
   internals that ordinary code cannot write but can call.
3. **Taught checked-source libraries and composition cards** — zero-trust code
   the project writes and ships, plus documented patterns ("cards") that compose
   the tiers below. These stay INSIDE the safety proof.

**The sorting principle (D16).** A capability is sealed ONLY when ordinary users
cannot reach par performance from the language primitives and checked libraries.
Everything user-implementable-at-par ships as taught checked source or a card,
not as a new form. The owner's words: provide the most basic and critical
methods; let users implement the rest; only seal what users cannot build. This
is why the catalog is deliberately lean and does not mirror the Rust standard
library.

**The admission endgame (D17).** Sealing is not permanent. Any sealed kernel
whose implementation is machine-proved leaves the trusted list and becomes
checked code with privileged representation rights (uninitialized storage,
elided checks). The same lane is open, long-term, to USERS: performance never
requires proof (the default is composing the sealed catalog), but any user
implementation that machine-proves its invariants earns the same rights. Proof
substitutes for trust; the trusted list only ever holds project kernels not yet
proved. Near-term realistic candidates are sequential representation invariants
(tables, sequences, pools); concurrent lock-free protocols stay sealed until the
proof technology matures. This is the constitution's "speed is earned by proof,
never by weakening a check" applied to representation rights.

**How the answer was derived.** A fresh pass (no prior candidate framing as
input) produced a 9-family, 51-scenario demand map (`SCENARIO-DEMAND-MAP.md`);
four independent designers each produced a complete catalog under a different
stance (built-in maximalist, minimal semantic core, evidence-split hybrid,
parameterized schemas); each design took three independent hostile attacks; one
judge compared all four against the D15 objective. The evidence-split **hybrid**
won (conditional); the **schema** stance was runner-up and contributed its
parameterized packaging. The comparison and the decisive shared finding are in
§3 below.

---

## 2. The ten sealed parts

Source: `CATALOG-V1-RECUT.md` (the D16 minimality re-cut, which reduced the
pass-1 recommendation from ~18 forms to ten sealed kernels) and
`MEMBER-AUDIT-THREADS-IO.md` (the member-level audit of the two largest pockets).
Accounting: 10 sealed kernels, 17 spec objects, ~85-100 sealed op rows,
~15-24k trusted lines of code (down from ~130-160 rows / ~25-35k LOC pre-cut).
Honest caveat carried from the re-cut: the residual trusted library still rivals
the 90-rule kernel spec in size.

Each part below: what it is, why it must be sealed (the named blocker), the
performance mechanism it preserves, and its acceptance-ledger status.

1. **`seq<T, inline N>` — growable sequence.** Uninitialized spare capacity,
   affine elements, take/replace/swap/insert-at/remove-at/drain-range; inline
   (SmallVec) mode kept and pinned embeddable in user structs (including pool
   slots). *Blocker:* typed uninitialized affine slots and take/replace out of
   containers are floor-forbidden; an Option-tag or default-init user route
   loses Vec parity mechanically (a tag+branch on the hot path, or an O(n)
   reserve). *Status:* the inline-keep resolves a FATAL contradiction (btree's
   node representation and xlc both need embeddable inline seq); gates are an
   embeddability demo and a tiny-vector xlc-shaped bench within 1.10x of
   SmallVec.

2. **`table<K,V>` — SwissTable hash map.** SIMD group probe, one-load-one-compare
   hit path; inline tiny-map knob removed (a card covers n<=8). *Blocker:*
   control bytes are user-buildable, but the uninitialized affine K/V slot array
   with take/replace on probe hit is not. *Status:* the M3 dry run validated the
   probe/iterate shape (§4); the tiny-map card must stay within 1.05x of an
   inline-mode reference or the knob is re-admitted.

3. **`pool<T>` — generational slab.** insert/take/get with generation-checked
   handles, plus one charged row: a checked disjoint two-slot `&uniq` borrow.
   *Blocker:* reusable uninitialized typed slots and generation witnesses are
   runtime disciplines the static checker cannot express. *Status:* the two-slot
   borrow is promoted from optional to required (btree rebalancing needs it to
   avoid handle reminting that would invalidate the sibling leaf chain); it must
   pass hostile review as new sealed surface.

4. **`arena<'a>` + generative branded id — bump allocator.** Typed emplacement
   of affine values into region memory, pointer-bump alloc, O(1) bulk free;
   brand/region integrated as one spec cluster. *Blocker:* typed emplacement
   into raw memory is the forbidden op; pool does not subsume it (freelist pop +
   generation write is ~3-5x the 2-3-instruction bump path and loses O(1) bulk
   free). *Status:* the per-arena-value brand (which lets deref skip the
   which-arena check) is the subject of kernel Delta 5 / BRAND-1 (§3); alloc
   fast-path must inspect to 2-3 instructions. *M4 dry run (§4):* the branded-id
   deref is asm-confirmed check-free (no which-arena or generation compare/branch)
   and a retained bounds check would cost ~0% in a pointer chase (latency-hidden);
   parity with C holds for memory-resident walks, with index-based deref ~1.6x a
   RAW-pointer arena for L1-resident hot walks (index arithmetic, orthogonal to
   checking; an index-based C arena matches it).

5. **threads + par — scoped spawn, parallel-for, work-stealing scheduler.** The
   only thread-spawning and task-scheduling surface; the Chase-Lev steal deque is
   an internal detail (the user endpoint is demoted). *Blocker:* no user spawn
   and no atomics on the floor — categorically closed. *Status:* member-audited
   (`MEMBER-AUDIT-THREADS-IO.md`): scoped spawn/join with child-trap = process
   abort, join-before-return; the kernel-rule layer is Delta 1 / CONC (§3). Needs
   its five-leg acceptance battery and an irregular fork-join bench within 1.10x
   of a crossbeam-deque hand scheduler.

6. **sync vocabulary — relaxed counter, publish/snapshot cell, `mutex<T>`,
   `once<T>`, condvar.** Five members, one rule each; cas-cell cut. *Blocker:* no
   atomic type or memory model on the floor. *Status:* the counter's "no
   publication" rule is normative; the cell's payload is pinned to Copy words
   (disjoint from `shared<T>`); `mutex<T>`'s guard is the subject of kernel
   Delta 1 / CONC-3 (§3). Condvar's keep is evidenced by the WAL leader-follower
   group commit.

7. **`conc_queue<T, {spsc, mpmc}, cap=2^k>` — bounded concurrent queue.**
   Crossbeam-channel-class endpoints; the steal endpoint is removed (internal to
   par); spsc kept conditionally on a necessity sweep. *Blocker:* lock-free queue
   protocols need atomics; affine endpoint tokens make SPSC zero-RMW typed — a
   genuine WIN. *Status:* the M6 dry run model-checked the SPSC protocol and
   confirmed zero-RMW in assembly (§4); endpoints ride kernel Deltas 5 (BRAND-1)
   and 1 (CONC).

8. **`shared<T>` — epoch-pinned publish/snapshot.** ~4 rows (new / publish /
   snapshot / guard-drop); readers write zero shared cache lines; reclamation
   after a grace period; the update-with row is cut. *Blocker:* per-thread epoch
   slots, fences, and grace-period reclamation need raw atomics; a refcount
   substitute is exactly the shared-line write that kills read scaling. *Status:*
   gated on the M7 read-mostly bench (>=12x read scaling at 16 cores), run only
   after the table clone constant was measured (§4).

9. **io-file — enumerated syscall leaves.** open (O_DIRECT / alignment),
   pread/pwrite, read/write, fdatasync, fsync, dir-fsync, rename, stat/len,
   block-size query, close; every row with a named consumer; evring deferred out
   of v1 (`IO-ROW-ENUMERATION.md`). *Blocker:* syscalls are sealed by definition.
   *Status:* the family is now row-enumerated with per-row scenario evidence; the
   durability rows carry the macOS F_FULLFSYNC pin (§4) and must be verified on
   both platforms before WAL ratification.

10. **extern-C records — C-ABI layout-guaranteed records.** Minimal row set.
    *Blocker:* ABI layout control is a codegen guarantee, not expressible in
    checked source. *Status:* the v1 consumer is named (differential-testing and
    benchmark harnesses linking pinned C references, plus sealed-io shims); only
    the rows those harnesses use ship in v1.

Everything else the pass-1 catalog proposed was pushed down a tier: btree, the
sort core, WAL, the layout modules, and bytescan/hash kernels are **checked
libraries**; LRU/FIFO ring, tiny-map, COW-republish map, sharded map,
sort-by-key, validated-view, and durability-ordering are **cards**; rcu_table,
evring, the sealed btree, cas-cell, seqlock-cell, tbuf, and others are
**deferred with reinstatement triggers**. The full cut log and trigger list are
in `CATALOG-V1-RECUT.md` §"Cut log" and §"Deferred list."

---

## 3. The kernel-rule layer: the ratified loan judgment + six drafted deltas

This is the language-core work. Note on counting: the milestone brief calls this
"six kernel deltas," but there are seven rule sets — the **loan/freeze judgment**
is a separate, already-ratified artifact (the gate-#1 mechanism, `m1-loan-
judgment/RULES-RATIFIED.md`), and the six numbered **kernel deltas** are drafts
in `m2-spec-mass/KERNEL-DELTAS-DRAFT.md`. All seven are covered below.

### 3.0 The loan/freeze judgment (15 rules — RATIFIED)

*What it does.* A per-binding lexical loan/freeze judgment plus confined
borrow-carrying values: a region-parameterized affine struct/enum may hold
borrows, is stack-confined (never stored in heap data), and is consumed like a
unique borrow at calls; a live token/cursor/guard minted from container C freezes
C until the token is consumed or its region ends. Issue/consume pairs are
declared in the form table, not inferred.

*Why it is needed.* This is the decisive cross-cutting finding (§ below): the
frozen ownership rules could not type resumable access tokens (hash-table entry
tokens, B-tree cursors, mutex guards, iterators, I/O views) — every design either
silently assumed the mechanism or invented it unpriced. Without it, no stance
survives and the fallback is state-threading everywhere at a large writability
tax.

*Review history.* M1 produced the rules and a machine-checked reference checker
(48/48 on the canonical corpus). Hostile review (`evidence/m1-hostile-review.json`)
found a FATAL — the checker accepted a `par` statement handing parallel bodies a
unique write-view and a shared view of the same place — which drove the AMD-1
statement-local mint-disjointness amendment; four more amendments (AMD-2..5)
followed. D18 ratified the consolidated 15-rule set. Current state (per the
repo's `AGENTS.md` status line and `RULES-RATIFIED.md`): exactly 15 rules,
machine-verified on a 97-program corpus with a 9/9 mutation-caught harness.
**Review-clean / ratified.**

### 3.1 Delta 6 — TAG-1 (tag parameters)

*What.* Closed-set lowercase tags (`h` in `table<K,V,h>` for the hasher, `ep` in
`conc_queue<T,ep,K>` for the endpoint discipline) are written in the type-argument
position, but the grammar's `targ` production only admits uppercase type names,
regions, and consts — so every `table<..., fold>` and `conc_queue<T, spsc, K>` in
the catalog is, as written, ungrammatical. TAG-1 adds a tag parameter sort and a
tag targ, a compile-time-erased monomorphization selector. *Why.* A latent
grammar gap the catalog assumed away; nothing spells a hasher or endpoint
discipline without it. *Review:* light (no fact channel, no loan, no runtime
representation); it gates nothing but is a prerequisite for the catalog parsing at
all. **Draft, review-light.**

### 3.2 Delta 2 — `tbl_clone` (whole-table clone)

*What.* A deep-copy whole-table row for `table`, K and V required Copy (the
enforceable predicate; v0 has no Clone contract). *Why.* Bulk control-byte + slot
copy at DRAM bandwidth (~1-2 ns/entry) is unreachable from a for-each-then-insert
rebuild (~10-40 ns/entry); the named consumer is the COW-republish card, where
the clone IS the publish. *Review:* light (no fact channel, no loan). One FATAL-
adjacent correction in review: an owning-value (heap V) clone is BLOCKED on a
separate Clone-contract delta and excluded here (a shallow "share the values"
fallback would be a double-free). **Draft, review-light.**

### 3.3 Delta 3 — LOAD-1/2 (machine-core byte loads)

*What.* Endian-explicit byte-load intrinsics (`load_le_u32`, etc.) reading K/8
bytes at an offset out of a borrowed byte slice. *Why.* Byte-assembly by hand is
4-8 instructions per field versus one load; consumers are validated-view and
serialization scenarios. *Load-bearing FATAL found and closed:* the out-of-range
trap was first spelled `off + K/8 > len(s)`, which WRAPS at the u64 edge and would
admit an out-of-bounds read on an attacker-influenced near-max offset even with
elision off. Fixed to the non-wrapping form `len(s) >= K/8 AND off <= len(s) -
K/8`, with align-1 lowering pinned and both x86-64 and arm64 asm-diffs in the
codegen corpus (`[DELTA-FIX-2]`). **Draft, review-light after the fix.**

### 3.4 Delta 4 — DOM-1 (length-dominates-bounds facts)

*What.* Two checked-fact forms that let the requires engine discharge bounds
traps: pow2-mask domination (a masked index into a pow2-length backing is in
bounds) and len-check domination (a checked prefix window). *Why.* Without them
every masked-ring access and validated-view field keeps a runtime bounds branch.
*Load-bearing FATALs found and closed (two):* (a) a producer computing `off + n`
by wrapping add could forge an unbounded in-bounds window — fixed by pinning the
accepted producers to non-wrapping adds (`iadd.trap` / `iadd.checked`-Ok /
`iadd.sat`) and rejecting `iadd.wrap` (`[DELTA-FIX-1]`); (b) a cross-call resize
could invalidate a formed fact into a use-after-free — fixed by a non-resizable-
backing formation condition (`buffer`/`slice`/`array` only) plus a stated
cross-call length-invalidation havoc rule (`[DELTA-FIX-3]`). *Status:* this is a
fact channel that elides a safety check — the exact class the standing rule
requires be re-attacked before shipping. **Draft, fact-channel, re-attack owed.**

### 3.5 Delta 5 — BRAND-1 (brand-cross-fn)

*What.* Makes brands (arena ids, queue endpoint identities) first-class nominal
type constituents so `TYPE-5` exact-match does identity checking at every boundary
(call, return, construction, container element, destructure). Separates generative
brands (queue endpoints, a fresh existential mint) from lexical brands (arena ids,
keyed on the arena's resolved place), with an all-carriers tie rule and an AMD-6
by-declaration split between loan tokens and affine brand-carriers. *Why.* The
writability trials' entry/pipeline failures (§4) traced to brand-cross-fn being
unspellable: you could not move a queue endpoint into a helper `fn`. *Review
history — three full loops.* v1 was rejected by two hostile reviews for unstated
foundations; v2 redrafted foundations-first; v2.1 and v2.2 closed a composition
regression (arena_get consuming affine ids) and named the transitive-carrier
forms. The v2.2 closing regression returned zero findings
(`evidence/brand1v22-close.json`: "CLOSE"). **Review-clean / closed.**

### 3.6 Delta 1 — Concurrency (CONC-0..4)

*What.* The concurrency bindings: scoped threads (`scope`/`spawn` with persistent
per-capture loan entries and join-on-every-exit-edge), a static Send/Share
capability judgment, a `mutex<T>` sealed form with an interior-view loan clause,
and child-trap = whole-process abort — all built on **CONC-0**, a proposed kernel
memory model (MM-0..MM-10) that gives D1 ("data-race impossibility") a checkable
reduction target it never had. *Why.* The sealed concurrency parts (threads+par,
mutex, conc_queue, shared<T>) all rest on cross-thread trust that was asserted,
not modeled. *Review history — the heaviest, four rounds.* The v2 draft was ruled
NOT ADOPTABLE by three reviews: it was built on a memory model the kernel does not
contain, and its checker-side rules admitted **three data races** (spawn-capture
loans evaporating at statement end; a vacuous container-Shareable column letting
two threads reach a never-Shareable endpoint; a guard interior view escaping past
unlock). v3 added CONC-0 and closed two of the three; the v3 re-attack confirmed
both and left one FATAL (the guard's free result region) plus two MAJORs
(memory-model circularity; a scope-effect-row false fact). v3.1 closed the last
FATAL by mirroring `arena_get` (tie the interior view to the receiver's own
region, no caller-chosen region), made CONC-0 a non-circular conditional reduction
target, and pinned the effect-row root region. The v3.1 closing regression
(`evidence/conc-v31-close.json`) returned "CLOSE (research-draft close, NOT
production) ... D1 is upheld." v3.2 cleared the remaining fail-closed residue
(the FN-7/`sync`-atom reconciliation, the guard entry-place wording, the
resolved-root region, one canonical `sync`-op count, and the borrow-held-entry
obligation pin). **Draft-closed at the review level, pending five owner decisions
(§5).** No data-race finding remains against the text, but CONC-0 is a proposed
kernel addition, not proven — its per-form memory-model proofs are the model phase
that follows adoption (§6, §7).

### The decisive cross-cutting finding (why 3.0 gates everything)

All four independently-attacked designs failed on the SAME kernel gap: the frozen
ownership rules (no reborrow, no borrows stored in data, unique borrows consumed
by calls) cannot type resumable access tokens — entry tokens, cursors, guards,
iterators, I/O views, or any helper that does more than one operation on a
container it borrows. The loan/freeze judgment (3.0) is the single new mechanism
that closes it, and its ratification (M1) is what unblocked the rest.

---

## 4. The evidence

All indicative research measurements on an Apple M4 (macOS arm64) unless noted;
the deploy target is Linux x86-64 and these validate SHAPE, not deploy magnitudes
(§7). Sources in `evidence/` and the `m3a-`/`m6a-` dry-run directories.

**Kernel-shape parity — seq and table (M3, `m3a-kernel-dryrun/RESULTS.md`).**
C kernels matching the catalog's pinned shapes vs Rust `Vec` and
`hashbrown 0.17.1`+`foldhash`, band <=1.25x = shape validated:

| benchmark | ratio (C / Rust) | band |
|---|---|---|
| seq push-then-sum | 0.67x | OK |
| table build (1M insert) | 1.18x | OK |
| table hit-lookup | 1.07x | OK |
| table miss-lookup | 1.26x | edge (1.22-1.28x across runs) |
| table iterate-sum | 0.71x | OK |

Differential correctness vs a std HashMap oracle: zero divergence over 3 seeds x
1M mixed ops. The one-16B-load-per-group probe shape was confirmed in
disassembly. A genuine catalog GAP was found: the K/V physical layout was
unspecified, and struct-of-arrays doubles insert cache misses (measured 2.0x) —
array-of-structs `(K,V)` slots must be pinned. Push parity needs the
guaranteed-inline lowering (a naive out-of-line grow is 1.6x).

**SPSC model check + zero-RMW (M6, `m6a-spsc-dryrun/RESULTS.md`).** An exhaustive
small-bound model check of the SPSC protocol under a release/acquire operational
semantics: the correct configuration is SAFE (249 states, 12 good terminals, 0
bad, 0 deadlock), and all four acquire/release halves are load-bearing —
weakening any one is caught (the data-less head/WAR pair included). The C
implementation vs `rtrb 0.3.4`: batched-32 throughput ~1050 M/s = **PASS** (13-16x
the >=80M/s band); round-trip latency ~31 ns/way is OVER the 6-15 ns band but is
platform-bound (rtrb also lands ~39 ns on this host; the C impl beats it, and
~30 ns is the M4 unpinned cross-core floor). The WIN is confirmed in assembly:
**zero RMW** on the steady-state path — one release store to publish, one acquire
load amortized below one-per-op by cursor caching. 50M items received strictly
FIFO.

**Clone-cost regimes (`evidence/microbench/RESULTS.md`).** The audit had assumed
~50-100 ns/entry; the measurement shows it depends entirely on the value kind:

| entry kind | ns/entry | mechanism |
|---|---|---|
| POD `u64->u64` | ~1-2 | bandwidth-bound bulk copy (~15-25 GB/s) |
| String 24B | ~9-32 | + one malloc + payload memcpy per entry |
| String 200B | ~36-249 | + larger memcpy, page faults at scale |
| `Vec<u8>` 4KB | ~363-3063 | allocation- and fault-dominated |

So the old assumption is 25-100x too high for POD and 3-30x too low for KB-scale
values. Since the clone cost IS the publish cost, the COW-republish card is viable
for POD/handle tables (~1-2 ms per 1M entries) and non-viable for KB-value tables
(~3 s per 1M entries); no single threshold is honest.

**macOS durability and drain findings.** (a) Plain `fsync`/`fdatasync` on macOS
return before the drive cache reaches stable media; the durable spelling is
`fcntl(fd, F_FULLFSYNC)`, which io-file rows 7/8/9 must lower to — until pinned,
no macOS-dev WAL-durability test counts (`IO-ROW-ENUMERATION.md` §durability pin).
(b) macOS does NOT honor listener `SO_RCVTIMEO`: the member audit's timed-accept
graceful-drain mechanism is broken on Darwin (accept blocked past the timeout);
per-connection `SO_RCVTIMEO` works, so the required Darwin path is the
self-connect / kqueue-drain fallback (`evidence/microbench/RESULTS.md` §B).

**Writability trials (M5, `evidence/m5-*.json`).** A mid-tier model, spec-only,
solving preregistered tasks across five areas (entry, lru, sweep, pipeline, seq).
Band: >=70% first-green AND >=80% of green solutions at par:

| round | green | of-green at par |
|---|---|---|
| trial 1 | 9/20 | 9/9 |
| rerun | 5/19 | 5/5 |
| round 3 | 5/19 | 4/5 |
| round 4 | 8/20 first -> 14/20 final | 14/14 |

The par-of-green signal is consistently strong (when a solution is green it is
essentially always at par). The green RATE is the writability signal, and it was
gated by the entry and pipeline areas, which stayed near-zero until brand-cross-fn
became spellable — this is exactly what drove BRAND-1 (Delta 5) and the loan
tokens. **Honest contamination note:** these rounds are NOT independent — each
round's failures fed spec fixes (the `M5-FIX`/`M5R2`/`M5R3`/`M5R4` families) that
the next round then benefited from, so round 4's 14/20 reflects a co-evolved spec,
not a clean single-shot trial. The trials are diagnostic (they located the
unspellable idioms) more than confirmatory.

**Spec-mass count (M2, `evidence/m2-spec-mass-count.json`).** Measured with
tiktoken cl100k_base: the two sampled catalog artifacts total 14,360 tokens
(optables 9,194 + cards 5,166); extrapolating the full normative appendix gives
~65k tokens against the <=40k-token band — **verdict FAIL**. This measurement is
itself the result: it forced the reduction levers (`m2-spec-mass/reduced-optables.md`,
`m2-spec-mass/reduced-cards-A.md` and `-B.md`) and the D18 ruling-5 split (eight load-bearing cards kept
with full examples, seven demoted to non-normative example files, targeting a
~48k always-loaded manual). The concurrency delta's own normative-only cut is
still owed (§5, decision 5).

---

## 5. The five owner decisions

Each: what it is in plain terms, the options, the trade-off, and a recommendation.

### Decision 1 — Land the concurrency memory model (CONC-0) as kernel text

*What.* Today D1 ("no data races") is stated as a law but the kernel defines no
data race, no happens-before, and no meaning for release/acquire. CONC-0
(MM-0..MM-10) is the minimal happens-before statement that makes D1 CHECKABLE — a
definition of race, the synchronization edges each sealed form must establish, and
a new `sync` effect atom. Every concurrency proof reduces to it.

*Options.* (a) Adopt CONC-0 into the kernel and let the concurrency layer proceed;
(b) leave D1 asserted and ship no concurrency (the sealed threads/mutex/queue
parts cannot be made sound without it); (c) adopt a different/smaller model.

*Trade-off.* CONC-0 is real new kernel foundation (~11 clauses + a new effect
atom + target-mapping proofs), and it is PROPOSED, not proven — its per-form
discharge is the model phase (§6). But it is minimal and correctly scoped, and
without it the entire concurrency tier (four of the ten sealed parts) has no
soundness basis.

*Recommendation.* **Adopt (a).** The concurrency review converged on CONC-0 as
the necessary and sufficient foundation; three review rounds could spell no
data race against the resulting text. The cost is unavoidable if xlang is to have
safe concurrency at all, and it is smaller than any alternative that was tried.

### Decision 2 — Loop-spawn: fixed vs dynamic fan-out

*What.* `OWN-11` forbids naming an outer region inside a loop body, so a `spawn`
inside a loop cannot capture an outer `&'p` borrow. As drafted honestly, the
concurrency delta therefore admits only fixed (straight-line) fan-out that shares
outer state; runtime-count N-worker data-parallel fan-out is not spellable.

*Options.* (a) Accept fixed fan-out for v1 (the honest restriction as drafted);
(b) authorize an `OWN-11` capture carve-out now (sound under the new persistent
capture-loan entries, which enforce cross-iteration disjointness, but it amends a
ratified region rule).

*Trade-off.* Option (a) is safe and needs no rule change, but it means the
delta's own D16 admission story (which names N-worker scenarios 11/38/41/42/44)
is not fully covered — a real expressiveness gap. Option (b) covers those
scenarios but amends a ratified rule and adds review surface.

*Recommendation.* **Lean (b), gated on the carve-out passing its own hostile
review** — the named consumers are exactly the data-parallel shapes the goal
targets, and the carve-out is sound under machinery already being adopted. If the
review timeline is tight, ship (a) for v1 and schedule (b), documenting the gap
rather than hiding it. This is the sharpest coverage-vs-authority call.

### Decision 3 — Ratify AMD-5-carve-out / AMD-7 / AMD-8

*What.* The concurrency delta amends three ratified rules: **AMD-7** adds a
capability premise to `par`/`spawn` slots (a replicate slot needs a Shareable
referent) and marks the R14 sync-exemption; **AMD-8** replaces R15's per-invocation
obligation with a concurrency schema (process-wide tenure exclusion, publication,
reclamation) attached by the `sync` marker; the **AMD-5 carve-out** lets `mutex`'s
`guard_uniq` issue a unique interior view off the guard's own token (AMD-5 as
ratified declaration-rejects the naive form).

*Options.* (a) Ratify all three via the D18 path; (b) ratify a subset; (c)
reject and redesign.

*Trade-off.* Each is load-bearing: without AMD-7, the Send/Share capability check
has no home on par slots; without AMD-8, R15 cannot carry mutex exclusivity (a
per-invocation predicate a conforming recursive lock satisfies while racing);
without the AMD-5 carve-out, the mutex has no callable interior-access op. All
three were adversarially reviewed to closure.

*Recommendation.* **Ratify all three (a).** They are the minimal amendments that
make the concurrency mechanisms sound, and each was closed against hostile
re-attack. Ratifying a subset leaves a mechanism unspellable.

### Decision 4 — Clone-row re-mode (a landed-catalog behavior change)

*What.* CQ-3 says "every endpoint op takes `&uniq`," but the catalog's
`cq_tx_clone`/`cq_rx_clone` rows take `&` receivers — which would let two threads
reach a shared endpoint and clone concurrently on unsynchronized state. The fix
re-modes both clone rows to `&uniq` receivers, restoring the invariant and making
a concurrent clone unspellable by construction.

*Options.* (a) Re-mode to `&uniq` receivers (smaller trusted surface); (b) keep
`&` and add both rows to the `sync` obligation net with a restored effect home.

*Trade-off.* Re-mode changes the ergonomics of a landed catalog row (a clone now
needs unique access to the endpoint briefly) but closes the race by construction;
the alternative preserves ergonomics but grows the trusted concurrent-invocation
obligation.

*Recommendation.* **Adopt (a).** It restores CQ-3's stated invariant literally and
is the smaller trusted surface; the ergonomic cost (clones happen at setup, not on
the hot path) is negligible.

### Decision 5 — The concurrency spec-mass cut

*What.* The honest kernel-rule count for the concurrency delta is ~11 rules plus a
full mutex form-table section, not the ~4 the draft's numbering implied; verbatim
adoption likely busts the ~48k always-loaded budget (M2 already measured the base
catalog at ~78-90% of it).

*Options.* (a) Take a normative-only cut of the concurrency delta (rule text +
form rows in the manual; rationale and regression walks moved to the review
record) with a measured token count before spec; (b) raise the budget; (c) defer
part of the concurrency surface.

*Trade-off.* The budget exists because the spec must fit a model context window
(D15's second binding requirement) — raising it weakens the core constraint.
Option (a) preserves the budget at the cost of a real editing pass.

*Recommendation.* **Adopt (a).** The compact-spec requirement is load-bearing to
the whole goal; the cut is mechanical editing (the content is written) and gives a
real token number to check against ~48k.

---

## 6. What remains after the decisions

The five decisions unblock the program; they do not finish it. What follows, in
dependency order:

1. **Gated real-compiler integration — the true performance milestone.** Every
   number in §4 is a C or Rust dry run validating SHAPE. The actual par claim is
   only earned when the sealed kernels are built behind `xlc` with the pinned
   codegen contracts and measured on the Linux x86-64 deploy target. This is the
   milestone the dry runs de-risk, not replace.
2. **Per-part five-leg acceptance batteries (D16 ledger).** Each sealed part must
   prove performance AND safety AND reliability before shipping: differential
   testing against the reference, exhaustive small-bound model checking,
   sanitizer/fuzz soak, hostile pre-ship review, and CI-pinned perf/asm shape —
   plus complete failure semantics, resource ceilings, and teardown with
   fault-injection evidence. The two largest pockets (threads+par scheduler,
   residual io family) have member audits done but not their full batteries; the
   MM-9 abort-mid-scenario fault-injection battery (out-of-order async-io
   completion, not only torn-tail) is required before any WAL-durability claim.
3. **The per-form safety-model / model-checking phase for concurrency.** CONC-0's
   MM-1..MM-6 edges are OBLIGATIONS each sealed form must discharge via its fences
   (MM-10 lowering) — the discharge is proved per form in a model phase that
   follows adoption: MM-3 O1-O4 for the mutex, MM-4/MM-5 for every queue op at
   memory-event granularity, MM-6 reclamation counting. The borrow-held loan entry
   that `guard_uniq` introduces also needs its full R4/R7/R10(b) specification for
   borrow results (it can only over-reject, never race).
4. **Production spec drafting at the ratified budget.** With decision 5 taken, the
   normative kernel text and the <=~48k catalog appendix are drafted and counted
   for real, and the loan judgment plus the seven kernel deltas fold into the
   production spec through the separate landing review.
5. **The preregistered validation ladder, remaining rungs.** M1 (loan judgment)
   is done and ratified; M2 (spec mass) is measured (and drove the cut); M3
   (seq/table shape) and M6 (SPSC) have dry-run passes. Still open with frozen
   bands: **M4** (arena brands in <=5 rules or a checked-deref fallback <=1.25x C),
   **M7** (`shared<T>`/rcu_table >=12x read scaling at 16 cores), **M8**
   (checked-source SIMD memchr/utf8), **M9** (LZ4 checked-source codec bet), and
   **M10** (boundary-semantics fault injection). Full bands in
   `DESIGN-COMPARISON-AND-RECOMMENDATION.md` §8.

---

## 7. Honest limitations

- **Everything here is research draft, not production.** No production language,
  spec, checker, compiler, runtime, standard-library, or teaching change is
  authorized; the whole dossier is gated on the five decisions and a separate
  landing review (D15).
- **Graders, not compilers.** The writability trials (M5) were solved and scored
  by models against the spec, not compiled by `xlc`; the kernel-shape and SPSC
  numbers (M3/M6) are C/Rust dry runs of the pinned shapes, not `xlc` output. No
  measurement in this dossier came through the production toolchain.
- **Indicative numbers, not deploy-target results.** All measurements are on an
  Apple M4 / macOS arm64; the deploy target is Linux x86-64. They validate the
  SHAPE of the mechanisms (parity class, instruction pattern, model-check safety),
  not the magnitudes that will hold on the reference pin host. Two numbers are
  explicitly platform-bound: the SPSC latency band miss (~31 ns vs 6-15 ns) is the
  M4 cross-core floor, and the macOS durability/drain findings differ materially
  from Linux and must be re-verified on the deploy target.
- **The safety-model phase is real work not yet done.** The D16 five-leg batteries
  and the per-form concurrency discharge (§6.2, §6.3) are outstanding for most
  parts; a green ledger is a shipping precondition and does not yet exist for the
  sealed set.
- **The concurrency memory model is proposed, not proven.** CONC-0 makes D1
  checkable and no data race can be spelled against the current rule text, but the
  model itself is a proposed kernel addition whose per-form soundness proofs, the
  MM-10 target lowering proofs, and the arm64 fence audits are the model phase that
  follows adoption — not results in hand.
- **The trusted surface is large.** Even after the D16 cut, ~15-24k trusted LOC
  across ten kernels rivals the 90-rule kernel spec; the D17 proof-gated lane is
  the long-term path to shrinking it, but near-term the sealed set is trusted, not
  proved.

---

## Appendix — cross-reference index and consistency notes

**Primary sources.** Architecture: `DESIGN-COMPARISON-AND-RECOMMENDATION.md`,
`SCENARIO-DEMAND-MAP.md`. Catalog: `CATALOG-V1-RECUT.md`,
`MEMBER-AUDIT-THREADS-IO.md`, `IO-ROW-ENUMERATION.md`. Kernel rules:
`m1-loan-judgment/RULES-RATIFIED.md`, `m2-spec-mass/KERNEL-DELTAS-DRAFT.md`
(+ `optables.md`, `cards.md`, `HANDOUT.md`). Directives:
`../../notes/user-directives.md` (D15-D18). Evidence: `evidence/*.json`,
`m3a-kernel-dryrun/RESULTS.md`, `m6a-spsc-dryrun/RESULTS.md`,
`evidence/microbench/RESULTS.md`. All cross-references above resolve to files
present in the tree.

**Consistency notes (places where the underlying drafts differ — flagged as part
of the job).**

1. **Form count: 18 vs 10.** `DESIGN-COMPARISON` §5 recommends eighteen taught
   forms / fourteen families; `CATALOG-V1-RECUT` (the later D16 minimality
   re-cut, same date) reduces this to ten sealed kernels + checked libraries +
   cards. This is a documented recut, not a contradiction — the ten supersede the
   eighteen — but a reader comparing the two docs sees different numbers. This
   dossier uses the ten (§2) as current.
2. **"Six kernel deltas" vs seven rule sets.** The loan/freeze judgment
   (`RULES-RATIFIED.md`, 15 rules, ratified) is a separate artifact from the six
   drafted deltas in `KERNEL-DELTAS-DRAFT.md`. §3 covers all seven and labels the
   count explicitly.
3. **Concurrency coverage is partial relative to the sealed concurrency parts.**
   `CATALOG-V1-RECUT` lists four concurrency kernels (threads+par; sync vocabulary
   incl. mutex/condvar/once/counter/cell; conc_queue; shared<T>). The drafted
   concurrency delta (CONC-0..4 + BRAND-1) provides the kernel-rule layer for
   scoped threads, conc_queue endpoints, and `mutex` only; once/condvar/counter/
   cell/`shared<T>` ride the same CONC-0 memory model but are not yet separately
   drafted. The memory-model foundation is shared; the per-member rule text is
   not complete.
4. **The clone constant.** `CATALOG-V1-RECUT` carries an open flag assuming
   ~50-100 ns/entry; `evidence/microbench/RESULTS.md` measures POD at ~1-2 ns
   (25-100x lower) and KB-values up to ~3063 ns (far higher). The measurement
   resolves the flag but changes the COW-republish card's honest coverage framing
   (viable for POD, non-viable for KB values) — the card text must select its
   constant by value type.
5. **macOS timed-accept.** `MEMBER-AUDIT-THREADS-IO.md` specifies listener
   `SO_RCVTIMEO` timed-accept as the graceful-drain mechanism (flagged pending a
   battery check); the microbench shows macOS does not honor it, so the audit's
   stated mechanism is broken on Darwin and the self-connect/kqueue fallback is
   required. Audit and measurement now disagree; the measurement wins and the
   audit's mechanism needs re-scoping.
6. **The spec-mass budget band evolved.** `DESIGN-COMPARISON` §7 states a
   `<=40k-token catalog appendix`; M2 measured the extrapolation at ~65k (FAIL);
   D18 ruling 5 then set an always-loaded ~48k target via an eight-keep /
   seven-demote card split. The current working target is ~48k always-loaded, and
   the concurrency delta's normative-only cut against it is still owed (decision
   5).
