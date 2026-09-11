Decision: Optional optimizer facts never change an accepted program's observable behavior, because the backend trusts an emitted attribute without re-checking it so a wrong attribute is a silent miscompile that only a correct facts-off reference can expose, because each fact family's performance gain is attributed by measuring it against that same facts-off reference, and because a facts-on build that behaved differently would reintroduce the debug-versus-release split in which one source ships two programs, instead of fact-selected lowering paths; the language tree's root holds the matching rule that facts never change acceptance.

Decision: A fact family may authorize an optimization only after its gain is attributed against a correct facts-off build and it has survived hostile tests that plant wrong facts, because an attribute the backend trusts is a miscompile if wrong and a measurement without a baseline attributes nothing, instead of enabling a fact channel because its emission looks right.

Decision: The checker is part of the trusted computing base, and a wrong discharge is a compiler soundness defect fixed in code and tests, because conformance expectations state the language and not the compiler, instead of revising a conformance verdict to match compiler behavior.

Decision: The compiler is one mutable safe-Rust crate whose internal interfaces may change as language experiments demand, because a permanent checked-artifact architecture treated research boundaries as product protocols and multiplied crates and gates before a resolver or backend existed, instead of a permanent artifact compiler with stable crate boundaries.

Decision: The checked in-memory representation is the sole lowering authority and no serialized or replayed form grants authority, because same-kernel serialization and replay added no independent semantic evidence and imposed a protocol before any real artifact consumer existed, instead of mandatory artifact replay.

Decision: Artifacts, replay, stable protocols, release machinery, and product-scale resource controls are later hardening and not compiler prerequisites, because they delayed the first executable compiler without serving the research goal while proportional independent tests keep the useful correctness constraints, instead of a product-scale checked-artifact toolchain.

Decision: Conformance cases are independent evidence and never an acceptance authority, because the numbered specification alone defines source-language behavior and a second authority would let tests define the language, instead of production acceptance driven by the corpus.

Decision: Any partition of the implementation into stages or slices describes build and test order only and never becomes a normative admission profile, a function or signature allowlist, or an alternate compiler path, because a capability implemented by shape or identity rather than by rule is a special case the language does not have, instead of profile-gated acceptance.

Decision: A post-resolution rejection must establish an actual numbered-rule violation and be deterministic for one compiler executable, while the choice among competing first errors is not part of the language's identity, because a writer acts on a rule and a location but two correct checkers may legitimately meet different first violations, instead of specifying a portable first-error order.

Rejected:
- A Python reference-model gate: rejected because it consumed a historical toy syntax tree and neither exercised nor compared with the Rust compiler, so it did not justify its workflow and maintenance cost.

Decision: Valid specified source that the compiler has not implemented stops as an explicit unsupported capability and is never reported as invalid source, because a compiler gap reported as a language rejection rewrites the language from the implementation side, instead of misreporting unsupported source as invalid.

Decision: The umbrella target is ripgrep with a fair two-times end-to-end objective, and performance comes first in that loop, so a missing performance capability stops downstream expansion until its owning layer is fixed rather than being written around, because the target exists to expose general language defects and to attribute wins to generated code, and writing downstream code on top of a missing capability hides the blocker inside an integration, instead of SQLite as the umbrella target or shipping the finished tool as the completion criterion.
