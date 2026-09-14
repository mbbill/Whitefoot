# Growable vector library results

This experiment bundles [`lib/containers/vector.wf`](../../../../lib/containers/vector.wf)
with `vector-library.wf`. It checks the reusable source implementation rather
than a second benchmark-only vector. The measured candidate is based on
`857b4d9e1ddcf0be809260a2a79b1d1b0ed4adf0`; `measurements.csv` contains the
retained samples. Measurements ran on 2026-09-14 on arm64 macOS 26.6.2 with
Apple Clang 21.0.0 and Rust 1.98.1.

## Executed contract

One round constructs an empty `GrowVector<u64>`, optionally reserves 32
elements, appends 32 values, inserts at index 16, removes that element, drains
the remaining values through a monomorphized behavior, folds a digest, and
releases the backing. The growth path omits reserve and therefore exercises
capacities 1, 2, 4, 8, 16, 32, and 64. The C controls perform the same ordered
operations, return the same digest, make the same number of allocation
requests, and reach the same peak live backing bytes.

`make check` executes 384 combinations of path, seed, round count, helper
contract, and implementation. The source-library gate separately runs scalar
and owning-`Box` operation chains in default, `--par`, and `--no-overlap`
modes. Its observer sees 11 requests in the owning chain, refuses each of the
nine nonzero requests in turn, stops at that request, and checks the exact
release order and one release per successful allocation. Successful backing
sizes are 0, 8, 16, 32, 0, 8, 16, 8, 32, 8, and 64 bytes.

## Timing

Each sample times 64 calls of 512 rounds. Fifteen samples in each of two
reversed-order cohorts alternate Whitefoot, a C control that reproduces the
source's take-back/replace/place-back exchange, and a C ceiling that swaps two
array cells directly. Values are median nanoseconds per complete round; ratios
are medians of paired samples. Host timing is descriptive and is not a gate.

| Helpers | Path | Whitefoot ns | matched C ns | direct C ns | WF / matched | matched / direct |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| ordinary | reserve first | 593.1 | 371.1 | 367.4 | 1.69x | 1.07x |
| ordinary | append growth | 875.0 | 589.8 | 572.4 | 1.48x | 1.04x |
| retained | reserve first | 1028.3 | 623.0 | 501.3 | 1.64x | 1.23x |
| retained | append growth | 1325.2 | 890.9 | 762.4 | 1.53x | 1.16x |

The allocation layout does not explain the gap: both controls use a 32-byte
descriptor and eight backing bytes per capacity unit, and every compared pair
has equal request counts and peak bytes. Optimized ordinary IR retains six
calls to `grow_vector_*`; forcing the source API and its internal helpers
retains thirteen. In the retained round, Whitefoot reserves a 160-byte result
frame and tests the tags of the fallible `Option`/`Result` values; the natural C
round has 40 bytes of local storage and returns its small failure outcomes in
the C ABI. That accounts for a material lowering/interface component. The
matched/direct C split bounds the source exchange shape at 16–23% with retained
helpers, while normal inlining removes nearly all of it. The remaining 48–69%
ordinary gap is not attributed more narrowly by this experiment.

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

The direct generic exchange remains rejected:

```whitefoot
invariant apart: left < right;
set (deref(values).storage[left], deref(values).storage[right]) =
  move deref(values).storage[right], move deref(values).storage[left];
```

LIV-2 reports `OverlappingCommitTargets` because OWN-7 recognizes distinct
subscripts only when both are unequal literals. `grow_vector_exchange` uses
take-back, replacement, and place-back instead. The pending
[`proof-directed-index-overlap`](../../../../design/amendments/proof-directed-index-overlap.md)
amendment proposes using an already completed finite integer proof for this
one overlap judgment; it is not implemented here.

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
