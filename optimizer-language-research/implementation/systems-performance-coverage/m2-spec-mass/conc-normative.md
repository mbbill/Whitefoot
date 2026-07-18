# Concurrency — normative manual section

Always-loaded normative extract of the concurrency layer (ratified per D19). This
is what a writer needs to write correct and efficient concurrent Whitefoot; the
rationale, review history, and regression walks live in `KERNEL-DELTAS-DRAFT.md`
(the review record). Rules cite the kernel spec and the ratified loan rules
(`../m1-loan-judgment/RULES-RATIFIED.md`, R1-R15, AMD-1..5). The amendments AMD-6
(brand-carrier declassification, from BRAND-1), AMD-7 (par/spawn slot capability
premise), and AMD-8 (R15 concurrency schema) are ratified per D19 but not yet
folded into RULES-RATIFIED.md — that consolidation is a landing step.

## 1. Memory model (MM-0..MM-10)

Data-race freedom (D1) holds by these clauses. MM-1..MM-6 and MM-10 are
discharge obligations on the sealed forms; a writer relies on the result:
correctly-typed concurrent code has no data race on language-managed memory.

- **MM-0.** A *location* is a byte range of a language-managed object. Two
  accesses *conflict* iff they overlap and at least one writes. *Program order*
  ranges over a thread's operations and the memory events internal to each sealed
  op it runs. *happens-before* (HB) is the transitive closure of program order
  plus the synchronizes-with edges MM-1..MM-6 that the executing forms discharge.
  A *data race* is a conflicting pair on distinct threads not HB-ordered. THEOREM
  (conditional): if every sealed form discharges its edges via its release/acquire
  fences (MM-10), an accepted program has no data race. Scoped by SCOPE-3 (gated
  FFI, external writers of RO maps, and TCB internals excluded; externally
  mappable bytes carry no facts).
- **MM-1 (spawn/publication).** Everything sequenced-before a `spawn['p]` in the
  parent happens-before the child body's first action; every capture is published
  to the child.
- **MM-2 (join).** Each child's last action happens-before the `scope 'p` exit
  that joins it (and before a `par` return); a never-run spawn contributes the
  empty edge.
- **MM-3 (mutex tenure).** Per mutex instance, a total HB order of guard tenures
  (a tenure = `mtx_lock` return to the matching unlock). **O1** at most one issued
  token live per instance at any instant, thread-agnostic (a same-instance
  re-acquire BLOCKS or TRAPS, never issues a second token). **O2** unlock_i
  synchronizes-with lock-return_{i+1}. **O3** the interior address is stable across
  the tenure. **O4** a token issues only on an acquiring path (`try`/`timeout`
  variants issue only on `Ok`).
- **MM-4 (queue publication).** `cq_send(v)` contains a release store that the
  matching `cq_recv` reads with an acquire load, matched by reads-from (the acquire
  reads the store that produced `v`). Cumulative: everything program-order-before
  the `cq_send` (including a user write through a moved-in `box` payload)
  happens-before everything program-order-after the matching `cq_recv`; one release
  per batch covers its items. Last-`cq_tx`-drop happens-before any `RecvClosed`;
  last-`cq_rx`-drop happens-before any `QueueClosed(value)` handback.
- **MM-5 (endpoint seriality).** All operations on one endpoint are
  HB-totally-ordered; `spsc` has exactly-two-endpoint totality (clone rows reject).
  Sealed proofs may assume HB-seriality, never thread identity.
- **MM-6 (reclamation).** Every op and drop on any endpoint happens-before the
  storage free in the last endpoint's drop; the free is unique (acq_rel counting);
  the freeing thread's item drops observe every published item.
- **MM-7 (effect opacity).** A sealed sync op is never `pure`; its shared-state
  effect is non-erasable (survives region confinement, CAT-5a(ii) must not strike
  it). The EFF-1 `sync` atom marks it: no region-disjointness, CSE, DSE,
  code-motion, or purity sees through it, and no reordering across the
  acquire/release direction is licensed. Requires-engine facts over
  shared-reachable state fail closed.
- **MM-8 (split views).** A `par`/`scope` split-unique slot and a `&uniq 'p`
  capture partition their backing into byte-disjoint ranges; two split views of one
  place must not overlap.
- **MM-9 (trap ordering).** A trap happens-before any effect of the violating
  operation; the abort path performs no language-visible write. Surviving threads
  observe only pre-trap-valid in-process state during the bounded teardown window.
  External (persisted/peer) state carries NO prefix guarantee: an arbitrary subset
  of writes submitted before the trap may land after abort, possibly out of order;
  a durability point requires a completed submission barrier (`IO_LINK`/drain/
  `fsync`) before it.
- **MM-10 (target mapping).** `release`/`acquire`/`acq_rel` map to the C11/LLVM
  orderings with per-target lowering proofs (arm64 the stress bed).

## 2. The `sync` effect atom (EFF-1/EFF-2/FN-7)

EFF-1's canonical order is `sync, reads, writes, allocates, traps` (`pure` is the
empty row). A declaration exhibits `sync` iff its body or `requires` block calls
any op whose row includes `sync`, checked both ways (EFF-2). Row-equality
(R10(c)/R12) normalizes to this order. `main`'s at-most row (FN-7) is `sync,
allocates(heap), traps`. `sync` is carried by, and only by: `mtx_lock`,
`mtx_try_lock` (Ok arm), the guard drop; the ten `cq_*` rows (eight data —
`cq_send`/`cq_try_send`/`cq_recv`/`cq_try_recv` + the four batch rows — plus the
two clone rows `cq_tx_clone`/`cq_rx_clone`); and the compiler-derived endpoint drop
bodies. The pure constructors `mtx_new`/`cq_new` and `guard_uniq` do NOT carry
`sync`. An op is in the R15/AMD-8 concurrent-invocation obligation net iff its row
declares `sync` (AMD-8).

## 3. Scoped threads (CONC-1)

`scope 'p { stmt* }` introduces a thread-scope region `'p`.
`spawn['p]<body_targs>(body, capture_list)` starts a child running the named
`fn body` (no closures, FN-5; per-thread state is the env-struct pattern).
`body_targs` carries `body`'s explicit type/region/brand arguments (TYPE-5 form).
`capture_list` is written GRAM-11-named (`name: atom`). `spawn` returns
`own Result<unit, SpawnError>` (`enum SpawnError { Eagain(); }`).

Rules:
- **Capture loans.** Each borrow capture issues a loan entry on the captured
  place — `shr` for a `&'p` capture, `uniq` for a `&uniq 'p` capture — held by a
  compiler-introduced scope holder (one per `scope`), removed only at `scope 'p`
  exit. This is `spawn`'s form-table loan clause. Capture-borrow liveness is to
  scope exit (OWN-4 named-region), NOT OWN-6 statement-scoped. Later parent
  statements and later spawns are governed via R5/R6/OWN-5: a `set` over or
  `&uniq`/own re-access of a captured place rejects; a second `&uniq 'p` capture of
  the same place rejects (R5 uniq-overlap); two `&` captures coexist.
- **`own` captures.** An `own` Sendable value is moved into the child only on the
  Ok path; on `Err(Eagain)` the spawning thread drops it before `spawn` returns
  (same-thread, trap-free).
- **Scope exit = every leaving edge.** Every control-flow edge leaving the scope
  block — fallthrough, `break` to an outer label, `return`, and a try-statement
  `Err` propagation (GRAM-4/ERR-3) — joins all children FIRST, before that edge's
  R7/STOR-3 releases and drops.
- **Publication.** The spawn statement is a release edge to the child body entry
  (MM-1); each child body exit is a release edge acquired by the scope-exit join
  (MM-2).
- **Scope effect row.** The `scope` statement exhibits the union of each body's row
  instantiated at the RESOLVED TRANSITIVE ROOT region of each captured place
  (`resolved(place)` per OWN-6, the attribution EFF-2 uses for reborrows), plus
  `allocates(heap)` (child stacks) and `sync`, dropping a region only if that
  resolved-root region is introduced inside the scope or a body. Effects on
  caller/enclosing regions never drop.
- **Loop-spawn restriction.** Inside a `loop @l`, OWN-11 forbids naming an outer
  region and forbids moving an outside binding in (copies exempt), so a loop-spawn
  admits only COPY-own captures and captures of loop-body-local regions;
  runtime-count fan-out sharing outer state is not spellable in v1.
- **Capture is a thread-boundary position** (not an R2 call argument): any capture
  whose type is confined (a form-table opaque token OR a user `confined(...)` type,
  R1) rejects at the capture (CONC-2).

Scoped-spawn shape (legal program):
```
fn producer[brand 'q : spsc](tx: own cq_tx<'q, u64>) -> own unit { /* loop send */ }
scope 'p {
  let ends: own cq_ends<'qa, u64> = cq_new<u64, spsc, 10>();
  match ends {
    QueueEnds(tx: t, rx: r) => {
      let sp: own Result<unit, SpawnError> = spawn['p]<'qa>(producer, tx: move t);
      let got = drain(rx: move r);
    }
  }
}   // scope exit blocks until the child joins
```

## 4. Send/Share capability (CONC-2)

`Sendable` (safe to transfer to another thread) and `Shareable` (safe to share by
`&`) are BUILT-IN structural predicates, never user-conformable. Computed by a
memoized greatest-fixpoint (assume-holds-on-cycle) over the post-monomorphization
type graph. The `confined(...)` classification is checked FIRST and dominates.
Anything unmatched is fail-closed (neither).

| type | Sendable | Shareable |
|---|---|---|
| any `confined(...)` type (form-table token OR user) — CHECKED FIRST | never | never |
| primitive, tag-only enum | yes | yes |
| NON-confined user `struct` | iff every field Sendable | iff every field Shareable |
| NON-confined user `enum`, `Option<T>`, `Result<T, E>` | iff every payload Sendable | iff every payload Shareable |
| `array<T, N>` | iff `T` Sendable | iff `T` Shareable |
| `box<T>`, `buffer<T>`, `seq<T, N>`, `table<K, V, h>` | iff every payload Sendable | iff every payload Shareable |
| `cq_tx<'q,T>`, `cq_rx<'q,T>`, `cq_ends<'q,T>` | iff `T` Sendable | never |
| `mutex<T>` (sealed synchronizer) | iff `T` Sendable | iff `T` Sendable |
| `slice<'r,T>`, `uslice<'e,T>`, `ahdl<place,T>` | fail-closed (v1) | fail-closed (v1) |

**Slot/capture capability premise (AMD-7).** Every `par` slot and every `spawn`
capture carries a capability premise on top of R14/AMD-1 loan disjointness (both
must pass): a replicate/`&` slot requires the referent Shareable; a
split-unique/`&uniq` slot requires the referent Sendable-and-exclusively-
transferred; an `own` slot requires the value Sendable. R14's "no writes through
replicated slots" ranges over loan-judged `writes(...)` rows only; a `sync`-rowed
op is the reviewed exception (each `sync`-rowed physical write is data-race-free by
discharged O1-O4 exclusion or an atomic-RMW contract).

## 5. Mutex (CONC-3)

`mutex<T>` requires `T` Sendable at instantiation. `guard<'m, T>` is the
opaque-confined loan-token guard. Rows (`sync` per MM-7):

| op | signature | own | loan | effects | failure | cg |
|---|---|---|---|---|---|---|
| `mtx_new` | `<T>(v: own T) -> own mutex<T>` (`T` Sendable) | consumes `v` | NONE | pure | — | — |
| `mtx_lock` | `['m](m: &'m mutex<T>) -> own guard<'m, T>` | own guard out | ISSUE uniq (interior) | `sync` [blocks] | — | CG-LOCK |
| `guard_uniq` | `['x](g: &'x guard<'m, T>) -> &uniq 'x T` | — | ISSUE uniq on `g`'s own place (AMD-5 carve-out), holder = the returned view, R8-transferable; result region `'x` (receiver-carried, no free region) | reads('x) | — | CG-INL |
| `mtx_try_lock` | `['m](m: &'m mutex<T>) -> own Result<guard<'m,T>, LockBusy>` | guard out on Ok | ISSUE uniq (interior) on Ok arm only (O4) | `sync` | Result: LockBusy | CG-LOCK |

`enum LockBusy { Busy(); }`. Guard drop is a surfaced STOR-3 compiler-derived drop
whose release is the unlock; it removes the interior-uniq entry, is BLOCKED while
any `guard_uniq` view entry on `g` is live (R7 overlapped-drop), and its body is
trap-free. **Interior-view rule (`guard_uniq`):** the view returns at the receiver
region `'x`, which OWN-10 bounds within `g`'s scope, so it cannot outlive the guard
tenure — a `give`/`return` of the view to any outer region rejects (GIVE-1/OWN-4).
The `(g, uniq, view)` entry on `g` freezes `g` (R6), so at most one live interior
view exists, and is R8-transferable so R7's drop-order check on `g` holds even if
the view is moved. Cross-thread exclusivity is the MM-3 O1-O4 obligation (R15/
AMD-8), discharged by the sealed lock. `mtx_new`/`cq_new` are pure constructors,
not in the obligation net.

Guard idiom (legal program):
```
fn bump['m](m: &'m mutex<u64>) -> own unit sync {
  let g: own guard<'m, u64> = mtx_lock(m);
  region 'x {
    let v: &uniq 'x u64 = guard_uniq(&'x g);
    set deref(v) = iadd.wrap<u64>(deref(v), 1_u64);
  }
}   // g drops here -> unlock
```

## 6. Trap = whole-process abort (CONC-4)

A runtime safety trap terminates the WHOLE process (D18): abort with no unwinding
and no cleanup (SCOPE-4/EFF-4). Consequences:
- A trapping child runs no drops (reaches no R7 scope-end release).
- No thread ever returns from a join or observes a trap as a value; the abort is
  process-wide.
- Surviving threads observe only pre-trap-valid in-process state during the bounded
  teardown window; no thread reaches its R7 drops on the abort path. Skipping
  teardown is sound for process memory (no surviving in-process observer;
  cross-process writable maps are a v1 non-goal; only the sealed sticky-flag signal
  handler runs).
- External (persisted/peer) state gets the MM-9 subset-plus-barrier semantics.

Guard unlock and queue drain-on-drop (CQ-5) are NORMAL-exit R7 actions; the abort
path bypasses them, and their bodies are trap-free on the normal path.

## 7. par.for_chunks (usage)

`par.for_chunks` (a library form over the `par` statement, R14/AMD-1 + AMD-7) runs
a body over checker-split disjoint chunks; the body receives `(chunk_index: u64,
in_chunk: &uniq [T])` and, optionally, a zipped disjoint out-slot; it returns only
after all chunks complete, and any trap aborts the process. Usage:
```
par.for_chunks(data: &uniq 's seq<u64, N>) |ci, chunk| {
  // chunk : &uniq disjoint sub-slice; writes stay in this body's split view
}
```
The checker-proved cross-thread disjointness (MM-8) is the nonforgeable fact; no
blocking calls in a par body (blocking IO lives on scoped threads feeding a
`conc_queue`).
