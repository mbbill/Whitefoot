# Phase-2 dynamic JIT and deoptimization findings

Direct source-specific extraction. Evidence cards, not design decisions.

| ID | Theme | Claim summary | Sources | Scope limit |
|---|---|---|---|---|
| J001 | specialization as the core dynamic optimization | Dynamic-language runtimes (CPython PEP 659, V8, PyPy) converge on runtime specialization: observe concrete types/shapes/values, emit specialized fast paths, and guard them with cheap checks. | https://peps.python.org/pep-0659/, https://v8.dev/docs/hidden-classes, https://doc.pypy.org/en/latest/architecture.html | Does not quantify the residual cost of guards versus static knowledge; sources are mixed authority (PEP, vendor docs, blog). |
| J002 | guards and deoptimization as the price of speculation | Speculative optimizations require invalidation machinery: CPython counters de-specialize instructions, V8 marks code for deoptimization when const-field or map assumptions break, and elem... | https://peps.python.org/pep-0659/, https://v8.dev/docs/hidden-classes, https://v8.dev/blog/elements-kinds | Deoptimization also enables optimizations static compilers cannot legally do without profiles; not purely a cost. |
| J003 | object shape stability | V8 optimizes property access via hidden-class maps and transition chains; monomorphic shapes and packed element kinds optimize best, while shape churn, out-of-order properties, holes, and... | https://v8.dev/docs/hidden-classes, https://v8.dev/blog/elements-kinds | V8-specific machinery; blog notes real-world differences are often small. |
| J004 | managed-runtime allocation optimization limits | Oracle documents HotSpot escape analysis as eliminating scalar-replaceable allocations and eliding locks, while explicitly stating it does not replace heap allocation with stack allocatio... | https://docs.oracle.com/en/java/javase/21/vm/java-hotspot-virtual-machine-performance-enhancements.html | Version-specific; says nothing about other JVMs or future changes. |
| J005 | tiering and warmup | HotSpot tiered compilation trades code-cache memory for startup speed by running the client compiler during profiling before the server compiler finalizes code — evidence that profile-dri... | https://docs.oracle.com/en/java/javase/21/vm/java-hotspot-virtual-machine-performance-enhancements.html | Single-implementation evidence; startup/memory tradeoffs vary by workload. |

## Main patterns observed

- Dynamic runtimes recover optimizer-needed facts (types, shapes, layouts, call targets) via profiling, inline caches, and speculation with guards.
- The recovery machinery has costs: warmup, memory, guard checks, deopt cliffs, one-directional generalization.
- Vendor documentation confirms managed allocation optimization is conditional (HotSpot escape analysis does not stack-allocate).
- Central debate question this raises: which facts should an optimizer-first language make static contracts, and which (if any) should be left to profile-driven machinery?

## Gaps

- Self/Smalltalk historical papers, TurboFan/deopt internals, PyPy guard papers, Julia internals beyond manual level.
