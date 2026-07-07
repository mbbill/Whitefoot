# Open questions

These are research/debate questions, not decisions.

## From deep-research workflow

- How should an optimizer-first new language balance Rust/SPARK-style ownership and anti-aliasing guarantees against C/Fortran-style opt-in `restrict`/dummy-argument contracts for usability and FFI?
- Which data-layout, array, iterator, and loop constructs from DSLs such as Halide, ISPC, Futhark, Chapel, MLIR, and Mojo best expose vectorization, locality, and parallelism without over-specializing the surface language?
- What effect, exception, and concurrency model would preserve enough ordering and side-effect information for aggressive optimization while remaining practical for systems and application programming?
- For managed memory options, when should the language prefer explicit ownership/ARC/regions/borrowing over GC plus escape analysis, given that HotSpot-style allocation and lock elimination are conditional rather than guaranteed?

## From synthesis/debate planning

- Evidence standard first: what qualifies as enough support for a design implication, and how should the corpus weigh specification text, implementation docs, source code, academic papers, benchmark data, production reports, and historical retrospectives?
- Breadth check before feature ranking: is the corpus broad enough to compare optimizer enablers, or must debates pause until arrays/layout/vectorization, dispatch/generics, numeric semantics, dynamic typing/JIT, memory models, concurrency, effects, and ABI have workflow-confirmed evidence?
- Layering check: for each claim under discussion, is it a source-language semantic rule, verifier/proof obligation, compiler analysis, IR annotation, runtime/JIT mechanism, ABI representation, backend pass fact, or empirical performance result?
- Aliasing as one option among several: should non-aliasing be modeled as a default semantic rule, opt-in contract, proof obligation, inferred property, dynamic guard, unsafe promise, or conservative optimization opportunity? Compare Rust, SPARK, Fortran, C restrict, Clean uniqueness, Pony capabilities, and JIT guard/deopt systems without selecting a winner.
- IR preservation question in neutral form: what evidence exists that preserving each source-level fact into MLIR or LLVM enables specific optimizations, which passes consume it, where can it be invalidated or dropped, and what costs or hazards follow?
- Managed/runtime question: when are escape analysis, scalar replacement, inline caches, guards, tiering, and deoptimization sufficient, and when would a language-level allocation, lifetime, ownership, or effect guarantee be needed?
- Exception/error/effect question: compare result types, return codes, zero-cost exceptions, funclets, checked exceptions, algebraic effects, condition systems, panic/abort, setjmp/longjmp, coroutines, and async cancellation while separating source ergonomics from ABI/backend representation.
- Array/layout/vectorization question: should array shape, layout, iteration spaces, contiguity, bounds, and scheduling be privileged as much as aliasing? Debate Fortran/APL/HPF/SISAL/NESL/SAC/Futhark/Halide/ISPC/MLIR evidence needs against the current alias-first seed corpus.
- Numeric semantics question: how much freedom should optimizers get around overflow, poison/undef/freeze, fast-math, NaNs, signed zero, rounding, reproducibility, and target-specific approximations? What user failure modes are acceptable?
- Concurrency and memory model question: how should data-race rules, atomics, actors, tasks, channels, Send/Sync-like capabilities, STM, GPU address spaces, and foreign threads bound alias/lifetime/value assumptions?
- Implementation gate question: what minimal semantic contract, LLVM/MLIR mapping, verifier check, threat model, and FileCheck or benchmark validation must a feature pass before moving from research evidence to design_candidate?
- Historical failure question: which failures must any future design explicitly avoid, including strict-aliasing surprises, UB/provenance traps, poison/undef miscompilations, checked-exception ergonomics, dynamic JIT warmup/deopt cliffs, Fortran aliasing exceptions, GPU portability traps, and unsound frontend metadata emission?
