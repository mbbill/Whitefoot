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

## Native layout comparison

Measured 2026-09-07 from repository revision
`f2a298666fcf3ac9ea454087623773fd06f4bc6f`. The measured `layout.c` has SHA-256
`23e69613d6705f5805a7a82978c3ab797bfe6dda7e235e58dfd3a9afa2d3282f`.

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
storage. Both copy exactly those same payload bytes. The validity initialization
counts state the bytes that must receive known validity state in these concrete
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

The timed functions are out of line. Each invocation copies and checksums its
output; every repetition contributes to a volatile published witness. The final
witness was `34359739618` at both optimization levels. This keeps the common work
observable without adding work to only one representation.

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

| State | Live payloads | Payload bytes initialized | State bytes initialized | Payload bytes copied |
| --- | ---: | ---: | ---: | ---: |
| full | 4,096 | 131,072 | 0 | 131,072 |
| prefix, full | 4,096 | 131,072 | 16 | 131,072 |
| prefix, quarter | 1,024 | 32,768 | 16 | 32,768 |
| ring, full and wrapped | 4,096 | 131,072 | 24 | 131,072 |
| ring, wrapped 17 | 17 | 544 | 24 | 544 |
| nullable dense | 3,584 | 114,688 | 512 separate / 4,096 tagged | 114,688 |
| nullable sparse | 256 | 8,192 | 512 separate / 4,096 tagged | 8,192 |
| nullable clustered | 256 | 8,192 | 512 separate / 4,096 tagged | 8,192 |

The fixed backing capacity is the same for full, prefix, and ring. Their initialized
payload count differs because their semantics differ. The table therefore does not
turn a prefix's unused capacity into initialization work or compare a 17-element
ring with a full array as if they performed one operation.

## Descriptive timing

Host: macOS 26.6.2 (`Darwin 25.6.0`, arm64). The sandbox did not expose a hardware
model string. Compiler: Apple clang 21.0.0 (`clang-2100.1.1.101`), target
`arm64-apple-darwin25.6.0`. Each entry is the median of nine process-local samples;
each sample performs 1,200 copies after untimed correctness checks and warm-up.
The order rotates between the compared implementations. Nanoseconds are per one
complete copy/compaction.

| Work | `-O3` median ns | Ratio | `-O2` median ns | Ratio |
| --- | ---: | ---: | ---: | ---: |
| full | 26,941.667 | 1.000 full | 27,157.500 | 1.000 full |
| full prefix | 27,184.167 | 1.009 / full | 27,319.167 | 1.006 / full |
| full wrapped ring | 27,080.000 | 1.005 / full | 27,319.167 | 1.006 / full |
| nullable dense, separate | 23,985.000 | 1.000 separate | 24,409.167 | 1.000 separate |
| nullable dense, tagged | 24,054.167 | 1.003 / separate | 24,250.000 | 0.994 / separate |
| nullable sparse, separate | 1,656.667 | 1.000 separate | 1,766.667 | 1.000 separate |
| nullable sparse, tagged | 1,684.167 | 1.017 / separate | 1,803.333 | 1.021 / separate |
| nullable clustered, separate | 1,656.667 | 1.000 separate | 1,722.500 | 1.000 separate |
| nullable clustered, tagged | 2,949.167 | **1.780 / separate** | 3,097.500 | **1.798 / separate** |

The full, prefix, and ring sample ranges overlap, as do both nullable encodings for
dense and sparse data. Those rows do not discriminate on this run. The clustered
case does: every tagged sample is above every separate sample at both optimization
levels, with a 1.78x and 1.80x median ratio. The sparse and clustered states have
the same population and copied bytes, while their tagged timings differ; occupancy
distribution therefore matters to this traversal. This one host and three synthetic
patterns do not establish a general throughput ranking or a workload-frequency
claim.

Raw `-O3` samples, in collection order, ns per copy:

| Work | Samples |
| --- | --- |
| full | 42907.500, 26941.667, 26838.333, 26858.333, 27038.333, 26990.833, 26978.333, 26830.833, 26872.500 |
| full prefix | 29011.667, 27291.667, 26840.833, 27168.333, 27332.500, 27203.333, 26925.000, 26874.167, 27184.167 |
| full wrapped ring | 27074.167, 27390.000, 26983.333, 27495.833, 27080.000, 27305.833, 27464.167, 27021.667, 27062.500 |
| nullable dense, separate | 25712.500, 28033.333, 24069.167, 24277.500, 23935.833, 23798.333, 23985.000, 23718.333, 23693.333 |
| nullable dense, tagged | 24410.000, 25417.500, 23746.667, 24395.000, 24054.167, 25860.000, 23714.167, 23834.167, 23775.833 |
| nullable sparse, separate | 1666.667, 1750.833, 1654.167, 1657.500, 1655.000, 1654.167, 1656.667, 1725.833, 1655.833 |
| nullable sparse, tagged | 1686.667, 1683.333, 1704.167, 1684.167, 1685.833, 1683.333, 1684.167, 1725.833, 1684.167 |
| nullable clustered, separate | 1650.000, 1712.500, 1669.167, 1656.667, 1700.833, 1652.500, 1650.000, 1658.333, 1655.000 |
| nullable clustered, tagged | 2990.833, 2916.667, 2920.833, 2940.000, 2949.167, 2959.167, 3022.500, 3022.500, 2912.500 |

Raw `-O2` samples, in collection order, ns per copy:

| Work | Samples |
| --- | --- |
| full | 41305.833, 26859.167, 27114.167, 27613.333, 27052.500, 27157.500, 26977.500, 27452.500, 27892.500 |
| full prefix | 28510.833, 27180.000, 27319.167, 27544.167, 26957.500, 27051.667, 26909.167, 29857.500, 27600.000 |
| full wrapped ring | 27475.833, 27090.833, 27143.333, 27650.000, 27165.833, 27319.167, 26976.667, 28925.833, 28255.000 |
| nullable dense, separate | 24409.167, 24340.833, 24299.167, 24283.333, 24958.333, 24066.667, 24596.667, 24652.500, 24728.333 |
| nullable dense, tagged | 23934.167, 24100.833, 24118.333, 24250.000, 24246.667, 24697.500, 24982.500, 24515.833, 51040.833 |
| nullable sparse, separate | 1754.167, 1783.333, 1754.167, 2370.833, 1729.167, 1766.667, 1740.000, 1785.000, 1773.333 |
| nullable sparse, tagged | 1794.167, 1890.833, 1810.000, 1779.167, 1741.667, 1846.667, 1803.333, 1795.000, 1872.500 |
| nullable clustered, separate | 1823.333, 1750.000, 1722.500, 1670.833, 1721.667, 1711.667, 1775.833, 1754.167, 1689.167 |
| nullable clustered, tagged | 3101.667, 3198.333, 3097.500, 2954.167, 3128.333, 3010.833, 3088.333, 3107.500, 3040.833 |

The raw lists retain scheduling and frequency outliers rather than deleting them.
No confidence interval or host-independent performance threshold was preregistered,
so the timing is descriptive. The structural counts are exact for the declared C
types and do not depend on timing stability.

## Interpretation for the foundation question

- A full fixed payload needs no per-element validity state. Prefix and ring states
  add only fixed descriptor words while allowing unused capacity to remain
  uninitialized. Their common full-copy timing is indistinguishable here.
- Arbitrary nullable occupancy needs separate state unless the payload itself
  supplies a valid niche. For this ordinary 32-byte payload without a niche, the
  packed map has a material and exact backing-size advantage over an explicit C
  tag beside every element. It also has one distribution-specific scan win in this
  experiment. Dense and periodic sparse scans are timing ties.
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

The program accepts optional positive `iterations` and `samples` arguments. Omitting
them selects 1,200 iterations and nine samples. It prints exact layout and work
counts before the raw timing rows, so a correctness-only gate may use smaller
positive counts without changing the structural witnesses.
