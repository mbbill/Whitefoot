# Hash-slot occupancy cost

## Question and scope

Does representing sparse occupancy as an ordinary WF value impose enough
lookup cost to justify investigating an additional language mechanism?
The starting concern is "one null check per hit versus hashbrown" in
[docs/todo.md](../../../docs/todo.md). This experiment compares representations
in a small C table, not WF compiler output or hashbrown performance.

The protocol and criterion below were written before compiling or timing the
benchmark, on 2026-09-20 UTC. The source checkout was
`de78fb19c2ad7bbe54dc2146a3b350e10f612067`, branch
`research/x1-hash-slot-cost`. No specification or design decision is changed.

## Language boundary

[The active specification](../../../spec/kernel-spec.md), TYPE-9 and WIN-1..3,
allows an initialized contiguous window, not arbitrary observable holes.
An initially empty `Slots<Option<Entry>, N>` can be filled with `None` values
through `place_back` until its window covers the table's capacity. Sparse
emptiness is then an enum value inside that full window. Runtime capacity
uses `Box<Slots<Option<Entry>>>`. A separate control array does not by itself
eliminate the need to match the option before accessing its payload.

OWN-1 makes these composites affine unless PROV-6 makes them linear. The
benchmark models affine entries; it does not model a linear resource's
must-consume protocol. OP-10 moves window boundaries and OP-11 permits `swap`
without exposing a hole. OP-12 admits a total in-place replacement. OP-13
constructs windows empty (copy-only array filling cannot duplicate an affine
option); OP-14 releases an empty window. A sparse table can replace/swap a
`None` value without moving a window boundary. None of these rules provides
writer-visible uninitialized memory. C is used only as a representation-cost
model; no C operation is proposed as a WF escape hatch.

## Pre-measurement decision criterion

Use a 5 percent threshold. A reproducible 5 percent tax on a hot table access
can matter across a compiler's symbol tables, while a smaller synthetic
difference on one machine is weak grounds for adding language and proof
complexity. This is a research screening threshold, not a universal product
budget or a decision to add a mechanism.

For each payload size and cache tier separately, compare paired repetition
ratios. A representation penalty warrants further mechanism research if
`b_tag / d_folded - 1` exceeds 5 percent on BOTH successful and dependent
lookups, at BOTH 0.5 and 0.875 final load factors. In all four cells require
the 25th percentile paired overhead to exceed 5 percent too. Failed and mixed
lookups and insertion costs are context, not alternate ways to pass this
criterion. Report every cell; do not select a favorable aggregate.

Attribute that result to the duplicate occupancy check only if the analogous
`b_tag / e_proved - 1` comparison also passes those four conditions.
`e_proved` retains b's stride and initialization and removes only its tag read
and check. Otherwise classify any b/d penalty as a representation/codegen
effect, with the precise cache contribution unmeasured. A and D intentionally
have the same lookup algorithm and compact representation; disagreement
between their independently timed samples is a noise/control check. If their
paired medians differ by over 5 percent in a decision cell, mark that cell
inconclusive until an explicitly labeled follow-up resolves it.

The nullable-pointer representation is reported against both baselines but
cannot alone establish a need for a new language mechanism: its allocation,
pointer load, and layout costs differ as well. Evidence against a duplicate
test cost includes the compiler eliminating it or the same-stride comparison
staying below the criterion. Reopen with real WF programs or another CPU if
this experiment does not establish the cost.

## Layouts and common algorithm

Keys are u64; payloads are 8, 32, or 128 bytes IN ADDITION to the 8-byte key.
Every payload word is initialized on insertion. A lookup consumes the first
64-bit payload field, as for a symbol ID or object metadata field; it does not
copy or traverse the entire object. Payload words contain no pointer niche.

| ID | Representation | Bytes per capacity slot, excluding allocator overhead |
| --- | --- | --- |
| a_raw | Control byte + uninitialized array of key/payload entries; control authorizes reading an occupied entry | 1 + 8 + P |
| b_tag | Control byte + inline tagged option; tag is tested before key/payload access | 1 + 16 + P |
| c_box | Control byte + nullable pointer; each occupied entry is separately allocated; pointer tested before dereference | 1 + 8, plus (8 + P) per occupied entry |
| d_folded | Option tag folded into the control byte; compact payload array; no second test | 1 + 8 + P |
| e_proved | Attribution control: b's physical tag, stride, and writes retained, but lookup assumes the control/tag invariant | 1 + 16 + P |

All five use the same 64-bit mixing hash, 7-bit fingerprint plus one (zero
means empty), power-of-two capacity, and scalar linear probing with wraparound.
They insert the same unique keys in the same order. C retains the identical
control-byte filter so it changes only the representation, not probing or
fingerprint filtering. The guards are ordinary conditional expressions, not
volatile reads, forced branches, or assembly barriers. Only final checksums
escape through a volatile sink. A and D may compile identically: that is the
expected limiting case. E is a counterfactual, not asserted to be legal WF.
The optional SIMD extension is deferred to keep the causal comparison small.

## Machine and size tiers

Measured metadata, before timings:

- Apple M1 Pro, 8 logical CPUs, 32 GiB RAM, arm64; AC power, battery charging.
- macOS 26.6.2 (25G83), Darwin 25.6.0.
- Apple clang 21.0.0 (clang-2100.3.34.2), arm64-apple-darwin25.6.0.
- Reported cache line: 128 bytes. Performance-level L1D/L2: 128 KiB/12 MiB;
  efficiency-level L1D/L2: 64 KiB/4 MiB. `hw.l3cachesize` reports no value.
- No affinity, priority, frequency, or power setting is changed. macOS chooses
  core placement; frequency, migration, and other applications remain noise.

Capacities are powers of two. For L1/L2, choose the largest power of two at or
below 32 KiB/(P+32) and 1 MiB/(P+32), respectively. This leaves room for the
query array and covers the largest inline layout. The small table plus query
array fits even the reported 64 KiB L1D; the middle tier exceeds L1 and fits
the smaller 4 MiB L2. Check actual requested and allocator entry bytes.

The large tier chooses the smallest power of two whose compact A allocation
is at least 128 MiB. This is well beyond every reported cache; it is labeled
`large`, not an invented measured L3 capacity. Apple exposes no L3 size here,
so literal L3 residency and any unreported system cache cannot be verified.
Tables coexist in memory but only one representation is warmed and timed at
a time. Report footprint per active table, plus its query array.

## Workloads and timing protocol

- `hit`: uniform successful lookups of stored synthetic keys.
- `miss`: uniformly distributed disjoint keys, all absent.
- `mix`: exactly half hits and half misses in a shuffled stream.
- `dependent`: successful lookups; the returned field feeds integer mixing
  and the index of the NEXT query. This measures a serial dependency chain.
- `insert`: repeated builds from empty to the stated final load, four new
  insertions per successful lookup (with a possible partial last group).
  Capacity allocation, reset, and teardown are outside each timed build;
  per-entry `malloc` for C is inside. All variants have the same instantaneous
  loads, progressing from zero to the final load. This is not insertion at a
  constant final load, and includes neither growth nor deletion.

Use load factors 0.5 and 0.875, 2 warmup repetitions and 12 measured
repetitions. Rotate/reverse representation order deterministically between
repetitions. Query streams are deterministic, uniform pseudorandom samples
with replacement, generated outside timing; each stream has capacity entries.
For each lookup sample use at least 2^20 operations and at least one full
query stream. Insertion samples sum timed builds to at least 2^18 operations;
reset/free time is excluded and the number of timed builds is recorded.
Every representation receives identical inputs for a configuration. Distinct
workloads use distinct streams. No timing result selects iteration counts.

Retain every measurement as CSV, including operation count, time, checksum,
minor/major page-fault deltas, and involuntary context switches. Timed regions
use a monotonic wall clock. Record a separate empty clock-pair measurement to
bound timing overhead; do not subtract it. Report per-cell median and
25th/75th percentiles (linear interpolation), paired overhead median/IQR,
and raw min/max in the generated summary. Page faults in insertion can be
part of the allocation cost; lookup faults and scheduling noise limit claims.

Verify exact hits, misses, duplicates, collisions, payload contents, occupancy
counts, and identical probe geometry against independent expected values
before timing. An optimized and an address/undefined-sanitized self-test are
separate from performance measurement. Inspect `-O3` assembly of each lookup
kernel for each payload size; retain short hit-path excerpts in RESULTS.md.

## Build and run

From this directory, the dependency-free build is one command:

```sh
TMPDIR="$PWD" clang -O3 -std=c11 -Wall -Wextra -Werror bench.c -o bench
```

Run the self-test with `./bench --self-test`. Run all timings with
`./bench > raw.csv 2> run.log`. Run `perl summarize.pl raw.csv > summary.md`
for Markdown timing/ratio tables (Perl's standard library only). Generate
assembly with the compiler command below.

```sh
TMPDIR="$PWD" clang -O3 -std=c11 -S bench.c -o bench.s
```

Repository-local heavy runs use the existing `.github/run-check.pl` wrapper.
First inspect its default host lock and any owner PID. This task restricts
writes to this directory, so set `WHITEFOOT_CHECK_LOCK_DIR="$PWD/check.lock"`
and `TMPDIR="$PWD"` when invoking `perl ../../../.github/run-check.pl` here.
The local lock cannot exclude a later host-wide run; record that limitation
and do not run benchmarks concurrently with another known heavy command.
Compiler construction and benchmark execution are separate stages.

README.md owns this protocol, bench.c the reproducible implementation,
raw.csv the observations, summarize.pl their aggregation, and RESULTS.md the
analysis. Supporting metadata, run logs, and a generated summary document the run. Remove build binaries,
full assembly, and sanitizer scratch outputs after verification. Retain the
source and dated evidence until this research question is superseded.
