# External container workload traces

This evidence belongs to the container architecture experiment in this directory.
Maintain it through the implementation selected in [REASSESSMENT.md](REASSESSMENT.md);
merge or retire it when a replacement investigation supersedes these claims. It
supplements the local compiler and Whitefoot fixtures. It does not define the
language or establish workload prevalence.

## Selection and evidence boundary

The sample covers three substantial application subsystems in three languages and
domains. They were selected to challenge different assumptions: contiguous search
windows, columnar query execution, and indexed scheduling state. The six traces
are deliberately related within each application; they are not six independent
observations of industry practice. Static occurrences, repository size, and the
existence of a fast application are not measurements of container hotness.

| Application and domain | Language | Pinned release commit |
| --- | --- | --- |
| ripgrep, text search | Rust | 14.1.1: [`4649aa9700619f94cf9c66876e9549d83420e16c`](https://github.com/BurntSushi/ripgrep/tree/4649aa9700619f94cf9c66876e9549d83420e16c) |
| DuckDB, analytical database execution | C++ | v1.2.0: [`5f5512b827df6397afd31daedb4bbdee76520019`](https://github.com/duckdb/duckdb/tree/5f5512b827df6397afd31daedb4bbdee76520019) |
| Kubernetes, scheduler state | Go | v1.32.0: [`70d3cc986aa8221cd1dfb1121852688902d3bf53`](https://github.com/kubernetes/kubernetes/tree/70d3cc986aa8221cd1dfb1121852688902d3bf53) |

The release references were resolved with `git ls-remote`, using the peeled commit
for annotated tags. All code links below use those commits, and line numbers were
read from their raw source. These are reproducible snapshots, not claims about
the latest release. No upstream build, benchmark, production profile, allocation
trace, or Whitefoot translation was run for this study.

An **observed** operation follows source and its local callers. A cost statement
describes explicit work or an algorithmic consequence, unless marked as an
upstream comment. It is not a timing result. A **Whitefoot hypothesis** must still
preserve behavior, lifetime, refusal, and resource contracts and be checked and
measured. Upstream assertions, unchecked C++ operations, Rust's library internals,
and Go's runtime are not proof authority for Whitefoot.

A follow-up direct source reread checked three consequential claims before their
integration into the architecture and memory records:

- EW2: `list_value.cpp` lines 28–53 write a child payload only on the valid arm,
  mark the other arm invalid, and publish a logical size covering both arms.
  [Producer][dd-list]. This does not establish the prior contents of reused NULL
  slots or the performance of bitmap versus enum storage.
- EW4: `ht_entry.hpp` lines 39–89 and `aggregate_hashtable.cpp` lines 647–750
  establish a nonzero reserved entry before row append/state initialization and
  later pointer installation. [Entry][dd-entry], [publication][dd-agg-find]. The
  reread checks phase ordering, not every collision path or exception guarantee.
- EW6: `active_queue.go` lines 92–114, 287–304, and 351–386 establish UID markers,
  suffix event retrieval, marker removal, and prefix pruning up to the next
  remaining marker. [State][k-events-state], [retrieval][k-events], [retirement][k-done].
  No concurrent execution, GC behavior, or bound on retained history was measured.

The checks used fresh retrievals of the pinned primary source, not the report's
summaries. They introduced no new upstream or Whitefoot execution evidence.

| Trace | Distinguishing demand |
| --- | --- |
| EW1: search window | Retained context, incomplete suffix, reusable backing, configured growth refusal |
| EW2: nullable nested columns and selection | Logical extent, payload initialization, validity, and selected rows are different domains |
| EW3: string result construction | Inline values and stable backing coexist; reserve, populate, and publish are separate phases |
| EW4: grouped aggregation | Sparse reservation states, pinned payloads, rehash without moving rows, many inputs targeting one result |
| EW5: indexed priority queue | Heap order and key lookup share a cross-container correspondence |
| EW6: shared event history | Stable markers, interior retirement, and reclamation determined by the oldest reader |

## EW1: ripgrep retains a search window, not just a byte prefix

**Observed trace.** Build a normally 64 KiB, zero-initialized `Vec<u8>`.
[Construction][rg-build]. A reader
borrows the buffer exclusively and resets its cursors. Fill spare space until a
complete searchable line exists; hold an incomplete final line separately. Search
the contiguous borrowed slice. Before the next fill, retain the context still
needed for subsequent matches, consume the earlier bytes, and move the retained
suffix to offset zero. Grow only when no writable space remains. At exhaustion,
make the final unterminated line searchable. The buffer can be reset and reused
without reallocating or zeroing its contents. These steps occur in the actual
[search loop][rg-glue], [context-retention calculation][rg-context], and
[buffer state and accessors][rg-state].

The relevant relation is `0 <= pos <= last_lineterm <= end <= buf.len()` at the
search interface. `[pos,last_lineterm)` is searchable; `[last_lineterm,end)` may
contain an incomplete line. Allocated and initialized bytes beyond `end` are not
current input. The absolute source offset is a separate logical coordinate,
updated on consumption rather than obtained from a memory address. Borrowed
slices cannot survive the next mutable roll or growth operation. The configured
binary mode also changes which bytes may become visible; a replacement cannot
quietly expose discarded bytes. [State][rg-state], [fill][rg-fill].

**Resources and costs.** Rolling explicitly copies the retained suffix; clearing
an exhausted window only resets indices. Growth resizes and zeroes new storage.
The eager policy adds twice the current length, yielding three times the prior
length when it is nonzero; this is an observed policy, not a required growth
factor. A configured cap on *additional* allocation returns an error when a line
or retained context cannot fit. This is distinct from a recoverable allocator
OOM contract, which this code does not provide. [Policy][rg-policy],
[roll and growth][rg-growth]. No evidence here establishes that zeroing, rolling,
or allocation dominates real search time.

**Whitefoot hypothesis.** Reusable dense storage plus read/write ranges and a
separate logical window can preserve this behavior. A ring could reduce movement,
but the existing matcher consumes one contiguous slice. A two-span replacement
must preserve matching across the boundary and context handling, or pay for
coalescing; it is not an equivalent drop-in merely because both hold bytes.
The line/context policy belongs above storage initialization and borrowing.

## EW2: DuckDB composes nullable nested data with selected row mappings

**Observed trace.** The list-value scalar function reserves a child vector for
`row_count * argument_count` elements. For each row and argument it follows the
input selection, writes a payload only when the input is valid, and otherwise
marks the output validity bitmap. It then writes each parent's offset/length and
publishes the child logical size. A NULL child therefore occupies a logical list
position without this producer writing its payload. [List construction][dd-list].
This is a concrete non-prefix payload-writing pattern; the vector layer permits
allocation without zero initialization. Whether reused NULL slots happen to
contain earlier initialized bytes is not established by this producer and is not
its validity contract. [Vector initialization][dd-vector-init].

A physical filter produces a selection. If every row survives it references the
input; otherwise it slices the chunk. Slicing an existing dictionary composes
selections; slicing ordinary data retains a child and attaches a dictionary
buffer. Struct children receive corresponding slices. A consumer requests a
unified view containing data, validity, and a row mapping; it may materialize a
non-flat child. Explicit dictionary flattening allocates a result and copies
selected values. [Filter][dd-filter], [slice and references][dd-slice],
[unified view][dd-unified], [flatten][dd-flatten]. These are alternative execution
paths, not a claim that every list expression is always followed by that filter.

The compositional obligations include: selected indices belong to the source
domain; a valid selected cell has initialized payload; parent list intervals
belong to the child logical extent; nested child types remain consistent; and
borrowed views keep their backing and auxiliary storage alive. `Vector::Reference`
retains shared buffer ownership. Selection storage may itself be owned or
borrowed. Neither a valid integer index nor a non-NULL bitmap bit alone proves
all these relations. [References][dd-slice], [selection ownership][dd-selection].

**Resources and costs.** Selection can avoid copying column payload, while still
allocating descriptors/selection storage and doing nested work. It is not
allocation-free. List reserve grows child storage; copying and size publication
are separate operations, and size checks may throw. We did not establish a strong
exception guarantee across nested growth. [List buffer][dd-list-buffer]. The
bitmap and column layout are application representations; source alone does not
show whether a per-cell enum layout would materially worsen a given query.

**Whitefoot hypothesis.** Owned column storage, nullable element semantics, and
checked row mappings are compatible with the selected foundation. Full arrays,
prefixes, and rings alone do not express this validity-to-payload relation. An
array of `Option<T>` is a safe candidate; equivalence to the separate bitmap and
payload layout is unmeasured. Source-level library authority over such a layout
is a concrete future question, not something the finite interval model already
settles. Dictionary selection is also not inherently a uniqueness certificate:
read mappings may repeat source positions, so mutable scatter needs additional
target-disjointness or combination evidence.

## EW3: DuckDB constructs strings into final storage and retains long-string backing

**Observed trace.** String concatenation first totals each result's required
length, allocates its string destination, and reuses the length scratch vector
as a written-byte cursor. It then copies each non-NULL input segment into that
destination and finalizes every result. The binary concatenation operator builds
a local string result and returns it by value after filling it.
[Concatenation][dd-concat]. The requirement is complete initialization before a
result becomes readable, not a requirement to copy a complete string value after
every appended segment.

Ordinary short strings store up to twelve bytes in their descriptor. Longer
strings carry a pointer and cached prefix; comparisons use length/prefix before
following the pointer when necessary. `StringVector` keeps short values inline
and allocates long values through a string heap. The heap explicitly promises
stable returned storage through its lifetime. Adding a chunk links a new
allocation rather than relocating earlier chunks. Vector buffers can retain
references to other buffers so referenced string bytes outlive the producing
descriptor. [String representation][dd-string-type], [string vector][dd-string-vector],
[heap contract][dd-string-contract], [arena allocation][dd-arena],
[retained references][dd-string-refs].

**Resources and costs.** The code demonstrates one payload fill per result and
whole-heap ownership, not a per-string free operation. Oversized long strings
throw before allocation; backing allocation has no local refusal result here.
Concatenation may already have allocated earlier results when a later allocation
fails. This study does not establish resumable partial success or transactionally
unchanged output on exceptions. [Heap construction][dd-string-heap]. The inline
threshold, prefix width, chunk policy, and allocator are implementation choices;
their current performance advantage was not independently measured.

**Whitefoot hypothesis.** A builder can hold an unsealed result place, prove each
written interval, and publish the initialized string after completion. Allocation
and input evaluation order must remain the source's order. Existing readable
bytes must not be inferred from the upstream name `EmptyString`. Inline storage
requires preventing relocation while an interior view is live; external backing
can remain stable while descriptors move. Handles are a possible alternative for
long strings, but replacing a stable pointer with a lookup changes the internal
consumer contract and needs its own cost evidence. This trace supports optional
stable ownership and direct result destinations, not mandatory handles or a
universal ban on inline values.

## EW4: DuckDB grouping separates sparse hash slots from pinned aggregate rows

**Observed trace.** A grouped aggregate table initializes a power-of-two pointer
table and separate tuple storage. Before processing a chunk it ensures room for
the chunk under its resize threshold. It hashes input groups, then repeatedly
partitions the remaining input positions into new slots, candidate matches, and
unmatched collisions. A new slot is first reserved by installing its salt; tuple
rows are appended and aggregate states initialized; only then are row pointers
installed. Candidate rows are compared and collisions probe onward. The output
address vector maps each input to its aggregate state, which `AddChunk` updates.
[Setup and pinning][dd-agg-setup], [find/create][dd-agg-find],
[update caller][dd-agg-update].

This entails three distinct slot states: empty, reserved, and carrying a usable
row pointer. The actual packed entry uses a nonzero sentinel during reservation;
`IsOccupied()` therefore does not independently prove a usable pointer in the
middle of a batch. The table's outer count equality is verified after the
operation, while the temporary selections account for work in progress.
[Packed entry][dd-entry], [verification][dd-agg-resize]. Different input keys may
collide; equal keys intentionally resolve to the same aggregate row. Distinct
input indices do not imply distinct mutable output addresses.

Resize allocates and clears a new pointer table, walks pinned tuple rows, and
rehashes their pointers. It does not relocate those rows in this operation.
Later scan finalizes aggregate results; under the destructive scan policy it
destroys aggregate states and resets exhausted collections. This is not a claim
that every repartitioning or scan mode has the same address lifetime.
[Resize][dd-agg-resize], [finalization][dd-agg-finalize].

**Resources and costs.** Pointer-table growth performs work proportional to table
capacity plus reinsertion work, avoiding a copy of every aggregate payload at
each rehash. Linear probing still depends on collisions. An upstream comment
attributes a preparatory lookup loop to amortizing cache misses; this study did
not measure that claim. The packed salt/pointer representation explicitly varies
by platform configuration. Bounds on probing in C++ end in internal exceptions;
they are not written proofs of termination for Whitefoot. Allocation and row
initialization can occur after slot reservation, with no resumable failure result
in this path. [Find/create][dd-agg-find], [entry layout][dd-entry].

**Whitefoot hypothesis.** Full initialized slot metadata plus a stable owning row
store can express the split without making every stored object a handle. A typed
slot sum with explicit reserve/install transitions is another candidate; matching
the packed representation's footprint is untested. Required contracts include
reservation conservation, initialized aggregate state, lookup equivalence across
rehash, and many-to-one result mappings. The first owned-place slice can provide
storage identity and legal destinations, but cannot by itself prove these
relations or admit parallel writes to the returned addresses.

## Focused revalidation: contracts and host mechanisms

A second primary-source pass reread the EW1, EW3, and EW4 files cited above and
the Rust standard library at the peeled Rust 1.89.0 tag commit
[`29483883eed69d5fb4db01964cdf2af4d86e9cb2`](https://github.com/rust-lang/rust/tree/29483883eed69d5fb4db01964cdf2af4d86e9cb2).
It supplies qualitative demand and comparison contracts, not occurrence counts,
profiles, or evidence that these forms are common.

| Evidence reread | Contract that a replacement must preserve | Host mechanism that the source does not make a Whitefoot requirement |
| --- | --- | --- |
| ripgrep `LineBuffer` state, fill, roll, and growth [state][rg-state], [fill][rg-fill], [growth][rg-growth] | Preserve the byte stream, searchable complete-line window, retained incomplete suffix, absolute offsets, EOF publication, binary cutoff, configured growth refusal, and reuse after reset. A scan view ends before mutation or relocation. | Rust `Vec<u8>`, zero-filled spare bytes, `copy_within`, and the observed three-times eager growth are choices. In particular, the zero fill satisfies this implementation's writable-slice route; the workload requires safe writable capacity, not readable zeros. |
| DuckDB concat, `string_t`, heap, and vector ownership [concat][dd-concat], [representation][dd-string-type], [heap][dd-string-contract], [retention][dd-string-refs] | For each row, compute the same NULL/value result, reserve its final payload destination, fill it, and publish only after completion. Long payload addresses remain valid through the retaining heap/vector lifetime. The batch implementation reserves all result destinations before copying any payload. | The twelve-byte inline cutoff, cached prefix, C++ return by value, arena chunk policy, and the name `EmptyString` are not semantic requirements. The source also does not promise rollback or resumable progress if a later batch allocation fails. |
| DuckDB packed entry and aggregate find/resize [entry][dd-entry], [find/create][dd-agg-find], [resize][dd-agg-resize] | A reserved slot cannot supply a row pointer before append and aggregate-state initialization; rehash preserves lookup and does not move pinned rows; equal groups may intentionally share one output address. | Empty/reserved/ready is a temporal protocol, not an explicit three-tag representation: `SetSalt` makes `IsOccupied()` true before `SetPointer`, and phase selections keep that state away from pointer readers. Salt/pointer packing, pointer width, linear probing, and C++ exceptions are choices. |
| Rust boxed-slice, vector spare-capacity, safe vector conversion, `MaybeUninit`, and array iteration source [boxed construction][rust-box-uninit], [boxed publication][rust-box-publish], [vector layout][rust-vec-layout], [spare capacity][rust-vec-spare], [length publication][rust-vec-publish], [safe boxed-array conversion][rust-safe-box-array], [fallible exact reservation][rust-try-reserve-exact], [partial cleanup][rust-maybe-partial], [array consume][rust-array-consume], [array residual cleanup][rust-array-cleanup] | These APIs witness demand for final-place element construction, an initialized/raw boundary, publication only after the new range is initialized, partial cleanup, and by-value consumption of a complete fixed array with exact cleanup of the unconsumed range. | Safe Rust can prefix-fill a valid `Vec<T>`, clear it on producer failure, and convert it to `Box<[T; N]>` when full; the [matched control][rust-safe-control] exercises that route. `try_reserve_exact` may receive excess capacity, and the full conversion discards it, so exact backing preservation is conditional on `len=cap=N`. Rust's raw-slot route instead uses `MaybeUninit`, `Box::assume_init`, and `Vec::set_len` behind unsafe library boundaries. This is evidence for checked transitions with a stronger resource contract, not evidence that safe Rust cannot express the workload or that Whitefoot writers need public unsafe authority. |

### Decisive same-contract comparisons

The smallest useful comparisons keep observable behavior and resource policy
fixed, then vary only the storage/state mechanism:

1. **Reusable search window.** Feed identical chunk boundaries, long lines, EOF,
   binary bytes, and capacity limits to the rolled contiguous design and a
   candidate ring/two-span design. Require identical visible bytes, matches,
   offsets, context, and refusal point. Record allocations, peak capacity,
   initialized bytes, bytes moved/coalesced, and bytes scanned. A two-span result
   is equivalent only if the matcher consumes both spans with the same boundary
   behavior or the coalescing cost is included.
2. **Fixed-block pool and direct construction.** Allocate a fixed pool once;
   exhaust it, return blocks in a different order, and reuse it without another
   backing allocation. For a block of non-copy, drop-counted elements, fail after
   constructing element `k`: exactly the constructed prefix is destroyed and the
   slot returns to the pool. On success, publication transfers the complete block
   without a whole-block copy. Compare the checked-in [safe initialized Rust
   baseline][rust-safe-control], Rust's pinned `MaybeUninit` route, and the checked Whitefoot place route at the same pool
   capacity and refusal points; count provider allocations, element
   constructions/destructions, copied bytes, peak storage, and steady-state work.
3. **Direct string result.** Use the same row validity and segment order, including
   empty, inline-sized, and long results. Require identical result bytes and
   lifetimes, one final payload reservation per long result, and no extra
   full-payload temporary copy. Count payload writes/copies, arena calls and
   chunks, peak retained bytes, and cleanup. Failure comparisons need an explicit
   common contract; DuckDB's inspected path does not supply a strong rollback
   contract to inherit.
4. **Sparse reserve and rehash.** Replay the same keys, hashes, collisions, resize
   thresholds, and aggregate updates. Require the same groups and results, no
   pointer read during reservation, stable row addresses across pointer-table
   reallocation, and no aggregate-payload move during rehash. Count allocations,
   metadata bytes, probes, reinsertions, payload bytes moved, and peak storage.
   Injection immediately after reservation and after row initialization checks
   cleanup, but its observable failure result must be defined rather than inferred
   from the throwing C++ path.
5. **Full-array consume.** Consume `k` values from a by-value fixed array of
   non-copy, drop-counted elements, then stop. Require ownership of those `k`
   values to transfer and exactly the other `N-k` values to be destroyed, with no
   second allocation or whole-array element copy. This separates consumption of
   a proven-full value from construction of a partial one even if both lower to a
   tracked live interval.

Two rare adversarial cases remain hard falsifiers without being prevalence
claims. A nullable slot whose validity is false may retain stale payload bits from
a previously destroyed or moved owner; it must neither read nor destroy those
bits, and validity may become true only after the replacement payload is
constructed. A recycled pool slot may later occupy the same address: a retained
identity for generation `g` must not read, release, or mutate generation `g+1`.
Static lifetime exclusion can make that trace impossible; a runtime generation
tag is needed only when identities are allowed to outlive retirement. Neither
case justifies bitmap storage or generation metadata on every dense container.

## EW5: Kubernetes maintains a key-to-position relation during heap mutation

**Observed trace.** The scheduler heap stores a slice of keys and a map from key
to `{object,index}`. Insertion appends a key and adds its map entry; heap swaps
update both moved entries' indices. Updating an existing key replaces its object
and fixes priority at the stored index. Deletion finds the index by key and
removes that heap position. Pop removes the key and map entry together. The
scheduler uses these heaps for active and backoff queues; a completed backoff
removes a record from one queue and attempts admission to the active queue.
[Heap][k-heap], [queue construction][k-queues], [backoff transfer][k-transfer].

The decisive invariant is a correspondence between the map domain and slice
keys, with `items[queue[i]].index == i`, unique keys, and the heap ordering
relation. Capacity and initialized-prefix proofs are necessary but insufficient.
Heap order may change while key identity remains stable. This heap's scheduler
key is namespace/name; the in-flight mechanism in EW6 instead uses pod UID.
They must not be collapsed into one universal identity notion. [Heap state][k-heap],
[scheduler key][k-key], [in-flight state][k-events-state].

**Resources and costs.** Indexing supports direct key lookup before logarithmic
heap repair/removal rather than a scan to find the changed object. Map lookup
complexity depends on the underlying map, and comparator cost is not constant by
definition. The heap has no internal synchronization; its caller owns that
boundary. Missing deletion and empty pop return errors, while insertion has no
fallible allocation result or fixed-capacity refusal policy. Go objects and
maps rely on runtime-managed lifetime here. None of this establishes a measured
benefit for one Whitefoot map implementation. [Heap implementation][k-heap].

**Whitefoot hypothesis.** A library-owned indexed heap built from ordinary checked
collections is plausible if its relational invariant can be expressed and
preserved through `Swap`, `Fix`, and helper calls. Stable keys do not require stable
slice addresses or a mandatory object store. Conversely, replacing the map with
a linear scan, adding a small hard cap, or silently dropping an insert changes
an algorithm or behavior constraint. A bounded-memory Whitefoot version needs an
explicit caller contract and failure policy, not an invented equivalence to this
uncapped insertion interface.

## EW6: Kubernetes retires event-history markers out of order

**Observed trace.** With scheduling queue hints enabled, popping a pod appends a
marker to a shared chronological linked list and records its element in a UID
map. Relevant events append to that same list under a lock. A scheduling attempt
reads the events after its marker, ignoring other pod markers. Finishing removes
its map entry and list marker, then prunes only the event prefix before the next
remaining marker. A repeated finish is a no-op. Requeueing defers this cleanup
so it runs on all exits from that operation. [Pop][k-pop], [event access][k-events],
[retirement][k-done], [requeue cleanup][k-requeue].

An illustrative trace is `marker A, event x, marker B, event y`. Retiring B first
removes B's marker but must retain x and y for A. Retiring A first can discard x
while retaining y for B. This is derived from the source, not a recorded runtime
trace. Events may outlive the operation that appended them. List-element
identity must remain usable while the UID map references it, and reclamation
depends on all remaining readers' start positions, not only the latest operation.
The code explicitly avoids overwriting a duplicate in-flight UID because doing
so would lose the old marker and leak retained history. [State rationale][k-events-state],
[duplicate handling][k-pop].

**Resources and costs.** Stable list elements permit interior marker removal
without moving other elements; reading an attempt's history traverses its suffix
and creates a result slice. Prefix pruning releases references once no remaining
attempt needs the events. No fixed bound on retained history is established:
a long-lived attempt can retain many later events. The linked-list choice is
consistent with these operations, but the inspected code supplies no measurement
against a segmented log or indexed representation. Go pointer/GC conventions make
this representation convenient; they do not make it the only algorithm.

**Whitefoot hypothesis.** A stable-node store is plausible. A segmented sequence
with monotone positions and separately managed markers is another candidate, but
must preserve event ordering, duplicate handling, out-of-order completion,
repeated-finish behavior, and reclamation without an extra scan or bound that
violates the intended resource contract. A simple ring of live objects does not
by itself express those obligations. This is a non-I/O container lifecycle
witness for stable identity, shared historical ranges, and compositional
reclamation; it does not prescribe scheduler or I/O protocol changes.

## Consequences for the selected architecture

No inspected trace falsifies typed owned places and explicit result destinations
as a lowering foundation. Several directly need stable backing during a borrow
or construction, while others deliberately move descriptors, keys, or compacted
bytes. The evidence supports keeping semantic ownership separate from any one
physical representation. It supplies no universal requirement for stable
addresses, contiguity, handles, zero tags, zero initialization, or zero allocation.

The foundation must leave the following distinctions intact:

1. **Storage versus semantic domains.** Physical capacity, initialized payload,
   NULL validity, selected rows, occupied slots, and current logical content are
   different facts. EW2 and EW4 prevent treating one integer length or bitmap as
   universal evidence for all of them. Sparse *logical selection* does not itself
   imply sparse initialization; EW2's nullable producer is the separate concrete
   sparse-payload witness.
2. **Place identity versus access authority.** A known target address is not proof
   that it is initialized, currently readable, uniquely mutable, or reclaimable.
   EW4 returns aliased group targets; EW3 and EW6 retain storage beyond local
   descriptor operations. Result-destination reuse must use actual alias and
   lifetime facts.
3. **Local transitions versus composed contracts.** Prefix construction and
   bounded loans can be compiler-owned initially. They do not automatically
   establish map/slice bijections, nullable nested intervals, reservation
   accounting, or oldest-reader reclamation. The selected first slice should not
   claim to close these later language questions.
4. **Application policy versus representation authority.** Growth factors,
   selection/materialization choice, hash collision handling, queue priority,
   and retention policy belong to application/library logic. This sample creates
   concrete reasons to reconsider checked-library layout authority if kernel
   states force a contract-breaking workaround. It neither proves that general
   authority is ready nor justifies rejecting family B permanently. A safe
   library prototype must prove its symbolic relations and preserve effects and
   cleanup; the finite interval model is insufficient evidence for that result.

The bounded follow-up questions are now concrete: can nullable bitmap/payload
construction be expressed without reading or destroying absent values; can an
indexed heap preserve correspondence through helpers; can a reserved slot become
a row reference without exposing its intermediate state; and can out-of-order
markers reclaim exactly the no-longer-observed history? These should select later
Whitefoot experiments, not expand the current lowering slice into simultaneous
implementations of all six applications.

Remaining gaps include measured size distributions and hot paths, allocator
refusal and partial-cleanup behavior across complete upstream operations, extreme
retention under slow readers, immutable/shared graph mutation, and the cost of
safe alternative layouts on wide or nontrivially destructible payloads. SQL NULL
handling is not evidence about arbitrary absent linear owners. A typed hash-slot
sum or nullable enum may be entirely adequate; a bitmap or packed pointer may
matter greatly; source alone cannot decide. These unknowns remain explicit
rather than being converted into mandatory representation rules.

[rg-policy]: https://github.com/BurntSushi/ripgrep/blob/4649aa9700619f94cf9c66876e9549d83420e16c/crates/searcher/src/line_buffer.rs#L5-L39
[rg-build]: https://github.com/BurntSushi/ripgrep/blob/4649aa9700619f94cf9c66876e9549d83420e16c/crates/searcher/src/line_buffer.rs#L120-L130
[rg-state]: https://github.com/BurntSushi/ripgrep/blob/4649aa9700619f94cf9c66876e9549d83420e16c/crates/searcher/src/line_buffer.rs#L288-L376
[rg-fill]: https://github.com/BurntSushi/ripgrep/blob/4649aa9700619f94cf9c66876e9549d83420e16c/crates/searcher/src/line_buffer.rs#L389-L471
[rg-growth]: https://github.com/BurntSushi/ripgrep/blob/4649aa9700619f94cf9c66876e9549d83420e16c/crates/searcher/src/line_buffer.rs#L474-L521
[rg-glue]: https://github.com/BurntSushi/ripgrep/blob/4649aa9700619f94cf9c66876e9549d83420e16c/crates/searcher/src/searcher/glue.rs#L11-L78
[rg-context]: https://github.com/BurntSushi/ripgrep/blob/4649aa9700619f94cf9c66876e9549d83420e16c/crates/searcher/src/searcher/core.rs#L119-L157
[dd-list]: https://github.com/duckdb/duckdb/blob/5f5512b827df6397afd31daedb4bbdee76520019/extension/core_functions/scalar/list/list_value.cpp#L28-L53
[dd-vector-init]: https://github.com/duckdb/duckdb/blob/5f5512b827df6397afd31daedb4bbdee76520019/src/common/types/vector.cpp#L309-L335
[dd-filter]: https://github.com/duckdb/duckdb/blob/5f5512b827df6397afd31daedb4bbdee76520019/src/execution/operator/filter/physical_filter.cpp#L23-L52
[dd-slice]: https://github.com/duckdb/duckdb/blob/5f5512b827df6397afd31daedb4bbdee76520019/src/common/types/vector.cpp#L133-L268
[dd-unified]: https://github.com/duckdb/duckdb/blob/5f5512b827df6397afd31daedb4bbdee76520019/src/common/types/vector.cpp#L1132-L1185
[dd-flatten]: https://github.com/duckdb/duckdb/blob/5f5512b827df6397afd31daedb4bbdee76520019/src/common/types/vector.cpp#L919-L943
[dd-selection]: https://github.com/duckdb/duckdb/blob/5f5512b827df6397afd31daedb4bbdee76520019/src/include/duckdb/common/types/selection_vector.hpp#L19-L121
[dd-list-buffer]: https://github.com/duckdb/duckdb/blob/5f5512b827df6397afd31daedb4bbdee76520019/src/common/types/vector_buffer.cpp#L59-L109
[dd-concat]: https://github.com/duckdb/duckdb/blob/5f5512b827df6397afd31daedb4bbdee76520019/src/function/scalar/string/concat.cpp#L42-L144
[dd-string-type]: https://github.com/duckdb/duckdb/blob/5f5512b827df6397afd31daedb4bbdee76520019/src/include/duckdb/common/types/string_type.hpp#L23-L169
[dd-string-vector]: https://github.com/duckdb/duckdb/blob/5f5512b827df6397afd31daedb4bbdee76520019/src/common/types/vector.cpp#L1968-L2039
[dd-string-contract]: https://github.com/duckdb/duckdb/blob/5f5512b827df6397afd31daedb4bbdee76520019/src/include/duckdb/common/types/string_heap.hpp#L15-L47
[dd-string-heap]: https://github.com/duckdb/duckdb/blob/5f5512b827df6397afd31daedb4bbdee76520019/src/common/types/string_heap.cpp#L15-L59
[dd-string-refs]: https://github.com/duckdb/duckdb/blob/5f5512b827df6397afd31daedb4bbdee76520019/src/include/duckdb/common/types/vector_buffer.hpp#L182-L210
[dd-arena]: https://github.com/duckdb/duckdb/blob/5f5512b827df6397afd31daedb4bbdee76520019/src/storage/arena_allocator.cpp#L63-L101
[dd-entry]: https://github.com/duckdb/duckdb/blob/5f5512b827df6397afd31daedb4bbdee76520019/src/include/duckdb/execution/ht_entry.hpp#L16-L97
[dd-agg-setup]: https://github.com/duckdb/duckdb/blob/5f5512b827df6397afd31daedb4bbdee76520019/src/execution/aggregate_hashtable.cpp#L38-L92
[dd-agg-resize]: https://github.com/duckdb/duckdb/blob/5f5512b827df6397afd31daedb4bbdee76520019/src/execution/aggregate_hashtable.cpp#L197-L295
[dd-agg-find]: https://github.com/duckdb/duckdb/blob/5f5512b827df6397afd31daedb4bbdee76520019/src/execution/aggregate_hashtable.cpp#L566-L751
[dd-agg-update]: https://github.com/duckdb/duckdb/blob/5f5512b827df6397afd31daedb4bbdee76520019/src/execution/aggregate_hashtable.cpp#L522-L540
[dd-agg-finalize]: https://github.com/duckdb/duckdb/blob/5f5512b827df6397afd31daedb4bbdee76520019/src/execution/radix_partitioned_hashtable.cpp#L799-L844
[rust-box-uninit]: https://github.com/rust-lang/rust/blob/29483883eed69d5fb4db01964cdf2af4d86e9cb2/library/alloc/src/boxed.rs#L632-L652
[rust-box-publish]: https://github.com/rust-lang/rust/blob/29483883eed69d5fb4db01964cdf2af4d86e9cb2/library/alloc/src/boxed.rs#L965-L994
[rust-vec-layout]: https://github.com/rust-lang/rust/blob/29483883eed69d5fb4db01964cdf2af4d86e9cb2/library/alloc/src/vec/mod.rs#L290-L332
[rust-vec-spare]: https://github.com/rust-lang/rust/blob/29483883eed69d5fb4db01964cdf2af4d86e9cb2/library/alloc/src/vec/mod.rs#L2907-L2946
[rust-vec-publish]: https://github.com/rust-lang/rust/blob/29483883eed69d5fb4db01964cdf2af4d86e9cb2/library/alloc/src/vec/mod.rs#L1875-L1955
[rust-safe-box-array]: https://github.com/rust-lang/rust/blob/29483883eed69d5fb4db01964cdf2af4d86e9cb2/library/alloc/src/boxed/convert.rs#L259-L311
[rust-try-reserve-exact]: https://github.com/rust-lang/rust/blob/29483883eed69d5fb4db01964cdf2af4d86e9cb2/library/alloc/src/vec/mod.rs#L1537-L1580
[rust-safe-control]: ../../experiments/container-representation/foundation/rust-baseline.rs
[rust-maybe-partial]: https://github.com/rust-lang/rust/blob/29483883eed69d5fb4db01964cdf2af4d86e9cb2/library/core/src/mem/maybe_uninit.rs#L112-L156
[rust-array-consume]: https://github.com/rust-lang/rust/blob/29483883eed69d5fb4db01964cdf2af4d86e9cb2/library/core/src/array/iter.rs#L35-L73
[rust-array-cleanup]: https://github.com/rust-lang/rust/blob/29483883eed69d5fb4db01964cdf2af4d86e9cb2/library/core/src/array/iter/iter_inner.rs#L29-L82
[k-heap]: https://github.com/kubernetes/kubernetes/blob/70d3cc986aa8221cd1dfb1121852688902d3bf53/pkg/scheduler/backend/heap/heap.go#L17-L239
[k-queues]: https://github.com/kubernetes/kubernetes/blob/70d3cc986aa8221cd1dfb1121852688902d3bf53/pkg/scheduler/backend/queue/scheduling_queue.go#L341-L350
[k-key]: https://github.com/kubernetes/kubernetes/blob/70d3cc986aa8221cd1dfb1121852688902d3bf53/pkg/scheduler/backend/queue/scheduling_queue.go#L1395-L1397
[k-transfer]: https://github.com/kubernetes/kubernetes/blob/70d3cc986aa8221cd1dfb1121852688902d3bf53/pkg/scheduler/backend/queue/scheduling_queue.go#L803-L829
[k-requeue]: https://github.com/kubernetes/kubernetes/blob/70d3cc986aa8221cd1dfb1121852688902d3bf53/pkg/scheduler/backend/queue/scheduling_queue.go#L755-L800
[k-events-state]: https://github.com/kubernetes/kubernetes/blob/70d3cc986aa8221cd1dfb1121852688902d3bf53/pkg/scheduler/backend/queue/active_queue.go#L75-L114
[k-pop]: https://github.com/kubernetes/kubernetes/blob/70d3cc986aa8221cd1dfb1121852688902d3bf53/pkg/scheduler/backend/queue/active_queue.go#L186-L233
[k-events]: https://github.com/kubernetes/kubernetes/blob/70d3cc986aa8221cd1dfb1121852688902d3bf53/pkg/scheduler/backend/queue/active_queue.go#L281-L343
[k-done]: https://github.com/kubernetes/kubernetes/blob/70d3cc986aa8221cd1dfb1121852688902d3bf53/pkg/scheduler/backend/queue/active_queue.go#L351-L405
