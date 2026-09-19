Node: compiler/checker-facts

Decision: An `==` invariant target is recorded as the one normalized bound `a-b <= 0` plus a flag saying the reversed bound `b-a <= 0` belongs to the same batch, because [INV-1] normalizes `a == b` to exactly that bound pair and proves each as one batch member, so the second member is the first with its sides exchanged and carries no independent written form, instead of storing the target as a list of relations, which would give every consumer  -  base batch, backedge batch, exhaustion export, certificate residual, and the `use` premise a name cites  -  a list to pick one member from where [PRF-1] admits exactly one inequality.

Decision: Every place that proves an invariant target proves every member of its batch and publishes every member as a fact, while the name an invariant declares keeps only the first member as the premise a later `use` may cite, because [INV-1] makes the batch simultaneous  -  no header conclusion is published unless every base target succeeds  -  while [PRF-1] adds one normalized premise into one sum, instead of publishing the pair under the cited name, which would make one `use` add two premises.

Decision: `==` written in a `use_premise` keeps its refusal and states that a certificate premise is one inequality and an equality is a bound pair, because [INV-1] admits `==` in an invariant target and refuses it in a premise, and the two positions differ only in which rule the diagnostic cites, instead of one shared message that calls equality no invariant relation, which is now false of the target position.

Rejected:
- Proving only the forward bound and treating the reverse as a consequence: rejected because the two bounds are independent affine goals and nothing in the affine fragment derives one from the other.
