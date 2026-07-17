# Sequential Storage Witness Derivations

Status: conditional paper derivations for hostile review, 2026-07-15. These
derivations test the checked resource-transition candidate. They do not select
source syntax, a proof logic, cleanup semantics, a language or specification
change, or a production mechanism.

## 1. Scope and assumptions

The payload `T` is sequential, uniquely owned, affine, region-free, and
borrow-free. Stored borrows, shared ownership, pinning, custom allocators,
external resources, FFI, and concurrency are excluded.

Let:

```text
Root(k, rho, T, C)
Dead(rho, S)
Live(rho, T, S)
```

denote an affine runtime storage root of kind `k`, a dead footprint containing
no `T`, and a live footprint containing exactly one owned `T` in each slot.
`rho` is generative. Separating composition is permitted only for disjoint
footprints. Root identities, footprint permissions, proof terms, focus tokens,
and invariant certificates erase.

The candidate assumes the following checked transitions:

```text
acquire_dead_trap<T>(C)
  -> exists rho. Root(heap,rho,T,C) * Dead(rho,[0,C))

release_dead
  : Root(k,rho,T,C) * Dead(rho,[0,C)) -> ()

split / join
  : exact footprint partition and recombination under a checked equality

init
  : address Root(k,rho,T,C) * exclusive Dead(rho,{i}) * own T
  -> Live(rho,T,{i})

take
  : address Root(k,rho,T,C) * exclusive Live(rho,T,{i})
  -> Dead(rho,{i}) * own T

borrow
  : address Root(k,rho,T,C) * locked Live(rho,T,S)
  -> root-and-version-tied borrow; the permission returns when the borrow ends

swap
  : address Root(k,rho,T,C)
    * exclusive Live(rho,T,{i}) * exclusive Live(rho,T,{j})
  -> Live(rho,T,{i}) * Live(rho,T,{j})

replace
  : address Root(k,rho,T,C) * exclusive Live(rho,T,{i}) * own T
  -> Live(rho,T,{i}) * own T
```

`init` and `take` require that no incompatible borrow rooted at the slot is
live. `split` and `join` erase. Relocation from live `i` to dead `j` is exactly
one `take(i)` followed by one `init(j)` only when both the intermediate and
successor footprints remain in the bounded shape grammar. The witness traces
below use boundary or hole movements with that property; arbitrary relocation
remains blocked. The affine conservation measure

```text
owned T in variables + T in live slots
```

is preserved by every transition except an intentional structural
destruction.

`swap` requires distinct disjoint slots and no overlapping borrow. It is one
checked live-shape-preserving transition because two nonadjacent takes from a
full range can create three live intervals, outside the bounded `Pair` grammar.
`replace` is also one shape-preserving transition: it installs the offered owner
and returns the displaced owner. A take-first decomposition may create three
live intervals when replacing an interior slot of `Pair`.

Four additional assumptions are explicit blockers rather than hidden parts of
the notation:

1. ordinary libraries can seal a fresh `rho` and its permissions behind an
   opaque nominal owner;
2. a normal-exit-closed focus opens that owner with an exact-use repacker;
3. existing full or inline storage can receive a checked generative root and
   full live permission without reinitialization; and
4. every abandonable owner has an executable exact disposal path.

The current language does not yet provide this package.

## 2. W-SMALL: inline-small sequence

For inline capacity `N`, the two representation states are:

```text
Inline(n):
  inline root = Live[0,n) * Dead[n,N)
  no heap root

Heap(n,C):
  inline root = Dead[0,N)
  heap root   = Live[0,n) * Dead[n,C)
```

Runtime state is the representation tag, length, heap pointer/capacity, and
inline bytes. The logical permissions add no runtime field.

For `n < N`, push isolates dead slot `n`, initializes it from the offered
owner, increments `n`, and reseals `Inline(n+1)`. Pop takes slot `n-1` and
reseals `Inline(n-1)`. Ordered removal takes the chosen slot and relocates the
suffix left.

Spill at `n = N` is:

1. acquire a dead heap root of capacity `C'` before opening the old owner;
2. if allocation traps under current OP-9, no recoverable postcondition exists;
   a future recoverable form must return the unchanged inline owner and offered
   `T` and remains OD-1;
3. relocate the `N` inline values to heap slots `[0,N)`;
4. initialize heap slot `N` from the offered `T`; and
5. reseal `Heap(N+1,C')`.

The success path performs `N` takes, `N+1` initializations, and no payload
clone. It allocates once only on spill. Automatic spill-back is not required.

Disposition: algorithmically derivable only if the basis includes checked
full/inline adoption, opaque generative sealing, exact focus, and executable
disposal. Heap-only dead acquisition is insufficient.

## 3. W-GAP: gap buffer

The state is:

```text
Gap(a,b,C) = Live[0,a) * Dead[a,b) * Live[b,C)
```

Runtime metadata is pointer, capacity, and endpoints `a,b`.

Moving the gap right by one takes slot `b`, initializes slot `a` with that
owner, and increments both endpoints. Moving left is the inverse. Moving by
distance `d` performs exactly `d` takes and `d` initializations and costs
`O(d)`.

Insertion at a nonempty gap initializes slot `a` and increments `a`. Deletion
after the gap takes slot `b` and increments `b`.

Growth first acquires a larger dead root. After acquisition succeeds, one
normal-exit-closed focus relocates the live prefix and suffix, releases the
now-dead old root, and seals the new two-range state. The focus admits no
recoverable early exit.

Disposal traverses only the two live ranges, takes and structurally destroys
each `T`, then releases the root. It never reads the gap as `T`.

Disposition: the transition trace is derivable with the bounded `Pair` shape,
but complete ordinary abandonment still depends on the unselected cleanup
model.

## 4. H-FLATSET: flat ordered set

The state is:

```text
Flat(n,C) = Live[0,n) * Dead[n,C)
            and SortedUnique(cmp,[0,n))
```

Runtime metadata is pointer, length, and capacity. Prefix liveness is a safety
obligation. Sortedness and uniqueness are ordinary set-correctness properties
unless the library explicitly exports them as checked contracts or optimizer
facts.

Search and all comparator calls finish before structural mutation; their
borrows end before focus begins. Duplicate insertion returns the offered owner
without mutation.

No-growth insertion at position `p`:

1. isolate dead slot `n`;
2. for `i = n` down to `p+1`, relocate `i-1` to `i`;
3. initialize `p` from the offered owner;
4. set `n := n+1`; and
5. reseal the prefix owner; ordinary algorithm reasoning and tests establish
   the set property unless a separate checked contract is requested.

This performs `n-p` takes and `n-p+1` initializations. Removal takes `p`,
relocates `[p+1,n)` left, decrements length, and returns the removed sole owner.

Growth acquires a new dead root, relocates entries below `p` unchanged and
entries at or above `p` to `i+1`, initializes `p`, releases the dead old root,
and seals. Under a future recoverable allocation contract, failure must occur
before focus and return the unchanged set plus offered owner.

Disposal takes and structurally destroys `[0,n)` and releases the root.

Disposition: the storage transitions fit the bounded prefix grammar. Complete
derivation additionally needs a checked loop invariant, behavior-call effects,
opaque sealing, focus, and cleanup. These are not supplied merely by counting
nine role names.

## 5. H-STORE: dense/sparse store

A dense/sparse representation uses:

```text
value root: Live[0,n) * Dead[n,C)
key root:   Live[0,n) * Dead[n,C)
position:   one fully initialized Copy buffer [0,U)
```

Lookup first rejects `k >= U`. Only then does it read `p = position[k]`; it
accepts occupancy only when:

```text
k < U and p < n and dense_key[p] = k
```

The prefix permission proves one live key and value for every dense index below
`n`. A stale `position[k]` is harmless because the `p < n` check makes both
dense reads safe and the dense-key equality rejects a semantic miss. Key
uniqueness and position coherence are ordinary map-correctness properties
unless explicitly exported as checked contracts or optimizer facts.

On successful insertion, capacity is secured before focus; key and value slots
`n` are initialized, `position[k]` is set to `n`, length is incremented, and
the joint invariant is resealed. Duplicate or out-of-universe insertion returns
the offered owner unchanged.

Removal at dense index `p` takes and returns `value[p]` and also takes the
corresponding `dense_key[p]`. The removed key is structurally destroyed after
its position metadata is no longer needed. If `p != n-1`, removal then takes
the final key/value pair, initializes both at `p`, updates the moved key's
position, and decrements `n`. If `p == n-1`, both final slots are already dead
before the decrement. Thus key and value roots both end as
`Live[0,n-1) * Dead[n-1,C)`. The operation performs `O(1)` payload moves and no
clone. Clear destroys both dense prefixes and sets `n=0`; it need not clear all
`O(U)` position entries.

Growth acquires both replacement dense roots before focus. A future recoverable
contract must release a first wholly dead acquisition if the second fails while
preserving the old store and offered owner. After both succeed, prefixes
relocate and old roots release.

Disposition: the dense/sparse algorithm needs no universal safety bitmap,
container-specific intrinsic, or general sparse-liveness proof. `p < n` plus
bounded prefix liveness authorizes initialized reads; key equality controls the
ordinary abstract result. An alternative layout with payloads stored directly
in sparse bucket slots would need a checked occupancy-to-liveness relation,
runtime slot tags, or another explicit safety disposition, but that layout is
not required merely to express H-STORE. The frozen requirement that H-STORE
exercise `ST-SPARSE` must be re-adjudicated as a possible mechanism
overconstraint before it gates the common basis.

## 6. Cleanup lower bound

Every abandonable owner requires executable cleanup:

```text
dispose_I : Owner<I> -> ()
```

Its proof must establish that execution:

- takes and destroys every live affine payload exactly once;
- never reads or destroys a dead slot;
- retains each root until its final payload is gone;
- releases each root exactly once; and
- has no recoverable or early normal exit.

The certificate erases; traversal, payload destruction, and deallocation do
not. A predicate cannot execute the required `O(n)` loop.

The remaining mechanisms are exact-linear manual close or a restricted
verified disposer. The latter is operationally a verified user destructor and
requires explicit termination, recursion, effect, allocation, trap, callback,
and nested-resource rules. Current STOR-3 selects neither generalized form.

## 7. Result

The transition algebra is a plausible common machine for all four algorithms,
but it is not a complete public basis as written:

- W-SMALL additionally requires full/inline adoption;
- every witness requires generative opaque sealing and exact focus;
- every abandonable witness requires executable cleanup; and
- H-STORE's dense-prefix safety route fits the bounded grammar, while its
  abstract map invariant remains ordinary correctness.

This is a useful separating result. Neither a privileged named-container
catalog nor a universal runtime bitmap has been shown necessary for these
traces. The traces also show that a small list of transition verbs alone does
not solve ordinary-library sealing, focus, cleanup, or performance.
