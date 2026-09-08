# Map layout and runtime validation cost

This experiment tests whether a retained extent check necessarily dominates a
scalar hash lookup, and separates that question from representation and algorithm
cost. It serves the container foundation comparison. Its consumer is the parent
container-representation check/measure targets. Retire or replace it when matched
Whitefoot map implementations supply the same isolating evidence.

These are native C and safe Rust controls, not a checked Whitefoot map or a model
of a new proof system. No upstream application was executed. The correctness
trace includes replacement, deletion, tombstone reuse, absence, a full table and
malformed extents; timings cover lookup only, without mutation or tombstones.
No generic non-copy payload, initialization permission, allocation failure or
concurrent access claim follows from this scalar experiment.

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
