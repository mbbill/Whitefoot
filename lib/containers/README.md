# Whitefoot container library

This directory contains reusable Whitefoot source. A program bundles the
container source before its own source on the `whitefootc` command line; the
canonical repository gate compiles and executes every bundled test program in
the default, `--par`, and `--no-overlap` lowering modes.

## Growable vector

[`vector.wf`](vector.wf) defines `GrowVector<T>` over a `Box<Slots<T>>`: a
runtime-capacity window exists only as the content of a cell [TYPE-9], and the
cell is the one heap object the vector owns [STOR-1]. Its operations are
generic over every linearity class accepted by the `linear` bound:

- `grow_vector_new` creates an empty vector over a zero-capacity backing.
- `grow_vector_reserve` preserves the window and raises capacity to at least a
  requested total, using `grow`, which remakes the cell's content whole and may
  reallocate in place [OP-10].
- `grow_vector_append` doubles full nonempty backing and takes one slot when the
  backing is empty.
- `grow_vector_insert` inserts before a proved index and preserves order.
- `grow_vector_remove` removes a proved index and preserves order.
- `grow_vector_drain` moves values from front to back into a monomorphized
  `VectorDrain` behavior supplied by the caller.

Every mutating public operation takes an ordinary `&GrowVector<T>` reference and
declares `writes(values.storage)`; there is no permission marker, and whether a
callee may write through a reference is its effect row alone [REF-1, EFF-1].
Contracts publish the exit state of the measures they change.

Three things this library carried in v0.59 are gone with the rules that
supplied them. Allocation is total in the source [STOR-8], so no operation
returns a `Result` whose error arm reports a refusal and no caller installs a
fallback owner; the exhausted heap ends the program from the trusted base,
outside the language [SCOPE-3]. The store surface is gone with regions, so
there is no `Heap<'s>` parameter and no `heap_vector` take. The manual
exchange-and-shift loops that `insert` and `remove` were written as retired
with [OP-10]'s `insert_at` and `remove_at`, which are the operations that move
the window boundary.

Run this slice with `make -C lib/containers check`.
The maintained allocation observer checks the release ledger in every lowering
mode: every allocation the program makes is released exactly once, none twice,
and none is left behind. Its v0.59 refusal sweep, which returned null at each
allocation request in turn and read the source-visible fallback, retired with
[STOR-8]: allocation never returns a failure to the source, so there is no
refusal for a program to observe. The matched
native comparison and current limits are recorded in
[`../../research/experiments/container-representation/vector-library/RESULTS.md`](../../research/experiments/container-representation/vector-library/RESULTS.md).
