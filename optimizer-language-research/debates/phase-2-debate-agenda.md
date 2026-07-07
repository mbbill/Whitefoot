# Debate agenda for optimizer-first language design

This agenda is for the next phase: agents should argue from sourced evidence before any final design commitments.

## Ground rules

- Treat the verified research files as evidence, not as decisions.
- Require citations or precise compiler/language references for new claims.
- Separate “optimizer wants this” from “language should expose this”.
- Record tradeoffs and failure modes, not only benefits.

## Questions to debate

1. How should an optimizer-first new language balance Rust/SPARK-style ownership and anti-aliasing guarantees against C/Fortran-style opt-in `restrict`/dummy-argument contracts for usability and FFI?
2. Which data-layout, array, iterator, and loop constructs from DSLs such as Halide, ISPC, Futhark, Chapel, MLIR, and Mojo best expose vectorization, locality, and parallelism without over-specializing the surface language?
3. What effect, exception, and concurrency model would preserve enough ordering and side-effect information for aggressive optimization while remaining practical for systems and application programming?
4. For managed memory options, when should the language prefer explicit ownership/ARC/regions/borrowing over GC plus escape analysis, given that HotSpot-style allocation and lock elimination are conditional rather than guaranteed?
17. How much alias information should be proven by the type system versus asserted in unsafe contracts?
18. What is the acceptable replacement for C-style UB when optimization contracts are violated: proof, debug checks, traps, unsafe-only assumptions, or some mix?
19. Should the language be human-authored, generated, or dual-mode? How does that change verbosity and explicitness?
20. Should the core IR be LLVM-first, MLIR-first, or a custom high-level IR lowering into LLVM?
21. How should high-level array/loop semantics be represented without overfitting to numerical/HPC workloads?
22. Which runtime features are banned from the performance core versus allowed at module boundaries?
23. How to preserve exact source-level facts through lowering so optimizer-visible metadata remains valid?
