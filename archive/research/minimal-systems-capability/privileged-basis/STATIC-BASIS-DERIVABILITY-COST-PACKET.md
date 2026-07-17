# Static-basis derivability and structural-cost packet

Status: D14 paper construction and cost accounting, 2026-07-15. Assumptions:
the research-recommended sealed compiler-embedded registry and abstract basis in
`MINIMAL-STATIC-PUBLIC-CAPABILITY-BASIS.md`. This packet reports no exact D-2 or
P-1 closure, benchmark, implementation, production selection, or standard
library.

## Result

The proposed basis gives ordinary-library paper routes for the complete
sequential witness set and category-level routes for shared ownership,
synchronization, channels, and async state machines. It also identifies exact
registry-leaf gaps for OS, FFI, target, device, and dynamic-code domains.

The strongest honest status is:

- **49/49 capability obligations accounted:** every row maps to an existing
  rule, a P/Q basis route, or an explicit open model/fact review in
  `MINIMAL-STATIC-PUBLIC-CAPABILITY-CROSSWALK.tsv`;
- **26/26 systems domains routed:** every domain below terminates as current,
  ordinary library, paper route, exact-leaf gap, deferred, redundant, or
  non-goal;
- **all registered topology witnesses routed at paper level:** no named
  container receives privilege;
- **four independent later held-outs remain deliberately open:** concurrent
  cuckoo migration needs Q5, crash-consistent LSM needs exact filesystem and
  persistence events, hot-swappable JIT needs P9 final-image validation, and
  sparse GPU residency needs exact P8/P7 device/DMA/reset rows;
- **dense exactness remains open:** 340 route obligations across 150 contexts,
  including Convert, allocator, and ZST/fullness cases, receive no positive D-2
  credit; and
- **performance remains structural only:** no candidate was built or measured,
  so P-1 remains fail-closed.

## Constructive ordinary-library routes

### Storage, collections, and traversal

| Demand | Ordinary construction over the basis | Time and allocation topology | Layout, traffic, metadata, and indirection | Cleanup, failure, and facts | Status |
|---|---|---|---|---|---|
| Fixed arrays and Copy buffers | Existing full storage; Q1 may adopt it without payload action | Existing O(1) indexing; one existing allocation for buffer | Native array/two-word buffer; no capacity, tag, generation, or new branch | Existing drops/checks; no new fact required | Protected current route |
| Dynamic sequence / byte builder | Q1 seals one P1 carrier with live prefix and vacant suffix; P2 implements push/pop; P3 implements reserve-first growth | O(1) access, amortized O(1) push, O(n) ordered insert/remove; O(log n_peak) geometric growth acquisitions | Contiguous payload; pointer/length/capacity only; no zero fill, spare `T`, per-slot tag, clone, or indirection | Failed P3 growth leaves owner/offered value unchanged; Q2 drops live prefix; state/bounds facts invalidate on growth | Paper route; dense D-2/P-1 open |
| Deque / ring | Q1 represents head, length, and at most two live intervals; P2 moves boundary places | O(1) ends/access; O(n) make-contiguous; geometric backing growth | One contiguous backing; head/length/capacity; no sparse bitmap or per-element allocation | Exact two-interval drop; failed growth unchanged; wrapped-range facts invalidate on rebalance | Paper route |
| String / UTF-8 builder | Dynamic byte sequence plus Q6 validation/refinement; edits operate on byte boundaries | Byte operations retain their bounds; validation O(n) or incremental | Same contiguous bytes; no text metadata on raw byte owner; no scalar-index promise | Mutation invalidates refinement; validation failure preserves bytes or returns exact error | Paper route; full Unicode deferred |
| Open-addressed hash map/set | Q1 relates control bytes/occupancy to P1/P2 payload places; Q4 supplies hash/equality; P3 reserve-first rehash | Expected O(1) lookup/insert/remove; geometric table growth; no per-entry allocation | Representation-selected control bytes plus contiguous keys/values; no universal tag beyond hash design; direct monomorphized calls | Duplicate/recoverable failure returns offered owners; exact live-slot drop; occupancy/probe facts invalidate on mutation | Paper route; exact hash family proof open |
| Flat ordered set / binary heap | Dense sequence plus Q4 comparator and Q3 disjoint swap | O(log n) search/heap repair; O(n) contiguous insertion shift; geometric growth | Contiguous payload and O(1) metadata; no tree nodes or per-item allocation | P2 take/put relocation; comparator outcomes cannot weaken safety; exact live-prefix drop | Paper route; H-FLATSET structural bounds retained |
| B-tree / rope | P1 heterogeneous/record places; Q1 seals node occupancy/child ranges; P2 split/merge; P3 node roots | O(log n) search/update; O(number of touched nodes) allocations/moves | Native nodes with compact counts/links; no universal bitmap; one allocation per selected node policy or pool | Reserve required nodes before commit; Q2 recursive/iterative disposition; node facts invalidate locally | Paper route; exact ordered-family lock open |
| Recyclable pool / slab | Q1 sparse places plus generation/retirement metadata; P2 insert/remove; P3 backing growth | O(1) lookup/update; amortized O(1) insertion; history O(max slots), not live count | Payload, occupancy/free list, generations; no checks on append-only P2 path | Exhaustion/alloc failure returns `own T`; stale handle never revives; exact occupied drop | Paper route; W-POOL budget retained |
| Graph / linked structure | Pools for nodes/edges; Q1 cross-index invariants; Q3 scoped multi-place repair; dense results for removed owners | O(1) handle lookup/known-neighbor repair; O(local degree) node removal; CSR O(V+E) | O(V+E+identity history); no global repair table or raw pointer; unique linked nodes may use P3 roots | Preallocate removal result before commit; return every payload once; invalidate affected handles/cursors | Paper route; no raw-intrusive credit |
| Arena | Q1 owns a dense registry of P3 block roots; P1 projects fresh places; P2 installs complete values; Q3 proves new writes disjoint from prior borrows | Amortized O(1) regular placement; block acquisition by chunk policy; no per-item allocation | Chunk/dedicated-block slack plus O(number of blocks) owner registry; registry moves tokens, not payloads | Failure returns offered `T` and releases uncommitted block; reset drops values before roots; placement facts root in backing block | Paper route; W-ARENA cost ledger retained |
| Inline-small sequence | P1 creates tag-free inline vacant places; Q1 switches between inline and heap representation; P2 moves owners; P3 first spill | Zero heap acquisitions through N; one spill at N+1; later geometric growth | Inline payload plus discriminant/length and optional heap owner; no inline liveness tags, second persistent copy, or per-item allocation | Failed spill returns offered owner and leaves inline state; exact active-representation drop | Paper route; W-SMALL bound retained |
| Gap buffer | Q1 owns live prefix/suffix and one vacant gap; P2 moves boundary elements | Amortized O(1) edit at gap; O(distance) move; geometric growth | One contiguous backing, two bounds; no per-slot tags or second sequence | Failed growth unchanged; destroy only prefix/suffix; gap state facts move with bounds | Paper route; W-GAP bound retained |
| ECS columns | Dense Q1 owners per column plus pool/reverse-location invariant; P2 migration/swap repair; P3 column growth | O(1) identity lookup; O(columns) migration; contiguous O(n) scans; no per-entity allocation | SoA columns, reverse map, identity history; no duplicate persistent archetype | Prepare all growth before commit; each component moves once; affected facts invalidate together | Paper route; W-ECS exact lock open |
| Shared/unique/owning cursors and drain | Q1 cursor state, Q2 exact abandonment/disposition, Q3 provenance/progression/disjointness, Q4 behavior; P2 owning yields | O(1) per yield; one pass; zero intermediate collections and adapter allocations on static path | O(adapter depth) inline state; direct monomorphized callbacks; no borrow table or per-element allocation | Early stop/error disposes remaining owners once; yielded leaves retain exact roots; cursor invalidation ends future authority | Paper route; BR-STORED variants remain open |
| Lazy pipeline | Nested Q4 stateful callables and Q3 source maps inside Q1/Q2 cursor protocols | One pass when contract permits; no adapter heap/per-element allocations; code size O(depth + callback bodies) | Inline state and direct calls; no materialization or dynamic dispatch unless explicitly selected | Exact call counts/order and early-exit ownership; advisory hints mint no access facts | Paper route; W-PIPE region-free/borrow-free only |

The WITNESS-REGISTRY exact persistent/peak memory, allocation-attempt,
committing-acquisition, payload-traffic, and code-size budgets remain controlling
for W-POOL, W-ARENA, W-SMALL, W-RECUR, W-GRAPH, W-ECS, W-GAP, W-PIPE,
H-FLATSET, H-STORE, H-LRU, and H-IPQ. This packet changes their route vocabulary
only; it does not relax a bound.

### Sharing, concurrency, and async

| Demand | Ordinary construction | Structural costs | Gap/status |
|---|---|---|---|
| Strong/weak shared owner | Q1/Q2 lifecycle state over P5 counts; P2 destroys payload at last strong and root at last weak; Q5 proves ordering/reclamation | Two selected counters and atomic traffic only on shared-owner route; unique owners pay none | Category paper route; Q5 weak-memory model open |
| Dynamic borrow cell | Q1 state plus checked shared-count/exclusive transitions; guards are Q2/Q3 owners; P5 only for cross-thread form | One borrow flag/count and guard traffic only for dynamic-borrow type; lexical borrow path unchanged | Category paper route; leak/abandon policy open |
| Mutex/RW lock/condition variable | P5 atomic state and wait/notify, P6 park/unpark, Q5 protected-data invariant | Lock word and contention runtime events; no tax on unrelated data | Category paper route; fairness/progress/poison policy open |
| Channel | Q1 queue/ring plus P5 publication and P6 wakeup; Q2 disconnect/drop | Selected queue metadata, atomics, and wake events; bounded/unbounded allocation policy ordinary | Category paper route; Q5 and exact progress open |
| Lock-free structure | Q1 resources, P5 CAS/fences, Q5 linearization and reclamation protocol | Algorithm-selected atomics/reclamation records; no global hazard tax | Open: memory model and safe reclamation not selected |
| Future/task/executor | Ordinary affine state machines and Q4 callables; P5/P6 wake/park; exact P7 timer/I/O events; P4 for retained buffers | Inline future state when statically known; scheduler queue/allocations only by selected executor; no privileged executor | Category paper route; P7 event inventory and cancellation proof open |

### OS, FFI, target, loader, and device routes

| Demand | Ordinary layer | Irreducible rows | Structural/event accounting | Status |
|---|---|---|---|---|
| File and buffered byte I/O | Buffering, retry, parsing, seek policy, and formatting are ordinary | Exact P7 open/read/write/seek/stat/close rows as required | One OS event per executed primitive call, exact partial byte count; no hidden retry; P4 only if provider retains buffers | Category need established; exact rows open |
| Socket/network I/O | Address parsing and protocol policy ordinary | Exact P7 socket/bind/connect/listen/accept/send/recv/shutdown/close and resolver rows | Exact syscall/event multiplicity, partial progress, blocking/cancel behavior; resource owner metadata only | Category need established; exact rows open |
| Process/environment | Argument/env construction and pipe orchestration ordinary | Exact P7 environment read, spawn, wait, signal/terminate, pipe and close rows | One event per row invocation; child/pipe owners explicit; no hidden shell or retry policy | Category need established; exact rows open |
| Clocks/timers | Duration/deadline arithmetic and timer wheel ordinary | Exact P7 monotonic clock, wall clock, sleep/timer-arm/cancel rows | Clock/timer events explicit; resolution/suspension semantics per row | Category need established; exact rows open |
| Outbound FFI | Marshaling wrappers ordinary over checked values | One P7 row per exact foreign function/frame; P4 for retained/pinned buffers | ABI transition and copies/pins explicitly charged; fact attenuation conservative | Narrow outbound category only; row inventory open |
| SIMD/target operation | Portable fallback and dispatch policy ordinary | One P8 row per exact operation/feature contract | Direct instruction sequence or specified fallback; target dispatch only when selected | Category need established; exact rows open |
| Virtual memory and JIT | Code-cache policy and eviction ordinary | Exact P8 map/protect/unmap/cache-flush rows plus P9 validate/activate/call/unload | Mapping/protection/flush/activation events explicit; static code pays none | Held-out open: P9 final-image proof unselected |
| MMIO | Driver protocol ordinary over opaque mapped resource | Exact P8 mapping and read/write event rows with widths/order/fault rules; P4 lifetime | Exact device-event count; no speculation, fusion, widening, duplication, or reordering beyond row contract | Held-out open: device model/rows unselected |
| DMA/GPU residency | Residency policy, sparse maps, queues, and recovery ordinary | Exact P7/P8 allocate/map/pin/submit/fence/reset/unmap rows; P4 leases | Every DMA submission/fence/reset and retained extent charged; CPU/GPU ownership epochs explicit | Held-out open: reset and external mutation semantics unselected |

## Twenty-six-domain closure map

| Domain | Route under the candidate basis | Honest status |
|---|---|---|
| D01 primitive/numeric/aggregate | Current scalar/checked operations plus P1/P2 aggregate place work; full math remains later | Current plus paper extension |
| D02 callable/core language | Q2/Q4; dynamic code additionally P9 | Paper route; source organization later |
| D03 generic behavior/borrowing | Q3/Q4; exact stored-borrow branches remain open | Paper route with exact gaps |
| D04 layout/memory/provenance | P1/P2/P3/P4; raw writer pointers remain inadmissible | Paper route; target layouts unimplemented |
| D05 Option/Result/errors | Current Option/Result and Q2 failure closure | Current/paper extension |
| D06 traps/diagnostics | Current abort trap; reporting is exact P7 runtime row | Current plus exact-leaf gap |
| D07 unique heap/allocation | P1/P2/P3 and ordinary allocator policy | Paper route |
| D08 shared/interior mutation | P5 plus Q1/Q2/Q3/Q5 | Open Q5 model |
| D09 sequential collections | P1/P2/P3 plus Q1-Q4/Q6 ordinary libraries | Paper-routed witnesses; D-2/P-1 open |
| D10 synchronous iteration | Q2/Q3/Q4 and P2 for owning yields | Paper route; stored-borrow variants open |
| D11 bytes/text | Dense bytes plus Q6 | Paper route; complete Unicode later |
| D12 formatting | Ordinary Q4 formatting over builders; output P7 | Library route after dependencies |
| D13 reflection/type identity | No basis need established | Deferred |
| D14 build/source metadata | Compiler immutable facts and exact build rows; package system later | Deferred/exact-leaf gap |
| D15 I/O | Ordinary buffering over exact P7 byte events | Exact-leaf gap |
| D16 filesystem/path | Pure paths ordinary; filesystem exact P7 rows | Exact-leaf gap |
| D17 environment/process | Ordinary construction over exact P7 rows | Exact-leaf gap |
| D18 FFI/OS handles | Exact P7 rows plus P4 resource leases | Exact-leaf gap; outbound only |
| D19 networking | Ordinary protocols over exact P7 rows | Exact-leaf gap |
| D20 clocks/timers | Pure arithmetic ordinary; exact P7 events | Exact-leaf gap |
| D21 threads/TLS | P6 plus Q5 | Open model/runtime rows |
| D22 atomics/synchronization | P5/P6 plus Q5 ordinary protocols | Open memory/reclamation model |
| D23 futures/tasks/pinning | Ordinary state machines over P4-P7/P9 as selected | Category paper route; exact leaves open |
| D24 target/MMIO | Exact P8 rows and P4 leases | Exact-leaf gap |
| D25 prelude/reexports | No semantic capability | Redundant |
| D26 compiler/runtime support | Static registry binds each P row to exact implementation and target | Toolchain obligation, not public primitive |

## Independent held-out re-adjudication

### Concurrent cuckoo index with incremental resizing

Two Q1 tables, P1/P2 slots, P3 growth, P5 atomics, and Q5 invariants can express
incremental bucket migration without a universal topology or privileged map.
Expected O(1) lookup/update and bounded migration work per operation are
structurally plausible; old/new tables and migration metadata are explicit.
No positive result is claimed because Q5's memory model, linearization, and
reclamation are unselected.

### Crash-consistent LSM storage engine

Memtables, immutable runs, bloom filters, manifests, and compaction policy are
ordinary P1-P3/Q1-Q4 structures. Correct durability needs exact P7 write,
sync, rename/replace, directory-sync, mapping, and crash-observation contracts.
Without those rows, source-level ownership cannot prove post-crash state. The
held-out remains an exact external-event gap, not evidence for a privileged LSM.

### Hot-swappable JIT code cache

Ordinary maps/eviction policy can own P3 code buffers and P4 leases, but safe
publication, target validation, relocation/import binding, W^X transition,
instruction-cache synchronization, call lifetime, and unload require exact P8
rows and P9. The held-out remains open until final-image validation is fixed.

### Sparse GPU residency with DMA and reset

Sparse residency maps and queue policy are ordinary Q1 structures. P4 leases
and exact P7/P8 DMA, mapping, submission, fence, invalidation, and reset events
are irreducible. Device reset and externally mutable memory must attenuate every
CPU fact. The held-out remains open until those event rows and ownership epochs
are selected.

## Structural-cost conclusions

The paper routes avoid unavoidable asymptotic regression and the specifically
forbidden simulations:

- no full-capacity initialization or zero fill for vacant affine storage;
- no hidden `Copy`, `Clone`, or `Default`;
- no universal liveness bitmap/tag, refcount, stability lease, borrow flag,
  dynamic dispatch object, or target mode;
- no per-element allocation for dense, ring, flat, pool, ECS, gap, or iterator
  routes;
- no whole-buffer copy for pop/remove/relocation;
- no intermediate collection for a static lazy pipeline; and
- no generic syscall/opcode/contract descriptor.

These are contract-level structural claims only. Constants, instruction count,
code size, compile time, proof size, optimizer quality, allocator rounding,
cache behavior, contention, and throughput remain unmeasured.

## Exact fail-closed ledger

The prior dense exact generator remains authoritative for unresolved work:

- 340 missing required route obligations;
- 150 affected contexts;
- 208 Convert-related obligations;
- 136 allocator-related obligations;
- 12 ZST/fullness obligations; and
- 16 obligations in both Convert and allocator sets.

This packet neither edits those generated artifacts nor converts category
routes into D-2 credit. `BR-STORED`, Q5, P7/P8 inventories, and P9 remain
additional explicit gaps.

## Smallest separately authorizable experiments

No experiment is authorized by this packet. If the owner accepts the abstract
basis for further research, the minimum useful sequence is:

1. **Deterministic safety-model slice:** formalize P1/P2 plus Q1/Q2 for only a
   dense prefix, two-interval gap, and representation transition; mutation-test
   abandonment, dead reads, duplicate owners, wrong drops, and forged state.
   This is a soundness experiment, not a benchmark.
2. **Structural no-tax slice:** in a detached prototype, lower fixed full
   buffer, dense affine sequence, and inline-small spill; compare layout,
   allocation calls, initialized/zeroed bytes, moves, drops, branches, checks,
   and optimized bodies against frozen native baselines. Runtime timing is not
   needed until structural parity passes.
3. **Exact external-row slice:** specify one partial byte-read row with P4
   retention forbidden, one retained-buffer asynchronous row, and one close
   row; adversarially test partial progress, interruption, cancellation,
   lifetime, and fact attenuation. No broad FFI or OS catalog is needed first.

Q5 concurrency, P9 JIT, MMIO, and GPU work should not begin before these
smaller sequential and external-boundary slices expose whether the basis needs
revision.
