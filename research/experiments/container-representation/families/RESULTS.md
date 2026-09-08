# Container-family probes

These probes ask whether current Whitefoot can execute representative container
operations. They are capability examples, not prevalence evidence or complete
container implementations. `make check` compiles and runs each supported
Whitefoot source in the default, `--par`, and `--no-overlap` modes, checks the
two intentional source rejections, and checks the matched priority-queue and
byte-growth C controls. The compiler implementation is revision
`3cd7a8ebbc459b52989806442eff73538f131b96`; the added sources and retained samples
are in this directory. Checks and measurements were run on 2026-09-08, arm64
macOS 26.6.2, using Apple Clang 21.0.0 and Rust 1.98.1.

## Supported shapes

`hashmap.wf` implements an eight-slot open-addressed table with
`Option<Entry>` payloads and a separate tombstone bit. Its pure
`map_trace(seed, repetitions)` covers colliding inserts, replacement, lookup,
removal, lookup across a tombstone, tombstone reuse, a missing removal, and
lookups after reuse. The three-mode native trace exits zero and independently
folds the expected operation results to `15521492327637928880` for seed 5 and
three repetitions. The keys and values are copyable scalars; non-copy payload
transfer, growth/rehash, hashing and equality interfaces, and allocation
failure are outside this probe.
`size` is supplied alongside the run and updated using explicitly wrapping
arithmetic. The trace supplies consistent values; this is not a verified
representation invariant for an arbitrary caller-supplied size. The `Full` return
is implemented but not exercised by this trace, whose maximum live size is three.

The map safely reads an affine option through a small borrowed helper. Current
`find` takes and returns the unchanged run because the rejected two-statement
child-reborrow spelling below cannot retain the helper result. This is a local
operator/region shape and an extra source/ABI burden. It does not show that an
`Option<Entry>` map or shared lookup is generally inexpressible.

`priority.wf` is a fixed-capacity indexed min-heap trace with scalar push and
pop behavior checked against both an independent sorting oracle and a matched
C heap. Creating the initial owner through `empty_heap()` also avoids the current
compiler's `OwnershipJoin` capability limit when a fresh run first becomes a
callee-attributed owner on a loop backedge. Existing compiler ownership tests
retain a smaller example of that limit; this is not a new language rejection.
Direct numeric postconditions carry length, room, and head changes across heap
operations without tracking global mutation history.

`ordered.wf` is a full-leaf B+ tree split component: it sorts the
candidate key, allocates both owning child cells before publishing the split,
returns duplicate without allocation, and preserves explicit refusal on either
allocation failure. It is not a complete ordered map or a claim about recursive
capacity.
Its executed cases insert 25 and 5 into `[10,20,30]`, and reject duplicate 20.
The source handles allocator refusal, but this experiment does not inject either
allocation failure and therefore does not dynamically validate those cleanup edges.

`boxed-migration.wf` exercises resource-owning slots in one concrete function.
It prepares colliding keys 1, 5 and 9 in four initialized optional slots, replaces
key 5's box, removes key 1, and reuses that exact extracted owner for key 13.
The setup, replacement, deletion and pre-migration inspection use selected
positions; they are not a general insertion/search implementation. A real loop
then traverses source indices, extracts each present `Entry<Box<u64>>`, computes
`(key +wrap step) % 8`, probes destination state and moves into the first empty
slot. The final owners and payload values are independently checked by destructive
extraction at their expected positions: key 9/value 30 at 1, key 13/value 10 at 5,
and key 5/value 11 at 6. The replaced value 20 was checked earlier.

An observation after two visited source positions reads both the remaining
source owner and an already-migrated owner, then the same loop continues. This is
not a returned pause state, reentrant helper, scheduler suspension or cancellation
test. The full-destination branch restores a pending entry to the source before
returning an error; this trace does not exercise that branch or inject any of the
four box-allocation refusals. Native success checks values and transfers, not an
independent allocation ledger. No address-stability or generic-library claim is
made. The duplicate shared `same` parameter on `view_box_entry` supplies a second
region occurrence for FORM-8; it is an authoring workaround, not a recommended API.

This extends the scalar map witness to actual runtime-indexed non-copy migration.
It does not complete the matched sparse-map cost comparison. Two initialized
`FixedVector` owners, separate u64 state arrays, and an ordinary option
discriminant are retained; no compact projected control layout is implemented.

`packed-page.wf` stores variable-length records as a byte length followed by
payload bytes in a fully initialized 32-byte run. It validates and hashes complete
records, prepends by shifting the used bytes backwards, and removes the first
record by shifting the remaining bytes forwards. Both shifts deliberately overlap
their source ranges. The hash includes each length byte and the ordered payload,
with independently calculated expectations for the written byte sequences.
The executed cases include two successive prepends, deletion/compaction, a
truncated record, an oversized page extent, insufficient spare capacity, a payload
shorter than its requested record, and empty deletion. Refused mutation leaves
the input owner and used extent available; ordinary invalid-input branches supply
the bounds facts before any indexed access. `remove_first` validates the first
record only, and `prepend` preserves the existing bytes without validating their
format. Neither claims that arbitrary input becomes a wholly validated page.

This is a byte-codec and movement witness, not the SQLite or listpack format, a
typed-record overlay, an optimized bulk-copy result, or a growing page. The full
32-byte initialization and run descriptor are retained source costs. Validation
is repeated on each scan; there is no exported validated-view capability. This
probe shows that variable byte-record lengths and overlap movement alone do not
require arbitrary typed vacant storage. It does not price those operations.

## Deliberate boundaries and failures

`rejected-wrapper.wf` records the active FN-9 restriction on measures over a
field-selected result place. Its `len_of(table.slots)` postcondition is rejected
with `InvalidPostconditionSelector`; the direct-run map contracts therefore
publish `len_of(result)` instead of hiding the run in a nominal wrapper.

`rejected-shared-option-view.wf` records the OWN-6 restriction on binding the
result of a child reborrow call and using it in a second statement. It is
rejected with `InvalidChildReborrow`. The case is a near-neighbor source-shape
boundary, not a map-wide rejection.

`boxed-helper-gap.wf` is a deferred compile reproducer for nested result-region
substitution. `compose` passes two boxes with the same declared store region to
`build`, then passes its returned run and a value made from its returned box to
`replace_one`. FN-2 substitutes the same region through those nested input/output
positions. The current first diagnostic is TYPE-5 at the latter run argument,
with identical printed expected/found types. The explicit shared region, not
the diagnostic's spelling alone, identifies this as a compiler discrepancy.
Later effect checking, body admission, lowering and runtime remain unvalidated
because that first error stops compilation. `make reproduce-region-gap` currently
fails at that error. This new deferred source is not a normative rejection or a
maintained passing runtime test; no previous check is removed. After repair,
validate the complete body and replace it with ordinary helper-composition
coverage. The single-function migration witness avoids this particular boundary.

`ordered-runtime-gap.wf` is valid source and compiles in all three modes, but
its native binary currently exits nonzero after replacement lowering leaves a
stale owning descriptor. The ordinary gate is compile-only for this file.
`make reproduce-runtime-gap` runs it and therefore fails until the backend
correctness defect is repaired; an abort is never counted as a passing result.
The source expectation remains exit zero after successful allocations and exit
70 on allocation refusal. This newly retained deferred reproducer replaces no
maintained runtime test. Once lowering is repaired, move it into the ordinary
native-success loop and remove its compile-only exception.

The reduced failure is `replace deref(tree) = move replacement` with
`tree: &uniq Box<RuntimeGapNode>`. Emitted LLVM passes the old box's pointee to
the callee. On the successful replacement path it obtains the new pointer, drops
and frees the old pointee, but does not store the replacement into a caller-owned
descriptor slot. The caller still cleans up its old pointer afterward. Independent
runs in all three modes observed nonzero native failure. Borrowed enum inspection
does not itself drop its temporary image. The failure belongs to box replacement
lowering/ABI, not ordered-map semantics; no particular signal is a required outcome.

`make measure` writes new priority samples under `.build/`. The retained CSVs
beside the sources are reviewed observations, not regenerated by `make check`.

## Binary heap cost control

Each round generates 16 integers in `[0,97)` from a runtime seed, inserts them
into an initially empty min-heap, then pops all 16 and folds them into a checksum.
The next round continues the same random recurrence. `priority.wf` uses verified
length/room/head contracts and ordinary consume/return helpers; its heap always
has head zero. It performs the same sift-up and sift-down algorithm as
`priority-costs.c`. The native heap declares the same 16 payload words and two
descriptor words, although native optimization may remove unused metadata.
Both sources initialize their fresh heap storage. No heap allocation occurs.

An independent `qsort` oracle checks 64 seeds at round counts `{0,1,2,7,16}`:
320 input pairs for both Whitefoot and each native control. The sort is not timed
as a heap competitor. The Whitefoot command also directly checks sorted values,
duplicates, and reuse after emptying. These are scalar, fixed-capacity witnesses;
generic comparison dispatch, resource-owning payloads, larger heaps and arbitrary
interleaved updates are outside the comparison.

`abi.rs` checks and changes only two linkage declarations: export
`wf_priority_trace(u64,u64) -> u64` and rename C `main` to `wf_fixture_main`.
It changes no source judgment, LLVM instruction or optimization attribute. The
compiler emits with `--no-overlap`; both emitted IR and C are optimized with
Clang `-O2`, without LTO. The normal C control permits helper inlining; a second
build sets `RETAIN_HELPERS` to keep the pointer-based C push/pop calls. The
external Whitefoot trace and the native trace remain distinct direct calls in
the batch loop, and varying seeds plus observable checksums retain their work.

Each sample warms its own variant with 256 calls, then measures 4096 calls with
seeds 19 through 4114. Seven samples alternate variant order. Timing uses
`CLOCK_MONOTONIC` (QPC on Windows); the recorded host is macOS only. The two
control executables ran separately, with no CPU affinity, frequency control or
exclusive host isolation. Compare Whitefoot and C within the same run. The
one-round boundary-control samples have visible drift and are not evidence for
small differences between the C controls.

Median nanoseconds per complete trace call; ranges are sample minima/maxima,
not confidence intervals:

| Control run | Rounds | Whitefoot median (range) | C median (range) |
| --- | ---: | ---: | ---: |
| Ordinary helpers | 1 | 845.95 (811.77–897.71) | 277.83 (263.92–298.34) |
| Ordinary helpers | 16 | 13,078.61 (13,043.70–13,244.38) | 4,371.83 (4,326.17–4,442.87) |
| Retained C helper calls | 1 | 1,068.85 (953.13–1,280.52) | 392.58 (299.07–473.88) |
| Retained C helper calls | 16 | 13,196.04 (13,008.79–13,297.12) | 4,328.61 (4,304.69–4,410.40) |

The ordinary run shows roughly three times the native elapsed time at this size.
Optimized Whitefoot IR retains push/pop calls and nine 16-byte load/store pairs
after each call to move the returned 144-byte run back to its owner; callee
argument/result transfers remain as well. The native control accesses the heap
through its existing destination. The retained-call control shows that ordinary
helper calls need not force those complete-run copies. It does not isolate all
ABI, addressing, instruction selection and inlining effects, so the timing gap
cannot be attributed exclusively to copies. No new layout or source primitive
was needed to admit this program, and no production lowering is changed here.

Retained observations are `priority-measurements.csv` and
`priority-boundary-measurements.csv`. Reproduce checks and generate fresh samples:

```sh
make -C research/experiments/container-representation/families check
make -C research/experiments/container-representation/families measure
```

Inspect optimized Whitefoot storage from this directory:

```sh
clang -O2 -Wno-override-module -x ir -S -emit-llvm .build/priority.ll -o .build/priority.opt.ll
rg -n 'define.*wf_priority_trace|call.*wf_(push|pop)|load <2 x i64>' .build/priority.opt.ll
```

## Explicit byte-run growth and refusal

`growth.wf` allocates and fills an initial byte run, then either keeps it, refuses
additional growth above a caller limit, or allocates a larger run, copies the
prefix and fills the suffix. It hashes every resulting byte and releases the
backing. Initial allocation refusal returns zero; growth allocation refusal
hashes the still-readable original bytes with a distinct status. The limit
applies only to additional growth: initial allocation and the no-growth path
can succeed with `limit < initial`. This is an intended policy, not an impossible
error arm added to satisfy proof. There is no growth or raw-pointer language
primitive hidden in the source.

The failed-growth path establishes preservation and cleanup *inside this trace*.
The scalar result does not establish a reusable library helper returning the old
owner and its capacity facts to a caller for retry. It also does not model
non-copy relocation, a wrapped source, inline/spill transitions, or outstanding
borrows during growth. Sparse migration has different obligations.

`growth-costs.c` compares the same reserve/fill/grow/digest/free contract using
a source loop, `memcpy`, or `realloc`. Each retains a trace call; shared native
code and digest may inline. Variant selection occurs outside the timed batch
loop, which makes direct calls. An independent oracle generates the logical
byte sequence without allocating or copying. Sixteen seeds, four initial sizes
`{1,16,257,4096}`, two requested extents, three limits and three allocation
refusal positions check 4,608 implementation/input combinations. The ledger
also checks request count, no surviving backing and logical live extents.
First- and second-request refusal are injected equally in all implementations.

`growth-abi.rs` is a private experiment adapter. It verifies the emitted signature,
exports the scalar trace, renames the fixture main, and redirects allocator calls
to the native ledger. It changes no acceptance rule or executable instruction
other than those symbol targets. After optimization it requires the Heap
parameter to have `readnone` and `nocapture`/`captures(none)` attributes before
linking a harness that supplies a placeholder. The current trace does not use
that parameter. This is not a public Heap constructor or FFI contract; a compiler
that starts accessing or retaining it must stop this experiment for adaptation.
Independent negative checks removed each required property and the adapter
rejected both. Normal native fixture execution in all three modes uses the
ordinary command Heap and no adapter.

The 84 retained samples in `growth-measurements.csv` were collected on the host
and toolchain above, with unchanged compiler code through revision
`297adb4e2e2ab736f660a69a7a8f64e71538a50b`. Both languages use Clang `-O2`, without
LTO. Seven samples rotate four variants at each size. Each warms itself with
32 complete calls (two at the largest size), then measures 65,536, 1,024 or eight
calls respectively. Seeds vary per call; observable sums are checked against
the independent oracle outside timing. Allocation instrumentation, ledger reset,
generation, copying, digest and cleanup are inside timing. These are complete
operations, not isolated resize latency. There is no exclusive-host or CPU
affinity control; the 4 KiB WF samples include visible drift.

Median nanoseconds per complete trace, with sample minimum/maximum in parentheses:

| Initial → requested bytes | Whitefoot | C source loop | C bulk copy | C realloc |
| --- | ---: | ---: | ---: | ---: |
| 16 → 32 | 93.48 (93.02–94.57) | 79.01 (77.93–81.27) | 78.75 (77.91–81.05) | 96.88 (95.61–97.61) |
| 4,096 → 8,192 | 16,966.80 (16,788.09–21,961.91) | 15,907.23 (15,802.73–16,193.36) | 15,791.02 (15,691.41–16,051.76) | 15,958.98 (15,873.05–16,008.79) |
| 524,288 → 1,048,576 | 2,155,875 (2,150,500–2,234,250) | 2,069,000 (2,015,875–2,118,750) | 2,093,500 (2,025,000–2,107,375) | 2,047,875 (1,995,625–2,150,000) |

The current WF loop needs no initialization of unused capacity. Its optimized
copy retains scalar byte accesses and circular-index select/subtract operations;
the C source loop is vectorized and the bulk control retains `memcpy`. This is
an actionable lowering difference, not an isolated causal explanation for every
timing difference. In this operation, byte generation and digest also execute
for the entire final sequence. The result does not show a general resize
performance ceiling or that bulk copying is irrelevant to a copy-heavy workload.

Allocate/copy has two explicit backing requests and simultaneous old-plus-new
logical extent. `realloc` has one malloc and one resize request. Its returned
address remained unchanged in all 56 timed largest-size calls and in none of
the smaller timed calls on this allocator. The ledger cannot observe storage
temporarily used inside realloc, allocator headers or resident memory; its
smaller recorded extent is **not** a measured physical peak-memory advantage.
Request counts include injected refusals. Realloc was slower at 16 bytes here;
its mere availability does not establish a performance win, while a provider
contract permitting it remains independently useful to test.
