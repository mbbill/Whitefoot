# Whitefoot container library

This directory contains reusable Whitefoot source. A program bundles the
container source before its own source on the `whitefootc` command line; the
canonical repository gate compiles and executes every bundled test program in
the default, `--par`, and `--no-overlap` lowering modes.

## Growable vector

[`vector.wf`](vector.wf) defines `GrowVector<'s, T>` over an ordinary
store-backed `Vector<'s, T>`. Its operations are generic over every linearity
class accepted by the `linear` bound:

- `grow_vector_new` creates an empty vector.
- `grow_vector_reserve` preserves the window and raises capacity to at least a
  requested total. `Err(unit)` leaves the existing owner installed.
- `grow_vector_append` doubles full nonempty backing and returns `Err(value)`
  without changing the vector if allocation or target representation refuses.
- `grow_vector_insert` inserts before a proved index and preserves order.
- `grow_vector_remove` removes a proved index and preserves order.
- `grow_vector_drain` moves values from front to back into a monomorphized
  `VectorDrain` behavior supplied by the caller.

Every mutating public operation takes `&uniq`; growth replaces only the backing
inside the borrowed owner. Successful `Result` values report the new length or
capacity, and contracts publish the corresponding exit state. No mutation
round-trips the complete container through a value parameter and result.
After growth transfers the old window, a local zero-length proof lets PROV-6
release that run's backing without requiring providers for its now-absent
generic elements; the backing provider and release effect remain explicit.

Run this slice with `make -C lib/containers check`.
The maintained allocation observer returns null at every allocation request
and checks the exact release ledger in all three lowering modes. Positive-byte
refusals return the owner or stop the operation chain; zero-byte formations
remain successful and never attempt to release a null backing. The matched
native comparison and current limits are recorded in
[`../../research/experiments/container-representation/vector-library/RESULTS.md`](../../research/experiments/container-representation/vector-library/RESULTS.md).
