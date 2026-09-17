# Evidence: the second core-model round, 2026-09-16

The three rule sets, nine derivation files and nine critiques (soundness,
cost, cross-examination per candidate) behind
[COMPARISON-CORE2.md](COMPARISON-CORE2.md). Agent output reviewed by the
primary agent; evidence, not decisions. Remove with the comparison it supports.


# File: rules-value-semantics.md

# Rules in full: candidate `value-semantics`

Key: `value-semantics`. Research date: 2026-09-16. This file writes out, in
full, the three rule sets VERDICT-CORE §6 O19 names as the places where every
candidate's stated verdicts stopped predicting what its rules admit: the rules
that **create and destroy storage** (set I), the rules that **split and rejoin
state** (set II), and the **bridge between an identity and the runtime values
that determine it** (set III).

Sources taken as given and not re-argued:

| Source | What is taken |
|---|---|
| [VERDICT-CORE.md](../../whitefoot-access-effects-research/research/investigations/access-effects/VERDICT-CORE.md) §1 | the selected model: value bindings; `let`/`inout`/`sink`; second-class projections; `Pool<P,T>`/`Handle<P,T>` with ghost slot and generation; the plane map; the frame rule with two-part footprints; `BIND` conjunction at joins; path conditions as premises only |
| VERDICT-CORE §1.2 | grafts **G1**–**G6** and repairs **P-a**–**P-d**, all included |
| [VERDICT-D0.md](../../whitefoot-access-effects-research/research/investigations/access-effects/VERDICT-D0.md) §2, §7 | R1–R15, M1–M11; every owner interim in §7 settled |
| [MECHANISM-MAP.md](../../whitefoot-access-effects-research/research/investigations/access-effects/MECHANISM-MAP.md) §3 | the hazard ladder and its comment style |
| [PROGRAMS.md](../../whitefoot-access-effects-research/research/investigations/access-effects/PROGRAMS.md) | P1–P19 |
| [CASES.md](../../whitefoot-access-effects-research/research/investigations/access-effects/CASES.md) | S01–S07, B01–B13, L01–L06 (appendix B) |

Every rule left unstated by the verdict and supplied here is marked
**[stated here]**. Nothing in this file selects, compares or ranks a candidate.
No rule is a recommendation; each is what this tuple must say for its own
derivations to follow.

Comment style throughout, from MECHANISM-MAP §3:
`// accept: <fact used>` and `// reject: <missing fact>`.

---

## 1. The model on one page

### 1.1 Names

There is no pointer type. Three forms replace it.

| Form | Spelling | Denotes | Span | May be stored? |
|---|---|---|---|---|
| value binding | `let x: T`, `var x: T` | storage of type `T`, existing exactly while the name is bound | lexical | it *is* the storage |
| projection (second class) | `f(inout c.a[i])`, `with inout v.data[lo..hi] as s { … }` | the subtree at a **resolved path**, for the span of the call or block | the call or the block; also the alias fact's scope (G4) | never: not stored, not returned, not captured by an escaping value |
| branded handle | `h: Handle<P,Node>` | a row key of the pool whose identity parameter is `P`; one machine word, ordinary copyable data | none — it is data | yes, freely |

### 1.2 The plane map (G3)

A declared backing's **plane map** is both its layout declaration and its
distinctness proof. Struct fields are planes of a one-element backing; there is
no second field-path mechanism.

```text
backing Cols       { a,b,c,d,e,f,g,h : plane<u64, len n, align 64> }
backing Vec<T>     { meta  : plane<Meta,1>,     data  : plane<T, len cap> }
backing Pool<P,T>  { meta  : plane<PoolMeta,1>, slots : plane<Slot<T>, len cap> }
backing Arena      { meta  : plane<Hdr,1>,      bytes : plane<u8, len N> }
```

The **containment tree** is `root → plane → extent → element`. It is the one
distinctness relation (C2).

### 1.3 What the checker holds

At every program point the checker holds exactly five components. Nothing else
is carried; in particular there is **no per-storage state** and no `ite` term.

| Component | Form | Joined at a merge by |
|---|---|---|
| `Σ` | `Name → {bound, unbound}` — the `BIND` map | conjunction (II-2) |
| `Γ` | a set of pairs `(fact, support)`, `support` = a finite set of resolved paths | intersection (II-3) |
| `Π` | an ordered list of **premises** in scope: `requires` terms, the syntactically enclosing branch conditions, and match-arm discriminants | discarded at the join (II-5); never joined, never a term in `Γ` |
| `G` | per handle name `h` in scope, at most the two ghost facts `σ(h) ∈ Live(P)` and `gen(P[σ(h)]) = γ(h)` | intersection (II-4) |
| `T` | the type table: plane maps, `NoReach`, pool brands. Immutable | — |

A **fact** in `Γ` is an ordinary value refinement (`i < len(v)`, `d != 0`,
`is_some(s)`, `σ(h) < cap(p)`). Its **support** is the set of resolved paths
whose contents it depends on. Support is what the frame rule reads.

### 1.4 What the writer writes

| Site | Written | Count |
|---|---|---|
| type declaration | the plane map; the per-backing plane invariant (III-2) | one line per plane; one invariant per backing |
| signature | one convention per parameter (`let`/`inout`/`sink`); `requires`/`ensures` value refinements with `old()`; `origin { … }` on a yielded projection; the pool identity parameter; a closure's capture clause | = parameters, + 0 |
| nominal pool or arena | one `invariant` per pool/arena type | once per type |
| statement | `sink` at a consuming transfer; `with … as` when one projection spans several statements; `use <Inv>(args)` where a written invariant must be instantiated | 0 in the common case |
| loop head | **nothing for memory safety**, except one liveness clause per pool the body frees into (II-7) | 0 or 1 |
| join | nothing | 0 |

### 1.5 What is erased

| Erased entirely | Survives as data the source declared |
|---|---|
| `σ`'s ghost status, `γ`, `gen(P[·])`, `Live(P)` | the handle's `idx` word (III-6) |
| every `use` step, every footprint, every `PATH`/`EXT`/`INV` derivation | `Option` discriminants |
| every premise in `Π` | free-list heads, arena bump offsets, `len`, `cap` |

Lowering: `let`/`inout`/`sink` → `readonly` / `noalias` / `byval`+`sret` with
`memory(argmem: …)`; a binding's initialization edge → `initializes((0,n))`,
never derived from a write convention; plane entries → one `!alias.scope` per
plane on every loaded data pointer; a `with … as` head →
`llvm.experimental.noalias.scope.decl` (G4), duplicated on unroll.

### 1.6 Rule form

Every rule below is written as

```text
premises: <what must hold in Σ, Γ, Π, G, T>
effect:   <the new Σ, Γ, Π, G>
```

followed by a 3–8 line example. `FP(s)` is the statement's **footprint**: a set
of `(resolved path, convention)` pairs. `FRAME(Γ, FP)` is defined once, in
II-12, and used by every rule that names it.

---

## 2. Rule set I — storage creation and destruction

23 rules. Every operation that creates, ends, replaces, relocates or reuses
storage is here; an operation not in this set does none of those things.

### I-1 Binding formation

```text
premises: the initializer expression is checked; its value has type T
effect:   Σ(x) := bound;  storage of type T exists from here to x's unbinding;
          Γ ∪= the initializer's own refinements, support {x}
```

There is no `Uninit`. A binding is initialized where it is bound; R1(a)(i) has
no instance by construction.

```text
var s: Option<u64> = None            // accept: formation is total; s holds a valid Option
let n: u64 = read_u64()              // accept: initialized at the binding site
let t: Cols                          // reject: missing initializer — a binding with no value
                                     //   is not representable (there is no Uninit state to enter)
s = Some(7)                          // accept: ordinary assignment on a bound var (I-7)
```

### I-2 Aggregate and backing formation

```text
premises: for `backing B { p1 : plane<T1,e1>, … }`, every plane extent e_i is a
          type-level constant or a measure path (III-1); alignments are declared
effect:   T gains B's plane map; the containment tree gains root(B) → p_i;
          PATH treats distinct planes of one backing as disjoint, with no proof
```

```text
backing Cols { a,b,c,d,e,f,g,h : plane<u64, len n, align 64> }
let mut k: Cols = Cols::zeroed(n)    // accept: one formation; the eight planes exist together
k.a[0] = 1; k.b[0] = 2               // accept: PATH(k.a[0], k.b[0]) refuted by the plane map (G3)
                                     //   no annotation, no proof, sound for any element type
let alias_of_a = k.b                 // reject: a plane is not a value; there is no plane-valued
                                     //   binding, so a second long-lived name for k.a cannot exist
```

### I-3 Scope exit

```text
premises: none
effect:   for every name x still bound at the closing brace, in reverse binding
          order: Σ(x) := unbound; x's affine release runs here;
          Γ := { (f,S) ∈ Γ | no p ∈ S is rooted at x }
```

The release is **unconditional** on this edge. No flag is consulted, because
II-2 has already made "consumed on some paths only" unrepresentable.

```text
{
  let f = open("a")                  // accept: I-1
  use(let f)                         // accept: BIND(f)
}                                    // accept: f's release runs here, on every path through
                                     //   this brace, with no drop flag (M2(ii))
use(let f)                           // reject: f is not in scope — the name does not resolve
```

### I-4 Explicit end (`sink x` as a statement)

```text
premises: Σ(x) = bound; no projection of any path rooted at x is open (II-16)
effect:   Σ(x) := unbound; x's affine release runs here;
          Γ := { (f,S) ∈ Γ | no p ∈ S is rooted at x }
```

```text
let b = alloc_buf(64)                // accept: I-1
sink b                               // accept: BIND(b); release runs at this statement
sink b                               // reject: BIND(b) = false — "b was consumed at line 2".
                                     //   Double release is refused by the binding map, not by
                                     //   a runtime count (R2)
let c = alloc_buf(64)                // accept: a FRESH identity, even where its bytes are b's
```

### I-5 Move into a call, and return

```text
premises: Σ(x) = bound; the parameter's convention is `sink`
effect:   Σ(x) := unbound; the callee's parameter binding is created (I-1);
          Γ := { (f,S) ∈ Γ | no p ∈ S is rooted at x }
          bytes may move; the move is unobservable because no projection
          outlives the statement that made it (II-19)
```

Named-return-value elision and in-place `inout` are **lowering rules**, not
optimizations: they are what keep R13 and M2(ii) true under value semantics.

```text
fn consume(sink v: Vec<u64>) -> u64 { len(v) }
let v = Vec::with_capacity(8)        // accept: I-8
let n = consume(sink v)              // accept: BIND(v) := false at the call
let m = len(v)                       // reject: BIND(v) = false — "v was consumed at line 3";
                                     //   classified R1(a)(ii), citing line 3 as the ending event
return n                             // accept: n is Copy; nothing is ended
```

### I-6 Move into another storage

```text
premises: Σ(x) = bound; the destination path d resolves (III-3);
          the conflict table (P-c) permits an `inout` access at d
effect:   Σ(x) := unbound; FP = { (d, inout) }; Γ := FRAME(Γ, FP);
          Γ ∪= refinements the assignment establishes, support {d}
```

```text
var slot: Option<Buf> = None
let b = alloc_buf(64)                // accept: I-1
slot = Some(sink b)                  // accept: BIND(b) := false; slot's contents facts killed
read(let b)                          // reject: BIND(b) = false — the value now lives in `slot`
match slot { Some(x) => sink x, None => () }
                                     // accept: both arms discharge R2 (C15); the None arm
                                     //   releases nothing because it holds nothing
```

### I-7 Assignment-over and `replace`

```text
premises: the destination path d resolves; conflict table permits `inout` at d;
          for `replace(inout d, sink new)`: Σ(new) = bound
effect:   storage identity at d is UNCHANGED (no storage ends, none is created);
          Σ(new) := unbound; FP = { (d, inout) }; Γ := FRAME(Γ, FP);
          the returned old value becomes a new binding (I-1)
```

`take` is `replace(inout d, None)` and is **total** on `Option<T>`: the checker
does nothing. There is no hole and no language state to enter.

```text
var s: Option<T> = Some(t)
let v = replace(inout s, None)       // accept: total on Option<T>; s still holds a valid Option
let r = read(let s)                  // accept: returns None. The case's "hole" is a discriminant
                                     //   the writer declared; the read is safe on both arms
let old = replace(inout s, Some(sink v))
                                     // accept: one exclusive access at path s; BIND(v) := false
sink old                             // accept: the returned Option is affine and must be consumed
```

### I-8 Anonymous backing formation (heap value)

```text
premises: the requested capacity is representable (R11 at the site)
effect:   Σ(x) := bound; the backing is ANONYMOUS — no name reaches it except
          through paths rooted at x; Γ ∪= cap(x) = c, len(x) = 0, support {x.meta}
```

Because the backing is anonymous and no projection outlives its statement, a
later reallocation of it is unobservable (I-9), and R8a's "interior pointer
into relocatable storage" has no instance.

```text
let v = Vec::<u64>::with_capacity(8) // accept: cap(v) = 8, len(v) = 0, support {v.meta}
let raw = backing_of(v)              // reject: `backing_of` is unwritable — a backing is not a
                                     //   value and has no name (R8a met by construction)
push(inout v.data[len(v)], inout v.meta, 1)
                                     // accept: I-9
let c = cap(v)                       // accept: cap(v) survives if its support was not written
```

### I-9 Vector `push` — append, with reallocation invisible

```text
signature: fn push<T>(inout v.data[len(v)], inout v.meta, e: T)
           ensures len(v) == old(len(v)) + 1 && cap(v) >= len(v)
premises:  BIND(v); the two-part footprint (P-a) resolves
effect:    FP = { (v.data[len(v)], inout), (v.meta, inout) }
           Γ := FRAME(Γ, FP);  then OLDCHAIN applies the ensures (II-14)
```

There is **one** rule for push: the reallocating and non-reallocating cases are
not distinguished, because nothing in the language can observe the difference
(O16(c): the contract is an `ensures` over `cap`, not a two-armed exit state).

```text
let a = v.data[3]                    // accept: REF from a written i < len(v)
push(inout v.data[len(v)], inout v.meta, 7)
                                     // accept: footprint names v.meta and the appended slot.
                                     //   The fact `3 < len(v)` IS killed (its support is v.meta)
                                     //   and re-derived: 3 < old(len(v)) /\ len = old(len)+1 (II-14)
let b = v.data[3]                    // accept: 3 < len(v) after the old()-chain
let c = v.data[len(v)]               // reject: missing `len(v) < len(v)`; the appended slot is
                                     //   at index old(len(v)), which is len(v)-1 here
```

### I-10 Vector `reserve` — capacity change only

```text
signature: fn reserve<T>(inout v.meta, m: u64)  ensures cap(v) >= m && len(v) == old(len(v))
premises:  BIND(v)
effect:    FP = { (v.meta, inout) };  Γ := FRAME(Γ, FP);  OLDCHAIN applies the ensures
```

The data plane is **not** in the footprint, so element contents facts survive
by the rule (G2). This is the case the base could not state and P9 turns on.

```text
let x = v.data[3]                    // accept: REF
reserve(inout v.meta, 128)           // accept: footprint is {v.meta} only
let y = v.data[3]                    // accept: `x == v.data[3]` survives BY THE FRAME RULE —
                                     //   its support is v.data[3], which reserve does not name
let n = len(v)                       // accept: len(v) == old(len(v)) restores the killed measure
sink v                               // accept: I-4
```

### I-11 Vector insert at an index — the shifting case

```text
signature: fn insert<T>(inout v.data[i..len(v)+1], inout v.meta, i: u64, e: T)
           requires i <= len(v)   ensures len(v) == old(len(v)) + 1
premises:  BIND(v); REF proves i <= len(v)
effect:    FP = { (v.data[i .. len(v)+1], inout), (v.meta, inout) }
           Γ := FRAME(Γ, FP);  OLDCHAIN applies the ensures
```

The footprint is an **extent**, not the whole plane: facts about `v.data[j]`
for a proved `j < i` survive.

```text
let lo = v.data[0]                   // accept: REF 0 < len(v)
let hi = v.data[9]                   // accept: REF 9 < len(v)
insert(inout v.data[5..len(v)+1], inout v.meta, 5, e)
                                     // accept: i <= len(v) from requires
let lo2 = v.data[0]                  // accept: `lo == v.data[0]` survives: EXT refutes
                                     //   overlap of {0} with [5, len+1)
let hi2 = v.data[9]                  // reject: EXT cannot refute 9 ∈ [5, len(v)+1);
                                     //   payload: missing `9 < 5` at v.data[9]
```

### I-12 Vector remove / pop

```text
signature: fn remove<T>(inout v.data[i..len(v)], inout v.meta, i: u64) -> T
           requires i < len(v)   ensures len(v) == old(len(v)) - 1
premises:  BIND(v); REF proves i < len(v)
effect:    FP = { (v.data[i .. len(v)], inout), (v.meta, inout) }
           Γ := FRAME(Γ, FP);  OLDCHAIN applies the ensures;
           the returned element becomes a new binding (I-1) carrying its obligations
```

```text
let e = remove(inout v.data[0..len(v)], inout v.meta, 0)
                                     // accept: 0 < len(v) from requires; e is bound here
sink e                               // accept: the removed value's obligation is discharged
let n = len(v)                       // accept: n == old(len(v)) - 1 by OLDCHAIN
let z = v.data[n]                    // reject: missing `n < len(v)`; n IS len(v) here
                                     //   payload: the two facts n == len(v) and n < len(v) conflict
```

### I-13 Vector `drain` / `clear`

```text
signature: fn drain<T>(inout v.data, inout v.meta) -> Vec<T>  ensures len(v) == 0
premises:  BIND(v)
effect:    FP = { (v.data, inout), (v.meta, inout) };  Γ := FRAME(Γ, FP);
           Γ ∪= len(v) == 0, support {v.meta};  the result is a new binding (I-1)
```

"Leaves `x`'s slot empty" is an **exit refinement over ordinary data**, not an
exit state on a storage. The whole `v.data` plane is named, so every element
fact dies — correctly, and the writer can compute that from the footprint.

```text
let x = v.data[3]                    // accept: REF
let d = drain(inout v.data, inout v.meta)
                                     // accept: judged from the signature alone (R7)
let y = v.data[3]                    // reject: missing `3 < len(v)`; len(v) == 0 from ensures.
                                     //   `x == v.data[3]` was killed: v.data is in the footprint
sink d                               // accept: the drained vector is affine
```

### I-14 Arena declaration and its written invariant

```text
premises: the arena's plane map is declared (I-2); the writer writes ONE
          invariant per arena type, verified against the arena's own body
effect:   T gains the invariant; INV may instantiate it at a written `use` step
```

```text
backing Arena { meta : plane<Hdr,1>, bytes : plane<u8, len N> }
                                     // Hdr = { top : u64 }
invariant IA : forall b, b' issued and live.
                 b != b' => extent(b) disjoint extent(b')
               && forall b issued and live. extent(b) ⊆ [0, A.meta.top)
                                     // accept: one written invariant, 3-8 use steps, once per
                                     //   arena implementation. Verified once against the body
```

### I-15 Arena carve (`alloc`)

```text
signature: fn alloc(inout A.meta, carve A.bytes, n: u64) -> Option<Buf<u8>>
premises:  BIND(A); n is representable
effect:    FP = { (A.meta, inout), (A.bytes, carve) }
           Γ := FRAME(Γ, FP)
           on the Some arm: a fresh binding b (I-1) whose extent is a SUB-EXTENT
             of the plane A.bytes — b is a DESCENDANT of A in the containment
             tree, so PATH refuses distinctness of A and b (R4(e)'s negative test)
           on the None arm: nothing is created; every obligation in scope must
             still be discharged on that arm (R2, C15)
```

`carve` is the third convention-like marker on a footprint and is not a fourth
parameter convention: it means "this plane is the source of a fresh extent",
and it kills nothing outside the extent it yields **[stated here]**.

```text
match alloc(inout A.meta, carve A.bytes, 64) {
  None     => { return Err(Exhausted) }
                                     // accept: a written arm; nothing is held yet, so R2 is
                                     //   discharged vacuously here (this arm is O5(c)'s: arena
                                     //   exhaustion is the writer's own free list deciding)
  Some(b1) => {
    match alloc(inout A.meta, carve A.bytes, 64) {
      None     => { free(inout A.meta, sink b1); return Err(Exhausted) }
                                     // accept: this arm DISPOSES b1 — R2 on every arm (C15)
      Some(b2) => { /* … */ }        // accept: extent(b1) disjoint extent(b2) by IA at (b1,b2)
    }
  }
}
```

### I-16 Arena block free

```text
signature: fn free(inout A.meta, sink b: Buf<u8>)
premises:  Σ(b) = bound; no projection of b is open
effect:    Σ(b) := unbound; FP = { (A.meta, inout) }
           Γ := FRAME(Γ, FP) then drop every fact rooted at b;
           b's extent returns to the arena's residue; NO coalescing rule exists
```

```text
free(inout A.meta, sink b2)          // accept: BIND(b2) := false; extent returns to the residue
let b4 = alloc(inout A.meta, carve A.bytes, 32)?
                                     // accept: a FRESH extent identity, even where its bytes
                                     //   are b2's. Address reuse never revives an ended identity
fill(inout b2)                       // reject: BIND(b2) = false — "b2 was consumed at line 1",
                                     //   classified R1(a)(ii). This is P2's required refusal
free(inout A.meta, sink b4)          // accept: each name bound exactly once
```

### I-17 Byte-extent `split` — the one enumerated M3 entry

```text
signature: fn split(sink b: Buf<u8>, at: u64) -> (Buf<u8>, Buf<u8>)
           requires at <= len(b)
           ensures  extent(result.0) ++ extent(result.1) == extent(old(b))
                 && extent(result.0) disjoint extent(result.1)
premises:  Σ(b) = bound; REF proves at <= len(b)
effect:    Σ(b) := unbound; two fresh bindings (I-1) with the ensures as facts
boundary obligation: the entry's disjointness ensures is discharged against the
           byte semantics of the target, not against WF source (M3)
```

There is **no rejoin**. HIS-D9-12 (split-and-join range tokens) stays refused,
so a coalescing arena is out of the language, not merely unwritten.

```text
let b = alloc(inout A.meta, carve A.bytes, 64)?
let (lo, hi) = split(sink b, 32)     // accept: at <= len(b); BIND(b) := false
par { fill(inout lo); fill(inout hi) }
                                     // accept: extents disjoint from the M3 entry's ensures,
                                     //   NOT from a type-level affinity table (G5)
let rejoined = join(sink lo, sink hi)// reject: `join` is unwritable — no rule produces a fact
                                     //   relating two extents' adjacency after a split
```

### I-18 Pool declaration and its written invariant

```text
premises: the pool's plane map is declared; P is the pool's identity parameter;
          the writer writes ONE invariant per pool type
effect:   T gains the invariant and the brand P; `Handle<P,T>` is the only type
          that mentions P; every other type has zero identity parameters
```

```text
backing Pool<P,T> { meta : plane<PoolMeta,1>, slots : plane<Slot<T>, len cap> }
invariant PBounds : forall σ. σ ∈ Live(P) => σ < cap(p)
                                     // accept: this is the bridge that makes p.slots[σ(h)]
                                     //   statically total (III-6); without it the access is
                                     //   an R11 site and M2(ii) is at issue (O2)
invariant Links : forall σ ∈ Live(P). Live(slots[σ].next) && Live(slots[σ].prev)
                                     // accept: stated ONCE on the nominal, reused everywhere
```

### I-19 Pool insert

```text
signature: fn insert<P,T>(inout p.slots[σ], inout p.meta, x: T) -> Handle<P,T>
           ensures Live(result)
premises:  BIND(p); Σ(x) = bound if T is affine
effect:    a FRESH ghost slot σ ∉ Live(P);  Live(P) ∪= {σ}
           γ(result) := gen(P[σ]);  Σ(x) := unbound
           FP = { (p.slots[σ], inout), (p.meta, inout) };  Γ := FRAME(Γ, FP)
           G ∪= { σ(result) ∈ Live(P), gen(P[σ(result)]) = γ(result) }
```

The free-list pop that chooses σ is the pool library's **own written data**
(O17(a)); it is not compiler bookkeeping.

```text
let h = insert(inout p.slots[σ], inout p.meta, node)
                                     // accept: fresh σ; Live(h) from the ensures
let nd = p.slots[σ(h)].value         // accept: Live(h) => σ(h) ∈ Live(P) => σ(h) < cap(p) by
                                     //   PBounds. NO bounds compare and NO generation compare
                                     //   is emitted (M2(ii)); the access is statically total
let h2 = insert(inout p.slots[σ'], inout p.meta, node2)
                                     // accept: Live(h) survives — the footprint names slots[σ']
                                     //   and meta, and σ' != σ(h) because σ' is fresh (III-8)
```

### I-20 Pool remove

```text
signature: fn remove<P,T>(inout p.slots[σ(h)], inout p.meta, h: Handle<P,T>) -> T
           requires Live(h)
premises:  BIND(p); G contains σ(h) ∈ Live(P) and gen(P[σ(h)]) = γ(h)
effect:    Live(P) \= {σ(h)};  gen(P[σ(h)]) := succ(γ(h))
           FP = { (p.slots[σ(h)], inout), (p.meta, inout) };  Γ := FRAME(Γ, FP)
           G := { facts of k ∈ G | EXT proves σ(k) != σ(h) }        [stated here, III-8]
           the removed value becomes a new binding (I-1)
```

The last line is the rule the selection did not write. Ghost slot equality is
not decidable, so removal kills the liveness fact of **every** handle of that
pool whose distinctness from `h` is not proved. The escape is the same one P1
uses for elements: a proved index inequality, here over the `slots` plane.

```text
remove(inout g.slots[σ(m)], inout g.meta, m)
                                     // accept: Live(m) from the caller's fact
                                     //   Live(P) \= {σ(m)}; gen bumped; every `gen = γ_m` killed
let nd = g.slots[σ(stale)].value     // reject: missing Live(stale) — gen(P[σ(stale)]) = γ(stale)
                                     //   was killed at line 1. R1(a)(ii): address reuse never
                                     //   revives an ended identity
let ok = g.slots[σ(other)].value     // reject: missing `σ(other) != σ(m)` [stated here]
                                     //   repair named: a written requires on the pool interface,
                                     //   or `use Links(other)` to re-derive Live(other)
use Links(other); let ok2 = g.slots[σ(other)].value
                                     // accept: INV re-derives Live(other) from the written
                                     //   pool invariant at a written use step
```

### I-21 Pool compact

```text
signature: fn compact<P,T>(inout p.slots, inout p.meta)
           ensures forall h. Live(h) => Live(h)
premises:  BIND(p); no projection into p is open
effect:    FP = { (p.slots, inout), (p.meta, inout) };  Γ := FRAME(Γ, FP)
           Live(P) and gen(P[·]) are NOT in the footprint, so every fact in G is
           framed IN (G2). σ(h) is re-read from the handle's own word (III-6),
           which the compaction rewrote: the handle IS the fixup form
```

```text
let h = insert(inout p.slots[σ], inout p.meta, x)
let ptr = pointer_of(h)              // reject: `pointer_of` is unwritable — there is no pointer
                                     //   type, so R8a's mutual exclusion holds by construction
compact(inout p.slots, inout p.meta) // accept: the footprint names slots and meta, not Live(P)
let v = p.slots[σ(h)].value          // accept: Live(h) framed in; the slot map resolves the
                                     //   CURRENT slot. The lookup is an operation the SOURCE
                                     //   wrote, not a compiler-inserted fixup
let (q, remap) = compact_renumber(sink p)
                                     // reject at the declaration: the handles it invalidates are
                                     //   ordinary data scattered through the program and the
                                     //   checker cannot enumerate them (R7 declaration check)
```

### I-22 Foreign storage, and the foreign kill rule (P-d)

```text
premises: the storage's type is `Foreign<T>`, declared with R14(iv)'s access class
effect:   creation and end are ordinary (I-1, I-4), BUT:
          at EVERY statement s, before s is checked:
            Γ := { (f,S) ∈ Γ | no p ∈ S is rooted at, or a descendant of,
                                a binding of type Foreign<U> in scope }
          this kill is unconditional and is the ONE stated exception to
          "the footprint is the written path"
```

```text
let fc: Foreign<u8> = map_device(sink buf)
let a = fc.bytes[0]                  // accept: an ordinary read in the declared access class
wait_completion()                    // accept: a written, program-observed event
let b = fc.bytes[0]                  // accept as a READ; but the fact `a == fc.bytes[0]` was
                                     //   killed at this statement by P-d, footprint or not
assert a == b                        // reject: missing `a == fc.bytes[0]`; the agent could have
                                     //   interleaved. R14(iv)'s kill obligation has its carrier
```

### I-23 Compiler reuse of ended storage

```text
premises: a storage has ended by I-3, I-4, I-5, I-6, I-16 or I-20
effect:   the compiler MAY reuse the bytes (stack coloring, spill reuse, a pool
          free list, an arena residue). It may NOT assume ended storage is
          unwritten by others (R14(iii)). No checker fact is created or revived
          by reuse; the ghost generation (III-7) is what makes this true for
          pools and the fresh-binding rule (I-1) is what makes it true elsewhere
```

```text
{ let x = big_value(); use(let x) }   // accept: x ends at the brace (I-3)
{ let y = big_value(); use(let y) }   // accept: y may occupy x's bytes; y is a FRESH identity
                                      //   and no fact about x is available to y
sink p_handle_of_x                    // reject: `p_handle_of_x` does not resolve — a binding's
                                      //   storage has no name other than the binding
```

---

## 3. Rule set II — splitting and rejoining state

23 rules.

### II-1 Branch split

```text
premises: the condition expression c is checked and is Copy
effect:   each arm is checked in a COPY of (Σ, Γ, G);
          the then-arm's Π := Π ++ [c];  the else-arm's Π := Π ++ [!c]
          the arms are checked independently and in written order
```

```text
if k != i {
  let x = v.data[k]                   // accept: EXT's inputs include the syntactically enclosing
                                      //   branch condition AS A PREMISE (Π), together with
                                      //   k < len(v) from requires
} else {
  let y = v.data[k]                   // accept: the premise here is `k == i`, which is not the
                                      //   missing fact for a bounds check but IS available
}
let z = v.data[k]                     // accept if k < len(v) is in Γ; the premise k != i is GONE
```

### II-2 Join: `BIND` and the R2 definiteness clause **[stated here]**

```text
premises: each arm has produced Σ_i
effect:   for every name x bound at the branch point:
            if Σ_i(x) = bound for all i        -> Σ(x) := bound
            if Σ_i(x) = unbound for all i      -> Σ(x) := unbound
            otherwise                          -> REJECT at the join
          names bound INSIDE an arm are out of scope after it (I-3)
```

The third line is the clause the selection left implicit. Conjunction alone
gives `Σ(x) := unbound` in the mixed case, and then nothing releases `x` on the
arm that did not consume it — an R2 leak admitted with no diagnostic. Since no
drop flag is representable and none may be inserted (M2(ii)), the mixed case
must be a rejection. It makes the naive shape of P8 reject one line earlier
than VERDICT-CORE §2's P8 derivation shows, at the join rather than at the
later use; the accepted set is unchanged, because the later use rejected anyway.

```text
if c { sink a } else { sink b }       // reject AT THE JOIN: a is consumed on the then-arm and
                                      //   not on the else-arm. Payload: the consuming statement,
                                      //   the arm that did not consume, and the repair below
let s = if c { sink a; b } else { sink b; a }
                                      // accept: each arm unbinds one name and YIELDS the other
                                      //   as a value. Σ(a) = Σ(b) = unbound on both arms
sink s                                // accept: exactly one release on every path (R2)
```

### II-3 Join: the fact set `Γ`

**[stated here]** — the selection states the `BIND` join and is silent on `Γ`.

```text
premises: each arm has produced Γ_i, and each fact carries its support
effect:   Γ := ⋂_i Γ_i, compared by syntactic equality of (fact, support) after
          resolving paths (III-3). No fact is weakened, widened, or turned into
          a disjunction; there is no `ite` and no condition term (O11(a))
```

```text
if c { reserve(inout v.meta, 64) } else { reserve(inout v.meta, 128) }
                                      // then-arm: cap(v) >= 64;  else-arm: cap(v) >= 128
let n = cap(v)                        // accept: cap(v) >= 64 survives — it is in BOTH arms
assert cap(v) >= 128                  // reject: missing `cap(v) >= 128`; it holds on one arm only
                                      //   payload names the arm; repair: hoist the reserve, or
                                      //   write the weaker bound the join keeps
```

### II-4 Join: the ghost pool facts `G`

**[stated here]**.

```text
premises: each arm has produced G_i
effect:   G := ⋂_i G_i on the pair (σ(h) ∈ Live(P), gen(P[σ(h)]) = γ(h)), per
          handle name h. A handle bound inside an arm is out of scope.
          Live(P) is never represented extensionally (III-8), so the join has no
          set to merge: it merges a bounded per-name fact table
```

```text
if c { remove(inout g.slots[σ(h)], inout g.meta, h) } else { () }
let nd = g.slots[σ(h)].value          // reject: missing Live(h) — the then-arm killed it and the
                                      //   join is an intersection. Payload names the arm
if c { () } else { use Links(h) }
let nd2 = g.slots[σ(h)].value         // reject: missing Live(h) on the then-arm. The repair is a
                                      //   `use` on both arms, or hoisting it above the branch
```

### II-5 Path conditions are premises, never state terms (O11(a))

```text
premises: none
effect:   a branch condition c enters Π for exactly the syntactic region guarded
          by it, and is DISCARDED at the join. c never enters Γ, never becomes a
          term in a fact, and no fact is indexed by c.
          Consequence: the flow-sensitive component of the checker's state is one
          boolean per name plus a support-keyed fact set; there is no
          condition-atom growth to bound (M10)
```

```text
var cond = read_flag()
if cond { s = None }                  // accept: ordinary assignment
if cond { s = Some(20) }              // accept: ordinary assignment
let r = read(let s)                   // accept: the read is TOTAL on Option; but
assert is_some(s)                     // reject: missing `is_some(s)`. Recovering it would require
                                      //   the first arm's effect to be a term over `cond`, which
                                      //   O11(a) refuses. Repair named: write one `if cond { … }`
                                      //   containing both statements, or match on s
```

### II-6 `match` arms

```text
premises: the scrutinee path resolves; the arm set is exhaustive over the
          declared sum type (no compiler-inserted arm, no default)
effect:   each arm is checked with Π ++ [the arm's discriminant equation] and a
          copy of (Σ, Γ, G); the binder of a payload is a BINDING (I-1) when the
          scrutinee is consumed and a PROJECTION (II-16) when it is not;
          the arms rejoin by II-2, II-3, II-4
```

```text
match c.slots[k] {
  Some(v) => v                        // accept: v is a projection of c.slots[k] for this arm;
                                      //   the premise `is_some(c.slots[k])` is in Π here
  None    => { let x = compute(k); c.slots[k] = Some(x); x }
                                      // accept: one exclusive access at c.slots[k]; the premise
                                      //   `is_none(c.slots[k])` is in Π and is NOT needed
}                                     // accept: every arm discharges R2; both arms yield a V
```

### II-7 Loop head — what must be re-established, what is written

```text
premises: the loop's body has been checked once against the head state
effect:   Σ at the head is the greatest fixpoint of a HEIGHT-2 lattice: reached
          in at most two passes over a reducible CFG
          Γ at the head := Γ_entry ∩ Γ_backedge, where any fact whose support is
            written anywhere in the body is absent from Γ_backedge (FRAME, II-12)
          G at the head := G_entry ∩ G_backedge
          WRITTEN at the head: nothing for memory safety, EXCEPT one liveness
            clause per pool the body frees into (the price of G1), and whatever
            R11 `invariant` clauses the writer needs for index proofs
```

```text
var h = start
loop {
  let nd = g.slots[σ(h)]              // accept: Live(h) at the head, from the entry fact and
                                      //   preserved on the back edge (the body frees nothing)
  if nd.next == null_handle { break } // accept: the writer's own branch on the writer's own data
  use(nd.value); h = nd.next          // accept: Live(h) re-established from INV on Links
}
loop { remove(inout g.slots[σ(x)], inout g.meta, x); x = next_of(x) }
                                      // reject at the head: missing Live(x) on the back edge —
                                      //   the body frees into g. Repair named: write the head's
                                      //   liveness clause, `invariant Live(x)`, and a `use` step
```

### II-8 Loop back edge

```text
premises: the back edge's (Σ, Γ, G) has been computed
effect:   compare against the head state; a name unbound on the back edge but
          bound at the head is REJECTED, naming the consuming statement
          ("consumed in a loop"). A fact absent on the back edge is simply absent
          from the head state; it is not an error unless a body statement needs it
```

```text
loop {
  sink v                              // reject on the back edge: BIND(v) is false here and true
                                      //   at the head. Payload: "v is consumed at line 2 and the
                                      //   loop may run again"; repair: move the sink out, or
                                      //   break immediately after it
}
loop { let w = clone(let v); sink w } // accept: w is bound and consumed inside one iteration
```

### II-9 Break and other early exit edges

```text
premises: each exit edge carries its own (Σ, Γ, G)
effect:   the loop's continuation state is the join (II-2, II-3, II-4) over ALL
          exit edges, INCLUDING the zero-iteration edge. II-2's definiteness
          clause applies: a name consumed on the break edge and not on the normal
          exit edge is a REJECT at the loop's exit join
```

```text
loop { if stop { sink a; break }; read(let a) }
sink a                                // reject at the loop exit join: a is consumed on the break
                                      //   edge and bound on the normal and zero-iteration edges.
                                      //   Repair named: carry the survivor as a VALUE out of the
                                      //   loop —
let r: Option<Obj> = loop { if stop { break None }; break Some(sink a) }
match r { Some(x) => sink x, None => () }
                                      // accept: one release on every exit edge (R2)
```

### II-10 The call rule — what a signature states

```text
A declaration states, and nothing else:
  - one convention per parameter: let (read) | inout (write) | sink (ends)
  - the resolved PATHS the conventions apply to, which ARE the footprint
  - value refinements: requires / ensures, with old() in ensures
  - the outcome type (R12)
  - `origin { … }` on a yielded projection (II-20)
  - a closure parameter's capture clause (II-21)
  - the pool identity parameter on Pool / Handle
Absent: region parameters, lifetime parameters, entry/exit storage states,
        loan clauses, memory effect rows, modes.
Declaration-side check: a requires that is syntactically unsatisfiable, or a
        result naming an origin no caller can name, is REJECTED at the
        declaration (R7(a))
```

```text
fn helper(inout out: plane<u8>, let input: plane<u8>)
                                      // accept: the convention IS the footprint; nothing else
fn find<P>(let g: Pool<P,Node>) -> Handle<P,Node>  ensures Live(result)
                                      // accept: the result NAMES ITS ORIGIN through the brand P
fn find_any() -> Handle<?, Node>      // reject at the declaration: the result names an origin no
                                      //   caller can name (R7(a), the provenance case)
```

### II-11 The call rule at the caller — what is learned

```text
premises: each argument path resolves; each `requires`, instantiated with the
          actual paths and actual value terms, is discharged from Γ ∪ Π;
          the conflict table (II-17) permits the whole argument list pairwise
effect:   Γ := FRAME(Γ, FP)          (II-12, applied FIRST)
          then Γ ∪= each `ensures`, instantiated, with support = the footprint's
          inout/sink paths, and with old(m) resolved by OLDCHAIN (II-14)
```

```text
reserve(inout v.meta, 128)            // accept: no requires; FP = {v.meta}
                                      // learn: cap(v) >= 128, support {v.meta}
let s = v.data[130]                   // reject: missing `130 < len(v)`. cap is not len —
                                      //   the payload names the length fact, not the capacity
push(inout v.data[len(v)], inout v.meta, 1)
                                      // accept: learn len(v) == old(len(v)) + 1
```

### II-12 The call rule at the caller — what is killed (FRAME, G2)

```text
FRAME(Γ, FP) = { (f,S) ∈ Γ  |  ∀ p ∈ S. ∀ (q,m) ∈ FP with m ∈ {inout, sink}.
                                   ¬ PATH_may_overlap(p, q) }
where PATH_may_overlap is the containment-tree prefix test refined by EXT.
A call falsifies EXACTLY the facts whose support its written footprint names;
every other fact survives BY THE RULE, not by a kill heuristic. kills = 0 by
construction over facts the footprint does not name (R6's metric).
One stated exception: I-22's foreign kill rule (P-d), applied at every statement.
```

```text
let a = v.data[3]                     // support {v.data[3]}
let b = w.data[3]                     // support {w.data[3]}
let n = len(v)                        // support {v.meta}
push(inout v.data[len(v)], inout v.meta, 1)
                                      // accept: `b == w.data[3]` survives — w is a different root
                                      // accept: `a == v.data[3]` survives — EXT refutes overlap
                                      //   of {3} with {len(v)} given 3 < old(len(v))
                                      // killed:  `n == len(v)` — v.meta IS in the footprint
let n2 = n                            // accept: n is a Copy value, not a fact about storage
assert n == len(v)                    // reject: missing `n == len(v)`; the killer is named:
                                      //   "push at line 4 writes v.meta" (R6's diagnostic)
```

### II-13 Two-part footprints for container and pool operations (P-a)

```text
premises: the operation is declared on a container or pool
effect:   its footprint has TWO parts: the addressed slot or extent, and the
          metadata plane it touches. It is an error at the DECLARATION to name
          the container root where a slot and a plane are meant, because the root
          is a prefix of every element path and kills everything
```

```text
fn get<P>(inout c.slots[k], inout c.meta, k: Key) -> V
                                      // accept: two parts. NOT `inout c`
with let c.slots[k1] as r {
  get(inout c.slots[k2], inout c.meta, k2)
                                      // accept iff `k1 != k2` is proved: let/inout on
                                      //   non-overlapping paths is the permitted cell (P-c)
  read(let r)                         // accept: the contents fact survives because the footprint
                                      //   names slots[k2] and meta, neither overlapping slots[k1]
}
fn get_bad<P>(inout c, k: Key) -> V   // reject at the declaration: `c` is a prefix of c.slots[k1]
                                      //   so every held reader is refused unconditionally
```

### II-14 `old()`-indexed exit refinements and their chaining rule (P-b)

```text
premises: an `ensures` clause of the form  m(x) REL expr(old(m1(x)), …, args)
          where each m is a MEASURE symbol bound by the plane map (III-1)
effect:   at the call, and only at the call:
            1. bind old(m_i(x)) := the term for m_i(x) held in Γ immediately
               BEFORE the FRAME step; if no term is held, the clause contributes
               nothing (no search for one)
            2. FRAME kills the measure facts as usual
            3. add the instantiated ensures, support = the footprint
            4. for EVERY fact f killed in step 2 whose support is exactly the
               measure's path, rewrite f ONCE by substituting the new ensures'
               relation for the measure, and re-add it if the result is in the
               linear fragment. ONE rewrite per fact per call. No iteration,
               no fixpoint, no search
```

```text
let c_i_lt = c.i < len(v)             // support {v.meta}
push(inout v.data[len(v)], inout v.meta, 5)
                                      // step 2: `c.i < len(v)` is killed — v.meta is named
                                      // step 3: len(v) == old(len(v)) + 1
                                      // step 4: c.i < old(len(v)) /\ len = old(len)+1
                                      //          |- c.i < len(v)   — ONE rewrite [P-b]
let a = read_at(let p, let c)         // accept: c.i < len(v) re-derived, not implicitly survived
sort(inout v.data, inout v.meta)      // ensures nothing about len
let b = read_at(let p, let c)         // reject: missing `c.i < len(v)` — sort names v.meta and
                                      //   states no ensures over len, so step 4 has no relation
                                      //   to substitute. Repair named: add `ensures len(v) ==
                                      //   old(len(v))` to sort's signature
```

### II-15 The callee side — checked once against its signature

```text
premises: the body is checked in a state where each parameter is a binding
          (`sink`) or a projection (`let`, `inout`) of an OPAQUE root;
          Π := the `requires` clauses; Γ := the refinements they carry
effect:   the body must (a) write only inside its `inout`/`sink` footprint,
          (b) establish every `ensures` on every exit, (c) discharge R2 on every
          arm. Checked ONCE for all call sites and, under R15, once for all
          instantiations — which is why the value class lives in the bound
```

```text
fn bump(inout v.meta, d: u64)  ensures len(v) == old(len(v)) + d {
  v.meta.len = v.meta.len + d         // accept: v.meta is in the footprint; the write's overflow
                                      //   obligation is an R11 site discharged from a requires
  v.data[0] = 0                       // reject: v.data is not in this signature's footprint —
                                      //   payload names the signature line to widen
}
fn pick_n<N: Nominal + affine>(c: Bool, sink x: N, sink y: N) -> N
                                      // accept: the bound carries the VALUE CLASS, so the body
                                      //   is checked once. The linear instantiation is
                                      //   FORECLOSED, not repaired — a stated capability loss
```

### II-16 Projection open (`with … as`)

```text
premises: the projected path p resolves (III-3); every index term in p is stable
          for the block (III-3); the conflict table (II-17) permits p's
          convention against every OTHER projection open in the same span and
          against the enclosing access
effect:   a NEW NAME is bound as a synonym for the resolved path p, for the
          block's lexical span ONLY. No storage is created. The block head is
          the alias-scope declaration site (G4);
          for the span, every access to a path PATH-overlapping p must go
          through the projection name, or be refused by II-17
```

```text
with inout v.data[i] as pi, inout v.data[j] as pj {
                                      // accept: PATH(v.data[i], v.data[j]) refined by EXT;
                                      //   `i != j` from requires. [G4] the head is the
                                      //   noalias.scope.decl site; pi and pj get sibling scopes
  pi = 1                              // accept: inout/inout on NON-overlapping paths
  pj = 2                              // accept: same
  let x = v.data[k]                   // reject: PATH(v.data[k], v.data[i]) unresolved.
                                      //   payload: missing `k != i && k != j` at this location
}
```

### II-17 Projection use, and the conflict table (P-c)

For two accesses to paths whose overlap `PATH`/`EXT` does not refute, **in one
sequential span**:

| | `let` (read) | `inout` (write) | `sink` (ends) |
|---|---|---|---|
| **`let`** | permitted | refused | refused |
| **`inout`** | refused | refused | refused |
| **`sink`** | refused | refused | refused |

Non-overlapping paths: always permitted. Two overlapping **reads** are
permitted — the cell P1, P5 and P6 rely on.

```text
with let s.x as rx {
  let a = s.x                         // accept: let/let on overlapping paths is the permitted cell
  let b = s.y                         // accept: PATH refutes overlap — sibling planes (G3)
  s.x = 1                             // reject: let/inout on overlapping paths.
                                      //   payload: the two paths, the table cell, and the repair
                                      //   ("move the write outside the block")
}
s.x = 1                               // accept: the projection is closed
```

### II-18 Projection close

**[stated here]** — the selection states the open and is silent on the close.

```text
premises: the block or call ends
effect:   the projection NAME leaves scope. No "put back" is written and none is
          checked: nothing was taken out.
          Every fact in Γ derived inside the block is REKEYED: a support path
          rooted at the projection name is rewritten to the resolved path it was
          a synonym for. A fact whose support mentions a name bound inside the
          block is DROPPED.
          Facts so rekeyed survive the close; the alias scope does not
```

```text
with inout v.data[i] as pi {
  pi = 7                              // Γ gains `v.data[i] == 7` after rekeying, support
                                      //   {v.data[i]} — not {pi}
  let tmp = pi + 1                    // Γ gains `tmp == v.data[i] + 1`, support {v.data[i], tmp}
}
let z = v.data[i]                     // accept: `v.data[i] == 7` survived the close, rekeyed
assert tmp == 8                       // reject: `tmp` is out of scope — its fact was dropped
```

### II-19 Projection as a call argument

```text
premises: as II-16, with the span being the single call statement
effect:   the projection exists for the call only. No `with` is written; the
          argument expression IS the projection. This is why a relocating move
          is unobservable (I-5): no projection outlives its statement
```

```text
helper(inout out[w*s .. (w+1)*s], let input)
                                      // accept: one projection per argument; ProvedRangePartition
                                      //   discharges the extent (III-1, family EXT)
                                      // [G4] one noalias.scope.decl per iteration, duplicated
                                      //   on unroll, because the property is per iteration
let keep = projection_of(out[0..4])   // reject: a projection is second class — it cannot be
                                      //   the value of a binding (O7/O12's boundary)
```

### II-20 Yield-once accessor (`subscript … origin`)

```text
declaration: subscript name(params) -> inout T   origin { p1, p2, … }
             where every p_i is a path in the parameter list; the origin set is
             FINITE and STATIC
premises:  as II-16 for the yielded projection
effect:    for the block's span, the footprint is the UNION of the origin set —
           which is WIDER than any single written path. This is the one declared
           exception to "footprint = written path" besides P-d.
           Every fact whose support overlaps ANY origin path is killed for the
           span. At the close, II-18 applies with the origin union as the key.
lowering:  a resumption ABI (a fourth interface form beside let/inout/sink)
```

```text
subscript pick(let c: Bool, inout s: S) -> inout T  origin { s.x, s.y }
with inout pick(c, inout s) as t {
  t = 1                               // accept: for the span, facts about BOTH s.x and s.y die
  let z = s.x                         // reject: PATH(s.x, origin) overlaps; the let/inout cell.
                                      //   payload names s.x, the origin set, and the cell
  let w = s.z                         // accept: s.z is not in the origin set (G3 sibling plane)
}
let z2 = s.x                          // accept: the span is over
```

### II-21 Second-class closure with a capture clause

```text
declaration: hh : (inout plane<u8>) reads { input } writes { }
premises:  the closure value is non-escaping: it may be passed DOWN and called,
           never stored, returned, or put in a container
effect:    at a call of hh, FP := hh's parameter footprint ∪ its capture clause.
           The capture clause is part of the closure's INTERFACE, not inferred
           from the body (R7): "the closure's footprint is its parameter list"
           is false and this clause is why
```

```text
let hh = |inout slice: plane<u8>| helper(inout slice, let input)
                                      // accept: hh : (inout plane<u8>) reads { input }
for w in 0..k { hh(inout out[w*s .. (w+1)*s]) }
                                      // accept: PAR-2 over w needs `reads {input}` to know the
                                      //   iterations do not conflict on input (two reads: the
                                      //   permitted cell). Without the clause the judgment has
                                      //   no input and the loop is sequential
let stored = Box::new(hh)             // reject: a closure is second class; it cannot be stored
```

### II-22 `par` split and rejoin

```text
premises: for every PAIR of branches (or, for PAR-2, every pair of iterations
          i != i'), their footprints are judged by the conflict table (II-17)
          under R5's ground (i). A cell that is not "permitted" refuses the
          overlap, not the program (M5), at an erasable construct
effect:   each branch is checked from a COPY of (Σ, Γ, G); a name may be bound
          or consumed in at most one branch (PATH over distinct roots decides);
          at the rejoin, Γ := ⋂ of the branch results, as II-3;
          no fact derived in one branch is available in another
```

Grounds (ii) (a checked accumulator law) and (iii) (a named weaker level) are
R14's vocabulary. This tuple neither supplies nor obstructs them; P15 is not
derived here.

```text
par { fill(inout b1); fill(inout b2); fill(inout b3) }
                                      // accept: b1,b2,b3 are three carved extents of the plane
                                      //   A.bytes; disjointness is EXT over the extents,
                                      //   licensed by IA instantiated at the three pairs [G5]
                                      // accept: A is not held between operations — the fills name
                                      //   sub-extents of A.bytes, and A.meta is a sibling plane
                                      //   no fill names
par for h in g.keys() { g.slots[σ(h)].value = f(g.slots[σ(h)].value) }
                                      // accept: per-iteration footprint {g.slots[σ(h)].value};
                                      //   pairwise distinctness of what keys() yields is a WRITTEN
                                      //   clause `ensures injective(keys)` (III-9), not an
                                      //   assertion that Handle values differ
par for h in traverse(g) { g.slots[σ(h)].value = 0 }
                                      // reject: missing `distinct(σ(h1), σ(h2))` — Links gives
                                      //   liveness, not injectivity, for data-derived handles.
                                      //   repair named: iterate keys(), or write an injectivity
                                      //   invariant and a use step
```

### II-23 Token or view split and rejoin — declared absent **[stated here]**

```text
This candidate has NO token algebra and NO view fold/unfold.
  - there is no permission token that can be split and rejoined;
  - there is no view that is folded at a call and unfolded after it;
  - the only split-shaped rule is I-17, the byte-extent `split`, which is
    one-way: two carved extents can never be recombined into one name.
Consequence, stated rather than hidden: a coalescing arena, a split-and-rejoin
range token (HIS-D9-12), and any API that hands out two halves and later takes
them back as one are OUT of the language, not merely unwritten.
```

```text
let (lo, hi) = split(sink b, 32)      // accept: I-17
par { fill(inout lo); fill(inout hi) }// accept: II-22 on disjoint extents
let whole = join(sink lo, sink hi)    // reject: no rule produces the adjacency fact; `join` is
                                      //   not declarable. The named alternative is to keep the
                                      //   parent `b` and project two sub-extents with `with`,
                                      //   which rejoins at the block close (II-18) instead
with inout b.bytes[0..32] as lo2, inout b.bytes[32..64] as hi2 { fill(inout lo2); fill(inout hi2) }
                                      // accept: the block close IS the rejoin; b was never split
```

---

## 4. Rule set III — the identity-to-runtime-value bridge

13 rules. This is the set VERDICT-CORE §6 O19 says nine of twelve candidates
lost a program to. For each runtime value a fact or an identity depends on: the
rule connecting the checker's static name to the value, how the fact is
established, when it is killed, and how it is re-derived.

| Runtime value | Static name | Rule |
|---|---|---|
| `len`, `cap` | a measure symbol bound to a path | III-1, III-2 |
| an element index `i` | a term in a resolved path | III-3 |
| an arena bump offset | `A.meta.top`, ordinary data | III-4 |
| a free-list head | `p.meta.free_head`, ordinary data | III-5 |
| a slot index `σ(h)` | the handle's own `idx` word | III-6 |
| a generation `γ(h)`, `gen(P[σ])` | ghost only, never materialized | III-7 |
| the live-slot set `Live(P)` | never extensional | III-8 |
| iterator key distinctness | a written `ensures` | III-9 |
| a branch condition | a premise in `Π` | III-10 |
| an `Option` discriminant | writer data, read by an arm | III-11 |
| foreign contents | no bridge exists | III-12 |
| `old(m)` | the pre-state term of a measure | III-13 |

### III-1 Measure symbols

```text
rule:        a plane declared `plane<T, len m>` binds the MEASURE SYMBOL m to a
             resolved path μ(m) inside the backing's metadata plane. `len(v)` is
             not a function the checker computes: it is a NAME for the runtime
             value at μ(len, v) = v.meta.len.
established: at formation (I-8), from the formation's own ensures
support:     every fact mentioning m(v) carries μ(m,v) in its support
killed:      when a footprint names μ(m,v) — i.e. when v.meta is written
re-derived:  by OLDCHAIN (II-14) from an ensures of the writing operation
```

```text
backing Vec<T> { meta : plane<Meta,1>, data : plane<T, len cap> }
                                      // Meta = { len : u64, cap : u64 }
let n = len(v)                        // accept: n names the value at v.meta.len; support {v.meta}
v.data[0] = 9                         // accept: FRAME keeps `n == len(v)` — the footprint is
                                      //   v.data[0], and v.meta is a SIBLING PLANE (G3).
                                      //   This is why an element write cannot kill a length
v.meta.len = 0                        // accept as a write; `n == len(v)` is killed here
```

### III-2 Plane extent, and the per-backing plane invariant

```text
rule:        the data plane's extent is the runtime value cap(v). An element
             access v.data[i] is in the plane's domain iff i < cap(v).
             The writer writes ONE invariant per backing relating the measures:
                 invariant VecOk : len(v) <= cap(v)
             verified once against the backing's own operations
established: the invariant at the type; the index fact at the access site
killed:      the invariant is NEVER killed (it is a type-level fact over the
             backing's own operations); the index fact is killed by III-1's rule
re-derived:  `i < len(v) /\ len(v) <= cap(v) |- i < cap(v)` at every access
```

```text
invariant VecOk : len(v) <= cap(v)
let y = v.data[i]     requires i < len(v)
                                      // accept: R11's domain fact is i < cap(v), discharged from
                                      //   the requires and VecOk. The writer wrote one of the two
let z = v.data[j]     requires j < cap(v)
                                      // accept for the DOMAIN (R11) ...
                                      // reject for the CONTENTS (R1): two separately named
                                      //   diagnostics [G6] — (1) R11 domain: satisfied;
                                      //   (2) R1 state: missing `j < len(v)`, so the slot holds
                                      //   no element the program wrote
```

### III-3 Index terms, path resolution, and index-term stability **[stated here]**

```text
rule:        a path step `[e]` resolves only if e is a PURE term over names that
             are STABLE for the fact's span: a `let` binding, a loop index, a
             parameter, or a `var` not assigned anywhere in the span.
             A `var` assigned in the span does NOT resolve a path step; every
             fact whose support mentions such a step is killed at the assignment.
established: at path resolution, per access
killed:      at any assignment to a name occurring in an index term of a support
             path — this is a kill by NAME, not by footprint, and is the third
             and last exception to "footprint = written path"
re-derived:  by re-establishing the index fact after the assignment
```

This is the rule whose absence lets `v.data[i]` mean two different storages in
one fact's lifetime.

```text
var i = 0
let a = v.data[i]                     // accept: i is not assigned before this fact's uses below
                                      //   Γ gains `a == v.data[i]`, support {v.data[i]}
i = i + 1                             // accept as a write; `a == v.data[i]` is KILLED here,
                                      //   because i occurs in its support's index term
let b = v.data[i]                     // accept: a NEW fact about a DIFFERENT resolved path
assert a == b                         // reject: missing `a == v.data[i]`; payload names the
                                      //   assignment at line 3 as the killer
```

### III-4 The arena bump offset

```text
rule:        A.meta.top is ORDINARY WRITER DATA in the metadata plane. The
             checker holds no bump pointer; it holds facts ABOUT A.meta.top,
             supported on A.meta, exactly like any other field.
             The connection between the offset and the issued extents is the
             WRITTEN arena invariant IA (I-14), never an inferred relation
established: by INV at a written `use` step, or by an alloc's own ensures
killed:      whenever a footprint names A.meta — that is, at every alloc and
             every free
re-derived:  `use IA(b, b')` — a written finite step the checker VERIFIES,
             never rediscovers (M1(b))
```

```text
let b1 = alloc(inout A.meta, carve A.bytes, 64)?
let b2 = alloc(inout A.meta, carve A.bytes, 64)?
                                      // A.meta written twice; any fact about A.meta.top from
                                      //   line 1 is killed at line 2 — correctly
use IA(b1, b2)                        // accept: INV instantiates the WRITTEN invariant at the
                                      //   written pair, yielding extent(b1) disjoint extent(b2)
par { fill(inout b1); fill(inout b2) }// accept: from the instantiated fact (II-22)
par { fill(inout b1); fill(inout b3) }// reject: missing `extent(b1) disjoint extent(b3)`;
                                      //   repair named: `use IA(b1, b3)`
```

### III-5 The free-list head

```text
rule:        p.meta.free_head is ORDINARY WRITER DATA, of type Handle<P,T> or an
             index. The pool's free list is the pool library's own written data
             structure, not compiler bookkeeping (M2(ii), settled by O17(a)).
             The checker knows nothing about it beyond the pool's written
             invariant
established: by the pool's own written invariant
killed:      whenever a footprint names p.meta
re-derived:  by INV at a written `use` step inside the pool's own body
```

```text
invariant FreeOk : free_head(p) ∈ Live(P) => false
                                      // accept: the pool's own written clause — a slot on the
                                      //   free list is not live. Verified inside the pool body
let h = insert(inout p.slots[σ], inout p.meta, x)
                                      // accept: the insert's OWN body pops the free list; the
                                      //   caller learns only `Live(h)` from the ensures
let f = p.meta.free_head              // accept as a read of ordinary data
assert Live(f) == false               // reject at the CALLER: missing the instantiation of
                                      //   FreeOk; it is a fact of the pool's implementation and
                                      //   the caller has no `use` step for it. Repair named:
                                      //   the pool exports it as an `ensures`, or the caller
                                      //   does not depend on it
```

### III-6 `σ(h)` and the handle's `idx` word **[stated here]**

This is the bridge whose absence VERDICT-CORE §4 records as fatal in the
generational-handles candidate ("the ghost `σ` and the runtime `idx` are never
related, so `h.idx < cap(P)` has no source and erasure has no theorem").

```text
rule:        `Handle<P,T>` is a one-word record with ONE declared field,
             `idx : u64`. The ghost σ(h) is DEFINED to be h.idx — it is a ghost
             NAME for a runtime field, not a second entity. Therefore:
               - `p.slots[σ(h)]` resolves to `p.slots[h.idx]` — an ordinary
                 element path whose index term is a field read (III-3);
               - the domain obligation h.idx < cap(p) is discharged from
                 σ(h) ∈ Live(P) and the WRITTEN pool invariant PBounds (I-18);
               - the access is STATICALLY TOTAL: no bounds compare is emitted,
                 which is the arm M2(ii) refuses and O2 asks about
established: at insert (I-19), with the ensures Live(result)
killed:      σ(h) as a TERM is killed by III-3 if h is a reassigned var;
             the LIVENESS fact is killed by I-20 and II-4
re-derived:  by INV from the pool invariant at a written `use` step
erasure:     h.idx survives to run time as the writer's own word; σ's ghost
             status and every Live fact are erased
```

```text
let nd = p.slots[σ(h)].value          // accept: σ(h) = h.idx; Live(h) => σ(h) ∈ Live(P) =>
                                      //   σ(h) < cap(p) by PBounds. STATICALLY TOTAL — no
                                      //   compare, no arm, no Option wrapper (G1)
let bad = q.slots[σ(h)].value         // reject: h : Handle<P,T>, q : Pool<Q,T>; the brands differ
                                      //   so `σ(h) ∈ Live(Q)` is missing. Cross-pool handle
                                      //   confusion is a TYPE error, not a runtime one
var hv = h; hv = other_h
let nd2 = p.slots[σ(hv)].value        // accept: the fact is established for the CURRENT value of
                                      //   hv, from Live(other_h); the earlier fact was killed
                                      //   at the assignment (III-3)
```

### III-7 `γ(h)` and `gen(P[σ])` — ghost only

```text
rule:        γ(h) is a GHOST FIELD of the handle value, fixed at insert, copied
             with the handle, never stored and never compared at run time.
             gen(P[σ]) is a GHOST FIELD of the slot, incremented only by remove.
             Neither is materialized; neither appears in the lowered program;
             LIVE(h) = σ(h) ∈ Live(P) /\ gen(P[σ(h)]) = γ(h) is a STATIC fact
established: γ(h) := gen(P[σ]) at insert
killed:      gen(P[σ(h)]) := succ(γ(h)) at remove, which falsifies the equality
             for every handle whose σ is not proved distinct (I-20)
re-derived:  never for a removed handle — that is the point (R1(a)(ii))
M6(iv) obligation: a build OUTSIDE the accepted program may materialize σ, γ and
             Live(P) and assert that every erased LIVE(h) would have held. This
             is the differential channel; it is not a build mode of the accepted
             program (M6(iv) forbids that)
```

```text
remove(inout g.slots[σ(m)], inout g.meta, m)
let m2 = insert(inout g.slots[σ'], inout g.meta, node)
                                      // accept: may reuse m's slot; σ' carries a FRESH generation
let nd = g.slots[σ(stale_m)].value    // reject: missing `gen(P[σ(stale_m)]) = γ(stale_m)`.
                                      //   Address reuse never revives an ended identity. No
                                      //   runtime generation compare exists — the refusal is
                                      //   entirely static (Tier B refused by name)
```

### III-8 `Live(P)` is never extensional **[stated here]**

The generational-handles critique found `Live(P)` growing by a fresh ghost
element per loop iteration with no widening. The rule that removes the need for
widening:

```text
rule:        the checker NEVER represents Live(P) as a set. It holds, per HANDLE
             NAME h in scope, at most the two facts of III-7. The number of such
             facts is bounded by the number of handle names in the enclosing
             declaration — a syntactic quantity. A loop creating a fresh σ per
             iteration adds no checker state: the loop-body handle name is one
             name, and II-7 intersects its facts at the head
established: per name, at insert or from a call's ensures
killed:      per name, at remove (I-20), with the proved-distinctness escape
re-derived:  per name, by INV
consequence: after ANY remove into p, every handle name k of p in scope loses
             LIVE(k) unless EXT proves σ(k) != σ(h). This is a stated cost, not
             a defect: it is the same shape as P1's `i != j`
```

```text
loop {
  let h = insert(inout p.slots[σ], inout p.meta, make_node())
                                      // accept: ONE handle name `h`; Live(h) at each iteration.
                                      //   Live(P) does not grow in the checker; no widening
                                      //   rule is needed and none is stated
  use(let p.slots[σ(h)].value)        // accept: Live(h)
}                                     // at the head: G := G_entry ∩ G_backedge; h is out of
                                      //   scope on both, so the intersection is trivial
```

### III-9 Iterator key distinctness

```text
rule:        the pairwise distinctness of what an iterator yields is a WRITTEN
             CLAUSE of the iterator's interface, never an inference from the
             yielded values being different handle values (which R4(b) forbids)
established: `ensures injective(keys)` on the Pool interface, verified once
             against the pool's own body
killed:      never — it is an interface clause of a declaration
re-derived:  not applicable
```

```text
fn keys<P,T>(let p: Pool<P,T>) -> Iter<Handle<P,T>>   ensures injective(keys)
par for h in g.keys() { g.slots[σ(h)].value = 0 }
                                      // accept: injective(keys) gives σ(h1) != σ(h2) for distinct
                                      //   iterations, which is an EXT fact over the slots plane
par for h in g.followers(start) { g.slots[σ(h)].value = 0 }
                                      // reject: `followers` states no injectivity; missing
                                      //   `distinct(σ(h1), σ(h2))`. repair named: add the clause
                                      //   and prove it, or iterate keys()
```

### III-10 A branch condition and its premise **[stated here]** (O11(a))

```text
rule:        a branch condition c is bridged to the runtime value ONLY as a
             premise in Π, available inside the syntactically enclosing guarded
             region. It is never a term in a fact and never indexes a fact.
             The premise is VALID only while no statement in the region writes a
             path named in c's support. Assignment to a name occurring in c
             KILLS the premise for the remainder of the region
established: at the branch (II-1)
killed:      at the join (II-5); or earlier, at an assignment into c's support
re-derived:  by re-testing the condition — which re-enters Π as a new premise
```

```text
if k != i && k != j {
  let x = v.data[k]                   // accept: the premise discharges EXT's side conditions
  k = k + 1                           // accept as a write; the premise `k != i` is KILLED here
  let y = v.data[k]                   // reject: missing `k != i && k != j` for the NEW k.
                                      //   payload names the assignment as the killer
}
var c = flag(); if c { … }; c = !c; if c { … }
                                      // accept both branches; NO relation between the two
                                      //   premises is carried (O11(a)). Under O11(b) it would be
```

### III-11 The `Option` discriminant

```text
rule:        vacancy is ORDINARY DATA: a discriminant word the source declared.
             The bridge is the written `match` arm. There is no language state
             `Uninit`, no occupancy bitmap the compiler maintains, and no
             write-once transition (R1(b)'s excluded case has no instance)
established: at formation and at every assignment
killed:      by FRAME when the slot's path is written
re-derived:  by a written arm
```

```text
backing Cache { meta : plane<CacheMeta,1>, slots : plane<Option<V>, len K> }
                                      // accept: a slot holds a VALID Option<V> from construction;
                                      //   it never goes from "holding no value" to "holding one"
match c.slots[k] {
  Some(v) => v                        // accept: total read; no state is consulted
  None    => { let v = compute(k); c.slots[k] = Some(v); v }
                                      // accept: one exclusive access at c.slots[k]
}                                     // cost: one discriminant word per slot, one written arm
                                      //   per lookup — the price of having no language state
```

### III-12 Foreign contents — there is no bridge

```text
rule:        for storage of type Foreign<T>, NO static name is connected to the
             runtime value. Nothing about its contents can be established, so
             nothing is re-derived. Every statement kills every contents fact
             about it (I-22 / P-d). The declared ACCESS CLASS (R14(iv)) is what
             makes the read defined rather than a race
established: never
killed:      at every statement, unconditionally
re-derived:  never; a writer who needs a stable value copies it into ordinary
             storage and reasons about the copy
```

```text
let snapshot = fc.bytes[0]            // accept: a read in the declared access class; `snapshot`
                                      //   is an ordinary Copy value in ordinary storage
if snapshot > 0 { use(snapshot) }     // accept: the fact is about `snapshot`, not about fc
if fc.bytes[0] > 0 { use(fc.bytes[0]) }
                                      // reject: missing any fact relating the two reads — the
                                      //   premise from the branch is about a value the agent
                                      //   could have changed between the two statements
```

### III-13 `old(m)` and the pre-state

```text
rule:        `old(m(x))` in an `ensures` denotes the term held for m(x) in Γ
             IMMEDIATELY BEFORE the call's FRAME step. It is not a second heap,
             not a snapshot, and not a stored value: it is a substitution
             performed once, at the call, by OLDCHAIN (II-14)
established: at the call, if a term for m(x) is held; otherwise the clause
             contributes nothing and NO search for a term is performed (M1)
killed:      the binding of old(m) does not outlive the call statement
re-derived:  not applicable — it is re-created at the next call
```

```text
let n = len(v)                        // Γ holds `n == len(v)`
push(inout v.data[len(v)], inout v.meta, 1)
                                      // accept: old(len(v)) binds to `n`; learn len(v) == n + 1
let m = len(v); assert m == n + 1     // accept
sort(inout v.data, inout v.meta); push(inout v.data[len(v)], inout v.meta, 2)
                                      // accept the push; but `len(v) == old(len(v)) + 1` binds
                                      //   old(len(v)) to NOTHING — sort killed every len term
                                      //   and stated no ensures. The clause contributes nothing
assert len(v) == n + 2                // reject: missing a term for len(v). repair named: give
                                      //   sort an `ensures len(v) == old(len(v))`
```

---

## 5. The judgment families

Every decision procedure the rules above invoke, named, with inputs, a
termination argument, and a degree (M1(b), M10(a)).

Quantities used in the degrees: `N` statements in a declaration; `V` names in
scope; `F` facts in `Γ`; `d` maximum path depth (a type-table constant); `k`
footprint size (a signature constant); `S` written proof steps (`use` lines);
`C` terms in the entering ProofContext; `I` the transitive instantiation
closure of a generic declaration.

VERDICT-CORE §1.3 names **six** judgments. Writing the rules out in full needs
**twelve**: the six, plus `RESOLVE`, `PLANE`, `CONFLICT`, `FRAME`, `OLDCHAIN`
and `GHOST`. Five of the six additions are constant-time or linear lookups;
`OLDCHAIN` is not, and `GHOST` was the family the generational-handles critique
found used in three derivations and named nowhere. The count is stated here
rather than defended.

| # | Family | Inputs | Procedure | Terminates because | Degree |
|---|---|---|---|---|---|
| 1 | **BIND** | the CFG, `Σ` | two-point lattice, conjunction at joins, greatest fixpoint | lattice height 2 over a reducible CFG; at most 2 passes | `O(N·V)` — linear in program size |
| 2 | **RESOLVE** (III-3) | a path expression, `Σ`, the set of names assigned in the span | walk the path's steps; check each index term's stability by a syntactic assignment query | path depth `d` is a type-table constant; the assignment query is a precomputed per-declaration set | `O(d)` per path; `O(N·d)` per declaration |
| 3 | **PLANE** (I-2, G3) | two resolved paths, `T` | table lookup: are these distinct planes of one backing? | a finite immutable type table | `O(1)` per pair |
| 4 | **PATH** | two resolved paths | prefix comparison over root → plane → extent → element, PLANE consulted at the plane step | paths are finite; comparison is structural | `O(d)` per pair |
| 5 | **EXT** | two element/extent steps, `Γ`, `Π` | a fixed linear fragment over loop indices, `requires` terms and the syntactically enclosing branch conditions, plus `ProvedRangePartition` for strided partitions (three side conditions: stride and base fixed at the loop preheader; `stride_nonnegative`; `base_nonnegative`) | the fragment is specification-fixed with no search; `ProvedRangePartition` is a single named family with a fixed side-condition list | the entailment closure over the ProofContext: **cubic in `C`** (the transitive product). Measured 29.9 / 164.5 / 1691.0 ms at N = 16/32/64, exponent ≈ 2.6–3.4 |
| 6 | **CONFLICT** (P-c) | two `(path, convention)` pairs | PATH/EXT, then one lookup in the fixed four-cell table | a constant table | `O(1)` after PATH/EXT; `O(k²)` per statement, `O(j²)` per span in open projections `j` |
| 7 | **FRAME** (G2, II-12) | `Γ`, `FP` | for each fact, for each support path, for each inout/sink footprint entry: PATH | finite sets; no fixpoint | `O(F·k·d)` per call |
| 8 | **OLDCHAIN** (P-b, II-14) | the killed facts, the `ensures` clauses, the pre-FRAME terms | bind `old(m)`; then ONE rewrite per killed measure fact per call | one rewrite per fact per call, by construction; no iteration and no search | `O(F·e)` per call, `e` = ensures clauses. Composes across calls without composing within a call |
| 9 | **REF** | a value refinement goal, `Γ`, `Π` | R11's existing discharge in the same linear fragment as EXT | as EXT | shares EXT's closure; cubic in `C` |
| 10 | **INV** (O15(b)) | a written `use Name(args)`, the written invariant, `Γ` | first-order syntactic instantiation at the WRITTEN arguments, then premise discharge by REF | the substitution is given by the writer; no matching search, no rediscovery (M1(b)) | `O(S·|Inv|)` per declaration — linear in written steps |
| 11 | **GHOST** (I-19, I-20, III-8) | the pool operation, `G` | insert: add two facts for one name. remove: for each handle name of that pool, keep its facts iff EXT proves `σ(k) != σ(h)` | `G` has at most one entry per handle name in scope, a syntactic bound; `Live(P)` is never a set | `O(H)` EXT queries per remove, `H` = handle names in scope |
| 12 | **NoReach(T)** (G5) | a type, `T` | depth-first walk of the type graph with memoization: does `T` reach a `Handle`, a pool row, or a `Foreign<U>`? | the type graph is finite; memoization visits each node once, so recursive types terminate | `O(|T|)` once per type, cached |

### 5.1 Declared violations

| Row | Status | Statement |
|---|---|---|
| **M1** | **satisfied, with a correction** | All twelve families are specification-fixed, total and terminating; none searches, none iterates to a cap, none consults a budget, machine speed or solver state. Harder goals arrive as written `use` steps verified by INV. **Correction declared:** the selection's claim of six families is wrong by six; the twelve above are the complete set these rules invoke, and `GHOST` in particular is load-bearing in I-19, I-20 and III-8 and is named in no inventory before this file. |
| **M2(i)** | **satisfied** | Every failure edge in an accepted program is a written arm: the `None` of a fallible `alloc`, the `None` of an `Option` slot, a typed outcome. Arena exhaustion is the writer's own free list deciding (O5(c)); heap and stack exhaustion are R14(v)'s defined stops. |
| **M2(ii)** | **satisfied under O3(a); VIOLATED under O3(b)** | Under O3(a): zero compiler-inserted branches. No drop flag — II-2 makes the mixed-consumption case a rejection, so no flag is needed and none is representable. No bounds compare at `p.slots[σ(h)]` — III-6 gives it a static domain from PBounds. No runtime generation compare — γ is ghost (III-7), Tier B refused by name. No occupancy structure the compiler maintains — III-11 makes vacancy the writer's discriminant; III-5 makes the free list the pool library's own data (O17(a)). The affine release at I-3 and I-4 is unconditional on its edge, so it is neither a branch nor bookkeeping state. **Under O3(b)** (a ghost generation is itself the refused mechanism): III-7 falls, I-20's kill has no carrier, and `p.slots[σ(h)]` needs either a bounds-and-occupancy compare (M2(ii) violated by its named case) or an undischarged domain (R11 violated). The model then reverts to the base with its stale-slot residue. |
| **M10** | **VIOLATED in the edit-stability clause; at the ceiling elsewhere** | (a) Degrees: families 1–4, 6–8, 10–12 are linear or low-polynomial in syntactic quantities and are inside M10's ceiling. Families 5 and 9 share the entailment closure, which is a **cubic transitive product** in ProofContext terms — at the ceiling, not above it, but measured at exponent ≈ 2.6–3.4 against a matched three-use control at 18.4 / 40.8 / 166.3 ms. Every `EXT`, `REF` and `INV` fact enters that context, and III-2 charges eight length facts per iteration in P5's shape. (b) Edit stability: M10(a) requires the verdict-changing set of a one-token edit to be bounded by a stated function of the edit's syntactic reach. In the callee direction that set is the **transitive instantiation closure** `I`, not "the enclosing declaration plus its callers" — measured at 82.4 s for a 375-line generic fixture against 42.8 s for 1757-line `wfgrep`, with 678 of 995 closures repeating an earlier input. This clause is **declared violated**, not partial. |
| **M3** | satisfied, with one enumerated entry | `split` (I-17) with its disjointness boundary obligation. `Pool` and `Arena` are WF libraries under O17(a). No `unsafe`, no bare assume: O15(b) makes `use` a named rule application with checked premises. |
| **M4** | satisfied | One spelling per construct: three conventions, `with … as`, `sink` as a statement, `subscript … origin`, `plane`, `Pool`/`Handle`, `invariant`/`use`, `old()`, the closure capture clause. `carve` (I-15) is a footprint marker on an existing convention list, not a fourth convention. |
| **M5** | satisfied | Overlap permission (II-22) reads footprints already in the signature; deleting the PAR judgment changes no verdict. |
| **M8** | **undecidable, pending O6** | Writing the rules out adds four taught items the selection's count of 13 did not carry: `carve` (I-15), the index-term stability rule (III-3), the per-backing plane invariant (III-2), and the R2 definiteness clause at joins (II-2). Count now **17**. K, the tokenizer and the subsystem share are unset, so no verdict is available. |

---

## 6. Open defects

Named, not repaired. Each is a place where these rules, written out in full, do
not deliver what the row or the program asks.

| # | Defect | Where it bites | Why it is not repaired here |
|---|---|---|---|
| **DV-1** | **Stored projection over a non-pooled container.** A cursor, an iterator, a callback or a closure that holds a projection cannot exist (II-16, II-19, II-21). P4's first required conjunct holds only when the container is a pool row; a cursor over a frame-local `Vec` stores an index and needs the container re-supplied at every use | P4 (partial, model at fault); R7; M7's projection-escape class | It is the model's defining boundary, not a gap in it. This is exactly owner decisions **O7** and **O12**; repairing it changes the base |
| **DV-2** | **The signature cascade.** The named repair for DV-1 is "thread the container or pool parameter through every intermediate signature", which is not a local fix. M7(a) requires the payload to name a tree location where a local fix exists | M7 (partial); P4, P16 | No local fix exists; naming one would be false |
| **DV-3** | **P7's two-alias premise is unwritable, so R3 has no acceptance test.** Two long-lived writable names for one slot are refused by the conflict table (II-17), and a projection cannot be stored to become the second name. R3(e) names P7 and nothing else | R3 (partial, model at fault); P7 | The premise is unwritable by construction (DV-1). R3 needs a different acceptance test or the row falls |
| **DV-4** | **The hole read is total.** `replace(inout s, None)` then `read(s)` returns `None` and is accepted (I-7, III-11). Every case and program whose expected verdict is "reject: hole" has no instance | P7 line 3; CASES S03, S06, B03, B04, B07, B09, L02 | It is the deliberate consequence of D3 ("no states: validity by construction"). The repair is named — declare the slot `Option` and write the arm — but the *refusal* the programs asked for is gone |
| **DV-5** | **OLDCHAIN is one step per call and cannot bridge a call with no `ensures`.** II-14 step 4 rewrites a killed measure fact once, using the writing call's own relation. An operation that writes a metadata plane and states no relation over the measure kills the fact with no re-derivation route | II-14's second example; every third-party container operation | Making the chain search for a route is a search (M1). The only route is to widen the callee's signature, which is a non-local edit |
| **DV-6** | **`remove` kills liveness for every handle of the pool.** III-8's rule keeps `LIVE(k)` only where EXT proves `σ(k) != σ(h)`. Handles are opaque data, so that proof comes from a written pool invariant or a written `requires` and from nowhere else | I-20; CASES B05, B06; P3 after a removal | Ghost slot equality is not decidable, and deriving distinctness from two handle values being different is exactly what R4(b) forbids. The cost is real and is stated, not hidden |
| **DV-7** | **The foreign kill rule does not follow handles.** P-d (I-22) kills contents facts about `Foreign<T>` storage and its containment descendants. A pool row that a foreign-shared identity names through a `Handle` is not a containment descendant, so its facts survive an interleaving | R10 (partial); P14; the `dup` case | Quantifying the kill over `NoReach` failure would kill every pooled fact in the program. Either `dup` is refused, or the open-file description's end becomes a written operation on the last wrapper. The owner has not ruled |
| **DV-8** | **The `origin` union is a footprint wider than any written path.** II-20 kills facts about every path in the origin set for the whole block. R6's identity "the footprint is the written access path" therefore has three declared exceptions, not one: P-d (I-22), index-term instability (III-3), and this | II-20; R6; P9's `pick` | Narrowing it requires knowing which origin the accessor yielded, which is the runtime condition the accessor branched on — a condition term at a join, refused by O11(a) |
| **DV-9** | **No split-and-rejoin.** II-23 declares the absence. A coalescing arena, a re-joinable range token, and any API handing out two halves and later taking them back are outside the language | P2's verdict ("no coalescing"); D9's refused HIS-D9-12 | The rejoin would need an adjacency fact over two ended extents, which no rule produces and which a written invariant cannot establish after both names exist |
| **DV-10** | **R5's grounds (ii) and (iii) have no rules here.** II-22 states ground (i) only. A recombinable accumulator under a checked operator law, and a named weaker determinism level, are R14's vocabulary | R5 (partial); P15 not derived | This tuple neither supplies nor obstructs them; writing them would be writing R14, which D1–D4 does not own |
| **DV-11** | **The ProofContext is the cost centre and these rules feed it.** Every `EXT`, `REF` and `INV` fact enters it; III-2 charges one length fact per plane per iteration; II-13's two-part footprints double the paths PATH compares per call | M10 (declared violated in §5.1) | The degree is stated, and it is at the ceiling. Reducing it is a compiler-architecture question these rules do not decide |
| **DV-12** | **M6's adequacy theorem is owed.** III-7 supplies the M6(iv) differential channel (a shadow build outside the accepted program that materializes σ, γ and `Live(P)`), which is evidence of a *checker defect*, not of soundness | M6 (partial); owner decision 9 | The theorem is a stated later obligation, not a rule |

---

## Appendix A — the frame rule's three exceptions, in one place

"The footprint is the written access path" is true except at exactly three
rules. Collected here because two of them are stated for the first time in this
file and a reader checking R6 needs all three.

| # | Exception | Rule | Kills |
|---|---|---|---|
| 1 | foreign contents (P-d) | I-22 | every contents fact about every `Foreign<T>` storage in scope, at **every statement**, footprint or not |
| 2 | index-term instability **[stated here]** | III-3 | every fact whose support path has an index step mentioning a name assigned by this statement — a kill by **name**, not by path |
| 3 | the `origin` union | II-20 | every fact overlapping any path in a yielded projection's origin set, for the block's span — a footprint **wider** than any written path |

## Appendix B — the 26 hand-derived cases, re-derived

CASES.md S01–S07, B01–B13, L01–L06 were written under DESIGN.md Candidate A
(locators plus a current resource context). Candidate A's `p = ref(a)` is a
**storable, rebindable locator**; this candidate has no such form. Two
renderings are available and both are used below:

- **binding rendering** — the objects are `var a: Obj`; `ref(a)` disappears and
  every access is written at the path `a`. Candidate A's locator variable has
  no image.
- **pool rendering** — the objects are pool rows; `p = ref(a)` becomes
  `let p: Handle<P,Obj> = h_a`, ordinary copyable data that may be stored,
  copied and rebound. This is the rendering that preserves the cases' shape.

Classification of each difference, as required: **CL** capability loss;
**DR** deliberate refusal with a named repair; **CC** correction of the case.

| Case | CASES.md expects | Here | Class | Note |
|---|---|---|---|---|
| S01 | ACCEPT | ACCEPT (pool rendering) | — | rebinding a handle is ordinary data assignment (III-6); writes go through paths |
| S02 | ACCEPT | ACCEPT | — | `replace` is total (I-7); old value-dependent facts die by FRAME |
| S03 | REJECT (read of empty) | ACCEPT, returns `None` | **DR** | the hole is the writer's discriminant (III-11); repair: declare `Option` and write the arm. DV-4 |
| S04 | ACCEPT | ACCEPT | — | `s = Some(sink v)` (I-6) |
| S05 | ACCEPT | ACCEPT | — | the moved-out value is independent; I-4 releases the storage |
| S06 | REJECT (no old value) | ACCEPT, returns `None` | **DR** | same as S03; repair: match the returned `Option`. DV-4 |
| S07 | REJECT after release | REJECT | — | binding rendering: `BIND` (I-4). Pool rendering: `LIVE` killed (I-20, III-7) |
| B01 | ACCEPT | ACCEPT | — | `LIVE(p)` holds on both arms and survives the intersection (II-4) |
| B02 | ACCEPT, and the swapped variant REJECTS | ACCEPT; the swapped variant also ACCEPTS | **DR** | the correlation between a path condition and a value fact is not tracked (II-5, O11(a)); repair: match on the `Option`. Evidence for **O11** |
| B03 | ACCEPT `read(q)`, REJECT `read(a)` | ACCEPT `read(q)`; `read(a)` ACCEPTS, returning the `Option` | **DR** | as S03. `q == p` is an ordinary value equality (REF) |
| B04 | ACCEPT; intermediate reads REJECT | ACCEPT; intermediate reads ACCEPT | **DR** | as S03 |
| B05 | ACCEPT `read(q)` after `release(p)` | REJECT | **CL** (pool rendering) / **DR** (binding rendering) | III-8: `remove` kills `LIVE(q)` unless `σ(p) != σ(q)` is proved. Repair named: a written distinctness premise on the pool interface, the same shape as P1's `i != j`; or restructure as P8. DV-6 |
| B06 | ACCEPT `read(saved)`, REJECT `read(q)` | REJECT both | **CL** | same as B05: the saved handle also loses `LIVE`. DV-6 |
| B07 | REJECT | ACCEPT (total read of `Option`) | **DR** | as S03 |
| B08 | ACCEPT, by recovering the path's state | ACCEPT the read; the value refinement is NOT recovered | **CC** | the read is total, so nothing needs recovering; the case's mechanism (state recovery from a repeated condition) has no instance. Under **O11(b)** the refinement would be derivable |
| B09 | REJECT | ACCEPT the read; the premise-kill rule the case is about is III-10 | **CC** | the case's content survives as a rule (III-10) with no instance in this program |
| B10 | the join must not erase the obligation | REJECT at the join | **DR** | II-2's definiteness clause. Repair named: `let keep = if c { sink a; b } else { sink b; a }; sink keep` |
| B11 | ACCEPT | REJECT as written; ACCEPT after the P8 restructuring | **DR** | `remaining = ref(b)` inside an arm is unwritable (DV-1); the survivor is yielded as a value |
| B12 | ACCEPT when exactly one of c, d holds | REJECT at the first join | **DR** | II-2. The case's own accepted variant (`if cond { release }; if !cond { release }`) also rejects. **The sharpest O11 discriminator in the set together with L05** |
| B13 | policy question | REJECT at the join | **DR** | II-2 removes the policy fork: the mixed case is not representable, so neither cleanup policy is needed |
| L01 | ACCEPT | ACCEPT | — | take/put are total (I-7); the loop head writes nothing (II-7) |
| L02 | REJECT (second take) | ACCEPT (both takes total) | **DR** | as S03; R2 still requires each taken value be consumed |
| L03 | ACCEPT with no annotation | ACCEPT with one written loop invariant | — (cost) | the relative invariant (`p`'s target `is_some`, `q`'s target `is_none`) is writable and verified by INV; the case's ACCEPT survives at one written line |
| L04 | ACCEPT | REJECT at the loop exit join | **DR** | II-9: consumed on the break edge, bound on the normal and zero-iteration edges. Repair named: `let r = loop { if stop { break None }; break Some(sink a) }` and a match |
| L05 | ACCEPT | REJECT | **DR** | the correlation between the source boolean `active` and `BIND(a)` is a condition term over state, refused by O11(a). Repair named: make the resource a value — `var slot: Option<Obj> = Some(a)`, consume via `replace(inout slot, None)`, match after the loop. **The second sharpest O11 discriminator** |
| L06 | ACCEPT | ACCEPT | — | the slot is always live; `v` must be consumed on the break path (Copy here, so free) |

Counts: 9 unchanged, 13 **DR**, 2 **CL**, 2 **CC**.

The two capability losses (B05, B06) are both DV-6. The thirteen deliberate
refusals fall into three families: seven are DV-4 (the hole is data, not a
language state), four are II-2's definiteness clause (B10, B12, B13, L04), and
two are O11(a) (B02, L05).


# File: rules-window-focus.md

# Rules: `window-focus`, repaired — storage, split/rejoin, and the bridge

Key: `window-focus`. Round: core2 (rule sets in full). Research date: 2026-09-16.
Source candidate: [candidate-wildcard.md](../core/candidate-wildcard.md), repaired
against [critique-wildcard-soundness.md](../core/critique-wildcard-soundness.md),
[critique-wildcard-cost.md](../core/critique-wildcard-cost.md) and the refusal row for
`window-focus` in VERDICT-CORE §4. Requirements: VERDICT-D0 §2 (R1–R15, M1–M11),
owner interim positions §7 taken as settled. Ladder style: MECHANISM-MAP §3 — a
pseudocode line, then `// accept: fact used` or `// reject: missing fact`.

This file states rules. It selects nothing, ranks nothing and compares nothing.
Where a rule differs from a CASES.md expected verdict the difference is classified
in §3.21 as *capability loss*, *deliberate refusal with a named repair*, or
*correction of the case*.

**What the repair changed, in one line each.**

| Fatal finding (VERDICT-CORE §4 / critiques) | Disposition here |
|---|---|
| A fine name cannot carry state across a call (S-F1) | **Repaired by re-cutting the seam.** State lives on **places** (persistent, static, in rows, crossing every cut); window entries carry **interference only** and no state. Third name kind declared and priced (§1.1, §5 M4/M8) |
| A fine name cannot be typed when a pointer must hold one; the interior pointer escapes with no diagnostic (S-F2, P13) | **Repaired, and it needs a fine name in a type.** `ptr<ρ↓w.k,T>` exists, is second class by a lexical scope discipline (`ESC`), and is rejected by the parser in every cut position. Priced in II19. **This is O12(b)** |
| A fine name cannot be minted in unbounded number (S-S4, C-S7) | **Repaired** by the parametric entry `[∀k ∈ E]` (II14) whose `dis` is per binder, and by the loop-minting rule I22 |
| Coarse names cannot express sibling disjointness (S-F3, C-F4) | **Repaired** by promotion-with-proved-extent `split ρ at [lo,hi)` (I11) and the extent algebra `EXT`; `fresh ∃ρ'` with no extent no longer exists |
| Coarse names cannot express a disjunctive post-state (S-F5, S-F7, C-F1) | **Repaired** by bounded guarded values `ite(c,·,·)` over captured boolean bindings, canonical, depth ≤ `Gmax` (II1–II6). **This is O11(b), in a bounded form** |
| P8 has no accepted spelling | **Repaired**: P8(a) is accepted verbatim under II3; no drop flag, by the guard-neutrality rule II6 |
| `dis` is not total | **Repaired**: `DIS` is defined on every identity form including guarded identities, origin sets, reached backings and cross-level window entries (§5, `DIS`) |
| `fresh ∃ρ' ⊑ ρ` has no discharge | **Repaired**: promotion carries a proved half-open extent inside the parent's *free* extent, discharged against a **written** invariant (I11, I13, III6). No checker-side free-set is maintained |

---

## 1. The model in one page

### 1.1 Names

```text
coarse name  ρ, β  ::= an atom minted at a formation site (I1, I11)
                       appears in types, struct parameters, signatures; persists until Gone
place        π     ::= ρ | π.f | π.backing | π[t] | π[lo..hi) | π\K   (residue of π minus keys K)
                       a static description; appears in rows; carries STATE; crosses every cut
window entry e     ::= w↓k    an entry of the carve opened at window head w
                       carries INTERFERENCE only; never carries state; retired at w's close
identity     ι     ::= ρ | π | e | { ι, … }        (a finite static origin set)
                     | ite(c, ι, ι)                 (a guarded identity)
guard atom   c     ::= a captured boolean *binding version* (III12); never materialized
guarded X    G[X]  ::= X | ite(c, G[X], G[X])       canonical: atoms in capture order, depth ≤ Gmax
state        s     ::= a state of auto(type(π));  default automaton { Uninit, Init, Gone }
index term   t     ::= a literal | a captured index binding (III8) | an affine term over a loop binder (III9)
```

`Gmax` is a specification constant (provisional `Gmax = 2`). It is not a budget in
M1's sense: it does not select acceptance by work done, it is a fixed precision
rule whose overflow is deterministic and diagnosed (II3).

Three identity kinds, not two. That is the M4 price of repairing S-F1 and it is
declared, not hidden (§5, declared violations).

### 1.2 What the checker holds

| Structure | Content | Span | Erased |
|---|---|---|---|
| `Auto` | per nominal: declared states and transitions; default `{Uninit, Init, Gone}` | program | yes |
| `St` | **place → `G[state]`**, a finite key tree with a residue key per level (II/COVER) | function; crosses calls through rows | yes |
| `Own` | affine owner binding → identity, with its obligation | lexical scope | yes |
| `Facts` | measures (`len_of`, `cap_of`, `off_of`, `extent_of`), contents, declared invariants, keyed by identity | until killed (II11) | yes |
| `Carve` | a **forest** of open windows; each node an ordered entry list over a parent identity | window span | yes |
| `Span` | binding → the window its type names (II19) | lexical | yes |
| `⊑` | containment forest over coarse names, each edge carrying a **proved half-open extent** | program | yes |
| `≡` | declared coincidences, admissible only under I/II's check (II10, `alias`) | program | yes |
| `Γg` | guard environment: live atoms, their recorded relations (`=`, `¬`), their versions | function | yes |

`St` is finite: its keys are the places the program text and the rows in scope
mention, plus one residue key per split level. Any other place resolves through
`COVER` (§5).

### 1.3 What the writer writes

| Where | What | How often |
|---|---|---|
| signature | one row line per identity the callee touches: `π : pre ⇒ post @g`, plus `split`/`end`/`relocates`/`absorb` clauses, `result : ι`, `requires`/`ensures`, `outcomes` | always (R7) |
| struct field holding a pointer | the store name as a struct parameter: `struct Cursor<ρ> { at: ptr<ρ,V>, i: u64 }` | per stored pointer |
| nominal with a protocol | `states`/`transitions`; `invariant` clauses | per protocol type |
| loop head | a row when the body's composed state edge is not the identity, when a name is minted, or when a guard must survive the back edge (II7) | per such loop |
| join | nothing (II3 merges); a written value only where the writer wants a runtime difference | — |
| statement | `focus ι as { A = …, B = … } where { … }` for a runtime-indexed split | per split |
| call site | `alias ρ σ` (checked, II10); an index or extent proof a fixed family cannot close | per pair, per split |

### 1.4 What is erased

Coarse names, places, window entries, carves, grades, guard atoms, origin sets,
states, containment, coincidences, invariants, every proof. Erasure residue, with
the repairs to the emission table (C-F5, C-F6, C-S15, S-F4, S-S11) applied:

| Source fact | Emitted | Side condition (repair) |
|---|---|---|
| a window entry, inside its window | `!alias.scope` + `!noalias` vs siblings | none |
| window head | `llvm.experimental.noalias.scope.decl` | none |
| `@E` on an identity | parameter `noalias` | **and** the identity's root is not in R14(iv)'s foreign access class and carries no foreign-agent automaton state (C-S15) |
| a `Uninit ⇒ Init` edge over an entry's whole extent | `writeonly` + `initializes((lo,hi))` | grounded on the **state edge**, never on `@W` (C-F5, S-S11) |
| `@R` with no `@W` | `readonly` | none |
| whole row `@0` | `memory(none)` | **and** the row carries no `split`, `end` or `relocates` clause (S-S11) |
| `split ρ' ⊑ ρ` on a result | `noalias` return **only when ρ has no `⊑` parent** (S-F4) | a contained promotion emits nothing |

---

## 2. Rule set I — storage creation and destruction

Form: `premises ⇒ effect on the checker's facts`. Every rule is syntax-directed
and reads only the structures of §1.2.

| # | Operation | # | Operation |
|---|---|---|---|
| I1 | heap formation (`new`) | I13 | arena carve |
| I2 | scope-local formation | I14 | arena block end |
| I3 | scalar write / initialize | I15 | arena reset |
| I4 | `take` | I16 | container push/insert, no reallocation |
| I5 | `replace` | I17 | container push, possible reallocation |
| I6 | sink (move a value into storage) | I18 | container remove / pop |
| I7 | `end` (free, dispose, close) | I19 | pool insert |
| I8 | scope exit | I20 | pool remove |
| I9 | relocating move | I21 | pool compact (relocation in place) |
| I10 | non-relocating move | I22 | minting inside a loop |
| I11 | promotion / split with proved extent | I23 | address reuse |
| I12 | merge / restore of promoted siblings | | |

### I1 `new` — heap formation

`premises:` none beyond the size expression's own domain (R11).
`⇒ effect:` mint fresh `ρ`; `⊑`-root; `St(ρ) := Uninit`; `Facts += extent_of(ρ)=[0,n)`,
`size_of(ρ)=n`; mint an affine owner binding with one end-obligation.

```text
(p, own_a) = new(10)          // accept: fresh ρa, St(ρa)=Uninit, extent_of(ρa)=[0,10)
read(p)                       // reject: missing fact St(ρa)=Init  — R1(a)(i), M7 names the sub-property
write(p, 1)                   // accept: I3 admits Uninit ⇒ Init for a copy scalar
x = read(p)                   // accept: St(ρa)=Init
```

### I2 scope-local formation

`premises:` the binding is in a scope with a statically known exit.
`⇒ effect:` mint `ρ`, `St(ρ) := Uninit`, owner obligation is the scope's (I8). No
heap obligation; the end event is the scope exit, which is contract-visible (C7).

```text
{ let s : S                    // accept: mint ρs, St(ρs)=Uninit; obligation owned by this scope
  write(s.x, 1)                // accept: St(ρs.x) := Init; St(ρs.y) untouched (COVER residue)
  read(s.y)                    // reject: missing fact St(ρs.y)=Init  — the residue key is Uninit
}                              // accept: I8 — ρs ends here; every place under it becomes Gone
```

### I3 scalar write / initialize

`premises:` `type(π)` is a copy type; `St(π) ∈ {Uninit, Init}`; the access passes II15.
`⇒ effect:` `St(π) := Init`; `KILL` contents facts whose support is not `dis` from `π`.

This is the *total* store. It is deliberately admissible from both `Uninit` and
`Init` (CASES B07: `put` would lack its premise on the full path, `replace` on the
empty path).

```text
p = ref(a)
if cond { old = take(p) }      // accept: I4 under the guard — St(ρa) = ite(cond, Uninit, Init)
read(p)                        // reject: missing fact St(ρa)=Init on the cond arm  (CASES B07)
write(p, 20)                   // accept: I3 admits Uninit|Init ⇒ Init for a copy scalar
read(p)                        // accept: St(ρa)=Init on every feasible guard assignment
```

### I4 `take`

`premises:` `St(π) = Init`; `type(π)`'s class is affine or linear; II15.
`⇒ effect:` `St(π) := Uninit`; the value's obligation moves to the taken binding;
`KILL` contents facts supported by `π`.

```text
v = take(p)                    // accept: St(ρ)=Init ⇒ Uninit; v now carries the obligation
read(q)                        // reject: missing fact St(ρ)=Init — the hole is on the STORAGE (P7)
put(q, move v)                 // accept: I6 — another alias fills the hole; nothing was attached to p
read(p)                        // accept: St(ρ)=Init
```

### I5 `replace`

`premises:` `St(π) = Init`; II15.
`⇒ effect:` one event `Init ⇒ Init`; no intermediate hole; the old value's obligation
moves to the result binding; `KILL` contents facts supported by `π`; `ensures contents(π)=new`.

```text
old = replace(p, new)          // accept: St(ρ)=Init, one event, no hole (SET-2)
read(q)                        // accept: reads `new` — the contents fact was killed and re-ensured
drop_or_move(old)              // accept: the old value's obligation is now the binding's (R3)
                               // omitting this line: reject at I8 — an undischarged affine value
```

### I6 sink — move a value into storage

`premises:` `St(π_dst) = Uninit` (for `put`) or the destination is a fresh element
slot (I16, I19); the source binding holds the value's obligation.
`⇒ effect:` `St(π_dst) := Init`; the obligation transfers from the binding to the
destination place; the source binding is consumed.

```text
v = take(src)                  // accept: I4
put(dst, move v)               // accept: St(π_dst)=Uninit ⇒ Init; obligation moves to π_dst
put(dst2, move v)              // reject: the affine binding v was consumed at the previous line (R3)
end(dst_owner)                 // accept: I7 — the obligation now under π_dst is discharged by I7's closure
```

### I7 `end` — free, dispose, close

`premises:` `Own(x) = ρ` live and unconsumed; **`Carve(ρ) = ∅`** (no open window over
`ρ` or any place under it); **`Span(ρ) = ∅`** (no live span-typed binding, II19);
**no live `⊑`-child of `ρ`** (every promotion is ended or merged, I12); every
obligation held *by a value inside* `ρ` is discharged; `St(ρ) ∈ {Init, Uninit}` on
every feasible guard assignment.
`⇒ effect:` `St(π) := Gone` for `ρ` and every `St`-key under it, permanently; `Own(x)`
consumed; every fact keyed by an identity under `ρ` is dropped; the `⊑` node stays
(so `dis` against a later sibling is still decidable, I23).

```text
free(A, b2)                    // accept: St(ρ2)=Init, Carve=∅, Span=∅, no child, owner b2 consumed
read(b2)                       // reject: St(ρ2)=Gone — R1(a)(ii), M7 names the end at the previous line
free(A, b2)                    // reject: the affine owner b2 was consumed (R3/R2)
free(v)                        // accept only if v's backing β was ended or is named in the same row:
                               //   free(v) row { ρ : Init ⇒ Gone ; backing_of(ρ) : Init ⇒ Gone }  (S-S6)
```

### I8 scope exit

`premises:` for every owner binding in the scope and every feasible guard assignment,
the obligation is discharged or moved out; every affine value binding is consumed.
`⇒ effect:` `St(π) := Gone` for every scope-local storage; obligations retire.

```text
{ a = new(10)
  if cond { free(a) }          // accept: I7 under the guard — St(ρa) = ite(cond, Gone, Init)
}                              // reject: missing discharge on the ¬cond arm (CASES B13, R2)
                               // M7 repair: "release a on the ¬cond path"; the model inserts nothing (M2(ii))
{ a = new(10)
  if cond { free(a) }; if !cond { free(a) }   // accept: guard relation ¬c recorded (II5); exactly one end
}                                             //   on every feasible assignment (CASES B12 variant)
```

### I9 relocating move

`premises:` as I7's structural premises (`Carve`, `Span`, no live child); `St(ρ)=Init`.
`⇒ effect:` `St(ρ) := Gone`; mint fresh `ρ''` with `St(ρ'') := Init`; `extent_of(ρ'')` is
`extent_of(ρ)`'s shape; facts about `ρ` re-key to `ρ''` **only** where the row `ensures`
them.

```text
t = move s                     // accept: Carve(ρs)=∅, Span(ρs)=∅; St(ρs):=Gone; fresh ρt Init
read(px)                       // reject: St(ρs)=Gone — the bytes moved (ladder Step 1)
read(ptr_of(t.x))              // accept: St(ρt.x)=Init from the move's ensures
```

### I10 non-relocating move

`premises:` the moved value is a handle whose pointee storage is a separate `ρ`.
`⇒ effect:` the *handle's* storage ends (I9 on the handle); the pointee's `St` is
untouched; `backing_of`/pointee bridges re-key to the destination binding (III4).

```text
b  = heap_box(v)               // accept: mint ρbox (the box cell) and ρc (the content), ρc ⊑ ρbox is FALSE:
                               //   they are separate roots; backing_of(ρbox) = ρc  (III4)
pc = ptr_of(deref(b).x)        // accept: pc : ptr<ρc, X>
b2 = move b                    // accept: the handle moved; St(ρc) untouched
read(pc)                       // accept: St(ρc.x)=Init — the content storage did not move (ladder Step 1)
```

### I11 promotion / split with a proved extent

This is the repair of S-F3 (sibling disjointness) and C-F4 (`fresh ∃ρ'` had no
discharge). There is no extent-free promotion in the model.

`premises:` `[lo,hi)` is proved (by `EXT`/`AFF`) to lie inside the parent's **free
extent** `free_of(ρ)`, which is a *written* fact of the parent's type (III6, III7);
`St(ρ) ≠ Gone`.
`⇒ effect:` mint `ρ'` with `ρ' ⊑ ρ` and `extent_of(ρ') := [lo,hi)`; `free_of(ρ) :=
free_of(ρ) \ [lo,hi)` as a **fact**, re-established by the parent's written invariant,
not by a checker-side set; `St(ρ') := Uninit`; mint `ρ'`'s owner obligation.
`dis(ρ', ρ'')` for two children of one parent is decided by `EXT` over their extents —
never by inequality of names (R4(b), C3).

```text
(b1,ρ1) = carve(A, 64)         // accept: [off,off+64) ⊆ free_of(ρS) from the arena invariant (III6)
(b2,ρ2) = carve(A, 64)         // accept: [off+64,off+128) ⊆ free_of(ρS) — off advanced by the same invariant
fill(b1) ∥ fill(b2)            // accept overlap: dis(ρ1,ρ2) by EXT on [off,off+64) vs [off+64,off+128)
                               //   — R5(a)(i). dis(ρ1,ρS) is FALSE (ρ1 ⊑ ρS): R4(b)'s negative test holds
(c1,c2) = split(buf, at=m)     // accept: split_at_mut's spelling — two coarse siblings, proved disjoint,
                               //   each first class, each storable and passable across a cut  (O7 evidence)
```

### I12 merge / restore of promoted siblings

`premises:` the named children are `⊑`-children of `ρ`; their extents are adjacent
and cover the range being restored; each child's `St` is `Init` or `Uninit` (not `Gone`);
`Carve`/`Span` empty for each.
`⇒ effect:` the children's names end (not `Gone` — *absorbed*); `free_of(ρ)` regains
the union; `St` keys under the children re-key under `ρ`'s residue at the meet of
their states.

```text
(c1,c2) = split(buf, at=m)     // accept: I11
fill(c1) ∥ fill(c2)            // accept: dis by EXT
buf = merge(c1, c2)            // accept: adjacent extents [0,m) and [m,n) cover [0,n); both Init
read(buf[0])                   // accept: St(ρbuf.residue)=Init, the meet of the two children
                               // omitting merge and then free(buf): reject at I7 — a live ⊑-child
```

### I13 arena carve (an instance of I11)

`premises:` the arena's declared invariant `carved_of(A) = [0, off_of(A))` and
`free_of(A) = [off_of(A), cap_of(A))` holds; `n ≤ cap_of(A) − off_of(A)` is discharged
or the operation's outcome arm is written (R12; owner interim O5(c): arena exhaustion
is an ordinary library outcome).
`⇒ effect:` I11 with `[off, off+n)`; `ensures off_of(A) = old(off_of(A)) + n`, which
re-establishes the invariant in one `INV` step.

```text
fn carve(A: ptr<ρS, Arena>, n: u64) -> (ptr<ρ',u8>, ρ')
  row { ρM : Init ⇒ Init @W ;                        // metadata plane only — NOT ρS @W  (S-S10 repair)
        split ρ' ⊑ ρS at [off_of(A), off_of(A)+n) ;
        requires off_of(A) + n <= cap_of(A) ;
        ensures  off_of(A) = old(off_of(A)) + n }
                               // accept: the body's one obligation is [off,off+n) ⊆ free_of(ρS),
                               //   discharged by INV against the written arena invariant (no bare assume, M3)
```

### I14 arena block end

`premises:` I7's premises for `ρ'`.
`⇒ effect:` `St(ρ') := Gone`; the extent returns to the **writer's own free list**, a
value of the arena type (O17(a)); the checker gains nothing automatically. A later
`carve` from a returned extent discharges its `⊆ free_of` obligation against the
arena's written invariant over that list, in one `INV` step.

```text
free(A, b2)                    // accept: St(ρ2):=Gone; row's `absorb [lo,hi) into free_of(ρS)` clause (S-S7)
(b4,ρ4) = carve(A, 32)         // accept: fresh ρ4, extent inside the reclaimed range by the written invariant
read(b2)                       // reject: St(ρ2)=Gone — address reuse never revives it (I23, R1(a)(ii))
fill(b4) ∥ fill(b1)            // accept: dis(ρ4,ρ1) by EXT — ρ2's old extent is disjoint from ρ1's
```

### I15 arena reset / bulk end

`premises:` **every** live `⊑`-child of `ρS` is `Gone`; `Carve(ρS)=∅`; `Span(ρS)=∅`.
`⇒ effect:` `free_of(ρS) := [0, cap_of(ρS))`; `off_of(ρS) := 0`; `St(ρS)` unchanged.

```text
(b1,ρ1) = carve(A, 64)
reset(A)                       // reject: ρ1 is a live ⊑-child — M7 names ρ1's formation site and "end it first"
free(A, b1)                    // accept: I14
reset(A)                       // accept: no live child; off_of(A):=0, free_of(A):=[0,cap)
```

### I16 container push / insert without reallocation

`premises:` `cap_of(ρ) > len_of(ρ)` is discharged (III3); `St(backing_of(ρ)) = Init`;
`Carve(backing_of(ρ)) = ∅`; `Span(backing_of(ρ)) = ∅`.
`⇒ effect:` `St(β[len_of(ρ)]) := Init` (I6 sink); `ensures len_of(ρ) = old(len_of(ρ))+1`;
`backing_of(ρ)` unchanged, so **no** element identity changes.

```text
reserve(v, len_of(v)+1)        // accept: establishes cap_of(ρ) > len_of(ρ)  (III3)
push(v, 5)                     // accept: I16's first premise discharged; β unchanged; pe stays valid
read(pe)                       // accept: St(β[i])=Init and i < len_of(ρ) — R1(a)(i) and R11 both discharged
```

### I17 container push with possible reallocation

`premises:` `St(ρ)=Init`; `Carve(β)=∅`; `Span(β)=∅`.
`⇒ effect:` a **two-armed** row selected by `cap_of(ρ) > len_of(ρ)`:
 - arm A (predicate discharged): I16.
 - arm B (negation discharged): `St(β) := Gone`; `split β' ⊑ ρ` fresh; `backing_of(ρ) := β'`;
   element facts about `β` die.
 - neither discharged: a **fresh guard atom** `g#` is bound to the predicate (III12,
   II12) and the post-state is `ite(g#, armA, armB)`. `g#` is a checker atom; by II6
   no accepted program's behavior may depend on it.

```text
push(v, 5)                     // accept: the call runs. Post-state without a capacity proof:
                               //   St(β)=ite(g#, Init, Gone), backing_of(ρ)=ite(g#, β, β')
read(pe)                       // reject: missing fact St(β)=Init on the ¬g# arm
                               //   M7: "prove cap_of(v) > len_of(v) before the push (e.g. reserve), or re-form pe"
reserve(v, len_of(v)+1); push(v, 5)
read(pe)                       // accept: arm A selected at the call; St(β)=Init unguarded  (P4)
```

### I18 container remove / pop

`premises:` `len_of(ρ) > 0`; `St(β[len_of(ρ)−1]) = Init`.
`⇒ effect:` `St(β[len−1]) := Uninit`; the element's obligation moves to the result
binding; `ensures len_of(ρ) = old(len_of(ρ)) − 1`; facts keyed by an element place
whose index is not `dis` from `len−1` are killed.

```text
x = pop(v)                     // accept: len_of(ρ)>0; St(β[len-1]):=Uninit; x holds the obligation
read(pe_last)                  // reject: missing fact St(β[len0-1])=Init — the slot is vacated (R1(a)(i))
read(pe_first)                 // accept: dis(β[0], β[len0-1]) by AFF given len0 > 1
```

### I19 pool insert

`premises:` `St(ρP.slots[k]) = Uninit`; `k` is a captured index binding (III8);
occupancy is the pool's **own written data** (III10), matched by the writer.
`⇒ effect:` `St(ρP.slots[k]) := Init`; the pool's declared invariant is re-established
by the operation's `ensures` in one `INV` step.

```text
match free_slot(P) {           // accept: the writer's own match on the pool's own data (HIS-D3-04)
  Some(k) => { put(ρP.slots[k], move node)   // accept: I6; St(ρP.slots[k]) := Init
               ensures Occupied(ρP,k) && Links }   // accept: INV, one written step
  None    => { /* writer's outcome arm */ }  // accept: R12 arm, R2 discharged on it
}
```

### I20 pool remove

`premises:` `St(ρP.slots[k]) = Init`; the pool's invariant's support is re-established
by the row's `ensures`.
`⇒ effect:` `St(ρP.slots[k]) := Uninit`; the element's obligation moves out; if `k` is a
runtime value not captured as a binding, `Occupied(ρP, ·)` is killed for every index
(III8's precision cliff).

```text
remove(P, m)                   // accept: row { ρP.slots[m] : Init ⇒ Uninit @W ; ensures !Occupied(ρP,m), Links }
read(deref(pm).data)           // reject: missing fact St(ρP.slots[m])=Init — every path to m is refused (R1(a)(i))
insert(P, x) reusing m         // accept: I19 — St(ρP.slots[m]) := Init again
read(deref(pm).data)           // accept on SAFETY, and it reads the NEW occupant.
                               //   Declared boundary: a type-stable pool never ends slot storage, so this is
                               //   R1(a)(i)-clean and a wrong-occupant logic error — open defect OD1 (owner O1)
```

### I21 pool compact — relocation in place

`premises:` `Carve(ρP) = ∅`; **`Span(ρP) = ∅`**; no live `⊑`-child; the operation's row
carries `relocates(ρP)`.
`⇒ effect:` every contents fact keyed by a place under `ρP` is killed; only facts the
row `ensures` survive (S-S8's repair: the pool declares an invariant that `compact`
`ensures`, otherwise `pool[h]` has no bounds fact afterwards).

```text
fn compact(P) row { ρP : Settled ⇒ Settled @E ; relocates(ρP) ;
                    requires Carve(ρP)=∅, Span(ρP)=∅ ;
                    ensures forall h. Occupied(ρP, slot_of(h)) }      // the fact pool[h] needs (S-S8)
focus ρP as { S = .slots[h] } { p = pointer_of(h)    // accept: p : ptr<ρP↓w.S, Node>  (II19)
  compact(P) }                 // reject: missing fact Carve(ρP)=∅ — M7 names the window head and "close it"
compact(P)                     // accept here: Carve(ρP)=∅ and Span(ρP)=∅ (p did not escape, II19)
read(pool[h])                  // accept: Occupied(ρP,slot_of(h)) from compact's ensures, one INV step
```

### I22 minting inside a loop

`premises:` a rule that mints (I1, I11, I13) occurs in a loop body.
`⇒ effect:` the minted name must, before the back edge, be (a) `Gone` (I7/I14), (b)
**merged** (I12), or (c) **sunk** into a written container whose loop-head row states
the quantified fact (`∀k ∈ live(P)`, a parametric key). Otherwise **reject** at the back
edge, naming the minting site. This is what keeps the name supply finite (S-S4's coarse
half) with no widening and no fixpoint.

```text
loop { (b,ρb) = carve(A, 64)
       use(b) }                // reject: ρb is live at the back edge — M7: "end it, merge it, or sink it"
loop { (b,ρb) = carve(A, 64)
       use(b); free(A, b) }    // accept: ρb is Gone at the back edge; the head sees one arena, not N names
loop { (b,ρb) = carve(A, 64)
       insert(P, b) }          // accept: sunk into P; head row states `∀k ∈ live(P). St(ρP.slots[k])=Init`
```

### I23 address reuse

`premises:` a formation (I1, I11) yields storage whose address overlaps storage that
is `Gone`.
`⇒ effect:` a **fresh** name is minted; no `Gone` name is ever revived; `dis` against the
`Gone` name is irrelevant because no access to it is admitted (I7).

```text
free(A, b2)                    // accept: St(ρ2):=Gone
(b4,ρ4) = carve(A, 32)         // accept: fresh ρ4 over ρ2's bytes — a NEW name (R1(a) closing line)
read(b2)                       // reject: St(ρ2)=Gone, permanently — not "b2 now aliases b4"
write(b4, 1); read(b4)         // accept: St(ρ4) Uninit ⇒ Init ⇒ read  (P2)
```

---

## 3. Rule set II — splitting and rejoining state

| # | Governs | # | Governs |
|---|---|---|---|
| II1 | guard capture at `if` | II11 | call: what is killed (frame) |
| II2 | arm entry | II12 | call: two-armed rows and outcomes |
| II3 | join at `if`/`else` | II13 | row subsumption (closures, fn-typed params) |
| II4 | join at `match` | II14 | window open (`focus`), incl. parametric entries |
| II5 | guard environment: relations, versions, liveness | II15 | window access rule |
| II6 | guard feasibility and **neutrality** (M2(ii)) | II16 | window close |
| II7 | loop head | II17 | window early exit (`return`, `break`, outcome) |
| II8 | loop back edge | II18 | nested windows; the carve forest |
| II9 | `break` / `return` edges and loop exits | II19 | the span-typed pointer and the escape discipline |
| II10 | call: what a signature states, what the caller learns | II20 | parallel split and rejoin |

### II1 guard capture at `if`

`premises:` the branch discriminant is a **binding** `c` of boolean type (a source
variable), or a `requires`-style predicate the checker introduced (II12).
`⇒ effect:` `Γg += atom(c, version)` if not already live; the then-arm is checked under
`c = true`, the else-arm under `c = false`. A discriminant that is an *expression* captures
nothing: the arms are checked under no new atom, and any state difference must then
survive II3's collapse or be written as a value.

```text
if is_empty(v) { … } else { … }   // accept: no atom captured (the discriminant is an expression)
                                  //   M7, when a later line needs the distinction:
                                  //   "bind the condition: `let c = is_empty(v); if c { … }`"
let c = is_empty(v)
if c { … } else { … }             // accept: atom(c,1) captured; both arms refine facts under it
```

### II2 arm entry

`premises:` an arm of a captured atom `c`.
`⇒ effect:` inside the arm, every read of `St`/`Facts` resolves the guarded value at
`c`'s arm value; every write writes **only** that arm's leaf.

```text
a = object(10); b = object(20)
if c { p = ref(a) } else { p = ref(b) }   // accept: p's identity is ite(c, ρa, ρb) — a guarded identity
q = p                                     // accept: q copies the guarded identity, not the variable name
write(p, 9)                               // accept: I3 writes the SELECTED leaf: St(ρa)=ite(c,Init,·)
read(q)                                   // accept: under each assignment q's target was just written (B03)
read(a)                                   // reject: missing fact St(ρa)=Init on the ¬c assignment
```

### II3 join at `if`/`else`

`premises:` two incoming edges under atom `c`.
`⇒ effect:` for every key of `St` and every fact in `Facts`, the merged value is
`ite(c, then, else)` in canonical atom order, with `ite(c,x,x) → x`. If the merged value's
guard depth would exceed `Gmax`, the **oldest** atom is collapsed by the meet `⊓` of the
automaton order (`Gone ⊓ Init = ⊥`, which makes every later access under that key
reject with the collapse named). No lattice widening, no fixpoint, no drop flag.

```text
if c { free(a) } else { free(b) }   // accept: St(ρa)=ite(c,Gone,Init), St(ρb)=ite(c,Init,Gone)
use(survivor)                       // this line depends on the writer's spelling:
read(a)                             // reject: missing fact St(ρa)=Init on the c assignment (M7 names both edges)
if c { read(b) } else { read(a) }   // accept: each arm resolves its own leaf  (P8(a) accepted verbatim)
free_survivor: if c { free(b) } else { free(a) }   // accept: exactly one end per assignment (R2)
```

### II4 join at `match`

`premises:` a `match` on a discriminant **binding** of a sum type.
`⇒ effect:` an atom per arm-set (a `k`-way atom, canonicalized as `k−1` nested boolean
atoms in arm order); otherwise II3. An arm that diverges contributes no edge.

```text
let r = checked_div(a, d)
match r { Ok(q)  => { s = q }              // accept: arm atom; R11 discharged inside the arm
          DivZero => { return E }          // accept: diverging arm contributes no edge to the join
        }
use(s)                                     // accept: s is bound on the only edge reaching the join
```

### II5 guard environment: relations, versions, liveness

`premises:` an assignment to a boolean binding, or a definition from another binding.
`⇒ effect:` a new **version** of the atom is created; the relation to the old version is
recorded when the right-hand side is `c`, `!c`, or a literal (`=`, `¬`, `const`) and is
otherwise unrecorded (the new atom is free); every guarded value mentioning the old
version keeps the old version, which stays live while any value mentions it.

```text
p = ref(a)
if cond { old = take(p) }      // accept: St(ρa) = ite(cond@1, Uninit, Init)
cond = !cond                   // accept: atom cond@2 created; relation cond@2 = ¬cond@1 recorded
if cond { write(p, 20) }       // accept: the arm is cond@2 = ¬cond@1, i.e. the NOT-taken path
read(p)                        // reject: missing fact St(ρa)=Init on cond@1 — still a hole there (CASES B09)
if !cond { put(p, 20) }        // accept: arm ¬cond@2 = cond@1, the taken path; then read(p) accepts
```

### II6 guard feasibility and neutrality

`premises:` a guarded value is consulted, or a program edge is emitted.
`⇒ effect:` **(a) feasibility**: an assignment of the live atoms is feasible iff it
satisfies every recorded relation (equalities and negations only); an obligation must
hold on every feasible assignment; `DIS`, `KILL` and `R2` all quantify over feasible
assignments. **(b) neutrality (M2(ii))**: no lowered branch, no release, no check and no
bookkeeping cell may be selected by an atom. Formally: the set of operations executed
by an accepted program is a function of the source's own control flow. A rule whose
discharge would need a guard-selected action is a **rejection with a named repair**, never
an inserted flag.

```text
a = object(10)
if c { release(a) }
if d { release(a) }            // reject: the assignment c ∧ d is feasible — double consumption (CASES B12)
                               //   M7: "prove ¬(c ∧ d), e.g. bind d = !c"
if c { release(a) }
if !c { release(a) }           // accept: no feasible assignment ends it twice; none leaves it unended
{ if c { free(a) } }           // reject at the scope exit (I8): a guard-selected cleanup would be a drop flag
```

### II7 loop head

`premises:` a loop head.
`⇒ effect:` the head carries a **row**. The default head row is the identity on every
`St` key and every fact. The writer writes a head row when (a) the body's composed
state edge is not the identity, (b) a name is minted in the body (I22), (c) a fact must
survive the back edge under a guard, or (d) the head's identities are existential
(a relative invariant). The checker verifies **two** entailments and nothing else:
*initiation* (the entry edge refines the head row) and *consecution* (body ∘ head
refines the head row). No fixpoint, no widening, no iteration count.

```text
p = ref(a); q = p
repeat n times { v = take(p); put(q, move v) }   // accept: no head row needed — body ∘ head = identity
read(p)                                          // accept: St(ρa)=Init, including n = 0  (CASES L01)

repeat n times { v = take(p) }                   // reject at the back edge: head says Init, body exits Uninit
                                                 //   M7 names the head state and the body's exit state (CASES L02)

head row { exists i1 i2. target(p)=i1, target(q)=i2, St(i1)=Init, St(i2)=Uninit, dis(i1,i2) }
repeat n times { v = take(p); put(q, move v); swap(p,q) }
                                                 // accept: consecution matches the binders POSITIONALLY by the
                                                 //   binding each is attached to (p, q) — a determined match,
                                                 //   no search; the swap maps the relation to itself  (CASES L03)
read(p)                                          // accept: St(target(p))=Init at every exit
```

### II8 loop back edge

`premises:` the back edge.
`⇒ effect:` atoms **captured inside the body** are retired (they do not cross the edge);
an atom that must cross is named in the head row and is then an ordinary head fact;
names minted in the body are subject to I22; every open window in the body is closed
(windows are statement-shaped, II16/II17).

```text
active = true
head row { ρa : ite(active, Init, Gone) }        // accept: the guard crosses because it is WRITTEN at the head
repeat n times { if active { read(p)
                            if stop { release(p); active = false } } }
                                                 // accept: consecution — the ¬active leaf is Gone on both paths
if active { release(p) }                         // accept: exactly one end on every feasible assignment (CASES L05)
                                                 // inserting read(p) between release and `active = false`:
                                                 //   reject — St(ρa)=Gone there, the head relation is not timeless
```

### II9 `break` / `return` edges and loop exits

`premises:` an edge leaving the loop.
`⇒ effect:` a `break`/`return` edge is **not** required to satisfy the head row; it carries
its own state to the loop's exit join, which is II3 over all exit edges (normal
completion, zero-trip, and each `break`). Every open window on the edge discharges
II17. Every R2 obligation is checked at the join, not at the head.

```text
repeat n times { if stop { release(p); break }
                 read(p) }                       // accept: the release path cannot reach this read (CASES L04)
read(p)                                          // reject: the exit join has St(ρa)=Gone on the break edge
repeat n times { v = take(p); if stop { break }; put(p, move v) }
release(p)                                       // accept: every exit edge has St(ρa) ∈ {Init, Uninit}; I7
                                                 //   admits both  (CASES L06)
```

### II10 call — what a signature states, what the caller learns

A row is the whole interface (R7). The grammar, with the clause kinds the critiques
found missing (S-S7) added:

```text
row { π : pre ⇒ post @g [carve { … }]        -- one line per identity touched; π is a PLACE, so a
                                             --   field-, element- or extent-level effect crosses the cut
       split ρ' ⊑ ρ at [lo,hi) : post        -- promotion (I11); the caller learns a new coarse child
       absorb [lo,hi) into free_of(ρ)        -- the arena-free clause (I14)
       relocates(ρ)                          -- the relocation clause (I21); requires Carve=Span=∅
       end ρ                                 -- the ending clause (I7)
       result : ι                            -- ι a coarse name, a fresh split, or a finite origin set
       requires φ ; ensures ψ                -- ordinary facts, incl. measures and old()
       outcomes { … }                        -- R12 arms, each with its own exit states
     }
g ::= @0 | @R | @W | @E
```

`premises (caller):` each formal store name is **instantiated** by an identity whose
span contains the call — a coarse name, a place, or an open window entry. (This is what
lets an entry be passed to a call with **no fine name in a written type**, C-S6's
missing mechanism.) Each `requires` is discharged; if a carve of an instantiated
identity's root is open, the whole row must lie within **one** entry (II15, II18).
`⇒ effect (caller):` apply each clause to `St`/`Facts` in written order; `KILL` per II11;
mint what `split` mints; learn `ensures`.
`premises (callee):` the body is checked **once** against the row.
`premises (declaration):` a row whose `requires` is syntactically unsatisfiable, or whose
`result` names an origin no caller can name, is rejected at the declaration (R7).

```text
fn drain(x: ptr<ρ,T>) row { ρ.f : Init ⇒ Uninit }      // accept: a PLACE-level exit state crosses the cut
drain(x); read(x.f)            // reject: missing fact St(ρ.f)=Init — from the row alone (S-F1 repaired)
read(x.g)                      // accept: St(ρ.g) untouched — the row named .f, so KILL spared .g (R6)
helper(slice_of(out, w*s..(w+1)*s), input)
                               // accept: the formal σ instantiates to the window entry for the call's extent;
                               //   nothing fine is written in any type  (P6)
alias ρ σ                      // accept ONLY when ρ and σ have a common ⊑ ancestor and EXT proves their
                               //   extents identical; otherwise reject — never a bare assume (M3, S-S5)
```

### II11 call — what is killed (frame)

`premises:` a row clause at grade `@W` or `@E` on identity `ι`, or a `relocates`/`end`
clause.
`⇒ effect:` a fact survives iff its support is `dis` from every written identity, **on
every feasible guard assignment**. Two kill classes are stated explicitly, because the
original model left one unnamed (S-S10):
 1. **footprint kill**: support meets a written identity.
 2. **containment kill**: support lies under a written identity in `⊑` (a write to a
    parent kills a child's contents; `dis` is false down the forest). A row that wants
    to spare siblings must name the *plane or place* it writes, not the parent
    (I13's `ρM` rather than `ρS`).
 3. **runtime-index kill**: a transition at a non-captured index kills every sibling
    element's facts (III8).

```text
carve(A, 64)                   // accept: the row writes ρM (metadata) only ⇒ contents facts about ρ1, ρ3 SURVIVE
                               //   (had the row said ρS @W, every live block's contents would be killed)
set_elem(v, i, x)              // accept: row { backing_of(ρ)[i] : Init ⇒ Init @W } ⇒ len_of(ρ) SURVIVES
                               //   (the measure's support is the descriptor plane, not an element)  (R6)
compact(P)                     // accept: relocates(ρP) ⇒ every contents fact under ρP dies except the ensures
```

### II12 call — two-armed rows and outcomes

`premises:` a row clause with `when φ | otherwise`.
`⇒ effect:` if `φ` is discharged, the first arm applies unguarded; if `¬φ` is discharged,
the second; otherwise a fresh atom `g#` is bound to `φ` and the post-state is the
guarded merge of the arms (II3), subject to `Gmax` and to II6(b). An `outcomes { … }` row
is different: it is a **source-visible** branch, so each arm is an ordinary edge and R2
discharges on each (R12).

```text
reserve(v, 100)                // accept: row { backing_of(ρ) : Init ⇒ Init when cap_of(ρ) >= 100
                               //                              | Init ⇒ Gone, split β' otherwise ;
                               //              ensures cap_of(ρ) >= 100 }    ← the ensures the original omitted
read(pe)                       // accept: the ensures makes the first arm's fact usable afterwards
push(v, x)                     // accept: with cap_of > len_of proved, arm A; otherwise St(β)=ite(g#,Init,Gone)
read(pe)                       // reject: missing fact on the ¬g# arm — M7 names the predicate to prove
```

### II13 row subsumption

`premises:` an argument of function type, or a closure, is passed where a row is
expected.
`⇒ effect:` accepted iff the actual row is **pointwise** below the formal: grades by
`@0 ⊑ @R ⊑ @W ⊑ @E`, states by equality, carves by refinement, `requires` implied by the
formal's `requires`, `ensures` implying the formal's. No width flexibility. Store
parameters on closures are **rank 1**: a closure's store parameters are fixed at its
formation site (C-S9's missing judgment, taken in its restricted form).

```text
fn map(g: ptr<ρ,Pool<T>>, f: fn(&ro T) -> T row { @R })
let h = |slice: ptr<σ, buffer<u8>>| helper(slice, input)
                               // accept: h's row is { σ : Uninit ⇒ Init @E ; τ : Init ⇒ Init @R }, τ the capture
map(g, |x| read_only_thing(x)) // accept: actual row @R ⊑ formal row @R, pointwise
map(g, |x| { write(other); x }) // reject: actual row carries @W on `other`, which the formal does not name
                               //   M7: "the callee's row admits @R only; the closure writes `other`"
```

### II14 window open — `focus`

`premises:` the carve's entries are written at the head; the obligations are written in
a `where` clause or discharged by `AFF`/`EXT`; entries must be provably pairwise
disjoint (`PART`).
`⇒ effect:` push a carve node onto `Carve` at the named identity; mint one window entry
per listed entry plus the complement entry `\S` when the carve is **partial**; **no state
is created or moved** — `St` is untouched (this is the whole of the S-F1 repair: a window
carries interference, not state).

Entry forms (closed grammar, with the parametric form C-S7 needed):

```text
entry ::= .f | backing(.f) | .f.g … (a resolved path, any depth — C-S8)
        | [t] | [lo..hi) | \S
        | [∀k ∈ E]            -- a PARAMETRIC entry over a loop or par_for binder;
                              --   its dis is proved ONCE per binder by the affine map (PAR-2), not pairwise
```

```text
focus β as { A = β[i], B = β[j] } where { i < len_of(β), j < len_of(β), i != j } {
                               // accept: PART discharges the three obligations at the head (AFF)
                               //   omit `i != j` and it cannot be closed: reject AT THE HEAD, naming it
  …
}
par_for k in 0..n { focus β as { E = [∀k] } { write(E, f(read(E))) } }
                               // accept: dis(E@k, E@k') per binder by the affine map (1,0) — O(1), not O(n²)
```

### II15 window access rule

> Inside a window carving `ι`, an access whose resolved place **meets** a carve entry is
> accepted iff it lies **provably within exactly one entry**, or provably covers a union
> of entries. Otherwise reject, naming the entry it meets, the window head, and three
> repairs: name the entry, prove the access lies in the complement, or move the access
> outside the window.

This is the repair of S-S3 / C-S1: the original text ("names that entry, or covers a
union") rejected `backing(.a)[i]` — i.e. every element access in P5, P3 and P16.
Consequence, stated because W4 denied it: a **total** carve is not vacuous. An access
through a runtime pointer whose resolved place is not provably within one entry is
rejected under a total carve too; the common case is discharged because a resolved
place through a formal names one field.

```text
fn kernel(s: ptr<ρ, Cols>) row { ρ : Init ⇒ Init @E }     // the signature's row IS a window (II18)
  set backing(.a)[i] = f(backing(.a)[i], backing(.c)[i])
                               // accept: the place lies provably within exactly one entry, backing(.a) / (.c)
  x = read(β[k])               // inside `focus β as {A=β[i],B=β[j]}`:
                               // reject: β[k] meets A and B and lies provably within neither.
                               //   M7 payload: rule II15, head location, missing fact "k != i && k != j"
  use k != i && k != j ;       // accept: a named application (INV), not a bare assume (owner O15(b))
  x = read(β[k])               // accept: now provably within the complement entry
```

### II16 window close

`premises:` the end of the `focus` statement.
`⇒ effect:` pop the carve node; retire the entry names; `Facts` keyed by a retired entry
are dropped; `St` needs **no** write-back (state was never in the window). Any span-typed
binding of the window is dead after this point (II19).

```text
focus β as { A = β[i], B = β[j] } where { … } {
  va = take(A)                 // accept: I4 on the PLACE β[i]; the entry only supplies dis
  put(A, move va')             // accept: I6 on the same place
}                              // accept: close. Nothing is written back; St(β[i]) already holds Init
push(v, x)                     // accept: Carve(β) = ∅, which I17 requires
```

### II17 window early exit

`premises:` an edge leaving a `focus` body (`return`, `break`, an outcome arm, a diverging
call).
`⇒ effect:` the edge performs II16 and then joins; every obligation the window's row
declares (its exit states, its affine values) is checked **on that edge**. This is the
repair of S-S2: the original attached the obligations to the closing brace only.

```text
focus β as { A = β[i] } { va = take(A)
                          if c { return }        // reject: va is an unconsumed affine binding on this edge
                          put(A, move va) }      //   and the window's declared exit state for A is Init.
                                                 //   M7 names the edge and both obligations (R2, R1(a)(i))
focus β as { A = β[i] } { va = take(A)
                          if c { put(A, move va); return }
                          put(A, move va) }      // accept: every leaving edge discharges both
```

### II18 nested windows; the carve forest

`premises:` a window opened over an identity that is itself an entry of an open window,
or a signature row that is a window containing a `focus`.
`⇒ effect:` `Carve` is a **forest**. `dis(m,n)` across levels: walk both to their lowest
common carve node; they are `dis` iff they descend from distinct entries of that node,
or `EXT`/`AFF` proves their places disjoint. A signature row's window is exempt from
I7/I9/I21's `Carve = ∅` test **for the identity the row itself names** (so
`fn destroy(x: ptr<ρ,T>) row { end ρ }` is writable — the repair of S-F6's second horn).

```text
fn destroy(x: ptr<ρ,T>) row { ρ : Init ⇒ Gone ; end ρ }
                               // accept: the signature's own window does not block its own ending clause
focus ρ as { A = ρ.left, B = ρ.right } {
  focus A as { A1 = A.hi, A2 = A.lo } {          // accept: a nested node under entry A
    write(A1, 1) ∥ write(B, 2) }                 // accept: lowest common node is the outer carve;
}                                                //   A1 descends from A, B is a sibling entry ⇒ dis
focus ρ as { A = ρ.left } { pick(c, x, y) }      // reject: pick's row names ρ whole, which meets A and is not
                                                 //   within one entry (II15) — M7: "close the window or carve
                                                 //   pick's row"
```

### II19 the span-typed pointer and the escape discipline — **O12(b), priced**

The unrepaired model typed every pointer at a coarse name, so P13's interior pointer
escaped its window and the use-after-relocation was admitted with no diagnostic (S-F2).
The repair needs a fine name in a type. It is taken, and priced.

```text
type   ::= ptr<ρ, T> | &ro ptr<ρ, T>          -- first class: copyable, storable, returnable
         | ptr<ρ ↓ w.k, T>                    -- SPAN-TYPED: second class
```

`premises (formation):` an operation whose row says `result : e` for a window entry `e`
of the open window `w`.
`⇒ effect:` the binding's type is `ptr<ρ↓w.k,T>`; `Span += (binding → w)`.
`ESC` (a syntactic judgment, §5): a span-typed binding may not be (i) stored in a field
or a container, (ii) returned, (iii) captured by a closure, (iv) passed where a
first-class pointer type is expected, (v) live after `w` closes. The **parser** rejects
`ptr<ρ↓w.k,T>` in a field, parameter or return position with a fixed message, so the
mistake is unmissable at the cut.

```text
focus ρP as { S = .slots[h] } {
  p = pointer_of(h)            // accept: p : ptr<ρP ↓ w.S, Node>  (the row's `result : S`)
  hold = Hold { at: p }        // reject: ESC(i) — a span-typed value in a field.
                               //   M7: "Hold.at is ptr<ρP,Node>; pass the handle, or promote with split"
  return p                     // reject: ESC(ii), same message shape
  read(deref(p).data)          // accept: inside w, St(ρP.slots[h])=Init
}
read(deref(p).data)            // reject: ESC(v) — p is dead after the close; the binding is out of scope
```

**Price of the second name kind in a type**

| Axis | Price |
|---|---|
| M4 | +1 type form, +1 judgment (`ESC`), and the reader must hold "a carve entry may appear in a local binding's type and nowhere else". Declared violation (§5) |
| M8 | +2 cards (`ptr<ρ↓w.k,T>`; the five `ESC` clauses) |
| M10 | `ESC` is linear in binding occurrences; no new reach |
| O7 | Projections stay **second class**. What is *recovered* relative to the unrepaired model: `split_at_mut`, a returnable slice and a byte-range cursor, because I11's promotion gives them **coarse** names with proved extents. What stays lost: an element-yielding iterator, a `get_mut` on a non-byte container, and any callback that stores a projection |
| O12 | Answered **(b)**: a span-scoped name appears in a type, governed by a scope discipline on bindings. Without it, S-F2's hazard is admitted with no diagnostic; that is the measured cost of (a) |

### II20 parallel split and rejoin

`premises:` an overlap construct (PAR-1 window, PAR-2 loop, `par_for`).
`⇒ effect:` **split**: each branch/iteration is checked against the same entry `St`, with
its footprint restricted to its own entry (a parametric entry for an iteration space,
II14). **Overlap admitted** iff for every pair of identities the two footprints touch,
both grades are `@R`, or `DIS` holds on every feasible guard assignment (R5(a)(i)).
**Rejoin**: the composed state edge is the *union* of the per-entry edges, which is
well-defined precisely because the entries are `dis`; a state edge on a **shared** key
(not `dis`) makes the construct ill-formed and is rejected (this rule is C-O3's unnamed
composition). At an erasable construct a missing fact leaves the program sequential
(M5); at a non-erasable construct it rejects (owner interim O2/O18(a)).

```text
focus β as { A = β[i], B = β[j] } where { i != j, … } {
  write(A, f(va)) ∥ write(B, g(vb))
                               // accept overlap: footprints {A}@E, {B}@E; dis(A,B) by the carve partition
                               //   rejoin: the edges are on distinct keys β[i], β[j] ⇒ their union is a function
}
par_for w in 0..k { helper(slice_of(out, w*s..(w+1)*s), input) }
  head row { ρ : [0, w*s) Init, [w*s, n) Uninit }      // written: the prefix-run invariant (C-S5)
                               // accept overlap: extents dis by EXT for w != w'; τ is @R on both
                               // accept rejoin: the per-iteration Uninit ⇒ Init edges cover [0, k*s) disjointly
par_for k in 0..n { s = s + a[k] }
                               // reject the overlap (not the program): the footprints share `s` at @W and s is
                               //   not dis from itself; ground (ii) is R5/D5 vocabulary this rule set does not
                               //   supply ⇒ erasable construct, sequential (M5)
```

### 3.21 The 26 hand-derived cases under these rules

Classification is required where a verdict differs from CASES.md (written under
DESIGN.md Candidate A). Nothing below is a capability loss; one is a deliberate refusal
with a named repair, two cost a written loop-head row.

| Case | Rules used | Verdict here | Same as CASES? |
|---|---|---|---|
| S01 aliases, stable copies | I3, I4 | accept | yes |
| S02 replacement transfers the old value | I5, R3 | accept | yes |
| S03 hole is a property of the target | I4 + `St` on the place | accept | yes |
| S04 another alias restores the hole | I6 | accept | yes |
| S05 hole need not be restored before end | I7 (`Init`/`Uninit` ⇒ Gone) | accept | yes |
| S06 replace requires old content | I5 premise | reject at the empty slot | yes |
| S07 release is permanent, every path | I7, I23 | reject every later access | yes |
| B01 one locator, alternative targets | II2, guarded identity | accept | yes |
| B02 target/initialization correlated | II2 (one atom carries both) | accept | yes |
| B03 write initializes the selected target | II2, I3 | accept; `read(a)` rejects | yes |
| B04 copied uncertain target, take/put | II2, I4, I6 | accept | yes |
| B05 correlated targets, selected release | II2, I7 | accept both reads and both ends | yes |
| B06 rebinding does not retarget copies | II2 (facts key on the guarded identity, not the name) | accept / reject as written | yes |
| B07 conditional hole, scalar overwrite | I3's `Uninit`/`Init` ⇒ Init | reject the read; accept after `write` | yes |
| B08 repeating an unchanged condition | II5 (same atom version) | accept | yes |
| B09 conditions are captured values | II5 (version + `¬` relation) | reject; accept the `!cond` variant | yes |
| B10 join must not erase obligations | II3 (guarded obligation) | obligation retained, guarded | yes |
| B11 locator names the remaining obligation | II2, II3 | accept | yes |
| B12 two conditional releases | II6(a) feasibility | reject the second; accept the `!c` variant | yes |
| B13 conditional scope cleanup | I8 + II6(b) | **reject at the exit, repair named** | **differs: deliberate refusal.** Automatic cleanup on one arm is a guard-selected release (M2(ii)); the repair `if !cond { release(a) }` is accepted |
| L01 take/restore preserve the head | II7 default row | accept | yes |
| L02 empty exit becomes the next input | II7 consecution | reject at the back edge | yes |
| L03 relative invariant, alternating roles | II7 with existential binders | accept, **cost: one written head row** | yes, with a stated cost |
| L04 release on a break edge | II9 | accept; post-loop unconditional use rejects | yes |
| L05 checked Boolean guards later iterations | II8 (guard written at the head) | accept, **cost: one written head row** | yes, with a stated cost |
| L06 a hole may leave through break | II9 + I7 | accept | yes |

Evidence this table carries for **O11**: 12 of the 26 cases (B01–B13 minus B10's framing,
L05) are decided by guarded states or guarded identities. Under the unrepaired equality
join, B01–B09, B11, B12 and L05 are rejections or losses of precision, and P8 has no
accepted spelling at all.

---

## 4. Rule set III — the identity-to-runtime-value bridge

VERDICT-CORE §6's third convergence place: *the unstated bridge between an identity and
the runtime values that determine it*. Every rule below has the same four parts, so the
writer learns one shape (M4): **connect / establish / kill / re-derive**.

| # | Runtime value | Static name it determines |
|---|---|---|
| III1 | (the schema all of them instantiate) | — |
| III2 | `len` | which element places are `Init`; R11 index domains |
| III3 | `cap` | which arm of I17 applies; whether `backing_of` survives |
| III4 | the data pointer of a container | `backing_of(ρ)`, the coarse backing name |
| III5 | `head` / wrap point | which extent of a circular run is `Init` |
| III6 | an arena bump offset | which extent I11 may promote |
| III7 | a block size | `extent_of(ρ')`, hence `dis` between siblings |
| III8 | a slot index held in a variable | the element place `π[t]`, hence the kill radius |
| III9 | a loop binder | a parametric entry `[∀k ∈ E]`, hence per-binder `dis` |
| III10 | an occupancy bit | nothing in the checker: it is the writer's own data |
| III11 | a generation counter | **nothing: declared absent** |
| III12 | a branch condition | a guard atom, hence every guarded fact |
| III13 | a descriptor cursor / foreign-written bytes | which contents facts survive an interleaving point |

### III1 the bridge schema

`connect:` a bridge is a **measure fact** `m_of(ι) = τ` in `Facts`, where `τ` is a term over
captured bindings and literals. It is not a state and not an identity; it is a fact
keyed by an identity, so `KILL` (II11) governs it like any other.
`establish:` only by (a) a formation rule's `ensures` (I1, I11), (b) a row's `ensures` at a
call, or (c) an `INV` application of a **written** invariant. Never by inference from a body,
never by a bare assume (M3, owner O15(b)).
`kill:` when the fact's support meets a written footprint, on any feasible guard assignment.
`re-derive:` by calling the operation whose row `ensures` it (`len(v)`), or by one `INV`
step against a written invariant. A re-derivation is always a **named, written** step; the
checker never searches for one.

```text
n = len(v)                     // accept: row { ρ : Init ⇒ Init @R ; ensures result = len_of(ρ) }
                               //   the bridge is now a fact keyed by ρ's descriptor plane
set_elem(v, i, x)              // accept: footprint backing_of(ρ)[i] ⇒ len_of(ρ) SURVIVES (II11, R6)
push(v, x)                     // accept: footprint names the descriptor ⇒ len_of(ρ) is KILLED
y = v[n-1]                     // reject: missing fact — len_of(ρ) was killed at the push.
                               //   M7: "re-bind n = len(v), or use push's ensures len_of = old+1"
```

### III2 `len`

`connect:` `len_of(ρ)` keys the container's **descriptor plane** `ρ.desc`, not its backing.
`establish:` formation `ensures len_of = 0`; `push` `ensures len_of = old(len_of)+1`;
`pop` `ensures len_of = old(len_of)−1`; `len(v)` returns it.
`kill:` a write whose footprint meets `ρ.desc`. An **element** write does not (this is the
mechanism that removes the measured 34-of-41 `len()` rebind tax; R6).
`re-derive:` `len(v)`, or the `ensures` of the operation that changed it.

```text
for i in 0..len(v) { set_elem(v, i, f(read_elem(v, i))) }
                               // accept: the loop bound's fact survives every iteration —
                               //   the body's footprint is backing_of(ρ)[i], dis from ρ.desc (II11)
                               // reject only if the body pushed: then len_of dies and M7 names the push
```

### III3 `cap`

`connect:` `cap_of(ρ)` keys `ρ.desc`; it is the term I17's arm predicate reads.
`establish:` formation; `reserve`'s `ensures cap_of(ρ) >= m`; `cap(v)`.
`kill:` a write meeting `ρ.desc`; **not** an element write.
`re-derive:` `reserve` (which also strengthens it), or `cap(v)`.

```text
reserve(v, len_of(v)+1)        // accept: ensures cap_of(ρ) >= len_of(ρ)+1  ← the clause the original row lacked
push(v, 5)                     // accept: I17 arm A selected AT THE CALL; no guard atom is introduced
read(pe)                       // accept: St(backing_of(ρ))=Init unguarded  (P4's required verdict)
```

### III4 the container's data pointer — `backing_of`

`connect:` `backing_of(ρ) = β` binds the container identity to a **coarse** name `β ⊑ ρ`
(the repair of S-S6/C9: the backing is not a window entry, so it can be ended, replaced
and named at a cut).
`establish:` formation `ensures backing_of(ρ) = β` with `extent_of(β) = [0, cap)`.
`kill / rebind:` I17 arm B (`St(β) := Gone`, `split β'`, `backing_of(ρ) := β'`); I9.
`re-derive:` there is no re-derivation of a `Gone` backing; the writer re-forms element
pointers from the container.

```text
free(v)                        // accept: row { ρ : Init ⇒ Gone ; backing_of(ρ) : Init ⇒ Gone ; end ρ, end β }
                               //   naming BOTH — otherwise reject at I7 (a live ⊑-child) and R2 would leak
pe = elem_ptr(v, i)            // accept: pe : ptr<β, u64> where β = backing_of(ρ), an ordinary coarse pointer,
                               //   storable and returnable  (P4's Cursor; no window needed)
```

### III5 `head` and the run shape

`connect:` `head_of(ρ)` plus `len_of(ρ)` determine which **extent** of the backing is `Init`
under the shipped sequence automaton (full array / prefix run / circular run).
`establish:` the container's declared invariant plus each operation's `ensures`.
`kill:` a write meeting `ρ.desc`.
`re-derive:` one `INV` step against the declared run invariant.

```text
type Ring states { Full, Prefix, Wrapped } invariant
  Prefix  => St(backing[0 .. len_of])       = Init
  Wrapped => St(backing[head_of .. cap])    = Init && St(backing[0 .. head_of+len_of-cap]) = Init
x = read_elem(r, j)            // accept: with `Prefix` and j < len_of, one INV step gives St(β[j])=Init
                               // reject under `Wrapped` without j's placement: M7 names the disjunct to pick
```

### III6 the arena bump offset

`connect:` `off_of(A)` determines `free_of(ρS) = [off_of(A), cap_of(ρS))`, the extent I11
may promote from. This is the discharge the unrepaired model lacked (C-F4).
`establish:` `arena_new` `ensures off_of = 0`; `carve` `ensures off_of = old(off_of)+n`;
the arena's **written** invariant ties `off_of` to `free_of`.
`kill:` any write meeting the arena's metadata plane `ρM` other than through its own rows.
`re-derive:` `INV` against the written invariant, one step per `carve`.

```text
invariant Bump : free_of(ρS) = [off_of(A), cap_of(ρS)) && carved_of(ρS) = [0, off_of(A))
(b1,ρ1) = carve(A, 64)         // accept: [off, off+64) ⊆ free_of(ρS) by Bump — the promotion's obligation
(b2,ρ2) = carve(A, 64)         // accept: off advanced by the ensures, so Bump re-holds; the new extent is
                               //   disjoint from ρ1's because it lies in the residue
fill(b1) ∥ fill(b2)            // accept overlap: dis(ρ1,ρ2) from EXT over the two extents — NOT from
                               //   inequality of names (R4(b), C3, P2's negative test)
```

### III7 a block size — `extent_of`

`connect:` `extent_of(ρ')` is the half-open range that makes `DIS` decidable between
`⊑`-siblings.
`establish:` I11's promotion clause (the only introducer).
`kill:` never while `ρ'` is live — the extent is fixed at the promotion; ended at I7.
`re-derive:` not needed; it is immutable.

```text
(b4,ρ4) = carve(A, 32)         // accept: extent_of(ρ4) = [lo,lo+32) from the split clause
dis(ρ4, ρ1)                    // accept: EXT proves [lo,lo+32) ∩ extent_of(ρ1) = ∅
dis(ρ4, ρS)                    // FALSE, correctly: ρ4 ⊑ ρS — the containment relation refuses it (R4(b))
```

### III8 a slot index held in a variable

`connect:` an element place is `π[t]` where `t` is a **captured index binding** (a source
variable, with a version, exactly like a guard atom). Two element places are `dis` iff
`AFF` proves their terms unequal.
`establish:` the binding's definition; a row's `ensures` (`ensures result = k`).
`kill:` assignment to the binding retires the version (facts keyed by the old version
keep the old version, as II5 does for guards); a transition at an **uncaptured** runtime
index kills the fact for every sibling element.
`re-derive:` re-bind the index, or discharge from a written invariant.

```text
let m = find_slot(P)           // accept: m is a captured index binding (version 1)
remove(P, m)                   // accept: row { ρP.slots[m] : Init ⇒ Uninit } — the kill radius is slot m only
read(elem(P, k))               // accept if AFF proves k != m; else reject naming "k != m"
remove(P, next_free(P))        // accept the call, but the index is UNCAPTURED ⇒ Occupied(ρP,·) dies for every
                               //   index. M7: "bind the index first: `let j = next_free(P); remove(P, j)`"
```

### III9 a loop binder — parametric entries

`connect:` a parametric window entry `[∀k ∈ E]` binds one entry name to the whole
iteration space; `dis` is proved **once per binder** by the affine map (PAR-2), never
pairwise, which is what keeps unbounded element partitions representable (S-S4/C-S7).
`establish:` at the window head, with the affine map of every access through the entry.
`kill:` at the window close; inside, by any access whose map is not affine in the binder.
`re-derive:` re-open the window.

```text
par_for k in 0..n {
  focus backing_of(ρ) as { E = [∀k] } {
    set E.data = f(read E.data)       // accept: every access through E uses the map (1,0) over k ⇒
  }                                   //   dis(E@k, E@k') for k != k' — one proof, O(1), not O(n²)
}
par_for k in 0..n { set elem(perm[k]).data = 1 }
                                      // reject the overlap: the map perm[k] is not affine in k ⇒ no dis.
                                      //   Erasable construct ⇒ sequential (M5); the program stands
```

### III10 an occupancy bit

`connect:` **nothing in the checker**. Occupancy is the pool's own written data (owner
interim O17(a)); `Occupied(ρP,k)` is a declared invariant term over that data, not a
checker structure. The checker maintains no free list and no bitmap, so M2(ii) has
nothing to object to.
`establish:` the pool's written invariant plus its operations' `ensures`.
`kill:` II11 over the row's footprint; III8's radius for an uncaptured index.
`re-derive:` the writer's own `match` on the pool's data, or one `INV` step.

```text
match slot_state(P, k) {       // accept: the writer's own match on the writer's own value (HIS-D3-04)
  Empty  => fill(P, k, v)      // accept: the arm gives Empty; I19 applies
  Filled => { }                // accept
}
read(elem(P,k))                // accept: both arms leave St(ρP.slots[k])=Init; II3 joins them  (P11)
```

### III11 a generation counter — declared absent

`connect:` nothing. There is no generation, no epoch and no version cell.
`consequence, stated:` a handle into a **type-stable, reusing** pool that was removed and
whose slot was refilled reads the *new* occupant. That is memory-safe (the slot's storage
never ended, so R1(a)(i) and (ii) are both satisfied) and it is a wrong-occupant logic
error. Storage that genuinely ends (I11's promoted arena block, I7's heap object) is
refused by `Gone`, which is where R1(a)(ii) does its work.
`why absent:` a runtime generation compare is a safety predicate the checker did not
discharge (M2(ii)); a ghost generation is a fourth name kind this candidate does not
buy. Open defect **OD1**; the owner's O1/O3 decide whether this is a defect or a boundary.

```text
h = handle(P, m)
remove(P, m)                   // accept: St(ρP.slots[m]) := Uninit
read(elem_of(h))               // reject: missing fact St(ρP.slots[m])=Init  (R1(a)(i)) — good
insert(P, x) reusing m
read(elem_of(h))               // accept — and reads x, not the removed node.  DECLARED: wrong occupant,
                               //   not memory unsafety; no diagnostic exists for it here  (OD1)
```

### III12 a branch condition — the guard atom

`connect:` a captured boolean binding version *is* the guard atom; every guarded state,
guarded identity and guarded obligation is keyed by it (II1–II6).
`establish:` at an `if`/`match` whose discriminant is a binding; at II12's two-armed call
(a fresh atom `g#` bound to the row's predicate).
`kill:` assignment to the binding (a new version; the relation is recorded when it is
`=`, `¬` or a literal); the back edge of a loop, unless written in the head row (II8);
the `Gmax` collapse (II3).
`re-derive:` re-test the same binding version (CASES B08), or write the head row.

```text
let c = check()
if c { take(p) }               // accept: St(ρ) = ite(c@1, Uninit, Init)
if c { put(p, v) }             // accept: the SAME version c@1 ⇒ the arms line up; St(ρ) = Init afterwards (B08)
c = compute()                  // accept: version c@2, no relation recorded
if c { put(p, v) }             // reject at a later read: c@2 is unrelated to c@1 — M7 names both versions
                               //   and the repair "test the same value, or bind the relation"
```

### III13 a descriptor cursor and foreign-written bytes

`connect:` a foreign-writable identity carries a declared access class (R14(iv)) and a
foreign-agent automaton state; a shared description's cursor is a measure on the
description identity, not on either wrapper.
`establish:` the operation's `ensures` (`ensures pos_of(D) = old(pos_of(D)) + n`).
`kill:` **every** contents fact about a foreign-class identity dies at every point the
agent could interleave (R14(iv)); a `seek` through either wrapper kills the cursor fact
for both, because both name one description identity.
`re-derive:` a program-observed operation that `ensures` it.

```text
f1 = open(path); f2 = dup(f1)  // accept: three identities — ρw1, ρw2 (wrappers) and ρD (description);
                               //   pos_of keys ρD, so both wrappers see one cursor
n = read(f1, buf, 100)         // accept: outcomes { Full | Short(n) | Error } — each arm discharges R2 (R12)
seek(f2, 0)                    // accept: ensures pos_of(ρD) = 0 — and it kills the pos fact learned through f1
read(f1, buf, 100)             // accept: the cursor fact is re-derived from seek's ensures, one step
map_for_device(buf)            // accept: ρbuf enters the foreign access class; every contents fact about it
                               //   dies here and at every later interleaving point (R14(iv))  (P12's shape)
```

---

## 5. The judgment families

Every decision procedure the rules above invoke, named once. Quantities, all
specification-defined and all measured on one declaration unless stated: `S` statements,
`K` keys of `St`, `F` facts, `E` entries of a carve, `D` resolved-path depth, `A` row
clauses, `B` binding occurrences, `W` written proof steps, `L = 2^Gmax` guard leaves
(a **constant**, provisionally 4), `N` live coarse names.

| # | Family | Inputs | Output | Termination | Degree (M1, M10) |
|---|---|---|---|---|---|
| 1 | `COVER` | `St`'s key tree, a place | the key governing it, or the meet of the keys it meets | descends a finite key tree, one level per step | O(D) |
| 2 | `STATE` | a statement, `St`, `Γg` | `St'` | one syntax-directed pass; no rule revisits a statement | O(S · L) |
| 3 | `GUARD` | `Γg`, a guard cube | feasible / infeasible, canonical form | union–find over ≤ `Gmax` atoms with only `=`/`¬` relations | O(Gmax · α) per query |
| 4 | `JOIN` | two guarded maps, an atom | the canonical merge, with the `Gmax` collapse | canonicalization is a fixed ordering; the collapse strictly reduces depth | O(K · L) |
| 5 | `DIS` | two identities | true / false (never "unknown" — total) | recursion follows `⊑` depth and path depth, both finite | O(D + PART + EXT); for origin sets, O(&#124;π&#124;·&#124;π'&#124;); for guarded identities, O(L) resolutions |
| 6 | `EXT` | two half-open extents over affine terms | disjoint / contains / overlaps | one normalization, then `AFF` | O(term) + `AFF` |
| 7 | `AFF` | linear index and extent obligations | discharged / not | the existing fixed affine family; runs to its specified completion | **inherited; measured 2.5–3.4 exponent** — see declared violations |
| 8 | `PART` | a carve's entry list | partition / reject, with the missing obligation named | finite list; each pair or binder checked once | O(E) for field/backing/path entries; **O(E²)** for element and extent entries; O(1) per parametric binder |
| 9 | `KILL` | a footprint, `Facts` | `Facts'` | one pass over facts | O(F · DIS) |
| 10 | `INST` | a row, the actuals, `St` | the caller's post-state, or a rejection | one pass over row clauses, each projected once | O(A · D) |
| 11 | `ROWSUB` | two rows | below / not | pointwise over clauses; no width search | O(A) |
| 12 | `ESC` | a span-typed binding's occurrences | escapes / does not | syntactic, one pass | O(B) |
| 13 | `AUTO` | a nominal's automaton, an edge | admitted / not | table lookup | O(1) |
| 14 | `INV` | a **written** invariant, a written instantiation | verified / not | verifies a written step; never searches for one (owner O15(b)) | O(W) |
| 15 | `MEAS` | a required measure term, `Facts` | discharged / not | one lookup plus `AFF` on the term | O(F) + `AFF` |
| 16 | `LOOPCHK` | head row, entry edge, body's composed row | initiation ∧ consecution | two `JOIN`-refinement comparisons; **no fixpoint, no widening** | 2 · O(K · L) |
| 17 | `OUT` | an `outcomes` clause | per-arm edges | bounded by the **written** arm count | O(arms) |

**M1 — satisfied.** Acceptance is one syntax-directed pass (`STATE`), plus `LOOPCHK`'s two
entailments at each head, plus the fixed families above. No search, no budget, no
iteration cap, no solver state, no order dependence: `JOIN`'s canonical order is capture
order; `PART`'s obligation order is written order; `GUARD`'s relations are equalities and
negations only; `Gmax`'s collapse drops the *oldest* atom, a deterministic choice.
Every family runs to its specified completion.

**M2 — satisfied, and it is rule II6(b) that makes it so.** Nothing in §1.2 exists at run
time. No drop flag (II3 keeps the difference as a fact, and II6(b) forbids a
guard-selected release, so B13-shaped cleanup is a *rejection*, not an inserted branch).
No generation, no epoch, no pin count (a "pin" is an open window or a live span binding),
no occupancy structure (III10 is the writer's data). Guard atoms are checker facts that
divide facts and never select an operation.

**Declared violations.**

| Row | Declaration | Ground |
|---|---|---|
| **M4** writer regularity | **violated** | Three identity kinds (coarse name, place, window entry) plus guarded values; two pointer type forms (`ptr<ρ,T>`, `ptr<ρ↓w.k,T>`); the window rule II15 has a stated scope a reader must hold. AI-D1-11's own tension line, now with the third kind the S-F1 repair requires |
| **M8** teaching budget | **violated** | The taught inventory is the row grammar and its seven clause kinds, four grades, the carve grammar with seven entry forms, totality, `focus`/`where`, II15 and its three repairs, the guard machinery (capture, versions, relations, feasibility, neutrality, `Gmax`), the place/`COVER` scheme, the split/merge pair, `ESC`, the default and three shipped automata, and the bridge schema with twelve instances: **≥34 cards**, ≥10–12k tokens by the cost critique's count. K, its tokenizer and the subsystem share are unset (owner O6), so the figure is unfalsifiable as well as over |
| **M10** degree ceiling | **violated (two grounds)** | (i) `AFF` is inherited with a **measured** exponent of 2.5–3.4 (`proof-use-cost/baseline-2026-09-14.tsv`, `growing` N=16/32/64 → 30.0 / 165.4 / 1690.7 ms) against the ceiling "never above cubic, at most quadratic per declaration", and this rule set *adds* obligations to it (every `PART`, `EXT`, `MEAS`). (ii) Edit reach: the default carve is a function of a **declared type**, so a one-token change to a field's type changes the entry set of every row naming that type — a reach looser than the interface, which is M10's own boundary |
| **R1(a)(iii)** stale layout | **not owned** | No layout axis exists in `Auto`, no layout edge in rule set I. R1(a)(i) and (ii) are delivered; (iii) is not. Open defect OD5 |
| **R5(ii)/(iii), R13, R14(i)(ii)(v)(vi), R8b** | **not supplied** | D5/D10/D11/R14 vocabulary. These rule sets neither supply nor obstruct them |

**Not violated, stated to avoid an implied claim:** R15's check-once clause is met only
because the default carve is a function of the **declared** type, with each type parameter
contributing one opaque entry (C-F2's repair). The loss is real and named in OD6: P5
generic over its column type gets one entry per parameter, not per instantiated column,
unless the writer writes the carve.

---

## 6. Open defects

Named, not repaired. Each states what it costs and what would close it.

| # | Defect | Cost | What would close it |
|---|---|---|---|
| **OD1** | **Wrong occupant in a type-stable reusing pool.** III11: no generation, ghost or otherwise. A handle whose slot was removed and refilled reads the new occupant with no diagnostic | P3's "removal invalidates every path to `m`" is delivered at R1(a)(i) granularity (the vacated read) and not for the refilled slot | Owner O1/O3: a ghost generation would be a fourth name kind with its own erasure theorem; this candidate does not buy it |
| **OD2** | **`Gmax` is an unjustified precision cliff.** The collapse in II3 is deterministic and diagnosed, but no evidence sets `Gmax = 2`; a program with three live correlated conditions loses the correlation | Rejections whose repair is "restructure the branches"; the CASES set never needs three, which is weak evidence | A measured corpus count of live correlated guards per place |
| **OD3** | **`AFF`'s measured exponent** (M10(i) above) | Every `PART`, `EXT` and `MEAS` obligation inherits it | A fragment change in the affine family, outside these rule sets; a carve-obligation series pinned to the same bundle |
| **OD4** | **M8 is unmeasured as well as over.** No tokenizer is named, K and the subsystem share are unset (owner O6) | The token figure above cannot discriminate anything | Owner O6 |
| **OD5** | **No layout axis** (R1(a)(iii)) | A stale-layout access is refused by nothing here | A layout state on `Auto` with edges on I9/I21, priced against M4 and M8 which are already declared violated |
| **OD6** | **The declared-type default carve loses facts under abstraction.** `Cols<E>` gets one opaque entry per parameter | P5 generic over the column type needs a written `carve` clause to recover the eight per-column facts; M9's headline is instance-conditional | A per-instance carve would defeat R15(a)'s check-once clause — the two cannot both hold, and this is the trade, not an oversight |
| **OD7** | **Element-level projection across a cut remains unwritable.** I11's promotion gives byte-extent splits coarse names, so `split_at_mut` and slices cross cuts; an **element** of a non-byte container has no merge rule, so `get_mut` and element-yielding iterators are still second class (II19's `ESC`) | O7's boundary, narrowed but not removed | An element promotion with a merge rule, which needs per-element extents in `⊑` and reopens S-S4's minting bound |
| **OD8** | **Concurrent window nesting (C11).** II18 states the sequential forest; two windows on one carve held by two threads is undecided | D10's lock-as-window construction is stated but not checked | R14(i)'s ordering vocabulary; not a D1–D4 question |
| **OD9** | **The emitter's place→IR map is uncharged** (C-S18, owner O9). II15 gives every checked access a carve entry, but something must carry that entry to the lowered load/store | M9's "no emission-time re-derivation" claim is conditional on owner O9's strict reading | O9, plus a priced side table that survives lowering (unroll already forces scope duplication) |
| **OD10** | **`alias ρ σ` is now checkable but narrow.** II10 admits it only for names with a common `⊑` ancestor and `EXT`-identical extents | Two independently formed identities that are in fact one storage — a foreign call returning a pointer the program already holds — cannot be related at all | An M3-enumerated boundary declaration carrying the coincidence as a named entry with an obligation |
| **OD11** | **Guard atoms do not cross a back edge unless written** (II8). An unproved I17 arm inside a loop re-splits every iteration | The writer writes the head row, or proves capacity before the loop | Nothing cheap: carrying an unwritten atom across a back edge is the fixpoint II7 exists to avoid |
| **OD12** | **`PART` is O(E²) for element and extent entries** (C-S2). A `focus` with `k` element entries costs `k(k−1)/2` inequalities, each written by the writer when `AFF` cannot close it | P1 writes one at `k = 2`; a four-way split writes six | The parametric entry (III9) avoids it when the entries are a binder family; an explicit heterogeneous split does not |

**One defect the rules deliberately do not repair, recorded so it is not read as an
oversight:** II6(b) makes B13-shaped automatic scope cleanup a rejection. A model that
released on one guard arm would need a runtime flag, which M2(ii) refuses by name. The
writer writes the other arm; the rule set accepts that spelling (I8's second example).


# File: rules-brand-context-bounded.md

# Candidate `brand-context-bounded` — the three rule sets, written out

Key: **brand-context-bounded**. Date 2026-09-16.

Base: `core/candidate-brand-context.md` (D1 allocation-scoped location name ×
D2 implicit checker context × D3 three states refined by condition terms × D4
unrestricted sequential aliasing), with the two mandatory changes:

1. **Condition terms are bounded.** §2's `ATOM` and `TERM` rules fix the
   canonical form (one atom per identity per point), the guard-context
   restriction, and the frozen-operand rule that removes atom death. §5 gives
   the terminating decision procedure for term equality and entailment.
2. **The judgment families are named.** §5 names nine, each with inputs, a
   decision procedure, a termination argument and a degree. The base's "exactly
   four automatic families and no fifth" is **withdrawn**, not defended.

Also repaired by rule, each named at its rule: the double `take` at a symbolic
index duplicating an affine value (critique-soundness F1 → `I-7`, `KEY`), the
traversal cursor reading ended storage (F2 → `I-24`, `KEY`, `III-6`), the
may-write marking both origins `Init` (F4 → `II-15`, `JF-ORG`).

No candidate is compared here and none is selected. Where a rule refuses what
`CASES.md` expected, §7 says whether the difference is a capability loss, a
deliberate refusal with a named repair, or a correction of the case.

---

## 1. The model on one page

### 1.1 Names

| Object | Grammar | Who writes it |
|---|---|---|
| identity path `π` | `'a \| π.f \| π[i] \| π[lo..hi) \| π@g` | `'a` in signatures and nominals; the rest is formed by the checker from the access |
| ghost index `@g` | a minted, erased, never-compared tag on a *reusable* extent (arena bytes, pool slot) | nobody; minted at `I-20`, `I-23` |
| frozen value `x` | an SSA scalar binding, an `old(e)` snapshot, a loop-head φ-value, a ghost index | the writer binds it with `let`, or the checker snapshots it at a guard |
| atom `α` | `x ⋈ y+c` (`⋈ ∈ {=,≠,<,≤}`), `b`, `¬b`, `π # π'` — **every operand frozen** | the guard of an `if`/`match`; a proposition in `requires`/`ensures`/`invariant`/`use` |
| state `s` | `Init \| Uninit \| Live \| Gone \| ⊤` | nobody |
| state term `T` | `s \| ite(α, s, s)` — **depth ≤ 1** | nobody |
| origin `O` | `π \| ite(α, π, π) \| {π₁…πₙ}` | `{…}` appears in a signature as `'x\|'y`; `ite` appears as an `ensures` |

`ptr<'a,T>` is copyable, has no mode, no duration, no region. `own<'a,T>` is an
affine binding carrying `'a`'s release obligation. `Ρ` is an **identity row**
(an ordered list of identity parameters) — the repair to the nominal-arity hole
(critique-cost F3); `N<Ρ>` is well-kinded at every arity.

### 1.2 The state lattice

```text
              ⊤                    ⊤    = Live ⊔ Gone: no state is known
             / \
          Live  Gone               Live = the storage exists, occupancy unknown
          /  \                     Gone = the storage has ended, permanently
       Init  Uninit
```

`Live` is the change that makes `CASES.md` L06 derivable and keeps L04 refused:
a join of `Init` and `Uninit` with no atom to discriminate is `Live`, not `⊤`,
and `free` accepts `Live` for a content type with no obligation.

### 1.3 What the checker holds, per program point

```text
st(π)        : T                 state term
own('a)      : held | discharged | ite(α, ·, ·)
owns('a)     : {π …}             the identities 'a's end also ends  (S6 repair)
obl(π)       : none | one        an undischarged content obligation at π (F5 repair)
layout(π)    : a variant/representation term
origin(v)    : O                 for every pointer-valued binding v
measure(π)   : len, cap, head, gen, slot → a frozen value  (rule set III)
RSt(π[lo..hi), s, X) : range state with a residual exception list X  (F7 repair)
val(π)       : a contents fact, keyed by its support paths
π ≤ π'       : containment
π # π'       : proved distinctness — never from inequality of names
Γ(point)     : the guard assumption list of the enclosing arms
```

Every state leaf carries the **source site of the transition that set it**, and
every `⊤` carries the **operation that widened it**. Both are M7 payload.

### 1.4 What the writer writes

| Site | Written | Zero when |
|---|---|---|
| signature | identity parameters or one row `Ρ`; `requires`/`ensures` over `st`,`own`,`obl`,`owns`,`layout`,`backing`,`len`,`cap`,`head`,`gen`,`slot`,`#`,`≤`,`RSt`; `reads`/`writes`/`ends`; `consumes`/`yields`; `exists 'b` | never — one parameter minimum per named storage |
| nominal | the full identity row, **closed** (no free identity in a field type) | never for a pointer-carrying field |
| loop head | `invariant` where a state, an own, an origin or a range fact at the head is not a constant and not a term over a head-frozen atom already in `Γ` | L01, L04's body, P5, P6, P8's loop |
| join | nothing, always | always |
| statement | a branch, or `use R(args)` = one application of a **named rule with checked premises** (never a bare assume — O15(b)) | P2, P3's writes, P5, P7 |

### 1.5 What is erased

Every identity name, ghost index, state, own, obligation, layout, measure
binding, atom, term, residual list, `use` step and clause. Lowering receives
machine pointers plus the R4 channel (proved-distinct pairs, per-span write
footprints, termination witnesses). No lowered branch tests anything the source
did not branch on.

---

## 2. The two bounding rules (mandatory change 1)

### ATOM — frozen operands; atoms never die

**Premises.** Every operand of an atom is a *frozen value*: an SSA scalar, an
`old(e)` snapshot bound at a call, a loop-head φ-value, or a ghost index. A
guard mentioning a **derived measure** (`len(v)`, `cap(v)`, `head(A)`) is
captured as an atom over the frozen value the bridge rule `III-1` binds at the
guard, not over the measure head.

**Effect.** An atom is immutable and immortal. A write never kills an atom; it
kills the *bridge fact* `len('v) = n₀` (rule `III-2`), which is a separate
object. `⊤` is therefore never produced by atom death, and the base's collapse
case disappears. This is the repair to critique-cost S7 and to soundness m3.

```text
if len(v) == 0 { free(a) } else { free(b) }
                       // capture: let n0 = len(v) (III-1); atom is `n0 = 0`
                       // accept: own('a) = ite(n0=0, discharged, held)
push(v, x)             // accept: writes 'v. kills the BRIDGE fact len('v)=n0.
                       //   the atom `n0 = 0` is untouched: n0 is frozen.
if n0 == 0 { use(b); free(b) } else { use(a); free(a) }
                       // accept: Γ={n0=0} entails the then-leaf; JF-ENT.
                       //   base candidate: reject with no taught route. repaired.
read(v[0])             // reject: len('v) is unbound after push.
                       //   missing fact: 0 < len(v). repair: rebind `let n1 = len(v)`
```

### TERM — the canonical form and the bound

**Bound.** *A state term, an own term and an origin term carry at most **one**
condition atom.* The canonical form is `ite(α, l₁, l₂)` with `l₁ ≠ l₂` leaves,
or a constant. There is no `ite` nesting anywhere in the checker's state.

**Guard-context restriction (GCR).** At a point π inside arms with guard
assumption list `Γ(π)`, every term is *evaluated* under `Γ(π)` before use
(`JF-TERM` step 1). A term whose atom is entailed or refuted by `Γ(π)` collapses
to a constant. A term therefore only survives to a point where its atom is
**independent of every enclosing guard** — which is what "restricted to the
syntactically enclosing guards" buys: accumulation inside nested arms is
impossible, because entering an arm strictly shrinks terms.

**Where a second atom would be needed, the rule refuses.** If a join would
produce a term whose arms are themselves terms over a *different* atom, the join
is rejected, naming both atoms, the identity, and two repairs: branch on the
outer atom first, or close the inner divergence inside the arm.

```text
if c1 { if c2 { take(p) } else { } } else { if c3 { } else { take(p) } }
   // inner joins: st('A) = ite(c2, Uninit, Init)  [arm c1]
   //              st('A) = ite(c3, Init, Uninit)  [arm ¬c1]
   // outer join: reject.
   //   "TERM: joining 'A would carry two atoms (c2 at L1, c3 at L3).
   //    repair: hoist the take above the inner if, or restore 'A in each arm."
if c1 { if c2 { take(p); put(p, v) } else { } } else { ... }
   // accept: the inner divergence is closed inside the arm; both arms exit Init
```

**Consequence.** Term size is O(1); the base's declared 2^d M10 amendment is
**withdrawn** — the exponential family no longer exists. What replaces it is a
refusal with two named repairs, and §7 records which `CASES.md`/`PROGRAMS.md`
lines pay for it.

### KEY — how a path resolves to a fact (the structural repair)

The base resolved `st`'s key by literal path identity everywhere and modulo
may-equality at one line. Here it is one rule.

**May-equal class.** `mayEq(π)` = every live path `π'` with the same base and
the same shape whose index/extent arguments are **not** proved distinct by
`JF-OVL`+`JF-ENT`+`JF-LIN`. `π ∈ mayEq(π)` always.

- **Read of a fact at π**: the value is `⊓` over `mayEq(π)` (the *worse* state).
- **Strong update at π** (the target's own state changes to `s`): permitted only
  when `mayEq(π) = {π}`.
- **Weak update at π** (`mayEq(π) ⊋ {π}`): `st(π) := s`, and for every other
  `π' ∈ mayEq(π)`, `st(π') := st(π') ⊔ s`.

```text
qi = take(v, i)     // accept: st('b[i]) = Init, mayEq = {'b[i]} (no other symbolic
                    //   path live). strong: st('b[i]) := Uninit
qj = take(v, j)     // reject: mayEq('b[j]) = {'b[j], 'b[i]}; read gives Init ⊓ Uninit
                    //   = Live. take requires Init.
                    //   "R1(i): 'b[j] may be the element taken at L1 (i = j).
                    //    missing fact: i # j"   <- soundness F1 repaired by rule
if i != j { qj = take(v, j) }
                    // accept: Γ = {i ≠ j} discharges mayEq to {'b[j]}; strong update
```

---

## 3. Rule set I — storage creation, ending, replacement, relocation, reuse

Form: **premises ⇒ effect on the checker's facts**. 24 rules.

### I-1 `alloc` — heap creation

**Premises** the allocator's contract holds. **Effect** mint fresh `'a` by
existential unpack; `st('a) := Uninit`; `own('a) := held`; `obl('a) := none`;
`owns('a) := {}`; for every live identity `x` that is not an ancestor of `'a`,
record `'a # x` **from the allocator's written `ensures`**, never from name
inequality.

```text
(p, o) = alloc<T>()   // accept: exists 'a. st('a)=Uninit, own('a)=held
write(p, 7)           // accept: I-6, st ≠ Gone, T is copy
read(p)               // accept: st('a) = Init
free(o)               // accept: I-4
read(p)               // reject: R1(ii), st('a) = Gone (site: free at L4)
```

### I-2 Frame binding creation (scope entry)

**Premises** a `let` with an initializer. **Effect** mint `'f` for the frame
slot; `st('f) := Init`; `own('f) := held`; the slot is *frame-placed*, so `I-11`
(relocating move) never applies to it while a `ptr<'f>` is live.

```text
let mut x = 3         // accept: exists 'f. st('f)=Init, own('f)=held
let px = ptr_of(x)    // accept: px : ptr<'f, u64>
x = 4                 // accept: I-6 through the binding = a write at 'f
read(px)              // accept: st('f)=Init; val('f) killed by L3, reload
```

### I-3 Scope exit

**Premises** for every `'f` minted in the scope: `own('f)` is the **constant**
`discharged`, or `obl('f) = none` and the frame slot is compiler-released.
**Effect** `st('f.π) := Gone` for every `π ≤ 'f`; facts supported under `'f` are
killed. A non-constant `own` at scope exit is **rejected**, never released by an
inserted branch (M2(ii)).

```text
{ let a = alloc(); if c { free(a) } }
                      // reject at `}`: own('a) = ite(c, discharged, held),
                      //   not the constant `discharged`.
                      //   "R2: 'a's obligation is outstanding when c is false
                      //    (site: alloc L1). repair: write `else { free(a) }`
                      //    or `if !c { free(a) }` before the scope ends."
{ let a = alloc(); if c { free(a) } else { free(a) } }
                      // accept: both arms exit discharged; join is constant
```

### I-4 `free` / `dispose` — ending storage

**Premises** `own('a) = held` (constant after `JF-TERM` evaluation under `Γ`);
`st('a) ≠ Gone`; `I-5`'s obligation guard. **Effect** for every `π ≤ 'a`:
`st(π) := Gone`, permanently — **no rule anywhere sets a `Gone` path to any
other state**; `own('a) := discharged`; `I-13` propagates to `owns('a)`; every
fact supported under `'a` is killed.

```text
p : ptr<'A,u64>; q = p
free(oA)              // accept: own held, st('A)=Live, obl none
read(q)               // reject: R1(ii), st('A)=Gone (site: free L2).
                      //   one fact refuses every path: the fact is on 'A
free(oA)              // reject: R2, own('A) = discharged (site: free L2)
```

### I-5 Obligation guard on ending (soundness F5)

**Premises** none. **Effect** `free('a)` additionally requires, for every
`π ≤ 'a` whose content type carries a release obligation, `st(π) = Uninit` (the
value has left) or `obl(π) = none`. `Live` does **not** satisfy this: the
occupancy is unknown, so the obligation is not known discharged.

```text
p : ptr<'A, Box<T>>
free(oA)              // reject: obl('A) = one and st('A) = Init.
                      //   "R2: freeing 'A drops the obligation of the value it
                      //    holds. missing: take(p) or replace(p, ·) first."
v = take(p); drop_box(v); free(oA)
                      // accept: st('A)=Uninit, obl('A)=none
```

### I-6 `write` / store

**Premises** `st(π) ≠ Gone`; if `T` carries an obligation, `st(π) = Uninit`
(otherwise use `I-9 replace`). **Effect** strong or weak per `KEY`:
`st(π) := Init`; `JF-KILL` kills every fact whose support may-overlap `π`.

```text
if c { take(p) }      // st('A) = ite(c, Uninit, Init)
write(p, 20)          // accept: u64 is copy; st ≠ Gone on both leaves;
                      //   st('A) := Init on both leaves -> constant Init
read(p)               // accept  <- CASES B07's accepting variant
pb : ptr<'B, Box<T>>
write(pb, newbox)     // reject: I-6's obligation clause, st('B)=Init.
                      //   repair: `old = replace(pb, newbox)` (I-9)
```

### I-7 `take`

**Premises** `st(π) = Init` after `KEY` resolution and `JF-TERM` evaluation.
**Effect** strong update `st(π) := Uninit` if `mayEq(π) = {π}`, else weak:
`st(π) := Uninit` and every sibling in `mayEq(π)` widens to `Live`;
`obl(π) := none`; the value leaves with its obligations (`R3`).

```text
v = take(p)           // accept: st('A)=Init -> Uninit
read(q)               // reject: R1(i), st('A)=Uninit (site: take L1 through p).
                      //   the hole is on 'A, visible through every name of 'A
take(p)               // reject: R3, 'A holds no value (used twice, not "never released")
```

### I-8 `put`

**Premises** `st(π) = Uninit`; `mayEq(π) = {π}` for a strong fill (a weak `put`
at an unresolved index is **rejected**: filling one of several may-equal paths
cannot be recorded). **Effect** `st(π) := Init`; `obl(π)` from the moved value.

```text
put(q, move v)        // accept: st('A)=Uninit -> Init (a different alias fills it)
read(p)               // accept: st('A)=Init
put(v2, k, move w)    // reject when mayEq('b[k]) ⊋ {'b[k]}:
                      //   "I-8: 'b[k] may be 'b[i] (site: take L1).
                      //    missing fact: k # i"
```

### I-9 `replace`

**Premises** `st(π) = Init`; `mayEq(π) = {π}`. **Effect** `st` unchanged; the old
value leaves with its obligations (`obl(π)` transfers to the result binding);
`val(π)` killed.

```text
old = replace(p, new) // accept: st('A) stays Init, obl moves to `old`
read(q)               // accept: reads `new`. val is keyed on 'A, so the read
                      //   through q sees it: no must-alias analysis (R6)
free(oA)              // reject: I-5, `old` still carries an obligation and 'A is
                      //   Init holding `new`.  <- soundness F5 repaired
drop(old); v = take(p); drop(v); free(oA)     // accept
```

### I-10 Non-relocating move of an owner

**Premises** the source binding holds `own('a)`. **Effect** the binding is
consumed; `own('a)` moves to the destination binding; **`st('a)` is unchanged —
the owner moved, the storage did not** (ladder Step 1).

```text
b = heap_box(); pc = ptr_of(deref(b).x)
b2 = move b           // accept: own('bx) rebinds; st('bx.x) unchanged
read(pc)              // accept: the box moved, its content storage did not
```

### I-11 Relocating move of storage

**Premises** the operation moves bytes (a frame aggregate moved by value, an
enum payload move, a return by value of a non-handle aggregate). **Effect**
`st('a.π) := Gone` for every `π`; mint fresh `'b` with `st('b.π) := st_old('a.π)`
pointwise; `own` transfers; **no rule revives `'a`**.

```text
s : S in 'a; px = ptr_of(s.x)
t = move s            // accept: st('a.π) := Gone; fresh 'b holds the contents
read(px)              // reject: R1(ii), st('a.x)=Gone (site: relocating move L2).
                      //   "repair: take a pointer into 'b after the move,
                      //    or pass the aggregate as a handle (C4)."
```

### I-12 Sink into another storage (move into a field, a slot, a container)

**Premises** `st(dst) = Uninit`; the source is an owner or a taken value.
**Effect** `st(dst) := Init`; `obl(dst)` := the value's; the source binding is
consumed; if the source was itself storage (`own<'s>`), `owns(dst_root)` gains
`'s` and `I-13` applies at the destination's end.

```text
node.payload = move boxed     // accept: st('n.payload)=Uninit -> Init;
                              //   owns('n) := owns('n) ∪ {'bx}
free(node_owner)              // accept: I-4 + I-13 ends 'n AND 'bx
read(p_into_bx)               // reject: R1(ii), st('bx)=Gone
                              //   (site: free L3, through owns edge)
```

### I-13 Owning-edge propagation on ending (soundness S6)

**Premises** `I-4`, `I-3`, `I-11`, `I-22` or `I-25` ends `'a`. **Effect** every
identity in `owns('a)` ends too, transitively; the edge set is exactly:
`backing`, container element backings, sunk owners (`I-12`), and a nominal's
owned identity parameters. **The set is enumerated; no other edge propagates.**

```text
v : Vec in 'v, backing('v)='b, pe : ptr<'b[i], Elem>
free(v_owner)         // accept: owns('v) = {'b}; st('v.π):=Gone, st('b.π):=Gone
read(pe)              // reject: R1(ii), st('b[i])=Gone (site: free L2 via
                      //   owns('v) ∋ 'b).   <- base candidate accepted this
```

### I-14 Downward closure of a contract-asserted state (soundness S3)

**Premises** a callee's `ensures` asserts `st('a) = Gone`. **Effect** `st(π) :=
Gone` for every `π ≤ 'a`. Asserting `Init` or `Live` on an ancestor implies
**nothing** about descendants; to state descendant occupancy the callee writes
`RSt` (`III-12`) or a per-path clause.

```text
fn wipe<'a>(p) ensures st('a) = Gone  writes 'a
wipe(pA)              // accept
read(pA_field)        // reject: R1(ii), st('a.f)=Gone by I-14
fn refill<'a>(p) ensures st('a) = Init
refill(pA)            // accept
read(pA_field)        // reject: I-14 gives nothing for 'a.f.
                      //   "missing fact: st('a.f) = Init.
                      //    repair: refill states `RSt` or a clause per field."
```

### I-15 Container `push`, no reallocation arm

**Premises** the container's contract; `st('v) = Init`; `len('v) = n₀`,
`cap('v) = c₀` (bridge facts, `III-2`/`III-3`); the atom `n₀ < c₀` holds under
`Γ`. **Effect** `st('b[n₀]) := Init`; `len('v) := n₀+1` (fresh frozen value);
`backing('v)` unchanged; `RSt('b[0..n₀+1), Init, {})`.

```text
let n0 = len(v); let c0 = cap(v)
if n0 < c0 { push(v, x) }
                      // accept: Γ={n0<c0}; backing unchanged; 'b survives
read(pe)              // accept: st('b[i]) untouched by the push
```

### I-16 Container `push`, reallocating arm — the two-armed contract (O16)

**Premises** the container's contract; `st('v) = Init`. **Effect** the exit state
is the *term* over the guard atom the contract names:

```text
fn push<'v,'b>(v: ptr<'v,Vec<'b>>, x: T)
  requires st('v)=Init and backing('v)='b and len('v)=n0 and cap('v)=c0
  ensures  len('v) = n0+1
       and backing('v) = ite(n0 < c0, 'b, 'b2)   and exists 'b2.
       and st('b) = ite(n0 < c0, Live, Gone)
       and st('b2) = ite(n0 < c0, Gone, Live)
       and RSt(backing('v)[0..n0+1), Init, {})
       and cap('v) >= n0+1                        // III-3: `cap` IS in the vocabulary
  writes 'v, 'b   ends  ite(n0 < c0, none, 'b)
```

```text
pe : ptr<'b[i], Elem>
push(v, 5)            // accept: contract above
read(pe)              // reject: st('b[i]) = ite(n0<c0, Live, Gone).
                      //   "R1(ii): 'b has ended when n0 >= c0
                      //    (site: push, ensures clause 3). missing fact: n0 < c0"
if n0 < c0 { push(v,5); read(pe) } else { … }
                      // accept: Γ={n0<c0} collapses the term to Live, then
                      //   RSt gives st('b[i])=Init for i < len   (III-12)
```

### I-17 `reserve`

**Premises** as `I-16`. **Effect** the same two-armed form, and — the repair to
critique-cost S1 — **a capacity postcondition**:

```text
fn reserve<'v,'b>(v, m) requires st('v)=Init and backing('v)='b and cap('v)=c0
  ensures exists 'b2. backing('v) = ite(c0 >= m, 'b, 'b2)
      and st('b)  = ite(c0 >= m, Live, Gone)
      and len('v) = len(old 'v)   and cap('v) >= m          // <- new clause
  writes 'v,'b   ends ite(c0 >= m, none, 'b)

reserve(v, n0+1)      // accept. after: cap('v) = c1 with c1 >= n0+1 (III-3)
push(v, 5)            // accept: I-16's atom is `n0 < c1`, and JF-ENT derives it
                      //   from c1 >= n0+1 by difference-bound closure
read(pe)              // reject: the cursor's cached 'b is ite(c0>=n0+1, Live, Gone)
                      //   from the RESERVE, not the push.
                      //   "missing fact: c0 >= n0+1"  <- soundness F3(b) repaired:
                      //   the proof belongs BEFORE the reserve, and the cursor
                      //   must carry 'b (I-19).
```

### I-18 Container `insert` / `remove` at an index (shifting)

**Premises** `i ≤ len`; the contract. **Effect** every element identity at
index `≥ i` is **re-derived**, not preserved: `RSt` is restated over the new
extent and every `val('b[j])` with `j ≥ i` is killed. Interior pointers into
`'b[j], j ≥ i` name storage whose *contents* moved; the identity `'b[j]` is the
slot, so `st` is unchanged and `val` dies. A writer holding a pointer to a
logical element, not a slot, is refused by the same fact.

```text
pe = ptr_of(v[5])
insert(v, 2, x)       // accept: writes 'b[2..len+1); RSt restated; val('b[j>=2]) dies
read(pe)              // accept: st('b[5]) = Init — but the VALUE is v[4]'s old one.
                      //   the checker never claimed otherwise: val('b[5]) was killed.
assert_eq(read(pe), old_v5)
                      // reject: no val fact supports it.
                      //   "R6: val('b[5]) killed by insert at L2 (footprint 'b[2..))"
```

### I-19 Stored pointers: a nominal's identity row is closed (soundness F3, m1)

**Premises** a nominal field's type mentions an identity. **Effect** that
identity is a parameter of the nominal. A free identity in a field type is a
**declaration-site rejection**.

```text
struct Cursor<'v> { at: ptr<'v, Vec<'b>>, i: u64 }
                      // reject at the declaration: 'b is free.
                      //   "I-19: field `at` mentions 'b, which is not a parameter
                      //    of Cursor. repair: `struct Cursor<'v,'b>` with
                      //    `where backing('v) = 'b`."
struct Cursor<'v,'b> { at: ptr<'v, Vec<'b>>, i: u64 } where backing('v) = 'b
                      // accept: 2 identity parameters. §6 charges the cost.
```

### I-20 Arena carve (`alloc(A, n)`)

**Premises** the arena's written invariant `INV_A`:
`head('A) = h`, `h + n ≤ size('A)`, and
`∀ live block b. hi(b) ≤ h`. **Effect** mint `'bk = 'A.bytes@g[h..h+n)` with a
**fresh ghost index `g`** (`III-6`); `st('bk) := Uninit`; `own('bk) := held`;
`'bk ≤ 'A.bytes`; `head('A) := h+n`; `writes 'A.meta` (or `'A.meta ∪` the free-list
extents if the arena declares an intrusive list — `III-4`, soundness S8).
Distinctness of two live blocks is **derived** from `INV_A` by `JF-LIN`
(disjoint half-open extents), never from freshness of names.

```text
b1 = alloc(A, 64)     // accept: 'b1 = 'A.bytes@g1[0..64),  head := 64
b2 = alloc(A, 64)     // accept: 'b2 = 'A.bytes@g2[64..128), head := 128
                      //   'b1 # 'b2 : JF-LIN, [0,64) ∩ [64,128) = ∅ from INV_A.
                      //   NOT from g1 ≠ g2 and NOT from name inequality (R4(b))
fill(b1) || fill(b2)  // accept: footprints {'b1},{'b2}, proved disjoint. R5(i)
distinct('A.bytes,'b1)// not derivable: 'b1 ≤ 'A.bytes, containment blocks # (R4(b))
```

### I-21 Arena free of one block

**Premises** `own('bk) = held`; `I-5`. **Effect** `st(π) := Gone` for every
`π ≤ 'bk` (which is `'A.bytes@g[lo..hi)`, ghost index included), permanently;
`writes 'A.meta`; the bytes return to the arena's free structure. `R14(iii)`: the
allocator may write inside the freed extent afterwards.

```text
free(A, b2)           // accept: st('A.bytes@g2[64..128)) := Gone, permanently
old : ptr<'A.bytes@g2[64..128), u8>
read(old)             // reject: R1(ii), st = Gone (site: free L1)
```

### I-22 Arena reset

**Premises** `own` held for every outstanding block, or the arena's contract
declares reset as an `ends` over `'A.bytes`. **Effect** `st(π) := Gone` for every
`π ≤ 'A.bytes`; `head('A) := 0`; `own` discharged for every block **that the
contract names** — a reset with outstanding blocks whose owners are live is
rejected by `R2`, not silently absorbed.

```text
b1 = alloc(A,64); b2 = alloc(A,64)
reset(A)              // reject: own('b1), own('b2) held.
                      //   "R2: reset ends 'b1,'b2 but their obligations are held.
                      //    repair: free them, or declare `reset` as
                      //    `consumes own('A.bytes[*])`."
free(A,b1); free(A,b2); reset(A)       // accept
```

### I-23 Storage re-creation at an ended extent (arena/pool byte reuse)

**Premises** `I-20` or `I-23`'s pool form carves an extent whose bytes coincide
with an ended block. **Effect** the minted identity carries a **fresh ghost
index**, so it is a *different path* from the ended one. `Gone` is never lifted;
the new identity starts `Uninit`. No runtime tag exists (`III-6`).

```text
free(A, b2)           // st('A.bytes@g2[64..128)) := Gone
b4 = alloc(A, 32)     // accept: 'b4 = 'A.bytes@g4[64..96). fresh g4.
                      //   'b4 and 'b2 are related by NO rule: different ghost
                      //   index -> not may-equal -> KEY never joins them
read(old_into_b2)     // reject: R1(ii), st('b2)=Gone. address reuse never revives
write(b4, 1)          // accept: st('b4)=Uninit -> Init
                      // M2(ii): nothing is compared at run time; g is erased.
                      //   DEPENDS ON OWNER DECISION O3 (§6, declared).
```

### I-24 Pool `insert` / `remove` (soundness F2)

**Premises** `remove(m)`: `st('P[m]@g) = Init`, `own` held, `I-5`.
**Effect** `st(π) := Gone` for every `π ≤ 'P[m]@g`, permanently; every fact under
it killed; the slot returns to the pool's **written** free list (`III-5`,
`O17(a)` — the free list is the pool library's own data, one enumerated M3 entry
for the byte-extent primitive). `insert` mints `'P[s]@g'` with a fresh ghost
index by `I-23`.

**The F2 repair is `KEY`, not this rule**: a cursor holding `'P[s]@g_s` with `s`
symbolic is may-equal to `'P[m]@g_m` **only if the ghost indices are the same
frozen value**; an `unpack` (`II-16`) of a stored link binds *both* `s` and
`g_s`, so the removal widens the cursor's state to `⊤` unless `g_s # g_m`.

```text
p = unpack(a.next)    // accept: II-16 binds fresh frozen s, g_s;
                      //   st('P[s]@g_s) = Init from the pool invariant (JF-QI)
remove(m)             // accept: st('P[m]@g_m) := Gone
read(p.data)          // reject: mayEq('P[s]@g_s) ∋ 'P[m]@g_m (neither s # m nor
                      //   g_s # g_m is proved); KEY gives Init ⊓ Gone = ⊤.
                      //   "R1(ii): 'P[s] may be the slot removed at L2.
                      //    missing fact: s # m"     <- base candidate ACCEPTED this
if s != m { read(p.data) }
                      // accept: Γ={s ≠ m} resolves mayEq; st = Init
```

---

## 4. Rule set II — splitting and rejoining state

16 rules.

### II-1 Arm entry

**Premises** entering the `then`/`else`/match arm of construct `C` with atom `α`
(or `¬α`, or the arm's discriminant atom). **Effect** push `α` onto `Γ`; **every
term in scope is immediately evaluated under the new `Γ`** by `JF-TERM`. This is
what makes GCR a shrinking rule.

```text
// st('b) = ite(c, Init, Gone), own('a) = ite(c, discharged, held)
if c {
  read(pb)            // accept: Γ={c}, st('b) collapses to Init
  free(oa)            // reject: Γ={c} collapses own('a) to `discharged`
                      //   "R2: 'a released at L0's then-arm"
} else {
  free(oa)            // accept: Γ={¬c} gives own('a) = held
}
```

### II-2 Arm exit

**Premises** leaving an arm. **Effect** each identity's exit state is recorded as
evaluated under the arm's `Γ`; it must be a **constant** or a term over an atom
already in the *enclosing* `Γ`. A term over the arm's own inner atom is rejected
here (this is where `TERM` bites), naming the inner construct.

```text
if c {
  if d { take(p) } else { }
                      // inner join: st('A) = ite(d, Uninit, Init)
}                     // reject at the arm exit: st('A) carries atom `d`, which is
                      //   not in the enclosing Γ.
                      //   "TERM/II-2: close 'A's divergence inside the `if d`
                      //    (restore with put), or hoist the take."
```

### II-3 `if`/`else` join — the one join rule (reused by R2, R3, origins)

**Premises** both arms reach the join. **Effect** per identity, per fact head:

| then | else | join |
|---|---|---|
| `s` | `s` | `s` |
| `s₁ ≠ s₂`, atom `α` available | — | `ite(α, s₁, s₂)` |
| `Init`,`Uninit` with the atom *unavailable* (a loop exit join, `II-9`) | | `Live` |
| `Live`/`Init`/`Uninit` vs `Gone`, atom unavailable | | `⊤` |
| `held`,`discharged` | | `ite(α, ·, ·)` if `α` available, else **reject** (`R2` definiteness) |
| origins `π₁`,`π₂` | | `ite(α, π₁, π₂)` if available, else `{π₁,π₂}` |

The atom is available iff it is `C`'s own guard atom (`ATOM`-frozen).

```text
if c { free(a) } else { free(b) }
                      // accept: st('a)=ite(c,Gone,Live),  own('a)=ite(c,discharged,held)
                      //         st('b)=ite(c,Live,Gone),  own('b)=ite(c,held,discharged)
                      //   NO drop flag, NO bit: the term is erased checker state
use(a); free(a)       // reject: R2, own('a) is discharged when c holds (site:
                      //   then-arm free L1). "missing: a branch on c"
if c { use(b); free(b) } else { use(a); free(a) }
                      // accept: II-1 collapses both terms inside each arm;
                      //   at this join own('a), own('b) are both the constant
                      //   `discharged`. The selecting branch is the WRITER's. M2(ii)
```

### II-4 `match` join

**Premises** an `n`-arm match on a discriminant `d` (frozen by `III-9`).
**Effect** at most **one** arm may leave an identity in a state differing from
the others; the join is `ite(d = K_i, s_i, s_common)`. Two differing arms exceed
`TERM`'s one-atom bound and are **rejected**, naming both arms.

```text
match tag {
  A => { take(p) }
  B => { }
  C => { }
}                     // accept: st('A') = ite(tag = A, Uninit, Init)
match tag {
  A => { take(p) }
  B => { free(oa) }
  C => { }
}                     // reject: "II-4/TERM: arms A and B leave 'A in different
                      //   states; the join needs two atoms. repair: nest the
                      //   match, or make the arms agree."
```

### II-5 Correlated leafwise evaluation

**Premises** several facts at a point carry the **same** atom. **Effect** every
query evaluates all of them at the same leaf. Correlation between an origin term
and a state term is therefore preserved exactly — this is what makes `CASES.md`
B02/B03/B05 derivable.

```text
if cond { p = ref(a); old = take(b) } else { p = ref(b); old = take(a) }
                      // origin(p)=ite(cond,'A,'B); st('A)=ite(cond,Init,Uninit);
                      //                            st('B)=ite(cond,Uninit,Init)
read(p)               // accept: leaf cond -> origin 'A, st('A)=Init  ✓
                      //         leaf ¬cond -> origin 'B, st('B)=Init ✓
read(a)               // reject: st('A) is not the constant Init.
                      //   "missing fact: cond"
```

### II-6 Loop head — what must be re-established, what is written

**Premises** a loop head `H` with entry edge `E` and back edge `B`. **Effect**
for each identity the head state is the `II-3` join of `E` and `B` **with no atom
available** unless a written `invariant` names one over a **head-frozen φ-value**.
The head state must be:

- a **constant**, or
- a term `ite(α, s₁, s₂)` where `α` is an atom over a head φ-value **and** the
  written `invariant` states it, or
- rejected, naming the identity, the back edge, and the two repairs
  ("restore before the back edge" / "write `invariant st(π) = …`").

Origins and `RSt` residual lists are subject to the same rule. Every residual
exception list (`JF-QI`) is **discarded at the head** unless re-established by
the written invariant: this is what bounds `r`.

```text
repeat n { v = take(p); put(p, move v) }
                      // accept: entry Init, back edge Init -> head constant Init.
                      //   nothing written.                      <- CASES L01
read(p)               // accept, including n = 0
repeat n { v = take(p) }
                      // reject at the head: entry Init, back edge Uninit, no atom.
                      //   join = Live, not a constant.
                      //   "R1: 'A's state at the loop head is not constant.
                      //    repair: restore before the back edge, or write
                      //    `invariant st('A) = Live` (then the take rejects)."
```

### II-7 Loop head with a written invariant over origins

**Premises** a written `invariant` mentioning `origin(x)`, `st(origin(x))`,
`origin(x) # origin(y)`. **Effect** the head admits a *relational* state whose
per-name physical role alternates.

```text
repeat n {
  invariant origin(p) # origin(q)
        and st(origin(p)) = Init and st(origin(q)) = Uninit
  v = take(p); put(q, move v); tmp = p; p = q; q = tmp
}                     // accept: the body maps the relation to itself;
                      //   entry establishes it (take(b) before the loop)
read(p)               // accept: st(origin(p)) = Init from the head relation
read(a)               // reject: origin(p) is not resolved to 'A.
                      //   "missing fact: origin(p) = 'A"     <- CASES L03
```

### II-8 Back edge check

**Premises** reaching the back edge. **Effect** every head fact must be entailed
by the state there (`JF-TERM` equality plus `JF-ENT` for the invariant's atoms).
A failure names the head fact and the statement in the body that broke it.

```text
repeat n { read(p); if stop { release(p); active = false } }
                      // reject at the back edge if the head says st('A)=Init:
                      //   "II-8: `invariant st('A) = Init` fails on the back edge;
                      //    broken by release at L1. repair: guard the body on a
                      //    source Boolean and state the head relation over it."
repeat n { invariant (active ==> st('A)=Init and own('A)=held)
                 and (!active ==> st('A)=Gone and own('A)=discharged)
           if active { read(p); if stop { release(p); active = false } } }
if active { release(p) }
                      // accept: exactly one release, including n = 0   <- CASES L05
```

### II-9 Break edge and exit join

**Premises** a `break` leaves the loop. **Effect** the break edge carries its
**actual** state to the exit join; the head invariant is *not* required on it.
The exit join is `II-3` over {zero-iteration edge, normal-completion edge, every
break edge} with **no atom available** — so `Init ⊔ Uninit = Live` and
`Live ⊔ Gone = ⊤`.

```text
repeat n { v = take(p); if stop { break }; put(p, move v) }
release(p)            // accept: exit join = Init ⊔ Init ⊔ Uninit = Live.
                      //   I-4 needs st ≠ Gone ✓; I-5 needs obl none (v is copy) ✓
read(p)               // reject if written before the release: st = Live, not Init.
                      //   "missing fact: st('A) = Init on the break exit"  <- CASES L06

repeat n { if stop { release(p); break }; read(p) }
read(p)               // reject: exit join = Live ⊔ Gone = ⊤ (widened by the break
                      //   edge at L1). "R1: no state is known for 'A."
                      // and at scope exit: own('A) = ⊤ -> I-3 rejects.  <- CASES L04
```

### II-10 Call rule (caller side)

**Premises** the callee's stated interface; the caller discharges every
`requires`, every `#` obligation for every pair of argument identities
**including sub-identities** (C3 — never an axiom), and binds every `old(e)` to a
frozen value. **Effect** the caller learns exactly the `ensures`; `JF-KILL` kills
exactly the facts whose support may-overlap the `writes` footprint; the `ends`
footprint applies `I-4`+`I-13`+`I-14`; `exists` results are unpacked by `II-16`;
`consumes`/`yields` moves obligations (`II-12`). **The body is never opened.**

```text
fn drain<'x>(p) requires st('x)=Init ensures st('x)=Uninit writes 'x consumes obl('x)
drain(q)              // accept: st('Q)=Init satisfies requires
read(q)               // reject: R1(i). "'Q holds no value; set by drain's
                      //   `ensures st('x)=Uninit` at the call on line 2"
put(q, move w)        // accept
free(oQ)              // accept: I-5 satisfied because drain wrote `consumes obl('x)`
```

### II-11 Callee-side check

**Premises** the declaration. **Effect** the body is checked once: every exit
satisfies the `ensures` under `JF-TERM` equality; the body's realized footprint
is a subset of the stated `reads`/`writes`/`ends`; a `requires` that is
syntactically unsatisfiable, or a result naming an origin no caller can name, is
rejected at the declaration (R7(a)).

```text
fn reserve<'v,'b>(v,m) ensures … and cap('v) >= m …
  { if cap(v) < m { let nb = alloc(2*m); copy; free(old); backing := nb } }
                      // accept: the then-path exhibits `ends 'b`, the else-path
                      //   exhibits `ends none`; both satisfy cap >= m
fn bad<'x>(p) ensures st('x)=Init { }
                      // reject at the declaration when entry allows Uninit:
                      //   "R7: the body does not establish `ensures st('x)=Init`
                      //    on the path from st('x)=Uninit."
```

### II-12 Obligation transfer at a call (soundness S10)

**Premises** none. **Effect** the interface vocabulary contains
`consumes obl(π)` (the callee discharged what it took) and `yields obl(π)` (the
callee installed one). A call with neither leaves `obl(π)` **unchanged**, so a
callee that takes a value and drops it is a declaration-side rejection.

```text
fn drain<'x>(p) ensures st('x)=Uninit writes 'x
                      // reject at the declaration: the body takes an
                      //   obligation-carrying value and neither returns it nor
                      //   declares `consumes obl('x)`.
                      //   "R2: drain's interface does not say what became of
                      //    'x's content obligation."
fn drain<'x>(p) ensures st('x)=Uninit writes 'x consumes obl('x)     // accept
fn steal<'x>(p) -> T  ensures st('x)=Uninit writes 'x yields obl(result)  // accept
```

### II-13 Projection / window: open, use, close

**Premises** `open`: a base path `π` and a proved extent or field selector.
**Effect** a projection is a **sub-identity name**, not a token: `open` records
`π' ≤ π` and nothing is consumed; `use` is an ordinary access at `π'`; `close` is
the end of the syntactic block and records nothing. Because nothing is consumed,
**a projection may be stored** (`I-19` closes the nominal's row) and two
projections of one base may be live at once — the base's writability is not
suspended.

```text
sw  = subrange(out, lo, hi)     // accept: slice<'o[lo..hi), u8>, 'o[lo..hi) ≤ 'o
sw2 = subrange(out, hi, n)      // accept: two live projections of one base
write(out, 0, 7)                // accept: D4, no suspension. kills val('o[0])
                                //   and val facts may-overlapping 'o[0] only
par { helper(sw) || helper(sw2) }
                                // accept: footprints {'o[lo..hi)}, {'o[hi..n)}
                                //   disjoint by JF-LIN. R5 ground (i)
```

### II-14 Origin split

**Premises** a pointer binding whose value differs across a construct's arms.
**Effect** `origin(v) := ite(α, π₁, π₂)` (atom available) or `{π₁,π₂}` (not).
Copying the binding copies the **origin term**, so `CASES.md` B06's `saved`
keeps the old term after `p` is rebound.

```text
if cond { p = ref(a); q = ref(b) } else { p = ref(b); q = ref(a) }
saved = p             // accept: origin(saved) = ite(cond,'A,'B) — a copy of the term
p = q                 // accept: origin(p) := ite(cond,'B,'A)
release(p)            // accept: leafwise, own('B) or own('A) discharged
read(saved)           // accept: leafwise the other identity, still Init and held
read(q)               // reject: origin(q) = origin(p); leafwise Gone.  <- CASES B06
```

### II-15 Access through a non-singleton origin (soundness F4)

**Premises** `origin(v)` is a term or a set. **Effect**:

| origin form | read | write / take / put / free |
|---|---|---|
| `ite(α,π₁,π₂)` | per `II-5`, leafwise | **strong, leafwise** — the atom decides which path changed |
| `{π₁…πₙ}` (no discriminant) | requires `st(πᵢ)=Init` for every `i` | **`st` unchanged**; `JF-KILL` kills `val` on every member; permitted only when every `st(πᵢ)` is already `Init` |

A write through an origin set therefore **never sets `Init`** on a member that
was `Uninit`. The named repair is to give the producing callee a discriminating
atom.

```text
r = pick(c, x, y)     // pick: -> ptr<'x|'y, T>, no discriminant
read(r)               // accept iff st('X)=Init AND st('Y)=Init
write(r, 5)           // reject when st('X)=Uninit: II-15 row 2.
                      //   "R1(i): writing through an origin set cannot initialize
                      //    'X, because the write may have landed on 'Y.
                      //    repair: declare pick as
                      //      ensures origin(result) = ite(c, 'x, 'y)"
                      // base candidate: ACCEPTED, marking BOTH Init.  repaired.
fn pick<'x,'y>(c,x,y) -> ptr<T> ensures origin(result) = ite(c, 'x, 'y)
write(r, 5)           // accept: II-15 row 1. strong leafwise:
                      //   st('X) := ite(c, Init, st('X)); st('Y) := ite(c, st('Y), Init)
```

### II-16 `unpack` / `pack` of an existential identity (soundness S7)

**Premises** a value of type `exists s, g. ptr<'P[s]@g, T>` (a stored link, an
`exists 'b2` result). **Effect** `unpack` binds **fresh frozen values** `s`, `g`
and gives `ptr<'P[s]@g, T>`; it is the *only* term former that reads an identity
out of a value, and it is erased. `pack` is its inverse and requires the
witness's facts to imply the packed type's. Two unpacks are **not** distinct
without a written fact — that is D4's declared cost, not a defect.

```text
p  = unpack(a.next)   // accept: fresh s1,g1; st('P[s1]@g1)=Init from the pool's
                      //   ∀-invariant by JF-QI
p2 = unpack(b.next)   // accept: fresh s2,g2
p  # p2               // not derivable: "II-16: two unpacked links are not distinct.
                      //   repair: `use INV_pool.distinct_links(a,b)` if the pool
                      //   states it, or branch on s1 != s2."
write(p.data, 1); write(p2.data, 2)
                      // accept sequentially (D4); par { … } rejects on R5(i)
```

---

## 5. Rule set III — the identity-to-runtime-value bridge

This is the meta-finding's third place (`VERDICT-CORE.md` §6, O19): the bridge
between a static name and the runtime value that determines it. 12 rules. Every
one has the same shape, stated once:

### III-1 The bridge schema (`FREEZE`)

**Premises** a *measure head* `M` (one of `len, cap, head, gen, slot, tag, size`)
applied to an identity path, evaluated at a point.
**Effect** the checker binds a **fresh frozen value** `x` and records the
*bridge fact* `M(π) = x`. The bridge fact has support `π` and dies under
`JF-KILL` like any other fact. **The frozen value `x` never dies** (`ATOM`).
Re-derivation is one re-application of `FREEZE`, which the writer spells as a
`let` or which the checker performs at a guard.

| | bridge fact | frozen value |
|---|---|---|
| killed by a write to its support | yes | **no** |
| may appear in an atom | no | yes |
| erased at lowering | yes | yes |
| re-derived by | `FREEZE` (one `let`) | never needed |

```text
let n0 = len(v)       // accept: FREEZE. bridge len('v) = n0; frozen n0
if n0 < 4 { … }       // accept: atom `n0 < 4` over a frozen operand (ATOM)
push(v, x)            // accept: writes 'v -> JF-KILL kills `len('v) = n0`.
                      //   the ATOM `n0 < 4` survives untouched
read(v[0])            // reject: "R11: index requires 0 < len(v);
                      //   len('v) unbound since push at L3. repair: `let n1 = len(v)`"
let n1 = len(v); read(v[0])      // accept: JF-ENT from `n1 = n0+1` (I-16's ensures)
```

### III-2 `len` — the element extent

**Established by** `FREEZE` at a `len(v)` expression, or by a contract clause.
**Killed by** any write whose footprint may-overlap `'v` (the container header),
**not** by a write to `backing('v)[i]` — element writes do not disturb `len`.
This is R6's measured tax, removed at the rule.
**Re-derived by** `FREEZE`, or from a contract's arithmetic (`len = n0+1`).

```text
let n0 = len(v)
write(v[3], 9)        // accept: footprint {'b[3]}; JF-KILL does NOT kill len('v)=n0
                      //   because 'b[3] does not may-overlap 'v.    <- R6(e)
read(v[n0-1])         // accept: n0 still bound; JF-ENT gives n0-1 < n0
push(v, 1)            // accept: footprint {'v,'b}; len('v)=n0 DIES
read(v[n0-1])         // reject: "R11: i < len(v) unproved; len('v) unbound"
```

### III-3 `cap` — the reallocation discriminant

**Established by** `FREEZE`, or by `I-17`'s `ensures cap('v) >= m`.
**Killed by** a write to `'v`. **Re-derived by** `FREEZE` or by a contract.
`cap` is **in** the closed vocabulary (soundness S5, cost S1).

```text
let c0 = cap(v); let n0 = len(v)
reserve(v, n0+1)      // accept: ensures cap('v) >= n0+1; FREEZE binds c1 with
                      //   the atom `c1 >= n0+1` recorded from the ensures
push(v, 5)            // accept: I-16's guard atom is `n0 < c1`; JF-ENT derives it
                      //   from c1 >= n0+1 by one difference-bound edge
read(pe)              // accept: st('b) = ite(n0 < c1, Live, Gone) collapses to Live,
                      //   and RSt gives Init for i < len.     <- cost S1 repaired
```

### III-4 `head` — the bump offset

**Established by** the arena's written invariant `INV_A`. **Killed by** `I-20`,
`I-21`, `I-22` (each writes `'A.meta`). **Re-derived by** `FREEZE` on `head(A)`.
The extent of a carved block is `[h, h+n)` with `h` the *frozen* head at the
carve, so two blocks' disjointness is `JF-LIN` over frozen bounds.

```text
let h0 = head(A)      // accept: bridge head('A) = h0
b1 = alloc(A, 64)     // accept: 'b1 = 'A.bytes@g1[h0..h0+64); head('A)=h0 dies;
                      //   I-20 rebinds head('A) = h1 with atom h1 = h0+64
b2 = alloc(A, 64)     // accept: 'b2 = 'A.bytes@g2[h1..h1+64)
b1 # b2               // accept: JF-LIN, [h0,h0+64) ∩ [h1,h1+64) = ∅ from h1 = h0+64
// intrusive free list: the arena declares `writes 'A.bytes[freelist]` and every
// block's val facts die at alloc/free. Declared, not assumed.   <- soundness S8
```

### III-5 `slot` — a pool index

**Established by** `II-16`'s unpack, or by `FREEZE` on a handle's index field.
**Killed by** nothing (the frozen value is immortal); the *pool invariant*
relating a slot to occupancy is killed by writes to the pool's free list and
re-established by `JF-QI`'s residual framing.
**The free list is the pool library's own written data** (O17(a)), not compiler
bookkeeping — this is the answer to soundness S2's "P3 hides the occupancy word
it charges P11 for": the word is charged here, once, in the pool's type.

```text
let s = h.idx         // accept: FREEZE, frozen s
use INV_pool.slot_live(s)
                      // accept: a NAMED rule application with checked premises
                      //   (O15(b)); premise `h` came from `insert` and was not
                      //   passed to `remove`. gives st('P[s]@g) = Init
read(pool[s].data)    // accept
remove(pool, h2)      // accept: I-24
read(pool[s].data)    // reject: KEY, mayEq widened by the removal.
                      //   "missing fact: s # idx(h2)"
```

### III-6 `gen` — the ghost index on a reusable extent

**Established by** `I-20`/`I-23`/`I-24` minting a fresh ghost index.
**Killed by** nothing. **Re-derived by** nothing — it is never re-derived,
because it is never compared: it exists only to keep two physically-coincident
extents from being one path. It is **erased**, never materialized, never
compared at run time (`M2(ii)` — *conditional on owner decision O3*, §6).

```text
free(A, b2)           // st('A.bytes@g2[64..128)) := Gone
b4 = alloc(A, 32)     // 'b4 = 'A.bytes@g4[64..96), g4 fresh
b4 # b2               // NOT derivable, and not needed: 'b2 is Gone and
                      //   not-may-equal to 'b4 because g4 ≠ g2 is a FRESHNESS
                      //   property of minting, used only to separate PATHS,
                      //   never emitted to the backend as a `#` fact (R4(b))
read(old_b2)          // reject: st = Gone
read(b4)              // reject: st('b4) = Uninit (not yet written)
write(b4, 1); read(b4)          // accept
```

### III-7 A branch condition

**Established by** the guard expression at an `if`/`match`, frozen by `FREEZE`
(`ATOM`). **Killed by** nothing. **Re-derived by** nothing.
A reassignment of the *variable* creates a new frozen value; the relation
between the two (for `b := !b`, `b := e` with `e` frozen) is recorded as an
ordinary atom in the boolean part of `JF-ENT`.

```text
if cond { old = take(p) }       // atom is over the frozen value c0 of `cond`
cond = !cond                    // FREEZE binds c1; atom `c1 = ¬c0` recorded
if cond { write(p, 20) }        // guard atom is `c1`
read(p)                         // reject: st('A) = ite(c0, Uninit, Init) and the
                                //   join over c1 gives ite(c1, Init, Uninit).
                                //   not constant.                  <- CASES B09
if !cond { put(p, 20) }         // (replacing L3) accept: Γ={¬c1} ⊢ c0 by the
                                //   recorded negation; st collapses to Uninit;
                                //   put fills it; join is the constant Init
read(p)                         // accept                            <- CASES B09 variant
```

### III-8 `backing` — a container's element storage

**Established by** a contract (`I-16`, `I-17`) or at construction.
**Killed by** any write to `'v`. **Re-derived by** the contract's own
`backing('v) = ite(α, 'b, 'b2)` term, which is an **origin term** and obeys
`TERM`'s one-atom bound and `II-15`'s access rules.
`owns('v) ∋ backing('v)` (`I-13`).

```text
push(v, 5)            // backing('v) = ite(n0 < c0, 'b, 'b2)
read(v[0])            // accept: leafwise. leaf 1 -> 'b[0]; leaf 2 -> 'b2[0];
                      //   RSt on backing('v)[0..len) gives Init on both leaves
read(pe)              // reject: pe : ptr<'b[i]>, and st('b) = ite(n0<c0, Live, Gone)
                      //   "missing fact: n0 < c0"
free(v_owner)         // accept: I-13 ends BOTH leaves of backing('v)   <- S6
```

### III-9 `tag` — the variant discriminant and layout currency (R1(iii))

**Established by** `FREEZE` on the discriminant. **Killed by** any write to the
enum's storage. **Re-derived by** a `match` (which is `II-4`).
`layout('a) = V_i` licenses access to `'a.payload_i`; a write that changes the
variant sets `st('a.payload_j) := Gone` for the old variant's payload paths
(`I-11`, relocating: the payload's storage ends).

```text
match e { Some(x) => { px = ptr_of(x) ; … } None => { } }
                      // in the Some arm: layout('e) = Some; px : ptr<'e.some.0>
e = None              // accept: I-6 on 'e; layout('e) := None;
                      //   st('e.some.0) := Gone by I-11 (the payload relocated out)
read(px)              // reject: R1(iii)/R1(ii): "'e.some.0 ended when 'e's variant
                      //   changed at L3. missing: layout('e) = Some"
```

### III-10 A handle→slot map (a compacting pool) — soundness S4

**Established by** the pool's **written data**: a `map: array<HandleId, Slot>`
that `compact` rewrites. **Killed by** `compact`. **Re-derived by** the pool's
written `∀`-invariant, re-established wholesale by `compact`'s `ensures`.
The map costs one word per outstanding handle, and it is charged here.

```text
fn compact<'P>(pool) ensures (forall s. st('P[s]@g_old) = Gone)
                         and exists 'P2. backing(pool) = 'P2
                         and (forall hid. live(hid) ==> slot(map,hid) < len('P2)
                                      and st('P2[slot(map,hid)]@g2) = Init)
                     writes 'P, 'P.map   ends 'P
pool.compact()        // accept
read(p)               // reject: R1(ii), st('P[h]@g_old)=Gone (first ensures, I-14)
read(pool[h])         // accept: `pool[h]` is `'P2[slot(map,h)]@g2`, and the third
                      //   ensures gives both the bound and the state by JF-QI.
                      //   the HANDLE survives because the MAP was rewritten,
                      //   not because an index is stable.
```

### III-11 The origin discriminant

**Established by** a callee's `ensures origin(result) = ite(α, 'x, 'y)`, where
`α` is over the callee's own frozen parameters. **Killed by** nothing (it is an
origin term, not a measure). **Re-derived by** nothing.
Absent a discriminant the result is an origin *set* and `II-15` row 2 applies.

```text
fn pick<'x,'y>(c: bool, x, y) -> ptr<T>  ensures origin(result) = ite(c, 'x, 'y)
r = pick(c, px, py)   // accept: origin(r) = ite(c, 'X, 'Y) — c is the CALLER's frozen
read(r)               // accept iff st('X)=Init on leaf c and st('Y)=Init on leaf ¬c
write(r, 5)           // accept: strong, leafwise (II-15 row 1)
                      //   writes footprint is still {'X,'Y} for R5: a footprint is
                      //   a may-write set; only the STATE effect is leafwise
```

### III-12 `RSt` — range state, and its residual exception list (soundness F7)

**Established by** a container's representation invariant or a contract.
**Form** `RSt(π[lo..hi), s, X)` with `X` a finite list of *exception* index atoms.
**Killed by** an `ends` on `π`. **Weakened, not killed, by** a point write at a
symbolic index `i`: `X := X ∪ {i}` and an obligation `st(π[i]) = s` is recorded —
`JF-QI`'s residual framing. **Re-derived by** discharging every exception.
`|X|` is reset at every loop head (`II-6`) and call boundary.

```text
// Vec's invariant: RSt('b[0..len), Init, {})
qi = take(v, i)       // accept: RSt := RSt('b[0..n), Init, {i})
if i != j { qj = take(v, j) }
                      // accept: RSt := RSt('b[0..n), Init, {i, j})
put(v, i, move qi)    // accept: exception i discharged. X := {j}
put(v, j, move qj)    // accept: X := {}
push(v, x)            // accept: I-16 requires RSt('b[0..len), Init, {}) — it holds.
                      //   written before either put: reject, naming 'b[i] and the
                      //   take site.                             <- P1's requirement
```

---

## 6. The judgment families (mandatory change 2)

Measures: `S` statements in the declaration, `I` live identities, `d` max path
depth, `n` frozen scalars in the guard context, `F` live facts, `r` residual
exceptions per `RSt`, `q` written proof steps, `|Ρ|` identity-row length.

The base's **"exactly four automatic families and no fifth" is withdrawn.**
There are **nine**, each specified below.

| # | Family | Inputs | Procedure | Terminates because | Degree |
|---|---|---|---|---|---|
| **JF-PATH** | path normalization and equality | two paths | normalize (constant-fold literal indices, canonicalize `[lo..hi)`), compare step by step | paths are finite trees of bounded depth `d` | `O(d)` per pair |
| **JF-OVL** | containment, may-overlap, and `KEY`'s may-equal class | a path, the live path set | for each live path with the same base: `JF-PATH` on the prefix, then `JF-ENT`/`JF-LIN` on the index/extent arguments; ghost indices compared by `JF-PATH` only | the live set is finite; no recursion | `O(I·d·n³)` per symbolic access; `O(d)` when every index is literal |
| **JF-ENT** | atom entailment | `Γ` (guard atoms + `requires` + `INV` conclusions), a goal atom `α` | (1) normalize every atom to `Σcᵢxᵢ + c ⋈ 0` with literal `cᵢ`; (2) split into a **difference-bound** part (`x − y ≤ c`) and a residue; (3) build the constraint digraph over the `n` frozen scalars and run Floyd–Warshall (a negative cycle is a contradiction, which discharges everything); (4) booleans by unit propagation over the recorded relations (`b₁ = ¬b₂` etc.); (5) the residue is matched **syntactically** against `Γ` after normalization — no case split, no search, no instantiation | Floyd–Warshall is `n` fixed rounds over `n²` edges; unit propagation is monotone over a finite literal set; step (5) is a single pass | `O(n³)` per guard context, computed once per context: `O(S·n³)` per declaration |
| **JF-LIN** | symbolic-coefficient interval disjointness and containment | two half-open extents with affine bounds over frozen scalars, `Γ` | match the goal against a **closed table of 9 schemas**: (1) literal bounds; (2) `a+c₁ ≤ a+c₂`; (3) monotone product `x ≤ y ∧ k ≥ 0 ⇒ kx ≤ ky`; (4) strided partition `k·w + k ≤ k·w′` from `w < w′ ∧ k ≥ 0`; (5) floor-division `w < k ∧ s = ⌊n/k⌋ ⇒ (w+1)·s ≤ n`; (6) prefix/suffix split; (7) sibling fields; (8) containment under a base; (9) ghost-index inequality. Each schema's side conditions go to `JF-ENT`. **One match attempt per schema, in a fixed order; no backtracking.** Outside the table the answer is "not proved", and the writer's route is a branch or a `use` of a written `INV` | 9 schemas, each applied at most once, each side condition a `JF-ENT` query | `O(9·n³) = O(n³)` per pair |
| **JF-QI** | quantifier instantiation and residual framing | a `∀`-fact or `RSt` with exception list `X`, an access path | instantiate **only** at the access path's own index (a written trigger; no pattern search, no nested instantiation); then check the index against every exception in `X` by `JF-ENT`. Framing a write at symbolic `i`: append `i` to `X` and record the point obligation. `X` is discarded at loop heads and call boundaries (`II-6`, `II-10`) | `X` is finite and only grows between two resets, each reset bounded by `S`; one instantiation per query, never recursive | `O(r·n³)` per query, `r ≤ S`; per declaration `O(S²·n³)` worst case |
| **JF-KILL** | the kill scan | a footprint (a set of paths), the fact table | facts are indexed by their support's **base identity**; for each base in the footprint, scan that bucket and test `JF-OVL`; a `∀`/`RSt` fact is **not** killed but weakened by `JF-QI` | the bucket is finite; one pass | `O(F_base·d)` per write; `O(S·F·d)` per declaration |
| **JF-TERM** | state/own/origin term evaluation, join, equality | a term, `Γ` | evaluate: one `JF-ENT` query on the atom; collapse or keep. join: the `II-3` table, one lookup. equality: atoms compared by `JF-PATH`-style normalization, leaves by identity. Terms are depth-≤1 by `TERM`, so nothing recurses | constant-size terms; the table is total | `O(n³)` per query (dominated by `JF-ENT`); `O(1)` for join and equality |
| **JF-ORG** | origin resolution | an origin term or set, an access | term: `JF-TERM` leafwise, then the access rule per leaf. set: the `II-15` row-2 premise over every member. Footprints always take the whole member set | finite member list | `O(\|O\|·n³)` per access |
| **JF-SUB** | identity substitution at instantiation | a generic declaration, an identity row `Ρ`, actuals | positional substitution through the row, pointwise lifting of `ite`/`\|` through a nominal's row, plus `HIS-REQ-07`'s unchanged cycle check | substitution is structural on a finite row; the cycle check is the existing terminating one | `O(\|Ρ\|·d)` per instantiation |

### 6.1 Degrees, rolled up, and the M10 declaration

Acceptance work for one declaration:

```text
W  =  O( S · n³ )                    JF-ENT, one closure per guard context
   +  O( S · I · d · n³ )            JF-OVL at every symbolic access
   +  O( S² · n³ )                   JF-QI residuals
   +  O( S · F · d )                 JF-KILL
```

- **M1 — met.** Every family is a total function of the source bytes: fixed
  procedure, fixed order, no budget, no cap, no search, no solver state. The
  Floyd–Warshall closure terminates by its fixed round count, not by a lattice
  height argument, so no iteration cap exists to tune.
- **M10(a), first clause — met.** No family exceeds cubic **in its own measure**:
  `JF-ENT` and `JF-LIN` are cubic in `n`, `JF-OVL` is linear in `I` and `d` and
  cubic in `n`, `JF-KILL` is linear in `F` and `d`, `JF-QI` is linear in `r`,
  `JF-TERM`/`JF-ORG`/`JF-SUB` are constant or linear.
- **M10(a), second clause — VIOLATED, declared, with the degree.** "Total work
  per declaration at most quadratic in **written proof steps**" does not hold:
  `W` is `Θ(S·I·d·n³ + S²·n³)` and `q` may be **zero**. A declaration with no
  written step can carry the full cost. The dominant driver is the *program*,
  not the *proof*. Recorded as an amendment, not argued away.
- **M10, edit stability — met.** `FN-1` is the reach boundary: a body edit
  changes the verdict of one declaration; an interface edit reaches its callers
  and no further. Frozen values are SSA-local, so a one-token edit inside an arm
  cannot change a term outside it (`TERM`, `II-2`).
- **The base's 2^d state-term amendment — withdrawn.** `TERM` bounds a term at
  one atom, so the family is `O(1)` in size and the exponential does not exist.
  What replaces it is `II-2`'s refusal, priced in §7.
- **M2(ii) — met, *conditional on owner decision O3*.** Nothing is compared at
  run time; the ghost index of `III-6` is minted and erased. If the owner rules
  that a ghost generation is the mechanism `M2(ii)` refuses by name (O3(b)),
  then `I-23`, `I-24` and `III-6` lose their separation of reused extents and
  the candidate is **M2(ii)-violated or arena/pool byte reuse is refused**. The
  candidate does not decide it.
- **M3 — met.** No bare assume exists: `use R(args)` is one application of a
  **named rule with premises the checker checks** (owner decision O15(b)). A
  disequality on two runtime scalars has **no such rule**, so `use i != j` is not
  writable and the only route is a written branch. The base's `use k != i` /
  `use d != 0` routes are **deleted**; §7 re-prices P1 and P17.
- **M8 — RED, declared.** The written vocabulary is: 5 path forms (+ the ghost
  index), 4 type formers (`ptr`, `own`, `slice`, `closure`), 1 row kind `Ρ`, 13
  fact heads (`st`, `own`, `obl`, `owns`, `layout`, `backing`, `len`, `cap`,
  `head`, `gen`, `slot`, `tag`, `RSt`), 5 state values + `ite`, 2 relations
  (`#`, `≤`), 7 clause keywords (`requires`, `ensures`, `invariant`, `use`,
  `exists`, `forall`, `where`), 5 footprint keywords (`reads`, `writes`, `ends`,
  `consumes`, `yields`), `old()`, `origin()`, `unpack`/`pack`, the `read` flag —
  **≈46 constructs**, up from the base's ≈30, against an unset `K` (owner
  decision O6). Declared red, not "small".
- **M7 — met.** Every rejection in this document names one rule, one identity
  path, the failing leaf **with the site of the transition that set it**, and
  the missing fact. The four taught routes are: write the branch; write the
  `invariant` at the head; `use` a named rule; close the divergence inside the
  arm. The base's unrepairable atom-death class no longer exists (`ATOM`).

---

## 7. Where this differs from `CASES.md` and from the base's derivations

`CASES.md`'s 26 expectations were written under `DESIGN.md` Candidate A. Each
row says: **same**, **capability loss**, **deliberate refusal + repair**, or
**correction of the case**.

| Case | This candidate | Difference |
|---|---|---|
| S01 S02 S04 S05 S07 | accept | same |
| S03 S06 | reject at the stated line | same |
| B01 B02 B04 B05 B06 B08 B11 | accept | same (`II-5` leafwise correlation) |
| B03 B07 B09 B10 B12 | as the case states | same |
| B13 | **reject at scope exit** when `own` is a term (`I-3`) | **deliberate refusal + repair**: explicit consumption is selected; automatic cleanup is refused because inserting `if !c { release(a) }` is a release the source did not write (`M2(ii)`, R2's Price). Repair: write the guarded release. The case presents both policies, so this is a selection inside the case, not against it |
| L01 L02 L04 L05 L06 | as the case states | same. L04's *post-loop* obligation is `⊤`, so the enclosing scope exit rejects under `I-3` — the case says exit policy is B13, so the same refusal-and-repair applies |
| L03 | accept **with one written loop invariant** (`II-7`) | **writer cost**, not a capability loss. The case derives the relation by hand and states it is "not yet a selected inference algorithm"; this candidate makes the writer state it |
| **P1** `take(v,j)` | **reject** without `i # j` | **correction of the base's derivation** (soundness F1). The base accepted a double take at may-equal indices, duplicating an affine element |
| **P1** cost | **one written branch per unproved index fact**, not two `use` steps | correction (cost F2, O15(b)). `i < len`, `j < len`, `i ≠ j`, `k ≠ i`, `k ≠ j` are five facts; none is `use`-writable; the accepted shape is `if i != j && … { … } else { outcome }` |
| **P3** `read(p.data)` after `remove(m)` | **reject** (`KEY`, `III-6`) | **correction** (soundness F2). The base accepted a traversal cursor reading ended storage |
| **P3** `use inv_links` | accept, via `JF-QI` residual framing (`III-12`) | **correction of the base's frame claim** (cost S6): the invariant is weakened by each symbolic write, not killed, and the four writes leave four exceptions that the one `use` discharges |
| **P3** slot disjointness | discharged from the pool's **written contract**, not a type premise | correction (cost S6, C3) |
| **P4** `Cursor` | `Cursor<'v,'b>`, 2 parameters | **correction** (soundness F3, m1): the base's 1-parameter form is ill-formed |
| **P4** repair order | the `cap` proof belongs **before** the reserve | correction (soundness F3(b)) |
| **P4** `free(v)` ends `'b` | derivable, by `I-13`'s enumerated owning edges | correction (soundness S6) |
| **P9** `write(r,5)` through `'x\|'y` | **reject** unless `pick` declares the discriminant (`II-15`) | **correction** (soundness F4). The base marked both origins `Init` after a write that touched one |
| **P6** partition disjointness | `JF-LIN` schemas 4 and 5, **named, with side conditions**, not an "O(1) sign test" | correction (cost S4). Still zero written steps for P6's shape, because the shape matches schemas 4 and 5; outside the table the writer writes a branch |
| **P2** block distinctness | from the arena's **written invariant** by `JF-LIN` over frozen bounds (`I-20`, `III-4`), never from name freshness | correction (soundness F6). Freshness is used only to separate *paths* (`III-6`), never emitted as a `#` fact |
| **P13** `read(pool[h])` | accept **only** through the written handle→slot map (`III-10`) | correction (soundness S3, S4). Interior `p` still dies: no fixup form (R8a's price, paid) |
| **P16** `pick<N>` | `pick<Ρx, Ρy, N>(…) -> N<Ρx ∨ Ρy>` with the pointwise lifting rule (`JF-SUB`) | correction (cost F3). The base's `N<'x>` is well-kinded only at arity 1 |
| **P11** Shape 1 | refused (`M2(ii)`); Shape 2's cost is **a word per slot + a branch per `get` + a kill on the fill arm** | correction of the cost line (cost S10) |

### Evidence the owner asked for, stated without selecting

- **O11 (are condition terms admitted at joins).** Under this candidate the
  answer is (b) **with a bound that did not exist before**: one atom per
  identity per point, shrinking on arm entry, never nesting. The price is
  `II-2`'s refusal of a divergence that crosses two independent guards, with two
  named repairs. What (b) buys, derived above: `CASES.md` B02, B03, B05, B08,
  B09, B12 and `P8`'s naive shape are accepted; under (a) (premises only) B03's
  `read(q)` and B05's `read(q)` become `Live`/`⊤` and reject, and P8 needs a
  restructuring. What (b) costs, derived above: `II-2`'s rejection class, ≈4
  extra constructs at M8, and `JF-ENT` in the inner loop of `JF-OVL`.
- **O7 / O12 (is "projections second class, references never stored" an
  acceptable permanent boundary).** This candidate does **not** take that
  boundary: `II-13` makes a projection an ordinary sub-identity name that is not
  consumed and may be stored, and `I-19` lets a nominal hold it by closing the
  identity row. The evidence for the owner is the *price* of not taking it, and
  it is exactly three items: (1) `I-19`'s closed row — `Cursor<'v,'b>`, and a
  quadratic `#`-clause count at any interface that **produces** an aggregate of
  separately-named storages (`O(f²)`, cost S9); (2) `III-8`'s conditional
  `backing`, which is what makes a stored cursor survive a `push` at all, and
  which forces `TERM`, `II-15` and `JF-ORG` into the model; (3) `JF-OVL`'s
  `O(I·d·n³)` per symbolic access, which exists because a stored pointer can
  name a symbolic element. A candidate taking O7(a)/O12(a) pays none of the
  three and loses `P4`'s first conjunct permanently.

---

## 8. Open defects — not repaired here

| # | Defect | Why it is not repaired |
|---|---|---|
| **D1** | **The coarsened-identity residue.** No rule refuses a writer who names one identity where two would do; the program is accepted, silently slower, with no diagnostic. A fact-loss report can only be a diagnostic channel (M1, M5), so it cannot force a fix | Refusing it needs a permission judgment, which is the thing D4 exists to delete. Inherited from the base's §6.1 and not answered |
| **D2** | **M10(a)'s written-step clause is violated**, degree `Θ(S·I·d·n³ + S²·n³)` with `q` possibly 0 | Declared in §6.1. The driver is program size, not proof size; no reformulation of the families moves it |
| **D3** | **`JF-LIN`'s 9-schema table has no adequacy argument.** P6's shape matches; a coalescing arena, a strided tile with two symbolic strides, and a modular index do not, and the only route is a written branch or an `INV` step | Proving the table adequate needs a class of goals nobody has enumerated; enlarging it is unbounded |
| **D4** | **The origin set stays read-only.** `II-15` row 2 refuses a write through `{π₁…πₙ}`, and the named repair (declare the discriminant) is unwritable when the callee selects on a value it does not expose | The sound alternative is a weak `st` update, which yields no usable state; the unsound one is what the base did |
| **D5** | **`M2(ii)` rests on owner decision O3.** `III-6`'s ghost index is the only thing separating a reused arena extent or pool slot from an ended one. If O3(b), `I-23`/`I-24` have no separation and either `M2(ii)` is violated or byte reuse is refused | Not the candidate's to decide |
| **D6** | **`P13`'s compaction is expensive and only half-answered.** The handle→slot map is a word per outstanding handle, `compact` rewrites it, and the `∀`-invariant is re-established wholesale rather than by residual framing (`JF-QI` resets at the call). Interior pointers still die: no fixup form | R8a's price; supplying a fixup channel is a D8 decision |
| **D7** | **Higher-order code over an unknown-length identity row.** `JF-SUB` lifts `ite`/`\|` pointwise through a *known* row; `fn for_each<F>(f: F)` needs `F` to abstract a row of unknown length **and** a footprint row on the function type. The rule shape is stated; its instantiation cost is not bounded | Bounding it needs a row-kind system nobody has priced at M8 |
| **D8** | **`R1(iii)` layout currency is carried by `III-9` alone**, exercised by one derived example and no P-program; the interaction of a variant write with a live interior pointer is stated, not measured | No program in the set exercises it |
| **D9** | **M8 is red at ≈46 constructs against an unset `K`** (owner decision O6), and the elision rule that would shrink the written form (`D12`) is outside this candidate while `P5`/`P16`'s counts depend on it | The candidate depends on a decision it does not make |
| **D10** | **R5(ii)(iii), R8b, R13, R14(i)(ii)(iv)(v)(vi) are unsupplied.** `'a` is the attachment point each would use; none is foreclosed and none is derived | Out of this round's scope; noted so a score on the sequential fragment is not read as a score on the list |


# File: derive-value-semantics-boundary.md

# Derivations: candidate `value-semantics`, part **boundary**

Derived on 2026-09-16 from
[rules-value-semantics.md](rules-value-semantics.md) ONLY. Every accept and
reject cites the rule number that decides it; where the rules do not cover a
line it is marked **rule missing** and the derivation continues. Comment style
from MECHANISM-MAP §3: `// accept: <fact used>`, `// reject: <missing fact>`.

Items assigned: **P4, P7, P8** (PROGRAMS.md) and **L01–L06** (CASES.md
appendix, the fixed-object loop cases).

Two renderings are used, as the candidate's Appendix B defines them, because
CASES.md and P7 are written with Candidate A's storable rebindable locator
`p = ref(a)`, a form this candidate does not have (§1.1: a projection is never
stored; there is no pointer type):

- **binding rendering** — the object is a `var`/`let` binding; `ref(a)`
  disappears and every access is written at the path.
- **pool rendering** — the object is a pool row; `p = ref(a)` becomes
  `let p: Handle<P,Obj> = h_a`, one word of ordinary copyable data that may be
  stored, copied and rebound (III-6). This is the rendering that preserves the
  cases' shape and it is the one used wherever the locator is load-bearing.

Nothing here selects a candidate.

---

## P4 Cursor over a growing vector

Source (PROGRAMS.md §P4):

```text
c = Cursor { at: pointer-or-handle to v, i: 0 }
a = c.read()                       // needs v initialized and i < len
push(v, 5)                         // may or may not reallocate
b = c.read()                       // legal iff the candidate carries "still valid" across push
free(v); c.read()                  // must be refused
```

Required: *a stored pointer or handle to a container in a struct*; validity
decided by state and proof, not by a borrow that blocks `push`.

### P4.a — the program as written

```text
struct Cursor { at : &v, i : u64 }    // reject: missing form — `&v` is not a type. §1.1 gives
                                      //   exactly three name forms and only the branded handle
                                      //   may be stored; a projection may NEVER be stored
                                      //   (§1.1 "may be stored? never"; II-19's second line,
                                      //   "a projection is second class — it cannot be the value
                                      //   of a binding (O7/O12's boundary)")
struct Cursor { at : Handle<P,Vec<u64>>, i : u64 }
                                      // reject: missing fact — `Handle<P,T>` names a ROW of the
                                      //   pool whose brand is P (I-18: "Handle<P,T> is the only
                                      //   type that mentions P"). `v` is an ANONYMOUS backing
                                      //   (I-8) and no rule in set I creates a handle to a
                                      //   backing, a plane, or an element of a non-pool
                                      //   container. I-2's third line refuses the sibling form
                                      //   ("a plane is not a value")
let raw = backing_of(v)               // reject: I-8 — `backing_of` is unwritable; a backing is
                                      //   not a value and has no name
```

P4's first required conjunct has no image. This is DV-1 verbatim ("a cursor
over a frame-local `Vec` stores an index and needs the container re-supplied at
every use"), and DV-2 records that the repair is a non-local signature cascade.

### P4.b — the rendering the rules do admit (index-only cursor)

```text
backing Vec<T> { meta : plane<Meta,1>, data : plane<T, len cap> }
                                      // accept: I-2 declares the plane map; III-1 binds the
                                      //   measure symbols len, cap to μ = v.meta
invariant VecOk : len(v) <= cap(v)    // accept: III-2, one invariant per backing
struct Cursor { i : u64 }             // accept: I-1 / I-2 — ordinary Copy data, no name reaches v
fn read_at(let v.data[c.i], let v.meta, let c: Cursor) -> u64  requires c.i < len(v)
                                      // accept: II-10 (one convention per parameter, the
                                      //   conventions ARE the footprint) + II-13 (two-part
                                      //   footprint: the addressed slot and the metadata plane;
                                      //   NOT `let v`, which II-13's last line refuses at the
                                      //   declaration as a prefix of every element path)
let v = Vec::<u64>::with_capacity(8)  // accept: I-8 — cap(v) = 8, len(v) = 0, support {v.meta}
push(inout v.data[len(v)], inout v.meta, 1)
                                      // accept: I-9; II-11 learns len(v) == old(len(v)) + 1;
                                      //   II-14 binds old(len(v)) := 0, so len(v) == 1
var c = Cursor { i: 0 }               // accept: I-1; Γ gains c.i == 0, support {c}
let a = read_at(let v.data[c.i], let v.meta, let c)
                                      // accept: II-11 discharges `c.i < len(v)` from c.i == 0
                                      //   and len(v) == 1 by REF; III-2 turns it into the R11
                                      //   domain fact c.i < cap(v) through VecOk
                                      // accept: III-3 resolves the index step — `c.i` is a field
                                      //   read of a name not assigned in this span (the same
                                      //   shape III-6 licenses for `p.slots[h.idx]`)
push(inout v.data[len(v)], inout v.meta, 5)
                                      // accept: I-9, ONE rule — the reallocating and
                                      //   non-reallocating cases are not distinguished because
                                      //   nothing in the language can observe the difference
                                      //   (I-8: "R8a's interior pointer into relocatable storage
                                      //   has no instance"; II-19: no projection outlives its
                                      //   statement)
                                      // killed: `len(v) == 1` — v.meta is in FP (II-12)
                                      // re-derived: II-14 binds old(len(v)) := 1 and adds
                                      //   len(v) == 2, support {v.meta}
                                      // survives: `c.i == 0` — support {c}, which FP does not
                                      //   name (II-12, the frame rule's positive direction)
let b = read_at(let v.data[c.i], let v.meta, let c)
                                      // accept: `c.i < len(v)` re-derived by REF from c.i == 0
                                      //   and len(v) == 2. NOTE: no fact about the backing's
                                      //   identity is used, because none exists to be lost
sink v                                // accept: I-4 — BIND(v); the release runs at this statement
let z = read_at(let v.data[c.i], let v.meta, let c)
                                      // reject: missing BIND(v) — "v was consumed at the sink
                                      //   above". I-4 / I-5 classify it R1(a)(ii) and name the
                                      //   ending event. Address reuse never revives it (I-23)
```

Finding for O7/O12, stated because it is the point of the assignment: the
candidate wins P4's second read *for exactly the reason it loses P4's first
line*. `b = c.read()` needs no "still valid" fact at all — a reallocation is
unobservable because no name can outlive the statement that made it (I-8, II-19,
I-5). Remove that restriction and the second read needs the very
backing-identity fact the base model could not carry.

**Verdict P4: differs from the required properties — capability loss.** The
stored pointer-or-handle-to-a-container conjunct is unwritable (§1.1, II-19,
I-18, DV-1), and the named repair — thread `v.data`, `v.meta` through every
intermediate signature — is non-local (DV-2), so M7(a)'s local-fix payload
cannot be honoured. The other two conjuncts (validity across `push` by proof;
`free` then read refused) are delivered exactly, at zero annotation.

---

## P7 Take, put, replace through aliases

Source (PROGRAMS.md §P7):

```text
p, q point to the same slot A holding an affine value
v = take(p); read(q)                  // refused: hole
put(q, move v); read(p)               // allowed
old = replace(p, new); read(q)        // allowed, reads new
free A; read(p)                       // refused forever
```

Required: state is a property of the storage, not of the pointer; take leaves a
hole another alias may fill; free is permanent.

### P7.a — the premise line, projection rendering

```text
with inout A as p, inout A as q { … } // reject: missing fact — PATH(A, A) refuted. II-17's table
                                      //   gives inout/inout on paths whose overlap PATH/EXT does
                                      //   not refute as REFUSED, and two opens of the SAME path
                                      //   can never be refuted (II-16's conflict premise)
let p = projection_of(A)              // reject: missing form — II-19, §1.1: a projection is never
                                      //   the value of a binding (DV-1)
```

### P7.b — the premise line, pool rendering (the rules DO admit it)

```text
let h = insert(inout g.slots[σ], inout g.meta, sink obj)
                                      // accept: I-19 — fresh σ; Live(h) from the ensures;
                                      //   Σ(obj) := unbound
var p: Handle<P,Obj> = h              // accept: §1.1 / III-6 — a handle is one word of ordinary
                                      //   copyable data and may be stored, copied and rebound
let q = p                             // accept: I-1; Γ gains `q == p` by REF, hence
                                      //   σ(q) == σ(p) (III-6: σ(h) is DEFINED to be h.idx, so
                                      //   value equality of handles IS slot-index equality)
                                      // accept: G gains Live(q) from Live(p) — III-6's third
                                      //   example (the fact is established for the CURRENT value)
                                      //   -- see "rule missing" note R2 below
```

Two long-lived writable names for one slot therefore **do** exist in this
candidate. This contradicts the rules file's own §6 **DV-3** ("P7's two-alias
premise is unwritable, so R3 has no acceptance test"): II-17's table is stated
for two accesses *in one sequential span*, and P7's accesses are sequential
statements, which the ladder's Step 0 says need no judgment. Recorded as a
correction to the candidate's own defect list, not as a new rule.

### P7.c — the four required lines, pool rendering

```text
let v = replace(inout g.slots[σ(p)].value, None)
                                      // accept: III-6 — Live(p) => σ(p) ∈ Live(P) => σ(p) < cap(g)
                                      //   by PBounds (I-18); the access is STATICALLY TOTAL, no
                                      //   bounds compare (M2(ii))
                                      // accept: I-7 — `take` is replace(·, None) and is TOTAL on
                                      //   Option<T>; storage identity at the path is UNCHANGED;
                                      //   v : Option<Obj> becomes a new binding (I-1)
let r = read(let g.slots[σ(q)].value)
                                      // ACCEPT, returning None: I-7 + III-11 — the slot holds a
                                      //   VALID Option<Obj>; vacancy is the writer's own
                                      //   discriminant word, not a language state. σ(q) == σ(p)
                                      //   from REF, so this is genuinely the same element
                                      // EXPECTED: reject (hole). The refusal has no instance (DV-4)
match v { Some(x) => g.slots[σ(q)].value = Some(sink x), None => () }
                                      // accept: II-6 — exhaustive arms over the declared sum; the
                                      //   Some arm is I-6 (move into another storage); both arms
                                      //   discharge R2 (C15)
let r2 = read(let g.slots[σ(p)].value)
                                      // accept: the value put through q is read through p —
                                      //   I-6 established the refinement with support
                                      //   {g.slots[σ(q)].value}, and PATH resolves σ(q) and σ(p)
                                      //   to one element. This IS P7's "state is a property of
                                      //   the storage, not of the pointer", delivered by the
                                      //   slot's own discriminant (III-11) rather than by a
                                      //   checker state
let old = replace(inout g.slots[σ(p)].value, Some(sink new))
                                      // accept: I-7 — one exclusive access at the path;
                                      //   Σ(new) := unbound; FRAME (II-12) kills the old contents
                                      //   fact; the returned old value is a new binding
let r3 = read(let g.slots[σ(q)].value)
                                      // accept: reads `new` — I-7's established refinement,
                                      //   support {g.slots[σ(p)].value}, read through the alias
sink old                              // accept: I-4 — the returned Option is affine and must be
                                      //   consumed (R2)
remove(inout g.slots[σ(p)], inout g.meta, p)
                                      // accept: I-20 — Live(p) from G; Live(P) \= {σ(p)};
                                      //   gen(P[σ(p)]) := succ(γ(p))
let r4 = read(let g.slots[σ(p)].value)
                                      // reject: missing Live(p) — gen(P[σ(p)]) = γ(p) was
                                      //   falsified by I-20 / III-7. R1(a)(ii); the refusal is
                                      //   entirely static, no generation compare is emitted
let r5 = read(let g.slots[σ(q)].value)
                                      // reject: missing Live(q) — I-20's kill clause keeps a
                                      //   handle's facts only where EXT proves σ(k) != σ(p),
                                      //   and here σ(q) == σ(p), so the kill is CORRECT, not the
                                      //   DV-6 over-kill. "refused forever" delivered
```

**Verdict P7: differs from the expected verdict — deliberate refusal with a
named repair.** Line 1's `read(q)` accepts and returns `None` instead of being
refused; the repair named by DV-4 is "declare the slot `Option` and write the
arm" (`match g.slots[σ(q)].value { Some(x) => …, None => <the writer's arm> }`),
which converts the case's static refusal into the writer's own arm — M2(ii)
stays intact because the branch is written, but the *refusal* P7 asked for is
gone. The other three required lines are delivered exactly. One correction is
recorded against the candidate's DV-3: the two-alias premise is writable in the
pool rendering, so R3 does have an acceptance test here; DV-3 stands only for a
frame-local or Vec-held slot, where the second name would have to be a stored
projection (DV-1).

---

## P8 Conditional release and loop exits

Source (PROGRAMS.md §P8):

```text
if c { free(a) } else { free(b) }; then use the survivor; then free it
loop { if stop { break }; take(p); ...; put(p) }; free(p) after the loop
```

Required: no runtime drop flag; the writer's own branches carry the state; the
rejection names the path whose state differs.

### P8.a — conditional release

```text
let a = alloc_buf(64)                 // accept: I-1 / I-8
let b = alloc_buf(64)                 // accept: I-1 / I-8
if c { sink a } else { sink b }       // accept per arm: II-1 copies (Σ,Γ,G) into each arm and
                                      //   puts c / !c into Π; I-4 unbinds a on the then-arm and
                                      //   b on the else-arm
                                      // reject AT THE JOIN: missing a definite BIND(a) —
                                      //   Σ_then(a) = unbound, Σ_else(a) = bound. II-2's
                                      //   definiteness clause. Payload: the consuming statement,
                                      //   the arm that did not consume, and the repair
use(let survivor)                     // not reached; `survivor` does not resolve either — no rule
                                      //   in set I binds a name to "whichever of a, b survived"
```

Two properties of this rejection, both required by P8:

```text
// no drop flag: II-2's mixed case is a REJECTION, so no flag is needed and none
//   is representable (§5.1 M2(ii): "the affine release at I-3 and I-4 is
//   unconditional on its edge, so it is neither a branch nor bookkeeping state")
// the rejection names the path: II-2's payload names the arm (M7)
// note: this rejects ONE LINE EARLIER than VERDICT-CORE §2's P8 derivation, which
//   rejects at the later use. II-2 states the move and states that the accepted
//   set is unchanged, because the later use rejected anyway
```

Repair, from II-2's own example:

```text
let s = if c { sink a; b } else { sink b; a }
                                      // accept: each arm unbinds one name and YIELDS the other as
                                      //   a value (I-6 into the new binding s). Σ(a) = Σ(b) =
                                      //   unbound on BOTH arms, so II-2's definiteness holds
use(let s)                            // accept: BIND(s)
sink s                                // accept: I-4 — exactly one release on every path (R2)
```

### P8.b — take/put across a loop with a break

```text
var s: Option<Obj> = Some(sink o)     // accept: I-1 formation is total; I-6 moves o into the slot
loop {
  if stop { break }                   // accept: II-9 — this exit edge carries Σ(s) = bound
  let v = replace(inout s, None)      // accept: I-7 — total on Option<Obj>; storage identity at s
                                      //   unchanged; v : Option<Obj> is a new binding (I-1)
  …                                   // accept: ordinary statements
  s = v                               // accept: I-7 assignment-over; Σ(v) := unbound
}                                     // accept: II-8 — Σ(s) = bound at the head and on the back
                                      //   edge; no name is consumed in a loop
                                      // accept: II-7 — nothing written at the head: no pool is
                                      //   freed into, and the reads need no index proof
match replace(inout s, None) { Some(x) => sink x, None => () }
                                      // accept: II-9 joins the break edge, the normal exit edge
                                      //   and the zero-iteration edge — Σ(s) = bound on all
                                      //   three, so II-2's definiteness holds
                                      // accept: II-6 — R2 discharged on every arm (C15)
```

**Verdict P8: accepted as expected, with a stated cost.** All three required
properties hold: no drop flag (M2(ii), because II-2 makes the mixed case
unrepresentable rather than flagged), the writer's own branches carry the state,
and the rejection names the differing path. The costs: (1) the conditional-release
half is a *rejection plus a restructuring*, not an acceptance, and the rejection
moves one line earlier than VERDICT-CORE §2's derivation (II-2); (2) the
take/put pair in the loop is total (I-7, III-11), so the "hole" P8 inherits from
CASES B10–B13 / L04–L06 has no instance — which is why part (b) accepts with
zero annotations.

---

## L01 Take and restoration preserve the next iteration's entry condition

Expected: **ACCEPT** (`read(p)` yields 10, including when n is zero).

Binding rendering (`p` and `q` both designate A; with no storable locator both
collapse to the path):

```text
var s: Option<u64> = Some(10)         // accept: I-1 — formation is total, s holds a valid Option;
                                      //   Γ gains `s == Some(10)`, support {s}
loop n {
  let v = replace(inout s, None)      // accept: I-7 — `take` is total on Option<T>; the checker
                                      //   does nothing, there is no hole and no state to enter
                                      // killed: `s == Some(10)` — FRAME (II-12), s is the
                                      //   footprint path
                                      // Γ gains `s == None`, support {s}; v == Some(10) at the
                                      //   first iteration, support {v}
  s = v                               // accept: I-7 assignment-over (`put`); Σ(v) := unbound;
                                      //   Γ gains `s == v`, support {s, v}
}                                     // accept: II-8 — Σ(s) = bound at the head and on the back
                                      //   edge. v is bound INSIDE the body, so it leaves scope at
                                      //   the brace (I-3); it is Copy here, nothing is released
                                      // accept: II-7 — nothing is written at the head
                                      // Γ head := Γ_entry ∩ Γ_backedge (II-7): `is_some(s)` is in
                                      //   both; `s == Some(10)` is NOT — the back-edge fact is
                                      //   `s == v` whose support mentions v, dropped at I-3
let r = read(let s)                   // accept: III-11 — the read is TOTAL on Option; no fact is
                                      //   consulted. Accepts on the zero-iteration edge and the
                                      //   back edge alike (II-9's join is over both)
sink s                                // accept: I-4 (the case's remaining disposal obligation)
```

**Verdict L01: accepted as expected, with a stated cost.** The ACCEPT holds and
the loop head carries no written annotation (II-7). The cost is that the case's
annotation `// ACCEPT: 10` is not a checker fact after the loop: `s == Some(10)`
survives the entry edge but not the back edge, so the II-7 intersection keeps
only `is_some(s)`, and `is_some(s)` itself is not needed because the read is
total. A writer who needs the value writes the loop invariant and a `use` step
(INV, family 10).

---

## L02 An empty exit state becomes the next iteration's input

Expected: **REJECT** (for unrestricted n, the second take accesses an empty
slot).

```text
var s: Option<u64> = Some(10)         // accept: I-1
loop n {
  let v = replace(inout s, None)      // accept: I-7 — `take` is TOTAL on Option<T>. On the second
                                      //   iteration the slot holds None and `replace` returns
                                      //   None. No fact is consulted, so none can be missing
                                      // accept: I-1 binds v; it is Copy here, so I-3 releases
                                      //   nothing at the brace. For an affine payload, I-3's
                                      //   unconditional release at the brace discharges R2
}                                     // accept: II-8 — Σ(s) = bound at the head and on the back
                                      //   edge; nothing is consumed in a loop
                                      // accept: II-7 — Γ head keeps only what is in both edges;
                                      //   nothing needs it
```

The case's rejection has no instance: it rests on a language state "live but
empty", which D3 removed (§1.3: "there is no per-storage state"; III-11: "there
is no language state `Uninit`").

**Verdict L02: differs from the expected verdict — deliberate refusal with a
named repair.** The repair, from DV-4 and III-11: the writer declares the slot
`Option` (it already is) and **writes the arm** —
`match replace(inout s, None) { Some(x) => …, None => <the writer's own arm> }`
— turning the case's static refusal into the writer's own branch. M2(ii) is not
violated (the branch is written, not inserted), but the refusal L02 asks for is
gone, and the writer who omits the arm gets a silent `None` instead of a
diagnostic.

---

## L03 Relative initialization invariant while physical roles alternate

Expected: **ACCEPT with no annotation** (post-loop `read(p)` yields 10).

The swap `tmp = p; p = q; q = tmp` is the case's whole content and needs a
rebindable name for a storage. The binding rendering has no image for it (a
binding *is* the storage; §1.1). The pool rendering does, because a handle is
data:

```text
let ha = insert(inout g.slots[σ], inout g.meta, sink obj_a)
let hb = insert(inout g.slots[σ'], inout g.meta, sink obj_b)
                                      // accept: I-19 twice — fresh σ, σ'; Live(ha), Live(hb);
                                      //   Live(ha) survives the second insert because the
                                      //   footprint names slots[σ'] and meta and σ' is fresh
                                      //   (I-19's third line, III-8)
let old_b = replace(inout g.slots[σ(hb)].value, None)
                                      // accept: III-6 static domain from PBounds; I-7 total
var p: Handle<P,Obj> = ha             // accept: §1.1 / III-6 — ordinary copyable data
var q: Handle<P,Obj> = hb             // accept: same
loop n {
  let v = replace(inout g.slots[σ(p)].value, None)
                                      // accept: III-6 — Live(p) => σ(p) < cap(g) by PBounds;
                                      //   statically total, no bounds compare (M2(ii))
                                      // accept: I-7 — total on Option<Obj>; no "full" fact needed
  g.slots[σ(q)].value = v             // accept: I-7 — total; no "empty" fact needed either
  let tmp = p; p = q; q = tmp         // accept: ordinary data assignment (III-6's third example);
                                      //   G re-establishes Live(p) for the CURRENT value from
                                      //   Live(q), and Live(q) from Live(tmp)
                                      //   -- rule missing R2 below: the GHOST family (§5 row 11)
                                      //      states only insert and remove
                                      // killed: III-3 — every fact whose support path has an index
                                      //   term mentioning p or q dies at these assignments (a kill
                                      //   by NAME). Costless here: no such fact is needed
}                                     // accept: II-8 — no name is consumed in the loop
                                      // accept: II-7 — G head := G_entry ∩ G_backedge; Live(p)
                                      //   and Live(q) hold on both edges, so the intersection is
                                      //   non-trivial and NOTHING is written at the head: the
                                      //   body frees nothing into g (II-7's one exception)
let r = read(let g.slots[σ(p)].value) // accept: Live(p) at the exit join (II-4, II-9); III-6's
                                      //   static domain; III-11's total read
```

Sub-claims the case also states, derived:

```text
let r2 = read(let g.slots[σ(q)].value)
                                      // ACCEPT, returning None. EXPECTED: "a post-loop read(q)
                                      //   rejects". DV-4 / III-11 again — deliberate refusal
remove(inout g.slots[σ(p)], inout g.meta, p)
                                      // accept: I-20 — Live(p) from G
remove(inout g.slots[σ(q)], inout g.meta, q)
                                      // reject: missing Live(q) — I-20's kill clause keeps a
                                      //   handle's facts only where EXT proves σ(q) != σ(p), and
                                      //   handles are opaque data, so that proof comes from a
                                      //   written pool invariant or a written requires and from
                                      //   nowhere else (III-8, DV-6). EXPECTED: "release(p);
                                      //   release(q) accepts and consumes both obligations"
                                      // repair named: a written distinctness premise on the pool
                                      //   interface, the same shape as P1's `i != j`
```

Under the **binding rendering** the two removes are `sink sa; sink sb`, which
accept (PATH refutes overlap over distinct roots, I-4) — but that rendering
cannot write the loop at all, so the two cannot be had together.

**Verdict L03: accepted as expected on its ACCEPT line** (pool rendering, zero
annotations at the head, matching the case's "with no annotation"), **with two
recorded differences on its sub-claims**: the post-loop `read(q)` accepts
instead of rejecting (deliberate refusal, DV-4), and `release(p); release(q)`
rejects on the second release (capability loss, DV-6). As in L01 the value
annotation `// ACCEPT: 10` is not a checker fact: III-3 kills every contents
fact whose index term mentions the reassigned `p`/`q` at the swap, so the value
would need a written loop invariant and a `use` step. The relative invariant the
case describes (p's target full, q's target empty) is writable and verified by
INV, but it is **not needed** for acceptance, because take, put and read are all
total (I-7, III-11).

---

## L04 A release on a break edge need not preserve the loop-head state

Expected: **ACCEPT** (the body is safe; the break edge carries dead A and the
consumed obligation).

```text
let a = make_obj(10)                  // accept: I-1
loop n {
  if stop {
    sink a                            // accept: I-4 — BIND(a) holds on this path; the release
                                      //   runs at this statement
    break                             // accept: II-9 — this exit edge carries Σ(a) = unbound
  }
  let r = read(let a)                 // accept: BIND(a) on this path — II-1 checked the arms in a
                                      //   copy of (Σ,Γ,G) and this path never took the then-arm
}                                     // accept: II-8 — the back edge has Σ(a) = bound, matching
                                      //   the head; nothing is consumed on it
                                      // reject AT THE LOOP EXIT JOIN: missing a definite BIND(a)
                                      //   — II-9 joins ALL exit edges INCLUDING the
                                      //   zero-iteration edge, and II-2's definiteness clause
                                      //   applies: unbound on the break edge, bound on the normal
                                      //   and zero-iteration edges. Payload names the consuming
                                      //   statement and the arm
```

The rejection lands at the join itself, not at a later use, so it fires even
though the fragment has nothing after the loop. This is exactly II-9's worked
example.

Repair, named (Appendix B's and II-9's own): carry the survivor as a value, so
that the binding map is definite on every edge and the Option discriminant — the
writer's own data (III-11) — carries what the case wanted the checker to carry:

```text
var slot: Option<Obj> = Some(sink a)  // accept: I-1 / I-6
loop n {
  if stop {
    match replace(inout slot, None) { Some(x) => sink x, None => () }
                                      // accept: I-7 total; II-6 — R2 on both arms (C15)
    break                             // accept: II-9 — Σ(slot) = bound on this edge
  }
  match slot { Some(v) => read(let v), None => () }
                                      // accept: II-6 — the payload binder is a PROJECTION because
                                      //   the scrutinee is not consumed; both arms yield
}
match replace(inout slot, None) { Some(x) => sink x, None => () }
                                      // accept: II-9's join is definite (Σ(slot) = bound on the
                                      //   break, normal and zero-iteration edges); R2 on both arms
```

**Verdict L04: differs from the expected verdict — deliberate refusal with a
named repair** (II-9 + II-2's definiteness clause; the repair is the
`Option`-slot restructuring above, which the candidate's Appendix B also names).
The cost is that the repair is not local: it changes the resource's declaration,
not the break edge.

---

## L05 A checked Boolean relation guards later iterations after release

Expected: **ACCEPT**, with exactly one completed release including the
zero-iteration case.

```text
let a = make_obj(10)                  // accept: I-1
var active = true                     // accept: I-1 — ordinary Copy data
loop n {
  if active {                         // accept: II-1 — the then-arm gets Π ++ [active]; the
                                      //   condition is a PREMISE, never a term in Γ (II-5, III-10)
    let r = read(let a)               // accept: BIND(a) at the loop head
    if stop {                         // accept: II-1
      sink a                          // accept: I-4 — BIND(a) on this path
      active = false                  // accept: ordinary assignment; III-10 kills the premise
                                      //   `active` for the remainder of the region
    }                                 // reject AT THIS JOIN: missing a definite BIND(a) —
                                      //   Σ_then(a) = unbound, Σ_else(a) = bound (the empty else
                                      //   arm). II-2's definiteness clause
                                      // missing fact, stated precisely: the correlation between
                                      //   the source Boolean `active` and BIND(a). Recovering it
                                      //   would make the arm's effect a TERM over `active`,
                                      //   which II-5 / III-10 / O11(a) refuse: "Π is discarded at
                                      //   the join; never joined, never a term in Γ"
  }
}                                     // not reached; had the inner join been passed, II-8 would
                                      //   reject on the back edge instead ("v is consumed at line
                                      //   k and the loop may run again")
if active { sink a }                  // not reached; Σ(a) is not definite at the loop exit either
```

Two facts worth separating, because L05 is the round's O11 evidence:

```text
// under O11(a) (this candidate): REJECT at the inner join. `active` is ordinary
//   data; no rule indexes BIND by a condition, and none may be added without an
//   ite term and a condition-atom growth bound (II-5's consequence clause, M10)
// under O11(b): the inner join would carry BIND(a) = ite(stop, unbound, bound),
//   the loop head would carry ite(!active, unbound, bound), and `if active {
//   sink a }` would discharge it — L05 would ACCEPT as written
```

Repair, named: make the resource a value and let the discriminant the writer
already declared do the work `active` was doing — which is the same repair as
L04, and it also deletes the flag:

```text
var slot: Option<Obj> = Some(sink a)  // accept: I-1 / I-6
loop n {
  match slot {
    Some(v) => { read(let v)
                 if stop { match replace(inout slot, None) { Some(x) => sink x, None => () } } }
                                      // accept: II-6 — the payload binder is a projection; I-7 is
                                      //   total; R2 on every arm (C15). Σ(slot) = bound on EVERY
                                      //   path, so II-2's definiteness holds at every join
    None    => ()                     // accept: the arm `active = false` used to select
  }
}
match replace(inout slot, None) { Some(x) => sink x, None => () }
                                      // accept: exactly one release on every path including the
                                      //   zero-iteration edge (R2), with no drop flag — the
                                      //   discriminant is the writer's own word (III-11, M2(ii))
```

**Verdict L05: differs from the expected verdict — deliberate refusal with a
named repair** (II-2's definiteness clause under II-5 / III-10 / O11(a); the
repair is the `Option`-slot form above). Together with B12 this is the sharpest
O11 discriminator in the set: the case is accepted verbatim under O11(b) and
rejected at its first inner join under O11(a).

---

## L06 A hole may leave through break when the continuation only releases it

Expected: **ACCEPT** (A is live on every exit, so `release(p)` accepts).

```text
var s: Option<u64> = Some(10)         // accept: I-1 — formation is total
loop n {
  let v = replace(inout s, None)      // accept: I-7 — total on Option<u64>; v : Option<u64> is a
                                      //   new binding (I-1); Γ gains `s == None`, support {s}
  if stop { break }                   // accept: II-9 — this exit edge carries Σ(s) = bound
                                      // rule missing: no rule states what happens to `v`, a name
                                      //   bound INSIDE the body, on a break edge. I-3 states the
                                      //   affine release "for every name x still bound at the
                                      //   closing brace"; II-9 states the join over exit edges and
                                      //   says nothing about scope exit. Costless here (v is Copy,
                                      //   nothing is released); load-bearing for an affine payload,
                                      //   which is precisely the case's own disclaimer
  s = v                               // accept: I-7 assignment-over (`put`); Σ(v) := unbound
}                                     // accept: II-8 — Σ(s) = bound at the head and on the back
                                      //   edge
                                      // accept: II-9 — the break, normal and zero-iteration edges
                                      //   ALL carry Σ(s) = bound, so II-2's definiteness holds
match replace(inout s, None) { Some(x) => (), None => () }
sink_slot(s)                          // accept: I-4 / I-3 — the slot's own release runs on every
                                      //   path; nothing is consumed conditionally
```

Sub-claim the case also states:

```text
let r = read(let s)                   // ACCEPT, returning None, when inserted before the release.
                                      //   EXPECTED: "inserting an unconditional read(p) before
                                      //   release rejects because the break exit is empty".
                                      //   DV-4 / III-11 — deliberate refusal, same family as L02
```

**Verdict L06: accepted as expected**, with one flagged rule gap (the release of
a body-bound name on a break edge, above) and one sub-claim difference (the
pre-release read accepts instead of rejecting, DV-4). The acceptance costs
nothing at the loop head (II-7 writes nothing) and needs no state fact, because
every operation in the case is total.

---

## M1 / M2 observations arising from these nine derivations

Recorded because the round's hard rules require them, not as a ranking.

- **M1** — every accept above used BIND, RESOLVE, PATH, EXT, REF, FRAME,
  OLDCHAIN, GHOST or INV at a fixed, written site. No derivation needed a
  search, an iteration to a cap, or a budget. The one place a search would have
  been required is II-14 step 4 at an `ensures`-less writer of `v.meta`, and the
  rules refuse it (DV-5); P4.b did not hit it because `push` states its relation.
- **M2(ii)** — no derivation above needed a check the source did not write. P4's
  `v.data[c.i]` has its domain from `requires` + VecOk (III-2); P7's and L03's
  `g.slots[σ(h)]` have theirs from PBounds (III-6), so no bounds compare and no
  generation compare is emitted; P8, L04, L05 replace a drop flag with a
  *rejection* (II-2) or with the writer's own `Option` discriminant (III-11).
  The one branch inserted anywhere in the repairs is a `match` arm the writer
  wrote. **M2(ii) holds across all nine items under O3(a)**; §5.1's O3(b) caveat
  is untouched by these derivations because none of them materializes γ.
- **M2(i)** — the only failure edges reached are written arms (`None` of an
  `Option`, the writer's `stop` branch).

---

## Summary table

| Item | Expected | Under `value-semantics` | Verdict | Rule(s) that decide it |
|---|---|---|---|---|
| **P4** | stored cursor; valid across `push`; refused after `free` | cursor field unwritable; index rendering accepts both reads and refuses after `sink` | **differs: capability loss** (DV-1, DV-2) | §1.1, II-19, I-18, I-8; then I-9, II-12, II-14, I-4 |
| **P7** | hole refused; put/replace through aliases; free permanent | premise IS writable (pool rendering); 3 of 4 lines exact; the hole read accepts returning `None` | **differs: deliberate refusal with repair** (write the `Option` arm; DV-4) + correction to DV-3 | III-6, I-7, I-6, I-20, III-11; II-17 read as span-local |
| **P8** | no drop flag; branches carry the state; rejection names the path | delivered; part (a) rejects at the join and is repaired by yielding the survivor; part (b) accepts unannotated | **accepted with a stated cost** (rejection one line earlier; take/put total) | II-2, II-1, I-4, II-8, II-9, I-7 |
| **L01** | ACCEPT | ACCEPT, no annotation | **accepted as expected** (cost: the value 10 is not carried out of the loop) | I-1, I-7, II-7, II-8, III-11 |
| **L02** | REJECT (second take) | ACCEPT (both takes total) | **differs: deliberate refusal with repair** (write the `None` arm; DV-4) | I-7, III-11, II-8 |
| **L03** | ACCEPT with no annotation | ACCEPT, no annotation (pool rendering) | **accepted as expected**; sub-claims differ: `read(q)` accepts (DR, DV-4), `release(p); release(q)` rejects (CL, DV-6) | III-6, I-7, I-19, I-20, II-7, III-3, III-8 |
| **L04** | ACCEPT | REJECT at the loop exit join | **differs: deliberate refusal with repair** (`Option` slot + `replace`/`match`) | II-9, II-2, I-4 |
| **L05** | ACCEPT | REJECT at the inner `if stop` join | **differs: deliberate refusal with repair** (`Option` slot; the flag disappears) | II-2, II-5, III-10 (O11(a)) |
| **L06** | ACCEPT | ACCEPT | **accepted as expected**; one rule gap flagged; sub-claim `read` before release accepts (DR, DV-4) | I-7, II-8, II-9, II-2, III-11 |

### Rules missing (marked, not improvised)

| # | Item | Line | What the rules do not state |
|---|---|---|---|
| R1 | **L06** | `if stop { break }` with `v` bound in the body | Which names bound inside a loop body run their affine release on a `break` edge. I-3 states the release "for every name x still bound at the closing brace"; II-9 states only the join over exit edges. Costless for L06's Copy payload, load-bearing for an affine one |
| R2 | **L03** | `let tmp = p; p = q; q = tmp` | How `G` propagates on a handle-to-handle assignment. III-6's third example accepts it ("the fact is established for the CURRENT value"), but the GHOST family (§5 row 11) states only *insert* and *remove*, and no rule in sets I–III states the assignment case |
| R3 | **L01**, **L03** | the loop head | Whether a fact whose *term* (not support path) mentions a name bound inside the body survives the head intersection. §1.3 defines support as "the resolved paths whose contents it depends on", which resolves it if `v` counts as support of `s == Some(v)`; I-3 and II-18 key their drops on support only, so the reading is not stated anywhere |

### Counts

4 accepted (P8 — accepted with a stated cost; L01, L03, L06 — accepted as
expected, L01 and L03 with stated precision costs); 1 capability loss (P4);
4 deliberate refusals (P7, L02, L04, L05); 0 case corrections; 3 rule gaps.
Two sub-claim differences sit inside accepted items (L03's `release(p);
release(q)`, a DV-6 capability loss; L03's and L06's post-loop reads, DV-4
deliberate refusals). One correction is recorded against the candidate's own
defect list (DV-3, at P7).


# File: derive-value-semantics-straight-and-early-branches.md

# Derivation: candidate `value-semantics`, part `straight-and-early-branches`

Items: CASES.md **S01–S07** and **B01–B06**. Every line below is derived from
[rules-value-semantics.md](rules-value-semantics.md) only — rule set I (storage
creation/destruction), rule set II (split/rejoin), rule set III (identity ↔
runtime value), plus the judgment families of its §5. Every accept names the
fact used and the rule that supplies it; every reject names the missing fact.
Where the rules do not cover a line it is marked **rule missing** and the
derivation continues.

Comment style from MECHANISM-MAP §3: a pseudocode line, then
`// accept: <fact used>` or `// reject: <missing fact>`.

Nothing here selects, ranks or repairs a candidate.

---

## 0. Rendering, fixed once

CASES.md's `p = ref(a)` is Candidate A's **storable, rebindable locator**. This
candidate has no such form: a projection is second class and may never be
stored (§1.1, II-16, II-19; the reject line
`let keep = projection_of(out[0..4])` in II-19 is the boundary). The rules file
supplies two renderings (Appendix B) and both are used here:

- **pool rendering** (primary, because it preserves the cases' shape): the
  objects are pool rows; `p = ref(a)` is `var p = ha`, an ordinary one-word
  copyable `Handle<P,T>` (III-6) that may be stored, copied and rebound.
- **binding rendering** (secondary): objects are `var a : Obj`, `ref(a)`
  disappears, every access is written at the path `a`. Candidate A's locator
  variable has no image at all.

Fixed preamble for the pool rendering, cited once and not repeated:

```text
backing Pool<P,T> { meta : plane<PoolMeta,1>, slots : plane<Slot<T>, len cap> }
                                     // accept: I-2 / I-18; the plane map is the distinctness proof (G3)
invariant PBounds : forall σ. σ ∈ Live(P) => σ < cap(g)
                                     // accept: I-18 — the bridge that makes g.slots[σ(h)] statically
                                     //   total (III-6); without it the access is an R11 site
let g : Pool<P,T> = Pool::new()      // accept: I-8, anonymous backing formation
```

| CASES.md operation | Rendered as | Rule |
|---|---|---|
| `a = object(10)` | `let ha = insert(inout g.slots[σ], inout g.meta, 10)` | I-19 |
| `p = ref(a)` | `var p = ha` (Copy data) | I-1, III-6 |
| `q = p` | `let q = p` | I-1 |
| `read(p)` | `let x = g.slots[σ(p)].value` | III-6, I-18 |
| `write(p,n)` | `g.slots[σ(p)].value = n` | I-7 |
| `v = take(p)` | `let v = replace(inout g.slots[σ(p)].value, None)` | I-7, III-11 |
| `put(p,v)` | `g.slots[σ(p)].value = v` | I-7 |
| `old = replace(p,n)` | `let old = replace(inout g.slots[σ(p)].value, n)` | I-7 |
| `release(p)` | `remove(inout g.slots[σ(p)], inout g.meta, p)` | I-20 |

Two standing consequences of the preamble, used below without re-deriving:

1. **Domain totality.** `g.slots[σ(h)]` resolves to `g.slots[h.idx]` (III-6);
   the obligation `h.idx < cap(g)` is discharged from `σ(h) ∈ Live(P)` and
   `PBounds` (I-18). No bounds compare and no generation compare is emitted
   (M2(ii)).
2. **No hole.** There is no `Uninit` and no per-storage state (I-1, §1.3).
   Vacancy is a discriminant the writer declared (III-11), so `take` is
   `replace(inout d, None)` and is **total** (I-7), and a read of an emptied
   slot is a total read of an `Option` (III-11).

**A note on pending obligations.** CASES.md lets a fragment end with an
undischarged obligation. Under this candidate that is not a state: every name
still bound at the closing brace is released there, unconditionally and with no
flag (I-3). The pool `g` carries its rows. So no case below ends in an R2
defect for want of a written `release`.

---

## 1. Straight-line cases

### S01 — sequential aliases and stable copies

```text
let ha = insert(inout g.slots[σ], inout g.meta, 10)
                                     // accept: I-19 — fresh σ ∉ Live(P); G gains σ(ha) ∈ Live(P)
                                     //   and gen(P[σ(ha)]) = γ(ha) from `ensures Live(result)`
                                     // rule missing (MR-3): I-19's only ensures is Live(result);
                                     //   no rule relates the inserted 10 to g.slots[σ(ha)].value,
                                     //   so the case's initial value never enters Γ
let hb = insert(inout g.slots[σ'], inout g.meta, 20)
                                     // accept: I-19 — σ' fresh; Live(ha) survives because
                                     //   FP = {g.slots[σ'], g.meta} and σ' != σ(ha) by freshness
                                     //   (I-19's third example, III-8); GHOST keeps ha's pair
                                     // Γ gains ha.idx != hb.idx (freshness, I-19/III-8)
var p = ha                           // accept: I-1 — a Handle is one-word Copy data (III-6).
                                     //   Γ: p == ha, support {p, ha}. G(p) = the pair, from
                                     //   Live(ha) at the current value (III-6, third example)
let q = p                            // accept: I-1. Γ: q == p, support {q, p}. G(q) from G(p)
g.slots[σ(p)].value = 11             // accept: I-7 — destination resolves (III-3 at p's CURRENT
                                     //   value, III-6 third example); domain from Live(p)+PBounds
                                     // FRAME (II-12): kills every contents fact whose support may
                                     //   overlap g.slots[p.idx]; PATH cannot refute overlap with
                                     //   g.slots[q.idx], so q's contents facts die here
                                     // Γ gains g.slots[p.idx].value == 11, support {g.slots[p.idx]}
let x = g.slots[σ(q)].value          // accept as an ACCESS: Live(q)+PBounds give a total domain
                                     // rule missing (MR-1): the case's value 11 needs the fact
                                     //   keyed at g.slots[p.idx] to discharge a goal at
                                     //   g.slots[q.idx] given q == p. II-3 keys facts by SYNTACTIC
                                     //   path equality after RESOLVE, II-12 and EXT only REFUTE
                                     //   overlap (the kill direction), and REF's fragment is stated
                                     //   as linear over indices with no selector congruence
p = hb                               // accept: I-7 over Copy data; Γ: p == hb
                                     // FRAME (II-12) kills `p == ha` and `q == p` — support {p}
                                     // III-3 kills every fact whose support path has an index step
                                     //   mentioning p: `g.slots[p.idx].value == 11` dies here
                                     // G(p) re-established from Live(hb) (III-6, third example)
g.slots[σ(p)].value = 21             // accept: I-7; domain from Live(p) = Live(hb) + PBounds
                                     // Γ gains g.slots[p.idx].value == 21, support {g.slots[p.idx]}
let y = g.slots[σ(q)].value          // accept as an ACCESS: q was never assigned, so q.idx is a
                                     //   stable index term (III-3) and Live(q) is untouched (no
                                     //   remove has run, so GHOST has killed nothing)
                                     // rule missing (MR-1) again: "11, still A" is not derivable.
                                     //   The case's subject — that q still designates A — IS
                                     //   derivable, as III-3's stability of q.idx
let z = g.slots[σ(p)].value          // accept: the value 21 IS derivable — the fact established at
                                     //   the previous write has support {g.slots[p.idx]}, p has not
                                     //   been assigned since, and no footprint names that path
```

Binding rendering: `ref` has no image, so the case reduces to `a = 11; read(a)`
and its subject (two names for one storage) has no instance.

**Verdict: accepted as expected, with a stated cost** — all three reads accept,
but two of the three value annotations ("11", "11, still A") are not derivable:
a write through `p` kills `q`'s contents facts (PATH cannot refute
`p.idx`/`q.idx` overlap) and no rule transports a fact to a provably equal path
(MR-1). The candidate's own Appendix B records S01 as unchanged; that verdict
is right about acceptance and does not predict the loss of the value facts the
case is written to exhibit.

### S02 — replacement preserves storage and transfers the old value

```text
let ha = insert(inout g.slots[σ], inout g.meta, 10)   // accept: I-19 (MR-3 as in S01)
var p = ha                                            // accept: I-1, III-6
let q = p                                             // accept: I-1; Γ: q == p
let old = replace(inout g.slots[σ(p)].value, 20)
                                     // accept: I-7 — "storage identity at d is UNCHANGED (no
                                     //   storage ends, none is created)". This IS the case's
                                     //   claim "the target identity remains A", derived, not assumed
                                     // accept: the returned old value becomes a new binding (I-7, I-1)
                                     // FRAME (II-12): kills contents facts overlapping g.slots[p.idx]
                                     //   — the case's "old value-dependent facts about A expire"
                                     // Γ gains g.slots[p.idx].value == 20, support {g.slots[p.idx]}
                                     // rule missing (MR-2): no relation is stated between `old` and
                                     //   the pre-state contents. II-14/III-13 bind old(m) only for
                                     //   MEASURE symbols bound by a plane map (III-1); a slot's
                                     //   contents is not a measure, and I-7 states no ensures
let a1 = g.slots[σ(q)].value         // accept as an ACCESS: Live(q)+PBounds
                                     // rule missing (MR-1): the value 20 sits at g.slots[p.idx]
let a2 = g.slots[σ(ha)].value        // accept as an ACCESS: Live(ha)+PBounds; `p == ha` is in Γ
                                     // rule missing (MR-1): same, keyed at g.slots[p.idx]
let a3 = old                         // accept: I-1 — `old` is an ordinary binding
                                     // rule missing (MR-2): the value 10 is not in Γ
```

**Verdict: accepted as expected, with a stated cost** — all three reads accept
and the case's structural claim (identity preserved, old facts expire) is
derivable from I-7 and II-12, but none of the three values (20, 20, 10) is,
because `replace` states no `ensures` over the returned value (MR-2) and facts
are keyed per written path (MR-1).

### S03 — a hole is a property of the target, not one locator

The payload must be declared `Option<u64>` for `take` to be writable at all
(III-11); that declaration is the only rendering this candidate admits.

```text
let ha = insert(inout g.slots[σ], inout g.meta, Some(10))   // accept: I-19
var p = ha ; let q = p                                      // accept: I-1 ×2; Γ: q == p
let v = replace(inout g.slots[σ(p)].value, None)
                                     // accept: I-7 — "`take` is `replace(inout d, None)` and is
                                     //   TOTAL on Option<T>: the checker does nothing. There is no
                                     //   hole and no language state to enter"
                                     // FRAME (II-12) kills contents facts overlapping g.slots[p.idx]
                                     // Γ gains is_none(g.slots[p.idx].value), support {g.slots[p.idx]}
let r = g.slots[σ(q)].value
                                     // accept: the read is TOTAL (I-7's second example, III-11).
                                     //   Domain from Live(q)+PBounds. NOTHING is missing, so the
                                     //   case's REJECT has no instance: there is no `Uninit` (I-1),
                                     //   no per-storage state (§1.3) and no occupancy structure (III-11)
match r { Some(k) => k, None => 0 }
                                     // accept: II-6 — the refusal the case asks for is available
                                     //   ONLY as a written arm over the writer's own discriminant
```

The variants (`read(a)`, a second `take(q)`) accept by the same two rules.

**Verdict: differs from the expected verdict — deliberate refusal with repair.**
Repair named by the rules: declare the slot `Option<T>` and write the `match`
arm (III-11, II-6); the program's behaviour is recovered, the *static refusal*
is not. This is the rules file's DV-4 and it agrees with its Appendix B.

### S04 — another alias can restore the hole

```text
let ha = insert(inout g.slots[σ], inout g.meta, Some(10))   // accept: I-19 (MR-3)
var p = ha ; let q = p                                      // accept: I-1 ×2; Γ: q == p
let v = replace(inout g.slots[σ(p)].value, None)
                                     // accept: I-7 total (S03's line 3)
                                     // rule missing (MR-2): `v == Some(10)` is not established
g.slots[σ(q)].value = v              // accept: I-7 — the destination resolves (III-3) and the
                                     //   domain comes from Live(q)+PBounds. `put`'s premise
                                     //   "target live and EMPTY" has no instance: the assignment is
                                     //   total over the declared Option (III-11)
                                     // (for an affine payload this line is I-6 instead: Σ(v) := unbound)
                                     // Γ gains g.slots[q.idx].value == v, support {g.slots[q.idx]}
let x = g.slots[σ(p)].value          // accept as an ACCESS: Live(p)+PBounds
                                     // rule missing (MR-1): the value needs the fact at
                                     //   g.slots[q.idx] to discharge a goal at g.slots[p.idx]
                                     // rule missing (MR-2): even transported, `v` has no value fact
```

"p and q continue to target A, never the new location holding v" is derivable:
neither `p` nor `q` is assigned, so both index terms are stable (III-3), and
`v` is a separate binding (I-1) that names no slot.

**Verdict: accepted as expected, with a stated cost** — the accept holds and
the case's aliasing claim is derivable, but the returned value 10 is not
(MR-1, MR-2, MR-3).

### S05 — a hole need not be restored before its storage ends

```text
let ha = insert(inout g.slots[σ], inout g.meta, Some(10))   // accept: I-19 (MR-3)
var p = ha                                                  // accept: I-1, III-6; Γ: p == ha
let v = replace(inout g.slots[σ(p)].value, None)
                                     // accept: I-7 total (MR-2 on v's value)
remove(inout g.slots[σ(p)], inout g.meta, p)
                                     // accept: I-20 — requires Live(p); G holds σ(p) ∈ Live(P) and
                                     //   gen(P[σ(p)]) = γ(p). I-20 has NO premise about the row's
                                     //   contents, so "there is no requirement to restore a hole
                                     //   before releasing its storage" is derived, not assumed
                                     // effect: Live(P) \= {σ(p)}; gen bumped; GHOST keeps only the
                                     //   facts of names k with EXT-proved σ(k) != σ(p) — here only
                                     //   hb-shaped names, none in scope
                                     // cost: I-20's returned value is a NEW BINDING carrying its
                                     //   obligations. Copy here, so free; for an affine payload the
                                     //   case would acquire an R2 obligation it does not have
let r = v                            // accept: I-1 — v is an ordinary binding, independent of the
                                     //   ended row; no fact of v was rooted at g (I-20's Γ filter)
                                     // rule missing (MR-2): the value 10 is not in Γ
```

Binding rendering is cleaner and also accepts: `var s : Option<u64> = Some(10);
let v = replace(inout s, None); sink s` — `sink` (I-4) has no contents premise
either.

**Verdict: accepted as expected, with a stated cost** — the accept and the
case's substantive claim (an emptied slot may end; the moved-out value is
independent, and is not disposed twice) are both derivable; the value 10 is not
(MR-2, MR-3), and an affine payload would add one R2 obligation via I-20.

### S06 — replacement requires old content at its commit

```text
let ha = insert(inout g.slots[σ], inout g.meta, Some(10))   // accept: I-19
var p = ha ; let q = p                                      // accept: I-1 ×2
let v = replace(inout g.slots[σ(q)].value, None)
                                     // accept: I-7 total
                                     // Γ gains is_none(g.slots[q.idx].value)
let old = replace(inout g.slots[σ(p)].value, v)
                                     // accept: I-7 — `replace` is TOTAL on Option<T>. Its premises
                                     //   are (a) the destination path resolves (III-3) and (b) the
                                     //   conflict table permits an inout at d (P-c, II-17). There
                                     //   is NO "old content" premise anywhere in I-7
                                     // the case's REJECT has no instance (DV-4): the returned value
                                     //   is None on this path, which is data, not a failure
                                     // rule missing (MR-2): `old == None` is not even stated —
                                     //   I-7 carries no ensures over the returned value
match old { Some(k) => …, None => … }
                                     // accept: II-6 — the refusal the case asks for is recoverable
                                     //   only as a written arm
```

The case's note that "operand evaluation that performs a take must not leave an
earlier initialization premise authoritative" has no instance either: there is
no initialization premise (I-1, §1.3).

**Verdict: differs from the expected verdict — deliberate refusal with repair.**
Repair named: match the returned `Option` (III-11, II-6). Same family as S03
(DV-4); agrees with the rules file's Appendix B.

### S07 — release is permanent and affects every access path

```text
let ha = insert(inout g.slots[σ], inout g.meta, 10)   // accept: I-19
var p = ha ; let q = p                                // accept: I-1 ×2; Γ: p == ha, q == p
remove(inout g.slots[σ(p)], inout g.meta, p)
                                     // accept: I-20 — Live(p) is in G. Releasing through a copy of
                                     //   the identity is ordinary: the handle IS the identity
                                     //   (III-6); this candidate has no owner-only release form
                                     // effect: Live(P) \= {σ(p)}; gen(P[σ(p)]) := succ(γ(p)) (III-7)
                                     // GHOST (I-20, [stated here]): keep facts of k iff EXT proves
                                     //   σ(k) != σ(p). For k = q, Γ holds q == p, which proves the
                                     //   OPPOSITE; for k = ha, Γ holds p == ha, likewise
                                     //   → LIVE(q) and LIVE(ha) are both killed here
let x = g.slots[σ(q)].value
                                     // reject: missing gen(P[σ(q)]) = γ(q) — killed at the previous
                                     //   line. R1(a)(ii): address reuse never revives an ended
                                     //   identity (III-7). The refusal is entirely static; no
                                     //   generation compare is emitted (M2(ii))
```

Variants, each derived:

```text
let y = g.slots[σ(ha)].value         // reject: missing gen(P[σ(ha)]) = γ(ha) — killed by GHOST,
                                     //   since p == ha refutes the distinctness escape
g.slots[σ(q)].value = 7              // reject: I-7's domain obligation σ(q) < cap(g) is discharged
                                     //   only from σ(q) ∈ Live(P) + PBounds (III-6, I-18), and
                                     //   σ(q) ∈ Live(P) is missing. A write cannot revive dead storage
let z = replace(inout g.slots[σ(q)].value, None)
                                     // reject: same missing fact at the same obligation
remove(inout g.slots[σ(q)], inout g.meta, q)
                                     // reject: I-20 requires Live(q) — missing. Double release is
                                     //   refused by the ghost facts, not by a runtime count (R2)
let r = q                            // accept: a Handle is Copy data (III-6); copying an inert
                                     //   identity reads no slot and consults no fact
```

Binding rendering: `sink a` (I-4) then any use of `a` rejects on `BIND(a) =
false`, naming the consuming statement — but there is no second access path, so
the case's subject only has an instance in the pool rendering.

**Verdict: accepted as expected** — every expected reject is delivered, with
one missing fact named per line, and the permitted operations (release through
a copy, copying an inert handle) accept.

---

## 2. Branch cases

### B01 — one locator can have alternative targets

Rendering note: I-1 forbids an uninitialized binding, so `p` is introduced
before the branch. The alternative `let p = if cond { ha } else { hb }` is
**rule missing (MR-4)**: II-2 joins `Σ` per name in scope at the branch point
and II-3/II-4 do the same for `Γ` and `G`; no rule states which facts a name
bound AT the join from an arm-yielded value carries. II-2's own repair example
(`let s = if c { sink a; b } else { sink b; a }`) relies on that transfer for
`Σ` and the file is silent for `Γ` and `G`. The derivation below routes around
the gap.

```text
let ha = insert(inout g.slots[σ],  inout g.meta, 10)   // accept: I-19
let hb = insert(inout g.slots[σ'], inout g.meta, 20)   // accept: I-19; Γ: ha.idx != hb.idx (freshness)
var p = ha                                             // accept: I-1; G(p) from Live(ha) (III-6)
if cond { p = ha } else { p = hb }
                                     // accept: II-1 — each arm is checked in a COPY of (Σ,Γ,G);
                                     //   Π gains cond / !cond
                                     // then-arm: Γ: p == ha; G(p) = the pair (III-6, third example)
                                     // else-arm: Γ: p == hb; G(p) = the pair (III-6, third example)
                                     // II-3 join: Γ := ⋂ by syntactic equality — `p == ha` and
                                     //   `p == hb` each hold on ONE arm, so NEITHER survives. No
                                     //   disjunction, no ite, no condition term (II-3, II-5, O11(a))
                                     // II-4 join: G := ⋂ per handle NAME. Both arms hold the same
                                     //   pair for the name p, so Live(p) SURVIVES
                                     // II-2 join: Σ unchanged on both arms
let x = g.slots[σ(p)].value
                                     // accept: Live(p) (II-4) + PBounds (I-18) give σ(p) < cap(g);
                                     //   the access is statically total (III-6), and the read is
                                     //   total over whatever the slot holds (III-11)
```

**Verdict: accepted as expected, with a stated cost** — the read accepts. The
case's stated mechanism ("the analysis represents both executions") is NOT what
this candidate does: neither execution is represented, the target identity is
dropped at the join (II-3, II-5), and the accept comes from a per-NAME liveness
fact that both arms happen to state identically (II-4). The cost is invisible
here and is what B02, B03, B05 and B06 turn on. Rule missing: MR-4 (facts of an
arm-yielded value bound at the join), routed around by the rendering.

### B02 — target and initialization must remain correlated

```text
let ha = insert(inout g.slots[σ],  inout g.meta, Some(10))   // accept: I-19
let hb = insert(inout g.slots[σ'], inout g.meta, Some(20))   // accept: I-19; Γ: ha.idx != hb.idx
var p = ha ; var old = None                                  // accept: I-1 ×2 (no Uninit)
if cond {
  p = ha                             // accept: I-7 over Copy data; G(p) from Live(ha) (III-6)
  old = replace(inout g.slots[σ(hb)].value, None)
                                     // accept: I-7 total; Π holds `cond` here (II-1) and is not used
                                     // Γ gains is_none(g.slots[hb.idx].value)
} else {
  p = hb                             // accept: I-7; G(p) from Live(hb) (III-6)
  old = replace(inout g.slots[σ(ha)].value, None)
                                     // accept: I-7 total; Γ gains is_none(g.slots[ha.idx].value)
}
                                     // II-3 join: `p == ha`, `p == hb`, and the two is_none facts
                                     //   each hold on ONE arm → ALL dropped
                                     // II-5: Π is discarded at the join; `cond` never enters Γ and
                                     //   no fact is indexed by it (O11(a))
                                     // II-4 join: Live(p) survives (both arms, same pair)
let x = g.slots[σ(p)].value
                                     // accept: Live(p)+PBounds; total read of Option (III-11)
```

Swapped variant (each arm takes p's own selected object), which CASES.md
expects to REJECT:

```text
if cond { p = ha; old = replace(inout g.slots[σ(ha)].value, None) }
else    { p = hb; old = replace(inout g.slots[σ(hb)].value, None) }
let x = g.slots[σ(p)].value
                                     // accept: identical derivation. I-7 made the take total, so
                                     //   the "empty selected target" the variant is about has no
                                     //   instance (DV-4), and the read is total (III-11)
```

Sharpened as an O11 discriminator — replace the final read by one that needs a
refinement:

```text
let k = unwrap(g.slots[σ(p)].value)  requires is_some(g.slots[σ(p)].value)
                                     // reject in BOTH the original and the swapped program:
                                     //   missing is_some(g.slots[p.idx].value). The fact that would
                                     //   distinguish them — `cond => is_none(g.slots[hb.idx])` — is
                                     //   exactly the condition-indexed fact II-3/II-5 refuse (O11(a))
```

**Verdict: differs from the expected verdict — deliberate refusal with repair.**
The case's ACCEPT survives but its discriminating variant, whose REJECT is the
whole point, also accepts. Repair named: declare the payload `Option` and write
the `match` arm (III-11, II-6). Under **O11(b)** the sharpened form would be
derivable on both arms and the original/swapped pair would separate; under
O11(a) the two programs are indistinguishable. Direct evidence for O11.

### B03 — a write initializes the selected target, not its entire may-set

```text
let ha = insert(inout g.slots[σ],  inout g.meta, Some(10))   // accept: I-19
let hb = insert(inout g.slots[σ'], inout g.meta, Some(20))   // accept: I-19
let old_a = replace(inout g.slots[σ(ha)].value, None)        // accept: I-7 total
let old_b = replace(inout g.slots[σ(hb)].value, None)        // accept: I-7 total
var p = ha ; if cond { p = ha } else { p = hb }
                                     // accept: II-1 / II-4 as B01. `p == ha`, `p == hb` dropped (II-3)
let q = p                            // accept: I-1; Γ: q == p; G(q) from G(p) (III-6)
g.slots[σ(p)].value = Some(9)        // accept: I-7; domain from Live(p) (II-4) + PBounds
                                     // FRAME (II-12): the two is_none facts die — PATH cannot refute
                                     //   overlap of g.slots[p.idx] with g.slots[ha.idx] or [hb.idx]
                                     // Γ gains g.slots[p.idx].value == Some(9), support {g.slots[p.idx]}
let y = g.slots[σ(q)].value          // accept as an ACCESS: Live(q)+PBounds
                                     // rule missing (MR-1): the case's 9 sits at g.slots[p.idx] and
                                     //   `q == p` cannot move it to g.slots[q.idx]
let z = g.slots[σ(ha)].value         // accept: Live(ha) is intact (no remove has run) and the read
                                     //   is TOTAL over the Option (III-11). The case's REJECT
                                     //   ("A may still be empty") has no instance (DV-4)
```

`read(b)` derives identically.

**Verdict: differs from the expected verdict — deliberate refusal with repair.**
`read(q)` accepts as the case expects (though its value needs MR-1); `read(a)`
accepts where the case rejects. Repair named: `match g.slots[σ(ha)].value { … }`
— the refusal returns as a written arm (III-11, II-6), never as a rejection.

### B04 — a copied uncertain target supports take and restoration

```text
let ha = insert(…, Some(10)) ; let hb = insert(…, Some(20))   // accept: I-19 ×2 (MR-3 on both values)
var p = ha ; if cond { p = ha } else { p = hb }               // accept: II-1/II-4 as B01
let q = p                                                     // accept: I-1; Γ: q == p
let v = replace(inout g.slots[σ(p)].value, None)
                                     // accept: I-7 total; domain from Live(p)+PBounds
                                     // FRAME (II-12): contents facts of g.slots[ha.idx] and
                                     //   g.slots[hb.idx] die — overlap with g.slots[p.idx] unrefuted
let t = g.slots[σ(ha)].value         // accept: total read of Option (III-11). The case's
                                     //   "between take and put, reading a or b unconditionally
                                     //   rejects" has no instance (DV-4)
g.slots[σ(q)].value = v              // accept: I-7; domain from Live(q)+PBounds; total (III-11)
                                     // Γ gains g.slots[q.idx].value == v, support {g.slots[q.idx]}
let x = g.slots[σ(ha)].value         // accept as an ACCESS: Live(ha)+PBounds
                                     // rule missing (MR-1/MR-2/MR-3): the value 10 was never in Γ
                                     //   (I-19 states no contents ensures), and what the restore
                                     //   established is keyed at g.slots[q.idx]
let y = g.slots[σ(hb)].value         // accept as an ACCESS; the value 20 likewise not derivable
```

**Verdict: differs from the expected verdict — deliberate refusal with repair.**
The two final ACCEPTs hold as accesses; the intermediate REJECTs are gone
(DV-4, repair: declare `Option` and write the arm). Stated cost on top of the
refusal: the case's conclusion "afterward both original slots are initialized"
is not derivable — the take killed both slots' contents facts and the put
restored one fact keyed on `q`'s path only.

### B05 — correlated distinct targets survive selected reclamation

```text
let ha = insert(inout g.slots[σ],  inout g.meta, 10)   // accept: I-19
let hb = insert(inout g.slots[σ'], inout g.meta, 20)   // accept: I-19
                                     // Γ: ha.idx != hb.idx — σ' is FRESH (I-19, III-8)
var p = ha ; var q = hb                                // accept: I-1 ×2; G(p), G(q) (III-6)
if cond { p = ha ; q = hb } else { p = hb ; q = ha }
                                     // accept: II-1 — each arm in a copy of (Σ,Γ,G)
                                     // then-arm Γ: p == ha, q == hb ; else-arm Γ: p == hb, q == ha
                                     // II-3 join: each of those four facts holds on ONE arm → all
                                     //   dropped. `σ(p) != σ(q)` is TRUE and DERIVABLE on both arms
                                     //   (from p == ha, q == hb, ha.idx != hb.idx and III-6's
                                     //   one-field record equality) but it is not a MEMBER of either
                                     //   Γ_i, and II-3 intersects the fact SETS, not their closures
                                     // II-4 join: Live(p), Live(q) survive (per NAME, both arms)
                                     // II-5: Π discarded; cond never enters Γ (O11(a))
remove(inout g.slots[σ(p)], inout g.meta, p)
                                     // accept: I-20 — Live(p) survived the join (II-4)
                                     // GHOST (I-20 [stated here], III-8): keep the facts of k iff
                                     //   EXT proves σ(k) != σ(p). For k = q, Γ after the join holds
                                     //   ha.idx != hb.idx but NOTHING tying p or q to ha or hb
                                     //   → LIVE(q) is KILLED
let x = g.slots[σ(q)].value
                                     // reject: missing σ(q) ∈ Live(P) and gen(P[σ(q)]) = γ(q).
                                     //   Payload: "remove at the previous line may have ended q's
                                     //   row; σ(q) != σ(p) is not proved"
remove(inout g.slots[σ(q)], inout g.meta, q)
                                     // reject: I-20 requires Live(q) — the same missing fact
```

Two derived observations the rules file's Appendix B does not state:

1. **The straight-line form accepts.** With `var p = ha; var q = hb` and no
   branch, EXT proves `σ(p) != σ(q)` from `p == ha`, `q == hb`,
   `ha.idx != hb.idx`, so GHOST keeps `LIVE(q)` and both later lines accept.
   The killer in B05 is therefore **II-3's syntactic intersection at the join**
   (O11(a)), not DV-6 by itself; DV-6's escape hatch exists and the join is
   what removes its input.
2. **The arm-duplicated form accepts.** Writing
   `if cond { remove(p); read(q); remove(q) } else { … }` accepts on each arm,
   because inside an arm the correlation is present. That is a repair with a
   non-local cost (the continuation is duplicated per join), not a local fix.

Binding rendering: `remaining`-free B05 cannot be written at all (DV-1), but
the P8 restructuring accepts —
`let (x, y) = if cond { (sink a, sink b) } else { (sink b, sink a) }; sink x;
read(y); sink y` — both arms consume both names (II-2 definiteness satisfied)
and `x`, `y` are fresh bindings (I-1).

**Verdict: differs from the expected verdict — capability loss** in the pool
rendering (the objects keep their identity; the correlation between the two
names is destroyed at the join and no local repair restores it). In the binding
rendering it is instead a **deliberate refusal with the named repair** "yield
the survivors as values (the P8 restructuring)", at the cost that the objects
are no longer identified by a stored name. Under **O11(b)** the correlation
would survive the join and the case would accept as written: B05 is the
sharpest O11/O12 discriminator in this batch.

### B06 — rebinding must update relations without retargeting saved copies

```text
let ha = insert(…, 10) ; let hb = insert(…, 20)   // accept: I-19 ×2; Γ: ha.idx != hb.idx
var p = ha ; var q = hb                           // accept: I-1 ×2
if cond { p = ha ; q = hb } else { p = hb ; q = ha }
                                     // accept: II-1; as B05 — II-3 drops the four correlation facts,
                                     //   II-4 keeps Live(p) and Live(q)
let saved = p                        // accept: I-1. Γ: saved == p, support {saved, p}
                                     // G(saved) = the pair, from G(p) (III-6, third example)
p = q                                // accept: I-7 over Copy data. Γ gains p == q
                                     // FRAME (II-12): FP = {(p, inout)} kills every fact whose
                                     //   support names the storage p — `saved == p` DIES HERE.
                                     //   This is the case's own point ("the old relation cannot
                                     //   remain attached to the variable names"), derived
                                     // III-3 also kills every fact whose support path has an index
                                     //   step mentioning p
                                     // G(p) re-established from Live(q) (III-6, third example)
remove(inout g.slots[σ(p)], inout g.meta, p)
                                     // accept: I-20 — Live(p) holds
                                     // GHOST: keep k iff EXT proves σ(k) != σ(p).
                                     //   k = q: Γ holds p == q, which proves the OPPOSITE → killed
                                     //   k = saved: Γ holds NO relation between saved and p — the
                                     //   arm facts were dropped at the join and `saved == p` was
                                     //   killed at the rebinding → killed
let x = g.slots[σ(saved)].value
                                     // reject: missing σ(saved) ∈ Live(P) and gen(P[σ(saved)]) =
                                     //   γ(saved). The case expects ACCEPT
let y = g.slots[σ(q)].value
                                     // reject: missing gen(P[σ(q)]) = γ(q); σ(q) == σ(p) is proved,
                                     //   so GHOST's escape is unavailable. The case expects REJECT
                                     //   and gets it, with the missing fact named
```

Derived observation, again sharper than Appendix B's "same as B05": B06's loss
is **not** caused by the join. Delete the branch and keep
`var p = ha; var q = hb; let saved = p; p = q; remove(p); read(saved)` — the
equality `saved == p` is still killed by FRAME at `p = q` (its support names
the storage `p`), and no rule rewrites it to `saved == ha` before the kill.
There is no substitution rule at an assignment: II-12 kills by support and I-7
adds only the new fact. So the reject survives without any branch at all.

**Verdict: differs from the expected verdict — capability loss.**
`read(saved)` rejects where the case accepts; `read(q)` rejects as expected.
The cause is II-12 killing an alias equality at a rebinding with no substitution
or re-keying rule, compounded by II-3 at the join. No local repair is available:
a written pool invariant (the repair named for DV-6) cannot state a fact about
two local handle variables, and the binding rendering has no locator to rebind
(DV-1).

---

## 3. Missing rules, collected

| # | Missing rule | Where the rules stop | Items |
|---|---|---|---|
| **MR-1** | **Use-direction path equality.** No rule says when a fact established at resolved path `g.slots[p.idx]` may discharge a goal at `g.slots[q.idx]` given `p == q`. | II-3 keys facts by *syntactic* equality of (fact, support) after RESOLVE; RESOLVE (§5 family 2) walks steps and checks stability, it does not substitute equalities; II-12 and EXT only *refute* overlap (the kill direction); REF (§5 family 9) is stated as "the same linear fragment as EXT" with nothing about congruence over a slot selector. The kill direction is therefore conservative and the use direction is silent — a fact can be destroyed by an alias it can never be read through. | S01, S02, S04, B03, B04 |
| **MR-2** | **`old()` over a non-measure path.** `replace` (I-7) returns the old value and no rule relates it to the pre-state contents. | II-14 and III-13 bind `old(m)` only for MEASURE symbols bound by the plane map (III-1). A slot's contents is not a measure, and I-7 states no `ensures`. | S02, S04, S05, S06, B04 |
| **MR-3** | **Pool `insert` states no contents `ensures`.** I-19's only `ensures` is `Live(result)`, so nothing ever relates the inserted value to `p.slots[σ(result)].value`. | I-19's effect adds two G facts and applies FRAME; no Γ fact about the row is created. Every CASES.md value that traces to `object(10)` is therefore absent from Γ in the pool rendering. | S01–S06, B01–B04 (all values) |
| **MR-4** | **Facts of an arm-yielded value bound at the join.** II-2/II-3/II-4 join per NAME in scope at the branch point; nothing states what `Γ`/`G` facts a name bound *at* the join from an arm's yielded value carries. | II-2's own repair example (`let s = if c { sink a; b } else { sink b; a }`) needs the transfer for `Σ`, and II-3/II-4 are silent for `Γ` and `G`. The derivations above route around it by binding before the branch. | B01 (and the repair forms named for B05, B10, L04) |

## 4. Divergences from the candidate's own Appendix B

The rules file's Appendix B re-derives the same 26 cases. Writing the
derivations out line by line changes three of its entries and sharpens two:

- **S01, S02, S04, S05** are recorded there as unchanged ("—"). They accept,
  but none of their value annotations is derivable in the pool rendering
  (MR-1, MR-2, MR-3). The accept is preserved; the property each case was
  written to exhibit is not.
- **B05** is attributed there to DV-6 (`remove` kills liveness of every handle)
  with the repair "a written distinctness premise on the pool interface". The
  derivation shows the straight-line form *accepts*: DV-6's EXT escape is
  available and II-3's syntactic join is what removes its input. The correct
  attribution is O11(a), and a pool-interface premise cannot repair it because
  the missing fact is about two local variables.
- **B06** is attributed there to "same as B05". The derivation shows B06
  rejects with the branch deleted: II-12 kills `saved == p` at the rebinding
  and no rule substitutes `saved == ha` first. The cause is the absence of a
  substitution/re-keying rule at assignment, not the join.
- **B01** is recorded as accepted; the derivation adds that the accept comes
  from a per-NAME liveness fact stated identically by both arms (II-4), not
  from representing both executions, and that the natural rendering
  (`let p = if …`) is not covered by any rule (MR-4).
- **B02, B03, B04, S03, S06** agree with Appendix B's **DR** classification;
  the line-by-line form adds the sharpened B02 variant that makes it an O11
  discriminator rather than only a DV-4 instance.

## 5. M1 and M2 as they appear in this batch

- **M1** (deterministic, budget-free, no search): every step used above is a
  table lookup, a syntactic query or a bounded per-name operation — BIND (two
  passes), RESOLVE (`O(d)`), PATH/PLANE, FRAME (`O(F·k·d)`), GHOST (`O(H)` EXT
  queries per remove). No line in this batch required a search, a fixpoint
  beyond BIND's height-2 lattice, or a budget. **Satisfied in this batch.**
- **M2(i)** (no unstated failure edge): every refusal above is a static
  rejection naming a missing fact; every admitted failure is a written `match`
  arm over a writer-declared discriminant (III-11). **Satisfied in this batch.**
- **M2(ii)** (no check, bookkeeping state or outcome-selecting branch the
  source did not write): the accepted accesses emit no bounds compare and no
  generation compare (III-6, III-7, PBounds), the releases at I-20 are
  unconditional on their edge, and no drop flag appears because no mixed
  consumption survives II-2. **Satisfied in this batch, and conditional on
  O3(a)** — under O3(b) the ghost generation of III-7 falls, I-20's kill has no
  carrier, and S07's and B05's refusals would need a runtime compare instead.

---

## 6. Summary table

| Item | CASES.md expects | Derived here | Verdict | Missing rules |
|---|---|---|---|---|
| **S01** | ACCEPT (11 / 11 still A / 21) | all reads accept; only the last value is derivable | accepted as expected, with a stated cost (alias value facts) | MR-1, MR-3 |
| **S02** | ACCEPT ×3 | all reads accept; identity preservation derivable, no value is | accepted as expected, with a stated cost (no value facts) | MR-1, MR-2, MR-3 |
| **S03** | REJECT (read of empty) | ACCEPT — total read of an `Option` | differs: deliberate refusal, repair = declare `Option` + written arm (III-11, II-6) | (MR-3) |
| **S04** | ACCEPT (10) | accepts; restoration and alias stability derivable, value not | accepted as expected, with a stated cost | MR-1, MR-2, MR-3 |
| **S05** | ACCEPT (release empty, read 10) | accepts; I-20 has no contents premise | accepted as expected, with a stated cost (value; affine payload adds one R2 obligation) | MR-2, MR-3 |
| **S06** | REJECT (no old value) | ACCEPT — `replace` is total | differs: deliberate refusal, repair = match the returned `Option` | MR-2, MR-3 |
| **S07** | ACCEPT release, REJECT every later access | identical, each reject naming the missing ghost fact | accepted as expected | — |
| **B01** | ACCEPT | accepts, from a per-name liveness fact; target identity dropped at the join | accepted as expected, with a stated cost | MR-4 |
| **B02** | ACCEPT, swapped variant REJECTS | both accept; the sharpened form rejects on both | differs: deliberate refusal, repair = `Option` + arm; O11 evidence | (MR-3) |
| **B03** | ACCEPT `read(q)`, REJECT `read(a)` | both accept | differs: deliberate refusal, repair = `Option` + arm | MR-1 |
| **B04** | ACCEPT ×2, intermediate reads REJECT | all accept; no value derivable | differs: deliberate refusal, repair = `Option` + arm | MR-1, MR-2, MR-3 |
| **B05** | ACCEPT `read(q)` and the second release | both REJECT: `LIVE(q)` killed, `σ(p) != σ(q)` dropped at the join | differs: capability loss (pool rendering); DR with the P8 restructuring in the binding rendering | — |
| **B06** | ACCEPT `read(saved)`, REJECT `read(q)` | both REJECT | differs: capability loss | — |

Counts for this batch: **6 accepted** (five of them with a stated cost),
**5 deliberate refusals**, **2 capability losses**, **0 case corrections**,
**4 distinct missing rules** across 9 items.


# File: derive-value-semantics-late-branches.md

# Derivations: candidate `value-semantics`, part `late-branches`

Items: CASES.md **B07, B08, B09, B10, B11, B12, B13**; PROGRAMS.md **P1, P3**
as regression checks. Research date: 2026-09-16.

Every line below is derived from
[rules-value-semantics.md](rules-value-semantics.md) only, citing the rule
number on each `accept` and the missing fact on each `reject`. Comment style is
MECHANISM-MAP §3: `// accept: <fact used>` / `// reject: <missing fact>`.
Where the rules do not cover a line it is marked `// rule missing: <what>` and
the derivation continues past it. Nothing here selects or ranks a candidate.

## Renderings

CASES.md B07–B13 are written in Candidate A's notation (`p = ref(a)`, a
storable rebindable locator). This candidate has no such form (II-19: a
projection cannot be the value of a binding). The rules file offers two
renderings and both are used below, as Appendix B of the rules does:

- **binding rendering** — the object is `var a: …`; `ref(a)` disappears and
  every access is written at the path `a`.
- **pool rendering** — the object is a pool row; `p = ref(a)` becomes
  `let p: Handle<P,Obj> = h_a`, ordinary copyable data (III-6).

CASES.md's scope fixes object contents as **ordinary copy integers**, so no
content obligation is at issue in B07–B09; the disposal obligation of the
object itself is what B10–B13 are about.

Two standing observations that recur below, stated once:

- **O-1 (the hole is data).** `take(p)` renders as `replace(inout d, None)`,
  which I-7 declares **total** on `Option<T>`, and `read` of an `Option` is a
  total read of a valid discriminant (III-11). Every CASES expectation of the
  form "REJECT: the slot is empty" therefore has no instance. This is the
  rules' own DV-4.
- **O-2 (the mixed-consumption join).** II-2's definiteness clause rejects at
  the join whenever `Σ(x)` differs across arms. Every CASES expectation that a
  conditional release leaves a conditional obligation therefore rejects one
  line earlier than the case's own failure point. This is the rules' own
  statement that no drop flag is representable (M2(ii)).

---

## B07 — A conditional hole prevents reading but not scalar overwrite

CASES.md expects: `read(p)` **REJECT** ("A may be empty"); the variant
`write(p, 20); read(p)` **ACCEPT** on both paths.

### Main program (binding rendering)

```text
var a: Option<u64> = Some(10)        // accept: I-1 — formation is total; a holds a valid Option.
                                     //   Γ ∪= is_some(a), support {a}
if cond {                            // accept: II-1 — arms are checked in a COPY of (Σ,Γ,G);
                                     //   Π := Π ++ [cond] for the then-arm
  let old = replace(inout a, None)   // accept: I-7 — `take` is replace(inout d, None) and is TOTAL
                                     //   on Option<T>; FP = {(a,inout)}; Γ := FRAME(Γ,FP) (II-12)
                                     //   kills is_some(a) (support {a} is named); old is a new
                                     //   binding (I-1). u64 payload is Copy, so R2 is vacuous here
}                                    // accept: I-3 — `old` leaves scope; facts rooted at old dropped
                                     // accept: II-2 — Σ(a) = bound on BOTH arms, so no definiteness
                                     //   rejection; II-3 — Γ := Γ_then ∩ Γ_else, so is_some(a) is
                                     //   gone (it holds on the else-arm only); II-5 — cond is
                                     //   discarded from Π at the join
let r = read(let a)                  // accept: III-11 — the read is TOTAL on Option<u64>; no state
                                     //   is consulted and none exists. Returns None on the
                                     //   original-true path, Some(10) on the false path
assert is_some(a)                    // reject: missing `is_some(a)` — II-3's intersection dropped
                                     //   it. Payload names the arm (II-3) and the repair:
                                     //   `match a { Some(v) => …, None => … }`
```

### Variant (`write` then `read`)

```text
var a: Option<u64> = Some(10)        // accept: I-1
if cond { let old = replace(inout a, None) }
                                     // accept: I-7, as above
a = Some(20)                         // accept: I-7 — assignment-over; storage identity at `a` is
                                     //   UNCHANGED, no occupancy premise is consulted on either
                                     //   path. Γ ∪= is_some(a), support {a} (I-6 effect clause)
let r = read(let a)                  // accept: III-11; and `is_some(a)` now holds on both paths
                                     //   because the assignment is after the join, not inside an arm
```

The variant's point — that an ordinary scalar assignment must not inherit
`put`'s empty-only premise or `replace`'s full-only premise — is delivered by
rule: I-7 states no such premise, and III-11 makes vacancy the writer's
discriminant rather than a language state.

**Verdict: differs from the expected verdict: deliberate refusal with repair
(O-1 / DV-4 — the hole is the writer's declared `Option` discriminant, III-11;
the named repair is to write the `match` arm, which is where the case's REJECT
reappears as a written arm rather than a rejection).** The variant is accepted
as expected.

---

## B08 — Repeating an unchanged condition recovers a path's state

CASES.md expects: **ACCEPT**, `read(p)` yielding 20 on the true path and 10 on
the false path, *by the second true edge implying the earlier take occurred*.

```text
var a: Option<u64> = Some(10)        // accept: I-1
let cond: bool = read_flag()         // accept: I-1 — `let`, so III-3's index/premise stability
                                     //   question does not arise: cond is never assigned
if cond { let old = replace(inout a, None) }
                                     // accept: I-7 (total); FRAME kills is_some(a) on this arm
                                     // then-arm Γ: is_none(a);  else-arm Γ: is_some(a)
                                     // accept: II-2 — Σ(a) = bound on both arms
                                     // accept: II-3 — Γ := ∩ = neither is_some(a) nor is_none(a)
                                     // accept: II-5 — cond leaves Π at the join
if cond { a = Some(20) }             // accept: I-7 — assignment-over, no premise on occupancy
                                     // accept: II-1 — a FRESH premise [cond] enters Π here;
                                     //   III-10 — "no relation between the two premises is
                                     //   carried". The first region's Π is not recoverable
                                     // then-arm Γ: is_some(a);  else-arm Γ: (nothing about a)
                                     // accept: II-3 — Γ := ∩ = nothing about a
let r = read(let a)                  // accept: III-11 — TOTAL read of a valid Option. At run time
                                     //   it yields Some(20) on the true path and Some(10) on the
                                     //   false path, exactly the case's values
assert is_some(a)                    // reject: missing `is_some(a)` — II-3's intersection. The
                                     //   correlation the case relies on ("the second true edge
                                     //   implies the earlier take occurred") would require the
                                     //   first arm's effect to be a term over `cond`, refused by
                                     //   II-5 / III-10 (O11(a)). Repair named by II-5: write ONE
                                     //   `if cond { … }` containing both statements
```

Merged form, for the record:

```text
if cond { let old = replace(inout a, None); a = Some(20) }
                                     // accept: I-7 twice inside one region; at the join II-3 keeps
                                     //   is_some(a) only if the else-arm also has it — it does,
                                     //   from I-1's formation, never killed on that arm
assert is_some(a)                    // accept: is_some(a) is in BOTH arms' Γ (II-3)
```

**Verdict: accepted as expected** — the read accepts and the runtime values are
the case's values. Stated cost, and a divergence from the rules file's own
Appendix B: Appendix B classes B08 **CC** on the ground that "the case's
mechanism has no instance". Line by line the *verdict* does not differ; what
differs is the **ground**. The acceptance is not obtained by recovering the
path's state (II-5 and III-10 forbid that); it is obtained because III-11 makes
the read total so that no state ever needed recovering. The case's *claim about
what the checker knows* — that a repeated unchanged condition restores a path
fact — is corrected: under this candidate it is false, and the residual
refinement `is_some(a)` is not derivable. This is direct evidence for **O11**:
under O11(b) the refinement would be derivable; under O11(a), as these rules
are written, it is not, and B08 still accepts because nothing needed it.

---

## B09 — Conditions refer to captured values, not permanent variable names

CASES.md expects: `read(p)` **REJECT** ("the original true path remains
empty"); the variant with `if !cond { put(p, 20) }` **ACCEPT**.

```text
var a: Option<u64> = Some(10)        // accept: I-1
var cond: bool = read_flag()         // accept: I-1 — a `var`, assigned below
if cond { let old = replace(inout a, None) }
                                     // accept: I-7; join by II-2 (Σ(a) bound on both arms),
                                     //   II-3 (Γ := ∩ = nothing about a), II-5 (cond leaves Π)
cond = !cond                         // accept: I-7 — ordinary assignment on a bound var.
                                     //   III-10's kill clause ("assignment to a name occurring in
                                     //   c KILLS the premise for the remainder of the region") has
                                     //   NOTHING TO KILL here: the premise was already discarded
                                     //   at the join one line earlier (II-5). III-3's kill clause
                                     //   likewise has no instance: no support path has an index
                                     //   term mentioning `cond`
if cond { a = Some(20) }             // accept: I-7; a fresh premise [cond] over the NEW value
                                     //   (III-10 "re-derived: by re-testing the condition — which
                                     //   re-enters Π as a new premise")
let r = read(let a)                  // accept: III-11 — total read. Returns None on the
                                     //   original-true path
assert is_some(a)                    // reject: missing `is_some(a)` — II-3. Payload: the arm
```

### Variant (`if !cond { put(p, 20) }` after the negation)

```text
cond = !cond                         // accept: I-7
if !cond { a = Some(20) }            // accept: I-7 inside the arm. Note that `!cond` here is the
                                     //   ORIGINAL cond only as a runtime value; no rule relates
                                     //   the two premises (III-10), so the acceptance is the same
                                     //   total-assignment acceptance as the main program's
let r = read(let a)                  // accept: III-11
```

**Verdict: differs from the expected verdict: deliberate refusal with repair
(O-1 / DV-4).** The difference's *cause* is identical to B07's: the read of a
possibly-`None` slot is total (III-11), so the case's REJECT has no instance;
the named repair is to write the `match` arm.

Divergence from the rules file's Appendix B, stated: Appendix B classes B07
**DR** and B09 **CC**, although line by line both verdict differences are
produced by the same rule (III-11's total read) and neither is produced by the
rule Appendix B cites for B09 (III-10). III-10 *does* record the case's stated
subject — a premise dies at an assignment to its support — but in this program
III-10 never fires, because II-5 discards the premise at the join before the
assignment is reached. So the case's subject is answered by a rule that has no
instance in the case's own program. On the derivation, B09 is a DR of the same
family as B07, with an additional note that the case's mechanism is not
reachable here.

---

## B10 — Join must not erase conditional disposal obligations

CASES.md expects: the join must **not** intersect the remaining-owner sets to
empty; each execution still has exactly one outstanding obligation.

### Binding rendering

```text
let a: Obj = make(10)                // accept: I-1 — a bound affine binding with a release
                                     //   obligation (R2), discharged by I-3 or I-4
let b: Obj = make(20)                // accept: I-1
if cond { sink a } else { sink b }   // then-arm: accept — I-4, Σ(a) := unbound, a's release runs
                                     //   here; Σ(b) = bound
                                     // else-arm: accept — I-4, Σ(b) := unbound; Σ(a) = bound
                                     // reject AT THE JOIN: II-2's definiteness clause — Σ(a) is
                                     //   unbound on the then-arm and bound on the else-arm, and
                                     //   Σ(b) likewise. Payload (II-2): the consuming statement,
                                     //   the arm that did not consume, and the repair
```

The case's demand ("the join must not erase the obligation") is met, but by
**refusing the shape** rather than by carrying a per-path obligation. The rules
state why in II-2: conjunction alone would give `Σ(a) := unbound` and then
nothing releases `a` on the arm that did not consume — an R2 leak with no
diagnostic — and since no drop flag is representable and none may be inserted
(M2(ii)), the mixed case must be a rejection.

### Repair, named by II-2

```text
let keep = if cond { sink a; b } else { sink b; a }
                                     // accept: II-2 — each arm unbinds one name and YIELDS the
                                     //   other as a VALUE; Σ(a) = Σ(b) = unbound on both arms, so
                                     //   the definiteness clause is satisfied
                                     // accept: II-3 — Γ := ∩; no fact about keep's provenance
                                     //   survives, and none is needed
sink keep                            // accept: I-4 — exactly one release on every path (R2)
```

### Pool rendering — a gap

```text
let ha = insert(inout g.slots[σ], inout g.meta, make(10))
                                     // accept: I-19 — Live(ha); Σ of the inserted value := unbound
let hb = insert(inout g.slots[σ'], inout g.meta, make(20))
                                     // accept: I-19; Live(ha) survives — σ' is fresh (III-8)
if cond { remove(inout g.slots[σ(ha)], inout g.meta, ha) }
   else  { remove(inout g.slots[σ(hb)], inout g.meta, hb) }
                                     // accept: I-20 on each arm (Live from the insert's ensures)
                                     // accept at the join: II-2 does NOT fire — `g` is bound on
                                     //   both arms and the ROWS are not bindings. II-4 intersects
                                     //   G per handle NAME: Live(ha) and Live(hb) are both gone
                                     //   (each arm killed one directly and the other by I-20's
                                     //   undecidable-σ clause)
                                     // rule missing: whether a live pool ROW carries an R2
                                     //   obligation at all, and if so what discharges it. I-19
                                     //   ends the inserted value's binding and creates no new
                                     //   binding; I-20 creates one; no rule says the pool's own
                                     //   end (I-3/I-4 on `g`) discharges the rows, nor that a row
                                     //   left live at scope exit is an R2 defect
```

This is worth recording for O7/O12 and for II-2's own standing: under the
**pool rendering the definiteness clause is bypassed entirely**, because the
conditional obligation lives in a data structure rather than in `Σ`, and no
stated rule puts it back. B10's hazard is therefore refused in one rendering
and unchecked-but-unstated in the other.

**Verdict: differs from the expected verdict: deliberate refusal with repair
(II-2's definiteness clause; repair `let keep = if cond { sink a; b } else {
sink b; a }; sink keep`), plus one rule missing (R2 status of a live pool row)
in the pool rendering.**

---

## B11 — A locator can identify the conditionally remaining obligation

CASES.md expects: **ACCEPT** for both `read(remaining)` and
`release(remaining)`.

### Binding rendering

```text
let a: Obj = make(10)                // accept: I-1
let b: Obj = make(20)                // accept: I-1
if cond {
  sink a                             // accept: I-4 — Σ(a) := unbound
  let remaining = projection_of(b)   // reject: a projection is SECOND CLASS — II-19 ("a projection
                                     //   cannot be the value of a binding"), II-16 (the name lives
                                     //   for the block's lexical span only). Missing: any storable
                                     //   form that names b's storage and outlives the arm. This is
                                     //   the model's defining boundary (DV-1, O7/O12), not a gap
} else { … }
                                     // and, with that line removed: reject AT THE JOIN — II-2, as
                                     //   B10. Two independent refusals on the same program
```

### Repair, named by II-2 and II-18

```text
let remaining = if cond { sink a; b } else { sink b; a }
                                     // accept: II-2 — each arm unbinds one name and yields the
                                     //   other as a VALUE; Σ(a) = Σ(b) = unbound on both arms
let v = read(let remaining)          // accept: I-1's binding is ordinary; the read needs no
                                     //   provenance fact and none is available (II-3)
sink remaining                       // accept: I-4 — exactly one release on every path (R2)
```

The case's own gloss — "the ordinary selected runtime pointer is enough for the
operation; the relationship to the obligation is static evidence" — survives
under the repair with the pointer replaced by the value itself: the *value*
moves out of the arm and `Σ` carries the obligation with it, so there is no
runtime selector and no bookkeeping state (M2(ii)).

### Pool rendering

```text
var remaining: Handle<P,Obj>
if cond {
  remove(inout g.slots[σ(ha)], inout g.meta, ha)
                                     // accept: I-20 — Live(ha) from the insert's ensures
                                     //   G-kill: Live(hb) is DROPPED here — I-20/III-8 keep only
                                     //   the facts of handles k with EXT-proved σ(k) != σ(ha)
  remaining = hb                     // accept: III-6 — a handle is ordinary one-word Copy data and
                                     //   may be stored, copied and rebound
} else { … remaining = ha }          // accept: symmetric
let v = g.slots[σ(remaining)].value  // reject: missing Live(remaining) — on each arm the survivor's
                                     //   liveness was killed by I-20, and III-6's assignment clause
                                     //   transfers only a fact that is HELD ("the fact is
                                     //   established for the CURRENT value of hv, FROM
                                     //   Live(other_h)"). II-4 then intersects nothing with nothing.
                                     //   Repair named by I-20: a written distinctness `requires` on
                                     //   the pool interface, or `use Links(remaining)` — but see the
                                     //   rule-missing line under P3 for whether that step follows
```

So the pool rendering, which is the one that preserves the case's *shape*
(a storable locator), loses the case for the DV-6 reason, while the binding
rendering loses it for the DV-1 reason and recovers it under the II-2 repair.

**Verdict: differs from the expected verdict: deliberate refusal with repair
(binding rendering: II-19's second-class projection plus II-2's definiteness
clause; the named repair yields the survivor as a value and restores both
ACCEPTs).** In the pool rendering the same case is a **capability loss** of the
B05/B06 family (DV-6), with no local repair.

---

## B12 — Two conditional releases separate safety from completion

CASES.md expects: ACCEPT when exactly one of `c`, `d` holds; REJECT at the
second release when both hold; and the correlated variant
`if cond { release(a) }; if !cond { release(a) }` **ACCEPT**.

```text
let a: Obj = make(10)                // accept: I-1
if c { sink a }                      // then-arm: accept — I-4, Σ(a) := unbound
                                     // else-arm (implicit): Σ(a) = bound
                                     // reject AT THE JOIN: II-2's definiteness clause. The
                                     //   rejection is at the FIRST join, before `d` is reached:
                                     //   the case's own failure point (the second release under
                                     //   c ∧ d) is never evaluated
if d { sink a }                      // not reached
```

### The case's correlated variant

```text
let a: Obj = make(10)                // accept: I-1
if cond { sink a }                   // reject AT THE JOIN: II-2, identically. The correlation
                                     //   `cond` / `!cond` that makes the variant safe is a term
                                     //   over a condition at a join, refused by II-5 and III-10
                                     //   (O11(a)); Π carries `cond` only inside the guarded region
if !cond { sink a }                  // not reached
```

### Repair, on writer-declared data

```text
var slot: Option<Obj> = Some(make(10))
                                     // accept: I-1 — formation is total (III-11: vacancy is the
                                     //   writer's discriminant, not a language state)
if c { match replace(inout slot, None) { Some(x) => sink x, None => () } }
                                     // accept: I-7 — replace is TOTAL on Option<T>; II-6 — both
                                     //   arms are written and each discharges R2 (the None arm
                                     //   releases nothing because it holds nothing, I-6)
                                     // accept at the join: II-2 — Σ(slot) = bound on both arms
if d { match replace(inout slot, None) { Some(x) => sink x, None => () } }
                                     // accept: identical. Under c ∧ d the second replace yields
                                     //   None and releases nothing
sink slot                            // accept: I-4 — the Option is affine and is consumed once on
                                     //   every path
```

Two things to record about the repair, because they are the evidence O11 turns
on:

1. It **accepts the c ∧ d case** that B12 asks to be rejected, and is safe:
   no double release occurs, because the second `replace` returns `None`. The
   case's static refusal is replaced by a **writer-declared discriminant word**.
2. M2(ii) is nevertheless satisfied, and that is the whole point: the branch
   that selects between releasing and not releasing is one the **source wrote**
   (the `match` arm), reading data the **source declared** (the `Option`
   discriminant). A checker-carried `BIND(a) = ¬c` with a lowered `if !c {
   release }` at exit would be the same branch with no source, which II-2 and
   M2(ii) refuse by name.

**Verdict: differs from the expected verdict: deliberate refusal with repair
(II-2's definiteness clause; the named repair moves the obligation into an
`Option` slot and the release decision into a written `match` arm).** Stated
cost: the repaired program *accepts* B12's `c ∧ d` row instead of rejecting it,
so the case's discrimination between "safe" and "complete" is no longer made by
the checker at all — it is made by the writer's discriminant. Together with
L05 this is the sharpest O11 discriminator in the set.

---

## B13 — Conditional scope cleanup is a policy choice with known input state

CASES.md expects: a **policy fork** — explicit-consumption policy demands the
false path discharge in source; automatic-cleanup policy releases only on the
false path, "logically `if !original_cond { release(a) }`".

```text
{
  let a: Obj = make(10)              // accept: I-1
  if cond { sink a }                 // reject AT THE JOIN: II-2's definiteness clause — Σ(a) is
                                     //   unbound on the then-arm, bound on the implicit else-arm
}                                    // not reached. Had the join accepted, I-3 would run a's
                                     //   release at this brace UNCONDITIONALLY, "with no drop flag
                                     //   (M2(ii))" — which is exactly why the join must reject:
                                     //   an unconditional edge cannot implement a conditional
                                     //   obligation, and a conditional edge is the flag M2(ii)
                                     //   forbids
```

### Both of the case's policies, under these rules

```text
{
  let a: Obj = make(10)              // accept: I-1
}                                    // accept: I-3 — automatic cleanup IS this candidate's policy
                                     //   for the unconditional case: the release runs at the
                                     //   closing brace on every path through it, no flag consulted
}
{
  var slot: Option<Obj> = Some(make(10))
                                     // accept: I-1
  if cond { match replace(inout slot, None) { Some(x) => sink x, None => () } }
                                     // accept: I-7 + II-6, as in B12; Σ(slot) = bound on both arms
}                                    // accept: I-3 — slot's affine release runs here on every path.
                                     //   On the original-true path it holds None and releases
                                     //   nothing (I-6's None-arm clause); on the false path it
                                     //   releases the object. This IS the case's
                                     //   `if !original_cond { release(a) }`, implemented on a word
                                     //   the SOURCE declared rather than on a compiler flag
```

**Verdict: differs from the expected verdict: deliberate refusal with repair
(II-2's definiteness clause removes the policy fork — the mixed case is not
representable, so neither of the case's two policies has to be chosen; the
named repair is the `Option` slot above, under which automatic cleanup at I-3
and explicit consumption coincide).** The case's closing demand — "Any future
generated cleanup must implement the selected cleanup semantics; it must not
repair an unproved access with a runtime safety test" — is met: the repaired
program's only runtime test is the writer's own discriminant read.

---

## P1 — Container split with a runtime index (regression)

Required (PROGRAMS.md): sequential legality by proof, not by borrow scope;
parallel permission for the two writes; a diagnostic at `v[k]` naming the
missing fact; no runtime check; `push` allowed only after both parts are back.

```text
fn split_write(inout v: Vec<u64>, i: u64, j: u64, k: u64, x: u64)
  requires i < len(v) && j < len(v) && i != j
                                     // accept: II-10 — one convention per parameter IS the
                                     //   footprint; the requires are ordinary value refinements.
                                     //   No region, lifetime or loan clause is written
with inout v.data[i] as pi, inout v.data[j] as pj {
                                     // accept: II-16 — both paths resolve (III-3: i and j are
                                     //   parameters, hence stable for the block); CONFLICT (II-17)
                                     //   is consulted and PATH/EXT REFUTES overlap of v.data[i]
                                     //   and v.data[j] from `i != j` in Γ. No storage is created
                                     // [G4] the block head is the noalias.scope.decl site
  pi = 1                             // accept: II-17 — inout/inout on NON-overlapping paths is
                                     //   "always permitted"; the proof, not a borrow scope, is
                                     //   what permits it (P1's first required property)
  pj = 2                             // accept: II-17, same
  par { pi = 1; pj = 2 }             // accept: II-22 ground (i) — the per-branch footprints are
                                     //   {v.data[i]} and {v.data[j]}, judged by the II-17 table
                                     //   under EXT's refutation. M5: a missing fact would leave
                                     //   the program sequential, not reject it
  let y = v.data[k]                  // reject: missing `k != i && k != j` — II-17's let/inout cell
                                     //   on paths whose overlap EXT cannot refute. Payload (M7):
                                     //   the two paths, the table cell, and the repair
  if k != i && k != j {              // accept: II-1 — the condition enters Π for this region
    let y2 = v.data[k]               // accept: II-17 — EXT now refutes overlap using the premise
                                     //   in Π (II-1's own example) together with k < len(v)
  }                                  // accept: II-5 — the premise is discarded at the join;
                                     //   III-10 — re-derived only by re-testing
  reserve(inout v.meta, 128)         // rule missing: II-17 PERMITS this line — FP = {(v.meta,
                                     //   inout)} (I-10) and v.meta is a SIBLING PLANE of v.data
                                     //   (G3/I-2), so PATH refutes overlap with both open
                                     //   projections. But `reserve` is the operation that
                                     //   REALLOCATES the anonymous backing (I-8, I-10), and pi and
                                     //   pj are open across it. I-8 justifies invisible
                                     //   reallocation by "no projection outlives the statement
                                     //   that made it (II-19)", which is true of II-19's call
                                     //   arguments and FALSE of II-16's block projections. No rule
                                     //   states whether an open projection name is RE-RESOLVED
                                     //   after its backing relocates or whether the relocating
                                     //   call is refused while it is open. I-4 and I-16 carry an
                                     //   explicit "no projection ... is open" premise; I-9, I-10,
                                     //   I-11, I-13 and I-21 carry none
  push(inout v.data[len(v)], inout v.meta, x)
                                     // rule missing, same clause: II-17 permits this too —
                                     //   EXT refutes overlap of {len(v)} with {i} from
                                     //   `i < len(v)`, and v.meta is a sibling plane. So P1's
                                     //   required ordering property ("push must be allowed only
                                     //   after both parts are back") is NOT delivered by any rule
}                                    // accept: II-18 — the projection names leave scope; "no `put
                                     //   back` is written and none is checked: nothing was taken
                                     //   out". Facts derived inside are REKEYED to v.data[i],
                                     //   v.data[j] and survive
push(inout v.data[len(v)], inout v.meta, x)
                                     // accept: I-9 — two-part footprint (P-a); no projection is
                                     //   open, so II-17 has no conflicting access to judge
                                     // accept: II-14 — learn len(v) == old(len(v)) + 1
let z = v.data[i]                    // accept: `i < len(v)` re-derived by OLDCHAIN step 4 from
                                     //   `i < old(len(v)) /\ len = old(len)+1` — ONE rewrite
```

Requirement-by-requirement: sequential legality by proof ✓ (II-16/II-17 with
`i != j` from a `requires`, never a borrow scope); parallel permission ✓
(II-22); the diagnostic at `v[k]` naming the missing fact ✓ (II-17's payload,
M7); no runtime check ✓ (M2(ii): the only branch at the `v[k]` repair is the
writer's `if`, and `v.data[i]`'s domain comes from III-2's `VecOk` plus the
`requires`, not from a compare). One capability note, not a difference from
P1's text: the two parts exist only for the block (II-16, II-19), so "take two
elements out" is a span, never two values — that is O7/O12's boundary showing
up in the program that most tolerates it.

**Verdict: accepted as expected for the written program, with one rule missing
(no rule relates an open `with` projection to a relocation of the backing it
resolves into, so P1's required ordering property "push only after both parts
are back" is not enforced and I-8's no-instance claim for R8a's interior
pointer does not cover II-16's block projections).**

---

## P3 — Graph with back edges in a pool (regression)

Required (PROGRAMS.md): link updates through several paths to one node legal
sequentially; removal invalidates every path to `m`; parallel data writes
permitted by node distinctness; the invariant "next/prev point to live nodes"
stated once and reused.

### Declaration

```text
backing Pool<P,Node> { meta : plane<PoolMeta,1>, slots : plane<Slot<Node>, len cap> }
                                     // accept: I-2 / I-18 — the plane map is the distinctness
                                     //   proof; meta and slots are disjoint with no proof (G3)
invariant PBounds : forall σ. σ ∈ Live(P) => σ < cap(p)
                                     // accept: I-18 — the bridge that makes p.slots[σ(h)]
                                     //   statically total (III-6)
invariant Links : forall σ ∈ Live(P). Live(slots[σ].next) && Live(slots[σ].prev)
                                     // accept: I-18 — "next/prev point to live nodes" written ONCE
                                     //   on the nominal and reused everywhere. P3's fourth
                                     //   required property is delivered by rule
```

### Four link writes (insert n between a and b)

```text
let hn = insert(inout p.slots[σ], inout p.meta, node)
                                     // accept: I-19 — a FRESH ghost σ; G ∪= Live(hn) from ensures
                                     // accept: Live(ha), Live(hb) SURVIVE — the footprint names
                                     //   slots[σ] and meta, and σ != σ(ha), σ != σ(hb) because σ
                                     //   is fresh (III-8, I-19's third example line)
p.slots[σ(ha)].next = hn             // accept: Live(ha) => σ(ha) ∈ Live(P) => σ(ha) < cap(p) by
                                     //   PBounds (III-6). STATICALLY TOTAL: no bounds compare and
                                     //   no generation compare is emitted (M2(ii))
p.slots[σ(hn)].prev = ha             // accept: Live(hn); II-12 — the previous write's footprint is
                                     //   {p.slots[σ(ha)].next}, which PATH/EXT does not overlap
p.slots[σ(hn)].next = hb             // accept: Live(hn)
p.slots[σ(hb)].prev = hn             // accept: Live(hb) — four link writes through several paths
                                     //   to one node, sequentially legal, with G untouched by
                                     //   element writes (G is not Γ; FRAME reads Γ only). P3's
                                     //   first required property ✓, and R9(e)'s negative test passes
```

### Removal

```text
remove(inout p.slots[σ(hm)], inout p.meta, hm)
                                     // accept: I-20 — Live(hm) is in G
                                     // effect: Live(P) \= {σ(hm)}; gen(P[σ(hm)]) := succ(γ(hm))
                                     // rule missing: a doubly linked list's `remove` MUST write
                                     //   p.slots[σ(prev)].next and p.slots[σ(next)].prev to
                                     //   re-establish `Links`, and those paths are NOT in I-20's
                                     //   declared two-part footprint {p.slots[σ(h)], p.meta}.
                                     //   II-15(a) requires the body to "write only inside its
                                     //   inout/sink footprint", so I-20 as written cannot be the
                                     //   signature of a list removal. Widening it to `inout
                                     //   p.slots` (the whole plane, as I-21's compact does) is
                                     //   permitted by II-13 but then FRAME kills EVERY element
                                     //   fact of the pool at every removal. No rule names a third
                                     //   option, and no rule states the neighbours' paths as a
                                     //   static footprint: they are `slots[σ(slots[σ(h)].prev)]`,
                                     //   a path whose index term is a load from the plane being
                                     //   written, which III-3 does not resolve for a fact's span
let d = p.slots[σ(hm)].data          // reject: missing `gen(P[σ(hm)]) = γ(hm)` — killed at the
                                     //   removal (III-7). "Removal invalidates every path to m" ✓,
                                     //   and it is invalidated STATICALLY: no runtime generation
                                     //   compare exists (M2(ii), Tier B refused by name)
let d2 = p.slots[σ(ha)].data         // reject: missing `σ(ha) != σ(hm)` — I-20's kill clause keeps
                                     //   a handle's facts only where EXT proves distinctness, and
                                     //   handles are opaque data (III-8). Every OTHER path in the
                                     //   program loses liveness at every removal. This is DV-6 and
                                     //   it is the B05/B06 capability loss appearing inside P3
use Links(ha); let d3 = p.slots[σ(ha)].data
                                     // rule missing: I-20's own example writes this step and calls
                                     //   it "INV re-derives Live(other)", but `Links` as written
                                     //   quantifies over σ ∈ Live(P) and yields liveness of the
                                     //   NEIGHBOURS of an already-live slot. Instantiating it at
                                     //   `ha` requires a premise `σ(ha) ∈ Live(P)` — the very fact
                                     //   being re-derived — or a witness handle `w` with Live(w)
                                     //   and slots[σ(w)].next == ha. INV (family 10) performs
                                     //   "first-order syntactic instantiation at the WRITTEN
                                     //   arguments, then premise discharge by REF", and REF has
                                     //   nothing to discharge that premise from. No rule supplies
                                     //   the witness
```

### Traversal

```text
var h = start                        // accept: I-1; Live(start) from the caller's fact
loop {
  let nd = p.slots[σ(h)]             // accept: II-7 — Live(h) at the head, from the entry fact
                                     //   intersected with the back edge; the body frees nothing,
                                     //   so no written liveness clause is required at the head
                                     //   ("WRITTEN at the head: nothing for memory safety")
  if nd.next == null_handle { break }
                                     // accept: the writer's own branch on the writer's own data
                                     // rule missing: `Links` as written asserts Live(slots[σ].next)
                                     //   for EVERY live σ, so a sentinel terminator is inconsistent
                                     //   with the invariant that II-7's own example relies on. No
                                     //   rule gives the terminator a form — an `Option<Handle>`
                                     //   would work under III-11, but then `Links` must be restated
                                     //   over the Some arm and no rule text does so
  use(nd.data); h = nd.next          // accept: Live(h.next) from `use Links(h)` (INV, I-18)
                                     // accept: III-3 — the fact for the NEW h is established at
                                     //   the new resolution; the old one was killed at the
                                     //   assignment (III-6's `var hv` example)
}                                    // accept: II-8 — no name is unbound on the back edge;
                                     //   II-9 — the exit join includes the zero-iteration edge
```

### Parallel map

```text
par for h in p.keys() { p.slots[σ(h)].data = f(p.slots[σ(h)].data) }
                                     // accept: III-9 + II-22 — `ensures injective(keys)` on the
                                     //   iterator's interface gives σ(h1) != σ(h2) for distinct
                                     //   iterations, an EXT fact over the slots plane. P3's third
                                     //   required property ✓, and it is delivered by a WRITTEN
                                     //   clause, never by "these handle values differ" (R4(b))
par for h in traverse(p) { p.slots[σ(h)].data = 0 }
                                     // reject: missing `distinct(σ(h1), σ(h2))` — `Links` gives
                                     //   liveness, not injectivity (III-9). Repair named: iterate
                                     //   keys(), or write and prove an injectivity clause
```

**Verdict: underivable: rule missing** — three rules, each load-bearing for P3:
(1) I-20's two-part footprint cannot cover the neighbour writes a linked-list
removal must perform, and no static path names them (II-15(a), III-3);
(2) no rule re-derives `LIVE` after a removal from `Links` without a witness
handle, so DV-6's named escape does not follow from the written invariant
(I-20's own example over-claims INV);
(3) `Links` as written admits no list terminator, which II-7's traversal
example requires.

P3's four required properties otherwise derive, at a stated cost: the four
link writes and the parallel map accept exactly as required, the removal
invalidates every path to `m` statically as required, and the invariant is
written once — but **every other handle in the program also loses its liveness
at that removal** (I-20/III-8, DV-6), which is the same capability loss as
B05/B06 and is what makes the missing re-derivation rule (2) decisive rather
than cosmetic.

---

## Summary table

| Item | CASES/PROGRAMS expects | Derived here | First deciding rule | Class |
|---|---|---|---|---|
| **B07** | REJECT (read of a possibly-empty slot) | ACCEPT (total read, returns `None`); the `write` variant accepts as expected | III-11 / I-7 | **DR** — repair: declare `Option` and write the `match` arm (DV-4) |
| **B08** | ACCEPT, by recovering the path's state | ACCEPT, with the case's mechanism corrected: the refinement `is_some(a)` is **not** recovered (II-3, II-5, III-10); the read never needed it | III-11 | **accepted as expected**; ground corrected. O11 evidence |
| **B09** | REJECT (original true path remains empty) | ACCEPT (total read); III-10, the rule the case is about, never fires because II-5 discards the premise at the join first | III-11 / II-5 | **DR** — same family as B07 (DV-4). Diverges from Appendix B's **CC** |
| **B10** | the join must not erase the obligation | REJECT at the join | II-2 | **DR** — repair: `let keep = if c { sink a; b } else { sink b; a }; sink keep`. Plus one rule missing (R2 status of a live pool row) |
| **B11** | ACCEPT read and release of `remaining` | REJECT at `remaining = ref(b)` (binding rendering), and again at the join; REJECT at the read (pool rendering) | II-19, then II-2 | **DR** with repair (binding rendering); **CL** of the B05/B06 family in the pool rendering |
| **B12** | ACCEPT when exactly one of c, d; REJECT at the second release | REJECT at the FIRST join; the case's own correlated variant also rejects | II-2 | **DR** — repair moves the decision to a written `Option`/`match`, which then **accepts** the c ∧ d row. Sharpest O11 discriminator with L05 |
| **B13** | a policy fork (explicit consumption vs automatic cleanup) | REJECT at the join; the fork has no instance | II-2 | **DR** — repair: `Option` slot; I-3's unconditional release then implements the case's `if !original_cond { release }` on writer data |
| **P1** | accept, with proof-based legality, parallel permission, a naming diagnostic, no runtime check | all four delivered; the written program accepts | II-16, II-17, II-22, M7 | **accepted as expected** + **rule missing**: relocation (I-9/I-10) while a II-16 projection is open |
| **P3** | four properties, all required | link writes ✓, static invalidation of `m` ✓, parallel map ✓, invariant written once ✓ — but three rules are missing and DV-6 costs every other handle's liveness | I-19, I-20, III-6, III-8, III-9 | **underivable: rule missing** (I-20's footprint vs the neighbour writes; INV's witness for re-deriving LIVE; `Links` vs a terminator) |

### What this part contributes to the open decisions

- **O11 (condition terms at joins).** B10, B12 and B13 all reject at a join
  under II-2, and B08's residual refinement is lost under II-3/II-5, for one
  reason: a fact indexed by a branch condition is refused. B12's repair shows
  the price exactly — the discrimination the case wanted from the checker is
  re-created as a writer-declared discriminant word, and the c ∧ d row flips
  from REJECT to a safe ACCEPT. B13 shows the benefit exactly — I-3's release
  edge is unconditional, so no drop flag is representable and M2(ii) holds by
  construction. The two are the same rule seen from both ends.
- **O7 / O12 (second-class projections, references never stored).** B11 is the
  case where the boundary bites in this part: the locator the case names is
  unwritable (II-19), and the repair is available only because the survivor can
  be yielded as a *value*. Where the object cannot be yielded as a value — the
  pool rendering — the case is a capability loss with no local repair, and P3
  shows the same loss inside a required program. P1 shows the boundary being
  paid without complaint, and also shows the one place where the rules have not
  finished stating it: a projection that outlives a statement (II-16) meets a
  relocating call with no rule between them.


# File: derive-window-focus-boundary.md

# Derivation: `window-focus`, part **boundary** — P4, P7, P8, L01–L06

Candidate: `window-focus`, rules in [rules-window-focus.md](rules-window-focus.md)
(rule sets I, II, III; judgment families §5). Round: core2. Research date: 2026-09-16.
Style: MECHANISM-MAP §3 — a pseudocode line, then `// accept: fact used` or
`// reject: missing fact`. Every line cites the rule it uses.

Sources of the items: P4, P7, P8 from
[PROGRAMS.md](../../../../private/tmp/whitefoot-access-effects-research/research/investigations/access-effects/PROGRAMS.md)
(`research/investigations/access-effects/PROGRAMS.md`); L01–L06 from the same
directory's `CASES.md`. Expected verdicts for L01–L06 were written under
DESIGN.md Candidate A; where this candidate's rules give a different verdict the
difference is classified. Nothing here selects a candidate.

Two conventions used throughout:

- **Rule missing** marks a line the candidate's rules do not cover. No rule is
  improvised for it; the derivation continues on the next line.
- A rule's own pseudocode example is treated as part of the rule (the file states
  one per rule, and §3.21 derives the CASES set from them). Where a derivation
  leans on an example that the rule's *prose* does not cover, that is said in place.

---

## P4 Cursor over a growing vector

Program text (PROGRAMS.md §P4). Required: a stored pointer or handle to a
container in a struct; validity decided by state and proof, not by a borrow that
blocks `push`.

### Setup and the stored pointer

```text
struct Cursor<β> { at: ptr<β, u64>, i: u64 }
                               // accept: §1.3 — the store name is a struct parameter; III4 gives the
                               //   element pointer the COARSE backing name β = backing_of(ρv), which
                               //   II19's first-class row admits in a field. ESC does not apply:
                               //   ESC governs ptr<ρ↓w.k,T> only, and no window is open (II19)
(v, own_v) = vec_new(n)        // accept: I1 mints ρv; III2 ensures len_of(ρv)=n, III3 cap_of(ρv),
                               //   III4 ensures backing_of(ρv)=β with extent_of(β)=[0,cap)
                               //   and β ⊑ ρv; St(β[0..n]) = Init under the Prefix run (III5)
c = Cursor { at: elem_ptr(v,0), i: 0 }
                               // accept: III4 — pe : ptr<β,u64>, "storable and returnable (P4's Cursor;
                               //   no window needed)". No Span entry is created, so I7/I17 are not blocked
```

### The first read

```text
a = c.read()                   // accept: St(β[c.i]) = Init by one INV step against the Prefix
                               //   invariant (III5) given c.i < len_of(ρv) (III2); R11's index domain
                               //   is discharged from the same len fact (MEAS, §5)
```

### The push, with no capacity proof

```text
push(v, 5)                     // accept: I17's premises hold — St(ρv)=Init, Carve(β)=∅, Span(β)=∅.
                               //   Neither cap_of(ρv) > len_of(ρv) nor its negation is discharged, so
                               //   I17's third case binds a fresh atom g# (III12, II12) and the
                               //   post-state is St(β)=ite(g#, Init, Gone), backing_of(ρv)=ite(g#,β,β')
                               //   M2(ii) check: g# is a checker atom; II6(b) forbids any lowered
                               //   branch or bookkeeping cell selected by it
b = c.read()                   // reject: missing fact St(β)=Init on the ¬g# arm (I17 arm B; II12).
                               //   M7 payload: rule I17, the push site, missing fact
                               //   "cap_of(v) > len_of(v)"; repairs "prove capacity before the push
                               //   (e.g. reserve)" or "re-form the cursor from the container" (III4)
```

### The push, with the capacity proof written

```text
reserve(v, len_of(v)+1)        // accept: III3 — ensures cap_of(ρv) >= len_of(ρv)+1, keyed by ρv.desc
push(v, 5)                     // accept: I17 arm A selected AT THE CALL (no atom is introduced);
                               //   I16 leaves backing_of(ρv) = β unchanged, so no element identity
                               //   changes; ensures len_of(ρv) = old+1 (III2)
b = c.read()                   // accept: St(β)=Init unguarded (I16/III3); c.i < len_of(ρv) still holds
                               //   because len only grew (III2's ensures). This is the required
                               //   "carry still-valid across push", carried by proof, not by a borrow
```

`push` was never blocked by the stored pointer: I17's premises name `Carve` and
`Span`, and a coarse `ptr<β,u64>` is in neither (II19, III4). That is P4's
"validity decided by state and proof, not by a borrow that blocks push".

### The free

```text
free(v)                        // accept: I7 with III4's row
                               //   row { ρv : Init ⇒ Gone ; backing_of(ρv) : Init ⇒ Gone ; end ρv, end β }
                               //   — both names required, since β ⊑ ρv and I7 refuses a live ⊑-child.
                               //   Carve(β)=∅, Span(β)=∅, own_v unconsumed. The live Cursor field does
                               //   not appear in I7's premises, so it does not block the end
c.read()                       // reject: St(β)=Gone (I7's effect, permanent; I23 — address reuse mints
                               //   a fresh name and never revives β). R1(a)(ii); M7 names the free site
```

The field `c.at` still names `β` after the end, and after I17 arm B it names a
`Gone` backing; III4 states there is no re-derivation of a `Gone` backing and the
writer re-forms the pointer from the container. No rule retypes the field, and
none is needed: every access rejects on the state.

**Verdict: accepted as expected, with a stated cost** — one written capacity
proof (`reserve`, or one `INV` step against a written invariant) before the push
buys the second read; without it the second read is a rejection with a named
repair, which is what P4's "legal iff the candidate can carry still-valid across
push" asks for.

---

## P7 Take, put, replace through aliases

Program text (PROGRAMS.md §P7). Required: state is a property of the storage,
not of the pointer; take leaves a hole another alias may fill; free is permanent.
The slot holds an **affine** value (P7's own wording), so I4/I5/I6's class
premises apply as written.

```text
(a, own_a) = new(T); put(a, move val0)
                               // accept: I1 mints ρa, St(ρa)=Uninit, one end-obligation;
                               //   I6 sinks val0, St(ρa) := Init, the value's obligation moves to the place
p = ptr_of(a); q = p           // accept: II19 first-class row — ptr<ρa,T> is copyable; both bindings
                               //   resolve to the same PLACE ρa (§1.1), so facts key on ρa, not on p or q
v = take(p)                    // accept: I4 — St(ρa)=Init, class affine; St(ρa) := Uninit; the obligation
                               //   moves to v; KILL drops contents facts supported by ρa (II11)
read(q)                        // reject: missing fact St(ρa)=Init — I4's own example line: "the hole is on
                               //   the STORAGE". R1(a)(i); M7 names the take site and the sub-property
put(q, move v)                 // accept: I6 — St(ρa)=Uninit, v holds the obligation; St(ρa) := Init;
                               //   the obligation transfers to the place; v is consumed
read(p)                        // accept: St(ρa)=Init — the fact was restored on the place, through the
                               //   OTHER locator (I6). This is P7's "state is a property of the storage"
old = replace(p, new)          // accept: I5 — St(ρa)=Init; one event Init ⇒ Init, no intermediate hole;
                               //   the old value's obligation moves to the binding `old`;
                               //   KILL contents facts supported by ρa; ensures contents(ρa) = new (III1)
read(q)                        // accept: reads `new` — the contents fact was killed and re-ensured by I5
drop(old)                      // accept: R3/I5's note — without this line the enclosing scope rejects at I8
                               //   (an undischarged affine value). COST: one writer line P7 does not show
v2 = take(p); drop(v2)         // accept: I4 then R3 — I7 requires "every obligation held by a value inside
                               //   ρa is discharged", and `new` is inside ρa. COST: one writer line.
                               //   M2(ii) forbids the alternative (a compiler-inserted release at the free)
free(a)                        // accept: I7 — own_a live and unconsumed, Carve(ρa)=∅, Span(ρa)=∅, no live
                               //   ⊑-child, no contained obligation, St(ρa)=Uninit ∈ {Init,Uninit} (S05).
                               //   Effect: St(ρa) := Gone permanently; own_a consumed
read(p)                        // reject: St(ρa)=Gone (I7's effect; I23 — a later formation over the same
                               //   bytes mints a fresh name and never revives ρa). R1(a)(ii)
free(a)                        // reject: the affine owner own_a was consumed at the previous free (R3/R2,
                               //   I7's example line) — distinguishable from the access rejection above (M7)
```

Every required property of P7 is delivered by the place scheme alone: no window
is opened, no fine name appears, and `p`/`q` are ordinary first-class pointers.

**Verdict: accepted as expected, with a stated cost** — two writer lines the
program text does not show (`drop(old)` after the replace, and emptying the slot
before the free), both forced by R2 rather than by this candidate's own
machinery; M2(ii) forbids the candidate to insert either.

---

## P8 Conditional release and loop exits

Program text (PROGRAMS.md §P8). Required: no runtime drop flag; the writer's own
branches carry the state; rejection names the path whose state differs.

### P8(a) — `if c { free(a) } else { free(b) }`, use the survivor, free it

```text
(a, own_a) = new(10); (b, own_b) = new(20)
                               // accept: I1 twice — ρa, ρb are distinct ⊑-roots, both Init after I3,
                               //   two end-obligations
let c = check()                // accept: II1/III12 — c is a boolean BINDING, so atom(c,1) is captured;
                               //   an expression discriminant would capture nothing (II1's first example)
if c { free(a) } else { free(b) }
                               // accept: I7 on each arm under II2 (each write hits only that arm's leaf);
                               //   II3 joins: St(ρa)=ite(c,Gone,Init), St(ρb)=ite(c,Init,Gone),
                               //   guard depth 1 ≤ Gmax. The obligations are retained, guarded (II3; B10)
read(a)                        // reject: missing fact St(ρa)=Init on the c assignment (II3's example).
                               //   M7 names BOTH edges — P8's "rejection names the path whose state differs"
if c { read(b) } else { read(a) }
                               // accept: II2 — each arm resolves its own leaf. §3.21 records this as
                               //   "P8(a) accepted verbatim under II3"; no drop flag, by II6
if c { free(b) } else { free(a) }
                               // accept: I7 on each arm; II6(a) feasibility — on every feasible assignment
                               //   of {c} exactly one end happens per object and none is ended twice (R2)
```

Survivor written as a binding instead of re-branching:

```text
if c { s = ptr_of(b) } else { s = ptr_of(a) }
                               // accept: II2 — s's identity is the guarded identity ite(c, ρb, ρa) (§1.1)
read(s)                        // accept: under each feasible assignment s resolves to the leaf that is
                               //   Init (II2's example, whose `write(p,9)`/`read(q)` pair is the same shape)
free(s)                        // accept: II2 + I7, the disposition §3.21 records for B05 ("accept both
                               //   reads and both ends"). NOTE: II2's PROSE scopes leaf resolution to
                               //   "inside the arm"; the outside-the-arm case is fixed only by II2's
                               //   example and by §3.21's B05 row, not by the rule text
```

M2(ii) check: no operation in the accepted spellings is selected by an atom —
every branch executed is one the writer wrote (II6(b)). No drop flag exists to
insert, and none is needed, because II3 keeps the difference as a *fact*.

### P8(b) — `loop { if stop { break }; take(p); …; put(p) }; free(p)`

```text
(a, own_a) = new(10); p = ptr_of(a)
                               // accept: I1 + I3; II19 first-class pointer
repeat n times {
  if stop { break }            // accept: II9 — the break edge is NOT required to satisfy the head row;
                               //   it carries its own state (St(ρa)=Init here) to the exit join
  v = take(p)                  // accept: I4 — St(ρa) Init ⇒ Uninit, obligation to v
  …
  put(p, move v)               // accept: I6 — St(ρa) Uninit ⇒ Init, v consumed
}                              // accept: II7 default head row (the identity) — the body's composed state
                               //   edge IS the identity, no name is minted (I22), no guard must cross
                               //   (II8 retires `stop` at the back edge). LOOPCHK: initiation from the
                               //   entry edge (Init), consecution from body ∘ head = identity. No fixpoint
free(p)                        // accept: II9 — the exit join is taken over every exit edge (zero-trip,
                               //   normal completion, break); each has St(ρa)=Init and own_a unconsumed,
                               //   so I7's premises hold on every one (II9's own example shape)
```

**Verdict: accepted as expected.** Both halves are delivered, the unconditional
use of the survivor is the rejection P8 asks for, and the "no drop flag" property
is a rule (II6(b)), not a claim.

---

## L01 Take and restoration preserve the next iteration's entry condition

CASES L01 expects ACCEPT, including `n = 0`.

```text
a = object(10)                 // accept: I1 mints ρa; I3 writes the copy scalar ⇒ St(ρa)=Init; one
                               //   end-obligation (I1's owner binding)
p = ref(a); q = p              // accept: II19 first-class row — ptr<ρa,u64> copyable; p and q resolve to
                               //   the same place ρa (§1.1), so no relation between the two names is needed
repeat n times {
  v = take(p)                  // accept: I4 — St(ρa) Init ⇒ Uninit.
                               //   RULE-TEXT NOTE: I4's premise names class "affine or linear" while the
                               //   CASES slot holds a copy integer. The rule file applies I4 to exactly
                               //   this shape in II7's L01 example and in I3's B07 example; the derivation
                               //   follows the file's own application. See "Rule-text gaps" below
  put(q, move v)               // accept: I6 — St(ρa) Uninit ⇒ Init through the OTHER locator; v consumed
}                              // accept: II7 default head row — body ∘ head = identity, so no head row is
                               //   written (II7's first example, "CASES L01"); II8 retires nothing (no
                               //   atom captured), I22 has no minted name to police
read(p)                        // accept: St(ρa)=Init on every exit edge — the zero-trip edge carries the
                               //   entry state, the normal edge carries the identity (II7/II9). R1(a)(i)
```

**Verdict: accepted as expected.** Zero cost: no head row, no proof step, no
window. Matches §3.21's L01 row.

---

## L02 An empty exit state becomes the next iteration's input

CASES L02 expects REJECT for unrestricted `n`.

```text
a = object(10)                 // accept: I1 + I3 ⇒ St(ρa)=Init
p = ref(a)                     // accept: II19
repeat n times {
  v = take(p)                  // accept as a statement: I4 — St(ρa) Init ⇒ Uninit
}                              // reject: II7 consecution — the (default) head row asserts St(ρa)=Init and
                               //   the body's composed edge exits at Uninit. LOOPCHK's second entailment
                               //   fails; M7 names the head state and the body's exit state (II7's second
                               //   example, "CASES L02"). No fixpoint and no iteration count is consulted,
                               //   so the verdict does not depend on n (M1)
```

Two sub-lines CASES raises:

```text
// n statically known ≤ 1     // rule missing: no rule of sets I–III introduces a trip-count fact.
                              //   II7's entailments are stated over the head row alone and §5's LOOPCHK
                              //   takes no count; `repeat n times` contributes no measure to Facts (III1's
                              //   introducers are formations, row ensures, and INV). The n ≤ 1 variant is
                              //   therefore still rejected at the back edge, with no rule to reason about it
read(p)  // after the loop    // not reached: the program already rejects at the back edge
```

The fragment leaves ρa's end-obligation pending; CASES states fragments omit
unrelated cleanup, and I8 would demand it at the enclosing scope exit.

**Verdict: accepted as expected** (reject at the back edge, with the head and
body states named). One rule missing for the `n ≤ 1` remark, which CASES states
as a semantic observation rather than as an expected verdict.

---

## L03 Relative initialization invariant while physical roles alternate

CASES L03 expects ACCEPT with `read(p)` reading 10; post-loop `read(q)` and
unconditional `read(a)`/`read(b)` reject; `release(p); release(q)` accepts.

```text
a = object(10); b = object(20) // accept: I1 twice ⇒ ρa, ρb distinct ⊑-roots, both Init, two obligations
old_b = take(b)                // accept: I4 — St(ρb) Init ⇒ Uninit (same class note as L01)
p = ref(a); q = ref(b)         // accept: II19 first-class pointers at ρa and ρb
head row { exists i1 i2. target(p)=i1, target(q)=i2,
           St(i1)=Init, St(i2)=Uninit, dis(i1,i2) }
                               // accept: II7(a) and II7(d) — the body's composed edge is not the identity
                               //   and the head's identities are existential, so the writer writes the row.
                               //   This is II7's third example verbatim ("CASES L03"). COST: one head row
                               // initiation: target(p)=ρa Init, target(q)=ρb Uninit, dis(ρa,ρb) by §5's DIS
                               //   NOTE: DIS's base case for two distinct ⊑-roots is not spelled out; §5
                               //   declares DIS total and I11 confines the "never by name inequality" ban
                               //   to ⊑-siblings, so the root case reads as true. See "Rule-text gaps"
repeat n times {
  v = take(p)                  // accept: I4 on the place target(p)=i1, St(i1)=Init from the head row
  put(q, move v)               // accept: I6 on i2, St(i2)=Uninit ⇒ Init; dis(i1,i2) keeps the two keys apart
  tmp = p; p = q; q = tmp      // accept: ordinary rebinding of first-class pointer bindings; facts key on
                               //   identities, not on binding names (§3.21's B06 ground)
}                              // accept: II7 consecution — "the binders are matched POSITIONALLY by the
                               //   binding each is attached to (p, q) — a determined match, no search;
                               //   the swap maps the relation to itself" (II7's third example). M1 holds:
                               //   LOOPCHK is two JOIN-refinement comparisons, no widening
read(p)                        // accept: St(target(p))=Init at every exit, including the zero-trip edge,
                               //   from the head row (II7's example line; II9's exit join)
read(q)                        // reject: missing fact St(target(q))=Init — the head row gives Uninit (R1(a)(i))
read(a)                        // reject: missing fact St(ρa)=Init — the head row is existential, and no rule
                               //   resolves i1 to ρa at the exit. M7 names the head row's binder
release(p); release(q)         // rule missing: I7's premise is `Own(x) = ρ` and §1.1's identity grammar
                               //   ι ::= ρ | π | e | {ι,…} | ite(c,ι,ι) has NO existential-binder form, so
                               //   an obligation reached through the head row's i1/i2 has no stated identity
                               //   to key on. II7(d) admits existential head rows; nothing states how an
                               //   obligation or an end event is carried on one past the loop exit.
                               //   CASES expects both releases to accept; that verdict is not derivable here
```

**Verdict: accepted with a stated cost (one written loop-head row)** for the
loop and the post-loop reads, matching §3.21's L03 row. The final
`release(p); release(q)` line of CASES L03 is **underivable: rule missing** — the
existential head-row binder is outside §1.1's identity grammar, so I7 has no
identity to consume.

---

## L04 A release on a break edge need not preserve the loop-head state

CASES L04 expects ACCEPT of the in-body `read(p)`; unconditional post-loop
`read(p)` or `release(p)` reject.

```text
a = object(10); p = ref(a)     // accept: I1 + I3 ⇒ St(ρa)=Init, one obligation; II19
repeat n times {
  if stop { release(p)         // accept: II1 captures atom(stop) when `stop` is a binding (an expression
                               //   discriminant captures none, II1, and the arm reasoning below is the same);
                               //   I7 inside the arm — St(ρa)=Init, Carve=∅, Span=∅, no ⊑-child, owner
                               //   unconsumed ⇒ St(ρa) := Gone on that arm's leaf (II2)
           break }             // accept: II9 — the break edge is not required to satisfy the head row; it
                               //   carries St(ρa)=Gone and the consumed owner to the loop's exit join
  read(p)                      // accept: the release path cannot reach this line; on the reaching edge the
                               //   ¬stop leaf has St(ρa)=Init (II2). II9's first example, "CASES L04"
}                              // accept: II7 consecution — the only back edge is the ¬stop path, whose
                               //   composed edge is the identity on St(ρa); the default head row holds.
                               //   II8 retires atom(stop) at the back edge (it is captured in the body)
read(p)                        // reject: missing fact St(ρa)=Init — the exit join's break edge carries Gone
                               //   (II9's example line). R1(a)(ii); M7 names the release site and the edge
release(p)                     // reject: I7's premises fail on the break edge — own_a is already consumed
                               //   (R3) and St(ρa)=Gone. Distinguishable from the access rejection (M7)
```

Pending-obligation note: on the zero-trip and normal-completion edges the
obligation is still live, so the enclosing scope exit demands a discharge (I8).
II6(b) refuses a guard-selected cleanup, so the writer writes L05's shape; CASES
routes this to B13 and the rule file records it as a deliberate refusal there
(§3.21, B13). Not part of L04's verdict.

**Verdict: accepted as expected.**

Rule-text gap used by this item: II9 says the exit join "is II3 over all exit
edges", but II3's merge is **indexed by an atom** and a loop exit has none (the
break edge's atom was retired at the back edge by II8). II9's own two examples
quantify the premise **per exit edge**, which is the reading used above and the
one that gives CASES' verdicts; the merged-value form at an atom-free join is
unstated.

---

## L05 A checked Boolean relation guards later iterations after release

CASES L05 expects ACCEPT with exactly one completed release, including the
zero-iteration case.

```text
a = object(10); p = ref(a)     // accept: I1 + I3 ⇒ St(ρa)=Init, one obligation
active = true                  // accept: II5/III12 — a boolean binding; atom(active@1) with the relation
                               //   `const true` recorded (II5 records `=`, `¬` and literals)
head row { ρa : ite(active, Init, Gone) }
                               // accept: II7(c) — a fact must survive the back edge under a guard, so the
                               //   writer writes it; II8: "the guard crosses because it is WRITTEN at the
                               //   head". COST: one head row. Initiation: active@1 = true and St(ρa)=Init
                               //   satisfy ite(active, Init, Gone)
repeat n times {
  if active {                  // accept: II2 arm entry — inside, St(ρa) resolves to the Init leaf
    read(p)                    // accept: St(ρa)=Init on the active leaf (II2). R1(a)(i) discharged
    if stop {                  // accept: II1 captures atom(stop); the cube is active ∧ stop
      release(p)               // accept: I7 on that leaf — St(ρa)=Init, owner unconsumed ⇒ Gone
      active = false           // accept: II5 — version active@2 with the relation `const false` recorded
                               //   on this arm; guarded values mentioning active@1 keep that version
    } } }                      // accept: II8 consecution, leaf by leaf —
                               //   ¬active: body skipped, St stays Gone, active stays false ⇒ row holds
                               //   active ∧ ¬stop: read only, St=Init, active true ⇒ row holds
                               //   active ∧ stop: Gone with active false ⇒ row holds
                               //   atom(stop) retires at the back edge (II8); `active` crosses because the
                               //   head row names it. Depth 1 ≤ Gmax. This is II8's example, "CASES L05"
if active { release(p) }       // accept: II9's exit join over the zero-trip and normal-completion edges,
                               //   both satisfying the head row; inside the arm St(ρa)=Init and the owner
                               //   is unconsumed ⇒ I7. II6(a): on every feasible assignment exactly one end
                               //   happens — the ¬active leaf ended it in the loop (R2 discharged)
```

Variants CASES names:

```text
// omit `active = false`       // reject at the back edge: consecution fails — the active leaf would carry
                               //   St(ρa)=Gone against the head row's Init (II7/LOOPCHK); M7 names the head
                               //   row and the body's exit state
// read(p) between release and `active = false`
                               // reject: St(ρa)=Gone at that point (I7's effect). The head relation is not
                               //   a timeless fact — II8's own note. R1(a)(ii)
```

M2(ii) check: `active` is the writer's own source boolean and every branch on it
is source-written; II6(b) inserts nothing and selects nothing. The checker does
not synthesize the flag, which is CASES L05's own requirement.

**Verdict: accepted with a stated cost (one written loop-head row carrying the
guard, II7(c)/II8).** Matches §3.21's L05 row.

---

## L06 A hole may leave through break when the continuation only releases it

CASES L06 expects ACCEPT of `release(p)`; an unconditional `read(p)` before it
rejects.

```text
a = object(10); p = ref(a)     // accept: I1 + I3 ⇒ St(ρa)=Init, one obligation
repeat n times {
  v = take(p)                  // accept: I4 — St(ρa) Init ⇒ Uninit (same class note as L01)
  if stop { break }            // accept: II9 — the break edge carries St(ρa)=Uninit to the exit join and
                               //   is not required to satisfy the head row
                               // NOTE: under I4's stated class premise the obligation moved to v, so this
                               //   edge would leave v unconsumed and I8 would reject (R2). CASES fixes v as
                               //   a discardable copy integer, and the rule file's own L01/B07 examples
                               //   apply I4 at that class; under that reading there is nothing to discharge
  put(p, move v)               // accept: I6 — St(ρa) Uninit ⇒ Init; v consumed
}                              // accept: II7 default head row — the normal path's composed edge is the
                               //   identity (take then put); no name minted (I22); atom(stop) retires (II8)
release(p)                     // accept: I7's premise St(ρ) ∈ {Init, Uninit} holds on EVERY exit edge —
                               //   zero-trip Init, normal completion Init, break Uninit. II9's second
                               //   example, "CASES L06". No hole-filling is required before the end (S05)
```

Variant CASES names:

```text
read(p)   // before release    // reject: missing fact St(ρa)=Init on the break edge (II9's exit join,
                               //   R1(a)(i)). M7 names the break edge, the path whose state differs
```

**Verdict: accepted as expected.** Uses the same atom-free exit-join reading as
L04 (see the gap recorded there), and the same I4 class note as L01.

---

## Rule-text gaps found while deriving

Recorded as gaps, not repaired here; no rule was improvised for any of them.

| # | Where | What the rules do not cover |
|---|---|---|
| G1 | L01, L02, L03, L06 (`take` on a CASES slot) | I4's premise names class "affine or linear"; the CASES slots hold copy integers. The rule file applies I4 to exactly these shapes in II7's L01 example and I3's B07 example, so the intent is clear, but the rule as stated does not admit the line |
| G2 | L02 (`n ≤ 1` variant) | No rule introduces a trip-count fact. III1's introducers are formations, row `ensures` and `INV`; `repeat n times` contributes nothing, and LOOPCHK consults no count. CASES' "if n is known to be at most one" has no vocabulary here |
| G3 | L03 (`release(p); release(q)`) | II7(d) admits existential head-row identities, but §1.1's identity grammar `ι ::= ρ \| π \| e \| {ι,…} \| ite(c,ι,ι)` has no binder form, and I7's premise is `Own(x) = ρ`. Ending an obligation reached through an existential binder is unstated |
| G4 | L03 (initiation's `dis(i1,i2)`) | DIS's base case for two distinct `⊑`-roots is not stated. §5 declares DIS total and I11 confines the name-inequality ban to `⊑`-siblings, so the root case reads as true — but it is read, not written |
| G5 | L04, L06 (the loop exit join) | II9 delegates the exit join to II3, whose merge is indexed by a guard atom; a loop exit has none, since II8 retires body atoms at the back edge. II9's own examples quantify the premise per exit edge; the merged-value form at an atom-free join is unstated |
| G6 | P8(a) (`free(s)` on a bound survivor) | II2's prose scopes leaf resolution to "inside the arm". Writes and ends through a guarded identity *outside* any arm are fixed only by II2's example and §3.21's B05 row |

None of these gaps changes an item's verdict except G3, which makes one line of
L03 underivable.

## M1 and M2 as exercised by these nine items

- **M1**: every acceptance above is one syntax-directed pass (`STATE`) plus, at
  the three loops, LOOPCHK's two entailments. No fixpoint, no widening, no
  iteration count, no budget, no order dependence. The `n ≤ 1` rejection in L02 is
  the *same* rejection at every `n`, which is M1 working, not a precision loss.
- **M2(ii)**: the only checker-introduced runtime-shaped object anywhere in these
  items is P4's atom `g#` at an unproved `push`, and II6(b) forbids any lowered
  branch, release or cell selected by it. P8's "no drop flag" and L05's "the
  checker does not insert a liveness test" are delivered by rule (II6(b)), not by
  argument. No item required a check the source did not write.

## Summary table

| Item | Rules used | Verdict here | Against the expected verdict | Cost / notes |
|---|---|---|---|---|
| P4 | I1, I16, I17, I23, I7, II12, II19, III2, III3, III4, III5, III12 | first read accept; push accept; second read reject without a capacity proof, accept with one; `free(v); c.read()` reject | accepted as expected | one written `reserve` or `INV` step; the Cursor's stored pointer is coarse (III4), so it blocks neither `push` nor `free` |
| P7 | I1, I4, I5, I6, I7, I23, II11, II19, III1 | every required line as P7 states it | accepted as expected | two writer lines P7 omits: `drop(old)`, and emptying the slot before `free` (R2; M2(ii) forbids inserting them) |
| P8 | I1, I3, I4, I6, I7, II1, II2, II3, II6, II7, II8, II9, III12 | (a) accept verbatim, unconditional survivor use rejects naming both edges; (b) accept, `free(p)` accepts at the exit join | accepted as expected | none. "No drop flag" is II6(b), a rule |
| L01 | I1, I3, I4, I6, II7, II9, II19 | accept, including `n = 0` | accepted as expected | zero: no head row, no proof step (G1) |
| L02 | I4, II7, LOOPCHK | reject at the back edge, head and body states named | accepted as expected | G2: the `n ≤ 1` remark has no vocabulary |
| L03 | I1, I4, I6, II7(a)(d), II9, §5 DIS/LOOPCHK | loop and post-loop reads accept; `read(q)`, `read(a)` reject; the two releases underivable | accepted with a stated cost; one line underivable (G3) | one written existential head row; G3, G4 |
| L04 | I7, II1, II2, II7, II8, II9 | in-body `read(p)` accepts; post-loop `read(p)` and `release(p)` reject | accepted as expected | G5 (atom-free exit join, fixed by II9's examples) |
| L05 | I7, II1, II2, II5, II6(a), II7(c), II8, II9, III12 | accept with exactly one release, zero-trip included; both CASES variants reject | accepted with a stated cost | one written loop-head row carrying the guard |
| L06 | I4, I6, I7, II7, II8, II9 | `release(p)` accepts over Init/Init/Uninit exits; the inserted `read(p)` rejects | accepted as expected | G1, G5 |

Evidence this part carries for the owner's decisions:

- **O7/O12**: P4 and P7 are written with **no** fine name and no window —
  III4's coarse `backing_of` and §1.1's places carry a stored container pointer
  and cross-alias state, so "projections are second class, references are never
  stored" costs nothing on these two boundary programs. The boundary bites only
  where II19's `ESC` applies (element-level projections, OD7), which none of
  P4, P7, P8 or L01–L06 needs.
- **O11**: P8(a) and L05 are decided by guarded states over captured condition
  terms at joins and at a loop head (II3, II5, II8). Without condition terms at
  joins, P8(a) has no accepted spelling and L05's head row cannot be written;
  Gmax depth never exceeded 1 in any of these nine items.


# File: derive-window-focus-straight-and-early-branches.md

# Derivation: `window-focus` — part `straight-and-early-branches` (S01–S07, B01–B06)

Candidate rules: [rules-window-focus.md](rules-window-focus.md) (rule sets I, II, III; §5
judgment families). Cases: CASES.md §"Straight-line cases" and §"Branch cases", written
under DESIGN.md Candidate A. Ladder style: MECHANISM-MAP §3 — a pseudocode line, then
`// accept: fact used` or `// reject: missing fact`, with the rule number on every line.

Nothing here selects a candidate. Where a line is not covered by a written rule it is
marked `rule missing` and the derivation continues on the reading the candidate's own
examples use; every such line is collected in §3.

---

## 0. Conventions: CASES operations under this rule set

The cases use a toy vocabulary. Each operation is mapped to exactly one rule, once, here;
the derivations then cite the rule.

| CASES operation | Rule | Reading |
|---|---|---|
| `a = object(10)` | **I1** then **I3** | I1 mints a fresh coarse `ρa` as a `⊑`-root, `St(ρa) := Uninit`, and mints one affine owner binding with one end-obligation; the literal initializer is I3's total copy-scalar store, `Uninit ⇒ Init`, `ensures contents(ρa)=10`. (I2 is the alternative reading; the cases record an explicit disposal obligation and an explicit `release`, which is I1's shape, not I2's scope-owned one.) |
| `p = ref(a)` | **II19** type grammar | `p : ptr<ρa, int>`, the **first-class** form: copyable, storable, returnable. No numbered rule introduces a pointer; II19's grammar gives the type and I3/II5/II7's own examples use `p = ref(a)` verbatim. |
| `q = p` | **II2** | "q copies the guarded identity, not the variable name." Here the identity is unguarded, which II2 covers a fortiori: a binding carries an *identity*, and every later fact keys on that identity, never on the variable name. |
| `read(p)` | premise only | Requires `St(place(p)) = Init` on every feasible guard assignment (II6(a)); it writes nothing. |
| `write(p, n)` | **I3** | Total copy-scalar store, admissible from `Uninit` **and** `Init`. |
| `old = replace(p, n)` | **I5** | One `Init ⇒ Init` event, no intermediate hole. |
| `v = take(p)` | **I4** | `Init ⇒ Uninit`; the obligation moves to the taken binding. See gap **G1**. |
| `put(p, v)` | **I6** | Sink into a destination whose state is `Uninit`. |
| `release(a)` / `release(p)` | **I7** | `St := Gone` permanently for `ρ` and every `St`-key under it; the owner obligation is consumed. See gap **G3** for the locator form. |
| `if c { … } else { … }` | **II1**, **II2**, **II3** | `c` is a boolean *binding*, so an atom is captured; each arm writes only its own leaf; the join merges every key to `ite(c, then, else)`, canonical, with `ite(c,x,x) → x`. |

Two standing facts used throughout:

- **Feasibility (II6(a)).** An obligation must hold on *every feasible assignment* of the
  live atoms. With one atom and no recorded relation, both assignments are feasible.
- **No written scope exit.** The fragments end without leaving a scope, so **I8**
  contributes no edge and the undischarged owner obligations in S01–S04, S06, B01–B04 are
  never checked. Were a scope exit written, I8 would reject there. That is CASES' own
  "fragments omit unrelated cleanup", not a verdict difference; it is not repeated below.

---

## 1. Straight-line cases

### S01 — Sequential aliases and stable copies

```text
a = object(10)     // accept: I1 mints ρa (a ⊑-root), St(ρa)=Uninit, one owner obligation;
                   //   I3 stores the literal: St(ρa):=Init, Facts += contents(ρa)=10
b = object(20)     // accept: I1 mints ρb, a SECOND ⊑-root (no ⊑ edge to ρa); I3: St(ρb)=Init, contents=20
p = ref(a)         // accept: p : ptr<ρa,int>, first class (II19's type grammar); identity(p)=ρa
q = p              // accept: II2 — q copies the IDENTITY ρa; the copy is not a fact about the name `p`
write(p, 11)       // accept: I3 — type is copy, St(ρa)=Init ∈ {Uninit,Init}; II15 is vacuous (Carve=∅).
                   //   Effect: St(ρa):=Init; KILL contents(ρa)=10 (its support is not dis from ρa);
                   //   ensures contents(ρa)=11
read(q)            // accept: identity(q)=ρa, fact St(ρa)=Init; contents(ρa)=11  ⇒ 11
p = ref(b)         // accept: an ordinary rebinding — identity(p):=ρb. q is untouched: II2 keys q on the
                   //   identity it copied, not on `p`
write(p, 21)       // accept: I3 on ρb; KILL over Facts: contents(ρa)=11 SURVIVES because DIS(ρa,ρb) holds
                   //   (§5 family 5, total) — two distinct I1 roots with no common ⊑ ancestor.  [gap G2]
read(q)            // accept: St(ρa)=Init, contents(ρa)=11  ⇒ 11, still A
read(p)            // accept: St(ρb)=Init, contents(ρb)=21  ⇒ 21, now B
```

No window is opened, so `Carve = ∅` and II15 never fires: the case's "no exclusive interval
is needed" is delivered by the rule set's shape, not by an extra permission. The whole case
turns on one property of `KILL` (II11/I3): a store kills facts whose support is not `dis`
from the written place, so the alias's fact survives a write to the other root.

**Verdict: accepted as expected.**

### S02 — Replacement preserves storage and transfers the old value

```text
a = object(10)     // accept: I1 + I3 — St(ρa)=Init, contents(ρa)=10, one owner obligation
p = ref(a)         // accept: II19 — identity(p)=ρa
q = p              // accept: II2 — identity(q)=ρa
old = replace(p, 20)
                   // accept: I5 — premise St(ρa)=Init holds; II15 vacuous. Effect: ONE Init ⇒ Init event,
                   //   no intermediate hole (so no line between here and the next can see a hole);
                   //   KILL contents(ρa)=10; ensures contents(ρa)=20; the old value's obligation moves to `old`
read(q)            // accept: identity(q)=ρa, St(ρa)=Init, contents(ρa)=20  ⇒ 20 — the identity did NOT change
read(a)            // accept: the owner binding names the same ρa; same two facts  ⇒ 20
read(old)          // accept ON SAFETY: `old` is a bound copy-class binding, so it is readable.
                   // rule missing: I5's effect writes `ensures contents(π)=new` and moves the obligation, but
                   //   no clause says `ensures result = old(contents(π))`. The VALUE 10 is therefore not
                   //   derivable from I5; only the acceptance is.  [gap G4]
```

I5's "one event, no intermediate hole" is what makes this case different from S06: the
premise `St = Init` is consumed and re-established in the same step, so the old-value
transfer never passes through `Uninit`.

**Verdict: accepted as expected** (the case's three ACCEPTs are reproduced; the printed
value on the last line rests on a clause I5 does not write — gap G4).

### S03 — A hole is a property of the target, not one locator

```text
a = object(10)     // accept: I1 + I3
p = ref(a)         // accept: II19 — identity(p)=ρa
q = p              // accept: II2 — identity(q)=ρa, the same identity, not a second one
v = take(p)        // accept: I4 — St(ρa)=Init; effect St(ρa):=Uninit, the obligation moves to v,
                   //   KILL contents(ρa)=10.
                   // rule missing: I4's premise says "type(π)'s class is affine or linear"; the case's
                   //   content is a copy integer. The rule set's own I3 and II5 examples take from a
                   //   copy-scalar slot, so the intended reading is that take is admitted and no
                   //   obligation moves for a copy class — but no rule states it.  [gap G1]
read(q)            // reject: missing fact St(ρa)=Init. identity(q)=ρa, and `St` is keyed by the PLACE,
                   //   not by the locator (§1.2: St : place → G[state]); I4's own example says it —
                   //   "the hole is on the STORAGE". R1(a)(i); M7 names the take on the previous line
```

Variants:

```text
read(a)            // reject: same identity ρa, same missing fact — the owner name has no privilege
v2 = take(q)       // reject: I4's premise St(ρa)=Init fails; St(ρa)=Uninit
```

**Verdict: accepted as expected** (the REJECT is reproduced at the intended line, and both
variants reject for the same missing fact).

### S04 — Another alias can restore the hole

```text
a = object(10)     // accept: I1 + I3
p = ref(a)         // accept: II19
q = p              // accept: II2 — identity(q)=ρa
v = take(p)        // accept: I4 — St(ρa):=Uninit; v holds the value  [gap G1]
put(q, move v)     // accept: I6 — premise St(π_dst)=Uninit holds, with π_dst = ρa resolved from identity(q),
                   //   and v is an unconsumed source binding. Effect: St(ρa):=Init; v is consumed
read(p)            // accept: St(ρa)=Init.  Value 10: I6's effect states the obligation transfer and the
                   //   state edge; it states no `ensures contents(π_dst) = the moved value`  [gap G4]
```

The case's "p and q continue to target A, never the new location holding v" is delivered
by I1/I2 being the **only** minting rules: `v` is a value binding, no formation site is
crossed, so no second identity exists to retarget to. Copying the locator while `ρa` is
`Uninit` is likewise fine: II2's copy reads an identity, not `St`.

Intermediate state, as the case requires: between the take and the put an unconditional
`read(a)` rejects — missing fact `St(ρa)=Init`.

**Verdict: accepted as expected.**

### S05 — A hole need not be restored before its storage ends

```text
a = object(10)     // accept: I1 + I3 — one owner obligation on ρa
p = ref(a)         // accept: II19
v = take(p)        // accept: I4 — St(ρa):=Uninit; the content obligation (none, copy class) moves to v  [gap G1]
release(a)         // accept: I7, each premise in turn —
                   //   Own(a)=ρa live and unconsumed ✓;  Carve(ρa)=∅ ✓ (no focus was opened);
                   //   Span(ρa)=∅ ✓ (no span-typed binding exists; p is ptr<ρa,int>, first class, II19);
                   //   no live ⊑-child ✓ (nothing was promoted, I11);
                   //   every obligation held by a value INSIDE ρa is discharged ✓ — the take moved it out;
                   //   St(ρa) ∈ {Init, Uninit} on every feasible assignment ✓ — it is Uninit, and I7
                   //     admits BOTH, which is exactly "a hole need not be restored".
                   //   Effect: St(ρa):=Gone permanently; Own(a) consumed; facts keyed under ρa dropped
read(v)            // accept: v is an ordinary binding, not a place under ρa, so I7's fact drop does not
                   //   reach it; and I7 discharged only obligations held INSIDE ρa — v's is still v's,
                   //   so nothing is disposed twice
```

**Verdict: accepted as expected.** I7's `St(ρ) ∈ {Init, Uninit}` premise is the rule that
carries this case; the candidate does not require the hole to be filled first.

### S06 — Replacement requires old content at its commit

```text
a = object(10)     // accept: I1 + I3
p = ref(a)         // accept: II19 — identity(p)=ρa
q = p              // accept: II2 — identity(q)=ρa
v = take(q)        // accept: I4 — St(ρa):=Uninit  [gap G1]
old = replace(p, move v)
                   // reject: missing fact St(ρa)=Init — I5's premise. identity(p)=ρa is the same place
                   //   q emptied (S03's principle: St is keyed by the place).
                   //   M7 names the take on the previous line and the two repairs
```

The case's warning — "operand evaluation that performs a take must not leave an earlier
initialization premise authoritative" — is discharged structurally: `STATE` (§5 family 2)
is **one syntax-directed pass** in statement order, so I5's premise is read against the
`St` that I4 just wrote, never against a stale one. Were the take written inside the
`replace` operand, the rule set would need an evaluation-order clause it does not state;
as written here, statement order settles it.

Variant: `put(p, move v)` accepts by I6 (`St(ρa)=Uninit` is exactly I6's premise).

**Verdict: accepted as expected.**

### S07 — Release is permanent and affects every access path

```text
a = object(10)     // accept: I1 + I3 — owner obligation on ρa
p = ref(a)         // accept: II19 — identity(p)=ρa
q = p              // accept: II2 — identity(q)=ρa
release(p)         // accept: I7 with ρ = ρa, taken from identity(p); Carve=Span=∅, no ⊑-child,
                   //   St(ρa)=Init; effect St(ρa):=Gone, the owner obligation consumed.
                   // rule missing: I7's premise is written `Own(x) = ρ live and unconsumed` and its
                   //   effect "Own(x) consumed". No clause says whether x must be the SYNTACTIC argument.
                   //   Read as "the obligation keyed by ρ", permissive locator-release is admitted and the
                   //   candidate's own §3.21 row for S07 assumes it; read as "x is the argument", this line
                   //   has no rule. The derivation continues under the first reading.  [gap G3]
read(q)            // reject: missing fact St(ρa)=Init — St(ρa)=Gone, permanently (I7's effect; I23's
                   //   "no Gone name is ever revived"). R1(a)(ii); M7 names the release
```

Variants, each rejecting for a named missing fact:

```text
read(a)            // reject: same identity ρa, St(ρa)=Gone
write(q, 7)        // reject: I3's premise St(π) ∈ {Uninit, Init} fails — Gone is not in it, so a store
                   //   cannot revive dead storage (I23 states the permanence)
take(q)            // reject: I4's premise St(π)=Init fails
release(q)         // reject: I7's premise "Own live and unconsumed" fails — consumed at the release;
                   //   independently St(ρa)=Gone
release(a)         // reject: the same consumed obligation (R3/R2), as I7's own second example line
q2 = q             // accept: II2 copies an identity and reads no state; ptr<ρa,int> is first class, so
                   //   II19's ESC does not apply — copying or discarding an inert locator is allowed
```

**Verdict: accepted as expected** (every listed variant rejects, and the inert-locator copy
is allowed), **with one line resting on the I7 reading recorded as gap G3.**

---

## 2. Branch cases

### B01 — One locator can have alternative targets

```text
a = object(10)     // accept: I1 + I3 — St(ρa)=Init
b = object(20)     // accept: I1 + I3 — St(ρb)=Init
if cond { p = ref(a) } else { p = ref(b) }
                   // accept: II1 — the discriminant `cond` is a boolean BINDING, so atom(cond,1) enters Γg
                   //   (had it been an expression, II1 captures nothing and the next line would need
                   //   II3's collapse to survive)
                   // accept: II2 — the then-arm writes only the true leaf, the else-arm only the false leaf
                   // accept: II3 — join: identity(p) = ite(cond, ρa, ρb), a guarded identity (§1.1),
                   //   canonical, depth 1 ≤ Gmax = 2
read(p)            // accept: II6(a) — the obligation St(place(p))=Init is checked on EVERY feasible
                   //   assignment: cond=true resolves p to ρa with St(ρa)=Init; cond=false resolves p to
                   //   ρb with St(ρb)=Init. Both hold, so the read is admitted with no runtime test
```

Note for M2(ii): the guarded identity divides *facts*; the operation executed is one
unconditional read whose operand is the ordinary selected runtime pointer. II6(b) has
nothing to object to.

**Verdict: accepted as expected.** Direct O11 evidence: the accept exists only because the
join kept a disjunction; under an equality join `identity(p)` would have no value.

### B02 — Target and initialization must remain correlated

```text
a = object(10)     // accept: I1 + I3
b = object(20)     // accept: I1 + I3
if cond {
    p = ref(a)     // accept: II2 — true leaf only: identity(p)@true = ρa
    old = take(b)  // accept: I4 under the arm — II2 writes only the true leaf: St(ρb)@true := Uninit  [gap G1]
} else {
    p = ref(b)     // accept: II2 — false leaf: identity(p)@false = ρb
    old = take(a)  // accept: I4 — St(ρa)@false := Uninit  [gap G1]
}
                   // accept: II3 — join over ONE atom:
                   //   identity(p) = ite(cond, ρa, ρb)
                   //   St(ρa)      = ite(cond, Init,   Uninit)
                   //   St(ρb)      = ite(cond, Uninit, Init)
                   //   depth 1 ≤ Gmax; no collapse
read(p)            // accept: II6(a) resolves each feasible assignment with the SAME atom on both sides:
                   //   cond=true  ⇒ place ρa, St(ρa)@true  = Init ✓
                   //   cond=false ⇒ place ρb, St(ρb)@false = Init ✓
                   //   The correlation survives because one atom keys the identity and the state; there
                   //   are no independent target and initialization sets to take a product of
```

Swap variant (each arm takes p's own selected object):

```text
if cond { p = ref(a); old = take(a) } else { p = ref(b); old = take(b) }
read(p)            // reject: missing fact St(ρa)=Init on cond=true (and St(ρb)=Init on cond=false);
                   //   II6(a) requires it on every feasible assignment, and it holds on neither
```

**Verdict: accepted as expected** (both the accept and the swapped rejection).

### B03 — A write initializes the selected target, not its entire may-set

```text
a = object(10)     // accept: I1 + I3
b = object(20)     // accept: I1 + I3
old_a = take(a)    // accept: I4 — St(ρa):=Uninit, unguarded  [gap G1]
old_b = take(b)    // accept: I4 — St(ρb):=Uninit, unguarded  [gap G1]
if cond { p = ref(a) } else { p = ref(b) }
                   // accept: II1 atom(cond,1); II2 per-arm; II3 join ⇒ identity(p)=ite(cond, ρa, ρb)
q = p              // accept: II2 verbatim — "q copies the guarded identity, not the variable name".
                   //   identity(q) is the SAME ite term, not a fresh may-set
write(p, 9)        // accept: I3 — premise St(π) ∈ {Uninit, Init} on every feasible assignment:
                   //   cond=true ⇒ π=ρa, St=Uninit ✓;  cond=false ⇒ π=ρb, St=Uninit ✓
                   // accept: II2's write clause — "every write writes ONLY that arm's leaf":
                   //   St(ρa) := ite(cond, Init, Uninit);  St(ρb) := ite(cond, Uninit, Init)
                   //   (NOT both to Init — that is the whole content of this case)
read(q)            // accept: identity(q) is the same guarded identity, so on each assignment the resolved
                   //   place is precisely the one just written: St=Init on both ⇒ 9
read(a)            // reject: missing fact St(ρa)=Init on the cond=false assignment, which is feasible
                   //   (II6(a)). M7 names the write and the arm
```

Variant `read(b)`: reject, missing `St(ρb)=Init` on cond=true.

**Verdict: accepted as expected.** This case is the sharpest O11 evidence in the set: the
accept of `read(q)` and the reject of `read(a)` differ only because the identity and the
state are guarded by the same atom.

### B04 — A copied uncertain target supports take and restoration

```text
a = object(10)     // accept: I1 + I3
b = object(20)     // accept: I1 + I3
if cond { p = ref(a) } else { p = ref(b) }
                   // accept: II1 + II2 + II3 ⇒ identity(p) = ite(cond, ρa, ρb)
q = p              // accept: II2 — identity(q) = the same term
v = take(p)        // accept: I4 on every feasible assignment — cond=true: St(ρa)=Init ✓;
                   //   cond=false: St(ρb)=Init ✓. II2's leaf rule gives
                   //   St(ρa)=ite(cond, Uninit, Init), St(ρb)=ite(cond, Init, Uninit);
                   //   contents facts on the selected leaves are killed  [gap G1; see the note below]
read(a)            // reject (the case's stated intermediate check): missing fact St(ρa)=Init on cond=true
put(q, move v)     // accept: I6 — premise St(π_dst)=Uninit on each assignment, with π_dst resolved from
                   //   identity(q) = the SAME term, so the restored place is the emptied one:
                   //   cond=true ⇒ ρa (Uninit ✓);  cond=false ⇒ ρb (Uninit ✓).
                   //   Effect on the selected leaf only (II2): St(ρa)=ite(cond, Init, Init),
                   //   St(ρb)=ite(cond, Init, Init); §1.1's canonical form applies ite(c,x,x) → x (II3),
                   //   so both become unguarded Init. v is consumed
read(a)            // accept: St(ρa)=Init, unguarded after the collapse
read(b)            // accept: St(ρb)=Init, unguarded after the collapse
```

Note on the take: `v` is one binding taken from a *guarded* source identity. For the copy
class of this fragment nothing moves, so the line is clean. For an affine content the
obligation arriving at `v` would itself be guarded; I4's effect ("the obligation moves to
the taken binding") is written for an unguarded source and does not state the guarded case.
Out of this case's scope, recorded as part of gap G1.

**Verdict: accepted as expected.** The `ite(c,x,x) → x` collapse (§1.1 canonicality, II3) is
what makes the two post-restoration reads unguarded rather than merely guarded-true.

### B05 — Correlated distinct targets survive selected reclamation

```text
a = object(10)     // accept: I1 + I3 — obligation O_a
b = object(20)     // accept: I1 + I3 — obligation O_b
if cond {
    p = ref(a)     // accept: II2 — true leaf
    q = ref(b)
} else {
    p = ref(b)     // accept: II2 — false leaf
    q = ref(a)
}
                   // accept: II3 — join over ONE atom:
                   //   identity(p) = ite(cond, ρa, ρb)
                   //   identity(q) = ite(cond, ρb, ρa)
                   //   The actual pairs are exactly (ρa,ρb) and (ρb,ρa): II6(a) enumerates ASSIGNMENTS of
                   //   the atom, not a product of two independent sets, so (ρa,ρa) is not a state
                   // accept: DIS(identity(p), identity(q)) = true — §5 family 5 resolves a guarded identity
                   //   per assignment (O(L) resolutions): dis(ρa,ρb) on true, dis(ρb,ρa) on false  [gap G2]
release(p)         // accept: I7 per feasible assignment —
                   //   cond=true : ρ=ρa; Own live ✓, Carve=Span=∅ ✓, no ⊑-child ✓, St(ρa)=Init ✓
                   //   cond=false: ρ=ρb; the same premises ✓
                   //   Effect: St(ρa)=ite(cond, Gone, Init), St(ρb)=ite(cond, Init, Gone); the owner
                   //   obligation is consumed on the selected leaf — a GUARDED obligation (II3, III12)
                   // accept under II6(b): the source executes exactly one release, unconditionally; the
                   //   atom divides facts and selects no operation. The operand is the ordinary selected
                   //   runtime pointer the source's own `if` computed. Contrast B13, where the atom would
                   //   have to select whether a release runs at all
                   // [gap G3: the locator-release reading of I7, as in S07]
read(q)            // accept: identity(q)=ite(cond, ρb, ρa); cond=true ⇒ ρb, St(ρb)@true=Init ✓;
                   //   cond=false ⇒ ρa, St(ρa)@false=Init ✓. q's target is the complement of p's under
                   //   the same atom, so the release never reaches it
release(q)         // accept: I7 per assignment — St=Init on the resolved place, its obligation still live.
                   //   After it: on every feasible assignment St(ρa)=St(ρb)=Gone and O_a, O_b are both
                   //   consumed exactly once (II6(a) over both assignments) — R2 discharged
```

Stated negative checks from the case:

```text
read(a)            // (in place of read(q)) reject: missing fact St(ρa)=Init on cond=true
release(a)         // (in place of release(q)) reject: O_a is consumed on the cond=true assignment —
                   //   II6(a) makes that assignment feasible, so the double consumption is refused (R3)
```

**Verdict: accepted as expected.**

### B06 — Rebinding must update relations without retargeting saved copies

```text
a = object(10)     // accept: I1 + I3
b = object(20)     // accept: I1 + I3
if cond {
    p = ref(a); q = ref(b)     // accept: II2 — true leaf
} else {
    p = ref(b); q = ref(a)     // accept: II2 — false leaf
}
                   // accept: II3 ⇒ identity(p)=ite(cond, ρa, ρb), identity(q)=ite(cond, ρb, ρa)
saved = p          // accept: II2 — saved takes the IDENTITY VALUE ite(cond, ρa, ρb).
                   //   Nothing in §1.2 keys on the variable `p`: St keys on places, Facts and Own key on
                   //   identities, so there is no name-level fact that can go stale later
p = q              // accept: an ordinary rebinding — identity(p) := ite(cond, ρb, ρa).
                   //   The former inequality is not stored anywhere: DIS is RECOMPUTED per query from the
                   //   identity terms (§5 family 5), so DIS(identity(p), identity(q)) is now FALSE on both
                   //   assignments while DIS(saved, identity(p)) is still TRUE on both  [gap G2]
release(p)         // accept: I7 per feasible assignment — cond=true ⇒ ρb (Init, owner live);
                   //   cond=false ⇒ ρa (Init, owner live). Effect:
                   //   St(ρb)=ite(cond, Gone, Init), St(ρa)=ite(cond, Init, Gone), obligation consumed
                   //   on the selected leaf.  II6(b) neutral, as in B05  [gap G3]
read(saved)        // accept: saved = ite(cond, ρa, ρb), the complement term;
                   //   cond=true ⇒ ρa, St(ρa)@true=Init ✓;  cond=false ⇒ ρb, St(ρb)@false=Init ✓
read(q)            // reject: identity(q)=ite(cond, ρb, ρa), which after `p = q` is the SAME term as
                   //   identity(p); on cond=true the resolved place is ρb with St(ρb)@true=Gone, and on
                   //   cond=false it is ρa with St(ρa)@false=Gone. Missing fact St=Init on BOTH feasible
                   //   assignments — R1(a)(ii). M7 names the release and the assignment
```

**Verdict: accepted as expected.** The case's requirement — "the old inequality between p
and q cannot remain attached to their variable names" — is met structurally rather than by
a rule: the candidate stores no name-keyed relation at all, and `DIS` is a query over
identity terms, so there is nothing to invalidate on assignment.

---

## 3. Rule gaps found in this part

None of these changes a verdict. Each is a line the candidate's rules do not literally
cover, derived under the reading the rule set's own examples use.

| # | Gap | Where the rules are silent | Items affected |
|---|---|---|---|
| **G1** | **`take` of a copy scalar has no rule.** I4's premise requires `type(π)`'s class to be *affine or linear*; the whole CASES fragment takes copy integers, and the candidate's own I3 example (`if cond { old = take(p) }` followed by a copy-scalar `write`) and II5 example do exactly that. No rule states the copy case or says that no obligation moves for it. I4's effect also does not state the guarded-source case (B04) | I4 premises, I4 effect | S03, S04, S05, S06, B02, B03, B04 |
| **G2** | **`DIS` on two unrelated `⊑`-roots has no clause.** §5 declares `DIS` total and I11 fixes sibling `dis` by `EXT` "never by inequality of names" (R4(b)); no rule says what `DIS(ρa, ρb)` is for two distinct I1 roots with no common ancestor. Every alias-survival step here needs it to be *true* | §5 family 5; I1 | S01 (load-bearing: `contents(ρa)` must survive the write to `ρb`), B05, B06 |
| **G3** | **Release through a locator has no rule.** I7's premise is `Own(x) = ρ live and unconsumed` and its effect is "`Own(x)` consumed"; no clause says whether `x` is the syntactic argument or merely the binding that owns the `ρ` the argument resolves to. CASES' permissive variant and the candidate's own §3.21 rows for S07/B05 need the second reading | I7 premises/effect | S07, B05, B06 |
| **G4** | **No `ensures` carries a value out of storage.** I5 writes `ensures contents(π)=new` but no `ensures result = old(contents(π))`; I6 states the obligation transfer but no `ensures contents(π_dst) = the moved value`. Acceptance is unaffected; the cases' printed values are not derivable | I5, I6 effects | S02, S04 |

Additionally, no numbered rule introduces a pointer binding (`p = ref(a)`) or an ordinary
rebinding; II19's type grammar and II2's identity-copy clause supply both, and the rule
set's own examples use them. This is recorded as a presentation gap, not a derivation gap.

---

## 4. Summary table

| Case | Rules used | Verdict here | CASES expectation | Classification | Gaps |
|---|---|---|---|---|---|
| S01 | I1, I3, II2, II19, `KILL`/`DIS` | accept all four reads; the alias fact survives the write to the other root | accept | **accepted as expected** | G2 |
| S02 | I1, I3, I5, II2 | accept; one `Init ⇒ Init` event, no hole | accept | **accepted as expected** | G4 |
| S03 | I4, `St` keyed by place | reject `read(q)`; `read(a)` and `take(q)` reject too | reject | **accepted as expected** | G1 |
| S04 | I4, I6, II2 | accept; the restored place is the emptied one | accept | **accepted as expected** | G1, G4 |
| S05 | I4, I7 (`Uninit ⇒ Gone` admitted) | accept the release of an empty slot; `read(v)` accepts | accept | **accepted as expected** | G1 |
| S06 | I4, I5 premise, `STATE`'s single pass | reject at `replace`; `put` variant accepts | reject | **accepted as expected** | G1 |
| S07 | I7, I23, I3/I4 premises, II19 | release accepted; every later access path rejects; inert copy allowed | accept then reject | **accepted as expected** | G3 |
| B01 | II1, II2, II3, II6(a) | accept; `identity(p)=ite(cond,ρa,ρb)` | accept | **accepted as expected** | — |
| B02 | II1, II2, II3, I4, II6(a) | accept; swapped variant rejects on both arms | accept | **accepted as expected** | G1 |
| B03 | II2's leaf-write clause, I3, II6(a) | `read(q)` accepts, `read(a)`/`read(b)` reject | accept / reject | **accepted as expected** | G1 |
| B04 | I4, I6, II2, II3's `ite(c,x,x)→x` | accept; both slots unguarded `Init` afterwards | accept | **accepted as expected** | G1 |
| B05 | II2, II3, I7, II6(a), II6(b), `DIS` | both reads and both releases accept; `read(a)`/`release(a)` reject | accept | **accepted as expected** | G2, G3 |
| B06 | II2, II3, I7, `DIS` per assignment | `read(saved)` accepts, `read(q)` rejects | accept / reject | **accepted as expected** | G2, G3 |

**Counts.** 13 items; 13 accepted as expected; 0 capability losses; 0 deliberate refusals;
0 corrections of a case; 0 items underivable. Four rule gaps, none verdict-changing.

**What this part contributes to O11.** Every branch case here (B01–B06) is decided by a
*guarded identity*, a *guarded state*, or both under one atom, and B02, B03, B05 and B06
are accepts that a join collapsing to equality cannot produce: B02 and B05 need the
identity and the state to share an atom, B03 needs the write to reach one leaf only, B06
needs `DIS` to be a query over identity terms rather than a stored name relation. In this
part, condition terms at joins are not a convenience — six of six branch cases fail without
them. Their cost here is bounded and visible: one atom, depth 1, well inside `Gmax = 2`;
none of these cases exercises the `Gmax` collapse (OD2) at all.

**What this part contributes to O7/O12.** Nothing directly: no case in this part forms a
projection, stores a reference, or crosses a cut, so II19's `ESC` discipline and I11's
promotion are never reached. The one adjacent observation is negative and useful — every
locator here is `ptr<ρ,T>`, the **first-class** form, so S01–S07's copying, rebinding and
saving of locators (including B06's `saved`) is unaffected by the second-class projection
boundary. The boundary bites only where a locator names something finer than a coarse name,
which this part never does.


# File: derive-window-focus-late-branches.md

# Derivation: candidate `window-focus`, part `late-branches`

Items: `CASES.md` B07, B08, B09, B10, B11, B12, B13; regression checks
`PROGRAMS.md` P1 and P3. Research date: 2026-09-16.

Every line below is derived **only** from
[`rules-window-focus.md`](rules-window-focus.md) (§1 the model, §2 rule set I,
§3 rule set II, §4 rule set III, §5 the judgment families). Each accept cites
the rule that admits it; each reject cites the rule and the missing fact. Where
the rule sets do not cover a line, the line is marked **rule missing (Gn)** and
the derivation continues under the reading the candidate's own worked examples
force, which is stated each time. No rule is improvised, no candidate is
compared, and none is selected.

`CASES.md`'s expected verdicts were written under DESIGN.md Candidate A. Where a
verdict here differs it is classified as *capability loss*, *deliberate refusal
with a named repair*, or *correction of the case*.

---

## 0. Vocabulary map and gap register

### 0.1 `CASES.md` operations, mapped to the rules once

`CASES.md` predates this candidate's vocabulary. The mapping is fixed here and
reused without restating it per case.

| `CASES.md` | Rule that admits it | Effect used below |
|---|---|---|
| `a = object(10)` | I1 then I3 | mint fresh `ρa`, a `⊑`-root; `St(ρa) := Uninit`; mint an affine owner binding `a` with one end-obligation; then `St(ρa) := Init` |
| `p = ref(a)` | no numbered rule; the form is used verbatim in I3's and II5's own worked examples; the type is II19's first-class `ptr<ρa,u64>` | the binding's identity is `ρa`; nothing is consumed and no window is opened (II14 is the only opener), so `p` imposes no interference |
| `read(p)` | premise `St(ι(p)) = Init` on every feasible guard assignment (I1's example, R1(a)(i), II6(a)) | — |
| `write(p, n)` | I3 | `St := Init` from `Uninit` **or** `Init`; `KILL` contents facts not `dis` from the place |
| `v = take(p)` | I4 | `St := Uninit`; the value's obligation moves to `v` (none here: a copy integer) |
| `put(p, v)` | I6 | premise `St = Uninit`; `St := Init` |
| `old = replace(p, n)` | I5 | premise `St = Init`; one event, no hole; `ensures contents(π) = n` |
| `release(a)` / `release(p)` | I7 | premises `Own(x)` live, `Carve(ρ)=∅`, `Span(ρ)=∅`, no live `⊑`-child, `St(ρ) ∈ {Init,Uninit}` on every feasible assignment; `St(ρ) := Gone` permanently; `Own(x)` consumed |
| `cond` as a branch discriminant | II1 + III12 | a boolean **binding version** `cond@v` becomes a guard atom; the arms are checked under `cond@v = true` / `= false` |
| `cond = !cond` | II5 | a new version `cond@2`; the relation `cond@2 = ¬cond@1` is recorded because the RHS is `!c` |
| the join after `if`/`else` | II3 | every `St` key and every `Facts` entry becomes `ite(c, then, else)`, canonical, with `ite(c,x,x) → x`; depth `≤ Gmax` |
| leaving the owning scope | I8 | every owner binding's obligation must be discharged or moved out **on every feasible guard assignment** |

Two consequences used repeatedly:

- **State is on the place, never on the locator** (§1.1, §1.2 `St`). Every
  `CASES.md` case that turns on "the hole belongs to the target, not one
  pointer" is decided by that one fact, and no rule about `p` or `q` is needed.
- **A window is the only thing that carries interference** (II14: "no state is
  created or moved"). None of B07–B13 opens a `focus`, so `Carve = ∅` and
  `Span = ∅` hold throughout this part and I7's and I17's structural premises
  are discharged trivially. They become load-bearing only in P1 and P3.

### 0.2 Gap register

Every line marked **rule missing** below resolves to one of these. They are
listed once here and cited by number.

| # | Gap | Where it bites |
|---|---|---|
| **G1** | **I4's premise contradicts I4's own worked example.** I4 requires `type(π)`'s class to be *affine or linear*; I3 requires `type(π)` to be a *copy* type. `CASES.md`'s slots hold copy integers, and I3's worked example — which names `CASES B07` on its own reject line — applies `take` to exactly such a slot. Under the written premise the `take` line has no rule; under the worked example it is admitted | B07, B08, B09 |
| **G2** | **II3 joins `St` and `Facts` only.** §1.2's structures also include `Own` (owner binding → identity *with its obligation*), and §1.2 lists **no** structure at all mapping an ordinary pointer binding to its identity, although II2's example asserts `p`'s identity is `ite(c, ρa, ρb)`. Neither has a join rule, yet I8, II6(a) and every conditional-release case need a guarded `Own` and B11 needs a guarded binding identity | B10, B11, B12, B13 |
| **G3** | **I7 is written over an owner binding at a coarse name** (`Own(x) = ρ`). Ending through a *locator* (the `CASES.md` working variant) and ending at a *guarded identity* `ite(c, ρb, ρa)` are both unstated, though §3.21's B05 and B11 rows assert them | B11 |
| **G4** | **`if !c` is an expression discriminant.** II1 says an expression discriminant "captures nothing", but II5's and II6's worked examples (and §3.21's B09 and B12 rows) require `if !c` to be checked under `¬atom(c)`. Nothing states the rule that admits it | B09 variant, B12 variant |
| **G5** | **II6(b)'s prose and its formal statement diverge.** The prose forbids "a release selected by an atom"; the formal statement is "the set of operations executed by an accepted program is a function of the source's own control flow", which an *edge-placed* cleanup on the source's own `else` edge satisfies. B13's refusal rests on the prose, not on the formal criterion, and no rule says which governs | B13 |
| **G6** | **No rule connects a guard atom to the predicate its defining expression computes.** III12 makes the *binding version* the atom; II5 records a relation only when the RHS is `c`, `!c` or a literal, and explicitly leaves any other atom **free**; II12 binds an atom to a predicate only for a *checker*-introduced arm. So `let ok = (k != i && k != j); if ok { … }` gives the then-arm no index fact | P1 |
| **G7** | **I3 and I6 have no `ensures contents` clause.** Both `KILL` the old contents fact; only I5 re-establishes one. The acceptance verdicts are derivable; the *value* claims of the cases ("returning 20", "20 if true, 10 if false") are not | B07, B08 |
| **G8** | **`DIS` between two distinct `⊑`-roots has no stated ground.** I1 mints a "fresh `ρ`", which is a naming effect, and R4(b)/C3 forbid grounding `dis` on inequality of names (I11 and III6 say so explicitly). Nothing derives `dis(ρa, ρb)` for two independent `object(…)` formations, which II11 needs so that releasing one spares the other's facts | B10, B11; also P1, P3 |

---

## 1. B07 — a conditional hole prevents reading but not scalar overwrite

Expected (CASES): `read(p)` REJECT ("A may be empty"); the variant
`write(p, 20); read(p)` accepts, returning 20 on both paths; `put` would lack
its premise on the full path and `replace` on the empty path.

### 1.1 The rejecting form

```text
a = object(10)                 // accept: I1 mints ρa (⊑-root, fresh), St(ρa) := Uninit, owner binding `a` with
                               //   one end-obligation; I3 then admits Uninit ⇒ Init for the copy scalar
p = ref(a)                     // accept: identity(p) = ρa; type ptr<ρa,u64>, first class (II19's grammar).
                               //   No carve is opened (II14 is the only opener) ⇒ Carve(ρa) = Span(ρa) = ∅
if cond { old = take(p) }      // accept (guard): II1 — the discriminant is a boolean BINDING, so atom cond@1
                               //   enters Γg; the then-arm is checked under cond@1 = true, the else under false
                               // accept (then-arm): II2 — reads resolve at the arm's leaf (St(ρa) = Init there);
                               //   I4's effect gives St(ρa) := Uninit on that leaf only, and KILLs contents(ρa)
                               //   rule missing (G1): I4's written premise wants an affine or linear class and
                               //   this slot holds a copy integer. Read as I3's worked example reads it — that
                               //   example cites CASES B07 by name — the line is admitted
                               // accept (obligation): `old` carries no obligation (CASES: a taken integer is an
                               //   ordinary scalar temporary), so I8 has nothing to require of it later.
                               //   Under G1's other reading it would, and this fragment would also reject at the
                               //   scope exit for never consuming `old`
                               // accept (else-arm): empty; the leaf keeps St(ρa) = Init
                               // accept (join): II3 — St(ρa) := ite(cond@1, Uninit, Init), one atom, depth 1 ≤ Gmax
read(p)                        // reject: missing fact St(ρa) = Init on the assignment cond@1 = true.
                               //   II6(a): that assignment is feasible — Γg records no relation constraining
                               //   cond@1 — and an obligation must hold on EVERY feasible assignment.
                               //   R1(a)(i); M7 names the take at the previous line (I1's diagnostic shape)
```

This is I3's worked example verbatim, including its reject line.

### 1.2 The accepting variant

```text
if cond { old = take(p) }      // as above: St(ρa) = ite(cond@1, Uninit, Init)
write(p, 20)                   // accept: I3 — its premise is St(π) ∈ {Uninit, Init}, satisfied on BOTH leaves;
                               //   this is the rule's stated "total store", written there against CASES B07
                               // accept (effect): II2's leafwise discipline gives St := Init on each leaf;
                               //   II3's canonicalization collapses ite(cond@1, Init, Init) → Init
                               // rule missing (G7): I3 KILLs contents(ρa) and establishes no new contents fact,
                               //   so "returns 20" is not derivable. The SAFETY verdict does not depend on it
read(p)                        // accept: St(ρa) = Init unguarded — no feasible assignment leaves a hole
```

### 1.3 The two operation-specific refusals CASES names

```text
if cond { old = take(p) }
put(p, 20)                     // reject: I6's premise is St(π_dst) = Uninit; on the cond@1 = false assignment
                               //   St(ρa) = Init. missing fact: Uninit on the ¬cond leaf
old2 = replace(p, 20)          // reject: I5's premise is St(π) = Init; on cond@1 = true St(ρa) = Uninit.
                               //   missing fact: Init on the cond leaf
```

Both refusals come from the operations' own premises and neither narrows I3,
which is exactly what the case requires.

**Verdict: accepted as expected.** Two gaps recorded: G1 (the `take` line has no
premise-level rule for a copy-scalar slot, only a worked example that names this
case), G7 (the value claim is not derivable, only the acceptance).

---

## 2. B08 — repeating an unchanged condition recovers a path's state

Expected (CASES): ACCEPT, reading 20 on the true path and 10 on the false path.

```text
a = object(10)                 // accept: I1 + I3
p = ref(a)                     // accept: identity(p) = ρa (§0.1)
if cond { old = take(p) }      // accept: II1 captures cond@1; I4 on the true leaf (G1 as in B07);
                               //   II3 joins to St(ρa) = ite(cond@1, Uninit, Init)
if cond { put(p, 20) }         // accept (guard): II1 — "Γg += atom(c, version) if not already live". No
                               //   assignment to `cond` intervened, so II5 minted no new version: this is the
                               //   SAME atom cond@1, not a second one. Γg is unchanged; Gmax is not approached
                               // accept (then-arm): II2 — every read resolves at cond@1 = true, where
                               //   St(ρa) = Uninit; I6's premise St(π_dst) = Uninit is therefore discharged,
                               //   and its effect writes only that leaf: St(ρa) := Init there
                               // accept (else-arm): empty; the cond@1 = false leaf keeps Init
                               // accept (join): II3 — ite(cond@1, Init, Init) → Init by the stated collapse
read(p)                        // accept: St(ρa) = Init, unguarded, on every feasible assignment (II6(a))
                               // rule missing (G7): "20 if true, 10 if false" is not derivable — I6 states no
                               //   contents ensures and I4 killed the old contents fact
```

This is III12's worked example, which cites B08 by name ("the SAME version c@1 ⇒
the arms line up").

**Verdict: accepted as expected.** G1 and G7 recorded; the safety verdict is
unaffected by either.

---

## 3. B09 — conditions refer to captured values, not permanent variable names

Expected (CASES): REJECT ("the original true path remains empty"); the variant
with `if !cond { put(p, 20) }` accepts.

### 3.1 The rejecting form

```text
a = object(10); p = ref(a)     // accept: I1 + I3; identity(p) = ρa
if cond { old = take(p) }      // accept: II1 captures cond@1; I4 (G1); II3 joins
                               //   St(ρa) = ite(cond@1, Uninit, Init)
cond = !cond                   // accept: II5 — an assignment to a boolean binding creates version cond@2, and
                               //   because the RHS is `!c` the relation cond@2 = ¬cond@1 is RECORDED
                               // accept: II5 — "every guarded value mentioning the old version keeps the old
                               //   version, which stays live while any value mentions it": St(ρa) still keys cond@1
                               //   (III12's kill clause: assignment retires the version for NEW guards only)
if cond { write(p, 20) }       // accept (guard): II1 — the discriminant is the binding `cond`, now at version 2;
                               //   the then-arm is checked under cond@2 = true, i.e. under cond@1 = false by the
                               //   recorded relation (II6(a) admits only recorded = and ¬ relations, and this is one)
                               // accept (then-arm): at cond@1 = false the leaf is Init; I3 admits Init ⇒ Init
                               //   and writes that leaf only (II2)
                               // accept (join): II3 — the cond@1 = true leaf was not reachable in this arm and is
                               //   unchanged: St(ρa) = ite(cond@1, Uninit, Init) still
read(p)                        // reject: missing fact St(ρa) = Init on the feasible assignment cond@1 = true
                               //   (equivalently cond@2 = false). II6(a) quantifies over feasible assignments.
                               //   M7 names BOTH versions and the repair (III12's stated diagnostic shape)
```

This is II5's worked example verbatim, which cites B09 by name.

### 3.2 The accepting variant

```text
if !cond { put(p, 20) }        // rule missing (G4): II1's premise admits a boolean BINDING or a checker-introduced
                               //   predicate; `!cond` is an expression, and II1 says an expression discriminant
                               //   "captures nothing". II5's own example writes this line and accepts it under
                               //   "arm ¬cond@2 = cond@1", and §3.21's B09 row asserts the acceptance.
                               //   Derived under the worked example's reading:
                               // accept (then-arm): ¬cond@2 ≡ cond@1 = true, the taken path, where St(ρa) = Uninit;
                               //   I6's premise is discharged; I6 writes that leaf: St(ρa) := Init there
                               // accept (join): II3 — ite(cond@1, Init, Init) → Init
read(p)                        // accept: St(ρa) = Init on every feasible assignment
```

Under II1's written premise instead, the variant captures no atom, both arms are
checked under the unrefined state, and the `put` line rejects on its own premise
at the `cond@1 = false` leaf — i.e. the variant CASES requires to accept would
reject. The gap is therefore load-bearing, not cosmetic.

**Verdict: accepted as expected** (rejection and variant both), with G4 recorded
as the rule the accepting variant needs and G1 as in B07.

---

## 4. B10 — a join must not erase conditional disposal obligations

Expected (CASES): no verdict line; the requirement is that the join retain
exactly one obligation per path — intersection (which yields the empty set) is
incorrect, and so is automatic omission.

```text
a = object(10)                 // accept: I1 + I3 — ρa, owner binding `a`, one end-obligation
b = object(20)                 // accept: I1 + I3 — ρb, owner binding `b`, one end-obligation
                               // rule missing (G8): nothing derives dis(ρa, ρb). Both are ⊑-roots; I1's "fresh"
                               //   is a naming effect and R4(b)/C3 forbid name inequality as the ground for dis.
                               //   II11 needs it below so that ending one spares the other's facts. Derived as
                               //   true here — it is the only reading under which I1 is usable at all
if cond { release(a) } else { release(b) }
                               // accept (guard): II1 — atom cond@1 enters Γg
                               // accept (then-arm): I7 on ρa — Own(a) live and unconsumed; Carve(ρa) = ∅ and
                               //   Span(ρa) = ∅ (no focus was opened, §0.1); no live ⊑-child (no I11 promotion);
                               //   no obligation held by a value inside ρa (a copy integer); St(ρa) = Init.
                               //   effect: St(ρa) := Gone permanently, Own(a) consumed, facts keyed under ρa dropped
                               // accept (else-arm): I7 on ρb, symmetrically
                               // accept (frame): II11(1) — each arm's footprint is one root; the other root's
                               //   facts survive because dis holds (G8)
                               // accept (join, St): II3 — St(ρa) = ite(cond@1, Gone, Init) and
                               //   St(ρb) = ite(cond@1, Init, Gone). Neither leaf is discarded; the difference is
                               //   kept as a FACT, which is the case's first requirement
                               // rule missing (G2, the case's second requirement): II3's effect is written over
                               //   "every key of St and every fact in Facts". Own is neither. Nothing states how
                               //   the consumption state of the affine owner bindings `a` and `b` joins.
                               //   Derived leafwise, the only reading under which I8's own worked example
                               //   ("reject: missing discharge on the ¬cond arm") and II6(a) make sense:
                               //   Own(a) is consumed on the cond@1 = true leaf and live on the false leaf;
                               //   Own(b) the mirror. Exactly one obligation outstanding per feasible assignment
                               // accept (no drop flag): II6(b) — the difference lives in the checker's guarded
                               //   values, which §1.4 erases; nothing is materialized (M2)
```

Intersection is excluded because II3 keeps both leaves rather than meeting them
(the meet `⊓` fires only at the `Gmax` collapse, and depth here is 1). Automatic
omission is excluded because II6(a) quantifies R2 over feasible assignments, so
the surviving obligation on each leaf is still owed; B13 shows where it is
collected.

**Verdict: accepted as expected** — the obligation is retained, guarded, exactly
as §3.21's B10 row claims. Two gaps recorded: G2 (the guarded `Own` this row
depends on has no join rule) and G8 (`dis` between two independent roots).

---

## 5. B11 — a locator can identify the conditionally remaining obligation

Expected (CASES): `read(remaining)` ACCEPT; `release(remaining)` ACCEPT, after
which neither obligation remains.

```text
a = object(10); b = object(20) // accept: I1 + I3 twice (G8 for dis(ρa, ρb))
if cond {
    release(a)                 // accept: I7 on ρa, premises as B10; St(ρa) := Gone; Own(a) consumed
    remaining = ref(b)         // accept: identity(remaining) = ρb; ptr<ρb,u64>, first class (II19)
} else {
    release(b)                 // accept: I7 on ρb; St(ρb) := Gone; Own(b) consumed
    remaining = ref(a)         // accept: identity(remaining) = ρa
}
                               // accept (join, St): II3 — St(ρa) = ite(cond@1, Gone, Init),
                               //   St(ρb) = ite(cond@1, Init, Gone)
                               // accept (join, identity): II2's guarded identity form — §1.1 admits
                               //   ι ::= ite(c, ι, ι), and II2's own example builds exactly this shape for a
                               //   locator bound in two arms: identity(remaining) = ite(cond@1, ρb, ρa)
                               // rule missing (G2): §1.2 lists no structure holding an ordinary binding's
                               //   identity (Own is owner bindings, Span is window bindings), and II3 joins only
                               //   St and Facts. Derived under II2's example, which asserts the guarded identity
read(remaining)                // accept: II2's resolution rule — under cond@1 = true the target is ρb with
                               //   St(ρb) = Init; under false it is ρa with St(ρa) = Init.
                               //   St = Init on every feasible assignment (II6(a)). R1(a)(i) discharged
                               // accept (this is the case's point): the state was read off the PLACE the guarded
                               //   identity selects, not off the name `remaining` — §1.2's St is keyed by place
release(remaining)             // rule missing (G3): I7's premises are written as `Own(x) = ρ` for an affine owner
                               //   binding at a COARSE name. Here the operand is a locator whose identity is the
                               //   guarded ite(cond@1, ρb, ρa). Neither the locator form (CASES's working variant)
                               //   nor the guarded-identity form is stated. §3.21's B05 and B11 rows assert both
                               // accept, derived under II2's selected-leaf schema (the schema I3 uses for
                               //   write(p,9) at a guarded identity, "writes the SELECTED leaf"):
                               //   leaf cond@1 = true  → I7 on ρb: St(ρb) = Init ✓, Carve = Span = ∅ ✓,
                               //     Own(b) live on this leaf (B10's leafwise Own) ✓ ⇒ St(ρb) := Gone, Own(b) consumed
                               //   leaf cond@1 = false → I7 on ρa, symmetrically
                               //   post-state: St(ρa) = ite(cond@1, Gone, Gone) → Gone and St(ρb) → Gone (II3's
                               //   collapse); no owner binding is live on any feasible assignment ⇒ R2 discharged
                               // accept (M2): II6(b) is satisfied on its FORMAL criterion and its prose alike —
                               //   exactly one release executes on every path, i.e. the operation set is the
                               //   source's own; only the pointer operand differs, and that operand is a value the
                               //   source itself computed. No flag, no guard-selected branch
// leaving the scope          // accept: I8 — on every feasible assignment both obligations are discharged
```

**Verdict: accepted as expected.** This is the strongest evidence in this part
for the candidate's seam choice (state on places, interference in windows): the
case is decided without any notion of which *name* owns what, because `St` is
keyed by place and the identity is allowed to be guarded. Gaps recorded: G2 (no
binding→identity structure and no `Own` join) and G3 (I7 not stated for a
locator or a guarded identity) — both are bookkeeping omissions the candidate's
own §3.21 rows presuppose, not design gaps.

---

## 6. B12 — two conditional releases separate safety from completion

Expected (CASES): REJECT at the second release; `if cond { release(a) };
if !cond { release(a) }` accepts when `cond` is unchanged.

### 6.1 The rejecting form

```text
a = object(10)                 // accept: I1 + I3 — ρa, owner binding `a`
if c { release(a) }            // accept (guard): II1 — atom c@1
                               // accept (then-arm): I7 on ρa, premises as B10 ⇒ St(ρa) := Gone, Own(a) consumed
                               // accept (join): II3 — St(ρa) = ite(c@1, Gone, Init); Own(a) consumed on the
                               //   c@1 = true leaf only (G2's leafwise reading)
if d { release(a) }            // accept (guard): II1 — atom d@1 enters Γg. II5 records a relation only for an
                               //   RHS of the form c, !c or a literal; `d` is an independent binding, so NO
                               //   relation between c@1 and d@1 is recorded and both atoms are free
                               // reject: II6(a) — the assignment c@1 = true ∧ d@1 = true is feasible, since
                               //   feasibility is "satisfies every recorded relation" and none constrains the pair.
                               //   On that assignment I7's premises fail twice:
                               //     missing fact St(ρa) ∈ {Init, Uninit} — it is Gone (R1(a)(ii))
                               //     missing fact Own(a) live and unconsumed — it was consumed at the first
                               //       release on this leaf (R3, R2)
                               //   M7: "prove ¬(c ∧ d), e.g. bind d = !c" — II6's own example, which names B12
                               // note: Gmax is not reached — two live atoms is depth 2 = Gmax; a third
                               //   independent conditional release would trigger II3's oldest-atom collapse (OD2)
```

CASES's `false,false` row ("no release; obligation still pending") is also
derivable: on that assignment `Own(a)` is live at the end of the fragment, which
I8 rejects at the enclosing scope exit — see B13.

### 6.2 The accepting variant

```text
if c { release(a) }            // accept: as above; St(ρa) = ite(c@1, Gone, Init)
if !c { release(a) }           // rule missing (G4): `!c` is an expression discriminant, which II1 says captures
                               //   nothing. II6's own example writes this exact pair and accepts it, and §3.21's
                               //   B12 row asserts the acceptance. Derived under the worked example's reading:
                               // accept (then-arm): the arm is ¬c@1, where St(ρa) = Init and Own(a) is live;
                               //   I7's premises hold ⇒ St(ρa) := Gone, Own(a) consumed on that leaf
                               // accept (safety): II6(a) — no feasible assignment ends ρa twice
                               // accept (completion): on both feasible assignments Own(a) is consumed exactly once,
                               //   so I8 at the scope exit has nothing outstanding — CASES's "c or d" requirement
                               // accept (M2): II6(b) — two source-written branches, two source-written releases;
                               //   nothing is selected by the checker's atom
```

**Verdict: accepted as expected** (rejection and variant both). G4 recorded: the
variant that separates safety from completion is exactly the one II1's written
premise does not admit.

---

## 7. B13 — conditional scope cleanup

Expected (CASES): no single verdict. The case states two policies as
alternatives — explicit consumption requires the `¬cond` path to discharge in
source; automatic cleanup would release A on that path, "logically
`if !original_cond { release(a) }`" — and records that this scalar model does
not establish that cleanup needs a runtime flag or a machine action.

```text
a = object(10)                 // accept: I1 + I3 — ρa, owner binding `a` with one end-obligation
if cond { release(a) }         // accept (guard): II1 — atom cond@1
                               // accept (then-arm): I7 on ρa ⇒ St(ρa) := Gone, Own(a) consumed on that leaf
                               // accept (join): II3 — St(ρa) = ite(cond@1, Gone, Init); Own(a) live on the
                               //   cond@1 = false leaf (G2's leafwise reading)
// leave the owning scope      // reject: I8 — its premise is "for every owner binding in the scope and every
                               //   feasible guard assignment, the obligation is discharged or moved out".
                               //   missing fact: discharge of Own(a) on the feasible assignment cond@1 = false
                               //   (II6(a)). R2. M7 repair, verbatim from I8's example: "release a on the ¬cond
                               //   path"; the model inserts nothing (M2(ii))
                               // the ground for refusing the OTHER policy: II6(b) — "no lowered branch, no
                               //   release ... may be selected by an atom. A rule whose discharge would need a
                               //   guard-selected action is a rejection with a named repair, never an inserted flag"
                               // rule missing (G5): II6(b)'s FORMAL criterion is "the set of operations executed
                               //   by an accepted program is a function of the source's own control flow". An
                               //   automatic release placed on the source's own `else` edge satisfies that
                               //   criterion — the source wrote the branch, and no flag or atom is consulted at
                               //   run time. II6(b)'s prose refuses it; its formal statement does not; no rule
                               //   says which governs. B13's refusal rests on the prose alone
```

The repaired form:

```text
a = object(10)
if cond { release(a) }
if !cond { release(a) }        // accept: B12.2's derivation exactly (G4); exactly one end on every feasible
                               //   assignment, and none left over
// leave the owning scope      // accept: I8 — Own(a) consumed on every feasible assignment
```

CASES's own observation that "later assignment to `cond` cannot change which path
needs cleanup" is delivered by II5's versioning: the repair must test `cond@1`,
and a `cond = …` between the two tests makes the second atom a different version
whose relation is unrecorded, which II6(a) then refuses (B09's mechanism).

**Verdict: differs from the expected verdict — deliberate refusal with a named
repair.** The named repair is the complementary guarded release
`if !cond { release(a) }`, which the rule set accepts (I8's second worked
example). The refusal is of the *automatic cleanup alternative* only: CASES B13
states two policies without selecting one, and this candidate selects explicit
consumption and refuses the other by name (§6's closing paragraph: "one defect
the rules deliberately do not repair"). It is not a capability loss — every
program CASES B13 contemplates has an accepted spelling — and the refused
alternative is a policy the case itself declines to require.

One qualification the derivation forces: the candidate's stated ground is
stronger than the case supports. CASES B13 says this scalar model "does not
establish that cleanup needs a runtime flag"; II6(b)'s prose refuses guarded
cleanup regardless, and its formal criterion would admit the edge-placed
lowering the case describes. The refusal is therefore a *rule* choice under M2's
prose reading, not a consequence measured from this fragment (G5).

---

## 8. P1 — container split with a runtime index (regression check)

Required: sequential legality by proof, not by borrow scope; parallel permission
for the two writes; a diagnostic at the read of `v[k]` that names the missing
fact; no runtime check.

### 8.1 Formation and the split

```text
(v, own_v) = vec_with(n)       // accept: I1 mints ρv (⊑-root) with an owner obligation
                               // accept: III4's establish — backing_of(ρv) = β, a COARSE name β ⊑ ρv with
                               //   extent_of(β) = [0, cap); β is first class, storable and nameable at a cut
                               // accept: III2's establish — len_of(ρv) = n, keyed on the DESCRIPTOR plane ρv.desc
                               // accept: III3's establish — cap_of(ρv), also on ρv.desc
let i = …; let j = …           // accept: III8 — i and j are captured index bindings at version 1. Without the
                               //   bindings, III8's kill clause and II11(3) would kill every element fact at the
                               //   first write through a runtime index
focus β as { A = β[i], B = β[j] }
      where { i < len_of(ρv), j < len_of(ρv), i != j } {
                               // accept: II14 — the entries are written at the head; PART checks the entry list
                               //   and names any obligation it cannot close. Here AFF cannot decide i != j for two
                               //   free bindings, so the writer WRITES it in the `where` clause (II14's own
                               //   example; OD12 prices it at one inequality for E = 2)
                               // accept: II14's effect — a carve node is pushed on Carve(β); one entry per listed
                               //   entry plus the complement entry \{A,B}, because the carve is partial;
                               //   St is UNTOUCHED — the window carries interference only
                               // NOTE (O7/O12 evidence): P1's `part_i`/`part_j` are the entry names A and B.
                               //   A binding pointing at one is typed ptr<β↓w.A,u64> by II19 and is SECOND CLASS:
                               //   ESC(i)–(v) forbid storing it in a field or container, returning it, capturing it
                               //   in a closure, passing it where a first-class pointer is expected, or using it
                               //   after the close. The parser refuses the type in a field, parameter or return
                               //   position. It may still be PASSED to a call, because II10 instantiates a formal
                               //   store name by a window entry with nothing fine written in any type
```

### 8.2 The two exclusive writes, sequentially and in parallel

```text
  write(A, x1)                 // accept: II15 — the resolved place β[i] meets entry A and lies provably within
                               //   exactly one entry. (This is II15's repaired form; the unrepaired rule rejected
                               //   every element access, §3's note on S-S3/C-S1)
                               // accept: I3 — u64 is a copy type, St(β[i]) ∈ {Uninit, Init} ⇒ St(β[i]) := Init
  write(B, x2)                 // accept: II15 (entry B) + I3
  write(A, x1) ∥ write(B, x2)  // accept overlap: II20 — the two footprints are {A}@W and {B}@W; for the one pair
                               //   they touch, DIS holds on every feasible assignment, discharged from the carve
                               //   partition PART established at the head (R5(a)(i))
                               // accept rejoin: II20 — the composed edge is the union of two edges on DISTINCT
                               //   keys β[i] and β[j], so the union is a function; no shared key is written
                               // accept: legality came from the written disjointness proof, not from a borrow
                               //   scope — P1's first required property, with the qualification in §8.5
```

### 8.3 The read at a runtime index — the required diagnostic

```text
  y = read(β[k])               // reject: II15 — β[k] meets A and meets B and lies provably within neither.
                               //   missing fact: "k != i && k != j" (AFF cannot close it; k is a free binding)
                               //   M7 payload, as II15 specifies it: rule II15, the window head's location, the
                               //   missing fact, and three repairs — name the entry, prove the access lies in the
                               //   complement, or move the access outside the window
  use k != i && k != j ;       // accept: II15's named application — an INV step over a written fact (owner O15(b)),
                               //   never a bare assume (M3). The step is written by the writer and checked, not
                               //   discovered
  y = read(β[k])               // accept: II15 — the place now lies provably within the complement entry \{A,B}
                               // accept (R11's index domain): k < len_of(ρv) must also be discharged; len_of
                               //   SURVIVED the two writes because II11(1)'s footprint is β[i], β[j] and III2's
                               //   kill clause spares an element write. This is the measured `len()` rebind tax
                               //   the rule removes (III2, R6)
```

The branch route P1 also names ("else the writer must branch") does **not**
derive:

```text
  let ok = (k != i && k != j)  // accept: II5 — a new boolean binding. Its RHS is neither `c`, `!c` nor a literal,
                               //   so II5 records NO relation and the atom is FREE
  if ok { y = read(β[k]) }     // reject: II15 — missing fact "k != i && k != j".
                               //   rule missing (G6): inside the arm the checker knows atom ok@1 = true and
                               //   nothing more. III12 makes the binding VERSION the atom; II12 binds an atom to a
                               //   predicate only for a checker-introduced arm; no rule turns a source-level
                               //   guard atom into the predicate its defining expression computes
```

So P1's read is admissible by exactly one route, the written `use`. That is a
cost, not a loss: the `use` step is erased (§1.4) and inserts no runtime check,
which is P1's fourth required property.

### 8.4 Return and push

```text
}                              // accept: II16 — pop the carve node, retire A and B; facts keyed by a retired entry
                               //   are dropped; St needs NO write-back, because state was never in the window.
                               //   Any span-typed binding is dead from here (II19's ESC(v))
                               // cost: II16 retires the WHOLE carve at once. There is no per-entry return, so
                               //   P1's "put both parts back" is one statement boundary, not two events
push(v, 5)                     // accept: I17's premises St(ρv) = Init, Carve(β) = ∅ and Span(β) = ∅ — the first
                               //   two hold only after the close (II16's own worked example ends on this line)
                               // accept (post-state): with no capacity proof, I17 introduces a fresh atom g# and
                               //   gives St(β) = ite(g#, Init, Gone), backing_of(ρv) = ite(g#, β, β'). Nothing in
                               //   P1 reads an element pointer afterwards, so no obligation is owed on either arm
push(v, 5)   // written INSIDE the focus body
                               // reject: I17 — missing fact Carve(β) = ∅. M7 names the open window head.
                               //   This is P1's "must be allowed only after both are back", delivered by I17's
                               //   structural premise rather than by a borrow scope
```

### 8.5 The required properties, line by line

| P1 requires | Delivered by | Cost / qualification |
|---|---|---|
| sequential legality by proof, not by borrow scope | II14's `where` obligations + PART + AFF; II15's within-one-entry test | the *permission* is still delimited by a lexical `focus` statement (II16 retires the entries at the close). What is proof-based is which accesses are legal inside it; what is scope-based is how long the exclusivity lasts |
| the two writes may run in parallel | II20 overlap + rejoin, `DIS` from the carve partition | none beyond the written `i != j` |
| a diagnostic at `v[k]` naming the missing fact | II15's stated M7 payload (rule, head, fact, three repairs) | — |
| no runtime check | II6(b), §1.4's erasure table | the only admitting route is the written `use` (G6 kills the branch route) |
| `part_i`, `part_j` as values | II19 | **second class**: `ptr<β↓w.A,u64>` cannot be stored, returned or captured. OD7 records that an element of a non-byte container has no promotion-with-merge, so this is a boundary, not a spelling problem. A byte-extent split (`split_at_mut`) would instead get coarse names by I11 and be first class |

**Verdict: accepted with a stated cost** — the cost being that the two parts are
window entries (second class, window-scoped, retired together), that the `v[k]`
read requires a written `use` step, and that the source-branch alternative the
program names is not derivable (G6).

---

## 9. P3 — graph with back edges in a pool (regression check)

Required: link updates through several paths to one node are legal sequentially;
removal invalidates every path to `m`; parallel data writes are permitted by
distinctness of nodes; the invariant "next/prev point to live nodes" is stated
once and reused.

### 9.1 The pool and its one written invariant

```text
P : Pool<Node>                 // accept: I1 mints ρP (⊑-root) with an owner obligation; the slots are PLACES
                               //   ρP.slots[t] under §1.1's place grammar, and their fields ρP.slots[t].next etc.
invariant Links :              // accept: III1's establish(c) and III10 — occupancy is the pool's OWN written data;
  forall k. Occupied(ρP,k) =>  //   Occupied(ρP,k) is a term over that data, not a checker structure. The checker
    St(ρP.slots[k]) = Init &&  //   maintains no free list and no bitmap, so M2(ii) has nothing to object to
    Occupied(ρP, next_of(k)) &&
    Occupied(ρP, prev_of(k))
                               // accept: "stated once" ✓ — this is the only introducer besides a formation's or a
                               //   row's ensures (III1). Every USE is one written INV step; the checker never
                               //   searches for one (owner O15(b)). That is the reuse cost: written, not free
```

### 9.2 Insert n between a and b — four link writes through several paths

```text
let ka = slot_of(a); let kb = slot_of(b)
                               // accept: III8 — captured index bindings. Without them III8's precision cliff and
                               //   II11(3) apply: a transition at an uncaptured runtime index kills
                               //   Occupied(ρP, ·) for EVERY index, and the invariant would have to be re-derived
                               //   from scratch after the first write
match free_slot(P) {           // accept: I19 + III10's re-derive — the writer's own match on the writer's own data
  Some(kn) => {
    put(ρP.slots[kn], move n)  // accept: I19 — premise St(ρP.slots[kn]) = Uninit comes from the ARM (the match is
                               //   on the pool's own occupancy data); I6 sinks the value, St := Init, and n's
                               //   obligation transfers from the binding to the place
    set ρP.slots[ka].next = kn // accept: I3 on the place ρP.slots[ka].next; its premise St(…) ∈ {Uninit, Init} is
                               //   discharged by ONE INV step against Links at ka, given Occupied(ρP, ka)
                               // accept (the case's first requirement): I3 has NO uniqueness or exclusivity
                               //   premise. St is keyed by the place (§1.2), so any number of handles or stored
                               //   ptr<ρP,Node> values naming that slot may write it in sequence. No window is
                               //   open, and II14 is the only thing that creates interference
    set ρP.slots[kn].prev = ka // accept: I3
    set ρP.slots[kn].next = kb // accept: I3
    set ρP.slots[kb].prev = kn // accept: I3
                               // accept (frame): II11(1) — each footprint is ONE field place, so facts about the
                               //   other slots survive; II11(2)'s containment kill does not fire because the
                               //   footprint is not a parent of them; II11(3) does not fire because ka, kb, kn are
                               //   captured bindings (III8)
    ensures Occupied(ρP,kn) && Links
                               // accept: I19's written ensures re-establishes the invariant in one INV step
  }
  None => { /* the writer's outcome arm */ }
                               // accept: I19 — an R12 arm; R2 is discharged on it because n's obligation is still
                               //   held by the binding on that arm
}
```

### 9.3 Remove m, and reuse its storage

```text
let m = slot_of(h)             // accept: III8 — a captured index binding at version 1
x = remove(P, m)               // accept: I20 — premise St(ρP.slots[m]) = Init by one INV step against Links;
                               //   row { ρP.slots[m] : Init ⇒ Uninit @W ; ensures !Occupied(ρP,m), Links }
                               //   effect: St(ρP.slots[m]) := Uninit; the element's obligation moves to x (R2 is
                               //   now owed by the binding, and I8 will require it)
read(deref(pm).data)           // reject: missing fact St(ρP.slots[m]) = Init (R1(a)(i)). Because the state is on
                               //   the PLACE, this refuses EVERY path to m at once — no path-by-path invalidation
                               //   is needed, which is P3's second requirement at this granularity (I20's example)
insert(P, y) reusing m         // accept: I19 — St(ρP.slots[m]) := Init again
read(deref(pm).data)           // accept ON SAFETY, and it reads y. III11: there is no generation, no epoch and no
                               //   version cell; the slot's storage never ended, so R1(a)(i) and (ii) are both
                               //   satisfied and nothing rejects. DECLARED: a wrong-occupant logic error for which
                               //   no diagnostic exists — open defect OD1, owner O1/O3
                               // ⇒ P3's "removal invalidates every path to m" is delivered up to the refill and
                               //   NOT past it. Storage that genuinely ends (I7's heap object, I11's arena block)
                               //   is refused by Gone; a type-stable reusing pool is where R1(a)(ii) has nothing
                               //   to say
```

### 9.4 Traversal — and why the links must be handles

```text
let t = next_of(a)             // accept: III8 — t is a captured index binding (version 1). Its VALUE is unknown;
                               //   what the rule needs is that its TERM is fixed
d = read(ρP.slots[t].data)     // accept: St(ρP.slots[t]) = Init by one written INV step against Links instantiated
                               //   at t = next_of(ka), given Occupied(ρP, ka) (III1's re-derive; III10)
                               // cost: one WRITTEN INV application per traversal step. The invariant is stated
                               //   once; its instantiation is never searched for (O15(b))
loop { t = next_of(t)
       d = read(ρP.slots[t].data) }
                               // accept only with a WRITTEN head row: II7(c) — a fact must survive the back edge
                               //   (Occupied(ρP,t)) and III8's kill retires t's version each iteration
                               //   head row { exists t. Occupied(ρP, t) }
                               // accept: II7 checks initiation and consecution and nothing else — no fixpoint, no
                               //   widening, no iteration count (LOOPCHK)
p = next_ptr_of(a)             // accept: p : ptr<ρP, Node> — a coarse, FIRST-CLASS pointer (II19), so P3's links
                               //   may be stored in the Node struct as ordinary fields
read(deref(p).data)            // reject: COVER (§5) resolves the place to ρP's slots key, not to one slot: the
                               //   pointer's type names the store ρP and no rule gives the index. COVER returns
                               //   the meet of the keys it meets, and after any remove (or with any free slot) that
                               //   meet includes Uninit. missing fact: St = Init at the resolved key
                               // ⇒ derived constraint: P3's "handle-or-pointer" must be a HANDLE (an index binding,
                               //   III8) for the traversal read to derive. A stored coarse pointer crosses cuts and
                               //   can be written through, but it does not resolve to a slot, so it cannot be READ
                               //   through unless a window (II14 `focus ρP as { S = .slots[h] }`) or an index
                               //   binding supplies the place
```

### 9.5 Parallel map over the nodes

```text
par_for k in 0..cap_of(ρP) {
  focus ρP as { E = [∀k ∈ 0..cap] } {
    set E.data = f(read E.data) } }
                               // accept: II14's parametric entry form + III9 — dis(E@k, E@k') is proved ONCE per
                               //   binder by the affine map (1,0) (PAR-2), O(1), not the O(E²) PART pays for an
                               //   explicit element list (OD12)
                               // accept overlap: II20 — the per-iteration footprint is the resolved path E.data,
                               //   and DIS holds for k != k' on every feasible assignment
                               // accept rejoin: II20 — the per-iteration edges are on distinct keys; their union
                               //   is a function
                               // accept (frame): II11(1) — the footprint is the `.data` field only, so the link
                               //   fields and Links' support survive the map (R6). This is what lets the pool's
                               //   invariant outlive a parallel data write
                               // reject (the premise the program must still supply): read E.data needs
                               //   St(ρP.slots[k].data) = Init for EVERY k in the space, and a pool with free
                               //   slots has Uninit ones. missing fact: Occupied(ρP,k) per k.
                               //   repairs: iterate an index space the invariant makes Init (a dense pool, or a
                               //   written live range), or carry Occupied(ρP,k) in the parametric entry's `where`
par_for over the link order { set ρP.slots[next_of(t)].data = 1 }
                               // reject the OVERLAP, not the program: III9 — the map t ↦ next_of(t) is not affine
                               //   in the binder, so no dis is provable. The construct is erasable ⇒ the program
                               //   stands and runs sequentially (M5)
```

### 9.6 The required properties, line by line

| P3 requires | Delivered by | Cost / qualification |
|---|---|---|
| link updates through several paths to one node, legal sequentially | `St` keyed by place (§1.2) + I3 with no uniqueness premise; no window is open | indices must be captured bindings (III8), or II11(3) kills the invariant's support at the first write |
| removal invalidates every path to `m` | I20 + place-keyed state: one `Uninit` refuses every path at once | **only until the slot is refilled.** III11 declares no generation; after I19 reuses the slot a stale handle reads the new occupant with no diagnostic (OD1) |
| parallel data writes permitted by distinctness | II14's parametric entry + III9's affine map + II20 | the iteration space must be the index space (not the link order), and every `k` in it must be `Occupied` by the written invariant |
| the invariant stated once and reused | III10 + III1: one written invariant; every use is one `INV` step | reuse is a *written* step at each use — never searched, never inferred (M3, O15(b)) |

**Verdict: accepted with a stated cost** — three costs: OD1's wrong occupant
after slot reuse (the one required property not fully delivered, and declared as
such), one written `INV` step per traversal read plus a written loop-head row,
and a parallel map that must run over the index space with an occupancy fact per
element.

---

## 10. Summary table

| Item | CASES / PROGRAMS expectation | This candidate | Deciding rules | Verdict |
|---|---|---|---|---|
| **B07** | `read(p)` REJECT; `write(p,20); read(p)` accept; `put`/`replace` refused on their own premises | same, verbatim in I3's worked example | II1, II2, II3, I4, **I3**, I6, I5, II6(a) | accepted as expected (G1, G7) |
| **B08** | ACCEPT (20 / 10) | same; the second `if` reuses the same atom version, so the arms line up | II1 ("if not already live"), II2, I6, II3's `ite(c,x,x)→x` | accepted as expected (G1, G7) |
| **B09** | REJECT; the `if !cond { put }` variant accepts | same; `cond@2 = ¬cond@1` recorded, guarded values keep `cond@1` | II5, II1, II6(a), I3, I6, III12 | accepted as expected (G1, G4) |
| **B10** | the join retains one obligation per path; no intersection, no omission | same; `St` guarded on one atom, `Own` guarded leafwise | II3, I7, II11(1), II6(a), II6(b) | accepted as expected (G2, G8) |
| **B11** | both lines ACCEPT; no obligation remains | same; guarded identity selects the leaf for both the read and the end | II2's guarded identity, II3, I7, II6(a)/(b) | accepted as expected (G2, G3) |
| **B12** | REJECT at the second release; `if c / if !c` accepts | same; `c ∧ d` is feasible because no relation is recorded | II1, II5, II6(a), I7, R3 | accepted as expected (G4) |
| **B13** | two policies stated as alternatives, neither selected | explicit consumption selected; automatic cleanup refused at the exit | I8, II6(b), II3 | **differs: deliberate refusal with a named repair** — write `if !cond { release(a) }` (G5) |
| **P1** | proof-based legality, parallel writes, a naming diagnostic, no runtime check | all four hold; the parts are second-class window entries; the `v[k]` read has exactly one admitting route | II14, II15, II20, I3, I17, II16, II19, III2, III8, `PART`, `AFF`, `INV` | accepted with a stated cost (G6) |
| **P3** | multi-path links, removal invalidates every path, parallel node writes, one reused invariant | three of four hold; the fourth holds only until the slot is refilled | I1, I19, I20, I3, II11, III8, III9, III10, III1, II7, II14, II20, `COVER` | accepted with a stated cost (OD1; no new gap) |

### Rule gaps recorded, not improvised around

| # | Item(s) | Missing or conflicting rule |
|---|---|---|
| G1 | B07, B08, B09 | I4's premise (affine or linear class) contradicts I4's and I3's own worked examples, which `take` from a copy-scalar slot and cite CASES B07 by name. Under the premise, the `take` line of all three cases has no rule |
| G2 | B10, B11, B12, B13 | II3 joins `St` and `Facts` only. `Own`'s consumption state has no join rule, and §1.2 lists no structure at all mapping an ordinary pointer binding to its identity, although II2 asserts a guarded one |
| G3 | B11 | I7 is stated for an owner binding at a coarse name; ending through a locator, or at a guarded identity `ite(c, ρb, ρa)`, is unstated |
| G4 | B09 variant, B12 variant | II1 says an expression discriminant captures nothing; II5's, II6's and §3.21's treatment of `if !c` requires it to be checked under `¬atom(c)` |
| G5 | B13 | II6(b)'s prose ("no release selected by an atom") and its formal criterion ("the operation set is a function of the source's own control flow") disagree about an else-edge-placed cleanup. B13's refusal rests on the prose |
| G6 | P1 | No rule turns a source-level guard atom into the predicate its defining expression computes; II5 explicitly leaves such an atom free. P1's "else the writer must branch" route is therefore not derivable — only the written `use` step is |
| G7 | B07, B08 | I3 and I6 `KILL` contents facts and establish no new one (only I5 does), so the cases' value claims are not derivable; their acceptance verdicts are |
| G8 | B10, B11 (also P1, P3) | `DIS` between two distinct `⊑`-roots has no stated ground, and R4(b)/C3 forbid grounding it on name inequality |

---

## 11. What this part contributes to the owner's open decisions

**O11 — are condition terms admitted at joins.** This part is the corpus segment
that lives or dies on the answer, and under this candidate's bounded form
(II3's `ite` over captured boolean *binding versions*, depth ≤ `Gmax`) all seven
branch cases derive. Specifically:

- B07, B08 and B09 are decided by *which version* the atom is (II1's "if not
  already live", II5's version-and-relation rule). Under an equality join they
  are losses of precision: B07's variant would still accept (I3 is total), but
  B08's `put` would lose its premise and B09's variant would have no way to say
  "the taken path".
- B10 and B11 are decided by the *guarded obligation* and the *guarded
  identity*. Under an equality join B10's obligation state meets to nothing
  usable and B11's `remaining` has no derivable target; those two cases are the
  sharpest O11 evidence in the set.
- B12's rejection is decided by II6(a)'s feasibility over *recorded relations
  only* — equalities and negations, union-find, no solver. That is what keeps
  O11(b) inside M1: the atoms divide facts, and the only question ever asked of
  them is whether an assignment satisfies recorded `=`/`¬` relations.
- The price appears in three places this part can name: `Gmax` is reached at two
  live independent atoms (B12 sits exactly at depth 2; a third independent
  conditional release triggers II3's oldest-atom collapse, OD2), a guard atom is
  free of its defining predicate (G6, which is what costs P1 its branch route),
  and `if !c` needs a rule II1 does not state (G4).

**O7 / O12 — are second-class projections and never-stored references an
acceptable permanent boundary.** This part supplies three pieces of evidence:

- *For* the boundary: B11 needs no projection at all. `remaining` is an ordinary
  first-class `ptr<ρ,u64>` whose identity is guarded, and the whole case is
  decided by place-keyed state. Nothing in B07–B13 wants a stored reference.
- *Against* it, at P1: `part_i` and `part_j` are window entries. Bound to
  pointers they are `ptr<β↓w.A,u64>`, second class by II19's `ESC`, retired
  together by II16, and refused by the parser in every cut position. OD7 records
  that an element of a non-byte container has no promotion-with-merge, so this
  is the boundary itself and not a spelling problem. What the candidate
  *recovers* relative to an unrepaired window model is the byte-extent case:
  I11's promotion gives `split_at_mut` coarse names with proved extents, which
  are first class and cross cuts.
- *Against* it, at P3: a stored coarse `ptr<ρP,Node>` is first class and
  writable through, but `COVER` resolves it only to the pool's slots key, so it
  cannot be *read* through without a window or an index binding. The practical
  consequence is that P3's links must be handles. That is O12 evidence of a
  different kind from II19's: even the first-class pointer buys less than it
  looks like it buys, because the place — not the pointer — carries the state.

**M1 / M2.** Nothing in this part's accepted derivations introduces a search, a
budget or an iteration count: every acceptance is one syntax-directed pass plus
`GUARD`'s union-find over at most `Gmax` atoms, `AFF`/`EXT` at P1's head, and
written `INV` steps at P3. Nothing in the accepted programs materializes a
checker structure: B13 is the one place where an inserted action would be
needed, and the rule set refuses it by name rather than inserting it — subject
to G5, which is the one place where the candidate's M2 argument rests on prose
its own formal criterion does not carry.


# File: derive-brand-context-bounded-boundary.md

# Derivation — candidate `brand-context-bounded`, part **boundary**

Source of rules: `core2/rules-brand-context-bounded.md` (§2 `ATOM`/`TERM`/`KEY`,
§3 rule set I, §4 rule set II, §5 rule set III, §6 judgment families). Every
accept cites the rule that licensed it; every reject names the missing fact.
Items: `PROGRAMS.md` P4, P7, P8; `CASES.md` L01–L06.

Style follows `MECHANISM-MAP.md` §3: a pseudocode line, then
`// accept: fact used` or `// reject: missing fact`.

No candidate is selected here. Where a line is not covered by any rule in the
document it is marked **rule missing** and the derivation continues.

---

## P4 — Cursor over a growing vector

Program (PROGRAMS.md P4): a stored pointer/handle to a container in a struct;
`c.read()`, `push(v,5)`, `c.read()` again, then `free(v); c.read()`.
Required: validity decided by state and proof, **not** by a borrow that blocks
`push`.

### P4.a The declaration (forced before any line can be written)

```text
struct Cursor<'v> { at: ptr<'v, Vec<'b>>, i: u64 }
                      // reject: I-19. 'b is free in the type of field `at`.
                      //   missing fact: 'b is not a parameter of Cursor.
                      //   repair (I-19): add the parameter and the where-clause
struct Cursor<'v,'b> { at: ptr<'v, Vec<'b>>, i: u64 } where backing('v) = 'b
                      // accept: I-19, the identity row is closed at arity 2
```

### P4.b The body

```text
(pv, ov) = vec_new<u64>()
                      // accept: I-1. exists 'v. st('v)=Uninit, own('v)=held,
                      //   obl('v)=none, owns('v)={}
init(pv)              // accept: I-6, st('v) ≠ Gone, u64 is copy -> st('v):=Init.
                      //   the container's representation invariant supplies
                      //   backing('v)='b and RSt('b[0..len), Init, {})  (III-8, III-12)
push(pv, 10)          // accept: I-15 arm, assumed for setup; len('v) becomes 1
c = Cursor<'v,'b>{ at: pv, i: 0 }
                      // accept: I-19 (row closed) + II-13. A projection/pointer is a
                      //   sub-identity NAME, not a token: nothing is consumed, the
                      //   base stays writable. This is the O7/O12 boundary NOT taken
let n0 = len(v)       // accept: III-1 FREEZE. bridge fact len('v)=n0; frozen n0 (ATOM)
let c0 = cap(v)       // accept: III-1 FREEZE + III-3. bridge cap('v)=c0; frozen c0
if 0 < n0 {
  a = c.read()        // accept: I-2/I-6 read of the field 'c.at (st('c)=Init);
                      //   deref at 'b[0]: JF-QI instantiates RSt('b[0..n0), Init, {})
                      //   at index 0, side condition 0 < n0 discharged by JF-ENT from
                      //   Γ={0<n0}.  (III-12, JF-QI, JF-ENT)
}
```

Without the writer's `if 0 < n0`, the read is:

```text
a = c.read()          // reject: R11 via III-12/JF-QI. missing fact: c.i < len(v).
                      //   repair (III-1): `let n0 = len(v)` and branch. M3 deletes the
                      //   `use i < len` route: `use R(args)` is a NAMED rule with
                      //   checked premises, and no rule proves a scalar inequality (§6)
```

The `push` and the second read — the whole point of P4:

```text
push(v, 5)            // accept: I-16. requires st('v)=Init ✓, backing('v)='b ✓,
                      //   len('v)=n0 ✓, cap('v)=c0 ✓, RSt('b[0..n0), Init, {}) ✓.
                      //   ensures: len('v)=n0+1;
                      //            backing('v) = ite(n0<c0, 'b, 'b2)   (III-8, TERM: 1 atom)
                      //            st('b)  = ite(n0<c0, Live, Gone)
                      //            st('b2) = ite(n0<c0, Gone, Live)
                      //            RSt(backing('v)[0..n0+1), Init, {})
                      //   ends ite(n0<c0, none, 'b);  writes 'v,'b
                      //   NOTE: the push is NOT blocked by the live Cursor. D4/II-13:
                      //   no suspension, no borrow. P4's required property, met
b = c.read()          // reject: the Cursor's type pins 'b, so the deref is at 'b[0],
                      //   and st('b) = ite(n0<c0, Live, Gone) (I-16 ensures clause 3).
                      //   missing fact: n0 < c0. site: push, ensures clause 3 (M7)
```

The accepted shape is the writer's own branch (I-16's second example):

```text
if n0 < c0 {
  push(v, 5)          // accept: I-16, and II-1 pushes n0<c0 onto Γ, so JF-TERM
                      //   collapses backing('v) to 'b and st('b) to Live
  b = c.read()        // accept: Γ={n0<c0} (II-1) collapses the term (JF-TERM);
                      //   st('b)=Live, and RSt(backing('v)[0..n0+1), Init, {}) gives
                      //   st('b[0])=Init by JF-QI with 0 < n0+1 from JF-ENT
} else {
  push(v, 5)          // accept: I-16; here Γ={¬(n0<c0)} collapses st('b) to Gone and
                      //   backing('v) to 'b2; I-13 ended 'b
  b2 = read(v, 0)     // accept: the access is written through `v`, i.e. at
                      //   backing('v)[0] = 'b2[0]; RSt gives Init (III-12, JF-QI).
                      //   the CURSOR is dead on this arm and the writer must not use it
  // c = Cursor<'v,'b2>{ at: pv, i: 0 }
                      // rule missing: the candidate states no rule for rebuilding a
                      //   stored nominal at the new identity inside the arm; I-19 gives
                      //   the declaration-site row, III-8 gives the new backing, but no
                      //   rule re-establishes `where backing('v) = 'b` for a LIVE value
}
```

The `reserve` route, which would look like the natural repair, does **not** work,
and the document contradicts itself about it:

```text
reserve(v, n0+1)      // accept: I-17. ensures backing('v)=ite(c0>=n0+1,'b,'b2),
                      //   st('b)=ite(c0>=n0+1, Live, Gone), cap('v)>=n0+1
push(v, 5)            // accept: I-16's atom `n0 < c1` from cap('v)=c1 >= n0+1 by
                      //   JF-ENT difference-bound closure (III-3)
b = c.read()          // reject: I-17's own example. st('b) is still the RESERVE's term
                      //   ite(c0 >= n0+1, Live, Gone). missing fact: c0 >= n0+1
                      // CONFLICT: III-3's example marks this same line `accept`,
                      //   composing st('b) from the push alone. I-17 + §7 ("the cap
                      //   proof belongs BEFORE the reserve") govern; III-3's example is
                      //   wrong. Recorded, not repaired here
```

`free` and the final read:

```text
free(ov)              // accept: I-4. own('v)=held (constant), st('v) ≠ Gone,
                      //   I-5 vacuous (u64 carries no obligation).
                      //   I-13: owns('v) = {backing('v)} — the enumerated container
                      //   element-backing edge — so st('b[π]):=Gone on BOTH leaves of
                      //   backing('v) (III-8's worked example)
c.read()              // reject: R1(ii). st('b[0]) = Gone, site: free at the line above,
                      //   through the owns('v) ∋ 'b edge (I-13). Permanent: I-4 says no
                      //   rule sets a Gone path to any other state.  P4's third
                      //   required property, met
```

**M1 / M2.** Families used: `JF-ENT` (`O(n³)`, Floyd–Warshall, fixed rounds),
`JF-QI` (one instantiation at the access index), `JF-TERM` (depth ≤ 1),
`JF-OVL` (literal index 0 here, so `O(d)`). All specified, terminating,
budget-free → M1 met. Every branch on this path (`if 0 < n0`, `if n0 < c0`) is
written by the writer; no generation counter, no drop flag, nothing compared at
run time → M2(ii) met for P4.

**Verdict P4: accepted with a stated cost.** The cost, itemized:
(1) `Cursor<'v,'b>` — two identity parameters plus a `where`, not one (I-19);
(2) two `FREEZE` lets (`n0`, `c0`) and one writer branch `if n0 < c0` with a
non-empty else arm, because M3 deletes every `use`-route to a scalar fact;
(3) the else arm cannot use the cursor at all and the rules do not say how to
rebuild it (**rule missing**, below);
(4) the `reserve` repair is unusable: I-17 leaves its own conditional `ends 'b`
on the cursor's identity, so the proof must precede the reserve, which is
vacuous.
P4's three required properties (validity across `push` by proof; `push` not
blocked by the stored pointer; `free(v)` then `c.read()` refused) are all
derivable.

**Rule missing (P4):** no rule states whether a nominal's `where backing('v)='b`
is a construction-site premise only, or a live obligation re-checked after
`JF-KILL` kills the `backing` fact at the `push` (III-8 kills it; I-19 covers
only the declaration site). Consequence: a `Cursor` value that survives a
reallocating `push` has an unsupported where-clause and no rule says whether it
may still be passed to a callee that requires the clause.

**Rule conflict (P4):** I-17's worked example rejects `read(pe)` after
`reserve; push`; III-3's worked example accepts the identical line. Same
candidate, same program, two verdicts.

---

## P7 — Take, put, replace through aliases

```text
(p, oA) = alloc<Box<T>>()
                      // accept: I-1. exists 'A. st('A)=Uninit, own('A)=held, obl none
put(p, move box0)     // accept: I-8. st('A)=Uninit, mayEq('A)={'A} (KEY: no symbolic
                      //   path live) -> strong fill. st('A):=Init; obl('A):=one
q = p                 // accept: §1.1, ptr<'A,T> is copyable and has no mode, no
                      //   duration, no region. origin(q)='A, a constant (II-14 needs
                      //   no term: one arm)
v = take(p)           // accept: I-7. st('A)=Init after KEY resolution (mayEq={'A}) and
                      //   JF-TERM evaluation (the term is the constant Init).
                      //   strong update st('A):=Uninit; obl('A):=none; the value leaves
                      //   with its obligation (R3)
read(q)               // reject: R1(i). st('A)=Uninit, site: take through p on the line
                      //   above. KEY: the fact is keyed on 'A, so every name of 'A sees
                      //   the hole — no must-alias analysis is needed (I-7's example).
                      //   P7's first required property, met
put(q, move v)        // accept: I-8. st('A)=Uninit ✓, mayEq('A)={'A} ✓ -> strong fill.
                      //   st('A):=Init; obl('A):= the moved value's = one.
                      //   Filling through the OTHER alias is I-8's own example.
                      //   P7's second required property, met
read(p)               // accept: st('A)=Init (I-8). val('A) is the value put through q
old = replace(p, new) // accept: I-9. st('A)=Init ✓, mayEq('A)={'A} ✓. st unchanged;
                      //   obl transfers from 'A to the binding `old`; val('A) killed
                      //   and reset to `new`
read(q)               // accept: I-9's example. val is keyed on 'A, so the read through q
                      //   sees `new`.  P7's third required property, met
free(oA)              // reject: I-5. obl('A)=one (the content `new` carries a release
                      //   obligation) and st('A)=Init. missing: the value must leave
                      //   first. repair named by I-5: `take(p)` or `replace(p,·)`
```

The accepted tail, with the two extra lines I-5 and R2 force:

```text
drop_box(old)         // accept: II-12/R2. `old` took 'A's obligation at the replace and
                      //   must be discharged on this path
w = take(p)           // accept: I-7. st('A):=Uninit; obl('A):=none
drop_box(w)           // accept: R2/R3, the taken value's obligation discharged
free(oA)              // accept: I-4. own('A)=held (constant) ✓; st('A)=Uninit ≠ Gone ✓;
                      //   I-5 satisfied (st('A)=Uninit, the value has left).
                      //   Effect: st(π):=Gone for every π ≤ 'A, permanently;
                      //   own('A):=discharged; every fact supported under 'A killed
read(p)               // reject: R1(ii). st('A)=Gone, site: free on the line above.
                      //   Permanent by I-4 ("no rule anywhere sets a Gone path to any
                      //   other state"): address reuse never revives it (I-23, III-6).
                      //   P7's fourth required property, met
free(oA)              // reject: R2. own('A)=discharged, site: free above (I-4).
                      //   The diagnostic distinguishes this (R2, released twice) from
                      //   `take(p)` twice (R3, used twice) — R3's distinguishability
                      //   requirement, met
```

**M1 / M2.** No symbolic index appears, so `JF-OVL` is `O(d)` and `JF-ENT` is
never called with a non-empty Γ. No branch, no bookkeeping state, no runtime
comparison → M2(ii) met.

**Verdict P7: accepted with a stated cost.** All four required properties are
derivable and the state is a property of the storage `'A`, not of `p` or `q`
(KEY). The cost is two lines the program's sketch does not write: with an
**affine** content (P7 says "affine value"), I-5 forbids `free A` while
`st('A)=Init`, so the writer must take-and-discharge first, and II-12/R2 force
the discharge of `old` from the `replace`. With a copy content both lines
vanish and the program is accepted exactly as written.

---

## P8 — Conditional release and loop exits

### P8.a `if c { free(a) } else { free(b) }`, then use the survivor, then free it

```text
(pa, oa) = alloc(); (pb, ob) = alloc(); init both
                      // accept: I-1 + I-6. st('A)=st('B)=Init; own both held
if c { free(oa) } else { free(ob) }
                      // accept: II-1 pushes c / ¬c; I-4 in each arm; II-2 each arm exits
                      //   with CONSTANTS under its own Γ; II-3 joins with the atom c
                      //   available (c is the construct's own guard atom, frozen by
                      //   III-7/ATOM):
                      //     st('A)=ite(c,Gone,Live)      own('A)=ite(c,discharged,held)
                      //     st('B)=ite(c,Live,Gone)      own('B)=ite(c,held,discharged)
                      //   TERM: one atom per identity per point ✓. II-5 keeps the four
                      //   terms correlated on the same atom.
                      //   M2(ii): the terms are ERASED checker state — no drop flag,
                      //   no bit, nothing lowered.  P8's first required property, met
use(pa); free(oa)     // reject: R2 via II-3. own('A) is not the constant `held`:
                      //   it is discharged on the leaf c. missing fact: a branch on c.
                      //   site named: the then-arm's free.  M7 names the path whose
                      //   state differs — P8's third required property, met
```

The accepted shape is the writer's branch (II-3's example):

```text
if c { use(pb); free(ob) } else { use(pa); free(oa) }
                      // accept: II-1 collapses every term inside each arm (JF-TERM under
                      //   Γ={c} resp. Γ={¬c}): on the c leaf own('B)=held, st('B)=Live;
                      //   on the ¬c leaf own('A)=held, st('A)=Live. I-4 in each arm.
                      //   At the join own('A) and own('B) are BOTH the constant
                      //   `discharged` (II-3 row 1), and st('A)=st('B)=Gone
}                     // accept: I-3 at scope exit. own is the constant `discharged` for
                      //   every identity minted in the scope
```

### P8.b `loop { if stop { break }; take(p); ...; put(p) }; free(p)`

```text
(p, oA) = alloc(); init
                      // accept: I-1 + I-6. st('A)=Init, own('A)=held
repeat n {
                      // accept: II-6. entry edge Init/held; back edge (below) Init/held;
                      //   the II-3 join with no atom available is the CONSTANT Init.
                      //   Nothing written at the head — §1.4's zero row for P8's loop
  if stop { break }   // accept: II-9. the break edge carries its ACTUAL state
                      //   (st('A)=Init, own held) to the exit join; the head invariant
                      //   is not required on it
  v = take(p)         // accept: I-7. Γ={¬stop}; st('A)=Init -> Uninit, mayEq={'A}
  put(p, move v)      // accept: I-8. st('A)=Uninit -> Init, strong (mayEq={'A})
}                     // accept: II-8 back edge. the head fact st('A)=Init is entailed by
                      //   JF-TERM equality (both constants)
free(oA)              // accept: II-9 exit join over {zero-iteration: Init/held,
                      //   normal-completion: Init/held, break: Init/held} = the constant
                      //   Init/held. I-4: own held ✓, st ≠ Gone ✓; I-5 vacuous (v copy).
                      //   Exactly one free on every path including n = 0
```

**M1 / M2.** The only atoms are the writer's `c` and `stop`; `TERM`'s one-atom
bound is never approached; no family is invoked with a symbolic index. M2(ii)
is met by construction: II-3's terms and II-9's join are erased checker state
and the only lowered branches are `if c` and `if stop`, both written by the
writer. **No drop flag exists** — P8's stated requirement.

**Verdict P8: accepted as expected.** All three required properties
(no runtime drop flag; the writer's branches carry the state; the rejection
names the path whose state differs) are derivable. One boundary to record for
the owner: if the source omits the else-arm free and relies on scope cleanup,
I-3 rejects (`own` is not the constant `discharged`) — §7 classifies that as a
deliberate refusal with the repair "write the guarded release" (case B13); P8
as written does not rely on it.

---

## L01 — Take and restoration preserve the next iteration's entry condition

```text
a = object(10)        // accept: I-1 mints 'A (st:=Uninit, own:=held, obl:=none), the
                      //   initializer is I-6 -> st('A):=Init
p = ref(a)            // accept: §1.1, ptr is copyable. origin(p)='A, a constant
q = p                 // accept: II-14. copying a binding copies the ORIGIN TERM; here
                      //   the term is the constant 'A, so origin(q)='A
repeat n {
                      // accept: II-6. entry edge st('A)=Init; back edge st('A)=Init
                      //   (established below); II-3 row 1 joins s with s -> the CONSTANT
                      //   Init. own('A)=held on both edges -> constant held.
                      //   Head state is a constant, so NOTHING is written (§1.4's zero
                      //   row for L01). No atom, no invariant, no φ-value needed
  v = take(p)         // accept: I-7. st('A)=Init after KEY resolution (mayEq('A)={'A}:
                      //   no symbolic path is live) and JF-TERM (constant).
                      //   strong update st('A):=Uninit; obl('A):=none
  put(q, move v)      // accept: I-8. st('A)=Uninit ✓; mayEq('A)={'A} ✓ -> strong fill.
                      //   st('A):=Init. The fill through the OTHER locator is I-8's own
                      //   example; the fact is keyed on 'A, not on p or q (KEY)
}                     // accept: II-8. the head facts st('A)=Init, own('A)=held are
                      //   entailed at the back edge by JF-TERM equality (constants)
read(p)               // accept: II-9 exit join over {zero-iteration edge: Init,
                      //   normal-completion edge: Init} = the constant Init (II-3 row 1).
                      //   No break edge exists. Reads 10 — including n = 0, because the
                      //   zero-iteration edge is one of the joined edges
```

Tail not written in the case (the case omits unrelated cleanup): at the
enclosing `}`, I-3 requires `own('A)` to be the constant `discharged`, so a
written `release(p)` is needed to close the scope. That is case B13's policy
selection, not a change to L01's stated verdict.

**M1 / M2.** No atom is ever created; `Γ` is empty throughout; `JF-TERM` only
compares constants. Nothing is compared at run time (M2(ii) met). The head
needs no fixpoint iteration beyond one join, so M1 is trivially met here.

**Verdict L01: accepted as expected.** The case's ACCEPT, including `n = 0`, is
derivable with zero written proof steps.

---

## L02 — An empty exit state becomes the next iteration's input

```text
a = object(10)        // accept: I-1 + I-6. st('A)=Init, own('A)=held
p = ref(a)            // accept: §1.1. origin(p)='A
repeat n {
                      // reject: II-6. entry edge st('A)=Init; back edge st('A)=Uninit;
                      //   no atom is available at a loop head (II-6: the II-3 join of E
                      //   and B with no atom unless a written invariant names one over a
                      //   head-frozen φ-value). II-3 row 3: Init ⊔ Uninit with the atom
                      //   unavailable = Live. Live is not a constant, so the head state
                      //   is neither a constant nor a term over a stated φ-atom.
                      //   missing fact: st('A) at the head is not constant.
                      //   named repairs (II-6): restore before the back edge (that is
                      //   L01), or write `invariant st('A) = Live` — and then the take
                      //   below rejects
  v = take(p)         // (under the written `invariant st('A) = Live`:)
                      // reject: I-7. the premise is st(π)=Init after KEY + JF-TERM;
                      //   Live is strictly above Init in §1.2's lattice.
                      //   missing fact: st('A) = Init
}
read(p)               // not reached; had the loop been accepted under the `Live`
                      //   invariant, this rejects too: R1(i), missing fact st('A)=Init
```

The case's second clause — "if n is known to be at most one, there is no
repeated-take error":

```text
repeat n where n <= 1 { v = take(p) }
                      // rule missing: no rule binds `repeat n`'s iteration counter as a
                      //   head φ-value, and no rule states the entry/exit relation
                      //   between that counter and n. II-6 admits a head term
                      //   ite(α, s1, s2) only when α is an atom over a head φ-value
                      //   stated in a written invariant; the candidate never says the
                      //   loop counter is such a value, so `ite(k = 0, Init, Uninit)`
                      //   is not writable. The n ≤ 1 refinement is therefore not
                      //   expressible; the loop rejects for every n
```

**M1 / M2.** The rejection is produced by one II-3 table lookup at the head; no
search, no budget (M1 met). No runtime state is introduced to recover the
program (M2(ii) met — the candidate refuses rather than inserting a flag).

**Verdict L02: accepted as expected.** The case's verdict for unrestricted `n`
is REJECT and the candidate rejects, naming the identity, the back edge and two
repairs (M7). The rejection lands at the **loop head** rather than at "the
second take"; the defect named is the same and the case's reasoning is the same.
**Rule missing:** the `n ≤ 1` sub-case of the case is not expressible (no
loop-counter φ-value rule), so the candidate rejects where the case says there
is no repeated-take error.

---

## L03 — Relative initialization invariant while physical roles alternate

```text
a = object(10)        // accept: I-1 + I-6. st('A)=Init, own('A)=held
b = object(20)        // accept: I-1 + I-6. st('B)=Init, own('B)=held
old_b = take(b)       // accept: I-7. st('B):=Uninit, mayEq('B)={'B} -> strong;
                      //   the taken integer is a copy value (R3), obl('B) was none
p = ref(a)            // accept: origin(p)='A (constant)
q = ref(b)            // accept: origin(q)='B (constant)
repeat n {
  invariant origin(p) # origin(q)
        and st(origin(p)) = Init
        and st(origin(q)) = Uninit
                      // accept: II-7. Without this written invariant the head is
                      //   REJECTED by II-6: the entry edge has origin(p)='A and the back
                      //   edge origin(p)='B, so II-3's origin row with no atom available
                      //   joins them to the SET {'A,'B}, and st('A)=Init ⊔ Uninit = Live,
                      //   st('B)=Live — neither a constant nor a stated φ-term.
                      //   With it, II-7 admits the relational head state whose per-name
                      //   physical role alternates. Entry establishes it: origin(p)='A
                      //   # origin(q)='B by JF-PATH on two distinct minted identities,
                      //   st('A)=Init ✓, st('B)=Uninit ✓ (the pre-loop take(b))
  v = take(p)         // accept: II-7's own worked example. The head relation gives
                      //   st(origin(p)) = Init, which is I-7's premise
  put(q, move v)      // accept: II-7's worked example. st(origin(q)) = Uninit is I-8's
                      //   premise, and origin(p) # origin(q) gives mayEq(origin(q)) =
                      //   {origin(q)} for the strong fill (KEY)
                      // note: II-15 row 2 would REFUSE this line if origin(q) were read
                      //   as an origin SET {'A,'B} ("a write through an origin set never
                      //   sets Init"). II-7's relational reading governs by its example
  tmp = p; p = q; q = tmp
                      // accept: II-14. copying a binding copies the origin term; the
                      //   swap exchanges the two terms
}                     // rule missing: II-8 requires every head fact to be entailed at
                      //   the back edge "by JF-TERM equality plus JF-ENT for the
                      //   invariant's atoms". The head facts here are RELATIONAL
                      //   (st(origin(p)) = Init), and no rule in §3–§5 states how a
                      //   statement UPDATES a relational fact keyed on origin(x):
                      //   I-7/I-8 update st(π) at a path, JF-ORG resolves an origin term
                      //   or set to leaves, and II-7 asserts "the body maps the relation
                      //   to itself" without a rule that computes it. The entailment is
                      //   asserted by II-7's example, not derived
read(p)               // accept: II-7's worked example. st(origin(p)) = Init from the
                      //   head relation, which holds on the normal-completion edge and
                      //   on the zero-iteration edge (entry established it). Reads 10
read(q)               // reject: R1(i). st(origin(q)) = Uninit from the head relation.
                      //   missing fact: st(origin(q)) = Init.  matches the case
read(a)               // reject: the fact is relational. missing fact: origin(p) = 'A
                      //   (II-7's own diagnostic).  matches the case
release(p); release(q)
                      // accept: I-4 twice. own('A) and own('B) are both the constant
                      //   `held` (untouched by the body: only st and origins move);
                      //   st ≠ Gone on both; I-5 vacuous (copy contents, obl none), so
                      //   the EMPTY slot's obligation is consumed too.  matches the case
```

**M1 / M2.** The invariant's conjuncts are atoms over identity paths (`#`) and
state heads, all frozen; `JF-ENT` sees no scalar. Nothing is compared at run
time; the alternation is carried by checker state that is erased (M2(ii) met).

**Verdict L03: accepted with a stated cost** — one written loop invariant with
three conjuncts (II-7). This matches §7's classification: writer cost, not a
capability loss; the case derives the same relation by hand and says it is "not
yet a selected inference algorithm", and this candidate makes the writer state
it. **Rule missing:** the update/entailment rule for relational facts keyed on
`origin(x)` (II-7 supplies the head form, II-8 the obligation, and II-15 row 2
points the other way; no rule computes `st(origin(p))` after a statement).

---

## L04 — A release on a break edge need not preserve the loop-head state

```text
a = object(10)        // accept: I-1 + I-6. st('A)=Init, own('A)=held
p = ref(a)            // accept: origin(p)='A
repeat n {
                      // accept: II-6. entry edge Init/held; the ONLY back edge is the
                      //   fall-through path below, which carries Init/held. II-3 row 1
                      //   joins to the constant Init/held. Nothing written (§1.4's zero
                      //   row for L04's body)
  if stop {
    release(p)        // accept: I-4. Γ={stop} (II-1); own('A)=held is a constant here;
                      //   st('A)=Init ≠ Gone; I-5 vacuous (copy content).
                      //   st('A):=Gone permanently; own('A):=discharged
    break             // accept: II-9. the break edge carries its ACTUAL state
                      //   (Gone/discharged) to the exit join and the head invariant is
                      //   NOT required on it.  The case's central claim, met
  }
  read(p)             // accept: on this path Γ={¬stop} (II-1), the release did not
                      //   execute, st('A)=Init. "the release path cannot reach this
                      //   read" is the fall-through arm's Γ, not a liveness test
}                     // accept: II-8. the head facts Init/held are entailed on the back
                      //   edge (the break edge is not a back edge)
read(p)               // reject: II-9 exit join over {zero-iteration: Init/held,
                      //   normal-completion: Init/held, break: Gone/discharged} with no
                      //   atom available: II-3 row 4 gives st('A) = Live ⊔ Gone = ⊤.
                      //   missing fact: no state is known for 'A (R1). The ⊤ carries the
                      //   widening operation (§1.3, M7 payload).  matches the case
release(p)            // reject: R2 via the same join. own('A) = held ⊔ discharged with
                      //   the atom unavailable.  matches the case
}                     // reject at the enclosing scope exit: I-3. own('A) is not the
                      //   constant `discharged`. §7: the case defers exit policy to B13,
                      //   and B13's selection here is explicit consumption — the repair
                      //   is to write the guarded release, which is exactly L05
                      // CONFLICT, recorded: II-3 row 5 says the OWN JOIN ITSELF rejects
                      //   when the atom is unavailable ("R2 definiteness"), which would
                      //   refuse the fragment at the loop exit; II-9's worked example
                      //   instead produces own('A) = ⊤ and defers the rejection to I-3.
                      //   §7 endorses II-9's reading, which is the one used above
```

**M1 / M2.** No atom survives the loop exit, by construction; `⊤` is produced by
a lattice join, not by a budget (M1 met). No flag is inserted to recover the
program: the candidate refuses and names L05's shape as the repair (M2(ii) met).

**Verdict L04: accepted as expected.** All three of the case's claims are
derivable: the body is safe, the break edge need not re-establish the head
state (II-9), and an unconditional post-loop `read(p)` or `release(p)` rejects
(`⊤`). The enclosing scope exit additionally rejects under I-3 — the case
explicitly assigns exit policy to B13, and §7 applies B13's refusal-and-repair
here, so this is not a difference from the case's stated verdicts.

---

## L05 — A checked Boolean relation can guard later iterations after release

```text
a = object(10)        // accept: I-1 + I-6. st('A)=Init, own('A)=held
p = ref(a)            // accept: origin(p)='A
active = true         // accept: III-7 + III-1 FREEZE. frozen value a0; atom `a0 = true`
                      //   recorded in the boolean part of JF-ENT
repeat n {
  invariant ( active ==> st('A)=Init and own('A)=held)
        and (!active ==> st('A)=Gone and own('A)=discharged)
                      // accept: II-6 + TERM. The two implications are the canonical
                      //   one-atom terms st('A)=ite(a_h, Init, Gone) and
                      //   own('A)=ite(a_h, held, discharged), with a_h the head φ-value
                      //   of `active`. II-6 admits a head term when its atom is over a
                      //   head φ-value AND the written invariant states it — both hold.
                      //   Entry: JF-ENT has a0 = true, JF-TERM evaluates both terms at
                      //   the true leaf -> Init/held ✓
  if active {
                      // accept: II-1 pushes a_h onto Γ; JF-TERM collapses st('A) to Init
                      //   and own('A) to held. This is GCR: entering the arm strictly
                      //   shrinks the terms
    read(p)           // accept: st('A)=Init under Γ
    if stop {
                      // accept: II-1 pushes stop; Γ = {a_h, stop}
      release(p)      // accept: I-4. own('A)=held is constant under Γ; st('A)=Init ≠
                      //   Gone; I-5 vacuous. st('A):=Gone; own('A):=discharged
      read(p)         // (the case's rejected variant, inserted here:)
                      // reject: R1(ii). st('A)=Gone, site: the release on the line
                      //   above. Γ still contains a_h, but I-4's effect is a fact about
                      //   'A, not about the variable. "release has invalidated the old
                      //   state implication" — the case's claim, met
      active = false  // accept: III-7. FREEZE binds a1; the relation `a1 = false` is
                      //   recorded as an ordinary boolean atom
    }
    // inner join (II-3): st('A) = ite(stop, Gone, Init),
    //                    own('A) = ite(stop, discharged, held),
    //                    and the boolean relation a1 = ite(stop, false, true), i.e.
    //                    a1 = ¬stop
  }
                      // rule missing: II-2 (arm exit) requires each identity's exit
                      //   state to be a constant or a term over an atom already in the
                      //   ENCLOSING Γ. Leaving the `if active` arm, st('A) and own('A)
                      //   carry the atom `stop`, which is in no enclosing Γ (the loop
                      //   body's Γ is empty), so II-2 rejects as written. The
                      //   derivation needs the term RE-KEYED from `stop` to the
                      //   invariant's boolean a1 (the two are related by a1 = ¬stop,
                      //   which JF-ENT's step 4 records) — and NO rule performs that
                      //   re-keying. JF-TERM's three operations are evaluate under Γ,
                      //   join by the II-3 table, and equality by normalization; none
                      //   substitutes an entailed-equivalent atom for a term's atom.
                      //   II-8's worked example nonetheless marks this exact program
                      //   `accept`
}                     // rule missing (second gap, at II-8/II-9): the head fact is a term
                      //   over the NEXT iteration's φ-value, and the back-edge state is
                      //   a term over `stop`; II-8 says "JF-TERM equality plus JF-ENT
                      //   for the invariant's atoms" without stating a leafwise
                      //   entailment check between two terms over DIFFERENT atoms
if active {
                      // accept if the loop head is reached: II-1 pushes the exit
                      //   φ-value; JF-TERM collapses own('A) to `held` on this leaf
  release(p)          // accept: I-4. own('A)=held constant under Γ; st('A)=Init ≠ Gone
}                     // accept: II-3 joins own('A) = discharged (this arm) with
                      //   own('A) = discharged (the ¬active leaf, already discharged in
                      //   the body) -> the CONSTANT discharged. Exactly one release on
                      //   every path, including n = 0
                      // rule missing (third gap): the loop EXIT join (II-9) is II-3 over
                      //   {zero-iteration edge, normal-completion edge}. The
                      //   zero-iteration edge carries the CONSTANTS Init/held; the
                      //   normal-completion edge carries the head TERMS. II-3's table
                      //   has no row for joining a constant with a term (nor two terms
                      //   over one atom); every row is constant-vs-constant
```

Variant the case names — removing `active = false`:

```text
repeat n { invariant (…) ; if active { read(p); if stop { release(p) } } }
                      // reject: II-8 back edge. the head fact (a_h ==> st('A)=Init and
                      //   own('A)=held) fails on the path where the release ran and the
                      //   boolean was not updated. missing fact: st('A)=Init on that
                      //   path; the diagnostic names the head fact and the release
                      //   statement that broke it (II-8).  matches the case
```

**M1 / M2.** `active` is an ordinary source Boolean: III-7 freezes it, and the
two branches on it are the writer's. The checker inserts no liveness test and
the invariant is erased (§1.5), so M2(ii) is met **if** the program is accepted
at all. M1 is unaffected by the gaps: the missing operation (atom re-keying)
would itself be a deterministic rewrite, not a search.

**Verdict L05: underivable — rule missing.** Three gaps, in the order they bite:
(1) no rule re-keys a state term's atom from the inner guard `stop` to the
invariant's boolean `a1` under the recorded relation `a1 = ¬stop`, so II-2
rejects the `if active` arm exit as written;
(2) II-8's back-edge check is not stated for two terms over different atoms;
(3) II-3's join table has no constant-vs-term row, which the loop exit join
needs.
II-8's worked example asserts this exact program is accepted "including n = 0",
so the candidate **intends** the case's ACCEPT with a stated cost (one written
two-implication invariant). If the three rules are supplied, L05 is *accepted
with a stated cost*; as the document stands it is not derivable line by line.

---

## L06 — A hole may leave through break when the continuation only releases it

```text
a = object(10)        // accept: I-1 + I-6. st('A)=Init, own('A)=held, obl('A)=none
                      //   (the content is an ordinary copy integer, CASES §Scope)
p = ref(a)            // accept: origin(p)='A
repeat n {
                      // accept: II-6. entry edge Init/held; back edge (after the put)
                      //   Init/held; II-3 row 1 -> the constant Init/held. Nothing
                      //   written at the head
  v = take(p)         // accept: I-7. st('A)=Init ✓; mayEq('A)={'A} (KEY: no symbolic
                      //   path live) -> strong update st('A):=Uninit; obl('A):=none
  if stop { break }   // accept: II-9. Γ={stop} (II-1); the break edge carries its ACTUAL
                      //   state st('A)=Uninit, own('A)=held to the exit join, and the
                      //   head invariant is not required on it. "There is no obligation
                      //   to fill the hole merely to execute break" — the case's claim,
                      //   met by II-9
  put(p, move v)      // accept: I-8. Γ={¬stop}; st('A)=Uninit ✓; mayEq('A)={'A} ✓ ->
                      //   strong fill, st('A):=Init
}                     // accept: II-8. the head facts Init/held are entailed at the back
                      //   edge by JF-TERM equality (constants)
release(p)            // accept: II-9 exit join over {zero-iteration: Init/held,
                      //   normal-completion: Init/held, break: Uninit/held} with no atom
                      //   available. II-3 row 3: Init ⊔ Uninit = **Live** — §1.2's
                      //   lattice change, introduced precisely so that this line is
                      //   derivable. own is `held` on all three edges -> constant.
                      //   I-4: own held ✓, st('A)=Live ≠ Gone ✓;
                      //   I-5: the content type carries no obligation, obl('A)=none ✓
                      //   (I-5 would REFUSE this on Live for an obligation-carrying
                      //   content — the case says the taken value is a discardable
                      //   integer and that non-copy cleanup needs its own accounting,
                      //   so the two agree exactly).  The case's ACCEPT, met
read(p)               // (the case's rejected variant, inserted before the release:)
                      // reject: R1(i). st('A) = Live, not Init; Live means the storage
                      //   exists with occupancy unknown (§1.2).
                      //   missing fact: st('A) = Init on the break exit.  matches the case
```

**M1 / M2.** One II-3 table lookup per edge; no atom, no `JF-ENT` call. The
`Live` element is a lattice value, not a runtime tag; nothing is compared at run
time (M2(ii) met).

**Verdict L06: accepted as expected.** Both the ACCEPT and the rejected variant
are derivable, and the accept depends on the four-element refinement of §1.2
(`Live` below `⊤`) that this candidate introduces for exactly this case.

---

## Summary

| Item | Verdict | Cost / difference | Rules missing or in conflict |
|---|---|---|---|
| **P4** | accepted with a stated cost | `Cursor<'v,'b>` (2 identity parameters + `where`, I-19); two `FREEZE` lets and one writer branch `if n0 < c0` with a non-empty else arm (M3 deletes every `use`-route to a scalar fact); the else arm cannot use the cursor; the `reserve` repair is unusable (I-17 leaves its own conditional `ends 'b`). All three required properties derivable; the stored pointer does **not** block `push` (II-13, D4) | **missing:** whether a nominal's `where backing('v)='b` is a construction-site premise or a live obligation re-checked after III-8's kill; no rule rebuilds a stored nominal at the new backing inside the reallocating arm. **conflict:** I-17's example rejects the post-`reserve; push` read, III-3's example accepts the same line |
| **P7** | accepted with a stated cost | With an affine content, I-5 forbids `free A` while `st('A)=Init` and II-12/R2 force the discharge of `old` from the `replace`: two lines the sketch omits. With a copy content, accepted exactly as written. State is a property of `'A`, not of `p`/`q` (KEY) | — |
| **P8** | accepted as expected | No drop flag (II-3's terms are erased checker state, M2(ii)); the naive `use(a); free(a)` rejects naming the then-arm's free (M7); the loop's exit join is the constant Init | — (B13's scope-cleanup refusal is outside P8 as written) |
| **L01** | accepted as expected | Zero written proof steps; head state is a constant by II-3 row 1; `n = 0` covered by the zero-iteration edge | — |
| **L02** | accepted as expected | REJECT as the case states, at the **loop head** (II-6, `Init ⊔ Uninit = Live` is not a constant) rather than at "the second take"; two named repairs | **missing:** no rule binds `repeat n`'s iteration counter as a head φ-value, so the case's `n ≤ 1` refinement is inexpressible and rejects too |
| **L03** | accepted with a stated cost | One written loop invariant with three conjuncts (II-7); §7's classification (writer cost, not capability loss) confirmed line by line. Post-loop `read(p)` accepts, `read(q)` and `read(a)` reject, `release(p); release(q)` accepts | **missing:** no rule updates a relational fact keyed on `origin(x)`; II-15 row 2 would refuse the `put` if `origin(q)` resolved to a set, and II-7 governs only by its worked example |
| **L04** | accepted as expected | Break edge carries its actual state (II-9); post-loop `read`/`release` reject on `⊤`; the enclosing scope exit rejects under I-3, which the case assigns to B13 | **conflict:** II-3 row 5 rejects the `own` join itself when the atom is unavailable; II-9's example produces `⊤` and defers to I-3. §7 endorses II-9 |
| **L05** | underivable: rule missing | The candidate *intends* the case's ACCEPT with one written two-implication invariant (II-8's worked example names L05), but the arm exit is not derivable | **missing (3):** (1) no atom re-keying from the inner guard `stop` to the invariant's boolean `a1` under `a1 = ¬stop`, so II-2 rejects the `if active` arm exit; (2) II-8's back-edge check is unstated for two terms over different atoms; (3) II-3's join table has no constant-vs-term row, which the loop exit join needs |
| **L06** | accepted as expected | ACCEPT and the rejected `read(p)` variant both derivable; depends on §1.2's `Live` element, introduced for this case. I-5 refuses the same line for an obligation-carrying content, which is exactly the case's own caveat | — |

Differences from `CASES.md`'s expected verdicts, classified: none is a
capability loss; none is a correction of a case; L03's written invariant and
P4's branch-plus-two-parameters are writer cost; L04's enclosing-scope-exit
refusal is case B13's deliberate refusal with the repair "write the guarded
release" (that repair is L05, which is itself not derivable today).


# File: derive-brand-context-bounded-straight-and-early-branches.md

# Derivation — candidate `brand-context-bounded`, part `straight-and-early-branches`

Items: `CASES.md` S01–S07 and B01–B06. Date 2026-09-16.

Source of rules: `core2/rules-brand-context-bounded.md` only (`ATOM`, `TERM`,
`KEY`; rule set I `I-1`…`I-24`; rule set II `II-1`…`II-16`; rule set III
`III-1`…`III-12`; judgment families `JF-*`). No rule is improvised: a line the
candidate's rules do not cover is marked **rule missing** and the derivation
continues. Requirement tags (`R1(i)`, `R1(ii)`, `R2`, `R3`, `R6`) are the
diagnostic names the candidate's own rules and examples use; they are cited as
diagnostics, not as extra rules.

Style follows `MECHANISM-MAP.md` §3: one pseudocode line, then
`// accept: fact used` or `// reject: missing fact`.

## Reading conventions used throughout

These four readings are fixed once here so every derivation below can cite them
by name instead of re-deriving them.

- **`a = object(10)`** is `I-1` (mint `'A`; `st('A) := Uninit`;
  `own('A) := held`; `obl('A) := none`; `owns('A) := {}`; `'A # 'B` recorded
  *from the allocator's written `ensures`*, never from name inequality) followed
  by the initializer store `I-6` (`st('A) := Init`). The case's "one recorded
  disposal obligation" is `own('A) = held`, not `obl('A)`: `obl` is the
  *content* obligation of `I-5`, and a copy integer carries none.
- **`p = ref(a)`, `q = p`** bind `origin(p)`, `origin(q)`. `§1.3` holds
  `origin(v)` for every pointer-valued binding; `§1.1` says a path other than a
  signature `'a` "is formed by the checker from the access"; `II-14` fixes that
  *copying a binding copies the origin term*. There is **no numbered rule** for
  forming or copying a pointer to a live identity — see the gap list.
- **`read(p)`** resolves `origin(p)` by `JF-ORG`, resolves the fact at the
  resulting path by `KEY` ("read of a fact at `π`: the value is `⊓` over
  `mayEq(π)`"), evaluates the resulting term under `Γ` by `JF-TERM`, and
  requires the constant `Init`. `R1(i)` names the failure for `Uninit`,
  `R1(ii)` for `Gone`. There is **no numbered rule** for `read` itself; `I-1`,
  `I-4`, `I-7`, `II-5`, `II-15` use this premise in their examples.
- **`release(p)` / `release(a)`** is `I-4` (premises `own = held` constant after
  `JF-TERM` under `Γ`; `st ≠ Gone`; `I-5`'s obligation guard), reached through
  `JF-ORG` when the argument is a locator. `II-14`'s own worked example uses
  `release(p)` on a term origin, so permissive locator release is inside the
  candidate, not an extension of it.

---

## S01: Sequential aliases and stable copies

```text
a = object(10)      // accept: I-1 mints 'A, st('A):=Uninit, own('A):=held, obl none;
                    //   I-6 stores the initializer, st('A):=Init
b = object(20)      // accept: I-1 mints 'B, I-6 -> st('B)=Init.
                    //   'A # 'B recorded by I-1 from the allocator's ensures (never from
                    //   name inequality, R4(b))
p = ref(a)          // accept: origin(p) := 'A, a singleton origin (§1.3).  [rule missing:
                    //   no numbered rule forms a pointer to a live identity]
q = p               // accept: II-14, copying the binding copies the origin term;
                    //   origin(q) := 'A
write(p, 11)        // accept: I-6. JF-ORG resolves origin(p) to the singleton 'A;
                    //   KEY gives mayEq('A) = {'A} -> strong update; st('A) ≠ Gone ✓;
                    //   u64 is copy so I-6's obligation clause is vacuous; st('A) := Init.
                    //   JF-KILL kills every fact whose support may-overlap 'A
read(q)             // accept: JF-ORG -> 'A; KEY read over mayEq = {'A}; JF-TERM gives the
                    //   constant Init; R1(i) satisfied.
                    //   the value 11 is NOT supported by a stated rule [rule missing: no
                    //   rule installs val on a write]
p = ref(b)          // accept: origin(p) := 'B. rebinding the name changes no fact on 'A
write(p, 21)        // accept: I-6 at 'B, strong (mayEq('B)={'B}); st('B) := Init.
                    //   JF-KILL's footprint is {'B}; 'A # 'B, so no fact on 'A is killed
read(q)             // accept: origin(q) is still 'A (II-14: the copy holds the term, not
                    //   the name p); st('A) = Init
read(p)             // accept: origin(p) = 'B, st('B) = Init
```

The case's "no exclusive interval is needed" is `D4` (unrestricted sequential
aliasing) in the candidate's base; no rule in set I or II suspends writability
of an aliased identity, and `II-13` states the same for projections.

**Verdict: accepted as expected.**

---

## S02: Replacement preserves storage and transfers the old value

```text
a = object(10)      // accept: I-1 + I-6, st('A)=Init, own held
p = ref(a)          // accept: origin(p) := 'A
q = p               // accept: II-14 copy of the origin term
old = replace(p, 20)
                    // accept: I-9. premise 1 st('A) = Init ✓ (JF-TERM constant, Γ = ∅);
                    //   premise 2 mayEq('A) = {'A} ✓ (KEY: no other live path with this
                    //   base). effect: st unchanged Init; the old value leaves with its
                    //   obligations, obl transfers to `old` (none, u64 is copy, R3);
                    //   val('A) killed
read(q)             // accept: I-9's own example — "reads `new`. val is keyed on 'A, so the
                    //   read through q sees it: no must-alias analysis (R6)".
                    //   st('A) = Init satisfies R1(i)
read(a)             // accept: the owner name is another name of the same identity 'A;
                    //   the fact is on 'A (I-4's phrasing: "one fact refuses every path:
                    //   the fact is on 'A"), so it accepts for the same reason
read(old)           // accept: `old` is an SSA scalar binding, a frozen value (§1.1), not
                    //   storage; no st fact governs it. I-9 gave it the departed value
```

"Old value-dependent facts about A expire" is exactly `I-9`'s `val(π)` kill plus
`JF-KILL`. "This does not establish preservation for references into future
subobjects" is outside the item.

**Verdict: accepted as expected.**

---

## S03: A hole is a property of the target, not one locator

```text
a = object(10)      // accept: I-1 + I-6
p = ref(a)          // accept: origin(p) := 'A
q = p               // accept: II-14
v = take(p)         // accept: I-7. premise st('A) = Init after KEY resolution and JF-TERM
                    //   evaluation ✓; mayEq('A) = {'A} -> strong update st('A) := Uninit;
                    //   obl('A) := none; the value leaves with its obligations (R3)
read(q)             // reject: R1(i). KEY resolves origin(q) = 'A, JF-TERM gives the constant
                    //   Uninit. missing fact: st('A) = Init.
                    //   diagnostic (I-7's example): "'A holds no value; site: take at L4
                    //   through p" — the hole is on 'A, visible through every name of 'A
```

Stated variants:

```text
read(a)             // reject: same fact on 'A, reached by the owner name. R1(i)
take(q)             // reject: I-7's premise st = Init fails; diagnostic R3 ("used twice"),
                    //   which I-7 distinguishes from R2 ("never released")
```

**Verdict: accepted as expected** (reject at the stated line).

---

## S04: Another alias can restore the hole

```text
a = object(10)      // accept: I-1 + I-6
p = ref(a)          // accept: origin(p) := 'A
q = p               // accept: II-14; origin(q) := 'A
v = take(p)         // accept: I-7, strong (mayEq = {'A}); st('A) := Uninit; obl := none
put(q, move v)      // accept: I-8. premise 1 st('A) = Uninit ✓; premise 2 mayEq('A) = {'A},
                    //   so this is a strong fill, not the rejected weak one;
                    //   effect st('A) := Init; obl('A) from the moved value = none (copy)
read(p)             // accept: origin(p) = 'A, st('A) = Init.
                    //   the value 10 is NOT supported by a stated rule [rule missing: I-8's
                    //   effect names st and obl, never val]
```

"Copying the locator while A is empty also works" — `q = p` is `II-14`'s term
copy and has no `st` premise; no rule in set I or II attaches a state premise to
copying a pointer binding.

**Verdict: accepted as expected.**

---

## S05: A hole need not be restored before its storage ends

```text
a = object(10)      // accept: I-1 + I-6; own('A) = held
p = ref(a)          // accept: origin(p) := 'A
v = take(p)         // accept: I-7; st('A) := Uninit; obl('A) := none; the value leaves
                    //   with its obligations (R3)
release(a)          // accept: I-4. premise 1 own('A) = held, and it is the CONSTANT held
                    //   (Γ = ∅, no term ever formed) ✓; premise 2 st('A) = Uninit ≠ Gone ✓;
                    //   premise 3 is I-5: 'A's content type u64 carries no release
                    //   obligation, and obl('A) = none anyway ✓.
                    //   effect: st(π) := Gone for every π ≤ 'A, permanently;
                    //   own('A) := discharged; I-13 propagates over owns('A) = {} (empty);
                    //   every fact supported under 'A is killed
read(v)             // accept: v is a value binding, not a path; I-4 kills facts SUPPORTED
                    //   under 'A, and v's support is not 'A — I-7 moved the value out
                    //   before the end. R3's "a move transfers a value with its
                    //   obligations" is what keeps it from a second disposal
```

Note that `I-5` is the rule that would refuse this if the content type carried an
obligation, and it accepts here for the stated reason (`st = Uninit`, "the value
has left"). The case's "non-copy content obligations are deferred" is `I-5`'s
subject and is not exercised by this item.

**Verdict: accepted as expected.**

---

## S06: Replacement requires old content at its commit

```text
a = object(10)      // accept: I-1 + I-6
p = ref(a)          // accept: origin(p) := 'A
q = p               // accept: II-14; origin(q) := 'A
v = take(q)         // accept: I-7 through q; KEY: mayEq('A) = {'A}, strong;
                    //   st('A) := Uninit. the hole is on 'A, not on q
old = replace(p, move v)
                    // reject: I-9's first premise. KEY resolves origin(p) = 'A and reads
                    //   st('A) = Uninit (JF-TERM constant).
                    //   missing fact: st('A) = Init.
                    //   diagnostic R1(i) with the site of the transition that set it
                    //   (§1.3: every state leaf carries its transition site) = take at L4
                    //   through q
```

Stated variant:

```text
put(p, move v)      // accept: I-8, st('A) = Uninit ✓, mayEq = {'A} -> strong fill
```

The case's accompanying point — "operand evaluation that performs a take must
not leave an earlier initialization premise authoritative" — is about
intra-statement evaluation order. As written the take is a *separate statement*,
so the derivation never needs it; **no rule in set I, II or III fixes operand
evaluation order within one statement** [rule missing], so the general form of
the case's point is not derivable here even though this instance is.

**Verdict: accepted as expected.**

---

## S07: Release is permanent and affects every access path

```text
a = object(10)      // accept: I-1 + I-6; own('A) = held, st('A) = Init
p = ref(a)          // accept: origin(p) := 'A
q = p               // accept: II-14; origin(q) := 'A
release(p)          // accept: JF-ORG resolves the singleton origin 'A, then I-4:
                    //   own('A) = held constant ✓; st('A) = Init ≠ Gone ✓; I-5 vacuous for
                    //   u64 content ✓. effect: st(π) := Gone for every π ≤ 'A, PERMANENTLY
                    //   ("no rule anywhere sets a Gone path to any other state");
                    //   own('A) := discharged; I-13 over owns('A) = {}; facts under 'A killed.
                    //   permissive locator release is II-14's own example form
read(q)             // reject: R1(ii). origin(q) = 'A; st('A) = Gone (site: release at L4
                    //   through p). missing fact: st('A) = Init.
                    //   I-4's example states the principle: "one fact refuses every path:
                    //   the fact is on 'A"
```

Stated variants, each replacing the last line:

```text
read(a)             // reject: R1(ii), the same fact on 'A reached by the owner name
write(q, 7)         // reject: I-6's premise st(π) ≠ Gone fails. a write cannot revive dead
                    //   storage — I-4's "permanently" clause is what forbids it
take(q)             // reject: I-7's premise st = Init fails (Gone). R1(ii)
release(q)          // reject: I-4's premise own('A) = held fails; own('A) = discharged
                    //   (site: release at L4). diagnostic R2, as in I-4's example line 4
release(a)          // reject: identical — the own fact is on 'A, not on a name
```

"Copying or discarding an inert locator is allowed without accessing A": `II-14`
copies the origin term with no `st` premise, and no rule attaches an obligation
to discarding a `ptr` (it is copyable, `§1.1`). "Later allocation/reuse is not
part of this case" — `I-23`/`III-6` would govern it and are not reached.

**Verdict: accepted as expected.**

---

## B01: One locator can have alternative targets

```text
a = object(10)      // accept: I-1 + I-6; st('A) = Init, own('A) = held
b = object(20)      // accept: I-1 + I-6; 'A # 'B from the allocator's ensures (I-1)
if cond { p = ref(a) } else { p = ref(b) }
                    // accept: III-7 freezes the guard's value as c0; ATOM's operand rule is
                    //   satisfied (c0 is an SSA scalar, frozen, immortal).
                    //   II-1 arm entry: Γ = {c0} in the then arm, {¬c0} in the else arm;
                    //   every term in scope is evaluated under the new Γ (none exist yet).
                    //   II-2 arm exit: st('A), st('B), own('A), own('B) leave both arms as
                    //   the SAME constants (Init, Init, held, held) — no inner atom, so
                    //   TERM/II-2 does not bite.
                    //   II-3 last row + II-14: origin(p) := ite(c0, 'A, 'B); the atom is
                    //   available because c0 is the construct's own guard atom.
                    //   TERM: one atom, depth 1, leaves distinct ✓
read(p)             // accept: JF-ORG on a term origin -> II-15 row 1, evaluated leafwise by
                    //   II-5. leaf c0: origin 'A, st('A) = Init ✓ (R1(i)).
                    //   leaf ¬c0: origin 'B, st('B) = Init ✓.
                    //   both leaves accept, so the access accepts. no branch is lowered:
                    //   the term is erased checker state (§1.5)
```

"The analysis represents both executions, not one guessed runtime choice" is
`II-3`'s origin row plus `II-5`'s leafwise evaluation; `KEY` is not involved
because each leaf is a distinct base identity, not a may-equal class.

**Verdict: accepted as expected.**

---

## B02: Target and initialization must remain correlated

```text
a = object(10)      // accept: I-1 + I-6
b = object(20)      // accept: I-1 + I-6; 'A # 'B by I-1
if cond {
    p = ref(a)      // accept: in-arm origin 'A (Γ = {c0})
    old = take(b)   // accept: I-7 at 'B; st('B) = Init ✓ (constant in this arm);
                    //   mayEq('B) = {'B} ('A # 'B) -> strong; st('B) := Uninit
} else {
    p = ref(b)      // accept: in-arm origin 'B (Γ = {¬c0})
    old = take(a)   // accept: I-7 at 'A; st('A) := Uninit
}
                    // accept: II-2 arm exit — each identity's exit state is a CONSTANT
                    //   within its arm (then: 'A Init, 'B Uninit; else: 'A Uninit, 'B Init),
                    //   so no inner atom crosses the arm boundary.
                    //   II-3 join with the atom c0 available:
                    //     st('A) = ite(c0, Init, Uninit)      one atom, TERM ✓
                    //     st('B) = ite(c0, Uninit, Init)      one atom, TERM ✓
                    //     origin(p) = ite(c0, 'A, 'B)          II-14, same atom
read(p)             // accept: II-5 correlated leafwise evaluation — all three facts carry
                    //   the SAME atom c0, so every query is evaluated at the same leaf.
                    //   leaf c0: origin 'A and st('A) = Init ✓
                    //   leaf ¬c0: origin 'B and st('B) = Init ✓
                    //   this is II-5's own worked example
```

Stated variant — swap the take targets so each arm takes p's selected object:

```text
if cond { p = ref(a); old = take(a) } else { p = ref(b); old = take(b) }
read(p)             // reject: II-5 leafwise. leaf c0: origin 'A, st('A) = Uninit.
                    //   missing fact: st('A) = Init on the c0 leaf. R1(i), site: take in
                    //   the then arm. the ¬c0 leaf fails symmetrically, so the diagnostic
                    //   names both — "rejects in both arms", as the case says
```

The case's "independent target/initialization sets cannot distinguish these
cases" is precisely what `II-5` buys and what an origin *set* (`II-15` row 2,
`{('A,'B)}` with no discriminant) would lose.

`old` is a copy integer; `R3` lets it be discarded with no obligation.

**Verdict: accepted as expected.**

---

## B03: A write initializes the selected target, not its entire may-set

```text
a = object(10)      // accept: I-1 + I-6
b = object(20)      // accept: I-1 + I-6; 'A # 'B by I-1
old_a = take(a)     // accept: I-7, strong; st('A) := Uninit
old_b = take(b)     // accept: I-7, strong; st('B) := Uninit
if cond { p = ref(a) } else { p = ref(b) }
                    // accept: III-7 freezes c0; II-2 exits with st('A), st('B) constant
                    //   Uninit in both arms; II-3/II-14 join: origin(p) = ite(c0, 'A, 'B)
q = p               // accept: II-14 — copying the binding copies the origin TERM, so
                    //   origin(q) = ite(c0, 'A, 'B), carrying the SAME atom as origin(p).
                    //   this is the captured equality the case relies on
write(p, 9)         // accept: I-6 through a term origin -> II-15 row 1: STRONG, LEAFWISE,
                    //   "the atom decides which path changed".
                    //   st('A) := ite(c0, Init, Uninit); st('B) := ite(c0, Uninit, Init)
                    //   each is one atom, TERM ✓. per leaf mayEq is a singleton ('A # 'B),
                    //   so KEY permits the strong update on the selected leaf.
                    //   III-11's note applies: the R5 write FOOTPRINT is still {'A,'B};
                    //   only the STATE effect is leafwise. no runtime branch (§1.5, M2(ii))
read(q)             // accept: II-5 — origin(q) carries the same atom c0 as the two state
                    //   terms, so all are evaluated at one leaf.
                    //   leaf c0: 'A, st('A) = Init ✓. leaf ¬c0: 'B, st('B) = Init ✓
                    //   [value 9: not supported by a stated rule — see the val gap]
read(a)             // reject: JF-TERM under Γ = ∅ cannot collapse ite(c0, Init, Uninit);
                    //   the state is not the constant Init.
                    //   missing fact: cond (i.e. the atom c0).
                    //   R1(i), site: take at L3, on the ¬c0 leaf. II-5's example uses this
                    //   exact diagnostic
```

Stated variant:

```text
read(b)             // reject: symmetric; st('B) = ite(c0, Uninit, Init) is not constant.
                    //   missing fact: ¬cond
```

This is the item that decides O11 in the affirmative for this candidate: under
O11(a) (premises only, no condition terms) `origin(q)` would be the set
`{'A,'B}`, `II-15` row 2 would refuse the *write* outright ("writing through an
origin set cannot initialize 'A"), and `read(q)` would then be unreachable. The
whole case is bought by admitting one atom at the join.

**Verdict: accepted as expected.**

---

## B04: A copied uncertain target supports take and restoration

```text
a = object(10)      // accept: I-1 + I-6; st('A) = Init
b = object(20)      // accept: I-1 + I-6; st('B) = Init; 'A # 'B by I-1
if cond { p = ref(a) } else { p = ref(b) }
                    // accept: III-7 freezes c0; II-3/II-14: origin(p) = ite(c0, 'A, 'B)
q = p               // accept: II-14 term copy; origin(q) = ite(c0, 'A, 'B), same atom
v = take(p)         // accept: I-7 through a term origin -> II-15 row 1, strong leafwise.
                    //   leaf c0: st('A) = Init ✓ -> Uninit. leaf ¬c0: st('B) = Init ✓ ->
                    //   Uninit. results:
                    //     st('A) = ite(c0, Uninit, Init);  st('B) = ite(c0, Init, Uninit)
                    //   one atom each, TERM ✓. per-leaf mayEq is a singleton, so I-7's
                    //   strong branch applies and no sibling widens to Live
put(q, move v)      // accept: I-8 through a term origin -> II-15 row 1, strong leafwise.
                    //   premise 1 per leaf: leaf c0 st('A) = Uninit ✓; leaf ¬c0
                    //   st('B) = Uninit ✓ — the same atom c0 as origin(q), so II-5 pairs
                    //   the selected path with its own hole.
                    //   premise 2 mayEq singleton per leaf ✓, so this is a strong fill and
                    //   NOT the weak put I-8 rejects.
                    //   effect: st('A) = ite(c0, Init, Init), st('B) = ite(c0, Init, Init);
                    //   JF-TERM collapses equal leaves to the CONSTANT Init
read(a)             // accept: st('A) is now the constant Init (the term collapsed). R1(i) ✓
read(b)             // accept: st('B) is the constant Init
```

The case's intermediate note:

```text
// between take and put:
read(a)             // reject: st('A) = ite(c0, Uninit, Init), not constant.
                    //   missing fact: cond.  R1(i)
read(b)             // reject: symmetric. missing fact: ¬cond
```

"Restoration acts on the same selected object" is `II-5`'s correlation between
`origin(q)` and the two state terms; the leaf-collapse at the end is `JF-TERM`'s
equality-of-leaves rule (`TERM`'s canonical form requires `l₁ ≠ l₂`, so a term
with equal leaves *is* the constant).

**Verdict: accepted as expected.**

---

## B05: Correlated distinct targets survive selected reclamation

```text
a = object(10)      // accept: I-1 + I-6; st('A)=Init, own('A)=held
b = object(20)      // accept: I-1 + I-6; st('B)=Init, own('B)=held; 'A # 'B by I-1
if cond {
    p = ref(a)      // accept: in-arm origins
    q = ref(b)
} else {
    p = ref(b)
    q = ref(a)
}
                    // accept: II-2 — no state, own or obl changed in either arm, so every
                    //   fact exits both arms as the same constant; only the origins differ.
                    //   II-3 last row + II-14, atom c0 available:
                    //     origin(p) = ite(c0, 'A, 'B)
                    //     origin(q) = ite(c0, 'B, 'A)
                    //   one atom each, TERM ✓, and it is the SAME atom in both (II-5)
release(p)          // accept: JF-ORG on a term origin -> II-15 row 1, strong leafwise;
                    //   I-4 checked per leaf.
                    //   leaf c0:  own('A) = held ✓ (constant, Γ = ∅ evaluation is trivial),
                    //             st('A) = Init ≠ Gone ✓, I-5 vacuous (u64) ✓
                    //   leaf ¬c0: own('B) = held ✓, st('B) = Init ≠ Gone ✓, I-5 ✓
                    //   effect, leafwise:
                    //     st('A) = ite(c0, Gone, Init)   own('A) = ite(c0, discharged, held)
                    //     st('B) = ite(c0, Init, Gone)   own('B) = ite(c0, held, discharged)
                    //   one atom each, TERM ✓. no drop flag, no bit: II-3's note
read(q)             // accept: II-5 — origin(q) = ite(c0, 'B, 'A) carries the same atom, so
                    //   it is evaluated at the same leaf as the state terms.
                    //   leaf c0:  'B, st('B) = Init ✓
                    //   leaf ¬c0: 'A, st('A) = Init ✓
                    //   this is exactly the correlation the case calls "(A,B) and (B,A),
                    //   not their Cartesian product"
release(q)          // accept: II-15 row 1 leafwise, I-4 per leaf.
                    //   leaf c0:  own('B) = held ✓ -> discharged, st('B) := Gone
                    //   leaf ¬c0: own('A) = held ✓ -> discharged, st('A) := Gone
                    //   after: own('A) = ite(c0, discharged, discharged) -> JF-TERM collapses
                    //   to the CONSTANT discharged; same for own('B).
                    //   so I-3 at the enclosing scope exit accepts: both owns are constant
                    //   discharged, which is I-3's premise. "neither obligation remains" ✓
```

Stated variants, immediately after the first release:

```text
read(a)             // reject: st('A) = ite(c0, Gone, Init) is not constant.
                    //   missing fact: ¬cond. R1(ii) on the c0 leaf, site: release at L8
release(a)          // reject: own('A) = ite(c0, discharged, held) is not the constant held,
                    //   which is I-4's premise. diagnostic R2 (definiteness), site: the
                    //   release at L8 on the c0 leaf. missing fact: ¬cond
```

**Verdict: accepted as expected.**

---

## B06: Rebinding must update relations without retargeting saved copies

```text
a = object(10)      // accept: I-1 + I-6
b = object(20)      // accept: I-1 + I-6; 'A # 'B by I-1
if cond {
    p = ref(a)
    q = ref(b)
} else {
    p = ref(b)
    q = ref(a)
}
                    // accept: II-3/II-14 as in B05:
                    //   origin(p) = ite(c0, 'A, 'B),  origin(q) = ite(c0, 'B, 'A)
saved = p           // accept: II-14 — "copying the binding copies the origin term", so
                    //   origin(saved) = ite(c0, 'A, 'B). the term is the VALUE of the
                    //   binding; it does not track the later fate of the name p
p = q               // accept: II-14; origin(p) := ite(c0, 'B, 'A).
                    //   nothing is attached to the names: distinctness lives on identities
                    //   (π # π', §1.3), never on variable names, which is the case's point
release(p)          // accept: II-15 row 1 leafwise, I-4 per leaf.
                    //   leaf c0:  own('B) = held ✓, st('B) = Init ≠ Gone ✓, I-5 ✓
                    //   leaf ¬c0: own('A) = held ✓, st('A) = Init ≠ Gone ✓, I-5 ✓
                    //   effect: st('A) = ite(c0, Init, Gone), own('A) = ite(c0, held, discharged)
                    //           st('B) = ite(c0, Gone, Init), own('B) = ite(c0, discharged, held)
read(saved)         // accept: II-5 — origin(saved) = ite(c0, 'A, 'B), same atom c0.
                    //   leaf c0:  'A, st('A) = Init ✓
                    //   leaf ¬c0: 'B, st('B) = Init ✓
                    //   the saved copy kept the OLD term, so it selects the surviving
                    //   identity on each leaf
read(q)             // reject: origin(q) = ite(c0, 'B, 'A) — the same term as origin(p)
                    //   after the rebinding at L10. leafwise:
                    //   leaf c0:  'B, st('B) = Gone. leaf ¬c0: 'A, st('A) = Gone.
                    //   R1(ii), site: release at L11 through p.
                    //   no missing fact can repair it: the selected identity has ended on
                    //   BOTH leaves, so no atom discriminates a live leaf
```

This is `II-14`'s own worked example, which cites `CASES.md` B06 by name.

**Verdict: accepted as expected.**

---

## Gaps found: lines the rules do not cover

None of these changes an accept/reject verdict above; each is recorded because
the derivation had to lean on prose or an example rather than a numbered rule.

| # | Gap | Items | Severity |
|---|---|---|---|
| G1 | **No numbered rule for `read`.** Set I covers `write`/`take`/`put`/`replace`/`free` and set II covers access through a non-singleton origin, but no rule states `read`'s premise. It is assembled from `KEY` ("read of a fact at π is `⊓` over `mayEq(π)`"), `R1(i)`/`R1(ii)` as the requirement, and the accepting/rejecting `read` lines in `I-1`, `I-4`, `I-7`, `II-5`, `II-15`'s examples | every item | low — unambiguous, but it is the most used operation in the fragment and it has no rule number |
| G2 | **No rule installs `val`.** `JF-KILL` kills `val`, `I-9` kills `val(π)`, `III-12` frames `RSt`, and `I-9`'s example asserts that a later read "reads `new`" — but no rule gives `write`, `put` or `replace` a `val(π) := v` effect. Every value the cases predict (11, 20, 10, 9) is therefore asserted by the candidate's examples, not derived from its rules | S01, S02, S04, B03, B04 | low for these verdicts (all access premises are on `st`), medium for `R6` frame claims generally |
| G3 | **No numbered rule forms or copies a pointer.** `§1.1` says non-signature paths are "formed by the checker from the access", `§1.3` holds an `origin` per pointer binding, and `II-14` states only the *split* case and the copy clause inside it. `p = ref(a)` with a singleton origin, and `q = p` in straight-line code, have no rule of their own | S01–S07, B03, B04, B06 | low |
| G4 | **No rule fixes intra-statement operand evaluation order.** S06's stated point (an operand that performs a take must not leave an earlier initialization premise authoritative) is general; the rules sequence *statements* only. S06 as written puts the take in its own statement, so this instance derives without it | S06 | medium for the case's stated principle, none for its written program |

`TERM`'s one-atom bound is never approached in this part: every case here
diverges on exactly one condition, so `II-2`'s refusal class (the price O11(b)
pays) is not exercised by S01–S07 or B01–B06. It first appears in B12's two
independent conditions, which is another part's item.

---

## Summary table

| Item | Result under `brand-context-bounded` | Ruling rules | Verdict |
|---|---|---|---|
| S01 | accept all four reads | `I-1`, `I-6`, `II-14`, `KEY`, `JF-KILL` | accepted as expected |
| S02 | accept; `replace` keeps `'A` and moves the old value out | `I-9`, `KEY`, `R6` | accepted as expected |
| S03 | reject `read(q)`; variants `read(a)`, `take(q)` reject | `I-7`, `KEY`, `R1(i)`, `R3` | accepted as expected |
| S04 | accept; the alias's `put` refills `'A` | `I-7`, `I-8`, `KEY` | accepted as expected |
| S05 | accept release of the empty slot and the later `read(v)` | `I-7`, `I-4`, `I-5`, `I-13`, `R3` | accepted as expected |
| S06 | reject `replace` after the take; `put` variant accepts | `I-9` premise 1, `I-8`, `R1(i)` | accepted as expected |
| S07 | accept `release(p)`; all five variants reject | `I-4` (permanence), `I-6`, `I-7`, `R1(ii)`, `R2` | accepted as expected |
| B01 | accept `read(p)` on both leaves | `III-7`, `II-1`, `II-2`, `II-3`, `II-14`, `II-15` row 1, `II-5` | accepted as expected |
| B02 | accept; swapped variant rejects in both arms | `II-3`, `II-5`, `I-7`, `II-15` row 1 | accepted as expected |
| B03 | accept `read(q)`; `read(a)`/`read(b)` reject | `II-14`, `II-15` row 1 (strong leafwise), `II-5`, `JF-TERM`, `III-11` | accepted as expected |
| B04 | accept both post-restoration reads; interim reads reject | `II-15` row 1, `I-7`, `I-8`, `JF-TERM` leaf collapse | accepted as expected |
| B05 | accept `read(q)` and the second release; `read(a)`/`release(a)` reject | `II-14`, `II-15` row 1, `I-4`, `I-5`, `II-5`, `I-3` | accepted as expected |
| B06 | accept `read(saved)`; reject `read(q)` | `II-14` (term copy and rebinding), `II-15` row 1, `I-4`, `II-5` | accepted as expected |

13 items, 13 accepted as expected; 0 capability losses, 0 deliberate refusals,
0 case corrections. Four rule gaps recorded (G1–G4), none verdict-changing.

This part therefore confirms the candidate's own `§7` rows for S01–S07 and
B01–B06 by independent line-by-line derivation, and it locates the whole of the
O11 evidence in this fragment at B03/B04/B05/B06: each of the four is carried by
`II-5`'s correlated leafwise evaluation of one atom shared between an origin
term and a state term. Without condition terms at joins (O11(a)) the origin
becomes a set, `II-15` row 2 refuses the write in B03 and the releases in
B05/B06, and all four fail. None of the four needs a second atom, so none of
them pays `II-2`'s price.

`M1` and `M2` in this part: every judgment used (`JF-ORG`, `JF-TERM`,
`JF-ENT` on a boolean atom, `KEY`/`JF-OVL` over singleton may-equal classes) is
a fixed-order total procedure with no search and no budget — `M1` holds for
these derivations. No rule applied here introduced a check, a bookkeeping word
or an outcome-selecting branch the source did not write: B05's and B06's
conditional own terms are erased checker state (`§1.5`, `II-3`'s note), and
every branch executed is the writer's `if cond` — `M2` holds for these
derivations.


# File: derive-brand-context-bounded-late-branches.md

# Derivation: candidate `brand-context-bounded`, part `late-branches`

Items: `CASES.md` B07, B08, B09, B10, B11, B12, B13; regression checks
`PROGRAMS.md` P1 and P3. Date 2026-09-16.

Every line below is derived **only** from
[`rules-brand-context-bounded.md`](rules-brand-context-bounded.md) (§2 `ATOM`,
`TERM`, `KEY`; §3 rule set I; §4 rule set II; §5 rule set III; §6 the judgment
families). Each accept cites the rule that admits it; each reject cites the rule
and the missing fact. Where the rule sets do not cover a line it is marked
**rule missing** and the derivation continues. No candidate is compared and none
is selected.

## 0. The `CASES.md` vocabulary, mapped to the rules once

`CASES.md`'s scope predates the candidate's vocabulary, so the mapping is fixed
here and reused without restating it per case.

| `CASES.md` | Rule that admits it | Effect used below |
|---|---|---|
| `a = object(10)` | `I-1` then `I-6` | mint `'A`; `st('A) := Uninit` then `:= Init`; `own('A) := held`; `obl('A) := none` (`u64` carries no release obligation) |
| `p = ref(a)` | §1.3 `origin(v)`, `II-13` | `origin(p) := 'A`; **nothing is consumed**, the base stays writable (`D4`, `II-13`) |
| `read(p)` | premise `st(origin(p)) = Init` after `JF-ORG` + `JF-TERM` | — |
| `write(p, n)` | `I-6` | `st := Init`; no obligation clause fires (`u64` is copy) |
| `v = take(p)` | `I-7` | `st := Uninit`, strong when `mayEq = {π}` |
| `put(p, v)` | `I-8` | premise `st = Uninit`; strong fill only |
| `old = replace(p,n)` | `I-9` | premise `st = Init` |
| `release(a)` / `release(p)` | `I-4` (+ `I-5`, + `II-15` row 1 for the locator form) | `st := Gone` permanently; `own := discharged` |
| `cond` as a guard | `III-7` + `ATOM` | `FREEZE` binds a frozen value `c0`; the atom is over `c0`, never over the *variable* |
| leaving the owning scope | `I-3` | requires `own` to be the **constant** `discharged` |

Two consequences used repeatedly:

- `II-1` (arm entry) evaluates **every** term in scope under the new `Γ`, so a
  term whose atom the arm's guard entails or refutes collapses to a constant
  before any premise is checked. This is `TERM`'s guard-context restriction.
- `II-3` joins with the construct's **own** guard atom available, so a
  one-atom `ite` is the join of two differing arms and nothing wider.

`CASES.md`'s permissive "release through a locator" variant is the one the rules
implement: `II-15`'s table lists `free` among the operations performed through an
origin term, so the obligation is not copied into each locator.

---

## B07 — a conditional hole prevents reading but not scalar overwrite

### B07.1 The rejecting form

```text
a = object(10)         // accept: I-1 mints 'A, own('A)=held, obl('A)=none;
                       //   I-6 st('A) := Init
p = ref(a)             // accept: origin(p) = 'A. II-13: nothing consumed, no
                       //   suspension of 'A's writability (D4)
if cond { old = take(p) }
                       // accept (guard): III-7 + ATOM — FREEZE binds the frozen
                       //   value c0 of `cond`; the atom is `c0`
                       // accept (then): II-1 pushes c0 on Γ; JF-ORG resolves
                       //   origin(p)='A; I-7 premise st('A)=Init holds;
                       //   KEY: mayEq('A) = {'A} (one root, no symbolic index),
                       //   so the update is strong: st('A) := Uninit
                       // accept (else): empty arm, st('A) = Init
                       // accept (join): II-3 row 2 — the arms differ and the
                       //   construct's own atom c0 is available:
                       //   st('A) = ite(c0, Uninit, Init).  TERM: one atom ✓
                       // accept (arm exit, II-2): each arm exits a CONSTANT under
                       //   its own Γ, so nothing carries an inner atom outward
read(p)                // reject: JF-TERM evaluates st('A) under Γ = {} — c0 is
                       //   independent of every enclosing guard, so the term
                       //   survives and is not the constant Init.
                       //   "R1(i): 'A holds no value when cond is true
                       //    (site: take at L3). missing fact: ¬c0."
                       //   M7 repair routes: branch on cond, or write the slot
```

### B07.2 The accepting variant (`write(p, 20); read(p)`)

```text
if cond { old = take(p) }
                       // as above: st('A) = ite(c0, Uninit, Init)
write(p, 20)           // accept: I-6. Premise st(π) ≠ Gone holds on BOTH leaves
                       //   (Uninit and Init are both ≠ Gone); the obligation
                       //   clause does not fire because u64 carries none.
                       //   KEY: mayEq('A)={'A}, strong update on each leaf:
                       //   st('A) := Init on both -> TERM collapses the term to
                       //   the CONSTANT Init (equal leaves are not a term)
                       // JF-KILL kills val('A); the new val('A) = 20 on both leaves
read(p)                // accept: st('A) = Init.  Reads 20 on both paths
```

This line is the rules' own worked example at `I-6` ("`<- CASES B07's accepting
variant`").

### B07.3 The two narrower operations the case names

```text
if cond { old = take(p) } ; put(p, 20)
                       // reject: I-8 premise st(π) = Uninit; the term
                       //   ite(c0, Uninit, Init) is not the constant Uninit.
                       //   "missing fact: c0"
if cond { old = take(p) } ; old2 = replace(p, 20)
                       // reject: I-9 premise st(π) = Init; same term is not the
                       //   constant Init.  "missing fact: ¬c0"
```

Both refusals are exactly the case's point — the operation-specific premises
lack their input on one leaf — and neither refusal is transferred to `I-6`, so
ordinary scalar assignment is not narrowed.

**Verdict B07: accepted as expected.** (§7's table lists B07 as "as the case
states".)

---

## B08 — repeating an unchanged condition recovers a path's state

```text
a = object(10)         // accept: I-1 + I-6. st('A)=Init, own('A)=held
p = ref(a)             // accept: origin(p) = 'A
if cond { old = take(p) }
                       // accept: III-7 freezes c0; I-7 in the then arm;
                       //   II-3: st('A) = ite(c0, Uninit, Init)
if cond { put(p, 20) }
                       // accept (guard): III-7 — `cond` was NOT reassigned, and a
                       //   frozen value is "killed by nothing / re-derived by
                       //   nothing", so this guard's atom is the SAME c0.
                       //   This is the whole content of the case: the atom is on
                       //   the captured value, and no rule ages it (ATOM: atoms
                       //   never die; only bridge facts die)
                       // accept (then): II-1 pushes c0; JF-TERM evaluates
                       //   st('A) under Γ={c0} -> collapses to the CONSTANT
                       //   Uninit. I-8 premise st=Uninit ✓; KEY mayEq={'A} so the
                       //   fill is strong (I-8 rejects only a weak put).
                       //   arm exit: st('A) = Init (constant)
                       // accept (else): II-1 pushes ¬c0; JF-TERM collapses
                       //   st('A) to the CONSTANT Init. arm exit: Init
                       // accept (join): II-3 row 1 — both arms Init -> the
                       //   constant Init. No term survives the second if
read(p)                // accept: st('A) = Init.  20 on the true path (val set by
                       //   the put), 10 on the false path (val('A) never killed
                       //   on that leaf, JF-KILL is leafwise under II-5)
```

Nothing here rests on discovering that the two guards agree: `III-7`'s frozen
value makes them the *same atom*, and `II-1` does the discharge. No `use` step,
no invariant, no written fact.

**Verdict B08: accepted as expected.**

---

## B09 — conditions refer to captured values, not permanent variable names

### B09.1 The rejecting form

```text
a = object(10)         // accept: I-1 + I-6
p = ref(a)             // accept
if cond { old = take(p) }
                       // accept: III-7 freezes c0; II-3:
                       //   st('A) = ite(c0, Uninit, Init)
cond = !cond           // accept: III-7 — reassigning the VARIABLE creates a new
                       //   frozen value c1, and the relation `c1 = ¬c0` is
                       //   recorded as an ordinary atom in JF-ENT's boolean part.
                       //   ATOM: the old atom c0 is untouched; nothing widens
if cond { write(p, 20) }
                       // accept (guard): the atom is c1, not c0
                       // accept (then): II-1 pushes c1; JF-ENT step (4), unit
                       //   propagation over `c1 = ¬c0`, derives ¬c0, so JF-TERM
                       //   collapses st('A) to Init. I-6 accepts; exit Init
                       // accept (else): Γ={¬c1} ⊢ c0; st('A) collapses to Uninit;
                       //   nothing executed; exit Uninit
                       // accept (join): II-3 row 2, atom c1 available:
                       //   st('A) = ite(c1, Init, Uninit).  TERM: one atom ✓
read(p)                // reject: JF-TERM under Γ={} leaves the term standing; it
                       //   is not the constant Init.
                       //   "R1(i): 'A holds no value when c1 is false, i.e. on the
                       //    ORIGINAL true path (site: take at L3).
                       //    missing fact: c1."
```

This is `III-7`'s own worked example ("`<- CASES B09`").

### B09.2 The accepting variant (`if !cond { put(p, 20) }`)

```text
cond = !cond           // accept: c1 bound, atom `c1 = ¬c0` recorded
if !cond { put(p, 20) }
                       // accept (then): Γ={¬c1}; JF-ENT boolean propagation gives
                       //   c0; JF-TERM collapses st('A) to the constant Uninit;
                       //   I-8 premise ✓, mayEq={'A} strong; exit Init
                       // accept (else): Γ={c1} ⊢ ¬c0; st('A) collapses to Init;
                       //   exit Init
                       // accept (join): II-3 row 1, both Init -> constant Init
read(p)                // accept: st('A) = Init
```

The case's requirement — "Boolean assignment invalidates reuse of the old
variable value, but its known relation to the new value can preserve useful
information. Forced loss of every relation on assignment would be an
approximation" — is met by `III-7` recording `c1 = ¬c0` and `JF-ENT`(4)
propagating it. No relation is lost and no search is used: step (4) is unit
propagation over a finite recorded literal set (M1 ✓).

**Verdict B09: accepted as expected.**

---

## B10 — a join must not erase conditional disposal obligations

```text
a = object(10)         // accept: I-1 mints 'A; I-6 st('A)=Init; own('A)=held
b = object(20)         // accept: I-1 mints 'B; st('B)=Init; own('B)=held
if cond { release(a) } else { release(b) }
                       // accept (guard): III-7 freezes c0
                       // accept (then): II-1 Γ={c0}; I-4 premises: own('A)=held is
                       //   a constant under Γ ✓; st('A) ≠ Gone ✓; I-5's obligation
                       //   guard is vacuous (u64 content, obl('A)=none) ✓.
                       //   Effect: st('A) := Gone permanently; own('A) := discharged;
                       //   I-13 propagates over owns('A) = {} (nothing)
                       // accept (else): II-1 Γ={¬c0}; I-4 on 'B symmetrically
                       // accept (join): II-3, atom c0 available on every head:
                       //   st('A)  = ite(c0, Gone, Init)
                       //   own('A) = ite(c0, discharged, held)      <- row 5
                       //   st('B)  = ite(c0, Init, Gone)
                       //   own('B) = ite(c0, held, discharged)      <- row 5
                       //   TERM: one atom per identity per point ✓; II-5 records
                       //   that all four carry the SAME atom, so every later query
                       //   evaluates them at the same leaf
```

The case's two failure modes, checked against the rules:

```text
// "Intersecting the two remaining-owner sets yields the empty set" —
//   no rule of the candidate performs that intersection. II-3 row 5 has exactly
//   two outcomes: the ite term (atom available) or a REJECT for R2 definiteness.
//   Dropping an obligation at a join is not in the table.
// "Explicit-consumption checking would mistakenly consider it complete" —
//   refused: own('A) is ite(c0, discharged, held), and I-3 at the enclosing
//   scope exit demands the CONSTANT discharged (see B13)
// "Automatic cleanup would mistakenly omit it" —
//   refused: no rule releases anything at a join; I-3 rejects instead of
//   inserting a release (M2(ii), R2's Price)
```

No line of B10 rejects, and one obligation per path is retained exactly.

Cost of the exactness, stated: the retention is bought by `II-3` row 5 admitting
a term, i.e. by owner decision **O11(b)**. Under O11(a) (premises only) `own`
would have no join value here and the construct itself would reject.

**Verdict B10: accepted as expected.**

---

## B11 — a locator can identify the conditionally remaining obligation

```text
a = object(10)         // accept: I-1 + I-6. st('A)=Init, own('A)=held
b = object(20)         // accept: I-1 + I-6. st('B)=Init, own('B)=held
if cond {
    release(a)         // accept: II-1 Γ={c0}; I-4 (+I-5 vacuous):
                       //   st('A):=Gone, own('A):=discharged
    remaining = ref(b) // accept: origin(remaining) := 'B in this arm
} else {
    release(b)         // accept: II-1 Γ={¬c0}; I-4: st('B):=Gone,
                       //   own('B):=discharged
    remaining = ref(a) // accept: origin(remaining) := 'A in this arm
}
                       // accept (join): II-14 origin split, atom c0 available:
                       //   origin(remaining) = ite(c0, 'B, 'A)
                       // II-3 on the states and owns:
                       //   st('A)=ite(c0,Gone,Init)   own('A)=ite(c0,discharged,held)
                       //   st('B)=ite(c0,Init,Gone)   own('B)=ite(c0,held,discharged)
                       // II-5: all five facts carry the SAME atom c0, so they are
                       //   evaluated at one leaf per query — the correlation
                       //   between "which storage `remaining` names" and "which
                       //   storage is still live" is kept exactly
read(remaining)        // accept: JF-ORG on an ite origin -> II-15 row 1, leafwise,
                       //   with II-5 pinning the leaf:
                       //     leaf c0  -> 'B, st('B)=Init ✓
                       //     leaf ¬c0 -> 'A, st('A)=Init ✓
                       //   Both leaves satisfy the read premise, so the access is
                       //   admitted with NO discriminating branch in the source
                       //   and no runtime test (M2(ii): the term is erased
                       //   checker state, and the lowered read is one load)
release(remaining)     // accept: II-15 row 1 — free through an ite origin is
                       //   STRONG and LEAFWISE, the atom deciding which path
                       //   changed. Per leaf, I-4's premises:
                       //     leaf c0  -> own('B)=held ✓, st('B)≠Gone ✓, I-5 ✓
                       //     leaf ¬c0 -> own('A)=held ✓, st('A)≠Gone ✓, I-5 ✓
                       //   Effect, leafwise:
                       //     own('A) = ite(c0, discharged, discharged)
                       //     own('B) = ite(c0, discharged, discharged)
                       //   TERM's canonical form requires l1 ≠ l2, so both
                       //   collapse to the CONSTANT `discharged`
                       //   st('A) = st('B) = constant Gone, likewise
// leaving the scope   // accept: I-3 — own('A), own('B) are both the constant
                       //   `discharged`.  "neither obligation remains" ✓
```

The case's claim that "the ordinary selected runtime pointer is enough for the
operation; the relationship to the obligation is static evidence" is exactly
`II-14` + `II-15` row 1: the runtime value is one pointer, the obligation
bookkeeping is the erased term.

**Verdict B11: accepted as expected.** (§7's table lists B11 as accept, same,
by `II-5` leafwise correlation.)

---

## B12 — two conditional releases separate safety from completion

### B12.1 The rejecting form (independent `c` and `d`)

```text
a = object(10)         // accept: I-1 + I-6. st('A)=Init, own('A)=held
if c { release(a) }    // accept (guard): III-7 freezes c0
                       // accept (then): II-1 Γ={c0}; I-4 premises all hold;
                       //   st('A):=Gone, own('A):=discharged
                       // accept (join): II-3 rows 4 and 5, atom c0 available:
                       //   st('A)  = ite(c0, Gone, Init)
                       //   own('A) = ite(c0, discharged, held)
if d { release(a) }    // accept (guard): III-7 freezes d0, an independent frozen
                       //   scalar; ATOM ✓
                       // reject (then): II-1 pushes d0; JF-TERM step 1 issues one
                       //   JF-ENT query on the atom c0 under Γ={d0}. The
                       //   difference-bound closure and the boolean propagation
                       //   record no relation between d0 and c0, so c0 is neither
                       //   entailed nor refuted and the term does NOT collapse.
                       //   I-4's premise is `own('a) = held` as a CONSTANT after
                       //   JF-TERM evaluation; own('A) = ite(c0, discharged, held)
                       //   is not constant.
                       //   "R2/I-4: 'A's obligation is already discharged when c
                       //    holds (site: release at L2).  missing fact: ¬c0.
                       //    repair: branch on c — write `if !c { release(a) }` —
                       //    or make the second release's guard imply ¬c."
```

The rejection is static and covers the whole program, not only the `c ∧ d`
execution; that is the case's own reading ("Independent unrestricted `c` and `d`
cannot justify the second release"). Note what is *not* emitted: no drop flag,
no per-path bit, no runtime comparison of `c` against `d` (M2(ii); `II-3`'s
"NO drop flag, NO bit: the term is erased checker state").

The case's second requirement — that completing the sequence also needs
`c ∨ d` — lands at the scope exit, on the same rule as B13:

```text
// with the second release deleted, leaving `if c { release(a) }` alone:
}                      // reject: I-3 — own('A) = ite(c0, discharged, held) is not
                       //   the constant `discharged`.  (identical to B13)
```

### B12.2 The accepting variant (`if cond { … }; if !cond { … }`)

```text
a = object(10)         // accept
if cond { release(a) } // accept: own('A) = ite(c0, discharged, held);
                       //   st('A) = ite(c0, Gone, Init)
if !cond { release(a) }
                       // accept (guard): `cond` unchanged -> III-7 gives the SAME
                       //   frozen c0; the guard atom is ¬c0
                       // accept (then): II-1 Γ={¬c0}; JF-TERM collapses
                       //   own('A) to held and st('A) to Init; I-4's premises hold;
                       //   effect: own := discharged, st := Gone. exit constants
                       // accept (else): Γ={c0}; own('A) collapses to discharged,
                       //   st('A) to Gone; nothing executed. exit the same constants
                       // accept (join): II-3 row 1 on both heads —
                       //   own('A) = discharged (constant), st('A) = Gone (constant)
}                      // accept: I-3 — own('A) is the constant `discharged`.
                       //   Exactly one release on every path, no flag
```

Both halves of the case are derived on the same two rules (`II-1`'s shrinking
`Γ` and `I-4`'s constancy premise); the difference between them is whether the
two guards' atoms are related, which `III-7` decides at the guard, not at the
release.

**Verdict B12: accepted as expected.**

---

## B13 — conditional scope cleanup is a policy choice with known input state

```text
a = object(10)         // accept: I-1 mints 'A with own('A)=held; I-6 st('A)=Init
if cond { release(a) } // accept (guard): III-7 freezes c0
                       // accept (then): II-1 Γ={c0}; I-4 -> st:=Gone,
                       //   own:=discharged
                       // accept (join): II-3:
                       //   st('A)  = ite(c0, Gone, Init)
                       //   own('A) = ite(c0, discharged, held)
// leave the owning scope
}                      // reject: I-3 — for every identity minted in the scope, the
                       //   premise is `own` is the CONSTANT `discharged`, or
                       //   `obl = none` AND the slot is a compiler-released frame
                       //   slot. 'A came from I-1 (an object with a disposal
                       //   obligation), not from I-2, so the second clause does
                       //   not apply. own('A) = ite(c0, discharged, held) is not
                       //   constant.
                       //   "R2: 'A's obligation is outstanding when cond is false
                       //    (site: object at L1).  repair: write
                       //    `else { release(a) }`, or `if !cond { release(a) }`
                       //    before the scope ends."
                       // I-3's second sentence is the decisive one: "A non-constant
                       //   `own` at scope exit is REJECTED, never released by an
                       //   inserted branch (M2(ii))."
```

The repaired program, derived:

```text
a = object(10)
if cond { release(a) }
if !cond { release(a) }
                       // accept: exactly B12.2 — own('A) becomes the constant
                       //   `discharged` by II-1 + II-3
}                      // accept: I-3
```

**What the difference is.** `CASES.md` B13 states two policies as alternatives
and selects neither. This candidate selects **explicit consumption** and refuses
automatic cleanup, because the generated `if !original_cond { release(a) }` is a
release the source did not write (`M2(ii)`; R2's Price; `I-3`'s own wording).
The refusal is repairable at one tree location with a named local fix — write
the guarded release — so M7 holds. The case's closing constraint ("Any future
generated cleanup must implement the selected cleanup semantics; it must not
repair an unproved access with a runtime safety test") is satisfied vacuously:
nothing is generated.

This is a selection **inside** the case's stated alternatives rather than a
contradiction of it, but since the automatic-cleanup policy the case also
admits is refused, it is recorded here in the strongest honest bucket.

**Verdict B13: differs from the expected verdict — deliberate refusal with a
repair (named: write the guarded release, `else { release(a) }` or
`if !cond { release(a) }`; automatic cleanup refused by `I-3` + M2(ii)).**
This matches the candidate's own §7 row for B13.

---

## P1 — container split with a runtime index (regression)

`PROGRAMS.md` P1 requires: sequential legality by proof rather than borrow
scope; parallel permission for the two writes; a diagnostic at `read(v[k])`
naming the missing fact; no runtime check. The `read(v[k])` requirement ("legal
iff `k` is provably not `i` and not `j`") fixes the reading: the two parts are
values **taken out** of the vector, leaving holes, and are put back before the
`push`. That is the shape `III-12`'s worked example derives.

### P1.1 The naive shape

```text
qi = take(v, i)        // reject: R11 — the index's own domain. I-7 resolves
                       //   'b[i] through JF-QI on the Vec invariant
                       //   RSt('b[0..len), Init, {}), and the instantiation needs
                       //   i < len(v). No len bridge fact is bound.
                       //   "R11: index requires i < len(v); len('v) is unbound.
                       //    repair: `let n0 = len(v)`"  (III-1, III-2)
```

### P1.2 With the bridge bound but no index branch

```text
let n0 = len(v)        // accept: III-1 FREEZE. bridge fact len('v) = n0 (support
                       //   'v); frozen value n0 (immortal, ATOM)
qi = take(v, i)        // reject: R11 again — JF-ENT has no route from Γ={} to
                       //   `i < n0`.  "missing fact: i < n0"
```

### P1.3 The shape the rules accept

```text
let n0 = len(v)                     // accept: III-1 FREEZE
if i < n0 && j < n0 && i != j {
                                    // accept: II-1 pushes three atoms on Γ; each
                                    //   operand is an SSA scalar, so ATOM's frozen
                                    //   -operand premise holds.
                                    //   M3/§6: `use i != j` is NOT writable — no
                                    //   named rule has a disequality on two runtime
                                    //   scalars as its conclusion — so a WRITTEN
                                    //   BRANCH is the only route (§7's P1 cost row)
  qi = take(v, i)                   // accept: JF-QI instantiates RSt at i (one
                                    //   instantiation at the access's own index, no
                                    //   pattern search); the exception list is empty,
                                    //   so st('b[i]) = Init. I-7's premise ✓.
                                    //   KEY: mayEq('b[i]) = {'b[i]} — no other
                                    //   symbolic element path is live — so the
                                    //   update is STRONG: st('b[i]) := Uninit.
                                    //   III-12: RSt := ('b[0..n0), Init, {i})
  qj = take(v, j)                   // accept: KEY — mayEq('b[j]) would contain
                                    //   'b[i], but JF-OVL discharges it with one
                                    //   JF-ENT query on `i ≠ j`, which Γ carries.
                                    //   Strong: st('b[j]) := Uninit.
                                    //   III-12: RSt := ('b[0..n0), Init, {i, j})
  par { write_part(qi, 1) || write_part(qj, 2) }
                                    // accept, with a flag. qi, qj are affine values
                                    //   that LEFT 'b[i], 'b[j] (I-7 + R3); their
                                    //   storage is two frame slots minted by I-2.
                                    //   R5(i) needs their footprints proved
                                    //   non-interfering. JF-OVL's procedure only
                                    //   compares "live paths with the same base", so
                                    //   two distinct roots never may-overlap.
                                    //   rule missing: no rule of set I STATES a
                                    //   distinctness effect for an I-2 frame mint,
                                    //   while I-1 explicitly requires the
                                    //   allocator's written `ensures` for the heap
                                    //   case and forbids deriving `#` from name
                                    //   inequality (R4(b)). Which of the two régimes
                                    //   a frame slot is under is unstated; the
                                    //   derivation above leans on JF-OVL's same-base
                                    //   restriction, a procedure detail, not a rule
  if k != i && k != j {
    read(v[k])                      // accept: JF-QI instantiates
                                    //   RSt('b[0..n0), Init, {i,j}) at k, then
                                    //   checks k against each exception by JF-ENT;
                                    //   Γ discharges both. st('b[k]) = Init
  } else {
    // a written outcome arm (R12)  // accept: M2(i) — the only branch at the v[k]
                                    //   read is the WRITER's; no check is inserted
  }
  put(v, i, move qi)                // accept: I-8 — st('b[i]) = Uninit ✓ and
                                    //   mayEq('b[i]) = {'b[i]} under Γ (i ≠ j), so
                                    //   the fill is strong (a weak put is rejected).
                                    //   III-12: exception i discharged, X := {j}
  put(v, j, move qj)                // accept: I-8; X := {}
  push(v, x)                        // accept: I-16 requires RSt('b[0..len), Init, {})
                                    //   and it holds. Exit state per I-16's
                                    //   two-armed contract:
                                    //     backing('v) = ite(n0 < c0, 'b, 'b2)
                                    //     st('b)  = ite(n0 < c0, Live, Gone)
                                    //   (unused below; no interior pointer survives)
}
```

The rejecting variants P1 demands:

```text
  qj = take(v, j)      // without `i != j` in Γ: reject. KEY — mayEq('b[j]) =
                       //   {'b[j], 'b[i]}; a read of the fact is ⊓ over the class,
                       //   Init ⊓ Uninit = Live; I-7 needs Init.
                       //   "R1(i): 'b[j] may be the element taken at L1 (i = j).
                       //    missing fact: i # j"
                       //   §7: CORRECTION of the base's derivation (soundness F1) —
                       //   the base accepted a double take at may-equal indices,
                       //   duplicating an affine element
  read(v[k])           // without `k != i && k != j`: reject. JF-QI matches k
                       //   against the exception list {i, j} and JF-ENT discharges
                       //   neither.
                       //   "R1(i): 'b[k] may be the element taken at L1 or L2.
                       //    missing fact: k ≠ i ∧ k ≠ j"    <- M7's P1 payload ✓
  push(v, x)           // written before either put: reject. I-16's premise is
                       //   RSt('b[0..len), Init, {}), and X = {i, j}.
                       //   "I-16: 'b[i] holds no value (site: take at L1)."
                       //   <- P1's "only after both are back" ✓
```

### P1.4 One further rule gap

If the `else` arm of the index guard **falls through** to the join rather than
returning or diverging, the join must combine the bridge fact `len('v) = n0`
(unchanged in the else arm) with `len('v) = n0+1` (after the `push` in the then
arm), and the `RSt` residual list on both edges.

```text
} else { /* fall through */ }
                       // rule missing: II-3's join table has rows for st, own and
                       //   origin only. No rule says how a BRIDGE fact (III-1,
                       //   III-2, III-3) or an RSt residual list joins when the
                       //   two edges disagree. II-6 states the discard at a LOOP
                       //   HEAD and II-10 at a CALL boundary; the if-join is
                       //   unstated. The conservative reading (drop the bridge
                       //   fact, re-FREEZE after the join) is consistent with
                       //   III-1's "re-derivation is one re-application of FREEZE",
                       //   but no rule states it
```

**Verdict P1: accepted with a stated cost** — one written guard carrying five
index facts (`i < n0`, `j < n0`, `i ≠ j`, `k ≠ i`, `k ≠ j`) plus a written
outcome arm, because `M3`/O15(b) deletes the `use k != i` route: `use R(args)`
is a named rule application with checked premises, and no named rule concludes a
disequality on two runtime scalars. The four required properties hold: legality
is by proof and not by a borrow scope (`D4`, `II-13`); the diagnostic at
`read(v[k])` names the missing fact (`JF-QI` + M7); no runtime check is inserted
(M2(ii) — every branch above is the writer's). **Two rule gaps recorded**: the
distinctness régime for `I-2` frame mints, and the join of bridge facts / `RSt`
residuals at an `if`-join.

---

## P3 — graph with back edges in a pool (regression)

`PROGRAMS.md` P3 requires: link updates through several paths to one node legal
sequentially; removal invalidates every path to `m`; parallel data writes
permitted by distinctness of nodes; the invariant "next/prev point to live
nodes" stated once and reused.

```text
// The pool 'P carries its WRITTEN data: a free list (III-5, O17(a) — the word is
// charged in the pool's type, not as compiler bookkeeping) and a written
// ∀-invariant INV_pool: forall s. live(s) ==> st('P[s]@g) = Init
//                               and next/prev of 'P[s] name live slots
pa = unpack(a.next)          // accept: II-16 — the only term former that reads an
                             //   identity out of a value. Binds FRESH frozen
                             //   values s_a, g_a and gives ptr<'P[s_a]@g_a, Node>.
                             //   st('P[s_a]@g_a) = Init from INV_pool by JF-QI
pb = unpack(b.prev)          // accept: II-16; fresh s_b, g_b
                             //   note: pa # pb is NOT derivable (II-16's own
                             //   statement: "two unpacks are not distinct without
                             //   a written fact — that is D4's declared cost")
pn = insert(pool, node)      // accept: I-24 insert mints 'P[s_n]@g_n with a FRESH
                             //   ghost index by I-23; st := Uninit; the slot leaves
                             //   the written free list (III-5, writes 'P.freelist).
                             //   I-8/I-6 fill it: st('P[s_n]@g_n) := Init
// four link writes
write(pa.next, pn)           // accept: I-6 — st('P[s_a]@g_a.next) ≠ Gone. Several
                             //   live paths to one node need no permission
                             //   judgment: II-13 consumes nothing and suspends no
                             //   writability (D4).            <- P3 requirement (1)
                             //   JF-KILL's footprint is the field path; JF-QI does
                             //   NOT kill INV_pool, it WEAKENS it:
                             //   the residual exception list X := X ∪ {s_a}
write(pn.prev, pa)           // accept: I-6; X := X ∪ {s_n}
write(pn.next, pb)           // accept: I-6; s_n already in X
write(pb.prev, pn)           // accept: I-6; X := X ∪ {s_b}
use INV_pool.links_live(s_a, s_n, s_b)
                             // accept: O15(b) / III-5 — ONE application of a NAMED
                             //   rule whose premises the checker checks (never a
                             //   bare assume, M3). It discharges the three
                             //   residual exceptions, restoring the ∀-invariant
                             //   with X = {}.                  <- P3 requirement (4)
                             //   §7: correction of the base's frame claim (cost S6)
                             //   — the invariant is weakened per symbolic write and
                             //   restored by one `use`, not killed and restated
remove(pool, m)              // accept: I-24 — premises st('P[s_m]@g_m) = Init, own
                             //   held, I-5 (data is u64, no content obligation).
                             //   Effect: st(π) := Gone for every π ≤ 'P[s_m]@g_m,
                             //   PERMANENTLY; every fact under it killed; the slot
                             //   returns to the written free list
read(p_m_direct.data)        // reject: R1(ii) — st('P[s_m]@g_m) = Gone
                             //   (site: remove at L?).  The fact is on the path, so
                             //   one fact refuses every name of it (I-4's wording)
                             //                             <- P3 requirement (2), direct
// traversal from any node following next
p = unpack(x.next)           // accept: II-16; fresh s, g_s; st = Init from INV_pool
read(p.data)                 // reject: KEY — mayEq('P[s]@g_s) ∋ 'P[s_m]@g_m, since
                             //   neither `s # s_m` nor `g_s # g_m` is proved
                             //   (JF-OVL compares ghost indices by JF-PATH only, and
                             //   g_s, g_m are distinct frozen values with no
                             //   recorded relation). The read of the fact is ⊓ over
                             //   the class: Init ⊓ Gone = ⊤.
                             //   "R1(ii): 'P[s] may be the slot removed at L.
                             //    missing fact: s ≠ s_m"
                             //   §7: CORRECTION (soundness F2) — the base accepted
                             //   a traversal cursor reading ended storage
if s != s_m { read(p.data) } // accept: II-1 pushes `s ≠ s_m`; JF-OVL's JF-ENT query
                             //   resolves mayEq to {'P[s]@g_s}; JF-QI on INV_pool
                             //   gives st = Init.      <- the named repair, P3 req (2)
// the storage of m is reused for a later insert
pm2 = insert(pool, w)        // accept: I-23 — the minted identity is 'P[s_m]@g'
                             //   with a FRESH ghost index. Gone is never lifted;
                             //   the new identity starts Uninit.
                             //   'P[s_m]@g' and 'P[s_m]@g_m are related by NO rule:
                             //   a different ghost index means not-may-equal, so
                             //   KEY never joins them (III-6).
                             //   M2(ii): g is minted and erased, never compared at
                             //   run time — CONDITIONAL ON OWNER DECISION O3 (§6,
                             //   D5). If O3(b) rules a ghost generation out by name,
                             //   this line loses its separation and the candidate is
                             //   either M2(ii)-violated or slot reuse is refused
read(p_m_direct.data)        // reject: R1(ii), st('P[s_m]@g_m) = Gone. Address reuse
                             //   never revives an ended identity (I-4: "no rule
                             //   anywhere sets a Gone path to any other state")
// parallel map over all nodes writing data
par forall s' in 0..len('P) { write(pool[s'].data, f(s')) }
                             // accept: R5(i) — the construct's own index atoms give
                             //   s1 ≠ s2 for two iterations; JF-OVL then separates
                             //   'P[s1]@g1 from 'P[s2]@g2 by one JF-ENT query on the
                             //   indices.                      <- P3 requirement (3)
par forall n in traverse(list) { write(n.data, …) }
                             // reject: the slots come from II-16 unpacks, and
                             //   "two unpacks are NOT distinct without a written
                             //   fact".  "R5(i): missing fact: s1 # s2.
                             //    repair: `use INV_pool.distinct_links(a,b)` if the
                             //    pool states it, or branch on s1 != s2."
                             //   Stated cost, with the repair named by II-16 itself
```

**Verdict P3: accepted with a stated cost.** The four required properties hold,
each at a price the rules name: (1) sequential multi-path link writes are free
(`D4`, `II-13`); (2) removal invalidates every path, including the traversal
cursor, but the cursor's rejection needs a written `s ≠ s_m` branch to be
repaired (`KEY`, `III-6`) — §7 records this as a **correction of the base's
derivation** (soundness F2), not a change to `PROGRAMS.md`'s requirement, which
it satisfies more strictly; (3) the parallel map is admitted over an index
range, and over a linked traversal only with a written pool distinctness
invariant discharged by one `use` (`II-16`, D4's declared cost); (4) the
invariant is stated once and reused through `JF-QI`'s residual framing plus one
`use` (`III-12`, `III-5`). The whole derivation carries **one open condition**:
`III-6`'s ghost index, and therefore slot reuse, rests on owner decision **O3**
(§6, D5). No rule gaps.

---

## Summary

| Item | `CASES.md` / `PROGRAMS.md` expectation | This candidate | Deciding rules | Verdict |
|---|---|---|---|---|
| **B07** | `read(p)` REJECT; `write(p,20); read(p)` accept | same; `put`/`replace` also refused on their own premises | `III-7`, `II-3`, `I-6`, `I-7`, `I-8`, `I-9`, `JF-TERM` | accepted as expected |
| **B08** | ACCEPT (20 / 10) | same; the second guard is the *same* frozen atom, `II-1` discharges both arms | `III-7`, `ATOM`, `II-1`, `I-8`, `II-3` | accepted as expected |
| **B09** | REJECT; `if !cond { put }` variant accepts | same; `c1 = ¬c0` recorded, `JF-ENT`(4) propagates it | `III-7`, `JF-ENT`, `II-1`, `II-3` | accepted as expected |
| **B10** | join retains one obligation per path; no intersection, no omission | same; `own('A)`, `own('B)` are correlated one-atom terms | `II-3` rows 4–5, `TERM`, `II-5` | accepted as expected |
| **B11** | both lines ACCEPT; no obligation remains | same; leafwise strong release through the origin term | `II-14`, `II-15` row 1, `II-5`, `I-4`, `I-3` | accepted as expected |
| **B12** | REJECT at the second release; `if cond / if !cond` accepts | same, statically; no drop flag | `I-4`'s constancy premise, `II-1`, `JF-TERM`, `III-7` | accepted as expected |
| **B13** | two policies stated as alternatives, neither selected | explicit consumption selected; automatic cleanup refused at the scope exit | `I-3`, M2(ii), R2's Price | **deliberate refusal with a repair** (write the guarded release) |
| **P1** | proof-based legality, parallel writes, a naming diagnostic, no runtime check | all four hold; five index facts must be written as one branch, no `use` route | `III-1`, `III-12`, `KEY`, `I-7`, `I-8`, `I-16`, `JF-QI`, `JF-ENT`, `M3` | accepted with a stated cost; **2 rule gaps** |
| **P3** | multi-path links, removal invalidates every path, parallel node writes, one reused invariant | all four hold; traversal read and linked parallel map each need a written fact; rests on O3 | `II-16`, `I-24`, `I-23`, `III-6`, `III-5`, `III-12`, `KEY`, `JF-QI` | accepted with a stated cost |

### Rule gaps recorded (not improvised around)

| Item | Line | Missing rule |
|---|---|---|
| P1 | `par { write_part(qi,·) \|\| write_part(qj,·) }` | No rule states a distinctness effect for an `I-2` frame mint. `I-1` requires the allocator's written `ensures` for the heap case and `R4(b)` forbids `#` from name inequality; whether two frame slots are distinct is left to `JF-OVL`'s same-base restriction, a procedure detail rather than a stated rule |
| P1 | `} else { /* fall through */ }` after a `push` in the then arm | `II-3`'s join table has rows for `st`, `own` and `origin` only. No rule states how a bridge fact (`III-1`/`III-2`/`III-3`) or an `RSt` residual list joins when the edges disagree; `II-6` states the loop-head discard and `II-10` the call-boundary discard, the `if`-join is unstated |

### What this part contributes to the owner's open decisions

- **O11 (condition terms at joins).** B07–B13 are the part of the corpus that
  lives or dies on this decision, and under (b)-with-a-bound every one of them
  derives: B08, B09 and B12.2 are accepted *because* `II-1` shrinks a term on
  arm entry; B10 and B11 are accepted *because* `II-3` may carry one atom and
  `II-5` keeps several facts correlated on it. Under (a) (premises only) B10's
  `own` join has no value, B11's `read(remaining)` and `release(remaining)` lose
  the correlation, and B12.2 loses its discharge. No item in this part pays
  `TERM`'s price: none of B07–B13 nests two independent divergences, so `II-2`'s
  refusal class is never reached here. The price is paid elsewhere in the
  corpus, not by the branch cases.
- **O7 / O12 (projections second class, references never stored).** This part
  supplies one piece of evidence: B11's `remaining` is an ordinary copied
  locator whose origin is a term, and P3's link writes go through unpacked
  stored pointers. `II-13` + `I-19` admit both; the price shows up at P3, where
  `II-16`'s "two unpacks are not distinct" forces a written pool invariant for
  the parallel map, and at `JF-OVL`'s per-symbolic-access cost in P1.
- **O3 (is a ghost index a mechanism M2(ii) refuses).** P3's slot reuse is
  wholly conditional on it (§6, D5).


# File: critique-value-semantics-soundness.md

# Critique: `value-semantics`, lens SOUNDNESS

Refuter report. Research date: 2026-09-16. Scope assigned: **P4, P7, P8, B04,
B05, B08, L03, L05**, re-derived from
[rules-value-semantics.md](rules-value-semantics.md) alone before the three
`derive-value-semantics-*.md` files were opened, then compared against them.

Method, as required: each item was derived line by line from sets I, II and III
and §5's family table with no reference to the derivation files; the derivations
were read afterwards; every line where a stated verdict does not follow from the
rules is recorded below, together with every hazard from **R1, R2, R3, R11, R12**
the rules admit, every **M2 clause (ii)** breach carried as a library or a typed
outcome, and every rule that consumes a fact it never sources.

Severity: **fatal** = a hazard admitted, or a settled requirement violated with
no declaration; **serious** = a wrong verdict, an understated cost, an unnamed
family, a missing rule; **minor** = wording.

Nothing here selects or ranks a candidate.

---

## 1. Findings table

| # | Sev | Item / line | Rule | Finding | Fix |
|---|---|---|---|---|---|
| **F1** | fatal | L03, `g.slots[σ(q)].value = v` (derive-boundary L03); also B04's `g.slots[σ(q)].value = v` | **I-7** | **R2(a) leak admitted.** I-7 bundles plain assignment-over with `replace`, and its premises are only "d resolves" + "the conflict table permits `inout` at d". Nothing requires the value *currently at d* to be Copy or to be handed back. Assigning over a slot holding an affine value silently drops its release obligation, on every path, with no diagnostic. The derivation states the hazard out loud — `// accept: I-7 — total; no "empty" fact needed either` — and L03 is safe only by the accident that `old_b = take(b)` emptied the slot first. Nothing in sets I–III recovers the dropped obligation: I-3 releases *names*, and the leaked value is inside a path. | Split I-7. Plain `d = e` is writable only where the current value at `d` is Copy; otherwise the only writable form is `replace(inout d, sink e)` whose returned old value is a binding (I-1) that R2 forces the writer to consume. |
| **F2** | fatal | I-21 `compact`, and the whole pool rendering that P7, B04, B05, L03 stand on | **I-21 vs III-6 vs II-15(a)** | **R1(a)(ii)/(iii) hazard: compaction invalidates every handle copy while `G` frames liveness in.** III-6 fixes `σ(h) ≡ h.idx` and makes `p.slots[σ(h)]` an *ordinary element path*. I-21 then claims "σ(h) is re-read from the handle's own word, which the compaction rewrote: the handle IS the fixup form". `compact(inout p.slots, inout p.meta)` cannot rewrite `h`: II-15(a) confines a body to its `inout`/`sink` footprint, and `h` is a local one-word Copy binding outside it. Handles are "ordinary copyable data scattered through the program" — I-21's *own* reject line for `compact_renumber` says the checker cannot enumerate them, and that is equally true of `compact`. So after `compact`, `p.slots[h.idx]` reads a **different live row** (or, if `cap` shrank, outside the plane, since PBounds is claimed never-killed), while I-21 explicitly frames `LIVE(h)` **in**. P13's required property is delivered unsoundly. | Either (a) declare compaction out of the language, as II-23 declares split-and-rejoin out; or (b) make the access a two-step indirection `p.slots[p.map[h.idx]]` through a written slot map — and then withdraw III-6's "statically total, no bounds compare, one element path" claim, add the map's own R11 domain rule, and re-price M9's "one indirection per pooled access" as two. |
| **F3** | fatal | I-10 `reserve` + II-22; same shape for I-9 `push` | **I-10, II-12, II-17, II-22** | **R5(a)/R1(a)(ii) hazard: a relocating operation whose footprint omits the plane it relocates.** I-10 declares `FP = {(v.meta, inout)}` and celebrates that `v.data` is absent ("element contents facts survive by the rule… the case the base could not state and P9 turns on"). But `reserve` is by P9's own definition the operation that *replaces `v`'s backing*. II-17's table and PATH then find **no overlap** between `{v.meta}` and `{v.data[0..4]}` — distinct planes of one backing, disjoint by G3 *with no proof* — so `par { reserve(inout v.meta, 128); fill(inout v.data[0..4]) }` is **permitted by II-22 ground (i)**, with `reserve` freeing and moving the storage the sibling branch is writing. No rule gap is needed for this one; the rules as written license it. The sequential variant (an open `with` projection across a `reserve`) is flagged as a *missing rule* in the P1 regression of `derive-…-late-branches.md:514`, but it is flagged there as a gap, never as a hazard, and the `par` form is not derived anywhere. | A relocating operation must carry the relocated plane in its footprint, or a third marker (beside `carve`) that PATH treats as overlapping **every** path under that backing. I-10's headline win then disappears, and the P9 claim that rests on it must be restated. |
| **F4** | fatal | II-14's worked example, `let c_i_lt = c.i < len(v)  // support {v.meta}` | **§1.3 vs II-14 vs II-12** | **R11/R1(a) hazard from an under-specified support set.** §1.3 defines support as "the set of resolved paths whose **contents it depends on**". `c.i < len(v)` depends on the contents of `c.i` and of `v.meta`. II-14's example assigns it support `{v.meta}` only. Taken as the rule's own example says, `c.i = 99` does **not** kill the fact: FRAME (II-12) keeps every fact whose support the footprint `{(c.i, inout)}` does not name, and III-3's kill-by-name fires only on *index steps inside a support path* — `v.meta` has none. The next `v.data[c.i]` is then accepted with a stale bound. This is an out-of-bounds read the rules admit. (P4.b avoids it only because the derivation silently uses the §1.3 reading, `c.i == 0` with support `{c}`, and re-derives through REF instead of through OLDCHAIN.) | State the support-closure rule once — support is every resolved path the fact reads, transitively through path steps — and correct II-14's example. |
| **F5** | fatal | L05's named repair; B10, B12, B13, L04's identical repair; §5.1 row M2(ii); §6 DV list | **II-2 + III-11 + I-7** | **M2 clause (ii) violated, carried as a typed outcome, and declared satisfied.** §5.1 asserts "No drop flag — II-2 makes the mixed-consumption case a rejection, so no flag is needed and **none is representable**", and both derivations repeat it ("no drop flag appears because no mixed consumption survives II-2"; "with no drop flag — the discriminant is the writer's own word"). The repair the rules themselves mandate for every conditional-release case is `var slot: Option<Obj>` + `replace(inout slot, None)` + a trailing `match slot { Some(x) => sink x, None => () }`. That discriminant word *is* a drop flag: it is bookkeeping state whose only content is "has the obligation already been discharged", and the trailing arm is a runtime branch selecting whether the release executes, on a predicate **the checker did not discharge** — the exact shape M2(a)(ii) refuses by name ("a Vale-style check wrapped as a library outcome… the check's condition is a safety predicate the checker did not discharge"). It satisfies the letter of "the source did not bind" only because II-2 forces the writer to bind it. P8's stated requirement — *"no runtime drop flag; the writer's own branches carry the state"* — is therefore not delivered for the five cases P8's own title scopes it to (B10–B13, L04–L06), and no row declares this. | Declare it. Add a `§5.1` row: M2(ii) satisfied **in letter only**; the flag is relocated from compiler to source by II-2, the release becomes runtime-selected on a writer-bound discriminant, and P8's "no drop flag" conjunct is a **capability loss**, not a delivery. Then let O11 be decided against that cost rather than against "the flag disappears". |
| **S1** | serious | P8 verdict, `derive-…-boundary.md` §P8 and its summary row | **II-2, II-5, III-10** | **Wrong verdict.** The section concludes "**accepted as expected, with a stated cost… All three required properties hold**", listing "the writer's own branches carry the state" among them. II-2 is precisely the rule that forbids a writer's branch from carrying binding state; the repair works by making *both* arms unbind *both* names, i.e. by deleting the branch-carried state the requirement asks for. And P8 is scoped by PROGRAMS.md as "**P8 Conditional release and loop exits (CASES.md B10-B13, L04-L06)**"; five of those six reject and repair with F5's flag. | Restate P8 as **differs: capability loss** — conjunct 1 (no drop flag) is delivered only by relocating the flag (F5); conjunct 2 (branches carry the state) is refused by II-2; conjunct 3 (the rejection names the path) is delivered. |
| **S2** | serious | L03 verdict, in *both* files, and Appendix B's L03 row | **III-3, I-19, II-7** | **Wrong verdict, and the two files contradict each other.** Appendix B: "ACCEPT **with one written loop invariant**… the relative invariant (p's target `is_some`, q's target `is_none`) is writable and verified by INV; the case's ACCEPT survives at one written line." The derivation: "**accepted as expected** (pool rendering, **zero annotations**)… the relative invariant… is writable and verified by INV, but it is **not needed**." Neither holds. III-3 resolves a path step `[e]` only if `e` is stable *for the fact's span*, and names `p`, `q` are assigned inside the loop body, which is the invariant's span — so `g.slots[σ(p)].value` **does not resolve** at the loop head and the relative invariant is **unwritable**, not merely unneeded. The derivation says as much two lines earlier ("III-3 kills every contents fact whose index term mentions the reassigned `p`/`q`") and then asserts writability anyway. Consequently the case's ACCEPT line, `read(p) // ACCEPT: 10`, is not derivable in any rendering. | Reclassify L03 as **capability loss**: the invariant the case is *about* — a relative, role-alternating invariant over two rebindable names — is not statable, because the only rebindable name form (a handle) is exactly the form III-3 refuses in a path step whose span contains its assignment. Reconcile Appendix B with the derivation in the same edit. |
| **S3** | serious | I-19's third example; `derive-…-early-branches` B05 (`Γ: ha.idx != hb.idx — σ' is FRESH`); B04, B06, L03, P3 | **I-19, III-8, R4(b), O8** | **A rule needs a fact it never sources, and the source it implies is a refused one.** I-19's effect says only "a FRESH ghost slot σ ∉ Live(P)". Its example then uses `σ' != σ(h) **because σ' is fresh** (III-8)`, and III-8 states no such rule. Every distinctness of two inserted handles in the pool rendering rests on it. Worse, grounding R4 distinctness **in freshness at a minting site** is exactly the alternative O8 puts to the owner, whose position taken in VERDICT-CORE (G5 included, O8(a) "Never… re-derived from containment") refuses it. The rules neither state the rule nor declare the departure. | Either add a minting-freshness rule to set I and declare the O8(a) departure in §5.1, or ground pool-slot distinctness in the pool's own written invariant and pay the `use` step at every pair. |
| **S4** | serious | B05, B06; Appendix B's B05 row and its named repair | **II-3** | **A missing rule decides the candidate's only two capability losses.** II-3 says `Γ := ⋂_i Γ_i, compared by syntactic equality`. It never says whether `Γ` is the *asserted* fact set or its entailment closure. On B05's two arms `σ(p) != σ(q)` is **true and derivable** (from `p == ha`, `q == hb`, `ha.idx != hb.idx`), but it is not a member of either `Γ_i`. Closed-before-intersection ⇒ B05 and B06 **accept**; not closed ⇒ they **reject**. The candidate's headline DV-6 capability losses therefore hang on an unwritten rule. The straight-line form of B05 accepts either way, which shows the join, not `remove`, is the killer — so Appendix B's attribution ("DV-6… repair: a written distinctness premise on the pool interface") is wrong twice over: wrong cause, and a pool-interface premise cannot repair a fact about two caller-local variables. `derive-…-early-branches` §4 records this; the rules file does not. | State in II-3 that `Γ` is the asserted set and is not closed under REF before intersection — and then charge the consequence honestly, because the alternative reading runs §5's cubic entailment closure (families 5/9) at **every join**, which the M10 degree table does not price. Correct Appendix B's B05/B06 cause and delete its named repair. |
| **S5** | serious | P7.b `let q = p`; B04 `let q = p`; B05/B06 `var p = ha`; L03's swap; III-6's own third example | **GHOST (§5 row 11), I-19, I-20, III-6** | **No rule propagates `G` on a handle copy or assignment.** GHOST is defined for *insert* (add two facts for one name) and *remove* (filter by EXT). Nothing states how `LIVE` reaches a name bound or assigned from another handle-valued expression. Every accept in the pool rendering of P7, B04, B05, B06 and L03 uses it, and III-6's `var hv = h; hv = other_h` example asserts the outcome without a rule. Both derivations mark it (MR/R2); the rules file does not. | Add a GHOST clause: on `x := e` where `e : Handle<P,T>`, `G(x) := G(e)` and `Γ ∪= x == e`; state its interaction with III-3's kill-by-name at the same place. |
| **S6** | serious | II-14's own worked example; DV-5 | **II-14 step 4, §1.3** | **OLDCHAIN's side condition is all but unsatisfiable, so DV-5 understates the cost.** Step 4 rewrites "every fact killed in step 2 whose support **is exactly the measure's path**". Under §1.3's support definition (F4), any useful fact — `i < len(v)`, `c.i < len(v)` — also reads the binding or path supplying `i`, so its support is never *exactly* `{v.meta}` and step 4 never fires for it. II-14's own example is then not derivable from II-14. DV-5 describes the gap as "a call with no `ensures`"; the real gap is "almost every fact". | Restate step 4 over facts whose support *contains* the measure path and whose other support paths the footprint does not name, and re-price DV-5 accordingly. Note that P4.b's accept survives either way, because it re-derives through REF from two independent facts rather than through the chain. |
| **S7** | serious | P7 verdict and its "correction to DV-3" | **II-17, R3** | **An understated verdict.** The derivation is right that the two-alias premise **is** writable in the pool rendering (II-17 judges one sequential span; P7's accesses are separate statements), and it corrects DV-3 accordingly. But the consequence is not drawn: R3's acceptance test — R3(e) names P7 and nothing else — now **exists**, and the candidate **fails** it, because R3(a)'s "a take leaves a hole" has no instance (DV-4): `read(q)` after `take(p)` accepts and returns `None`. The verdict records this as one DR among four lines; it is also a settled-requirement row losing its only acceptance test's discriminating clause. | Record R3 as **partial with a live, failing test** rather than as DV-3's "no test exists", and state which of R3(a)'s four clauses (copy, affine, linear, take/put/replace) the candidate delivers. |
| **S8** | serious | §1.1's table, I-3, I-4; the derivations' shared preamble ("the pool `g` carries its rows") | **I-3, I-4, R3** | **Two unnamed families.** (a) **No rule assigns a value class.** Copy, affine and Nominal are used throughout (I-5 "n is Copy", II-1 "c… is Copy", II-15's bound) and no rule in sets I–III says where a type's class comes from. (b) **R3's linear class has no rules at all.** I-3 runs an unconditional affine release for *every* name still bound at a brace, which is exactly what a linear value must not get; so every non-Copy value is affine and R3(a)'s linear class is unrepresentable — undeclared anywhere (II-15 declares only the *generic-instantiation* foreclosure). (c) **No rule defines what "x's affine release" is** or who writes it, which matters at I-3 for a pool binding with live rows: releasing `g` must walk runtime occupancy, and III-8 forbids `Live(P)` from being extensional. The derivations assert "the pool `g` carries its rows" with no rule behind it; if that walk is compiler-generated it is an M2(ii) breach, and if it is the library's own written drop it needs II-7's per-pool liveness clause and a rule saying so. | Add a value-class rule to set I; declare the linear class absent (a stated R3 capability loss); state the affine release as a *written* operation on a nominal, and derive the pool's own release under II-7. |
| **S9** | serious | I-15 `carve`; III-4 | **II-12, I-15, III-4** | **FRAME at `carve` needs a fact III-4 says is unavailable.** FRAME is defined only over footprint entries with `m ∈ {inout, sink}`, so a `carve` entry kills **nothing** — which is sound only because the arena invariant IA really holds of every issued live block. But III-4 says the connection between the bump offset and the issued extents "is the WRITTEN arena invariant IA, **never an inferred relation**… re-derived by `use IA(b, b')`, a written finite step". So IA is simultaneously a standing soundness premise of the frame rule and a fact obtainable only at a written `use`. One of the two statements has to give. | Say explicitly that IA, once verified against the arena body (I-14), is a **standing** truth available to FRAME, and that `use IA(b,b')` is needed only to obtain the *pairwise* disjointness fact in `Γ` for II-22. |
| **S10** | serious | P3's removal (`derive-…-late-branches` §P3) | **I-20, II-13, II-15(a), III-3** | **A missing rule that removes R6's and P3's acceptance tests.** A doubly linked list's `remove` must write `p.slots[σ(prev)].next` and `p.slots[σ(next)].prev`, neither of which is in I-20's declared two-part footprint; II-15(a) confines the body to its footprint, so I-20 as written cannot be the signature of a list removal. Widening to `inout p.slots` is permitted by II-13 but kills every element fact of the pool at every removal — the exact metric R6 is measured by. The neighbours' paths, `slots[σ(slots[σ(h)].prev)]`, have an index term that is a load from the plane being written, which III-3 does not resolve. No rule names a third option. | Either state a footprint form for a load-derived neighbour path (with its III-3 resolution rule), or record that R6's P3 acceptance test and P3's first required property are not delivered. |
| **S11** | serious | B04's final two lines | **I-19 (MR-3), II-3** | **Understated class.** The derivation correctly reports that the values 10 and 20 are not derivable, but files the whole case as **DR**. The case's stated conclusion — "afterward **both original slots are initialized**" — is not a refusal with a repair: under these rules there is no "initialized" predicate to obtain, so that sub-claim is a **case correction (CC)**, in the same family as B08/B09's, and B04 should carry two classes. The `Option` repair DV-4 names does not restore it either, because I-19 states no contents `ensures` at all (MR-3): no value ever entering a pool row is relatable to the row's path. | Give B04 two rows, DR for the intermediate reads and CC for the initialization conclusion; and add a contents `ensures` to I-19, or state MR-3 as a defect in §6. |
| **m1** | minor | §5.1 row M10 and §5's degree table | **II-3, families 5/9** | The degree table charges the cubic entailment closure to `EXT`/`REF` per call site. If S4 is resolved the other way (closure before intersection), the same cubic closure runs at every **join** as well, and nothing in the table or in the M10 row prices joins. Even under the no-closure reading, the table should say so. |
| **m2** | minor | Appendix B counts ("9 unchanged, 13 DR, 2 CL, 2 CC") | — | Stale after the line-by-line derivations: S01/S02/S04/S05 keep their ACCEPT but lose the property each case exhibits; B05 and B06 change cause; B08 changes class; L03 changes annotation count (S2). The counts should be regenerated from the derivations or dropped. |
| **m3** | minor | B08, and §Renderings in `derive-…-late-branches` | III-11 | The verdict "accepted as expected" and Appendix B's "CC" are two names for the same result; pick one. Separately, the rendering fixes B07–B09's contents as **Copy integers** (declared, not hidden), so B08 establishes nothing about the affine payload — where `let old = replace(inout a, None)` inside the arm would be released at the arm's brace by I-3, silently, which is the behaviour the case was written to interrogate. Worth one line in the verdict. |
| **m4** | minor | II-2's repair example `let s = if c { sink a; b } else { sink b; a }` | II-2, II-3, II-4 | MR-4: no rule states which `Γ`/`G` facts a name bound *at* the join from an arm-yielded value carries. The example is the rules file's own headline repair for B10/B12/B13/L04 and is not covered by any rule. |
| **m5** | minor | I-21's signature | I-21 | `ensures forall h. Live(h) => Live(h)` is a tautology and states nothing. All the work is done by the prose clause "Live(P) and gen(P[·]) are NOT in the footprint" — which is exactly the clause F2 shows to be unsound. |

---

## 2. Hazards admitted, by requirement row

| Row | Hazard the rules admit | Where |
|---|---|---|
| **R1(a)(ii)** | after `compact`, every stored copy of a handle names a row it no longer denotes, while `LIVE` is framed in | F2 |
| **R1(a)(ii)** | a `par` branch writes elements of a plane that a sibling branch's `reserve` has freed and moved | F3 |
| **R1(a)(iii)** | same, as a stale-layout access when `cap` shrinks under compaction and PBounds is declared never-killed | F2 |
| **R2(a)** | assignment-over an affine destination drops the destination's release obligation | F1 |
| **R2** (indirect) | the release of a pool binding at a scope exit has no rule; if it is compiler-generated it walks runtime occupancy | S8(c) |
| **R3(a)** | the linear class has no rules; I-3's unconditional release makes every non-Copy value affine | S8(b) |
| **R3(a)** | "a take leaves a hole" has no instance on R3's only acceptance test, which is now writable | S7 |
| **R11** | `v.data[c.i]` accepted after `c.i` is overwritten, because II-14 gives the bound fact a support that omits `c.i` | F4 |
| **R12** | not breached in the assigned items. The `None` arms of I-15 and III-11 are written arms under O5(c); recorded as clean. | — |
| **R5(a)** | overlap permitted between a relocating operation and an access to the plane it relocates | F3 |

**M2 clause (ii)** breaches carried as a library or a typed outcome: **one**, F5
— the `Option`-slot repair mandated by II-2 for B10, B12, B13, L04 and L05. No
other item in scope required a check the source did not write: P4.b's domain
comes from `requires` + VecOk, and P7/B04/B05/L03's pooled accesses come from
PBounds, both statically.

**Rules that consume a fact they never source:** S3 (freshness ⇒ slot
distinctness), S5 (`G` on a handle copy), S4 (whether `Γ` is closed at a join),
S9 (IA standing vs `use`-instantiated), S8(a)(c) (value class; the meaning of
"affine release"), S10 (a load-derived neighbour path), plus the derivations'
own MR-1 (use-direction path equality under `p == q`), MR-2 (`old()` over a
non-measure path) and MR-4 (facts of an arm-yielded value).

---

## 3. Verdicts overturned

| Item | Stated | Overturned to | Ground |
|---|---|---|---|
| **P8** | "accepted as expected, with a stated cost… all three required properties hold" | **differs: capability loss** | S1 + F5: conjunct 2 is refused by II-2; conjunct 1 is delivered only by relocating the drop flag into source, across the five cases P8's own title scopes it to |
| **L03** | derivation: "accepted as expected… zero annotations"; Appendix B: "ACCEPT with one written loop invariant" | **differs: capability loss** | S2: III-3 does not resolve `g.slots[σ(p)].value` in a span where `p` is assigned, so the relative invariant is unwritable and the case's ACCEPT value is not derivable in either rendering |
| **L05** | "deliberate refusal with a named repair… the flag disappears" | **differs: capability loss** (DR only for the *rejection*; the repair is not a repair of the requirement) | F5: the trailing `match` on the `Option` discriminant selects the release at run time on a predicate the checker did not discharge — the flag is relocated, not deleted |
| **P7** | "differs: deliberate refusal with repair… 3 of 4 lines exact" + "correction to DV-3" | **differs: deliberate refusal, and R3's now-live acceptance test fails** | S7: the DV-3 correction makes R3(e)'s test writable; the candidate then fails R3(a)'s hole clause on it |
| **B04** | "differs: deliberate refusal with repair" | **DR + CC** (two classes) | S11: "both original slots are initialized" is a case correction, not a repairable refusal; MR-3 leaves no route to it |
| **B05** (Appendix B's row only) | "DV-6… repair: a written distinctness premise on the pool interface" | **cause is II-3's unclosed join; no pool-interface premise repairs it** | S4; `derive-…-early-branches` §4 already overturns it, the rules file does not |
| **§5.1 row M2(ii)** | "satisfied (under O3(a))" | **satisfied in letter only; declared violation owed** | F5 |
| **§5.1 row M1** | "satisfied, with a correction (twelve families)" | **stands**, with S4's caveat: if `Γ` is closed before a join, the join runs the cubic closure and the M10 degree table is incomplete | m1 |

Not overturned: **P4** (capability loss, correctly classed; the second read's
accept follows from I-9/II-11/II-14 and REF, and the `sink v` refusal from I-4);
**B08** (the read accepts and the correlation is not recoverable — both follow
from III-11 and II-5/III-10); **B05/B06** in the *derivation* (their reject
follows from II-3 as the derivation reads it, and the derivation names the
reading).

---

## 4. Evidence for the round's open decisions

- **O11.** The refuter's independent derivations reproduce the derivation files'
  O11 evidence at B08 and L05, and add F5: under O11(a) the price is not "one
  restructuring" but a **writer-bound drop flag with a runtime release-selecting
  arm** in every conditional-release case. O11 should be decided against that
  price, not against "no flag is representable".
- **O7 / O12.** P4's first conjunct is unwritable exactly as DV-1 says. F2
  sharpens the boundary: the pool rendering, which is the *only* rendering that
  restores a storable, rebindable name, is the rendering whose bridge (III-6)
  contradicts its own relocation rule (I-21). A second-class-projection boundary
  therefore does not by itself buy back P4 — it buys back P4 only if compaction
  is declared out of the language.
- **O8.** S3: the pool rendering's distinctness rests on minting freshness,
  which O8(a) — the position VERDICT-CORE takes with G5 — refuses.


# File: critique-value-semantics-cost.md

# Critique: candidate `value-semantics`, lens **COST AND DETERMINISM**

Target: [rules-value-semantics.md](rules-value-semantics.md) (1717 lines, rule
sets I/II/III, judgment families §5, declared violations §5.1, defects §6,
Appendix A/B) and its three derivations
([straight-and-early-branches](derive-value-semantics-straight-and-early-branches.md),
[late-branches](derive-value-semantics-late-branches.md),
[boundary](derive-value-semantics-boundary.md)).

What this lens checked, in order: (1) every judgment family for a fixed,
terminating, search-free procedure (M1) with a stated degree (M10); (2) the
condition-atom bound, which this candidate claims it does not need (II-5,
O11(a)); (3) the count of constructs the rules introduce (M8); (4) what the
writer must write per derived item, tabulated; (5) M7 — is every rejection in
the derivations locally repairable from the diagnostic the rules would emit.

Nothing here selects, ranks or repairs a candidate. Severity per the round's
definitions: **fatal** = a hazard admitted or a requirement violated
undeclared; **serious** = a wrong verdict, an understated cost, an unnamed
family, a missing rule; **minor** = wording.

---

## 1. Findings

### 1.1 Fatal

| # | Item / line / rule | Finding | Fix |
|---|---|---|---|
| **F1** | rules §II-14 step 1, line 947 (and step 4, line 950); §III-13 | **OLDCHAIN's input is not unique, so acceptance is selected by the fact set's internal order, not by the source bytes (M1(a) violated, and §5.1 declares M1 "satisfied").** Step 1 binds `old(m_i(x)) :=` *"the term for `m_i(x)` held in Γ"*. Γ is a set and routinely holds several: III-13's own example produces `n == len(v)` and then `let m = len(v)`, after which two equalities name the same measure. Step 4 has the same hole in the other direction — it substitutes *"the new ensures' relation for the measure"* where a signature like I-9's `ensures len(v) == old(len(v)) + 1 && cap(v) >= len(v)` offers two relations mentioning `len(v)`. The rewritten facts differ syntactically per choice, and II-3 joins by *syntactic* equality (line 696), so two implementations of the same specification can disagree on a later join. This is exactly "state outside the specification selects acceptance". | State a selection rule that is a function of the source: bind `old(m)` to the measure symbol itself (not to a held term) and require at most one measure-defining `ensures` clause per call, rejecting a signature with two relations over the same measure at the declaration. |
| **F2** | rules §5 table (lines 1584–1615) vs II-1/II-5/II-18/III-3/I-22/II-2–II-4/II-10 | **At least six procedures the rules invoke are named in no family and carry no degree, three of them running at *every statement*; §5.1's M10 row (line 1624) declares only the edit-stability clause violated and calls everything else "at the ceiling", which cannot be asserted of a family with no stated bound (M10(a) requires "each admitted automatic family has a stated worst-case bound").** Unnamed: (a) **PREMISE** — push at II-1, discard at II-5, arm discriminants at II-6, and III-10's kill of a premise when a name in its support is assigned; this needs a per-premise support set and a per-statement rescan that no family describes. (b) **FOREIGNKILL** — I-22 line 598, "at EVERY statement s, before s is checked", scans all of Γ against every `Foreign` binding in scope: `O(F·V)` per statement, `O(N·F·V)` per declaration, the largest per-statement cost in the system and nowhere counted. (c) **NAMEKILL** — III-3 line 1287, a kill by *name* over all of Γ at every assignment, `O(F·d)` per assignment; RESOLVE (family 2) covers only the stability *test*, not the kill. (d) **REKEY** — II-18 line 1058 rewrites the support of every fact derived inside a projection block at the close. (e) **ΓG-JOIN** — II-3/II-4 intersections; family 1 BIND covers `Σ` only, and the `Γ` intersection by syntactic equality is `O(F²)` naive or `O(F log F)` with a stated normal form (there is none, see S8). (f) **DECL** — II-10 line 857, deciding that a `requires` is "syntactically unsatisfiable" and that a result names an unnameable origin. | Add these six to §5's table with inputs, procedure, termination argument and degree, and restate §5.1's M10 row as violated in the degree clause as well as the edit-stability clause; the three frame exceptions of Appendix A in particular must appear as families, since they are the three highest-frequency procedures in the candidate. |
| **F3** | rules §5 families 6, 7, 10, 11 (lines 1609, 1610, 1613, 1614) | **The stated degrees of CONFLICT, FRAME, INV and GHOST omit the EXT/REF entailment closure they call, understating each by the closure's cubic factor; with the omission repaired the "at the ceiling" claim of §5.1 is false and M10's second ceiling clause is breached undeclared.** FRAME is given as `O(F·k·d)` "per call", but its own definition (II-12) says `PATH_may_overlap` is "the containment-tree prefix test **refined by EXT**", so FRAME issues `F·k·d` EXT queries, each a cubic transitive product: `O(F·k·d·C³)` per call. CONFLICT is `O(1)` *"after PATH/EXT"* — the "after" hides the same factor, at `O(k²)` queries per statement and `O(j²)` per open-projection span. GHOST is `O(H)` *"EXT queries"* per remove, i.e. `O(H·C³)`. INV is `O(S·|Inv|)` "linear in written steps" although its second half is "premise discharge by **REF**", which §5 itself says "shares EXT's closure; cubic in `C`". M10's per-declaration clause ("total work per declaration at most quadratic in written proof steps") is never evaluated anywhere in §5.1, and in the derivations `S = 0` for most items while the work is not constant. | Restate each dependent family's degree as the product of its query count and the closure's degree, then evaluate M10's per-declaration clause explicitly; if the product is above the ceiling, declare M10 violated in the degree clause rather than "at the ceiling". |

### 1.2 Serious

| # | Item / line / rule | Finding | Fix |
|---|---|---|---|
| **S1** | rules §5.1 M10 row, line 1624 | **Understated cost: the row reports EXT/REF as "at the ceiling, not above it, but measured at exponent ≈ 2.6–3.4". An exponent of 3.4 is above cubic.** The row also compares two different quantities without saying so: the ceiling is stated in `C` (ProofContext terms) and the measurement is in `N` (bundle size), so the measurement neither confirms nor refutes the stated degree. | Report the measured exponent against its own quantity, and state `C(N)` so the measurement and the ceiling are comparable; if `C` grows with `N`, the composed exponent is what M10 judges. |
| **S2** | rules §5 preamble quantities (line ~1590), families 5, 7, 9 | **`F` (facts in Γ) and `C` (ProofContext terms) are used as the measures of the three most expensive families but are never bounded by a syntactic quantity, so no M10 verdict follows from the table at all.** M10(a) explicitly requires the bound to be "in program size including instantiations" (SYS-D0-15). Every rule that adds facts — I-1's initializer refinements, II-11's `ensures`, II-14 step 4's rewrites, III-2's per-plane length facts, and the pairwise handle distinctness S5 shows inserts must materialize — feeds `F` and hence `C` with no stated cap. | State `F` and `C` as functions of program size (statements × facts-added-per-rule, instantiations included), or declare M10's degree clause unverifiable for this candidate. |
| **S3** | rules §II-5, lines 739–740 | **Understated cost: "there is no condition-atom growth to bound (M10)" is true of the *state* and false of the *cost*.** Π carries the `requires` terms, every syntactically enclosing branch condition and every match discriminant, and families 5 and 9 read Π as an input: every condition atom in scope enters the entailment closure and is multiplied cubically. At branch nesting depth `D` with `a` atoms per condition the closure carries `D·a` extra terms at every access inside the nest. The candidate refuses condition *terms in facts* (O11(a)) but still pays a condition-atom cost per access, and states no bound on `|Π|`. | Bound `|Π|` explicitly (nesting depth × atoms per condition + `requires` atoms), and charge it into `C` in family 5's degree. |
| **S4** | rules §II-7 line 783; families 1 and 11 | **Unnamed iteration and an unstated pass bound: `G at the head := G_entry ∩ G_backedge` is a genuine fixpoint whose height is the number of ghost facts (`2H`), not the height-2 lattice the same rule argues for `Σ`.** `Γ` is given a one-pass syntactic shortcut ("any fact whose support is written anywhere in the body is absent from Γ_backedge"); `G` is given none, so each removed liveness fact can require another pass over the body, up to `O(H)` body re-checks per loop and `O(H)^depth` for nested loops. No family owns the loop-head fixpoint and no degree is stated for it. | Give `G` the same one-pass syntactic shortcut as `Γ` (drop the facts of every handle name of a pool the body removes into), or state the fixpoint as a family with height `2H` and its body-recheck cost. |
| **S5** | rules §I-19 (line 509), §III-8 (line 1427); derivation S01 line 87 and B01 | **Missing rule with a cost consequence: no rule adds the freshness distinctness of two inserted handles to Γ, yet GHOST's escape (`EXT proves σ(k) != σ(h)`, line 543) and every derivation that survives a removal depend on it.** I-19's *effect* clause adds two `G` facts and applies FRAME; only its example comment claims "σ' != σ(h) because σ' is fresh". If the rule is repaired the way the derivations use it, each insert must materialize `O(H)` pairwise inequalities, so `H` inserts add `O(H²)` facts to Γ — a quadratic growth in the cost centre DV-11 already names, invisible in family 11's `O(H)` degree. | Add the distinctness to I-19's effect explicitly and charge the `O(H)` facts per insert to family 11's degree, or state that the escape is available only from a written `requires`/invariant and remove the freshness claim from I-19's example. |
| **S6** | rules §I-20 example, line 562 (`use Links(other); let ok2 = …` marked **accept**); derivation late-branches, P3 "Removal" | **Wrong verdict in the rules' own example, and the repair it names does not exist.** `Links` is written as `forall σ ∈ Live(P). Live(slots[σ].next) && Live(slots[σ].prev)`; instantiating it at `other` needs either `σ(other) ∈ Live(P)` — the very fact being re-derived — or a witness handle `w` with `Live(w)` and `slots[σ(w)].next == other`. INV (family 10) performs "first-order syntactic instantiation at the WRITTEN arguments, then premise discharge by REF", and REF has nothing to discharge that premise from. The line must reject. Because DV-6's named repair *is* this `use` step, DV-6 has no repair at all, and the same gap makes P3 underivable. | Either supply a witness form (`use Links(w)` with `w` the live neighbour, plus a rule relating `slots[σ(w)].next` to a handle name), or withdraw the `use` step as DV-6's repair and record DV-6 as unrepairable. |
| **S7** | derivations MR-4 (straight §3) ; rules §II-2 line 661, §II-3 line 690, §II-4 line 716 | **Missing rule, load-bearing for the candidate's most-used repair: nothing states which `Γ` and `G` facts a name bound *at* a join from an arm-yielded value carries.** II-2's own repair example `let s = if c { sink a; b } else { sink b; a }` needs the transfer for `Σ`, and this is the named repair for B10, B12, B13 and L04 and the rendering route for B01. Under the rules as written, those repairs cannot be shown to work. | Add a join rule for a value yielded by arms: the bound name carries the intersection of the arms' facts about the yielded value, rekeyed to the new name (the II-18 rekey shape). |
| **S8** | rules §II-3 line 696; §II-12 | **No normal form for facts is stated, so the join's "syntactic equality of (fact, support)" makes acceptance depend on the writer's spelling.** Two arms that establish `3 < len(v)` and `len(v) > 3` lose the fact at the join; `i+1 <= len(v)` and `i < len(v)` likewise. The same silence makes the `O(F²)`/`O(F log F)` choice in the ΓG-JOIN procedure (F2(e)) undecidable. | State a normal form for facts and supports (canonical relation direction, canonical term ordering) and make II-3's comparison equality of normal forms. |
| **S9** | rules §5 preamble, line ~1586 ("VERDICT-CORE §1.3 names six judgments … needs twelve"); §5.1 M1 row | **Understated family count: the correction from six to twelve is itself short by at least six (F2), so the "complete set these rules invoke" claim is wrong a second time.** The correct count from the rules as written is ≥ 18. | Publish the full inventory (12 + the six of F2) and drop the completeness claim, or state the closure condition under which the inventory is complete. |
| **S10** | rules §5.1 M8 row, line 1628 ("Count now **17**") | **Understated construct count.** Not counted: the ghost vocabulary the writer must *write in signatures and invariants* — `σ(h)` in a footprint path, `Live(·)`, `gen(P[·])`, `extent(·)`, `disjoint`, `injective` (I-18, I-20, I-14, III-9); `Foreign<T>` plus its declared access class (I-22); `par` / `par for` (II-22); the `split` M3 entry with its `extent ++` vocabulary (I-17); the measure binder `len m` inside `plane<T, len m>` (III-1); `NoReach` (§1.3's type table, family 12). A count of the taught surface from the rules as written is ≥ 24. | Recount the taught surface over the whole file, including ghost-term syntax the writer must produce, before M8 is handed to O6. |
| **S11** | rules §1.4 table, lines 83–85; derivations B10, B11, B12, B13, L04, L05, P8, B01 | **Understated writer cost: the table's cost model has rows for annotations, invariants and `use` steps but no row for *restructuring*, and it claims "0 in the common case" at the statement, "0 or 1" at the loop head and "0" at the join.** Eight of the 26 derived items are repaired only by rewriting the program's data model or control flow (§2 below), which is the dominant writer cost in the derivations and does not appear in the model at all. | Add a "restructuring" row to §1.4 with its own count, and cite §2's per-item tabulation as its measurement. |
| **S12** | rules §1.4 "statement … `use <Inv>(args)` … 0 in the common case"; §I-20 / §III-8; derivation P3 | **Understated writer cost at the only place the candidate charges `use` steps: III-8's rule costs one `use` per *surviving handle name in scope* per removal, not one per removal.** A list or graph program with `H` live handle names pays `O(H)` written steps per removal and `O(H)` per iteration of a removal loop — quadratic in handle names over the loop — and by S6 those steps do not currently discharge. | State the `use`-step cost as `O(H)` per removal in §1.4 and in family 11's degree, and name the interface clause (a written `requires` on the pool) that would replace it. |
| **S13** | rules §5.1 declared-violations table, lines 1617–1628; DV-2, DV-3, DV-5; derivation P4 verdict | **M7 is violated and the violation is not declared: the table carries rows for M1, M2, M3, M4, M5, M8 and M10 and no row for M7 (nor M6, M9, M11).** The derivations produce at least four rejection classes with no local repair: P4/DV-1's stored cursor (repair = thread `v.data`, `v.meta` through every intermediate signature — DV-2 says in terms "no local fix exists"); DV-5's `ensures`-less container writer (repair = widen a third-party callee's signature); B05/B06 (derivation: "No local repair is available"); and the MR-1 class, where the rejection has no diagnostic at all because no rule covers the goal. M7(a) requires a non-empty payload naming a tree location where a local fix exists, for 100% of rejections. | Add an M7 row to §5.1 declaring it violated, listing the four unrepairable classes; where a class is the model's boundary (DV-1), say the boundary is the reason rather than naming a repair. |
| **S14** | derivations MR-1 (straight §3); items S01, S02, S04, B03, B04 | **Missing rule: no use-direction equality.** A fact established at `g.slots[p.idx]` cannot discharge a goal at `g.slots[q.idx]` given `p == q`; II-12 and EXT only *refute* overlap, and REF's fragment says nothing about congruence over a selector. The kill direction is conservative and the use direction is silent, so a fact can be destroyed by an alias it can never be read through. Cost consequence: every value annotation in S01, S02, S04, B03 and B04 is lost, and the writer's only route is to re-establish the fact at the second path, which the rules also do not provide. | State a congruence rule (a fact may be rekeyed along a proved equality of index terms) with its degree, or declare the loss and name the rewrite the writer must perform. |
| **S15** | rules §II-14 DV-5; §5 family 8 | **Understated cost: OLDCHAIN is `O(F·e)` per call but the *writer's* cost when it fails is non-local.** Step 4 has no route when the writing operation states no relation over the measure, and the only repair is to widen the callee's signature — a third-party or library interface in the general case. This is an M7 and an M10 edit-stability cost (the edit's reach is the callee's whole caller set), and it is charged in neither. | Charge the DV-5 repair in the edit-stability clause of §5.1 and name the interface-widening cost in §1.4's signature row. |
| **S16** | derivation boundary, "Rules missing" R2 (L03); rules §5 family 11 | **Missing rule inside a named family: GHOST states `insert` and `remove` only, and no rule states how `G` propagates on a handle-to-handle assignment**, although III-6's third example and S01/B06/L03 all depend on it. | Add the assignment case to GHOST's procedure (G(dest) := G(source), and kill G(dest) when the source is not a stable term) with its degree. |

### 1.3 Minor

| # | Item / line | Finding | Fix |
|---|---|---|---|
| **m1** | rules Appendix B, L03 row ("ACCEPT with one written loop invariant") vs derivation boundary L03 ("ACCEPT, no annotation (pool rendering)") | The two documents disagree by one written line on the same item's cost. | Pick one rendering for the cost claim and state which invariant (PBounds is shared preamble, not per-item). |
| **m2** | rules §5 preamble ("Five of the six additions are constant-time or linear lookups") | FRAME is `O(F·k·d)` (and by F3 far more) and CONFLICT is `O(k²)`/`O(j²)` per span; neither is a lookup. | Reword to "four of the six are lookups; FRAME and CONFLICT are quadratic or worse in their inputs". |
| **m3** | rules §5.1 M4 row | The spelling inventory omits `Foreign<T>` and its access class, and `par` / `par for`, both of which the rules use. | Add them to the inventory. |
| **m4** | rules §5.1 M10 row | The measurement ("29.9/164.5/1691.0 ms at N = 16/32/64") is quoted without naming what `N` is in the pinned bundle. | Name the bundle quantity `N` measures. |

---

## 2. What the writer writes, per derived item

Counted from the three derivations. **Pre** = shared preamble in the pool
rendering (plane map + `PBounds`), charged once, not per item. **A** = a written
annotation or declaration (`Option` field, `requires`, capture clause). **I** =
a written invariant. **U** = a written `use` step. **R** = a restructuring: a
change to the program's data model or control flow, not a line added.

| Item | A | I | U | R | Note |
|---|---|---|---|---|---|
| S01, S02 | 0 | 0 | 0 | 0 | accept; value facts lost (S14) |
| S03 | 2 | 0 | 0 | 0 | `Option` field + `match` arm |
| S04 | 1 | 0 | 0 | 0 | `Option` field |
| S05 | 0 (+1 `sink` if affine) | 0 | 0 | 0 | |
| S06 | 2 | 0 | 0 | 0 | `Option` + arm |
| S07 | 0 | 0 | 0 | 0 | rejections as expected |
| B01 | 0 | 0 | 0 | **1** | bind before the branch; the natural `let p = if …` is MR-4 (S7) |
| B02 | 2 | 0 | 0 | 0 | sharpened variant **unrepairable** under O11(a) |
| B03, B04 | 2 each | 0 | 0 | 0 | |
| B05, B06 | — | — | — | — | **no repair exists** (capability loss) |
| B07, B09 | 1 each | 0 | 0 | 0 | `match` arm |
| B08 | 0 | 0 | 0 | 0 | accept; case's mechanism has no instance |
| B10 | 0 | 0 | 0 | **1** | yield the survivor as a value — and the repair itself is MR-4 (S7) |
| B11 | 0 | 0 | 0 | **1** | binding rendering only; pool rendering unrepairable |
| B12, B13 | 2 each | 0 | 0 | **1** each | `Option` slot + `replace`/`match` after the join |
| L01, L06 | 0 | 0 | 0 | 0 | |
| L02 | 1 | 0 | 0 | 0 | |
| L03 | 0 | 0 or 1 | 0 | 0 | m1: the two documents disagree |
| L04 | 1 | 0 | 0 | **1** | loop yields `Option`, then a `match` |
| L05 | 1 | 0 | 0 | **1** | `Option` slot; the source flag disappears |
| P1 | 1 (`requires i != j`) | 0 | 0 | 0 | accepted as expected |
| P3 | 1 (`injective(keys)`) | 2 (`PBounds`, `Links`) | **O(H) per removal** | 0 | and the `use` does not discharge (S6); three rules missing |
| P4 | 2 params × call-chain depth | 1 (`VecOk`) | 0 | **non-local** | signature cascade, DV-2 |
| P7 | 2 | 0 | 0 | 0 | |
| P8 | 0 | 0 | 0 | **1** | yield the survivor (same shape as B10) |

Totals over the 26 cases and 5 programs: **19 written annotations**, **3
invariants** beyond the shared preamble, **`O(H)` `use` steps per removal**
(unbounded, and non-discharging), **8 restructurings**, **1 non-local signature
cascade**, **3 items with no repair at all**. §1.4's model predicts "0 in the
common case" at the statement, "0" at the join and "0 or 1" at the loop head;
the join row is contradicted by eight items and the statement row by P3.

---

## 3. M7: repairability of each rejection class in the derivations

| Rejection class | Diagnostic the rules emit | Local repair? |
|---|---|---|
| II-2 definiteness at a join (B10, B12, B13, L04, P8a) | the consuming statement, the arm that did not consume, the yield-the-survivor repair | **yes in form, unverifiable in fact** — the named repair's own rule is missing (S7) |
| DV-4 (`Option` is data, S03/S06/B07/B09/L02/P7) | no rejection at all; the read is accepted | n/a — the case's refusal is gone, the repair is a rewrite |
| DV-6 (I-20 kills every handle's liveness; B05, B06, L03, P3) | "missing `σ(k) != σ(h)`", repair named as `use Links(k)` | **no** — the named `use` does not discharge (S6), and no pool-interface premise can relate two local handle variables |
| DV-1/DV-2 (stored projection; P4, B11 pool rendering) | "a projection cannot be the value of a binding" | **no** — the repair is a signature cascade, DV-2 says so in terms |
| DV-5 (`ensures`-less writer of a metadata plane) | "missing `c.i < len(v)`", repair = add an `ensures` to the callee | **no** — the callee may be a third party |
| MR-1 (use-direction equality; S01, S02, S04, B03, B04) | **none** — no rule covers the goal, so no missing-fact payload exists | **no** |
| O11(a) correlation (B02 sharpened, L05) | "missing `is_some(...)`" naming the arm | **repair changes the data model** (R in §2), not a local fix |

M7(a) asks for a non-empty payload naming a tree location where a local fix
exists, for 100% of rejections. Five of the seven classes above fail it, and one
emits no payload at all.

---

## 4. Verdicts overturned

1. **§5.1 M1 = "satisfied, with a correction"** → **violated**: II-14's
   `old()` binding and its step-4 substitution are not functions of the source
   (F1).
2. **§5.1 M10 = "VIOLATED in the edit-stability clause; at the ceiling
   elsewhere"** → **violated in the degree clause as well**: six families with
   no degree (F2), four degrees that omit the closure they call (F3), a
   measured exponent of 3.4 against a cubic ceiling (S1), and measures `F`/`C`
   that are not functions of program size (S2).
3. **§5 "the twelve above are the complete set these rules invoke"** →
   **incomplete**: ≥ 18 (F2, S9).
4. **§5.1 M8 count = 17** → **≥ 24** (S10).
5. **§5.1's silence on M7** → **M7 violated**, five unrepairable rejection
   classes (S13, §3).
6. **I-20's example line 562, `use Links(other); let ok2 = …` = accept** →
   **reject**: INV cannot discharge `σ(other) ∈ Live(P)`; DV-6 therefore has no
   named repair (S6).
7. **II-5's "there is no condition-atom growth to bound"** → **true of the
   state, false of the cost**: Π's atoms enter the cubic closure at every
   access inside a branch nest (S3).
8. **§1.4's writer-cost model (statement 0 / join 0 / loop head 0–1)** →
   **eight restructurings, `O(H)` `use` steps per removal, one non-local
   cascade** (S11, S12, §2).


# File: critique-value-semantics-cross.md

# Cross-examination of `value-semantics` against `window-focus` and `brand-context-bounded`

Key: `value-semantics`. Lens: **CROSS** (cross-examination). Research date: 2026-09-16.

Read: `rules-value-semantics.md`, `rules-window-focus.md`,
`rules-brand-context-bounded.md`, and all six `derive-*` files. Requirements from
VERDICT-D0 §2/§7; hazard ladder from MECHANISM-MAP §3; programs from PROGRAMS.md;
cases from CASES.md. Nothing here selects a candidate. Abbreviations: **VS** =
`value-semantics`, **WF** = `window-focus`, **BCB** = `brand-context-bounded`.

Severity, as the assignment fixes it: **fatal** = a hazard admitted or a
requirement violated undeclared; **serious** = a wrong verdict, an understated
cost, an unnamed family, a missing rule; **minor** = wording.

---

## 1. Findings table

| # | Sev | Item | Where (VS) | Finding | Rival rule that decides it | One-sentence fix |
|---|---|---|---|---|---|---|
| **F1** | **fatal** | Relocation while a `with` projection is open (P1; ladder Step 3; R8a) | `rules-value-semantics.md` I-8 (L260–275), I-9 (L281), I-10 (L306), II-16 (L999), II-17 (L1025); `derive-value-semantics-late-branches.md` L514, L529, verdict L556 | I-8 grounds "R8a's interior pointer into relocatable storage has no instance" on "no projection outlives its statement", which is true of II-19 (call arguments) and **false of II-16** (`with … as`, a block span). Inside `with inout v.data[i] as pi { … }`, `reserve(inout v.meta, 128)` and `push(inout v.data[len(v)], inout v.meta, x)` are **permitted** by II-17, because `v.meta` is a sibling plane of `v.data` and PATH refutes overlap (G3) — the plane map, the thing that makes the frame rule precise, is exactly what lets the reallocating call past the open projection. I-4 and I-16 carry an explicit "no projection is open" premise; **I-9, I-10, I-11, I-13 and I-21 carry none**. Both readings fail: if `pi` is a held address, this is use-after-relocation (ladder Step 3 verbatim, `read(pe)` after `push`); if `pi` is re-resolved per access, then the G4 `noalias.scope.decl` emitted at the block head is asserted across a call that frees the buffer, and P1's required ordering property ("`push` allowed only after both parts are back") is not enforced by any rule. R8a(a) requires every reallocating operation to be a contract-visible ending event, and R8a's price line is "the compiler relocates no storage while an interior pointer names it". **§5.1 carries no R8a row and §6 carries no such defect: undeclared.** | WF **I17** (L369): `push` is a two-armed row on `cap_of > len_of`; arm B sets `St(β) := Gone` and `read(pe)` **rejects** with "prove `cap_of(v) > len_of(v)` before the push". WF **II19** (L879) adds `Span` + `ESC(v)` so a window-scoped name cannot be live across the close. BCB **I-16** (L423) / **I-17** (L451): `ensures backing('v) = ite(n0<c0,'b,'b2)`, `st('b) = ite(n0<c0, Live, Gone)`, and `read(pe)` rejects naming `n0 < c0`; **III-3** (L989) puts `cap` in the vocabulary so `reserve` then `push` is provable | Add "no projection whose resolved path is rooted in the backing this operation may relocate is open" as a premise of I-9/I-10/I-11/I-13/I-21, **or** give `push`/`reserve` the two-armed `cap` contract both rivals write and let II-17 reject the held projection by the missing fact. |
| **F2** | **fatal** | P3 (graph in a pool) is underivable while the model records it accepted | `derive-value-semantics-late-branches.md` L612, L636, L660, verdict L689; against VERDICT-CORE §2 "P3 accepted as required" and `rules-value-semantics.md` §6 (L1632), which lists no P3 defect | Three load-bearing rules are missing: (1) a doubly linked list's `remove` must write `p.slots[σ(prev)].next` and `p.slots[σ(next)].prev`, which are not in I-20's two-part footprint, and II-15(a) forbids writing outside it — widening to `inout p.slots` makes FRAME kill every element fact of the pool at every removal, and III-3 cannot resolve `slots[σ(slots[σ(h)].prev)]` because its index term is a load from the plane being written; (2) I-20's own example writes `use Links(ha)` and calls it "INV re-derives Live(other)", but `Links` quantifies over `σ ∈ Live(P)` and instantiating it at `ha` needs the premise `σ(ha) ∈ Live(P)` — the fact being re-derived — and family 10 (INV) has no witness rule; (3) `Links` as written admits no list terminator, which II-7's own traversal example needs. **R6(e)'s named acceptance test is P3** ("the invariant stated once and reused across four link writes and a removal"); with (2) and (3) missing it is not delivered, and R6 has no declared-violation row | WF derives P3 **accepted with a stated cost** (`derive-window-focus-late-branches.md` L708), at the price of OD1 (see §4). BCB derives P3 **accepted with a stated cost** (`derive-brand-context-bounded-late-branches.md` L691) using `JF-QI` residual framing (III-12) so the invariant is *weakened* by each symbolic write rather than killed, and the one `use` discharges the four exceptions | Give I-20 a `writes p.slots` footprint with residual framing in place of a kill (BCB III-12's shape), state a witness rule for INV instantiation, and give `Links` an `Option<Handle>` terminator arm under III-11. |
| **S1** | serious | Missing rule: use-direction path equality (MR-1) | `derive-value-semantics-straight-and-early-branches.md` §3 L606; bites S01, S02, S04, B03, B04 | No rule says when a fact established at resolved path `g.slots[p.idx]` discharges a goal at `g.slots[q.idx]` given `p == q`. II-3 keys facts by **syntactic** equality of `(fact, support)`; RESOLVE (family 2) walks steps and checks stability but does not substitute equalities; II-12 and EXT only **refute** overlap. Kills are therefore by may-overlap and uses by syntactic identity: *a fact can be destroyed by an alias it can never be read through.* R6(a) requires that the writer can compute which facts survive; a fact that survives unreadably is not a survivor | WF **`COVER`** (§5 family 1, L1248ff) returns "the key governing a place, **or the meet of the keys it meets**" — the use direction is stated. BCB **`KEY`** (L164) defines `mayEq(π)` and fixes the read as `⊓` over the may-equal class, with strong/weak update rules | State the use direction: a goal at π is discharged from the `⊓` over `mayEq(π)`, with `mayEq` computed by the same EXT queries the kill direction already runs. |
| **S2** | serious | Missing rule: pool `insert` and `replace` relate no contents (MR-2, MR-3) | I-19 (L509), I-7 (L237); `derive-value-semantics-straight-and-early-branches.md` §3 L606 | I-19's only `ensures` is `Live(result)`, so no Γ fact ever relates the inserted value to `p.slots[σ(result)].value`; I-7 (`replace`) states no `ensures` and II-14/III-13 bind `old(m)` only for **measure** symbols from the plane map (III-1), so a slot's contents has no `old()`. Consequence: **every value in S01–S06 and B01–B04 is absent from Γ in the pool rendering** — the rendering Appendix B calls "the rendering that preserves the cases' shape" | Both rivals carry contents: WF `Facts` holds contents keyed by identity (§1.2 L57) and I4/I5/I6 move them; BCB `val(π)` is a fact head and `I-7`/`I-9` state the transfer | Add `ensures p.slots[σ(result)].value == x` to I-19 and an `old`-relating `ensures` to I-7, or declare Appendix B's pool rendering value-free. |
| **S3** | serious | B05/B06: wrong attribution and a named repair that does not repair | Appendix B (L1666) rows B05, B06; §6 DV-6 (L1632); corrected by `derive-value-semantics-straight-and-early-branches.md` L543, L594, §4 | Appendix B attributes both to DV-6 (`remove` kills liveness of every handle) with the repair "a written distinctness premise on the pool interface, the same shape as P1's `i != j`". The line-by-line derivation shows the straight-line form **accepts** — DV-6's EXT escape is available — and that what removes its input is **II-3's syntactic join** discarding `σ(p) != σ(q)`, i.e. O11(a). A pool-interface premise cannot repair it: the missing fact is about two *local variables*. B06's cause is different again — II-12 kills `saved == p` at the rebinding and no rule substitutes `saved == ha` first (the MR-1 family), so B06 rejects even with the branch deleted. M7(a) requires the payload to name a repair that exists | WF **II2** (L518): facts key on the **guarded identity**, not the name, so B05 accepts both reads and both ends and B06 accepts/rejects exactly as the case states (§3.21 L947). BCB **II-5** (L693) correlated leafwise evaluation: B05, B06 "same" (§7 L1259) | Withdraw the pool-interface repair, re-file B05 under O11(a) + II-3 and B06 under the missing re-keying rule, and record both as capability losses with no local repair. |
| **S4** | serious | The `Option`-slot repair family is priced at zero and is not free | Appendix B rows B10, B12, B13, L02, L04, L05; §5.1 M2(ii) row (L1617); `derive-value-semantics-boundary.md` L560–615 | The standing repair for II-2's definiteness rejections and for every DV-4 hole case is "make the resource a value: `var slot: Option<Obj>`, consume via `replace`, `match` after the loop". §5.1 presents this as a **satisfaction** of M2(ii) ("III-11 makes vacancy the writer's discriminant"). It is also a **runtime cost the rivals do not pay**: one discriminant word per resource and one branch per release site, in every program that would otherwise have carried the correlation as an erased checker fact. CASES L05 states the boundary explicitly — the checker must not "require every locator to carry such a flag" — and the repair requires exactly that, in source, so M2(ii) is satisfied only by pushing the branch across the source boundary | WF II3 keeps the difference as a **guarded obligation** and II6(b) forbids the guard-selected release, so nothing is materialized (§5 "M2 — satisfied, and it is rule II6(b) that makes it so"). BCB carries `own('a) = ite(α, discharged, held)` over a **frozen** atom (ATOM, L103), erased at lowering | State, in the case table and in §5.1's M2(ii) row, that the named repair costs one tag word per resource and one branch per release site. |
| **S5** | serious | R15 check-once is claimed and measured false in the same file | II-15 (L974–982) vs §5.1 M10 row (L1617) | II-15: the callee body is "checked ONCE for all call sites and, **under R15, once for all instantiations**". §5.1's M10 row declares edit stability **violated** because the verdict-changing set in the callee direction is "the **transitive instantiation closure** `I`", measured at 82.4 s for a 375-line generic fixture with **678 of 995 closures repeating an earlier input**. A closure that repeats per instantiation is not a check-once. One of the two statements is wrong and no rule says which; the accepted set and the M10 declaration both depend on the answer | WF names the trade and picks a side: **OD6** — the default carve is a function of the **declared** type "with each type parameter contributing one opaque entry", which keeps R15(a) check-once and **loses** P5's eight per-column facts under abstraction; "a per-instance carve would defeat R15(a)'s check-once clause — the two cannot both hold, and this is the trade". BCB declares **M10 edit stability met** with `FN-1` as the reach boundary (§6.1) | Choose: either state which facts are lost under abstraction (WF's OD6 shape) and keep II-15's sentence, or withdraw the sentence and keep the measured closure as the declared degree. |
| **S6** | serious | R6 has three exceptions and no declared-violation row | Appendix A (L1654); §6 DV-8 (L1632); §5.1 (L1617) | Appendix A collects three exceptions to "the footprint is the written access path": the foreign kill (I-22), index-term instability (III-3), and the `origin` union (II-20), of which two are stated for the first time in this file. Exceptions 1 and 3 kill facts whose support the stated footprint does **not** name, which is precisely the quantity R6(e)'s metric fixes at zero over P1–P9. §5.1 lists M rows only and **no R row at all**, so R6's status is nowhere declared | WF's §5 declared-violations table names its unsupplied R rows by number (`R1(a)(iii)` not owned; `R5(ii)/(iii), R13, R14(i)(ii)(v)(vi), R8b` not supplied). BCB's §8 **D10** does the same | Add an R6 row to §5.1 declaring the metric not met at exceptions 1 and 3, with the count over P1–P9. |
| **S7** | serious | II-3's join rule contradicts its own example, and either reading costs something unnamed | II-3 (L690–712) | The rule: `Γ := ⋂ Γ_i`, "compared by **syntactic equality** of (fact, support)… No fact is weakened, widened, or turned into a disjunction". The example then accepts `cap(v) >= 64` after `if c { reserve(…,64) } else { reserve(…,128) }`, where the else-arm's fact is `cap(v) >= 128` — a **weakening**, which syntactic intersection cannot produce. If the rule is normative, the example is wrong and every post-join refinement the derivations use must be re-checked; if the example is normative, an entailment-closed join exists and is **not among §5's twelve families**, immediately after §5 corrects the family count from six to twelve | WF names `JOIN` as family 4 with a degree (`O(K·L)`) and a canonicalization order. BCB names `JF-TERM`'s join as a single table lookup and routes entailment to `JF-ENT` with `O(n³)` per guard context | Pick one: if the join entails, name the family, its inputs and its degree in §5 and re-state M1 against thirteen families. |
| **S8** | serious | The capability delta with both rivals is two-sided and its size is nowhere stated | Appendix B (L1666) vs WF §3.21 (L947) and BCB §7 (L1259) | **VS refuses 8 items both rivals accept**: B05, B06, B10, B11, B12 (including the case's own accepted `if cond { release }; if !cond { release }` variant), L04, L05, and P4's stored-cursor conjunct. **VS accepts 7 items both rivals refuse**: S03, S06, B03's `read(a)`, B04's intermediate reads, B07, B09's read, L02's second take — every one of them a "read of a hole", which is the ladder's **Step 0** rung and its **Step 2** index refinement. Fifteen of twenty-six cases diverge from both rivals, in both directions. §6 names the families (DV-4, DV-6, O11(a)) but states neither the count nor that it is two-sided, and Appendix B's summary line ("9 unchanged, 13 DR, 2 CL, 2 CC") is computed against CASES.md alone | rule numbers on both sides are in §2 and §3 below | State the two-sided count in §6 and route the two halves to their owner decisions: the seven accepts to **D3/O-hole**, the eight refusals to **O11**. |
| **S9** | serious | Two "correction of the case" classifications are not corrections | Appendix B rows B08, B09 | A classification of **CC** asserts the case is wrong. For B08 ("repeating an unchanged condition") VS records "the read is total, so nothing needs recovering; the case's mechanism has no instance" — but the case's mechanism is a *refinement* recovery, and the refinement is not recovered, which is a precision loss, not a correction. For B09 VS records "the case's content survives as a rule (III-10) with no instance in this program" and accepts the read the case rejects. Both rivals reproduce both of the case's verdicts **by rule**, which is direct evidence the cases have instances | WF **II5** (L564): B08 accepts by the same atom **version**; B09 rejects by the recorded `¬` relation, and the `!cond` variant accepts (§3.21). BCB §7: B08 and B09 "as the case states" | Reclassify B08 as a precision loss under O11(a) and B09 as a deliberate refusal under DV-4. |
| **S10** | serious | Appendix B was not republished from the derivations that supersede it | Appendix B (L1666) vs `derive-value-semantics-straight-and-early-branches.md` §4 (L622ff) and the three derivation summary tables | The candidate's own line-by-line work changes or qualifies nine of the twenty-six rows (S01, S02, S04, S05 accept-without-the-property; B01 accepts only in a rendering the case does not write, MR-4; B05 and B06 re-attributed; plus P1 and P3 in the program set). A reader comparing candidates reads Appendix B, which is the stale artifact | — | Republish Appendix B from the derivations, or mark it superseded in place. |
| **m1** | minor | DV-3 is stale in the candidate's own favour | §6 DV-3 (L1632) vs `derive-value-semantics-boundary.md` P7 (L159, L233) | DV-3 says P7's two-alias premise is unwritable, so "R3 has no acceptance test" and "R3 falls with it". The boundary derivation shows the premise **is** writable in the pool rendering and that three of P7's four lines derive exactly. The defect list overstates a loss | — | Narrow DV-3 to the projection rendering and restore R3's status. |
| **m2** | minor | `G` is overloaded | §1.3 (L59) vs §1.2's graft labels used throughout the derivations | `G` is both the ghost-pool-fact component of the checker state ("G is not Γ; FRAME reads Γ only") and the graft numbering `[G1]`–`[G6]` used in the same paragraphs | — | Rename the component (`Gh`) or the graft marks. |
| **m3** | minor | §1.3's "what the checker holds" table does not carry §5's correction | §1.3 (L59) vs §5 (L1584) | §5 corrects the family count from six to twelve and names `RESOLVE`, `PLANE`, `CONFLICT`, `FRAME`, `OLDCHAIN`, `GHOST`; §1.3's one-page table still shows five components and names none of them, so the one page a reader holds is the wrong one | — | Add the six families to §1.3 or cross-reference §5 from it. |

---

## 2. Every item a rival accepts that `value-semantics` refuses or cannot derive

| Item | VS verdict and rule | WF verdict and rule | BCB verdict and rule | Class VS assigns / class earned |
|---|---|---|---|---|
| **B05** `read(q)` and the second `release` after a selected release | **reject both**; II-3 (syntactic join drops `σ(p) != σ(q)`), III-8 | accept both reads and both ends; **II2** guarded identity + **I7** | accept; **II-5** leafwise correlation | VS: CL "DV-6, repair = a pool-interface premise" → **CL under O11(a), no repair** (S3) |
| **B06** `read(saved)` after `p = q; release(p)` | **reject**; II-12 kills `saved == p` at the rebinding, no re-keying rule | accept `read(saved)`, reject `read(q)` — exactly as the case states; **II2** | accept; **II-5** | VS: CL "same as B05" → **CL, different cause** (S3) |
| **B10** the join must not erase the obligation | **reject at the join**; **II-2** definiteness clause | obligation retained, guarded; **II3** | accept; **II-3** | DR, repair = yield the survivor from the `if` |
| **B11** `remaining = ref(b)` inside an arm | **reject as written**; §1.1 + **II-19** (a projection cannot be the value of a binding) | accept; **II2/II3** | accept; **II-13** (a projection is an ordinary sub-identity name that may be stored) | DR — and it is the O7/O12 boundary showing up inside a *case*, not a program |
| **B12** two conditional releases, including the case's own accepted `if cond {…}; if !cond {…}` variant | **reject at the first join**, both variants; **II-2** | reject the second, **accept the `!c` variant**; **II6(a)** feasibility | as the case states; **II-3**/`JF-ENT` | DR — VS is the only one of the three that loses the case's *accepted* variant |
| **B13** conditional scope cleanup | reject; **II-2** | reject, repair named; **II6(b)** | reject; **I-3** | all three refuse — **not** a discriminator |
| **L04** release on a break edge | **reject at the loop-exit join**; **II-9** + II-2 | accept; **II9** | as the case states; **II-9** | DR, repair = `break Some(sink a)` + `match` |
| **L05** a checked Boolean guarding later iterations | **reject at the inner `if stop` join**; II-2 / II-5 / **III-10** (O11(a)) | accept, one written head row; **II8** | §7 claims "same"; **its own derivation says underivable — rule missing** (`derive-brand-context-bounded-boundary.md` L624) | DR — the sharpest O11 discriminator, and only WF actually delivers it |
| **L02**, **L03** sub-claims (`read(q)`, `release(p); release(q)`) | reject the second release; DV-6 | accept; II2/II7 | accept | CL inside an item VS records as accepted |
| **P4** stored cursor over a growing container | **capability loss**; §1.1, **II-19**, I-18 — `&v` is not a type and `Handle<P,T>` names only a pool row (DV-1, DV-2) | accepted, one written capacity row; **II19** `ptr<ρ↓w.k,T>` + **I17** | accepted with a stated cost; **I-19** closed identity row + **III-8** conditional `backing`, `Cursor<'v,'b>` | CL — the only one of the three that loses P4's first conjunct |
| **P7** the two-long-lived-writable-names premise, projection rendering | **unwritable**; **II-17** conflict table refuses two overlapping `inout` in one span | accepted, two writer lines; **II15** | accepted with a stated cost; **II-13**, **D4** records the residue | DR — VS recovers it only in the pool rendering (see m1) |
| **P2** coalescing arena | **refused**; II-23 declares no split-and-rejoin (DV-9) | **I12** merge/restore of promoted siblings exists | `I-22` reset, `III-4` frozen-bound disjointness | DR, undeclared as a rival-relative loss |
| **P3** whole program | **underivable** (F2) | accepted with a stated cost | accepted with a stated cost | **overturned** |

---

## 3. Every item `value-semantics` accepts that a rival refuses

| Item | VS verdict and rule | WF verdict and rule | BCB verdict and rule | Reading |
|---|---|---|---|---|
| **S03** `read(q)` after `take(p)` | **accept, returns `None`**; I-7 (`replace` is total) + **III-11** (vacancy is the writer's discriminant) | reject the read; **I4** + `St` on the place | reject at the stated line; **I-7** + `KEY` | The ladder's Step 0 rung "reject: read of a hole" has **no instance** in VS |
| **S06** `replace` with no old value | **accept**; I-7 | reject at the empty slot; **I5** premise | reject; **I-9** | same family |
| **B03** `read(a)` | **accept**, returning the `Option`; III-11 | reject; **II2** + **I3** | as the case states | same family |
| **B04** the intermediate reads | **accept**; I-7 | reject; **II2**, **I4**, **I6** | as the case states; **II-5** | same family |
| **B07** conditional hole, then read | **accept**; III-11 | reject the read, accept after `write`; **I3** | as the case states | same family |
| **B09** the read after a captured condition | **accept**; III-10 | reject, accept the `!cond` variant; **II5** version + `¬` | as the case states | VS classifies this **CC** (S9) |
| **L02** the second `take` | **accept**, both takes total; I-7, II-8 | reject at the back edge; **II7** consecution | as the case states | same family |
| **P3 / S07** read of a **refilled** pool slot | **reject**, statically; **I-20** kills `gen(P[σ(h)]) = γ(h)`, **III-7** keeps γ ghost, no runtime compare | **admitted with no diagnostic**; **III11** declares a generation absent, **OD1** records the wrong-occupant read | reject; **III-6** ghost index `@g` | **VS refuses a hazard WF admits** — the one place in the ladder where VS is strictly the safest of the three |
| **par for h in p.keys()** distinctness from a written `ensures injective(keys)` | accept; **III-9** + II-22 | accept; **II20** + parametric entry II14 | accept; pool contract III-5 | no delta |

Seven of the eight accepts are one family: **D3's "no states: validity by
construction"** turns every "reject: read of a hole" into "accept, returns
`None`". This is the candidate's largest single divergence from both rivals and
it is declared (DV-4) — but it is declared as a *consequence*, not counted, and
its collateral is R3(e), whose only acceptance test is P7 (see m1).

---

## 4. What this candidate asks the owner to accept that the others do not

1. **A permanent capability boundary — O7(a)/O12(a).** A projection is never
   stored, returned, captured or bound (§1.1's "may be stored? never"; II-19).
   The price is DV-1 (no cursor, iterator, callback or closure holding a
   projection over a non-pooled container) and DV-2 (the named repair is a
   **non-local signature cascade**, so M7(a)'s local-fix payload cannot be
   honoured). **BCB refuses this boundary** (II-13 makes a projection an
   ordinary storable sub-identity; I-19 closes the nominal's identity row) and
   prices its refusal at exactly three items (`Cursor<'v,'b>` and an `O(f²)`
   `#`-clause count at producing interfaces; III-8's conditional `backing`;
   `JF-OVL`'s `O(I·d·n³)` per symbolic access). **WF takes O12(b)** and buys back
   `split_at_mut`, returnable slices and byte-range cursors for **one type form
   plus one syntactic judgment** (`ptr<ρ↓w.k,T>` + `ESC`, priced at +2 M8 cards),
   keeping element-yielding iterators lost (OD7).
2. **A name kind in a type after all.** The pool brand `P` on `Pool<P,T>` and
   `Handle<P,T>` (I-18, graft G1) is the one type-level identity parameter, and
   VERDICT-CORE's Disputed **D-6** already asks whether it reopens D1. **WF
   refuses a generation entirely** (III11) and pays OD1 (the wrong occupant, §3);
   **BCB** carries the same ghost index and the same **O3 dependence** (D5). VS's
   §5.1 states M2(ii) is **violated under O3(b)**, in which case the model
   "reverts to the base with its stale-slot residue" — so the owner is asked to
   rule O3(a) *and* to accept the brand parameter.
3. **A written invariant per structure, not per program.** One plane invariant
   per backing (III-2 `VecOk`), one pool invariant per pool type with 3–8 `use`
   steps (`PBounds`, `Links`), one arena invariant (`IA`), and one liveness
   clause at the head of any loop that frees into a pool (II-7). This is cheaper
   than BCB's per-declaration rows, but it is **not** the "zero written for
   memory safety" the one-page summary implies.
4. **A restructuring per case, whose standard form is an `Option` slot.** B10,
   B11, B12, B13, L04, L05 and P8(a) all repair to the same shape, and the shape
   costs a runtime discriminant word and a branch per release (S4). Neither rival
   materializes anything for these cases.
5. **O11(a): no condition term anywhere**, even bounded. WF bounds them at
   `Gmax = 2` guard leaves (a constant, with OD2 recording that no evidence sets
   the constant); BCB bounds them at **one atom per identity per point** with a
   guard-context restriction that makes accumulation impossible (TERM, L129) and
   a refusal class with two named repairs. VS refuses the mechanism rather than
   bounding it, and pays for it in eight case items (§2).
6. **M10's edit-stability clause, declared violated**, with the callee-direction
   reach measured at the transitive instantiation closure (82.4 s; 678 of 995
   closures repeating). **BCB declares the same clause met.**

## 5. What it spares the owner

1. **One distinctness relation.** The containment tree `root → plane → extent →
   element`, with `PATH`/`EXT` over it and `NoReach` as a type predicate — C2's
   single relation, achieved. WF carries `⊑`, `≡` and a total `DIS` over five
   identity forms; BCB carries `#`, `≤` and `mayEq`.
2. **No state at all.** Σ is a two-point lattice, Γ is a set of `(fact, support)`
   pairs, Π is discarded at every join, and **there is no `ite` anywhere in the
   checker's state**. WF carries `Auto`, `St` as place → `G[state]`, a guard
   environment with versions and relations, and `Gmax`; BCB carries a five-point
   lattice, depth-≤1 terms, `RSt` with residual exception lists, `obl`, `owns`
   and `layout`.
3. **Nothing written at a loop head for memory safety** (II-7), except one pool
   liveness clause. WF writes a head row for L03, L05, any minted name and any
   guard that must cross the back edge (OD11); BCB writes an `invariant`
   wherever a head fact is not a constant.
4. **M4 satisfied.** One spelling per construct, `carve` explicitly not a fourth
   convention. **WF declares M4 violated** (three identity kinds plus guarded
   values plus two pointer type forms).
5. **The smallest taught inventory of the three**: 17 constructs, against WF's
   "≥34 cards, ≥10–12k tokens" and BCB's "≈46 constructs" — all three against an
   unset `K` (owner O6), so the ordering is stable but no verdict is available.
6. **No drop flag is representable**, structurally. WF needs II6(b)'s
   guard-neutrality **as a rule** to reach the same place; BCB needs I-3's
   refusal.
7. **The safest rung on the ladder's pool case**: the refilled-slot read is
   refused statically with no runtime compare (§3, last row), where WF's OD1
   admits it with no diagnostic.

---

## 6. Constraint check (M1, M2)

- **M1.** VS's twelve families are each specification-fixed and terminating, and
  §5 corrects the selection's count of six rather than defending it — the most
  honest family inventory of the three. **But** S7 puts a thirteenth family
  (join entailment) in question immediately after that correction, and F2's
  missing INV witness rule is a place where the only route to the verdict the
  model claims would be a **search** for a witness handle. M1 is not yet
  established for P3.
- **M2(i).** Satisfied in every derivation read: each failure edge reached is a
  written arm.
- **M2(ii).** Satisfied at the checker and **conditional on O3(a)**, as §5.1
  declares. The unstated half is S4: the repair family that keeps it satisfied
  moves a discriminant word and a branch into the source, in cases where both
  rivals materialize nothing. F1 is the one place where M2(ii)'s companion
  obligation — that the *emitted* facts be true — is not established: the G4
  alias scope declared at a `with` head is asserted across a call that may free
  the backing, and no rule forbids it.

---

## 7. Verdicts overturned

| Item | Recorded verdict | Overturned to | Ground |
|---|---|---|---|
| **P3** | "accepted as required" (VERDICT-CORE §2; no entry in §6) | **underivable — three missing rules**; R6(e)'s named acceptance test not delivered, undeclared | F2 |
| **P1** | "accepted as required" | **accepted minus one required property** ("`push` only after both parts are back") **and with an R8a hazard admitted** | F1 |
| **B05** | CL under DV-6, repair = a written pool-interface premise | **CL under O11(a) + II-3, no local repair** | S3 |
| **B06** | CL, "same as B05" | **CL under the missing re-keying rule at assignment**; rejects with the branch deleted | S3 |
| **S01, S02, S04, S05** | "—" (unchanged) | **accept, but the property each case exhibits is not derivable** (MR-1, MR-2, MR-3) | S1, S2 |
| **B01** | accepted, "—" | **accepted only in a rendering the case does not write**; the natural `let p = if …` is uncovered (MR-4) | S10 |
| **B08** | CC, correction of the case | **precision loss under O11(a)**; both rivals derive the case's ACCEPT with the refinement | S9 |
| **B09** | CC, correction of the case | **DR under DV-4**; both rivals reproduce the case's REJECT by rule | S9 |
| **DV-3 / R3** | "R3 falls with P7" | **overstated**: P7's premise is writable in the pool rendering and three of four lines derive | m1 |
| **Appendix B counts** | 9 unchanged / 13 DR / 2 CL / 2 CC | **superseded**: nine rows change or acquire a qualification in the candidate's own derivations | S10 |


# File: critique-window-focus-soundness.md

# Critique: `window-focus`, lens **SOUNDNESS**

Candidate: `window-focus` ([rules-window-focus.md](rules-window-focus.md)).
Derivations under review: [boundary](derive-window-focus-boundary.md) (P4, P7, P8,
L01–L06), [straight-and-early-branches](derive-window-focus-straight-and-early-branches.md)
(B04, B05), [late-branches](derive-window-focus-late-branches.md) (B08).
Items re-derived independently from the rules before the derivations were read:
**P4, P7, P8, B04, B05, B08, L03, L05**. Round: core2. Research date: 2026-09-16.

Method: each item was derived line by line from rule sets I, II, III and §5's
families alone, in MECHANISM-MAP §3 style, then compared against the candidate's
derivation files. This file reports (1) lines whose verdict does not follow from
the rules, (2) hazards from R1, R2, R3, R11, R12 the rules admit, (3) M2(ii)
clause violations carried as a library or a typed outcome, (4) rules that need a
fact whose source is never stated. Nothing here selects or ranks a candidate.

Severity: **fatal** = a hazard admitted or a requirement violated undeclared;
**serious** = a wrong verdict, an understated cost, an unnamed family, a missing
rule; **minor** = wording.

---

## 1. Findings

| # | Sev | Item / line | Rule | Finding | Fix |
|---|---|---|---|---|---|
| **F1** | fatal | P4, `a = c.read()` — boundary §"The first read": *"accept: St(β[c.i]) = Init by one INV step against the Prefix invariant (III5) given c.i < len_of(ρv)"* | III1, III5, `INV` (§5 #14) | A **declared invariant may conclude `St`**, and `INV` verifies a written instantiation without asking where the antecedent came from. So a measure fact establishes a *state*: any row with `ensures len_of(ρ)=m` plus III5's `Prefix ⇒ St(backing[0..len_of])=Init` hands the checker `Init` for `m` elements nothing ever wrote. That is R1(a)(i) — a read of storage holding no value — admitted with no diagnostic, and it is the `set_len` hazard by construction. The same route reaches every bridge in §4: `head_of`, `off_of`, `Occupied` | State that `St` is established **only** by a rule-set-I transition; an invariant may *constrain* `St` but a row that establishes its antecedent carries the corresponding I-rule state edge as an obligation |
| **F2** | fatal | L03, the loop body — boundary §L03: *"v = take(p) // accept: I4 on the place target(p)=i1, St(i1)=Init from the head row"* | II7, `LOOPCHK` (§5 #16) | **II7 states no frame at the loop head.** It says only that the checker verifies initiation and consecution, and that "the default head row is the identity on every `St` key and every fact". Under that default-plus-written reading the entry facts `St(ρa)=Init`, `St(ρb)=Uninit` are still live inside L03's body, so an in-body `read(a)` on iteration 2 is admitted from a stale fact while `ρa` is `Uninit` — R1(a)(i). Under that same reading the head is also *contradictory* (identity on `ρa` says `Init`, the existential says `St(i2)=Uninit` with `i2 = ρa` in head state 2), so consecution fails and **L03 rejects**. Under the only reading that accepts L03, the written row *replaces* `St`/`Facts`, i.e. every unnamed key is havoc'd — which is a large unstated writer cost and contradicts L01's "no head row needed". The derivation picks the accepting reading without saying so | State the head-row frame: the written row replaces `St`/`Facts` for every key it names and is the identity elsewhere, plus an obligation that no identity named existentially is reachable through an unnamed key |
| **F3** | fatal | P7, `v2 = take(p); drop(v2)` — boundary §P7: *"accept: I4 then R3 — I7 requires 'every obligation held by a value inside ρa is discharged' … M2(ii) forbids the alternative"* | I6 example, I7 premises, §1.2 | The rules give **two contradictory answers** and no structure for either. I7's premise demands the writer empty the storage; I6's own example says the opposite — *"end(dst_owner) // accept: I7 — the obligation now under π_dst is discharged by I7's closure"*, i.e. a compiler-inserted recursive release, which is exactly the release "the source did not write" M2(ii) refuses (and which the §5 M2 row claims is absent). §1.2 has **no structure keying an obligation to a place**: `Own` maps *bindings* to identities. Worse, II11's `KILL` destroys the contents facts I7's premise quantifies over, so after any `@W` call the premise is undecidable. The outcomes are a leak (R2(a), released never) or an inserted release (M2(ii)) — undeclared either way; the derivation resolves it silently and charges P7 two writer lines | Add an obligation ledger keyed by **place** to §1.2, rule on which of I6's and I7's readings holds, and if it is I6's, declare the M2(ii) violation |
| **F4** | fatal | I9's effect and example: *"mint fresh ρ'' with St(ρ'') := Init"*, *"read(ptr_of(t.x)) // accept"* | I9, `COVER` (§5 #1) | I9 sets the **whole new name** to `Init` instead of re-keying the source's per-place state tree. I2's own example builds exactly the counterexample (`St(ρs.x)=Init`, `St(ρs.y)=Uninit` via the `COVER` residue); after `t = move s` the residue under `ρt` resolves to `Init` and `read(t.y)` is admitted. R1(a)(i), on a partially initialized struct move — the commonest shape in the corpus | I9 re-keys the whole `St` subtree of `ρ` pointwise onto `ρ''`; `Init` is not a state I9 may invent |
| **F5** | fatal | I12's effect: *"the children's names end (not Gone — absorbed)"* | I12, I7, II19 | Absorption leaves a **live first-class `ptr<ρc1,T>` pointing at a name with no state**. `Carve` and `Span` do not see first-class pointers (II19 is explicit: only span-typed bindings enter `Span`), so I12's premises cannot notice one, and `ρc1` is neither `Gone` nor re-keyed. I7 later sets `Gone` for "`ρ` and every `St`-key under it" — the absorbed child was re-keyed under the parent's residue, so its own key can survive `Init`. A read through the retained pointer after `merge(c1,c2); free(buf)` is then admitted: use-after-free, R1(a)(ii) | Give absorption a terminal state that rejects every access through the absorbed name (not `Gone`, so I23's `dis` argument still holds), or make live pointers at a child name an I12 premise |
| **F6** | fatal | §1.4 erasure table, row *"`@E` on an identity ⇒ parameter `noalias`"* | §1.4, II10, II15 | The side condition excludes only R14(iv)'s foreign class. Nothing forbids one row from naming `ρ1 @E` and `ρS @W` where `ρ1 ⊑ ρS` (I11's arena block and its arena — `dis(ρ1,ρS)` is *false* by III7's own line). II15's "within one entry" test governs **carves**, not `⊑`-children, so the call is accepted and the emitted `noalias` is false in every execution: R4(a) soundness, a miscompile, not a lost fact. The S-F4 repair in the same table covers only `split` **results** | Extend the side condition: `@E ⇒ noalias` only when no other identity in the same row has a live `⊑` relation to it |
| **S1** | serious | P4, boundary §"The push, with the capacity proof written": *"reserve(v, len_of(v)+1) // accept"* then *"b = c.read() // accept: St(β)=Init unguarded"*; same chain in the rules at III3 and I16 | II12 example, I17, III3, III4 | **The named repair does not work, and this verdict is wrong.** II12's own worked row for `reserve` is two-armed — `backing_of(ρ) : Init ⇒ Init when cap_of(ρ) >= m | Init ⇒ Gone, split β' otherwise`. At `reserve(v, len_of(v)+1)` neither arm is discharged, so II12 mints `g#` and the post-state is `St(β)=ite(g#, Init, Gone)`, `backing_of(ρv)=ite(g#, β, β')`. The `ensures cap_of ≥ m` holds on both arms and therefore selects push's arm A, but **the cursor already names a guarded backing**, so `b = c.read()` rejects on the ¬`g#` arm exactly as it did before the repair. `reserve` only moves the split from `push` to `reserve`. P4's second conjunct is still reachable — from `vec_new`'s own `ensures cap_of > len_of`, or one `INV` step against a written container invariant — but not by the spelling the rules and the derivation give | Replace the `reserve` chain in III3, I16 and the P4 derivation with a capacity fact from formation or from a written invariant, and state that `reserve` re-forms the cursor |
| **S2** | serious | P4, `c = Cursor{ at: …, i: 0 }` then `St(β[c.i])` | §1.1 `index term`, III8 | `c.i` is a **struct field load**, and §1.1 admits only `t ::= a literal | a captured index binding (III8) | an affine term over a loop binder`. III8's "captured index binding" is explicitly "a source variable, with a version". So the place `β[c.i]` cannot be formed at all, and III8's uncaptured-index clause ("kills the fact for every sibling element") is the rule that actually applies. P4 is *the* stored-index program, so this is the item's own crux | Add a rule capturing a field load into an index binding (`let i = c.i`) with its own version, or admit a resolved field path as an index term and state its kill radius |
| **S3** | serious | P4, `push(v,5)` arm B; III4's *"`backing_of(ρ) = β` binds … a coarse name `β ⊑ ρ`"* | I17 arm B, I11, I10, III4 | I17 arm B mints `split β' ⊑ ρ`, and I11 admits a promotion **only** against a proved extent inside `free_of(ρ)`, a written fact of the parent's type (III6/III7). A container descriptor has no free extent and no arena invariant, so I17 arm B's promotion premise has no source. I10's example states the opposite relation for the same shape — *"ρc ⊑ ρbox is FALSE: they are separate roots"* — so the rules contradict themselves on whether a backing is contained in its container | Make a (re)allocated backing a fresh I1 root with `backing_of(ρ) := β'` as an ordinary bridge re-binding (III4's `kill/rebind`), and delete the `⊑` edge |
| **S4** | serious | B05, `release(p)` and `release(q)` — straight §B05: *"the owner obligation is consumed on the selected leaf — a GUARDED obligation (II3, III12)"*; same in P8(a)'s `free(s)` and B06's `release(p)` | §1.2 `Own`, II3 | No rule provides a **guarded `Own`**. §1.2's `Own` is an unguarded map from an affine owner binding to an identity, and II3 merges "every key of `St` and every fact in `Facts`" — not `Own`, and not the binding→identity map, which §1.2 does not contain at all (the late-branches file registers this as its G2; the straight file asserts it for B05/B06 and registers **no** gap; the boundary file registers only II2's prose scope as G6). B05's two accepts and P8(a)'s `free(s)` are therefore not derivable from the rules as written | Add `Own` and a binding→identity map to §1.2 and to II3's join, with guarded consumption defined and II6(a) quantifying over it (which is what B12's refusal already assumes) |
| **S5** | serious | L05, the head row and `if active { release(p) }` — boundary §L05: *"atom(active@1) with the relation `const true` recorded"*, then consecution checked "leaf by leaf" over a free `active` | II5, II6(a), II7, II8 | The derivation uses one atom two incompatible ways. II5 **records** `active@1 = const true` because the RHS is a literal; II6(a) then makes every assignment with `active@1 = false` **infeasible**, so the head row's `¬active` leaf is vacuous and the stop path's `St(ρa)=Gone` is checked against the pinned `Init` leaf — **consecution fails and L05 rejects**. Accepting L05 needs a *head version* of `active` whose entry relation is an initiation obligation rather than a standing relation; no rule creates one, and II5's version scheme has no merge rule for a binding assigned in a loop body. The post-loop `if active` then has no stated relation to the head row's atom, which is what R2 rests on here | Define a head version for every binding a head row names: the entry relation is discharged at initiation only, the version is free inside the body and at the exit join |
| **S6** | serious | L05, the back edge — boundary §L05: *"atom(stop) retires at the back edge (II8)"* alongside the leaf-by-leaf consecution check | II8, II3 | **The order is unstated.** II8 says body atoms are retired at the back edge; if the back edge joins first (II3) and then retires `stop`, the value carried is `St(ρa)=ite(stop,Gone,Init)` with `active=ite(stop,false,true)` and retiring `stop` destroys the correlation the head row exists to carry — L05 rejects. Only "check consecution on each incoming path, then retire" gives the derivation's verdict. The boundary file registers the atom-free *exit* join as G5 but not this | State in II8 that consecution is verified per back edge before atom retirement |
| **S7** | serious | L03, the head row itself: `head row { exists i1 i2. target(p)=i1, …, dis(i1,i2) }` | II7(d), II10, §1.1, §1.2 | The row L03 needs is **outside the only row grammar the rules define**. II10's grammar admits `π : pre ⇒ post @g`, `split`, `absorb`, `relocates`, `end`, `result`, `requires`/`ensures`, `outcomes` — no existential binder; §1.1's identity grammar `ι ::= ρ | π | e | {ι,…} | ite(c,ι,ι)` has no binder form; `target(p)` needs a binding→identity structure §1.2 does not list; and `dis` is a **judgment**, not one of `Facts`' three kinds (measures, contents, declared invariants), so it cannot be a row clause. II7's example writes all four anyway | Either extend §1.1/§1.2/II10 with an existential-binder identity, a binding→identity structure and a disjointness fact form, or classify L03 as a capability loss |
| **S8** | serious | L03 summary row — boundary §Summary: *"accepted with a stated cost; one line underivable (G3)"* | round rule (classification) | `release(p); release(q)` is an **ACCEPT in CASES.md** and is underivable here. The round requires every difference from a CASES verdict to be classified as *capability loss*, *deliberate refusal with a named repair*, or *correction of the case*. "Underivable" is none of the three, and the rules' §3.21 L03 row still reads "yes, with a stated cost", which is now false | Classify it — on the current grammar it is a capability loss — and correct §3.21's L03 row |
| **S9** | serious | L03 initiation `dis(ρa,ρb)`; also B05, B06, S01, P8(a) | §5 `DIS`, I1, I11, III6 | All three derivation parts register that **`DIS` has no clause for two unrelated `⊑`-roots** (boundary G4, straight G2, late G8) and then resolve it as *true* on the ground that the names are distinct. R4(b) bans that ground by name ("never inequality of identity names"), and I11 and III6 restate the ban. The reading is almost certainly the intended one, but as taken it is the forbidden inference, and it is load-bearing for every alias-survival step in the set | Add a `DIS` base clause grounded on **formation** — two identities minted by distinct I1/I2 events with no `⊑` relation are disjoint — which is a formation ground, not a name ground |
| **S10** | serious | I13, arena carve: *"or the operation's outcome arm is written (R12; owner interim O5(c): arena exhaustion is an ordinary library outcome)"* | I13, R11(a), R12(a) | **O5(c) is not the settled interim.** VERDICT-D0 §7 decision 5's interim is (b) — abort with a resource record, outside the outcome model, with R12's clause marked red; VERDICT-CORE §7's O5 row says in terms that (c) is the position *the selection assumes* and that under (b) "P2's and P18's derivations rest on a refused spelling". Under the settled interim an unproved `off_of(A)+n ≤ cap_of(A)` is an R11 rejection, not a library outcome. This is also the one place an M2(ii)-shaped discharge is carried as a typed outcome: the domain obligation is satisfied by the presence of a runtime comparison inside a library | Derive I13 under interim (b) — reject without the bound — and mark the outcome arm as an assumption on O5, not as settled |
| **S11** | serious | every `read(…)` line in all three parts; the vocabulary tables map it to "premise only" | I, II, III (absent) | **No numbered rule states the premises of a read** — the operation R1(a) exists for. The three parts each map `read(p)` to "premise `St = Init`", citing I1's example. R11's index domain (asserted on I16's example line and on P4's first read) and R1(a)(iii)'s layout have no rule to carry them either | Add an access rule to set I: premises `St(π)=Init` on every feasible assignment, the domain obligation for every index term (`MEAS`/`AFF`), II15, and the layout axis when OD5 closes |
| **S12** | serious | III1's own example: *"y = v[n-1] // reject: missing fact — len_of(ρ) was killed at the push"* | III1, I16, I17, II10 | The example contradicts the rules it illustrates. II10 applies a row as `KILL` **then** `ensures`, and I16/I17 arm A `ensures len_of(ρ)=old(len_of(ρ))+1`; with `n = old(len_of)` bound, `len_of = n+1` is restored and `n-1 < n+1`, so the line **accepts** under III5's prefix invariant. As written the example teaches a rebind tax the rules do not charge | Correct the example to a footprint that has no `ensures`, or state the KILL/`ensures` order and the resulting verdict |
| **M1** | minor | B08, `if cond { put(p, 20) }` — late §2 | I6 | I6's premise names "the **source binding** [that] holds the value's obligation"; `20` is a literal with no binding. No rule admits a literal operand to a sink | Admit a literal or a copy-class value as an I6 source with no obligation transfer |
| **M2** | minor | P4 setup, *"St(β[0..n]) = Init under the Prefix run (III5)"* | III5 | III5's `Prefix` is a **declared type invariant**; the derivation never writes the vector nominal that declares it, so the fact's source is the rule's own example | Write the container's `states`/`invariant` block in the derivation, as III5's own example does for `Ring` |
| **M3** | minor | P7, *"read(q) // accept: reads `new`"* | I5, G4 | The straight and late parts register that no `ensures` carries a value out of storage (their G4/G7) and mark the cases' printed values underivable; the boundary part asserts P7's value claim with no marker | Add the same marker, or add `ensures result = old(contents(π))` to I5 |
| **M4** | minor | §5 declared violations, "not supplied" row | R12(a) | The row lists R5(ii)/(iii), R13, R14(i)(ii)(v)(vi) and R8b, but not R12's context-prohibition clause ("a context may forbid a failure outcome … and the checker enforces it"), for which no rule exists | Add R12's context clause to the not-supplied row |
| **M5** | minor | the three gap registers | — | `G1`–`G6` (boundary), `G1`–`G4` (straight) and `G1`–`G8` (late) reuse the same identifiers for different gaps — the boundary's G3 is L03's existential release, the straight's G3 is locator release, the late's G3 is a third thing — so a cross-part citation is ambiguous | Number gaps once per candidate, not once per part |

---

## 2. M2 clause (ii) violations carried as a library or a typed outcome

| Where | Carried as | Judgment |
|---|---|---|
| I6's example, *"the obligation now under π_dst is discharged by I7's closure"* | a **library** `end`/destructor | **Violation, undeclared** (F3). A release the source did not write, selected by what the storage happens to contain. The §5 M2 row claims nothing of the kind exists |
| I13, arena exhaustion as "an ordinary library outcome" | a **typed outcome** on a library operation | **Violation of the settled interim** (S10). R11(a) admits an outcome only where R12 classifies the operation fallible, and decision 5's interim places exhaustion outside the outcome model |
| III1/III5 + `INV`, a state concluded from a measure | a **written invariant** | **Violation, undeclared** (F1). The safety predicate ("these elements hold values") is decided by a runtime value through an implication nothing obliges the writer to ground |
| III10's `match slot_state(P,k)`, I19 | the writer's own data | **Not a violation.** The match is source-written, the bit is bound in source, and the operations' `ensures` re-establish the invariant against a body checked once (II10). It becomes F1's hazard only through the `St`-concluding invariant, which is where it should be fixed |
| I17/II12's `g#` | a checker atom | **Not a violation.** No operation is selected by it (II6(b)); the cost is a rejection, which is what M2(ii) asks for |
| L05's `if active { release(p) }` | a source boolean | **Not a violation.** Every branch is written; II6(b)'s formal criterion holds |

---

## 3. Facts used with no stated source

Collected from §1 so the owner can see the shape: each is a rule reading a
structure or a fact the rule set never introduces.

1. `St` concluded from a declared invariant over measures (F1) — III1/III5/`INV`.
2. The loop-head frame: which facts survive into a body (F2) — II7.
3. Obligations held by a **value inside storage** (F3) — I6/I7, no structure in §1.2.
4. A guarded `Own`, and any binding→identity map at all (S4) — §1.2/II3.
5. `free_of` of a container, needed by I17 arm B's promotion (S3) — I11/III4.
6. A loop-head version of a boolean binding (S5) — II5/II7/II8.
7. Existential binders, `target(·)` and `dis` as row clauses (S7) — II10/§1.1.
8. `DIS` for two unrelated roots (S9) — §5 #5.
9. The premises of a read (S11) — absent from I, II and III.
10. An index term that is a field load (S2) — §1.1/III8.

---

## 4. Verdicts overturned

| Item | Derivation's verdict | This critique | Ground |
|---|---|---|---|
| **P4** (second read after the repair) | accept, "carried by proof, not by a borrow" | **reject** as spelled; accept only from a formation or invariant capacity fact | S1 — `reserve` is itself two-armed (II12), so the cursor's backing is already `ite(g#, β, β')` |
| **P4** (first read) | accept | **not derivable**, and the route used admits a hazard | S2 (`c.i` is not an index term), F1 (`St` from `len_of`) |
| **P7** | "accepted as expected, with two writer lines" | **undetermined**: two writer lines, or an inserted recursive release | F3 — I6's example and I7's premise contradict, and §1.2 has no place-keyed obligation |
| **B05** | "accepted as expected" | **not derivable** | S4 — both `release` lines need a guarded `Own` no rule provides |
| **P8(a)** (`free(s)` on a bound survivor) | accept | **not derivable** (the branching spelling still accepts) | S4, and the boundary file's own G6 |
| **L03** | "accepted with a stated cost" | **reject** under II7's identity-default reading; accept only under an unstated havoc reading | F2 |
| **L03** (`release(p); release(q)`) | "underivable: rule missing" | **capability loss**, and §3.21's "yes, with a stated cost" row is false | S8, S7 |
| **L05** | "accepted with a stated cost" | **reject** under the literal reading of II5 + II6(a), and under join-then-retire at the back edge | S5, S6 |

Unchanged: **P8(b)** (accept), **B04** (accept; its `ite(c,x,x) → x` collapse is
correct and load-bearing), **B08** (accept; the safety verdict survives G1/G7 and
M1). **L01, L02, L04, L06** were not in this lens's set and are not ruled on,
except that L04 and L06 inherit F2's missing head-row frame.

---

## 5. What this leaves for O7, O11, O12

Stated without selecting: the soundness holes above are not evenly spread. F1,
F3, F4, F5 and F6 are in rule set I and §1.4 — creation, destruction and erasure —
and none of them is caused by the projection boundary. F2, S5, S6 and S7 are at
loop heads. Only S2 and S3 touch the stored-pointer boundary O7 and O12 ask
about, and both are repairable inside the candidate. The O11 evidence the rules
claim (§3.21: twelve of twenty-six cases decided by guarded values) survives F1–F6
for B04 and B08 but **not** for B05, P8(a) and L05, whose guarded verdicts all
rest on a guarded `Own` (S4) or a head-row version (S5) that no rule supplies; on
this lens the count of cases actually derivable under condition terms at joins is
lower than the rules claim.


# File: critique-window-focus-cost.md

# Critique: `window-focus` — lens **COST AND DETERMINISM**

Target: [rules-window-focus.md](rules-window-focus.md) and the three derivations
([straight-and-early-branches](derive-window-focus-straight-and-early-branches.md),
[late-branches](derive-window-focus-late-branches.md),
[boundary](derive-window-focus-boundary.md)). Round: core2. Research date: 2026-09-16.

Scope of this lens: every judgment family checked for a fixed terminating procedure with
no search (M1) and a stated degree (M10); the condition-atom bound; the construct count
(M8); the writer's per-item written burden; and M7 repairability of every rejection the
derivations produce. No candidate is selected here and nothing is compared to another
candidate. Line numbers are of `rules-window-focus.md` unless another file is named.

A declared violation is not a finding. `M4`, `M8`, `M10(i)`/`(ii)`, `R1(a)(iii)` and the
un-supplied R5/R13/R14 vocabulary are declared at lines 1290–1299 and are not charged
below except where the declaration is *narrower than the defect it covers*.

---

## 1. Findings

| # | Sev | Item | Line / rule | Finding | Fix |
|---|---|---|---|---|---|
| **F1** | fatal | B10, B12, B13, P8(a), L05; every `KILL` | 581–588 (II6(a)); 1260 (§5 family 3 `GUARD`) | II6(a) quantifies every obligation, `DIS`, `KILL` and `R2` over "every feasible assignment **of the live atoms**". `Gmax` bounds a *value's* depth, not `|Γg|`: II5 mints a version per boolean assignment and keeps it live "while any value mentions it", and nothing retires atoms in straight-line code, so `|Γg|` grows with the number of `if`s in a declaration and the enumeration is 2^`|Γg|`. `GUARD`'s degree prices **one cube test** (`O(Gmax·α) per query`), not the enumeration, which is named by no family. M10(a)'s "never exponential" and M1(b)'s stated-bound clause are violated and the violation is **undeclared** (1290–1299 charge only `AFF` and edit reach). | Restrict the quantification to the atoms occurring in the values the obligation consults (≤ `k·Gmax` for `k` consulted values), add a quantity symbol for that set, and state the resulting degree in §5. |
| **F2** | fatal | P1 (§8.2–8.3 of late-branches), P3 (§9.5), P5-shaped kernels | 796–815 (II15); §5 table 1256–1274 | The per-access "lies provably within exactly one entry, or provably covers a union of entries" test is **not one of §5's seventeen families**, has no stated degree, and is the most frequently invoked judgment in the model: II15's own example asserts "the signature's row IS a window (II18)", so every access in every declaration is inside a window. Deciding the complement entry `\S` requires disjointness against every listed entry, i.e. `O(E·AFF)` per access and `O(S·E·AFF)` per declaration; no such number appears anywhere. Undeclared M1(b)/M10(a). | Name the family (e.g. `WITHIN`), state its degree with the `AFF` factor, and add the composed per-declaration total to the declared-violations table. |
| **F3** | fatal | every call and every in-window access; OD6's P5 claim | 767–795 (II14), 851–865 (II18), 808–811 (II15 prose), 1296 / 1300–1303 / 1319 | The **default (total) carve has no defining rule.** II14 opens only a *written* `focus` ("the carve's entries are written at the head"); II18 asserts a signature row is a window without saying what its entries are; yet II15's prose ("a total carve is not vacuous"), M10(ii)'s edit-reach declaration, R15's note and OD6 all reason about "the default carve … a function of the declared type". A judgment with no rule has no procedure and no bound, so acceptance of an in-window access is not a total function of the source bytes and the specification (M1(a)). | Add a numbered rule deriving a row's entry set from the declared type (one entry per field, one opaque entry per type parameter), with its family and degree. |
| **F4** | fatal | B09 variant, B12 variant, B13 repair | 501–508 (II1) vs 578 (II5 example), 595 (II6 example), 322 (I8 example) | **The rule set's own printed repairs are refused by its own rules.** II6's diagnostic ("prove ¬(c ∧ d), e.g. bind d = !c"), I8's ("release a on the ¬cond path") and III12's are all spelled `if !c { … }`; II1's premise admits only a boolean **binding** and states that an expression discriminant "captures nothing", so the repaired program re-rejects at the identical location with the identical missing fact. M7(a)'s clause "a repair applied as named does not reintroduce the same rejection at the same location" is violated, and M7 is **not** in the declared-violations table. The derivations reach the CASES verdicts only by following the worked examples against the rule (late-branches G4). | Either extend II1's premise to `!c` over a captured binding (II5 already records the `¬` relation for a *definition* from another binding), or rewrite every printed repair as `let nc = !c; if nc { … }`. |
| **F5** | fatal | P1 | 796–815 (II15) with 564–570 (II5), late-branches §8.3 (G6) | P1's required property is a diagnostic at `v[k]` that names the missing fact **and** is repairable. The writer's natural repair from that diagnostic — bind the named fact and test it, `let ok = (k != i && k != j); if ok { … }` — reintroduces the identical rejection at the identical location, because II5 records a relation only for an RHS of `c`, `!c` or a literal and leaves every other atom free, so no rule connects a guard atom to the predicate its defining expression computes. Exactly one route (a written `use` step) admits the read, and no rule says so. Undeclared M7 violation. | State in II15 that the sole admitting route is a written `use`/`INV` step and that a source-level test establishes nothing, or add the atom-to-predicate rule; either way add M7 to the declared violations. |
| **S1** | serious | all | 1258, 1259, 1265, 1267 (§5) | **Four of seventeen degrees are understated and no composed per-declaration bound exists.** `STATE` `O(S·L)` omits the `KILL`, `COVER`, `MEAS` and `INST` it invokes per statement (I3's effect alone calls `KILL`, which is `O(F·DIS)`). `COVER` `O(D)` omits "the meet of the keys it meets", which P3 §9.4 takes over *every* slot key, i.e. `O(K)`. `PART` `O(E²)` omits the `AFF` factor that `EXT`'s own row (1263) includes, although every pair obligation is an `AFF` call. `INST` `O(A·D)` omits II10's own "the whole row must lie within one entry" test. M10(a)'s second clause (total work per declaration at most quadratic in written proof steps) is neither computed nor declared violated. | State each degree with its callees' factors and add one composed per-declaration bound to the declared-violations table. |
| **S2** | serious | every `PART`, `EXT`, `MEAS`, II15 obligation | 1264 (§5 family 7) | `AFF` has no degree, only a measurement ("inherited; measured 2.5–3.4 exponent"). M1(b) requires each family as a decision procedure **with a stated bound**; a measured exponent on one bundle is evidence, not a specified bound, and cannot be checked against M10's ceiling for inputs this rule set newly generates. The declaration at 1296 charges the *ceiling breach*, not the *missing bound*. | State `AFF`'s specified worst-case degree over spec-defined quantities, or add M1(b) to the declared violations. |
| **S3** | serious | L04, L05, I17-in-a-loop (OD11) | 564–570 (II5) vs 630–635 (II8) | II5 keeps a version live "while any value mentions it"; II8 retires **every** body-captured atom at the back edge. A body value guarded by a body atom that reaches the back edge (I17's `g#` is the stated case) has two different post-states and no rule selects between them: acceptance is then not a total function of the source (M1(a)), and the cost differs (a retained atom widens `|Γg|`; a retired one forces the `⊓` collapse). | State the retirement rule — retired atoms collapse by `⊓` as in II3's `Gmax` clause — and subordinate II5's liveness clause to it. |
| **S4** | serious | L03, P3 (§9.4 head row) | 603–628 (II7(d)); boundary G3/G4 | LOOPCHK's "binders matched **positionally** by the binding each is attached to (p, q) — a determined match, no search" is stated only for binders attached to a binding. P3's derivation writes `head row { exists t. Occupied(ρP, t) }`, whose binder is an index term attached to no binding, and L03's binders additionally carry a `dis` obligation. For that case there is no procedure, no degree and no ground for the "no search" claim. | Restrict II7(d) to binders attached to a binding and reject the rest, or state the matching procedure and its degree. |
| **S5** | serious | L03 | 1174 (§3.21's L03 row), 1168 ("Nothing below is a capability loss"); boundary derivation L03 | §3.21 records L03 as "accept, cost: one written head row … yes, with a stated cost". The boundary derivation finds the case's final line `release(p); release(q)` **underivable**: §1.1's identity grammar has no existential-binder form and I7's premise is `Own(x) = ρ`. A required CASES line with no rule is a difference the file's own scheme must classify as a capability loss, a refusal with a named repair, or a correction of the case; it classifies it as none, and the blanket "nothing below is a capability loss" is unsupported. | Add the binder identity form to §1.1 and an I7 clause for it, or classify L03 in §3.21 as a capability loss. |
| **S6** | serious | M8 accounting | 1295 (declared M8 row) vs 666–680 (II10 grammar), §4 (III1–III13) | The M8 declaration undercounts its own inventory, so the declared figure understates the declared violation. "Seven clause kinds" against II10's grammar, which lists the row line, `carve`, `split`, `absorb`, `relocates`, `end`, `result`, `requires`, `ensures`, `outcomes` (ten) plus II12's `when φ \| otherwise` (eleven). "≥34 cards" omits the ≈14 measure names the writer must know (`len_of`, `cap_of`, `off_of`, `free_of`, `carved_of`, `extent_of`, `size_of`, `backing_of`, `head_of`, `pos_of`, `slot_of`, `contents`, `target`, `Occupied`), the default plus three shipped automata, and — under M8(a)'s one-lookup addressability clause — the 56 rule IDs and 17 family names a diagnostic can cite. Writer-visible inventory is ≈69 vocabulary items plus ≈73 addressable IDs, roughly double the stated card count. | Recount against II10's grammar and §4's measure list and restate the figure. |
| **S7** | serious | P1 | 796–802 (II15's three repairs) | II15 promises three repairs; one is not a repair and one is forbidden as a sole fix. "Name the entry" relocates the same obligation to the window head, where `PART` demands the same `k ≠ i, k ≠ j` (OD12's own pricing); "move the access outside the window" is a non-local restructuring, which M7's Price forbids as the only fix. Only the written `use` step is a local repair, and F5 shows it is also the only route. | State the `use` step as the primary repair and mark the other two as relocations, not fixes. |
| **S8** | serious | P3 §9.4, L03, any `Gmax` collapse | 1258 (`COVER`), 533–540 (II3 collapse), OD2 at 1315 | Three rejection classes carry no M7 payload. (i) The `COVER` rejection of a read through a stored coarse `ptr<ρP,Node>` has no stated diagnostic and no repair short of redesigning links into handles — non-local and unnamed. (ii) II3's `Gmax` collapse "makes every later access under that key reject with the collapse named"; OD2 states the only repair as "restructure the branches", again non-local and unnamed. (iii) L03's end-through-a-binder has no rule, hence no payload at all. M7's metric (payload non-empty for 100 % of corpus rejections) fails, undeclared. | Give `COVER` and the `Gmax` collapse a diagnostic with a named local repair, or add M7 to the declared violations with these three classes listed. |
| **S9** | serious | P1 | 877–900 (II19 `ESC`), OD7 at 1321 | II19's `ESC` message names the repair "pass the handle, or promote with `split`". OD7 records that an **element** of a non-byte container has no promotion-with-merge, so for P1's `part_i`/`part_j` the second named repair does not exist; the diagnostic points at a construct the rule set does not supply. | Make the `ESC` message name only the handle route when the entry is an element of a non-byte container. |
| **S10** | serious | P2, P5 (not derived) | rules I13, III6 (arena), OD6 at 1319 | The two items that drive the candidate's largest per-item writer costs are derived nowhere in the three derivation files, which cover S01–S07, B01–B13, L01–L06, P1, P3, P4, P7, P8 only. OD6 asserts P5-generic "needs a written `carve` clause to recover the eight per-column facts" and I13/III6 assert one `INV` step per `carve` against a written arena invariant; both are cost claims with no line-by-line support, so the annotation count for the highest-cost items is unfalsifiable. | Derive P2 and P5 line by line, or drop the per-item cost claims made about them. |
| **S11** | serious | all 31 derived items | §3.21 (1136–1166) | §3.21 tabulates *rules used* and a verdict, never *written artifacts*. The costs it does state ("two cost a written loop-head row") are for the 26 CASES only and omit P1's `focus` + three `where` obligations + `use` step, P3's written invariant + head row + per-read `INV` step, P4's store parameter + `reserve`, and P7's two extra writer lines. §2 below supplies the count the candidate does not. | Add a written-artifact column to §3.21 and extend it to the derived programs. |
| **m1** | minor | §5 | 1250–1255 | The quantity list defines `N` (live coarse names), which no degree mentions, and defines no symbol for guard atoms or versions, on which `STATE`, `JOIN`, `GUARD` and `DIS` all depend. | Add an atom-count symbol; use or delete `N`. |
| **m2** | minor | §1.1 / OD2 | 66–69, 1315 | §1.1 calls `Gmax` "a specification constant (provisional `Gmax = 2`) … not a budget in M1's sense" while OD2 calls the value unjustified and awaiting a corpus measurement; the two senses of "provisional" should be reconciled, since only the first keeps `L = 2^Gmax` a constant in every degree. | One sentence stating that `Gmax` is fixed by the specification and that a revision is an M11 amendment, not a measurement-selected knob. |
| **m3** | minor | L02 | 603–628 (II7's second example) | The diagnostic names the head state and the body's exit state but no repair; M7 wants the restructuring named. | Add "restore the state before the back edge, or write a head row admitting the exit state". |
| **m4** | minor | §5 family 5 | 1262 | `DIS`'s origin-set degree is `O(\|π\|·\|π'\|)` over a quantity absent from the quantity list. | Add the origin-set size to §5's quantities. |
| **m5** | minor | II13 | 748–760 | "carves by refinement" has no entry-correspondence procedure, so `ROWSUB`'s `O(A)` is asserted over a matching step that may not be positional. | State that entries correspond by name and that an unmatched entry rejects. |
| **m6** | minor | B13 | 581–592 (II6(b)); late-branches G5 | II6(b)'s prose and its formal criterion disagree about an else-edge-placed cleanup, so the *writer cost* of B13 — one extra source branch in every conditional-cleanup program — rests on prose rather than on the stated criterion. | State which of the two governs. |

---

## 2. Writer burden per item, as the derivations actually spend it

Counting only what the writer must type beyond the program the item states: `A` = annotation
(loop-head row, `where` clause, written invariant, struct store parameter, row clause the
program text does not show), `U` = written `use`/`INV` step, `R` = source restructuring
(a branch or statement added to satisfy a rule).

| Item | A | U | R | What is written | Note |
|---|---|---|---|---|---|
| S01–S07 (7 items) | 0 | 0 | 0 | — | zero-cost segment |
| B01–B08, B10, B11 (10) | 0 | 0 | 0 | — | zero-cost segment |
| B09 variant | 0 | 0 | 1 | `if !cond { put(p, 20) }` | **rejects as written** (F4) |
| B12 variant | 0 | 0 | 1 | `if !c { release(a) }` | **rejects as written** (F4) |
| B13 | 0 | 0 | 1 | the complementary release | **rejects as written** (F4); classified in §3.21 as a refusal with a named repair that its own II1 refuses |
| L01, L04, L06 | 0 | 0 | 0 | — | |
| L02 | — | — | — | rejection; no repair named (m3), and the `n ≤ 1` variant has no vocabulary at all | |
| L03 | 1 | 0 (+1 unstated) | 0 | existential head row: 2 binders, 3 state facts, 1 `dis` obligation | final release line **underivable** (S5); consecution needs an `INV` step the derivation does not write |
| L05 | 1 | 0 | 0 | head row `{ ρa : ite(active, Init, Gone) }` | |
| P1 | 4 | 1 | 0 | `focus` head + 3 `where` obligations, 2 captured index bindings | `use k != i && k != j` is the **only** admitting route (F5); the branch route the program names does not derive |
| P3 | ≥5 | ≥1 per traversal read | 0 | written `Links` invariant (4 conjuncts), 4 captured index bindings, loop-head row, parametric-entry `focus`, an occupancy `where` per parallel element, `ensures` per pool op | one `INV` step at every read through the invariant; the cost grows with read sites, not with declarations |
| P4 | 2 | 1 | 0 | `struct Cursor<β>` store parameter; `free` row naming both `ρv` and `β` | `reserve` (or one `INV` step) buys the second read |
| P7 | 0 | 0 | 2 | `drop(old)`; empty the slot before `free` | forced by R2; M2(ii) forbids inserting them |
| P8 | 0 | 0 | 0 | — | |

Totals over the 31 derived items: **13 annotations, ≥3 written proof steps (P3's grows per
read site), 5 source restructurings of which 3 do not derive under II1 as written.** The
burden is not spread: 26 of 31 items cost nothing, and four items (L03, P1, P3, P4) carry
all of it. That concentration is the honest headline and §3.21 does not state it (S11).

---

## 3. Family audit — M1 procedure and M10 degree

| # | Family | Fixed terminating procedure? | Degree stated? | Verdict |
|---|---|---|---|---|
| 1 `COVER` | yes | `O(D)` | **understated**: the "meet of the keys it meets" is `O(K)` (S1) |
| 2 `STATE` | yes | `O(S·L)` | **understated**: omits per-statement `KILL`/`COVER`/`MEAS`/`INST` (S1) |
| 3 `GUARD` | one cube: yes | `O(Gmax·α)` per query | **wrong measure**: prices a cube, not the enumeration over live atoms, which is unnamed and exponential (F1) |
| 4 `JOIN` | yes | `O(K·L)` | ok |
| 5 `DIS` | yes | `O(D + PART + EXT)`, origin sets `O(\|π\|·\|π'\|)`, guarded `O(L)` | ok modulo m4; inherits `PART`'s missing `AFF` factor |
| 6 `EXT` | yes | `O(term) + AFF` | ok — and the only row that composes `AFF` honestly |
| 7 `AFF` | "runs to its specified completion" | **no degree**, only a measurement | S2; declared as a ceiling breach only |
| 8 `PART` | yes | `O(E)` / `O(E²)` / `O(1)` per binder | **understated**: every pair is an `AFF` call (S1) |
| 9 `KILL` | yes | `O(F·DIS)` | ok per call; per-statement composition unstated (S1) |
| 10 `INST` | yes | `O(A·D)` | **understated**: omits II10's within-one-entry test over `E` (S1) |
| 11 `ROWSUB` | pointwise | `O(A)` | m5: carve refinement has no correspondence rule |
| 12 `ESC` | syntactic | `O(B)` | ok |
| 13 `AUTO` | table | `O(1)` | ok |
| 14 `INV` | verifies a written step | `O(W)` | ok; the writer cost is real and counted in §2 |
| 15 `MEAS` | yes | `O(F) + AFF` | ok |
| 16 `LOOPCHK` | two entailments, no fixpoint | `2·O(K·L)` | **incomplete**: existential binder matching is unspecified for binders not attached to a binding (S4) |
| 17 `OUT` | yes | `O(arms)` | ok |
| — | **feasible-assignment enumeration** | — | — | **unnamed** (F1) |
| — | **II15 within-one-entry test** | — | — | **unnamed** (F2) |
| — | **default-carve derivation** | — | — | **no rule at all** (F3) |

Three of the model's most frequently invoked judgments are not in the table. A family that
is not named cannot be given a bound, so §5's closing claim "**M1 — satisfied** … no search,
no budget, no iteration cap, no solver state, no order dependence" is not established by the
table it rests on.

---

## 4. The condition-atom bound

The candidate does carry condition terms (II1–II6, III12), so the bound is in scope.

- **Depth**: bounded, `Gmax = 2` (§1.1, line 66), with a deterministic diagnosed overflow
  (II3's oldest-atom `⊓` collapse). Stated. Unjustified by evidence, declared in OD2.
- **Leaves per value**: `L = 2^Gmax = 4`, a constant. Stated and used correctly in `JOIN`,
  `STATE` and `DIS`.
- **Atoms live in `Γg`**: **unbounded and unpriced.** II1 adds an atom per `if` on a
  binding; II12 adds `g#` per undischarged two-armed row; II5 adds a version per boolean
  assignment and retires none in straight-line code. No quantity symbol exists for the set
  (m1), and II6(a) quantifies obligations over assignments of *all* live atoms (F1).
- **Versions**: unbounded; retired only by II8, which contradicts II5 (S3).
- **Relations**: `=`, `¬`, `const` only, decided by union–find. This half is genuinely
  cheap and genuinely search-free; it is the enumeration, not the relation algebra, that is
  unbounded.

So the bound the candidate states (`Gmax`) is the right kind of bound on the wrong
quantity for M1/M10 purposes: it bounds precision per value, not work per obligation.

---

## 5. Construct count (M8)

Recount against the rules' own grammars: identity kinds 3 + guarded identity + origin set
(5); place forms (6); index-term forms (3); pointer type forms (3); struct store parameter
(1); grades (4); row clause kinds (11, per II10 + II12 — not the declared seven); carve
entry forms (7); writer statements and constructs — `focus`/`where`, `use`, `alias`,
`invariant`, `states`/`transitions`, `split`, `merge`, head row, `par_for`, `∥`, `outcomes`
arms (11); measure vocabulary (≈14); automata (4). Total ≈ **69 writer-visible items**,
plus **≈73 addressable IDs** (56 rules + 17 families) that M8(a)'s one-lookup clause
charges. The declared "≥34 cards" is roughly half the inventory (S6). The declaration that
M8 is violated stands; its magnitude does not.

---

## 6. M7 audit of every rejection in the derivations

| Rejection | Payload | Local repair exists? |
|---|---|---|
| I1/I3/I4/I5/I6/I7 premise failures (S03, S06, S07, B03, B06, B07, P7, P8, L04, L06) | rule + missing fact + site | yes |
| I17 unproved-capacity arm (P4, P1) | rule + push site + `cap_of > len_of` | yes (`reserve`) |
| II6(a) double consumption (B12) | rule + "prove ¬(c ∧ d), e.g. bind d = !c" | **no — the named repair rejects** (F4) |
| I8 undischarged arm (B13) | rule + "release a on the ¬cond path" | **no — the named repair rejects** (F4) |
| II15 element access at a runtime index (P1) | rule + head + fact + three repairs | one of three works (S7); the writer's natural route re-rejects (F5) |
| II7 consecution (L02, L05 variants) | head state + body state | no repair named (m3) |
| II19 `ESC` (P1) | fixed parser message + two repairs | one of two exists (S9) |
| `COVER` through a stored coarse pointer (P3 §9.4) | none stated | **no** (S8) |
| II3 `Gmax` collapse | "the collapse named" | **no local repair** (S8) |
| end through an existential binder (L03) | no rule, so no payload | **no** (S5, S8) |
| I22 live minted name at a back edge | site + "end it, merge it, or sink it" | yes, though (c) is a restructuring |

M7's stated metric is a non-empty payload on 100 % of corpus rejections with a named local
fix. Four rejection classes fail it and M7 is absent from the declared-violations table.

---

## 7. Verdicts overturned

1. §5's "**M1 — satisfied**" (1276–1282) — overturned by F1, F2, F3: two invoked judgments
   are unnamed, one has no rule, and one quantification is exponential in an unbounded set.
2. The **declared-violations table** (1290–1299) — overturned as incomplete: **M7** belongs
   in it (F4, F5, S7, S8, S9), and so does M1(b) for `AFF` (S2) and M10(a)'s per-declaration
   total-work clause (S1).
3. §3.21's **L03** row ("accept … yes, with a stated cost") and the blanket "**Nothing below
   is a capability loss**" (1168) — overturned: one required line of L03 is underivable and
   unclassified (S5).
4. **B09 variant, B12 variant, B13 repair** marked "accepted as expected" in
   derive-window-focus-late-branches §3.2, §6.2, §7 — overturned: under II1 as written all
   three reject; the derivations accept them only on a worked example that contradicts the
   rule (F4).
5. derive-window-focus-late-branches §8.5's P1 row "**a diagnostic at `v[k]` naming the
   missing fact — delivered**" — overturned on M7: delivered as a name, not as a repair (F5,
   S7).
6. §5's degrees for `STATE`, `COVER`, `PART` and `INST`, and OD12's `PART` pricing —
   overturned as understated (S1).
7. §5's `GUARD` row as the price of the guard machinery — overturned: it prices one cube
   test, not the work II6(a) demands (F1).


# File: critique-window-focus-cross.md

# Critique: `window-focus`, lens **CROSS** (cross-examination)

Key: `window-focus`. Lens: CROSS. Round: core2. Date 2026-09-16.

Read against the other two candidates' rules and derivations:
[`rules-value-semantics.md`](rules-value-semantics.md),
[`rules-brand-context-bounded.md`](rules-brand-context-bounded.md),
`derive-value-semantics-{straight-and-early-branches,late-branches,boundary}.md`,
`derive-brand-context-bounded-{straight-and-early-branches,late-branches,boundary}.md`,
against [`rules-window-focus.md`](rules-window-focus.md) and its three
derivations.

Abbreviations: **WF** = `window-focus`, **VS** = `value-semantics`,
**BCB** = `brand-context-bounded`.

This file selects nothing and ranks nothing. It reports, for WF only:
what the other two admit that WF refuses or cannot derive; what WF admits that
they refuse; what WF asks the owner to accept that they do not; and what WF
spares the owner. Where a rule is missing on WF's side and present on both
others', the other side's rule number is given, because the round exists to
decide on rules, not on stated verdicts (VERDICT-CORE §6, O19).

Severity, as the lens fixes it: **fatal** = a hazard admitted, or a requirement
violated and *not* declared in §6; **serious** = a wrong verdict, an understated
cost, an unnamed family, a missing rule; **minor** = wording.

---

## 1. Findings table

| # | Sev | Item | WF line / rule | The other side's rule | Finding | Fix (one sentence) |
|---|---|---|---|---|---|---|
| **X1** | **fatal** | P9 `pick`, and every `result : ι` with `ι = {ρx, ρy}` | `rules-window-focus.md:42` (`ι ::= … \| { ι, … }`), `:678` (`result : ι`), `:1262` (`DIS` costs origin sets), I3 `:151`, I4 `:168`, I7 `:208` | BCB **II-15** (row 2: a write through an origin set leaves `st` unchanged; row 1 strong leafwise under a term) + **III-11**; VS **II-20** (`subscript … origin`, footprint = the origin union) | WF puts finite origin sets in the identity grammar and prices `DIS` over them, but **no rule in set I, II or III states the effect of a write, `take`, `put` or `end` whose identity is an origin set**. I3's effect is `St(π) := Init` for a *place*; an unresolved `{ρx,ρy}` either marks both `Init` (exactly the soundness-F4 hazard BCB's II-15 exists to repair — a later `read(y)` on storage never written is then admitted, R1(a)(i)) or is undefined. Not in §6's OD1–OD12. P9 is a required program and WF derives it nowhere | Add an II-15-shaped rule: a read through `{ρ₁…ρₙ}` requires `Init` on every member, and a write leaves `St` unchanged and only kills contents facts, with the named repair "declare the callee's discriminant `ensures result : ite(c,ρx,ρy)`" |
| **X2** | **fatal** | S01, B05, B06, B10, B11, P1, P3 — every alias-survival step | §5 family 5 `DIS` `:1262`; I1 `:125`; I11's "never by inequality of names (R4(b), C3)" `:270` | VS **PATH** (§5 family 4: prefix comparison over `root → plane → extent → element`) and **PLANE** (family 3, `O(1)` table); BCB **KEY**/**JF-OVL** (`mayEq` restricted to *the same base*), plus **I-1**'s `'a # x` from the allocator's written `ensures` | `DIS` is declared **total** but has **no base case for two identities rooted at distinct `⊑`-roots**, and R4(b)/C3 forbid grounding it on name inequality. Read one way, `contents(ρa)` does not survive a write to `ρb` and S01/B05/B06 reject; read the other way, WF is grounding distinctness on freshness of names, which is what O8(a) and R4(b) refuse. WF's own derivations record it three times (straight G2, boundary G4, late G8) and §6 does **not** list it | State the base case as a rule: two identities with no common `⊑` ancestor are `dis`, grounded on the containment forest (not on name inequality), exactly as VS's PATH grounds it on the containment tree |
| **X3** | **fatal** | B10, B11, B12, B13, P8, I8's scope exit, L05 | `:63` (`Own` is a checker structure), I7 `:208`, I8 `:227`, II3 `:533` ("for every key of `St` and every fact in `Facts`"), §3.21 `:947` | VS **II-2** — marked **[stated here]**, with the mixed case an explicit REJECT and the reason given ("conjunction alone … an R2 leak admitted with no diagnostic"); BCB **II-3 row 5** (`held`/`discharged` → `ite(α,·,·)` if the atom is available, **else reject**, R2 definiteness) | WF's join rule merges `St` and `Facts` only. **`Own` — the obligation state — has no join rule anywhere**, yet §3.21 claims B10 "obligation retained, guarded", B11, B12 and P8 accept, and I8's premise quantifies an obligation "on every feasible guard assignment" with nothing defining the guarded obligation. R2's definiteness at a join is the second of VERDICT-CORE §6's three convergence places and both other candidates wrote it as an explicitly-added rule; WF did not, and §6 does not list the omission. WF's late-branches gap G2 records it | Add `Own` to II3's merged structures with BCB's row 5 semantics (guarded when the atom is available, reject otherwise), and state what a `Gmax` collapse does to a guarded obligation |
| **X4** | **fatal** | P1's "else the writer must branch"; every bounds, index, capacity or measure fact a source branch would discharge | II1 `:501` ("a discriminant that is an *expression* captures nothing"); II5 `:564` (a relation is recorded only for `c`, `!c`, a literal); III12 `:1225`; derivation `derive-window-focus-late-branches.md:515-528` | BCB **ATOM** (`rules-brand-context-bounded.md:103`) — every atom operand is frozen, so `if i != j` yields the atom `i ≠ j` — plus **II-1** (push α onto Γ) and **JF-ENT**; VS **II-1** + **EXT**/**REF** ("EXT's inputs include the syntactically enclosing branch condition AS A PREMISE") | Under WF a source branch on a scalar comparison contributes **nothing but a free atom**. Consequence, wider than the derivation states: `if i != j`, `if k < len(v)`, `if n0 < c0`, `if d != 0` can never discharge an `AFF`/`EXT`/`MEAS`/R11 obligation. P1's second required route is underivable (WF's own derivation says so and still records "accepted with a stated cost"); the only admitting route anywhere is a written `use`/`INV` step, which makes O15(b) load-bearing in a way §5 does not charge. Not in §6 | Add an II1 clause binding a guard atom to its defining expression when the discriminant is a pure comparison over captured index/measure bindings, and let `AFF`/`EXT`/`MEAS` read `Γg` — BCB's ATOM and VS's Π are two working spellings |
| **X5** | **fatal** | B09 variant, B12 variant, I8's own second example `:239`, B13's named repair `:1329` | II1 `:501` (an expression discriminant captures nothing) vs II5 `:564`, II6 `:581`, §3.21 `:947` | BCB **JF-ENT** step 4 (booleans by unit propagation over `b₁ = ¬b₂`) with `¬b` a first-class atom form (`:37`); VS **II-1** (the else-arm's Π gains `!c`) | `if !cond { … }` is an *expression* discriminant, so by II1 it captures nothing — yet II5's, II6's, I8's and §3.21's accepting variants for B09, B12 and B13 all require the arm to be checked under `¬atom(cond)`. The rule set therefore contradicts itself at the exact point where **B13's deliberate refusal is justified by the availability of its repair**; if the repair is underivable, the refusal has no named repair and M7(a) fails undeclared. WF's late-branches gap G4 records the conflict | Amend II1 to capture `!c` (and `c == literal`) as the negation of the existing atom, so the accepting variants and the B13 repair derive |
| **X6** | serious | P3, "removal invalidates every path to `m`" — an R1(e) test | III11 `:1184` ("a generation counter — declared absent"), I20 `:417`, OD1 `:1314` | BCB **III-6** (a fresh ghost index `@g` per reusable extent) + **I-23**/**I-24** + **KEY**; VS **III-7** (ghost `γ`, `gen(P[σ])`) + **I-20** + **III-8** | After `remove(P,m); insert(P,x) reusing m`, WF **accepts** `read(deref(pm).data)` and reads the new occupant with no diagnostic. Both other candidates refuse it by rule. R1(e) names P3's removal clause as a test of R1, so this is a requirement test WF does not pass. It is declared (OD1, III11) — hence serious, not fatal — but the OD1 entry understates it as "wrong-occupant logic error" without recording that **both alternatives deliver it** | Either state the boundary as an O1(b) position the owner is being asked to ratify, or buy the fourth name kind; the OD1 row should name BCB's III-6 and VS's III-7 as the two existing constructions and price them |
| **X7** | serious | I12 merge / `split_at_mut` + `merge`; the coalescing arena | I12 `:293`, I14 `:329` (`absorb`), §5 family 6 `EXT` `:1262` | VS **II-23** — declares split-and-rejoin **absent by rule** and says so ("a coalescing arena … is OUT of the language, not merely unwritten", DV-9); BCB supplies no merge and D3 records that a coalescing arena is outside JF-LIN's 9-schema table | WF is the only candidate claiming the rejoin, and it is the only one with **no judgment family that can decide it**: `EXT` returns *disjoint / contains / overlaps* only, so I12's premise "extents are adjacent and cover the range being restored" is undecidable by any named family, and `free_of(ρ) := free_of(ρ) ∪ [lo,hi)` is asserted to be "re-established by the parent's written invariant" with no `INV` instantiation exhibited. The capability that O7 evidence rests on (byte splits are first class *and* re-joinable) is therefore unsupported | Add an adjacency/cover output to `EXT` (or a tenth `JF-LIN`-style schema) and exhibit I12's `INV` step, or withdraw the merge and match VS's honest II-23 |
| **X8** | serious | P4's `Cursor` | `derive-window-focus-boundary.md:31-45` (`struct Cursor<β>`); §1.3's table row `:82`; no numbered rule | BCB **I-19** — a numbered rule with a **declaration-site rejection** for a free identity in a field type, forcing `struct Cursor<'v,'b> { … } where backing('v) = 'b`; VS §1.1 + **DV-1** (declares the stored projection a capability loss rather than pretending it is cheap) | Two problems. (i) WF has **no numbered rule** for a nominal's store parameters — only a row in the "what the writer writes" table — so the declaration-side check R7(a) has nothing to apply. (ii) `Cursor<β>` is one parameter short: `c.read()` needs `c.i < len_of(ρv)`, and `len_of` keys `ρv.desc`, which `Cursor<β>` cannot name; under R7's signature-only judgement a function taking only the cursor cannot discharge it. BCB charges two parameters plus a `where`; WF's verdict "accepted as expected" is costed at one | Promote §1.3's row to a numbered rule with I-19's declaration-site rejection and a `where`-clause form, and re-cost P4 at two parameters |
| **X9** | serious | P5 generic over its column type; M9's headline | OD6 `:1319`, II14's declared-type default carve `:767`, §5's R15 note `:1301`, M10 row (ii) `:1295` | VS **I-2**/**G3** — the plane map is a *declaration*, so `PLANE` refutes overlap by `O(1)` table lookup, "no annotation, no proof, sound for any element type", and check-once under R15 holds; BCB **JF-LIN** schema 7 (sibling fields) | WF's default carve is a function of the **declared type**, so `Cols<E>` gets one opaque entry per parameter and P5's eight per-column facts need a written `carve` clause. OD6 presents this as a forced trade against R15(a) — but it is **not** forced: VS and BCB both deliver per-field distinctness under abstraction with zero written clauses and keep check-once. The same mechanism is also the ground of WF's declared M10 edit-reach violation (a one-token field-type change changes the entry set of every row naming the type — a reach strictly looser than BCB's FN-1 caller boundary) | Rewrite OD6 to record that two constructions deliver P5 under abstraction without the R15 conflict, so the owner sees a choice rather than a law |
| **X10** | serious | L03's `release(p); release(q)` | II7(d) `:603` (existential head-row identities), I7 `:208` (`Own(x) = ρ`), §1.1's identity grammar `:42` (no binder form); `derive-window-focus-boundary.md:472` gap G3 | BCB **II-7** (a head invariant over `origin(x)`, `st(origin(x))`, `origin(x) # origin(y)`) + **I-3**/**I-4**, which its boundary derivation confirms line by line ("`release(p); release(q)` accepts") | WF's II7 admits existential head-row identities but the identity grammar has no binder form and I7's premise is an owner binding at a **coarse name**, so ending an obligation reached through an existential binder has no rule. BCB derives the same two lines. §3.21's L03 row reports "accept, cost: one written head row / yes, with a stated cost" and does not record the underivable line, which its own derivation does | Add a binder form to `ι` and an I7 clause for an obligation reached through a head-row existential, or record the loss in §3.21 and §6 |
| **X11** | serious | S03, S04, S05, S06, B02, B03, B04, B07, B08, B09, L01, L02, L03, L06 | I4 `:168` — premise "`type(π)`'s class is affine or linear" — contradicted by I4's, I3's and II7's own worked examples, which `take` copy integers | BCB **I-7** (premise is `st(π) = Init` after `KEY`; no class restriction); VS **I-7** (`take` is `replace(inout d, None)`, total, no class premise) | The entire CASES fragment takes copy scalars. Under I4's premise as written, **the `take` line of fourteen items has no rule**; the rule set's own examples apply I4 to exactly those shapes. Both other candidates carry no class restriction on the analogous rule. Recorded as gap G1 in two derivations; absent from §6 | Delete the class premise from I4 and state instead that no obligation moves when the type is copy |
| **X12** | serious | L04, L06, and every loop exit | II9 `:649` (delegates the exit join to II3), II3 `:533` (the merge is indexed by an atom), II8 `:630` (body atoms are retired at the back edge) | BCB **§1.2** introduces a `Live` lattice element *for exactly this* and **II-3 row 3** states the atom-free join (`Init ⊔ Uninit = Live`, `Live ⊔ Gone = ⊤`); VS **II-9** joins over all exit edges including the zero-iteration edge by intersection | A loop exit has no available atom, so II3's merged value is undefined there. L04's and L06's verdicts rest on II9's worked examples rather than on a stated merge. Both other candidates state the atom-free case as a rule, and BCB added a lattice element to get L06's verdict right. Gap G5; absent from §6 | State II3's atom-free case explicitly, with the meet used and what it does to `Own` (see X3) |
| **X13** | serious | II19's `ESC`, and I7/I17/I21's `Span(ρ)=∅` premise | II19 `:877`; I7 `:208`; I17 `:369`; I21 `:434` | BCB **II-13** — "nothing is consumed, so **a projection may be stored** and two projections of one base may be live at once — **the base's writability is not suspended**" (D4) | WF is the only candidate in which *holding a projection blocks an operation on the base*: a live span binding refuses `free`, `push`'s reallocating arm and `compact`. That is a borrow-scope-shaped restriction, and the ladder's own closing row records that no hazard in Steps 0–5 needs one. II19 prices the second name kind against M4 and M8 but **does not price it as a capability cost**, and §6 does not carry it | Add an OD row: "a live span binding suspends `end`, relocation and reallocation on its base — a scope-shaped restriction BCB's II-13 shows is not forced", so O7/O12 evidence carries both halves |
| **X14** | serious | O11's price; B12 and any third correlated condition | II3's `Gmax` collapse `:533`; `Gmax = 2` `:52`; OD2 `:1315` | BCB **TERM** (`:131`) — a hard bound of **one** atom, with the join **rejected at the join**, naming both atoms, the identity and two repairs; VS **II-5**/**O11(a)** — no terms at all, so no bound is needed | Two costs WF understates. (i) `Gmax = 2` is set by nothing (OD2 admits it), while BCB's bound of 1 is structural and VS needs none. (ii) WF's overflow **collapses silently at the join** and surfaces as a rejection at a *later access* ("with the collapse named"); BCB rejects at the join at the site of the second atom. M7(a) asks for a payload naming a location where a local fix exists, and a collapse reported at a downstream access does not | Either take BCB's bound-at-one with a join-site rejection, or state the collapse's M7 payload as naming the join and both atoms, and record the deferred-diagnostic cost in OD2 |
| **X15** | serious | P6 | II20's worked example `:918` writes `head row { ρ : [0, w*s) Init, [w*s, n) Uninit }`; the boundary summary does not charge it | BCB **JF-LIN** schemas 4 and 5 (strided partition, floor-division), which its §7 states give P6 "**zero written steps**"; VS **EXT**'s `ProvedRangePartition` with three named side conditions, likewise zero written steps at the call | WF needs one written prefix-run loop-head invariant for P6's rejoin (its own II20 example writes it, citing C-S5). Both others discharge P6's shape from a named schema with nothing written. P6's stated requirement is "iteration overlap permitted **from the arithmetic of the ranges**" | Charge P6 at one written head row in the cost tables, or supply a schema that discharges the prefix-run rejoin without it |
| **X16** | serious | M3 / O15(b) exposure | §5 family 14 `INV` `:1262`; II15's `use k != i && k != j` `:796`; III1's "never by a bare assume" `:1011` | BCB **M3** line (`:1237`): "a disequality on two runtime scalars has **no such rule**, so `use i != j` is not writable and the only route is a written branch"; VS **INV** (§5 family 10) instantiates a *written invariant* at *written* arguments | II15's own accept line is `use k != i && k != j` — a `use` over a disequality between two runtime scalars with no written invariant behind it. Under O15(b) as VERDICT-CORE states it, and as BCB reads it, that is exactly a bare assume. With X4 (no branch route) WF then has **no** admitting route for P1's read at all | Either exhibit the written invariant `use k != i && k != j` instantiates, or accept X4's fix so the branch route exists |
| **X17** | serious | M8 inventory | §5's M8 row `:1295` ("≥34 cards") | VS §5.1 counts **17**; BCB §6.1 counts **≈46**, both against an unset `K` (O6) | WF's count omits at least three taught items its own rules require: the nominal store-parameter form (X8), I12's merge/`absorb` pair, and origin sets as an identity form with their access rule (X1). The declared figure is therefore low as well as, per OD4, unmeasurable | Add the three omissions to the count and say the figure is a floor |
| **m1** | minor | OD7's framing `:1320` | — | BCB **II-13** + **I-19** | OD7 says element-level projection across a cut "remains unwritable" and names as the closing move "an element promotion with a merge rule". BCB *writes* the capability today by a different construction (sub-identity names plus a closed nominal row). The defect should name the existing alternative, not only the unbuilt one | Cite BCB's construction in OD7 |
| **m2** | minor | §3.21's B10 row `:947` | — | — | "obligation retained, guarded" asserts a result X3 shows has no rule; the row reads as a derivation | Footnote the row to the missing `Own` join |
| **m3** | minor | `⊥` in II3's collapse `:533` | §1.1's state grammar `:46` | — | `Gone ⊓ Init = ⊥` introduces a state `⊥` that §1.1's `state s ::= a state of auto(type(π))` does not contain; BCB puts its analogous `⊤` in the lattice explicitly (`:47`) | Add `⊥` (or `⊤`) to the state grammar |
| **m4** | minor | §1.4's erasure row for `@E` `:105` | — | VS §1.5 lowers `inout` → `noalias` from a *convention*, one line | WF's `@E → noalias` carries two prose side conditions (R14(iv) access class, no foreign-agent automaton state) that no numbered rule states | Make the side conditions a clause of II11 or the erasure table's own rule |
| **m5** | minor | OD10 `alias ρ σ` `:1323` | — | BCB **II-16** (`pack`/`unpack` with a written premise); VS has no analogue and says so | OD10 reads as a WF-specific narrowness; it is in fact the strongest of the three positions and should say so | Note that VS forecloses the relation entirely and BCB routes it through a written pool invariant |

---

## 2. Items the other two admit that `window-focus` refuses or cannot derive

Stated as a ledger. "Cannot derive" means no rule of set I, II or III reaches
the line; "refuses" means a rule reaches it and rejects.

| Item | VS | BCB | WF | Rule numbers (WF ← others) | Finding |
|---|---|---|---|---|---|
| P1's writer-branch route (`if i != j { … }`) | accept — II-1 + EXT/REF read Π | accept — ATOM + II-1 + JF-ENT read Γ | **cannot derive** | WF II1/II5/III12 ← VS II-1, BCB ATOM | X4 |
| A source branch discharging any bounds/capacity/index fact | accept | accept | **cannot derive** | same | X4 |
| `if !c { … }` refining under `¬atom(c)` | accept — II-1 | accept — JF-ENT(4) | **cannot derive** (II1 says an expression captures nothing) | WF II1 ← VS II-1, BCB `:37` | X5 |
| P9's write through `pick`'s result | accept, footprint = origin union — II-20 | refuse with a named repair — II-15 row 2 | **no rule** (grammar only) | WF I3/I6/I7 ← VS II-20, BCB II-15 | X1 |
| Obligation state at a join | REJECT by rule — II-2 | guarded or REJECT by rule — II-3 row 5 | **no rule** | WF II3 ← VS II-2, BCB II-3 | X3 |
| `dis` of two unrelated roots | accept — PATH/PLANE | accept — KEY/JF-OVL by base | **no base case** | WF §5 `DIS` ← VS families 3–4, BCB JF-OVL | X2 |
| P3 "removal invalidates every path to `m`" after refill | REJECT — III-7 | REJECT — III-6 + KEY | **accept, reads the new occupant** | WF III11 ← VS III-7, BCB III-6 | X6 |
| P5 per-column distinctness under a generic column type | accept, 0 written — I-2/G3/PLANE | accept, 0 written — JF-LIN 7 | needs a written `carve` (OD6) | WF II14 ← VS I-2, BCB JF-LIN | X9 |
| P6 rejoin with 0 written steps | accept — ProvedRangePartition | accept — JF-LIN 4,5 | 1 written head row | WF II20 ← BCB JF-LIN | X15 |
| L03's `release(p); release(q)` | reject (CL, DV-6) | **accept** — II-7 + I-4 | **cannot derive** (G3) | WF I7/II7 ← BCB II-7 | X10 |
| `take` of a copy scalar | accept — I-7 total | accept — I-7, no class premise | **no rule** (I4's class premise) | WF I4 ← VS I-7, BCB I-7 | X11 |
| Atom-free loop-exit merge | accept — II-9 intersection | accept — II-3 row 3 + `Live` | **no rule** (G5) | WF II9/II3 ← BCB II-3 | X12 |
| Storing a projection without suspending the base | n/a (no pointer type) | **accept** — II-13, D4 | refuse — `Span(ρ)=∅` in I7/I17/I21 | WF II19 ← BCB II-13 | X13 |
| A nominal field holding a pointer, checked at the declaration | n/a (unwritable, DV-1) | accept — I-19 with a declaration-site rejection | table row only, no rule | WF §1.3 ← BCB I-19 | X8 |

**Net:** fourteen items. Four are refusals WF makes deliberately (P3-after-refill
is a declared boundary; the `Span` suspension is a priced consequence of II19).
**Ten are lines WF's rules do not reach at all**, and eight of those ten are
absent from §6.

---

## 3. Items `window-focus` admits that the others refuse

| Item | WF rule | VS | BCB | Verdict on the claim |
|---|---|---|---|---|
| P8(a) verbatim (`if c { free(a) } else { free(b) }`, use the survivor, free it) | II3 + II6(a)(b) | **REJECT at the join** — II-2 definiteness | accept — II-3 rows 4–5 | Stands as against VS; the claim is **not derivable as written** against X3 (no `Own` join) |
| B05, B06 (correlated targets, saved copies) | II2 guarded identity + II3 | **capability loss** — DV-6, `remove` kills liveness of every handle | accept — II-5 + II-14 | Stands as against VS; rests on X2's unwritten `DIS` base case |
| B10, B11, B12 | II3 + II6(a) | **REJECT at the join** — II-2 | accept | Stands as against VS; **X3** again |
| P4's stored cursor over a non-pooled container | III4's coarse `backing_of` + II19's first-class row | **capability loss** — DV-1/DV-2 | accept at two identity parameters — I-19 | Stands as against VS; **under-costed by one parameter and one `where`** vs BCB (X8) |
| L05 (checked Boolean guards later iterations) | II8's written head row + II5's literal-assignment relation | **REJECT** — O11(a), DV-6 | **underivable — 3 missing rules** | **Stands, and WF is the only candidate that derives L05.** Its own `active = false` is a literal assignment, which is precisely the case II5 records |
| I12 merge / a coalescing arena / re-joinable `split_at_mut` | I12 + I14's `absorb` | **declared absent by rule** — II-23, DV-9 | not supplied (D3) | **Does not stand as stated**: no judgment family decides adjacency-and-cover (X7) |
| P2 block distinctness with no written step per pair | I11 + III6 + `EXT` | needs `use IA(b,b')` **per pair** — III-4 | JF-LIN over frozen bounds, automatic | Stands, and is a real saving against VS's quadratic `use` tax |
| `M2(ii)` unconditional on owner decision O3 | III11 (no generation, ghost or otherwise) | satisfied under O3(a), **VIOLATED under O3(b)** | met **conditional on O3** (D5) | Stands, and it is WF's strongest structural claim — bought at X6 |

**Net:** eight items. Five stand. One (L05) is unique to WF. Two are
over-claimed: P8(a)/B10/B11/B12 rest on a join rule that does not exist (X3),
and the merge rests on a family that cannot decide its premise (X7).

---

## 4. What this candidate asks the owner to accept that the others do not

1. **A second pointer type form, with a fine name in it** — `ptr<ρ↓w.k,T>`,
   second class by the five-clause `ESC` judgment and refused by the *parser*
   in every cut position (II19, `:877`). This is O12(**b**) taken explicitly.
   VS asks for **no pointer type at all** (O12(a) by construction, §1.1);
   BCB asks for **one first-class pointer type** and takes neither boundary
   (II-13, I-19). WF is the only candidate asking the owner to hold "a carve
   entry may appear in a local binding's type and nowhere else".
2. **A third identity kind.** Coarse name + place + window entry, plus guarded
   values over all three (§1.1, `:33`). VS has three *forms* but one identity
   notion (a resolved path); BCB has one path grammar plus a ghost index.
   WF declares M4 violated for this and it is the declaration the owner is
   being asked to ratify.
3. **A bounded condition term with a magic constant.** `Gmax` (provisionally 2)
   with an oldest-atom collapse to `⊥` at overflow (II3, `:533`). BCB asks for
   the same O11(b) answer at a **structural** bound of one atom with a
   join-site rejection; VS asks for O11(**a**) and no terms. WF's constant is
   set by nothing (OD2) and its overflow diagnostic is deferred to a later
   access (X14).
4. **A lexical exclusivity window that suspends operations on the base.**
   `Carve(ρ)=∅` and `Span(ρ)=∅` are premises of `end`, relocation and
   reallocation (I7, I17, I21). BCB explicitly does not suspend the base
   (II-13/D4); VS has nothing to suspend. This is the one place where WF
   re-introduces a scope-shaped restriction, and it is priced only at M4/M8
   (X13).
5. **A restructuring per case, twice:** B13's automatic scope cleanup is
   refused (II6(b)) and P1's read is admissible by exactly one route, a written
   `use` step (X4, X16). BCB refuses B13 identically but keeps P1's branch
   route; VS refuses B13 by removing the representability of the mixed case.
6. **R1(e)'s P3 clause as a permanent boundary** (OD1/III11): a stale handle
   into a type-stable reusing pool reads the new occupant with no diagnostic.
   This is the owner's O1 asked as a *position*, not as a question — and it is
   the one requirement test where both alternatives pass and WF does not.
7. **A default carve that is a function of a declared type** — hence OD6's
   abstraction loss and the declared M10 edit-reach violation. Neither other
   candidate asks for a reach looser than the interface: BCB's FN-1 stops at
   callers, VS's declared violation is the transitive *instantiation* closure,
   which at least follows the call graph.

---

## 5. What this candidate spares the owner

1. **No ghost entity, and therefore no dependency on O3.** VS's M2(ii) is
   "satisfied under O3(a), **VIOLATED** under O3(b)" (§5.1) and BCB's is "met,
   *conditional on owner decision O3*" (§6.1, D5) — under O3(b) BCB must choose
   between violating M2(ii) and refusing arena/pool byte reuse. WF's III11
   declares the generation absent, so its M2(ii) argument survives either
   ruling. This is the cleanest thing in the candidate.
2. **No quadratic written-step tax on a pool interface.** VS's III-8 kills
   `LIVE(k)` for *every* handle of a pool after any `remove` unless `EXT`
   proves `σ(k) != σ(h)` (DV-6), which costs it B05, B06 and part of L03 as
   capability losses. WF keys state on places and pays none of it.
3. **No `use IA(b,b')` per pair.** VS's arena distinctness needs one written
   `INV` instantiation per block pair (III-4); WF's `EXT` over promotion
   extents and BCB's `JF-LIN` both decide it with nothing written (X-free).
4. **No closed identity row on every pointer-carrying nominal.** BCB's I-19
   forces a parameter per stored identity and charges `O(f²)` `#`-clauses at
   any interface producing an aggregate of separately-named storages (its §7,
   item 1). WF needs one parameter per stored pointer and no pairwise clauses —
   though X8 shows the P4 count is understated, so the saving is smaller than
   the boundary derivation claims.
5. **No whole-program instantiation closure.** VS declares M10's edit-stability
   clause violated because the callee-direction verdict-changing set is the
   transitive instantiation closure `I` (measured 82.4 s on a 375-line generic
   fixture). WF's `LOOPCHK` and `ROWSUB` are per-declaration; its own M10
   violation is different in kind (X9), and smaller in the callee direction.
6. **One join rule instead of two shapes.** BCB has both `TERM`'s refusal class
   (II-2) and the `⊤`/`Live` widening lattice; VS has intersection plus a
   definiteness rejection. WF's II3 is a single canonical merge with one
   collapse — which is a genuine M4 saving *if* X3, X12 and m3 are repaired.
7. **A loop rule with no fixpoint and no widening at all.** `LOOPCHK` checks
   initiation and consecution, two entailments, nothing else (II7, `:603`).
   VS's II-7 still runs a height-2 greatest fixpoint for `Σ`; BCB's II-6 joins
   at the head with a lattice. WF's is the only one with no iteration in it.

---

## 6. Counts

| Severity | Count | Items |
|---|---|---|
| fatal | 5 | X1, X2, X3, X4, X5 |
| serious | 12 | X6–X17 |
| minor | 5 | m1–m5 |

Of the five fatals, **four are rules that both other candidates wrote and this
one did not** (X1 ← BCB II-15 / VS II-20; X2 ← VS PATH / BCB JF-OVL;
X3 ← VS II-2 / BCB II-3 row 5; X4 ← BCB ATOM / VS Π). All five land in
VERDICT-CORE §6's second and third convergence places — the rules that split
and rejoin state, and the bridge from an identity to the runtime values that
determine it — which is the finding this round exists to test, reproduced
inside the candidate that was supposed to test it.

## 7. Item verdicts overturned

| Item | Candidate's verdict | Overturned to | Ground |
|---|---|---|---|
| P1 | "accepted with a stated cost" | **accepted with one required property not delivered** | P1's own "else the writer must branch" is underivable (X4), and the single remaining route is a `use` over a runtime-scalar disequality with no written invariant behind it (X16) |
| P9 | not derived; rules imply a verdict | **underivable; a hazard admitted on one reading** | No access rule for an origin identity set (X1) |
| B10, B11, B12, P8(a) | "accepted as expected" (§3.21) | **not derivable as stated** | No join rule for `Own` (X3) |
| B09 variant, B12 variant, B13's repair | "accepted" / "the repair is accepted" | **underivable** | II1 captures nothing from `if !c` (X5) |
| B13 | "deliberate refusal with a named repair" | **deliberate refusal whose named repair is underivable** | Same as above; M7(a) then fails undeclared |
| S01, B05, B06 | "accepted as expected" | **rest on an unwritten `DIS` base case** | X2 |
| L03 | §3.21: "accept, cost: one written head row — yes, with a stated cost" | **accept with one line underivable** | The candidate's own boundary derivation (G3) contradicts its §3.21 row; BCB derives the line (X10) |
| P4 | "accepted as expected" | **accepted with an understated cost** | One identity parameter and one `where` short; no declaration-side rule (X8) |
| I12 / `split_at_mut` + `merge` as O7 evidence | delivered | **claimed without a decision procedure** | `EXT` cannot decide adjacency-and-cover (X7) |
| OD6 ("this is the trade, not an oversight") | a forced trade against R15(a) | **not forced** | VS's plane map and BCB's sibling-field schema both deliver P5 under abstraction and keep check-once (X9) |
| §5's M8 row "≥34 cards" | a floor, declared over | **still low** | Three taught items omitted (X17) |


# File: critique-brand-context-bounded-soundness.md

# Critique — candidate `brand-context-bounded`, lens **SOUNDNESS**

Key: **brand-context-bounded**. Date 2026-09-16. Refuter lens: soundness.

Method: P4, P7, P8, B04, B05, B08, L03 and L05 were re-derived line by line from
`core2/rules-brand-context-bounded.md` alone, before opening
`derive-brand-context-bounded-{straight-and-early-branches,late-branches,boundary}.md`.
The findings below are the differences: lines whose verdict does not follow from
the rules, hazards under R1/R2/R3/R11/R12 that the rules as written admit, M2(ii)
violations carried by a library or a typed outcome, and rules that consume a fact
no rule produces. No candidate is selected here and no rule is repaired here; each
row carries a one-sentence fix where one exists.

Default on uncertainty: report. Where a reading exists under which a rule is
sound, the row says which reading, and why the document does not fix it.

---

## 1. Findings table

Severity: **fatal** = a hazard admitted or a requirement violated undeclared;
**serious** = a wrong verdict, an understated cost, an unnamed family, a missing
rule; **minor** = wording.

| # | Sev | Item / line | Rule | Finding | Fix |
|---|---|---|---|---|---|
| F1 | fatal | any post-`break` join (`L04` exit; P8's loop shape if a release moves onto the break edge) — `write(p, 20)` after the loop | `I-6` premise, `I-4` premise, §1.2 | Both premises are written `st ≠ Gone`. `⊤` ("no state is known", §1.2) is a distinct lattice element, so it satisfies `≠ Gone` literally. The boundary derivation itself computes `st('A) = ⊤` at L04's exit and rejects only `read` and `release`; `write(p, 20)` there is admitted by `I-6` as written — a write to storage that has ended on the break edge. **R1(ii) hazard.** | Restate both premises as `st(π) ⊑ Live` (i.e. `Live`, `Init` or `Uninit`), never `≠ Gone`. |
| F2 | fatal | **P4**, `b = c.read()` after `push(v,5)` (`derive…-boundary.md` P4.b) | `I-14`, `I-16`, `JF-KILL` | `I-14` closes a contract-asserted state downward **only for the constant `Gone`** ("asserting `Init` or `Live` on an ancestor implies nothing about descendants"). `I-16`'s ensures asserts the *term* `st('b) = ite(n0<c0, Live, Gone)`, which `I-14` does not cover, and §1.3 keeps `st` as a per-point map, not one of the "facts" `JF-KILL` scans. So the pre-`push` point fact `st('b[0]) = Init` survives the reallocating leaf and `c.read()` is **accepted**: use-after-free. The derivation's reject silently propagates the ancestor term to `'b[0]`. **R1(ii) hazard.** | Extend `I-14` to close a *term*-valued ancestor state leafwise, and state explicitly that a conditional `ends` footprint (`ends ite(n0<c0, none, 'b)`, `I-16`/`I-17`) applies `I-4`+`I-13` leafwise. |
| F3 | fatal | `III-12` / `JF-QI` residual list at a call or a loop head; bites P1's shape the moment any call sits between the `take` and the `put` | `III-12`, `JF-QI`, `II-6`, `II-10`, `KEY` | `X` is "discarded"/"reset" at every loop head and call boundary — and that discard is what bounds `r` (§6). Under the plain reading (`X := {}`, `RSt` retained) the hole recorded at `'b[i]` is forgotten, and `JF-QI` re-instantiates `RSt('b[0..n), Init, {})` at `i`. Nothing in `KEY` orders a point `st` fact against a `∀`-derived state **at the same path** ("⊓ over `mayEq(π)`" ranges over paths, not over fact sources), so the second `take(v,i)` after an intervening call is admissible: a hole is read and an affine element is taken twice. **R1(i) + R3 hazard.** | Discard the `RSt` fact itself, not its exception list, and state in `KEY` that a point fact at π dominates any `∀`/`RSt`-derived state at π. |
| F4 | fatal | every element access (`P1`, `P4`, `P5`); `write(v[i], x)` with `i ≥ len` | `I-6`, `I-7`, `I-8`, §1.3, §8 D10 | No rule in sets I/II/III carries an index-domain premise. `I-6`'s only premises are `st(π) ≠ Gone` and the obligation clause; the default `st` of a path with no established fact is never specified. Under the `⊤` default an out-of-bounds write is accepted; under a "no fact" default the rule text still passes. Reads are refused only incidentally, because `JF-QI` supplies no `Init` outside `[0, len)` — an R1 route with an R11 diagnostic. **R11 hazard**, and R11 and R12 are absent from §8 D10's declared-unsupplied list, so this is undeclared. | Add an in-domain premise to `I-6`/`I-7`/`I-8` and the read rule, specify the default state of an unestablished path, and add R11 and R12 to D10. |
| F5 | fatal | `I-23` byte reuse; `II-16` two unpacked links; P3's parallel map | `III-6` vs `JF-LIN` schema 9 | `III-6` states a ghost index is "never emitted to the backend as a `#` fact (R4(b))" and exists only to separate paths; `JF-LIN`'s closed table lists **schema (9) ghost-index inequality** as a disjointness schema, and `JF-LIN`'s output *is* the R4 relation that R5 reads. Two paths that differ only in ghost index can be the same bytes (`I-23`'s reuse of `b2`'s extent) or the same pool slot (`II-16` binds a fresh `g` per unpack, and `II-16`'s own example says `p # p2` is *not* derivable). **R4 soundness / R5 race hazard.** | Delete schema (9) from `JF-LIN` and confine ghost indices to `KEY`'s path separation, or restrict (9) to minted tags on *simultaneously live* extents and prove that side condition. |
| F6 | fatal | **L05**, the `if active { … if stop { release(p); active = false } }` arm exit | `TERM`, `II-2`, `JF-ENT`(4) | The boundary derivation calls L05 "underivable — rule missing (atom re-keying)". It is stronger than that: `TERM` **explicitly refuses** this shape ("if a join would produce a term whose arms are themselves terms over a different atom, the join is rejected"), and the proposed repair does not exist — the exit relation is `a_exit = ite(a_h, ¬stop, false)`, a two-atom term, so `JF-ENT`'s step 4 (unit propagation, no case split) cannot record `a1 = ¬stop` outside the arm, and no re-keying rule could. L05 is therefore a **capability loss with no named repair**, while §7 records L05 as "same as the case states" and `derive…-late-branches.md` claims "no item in this part pays `TERM`'s price". This is the candidate's central O11 evidence and it is stated backwards. | Record L05 in §7 as `TERM`/`II-2`'s refusal class with no repair, and re-state the O11(b) price as "one atom per identity **and** one atom per boolean φ-relation", which L05 exceeds. |
| F7 | fatal | **P7**'s `free(oA)` generalised; any `free` of an aggregate whose content carries an obligation | `I-5`, `JF-QI` | `I-5` quantifies: "for every `π ≤ 'a` whose content type carries a release obligation, `st(π) = Uninit` or `obl(π) = none`". No family discharges a universal premise — `JF-QI` instantiates "**only** at the access path's own index (a written trigger)", and there is no access path here. So `I-5` is undecidable for any container, and the R2 obligation on its elements can never be discharged; a checker that instantiates it anyway is unsound. P7 escapes only because `'A` is a leaf. **R2 hazard (undischargeable obligation) or an unsound instantiation.** | Restate `I-5` over `RSt`-covered extents with a stated discharge rule, and name the family that checks a `∀` premise. |
| F8 | fatal | `'A # 'B` in **B04**, **B05**, B06, **L03**, P1's `par`, P5 | `I-1`, `KEY` §1.3, R4(b) | `I-1` records `'a # x` "**from the allocator's written `ensures`**, never from name inequality" — but that `ensures` must range over the *caller's* live identity set, which no clause in §1.1/§1.4's vocabulary can express (a callee cannot name the caller's identities). `I-2` records no distinctness at all. So the fact every correlated-origin case rests on has no writable source, and both derivations take the route the rules forbid (`derive…-straight-and-early-branches.md`: "'A # 'B by I-1"; `derive…-boundary.md` L03: "by `JF-PATH` on two distinct minted identities" — name inequality, verbatim what §1.3 and R4(b) refuse). **R4(b) violated undeclared** wherever the fact reaches `par`. | Add a rule minting `'a # x` at `I-1` as a checker fact with its own ground (a fresh allocation is disjoint from every live identity that does not contain it), state the `I-2` frame case, and say why this is not name inequality. |
| F9 | fatal | `III-5`'s `use INV_pool.slot_live(s)`; `III-10`'s `forall hid. live(hid) ==> …` | `M2(ii)`, `M3`, §6 | `III-5`'s premise is "`h` came from `insert` and was not passed to `remove`" — a whole-program history predicate over a **copyable** handle, checked by none of the nine families. The only implementation is the pool's runtime occupancy/free-list word tested at the `use` site, which is precisely the "generational or epoch check … wrapped as a library outcome" M2(ii) refuses by name; `III-10`'s `live(hid)` is the same predicate for the compacting pool. §6 declares M2(ii) met "conditional on owner decision O3" and names only the ghost index. **M2(ii) clause (ii) violated undeclared, hidden as library data.** | Either give `slot_live` a checked static premise (an affine handle consumed by `remove`) or declare the pool's occupancy test as a second, named M2(ii) exposure alongside O3. |
| S1 | serious | **P8.a**, `st('A) = ite(c, Gone, Live)` and then `if c { use(pb); free(ob) } else { use(pa); free(oa) }` | `I-4`, `II-3` | `I-4` touches only `π ≤ 'a`, so on the `¬c` leaf `'A` keeps its entry state `Init`; `Live` is produced by no rule here (it is copied from `II-3`'s own sloppy example). The derivation then writes "on the `¬c` leaf own('A)=held, **st('A)=Live**" and accepts `use(pa)` on that leaf — a read at `Live` must reject under R1(i). Either the term is wrong or P8's "use the survivor" requirement fails. | Write the join as `ite(c, Gone, Init)` in `II-3`'s example and in P8.a. |
| S2 | serious | **L03**, `v = take(p)` and `put(q, move v)` | `II-7` vs `II-15` row 2 | At the head, `II-3`'s origin row with no atom available gives `origin(p) = {'A,'B}`, and `II-15` row 2 refuses a take/put through an origin set (and "a write through an origin set never sets `Init`"). `II-7` asserts the opposite by worked example. The derivation notes the conflict and then records ACCEPT anyway. Two rules, opposite verdicts, no precedence clause. | Give `II-7` an explicit precedence over `II-15` row 2 for origins constrained by a written head relation, plus the rule that computes `st(origin(x))` after a statement (the derivation's own recorded gap). |
| S3 | serious | **L03**, post-loop `read(p) // ACCEPT: 10` | `II-9`, `II-7` | No rule carries a loop-head invariant to the loop exit. `II-9` **recomputes** the exit join from the edges with no atom available, which gives `origin(p) = {'A,'B}` and `st('A) = st('B) = Live`; under `II-15` row 2 the read then rejects. The accept rests on `II-7`'s example, not on a rule. | State in `II-9` that the head facts hold on the zero-iteration and normal-completion exit edges (and only there). |
| S4 | serious | **L03**, `release(p); release(q)`; also B05, B06, B10, B11 | `II-15` | `II-15`'s table has columns for `read` and for `write/take/put/free`, but its cells describe only the effect on **`st`** (and `JF-KILL` on `val`). No rule says what a release through a non-singleton origin does to `own` or `obl` — which is the entire R2 content of B05, B06, B10, B11 and L03. Row 2 would additionally refuse the L03 release outright. | Add `own` and `obl` columns to `II-15` (row 1: leafwise; row 2: refuse). |
| S5 | serious | **P7**, `old = replace(p, new)` → `read(q) // reads new` | `I-9`, `JF-KILL` | `I-9`'s Effect says `val(π)` **killed**; the derivation writes "val('A) killed **and reset to `new`**" and P7's third required property ("replace … reads new") is asserted from that added effect. No rule anywhere installs `val` (the straight-line derivation records this as G2 and rates it "low"; it is load-bearing for a P-program requirement, so it is not low). | Give `I-6`, `I-8` and `I-9` an explicit `val(π) := v` effect. |
| S6 | serious | **P4**, `RSt(backing('v)[0..n0+1), Init, {})` in `I-16`/`I-17` and both P4 arms | `III-12`, `III-8` | `III-12`'s form is `RSt(π[lo..hi), s, X)` over a **path**; `backing('v)` after a `push` is an origin *term*. No rule gives `RSt` a leafwise reading over a term, and P4's accepting arm and the else-arm's `read(v,0)` both depend on one. | State `RSt` over an origin term as leafwise, under `TERM`'s one-atom bound. |
| S7 | serious | `I-13` premise list | `I-13`, `I-21`, `I-24` | The list is "`I-4`, `I-3`, `I-11`, `I-22` or **`I-25`**" — set I has 24 rules, so `I-25` does not exist, and `I-21` (arena block free) and `I-24` (pool remove) are omitted. An owner sunk into an arena block or a pool slot by `I-12` therefore never ends when that block or slot ends: an R2 obligation left outstanding with no diagnostic (partly, not wholly, screened by `I-5`). | Replace `I-25` with `I-21` and `I-24` in `I-13`'s premises. |
| S8 | serious | every call with a measure in its contract (**P4**'s `push`/`reserve`, P1) | `II-10`, `JF-SUB`, `III-11` | `JF-SUB` substitutes *identities* through a row. No family substitutes the callee's frozen **scalars** (`n0`, `c0`, the callee's `c`) by the caller's, yet `II-10` requires binding every `old(e)` and `III-11`'s example equates the callee's `c` with "the CALLER's frozen" value in one step. This is the third meta-finding place (the identity↔runtime-value bridge) and it has no family. | Name a tenth family, or extend `JF-SUB` to scalar actuals with a stated bound. |
| S9 | serious | `let n1 = len(v)` after a kill (**P4**, P1; `III-1`'s own example) | `III-1`, `III-2`, `JF-ENT` | `III-1`'s table says a bridge fact "may appear in an atom: **no**", and `JF-ENT`'s only inputs are atoms. So no bridge fact can ever discharge an index bound or `I-16`'s `n0 < c0`. `III-1`'s own accepting line ("`JF-ENT` from `n1 = n0+1`") additionally needs measure-head **functionality** — from `len('v)=n1` and `len('v)=n0+1` conclude `n1 = n0+1` — which no rule states. | State that a live bridge fact contributes `M(π) = x` to `Γ`, and that two live bridge facts on one head are equated. |
| S10 | serious | `I-2`; `II-3`'s join table | `I-2`, `II-3` | `I-2` never initialises `obl` for a frame binding (so `let old = replace(…)` has an undefined `obl`, and `I-3`'s check is undecidable), and `II-3`'s table has rows for `st`, `own` and `origin` **only** — no row for `obl`, `val`, `layout`, `tag`, the bridge facts or an `RSt` residual list. B10, B11, B13 and P7's `old` all carry an obligation through a join. The late-branches derivation records the bridge/`RSt` half of this at P1.4; the `obl` half is unrecorded. | Add the `obl` clause to `I-2` and the missing rows to `II-3`. |
| S11 | serious | `I-2`, "`I-11` never applies to it **while a `ptr<'f>` is live**" | `I-2`, §6 | Acceptance is made to depend on pointer liveness, which none of the nine families decides, and which is the borrow-duration notion D4 exists to delete. | Restate as a premise on the move: a frame slot is never a relocating-move source. |
| S12 | serious | `III-10`'s `compact` contract; P13 | `II-11`, `JF-QI` | `JF-QI` **instantiates** a `∀` fact; no family **introduces** one. `II-11` therefore cannot check `compact`'s three-clause `∀` ensures against its body, so P13's accept is asserted. (Same shape as F7 on the premise side.) | Name the family that discharges a `∀` conclusion in a callee body, or declare P13 unsupported alongside D6. |
| S13 | serious | **B08**/B09, the second `if cond` | `III-1`, `III-7`, §1.1 | B08 turns entirely on the second guard carrying the **same** atom. `III-1`'s `FREEZE` is defined only for a *measure head* and "binds a **fresh** frozen value" per application; `III-7` says a guard condition is "established by … `FREEZE`" and "re-derived by nothing", while §1.1 says an SSA scalar binding is itself frozen. Under the `FREEZE` reading B08 rejects; under the SSA reading it accepts. Both derivations take the SSA reading without saying so. | State that a guard atom is over the SSA value of the guard expression, and that `FREEZE` applies to measure heads only. |
| S14 | serious | every `read` line in every derivation | set I | There is no numbered rule for `read` — its premise is assembled from `KEY`, R1 and the accepting/rejecting examples of `I-1`, `I-4`, `I-7`, `II-5`, `II-15`. Every R1(i)/R1(ii) verdict in this round rests on it. (Self-reported as G1 at severity "low"; a requirement-bearing operation with no rule is not low.) | Add `I-0 read`, with the in-domain premise of F4. |
| S15 | serious | **P4**, the `reserve` route | `I-17` vs `III-3`; §7 | The boundary derivation records that `I-17`'s example rejects the post-`reserve; push` read while `III-3`'s example accepts the identical line. §7's cost table still cites the `III-3` route as the repair of cost S1 ("`cap` IS in the vocabulary … cost S1 repaired"), so §7 is on the losing side of the candidate's own conflict, and the repair the cost table claims does not exist. | Delete `III-3`'s accepting line, and re-price cost S1 in §7 as "unrepaired: the `cap` proof must precede the `reserve`, which is vacuous". |
| m1 | minor | **B08**, `read(p)` closing comment | `II-5` | "`JF-KILL` is leafwise under `II-5`" — `II-5` governs leafwise **evaluation** of facts sharing an atom; no rule makes a kill leafwise. | Drop the clause or add a leafwise-kill sentence to `II-5`. |
| m2 | minor | **B04**, "per-leaf `mayEq` is a singleton" | `KEY` | `KEY` defines `mayEq` over live paths at a point, not per leaf of a term; `II-15` row 1 makes only the *state effect* leafwise. | Say "`II-15` row 1 resolves the origin first, then `KEY` applies at the resolved path". |
| m3 | minor | **L05**'s invariant, `==>` | §1.1, M8 | The implication connective appears in `II-8`'s worked example and in the derivation but in neither §1.1's atom grammar nor M8's ≈46-construct count. | Either spell the invariant as `st('A) = ite(active, Init, Gone)` throughout, or add `==>` to §1.1 and to the M8 count. |
| m4 | minor | §7, row "`P3` `use inv_links`" and D5 | §7, §8 | §7 presents P3's traversal repair and D5 presents O3 as the single open condition, but F5's schema-9 conflict means P3's *parallel* map is exposed independently of O3. | Add the `JF-LIN` schema-9 conflict to D5's statement. |

---

## 2. Hazards admitted, by requirement

| Requirement | Admitted by | Finding |
|---|---|---|
| R1(i) no read of storage holding no value | `III-12`/`JF-QI` residual discard with no point-fact precedence | F3 |
| R1(ii) no access to ended storage | `I-6`/`I-4` premises satisfied by `⊤`; term-valued `ends` not closed downward | F1, F2 |
| R2 released exactly once on every path | `I-5`'s undischargeable `∀` premise; `I-13`'s premise list omitting `I-21`/`I-24` | F7, S7 |
| R3 affine value moved at most once | double `take` at one symbolic index across a call boundary | F3 |
| R4(b) distinctness never from name inequality | `JF-LIN` schema 9; both derivations' `'A # 'B` route | F5, F8 |
| R5 no data race | schema 9 feeds R4's relation, which R5 reads exactly | F5 |
| R11 partial operations inside their domain | no rule; unspecified default state of an unestablished path | F4 |
| R12 typed outcomes | no rule at all; §7 and P1's accepted shape write `else { outcome }` with no join rule for R2 on the arm | F4 |
| M2(ii) no bookkeeping state the source did not bind | `III-5`'s `slot_live` premise, `III-10`'s `live(hid)` | F9 |

R11 and R12 are the two requirements with **no rule and no declaration**: §8 D10
lists R5(ii)(iii), R8b, R13 and R14 as unsupplied and omits both.

---

## 3. Verdicts overturned

| Item | Derivation's verdict | This critique | Ground |
|---|---|---|---|
| **P4** | "accepted with a stated cost" | **not derivable as written; the rules admit the hazard** — `b = c.read()` after a reallocating `push` is *accepted*, because `I-14` closes only a constant `Gone` and `st` is not a `JF-KILL` fact | F2 (with S6, S9, S15) |
| **P7** | "accepted with a stated cost"; `read(q)` after `replace` accepts and "reads `new`" | accept of the **access** stands; the *contents* claim is overturned — `I-9` kills `val` and no rule installs it, so P7's third required property is not derivable | S5 |
| **P8.a** | `st('A) = ite(c, Gone, Live)`; `if c { use(pb); free(ob) } else { use(pa); free(oa) }` accepts | the term is wrong (`ite(c, Gone, Init)`); as written the derivation accepts a read at `Live`, which R1(i) refuses | S1 |
| **B04** | accepted as expected | stands, **conditional on F8**: the `'A # 'B` it cites has no writable source | F8 |
| **B05** | accepted as expected | stands for `st`; the `own`/`obl` half of every line is unruled (`II-15` has no such column), and `'A # 'B` is F8 | S4, F8 |
| **B08** | accepted as expected | stands **only under the SSA reading** of a guard's frozen value; under `III-1`'s `FREEZE` reading the second `if cond` is a fresh atom and B08 rejects | S13 |
| **L03** | "accepted with a stated cost" (one written invariant) | **underivable, and rejected on the plain reading**: the body take/put is refused by `II-15` row 2, the post-loop `read(p)` is refused by `II-9`'s recomputed join, and the two releases have no effect rule | S2, S3, S4 |
| **L05** | "underivable: rule missing (3 gaps)" | **refused by `TERM`/`II-2`, a capability loss with no named repair** — not a missing rule: the boolean φ-relation `a_exit = ite(a_h, ¬stop, false)` needs two atoms, so no re-keying rule can exist | F6 |

`CASES.md` classification consequences the candidate must state and does not:
L05 moves from "same" to **capability loss** (§7's row and the O11 evidence
paragraph both assert the opposite); L03 moves from "writer cost" to
**underivable pending S2–S4**; P4's first required property is **admitted
unsoundly** rather than met.

---

## 4. What this does not dispute

The derivations are unusually honest and several of these findings extend a gap
they recorded themselves: P4's `where`-clause gap and the `I-17`/`III-3` conflict,
P1.4's bridge-fact join, L03's relational-update gap, L05's three gaps, and
G1–G4. `KEY` genuinely repairs soundness F1 (the double `take` at a symbolic
index) **within one declaration**; F3 is about what happens across a call or a
loop head, which `KEY` does not reach. `I-13`'s enumerated owning edges genuinely
repair S6 for the container case; S7 is about the two ending rules the premise
list forgot. P7 and P8's loop are derivable as claimed, and P8's "no drop flag"
holds: `II-3`'s terms are erased and every branch is the writer's.


# File: critique-brand-context-bounded-cost.md

# Critique: candidate `brand-context-bounded`, lens **COST AND DETERMINISM**

Target: [rules-brand-context-bounded.md](rules-brand-context-bounded.md) (1329
lines: §1 the model, §2 `ATOM`/`TERM`/`KEY`, §3 rule set I, §4 rule set II, §5
rule set III, §6 the nine judgment families and the M-declarations, §7 the
`CASES.md` diff, §8 open defects) and its three derivations
([straight-and-early-branches](derive-brand-context-bounded-straight-and-early-branches.md),
[late-branches](derive-brand-context-bounded-late-branches.md),
[boundary](derive-brand-context-bounded-boundary.md)).

What this lens checked, in order: (1) every judgment family for a fixed,
terminating, search-free procedure (M1) with a stated degree (M10); (2) the
condition-atom bound — this candidate's headline mandatory change, `TERM`'s
one-atom canonical form and `ATOM`'s frozen-operand rule; (3) the count of
constructs the rules introduce (M8); (4) what the writer must write per derived
item, tabulated; (5) M7 — is every rejection in the derivations locally
repairable from the diagnostic the rules would emit.

Nothing here selects, ranks or repairs a candidate. Severity per the round's
definitions: **fatal** = a hazard admitted or a requirement violated
undeclared; **serious** = a wrong verdict, an understated cost, an unnamed
family, a missing rule; **minor** = wording.

The candidate is unusually forthcoming about its own costs — §6.1 declares M10's
written-step clause violated, §6.1 declares M8 red at ≈46, §8 lists ten open
defects, and the three derivations themselves record fourteen rule gaps. That
honesty is the reason this critique can be specific. Every finding below is a
place where the declared position is *still* short of what the rules as written
cost.

---

## 1. Findings

### 1.1 Fatal

| # | Item / line / rule | Finding | Fix |
|---|---|---|---|
| **F1** | rules §6 measure list, line 1181; §6 `JF-OVL`/`JF-ENT`/`JF-KILL` rows, lines 1191, 1192, 1195; §6.1 rollup lines 1202–1210; §6.1 "M10(a), first clause — met", line 1215 | **The three cost measures `n`, `I` and `F` are not functions of program size, and `n` in particular is `Θ(S)` by the candidate's own `ATOM` rule, so every degree stated "in its own measure" is at least quintic in program size and M10(a)'s *first* clause is violated undeclared.** M10(a) requires the bound "in program size including instantiations" (SYS-D0-15). `n` is defined as "frozen scalars in the guard context". `ATOM` (line 103) makes a frozen value *immortal* — "killed by nothing", "re-derived by never needed" — and §1.1 admits four producers: every SSA scalar, every `old(e)` bound at a call (`II-10`), every loop-head φ-value (`II-6`), **and every ghost index** (`I-20`, `I-23`, `I-24`, `III-6`). No rule anywhere removes a frozen scalar from `JF-ENT`'s digraph. A declaration with `S` statements therefore carries `n = Θ(S)` scalars, so `JF-ENT` is `Θ(S³)` per closure, `JF-OVL` is `Θ(S³·I·d)` per symbolic access, and the rollup `W = O(S·I·d·n³ + S²·n³)` is `Θ(S⁴·I·d + S⁵)`. `I` and `F` are likewise never bounded by a syntactic quantity: `I-1` mints an identity per allocation, `I-20` per carve, `I-24` per insert, `I-11` per relocating move, and `II-16` per `unpack`. §6.1 declares only the *second* clause (written proof steps) violated and asserts the first clause met. | State `n`, `I` and `F` as functions of program size (frozen-scalar mint sites, identity mint sites, facts-added-per-rule × statements, instantiations included), then re-evaluate M10(a)'s first clause against program size rather than against the measure; if the composed exponent is above cubic, declare the first clause violated in §6.1 beside the second. |
| **F2** | rules §II-6, lines 710–739; §II-3 table, lines 647–655; §II-8, line 758; §6's family table (no row); derivation boundary L02 line 380, L05 lines 586 and 599 | **The loop head is a genuine fixpoint that no family owns, with no stated iteration procedure, no termination argument and no degree — and the join it is defined in terms of is not total, so the "lattice" M1(a) demands does not exist.** `II-6` computes the head state as "the `II-3` join of `E` and `B`", but `B`, the back-edge state, is the *output* of checking the body under the head state. The document never says how many times the body is checked, in what order, or what the head is initialized to; every derivation quietly assumes the answer (`L01`: "back edge Init (established below)"; `L04`: "the ONLY back edge is the fall-through path below"). M1(a) is explicit: "a fixpoint terminates by lattice height, never by an iteration cap", and §6.1 line 1211 declares M1 met. Worse, `II-3`'s table is constant-vs-constant in every row: the candidate's own boundary derivation finds it **has no row for joining a constant with a term** (L05 gap 3, line 599) and **no row for `obl`, `layout`, `owns`, a bridge fact or an `RSt` residual list** (P1.4, late-branches line 569). A partial join is not a lattice, so neither the height argument nor the convergence argument is available. Origins compound it: `II-3`'s last row joins to a *set* `{π₁…πₙ}` whose height is `I`, not 2. | Name the loop head as a family: state the initial head assignment, the body re-check order, the termination measure (lattice height over each fact head, with the origin-set height stated as `I`), and the degree as body-rechecks × per-statement cost; and complete `II-3`'s table with a constant-vs-term row and rows for every fact head in §1.3 before `II-6` is defined in terms of it. |
| **F3** | rules §6.1 "M3 — met", line 1237; §III-5 example lines 1034–1038; §III-12; derivation late-branches P3 line 628 | **`use R(args)`'s premise checking is named by no family, has no procedure, no termination argument and no degree, and one of its premise forms is a whole-declaration history analysis.** §6.1 rests M3 entirely on `use R(args)` being "one application of a **named rule with premises the checker checks**", and §6.1's M7 row makes "`use` a named rule" one of four taught routes. `III-5`'s own worked example states the premise as *"`h` came from `insert` and was not passed to `remove`"* — that is a reaching-definitions plus may-alias query over the whole declaration, not an entailment over `Γ`, and no family in §6 takes a program point and a binding as input. `P3`'s `use INV_pool.links_live(s_a, s_n, s_b)` needs `JF-QI` at three symbolic indices plus a match of the written rule's conclusion against three residual exceptions; §6 gives `JF-QI` one instantiation "at the access path's own index", which is not this. §6.1 declares M3 met and M10(a) met on the strength of a procedure it does not state. | Add a `JF-USE` family with inputs (the named rule, the actual arguments, `Γ`, the fact table), a procedure (argument matching, premise dispatch to `JF-ENT`/`JF-QI`/`JF-OVL`, conclusion installation), a termination argument, and a degree — and either give the history premise form (`"came from insert"`) its own family with a degree, or delete it from `III-5` and restate the premise over facts. |
| **F4** | rules §6.1 "M7 — met", line 1251; derivation boundary P4 lines 101 and 145–155, L02 line 386, L05 lines 573–624; §7 B13 row and L04 row | **M7 is declared met, and at least four rejection classes in the candidate's own derivations have either no local repair or a named repair the rules cannot check.** (a) **P4's reallocating arm**: after `push` with `n0 ≥ c0` the cursor's `'b` is `Gone`; the boundary derivation records that "no rule rebuilds a stored nominal at the new backing inside the arm" (line 101) and that the natural repair, `reserve`, "is unusable" because `I-17` leaves its own conditional `ends 'b` on the cursor's identity (line 153). The rejection has no repair. (b) **`I-3`'s non-constant-`own` class** (B13, L04's enclosing scope, P8's omitted else): §7 names the repair "write the guarded release", which for the loop shape is exactly `L05` — and the boundary derivation finds **L05 underivable**, with three missing rules (line 624). The candidate's largest refusal class names a repair its own rules cannot check. (c) **`II-2`'s arm-exit refusal in L05** (line 573): the term carries `stop`, the invariant carries `a1`, `JF-ENT` records `a1 = ¬stop`, and **no rule re-keys a term's atom along an entailed equivalence** — the repair would be to rewrite the program so the inner divergence never crosses the arm, which the case's control flow does not permit. (d) **L02's `n ≤ 1` sub-case** (line 386): `II-6`'s two named repairs are "restore before the back edge" (a different program) and "write `invariant st('A) = Live`" — and the rules' own `II-6` example admits "then the take rejects", i.e. the named repair reintroduces a rejection, which M7(a) forbids by name. | Add an M7 row to §6.1 declaring it violated and listing the four classes; where a class is the model's boundary (P4's reallocating arm), say the boundary is the reason rather than naming a repair; and either supply the atom re-keying rule L05 needs or withdraw "write the guarded release" as the repair for the loop shape of the `I-3` class. |

### 1.2 Serious

| # | Item / line / rule | Finding | Fix |
|---|---|---|---|
| **S1** | rules §6 `JF-KILL` row, line 1195; §6.1 rollup line 1209; §6 `JF-ORG` row line 1197 | **Understated degree: `JF-KILL` is stated `O(F_base·d)` per write although its own procedure says it "test[s] `JF-OVL`" on every fact in the bucket, and `JF-OVL` is `O(I·d·n³)`.** The true degree is `O(F_base·I·d·n³)` per write and `O(S·F·I·d·n³)` per declaration, not the `O(S·F·d)` the rollup carries. `JF-KILL` runs at *every* write in the language, so this is the highest-frequency procedure in the candidate and the one whose stated degree omits the most. `JF-ORG` has the same shape: `O(|O|·n³)` counts the `JF-TERM` evaluations and omits the per-leaf `KEY` resolution each leaf then performs. | Restate each dependent family's degree as the product of its query count and the called family's degree, and recompute `W`. |
| **S2** | rules §6 `JF-ENT` row, line 1192 (`O(n³)` "computed once per context") vs `JF-OVL` row, line 1191 (`O(I·d·n³)` "per symbolic access"); §6.1 rollup lines 1204–1205 | **The same entailment closure is charged once per guard context in one row and again per access in another, so `W` is not a bound under either reading.** If the closure is amortized per context, `JF-OVL`'s per-access `n³` is wrong (queries are digraph lookups) and `W`'s second term collapses; if it is per query, `JF-ENT`'s "computed once per context" is wrong and `W`'s first term is a subset of its second. The document also never says what invalidates a closure: `III-7` installs `c1 = ¬c0` mid-context, `I-17` installs `cap('v) ≥ m` from an `ensures` mid-context, `III-4` installs `h1 = h0+64` mid-context — each requires a re-closure at a statement that is not a guard. | Pick one accounting, state the re-closure trigger (any statement that installs an atom), and recompute `W` with the closure count as a stated function of `S`. |
| **S3** | rules §I-1, lines 196–202; derivations straight §S01–S07, B01–B06 (every item), late-branches B10/B11, boundary P8 | **Load-bearing rule that cannot fire as written, with an uncounted quadratic fact growth behind it.** `I-1`'s effect is: "for every live identity `x` that is not an ancestor of `'a`, record `'a # x` **from the allocator's written `ensures`**, never from name inequality." An allocator's `ensures` is a fixed clause in a fixed signature; it cannot quantify over the *caller's* live identity set, which is what the rule asks it to supply. Every derivation in the straight-and-early-branches part uses `'A # 'B` "by `I-1` from the allocator's `ensures`" — thirteen items rest on it. Either the fact is not derivable (and `KEY`'s `mayEq` resolution, `I-7`'s strong updates and `II-5`'s correlations in S01–S07 and B01–B06 all fail), or it is derived from mint freshness, which `R4(b)` and §7's P2 row forbid by name. The cost side: as written the rule records `O(I)` distinctness facts per allocation, i.e. `O(S·I)` fact growth feeding the undefined measure `F` (F1), owned by no family and counted in no degree. | Either state that two *separately minted* identities are distinct as a minting property of `I-1`/`I-2`/`I-20`/`I-24` — and then say explicitly why that is not the name-inequality `R4(b)` refuses — or drop the clause and make `JF-OVL`'s same-base restriction the stated rule; in both cases state how `#` is represented so `O(I)` facts per mint are not materialized. |
| **S4** | rules §1.4 table, lines 84–90; derivations late-branches P3 line 628, P1.4 line 569; boundary P4 lines 145–155 | **Understated writer cost: §1.4's cost model is contradicted by the candidate's own derivations at three of its five rows.** (i) The statement row claims zero "when … **P3's writes**" — the P3 derivation writes `use INV_pool.links_live(s_a, s_n, s_b)` at exactly those writes (line 628), and since `III-12` resets `X` at every loop head and call boundary, a traversal *loop* pays that `use` **per iteration** plus a written head invariant restating the ∀-fact with `X = {}`. (ii) The join row claims "nothing, always" — the P1 derivation finds no rule joins a bridge fact or an `RSt` residual list at an `if`-join (line 569), and the only conservative reading it can name is "drop the bridge fact, re-`FREEZE` after the join", i.e. **one written `let` per measure per join**. (iii) The signature row omits the `O(f²)` `#`-clause count that §7's own O7/O12 evidence paragraph (line 1305) charges to any interface producing an aggregate of separately-named storages. | Add a "restructuring" and a "re-`FREEZE`" row to §1.4, correct the P3 entry in the statement row, and cross-reference §7's `O(f²)` clause count into the signature row; §2 below tabulates the measurement. |
| **S5** | rules §7 table, "L01 L02 L04 L05 L06 — as the case states. same"; derivation boundary "Verdict L05: underivable — rule missing", line 624 | **Wrong verdict in §7: L05 is listed as "same" although the candidate's own boundary derivation finds it underivable under the rules as written.** Three rules are missing (line 624): no atom re-keying from the inner guard `stop` to the invariant's boolean `a1` under the recorded `a1 = ¬stop`, so `II-2` rejects the `if active` arm exit; `II-8`'s back-edge check is unstated for two terms over different atoms; `II-3` has no constant-vs-term row for the exit join. `II-8`'s worked example (line 770) nonetheless marks the identical program `accept`, so the document contradicts itself on the item. This matters beyond L05: L05 is the named repair for the `I-3` refusal class (F4(b)). | Restate the §7 row as "intended accept, not derivable; three rules missing", supply the three rules, and re-mark `II-8`'s example as pending on them. |
| **S6** | rules §6.1 M8 row, lines 1242–1250 ("≈46 constructs") | **Understated construct count: at least eleven taught constructs the rules and derivations actually use are not in the inventory.** Not counted: `par` and `par forall` (used at `II-13`'s example, `P1`'s parallel writes, `P3`'s parallel map — 2); `move` at every transfer (`put(q, move v)` — 1); `own`'s values `held`/`discharged`, which §1.3 holds and every `I-3`/`I-4` premise names (2); `obl`'s values `none`/`one` (2); the measure head `size`, which `I-20`'s `INV_A` uses and which §1.4's signature vocabulary omits although `III-1` lists it among the seven measure heads (1); the origin-set former `'x\|'y`, which §1.1 says "appears in a signature" (1); `RSt`'s written exception-list argument `X` (1); the projection vocabulary `subrange`/`open`/`close` (`II-13` — ≥1). Recount ≥ **57**, not ≈46, before M8 is handed to owner decision O6 — and the gap matters because §8's D9 already declares the count red against an unset `K`. | Recount the taught surface over the whole document, including every value of every fact head and every construct the examples write, before the number goes to O6. |
| **S7** | rules §6.1 "M10, edit stability — met", line 1224; §I-19, lines 492–506; derivation boundary P4.a | **M10's edit-stability clause is declared met on a reach claim the candidate's own `I-19` breaks.** The claim is "a body edit changes the verdict of one declaration; an interface edit reaches its callers and no further". `I-19` makes a nominal's identity row *closed*, so adding a pointer field to a struct changes the struct's arity — `Cursor<'v>` becomes `Cursor<'v,'b>` (P4.a) — and that reaches every declaration that *mentions* the nominal, every nominal that has a field of that type, and transitively upward, not "its callers". The one-token edit `{ at: ptr<'v,Vec<'b>>, i: u64 }` → adding `prev: ptr<'p,Node>` is a whole-program signature cascade. | Restate the reach function for a nominal-row edit as the transitive closure of declarations mentioning the nominal, and declare the edit-stability clause violated for that edit class or state the function that bounds it. |
| **S8** | rules §6 `JF-LIN` row, line 1193; §I-20 example lines 519–526; §III-4 example lines 1013–1020; §7 P2 row | **§7 asserts P2's arena block distinctness is derivable "by `JF-LIN` over frozen bounds", and for a symbolic block size no schema in the closed table matches.** Both worked examples use a *literal* 64. With `alloc(A, m)` for symbolic `m`, `I-20` mints `[h0, h0+m)` and `[h1, h1+m')` with `h1 = h0+m`, and the disjointness goal `h0+m ≤ h1` is a three-variable relation outside `JF-ENT`'s difference-bound fragment; step (5)'s syntactic residue match against `Γ` succeeds only if the writer spelled the goal exactly. None of schemas (1)–(9) is stated to cover it — (2) is `a+c₁ ≤ a+c₂` with literal constants, (6) "prefix/suffix split" is unspecified. §8's D3 admits the table "has no adequacy argument", but §7 still claims the P2 row, and **P2 is derived in none of the three derivation files**. | Either add a schema for `[h, h+m) # [h+m, h+m+m')` with its side conditions, or restate the §7 P2 row as "derivable for literal sizes; a written branch or `INV` step otherwise", and derive P2 before claiming it. |
| **S9** | rules §I-4 lines 244–250, §I-14 lines 391–398, §KEY lines 168–176; §6 measure `F` | **The fact representation for `st` over the path lattice is unstated, so `F` is undefined and `KEY`'s read has no rule for a path carrying no fact.** `I-4` says "for every `π ≤ 'a`: `st(π) := Gone`" over a path language that includes `π[i]` at symbolic `i` and `π@g` — an infinite set. `I-14` says asserting `Gone` on an ancestor closes downward but asserting `Init` implies nothing about descendants, so the lookup for `st(π)` when no fact is keyed at `π` is: search ancestors for `Gone`, and otherwise *unspecified*. Every derivation leans on the root-keyed reading ("the fact is on `'A`", `I-4`'s own wording) while the rule text is per-path. | State the representation (root-keyed facts plus the `I-14` ancestor rule, with an explicit "no fact at `π` and no `Gone` ancestor" case), and define `F` against it. |
| **S10** | rules §KEY, line 168; §6 `JF-OVL` row, line 1191; derivation late-branches P1.3 line 500 | **`mayEq`'s "live path set" is undefined, and strong-versus-weak update — hence acceptance — turns on it.** `KEY` says `mayEq(π)` ranges over "every **live** path `π'` with the same base and the same shape". No rule says when a path enters or leaves that set. The P1 derivation asserts `mayEq('b[i]) = {'b[i]}` at the first `take` because "no other symbolic element path is live" and then `mayEq('b[j]) ∋ 'b[i]` at the second — the difference is whether the checker retained `'b[i]` after `qi` was bound, which no rule states. `I` is also the size of this set and is `JF-OVL`'s cost measure (F1). | State the live-path set as a rule: a path enters when a fact is keyed at it or a binding's origin names it, and leaves when every such fact is killed and no binding names it; then `I` is a stated function of the fact table. |
| **S11** | rules §6 `JF-ENT` step (5), line 1192; §6 `JF-TERM` "equality: atoms compared by `JF-PATH`-style normalization", line 1196 | **No normal form for atoms is stated, so the residue match and term equality are spelling-dependent and acceptance depends on how the writer wrote the guard.** Step (1) normalizes to `Σcᵢxᵢ + c ⋈ 0` without fixing the ordering of the `xᵢ` or the direction of `⋈`; two arms that establish `n0 < c0` and `c0 > n0` need not compare equal, and `JF-TERM`'s join (`II-3` row 2, "the atom is available iff it is `C`'s own guard atom") compares atoms for identity. `JF-PATH` is defined for *paths*, so "`JF-PATH`-style normalization" for atoms names nothing. M1(a) forbids "state outside the specification" selecting acceptance; a spelling is source, but two spellings of one atom producing two verdicts at a join is a defect the row asks to be closed. | State the atom normal form: a canonical scalar ordering (mint order is a function of the source), a canonical relation direction, and constant folding; make `JF-TERM`'s equality equality of normal forms. |
| **S12** | rules §II-6 line 710, §I-20 line 508, §I-24 line 576 | **Missing rule with a cost consequence: no rule states how an identity minted on the back edge joins at the loop head.** The arena's canonical shape is a carve inside a loop and the pool's is an insert inside a loop; each mints a fresh identity with a fresh ghost index per iteration (`I-20`, `I-23`, `I-24`). `II-6` requires each identity's head state to be a constant or a stated φ-term, but an identity that does not exist on the entry edge has no head join at all, and `III-6`'s ghost index is a frozen value that is fresh **per execution of the mint**, not per mint site. Neither the arena nor the pool loop is derived anywhere. | State the loop-head rule for an identity minted in the body (it leaves scope at the back edge unless a written invariant names it; a ghost-indexed identity carried across iterations needs a head φ-ghost), and derive one arena loop. |
| **S13** | rules §III-3 example, lines 997–1004, vs §I-17 example, lines 464–472; derivation boundary P4 line 118 | **The document gives two verdicts for one line, and which example governs decides whether P4 has a repair at all.** `I-17`'s example marks `read(pe)` after `reserve(v, n0+1); push(v,5)` **reject** ("the cursor's cached `'b` is `ite(c0>=n0+1, Live, Gone)` from the RESERVE, not the push"); `III-3`'s example marks the identical line **accept** ("cost S1 repaired"). §7 and §8 both cite the `reserve` route as the repaired `cap` path. M1(a) requires acceptance to be a function of the specification; two rules of one specification disagreeing on one line is exactly the hole. | Delete or correct `III-3`'s example, and restate §7's P4 "repair order" row: under `I-17` the `reserve` route does not repair the stored cursor at all, because `I-17` leaves its own conditional `ends 'b` on the cursor's identity. |
| **S14** | rules §II-2, lines 626–642 | **`II-2`'s second alternative is vacuous, so the rule admits only constants at an arm exit — which is the whole of `TERM`'s price and is larger than §7 prices it.** `II-2` admits an exit state that is "a **constant** or a term over an atom already in the *enclosing* `Γ`". But `II-1` (line 608) evaluates **every** term in scope under the new `Γ` on arm entry, and a term whose atom `Γ` entails or refutes collapses to a constant. A term over an atom already in the enclosing `Γ` therefore cannot exist at the arm exit — it was collapsed on entry. So `II-2` in effect requires a constant, i.e. **every divergence nested inside any arm is rejected**, which is exactly what bites L05 (F4(c), S5). §7 says the price is "recorded which `CASES.md`/`PROGRAMS.md` lines pay for it" and then records none; the straight-and-early-branches derivation confirms "`TERM`'s one-atom bound is never approached in this part" and points at B12, whose derivation does not nest either. | Delete the vacuous alternative (or state the case in which a non-collapsed enclosing-atom term reaches an arm exit), and list L05 in §7 as the item that pays `TERM`'s price. |
| **S15** | rules §II-1 line 608, §II-2 line 626; §6.1 rollup lines 1202–1210 | **Unnamed procedure at the highest frequency in the candidate: `II-1`'s re-evaluation of every term in scope at every arm entry, and `II-2`'s scan of every identity at every arm exit, appear in no family and in no term of `W`.** Both are `O(I)` `JF-TERM` queries per arm, i.e. `O(S·I·n³)` per declaration under the per-query reading of S2. `II-1` is the mechanism the candidate's guard-context restriction is built on, so this is the cost of its central claim. | Add the arm-entry re-evaluation and the arm-exit scan to §6's table with their degrees, and to `W`. |
| **S16** | rules §III-7 line 1064, §I-17 line 451, §I-20 line 508, §II-10 line 798; §6 measure `n` "frozen scalars in the guard context", line 1182 | **Unnamed procedure: the installation of non-guard atoms into `Γ`, and the measure that is supposed to count them excludes them.** `III-7` records `c1 = ¬c0` on a boolean reassignment; `I-16`/`I-17` record `len('v) = n0+1` and `cap('v) ≥ m` "from the `ensures`"; `III-4` records `h1 = h0+64` at a carve; `II-10` binds every `old(e)` to a frozen value; `I-1` records `#` facts. The procedure that extracts atoms from a callee's `ensures`, decides which are installed, and scopes them is stated nowhere, yet `JF-ENT`'s input is "`Γ` (guard atoms + `requires` + `INV` conclusions)" and its measure is "frozen scalars **in the guard context**" — a phrase that excludes every atom just listed. B09, B12.2, III-3's `cap` chain and III-4's arena disjointness are all carried by atoms the measure does not count. | Name the atom-installation family with its inputs and degree, and redefine `n` as all frozen scalars reachable at the point, not the guard-context subset. |
| **S17** | derivation straight G2, line 542; rules §I-9 example line 328, §III-12, §6 `JF-KILL` | **Missing rule: nothing installs `val`, although `JF-KILL` kills it, `I-9` kills and re-asserts it, and `R6`'s frame claims are stated over it.** `I-9`'s example asserts "reads `new`. `val` is keyed on `'A`, so the read through `q` sees it: no must-alias analysis (`R6`)" — the candidate's whole R6 answer — from a fact no rule produces. Cost consequence: `F`, the measure of `JF-KILL`, has an unbounded producer that is not written down, and `III-2`'s prized "element writes do not disturb `len`" tax removal (line 972) is a claim about a fact family with no introduction rule. | Give `write`, `put` and `replace` a `val(π) := v` effect with its support set, and state `val`'s read rule (the `KEY` `⊓` does not apply to a value). |

### 1.3 Minor

| # | Item / line | Finding | Fix |
|---|---|---|---|
| **m1** | rules §TERM "Guard-context restriction", lines 137–144 | "A term therefore only survives to a point where its atom is **independent of every enclosing guard** — which is what … buys: accumulation inside nested arms is impossible, because entering an arm strictly shrinks terms." The conclusion does not follow from the premise: an independent atom's term does *not* shrink on arm entry, and a new divergence inside the arm creates a new term, which is accumulation. What actually prevents accumulation is `II-2`'s **rejection**, not shrinking. The distinction matters because §6.1 withdraws the base's `2^d` amendment on this sentence. | Reword: accumulation is prevented by `II-2`'s refusal; `II-1`'s collapse is what makes the refusal rare rather than what makes accumulation impossible. |
| **m2** | rules §KEY line 168 vs §6 `JF-OVL` row, line 1191 | `KEY` defines `mayEq` as the paths "not proved distinct by `JF-OVL`+`JF-ENT`+`JF-LIN`", and `JF-OVL` is defined as "containment, may-overlap, and `KEY`'s may-equal class". The definition is circular as written. | Stratify: `JF-OVL` decides overlap between two paths; `mayEq` is the derived set. |
| **m3** | rules §6 measure list, line 1183; §6.1 M10 second clause, line 1219 | `q` (written proof steps) and `|Ρ|` are declared measures, and `q` appears in no family's degree — only in the sentence declaring M10's second clause violated. | Either drop `q` from the measure list or state which family it measures. |
| **m4** | rules §KEY example line 182, §I-8 example line 315, §III-5 example line 1042 | Diagnostics print sets (`mayEq('b[j]) = {'b[j], 'b[i]}`, exception lists `{i, j}`) with no stated ordering, so byte-stable diagnostics (M7(a), M1(e)) are not established. | State the print order (mint order for identities, source order for exceptions). |
| **m5** | rules §1.4 loop-head row, line 87 ("Zero when … P5, P6 …"); statement row, line 90 ("Zero when P2, P3's writes, P5, P7") | Four of the six programs cited as zero-cost (P2, P5, P6, P7) are derived in none of the three derivation files for this candidate, and the fifth (P3) is contradicted by its own derivation (S4). | Cite only items the derivations cover, or derive P2/P5/P6. |
| **m6** | rules §6.1 M2(ii) row, line 1231; §8 D5 | The M2(ii) declaration is "met, *conditional on owner decision O3*" and D5 restates it. This is correctly declared; noted here only because the conditional covers `I-23`, `I-24` and `III-6`, i.e. every reuse item in P2, P3 and P13, so a reader who scores the candidate on those programs is scoring an undecided premise. | No fix; keep the conditional visible in any per-program scoring. |

---

## 2. What the writer writes, per derived item

Counted from the three derivations. **A** = a written annotation, declaration,
identity parameter, `where` clause or `FREEZE` `let`. **I** = a written loop
invariant (conjunct count in the note). **U** = a written `use` step. **B** = a
written branch whose only purpose is to carry a proof fact. **R** = a
restructuring: a change to the program's data model or control flow.

| Item | A | I | U | B | R | Note |
|---|---|---|---|---|---|---|
| S01–S07 | 0 | 0 | 0 | 0 | 0 | accept as expected; `val` unsupported (S17) |
| B01–B06 | 0 | 0 | 0 | 0 | 0 | accept; carried entirely by `II-5` on one atom |
| B07, B08, B09 | 0 | 0 | 0 | 0 | 0 | rejections and variants as the cases state |
| B10, B11 | 0 | 0 | 0 | 0 | 0 | accept; `II-3` row 5 term + `II-15` row 1 |
| B12 | 0 | 0 | 0 | 0 | 0 | the `if c / if !c` shape is the case's own program |
| **B13** | 0 | 0 | 0 | **1** | 0 | a written guarded release; deliberate refusal, §7 |
| L01, L06 | 0 | 0 | 0 | 0 | 0 | zero written steps; `L06` needs §1.2's `Live` |
| **L02** | — | — | — | — | — | reject as expected for unrestricted `n`; the case's `n ≤ 1` sub-case is **inexpressible** (no loop-counter φ-value rule) and rejects too — an unlisted capability loss |
| **L03** | 0 | **1** (3 conjuncts) | 0 | 0 | 0 | §7 classes it writer cost; the relational update rule is missing |
| **L04** | — | — | — | — | — | reject as expected; the enclosing scope exit also rejects and its named repair is L05 |
| **L05** | 0 | **1** (2 implications) | 0 | 0 | ? | **underivable**: three missing rules (S5, F4(c)) |
| **P1** | **1** (`let n0 = len(v)`) | 0 | 0 | **2** (5 index atoms) | **1** (a written outcome arm, R12) | plus **1 re-`FREEZE` per join** under P1.4's conservative reading (S4(ii)) |
| **P3** | 0 | **1** (the pool's ∀-invariant + free list as written data) **+1 per traversal loop head** (`X` resets, `III-12`) | **1 per group of symbolic link writes, per iteration** | **1 per traversal read** (`if s != s_m`) | 0 | plus a written distinctness invariant or branch per parallel traversal |
| **P4** | **5** (2 identity params + 1 `where` + 2 `FREEZE` lets) | 0 | 0 | **1** (`if n0 < c0`, non-empty else) | **unrepairable** | the else arm cannot use the cursor and no rule rebuilds it; the `reserve` repair is unusable |
| **P7** | **2** (discharge `old`; take-and-discharge before `free`) | 0 | 0 | 0 | 0 | zero with a copy content; two lines with an affine one |
| **P8** | 0 | 0 | 0 | 0 | 0 | no drop flag; `II-3`'s terms are erased |

Totals over the 26 cases and the 5 derived programs: **8 written annotations**,
**3 invariants plus one per traversal loop head**, **`O(1)` `use` steps per
symbolic-write group per iteration** (unbounded over a loop), **4 proof-only
branches**, **1 written outcome arm**, **1 re-`FREEZE` per join** for any
program that carries a measure across an `if`, **1 item inexpressible** (L02's
`n ≤ 1`), **1 item underivable** (L05), and **1 unrepairable rejection class**
(P4's reallocating arm).

§1.4's model predicts "join: nothing, always", "statement: zero … for P3's
writes", and "loop head: zero … P5, P6". The join row is contradicted by P1, the
statement row by P3, and the loop-head row cites two programs derived nowhere.

The straight-line and early-branch fragment (S01–S07, B01–B06, B07–B12, L01,
L06, P7 with copy content, P8) really is zero-cost, and that is the candidate's
strongest measured result: **19 of 26 cases cost the writer nothing**. The cost
is concentrated entirely in the three places `VERDICT-CORE.md` §6 predicted —
creation/ending (P4's `I-19` row, P3's pool), split/rejoin (L03, L05, the `I-3`
class), and the identity↔runtime bridge (P1's `FREEZE` lets and index branches,
P4's `cap` chain).

---

## 3. M7: repairability of each rejection class in the derivations

| Rejection class | Diagnostic the rules emit | Local repair? |
|---|---|---|
| `R1(i)`/`R1(ii)` at a constant state (S03, S06, S07, P7, B06) | rule, identity path, failing leaf, transition site, missing fact | **yes** — and B06's has no missing fact by construction (the identity ended on both leaves), which is the case's intended verdict but leaves the payload site-only |
| `JF-TERM` non-constant term (B03's `read(a)`, B05, B07, B09, B12.1, P4's `c.read()`) | the term, the leaf that fails, the site, "missing fact: `cond`" | **yes** — write the branch, the candidate's first taught route |
| `KEY` may-equal widening (P1's `take(v,j)`, P3's cursor) | "`'b[j]` may be the element taken at L1", "missing fact: `i # j`" | **yes** — write the branch; M3 deletes the `use` route, so the branch is the only route and it is local |
| `I-3` non-constant `own` at scope exit (B13, L04's enclosing scope) | "'A's obligation is outstanding when `cond` is false (site: …)", repair named as the guarded release | **straight-line yes, loop shape no** — the loop-shape repair is L05, which is underivable (F4(b), S5) |
| `II-2` arm exit carrying an inner atom (L05; `TERM`'s example) | "close 'A's divergence inside the `if d`, or hoist the take" | **no for L05** — no atom re-keying rule, and the case's control flow does not admit either named repair (F4(c), S14) |
| `II-6` non-constant loop head (L02) | identity, back edge, two named repairs | **no for the `n ≤ 1` sub-case** — repair 1 is a different program, repair 2 ("`invariant st('A) = Live`") reintroduces a rejection at the take, which M7(a) forbids (F4(d)) |
| `I-16`/`III-8` conditional `backing` after a reallocating `push` (P4) | "'b has ended when `n0 >= c0` (site: push, `ensures` clause 3). missing fact: `n0 < c0`" | **no in the else arm** — the fact is false there; no rule rebuilds the stored nominal at `'b2`, and `reserve` does not repair it (F4(a), S13) |
| `II-15` row 2, write through an origin set (P9, §8 D4) | "writing through an origin set cannot initialize 'X … repair: declare the discriminant" | **no when the callee selects on a value it does not expose** — §8 D4 says so in terms |
| `II-16` two unpacks not distinct (P3's parallel traversal) | "missing fact: `s1 # s2`. repair: `use INV_pool.distinct_links(a,b)` if the pool states it, or branch" | **conditional** — the repair exists only if the library wrote the invariant; and the `use` route's premise checking is unspecified (F3) |

M7(a) asks for a non-empty payload naming a tree location where a local fix
exists, for 100% of rejections, and that a repair applied as named does not
reintroduce the same rejection. Four of the nine classes fail it, one
conditionally. §6.1 declares M7 met.

---

## 4. The condition-atom bound, checked

The lens's specific question: the candidate has condition terms, so is the bound
real?

**On the state, yes.** `TERM` fixes one atom per identity per point, depth ≤ 1,
canonical form `ite(α, l₁, l₂)` with `l₁ ≠ l₂`. A term is `O(1)`, the base's
`2^d` family genuinely does not exist, and `JF-TERM`'s join and equality are
`O(1)` table operations. This is a real repair of the base and the derivations
confirm it: no item in the 26 cases produces a two-atom term, because `II-2`
rejects before one can form.

**On the cost, no.** Three leaks:

1. `Γ` is not bounded by anything. `ATOM` makes atoms immortal, `III-7` installs
   a relation per boolean reassignment, `I-16`/`I-17` install one per container
   call, `III-4` one per carve, `II-10` one per `old(e)`. The constraint digraph
   grows with `S`, so the *state* is `O(1)` and the *entailment* is `Θ(S³)` per
   closure (F1). `TERM` bounds the wrong quantity for M10.
2. `II-1`'s per-arm re-evaluation of every term in scope is what makes the bound
   hold, and it is in no family and no degree (S15).
3. The bound is enforced by a **rejection** (`II-2`, `II-4`), not by widening,
   and §7 prices that rejection at nothing while the derivations pay for it once
   — at L05, the one item in the corpus that nests two independent divergences,
   and the item the candidate's own derivation cannot complete (S5, S14).

So the honest statement of O11(b) under this candidate is: one atom per identity
per point buys nineteen of twenty-six cases at zero writer cost, at the price of
a cubic-in-program-size entailment closure and a refusal class whose single
exercised instance is currently underivable.

---

## 5. Verdicts overturned

1. **§6.1 "M10(a), first clause — met"** → **violated**: the measures `n`, `I`
   and `F` are not functions of program size, `n = Θ(S)` by `ATOM`, and
   `JF-KILL`/`JF-ORG` omit the families they call (F1, S1, S2).
2. **§6.1 "M1 — met"** → **violated**: the loop head is an unstated fixpoint
   over a partial join, and `use R(args)`'s premise checking — including a
   history premise — has no procedure (F2, F3); atom normalization is unstated
   (S11); `I-17` and `III-3` give one line two verdicts (S13).
3. **§6.1 "M7 — met"** → **violated**: four rejection classes with no local
   repair or with a repair the rules cannot check (F4, §3).
4. **§6.1 "M10, edit stability — met"** → **violated for nominal-row edits**:
   `I-19`'s closed row makes a field-type edit reach every declaration
   mentioning the nominal, transitively (S7).
5. **§6.1 "M8 — ≈46 constructs"** → **≥ 57** (S6).
6. **§6 "There are nine [families]"** → **≥ 13 procedures are invoked**: the
   loop-head fixpoint, `use` premise checking, atom installation into `Γ`, and
   the `II-1`/`II-2` per-arm scans are named nowhere (F2, F3, S15, S16).
7. **§7 "L05 — as the case states. same"** → **underivable under the rules as
   written**, three missing rules, and `II-8`'s example asserts the opposite
   (S5).
8. **§7 "P2 block distinctness — derivable by `JF-LIN` over frozen bounds"** →
   **not derivable for a symbolic block size**, and P2 is derived in no
   derivation file (S8).
9. **§1.4 "join: nothing, always" and "statement: zero … P3's writes"** →
   **contradicted by the candidate's own derivations**: a re-`FREEZE` per join
   and a `use` per symbolic-write group per iteration (S4, §2).
10. **§7 "L02 — as the case states. same"** → **the case's `n ≤ 1` sub-case is
    inexpressible** (no loop-counter φ-value rule), so the candidate rejects
    where the case says there is no repeated-take error — an unlisted capability
    loss (§2, boundary derivation line 386).
11. **`I-1`'s `'a # x` "from the allocator's written `ensures`"** → **cannot
    fire as written**, and thirteen derived items depend on it (S3).


# File: critique-brand-context-bounded-cross.md

# Cross-examination: candidate `brand-context-bounded`

Lens: **CROSS**. Refuter's brief: read the other two candidates' rules and
derivations, list every item they accept that `brand-context-bounded` refuses or
cannot derive and every item it accepts that they refuse, with rule numbers on
both sides; then state what this candidate asks the owner to accept that the
others do not, and what it spares the owner.

Sources read in full: `rules-brand-context-bounded.md`,
`rules-value-semantics.md`, `rules-window-focus.md`,
`derive-brand-context-bounded-{straight-and-early-branches,late-branches,boundary}.md`,
`derive-value-semantics-*`, `derive-window-focus-*`, plus `PROGRAMS.md`
(P1–P19) for the required properties. Short keys below: **BCB** =
`brand-context-bounded`, **VS** = `value-semantics`, **WF** = `window-focus`.

Nothing here selects a candidate. Two findings are recorded against VS and WF
where a cross-comparison turned on a claim of theirs that their own rules do not
carry; they are marked as such so the owner does not read a BCB refusal as a
capability difference when in fact all three refuse.

---

## 1. Ledger A — items the others accept that BCB refuses or cannot derive

| # | Item and line | Other candidate: rule that accepts | BCB: rule that refuses or falls short | Class |
|---|---|---|---|---|
| A1 | **L05**, `}` closing the `if active` arm inside the loop | WF accepts: `II7(c)` head row + `II8` (guard written at the head) + `II6(a)` feasibility; WF §3.21 L05 row, cost = one written head row | BCB **cannot derive it**: `II-2` (arm exit) requires the exit term's atom to be in the *enclosing* `Γ`; the arm carries `stop`, the invariant carries `a_h`, and no rule re-keys one to the other under the recorded `a1 = ¬stop`. Two further gaps at `II-8` (two terms over different atoms) and `II-3` (no constant-vs-term row). `rules` §7 row "L01 L02 L04 L05 L06 — as the case states — same" and `II-8`'s own worked example (line 823-ish, `rules` L809–L818) both assert ACCEPT | **capability loss, undeclared** |
| A2 | **P4**, `reserve(v, n0+1); push(v, 5); c.read()` | VS accepts with zero proof steps: `I-9` is one push rule (reallocation unobservable), `I-10` reserve's footprint is `{v.meta}` only, so element facts survive by `II-12`; `II-14` re-derives `c.i < len(v)`. WF *claims* accept at `III3`+`I17` arm A (see O3 below — overturned) | BCB **refuses**: `I-17` leaves its own conditional `ends ite(c0 >= m, none, 'b)`, so the cursor's `'b` carries `ite(c0 >= n0+1, Live, Gone)` and the missing fact is `c0 >= n0+1` — provable only *before* the reserve, i.e. vacuously. `rules` §7 "P4 repair order: the `cap` proof belongs **before** the reserve" and `III-3`'s "cost S1 repaired" are both void | **capability loss vs VS; understated** |
| A3 | **P4**, the reallocating arm's continuation | WF: `III4` states there is no re-derivation of a `Gone` backing and **the writer re-forms the element pointer from the container**; `II19` needs no field retyping because every access rejects on the state. VS: no instance (no stored pointer, DV-1) | BCB: the `else` arm cannot use the cursor and **no rule rebuilds a stored nominal at the new backing**. `I-19` fixes the declaration-site row only; nothing says whether `where backing('v) = 'b` is a construction premise or a live obligation after `III-8`'s kill | **missing rule at a required program line** |
| A4 | **P1** / **P17**, `read(v[k])` with `k # i`, `k # j` unproved; `q = a / d` | WF accepts the written proof step: `II15`'s own example `use k != i && k != j ; x = read(β[k]) // accept: a named application (INV), not a bare assume (owner O15(b))`, family `INV` (§5 row 14) | BCB **deletes the route**: §6 M3 — "a disequality on two runtime scalars has **no such rule**, so `use i != j` is not writable and the only route is a written branch. The base's `use k != i` / `use d != 0` routes are **deleted**". The accepted shape is `if … { … } else { outcome }` — a branch and an outcome arm in the emitted program | **same owner interim (O15(b)), opposite reading** |
| A5 | **P2**, arena block reuse at coinciding bytes | WF `I23`: a **fresh coarse name** `ρ4` over `ρ2`'s bytes; `dis` against the `Gone` name "is irrelevant because no access to it is admitted (I7)". No ghost, no O3 exposure. VS `I-16`: "a FRESH extent identity, even where its bytes are b2's" via `I-1` binding formation; ghost generation is charged for **pools only** (`III-7`) | BCB `I-23`/`III-6` mints a **ghost index** for the arena case too, because `KEY`'s may-equal class is keyed on base + shape + extent arguments, so two coinciding extents are may-equal without it. D5 therefore puts **P2 as well as P3** under owner decision O3 | **broadest O3 exposure of the three; presented as inherent** |
| A6 | A join whose merged value needs **two** independent atoms | WF `II3`: `ite` in canonical atom order to depth `Gmax` (provisionally 2), and on overflow a **deterministic, diagnosed collapse** by the automaton meet — a precision loss, never a rejection | BCB `TERM` + `II-2`: "If a join would produce a term whose arms are themselves terms over a *different* atom, the join is **rejected**". The refusal class is declared and never exhibited on the fixed corpus — and A1 is exactly an instance of it, booked as "rule missing" instead | **narrower acceptance; the price is unpriced** |
| A7 | **P9**-shaped write through a value that may name one of two storages | VS `II-20` (`subscript … origin { s.x, s.y }`): `t = 1 // accept`, paying a footprint wider than any written path for the block's span | BCB `II-15` row 2: a write through an origin set `{π₁…πₙ}` leaves `st` **unchanged** and is permitted only when every member is already `Init`; D4 records that the named repair (declare the discriminant) "is unwritable when the callee selects on a value it does not expose" | **capability boundary, filed as an open defect** |
| A8 | Distinctness of two frame-local storages (P1's `qi`, `qj` after `take`) | VS `I-2` + `PATH`: distinct roots of the containment tree are disjoint by the plane map, "no annotation, no proof, sound for any element type" | BCB: **no rule**. `I-2` states no distinctness effect; `I-1` requires the allocator's written `ensures` for the heap case; `R4(b)` forbids `#` from name inequality. BCB's own P1 derivation (`derive-…-late-branches` L506–L514) leans on `JF-OVL`'s same-base restriction, "a procedure detail, not a rule". WF shares the gap (its G2) | **missing rule; VS closes it structurally** |

## 2. Ledger B — items BCB accepts that the others refuse

| # | Item and line | BCB: rule that accepts | Other candidate: rule that refuses | Class |
|---|---|---|---|---|
| B1 | **P4**, `struct Cursor { at: ptr<…>, i }` — a stored pointer into a container | `I-19` (closed identity row) + `II-13` (a projection is a sub-identity *name*, nothing is consumed, the base stays writable) | VS **capability loss**: `§1.1` (projections second class), `II-16`, `II-19`, `II-21` — "a projection is second class — it cannot be the value of a binding". DV-1 + DV-2 (the signature cascade), with no local fix | BCB + WF vs VS |
| B2 | **P7**, two long-lived writable names for one slot | `§1.1` (`ptr<'a,T>` copyable, no mode, no duration, no region) + `D4` + `KEY` (state is keyed on `'A`, so every name sees the hole) | VS DV-3: the premise is unwritable — `II-17`'s conflict table refuses `inout`/`inout` on overlapping paths and a projection cannot be stored to become the second name. "R3 has no acceptance test" | BCB + WF vs VS |
| B3 | **S03, S06, B07, L02, B04** — the hole refusals | `I-7` (`take` → `Uninit`), `I-8`/`I-9` premises, `KEY` (⊓ over the may-equal class) | VS DV-4: `I-7` makes `replace(inout s, None)` **total**; every case whose expected verdict is "reject: hole" has no instance. 7 of VS's 13 deliberate refusals are this family | BCB + WF vs VS |
| B4 | **B05, B06** — correlated distinct targets survive selected reclamation | `II-14` (origin split) + `II-15` row 1 (strong leafwise) + `II-5` (correlated leafwise evaluation of facts sharing one atom) | VS: **two capability losses**. `II-3` joins `Γ` by syntactic intersection, `III-8`/`I-20` kill `LIVE(k)` for every handle not proved distinct; VS's own derivation re-attributes B05 to O11(a) and B06 to a missing re-keying rule | BCB + WF vs VS |
| B5 | **B10, B11, B12-variant, B13-repair, L04** | `II-3` rows 4–5 (guarded `own`), `II-5`, `II-14`, `II-9` (break edge carries its actual state) | VS `II-2`'s definiteness clause `[stated here]`: mixed consumption at a join is a **rejection**; B10, B12, B13 and L04 all reject at a join, each needing an `Option`-slot restructuring | BCB + WF vs VS |
| B6 | **P3** as a whole | `II-16` (unpack), `I-24`, `I-23`, `III-5`, `III-6`, `III-12` (`RSt` residual framing: the ∀-invariant is *weakened* per symbolic write and restored by one `use`, not killed and restated) | VS: **underivable — rule missing** (three: `I-20`'s footprint vs the neighbour writes; `INV`'s witness for re-deriving `LIVE`; `Links` vs a terminator), plus DV-6's cost on every other handle | BCB (and WF) vs VS |
| B7 | **P13**, `read(pool[h])` after `compact()` | `III-10`: a **written** `map : array<HandleId, Slot>` that `compact` rewrites, charged at one word per outstanding handle; the handle survives "because the MAP was rewritten, not because an index is stable" | VS `I-21` asserts "the handle IS the fixup form" with **no map plane declared** in `backing Pool<P,T> { meta, slots }` — and its own second example rejects `compact_renumber` precisely because "the handles it invalidates are ordinary data scattered through the program and the checker cannot enumerate them". WF `I21`'s `ensures forall h. Occupied(ρP, slot_of(h))` names `slot_of` with no declared carrier | **BCB is the only one that declares and charges the map** — see O2, O3 |
| B8 | **P1**'s double `take` at may-equal indices | `KEY` (strong update only when `mayEq(π) = {π}`) + `III-12`'s exception list | Not refused by the others — VS gets it from `II-17`'s conflict table plus `i != j` in a `requires`; WF from `PART`'s written obligation. All three deliver; listed so the owner does not read BCB's F1 repair as unique | — |

---

## 3. What BCB asks the owner to accept that the others do not — and what it spares

### Asks

| Ask | BCB | VS | WF |
|---|---|---|---|
| **A name kind inside an identity path**: the erased ghost index `@g` on *every* reusable extent — arena bytes **and** pool slots (`III-6`, `I-20`, `I-23`, `I-24`) | yes, and **M2(ii) is conditional on owner decision O3** for both P2 and P3 (D5) | a ghost generation for **pools only** (`III-7`); arenas use fresh bindings (`I-16`). O3 exposure on P3 only | **none** (`III11` declared absent); pays OD1 instead — a refilled slot reads the wrong occupant with no diagnostic |
| **A row kind in a type**: `Ρ`, an ordered identity row, `N<Ρ>` well-kinded at every arity, with every pointer-carrying nominal's row **closed** (`I-19`) — `Cursor<'v,'b>` plus a `where`, and `O(f²)` `#` clauses at any interface producing an aggregate of separately named storages (cost S9) | yes | no: **one** pool brand `P`; "every other type has zero identity parameters" (`I-18`) | one store name per stored pointer (`§1.3`), **plus** a second pointer type form `ptr<ρ↓w.k,T>` and the `ESC` judgment (`II19`) |
| **A bounded condition term with a refusal on overflow**: one atom per identity per point, shrinking on arm entry (`ATOM`, `TERM`, `II-1`, `II-2`) | yes — refuses | no terms at all (O11(a), `II-5`); path conditions are premises discarded at the join | `Gmax = 2` with a **deterministic diagnosed collapse** (`II3`), not a refusal; OD2 records that no evidence sets the constant |
| **A pool/container invariant with a residual exception list**: `RSt(π[lo..hi), s, X)` weakened per symbolic write, `X` reset at loop heads and calls (`III-12`, `JF-QI`) | yes — unique to BCB, and it is what delivers P3's requirement (4) | `OLDCHAIN` — one rewrite per killed measure fact per call, no residual list (DV-5: no route without an `ensures`) | `COVER` residue keys + wholesale re-establishment by a row's `ensures` |
| **A restructuring per case** | **zero** across S01–S07, B01–B12, L01–L06 (B13's guarded release is a line the case itself offers) | **thirteen**: `Option` slots and yielded survivors at S03, S06, B02, B03, B04, B07, B09, B10, B11, B12, B13, L02, L04, L05 | **one** (B13), same shape as BCB's |
| **Nine judgment families** with `M10(a)`'s written-step clause declared violated at `Θ(S·I·d·n³ + S²·n³)` with `q` possibly zero | yes, **asymptotic only, unmeasured** | twelve families; M10 edit-stability declared violated, **measured** (82.4 s vs 42.8 s; 29.9/164.5/1691.0 ms, exponent ≈2.6–3.4) | seventeen families; M10 declared violated on two grounds, **measured** (30.0/165.4/1690.7 ms, exponent 2.5–3.4) |
| **M8** | ≈46 constructs, declared red against an unset `K` | 17 taught items, "undecidable pending O6" | ≥34 cards, declared violated |

### Spares

- **No second-class discipline.** No `ESC` judgment, no second pointer type
  form, no parser-position restriction, no span table (WF `II19`, priced at +1
  type form, +1 judgment, +2 M8 cards). BCB's answer to O7/O12 is (b)-not-taken:
  `II-13` makes a projection an ordinary sub-identity name that may be stored,
  and the base's writability is never suspended.
- **No window machinery.** No `focus`/`carve` statement, no carve forest, no
  cross-level `dis` walk, no `PART` at `O(E²)` with `k(k−1)/2` writer-supplied
  inequalities for a `k`-way split (WF OD12), no declared-type default carve that
  loses per-column facts under abstraction (WF OD6).
- **No plane-map declaration per backing and no conflict table.** VS asks for
  one `plane<…>` line per field and one invariant per backing (`I-2`, `III-2`)
  and a four-cell `let`/`inout`/`sink` table (`II-17`) that refuses two
  overlapping writable names; BCB gets sibling-field distinctness from `JF-LIN`
  schema 7 and admits unrestricted sequential aliasing (D4).
- **No capability loss on the 26 cases** — subject to F3 below, which is one.
  VS books two (B05, B06) plus P4.
- **The only declared, charged handle→slot map** (`III-10`). Both competitors
  assert handle survival across compaction with no carrier (Ledger B7); BCB pays
  the word and says so.
- **No unjustified precision constant.** `TERM`'s bound is one atom by
  construction; WF's `Gmax = 2` is an unevidenced cliff (OD2).

---

## 4. Findings

Severity: **fatal** = a hazard admitted or a requirement violated undeclared;
**serious** = a wrong verdict, an understated cost, an unnamed family, a missing
rule; **minor** = wording.

| # | Sev | Item / line / rule | Finding | Fix |
|---|---|---|---|---|
| **F1** | **fatal** | `I-4` (`rules` L244–L258) premise *"`st('a) ≠ Gone`"*; same wording in `I-6` (L276) | `§1.2`'s lattice has a top, `⊤ = Live ⊔ Gone`, and `⊤ ≠ Gone` holds syntactically, so **`free` and `write` are accepted at `⊤`** — a state that includes the Gone leaf. The path is writable with BCB's own worked example: `fn wipe<'a>(p) ensures st('a) = Gone writes 'a` (`I-14`) ends storage **without** an `ends` clause, so `own('a)` stays the constant `held` while `st('a)` becomes `ite(c, Gone, Init)` and then `⊤` at an atom-free join (`II-9`). `I-5`'s guard is vacuous for a copy content and `I-3` never runs. No rule refuses an access at `⊤`; `read` has no rule at all (BCB's own gap G1). WF is closed here by a **positive** premise, `I7`: "`St(ρ) ∈ {Init, Uninit}` on every feasible guard assignment"; VS is closed by `Σ(x) = bound` over a two-point map with `II-2`'s explicit mixed-case rejection | Replace the negative premise in `I-4` and `I-6` with the positive one `st(π) ⊑ Live` (i.e. `Init`, `Uninit` or `Live`, excluding `⊤`), which keeps L06's `free` at `Live` and closes the top |
| **F2** | **fatal** | `II-3` join table (L643–L660) vs `I-5` (L260–L275); `§1.3`'s `obl(π) : none \| one`; `TERM` (L129) | **`obl` has no join rule and no term form**, so `I-5`'s F5 repair is bypassed by one `if`. After `if c { v = take(p) }` on an obligation-carrying content, `st('A) = ite(c, Uninit, Init)` and `obl('A)` is `none` on one edge and `one` on the other; `II-3`'s table has rows for `st`, `own` and `origin` only, and `TERM` admits terms for state, own and origin only — `ite(c, none, one)` is not representable. `free(oA)` then meets `I-5`'s disjunction with an undefined second disjunct and nothing rejects, dropping the value's obligation on the `¬c` leaf. VS states the clause it needs (`II-2` *"[stated here]"*, an explicit rejection of mixed consumption); WF's `I7` quantifies "every obligation held by a value inside ρ is discharged" over feasible guard assignments | Add an `obl` row to `II-3` that **rejects** when the two edges disagree and no atom is available, and add `obl` to `TERM`'s term-carrying heads so the correlated case is representable |
| **F3** | **fatal** | **L05** (`derive-…-boundary` L573–L605, L624–L634) vs `rules` §7 (L1272) and `II-8`'s worked example (L809–L818) | The candidate's mandatory change #1 (`TERM`/GCR/`II-2`) **rejects the case the round uses to test O11(b)**, and the rules file asserts the opposite in two places. BCB's own derivation records three missing rules and calls L05 "underivable"; the first of them is `II-2` refusing the `if active` arm exit because its term carries `stop`, which is not in the enclosing `Γ`. WF derives L05 under `II7(c)`/`II8`/`II6(a)` at the cost of one written head row. Consequences: (i) the §7 "0 capability losses" line is false; (ii) the O11(b)-with-a-bound claim is contradicted by the loop case; (iii) the general no-drop-flag loop idiom (a source Boolean carrying an obligation across iterations) has no accepted spelling | State the atom re-keying rule that `JF-ENT` step 4's recorded relation already licenses (substitute an entailed-equivalent atom into a term), give `II-8` a leafwise entailment check across two atoms, and add `II-3`'s constant-vs-term row — then reclassify L05 as *accepted with a written invariant* |
| **F4** | **fatal** | `II-15` row 1 (L896, L914) — *"strong, leafwise … `st('X) := ite(c, Init, st('X))`"* — against `TERM` (L131–L136): *"There is no `ite` nesting anywhere in the checker's state"* | A leafwise write through an origin term over atom `c` **substitutes into** the target's existing state term. When that state is already a term over an independent atom `d` the result is `ite(c, Init, ite(d, Uninit, Init))` — depth 2. The shape is reachable in three statements (`if d { take(px) }`; `if c { p = ref(x) } else { p = ref(y) }`; `write(p, 5)`) with `Γ = {}`. `TERM`'s bound is stated for terms *produced by a join* and `II-2` refuses only there; **no rule covers the write case**, and `II-5` is premised on the facts carrying "the **same** atom", so `JF-ORG`'s leafwise resolution is undefined here too. The checker therefore has no specified behaviour on this input, which breaks **M1**'s "specification-fixed" clause undeclared, and it voids §6's withdrawal of the base's `2^d` M10 amendment. WF states the overflow rule its bound needs (`II3`'s deterministic oldest-atom collapse); VS has no terms | Extend `TERM`'s bound and `II-2`'s refusal to *every* term-producing rule, and state explicitly that `II-15` row 1 rejects (naming both atoms and the two repairs) when the target's state term's atom differs from the origin term's |
| **F5** | serious | **P4** — `I-17` (L451–L470) example rejects `read(pe)` after `reserve; push`; `III-3` (L989–L1003) example marks the identical line **accept** | Same candidate, same program, two verdicts, at the bridge rule the round exists to write out. §7's "P4 repair order: the `cap` proof belongs **before** the reserve" makes the repair vacuous (proving `c0 >= n0+1` before the reserve removes the need for it), and `III-3`'s header claim "cost S1 repaired" is therefore false. VS accepts the line with zero proof steps (`I-9` single push rule + `I-10`'s `{v.meta}` footprint + `II-14`) | Delete `III-3`'s example, make `I-17`'s `ends` unconditional (the reallocation is unobservable to a caller that re-forms its pointers) or add a writer-visible `rebind` form, and replace §7's repair row with the branch route the derivation actually uses |
| **F6** | serious | **P4** reallocating arm (`derive-…-boundary` L94–L106, L158–L163) | No rule rebuilds a stored nominal at the new backing, and no rule says whether `I-19`'s `where backing('v) = 'b` is a construction-site premise or a live obligation re-checked after `III-8`'s kill. P4's required line "`b = c.read()` — legal iff the candidate can carry *still valid* across push" is therefore accepted only on the non-reallocating arm; on the other arm the program has no continuation the rules can express. WF supplies the route (`III4`: "the writer re-forms element pointers from the container"). The summary books this as "accepted with a stated cost" | Add the reconstruction rule: re-establishing a nominal's `where` clause at a construction site discharges it against the current `backing` term's selected leaf, and state that the clause is a construction premise, not a live obligation |
| **F7** | serious | §6 M3 (L1237–L1241) vs WF `II15` (L796–L820) + WF family `INV` | The two candidates read owner interim **O15(b)** in opposite directions on the same line of P1. BCB deletes `use k != i` because no named rule concludes a scalar disequality; WF writes exactly that step and calls it "a named application (INV), not a bare assume (owner O15(b))". The cost difference is a runtime branch plus an outcome arm in P1 and P17 — kernels — under BCB and nothing under WF | Say in §6 that the deletion is BCB's *reading* of O15(b), name the disagreement, and give the owner the two costs side by side rather than presenting the deletion as settled |
| **F8** | serious | **P1** cost row, §7 (L1275) and `derive-…-late-branches` L580–L584 | The P1 cost is **overstated**: the derivation presents `if i < n0 && j < n0 && i != j { … }` as "the only route", but `JF-ENT`'s stated inputs are "`Γ` (guard atoms + **`requires`** + `INV` conclusions)", so the five index facts can be written as a callee `requires` with **no runtime branch** — which is exactly VS's rendering (`fn split_write(…) requires i < len(v) && j < len(v) && i != j`, `II-10`). Reporting only the branch route distorts the M3/O15(b) evidence this round is collecting | Add the `requires` route to the P1 cost row and restate the cost as "one written premise per unproved index fact, at a signature or in a branch" |
| **F9** | serious | `II-3` (L643–L660) — the table's rows | The **one join rule is not total over the candidate's own fact components**. `§1.3` holds `st`, `own`, `owns`, `obl`, `layout`, `backing`, `measure` bridge facts (`len`, `cap`, `head`, `gen`, `slot`, `tag`), `RSt` with its residual list, `val`, `≤` and `#`; `II-3` states rows for `st`, `own` and `origin` only. BCB's own P1 derivation records the bridge/`RSt` case as a gap (P1.4); the `obl` case is F2. VS's `II-3` is total by construction (intersection of `Γ` with support), WF's `II3` merges "every key of `St` and every fact in `Facts`" | Make `II-3` total: state one default (intersection / drop, with re-`FREEZE` named as the repair) and enumerate the heads that instead carry a term |
| **F10** | serious | `II-9` (L777–L795) vs `II-3` row 5 (L655) | Two verdicts for the `own` join at an atom-free exit: `II-3` row 5 says **reject**, `II-9`'s L04 example produces `own('A) = ⊤` and defers to `I-3`. §7 endorses `II-9`, but `⊤` is not a value of `own` in `§1.3` (`held \| discharged \| ite`) and no rule gives it a meaning. Under `II-3` L04's loop rejects at the exit join; under `II-9` it rejects later, at the scope exit. BCB's own boundary derivation flags the conflict | Pick `II-3` row 5 (reject at the join, naming the release site), delete `⊤` from `II-9`'s example, and say L04's refusal is at the exit join |
| **F11** | serious | `TERM` (L129–L162), §7's O11 paragraph (L1291–L1299) | The refusal class `TERM` buys is **declared but never exhibited on the fixed corpus**. All three BCB derivation parts state the class is "never reached here"; meanwhile L05 fails on exactly this bound (F3) and is booked as "rule missing". WF reports its own depth per case (B12 at depth 2, never 3) so the owner can price the bound. As written the owner is asked to accept a refusal class with no instance and a "0 capability losses" tally that the one instance would break | Reclassify L05 under `TERM`/`II-2`, exhibit the two-atom program §2's example already contains, and correct the §7 loss tally |
| **F12** | serious | `I-23`, `III-6`, D5 (L1324) | The **arena** half of the O3 exposure is self-inflicted and presented as inherent. WF (`I23`) and VS (`I-16`/`I-1`) both reuse arena bytes with a fresh name and no ghost, because `Gone` is permanent and no access to the old name is admitted; BCB needs the ghost only because `KEY`'s may-equal class is keyed on the extent, so two coinciding extents are may-equal. D5 puts P2 and P3 together under O3, making BCB's O3 exposure the broadest of the three without saying why | Restrict `III-6`'s ghost to pool slots (where `KEY` genuinely needs it) and add a `KEY` clause that a path whose state is `Gone` is excluded from every may-equal class — which removes P2 from D5 |
| **F13** | serious | D4 (L1323), `II-15` row 2 | A write through an undiscriminated origin set is a **capability boundary**, not an open defect: D4 itself says the named repair is unwritable when the callee selects on a value it does not expose, and VS delivers the same capability span-locally at `II-20` by paying a wider footprint. Filing it under "open defects" hides an O7/O12-adjacent boundary the owner is being asked to accept | Move D4 into §7's boundary list beside the O7/O12 paragraph and price it against VS's `II-20` |
| **F14** | serious | `I-2` (L214–L224); P1 gap in `derive-…-late-branches` L506–L514 | Distinctness of two frame-local mints has **no rule**, so P1's `par { write_part(qi,·) \|\| write_part(qj,·) }` rests on `JF-OVL`'s same-base restriction — a procedure detail. `R4(b)` forbids deriving `#` from name inequality and `I-1` demands a written `ensures` for the heap case, so which régime a frame slot is under is unstated. VS closes it structurally (`I-2` + `PATH` over distinct containment-tree roots, no proof); WF shares the gap | Give `I-2` an explicit effect: a frame mint is `#` from every live identity that is not an ancestor or descendant, by construction of the containment forest, not by name inequality |
| **F15** | serious | §6.1's M10 declaration (L1219–L1223) | BCB's M10 violation is **asymptotic only**. VS and WF both declare their M10 violations against measured numbers on the same bundle (≈2.6–3.4 and 2.5–3.4 exponents; VS's 82.4 s edit-reach figure). The owner is being asked to compare three M10 declarations of which one has no measurement, on the round's only quantitative axis | State that the degree is unmeasured and not comparable, or run the same fixture the other two cite |
| **F16** | minor | §6 M8 (L1242–L1250) | "≈46 constructs" is not comparable to WF's "≥34 cards" or VS's "17 taught items" — three units, one unset `K` (O6). The three M8 lines cannot be read against each other | Name the counting unit and count all three the same way, or say the number is unit-incomparable |
| **F17** | minor | §7 (L1265–L1288) | §7 asserts verdicts for **P2, P6, P9, P11, P13, P16** in the same table as the rows this round derived line by line, with no derivation behind them. The round's hard rule is that every derivation is line by line; the others carry the same debt but do not mix the two kinds of row in one table | Mark the underived rows, or move them to a separate "asserted, not derived" table |
| **F18** | minor | `I-23` example (L1058), `III-6` (L1050) | `// DEPENDS ON OWNER DECISION O3` appears inside an `accept` comment. MECHANISM-MAP §3's style reserves the comment for the fact used or the fact missing; a conditional verdict has no comment form | Write the conditionality in the rule's prose and leave the ladder comment to the fact |
| **O1** | serious (**against WF**) | WF `derive-…-boundary` L70–L80 vs WF `rules` `II12` (L730–L746) | WF's P4 accept of `reserve(v, len_of(v)+1); push(v,5); b = c.read()` is **not derivable under its own rules**: `II12`'s worked reserve row is two-armed on the backing (`Init ⇒ Init when cap >= 100 \| Init ⇒ Gone, split β' otherwise`), so `St(β)` after the reserve is `ite(g#, Init, Gone)` exactly as BCB's `I-17` says. The derivation reads only `III3`'s `cap` ensures and ignores the backing clause. Recorded here because it removes an apparent capability difference: on A2 the real split is VS-accepts vs BCB-and-WF-refuse | Re-derive WF's P4 with `II12`'s reserve row applied, or give `reserve` a one-armed backing clause |
| **O2** | serious (**against VS**) | VS `rules` `I-21` (L567–L591) | VS's P13 accept rests on "σ(h) is re-read from the handle's own word, **which the compaction rewrote**: the handle IS the fixup form", but `Handle<P,T>` is ordinary copyable data scattered through the program, `compact`'s footprint is `{p.slots, p.meta}`, no `map` plane is declared in `backing Pool<P,T>`, and `I-21`'s own second example rejects `compact_renumber` because "the checker cannot enumerate" those handles. The accept and the rejection contradict each other | Declare the map plane and charge it, as BCB's `III-10` does, or withdraw the P13 accept |

---

## 5. Verdicts overturned

**On `brand-context-bounded`:**

1. `rules` §7, L1272 — "L01 L02 L04 **L05** L06 | as the case states | same". **Overturned:** L05 is rejected by `II-2` at the `if active` arm exit. The candidate's own derivation says underivable; the cause is `TERM`, not an accident of drafting (F3).
2. `derive-…-boundary` summary — "Differences from `CASES.md`'s expected verdicts, classified: **none is a capability loss**". **Overturned:** L05 is one, and `II-8`'s worked example asserting it accepts is wrong (F3, F11).
3. `rules` §7, L1280 — "**P4** repair order | the `cap` proof belongs **before** the reserve | correction (soundness F3(b))", and `III-3`'s "cost S1 repaired". **Overturned:** the repair is vacuous and the two worked examples give opposite verdicts on the same line (F5).
4. `derive-…-boundary` summary — "**P4** | accepted with a stated cost". **Overturned to:** accepted on the non-reallocating arm only; the reallocating arm has no expressible continuation and the reconstruction rule is missing (F6).
5. `I-4`'s statement that a `Gone` path is never revived and `R1(ii)` is closed. **Overturned:** `free` and `write` are admitted at `⊤`, whose Gone leaf they do not exclude (F1).
6. `I-5`'s header — "Obligation guard on ending (**soundness F5**)". **Overturned:** the guard is bypassed at the first conditional `take`, because `obl` has no join rule and no term form (F2).
7. `TERM`'s consequence clause — "Term size is `O(1)`; the base's declared `2^d` M10 amendment is **withdrawn** — the exponential family no longer exists". **Overturned:** `II-15` row 1 produces a depth-2 term and no rule covers it (F4).
8. §6's "**M2(ii) — met**, conditional on owner decision O3" scoped to pool and arena reuse alike. **Overturned in scope:** the arena half is a consequence of `KEY`, not of the problem, and both competitors do P2 with no ghost (F12).
9. `derive-…-late-branches` P1 — "one written guard carrying five index facts … because `M3`/O15(b) deletes the `use k != i` route". **Overturned as the only route:** a callee `requires` discharges the same five atoms with no runtime branch, by `JF-ENT`'s own stated inputs (F8).

**On the other candidates, where the overturn changes the cross-comparison:**

10. WF `derive-…-boundary` P4 — "`b = c.read()` // accept: `St(β)=Init` unguarded". **Overturned** by WF's own `II12` two-armed reserve row (O1). The A2 difference is VS-alone-accepts, not two-against-one.
11. VS `rules` `I-21` — "the handle IS the fixup form" and the P13 accept. **Overturned** (O2). BCB's `III-10` is the only stated and charged carrier of the three, which strengthens Ledger B7 in BCB's favour.

---

## 6. One-paragraph answer to the brief

`brand-context-bounded` buys, relative to `value-semantics`, the whole of P4's
stored pointer, P7's two writable aliases, B05/B06's correlated release, the
hole refusals of R1(a)(i), and P3 — sixteen case verdicts and two programs that
VS loses or cannot derive — and it buys them **without** WF's second-class
discipline, window machinery, `ESC` judgment or `Gmax` cliff. What it asks for
in exchange is a ghost index inside the path (broader than VS's, and unlike
WF's, which does not exist), an identity-row kind with closed rows on every
pointer-carrying nominal, a one-atom condition term whose overflow is a refusal
rather than a collapse, and a residual-exception list on every range invariant.
What it does **not** yet have is a total join rule, a positive premise on its
ending and writing rules, a term-overflow rule for its own leafwise write, and
an accepted spelling for L05 — and those four are the places
`VERDICT-CORE` §6 predicted: the rules that destroy storage, the rules that
rejoin state, and the bridge between a name and the values that determine it.
Three of the four are one clause each; the fourth (L05) needs the atom re-keying
step `JF-ENT` already records.

