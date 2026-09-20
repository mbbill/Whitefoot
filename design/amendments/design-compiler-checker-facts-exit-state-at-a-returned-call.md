Node: compiler/checker-facts

Decision: A `return` whose selected expression is a call applies that call's whole boundary  -  its projected write kills and its published exit relation  -  before the postcondition is queried at that return, because [FN-9] reads a bare measure of a written reference parameter "over that parameter's resolved referent immediately before each selected return, after the return's ordinary effects and kills" and [CALL-6] establishes an unrouted relation over a written parameter's exit state "on the call's normal continuation, after the call's ordinary transfer, consumes, borrow commits, target commit and kills", instead of judging the clause first and applying only the expression's kills afterwards, which read the referent's entry state at the exit and made `ensures deref(p).len == deref(entry(p)).len` hold over a callee that had just moved the boundary.

Decision: The returned value's own consume stays after the query, because that consume is the return transfer itself and [FN-9] places the query immediately before it, instead of moving every kill the returned expression carries ahead of the query, which would end the facts a clause over the returned place needs.

Rejected:
- Reading the exit measure from the parameter's entry datum when the return's own call has not yet been judged: rejected because [MSR-3] states that "Entry and exit measures are distinct terms even when both project from the same formal and actual", so a reading that makes them one term publishes a relation the callee does not establish.
