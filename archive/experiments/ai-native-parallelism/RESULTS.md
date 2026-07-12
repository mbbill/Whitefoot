# AI-Native Parallelism Results

Status: E0 complete, E1 first measurement complete; not decision-ready.

## Completed Locally

- Defined the investigation split from old automatic parallelization: AI proposes
  plans and obligations; the compiler verifies, guards, and measures.
- Added a draft bundle schema: `parallel-plan-bundle.schema.json`.
- Added a manual seed bundle for the known structural scatter case:
  `examples/scatter_permutation.bundle.json`.
- Added a guarded scatter variant to the existing benchmark:
  `../regions-effects-vs-safe-rust/code/auto-parallel-regions/src/main.rs`.
- Measured conservative per-dispatch `O(n)` permutation verification cost on the
  existing 16.8M-element scatter benchmark.
- Measured a reusable stamp-table verifier; it was slower than the simple
  byte-table verifier on this workload.

## Current Working Hypothesis

The first viable target is not general automatic parallelization. It is verified
parallel plan generation:

1. The sequential spec remains the semantic fallback.
2. AI-authored plans are untrusted.
3. Static proofs or runtime guards promote candidates from proposed to usable.
4. Measurement chooses among safe candidates when the cost model is uncertain.

## E1: Guarded Scatter Measurement

Commands:

```
cd experiments/regions-effects-vs-safe-rust/code/auto-parallel-regions
cargo run --release -- 16777216 7 8
cargo run --release -- 16777216 5 32
cargo run --release -- 16777216 5 64
```

Machine: same local Apple M4 environment as the earlier scatter study. Values
below are medians in milliseconds from sequential runs. `best safe` is the best
of the safe Rust adversaries in the benchmark. `direct` is the unsafe/xlang
ceiling scatter. `guarded` verifies the permutation with an `O(n)` byte-table
pass on every dispatch and then runs direct scatter. `stamp guarded` uses a
reusable `u32` stamp table to avoid per-dispatch allocation/zeroing.

| n | work | best safe | direct | guard only | guarded | best safe / guarded | stamp guard | stamp guarded | best safe / stamp guarded |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 16,777,216 | 8 | 54.432 | 35.513 | 15.659 | 53.613 | 1.02x | 65.326 | 103.012 | 0.53x |
| 16,777,216 | 32 | 153.530 | 113.409 | 17.130 | 131.189 | 1.17x | 65.247 | 180.065 | 0.85x |
| 16,777,216 | 64 | 368.022 | 328.101 | 35.274 | 366.323 | 1.00x | 76.194 | 425.078 | 0.87x |

### Interpretation

The conservative "verify every dispatch" design weakens the AI-native plan
story sharply.

- The unguarded structural residual still exists: direct scatter beats best safe
  by 1.53x at work=8, 1.35x at work=32, and 1.12x at work=64 in these runs.
- Paying the guard every time mostly erases the memory-bound win: work=8 drops
  to only 1.02x over best safe.
- The only meaningful guarded win in this pass is work=32, at 1.17x.
- Work=64 is parity at 1.00x.
- The reusable stamp-table guard is worse here. It avoids allocation/zeroing but
  writes a larger random side table (`u32` instead of `u8`), so it loses on
  memory traffic.

This does not kill the branch, but it narrows it. AI-native parallelism cannot
depend on an expensive proof pass before every kernel dispatch. It needs at
least one of:

1. static proof from construction,
2. cached proof for immutable or versioned data,
3. cheaper trusted guard templates than the two measured here,
4. workloads where the guarded kernel runs many times per proof,
5. a larger residual than the current scatter benchmark shows.

Without one of those, verified direct scatter is not a project-level pillar; it
is at most a narrow optimization for amortized permutation-heavy kernels.

### Amortized Proof Projection

If the permutation proof can be cached and reused for K kernel dispatches, the
per-use cost is approximately:

```
direct_scatter + guard_only / K
```

Using the measured byte-table guard:

| work | K=1 | K=2 | K=4 | K=8 | K=16 |
|---:|---:|---:|---:|---:|---:|
| 8 | 1.06x | 1.26x | 1.38x | 1.45x | 1.49x |
| 32 | 1.18x | 1.26x | 1.30x | 1.33x | 1.34x |
| 64 | 1.01x | 1.06x | 1.09x | 1.11x | 1.11x |

Values are speedup over the best safe Rust adversary. This makes the next gate
clear: the AI-native plan path needs a verified way to cache or statically derive
the permutation proof. Without amortization, the memory-bound headline drops
from 1.53x unguarded to roughly parity.

## Immediate Unknowns

1. **Proof amortization:** The first guard-cost measurement shows per-dispatch
   verification is too expensive for the memory-bound headline. We need a model
   for cached/static proofs and invalidation.
2. **Plan size:** The seed bundle is much larger than the source loop. We need a
   prompt/source-size budget before claiming this avoids DPJ's annotation burden.
3. **Verifier scope:** The current schema separates facts and guards, but does
   not yet prove that guard code itself is correct. A real verifier must either
   synthesize trusted guards or check guard implementations against a small set
   of known guard templates.
4. **AI generation:** No model has generated one of these bundles yet. Manual
   seed bundles prove expressibility only.

## Next Measurement

Measure amortized proof modes for the existing scatter benchmark:

- one-time verification followed by K guarded uses for K in {1, 2, 4, 8, 16},
- versioned/cached proof object with invalidation on `perm` mutation,
- bitset vs stamp-vector verifier variants,
- smaller/larger `n` to find the crossover point.

The key number is:

```
amortized_guarded_direct_scatter / best_safe_range_owned
```

If the ratio is not meaningfully below `1.0` after realistic amortization, the
AI-native plan approach loses its strongest local example before model trials.
