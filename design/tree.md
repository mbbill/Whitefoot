# Whitefoot

Decision: Whitefoot is a systems language whose writer is an AI agent and whose approver is a person, because an AI writer pays verbosity in tokens, has no installed base or style attachment, and must not be given an escape hatch it will use when stuck, instead of a human-first language with AI assistance.
Decision: Every required source fact is machine-checked before acceptance and erased before lowering, with no writer-accessible unsafe, trusted theorem, or runtime proof trap, because any writer-reachable escape becomes an unauditable failure edge in generated code, instead of Rust-style unsafe policed by convention.
Decision: Within required safety and practical development feasibility, runtime performance is preferred over ease of writing and speed of compilation, because the target projects are kernels, compilers, and browsers, instead of optimizing manual writability.
Decision: Optional optimizer facts never change which programs are accepted, because acceptance must depend only on the specification-fixed checker and never on an optimizer's version, target, or pass order, instead of fact-dependent acceptance.
Decision: Optional optimizer facts never change an accepted program's observable behavior, because the backend trusts an emitted attribute without re-checking it so a wrong attribute is a silent miscompile that only a correct facts-off reference can expose, because each fact family's performance gain is attributed by measuring it against that same facts-off reference, and because a facts-on build that behaved differently would reintroduce the debug-versus-release split in which one source ships two programs, instead of fact-selected lowering paths.
Decision: Before real project adoption, migration cost of existing language designs is not a ground for language choices, because there are no users to migrate, instead of compatibility-driven design.

Scope: the whole project
