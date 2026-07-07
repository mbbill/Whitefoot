# Caveats and evidence discipline

These caveats constrain how the seed corpus may be used. They are meant to prevent premature language-design decisions.

## From deep research result

All included claims were unanimously verified, but the corpus is narrow relative to the original research scope. It strongly covers aliasing, ownership, no-alias IR metadata, escape analysis, exception representation, and Swift ARC/SIL; it does not provide similarly verified coverage for Zig, Go, C#, Julia, Haskell/ML/Futhark, Python/Ruby/JavaScript, Halide, ISPC, Chapel, Mojo, MLIR, dynamic typing, numeric semantics, memory models, or most concurrency/iterator/data-layout topics. Rust aliasing evidence comes partly from the Nomicon, which is explanatory rather than a complete formal aliasing model; unsafe Rust, `UnsafeCell`, raw pointers, and future aliasing-model changes remain important caveats. Fortran rules include POINTER, TARGET, and coarray exceptions; SPARK permits benign/read-only and ownership-controlled aliasing; HotSpot optimizations depend on inlining, analysis limits, tier/options, and implementation details; Swift SIL has OSSA/non-OSSA phase distinctions. Source time-sensitivity is moderate: compiler IR semantics and VM optimizer behavior can change, though checked sources were current or corroborated across current documentation where noted.

## From synthesis critique

- Do not edit the raw deep-research JSON; provenance and checksum belong in the sidecar manifest.
- Do not claim the artifact contains all 134 extracted claims or intermediate per-agent outputs unless direct inspection confirms that content is present.
- Do not treat an empty unverified list from the selected verification batch as evidence that all unverified or dropped claims are settled; the run reported 134 extracted claims, 25 verified, 23 confirmed, 2 refuted, 9 synthesized findings, and at least 2 budget-dropped claims.
- Do not silently discard the two refuted LLVM exception claims; store them with exact reasons so similar overbroad claims are not reintroduced later.
- The corpus is aliasing-heavy because of selection, not because relative importance has been established across arrays, layout, dispatch, numerics, concurrency, DSLs, dynamic runtimes, or ABI constraints.
- Documentation/specification evidence can show what a language or compiler claims, but usually does not quantify performance magnitude, code-size impact, compile-time cost, portability, safety, or ergonomics.
- Version-sensitive sources require dates and version fields: Swift docs, LLVM 23.0.0git docs, HotSpot docs across Java versions, vendor blogs, implementation docs, project homepages, and source-code commits may drift.
- Rust, Fortran, SPARK, HotSpot, LLVM, and Swift findings must carry their caveats: unsafe escape hatches, POINTER/TARGET/coarray exceptions, benign/read-only aliasing, inlining and implementation limits, EH wording/refutation boundaries, OSSA/non-OSSA phase distinctions, and stale-doc risks.
- Implementation files are research scaffolding, not production compiler design; they should constrain later debates by asking what can be represented, verified, preserved, and tested, not decide source syntax or semantics.
- No final language design decisions should be made from this plan; every proposed file should preserve evidence, gaps, scope limits, and questions for the next discussion phase.

## Standing rule

Any future design proposal must cite exact evidence cards and explicitly state what remains unproven or unbenchmarked.
