# Whitefoot

Decision: Whitefoot is a systems language whose writer is an AI agent and whose approver is a person, because an AI writer pays verbosity in tokens, has no installed base or style attachment, and must not be given an escape hatch it will use when stuck, instead of a human-first language with AI assistance.
Decision: Every required source fact is machine-checked before acceptance and erased before lowering, with no writer-accessible unsafe, trusted theorem, or runtime proof trap, because any writer-reachable escape becomes an unauditable failure edge in generated code, instead of Rust-style unsafe policed by convention.
Decision: Within required safety and practical development feasibility, runtime performance is preferred over ease of writing and speed of compilation, because the target projects are kernels, compilers, and browsers, instead of optimizing manual writability.
Decision: Optional optimizer facts may improve an accepted program but never change acceptance or semantics, because facts-off compilation must remain correct, instead of fact-dependent acceptance.
Decision: Before real project adoption, migration cost of existing language designs is not a ground for language choices, because there are no users to migrate, instead of compatibility-driven design.

Scope: the whole project
