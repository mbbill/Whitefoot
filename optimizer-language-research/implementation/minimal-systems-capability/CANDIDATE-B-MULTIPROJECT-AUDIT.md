# Candidate B Multiproject Source Audit

Date: 2026-07-15

Phase 1 disposition: `SOURCE-AUDIT-COMPLETE`.

This is a read-only structural audit of exactly fourteen operations. It does
not score a Candidate B architecture, prove safety, inspect generated code, or
measure performance.

## 1. What adding three projects changed

Hashbrown alone asks whether control metadata can safely authorize sparse
payload without initializing every slot. The other projects show that this is
only one instance of a wider problem:

- mimalloc changes the interpretation of the same block from free-list node to
  uninitialized caller storage to local or atomically published free-list node;
- SQLite derives variable byte ranges from page metadata, retains references
  across several scratch and cache roots, and deliberately returns errors that
  require transaction-level rollback rather than local inversion; and
- Crossbeam makes protected loads with no added per-load guard event, then moves
  one unique retirement right through unrelated batching roots before a
  library-selected grace-period policy permits destruction.

The common question is therefore not whether B needs a `Sparse` keyword. It is
whether a small set of rules can express physical places, state-dependent
interpretation, exact owner traffic, scoped partial progress, physical-root
provenance, executable cleanup, and concurrent observation without naming any
of these four projects.

## 2. Pinned sources

| Project | Pinned source | Why it is in the audit |
|---|---|---|
| Hashbrown | v0.17.1, commit [`c62a63a61b7caf2de8f9ecb7b06a66b0ab6bdf3d`](https://github.com/rust-lang/hashbrown/tree/c62a63a61b7caf2de8f9ecb7b06a66b0ab6bdf3d) | Sparse control/payload admission, direct owner replacement/removal, and partial rehash. |
| mimalloc | v3.3.2 annotated tag object `5687270e7fbb15d494a46b0d048f978bad973e4f`, source commit [`30b2d9d89099bee08e9f67a1ffb3e12e7ba45227`](https://github.com/microsoft/mimalloc/tree/30b2d9d89099bee08e9f67a1ffb3e12e7ba45227) | Uninitialized block payload, intrusive overlays, suballocation, local versus atomic owner transfer, and page lifecycle. |
| SQLite | 3.53.3, canonical Fossil source ID `d4c0e51e4aeb96955b99185ab9cde75c339e2c29c3f3f12428d364a10d782c62`, official mirror commit [`92a6c5c3636faa021ecc3be5403a00f50f65eda7`](https://github.com/sqlite/sqlite/tree/92a6c5c3636faa021ecc3be5403a00f50f65eda7) | Variable packed ranges, multi-root partial progress, bounded recursive topology, and rollback policy. |
| Crossbeam Epoch | 0.9.18, commit [`9c3182abebb36bdc9446d75d4644190fef70fa01`](https://github.com/crossbeam-rs/crossbeam/tree/9c3182abebb36bdc9446d75d4644190fef70fa01/crossbeam-epoch) | Guard-scoped access, unique retirement, cross-root delayed disposition, quiescence, and policy-defined reclamation. |

The mimalloc annotated tag was dereferenced before routing. For SQLite, the
Fossil identity is canonical and the Git identity is the retrieval pin.

## 3. The fourteen concrete operations

| ID | Operation | Native shape that must remain expressible |
|---|---|---|
| `H-LOOKUP` | Hashbrown lookup | One FULL control observation authorizes one payload; the probe adds no payload traffic on misses. |
| `H-INSERT` | Hashbrown vacant insertion | Offered owners remain outside until one logical commit writes the selected uninitialized slot. |
| `H-REPLACE` | Hashbrown replacement | Stored key stays, new value moves in, old value moves out, and the duplicate key is destroyed once. |
| `H-REMOVE` | Hashbrown removal | One owner moves out while control becomes EMPTY or DELETED without zeroing the payload. |
| `H-REHASH` | Hashbrown resize or in-place rehash | A transition-local DELETED state may still contain an unprocessed live owner; it must not escape as ordinary state. |
| `M-ALLOC` | mimalloc small allocation | Pop one page-local free node and return the rest of the block uninitialized; no permanent per-block tag or atomic event. |
| `M-LOCAL-FREE` | mimalloc local free | Consume one block owner and overlay its first word with a local free-list link using non-atomic page authority. |
| `M-REMOTE-FREE` | mimalloc remote free and collection | A successful CAS transfers one block owner; bounded later collection and page disposition use the same physical root across metadata moves. |
| `S-INSERT-SPLIT` | SQLite B-tree insert and split | Exact cell subranges move among pages and scratch roots; failure may require pager rollback rather than local restoration. |
| `S-DELETE-BALANCE` | SQLite delete and rebalance | Cell and overflow-page ownership moves or is freed exactly; ordinary mode retains stale bytes instead of zeroing them. |
| `S-ROLLBACK` | SQLite pager rollback | Old pages stream from journal or WAL through one scratch page; failed replay enters a persistent unusable state. |
| `X-PROTECTED-LOAD` | Crossbeam pin and load | Pin pays domain work once; each protected load is only the caller-selected atomic load, while publication and interference remain separate. |
| `X-RETIRE` | Crossbeam unlink and defer destroy | Successful unlink creates one disposition right even though Rust's address value is copyable; batching must not re-root it. |
| `X-COLLECT` | Crossbeam advance and collect | A live-prefix bag, participant scan, bounded queue work, and one-shot callbacks implement one EBR policy without making EBR language law. |

The TSV companion gives the exact source anchors, owner account, provenance,
events, forbidden deltas, unknowns, and falsifier for every row.

## 4. Evidence by project

### 4.1 Hashbrown: metadata is evidence, not ownership

Hashbrown stores control bytes and payload in separate regions of one
allocation. A FULL byte with the right tag is only the first step: it admits an
equality callback on the corresponding live payload. EMPTY ends a probe and
DELETED preserves reachability. An integer bucket index or hash grants no
payload authority.

Insertion, replacement, removal, and rehash show why uninitialized storage
alone is insufficient. The language also needs exact owner transitions and a
logical commit boundary. In-place rehash is the decisive case: its temporary
DELETED state means "unprocessed but live," while ordinary DELETED means
"vacant tombstone." The same bits have different authority only inside a
non-escapable protocol.

Key anchors are [`RawTableInner` and control state](https://github.com/rust-lang/hashbrown/blob/c62a63a61b7caf2de8f9ecb7b06a66b0ab6bdf3d/src/raw.rs#L557-L580),
[`find_inner`](https://github.com/rust-lang/hashbrown/blob/c62a63a61b7caf2de8f9ecb7b06a66b0ab6bdf3d/src/raw.rs#L2008-L2044),
and [in-place rehash](https://github.com/rust-lang/hashbrown/blob/c62a63a61b7caf2de8f9ecb7b06a66b0ab6bdf3d/src/raw.rs#L2984-L3076).

### 4.2 mimalloc: a place can change layout without acquiring a tag

A mimalloc free block uses its first word as an intrusive next pointer. The
rest of the block can remain uninitialized. Allocation pops the node, clears
only the internal pointer in the ordinary release path, and returns a unique
raw block. Local free consumes that owner and writes the next pointer back.

Remote free is materially different. Before a successful CAS, the caller still
owns the block and may rewrite its unpublished link after interference. The
successful CAS is the owner-transfer commit. Collection later detaches a list,
walks at most the page capacity, updates `used`, and chooses whether to release,
reclaim, re-advertise, or unown an abandoned page.

Page metadata can live separately from payload storage, and heap association
can change. The stable provenance root is the physical page allocation, not the
page-map slot, metadata address, heap, or thread identifier.

Key anchors are [the allocation fast path](https://github.com/microsoft/mimalloc/blob/30b2d9d89099bee08e9f67a1ffb3e12e7ba45227/src/alloc.c#L29-L115),
[local and remote free](https://github.com/microsoft/mimalloc/blob/30b2d9d89099bee08e9f67a1ffb3e12e7ba45227/src/free.c#L26-L87),
and [bounded remote-list collection](https://github.com/microsoft/mimalloc/blob/30b2d9d89099bee08e9f67a1ffb3e12e7ba45227/src/page.c#L142-L260).

### 4.3 SQLite: safe failure does not always mean local rollback

One SQLite B-tree page is a byte buffer containing a header, cell-pointer
array, gap, variable cell bodies, freeblocks, fragments, and overflow links.
Access comes from initialized derived metadata plus checked subranges. Coarse
whole-page borrowing would force copies that the source avoids by updating
pages in a dependency order and preserving nonoverlapping ranges.

Insertion and deletion may modify multiple pages before a later allocation,
pointer-map, or I/O error. Local cleanup releases every page pin and scratch
allocation, but it does not reconstruct the old B-tree. The valid result is a
non-public "rollback required" state owned by the pager transaction. Requiring
every local function to install inverse closures would add allocations, owner
records, and work absent from SQLite.

Rollback then streams original page images from a journal or WAL through one
scratch page, overwrites existing cache allocations, invalidates every derived
page fact, and returns to reader state only after successful finalization. A
failed rollback enters `PAGER_ERROR`; it must not escape as a usable database.

Key anchors are [general page balancing](https://github.com/sqlite/sqlite/blob/92a6c5c3636faa021ecc3be5403a00f50f65eda7/src/btree.c#L8245-L9027),
[insert](https://github.com/sqlite/sqlite/blob/92a6c5c3636faa021ecc3be5403a00f50f65eda7/src/btree.c#L9409-L9709),
[delete](https://github.com/sqlite/sqlite/blob/92a6c5c3636faa021ecc3be5403a00f50f65eda7/src/btree.c#L9841-L10040),
and [pager rollback](https://github.com/sqlite/sqlite/blob/92a6c5c3636faa021ecc3be5403a00f50f65eda7/src/pager.c#L6788-L6857).

### 4.4 Crossbeam: access, retirement, and reclamation are three relations

Crossbeam's `Atomic::load` accepts a guard but ignores its value at runtime.
Pinning has already paid the participant and barrier cost; the load itself is
just the requested atomic load. The guard delays reclamation but does not prove
that initialization was published, that the guard belongs to the right
collector, or that payload mutation is race-free.

After successful unlink, one conceptual right may retire the allocation.
Rust's `Shared` address is Copy, so Rust leaves exactly-once retirement and
unreachability as unsafe obligations. The disposition then moves through a
participant bag and global queue node without changing its target allocation
root.

Collection scans participants, advances epochs, pops at most eight eligible
bags in an ordinary call, and invokes only each bag's active prefix. Two-epoch
expiry, 64-entry bags, the eight-pop limit, and the 128-pin trigger are
Crossbeam policy. They must not become the only language-level reclamation
policy.

Key anchors are [protected load](https://github.com/crossbeam-rs/crossbeam/blob/9c3182abebb36bdc9446d75d4644190fef70fa01/crossbeam-epoch/src/atomic.rs#L366-L384),
[deferred destruction](https://github.com/crossbeam-rs/crossbeam/blob/9c3182abebb36bdc9446d75d4644190fef70fa01/crossbeam-epoch/src/guard.rs#L199-L270),
and [epoch collection](https://github.com/crossbeam-rs/crossbeam/blob/9c3182abebb36bdc9446d75d4644190fef70fa01/crossbeam-epoch/src/internal.rs#L155-L266).

## 5. Project-independent demands

| Demand | Repeated evidence | Performance reason |
|---|---|---|
| Physical roots and partitioned places | Hashbrown table slots, mimalloc page blocks, SQLite page/cache ranges, Crossbeam pointee allocations | Avoid per-element allocation, backpointers, wrapper-rooted references, and relocation. |
| State-indexed interpretation | Hashbrown DELETED-live phase, mimalloc payload/free-node overlay, SQLite cell/freeblock bytes, Crossbeam queue sentinel | Avoid permanent tags, dummy values, eager initialization, and container-specific layout rules. |
| Closed metadata-to-place admission | Hashbrown control bytes, mimalloc list or page state, SQLite pointer arrays, Crossbeam participant and bag prefixes | Admit only exact live ranges without a universal bitmap, scan, or arbitrary writer predicate. |
| Direct affine transitions | Insert/replace/remove, block pop/push/CAS, page-pin and overflow transfer, retirement disposition | Avoid clone, extra move, temporary box, reference count, and owner reconstruction. |
| Scoped partial progress | Rehash, abandoned-page collection, B-tree balance plus rollback-required state, bounded collection | Avoid full snapshots, inverse closures, persistent protocol tags, and forced local rollback. |
| Physical-root provenance | Table growth, separate page metadata, cache/scratch rekeying, target through deferred bags | Avoid runtime borrow tables, dynamic root checks, per-block backpointers, and wrong-root use. |
| Executable exact cleanup | Rehash guards, bounded remote-list collection, page-pin release and journal replay, active-prefix callbacks | Avoid full-capacity scans, leaks, duplicate destruction, and a cleanup special case per container. |
| Shared access separated from reclamation policy | mimalloc local/remote paths and abandoned readers; Crossbeam guard, retire, and epoch collection | Preserve local non-atomic and per-load fast paths while permitting different reclamation strategies. |
| Exact machine and external leaves | Hashbrown group loads, mimalloc atomics/page map, SQLite VFS/journal/WAL, Crossbeam memory ordering | Avoid hidden extra loads, fences, syncs, allocations, and backend-specific unsound assumptions. |

These are demands, not the selected B capability set. The design phase must
still show that a closed grammar can express them without converging toward
Candidate A or Candidate C.

## 6. Important boundaries

1. Unsafe source reveals an obligation; it does not justify an unchecked xlang
   primitive.
2. SQLite's B-tree logical correctness and crash consistency remain ordinary
   library properties. The language must make memory, owner, protocol, and
   external-event misuse unrepresentable without proving SQL semantics.
3. Crossbeam's reclamation policy does not prove publication or payload
   interference safety. Those relations stay separate.
4. Source comments about instruction count or speed are not measurements in
   this audit.
5. Exact VFS, WAL, platform page-map, portable atomic, generated-code, formal
   safety, progress, and callback-termination questions remain unresolved.
6. Four projects and fourteen operations are a bounded falsification set, not
   ecosystem-completeness evidence.

## 7. Phase result

The source evidence is sufficient to compare the three frozen Candidate B
architectures. It does not yet support `B-SELECT`, `B-REVISE`, or `B-NONE`.
Those dispositions belong only to the 42-row design comparison.
