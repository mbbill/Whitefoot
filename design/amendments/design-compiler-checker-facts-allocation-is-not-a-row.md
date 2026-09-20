Node: compiler/checker-facts

Decision: [EFF-2]'s "exactly the row the body exhibits" is judged over the `reads` and `writes` categories alone, and the allocation fact is excluded from that comparison, because [EFF-1] gives the row exactly two categories and [STOR-8] gives allocation no entry in it, so a writer has no spelling that could declare allocation and comparing it would reject every `pure` function that calls a construction row against a row the language gave it no way to write, instead of comparing the whole effect record, which made a correct `fn make(n: own u64) -> made: own Box<Slots<i32>> pure` fail against a row rendered identically to its own.

Decision: The boundary's allocation fact recorded in the checked program is the one the body exhibits, including the union of the allocation facts of the calls it contains, rather than the source row, because [EFF-3] reads that fact to except every transitively allocating call from deduplication and reordering, while a body-less [PRE-1] record exhibits exactly its compiler-owned declaration fact; this propagation follows the ordinary checked call dependencies and adds neither a row category nor a separate source-visible fixpoint.

Rejected:
- Marking only the immediate construction call and withholding allocation from its source callers: rejected because [EFF-3]'s licence is asked at each call boundary, and duplicating or reordering a wrapper that transitively takes from the finite heap changes the program just as duplicating the construction call does.
- Adding allocation to the source effect row in order to propagate it: rejected because [EFF-1] closes the row over `reads` and `writes`, and the checked call walk already carries the independent metadata through the finite dependency graph.
