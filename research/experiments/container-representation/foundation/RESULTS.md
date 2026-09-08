# Container foundation experiments

## Finite construction protocol

`model.rs` is a safe Rust concrete state model distinct from the existing
`../authority/model.rs` range-certificate experiment. It tests responsibilities
that interval coverage alone does not establish: allocation refusal, helper
failure recovery, conversion to a full state, zero extent, relocation order, and
backing retirement. Its public operations inspect an owner descriptor; a separate
per-slot oracle counts each linear value exactly once, checks initialized coverage
and allocation identity, and preserves the payload snapshot of each observer.

`make check` runs 165 construction/failure traces at capacities zero through eight,
including every constructed prefix and every split between two helper batches;
42 allocation-first relocation traces at source capacities zero through six and
destination capacities through eight; and nine publication/retirement traces.
Allocation failure leaves the complete state unchanged. Every successful transfer
preserves value order, and explicit cleanup discharges each value exactly once.
Five tests include deliberately weakened initialization, conservation, backing
identity, and DONE-as-retirement controls which the independent oracle detects.

The modeled helper split is an execution boundary, not a verified Whitefoot
contract. A payload is one abstract linear identity, not a recursively constructed
source type. The model reserves logical allocation identities even at zero extent;
it does not select the physical zero-sized allocation or provider resource rule.
It conservatively excludes all outstanding loans during state conversion. Exact
address-preserving transfer under a loan could be sound with a separate provenance
rule; this model does not disprove it. No symbolic generic proof, representation
abstraction, actual concurrent execution, allocator ABI, or proof erasure is tested.

This protocol is sufficient to distinguish full initialization from a partial
prefix and completed computation from returned borrowing authority. It cannot by
itself select the source API, a new compiler-owned builder, or public raw storage.

## Current Whitefoot large-result boundary

`large-result.wf` is an accepted current-language probe: a producer returns either
an error or a record containing 4,096 bytes; a helper immediately matches that
result and appends the record to a two-slot inline run. The executable checks all
4,096 successful payload bytes and the empty result on refusal. `make check`
builds and executes it as well as retaining its unoptimized LLVM for inspection.
This is one capability/cost witness, not an application-distribution sample.

On the merged `f2a29866` compiler, the producer constructs array, record, and enum
storage separately before delivery. To separate that representation question from
inlining, `make measure` also exports the two producer/helper symbols and marks the
producer `noinline` in an experimental LLVM copy. Apple Clang 21 `-O2` retains a
whole `Result<Record,u8>` and a whole `Record` in the helper frame, as well as an
8,208-byte final-run initialization. The retained call is a deliberate native
control, not a source acceptance rule or a default scheduling claim. Symbol
renaming affects only this experiment; production lowering may not select behavior
by producer names. No timing or portable stack-size ranking is inferred from this
LLVM shape, and the adapted module is not claimed to test an implemented alternative
ABI. The full ordinary source executable, not the adaptation, supplies the behavior
check.

The relevant construction distinction is therefore real at this compiler boundary:
logical ownership transfer into a run does not by itself promise construction at
that slot. It does not follow that an explicit source-level raw destination is
necessary; a generic internal result-destination design remains a separate candidate.

## Finite result-tree construction model

`construction.rs` is a safe Rust model of that candidate, measured 2026-09-07.
It lowers seven finite source cases through both a whole-result strategy and a
generic destination-tree strategy, then executes the lowering events in a
destination-indexed storage machine. A separate map/set oracle checks the result
value, effect order, release order, and final sink. The model derives these counts
from executed whole-slot allocation and whole-record transfer events; the logical
record size is 4,096 bytes.

| Source case | Whole-result slots / transfers | Result-tree slots / transfers |
| --- | ---: | ---: |
| fresh total record | 1 / 1 | 0 / 0 |
| fresh `Ok(record)` | 1 / 1 | 0 / 0 |
| `Err` before fields | 1 / 0 | 0 / 0 |
| `Err` after two fields | 1 / 0 | 0 / 0 |
| forward existing owner | 1 / 2 | 0 / 1 |
| force whole-result observation | 1 / 1 | 1 / 1 |
| fresh path releasing an unused input owner | 1 / 1 | 0 / 0 |

All 14 strategy executions match the oracle. The late error releases its two
constructed fields exactly once in reverse construction order. The mixed fresh
path explicitly releases its unused input payload and header before publishing
the new record; the alternate path transfers that input owner. Validation checks
every finite path, including unselected alternatives, and rejects an unconsumed
input, duplicate release, incomplete record, and input/source token collision.
Six mutated lowering streams independently detect a misrouted field, omitted
transfer, premature publication, duplicate owner, duplicate publication, and
duplicate release. Call-site renaming does not change lowering behavior. A claimed
direct guarantee is admitted only for an all-fresh, unobserved result-tree producer;
a reachable forwarding path or whole-result observation rejects it deterministically.

This model does not implement a Whitefoot ABI, source form, proof rule, staged
call path, allocation, or borrow retirement. Its two-field records and finite
paths establish ownership accounting and distinguish construction events; they
do not prove that production lowering can route every admitted producer, that
the event counts predict native copies, or that writer-visible vacant storage is
unnecessary beyond these cases. Forwarding an existing owner retains one intrinsic
transfer, and whole-result observation retains one materialization in the modeled
result-tree strategy.

## Safe Rust reusable-backing control

`rust-baseline.rs` is a safe, `#![forbid(unsafe_code)]` control for the one-block
lifecycle within the fixed-block pool contract. It reserves an empty `Vec<Tracked>` with `try_reserve_exact`, models
producer refusal after every initialized prefix `0..=N`, clears exactly that prefix,
and reuses the returned vector. A successful full vector is converted through the
safe `TryFrom<Vec<T>> for Box<[T; N]>` implementation, converted back to `Vec<T>`,
cleared, and checked out three times. An incomplete `TryFrom` returns the original
vector owner before cleanup. Every `Tracked` value is non-copy, carries a unique
ordinal whose order is checked before and after each successful conversion, and
records that it is destroyed exactly once. Drop equality is checked after every
failure cleanup and every completed-owner retirement, not merely at process exit.

The standard library's pinned source states that `try_reserve_exact` requests the
minimum but the allocator may provide more capacity [Rust reserve source][rust-reserve].
The safe vector-to-boxed-array conversion returns the vector on a length mismatch;
on success it uses `into_boxed_slice`, which discards spare capacity, and is in-place
when capacity already equals `N` [Rust conversion source][rust-convert]. Thus safe
Rust can express allocate, prefix-fill, cleanup, publication, and reuse without
writer-authored `unsafe`. A Whitefoot same-backing transition would offer a stronger
statically checked resource contract when `len=cap=N`; it is not needed merely to
make this behavior expressible.

The checked-in source had SHA-256
`24754a87be0838b0676aaa16eaeea4c1f5d0ad4bee0638781ccd496ccaa20ea0` when run on
2026-09-07 with `rustc 1.98.1 (48a229cea 2026-09-01)`, host
`aarch64-apple-darwin`, at `-C opt-level=2`:

```text
extent=0 reserved_capacity=0 failure_prefixes=1 incomplete_try_from_pointer_same=0/0 failure_clear_pointer_same=1/1 exact_capacity_conversions=3/3 vec_to_box_pointer_same=3/3 box_to_vec_pointer_same=3/3 created_and_dropped=0
extent=8 reserved_capacity=8 failure_prefixes=9 incomplete_try_from_pointer_same=8/8 failure_clear_pointer_same=9/9 exact_capacity_conversions=3/3 vec_to_box_pointer_same=3/3 box_to_vec_pointer_same=3/3 created_and_dropped=60
```

Capacity 8 and the pointer equalities are observations of this allocator and run.
The conditional in-place result when `len=cap=N` is the documented guarantee; an
overallocated reservation may shrink or reallocate during conversion. For `N=0`,
the pointer is a non-dereferenced sentinel, so equality does not witness retained
allocated bytes. This probe does not execute multi-block exhaustion or out-of-order
return, inject allocator refusal, or instrument allocator calls. Pointer equality
alone cannot count allocation operations. The probe contains no timing and supports no allocator or
throughput ranking. It remains wired into this experiment's `make check` until a
matched Whitefoot fixed-block experiment replaces it.

## Native layout comparison

Measured 2026-09-07 against compiler baseline
`f2a298666fcf3ac9ea454087623773fd06f4bc6f`. The measured `layout.c` has SHA-256
`64f7a1c8ca65b4006f9f269f04decf77967800272703d79b0d73011000c16a04`.

This is a native C11 representation-cost control. It is not an accepted
Whitefoot program, a language proposal, or evidence that a proposed checker,
proof rule, or backend implementation works. Retire this probe when maintained
compiler experiments cover the same representation questions.

## Question and controls

The experiment uses a fixed capacity of 4,096 and a 32-byte, eight-byte-aligned
payload. It distinguishes four semantic states:

- a fully initialized fixed payload with no dynamic state;
- an initialized prefix represented by `len`, `cap`, and payload backing;
- a circular window represented by `head`, `len`, `cap`, and payload backing;
- arbitrary nullable occupancy, represented either by a separate one-bit-per-slot
  validity map and payload array or by one explicit C byte tag beside every payload.

Full, prefix, and ring are not rival answers to one operation. The timed row for
each is the common operation they do share: copy 4,096 live payloads in logical
order. The ring starts at physical slot 4,059 and wraps. Partial prefix and ring
states are correctness and byte-count witnesses only.

The nullable pair is the matched comparison. Both encodings hold exactly the same
payloads and compact the present values in ascending logical-index order. `dense`
has 3,584 present values (all indices except multiples of eight). `sparse` has 256
present values (every sixteenth index). `clustered` has the same 256 values in
eight clusters of 32. The separate implementation scans 64 validity words and
iterates their set bits; the tagged implementation examines the tag of each of
the 4,096 elements. That is the natural traversal each physical representation
enables, rather than extra work added to one rival.

Initialization is outside the timed region. Both nullable forms initialize only
present payloads, exactly 32 bytes per present value; neither zeros absent payload
storage. Both copy exactly those same payload bytes. The validity-state counts
state the bytes that must receive known validity state in these concrete
encodings. They do not include avoidable payload initialization. Each reported
layout uses one allocation; allocator bookkeeping is unmeasured. The common
destination workspace is excluded from per-container backing size and allocation
counts.

Every run first checks these witnesses and exits nonzero on a mismatch:

- full, full-prefix, and full wrapped-ring copies are byte-for-byte equal and have
  the same checksum;
- a quarter prefix and a 17-element ring crossing the physical end reproduce their
  construction sequence;
- zero-extent copies touch no source or destination, and empty prefix, ring, and
  nullable states copy no payload;
- separate and tagged nullable outputs are byte-for-byte equal for all three
  populations.

The timed functions are out of line. Each invocation copies its output and adds
one 64-bit word per copied payload to a witness; every repetition contributes to
a volatile published value. Byte equality is checked independently before timing.
The final witness was `34359743826` at both optimization levels. This keeps the
common work observable without adding work to only one representation. The
witness is still part of the timed operation, especially at high occupancy, so
these are copy-plus-observation timings rather than pure memory-copy throughput.

## Structural result

`peak backing` is the exact `sizeof` of one allocation on this target. It excludes
allocator overhead and the common output workspace.

| Representation | Payload capacity | State bytes | Padding from state/payload layout | Layout and peak backing | Allocations |
| --- | ---: | ---: | ---: | ---: | ---: |
| full | 131,072 | 0 | 0 | 131,072 | 1 |
| prefix | 131,072 | 16 | 0 | 131,088 | 1 |
| ring | 131,072 | 24 | 0 | 131,096 | 1 |
| nullable, separate bitmap | 131,072 | 512 | 0 | 131,584 | 1 |
| nullable, explicit C tag | 131,072 | 4,096 | 28,672 | 163,840 | 1 |

The explicit tagged slot is 40 bytes: its payload begins at offset eight after a
one-byte tag. At this payload alignment the tagged allocation is 32,256 bytes, or
24.51%, larger than the separate form. The separate map adds 0.39% over payload
capacity; the tagged array adds 25%. This is an ordinary C ABI result for the
declared struct. It is not a claim about Rust `Option`, which may use a niche or a
different target-specific layout.

| State | Live payloads | Payload bytes initialized | Distinct state bytes defined | Payload bytes copied |
| --- | ---: | ---: | ---: | ---: |
| full | 4,096 | 131,072 | 0 | 131,072 |
| prefix, full | 4,096 | 131,072 | 16 | 131,072 |
| prefix, quarter | 1,024 | 32,768 | 16 | 32,768 |
| ring, full and wrapped | 4,096 | 131,072 | 24 | 131,072 |
| ring, wrapped 17 | 17 | 544 | 24 | 544 |
| nullable dense | 3,584 | 114,688 | 512 separate / 4,096 tagged | 114,688 |
| nullable sparse | 256 | 8,192 | 512 separate / 4,096 tagged | 8,192 |
| nullable clustered | 256 | 8,192 | 512 separate / 4,096 tagged | 8,192 |

The state column counts each byte whose value must be defined once. It is not a
count of executed stores or cache-line traffic: the reference setup first clears
all validity state and then sets present entries, and those setup operations are
untimed. The fixed backing capacity is the same for full, prefix, and ring. Their
initialized payload count differs because their semantics differ. The table
therefore does not turn a prefix's unused capacity into initialization work or
compare a 17-element ring with a full array as if they performed one operation.

## Descriptive timing

Host: macOS 26.6.2 (`Darwin 25.6.0`, arm64). The sandbox did not expose a hardware
model string. Compiler: Apple clang 21.0.0 (`clang-2100.1.1.101`), target
`arm64-apple-darwin25.6.0`. Each entry is the median of nine process-local samples;
each sample performs 1,200 copies after untimed correctness checks and warm-up.
The order rotates between the compared implementations. Nanoseconds are per one
complete copy/compaction.

| Work | `-O3` median ns | Ratio | `-O2` median ns | Ratio |
| --- | ---: | ---: | ---: | ---: |
| full | 3,484.167 | 1.000 full | 3,309.167 | 1.000 full |
| full prefix | 3,510.833 | 1.008 / full | 3,402.500 | 1.028 / full |
| full wrapped ring | 3,978.333 | 1.142 / full | 3,792.500 | 1.146 / full |
| nullable dense, separate | 3,465.000 | 1.000 separate | 3,376.667 | 1.000 separate |
| nullable dense, tagged | 3,776.667 | 1.090 / separate | 3,600.833 | 1.066 / separate |
| nullable sparse, separate | 235.833 | 1.000 separate | 243.333 | 1.000 separate |
| nullable sparse, tagged | 1,545.000 | **6.551 / separate** | 1,565.000 | **6.432 / separate** |
| nullable clustered, separate | 245.000 | 1.000 separate | 258.333 | 1.000 separate |
| nullable clustered, tagged | 1,546.667 | **6.313 / separate** | 1,595.833 | **6.177 / separate** |

The low-occupancy nullable cases discriminate: every tagged sparse and clustered
sample is above every corresponding separate sample at both optimization levels,
with 6.18x to 6.55x median ratios. The packed form examines 64 validity words and
then the 256 present payloads; the tagged form must examine 4,096 strided tags.
Dense medians favor separate validity by 7% to 9%, but the `-O2` sample ranges
overlap. Prefix remains close to full. The ring's two-span logical traversal is
about 14% slower than full here, but ring and full have different state semantics
and this row is not a representation-rival selection. This one host and three
synthetic patterns do not establish a general throughput ranking or a
workload-frequency claim. The common one-word witness can also reduce sensitivity
to metadata traversal at high occupancy; it does not affect the exact layout and
byte counts.

Raw `-O3` samples, in collection order, ns per copy:

| Work | Samples |
| --- | --- |
| full | 3484.167, 3294.167, 3665.833, 4203.333, 5740.833, 3478.333, 3397.500, 3473.333, 3496.667 |
| full prefix | 3534.167, 3566.667, 6090.833, 4024.167, 3510.833, 3393.333, 3441.667, 3483.333, 3460.000 |
| full wrapped ring | 3950.833, 3761.667, 3880.833, 5171.667, 6706.667, 4021.667, 4200.000, 3923.333, 3978.333 |
| nullable dense, separate | 3269.167, 3445.833, 3447.500, 3482.500, 3468.333, 3456.667, 3465.000, 3488.333, 3503.333 |
| nullable dense, tagged | 3545.000, 3558.333, 4150.833, 3776.667, 3775.000, 5158.333, 3779.167, 3916.667, 3711.667 |
| nullable sparse, separate | 252.500, 235.833, 235.833, 237.500, 236.667, 235.833, 235.833, 235.833, 235.833 |
| nullable sparse, tagged | 1564.167, 1545.000, 1543.333, 1541.667, 1545.000, 1544.167, 1542.500, 1550.000, 1559.167 |
| nullable clustered, separate | 245.000, 244.167, 245.000, 245.000, 244.167, 246.667, 251.667, 245.000, 245.000 |
| nullable clustered, tagged | 1549.167, 1545.833, 1546.667, 1546.667, 1546.667, 1554.167, 1547.500, 1547.500, 1546.667 |

Raw `-O2` samples, in collection order, ns per copy:

| Work | Samples |
| --- | --- |
| full | 3590.833, 3309.167, 3280.833, 3407.500, 3400.000, 3436.667, 3278.333, 3286.667, 3275.000 |
| full prefix | 3500.833, 3494.167, 3406.667, 3424.167, 3402.500, 3360.833, 3291.667, 3294.167, 3300.833 |
| full wrapped ring | 3935.833, 3762.500, 3792.500, 3918.333, 3817.500, 3895.000, 3715.000, 3711.667, 3710.833 |
| nullable dense, separate | 3450.000, 3402.500, 3599.167, 3394.167, 3326.667, 3274.167, 3367.500, 3376.667, 3310.833 |
| nullable dense, tagged | 3639.167, 3600.833, 3692.500, 3601.667, 3600.000, 3605.833, 3550.000, 3595.833, 3575.833 |
| nullable sparse, separate | 244.167, 252.500, 250.000, 243.333, 253.333, 242.500, 242.500, 242.500, 242.500 |
| nullable sparse, tagged | 1575.000, 1580.000, 1554.167, 1565.000, 1548.333, 1545.833, 1556.667, 1570.833, 1573.333 |
| nullable clustered, separate | 252.500, 263.333, 272.500, 296.667, 290.833, 251.667, 258.333, 254.167, 252.500 |
| nullable clustered, tagged | 1545.833, 1595.833, 1584.167, 1878.333, 1622.500, 1553.333, 1552.500, 1655.833, 1646.667 |

The raw lists retain scheduling and frequency outliers rather than deleting them.
No confidence interval or host-independent performance threshold was preregistered,
so the timing is descriptive. The structural counts are exact for the declared C
types and do not depend on timing stability.

## Interpretation for the foundation question

- A full fixed payload needs no per-element validity state. Prefix and ring states
  add only fixed descriptor words while allowing unused capacity to remain
  uninitialized. Prefix stayed close to full; the wrapped ring was about 14%
  slower on this host. Their differing semantics prevent a universal choice from
  that common copy-plus-observation operation.
- Arbitrary nullable occupancy needs separate state unless the payload itself
  supplies a valid niche. For this ordinary 32-byte payload without a niche, the
  packed map has a material and exact backing-size advantage over an explicit C
  tag beside every element. It also has a clear scan win at low occupancy in this
  experiment; the dense timing difference is much smaller.
- The experiment gives no allocation-count advantage to either nullable rival:
  both use one allocation. It gives them the same semantically necessary payload
  initialization and copy work. The observed differences come from state layout
  and the traversal each layout directly supports.
- These native facts support keeping full, prefix, ring, and arbitrary nullable
  states distinct in the container foundation rather than forcing one universal
  per-element tagged representation. They do not choose source syntax, prove that
  separate validity is safe, or show that Whitefoot can check or lower it. W3 still
  requires compiler-enforced authority before any writer can use uninitialized
  backing, and W1 needs real Whitefoot programs and writer evidence beyond this
  native control.

## Reproducing

From the repository root:

```sh
cc -std=c11 -O3 -Wall -Wextra -Wpedantic -Werror \
  research/experiments/container-representation/foundation/layout.c \
  -o /tmp/whitefoot-layout-o3
/tmp/whitefoot-layout-o3 1200 9

cc -std=c11 -O2 -Wall -Wextra -Wpedantic -Werror \
  research/experiments/container-representation/foundation/layout.c \
  -o /tmp/whitefoot-layout-o2
/tmp/whitefoot-layout-o2 1200 9
```

`layout.c` defines `_POSIX_C_SOURCE=200809L` before system headers on POSIX and
uses `clock_gettime(CLOCK_MONOTONIC)`. On Windows it uses
`QueryPerformanceCounter`; the corresponding strict MSVC command is:

```bat
cl /std:c11 /O2 /W4 /WX research\experiments\container-representation\foundation\layout.c /Fe:layout.exe
layout.exe 1200 9
```

The same source was also cross-compiled during the portability audit with Zig
0.14.0 for `x86_64-linux-gnu` and `x86_64-windows-gnu`, using `-std=c11`, `-O2`,
`-Wall`, `-Wextra`, `-Wpedantic`, and `-Werror`. Those artifacts were not
executed, and an MSVC build was not available on the measurement host.

The program accepts optional positive `iterations` and `samples` arguments. Omitting
them selects 1,200 iterations and nine samples. It prints exact layout and work
counts before the raw timing rows. The maintained checks use the full defaults;
the arguments also support standalone measurement runs.

[rust-reserve]: https://github.com/rust-lang/rust/blob/29483883eed69d5fb4db01964cdf2af4d86e9cb2/library/alloc/src/vec/mod.rs#L1540-L1579
[rust-convert]: https://github.com/rust-lang/rust/blob/29483883eed69d5fb4db01964cdf2af4d86e9cb2/library/alloc/src/boxed/convert.rs#L259-L311
