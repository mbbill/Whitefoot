# Owning map library costs

This explicit experiment belongs to the generic owning-map trial in
[X1-LIBRARY.md](../../../investigations/containers-and-resources/X1-LIBRARY.md#generic-owning-map-trial-after-the-ring-comparison).
It stays outside daily correctness CI. The first maintained caller is
`make -C research/experiments/container-representation map-native-check`,
which checks the native controls only. `map-check` also executes both source
candidates and owning-child callers in three CLI configurations, checks their
exact allocation identities with the existing formal observer, and compares
WF/C trace contents and allocation totals. `map-measure` is the explicit
timing caller; it is not a daily gate. The combined source and cost checks
pass; no operation timings are reported yet. Keep this experiment while it
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

Count map owner width, backing headers, cell/entry/index strides, every
reserved capacity, result layouts, allocation requests, requested bytes and
peak simultaneously live bytes. Inspect retained optimized functions for
actual transfers and stores; a missing memcpy intrinsic is not zero transfer.
Native shared-payload results and the original WF product results are not the same ABI. Normal
versus retained also changes visibility and native calling conventions, so
that ratio is not a copy-only attribution.

In retained mode, only the public `new`, `put`, `remove`, `lookup`, `reserve`,
`rehash` and `free` operations, plus supplied hash, equality, borrowed observer
and consuming callbacks, are marked `noinline` on both sides. Private probing,
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

`native-check` covers ten C variants, nine capacity/population shapes
(`0/0`, `1/0`, `1/1`, `3/2`, `3/3`, `63/55`, `64/32`, `64/56`, `64/64`),
seven paths, three round counts, three seeds and two hash distributions:
11,340 trace executions plus ten policy chains per mode. Both normal and
retained controls passed: 22,680 traces and twenty policy chains in total.
The initial native-only construction and execution took 2.81 seconds with
Apple Clang; this is validation time, not an operation-timing comparison.

The integrated check also passes all fourteen WF/C variants over the same
matrix: 15,876 traces and ten C policy chains per boundary mode, or 31,752
traces and twenty policy chains in total. Four source callers execute both
with ordinary deallocation and with the quarantining observer, in each of
the three CLI configurations. Their allocation counts are respectively
24, 14, 29 and 14, each identity released exactly once. The observer's
concurrent and three negative controls pass. Optimized IR retains each
listed public operation and callback in retained mode on both sides.
Using the existing v0.67 gate-profile compiler and Apple Clang 21, guarded
construction took 31.18 seconds and the complete execution/check command
took 9.33 seconds. These commands did not rebuild the Rust compiler.

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

The maintained matrix has 68 workload cohorts: 56 mixed-hash combinations
(two payload sizes, two capacities, two occupancies, seven paths), plus twelve
colliding-hash combinations (scalar capacity 64, two occupancies, six paths;
setup is not repeated). Each compares seven implementations at its payload
size. Eleven seeds are sampled in each of two reversed cohorts and both
boundary modes: 20,944 rows when all samples pass. Mode order is normal then
retained for cohort zero and reversed for cohort one; implementation order
rotates with the sample and reverses between cohorts. The regular paths use
`floor(8192/count)` rounds. Setup uses zero rounds and growth one round; each
repeats `floor(8192/count)` complete traces with successive seeds. Their time
includes fill and cleanup on every repetition. The expected checksum folds
each independent trace with multiplier 257; allocation counts and total
requested bytes scale by repetitions, while peak bytes do not. In-place edit
and compact-result source controls remain separate bounded comparisons; this
matrix does not yet include them.

## Maintained commands and source candidates

Run heavy commands through the repository's shared guard. Construction and
execution can be reported separately without weakening the combined check:

```sh
make -C research/experiments/container-representation/map-library build-candidates build-costs CLANG=/usr/bin/clang
make -C research/experiments/container-representation/map-library check CLANG=/usr/bin/clang
make -C research/experiments/container-representation/map-library measure CLANG=/usr/bin/clang
```

`BUILD` and `WHITEFOOTC` are overridable. The source candidates are
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
