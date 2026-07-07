# Phase-2 dispatch, generics, and runtime representation findings

Direct source-specific extraction after broad/narrow workflows stalled or returned empty. These are evidence cards, not design decisions.

| ID | Theme | Claim summary | Sources | Scope limit |
|---|---|---|---|---|
| D001 | static specialization / monomorphization | Rust generics are compiled by monomorphization into concrete type-specific versions for used types, giving runtime behavior like concrete code but with possible compile-time/code-size tra... | https://doc.rust-lang.org/book/ch10-01-syntax.html | Does not quantify code bloat, compile time, or cross-crate optimization limits. |
| D002 | dynamic dispatch representation/cost | Rust trait objects carry a data pointer plus method table/vtable and calls through dyn Trait use runtime virtual dispatch, which the Rust Book says adds lookup cost and prevents inlining ... | https://doc.rust-lang.org/book/ch18-02-trait-objects.html, https://doc.rust-lang.org/reference/types/trait-object.html | Trait objects provide heterogeneity and compile-time trait checking; exact ABI/devirtualization rules are not specified by these pages. |
| D003 | object safety / dyn compatibility restrictions | Rust restricts which traits and methods can be used for trait-object dispatch: dyn-compatible traits exclude Self:Sized requirements, associated constants, generic dispatchable methods, o... | https://doc.rust-lang.org/reference/items/traits.html#dyn-compatibility | These restrictions are semantic/type-system rules, not direct proof of optimization benefit. |
| D004 | optimization boundary / whole-program visibility | Rust codegen units, incremental compilation, and LTO expose a compile-time/runtime tradeoff: more separate units can speed compilation but may inhibit optimization, while LTO enables LLVM... | https://doc.rust-lang.org/rustc/codegen-options/index.html | Does not identify specific lost optimizations for generics; compiler options are implementation-specific. |
| D005 | runtime generics representation | .NET generics are reified enough that CIL contains generic metadata and reflection can inspect type arguments; the runtime specializes value-type instantiations but shares a reference-typ... | https://learn.microsoft.com/en-us/dotnet/csharp/programming-guide/generics/generics-in-the-run-time, https://learn.microsoft.com/en-us/dotnet/csharp/advanced-topics/reflection-and-attributes/generics-and-reflection | Does not prove specific JIT optimizations; reflection metadata may constrain closed-world assumptions unless restricted. |
| D006 | semantic finality | C# sealed classes prevent inheritance and sealed overrides prevent further overriding, giving semantic finality information; the fetched Microsoft source does not itself claim devirtualiz... | https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/keywords/sealed | Do not overclaim optimizer benefit without separate JIT/compiler evidence. |
| D007 | compiler devirtualization and IPA | GCC documents devirtualization from virtual calls to direct calls, speculative guarded devirtualization, indirect inlining after targets become known, and LTO/IPA mechanisms that increase... | https://gcc.gnu.org/onlinedocs/gcc/Optimize-Options.html | Compiler- and option-dependent; speculative devirtualization may be reverted, and LTO/IPA costs compile time/memory/code size. |
| D008 | semantic interposition / dynamic linking boundary | GCC documents that ELF semantic interposition can restrict interprocedural analysis, inlining, and propagation for exported symbols, while -fno-semantic-interposition assumes interposed d... | https://gcc.gnu.org/onlinedocs/gcc/Optimize-Options.html | ELF/GCC-specific; source-language design must separately choose module/ABI/interposition rules. |
| D009 | dynamic specialization / type stability | Julia performance guidance emphasizes type-stable functions, concrete fields/containers, function barriers, and careful specialization because abstract or changing types and Function-type... | https://docs.julialang.org/en/v1/manual/performance-tips/ | Heuristic and workload-dependent; over-specialization can increase compile-time/cache pressure. |
| D010 | multiple dispatch / method specialization | Julia dispatch selects methods by all positional argument types, and broad methods may still be compiled separately for concrete argument tuples so downstream calls can be resolved ahead ... | https://docs.julialang.org/en/v1/manual/methods/ | Methods listed by users are not the same as compiler specializations; method redefinition/world age and unlimited cases complicate optimizer assumptions. |

## Main patterns observed

- Static specialization/monomorphization exposes concrete types but can trade compile time/code size for runtime speed.
- Dynamic dispatch has representational costs: vtables/method tables, indirect calls, lost inlining unless devirtualized.
- Some languages restrict dynamic dispatch interfaces/object safety to keep trait-object representation coherent.
- JIT/dynamic systems can recover performance through type inference, type stability, method specialization, and function barriers, but this is heuristic and runtime-dependent.
- ABI/linking/reflection/interposition boundaries can prevent whole-program assumptions.

## Gaps

- Direct HotSpot primary extraction failed due 403; need alternative OpenJDK sources for CHA, inline caches, speculation/deoptimization, and escape analysis.
- C++ primary standard text/templates and virtual dispatch sources need better authoritative URLs than cppreference, which blocked fetch.
- Swift SIL/generics/witness tables remain unintegrated.
