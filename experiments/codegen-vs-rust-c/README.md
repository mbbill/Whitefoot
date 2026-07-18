# Codegen/perf experiment: Whitefoot vs C / C++ / Rust

Reproducible sources for the 2026-07-08 head-to-head. **Findings & verdict:**
`../../optimizer-language-research/notes/codegen-vs-rust-c-2026-07-08.md`.

## Layout
- `xl/`  — Whitefoot kernels (`.wf`) + democ-emitted `.ll` (note the auto `noalias`/`readonly`).
- `c/ cpp/ rs/` — the same kernels in C / C++ / Rust.
- `kb/` — Kernel-B accumulate/reduction variants (naive vs `restrict` vs Whitefoot vs Rust) + shared `driver.c`.
- `asm/` — `-O2` hot-loop assembly for every version (the codegen evidence).
- `time_it.py` — the timing harness (warmup + median of N runs).

## Kernels
- **A — splitmix64** mixing, xor-accumulated: parity test (byte-identical hot loop → parity).
- **B — `accumulate(&uniq acc, & addend, n)`**: the noalias demonstration.
  - `B1` non-affine body → Whitefoot == C-restrict == Rust codegen; runtime latency-hidden on M4.
  - `B-add` affine reduction `*acc += *addend` → Whitefoot == C-restrict == Rust codegen; noalias collapses O(n)→O(1) (`madd`), ~22x vs naive C.

## Reproduce
Whitefoot: `python3 ../../prototype/democ/democ.py xl/kernelA.wf` → `.ll`, then `/usr/bin/clang -O2 xl/kernelA.ll -o A`.
C/C++: `/usr/bin/clang[++] -O2 c/kernelA.c -o A`. Rust: `rustc -C opt-level=3 rs/kernelA.rs -o A`; Kernel-B Rust variants live in `kb/acc.rs` and `kb/add.rs`.
Assembly: add `-S` (or `rustc --emit asm`). Timing: `python3 time_it.py`.
Note: democ hard-codes `/usr/bin/clang` (Apple clang 21); the default PATH `clang` here is a WASI cross-compiler — use `/usr/bin/clang` for an apples-to-apples LLVM 21 comparison.
