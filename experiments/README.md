# xlang performance investigation — audit package

**Purpose.** This folder is a self-contained record of the empirical investigation into a single
make-or-break question for the xlang (XL) project:

> **Does XL have a durable performance (or other) advantage over Rust that justifies building it
> as a new systems language?**

It is written to be **audited by a third party**. The conclusion is largely **negative**, so the
audit's job is to check whether that negative conclusion is *sound and fairly reached* — i.e. that
the comparisons were fair, the safe-Rust baselines were genuinely best-effort, the "wins" were not
inflated, and the reasoning holds. Every claim below points to reproducible code or primary
literature so you can verify it yourself. **§6 (Audit guide) lists exactly what to scrutinize.**

Provenance: all work 2026-07-08, machine **Apple M4 (arm64), macOS**; run by AI subagents under a
skeptic/falsify-first mandate. Git commits: `38f46ea` (study 1), `ad8dbe0` (study 2), `979e3e2`
(study 3). Raw research notes also in `../optimizer-language-research/notes/`.

---

## 0. Context an auditor needs

- **XL** is a research language: ownership (`own` / `&'r` / `&uniq 'r`), explicit **regions** (`'r`),
  and an **effect system** (per-function rows: `pure | reads('r) | writes('r) | allocates | traps`).
  Its stated goal (project constitution) is *"AI codegen + performance."* Its **R0 rule** requires
  every design decision to name a delta over Rust, on **any** of P0 (performance), W3 (cheat-proof
  safety), or W1 (AI-writability).
- **XL has no optimizing compiler yet.** The current bootstrap (`prototype/democ`) emits LLVM IR and
  defers to `clang -O2`. So no study here runs "XL" directly. Where a study needs the code XL's
  facts *would* produce, it uses an honest **proxy**: `restrict`-annotated / `unsafe` Rust or C that
  carries the same facts (noalias, no bounds check, disjointness). This proxy assumption is the
  single most important thing for an auditor to accept or challenge — see §6.
- **The comparison is as fair as possible:** XL and C/C++ go through the **same** Apple `clang 21`
  binary (`/usr/bin/clang`, LLVM 21) that democ shells out to. Rust is `rustc 1.91.1` (bundled LLVM
  21.1.2). Same LLVM major for all three.

---

## 1. Study 1 — codegen vs C / C++ / Rust  (`codegen-vs-rust-c/`)

**Hypothesis tested:** does XL's automatic fact-emission (noalias/readonly from `&uniq`/`&`, no-UB
arithmetic) make the shared LLVM backend produce *faster* code than C/Rust?

**Kernels:** (A) splitmix64 mixing, 2·10⁸ iters, xor-accumulated (pure scalar). (B) `accumulate`
through `&uniq`/`&` pointers — the noalias case.

**Results (median):**
- **A — full parity.** Hot loop is **byte-identical** across xlang/C/C++/Rust (9 register-only
  arm64 instrs). Runtime: xlang 3.02, C 3.02, C++ 3.06, Rust 3.11 ns/iter. democ's naive
  alloca-in-loop IR is fully cleaned by the shared backend — no penalty, no win.
- **B — the noalias fact:** with the fact, LLVM collapses an affine reduction O(n)→O(1) (`madd`):
  naive C **60.88 ms** → `restrict`-C / xlang **2.75 / 2.92 ms** (≈ **22×**). But a non-affine body
  is register-latency-bound so the codegen win **doesn't show in wall-clock** (all ~0.93 ns/iter).

**Verdict.** The 22× is over **naive C** (no `restrict`, which is unusual). It is **not** a win over
Rust: Rust's everyday `&mut`/`&` already emit noalias, so Rust gets the same O(1). **Parity with
Rust; a win only over un-annotated C.** Full write-up: `codegen-vs-rust-c/SUMMARY.md`; code + `-O2`
assembly + timing harness in that folder.

---

## 2. Study 2 — regions+effects vs best-effort SAFE Rust  (`regions-effects-vs-safe-rust/`)

**Hypothesis tested:** do regions+effects give XL a *structural* perf win over **safe** Rust — a gap
that best-effort safe Rust (rayon, bumpalo, iterators, fat-LTO; **no `unsafe`**) **cannot** close?
Method per kernel: natural safe Rust → best-effort safe Rust → XL-equivalent ceiling (the transform
XL's facts would produce; `unsafe`-Rust/C as the cost floor). Residual best-effort-safe can't close
= the structural win.

| Kernel | Verdict | best-effort safe Rust vs XL ceiling |
|---|---|---|
| Interprocedural pure-call hoist (across dylib) | **AUTOMATION** | **0% — bit-identical asm** |
| Region bulk-free vs per-node `Drop` | **AUTOMATION** | **0%** (safe u32-index pool = ceiling); bumpalo within 21% |
| Pure-stage fusion / deforestation | **AUTOMATION** | safe Rust **15% faster** (iterator chain) |
| **Parallel scatter, disjoint permutation** | **STRUCTURAL** | **1.13×–1.51×** (compute→memory bound) |

**Verdict.** 3 of 4 collapse to **automation** — safe Rust reaches the ceiling exactly. The
eye-popping corpus numbers (12,500× dylib call-in-loop, 52,000× Drop-vs-region-free, 3–5×) are all
**naive-code footguns that safe Rust also fixes**, not ceilings it can't reach (e.g. the 52,000×
free-cost becomes 43 ms safe-u32-pool vs 52 ms safe-bumpalo). The **one** structural residual:
provably-disjoint parallel scatter `out[perm[i]]=f(in[i])` — safe Rust can't prove `perm` injective,
so it pays redundant memory traffic; worth **~1.5× memory-bound, ~1.1× compute-bound**, and **one
`unsafe` store closes it**. Full raw results (all 4 kernels + synthesis): `RESULTS.md`; kernel code:
`code/`; summary: `SUMMARY.md`.

---

## 3. Study 3 — auto-parallelism feasibility  (`auto-parallelism-feasibility/`)

**Hypothesis tested:** can XL's effects+regions deliver **safe automatic parallelism** ("write
sequential, get parallel") that beats Rust's structurally-manual model (threads/Send-Sync/rayon;
async ≠ parallelism)? Method: 4 prior-art surveys (DPJ/effect-determinism, region-parallel
languages, data-parallel/auto-vectorize, Rust) → adversarial feasibility verdict.

**Verdict: NO on the strong claim; narrow PARTIAL yes on a scoped feature; do not bet the project.**
- Regions+effects prove **task**-level independence (Bernstein over regions), but the volume of
  parallelism is in **loops**, where region granularity is **too coarse** — a data-parallel loop is
  `writes('out)` every iteration → the effect system says *conflict*. The fact that parallelizes it
  (`perm` injective) is **not expressible in XL's effect vocabulary** — it needs index-parameterized
  regions / injectivity, the fine machinery that **broke DPJ's annotation budget**. XL's own scatter
  win **isn't provable from effect rows alone.**
- No type system supplies a **cost model / granularity** (the part that defeated auto-parallelizers
  for 40 years); HELIX shows a perfect disjointness oracle exposes only modest parallelism because
  real code is genuinely sequential.
- **Adoption:** DPJ is XL's near-twin (region+effect, sound, deterministic, inference-assisted) and
  got ~zero uptake; its creator disavowed the effect-annotation layer (Bocchino, WoDet 2013). rayon
  (`.par_iter()`, one token) already **ties** XL's ceiling.
- **Survivor:** automate the *safety* of programmer-exposed data-parallelism (no `unsafe`/Send-Sync
  ceremony) on the injective-scatter band — the ISPC/Futhark shape, ~1.1–1.5×. Real, durable, niche.

Full verdict + the 4 prior-art surveys with citations: `RESULTS.md`; summary: `SUMMARY.md`.

---

## 4. Consolidated conclusion

Three independent, adversarial, falsification-first studies converge:

> **XL ties Rust on performance. The only structural residual found anywhere is a single ~1.1–1.5×
> on injective-scatter kernels (closable in Rust with one `unsafe` store). Everything else is
> automation / ergonomics — valuable for an AI-codegen target, but not a raw-performance capability
> safe Rust structurally lacks.**

Combined with the two non-performance pillars (independently judged, not in these studies): **W1**
(AI-writability) is weak and declining — LLMs already write Rust well, and a new language carries an
unfamiliarity/context tax; **W3** (safety) is marginal — `#![forbid(unsafe_code)]` + CI gives
memory-safe Rust. **On this evidence, the performance case for XL as a general-purpose Rust
competitor is not supported.**

**One untested survivor** (not covered by these studies): the *distributional / AI-codegen* thesis —
"XL lands real/AI-written code closer to optimal than Rust does, because it removes footgun classes
and forces the fast form." It is congruent with the stated goal, but it is unproven, it cuts both
ways (constraints that force optimality can create their own footguns — Rust's borrow-checker →
defensive-`.clone()` is the proof), and it requires an AI-codegen (M3) harness to evaluate.

---

## 5. Reproduce

- **Study 1:** see `codegen-vs-rust-c/README.md`. xlang: `python3 ../../prototype/democ/democ.py
  xl/kernelA.xl` → `.ll`, then `/usr/bin/clang -O2`. C/C++: `/usr/bin/clang[++] -O2`. Rust:
  `rustc -C opt-level=3`. Timing: `time_it.py`. **Use `/usr/bin/clang`** (the PATH `clang` here is a
  wasm cross-compiler).
- **Study 2:** kernel sources in `regions-effects-vs-safe-rust/code/` (per-kernel subdirs). Build
  the Rust/C variants at `-C opt-level=3 -C lto=fat` / `clang -O2`, link the `accumulate`/kernel
  object against the shared driver, run ≥7×, take the median. Numbers + method in `RESULTS.md`.
- **Study 3:** literature/feasibility — no code. Verify the cited primary sources (DPJ, HELIX,
  rayon SPAA'24, Halide/Futhark/ISPC) in `RESULTS.md`.

---

## 6. Audit guide — what to scrutinize

The conclusion is negative, so the highest-value audit checks are the ones that could **overturn**
it (find a real, un-inflated Rust-beating win) or **confirm** it:

1. **The proxy assumption (most important).** Studies 1–2 use `unsafe`-Rust / `restrict`-C as a
   stand-in for "what XL's facts would compile to." Is that fair? Could a real XL optimizer do
   *better* than the proxy (e.g. exploit region/effect facts LLVM can't get even from `restrict`)?
   If yes, some AUTOMATION verdicts might understate XL. Challenge the proxies.
2. **Was the safe-Rust adversary genuinely best-effort?** Study 2's verdicts hinge on safe Rust
   *reaching* the ceiling. Inspect `code/` — did it use the strongest safe idiom (iterators,
   `chunks_exact`, rayon, bumpalo, LTO)? A weaker adversary would fake a "structural" win.
3. **Naive-vs-optimized honesty.** The 12,500× / 52,000× / 3–5× numbers are labeled *footguns safe
   Rust also fixes*, not ceilings. Verify that framing — an inflated write-up would present these as
   "XL beats Rust."
4. **The scatter STRUCTURAL verdict.** It is labeled structural yet closable with one `unsafe` store,
   and most of its raw speedup is parallelization safe rayon recovers. Is calling the residual
   1.1–1.5× "structural" fair, or should it be "automation + a small injectivity residual"?
5. **Study 3's technical claim** that XL's scatter win is *not provable from effect rows alone*
   (needs injectivity/index-parameterized regions). This is load-bearing — check it against the
   effect-row semantics in `../spec/`.
6. **Citations in Study 3** (DPJ Table 3 ~10.7%/≤22.6% annotation lines; Bocchino WoDet 2013 pivot;
   HELIX limits; rayon SPAA 2024). Verify against the primary PDFs.
7. **Coverage.** These are a handful of kernels + a literature review, single machine, AI-run. The
   conclusion is *"not supported by this evidence,"* not *"proven impossible."* Propose additional
   kernels/angles that could exhibit a durable win (the strongest untested candidate is the
   distributional/AI-codegen thesis in §4, which needs an M3 harness, not a kernel).
