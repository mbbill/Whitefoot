# Map layout and runtime validation cost

The scalar experiment tests whether a retained extent check necessarily dominates a
scalar hash lookup, and separates that question from representation and algorithm
cost. It serves the container foundation comparison. Its consumer is the parent
container-representation check/measure targets. Retire or replace it when matched
Whitefoot map implementations supply the same isolating evidence.

These are native C and safe Rust controls, not a checked Whitefoot map or a model
of a new proof system. No upstream application was executed. The scalar correctness
trace includes replacement, deletion, tombstone reuse, absence, a full table and
malformed extents; timings cover lookup only, without mutation or tombstones.
No generic non-copy payload, initialization permission, allocation failure or
concurrent access claim follows from this scalar experiment.

The owning sparse control below separately tests a resource-owning operation
chain and one-backing layout costs. Its protocol evidence and measurement scope
do not widen the scalar lookup conclusions.

## Reproduction and comparison boundary

Run `make check` or `make measure` here. The latter writes fresh raw samples into
`.build/`; retained CSVs are explicitly updated after interpretation. The root
research gate runs correctness and format checks, never a timing threshold.

Recorded 2026-09-08 on arm64 macOS, Apple Clang 21.0.0 and Rust 1.98.1
(`48a229ceaefd4985c50990b14116b6d856af0985`, LLVM 22.1.8), both at optimization level
2, without LTO or target-native flags. The C and Rust compiler pipelines differ;
cross-language timing is not an isolated language or proof benefit. Coordinated
source investigation and intermittent compiler work shared the host; there was
no CPU isolation, frequency control, confidence interval or production profile.

The C variants use the same mixed u64 key hash, scalar linear probing, collision
comparison and termination. A payload is one key plus three u64 values. The
tagged representation puts a byte state before each payload; the split form puts
one control byte per slot beside a separate payload allocation. On this host,
these occupy 40 versus 33 backing bytes per capacity slot, including the 32-byte
payload. This is a 17.5% reduction in requested backing, not a claim about Rust
Option layout or current WF enum layout. Table descriptors and allocator headers
are excluded. Payload is initialized before its live tag is published; absent
payload is not read. The timed maps remain immutable.

The four C variants are:

- `tagged`: interleaved state/payload;
- `split`: separate control/payload, correct extents supplied by construction;
- `split_check_each`: the split representation checks each occupied candidate's
  index against the supplied payload extent and returns an error on mismatch;
- `split_validate_batch`: reject mismatched extents once before the batch, then
  use the split lookup. This validates extents, not payload initializedness.

Every timed query calls a direct noinline lookup; variant selection occurs once
per batch. This boundary isolates lookup costs but may dilute a tiny check cost
relative to fully inlined application code. Clang assembly was inspected: batch
calls and the checked lookup's compare/branch survive. A volatile observation in
the C batch and black_box inputs in Rust prevent repeated-call elimination.
Both versions return checksums observed by the harness.

The Rust control uses std HashMap with a custom deterministic u64 hasher producing
the exact same hash outputs. It uses the same payload, query stream and checksum;
the implementation's table algorithm differs. Requested element capacity is
7/8 of the C slot count; this run reports capacities 224 and 57,344. That is
Rust's element capacity, not a measurement of its allocation bytes. This
algorithm/implementation comparator must not be called a same-layout control.

Capacities are 256 and 65,536; occupancy is 25% and 87.5%. Each case generates
32,768 deterministic queries, approximately half absent, and repeats the batch
12 times per sample. Each variant gets its own untimed warm-up before every
sample, avoiding preferential reuse among the three split variants sharing one
backing. Seven C samples rotate variant order; Rust runs separately. Results
describe warmed lookup, not cold-cache latency, steady mutation or tail latency.

## Correctness evidence

Both executables check seven capacities from 1 through 64 against an independent
membership array and a value formula. Full/refused insertion is a C fixed-table
contract; Rust's standard table grows and is not scored on that refusal contract.
Both test replacement, remove/reinsert and absent lookup. The C extent variants
also reject deliberately shortened payload views before out-of-range access.

Timed queries are independently classified by their generated key range. All
variants match that membership/value oracle. The C and Rust CSVs have identical
query counts and checksums for each of the four workloads. There are 112 C samples
and 28 Rust samples. Format/build warnings and correctness checks pass.

## Recorded measurements

Median nanoseconds per lookup, including the stated lookup-call boundary:

| Slots | Occupancy | C tagged | C split | C check each | C validate batch | Rust standard |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 256 | 25% | 8.764 | 8.273 | 8.367 | 8.062 | 7.851 |
| 256 | 87.5% | 18.700 | 18.074 | 18.417 | 18.295 | 14.330 |
| 65,536 | 25% | 11.375 | 10.317 | 10.551 | 10.373 | 8.319 |
| 65,536 | 87.5% | 29.238 | 28.445 | 28.628 | 28.234 | 16.153 |

[C raw samples](measurements.csv), [Rust raw samples](rust-measurements.csv).

The per-access extent check adds about 0.6-2.3% to the corresponding C median in
this run; sample ranges overlap and no statistical significance is asserted.
Batch-validation medians are close to the split control. This supports including
retained checks as performance candidates, not claiming checks are free. It says
nothing about repeated ownership validation or a full-map invariant scan.

The split layout saves requested bytes and has lower lookup medians here, but
the small time differences do not select a universal layout. The Rust comparator
widens the difference at high occupancy; algorithm and generated code remain
confounded. Improving static proof alone does not grant the scalar C algorithm
the standard table's performance. Allocation, initialization and rehash timing
remain separate missing experiments.

Independent review found and corrected two initial control issues before this
recorded run: per-query C indirect dispatch differed from Rust's direct boundary,
and shared split backing received unequal prewarming. No result from the initial
run selects a design. The corrected run keeps both boundaries direct and warms
each variant independently.

## Owning sparse layout and migration

`sparse-owned.c` is a separate same-algorithm C control. Each cell contains a
u64 key and an owning pointer to a 40-byte resource (identity plus 32 data bytes).
Interleaved slots hold control and cell together; the split form puts control
bytes and aligned cells in one allocation. Both use the same seven-bit
fingerprint, Empty/Deleted encoding, modulo bucket selection and linear probing.
These are native ownership conventions checked by the experiment, not WF
admission or a soundness proof for a projected enum/resource-permission API.

The deterministic trace covers colliding insertion, duplicate replacement after
a tombstone, deletion, lookup past deletion, reuse and full-table refusal.
Replacement returns the displaced resource; refusal retains the offered one.
An independent unsorted key/value oracle and a separate resource-identity ledger
check contents and exact ownership across table, caller, held migration cell and
destruction. Initial allocation refusal and rehash allocation refusal are injected;
the latter leaves the old descriptor, contents and ownership unchanged.
Successful 8-to-16 growth is also executed. Refusal injection covers table backing;
resource construction asserts allocation success, and migration steps do not
allocate. Resource-constructor refusal and mid-migration allocation failure are
not covered.

Migration owns source and target plus either a scan cursor, a complete held cell
with persisted probe progress, or a terminal state. One budget unit inspects one
control byte with its attached constant work. The three-colliding-entry trace
checks cleanup at every fresh stop from 0 through 14, then independently resumes
in one-unit calls. Zero-budget calls preserve returned states field by field.
A deliberately full target returns a blocked state retaining the held resource;
retrying unchanged owners makes no progress. Cleanup destroys source, target and
held resources exactly once. It is cancellation, not restoration of the old map.
External updates are frozen during migration; this is not Redis's interleaved
lookup/update service contract, a wall-clock bound or a concurrency experiment.

On the measured 64-bit ABI, at capacity 4096 with 2048 entries:

| Cost | Interleaved | Split single backing |
| --- | ---: | ---: |
| Slot backing bytes per capacity unit | 24 | 17 |
| One table backing | 98,304 B | 69,632 B |
| Source + target peak backing during same-capacity rehash | 196,608 B | 139,264 B |
| Live table backing allocations during rehash | 2 | 2 |
| Control examinations per rehash | 6,144 | 6,144 |
| Logical cells relocated | 2,048 | 2,048 |
| Logical cell bytes relocated | 32,768 B | 32,768 B |
| Control bytes initialized for initial source + target | 8,192 B | 8,192 B |

The split payload starts at byte 4096; checked alignment/extent arithmetic includes
any required padding at other capacities. The 29.17% backing reduction excludes
the common 81,920 resource bytes and allocator overhead. Current native table
descriptors are 24 and 48 bytes; redundant split pointers/offset make this a
control implementation, not a minimum metadata claim for WF. Peaked backing
counts are requested live extents, not RSS or allocator-internal peaks. Logical
relocation counts a cell once; source-to-held and held-to-target are two C
assignments, whose eventual machine traffic is not established by that counter.
Vacant payload bytes are not initialized in either control.

The retained [raw samples](sparse-measurements.txt) were recorded on 2026-09-08,
Apple M1 Pro, arm64 macOS 26.6.2, Apple Clang 21.0.0, strict C11 at `-O2` without
LTO or target-native flags. Source SHA-256:
`e458d32cd64d61d69cf76e754ef052146a0a098d17764bb0c82730e23c83985e`.
Five samples per layout alternate order. Each separately executes three untimed
warmups and 100 timed rounds, with identical resource identities and checked
witness/work equality. The monotonic-clock interval includes target allocation,
control initialization, migration, old backing release and a full digest scan;
resources are constructed before timing and destroyed afterward.

Median **rehash + digest** time is 21.780 microseconds interleaved (range
21.750–23.650), versus 19.710 microseconds split (19.650–20.050). The split median
is about 9.5% lower in this run. Timed tables are half full with direct-bucket keys,
so migration encounters no collisions; collision/progress correctness is checked
separately. This is neither isolated rehash time nor a general map performance
result. Sample witness/examination/move totals include all 103 rounds; elapsed
time is divided by only the 100 timed rounds. Each timed target initializes 4096
control bytes, not the source-plus-target 8192 from the structural check.

Optimized-code review confirms direct migration calls and the included work remain;
there is no per-entry layout dispatch. `NDEBUG` is rejected because this checked
control uses assertions to execute and verify operations. Strict builds and
deterministic checks pass; sanitizer checks cover the same protocol. The Windows
QPC branch has been read but was not executed in this measurement.

The structural reduction and bounded cost result justify testing projected layout
for valid WF slot values. They do not distinguish projected values from resource
permissions if both produce the same code, nor prove WF borrowing, generic
behavior, fallible in-place construction, stable row references or reclamation.
The proposed authority must still admit the actual operation chain. This control
is wired into `check` and `measure`; retire it when a maintained checked WF map
provides the same discriminating operation and layout evidence.
