Decision: Every required source fact is machine-checked before acceptance and erased before lowering, with no writer-accessible unsafe, trusted theorem, or runtime trap of any kind, because any writer-reachable escape becomes an unauditable failure edge in generated code, instead of Rust-style unsafe policed by convention or panics admitted as a language feature.

Decision: Compilation and proof checking never require work that grows exponentially with program or written-proof size, because the target projects must stay practical to iterate on and guaranteed termination alone does not make a checker usable, instead of admitting exponential families and relying on timeouts or budgets to keep them practical.

Decision: One source produces one program: no build mode, compiler flag, or backend setting changes an accepted program's observable behavior, and what the compiler hands its backend, such as aliasing attributes or inlining, is an implementation detail the language never sees, because a debug build and a release build that behave differently, as integer overflow does in Rust, ship two programs from one source, instead of debug and release modes.

Decision: Before real project adoption, migration cost of existing language designs is not a ground for language choices, because there are no users to migrate, instead of compatibility-driven design.
