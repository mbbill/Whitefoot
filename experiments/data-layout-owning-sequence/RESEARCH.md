# E0.1 research notes

These notes separate facts from design inferences.  External results motivate
controls; they do not substitute for measurements on wfc.

## Ownership and initialized-prefix storage

- Rust's `Copy` contract requires every component to be copyable and excludes
  types with `Drop`.  More importantly for E0.1, assignment/use of a Copy value
  is an implicit value copy.  **Inference:** using either automatic or declared
  Copy to admit record storage also permits implicit aggregate copies at every
  ordinary use.  Whether that total bundle is preferable to a separate affine
  storage predicate remains an experimental question.
  <https://doc.rust-lang.org/reference/special-types-and-traits.html#copy>
- Swift's noncopyable-value proposal uses a file descriptor represented only by
  an integer as a motivating example: structural scalar fields do not imply that
  duplicating the nominal value preserves its intended protocol.  **Inference:**
  automatic structural Copy cannot infer contraction intent from representation;
  this is independent of memory/drop safety.
  <https://github.com/swiftlang/swift-evolution/blob/main/proposals/0390-noncopyable-structs-and-enums.md>
- Move exposes `copy` and `store` as distinct abilities.  A value can therefore
  be eligible for persistent storage without being duplicable.  **Inference:**
  separating fixed-storage eligibility from Copy is a coherent non-Rust design
  precedent, but it does not establish that Whitefoot should pay the initializer and
  checker cost of that separation.
  <https://move-language.github.io/move/abilities.html>
- Rust documents `Vec<T>` as a pointer/capacity/length owner with exactly the
  first `len` elements initialized.  Spare capacity is logically uninitialized;
  growth can reallocate.  `push_within_capacity` is specifically the no-realloc
  append operation.  **Inference:** current Whitefoot `buffer<T>`, whose full fixed
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
  layouts.  This makes wfc's AST SoA a particularly important consumer-side
  baseline rather than bootstrap debt to erase.  <https://arxiv.org/abs/2605.01140>

The combined evidence supports only a workload-dependent conclusion: Whitefoot
should be able to express both layouts without overhead, while its default
writer guidance must be decided from access patterns and measured outcomes.
