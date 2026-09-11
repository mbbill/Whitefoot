# Requirement enforcement

Scope: the execution boundary of every internal ordinary call

Decision: All substituted requirements of an ordinary call are judged independently in one shared pre-transfer state, after resolution, concrete instantiation, actual-expression obligations, and borrow feasibility are complete and before any consume, borrow commit, or callee-effect kill, and a refuted or unproved requirement rejects at the call with no fallback runtime check and no callee prologue, because an unconditional callee prologue let a helper hide a protected leaf behind a runtime trap, instead of a callee-entry prologue.

Decision: The sole command entry carries no contract and cannot be called from source, so program start has no requirement judgment, wrapper check, or contract-owned trap path, because with one source-uncallable entry, no foreign interface, and contracts forbidden on the entry, every requirement belongs to an internal call and an entry-only runtime exception would violate the no-trap boundary, instead of a checked process-entry wrapper.

Decision: A future foreign or exported boundary must introduce its own separately selected proof adapter before it can call an internal required function, because this closed-world rule reserves no unchecked entry exception for a design that does not exist yet, instead of a standing entry exception.

Rejected:
- Callee-entry prologue evaluating the requirement on every invocation: rejected because it let a helper hide a protected leaf behind a runtime trap and added the prologue's reads to the callee effect row.
- Ordinary pre-transfer proof plus a checked process-entry wrapper: rejected because with one uncallable command entry and no foreign interface, the wrapper was an entry-only runtime exception that violated the no-trap boundary.
