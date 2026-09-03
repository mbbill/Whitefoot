# Containers: owners, views, and the facts that cross a call

> **Superseded in place by `DESIGN.md`.** The integrated containers-and-resources
> design is `DESIGN.md` beside this file; read it first.

The container half of the batch 0116 drafting round, reduced to the material
`DESIGN.md` does not carry: the goals and non-goals it was written against, the
parts of the fact discipline that are argument rather than rule, the migration
of the four corpus programs, the four unaware-writer walkthroughs, and this
file's own verified-versus-reasoned register. The laws, the rules, the
amendment register and the open questions all moved; the map below says where.

Tree read: `batch/0116-containers-and-resources` at `main a40c7e70`,
`spec/kernel-spec.md` **v0.40 ACTIVE**. Bare four-digit line numbers are that
file. Nothing here is implemented.

Provenance of the decisions this file was built on, which it does not reopen:
the owner's design discussion of 2026-09-01 (container and backing separation;
push never grows; grow and reserve owner-level with explicit effect and typed
failure; system I/O over views; `par` via reserved disjoint ranges; the names
`HeapVector`, `FixedVector<T, N>`, `Span`, `MutSpan`, `AppendView`), and the
sweep ledger's **D1**, extracted beside this file in
`EVIDENCE-sweep-D1.md`.

## What moved to `DESIGN.md`

```text
| this file's section                  | where it now lives                        |
|--------------------------------------|-------------------------------------------|
| 2. The laws (L1-L7)                  | DESIGN.md section 2, as L10-L15 and L4    |
| 3.1 [CNT]                            | DESIGN.md section 3.5                     |
| 3.2 [VIEW]                           | DESIGN.md section 3.6                     |
| 3.3 [CALL]                           | DESIGN.md section 3.7                     |
| 3.4 [SEQ]                            | DESIGN.md section 3.8                     |
| 3.5 [BLD]                            | DESIGN.md section 3.9                     |
| 3.6 amendment register               | DESIGN.md section 3.12, merged            |
| 4.1 terms, 4.3 the absorb commit     | DESIGN.md [CNT-2] and [VIEW-6]            |
| the Pool / PoolVector seam           | DESIGN.md section 3.10, resolved          |
| 7. Open questions                    | DESIGN.md section 5, merged and renumbered|
```

Three things `DESIGN.md` decided differently, recorded here so this file is not
read as current: `rebind p = e;` became a reinitializing `set p = e;` whose
premise is that the target is dead ([CALL-7] there), which is the owner's own
spelling; `ResourceError` became the payload-carrying failure family
`OutOfMemory<V>`, `PoolExhausted<T>`, `NeedCapacity<T>`, `Full<T>`, `TooSmall`;
and `PoolVector<'p, T>` became `PoolVector<'p, T, N>`, with its capacity fixed
at pool reservation rather than at lease. The snippets below still use the older
spellings, and `Err(value: e)` where [PRE-1] declares `Err(error: e)`.

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
| G6 | Affine elements | A kernel object table `FixedVector<Handle, 64>` holds affine handles, with drop in a fixed order |
| G7 | No runtime tag, vtable, or allocator pointer anywhere in a container value | Layout of `FixedVector<u8, N>` is `N` bytes plus one `u64`; of `HeapVector<T>` one pointer plus two `u64` |
| G8 | `par` can fill a sequence | A counted loop can write disjoint reserved slots and publish one length at the join |

### 1.2 Non-goals

| # | Non-goal | Why |
|---|---|---|
| N1 | A generic backing abstraction, allocator parameter, or `Vector<T, Policy>` | The owner's decision 6: the core is the contiguous initialized prefix; keyed and sparse containers are separate fixed families later. A policy parameter reintroduces the effect-polymorphism problem the 2026-09-01 refutation closed |
| N2 | Keyed, sparse, or segmented containers | Separate families, later, with their own occupancy design |
| N3 | A `SmallVector` (inline-then-heap) | It is a runtime sum over backings; it belongs to a later family and must not be smuggled in as a state of `FixedVector` |
| N4 | Effect polymorphism | Owner-level operations stay concrete per owner; only view-taking algorithms are shared |
| N5 | Deciding OOM policy | `RESOURCES.md` owns it. This file only guarantees that every allocation site is one named operation with a typed failure, so whatever OOM policy lands touches no algorithm |
| N6 | Iterator, closure, or trait-object protocols | No function values in the kernel [FN-5] |
| N7 | Retiring `array<T, N>` | `array` is the `len ≡ cap ≡ N` case and stays exactly as it is; `fir_filter.wf` is unchanged by this design |

---

---

## 2. The fact discipline

### 2.1 Value images

An owner's value image is `(len, cap, the elements of [0, len))`. A view's value
image is `(the origin set, base, len, cap)`. Two consequences:

- **A view's image is not the owner's image.** Advancing `len(a)` changes the
  view's image and changes the owner's element storage; it does not change
  `len(v)` until `absorb` publishes it. That is why the caller's `len(v)` fact
  is *correct*, not merely *retained*, across a callee that appends.
- **[ENT-5]'s invariant-conclusion rule applies unchanged.** An [INV-1]
  conclusion about `len(a)` is a theorem about the image captured when it was
  proved, and a later `seq_push` produces a new image rather than falsifying the
  theorem.

### 2.2 What dies when the owner is moved into a call

Everything supported by that binding's root, under [ENT-5] clause (c), at the
consume. The result binding is fresh and its facts are exactly [ENT-3.S12]'s
substitution of the callee's [FN-9] relations. There is no frame rule, no
"unchanged elsewhere" inference, and no reconstruction of the caller's old
facts on the new binding — a caller who needs `len` on the result must read it
([SEQ-3]) or the callee must publish it ([CALL-4]).

### 2.3 D1, re-derived

D1's program, verbatim (§8 reproduces it accepted by today's compiler):

```wf
fn shrink['a](handle: &uniq 'a buffer<u8>) -> discarded: own buffer<u8> reads(handle), writes(handle), allocates(heap) {
  let smaller = buffer_new(2_u64, 0_u8);
  let old = replace deref(handle) = move smaller;
  return move old;
}

command fn main() -> status: own ExitStatus allocates(heap) {
  let line = buffer_new(10_u64, 0_u8);
  region 'r {
    let dropped = shrink<'r>(handle: &uniq 'r line);
  }
  let tail = line[9_u64];
  return exit_status(code: 0_u8);
}
```

Under this design it is **inexpressible three times over**, and each refusal is
independent:

1. **The signature does not typecheck.** `handle: &uniq 'a HeapVector<u8>` is
   refused by [CNT-8] at the `param` node. There is no `&uniq` container
   parameter to project a write through.
2. **The body has no operation.** Even granting the parameter, no `[SEQ]` row
   replaces a sequence's backing through a borrow; the only capacity-changing
   rows, [SEQ-14] and [SEQ-16], take the owner by value.
3. **The by-value rewrite is rejected at the use site, by type.** The nearest
   legal program is:

   ```text
   fn shrink(handle: own HeapVector<u8>, heap: &uniq 'h Heap) -> smaller: own HeapVector<u8> ...

   let smaller = shrink(handle: move line, heap: &uniq 'h heap);
   let tail = smaller[9_u64];        // rejected
   ```

   `move line` kills `len(line) = 10` at the consume ([CALL-2], [ENT-5](c)),
   `smaller` is fresh with no published length, and the subscript's [OP-4]
   obligation `9_u64 < len(smaller)` is unproved. **This exact behavior is
   verified on today's compiler** (probe `p1`, §8, residual `9_u64 < len(b)`).

4. **The `AppendView` rewrite cannot shrink at all.** A callee given
   `own AppendView<'o, u8>` can push, pop back to its own base, and truncate to
   its own base — never below it (L6, [VIEW-5]) — and cannot `absorb`
   ([VIEW-7]). The caller's `len(line)` fact is therefore live *and true* after
   the call, which is what [CALL-3] certifies.

The contrast with the located D1 mechanism is the point. The old repair
question was "should the `element` flag be `true` here?", answerable only by
opening the callee. The new question is "what is the parameter's declared
type?", answerable from the signature, which is L2.

---


---

## 3. Migration: the four programs

### 3.1 `wfgrep.wf` — `append_slice` becomes the canonical view algorithm

`wfgrep` already hand-rolls an `AppendView`: a `&uniq buffer<u8>` plus a `filled:
own u64` carried in and a new length returned. The design's job is to make that
one type.

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

```wf
fn append_span['o, 'm](output: own AppendView<'o, u8>, text: own Span<'m, u8>) -> written: own AppendView<'o, u8> reads(text), writes(output) contract {
  requires ile(len(text), cap(output));
  ensures ile(len(written), cap(output));
} {
  let count = len(text);
  let out = move output;
  for (at in 0_u64..count, invariant filled: ile(len(out), at)) {
    let byte = text[at];
    rebind out = seq_push(view: move out, value: byte);
  }
  return move out;
}
```

‡ **That `filled` invariant is not writable under [INV-1] as it stands**, whose affine
atoms are literals and "live own-mode integer values" — `len(out)` is neither.
It is what discharges `seq_push`'s requirement at every iteration
(`len(out) <= at < count <= cap(out)`), so the loop above is not merely
unproved today, it is unstatable. That makes [INV-1]'s atom vocabulary part of
Q6, not a separate concern.

Three things changed and one did not. The `filled` parameter is gone — it is
`len(output)`. The truncating fallback is gone — the requirement is proved by
the caller instead, which is L3's whole point: the function is total. The
`&uniq buffer` disappears with [CNT-8]. What did not change is the contract's
shape: single-state, entry image on the left, result on the right, [FN-9]
machinery unchanged.

The caller:

```wf
region 'report {
  let view = seq_append_view(&uniq 'report report);
  rebind view = append_span<'report, 'prefix>(output: move view, text: move prefix);
  rebind view = append_span<'report, 'reason>(output: move view, text: move reason);
  set length = absorb(view: move view);
}
region 'w {
  let published = seq_span(&'w report);
  let done = publish_all<'w, 'w>(output: &uniq 'w deref(err), source: move published, length: length);
}
```

`length` survives the region: its support is the ordinary binding and the
commit value, neither of which the region exit kills ([VIEW-6] step 4 states
the relation over the owner place `report`, not through the holder).

**The heap consequence (G4).** `search_file` allocates two buffers:

```text
let input = buffer_new(4096_u64, 0_u8);      // before: allocates(heap)
let batch = buffer_new(8192_u64, 0_u8);
```

becomes

```text
let input = seq_fixed<u8, 4096>();          // after: pure, inline in the frame
let batch = seq_fixed<u8, 8192>();
```

and with [SYS-8] taking views, `search_file`, `publish_all` and `read_at` lose
`allocates(heap)` entirely. All eleven `buffer_new` calls in the program go the
same way, and `wfgrep` becomes a program with no heap on its call graph —
progress toward the `RESOURCES.md` resource-closed property, reached without
changing one line of its matching logic. It is not the whole property: the
bytes move into stack frames, and `walk` recurses per directory level with
76,672 bytes of them, so the stack budget `RESOURCES.md` owns decides whether
this is a win. The container design's contribution is that the choice is now
**available** — today the type `buffer<T>` forces the heap.

### 3.2 `growable_vec.wf` — the hand-built vector disappears

**Before** (`tests/programs/growable_vec.wf:1`): a `struct ByteVec { buf:
buffer<u8>; fill: u64; }`, a 40-line `vec_push` that grows by allocate-copy-
`replace` through `&uniq 'a ByteVec`, and an `Overflow` error type. This is
**exactly D1's shape** — a callee replacing a buffer reached through a `&uniq`
actual — and it is the reason D1 was not a contrived probe.

**After** (design text): the struct, `vec_new`, and `vec_push` are all deleted.

```wf
command fn main(command.heap as heap: own Heap) -> status: own ExitStatus allocates(heap), writes(heap) {
  let total = 0_u64;
  let b = 7_u8;
  region 'h {
    let empty = seq_heap<u8>();
    match seq_reserve(vector: move empty, provider: &uniq 'h heap, additional: 20_u64) {
      Ok(value: v) => {
        region 'fill {
          let view = seq_append_view(&uniq 'fill v);
          for (i in 0_u64..20_u64, invariant room: ile(i, 20_u64)) {
            rebind view = seq_push(view: move view, value: b);
            set b = b +wrap 7_u8;
          }
          set total = absorb(view: move view);
        }
        ...
      }
      Err(error: problem) => { return exit_status(code: 1_u8); }
    }
  }
  ...
}
```

The reserve is the only failure point, it happens once, before any element
moves, and it is where the 2026-09-01 refutation's item 3 said it must be. The
20 pushes are total: `seq_push`'s requirement is `ilt(len(view), cap(view))`,
and `cap(view) >= 20` comes from `seq_reserve`'s published relation through
[VIEW-2]'s formation equality. `bytes_append` collapses to one `append_span`.

One honest cost: `rebind view = seq_push(...)` inside a counted loop needs the
loop-carried `len(view) < cap(view)` at every head, which is a header invariant
the writer states. Section 8 records that a `len`-anchored postcondition across
a loop **did not** discharge in my probe, so this is the shape most in need of
prover work; it is Q6.

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

```wf
fn decode['o, 's](out: own AppendView<'o, u8>, src: own Span<'s, u8>) -> written: own AppendView<'o, u8> reads(src), writes(out) contract {
  requires ige(cap(out), len(src));
  ensures ile(len(written), cap(out));
} { ... rebind acc = seq_push(view: move acc, value: byte); ... }
```

The `writes_behind_scan` header invariant `ile(output_index, input_index)`
becomes `ile(len(acc), input_index)` and does the same work: it is what keeps
`seq_push`'s requirement discharged inside the loop — and it runs into §5.1's
‡ exactly as `append_span` does, because `len(acc)` is not an [INV-1] atom
today (Q6). The caller passes a
`Span` of its input, which may be a `FixedVector`, a `HeapVector`, or an I/O
buffer — the same body serves all three (G2), where today the parameter type
`own buffer<u8>` forces heap.

### 3.4 `fir_filter.wf` — unchanged

`array<f64, 8>` in a struct field, written by subscript through the owner. No
length, no capacity, no view. [CNT-9] keeps it exactly as it is. It is in this
list to show the design's floor: a program that needs no sequence state pays
nothing for one.

---


---

## 4. From an unaware writer to an accepted program

Four walkthroughs. Each starts from what a writer who has not read this file
would naturally write.

### 4.1 Push without a capacity proof

```wf
region 'fill {
  let view = seq_append_view(&uniq 'fill out);
  rebind view = seq_push(view: move view, value: byte);
}
```

```text
Semantics/Source [SEQ-4]: UndischargedCapacityObligation
  residual: "len(view) < cap(view)"
  at line 2 in "  rebind view = seq_push(view: move view, value: byte);"
  mechanical_fix: seq_push never grows a sequence. Establish the residual
    before the push — reserve on the owner before forming the view
    (seq_reserve), state a header invariant that carries it around a loop,
    or dominate the push with a branch on seq_len — or use seq_try_push,
    which returns the value when the window is full.
```

Three repairs, each a real program:

```text
// (a) prove it once, outside                    (b) carry it around a loop
let v = propagate seq_reserve(                   for (i in 0_u64..n,
  vector: move v,                                    invariant room: ile(i, claimed)) {
  provider: &uniq 'h heap,                         rebind view = seq_push(...);
  additional: n);                                }

// (c) don't prove it
let (rest, leftover) = seq_try_push(view: move view, value: byte);
match leftover { Some(value: unplaced) => { ... } None() => { } }
```

### 4.2 Using a view after the owner moved

```wf
region 'r {
  let view = seq_mut_span(&uniq 'r data);
  let elsewhere = consume(vector: move data);
  sort<'r>(items: move view);
}
```

```text
Semantics/Source [OWN-5]: MoveOfBorrowedPlace
  root_class: "owner frozen by a live view"
  at line 3 in "  let elsewhere = consume(vector: move data);"
  loan: created at line 2 by seq_mut_span, live to the end of region 'r
  mechanical_fix: a view holds an exclusive loan on its owner for its whole
    region [VIEW-2]. Finish the view's work and let 'r end before moving,
    dropping, or growing the owner; or narrow 'r to the statements that use
    the view.
```

The diagnostic is [OWN-5]'s existing one — this design adds no exclusivity
rule, it only makes formation a loan (L1 plus [VIEW-2]). The repair is to close
the region:

```wf
region 'r { let view = seq_mut_span(&uniq 'r data); sort<'r>(items: move view); }
let elsewhere = consume(vector: move data);
```

### 4.3 Trying to grow through a view

```wf
fn collect['o](output: own AppendView<'o, u8>, source: own Span<'s, u8>) -> written: own AppendView<'o, u8> ... {
  rebind output = seq_reserve(vector: move output, provider: &uniq 'h heap, additional: 64_u64);
```

```text
Semantics/Source [OP-1]: InvalidOperation
  no seq_reserve row has receiver AppendView<'o, u8>
  at line 2 in "  rebind output = seq_reserve(vector: move output, ...);"
  mechanical_fix: a view never grows [CNT-7, L3]: it does not own the backing
    and carries no provider. Either strengthen this function's requirement so
    the caller reserves (requires ile(len(source), cap(output))), or return a
    NeedCapacity outcome and let the owning shell reserve and re-form the view.
```

The second repair is the resumable-core shape the 2026-09-01 discussion
settled on, and it is what makes one algorithm serve a growing and a fixed
container:

```text
fn decode_chunk['o, 's](output: own AppendView<'o, u8>, source: own Span<'s, u8>)
  -> (written: own AppendView<'o, u8>, outcome: own ChunkOutcome) ...

// heap shell: NeedCapacity -> absorb, reserve, re-form, continue
// fixed shell: NeedCapacity -> report Full
```

Only the shell is backing-specific; `decode_chunk` is written once.

### 4.4 Two containers, one function

```wf
fn partition['a, 'b](accepted: own HeapVector<u8>, rejected: own HeapVector<u8>, input: own Span<'i, u8>) -> result: own HeapVector<u8> ...
```

```text
Semantics/Source [OWN-1]: ConsumedOwnerNotReturned
  parameter "rejected" is consumed by this function and is not the result
  at line 1 in the complete param node
  mechanical_fix: an owner passed by value is dead in the caller [CALL-2].
    Return every owner this function consumes: write the result as a list —
    -> (accepted: own HeapVector<u8>, rejected: own HeapVector<u8>) — and bind
    it with let (a, r) = partition(...);
```

```wf
fn partition['i](accepted: own HeapVector<u8>, rejected: own HeapVector<u8>, input: own Span<'i, u8>)
  -> (kept: own HeapVector<u8>, dropped: own HeapVector<u8>) ... { ... }

let (kept, dropped) = partition<'i>(accepted: move a, rejected: move r, input: move view);
```

This is [CALL-5]'s reason to exist. Note the alternative the writer should
usually prefer, and which the diagnostic could name: if neither container's
length changes, pass two `MutSpan`s and keep both owners in the caller.

---


---

## 5. Verified versus reasoned

This register is kept as the draft's own evidence. Its cross-references (sections
3, 4.5, 5, 5.1, 7, 8, and the law numbers L1 to L7) name the sections this file
carried before `DESIGN.md` superseded them; the map in "What moved to
`DESIGN.md`" translates each one. `DESIGN.md` section 6 re-ran every probe below
against the gate compiler and records the verdicts obtained in that session,
together with five further probes that isolate Q6.

**Verified** means a compiler executed it. The binary is
`whitefootc`, built at the branch tip with
`cargo build --locked --offline --profile gate --bin whitefootc`. Probe sources
are scratch files, reproduced inline here so each verdict is re-checkable; no
timing figure from this machine appears anywhere in this file.

| probe | program | verdict | what it establishes |
|---|---|---|---|
| `d1` | D1's `shrink` through `&uniq buffer<u8>`, then `line[9_u64]` (§4.5) | **accepted**, exit 0 | D1 reproduces at this tip. The design's target is a live defect, not a hypothetical |
| `p1` | `fn passthrough(out: own buffer<u8>) -> own buffer<u8>`, then `b[9_u64]` | **rejected**, [OP-4] residual `9_u64 < len(b)` | [CALL-2] already holds mechanically: an owner passed and returned carries no length onto the result |
| `p6` | `fn observe['a](handle: &'a buffer<u8>) -> own u64`, then `line[9_u64]` | **accepted** | [CALL-1] already holds: a shared-borrow call kills nothing |
| `p7` | `set view[0_u64] = 1_u8;` on a `slice_of` result | **rejected**, [SET-1] `root_class: "slice view"` | Slices are read-only today; `MutSpan` is genuinely new capability, not a rename |
| `p4` | single-state `ensures ile(result, capacity)` with `define capacity = len(deref(destination))`, one element write | **accepted** | The entry-image contract shape [CALL-4] extends is real and works today |
| `p2` | `ensures ige(len(result), capacity);` | **rejected**, [GRAM-9] | `len(result)` does not parse today. [CALL-4] admission 2 is an amendment, correctly labelled |
| `p8` | `fn pair() -> (first: own u64, second: own u64)` | **rejected**, [GRAM-2] expected IDENT | Multi-return is new syntax |
| `p9` | `array_new<box<u64>, 4>(move cell)` | **rejected**, [OP-1] `InvalidOperation` | Affine elements have no construction route today; [CNT-4] is new capability |
| `p10` | `set a = take(b: move a);` for `a : own buffer<u8>` | **rejected**, [STOR-1] `AffineSetTarget`, whose fix text reads *"the right-hand side consumes the target root, so replace cannot commit into it"* | Half of [CALL-7]'s premise, from the compiler itself |
| `p11` | `let old = replace a = take(b: move a);` | **rejected**, [OWN-1] `UseAfterMove`, fix *"introduce a new `let` binding before reuse"* | The other half. Neither `set` nor `replace` can rebind an affine local from a call that consumed it, and a new `let` is what a loop body cannot use |
| `p3`, `p5` | `ensures ile(result, capacity)` proved across a counted loop, with a header invariant and a continuation `invariant_stmt` | **rejected**, [FN-9] `at - len(deref(destination)) <= 0` unproved — in `p5` even with **no writes at all** | Q6. The failure is not the element write and not the borrow: it is the connection from an [INV-1] conclusion to a `len`-anchored [FN-9] query across a loop |
| — | `tests/programs/wfgrep.wf` at this tip | **compiles**, exit 0 | The migration baseline in §5.1 is a program that builds today |

**Reasoned, not verified.** Everything else. Specifically:

- Every rule in section 3. No compiler has seen `FixedVector`, `HeapVector`,
  `ArenaVector`, `PoolVector`, `Span` as a distinct type, `MutSpan`,
  `AppendView`, `Builder`, `seq_*`, `absorb`, `rebind`, multi-return, or
  `cap(P)`.
- [CALL-7] in particular was **found by writing section 5's snippets, not by
  designing**: the first draft wrote `set view = seq_push(...)`. Its premise is
  now verified rather than reasoned (probes `p10`, `p11`) — neither `set` nor
  `replace` admits it — but the `rebind` statement itself is design text. Without it,
  L1's consume-and-return convention cannot be written into any loop, and every
  migrated program in section 5 is a loop. That the gap surfaced only at the
  migration snippets is the argument for keeping section 5 in this file.
- Every "after" snippet in section 5 and every repair in section 6. They are
  design text and were not compiled, because they cannot be.
- Every diagnostic in section 6. The wording follows [DIAG-1]'s single-rule,
  single-location, one-mechanical-fix discipline and the register of the
  existing diagnostics quoted in section 8's verified rows, but no compiler
  emits them.
- The claim that `wfgrep` becomes heap-free (§5.1, G4). Its only
  `allocates(heap)` sources are eleven `buffer_new` calls (verified by count at
  the branch tip) reaching three declared rows, all of which this design
  replaces with `seq_fixed<T, N>` — but the substitution was not performed and
  compiled, because [SYS-8] cannot take a view today. The claim also moves
  76,672 bytes of `walk`'s five buffers, 12,288 of `search_file`'s two, and
  6,656 of `main`'s four out of the heap and into stack frames, which is a
  **stack budget question `RESOURCES.md` owns**, not a free win: `names` alone
  is a 65,664-byte inline `FixedVector`, and `walk` recurses into itself once
  per directory level (line 1144).
- [BLD-4]'s claim that [PAR-2] needs exactly one refinement. It was checked by
  reading [PAR-2]'s conditions one at a time against the `Builder` shape; it was
  not checked by running the PAR ledger, which has no such shape to report on.

**The two places this design is weakest**, stated plainly.

[BLD-3]'s coverage certificate (Q5) is a shape rule standing in for a proof. It
admits one loop form and refuses every other, which is defensible — [PAR-2]
made the same choice — but it is the one rule here whose correctness rests on
an enumeration rather than on a law.

Q6 is worse, and it is the finding this file would lead with if it led with
one: the invariant that every migrated loop in section 5 needs cannot be
**written** under [INV-1]'s atom rule, and the postcondition it would support
did not discharge across a loop in probe `p5` even when it could be stated over
ordinary locals. No amount of container design fixes either half. If the
container surface lands with Q6 open, the accepted programs are the ones whose
loops return from inside — which is what `wfgrep` already does, by hand, for
what is very likely this reason.
