# Container-family probes

These probes ask whether current Whitefoot can execute representative container
operations. They are capability examples, not prevalence evidence or complete
container implementations. `make check` compiles and runs each supported
Whitefoot source in the default, `--par`, and `--no-overlap` modes, checks the
two intentional source rejections, and checks the matched priority-queue C
controls. The compiler implementation is revision
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
