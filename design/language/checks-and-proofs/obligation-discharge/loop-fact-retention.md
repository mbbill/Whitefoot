Decision: A pre-loop fact survives a loop head unless a kill inside the body can reach a later head without leaving the body, so a return, propagated error, or break kills nothing at the head, because the scope-exit rule discarded every pre-loop fact in any loop containing a return and was the main cause of the DEFLATE decoder proving 5 of its 29 obligation sites where 17 had been predicted, instead of treating every scope-leaving edge as a kill at the head.

Rejected:
- Every scope-leaving edge kills each binding's facts at the loop head: rejected because one return inside a loop body discarded every pre-loop fact, and the identical program discharged with the return outside the loop and rejected with it inside.
