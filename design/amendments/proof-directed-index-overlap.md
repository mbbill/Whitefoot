Node: language/ownership

Decision: Overlap is judged conservatively over complete resolved paths, with distinct fields, integer indices whose unequal values are established by a completed deterministic proof, and proved half-open view extents establishing disjointness, because generic container algorithms must exchange two runtime-selected slots without introducing a temporary uninitialized hole, instead of restricting subscript separation to unequal written literals or adding a container-specific swap operation. This replaces the current overlap decision's unequal-literal-only index clause; whole-place accesses remain conservative and an undischarged inequality still overlaps.

Rejected:
- Keep all non-literal indices overlapping: rejected because insert, remove, heap adjustment, and partition algorithms then need extra run mutations or allocation to exchange two elements already proved to occupy different slots.
- Add a kernel swap operation for runs: rejected because the multi-target commit already defines the required atomic ownership transfer and a container-specific operation would duplicate its semantics.
