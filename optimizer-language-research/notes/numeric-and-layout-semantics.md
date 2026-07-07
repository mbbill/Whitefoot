# Numeric and layout semantics research note

This note marks a high-priority evidence gap. The first workflow did not sufficiently verify numeric or representation choices. Do not choose defaults until primary sources are gathered.

## Numeric topics needing primary evidence

- Integer overflow: trapping, wrapping, saturating, checked, undefined, poison/undef/freeze, `noundef`, range metadata.
- Floating point: IEEE 754 strictness, NaNs, signed zero, infinities, rounding modes, FP exceptions, reassociation, FMA contraction, fast-math flags, reproducibility, target-specific approximations.
- Optimizer consumers: InstCombine, ScalarEvolution, vectorizers, loop transforms, target lowering.

## Layout topics needing primary evidence

- Fortran column-major arrays and array descriptors.
- C/C++ row-major arrays, structs, padding, aliasing/provenance.
- AoS vs SoA vs AoSoA and cache/vector implications.
- Object headers, tagged values, boxing/unboxing, nullable/reference representation.
- C# structs, Swift value types, Rust niche optimization, Haskell unboxed types, MLton flattening.
- User-controlled layout in Zig/Rust/C/C++ and constraints around ABI/FFI.

## Required next step

Create evidence cards with exact sources before numeric or layout semantics become design candidates.
