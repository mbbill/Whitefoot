# THE PLAN (consolidated 2026-07-10)

The single current-state document: what this project believes, what it has
proven, and what it does next. Supersedes DECISION_SPRINT.md's phase plan
(kept as history). Law lives in CONSTITUTION.md; rulings in
`optimizer-language-research/notes/user-directives.md`; the lab notebook in
`optimizer-language-research/implementation/decision-gates.md`. This file is
the map, re-derived from those sources; on conflict, they win in that order.

## 1. What xlang is

A systems language for AI-written, human-approved code (D0a). Entire bug
classes are unrepresentable (memory corruption, races, silent overflow,
uninitialized reads — T1/T2/D1) via an ownership/region/effects checker with
no unsafe escape (W3). The checker's proofs double as optimizer facts (P0).
One canonical spelling per program (FORM-1/2). The spec fits in a context
window (D2 — still binding per owner: context is the resource that survives
model scaling). Design patterns are a closed, taught catalog, at architecture
scale as at statement scale (D6, PATTERNS.md).

## 2. Standing rulings (digest — full text in user-directives.md)

- D0a AI-authored, human-approved. D1a simple checker: reject-when-unsure.
- D2a spec compactness binds; program verbosity does not.
- D3 never copy Rust by default; lexicon names checked invariants.
- D4 rewrite-first, FFI-narrow.
- D5 model-tier writer sprint deprioritized (models improve too fast to gate
  on today's weak models; harness stays shelf-ready).
- D6 pattern doctrine: catalog must be complete AND efficient; taught, not
  discovered.
- D7/D7a impact target: swap-in everyday artifacts; headline formula is
  faster + parity + bug-class-unrepresentable + AI-authored. Safe-direction
  constraint: performance/correctness framing only.
- D9 confidence gate: no big compiler investment until (leg B) one real
  program shows a fact-attributable win over best-effort safe Rust AND
  (leg A) the winning pattern is frequent in real corpora.

## 3. Evidence to date — honest ledger

Measured wins (real, replicable, with caveats in each RESULTS.md):
- Floor-raising (binary-trees): the slow design is unrepresentable; ports
  land on the fast shape by construction. The 12x-vs-Box number is a shape
  effect — identical-shape Rust is ~11% faster than us (checked-semantics
  tax, part of which is xlang doing MORE checking than release Rust).
- Checked-law channel: 3.3x over the obvious fold; false laws refuted at
  compile time (the W3 jewel — expert Rust's manual reassociation is an
  unchecked assertion).
- Effect-attr channel: LTO-grade cross-module optimization at per-file build
  cost; guarantee-vs-heuristic; survives truly opaque boundaries.
- Scoped-alias channel: short-trip wins + 17x code size vs guard-versioned
  Rust loops; parity at long trips (runtime checks amortize).
- Utility ports: wc full counts 2.1x GNU / 2.4x uutils-Rust; base64 1.6x
  GNU/uutils at RFC-identical output; both at full checked semantics.
- Codegen debt retired: OWN-1 Bool-copy amendment -> i1 dataflow -> C/Rust
  parity on the classifier kernel (was 1.6-1.8x behind); 2-variant enums i1.

Measured non-wins (equally load-bearing):
- No real program yet shows a FACT-attributable win over best-effort safe
  Rust (D9 leg B open). Same-algorithm safe Rust reaches parity wherever the
  algorithm is expressible (chunk-summary wc: parity; binary-trees: Rust
  slightly ahead same-shape).
- facts on/off is neutral on single-buffer byte kernels (wc, base64) — the
  channels need aliasing/effect pressure to matter.
- uutils' -l (bytecount SIMD) beats our naive scan 2-3x; GNU memchr too.
  Hand-tuned kernels remain ahead of our autovectorized naive shapes.

## 4. The bets, ranked (what we do next)

1. **Proof-elided checks (OP-4 tier)** — the D9-decisive candidate. Design
   first, not code first: a precondition surface (FN-1-style declared
   bounds/invariants the checker proves at call sites) so ELISION covers what
   LLVM cannot derive locally; writers keep `.trap` semantics and the
   compiler earns the speed. Evidence already filed: provably-safe trap
   reductions stay scalar; ~18 surviving bounds branches block base64
   vectorization (controlled elision experiment designed in gates, no gain
   claimed until run). This is also the principled answer to "bad code
   exists": never push writers to `.wrap`. STATUS 2026-07-10: ceiling
   MEASURED — 1.7x on base64 (scalar; no SIMD unlock), design card in gates;
   PROOF-1 structural prover is the next build unit, PROOF-2 preconditions
   need owner ratification as spec surface.
2. **Leg-A frequency study** — never run, cheap, decision-relevant: how often
   do the channel patterns (alias-guard versioning, opaque hot calls, manual
   reassociation idioms) occur in real Rust corpora. Directly answers the
   niche-vs-common doubt.
3. **Channel 4: blessed interpreter dispatch** (carded): lower naive
   loop+match to threaded/musttail dispatch — structural delta over Rust,
   parity with expert C from the obvious shape. Eventual benchmark: the
   owner's own engine (Silverfir), owner-refereed.
4. **Coreutils ladder** (D7a): wc, base64 done; next utilities need the I/O
   frame (first D4 FFI instance) and chunked-driver parity. The AI-authorship
   headline runs through the shelved trial harness when the time comes.

## 5. The pivot clause (pre-registered)

If bets 1 and 2 both come back thin, the honest conclusion is xlang ~= Rust
on raw speed for expressible algorithms, and the pitch formally becomes:
**C-class speed with everything checked, everything reproducible, written by
AI under a checker that makes cheating unrepresentable** — safety-at-parity
plus floor-raising plus build economics, not "faster than Rust." R0's
fallback options (verified-facts frontend; linted-Rust) get a fair hearing.
That outcome changes the pitch, not the honesty; it is a finding, not a
failure.

## 6. Standing process rules

- Durability: commit + gates line per completed step (rewind-proof).
- Fact channels get adversarial review BEFORE ship (the willreturn lesson:
  green checks missed a real unsoundness; the refutation attempt caught it).
- Agent tiering: sonnet floor for mechanical work, opus for most fan-outs,
  top tier only for subtle soundness/design (owner ruling).
- Claims discipline: verifiers before headlines; report the number that
  survives adversarial review, with the caveat attached.
- Safe-direction framing: performance and correctness language only.
