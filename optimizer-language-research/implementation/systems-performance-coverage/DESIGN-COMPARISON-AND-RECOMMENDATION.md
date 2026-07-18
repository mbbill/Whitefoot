# Systems-Performance Coverage Research: Design Comparison and Recommendation

Date: 2026-07-16

Status: research result under D15. It authorizes nothing by itself; every
production language, specification, checker, compiler, runtime, or teaching
change remains separately gated on explicit owner decisions.

## 1. Question and method

D15 restates the objective: for most systems-programming scenarios, at least
one blessed way of writing Whitefoot must reach or exceed the performance of the
best existing implementations; the provided form count n must stay small under
the binding spec-compactness requirement; line-by-line reproduction of
existing software is explicitly not required.

Method of this pass, run as a fresh derivation with no prior candidate
framing as input:

1. Three independent mappers produced a 9-family, 51-scenario demand map of
   what performance-critical systems software does and why the fast shapes
   are fast (`SCENARIO-DEMAND-MAP.md`).
2. Four independent designers produced complete catalogs under materially
   different stances: built-in-forms maximalist, minimal semantic core,
   evidence-split hybrid, and parameterized form schemas. Each design covers
   every scenario row, states its trusted surface, estimates its spec cost,
   and preregisters falsifiable experiments.
3. Each design received three independent hostile attacks (performance holes;
   safety and TCB; spec size, teachability, evolution) — twelve attack
   reports total.
4. One judge compared all four designs with their attack results against the
   D15 objective function.

Raw, unedited agent outputs are preserved in `evidence/`.

## 2. The four designs and how they fared

| Stance | Forms | Claimed coverage | Credible after attack | Claimed rules | Honest recount | FATALs | Verdict |
|---|---|---|---|---|---|---|---|
| builtin (30 sealed TCB forms, minimal language change) | 30 | 48/51 par-or-win | ~37-40/51 | +36 | ~5-8x more | 4 | reject as-is; salvage TCB verification battery, hidden epoch, perf pins |
| core (cells/sealed/erasure; zero trusted container internals) | 20 | 45/51 | ~33-36/51 | +50 | 79-124 rules | 5 | reject as architecture; salvage confined borrow-carrying values, copy struct, atomic vocabulary |
| hybrid (three-tier evidence split; 22 forms + 8 patterns) | 23 | 45/51 | ~42-45/51 post-repair (best) | +47 | ~250-350 effective incl. op tables | 4 (all with named additive repairs) | WINNER, conditional |
| schema (11 parameterized schemas + verified lowering templates) | 20 | 48/51 | ~41-44/51 post-repair | +48 | 3-4x payload undercount | 2 (fewest) | runner-up; merge packaging, in-template hot paths, proved fact sheets |

Design essences:

- **builtin**: keep the 90-rule kernel almost intact; add one normative form
  table of ~30 opaque built-in forms (containers, memory, byte kernels,
  concurrency, I/O) with TCB internals. Died on: two arena safety FATALs
  (Copy handles yielding two live `&uniq`, reset without re-branding), an
  iterator/entry story that requires exactly the loan tracking its checker
  story denies, a refuted closed "classify64" SIMD algebra (4-16x off on
  numeric/columnar loops), and a 5-8x spec undercount breaching its own
  preregistered fail line.
- **core**: one load-bearing idea — `cells<T>` storage whose per-slot
  Vacant/Live state is checked and then erased, plus `sealed` types whose
  representation invariant (a closed decidable predicate language, e.g.
  `ctrl[i]<0x80 <-> live(slots,i)`) is verified per method. Best
  checked-source parity story produced for Vec/SwissTable-class sequential
  kernels; died on: no shared-immutable-ownership story at all (read-mostly
  hole), mmap fact forgery (externally mutable bytes validating once),
  forgeable role tokens across instances, concurrency custody clauses that
  are trusted assertions without a memory model, and a rule recount 1.6-2.5x
  over its own kill bound.
- **hybrid**: a falsifiable three-tier placement rule — language primitive
  only when reused by >=3 forms with decidable client obligations; sealed
  built-in when par depends on a partial-initialization invariant or a
  paper-grade concurrency proof (implement-rarely/use-often); taught
  composition otherwise. All four of its FATALs have named additive repairs
  supplied by the attackers themselves; highest credible coverage post-repair.
- **schema**: the compiler knows ~11 form *generators* (schema = machine-
  checkable invariant + closed parameter/policy signature + verified lowering
  template); source instantiates schemas. Best abstraction economy and
  friendliest weak-writer surface (misparameterization is a compile error);
  fewest FATALs — both instances of the shared kernel gap below.

## 3. The decisive cross-cutting finding

All four designs, attacked independently, failed on the same kernel gap:

> The frozen ownership rules (no reborrow, no borrows stored in data, unique
> borrows consumed by calls) cannot type resumable access tokens: hash-table
> entry tokens, B-tree cursors, mutex guards, iterators, `fill_buf` I/O
> views, or any helper function that performs more than one operation on a
> container it borrows.

Every design either silently assumed such tokens (builtin, hybrid, schema) or
invented the mechanism without pricing it (core). The judge's recommendation
is therefore explicitly conditional on one new, honestly priced ownership
mechanism as gate #1:

- **Confined borrow-carrying values** (~5-6 rules): region-parameterized
  affine structs/enums that may hold borrows, stack-confined, never storable
  in heap data, consumed like unique borrows at calls.
- **Loan/freeze judgment** (~6-8 rules): a per-binding lexical loan set — a
  live token/cursor/guard minted from container C freezes C until the token
  is consumed or its region ends; issue/consume pairs are declared in the
  form table, not inferred.

If this cannot be specified decidably in <=15 syntax-directed rules, no
stance on the table survives, and the fallback is state-threaded operations
everywhere with a large writability tax. This interacts with the settled
no-reborrow decision (T-A) and must go through the design-tree re-decision
discipline if adopted; the greenlit-but-unimplemented OWN-6/OWN-14 relief
valves are adjacent evidence.

## 4. Recommended architecture

Adopt the hybrid three-tier placement rule as the governing architecture,
amended with the judge's mandatory corrections:

1. Ship the shared-snapshot/epoch read primitive (`shared<T>` publish/
   snapshot + `rcu_table`) in v1. The read-mostly hole was the widest FATAL
   across designs; sharded locks provably plateau 10-20x under Zipf read
   traffic.
2. Ship a portable lane-arithmetic SIMD value core with named intrinsics and
   one-time ISA dispatch. Closed special-purpose stage algebras are refuted.
   Ordinary writers call shipped kernels; kernel authorship is project-tier.
3. Promote codegen contracts to first-class CI-gated spec: designated
   guaranteed-inline hot ops, callee-in-place move ABI, an explicit iteration
   protocol (internal `for_each` with guaranteed monomorphized inlining as
   default), 1-load endian access, strict-select never-branch, zero-memset
   I/O read path. Every design's hot-loop numbers silently assumed these.
4. Generative per-arena-value brands (sibling-arena confusion rejected at
   compile time); fallback if over the frontend-simple budget: an always-on
   elidable checked arena deref with the traversal cost stated honestly.
5. Write the missing semantics before any experiment claims anything: trap =
   process abort ruling; ring drain-on-drop teardown; per-form conditional
   send/share table; minimal publication-only memory model with a paper-grade
   DRF review; explicit fact kill columns (facts boundary-consumed only in
   v1; any `&uniq` escape kills all facts on the binding).
6. Drop the float/overflow-checked-int parallel-reduction WIN (fails the
   language's own law refuter). Surviving WINs: radix sort on closed derived
   keys, typed SPSC endpoints (affine endpoints eliminate hot-path RMW),
   strict-select guaranteed-branchless lowering.
7. Two-tier spec partition as an explicit mechanism: kernel rules plus a
   token-budgeted normative catalog appendix, both counted against the D2
   budget, drafted and counted before implementation. Every stance
   undercounted spec mass 3-8x; the honest budget is itself a headline
   result: **~60-75 new kernel rules on the 90-rule base plus a <=40k-token
   catalog appendix**.
8. Ratify named v1 non-goals with an expedited escalation lane rather than
   silent gaps: concurrent ordered map (LSM memtable class), writable
   MAP_SHARED IPC, inbound FFI callbacks, user-authored novel lock-free
   structures.

From the losing designs, merge: schema's parameterized packaging (closed
policy enums; 11-14 spec objects instead of 22-30 forms), relocation of
composite hot paths inside sealed forms (pool link-edit/splice/get_disjoint,
table retain/drain/prefetch_key, btree bulk-load), fact sheets as
machine-proved lemmas of the form invariant, and the verified-template
program as a preregistered upgrade path for table/pool (4-person-week
proof-closure tripwire, defaulting to sealed-TCB); core's confined
borrow-carrying values, `copy struct` declaration (settles the open records
question declaratively), closed atomic vocabulary, `split_uniq` combinator,
and cells/sealed/erasure kept as a research track for later de-TCBing of
individual sequential kernels; builtin's five-part TCB verification battery
(differential testing, exhaustive small-bound model checking, sanitizer/fuzz
soak, hostile pre-ship review, frozen per-form perf pins as CI build breaks)
and hidden-epoch reclamation with no user-facing pin/retire API.

## 5. Recommended catalog skeleton

Eighteen taught forms plus eight composition cards; as spec objects,
fourteen parameterized families. One line each (full text in
`evidence/judge.json`):

1. `seq<T, inline=N, growth>` — sealed — growable vector, small/inline
   vector, byte-builder base; (ptr,len,cap), tail-line push, geometric
   growth, reserve exports a capacity-slack fact, stride-1 slice views.
2. `ring<T>` — sealed, shares seq kernel — deque/worklists/sliding windows;
   pow2 masked head/tail, as_slices views.
3. `table<K,V, inline=N, hasher in closed set>` — sealed — SwissTable
   control bytes, 16B SIMD group probe, loan-typed entry token (one hash,
   one probe), retain/drain, prefetch_key.
4. `btree<K,V, key_kind closed>` — sealed — ~256B nodes, branchless in-node
   scan, leaf-chained range cursors, bulk-load-from-sorted.
5. `pool<T>` — sealed — slab/identity substrate; intrusive freelist,
   generational handles with stated wrap contract, no re-zeroing, dense
   iteration, in-template link-edit/splice/get_disjoint.
6. `arena<'a>` + generative branded id — language primitive + sealed kernel
   — bump alloc, per-arena-value brand for check-free deref, reset via
   region reincarnation.
7. Buffer core: `wbuf` + margin views — language primitive — typed
   initialized extent (zero-memset I/O), W-slop margins with
   initialization-at-introduction; mmap bytes never margined.
8. Layout view — schema-instantiated — declarative field/endian/offset
   schemas over byte buffers; validate-once fact, then one load per field,
   asm-diff-pinned.
9. Cursor — checked library over margin/loan core — varint/TLV decode,
   interleaved bit readers, overlap-tolerant LZ copies.
10. bytescan + hash kernels — checked library over the SIMD core —
    memchr/memmem/utf8/json-stage1/base64, crc32c/xxh3/siphash; writers call
    shipped instances.
11. alg kit — sealed sort core + checked search/heap/format — ipnsort-class
    adaptive sort with radix auto-dispatch on closed derived keys (WIN),
    strict-select branchless lower_bound, implicit 4-ary heap, itoa/ryu
    numeric emission.
12. machine core — language primitive — SIMD value types + closed named
    intrinsics + one-time ISA dispatch; endian loads; bit ops; select
    hint/strict; prefetch; NT copy/fill; align(N)/cachepad.
13. sync vocabulary — sealed TCB — relaxed counter, publish/snapshot cell,
    cas-cell, futex-adaptive `mutex<T>` with loan-typed guard, `once<T>`,
    condvar with consume-and-reissue guard; no rwlock, no user-facing
    ordering menu.
14. `conc_queue<T, endpoints in {spsc, mpmc, steal}, cap=2^k>` — sealed TCB
    — affine endpoint tokens make SPSC zero-RMW typed (WIN); Vyukov MPMC;
    Chase-Lev behind the par runtime.
15. `shared<T>` snapshot + `rcu_table<K,V>` — sealed TCB — epoch-pinned
    read-mostly state; readers write zero shared lines; reclamation fully
    hidden.
16. threads + par — sealed TCB — scoped spawn/join typed by lexical regions,
    blessed disjoint-split combinator, work-stealing substrate.
17. io family: file/net/evring/filemap/wal/os — sealed TCB — fill_buf
    borrow-the-buffer loans, buffer-loan affine moves through the completion
    ring with specified drain-on-drop teardown, io_uring lowering with epoll
    emulation, group-commit WAL + atomic replace.
18. extern-C boundary records — trusted-record class — bare ABI out-calls
    with per-argument ownership/aliasing contracts; inbound callbacks
    deferred.

Composition cards (taught patterns, zero new forms): LRU/CLOCK cache,
interner, CSR graph two-phase build, pool-linked trees, intrusive membership
links, timer wheel, sharded map, group-prefetch probe + Eytzinger layout.

## 6. Owner decision points

1. Gate #1 mechanism: authorize M1 (paper falsification of the loan/freeze +
   confined borrow-carrying values judgment). Interacts with settled T-A
   no-reborrow; requires a design-tree re-decision if adopted.
2. `pool<T>` checked generational recycling supersedes P2's never-recycle
   ruling — an explicit re-decision, not a silent override. The generation
   check makes stale handles trap; the never-recycle pattern remains
   available where peak-live memory allows it.
3. Trap = process abort ruling for the concurrency layer.
4. Ratify the named v1 non-goals (section 4 item 8).
5. `copy struct` declaration as the resolution of the open records question.
6. The D2 budget split: ~60-75 kernel rules + <=40k-token catalog appendix.

## 7. Open questions

Carried verbatim from the judge (full text in `evidence/judge.json`): the
loan-judgment decidability question (existential); the honest token count;
brands vs checked deref; iteration protocol; effect-row treatment of writer
behaviors (sort comparators) against the no-inference ruling; which kernels
get machine-verified templates vs plain TCB; sufficiency of the
publication-only memory model; trap-scope semantics for in-flight I/O and
held locks; boundary-only fact concession; catalog evolution procedure;
non-goal ratification; residual mmap cost honesty.

## 8. Validation plan (preregistered, cheapest-decisive first)

Pass/fail bands are frozen here before any implementation; full text in
`evidence/judge.json`.

- **M1** (paper, ~1 week; kills the architecture on failure): write the
  loan/freeze + confined-borrow typing rules; machine-check six canonical
  programs (accept push;push, guard+condvar reissue, borrow-carrying par
  env; reject entry-across-rehash, cursor-with-mutation, two-ring token
  swap). PASS: all six verdicts, <=15 rules, no flow analysis beyond
  per-binding loan sets.
- **M2** (paper, ~1-2 weeks): author real normative text for seq + table +
  conc_queue plus two cards in strict v0.6 ANF; count tokens; extrapolate.
  PASS: kernel+catalog <=40k tokens.
- **M3** (~2-3 weeks): prototype sealed seq/table with codegen contracts;
  locked workloads vs Rust Vec / hashbrown+foldhash. PASS: push/lookup
  <=1.10x, zero residual checks after the reserve fact in asm, entry loop
  one-hash-one-probe, iterate <=1.2x.
- **M4** (~1 week): arena brands in <=5 rules or measure checked-deref
  fallback <=1.25x C pointer arena on an L1-resident tree walk.
- **M5** (~2 weeks): W1 writability trial — mid-tier model, spec-only, 20
  tasks. PASS: >=70% first-green and >=80% of green solutions within 1.3x of
  the optimal spelling.
- **M6** (~2-3 weeks): spsc + mutex kernels — exhaustive small-bound model
  checking + 24h TSAN soak + perf bands (spsc round trip 6-15 ns, >=80M
  items/s batched; mutex uncontended <=20 ns; zero hot-path RMW in asm).
- **M7** (~1-2 weeks): shared<T>/rcu_table — 32-core Zipf read-mostly bench
  vs arc-swap/papaya/sharded-mutex. PASS: >=12x read scaling at 16 cores.
- **M8** (~2 weeks): checked-source SIMD — memchr >=0.85x memchr-crate;
  utf8_validate >=0.70x simdutf incl. the NEON movemask bridge; elision
  report shows main-loop bounds discharged.
- **M9** (codec bet, preregistered fallback): LZ4 block decode as checked
  source over margin/wbuf >=0.75x reference, fuzz-clean; else codecs move to
  sealed kernels and the row stays PARTIAL.
- **M10** (~1 week): boundary-semantics fault injection — evring
  drain-on-drop under ASAN (zero completions into freed memory);
  group-prefetch join probe >=2x naive.

## 9. Relation to prior tracks

All prior capability artifacts remain historical evidence and falsifiers per
D15. Notable convergences discovered independently by this pass: the buffer
core's typed initialized extent restates the earlier vacant/bytes leaf
distinction; generative arena brands restate generative roots; the
loan/freeze judgment is a bounded relative of the earlier obligation/escrow
machinery; the sealed-form op table with fact and kill columns is the
practical descendant of "metadata selects authority, never mints it". The
economic difference is structural: a narrow language core plus a sealed,
taught, per-form-verified catalog — with performance obligations pinned by
CI on exactly n forms — replaces a general capability calculus that had to
be sound for arbitrary library code. The fourteen-operation corpus and its
audits remain useful as stress evidence and workload baselines for M3-M10.
