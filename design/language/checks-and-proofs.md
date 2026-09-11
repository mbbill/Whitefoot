Decision: Required partial-operation domains are established only by the specification's deterministic proof system, whose facts come from admitted types and declarations, selected control-flow edges, verified contracts, and checked invariants, because a writer conclusion taken on trust is an unauditable escape, instead of trusting writer assertions.

Decision: Automatic derivation is a fixed set of terminating families that run to completion, and a harder proof is a finite explicit certificate the checker verifies without rediscovering it, because acceptance must never depend on solver state, a timeout, machine speed, or a work budget, instead of SMT-backed acceptance.

Decision: A nonempty explicit certificate is rejected when the automatic families already prove its target, because the boundary between automatic and written proof must be decidable from the language rules rather than by probing the compiler, instead of tolerating redundant certificates.

Decision: Proof syntax is erased before lowering and a partial operation never receives a runtime proof fallback, because a proof that adds a branch is executable control flow and not evidence, instead of retained-check or claim-trap models.

Rejected:
- Tolerating an unused explicit certificate as advice rather than an error: rejected because the redundancy rule ties the writer's choice to the exact language version instead of to compiler behavior
- Limiting the certificate language to what the automatic families can prove: rejected because the explicit certificate language, not the absence of SMT, determines which further proofs can be written.
