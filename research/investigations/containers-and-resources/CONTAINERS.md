# Containers: owners, views, and the facts that cross a call

> **Superseded in place by `DESIGN.md`.** The integrated containers-and-resources
> design is `DESIGN.md` beside this file; read it first. `DESIGN.md` is at its
> second draft, after falsifier round 1; this file has been brought to that draft
> and carries no rule text of its own.

The container half of the batch 0116 drafting round, reduced to the material
`DESIGN.md` does not carry: the goals and non-goals it was written against, the
parts of the fact discipline that are argument rather than rule, the migration of
the four corpus programs, and the unaware-writer walkthroughs. The laws, the
rules, the amendment register, the open questions and the whole
verified-versus-reasoned register moved; the map below says where.

Tree read: `batch/0116-containers-and-resources` at `main a40c7e70`,
`spec/kernel-spec.md` **v0.40 ACTIVE**. Bare four-digit line numbers are that
file. Nothing here is implemented.

Provenance of the decisions this file was built on, which it does not reopen: the
owner's design discussion of 2026-09-01 (container and backing separation; push
never grows; grow and reserve owner-level with explicit effect and typed failure;
system I/O over views; `par` via reserved disjoint ranges; the names `HeapVector`,
`FixedVector<T, N>`, `Span`, `MutSpan`, `AppendView`), and the sweep ledger's
**D1**, extracted beside this file in `EVIDENCE-sweep-D1.md`.

## What moved to `DESIGN.md`

```text
| this file's section                  | where it now lives                        |
|--------------------------------------|-------------------------------------------|
| 2. The laws (L1-L7)                  | DESIGN.md section 2, as L10-L15 and L4    |
| 3.1 [CNT]                            | DESIGN.md section 3.6                     |
| 3.2 [VIEW]                           | DESIGN.md section 3.7                     |
| 3.3 [CALL]                           | DESIGN.md section 3.9                     |
| 3.4 [SEQ]                            | DESIGN.md section 3.10                    |
| 3.5 [BLD]                            | deleted; see "What round 2 changed" below |
| 3.6 amendment register               | DESIGN.md section 3.13, merged            |
| 4.1 terms, 4.3 the absorb commit     | DESIGN.md [MSR-1] and [VIEW-6]            |
| the Pool / PoolVector seam           | DESIGN.md section 3.11, resolved          |
| 7. Open questions                    | DESIGN.md section 5, merged and renumbered|
| the verified/reasoned register       | DESIGN.md section 6, re-run and extended  |
```

## What round 2 changed, so this file is not read as current

`DESIGN.md`'s first draft was falsified in four passes, and five of its decisions
did not survive. The snippets below are written against the surviving design; if
one contradicts `DESIGN.md`, `DESIGN.md` wins.

- **`Builder<'r, T>` and the `[BLD]` family are deleted.** Their coverage
  certificate certified an index range rather than a set of writes, so a
  conditional or broken fill committed raw slots; their abandonment release could
  not be written without the per-slot state L12 forbids; and the [PAR-2]
  permission they existed to obtain was denied by a condition they did not amend.
  A `par` fill is now `seq_filled` plus a `MutSpan` plus direct subscript writes,
  which is the shape [PAR-2] already permits.
- **`cap` and `room` are readable values.** L15 forbade reading them, on a
  rationale that only forbids reading the *allocator's* extent; the consequence
  was that every pop proved and no push did.
- **`len`, `cap` and `room` are one algebra with a standing identity**, defined
  once in the term language over any admitted place, subscripts included, and
  carrying affine value images. `len + room = cap` is a fact.
- **A view value holds its own loan**, exclusive for `MutSpan` and `AppendView`,
  ending when the value is consumed. The first draft said the formation borrow was
  the freeze; it is a call-scoped temporary and it is not.
- **`rebind p = e;` is spelled `set p = e;`** and its premise is that the target is
  dead; and affine liveness must agree at every join, which is what makes a
  compiler-derived release unconditional.

Two smaller corrections: `ResourceError` became the payload-carrying family
`OutOfMemory<V>`, `PoolExhausted<T>`, `NeedCapacity<T>`, each with one field
`rejected`; and `PoolVector<'p, T>` became `PoolVector<'p, T, N>`, with its
capacity fixed at pool reservation rather than at lease.

---

## 1. Goals and non-goals

### 1.1 Goals

| # | Goal | Test it must pass |
|---|---|---|
| G1 | One growable sequence per resource, with the resource in the type | Reading `HeapVector<u8>` in a signature tells you the callee needs `Heap`; reading `FixedVector<u8, 4096>` tells you it does not |
| G2 | One algorithm body, many backings | `checksum`, `sort`, `escape` are written once and called with a `FixedVector`, a `HeapVector`, and an `ArenaVector` without monomorphizing on the backing |
| G3 | No hidden growth and no hidden failure | Every allocation is at a source point whose operation names a provider and returns a typed failure |
| G4 | Heap-free programs can do I/O and can build sequences | `wfgrep`'s inner loop compiles with no `allocates(heap)` anywhere on its call graph |
| G5 | Facts that cross a call are readable from the signature | D1 is not expressible, and the reason is a rule about types, not about a repaired projection flag |
| G6 | Affine elements | A kernel object table `FixedVector<Handle, 64>` holds affine handles, is constructible, admits removal from the middle, and drops in a fixed order |
| G7 | No runtime tag, vtable, or allocator pointer anywhere in a container value | Layout of `FixedVector<u8, N>` is `N` bytes plus one `u64`; of `HeapVector<T>` one pointer plus two `u64` |
| G8 | `par` can fill a sequence | A counted loop can write disjoint slots of a filled owner through a `MutSpan` and receive [PAR-2] permission with no new refinement |

G6 and G8 are the two the first draft failed. G6 failed because `seq_fixed` gives
`len = 0` and nothing could then address a slot, and because a prefix has no
middle removal; `seq_filled` and `seq_take_at` are the answer. G8 failed because
`Builder`'s writes were callee-projected, which [PAR-2] denies outright; the
answer is not to refine [PAR-2] but to use the write shape it already proves.

### 1.2 Non-goals

| # | Non-goal | Why |
|---|---|---|
| N1 | A generic backing abstraction, allocator parameter, or `Vector<T, Policy>` | The owner's decision 6: the core is the contiguous initialized prefix; keyed and sparse containers are separate fixed families later. A policy parameter reintroduces the effect-polymorphism problem the 2026-09-01 refutation closed |
| N2 | Keyed, sparse, or segmented containers | Separate families, later, with their own occupancy design |
| N3 | A `SmallVector` (inline-then-heap) | It is a runtime sum over backings; it belongs to a later family and must not be smuggled in as a state of `FixedVector` |
| N4 | Effect polymorphism | Owner-level operations stay concrete per owner; only view-taking algorithms are shared |
| N5 | Deciding OOM policy | The resource half owns it. This file only guarantees that every allocation site is one named operation with a typed failure, so whatever OOM policy lands touches no algorithm |
| N6 | Iterator, closure, or trait-object protocols | No function values in the kernel [FN-5] |
| N7 | Retiring `array<T, N>` | `array` is the `len = cap = N` case and stays exactly as it is; `fir_filter.wf` is unchanged by this design |

---

## 2. The fact discipline

### 2.1 Measures and images

An owner's measure state is `(len, cap)`, with `room = cap - len` standing as an
identity rather than a computation, and its value state is the elements of its
initialized region. A view's state is `(the origin set, base, len, cap)`. Three
consequences, and the third is the one the first draft did not have:

- **A view's measures are not the owner's measures.** Advancing `len(a)` changes
  the view's state and changes the owner's element storage; it does not change
  `len(v)` until `absorb` publishes it. That is why the caller's `len(v)` fact is
  *correct*, not merely *retained*, across a callee that appends.
- **[ENT-5]'s invariant-conclusion rule applies unchanged.** An [INV-1] conclusion
  about `len(a)` is a theorem about the image captured when it was proved, and a
  later `seq_push` produces a new image rather than falsifying the theorem.
- **A measure carries an affine value image, and an operation's declared relation
  transfers it.** This is what makes a push provable inside a loop: `seq_push`
  declares `room(result) = room(view) - 1`, so a header invariant over `room` is
  preserved on the backedge with no writer premise. Without it, `room` is a
  quantity nothing relates to anything and the central operation of the surface is
  dischargeable only in straight-line code.

### 2.2 What dies when the owner is moved into a call

Everything supported by that binding's root, under [ENT-5] clause (c), at the
consume. The result binding is fresh and its facts are exactly the substitution of
the callee's declared relations. There is no frame rule, no "unchanged elsewhere"
inference, and no reconstruction of the caller's old facts on the new binding — a
caller who needs a measure on the result must read it or the callee must publish
it.

One sentence of [FN-9] has to change for this to be usable, and `DESIGN.md`
[CALL-4] changes it: a consuming use of an `own` parameter must not invalidate
that parameter's entry image. The image denotes the value the caller supplied, and
moving a value does not change it. Without the amendment `let acc = move out;`
deletes `cap(out)` from the clause that names it, and no helper that consumes its
owner can state anything about it — which is every helper in this design.

### 2.3 D1, re-derived

D1's program is reproduced in `DESIGN.md` section 1.2 and reproduces accepted at
this tip. Under this design it is **inexpressible four times over**, and each
refusal is independent:

1. **The signature does not typecheck.** `handle: &uniq 'a HeapVector<u8>` is
   refused by [CNT-9] at the `param` node. There is no `&uniq` container parameter
   to project a write through.
2. **The body has no operation.** Even granting the parameter, no `[SEQ]` row
   replaces a sequence's backing through a borrow; the only capacity-changing rows
   take the owner by value.
3. **The by-value rewrite is rejected at the use site, by type.** `move line`
   kills `len(line) = 10` at the consume ([CALL-2], [ENT-5](c)), the result is
   fresh with no published length, and the subscript's [OP-4] obligation is
   unproved. **This exact behavior is verified on today's compiler** (probe `p1`,
   residual `9_u64 < len(b)`).
4. **The `AppendView` rewrite cannot shrink at all.** A callee given
   `own AppendView<'o, u8>` can push, pop back to its own base, and truncate to
   its own base — never below it (L14, [VIEW-5]) — and cannot `absorb`
   ([VIEW-7]). The caller's `len(line)` fact is therefore live *and true* after the
   call, which is what [CALL-3] certifies.

The contrast with the located D1 mechanism is the point. The old repair question
was "should the `element` flag be `true` here?", answerable only by opening the
callee. The new question is "what is the parameter's declared type?", answerable
from the signature, which is L11.

One dependency of (4) is worth recording because it is invisible: [CALL-3]'s
length-fixed class is sound because a view descriptor is affine, so [SET-1]
refuses a `set` of it, and provenance-bearing, so [SET-2] refuses a `replace` of
it. `DESIGN.md` [VIEW-4] states that ground once. The day any view type becomes
region-free, [CALL-3] becomes D1.

---

## 3. Migration: the four programs

### 3.1 `wfgrep.wf` — `append_slice` becomes the canonical view algorithm

`wfgrep` already hand-rolls an `AppendView`: a `&uniq buffer<u8>` plus a
`filled: own u64` carried in and a new length returned. The design's job is to
make that one type.

**Before** (`tests/programs/wfgrep.wf:131`, abridged):

```wf
fn append_slice['d, 'm](destination: &uniq 'd buffer<u8>, filled: own u64, text: own slice<'m, u8>) -> result: own u64 reads(destination, text), writes(destination) contract {
  define capacity = len(deref(destination));
  define admitted = ile(filled, capacity);
  requires admitted;
  ensures ile(result, capacity);
} {
  let capacity = len(deref(destination));
  ...
  for @append (at in filled..capacity) {
    let taken = at -wrap filled;
    let done = ige(taken, length);
    if done { return at; }
    let byte = text[taken];
    set deref(destination)[at] = byte;
  }
  return capacity;
}
```

**After** (design text):

```wf-design
fn append_span['o, 'm](output: own AppendView<'o, u8>, text: own Span<'m, u8>) -> written: own AppendView<'o, u8> reads(output, text), writes(output) contract {
  requires ile(len(text), room(output));
  ensures ile(len(written), cap(output));
} {
  doc "Appends every byte of text into the view's spare window.";
  let count = len(text);
  let acc = move output;
  for @append (
    at in 0_u64..count,
    invariant spare: ige(room(acc) + at, count)
  ) {
    let byte = text[at];
    set acc = seq_push(view: move acc, value: byte);
  }
  return move acc;
}
```

Four things changed and one did not. The `filled` parameter is gone — it is
`len(output)`. The truncating fallback is gone — the requirement is proved by the
caller instead, which is L3's whole point: the function is total. The `&uniq
buffer` disappears with [CNT-9]. And the requirement is over `room`, not `cap`,
because "there is spare for this whole source" is then a two-term difference
bound. What did not change is the contract's shape: single-state, entry image on
the left, result on the right.

The header invariant is the load-bearing line, and it is worth reading twice. Its
base holds because `cap(acc) = room(output)` at formation and the `requires` bounds
the source. Its backedge holds because `room(acc)` decreases by one while `at`
increases by one, so the sum is unchanged. And `seq_push`'s own requirement
`igt(room(acc), Z)` follows from the invariant and the counted loop's `at < count`
by [MSR-4]'s two-premise family. The first draft printed
`invariant filled: ile(len(acc), at)`, a relation over `len` when the goal is over
`room`, and there was no rule connecting the two.

The caller:

```wf-design
region 'report {
  let view = seq_append_view<'report>(vector: &uniq 'report report);
  set view = append_span<'report, 'prefix>(output: move view, text: move prefix);
  set view = append_span<'report, 'reason>(output: move view, text: move reason);
  set length = absorb(view: move view);
}
region 'w {
  let published = seq_span<'w>(vector: &'w report);
  let done = publish_all<'w, 'w>(output: &uniq 'w err, source: move published, length: length);
}
```

`length` survives the region: its support is the ordinary binding and the commit
value, neither of which the region exit kills. The second region is legal because
the view's loan ends when `absorb` consumes it, not at the end of `'report`; under
the first draft's reading the owner stayed frozen and this shape needed a nested
region per phase.

**The heap consequence (G4).** `search_file` allocates two buffers:

```text
let input = buffer_new(4096_u64, 0_u8);      // before: allocates(heap)
let batch = buffer_new(8192_u64, 0_u8);
```

becomes

```text
let input = seq_filled<u8, 4096>(value: 0_u8);   // after: pure, inline in the frame
let batch = seq_filled<u8, 8192>(value: 0_u8);
```

**`seq_filled`, not `seq_fixed`, and that distinction is what makes the migration
reachable at all.** `seq_fixed` publishes `len = 0`; under [CNT-3] a zero-length
container is unreadable and unwritable until elements have been placed one at a
time, and a `MutSpan` formed on it has `len = 0` and names no bytes, so `read_at`
over a view could never fill it. The first draft had no filled constructor and
recorded the migration as blocked on [SYS-8]; the real obstacle was upstream of
that.

With [SYS-8] taking views, `search_file`, `publish_all` and `read_at` lose
`allocates(heap)` entirely. All eleven `buffer_new` calls in the program go the
same way, and `wfgrep` becomes a program with no heap on its call graph. It is not
the whole property: the bytes move into stack frames, and `walk` recurses per
directory level with 76,672 bytes of them, so the stack budget decides whether
this is a win. The container design's contribution is that the choice is now
**available** — today the type `buffer<T>` forces the heap.

### 3.2 `growable_vec.wf` — the hand-built vector disappears

**Before** (`tests/programs/growable_vec.wf:1`): a `struct ByteVec { buf:
buffer<u8>; fill: u64; }`, a 40-line `vec_push` that grows by
allocate-copy-`replace` through `&uniq 'a ByteVec`, and an `Overflow` error type.
This is **exactly D1's shape** — a callee replacing a buffer reached through a
`&uniq` actual — and it is the reason D1 was not a contrived probe.

**After** (design text): the struct, `vec_new`, and `vec_push` are all deleted.

```wf-design
command fn main(command.heap as heap: own Heap) -> status: own ExitStatus reads(heap), writes(heap), allocates(heap) {
  doc "Grows one heap vector once, fills it, and reports what it holds.";
  let total = 0_u64;
  let b = 7_u8;
  let empty = seq_heap<u8>();
  region 'h {
    let reserved = seq_reserve<'h>(vector: move empty, provider: &uniq 'h heap, additional: 20_u64);
    match reserved {
      Ok(value: v) => {
        region 'fill {
          let view = seq_append_view<'fill>(vector: &uniq 'fill v);
          for @seed (
            i in 0_u64..20_u64,
            invariant spare: ige(room(view) + i, 20_u64)
          ) {
            set view = seq_push(view: move view, value: b);
            set b = b +wrap 7_u8;
          }
          set total = absorb(view: move view);
        }
      }
      Err(error: refused) => {
        let recovered = move refused.rejected;
        return exit_status(code: 1_u8);
      }
    }
  }
  return exit_status(code: 0_u8);
}
```

The reserve is the only failure point, it happens once, before any element moves,
and it is where the 2026-09-01 refutation's item 3 said it must be. The twenty
pushes are total: `cap(view) = room(v) = 20` comes from `seq_reserve`'s published
`cap(v) = cap(empty) + 20` and the formation relation, and the header invariant
carries it around the loop. `bytes_append` collapses to one `append_span`.

The first draft printed `invariant room: ile(i, 20_u64)`, which is trivially true
and discharges nothing — it does not mention the view at all.

### 3.3 `percent_decode.wf` — the output parameter becomes a view

**Before** (`tests/programs/percent_decode.wf:1`):

```wf
fn decode['r](out: &uniq 'r buffer<u8>, src: own buffer<u8>) -> result: own u64 reads(src), writes(out) contract {
  define output_length = len(deref(out));
  define source_length = len(src);
  define sufficient = ige(output_length, source_length);
  requires sufficient;
} { ... set out[output_index] = byte; ... }
```

**After**:

```wf-design
fn decode['o, 's](out: own AppendView<'o, u8>, src: own Span<'s, u8>) -> written: own AppendView<'o, u8> reads(out, src), writes(out) contract {
  requires ige(room(out), len(src));
  ensures ile(len(written), cap(out));
} { ... set acc = seq_push(view: move acc, value: byte); ... }
```

This is the migration that has the sharpest remaining cost, and it is a join
problem rather than an atom problem. `decode` advances its output on three of four
paths and leaves it alone on the fourth. Today `output_index` is an ordinary local
and P19's join rule admits an unchanged-versus-`+1` image. Under this design the
advancing thing is `room(acc)`, which is not a binding — but it **is** a value
image ([MSR-3]), and the images on the two arms share a coefficient vector and
differ only in their constant, so [ENT-6]'s join gives the common form plus one
delta atom over `[-1, 0]` and the header invariant is re-established. That is
exactly P19's second accepted body shape, one level down.

The first draft claimed the invariant "becomes `ile(len(acc), input_index)` and
does the same work". It does not: the goal is over `room`, and the relation between
the two was declared not to be a fact. It is a fact now, and that is what makes
this program writable.

The caller passes a `Span` of its input, which may be a `FixedVector`, a
`HeapVector`, or an I/O buffer — the same body serves all three (G2), where today
the parameter type `own buffer<u8>` forces heap.

### 3.4 `fir_filter.wf` — unchanged

`array<f64, 8>` in a struct field, written by subscript through the owner. No
length, no capacity, no view. [CNT-10] keeps it exactly as it is. It is in this
list to show the design's floor: a program that needs no sequence state pays
nothing for one.

---

## 4. From an unaware writer to an accepted program

Four walkthroughs. Each starts from what a writer who has not read this file would
naturally write. Every rule id cited below exists in `DESIGN.md` section 3.

### 4.1 Push without a capacity proof

```wf-design
region 'fill {
  let view = seq_append_view<'fill>(vector: &uniq 'fill out);
  set view = seq_push(view: move view, value: byte);
}
```

```text
Semantics/Source [SEQ-5]: UndischargedOperationDomain
  operation: seq_push
  residual: "Z < room(view)"
  at line 3 in "  set view = seq_push(view: move view, value: byte);"
  mechanical_fix: seq_push never grows a sequence. Establish the residual before
    the push -- reserve on the owner before forming the view (seq_reserve),
    dominate the push with a branch on room(view), or state a header invariant
    over room(view) that carries it around a loop -- or use seq_try_push, which
    returns the value when the window is full.
```

Four repairs, each a real program:

```text
// (a) prove it once, outside                (b) dominate it with a branch
let ready = seq_reserve<'h>(                 let spare = room(view);
  vector: move v,                            let more = igt(spare, 0_u64);
  provider: &uniq 'h heap,                   if more {
  additional: n);                              set view = seq_push(...);
                                             }

// (c) carry it around a loop                (d) do not prove it
for @fill (                                  let (rest, leftover) =
  i in 0_u64..n,                               seq_try_push(view: move view, value: byte);
  invariant spare: ige(room(view) + i, n)    set view = move rest;
) {                                          match leftover {
  set view = seq_push(...);                    None() => { }
}                                              Some(value: unplaced) => { ... }
                                             }
```

The first draft's version of this message named three repairs, of which two could
not be written — the header invariant did not parse and a branch on `seq_len` does
not reach a goal over `room` — and it cited `[SEQ-4]`, which is `len`, with a
residual `len(view) < cap(view)` that is the stated requirement only under an
identity the draft declared not to be a fact. All four above are writable and all
four discharge.

### 4.2 Using a view after the owner moved

```wf-design
region 'r {
  let view = seq_mut_span<'r>(vector: &uniq 'r data);
  let elsewhere = consume(vector: move data);
  let sorted = sort<'r>(items: move view);
}
```

```text
Semantics/Source [OWN-5]: MoveOfBorrowedPlace
  root_class: "owner held by a live exclusive view"
  at line 3 in "  let elsewhere = consume(vector: move data);"
  loan: created at line 2 by seq_mut_span, live until the view value is consumed
  mechanical_fix: a view value holds an exclusive loan on its owner for its whole
    life [VIEW-2]. Finish the view's work -- consume it -- before moving,
    dropping, or growing the owner.
```

The diagnostic is [OWN-5]'s existing one, and the repair is to order the
statements rather than to close a region:

```wf-design
region 'r {
  let view = seq_mut_span<'r>(vector: &uniq 'r data);
  let sorted = sort<'r>(items: move view);
}
let elsewhere = consume(vector: move data);
```

The loan ending at the consume rather than at the end of `'r` is a round-2 change,
and it is the one that makes this repair one line rather than a nested region per
phase.

### 4.3 Trying to grow through a view

```wf-design
fn collect['o, 's, 'h](output: own AppendView<'o, u8>, source: own Span<'s, u8>, heap: &uniq 'h Heap) -> written: own AppendView<'o, u8> ... {
  set output = seq_reserve<'h>(vector: move output, provider: &uniq 'h deref(heap), additional: 64_u64);
```

```text
Semantics/Source [SEQ-0]: InvalidReceiver
  no seq_reserve row has receiver AppendView<'o, u8>
  at line 2 in "  set output = seq_reserve<'h>(vector: move output, ...);"
  mechanical_fix: a view never grows [CNT-8, L4]: it does not own the backing and
    carries no provider. Either strengthen this function's requirement so the
    caller reserves (requires ile(len(source), room(output))), or return a
    NeedCapacity outcome and let the owning shell reserve and re-form the view.
```

The second repair is the resumable-core shape the 2026-09-01 discussion settled
on, and it is what makes one algorithm serve a growing and a fixed container:

```wf-design
fn decode_chunk['o, 's](output: own AppendView<'o, u8>, source: own Span<'s, u8>) -> (written: own AppendView<'o, u8>, consumed: own u64, outcome: own ChunkOutcome) ...

// heap shell: NeedCapacity -> absorb, reserve, re-form, continue from consumed
// fixed shell: NeedCapacity -> report Full
```

Only the shell is backing-specific; `decode_chunk` is written once. Note that the
result is a **three-element** list: the view, how far the source was consumed, and
the outcome. The first draft named this repair and wrote no example, and a writer
told to restructure into a resumable core with no example does not find the shape.

### 4.4 Two containers, one function

```wf-design
fn partition['i](accepted: own HeapVector<u8>, rejected: own HeapVector<u8>, input: own Span<'i, u8>) -> result: own HeapVector<u8> ...
```

There is **no diagnostic here**, and that is the entry's whole content. Affinity
means a consumed owner may be dropped: probe `q16` takes two `own buffer<u8>`
parameters, returns one, drops the other, and is **accepted** today. So the
compiler will happily accept a function that silently destroys its caller's second
container, and the first draft's walkthrough — which quoted an
`[OWN-1] ConsumedOwnerNotReturned` rejection that exists in no rule — taught a
writer to expect a guardrail that is not there.

The advice is therefore advice, and it has two halves. If the function changes
neither container's length, pass two `MutSpan`s and keep both owners in the
caller; nothing crosses the boundary and no length fact dies. If it does change
them, return every owner it consumes, which is [CALL-5]'s reason to exist:

```wf-design
fn partition['i](accepted: own HeapVector<u8>, rejected: own HeapVector<u8>, input: own Span<'i, u8>) -> (kept: own HeapVector<u8>, dropped: own HeapVector<u8>) ... { ... }

let (kept, dropped) = partition<'i>(accepted: move a, rejected: move r, input: move view);
```

One trap this entry should name because nothing else does: if the two results had
been views of the same type and the same formal region, each would alias every
such parameter, and `DESIGN.md` [VIEW-10] makes that a declaration error rather
than a discovery.

---

## 5. Evidence

The verified-versus-reasoned register that stood here has moved to `DESIGN.md`
section 6, where every probe was re-run and eleven more were added, including the
three that establish why a view value must hold its own loan and the three that
establish why a helper cannot lend a provider onward without an [OWN-6]
amendment. `DESIGN.md` section 6.5 lists every falsifier finding against the first
draft and the rule that now refuses it.

Nothing in this file is verified. Every "after" snippet above is design text and
compiles nowhere.
