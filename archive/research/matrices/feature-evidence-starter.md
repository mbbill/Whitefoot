# Feature evidence matrix — starter

This matrix is a research scaffold only. “Observed optimizer implication” summarizes verified evidence; it does not decide whether the future language keeps or ditches a feature.

| Area | Evidence theme | Languages / systems in evidence | Observed optimizer implication | Primary finding IDs |
|---|---|---|---|---|
| Aliasing / non-interference | Language-level non-aliasing and non-interference rules | Rust, Fortran, SPARK, G++ restrict, LLVM | Enables caching, register promotion, load/store elimination, reordering when writes cannot secretly interfere | F1,F2,F3,F4,F5,F6 |
| IR contracts | Optimizer-facing no-alias encodings | LLVM IR | `noalias` and scoped alias metadata preserve source facts for later passes; violating promises is UB | F3,F4 |
| Fortran array/procedure semantics | Dummy argument and TARGET rules | Fortran / Flang | Compiler may assume conforming programs obey aliasing restrictions; non-TARGET variables narrow pointer alias sets | F5 |
| Verification-oriented restrictions | Anti-aliasing and ownership in analyzable subset | SPARK | Restricts mutable aliasing and adds ownership/contracts for stronger static reasoning | F6 |
| Managed runtimes | Escape analysis / lock elimination | Java HotSpot, Go/managed runtime evidence from corpus | Can recover some allocation/synchronization performance conditionally, but not same as source-level guarantee | F7 |
| Exceptions / control flow | EH representation affects backend optimization | LLVM EH models | Some EH forms constrain register allocation/control-flow optimization; exception semantics are not free | F8 |
| High-level IR | Preserve source semantics into optimizer | Swift SIL, LLVM metadata, MLIR-like design evidence | Avoid erasing optimization-relevant facts too early; expose ARC/effects/ownership-like facts at IR level | F9 |
