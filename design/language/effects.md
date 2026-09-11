Decision: An effect row is checked for exactness in both directions, an exhibited but undeclared effect and a declared but unexhibited effect are both errors, and a row stays exact even where a later proof could remove the effect, because exactness maximizes the facts an optimizer can trust and blocks padding a row as a place to smuggle effects, and the later-proof clause keeps acceptance decidable from the artifact alone, instead of rows as upper bounds.

Decision: Effects describe ordinary state including opaque system resources, reads and writes name parameter paths, regions describe lifetimes and not state identity, and host scheduling belongs to target lowering, because one state vocabulary serves system resources and memory alike, instead of separate external, blocks, and traps categories.

Decision: Contracts and invariants are erased proof syntax and introduce no effect, because a proof-only statement that changed a row would be observable, instead of proof statements contributing effects.

Rejected:
- Separate external, blocks, and traps effect categories: rejected because system resources are ordinary state under ownership and scheduling belongs to lowering, so the categories described mechanisms rather than state.
