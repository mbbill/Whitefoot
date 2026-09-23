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

The original 1,152-row replay is owned by revision `1811f2d01`, which preserves
the four-path driver, original samples and identities. Run the following
commands from a checkout of that revision. The current driver has the fifth
replacement-only path; its separate reconstruction/check/comparison commands
appear in the candidate section below.

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
the identity file separately records that revision's Makefile after its
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

Result representation is also qualified. WF payload enums use a four-byte
tag; source C uses a byte tag. Put/Taken/Promotion keep the same aggregate
extent and pair/entry offset, but Put's reason is at offset four in WF and one
in C. WF `Result<u64,unit>` reserves 24 bytes (`i32`, `i64`, `i8`), whereas the
C Lookup result reserves 16 and omits the unit byte. Thus source C is matched
to the algorithm, node layout and ownership-transfer shapes, not an identical
result ABI. These frozen measurements cannot isolate exact-ABI lowering cost.
This limitation does not differ between the two WF arms in the discriminator
below; their native normalization controls remain unchanged.

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

## Single-descent insertion candidate

**Outcome: the candidate fails the registered no-regression criterion.** Its
replacement-only cost rises materially in both reversed cohorts. Normal
264-byte pairs at count 256 have source-C-normalized ratios 1.6214 and 1.6331
against a 3% band; counts 8 and 4096 also lose. Retained wide replacement loses
at all three counts. Build and some churn paths improve, but the criterion
does not permit those gains to hide replacement regressions. The published
library remains the baseline; no second sampling pass was run.

The prospective [single-descent discriminator](../../../investigations/containers-and-resources/X1-LIBRARY.md#single-descent-insertion-discriminator)
was committed at `881f6c888` before this comparison. The
[candidate patch](insertion-candidate.patch) is the sole additional source
artifact: its Makefile caller extracts the baseline from `1811f2d01` into the
build directory, applies the patch, and checks both full source hashes.
The reconstructed candidate is
`ddc53f3bd12e3682c26ea72f33d94da5da787371afb7461fd7d69d1798aca4ec`;
the baseline remains
`affaee669a09320aafc9ec5badbea6c11fd95623e1ea8987ad9811742c4b70bb`.
Retire or supersede this patch with the candidate claim; it is not a second
maintained library or a gate dependency.

The shared WF/C/sorted-oracle driver adds only replacement of existing keys
below the logical ceiling as path `P`. At step `(round,index)`, its offered
value seed is `seed + count + round * count + index`, with wrapping arithmetic;
it consumes the returned old pair. It uses the same 8192 round/key steps and
full construction/visitation/cleanup accounting as the other operation paths.
Native map algorithms are unchanged, and source C remains explicitly
**baseline-shaped**, not a source translation of the candidate. No C `restrict`
or result-layout revision was folded into this comparison.

Construction used the same frozen compiler and native flags as the baseline.
Both images in both modes passed 330 configurations through all four
implementations: 1,320 executions and six complete native audits per image/mode.
All node accounting matched; every timed checksum also passed. The two images'
C control IR files are byte-identical. Each retained WF image marks the same
29 public operation/callback instances and retains 26 public-operation call
sites; each C specialization retains 20 including audit callers.

| Combined stage | Command seconds | Guard seconds | Registered budget seconds |
| --- | ---: | ---: | ---: |
| Initial baseline/candidate construction | 11.54 | 11.65 | 120 |
| Complete correctness and clock observation | 3.34 | 3.44 | 40 |
| Complete fresh A/A and A/B matrix | 14.90 | 14.97 | 60 |

The sources were frozen before timing: driver SHA-256 starts `cd3c8cca`, WF
trace `779ffc60`, Makefile `39a4103c`, and candidate patch `90c71e4b`. Complete
hashes, both images and their IR are recorded in
[the new identities](insertion-initial-identities.txt). The original
[1,152 rows](measurements.csv) and [identities](identities.txt) are unchanged.
The separate [11,520 rows](insertion-initial.csv) retain every warm-up and sample.

A/A uses two fresh processes of the same baseline executable. A/B uses the
rebuilt baseline and candidate. Each comparison has two cohorts; cohort one
reverses arm/mode order and the alternating within-cell implementation order.
Every arm measures its own three native controls, with the same seeds and one
warm-up plus five samples per cell. There are 60 payload/count/path/mode cells,
1,440 rows per complete arm, and eight complete arms. No row from the original
baseline dataset is reused to estimate the candidate's gain.

For each sample, primary normalization is
`(WF_B / sourceC_B) / (WF_A / sourceC_A)`; each displayed cohort value is the
median of its five paired ratios. Direct-C and AVL columns apply the same
formula independently. The cell's symmetric band is the maximum of 3%, the
absolute A/A normalized departure in either cohort, source-C median drift
between arms, and four clock quanta divided by the shortest WF/source-C median
interval. Drift means `abs(median(C_B) / median(C_A) - 1)`, maximized over A/A,
A/B and both cohorts. The [clock observation](insertion-clock.csv) reports
1000 ns for `clock_getres`, the positive-delta grid and the minimum positive
delta across 4096 reads, so the clock term uses 4000 ns. These interpretations
were stated before outcomes were inspected.

Tables show cohort `0 / 1`. Ratios below one favor the candidate. `gain` means
both cohorts beat the lower boundary; `loss` means at least one crosses the
upper boundary; `within` means neither decision is established. Classification
uses unrounded values. `B/L/C/R` retain their earlier meanings; `P` is
replacement-only. No cross-cell average enters the criterion.

**Normal candidate comparison.**

| Pair bytes | Count | Path | Band | A/A source C | A/B source C | A/B direct C | A/B AVL | Result |
| ---: | ---: | --- | ---: | ---: | ---: | ---: | ---: | --- |
| 16 | 8 | B | 5.56% | 1.0069 / 1.0127 | 0.9494 / 0.9494 | 0.9494 / 0.9487 | 0.9487 / 0.9438 | within |
| 16 | 256 | B | 3.95% | 0.9997 / 0.9930 | 0.8732 / 0.8769 | 0.8750 / 0.8617 | 0.9073 / 0.8760 | gain |
| 16 | 4096 | B | 3.71% | 0.9975 / 1.0025 | 0.6607 / 0.6676 | 0.6591 / 0.6674 | 0.6424 / 0.6625 | gain |
| 16 | 8 | L | 7.02% | 0.9830 / 1.0000 | 0.9828 / 0.9989 | 0.9828 / 0.9928 | 0.9661 / 1.0045 | within |
| 16 | 256 | L | 10.98% | 1.0174 / 1.0056 | 0.9343 / 1.0234 | 0.9913 / 0.9790 | 0.9704 / 0.9721 | within |
| 16 | 4096 | L | 4.59% | 1.0073 / 0.9994 | 0.8849 / 0.8696 | 0.8811 / 0.8678 | 0.8848 / 0.8675 | gain |
| 16 | 8 | C | 3.79% | 0.9812 / 0.9882 | 1.0195 / 1.0339 | 1.0269 / 1.0450 | 1.0234 / 1.0311 | within |
| 16 | 256 | C | 4.33% | 1.0302 / 0.9960 | 1.0226 / 1.1022 | 1.0271 / 1.1023 | 1.0596 / 1.0922 | loss |
| 16 | 4096 | C | 3.00% | 1.0231 / 0.9999 | 0.8447 / 0.8428 | 0.8468 / 0.8528 | 0.8359 / 0.8432 | gain |
| 16 | 8 | R | 4.67% | 1.0090 / 0.9904 | 1.0000 / 1.0000 | 1.0000 / 1.0000 | 1.0000 / 0.9908 | within |
| 16 | 256 | R | 3.50% | 0.9994 / 0.9912 | 0.9589 / 0.9597 | 0.9668 / 0.9803 | 0.9522 / 0.9739 | gain |
| 16 | 4096 | R | 3.29% | 1.0229 / 1.0043 | 0.8856 / 0.8806 | 0.8794 / 0.8842 | 0.8967 / 0.8619 | gain |
| 16 | 8 | P | 10.00% | 0.9783 / 1.0244 | 1.4000 / 1.3750 | 1.4146 / 1.3750 | 1.4146 / 1.3750 | loss |
| 16 | 256 | P | 5.56% | 0.9909 / 0.9810 | 1.4366 / 1.4854 | 1.4637 / 1.5287 | 1.4500 / 1.5541 | loss |
| 16 | 4096 | P | 3.64% | 0.9971 / 0.9937 | 0.8768 / 0.8986 | 0.8857 / 0.8921 | 0.9009 / 0.8939 | gain |
| 264 | 8 | B | 4.22% | 1.0112 / 0.9911 | 0.8802 / 0.8663 | 0.8822 / 0.8635 | 0.8866 / 0.8719 | gain |
| 264 | 256 | B | 4.74% | 1.0089 / 0.9947 | 0.8898 / 0.9009 | 0.8989 / 0.8973 | 0.8954 / 0.8939 | gain |
| 264 | 4096 | B | 5.84% | 1.0073 / 1.0028 | 0.8116 / 0.8114 | 0.8173 / 0.8145 | 0.8153 / 0.8235 | gain |
| 264 | 8 | L | 7.05% | 1.0063 / 0.9945 | 1.0003 / 1.0000 | 1.0010 / 1.0000 | 0.9990 / 1.0000 | within |
| 264 | 256 | L | 12.32% | 0.8768 / 1.0157 | 1.0435 / 0.9804 | 1.0370 / 0.9825 | 1.0414 / 0.9775 | within |
| 264 | 4096 | L | 3.00% | 1.0014 / 0.9961 | 0.8932 / 0.8928 | 0.8995 / 0.8921 | 0.8916 / 0.8857 | gain |
| 264 | 8 | C | 3.00% | 1.0023 / 0.9865 | 1.0100 / 1.0160 | 1.0311 / 1.0187 | 1.0084 / 0.9959 | within |
| 264 | 256 | C | 3.27% | 0.9861 / 1.0032 | 1.0331 / 1.0330 | 1.0412 / 1.0410 | 1.0450 / 1.0250 | loss |
| 264 | 4096 | C | 3.00% | 0.9948 / 1.0014 | 0.9833 / 0.9845 | 0.9946 / 0.9893 | 1.0036 / 0.9778 | within |
| 264 | 8 | R | 4.38% | 1.0017 / 1.0000 | 1.0016 / 0.9966 | 1.0067 / 0.9966 | 1.0257 / 0.9966 | within |
| 264 | 256 | R | 3.00% | 1.0169 / 0.9991 | 1.0103 / 0.9919 | 1.0215 / 1.0057 | 1.0123 / 0.9972 | within |
| 264 | 4096 | R | 4.18% | 0.9887 / 0.9967 | 0.9415 / 0.9474 | 0.9334 / 0.9506 | 0.9327 / 0.9561 | gain |
| 264 | 8 | P | 3.87% | 1.0010 / 1.0002 | 1.5494 / 1.5848 | 1.5573 / 1.5263 | 1.5504 / 1.5446 | loss |
| 264 | 256 | P | 3.00% | 0.9993 / 1.0067 | 1.6214 / 1.6331 | 1.6524 / 1.5928 | 1.6169 / 1.6335 | loss |
| 264 | 4096 | P | 3.42% | 0.9879 / 1.0044 | 1.0955 / 1.1166 | 1.1027 / 1.1034 | 1.1128 / 1.1183 | loss |

**Retained candidate comparison.**

| Pair bytes | Count | Path | Band | A/A source C | A/B source C | A/B direct C | A/B AVL | Result |
| ---: | ---: | --- | ---: | ---: | ---: | ---: | ---: | --- |
| 16 | 8 | B | 4.71% | 0.9962 / 1.0017 | 0.8616 / 0.8900 | 0.8333 / 0.8880 | 0.8450 / 0.8900 | gain |
| 16 | 256 | B | 3.00% | 1.0181 / 1.0003 | 0.7448 / 0.7167 | 0.7367 / 0.7244 | 0.7252 / 0.7311 | gain |
| 16 | 4096 | B | 3.00% | 1.0071 / 1.0019 | 0.6420 / 0.6503 | 0.6209 / 0.6327 | 0.6287 / 0.6436 | gain |
| 16 | 8 | L | 3.00% | 1.0072 / 1.0072 | 1.0000 / 0.9985 | 1.0053 / 0.9897 | 1.0000 / 1.0024 | within |
| 16 | 256 | L | 6.52% | 1.0496 / 0.9650 | 0.9943 / 0.9896 | 1.0045 / 1.0242 | 0.9866 / 0.9729 | within |
| 16 | 4096 | L | 3.00% | 1.0223 / 0.9938 | 0.8382 / 0.8804 | 0.8581 / 0.8771 | 0.8464 / 0.8802 | gain |
| 16 | 8 | C | 3.00% | 1.0000 / 0.9926 | 0.9411 / 0.9082 | 0.9407 / 0.9409 | 0.9361 / 0.9438 | gain |
| 16 | 256 | C | 3.00% | 0.9904 / 0.9825 | 0.8706 / 0.8740 | 0.8685 / 0.8792 | 0.8655 / 0.8827 | gain |
| 16 | 4096 | C | 3.00% | 1.0200 / 0.9857 | 0.8147 / 0.7913 | 0.7829 / 0.7940 | 0.7876 / 0.7977 | gain |
| 16 | 8 | R | 3.39% | 1.0000 / 1.0000 | 1.0000 / 1.0112 | 0.9992 / 1.0112 | 1.0054 / 1.0009 | within |
| 16 | 256 | R | 3.00% | 1.0013 / 1.0026 | 0.9974 / 0.9885 | 0.9997 / 0.9923 | 0.9995 / 0.9987 | within |
| 16 | 4096 | R | 4.05% | 0.9787 / 1.0001 | 0.8744 / 0.8823 | 0.8852 / 0.8939 | 0.8801 / 0.8856 | gain |
| 16 | 8 | P | 5.41% | 1.0132 / 0.9878 | 1.2146 / 1.1604 | 1.2324 / 1.1622 | 1.1773 / 1.1622 | loss |
| 16 | 256 | P | 6.49% | 1.0070 / 0.9927 | 1.1317 / 1.1161 | 1.0954 / 1.2609 | 1.0916 / 1.0557 | loss |
| 16 | 4096 | P | 6.67% | 1.0262 / 0.9898 | 0.8758 / 0.8663 | 0.8694 / 0.8633 | 0.8759 / 0.8746 | gain |
| 264 | 8 | B | 6.85% | 1.0230 / 1.0144 | 0.9969 / 1.0053 | 1.0183 / 0.9990 | 1.0149 / 1.0022 | within |
| 264 | 256 | B | 4.76% | 1.0110 / 1.0014 | 0.9695 / 0.9626 | 0.9735 / 0.9621 | 0.9783 / 0.9812 | within |
| 264 | 4096 | B | 4.45% | 1.0037 / 1.0031 | 0.8568 / 0.8600 | 0.8581 / 0.8423 | 0.8463 / 0.8626 | gain |
| 264 | 8 | L | 4.69% | 0.9906 / 1.0000 | 1.0159 / 1.0053 | 1.0049 / 1.0053 | 1.0025 / 0.9951 | within |
| 264 | 256 | L | 23.55% | 1.2355 / 1.0141 | 1.0054 / 0.9865 | 0.9764 / 0.9886 | 1.0000 / 0.9962 | within |
| 264 | 4096 | L | 3.00% | 1.0114 / 1.0149 | 0.9288 / 0.9442 | 0.9292 / 0.9115 | 0.9337 / 0.9325 | gain |
| 264 | 8 | C | 3.00% | 0.9901 / 1.0030 | 1.0399 / 1.0503 | 1.0451 / 1.0540 | 1.0473 / 1.0368 | loss |
| 264 | 256 | C | 3.00% | 0.9994 / 0.9997 | 0.9715 / 0.9658 | 0.9634 / 0.9657 | 0.9696 / 0.9599 | within |
| 264 | 4096 | C | 3.00% | 1.0030 / 1.0014 | 0.9513 / 0.9492 | 0.9462 / 0.9495 | 0.9521 / 0.9546 | gain |
| 264 | 8 | R | 4.00% | 0.9917 / 1.0084 | 1.0067 / 0.9739 | 0.9822 / 1.0050 | 1.0183 / 1.0017 | within |
| 264 | 256 | R | 4.42% | 0.9983 / 0.9975 | 0.9987 / 0.9967 | 0.9966 / 0.9950 | 0.9880 / 0.9981 | within |
| 264 | 4096 | R | 3.00% | 1.0033 / 0.9958 | 0.9512 / 0.9620 | 0.9546 / 0.9484 | 0.9454 / 0.9514 | gain |
| 264 | 8 | P | 3.14% | 0.9979 / 0.9935 | 1.1692 / 1.1427 | 1.1684 / 1.1318 | 1.1614 / 1.1597 | loss |
| 264 | 256 | P | 5.18% | 0.9898 / 1.0138 | 1.2127 / 1.1687 | 1.2286 / 1.1554 | 1.1892 / 1.1369 | loss |
| 264 | 4096 | P | 3.00% | 1.0156 / 1.0160 | 1.0848 / 1.0521 | 1.0827 / 1.0464 | 1.0690 / 1.0536 | loss |

The large replacement regressions survive both cohort orders and all three
native normalizations. Some lookup cells have large A/A bands (23.55% for
retained wide/count 256), so those cells alone would not support a small-effect
decision. They do not erase the independent, consistent replacement losses.
The optional complete repeat was reserved for variance or directional
uncertainty; neither prevents rejecting this candidate, so it was not used.
No cell was selectively rerun or removed.

Optimized IR confirms the intended transfer removal. Baseline wide
`ordered_map_insert_link$instance$89` loads all 33 words of the owned Pair at
entry and stages it for recursion; candidate `$instance$91` takes a reference
to the 272-byte one-slot carrier and passes that same pointer recursively.
The Pair is extracted only at the vacant leaf; replacement swaps the resident
pair with the occupied carrier.

The candidate also changes replacement's result path. Its wide matched-hit arm
clears a 288-byte `Option<Entry>` result, and each recursive None unwind clears
another 288-byte result. This appears as `llvm.memset(..., 288)` in both modes.
The baseline's Boolean replacement search has no optional-promotion return
chain. The presence of these transfers supports the predicted cost mechanism;
the measured percentages do not isolate zeroing, recursion, or carrier
materialization from one another. Source-C result ABI and alias differences
also prevent using that control alone as a causal decomposition.

Lookup/range and replacement-only timings still include complete construction.
Consequently, a faster 4096-pair lookup/range trace does not establish a faster
lookup/range helper; the candidate's lower build cost contributes to it. The
small-count replacement paths amortize construction across many rounds and
expose the regression that a combined churn path could conceal.

Current explicit targets, from this revision, are:

```sh
WHITEFOOT_CHECK_TIMEOUT=120 perl .github/run-check.pl ordered-insertion-build \
  make -C research/experiments/container-representation/ordered-library insertion-build \
  WHITEFOOTC=/path/to/the/recorded/whitefootc
WHITEFOOT_CHECK_TIMEOUT=40 perl .github/run-check.pl ordered-insertion-check \
  make -C research/experiments/container-representation/ordered-library insertion-verify
WHITEFOOT_CHECK_TIMEOUT=60 perl .github/run-check.pl ordered-insertion-measure \
  make -C research/experiments/container-representation/ordered-library insertion-measure-only \
  INSERTION_RUN=initial
make -C research/experiments/container-representation/ordered-library insertion-identities \
  WHITEFOOTC=/path/to/the/recorded/whitefootc WHITEFOOTC_LABEL=frozen-v0.68-whitefootc
```

`insertion-build` reconstructs only checked build-directory copies and never
edits `lib/containers/ordered-map.wf`. `insertion-verify` includes the full
five-path oracle, native audits, retained-boundary checks, control-IR equality
and clock record. `insertion-measure-only` writes a separate complete matrix;
`INSERTION_RUN=repeat` is available solely for the registered one-repeat rule.
These are explicit research targets and add no correctness-gate dependency.

**Design suitability.** Borrowing the carrier removes the intended recursive
Pair copies, but this particular combined-search protocol introduces a
material replacement cost. Retain the baseline source under the registered
criterion. The evidence does not reject every one-pass or carrier formulation,
does not select an ordered representation, and does not by itself require a
language or compiler change.
