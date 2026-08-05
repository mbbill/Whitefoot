# WF-SCAN-FLOOR results

Status: PREREGISTERED — no comparative current-compiler timing has run

The exact scanner shapes, runtime inputs, correctness oracle, ordinary compiler
path, experiment-only linkage boundary, code-shape inspection, paired schedule,
statistics, interpretation bands, and stop condition are frozen in
`PROTOCOL.md` and the committed sources beside this file.

Preparation has established only that both Whitefoot sources compile through
the ordinary current compiler and all six implementations pass the independent
small-case correctness gate. Comparative timing has not run. This file makes no
performance or language-floor claim.

## Frozen pre-timing code shape

The full-pass Whitefoot raw LLVM retains one explicit bounds trap in the source
index loop. Apple Clang `-O2` removes that trap from `wf_scan`, builds a main
`<16 x i8>` vector loop plus a `<4 x i8>` vector epilogue, and emits no helper or
library call. The same-Clang C control has the same 16-byte load, two byte
comparisons, widening, accumulation, and scalar-tail structure; register
allocation and assembly metadata differ.

The early-exit Whitefoot helper likewise contains one raw bounds trap. The
optimized `wf_scan` contains no trap call and inlines four scalar byte-search
loops. The C control has the same four-loop structure and no helper or library
call. The safe-Rust control retains four calls to its cross-crate `find_byte`,
so its early result is secondary toolchain/source-boundary evidence rather than
a substitute for the Whitefoot/C causal comparison.

These observations establish the expected final-code consequences before
timing; they do not establish throughput parity.
