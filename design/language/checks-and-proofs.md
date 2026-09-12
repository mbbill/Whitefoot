Decision: Automatic derivation is a fixed set of terminating families that run to completion, and a harder proof is a finite explicit certificate the checker verifies without rediscovering it, because acceptance must never depend on solver state, a timeout, machine speed, or a work budget, instead of SMT-backed acceptance.

Decision: Required partial-operation domains are established only by the specification's deterministic proof system, whose facts come from admitted types and declarations, selected control-flow edges, verified contracts, and checked invariants, instead of writer assertions.

Decision: A nonempty explicit certificate is rejected when the automatic families already prove its target, because the boundary between automatic and written proof must be decidable from the language rules rather than by probing the compiler, instead of tolerating redundant certificates.

Rejected:
- Tolerating an unused explicit certificate as advice rather than an error: rejected because the redundancy rule ties the writer's choice to the exact language version instead of to compiler behavior.
- Limiting the explicit certificate language to what the automatic families can already prove: rejected because a certificate that can only restate an automatic proof adds no proof, so the certificate language must reach beyond the automatic families.
