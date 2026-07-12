# THE PLAN (consolidated 2026-07-12)

The single current-state document: what this project believes, what it has
proven, and what it does next. Supersedes the DECISION_SPRINT/ROADMAP phase plans
(kept in archive/). Law lives in CONSTITUTION.md; rulings in
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
- D9 leg A now has a **directional, non-gating pilot**, not a population result:
  30 popular source crates and 12 command-line applications. Manual review found
  no current checked-law or scoped-alias win. Three optimized-IR builds found
  hot surviving bounds checks in `comrak` and `inferno`; a single-check
  `comrak` intervention was byte-identical but neutral. The useful signal is a
  family of relational/precondition proofs to test in a real port. D9 remains
  open.
- facts on/off is neutral on single-buffer byte kernels (wc, base64) — the
  channels need aliasing/effect pressure to matter.
- uutils' -l (bytecount SIMD) beats our naive scan 2-3x; GNU memchr too.
  Hand-tuned kernels remain ahead of our autovectorized naive shapes.

## 4. The bets, ranked (what we do next)

1. **Proof-elided checks (OP-4 tier)** — the D9-decisive candidate. Design
   first, not code first: a checked concrete-function `requires` prologue
   computes and verifies a boundary fact once at callee entry, then the
   deterministic prover uses it to elide dominated implicit checks that LLVM
   cannot derive locally. Calls remain legal without caller proof; writers
   keep `.trap` semantics and the compiler earns the speed. Evidence already
   filed: provably-safe trap
   reductions stay scalar; ~18 surviving bounds branches block base64
   vectorization (controlled elision experiment designed in gates, no gain
   claimed until run). This is also the principled answer to "bad code
   exists": never push writers to `.wrap`. STATUS 2026-07-10: ceiling
   MEASURED — 1.7x on base64 (scalar; no SIMD unlock), design card in gates;
   PROOF-1 now proves exact len-guarded indexes, fixed-stride remainder-loop
   offsets/tails, and masked const-array indexes through unsigned widening.
   Its 50-case per-site corpus gates 18 positives, one mixed classification,
   and 31 adversarial/conservative near-misses. On base64 it discharges 15/27
   sites and measures 2.50 -> 2.93 GB/s (1.17x), recovering 36% of the
   perfect-prover time gap; all 12 remaining sites are output-capacity writes.
   PROOF-2 was owner-approved and implemented 2026-07-11 as the concrete-only
   checked `requires { let_stmt* check_stmt }` first slice (FN-8); its semantics
   are selected and its spelling remains R3-provisional. The checked 3:4
   capacity relation plus i=3k/o=4k proof discharges base64 27/27 sites, reaches
   the perfect-index-elision ceiling (77 instructions, one retained entry trap),
   and measures 2.480 -> 4.233 GB/s (1.71x). The combined bounds corpus is 94
   cases: 50 PROOF-1 plus 44 PROOF-2 capacity/lockstep cases, including alias,
   mutation, relation, stride, ordering, and tail adversaries. Contract
   refinement remains deferred; prototype artifact-embedded proof references,
   gated FFI frames, and machine-readable trap reports remain explicit debt.
   FIRST REVIEWED SLICE IMPLEMENTED 2026-07-11: PROOF-2 now derives its
   obligation from the body before independently normalizing `requires`, emits
   deterministic first-missing-fact / first-failed-premise site diagnostics,
   and gates those diagnostics in both facts modes across all 44 cases. The
   reporting path is byte-transparent in 190/190 report/no-report compilations
   over all 95 corpus `.xl` files in both facts modes. CHECKED-AUTOMATION
   BOUNDS-V1 IMPLEMENTED 2026-07-11: the facts-on site report carries complete
   schema-versioned analyzer provenance; the B2 classifier passes proved and
   affirmatively intrinsic-dynamic sites, fails
   missing/mismatched obligations as hard findings, and fails every
   indeterminate state closed. The unfilterable `--promotion` invocation
   dual-pins base64's source/function/digest and checks its whole compilation
   unit (27 automatically accounted, zero findings). Fifteen review-pinned corpus policy
   oracles, including the n27–n33 syntax/alias frontier, plus 21 unit tests
   separate diagnostic regression maturity from bounds-v1 build evaluation.
   This is an authorization-free enforcement slice: approvals remain forbidden
   until per-site dependency-cone identity exists, backend credit and
   explicit-check enforcement are absent,
   and protected external owner review must govern coordinated changes to both
   repository pins. The subgate is not canonical DIAG-2 or complete artifact
   promotion. Guard versioning and variable-output work remain staged by
   `optimizer-language-research/implementation/requires-check-accounting-REVIEW.md`;
   no normative language rule changed.
2. **Leg-A frequency pilot** — complete; stop building analysis infrastructure.
   Raw source heuristics substantially overcounted alias and saturating-law
   opportunities, while optimized `comrak` exposed many surviving checks in
   genuinely hot parser paths. The pilot therefore redirects the next bounded
   experiment toward generalized relational/precondition proof in one real
   workload. It does not clear D9, estimate ecosystem prevalence, or justify
   further frequency-tool investment.
3. **Channel 4: blessed interpreter dispatch** (carded): lower naive
   loop+match to threaded/musttail dispatch — structural delta over Rust,
   parity with expert C from the obvious shape. Eventual benchmark: the
   owner's own engine (Silverfir), owner-refereed.
4. **Coreutils ladder** (D7a): wc, base64 done; next utilities need the I/O
   frame (first D4 FFI instance) and chunked-driver parity. The AI-authorship
   headline runs through the shelved trial harness when the time comes.

STATUS UPDATES (2026-07-12): bet 1's owed adversary ran — assert-up-front
recovers NOTHING (elision is checker-attributable), but expert iterator
restructuring ties on base64's regular shape, so the leg-B decider is QOI
decode (variable-size writes have no iterator escape). Leg A still pending.
THE BUILD TRACK IS ACTIVE: `compiler/` hosts xlc, the production compiler
written in xlang itself (SoA-tape architecture per P2, fixed-capacity
buffers, no generics needed), bootstrapped by prototype/democ as stage 0,
with its own gate (`make -C compiler check`, incl. the self-parse gate).

## 5. The pivot clause (pre-registered)

The pivot remains binding, but the throwaway frequency pilot cannot trigger it:
it was directional and not powered to establish rarity. The pivot requires
either that the bounded leg-B program finds no qualifying win, or that every
qualifying winner's own matching study establishes that mechanism as thin. If
that bar is met, the honest conclusion is xlang ~= Rust
on raw speed for expressible algorithms, and the pitch formally becomes:
**C-class speed with everything checked, everything reproducible, written by
AI under a checker that makes cheating unrepresentable** — safety-at-parity
plus floor-raising plus build economics, not "faster than Rust." R0's fallback
options (verified-facts frontend; linted-Rust) get a fair hearing. That outcome
changes the pitch, not the honesty; it is a finding, not a failure.

## 6. Standing process rules

- Durability: commit + gates line per completed step (rewind-proof).
- Codegen parity is an explicit verification layer: deterministic earned
  properties gate every `make check`; unresolved targets remain visible audits
  until independently verified and promoted (`CODEGEN-PARITY.md`).
- Fact channels get adversarial review BEFORE ship (the willreturn lesson:
  green checks missed a real unsoundness; the refutation attempt caught it).
- Agent tiering: sonnet floor for mechanical work, opus for most fan-outs,
  top tier only for subtle soundness/design (owner ruling).
- Claims discipline: verifiers before headlines; report the number that
  survives adversarial review, with the caveat attached.
- Safe-direction framing: performance and correctness language only.
