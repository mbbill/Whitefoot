Decision: Source contracts and conformances, the trait-like declarations of member signatures and laws, are compile-time signature-and-law metadata that create no runtime value, ABI component, dispatch path, lowering operation, or optimizer authority, because a contract that lowered to anything would be a second semantics, instead of runtime interface objects.

Decision: Member compatibility is exact after positional region renaming, with independently valid effect rows required to have equal normalized read, write, and allocation components and no subtyping, because normalized equality preserves signature regularity across irrelevant occurrence order, repetition, and region spelling while still requiring every capability on both sides, instead of raw source-row equality or effect subtyping.

Decision: A conformance that carries laws must discharge every law for the source to be accepted, and a law is used only where a rule of the specification names it, such as the permission for a parallel reduction, because a law proved anywhere else, by an optimizer or on trust, would be a second proof path outside the checker, instead of optimizer-side law proofs.

Decision: Generic bounds over source contracts and member calls stay absent until a real behavior consumer and the interfaces experiment justify their complete syntax, semantics, diagnostics, and direct-call proof, because a capability with no consumer cannot be measured, instead of adding the surface ahead of a need.

Rejected:
- Raw source-row equality for member effect compatibility: rejected because it breaks signature regularity across irrelevant occurrence order, repetition, and region spelling.
