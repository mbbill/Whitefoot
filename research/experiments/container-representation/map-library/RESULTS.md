# Owning map library costs

This explicit experiment belongs to the generic owning-map trial in
[X1-LIBRARY.md](../../../investigations/containers-and-resources/X1-LIBRARY.md#generic-owning-map-trial-after-the-ring-comparison).
It stays outside daily correctness CI. The first maintained caller is
`make -C research/experiments/container-representation map-native-check`,
which checks the native controls only. `map-check` also executes both source
candidates and owning-child callers in three CLI configurations, checks their
exact allocation identities with the existing formal observer, and compares
WF/C trace contents and allocation totals. `map-measure` is the explicit
timing caller; it is not a daily gate. The eight-path source/control checks
and the first paired timing matrix passed. The measurements split the
representation tradeoff: sparse lookup/edit and dense wide rebuilding win
different traces. They reopen the ordinary single-slot direct-migration
alternative; they do not select a library representation yet. Keep this experiment while it
owns this library comparison; remove it when a maintained successor preserves
the same contracts and evidence.

## Contract fixed before measurement

Both representations own inline keys and values. Put installs the offered
pair and either inserts it, returns the replaced pair, or returns the offered
pair at the capacity ceiling. A replacement remains possible at that ceiling.
Remove returns the owned pair. Lookup borrows its query and visits the stored
value; references and persistent iterators do not escape. No per-payload Box
is required, and the comparison must not add one to a native control.

The first trial uses no cached hash. Rehash recomputes the hash of every live
key and never calls equality or deduplicates entries. The initial runtime
capacity is exact, without rounding. Reserve grows to its explicit target,
does nothing below the current capacity, and refuses above the ceiling.
Same-capacity rehash is a separate operation. Automatic growth occurs only
after a complete insertion probe finds no free slot, doubling the capacity
or saturating at the ceiling, with zero growing to one. This fixes a matched
ordinary algorithm for comparison, not an optimal load-factor policy.

The same deterministic, read-only hash environment and equality body apply
to WF and C. Mixed and deliberately colliding hash distributions are separate
cohorts. Hostile equality belongs to correctness cases; no equality law may
establish a memory bound, discard an owner, or justify a rehash deduplication.

## Distinguishing layout, algorithm and lowering

Compare both admitted WF candidates, their C algorithm controls, and native
sparse/dense floors. The sparse native floor allocates a new table and moves
each live pair directly; it does not inherit a WF permutation plan or force
initialization of an inactive payload. The candidate's temporary planning
backings, slot exchanges and emitted initialization remain measured costs.
The dense control includes a reverse bucket index and repairs it when removal
moves the last entry. Its reserved entry capacity counts in full, not merely
its live population. A compact native index and a copy-enum index are distinct
layouts whenever their strides differ.

The current C controls distinguish five combinations, each at scalar and
256-byte values: sparse direct rebuilding; sparse planning plus permutation;
dense direct rebuilding with a tagged index; the same native algorithm with
a compact index; and dense planning followed by a full bucket scan to repair
the reverse indexes. Sparse planning drops its one-byte-per-target used array
before growing the payload backing; its destination array remains live until
the permutation completes. A deleted sparse slot is restored through the
same complete-slot exchange and reconstruction algorithm as the source
candidate. C leaves inactive payload bytes uninitialized; WF's emitted stores
are a separate lowering cost. Both C map owners contain two words. The ceiling
is a compile-time parameter in both languages, with separate ceiling-three
instances for the correctness policy chains.
The source signatures expose different metadata bounds: sparse planning's
`u64` array requires `ceiling <= 2305843009213693951`, while the dense
16-byte index array requires `ceiling <= 1152921504606846975`. The experimental
ceiling 16384 satisfies both; this comparison does not erase that API difference.

Count map owner width, backing headers, cell/entry/index strides, every
reserved capacity, result layouts, allocation requests, requested bytes and
peak simultaneously live bytes. Inspect retained optimized functions for
actual transfers and stores; a missing memcpy intrinsic is not zero transfer.
Native shared-payload results and the original WF product results are not
the same ABI. Normal versus retained also changes visibility and native calling conventions, so
that ratio is not a copy-only attribution.

In retained mode, only the public `new`, `put`, `remove`, `lookup`, `edit`, `reserve`,
`rehash` and `free` operations, plus supplied hash, equality, borrowed observer
and edit/consuming callbacks, are marked `noinline` on both sides. Private probing,
bounded insertion, planning, normalization and permutation helpers remain
ordinarily optimizable. The trace entry is retained in both modes. The WF
adapter must select those exact names and verify the optimized calls before
measurement; a broad prefix-based barrier would not match this C control.

The original WF put has two distinct owned-pair alternatives and therefore
uses 40 bytes for scalar and 536 bytes for record results in the current
target layout. C shares the alternative pair payload, using 24 and 272 bytes.
A separately identified source control that groups the return reason beside
one pair can test this boundary cost; it must not replace the original
candidate's measurements silently.

### Allocation oracle fixed before timing

Let `C` be the initial capacity, `T` the requested growth capacity,
`B(C) = 16 + C * S` the slot/entry backing, `I(C) = 8 + C * J` the dense
index backing, `P(C) = 8 + 8*C` the sparse destination plan, and
`U(C) = 8 + C` its used array. `S` is 24 or 272 bytes for scalar or record;
`J` is 16 for the tagged index and 8 for the compact native index. Allocation
headers belonging only to the observer are excluded equally from requested
bytes. Every request has one release and final live bytes must be zero.

| Algorithm | Initial requests / bytes | One growth: additional requests / bytes | Growth peak | One same-capacity rehash: additional requests / bytes | Rehash peak |
|---|---:|---:|---:|---:|---:|
| Sparse direct | 1 / `B(C)` | 1 / `B(T)` | `B(C)+B(T)` | 1 / `B(C)` | `2*B(C)` |
| Sparse planned | 1 / `B(C)` | 3 / `P(T)+U(T)+B(T)` | `B(C)+P(T)+max(U(T),B(T))` | 2 / `P(C)+U(C)` | `B(C)+P(C)+U(C)` |
| Dense, either algorithm | 2 / `B(C)+I(C)` | 2 / `B(T)+I(T)` | `B(C)+I(C)+I(T)+B(T)` | 1 / `I(C)` | `B(C)+2*I(C)` |

Growth means `T > C`; a smaller/equal reserve is a no-op. Repeated same-capacity
rebuilds multiply total requests/bytes, not peak bytes. These formulas are
independent of payload contents and hashing. The adapter must confirm WF's
actual allocation requests against them before the matching claim holds.

## Bounded workload and oracle

The principal sizes are capacities 64 and 4096, with live counts at one half
and seven eighths of capacity. Scalar and 256-byte inline values exercise
hit lookup, absent lookup, replacement, and remove/absence/reinsert churn.
Growth and same-capacity rehash are separate paths. The growth timing cohort
uses one round so repeated no-op reserves do not dilute its label. Each rehash
round removes every even key ID, checks absence, rebuilds with those tombstones
present, checks all surviving and absent keys, and reinserts the removed keys
with their next value version. It never asks rehash to deduplicate. Zero-round
fill/cleanup traces measure construction
and initial insertion together; they are not subtracted from other traces to
claim isolated operation latencies. Collision pressure needs only the small
scalar case, not another full Cartesian matrix.

Lookup reads a payload identity rather than checksumming 256 bytes per hit;
construction and consuming paths still observe every record word. Ordered
operation outcomes have an ordered digest. Final consumption has no promised
iteration order, so its independent content oracle compares an order-neutral
multiset digest and count. Expected lookup identities and consumed contents
are derived from a direct key-ID inventory, not another hash table. Empty, singleton,
irregular capacities, full-table replacement/refusal, tombstones, growth from
zero and repeated rehash are explicit controls.

The current correctness matrix covers nine capacity/population shapes
(`0/0`, `1/0`, `1/1`, `3/2`, `3/3`, `63/55`, `64/32`, `64/56`, `64/64`),
eight paths, three round counts, three seeds and two hash distributions.
Ten C variants give 12,960 executions per mode; adding the four WF variants
gives 18,144. Each mode also runs ten C policy chains. All passed, including
full-table hostile-equality misses. The actual guarded costs were:

| Source shape | Construction | Execution/check | Validated scope |
|---|---:|---:|---|
| original | 31.92 s | 10.29 s | C-only and WF/C matrices; 24 source/observer executions; retained-call checks |
| compact | 14.49 s | 1.03 s | WF/C matrix; retained-call checks |

Four source callers execute with ordinary deallocation and the quarantining
observer, in each of the three CLI configurations. Their allocation counts
are 24/14 for dense and 29/14 for sparse, each identity released exactly once.
The observer's concurrent and three negative controls passed. Optimized IR
retains every selected public operation and callback in retained mode on both
sides. The compact check exercises the full trace oracle, not a new run of
the separately authored source/observer fixtures. These commands used the
existing v0.67 gate-profile compiler and Apple Clang 21; they did not rebuild
Rust. Earlier seven-path evidence remains in commit `30f36dff`.

Every accepted timed trace must match its independent content/outcome oracle
and the per-representation request/release counts, requested bytes, peak bytes
and final zero live bytes. The cost driver's header signature is a local
consistency check, not an exact allocation-identity release ledger: it frees
immediately, so reading the header again after a duplicate release is invalid
and reused addresses do not retain allocation identity. The formal library
caller uses the existing quarantining observer and its negative controls for
that stronger check, together with must-consume and owning-child cleanup.
A timing checksum is not a proof of ownership. Allocation is total under
STOR-8: a capacity refusal is an ordinary map outcome, not a recoverable
allocation failure.

Reuse the Slab/Deque harness's ordinary and retained modes, fresh-allocation
`noalias nonnull` instrumentation, rotated implementation order, reversed
cohorts and eleven samples. Compare paired medians within the same source,
seed, occupancy and boundary treatment; keep raw samples. Do not average
different workloads into a fabricated application distribution or select a
layout from bytes alone. Record unexplained costs, including wide owning
results, before claiming a performance floor or choosing further machinery.

The selected primary matrix has 31 workload cells, chosen to separate costs
without inventing an application distribution:

| Capacity / occupancy / hash | Payloads | Paths | Cells |
|---|---|---|---:|
| 64 / 7/8 / mixed | Scalar and record | All seven original paths | 14 |
| 4096 / 7/8 / mixed | Scalar and record | Hit, miss, grow, rehash, setup/cleanup | 10 |
| 64 / 1/2 / mixed | Scalar and record | Miss and churn | 4 |
| 64 / 7/8 / colliding | Scalar | Miss, churn and rehash | 3 |

The larger backing separates dependent lookup and rebuilding costs; the
half-full pair checks load sensitivity; the collision subset checks long
probes and tombstones. A capacity-dependent reversal, anomalous half-full
behavior, or an unexplained operation-specific gap triggers a targeted
extension, not an automatic Cartesian expansion. Correctness still covers
all nine shapes and both hash kinds. Each cell compares seven implementations
at its payload size. Eleven paired seeds in each of two reversed cohorts and
both boundary modes yield 9,548 primary rows. Mode order is normal then
retained for cohort zero and reversed for cohort one; implementation order
rotates with the sample and reverses between cohorts. The regular paths use
`floor(8192/count)` rounds. Setup uses zero rounds and growth one round; each
repeats `floor(8192/count)` complete traces with successive seeds. Their time
includes fill and cleanup on every repetition. The expected checksum folds
each independent trace with multiplier 257; allocation counts and total
requested bytes scale by repetitions, while peak bytes do not.

`MEASURE_SET=edit` selects four additional cells: capacity 64, both payloads
and occupancies, mixed hash. Its callback increments only the first stored
`u64` and returns that identity; the wide value's other 31 words stay unchanged
and are all checked at cleanup. This is a different operation from replacement,
so their timing ratio is not a pure-copy attribution. The additional cells
yield 1,232 rows with the same paired schedule. `MEASURE_SET=boundary` selects
capacity 64 at 7/8 occupancy, both payloads, mixed-hash replace and churn:
1,232 rows per source shape for the original and separately transformed
compact-result control. The `source_shape` CSV column labels WF original or
compact results; C's shared-payload result stays `c-shared`. Source hashes and
independent build directories identify the actual compiled modules; the label
alone is not an identity check. No production library result shape changes
as part of this control.

## Maintained commands and source candidates

Run heavy commands through the repository's shared guard. Construction and
execution can be reported separately without weakening the combined check:

```sh
make -C research/experiments/container-representation/map-library build-candidates build-costs CLANG=/usr/bin/clang
make -C research/experiments/container-representation/map-library check CLANG=/usr/bin/clang
make -C research/experiments/container-representation/map-library measure CLANG=/usr/bin/clang
make -C research/experiments/container-representation/map-library summarize
make -C research/experiments/container-representation/map-library build-compact check-compact CLANG=/usr/bin/clang
```

`BUILD`, `WHITEFOOTC`, `SOURCES`, `MEASURE_SET` and `SOURCE_SHAPE` are overridable;
use a distinct build directory for each transformed source. `summarize` reads
`SAMPLES` (default `$(BUILD)/measurements-$(MEASURE_SET)-$(SOURCE_SHAPE).csv`)
and writes medians grouped by
mode, reversed cohort, workload, implementation and source shape, retaining
all eleven paired samples in the raw CSV. It reports whole-trace elapsed time
and either time per complete trace for growth/setup or time per item-round
for the other paths, including their setup and cleanup. It does not subtract
baselines or pool the two cohort orders. The source candidates are
`dense-map.wf` and `sparse-map.wf`; their corresponding `*-check.wf` and
`*-owned-check.wf` callers cover source outcomes, hostile equality, wrapping,
contracts and owning-child cleanup. Current exact allocation expectations are
24/14 for the dense callers and 29/14 for the sparse callers. The three CLI
configurations are default, `--no-overlap` and `--par`; default and
`--no-overlap` currently select the same lowering, so these are two distinct
lowerings. The ordinary allocator runs as well as the quarantining observer;
the latter's concurrent and three negative controls are reused without
creating another observer implementation. Once a library representation is
selected, move its implementation and maintained correctness callers to their
existing library/formal-test homes and update these callers; do not maintain
a frozen duplicate library here. The other candidate stays only while it
provides this explicitly selected representation comparison.

The compact overlay is [compact-put.patch](compact-put.patch), not a second
copy of either library. `compact-source` copies the three original source
files into `$(BUILD)/compact-source` and applies that patch with zero fuzz.
`build-compact` and `check-compact` use those generated sources in an independent
`$(BUILD)/compact-build`, reusing the existing compilation, retained-boundary
and oracle targets. Each library changes one enum declaration, five returned
owner constructors and one paired outcome match; the adapter changes four
paired matches per representation. The nested reason match has the same
branch bodies, with renamed binders. Hashing, probing, allocation, mutation,
edit callbacks and the operation trace are unchanged. The overlay exists only
for this result-shape discriminator; remove it when the selected source or a
maintained successor preserves the comparison, citing this checkpoint in Git
instead of retaining duplicate libraries. Applying it reproduces the compact
source hashes below exactly. It adds no daily gate dependency.
The overlay SHA-256 is
`d63ef0692294ae534127575065b5765125964e8e3789c6605dec5e03d3634e5a`;
the maintained generation target was executed and all three generated files
compared byte-for-byte with the measured compact sources. The recursive
build/check caller was dry-run checked; it reuses the already verified
recipes, without another construction or timing run for the wiring change.

`measure MEASURE_SET=primary` and `measure MEASURE_SET=edit` keep their output
files separate. For the two-build boundary comparison, run the checked native
binaries in this order inside one guarded command (paths are build outputs
under this experiment):

```sh
original_build=.build
compact_build=.build/compact-build
for cohort in 0 1; do
  if test "$cohort" = 0; then
    modes='normal retained'; shapes='original compact'
  else
    modes='retained normal'; shapes='compact original'
  fi
  for mode in $modes; do
    for shape in $shapes; do
      if test "$shape" = original; then binary_dir=$original_build; else binary_dir=$compact_build; fi
      "$binary_dir/map-costs-$mode" measure "$cohort" boundary "$shape" > "$original_build/boundary-$shape-$cohort-$mode.csv"
    done
  done
done
```

Do not run all original cohorts before all compact cohorts. The output
filename, independent build directory and source hashes jointly preserve
the image identity even for C's `c-shared` rows.

## Measured comparison, 2026-09-22

The host was an Apple M1 Pro with eight cores (six performance, two efficiency)
and 32 GB RAM, arm64 macOS 26.6.2 / Darwin 25.6.0. The native compiler was
Apple Clang 21.0.0 (`clang-2100.3.34.2`), at `-O2` with no LTO. All heavy
commands used the repository's shared guard; construction/checks used
`WF_WORKERS=2`. Timed traces are sequential and do not publish parallel work.
The original first normal/cohort-zero primary command took 1.04 seconds;
the remaining primary, edit and boundary commands together took 4.90 seconds.
Those command durations include oracle/accounting and CSV output outside
each measured interval; they are not operation times.

The first primary command was a bounded cost pilot and is retained as cohort
zero's normal result, not discarded or rerun. It was followed by original
retained/cohort zero, retained/cohort one and normal/cohort one. Edit uses
the same mode/cohort order. Boundary uses original then compact in cohort
zero and compact then original in cohort one, with normal/retained mode
order also reversed. All samples, including the first, remain published.

Each table entry below is the median of eleven **per-sample elapsed-time
ratios**, pairing the same seed, trace sizes and workload. It is not the ratio
of two independently computed medians. `A / B` gives cohorts zero and one
separately. Every trace passed its content/outcome and allocation oracle.
An independent readback checked all 13,244 rows for uniqueness, eleven-sample
groups, cross-implementation/mode/cohort checksums, and exact request/byte/peak
formulas. Observed durations are quantized to 1,000 ns; the shortest is
21,000 ns. A few-percent difference in a short trace does not select a winner.
If such a difference becomes decisive, repeat complete traces in a declared
batch rather than increasing rounds and changing the setup proportion.

### Representation comparison

These ratios are **WF dense / WF sparse**, using the original three-variant
result. Values below one favor dense. Every primary workload is shown; no
application-frequency weighting or overall score is assigned.

| Value bytes | Capacity/live | Hash | Whole trace | Normal A / B | Retained A / B |
|---:|---:|---|---|---:|---:|
| 8 | 64/32 | mixed | churn | 1.667 / 1.585 | 1.264 / 1.276 |
| 8 | 64/32 | mixed | miss | 1.111 / 1.088 | 1.267 / 1.289 |
| 8 | 64/56 | colliding | churn | 1.218 / 1.215 | 1.068 / 1.067 |
| 8 | 64/56 | colliding | miss | 1.416 / 1.430 | 1.091 / 1.095 |
| 8 | 64/56 | colliding | rehash | 1.195 / 1.193 | 1.094 / 1.095 |
| 8 | 64/56 | mixed | churn | 1.534 / 1.542 | 1.172 / 1.138 |
| 8 | 64/56 | mixed | grow | 0.964 / 0.962 | 0.993 / 1.005 |
| 8 | 64/56 | mixed | hit | 1.217 / 1.216 | 1.209 / 1.218 |
| 8 | 64/56 | mixed | miss | 1.233 / 1.233 | 1.107 / 1.149 |
| 8 | 64/56 | mixed | rehash | 1.204 / 1.196 | 1.127 / 1.144 |
| 8 | 64/56 | mixed | replace | 1.194 / 1.194 | 1.126 / 1.125 |
| 8 | 64/56 | mixed | setup-cleanup | 1.013 / 1.000 | 1.045 / 1.040 |
| 8 | 4096/3584 | mixed | grow | 0.838 / 0.844 | 0.881 / 0.881 |
| 8 | 4096/3584 | mixed | hit | 1.164 / 1.185 | 1.110 / 1.124 |
| 8 | 4096/3584 | mixed | miss | 1.088 / 1.079 | 1.100 / 1.096 |
| 8 | 4096/3584 | mixed | rehash | 0.970 / 0.977 | 0.936 / 0.948 |
| 8 | 4096/3584 | mixed | setup-cleanup | 1.015 / 1.007 | 1.016 / 1.023 |
| 256 | 64/32 | mixed | churn | 1.150 / 1.140 | 1.146 / 1.143 |
| 256 | 64/32 | mixed | miss | 1.111 / 1.111 | 1.220 / 1.340 |
| 256 | 64/56 | mixed | churn | 1.186 / 1.178 | 1.147 / 1.142 |
| 256 | 64/56 | mixed | grow | 0.795 / 0.781 | 0.840 / 0.845 |
| 256 | 64/56 | mixed | hit | 1.180 / 1.200 | 1.198 / 1.213 |
| 256 | 64/56 | mixed | miss | 1.185 / 1.162 | 1.100 / 1.115 |
| 256 | 64/56 | mixed | rehash | 0.866 / 0.871 | 0.931 / 0.917 |
| 256 | 64/56 | mixed | replace | 0.980 / 0.985 | 1.013 / 1.018 |
| 256 | 64/56 | mixed | setup-cleanup | 0.883 / 0.881 | 0.951 / 0.950 |
| 256 | 4096/3584 | mixed | grow | 0.702 / 0.704 | 0.743 / 0.745 |
| 256 | 4096/3584 | mixed | hit | 0.916 / 0.919 | 0.956 / 0.960 |
| 256 | 4096/3584 | mixed | miss | 0.944 / 0.944 | 0.954 / 0.951 |
| 256 | 4096/3584 | mixed | rehash | 0.818 / 0.807 | 0.827 / 0.830 |
| 256 | 4096/3584 | mixed | setup-cleanup | 0.778 / 0.781 | 0.842 / 0.850 |

The two orders agree on the substantial split: sparse wins the measured
small-table access/churn cases, while dense wins wide growth and rebuilding.
Large-record lookup changes direction at the larger capacity. Those are
specific measured consumers, not a universal dense-versus-sparse ranking.

The native controls distinguish the wide 4096/3584 rebuilding gap. They use
the same hash, occupancy, outcome and allocation policy; the planned/repaired
controls additionally follow the candidate algorithm, while the direct/tagged
controls expose a cheaper ordinary-native formulation.

| Whole trace | Mode | WF sparse / C planned A / B | C planned / C direct A / B | WF sparse / C direct A / B | WF dense / C repaired A / B | C repaired / C tagged A / B |
|---|---|---:|---:|---:|---:|---:|
| grow | normal | 1.209 / 1.211 | 1.475 / 1.481 | 1.784 / 1.782 | 1.120 / 1.116 | 1.191 / 1.202 |
| grow | retained | 1.043 / 1.051 | 1.704 / 1.678 | 1.777 / 1.769 | 1.104 / 1.102 | 1.295 / 1.296 |
| rehash | normal | 1.108 / 1.115 | 1.288 / 1.295 | 1.427 / 1.450 | 1.061 / 1.057 | 1.132 / 1.140 |
| rehash | retained | 1.010 / 1.022 | 1.409 / 1.329 | 1.396 / 1.385 | 1.035 / 1.038 | 1.158 / 1.158 |

The algorithm control retains much of sparse's gap even in C; it cannot all
be charged to the WF helper boundary. Dense reverse-index repair is also
measurable. These are whole traces including fill, verification and cleanup,
so subtracting columns would not isolate memcpy time. The result meets the
recorded reopening condition for a `Slots<Pair,1>` sparse cell with direct
migration. That additional ordinary source formulation still has to admit,
preserve every owner, and justify its larger cell/double-backing peak. No
third-source timings or library selection are claimed here.

Wide replacement is a separate remaining cost. At 64/56, original WF sparse
is 1.466 / 1.459 times planned C in normal mode and 1.559 / 1.459 with
retained helpers; dense is 1.266 / 1.272 times repaired C and
1.593 / 1.522 respectively. Against the direct/tagged native floors the
corresponding WF ratios are 2.376 / 2.408 and 2.922 / 2.734 for sparse,
2.201 / 2.175 and 2.807 / 2.616 for dense. Algorithmic transfers and the
different result ABI both remain relevant. The compact control below tests
one source formulation; it is not evidence that these costs have all gone.

### Storage and allocation costs

Measured requests agree with the formulas above. At capacity 4096 the
following requested-byte counts exclude only the common observer header.
The growth columns describe one complete setup plus 4096-to-8192 reserve,
not the number of repeated traces in a timing row. Every listed total has
matching releases and zero final live bytes.

| Value bytes | Algorithm | Initial requests / bytes | Growth total requests / bytes | Growth peak bytes |
|---:|---|---:|---:|---:|
| 8 | sparse direct | 1 / 98,320 | 2 / 294,944 | 294,944 |
| 8 | sparse planned (WF and C) | 1 / 98,320 | 4 / 368,688 | 360,488 |
| 8 | dense tagged/repaired (WF and C) | 2 / 163,864 | 4 / 491,568 | 491,568 |
| 256 | sparse direct | 1 / 1,114,128 | 2 / 3,342,368 | 3,342,368 |
| 256 | sparse planned (WF and C) | 1 / 1,114,128 | 4 / 3,416,112 | 3,407,912 |
| 256 | dense tagged/repaired (WF and C) | 2 / 1,179,672 | 4 / 3,538,992 | 3,538,992 |

Both sparse cells and dense entries are 24/272 bytes, but dense additionally
reserves a 16-byte index per capacity unit and an eight-byte index header.
The compact native index is eight bytes per unit; its initial totals are
131,096/1,146,904 bytes. Same-capacity sparse planning needs two temporary
arrays (three requests including construction), with peaks 135,200/1,151,008
bytes; dense rebuild needs one new index (also three including construction),
with peaks 229,408/1,245,216. Fewer temporary requests alone therefore does
not determine the smaller peak.

### Borrowed in-place edit

The separate four-cell edit trace changes only the first word and validates
the complete final record. The dense/sparse ratios are:

| Value bytes | Capacity/live | Normal A / B | Retained A / B |
|---:|---:|---:|---:|
| 8 | 64/32 | 1.188 / 1.214 | 1.220 / 1.231 |
| 8 | 64/56 | 1.151 / 1.158 | 1.239 / 1.224 |
| 256 | 64/32 | 1.176 / 1.172 | 1.393 / 1.385 |
| 256 | 64/56 | 1.205 / 1.200 | 1.281 / 1.323 |

This establishes an ordinary borrowed mutation path without returning the
whole value. It does not establish that arbitrary wide mutation is cheap,
nor that the difference from the replacement workload is all transfer cost.
The latter computes and consumes different contents.

### Compact-result boundary control

Compact uses `Inserted | Returned(kind: ReplacedOrFull, pair: Pair)` and
preserves the original operation sequence. It changes only result declarations,
constructors and corresponding matches. The actual layout is scalar 40 to
24 bytes and wide 536 to 272 bytes; `Option<Pair>` stays 24/272 bytes.
These ratios are **compact WF / original WF**. `C-normalized` is the median
of each sample's WF ratio divided by that sample's matching C-algorithm ratio,
not a division of the displayed aggregate medians.

| Representation / value / path | Normal A / B | Normal C-normalized A / B | Retained A / B | Retained C-normalized A / B |
|---|---:|---:|---:|---:|
| sparse / 8 / replace | 1.000 / 1.000 | 0.986 / 1.000 | 0.898 / 0.923 | 0.898 / 0.927 |
| sparse / 8 / churn | 1.000 / 1.017 | 1.008 / 1.038 | 0.985 / 0.985 | 0.968 / 0.977 |
| dense / 8 / replace | 0.989 / 0.976 | 0.936 / 0.964 | 0.966 / 0.953 | 0.931 / 0.955 |
| dense / 8 / churn | 0.996 / 1.014 | 0.997 / 1.009 | 0.993 / 0.998 | 0.986 / 1.007 |
| sparse / 256 / replace | 0.886 / 0.914 | 0.883 / 0.883 | 0.826 / 0.852 | 0.793 / 0.854 |
| sparse / 256 / churn | 0.897 / 0.909 | 0.904 / 0.879 | 0.919 / 0.916 | 0.918 / 0.913 |
| dense / 256 / replace | 0.898 / 0.922 | 0.902 / 0.913 | 0.830 / 0.852 | 0.792 / 0.857 |
| dense / 256 / churn | 0.905 / 0.924 | 0.924 / 0.920 | 0.950 / 0.949 | 0.952 / 0.952 |

The wide improvement survives both orders, but width alone does not explain
it. The original/compact C controls' optimized modules are byte-identical;
their per-cell paired time ratios span 0.982–1.046, indicating the remaining
run-to-run variation. Small scalar changes are not a basis for a general
performance claim.

The following is the optimized wide **public put** path under both source
shapes. The entry copy occurs before any outcome; different outcome rows
are mutually exclusive and must not be added together.

| Path | Original product result | Compact shared-pair result |
|---|---|---|
| entry snapshot | 1 × 256-byte memcpy | unchanged |
| Inserted | 536-byte result memset | 272-byte result memset |
| Replaced | 536-byte result memset + 1 × 264-byte memcpy | header/tag stores + 2 × 264-byte memcpy |
| Full at ceiling, or after refused reserve | 2 × 264-byte memcpy, including projection; 272-byte prefix memset | same two copies; header/tag stores |
| growth/retry | 264-byte projection + 256-byte retry argument; try-put writes the final result | same transfers |
| caller consumes Replaced/Full | 264-byte Pair projection + 256-byte value projection | unchanged |

Compact's shared projection precedes its inner reason match, introducing the
extra public Replaced copy even while reducing inactive-payload initialization.
Its public put IR frame extent falls from 1584 to 1056 bytes; this is not a
native whole-call-stack bound. Normal optimization retains the same relevant
entry and caller transfers. Private try-put remains a real call in both
modes and shapes without an imposed private `noinline` barrier.
Removing repeated static copy sites by sharing a match arm does not reduce
the dynamic copies on a path. Wide remove's `Option<Pair>` layout and its
optimized function instructions are unchanged between shapes after nominal
type/metadata renumbering; its caller still projects 264 then 256 bytes.
Dense removal also carries its whole-entry swap work. The compact control
therefore establishes a useful source-shape opportunity, not a solved generic
owning-result transfer path or a compiler-layout requirement.

### Sample and build identities

These are successors of the source checkpoint `30f36dff`; the exact measured
input bytes are identified below. Kernel v0.67's existing compiler executable
has SHA-256 `bec227121f4a223906f6f429cec0e4a34a108e8dafac46daa56ffc8a23d19674`.
No compiler implementation changed for this experiment. The linked runtime
comes from base `e6349b80` (backend tree
`ad0f0f663b6daf02cbcfee8efbc6a37ff0d61616`). All twelve runtime object files
are byte-identical between the two builds. C driver SHA-256 is
`6c1996164bfb1e932fd4ed4487de2b1940271d8518cdd17a8f71614e8640cff0`.
The later Makefile edits distinguish output filenames and reproduce the
compact source overlay; they do not change instrumented IR or compiler flags.

| Artifact | Original SHA-256 | Compact SHA-256 |
|---|---|---|
| dense source | `4888f8d2f5cdc8ebd5deb90b920daf41a8b04ba1f13ba70ef347cc2cc97ab62e` | `80c13a5cf4ab0c130a920af2d899ed7d3ac979e8949c49031b292195eb8fbc53` |
| sparse source | `f8bf9f5a4d010f79e388aeb4ea27bc58c500141a4d44ecd996736c47a8aefdb9` | `38f60b5f7d3997a0d82c637aa799d7b4dcfe328fd3dfa68c6a484a10ec5bbd79` |
| trace adapter | `737eb59f8989d4b5a3c1f9f2188387125748c8a990c3ed5e67faecd94406b285` | `7f1ba38e3249ad2c863f1eca8d1cfb8eca4637e849d56b8b1913a0176138420b` |
| raw WF module | `9731b64977e3609cb17769de63f2c421e0c7d8e14c6ecf290135bd600f3192c4` | `ad0d964bf62265bef62617b4d33f9cb7e8e3faf221243a346db47d7bc164ef56` |
| normal optimized WF | `910b76f53b31fb91bb70dbb2fda4977a345d0a104318f42b1b9889e3820bcd67` | `2140143cfad184987147603e6ddd488266b12fe87e52cfef1e81687e060335a6` |
| retained optimized WF | `4f9e46fcf7bb5909365b8780fee39a3453629839c6265bce704b78a3a9e6b6e3` | `17db060dc0ffe60025732bfd160fc8115e8e7cce9296659f613179ee624333ef` |
| normal executable | `1e494f95003ec35217cf1a73462781543e0f11663ff6381994b3e9690115dce7` | `0764aad7137b5592370bdb886c4c17f20f30a92eb31950ff9fe06b9d02882f64` |
| retained executable | `653951fe2f072963b763cd8b9e235384b243c55752f51553081e7bfe99736a2a` | `0ad1600f9c7c6f029d445a4b765eb38ae48c4cfd523ef17b0bdef3b74d1fc8ec` |

Both builds have the same C optimized module hashes:
`7b2cc6f64c3bdbccd7a529ee8d44a5611db8d8fca064eabc9ec10d018bfaf432`
(normal) and
`2011bc70afeced36525939684be33163b73760ffd0356a457a22779c13080b46`
(retained). The matching runtime-object manifest sorts each relative path
under `native/`, with one `SHA256  path` line per object, ending in a newline;
its SHA-256 is
`cfd944afa96c7d0c49c39b28239a1a2baa4a28d87faf7b59cafd62c6827368b5`.

The four archives retain the native driver's original seventeen columns.
They concatenate cohort-zero normal/retained and cohort-one retained/normal,
with one unchanged header. Boundary-original and boundary-compact remain
separate because C's `source_shape=c-shared` does not by itself identify which
linked executable was timed. No synthetic run column was added. Filtering
the header and one `(contract,cohort)` pair recovers that original CSV exactly.

| Raw archive | Data rows | gzip SHA-256 | Uncompressed CSV SHA-256 |
|---|---:|---|---|
| [primary original](measurements-primary-original.csv.gz) | 9,548 | `010db16f338399c04ee83317bbebec749f881039641890967a40c5c76ce7d183` | `9e362c8172f9ab4b4d1376bd7ee0c7cd4d7a3d7f975037d610b7011dced4f234` |
| [edit original](measurements-edit-original.csv.gz) | 1,232 | `291ae62d5d1cf234d859675ea6d68fe8340d1b73e222e6ae5c573664a8504c8f` | `ae74e4b4d8a7dc5cca71259cc54aa521da47bec4d5e3c957a6b1ca1bc93a85be` |
| [boundary original](measurements-boundary-original.csv.gz) | 1,232 | `d62d0b5983f5a7460efc4ba6d0254c622a2b7288fbd3fc29da93a46d9e3e9b3b` | `66e61ed26a843b912d7069b3e976268fc6a18f0ecf28ff36f0fe23006a15c52f` |
| [boundary compact](measurements-boundary-compact.csv.gz) | 1,232 | `55770d7e49908e0da652cc545e10545fe3a74a40d1ee176815e081c96b89d5b4` | `695f38ad600fa4c8ca04bb7e9304330607892e0adbdd0cda03c82df7c7b76dee` |

For example, from this directory the complete primary dense/sparse paired
table can be recomputed without building or timing anything:

```sh
gzip -dc measurements-primary-original.csv.gz | perl -F, -lane '
  next if $. == 1;
  next unless $F[7] =~ /^(?:word|record)-wf-(dense|sparse)$/;
  $v = $1; $g = join q{,}, @F[0..6]; $t{$g}{$F[9]}{$v} = $F[12];
  END { for $g (sort keys %t) {
    die "incomplete group" unless keys(%{$t{$g}}) == 11;
    @r = sort {$a <=> $b} map {$t{$g}{$_}{dense}/$t{$g}{$_}{sparse}} 0..10;
    printf "%s,%.6f\n", $g, $r[5];
  }}'
```

Use the edit archive with the same command for its paired table. The
`summarize` target instead reports individual implementation medians and
whole-trace units; those medians must not be divided and called the paired
statistic above. All archives remain evidence for these exact source shapes,
even if a later library choice supersedes their implementation.
