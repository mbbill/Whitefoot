# Whitefoot container source

These are reusable libraries written in ordinary Whitefoot. Bundle a library
source before its caller on the compiler command line; this directory gives
its code no special source-language status, import behavior or native ABI.

| Source | Operations and boundary |
| --- | --- |
| [grow-vector.wf](grow-vector.wf) | Reserve, growing append, indexed insertion/removal, and ordered consumption over `Box<Slots<T>>`. |
| [deque.wf](deque.wf) | Reference-taking endpoints, logical visitation and drain over `Box<Ring<T>>`; explicit consuming rebase produces a new backing. No automatic endpoint growth or zero-copy two-span interface. |
| [slab.wf](slab.wf) | One bounded backing with lazily materialized slots, generation handles, borrowed lookup/edit, reuse, expiry, exhaustion returning the offered owner, and explicit consumption. Handles are relative to the supplied slab. |
| [hash-map.wf](hash-map.wf) | Generic owning keys and values with supplied hash/equality, growing insertion, removal, borrowed lookup/edit, rehash, and explicit consumption. No stable bucket index or payload address. |
| [priority-queue.wf](priority-queue.wf) | Growing boxed binary heap with supplied comparison, proved-nonempty borrowed peek and owning pop/replacement, bottom-up heapify, ordered drain, and explicit final consumption. Indexed operations additionally report resident positions and support arbitrary-position removal/replacement. |

The caller-selected capacity ceiling supplies the size bound for each concrete
allocation. See [the writer pattern](../../docs/patterns.md#p2-choose-the-storage-shape-from-its-occupancy-rule)
for the contracts and [the container investigation](../../research/investigations/containers-and-resources/X1-LIBRARY.md)
for comparison evidence and remaining interface limits. These evolving libraries
do not claim native parity for every operation.

`SlabVisit` and `SlabEdit` lend a payload only after checking its handle's
index, generation and occupancy. `slab_visit` supplies a read-only value;
`slab_edit` permits its member to write the payload and return ordinary owned
data while preserving the outer storage length and capacity. Missing or stale
handles return `Err` without calling the member. Neither operation returns a
reference or authenticates a different Slab. The edit environment must be
disjoint from the Slab's cells under the ordinary effect-path rules.

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

`PriorityQueue<T, ceiling>` starts empty at zero capacity. A `PriorityOrder<T, E>`
binding compares borrowed elements using a borrowed environment; a negative
comparison puts the left element first. `priority_queue_push` grows from zero
to one, then doubles or saturates at the ceiling, and returns the offered owner
in `Err` when the length has reached that ceiling. `priority_queue_reserve`
takes a caller-proved total within the ceiling. `priority_queue_len` publishes
the relation used to prove nonemptiness for callback-based `priority_queue_peek`,
owning `priority_queue_pop`, and `priority_queue_replace_top`.

`priority_queue_heapify` consumes an initialized `Box<Slots<T>>` and builds the
heap bottom-up in O(n), without allocating. `priority_queue_drain` consumes in
pop order in O(n log n) while retaining capacity; `priority_queue_free` consumes
in reverse physical-slot order in O(n) and releases the backing. Both accept
explicit consuming callbacks for arbitrary elements, including `nodrop` owners.
Ordering requires a consistent comparator and environment, but bounded sift
progress and ownership preservation hold for every returning comparator. Equal
priorities have no stability guarantee, and no operation promises stable slot
identity or an escaping reference.

The indexed candidate uses the same `PriorityQueue<T, ceiling>` and shared
sifts. `priority_queue_push_indexed` and `priority_queue_heapify_indexed` have
the ordinary ownership outcomes and additionally report initial placements
and every exchanged resident position. `priority_queue_remove_at_indexed`
returns the selected owner and repairs the entry installed from the end;
`priority_queue_replace_at_indexed` exchanges an offered owner at the selected
position, returns the old owner and repairs in either direction. Both require
`index < queue.storage.inner.len`. `priority_queue_peek_at` lends an entry at
that same proved bound so a caller can validate its identity before mutation.

Each indexed mutation accepts `order_env` and `position_env` separately. Its
generic arguments are a `PriorityOrder` binding, the position-environment
type, a function-kind `placed` argument, and the ceiling. The callback has
`fn placed(env: &P, value: &T, index: u64) -> result: unit reads(value), writes(env)`;
it receives only entries still resident in the heap. The caller separately
retires a removed or replaced identity's membership. Position state must be
disjoint from the queue and comparator state. A handle-based consumer must
validate the current position and complete handle; a saved index is neither
stable identity nor a standing bounds proof. Correct ordering and reverse
metadata depend on the supplied behaviors, while heap bounds and bounded sift
progress do not assume their logical laws.
Reports are sequential, with the first update visible to the second callback.
A faithful reporter restores the complete reverse map by operation return.
While reverse metadata is live, the caller uses indexed mutations throughout;
plain mutations on this same queue type do not maintain that metadata. The
library checks each supplied position's bound, while the application owns the
entry-identity and membership protocol.

Plain operations retain their existing public signatures and use the shared
sifts with a no-op reporter. The indexed-composite
[trial](../../research/investigations/containers-and-resources/X1-LIBRARY.md#indexed-composite-trial)
compares that specialization with the preserved plain queue and a standalone
indexed control; source reuse alone is not evidence of unchanged executable cost.

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
