# Scenario Demand Map: Systems Programming at Top-Tier Performance

Date: 2026-07-16

Status: research coverage target under D15. This map records what
performance-critical systems software does and why the fast implementations
are fast. It is the target that any capability/form catalog must cover; it is
not a normative language document. Produced by three independent mappers from
real-software knowledge; raw output in `evidence/scenario-map.json`.

Reading guide: "Par target" names the reference implementation and the rough
number a blessed Whitefoot writing must match. "Substitutability" answers the
owner's central question: can ONE blessed shape serve all uses of the
scenario at par, or do uses genuinely split.


## F1 — sequential owned storage

### 1. Growable contiguous vector (append-heavy owned buffer)

- **Where**: The single most-used structure in systems code: compiler IR op lists (LLVM, rustc SoA tapes), database row/page buffers (SQLite record assembly), network packet assembly, simdjson tape output, game engine component arrays. Essentially 100% of nontrivial programs.
- **Contract**: Owns a contiguous, growable run of T. push (amortized O(1)), pop, indexed read/write, reserve, truncate, in-order iteration yielding &T/&uniq T, move-out of the whole buffer, pass-as-slice to any function taking a contiguous view. Failure: growth may allocate (OOM abort or fallible reserve); out-of-bounds is checked.
- **SOTA shape**: Rust std::vec::Vec / C++ std::vector / folly::fbvector: (ptr, len, cap) triple, geometric growth (2x or 1.5x), realloc-aware growth path, no per-element allocation.
- **Why fast**: Sequential append touches exactly the tail cache line (1 new line per 64/sizeof(T) pushes); the capacity check is one predicted branch (mispredict only at growth, ~1 in n); growth is a bulk memcpy running at DRAM bandwidth (~25-35 GB/s/core); iteration is a stride-1 hardware-prefetched loop that auto-vectorizes. Zero pointer indirection per element.
- **Par target**: Rust std Vec (1.80): ~1 ns amortized per u64 push including growth; iteration/sum of 1M u64 at ~1 element/cycle vectorized. Any blessed form must match Vec push/iterate within noise on a 1M-element append-then-scan workload.
- **Weight**: high. **Substitutability**: Highest substitutability in the whole map: ONE blessed vector can serve virtually all uses at par, PROVIDED it exposes a slice view so algorithms are written against the view, not the container. The one real split is inline capacity (see small vector) — unifiable as a single shape with a compile-time inline-capacity parameter defaulting to 0.

### 2. Small/inline vector (allocation-count-dominated short sequences)

- **Where**: Compilers above all: LLVM uses SmallVector in tens of thousands of sites (operand lists, worklists, successor lists where n is usually 2-8); also scatter/gather iovec arrays, per-request header lists in servers, small polygon/child lists in engines.
- **Contract**: Same contract as the growable vector (push/index/iterate/slice view), plus: no heap allocation while len <= N; spills transparently to heap beyond N; value can be moved cheaply enough that spill state is invisible to the caller.
- **SOTA shape**: llvm::SmallVector<T,N> / rust smallvec 1.x / absl::InlinedVector: inline storage of N elements in the object footprint, heap spill path, capacity tag distinguishing inline vs spilled.
- **Why fast**: Eliminates a malloc+free pair per short-lived sequence (~30-60 ns round trip even under mimalloc/jemalloc fast paths, plus the cache miss of touching allocator metadata); keeps the elements on the same cache line(s) as the owner, so a 4-element operand list is 1 line total instead of 2 lines + allocator traffic. In allocation-dominated compiler workloads this is a 2-5x win on the affected code, not a percent-level tweak.
- **Par target**: llvm::SmallVector<T,4> vs std::vector on LLVM-style build/discard of millions of 2-6 element lists: SmallVector ~3-5x faster (allocation elimination dominates). Reference: smallvec 1.13 bench push-4-then-drop ~4 ns vs Vec ~40 ns.
- **Weight**: high. **Substitutability**: Cleanly unifiable with the growable vector as one parameterized shape (Vec<T, const INLINE: usize>), which is exactly what absl::InlinedVector proves. It should NOT be a separate blessed form; it is a knob on the vector. Only residual split: types whose inline copy on spill is expensive — negligible in practice.

### 3. String/byte builder (append-only formatting and serialization buffer)

- **Where**: Every server (HTTP response assembly), serializers (serde_json, protobuf encoders), SQL text generation in SQLite, compiler diagnostics and codegen emission, log formatting. Ubiquitous.
- **Contract**: Append bytes/str/formatted numbers to a growing owned byte buffer; single final move-out as an immutable string/bytes; reserve-ahead; no intermediate allocations per append; write-through interface so third-party formatters can target it.
- **SOTA shape**: Rust String/Vec<u8> + itoa/ryu for numeric formatting; folly::IOBuf-style for scattered output. Core shape is just the vector specialized to bytes with a formatting write trait.
- **Why fast**: Appends are memcpy into reserved tail (SIMD copy, no length re-checks per byte); integer formatting via itoa-style lookup tables is ~3-6 ns per u64 vs ~100 ns for snprintf (no locale, no format-string parse, no branching per digit); one reservation replaces N reallocations; the buffer is written exactly once and read exactly once — perfect streaming access pattern.
- **Par target**: Rust String + itoa 1.0 / ryu 1.0: u64 -> decimal ~4 ns, f64 -> shortest ~20 ns; bulk str append at memcpy bandwidth. A blessed builder must match itoa-class numeric emission and memcpy-class byte append; snprintf-class (20x slower on numbers) is a fail.
- **Weight**: high. **Substitutability**: Yes — one shape. It IS the byte vector plus a blessed formatting-write protocol; it needs no independent container. The only genuine split is scattered/zero-copy output (IOBuf chains for network stacks that splice buffers), which is a different scenario (buffer chains), not a variant of the builder.

### 4. Sorting owned runs (comparison and key-based)

- **Where**: Databases (SQLite ORDER BY, external merge runs), compilers (relocation/symbol sorting, deterministic output ordering), analytics columnar engines, graphics (draw-call/depth sorting per frame), dedup pipelines.
- **Contract**: Sort a contiguous slice in place by a comparator or extracted key; unstable acceptable for most uses, stable required when equal-key order is meaningful (database ties, deterministic compiler output); O(n log n) worst case guaranteed (no quicksort blowup); sorts of already-sorted/mostly-sorted input near O(n).
- **SOTA shape**: pdqsort lineage: Rust 1.81 sort_unstable = ipnsort, sort = driftsort; pattern-defeating branchless partitioning, insertion sort at small n, run detection. For u32/u64 keys at large n: LSB/MSB radix (ska_sort, voracious) beats comparison sort 2-4x.
- **Why fast**: Branchless partitioning converts the data-dependent compare branch (near-50% mispredict on random data, ~15-20 cycles each) into cmov/offset writes; small-n insertion sort keeps the working set in L1; run detection makes sorted input O(n) with sequential access only; radix replaces log n passes of unpredictable compares with ~4-8 predictable counting passes at streaming bandwidth.
- **Par target**: Rust 1.81 ipnsort: ~55-80M u64/s on random 1M-element input (single core); driftsort within ~10% for stable; voracious radix ~2.5x ipnsort at 10M u64 keys. Blessed sort must hit ipnsort-class on random u64 and near-O(n) on sorted input.
- **Weight**: medium. **Substitutability**: Mostly one form: a single adaptive sort entry point can internally dispatch (ipnsort already fuses insertion/heap/pdq; radix specialization for primitive keys can be a compiler-known fast path inside the same blessed call). The stable/unstable split is real semantics, not performance — two spellings or one flag, not two shapes. Verdict: one blessed sort with a stability choice reaches par everywhere that matters.

### 5. Binary search / lookup over sorted contiguous runs

- **Where**: SQLite b-tree page-interior key search, IP routing/longest-prefix tables, symbol tables frozen after load, time-series timestamp lookup, unicode property tables (utf8/regex engines), jump tables in interpreters.
- **Contract**: Given a sorted slice, find key or insertion point (partition_point); read-mostly or fully frozen data; caller controls layout since data is just a slice; predictable worst case; no allocation.
- **SOTA shape**: Branchless binary search (cmov-based, as in Rust slice::binary_search post-1.25 shape); for hot huge read-only tables, Eytzinger (BFS) layout with explicit prefetch (Khuong & Morin, 'Array layouts for comparison-based searching').
- **Why fast**: Branchless probe replaces an unpredictable compare branch per level (log2 n mispredicts, ~15-20 cycles each) with cmov; cost collapses to the chain of cache misses — log2(n) - log2(64/sizeof(K)) line touches; Eytzinger makes the first ~4 levels share lines and prefetchable, cutting effective misses ~2x on DRAM-resident tables.
- **Par target**: Branchless search on 1M u64: ~40-60 ns/lookup DRAM-resident (Khuong-Morin numbers); Eytzinger ~2x better at 100M+ elements. Blessed form must match branchless std binary_search; Eytzinger is an optional layout, same algorithm.
- **Weight**: medium. **Substitutability**: One blessed branchless search over slices covers ~95% of uses at par because the input is just the vector's slice view. Eytzinger is a genuine but narrow split (giant frozen hot tables); it can be a taught layout transform over the same search shape rather than a new container.


## F2 — associative lookup

### 6. General hash map/set (unordered point lookup, medium-to-large n)

- **Where**: Everywhere: compiler symbol/type tables, database hash joins and page caches, server session/routing tables, deduplication, graph visited sets. After the vector, the second most demanded structure in systems code.
- **Contract**: insert/lookup/remove by key with owned K,V; get returns &V/&uniq V; entry-style get-or-insert (one hash, one probe for the read-modify-write pattern — critical, counters and caches live on this); iteration in arbitrary order; tombstone-free amortized behavior under churn; caller-supplied hasher.
- **SOTA shape**: SwissTable: hashbrown 0.15 / absl::flat_hash_map / folly F14. Open addressing, flat storage, separate control-byte array holding 7-bit hash fragments, SIMD probe of 16-byte groups, load factor 87.5%.
- **Why fast**: One lookup = hash + one 16-byte SIMD compare against control bytes (SSE2 movemask / NEON) filtering 16 slots in ~3 instructions, then typically one element-line touch: ~2 cache lines total per hit at any table size; no per-entry allocation, no chain pointer chasing; 7-bit fragments make false positives ~1/128 so full key compares are rare; deletion uses group-level tombstone elision keeping probes short under churn.
- **Par target**: hashbrown 0.15 with foldhash: u64->u64 hit ~5 ns L2-resident, ~25-40 ns DRAM-resident (dominated by the 2 line misses); insert-with-growth amortized ~15-25 ns; 1M-key uniform-random workload. Blessed form must be SwissTable-class; chained std::unordered_map-class (3-5x slower, 1 alloc/entry) is an explicit fail.
- **Weight**: high. **Substitutability**: One SwissTable-shaped blessed map serves the large majority at par. Honest splits that remain: (a) tiny maps (below), (b) ordered/range access (below), (c) concurrent access (below). Within 'single-threaded unordered map of n>=~16', substitutability is near-total — hashbrown itself replaced every niche map in rustc, Firefox, and the Rust std.

### 7. Tiny map/set (n <= ~8-16, allocation- and hash-cost dominated)

- **Where**: Compilers (per-scope binding tables, attribute lists — rustc's SsoHashMap exists precisely for this), HTTP header maps (typical request: 8-20 headers), small config/registry lookups, per-node annotation maps.
- **Contract**: Same map contract but n is statically expected to be tiny; construction cost matters as much as lookup (created and dropped millions of times); often no removal; keys frequently short strings.
- **SOTA shape**: Inline linear-scan array of (K,V) pairs with spill to SwissTable at threshold (rustc SsoHashMap, LLVM SmallDenseMap); for headers, a flat vec of pairs scanned linearly (http crate HeaderMap hybrid).
- **Why fast**: Skips hashing entirely (a quality string hash is ~1 ns/8B — comparable to the whole linear scan at n=4); zero heap allocation at tiny n (inline pairs live on the owner's cache lines); a scan of 8 inline pairs touches 1-2 lines with perfectly predicted loop branches, vs SwissTable's hash + group load on a freshly-allocated (cold) table.
- **Par target**: LLVM SmallDenseMap<K,V,8> vs DenseMap on build-4-lookup-4-drop microcycle: ~3-4x. Blessed form must beat plain SwissTable on the 'millions of 4-entry maps' compiler workload by >=2x.
- **Weight**: medium. **Substitutability**: Unifiable with the general map the same way small vector unifies with vector: one blessed map with a compile-time inline capacity (linear scan until spill). absl and LLVM both ship exactly this hybrid. Does not justify an independent shape — it is a knob, and the knob's default (0) preserves the simple story.

### 8. Ordered map with range queries (B-tree class)

- **Where**: Databases fundamentally (SQLite's entire storage layer is a B-tree; every LSM memtable), timestamp-indexed event stores, address-ordered structures in allocators (free-region trees), schedulers keyed by deadline, language runtimes' address->metadata maps.
- **Contract**: insert/lookup/remove by ordered key PLUS: range scan [lo, hi) in key order, first/last, predecessor/successor queries, ordered iteration. This range capability is the entire reason to pay the ordered tax; point-lookup-only users should be on the hash map.
- **SOTA shape**: Cache-friendly B-tree: Rust BTreeMap (B=6, 11 entries/node), absl::btree_map (node ~256B). NOT red-black/AVL — per-node allocation and 2-3 pointer chases per level lose by 2-4x.
- **Why fast**: Fanout ~11-16 packs each level's decision into 1-4 sequential cache lines (linear/branchless scan within node), so a 1M-entry tree is ~5 levels = ~5-8 line touches vs ~20 dependent misses for a binary tree; range scans walk leaf nodes nearly contiguously at close to streaming bandwidth; ~16x fewer allocations than node-per-entry trees.
- **Par target**: Rust BTreeMap: 1M u64 keys, point lookup ~60-90 ns DRAM-resident (~2-3x hash map — accepted tax for ordering); ordered full scan within ~2x of vector iteration. absl::btree_map comparable. Blessed form must be B-tree-class; RB-tree-class is a fail.
- **Weight**: medium. **Substitutability**: Cannot be merged with the hash map — range queries are a different contract, and paying B-tree cost for point lookups (or losing ranges) is not par. This is a genuine second blessed associative shape. Within ordered maps, one B-tree shape covers nearly all uses; the residual split (concurrent ordered maps, e.g. skiplists in LSM engines) is the concurrency axis, not a third sequential shape.

### 9. Bounded cache with eviction (LRU class)

- **Where**: Database page caches (SQLite pager), DNS/route caches, JIT/inline caches, CDN and proxy object caches, kernel dentry/inode caches. Every long-running server has several.
- **Contract**: get(key) that both returns &V and updates recency state; insert with capacity-triggered eviction (evicted entry observable for writeback); O(1) for get/insert/evict; capacity in entries or bytes; hit-rate quality matters as much as ns/op (a 1% hit-rate loss can outweigh any constant-factor win).
- **SOTA shape**: Hash map whose values embed intrusive doubly-linked recency links (indices into the same slab) — lru crate / Linux page LRU shape; modern hit-rate-superior variants: CLOCK, S3-FIFO, W-TinyLFU (caffeine/moka) which also drop the promote-write.
- **Why fast**: get is one SwissTable probe plus a 2-4 store list unlink/relink within already-fetched lines (links stored as u32 indices into the entry slab — no extra allocation, no extra cache line); eviction is O(1) tail pop; CLOCK/S3-FIFO variants replace the per-hit list write with a flag set, eliminating the write-contention line entirely.
- **Par target**: lru 0.12 crate: get-with-promote ~20-30 ns at 100K entries. Composition test: the blessed form is map + slab + intrusive links composed from catalog shapes — if that composition reaches lru-crate parity, no dedicated cache primitive is needed.
- **Weight**: medium. **Substitutability**: This is the canonical COMPOSITION scenario, not a primitive: hash map + slab + embedded index links. One blessed composition pattern serves LRU/CLOCK/segmented variants (they differ only in link discipline). Genuine split: sharded concurrent caches (moka/caffeine class) sit on the concurrency axis. The catalog question is whether embedded-index links are expressible; if yes, zero new shapes needed.

### 10. Interning / deduplication (value -> stable small id)

- **Where**: Compilers pervasively (rustc Symbol interner, type interning — every identifier compare becomes a u32 compare), databases (dictionary encoding in columnar formats: Parquet/DuckDB), string tables in linkers, protocol atom tables (HTTP/2 HPACK).
- **Contract**: intern(bytes) -> Id (u32-class), idempotent (equal input, same id); resolve(Id) -> &[u8] O(1); ids stable for the interner's lifetime; equality on Id replaces equality on content thereafter; typically append-only (no un-interning).
- **SOTA shape**: SwissTable keyed by content hash whose entries are (hash, span) into a bump-arena byte store; ids are sequence numbers indexing a span vector. rustc's interner / string-interner crate / DuckDB dictionary encoding.
- **Why fast**: Each unique string's bytes are stored exactly once in a bump arena (no per-string malloc — allocation is a pointer bump, ~2 ns); the map stores 8-byte spans, not owned strings, so probe lines are dense; every downstream comparison collapses from memcmp (~1 ns/16B + 2 pointer derefs) to a 1-cycle u32 compare, and downstream sets/maps of ids become dense integer structures.
- **Par target**: string-interner 0.17 / rustc Symbol: intern hit ~15-25 ns (hash + one probe), resolve ~2 ns (one indexed load); arena append at memcpy speed. Must beat 'HashMap<String, u32>' (which pays a malloc per unique string and pointer-chasing probes) by ~2x on compiler-shaped workloads.
- **Weight**: medium. **Substitutability**: Pure composition: bump arena + span-keyed hash map + id vector, all catalog shapes. Needs a blessed borrowed-key-lookup capability on the map (probe by &[u8] against arena-owned entries without allocating) — that map capability, not a new container, is the real requirement. One taught pattern covers strings, types, and general node interning identically.

### 11. Concurrent hash map (shared read-mostly or sharded-write lookup)

- **Where**: Servers: session tables, connection registries, metrics registries; databases: lock tables, buffer-pool page tables; runtimes: type/method caches. Anything multithreaded with a shared keyed registry.
- **Contract**: Concurrent get from many threads with no writer starvation; insert/remove from multiple threads; per-key atomic get-or-insert; read path must not write shared cache lines (read scaling is the point); snapshot iteration acceptable as weakly consistent.
- **SOTA shape**: Two SOTA shapes by read/write mix: (a) sharded SwissTables — N=64-256 shards each behind its own lock, shard picked by hash (dashmap 6, folly ConcurrentHashMap SIMD mode); (b) read-optimized epoch/RCU maps — readers lock-free and write-free (evmap, Linux RCU hash tables, papaya).
- **Why fast**: Sharding confines lock and data contention to 1/N of traffic — uncontended shard lock is one uncontested CAS (~10-15 ns) and the inner probe is plain SwissTable; RCU-class reads execute ZERO atomic RMWs and dirty zero shared lines (no reader-count cache-line ping-pong, which is what caps RwLock<HashMap> at ~2-3 threads), so read throughput scales linearly with cores.
- **Par target**: dashmap 6 (64 shards): ~1.5-2x single-thread hashbrown cost per op uncontended, near-linear to 16 threads mixed 90/10; papaya/evmap read ~hashbrown-parity per op with linear read scaling. RwLock<HashMap>-class read plateau is the fail line.
- **Weight**: medium. **Substitutability**: Sharding is composition (array of locked blessed maps + hash-based shard pick) — one taught pattern, reaches par for write-mixed workloads. The RCU/epoch read-mostly class is NOT composition: it needs a deferred-reclamation mechanism (epoch/hazard), which for Whitefoot is a checker/runtime capability question, not a container question. Honest split: sharded (composable now) vs read-mostly-linear-scaling (needs a reclamation primitive).


## F3 — identity & linked structures

### 12. Bump arena (mass-free allocation for build-then-drop object graphs)

- **Where**: Compilers as the backbone (rustc arenas for AST/HIR/types, LLVM BumpPtrAllocator), per-request server allocation (one arena per request, reset at end), parsers building trees (simdjson DOM allocates into a contiguous pool), frame allocators in game engines (reset per frame).
- **Contract**: alloc(T) -> reference tied to the arena's lifetime; all allocations freed at once when the arena drops/resets; no individual free; allocated objects may reference each other (same-region references); reset-and-reuse retains capacity across cycles.
- **SOTA shape**: bumpalo 3 / llvm::BumpPtrAllocator / typed_arena: chunked bump-pointer allocation, geometric chunk growth, reset rewinds the pointer keeping chunks.
- **Why fast**: Allocation is a pointer add + one predicted capacity branch (~1-2 ns vs ~15-30 ns for a mimalloc/jemalloc malloc fast path — and no free path AT ALL, which is half the allocator cost); consecutively allocated objects are physically adjacent, so tree/IR traversal in construction order is near-sequential (prefetcher-friendly) instead of malloc's scattered placement; zero per-object metadata.
- **Par target**: bumpalo 3.16: ~1.5-2 ns per small alloc, ~10x malloc/free round trip on alloc-heavy parse workloads; rustc-style AST construction 2-3x faster than Box-per-node. Blessed form must be bump-class, and same-arena cross-references must be expressible (this is the hard checker part, and the whole value).
- **Weight**: high. **Substitutability**: One blessed arena covers compilers, request handlers, and frame allocators identically — the contract (uniform lifetime, mass free) is the same. Genuine split from the SLAB scenario below: arenas cannot free individually. A single 'region' primitive cannot serve both at par; these are two shapes. Within mass-free lifetimes, substitutability is total.

### 13. Slab/pool with generational handles (stable identity + individual free)

- **Where**: Long-lived registries with churn: connection/socket tables in servers and io_uring userdata slots, game entity systems (slotmap is the ECS backbone), kernel object tables (file descriptors are exactly this shape), timer registries, DOM-node stores.
- **Contract**: insert(T) -> Handle (index+generation, Copy, 8 bytes); get(Handle) -> Option<&T> that detects stale handles (generation mismatch) instead of misbehaving; remove(Handle) frees the slot for reuse O(1); iteration over live entries; handles remain valid across other elements' insert/remove.
- **SOTA shape**: slotmap 1.0 / slab 0.4 / generational-arena: contiguous slot array, free slots form an in-place freelist (union of T and next-free index), 32-bit generation per slot bumped on free.
- **Why fast**: insert pops the intrusive freelist head (one load+store, ~2-3 ns, zero malloc); get is one bounds check + one generation compare + indexed load — 1 cache line, no pointer chase; handles are 8-byte Copy values that thread freely through APIs where references can't (no borrow entanglement); dense slot array keeps live-entry iteration near vector speed.
- **Par target**: slotmap 1.0: insert ~3-5 ns, get ~1-2 ns L1-resident; iteration within ~1.5x of Vec. This is ALSO the safety keystone: generational handles make use-after-free structurally detectable, so it aligns with T1/T2 rather than fighting them.
- **Weight**: high. **Substitutability**: One blessed slab serves connection tables, entities, fd tables, and timer slots at par — the shape is remarkably universal. It is also the SUBSTITUTION TARGET for raw pointer graphs (see graphs/trees below), which multiplies its importance. Split from arena is genuine (individual free vs mass free). Sub-split within slabs (dense iteration-optimized vs sparse lookup-optimized, slotmap's DenseSlotMap vs SlotMap) is a minor knob, not a shape.

### 14. Graphs (CFGs, dependency graphs, meshes) via index adjacency

- **Where**: Compilers (CFG/dominator/dataflow — the core of any optimizer), build systems and schedulers (dependency DAGs), databases (query plan DAGs), routing (road networks), game navmeshes and scene graphs.
- **Contract**: Nodes and edges carrying payloads; iterate out-(and often in-)neighbors of a node cheaply; add nodes/edges (build phase often separate from query phase); traversals (BFS/DFS/topo) over millions of edges; node identity stable under growth.
- **SOTA shape**: Two-phase SOTA: mutable build as Vec-of-adjacency-Vecs over u32 node ids (petgraph Graph), then freeze to CSR (compressed sparse row: one offsets array + one packed edge array) for query-heavy phases (petgraph Csr, every serious graph engine, sparse linear algebra).
- **Why fast**: u32 ids instead of pointers halve edge memory (4B vs 8B — graph traversal is bandwidth-bound, so this is directly ~2x fewer bytes moved); CSR packs each node's neighbors contiguously, so scanning them is stride-1 prefetched instead of a linked-node pointer chase (~90 ns dependent miss per edge); node payloads live in parallel dense arrays (SoA), touched only when needed.
- **Par target**: BFS over CSR, ~10M edges DRAM-resident: ~100-300M traversed edges/s/core (bandwidth/latency bound; the frontier's random node touches dominate). Pointer-per-node adjacency (linked lists of edges) runs ~5-10x slower — that is the fail shape.
- **Weight**: medium. **Substitutability**: Graphs are a PATTERN over F1/F3 shapes, not a container: Vec + u32 ids covers build, CSR (two Vecs) covers query. One taught two-phase pattern reaches par across compilers, schedulers, and routing. No new primitive needed; the catalog requirement is merely that parallel arrays indexed by the same id are ergonomic to keep consistent.

### 15. Trees with parent pointers / cross-links (navigable node soups)

- **Where**: AST/HIR with parent navigation (rust-analyzer, DOM trees in browsers), scene graphs, GUI widget trees, B-tree/decision-tree internals with up-links, expression graphs with shared subterms.
- **Contract**: Nodes with parent, first-child, next-sibling navigation in O(1) each; insert/detach subtrees; mutation of a node while holding its identity from elsewhere (the aliasing pattern ownership trees famously reject); stable node identity across restructuring.
- **SOTA shape**: Index-based tree in a slab: nodes in a slotmap/vec, parent/child/sibling stored as u32 handles (indextree, ego-tree, rust-analyzer's rowan red-green trees). Raw-pointer versions (C++ DOM) are the legacy shape it replaces at par.
- **Why fast**: u32 links shrink a 4-link node header to 16B (fits with payload in 1-2 lines; pointer version is 32B of links alone); slab placement gives allocation-order locality so structural traversals hit warm lines; detach/reattach is 3-5 index stores with zero allocator traffic; identity checks are integer compares.
- **Par target**: ego-tree / indextree: child iteration within ~1.2x of pointer-based equivalents; rust-analyzer's rowan demonstrates production-parser-grade parity in fully safe index form. The comparison baseline is C++ pointer DOM traversal — index form must be within ~10-20%.
- **Weight**: medium. **Substitutability**: Same substitution as graphs: slab + u32 handle links, one taught pattern. Parent pointers — unrepresentable as owning references — become plain data as indices, which is exactly how safe Rust already solved this. Split from CSR graphs is real (mutable navigable structure vs frozen scan structure) but both reduce to the same slab primitive; the split is between two patterns, not two primitives.

### 16. Intrusive lists / membership links (O(1) unlink from within the element)

- **Where**: Kernels pervasively (Linux list_head: run queues, wait queues, LRU lists — an object on 3 lists simultaneously), allocator internals (mimalloc's in-page free lists ARE intrusive links threaded through free blocks), timer wheels' per-bucket lists, connection state machines moving between pending/active/closing lists.
- **Contract**: An element belongs to one or more lists via links embedded in itself; O(1) unlink given only the element (no list-head scan); O(1) splice between lists; zero allocation per membership change; element identity independent of list membership.
- **SOTA shape**: C: struct with embedded list_head + container_of. Safe SOTA substitute: elements in a slab, membership links as u32 prev/next index fields inside the element (intrusive-collections crate for the pointer form; the index form is what safe systems code actually ships).
- **Why fast**: Links live inside the element's own cache line(s), so unlink = 2-4 stores to lines already fetched by whoever decided to unlink — no separate node allocation, no hash lookup to find the list node, no allocator traffic ever; moving an element between states (pending -> active) is a splice, not a remove+insert with rehash.
- **Par target**: Linux list_del: ~2 stores. Index-in-slab substitute: same 2-4 stores plus one slab base add — measured parity within noise (the mimalloc free-list pattern in index form is the benchmark: push/pop free block ~2-3 ns). Fail shape: VecDeque/HashMap-based membership at ~10-30x for the unlink-from-middle operation.
- **Weight**: medium. **Substitutability**: The index-in-slab substitute reaches par for every use where elements live in one owned slab — which covers kernels' object tables, allocators, timers, and connection tables. The irreducible residue is intrusive links through memory the structure does not own (mimalloc threading links through FREE user blocks): that specific trick needs either a trusted built-in allocator or an owned-slab reframing. One taught embedded-index-links pattern, riding on the slab primitive, covers the rest.


## F4 — queues & rings

### 17. Deque / FIFO work buffer (single-threaded)

- **Where**: BFS frontiers (compilers: worklist algorithms are THE optimizer idiom — dataflow, dominance, constant propagation), sliding-window algorithms (codecs, rate limiters), undo buffers, token lookahead buffers in parsers.
- **Contract**: push_back/pop_front (and often both ends) O(1) amortized; indexed access; iteration front-to-back; contiguous-slice access when possible (as_slices); grows on demand; single-threaded — no synchronization tax tolerated.
- **SOTA shape**: Power-of-two ring over a growable buffer with masked head/tail indices: Rust VecDeque. (C++ std::deque's chunked shape loses on locality and is not the SOTA.)
- **Why fast**: push/pop are one masked index update + one write/read — same cache behavior as Vec (tail/head lines stay resident); index masking (idx & (cap-1)) is 1 cycle with no branch, no modulo; growth is one bulk memcpy; no per-element allocation ever; worklist loops run entirely in L1 when the frontier is small.
- **Par target**: Rust VecDeque: push_back+pop_front cycle ~2-3 ns; within ~10% of Vec push/pop. Blessed form must match VecDeque-class; chunked-deque-class or linked-list-class (per-node alloc, pointer chase) is a fail.
- **Weight**: medium. **Substitutability**: One blessed ring-over-vector shape fully covers single-threaded deque/FIFO/sliding-window uses at par — no genuine splits within the sequential case. It is nearly a knob on the vector (two indices instead of one length); the catalog can plausibly teach it as the vector's sibling with shared internals.

### 18. SPSC ring buffer (two-thread streaming handoff)

- **Where**: Audio/video pipelines (the audio industry standard — real-time thread feeds/drains without locks), NIC/driver descriptor rings (every DMA ring is an SPSC ring), io_uring's SQ/CQ rings themselves, logging fast paths (app thread -> flusher thread), sensor/DSP streams.
- **Contract**: Exactly one producer thread and one consumer thread; bounded capacity fixed at creation; push fails/blocks when full, pop when empty (backpressure is the feature); FIFO order; batched reserve/commit of multiple slots; wait-free progress on both sides (real-time audio cannot take a lock or a syscall).
- **SOTA shape**: Lamport ring refined: fixed power-of-two buffer, monotonic head/tail counters on SEPARATE cache-line-padded lines, acquire/release (no CAS, no SeqCst), each side caching the other's index to skip most atomic loads (rtrb 0.3, folly ProducerConsumerQueue, DPDK/io_uring rings).
- **Why fast**: Zero read-modify-write atomics — each counter has exactly one writer, so push = plain write + one release store; cached opposing-index means most operations execute ZERO cross-core loads (the counter line ping-pongs only when the cached bound is exhausted, ~once per batch); cache-line padding prevents head/tail false sharing (~100+ cycle line bounce per op otherwise); batching amortizes even that to ~0.1 atomic ops/item.
- **Par target**: rtrb 0.3: ~8-15 ns per push+pop round trip single-item, >100M items/s batched, cross-core. io_uring SQ ring is the same math in kernel ABI form. Fail shapes: mutex ring (~10x), or MPMC queue used as SPSC (~3-5x — the CAS and sequence overhead is pure waste here).
- **Weight**: medium. **Substitutability**: Genuinely NOT substitutable by an MPMC shape at par — the entire win IS the single-writer-per-counter structure that MPMC must give up. Within SPSC, one blessed ring covers audio, drivers, and logging identically. This scenario also carries outsized structural weight: it is the minimal two-thread ownership-handoff primitive the safety model must be able to type.

### 19. MPMC bounded queue (many-to-many channel buffer layer)

- **Where**: Thread-pool task submission (rayon/tokio injector queues), bounded channels (crossbeam-channel's array flavor, Go buffered channels), pipeline stages with multiple workers on both sides, connection accept queues.
- **Contract**: Any thread may push/pop; bounded capacity with full/empty as normal Results (backpressure); per-element FIFO-ish ordering; lock-free progress under contention; element ownership transfers through the queue (move in, move out — the safety-relevant handoff).
- **SOTA shape**: Vyukov bounded MPMC queue: ring of slots each carrying an atomic sequence number; enqueue claims a slot with one fetch_add/CAS on the tail, writes the element, publishes via the slot's sequence (crossbeam ArrayQueue, rigtorp::MPMCQueue, tokio injector).
- **Why fast**: Exactly one contended RMW per operation (tail fetch_add ~20-40 cycles uncontended); per-slot sequence numbers make publication local to the slot's own cache line, so producers contend on the tail counter but NOT on each other's slots; no head-tail shared lock, no per-element allocation (contrast Michael-Scott linked queues: alloc + 2 CAS per op, ~2-3x slower and allocator-entangled).
- **Par target**: crossbeam ArrayQueue: ~15-25 ns/op uncontended, ~50-150 ns/op at 8 contending threads (counter line bouncing is the physics floor); ~10-20M ops/s sustained 4p4c. Blessed form must be Vyukov-class; Mutex<VecDeque>-class collapses to ~1-2M ops/s under contention and is the fail line.
- **Weight**: medium. **Substitutability**: One Vyukov-shaped blessed queue covers general MPMC at par, and MPSC-specialized variants only buy ~1.5x — arguably a skippable refinement. It CANNOT replace SPSC (3-5x tax) or the work-stealing deque (wrong operation set). Honest tri-split of concurrent queues: SPSC ring / MPMC bounded / stealing deque — three shapes, each at par only for its own regime, none subsumable.

### 20. Priority queue (heap) and the timer specialization

- **Where**: Schedulers (deadline/EDF scheduling, OS run queues), Dijkstra/A* (routing, game pathfinding), event simulation, top-k selection (databases: ORDER BY ... LIMIT k), compression (Huffman construction), and the giant special case: timeout management in every server runtime.
- **Contract**: push(item, key), pop_min O(log n); peek O(1); no stable order among equal keys required; for timers specifically: massive insert+cancel volume (most timeouts never fire), coarse deadline resolution acceptable, cancellation must be cheap.
- **SOTA shape**: General: implicit d-ary heap in a flat vector (4-ary: dary_heap crate; std BinaryHeap is the 2-ary baseline). Timers: hierarchical timer wheel (Varghese-Lauck; Linux kernel timers, tokio's timer wheel) — a different shape, deliberately.
- **Why fast**: Implicit heap = zero pointers, zero allocation, array indexing only; 4-ary halves tree height vs binary and puts all 4 children in one cache line, cutting lines touched per sift by ~2x; sift-down branches on comparisons but the array layout keeps upper levels permanently L1-resident. Timer wheel replaces O(log n) sifts with O(1) bucket append (one masked index + list push, ~3-5 ns) and O(1) cancellation via embedded link removal — at 1M pending timeouts this is the difference between feasible and not.
- **Par target**: std BinaryHeap: push+pop ~30-80 ns at 1M u64 (miss-dominated at depth); dary_heap 4-ary ~1.5x better. Timer path: tokio's wheel ~10-20 ns insert/cancel regardless of count. Blessed heap must be implicit-array-class (pointer-based/pairing heaps lose on cache behavior for the common case).
- **Weight**: medium. **Substitutability**: One implicit d-ary heap covers general priority uses at par. The timer wheel is a genuine split — O(1)-with-coarse-time beats O(log n)-exact for the timeout workload and no heap parameter closes that gap — BUT the wheel is itself a composition of catalog shapes (array of buckets + embedded-link lists + masked index), so it is a taught pattern over F3/F4 primitives, not a new primitive. Decrease-key uses (textbook Dijkstra) are served at par by lazy deletion (push duplicates, skip stale pops) — no indexed heap needed.

### 21. Work-stealing deque (per-worker task queue with thief end)

- **Where**: Every parallel runtime: rayon, tokio's worker queues, Go's runtime scheduler, TBB, Java ForkJoinPool. Few programs write one, but every parallel program runs on one, and a systems language that cannot express its own runtime's core structure outsources its heart.
- **Contract**: Owner thread: push/pop at the bottom, called extremely hot (every task spawn/finish), must cost near-plain-deque; other threads: steal from the top, rare, may be O(CAS); tasks are owned values handed off exactly once; correct under concurrent steal during owner pop of the last element.
- **SOTA shape**: Chase-Lev deque (crossbeam-deque 0.8, Go runqueues as the fixed-size variant): growable ring, owner updates bottom with plain ops + one memory fence, stealers CAS the top; the only synchronization conflict is the single-element race.
- **Why fast**: Owner's hot path has NO atomic RMW — push is a store + release fence-equivalent, pop is a decrement + one SeqCst fence (~20 cycles on x86, the acknowledged cost) with a CAS only on the last-element race; steals (rare by design — work-stealing's whole premise) pay the CAS; owner's lines stay in its own cache, stealers touch only top + the stolen slot, so there is no steady-state line ping-pong.
- **Par target**: crossbeam-deque 0.8: owner push+pop ~5-10 ns; steal ~25-40 ns cross-core. rayon's task throughput (~10-20 ns/spawn overhead) is built directly on this number. Fail shape: Mutex-per-queue or MPMC-for-everything, which taxes every task spawn ~3-10x.
- **Weight**: medium. **Substitutability**: Not substitutable by any other queue shape at par — the asymmetric owner/thief contract IS the optimization. For the catalog it is a candidate for a trusted built-in or a once-proved library form rather than a taught general pattern: its memory-ordering subtlety (the SeqCst fence pairing is a published-paper-grade argument) is exactly what should be proved once by experts, exposed behind a safe facade, and never rewritten by ordinary writers.


## F5 bytes & parsing

### 22. Single/few-byte scanning (memchr class)

- **Where**: ripgrep and every grep-class tool, lexers finding delimiters, HTTP/1 header CRLF scans, CSV/newline splitting, strlen/memchr in libc, SQLite tokenizer, log processors. Essentially every program that touches text or wire bytes; among the most-executed loops in systems software.
- **Contract**: Given a borrowed byte slice and 1-3 needle bytes (or a small byte class), return the first/last match index or iterate all matches. No allocation, caller retains ownership of the buffer, correct on any length including 0-15 bytes, and a reverse variant (memrchr). Failure = Option/None, never a fault.
- **SOTA shape**: memchr crate 2.7 / glibc AVX2 memchr: unaligned head, aligned main loop unrolled 4x (128B/iteration), 32B vector compare + movemask + tzcnt for match extraction, scalar/SWAR tail; runtime ISA dispatch resolved once.
- **Why fast**: 32-64 bytes examined per vector compare; ~1 taken branch per 128 bytes (loop back-edge plus one movemask test); saturates the 2x32B/cycle L1 load ports; tzcnt converts the match mask to an index with zero branches; no allocation, no per-byte loop.
- **Par target**: memchr crate 2.7 on x86-64/AVX2: roughly 20-40 GB/s on >1KB haystacks with rare matches; short-haystack (<=16B) calls in ~2-5 ns. Workload: uniform random bytes, rare needle; metric GB/s and ns for short inputs.
- **Weight**: high. **Substitutability**: Largely yes: one blessed 'SIMD scan over a byte slice with a small match predicate' form covers memchr/memchr2/memchr3/memrchr and byte-class skip, with internal short-input and ISA dispatch hidden. Genuine split: substring search (memmem) needs a different algorithm shell (rare-byte heuristic + Two-Way/Rabin-Karp fallback) layered on the same scan primitive — one primitive, two catalog entries.

### 23. Block-parallel validation and structural indexing (simdjson/simdutf class)

- **Where**: UTF-8 validation on every string ingestion boundary (Rust str::from_utf8, simdutf in Node/Bun), simdjson stage-1 structural indexing, base64 encode/decode, whitespace skipping in parsers, HTML/JSON tokenization in browsers and API servers. Every service boundary that accepts external text.
- **Contract**: Validate or classify a borrowed byte buffer in bulk: return ok/first-error-position, or produce a compact index (bit masks or position array) of structural characters. No per-byte callback, no allocation proportional to input for validation, streaming-chunk capable.
- **SOTA shape**: simdjson 3.x stage 1 / simdutf: 64B blocks classified with pshufb nibble table lookups, Keiser-Lemire branchless UTF-8 range validation, carry-less multiply (clmul) for the in-string quote mask, results accumulated as one u64 bitmask per 64B block.
- **Why fast**: 64 bytes classified per iteration in ~10-25 vector ops with zero data-dependent branches, so unpredictable input causes no mispredicts (vs ~15-20 cycles per miss in a branchy validator); pshufb performs 16-32 parallel table lookups per cycle; bitmask output means downstream consumes 64 positions per u64 load.
- **Par target**: simdjson 3.x stage 1: ~6-10 GB/s on twitter.json with AVX2; simdutf UTF-8 validation: ~20+ GB/s ASCII-heavy, ~8-12 GB/s mixed multibyte. Metric: GB/s validated/indexed on standard corpora.
- **Weight**: high. **Substitutability**: Partial. One blessed 'block classifier: 64B chunk -> bitmask pipeline (table lookups, range checks, mask carry across blocks)' form covers UTF-8 validation, JSON stage-1, base64, and whitespace skipping. Transcoding (UTF-8<->UTF-16) and full-format parsers need additional shapes on top (gather/expand, checked cursor), but they reuse the classifier; the split is classifier vs transcoder, both expressible over the same vector core.

### 24. Serialization/deserialization of structured messages

- **Where**: protobuf/gRPC in virtually every service fleet, flatbuffers/capnp in games and low-latency trading, serde/bincode/postcard in Rust systems, row encoding in RocksDB/SQLite records, wire protocols (Kafka, Postgres). Ubiquitous in servers and databases.
- **Contract**: Encode: append fields into one growable byte buffer, no per-field allocation. Decode: walk a borrowed slice with truncation-checked reads, produce either owned values or zero-copy views (strings/bytes borrow the input); error out on malformed input without UB; per-field cost must be constant, not per-byte.
- **SOTA shape**: upb/prost-style monomorphized straight-line field code; varint decode via one unaligned 8-byte load plus shift/mask bit tricks (or pext) instead of a byte loop; fixed fields as direct unaligned loads; flatbuffers validate-then-view offset graphs; encode into a bump-grown buffer.
- **Why fast**: One unaligned 8B load + a few ALU ops per varint instead of a 1-10 iteration data-dependent byte loop (kills the mispredict chain); one bounds check per field or amortized per message rather than per byte; zero decode allocations because views borrow; encode is pure sequential stores, memcpy-class at ~32B/cycle.
- **Par target**: upb / protobuf C++ arena decode: ~1-3 GB/s on typical message mixes; bincode/postcard on POD-heavy structs: near-memcpy (>10 GB/s). Metric: GB/s decoded on a fixed message corpus; encode allocations per message = 0-1.
- **Weight**: high. **Substitutability**: Splits into three sub-shapes: (a) fixed-layout blit for POD structs (memcpy-class, trivially one shape), (b) 'checked cursor over borrowed bytes' for varint/TLV stream formats, (c) 'validated offset-graph view' for flatbuffers/capnp-style zero-copy. One blessed checked-cursor form covers (b) and, combined with the F5 typed-view form, most of (c). Two-to-three catalog entries, not one.

### 25. Checksums and non-cryptographic hashing

- **Where**: Hash computation in every hash map (hashbrown default, SipHash/foldhash), CRC32C in SQLite WAL, RocksDB blocks, iSCSI/filesystems, zlib/zstd frame checksums (xxh64), content addressing (blake3), DB join key hashing. Universal.
- **Contract**: Bytes -> u32/u64/u128 digest, one-shot and streaming/incremental. Two distinct demand profiles: map hashing wants minimal latency on 4-32B keys (and optionally HashDoS resistance); integrity checksums want maximal throughput on 4KB-1MB pages.
- **SOTA shape**: CRC32C: hardware crc32 instruction with 3 interleaved streams (or PCLMULQDQ folding, as in isa-l); xxh3/wyhash/foldhash: unaligned 8-16B loads folded through 64x64->128 multiplies, with branch-minimal specialized paths for length <=16; blake3 for cryptographic (SIMD tree).
- **Why fast**: crc32q retires 1/cycle with 3-cycle latency, so 3 independent streams reach ~15-25 GB/s; a widening multiply mixes 16 bytes per ~3-cycle op; short-key paths hash an 8-16B key in ~5-10 cycles total with the length dispatch being 2-3 predictable branches; everything runs from registers — zero memory traffic beyond the input itself.
- **Par target**: xxh3 (xxhash 0.8): ~30-50 GB/s bulk on AVX2, ~4-6 ns one-shot for 16B keys; crc32c via SSE4.2 3-way: ~15-25 GB/s. Metric: ns/hash at 8/16/64B and GB/s at 64KB.
- **Weight**: high. **Substitutability**: Yes at the shape level: one blessed 'byte-stream fold kernel' (unaligned wide loads + wide multiply / carryless multiply / hardware CRC step, plus a short-input path) hosts xxh3, wyhash, CRC32C, and SipHash as instances. The real split is algorithm policy (DoS-resistant vs fastest vs fixed-polynomial), not language shape; one form with the clmul/crc32 instructions reachable suffices for par.

### 26. Compression codec inner loops (LZ + entropy)

- **Where**: zstd/LZ4 in RocksDB, Parquet, Kafka, filesystems (btrfs, ZFS), HTTP content encoding, game asset streaming; zlib legacy everywhere. Concentrated in storage/network stacks but those are core systems audiences.
- **Contract**: Decode a compressed block into a caller-provided output buffer, where the inner loop copies matches from earlier in the same output buffer (overlapping, self-referential copies with offset possibly < length); encoder side needs a hash-table match finder over a sliding window. Malformed input must error, never corrupt.
- **SOTA shape**: LZ4 1.9 decode: 16-32B 'wild copies' that deliberately store past the copy length into guaranteed slop space, shuffle-based patterns for offsets < 16; zstd: FSE/Huffman decode with 2-4 interleaved independent bitstreams; match finder: 4-byte hash into a chain/HC table.
- **Why fast**: Wild copy turns any length<=32 copy into 1-2 vector stores with no length loop and no per-copy length branch — legal only because the output buffer contract guarantees writable margin; interleaved bitstreams break the serial refill dependency giving 3-4x ILP; table-driven symbol decode is one load + shift per symbol.
- **Par target**: LZ4 1.9 decompress: ~4-5 GB/s/core; zstd 1.5 decompress: ~1.5-2.5 GB/s/core (Silesia corpus). Metric: GB/s decompressed single-core.
- **Weight**: medium. **Substitutability**: The hard case in F5. Requires two blessed sub-shapes that generic safe code cannot reach at par: (a) a 'margin buffer' — output slice with a typed guarantee of writable slop so over-length vector copies and overlapping self-copies are checkable without per-copy branches; (b) a multi-stream bit-reader with amortized bounds (padded input or checked refill). Both are reusable across all LZ-family and entropy codecs, so two entries cover the domain; without them, safe code loses ~2-4x here.

### 27. Zero-copy typed views over binary layouts (packets, pages, records)

- **Where**: Packet header parsing (DPDK-style networking, eBPF, smoltcp), database page layouts (SQLite b-tree pages with big-endian u16 cell pointers, InnoDB), file formats (ELF loaders, Parquet), mmap'd index structures (tantivy, LMDB). High across kernels, databases, networking.
- **Contract**: Overlay typed, endian-explicit, alignment-explicit field accessors on a borrowed byte buffer after a one-time length/shape validation; each subsequent field access must be a plain load with no residual checks; no copying; view lifetime tied to the buffer; variable-length internal offsets must be checkable once then trusted.
- **SOTA shape**: zerocopy/bytemuck FromBytes-style transmuting views; C packed structs with byte-order accessor macros; capnp's validate-pointers-once-then-raw-access; SQLite's direct big-endian reads from page bytes.
- **Why fast**: After the one upfront validation, a field access is exactly 1 (possibly unaligned) load plus an optional bswap — zero bounds checks, zero validity branches, zero deserialization allocation; a packet header or page cell stays within 1-2 cache lines; codegen is bit-identical to a C struct dereference.
- **Par target**: zerocopy 0.8: field access codegen identical to raw C pointer access (0 instructions overhead, verified by asm diff); SQLite page-cell walks at a few ns per cell. Metric: instructions per field access vs C baseline (must equal 1 load [+1 bswap]).
- **Weight**: high. **Substitutability**: Mostly one shape: 'validated typed view over borrowed bytes, endianness and alignment declared per field' covers fixed-header packets, records, and page headers. Split: formats with data-dependent internal offsets (cell pointer arrays, variable-length records) additionally need the checked-cursor shape with once-proven bounds facts; the two together cover nearly everything.


## F6 memory management

### 28. General-purpose allocator fast path (mimalloc/jemalloc class)

- **Where**: Every program's default heap: browsers, servers (jemalloc in Redis historically, mimalloc in many services), language runtimes. Also the allocator itself is a canonical piece of systems software the language should be able to express or embed.
- **Contract**: alloc/free of arbitrary sizes from any thread, cross-thread free (alloc on A, free on B), amortized O(1), low fragmentation, and a pluggable global-allocator hook so all library containers route through it. Consumers never see internals — they need malloc/free/realloc semantics at SOTA cost.
- **SOTA shape**: mimalloc 2.x: per-thread heaps, size-class segregated free lists embedded in 64KB pages, free = push onto the page's thread-local list, remote frees batched onto a single atomic-exchange list drained lazily; tcmalloc per-CPU caches via restartable sequences.
- **Why fast**: Fast-path malloc is ~10-20 instructions touching 1-2 cache lines (thread-local heap struct + the returned block), with zero atomics, zero fences, zero locks; size-class rounding replaces searching with indexing; thread-sharding makes the common path contention-free; remote-free batching amortizes the one atomic to ~1/n.
- **Par target**: mimalloc 2.x: ~15-25 ns malloc+free pair for small sizes single-threaded, near-linear scaling on mimalloc-bench (larson, cache-scratch) to 32+ threads. Metric: ns per alloc/free pair and larson throughput vs mimalloc.
- **Weight**: high. **Substitutability**: For consumers, yes with n=1: a single global allocator interface serves all call sites at par, and a trusted built-in (TCB) implementation is a legitimate answer. The honest split is authorship: writing a new allocator requires intrusive free lists threaded through untyped/reused memory and type-changing block reuse — capabilities most user code never needs. Blessing consumption (n=1) and treating allocator authorship as a separate TCB-adjacent capability is viable.

### 29. Arena / bump allocation with scope-lifetime free

- **Where**: Compilers (rustc arenas, LLVM BumpPtrAllocator, upb protobuf arenas), request-scoped allocation in servers, per-frame allocators in game engines, AST/IR construction in every parser. Very high in exactly this project's audience.
- **Contract**: Allocate many heterogeneous objects extremely fast; objects may reference each other (same lifetime region); no individual free; the whole arena is freed or reset at scope end; optionally reuse the arena's memory across iterations after a reset that provably invalidates all references into it.
- **SOTA shape**: bumpalo 3.x / LLVM BumpPtrAllocator: bump pointer + capacity compare within a chunk, geometric chunk-list growth, no per-object headers; typed-arena variants either run drops at reset or restrict to droppable-free types.
- **Why fast**: Allocation is 2-4 instructions (add, compare, rarely-taken branch) versus ~15-25 ns for malloc — roughly 5-10x; no free-list or size-class metadata touched, so no extra cache lines per alloc; sequentially allocated object graphs get spatial locality (parent and children on the same lines); deallocation of N objects is one pointer reset, zero per-object work.
- **Par target**: bumpalo 3.x: ~1-3 ns per 16-64B allocation; end-to-end, arena-allocated AST construction in rustc-style workloads runs measurably (tens of %) faster than Box-per-node. Metric: ns/alloc and total time building a 1M-node tree.
- **Weight**: high. **Substitutability**: Nearly one shape: 'region with same-lifetime interior references' covers compilers, parsers, request scopes, and frames. Two modes, not two shapes: (a) types needing drop (typed arena runs drops at reset) vs plain data; (b) reset-and-reuse requires the lifetime system to prove no references survive reset — same form, extra proof obligation. One catalog entry with two modes is honest.

### 30. Slot pools / slabs: O(1) reuse without re-zeroing

- **Where**: Connection/request objects in network servers, entity slots in game engines (generational indices), kernel SLUB slab caches, RocksDB memtable node pools, freelists throughout allocators and runtimes.
- **Contract**: Fixed-type slots with O(1) insert/remove of individual objects, stable handles that survive other slots' churn, ABA/use-after-free detection via generation counters, dense storage for fast iteration, and — critically — recycled slots are NOT re-zeroed; the old bytes are overwritten only by the new object's construction.
- **SOTA shape**: slotmap/slab-crate shape: a dense Vec of union{value, next_free_index} with an intrusive free-list head, plus a generation counter per slot; kernel SLUB adds per-CPU partial pages for the multicore variant.
- **Why fast**: Insert = pop free-list head: 1-2 cache-line touches, no malloc call, no size-class lookup; remove = push: same; skipping the memset of a recycled 256B slot saves ~all of its cost since construction writes every live field anyway; dense Vec storage packs 1-4 slots per 64B line so full iteration runs at streaming bandwidth instead of pointer-chasing.
- **Par target**: slotmap 1.x / slab 0.4: ~2-5 ns per insert/remove at steady state vs ~20-40 ns for HashMap-based registries; iteration at Vec speed. Metric: ns per insert+remove cycle, ns per element iterated.
- **Weight**: high. **Substitutability**: Largely one shape: the generational slot pool covers servers, games, and runtime registries. Splits: (a) intrusive in-object free links (kernel-style, saves the index array) vs external — a memory-footprint refinement, same semantics; (b) concurrent multi-producer pools need a genuinely different (sharded/atomic) shape belonging to the concurrency family. One blessed single-threaded form reaches par for the dominant uses.

### 31. Uninitialized buffers: zero-init avoidance with tracked init extent

- **Where**: read()/recv() into fresh buffers in every I/O stack, Vec::with_capacity + extend patterns, decompression and decode output buffers, io_uring registered buffers, network receive rings. Anywhere bytes are about to be fully overwritten by an external writer.
- **Contract**: Obtain writable memory without paying a memset; the type system tracks the initialized extent (prefix/cursor); the kernel or a decoder writes into the uninitialized region; only the written prefix is then exposed as initialized bytes. Reading uninitialized memory must remain unrepresentable — the whole point is proving the write happened.
- **SOTA shape**: Rust Read::read_buf with BorrowedBuf/BorrowedCursor (init-extent-tracking cursor over MaybeUninit), C's plain uninitialized buffers (fast but unsafe), kernel discipline around avoiding double zeroing (__GFP_ZERO used once, not twice).
- **Why fast**: Eliminates one full store pass over the buffer: memset of 64KB at ~30 GB/s costs ~2 microseconds plus evicting 64KB of useful cache — for small-to-medium reads this is 20-50% of total syscall-path cost; for a decode loop it removes half the store traffic. The win is purely 'stores not executed and cache lines not dirtied twice'.
- **Par target**: C read() into an uninitialized 64KB stack/heap buffer (zero memset instructions in the path) — Rust's BorrowedBuf matches it. Metric: codegen inspection showing zero memset/zeroing in the I/O hot path, plus throughput parity on a read-heavy microbench vs C.
- **Weight**: high. **Substitutability**: Yes, essentially one shape: a 'write-cursor buffer with type-tracked initialized extent' covers syscall reads, decoder outputs, and Vec spare capacity uniformly; the assume-init/freeze step is the single point where the safety proof concentrates. The only split is partially-initialized structs (field-wise init tracking), which is a different, smaller feature.

### 32. Memory-mapped data: file-backed byte regions

- **Where**: LMDB (entire B-tree in a shared map), SQLite mmap mode, search indexes (tantivy, Lucene-class), ripgrep's mmap path, executable/dylib loaders, ML weight loading. Concentrated in databases and indexes.
- **Contract**: Treat a file region as a borrowed byte slice; layer the F5 zero-copy typed views on it; random access into data far larger than RAM with page-fault-driven laziness; lifetime tied to the map; honesty about the trust model — file contents are external, possibly changed by other processes.
- **SOTA shape**: memmap2 + zerocopy-style views; LMDB's copy-on-write B-tree pages read directly in the shared map; madvise(WILLNEED/RANDOM) tuning.
- **Why fast**: Removes the read() copy entirely: warm access is a direct page-cache load (zero syscalls, zero copies) vs one syscall + one full buffer copy per read; a 100GB index is addressable without materialization; the OS page cache provides adaptive caching for free; cold-path cost is a page fault (~1-5 us) amortized over 4KB.
- **Par target**: LMDB 0.9: ~100-200 ns per warm read-transaction get; ripgrep mmap-vs-read within ~10% either way on large files; tantivy term lookups at Lucene par. Metric: warm random-read latency vs an explicit-buffer baseline.
- **Weight**: medium. **Substitutability**: One shape covers the common case: a read-only frozen map exposing &[u8] plus the typed-view form. Honest split: shared mutable maps (another process or a writer thread mutating the same pages, LMDB-writer style) break the immutable-borrow assumption and need an 'untrusted/volatile external bytes' contract with per-access or per-snapshot discipline — rarer, genuinely harder, and reasonable to bless separately or defer.


## F9 machine operations

### 33. Portable SIMD vector types with runtime ISA dispatch

- **Where**: The substrate under all of F5 (memchr, simdjson, simdutf, hashing), image/video codecs (dav1d), string routines, ML/DSP kernels, column scans in analytical databases (DuckDB filters). If F5 kernels are demanded, this facility is what they are written in.
- **Contract**: Width-explicit vector types (u8x16/u8x32/f32x8...) with lane-wise ops, shuffles/permutes, compare-to-mask, movemask-to-scalar; write a kernel once, dispatch per-CPU (SSE2/AVX2/AVX-512/NEON) selected once at startup; masks convert cheaply to scalar control flow.
- **SOTA shape**: Rust std::simd / Google highway: portable vector API compiled per-target plus cached-CPUID function multiversioning; direct intrinsics reserved for ISA-specific instructions (pshufb-as-table, clmul, AES rounds, vpermb).
- **Why fast**: 16-64 lanes per instruction at 1-2 vector ops/cycle; movemask+tzcnt bridges vector results to scalar decisions in 2 instructions; one-time dispatch reduces per-call cost to a predictable indirect (or patched direct) call; shuffle units do a full 16/32-entry table lookup per cycle.
- **Par target**: highway / std::simd kernels within 0-10% of hand-written intrinsics on classification/scan kernels; concretely, the blessed form must reproduce memchr-crate and simdutf numbers (F5-1/F5-2) when those kernels are written in it. Metric: kernel GB/s vs intrinsics reference.
- **Weight**: high. **Substitutability**: One portable-SIMD form covers ~80-90% of kernels. Genuine split: a minority of SOTA kernels are built on ISA-specific instructions with no portable equivalent at par (clmul, GFNI, AES-as-mixer, vpermb across 64 lanes). Resolution is one blessed form plus a named-intrinsic escape inside the same type system — not a second competing form.

### 34. Unaligned loads, endianness, and alignment control

- **Where**: Every parser and serializer (F5-3/6) reads multi-byte integers at arbitrary offsets; network code converts big-endian wire fields; SIMD wants 32/64B-aligned buffers to avoid split-line loads; concurrent counters want 64B padding against false sharing (crossbeam CachePadded, folly cacheline_align).
- **Contract**: Read/write u16/u32/u64/u128 at arbitrary byte offsets with declared endianness, compiling to exactly one load/store plus at most one bswap; declare over-alignment on types and allocations (16/32/64B, 4KB); never a byte-at-a-time fallback.
- **SOTA shape**: Rust's from_le_bytes/from_be_bytes over slices (optimizes to one mov / movbe), C's memcpy-idiom unaligned access, #[repr(align(64))]/alignas(64), posix_memalign for page-aligned buffers.
- **Why fast**: Modern x86/ARM unaligned loads are penalty-free unless they split a cache line (~2x when they do — which alignment control prevents); the alternative byte-assembly costs 4-8 instructions per field; 64B padding gives each contended counter its own cache line, eliminating false-sharing ping-pong worth 5-20x on multicore counters.
- **Par target**: Codegen parity: from_le_bytes-class access = 1 mov (+1 bswap for BE) verified by asm inspection vs C; crossbeam-utils CachePadded benchmark: contended per-thread counters ~5-20x over unpadded. Metric: instruction-level parity plus the false-sharing microbench.
- **Weight**: high. **Substitutability**: Yes — cleanly one surface: an endian-explicit load/store form on byte buffers plus an alignment attribute on types/allocations. No genuine use-case split; this is the rare scenario where n=1 is simply true.

### 35. Word-level bit operations: popcount/ctz/clz/pdep-pext and set-bit iteration

- **Where**: hashbrown/SwissTable's group probe (movemask + trailing_zeros candidate loop), roaring bitmaps, chess/game bitboards, succinct rank/select structures, allocator occupancy bitmap scans, compiler register-allocation bitsets, Bloom filters.
- **Contract**: popcount, tzcnt/lzcnt, rotate, pdep/pext on machine words, each compiling to its single instruction; ergonomic iteration over set bits of a mask (the x &= x-1 loop); wrapping arithmetic without checks where proven safe.
- **SOTA shape**: tzcnt-driven set-bit iteration with x &= x-1; BMI2 pdep/pext for select/rank (with a fallback table on pre-Zen3 AMD where they are microcoded ~18 cycles); SWAR tricks for pre-BMI targets.
- **Why fast**: One instruction replaces a loop: popcnt is 1 cycle vs a 64-iteration bit loop; set-bit iteration takes ~1-2 cycles per set bit with a perfectly predicted loop count equal to popcount; pext gathers scattered bits in 3 cycles vs ~20 scalar ops; hashbrown's entire probe loop is movemask+tzcnt+shift, ~4 instructions per candidate.
- **Par target**: hashbrown 0.15's probe loop cost (its lookup numbers depend on this compiling exactly); roaring-rs at CRoaring parity on rank/contains. Metric: codegen shows exactly one instruction per bit op; bitset iteration ~1-2 cycles/set bit.
- **Weight**: high. **Substitutability**: Yes — these are pure operations on integer values with no ownership or aliasing surface; one blessed integer-method set compiling to single instructions serves every use. The only wrinkle (pdep/pext AMD fallback) is an implementation-dispatch detail, not a shape split.

### 36. Branchless selection and predication (cmov class)

- **Where**: pdqsort's branchless partition (Rust sort_unstable), branchless binary search over sorted arrays (Khuong/Morin style, used in DB index probes), clamping/saturation in codecs and DSP, min/max reductions, constant-time comparisons in crypto.
- **Contract**: A data-dependent select that reliably lowers to cmov/csel or arithmetic masking rather than a conditional branch — either as a strong optimization guarantee for unpredictable-data hot loops, or as a hard guarantee (constant-time) for crypto; plus conditional-store/advance idioms for partition loops.
- **SOTA shape**: pdqsort's branchless partition (unconditionally store, advance the pointer by the predicate result); branchless lower_bound (offset += (probe < key) * half); explicit cmov via two-sided arithmetic select where compilers waver.
- **Why fast**: A mispredicted branch flushes ~15-20 cycles; on 50/50-unpredictable comparisons a cmov select costs 1-2 cycles — 5-10x on the comparison-dominated loop; branchless partition sustains ~1 element/cycle where the branchy version averages ~5 cycles/element on random data; the cost trade (cmov adds a data dependency) only loses when the branch is predictable, which the writer chooses per site.
- **Par target**: Rust sort_unstable (pdq-derived) on random u64: ~1.5-2 ns·n; Khuong-style branchless binary search: ~2x over std::lower_bound on unpredictable queries at L2-resident arrays. Metric: cycles/element on uniformly random data.
- **Weight**: medium. **Substitutability**: Mostly one shape: a select/blend expression form with a lowering guarantee covers sorting, search, and clamping. Genuine split in guarantee strength only: performance code needs 'strong hint, compiler may still choose a branch when profitable'; constant-time crypto needs 'must never branch, verified'. One form with two strictness levels, the strict level being a small separate promise.

### 37. Software prefetch and non-temporal memory streams

- **Where**: Hash-join and hash-aggregate probe loops in databases (group prefetching, DuckDB/Hyper lineage), graph analytics neighbor expansion, B-tree/heap probe pipelining, GC marking loops, and NT stores in large memcpy/memset (glibc switches above ~L3-size) and write-once output buffers.
- **Contract**: Issue a prefetch hint for a computed address (semantically a no-op: never faults, never observable, address validity irrelevant); select non-temporal stores for large write-once regions; both must be expressible in ordinary loops without changing the loop's safety obligations.
- **SOTA shape**: Group/software-pipelined prefetching: compute N probe addresses, prefetcht0 all, then process N (keeping 8-16 misses in flight); glibc memcpy's NT-store path for copies above ~half of L3; rep movsb vs NT crossover tuning.
- **Why fast**: DRAM latency is ~100-300 cycles; a serial probe loop is latency-bound at 1 miss at a time, while 8-16 outstanding prefetches use the full memory-level parallelism of the core — 2-4x probe throughput on DRAM-resident tables; NT stores skip the read-for-ownership (halving bus traffic per store line, ~1.5-2x on huge fills) and avoid evicting the working set.
- **Par target**: Group-prefetching hash probe: 2-4x over the naive loop at >100M-entry DRAM-resident tables (Chen et al.-style results, reproduced in modern engines); NT-store memcpy matching glibc on >8MB copies. Metric: probes/sec at a DRAM-resident working set; GB/s on huge memset/memcpy.
- **Weight**: medium. **Substitutability**: Yes — prefetch is a single hint intrinsic that is safe by construction (hardware ignores bad addresses), and NT stores are a variant flag on the blessed copy/fill primitives; neither creates a new shape. The only demand is that the blessed loop forms admit the compute-ahead batching pattern, which is a scheduling idiom, not a new form.


## F7 concurrency

### 38. Thread lifecycle: spawn/join, worker pools, thread-local state

- **Where**: Every server (worker pools), allocators (mimalloc/jemalloc per-thread heaps and caches), rayon/TBB pool bootstrap, game-engine job systems with persistent workers, compilers parallelizing codegen units. Ubiquitous: essentially all multithreaded systems code starts here.
- **Contract**: Spawn a thread that either takes ownership of its captured state or borrows stack data with a proof it cannot outlive the borrow (scoped spawn); join returns a value or propagates failure; long-lived pools amortize spawn cost to zero per task; per-thread mutable state (caches, RNGs, scratch buffers) accessed without atomics and torn down at thread exit.
- **SOTA shape**: pthread/clone-backed std::thread plus crossbeam::scope-style scoped spawn; persistent worker pool; ELF initial-exec TLS (fs/gs segment-relative addressing) for per-thread state, lazy-init thread_local as the flexible variant.
- **Why fast**: Thread spawn is ~10-30us (stack mmap + clone syscall), so pools amortize it to ~0 per task. Initial-exec TLS access is a single fs:offset load, 1-2 cycles, zero atomics. Per-thread state eliminates all cache-coherence traffic: mimalloc's fast allocation path is TLS heap + local free list, ~20-30 cycles, zero shared cache lines touched.
- **Par target**: std::thread::spawn ~15us on Linux; crossbeam::scope for borrowed spawns; initial-exec TLS load ~1ns vs Rust thread_local! ~2-3ns (init-check branch); mimalloc per-thread malloc ~5ns as the consumer of this scenario.
- **Weight**: high. **Substitutability**: One scoped-spawn form plus one pool form covers nearly everything. TLS splits into static initial-exec (fast, no init branch, bounded set) vs lazy dynamic (flexible, +1 predictable branch): a single lazy form is at par for all consumers except allocator-grade inner loops, which need the raw static variant.

### 39. Atomic cells and the memory-ordering vocabulary

- **Where**: Refcounts (Arc/shared_ptr), shutdown flags, sequence counters, statistics counters in every server, and the internals of every lock/queue/map. Foundational: most code USES atomics through a handful of stereotyped idioms rather than the full C11 menu.
- **Contract**: A shared integer/pointer cell with load/store/fetch_add/compare_exchange; the key guarantee consumers actually invoke: data written before a release-store is visible after the matching acquire-load (publication), plus plain relaxed counting and CAS retry loops.
- **SOTA shape**: C11/LLVM atomics lowered to hardware: on x86, plain mov for acquire/release loads/stores and lock xadd/cmpxchg for RMW; on ARM, ldapr/stlr and LSE atomics. Contended-object designs pad hot atomics to their own 64B cache line.
- **Why fast**: Uncontended fetch_add ~1.5-7ns (one locked RMW on an M-state line); acquire/release costs zero extra instructions vs relaxed on x86; contended line ping-pong costs 40-100ns per op regardless of instruction choice. SeqCst adds an mfence-class fence (~20+ cycles) per store on x86 and dmb traffic on ARM, so a fixed all-SeqCst semantics would tax every hot atomic on ARM measurably.
- **Par target**: Rust core::sync::atomic / C++ std::atomic codegen parity: uncontended relaxed fetch_add ~2-7ns; CAS loop at 8-thread contention ~100-300ns/op; acquire/release identical to relaxed on x86.
- **Weight**: high. **Substitutability**: A small closed vocabulary — relaxed counter, release-publish/acquire-read pair, CAS-retry form — covers ~95% of source-level uses at par; writers do not need the six-ordering C11 menu. A SeqCst-only simplification would NOT be at par on ARM (per-op fences). The residual (fences, mixed-size atomics, seqlock reads) matters only to lock/queue implementers, which the language can absorb as shipped forms.

### 40. Mutual exclusion: mutex, rwlock, one-time init

- **Where**: SQLite (per-connection and global mutexes), servers protecting session/config state, lazy global initialization, allocator slow paths, virtually every multithreaded program. The single most-used concurrency primitive.
- **Contract**: A scoped guard granting exclusive (or shared) access to the protected data — lock-protects-data, not bare lock; blocking with parking (no busy-burn), condvar wait for state changes, one-shot initialization that is safe to race.
- **SOTA shape**: Word-sized futex-based adaptive mutex (parking_lot::Mutex, absl::Mutex, Rust std futex mutex since 1.62): one CAS on the uncontended path, bounded spin (~40-100 iterations) before futex_wait; lock word inline with the data it protects.
- **Why fast**: Uncontended lock+unlock is ~10-20ns: two uncontended RMWs on a line already owned by the acquiring core, zero syscalls, zero allocation; word-sized inline lock keeps lock word and protected data on the same cache line (one line touched total). Contended path parks via futex instead of burning cycles, keeping other cores' pipelines clean.
- **Par target**: parking_lot::Mutex ~12ns uncontended lock/unlock roundtrip; absl::Mutex comparable; std::sync::Mutex within ~1.2x; Once/lazy-static init check ~1ns after initialization (one acquire load + predicted branch).
- **Weight**: high. **Substitutability**: ONE word-sized adaptive mutex-protecting-data form covers the great majority at par. Genuine splits: (a) read-mostly hot data — classic rwlock still bounces a reader-count cache line, so heavy read paths need a seqlock/RCU-class snapshot-read shape instead (rwlock is NOT at par there); (b) no-OS/kernel contexts needing pure spinlocks. So: one mutex form + one read-mostly-snapshot form.

### 41. Queues and channels: SPSC ring vs MPMC channel

- **Where**: Audio/realtime pipelines (SPSC rings, rtrb-class), logging (MPSC), thread-pool task injection (Vyukov queue in tokio), inter-stage pipelines, crossbeam-channel throughout server code. Near-universal in multithreaded designs.
- **Contract**: send/recv of owned values between threads; bounded capacity with backpressure or unbounded; blocking recv integrated with OS parking; occasionally select over multiple channels; realtime consumers demand wait-free, allocation-free push/pop.
- **SOTA shape**: SPSC: single-producer ring with head/tail on separate padded cache lines and locally cached remote indices (rtrb, folly ProducerConsumerQueue). MPMC: Vyukov bounded queue (per-slot sequence numbers) or crossbeam-channel's segmented linked blocks with futex-parked blocking.
- **Why fast**: SPSC push is one plain data store + one release index store — no RMW instructions at all, ~5-10ns; cached-index trick means the producer reads the consumer's line only when its cached copy says full, so steady-state ops touch only producer-owned lines. MPMC fundamentally requires one fetch_add or CAS per op (~30-100ns under contention); Vyukov's per-slot seqno confines each op to one RMW plus one slot cache line.
- **Par target**: rtrb push/pop ~8ns/op; crossbeam-channel bounded ~60-100ns/op at 4 threads; Vyukov MPMC ~40-80ns/op; std::sync::mpsc within ~2x crossbeam after its 2022 rewrite.
- **Weight**: high. **Substitutability**: Genuinely splits. An MPMC channel used in an SPSC role is 5-10x slower than a true SPSC ring — it cannot drop the RMWs without knowing endpoints are unique. Minimum catalog: (a) bounded ring with single-producer/single-consumer specialization, (b) general blocking MPMC channel. A single channel form could unify at par ONLY if the compiler can prove endpoint uniqueness — which is precisely a linear/unique-ownership fact, so unification is plausible in a language with linear endpoint types.

### 42. Concurrent keyed state: sharded maps and read-mostly tables

- **Where**: Session tables in servers, string interners in compilers, dashmap/folly ConcurrentHashMap consumers, routing/config tables read millions of times per second; also the shard-by-thread substitute (per-thread map + merge) in profilers and stats pipelines.
- **Contract**: get/insert/remove from many threads; read-heavy consumers need reads that write NO shared memory (else scaling dies); occasional whole-table snapshot or iteration; consumers rarely need cross-key transactional consistency.
- **SOTA shape**: Mixed workloads: 2^k-way sharded SwissTable, shard picked by hash bits, per-shard mutex (dashmap). Read-dominated: open-addressing map with lock-free epoch/hazard-protected readers (folly ConcurrentHashMap, flurry) or full read-copy snapshot (evmap/arc-swap of an immutable map).
- **Why fast**: Sharding divides contention: at 16 shards / 8 threads most ops take an uncontended shard lock (~15ns) plus one SwissTable probe (1-2 cache lines, 16-way SIMD tag scan). Lock-free readers write zero shared cache lines — no reader count — so read throughput scales linearly with cores where any lock-based reader plateaus on line bouncing.
- **Par target**: dashmap 6.x get ~20-40ns uncontended, ~50-100M mixed ops/s across 16 threads; folly ConcurrentHashMap wait-free reads ~15ns; arc-swap snapshot load ~2ns per read for read-copy tables.
- **Weight**: medium. **Substitutability**: Splits on read/write ratio. Sharded-lock map is at par for mixed workloads; read-mostly hot paths need reader-writes-nothing designs the sharded map cannot match. Two blessed shapes suffice: sharded map + snapshot/read-copy map. The shard-by-thread pattern (per-thread map, merge on demand) is a third idiom but composes from thread-local + plain map, needing no new form.

### 43. Safe memory reclamation (epoch/hazard/RCU class)

- **Where**: IMPLEMENTED in only a handful of places — crossbeam-epoch, folly hazptr, Linux kernel RCU, libcds; USED transitively by every lock-free map/queue/list that frees nodes. Application code virtually never implements it; this is the clearest implement-vs-use asymmetry in systems programming.
- **Contract**: Read a linked/shared structure while other threads unlink and free parts of it, with a hard guarantee that freed memory is never dereferenced. Consumers want this invisible — at most a scoped read-guard (pin) whose cost is a few ns; they never want to write retire lists or epoch advancement.
- **SOTA shape**: Epoch-based reclamation (crossbeam-epoch: thread-local epoch stamp, global epoch, per-thread deferred-free bags); hazard pointers (folly: per-thread protected-slot array) where garbage must stay bounded despite stalled readers; quiescent-state RCU in kernels for literally-free readers.
- **Why fast**: Epoch pin is a thread-local store plus a compiler fence, ~1-3ns, no shared RMW — readers touch zero shared cache lines, so read-side scaling is perfectly linear. Frees are deferred and batched, amortizing reclamation to well under 1ns per read. Hazptr protect is store+load+fence ~4-8ns but bounds unreclaimed memory. RCU read-lock compiles to nothing.
- **Par target**: crossbeam-epoch pin ~2ns; folly hazptr protect ~5ns; Linux RCU read_lock 0 instructions (barrier only). The par bar is: read-side cost of a shipped concurrent structure must match crossbeam-epoch-backed equivalents.
- **Weight**: medium. **Substitutability**: The strongest hide-it candidate in F7: if the language ships concurrent maps/queues as blessed forms, reclamation lives entirely inside their implementation (proof or disciplined TCB) and NO user-facing reclamation form is needed — matching how 99% of C++/Rust programmers already live. If users may build their own linked lock-free structures, one guard-scoped epoch form suffices; the epoch-vs-hazptr split matters only for bounded-memory-under-stalled-reader requirements (kernel-adjacent), acceptable to exclude from a small catalog.

### 44. Data parallelism: parallel-for / fork-join with work stealing

- **Where**: rayon in ripgrep/rustc/polars, Intel TBB, OpenMP loops in codecs and scientific kernels, parallel sort/map/reduce over large arrays, game-engine job graphs. The default way CPU-bound work exploits cores.
- **Contract**: Parallel iterate/map/reduce over a splittable collection; nested fork-join (recursive divide-and-conquer) with automatic load balancing; share read-only input and mutate provably disjoint chunks; deterministic reduction results when asked.
- **SOTA shape**: Chase-Lev work-stealing deque per worker plus recursive geometric splitting (rayon join, TBB task arena); block sizes chosen so per-task work far exceeds steal cost; thieves steal from the top, owners push/pop the bottom.
- **Why fast**: Owner-side deque push/pop needs no RMW (single owner, one release store; only the rare steal path pays a CAS), so steady-state task overhead is ~20-50ns amortized and shared-line traffic is near zero. Geometric splitting makes steals logarithmically rare, so throughput becomes memory-bandwidth-bound: a parallel sum over 100M u64 saturates socket DRAM bandwidth with near-linear core scaling.
- **Par target**: rayon par_iter().sum() over 100M u64 reaches ~25-40GB/s socket DRAM bandwidth; rayon::join fork overhead ~25ns on a warm pool; TBB parallel_for equivalent.
- **Weight**: high. **Substitutability**: ONE parallel-for/fork-join form over splittable collections covers most data parallelism at par — rayon proves this is achievable as a library over threads+deques. Genuine splits: pipeline parallelism (stages connected by queues) and latency-sensitive job systems wanting priorities/affinity (game engines) do not fit parallel-for and fall back to the thread/channel forms; those are compositions, not new primitives.


## F8 external boundary

### 45. Buffered sequential file I/O

- **Where**: Every CLI tool, compilers reading sources, linkers, log writers, grep-class scanners (ripgrep), serialization dumps. The single most common I/O shape in systems code.
- **Contract**: Open a file, read/write a byte stream with app-defined record structure, amortized syscalls, explicit flush, precise error reporting. High-end consumers also want to borrow the internal buffer (fill_buf-style) to parse in place without a second copy.
- **SOTA shape**: 64-256KB user-space buffer over pread/pwrite (Rust BufReader/BufWriter, stdio with enlarged buffers) riding kernel readahead; databases split off to O_DIRECT with aligned buffers and their own cache.
- **Why fast**: A syscall costs ~300-600ns with mitigations; a 64KB buffer amortizes that to ~0.01ns/byte. In-buffer reads are memcpy at 10-30GB/s, and borrow-the-buffer parsing eliminates even that copy. Warm page-cache sequential read runs 5-10GB/s single-threaded; cold reads hit device bandwidth via readahead.
- **Par target**: ripgrep buffered scanning >2GB/s per core on warm cache; fio buffered sequential read = NVMe device bandwidth (3-7GB/s); Rust BufReader within noise of C stdio at equal buffer size.
- **Weight**: high. **Substitutability**: One buffered-stream form with a borrow-the-buffer read covers the vast majority at par. Genuine split: database engines need O_DIRECT + alignment + own caching (double-copy through page cache and eviction interference are unacceptable — RocksDB/Postgres/SQLite paths), so a second unbuffered aligned-I/O form is required for that audience.

### 46. Network sockets: TCP/UDP request paths

- **Where**: Every server and proxy (nginx, envoy, redis), RPC stacks, database wire protocols, message brokers. Defines the server half of the audience.
- **Contract**: Accept connections, read request bytes, write response bytes across tens of thousands of concurrent connections; backpressure, timeouts, graceful shutdown; zero per-request allocation via buffer reuse.
- **SOTA shape**: Nonblocking sockets in an event loop; writev/sendmsg vectored writes; TCP_NODELAY/cork discipline; SO_REUSEPORT accept sharding per core; io_uring multishot recv/send on newest stacks; recvmmsg/GSO batching for UDP/QUIC.
- **Why fast**: The budget is syscalls and copies: a naive request costs one recv + one send (~1us of syscall time). Vectored writes merge header+body into one syscall; io_uring multishot drops syscalls per request below 1; per-core REUSEPORT sharding removes the shared accept queue and cross-core cache-line traffic; reused buffers make the steady state allocation-free.
- **Par target**: redis ~100-200k GET/s per core on epoll; io_uring echo servers exceed 1M msgs/s/core in axboe-class benchmarks; nginx ~100k+ plaintext req/s/core.
- **Weight**: high. **Substitutability**: One nonblocking-socket-in-event-loop form (with vectored writes and buffer reuse) serves most servers at par. Splits: UDP/QUIC needs batching syscalls (recvmmsg, GSO) as a distinct sub-shape; kernel-bypass (DPDK/XDP) is a real but niche tier reasonable to exclude from a small catalog.

### 47. Readiness/completion multiplexing (epoll/kqueue/io_uring/IOCP class)

- **Where**: IMPLEMENTED by a handful of runtimes — tokio/mio, libuv, folly EventBase, seastar, boost.asio; USED transitively by essentially all scalable network software. Application code should never implement this layer.
- **Contract**: The consumer contract is a scheduler, not epoll: run thousands of I/O operations concurrently, wake my logic when each completes, integrate timers/deadlines, cancel cleanly. Crucially the buffer for an in-flight op is loaned to the kernel until completion — an ownership fact.
- **SOTA shape**: Readiness: edge-triggered epoll/kqueue + nonblocking retry (mio). Completion: io_uring shared-memory submission/completion rings with batched submit, optional SQPOLL, registered fixed buffers/files; IOCP on Windows.
- **Why fast**: epoll_wait harvests up to 64 events per syscall; io_uring submits N ops in one io_uring_enter (or zero syscalls with SQPOLL) and completions are read from shared memory — syscalls per op approach 0. Registered buffers/files skip per-op refcount and address translation. Batch dispatch keeps the icache/branch predictor on one hot loop.
- **Par target**: mio/tokio epoll loop drives hundreds of k events/s/core; io_uring roughly 2x epoll on syscall-bound echo workloads; liburing NOP throughput >10M ops/s/core as the mechanism ceiling.
- **Weight**: high. **Substitutability**: The prime language-ships-it case in F8: ONE completion-style blessed interface (submit a set of ops, await completions) can lower to io_uring where available and epoll+nonblocking emulation elsewhere at par — libuv and tokio prove the portability layer. Readiness-style exposure is needed only for integrating foreign event loops. The safety crux is the buffer-loaned-until-completion lifetime, a linear-ownership fact the language can make nonforgeable.

### 48. mmap and shared memory

- **Where**: LMDB (entire DB read through a map), SQLite's optional mmap mode, ripgrep on large files, mold/lld mapping inputs and writing output in place, allocators reserving arenas, kernel ring interfaces (perf buffers, io_uring rings themselves).
- **Contract**: Map a file or anonymous region as a byte slice; random access at page-cache speed with zero syscalls after fault-in; alignment/hugepage control for arenas; for IPC, two processes sharing writable memory with an access discipline.
- **SOTA shape**: mmap PROT_READ MAP_PRIVATE with madvise(SEQUENTIAL/RANDOM/WILLNEED); anonymous 2MB-aligned reservations with MADV_HUGEPAGE for allocator arenas; MAP_SHARED rings for IPC with atomic index protocols.
- **Why fast**: After fault-in, access is pure loads: zero syscalls and zero copies versus read()'s copy-per-buffer; soft fault ~1-3us amortized over 4KB-2MB of data; hugepages cut TLB misses ~512x on big arenas; mold writes the output file through a map, eliminating an entire write pass; page cache is shared across processes for free.
- **Par target**: LMDB read-txn get ~100-200ns (pure pointer-chase in mapped memory, no syscalls); mold links clang-scale outputs in ~1-2s largely on mmap discipline; ripgrep mmap mode matches buffered reads warm and wins on very large files.
- **Weight**: medium. **Substitutability**: Cannot be one shape. (a) Read-only private map of an agreed-immutable file blesses cleanly as an immutable byte-slice form — the residual hazard is SIGBUS on external truncation, which the language must own (file locking, fallback copy, or a defined trap). (b) Writable MAP_SHARED for IPC is semantically different: an external writer breaks any single-process aliasing proof, forcing an atomic/volatile access discipline. Two forms minimum; anonymous arena reservation can hide inside the allocator.

### 49. Durability ordering: fsync discipline and write-ahead logging

- **Where**: SQLite, Postgres, RocksDB, etcd, mail queues, package managers (atomic rename), anything that claims crash safety. Concentrated in persistence layers, but those layers anchor the database audience.
- **Contract**: "These bytes are on stable storage before that state becomes valid": WAL append then single fdatasync per commit batch; data-fsync-before-commit-record ordering; atomic file replacement (write temp, fsync file, rename, fsync directory); checksummed frames so torn tails are detectable.
- **SOTA shape**: SQLite WAL / Postgres group commit: append frames, one fdatasync amortized over N transactions; fdatasync over fsync (skips inode metadata); O_DIRECT|O_DSYNC or FUA writes on NVMe for single-syscall durable writes.
- **Why fast**: fsync is the wall: ~20-100us on NVMe, 1-5ms on SATA SSD — thousands of times a write. SOTA wins by minimizing fsync count, not write speed: group commit turns 1-fsync-per-txn into 1-per-batch, taking SQLite from ~250 commits/s (synchronous rollback journal, SATA-class) to tens of thousands per second batched in WAL mode.
- **Par target**: SQLite 3.45 WAL mode: ~20-50k commits/s batched on NVMe vs ~100-250/s naive per-commit fsync; Postgres group commit shows the same shape.
- **Weight**: medium. **Substitutability**: Most consumers want the pattern, not the syscalls: a blessed durable-append-log form (WAL with group commit and checksummed frames) plus an atomic-file-replace form covers the majority at par. Engines with custom layouts (Postgres heap+WAL, RocksDB SSTs) still need the thin low-level layer — per-fd fdatasync, directory sync, write-ordering control. Two high-level forms plus one low-level escape; the low-level layer is small but non-optional.

### 50. FFI call boundary (calling and being called by C)

- **Where**: Everywhere real systems live: libc/syscall wrappers, OpenSSL, GPU APIs (Vulkan/CUDA), SQLite embedding, codec libraries, plugin ABIs. A systems language without at-par FFI is locked out of most existing infrastructure.
- **Contract**: Call C functions at native cost with C-ABI-layout structs; expose callbacks C can invoke; pass buffers as pointer+length with explicit ownership/lifetime agreements; no marshaling, no thread-state save/restore, no allocation per call.
- **SOTA shape**: Direct C-ABI call as in Rust extern "C" / C++: a bare call instruction honoring the platform ABI, #[repr(C)] layout compatibility. The anti-patterns quantify the stakes: Go cgo ~40-60ns per call (stack switch, scheduler interaction), JNI ~100ns+ with pinning bookkeeping.
- **Why fast**: Zero-cost means the machine code is identical to a C-to-C call (~1-2ns overhead): arguments in ABI registers, no copying, no pinning, no runtime bookkeeping; buffers cross as pointer+len because layout is bit-compatible, so no serialization exists at all.
- **Par target**: Rust extern "C" call overhead ~1-2ns (a call instruction plus ABI register shuffles); Go cgo's ~40ns is the documented anti-target; a hot loop calling BLAS or a codec must be indistinguishable from C driving it.
- **Weight**: high. **Substitutability**: One extern-call form suffices mechanically — the machine shape does not split. The real axis is trust, not performance: FFI declarations are unavoidably TCB (no checker sees the C side), so the blessed shape is a boundary declaration carrying stated ownership/aliasing/lifetime contracts subject to audit. A checked-source alternative cannot exist here by definition; the discipline is containing and auditing the trusted surface.

### 51. Process and OS services: spawn, time, entropy, signals

- **Where**: Build systems (ninja spawning thousands of compilers), shells, supervisors, test runners; monotonic time in every profiler, scheduler, and timeout path; entropy for hash seeding (SipHash keys); signals consumed mostly as shutdown notice.
- **Contract**: Spawn a process with controlled fds/env/cwd, wait for exit status, wire pipes; read monotonic and wall-clock time at negligible cost from hot paths; obtain entropy; install a shutdown handler. Consumers want a facade, not fork semantics.
- **SOTA shape**: posix_spawn (vfork-backed) instead of fork+exec, avoiding page-table copy of a large parent; vDSO clock_gettime with no kernel entry; getrandom-seeded userspace ChaCha buffer for cheap per-call randomness.
- **Why fast**: fork of a multi-GB-RSS parent costs milliseconds of page-table copying; posix_spawn/vfork is ~50-100us independent of parent size — ninja's job throughput rests on this. vDSO clock_gettime is ~20-25ns with zero syscalls, safe to call millions of times per second; buffered userspace entropy is ~ns versus hundreds of ns per getrandom syscall.
- **Par target**: posix_spawn ~60us regardless of parent RSS vs millisecond-scale fork+exec from a 1GB+ parent; clock_gettime(CLOCK_MONOTONIC) ~25ns via vDSO; getrandom syscall ~200-300ns vs ~2-5ns from a userspace CSPRNG buffer.
- **Weight**: low. **Substitutability**: One std-facade covers nearly all uses at par: the performance-differentiating choices (spawn mechanism, vDSO clocks, buffered entropy) are implementation decisions inside the facade, invisible to consumers. No meaningful split — the clearest single-blessed-shape scenario in either family.
