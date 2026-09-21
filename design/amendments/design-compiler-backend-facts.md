Node: compiler/backend-facts

Decision: Supply checked facts as target attributes, instruction flags, metadata or assumptions only when retained evidence establishes the complete target contract, because DIAG-2 permits those optimization forms but source rule names alone do not establish their full alias, capture, extent, memory or arithmetic promises. Optional fact emission changes neither source acceptance nor target qualification and adds no runtime checks, instead of withholding all proved information or using optimizer assertions as a new source of proof. No current `llvm.assume` is emitted without such a complete mapping.

Decision: Reference attributes use ordinary EFF-5 call-site disjointness where the target contract needs it, excluding `noalias` for the alias-permitting `swap`; `dereferenceable` uses the selected target's guaranteed referent size, and exact arithmetic alone receives the corresponding no-wrap flags, because the language layout ceiling is an upper bound rather than a guaranteed extent and wrapping operations deliberately admit overflowing operands, instead of uniform attributes across operations with different contracts.

Decision: Probe the no-capture spelling with the build's assembler and fall back to the supported legacy spelling, because vendor version strings do not determine which attribute syntax that assembler accepts, instead of pinning LLVM or inferring support from a marketing version number.

Rejected:
- Inferring `memory(argmem: read)` from a row without writes: rejected because the row does not account for constant storage or allocation. IR carries an allocation fact, but there is no complete transitive mapping of all non-argument memory effects; partial metadata cannot justify the attribute.
- Treating eligible fact families as an implemented blanket set: rejected because each needs its own complete retained-evidence-to-target mapping, and missing optimizer metadata must not become an acceptance failure.
