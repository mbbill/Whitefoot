Node: compiler/checker-facts

Decision: A [SET-1] commit whose target is a directly named binding records, on the checked target, whether that binding still holds a value at the commit, because [WIN-3] releases the old value of any owned place a write assigns over and [STOR-3] derives that release from the type, while [SET-1] and [LIV-1] make the premise a liveness judgment the checker has already made  -  the statement revived an entry-dead binding, or the right-hand side read the target's own value out  -  and neither answer is readable from the target's type or path, instead of leaving lowering to answer it from the shape alone, which released nothing for a named binding and leaked the cell `set cell = <new Box>;` displaced.

Decision: The record is taken after the right-hand side rather than where the target is resolved, because [SET-1] "rechecks its premises after its right-hand side under [LIV-1]" and a right-hand side that consumes the target leaves nothing for the write to displace, instead of reading the liveness at target resolution, which would derive a release of a value the same statement had already moved out.

Rejected:
- Deriving the release in lowering from the target's type alone: rejected because re-initializing a moved-out binding is an accepted program [OWN-1], so the derived release would free a value that is already gone.
