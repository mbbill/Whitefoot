# Containers: owners, views, and the facts that cross a call

Design deliverable for batch 0116, the **container half** of the
containers-and-resources pair. Its sibling `RESOURCES.md` owns the providers
`Heap`, `Arena<'p>` and `Pool<'p, T, N>`, the resource envelope `E`, and the
`resource-closed` judgment; this file names those concepts and never redefines
them. The seam between the two is exactly one sentence in each file:
`RESOURCES.md` does not define the container operations that consume providers,
and this file does not define what a provider is, how one is minted, or what
`E` promises. Where this file writes `provider: Prov`, `RESOURCES.md`'s
`[PROV-n]` rules fix the type and the mode.

Tree read: branch `batch/0116-containers-and-resources` at `main a40c7e70`,
`spec/kernel-spec.md` **v0.40 ACTIVE**. Bare four-digit line numbers are that
file. Nothing here is implemented, no compiler code was written for it, and the
rule text in section 3 is draft text for a work branch, not an amendment.

Provenance of the decisions this file builds on, which it does not reopen: the
owner's design discussion of 2026-09-01 (container/backing separation; push
never grows; grow and reserve owner-level with explicit effect and typed
failure; system I/O over views; `par` via reserved disjoint ranges; the names
`HeapVector` / `FixedVector<T, N>` / `Span` / `MutSpan` / `AppendView`), and the
sweep ledger's **D1** — a callee that replaces the whole referent through a
`&uniq buffer<u8>` actual leaves the caller's `len(line) = 10` alive, an
accepted out-of-bounds read *and* write. The lesson the owner drew from D1 is
the spine of section 3.3: **which facts survive a call must come from
signature-visible information, never from the actual's spelling.**

Section 8 separates what a compiler executed from what is argued on paper.
Thirteen programs were run against the gate binary, and every load-bearing
claim this file makes about *today's* language is a machine verdict rather than
a reading. Every syntax it introduces is design text no compiler has seen.

**Two findings came out of writing this rather than out of designing it**, and
both are in section 8 with their evidence. First, L1's consume-and-return
convention has no statement form: neither `set` nor `replace` can rebind an
affine local from a call that consumed it (probes `p10`, `p11`), so [CALL-7]
adds `rebind`, and without it not one loop in section 5 can be written.
Second — and this is the file's real risk — the header invariant every migrated
loop needs is **unstatable** under [INV-1]'s atom rule, because `len(P)` is not
an affine atom. That is Q6, it is a proof-surface question rather than a
container question, and it decides whether this design is usable.

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

## 2. The laws

Seven laws. Every rule in section 3 is an instance of one, and **a rule that
cannot name its law is not admitted.** They are stated first because the D1
class of defect is what happens when a clause is restated instead of derived:
D1's projection flag was a locally reasonable sentence ("an element write never
kills a length fact") applied at a place — the call boundary — where nothing
established that the write was an element write.

**L1 — A view is a value.**

> A view is an affine value with a static type, not a reference the callee
> writes through and not a hidden pointer to the owner's header. A function
> that changes a view's state consumes it and returns the new one. Therefore
> every state change a view can cause is visible in the callee's signature as a
> parameter consumed and a result produced, and the caller's post-state is the
> result value, not a re-read of some place the callee touched.

*What it delivers.* The write-back problem stated in the 2026-09-01 discussion
("`AppendView` cannot be a by-value `{ptr, len, cap}`, or the `len` the function
advanced never reaches the owner") is answered without a hidden protocol: the
advanced `len` **is** the result value. Pass-by-pointer for a parameter moved in
and returned on every path is then an ABI choice, not a semantics.

**L2 — Length is a type fact or a contract fact, never a guess.**

> At every program point the checker's knowledge of a sequence's length comes
> from exactly one of: the type (`array<T, N>`, `FixedVector`'s `cap = N`,
> `MutSpan`'s fixed length), an established fact with live support [ENT-3,
> ENT-5], or a verified contract relation [FN-9]. No rule infers a length from
> the shape of an argument, the name of a callee, the absence of a write, or
> what a body was seen to do.

*What it delivers.* D1's repair is not "fix the flag": it is that no `element`
flag derived from an actual exists to be wrong. Section 3.3 has three call
rules and each reads only declared types and declared contracts.

**L3 — No hidden growth.**

> No operation both uses existing capacity and acquires new capacity. Every
> operation that may acquire capacity takes an owner and a provider, names its
> allocation effect, and returns a typed failure. Every operation that only
> uses existing capacity is total under a proved capacity requirement and can
> allocate on no path.

*What it delivers.* The 2026-09-01 refutation's items 1, 3 and 4: one `push`
cannot have one return type and one effect row across backings; a growing
`push` inside a loop leaves partial commitments no clean semantics can
describe; and a view can never grow because it neither owns the backing nor
carries a provider.

**L4 — The initialized prefix is the only initialization state.**

> A sequence's storage is exactly `[0, len)` initialized and `[len, cap)` raw.
> The boundary is checker-maintained typestate carried by the owner's static
> type. No per-slot tag, `Option` wrapper, occupancy bitmap, or runtime
> discriminant exists, and no operation returns, borrows, or reads a raw slot.

*What it delivers.* Open question 4 of the 2026-09-01 audit ("how does a view
hide vacant slots?") dissolves: there is no per-slot state to quantify over, so
the checker never needs `for all i < len, slot[i] is Some`. It needs one scalar
relation, `len <= cap`, which is a difference bound [ENT-4] already derives.

**L5 — Release belongs to the owner's backing type.**

> The release action of a sequence is fixed by the owner's type under [STOR-3]
> and by nothing else: drop `[0, len)` in ascending index order, then the
> backing's own release. A view never releases the backing. No source
> construct selects, replaces, or observes the action.

*What it delivers.* Owner decision 2's drop order, and the reason owning
generics stay concrete (N4): the release row is a property of the backing, so a
function that may drop its owner cannot be backing-generic until effect
polymorphism exists.

**L6 — An `AppendView` reaches only what it appended.**

> An `AppendView` presents the spare window `[base, cap)` of its owner, where
> `base` is the owner's length at formation. Its own `len` counts the elements
> appended through it and starts at zero. No operation on an `AppendView`
> reaches an index below `base`, and no operation on it decreases the owner's
> length.

*What it delivers.* This is the D1 law in the container surface. A callee
holding an `AppendView` **cannot shrink the caller's sequence**, so the
caller's `len` fact does not need to die at the call, and the design does not
buy soundness by throwing away every length fact at every call.

**L7 — Capacity is a proof term, not a value.**

> `cap(v)` exists as an [ENT-2] term and in contract relations. No operation
> returns it as a runtime `u64`. Programs prove with it; programs cannot branch
> on it.

*What it delivers.* Open question 8 of the audit: an allocator that rounds a
request up may not change any accepted program's behavior. `seq_reserve`
guarantees `cap >= len + additional` and rounding stays unobservable. It also
keeps growth policy out of the language: no program can observe whether growth
was exact, 1.5x, or 2x.

---

## 3. The rules

Stated in the specification's register: each names its checker judgment, what
it publishes, and which existing rule it amends or retires.

### 3.1 `[CNT]` — owners and typestate

**[CNT-1] The owner inventory.** Exactly four sequence owners, each with a
static backing fixed by its type.

| type | backing | placement | provider needed | `cap` | growth |
|---|---|---|---|---|---|
| `FixedVector<T, N>` | inline, `N` slots | stack frame, `static`, or a struct field of an owner in any of those | none | `N`, a constant | never |
| `HeapVector<T>` | one heap allocation, none while empty | frame-resident descriptor | `Heap` (`RESOURCES.md`), to grow | runtime | `seq_reserve` |
| `ArenaVector<'r, T>` | one arena block in `'r` | frame-resident descriptor | the arena of `'r`, to grow | runtime | `seq_reserve` in `'r` |
| `PoolVector<'p, T>` | one pool lease | frame-resident descriptor | the pool provider of `'p`, at lease | fixed at lease | never |

One seam to flag rather than paper over: `RESOURCES.md`'s `Pool<'p, T, N>` names
`N` interchangeable single-`T` slots, and a `PoolVector` needs one **contiguous
run** of them. Either the pool grows a contiguous-run lease or `PoolVector` is
dropped from this inventory; the container side does not get to decide, and
nothing else in this file depends on the answer.

A container type is a compiler-owned nominal: it has no writer-visible field,
is constructed only by the `[SEQ]` operations, and has no source construction
form [GRAM-8]. *Amends* [TYPE-2] (four added composite types) and answers open
question 1 of the 2026-09-01 audit with candidate A: opaque. Candidate C — an
ordinary struct whose invariants are reproved at every use — is refused because
`length <= capacity` would then be a fact with support the writer can kill,
and `[ENT-5]` would delete it at the first unrelated `set`.

**[CNT-2] Container state is typestate, not stored data the writer can reach.**
Each owner carries `len` and, where it is not a constant, `cap`. The checker
holds `len(v)` and `cap(v)` as [ENT-2] length-class terms of fragment type
`u64`. The implicit facts `Z <= len(v)`, `len(v) - cap(v) <= 0`, and — for
`FixedVector<T, N>` — `cap(v) = N` hold at every program point at which `v` is
live, exactly as [ENT-2]'s `len(P) = N` implicit fact does for `array`. *Amends*
[ENT-2] clause (b), which today admits `len(P)` only for `array`, `slice`, and
`buffer`.

**[CNT-3] Raw slots are unreachable.** No operation of `[SEQ]`, no subscript,
and no borrow yields a place in `[len, cap)`. A subscript on an owner or view
carries the ordinary [OP-4] obligation `ilt(index, len(base))` — against `len`,
never `cap`. There is no uninitialized read to reject because there is no
spelling that reaches one (L4).

**[CNT-4] Affine elements.** `T` may be affine in every owner. The initialized
prefix is what makes this sound: an element enters at `len` and leaves at
`len - 1`, so no slot is ever read before it is written or after it is taken.
`FixedVector<Handle, 64>` is the kernel object table the owner named.
*Amends* [TYPE-2]'s `array` restriction only by not inheriting it: `array<T, N>`
keeps its copy-only element domain, because `array` carries no length separate
from `N`, so every slot is live at once and there is no prefix boundary to make
an affine element's entry and exit unambiguous (L4). *Verified today:* `array_new<box<u64>, 4>` is `InvalidOperation`
at [OP-1] (§8, probe `p9`), so this capability is new, not a restatement.

**[CNT-5] Release.** The release action of every owner, under [STOR-3]:

```text
drop element [0]                  ascending index order,
drop element [1]                    each element's own compiler-derived drop
...
drop element [len-1]
release backing:
  FixedVector    nothing (inline in its owner)
  HeapVector     one compiler-derived heap free
  ArenaVector    nothing at the value; the block goes with 'r  [STOR-4]
  PoolVector     one lease return, which writes the pool provider's state
```

Only `PoolVector`'s backing release carries a nonempty effect row, contributed
under [EFF-2]'s release contribution. *Amends* [STOR-3]'s `buffer<T>` drop
sentence by superseding it (§3.6).

**[CNT-6] Containers are storable; views are not.** A container type is
region-free except `ArenaVector<'r, T>` and `PoolVector<'p, T>`, which are
region-bearing exactly as `arena<'r, T>` is. A `FixedVector` or `HeapVector`
may be a struct field, `box` content, or arena content. *Extends* [STOR-5]'s
region-bearing relation to the two region-bearing owners and to all three view
types.

**[CNT-7] Acquiring capacity is owner-level and provider-bearing.** Every
operation that may change `cap(v)` takes the owner **by value**, takes the
provider, and names its allocation effect. It returns `Result` when acquisition
can fail ([SEQ-14]) and the owner directly when it cannot, because the old
backing is kept on failure ([SEQ-16]). There is no capacity-changing operation
on a borrow and none on a view (L3).

**[CNT-8] A container type never appears behind `&uniq`.** A `param`,
`rtype`, or `let`-bound holder whose mode is `&uniq 'r` and whose direct type is
a container type is a hard error citing CNT-8 at the complete `param` (or
`rtype`) node, with the restructuring `pass a MutSpan or AppendView for element
and append work, or take the owner by value and return it`. A shared `&'r`
container parameter remains legal: it can observe `len` and read elements and
can change nothing.

This is the rule that retires D1's shape. *Retires* the writer-facing
`&uniq buffer<T>` and `&uniq Container` state-borrow forms; `&uniq` survives for
`MutSpan`-style element writes, where the type fixes the length ([CALL-3]).

**[CNT-9] `array<T, N>` is retained unchanged** as the `len ≡ cap ≡ N` case.
A program that needs no length carries no length. `fir_filter.wf` is untouched
by this design.

### 3.2 `[VIEW]` — views, formation, and write-back

**[VIEW-1] The three views.**

| type | reads | writes elements | changes length | may allocate | affine |
|---|---|---|---|---|---|
| `Span<'r, T>` | yes | no | no | no | yes |
| `MutSpan<'r, T>` | yes | yes | no — fixed by the type | no | yes |
| `AppendView<'r, T>` | the window it appended | the window it appended | grows the window only | no | yes |

Each is an `own` affine value carrying a region `'r`, exactly as
`slice<'r, T>` does today. `Span<'r, T>` **is** today's `slice<'r, T>` renamed;
the rename is the whole of the change to it. *Amends* [TYPE-2] (two added view
types), [STOR-5] (all three are region-bearing and unstorable), and [OWN-1]
(all three are affine).

**[VIEW-2] Formation freezes the owner.** A view is formed from a borrow of
the owner and holds that loan for `'r`:

```text
seq_span(&'r v)             -> own Span<'r, T>          shared loan on v
seq_mut_span(&uniq 'r v)    -> own MutSpan<'r, T>       exclusive loan on v
seq_append_view(&uniq 'r v) -> own AppendView<'r, T>    exclusive loan on v
```

While the loan is live, [OWN-5] already forbids moving, dropping, growing, or
otherwise writing `v` — no new exclusivity rule is needed, and the freeze the
owner asked for is the existing loan. Formation publishes:

```text
seq_span         len(s)  = len(v)
seq_mut_span     len(m)  = len(v)
seq_append_view  len(a)  = 0        and    cap(a) = cap(v) - len(v)
```

The last is a difference relation over live terms, so `cap(a)` needs no
subtraction operation in the fact domain: the checker establishes
`cap(a) + len(v) = cap(v)` as the two bounds [ENT-4]. *Amends* [ENT-3.S6],
which today has one `slice_of` row; these are three rows of the same kind.

**[VIEW-3] View provenance is slice provenance.** Every view value carries the
finite origin set [OWN-5] defines for slices, formed and preserved by the same
sentences: formation makes a singleton, and binding, moving, passing, and
returning preserve the set. An access through a view is judged as one access
through every origin. *Amends* [OWN-5] by generalizing "`slice<'r, T>` value" to
"view value" throughout; no clause of it changes shape.

**[VIEW-4] `MutSpan`'s length is fixed by its type.** No operation in `[SEQ]`
takes a `MutSpan` and produces a different length, and none takes one and
changes its owner's length. This is a closed property of the operation table,
readable from the type alone, and it is what [CALL-3] consumes.

**[VIEW-5] `AppendView` is a spare window (L6).** Its `base` is the owner's
length at formation and is not a source-visible value. `len(a)` counts what
this view appended. Every `[SEQ]` operation on an `AppendView` acts on
`[base + i]` for `0 <= i < len(a)`; `seq_truncate` on an `AppendView` may reduce
`len(a)` to zero and no further. A callee that receives an `AppendView`
therefore cannot reduce its caller's `len(v)` — which is why [CALL-3] can leave
the caller's length facts alive.

**[VIEW-6] `absorb` is the commit event.**

```text
let written = absorb(view: move a);
```

`absorb` consumes the `AppendView`, ends its append window, and returns
`own u64`. Its checker judgment, in this order, exactly mirrors [ENT-3.S5]'s
commit-value discipline for a `set`:

1. the operand's origin set is resolved to one owner place `P` ([VIEW-7]);
2. the result value is bound to the compiler-owned commit value `w`, with
   `w = len(a)` established at it;
3. every fact supported by `len(P)` dies, under [ENT-5] clause (a), as a
   whole-place length event on `P`;
4. only then are `written = w` and `len(P) = old + w` established, where `old`
   is the term the state held for `len(P)` immediately before step 3 when one
   was derivable, and no relation is established when none was.

Step 4's `old + w` is a three-term relation and therefore not an L0 difference
bound. Two derivable cases carry it: `old` derivable as a constant `k` gives
`len(P) = k + w`, a difference bound over `w`; otherwise the checker retains
`len(P) - old >= 0` and `len(P) - old <= cap(P) - old`, and the exact sum is
available only through [INV-1]'s affine domain. This is honest, narrow, and is
open question **Q4**.

**[VIEW-7] `absorb` is admitted only in the formation function.** The operand's
origin set must be a singleton resolved place of the current function. An
`AppendView` reaching a function as a parameter has a formal-view origin
[OWN-5], not a resolved place, so a callee cannot commit its caller's length
behind the caller's back. A violation is a hard error citing VIEW-7 at the
operand `atom`, with the restructuring `return the view to the function that
formed it and absorb it there`.

**[VIEW-8] An abandoned `AppendView` drops what it appended.** Its compiler-
derived release action under [STOR-3] is: drop the elements of `[base, base +
len(a))` in ascending order, then nothing. The owner's `len` is unchanged, so
the abandoned elements are neither leaked nor double-dropped, and no fact about
`len(P)` was ever published. This is what makes `absorb` an ordinary operation
rather than a must-use obligation: **not** absorbing is a well-defined, safe
program that discards work.

**[VIEW-9] Views are never stored** [STOR-5], and never returned except under
[VIEW-10].

**[VIEW-10] View return provenance.** [FN-1]'s slice-result ceiling applies
unchanged to each view type: a function whose written result is
`own Span<'r, T>` (resp. `MutSpan`, `AppendView`) has the ceiling containing
`immutable-const` and the formal-view origin of every parameter whose written
mode and type are exactly `own Span<'r, T>` (resp.) with the same formal region
and element type. A borrow-mode result of direct view type stays rejected for
[FN-1]'s stated reason: two provenance relations, one summary. *Amends* [FN-1]
by generalizing "slice" to "view".

### 3.3 `[CALL]` — what survives a call

This is D1's section. Exactly three transports exist, and **each reads only the
callee's declared parameter modes and types and its declared contract.**

**[CALL-1] Through a shared borrow, every fact survives.** For an argument
whose parameter mode is `&'r` — of any type, container and view included — the
call is not a kill event for any fact supported by the actual's resolved place.
Ground: [OWN-5] admits no write through a shared holder, so [EFF-2] can project
no `writes` occurrence onto that place, so [ENT-5] clause (b) does not fire.
*Verified today* for `&'a buffer<u8>`: probe `p6` (§8) keeps `len(line) = 10`
across the call and the subsequent `line[9_u64]` is accepted.

**[CALL-2] Through a value passed and returned, only the contract's facts
exist on the result.** An `own` argument is a consuming use [OWN-1], so
[ENT-5] clause (c) kills every fact whose support contains that binding's root.
The result is a fresh binding carrying exactly the callee's [FN-9]-verified
relations under [ENT-3.S12], and nothing else. In particular, `len` and `cap`
of the result are unknown unless the callee's `ensures` states them.
*Verified today:* probe `p1` (§8) — `passthrough(out: move a)` returning the
same buffer, then `b[9_u64]`, is **rejected** with residual `9_u64 < len(b)`.
The transport this design needs already behaves correctly; what is missing is
the contract vocabulary to publish across it, which is [CALL-4].

**[CALL-3] An element write through a length-fixed view never touches length
facts.** For an argument whose parameter's declared type is `MutSpan<'r, T>`,
`&uniq 'r MutSpan<'r, T>`, or `Builder<'r, T>` ([BLD-1]) — the types [VIEW-4]
and [BLD-1] fix a length for — a projected callee `writes` occurrence kills
every fact whose support overlaps the viewed **element storage** and kills no
length term over that origin. For an argument whose parameter's declared type
is `AppendView<'r, T>`, the same holds, plus: the callee cannot decrease the
owner's length (L6, [VIEW-5]), so the caller's `len(v)` facts survive; the
callee cannot increase it either, because only `absorb` publishes an increase
and [VIEW-7] denies `absorb` to a callee.

For every other parameter type the projected write kills length facts as an
ordinary whole-place event.

**[CALL-4] Contract vocabulary for containers and views.** [FN-9]'s clause
grammar is extended by exactly three admissions, and by nothing else:

1. a clause operand may be `len(P)` or `cap(P)` where `P` is an admitted formal
   place of container or view type (today: `len(P)` only, and only for `array`,
   `slice`, `buffer`);
2. a clause operand may be `len(result)` or `cap(result)` when the written
   result type is a container or view type — this is the admission that makes
   `own`-in / `own`-out contracts possible at all, and today's result-datum
   restriction to fragment integers forbids it;
3. one comparison operand may be `t + k` for an admitted term `t` and a
   **constant** `k` (literal or named integer const), because `ile(x, y + k)`
   normalizes to the difference bound `x - y <= k`. A non-constant offset is not
   admitted here and is open question **Q4**.

So the canonical append contract is writable:

```wf
const CEILING: u64 = 6;

fn escape_json['i, 'o](
  input: own Span<'i, u8>,
  output: own AppendView<'o, u8>
) -> written: own AppendView<'o, u8> reads(input), writes(output) contract {
  define room = cap(output);
  define needed = len(input);
  requires ile(needed, room);
  ensures ile(len(written), CEILING);
} { ... }
```

The clause is single-state: `output` denotes the entry image of the parameter
and `written` the result, both under [FN-9]'s existing entry-image machinery,
with no second state, no `old()`, and no frame rule.

*Verified today:* probe `p4` (§8) compiles a single-state `ensures` anchored on
`len(deref(destination))` with a fragment result, so the entry-image half of
this shape works. Probe `p2` shows `len(result)` does not parse today
([GRAM-9]), which is why admission 2 is written as an amendment.

**[CALL-5] Multi-return.** A function may declare an ordered result tuple:

```wf
fn split_two['r](source: own MutSpan<'r, u8>, at: own u64)
  -> (head: own MutSpan<'r, u8>, tail: own MutSpan<'r, u8>) ...

let (head, tail) = split_two<'r>(source: move whole, at: 16_u64);
```

Each element has its own mode, type, and, under [CALL-4], its own contract
relations; the destructuring `let` binds each as an ordinary fresh binding.
The result is not a value: there is no tuple type, no tuple place, and no way
to store or pass one. It is a return-and-bind form only, which keeps [STOR-5]
and [TYPE-2] untouched. *Verified today:* the syntax does not parse ([GRAM-2],
probe `p8`), so this is new syntax and is labelled as such.

Multi-return is load-bearing, not a convenience: `seq_pop` must return a view
and an element, and no single value can carry both — an enum payload holding a
view is refused by [STOR-5], and a struct field holding one is refused by the
same rule.

**[CALL-6] No transport reads the actual's spelling.** The three transports
above are selected by the callee's declared parameter mode and type and by its
declared contract. No rule of this design consults the argument expression's
shape, the callee's body, its name, or any per-parameter summary derived from
its body. A parameter type for which no transport is selected kills
conservatively.

*This is D1 stated as a rule.* The located mechanism of D1 —
`argument_referent` returning `element = true` for every `&uniq buffer<T>`
actual (`compiler/src/semantic/places.rs:349-355`) — is a fact derived from the
actual's shape, and under CALL-6 no such fact exists to be derived. The
precision it was buying is bought instead by the type: a `MutSpan` argument is
element-only **because its type admits nothing else**.

**[CALL-7] Affine rebinding — the statement L1 requires.** A consume-and-return
call cannot be written into a loop today, and this is not a style problem: [SET-1]
and [STOR-1] refuse an affine `set` target, [SET-2]'s `replace` refuses a
right-hand side that moved the target root, and [OWN-1] says reinitialization
requires a new `let` — which a loop body cannot do, because the next iteration
needs the previous iteration's value. So:

```wf
rebind view = seq_push(view: move view, value: byte);
```

`rebind p = e;` requires `p` to be a bare local binding of affine type whose
value `e` consumes, and `e` to produce exactly that binding's type. Its
judgment: evaluate `e` under ordinary rules, including the consume of `p` inside
it; every fact whose support contains `p`'s root dies at that consume ([ENT-5]
clause (c)); then the binding is reinitialized with `e`'s value, live and
usable, with no observable program point between. It derives no drop and no
release: nothing is destroyed, exactly as [SET-2]'s commit derives none. Its
[ENT-3] image is [SET-1]'s commit-value discipline with the kill already
performed by the consume.

*Amends* [OWN-1] (one reinitialization route that is not a new `let`), [STOR-1]
and [SET-1] (whose affine-target rejections keep their wording and gain
`rebind` as the named alternative in their mechanical fixes), and [GRAM-4]
(one statement production). It is the sole writer-facing cost of L1, and it
buys the whole write-back story: the advanced length reaches the owner because
it *is* the value, and `rebind` is where it lands.

Lowering is the ABI note the owner already made: a parameter moved in and
returned on every path is passed by pointer, so `rebind view = seq_push(view:
move view, value: byte);` lowers to a store and a length increment on one
in-place descriptor, with no copy.

### 3.4 `[SEQ]` — the operation table

One operation family per row, resolved by name and then by receiver type within
the family, exactly as `len` and `slice_of` resolve today [OP-1]. Constructors
carry distinct names rather than one overloaded `seq_empty`, because selecting a
row by the result type would be expected-type selection, which [TYPE-5] forbids
and [OP-1] refuses ("operand types never select between an operation family, a
function, and a system operation"). `V` ranges
over the four owners; `Prov` is the owner's provider (`&uniq 'h Heap` for
`HeapVector`, the arena or pool provider otherwise; `RESOURCES.md` fixes the
types).

| # | op | receiver | signature | requires | publishes | effects | failure |
|---|---|---|---|---|---|---|---|
| [SEQ-1] | `seq_fixed<T, N>` | — | `() -> own FixedVector<T, N>` | — | `len = 0`, `cap = N` | `pure` | none |
| [SEQ-2] | `seq_heap<T>`, `seq_arena<'r, T>` | — | `() -> own HeapVector<T>`; `() -> own ArenaVector<'r, T>` | — | `len = 0`, `cap = 0` | `pure` | **none** — an empty growable sequence owns no backing and allocates nothing; every allocation is [SEQ-14]'s |
| [SEQ-2b] | `seq_lease<'p, T>` | — | `(provider: Prov, capacity: own u64) -> own Result<PoolVector<'p, T>, ResourceError>` | — | on `Ok(value: r)`: `len(r) = 0` | `allocates(...)`, `writes(provider)` | typed |
| [SEQ-3] | `seq_len` | owner, view | `(v: &'r V) -> own u64` | — | `n = len(v)` | `reads(v)` | none |
| [SEQ-4] | `seq_push` | `AppendView` | `(view: own AppendView<'r, T>, value: own T) -> own AppendView<'r, T>` | `ilt(len(view), cap(view))` | `len(result) = len(view) + 1` | `writes(view)` | **none — total** |
| [SEQ-5] | `seq_try_push` | `AppendView` | `(view: own AppendView<'r, T>, value: own T) -> (rest: own AppendView<'r, T>, outcome: own Option<T>)` | — | `ile(len(rest), cap(rest))` | `writes(view)` | `Some(value)` returns the value unconsumed |
| [SEQ-6] | `seq_pop` | `AppendView` | `(view: own AppendView<'r, T>) -> (rest: own AppendView<'r, T>, value: own T)` | `igt(len(view), Z)` | `len(rest) = len(view) - 1` | `writes(view)` | none |
| [SEQ-7] | `seq_truncate` | `AppendView` | `(view: own AppendView<'r, T>, keep: own u64) -> own AppendView<'r, T>` | `ile(keep, len(view))` | `len(result) = keep` | `writes(view)` | none; drops `[keep, len)` descending |
| [SEQ-8] | subscript `p[i]` | owner, `Span`, `MutSpan` | element place | `ilt(i, len(p))` [OP-4] | — | per access | none |
| [SEQ-9] | `seq_get` | owner, `Span`, `MutSpan` | `(v: &'r V, index: own u64) -> own Option<T>` — `T` copy | — | — | `reads(v)` | `None` out of range |
| [SEQ-10] | `seq_span` | owner | `(&'r v) -> own Span<'r, T>` | — | `len(s) = len(v)` | `pure` | none |
| [SEQ-11] | `seq_mut_span` | owner | `(&uniq 'r v) -> own MutSpan<'r, T>` | — | `len(m) = len(v)` | `pure` | none |
| [SEQ-12] | `seq_append_view` | owner | `(&uniq 'r v) -> own AppendView<'r, T>` | — | `len(a) = 0`, `cap(a) + len(v) = cap(v)` | `pure` | none |
| [SEQ-13] | `absorb` | `AppendView` | `(view: own AppendView<'r, T>) -> own u64` | — | [VIEW-6] | `writes(view)` | none |
| [SEQ-14] | `seq_reserve` | `HeapVector`, `ArenaVector` | `(vector: own V<T>, provider: Prov, additional: own u64) -> own Result<V<T>, ResourceError>` | — | on `Ok(value: r)`: `ige(cap(r), len(r) + additional)`†, `len(r) = len(vector)` | `allocates(...)`, `writes(provider)` | typed; on `Err` the vector is returned inside the error, unchanged |
| [SEQ-15] | `seq_clear` | owner | `(vector: own V<T>) -> own V<T>` | — | `len(result) = 0` | — | none; drops `[0, len)` descending |
| [SEQ-16] | `seq_shrink` | `HeapVector` | `(vector: own HeapVector<T>, heap: &uniq 'h Heap) -> own HeapVector<T>` | — | `len(result) = len(vector)` | `allocates(heap)`, `writes(heap)` | none; on failure keeps the larger backing |

† `ige(cap(r), len(r) + additional)` uses [CALL-4] admission 3 with a
non-constant offset and is therefore **not writable under this draft**. The
writable form is `ige(cap(r), K)` for a constant `K`, or the relation is carried
by the operation's own contract rather than by a written clause — a table
operation's published facts are [ENT-3] rows, not [FN-9] clauses, so [SEQ-14]'s
guarantee lands as a fact source and needs no clause grammar at all. The gap
is real only for **user** functions, and that is Q4.

Notes on the table:

- **[SEQ-4] is the operation the whole design exists for.** It is total,
  allocation-free on every backing, and lowers to `store` plus `len + 1` with
  no capacity branch, because its requirement is discharged before lowering.
  The 2026-09-01 refutation's item 2 is respected: the writer calls a total
  `push` and the checker proves the requirement; the proof never rewrites a
  `Result`-returning operation into a `unit`-returning one.
- **There is no `push` on an owner.** A writer who wants push-with-growth
  writes the shell in section 6.1: reserve, form the view, push, absorb.
- **[SEQ-5]'s `Option<T>` returns the value**, so no owner is lost on the full
  path (audit question 8, the affine-value disposition).
- **[SEQ-14] returns the vector inside its error**, so a failed reserve loses
  nothing and changes nothing (audit question 8, failure atomicity). The order
  is fixed: compute the new capacity and discharge its arithmetic and
  allocation-domain obligations [OP-9]; acquire; move elements; commit the
  descriptor; release the old backing. Nothing observable changes before the
  acquisition succeeds.
- **No row reads `cap` as a value** (L7).

### 3.5 `[BLD]` — the `par` builder

The problem [PAR-2] cannot solve as stated: a counted loop cannot share one
`AppendView`, because every iteration would write one `len`. The 2026-09-01
refutation's item 7 is exactly this. The answer is to reserve first and then
give each iteration a slot it can prove is its own.

**[BLD-1] `Builder<'r, T>`** is a fourth view type: a claimed, write-once range
of a sequence's spare window. `len(b)` is its slot count and is fixed at
formation, so `Builder` is a length-fixed type under [CALL-3].

```text
seq_claim(view: own AppendView<'r, T>, count: own u64) -> own Builder<'r, T>
    requires ile(count, cap(view))            len(b) = count
builder_set(slots: &uniq 'b Builder<'r, T>, index: own u64, value: own T) -> unit
    requires ilt(index, len(slots))           writes(slots)
seq_finish(builder: own Builder<'r, T>) -> own AppendView<'r, T>
    requires the coverage certificate below   len(result) = len(view) + len(builder)
```

**[BLD-2] Write-once slots.** A `Builder` slot is written at most once: the
admission in [BLD-3] gives each iteration a distinct index, and `builder_set`
has no read path, so no slot is observed before it is written. Elements written
into a `Builder` and never finished are dropped by the `Builder`'s release
action, exactly as [VIEW-8] does for an `AppendView`.

**[BLD-3] The coverage certificate.** `seq_finish` is admitted only when its
operand's sole write history is one counted `for_stmt` whose body contains
exactly one `builder_set` on it, whose `index` actual is that loop's binder
under [PAR-2]'s retained affine map with `a = 1, b = 0`, and whose loop range is
exactly `0_u64..len(b)` with both endpoints admitted [ENT-2] terms. The
certificate is then: distinct binder values give distinct indices ([PAR-2]'s own
argument), and the half-open range covers `[0, len(b))` exactly. A violation is
a hard error citing BLD-3 at the `seq_finish` operand, with the restructuring
`fill every claimed slot in one counted loop indexed directly by its binder, or
claim only the slots you fill`.

This is deliberately the narrowest rule that admits the shape the owner named
and refuses everything else rather than starting a search. It is the weakest
part of this design and is open question **Q5**.

**[BLD-4] `par` permission comes from [PAR-2] unchanged.** `builder_set` is a
call whose `writes(slots)` projects to the builder's element storage; the
`&uniq 'b Builder` argument is rooted in a binding declared outside the loop,
so today [PAR-2] would deny it on the exclusive-loan condition. The single
amendment is: **a `&uniq` loan on a `Builder` whose only body use is one
`builder_set` under [BLD-3]'s map is refined to the single-element range
`[i, i+1)`**, exactly as [PAR-2] already refines a direct subscript write. No
other condition of [PAR-2] changes, and the accumulator, endpoint, and exit
conditions are untouched.

### 3.6 Amendment and retirement register

| existing rule | disposition |
|---|---|
| [TYPE-2] `buffer<T>` | **retired from the writer surface.** `HeapVector<T>` replaces it. The flat `{data-pointer, u64 length}` value survives as `HeapVector`'s compiler-owned backing, which no source names |
| [TYPE-2] `slice<'r, T>` | **renamed** `Span<'r, T>`; semantics unchanged (read-only, region-carrying, affine, origin-tracked) |
| [TYPE-2] composite list | **+4 owners, +3 views** ([CNT-1], [VIEW-1], [BLD-1]) |
| [OP-1] `buffer_new`, `buffer_vacant`, `slice_of` | **retired**, replaced by [SEQ-1], [SEQ-2], [SEQ-10]. `buffer_vacant`'s `Option`-element construction has no successor: [CNT-4] makes it unnecessary (L4) |
| [OP-4] indexable bases | **extended** to the four owners, `Span`, and `MutSpan`; the obligation is against `len`, never `cap` |
| [OP-9] `buffer_fits` | **retained**, renamed to the allocation-domain predicate the two allocating rows use |
| [STOR-1] storage class by type | **extended**: the four owners join the table; `buffer<T>`'s sentence and the growable-collection paragraph are superseded in place |
| [STOR-3] `buffer<T>` drop | **superseded** by [CNT-5]; the affine-element drop order it already states is what [CNT-5] keeps |
| [STOR-5] region-bearing | **extended** to views and to the two region-bearing owners |
| [OWN-1] affine classification | **extended**: owners and views are affine |
| [OWN-5] slice origins | **generalized** from `slice` to view; no clause changes shape |
| [FN-1] slice-return ceiling | **generalized** to view-return ceiling ([VIEW-10]) |
| [FN-9] clause operands | **extended** by [CALL-4]'s three admissions |
| [FN-9]/[FN-1] result shape | **extended** by [CALL-5]'s multi-return |
| [EFF-2] slice parameter names the backing | **generalized** to view parameter; the sentence already says what is needed |
| [ENT-2] length terms | **extended** to owners and views; `cap(P)` added as a term of the same class |
| [ENT-3.S6] | **+3 rows** for the three formations ([VIEW-2]) |
| [ENT-3.S5] | **+1 source**: `absorb`'s commit value ([VIEW-6]) |
| [ENT-5] clause (b) | **superseded for containers and views** by [CALL-1..3] and [CALL-6]; the "element write never kills a length fact" sentence keeps its meaning for `array` and gains a type-derived premise everywhere else |
| [SYS-8] buffer parameters | **retired**: `read_at`, `write_once`, `directory_next`, `host_copy_bytes`, `host_copy_utf8`, `open_directory`, `open_file` take `MutSpan<'r, u8>` or `Span<'r, u8>`. Their `start <= end` and `end <= len(view)` obligations are unchanged in form; `len(deref(buffer))` becomes `len(view)`. This is the change that lets a heap-free program do I/O (G4) |
| [PAR-2] | **+1 refinement** for a `Builder` loan ([BLD-4]) |
| [GRAM-2], [GRAM-4] | **+ multi-return `rtype` list and destructuring `let`** ([CALL-5]) |
| [OWN-1] reinitialization | **+1 route**: `rebind` reinitializes one affine local from a call that consumed it ([CALL-7]). The sentence "reinitialization requires a new `let`" is **restated to name both routes**, not given an exception clause [META-3] |
| [SET-1], [STOR-1] affine-target rejections | **unchanged in judgment**; their mechanical fixes gain `rebind` beside `replace` |
| [GRAM-4] | **+1 statement production**, `rebind_stmt` ([CALL-7]) |

---

## 4. The fact discipline

### 4.1 Terms

| term | fragment type | established by | support [ENT-5] |
|---|---|---|---|
| `len(v)`, owner | u64 | [SEQ-1], [SEQ-2], [SEQ-7], [SEQ-14], [SEQ-15], [VIEW-6] | `v`'s root and every holder used to reach it; **not** its element storage |
| `cap(v)`, owner | u64 | implicit `= N` for `FixedVector`; [SEQ-14] | same |
| `len(s)`, view | u64 | [VIEW-2], [SEQ-4..7] | the view binding's root; **not** the viewed element storage |
| `cap(s)`, view | u64 | [VIEW-2] | same |

Term identity is [ENT-2]'s spelling identity, unchanged: `len(a)` for a view
and `len(v)` for its owner are **distinct terms** related only by the equalities
[VIEW-2] and [VIEW-6] publish. This is deliberate (Q3): making them one term
would require the fact domain to model aliasing, which [ENT-2] explicitly
declines to do ("term identity under-approximates aliasing, while kills use
[OWN-7]'s overlap relation and over-approximate it").

### 4.2 Value images

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

### 4.3 What the `absorb` commit publishes

Exactly [VIEW-6] steps 2–4: the commit value `w` with `w = len(a)`; the death of
every fact supported by `len(P)`; then `written = w`, and the sum relation in
the two derivable cases named there. It publishes **nothing** about the
elements, about `cap(P)`, or about any other place. It is one event at one
source point, like a `set` commit, and it is the only route by which a
sequence's length increases.

### 4.4 What dies when the owner is moved into a call

Everything supported by that binding's root, under [ENT-5] clause (c), at the
consume. The result binding is fresh and its facts are exactly [ENT-3.S12]'s
substitution of the callee's [FN-9] relations. There is no frame rule, no
"unchanged elsewhere" inference, and no reconstruction of the caller's old
facts on the new binding — a caller who needs `len` on the result must read it
([SEQ-3]) or the callee must publish it ([CALL-4]).

### 4.5 D1, re-derived

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

## 5. Migration: the four programs

### 5.1 `wfgrep.wf` — `append_slice` becomes the canonical view algorithm

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

### 5.2 `growable_vec.wf` — the hand-built vector disappears

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

### 5.3 `percent_decode.wf` — the output parameter becomes a view

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

### 5.4 `fir_filter.wf` — unchanged

`array<f64, 8>` in a struct field, written by subscript through the owner. No
length, no capacity, no view. [CNT-9] keeps it exactly as it is. It is in this
list to show the design's floor: a program that needs no sequence state pays
nothing for one.

---

## 6. From an unaware writer to an accepted program

Four walkthroughs. Each starts from what a writer who has not read this file
would naturally write.

### 6.1 Push without a capacity proof

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

### 6.2 Using a view after the owner moved

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

### 6.3 Trying to grow through a view

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

### 6.4 Two containers, one function

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

## 7. Open questions

Each has two candidates and a recommendation. None is settled by this file.

**Q1 — `AppendView` write-back: scope-end commit or explicit `absorb`?**

| candidate | what it costs |
|---|---|
| (a) commit automatically on the edge leaving the view's region | needs a must-be-live obligation on the view at that edge — a new obligation kind — and puts a fact-publishing event on an edge with no source spelling, which [DIAG-1] must then locate somewhere |
| (b) explicit `absorb` ([VIEW-6]), abandonment defined by a release action ([VIEW-8]) | the writer can forget to absorb and silently discard work; no rule catches it |

**Recommendation: (b), as drafted.** The commit gets a source point, [ENT-3.S5]
supplies the whole discipline unchanged, and abandonment is *safe* rather than
*prevented* — [VIEW-8]'s release drops the pending elements, so nothing leaks
and nothing double-drops. (a)'s must-use obligation would be the first
obligation in the language attached to a value rather than to an operation.
Mitigation for (b)'s cost: a compiler note, not a rule, where an `AppendView`
with a nonzero proved `len` reaches its release.

**Q2 — May a `MutSpan` be split for `par`?**

| candidate | what it costs |
|---|---|
| (a) `seq_split_at(span, at) -> (head, tail)` with `len(head) = at`, `len(tail) = len(span) - at` | the two results share one origin set, so [OWN-5]'s origin machinery must learn that two views of one place are disjoint — today it says two accesses through one origin conflict |
| (b) no split; `par` writes go through `Builder` ([BLD-1]) only | recursive divide-and-conquer over a span (merge sort, parallel scan) is not writable |

**Recommendation: (b) for this design, (a) as the next extension.** [BLD-3]'s
certificate covers the shape the owner named — reserve, fill disjoint slots,
publish one length — with rules that are checkable today. (a) needs a
disjointness relation on origin sets that [OWN-5] does not have, and adding one
to admit a shape no current program writes is exactly the work the project's
priority order defers. The design does not block (a): a split's two results
would be ordinary views with the same origin plus a range, and [VIEW-3] is
where the range would live.

**Q3 — How do `len(view)` and `len(owner)` relate in the fact domain?**

| candidate | what it costs |
|---|---|
| (a) distinct terms plus the [VIEW-2] and [VIEW-6] equalities (as drafted) | a writer who reasons about `len(v)` while a view is live gets nothing; every relation must be restated on the view's term |
| (b) one term: `len(a)` *is* `len(v)` | [ENT-2]'s spelling-based term identity would have to model that two spellings denote one storage — the aliasing the fact domain explicitly declines |

**Recommendation: (a).** It is the only one compatible with [ENT-2] as written,
and (a)'s cost is nearly zero in practice because the owner is frozen while the
view lives, so there is no useful reasoning about `len(v)` in that window
anyway. `MutSpan` shows why: `len(m) = len(v)` is published once at formation
and both terms are immutable for the view's life.

**Q4 — Non-constant offsets in contract relations.**

`ile(len(written), len(output) + n)` with `n` a parameter is not a difference
bound. Candidates: (a) admit three-term affine relations into [FN-9]'s
RelationTemplate and into L0, or (b) route them through [INV-1]'s affine domain,
which already handles `ile(sum, 255_u32 * (i + 1_u64))`.

**Recommendation: (b).** [INV-1] and [PRF-1] already own an affine domain with
stated ceilings, a normalization, and a certificate form; L0 is a difference-
bound domain on purpose, and widening it changes the closure's complexity and
its determinism argument. The work is to let a verified contract relation be
*stated* in the affine domain and *queried* there, which is a boundary change,
not a new prover. Until that lands, [SEQ-14]'s guarantee is carried as a table-
operation fact source ([ENT-3]) rather than a written clause, and user functions
write constant ceilings.

**Q5 — Is [BLD-3]'s coverage certificate the right shape?**

| candidate | what it costs |
|---|---|
| (a) syntactic: one counted loop, binder-indexed, range exactly `0..len(b)` (as drafted) | refuses a two-loop fill, a strided fill, a fill with an early `break`, and a nested fill |
| (b) a written `invariant` the writer proves at `seq_finish` — "every index below `len(b)` was written" | needs a quantified proposition, which [INV-1]'s affine domain does not have and [ENT] declines |

**Recommendation: (a), and say plainly that it is a shape rule.** (b) is the
right long-term answer only if the language ever grows a bounded quantifier;
until then (a) admits the one shape with a machine-checkable certificate and
refuses everything else rather than starting a search — the same choice
[PAR-2] made for its affine map, and [BLD-4] reuses that map exactly.

**Q6 — Loop-carried length facts through a write-back contract.**

Probe `p5` (§8) shows that today a `len`-anchored [FN-9] postcondition is not
discharged at a return following a counted loop that assigns the result
binding, even with a header invariant and a continuation `invariant_stmt`. The
shapes that verify are loop-free (`p4`) or return from inside the loop
(`wfgrep`'s `append_slice`). The question has two halves, and §5.1's ‡ note
shows the second is the harder one.

| candidate | what it costs |
|---|---|
| (a) a prover fix: connect an [INV-1] value-image conclusion to an [FN-9] `len`-anchored query across a loop | work inside the existing rules; nothing in the language surface changes; does not by itself make §5.1's invariant *writable* |
| (b) a language fix: admit `len(P)` and `cap(P)` as [INV-1] affine atoms | widens the affine domain's atom set, so [INV-1]'s formation, normalization, and structural ceilings each need a sentence about a length atom's support and its kills |

**Recommendation: both, (b) first.** (b) is the blocking half: under [INV-1]'s
current atom rule the invariant every migrated loop needs cannot be *written*,
so no amount of (a) reaches it. (b) is also the smaller change than it looks —
a length atom's support is already fixed by [ENT-5] (the viewed place's
non-element root path), so the sentences (b) needs are restatements of rules
that exist, not a new domain. Every migrated program in section 5 needs this
shape, so the answer decides whether the container surface is usable, not
merely whether it is elegant. It is the highest-value follow-up in this batch
and it is a **prover and proof-surface question, not a container question** —
which is why this file states it rather than bending the design to avoid it.

**Q7 — Syntax for multi-return and for view formation.**

Multi-return: `-> (a: own T, b: own U)` with `let (a, b) = f(...)` (drafted), or
a named-result form `-> { a: own T, b: own U }` with `let a, b = f(...)`.
**Recommendation: the drafted form**, because it reuses `param_list`'s
`IDENT ":" mode type` shape exactly and therefore adds one production rather
than a second binding grammar. View formation: the drafted operation calls
(`seq_span(&'r v)`) or a suffix form (`v.span<'r>()`). **Recommendation:
operation calls**, because [GRAM-5] has no method-call production and adding one
for three operations is a large grammar change for a small readability gain.

---

## 8. Verified versus reasoned

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
