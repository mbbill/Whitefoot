Decision: A pre-loop fact is removed at an iteration head only by a continuing kill, a kill whose structural normal successor can reach a later head of the same loop without leaving its body, so a return, propagated error, or loop-leaving break inside the body removes no pre-loop fact at that head, because the scope-exit rule discarded every pre-loop fact in any loop containing a return and was the dominant measured cause of the deflate acceptance divergence, instead of treating every scope-leaving edge as a kill at the head.

Rejected:
- Every scope-leaving edge kills each binding's facts at the loop head: rejected because one return inside a loop body discarded every pre-loop fact, and the identical program discharged with the return outside the loop and rejected with it inside.
