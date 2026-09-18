# Range families: proposed x0 extension E2

Status after the owner's process correction: parked, unselected mechanism
draft. Interval accounting and its operation restrictions were agent proposals,
not necessary consequences or owner-selected repairs of x0. Preserve the local
cases for comparison; do not continue this experiment by default. Current
preparation is in [DESIGN.md](DESIGN.md#next-candidate-preparation-temporary-references-and-effect-derived-calls).

This note proposes one bounded rule-and-code experiment for the frozen x0
matrix in [MATRIX-X0.md](MATRIX-X0.md). It is not an x0 verdict edit, an
official language rule, or a claim that general predicates, providers,
existentials, sparse containers, or parallel resource transfer are solved.
E2 gives one fixed backing a finite certificate for a runtime-length interval
of affine or linear elements. R1-R16 and the v0.59 language floor otherwise
remain unchanged. Maintain this note while E2 is tested; supersede it in place
if the experiment changes, and remove it with this investigation.

## Alternatives considered in this draft

The discriminator is a dynamic `n`: create empty live capacity, transfer
exactly `n` noncopyable values into it, split at runtime `k`, take and restore
one bounded element, then clean every value and release both backings. The
checker must reject a lost, duplicated, wrong-backing, or out-of-range duty by
finite structural checking. Proof size and automatic work must not expand with
runtime `n`; proofs erase; proof machinery adds no flag, map, token, lock,
branch, allocation, or uniqueness restriction on copied locators. The written
reserve, refusal match, transfers, and cleanup loops remain executable.

1. Materialize one proof token per element. This makes runtime length require
   runtime token allocation or compiler expansion proportional to `n`; it
   fails the representation and deterministic-acceptance criterion.
2. Store an occupancy bitmap or `Option<T>` tag per slot. Cleanup is executable,
   but it changes layout and makes proof state a runtime partial-state map.
3. Use a symbolic interval partition whose leaves are uniformly `Full` or
   `Empty`, with a scoped one-index focus. Split/join and focus/unfocus have
   fixed arity, and existing numeric proofs discharge their bounds.

This unselected draft assumes alternative 3. It deliberately supports interval
partitions rather than an arbitrary finite set. Sparse holes are expressible
only while each hole is a written focus or a finite partition leaf; unbounded
sparse cleanup remains Q.

## Certificate forms and state

The following is explanatory syntax, not proposed final grammar.

```text
Backing<A,T,n>                         // executable owner descriptor for A
Dispose<A>                             // unique allocation-disposal duty
Range<A,T,[lo,hi), S>                  // erased certificate; S = Empty | Full
Elem<A,T,i,S>                          // erased focused one-element certificate
View<A,T,[lo,hi)>                      // executable non-owning view/locator range

reserve_slots<T>(n) -> Option<(Backing<A,T,n>, {Dispose<A>, Range<A,T,[0,n),Empty>} certs)>
certify_full(owner, lo, hi) -> {Range<A,T,[lo,hi),Full>}
project({range}) -> View<A,T,[lo,hi)>
focus {range} at i -> {left, elem, right}
split {range} at k -> {left, right}
join({left, right}) -> {range}
take(view[i], {elem}) -> (own T, {Elem<A,T,i,Empty>})
put(view[i], move(value), {elem}) -> {Elem<A,T,i,Full>}
consume(move(value))
release(backing, dispose, empty_range)
```

`A` is one captured backing identity, and `n`, `lo`, `hi`, and `i` are
immutable mathematical values captured when the certificate operation is
checked. `Range` and `Elem` are linear proof-context entries, never source
values, ABI arguments, or storable fields. Braces in signatures mark this
erased channel; a surface owner carries `Dispose`, tracked separately. `Full`
means every logical slot in the interval is
initialized and its exact `T` ownership duty is held by that certificate;
`Empty` means every slot is live raw storage and the certificate holds no `T`
duty. Neither state carries `Dispose<A>`. `Backing` describes typed live
storage and its fixed physical extent; `Dispose<A>` alone authorizes ending A.
For zero length, both uniform states contain zero content duties but remain
distinct certificate propositions; neither manufactures or consumes a value.

An ordinary `View` owns neither content nor backing. Copying it copies only its
locator descriptor. A locator to one element can coexist with other locator
aliases; E2 adds no persistent unique borrow. Existing operation-time access,
loan, effect, liveness, layout, and separation rules still decide executable
reads and writes.
Certificate `let`, assignment, and omission below are checker notation and all
erase. A view projected from `[0,n)` uses absolute indices `0 <= i < n`;
subviews retain their captured backing coordinates rather than rebasing i.

## Fixed structural rules

Every displayed arithmetic premise is submitted to the existing fixed numeric
derivations or to explicit finite `use` steps. E2 adds no solver, search,
timeout, cumulative budget, runtime enumeration, or inferred induction.

**Reserve.** `reserve_slots<T>(n)` requires the existing allocation-fit domain
for `(T,n)` and returns `None` on allocation refusal, producing no A or duty.
On success it creates fresh A, one descriptor whose immutable
extent is `n`, one `Dispose<A>`, and `Range<A,T,[0,n),Empty>`. Its executable
contract reserves `stride_ceiling(T)*n` suitably aligned bytes, initializes no
`T`, and publishes typed slot layout for exactly `0 <= i < n`. It performs no
element read, write, release, or construction. All size/stride overflow and fit
obligations are proved before the call. Provider effects and an infallible
proved-room variant remain U. This is a new fallible primitive contract, not a
disguised full array.

**Introduce full.** `certify_full(owner,lo,hi)` is admitted only for an
existing owned, live, typed backing A and a proved full initialized interval
`[lo,hi)` whose exact per-element duties are already held by `owner`, with
`0 <= lo <= hi <= extent(A)`. It moves those duties into one `Full` certificate;
it cannot assert initialization. The premise is an existing ownership-context
judgment, not the owner's type spelling or a scan of runtime bytes. The general
case has no v0.59 constructor for an arbitrary resource range, so the relocation
program receives its source Full family as a checked caller fixture. This rule
does not bootstrap linear values from nothing; a real element constructor must
supply every duty.

For the concrete ambient `box<u64>` witness, however, no extra family rule is
needed: start with `reserve_slots<box<u64>>(n)`, carry Full prefix plus Empty
tail, create `x=box_new(0_u64)` on each symbolic iteration, focus the Empty
tail head, `put` x, and join the returned Full singleton to the prefix. The
loop is exactly the destination half of the program below; `box_new` supplies
the actual fresh owner and its `allocates(heap)` effect under the baseline
external-resource assumption. No provider parallelism follows. General
linear issuers and provider-backed constructors remain U.

```text
// requires allocation_fit<box<u64>>(n)
let Some(b) certs { d, empty } = reserve_slots<box<u64>>(n) else { return None; }
let v=project(empty); let (f0,e)=split empty at 0; let f=zero_state(f0)
for i in 0..n invariant { Full[0,i), Empty[i,n) } {
  let (slot,e_next)=focus e at i; let x=box_new(0_u64)
  let slot1=put(v[i],move(x),slot); set (f,e)=(join(f,singleton(slot1)),e_next)
}
omit_zero(e) // b, d, and f are the concrete initialized family
```

**Project.** From `Range<A,T,[lo,hi),S>`, `project` forms
`View<A,T,[lo,hi)>` and leaves the certificate unchanged. It requires A live
and current compatible layout. Projection supplies no Full fact, element duty,
disposal duty, exclusion, or future validity. A projected `Full` range may be
read under ordinary rules; an `Empty` range may not.

**Focus.** Given `lo <= i < hi`, `focus r at i` consumes
`Range<A,T,[lo,hi),S>` and returns the ordered ghost partition
`Range<A,T,[lo,i),S>`, `Elem<A,T,i,S>`, and
`Range<A,T,[i+1,hi),S>`. Empty side intervals are valid. Ordinary
same-state joins may re-form any uniform part. A take followed by put can thus
re-form the original Full range; a consumed element may join an Empty prefix
but cannot be hidden in a Full range. The parent certificate is consumed, so a
second focus or duty projection at i fails structurally.
This is ghost decomposition, not a runtime view, loan, flag, token, or lock. It
neither revokes locator aliases nor grants exclusive access; alias reads still
use the current Full leaf and ordinary access rules.

**Split and join.** From `Range<A,T,[lo,hi),S>` and `lo <= k <= hi`, split
consumes it and returns `Range<A,T,[lo,k),S>` plus
`Range<A,T,[k,hi),S>`. Join consumes
`Range<A,T,[lo,k),S>` and `Range<A,T,[k,hi),S>` and returns the original
range. Backing, element type, boundary, and state must match exactly; different
A values, a gap, overlap, reversed order, or mixed states reject. At `k=lo` or
`k=hi` one result is empty and the same join premises apply.
For `lo=hi`, `zero_state` converts either state to the other for the same A, T,
and endpoints. Both contain zero duties; A must remain live and laid out and
`lo <= extent(A)` must hold. This is the only no-value state conversion.
`singleton` converts `Elem<A,T,i,S>` to `Range<A,T,[i,i+1),S>` and back.
Zero leaves are structural partition units and may be omitted, so focusing the
head of `[i,hi)` may bind only its element and right tail.
`focus_last [lo,i+1) at i` is the same focus with its zero right leaf omitted.

**Take and put.** `take(v[i],e)` requires v and e name the same A and i,
`e: Elem<A,T,i,Full>`, A live with current T layout, and an ordinary executable
write/initialization-state access to the slot. It moves the actual `T` value
and all duties contained in it to the result, leaves raw typed storage, and
returns `Elem<A,T,i,Empty>`. `put` requires the corresponding Empty certificate
and moves exactly one `T` owner into the slot, returning Full. Neither changes
`Dispose<A>`. A copied locator cannot duplicate either operation's linear
certificate premise.

Any alias operation that can change initialization or a contained duty must
consume the affected leaf and produce its exact successor, or it rejects while
that certificate is live. E2 defines no general replace transition. In
particular, releasing a child owner's referent through a saved locator while
its owner descriptor remains in a Full element rejects: it would leave a
phantom duty. The supported route takes the whole element owner, then releases
or destructures it. Taking the owner moves its child obligation; it frees
neither the child nor the container backing.

**Consume and release.** `consume(move(x))` uses T's existing explicit release
or whole-destructure route and discharges all of x's duties; it changes no slot
certificate. `release(b,d,r)` requires matching A and
`r: Range<A,T,[0,n),Empty>`, consumes the sole `Dispose<A>`, ends A, and
consumes r, then invalidates every locator and state fact for A. It runs no element cleanup.
Releasing with Full content, a partial interval, the wrong backing, or without
the duty rejects. Moving `Backing` may carry `Dispose` in a surface owner; the
judgments remain logically distinct so content ownership never authorizes free.
Any other end through an owner or locator rejects while a Range or Elem for A
exists. Even an all-Empty partition must first join to the whole range.

Certificate operations are fixed syntax with fixed-arity premises. Checking a
program visits only its written fixed-arity certificate steps and never expands
a runtime interval. Existing numeric derivation costs remain those of v0.59.

## Full discriminator program

The source parameter supplies the explicit runtime-many family fixture; its
construction remains outside E2. The concrete resource is v0.59 `box<u64>`, an
affine heap owner. `retire_box` is an ordinary checked body: its parameter's
unconditional R7/STOR-3 scope-exit cleanup consumes the box content and runs
the ambient heap free. Its summary records consuming x and ending its referent;
the absence of a writable ambient-provider path does not make the call pure.

```text
fn retire_box(x: own box<u64>) -> result: own unit
  transfers consumes(x) accesses ends(deref(x)) { return unit; }

fn relocate_and_clean<A>(
    src_b: Backing<A,box<u64>,n>, n: u64, k: u64)
  certs { src_d: Dispose<A>, src: Range<A,box<u64>,[0,n),Full> }
  -> Result<(), Backing<A,box<u64>,n>>
  error certs { src_d, src }
  requires 0 <= k, k <= n, allocation_fit<box<u64>>(n)
{
    let Some(dst_b) certs { dst_d, dst0 } = reserve_slots<box<u64>>(n)
      else { return Err(move(src_b)) certs { src_d, src } }
    let sv = project(src)
    let dv = project(dst0)

    let (se0,sf) = split src at 0
    let se = zero_state(se0)
    let (df0,de) = split dst0 at 0
    let df = zero_state(df0)
    for i in 0..n
      invariant {
        Range<A,box<u64>,[0,i),Empty>,
        Range<A,box<u64>,[i,n),Full>,
        Range<B,box<u64>,[0,i),Full>,
        Range<B,box<u64>,[i,n),Empty>
      }
    {
        let (sx,sf_next) = focus sf at i
        let (x,sx0) = take(sv[i],sx)
        let se_next = join(se,singleton(sx0))
        let (dx,de_next) = focus de at i
        let dx1 = put(dv[i],move(x),dx)
        let df_next = join(df,singleton(dx1))
        set (se,sf,df,de) = (se_next,sf_next,df_next,de_next)
    }

    omit_zero(sf,de)
    let src_empty = se
    let dst_full = df
    let (left,right) = split dst_full at k
    if k < n {
      let (e,tail)=focus right at k
      let (x,e0)=take(dv[k],e); let e1=put(dv[k],move(x),e0)
      set right=join(singleton(e1),tail)
    } else if 0 < k {
      let j = k-1
      let (head,e)=focus_last left at j
      let (x,e0)=take(dv[j],e); let e1=put(dv[j],move(x),e0)
      set left=join(head,singleton(e1))
    }
    set dst_full = join(left,right)

    let (de0,df) = split dst_full at 0
    let de = zero_state(de0)
    for i in 0..n
      invariant { Range<B,box<u64>,[0,i),Empty>,
                  Range<B,box<u64>,[i,n),Full> }
    {
      let (e,df_next)=focus df at i
      let (x,e0)=take(dv[i],e)
      retire_box(move(x))
      let de_next=join(de,singleton(e0))
      set (de,df)=(de_next,df_next)
    }
    omit_zero(df)
    release(dst_b,dst_d,de)
    release(src_b,src_d,src_empty)
    return Ok(())
}
```

`B` is the fresh identity returned by `reserve_slots`; the pseudocode reuses
the range names as loop-carried certificate bindings. At `n=0` both loops are
empty, `k=0`, no focus executes, and both empty backings release. At `n>0`,
`k=0` selects the right element and `k=n` selects the left element. A missing
`k<=n` proof rejects the split; a direct focus at `n` rejects its strict bound.
The loop invariant is a written finite schema checked initially, after one
symbolic body, and at the backedge; the checker never expands `0..n`.

## Boundaries and failures

- Executable mutation that changes state or duties must consume and replace its
  ledger leaf or reject; alias mutation is no exception. A linear certificate
  cannot be invalidated or kept stale. Reallocation/shifting to U requires an
  explicit transfer into a fresh U family; A views/certificates cannot name U.
- Early return, `break`, and error propagation must carry their actual finite
  partition and every extracted value. They may release only a whole Empty
  backing. Partial initialization can be unwound by a written cleanup loop over
  the initialized prefix; a shared exit that forgets the prefix boundary is
  still Q and rejects.
- Resource-specific loop overlap remains open. R15/PAR-2 retains direct
  same-affine-map `set_stmt` access for arrays/buffers, adjacent VIEW-2 ranges
  with exact `[s*i+b,s*i+b+s)` images through eligible EFF-2 helpers, and fixed
  reductions, including beside an untouched/read-only E2 ledger. It does not
  admit this inline take/put/consume loop. PAR-1 still admits declared calls
  that consume and return distinct noncopy roots when their full effects are
  disjoint. A new bounded R15 transition and lowering audit is required;
  shared captured writes still deny overlap.
- Separate A slots do not prove their stored owners' referents or provider
  metadata separate. Such operations need existing referent/provider grounds.

Retained current permissions include these explanatory shapes. Assume the
ordinary bounds and checked arithmetic obligations, `n <= len(data)`, stable
nonnegative width, and `parts*width <= len(out)`; a and b are distinct full
noncopy roots. The helpers access only their declared targets. Missing one of
these premises denies the corresponding permission:
```text
// with n <= len_of(deref(data)), request iteration overlap:
for i in 0..n { x=deref(data)[i]; set deref(data)[i]=x +wrap 1 }
fn fill(p: own MutSlice<u64>) -> result: own unit reads(p), writes(p) {
  let m=len_of(p); for j in 0..m { set p[j]=0_u64; } return unit;
}
for w in 0..parts { p=mut_slice_of(&uniq deref(out),w*width,w*width+width); fill(p: move p) }
fn forward(ticket: own Ticket) -> result: own Ticket pure { return move ticket; }
let left=forward(ticket: move a); let right=forward(ticket: move b) // request call overlap
```
An untouched E2 ledger must not deny the first two; consumed-own roots remain
PAR-1 footprints in the third.

## Adversarial rerun

| Source fragment | Verdict and decisive premise |
|---|---|
| `q=r; focus r at i; focus q at i` | Reject: Range is linear checker state; `r` cannot copy and the first focus consumes it. |
| `take(v[i],e); take(v[i],e)` | Reject: first take consumes Full e and returns Empty e0. |
| `join(Full[0,i),Full[i+1,n))` across `Empty[i,i+1)` | Reject: adjacency and uniform-state premises fail. |
| `put(view<B>[i],x,Elem<A,i,Empty>)` | Reject: backing identities differ. |
| `focus Range<A,[0,n),S> at n` | Reject: strict `i<n` premise fails. |
| `zero_state(Range<A,[0,1),Full>)` | Reject: endpoints are unequal. |
| `focus Full at i; read(alias[i])` for Copy `Int` | Accept with ordinary access premises: focus creates no exclusive borrow. |
| `replace(alias[i],owner)` while Full leaf lives | Reject: E2 has no transition consuming old leaf and returning an exact successor. |
| `linear struct Ticket{id:u64}; (t,e0)=take(v[i],e); return unit` | Reject: t's linear duty remains live on the exit. |
| `let Ticket(id)=move(t); return unit` | Accept: whole destructure is Ticket's explicit one-time linear discharge route. |
| `release(b,d,leftEmpty)` with `rightEmpty` live | Reject; `join(left,right); release(b,d,whole)` accepts. |
| `k=old_k; split r at k; set k=0` | Existing split keeps captured `old_k`. |
| `None=reserve_slots(n); return Err(move(src_b)) certs {src_d,src}` | Accept: refusal created no B and returns every source duty unchanged. |

One retained hole has concrete cleanup. `drain_range` is a nonrecursive checked
helper whose body is exactly the discriminator's second counted loop, parameterized
by its captured `[lo,hi)` rather than `[0,n)`:
```text
let (l,e,r)=focus full at k; let (x,e0)=take(v[k],e); retire_box(move(x))
let le=drain_range(v,l); let re=drain_range(v,r)
let all=join(join(le,singleton(e0)),re); release(b,d,all)
```
`drain_range` consumes one Full interval, returns the same interval Empty, and
is not an assertion or hidden conditional destructor. Scattered holes remain U.

## Local matrix rerun

These are E2-local derivations, not edits to the frozen matrix.

| Gap/cell IDs | E2 result | Remaining boundary |
|---|---|---|
| F: 1-17, 2-17, 10-17, 15-17, 17-20, 20-20 | D for E2 operations and the displayed `box_new` constructor | General issuers/providers, predicates, recursion, and sparse sets remain U. |
| F: 2-15, 3-15, 4-15, 5-15 | E2 fragment D: exact member duties and initialized prefixes | Original container witnesses remain U without P and a source constructor. |
| F: 4-23, 15-23 | E2 separation fragment D | Inline E2 transitions lack R15 permission; existing eligible helpers and PAR-1 calls remain admitted. |
| F: 14-17, 14-20 | U | E2 supplies no provider/child formation or dependency family (V). |
| Q: 11-13, 11-17, 11-20 | E2 fragment D for a written prefix or one retained selected hole | Unbounded sparse cleanup and hidden conditional drop remain U. |
| Q: 2-11, 11-15 | U | General partial aggregate cleanup and predicate-owned state are excluded. |
| 11-13, 13-24, 17-24 | E2 scoped C: sequential certificate erasure is specified | Physical lowering facts remain separate. |
| 13-15 | Original U | E2 exposes ranges but supplies no container predicate P. |

The concrete box constructor supplies the relocation fixture; general resources
still require a checked issuer. This tests F without claiming blanket U closure.
The hand derivation represents runtime-many ownership with constant-size written
loop schemas and erased interval certificates. Its
principal known losses are source verbosity, uniform-interval restriction,
unresolved general issuer/provider construction, and no new parallel permission.
This Sol challenge was a bounded hand derivation, not a compiler run or proof.
The only mechanical checks were file shape and `git diff --check`; every rule
verdict above remains research pseudocode to implement and test.
