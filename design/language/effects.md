Decision: An effect row is checked for exactness in both directions, an exhibited but undeclared effect and a declared but unexhibited effect are both errors, and a row stays exact even where a later proof could remove the effect, because an exact row is a statement a reader and the checker can rely on without reading the body, padding a row would be a place to smuggle effects, and the later-proof clause keeps acceptance decidable from the signature alone, instead of rows as upper bounds.

Decision: Effects describe resolved parameter storage, including ordinary opaque values, while moves into local storage create no effect root from an earlier owner, because a signature must describe the storage it accesses without recovering value history from callee bodies, instead of separate external categories or ownership-routing summaries.

Decision: Contracts and invariants are erased proof syntax and introduce no effect, because a proof-only statement that changed a row would be observable, instead of proof statements contributing effects.

Decision: `pure` states that a function has no state effects and promises nothing about termination, because the language has no termination checker, and a promise the checker cannot verify would be a trusted theorem, instead of `pure` as totality.

Decision: Store-branded allocation through Heap or Arena requires the provider as an ordinary parameter and names it in the effect row, while the legacy unbranded box and buffer operations retain ambient allocation, because different branded stores need distinct caller-visible capabilities and the legacy forms carry no such identity, instead of hiding branded allocation behind the legacy ambient-heap rule.

Decision: An exclusive helper's projected declared writes kill exactly the caller facts whose storage support overlaps, before verified exit contracts publish replacement facts, because preserving a changed inner run's former length is unsound and killing unrelated facts prevents useful modular proofs, instead of killing by the syntax of an actual argument or keeping an unknown replacement unchanged.

Rejected:
- Separate external, blocks, and traps effect categories: rejected because system resources are ordinary state under ownership and scheduling belongs to lowering, so the categories described mechanisms rather than state.
- Following owned values through locals, results, or exclusive replacement to derive additional effect roots: rejected because ordinary storage paths and explicit contracts supply the callable boundary, while ancestry makes that boundary depend on implementation bodies.
