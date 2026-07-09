# Do regions + effects beat best-effort SAFE Rust? Adversarial study (2026-07-08)

**Make-or-break R0 evidence. Headline: mostly NO.** Four kernels — the canonical cases where
regions/effects should shine — with a safe-Rust adversary (rayon/bumpalo/iterators/fat-LTO, no
`unsafe`) trying to reach the xlang-equivalent ceiling. Apple M4, rustc 1.91.1 `-C opt-level=3
lto=fat`, clang 21. Method: natural safe Rust baseline → best-effort safe Rust → xlang-equivalent
ceiling; the residual best-effort-safe can't close = the structural win.

| Kernel | Verdict | best-effort safe Rust vs xlang ceiling |
|---|---|---|
| Interprocedural pure-call LICM/CSE/DCE (across dylib) | **AUTOMATION** | 0% — bit-identical asm |
| Region bulk-free vs per-node Drop | **AUTOMATION** | 0% — safe u32-index pool ties; bumpalo within 21% |
| Pure-stage fusion / deforestation | **AUTOMATION** | safe iterator chain **15% faster** than hand-fused |
| **Parallel scatter, disjoint permutation** | **STRUCTURAL** | **1.13×–1.51×** (compute→memory bound) |

**The one real structural win:** a provably-disjoint parallel scatter `out[perm[i]] = f(in[i])`
(`perm` a bijection). Safe Rust can't express it — the borrow checker can't prove `perm`
injective — so every safe reformulation (range-ownership P-fold rescans, bucketed double-writes,
serial inverse) pays redundant memory traffic. Residual **~1.5× memory-bound, ~1.1× compute-bound**;
one `unsafe` store closes it entirely. Narrow domain (scatter/gather numerics: particle reorder,
radix, spatial sort). Real, durable, modest.

**The eye-popping numbers are footguns, not ceilings.** 12,500× (dylib call-in-loop), 52,000×
(Drop vs region free), 3–5× (fusion, parallelization) are all NAIVE-code-vs-optimized gaps that
best-effort *safe* Rust also captures — not gaps safe Rust structurally fails to reach. E.g. the
52,000× free-cost becomes 43ms (safe u32-pool) vs 52ms (safe bumpalo) once the dev uses an arena.

**Bottom line.** No general "regions+effects beat safe Rust on performance" claim survives. Three
of four probes are parity-with-effort; the lone structural residual is 1.1–1.5× on a niche kernel.
Combined with the already-conceded weak W1 (LLMs write Rust well) and marginal W3 (safe Rust + CI),
**the raw-performance case for xlang as a general-purpose Rust competitor is not supported by this
evidence.**

**The one surviving thread** is *shift-left/automation*, which is the project's actual stated goal
(AI codegen + performance): xlang makes the *naive* form the *fast* form (auto-hoist across a dylib
LTO can't reach; auto bulk-free from a plain owned tree; auto-fuse from separate signatures), so
code reaching for the obvious shape doesn't eat the 3–12,500× footguns. Real and AI-relevant — but
undercut by "a good model writes the fast Rust form anyway," and only evaluable with the unbuilt M3
AI-codegen harness (does an AI writer actually produce correct+fast code more reliably in xlang
than in Rust?). That is a different, narrower thesis than "beat Rust on speed."

Evidence: `scratchpad/reperf/{interproc-effects,auto-parallel-regions,region-bulk-free,pure-fusion}/`.
