Decision: Every requirement of an ordinary call is proved in the caller's state just before the call, with no fallback runtime check and no callee prologue, because an unconditional callee prologue let a helper hide a protected leaf behind a runtime trap, instead of a callee-entry prologue.

Decision: The sole command entry carries no contract and cannot be called from source, because with one uncallable entry, no foreign interface, and contracts forbidden on the entry, every requirement belongs to an internal call and an entry-only runtime exception would violate the no-trap boundary, instead of a checked process-entry wrapper.

Decision: A future foreign or exported boundary must introduce its own separately selected proof adapter before it can call an internal required function, because this closed-world rule reserves no unchecked entry exception for a design that does not exist yet, instead of a standing entry exception.

Rejected:
- Callee-entry prologue evaluating the requirement on every invocation: rejected because it let a helper hide a protected leaf behind a runtime trap and added the prologue's reads to the callee effect row.
- Ordinary pre-transfer proof plus a checked process-entry wrapper: rejected because with one uncallable command entry and no foreign interface, the wrapper was an entry-only runtime exception that violated the no-trap boundary.
