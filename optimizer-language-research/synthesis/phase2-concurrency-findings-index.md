# Phase-2 concurrency and memory model findings

Direct source-specific extraction after broad/narrow workflows stalled or returned empty. Evidence cards, not design decisions.

| ID | Theme | Claim summary | Sources | Scope limit |
|---|---|---|---|---|
| C001 | DRF-SC convergence | Major languages (C, C++, Java, JavaScript, Rust, Swift, Go) converge on the DRF-SC guarantee: data-race-free programs behave sequentially consistently, with synchronization operations def... | https://research.swtch.com/plmm, https://go.dev/ref/mem | Does not decide how racy programs should behave (UB vs bounded); survey is a secondary source. |
| C002 | racy-program semantics split | Languages split on race semantics: C/C++/Rust/Swift make data races undefined behavior, while Java/JavaScript/Go bound racy outcomes (Java for security, Go by explicitly constraining impl... | https://research.swtch.com/plmm, https://go.dev/ref/mem, https://www.cs.umd.edu/~pugh/java/memoryModel/jsr-133-faq.html | No performance quantification of the difference; the right choice for a new language is an open debate question. |
| C003 | concurrency constrains ordinary optimization | Memory models restrict classic optimizations on possibly-shared memory: compilers must not introduce writes on paths that would not store, must not let one read observe multiple values (r... | https://go.dev/ref/mem, https://www.cs.umd.edu/~pugh/java/memoryModel/jsr-133-faq.html, https://research.swtch.com/plmm, https://llvm.org/docs/Atomics.html | Rules differ per language/IR; LLVM IR semantics (undef racing loads) are not identical to source models. |
| C004 | ownership as race prevention | Rust demonstrates a type-system route to data-race freedom: Send/Sync auto traits plus exclusive &mut borrowing prevent data races in safe code, so the optimizer can treat non-atomic memo... | https://doc.rust-lang.org/nomicon/send-and-sync.html, https://doc.rust-lang.org/nomicon/atomics.html | Unsafe code and incorrect manual Send/Sync impls are UB; does not eliminate the need for an atomics model. |
| C005 | IR-level atomics gradation | LLVM grades atomic orderings from NotAtomic through SequentiallyConsistent, each enabling/forbidding specific transformations (splitting, narrowing, rematerialization, hoisting, DSE), and... | https://llvm.org/docs/Atomics.html | IR contracts require correct frontend lowering; pass-specific legality varies. |
| C006 | publication/initialization safety | Java final fields give initialization safety (readers see final field values without synchronization) only when the reference does not escape construction — an example of a language carvi... | https://www.cs.umd.edu/~pugh/java/memoryModel/jsr-133-faq.html | FAQ-level source; escape analysis and constructor discipline are preconditions. |

## Main patterns observed

- DRF-SC is the common baseline across modern languages; the real design split is what racy programs may do (UB vs bounded outcomes).
- Shared-memory possibility taxes ordinary optimizations: no invented writes, no re-loading single reads, no register-caching of atomics/volatiles, fences as motion barriers.
- Rust shows sharing can be a type-level fact (Send/Sync + borrowing), letting non-shared memory keep full optimizer freedom — this composes with the aliasing evidence (F001, C004).
- LLVM provides a graded atomic vocabulary; source design must map onto it soundly.
- Failed fetches: eel.is C++ draft, docs.oracle.com JLS, cppreference (403/connection). C++/Java claims rest on JSR-133 FAQ and the plmm survey; upgrade to primary standard text later.
