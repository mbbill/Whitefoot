Decision: The compiler has no totality analysis and emits no will-return fact, and any future totality fact is derived after semantic checking and never lets proof elision alter source effects or acceptance, because a fact that changed acceptance would violate the rule that optional facts never change which programs compile, instead of a totality analysis wired into checking.

Decision: If a totality fact is derived later, it follows from loop freedom, trap freedom, and total callees and not from a fully pure effect row, because a loop-free, trap-free function with memory effects still returns, so demanding a pure row blocked the fact with no soundness gain, instead of granting totality only to pure functions.

Rejected:
- Will-return granted only to a function whose whole effect row is pure: rejected because memory-only effects are termination-irrelevant, so the restriction blocked the fact with no soundness gain.
