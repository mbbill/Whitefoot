Node: compiler

Decision: The compiler remains one safe-Rust research crate with private evolving interfaces, while compiler-version-private persistent query results retain checked work across invocations through the same semantic path and ordinary final native linking, because the [modular-compilation design](../../research/investigations/modular-compilation/DESIGN.md) supplies a concrete consumer for retained verification and optimized objects without requiring a stable artifact ABI or independent proof replay, instead of repeating all semantic work before splitting LLVM output or constructing a product-grade artifact protocol before a consumer exists.

Rejected:
- Replaced first compiler decision beginning "The compiler is a research instrument for iterating the language": rejected because its artifact boundary does not yet distinguish the requested version-private persistent results from the refused product protocol; its safe-Rust crate, research purpose, private interfaces and refusal of stable artifact/replay/release infrastructure remain in force.
- A separate incremental checker beside the clean checker: rejected because cache state must affect reused computation, not which semantic rules decide acceptance.
