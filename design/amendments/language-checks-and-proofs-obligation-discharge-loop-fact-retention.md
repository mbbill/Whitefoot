Node: language/checks-and-proofs/obligation-discharge/loop-fact-retention

Decision: A pre-loop fact survives a loop head unless a body kill can reach a later head without leaving the body, because return, propagated error and break paths cannot reach that head; the [return-placement comparison](https://github.com/mbbill/Whitefoot/blob/b3341e32ab8d69e2974227aee7146ac16eb2fe00/research/investigations/obligation-discharge/ACCEPTANCE.md#the-dominant-cause-unattributed-before-this-run) isolates the fact loss caused by treating them as backedges, instead of killing head facts on every scope-leaving edge.

Rejected:
- Every scope-leaving edge kills each binding's facts at the loop head: rejected because one return inside a loop body discarded every pre-loop fact, and the identical program discharged with the return outside the loop and rejected with it inside.
