# Whitefoot container source

These are reusable libraries written in ordinary Whitefoot. Bundle a library
source before its caller on the compiler command line; this directory gives
its code no special source-language status, import behavior or native ABI.

| Source | Operations and boundary |
| --- | --- |
| [grow-vector.wf](grow-vector.wf) | Reserve, growing append, indexed insertion/removal, and ordered consumption over `Box<Slots<T>>`. |
| [deque.wf](deque.wf) | Reference-taking endpoints, logical visitation and drain over `Box<Ring<T>>`; explicit consuming rebase produces a new backing. No automatic endpoint growth or zero-copy two-span interface. |
| [slab.wf](slab.wf) | One bounded backing with lazily materialized slots, generation handles, reuse, expiry, exhaustion returning the offered owner, and explicit consumption. Handles are relative to the supplied slab. |
| [hash-map.wf](hash-map.wf) | Generic owning keys and values with supplied hash/equality, growing insertion, removal, borrowed lookup/edit, rehash, and explicit consumption. No stable bucket index or payload address. |

The caller-selected capacity ceiling supplies the size bound for each concrete
allocation. See [the writer pattern](../../docs/patterns.md#p2-choose-the-storage-shape-from-its-occupancy-rule)
for the contracts and [the container investigation](../../research/investigations/containers-and-resources/X1-LIBRARY.md)
for comparison evidence and remaining interface limits. These evolving libraries
do not claim native parity for every operation.

`HashMap<K, V, ceiling>` uses a `HashMapKey` binding for hash and equality.
`hash_map_len` and `hash_map_capacity` report its logical length and usable
capacity. `hash_map_put` returns `HashMapInserted` or `HashMapReturned`, whose
fields are `reason` and `pair`: `HashMapReplaced` returns the old complete pair
after installing the offered pair, and `HashMapFull` returns the offered pair.
`hash_map_try_put` keeps the current capacity; `hash_map_put` grows only after a
full probe finds no reusable slot, taking capacity zero to one and otherwise
doubling or saturating at the ceiling. `hash_map_reserve` is a no-op at or below
current capacity and refuses requests above the ceiling; `hash_map_rehash`
rebuilds at the same capacity. Capacity refusal is not an allocation-failure
outcome.

`hash_map_lookup` and `hash_map_each` borrow stored pairs for callbacks.
`hash_map_edit` permits its callback to change the value while reading the key,
and the callback can return ordinary owned data. `hash_map_remove` returns the
removed owned pair. Supply a `HashMapConsume` binding to `hash_map_free` and
`hash_map_pair_consume` to consume every key and value, including `nodrop`
owners. Hash/equality consistency determines ordinary map contents; ownership
and bounded probing do not assume those laws. Neither iteration order nor
stable indices are promised.

From the repository root, after building `whitefootc`:

```sh
compiler/target/gate/whitefootc lib/containers/grow-vector.wf tests/programs/containers/grow-vector-program.wf -o /tmp/whitefoot-grow-vector
```

The callers and shared allocation observer stay in `tests/programs/containers/`.
The Slab caller also bundles `slab-membership-program.wf`, which exercises weak
indexes and a composite retained-membership protocol. The ordinary Rust corpus
tests bundle each library with its caller, execute sequential and parallel
outputs, and check the exact release ledger. They run
through canonical `make check`; there is no separate library test stage or
Makefile. Research experiments remain explicitly invoked outside that gate.
