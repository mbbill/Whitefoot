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

Run this slice with `make -C lib/containers check`.
