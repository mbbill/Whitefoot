# Verdict CORE: the core model (D1 to D4), selected by debate; not yet ruled

Research date: 2026-09-16. Synthesized verdict of the core-model debate:
twelve seeded candidates, twenty-four adversarial critiques, three judges, one
synthesizer and one disposition clerk (record in
EVIDENCE-debate-core-2026-09-16.md (`EVIDENCE-debate-core-2026-09-16.md`, removed in the cleanup, recoverable at commit 4ca62f8db758);
family dispositions in [DISPOSITIONS-CORE.md](DISPOSITIONS-CORE.md)). It
selects a base and grafts for the decision points D1 (storage identity), D2
(where permission and state live), D3 (state vocabulary) and D4 (sequential
aliasing) of [OPTIONS.md](OPTIONS.md), against the requirement list of
[VERDICT-D0.md](VERDICT-D0.md). The owner has not ruled on it; its section 6
lists the decisions reserved for the owner, and nothing here amends the live
design tree. Supersede in place.


Research date: 2026-09-16. Synthesis of twelve candidates
(`core/candidate-*.md`), twenty-four critiques (`core/critique-*.md`) and three
judge reports (`core/judge-1.md`, `core/judge-2.md`, `core/judge-3.md`) against
VERDICT-D0 §2 (R1–R15, M1–M11), §4 (C1–C18), §5 (premises), §7 (owner interims
taken as settled), MECHANISM-MAP §3 (the hazard ladder), PROGRAMS.md (P1–P19)
and OPTIONS.md §§D1–D4.

All three judges ranked `value-semantics` first and named it the best **base**;
no judge claimed it is the most capable. The selection below is that base plus
the six grafts two or more judges named, plus four repairs the critiques showed
are load-bearing and no judge contested. Single-judge grafts and judge
disagreements are recorded as **Disputed** notes, never silently merged.

---

## 1. The selected core model

### 1.1 The tuple

| Point | Family (OPTIONS.md name, verbatim) | Source | Members refused inside the family |
|---|---|---|---|
| **D1** primary | **No storage identity (values only)** | base | RAD-D1-14 (immutable values only): in-place mutation is kept |
| D1, shared and cyclic structure | **Index into a named pool / row key**, the handle **branded by its pool** and carrying a **ghost slot and generation** | base + graft G1 (`generational-handles` D1/D3, **Tier A only**) | HIS-D1-25 (bare `u64`); RAD-D1-06 / HIS-D1-11 / HIS-D1-12 (a generation compared at run time, Tier B) — refused by name |
| D1, layout and aggregate decomposition | **Column / plane within one backing** as the *layout and distinctness* vocabulary | graft G3 (`columns-first` D1) | SYS-D1-14 (tile named by a schedule); RAD-D1-03 (allocation site + iteration vector) |
| D1, results | **Result-origin provenance sets** | base | HIS-D1-15 / HIS-D1-16 (body-derived origins) |
| **D2** | **Access path or parameter convention per call** | base | the Swift half of HIS-D2-10 (dynamic exclusivity fallback): no dynamic check anywhere (M2) |
| **D3** primary | **No states: validity by construction** | base | RAD-D3-01 (nothing ever ends): affine consumption still ends storage |
| D3, holes and vacancy | **State as ordinary data or typed outcome** (zero language states) | base | HIS-D3-12 / SYS-D3-18 as *language* state: occupancy is the writer's own field |
| **D4** primary | **Scoped projections and non-escaping references** | base | THE-D4-19 (capture-checked first-class references) |
| D4, decision procedure | **Overlap judged by resolved paths and proved extents** | base | HIS-D4-14 (literal-only ranges) |
| D4, fact scope | the projection block is the **window** an alias fact is declared over | graft G4 (`window-focus` D1 fine level / D4) | window-focus's two *name kinds*: a fine name still never appears in a type, and no `ptr<ρ↓w.k,T>` exists |

**One sentence.** *There are no pointers, so there is nothing to name, nothing to
track and nothing to invalidate; permission is where the access is written;
everything a tree cannot hold lives in a pool whose handles are branded by that
pool; and the layout of an aggregate is declared where the aggregate is, which
is also what proves its parts distinct.*

### 1.2 The grafts, with the judge count

| # | Graft | From | Defect it closes | J1 | J2 | J3 |
|---|---|---|---|---|---|---|
| **G1** | `Handle<P,T>` branded by its pool, with a ghost slot `σ(h)` and a ghost generation `γ(h)`, never materialized and never compared at run time | `generational-handles` D1/D3 Tier A | `pool[h]`'s undischarged domain (the M2(ii) bounds arm), cross-pool handle confusion, the stale-slot residue, and R7's returned-handle provenance hole | ✓ | ✓ | ✓ |
| **G2** | The separating **frame rule** as the *statement* of R6 — a call falsifies only what its written pre names; kills = 0 by rule, not by a kill heuristic. The rule, not the view algebra | `views-fold-unfold` D3 | value-semantics' R6 claim is false at every whole-container call (`push`, `reserve`, `drain`, `insert`, `get`, `compact`) | ✓ | ✓ | ✓ |
| **G3** | The **plane map** as the layout declaration doubling as the distinctness proof: stride, offset, alignment and SoA/AoS declared where the type is declared | `columns-first` D1 / R8b | R8b partial → met with no new construct; replaces the unsound fixed-table `AFF`; a length in a sibling plane cannot be killed by an element write | ✓ | ✓ | ✓ |
| **G4** | The `with … as` projection block is the **scope-declaration site** for `!alias.scope` / `llvm.experimental.noalias.scope.decl`, so the source construct and the backend fact are one object | `window-focus` §2.9 | the emission-time re-derivation value-semantics still performs (cost S7); the gain the critique withdrew | ✓ | ✓ | ✓ |
| **G5** | Restrict affine distinctness to a **checked type predicate** `NoReach(T)` (no reachable `Handle`, pool row or `Foreign<T>`), and derive the rest from the containment tree | `brand-context` / `refinement-heap` reading of R4(b) | `AFF` is unsound for two `dup`'d `File` wrappers over one pool row (soundness F3); collapses three distinctness relations toward C2's one | ✓ | ✓ | ✓ (as "plane map replaces AFF") |
| **G6** | The missing-fact diagnostic form: the payload **is** the rule's own side condition (`k != i && k != j`), and `v[i]` yields two separately named diagnostics — the R11 domain fact and the R1 state fact | `brand-context` P17, `brand-tokens` | M7's non-empty-payload metric becomes structural rather than aspirational | ✓ | ✓ | — |

Repairs applied with the grafts; the critiques showed each is load-bearing and
no judge contested any of them:

| # | Repair | Source | Why it is not optional |
|---|---|---|---|
| **P-a** | The **two-part footprint** for container operations: the addressed slot or extent *plus* the metadata plane | J1 graft 2; falls out of G2 + G3 | Without it `get(inout cache, k2)` rejects a held reader of `cache[k1]` unconditionally (cost S4), and P11 has no local repair |
| **P-b** | `old()`-indexed exit refinements with a fixed chaining rule | J1 graft 7; soundness S3 | Without it P4's surviving cursor fact is accepted for a reason R6 does not grant; `old()` is used in the base's §2.3 and §3 and listed in no inventory |
| **P-c** | **D4's policy**, not only its decision procedure: the four-cell read/write conflict table over overlapping paths | J1 graft 10; soundness S1 | Load-bearing for P1, P7, P9 and P11 and stated nowhere in the base |
| **P-d** | The **foreign kill rule**: every statement kills every contents fact about every `Foreign<T>` storage in scope, footprint or not | soundness S15 | R14(iv)'s kill obligation has no carrier under "footprint = written path", which is the one case where that identity is false |

### 1.3 The model

**Names.** There is no pointer type. Three forms replace it.

| Form | Spelling | What it denotes | Span |
|---|---|---|---|
| value binding | `let x: T`, `var x: T` | storage of type `T`, existing exactly while the name is bound | lexical |
| projection (second class) | `f(inout c.a[i])`, `with inout v[lo..hi] as s { … }` | the subtree at a **resolved path**, for the lexical span of the call or block; passed *down*, never up, never stored, never returned | the call or the block — and this span is also the alias fact's scope (G4) |
| branded handle | `h: Handle<P, Node>` | a row key of the pool whose identity parameter is `P`. Ordinary copyable data, one machine word. Ghost: `σ(h)` (slot), `γ(h)` (generation) | none — it is data; `Live(h)` is a **static** fact |

**The plane map** is part of a declared aggregate's type, and is both the layout
declaration and the distinctness proof (G3):

```text
backing Cols       { a,b,c,d,e,f,g,h : plane<u64, len n, align 64> }
backing Vec<T>     { meta  : plane<Meta,1>,      data  : plane<T, len cap> }
backing Pool<P,T>  { meta  : plane<PoolMeta,1>,  slots : plane<T, len cap> }
```

Struct fields are planes of a one-element backing. `'a.x` of the ladder's Step 1
*is* a plane entry; there is no second field-path mechanism.

**What the checker holds.** No per-storage state. Five judgments and one type
predicate, each named, each fixed, each terminating:

| Judgment | Form | Decided by |
|---|---|---|
| `BIND(n)` | per name, a boolean: still bound | a two-point lattice joined by conjunction; at most two passes over a reducible CFG |
| `PATH(p,q)` | two resolved paths overlap iff one is a prefix of the other after refinement, over the containment tree **root → plane → extent → element** | syntactic prefix comparison (HIS-D4-12) |
| `EXT(p,q)` | element and extent steps refine `PATH` by half-open extents | a fixed linear fragment over loop indices, `requires` terms **and the syntactically enclosing branch condition**, plus `ProvedRangePartition` for strided partitions (three side conditions: stride and base fixed at the loop preheader, `stride_nonnegative`, `base_nonnegative`) |
| `LIVE(h)` | `σ(h) ∈ Live(P) ∧ gen(P[σ(h)]) = γ(h)` — **ghost**, erased, never materialized, never compared | `INV` below |
| `INV` | syntactic instantiation of a **written** pool invariant at a **written** `use` step | verification of a written finite step; never rediscovery (M1(b)) |
| `NoReach(T)` | a checked type predicate: `T` reaches no `Handle`, no pool row and no `Foreign<T>` | a structural type-table fact |

`AFF` is not a sixth relation. Where `NoReach(T)` holds, "two distinct live
non-copy `T` values are disjoint" is a **corollary** of `PATH` over distinct
roots; where it does not hold, the fact is not available and distinctness must
come from the plane map or from `EXT`. C2's single relation is the containment
tree.

**The conflict table — D4's policy (P-c).** For two paths in one sequential
span whose overlap `PATH`/`EXT` does not refute:

| | `let` (read) | `inout` (write) | `sink` (ends) |
|---|---|---|---|
| **`let`** | permitted | refused | refused |
| **`inout`** | refused | refused | refused |
| **`sink`** | refused | refused | refused |

Non-overlapping paths: always permitted. Two overlapping *reads* are permitted,
which is what P1, P5 and P6 rely on and what the base never stated.

**The frame rule — R6's statement (G2).** A call falsifies exactly the facts
whose support its written footprint names; every other fact survives **by the
rule**. A footprint is the set of resolved paths in the parameter list, and a
container operation names **two** parts (P-a): the addressed slot or extent and
the metadata plane it touches.

```text
fn push   (inout v.data[len(v)], inout v.meta, e: T)   ensures len(v) == old(len(v)) + 1
fn reserve(inout v.meta, m: u64)                       ensures cap(v) >= m
fn get    (inout c.slots[k], inout c.meta, k: Key) -> V
fn compact(inout p.slots, inout p.meta)                ensures forall h. Live(h) => Live(h)
```

One stated exception (P-d): **every statement kills every contents fact about
every `Foreign<T>` storage in scope**, footprint or not.

**Take / put / replace / end / move / realloc.**

| Operation | Spelling | Checker effect | What changes |
|---|---|---|---|
| take | `let v = replace(inout s, None)` | nothing: total on `Option<T>` | a value moves out; `s` still holds a valid `Option<T>` |
| put | `s = Some(sink v)` | `BIND(v) := false` | ordinary assignment |
| replace | `let old = replace(inout s, sink new)` | `BIND(new) := false` | one exclusive access to path `s` |
| end | `sink x`, or `x` leaves scope | `BIND(x) := false`; the affine release runs here (per arm, not per join) | the storage ends; no name reaches into it |
| move | `f(sink x)`, `return x` | `BIND(x) := false` | bytes may move; unobservable — no projection outlives its statement |
| realloc | `push(inout v.data[…], inout v.meta)` | one exclusive access on two paths; `ensures` over `len` | the backing is anonymous; `len(v)` is killed and re-derived by the `old()` chain (P-b) |
| pool insert | `let h = insert(inout p.slots[σ], inout p.meta, x)` | fresh ghost `σ ∉ Live(P)`; `Live(P) ∪= {σ}`; `γ(h) := gen(P[σ])` | the pool's own written free-list pop |
| pool remove | `remove(inout p.slots[σ(h)], inout p.meta, h)` | requires `Live(h)`; `Live(P) \= {σ(h)}`; `gen(P[σ(h)]) := succ(γ(h))`; every fact `gen(P[σ(h)]) = γ_old` is killed | free-list push; **address reuse never revives the ended identity** |
| pool compact | `compact(inout p.slots, inout p.meta)` | footprint names `slots` and `meta`, not `Live(P)` or `gen`, so `Live(h)` is framed in | slot-map rewrite |

A commitment the model makes: **no implicit deep copy, no implicit
retain/release, no implicit copy-on-write**; in-place `inout` and
named-return-value elision are *lowering rules*, not optimizations. (Judge 3
graft 7: these are what keep R13 and M2(ii) true under value semantics and are
the easiest thing to lose in a merge.)

**Join and loop head.**

```text
if c { sink a; b } else { sink b; a }   // join: BIND(a)=false, BIND(b)=false; the if yields one value
```

- `BIND` joins by **conjunction**. A name bound on one arm and not the other is
  unbound after the join; the diagnostic names the arm that unbound it. There is
  no drop flag and none is representable: the merge of two owned values is a
  *value* merge.
- A branch's condition is available as a **premise** inside its guarded region —
  that is what makes P1's repair derivable — but **no storage state is a term
  over conditions**. The join is conjunction, never `ite`. (See Disputed D-1.)
- Loop head: the `BIND` fixpoint has height 2. **Nothing is written at a loop
  head for memory safety**, except one liveness clause per pool the body frees
  into (the price of G1).

**What a signature states.**

```text
fn helper (inout out: plane<u8>, let input: plane<u8>)
fn reserve(inout v: Vec<T>, m: u64)                   ensures cap(v) >= m
fn drain  (inout v: Vec<T>) -> Vec<T>                 ensures len(v) == 0
fn find<P>(let g: Pool<P,Node>) -> Handle<P,Node>     ensures Live(result)
subscript pick(let c: Bool, inout s: S) -> inout T    origin { s.x, s.y }
```

Vocabulary: three conventions (`let`, `inout`, `sink`); types; value refinements
with `old()`; outcome types; an `origin` set on a yielded projection; the pool
identity parameter on `Pool` and `Handle`; the plane map at the type
declaration. **Absent**: region parameters, lifetime parameters, entry/exit
storage states, loan clauses, memory effect rows, modes.

**What is erased.** `σ`, `γ`, `Live(P)`, every `use` step, every footprint,
every distinctness fact. What survives is what the source wrote as data:
`Option` discriminants, free-list heads, the handle's slot index.

| Source fact | LLVM form |
|---|---|
| `let` / `inout` / `sink` parameter | `readonly` / `noalias` / `byval`+`sret`, `memory(argmem: …)` |
| a binding's initialization edge | `initializes((0,n))` — **never** derived from a write convention (judge 2 graft 6) |
| plane entries of one backing | one `!alias.scope` per plane on every loaded data pointer |
| a `with … as` block (G4) | one `llvm.experimental.noalias.scope.decl` at the block head, one `!alias.scope` per projected path, `!noalias` against siblings — duplicated on unroll, because the property is per iteration |
| `sink` result of a fresh formation | `noalias` return, **only** where the result has no containment parent |

### 1.4 Disputed notes (minority judge positions, recorded not merged)

| # | Question | Majority (adopted) | Minority | Cost of the minority position |
|---|---|---|---|---|
| **D-1** | Condition terms at joins (C6) | Path conditions are **premises** inside a guarded region (enough for P1's repair); state is never a term. J2 refuses condition terms "in any form, until the owner rules"; J1 does not graft them | **J3**: state as a term over the writer's own captured branch condition, at joins only, restricted to the syntactically enclosing guard — needed for P8's naive shape and P4/P9 precision | an unbounded condition-atom growth nobody has bounded (M10); P8's restructuring disappears |
| **D-2** | R7's returned-handle provenance | Repaired by **G1**: `Handle<P,T>` names its origin in the signature | **J3 graft 2**: one effect row per declaration over parameter-relative paths (`ownership-topology` D2 / `window-focus`) | a second contract vocabulary beside the conventions; M8 and M4 both pay |
| **D-3** | The permission carrier | Three conventions; `let`/`inout`/`sink` already map to `readonly`/`noalias`/`memory(argmem)` | **J2 graft 5**: the `@0/@R/@W/@E` four-element join lattice on the parameter convention, making the pay-per-fact dial explicit | one more taught axis; the dial's benefit is a fact-precision gain nobody measured |
| **D-4** | Infer-and-pin | Admitted as a **tooling layer that never enters acceptance** (M1(b) permits it in terms): the tool prints derived footprints and distinctness, the writer pastes, the checker reads only written text | **J2 graft 8** proposes it; J1 and J3 do not | none if it stays outside acceptance; M1 dies the moment acceptance reads only pinned text |
| **D-5** | The M7 payload shape | The four rejection shapes are enumerated in the specification (below) because it costs nothing and no judge opposed it | **J1 graft 8** only (J3 praises it in scoring) | — |
| **D-6** | Does G1 reopen D1? | It **refines** it: one identity parameter on one nominal, reachable only through `Pool`/`Handle`; every other type has zero | J2 asks the owner to rule (owner decision O4) | if it reopens D1, C1 and C3 return and R15's parameter count leaves its floor |

**The four rejection shapes** (specification obligation under M7, from
`window-focus`):

| Shape | Payload | Repair named |
|---|---|---|
| overlapping resolved paths at one access | the two paths, the conflict-table cell, the missing inequality (`k != i && k != j`) | prove it, name the projection, or move the access outside the block |
| `BIND` unbound at a join, a loop back edge, or after a `sink` | the consuming statement and the arm that unbound the name | yield the survivor from the `if`; do not consume it in the loop |
| projection escape | the projection's binding site and the block that owns it | return an index or a `Handle`; pass a closure down |
| missing `REF` / `LIVE` | the missing fact and its tree location — separately for the R11 **domain** fact and the R1 **state** fact (G6) | a `requires`, a prior branch, a `use` step against the pool invariant |

---

## 2. Derivations under the selected model

Notation: `// accept: fact` or `// reject: missing fact`. `[G]` marks a line
whose verdict or ground comes from a graft or repair rather than from the base.

### P1 Container split with a runtime index

```text
fn split_write(inout v: Vec<u64>, i: u64, j: u64, k: u64)
  requires i < len(v) && j < len(v) && k < len(v) && i != j
{
  with inout v.data[i] as pi, inout v.data[j] as pj {
                              // accept: PATH(v.data[i], v.data[j]) refined by EXT; i != j from requires
                              // [G4] the block head is the noalias.scope.decl site; pi and pj get sibling scopes
    pi = 1                    // accept: `inout`/`inout` on non-overlapping paths -> permitted cell of the table [P-c]
    pj = 2                    // accept: same. PAR-1 overlap permitted, footprints {v.data[i]} vs {v.data[j]} (R5(i))
    let x = v.data[k]         // reject: PATH(v.data[k], v.data[i]) unresolved.
                              //   payload: missing `k != i && k != j` at this tree location [G6]
  }                           // both projections end at the block; no "put back" is written
  push(inout v.data[len(v)], inout v.meta, 7)
                              // accept: BIND(v) exclusive again; the projections are gone
}

// the repair the program itself demands:
    if k != i && k != j { let x = v.data[k] }
                              // [G] accept: EXT's stated inputs now include the syntactically enclosing
                              //   branch condition as a PREMISE. No state is a term over it (Disputed D-1),
                              //   so M10's flow-sensitive component is still one boolean per name.
```

Both writes carry no runtime comparison: `i != j` is a precondition (M2(i)).
**Verdict: accepted as required.** Cost: one `with` line, one written inequality.

### P2 Arena with independently released blocks

```text
backing Arena { meta : plane<Hdr,1>, bytes : plane<u8, len N> }     // [G3] the plane map IS the layout
invariant IA : forall b, b' issued and live. b != b' => extent(b) disjoint extent(b')
                                                                    // one written invariant per arena impl

match alloc(inout A.meta, carve A.bytes, 64) {                       // [G] two-part footprint: meta + a carve of bytes
  None      => { return Err(Exhausted) }                             // accept: a written arm; R2 discharges (nothing held yet)
  Some(b1)  => {                                                     // b1 : Buf<u8>, affine, extent proved from IA
    match alloc(inout A.meta, carve A.bytes, 64) {
      None     => { free(inout A.meta, sink b1); return Err(Exhausted) }
                                                                    // [G] accept: the arm disposes b1. R2 on every arm (C15)
      Some(b2) => { … same for b3 … }
    }
  }
}
par { fill(inout b1); fill(inout b2); fill(inout b3) }
      // [G5] accept: NOT by AFF. b1,b2,b3 are three carved extents of the plane A.bytes;
      //   disjointness is EXT over the extents, licensed by IA instantiated at the three pairs.
      //   AFF's fixed table is unavailable here: Buf<u8> carved from an arena is not NoReach-clean.
      // accept: A is not held between operations — the fills name A.bytes[..] sub-extents, and
      //   A.meta is a sibling plane no fill names (R5(a)(i), R2(e))
free(inout A.meta, sink b2)     // accept: BIND(b2) := false; b2's extent returns to the residue
let b4 = alloc(…, 32)?          // accept: a FRESH extent identity, even where its bytes are b2's
fill(inout b2)                  // reject: BIND(b2) = false. "b2 was consumed at line 12"
read(let b1); free(… sink b1); free(… sink b3); free(… sink b4)
                                // accept: each name bound exactly once; double free is BIND-rejected
```

Three notes the critiques force and the base did not carry:

1. `alloc` returning `Option`/`Result` is an **arena** exhaustion, a condition
   the writer's own free list decides — not heap exhaustion, which owner
   decision 5's interim (b) places outside the model. This is owner decision
   **O5** and P2's derivation rests on it.
2. `split(sink b, at) -> (Buf, Buf)` — carving one extent into two byte-disjoint
   affine values — is **one enumerated M3 entry with a boundary obligation**,
   needed for the non-coalescing arena too (soundness S8). Re-joining is not
   admitted: HIS-D9-12 stays refused, so a coalescing arena is out.
3. R4(e)'s negative test (`S` and `b1` must **not** be distinct) holds here and
   did not under the base: `b1` is a carved extent of the plane `A.bytes`, so it
   is a descendant of `S` in the containment tree and `PATH` refuses their
   distinctness. The base derived `b1` as a moved-out value, which made the
   negative test have no instance (soundness S7).

**Verdict: accepted with a stated cost** — one written arena invariant (3–8
`use` steps, once per arena implementation), one M3 entry for the split
primitive, one arm per fallible `alloc`, and no coalescing.

### P3 Graph with back edges in a pool

```text
backing Pool<P,Node> { meta : plane<PoolMeta,1>, slots : plane<Node, len cap> }
Node { next: Handle<P,Node>, prev: Handle<P,Node>, data: u64 }
invariant Links : forall σ in Live(P). Live(slots[σ].next) && Live(slots[σ].prev)
                                       // [G1] stated ONCE on the nominal, reused everywhere (R6(e))

// insert n between a and b — four link writes through several paths to one node
g.slots[σ(a)].next = n                 // accept: one path, one statement. Aliased sequential writes need
g.slots[σ(n)].prev = a                 //   no judgment (ladder Step 0); R9's negative test passes
g.slots[σ(n)].next = b                 // accept: no proof that a != n or n != b is required
g.slots[σ(b)].prev = n                 // accept
use Links_restore(n,a,b)               // [G1] a WRITTEN finite step; the checker verifies it (INV), never
                                       //   rediscovers it (M1(b))

// traversal
var h = start
loop {
  let nd = g.slots[σ(h)]               // [G1] accept: pool indexing is STATICALLY TOTAL. Live(h) discharged
                                       //   by INV from Links; NO bounds compare and NO generation compare
                                       //   is emitted — M2(ii)'s arm that capped the base is gone
  if nd.next == null_handle { break }  // the writer's own branch on the writer's own data
  use(nd.data); h = nd.next            // accept: Live(h) from Links instantiated at σ(h)
}

// parallel map over all nodes
par for h in g.keys() { g.slots[σ(h)].data = f(g.slots[σ(h)].data) }
                                       // [G] accept: per-iteration footprint {g.slots[σ(h)].data};
                                       //   pairwise distinctness of what keys() yields is a WRITTEN clause of
                                       //   the Pool interface (`ensures injective(keys)`), not an assertion
                                       //   that Handle values differ (soundness S13). .data is a sub-path of
                                       //   slots; meta is a sibling plane no iteration names (R6 by G2/G3)

// removal, then reuse — the line the base could not run
remove(inout g.slots[σ(m)], inout g.meta, m)
                                       // accept: Live(P) \= {σ(m)}; gen(P[σ(m)]) := succ(γ(m));
                                       //   every fact `gen(P[σ(m)]) = γ_m` is killed
let m2 = insert(inout g.slots[σ'], inout g.meta, node)
                                       // accept: may reuse m's slot; σ' carries a fresh generation
let stale = old_handle_to_m
let nd = g.slots[σ(stale)]             // [G1] REJECT: Live(stale) requires gen(P[σ(stale)]) = γ(stale),
                                       //   killed at the removal. R1(a)(ii): address reuse never revives an
                                       //   ended identity. This is the base's largest declared residue, closed.
// still refused, and named:
par for h in traverse(g) { g.slots[σ(h)].data = … }
                                       // reject: Links gives liveness, not injectivity, for data-derived
                                       //   handles. missing fact: distinct(σ(h1), σ(h2)).
                                       //   repair named: iterate keys(), or write an injectivity invariant
```

**Verdict: accepted as required**, at a cost the base did not pay: one identity
parameter on `Pool`/`Handle`, one written pool invariant with 3–8 `use` steps
once per pool type, and one liveness clause at the head of any loop that frees
into the pool. The traversal-order parallel map is refused with a named repair.

### P4 Cursor over a growing vector

```text
struct Cursor<P> { at: Handle<P, Vec<u64>>, i: u64 }    // [G1] the stored form P4 asks for — for a POOLED container
fn read_at<P>(let p: Pool<P,Vec<u64>>, let c: Cursor<P>) -> u64
  requires Live(c.at) && c.i < len(p.slots[σ(c.at)])
{ p.slots[σ(c.at)].data[c.i] }

let a = read_at(let p, let c)          // accept: REF from the caller's own i < len fact
push(inout v.data[len(v)], inout v.meta, 5)
                                       // accept: two-part footprint. ensures len(v) == old(len(v)) + 1
let b = read_at(let p, let c)          // [P-b] accept — and NOT for the base's stated reason. push's
                                       //   footprint NAMES v.meta, so R6 kills `c.i < len(v)`. Survival is a
                                       //   re-derivation: c.i < old(len(v)) /\ len(v) = old(len(v))+1
                                       //   |- c.i < len(v), by the old()-chaining rule. Named, not implicit.
                                       // [G1] Live(c.at) survives: push names v.data and v.meta, not Live(P)
sink v                                 // accept
let d = read_at(let p, let c)          // reject: R1(ii) use of ended storage. The diagnostic CLASSIFIES itself
                                       //   as R1(ii) and cites the consuming statement as the ending event
                                       //   (soundness S5: the base's "unwritable, no instance" is withdrawn)
```

The second required conjunct — *validity decided by state and proof, not by a
borrow that blocks `push`* — holds, with **state** replaced by an ordinary value
refinement and `push` never blocked. The first — *a stored pointer or handle to
a container in a struct* — holds **only for a pooled container**: a cursor over
a frame-local `Vec` stores an index and needs the container re-supplied at every
use, so a stored iterator or a callback holding a cursor over a non-pooled
container is still unwritable.

**Verdict: partial — the model is at fault.** Graded by the base's own rule for
P3 and P7 (soundness S4). The loss is carried into R7 and W6.

### P5 Struct-of-arrays kernel

```text
backing Cols { a,b,c,d,e,f,g,h : plane<u64, len n, align 64> }   // [G3] one line per plane; the writer wanted
                                                                 //   the layout declaration anyway
fn kernel(inout k: Cols, n: u64) {
  for i in 0..n {
    k.a[i] = f(k.a[i], k.c[i], k.d[i])
                       // [G3] accept: the eight planes are pairwise disjoint BY THE PLANE MAP — a partition of
                       //   one backing, no proof, no annotation, and sound for any element type (the base's
                       //   fixed-table AFF was not: soundness F3)
    k.b[i] = g(k.b[i], k.e[i], k.f[i], k.g[i], k.h[i])
                       // accept: same
                       // [C] R11 charges eight `i < len(plane)` facts per iteration; each is discharged from
                       //   the loop bound against the plane's own declared `len`, so the writer writes none —
                       //   but eight facts per iteration do enter the ProofContext, whose measured growing
                       //   cost is 29.9 / 164.5 / 1691.0 ms at N = 16/32/64 (M10, graded partial)
  }                    // PAR-2: iteration footprints {k.a[i], k.b[i]} disjoint by EXT (i != i')
}
```

Backend: one `!alias.scope` per plane on the *loaded* data pointer — the half
parameter `noalias` cannot reach (EVIDENCE §B.2). The identity the metadata
wants is the plane entry, which is in the type; for a projected sub-extent the
scope is declared at the `with` head (G4). The residual honest note: the
function-entry column scopes are still manufactured at emission from the
resolved path, so **the gain is partial, not withdrawn** (owner decision O9).

**Verdict: accepted as required** for M9's run-time half, with an unstated
checking cost at M10.

### P6 Range partition helper

```text
fn helper(inout out: plane<u8>, let input: plane<u8>)   // the convention IS the footprint

for w in 0..k {
  helper(inout out[w*s .. (w+1)*s], let input)
     // [C] accept, by ProvedRangePartition — NOT by the plain linear fragment. An ENT-4 relation term is
     //   "one datum displaced by a written constant" and `w*s` has a symbolic factor. The named family
     //   demands, and the model states, three side conditions:
     //       s, b fixed at the loop preheader      (a shape rule)
     //       stride_nonnegative                    (a retained proof)
     //       base_nonnegative                      (a retained proof)
     //   plus, per iteration and charged to R11: w*s <= (w+1)*s, (w+1)*s <= len(out), representability
     // [G4] the call is the window: one noalias.scope.decl per iteration, duplicated on unroll
}
```

**Verdict: accepted with a stated cost** — one named partition family with a
preheader shape rule and two nonnegativity derivations, plus the ordinary
formation, bounds and overflow obligations at the slice expression.

### P7 Take, put, replace through aliases

```text
var s: Option<T> = Some(t)
let v = replace(inout s, None)     // accept: total; s now holds None
let r = read(let s)                // ACCEPTED, returns None. P7 requires "refused: hole" — NOT REPRODUCED.
                                   //   vacancy is the writer's own discriminant; the read is safe in both arms
s = Some(sink v)                   // accept: BIND(v) := false
let r2 = read(let s)               // accept: Some
let old = replace(inout s, sink new)
sink old                           // accept — the base's derivation left `old` undisposed (soundness m2)
sink s                             // accept: the storage ends here
let r3 = read(let s)               // reject: BIND(s) = false. "s was consumed at line 8" — free is permanent
```

The program's premise — two long-lived writable names for one slot — is
**unwritable**: the conflict table refuses two overlapping `inout` projections
in one span, and a projection cannot be stored to become a second long-lived
name. Two of three required properties hold (state is the storage's; free is
permanent); the third (*take leaves a hole another alias may fill*) has no
instance.

**Verdict: partial — the model is at fault on line 3.** R3's only acceptance
test is P7, so **R3 falls with it** (soundness S12).

### P8 Conditional release and loop exits

```text
let s = if c { sink a; b } else { sink b; a }   // accept: each arm unbinds one name and yields the other as a
                                                //   VALUE. The join is a value merge, so no drop flag is
                                                //   representable — structurally, not by policy (M2(ii))
use(inout s)                                    // accept: BIND(s)
sink s                                          // accept: exactly one release on every path (R2)

// the shape the program writes literally:
if c { sink a } else { sink b }
use(let a)                                      // reject: BIND(a) false on the then-arm.
                                                //   "a is unbound on the path through line 1's then-branch";
                                                //   payload names the arm and the mechanical fix (M7 shape 2)

loop {
  if stop { break }
  let v = replace(inout p, None)                // accept: total
  …
  p = Some(sink v)                              // accept: BIND(v) := false
}                                               // loop head: fixpoint in one pass; NOTHING is written here
sink p                                          // accept: one release on every exit edge, including break
```

**Verdict: accepted with a stated cost** — one restructuring: the `if` becomes
an expression yielding the survivor. *Disputed D-1: judge 3 would accept the
naive shape by admitting a condition term at the join; judge 2 refuses condition
terms in any form pending owner decision O11.*

### P9 A callee that changes storage state

```text
fn drain  (inout v.data, inout v.meta) -> Vec<T>   ensures len(v) == 0
                                   // accept: "leaves x's slot empty" is an exit REFINEMENT on ordinary data,
                                   //   not an exit STATE on a storage
fn reserve(inout v.meta, m: u64)                   ensures cap(v) >= m
                                   // accept: "may replace v's backing" needs no clause — nothing names the
                                   //   backing (R8a has no instance here)
let d = drain(inout v.data, inout v.meta)          // accept: judged from the signature alone
reserve(inout v.meta, 128)         // [G2][P-a] accept: the footprint names v.meta only, so a fact about
                                   //   v.data[3] survives BY THE FRAME RULE. Under the base's whole-path
                                   //   footprint it was killed and the R6 "no instance" claim was false (S5)

fn find<P>(let g: Pool<P,Node>) -> Handle<P,Node>  ensures Live(result)
                                   // [G1] accept at the declaration: the result NAMES ITS ORIGIN in the
                                   //   signature. R7(a)'s provenance clause failed for every pooled API
                                   //   under the base (soundness S10); it passes here

subscript pick(let c: Bool, inout s: S) -> inout T  origin { s.x, s.y }
                                   // [C] a YIELD-ONCE ACCESSOR: a fourth interface form with its own
                                   //   declaration form, a resumption ABI and an origin set. Priced here,
                                   //   in the writer-cost table and in M8 — the base priced only the origin set
with inout pick(c, inout s) as t { t = 1 }
                                   // accept: for the span, facts about BOTH s.x and s.y are killed
let z = s.x                        // reject inside the span: PATH(s.x, origin) overlaps; `let`/`inout` cell
```

**Verdict: accepted with a stated cost** — `drain` and `reserve` are cheaper
than the row's own vocabulary anticipates; `pick` costs a fourth interface form.

### P11 Write-once-then-frozen cache

```text
backing Cache { meta : plane<CacheMeta,1>, slots : plane<Option<V>, len K> }

fn get<P>(inout c.slots[k], inout c.meta, k: Key) -> V   where V: Copy {
  match c.slots[k] {
    Some(v) => v                       // accept: total read, no state consulted
    None    => { let v = compute(k); c.slots[k] = Some(v); v }
  }                                    // accept: one exclusive access on path c.slots[k] and on c.meta
}

with let c.slots[k1] as r {
  get(inout c.slots[k2], inout c.meta, k2)
                                       // [P-a] accept iff k1 != k2 is proved. Under the base this REJECTED
                                       //   UNCONDITIONALLY, because get's footprint was the whole path `cache`
                                       //   and `cache` is a prefix of `cache[k1]` (cost S4). The two-part
                                       //   footprint names slots[k2] and meta; `r` is a read of slots[k1],
                                       //   and `let`/`inout` on non-overlapping paths is the permitted cell
  read(let r)                          // accept: the Init fact never depended on the fill; the CONTENTS fact
                                       //   survives because k1 != k2 was proved (R6 by G2)
}
```

**Pass/fail under R1 alone: PASS, with no new vocabulary.** A slot never goes
from "holding no value" to "holding a value" — it holds a valid `Option<V>` from
construction, so R1(b)'s deliberately excluded write-once transition has no
instance. A held reader across another slot's fill is admitted exactly when
`k1 != k2` is proved.

**Verdict: accepted with a stated cost** — one discriminant word per slot, one
written arm per lookup; concurrent readers with one lazy writer is R9/D10.

### P13 Relocation by a compacting third party

```text
let h = insert(inout p.slots[σ], inout p.meta, x)  // accept: h : Handle<P,T>, ordinary data
let ptr = pointer_of(h)                            // UNWRITABLE: there is no pointer type. R8a's mutual
                                                   //   exclusion holds by construction, not by a rule
compact(inout p.slots, inout p.meta)               // accept: the footprint names slots and meta; it does NOT
                                                   //   name Live(P) or gen, so Live(h) is FRAMED IN (G2)
let v = p.slots[σ(h)]                              // [G1] accept: Live(h) survives; the slot map resolves the
                                                   //   CURRENT slot. The handle IS the contract-visible fixup
                                                   //   form R8a demands, and the lookup is an operation the
                                                   //   SOURCE wrote, not a compiler-inserted fixup.
                                                   //   Under the base this was not derivable (soundness P13).
                                                   //   cost: one indirection per pooled access (M9)
```

The renumbering form `compact(sink p) -> (Pool, Remap)` stays **refused** at the
declaration: the handles it would invalidate are ordinary data scattered through
the program and the checker cannot enumerate them.

**Verdict: accepted as required**, with a stated cost of one indirection per
pooled access and the refusal of key-renumbering compaction.

### P16 Abstraction over identities

```text
fn map_nodes<P,T>(inout g: Pool<P,Node<T>>, f: (let T) -> T)
                     // accept: ONE identity parameter (the pool brand), zero per place. R15(e)'s metric is
                     //   "a stated function of the storages the callee touches" — here the identity function
                     //   on pool roots
fn kernel<E>(inout c: Cols<E>)
                     // accept: ZERO identity parameters; the plane map is a function of the declared type,
                     //   so PATH over plane entries is structural in the generic body

let hh = |inout slice: plane<u8>| helper(inout slice, let input)
                     // [C] accept as a SECOND-CLASS closure — and its interface must carry the CAPTURE SET:
                     //       hh : (inout plane<u8>) reads { input }
                     //   "the closure's footprint is its parameter list" is false: hh reads `input`, which the
                     //   caller's PAR-2 judgment over w needs (cost S11). One capture clause per closure.
                     // [C] the language has no closure or lambda construct today; the whole binder family is
                     //   new vocabulary, counted in M8 below (cost S10)
for w in 0..k { hh(inout out[w*s .. (w+1)*s]) }    // accept: ProvedRangePartition as in P6

fn pick_n<N: Nominal + affine>(c: Bool, sink x: N, sink y: N) -> N
                     // [C] the bound carries the VALUE CLASS. The base wrote `N: Nominal` and then gave one
                     //   definition two verdicts by value class, which is R15(a)'s check-once violated
                     //   (cost F2). With the bound, the body is checked once — and the linear instantiation
                     //   is FORECLOSED, not repaired. That is a capability loss, stated
```

**Verdict: accepted with a stated cost** — closures are second-class and
non-escaping and carry a capture clause; a linear `pick` must return both
operands or be written at a different bound; one identity parameter per pool
root.

### P17 Partial operations

```text
let q = a / d           requires d != 0             // accept: REF, from a written requires or a prior branch
match checked_div(a, d) { Ok(q) => …, DivZero => … }// accept: the written-outcome alternative (R12)
let b: u8 = narrow(x)   requires x < 256            // accept: REF
let y = v.data[i]       requires i < len(v.data)    // accept: REF against the plane's own declared measure
let y = v.data[i]                                   // reject: TWO named diagnostics, never one [G6]:
                                                    //   (1) R11 domain: missing `i < len(v.data)` here
                                                    //   (2) R1 state:   if the plane's run is not Full here
let nd = g.slots[σ(h)]                              // [G1] NOT an R11 site: pool indexing is statically total
```

Because a call's footprint is its `inout`/`sink` paths refined by the plane map,
`len(v)` survives a write to `w`, to `v.data[j]`, or to any sibling plane
without a frame annotation, so index proofs are rarely re-established.

**Verdict: accepted as required.**

### Summary

| Program | Verdict | Cost or fault |
|---|---|---|
| P1 | accepted as required | one `with` line, one written inequality |
| P2 | accepted with a stated cost | written arena invariant; one M3 entry (`split`); one arm per `alloc`; no coalescing |
| P3 | accepted as required | one identity parameter; one pool invariant + 3–8 `use` steps once per pool type |
| P4 | **partial — model at fault** | the stored-handle half holds only for a pooled container |
| P5 | accepted as required | eight length facts per iteration enter the ProofContext (M10) |
| P6 | accepted with a stated cost | `ProvedRangePartition` with three side conditions |
| P7 | **partial — model at fault** | the hole read is total, not refused; two aliases to one slot unwritable |
| P8 | accepted with a stated cost | one restructuring (the survivor is a value) |
| P9 | accepted with a stated cost | yield-once `subscript` is a fourth interface form |
| P11 | accepted with a stated cost | one discriminant per slot; one written arm per lookup |
| P13 | accepted as required | one indirection per pooled access; key renumbering refused |
| P16 | accepted with a stated cost | second-class closures with a capture clause; value class in the bound |
| P17 | accepted as required | — |

**5 accepted as required, 6 accepted with a stated cost, 2 partial with the
model at fault.**

---

## 3. Requirements table

| Row | Verdict | Reason | Price on the compiler | Writer cost |
|---|---|---|---|---|
| **R1** Sequential memory safety | **met**, conditional on **O3** | (i) every binding is initialized where bound and vacancy is data; (ii) storage ends only where a name is unbound, and **G1's ghost generation makes address reuse never revive an ended identity**, which closes the base's declared stale-slot residue and its undeclared cross-pool confusion; (iii) no name reaches into storage that can change layout (`pointer_of` unwritable). If the owner rules a ghost generation is the mechanism M2(ii) refuses, this reverts to **partial** with the residue | none on the backend; the compiler may reuse ended storage the program cannot reach | one pool invariant per pool type; one liveness clause at the head of a loop that frees into a pool |
| **R2** Resource lifecycle | **met** | Affine bindings release at `sink` or scope exit; joins conjoin `BIND`; **every outcome arm discharges `BIND`, including the `None` arm of a fallible `alloc`** (the base's P2 wrote none); no drop flag is representable | no release inserted, removed, or selected by runtime state the source did not branch on | one arm per fallible operation (C15) |
| **R3** Value classes and transfer | **partial — model at fault** | `sink`/`Option`/`replace` are the native vocabulary and the multiplicity lemma is the context discipline, but **R3(e)'s only acceptance test is P7**, whose "hole visible through `q`" premise is unwritable, so the row is met on no test (soundness S12) | none | one `sink` per consuming transfer; every movable-out field declared `Option` at its declaration site |
| **R4** Licensed facts | **met** | **One** relation: the containment tree (root → plane → extent → element), decided by `PATH`+`EXT`. `NoReach(T)` is a checked type predicate, not a third relation (G5), so C2's "stated once" holds. R4(e)'s negative test has an instance: a carved block is a descendant of the arena's byte plane, so `S` and `b1` are not distinct | forbids emitting a fact the source did not check; forbids *requiring* re-derivation to reach a stated fact — **partially discharged** (G4 for projections; owner decision **O9** for whole-function plane scopes) | zero: the plane map is the layout declaration the writer wanted |
| **R5** Race freedom and overlap | **partial** | Ground (i) is discharged from footprints that are already the parameter list (P1, P2, P3's map, P5, P6); grounds (ii) (checked accumulator law) and (iii) (named level) are R14's vocabulary, which this tuple neither supplies nor obstructs. P15 is not derived | forbids reordering across a stated sync edge; nothing else about scheduling | one `ensures injective(keys)` on a distinct-yielding iterator |
| **R6** Frame precision | **met** | **The frame rule is stated (G2)**: a call falsifies exactly what its written pre names. A footprint is two parts for a container operation (P-a), and a measure lives in a sibling plane (G3), so an element write cannot kill a length; a length-changing call kills it and the `old()` chain restores it (P-b). The 34/41 measured tax has no instance at projected *or* whole-container calls. One stated exception: the foreign kill rule (P-d) | none on the backend | zero when precise; one `old()`-chained `ensures` per measure-changing operation |
| **R7** Compositional call judgement | **partial** | Entry/exit states, holes, ended storage and replaced backings are all types, conventions and refinements; **the returned-handle provenance hole is closed by G1** (`-> Handle<P,Node>` names its origin). What remains partial: a returned *interior pointer* is only a yielded projection with an origin set and cannot be stored | caller acceptance never depends on a callee body beyond the interface | one convention per parameter; `origin` on a yielded projection; one identity parameter per pool root |
| **R8a** Storage-ending and relocation | **met** | The ending set is small, syntactic and contract-visible (scope exit, `sink`, assignment-over, insertion, return, pool removal); relocation is unobservable because nothing names a subterm across a statement; **the handle is the contract-visible fixup form for third-party compaction** (P13, by G1+G2). The conditional reallocation contract is an `ensures` over `cap`, not a two-armed state — which is why R8a's red clause does not bite here | the compiler relocates no storage while a projection names it | one `ensures` per relocating operation |
| **R8b** Placement and physical demands | **met** (moved from partial by G3) | **The plane map *is* stride, offset, alignment and SoA/AoS, declared where the type is declared**, so `align(64)` is not a second mechanism and does not need to pin a binding. Whether R8b is in scope at D1–D4 at all is owner decision **O13** | forbids reordering planes or choosing padding where a plane map states offsets or alignment; forbids DSE of a marked store | one line per plane, which is the layout declaration the writer wanted anyway |
| **R9** Shared mutation among holders | **partial** | Several long-lived holders are several `Handle`s into one pool, and **with the two-part footprint two holders writing distinct slots no longer conflict** (the base serialized every write through the whole pool path). What remains unwritable is two *simultaneous* long-lived writable names; the concurrent half is D10's | forbids inserting a lock, fence or generational check the source did not write | zero sequentially |
| **R10** External resources | **partial** (from met) | A descriptor is an affine wrapper, the open-file description is a pool row, contents are `Foreign<T>` — but **P14 is not derived**, and the row's hazard is real: under G1, `close(sink f1)` bumps the description row's generation and kills `f2`'s handle, which is safe and *wrong* for `dup`. Either `dup` is refused or the description's end is a written operation on the last wrapper | forbids a compiler-special path for I/O values | one interference qualifier per foreign identity |
| **R11** Partial operations | **met** | Orthogonal, and cheaper: R6's path-exact frame means index and divisor facts survive unrelated writes, and **pool indexing is no longer an R11 site** (G1 makes it statically total) | none | one proof or arm per partial site |
| **R12** Failure paths and typed outcomes | **met** | Outcomes are ordinary sums; every arm discharges `BIND`; the same mechanism carries vacancy, so it is one construct, not two. The arm count is **per fallible operation**, not per pooled access — that is G1's doing | forbids eliding or inserting an outcome arm | one arm per fallible operation |
| **R13** Resource bounds | **partial** (from met) | A byte budget over an arena is a counting fact — the sum of issued extents is bounded by the parent plane's extent **by construction** (G3), so P18's budget half needs no cost semantics — but P18 and P19 are **not derived**, and depth and termination are untouched by D1–D4 | forbids introducing allocation or stack growth where a bound is promised | budget clause plus proof steps, where promised |
| **R14** Stated execution model | **partial** | The tuple supplies (iv)'s carrier — a foreign-writable storage is a *type* (`Foreign<T>`) — **and the kill rule that carrier needs (P-d)**, which the base lacked. (i), (ii), (v) and (vi) are R14's own text | forbids weakening a stated happens-before | zero |
| **R15** Abstraction | **partial** | Zero identity parameters everywhere except one on `Pool`/`Handle`; `PATH`/`EXT` are structural in a generic body; **check-once is repaired by putting the value class in the bound** (`N: Nominal + affine`), at the cost of foreclosing the linear instantiation; the closure's capture set is in its interface | forbids losing a stated fact at an abstraction boundary | one bound per abstracted value class; one capture clause per closure |
| **M1** Deterministic acceptance | **met** | Six named families, each total and terminating with no search, no solver, no budget, no iteration cap: `BIND` (height-2 lattice), `PATH` (syntactic prefix), `EXT` (fixed linear fragment + `ProvedRangePartition` with three stated side conditions), `REF` (R11's existing discharge), `INV` (syntactic instantiation of a *written* invariant at a *written* `use` step), `NoReach` (a type-table predicate). Path conditions enter as premises, never as an accumulated state term (Disputed D-1). Two inherited **capacities** (the 4096-entry proof-use ceiling, the affine-formation capacity) bound representable programs and do not select acceptance | forbids budget-selected acceptance | zero |
| **M2** No unstated failure edge | **met**, conditional on **O3** | (i) every failure edge is a written arm; (ii) **zero compiler-inserted branches**: no drop flag (the join is a value merge), no bounds compare in `pool[h]` (G1 gives it a static domain, which is the arm that capped the base on all three judges' scoring), no runtime generation compare (Tier B is refused **by name**), no occupancy structure the compiler maintains. The `Option` discriminant is a field the source declared with both arms safe. **If the owner rules that a ghost generation is itself the refused mechanism, this row is violated** and the model reverts to the base's residue | forbids inserting any check, trap or bookkeeping the source did not write | the branches the writer writes |
| **M3** Trusted base | **met**, with one new enumerated entry | No `unsafe`, no bare assume. `Pool<P,T>` is ordinary WF code over one plane and a written free list. **One** new entry: `split(sink b: Buf<u8>, at) -> (Buf, Buf)`, the byte-extent carve, with its disjointness boundary obligation — needed by the non-coalescing arena too (soundness S8). Owner decision **O17** confirms the library-versus-entry split | forbids trusting any source-level assertion | zero |
| **M4** Writer regularity | **met** | One spelling per construct across the inventory: three conventions, `with … as`, `sink` as a statement, `subscript … origin`, `plane`, `Pool`/`Handle`, `invariant`/`use`, `old()`. No mode axis, no region syntax, no second element spelling (an element is `[i]`; an occupant is `Handle<P,T>`, a different construct with a different meaning) | none | — |
| **M5** Acceptance independent of permission | **met** | Overlap permission is computed from footprints already in the signature; deleting the PAR judgment changes no verdict | none | — |
| **M6** Soundness evidence | **partial** | The bounded small-model check is unusually cheap (the state space is resolved paths × one boolean per name × a bounded ghost generation domain); frame soundness (ii) is the frame rule itself; erasure (iii) is total. **G1 additionally restores an M6(iv) channel the base had none of**: a build outside the accepted program may materialize `σ`, `γ` and `Live(P)` and assert that every erased `Live(h)` would have held. The adequacy theorem is owner decision 9's later obligation | none | — |
| **M7** Repairable rejection | **partial** | Four rejection shapes are enumerated with a tree location and a named missing fact (§1.4), and G6 makes the payload the rule's own side condition. **The projection-escape class has no local fix**: P4's stored cursor and P16's stored closure repair by threading a container or pool parameter through every intermediate signature, which is a signature cascade, not a local fix (cost S6) | forbids a rule whose only fix is non-local and unnamed | — |
| **M8** Teaching budget | **partial, pending O6** | Counted, not asserted — the constructs the derivations use: `let`/`inout`/`sink`; `with … as`; `sink` as a statement; `subscript` + `origin`; `plane`/the plane map; `Pool<P,T>` and `Handle<P,T>` with the ghost slot and generation; `invariant` + `use`; `old()`; `Foreign<T>`; the no-implicit-copy rule; the conflict table; `ProvedRangePartition`'s three side conditions; and a **second-class closure form with a capture clause, which the language does not have today**. That is **13 constructs, one of them a new binder family**, against the base's claimed five and against rivals at 30–50. K, the tokenizer and the subsystem share are unset, so "met" is not decidable | may forbid a fact vocabulary that breaks K | — |
| **M9** Measured performance floor | **partial** | P5's and P6's obvious shape is the written shape with no hints, no guards and no versioning, and the per-plane facts are a consequence of the layout declaration. **G4 removes the emission-time re-derivation for a projection's facts**; the function-entry plane scopes are still manufactured from the resolved path (owner decision **O9**). Costs: one indirection per pooled access, one discriminant word and one branch per vacancy read, one dependent table load per traversal hop; in-place `inout` and NRVO must be lowering **rules** | forbids a design whose default-accepted shape is off the floor | — |
| **M10** Checking cost and edit stability | **partial** | The flow-sensitive component is one boolean per name, so joins are linear in live names and there are no condition atoms to bound. But **every `EXT`, `REF` and `INV` fact enters the ProofContext**, whose measured growing-context cost is 29.9 / 164.5 / 1691.0 ms at N = 16/32/64 (exponent ≈ 2.6–3.4) against a matched three-use control at 18.4 / 40.8 / 166.3 ms, and the flow closure is a cubic transitive product. The edit-reach function in the callee direction is the **transitive instantiation closure**, not "the enclosing declaration plus callers" (375-line generic fixture at 82.4 s against 1757-line `wfgrep` at 42.8 s). Degrees are owed, not stated | forbids a family above its stated degree | — |
| **M11** Evolution with stated loss | **met** | The verdict diff against today's borrow rules is a **rewrite, not a delta**, and is stated as such: borrow modes, regions and `&uniq` have no image in this tuple | none | migration rounds counted per revision |

**Counts: met 14 (R1, R2, R4, R6, R8a, R8b, R11, R12, M1, M2, M3, M4, M5, M11),
partial 13 (R3, R5, R7, R9, R10, R13, R14, R15, M6, M7, M8, M9, M10), violated
0.** R1 and M2 are met **conditional on owner decision O3**; if the owner rules
against the ghost generation, both revert (R1 → partial, M2 → violated) and the
model is the base with its declared residue.

Against the base's own claim (met 20 / partial 7 / violated 0) and the cost
critique's recount of the base (met 12 / partial 11 / violated 1 / not derived
2), this is the honest middle: the grafts convert R1, R6, R8b and M2 and the
repairs force R3, R10 and R13 down.

---

## 4. Refused candidates

| Candidate | Its strongest point, engaged | Why it is not the base | Grafted into the selection |
|---|---|---|---|
| **window-focus** (wildcard) | *"A fact has a span, not a lifetime."* Making the source-level fine name and LLVM's window-scoped `!alias.scope` the same object is the sharpest M9 argument anyone offered, and its M7 four-shape payload is the best answer to a red row in the set | Every fatal finding is a case where the model needs a name in the level it just forbade: a fine name cannot carry state across a call (F1), cannot be typed when a pointer must hold one (F2 — the interior pointer therefore escapes and the use-after-relocation is admitted with **no diagnostic**), cannot be minted in unbounded number (S4). Coarse names cannot express sibling disjointness (F3) or a disjunctive post-state (F5, F7), and **P8 has no accepted spelling**, so R2's own acceptance test fails undeclared. Four judgments are simply absent | **G4** (the projection block as the alias fact's scope) and the **four rejection shapes** under M7. Its two-name-kind split is *not* taken: a fine name still never appears in a type |
| **columns-first** | The only candidate that answers **R8b** without a second mechanism — the plane map *is* stride, offset and alignment, so an `align(64)` demand has a home where the identity is declared — and the cleanest statement of why the 34/41 tax disappears (the length lives in a sibling plane) | M4 declared violated (four reference forms, two element spellings, three carriers of absence) and +20 to +28 constructs. Worse, the soundness findings are not repairs: extents have **no liveness state**, so `read(b2[0])` after `free(A,b2)` is *accepted* and a double free is refused by nothing (F2); `disj` is an `iff` over four cases and none applies to arena blocks (F3); origin sets are not in the grammar, so a race is admitted (F4); the ghost `slot_of` is erased, so P13's accept reads the wrong occupant (F5). It wins the programs that were never hard | **G3** (the plane map as layout vocabulary and distinctness proof) and, through it, **P-a** (the two-part footprint) |
| **generational-handles** | The most honest document in the set — the Tier A/B split states its own refusal before the argument — and its P13 answer (the handle **is** the contract-visible fixup form, written by the source) is conceptually the best R8a result | Two independent critiques find **Tier A** also violates M2(ii) or M3 (the pool's free list is either compiler bookkeeping or an unenumerated entry); the ghost `σ` and the runtime `idx` are never related, so `h.idx < cap(P)` has no source and erasure has no theorem; `free` requires only `Live(h)`, so P7's `free A` leaks a linear occupant; `Live(P)` grows by a fresh ghost element per loop iteration with **no widening stated**, and a fifth family (quantified container state) is used in three derivations and named nowhere | **G1** (Tier A only), with the M1 and M3 holes closed here: the free list is the pool library's own written data (M3 entry named), and `INV` is declared as a family |
| **views-fold-unfold** | Best interference score in the set. `⊗` genuinely **is** the disjointness relation, so C3's call-site discharge problem disappears rather than being answered, and `Gone` as the absence of a view with never-reminted gnames is the cleanest treatment of address reuse anywhere | The worst writer cost in the set: ~50 taught items, an open unenumerated lemma family, and a proof script per statement — and P5's unannotated fallback has a **measured** price in this repository (2.1 vs 0.4 ns/element, 3.4–5.7×, flat across trip count), so M9's obvious-shape clause is measured-negative. `free` consumes `own \| uninit` and says nothing about contents, so freeing storage holding an obligation is admitted; `frozen` is one-way, so P11's `tok` is permanently undischargeable. M1 survives only by fixing an enumeration order over `r^m` witnesses | **G2** — the frame **rule**, taken without the view algebra that carries it |
| **brand-context** | The single best headline in the set: deleting the exclusivity rule makes P7, P3's four link writes and R9 fall out for free, and `distinct`-by-default keeps the default fact strong. Its M10 self-declaration is honest where most would have hidden it | `st` is a path-keyed map resolved modulo may-equality at exactly one line of the document and by literal path identity everywhere else, and every wrong verdict is an instance: a double `take` duplicating an affine value, a traversal cursor reading ended storage, a may-write marking both origins `Init`. M1's "exactly four families and no fifth" is false on its own derivations (≥4 unnamed). `pick<N: Nominal>` is well-kinded only at identity-arity 1 | **G5** (R4(b)'s discipline: distinctness is a proved relation over containment, never a type-table shortcut) and **G6** (the P17 two-diagnostic form) |
| **ownership-topology** | The second-best authoring story of the identity-carrying candidates: in P5, P6, P7, P8 and P9 the writer writes **no identity name at all**, and one path grammar + one row grammar + one state vocabulary is genuinely regular | It abolishes pointer duration and then writes three rules that cannot be evaluated without one (`pin` release, `drop p`, `with_fixup`); index children and extent children of one parent are declared disjoint while overlapping (a representable race); `p = p.next` has **no path-join rule**, so the loop-head computation does not terminate; and there is no effect row on an arrow or closure type, so `map_nodes`'s own row is either unsound or silently assumes `f` pure | Nothing structural. Its "one effect row per declaration" is recorded as **Disputed D-2**; R7's hole is closed here by G1 instead |
| **brand-tokens** | R6 is the best-argued row in the candidate: the frame is exact **by construction** because there is no kill rule at all — a call can falsify only facts about extents whose tokens it received | The core judgment "does the context contain a token of the required type" is an **exact-cover search over symbolic extents** with no fragment, no normalizer and no bound; `Pt<ℓ▷e,q,s>` is a dependent type over program variables; the loop rule is multiset *equality*, which rejects P6's own k-tile loop. Extents are erased, so sub-location distinctness reaches the backend only by retaining the derivation, which R4's price forbids requiring | Nothing. The token-routing cost profile is the one judge 3 handles worst |
| **reference-refined** | The incumbent family's best form, and it identifies the right seam: validity and occupancy move to the path, permission stays on the reference — which is exactly what makes `push` writable while a cursor exists | ~45 added constructs against six retired rules (≈7:1 in the wrong direction), and then M1 — the row the design is organized around — is violated undeclared in two places (propositional entailment over condition terms doubling per diamond). No propagation rule over the containment structure of paths, so P2's `read(p2)` after `free(b2)` is **accepted**: the one refusal the whole D1 choice exists to deliver | Nothing |
| **whole-program-pinned** | "Zero writer tokens" is the largest single reduction available against K, and it is aimed squarely at an AI writer. R7(d) does leave authorship open | For a zero-token writer the accept set is exactly what `wf pin` can pin, so **the tool *is* the acceptance relation** — and the candidate then permits widening inside it. The monotone re-pin rule makes the source of record a function of edit history, so two runs over byte-identical code give different pins and different diagnostics. Nine fatal findings including a double free and a use-after-end | **Inverted, as Disputed D-4**: infer-and-pin as a tooling layer that prints derived footprints for the writer to paste, with the checker reading only written text (M1(b) permits this in terms) |
| **typestate-protocols** | Making interior pointers and third-party relocation mutually exclusive **by the pool's state** is stronger than R8a asks, and is the candidate's real contribution | `mints 'a # 'b` is a **writer-written distinctness axiom** with no body-side obligation, so a wrong arena body makes P2's parallel fill an admitted race and M3 is violated rather than met; the candidate's own P2 line concedes `'B4 # 'B2` holds "although the bytes coincide", which is inequality of identity names, which R4(b) forbids **and names P2 as the reason**. The join is not a total function: the state hierarchy is undeclared and indexed states make the lattice infinite | Nothing. Writer-declared automata move the safety vocabulary into libraries the writer must also author |
| **refinement-heap** | One decidable fragment serving R1, R4, R5, R6, R11 and R13 is the most economical idea in the debate, and bunched `∗`/`;` groups are an elegant, cheap signature form | Its load-bearing sentence fails on its own terms: **the stated closure does not terminate** (rules (b) and (c) generate unboundedly many constraints; "the monomial set never grows" is a non-sequitur because the closure iterates the *constraint* set), and rule (d) either grows the monomial set or does not reach P6's goal. The kill rule says "overlaps" where soundness needs "not provably disjoint", so a stale refinement discharges an R11 domain and produces a **division by zero in an accepted program** | **G5's** half: ground distinctness in a proved relation over containment, and restrict what a type-level shortcut may claim |

---

## 5. Consequences for D5–D13

| Point | Forced by the selection | Foreclosed |
|---|---|---|
| **D5** interference source | *Effect footprints over identities* degenerates to **footprints = the parameter convention list**, refined by the plane map and by `EXT`; a container operation's footprint has **two parts** (addressed extent + metadata plane). *Ownership transfer and shared-nothing* is the natural parallel construct. An affine index fragment plus `ProvedRangePartition` is required | *Loans on references beside footprints*; *Fractional or counting permissions*; *Runtime dependence tracking with static summaries*; *Schedule separate from the algorithm* as a second checked artifact; whole-program points-to as the fact source |
| **D6** distinctness at boundaries | **Largely dissolved**: two arguments cannot be the same storage unless their paths overlap, which the caller's own `PATH`/`EXT` already decides (*Value semantics removes the question*). **G1 reopens exactly one sliver**: two `Pool` parameters with distinct identity parameters need call-site discharge, so *Distinct by default, call-site proof* and C3 return **for that one nominal and nowhere else** | *Pairwise clauses in the signature*; *Apartness as a judgment* in general; *Per-instantiation distinctness*; *Dynamic distinctness check*; distinctness from inequality of identity names, anywhere |
| **D7** contract vocabulary | *Ownership and identity transfer as the summary* plus *Footprint and frame clauses*, both collapsed into the three conventions; **three additions are required**: a finite static `origin` set on a yielded projection, `old()`-indexed exit refinements with a fixed chaining rule, and the yield-once `subscript` form. The pool identity parameter is the only existential-result vocabulary | *Typestate and permission pre/post per parameter*; *Store-typing rewrite signatures*; *Prophecies and backward functions*; a separate effect-row language beside the conventions (**Disputed D-2**) |
| **D8** placement and relocation | *Projections and accessors instead of interior pointers*; *Pools with stable index handles*; **Slot reuse in pools: generations and free lists** — no longer contested, G1 settles it; *Destination-passing and in-place result construction* (required by M2's no-unchosen-cost); *Writer-declared placement per binding* through the plane map | *Pinning and immobility* (nothing needs pinning); *Relocation with a fixup channel* in its runtime-rewriting form (nothing rewrites a stored pointer, because none exists); *Reallocation as a contract-visible name change* (invisible); a moving collector while a projection is open |
| **D9** containers and backings | *Index-only access with bounds proofs*; *Bounded views and proved subrange subdivision*; *Disjointness by index arithmetic or declared partition*; **Structural decomposition: chunks, columns, lanes** (the plane map); *Pools, free lists and slot recycling* with *Relocation-stable ghost identity*; **the byte-extent `split` primitive as the one M3 entry** | *Per-element ghost tokens*; *Existential backing per formation*; *Generative index brands*; stored *Iterator and cursor objects*. **Needed and still refused:** *Split-and-join range tokens* (HIS-D9-12) — so the coalescing arena stays out |
| **D10** concurrency | *Structured fork/join and linear task handles* and *Message passing only* fit with no new vocabulary (a task takes `sink` values and returns them). **A lock is a scoped projection**: `with lock(inout m) as g { … }` — C11 with the guard's *span* replacing the guard's lifetime, and the guard's window is also the alias fact's scope (G4). A shared pool's free list is itself mutable shared state, so the allocator needs *Lock as custodian* or *Per-core shards* | *Ambient per-thread context*; *Typed reference capabilities and sendability* as a second mode axis (sendability is `sink`); long-lived cross-thread holders without a pool and a lock; lock-free slot reuse without R14(iv)'s declared access class |
| **D11** external resources | *Host values as ordinary declarations, one domain*; *Three identities per descriptor* maps to (affine wrapper, pool row, `Foreign<T>` type qualifier) **with the foreign kill rule stated as an exception to the footprint identity** (P-d). *Typed failure and partial outcomes* for short reads | *Ambient and forgeable host access*; *World token, monadic sequencing, pure core*; a second host-authority category. **Left open:** whether `dup` survives (R10 partial) — the description row's end must be a written operation, or `dup` is refused |
| **D12** surface form | *Conventions instead of pointer types*; *Explicit-only core, one spelling per construct*; *Annotation site placement* = type definitions (the plane map), signatures and projection blocks only; *Diagnostics as a specified surface* (the four rejection shapes) | *Modes as sugar over pointer plus state*; *Region sigil fate* and *Region block and borrow-endpoint spelling* (there is no region to spell). **Newly wanted:** *Elision and implicit identity parameters* for the pool brand, which G1 introduces and which was previously "nothing to elide" |
| **D13** validation | *Bounded exhaustive enumeration* is cheapest here — the state space is resolved paths × one boolean per name × a bounded ghost-generation domain. *Fixed discriminating program set* as the acceptance-set diff. **G1 re-opens a channel the base foreclosed**: because `σ`, `γ` and `Live(P)` have a direct runtime shadow, *Executable semantics with a dynamic shadow-state oracle* becomes a ready-made M6(iv) channel **outside** the accepted program | *Debug-only shadow state* as a build mode (M6(iv) forbids it); *Dual acceptance and acceptance-set diffs* as a migration path — the acceptance-set diff against today's borrow rules is a rewrite, and M11's loss statement says so |

---

## 6. Owner decisions

Ordered by how much each moves the **selected** model. Every one is a
disagreement at least one judge said the debate cannot settle.

| # | Question | Options and what each costs | Effect on the selection |
|---|---|---|---|
| **O3** | Is a **ghost** generation — never materialized, never compared at run time, used only to key a static liveness fact — the mechanism M2(ii) refuses by name? (J1 #3) | (a) Clean: G1 is admissible; R1 met, M2 met, P3 and P13 accepted as required. (b) Refused: the model reverts to the base's residue — a stale handle in a reusing pool reads a type-correct wrong occupant, `pool[h]` carries an undischarged bounds arm, and the owner is choosing **append-only pools with unbounded growth** as doctrine | **Decides the model.** Everything in §1.2's G1 row depends on it |
| **O2** | Does M2(ii) refuse a bounds or occupancy compare wrapped in a total `pool[h] -> Option<T>`? (J1 #2, J2 #1) | (a) The base's test — "both arms safe, both written, deleting the branch gives a wrong program not an unsafe one": the compare is admitted and G1 becomes optional. (b) The critiques' test — "the check's condition is a safety predicate the checker did not discharge": the compare is refused and **G1 is mandatory** | With O3(a) the question is moot (there is no compare). With O3(b) this decides whether the base is M2-violated |
| **O4** | May a handle carry its pool as a type parameter — does that **reopen** D1 or **refine** it? (J2 #2) | (a) Refines: one parameter on one nominal; R15 keeps its floor everywhere else; D6 reopens for pool parameters only. (b) Reopens: C1 and C3 return, R15's parameter count leaves its floor, and the D1 family choice must be re-run | Decides §5's D6 row and R15's grade (Disputed D-6) |
| **O1** | Is a stale handle into a reusing pool an **R1(a)(ii)** violation (removal ran the node's affine release, so the storage ended, and address reuse never revives it) or an **M3(d)** logic error the constitution leaves to the requirement author? (J1 #1, J3 #1) | (a) R1(a)(ii): G1 is mandatory and the base's §6 answer 1 is argued on a ground R1(a) does not grant. (b) M3(d): G1 is a capability choice, not a safety repair, and P3's removal clause is simply not delivered | Decides whether G1 is required or optional, and decides P3 for **every** pooled candidate |
| **O17** | Is `Pool`/`Arena` a WF library or an enumerated M3 entry? (J1 #11, J3 #7) | (a) Library, derived under the tuple's own rules, plus **one** enumerated entry for the byte-extent `split` primitive (the position taken here). (b) Fully enumerated: its occupancy bookkeeping and free list become "bookkeeping state the source did not bind" (M2(ii)) unless the entry's obligation covers them. (c) Neither, as today: **nothing pooled is checkable** | Six candidates lean on it and none derived it; this is "the single most-repeated defect across all twenty-four critiques" |
| **O5** | Allocation failure, made concrete (owner decision 5). Three candidates write `alloc(A,64)?` while the settled interim places heap exhaustion outside the outcome model under R14(v) | (a) Typed outcome on every allocation — whole-corpus rewrite; R2 arms multiply (C15). (b) Abort with a record, outside the model (the live interim) — then **P2's and P18's derivations rest on a refused spelling**. (c) Distinguish: *arena* exhaustion is a condition the writer's own free list decides and is an ordinary library outcome; *heap* exhaustion stays under R14(v) — the position §2's P2 takes | P2's and P18's derivations. The selection assumes (c); if the owner picks (b), P2's arms are rewritten |
| **O11** | Are condition terms over captured branch conditions (**C6**) admitted at joins? (J2 #3, J3 #3) | (a) No, premises only (majority; the position taken here) — P8's naive shape needs a restructuring; M10 has no data-dependent term to bound. (b) Yes, restricted to the syntactically enclosing guard (J3) — P8's naive shape is accepted and P4/P9 gain precision, at an unbounded condition-atom growth nobody has bounded. **No candidate found a third answer**; four refused C6 and needed it within two programs, three adopted it and paid a representation doubling per diamond | Disputed D-1. Decides P4, P8 and P9 together |
| **O6** | Fix **K**, the tokenizer, the per-file fraction `f` and the ownership subsystem's share of K (owner decision 16). (J1 #5, J2 #7, J3 #6) | (a) Owner fixes each now — M8 becomes decidable and the writer-cost axis can be settled by evidence. (b) Delegate a derivation rule per threshold. (c) Leave provisional (the live interim) — **M8 cannot discriminate between any candidate**, independent counts ran from 11 to 50, and the selection's own count of 13 is unfalsifiable | M8's grade. It is judge 1's heaviest axis and it currently rests on a threshold nobody has set |
| **O9** | May the emitter re-derive the per-load identity from the checker's resolved place **at emission time**? (J1 #8, J2 #6) | (a) Strict reading of R4's price ("forbids *requiring* re-derivation") — the projection block must be the scope-decl site (G4), and the whole-function plane scopes need a checked-place→IR-access map that nobody has priced. (b) Loose reading — the retired `scoped-alias-channel`'s architecture is acceptable and the base's M9 claim stands as-is | R4's price line, M9's grade, and whether G4 is a requirement or a convenience |
| **O7** | Is **second-class projection** an acceptable permanent capability boundary — no stored iterator, no stored callback holding a projection, no `pick` returning a keepable reference, no cursor outliving its container's scope? (J1 #6) | (a) Accept: the model stands and P4's first conjunct is permanently partial. (b) Refuse: the best base changes, and `window-focus`'s two-level split becomes the right shape for the same problem | Not a defect and not a checker question. It is the selection's largest *capability* cost |
| **O8** | May R4's distinctness ever be grounded in the **type discipline** ("two distinct live non-copy values are disjoint") or in **freshness at a minting site**, rather than in a proved relation over a containment structure? R4(b) forbids inequality of identity *names* and is silent on type-level affinity (J1 #7) | (a) Never: G5 is mandatory and `NoReach(T)` must be a checked predicate whose conclusion is re-derived from containment. (b) Type-level affinity is admissible: the base's `AFF` table stands and G5 is optional — but two `dup`'d `File` wrappers over one pool row are then a licensed race | Decides P2 and P5 for half the candidate set, and whether G5 is a repair or a preference |
| **O16** | May a callee's exit state be a **two-armed conditional** (the conditional reallocation contract)? R8a is red for exactly this (J3 #4) | (a) Yes: three candidates need it and cannot represent it. (b) No: `push` is split in two operations (M4 cost) or becomes a typed outcome (M2(i) cost). (c) The position taken here — express it as an `ensures` over `cap` rather than as a state, which works **because there is no state to be two-armed about** | Confirms R8a met in the selection; reopens it if the owner wants states |
| **O13** | Is **R8b** in scope at D1–D4? (J2 #5) | (a) In scope: G3 answers it and the selection's R8b is met; the ranking changes materially for every candidate that deferred it. (b) Out of scope: eleven candidates should stop claiming it and G3 is bought for distinctness alone | R8b's grade and G3's justification |
| **O15** | May `use <fact>` be a bare assume on unrelated runtime scalars, or must it be a **named rule application with checked premises**? M3(a) refuses the first by name (J3 #2) | (a) Bare assume: four candidates' per-statement writer costs are right and M3 is breached. (b) Named application (the position taken here — `INV` verifies a written instantiation of a *written* invariant): four candidates' writer-cost tables are wrong, and the only route for an undischargeable goal is a written branch plus an R12 arm, which makes R12's red status load-bearing | `INV`'s definition and every pool-invariant cost in §3 |
| **O18** | Does R5's **non-erasable** direction (spawn/join) reject, or degrade to sequential like an erasable construct? (J3 #8) | (a) Rejects (owner decision 2's interim). (b) Degrades — several candidates state a blanket "a failed derivation leaves the construct sequential, it never rejects", which silently retunes C2 and C16 | R5's grade once a thread construct exists |
| **O10** | May the acceptance-visible artifact be **machine-authored**? R7(d) leaves authorship open; M1 forbids precision selecting acceptance (J1 #9, J2 graft 8, J3 #5) | (a) No: infer-and-pin lives strictly outside acceptance (Disputed D-4's position; M1(b) permits it in terms). (b) Yes: `whole-program-pinned` has the strongest writer-cost claim in the set, and the tool becomes the acceptance relation | A fork in the road for the project, not a defect in one candidate |
| **O12** | May a fine, span-scoped name appear **in a type**? (J2 #4) | (a) Never (the position taken here): stored cursors over non-pooled containers, `get_mut`, `split_at_mut` across a call and element-yielding iterators are all lost. (b) Yes: `window-focus`'s F2 shows the escape must then be governed by a scope discipline on bindings, and M4/M8 pay for a second name kind | The same boundary as O7, asked at the type level |
| **O14** | How much weight does an **undeclared** violation carry against a **declared** one? (J2 #9) | (a) As the debate's rules stand, the honest candidate scores worse: `brand-context` declares M10 violated and is penalised, `whole-program-pinned` claims 0 violated with M1, M2, M3 and M10 broken undeclared. (b) Re-weight | Not a model question; a judging-procedure question for the next debate |
| **O19** | *Meta-finding (J1 §7, all three judges' critiques converge).* The critiques converged on three places across almost every candidate: **the rules that create and destroy storage**; **the rules that split and rejoin state**; and **the unstated bridge between an identity and the runtime values that determine it** (`len`, `cap`, a bump offset, a generation, a slot index). Nine of twelve candidates lose at least one program to the third | Whatever tuple is selected, those three should be **written out in full before the tuple is defended** — because on this evidence a tuple's stated verdicts are not a reliable guide to what its rules actually admit | The selection's §1.3 writes out the first (the operations table) and the second (the join rule); the third is the `Live(h)` / `old()` / plane-measure bridge, which is exactly where G1, P-a and P-b land |
