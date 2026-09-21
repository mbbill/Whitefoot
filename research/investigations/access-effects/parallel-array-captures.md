# Parallel array capture experiment

This investigation addresses PR 70's `records` performance regression. It owns
the experiment and its retained grounds; it does not change source semantics,
the formal performance gate, or the deferred general storage-layout question.
Keep it as evidence for the capture decision, or retire it if that decision is
superseded and no current claim needs these measurements.

## Observed regression and diagnostics

The exact PR head `ea141997` passed canonical `make check` and hosted
correctness/IO. Its [compute comparison](https://github.com/mbbill/Whitefoot/actions/runs/35540986407)
failed `records`: baseline/candidate wall ratios were 1.065274 at W1,
0.935681 at W2, and 0.878450 at W4, with five adverse pairs at each parallel
width. All other kernels and the instrument's null/slow controls passed.

Retained Clang 18 images have equal 440-byte record workers, with the candidate
store using an eight-byte displacement for the new in-block header. Runtime
objects, input generation, grain rounding and leaf counts agree; no repeated
parent Box-slot load survives optimization. These observations narrow the
question but do not identify a microarchitectural cause.

The manual [forced-residue experiment](https://github.com/mbbill/Whitefoot/actions/runs/35542013762)
on EPYC 7763 reproduced the original failure but did not reproduce it under
either forced payload-residue shift, 16 to 24 or 48 to 56. An unchanged stencil
control was a W4 suspect, so the preregistered outcome is **inconclusive**.
It does not rule out allocation placement in the original images.

The subsequent [element-base experiment](https://github.com/mbbill/Whitefoot/actions/runs/35543004912)
kept ordinary allocation, the in-block header and effective element addresses.
It passed the element pointer through the existing task ABI instead of the
block pointer, and removed the corresponding header displacement inside the
worker. It changed no workload, runner, oracle or native runtime object.

| Comparison on EPYC 7763 | W1 | W2 | W4 |
|---|---:|---:|---:|
| Original baseline / candidate | 1.076414 | 0.931138 | 0.869396 |
| Baseline / element-base | 1.067342 | 0.990442 | 1.064312 |
| Candidate / element-base | 0.998313 | 1.060983 | 1.181825 |

Every candidate/element-base parallel pair improved, and baseline/element-base
had no adverse width. Nevertheless, unchanged stencil signaled at W2 in
reproduction and W4 in candidate/element-base. Its preregistered strict
controls therefore also give **inconclusive**, despite the useful lead.
The original candidate null was clean and the slow control detected all five
kernels. Neither diagnostic is being resampled or relabeled as conclusive.
Their exact protocols and artifact identities remain on the
[research branch](https://github.com/mbbill/Whitefoot/blob/8579a9b92cf3dfde02d29d126a4963803f2c4e00/research/investigations/access-effects/data-placement-provenance/README.md).

## General compiler candidate and result

The compiler experiment implemented a reversible, one-word capture encoding
for promoted `Box<Array<T>>` roots in synthesized range splits. It tests a
general compiler candidate, not a causal conclusion from the preceding
diagnostics.
For allocation base `B`, let `H` be the target-layout offset of its elements:

```text
parent:  P = B + H; capture P
chunk:   B = P - H; reconstruct borrowed local owner storage
body:    use the existing Box projection and element lowering
join:    parent retains the sole source owner and cleanup obligation
```

`P` has a distinct internal pointer type. It may name the one-past position of
an empty array, so it is not a source reference to an existing element. The
inverse operation recovers the original block pointer within the allocation.
Both operations use pointer GEPs; `H` is derived from the emitted aggregate's
field offset, including alignment padding, rather than a literal eight.
PAR-2 excludes whole-owner replacement, release and growth during an
actualized loop, and the structured join keeps the owner's storage alive.
Existing measure and whole-owner uses see reconstructed `B`.

Capture-all can also forward an already-consumed binding that the source body
does not use. The conversions do not dereference its allocation. LLVM 18's
[GEP contract](https://releases.llvm.org/18.1.8/docs/LangRef.html#getelementptr-instruction)
permits offsets inside the original allocation even after deallocation;
`inbounds` alone does not assert that the storage is alive. This relies on
retaining initialized pointer bits, not a null or uninitialized tombstone.
The consumed binding cannot regain source access or cleanup authority.

The padded-element control uses whole-element assignment and reads into copy
locals. Direct field suffixes on the legacy runtime Array path remain a
separate compiler gap recorded in `docs/todo.md`. Its original whole-root
measure read beside an element write is retained as a PAR-2 denial control;
the split control reads that length in the preheader, preserving every write
and the length/edge-value oracle. Existing byte-map controls compare every
output byte and inspect the source owner's release paths. Workers still read
the live empty Box's header.
Nested read-only reductions exercise both measure and element access through
the same reconstructed owner, and project it again at the inner split.

This changes neither the source Box ABI nor allocation, payload placement,
header ordering, host adapters or release. Inline aggregates and runtime
Slots/Ring retain their current captures. The policy applies to the whole
runtime Array type family, with no workload or source-name test. Its cost is
an extra internal pointer representation and reversible lowering operations;
its hypothesized benefit is making the stable element base explicit across
the outlined task boundary. On x86, the displacement was already folded into
the store: this is not a claim to remove a dynamic address instruction.

Before timing, the adoption conditions were fixed: the complete canonical
correctness gate had to pass, including native empty, aligned-element, mixed
measure/access, ownership and sequential/parallel controls; the exact
published revision had to pass hosted correctness, IO and compute checks; and
its `records` baseline/candidate wall ratio had to be at least 0.97 at
W1/W2/W4. The formal null, slowdown, workloads, oracles, five-pair schedule
and thresholds stayed unchanged.

The compiler candidate met that prospective criterion. PR head `cf6513bb`
passed local `make check` in 287.45 seconds and hosted
[correctness](https://github.com/mbbill/Whitefoot/actions/runs/35544999272),
[IO](https://github.com/mbbill/Whitefoot/actions/runs/35544999274), and
[formal compute](https://github.com/mbbill/Whitefoot/actions/runs/35545002043).
The compute run tested synthetic merge `649d70e0` against `c12d6dd1` on an
AMD EPYC 7763 host with Clang 18.1.3. Every formal kernel passed with no
comparison suspect. The `records` wall ratios and adverse-pair counts were:

| baseline / candidate | W1 | W2 | W4 |
|---|---:|---:|---:|
| Wall ratio | 1.087361 | 1.004834 | 1.060012 |
| Adverse pairs | 0/5 | 2/5 | 1/5 |

The identical-image control formally passed but retained a visible `records`
W4 suspect at 0.962815708 with four adverse pairs; it must not be dismissed as
noise. The deliberate slowdown was detected in all 15 kernel-width rows.
Retained optimized images show byte-identical 440-byte baseline and candidate
record chunks, both with a zero-displacement element store. The candidate
changes its synthesized capture frame while leaving the source Box ABI,
allocation, header and runtime objects unchanged.

This result selects the bounded compiler experiment provisionally. It does
not establish why the earlier candidate regressed, why this encoding changed
the hosted ratios when the optimized hot chunks are identical, or whether the
remaining null suspect is host, placement or measurement variability. The
forced-residue and element-base diagnostics above remain inconclusive.

A whole-Box payload-biased ABI would affect allocation, growth, release and
host adapters without being needed for this experiment. A use-directed
projected-place capture could avoid reconstructing `B` but needs a larger
use-analysis and mixed-use rewrite. The reversible synthetic encoding is the
smaller bounded candidate. Its successful formal comparison is evidence for
this capture family, not a resolution of all representation, placement or
measurement costs.
