# Study 2 — Regions+Effects vs best-effort SAFE Rust: RAW RESULTS

Adversarial workflow output (4 kernels + synthesis). Verbatim agent returns.


## Kernel: interproc-effects — LICM/CSE/DCE across a non-inlined effectful (pure) call boundary

**Verdict: AUTOMATION** — residual: parity (0%) — best-effort safe Rust (0.297 ms) is literally the same code and same asm as the xlang ceiling (0.297 ms), bit-identical output. No structural residue.

**Workload:** Tone-map style grid op: out[i] = compute_gain(cfg) * input[i] over N=2,000,000 elements. compute_gain is an expensive (~1.9µs), PURE (reads-only, no writes/alloc/trap) fixed-point iteration, marked #[inline(never)] to model a real medium/separately-compiled function. Its argument is loop-invariant, so the call is a textbook hoist (LICM) target. Tested at two boundaries: (a) same rlib crate with fat LTO, (b) a true cdylib dynamic boundary. All 4 versions verified bit-identical (checksum 2514791.3154703910; gain equal across both paths).

**Numbers:**
```
Apple M4, rustc 1.91.1, -O3 lto=fat codegen-units=1, median of 9 (two runs):
1 baseline_samecrate (call in loop, rlib):   0.865 / 0.392 ms  <- LLVM auto-hoisted it
2 hoisted == XLANG CEILING (call once):      0.664 / 0.331 ms
4 baseline_dylib (call in loop, cdylib):     3802.7 / 3856.4 ms  <- opaque, 2M calls, no vectorize
5 hoisted_dylib = BEST-EFFORT SAFE RUST:     0.305 / 0.297 ms  <- one-line manual hoist
Magnitude automated across the dylib footgun: 3856 ms -> 0.30 ms = ~12,500x.
Best-effort safe Rust (0.30 ms) == xlang ceiling (0.30 ms), bit-identical.
```

**Why / mechanism:** see above

**Honest note:** Best-effort safe Rust had to write ONE line: `let g = compute_gain_dyn(cfg); for i {...}` — hoisting a manifestly loop-invariant call before the loop. That hoist needs zero unsafe (only the FFI call itself is unsafe, which is inherent to calling any C symbol, not to the optimization). It fully recovered both the hoist and the auto-vectorization the opaque call had blocked. Two honest caveats that keep this from being a P0 structural win: (1) Within a single crate with LTO, rustc's own readnone/memory(none) attribute inference ALREADY auto-hoists and vectorizes the #[inline(never)] pure call (baseline 0.39ms ~= hoisted 0.33ms) — there is nothing to win at all inside a crate. (2) The dramatic 12,500x number (3856ms->0.30ms) is the size of a NAIVE-CODE FOOTGUN across a dylib, not a ceiling safe Rust can't reach — safe Rust reaches the exact same ceiling. LICM/CSE/DCE of pure calls are source-level transforms a programmer can always reproduce by hand, so this kernel is genuine convenience/safety-net value for AI-generated code (it fixes the footgun automatically from the signature, even across a dylib where LTO can't) but NOT a structural performance advantage over an expert writing safe Rust. Verdict: AUTOMATION, residual gap 0%.


## Kernel: auto-parallel-regions: scatter-by-permutation (out[perm[i]] = transform(input[i]), perm a bijection => writes provably disjoint => embarrassingly parallel, but disjointness unprovable to the borrow checker)

**Verdict: STRUCTURAL** — residual: Residual best-safe vs xlang-ceiling = 13%-51% (safe Rust 1.13x-1.51x slower; compute-bound -> memory-bound). Does NOT close: three distinct safe reformulations (gather+inverse, range-ownership, bucketed 2-phase) all stayed behind the unsafe parallel scatter. Modest and regime-dependent, not a 2x+ win.

**Workload:** Reorder-and-transform: 16.8M f64 particles, a precomputed random permutation `perm` (spatial-sort order), apply a per-element FMA transform and scatter each result to out[perm[i]]. `work` knob sweeps arithmetic intensity (8=memory-bound random-scatter, 64=compute-bound). n=1<<24, Apple M4 (10 cores), rustc 1.91 -C opt-level=3, fat LTO, codegen-units=1. All versions verified bit-identical to the sequential reference before timing; outputs black_box'd.

**Numbers:**
```
median ms, n=16.8M, 10 threads. Columns: V1 baseline seq-scatter | best safe adversary (V2c range-ownership) | xlang ceiling (unsafe par-scatter) [+ other safe attempts].
work=8  (mem-bound):   67.8 | 51.1 | 33.7   [V2 gather+inv 118.9, V2b gather-only 54.5, V2d bucketed 55.4]   safe/ceiling 1.51x, baseline/ceiling 2.01x
work=32:              552.5 |146.2 |108.4   [gather+inv 273.0, bucketed 153.2]                                 safe/ceiling 1.35x, baseline/ceiling 5.10x
work=64 (compute):   1712.1 |361.1 |318.7   [gather+inv 503.1, bucketed 366.7]                                 safe/ceiling 1.13x, baseline/ceiling 5.37x
Ceiling = rayon into_par_iter over input indices scattering through a Send/Sync *mut (one unsafe store; the write xlang would auto-emit from the permutation-disjoint fact).
```

**Why / mechanism:** Safe Rust cannot write out[perm[i]] in a parallel loop (borrow checker can't prove perm injective), so every safe-parallel form pays redundant memory traffic to route work to disjoint owned regions: V2c has each thread OWN a contiguous output range (par_chunks_mut) and re-scan ALL of perm to filter its share => (P-1)*n extra streaming reads of perm (~9 extra passes over 64MB at work=8, ~11ms of the gap); V2d buckets to avoid the rescan but then writes the payload twice + allocates P*P vecs (came out no better); V2 builds the inverse permutation but that build is ITSELF a serial random scatter (~66ms, unparallelizable in safe Rust) and loses outright. The ceiling does exactly n sequential reads + n random writes + n transforms, no redundancy. Assembly: `transform` inlines to an identical FMA chain (fmla/fmul) in every version; the safe indexed writes carry bounds-check panic branches (core::panicking::panic) while the ceiling store is a bare `str` through the raw pointer. The redundant traffic is the literal price of the missing permutation-disjointness fact, so the residual shrinks from 1.51x (memory-bound, redundant scans dominate) to 1.13x (compute-bound, transform amortizes them).

**Honest note:** Most of the raw speedup here is AUTOMATION, not structural: the 2.0x-5.4x baseline->ceiling gain is the parallelization itself, and safe rayon RECOVERS the bulk of it via par_chunks_mut output-range ownership with zero unsafe -- so "safe Rust can't parallelize this at all" is FALSE. What safe Rust genuinely cannot recover is the last 1.13x-1.51x, because it must do redundant memory work (P-fold rescans, or double-writes, or a serial inverse build) precisely to sidestep the unprovable direct scatter; every safe form I wrote paid that tax and none reached the ceiling. I label it STRUCTURAL because the defining test -- does the residual close in safe Rust -- is no, and it reproduces across regimes; but I am not rounding it up: the honest headline is "safe Rust ties the parallelization and lands within 13-51% of the region/effect ceiling," strongest (51%) only in the pure memory-bound random-scatter regime and nearly parity (13%) once the kernel is compute-bound. An expert who reaches for `unsafe`/get_unchecked closes it entirely (the ceiling IS that unsafe code). So: a real but modest structural win for xlang's permutation-disjointness fact, mostly a convenience/automation story otherwise.


## Kernel: region-bulk-free — region bulk allocation/free vs per-object Drop

**Verdict: AUTOMATION** — residual: Parity (0%) against best-effort safe Rust: the xlang ceiling (Vec<Expr> u32-index pool) is itself 100% safe Rust, so safe Rust reaches the ceiling exactly. With the far-less-invasive bumpalo drop-in (keeps the pointer-tree shape, just bulk-resets), safe Rust is 1.21x — i.e. 21% slower than the ceiling, still fully safe. Either way, nothing here requires unsafe.

**Workload:** Compiler-front-end pattern: per batch, build 1000 short-lived binary expression ASTs, each a full tree of depth 12 (8191 small owned nodes; ~8.19M nodes/batch), fold-evaluate each tree (touches every node, defeats DCE), then discard it. All three versions produce a bit-identical i64 checksum (0x3a17a89159cd899), verified before timing. Apple M4, rustc 1.91.1, opt-level=3 + fat LTO + codegen-units=1 + panic=abort. Median of 15 runs, 3 warmup, black_box on outputs.

**Numbers:**
```
FULL BATCH (build+eval+free), median ms, 8.19M nodes/batch:
  1 BASELINE  Box<Expr> / per-node Drop ...... 154.67 ms   (min 151.6)
  2 SAFE      bumpalo arena, bulk reset/free ..  52.67 ms   (min 51.4)  [best-effort safe adversary, keeps tree shape]
  3 CEILING   Vec<Expr> u32-index pool (SAFE) ..  43.36 ms   (min 43.2)  [xlang region+effect codegen; also pure safe Rust]
  (+floor) the ceiling IS the unsafe-free floor — no unsafe needed; raw bump would match.
  ratios: baseline/ceiling 3.57x | bumpalo/ceiling 1.21x | baseline/bumpalo 2.94x

ISOLATED TEARDOWN of 8.19M live nodes (build untimed, free only), median ms:
  Box/Drop per-node free ......... 82.881 ms
  region bulk free (Vec drop) ....  0.0016 ms
  free-cost ratio Box/region = ~52,000x
  (teardown alone is 54% of the baseline's 154.7 ms total; the region pays ~0 for it)
```

**Why / mechanism:** Assembly confirms the mechanism. free_region compiles to: load ptr, `len<<4` (16-byte Copy node), tail-call `__rust_dealloc` — ONE deallocation, no loop, no destructor, because Vec<Expr> with `Expr: Copy` has zero drop glue. free_box compiles to a per-node recursive teardown (branch + free per node), calling free() 8191x per tree × 1000 trees on heap-scattered, cache-missing nodes → 82.9 ms. Allocation is asymmetric too: Box does malloc-per-node (baseline build+eval ~72 ms vs region ~43 ms), so both alloc and free favor the contiguous pool. The region/effect fact ("nodes are region-local, POD, freed together") is exactly what turns O(n) scattered free()+destructors into one bulk dealloc.

**Honest note:** No unsafe was written anywhere — the ceiling is pure safe Rust, so this is AUTOMATION, not STRUCTURAL, and I refuse to round the eye-popping 52,000x free-cost number up to a structural win. The win over the NAIVE idiomatic baseline is real and large (3.57x end-to-end; its teardown alone is 82.9 ms / 54% of runtime that the region pays ~0 for). But safe Rust recovers all of it: bumpalo/typed-arena give the bulk free with a near-drop-in change (get within 21%), and a u32-index Vec pool matches the ceiling exactly. What the human had to write to close it: abandon `Box<Expr>` and either thread a bumpalo `&'a Bump`/`&'a Expr<'a>` lifetime through build+eval, or convert children to `u32` indices into a shared `Vec<Expr>` passed to every function. Both are established Rust idioms (rustc's typed arenas, la-arena, id-arena, bumpalo, typed-arena) but both are a NON-LOCAL rewrite of the data type and every signature touching it. xlang's actual contribution is ergonomic: from natural `Box`-like source, the region+effect facts let the optimizer emit the arena/bulk-free codegen automatically — the developer who reaches for the natural owned tree eats 3.57x, whereas in xlang the naive form is the fast form. That is an automation/shift-left win for the AI-codegen loop, not a capability safe Rust structurally lacks.


## Kernel: pure-fusion (deforestation / pure-stage pipeline fusion across a Vec-returning boundary)

**Verdict: AUTOMATION** — residual: parity (0%): best-effort safe Rust matches AND slightly beats the xlang-equivalent ceiling (safe iterator chain 14.6 ms vs 17.2 ms manual fused loop = safe Rust 15% faster). No residual gap for xlang to claim.

**Workload:** 6-stage pure element-wise numeric ETL pipeline over N=32M f64 (256 MB/array, memory-bandwidth bound). Cheap FMA-style ops per stage so pass-count and intermediate allocation dominate. Baseline expresses each stage as a separate #[inline(never)] fn returning Vec<f64> (real function/module boundary that iterators do NOT fuse across). All 5 versions produce bit-identical output (xor-of-bits checksum verified before timing). Apple M4, rustc 1.91.1 -C opt-level=3 -C target-cpu=native -C lto=fat -C codegen-units=1. Median of 9 runs, 2 warmups, black_box on input+output.

**Numbers:**
```
A  BASELINE (6 separate Vec-returning fns) .......... median 45.52 ms  (min 44.69)
A2 BASELINE-inl (#[inline] allowed) ................ median 45.19 ms  (min 44.76)
D  DYN-PIPELINE (Vec<Box<dyn Fn>>, opaque boundary)  median 162.51 ms (min 162.28)
B  BEST-EFFORT SAFE (fused iterator chain, no unsafe) median 14.62 ms  (min 14.28)  <-- adversary
C  XLANG CEILING (hand fused single-pass loop) ...... median 17.21 ms  (min 16.77)
Fusion win over naive baseline: 45.52 -> 14.62 = 3.11x. Best-effort safe (B) is FASTER than the xlang-equivalent ceiling (C) by 15%.
```

**Why / mechanism:** Confirmed in asm: BASELINE emits 6 independently NEON-vectorized (.2d) passes plus repeated __rust_alloc/__rust_dealloc for six 256MB intermediates; allowing inlining (A2) changes nothing because the heap Vec allocations survive and LLVM does not fuse across the collect/Vec-push boundary. The safe iterator chain (B) deforests to ONE vectorized pass with a single output allocation — exactly the code xlang's effect rows would license, and it even beats the naive manual index loop (C) because iterator internal iteration eliminates per-element bounds checks. STRUCTURAL impossibility for xlang: loop fusion fundamentally needs each stage BODY at the fusion site to interleave per element; an effect row on a signature only proves fusion is LEGAL, it does not supply the body — so xlang must ALSO inline the bodies to emit the fused loop. Wherever xlang can fuse (bodies available), safe Rust can express the identical fused chain with zero unsafe; wherever bodies are opaque (case D: runtime-dynamic stage selection), xlang's STATIC effect rows equally cannot fuse. So there is no boundary xlang crosses that safe Rust structurally cannot.

**Honest note:** The safe-Rust adversary had to do exactly one thing: rewrite the pipeline of six Vec-returning functions into a single iterator chain input.iter().map(s1).map(s2)...collect() — no unsafe, no arena, no rayon, no restructuring of data types. That trivial rewrite recovered the entire 3.1x fusion win and then edged past a hand-rolled fused loop. Where safe Rust genuinely CANNOT fuse is the dynamic plugin pipeline (Vec<Box<dyn Fn>>, 162 ms, 11x slower than fused) — but that is runtime-chosen stage composition, which xlang's compile-time effect rows cannot fuse either, so it is not an xlang win. Caveat on generality: this is a pure element-wise pipeline; the automation is a one-liner. For a large real codebase with stages scattered across crates compiled without LTO, the fused chain must be authored by hand at one site, so xlang's value is real-but-ergonomic (it fuses automatically from signatures) rather than a performance capability safe Rust lacks. Net: fusion is a big win over NAIVE code, zero win over an expert who types .map().map(), so AUTOMATION not STRUCTURAL — do not bank a P0 perf claim here.


## Cross-kernel synthesis

# CROSS-KERNEL VERDICT: Do REGIONS + EFFECTS give xlang a durable perf win over BEST-EFFORT SAFE Rust?

**Headline: Mostly NO. Four kernels, one modest structural residual.** Three of four probes collapse to AUTOMATION — best-effort safe Rust reaches the xlang ceiling exactly (twice it is bit-identical asm; once it is 15% *faster*). Only the permutation-scatter kernel leaves a residual that safe Rust genuinely cannot close without `unsafe`, and it is a regime-dependent 1.13x–1.51x, not a 2x+ win. The eye-popping numbers in the corpus (12,500x, 52,000x, 5.4x) are all NAIVE-CODE-vs-optimized gaps that safe Rust also captures — not ceilings safe Rust structurally fails to reach.

---

## (1) STRUCTURAL wins — safe Rust cannot recover without `unsafe`

**Exactly one, and it is modest.**

**auto-parallel-regions (permutation-disjoint parallel scatter).** Residual best-safe-Rust vs xlang-ceiling = **13%–51%** (safe Rust 1.13x–1.51x slower), strongest in the memory-bound random-scatter regime (work=8: 51.1 vs 33.7 ms), shrinking to near-parity as the kernel becomes compute-bound (work=64: 361 vs 319 ms).

- **Why it's real:** safe Rust cannot write `out[perm[i]] = …` in a parallel loop (borrow checker can't prove `perm` injective). Every safe reformulation — range-ownership (P-fold rescans of `perm`), bucketed 2-phase (double-writes), gather+inverse (a *serial* random scatter that loses outright) — pays redundant memory traffic precisely to route work into provably-disjoint owned regions. Three distinct safe rewrites all stayed behind; the residual is the literal price of the missing permutation-disjointness fact.
- **Why not to inflate it:** the *bulk* of the raw speedup (2.0x–5.4x baseline→ceiling) is parallelization, which safe rayon RECOVERS via `par_chunks_mut`. "Safe Rust can't parallelize this" is FALSE. What it can't recover is only the last 13–51%. An expert reaching for `get_unchecked` closes it entirely — the ceiling *is* that unsafe code.
- **Realism:** the workload (reorder-and-transform through a bijection: spatial sorts, sparse scatters, histogram/radix-style permutes) is common in graphics/physics/DB. So the win is real, durable, and applies to a recognizable class — but it is modest and only wide open in the pure memory-bound regime.

---

## (2) AUTOMATION wins — parity with hand-tuned safe Rust; value only at whole-program scale

All three are large wins over the NAIVE idiomatic baseline that best-effort safe Rust **fully recovers with zero `unsafe`**:

| Kernel | Naive baseline → best-effort safe | Safe vs xlang ceiling | What the human had to type |
|---|---|---|---|
| **interproc-effects** (LICM of pure call across dylib) | 3856 ms → 0.30 ms (**~12,500x**) | **0% — bit-identical asm** | ONE line: hoist the loop-invariant call |
| **region-bulk-free** (bulk free vs per-node Drop) | 155 ms → 43 ms (**3.57x**) | **0%** (u32-index Vec pool = ceiling, itself safe); bumpalo drop-in within 21% | non-local rewrite: `Box<Expr>` → arena/`u32`-index pool through every signature |
| **pure-fusion** (deforestation across Vec-returning boundary) | 45.5 ms → 14.6 ms (**3.11x**) | **0% — safe is 15% FASTER** (iterator chain beats hand-fused loop) | one-liner: `.map(s1).map(s2)…collect()` |

Common shape: xlang's region/effect facts let the optimizer emit the good codegen automatically *from natural, naive source*. But at any single fusion/hoist/arena site, an expert reproduces the ceiling with a source-level transform (`.map()` chains, a hoist, an arena type). These are transforms a programmer can always write by hand. **The value is ergonomic/shift-left, not a capability safe Rust lacks** — it pays off only at whole-program scale (many sites, crates compiled without LTO, AI-generated code that reaches for the naive form), where authoring every transform by hand is the cost.

A notable structural NON-win inside this bucket: the fusion probe found that an effect row proves fusion *legal* but doesn't supply the stage *bodies* — xlang must still inline to emit the fused loop. Where bodies are opaque (runtime-dynamic pipeline, 162 ms), xlang's static effect rows can't fuse either. So effects don't cross a boundary safe Rust structurally can't.

---

## (3) Genuine parity / no win

**interproc-effects inside a single crate** and **pure-fusion vs an expert iterator chain** are true parity: within a crate + LTO, rustc's own `memory(none)` attribute inference already auto-hoists and vectorizes the pure `#[inline(never)]` call (0.39 vs 0.33 ms) — nothing to win. The fused iterator chain even edges past the hand-rolled loop (bounds-check elision via internal iteration). rustc is already at or past the ceiling here.

---

## BOTTOM LINE

**Is there a real, durable, worth-building-a-language perf advantage in regions + effects vs best-effort safe Rust? For raw performance alone: weak.** Three of four kernels are parity-with-effort; the one structural residual is 1.13x–1.51x and regime-dependent. That is not a P0 "safe Rust leaves 2x on the table" story. The corpus's own honest labels — three AUTOMATION, one modest STRUCTURAL — should not be rounded up.

**Where the durable advantage does live:**
- **The single strongest demonstrable structural win** is the **permutation-disjoint parallel scatter: ~1.5x (51% residual) in the memory-bound regime**, unreachable in safe Rust because no safe formulation can express a provably-disjoint parallel indexed write. This is the one number to demonstrate, honestly captioned as ~1.5x (memory-bound) degrading to ~1.1x (compute-bound), and fully closable with one `unsafe` store. Its natural domain is scatter/gather-heavy numerics (spatial sort, particle reorder, radix/histogram).
- **Everything else is an AUTOMATION/shift-left case**, whose honest justification is the **AI-codegen loop and whole-program scale**, not a per-kernel ceiling: xlang makes the *naive* form the *fast* form (auto-hoist across a dylib where LTO can't reach; auto bulk-free from a natural owned tree; auto-fuse from separate signatures), so a model — or a dev — who writes the obvious code doesn't eat the 3–12,500x naive-code footguns. That is genuinely valuable and aligns with the project's stated goal (AI codegen + performance), but it is convenience and a safety-net, not a performance capability safe Rust structurally lacks.

**Recommendation:** do not bank a general "regions+effects beat safe Rust on performance" claim. Bank the narrow, true one — *provable disjointness enables a safe parallel scatter that safe Rust cannot express, worth up to ~1.5x on memory-bound reorder kernels* — and frame the rest as automation that removes naive-code footguns automatically from signatures, which is the actual, defensible reason to put these facts in the language for an AI-codegen target.

Kernel evidence dirs: `/private/tmp/claude-501/-Users-bytedance-Dev-xlang/58c81074-5f53-4673-8f02-79dec1070188/scratchpad/reperf/<kernel>/` (interproc-effects, auto-parallel-regions, region-bulk-free, pure-fusion).