# Containers over the x1 language

The reassessment and first Vector trial below use the merged PR #70 baseline,
`36be8784e84a26d34bc24668babd789e0f4c96fb`, kernel v0.60, and their stated
subsequent implementation revisions. The Box-placement and consumption
follow-up at the end starts from kernel v0.61. The initial study
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

The family sketches below started as recommendations for implementation trials,
not adopted library interfaces. The later Vector and v0.63 sections identify
the executable libraries, their evidence and the selected boundaries. Existing temporary-reference,
global-heap, no-hole and no-stored-reference choices remain premises. Their grounds are in
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
| Deque | `Box<Ring<T>>`, fresh ring plus a counted take/place transfer for exact rebase contracts; fixed Ring for bounded queues | Push/pop both ends, wrap, indexed access, grow/rebase, consume, release | Bounds in logical coordinates; front operations invalidate old slot references; new backing invalidates every old path; source emptied before releasing it | O(1) endpoints and logical access; O(n) growth; one visit per drained element. Generic two-span consumption remains a separate unavailable interface, discussed below. |
| Slab | `Box<Slots<SlabCell<T>>>`, with `Slots<T,1>` occupancy, a free-list head and generation handles; compare a compact tagged native cell | Insert/handle, validate/get, remove, expiry, reuse, exhaustion/limit outcome, final consumption | Return every removed T; never wrap a generation into an old handle's generation; distinguish a slot ID from a physical address | O(1) free-list operations and validation; no payload movement on ordinary reuse. The bounded library does not grow its backing; its extra occupancy word is a measured cost. |
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
is not. The generic library instead uses the explicit consuming rebase in the
v0.63 trial below; this reference-based exchange is not its growth contract.

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

REF-4 refuses *all* Ring range references. Therefore a function
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

The original enum representation sketch was:

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
not new brands or memory-safety authority. The v0.63 insertion trial below
exposes the missing variant fact after the reverse exchange; the implemented
Slab therefore uses a one-element window in each cell. The sketch alone is
not a complete nodrop implementation.

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

#### Generic owning-map trial after the Ring comparison

The reusable-map trial starts from `e6349b80`, kernel v0.67. Its contract is a
map over arbitrary owned K and V, including must-consume values, with hash and
equality supplied through the existing `interface`/`binding` mechanism. Neither
key nor value requires a separate Box. Collision insertion, replacement that
returns the previous pair, lookup past deletion, removal, slot reuse, growth
and rehash, visitation, and explicit final consumption belong to the same
operation chain. A caller-selected capacity ceiling may return the offered
pair; allocation itself has no source-visible refusal under STOR-8. Returned
references, persistent iterators and stable payload addresses are not part of
this contract.

Borrowed value editing belongs beside read-only lookup: a callback may read
the stored key, write the selected value and its disjoint environment, and
return an owned result. Without that form, a counter or wide record update
would require replacing or removing and reinserting the complete value merely
because references cannot escape. Ordinary effect rows already express this
boundary. Validate value-only edits, missing keys, owning children, and unchanged
map extents; no key mutation or retained reference is implied. Compare a small
scalar and wide-record edit trace with the same native callback contract before
claiming that it removes a measured cost.

Before using measurements to select a representation, hold the operation
trace, supplied behaviors, occupancy, capacity policy and ownership outcomes
fixed. Compare ordinary enum slots with dense entries plus sparse indexes;
the Slab's one-element window is an available occupancy encoding, not a
preselected map layout. Count complete backing and result layouts, sparse and
dense reserved capacity, allocations, probe work, relocation during growth,
and helper-boundary transfers. Use both ordinary optimization and retained
helpers, scalar and wide inline values, and a native C control for the same
contract. A layout improvement must survive inclusion of reverse-index repair
and dense-growth costs; reducing table bytes alone does not select it.

The first matched trace uses exact runtime capacities, no cached hash, and
rehashes each live key through the supplied behavior. Reserve names its target
capacity. An insertion that exhausts its bounded probe grows geometrically,
saturating at the caller's ceiling; zero capacity grows to one. Replacement
still succeeds at that ceiling. Steady-state comparisons use half-full and
seven-eighths-full tables, with growth measured separately. This simple shared
policy isolates the representation comparison; it does not select an optimal
production load factor. The native sparse floor may rebuild directly into a
new backing and initialize only empty tags. It is not charged WF's temporary
planning arrays, whole-slot swaps, or zeroed inactive payload bytes.

The correctness discriminator includes hostile equality, collision-heavy and
full-table traces, zero capacity, replacement at the capacity ceiling, and
cleanup after every owner-returning outcome. No equality law justifies a
bound or permits a value to disappear. Rehash must retain all entries without
asking equality to deduplicate them. If an ordinary formulation fails, retain
its exact source and separate a specified limit from a compiler defect before
changing either interface or representation.

The sparse rehash candidate plans destinations in copyable metadata, then
grows the existing owning backing and permutes complete slots through `swap`.
This avoids discarding a temporary enum on an unproved vacancy assumption.
The dense candidate rebuilds only sparse metadata before publication, leaving
its separate dense owners in place. Both routes need checked complete source;
neither is selected on an assertion that the other is inexpressible. Temporary
planning allocations and payload relocation are part of the sparse candidate's
cost, just as reverse-index maintenance and the dependent lookup are part of
the dense candidate's cost.

A third admitted ordinary formulation gives each sparse cell
`Slots<Pair<K,V>, 1>` and a deleted flag. After
allocating an empty destination with at least the old materialized capacity,
swap the backing, drain each old cell, and append its single live pair into
an empty destination window. The ordinary append postcondition establishes
that the local source is empty, so cleanup needs no assumed enum refinement.
Hash each owner once; equality is unnecessary. At owner j, at most j-1 of the
M destinations are filled, so a complete cyclic scan finds a vacancy when
j <= old capacity <= M. The complete source and its scalar/owning-child
caller execute in default, sequential and parallel CLI configurations. The
allocation observer checks all eighteen identities, each released exactly
once; a zero-capacity rebuild allocates nothing. The vacancy argument is
not a published numeric contract or a general compiler proof of termination.

That route avoids permutation plans and the bulk old-backing grow copy, but
costs an extra word per cell: 32 rather than 24 bytes for the scalar pair,
280 rather than 272 for the wide pair. It also retains both payload backings
during migration. Ignoring fixed headers, growing C to M has peak bytes
`(C+M)*(B+8)`, versus the enum route's `(C+M)*B+8*M`: about 8*C more bytes.
Same-capacity rebuilding needs a second full payload backing rather than
only the enum route's metadata. Empty-cell construction and local cell/append
transfers remain costs to inspect. Its comparison criterion is material
copying/permutation cost or a split between sparse lookup/edit and dense
growth. Compare both growth and same-capacity rebuild, scalar and wide values,
with retained helpers. Do not reject it as inexpressible or infer a smaller
peak from fewer allocations.

The first matched timings trigger that additional comparison. At capacity
4096 and seven-eighths occupancy, the first cohort's wide growth trace takes
about 0.70 of the sparse time for dense storage under normal optimization,
and 0.74 with retained helpers, while ordinary sparse lookup and edit win
elsewhere. The same sparse growth is about 1.78 times its direct native
control but 1.21/1.04 times its planned-algorithm C control. The
[comparison record](../../experiments/container-representation/map-library/RESULTS.md)
owns the complete paired samples and both cohorts; these particular results
motivate a discriminator, not a layout selection. Extend the ordinary
one-slot route only far enough to compare its admitted owning operation chain
and matched growth/rehash traces. Count its larger cells and double-backing
peak, as well as actual transfer work. Select it only if the observed benefit
justifies those costs for the exposed contract; do not extend the full timing
matrix merely because a third representation exists.

The bounded discriminator uses only scalar/wide growth and same-capacity
rehash at capacity 4096 with 3584 entries, under the original seeds, complete
traces, optimization modes and reversed cohorts. Draining the old window
visits buckets in descending order, whereas the original direct C floor
visits ascending buckets. Retain that floor, then separate descending enum
storage, descending one-slot storage, the source's cell-take/append algorithm
in C, and the WF source. Keep the original sparse/dense implementations in
the same run as references. This prevents a changed insertion order or
source migration algorithm from being attributed solely to layout.

The extra per-bucket word is not yet established as necessary for direct
migration. An additional source discriminator keeps the enum buckets and
uses one local `Slots<Pair<K,V>,1>` as the pending-owner carrier: pop and
match the old slot, stage its pair, probe by occupancy, exchange at the
vacancy, and restage any complete pair the exchange returns. An empty
carrier ends that owner's migration. The earlier PROV-6 rejection concerned
an unconsumed displaced enum, not this exhaustive ownership protocol.
That formulation admits and executes against the unchanged scalar and
owning-child callers under sequential lowering. Their exact allocation
ledgers fall from 29/14 to 15/12, each identity released once: six initial
backings, five growths and four same-capacity rebuilds for the scalar chain;
ten children and two backings for the owning chain. The local staging window
allocates nothing. It therefore joins the same four rebuild cells before
charging direct migration the larger bucket layout. Each attempt hashes
the current staged key and never uses equality to deduplicate. Transfer
costs and paired timings are recorded in the same comparison: scalar growth
improves, while wide growth retains substantial staging and result work.
Admission and the conceptual vacancy argument alone select neither
implementation nor representation.

The optimized migration path exposes one further source choice before
promotion: it calls the general exchange helper even though migration needs
only to consume the displaced enum, not to construct a public insertion
result. A bounded alternative puts the same swap and exhaustive enum match
inside rebuild, restoring any displaced pair to the same pending window.
It may remove a wide intermediate result and let ordinary optimization use
the just-observed vacancy; no variant is discarded on that observation.
Select this form only if the unchanged owning and hostile-behavior callers
pass, its emitted migration removes the result construction without adding
storage, and the same four rebuild cells support the cost improvement.
The public insertion interface and helper-retention policy stay fixed.

The direct route exposes a separate contract boundary. After extending a
fresh `previous` backing to `capacity`, it publishes that owner with
`swap(first: &deref(map).cells, second: &previous)`. PRE-1 declares only
`writes(first), writes(second)` for swap. OP-11 exchanges both values and
keeps their ownership live, but MSR-3 does not define swap as a measure-fact
placement, and CALL-6 has no declared postcondition to publish. Consequently
the previous `previous.inner.len == capacity` proof does not establish
`deref(map).cells.inner.len == capacity` afterward. Re-reading the actual
extent permits bounded migration; it does not establish an entry/exit
monotonicity promise. This is the current proof contract, not a lost owner or
an observed lowering defect. Keep that limitation in the existing contract
backlog rather than add a runtime branch solely to satisfy an ensures.

Result layout is a separate source choice. The initial common result has three
variants, `Inserted`, `Replaced(previous: Pair<K,V>)`, and
`Full(offered: Pair<K,V>)`. Current product layout reserves both Pair regions.
An ordinary alternative is `Inserted | Returned(reason: ReturnReason,
pair: Pair<K,V>)`, with tag-only `Replaced / Full` reasons. It preserves every
ownership outcome while sharing one payload region, at the cost of a nested
match. For an eight-byte-aligned Pair of 16 or 264 bytes, the current layout
calculation predicts 40 or 536 bytes for the original and 24 or 272 bytes for
the shared form. A zero-byte Pair instead grows from four to eight bytes; this
is not an unconditional layout win. These are layout deductions, not timings.
Compare the same replacement and refusal/retry consumers under both source
forms before choosing either the public result or a compiler optimization.
Keep initialization, construction and consuming projection transfers separate
from the result's reserved width. The native control already has one Pair
region, so its matching semantic outcome alone does not establish ABI parity
with the initial WF result.

The first reusable library uses the compact returned-pair result and direct
enum migration through the shared exchange helper. Its complete
[public caller](../../../tests/programs/containers/hash-map-program.wf)
uses no representation fields and executes collision, replacement after a
tombstone, removal, reuse, bounded growth, rehash, borrowed editing and final
consumption. Separate key/value identities distinguish returning the old pair
from merely returning an equivalent offered key. Must-consume children,
zero-sized pairs, inconsistent equality and a fresh owned callback result
exercise the same source. The allocator observer accounts for seventeen
backings, ten child Boxes and the callback's returned Box, each released once.
The formal program test bundles the actual library; it has no research input.

The source selection has two distinct grounds. Sharing the public result's
payload reduces the measured wide replacement and churn cost as well as its
reserved width. Direct enum migration removes the planning arrays and avoids
the extra word in every one-slot bucket. The selected helper source's wide
growth trace is about 0.93 of the planned sparse source, but remains
1.64--1.69 times the direct C floor with the same descending migration
direction and 1.25--1.34 times the dense WF source. Same-capacity rehash also
retains two complete backings. These are reasons to preserve dense storage and
lowering improvements as measured opportunities, not to call this a universal
fastest map. The C floor, source-shaped C, raw paired samples and peak-byte
accounting remain separate in the
[comparison](../../experiments/container-representation/map-library/RESULTS.md#constant-interface-comparison-and-selected-helper-body).
No application-frequency distribution is inferred from the test matrix.

The inline-exchange criterion above was **not fully met**. Holding the compact
public interface fixed removes the private result's stack slot, call and
Inserted clearing, but reversed ABBA source controls do not establish a stable
independent timing benefit after the unchanged C controls' variation is
accounted for. The library therefore retains the shared exchange helper. The
inline source and its measurements remain a replayable rejected alternative,
not the delivered library or evidence that the removed clear alone costs the
observed timing difference. No compiler implementation, source-language rule
or conformance evidence is changed. The pending storage amendment records the
two proposed library choices; the proof-contract and remaining performance limits stay in
[`docs/todo.md`](../../../docs/todo.md).

The following complete controls retain the trial's proof boundaries under
kernel v0.67 and the compiler identified in the comparison's
[build identities](../../experiments/container-representation/map-library/RESULTS.md#sample-and-build-identities).
Each standalone program and each listed variant was checked by guarded LLVM
emission with that executable; apply variants independently to their stated
baseline. The reported rejection is the first diagnostic, not a claim that
every alternative formulation fails. For a standalone block saved as
`control.wf`, the invocation is:

```sh
perl .github/run-check.pl map-proof-control compiler/target/gate/whitefootc --emit-llvm control.wf -o control.ll
```

**Contract formation and publication.** This complete baseline accepts:

```wf
struct Counter {
  length: u64;
}

fn scalar(map: &Counter) -> result: own unit writes(map) {
  set deref(map) = Counter(length: deref(map).length);
  return unit;
}

fn append_proof(values: &Slots<u64, 1>, value: own u64) -> outcome: own Result<unit, unit> writes(values) contract {
  requires deref(values).len < deref(values).cap;
  ensures deref(values).len == deref(entry(values)).len + 1_u64;
} {
  place_back(window: values, value: value);
  return Ok<unit, unit>(value: unit);
}

fn replace(cells: &Box<Slots<u64>>) -> result: own unit writes(cells) {
  let previous = box_slots_new::<u64>(capacity: 1_u64);
  place_back(window: &previous.inner, value: 7_u64);
  invariant prepared: previous.inner.len == 1_u64;
  swap(first: cells, second: &previous);
  return unit;
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
```

The exact variants are:

1. Replace `scalar`'s first line with these three lines:

   ```wf
   fn scalar(map: &Counter) -> result: own unit writes(map) contract {
     ensures deref(map).length >= deref(entry(map)).length;
   } {
   ```

   The first diagnostic is FN-9 `InvalidPostconditionRelation` at that clause.
   FN-9 gives exit-state denotation to a written reference's measures, and
   MSR-3's `entry` adds no scalar snapshot family. Ordinary counter reads and
   writes remain available.
2. In `append_proof`, replace its ensures with
   `ensures when Ok(value: success): deref(values).len == deref(entry(values)).len + 1_u64;`.
   The first diagnostic is FN-9 `InvalidPostconditionSelector` at `Ok`.
   FN-9's routed form requires an integer success payload. The accepted
   baseline instead publishes an unrouted exit-measure relation from a
   `Result<unit, unit>` function; such relations are also legal on a plain
   unit-returning function. Unit results do not prohibit ensures generally.
3. Add the enum below, change `append_proof`'s result type from
   `Result<unit, unit>` to `DenseProofPut`, replace its ensures with
   `ensures when DenseProofReplaced(previous: old_value): deref(values).len == deref(entry(values)).len + 1_u64;`,
   and replace its return with `return DenseProofReplaced(previous: value);`.
   The first diagnostic is FN-9 `InvalidPostconditionSelector` at
   `DenseProofReplaced`. FN-9 admits only the prelude `Result.Ok` route, so a
   custom `Inserted / Replaced / Full` result cannot publish a relation for
   each outcome. An unconditional insertion interval alone does not prove
   that the Full arm preserves length before a retry.

   ```wf
   enum DenseProofPut {
     DenseProofInserted();
     DenseProofReplaced(previous: u64);
   }
   ```

4. Replace `replace`'s first line with these three lines:

   ```wf
   fn replace(cells: &Box<Slots<u64>>) -> result: own unit writes(cells) contract {
     ensures deref(cells).inner.len == 1_u64;
   } {
   ```

   The first diagnostic is FN-9 `UndischargedPostcondition` at its return,
   with relation `deref(cells).inner.len = 1` and disposition `Unproved`.
   PRE-1 declares swap's writes without an ensures; ENT-5 kills the supported
   facts, MSR-3 has no swap placement, and CALL-6 has no relation to publish.
   Starting from this variant, replacing only the swap statement with
   `set deref(cells) = move previous;` accepts through MSR-3's ordinary
   placement. That control discards the old destination owner; it does not
   implement the map's exchange-and-migrate algorithm.

**Equality at a call after a counted loop.** This complete baseline accepts:

```wf
struct Holder {
  cells: Box<Slots<u64>>;
}

fn preserve(cells: &Box<Slots<u64>>) -> result: own unit writes(cells) contract {
  requires deref(cells).inner.cap <= 1_u64;
  ensures deref(cells).inner.len == deref(entry(cells)).inner.len;
  ensures deref(cells).inner.cap == deref(entry(cells)).inner.cap;
} {
  let capacity = deref(cells).inner.cap;
  grow(cell: cells, capacity: capacity);
  return unit;
}

fn apply(cells: &Box<Slots<u64>>, plan: &Box<Array<u64>>) -> result: own unit writes(cells), writes(plan) contract {
  requires deref(cells).inner.cap <= 1_u64;
  requires deref(plan).inner.len <= deref(cells).inner.len;
  requires deref(plan).inner.len >= deref(cells).inner.len;
} {
  grow(cell: cells, capacity: 1_u64);
  if deref(plan).inner.len > 0_u64 {
    set deref(plan).inner[0_u64] = 0_u64;
  }
  return unit;
}

fn nested(holder: &Holder, capacity: own u64) -> result: own unit writes(holder.cells) contract {
  requires deref(holder).cells.inner.cap <= 1_u64;
  requires deref(holder).cells.inner.len == capacity;
  requires capacity <= 1_u64;
} {
  let plan = box_array_filled::<u64>(count: capacity, value: 0_u64);
  for (
    index in 0_u64..capacity,
    invariant retained: deref(holder).cells.inner.len == capacity,
    invariant planned: plan.inner.len == capacity,
    invariant bounded: deref(holder).cells.inner.cap <= 1_u64
  ) {
    preserve(cells: &deref(holder).cells);
  }
  invariant equal_extent: plan.inner.len == deref(holder).cells.inner.len;
  apply(cells: &deref(holder).cells, plan: &plan);
  return unit;
}

fn main() -> status: own ExitStatus pure {
  let cells = box_slots_new::<u64>(capacity: 1_u64);
  place_back(window: &cells.inner, value: 7_u64);
  let holder = Holder(cells: move cells);
  if holder.cells.inner.cap <= 1_u64 {
    if holder.cells.inner.len == 1_u64 {
      nested(holder: &holder, capacity: 1_u64);
    }
  }
  return exit_status(code: 0_u8);
}
```

Replace only `apply`'s paired length requirements with
`requires deref(plan).inner.len == deref(cells).inner.len;`. The first
diagnostic is FN-8 `UndischargedCallRequirement` at the call to `apply`, with
instantiated goal `plan.inner.len == deref(holder).cells.inner.len` and
disposition `Unproved`. Removing only that call from this rejected variant
accepts, including the immediately preceding `equal_extent` invariant.
INV-1 splits equality into two affine inequalities; ENT-6's affine
normalization of signed FN-8 goals admits ordering leaves only. Ordinary
L0 equality remains available, but these affine premises do not take that
route. The paired requirements preserve exact equality without runtime work.

**Conditional loop preservation.** This complete source rejects:

```wf
fn preserve(values: &Box<Slots<u64>>) -> result: own unit writes(values) contract {
  requires deref(values).inner.cap <= 1_u64;
  ensures deref(values).inner.len == deref(entry(values)).inner.len;
  ensures deref(values).inner.cap == deref(entry(values)).inner.cap;
} {
  let capacity = deref(values).inner.cap;
  grow(cell: values, capacity: capacity);
  return unit;
}

fn conditional(values: &Box<Slots<u64>>, enabled: own Bool) -> result: own unit writes(values) contract {
  requires deref(values).inner.cap <= 1_u64;
  ensures deref(values).inner.len == deref(entry(values)).inner.len;
} {
  let count = deref(values).inner.len;
  for (
    index in 0_u64..count,
    invariant retained: deref(values).inner.len == count,
    invariant bounded: deref(values).inner.cap <= 1_u64
  ) {
    if enabled {
      preserve(values: values);
    }
  }
  return unit;
}

fn main() -> status: own ExitStatus pure {
  let values = box_slots_new::<u64>(capacity: 1_u64);
  place_back(window: &values.inner, value: 7_u64);
  let enabled = True();
  conditional(values: &values, enabled: enabled);
  if values.inner.len != 1_u64 {
    return exit_status(code: 1_u8);
  }
  return exit_status(code: 0_u8);
}
```

The first diagnostic is INV-1 `UndischargedLoopInvariant`, name `retained`,
obligation `Backedge`, with diagnostic relation `deref(values).len <= count`.
Adding `invariant restored: deref(values).inner.len == count;` immediately
after `preserve` inside the true arm leaves that same first rejection.
Replacing the entire loop body with the following accepts:

```wf
let before = deref(values).inner.len;
let before_capacity = deref(values).inner.cap;
if enabled {
  preserve(values: values);
}
invariant rejoined: deref(values).inner.len == count;
invariant rejoined_capacity: deref(values).inner.cap <= 1_u64;
```

Removing only the `before_capacity` binding and `rejoined_capacity` invariant
from this accepted body moves the first rejection to INV-1's `bounded`
backedge obligation, relation `deref(values).cap <= 1_u64`. Replacing the
original loop body with the unconditional `preserve(values: values);` also
accepts. ENT-5 joins closed L0 relations over every reaching branch; ENT-6
retains only canonically identical affine inequalities over their immutable
images. Capturing both current measures supplies connections across this
small conditional. The rejected form does not establish a general inability
to preserve measures through conditionals.

The full sparse candidate remains a separate unresolved case. Use
[sparse-map.wf](https://github.com/mbbill/Whitefoot/blob/c206655d898ea32de9995bca2b19444c92c1f973/research/experiments/container-representation/map-library/sparse-map.wf)
and its
[sparse-check.wf driver](https://github.com/mbbill/Whitefoot/blob/c206655d898ea32de9995bca2b19444c92c1f973/research/experiments/container-representation/map-library/sparse-check.wf)
from `c206655d898ea32de9995bca2b19444c92c1f973`, bundled in that order for
`--emit-llvm`. In `sparse_map_rebuild`, replace only this loop-body statement:

```wf
sparse_map_clear_deleted::<K, V>(cells: &deref(map).cells, index: index);
```

with:

```wf
let deleted = sparse_map_is_deleted::<K, V>(slot: &deref(map).cells.inner[index]);
let current_extent = deref(map).cells.inner.len;
if deleted {
  sparse_map_canonicalize::<K, V>(cells: &deref(map).cells, index: index);
}
invariant unchanged_extent: deref(map).cells.inner.len == current_extent;
invariant rejoined_extent: deref(map).cells.inner.len == capacity;
```

Keep the existing helper and every other line. The first diagnostic is INV-1
`UndischargedLoopInvariant`, name `rebuilt_extent`, obligation `Backedge`,
with diagnostic relation `deref(map).cells.len <= capacity`. This reproduces
the full candidate's failure despite its written post-join bridges. Whether
that remaining refusal is required by the fixed proof rules or is a compiler
defect has not been established. The admitted candidate retains the
length-preserving helper around the conditional tombstone operation; that
helper does not relocate live payloads, and its call structure remains part
of the matched source-cost comparison. Neither this failed formulation nor
the admitted small control settles the full candidate's normative diagnosis.

The maintained [TODO](../../../docs/todo.md) remains the owner of unresolved
issues. This trial reopens must-consume sparse slot state, aggregate result
transfer costs, and any contract boundary its actual source crosses. Ring
two-span access, automatic reference-based rebase, ordered-drain movement,
header-plus-tail storage, retained membership, fixed-resource execution and
generic checking cost retain their own evidence and reopening conditions.
A passing map does not resolve those independent questions. New evidence
updates the existing entry rather than creating a second backlog here.

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
their performance ceiling. The restoration at `8c02e875` restored the library
home and updated the evidence and recommendations. The subsequent Vector
consumption trial below implements the first library slice.

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

The original consumption decision is recorded in
[`design/language/data-model/vector-consumption.md`](../../../design/language/data-model/vector-consumption.md).
The follow-up below proposes replacing its separate reversal pass.

The initial trial implemented the selected operations, including explicit cleanup
of a nodrop element vector. The formal bundle observes original callback order,
retained contents and capacity, reuse and 25 exact-once allocation releases in
sequential and parallel lowering. The initial
[cost comparison](../../experiments/container-representation/vector-library/RESULTS.md)
used the actual library, matched reverse C and direct C, and scalar/256-byte
elements. It isolated an unnecessary Slots wrap computation and retained the
measured cost of suffix reversal after that repair. That source form established
an O(n) baseline, not native parity across workloads. Both its residual
performance question and the separate Box-measure placement defect motivated
the follow-up below.

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

The approved ENT-2/MSR-3 clarification makes the owned descendants explicit
at an ordinary naming event such as taking this wrapper apart:

```whitefoot
let GrowVector(storage: storage) = move values;
free_empty(window: move storage);
```

The `efe41016` compiler loses `values.storage.inner.len == 0` at that naming
event and rejects the second line under OP-14 because its placement walk
stops at Box content. The follow-up carries the established emptiness fact
through both forms. The
[`descriptor_invalidation` regressions](../../../compiler/src/semantic/tests/descriptor_invalidation.rs)
cover this exact generic witness, projected and recursive Box moves,
constructor placement into Box content and elements, and overlapping writes
that must still kill an old fact. Completion review found that v0.62's
placement table did not explicitly cover measured descendants of an unmeasured
owner and that its datum identity omitted their relative projection. The
owner-approved v0.63 amendment states that boundary and identity; the repair
implements it using existing measure datums. This does not require a new
representation of an empty owning vector.

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

## Box placement and Vector consumption follow-up

This comparison starts from `efe41016d10379325ed4513d0ac7457ec7f24c5b`,
kernel v0.61, including the Vector trial and the wildcard-reference amendment.
Earlier observations retain their recorded baselines; the old absence of
wildcard traversal is not a premise of this follow-up. The two remaining
questions are ordinary Box-content measure transport and the extra transfers
required by ordered Vector consumption.
The integrated measurement revision also includes main `f3cf41d4`, kernel
v0.62. Paired source measurements use one rebuilt integrated compiler; earlier
v0.61 compiler comparisons retain their own identities and conditions.
Following completion review, the owner approved the v0.63 ENT-2/MSR-3
clarification: existing within-body placements carry current facts through
exact owned fields, enum payloads and Box content, with the relative descendant
projection included in datum identity. Existing kills, cross-function contract
boundaries and the exclusion of implicit window slots remain. The amendment
adds no runtime check and does not change the measured lowering or library
source. The dated measurements remain v0.62 evidence, not a new v0.63 run.

The Box witness above is a naming event, not a new relation theorem. The
selected repair transfers available measure facts through the owned descendants
now explicit in MSR-3 and preserves ENT-5's invalidation after an overlapping write,
replacement or call. Check both directions: an admitted consume becomes
provable after rebinding, and an obsolete pre-mutation measure cannot authorize
a later operation. Recursive nominal types must not cause infinite path
enumeration or an arbitrary depth limit. The v0.61 distinction between a
possible descendant cover and an exact captured target still applies.

For consumption, reuse the actual library and the existing reverse-C and
direct-C controls. Keep callback order, retained prefix and capacity, nodrop
ownership, allocation policy and helper-retention settings fixed. A candidate
that introduces scratch allocation, omits consuming callbacks or weakens their
allowed effects does not meet this operation contract. Distinguish an ordinary
algorithm or representation change from a general lowering repair and from an
operation the current source rules cannot express.

Before selecting a candidate, compare it with both the unchanged WF baseline
and the native controls in paired runs. Count the source-required element
transfers and inspect optimized IR with helpers retained; then measure the
existing scalar/large-record and short/long-window cases with ordinary and
retained helpers. The candidate must remove or demonstrably reduce the
identified consumption cost, with reproducible improvement in the affected
large-record path and an explicit account of changes to the other operations.
A new attribution alone is not an improvement result. Any native-parity claim
must be supported by the resulting comparisons, including measurement
variation, rather than inferred from O(n) complexity or fewer IR instructions.

The placement repair uses the existing measure datums and invalidation path.
Owned fields, payloads and Box content share the structural inventory. When
a nominal recurs, the structural walk stops and exact already-interned source
measure paths supply deeper descendants after replay against the operand's
type. Earlier numeric evidence already requires those finite terms; even a
capacity- or head-only read registers the sibling length term. Unmentioned
descendants need only their ordinary standing type facts. The inventory does
not retain old facts or convert an alias cover into an exact owner: a datum
is equated to its source only in the current fact state. Each placement
finishes collecting paths before minting datums, and the entailment walk
visits each static statement once, including loop bodies. Acyclic operands
skip the supplementary scan.

Unbounded type unfolding cannot terminate on a recursive nominal; merely
cutting the cycle loses known facts at deeper written paths, and a fixed
depth cutoff has no language ground. Reusing existing exact terms avoids a
second path-analysis pass. The approved compiler-tree addition records this
choice. The focused `descriptor_invalidation` group passes all 17 cases,
including distinct measures beyond two recursive nominal cycles,
capacity/head-only evidence, an overwritten recursive descendant and a cursor
whose possible-descendant fact cannot establish the owner's measure. The
original generic witness fails under `efe41016` and passes after the repair.
The recursive cases also exposed a constructor-placement omission at
`set Box.inner = ...`; using the existing exact destination path repairs that
naming event without changing commit order or invalidation. These focused
checks distinguish the placement repair; the canonical gate also covers its
other semantic, lowering and native-program consumers. The normative
[`owned-descendant` positive](../../../tests/conformance/cases/msr3-pos-owned-descendant-measures.wf)
carries two distinct lengths through fields, Box content, owner rebinding,
an enum payload and destructuring. Its
[`replacement` negative](../../../tests/conformance/cases/msr3-neg-replaced-owned-descendant-measure.wf)
requires an overlapping replacement to invalidate the old length.

The consumption candidate takes a rear element into an owned local before
exchanging it with the next suffix slot and calling the consumer. At offset
`k < floor(removed / 2)`, the post-take length still exceeds `retained + k`;
the source writes that bound as an ordinary finite certificate. The reversed
remainder is then consumed from the back. Only removed elements move, so
almost complete retention does not turn truncation into a prefix walk. The
interface, permitted callback effects, retained contents and capacity are
unchanged. This still relocates rear elements and is not a minimum-transfer
algorithm.

Two target choices address the demonstrated extra temporary copies. A complete
take captures the old physical slot, updates the window descriptor, then
transfers the element. Its header and element bytes are disjoint, and no call,
release or source observation occurs between those steps. Zero-size elements
touch no bytes. Separately, unrelated storage groups become independent entry
stack allocations only after complete target qualification establishes positive
sizes, one common natural alignment, no requested-alignment discrepancy and
no inter-group or tail padding. Every eligible ordering has the same complete
extent. Mixed alignment, padding and zero-size roots retain the qualified
frame struct; complete parents, storage interference, result destinations,
probing and parallel lifetimes are unchanged. The old canonical offsets
remain qualification accounting, not physical offsets between independent
objects.

The [transfer experiment](../../experiments/container-representation/vector-library/RESULTS.md)
records the alternatives and isolating evidence. Early capture annotations or
independent slots alone did not reduce the candidate's four transfers per
first-half iteration; the descriptor order and independent slots together
reduced them to two after local Clang 21 optimization. Apple Clang 15 retains
an additional immutable argument snapshot in the straight-line regression,
so this is not an optimizer-independent copy-count guarantee. The regression
checks the compiler-owned allocation and descriptor-order properties and the
native result; optimized transfer counts remain toolchain-specific evidence.
Forwarding a consumed mutable local into an ordinary call is a separate
opportunity: it needs a liveness and interference argument across all arguments
and result/input reuse, and the existing result-coalescing path does not cover
a consumer returning unit. That opportunity is retained in the maintained
TODO rather than broadening the present changes without that argument.
The unchanged v0.61 compiler emits one aggregate frame for the regression's
distinct roots and copies the taken element before updating the descriptor;
each new raw-IR assertion therefore distinguishes its corresponding lowering
change without depending on a downstream optimizer.
Broad ABI promises and
unrestricted frame splitting were therefore not adopted. The general mixed-
alignment case needs a separate complete-frame argument if a concrete workload
later demonstrates a benefit. The owner approved the library and compiler
choices, including the measured large-record benefit alongside the repeatable
8.2–10.6 percent short-scalar regression. Remaining consumption and lowering
costs stay in `docs/todo.md`; the selection claims neither uniform improvement
nor native parity. Measurements compare both source algorithms through the
same integrated compiler and keep the historical compiler comparisons separate.

## Slab and Deque trial over v0.63

This comparison began at merged `3a969235`, kernel v0.63. Box descendant
placement and the selected Vector consumption repair were available; the
remaining Vector timing costs retain their explicit reopening conditions.
The end-to-end question is whether stable-slot reuse and a two-ended
queue can support copyable, owning droppable and must-consume elements over
ordinary values at an attributable native cost. The candidates below began as
implementation trials; the selection and remaining-cost sections record their
outcomes without a claim of native parity.

For Slab, first try one boxed backing of cells, each containing an inline
`Slots<T, 1>`, a generation and a free-list link. The inner window expresses
vacancy with its ordinary zero-or-one length; insertion and removal use the
existing place/take rows. An enum of vacant/live cells is a useful compact
native control, but a generic source exchange currently loses the variant
needed to consume an extracted vacant enum without an impossible ownership
arm. Separate metadata plus dense payloads is another ordinary alternative;
it spends a second backing and repairs reverse indexes on removal. The
single-slot candidate is selected for a trial because it keeps one backing,
constant-time reuse and each live payload in its slot without that repair.
Its possible extra metadata word must be measured, not treated as free.

The Slab constructor accepts a runtime capacity within a written const ceiling,
which bounds allocation size, and materializes cells only on first insertion.
Free-list reuse concerns already materialized empty cells. A slot retires
when its generation reaches a caller-selected limit rather than wrapping;
small limits exercise that same production rule. Exhaustion returns the
uninserted owner. Handles are relative to the supplied slab and are ordinary
data, not brands authenticating the allocation. The membership caller compares
weak indexes with a composite retained protocol: removing one index preserves
a remaining reader, object deletion expires weak handles, and the retained
protocol refuses object deletion until its memberships are removed. This is
not a theorem about arbitrary clients maintaining or being unable to forge
ordinary bookkeeping fields.

For Deque, keep endpoint mutation over references to `Box<Ring<T>>` and make
reallocation an explicit consuming rebase that returns a newly allocated
owner. A counted front-take/back-place loop can prove exact length
preservation and release the emptied old backing for arbitrary T. This is a
new-owner conversion, not an automatic-growth promise on a reference helper.
The ordinary `swap` row does not publish exchanged measures, so exchanging
old/new owners does not by itself establish the old-empty and new-length
relations needed by that other interface. REF-4 also still refuses a Ring
range, including an empty one: per-slot visitation does not supply zero-copy
two-span access. Retain the exact rejected programs and these distinctions
alongside the executable candidate.

Before selecting either representation, require complete operation and cleanup
chains with independent results, generation/expiry and wrap boundary cases,
and exact allocation-release accounting through the ordinary formal tests.
Reuse their existing sequential and parallel construction paths; the older
three-mode and allocation-refusal prescriptions are not the v0.63 contract.
Experiments stay outside daily correctness checking. For each candidate,
compare the actual library with C implementing the same representation and
operation contract, then a compact-slot or bulk-rebase control where it
separates a source/layout cost. Record actual bytes per capacity unit,
construction cost separately from steady-state lookup or churn, allocation
counts, and emitted transfers with ordinary and retained helpers. Use scalar
and large owning records, short and longer capacities, and checksum-sensitive
operation sequences. A runtime branch or extra word is a measured cost;
acceptance alone does not select it, and a changed ownership or callback
contract is not a faster implementation of the same operation.

### Executed library boundary

The [Slab](../../../lib/containers/slab.wf) and
[Deque](../../../lib/containers/deque.wf) sources have complete native callers
in the formal corpus, reusing its sequential and parallel modes and shared
allocation observer. Slab covers copy, owned Box and nodrop elements, lazy
materialization, genuine full outcomes, wrong/expired handles, reuse and
generation limits zero, one and u64 maximum. Deque adds zero-sized elements,
zero/one/full capacities, wrapping from both ends, growth and shrink through
rebase, callback order and reuse. Its precise back-append row also permits an
existing filled-slot reference to remain valid. The independent release
ledgers count 19 successful Slab allocations and 21 Deque allocations, including
the membership caller; both ordinary deallocation and observed images execute.
Heap allocation itself is total under STOR-8, so these are not allocation-
refusal tests. Slot exhaustion is a separate ordinary Slab outcome.

The [membership program](../../../tests/programs/containers/slab-membership-program.wf)
answers the retained-membership question for a concrete composite: removing
one index preserves the other reader; weak indexes expire on owner deletion;
the retained composite refuses deletion with two memberships and still with
one, then permits it after both retire. Its two index fields cover one central
object. General multi-object indexing and protection from independently
authored bookkeeping mutations are not established by this example.

The matched [Slab comparison](../../experiments/container-representation/slab-library/RESULTS.md)
and [Deque comparison](../../experiments/container-representation/deque-library/RESULTS.md)
own cost conditions and results. Source acceptance and the formal execution
above are not performance selection evidence.

### Selection and remaining costs

The selected Slab choice keeps one allocation and stable cell positions for
arbitrary owned T. Its scalar cell costs 32 bytes against the tagged C
control's 24; the 256-byte payload costs 280 against 272. The C window/tagged
comparison isolates that extra word, while WF/window timings also include
result layout and call lowering. Retained wide removal and consumption still
perform redundant transfers, so these results do not select the representation
as a performance ceiling. Keep this ordinary implementation available for
composition, and test the remaining transfers before using a Slab cost alone
to justify a compiler-known sparse layout. The approved
`slab-storage` decision records that qualified choice.

The selected Deque choice keeps precise endpoint rows and an explicit new-owner
conversion. Its strongest measured gap is ordinary scalar forward churn;
retaining helpers largely removes that gap, exposing optimization of the
inlined address path rather than an unavoidable reference-interface cost.
A bounded GEP-fact probe removes the redundant descriptor traffic, but its
general target qualification and timing recovery remain unverified. Keep the
library interface and the measured baseline while qualifying that optimization;
do not add a Ring growth primitive solely from this comparison. The approved
`deque-rebase` decision records the interface and its two-span limitation.

These selected library choices supplement the typed-constant representation
choice and the two approved corrections to storage/range correspondence. The
two new leaf nodes do not increase the tree's depth.
The affected current guidance is the container writer pattern and library
README; unresolved source interfaces and lowering opportunities remain in
`docs/todo.md`. No specification or conformance verdict change is part of this
library slice.

The original CSVs and artifact hashes remain evidence for the recorded
pre-merge compiler. Main `7127bcb6` adds checker and aggregate-lowering repairs;
its clean merge also required supplying the const-type inventory to a newly
added unit-test context. The earlier emission comparison established only the
const-forwarding repair's correspondence, not the main integration. The
integrated compiler at `dcbfdc0f` was then checked separately: Deque raw and
optimized IR were byte-identical to the preserved baseline, while Slab's
optimized IR differed only in SSA names after the alias removal. Compiling
the old/current optimized modules with Apple Clang 21 on arm64 produced
byte-identical complete WF module assembly in both normal and retained modes.
The [Slab integration record](../../experiments/container-representation/slab-library/RESULTS.md#integration-verification-at-dcbfdc0f)
and [Deque integration record](../../experiments/container-representation/deque-library/RESULTS.md#integration-verification-at-dcbfdc0f)
give the identities, method and rerun correctness checks. This establishes
that emitted-module correspondence, not linked-image identity or a new timing
run; the original CSVs keep their original compiler and measurement identity.

The subsequent merge of main `95b21cfd` at `4da1710e` was checked with the
rebuilt compiler as well. Fresh raw and normal/retained optimized LLVM for
both libraries matched the saved `dcbfdc0f` artifacts byte-for-byte; all four
C-control optimized modules were unchanged. The same integration records
give this compiler identity and the bounded re-emission commands. The earlier
native, assembly and timing evidence retains its original scope and identity.

Main `e8e1c411`, integrated at `ce9a3870`, subsequently changed Ring front
predecessor lowering and the resource-record writer. Slab's raw module changed,
but both complete optimized modules remained byte-identical, so its earlier
timing evidence was retained with that qualification. Deque's reverse-churn
path changed and was measured again with the unchanged full matrix and C
controls: 2,592 correctness executions passed and 6,336 fresh samples are in
[`measurements-v0.64.csv`](../../experiments/container-representation/deque-library/measurements-v0.64.csv).
The RESULTS records separate this new measurement from the preserved baseline
and identify the changed helper instructions; no Slab timing or optional GEP
probe was rerun.

### Exact unavailable source forms

These are the rejected additions or functions in the linked library's type
context. They state current rules, not proposed amendments.

| Rejected source | Rule and cause | Implemented alternative |
| --- | --- | --- |
| In `slab_new`: `ensures result.cells.inner.len == 0_u64;` | FN-9's result-selector domain does not include an arbitrary aggregate result field. A nominal Deque wrapper's `made.storage.inner.len` has the same limit. | Slab retains its necessary free-list state; the caller establishes length through an ordinary read/branch. Deque needs no extra wrapper state and uses direct `Box<Ring<T>>`, whose `made.inner.len` is admitted. |
| In `slab_find_index`: `ensures when Ok(value: index): deref(slab).cells.inner[index].storage.len > 0_u64;` | FN-9/CALL-4 do not admit this indexed postcondition target. | Export the outer index bound; use `slab_visit` to keep validation and callback in one helper, or re-read occupancy before direct access. |
| The signature `fn borrow_out<T>(value: &T) -> result: &T reads(value) {` | GRAM-3 requires `own` at the result; REF-3/FN-1 prohibit reference escape. | Return owned callback data or a validated index. |
| After `let values = box_ring_new::<u64>(capacity: 4_u64);`: `let count = observe::<u64>(first: &values.inner[0_u64..0_u64], second: &values.inner[0_u64..0_u64]);`, with `observe` taking two `&[T]` arguments | REF-4 refuses even empty Ring ranges. | Per-slot visitation; this remains an explicit missing zero-copy two-span interface. |

The vacant variant does not travel through swap's row to the extracted local.
This complete generic helper rejects under PROV-6 because the admitted facts
cannot show that `offered` no longer holds a must-consume T:

```wf
fn insert<T>(slot: &Option<T>, value: own T) -> result: own Result<unit, T> writes(slot) {
  match deref(slot) {
    Some(value: occupied) => {
      return Err<unit, T>(error: move value);
    }
    None() => {
    }
  }
  let offered = Some<T>(value: move value);
  swap(first: slot, second: &offered);
  return Ok<unit, T>(value: unit);
}
```

The library's `Slots<T,1>` carries vacancy as an ordinary length whose
place/take contracts are available. No impossible occupied cleanup arm or
implicit nodrop discard is introduced. A native tagged cell remains a layout
comparator; the Slab result does not solve every map operation.

Swap also does not publish exchanged measures. Its ordinary write row kills
both operands' old facts under ENT-5, so this helper fails its FN-9 return
obligation:

```wf
fn replace_empty<T>(old: &Box<Ring<T>>, built: own Box<Ring<T>>) -> previous: own Box<Ring<T>> writes(old) contract {
  requires deref(old).inner.len == 0_u64;
  ensures previous.inner.len == 0_u64;
} {
  swap(first: old, second: &built);
  return move built;
}
```

A reread could validate an empty branch, but would not consume a nonempty
nodrop remainder on the other branch. The existing
[atomic-publish probe](../../experiments/container-representation/x1/linear-ring-publish.wf)
also rejects under OP-12/WIN-3 for a nodrop target. The implemented rebase
consumes the original owner directly and returns a genuinely new backing;
no exchange fact is needed. Its counted element loop proves exact length;
append's current lower-bound contract does not state the two-entry sum.

### Ring range correspondence

The [range-reference decision](../../../design/language/ownership/range-reference.md)
previously listed Ring. Commit `a907d9d6` adopted that candidate text. The earlier
`c79188a1` investigation's
[amendment map](../access-effects/SPEC-AMENDMENT-MAP.md) had asked whether to
refuse Ring ranges or admit proved non-wrapping ones, recommending refusal
under its owner-rulings-needed heading. The subsequent `7bc07c04` specification
draft introduced today's blanket refusal. Neither its commit message nor the
retained PR #70 discussion supplies a Ring-specific ruling. The earlier
released v0.59 VIEW-2 had admitted proved non-wrapping views, including empty
ones. This established tree/spec drift and an unrecovered selection
ground; it does not establish that the narrowing was unauthorized.

The owner selected retention of REF-4 as a fresh ruling: the tree now excludes
Ring ranges, including empty and non-wrapping ones. The alternative of admitting
proved-contiguous Ring ranges still needs an explicit empty-range rule plus
formation, invalidation and lowering evidence. Arbitrary wrapped logical ranges
would also exceed the selected pointer/count representation. The ruling does
not reconstruct the missing historical ground and does not make slot visitation
a replacement for a two-span consumer; that capability remains a follow-up.

### Compiler corrections exposed by composition

The Slab visitor uncovered a partial-instantiation spelling defect. A generic
callee binding R to unit while forwarding a type, const or function parameter
was treated as its own symbolic spelling authority; `move` was then rejected
as a copy move. OWN-1/FN-2 assigns that authority to the written template, not
every partly substituted body. The correction compares declaration identities
with the template's own symbolic substitution. The generic tests retain
canonical copy-move and repeated-consume negatives; no ownership or overlap
rule is weakened.

A separate const-guard probe exposed lost source type information:

```wf
fn increment<const limit: u64>(value: own u64) -> result: own u64 pure {
  if value < limit {
    return value + 1_u64;
  }
  return value;
}
```

The missing implicit maximum prevented ENT-2/ENT-4 from proving the OP-2
increment domain. A local alias or a redundant written invariant happened to
supply enough evidence through a different path. Conversely, the affine image
path assigned u64 bounds even to a signed const parameter, incorrectly
accepting a symbolic assertion that every i8 parameter is nonnegative.
This demonstrated an incorrect template judgment, not an executable negative-
value misuse. MSR-6 already requires the exact declared integer type.

The correction retains that type with the declaration-anchored term and its
affine image. A read-only declaration-type inventory serves the ordinary
entailment and isolated refinement contexts, avoiding type propagation through
every storage-extent and captured-index representation. This distinction
matters when a u8 const parameter supplies a u64 formal or storage extent:
the constant must remain one identity with its source bounds. It adds no
automatic proof family. The corresponding compiler choice is recorded in
checker-facts; domain, insufficient-guard and forwarding regressions
exercise the existing proof checker.

## Ring payload address qualification

The next lowering experiment starts from merged main `1b916975`. The
[Deque comparison](../../experiments/container-representation/deque-library/RESULTS.md)
records a 2.256–2.405x normal-mode scalar forward-churn cost against its C
control. Its separate, untimed four-GEP probe identifies an unsigned offset
fact that removes repeated descriptor traffic in one concrete instance.
That is a reason to investigate the general lowering contract, not evidence
that adding the flag to every element address is valid or faster.

The question is whether existing source bounds and selected-target layout
qualification establish the complete LLVM promise for ordinary Slots/Ring
payload projections. The argument must cover the actual padded header,
element allocation stride, enclosing object extent, intermediate offsets,
pointer-index width, zero capacity, zero stride, and any one-past use of the
shared projection. Logical Ring wrap arithmetic is a separate operation;
the candidate must not attach an unsigned no-wrap assertion to it merely
because a final physical element is in bounds. The
[LLVM instruction contract](https://llvm.org/docs/LangRef.html#getelementptr-instruction)
is the target obligation; STOR-6 and DIAG-2 remain the language authorities.

Selection criteria, recorded before this experiment's timings:

- Every emitted optional fact has a complete argument from existing checked
  and selected-target facts. Source acceptance and target-layout qualification
  agree with this optional fact enabled or withheld. Unsupported assembler spelling
  retains a correct portable lowering rather than rejecting a WF program.
- Existing maintained regressions cover relevant ownership, wrap, zero-size,
  nested-layout and target-domain boundaries. Add only observations absent
  from those cases. Successful timing runs do not establish allocation-failure
  behavior: the maintained exhaustion tests own the resource-failure boundary,
  while the container caller and allocator observer own the exact release ledger.
- Reuse the existing Deque workload, independent checksums, allocation ledger,
  scalar and wide-record payloads, normal and retained helpers, C controls and
  counterbalanced matrix. Compare fresh baseline and candidate emissions and
  timings on the same toolchain. Withholding this fact is a correctness
  control, not a substitute for isolating its performance contribution; this
  experiment does not require a new public facts-mode switch.
- Select production emission only if the complete target contract holds and
  the paired results show a repeatable benefit beyond cohort and control
  variation, without an unexplained material loss elsewhere in the matrix.
  Attribute any remaining gap rather than claiming universal native parity.

Keep the implementation within ordinary address emission and the existing
toolchain-capability path where possible. Do not change source syntax,
container representation, Ring range admission, or host/runtime protocols to
obtain this result. The material compiler choices were reviewed as amendments
beside the design tree; the owner approved both revisions after DCR.

### Representation premise and target contract

Inspection found a representation mismatch before extending optional facts.
The target calculator gives `Slots<T, 0>` its eight-byte descriptor with
eight-byte alignment. Emission instead retained `{ i64, [0 x T] }`.
LLVM arrays keep their element's ABI alignment even at length zero
([DataLayout implementation](https://llvm.org/doxygen/DataLayout_8cpp_source.html)).
For the ordinary opaque `OutputStream` type, whose emitted representation is
16-byte aligned, that LLVM aggregate has size and alignment 16. An outer
runtime Ring of such values is therefore checked with header/stride 24/8
while its emitted allocation arithmetic uses 32/16. This is a mismatch in
qualification; the allocator uses the emitted size, so these calculations
alone do not demonstrate an observed undersized allocation.

OP-9's zero repetitions contain no element layout. Raising the checked size
and alignment would force the nested allocation to fail the unchanged
language ceiling. The selected repair instead extends the existing
`Array<T, 0>` emission convention to the empty payload of constant-capacity
Slots and Ring: `[0 x i8]`, retaining their length/head words. Nonzero-capacity
zero-size elements and runtime-capacity typed tails keep their distinct
representations. A maintained regression must compare the complete nested
object and actual emitted allocation against the checked byte boundary,
including Array conversion and cleanup, rather than only repeat the
checker's preexisting size result.

The maintained backend regression
`zero_capacity_windows_keep_header_layout_inside_nonempty_storage` embeds
both empty window shapes between scalar sentinels in one record. The checked
record is 40 bytes with alignment eight, and its one-slot runtime Ring needs
64 bytes including the header. A same-source native before/after run with
retained helpers and an allocation observer measured 96 bytes from the saved
`1b916975` baseline and 64 from the repair: the independent 64-byte oracle
exits six before the fix and zero afterward. The test also checks a synthetic
64-byte allocation limit and the rejection one byte below it, plus empty
range formation, Array conversion, sentinel preservation and cleanup. These
observations distinguish actual layout correspondence from an assertion
about the calculator alone.

Once representation and qualification agree, let `H` be the actual padded
tail-field offset, `S` the actual allocation stride of that GEP's element,
`C` the capacity, and `E` the complete window extent. On the currently
supported targets the pointer index is i64 and qualification establishes
`E <= M = 2^63 - 1`. For a positive-stride payload and an admissible physical
offset `p`, including a one-past pointer when permitted:

```text
0 <= p <= C <= floor((M - H) / S)
0 <= H <= H + p*S <= E <= M
```

The GEP's successive offsets are zero, the nonnegative header offset, and
the nonnegative scaled physical index. Each fits both signed and unsigned
index arithmetic. For a nested window at parent offset `q`, complete-parent
qualification also establishes `q + E <= A`, the parent allocation extent.
Every intermediate pointer remains in that same allocation. LLVM allocations
cannot cross the unsigned address-space boundary, including their one-past
pointer ([allocated-object contract](https://llvm.org/docs/LangRef.html#allocated-objects)).
These facts establish all four `nuw` requirements, not only index scaling.
For zero stride, ordinary emission already uses physical address operand
zero; no bound on the potentially huge logical capacity is needed. For an
erased constant-capacity-zero payload, the actual tail type is i8 and its
only admissible boundary offset is zero. Ring wrap operations retain their
own wrapping semantics and receive no new arithmetic assertion from this
argument.

Two target expressions deserve comparison before selection: direct GEP `nuw`
with a supported-spelling probe and plain-`inbounds` fallback, or an
`llvm.assume` that the actual normalized address index is signed nonnegative
beside the existing `inbounds` projection. The latter states a subset of the
same proved domain with portable syntax; whether optimization recovers the
same benefit is an empirical question. GEP `nuw` first appears in
[LLVM 19](https://releases.llvm.org/19.1.0/docs/ReleaseNotes.html#changes-to-the-llvm-ir);
[LLVM 18](https://releases.llvm.org/18.1.8/docs/LangRef.html#getelementptr-instruction)
already defines the nonnegative inbounds implication and the assumption
intrinsic. Use the existing four-position probe only to discriminate these
expressions, then validate the selected compiler path and full matrix.

The four-position comparison on Apple Clang 21 admits both expressions and
passes the same independent oracle. Both reduce the scalar forward loop from
four loads/four stores to one payload load/one payload store; their hot-loop
assembly is identical. Their complete scalar functions are not identical,
so this observation does not establish timing parity elsewhere. The selected
candidate for the full compiler experiment is the nonnegative-index
assumption: it recovers the observed local optimization without a new LLVM
dialect requirement. A build-time syntax probe would inspect the build's
compiler, while the native builder uses its selected native compiler; those
need not be the same consumer. The existing [comparison record](../../experiments/container-representation/deque-library/RESULTS.md)
owns the probe commands, counts and artifacts. After the complete
normal/retained paired comparison, the owner selected production emission
provisionally with the tradeoff recorded below.

The implementation stays in the shared run projection and intrinsic registry.
A test-only withholding choice exercises that same emitter after the same
target validation; it is not a source mode or a second qualification path.
The corresponding decision is in `compiler/backend-facts`; the zero-capacity
representation decision is in `compiler/storage-representation`. This keeps
the representation repair independent of the optional optimization and needs
neither a second table of per-address qualification nor a public compiler
switch. No source rule or container contract changes.

### Retained reverse-path discriminator

The first complete Clang 21 baseline/candidate sweeps show a reproducible
retained-scalar reverse-churn regression, about seven percent after the
in-binary C normalization, alongside the large forward-path gain. Independent
disassembly comparisons find identical reverse-loop and endpoint-helper
instructions, register dependencies and memory operations. The loop stays at
the same address, but both endpoint helpers move 32 bytes earlier. This is
evidence of a code-placement difference, not yet evidence that placement
causes the timing difference.

Before additional timing, the discriminator is to give both scratch retained
modules' scalar `pop_back` definition 128-byte alignment. Verify that the
two endpoint helpers then occupy identical addresses and preserve their
instructions, and that the calling loop remains unchanged; otherwise the
experiment does not isolate the proposed cause. Use the existing native
oracle and counterbalanced retained measurements, with the same source,
seeds and C controls. If the regression collapses after this normalization,
report the production-layout cost as placement-sensitive; if it persists,
keep the regression unexplained. The scratch alignment is an instrument,
not a proposed production alignment policy or a reason to select facts by
element identity.

The 128-byte-alignment construction failed that isolation check: it made
the two endpoint addresses agree, but the linker also moved both complete
WF text regions by 96 bytes, including the trace and callbacks. No oracle
or timing was run on those images. The follow-up instrument therefore keeps
the original section alignment and adds exactly 32 bytes of unreachable
text padding before the candidate's scalar `pop_back` symbol. Before timing,
verify against the original binaries that the trace and callbacks retain
their addresses and instructions and both endpoints recover the original
baseline addresses and instructions. If this narrower construction also
fails isolation, do not time it or reinterpret the first experiment as a
success. This padding is likewise confined to scratch evidence.

The 32-byte padding control met the isolation checks and passed both native
oracles. Its two measurement orders disagree: the normalized retained scalar
reverse ratio stays about 1.06–1.08 in the first pair, then approaches one in
the reversed pair because the last baseline itself slows. The original
production-layout regression therefore remains unexplained; neither the
first alignment failure nor the second mixed result establishes a particular
cache or branch-prediction cause. No padding enters production.

The predeclared selection criterion is **not fully met**. After DCR, the owner
selected provisional retention of the qualified fact for its reproducible
inline scalar forward and rebase improvements, explicitly accepting the
recorded retained-reverse cost and its unresolved cause. The alternative is
to remove the optional fact and deliver the independently verified
zero-capacity representation repair alone. No application-frequency
distribution has been measured, so the results do not establish that every
Deque consumer benefits from the tradeoff. The owner-approved backend-facts
decision records the first choice; it does not establish that the original
criterion passed. Reopen that choice for a consumer
dominated by retained reverse calls, a changed native toolchain, or a further
material regression under the same matched-contract comparison. The
maintained TODO keeps the remaining scalar gap and cost attribution open.
