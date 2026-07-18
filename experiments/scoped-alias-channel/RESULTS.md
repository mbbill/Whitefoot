# Channel 1: Scoped-Alias Metadata from Ownership Provenance (F003)

Status: BUILT + MEASURED 2026-07-09. First differentiated result: real deltas at
short trip counts and a 17x code-size win; parity elsewhere.

## The fact channel

`buffer<T>` is affine single-owner (OWN-1/T1): two distinct live buffer values
never overlap. `&uniq` is exclusive (OWN-2/5) and loans have singleton
provenance (T-A). So inside a function taking `s: &uniq 'r Cols`, every buffer
field of `s` is pairwise-disjoint element memory, disjoint also from the struct
memory itself (v0 buffer elements are primitives in their own heap allocation).

rustc has NO source channel for this on loaded pointers: `&mut Cols` gets
parameter `noalias`, but the Vec data pointers loaded from it are fresh
provenance roots LLVM must treat as may-alias. democ emits per-provenance-class
`!alias.scope`/`!noalias` metadata (one scope per uniq-rooted buffer field, one
for struct memory, one shared class for all shared-borrow-rooted access), plus
`dereferenceable/align` on borrow params (borrow validity is a checker fact).

## Benchmark

`kernel.wf`: struct-of-arrays update, 8 u64 columns, two written, six read,
loop bounded by the imin of all lengths — the obvious source shape. Opaque
boundary: kernel compiled alone, called from a C driver (no LTO). Rust
adversaries (same semantics, `#[inline(never)]`): `obvious` (index via
`s.field[i]`), `rebind` (destructure + local slices), `innerfn` (expert shape:
inner function with 8 slice params — the only way safe Rust hands LLVM
noalias). rustc 1.x -O3, clang -O2, Apple M4.

ns/element (medians of the fixed-work sweep, k = 8e7/n):

| n | whitefoot-facts | whitefoot-control | rust-obvious | rust-rebind | rust-innerfn |
|---:|---:|---:|---:|---:|---:|
| 8 | **0.614** | 2.109 | 1.210 | 1.143 | 0.718 |
| 16 | **0.528** | 2.100 | 0.623 | 0.632 | 0.693 |
| 32 | 0.418 | 2.185 | 0.528 | 0.515 | 0.426 |
| 64 | 0.402 | 2.213 | 0.473 | 0.474 | 0.391 |
| 128 | 0.416 | 2.273 | 0.418 | 0.418 | 0.383 |
| 512 | 0.373 | 2.106 | 0.380 | 0.414 | 0.380 |
| 4096 | 0.886 | 2.541 | 0.673 | 0.677 | 0.648 |

Generated code (the mechanism, verified in asm):

| variant | vector adds | runtime guards | asm lines |
|---|---:|---:|---:|
| whitefoot-facts | 8 | 0 | 121 |
| whitefoot-control | 0 | 0 | 108 |
| rust-obvious | 65 | 29 | 2132 |
| rust-rebind | 73 | 54 | 2360 |
| rust-innerfn | 14 | 0 | 499 |

## Interpretation (honest)

1. **Rust's obvious shape IS vectorized** — via loop versioning with runtime
   alias checks. At n >= 32 those guards amortize and everything ties. The
   pre-registered risk ("recovered by runtime checks") materialized for long
   trips.
2. **Short trips are the real delta**: n=8 -> whitefoot-facts is 2.0x rust-obvious,
   1.9x rust-rebind, and 1.17x even the expert innerfn shape (which pays its
   extra call). n=16 -> 1.18x obvious. Short-trip kernels called in a loop are
   common (per-row updates, small fixed-width blocks).
3. **Code size is the durable win**: 121 lines vs 2132 for the same large-n
   speed — versioned-loop bloat is invisible in a microbenchmark but is
   i-cache pressure in real programs. Facts prune whole speculative versions,
   not just guards.
4. **Guarantee vs heuristic**: Rust's recovery is quadratic-in-pointers runtime
   disambiguation that eventually exceeds the vectorizer's check budget; the
   Whitefoot fact is static and O(1). (Bail-point sweep with wider structs:
   follow-up.)
5. The n=4096 facts-vs-rust gap (0.886 vs 0.65) is allocation-layout noise
   (C malloc vs Vec alignment); at k=1e5 fixed it measured 0.760 vs 0.754.
   Worth re-measuring with aligned allocation before quoting large-n numbers.
6. W1 story unchanged and strengthened: the OBVIOUS Whitefoot shape is the fast
   shape at every n; Rust's obvious shape is fine at large n only, and its
   fast-everywhere shape requires the inner-fn-with-slice-params idiom.

## Regression pin

`prototype/democ/perf_regress.py` now asserts (in `make check`): the
soa_kernel facts build carries `!alias.scope` and vectorizes with zero guards;
the control does neither.

## Addendum: 16-column bail-point probe (2026-07-09)

Falsification attempt on claim 4: at 16 columns (4 written, 12 read) LLVM STILL
version-vectorizes Rust's obvious shape — guards grow 29 -> 111 and asm
2132 -> 2836 lines (whitefoot-facts: 183 lines, 0 guards), but the vectorizer does
not bail, and times are near-parity at n >= 512 (memory-bound) with only ~1.16x
Whitefoot advantage at n=64. The versioning budget is far larger than the old
threshold-8 lore. So: the static-vs-quadratic claim is structurally true
(guards and code size scale with column count) but does NOT convert into a
large-n time delta at any width probed. The durable channel-1 deltas remain
exactly: short-trip performance (guards can't amortize), code size (17x at 8
cols), and the W1 obvious-shape-is-fast property.
