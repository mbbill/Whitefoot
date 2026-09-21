# Growable vector library results

This dated experiment bundled the reusable vector source with
`vector-library.wf`, rather than using a second benchmark-only vector.
Reproduce the v0.58 program and allocation-refusal contract at revision
`5fcf1ce2`; the sources here retain that historical language and are not a
current-spec benchmark. The maintained successor is
[`grow-vector.wf`](../../../../tests/programs/containers/grow-vector.wf),
executed with its caller and release observer by the ordinary program corpus.
`measurements.csv` contains the retained samples for the measured revision.
Measurements ran on 2026-09-14 on arm64 macOS 26.6.2 with
Apple Clang 21.0.0 and Rust 1.98.1.

## Executed contract

One round constructs an empty `GrowVector<u64>`, optionally reserves 32
elements, appends 32 values, inserts at index 16, removes that element, drains
the remaining values through a monomorphized behavior, folds a digest, and
releases the backing. The growth path omits reserve and therefore exercises
capacities 1, 2, 4, 8, 16, 32, and 64. The C controls perform the same ordered
operations, return the same digest, make the same number of allocation
requests, and reach the same peak live backing bytes.

`make check` executes 512 three-way configurations across the ordinary and
retained-helper binaries: 1,536 implementation executions spanning path, seed,
round count, helper contract, and implementation. The source-library gate separately runs scalar
and owning-`Box` operation chains in default, `--par`, and `--no-overlap`
modes. Its observer sees 11 requests in the owning chain and returns null at
each request in turn. The nine positive-byte refusals stop at that request;
the two zero-byte formations remain successful by the allocation contract and
the chain continues. Every case checks the exact release order and one release
per successful allocation, including that no release is attempted for a null
zero-byte backing. Successful backing
sizes are 0, 8, 16, 32, 0, 8, 16, 8, 32, 8, and 64 bytes.

## Timing

Each sample times 64 calls of 512 rounds. Fifteen samples in each of two
reversed-order cohorts alternate Whitefoot, a matched C control that directly
swaps two array cells as the source now does, and a legacy C control that
reproduces the former take-back/replace/place-back workaround. Values are median nanoseconds per complete round; ratios
are medians of paired samples. Host timing is descriptive and is not a gate.

| Helpers | Path | Whitefoot ns | matched C ns | legacy C ns | WF / matched | legacy / matched |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| ordinary | reserve first | 270.6 | 176.8 | 178.0 | 1.54x | 1.01x |
| ordinary | append growth | 402.9 | 279.2 | 280.3 | 1.44x | 1.00x |
| retained | reserve first | 354.3 | 221.1 | 269.0 | 1.60x | 1.21x |
| retained | append growth | 500.1 | 338.1 | 410.7 | 1.50x | 1.21x |

The allocation layout does not explain the gap: both controls use a 32-byte
descriptor and eight backing bytes per capacity unit, and every compared pair
has equal request counts and peak bytes. Optimized ordinary IR retains three
calls to `grow_vector_*`; forcing the source API and its internal helpers
retains thirteen. In the retained round, Whitefoot reserves a 160-byte result
frame and tests the tags of the fallible `Option`/`Result` values; the natural C
round has 40 bytes of local storage and returns its small failure outcomes in
the C ABI. That accounts for a material lowering/interface component. The
legacy/matched C split measures the removed workaround at about 21% with
retained helpers and no stable cost after normal inlining. With the source now
using the direct commit, the remaining 44–60% gap is not attributed more
narrowly by this experiment.

## Limits encountered

The v0.57 release rule rejected the growth handoff:

```whitefoot
let old = replace deref(values).storage = move built;
invariant old_empty: len_of(old) <= 0_u64;
dispose old;
```

PROV-6 judged the hypothetical `T: linear` element node even after the run was
empty. v0.58 admits this exact proved-empty direct-run release while retaining
the backing provider and effect. No unchecked release operation was added.

The direct generic exchange is admitted by the pending amendment implemented
on this branch:

```whitefoot
invariant apart: left < right;
set (deref(values).storage[left], deref(values).storage[right]) =
  move deref(values).storage[right], move deref(values).storage[left];
```

OWN-7 submits the two fixed strict-order goals at target formation; the helper's
branch discharges one before LIV-2 reads both affine elements out and commits
them. `grow_vector_exchange` therefore uses this direct form. The pending
[`proof-directed-index-overlap`](../../../../design/amendments/proof-directed-index-overlap.md)
records the choice beside the live design tree until the owner rules on it.

This ordinary read is admitted by the specification but not implemented by
the compiler:

```whitefoot
let id = deref(owners.storage[index]).id;
```

Checking stops as `Unsupported(CompositeValues)` without a rule. The witness
passes `&owners.storage[index]` to a helper and projects after dereference
there. The defect is tracked in `docs/todo.md`; the result is not presented as
a source rejection.

Finally, every temporary `&uniq` of a local owner must be formed in a written
local region under FORM-8 and OWN-10. Calls such as
`grow_vector_remove(values: &uniq values, ...)` therefore sit inside `region`
blocks. This is the standing explicit-region question already recorded in
`docs/todo.md`, not a container-specific rule.
