# E0.1 research notes

These notes separate facts from design inferences.  External results motivate
controls; they do not substitute for measurements on xlc.

## Ownership and initialized-prefix storage

- Rust's `Copy` contract requires every component to be copyable and excludes
  types with `Drop`.  More importantly for E0.1, assignment/use of a Copy value
  is an implicit value copy.  **Inference:** using structural Copy merely to
  admit record storage would also permit invisible large aggregate copies, so
  xlang's Flat-storage predicate must stay distinct from implicit Copy.
  <https://doc.rust-lang.org/reference/special-types-and-traits.html#copy>
- Rust documents `Vec<T>` as a pointer/capacity/length owner with exactly the
  first `len` elements initialized.  Spare capacity is logically uninitialized;
  growth can reallocate.  `push_within_capacity` is specifically the no-realloc
  append operation.  **Inference:** current xlang `buffer<T>`, whose full fixed
  length is initialized and dropped, cannot implement an affine Vec without an
  additional initialized-prefix/raw-storage abstraction or per-slot overhead.
  <https://doc.rust-lang.org/std/vec/struct.Vec.html#guarantees>
- Rust `Layout` limits allocation size after alignment rounding to
  `isize::MAX`.  This is not adopted by analogy alone: it matches LLVM's pointer
  index arithmetic constraints below.
  <https://doc.rust-lang.org/std/alloc/struct.Layout.html>

## LLVM code-shape and allocation constraints

- LLVM `getelementptr` converts offsets to the target pointer index type.
  `inbounds` includes signed-no-wrap requirements; violating them produces
  poison.  **Inference:** OP-9's current `u64 count * element_size` overflow
  check is insufficient once variable-size records/over-aligned types enter;
  object size, alignment rounding, and target index limits must also be checked.
  <https://llvm.org/docs/LangRef.html#getelementptr-instruction>
- LLVM's frontend guidance explicitly recommends avoiding aggregate values and
  loading/storing individual fields.  This directly motivates the field-only
  GEP/scalar-load gate; relying on later SROA to remove whole-record transfers is
  not accepted.
  <https://llvm.org/docs/Frontend/PerformanceTips.html#avoid-creating-values-of-aggregate-type>
- LLVM can vectorize some interleaved/strided access groups, but legality and a
  target cost model decide whether it does.  AoS therefore cannot be assumed to
  recover SoA's unit-stride behavior.
  <https://llvm.org/docs/Vectorizers.html>
  <https://llvm.org/docs/VectorizationPlan.html>

## Data-layout evidence

- SoAx reports large SoA wins for its particle workloads across CPUs,
  accelerators, and GPUs.  This supports protecting columnar consumers, not a
  universal SoA rule.  <https://arxiv.org/abs/1710.03462>
- A separately studied Lennard-Jones implementation reports an AoS-with-padding
  win after target-specific vectorization.  This is a useful hostile counterexample
  to “SoA always wins.”  <https://arxiv.org/abs/1806.05713>
- SoCal (2026) studies serialized recursive data, including compiler-like tree
  workloads, and reports a geometric-mean gain from factored multi-buffer
  layouts.  This makes xlc's AST SoA a particularly important consumer-side
  baseline rather than bootstrap debt to erase.  <https://arxiv.org/abs/2605.01140>

The combined evidence supports only a workload-dependent conclusion: xlang
should be able to express both layouts without overhead, while its default
writer guidance must be decided from access patterns and measured outcomes.

