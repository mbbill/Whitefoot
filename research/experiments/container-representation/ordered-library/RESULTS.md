# Ordered map library costs

This explicit experiment compares the complete boxed-node B-tree in
[`ordered-map.wf`](../../../../lib/containers/ordered-map.wf), source-shaped C,
direct C, and a complete native AVL. The prospective criterion is
[Ordered map trial at v0.68](../../../investigations/containers-and-resources/X1-LIBRARY.md#ordered-map-trial-at-v068),
committed before implementation at `5f10d1d3a`. The complete matched matrix
passed on 2026-09-23 at library revision `06f9a6a7c`. Its costs depend on payload
and workload: the boxed B-tree is not a default representation winner.
The [raw samples](measurements.csv), [identities](identities.txt), and
[structural counts](structural-paths.csv) preserve the complete evidence.
Retire or supersede this directory when a maintained comparison replaces this
contract and preserves its evidence. Its Makefile is its explicit caller;
these targets have no daily gate or CI dependency.

## Contract and construction

The payloads are 16-byte scalar pairs and 264-byte inline pairs. Counts 8, 256,
and 4096 cover one leaf and multiple B-tree levels. Initial insertion and each
round visit ranks `(73 * index) % count`; all selected counts are coprime to
73. Keys are `2 * rank + 1`. Work consists of complete build/cleanup, balanced
hit/miss lookup, replacement/edit/remove/reinsert churn at fixed cardinality,
and half-open ranges spanning at most 16 keys. The library and both C trees
share the 8192-entry logical ceiling; replacement is permitted at the ceiling.
Native AVL implements the same ownership outcomes and operation callbacks.

The matched source control now uses the admitted bundled-entry candidate:
one leading zero-or-one link and 15 entries, each holding a pair and its
following zero-or-one link. It preserves the replacement presearch, recursive
owned Pair insertion, bottom-up promotion, seven reverse/take transfers and
three entry swaps when splitting, detached-separator underflow repair,
successor-only internal deletion, and consuming cleanup order. Direct C keeps
a compact two-array node with top-down split/repair and contiguous transfers.
AVL retains one allocation per pair and height-based rotations. The direct
controls therefore compare complete ordinary alternatives. Source C matches
the argument shapes, layout, branches and transfers; the comparison includes
the languages' different inferred alias facts, as qualified below.

| Representation | Scalar node bytes | Wide node bytes |
| --- | ---: | ---: |
| WF / source C bundled B-tree | 504 | 4224 |
| Direct C B-tree | 376 | 4096 |
| Native AVL | 40 | 288 |

These extents passed both `sizeof` assertions and the exact allocation
observer. They exclude allocator bookkeeping.

Every observed payload word contributes to an ordered digest. Every returned
or finally consumed pair contributes to a separate sum, mixed parity and
count, because final cleanup order is unspecified. The independent oracle
uses a sorted flat sequence and binary search; it has no tree shape or
rebalancing code. The maintained owning caller supplies the separate nodrop
ownership and hostile-comparator checks; inline scalar/record digests are not
a claim of exact resource-identity accounting.

Normal and retained modes use the same sources. Retained mode keeps only the
public operations and supplied callbacks out of line in both languages;
private node, payload-content, result-processing and digest helpers remain
optimizable. The native controls are separately specialized for each payload,
as are the WF interface instances. No runtime comparator dispatch or payload
stride branch is introduced. Saved optimized IR supports attribution.
The harness gives WF the same closed helper scope as C's static functions:
only the two measured WF trace bridges stay exported. Its LLVM adaptation
changes helper linkage, allocator observation names/facts, launcher naming and
the stated public `noinline` attributes; it changes no algorithm instructions.
Tree-shape validation and structural counters run only in correctness/audit
commands. No extra bounds assertion is present in a timed C tree helper.

All node allocations pass through the same counter and retain allocator
`noalias nonnull` information. The observer's allocation header is excluded
equally from requested bytes. Each request must equal that representation's
node size; all nodes and requested bytes must be released after every trace.
The WF and source-shaped C allocation count, requested bytes and peak storage
must agree. Direct C and AVL report their own storage. Peak-node utilization
is `count / (15 * peak_nodes)` for B-trees and 1 for AVL; this is a reserved
capacity measure, not a per-node occupancy histogram.

## Reproduction and budgets

Run construction, correctness and timing separately under the shared guard:

```sh
WHITEFOOT_CHECK_TIMEOUT=120 perl .github/run-check.pl ordered-cost-build \
  make -C research/experiments/container-representation/ordered-library build \
  WHITEFOOTC=/path/to/the/recorded/whitefootc
WHITEFOOT_CHECK_TIMEOUT=40 perl .github/run-check.pl ordered-cost-check \
  make -C research/experiments/container-representation/ordered-library verify
WHITEFOOT_CHECK_TIMEOUT=60 perl .github/run-check.pl ordered-cost-measure \
  make -C research/experiments/container-representation/ordered-library measure-only
make -C research/experiments/container-representation/ordered-library identities \
  WHITEFOOTC=/path/to/the/recorded/whitefootc \
  WHITEFOOTC_LABEL=frozen-v0.68-whitefootc
```

Compiler construction and the canonical repository gate have separate costs.
Investigate an exceeded stage before extending its budget or running it again.
Timing consists of one warm-up (sample 0) and five paired samples (1--5), with
implementation order reversed on alternating samples. Build/cleanup batches
4096 entries across whole traces; other paths run `8192 / count` rounds in a
single complete trace. Report these as amortized whole-trace costs, including
build, full ordered visitation and cleanup, without subtracting setup to claim
isolated operation latency. No additional samples were run for these unchanged
images; a changed candidate needs its own prospective comparison criterion.

## Construction and correctness

The measured compiler is the frozen v0.68 binary with SHA-256
`cbffd4dd1ae8641ef03790457181188988bf70cc4af1a53c50c1f406307bb7f9`.
Native compilation used Apple Clang 21.0.0 (`clang-2100.3.34.2`), `-O2`, on
arm64 Darwin 25.6.0. The identity file records all measured sources, generated
IR and executable hashes. No compiler or specification change was needed.
Published identity labels omit hostnames and local installation/source paths.
The original measured Makefile hash is
`0fd32a941646865a41d702adf41399406c9b838a2ea37da35bd28850b08a5a00`;
the identity file separately records the current Makefile after its
metadata-only portable-label recipe change. That change modifies no measured
C/WF source, LLVM instruction, executable or sample; timings were not rerun.

| Stage | Command seconds | Guard seconds | Budget seconds |
| --- | ---: | ---: | ---: |
| Complete matched construction | 5.12 | 5.15 | 120 |
| Initial matched verification | 1.66 | 1.73 | 40 |
| Final C aggregate-move alignment rebuild | 3.74 | 3.80 | 120 |
| Final matched verification | 1.45 | 1.50 | 40 |
| Single complete timing matrix | 1.67 | 1.74 | 60 |

Three earlier cost-caller admission attempts took 0.54, 0.05 and 0.05 seconds
(0.64, 0.11 and 0.11 guarded). They exposed obsolete `own` type modifiers,
enum binder freshness, and an incomplete forwarded interface application;
all were corrected in the caller before the successful construction. The
initial provisional two-array native-only construction/check took 2.66/1.40
seconds and was superseded by the bundled source control. No stage exceeded
its budget. No timing sample was collected before final alignment and checks,
and no sample was added afterward.

Each normal and retained executable passed 264 sorted-oracle configurations
through all four implementations: 1,056 executions per mode. Each also passed
six 8192-entry native audits of ordering, occupancy, equal B-tree leaf depth,
AVL balance/height, cardinality, replacement/refusal at the ceiling, exact
returned payloads, complete removal, and empty-root reuse. Every trace released
all allocations; WF and source C matched the complete allocation-accounting
record. Both timed modes independently checked every sample against the oracle
and checked the same allocation agreement outside each measured interval.

A separate source-control build counts structural branches; timed builds
contain no such counters. It follows the maintained scalar trace: insert
16--151 ascending and 15--8 descending; remove 40; insert 7; remove 112 and 79;
then remove `7 + (67 * step) % 145`, skipping the three already removed keys,
and reuse the empty map. Independent shape validation follows each removal.

| Structural event | Count |
| --- | ---: |
| Root / nonroot split | 2 / 16 |
| Borrow left, leaf / internal | 23 / 1 |
| Borrow right, leaf / internal | 9 / 1 |
| Merge, leaf / internal | 17 / 1 |
| Internal successor replacement | 19 |
| Root contraction onto a child | 2 |

The algorithm has no predecessor replacement; its recorded count is zero.
The contraction counter counts height reductions onto a nonempty child;
the final empty leaf release is checked by empty-root reuse and accounting.
These are observed branches of the final matched source C, supporting the
trace's structural attribution. They are not direct instrumentation of WF.
The [maintained owning caller](../../../../tests/programs/containers/ordered-map-program.wf)
separately verifies every allocation and each payload serial exactly once,
including owned edit results, refusal/retry, and hostile-comparator progress.

## Timings

Values below are medians of the five measured samples, in nanoseconds per
inserted pair for build/cleanup or per round/key step for other paths. A lookup
step contains one hit and one miss; churn contains replacement, edit, removal
and reinsertion. Range steps visit at most 16 pairs. Every value includes its
amortized complete build, final ordered visit and cleanup. These are not
isolated-operation timings. `B`, `L`, `C`, `R` denote those four paths.

**Normal optimization.**

| Pair bytes | Count | Path | WF | Source C | Direct C | AVL C |
| ---: | ---: | --- | ---: | ---: | ---: | ---: |
| 16 | 8 | B | 19.5 | 18.3 | 13.2 | 39.1 |
| 16 | 256 | B | 34.7 | 34.7 | 22.5 | 47.9 |
| 16 | 4096 | B | 81.3 | 87.4 | 70.3 | 74.7 |
| 16 | 8 | L | 6.8 | 10.3 | 5.9 | 4.9 |
| 16 | 256 | L | 22.9 | 29.9 | 16.6 | 13.3 |
| 16 | 4096 | L | 109.7 | 120.6 | 100.7 | 58.6 |
| 16 | 8 | C | 33.4 | 32.1 | 25.4 | 41.9 |
| 16 | 256 | C | 121.1 | 124.9 | 87.9 | 113.0 |
| 16 | 4096 | C | 210.8 | 222.7 | 212.9 | 202.8 |
| 16 | 8 | R | 12.3 | 12.3 | 7.3 | 13.1 |
| 16 | 256 | R | 54.8 | 57.0 | 29.4 | 50.5 |
| 16 | 4096 | R | 113.3 | 115.8 | 90.1 | 131.1 |
| 264 | 8 | B | 103.8 | 86.4 | 56.2 | 74.5 |
| 264 | 256 | B | 197.0 | 161.4 | 100.6 | 87.2 |
| 264 | 4096 | B | 240.7 | 210.4 | 151.9 | 129.2 |
| 264 | 8 | L | 19.4 | 19.8 | 20.0 | 22.6 |
| 264 | 256 | L | 32.8 | 33.9 | 27.3 | 35.8 |
| 264 | 4096 | L | 202.3 | 199.5 | 152.5 | 120.8 |
| 264 | 8 | C | 194.1 | 179.9 | 127.2 | 111.5 |
| 264 | 256 | C | 303.7 | 282.7 | 208.9 | 207.8 |
| 264 | 4096 | C | 543.6 | 497.3 | 352.1 | 317.3 |
| 264 | 8 | R | 72.4 | 71.7 | 71.3 | 70.4 |
| 264 | 256 | R | 272.2 | 268.9 | 263.2 | 259.4 |
| 264 | 4096 | R | 414.7 | 400.6 | 374.1 | 380.6 |

**Retained optimization.**

| Pair bytes | Count | Path | WF | Source C | Direct C | AVL C |
| ---: | ---: | --- | ---: | ---: | ---: | ---: |
| 16 | 8 | B | 25.6 | 31.0 | 21.2 | 41.0 |
| 16 | 256 | B | 53.5 | 61.3 | 38.8 | 51.8 |
| 16 | 4096 | B | 107.9 | 123.0 | 95.2 | 85.0 |
| 16 | 8 | L | 17.1 | 20.5 | 15.9 | 9.4 |
| 16 | 256 | L | 59.4 | 66.9 | 52.2 | 36.7 |
| 16 | 4096 | L | 157.1 | 170.2 | 147.0 | 98.5 |
| 16 | 8 | C | 47.1 | 64.7 | 45.4 | 51.8 |
| 16 | 256 | C | 179.9 | 200.8 | 142.6 | 141.2 |
| 16 | 4096 | C | 290.3 | 321.0 | 299.7 | 244.5 |
| 16 | 8 | R | 21.6 | 21.2 | 16.0 | 26.0 |
| 16 | 256 | R | 95.8 | 94.8 | 74.5 | 113.0 |
| 16 | 4096 | R | 161.5 | 168.5 | 138.8 | 184.0 |
| 264 | 8 | B | 91.8 | 89.6 | 56.6 | 78.6 |
| 264 | 256 | B | 189.0 | 184.1 | 114.3 | 98.6 |
| 264 | 4096 | B | 239.0 | 243.4 | 169.4 | 140.9 |
| 264 | 8 | L | 22.9 | 23.4 | 22.0 | 24.8 |
| 264 | 256 | L | 75.0 | 77.9 | 69.1 | 56.5 |
| 264 | 4096 | L | 236.8 | 241.9 | 195.9 | 164.3 |
| 264 | 8 | C | 208.6 | 184.7 | 128.2 | 120.7 |
| 264 | 256 | C | 345.8 | 347.3 | 242.4 | 261.2 |
| 264 | 4096 | C | 633.8 | 612.5 | 447.9 | 402.7 |
| 264 | 8 | R | 73.1 | 73.2 | 72.5 | 73.2 |
| 264 | 256 | R | 293.0 | 292.5 | 287.6 | 290.5 |
| 264 | 4096 | R | 439.6 | 439.7 | 400.0 | 410.2 |

Within-sample WF/source-C ratios range 0.67--1.07 for normal scalar and
0.95--1.20 for normal wide; retained ranges are 0.73--1.01 and 0.95--1.13.
These are ranges of each cell's median paired ratio, not ratios of separately
selected best runs. The largest relative WF five-sample spread occurs in retained
scalar lookup at count 256 (55.1--71.7 ns/step); its source-control ratio remains
at or below one in every pair. The matrix supports workload-level differences;
one-percent differences do not establish an ordering.

At 4096 wide pairs, normal WF build/cleanup and churn cost 1.85x and 1.71x the
native AVL; retained costs are 1.70x and 1.54x. Ordinary representation and
movement choices therefore dominate the small matched-lowering difference.
At eight scalar pairs, normal WF build/cleanup and churn cost 0.51x and 0.80x
AVL, and scalar range at 4096 costs 0.87x AVL. The complete native AVL is a
useful alternative rather than evidence that every tree should be binary.
No WF AVL was implemented, so its WF cost is not established by this floor.

## Allocations and reserved storage

Counts and occupancy are independent of payload width and mode for these
streams. Build requests below are per complete trace; churn requests cover
all 8192 round/key steps plus construction. Lookup and range preserve build
storage. `WF` also describes source C, which agrees exactly.

| Count | Path | WF requests / peak nodes | Direct requests / peak nodes | AVL requests / peak nodes | WF / direct peak utilization |
| ---: | --- | ---: | ---: | ---: | ---: |
| 8 | Build | 1 / 1 | 1 / 1 | 8 / 8 | 53.3% / 53.3% |
| 256 | Build | 31 / 31 | 31 / 31 | 256 / 256 | 55.1% / 55.1% |
| 4096 | Build | 363 / 363 | 365 / 365 | 4096 / 4096 | 75.2% / 74.8% |
| 8 | Churn | 1 / 1 | 1 / 1 | 8200 / 8 | 53.3% / 53.3% |
| 256 | Churn | 36 / 33 | 93 / 33 | 8448 / 256 | 51.7% / 51.7% |
| 4096 | Churn | 563 / 562 | 366 / 366 | 12288 / 4096 | 48.6% / 74.6% |

Multiply requests or peak nodes by the verified node extent to obtain requested
or peak bytes; both quantities also appear unrounded in every raw sample.
For 4096-pair churn, WF reserves 283,248 scalar / 2,373,888 wide bytes at peak;
direct C reserves 137,616 / 1,499,136; AVL reserves 163,840 / 1,179,648.
Bottom-up repair plus reinsertion changes this stream's WF occupancy from
75.2% at construction to 48.6% peak utilization. Native AVL pays one allocation
per reinsertion but does not reserve vacant pair slots. Fewer allocations alone
does not establish lower reserved storage or lower elapsed cost.

## Transfer and generated-code attribution

No full node is copied on ordinary search/insertion/removal descent. Both
source implementations retain the independent replacement presearch and the
owning pair argument on insertion: 16 or 264 payload bytes travel at each
recursive level. The direct AVL instead borrows its offered pair while
descending. The source split transfers seven upper entries, reverses them with
three swaps, extracts one median and inserts one carried entry; each entry is
32 or 280 bytes, including its 16-byte link. Insert/remove shifts and merge
append stay linear in fanout. Sibling borrowing exchanges complete pairs and
links locally, without relocating unrelated nodes. Cleanup visits each pair
once and each node once, independently of comparison results.

The saved optimized IR exposes two remaining lowering costs, not new runtime
bounds checks. Wide WF `ordered_map_insert_item$instance$118` has 482 static
loads, 498 stores and 28 explicit 280-byte memcpy call sites; source C's private
`record_source_insert_item` has 6 loads, 18 stores and 21 such call sites, plus
bulk node/window transfers. WF expands the node-to-Box construction into many
field transfers, while Clang retains bulk copies. These are static IR counts
over multiple branches, not dynamic bytes moved or an isolated causal timing.

The alias assumptions are not identical. Retained WF lookup/edit/remove
parameters include `noalias` and `dereferenceable` facts for borrowed map,
key and callback-environment pointers. Source C uses ordinary pointers without
`restrict`; its corresponding optimized parameters lack `noalias`. C does
carry `noalias` on aggregate-result storage, and both languages retain fresh
allocation facts. This is a comparison of matched source algorithms with their
ordinary language/compiler facts, not a lowering-only comparison holding all
alias metadata constant. The metadata can affect optimization, but this dataset
does not isolate its timing contribution; it cannot assign the entire WF/C
ratio to copying or alias information alone. The frozen controls were not
rewritten after this read-only audit.

Both WF modes also retain one 504/4224-byte memcpy per exhausted node during
cleanup before releasing that Box and recursively consuming its leading link.
Source C spells the same final aggregate move, but Clang removes unused fields
and carries only the link. This copy occurs once per released node, not at each
search level, and does not make cleanup quadratic. A consumer of the leading
subobject should need only its 16 bytes; validating that optimization requires
an isolated compiler/source experiment beyond this matrix.

Retained WF has exactly 29 explicitly marked public operations/callback
instances (18 operations and 11 callbacks); private helpers remain unmarked.
Its optimized IR retains 24 public-operation call sites; each C specialization
retains 19 including correctness-only callers. C control tree helpers contain
zero executable proof-only assertions. Logical-ceiling, vacancy, order and
underflow branches remain algorithm work; allocation extent/identity/host
failure checks belong to the shared observer in every implementation. No
extra bounds guard is charged only to a C retained helper.

**Design suitability.** The complete ordinary boxed implementation satisfies
the ownership and progress criterion and provides a reusable consumer for
lowering work. This matrix does not select it as the default map: direct C and
AVL expose material wide-pair transfer and occupancy costs, while scalar and
range results are mixed. The already recorded single-descent insertion and
borrowed owning-pair carrier are concrete ordinary alternatives; no new proof
mechanism or second WF library follows from these measurements alone.
