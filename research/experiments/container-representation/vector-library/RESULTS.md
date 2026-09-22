# Growable vector library costs

This experiment bundles the current reusable
[`GrowVector`](../../../../lib/containers/grow-vector.wf), not a second
benchmark-only implementation. The selection criteria precede measurement in
[X1-LIBRARY.md](../../../investigations/containers-and-resources/X1-LIBRARY.md#vector-consumption-trial).
The source uses kernel v0.60's global heap and total allocation. Measurements
are descriptive evidence, outside correctness CI, not a native-parity gate.

## Contract and controls

One work round fills an empty vector, inserts and removes a middle marker,
swap-removes the first element when present, consumes the suffix after the
midpoint, then drains the retained prefix. Both consuming operations call the
member in original element order. Every element contributes to an
order-sensitive checksum. The independent C oracle computes that logical
sequence without constructing or mutating a vector.

The three paths are reserve-before-fill, growth by append, and reuse of one
reserved allocation across rounds. All have a final empty-owner release.
Lengths 16, 256 and 4096 are timed with `16384 / length` rounds per sample.
Elements are an 8-byte scalar or a 256-byte `nocopy` record of 32 words; each
record word contributes to the checksum. These are operation trials, not a
claim about real application frequencies or a measurement of Box-payload
allocation costs.

- **Whitefoot:** the actual library's reverse-suffix, take-back composition.
- **Reverse C:** the same algorithm, growth policy, element ownership transfer
  and callback sequence. Its difference from WF measures compiler/lowering
  costs under this source shape, including ordinary native ABI differences.
- **Direct C:** the same contract with a direct ordered suffix consumer. The
  callback cannot observe the vector, so the control can traverse the suffix
  and shorten the length once. Its difference from reverse C measures the
  composition's cost, separately from WF lowering.

Each implementation has one pointer owner and a header-first allocation:
16 bytes for length/capacity, then `capacity * sizeof(element)` bytes.
Growth allocates, copies the live run, and releases the previous backing;
there is no realloc-policy difference. Allocation counts, requested bytes,
peak live bytes and final zero live bytes must match before any timing is
accepted. The common accounting wrapper adds the same bookkeeping to all
three implementations; reported sizes exclude its private header.

The ordinary mode permits normal Clang O2 inlining. The retained mode marks
library and workload helpers and the element make/accept functions noinline
on both sides. WF retains 26 selected definitions. Compiler-owned primitive
operations remain eligible for inlining, just as C primitive operations do.
Generated optimized WF and C IR remain in `.build/` for inspection.

## Correctness and timing method

`make check` in this directory checks 540 configurations per helper mode:
10 lengths (including 0, 1, 8192), three round counts (including 0), three
seeds (including u64 max), two element sizes and three allocation paths.
All three implementations run each configuration: 3,240 executions across
the two modes. This is an experiment check, not a daily gate dependency.

The formal corpus separately bundles the library and
[`grow-vector-program.wf`](../../../../tests/programs/containers/grow-vector-program.wf).
It executes sequential and parallel lowering, normally and with an allocator
observer that records every identity and detects stale/double release. Copy,
affine Box and nodrop owning chains check retained prefix, callback order,
empty/singleton cases, same-index swap-remove, reuse and 25 exact-once releases.
These regressions, rather than the research harness, run in canonical
`make check`.

Measurements run on Apple M1 Pro (8 logical CPUs), arm64 macOS 26.6.2,
Apple Clang 21.0.0. Each of two cohorts has 11 samples and reverses the
implementation order; samples rotate the first implementation. Each timing
still checks the oracle and complete release. Raw times cover the whole
operation chain, not just drain. The 32-word checksum is material work in
large-record samples, so these timings do not isolate pure memory bandwidth.
Reported timings are medians in nanoseconds per round; ratios are ratios of
those medians. No cross-machine or application-wide speed claim follows.

## Lowering attribution

The initial dataset is
[`measurements-x1-before-address.csv`](measurements-x1-before-address.csv).
It uses compiler `b3d323a7`, the current WF workload and C controls. Optimized
IR retains three unnecessary capacity-based wrap decisions in consuming
truncation: two per reverse pair and one per taken element. Slots has origin
zero and OP-4/OP-10 already prove each selected slot lies inside capacity;
only Ring needs head-relative wrap. Removing this Slots arithmetic changes
no source contract, backing layout, allocation, callback or element transfer.

For 4096 elements the initial WF/reverse-C ratios were:

| Helpers | Element bytes | Reserved | Growth | Reuse |
| --- | ---: | ---: | ---: | ---: |
| ordinary | 8 | 1.564 | 1.528 | 1.552 |
| retained | 8 | 1.182 | 1.176 | 1.190 |
| ordinary | 256 | 1.213 | 1.169 | 1.206 |
| retained | 256 | 1.063 | 1.057 | 1.058 |

The final samples are
[`measurements-x1.csv`](measurements-x1.csv), measured on 2026-09-21 PDT with
compiler `4d7a4c629` (including the address repair in `ad62a039f`). The workload,
C sources, inputs and harness are unchanged between the two datasets. The
address repair has a backend regression covering subscripts and back
placement/take for inline and boxed Slots; Ring retains wrap in both placements.
The final 4096-element WF/reverse-C ratios are:

| Helpers | Element bytes | Reserved | Growth | Reuse |
| --- | ---: | ---: | ---: | ---: |
| ordinary | 8 | 1.170 | 1.173 | 1.174 |
| retained | 8 | 0.988 | 0.989 | 0.988 |
| ordinary | 256 | 1.192 | 1.147 | 1.186 |
| retained | 256 | 1.043 | 1.036 | 1.041 |

For the reused 4096-element chain, absolute times separate the remaining
lowering gap from the cost of the source composition:

| Helpers | Element bytes | WF ns/round | Reverse C ns/round | Direct C ns/round | WF / direct C |
| --- | ---: | ---: | ---: | ---: | ---: |
| ordinary | 8 | 13,500 | 11,500 | 6,250 | 2.160 |
| retained | 8 | 20,500 | 20,750 | 19,500 | 1.051 |
| ordinary | 256 | 220,875 | 186,250 | 161,000 | 1.372 |
| retained | 256 | 226,500 | 217,500 | 191,250 | 1.184 |

The isolated code change removes all three Slots wrap decisions from retained
truncation. For the reused scalar chain, WF time falls 27.5 percent with
ordinary inlining and 18.0 percent with retained helpers; the corresponding
C controls change by 4.2 and 1.2 percent. This supports a real scalar benefit,
not attributing every timing difference to the patch. The large-record
change is much smaller. These short samples have no calibrated confidence
interval; the retained scalar result is approximate parity, not a speed win.

Optimized IR shows the remaining transfers directly:

| Retained 256-byte truncate | Per reversed pair | Per consumed record |
| --- | --- | --- |
| WF | 2 memcpy + 1 memmove, each 256 bytes | 1 memcpy into the callback argument |
| Reverse C | 3 memcpy, each 256 bytes | 1 memcpy into the callback argument |
| Direct C | none | 1 memcpy into the callback argument |

All three retain a direct callback call. WF's alias-permitting `swap` keeps
memmove and conservative element alignment where C keeps memcpy/alignment
facts. The transfer counts explain why removing wrap arithmetic cannot erase
the reversal cost; they do not isolate the timing contribution of alignment
or alias facts. In the retained large-record reuse case, reverse C is 26,250
ns slower than direct C and WF is a further 9,000 ns slower than reverse C.
Both are costs of the full chain. In ordinary mode inlining changes the
surrounding loops too, so subtracting retained and ordinary timings is not a
measurement of call overhead alone. The optimizer retains 7 ordinary and 42
retained WF library call sites; the retained C IR keeps 12 append/truncate
call sites.

The shortest scalar reuse case also remains significant: 49.8 ns WF versus
30.3 ns reverse C and 26.4 ns direct C at length 16 with ordinary inlining.
The 4096-element table must not stand for every length or element size; all
36 comparison cells are present in the raw samples.

The proposed library form is therefore a correct O(n), no-allocation baseline
with complete ownership cleanup, not the final minimum-transfer ordered
consumer. The measured gap reopens the blanket zero-extra-cost claim in
kernel minimality. Keep the operation inventory unchanged in this trial;
record direct ordered consumption and the residual ordinary-inlining cost
in `docs/todo.md`. A follow-up must compare representations or an operation
under the same callback/ownership contract before selecting language support.
Slab/Deque construction need not depend on a claim of Vector native parity.

## Source and proof boundaries

[Vector source obligations](../../../investigations/containers-and-resources/X1-LIBRARY.md#vector-source-obligations)
records the exact rejected fragments, rules and ordinary forms:

- ENT-5 removes a loop-header hypothesis at the loop exit; an INV-1 local
  theorem immediately before the break exports the required conclusion.
- FN-8's affine Signed Goal leaves are order comparisons. An unsigned
  `len <= 0` requirement expresses empty without a runtime check when the
  available proof is an invariant; equality works on the ordinary L0 route.
- The compiler wrongly rejected moving the sole linear field of a wrapper.
  The PROV-6 repair judges the unselected residual and includes regressions
  that still reject an abandoned generic or fieldless nodrop member.
- Destructuring a Box-containing wrapper currently loses the content measure
  fact. The separate placement defect is recorded in `docs/todo.md`; direct
  field consumption serves this library without an artificial failure branch.

No specification rule changes in this trial.

## Reproduction and earlier evidence

Build the gate-profile compiler, then run from the repository root, one
shared verification owner at a time:

```sh
make -C compiler build
perl .github/run-check.pl vector-costs make -C research/experiments/container-representation/vector-library check
perl .github/run-check.pl vector-measure make -C research/experiments/container-representation/vector-library measure
```

`measure` writes `.build/measurements.csv`. `WHITEFOOTC=/path/to/whitefootc`
and a fresh `BUILD=/path/to/scratch` select another compiler without changing
the workload; use the `b3d323a7` compiler to reproduce the before-address case.
The checked-in dated samples retain the original run; a new run does not
silently replace them.

The older [`measurements.csv`](measurements.csv) belongs to revision
`5fcf1ce2`, kernel v0.58, measured on 2026-09-14. Its provider/refusal contract,
32-byte descriptor and repeated front removal differ from this experiment.
Reproduce its code at that revision. Its four WF/matched-C ratios were
1.54/1.44 (ordinary reserve/growth) and 1.60/1.50 (retained); they are historical
evidence and are not current performance or refusal coverage.
