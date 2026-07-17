# Ordinary-Library and Held-Out Witness Registry

Status: proposed G0-Core research registry, 2026-07-14; owner review pending.
The exact-hash review disposition is recorded separately in
`G0-CORE-HOSTILE-REVIEW.md`. On G0-Core closure, this file freezes
research coverage purposes, observable contracts, family dependencies,
held-out budgets, and anti-special-casing rules for later lock drafting. That
research freeze is not a language, mechanism, experiment, or production
decision. This file contains no witness implementation, candidate mechanism,
harness, scored trace, or production authorization.

## 1. Claim being tested

Rust's standard library is a finite demand anchor, not a generativity proof.
Passing only Rust-shaped containers could mean that xlang or a privileged
standard library prebuilt those exact types. The stronger detailed claim is:

> Ordinary no-unsafe xlang libraries can implement the registered sequential,
> unique-owner collection and topology contracts efficiently through the same
> public checked mechanisms available to unrelated libraries.

This is not the complete general-purpose systems-language claim. Concurrency,
shared ownership, resources and FFI, custom allocation, async cancellation,
pinning/address-sensitive values, full text semantics, and target intrinsics
retain separate blocked claims in `SYSTEMS-DOMAIN-LEDGER.md`.

The witness set is deliberately finite. Visible witnesses separate known
topologies and failure obligations. Four training-excluded held-outs test one
dense-storage derivation, one sparse-storage derivation, and two cross-container
invariants. A named data
structure need not become a kernel or standard-library type merely because it
is a witness.

### 1.1 Payload boundary

Every B, W, and H retained data value in this registry is region-free and
contains no borrow. This restriction applies independently to keys, values,
components, nodes, edges, priorities, callback environments, separate state,
cached or queued Items, and values nested inside another payload. A borrow
returned by `get`, lookup, iteration, or arena placement is an owner-tied
access result; it is not a borrow stored inside an arbitrary retained value. A
later witness may admit such a borrow-bearing value only after the
stored-borrow family closes and its exact budget explicitly includes
`BR-STORED`. No current witness budget does so. Opaque source-cursor authority
may still carry its sealed provenance map under `BR-CURSOR`; projecting or
storing that cursor as arbitrary data would require `BR-STORED`.

## 2. Protected baselines

| ID | Contract | Protection |
|---|---|---|
| B-FIX | Existing fixed, fully initialized Copy buffer | Preserve its two-word owner, one allocation, fixed length, checked indexing, and optimized body. No capacity, occupancy, generation, sharing, or policy field/branch may appear. |
| B-P2 | Existing append-only SoA/index pool | Preserve non-reused indices and the measured append-only access path. No generation, recycling, sparse-state, shared-ownership, or provenance branch may appear. |

These controls run for every family even when that family does not use their
topology. A shared substrate is rejected if its generality becomes a tax on
either protected contract.

### 2.1 Closed dependency vocabulary

Every dependency budget below is an exact allowlist. A token must be one of:

- an exact capability ID from `CAPABILITY-OBLIGATION-REGISTRY.tsv`;
- an exact G0 coverage-cluster ID from `RUST-DATA-CONTRACT-CENSUS.tsv`;
- a baseline or witness ID defined in this file;
- one of the exact family-closure IDs below; or
- a named frame ID from `SYSTEMS-DOMAIN-LEDGER.md`.

| ID | Exact meaning |
|---|---|
| `K-SCALAR` | The existing checked scalar, Boolean, index, control-flow, `Option`-class, and `Result`-class kernel operations. It grants no storage transition, callable behavior, unchecked memory access, or privileged frame. |
| `FAM-DENSE` | The selected ordinary-library dense affine sequence public contract, but only after its own family lock, evidence, hostile review, and adoption close. |
| `FAM-UMAP` | The selected ordinary-library unordered map public contract, but only after its own family lock, evidence, hostile review, and adoption close. |

Wildcards, prefixes, negated sets, and prose aliases are forbidden in a budget.
Adding a future capability whose ID shares a prefix with an allowed token does
not widen any budget. A G0 coverage-cluster token names only a coarse obligation
envelope; it grants no operation, capability union, cost, or executable witness
dependency. Before witness construction, the applicable Family Lock must bind
every such token to exact `member_contract_id` and `outcome_id` units and replace
the budget edge with those lock-local or adopted-family public contracts. Every
executable dependency token grants only the frozen public checked caller contract
named by that token after its required gate closes. It never imports private
representation, private capabilities, internal facts, unchecked transitions,
compiler recognition, or implementation-only access.
A frame token likewise grants only its reviewed public checked caller contract.
It never grants raw payload access, manual liveness authority, unchecked
capacity mutation, allocator-identity forgery, manual deallocation, or any
other private frame privilege.

## 3. Visible capability and topology witnesses

The mandatory operation canaries in the D11 capability matrix remain in force.
The table below refines the ordinary-library topology witnesses that prevent a
named-container-only result.

| ID | Role | Frozen observable contract | Separating purpose | Capability dependency budget |
|---|:---:|---|---|---|
| W-POOL | W | Insert, shared `get`, replace, and remove affine payloads through finite copyable logical handles; a returned borrow is tied to the pool and blocks incompatible mutation. Each slot generation advances without wrap; reaching its maximum retires that slot permanently. The pool has a frozen maximum slot count, so insertion returns `IdentityExhausted(own T)` as the sole owner before any stale handle could revive, with the pool unchanged. Storage/history is O(maximum slots), including retired slots. A same-typed handle from another pool is a memory-safe logic error with no guaranteed rejection. Insertion is amortized O(1), including identity-exhaustion detection. Once no committing growth is required, insertion, shared get, replace, and remove each touch O(1) slots and metadata. Disposition is exact. | Distinguishes reusable storage and temporal freshness from an append-only index or a bare-key slab. | `K-SCALAR`, `ST-SPARSE`, `OW-INIT`, `OW-MOVEOUT`, `OW-REPLACE`, `OW-DROP`, `EX-NORMAL`, `EX-ABANDON`, `EX-ABORT`, `BR-PROV`, `BR-REBORROW`, `BR-RESULT`, `BR-INVALIDATE`, `FL-CAPACITY`, `FL-ALLOC`, `FL-ATOMIC`, `ID-LOGICAL`, `ID-FRESH`, `ID-POOL`, `FT-STATE`, `FT-IDENTITY`, `AB-SEAL`, `AB-GENERIC`, `F-ALLOC` |
| W-ARENA | W | Amortized O(1) placement of an already-complete `own T` uses an owner-tied checked placement authority whose exact write footprint contains only allocation metadata and fresh dead storage. It returns a borrow tied to the arena; that borrow remains valid across later arena placements because each later write footprint is proved disjoint from every prior payload borrow and no prior payload relocates. Whole-arena unique access, hidden shared-write/interior mutation, or a runtime borrow table does not satisfy this contract. Any reset/destroy expression requiring incompatible arena access is statically rejected until every phase borrow ends. No individual free exists; after borrows end, reset/destroy disposes every complete affine payload exactly once. No partial aggregate is constructed inside arena storage. `FAM-DENSE` is a required exercised predecessor used only as the unbounded registry of sealed affine `F-ALLOC` backing-block owners. Every committed regular chunk and dedicated oversized block has exactly one retained block owner until reset/destruction. Registry growth may relocate sealed block-owner tokens, but relocation changes neither backing-allocation identity nor live payload-borrow provenance and moves neither allocation nor payload. Each payload borrow is rooted in the arena owner and exact retained backing allocation, never in a registry slot or token address. The reviewed `F-ALLOC` caller contract and access proof must establish this property; token relocation alone is not evidence. Recoverable block-acquisition or registry-retention failure returns `Failure(error, own T)` as the sole owner, destroys any uncommitted block owner exactly once, and leaves all existing contents and borrows unchanged. For a lock-frozen regular-chunk usable-byte budget C, C must hold at least eight minimum test payloads; a request is regular only when its maximum aligned footprint fits an empty regular chunk and is otherwise oversized. The resource envelope below charges regular payload extents R, regular alignment and terminal fragmentation P, dedicated aligned request footprints and actual acquired usable bytes, retained block-owner count K_peak, noncommitting calls F, successful noncommitting bytes Z, peak memory, and growth policy. Each successful oversized request uses one separately charged dedicated block and committing successful call. No logical identity, observable address promise after a borrow ends, self-reference, `Pin`-class projection, or pre-drop notification is implied. | Bulk phase reclamation differs from per-slot deletion and from Rust bump allocators that intentionally skip payload destruction, without importing the deferred address-stability family. W-ARENA is a post-dense composition/access witness and cannot close, select, or adopt `FAM-DENSE`. | `K-SCALAR`, `FAM-DENSE`, `ST-DENSE`, `OW-INIT`, `OW-DROP`, `EX-NORMAL`, `EX-ABANDON`, `EX-ABORT`, `BR-PROV`, `BR-REBORROW`, `BR-RESULT`, `BR-DISJOINT`, `BR-INVALIDATE`, `BR-CURSOR`, `FL-CAPACITY`, `FL-ALLOC`, `FL-ATOMIC`, `AB-SEAL`, `AB-GENERIC`, `FT-STATE`, `F-ALLOC` |
| W-SMALL | W | No heap allocation through inline capacity N; insertion at N+1 performs one sound spill; contiguous slice semantics survive; affine pop/remove/drop work; failed spill returns `Failure(error, own T)` as the sole owner and leaves the small sequence unchanged; automatic spill-back is not required. | Exposes a one-time representation transition and an externally measurable allocation ceiling absent from an ordinary growable sequence contract. | `K-SCALAR`, `ST-FULL`, `ST-DENSE`, `ST-HOLE`, `OW-INIT`, `OW-MOVEOUT`, `OW-REPLACE`, `OW-RELOCATE`, `OW-DROP`, `EX-NORMAL`, `EX-ABANDON`, `EX-ABORT`, `BR-PROV`, `BR-REBORROW`, `BR-RESULT`, `BR-INVALIDATE`, `FL-CAPACITY`, `FL-ALLOC`, `FL-ATOMIC`, `AB-SEAL`, `AB-GENERIC`, `FT-STATE`, `F-ALLOC` |
| W-RECUR | W | Finite layout, failure-atomic unique recursive construction from complete offered field/child owners, node extraction, mutable cursor, and bounded-stack destruction for adversarial depth. | Tests unique recursive ownership without importing shared ownership or stable raw addresses. | `K-SCALAR`, `BOX-NEW-01`, `OW-MOVEOUT`, `OW-REPLACE`, `OW-DROP`, `EX-NORMAL`, `EX-ABANDON`, `EX-ABORT`, `BR-PROV`, `BR-REBORROW`, `BR-RESULT`, `BR-INVALIDATE`, `BR-CURSOR`, `FL-CAPACITY`, `FL-ALLOC`, `FL-ATOMIC`, `AB-SEAL`, `AB-GENERIC`, `F-ALLOC` |
| W-GRAPH | W | Frozen CSR has O(V+E) storage and contiguous edge scans. Dynamic form has stable non-reviving node/edge identities, O(1) lookup, O(local degree) node removal including incident edges, unrelated-handle stability, and O(1) known-neighbor handle-based splice/rewire. | A pool alone does not test referential integrity, cascading mutation, or multi-node repair. Handle-based rewiring is the safe analogue under the current pin/intrusive deferral. | `K-SCALAR`, `FAM-DENSE`, `W-POOL`, `ST-DEPENDENT`, `OW-MOVEOUT`, `OW-REPLACE`, `OW-DROP`, `EX-NORMAL`, `EX-ABANDON`, `EX-ABORT`, `BR-PROV`, `BR-REBORROW`, `BR-RESULT`, `BR-DISJOINT`, `BR-INVALIDATE`, `BR-CURSOR`, `FL-CAPACITY`, `FL-ALLOC`, `FL-ATOMIC`, `ID-LOGICAL`, `ID-FRESH`, `ID-POOL`, `FT-STATE`, `FT-IDENTITY`, `AB-SEAL`, `AB-GENERIC`, `IT-SHARED`, `IT-UNIQ` |
| W-ECS | W | Two or three fixed archetypes suffice. Entity identity remains stable while adding/removing a component migrates aligned affine columns; swap-removal repairs the displaced entity's reverse location; column scans remain contiguous; no per-entity allocation; failure duplicates or loses no payload. | Existing append-only compiler SoA does not test atomic movement across several aligned buffers plus reverse-index repair. | `K-SCALAR`, `FAM-DENSE`, `W-POOL`, `ST-DENSE`, `ST-DEPENDENT`, `ST-HOLE`, `OW-INIT`, `OW-MOVEOUT`, `OW-SWAP`, `OW-RELOCATE`, `OW-DROP`, `EX-NORMAL`, `EX-ABANDON`, `EX-ABORT`, `BR-PROV`, `BR-REBORROW`, `BR-RESULT`, `BR-DISJOINT`, `BR-INVALIDATE`, `FL-CAPACITY`, `FL-ALLOC`, `FL-ATOMIC`, `ID-LOGICAL`, `ID-FRESH`, `FT-STATE`, `FT-IDENTITY`, `AB-SEAL`, `AB-GENERIC`, `IT-SHARED`, `IT-UNIQ` |
| W-GAP | W | Logical sequence content is independent of gap position; shared indexed observation returns an owner-tied borrow; insert/delete at the gap are amortized O(1); moving the gap is O(distance); the hole is never readable or droppable as T; growth and recoverable failure preserve the old logical sequence. Use bytes and affine records, not Unicode semantics. | Separates a simultaneous initialized prefix and suffix from a one-prefix owner or arbitrary sparse bitmap, and prices direct bulk movement. | `K-SCALAR`, `ST-DENSE`, `ST-HOLE`, `OW-INIT`, `OW-MOVEOUT`, `OW-RELOCATE`, `OW-DROP`, `EX-NORMAL`, `EX-ABANDON`, `EX-ABORT`, `BR-PROV`, `BR-REBORROW`, `BR-RESULT`, `BR-INVALIDATE`, `FL-CAPACITY`, `FL-ALLOC`, `FL-ATOMIC`, `FT-STATE`, `AB-SEAL`, `AB-GENERIC`, `F-ALLOC` |
| W-PIPE | W | An ordinary external library composes lazy sources, nested stateful transform/select adapters, a two-input adapter, and an early-stop or recoverable-error consumer over shared, unique, and owning affine inputs. Output order, callback order/count, progress, and exhaustion are exact. Every retained callable environment, separate State, and cached or queued Item in this witness is region-free and borrow-free; callbacks may observe call-scoped input borrows but may not retain them. Every early stop, error, and permitted abandonment leaves borrows valid and disposes each consumed and remaining owner exactly once. Every non-lending unique Item is disjoint from all still-live sibling Items. The affine cursor preserves an exact field/branch/epoch provenance map across moves: chain branches and zip fields retain their corresponding sources; shared inputs may use the same owner; the unique-input case uses separate owners and overlapping unique sources are rejected. An already yielded external shared borrow may outlive adapter destruction while its source remains live. Any borrow-bearing callable environment, State, or cached Item is rejected as outside W-PIPE rather than silently authorized. Advisory size hints never authorize unchecked access or uninitialized reads. | Separates reusable traversal composition from one hand-written loop and tests whether xlang can derive a zero-materialization pipeline without copying Rust's trait surface. | `K-SCALAR`, `BR-PROV`, `BR-REBORROW`, `BR-RESULT`, `BR-DISJOINT`, `BR-INVALIDATE`, `BR-CURSOR`, `OW-MOVEOUT`, `OW-DROP`, `EX-NORMAL`, `EX-ABANDON`, `EX-ABORT`, `FL-CALLBACK`, `AB-BEHAVIOR`, `AB-STATEFUL`, `AB-GENERIC`, `IT-SHARED`, `IT-UNIQ`, `IT-OWN`, `IT-COMPOSE` |

W-ARENA is homogeneous: one arena instantiation and owner store one fixed `T`.
The lock-frozen maximum aligned footprint of `T` selects the route for that
entire instantiation. If it fits an empty regular chunk, every placement uses
regular chunks; otherwise every placement uses one dedicated oversized block.
One arena owner never mixes the two classes. Type erasure, runtime type/drop
metadata, or a per-placement heterogeneous representation branch is outside
this witness budget.

W-PIPE's structural gate rejects every intermediate collection, adapter heap
allocation, per-element allocation, indirect behavior call, stronger-than-
O(depth) live adapter state, and avoidable second source pass. It compares the
ordinary-library composition with a hand-fused xlang loop and the equivalent
idiomatic Rust 1.97.0 pipeline under matched callbacks and inputs. Code-size
growth from monomorphization remains a charged output rather than a hidden
cost.

W-PIPE's provenance canaries swap chain branch tags and zip field-source tags,
use two shared inputs from one owner, and use separate owners for unique inputs
while rejecting overlapping unique sources. Moving the cursor preserves the
complete provenance map. An external yielded shared borrow remains usable after
adapter destruction while its source lives; source death or incompatible
mutation is rejected. Any borrow-bearing callable environment, separate State,
or cached Item is rejected as outside this witness rather than silently
authorized by `AB-STATEFUL` or `BR-CURSOR`. No runtime provenance table or tag
is permitted beyond semantically required cursor phase/state.

W-ARENA's structural gate includes a positive trace that places one payload,
reads its returned borrow, places a second payload while the first borrow is
live, and reads the first borrow again. Negative canaries reject a whole-root
unique placement API while that borrow is live, any placement path that can
relocate a prior payload, reset or destruction while a payload borrow is live,
and every forged, stale, or overlapping fresh-slot proof. A static footprint
proof adds no runtime borrow flag, borrow table, or per-placement borrow check.

The arena owner registry is not optional allowlist headroom. `FAM-DENSE` must
retain the sole sealed owner of every regular and dedicated `F-ALLOC` block.
Every newly acquired empty block must be committed to that registry before
`own T` is written; after commitment the payload write is infallible under the
proved footprint and capacity. A proof-equivalent ordering is admissible only
if it establishes the same atomic ownership result. If block acquisition or
registry retention fails, the offered `own T` is the sole returned owner, any
uncommitted block owner is destroyed once, and all earlier payloads and borrows
are unchanged. `F-ALLOC` grants no raw or copyable deallocation ticket, manual
release, pointer chain, leaked owner, fixed block-owner ceiling, or hidden
standard-library registry.

Moving a sealed block-owner token during registry growth is distinct from
relocating the allocation it owns. The token's source slot dies and one
destination slot becomes its sole owner, while allocation identity, payload
addresses for the duration of every live borrow, and borrow provenance remain
unchanged. Payload borrows derive jointly from the arena owner and the exact
retained allocation; they never derive from a registry index, registry-slot
address, or token address. The reviewed `F-ALLOC` caller contract and the
access proof must state this preservation explicitly. Mere pointer-like token
shape, successful allocation, or token relocation is not a provenance proof.

Mandatory arena testing uses two distinct homogeneous instantiations. The
regular trace uses a small `T`, crosses multiple regular chunks, forces at least
two owner-registry growth events, and keeps a borrow into the first chunk live
and readable after every growth. It acquires no dedicated blocks and reports
`D_req = D_acq = D_peak = J = 0`. The oversized trace uses a `T` whose maximum
aligned footprint exceeds C, retains multiple dedicated blocks, forces at least
two owner-registry growth events, and keeps a borrow into the first dedicated
payload live and readable after every growth. It acquires no regular chunks and
reports `R = P = 0` with zero regular-block commits. Failure injection covers
every applicable block acquisition and registry push or growth point in each
trace. After all borrows end, reset and final destruction each drop every
complete payload exactly once while its backing block owner remains live. Only
after the last payload in a block is destroyed may that sealed block owner be
released, exactly once. Reset and final destruction free every retained block owner exactly once.
Neither path visits dead storage as `T`. Canaries reject
retaining only the current block, a fixed owner table, leaked regular or
dedicated owners, registry-slot/token-address provenance, borrow retargeting or
invalidation after token relocation, block release before its last payload
destruction, registry growth that moves payload allocations, one regular
backing allocation per payload, unreported registry storage,
dedicated-block alignment/rounding/slack, arbitrary dedicated over-allocation,
or any post-commit recoverable edge without an exact rollback proof.

### 3.1 Exact visible-witness ownership and failure outcomes

The following outcomes complete the coarse W contracts without choosing API
spelling, storage representation, or a language mechanism. An `own` argument
is dead at the caller after the call begins. Every nonsuccess outcome that
returns it creates the sole new owner; "preserve or return" is not an allowed
alternative. `Unchanged` means all logical contents, order, identities,
metadata-to-payload relations, and live borrows are exactly as before the call.

| Witness | Exact ownership and recoverable-failure outcomes |
|---|---|
| W-POOL | Successful insert consumes `own T` and returns one fresh live handle. Identity exhaustion or recoverable capacity/allocation failure returns `IdentityExhausted(own T)` or `Failure(error, own T)` with the pool unchanged. Replace on a matching live handle consumes the offered `own T` and returns the old `own T`; an invalid or stale handle returns `Invalid(own T)` unchanged. Remove returns `Some(own T)` for a matching live handle or `None` unchanged. Shared get returns an owner-tied borrow only for a matching live handle. Cross-pool handle behavior remains the explicitly frozen memory-safe logic-error policy in the primary row. |
| W-ARENA | Successful placement consumes an already-complete `own T` through the checked footprint-limited authority and returns an owner-tied borrow. Every new regular or dedicated block owner is committed to the imported `FAM-DENSE` registry before the payload write. Recoverable block-acquisition, registry-growth, capacity, or allocation failure returns `Failure(error, own T)` with every prior placement and borrow unchanged; any uncommitted block owner is destroyed exactly once. No partial aggregate enters arena storage. A later placement proves its metadata/fresh-slot write footprint disjoint from every live payload borrow and never relocates prior payloads. Registry growth may relocate only sealed block-owner tokens; it preserves backing-allocation identity and every arena/allocation-rooted payload-borrow provenance, and no borrow derives from a registry slot or token address. Any reset/destroy expression requiring incompatible arena access is statically rejected while a phase borrow is live. After every phase borrow ends, reset/destroy destroys each complete payload while its backing block owner remains live, then releases that block owner exactly once after its last payload is destroyed. There is no runtime `BorrowLive` result or dynamic borrow-state dependency. |
| W-SMALL | Push consumes `own T` and returns `Pushed`; failure, including failed inline-to-heap spill, returns `Failure(error, own T)` unchanged. Pop returns `None` unchanged or `Some(own T)`. Ordered removal of a valid index returns the selected `own T` and shifts remaining owners once; an invalid index returns `OutOfBounds` unchanged. Destruction drops every live inline or spilled payload exactly once. |
| W-RECUR | Successful node construction consumes every complete offered affine field/child owner exactly once. Recoverable construction, capacity, or allocation failure occurs before commitment and returns every offered owner unchanged; no partial node becomes owned state. Node extraction consumes the node and returns each field/child owner exactly once. A cursor returns only owner-tied borrows and owns no payload. Normal abandonment and adversarial-depth destruction of committed nodes leave no lost or duplicate owner. |
| W-GRAPH | Node/edge insertion consumes offered payload owners only on success and returns fresh handles; invalid endpoints, duplicate edge under the frozen graph policy, identity exhaustion, or recoverable capacity/allocation failure returns every offered owner in one exact error variant with the graph unchanged. Edge removal returns its `own edge_payload` or `None`. Node removal returns the `own node_payload` and every incident `own edge_payload` exactly once in the selected dense result owner, or `None` unchanged; any result-storage allocation failure occurs before commit and returns an error unchanged. Known-neighbor rewire consumes no payload and either commits all endpoint/link repairs or returns an error unchanged. |
| W-ECS | Spawn consumes the fixed archetype's offered component owners and returns one entity handle, or returns `Failure(IdentityExhausted, all offered owners)` on identity exhaustion and `Failure(error, all offered owners)` on other recoverable failure, with the ECS unchanged. Add-component migration consumes the offered component only on success; an already-present component returns `AlreadyPresent(own component)` unchanged, and recoverable failure returns `Failure(error, own component)` with the entity in its old archetype. Remove-component returns the removed `own component` and commits migration, or `Missing` unchanged. Despawn returns every live component owner exactly once. Every swap-repair moves the displaced entity's owners once and repairs its reverse location in the same successful transition. |
| W-GAP | Insert at the gap consumes `own T` and returns `Inserted`; recoverable growth/capacity failure returns `Failure(error, own T)` with logical content and gap position unchanged. Delete immediately after the gap returns `None` unchanged at logical end or `Some(own T)` while preserving the order of all other values. Moving the gap consumes no payload, is total for an in-range destination, and changes no logical sequence content. Destruction drops every live prefix/suffix payload once and never drops the hole. |
| W-PIPE | Shared and unique yields retain their exact per-leaf source provenance, every unique yield is disjoint from still-live siblings, owning yields transfer each item once, and every early stop, recoverable error, exhaustion, or permitted abandonment disposes consumed and remaining owner state exactly once. Cursor movement preserves the field/branch/epoch provenance map. Retained callable environments, State, and cached Items remain region-free and borrow-free; borrow-bearing variants are outside this witness. |

### 3.2 Coarse mandatory resource envelopes

These are G0 asymptotic rejection ceilings, not Family Lock A algorithms or
numeric thresholds. Let n be current live payload count and c current retained
payload capacity. M is the sum, for the witness being accounted, of the frozen
maximum slot counts of every imported W-POOL; for W-POOL itself it is that
pool's maximum. Every live, vacant, and permanently retired identity/history
slot is charged to M, never to peak live population. Let V/E be current graph
counts, U a frozen key universe, d pipeline depth, h recursive height, K_peak
the maximum simultaneously retained regular plus dedicated arena block-owner
count over one arena-owner lifetime, J the lifetime number of committed
dedicated-block acquisitions, and R the lifetime sum
of payload extents successfully committed in regular arena chunks, excluding
placement-start padding and unused chunk-tail bytes, and P the lifetime sum of
that excluded placement-start padding plus terminal unused bytes whenever a
regular chunk ceases accepting placements because of a nonfitting request or
the reset/release policy. At trace end P also charges every noncurrent regular
chunk's terminal bytes, so at most one current chunk remainder is omitted from
P and covered by the separate +C term. D_req is the lifetime sum of maximum
aligned request footprints for committed dedicated blocks. D_acq is their
lifetime sum of actual acquired usable bytes, including start-alignment
padding, allocator rounding, and unused dedicated tail; D_peak is the maximum
simultaneously retained portion of those actual bytes. Dedicated payload extent,
D_req, D_acq, and D_peak are reported separately. These variables define the
union envelope across two distinct homogeneous instantiations, not one mixed
arena. The regular instantiation reports D_req, D_acq, D_peak, and J as zero;
the oversized instantiation reports R and P plus regular-block commits as zero.
Family Lock A freezes finite
constants alpha and beta such that `D_acq <= alpha*D_req + beta*J` before
scoring; every in-flight dedicated acquisition also reports its actual usable
bytes, so neither retained nor transient slack is hidden. A is the lifetime number of successful
ownership-inserting graph operations, Q the lifetime number of successfully
created recursive nodes, and F the lifetime number of noncommitting backing-
allocation calls. F includes every call that reports failure and every
successful acquisition later rolled back or released without its enclosing
operation committing that acquisition. A successful acquisition is committing
if and only if a valid base or result owner retains it at that operation's
normal return; every other successful acquisition is noncommitting. Z is the
lifetime usable-byte total of
successful acquisitions counted in F; calls that report failure contribute
zero bytes to Z. A `_peak` suffix means the maximum over one owner lifetime, not
the final snapshot. Allocation ceilings below count committing successful
backing acquisitions; total attempted calls equal those committing calls plus
F, and both counts are reported. Every row also reports Z separately from
retained memory and committing acquisition bytes. Peak values, explicit
lifetime totals, F, and Z never reset after a failed retry,
pop, remove, clear, reset, shrink, or regrow. `O(...)` hides only lock-frozen
constant factors. A later lock must replace each applicable symbolic policy
with exact growth constants, allocator calls, targets, and measured counters
before scoring.

| ID | Persistent and retained memory | Peak/transient memory | Backing allocations | Separating traffic or code ceiling |
|---|---|---|---|---|
| W-POOL | O(M), including payload capacity, occupancy, generations, and permanently retired history; no term may grow with operation history beyond M. | O(M) plus O(1) operation-local state. | At most O(1 + log(1 + M)) committing successful backing growth acquisitions, or one committing successful preallocation; total attempts add F. No per-insert backing acquisition occurs after reserved capacity. | Across insert attempts and committing capacity growth, slot and metadata work is linear in attempts plus newly materialized capacity. A non-growing insert, get, replace, or remove touches O(1) slots and metadata; neither vacancy discovery nor identity-exhaustion detection may scan live or retired history. |
| W-ARENA | Regular-chunk retained and cumulatively committing-acquired usable bytes are at most R + P + C for lock-frozen usable chunk budget C; P charges the exact start-padding and retired/noncurrent terminal bytes defined above, while +C covers at most one current remainder. Current dedicated blocks add their actual acquired usable bytes, bounded by D_peak; their cumulative committing-acquired bytes add D_acq. Dedicated payload extent, D_req, D_acq, and D_peak remain separate. Regular/dedicated headers, sealed block-owner metadata, and the imported `FAM-DENSE` registry including retained capacity are O(K_peak), with no term growing with placement history beyond K_peak. All-attempt cumulative acquired bytes additionally charge Z. | Persistent bound plus O(C) regular-growth transient memory, the exact actual usable bytes of at most one in-flight dedicated acquisition, and O(K_peak) simultaneous old/new owner-token storage during registry growth. Dedicated alignment padding, allocator rounding, and unused tail are included rather than hidden. Registry relocation moves no payload bytes and no payload-proportional rollback copy is permitted. | Committing successful regular block acquisitions are at most `ceil((R+P)/C)+1`; J is exactly the number of committing successful dedicated acquisitions, one per successfully placed oversized request. The imported `FAM-DENSE` registry separately reports O(1 + log(1 + K_peak)) committing growth acquisitions or its stricter selected bound. Total block and registry attempts add F; Z includes every successful noncommitting acquisition from either route. No unclassified request may exceed an empty chunk's maximum aligned footprint. | Placement writes each complete payload once. Registry growth may move owner tokens but zero payload bytes or backing allocations. Reset/destruction visits each complete payload before releasing its retained block owner and visits each registry entry once. R, P, D_req, D_acq, D_peak, J, K_peak, regular/dedicated headers and slack, block commits, `FAM-DENSE` commits, F, and Z remain separately charged. |
| W-SMALL | O(N+c) retained bytes with c=O(N+n_peak); no spill-back is assumed after deletions. | O(N+c_peak) during spill/growth, including simultaneous old/new storage; no second persistent payload copy. | Zero committing successful heap acquisitions through N; thereafter O(1 + log(1 + max(0, n_peak-N))) committing successful growth acquisitions. Total attempts add F; no per-element allocation. | First spill relocates N live values once; each monotone growth phase has linear total relocation in its peak. Shrink/regrow traffic remains charged and cannot satisfy the cumulative allocation bound by resetting history. Exact constants freeze later. |
| W-RECUR | O(n_peak) payload/link/header capacity and any destruction scratch pre-reserved before commit. | O(n_peak+h_peak) only when already-owned auxiliary traversal storage is selected; machine call-stack use is O(1) in adversarial-depth destruction. | At most O(Q) committing successful backing acquisitions, with a lock-frozen constant per successfully created node covering node storage and any committed scratch growth, or fewer under a separately budgeted pool. Total attempts add F. Destruction performs no fallible allocation, and reservation failure returns every offered owner unchanged. | Construction/extraction touches O(1) local nodes per operation; destruction visits each live node once and never performs quadratic rescans. |
| W-GRAPH | O(V_peak+E_peak+M) retained payload, adjacency/link metadata, reverse relations, and stable-identity history. | O(V_peak+E_peak+M) plus O(local degree) node-removal result/repair state; no O(VE) table. | At most O(1 + A) committing successful node/edge backing acquisitions in the most fragmented admissible route, including calls through imported FAM-DENSE and W-POOL contracts; total attempts add F, and read traversal performs none. A selected route must report the stricter actual count. | Lookup/known-neighbor rewire touches O(1) records; node removal touches O(local degree); no global repair scan. |
| W-ECS | O(sum of peak archetype-column capacities + M) retained bytes. M charges the imported W-POOL payload, identity, reverse-location, and permanently retired history; this is not a peak-live-only bound. | Same bound plus the charged simultaneous old/new column growth transient required by the selected atomic route; no duplicate persistent archetype copy. | Across fixed archetypes/columns and entity reverse-location backing, at most the sum of O(1 + log(1 + peak_column_capacity)) committing successful lifetime growth acquisitions, plus the imported W-POOL's separately reported committing allocation bound. Total attempts add F, and there is no per-entity backing allocation. | Migration moves each affected component once; swap-removal moves one displaced row and repairs one reverse location; column scan is one contiguous pass. |
| W-GAP | O(c) with c=O(n_peak+1) under the later lock's bounded geometric growth policy. | O(c_peak) plus O(c_peak) only during growth; no permanent second sequence or per-slot tag required by this witness. | At most O(1 + log(1 + n_peak)) committing successful growth acquisitions; total attempts add F, and there is no per-element backing allocation. | Insert/delete at gap moves O(1); moving gap moves O(distance); each monotone growth phase has O(n_peak) relocation, while explicit shrink/regrow traffic remains charged. |
| W-PIPE | O(source state + d) live inline adapter state. | Same asymptotic bound; zero adapter heap allocations, per-element allocations, and intermediate collections. | No adapter backing allocation; only source/consumer contracts may allocate as separately budgeted. | One pass when the contract permits. Reachable specialized pipeline machinery is O(d + sum of retained callback-body sizes), rejecting exponential code growth in d; exact inlining/code-size limits freeze later. |
| H-FLATSET | O(c) contiguous payload backing plus O(1) metadata, with c=O(1+n_peak). | O(c_peak), plus one separately charged O(c_peak) old/new backing transient during growth and O(1) operation-local state. | At most O(1 + log(1+n_peak)) committing successful growth acquisitions under the later frozen policy; total attempts add F, successful noncommitting bytes add Z, and there is no per-element allocation. | Lookup and duplicate detection use O(log n) comparisons and zero payload moves. No-grow insertion and removal use O(log n) comparisons plus O(n-position) relocations. Growth relocation is charged separately; iteration, clear, and destruction are one O(n) pass. No second linear search or extra full rebuild is permitted. |
| H-STORE | Exhaustive persistent bound O(c+U): O(c) payload/key-position storage plus O(U) direct metadata, with c=O(n_peak). | O(c_peak+U) plus O(1) operation-local state, except O(c_peak) only during a separately charged backing growth transition. | At most O(1) committing successful metadata backing acquisitions with a lock-frozen constant independent of U, plus O(1 + log(1 + c_peak)) committing successful payload growth acquisitions. Total attempts add F; there is no per-element allocation. | Insert/remove/lookup touch O(1) positions; live iteration is one O(n) pass; swap-repair moves O(1) payloads. |
| H-LRU | O(C + M) = O(C) exhaustive backing memory for fixed positive capacity C, including hash, pool identity/history, links, keys, and values; every satisfying route must prove M = O(C). | O(C + M) after construction plus O(1) operation-local state; no second cache-sized transient in steady state. | At most O(1) committing successful preallocation backing acquisitions through imported FAM-UMAP and W-POOL contracts, with a lock-frozen constant independent of C; all allocation calls made by failed construction attempts are included in F. Successful steady-state operations allocate nothing, and there is no per-entry allocation. | Expected O(1) operations touch O(1) hash/link/pool records; no scan or payload clone. |
| H-IPQ | O(c) exhaustive heap, key, priority, and reverse-map backing memory with c=O(n_peak). | O(c_peak), including O(c_peak) only during charged growth, plus O(1) operation-local state. | At most O(1 + log(1 + n_peak)) committing successful growth acquisitions per dense/map backing family; total attempts add F, and there is no per-item allocation. | Peek O(1); modifying operations exchange and repair O(log n) positions, with no O(n) coherence repair or duplicate payload. |

Except for W-PIPE's separately stated depth-dependent bound, reachable
witness-specific machine code for each W or H contract must be O(1) in every
runtime size, capacity, key-universe, topology, and trace-length variable for
each frozen concrete payload and behavior instantiation. No capacity, element,
key, node, edge, or runtime state may induce specialization. Family Lock A
freezes exact IR and machine-code byte ceilings before scoring.

### 3.3 Visible controls and optional compositions

- **O-SLAB:** a bare reusable integer-key slab is a weaker performance/control
  contract. Its stale-key alias risk must be explicit. It cannot discharge
  W-POOL.
- **O-ROPE-UNIQUE:** a uniquely owned rope may be admitted later as composition
  over growable chunks and ordered recursive nodes. O(1) clone or persistent
  snapshots depend on the shared-ownership deferral and are not implied.
- **O-INTRUSIVE:** true pointer-intrusive, self-referential, or multiple-membership
  structures remain scoped to the pin/address family. W-GRAPH's handle-based
  known-neighbor rewiring is required now; it does not claim address pinning.
- **O-LAZY-DRAIN:** the lazy partially consumed form remains optional unless a
  family promotes it. Eager drain/compaction remains mandatory. The lazy form
  cannot inherit correctness from Rust's `Drop` guards without an xlang exit
  proof.

## 4. Exact held-out contracts

The semantic contracts and dependency budgets below are visible and frozen.
Candidate source, tests, traces, and performance observations remain excluded
until Candidate Freeze B. A held-out may not be used for candidate training,
manual tuning, compiler pattern recognition, or threshold selection.

### H-FLATSET — direct dense substrate: sorted affine flat set

Caller-visible contract:

- empty construction creates no `T`;
- payloads are arbitrary region-free, borrow-free affine `T`, including
  drop-bearing record values;
- at steady state all live payloads occupy one contiguous initialized prefix in
  ascending comparator order, with no per-slot occupancy tag, dummy value,
  hidden `Copy`/`Clone`/`Default`, per-element allocation, or second persistent
  payload copy;
- the behavior parameter is a stateless directly monomorphized
  `cmp(shared T, shared T) -> Ordering`; its borrows do not escape, and `Equal`
  defines duplicate equivalence;
- the frozen total-order law supports sortedness and uniqueness. A broken law
  may cause a contained logical error, but comparator results never authorize
  payload liveness, forge an index fact, lose ownership, or cause an invalid
  read or drop;
- `contains` is O(log n); `get` is O(log n) and returns `None` or an owner-tied
  shared borrow to the stored equivalent value;
- `insert(own T)` returns exactly `Inserted`, `Duplicate(own T)`, or
  `Failure(error, own T)`. The caller's input binding is dead once insertion
  begins; each nonsuccess result is the sole returned owner and leaves the set
  unchanged;
- search and every comparator call finish before capacity commitment,
  relocation, or opening `ST-HOLE`. Every comparison borrow ends before any
  owner moves, and no comparator or other fallible callback runs while a hole
  is live;
- recoverable capacity or allocation failure occurs before destructive
  commitment. After commitment, relocation, hole closure, initialization, and
  length update are infallible or carry an exact rollback proof;
- `remove(query)` completes its search before mutation, then returns `None`
  unchanged or `Some(own T)` and closes the suffix hole without cloning or
  dropping the removed value;
- clear and destruction drop every live payload exactly once and never visit
  spare or moved-from storage as `T`; and
- shared iteration yields every live value once in ascending order by
  owner-tied borrow. Abandonment leaves the set valid, owns no payload, and
  needs no repair; structural mutation is rejected while the iterator or a
  yielded borrow is live.

Dependency budget:

`K-SCALAR`, `ST-AOS`, `ST-DENSE`, `ST-HOLE`, `OW-INIT`, `OW-MOVEOUT`,
`OW-RELOCATE`, `OW-DROP`, `EX-NORMAL`, `EX-ABANDON`, `EX-ABORT`, `BR-PROV`,
`BR-REBORROW`, `BR-RESULT`, `BR-INVALIDATE`, `BR-CURSOR`, `FL-CAPACITY`,
`FL-ALLOC`, `FL-ATOMIC`, `FL-CALLBACK`, `AB-SEAL`, `AB-BEHAVIOR`,
`AB-GENERIC`, `IT-SHARED`, `FT-STATE`, and `F-ALLOC`.

The allowlist is exact. `ST-AOS`, `ST-DENSE`, and `ST-HOLE` are required
exercised dependencies rather than optional headroom. `FAM-DENSE`, a finished
sequence/set/map, `ST-SPARSE`, `OW-CLONE`, `AB-STATEFUL`, `ST-REFINE`, and
`FT-REFINE` are forbidden. Growth, insertion, removal, clear, and destruction
invalidate applicable live-prefix and result-borrow facts; comparator results
never substitute for `ST-DENSE` or `FT-STATE` liveness evidence. The held-out
rejects a second linear search, an extra full rebuild outside the separately
charged growth transition, comparator calls after commitment, and uncharged
payload traffic. Exact comparator counts and constants freeze only in Family
Lock A.

H-FLATSET is assigned to dense-family closure. It exercises the public dense,
AoS, ownership, failure, and traversal capabilities selected by that lock and
therefore cannot import a completed `FAM-DENSE` container.

### H-STORE — direct storage substrate: bounded-key sparse set

Caller-visible contract:

- creation fixes a key universe `[0, U)` but constructs no payload value;
- `insert(key, own T)` admits at most one live payload per key and returns
  exactly `Inserted`, `Duplicate(own T)`, or `Failure(error, own T)`. The
  caller's input binding is dead once the call begins; either nonsuccess result
  is the sole returned owner and leaves the set unchanged;
- for a key outside `[0, U)`, insert returns
  `Failure(OutOfUniverse, own T)` unchanged, `contains` is false, and `get` and
  `remove` return `None` unchanged;
- `contains` and shared `get` are O(1); the returned borrow is tied to the set
  owner and blocks incompatible mutation. `remove` is O(1) and returns exactly
  `None` unchanged or `Some(own T)` containing the sole removed owner;
- shared live iteration is O(n), visits each current key/value exactly once by
  owner-tied borrow, and makes no stable-order promise across removal;
- there is no per-element heap allocation;
- payload storage is O(capacity), key-position metadata is O(U), and operation
  traces freeze both terms separately; and
- removal, swap-repair, clear, destruction, and every failure path preserve the
  key-to-position/value relation and exact drop.

Allowed dependencies:

`K-SCALAR`, `ST-FULL`, `ST-DENSE`, `ST-SPARSE`, `ST-DEPENDENT`, `ST-HOLE`, `OW-INIT`,
`OW-MOVEOUT`, `OW-REPLACE`, `OW-SWAP`, `OW-RELOCATE`, `OW-DROP`,
`EX-NORMAL`, `EX-ABANDON`, `EX-ABORT`, `BR-PROV`, `BR-REBORROW`,
`BR-RESULT`, `BR-DISJOINT`, `BR-INVALIDATE`, `BR-CURSOR`, `FL-CAPACITY`,
`FL-ALLOC`, `FL-ATOMIC`, `AB-SEAL`, `AB-GENERIC`, `IT-SHARED`, `FT-STATE`,
and `F-ALLOC`.

Anti-privilege rejection criteria:

The allowlist above is exhaustive; every unlisted dependency is rejected
mechanically. The following examples are additional review canaries, not a
second negated dependency budget:

- any finished growable sequence, map, set, pool, slab, small-sequence, or ECS
  library;
- a container-specific intrinsic, compiler opcode, recognizable source-name
  rule, or sealed-standard-library raw payload access; and
- shared ownership, pinning, concurrency, custom allocators, or unsafe FFI.

H-STORE therefore tests public checked storage transitions directly. `F-ALLOC`
permits only its reviewed public checked allocation facade. It grants no raw
bytes, manual liveness authority, unchecked capacity change, allocator-identity
forgery, or manual deallocation, and H-STORE cannot rebuild or expose the frame.

`ST-SPARSE` is a required exercised dependency of H-STORE, not optional
allowlist headroom. A dense payload prefix plus fully initialized O(U) position
metadata is admissible, but it closes this witness only by establishing the
same public checked arbitrary-occupancy relation available to an unrelated
ordinary library. For owner/version v and an in-universe key k, a payload read
or drop requires the lock-equivalent proof that `p = position[k]`, `p < len`,
`dense_key[p] == k`, and `valid_T(dense_value[p])`. Get, iteration, and drop
consume that proof; insert, remove, swap-repair, clear, and destruction
atomically establish, preserve, invalidate, or discharge it. This permits the
classic dense/sparse-array representation and does not require a bitmap or
per-slot tag. `ST-FULL` plus `ST-DENSE` plus `ST-DEPENDENT`, a private H-STORE
sentinel/fact, a recognizable source name, or a finished pool/map/set does not
substitute for the public `ST-SPARSE` contract. `FAM-DENSE` remains outside the
held-out budget.

H-STORE is assigned to sparse-family closure after the dense family closes.
That predecessor makes adopted public dense capabilities available but does not
permit H-STORE to import `FAM-DENSE` or a finished sequence. H-STORE still
tests public sparse occupancy directly.

### H-LRU — composition under two-way coherence

Caller-visible contract:

- construction takes a fixed positive capacity, preallocates the complete
  steady-state backing budget, and either returns an empty cache or a
  recoverable allocation/capacity error with no live partial cache;
- lookup takes unique cache access; success promotes exactly that key to most-
  recently used, then returns a shared result-reborrow tied to the cache owner.
  The result blocks subsequent incompatible mutation until it ends;
- missing lookup returns `None` and leaves order unchanged;
- insertion of a new key below capacity consumes the offered key/value and
  returns `Inserted`;
- insertion of an equivalent existing key atomically replaces both stored key
  and value, promotes the entry, consumes the offered pair, and returns
  `Replaced(old_key, old_value)` with both old owners;
- insertion of a new key at capacity consumes the offered pair, reuses
  preallocated storage, promotes the new entry, and returns
  `Evicted(old_key, old_value)` for exactly the least-recently-used pair;
- removal returns `Some(owned_key, owned_value)` or `None` without mutation;
- no successful steady-state lookup/insert/update/remove performs a backing
  allocation; any recoverable pre-commit failure returns every offered affine
  owner and leaves membership, order, and payloads unchanged;
- expected O(1) get/insert/remove under the frozen hash/adversary assumptions;
- no per-operation scan of all entries; and
- hash membership, stable identity, linked order, payload ownership, and
  failure behavior remain coherent. Hash/equality traps follow EFF-4: no
  recoverable post-state is promised, but no invalid access occurs before
  abort.

Dependency budget: `K-SCALAR`, `FAM-UMAP`, `W-POOL`, `BR-PROV`,
`BR-REBORROW`, `BR-RESULT`, `BR-INVALIDATE`, `OW-MOVEOUT`, `OW-REPLACE`,
`OW-DROP`, `EX-NORMAL`, `EX-ABANDON`, `EX-ABORT`, `FL-CAPACITY`, `FL-ALLOC`,
`FL-ATOMIC`, `FL-CALLBACK`, `AB-BEHAVIOR`, `AB-SEAL`, and `AB-GENERIC`.
Only the frozen public APIs of `FAM-UMAP` and `W-POOL` are importable. H-LRU
may not receive new raw storage privilege or a compiler-recognized LRU path.
It is a composition witness and cannot substitute for H-STORE.

### H-IPQ — composition under heap/reverse-index coherence

Caller-visible contract:

- keyed items are unique;
- peek is O(1) and returns `None` or a pair of coexisting shared key/priority
  borrows tied to the queue;
- successful push consumes the offered owned key/priority pair; a duplicate
  returns `Duplicate(owned_key, owned_priority)` with the queue unchanged, and
  a recoverable capacity/allocation failure returns
  `Failure(error, owned_key, owned_priority)` with the queue unchanged;
- pop returns `None` or the owned minimum/maximum key/priority pair selected by
  the frozen order direction;
- keyed removal returns `None` or the owned matching key/priority pair;
- priority change returns `Missing(owned_new_priority)` without mutation, or
  consumes the new priority and returns `Changed(owned_old_priority)` while
  retaining the stored key owner;
- push, pop, keyed removal, and priority change are O(log n);
- every heap exchange atomically repairs the keyed reverse position;
- ownership and failure behavior are exact for affine item/priority payloads;
- comparison/hash/equality traps follow EFF-4 and perform no invalid access
  before abort; recoverable allocation failure occurs before destructive
  commit; and
- no operation repairs coherence by an O(n) search.

Dependency budget: `K-SCALAR`, `FAM-UMAP`, `FAM-DENSE`,
`BR-PROV`, `BR-REBORROW`, `BR-RESULT`, `BR-DISJOINT`, `BR-INVALIDATE`,
`OW-INIT`, `OW-MOVEOUT`, `OW-REPLACE`, `OW-SWAP`, `OW-DROP`, `EX-NORMAL`,
`EX-ABANDON`, `EX-ABORT`, `FL-CAPACITY`, `FL-ALLOC`, `FL-ATOMIC`,
`FL-CALLBACK`, `AB-BEHAVIOR`, `AB-SEAL`, and `AB-GENERIC`. Only the frozen
public APIs of `FAM-UMAP` and `FAM-DENSE` are importable; the held-out itself
implements heap order and reverse-index repair through those APIs. H-IPQ
receives no finished heap, bespoke compiler path, or exchange hook and cannot
substitute for H-STORE.

## 5. Ordinary-library generativity gate

Every W and H witness must satisfy all of the following:

1. Compile as an ordinary external xlang library with the same compiler
   artifact and public capability set used for unrelated programs.
2. Import only its frozen dependency allowlist. No privileged module, hidden
   intrinsic, container-specific opcode, unchecked payload access, or
   standard-library-only transition is legal.
3. Pass negative canaries for inaccessible-state reads, duplicate/drop loss,
   interrupted transitions, stale identity, overlapping mutable places,
   partial construction, and all applicable failure points.
4. Meet frozen asymptotics before timing, then report allocation count,
   initialized/touched/moved bytes, live/high-water/transient memory, metadata,
   checks, branches, code size, and failure injection where applicable.
5. Demonstrate that facts-off changes only retained checks/performance, never
   acceptance or semantics.
6. Reject candidate-specific compiler recognition even if the result is fast.

A standard-library implementation may serve as a reviewed witness of API
sealing, but it cannot alone close W or H. This gate is the difference between
"xlang ships useful containers" and "ordinary libraries can build an unseen
efficient structure."

## 6. Holdout custody and rotation

- G0-Core freezes the contracts, budgets, and exclusion rules above.
- Family Lock A freezes exact test-oracle schemas, payload classes, targets,
  trace families, endpoints, and the custodian responsible for source/tests.
- Before candidate construction, the custodian records hashes of the hidden
  source, tests, trace generator, and sealed inputs outside candidate-visible
  material. Candidate agents receive only the visible contract and allowlist.
- Candidate Freeze B records candidate hashes first; only then may the held-out
  implementation and immutable scoring inputs be disclosed to the scoring
  process.
- Any leak, candidate-specific correction, compiler recognition, or post-freeze
  semantic/gating change compromises the held-out. The family lock reopens and
  the custodian rotates to a new implementation within the same frozen contract
  and dependency budget. The compromised result is diagnostic only.
- H-FLATSET, H-STORE, H-LRU, and H-IPQ are logically independent. One passing held-out
  cannot compensate for another failure.

No hidden artifact is created by G0-Core; custody begins only under a separately
authorized Family Lock A. This avoids false claims that a plaintext repository
fixture is training-excluded.

## 7. Primary evidence

- Rust's intentionally bounded standard collection set:
  <https://doc.rust-lang.org/1.97.0/std/collections/>
- Generational identity and exhaustion pressure:
  <https://docs.rs/generational-arena/latest/generational_arena/>
- Bare slab key reuse:
  <https://docs.rs/slab/latest/slab/>
- Phase/bump arena behavior:
  <https://docs.rs/bumpalo/latest/bumpalo/>
- Inline-to-heap sequence transition:
  <https://docs.rs/smallvec/latest/smallvec/>
- Stable graph mutation:
  <https://docs.rs/petgraph/latest/petgraph/stable_graph/struct.StableGraph.html>
- ECS table/archetype storage:
  <https://docs.rs/bevy_ecs/latest/bevy_ecs/storage/> and
  <https://docs.rs/bevy/latest/bevy/ecs/archetype/>
- Movable gap behavior:
  <https://www.gnu.org/software/emacs/manual/html_node/elisp/Buffer-Gap.html>
- LRU contract:
  <https://docs.rs/lru/latest/lru/struct.LruCache.html>
- Indexed priority queue contract and costs:
  <https://docs.rs/priority-queue/latest/priority_queue/priority_queue/struct.PriorityQueue.html>
- Address-sensitive/intrusive motivation:
  <https://doc.rust-lang.org/1.97.0/std/pin/> and
  <https://docs.rs/intrusive-collections/latest/intrusive_collections/>
- Rope composition and sharing distinction:
  <https://docs.rs/ropey/latest/ropey/struct.Rope.html> and
  <https://research.google/pubs/ropes-an-alternative-to-strings/>
