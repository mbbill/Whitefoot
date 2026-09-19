Node: compiler/checker-facts

Decision: The base case of the allocation fact [EFF-3] is the declaration identity of the nine [OP-13] construction functions and [OP-10]'s `grow`, recognized by name where the [PRE-1] record's signature is built, and every other boundary's fact is the union of the facts of the calls its body exhibits, because [STOR-8] gives allocation no effect entry so no row can declare it, [EFF-3] still excepts an allocating call from deduplication and reordering, and those ten records are the complete set of boundaries that take from the heap by definition rather than by what they call, instead of reviving an `allocates` effect spelling the row grammar no longer has or rederiving the fact at lowering, which the pending compiler/checker-facts decision already refuses.

Decision: A function-kind binding whose supplied actual allocates where the instantiated formal does not is an [FN-4] mismatch, because a caller frames the formal interface's licence, so an actual that allocates under a non-allocating formal would let a caller duplicate a take from the finite heap [EFF-3, STOR-8], instead of leaving the allocation fact out of the [FN-4] comparison because it is not part of the declared row.

Rejected:
- Deriving the base case from the callee's body rather than from the record's identity: rejected because a [PRE-1] record has no source body, so there is nothing to derive it from.
