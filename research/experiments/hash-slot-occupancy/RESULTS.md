# Hash-slot occupancy results

## Measurement and decision

The registered comparison and 5 percent criterion are in [README.md](README.md).
This run does not establish a recurring 5 percent cost for the duplicate
occupancy check. Seven of the nine payload/tier groups fail the registered
screen; two are inconclusive because the identical-code A/D control exceeds
its noise limit. None passes all four required cells (successful and
dependent lookups at both loads), either for total B/D cost or same-stride
B/E cost. These observations do not justify adding a language mechanism
solely on the recorded "one null check per hit" concern. They also do not
establish that the check is always cheap: there are individual costly hit
cases, and the background-job overlap warrants a quiet replication before
making a close performance decision.

The final run completed in 1,557.23 seconds, exit 0, on the Apple M1 Pro and
Apple clang 21 described in README.md. It contains 5,400 measured samples and
900 warmup samples: 12 measured repetitions for each of 450 cells. Every
cross-representation checksum matched. [raw.csv](raw.csv) retains all samples;
[summary.md](summary.md) contains every median/IQR, paired ratio, min/max,
footprint, and resource total. The complete ns/op tables are also below.
Percentages here are medians of paired repetition ratios, not ratios of the
rounded medians printed in a time table.

| Tier | Payload bytes | B/D cells above criterion | B/E cells above criterion | Noisy A/D decision cells | Registered verdict |
| --- | --- | --- | --- | --- | --- |
| L1 | 8 | 0/4 | 0/4 | 0/4 | Threshold not met |
| L1 | 32 | 1/4 | 0/4 | 0/4 | Threshold not met |
| L1 | 128 | 1/4 | 0/4 | 0/4 | Threshold not met |
| L2 | 8 | 2/4 | 0/4 | 1/4 | Inconclusive |
| L2 | 32 | 0/4 | 1/4 | 0/4 | Threshold not met |
| L2 | 128 | 0/4 | 1/4 | 2/4 | Inconclusive |
| Large | 8 | 1/4 | 1/4 | 0/4 | Threshold not met |
| Large | 32 | 1/4 | 0/4 | 0/4 | Threshold not met |
| Large | 128 | 1/4 | 0/4 | 0/4 | Threshold not met |

The noisy decision cells are L2/P8/hit/load 0.5 (A/D +7.58 percent),
L2/P128/hit/load 0.5 (+10.77 percent), and L2/P128/hit/load 0.875
(+5.19 percent). There are also control disagreements outside the decision
workloads: for example L1/P8/miss/load 0.5 has A/D +12.61 percent despite
identical instructions. Layout addresses, branch history, scheduling, and
cache state are not independently controlled; such differences cannot be
attributed to a source-level occupancy rule. No quiet follow-up is claimed.

## Costs visible in the data

In the six large-table successful-lookup cells, inline tagging B is
2.36 to 13.28 percent slower than A and 3.08 to 11.47 percent slower than D.
Removing only the check, B/E, ranges from 0.61 to 7.33 percent. The largest
same-stride result is P8/load 0.5: B takes 14.18 [13.55, 14.34] ns/op,
E 12.93 [12.64, 13.42], with paired overhead 7.33 [5.15, 10.83] percent.
That hit case crosses the threshold; its dependent counterpart, 1.33
[0.10, 2.50] percent B/E, does not. The corresponding high-load hit is
2.89 [0.98, 4.47] percent. Thus one favorable case is not the registered
across-load result.

For dependent lookups across all 18 payload/tier/load cells, the B/E paired
median ranges from -0.65 to +2.64 percent. In L1, B can be roughly 6 to 7
percent slower than D on hits while B/E is below 1 percent; keeping the
stride loses most of the purported benefit. This separates a compact-layout
benefit from the generated consequence of deleting the check. It does not
identify a pure cache penalty: address generation and instruction placement
can matter even when the table fits a cache.

There are larger same-stride hit effects outside the large tier too:
L2/P32/load 0.5 has B/E 13.32 [9.72, 15.46] percent. Its B/D lower quartile
is -0.16 percent and its dependent B/E median is -0.32 percent, so it does
not pass the screen. L2/P128 hit B/E medians swing from -21.02 to +37.16
percent between loads, with wide spreads and failing A/D controls. They
are retained as noisy observations, not evidence for a 37 percent tag tax.

Boxing C has a larger, more consistent large-table cost. Large hits cost
19.58 to 48.69 percent more than A (20.03 to 47.52 percent more than D).
Large dependent lookups cost 32.18 to 48.92 percent more than A, or 29.88
to 48.51 percent more than D. For example, P8/load 0.5 takes 367.27
[365.10, 368.66] ns/op with C, versus 246.74 [245.64, 248.32] with A.
C is sometimes slightly faster in L1, so this is not a universal boxing tax.
Across insertion cells, C/A is about 1.47 to 3.99 times; this includes the
allocator and pointer representation, not just the null branch.

The following tables give the requested A/D comparisons for every large
hit and dependent case. Entries are paired overhead percent; the full IQRs
and all miss/mix/insert comparisons are in summary.md.

| Payload | Load | Workload | B/A | B/D | C/A | C/D | B/E |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 8 | 0.5 | hit | 13.28 | 11.47 | 48.69 | 47.35 | 7.33 |
| 8 | 0.875 | hit | 5.79 | 6.63 | 33.09 | 35.94 | 2.89 |
| 32 | 0.5 | hit | 10.23 | 9.99 | 47.91 | 47.52 | 3.60 |
| 32 | 0.875 | hit | 2.36 | 3.08 | 29.17 | 30.29 | 0.61 |
| 128 | 0.5 | hit | 7.40 | 7.86 | 23.41 | 23.67 | 2.97 |
| 128 | 0.875 | hit | 3.49 | 4.39 | 19.58 | 20.03 | 0.65 |
| 8 | 0.5 | dependent | 3.74 | 3.52 | 48.92 | 48.51 | 1.33 |
| 8 | 0.875 | dependent | 3.58 | 3.74 | 46.23 | 45.40 | 0.35 |
| 32 | 0.5 | dependent | 2.41 | 1.77 | 45.60 | 45.31 | -0.65 |
| 32 | 0.875 | dependent | 2.02 | 2.63 | 44.56 | 45.60 | 0.23 |
| 128 | 0.5 | dependent | 3.53 | 2.48 | 32.18 | 34.18 | 2.64 |
| 128 | 0.875 | dependent | 1.90 | -1.99 | 35.47 | 29.88 | 0.79 |

All 5,400 measured regions report zero minor and major page faults. They
record 17,728 involuntary context switches in total, including 10,606 in
dependent lookups. Absence of page faults does not establish cache residency
or absence of interference. The measured allocator entry sizes are 16, 48,
and 160 bytes for the 16-, 40-, and 136-byte requested entries; the footprint
tables include this rounding, but not allocator metadata. The largest active
table plus query array is 264 MiB (B/E, P8); the largest L1 case is below
32 KiB and the largest L2 case below 1 MiB, excluding allocator metadata.

## Instrumentation correction and environment

The first attempt used `CLOCK_MONOTONIC`. Its clock-pair observations were
min/median/p75 0 ns and maximum 1000 ns; elapsed samples were quantized in
microseconds, too coarse for a single small insertion build. That attempt
was stopped after 78.17 seconds (wrapper exit 143). Its 4,302 complete CSV
rows plus a partial final row
and log are retained as [coarse-clock.csv](coarse-clock.csv) and
[coarse-clock.log](coarse-clock.log); none enters the reported statistics.
[original-protocol.sha256](original-protocol.sha256) records that source.

The only C change for the restarted run selects `CLOCK_MONOTONIC_RAW` on
Apple, leaving the portable monotonic fallback elsewhere. The registered
criterion, workloads, operation counts, and analyzer are unchanged. Both
optimized and ASan/UBSan self-tests were rerun. The corrected clock-pair
observations were min/median 0 ns, p75 41 ns, and maximum 84 ns. Zero deltas
reflect the clock's finite resolution, not a zero-cost timer. No timer cost
is subtracted; small insertion builds still have quantization error, averaged
over many builds. Lookup samples contain at least a million operations.

The original host test owner finished before either run started. The final
run started at 04:29:30 UTC on 2026-09-20. A different worktree subsequently
started a short focused Cargo test, observed through the host lock at
04:29:36 UTC. Further program tests and builds were observed later; the
partial host-lock observations are in [host-activity.log](host-activity.log).
Continuing with those overlaps deviates from the protocol's intended quiet
run; the experiment-local lock could not exclude work in other worktrees
without writing outside the authorized directory. Thus this is an
uncontrolled desktop run, not guaranteed isolation. A/D
agreement cannot establish that other workloads had no effect. No final-run sample is
discarded for its timing or scheduling statistics. The registered A/D noise
check remains part of the conclusion.

## What the compiler emits

These are excerpts from the timed, inlined lookup loops, generated with
`TMPDIR="$PWD" clang -O3 -std=c11 -S bench.c -o bench.s` on the recorded
Apple clang 21 compiler. They are the 8-byte payload's ordinary lookup path;
hashing, empty-control detection, iteration, and checksum accumulation are
omitted. Original instruction operands and branch labels are retained.
`x0` points to the table descriptor; its offset 8 holds the payload/pointer
array base. The initial compare is the control-byte fingerprint match.

A, `_lookup_raw_8`:

```asm
LBB0_17:
    cmp     w16, w17
    b.ne    LBB0_16
    ldr     x17, [x0, #8]
    lsl     x3, x15, #4
    ldr     x3, [x17, x3]
    cmp     x3, x14
    b.ne    LBB0_16
    add     x14, x17, x15, lsl #4
    ldr     x14, [x14, #8]
```

B, `_lookup_tag_8`: the byte load and `cbz` are the second occupancy test.
It is neither folded into the fingerprint compare nor eliminated.

```asm
LBB4_18:
    cmp     w17, w3
    b.ne    LBB4_17
    ldr     x3, [x0, #8]
    madd    x3, x16, x14, x3
    ldrb    w5, [x3]
    cbz     w5, LBB4_17
    ldr     x5, [x3, #8]
    cmp     x5, x15
    b.ne    LBB4_17
    ldr     x15, [x3, #16]
```

C, `_lookup_box_8`: a pointer-array load, null branch, then a dependent
entry-key load. This is an additional address dependency, not just a test.

```asm
LBB8_18:
    cmp     w16, w17
    b.ne    LBB8_17
    ldr     x17, [x0, #8]
    ldr     x17, [x17, x15, lsl #3]
    cbz     x17, LBB8_17
    ldr     x3, [x17]
    cmp     x3, x14
    b.ne    LBB8_17
    ldr     x14, [x17, #8]
```

D, `_lookup_folded_8`: the same instructions as A, with different labels.
Separate function bodies are independently timed, not an assumed zero result.

```asm
LBB12_17:
    cmp     w16, w17
    b.ne    LBB12_16
    ldr     x17, [x0, #8]
    lsl     x3, x15, #4
    ldr     x3, [x17, x3]
    cmp     x3, x14
    b.ne    LBB12_16
    add     x14, x17, x15, lsl #4
    ldr     x14, [x14, #8]
```

E, `_lookup_proved_8`: the same 24-byte stride as B, but no tag load/branch.
Clang also selects a writeback addressing form for the key. Thus B/E measures
the generated consequence of removing the check, including this codegen
change, rather than the isolated latency of one instruction.

```asm
LBB16_17:
    cmp     w17, w3
    b.ne    LBB16_16
    ldr     x3, [x0, #8]
    madd    x3, x16, x14, x3
    ldr     x5, [x3, #8]!
    cmp     x5, x15
    b.ne    LBB16_16
    ldr     x15, [x3, #8]
```

All 15 kernels (five layouts by three payload sizes) were inspected. A/D
instruction bodies match after normalizing function names, block labels,
comments, and whitespace at all three sizes. Both the dependent and ordinary
loops retain B's `ldrb`/`cbz` and C's pointer `ldr`/`cbz`; neither A, D, nor E
has a second occupancy check. Per full kernel, B has six `ldrb` and nine `cbz`
instructions, C four/nine, and A/D/E four/seven. These static counts include
both loop modes and loop setup; they are not dynamic instructions per lookup.

For valid tables the B/C occupancy branch is always non-null after a matched
occupied control byte, including fingerprint collisions. It should therefore
be predictable; this experiment does not measure branch-misprediction counts.
The test runs once per matching fingerprint candidate, not literally only
once per successful operation. A failed lookup can also reach it.

## Attribution boundaries

B's tag has 7 bytes of alignment padding. Compared with compact A/D, the
payload stride grows from 16 to 24, 40 to 48, and 136 to 144 bytes for payloads
8, 32, and 128. Including the control byte, allocated table size rises by
47.1, 19.5, and 5.8 percent respectively. The extra tag load often shares a
cache line with the key; depending on slot alignment it can instead reach a
different line. E retains that stride, tag initialization, and slot placement
formula. B/E is the closer test of removing the guard; B/D combines guard,
stride, address generation, and memory-layout effects. Physical allocation
addresses still differ between tables.

C has a compact pointer array but separately allocated entries. Allocator
size classes and address locality affect its footprint; the returned pointer
must arrive before the entry can be addressed. Its insertion timings include
per-entry allocation, so they cannot be called the cost of a null branch.
No hardware cache-miss or branch counters were collected; exact causal
fractions for cache effects, prediction, and extra loads are not measured.

## Limits

- One actively used desktop machine and one compiler; no core affinity,
  frequency control, or performance-counter attribution. The pre-existing
  heavy test owner was inspected before timing. The local experiment lock
  cannot prevent a different worktree from subsequently starting a job.
- The machine reports L1/L2 but no L3 size. The large tier's compact table is
  at least 128 MiB, beyond the reported caches; no claim of a measured Apple
  L3 or system-cache capacity is made. A table's allocated bytes are not an
  exact measurement of the resident working set or cache hit rate.
  Supplementary context checked after registration: Philip Turner's
  [GPU microarchitecture measurements](https://github.com/philipturner/metal-benchmarks#overview)
  list 24 MB of L3/SLC for M1 Pro. The selected arrays exceed that published
  figure too, but this is external GPU evidence, not a local CPU cache
  measurement. It did not select or change the capacities.
- Synthetic uniformly distributed u64 keys and repeated fixed query streams,
  not traces from a real compiler or service. Small streams can train branch
  history. Mixed queries are exactly half absent; there are no adversarial
  collisions, hot-key skew, deletion, tombstones, or resizing.
- Scalar linear probing with a fingerprint filter, not hashbrown's SIMD
  groups or complete implementation. Its absolute timings do not measure
  hashbrown or establish a SwissTable performance ratio.
- A lookup reads only one payload field. Full object traversal, strings,
  hashing long keys, actual WF proof checking/lowering, and destruction of
  affine or linear resources are not measured. C's nullable pointer is a
  representation model, not a claim about current WF backend niche layout.
- Integer words stand in for the size of an arbitrary affine entry; they do
  not themselves exercise resource ownership. A concrete entry type with a
  cheap valid dummy value can initialize every slot with that value and use
  control bytes without an option test in WF today. The measured option
  layouts concern entries for which a real `None` representation is needed;
  they do not establish that every non-Copy type incurs this check.
- The dependent loop also loads its next key from the query array (8 to
  64 MiB in the large tier). This common memory dependency and integer mixing
  contribute to ns/op and can dilute relative representation overhead. It is
  not a measurement of the payload-access latency alone.
- Insert is a build from empty to the reported load, with one hit per four
  insertions. Table allocation, reset, and free are excluded; per-entry
  allocation, synthetic key generation (`key_at`), payload-value construction,
  and all payload writes are included. The common construction work can
  dilute representation ratios. It does not measure a
  constant-load mutation loop or the lifecycle cost of a complete table.
- IQRs describe run-to-run spread, not confidence intervals. The predeclared
  criterion is a research screen, not proof that a language feature is or is
  not needed. Any mechanism would still need a safe, usable specification.

## Reproduction and validation

The protocol, source, and analyzer digests recorded before timing are in
[protocol.sha256](protocol.sha256); machine metadata is in
[machine.txt](machine.txt). [validation.log](validation.log) records the
optimized build and self-test, followed by the separate ASan/UBSan build and
self-test; all exited 0. Each self-test checked 15,360 exact hit/miss results
across 30 layouts, plus collisions, duplicate insertion, every payload word,
build results, and independent batch/dependency oracles. Sanitizers were not
enabled during timing. The completed benchmark's diagnostic and wrapper
output is in [run.log](run.log).

Regenerating `summary.md` from `raw.csv` reproduces its bytes exactly. The
analyzer rejects the retained interrupted CSV for its incomplete last row;
it also checks full-run counts and each measured cell's 12 repetitions.
Independent recomputation verified all timing and paired-ratio quantiles,
the registered verdicts, footprints, resource totals, and 1,260 five-layout
checksum/operation-count groups. These are checks of this experiment's
implementation and evidence, not measurements from another machine.

The delivered files are uncommitted work under
`research/experiments/hash-slot-occupancy/` on the base revision recorded in
README.md. No specification, conformance case, compiler file, or design tree
is amended, and the repository-wide correctness gate is outside this task.

## Complete timing matrix

Median [p25, p75] ns/operation, 12 measured repetitions per cell.
A = raw, B = tagged, C = boxed, D = folded, E = same-stride proved control.

### L1, 8-byte payload

| Load | Workload | A | B | C | D | E |
| --- | --- | --- | --- | --- | --- | --- |
| 0.500 | hit | 1.80 [1.79, 1.80] | 1.76 [1.75, 1.76] | 1.64 [1.64, 1.64] | 1.79 [1.79, 1.80] | 1.75 [1.74, 1.75] |
| 0.500 | miss | 1.80 [1.80, 1.81] | 1.80 [1.80, 1.80] | 1.80 [1.80, 1.81] | 1.60 [1.60, 1.60] | 1.60 [1.60, 1.61] |
| 0.500 | mix | 1.75 [1.74, 1.77] | 1.74 [1.72, 1.79] | 1.67 [1.67, 1.75] | 1.75 [1.73, 1.82] | 1.74 [1.72, 1.76] |
| 0.500 | dependent | 14.52 [14.51, 14.56] | 15.21 [15.20, 15.28] | 14.82 [14.80, 14.86] | 14.57 [14.51, 14.61] | 15.14 [15.08, 15.23] |
| 0.500 | insert | 2.40 [2.39, 2.42] | 2.33 [2.33, 2.34] | 8.31 [8.28, 8.35] | 2.39 [2.39, 2.39] | 2.33 [2.32, 2.33] |
| 0.875 | hit | 2.64 [2.62, 2.66] | 2.58 [2.57, 2.59] | 2.44 [2.43, 2.45] | 2.63 [2.61, 2.66] | 2.59 [2.58, 2.59] |
| 0.875 | miss | 8.92 [8.69, 9.19] | 8.59 [8.37, 8.97] | 8.22 [8.12, 8.30] | 9.24 [8.91, 9.43] | 10.12 [9.71, 10.36] |
| 0.875 | mix | 5.38 [5.31, 5.48] | 5.26 [5.16, 5.51] | 5.51 [5.34, 5.65] | 5.57 [5.35, 5.65] | 5.23 [5.19, 5.30] |
| 0.875 | dependent | 18.94 [18.92, 18.99] | 19.62 [19.59, 19.63] | 19.20 [19.15, 19.61] | 18.93 [18.89, 19.06] | 19.57 [19.55, 19.70] |
| 0.875 | insert | 3.15 [3.12, 3.19] | 3.12 [3.07, 3.17] | 8.83 [8.65, 9.08] | 3.18 [3.08, 3.20] | 3.05 [3.04, 3.19] |

### L1, 32-byte payload

| Load | Workload | A | B | C | D | E |
| --- | --- | --- | --- | --- | --- | --- |
| 0.500 | hit | 1.65 [1.64, 1.67] | 1.76 [1.75, 1.77] | 1.65 [1.64, 1.65] | 1.65 [1.64, 1.65] | 1.74 [1.74, 1.75] |
| 0.500 | miss | 1.85 [1.80, 1.87] | 1.84 [1.80, 1.86] | 1.82 [1.80, 1.86] | 1.66 [1.60, 1.66] | 1.65 [1.60, 1.66] |
| 0.500 | mix | 1.67 [1.66, 1.68] | 1.72 [1.72, 1.73] | 1.67 [1.66, 1.68] | 1.69 [1.69, 1.70] | 1.73 [1.72, 1.73] |
| 0.500 | dependent | 15.65 [15.54, 15.83] | 15.60 [15.40, 15.80] | 15.27 [14.99, 15.42] | 15.49 [15.12, 15.59] | 15.53 [15.32, 15.63] |
| 0.500 | insert | 2.42 [2.34, 2.42] | 2.41 [2.37, 2.43] | 9.63 [9.43, 9.64] | 2.42 [2.41, 2.42] | 2.44 [2.43, 2.44] |
| 0.875 | hit | 2.63 [2.61, 2.67] | 2.73 [2.67, 2.74] | 2.58 [2.56, 2.59] | 2.60 [2.56, 2.63] | 2.73 [2.68, 2.73] |
| 0.875 | miss | 10.79 [10.24, 11.15] | 8.98 [8.42, 9.69] | 9.16 [8.56, 9.90] | 11.34 [11.01, 11.78] | 9.79 [9.52, 9.94] |
| 0.875 | mix | 5.88 [5.81, 5.98] | 5.46 [5.40, 5.65] | 5.42 [5.34, 5.52] | 6.14 [5.91, 6.40] | 5.80 [5.58, 6.14] |
| 0.875 | dependent | 20.58 [20.52, 20.65] | 20.65 [20.49, 20.80] | 20.11 [19.88, 20.23] | 20.56 [20.33, 20.68] | 20.41 [20.17, 20.61] |
| 0.875 | insert | 3.04 [3.03, 3.05] | 3.06 [3.05, 3.08] | 9.79 [9.74, 9.92] | 3.04 [3.04, 3.06] | 3.07 [3.06, 3.09] |

### L1, 128-byte payload

| Load | Workload | A | B | C | D | E |
| --- | --- | --- | --- | --- | --- | --- |
| 0.500 | hit | 1.70 [1.69, 1.70] | 1.81 [1.80, 1.82] | 1.69 [1.69, 1.69] | 1.70 [1.70, 1.70] | 1.80 [1.80, 1.81] |
| 0.500 | miss | 1.70 [1.70, 1.72] | 1.56 [1.55, 1.60] | 1.70 [1.69, 1.74] | 1.56 [1.56, 1.59] | 1.70 [1.69, 1.74] |
| 0.500 | mix | 1.71 [1.67, 1.75] | 1.73 [1.71, 1.77] | 1.68 [1.66, 1.72] | 1.76 [1.72, 1.78] | 1.71 [1.67, 1.74] |
| 0.500 | dependent | 13.70 [13.64, 13.75] | 13.82 [13.81, 13.96] | 13.46 [13.40, 13.63] | 13.68 [13.64, 13.78] | 13.70 [13.68, 13.71] |
| 0.500 | insert | 3.50 [3.50, 3.51] | 3.57 [3.56, 3.58] | 10.18 [10.11, 10.62] | 3.50 [3.50, 3.52] | 3.55 [3.54, 3.56] |
| 0.875 | hit | 2.34 [2.34, 2.34] | 2.44 [2.44, 2.44] | 2.30 [2.30, 2.30] | 2.34 [2.33, 2.34] | 2.44 [2.44, 2.44] |
| 0.875 | miss | 5.09 [5.08, 5.19] | 5.10 [5.09, 5.11] | 5.10 [5.07, 5.18] | 5.10 [5.08, 5.15] | 5.09 [5.08, 5.16] |
| 0.875 | mix | 3.55 [3.55, 3.57] | 3.64 [3.60, 3.70] | 3.54 [3.52, 3.57] | 3.55 [3.54, 3.58] | 3.61 [3.60, 3.62] |
| 0.875 | dependent | 19.18 [19.15, 19.32] | 19.16 [19.13, 19.24] | 18.68 [18.62, 18.73] | 19.17 [19.15, 19.28] | 19.13 [19.10, 19.18] |
| 0.875 | insert | 3.83 [3.82, 3.85] | 3.88 [3.87, 3.89] | 9.82 [9.79, 9.88] | 3.82 [3.82, 3.84] | 3.87 [3.86, 3.87] |

### L2, 8-byte payload

| Load | Workload | A | B | C | D | E |
| --- | --- | --- | --- | --- | --- | --- |
| 0.500 | hit | 5.65 [5.39, 5.85] | 5.60 [5.38, 5.67] | 5.14 [5.05, 5.51] | 5.23 [5.17, 5.31] | 5.41 [5.26, 5.54] |
| 0.500 | miss | 10.49 [9.98, 10.93] | 10.85 [10.79, 10.98] | 10.84 [9.84, 10.97] | 11.11 [10.94, 11.13] | 9.21 [8.89, 10.13] |
| 0.500 | mix | 9.89 [8.97, 11.23] | 10.49 [10.12, 11.27] | 8.88 [8.56, 9.15] | 10.91 [9.91, 11.03] | 10.50 [10.15, 10.63] |
| 0.500 | dependent | 20.30 [19.95, 21.10] | 21.99 [21.46, 22.78] | 24.15 [23.83, 25.28] | 20.05 [19.86, 20.67] | 21.89 [21.09, 22.13] |
| 0.500 | insert | 4.79 [4.69, 5.05] | 5.07 [4.87, 5.23] | 11.82 [11.79, 11.90] | 4.59 [4.42, 4.66] | 6.51 [6.46, 6.57] |
| 0.875 | hit | 11.90 [11.84, 12.15] | 11.91 [11.40, 12.03] | 12.12 [12.03, 12.42] | 11.42 [11.25, 11.83] | 11.91 [11.76, 12.24] |
| 0.875 | miss | 36.27 [36.24, 36.63] | 36.85 [36.73, 37.16] | 36.43 [36.29, 36.61] | 36.81 [36.77, 37.06] | 36.52 [36.24, 36.90] |
| 0.875 | mix | 24.37 [24.33, 24.54] | 25.30 [25.23, 25.65] | 25.51 [25.48, 25.61] | 24.87 [24.86, 25.23] | 25.33 [25.28, 25.45] |
| 0.875 | dependent | 25.33 [25.24, 25.47] | 27.18 [27.16, 27.28] | 30.18 [30.15, 30.56] | 25.35 [25.22, 25.47] | 26.96 [26.76, 27.00] |
| 0.875 | insert | 10.86 [10.68, 11.21] | 11.00 [10.92, 11.02] | 16.76 [16.73, 16.92] | 10.74 [10.65, 10.92] | 11.62 [11.61, 11.64] |

### L2, 32-byte payload

| Load | Workload | A | B | C | D | E |
| --- | --- | --- | --- | --- | --- | --- |
| 0.500 | hit | 5.05 [5.01, 5.42] | 5.59 [5.54, 5.63] | 5.68 [5.52, 5.74] | 5.06 [5.02, 5.50] | 4.87 [4.84, 4.98] |
| 0.500 | miss | 10.55 [10.30, 10.72] | 10.82 [10.61, 11.05] | 10.76 [10.34, 11.12] | 9.93 [9.76, 10.21] | 10.61 [10.26, 11.00] |
| 0.500 | mix | 10.95 [10.23, 11.13] | 10.62 [9.37, 10.94] | 11.14 [10.75, 11.31] | 9.28 [8.64, 9.87] | 10.85 [10.40, 11.02] |
| 0.500 | dependent | 23.17 [22.81, 23.33] | 23.15 [22.99, 23.35] | 26.09 [25.96, 26.32] | 22.77 [22.67, 22.92] | 23.29 [23.19, 23.35] |
| 0.500 | insert | 5.51 [5.30, 6.63] | 6.11 [5.87, 6.65] | 12.46 [12.31, 12.62] | 5.86 [5.73, 5.95] | 6.16 [5.28, 6.21] |
| 0.875 | hit | 12.08 [11.98, 12.24] | 12.30 [12.24, 12.38] | 12.35 [12.16, 12.46] | 11.83 [11.69, 12.12] | 11.73 [11.67, 12.04] |
| 0.875 | miss | 38.36 [37.99, 38.57] | 37.82 [37.60, 38.00] | 37.79 [37.59, 37.90] | 37.88 [37.63, 38.07] | 38.34 [38.16, 38.56] |
| 0.875 | mix | 26.82 [26.58, 26.89] | 26.23 [26.08, 26.33] | 26.08 [25.95, 26.48] | 26.12 [25.98, 26.43] | 26.42 [26.23, 26.54] |
| 0.875 | dependent | 28.46 [28.31, 28.69] | 28.58 [28.55, 28.82] | 31.50 [31.30, 31.80] | 28.30 [28.25, 28.52] | 28.57 [28.51, 28.78] |
| 0.875 | insert | 12.04 [11.97, 12.20] | 12.14 [12.05, 12.23] | 18.43 [18.23, 18.52] | 12.16 [12.09, 12.22] | 11.39 [11.35, 11.51] |

### L2, 128-byte payload

| Load | Workload | A | B | C | D | E |
| --- | --- | --- | --- | --- | --- | --- |
| 0.500 | hit | 2.86 [2.72, 3.76] | 2.27 [2.25, 2.27] | 2.63 [2.51, 3.43] | 2.53 [2.06, 3.19] | 2.94 [2.80, 3.36] |
| 0.500 | miss | 2.45 [2.08, 2.67] | 1.88 [1.85, 2.07] | 5.60 [4.34, 6.25] | 2.22 [2.03, 3.49] | 2.09 [2.06, 2.22] |
| 0.500 | mix | 1.90 [1.89, 1.98] | 2.00 [1.98, 2.07] | 3.72 [3.19, 4.24] | 1.94 [1.89, 3.00] | 1.98 [1.96, 1.98] |
| 0.500 | dependent | 17.36 [17.01, 17.48] | 18.08 [17.79, 18.24] | 19.39 [19.05, 19.60] | 17.36 [17.10, 17.53] | 17.86 [17.56, 18.07] |
| 0.500 | insert | 3.64 [3.63, 3.68] | 3.75 [3.71, 3.78] | 11.61 [11.57, 11.90] | 3.69 [3.65, 3.74] | 3.80 [3.75, 3.86] |
| 0.875 | hit | 8.88 [5.72, 9.67] | 9.12 [7.34, 9.32] | 7.31 [6.83, 7.99] | 9.16 [6.66, 9.23] | 6.57 [6.47, 6.88] |
| 0.875 | miss | 26.51 [26.36, 26.87] | 26.45 [26.38, 26.92] | 27.02 [26.98, 27.17] | 26.36 [26.31, 26.45] | 26.35 [26.32, 26.44] |
| 0.875 | mix | 16.93 [16.82, 17.06] | 17.22 [17.04, 17.23] | 16.93 [16.85, 17.10] | 16.68 [16.57, 16.88] | 17.28 [17.08, 17.40] |
| 0.875 | dependent | 22.88 [22.79, 23.03] | 23.62 [23.58, 23.79] | 24.24 [24.11, 24.33] | 22.92 [22.83, 23.29] | 23.12 [23.06, 23.21] |
| 0.875 | insert | 10.52 [7.39, 10.76] | 6.73 [5.63, 7.68] | 16.14 [15.90, 16.61] | 9.28 [9.02, 9.72] | 9.73 [8.60, 10.09] |

### large, 8-byte payload

| Load | Workload | A | B | C | D | E |
| --- | --- | --- | --- | --- | --- | --- |
| 0.500 | hit | 12.22 [11.90, 12.48] | 14.18 [13.55, 14.34] | 18.31 [17.98, 18.64] | 12.38 [12.12, 12.55] | 12.93 [12.64, 13.42] |
| 0.500 | miss | 16.27 [15.75, 17.03] | 16.39 [15.73, 17.74] | 16.99 [16.09, 17.95] | 16.23 [15.86, 16.97] | 16.54 [15.84, 17.06] |
| 0.500 | mix | 25.51 [25.38, 25.90] | 26.27 [26.16, 27.15] | 29.33 [29.22, 29.68] | 26.36 [26.09, 27.28] | 26.33 [25.72, 26.82] |
| 0.500 | dependent | 246.74 [245.64, 248.32] | 255.64 [254.32, 258.55] | 367.27 [365.10, 368.66] | 246.67 [244.79, 249.97] | 252.40 [251.18, 255.88] |
| 0.500 | insert | 13.35 [13.19, 13.68] | 14.02 [13.92, 14.63] | 28.51 [28.08, 29.08] | 13.31 [13.18, 13.45] | 14.03 [13.85, 14.13] |
| 0.875 | hit | 24.50 [24.15, 24.77] | 25.80 [25.29, 26.14] | 32.55 [32.29, 32.68] | 24.08 [23.72, 24.41] | 25.19 [24.88, 25.51] |
| 0.875 | miss | 55.70 [55.03, 57.09] | 59.61 [58.55, 60.85] | 76.54 [74.14, 77.11] | 56.67 [55.30, 57.99] | 58.49 [56.91, 59.78] |
| 0.875 | mix | 56.71 [56.41, 57.47] | 58.79 [58.46, 59.75] | 76.11 [75.89, 77.65] | 57.28 [56.05, 57.76] | 59.37 [58.78, 59.75] |
| 0.875 | dependent | 269.63 [267.65, 270.84] | 279.68 [276.32, 281.53] | 393.46 [392.59, 395.81] | 270.18 [269.23, 270.61] | 278.42 [275.02, 279.00] |
| 0.875 | insert | 24.29 [24.15, 24.83] | 25.32 [25.18, 25.72] | 38.28 [37.90, 38.99] | 24.92 [24.32, 25.22] | 25.48 [25.01, 25.76] |

### large, 32-byte payload

| Load | Workload | A | B | C | D | E |
| --- | --- | --- | --- | --- | --- | --- |
| 0.500 | hit | 11.41 [11.37, 11.70] | 12.59 [12.55, 12.84] | 16.89 [16.82, 17.02] | 11.46 [11.44, 11.49] | 12.15 [12.11, 12.19] |
| 0.500 | miss | 14.30 [14.12, 14.61] | 14.21 [14.05, 14.55] | 14.51 [14.19, 14.99] | 14.59 [14.23, 14.90] | 14.48 [14.08, 14.75] |
| 0.500 | mix | 23.15 [23.10, 23.21] | 22.38 [22.34, 22.46] | 24.31 [24.24, 24.37] | 22.18 [22.10, 22.56] | 22.90 [22.79, 23.49] |
| 0.500 | dependent | 232.07 [230.05, 233.37] | 235.38 [235.02, 236.36] | 338.47 [336.34, 340.80] | 231.48 [229.90, 233.48] | 237.37 [235.04, 238.85] |
| 0.500 | insert | 12.91 [12.86, 13.04] | 13.20 [13.17, 13.45] | 27.65 [27.47, 28.75] | 12.81 [12.76, 12.88] | 13.09 [13.07, 13.34] |
| 0.875 | hit | 22.16 [21.95, 22.30] | 22.74 [22.63, 22.89] | 28.48 [28.43, 28.86] | 22.00 [21.86, 22.53] | 22.56 [22.35, 22.81] |
| 0.875 | miss | 45.65 [45.08, 47.15] | 48.61 [46.83, 49.45] | 63.66 [62.43, 66.53] | 45.50 [44.64, 47.48] | 47.64 [46.56, 48.72] |
| 0.875 | mix | 50.17 [49.43, 50.51] | 52.01 [51.70, 52.42] | 67.89 [67.49, 68.27] | 48.66 [47.98, 49.37] | 51.74 [51.05, 52.56] |
| 0.875 | dependent | 251.07 [247.49, 253.00] | 256.03 [252.82, 257.82] | 363.08 [361.01, 363.78] | 249.39 [247.72, 251.46] | 254.32 [251.78, 256.34] |
| 0.875 | insert | 21.67 [21.56, 22.29] | 22.76 [22.30, 22.89] | 36.73 [36.12, 36.87] | 21.37 [21.05, 21.83] | 22.09 [21.89, 22.89] |

### large, 128-byte payload

| Load | Workload | A | B | C | D | E |
| --- | --- | --- | --- | --- | --- | --- |
| 0.500 | hit | 10.38 [10.25, 10.52] | 11.18 [11.06, 11.24] | 12.88 [12.65, 13.14] | 10.33 [10.22, 10.47] | 10.88 [10.73, 11.18] |
| 0.500 | miss | 13.22 [13.19, 13.59] | 13.20 [13.12, 13.67] | 13.46 [13.23, 13.64] | 13.32 [13.20, 13.53] | 13.23 [13.17, 13.64] |
| 0.500 | mix | 15.62 [15.22, 15.78] | 16.05 [15.97, 16.56] | 17.84 [17.71, 18.37] | 15.41 [15.19, 15.69] | 15.38 [15.35, 15.88] |
| 0.500 | dependent | 154.69 [148.57, 161.59] | 162.03 [155.47, 164.79] | 209.61 [201.86, 212.86] | 150.87 [147.25, 165.37] | 155.48 [151.55, 159.27] |
| 0.500 | insert | 19.15 [18.99, 19.27] | 20.48 [20.25, 20.65] | 28.83 [28.31, 29.55] | 19.42 [19.19, 19.63] | 20.48 [20.30, 20.59] |
| 0.875 | hit | 18.65 [18.26, 18.85] | 19.41 [18.99, 19.61] | 22.36 [21.85, 22.71] | 18.62 [18.13, 18.72] | 19.15 [18.90, 19.36] |
| 0.875 | miss | 37.85 [36.70, 39.25] | 39.12 [37.89, 40.17] | 44.12 [42.05, 44.46] | 37.58 [36.56, 39.31] | 39.08 [37.79, 40.88] |
| 0.875 | mix | 40.34 [40.24, 41.11] | 41.33 [40.81, 42.06] | 49.22 [48.12, 51.24] | 40.64 [40.07, 41.48] | 41.61 [40.96, 42.52] |
| 0.875 | dependent | 173.30 [168.61, 180.75] | 176.49 [170.27, 179.46] | 234.35 [227.53, 242.14] | 176.28 [169.76, 186.87] | 175.51 [171.67, 181.65] |
| 0.875 | insert | 23.77 [23.55, 24.10] | 24.86 [24.58, 25.21] | 34.95 [34.57, 35.93] | 23.73 [23.54, 24.00] | 25.01 [24.50, 25.29] |
