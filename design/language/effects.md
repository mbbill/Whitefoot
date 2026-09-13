Decision: An effect row is checked for exactness in both directions, an exhibited but undeclared effect and a declared but unexhibited effect are both errors, and a row stays exact even where a later proof could remove the effect, because an exact row is a statement a reader and the checker can rely on without reading the body, padding a row would be a place to smuggle effects, and the later-proof clause keeps acceptance decidable from the signature alone, instead of rows as upper bounds.

Decision: Effects describe ordinary state including opaque system resources, reads and writes name parameter paths, regions describe lifetimes and not state identity, and host scheduling belongs to target lowering, because one state vocabulary serves system resources and memory alike, instead of separate external, blocks, and traps categories.

Decision: Contracts and invariants are erased proof syntax and introduce no effect, because a proof-only statement that changed a row would be observable, instead of proof statements contributing effects.

Decision: `pure` states that a function has no state effects and promises nothing about termination, because the language has no termination checker, and a promise the checker cannot verify would be a trusted theorem, instead of `pure` as totality.

Decision: Ordinary heap allocation is ambient and absent from a function's effect row, while allocation into a store-branded provider such as an arena appears as a formal-rooted allocates path and needs that provider as an explicit parameter, because only a store whose identity can vary needs a caller-visible capability to tell it from another, and there is one heap, instead of making every allocation capability-visible or none.

Rejected:
- Separate external, blocks, and traps effect categories: rejected because system resources are ordinary state under ownership and scheduling belongs to lowering, so the categories described mechanisms rather than state.
