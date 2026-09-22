# Containers over the x1 language

This investigation now uses the merged PR #70 baseline,
`36be8784e84a26d34bc24668babd789e0f4c96fb`, kernel v0.60. The initial study
used the earlier published snapshot `efd6ebc9`; its dated observations remain
in the [probe results](../../experiments/container-representation/x1/RESULTS.md).
The merged-baseline reassessment below supersedes outstanding-work claims
about that snapshot. Compiler implementation failures are distinguished from
specified language limits. The active
[specification](../../../spec/kernel-spec.md) remains the language authority.

The question is whether full system-container operations can be expressed at
competitive cost using the existing framework, and which implementation or
measurement should come next. One global heap, local nonescaping references,
structural copy/drop capabilities, ordinary owned values, and monomorphized
`interface`/`binding` behavior are premises, not alternatives reopened here.
Older provider/refusal and region contracts in [REASSESSMENT.md](REASSESSMENT.md)
and [FOUNDATION.md](FOUNDATION.md) are dated evidence, not requirements of this
library. A checked branch is acceptable when its cost meets the operation's
requirements; inability to erase it is not itself a language gap.

## Evidence and comparison criteria

The [external workload traces](EXTERNAL-WORKLOADS.md) supply application
contracts from pinned Rust, C++ and Go programs, including the later Linux,
Redis and SQLite extension. They do not measure prevalence. Application needs
must be separated from the host language's chosen representation: e.g. stable
logical identity does not by itself require pointer-linked nodes.

Use these labels throughout the assessment:

- **Rule deduction:** follows from the cited v0.60 rule, not a compiler run.
- **Source observation:** present in pinned source or generated lowering.
- **Checked experiment:** an identified program passed an identified binary.
- **Measured control:** an identified native model, not WF performance.
- **Candidate:** an interface/representation awaiting implementation evidence.

Before any new selection experiment, keep the operation contract and its
observable result fixed. For each representation record construction, lookup,
mutation, growth and cleanup; occupied and reserved bytes; allocations;
element transfers; and the facts required across helpers. Test copy, owning
droppable and must-consume elements separately where relevant. No failed
program may be called a language limit when it contradicts the specification.

The initial discriminators are:

1. Complexity: drain must move O(n) elements, ordinary probing must terminate
   after at most capacity probes even with hostile hash/equality, and rehash
   must account for its new allocation plus old live elements. Reject an
   implementation that meets only a small fixed-size timing while violating
   the promised complexity.
2. Sparse storage: compare one backing of valid enum slots with boxed payloads
   and with a dense payload store plus sparse indexes. Count all metadata and
   identity-repair work; do not charge a tag only to one side or assume niche
   encoding in a WF layout.
3. References: distinguish stable slot identity, stable physical addresses and
   retained membership. Compare the same removal/invalidation contract.
4. Construction: retain helper boundaries and separate the irreducible move
   between different storage from avoidable temporary/result transfers.
5. New proof or storage machinery needs a concrete consumer and evidence of
   the remaining cost after the best ordinary representation. No universal
   percentage threshold is invented for all workloads.

## Scope of the resulting comparison

The common families are growable vector, ring deque, generational slab,
owning hash map, priority queue, and an ordered tree. The ceiling challenges
are multiple indexes retaining one object, large-value construction and
compact variable-sized records. A composite in-memory index workload will
connect the families without being described as a production database.

This note is the current home for this comparison, with experimental evidence
linked at its source. It will retain its pinned conditions when subsequent
implementation replaces individual candidate conclusions.

## Findings and boundaries

The six families have plausible ordinary-value representations. Two narrow
owned-element operations now have checked/native evidence in the
[x1 probes](../../experiments/container-representation/x1/RESULTS.md): indexed
vacancy exchange with a must-consume element, and a generic ordered drain with
linear element movement. This is not evidence that six complete libraries
already exist, or that their native costs are at parity.

Three different questions must not be collapsed:

| Kind | Example | Consequence |
| --- | --- | --- |
| Library algorithm choice | Repeated `remove_at(0)` versus reverse then `take_back` | A linear-time ordinary alternative exists; no new primitive follows from the quadratic source. |
| Specified interface limit | A range over a Ring, even an empty/unwrapped one, is refused by REF-4 | A generic two-span API is unavailable on that representation. A fully initialized copy-element Array is a different available representation. |
| Resolved snapshot defect | The earlier reserve helper supplied unrestricted u64 capacity to `grow` | OP-9 requires a size bound. The merged library supplies one through `ceiling`; the unbounded research negative remains correctly rejected. |

The candidate layouts below are recommendations for implementation trials,
not changes to language decisions or adopted library interfaces. No live-tree
revision is proposed by this research. Existing temporary-reference, global-heap,
no-hole and no-stored-reference choices remain premises. Their grounds are in
the [data-model](../../../design/language/data-model.md),
[ownership](../../../design/language/ownership.md) and
[generics](../../../design/language/generics.md) decisions; rules, not those
records, determine acceptance.

## Common families: representations and complete operations

Independent source sketches below omit bodies and are not a single bundle:
their names illustrate interfaces, not final exported names. `T` is unbounded
when the operation can support must-consume elements. `T: drop` is an explicit
extra requirement only for operations that discard elements. A copy instance
of an unbounded generic body retains its template's move/swap spelling under
FN-2; separate copy algorithms are performance alternatives, not a requirement
to duplicate every library.

| Family | First ordinary candidate | Complete operation chain | Proof and ownership obligations | Cost obligations, not measured WF parity |
| --- | --- | --- | --- | --- |
| Vector | `Box<Slots<T>>`; `Slots<T,N>` for inline bounded use | Construct, reserve, append, insert, ordered remove, swap-remove, truncate, ordered consume/drain, final release | OP-9 on growth; index/room facts; exact count changes over references; a consume callback for nodrop T; `free_empty` after complete consumption | O(1) no-growth append; amortized O(1) under geometric growth; O(n-i) ordered insert/remove; O(1) swap-remove; O(n) drain. No owner round trip for mutation. |
| Deque | `Box<Ring<T>>`, fresh ring plus `append` for growth; fixed Ring for bounded queues | Push/pop both ends, wrap, indexed access, grow/rebase, consume, release | Bounds in logical coordinates; front operations invalidate old slot references; new backing invalidates every old path; source emptied before releasing it | O(1) endpoints and logical access; O(n) growth; one visit per drained element. Generic two-span consumption remains a separate unavailable interface, discussed below. |
| Slab | `Box<Slots<Cell<T>>>`, a free-list head and generation handles; every materialized Cell is a valid enum | Insert/handle, validate/get, remove, expiry, reuse, exhaustion/limit outcome, grow if selected, final consumption | Swap a valid vacancy with an occupant; return every removed T; never wrap a generation into an old handle's generation; distinguish a slot ID from a physical address | O(1) free-list operations and validation; no payload movement on ordinary reuse. Backing growth moves inline cells; boxing each payload trades that for allocations and indirection. |
| Hash map | One window of valid `Vacant/Deleted/Occupied(K,V)` slots; compare a dense-entry/index-table alternative for large payloads | Construct, collision insertion, duplicate replacement, lookup after tombstone, remove, reuse, growth/rehash, iterate, consume | K and V need not be copy/drop if replacement returns the old pair and final cleanup explicitly consumes them; bounded probing; no assumed behavior laws; preserve every owner during rehash | Hash/equality cost plus probes; ordinary load-dependent expected constant access, capacity-bounded worst-case lookup. Rehash scans old capacity and reinserts live entries; adversarial collisions can make it quadratic. |
| Priority queue | Slots of T, with an explicit comparison behavior; indexed variant adds a reverse-position map | Push, peek via a local reference/callback, pop, replace top, change priority/remove by handle when selected, heapify, drain | Compare borrowed T; exchange slots then take a boundary value; every sift step progresses along an index; arithmetic domains and child bounds proved independently of comparator laws | O(log n) sifts and O(n) bottom-up heapify; no aggregate owner transfer per level. Count reverse-index repairs in an indexed queue. |
| Ordered map | Pool-indexed B-tree nodes with fixed-capacity key/value and child windows; compare owned Box-linked nodes | Find, insert, split/promotion, replace, remove, borrow/merge, root contraction, ordered iteration, cleanup | Node and child bounds; distinct node indexes at simultaneous writes; rotations return/move all owners; no key duplication assumed; index stack for paths | O(log n) node visits at balanced height, O(B) local shifts, O(n) full traversal. Bound retained path storage; charge pool validation and node fragmentation. |

The map witness in
[owning-behavior.wf](../../../tests/programs/containers/owning-behavior.wf)
is useful existing code but binds `K: drop` and fixes its payload to a Box.
It does not establish the unbounded K/V row above. The
[priority witness](../../../tests/programs/containers/priority.wf) is a fixed
u64 queue, and [ordered.wf](../../../tests/programs/containers/ordered.wf)
is explicitly one leaf split. Each has a narrower claim than a reusable library.

### Vector: same order without quadratic drain

An ordered drain can first swap symmetric elements, then repeatedly take the
last element and call a monomorphized consuming behavior. The probe consumes
four nodrop Items in the sequence 1,2,3,4 and empties the storage. The callback
receives its own environment and the current element, not access to the vector;
its effect boundary must exclude observing the reordered remaining contents.
No callback can store a reference under REF-3. This candidate does not promise
that partially processed contents stay in original order while a callback is
running, nor a pause/resume drain object.

At the abstract element-transfer level a conventional swap uses three moves.
For n elements the reverse-then-pop algorithm uses at most
`3*floor(n/2) + n` moves, against `n*(n-1)/2` shifts plus n extractions for
repeated front removal, and n extractions for a native queue-front drain.
For n=4096 these are 10,240, 8,390,656 and 4,096 respectively. These counts
exclude helper/result ABI transfers, callback work and optimizer elimination;
they are not timings. Slots can meet O(n) today, but this does not claim the
smallest possible constant or solve final-slot construction.

Reserve has a source size ceiling even though heap allocation has no refusal
arm. A capacity limit is a quantity/representation contract, not an OOM
protocol. For a concrete u64 backing, `requires count <= 1024_u64` makes the
count multiplication plainly representable. The merged generic library takes
an explicit const ceiling, with each T/ceiling instance checked against OP-9.
This filters supported instances; it is not a target-independent runtime
`sizeof<T>` query and not a blanket proof for every possible T. The public
choice between a caller-proved maximum and a genuine application Full outcome
must be explicit. Never add a fake OOM or impossible fallback to satisfy OP-9.

### Deque: endpoints and span consumers are different contracts

The ordinary growth candidate allocates a larger Ring, appends the old logical
window into it, then swaps the owning Box into the deque and consumes the now
empty old backing. For nodrop T the zero-length fact must reach the owned
backing that `free_empty` consumes; after a swap or an ordinary owned helper
return, check whether its declared contracts preserve that fact. An explicit
length reread is a possible validation cost; silently dropping the old owner
is not. This complete generic growth chain has not been checked here.

The merged append row now also publishes
`destination.len >= entry(source).len`. That is useful when the destination
starts empty, but it still neither publishes the two-entry sum nor transports
the old backing's zero-length fact across a later owner exchange. Reuse the
stronger existing row before proposing any additional contract mechanism.

The [linear-ring-publish probe](../../experiments/container-representation/x1/linear-ring-publish.wf)
tests another route: append first, then atomically call an ordinary helper
that requires the old backing empty, consumes it, and returns the new Box.
The compiler rejects its `set deref(values) = publish(...)` under WIN-3.
That agrees with the active text: OP-12 admits its atomic form only for affine
and copy targets, and WIN-3 refuses assignment to a linear target. Neither an
unbounded T nor a nodrop instance qualifies. Thus this route is unavailable
under the current rules even though no old element would be implicitly dropped.
This does not prove that every ordinary growth algorithm is impossible.

The candidate's atomic paragraph and the tree describe admission without that
class restriction; the candidate also generally refuses linear assignment.
X1-P3 below therefore asks the owner to clarify their intended scope, rather
than treating the omitted qualifier as proof that linear updates were selected.
No broadened atomic rule is assumed here. A by-value rebase consuming and
returning a genuinely new backing, or a different deque representation, is a
separate interface candidate if the restriction is intentional. The first
library trial must settle a complete nodrop growth route before claiming an
unbounded growable Deque.

REF-4 deliberately refuses *all* Ring range references. Therefore a function
that accepts two `&[T]` extents in queue order cannot obtain them directly
from Ring, even after testing that a subrange does not wrap.

For a byte queue, or another copy T, an ordinary fully initialized Array plus
head/count can implement the physical ring. It may pass the tail and beginning
as two local range arguments without relocating data. The copy-ring-spans
probe exercises the ordered physical ranges; it is not a complete queue.
Initialization of spare capacity is an extra cost, paid at construction/growth,
and must be compared with the actual native consumer contract. A native
uninitialized spare-byte implementation must not be charged artificial zeroing
merely to make the WF comparison agree.

For arbitrary owning T there is no universal filler. An Array of enums gives
ranges of enums, not ranges of T. The available alternatives are single-slot
visitation on Ring or O(n) transfer into contiguous Slots; those do not meet a
strict zero-copy two-span contract. This is the exact scoped interface limit,
not evidence that queues in general require a language amendment.

### Slab: reuse, addresses, and retained membership

One representation sketch is:

```wf
struct Handle {
  index: u64;
  generation: u64;
}

enum Cell<T> {
  CellEmpty(generation: u64, next: Option<u64>);
  CellLive(generation: u64, value: T);
  CellRetired();
}

struct Slab<T> {
  cells: Box<Slots<Cell<T>>>;
  free_head: Option<u64>;
}
```

Remove swaps the indexed Cell with a valid local vacancy, consumes the returned
enum and hands its T to the caller. A slot whose generation reaches the maximum
is retired instead of wrapping; a reduced-width model should force this path
in the next implementation's tests. A handle is interpreted relative to the
supplied slab. Without an additional application identity field it does not
authenticate a different slab. These are ordinary data contracts under OP-13,
not new brands or memory-safety authority.

`find_index` may return `Result<u64, unit>` with
`ensures when Ok(value: i): i < deref(slab).cells.inner.len;`.
Generation and variant are validated on the path before access. A callback
receives a locally formed `&T`; REF-3 precludes returning or storing it. FN-9
does not currently export arbitrary indexed-field or variant relations, so a
second helper may need another read/match. Price that cost on a lookup chain;
do not describe it as lost runtime state or a need for quantified occupancy.

Stable slots mean stable index/generation, not stable addresses. Growing a
window of inline records moves their storage. A window of Box records keeps
pointee addresses but spends one allocation per record; source references still
obey REF-2 and do not survive merely because machine addresses happen to agree.

Two memberships have two different valid application contracts:

- **Weak indexes:** the central slab owns the object; either index may outlive
  deletion and later report Expired. Generation checks give this cheaply.
- **Retained indexes:** deletion checks an ordinary member count and returns
  Busy while any membership remains. One composite owner updates indexes and
  counts together; explicit consuming tickets can prevent accidental ticket
  loss. Bounds and checked counters protect memory/overflow, but the current
  language does not prove that arbitrary separately authored functions maintain
  the global count-to-index relation. Tickets are not unforgeable capabilities
  merely because a source struct is nodrop. Ordinary constructors and direct
  mutations remain available.

Thus a checked WF protocol with defensive validation is a viable trial, but
an unrestricted library has no demonstrated static retained-membership theorem.
The existing [Rust control](../../experiments/container-representation/authority/RESULTS.md#two-indexes-and-object-lifetime)
distinguishes the contracts, not WF acceptance. The next trial must include
wrong index/generation, removal from one index, a remaining reader, final
membership retirement, and final store destruction.

### Hash map: representation before occupancy machinery

Start the complete generic trial from a valid slot enum:

```wf
enum MapSlot<K, V> {
  EmptySlot();
  DeletedSlot();
  FilledSlot(key: K, value: V);
}

interface Key<K, E> {
  fn hash(env: &E, key: &K) -> hash: own u64 reads(env), reads(key);
  fn equal(env: &E, left: &K, right: &K) -> equal: own Bool reads(env), reads(left), reads(right);
}
```

An insertion returns its previous slot or the replaced key/value pair, so a
duplicate need not drop an unbounded K. Removal returns an owned outcome.
Final cleanup takes every materialized slot and consumes each active payload,
then frees the empty window. The vacancy-exchange probe establishes one of
these operations, not the entire rehash protocol.

Hash/equality are not assumed reflexive, consistent, terminating or constant
time. Conditional on callbacks returning, public probe loops stop after
capacity inspections; comparator/equality answers cannot authorize an
out-of-bounds access or an ownership loss. Existing entries can be transferred
by occupancy rather than re-deduplicated with a newly inconsistent equality.
Rehash needs an ordinary owner-preserving insertion protocol. A supposedly
impossible Full arm that invents a value, leaks the pending entry, or silently
loses it is not an implementation. Complete checking of that protocol remains
part of the library trial; no per-slot proof is assumed here.

Let C be table capacity, n occupancy, P the entry stride, and I an index-slot
stride. These are comparison models, not promises of WF ABI layout:

| Representation | Storage to count | Rehash/mutation consequences |
| --- | --- | --- |
| Inline enum slots | C times the actual enum stride, including discriminant/padding; optional fingerprint storage separately | Rehash moves live K/V payloads. One backing, direct payload access. |
| Enum/optional Box slots | C times slot stride, plus n allocations of P and allocator overhead | Moves pointers during table rebuild; dependent payload load and allocation/free costs. No assumed Option niche. |
| Sparse indexes plus dense entries | C*I plus reserved dense capacity*P plus reverse-location metadata | Table-only rehash can leave payloads in place; deletion repairs the moved last entry's index. Dense growth still relocates payloads. |

Compare all three at the same occupancy and identity contract. The independent
[hash-slot study](https://github.com/mbbill/Whitefoot/blob/38c28403a2defd0b65b8a2ab2b5e4794315e9940/research/experiments/hash-slot-occupancy/RESULTS.md)
did not meet its registered criterion for a recurring tag-check tax and found
more consistent costs for boxing in its large-table cells. It is a native C
representation study with some noisy controls, not WF emitted-code evidence or
a comparison against SIMD-group probing. It argues against adding a projected
layout solely because one branch exists, not for a universal winning layout.

### Priority queue and ordered map

A priority queue compares two local references through an interface and swaps
their slots. Pop can swap root with the last slot, take the last slot, and sift
the new root. Unbounded T supports this spelling, including a copy instance.
With inconsistent comparison the returned order need not be meaningful, but
each sift must move strictly toward a parent or child and stay within checked
bounds. An indexed queue additionally repairs handle-to-position metadata on
every swap, matching the Kubernetes workload rather than a simpler bare heap.

For an ordered map, use a B-tree as the first move-only-key candidate: a split
*moves* the median key/value into a parent. A B+ tree that duplicates a leaf
key into an internal separator needs a copy key, an explicit cloning behavior,
or a distinct separator representation. The leaf-split u64 fixture cannot
establish that operation for arbitrary K. Full binary and B-tree alternatives
must compare allocation count, fanout, shifts and traversal cost, not just
whether one syntax is shorter.

Pool-indexed nodes make descent a scalar-ID loop under REF-1. Store an explicit
index path for rebalancing and reacquire references after growth or mutation;
avoid silently re-searching from the root after every iterator step. Box-linked
nodes may use recursion, but loop-carried path extension is refused. The
[wildcard-path investigation](https://github.com/mbbill/Whitefoot/blob/dafff1748603530e867cd7d7c164a35b78f8f5a9/research/investigations/wildcard-path/REPORT.md)
identifies the refinement and finite-analysis work behind that proposed
extension. Its proposal is not assumed present in this baseline. Compare
pool descent and recursion first; an extra mechanism needs the remaining
performance or resource problem as its consumer.

## Ceiling challenges connected to real source contracts

| Challenge and external pressure | Ordinary x1 candidate | What remains to establish |
| --- | --- | --- |
| Redis incremental rehash: lookups can also advance migration | Two owning tables, migration cursor, explicit write row, bounded migration budget; query both tables until done | Complete partial-progress ownership and per-operation work bound. An eager rebuild is not the same latency contract. |
| DuckDB aggregate rows pinned while a sparse table grows | Slab/index table when logical identity suffices; boxed or segmented rows for fixed addresses | Distinguish address stability from a legal surviving reference. Include retention lifetime and address lookup/indirection costs. |
| DuckDB string construction into final payload storage | Full byte storage plus direct bounded filling, or an initialized Slots prefix built sequentially | Spare-byte initialization and extra aggregate transfers. No inference that `move` or OP-12 guarantees destination placement. |
| Redis listpack and SQLite variable-length page records | Initialized byte Array/Slots, offsets, codecs, local shifting and compaction | Encoding/decoding and validation cost, overlapping byte movement, relocation fixups and peak scratch. No typed pointer reinterpretation is required by the byte contract. |
| SDS-style pointer-sized handle with typed header immediately before dynamic tail | Existing boxed byte block with encoded header; compare an inline header plus Box data | A general source struct with a runtime-sized typed tail is forbidden by TYPE-9. `Box<Record>` containing a second Box uses two objects; plain Record plus Box data uses one allocation but a wider handle and separated header. Bytes are a real alternative with codec costs. |
| Large owning records or fixed inline arrays appended through helpers | Current ordinary construction followed by placement; inspect destination forwarding in optimized IR | Per-element temporary/result transfers with helpers retained; failures during input-driven construction must consume the initialized prefix. Heap exhaustion is not that failure case under STOR-8. |

The pinned [external traces](EXTERNAL-WORKLOADS.md#focused-revalidation-contracts-and-host-mechanisms)
already separate workload obligations from Vec, pointer, arena, or dictionary
choices in the host language. Their old WF adaptation comments are historical;
the candidates here use x1. No upstream application was newly benchmarked.

A useful composite trial is a small in-memory record index, not a production
database: a Slab owns records, a HashMap maps IDs to handles, and an indexed
priority queue schedules retirement. Feed inserts, duplicate replacement,
priority updates, deletion, expiry and slot reuse; verify a sorted-vector oracle
and a per-record consumption ledger. Run separate weak-index and retained-index
contracts, rather than comparing them as interchangeable implementations.
Force generation exhaustion with a bounded test representation. Add variable
payload sizes and a batched byte-page variant to expose boxing and encoding
tradeoffs. The ordered-tree trial separately adds range scans and rebalancing;
it need not make this first composite program larger.

For later matched measurements use distinct construction and steady-state
phases, capacities spanning cache regimes, small and large payloads, controlled
collision distributions, retained and normally inlined helpers, and explicit
allocation/transfer/peak-space accounting. C or Rust controls must implement
the same selected contracts. End-to-end latency, not compiler proof count,
chooses whether a remaining runtime validation is material. General concurrent
reclamation, RCU and lock-free containers are outside these sequential/fork-join
trials and receive no coverage claim from them.

## Findings rechecked against merged PR #70

These rows distinguish the merged `8c02e875` restoration baseline from the
subsequent library trial. The restoration alone changed no container algorithm,
contract, specification rule or compiler implementation.

| ID | Exact witness/source | Status at the merged baseline | Next action |
| --- | --- | --- | --- |
| X1-P1 | Baseline `grow_vector_drain` used repeated `remove_at(..., index: 0_u64)`; OP-10 | The later Vector trial replaces quadratic movement with suffix reversal and back consumption in [grow-vector.wf](../../../lib/containers/grow-vector.wf). | The original-order callback contract is preserved. The [comparison](../../experiments/container-representation/vector-library/RESULTS.md) measures the remaining cost against a direct consumer; O(n) is not a minimum-transfer claim. |
| X1-P2 | [unbounded-reserve.wf](../../experiments/container-representation/x1/unbounded-reserve.wf) records the old missing-requirement shape | Resolved in the shipped GrowVector: `const ceiling`, `requires total <= ceiling`, bounded doubling and saturation replace unrestricted growth. MSR-4 now supplies the specified affine-left/L0-right bridge needed by the ordinary caller proof. The deliberately unbounded probe should still reject under OP-9. | Keep the size requirement. A library/application Full outcome may return the offered owner when its selected limit is reached; heap allocation itself has no refusal arm. Do not carry this old finding forward as a compiler or current-library defect. |
| X1-P3 | [linear-ring-publish.wf](../../experiments/container-representation/x1/linear-ring-publish.wf):18; OP-12 and WIN-3 versus the atomic-update paragraphs in [CANDIDATE-X1.md](../access-effects/CANDIDATE-X1.md) and [affine-replacement.md](../../../design/language/ownership/affine-replacement.md) | The active affine/copy restriction remains. A nodrop Ring cannot use this atomic-publication route; the candidate's general linear-assignment refusal also remains. The broader atomic paragraph alone does not establish a selected linear exception. | Obtain an explicit intended-domain ruling before changing OP-12 or its record. In parallel, test ordinary swap/contract and consuming-rebase alternatives without claiming all deque designs impossible. No language widening is part of this restoration. |

The following are specified limits, not bugs to silently fix in #70:

| Exact fragment | Rule | Ordinary workaround and limit |
| --- | --- | --- |
| `count(part: &ring[0_u64..0_u64])` in [ring-range.wf](../../experiments/container-representation/x1/ring-range.wf) | REF-4 | Element visitation, copying into Slots, or a full copy-element Array with two physical spans. Only the last preserves zero-copy ranges, and it requires initialization/filler. |
| `ensures deref(destination).len == deref(entry(destination)).len + deref(entry(source)).len;` in [append-contract.wf](../../experiments/container-representation/x1/append-contract.wf) | FN-9 admits only one datum plus a constant on each relation side | Reread lengths and use ordinary control flow where necessary; an affine postcondition extension is separately proposed work, not assumed here. |
| `struct Record { header: Header; tail: Array<u8>; }` (declaration fragment) | TYPE-9 permits a runtime-capacity shape only directly as Box content | Encoded byte block, or a separate Box for the tail. Neither is an implicitly packed typed trailing member. |
| `set cursor = &deref(cursor).next.Some.value.inner;` carried by a loop (path fragment) | REF-1 static path shape; REF-2 also matters when leaving the Some arm | Pool indexes or ordinary recursive descent. Do not assume wildcard-path or musttail work has already landed. |
| An `ensures` exporting a returned handle's indexed generation/variant relation | FN-9's relation datums and routes exclude that shape | Return a bounded scalar index, then validate/match locally or inside the consuming callback. Measure repeated checks before widening contracts. |

## Library home and evidence after the merge

The owner selected root `lib/` for reusable WF source. The restoration at
`8c02e875` placed the merged GrowVector implementation, byte for byte, at
[`lib/containers/grow-vector.wf`](../../../lib/containers/grow-vector.wf).
Its caller and C allocation observer remain under `tests/programs/containers/`;
the existing corpus test still builds the same source bundle in sequential
and parallel modes, then executes each normally and with the observer. The
test reads the relocated source with a separate, valid logical source name;
filesystem parent components are not source-envelope names. No separate
Makefile, test group, import mechanism or library ABI is restored.

This ownership split makes the implementation available to user programs
without making test support or research models library dependencies. Keep
the other container fixtures as fixtures until they meet a reusable contract:
`priority.wf` is a u64 heap of capacity 16, `ordered.wf` exercises leaf splits,
and the behavior map requires droppable keys and fixes the payload to a Box.
Their useful coverage does not establish the complete generic families.

At that restoration baseline the GrowVector caller covered scalar and owned droppable Box elements,
zero capacity, doubling, ceiling saturation, insertion, removal and drain.
Its release observer checked exactly twelve allocations. That evidence did not establish
must-consume-element construction/cleanup, an order-sensitive drain oracle,
large-element costs, or native parity of the merged implementation. The
retained v0.58 experiment has a different storage and allocation-refusal
contract and must not be used as current performance evidence. The later
Vector trial below supplies the expanded ownership and current cost evidence.

## Recommended implementation and measurement order

1. **Finish the existing reusable Vector first.** Implement O(n) ordered drain,
   then the missing selected operations such as swap-remove and consuming
   truncation. Extend the existing caller with order-sensitive observations,
   copy/drop/nodrop instances and full construction-to-cleanup chains; avoid
   duplicating a native harness. Compare the actual merged implementation with
   a matched C control, separately pricing growth, initialized spare storage,
   large-element transfers and retained-helper overhead. A proof-erased branch
   count is not a substitute for those measurements.
2. **Complete the first slice with Slab and Deque.** Slab trials establish
   vacancy exchange, generation exhaustion, expiry and the selected membership
   contract. Deque trials establish both-end operations, wrap, grow/rebase and
   cleanup; choose explicitly between slot visitation and the copy-element
   physical-span representation. Resolve the complete nodrop growth route
   before declaring an unbounded growable Deque. These are independent trials;
   the unresolved atomic-update domain need not block Slab or Vector work.
3. **Build the keyed and composite slice.** Generic HashMap and PriorityQueue
   must run complete chains, including owned keys/values, rehash and reverse-map
   repair. Compare sparse inline and index/dense layouts at the same identity
   contract. Run the record-index workload to expose the cost and correctness
   of retaining one object through more than one index.
4. **Run the ordered/layout challenge.** Full B-tree operations and scans,
   a Box-linked comparison, compact page records and large-value construction
   provide the consumers for existing contract, traversal and lowering
   proposals. No primitive is selected merely because a prototype was awkward.

After each slice, report expressibility, completed operations and measured cost
separately. A source rejection, unsupported lowering, wrong native result and
unmeasured candidate are four different outcomes. Required library behavior
must not be weakened to obtain a green experiment. Merging PR #70 establishes
the baseline; it does not by itself complete these libraries or establish
their performance ceiling. This branch restores the library home and updates
the evidence and recommendations, without implementing the next slice.

## Vector consumption trial

The first library implementation continues from merged `8c02e875`. Its
question is whether ordinary prefix-window operations can provide ordered
consumption, arbitrary removal and complete ownership cleanup at a competitive
cost, including when the element is large or must be explicitly consumed.

Use the existing `GrowVector<T, ceiling>` and `VectorDrain<T, E>` boundary.
Ordered truncation takes a retained length, requires it not to exceed the
current length, and hands the removed suffix to the member in original order.
Drain is truncation to zero. Both preserve capacity and publish their exit
length. Unordered removal swaps the selected slot with the last slot and
takes the last value; OP-11 admits equal indexes, so removing the last element
needs no special alias branch. Empty-vector destruction consumes its owner
under the ordinary OP-14 empty-window requirement.

The consuming member's `writes(env)` and the vector helper's
`writes(values.storage)` must be disjoint at the call under EFF-5. The member
receives no reference to the vector. Consequently it cannot inspect the
remaining suffix while the helper is rearranging it. Reverse just that suffix,
then take from the back: the member observes the original order, the retained
prefix is unchanged, and no scratch allocation or invalid slot is introduced.
This is a synchronous operation, not a resumable drain value or an interface
that lets a callback inspect partial container contents.

The trial compares this composition with two C controls: the same
reverse-and-take algorithm and a direct ordered consumer over the same
unobservable backing. Both have the same observable callbacks, retained
capacity and allocation policy. Their difference isolates the source
composition's extra element movement instead of charging it to language-call
overhead. An allocation/free/memmove growth control matches the current
lowering; it is not evidence against a separately measured realloc policy.

Before measuring, use these discriminators:

- Ordered drain/truncation must have O(removed length) element movement,
  preserve the exact callback sequence and retained prefix, and allocate
  nothing. Swap-remove must have O(1) element movement and return the selected
  owner. Test empty, singleton, same-index, boundary and large-length cases.
- Extend the existing formal corpus bundle instead of adding another native
  test framework. Exercise copy, droppable owning and nodrop owning elements
  through construction, growth, insertion, removal, consuming truncation,
  drain, reuse and final release. Check every allocated identity is released
  once and every nodrop payload is explicitly consumed; no synthetic OOM
  outcome belongs to the global-heap contract.
- Measure the actual library, not a benchmark-only implementation, for scalar
  and large owned records, short and long windows, ordinary optimization and
  retained helpers. Compare complete outputs and allocation bytes before
  timing; interleave implementations and retain raw samples. Separate
  construction/growth from a preallocated consumption loop where possible.
- Count optimized transfers and retained calls to explain a gap. The
  algorithm-matched control distinguishes lowering cost; the direct control
  distinguishes the ordinary composition's cost. If a material gap remains,
  record its concrete operation and size domain before choosing another
  representation or proposing language support. O(n) alone is not a parity
  claim, and no percentage threshold is invented for all workloads.

The proposed consumption decision remains in
[`design/amendments/vector-consumption.md`](../../../design/amendments/vector-consumption.md)
while implementation and measurement proceed; it has not changed the live tree.

The trial now implements the selected operations, including explicit cleanup
of a nodrop element vector. The formal bundle observes original callback order,
retained contents and capacity, reuse and 25 exact-once allocation releases in
sequential and parallel lowering. The current
[cost comparison](../../experiments/container-representation/vector-library/RESULTS.md)
uses the actual library, matched reverse C and direct C, and scalar/256-byte
elements. It isolates an unnecessary Slots wrap computation and retains the
measured cost of suffix reversal after that repair. The source form is an O(n)
baseline, not native parity across workloads. The residual performance question
and a separate Box-measure placement defect remain in `docs/todo.md`.

### Vector source obligations

An ordinary loop's header hypothesis does not survive its exit [ENT-5].
Consequently, this source does not establish its declared postcondition:

```whitefoot
loop @truncate (
  invariant prefix: deref(values).storage.inner.len >= retained
) {
  if deref(values).storage.inner.len <= retained {
    break @truncate;
  }
  let value = take_back(window: &deref(values).storage.inner);
  VectorDrain::accept(env: env, value: move value);
}
return unit;
```

FN-9 cannot prove the exit length equal to `retained`. Publishing
`invariant exhausted: deref(values).storage.inner.len == retained;` immediately
before the break uses INV-1's existing local theorem route, which ENT-5 retains
across that edge. The theorem is proved from the header and the exit guard;
it introduces no runtime branch. This is a specified proof-writing boundary,
not a compiler bug or a relaxed postcondition.

The initial compiler rejected the following admitted consume under PROV-6:

```whitefoot
fn grow_vector_free_empty<T, const ceiling: u64>(values: own GrowVector<T, ceiling>) -> result: own unit pure contract {
  requires values.storage.inner.len <= 0_u64;
} {
  free_empty(window: move values.storage);
  return unit;
}
```

The only field moves into the operation, leaving no residual. WIN-3 consumes
the whole wrapper; PROV-6 refuses a linear *remaining* part, not the moved
field. The checker instead tested the complete original type. The repair
judges the residual inventory and keeps unbounded generic and fieldless
nodrop residuals in that inventory even when their release emits no action.
Positive and negative compiler tests cover that distinction; the library's
nodrop chain exercises the complete source-to-native cleanup path.

Taking the wrapper apart first is not a substitute for that repair:

```whitefoot
let GrowVector(storage: storage) = move values;
free_empty(window: move storage);
```

The initial compiler loses `values.storage.inner.len == 0` at that naming
event and rejects the second line under OP-14. Its placement walk stops at
Box content. This is recorded in `docs/todo.md` as a separate implementation
gap against ordinary field-based measure placement, not as evidence that the
language cannot represent an empty owning vector. The direct consume above
needs no workaround branch and no new proof mechanism.

FN-8's Signed Goal affine route has a separate spelling boundary [ENT-6]:

```whitefoot
requires deref(values).storage.inner.len == 0_u64;
```

With only a proved loop-header theorem supplying emptiness, this equality
requirement is unproved even when a local invariant can restate that same
equality. ENT-6's affine Signed Goal leaves are the four order comparisons,
not equality. The equivalent `requires deref(values).storage.inner.len <=
0_u64;` succeeds because the length is an unsigned measure. The reuse work
helper and empty-owner consumer use that spelling, without a runtime test or
weaker domain. FN-9's numeric postcondition route still permits equality.
