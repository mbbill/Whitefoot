# THE PLAN (consolidated 2026-07-15)

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
- D5 broad model-tier writer sprint deprioritized; D9a narrowly requires one
  fixed low-tier run to measure the production default path.
- D6 pattern doctrine: catalog must be complete AND efficient; taught, not
  discovered.
- D7/D7a impact target: swap-in everyday artifacts; headline formula is
  faster + parity + bug-class-unrepresentable + AI-authored. Safe-direction
  constraint: performance/correctness framing only.
- D9a confidence gate: compare one fixed `gpt-5.6-terra`/medium model's first
  correctness-green xlang artifact with one exact, unmodified, commonly used
  shipped Rust library. Freeze before timing; facts-on/off explains the win.
  Expert Rust is ceiling evidence only. Replicate once before a broad claim.

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
- D9a default floor now has two independently locked shipped-library wins from
  first-green `gpt-5.6-terra` trajectories: `percent_decode` is 1.653x
  `percent-encoding` 2.3.2 [1.631, 1.667], and one-shot UTF-8 parsing is 1.098x
  `utf8parse` 0.2.2 [1.085, 1.145]. In both targets facts retain every bounds
  site, so these are default implementation-shape wins, not proof-elision
  wins. This is a replicated floor result, not a universal language claim.
- Codegen debt retired: OWN-1 Bool-copy amendment -> i1 dataflow -> C/Rust
  parity on the classifier kernel (was 1.6-1.8x behind); 2-variant enums i1.

Measured non-wins (equally load-bearing):
- Expert safe Rust reaches parity on regular base64 after iterator
  restructuring. That remains useful ceiling evidence, but D9a no longer uses
  a benchmark-specialized implementation as the primary comparator.
- The old D9 leg-A study is a **directional, non-gating pilot**, not a
  population result:
  30 popular source crates and 12 command-line applications. Manual review found
  no current checked-law or scoped-alias win. Three optimized-IR builds found
  hot surviving bounds checks in `comrak` and `inferno`; a single-check
  `comrak` intervention was byte-identical but neutral. The useful signal is a
  family of relational/precondition proofs to test in a real port. It is now
  context for generalization, not a prerequisite for the primary score.
- PROOF-2 makes facts-on base64 1.71x faster than the same facts-off source by
  discharging 27/27 indexed sites. Expert Rust can express a different shape
  that reaches parity; both facts matter, but they answer different questions.
- uutils' -l (bytecount SIMD) beats our naive scan 2-3x; GNU memchr too.
  Hand-tuned kernels remain ahead of our autovectorized naive shapes.

## 4. Active build track and ranked bets

1. **Systems-performance coverage research** — D15 FRESH-DERIVATION TRACK
   ACTIVE (2026-07-16). The owner redirected the capability research: for
   most systems-programming scenarios, at least one blessed way of writing
   must reach or exceed the performance of the best existing
   implementations; the provided form count n stays small under the binding
   spec-compactness requirement; line-by-line reproduction of existing
   software is explicitly not required; compiler-known forms with
   disciplined trusted internals are readmitted to the answer space. The
   D14 B-Strata-only lock no longer constrains the active derivation; all
   prior candidate artifacts remain historical evidence and falsifiers.

   The first fresh pass is complete: a 9-family/51-scenario demand map,
   four independent complete designs (builtin-maximalist, minimal semantic
   core, evidence-split hybrid, parameterized schemas), twelve hostile
   attack reports, and one cross-design judgment, all durably recorded in
   `optimizer-language-research/implementation/systems-performance-coverage/`.
   Recommendation: a three-tier architecture (narrow language core + sealed
   parameterized taught forms + composition cards, ~14 spec-object families,
   honest budget ~60-75 kernel rules plus a <=40k-token catalog appendix),
   explicitly conditional on gate #1 — a decidable loan/freeze judgment
   plus confined borrow-carrying values (the one kernel gap all four
   designs failed on: entry tokens, cursors, guards, and iterators are
   untypeable under the frozen no-reborrow/no-borrows-in-data rules).
   Preregistered validation ladder M1-M10 with frozen pass/fail bands;
   M1 (paper falsification of the loan judgment, ~1 week) kills or
   confirms the architecture first. Owner decision points: authorize M1;
   pool recycling vs P2 never-recycle re-decision; trap = process abort;
   v1 non-goals; `copy struct`; the D2 budget split. No production change
   is authorized before those decisions.

   STATUS (2026-07-17): all six kernel deltas have completed drafting and
   adversarial review. Gate #1 (loan/freeze, 15 rules) is passed and ratified — the
   loan/freeze judgment plus confined borrow-carrying values landed at
   exactly 15 rules, machine-verified on a 97-program corpus with a 9/9
   mutation harness, hostilely reviewed and repaired; RULES-RATIFIED.md is
   the normative research draft. Owner rulings D16-D18 bind the drafts
   (catalog minimality: 10 sealed kernels; acceptance ledgers; trap = abort;
   generational pools; four v1 non-goals; proof-gated admission; spec budget
   option 2). Kernel-shape dry runs validate seq/table (4/5 in band, two
   wins, AoS layout pinned) and the SPSC queue (exhaustive model check SAFE,
   zero-RMW win confirmed, beats rtrb). Writability round 4 PASSES (70% green
   after one diagnostic-feedback revision, 100% par form choice); the eight
   catalog cards are authored and the always-loaded set is 42.3k of the 48k
   budget. All six kernel deltas (loan/freeze, TAG-1, tbl_clone, byte loads,
   DOM-1, BRAND-1, concurrency) are drafted and adversarially reviewed;
   BRAND-1 and concurrency each took four review rounds, the rest one or two;
   the program-wide review scorecard is ~9 FATAL and ~25 MAJOR findings, every
   one caught by an attacker with a concrete program. Adoption is gated on
   five owner escalations (land CONC-0 as kernel memory-model text; OWN-11
   loop-spawn coverage; ratify AMD-5-carve-out/AMD-7/AMD-8; the clone re-mode;
   the concurrency budget cut) and a separate landing review. Next: the owner
   decisions, then the gated real-compiler integration and per-form safety
   models.

   Historical context of the superseded lock (evidence, not active
   authority): after the completed `B-REVISE` gate, the owner had selected
   `B-STRATA` as the sole capability architecture. That outcome was forced:
   `STRATA-YES` with a normalized minimal core, safe closure of fourteen frozen
   systems-performance demands, hostile safety and erasure review,
   implementable deterministic checking/lowering, and bounded performance and
   resource evidence; or `STRATA-NO` with the irreducible safety, performance,
   interaction-growth, or implementation reason it cannot work. Candidate C was
   not a fallback, and B-Graphs received no competing design track. Its plan
   was `optimizer-language-research/implementation/
   minimal-systems-capability/CANDIDATE-B-STRATA-DECISIVE-PLAN.md`; under D15
   it is historical evidence, not active authority.

   The fourteen audited source operations now serve as fixed demand cases,
   workload baselines, and structural stress tests, not mandatory final
   implementations. A route may preserve the reference topology or substitute
   another data structure, reclamation strategy, or algorithm when it preserves
   the frozen consumer contract and safety/progress requirements and meets the
   preregistered non-inferiority and resource bands. Exact source-route failure
   alone is not `STRATA-NO`. The final minimum is the union of rules needed by
   one passing route per demand, so a reference-only rule must be removed when a
   qualified substitute makes it unnecessary.

   The completed packet freezes 15 conjunctive performance-demand families,
   three materially different capability hypotheses, 17 uniform comparison
   dimensions, 42 protected/witness/held-out paper routes, 22 hostile attacks,
   and six separately authorizable validation requests. That comparison found
   no evidence-backed winner. The earlier research order selected C as the
   first bounded validation hypothesis, B as the later compression challenge,
   and A as a generality control; the new B-Strata-only ruling supersedes that
   active ordering without changing its historical evidence. Stage 0 froze C
   v0, and Stage 1's five-operation Hashbrown paper
   calibration reached the mandatory Gate 1 stop with `C-REVISE`. The slice
   needs exact reusable sparse control-to-payload, transition/cleanup,
   provenance, and fact rules plus exact C0 group-operation and growth-
   allocation rows. It found no need for Hashbrown-name recognition, no new
   admitted family, and no unavoidable structural tax, but provides no safety,
   code-shape, or measured-performance closure.

   The completed bounded paper-repair route compared three alternatives:
   operation-closed sparse profiles, one profile-indexed sparse automaton, and
   an orthogonal factoring control. The exact fifteen-row matrix gives five
   `CLOSED` routes each to the first two and five `CONVERGES-B` routes to the
   third. The Sparse Repair Gate selects `SR-PROFILE` only as the next research
   hypothesis: one compiler-owned profile-indexed sparse automaton plus fixed
   insert, replace, remove, relocate, and rehash templates. Candidate C v0 is
   unchanged, and no safety, implementation, code-shape, or performance claim
   follows.

   Candidate B's bounded compression comparison is complete across fourteen
   operations and four pinned source revisions: five Hashbrown operations;
   allocation, local free, and remote free/collection in mimalloc; B-tree
   insertion/split, deletion/rebalance, and pager rollback in SQLite; and
   protected load, retirement, and collection in Crossbeam Epoch. The exact
   42-route matrix gives the original `B-FORMS` fourteen open rows,
   `B-STRATA` six closed and eight open rows, and `B-GRAPHS` six closed and
   eight open rows. The mandatory Design Gate is `B-REVISE`.

   `B-STRATA` is the best-defined revision hypothesis, not a selected language
   design. Its eight project-independent layers compress physical places,
   structural liveness, owner transitions, scoped progress and repair,
   physical-root provenance, finite executable disposition, invalidatable
   facts, and concurrent custody. The same layers close the paper capability
   routes for all five Hashbrown operations and complete mimalloc small
   allocation. Final allocator-page disposition, policy-neutral quiescence,
   Crossbeam's heterogeneous erased one-shot action, and exact SQLite pager/
   VFS/WAL and reinitializer contracts remain open. Independent hostile review
   removed a validator-to-liveness authority hole, rejected hot-subpath credit
   for complete operations, downgraded four SQLite mutation routes and both
   local-free routes, and changed three overclaimed graph convergence rows to
   `OPEN`. No evidence-backed production winner follows.

   The sealed compiler-embedded registry, P1-P9/Q1-Q6 proposal, production-
   language census, gate attacks, 49-row crosswalk, 26-domain routing, held-outs,
   and structural-cost fields remain historical evidence and candidate inputs.
   They are not a selected direction. Isolation applies only if a surviving
   residual machine capability requires compiler-private definition; it is not
   the objective or derivation order. Cryptographic authorization and
   independently distributed privileged extensions remain a separate problem.

   The finite evidence remains load-bearing and fail-closed: the Rust 1.97.0
   anchor, 276 coverage clusters, 26-domain systems envelope, dense Family Lock,
   and held-outs define coverage and falsification demands. The dense ledger
   still has 340 unresolved required route obligations across 150 contexts,
   including 208 Convert-related, 136 allocator-related, and 12 ZST/fullness
   obligations. Exact D-2 derivability and P-1 same-contract structural
   performance remain pending. Routed categories receive no exact derivability
   credit, paper routes receive no measured-performance credit, and structural
   analysis is not a formal safety proof.

   Its contract (suspended under D15, retained as evidence) is
   `optimizer-language-research/implementation/minimal-systems-capability/
   CANDIDATE-B-STRATA-DECISIVE-PLAN.md`. Phase 1 first normalizes the eight
   analytical strata into a finite working upper-bound core and front-loads
   liveness authority, policy-neutral quiescence, erased one-shot disposition,
   and exact external-repair boundaries as reference-pressure tests. Phase 2
   freezes the same fourteen demand contracts and a bounded reference/substitute
   route frontier, then exposes which rules can be deleted. Demand-level paper
   closure precedes hostile safety and erasure modeling; only a paper YES and
   model YES authorize the smallest preregistered cross-project prototypes,
   generated-code inspection, and measurement needed for the final verdict.
   After evidence, one passing route per demand determines the final reduced
   core and all earlier gates rerun. No additional candidate, project, or demand
   is authorized. Production language, specification, checker, compiler,
   verifier, runtime, standard-library, container, xlc, migration,
   fact-channel, selection, teaching, or shipping changes remain separately
   gated after a final `STRATA-YES`. Exact D-2/P-1 remain fail-closed, and the
   separate xlc self-hosting build track must not be mixed into this research.

   The completed research charter is
   `optimizer-language-research/implementation/minimal-systems-capability/
   PERFORMANCE-FIRST-CAPABILITY-RESEARCH-CHARTER.md`; the completed bounded
   Candidate C validation plan is `CANDIDATE-C-BOUNDED-VALIDATION-PLAN.md`,
   exhausted at Gate 1. The completed paper-repair contract is
   `CANDIDATE-C-SPARSE-REPAIR-PLAN.md`; its authorization is exhausted at the
   Sparse Repair Gate. The completed bounded contract is
   `CANDIDATE-B-ELEGANT-DESIGN-PLAN.md`; its authorization is exhausted at the
   `B-REVISE` Design Gate. Their old authorization boundaries remain historical
   facts rather than constraints on the new decisive plan.
2. **xlc self-hosting build track** — ACTIVE BUILD TRACK. The production compiler
   now parses, validates, indexes, and resolves all types in its own 477-function
   unit. The first S1 baseline is established: a pure whole-unit semantic driver
   deterministically reports 15 clean, 462 legal-unsupported, and zero semantic
   rejects, with the first source-order frontier at `lexer_scan_op_suffix`.
   Legal non-profile functions are no longer misreported as type errors, and the
   existing 15-function LLVM module remains byte-identical. E0 is now an ordered
   five-part expressiveness validation: flat data layout/owning sequences; closed-unit
   modules, inherent impl and static contracts; borrowed aggregates plus
   fact-carrying loops/range match; byte constants/bulk append; and typed SIMD
   with target dispatch. Exactly one part runs at a time, beginning with the
   current SoA-vs-AoS/owning-sequence question. Each part freezes a factorial
   protocol, preserves unchanged-source code shape, measures default-writer
   selection where applicable, and receives hostile review before any language
   change. The earlier local-own/Bool-if/formatting observations are parked as
   surface costs, not treated as evidence of lost performance. After those five
   decisions, the next S1 slice adds an acyclic-decision semantic family for
   `lexer_scan_op_suffix` plus `lexer_scan_word`, without adding lowering; later
   slices extend copy-scalar, type, ownership, and effect rules across the
   remaining functions. The staged route continues through whole-unit lowering,
   a stage-1 compiler, and the byte-identical self-hosting fixpoint; every slice
   keeps both repository gates green. **E0.1 status (2026-07-13):** the first
   detached record-buffer prototype is rejected but durably archived; owner choice
   among automatic structural Copy, declarative `copy struct`, and nominally affine
   record storage is reopened, so no production design is selected. Independently
   authorized current-language conformance repairs close the checker drift at
   `7438e17` and parser drift at `50a1ddd`. Strict GRAM-9 required 744 additional
   bindings in the
   self-hosted compiler source, but the isolated whole-compiler optimized object is
   byte-identical (264,288 bytes, SHA-256
   `eef9c30193b69be22452f47bc8050453f5da9d86454ff165c74da0be6241b522`). The MCTS-Mem
   design tree remains unchanged until the owner selects a production route.
   The paired E0.1 ownership screen must not enter Lock A. Any later dense-family
   Lock A must explicitly retain, revise, or supersede every relevant E0.1 arm
   and measurement; the fixed builder and declarative-Copy arms do not cover the
   general sequence, sparse-slot, or deletion/reuse contracts. G0-Core alone is
   not candidate authorization.
3. **Default-floor experiment against shipped Rust (D9a)** — COMPLETE on two
   separately preregistered targets. First-green Terra xlang records paired
   throughput ratios of 1.653x [1.631, 1.667] against `percent-encoding` 2.3.2
   and 1.098x [1.085, 1.145] against one-shot `utf8parse` 0.2.2. Both retain
   every reported xlang bounds site, so neither is a proof-elision win. This is
   cross-target performance-floor evidence, not a pooled ecosystem result or
   expert-Rust ceiling claim. Canonical wording and caveats are in
   `experiments/default-floor/RESULTS.md`; no third target is required for D9a.
4. **Proof-elided checks (OP-4 tier)** — enabling evidence for the experiment.
   A checked concrete-function `requires` prologue
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
5. **Leg-A frequency pilot** — complete; stop building analysis infrastructure.
   Raw source heuristics substantially overcounted alias and saturating-law
   opportunities, while optimized `comrak` exposed many surviving checks in
   genuinely hot parser paths. The pilot therefore redirects the next bounded
   experiment toward generalized relational/precondition proof in one real
   workload. It does not clear D9, estimate ecosystem prevalence, or justify
   further frequency-tool investment.
6. **Channel 4: blessed interpreter dispatch** (carded): lower naive
   loop+match to threaded/musttail dispatch — structural delta over Rust,
   parity with expert C from the obvious shape. Eventual benchmark: the
   owner's own engine (Silverfir), owner-refereed.
7. **Coreutils ladder** (D7a): wc, base64 done; next utilities need the I/O
   frame (first D4 FFI instance) and chunked-driver parity. The AI-authorship
   headline runs through the shelved trial harness when the time comes.

STATUS UPDATE (2026-07-13): D9a is complete on two separately preregistered
targets. Default Terra xlang beats shipped `percent-encoding` 2.3.2 by 1.653x
and shipped `utf8parse` 0.2.2 by 1.098x on their locked workloads. Neither win
comes from proof-elided bounds checks. This supports a replicated default-shape
thesis while remaining limited to these implementations, corpora, and machine.
THE BUILD TRACK IS ACTIVE: `compiler/` hosts xlc, the production compiler
written in xlang itself (the current baseline remains the P2 SoA-tape
architecture with fixed-capacity buffers and no generics), bootstrapped by
prototype/democ as stage 0,
with its own gate (`make -C compiler check`, incl. the self-parse gate).

## 5. The pivot clause (pre-registered)

The primary outcome is scored exactly once on the frozen first-green
artifact. If it does not beat the shipped Rust baseline outside the
pre-registered equivalence band, the default-floor hypothesis is unsupported
for this task; do not rescue it with human tuning, a stronger model, or expert
Rust. If it wins, pre-register and run a second shipped-library task before
claiming a general performance-floor effect. A ceiling result from expert Rust
does not erase a default-floor win, and a facts-off result determines whether
the measured source of the win was proof elision or another taught/forced
shape. If the default path repeatedly ties or loses, the honest pitch becomes:
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
- Repository language: every repository-resident artifact and filename uses
  English only (D10); translated duplicates and language-suffixed reports are
  forbidden.
