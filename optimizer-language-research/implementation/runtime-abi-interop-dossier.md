# Runtime, ABI, and interoperability dossier

Capture constraints that determine which optimizer-visible facts can safely cross boundaries.

Topics: calling conventions, object layout, name mangling, generics/monomorphization vs dictionaries, vtables/witness tables, closures, stack maps, GC/RC hooks, unwinding personalities, TLS, coroutine frames, dynamic linking, LTO, debug info, sanitizers, profilers, package/versioning.

Interop targets: C, C++, Fortran, Rust, Swift, JVM/.NET, Python extension APIs, GPU/accelerator runtimes, and system libraries.


## Phase-2 dispatch/generics evidence summary

See `notes/phase2-dispatch-findings.jsonl` and `synthesis/phase2-dispatch-findings-index.md`. Key evidence tracks:

- Rust monomorphization and trait objects show static vs dynamic dispatch tradeoffs.
- Rust dyn compatibility shows dispatch interfaces may be restricted by type-system rules.
- .NET generics specialize value types while sharing reference-type generic bodies and preserving runtime metadata/reflection.
- C# sealed provides semantic finality but fetched source does not itself prove devirtualization.
- GCC documents devirtualization, speculative guarded devirtualization, IPA, LTO, and semantic-interposition boundaries.
- Julia shows a dynamic/multiple-dispatch system relying on type stability, inference, specialization, and function barriers, with over-specialization costs.

Open: HotSpot, Swift, better C++ standard/compiler evidence, and empirical validation.
