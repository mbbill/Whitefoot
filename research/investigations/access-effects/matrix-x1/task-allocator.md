# Engineering task under candidate x1: a memory allocator written in the language

Derived against the frozen rule set in `CANDIDATE-X1.md` (2026-09-18) and only that
file. `PROGRAMS.md` supplies the task row "Allocator over reserved memory" and the
discriminating programs P2 (arena with independently released blocks) and P18
(allocation failure and a byte budget); their requirements are quoted where they
decide something. Nothing under "Not in this candidate" or "Deferred" is used: no
`with` blocks, no `uniq`/`mut` markers, no regions or store brands, no `take`/`put`
holes, no quantified facts, no channels, no atomics, no destructors, no traps, no
header-plus-tail heap block. Rule 16 is provisional and this task depends on it;
the dependence is stated in full at the end.

Cost convention from the protocol: **runtime performance is the only cost**.
Verbosity, extra parameters, extra contract clauses and duplicated code are not
costs. Instruction counts below are counted against a size-class C++ allocator of
the tcmalloc/jemalloc shape and against a Rust slab/bump allocator whose interior
uses `unsafe`, since no safe Rust allocator exists either.

---

## 1. The question this task tests

Can a program written under candidate x1 own a reserved region of memory, hand out
several blocks that are in use simultaneously, release one block without disturbing
the others, reuse the released space for a later request, and keep its management
state shared between all of those operations — and what does the rule set admit
between operations on different blocks?

The short answer, established below: **yes, with cost, and the cost is concentrated
in exactly two places.** Cross-block parallelism over separately allocated blocks
costs a hoisted disjointness test because Rule 11 forbids the quantified fact the
allocator would need to export; and user-level release can never invalidate a
reference into the block it releases, because Rule 3's invalidation is prefix-based
and a user-level release writes no prefix of a payload path. The second point is a
task-level safety failure against P2's explicit requirement, not a memory-safety
failure.

---

## 2. Shared declarations

Everything here is owned data. Rule 1: "Every struct, enum, tuple, array,
slice-like value, Box, DynBox, and generic instantiation holds only owned values."
No handle, header or free-list link is a reference; each is a `u64` index or offset,
which is what Rule 16 prescribes ("A pool is a DynBox plus indices used as
handles").

```text
const NWORDS: u64 = 1 << 20        // the reserved region, in 64-bit words
const NHDR:   u64 = 1 << 14        // the largest number of simultaneously tracked blocks
const NCLASS: u64 = 20             // size classes 1, 2, 4, ..., 2^19 words
const NIL:    u64 = NHDR           // "no successor"; deliberately out of range for hdr

struct Hdr {
    off:      u64,      // first word of the block inside the region
    size:     u64,      // words, always a power of two in this design
    gen:      u64,      // bumped on every release (Rule 16: "keep a generation number as data")
    nextfree: u64,      // free-list link, an index into hdr, or NIL
    live:     u8,       // 1 while the block is handed out
}
// every field is Copy, so Hdr is copy (Rule 8) and DynBox::filled applies (Rule 6)

struct Meta {
    hdr:   DynBox<Hdr>,             // one header per block ever created; the index is the handle
    heads: array<u64, NCLASS>,      // per-class free-list heads (Rule 7 admits array<T, N>)
    front: u64,                     // bump frontier: words [0, front) have been carved
    nhdr:  u64,                     // headers in use: [0, nhdr)
}

struct Region {
    data: DynBox<u64>,              // the reserved region itself
    meta: Meta,                     // the shared management state, a sibling field of data
}

struct Blk { id: u64, off: u64, size: u64, gen: u64 }   // the handle; Copy
```

Two representation choices are load-bearing and are justified where they are used:

- **The metadata is a sibling field of the payload, not embedded in it.** `Meta`
  sits beside `data` inside `Region`. Rule 10's own example decides why this
  matters: `two(&a.x, &a.y)  // accepted: distinct fields`. An in-band header, the
  classic dlmalloc layout, would put the free-list link inside `data` and make every
  release write a path under `data`.
- **Every bound the allocator exports is phrased over a constant, never over a
  measure.** `result.id < NHDR`, not `result.id < len_of(m.hdr)`. Rule 11 says "A
  fact that mentions a path is invalidated when that path is written (by statement
  or call) unless the callee's `ensures` re-establishes it." A fact that mentions no
  path is never invalidated, so a handle's bound survives arbitrarily many
  intervening allocations and releases with no re-proof and no runtime test. This
  single phrasing choice removes one compare and one branch from every release.

---

## 3. Program D1 — the strongest program: one contiguous region, size classes, independent release, byte reuse

This is the strongest form because everything the checker would want is
data-determined: the size class comes from a runtime request, the free-list head is
a `u64` pulled out of an array with no fact attached to it, the reused offset is
whatever an earlier release put on the list, and the allocator is called through
signatures from another compilation unit, so no caller can see a body.

### 3.1 Construction

```text
fn region_new() -> Result<own Region, Oom>
    contract {
        ensures when Ok: len_of(result.data) == NWORDS;
        ensures when Ok: len_of(result.meta.hdr) == NHDR;
        ensures when Ok: result.meta.front <= NWORDS;
        ensures when Ok: result.meta.nhdr <= NHDR;
    }
{
    d = DynBox::filled(NWORDS, 0)?                    // Rule 6: T Copy, cap_of == len_of == NWORDS
    h = DynBox::filled(NHDR, Hdr { off: 0, size: 0, gen: 0, nextfree: NIL, live: 0 })?
    hs: array<u64, NCLASS>
    for c in 0..NCLASS { invariant hc: c <= NCLASS; hs[c] = NIL }
    return Ok(Region { data: move d, meta: Meta { hdr: move h, heads: hs, front: 0, nhdr: 0 } })
}
```

`DynBox::filled` is the only way to get a fully initialized block, and Rule 12
demands it: "No place is ever partially moved: every path is either wholly present
or the program cannot name it", with the worked example "slots: DynBox<Entry> //
fully initialized (zero-filled); a 'logically empty' slot holds a valid value".
Zero-filling `NWORDS` words is a one-time cost the C++ form does not pay; see the
cost table, line R0.

### 3.2 The representation invariant travels in the contracts

x1 has no invariant attached to a type. Every operation therefore states the
allocator's representation invariant as a `requires`/`ensures` pair, and the caller
carries it. This is pure verbosity and costs nothing at runtime, but it is what
makes every bound below provable without a test.

```text
// the invariant, written once and repeated in every signature:
//   INV(m) ==  len_of(m.hdr) == NHDR  &&  m.front <= NWORDS  &&  m.nhdr <= NHDR
```

### 3.3 Allocation

```text
fn class_of(n: u64) -> u64
    contract {
        requires n > 0;
        requires n <= 1 << (NCLASS - 1);
        ensures result < NCLASS;
        ensures (1 << result) >= n;
    }

fn alloc_words(m: &Meta, n: u64) -> Option<Blk>   writes(m)
    contract {
        requires n > 0;
        requires n <= 1 << (NCLASS - 1);
        requires len_of(m.hdr) == NHDR;
        requires m.front <= NWORDS;
        requires m.nhdr <= NHDR;
        ensures  len_of(m.hdr) == NHDR;
        ensures  m.front <= NWORDS;
        ensures  m.nhdr <= NHDR;
        ensures when Some: result.id < NHDR;
        ensures when Some: result.size >= n;
        ensures when Some: result.off + result.size <= NWORDS;
    }
{
    c  = class_of(n)                     // c < NCLASS, (1 << c) >= n
    sz = 1 << c                          // no overflow: c < NCLASS == 20, so sz <= 2^19
    id = m.heads[c]                      // array<u64, NCLASS> at c < NCLASS: Rule 7 discharged statically

    if id < NHDR {                       // reuse path; see trace T3
        h = m.hdr[id]                    // Rule 7: id < NHDR == len_of(m.hdr)
        m.heads[c]  = h.nextfree
        m.hdr[id]   = Hdr { off: h.off, size: h.size, gen: h.gen, nextfree: NIL, live: 1 }
        return Some(Blk { id: id, off: h.off, size: h.size, gen: h.gen })
    }

    if m.front + sz <= NWORDS {          // carve path; C++ performs the same test
        if m.nhdr < NHDR {               // header exhaustion; this is P18's declared budget
            j   = m.nhdr
            off = m.front
            m.hdr[j] = Hdr { off: off, size: sz, gen: 0, nextfree: NIL, live: 1 }
            m.nhdr   = j + 1
            m.front  = off + sz
            return Some(Blk { id: j, off: off, size: sz, gen: 0 })
        }
    }
    return None                          // Rule 14: allocation fails by value, it never traps
}
```

### 3.4 Release

```text
fn free_words(m: &Meta, b: Blk) -> Bool   writes(m)
    contract {
        requires b.id < NHDR;
        requires len_of(m.hdr) == NHDR;
        requires m.front <= NWORDS;
        requires m.nhdr <= NHDR;
        ensures  len_of(m.hdr) == NHDR;
        ensures  m.front <= NWORDS;
        ensures  m.nhdr <= NHDR;
    }
{
    h = m.hdr[b.id]                      // Rule 7: b.id < NHDR == len_of(m.hdr), from requires
    if h.live == 0  { return false }     // double release, detected as data
    if h.gen  != b.gen { return false }  // stale handle, detected as data (Rule 16)
    c = class_of(h.size)
    m.hdr[b.id] = Hdr { off: h.off, size: h.size, gen: h.gen + 1, nextfree: m.heads[c], live: 0 }
    m.heads[c]  = b.id
    return true
}
```

Release writes `m` and nothing else. It does not touch `data`. That is the whole
design decision behind the parallelism results in section 5.

### 3.5 Payload work

```text
fn fill(part: &DynBox<u64>, n: u64, v: u64)   writes(part)
    contract { requires n <= len_of(part); }
{
    for k in 0..n { invariant fk: k <= n; part[k] = v }   // k < n <= len_of(part): Rule 7, no test
}

fn checksum(part: &DynBox<u64>, n: u64) -> u64   reads(part)
    contract { requires n <= len_of(part); }
{
    s = 0
    for k in 0..n { invariant ck: k <= n; s = s +wrap part[k] }
    return s
}
```

### 3.6 The driver, which is P2 written out

```text
r = region_new()?

b1 = alloc_words(&r.meta, 8)?            // P2's "b1 = alloc(A, 64)" at word granularity
b2 = alloc_words(&r.meta, 8)?
b3 = alloc_words(&r.meta, 8)?

// P2: "fill(b1) || fill(b2) || fill(b3)  // must be permitted to overlap"
if b1.off + b1.size <= b2.off && b2.off + b2.size <= b3.off {      // see trace T6
    par {
        fill(&r.data[b1.off .. b1.off + b1.size], b1.size, 1)
        fill(&r.data[b2.off .. b2.off + b2.size], b2.size, 2)
        fill(&r.data[b3.off .. b3.off + b3.size], b3.size, 3)
    }
} else {
    fill(&r.data[b1.off .. b1.off + b1.size], b1.size, 1)
    fill(&r.data[b2.off .. b2.off + b2.size], b2.size, 2)
    fill(&r.data[b3.off .. b3.off + b3.size], b3.size, 3)
}

free_words(&r.meta, b2)                  // P2: "free(A, b2)"
b4 = alloc_words(&r.meta, 4)?            // P2: "b4 = alloc(A, 32)"; reuses b2's words

s1 = checksum(&r.data[b1.off .. b1.off + b1.size], b1.size)        // P2: "read(b1)"
free_words(&r.meta, b1)
free_words(&r.meta, b3)
free_words(&r.meta, b4)
```

---

## 4. Rule-by-rule trace at the interesting points

**T1. `Region`, `Meta`, `Hdr` and `Blk` are legal types.** Rule 1: "Every struct,
enum, tuple, array, slice-like value, Box, DynBox, and generic instantiation holds
only owned values. This is recursive and closed under wrapping." `Region` holds a
`DynBox<u64>` and a `Meta`; `Meta` holds a `DynBox<Hdr>`, an `array<u64, NCLASS>`
and two `u64`s; `Hdr` and `Blk` hold only `u64` and `u8`. No field is a reference,
so none of Rule 1's rejected forms appears. Rule 1's consequence — "any value can be
relocated by copying its bytes (memmove, realloc), because nothing inside it points
anywhere" — is what makes the whole design work: a block's identity is an offset, so
nothing needs fixing up when a `Region` is moved.

**T2. `Hdr` is copy, so a header read is one load and a header write is one store.**
Rule 8: "A type is copy, affine, or linear. Copy values may be duplicated."
`Hdr`'s fields are `u64` and `u8`, and the notation section states "`Int` and `u64`
are Copy". Nothing here is linear, so Rule 8 imposes no obligation on any exit path
and, per Rule 8, "at scope exit the compiler releases their memory recursively (Box,
DynBox) and runs no user code" — the region and its header array are freed at scope
exit of `r` with no destructor and no user code. A linear payload changes this; see
section 8.

**T3. The free-list head must be tested; the handle bound must not.** Inside
`alloc_words`, `id = m.heads[c]` is a `u64` read out of storage. Rule 11 states the
limit exactly: "There are no quantified facts over array elements ('for all i ...')
and no per-slot occupancy facts; occupancy that is determined by data is stored as
data (Rule 6, Rule 12)." So there is no fact "every value on a free list is a valid
header index", and `m.hdr[id]` needs `id < len_of(m.hdr)` from somewhere. It comes
from the dominating branch `if id < NHDR`, which Rule 11 admits as a "refinement
fact from a dominating branch", combined with `len_of(m.hdr) == NHDR` from the
`requires`. Rule 7 then reads: "Every index must be proved in bounds; when the proof
is unavailable the program tests the measure, which is ordinary data", with the
example "if i < len_of(buf) { use(&buf[i]) } // the test establishes the fact; one
compare, no trap". **This compare is not a cost**: a C++ size-class allocator writes
the same test as `if (head != nullptr)`, and `NIL == NHDR` was chosen so that the
one comparison serves both as the null check and as the bound proof.

By contrast, in `free_words` the bound `b.id < NHDR` arrives as a `requires` that the
caller discharges from `alloc_words`'s `ensures when Some: result.id < NHDR`. That
fact mentions no path, so Rule 11's invalidation clause — "A fact that mentions a
path is invalidated when that path is written" — never fires on it, and the fact
survives every intervening `alloc_words` and `free_words`, each of which writes `m`.
Had the bound been phrased `result.id < len_of(m.hdr)`, it would mention `m.hdr` and
die at the very next `writes(m)`, forcing one compare and one branch at every
release. This is the single most valuable idiom found in this task.

**T4. The two liveness tests are the price of Rule 16, and they are optional.**
`h.live == 0` and `h.gen != b.gen` in `free_words` implement Rule 16's stance: "A
stale index that is still in bounds names the current occupant of that slot: a logic
error, not a memory error. Programs that need to detect it keep a generation number
as data." A program that does not need detection deletes both tests, and what it
inherits is a logic error — a stale handle names a live block — not memory
corruption, because every word of `r.data` is an initialized `u64` (T1, Rule 12) and
because Rule 14 says "Addresses are not observable". C++ inherits undefined behaviour
in the same situation. So x1 is strictly better here at equal instruction count, or
two compares worse with detection.

**T5. Forming the payload reference costs nothing.** `p1 = &r.data[b1.off .. b1.off +
b1.size]` is a path by Rule 2: "A path starts at a local variable or a parameter and
continues through fields, `*` (Box content), `[i]` (index, Rule 7), or `[lo..hi]`
(range, Rule 7)." Rule 7's obligation is "part = &buf[lo..hi] // requires lo <= hi <=
len_of(buf)". The left half, `b1.off <= b1.off + b1.size`, holds for `u64`. The right
half needs `b1.off + b1.size <= len_of(r.data)`, and it decomposes into:

- `b1.off + b1.size <= NWORDS`, from `alloc_words`'s `ensures when Some: result.off +
  result.size <= NWORDS`. Constant-phrased, hence immortal (T3).
- `NWORDS == len_of(r.data)`, from `region_new`'s `ensures when Ok: len_of(result.data)
  == NWORDS`, and nothing in the program writes the path `r.data`. `fill` writes
  `r.data[lo..hi]`, a different path; Rule 3's content-write clause is the rule
  file's own statement that a write below a path is not a write of that path —
  "Writing the storage at p's path or below it (a content write) does not invalidate
  p" — and Rule 6 independently guarantees that `len_of` is "changed only by the
  built-in operations below", none of which appears here. See Gap 3 for the residue.

Both conjuncts are static, so the range formation is one address computation and no
test. **Zero cost against C++'s `base + off`.**

**T6. Parallel fills of three separately allocated blocks are refused without the
test.** Rule 13: "`par { A; B }` is accepted when A's write paths are disjoint from
B's read and write paths and vice versa, using the same path-overlap and
index/range-disjointness judgment as Rule 10." Rule 10 clause 1: "Two effects on
overlapping paths where at least one is a write must be proved disjoint (different
roots, or indices or ranges proved distinct); otherwise the call is rejected." The
three write paths are `r.data[b1.off .. b1.off+b1.size]`, `r.data[b2.off ..]` and
`r.data[b3.off ..]`. Same root `r.data`, so they must be **ranges proved distinct**.

Where would that proof come from? Only from the allocator, and the allocator cannot
state it. The fact it would have to export is "the returned block overlaps no live
block", which quantifies over the live set — a per-slot occupancy fact, which Rule 11
forbids in the sentence quoted in T3. Nor can a batch helper export "block `w` sits at
`base + w*sz`", because that is a quantified fact over the elements of the returned
array, forbidden by the same sentence.

So the naked `par` over `b1, b2, b3` is **rejected**, and the best rewrite is the
dominating test written in the driver: `if b1.off + b1.size <= b2.off && b2.off +
b2.size <= b3.off`. Rule 11 admits it as a "refinement fact from a dominating branch",
and the third pair — `b1` against `b3` — follows by affine chaining, which Rule 11
admits as "affine comparisons over measures and integer values": from `b1.off +
b1.size <= b2.off`, `b2.off <= b2.off + b2.size` (a `u64` fact) and `b2.off + b2.size
<= b3.off`, the checker derives `b1.off + b1.size <= b3.off`. **k blocks therefore
cost k-1 tests, not k(k-1)/2**, once the handles are in ascending offset order.

**T7. Release does not invalidate any live payload reference.** Rule 3: a reference's
validity "is established when p is formed and invalidated when any proper prefix of
p's path is written, moved out of, replaced, or freed, by a statement or by a call."
Rule 10 clause 3 restates it for calls: "A live reference outside the call whose path
has a proper prefix among the call's write paths becomes invalid after the call."
The proper prefixes of `r.data[b1.off .. b1.off + b1.size]` are `r.data` and `r`. The
write path of `free_words(&r.meta, b2)` is `r.meta`. `r.meta` is neither. So `p1`
survives the release of `b2`, which is exactly P2's requirement that "nobody holds A
exclusively between operations" and that releasing one block leaves the others
usable. **Satisfied at zero cost.** The same reasoning is what makes `free_words`
overlap payload work in section 5.

The same rule is also the source of this task's one genuine failure; see section 7.

**T8. Allocation failure discharges P18 without a trap.** Rule 14: "Allocation returns
a `Result` and never traps", and Rule 5's line "b = Box::new(Node { ... })? //
allocation can fail: Result<Box<Node>, Oom>". `region_new` returns `Result` and
propagates with `?`. The user-level `alloc_words` returns `Option`, and both failure
arms — region exhausted, header array exhausted — return `None` having written
nothing, so P18's "the arm must free nothing twice and leave earlier blocks owned"
holds by construction: earlier blocks are `Blk` values in caller locals, untouched.
P18's "the declared budget is proved or refused" is the pair `m.front + sz <= NWORDS`
and `m.nhdr < NHDR`, both tested, both matching a C++ arena's own tests.

**T9. Separate compilation does not weaken anything.** A caller that receives `&Meta`
from elsewhere carries the invariant in its own `requires`/`ensures` and passes it
down. Rule 10's last paragraph — "Function-typed parameters carry a full signature
with its own row and contract, and a call through one uses that row. Recursion is
checked through contracts, never by unfolding bodies" — means the allocator's bodies
are never inspected at a call. Every fact used above crosses a signature: the
constant-phrased bounds, the invariant pair, and the failure routing through `ensures
when Some`. **No cost, and no loss relative to a whole-program analysis**, because
nothing in the design relies on one.

---

## 5. Parallelism: what the rules admit and what they refuse

All judgments below are Rule 13 with Rule 10's overlap test. Rule 14 removes the
allocator itself from the picture — "Allocation and release carry no effect entry and
never make two parallel arms conflict" — but note carefully that this privilege
belongs to the *trusted base's* heap, not to `alloc_words`; the user-level allocator's
release declares `writes(m)` and does conflict. Section 7 makes that the headline
counterexample.

### 5.1 Admitted at zero cost

| Combination | Why the rules admit it |
|---|---|
| `par { fill(&r.data[lo..mid], ...); fill(&r.data[mid..hi], ...) }` — splitting **one** block among workers | Adjacent ranges from loop arithmetic; Rule 13's own example `kernel(&v[0..mid], &out[0..mid]); kernel(&v[mid..n], &out[mid..n])   // disjoint ranges: accepted`. The disjointness is affine in the loop index, so no test. |
| `par { s1 = checksum(&r.data[R1], ..); s2 = checksum(&r.data[R2], ..) }` for **any** two ranges, even overlapping | Rule 13: "Read/read overlap is allowed." No disjointness obligation at all. Better than the reasoning a C++ author must do by hand. |
| `par { free_words(&r.meta, b2); fill(&r.data[b1.off .. b1.off+b1.size], ..) }` | `r.meta` and `r.data[...]` have **different roots** in Rule 10's sense: Rule 10's example `two(&a.x, &a.y)  // accepted: distinct fields`. Metadata maintenance overlaps payload work with no lock and no atomic. |
| `par { alloc_words(&r.meta, 8); fill(&r.data[b1.off .. b1.off+b1.size], ..) }` | Same reason — **subject to Gap 2**, which asks whether an operation that names no measure of `data` may really declare `writes(m)` alone. The design deliberately keeps `NWORDS` a constant and `front` inside `Meta` so that `alloc_words` never mentions `r.data`, which is what makes the answer "yes" under either reading of Gap 2. |
| `par { work(&r.data[b1..]); work(&r.data[b2..]) }` where `b1`, `b2` come from one **bump** epoch with no intervening release | The handles' offsets are monotone by construction in the caller's own loop, so the disjointness is affine, exactly as in row 1. Reuse is what destroys this, not allocation. |

### 5.2 Admitted with a hoisted runtime test

| Combination | Cost |
|---|---|
| `par { fill(b1); fill(b2); ...; fill(bk) }` for k blocks drawn from a **reusing** free list | k-1 compares and one branch, hoisted out of the parallel region (T6). Requires the handles sorted by offset first; a sort of k handles, k ≤ worker count. Against C++/Rust: k-1 compares and a k-element sort that neither pays. For blocks of any nontrivial size this is below measurement noise; for k = 64 and 4 KiB blocks it is ~63 compares against ~32768 stores. |

### 5.3 Refused

| Combination | Rule | Best rewrite and its residual cost |
|---|---|---|
| `par { alloc_words(&r.meta, 8); alloc_words(&r.meta, 8) }` | Rule 13 with Rule 10 clause 1: `writes(m)` against `writes(m)`, same root, cannot be proved disjoint from itself. | Per-worker allocators: `metas: DynBox<Meta>` with `par { alloc_words(&metas[i], 8); alloc_words(&metas[j], 8) }` for loop-derived `i != j`. Rule 10's `two(&v[i], &v[j])  // accepted only with the fact i != j` is discharged by the loop indices, so **zero runtime cost**. This is tcmalloc's per-thread cache, reached by the rules rather than by convention. Residual cost: each worker's region is a fixed slice of the reserve, so a worker that exhausts its slice fails while another's is idle — fragmentation, a footprint cost, not an instruction cost. |
| `par { free_words(&r.meta, b2); free_words(&r.meta, b3) }` | Same. | Same per-worker rewrite, **provided every block is released by the worker that allocated it**. |
| `par { alloc_words(&r.meta, 8); free_words(&r.meta, b2) }` | Same. | Same. C++ pays a lock or a CAS for exactly this; x1 refuses it statically and the rewrite is the one a fast C++ allocator already uses. |
| **Cross-worker release**: worker `j` releases a block worker `i` allocated | Not a rule rejection — it is inexpressible. The block must reach `metas[i]`, and the only mechanisms are an atomic remote-free list or a queue. "channels or atomics" are listed under **Not in this candidate**, and the channel primitive is explicitly **Deferred**: "A channel primitive in the trusted base (ownership-transfer queue) for producer/consumer pipelines." | Batch the foreign handles into a per-worker `DynBox<Blk>` and drain them at the next fork-join barrier. Per operation this is **cheaper** than jemalloc's one atomic CAS — one bounds-tested `push_nogrow` against one CAS. The loss is not throughput: it is that the block's words stay reserved until the barrier (peak footprint grows by one epoch of cross-worker frees) and that a workload with no barrier — a long-running server whose threads migrate objects — has nowhere to drain. That is the one shape of this task x1 cannot express. |

### 5.4 Why release conflicts, stated plainly

Release conflicts with allocation and with other releases for one reason and one
reason only: **they write one path.** Rule 13 needs A's write paths disjoint from B's
write paths, and `r.meta` is not disjoint from `r.meta`. This is not an artifact; the
free list is genuinely shared mutable state and the C++ allocator protects it with a
lock or an atomic. What x1 adds is that the refusal is static and the standard
rewrite — split the metadata per worker — is admitted at zero cost.

Release does **not** conflict with payload work on other blocks, and that is a real
win: metadata is a sibling field, so Rule 10's different-roots clause clears the pair
outright. A C++ allocator with a global lock serializes `free()` against nothing, but
a C++ allocator with an in-band header writes into the freed block's first words, and
a reader that still holds a pointer into that block races. x1's out-of-line header
makes the same overlap provably absent.

The price of the out-of-line header is section 6's line R4: `NHDR` headers of 40
bytes each, against dlmalloc's 8-to-16-byte in-band header, and one cold cache line
for the header instead of the payload's own line.

---

## 6. Cost table against C++ and Rust

Baselines: **C++** = a size-class allocator of the tcmalloc shape over a reserved
`mmap` region, in-band free-list links, no double-free detection. **Rust** = a slab
or bump allocator whose interior is `unsafe` and which hands out `&mut [u64]`; the
borrow checker then gives disjointness of two live blocks at zero runtime cost,
which is the only place Rust beats x1 structurally.

| # | Operation | x1 | C++ | Rust (unsafe interior) | Delta |
|---|---|---|---|---|---|
| R0 | Region construction | `DynBox::filled(NWORDS, 0)` writes 8 MiB of zeros | `mmap` supplies zero pages lazily | same as C++ | **8 MiB of eager stores, once.** Unavoidable: Rule 12 admits no uninitialized slot ("a 'logically empty' slot holds a valid value"). Amortized over the region's life this is one pass; for a short-lived arena it is the dominant cost. |
| R1 | `alloc_words`, reuse path | 1 load `heads[c]`, 1 cmp/branch, 1 header load (40 B), 2 stores, build `Blk` | 1 load of the list head, 1 cmp/branch, 1 dependent load of the in-band next, 1 store | as C++ | **+1 header store, +32 B touched.** The cmp/branch is parity (`id < NHDR` doubles as the null check). |
| R2 | `alloc_words`, carve path | 2 cmp/branch, 1 header store, 2 stores | 1-2 cmp/branch, 1 header store | as C++ | **+1 cmp/branch** for the header-array bound, which is P18's declared budget and has no C++ counterpart because C++ has no budget. |
| R3 | Returning the handle | `Blk` is 32 B: on SysV this returns through memory, +4 stores +4 loads | 8-byte pointer in a register | 16-byte `&mut [u64]` in two registers | **+4 stores +4 loads**, removable: pack to `Blk { id: u64, tag: u64 }` with `off` in the low 20 bits of `tag` and `gen` in the high 44, giving 2 registers and **+2 ALU ops per block acquisition** to unpack. Use the packed form. |
| R4 | Metadata footprint | 40 B per header, out of line, `NHDR` of them reserved up front | 8-16 B per block, in band, allocated with the block | 8-16 B | **+24 to +32 B per block, and the full `NHDR * 40` reserved eagerly.** Forced: the out-of-line header is what buys row 3 of table 5.1. |
| R5 | `free_words` | 1 header load, 2 cmp/branch (detection), 1 header store, 1 head store | 2 stores | 2 stores | **+1 load +2 cmp/branch.** Drop detection and it is +1 load; the header must be read for `size` in either case. The bound `b.id < NHDR` costs **zero** because of the constant-phrasing idiom (T3). |
| R6 | Forming a block reference | address computation only | pointer already held | pointer already held | **zero** (T5). |
| R7 | Payload store of a `u64` | 1 store; the loop bound is proved by the `invariant`, no per-element test | 1 store | 1 store | **zero.** |
| R8 | Payload store of a `u8`/`u16`/`u32` field into the `u64` region | load, mask, shift, or, store — 5 ops | 1 store | 1 store | **+4 ops.** No rewrite: the candidate has no cast between a `DynBox<u64>` range and a `DynBox<u8>` range, and "slice types as first-class values" is under **Not in this candidate**. Choosing a `DynBox<u8>` region inverts the problem: bytes at parity, every `u64` field at 8 loads + 7 shifts + 7 ors. **A region serves exactly one scalar width at parity.** |
| R9 | Payload holding mixed scalar kinds (`u64` and `f64` in one object) | one region per kind, so one object spans two regions | one object, one cache line | one object, one cache line | **+1 cache line and +1 dependent stream per object in a cold traversal.** Alternative: a `DynBox<Cell>` region with `enum Cell { I(u64), F(f64) }`, which costs +1 tag load, +1 branch and +8 B per scalar, since Rule 11 supplies no per-slot fact that would let the tag check be elided. Both are real; neither is removable. |
| R10 | Payload holding a typed object of the caller's choosing | **not expressible** | `void*` plus placement new | `MaybeUninit<T>` plus `unsafe` | Not a cost line, a capability line. Section 8. |
| R11 | Parallel fill of k blocks from a reusing list | k-1 cmp/branch + a k-element sort, hoisted | 0 | 0 | **+k-1 compares per parallel region.** Zero if the k blocks come from one bump epoch, or if the k blocks are one block split by range arithmetic. |
| R12 | Concurrent alloc/free from k workers | per-worker `metas[i]`; zero instructions beyond the single-worker path | per-thread cache, same | same | **zero** on the fast path; **fragmentation** across per-worker reserves. |
| R13 | Cross-worker release | batch to a `DynBox<Blk>` and drain at a barrier: 1 bounds-tested push | 1 atomic CAS onto a remote free list | 1 atomic CAS | **cheaper per operation, but requires a barrier.** A barrier-free workload is inexpressible. |
| R14 | Stale-handle detection | 1 load + 1 cmp + 1 branch **per block acquisition**, not per element access, because the reference is then a name for a path (Rule 2) | undefined behaviour | undefined behaviour inside the allocator | **+3 ops per acquisition** for a guarantee neither baseline offers. Noise for a 4 KiB block; ~+60% on the access path for a workload of 4-word blocks touched once. |

Summary of the deltas that survive every rewrite I can construct: **R0** (eager
zeroing), **R4** (metadata footprint), **R8/R9** (one scalar width per region),
**R11** (k-1 compares for cross-block parallelism under reuse), **R13** (a barrier
for cross-worker release), and **R10** (not a cost, a missing capability).

---

## 7. The strongest counterexample: release cannot invalidate a reference into the released block

P2 states the requirement without qualification: "every old pointer into b2 must be
refused afterwards" and "a pointer into freed b2 is refused even though its address
is reused by b4". x1 accepts the program below.

```text
b2 = alloc_words(&r.meta, 8)?
p2 = &r.data[b2.off .. b2.off + b2.size]         // a live reference into b2's words

free_words(&r.meta, b2)                          // writes(r.meta)

b4 = alloc_words(&r.meta, 4)?                    // reuses the words b2 occupied
q4 = &r.data[b4.off .. b4.off + b4.size]

fill(p2, b2.size, 1)                             // accepted: writes through a released block
s  = checksum(q4, b4.size)                       // reads the words fill(p2, ...) just clobbered
```

Trace of the acceptance:

- Rule 3 lists the invalidating events exactly: validity is "invalidated when any
  **proper prefix** of p's path is written, moved out of, replaced, or **freed**, by a
  statement or by a call." `p2`'s proper prefixes are `r.data` and `r`.
- Rule 10 clause 3 restates it for the call: "A live reference outside the call whose
  path has a **proper prefix** among the call's write paths becomes invalid after the
  call." `free_words`'s only write path is `r.meta`.
- `r.meta` is not `r.data` and not `r`. `p2` is valid.
- The word "freed" in Rule 3 refers to the trusted base's heap operation (Rule 5: "A
  Box is affine: at scope exit the compiler releases its memory recursively"; Rule 14:
  "There is one heap, provided by the trusted base"). A user-level release frees
  nothing in that sense; it edits a `u64` field. Rule 3's invalidation machinery is
  therefore structurally blind to a user-written allocator's release.

Now the decisive part: **there is no effect a user-level release can declare that
would invalidate `p2`, short of one that invalidates every reference into the region.**

- Declaring `writes(r.data[b2.off .. b2.off + b2.size])` on `free_words` — the in-band
  header design — does not help. That path is *equal* to `p2`'s path, not a proper
  prefix of it, and Rule 3 excludes equality explicitly in the next sentence: "Writing
  the storage at p's path or below it (a content write) does not invalidate p." `p2`
  survives. It also costs the parallelism of table 5.1 row 3, because `r.data[b2...]`
  now overlaps every unproved fill range.
- Declaring `writes(r.data)` — the whole region — does invalidate `p2`, since `r.data`
  *is* a proper prefix of `p2`'s path. It equally invalidates `p1`, `p3` and every
  other live payload reference, and under Rule 13 it conflicts with every parallel
  fill, because `writes(r.data)` overlaps `writes(r.data[b1...])` and cannot be proved
  disjoint from it.

So the region design forces a dichotomy:

| Declared release effect | Invalidates the released block's references? | Parallel with payload work on other blocks? |
|---|---|---|
| `writes(m)` | **no** (and no other block's either) | **yes**, at zero cost |
| `writes(m), writes(r.data[off..off+size])` | **no** (equal path, not a proper prefix) | only with a hoisted disjointness test |
| `writes(m), writes(r.data)` | **yes** — but every block's | **no**, ever |

Neither endpoint is P2. That is the sharpest thing this task has to say about
candidate x1.

**Is the accepted program unsafe?** Not at the language's level. Every word of
`r.data` is an initialized `u64` (Rule 12, T1), no slot carries a tag (Rule 6: "No
slot ever carries a tag"), and Rule 14 makes addresses unobservable, so the aliased
write corrupts data, not memory. It is precisely the class Rule 16 admits: "A stale
index that is still in bounds names the current occupant of that slot: a logic error,
not a memory error." Note that Rule 16 says this about *indices*, and here it is a
*reference* that survives — Rule 3 was supposed to be the mechanism that kills stale
references, and it silently does not apply. The parallel version of the same program
is also safe: `par { fill(p2, ..); fill(q4, ..) }` still needs the section 5.2
disjointness test, and for a genuinely reused range that test evaluates false and
takes the sequential arm. **No race is introduced; only a sequential logic error.**

**The rewrite that does satisfy P2, and its cost.** Give each block its own `DynBox`
and hold them in a pool:

```text
struct Pool { blocks: DynBox<DynBox<u64>>, gen: DynBox<u64>, freeix: DynBox<u64> }

fn pool_alloc(p: &Pool, slot: u64, n: u64) -> Result<Bool, Oom>   writes(p.blocks[slot]), writes(p.gen[slot])
    contract { requires slot < NSLOT; requires len_of(p.blocks) == NSLOT; requires len_of(p.gen) == NSLOT; }
{
    d = DynBox::filled(n, 0)?                    // Rule 14: the trusted base allocates; no effect entry
    p.blocks[slot] = move d                      // replaces the old block; the old DynBox is released (Rule 6/8)
    p.gen[slot] = p.gen[slot] + 1
    return Ok(true)
}

fn pool_free(p: &Pool, slot: u64)   writes(p.blocks[slot]), writes(p.gen[slot])
    contract { requires slot < NSLOT; requires len_of(p.blocks) == NSLOT; }
{
    p.blocks[slot] = DynBox::filled(0, 0)?       // an empty block; the old one is released
    p.gen[slot] = p.gen[slot] + 1
}
```

Now `pool_free(&p, i)` writes `p.blocks[i]`, which **is** a proper prefix of a live
`&p.blocks[i][k]` — so Rule 3 and Rule 10 clause 3 invalidate exactly the references
into the released block, and a live `&p.blocks[j][k]` for a proved `j != i` survives
because `p.blocks[i]` is not among its prefixes. **This is P2 exactly**, including
"every old pointer into b2 must be refused afterwards", and it is a compile-time
refusal with a diagnostic, not a runtime check.

Parallelism in this form is also better: `par { fill(&p.blocks[i], ..); fill(&p.blocks[j], ..) }`
needs only Rule 10's `i != j`, which is affine in the loop indices when the caller
assigns slots from a counted loop — **zero runtime cost, no sort, no offset
arithmetic**, against the k-1 compares of the region form.

What it costs, exactly:

- **One trusted-base heap allocation per block** (~15-100 cycles) instead of a bump
  increment (~2 cycles) or a free-list pop (~4). Amortization is now the base
  allocator's job, and the program no longer controls byte reuse at all.
- **The reserved-memory property is gone.** P18's "arena A with budget B bytes" is no
  longer enforced by the allocator; the budget becomes `NSLOT` blocks of unbounded
  size. Enforcing bytes again requires a counter the program maintains as data, which
  is one add and one compare per allocation — cheap, but it is bookkeeping the region
  form got for free from `m.front <= NWORDS`.
- **Contiguity is gone.** k blocks are k independent cache streams instead of one
  ascending sweep. For a workload that allocates many small blocks and then walks them
  in allocation order, the region form's single stream is the point; the pool form
  loses it.
- **`pool_free` allocates.** The empty replacement `DynBox::filled(0, 0)` returns a
  `Result`, so release can fail. Rule 14 puts `Result` on every allocation without
  exception, and there is no zero-size exemption in the rule text. A release that can
  fail is a poor primitive; the honest repair is to keep one preallocated empty
  `DynBox` per pool and swap it in, which the rules admit only through an atomic
  in-place update (Rule 6) whose function consumes the old block. That works, and it
  is what a real implementation would write.

So the counterexample has a clean answer, at the price of delegating byte reuse to
the trusted base. **A program that must control its own bytes gets no invalidation; a
program that accepts the base's allocator gets invalidation and better parallelism.**

---

## 8. Typed payloads: the pool form, and the capability that is simply absent

The task says "reuse its space". For untyped words, section 3 does it. For a typed
payload the situation is different and worth stating precisely, because it is the
difference between "an allocator" and "an arena for one type".

**There is no cast.** Nothing in the candidate turns a range of `r.data` into a `T`.
Rule 7 enumerates what may be indexed — "an inline `array<T, N>` ..., a `DynBox<T>`
(Rule 6), a range reference into either, and a `const` table" — and the element type
is fixed at allocation by Rule 6 ("`DynBox<T>` owns one heap block of `cap_of` slots,
fixed at allocation"). "slice types as first-class values" is under **Not in this
candidate**. So a user-written general-purpose allocator that serves arbitrary types
out of one reserve is **not expressible at all**, at any cost. Byte reuse across types
is the capability that is lost.

What is expressible is Rule 16's pool, one per type:

```text
struct Slab<T> { slot: DynBox<Option<T>>, gen: DynBox<u64>, freeix: DynBox<u64> }

fn slab_alloc<T>(s: &Slab<T>, x: own T) -> Option<u64>   writes(s)
    contract { requires len_of(s.slot) == NSLOT; ensures len_of(s.slot) == NSLOT;
               ensures when Some: result < NSLOT; }

fn slab_free<T>(s: &Slab<T>, k: u64)   writes(s)
    contract { requires k < NSLOT; requires len_of(s.slot) == NSLOT; ensures len_of(s.slot) == NSLOT; }
{
    s.slot[k] = clear(s.slot[k])        // atomic in-place update (Rule 6); clear consumes the old value
    s.gen[k]  = s.gen[k] + 1
    push_nogrow(&s.freeix, k)
}

fn clear<T>(x: own Option<T>) -> own Option<T> { return None }
```

Points that matter:

- `Option<T>` is forced, not chosen. Rule 12 states it directly: "No place is ever
  partially moved: every path is either wholly present or the program cannot name it.
  Structures whose occupancy is decided by data keep that occupancy as data", with the
  worked line "slots: DynBox<Option<Entry>>   // non-Copy payloads: Option, using a
  null niche where the type has one". Cost: **one null check per access**, already
  recorded in the rule file's own "Known costs" list, plus a tag word where the type
  has no niche.
- `swap_remove` is the wrong primitive here even though Rule 6 offers it, because it
  moves the last element into slot `k` and silently changes another block's handle.
  For an allocator whose handles must be stable, the atomic in-place update is the only
  correct form.
- **A linear payload works.** If `T` is linear (Rule 8: "linear type File"), `Slab<T>`
  is linear by containment and Rule 6 obliges the program to drain it: "A DynBox whose
  element type is linear is itself linear (Rule 8) and must be truncated to zero by the
  program before it can go out of scope." `clear` would then have to consume the `T`
  explicitly rather than drop it. So an allocator over linear resources is expressible
  and its release obligations are checked — a capability neither C++ nor safe Rust
  gives. Cost: the drain loop, which C++ gets from destructors at the same instruction
  count.
- **Release invalidates the right references.** `s.slot[k] = clear(s.slot[k])` writes
  `s.slot[k]`, a proper prefix of any live `&s.slot[k].field`, so those references die
  by Rule 3, and `&s.slot[j].field` for a proved `j != k` survives. A bare `&s.slot[k]`
  is *not* invalidated, since the written path equals it rather than properly prefixing
  it — the same equality gap as section 7, smaller in blast radius.
- Parallelism: `par { work(&s.slot[i]); work(&s.slot[j]) }` on Rule 10's `i != j`,
  zero cost when the indices are loop-derived, one compare and one branch when they
  come out of data.

---

## 9. Rule gaps

Each is quoted exactly and left unresolved, per the protocol.

**Gap 1 — may a program-defined handle type be declared linear?** Rule 8: *"Linearity
is declared on external-resource types and propagates through aggregates: a struct,
array, Box, or DynBox containing a linear part is linear."* What is missing: whether
"external-resource types" is a restriction on what may be declared linear, or a
description of the intended use. It decides a lot here. If a program may write
`linear type Blk`, then `free_words(m, move b)` consumes the handle, double release
becomes a compile error, the generation field disappears from both the handle and the
header, and table 6 lines R5 and R14 drop to zero — **a guarantee neither C++ nor safe
Rust offers, at negative cost.** If it may not, Rule 16's generation-as-data is the
only route and those lines stand. I do not resolve it.

**Gap 2 — does reading a measure, or naming a path only inside a contract, require a
`reads` entry in the row?** Rule 9: *"A function body is checked against its own row:
every statement's effect and every callee's substituted row must be covered by the
declared row."* What is missing: whether evaluating `len_of(p)` is a statement effect,
and whether a `requires`/`ensures` clause that mentions `p` obliges an entry for `p`.
It decides table 5.1 row 4: if an operation that consults `len_of(r.data)` must declare
`reads(r.data)`, then `par { alloc_words(...); fill(&r.data[...], ...) }` is rejected by
Rule 13, because `reads(r.data)` overlaps `writes(r.data[b1...])` with one of them a
write. This task's design sidesteps the gap on purpose — `NWORDS` is a constant and
`front` lives in `Meta`, so no allocator operation ever names `r.data` — but the
sidestep is a workaround, not a resolution, and an allocator that sized itself from
`len_of(data)` at runtime would land squarely on it.

**Gap 3 — does writing a sub-path invalidate a fact that mentions the parent path?**
Rule 11: *"A fact that mentions a path is invalidated when that path is written (by
statement or call) unless the callee's `ensures` re-establishes it."* What is missing:
whether `writes(r.data[lo..hi])` counts as writing `r.data` for the purpose of facts.
Rule 3 answers the analogous question for *references* explicitly — "Writing the
storage at p's path or below it (a content write) does not invalidate p" — and Rule 11
carries no matching sentence. Under the strict-path reading the fact `NWORDS ==
len_of(r.data)` survives every fill and T5 is free; under the overlap reading it must
be re-established after every payload write, costing **one header load per block
acquisition**. Rule 6's "changed only by the built-in operations below" points strongly
at the strict reading, but Rule 11 does not say so and I do not decide it.

**Gap 4 — how is a range-reference parameter spelled, and where does its length come
from?** Rule 7: *"part = &buf[lo..hi]                  // requires lo <= hi <=
len_of(buf); len_of(part) == hi - lo"*, against **Not in this candidate**: *"slice
types as first-class values"*. What is missing: the declared parameter type that
`fill(&r.data[lo..hi], ...)` binds to, and whether `len_of(part)` inside the callee is
a compiler-known quantity or a runtime word the callee must be handed. This task passes
the length as a separate by-value parameter so the answer does not change any cost line
— a range reference then carries base and length like Rust's `&mut [u64]`, which is
parity — but a callee that performed a window operation on `part` would need the answer.

---

## 10. Result

**Rules consistent: yes.** Nothing in this task produced a contradiction between two
rules. Rules 1, 2, 3, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15 and 16 compose here without
conflict. What they produce instead is a forced dichotomy (section 7) and four
underspecified points (section 9) — withheld guarantees and silence, not
self-contradiction.

**Task achievable: with cost.** Concretely:

- Several blocks in use at once: **yes, zero cost.**
- Release one block independently, leaving the others usable: **yes, zero cost**
  (T7).
- Reuse the released space: **yes, zero cost** for a homogeneous region; **not
  expressible across types**, because the candidate has no cast (section 8, R10).
- Shared management state: **yes, zero cost**, and better placed than C++'s in-band
  header, which is what buys `free ∥ fill` (table 5.1 row 3).
- Parallelism between operations on different blocks: **yes**, free for sub-ranges of
  one block, free for reads, free for release against another block's payload work;
  **k-1 hoisted compares** for fills of k separately allocated blocks once reuse has
  destroyed offset monotonicity (T6, R11), because Rule 11 forbids the quantified
  non-overlap fact the allocator would need to export.
- Concurrent allocation and release: **refused on one `Meta`**, rewritten to
  per-worker `Meta`s at **zero cost**; **cross-worker release is not expressible**
  without a barrier, since atomics are excluded and channels are deferred (R13).
- P2's "every old pointer into b2 must be refused afterwards": **failed** by the
  region design at any declared effect (section 7), **satisfied** by the pool design
  at the price of one base-heap allocation per block, lost contiguity, and byte reuse
  delegated to the trusted base.

**Depends on Rule 16: yes, centrally.** Every handle in every program here is an
index or an offset with a generation kept as data, which is Rule 16 verbatim: "A pool
is a DynBox plus indices used as handles; an arena is a DynBox used with
`push_nogrow` as allocation and `truncate(0)` as reset. A stale index that is still in
bounds names the current occupant of that slot: a logic error, not a memory error.
Programs that need to detect it keep a generation number as data." If the owner
refuses Rule 16, this task has no rendering at all under the remaining rules: Rule 4
forbids returning a reference to a block ("It cannot be assigned into any aggregate
(Rule 1), cannot be returned"), Rule 1 forbids storing one, and an index handle is the
only remaining way to name a block across a call. **A refusal of Rule 16 is a refusal
of the task.**

**Verdict: `rejected-real-cost`.**

The token is chosen against the strongest program, not the ordinary one. The ordinary
allocator — one worker, homogeneous words, no stale-handle guarantee — is
`accepted-fine` and matches C++ instruction for instruction on the fast path. The
strongest program is P2's, and P2's explicit refusal requirement is not met by any
declarable release effect in the region form (section 7); the rewrite that meets it
gives up byte reuse to the trusted base and pays one heap allocation per block. That
is an unavoidable runtime cost for the task as stated, which is what
`rejected-real-cost` names.

Per-program verdicts, for the record:

| Program | Verdict |
|---|---|
| D1 region allocator, sequential, homogeneous words | `accepted-fine` |
| Parallel fill of sub-ranges of one block | `accepted-fine` |
| Parallel fill of k blocks from one bump epoch | `accepted-fine` |
| Parallel fill of k blocks after reuse | `rejected-real-cost` — k-1 hoisted compares plus a sort |
| Concurrent alloc/free on one `Meta` | `rejected-zero-cost` — per-worker `Meta`s cost nothing |
| Cross-worker release | `rejected-real-cost` — batch to a barrier; barrier-free is inexpressible |
| Release invalidating references into the released block (P2) | `accepted-unsafe` at the allocator's abstraction level — the rules accept a program P2 requires to be refused; memory safety is untouched (Rule 12, Rule 14, Rule 16) |
| `linear type Blk` to make double release a compile error | `undecided-rule-gap` (Gap 1) |
| `par { alloc_words(..); fill(other block) }` | `undecided-rule-gap` (Gap 2) if the allocator names `len_of(data)`; `accepted-fine` in the constant-phrased design used here |
| Typed heterogeneous blocks out of one reserve | `rejected-real-cost` — not expressible; one pool per type, R8/R9 |
