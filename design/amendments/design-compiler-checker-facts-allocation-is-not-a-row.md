Node: compiler/checker-facts

Decision: [EFF-2]'s "exactly the row the body exhibits" is judged over the `reads` and `writes` categories alone, and the allocation fact is excluded from that comparison, because [EFF-1] gives the row exactly two categories and [STOR-8] gives allocation no entry in it, so a writer has no spelling that could declare allocation and comparing it would reject every `pure` function that calls a construction row against a row the language gave it no way to write, instead of comparing the whole effect record, which made a correct `fn make(n: own u64) -> made: own Box<Slots<i32>> pure` fail against a row rendered identically to its own.

Decision: The boundary's allocation fact recorded in the checked program is the one the body exhibits rather than the one the declaration carried, because [EFF-3] reads that fact to except an allocating call from deduplication and reordering, and a body that calls a construction row does allocate whatever its written row says, while a body-less [PRE-1] record exhibits exactly its declared set and so still reports its own allocation, instead of recording the declared fact, which is false for every source function that constructs.

Rejected:
- Deriving each source function's declared allocation fact from its callees before its body is checked, so callers inherit it: rejected because the fact would then need a fixpoint over the call graph before any body is judged, and nothing in this version's [EFF-3] consumers reads a transitively allocating source caller; the call that allocates is itself marked where it occurs.
