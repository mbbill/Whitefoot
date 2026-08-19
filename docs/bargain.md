# The bargain ledger

Status: DATED, NON-AUTHORITATIVE SYNTHESIS. The living
[`Direction Outline`](roadmap.md) owns current status and candidate placement;
an `ACTIVE` [`current-plan.md`](current-plan.md) owns execution sequencing.
Status words and
phase references below describe the ledger when compiled and must not be used
as current authorization. Compiled 2026-07-28 from the founding directives,
the round-2/3/4 design debates, the headline brainstorms, the research backlog,
the capability-research era, the live docs, and one fresh brainstorm. §8 (the
ML direction) and §9 (the embedded direction) added 2026-07-31, each after an
external research pass (eight and seven clusters respectively) whose dated
findings they cite; both passes refuted parts of their own briefings, and the
refutations are recorded inline.

This file explains the founding trade and preserves cross-direction rationale;
it is no longer a work queue or periodic re-ranking owner. D0, 2026-07-01,
gives away human ergonomics entirely and designs for an AI writer with a human
approver. Every entry below answers one question: **what does that trade buy
here, what does it cost, and what was its evidential status?** Promote a live
status, gap, or next gate to the Direction Outline and leave measurements and
long explanations in their canonical evidence owners.

Status vocabulary: **LAW** (constitution/spec/tree invariant) · **MEASURED**
(a RESULTS record exists) · **ADOPTED** (in the language or doctrine, not
separately measured) · **CARDED** (a written card or idea entry, not built) ·
**OPEN** (genuinely undecided) · **CONTESTED** (mechanism exists, value
disputed) · **SPECULATIVE** (this file is its first record) · **KILLED**
(rejected with a recorded reason — do not re-propose without naming the lapsed
reason).

What the writer-side of the trade actually provides, from D0–D6: unlimited
explicitness (program verbosity is free), zero style attachment, no installed
base, no familiarity demands, tolerance of rejection loops, in-context
teachability — and one scarce resource that replaces all the human ones: the
**spec/teaching token budget**, which is the real currency every entry below
spends or saves.

## 1. Performance: facts the writer cannot refuse to state

- **Effect rows → cross-boundary optimization.** Verified `reads/writes/
  allocates/traps` rows become function attributes, so calls optimize across
  boundaries no inliner or LTO can cross. *Delta:* per-file `-O2` equals Rust's
  fat-LTO configuration; O(n)→O(1) measured at an opaque `.o` boundary.
  *Status:* MEASURED (democ era); re-emission is Phase 10 work, gated on the
  ABI split and the `willreturn` hazard. (`research/experiments/effect-attrs-channel/`)
- **Borrow modes → alias facts.** `&uniq` is universal `noalias` with no
  interior-mutability holes; single-owner columns give guard-free
  vectorization. *Delta:* 0 runtime alias guards and 121 asm lines where
  Rust's obvious shape emits 29 guards and 2,132 lines; 2.0x at short trips;
  parity at long trips. *Status:* MEASURED (democ era). (`research/experiments/scoped-alias-channel/`)
- **Checked algebraic laws (FN-4).** A stated `associative`/`commutative`/
  `identity` law is discharged or refuted at compile time; a discharged law can
  license reassociation. *Delta:* 3.3x over the obvious fold, ties expert Rust
  — and the false law (signed saturating add) is a compile error where Rust
  silently miscompiles the hand-written idiom. No surveyed language machine-
  checks the law. *Status:* MEASURED (democ era); optimizer consumption needs a
  separately approved fact family; zero opportunity in the current corpus.
- **Static call contracts → proof-based check elimination.** Every ordinary
  caller proves every `requires` clause before transfer; the callee receives
  those facts without an executable prologue, and the sole command entry has
  no contract. *Historical delta:* the democ-era base64 experiment discharged
  27/27 bounds sites and measured 1.71x, while Rust's `assert!` recovered
  nothing. Honest adversary: Flux/RefinedRust-style refinement typing, not
  vanilla rustc. *Status:* ADOPTED; current compiler path is proof-only.
- **Per-operation numeric semantics.** Wrapping, checked, and saturating forms
  are explicit values. Exact arithmetic is admitted only after its matching
  total `.defined` goal is proved; an explicit `claim` over that goal is the
  sole runtime backstop. Debug and release are therefore the same program,
  with neither signed-overflow UB nor a hidden arithmetic trap. *Status:*
  ADOPTED + MEASURED (checked and proof-discharged foldings).
- **Signature-complete provenance.** v0.17 slice origin sets and resolved-place
  effects give interprocedural alias/effect answers without opening bodies.
  *Status:* ADOPTED (checker-side); backend consumption not started.
- **Obligation-driven check accounting.** The compiler derives obligations from
  the body and offers a proven fast path plus checked fallback, rather than
  trusting writer-spelled facts; includes the GATE-1 guard against a writer
  "unlocking" codegen with an absurd over-restrictive bound. *Status:* reviewed
  design, narrow base64 slice was built, harness retired 2026-07-22; preserved
  research. (`research/requires-check-accounting-design.md`)
- **Refined integer domains → automatic niches.** Compiler-checked "nonzero
  u64"-class refinements make invalid bit patterns known, so `Option<T>` packs
  and layout improves without sentinels. *Status:* CARDED. (`docs/ideas.md`)
- **Totality facts.** Derived termination would license `willreturn`, hoisting,
  and dead-pure-call deletion; blocked on a termination checker; pure-row
  totality was KILLED (memory effects are termination-irrelevant). *Status:* OPEN.
- **Threaded/musttail interpreter dispatch from one naive `match`.** The closed
  exhaustive match licenses tail-duplicated dispatch Rust cannot express (no
  guaranteed tail calls, no computed goto). *Risk:* LLVM SimplifyCFG re-merges
  the duplication. *Status:* CARDED ("channel 4"), unbuilt.
- **Injective-scatter disjointness.** `out[perm[i]] = f(in[i])` proven disjoint
  is the one structural parallel win safe Rust cannot express. *Delta:*
  1.13–1.51x, narrow domain. *Status:* MEASURED residual; the fact is not yet
  in the effect/place vocabulary.
- **Declared parallelism.** Writer marks the construct; the compiler proves
  non-interference from rows and places — no `Send`/`Sync` ceremony, irregular
  cases rejected at compile time (29% of parallel accesses in a 14-benchmark
  Rust port study forced `unsafe` or dynamic checks — SPAA 2024). *Status:*
  roadmap Phase 11, gated; the auto-discovery form is KILLED.
- **Check motion for vectorization.** Sinking overflow checks / hoisting bounds
  checks is sound because traps abort and no global mutable state exists;
  reaches C's UB-assumed loop speed proof-backed. *Status:* Phase 10 completion
  set, carries the DIAG-3 amendment (the trap-semantics pilot).
- **Proof-guided autotuning.** The checker admits semantically-proven variants;
  a benchmark picks the fastest — cost selection can never make an unsafe
  variant legal. *Status:* CARDED. (`docs/ideas.md`)
- **Closed-world specialization.** No dynamic dispatch, no open world: full
  monomorphization and whole-program layout/const knowledge are the default,
  not an LTO upgrade. *Status:* ADOPTED (inherent), never separately measured.

## 2. The floor: the worst accepted program is a good program

Reframed 2026-07-27 (W1 = floor robustness): an accepted program has been
forced onto a fast shape; the writer's only alternative is a program that does
not compile. The floor is engineered, finding by finding — it is the project's
best-supported differentiator to date.

- **One spelling, one loop, one conditional → naive = fast.** *Delta:* obvious
  base64 1.6x over obvious Rust; first-green floor series 1.65x
  (`percent-encoding`) and 1.10x (`utf8parse`) over shipped crates — with fact
  channels at parity, so shape alone carried it. *Status:* MEASURED (narrow,
  explicitly not generalized).
- **Closed pattern catalog (D6).** Slow architectures are unrepresentable, not
  discouraged: no `Rc<RefCell>` graphs, no scattered mutation, no
  pointer-chasing defaults. *Delta:* binary-trees showed a 13.5x Rust
  shape-effect that Whitefoot cannot express. *Status:* SEEDED in
  `docs/patterns.md`; not yet ratified as normative doctrine.
- **SoA pools + copy handles + generations.** Contiguous columns for cache and
  vectorization; stale handles trap deterministically instead of silently
  reading the new occupant (Rust's index-arena footgun closed; Vale's
  generational references are the prior art). *Status:* the append-only SoA
  pattern is seeded; generational reuse and the catalog mechanism are deferred.
- **Boolean i1 dataflow (P7).** Keeping scanner state in `Bool` vectorizes at
  width 16 instead of 2. *Delta:* closed a measured 1.6–1.8x loss to parity;
  drove the OWN-1 Bool-copy amendment. *Status:* MEASURED, taught.
- **Traps to the boundary (P8).** Validation at edges keeps hot interiors
  trap-free and vectorizable. *Delta:* 2x on wc -l from one counter-mode
  change. *Status:* MEASURED, taught.
- **Feature replacements selected for the AI writer** (round 2, all adopted):
  env-structs replace closures; sum types + exhaustive match replace dynamic
  dispatch (missing arms become a machine-executable edit list); `Result` +
  derived context replaces unwinding; deterministic external generators replace
  macros; re-checked function contracts replace module interfaces; checked
  storage contracts + visible drops replace GC/ARC. Generics were KEPT
  precisely because N-copy "makes each rewrite a fresh hallucination
  opportunity."
- **The floor audit loop** (2026-07-27 protocol): diff what an AI actually
  writes against a measured reference shape; every slower-but-accepted
  divergence becomes a rejection rule, a pattern card, or a lowering repair.
  The AI is a mistake generator; the finish line is the checker. *Status:*
  ADOPTED protocol, first bounded round in the Phase 10 completion set.
- **Complexity/resource budgets as checked contracts.** `allocates` rows
  already make allocation signature-visible; the extension — declared
  allocation counts or bounds a checker enforces structurally — would make "no
  allocation in this subtree" a compile-time fact. *Status:* SPECULATIVE
  (adjacent: resource certificates, `docs/ideas.md`).

## 3. Trust: cheating made unrepresentable (W3)

- **No writer-emittable third state.** Nothing writer-stated is trusted
  unchecked — no unsafe, no assume, no trusted annotation; across four round-3
  debates no advocate even defended a free-form hatch. *Status:* LAW.
- **Bidirectional effect checking.** Declared-but-unexhibited is an error too,
  so a row cannot be padded as a smuggling surface. *Status:* LAW.
- **Contract-edit gating.** Editing a declared contract is a privileged
  operation, so a writer cannot launder a wrong body by rewriting its own
  declaration. *Status:* ADOPTED direction (round 2).
- **Canonical bytes → nowhere to hide.** Formatting-only diffs cannot exist; a
  sneaky edit cannot hide in noise; review is always semantic. *Status:* LAW.
- **The sealed-kernel catalog + D16 minimality + D17 proof lane.** Risk
  concentrated in ~10 audited kernels instead of distributed `unsafe`;
  admission only for capabilities users cannot reach at par performance; a
  machine-proved invariant buys the same privilege permanently ("trust is a
  loan, proof buys permanent privilege"). *Status:* the Constitution now records
  D17's long-term proof lane; the roadmap defers the exact catalog/storage
  mechanism and production work; the honest caveat on the trusted-LOC size
  stands (§8).
- **The law refuter.** A false algebraic claim is a compile error with a cited
  rule — the transform every performance engineer does on faith becomes
  checked. *Status:* ADOPTED (validation-only in v0.17).
- **GATE-1 anti-gaming.** Strengthening a `requires` predicate sits outside
  ordinary writer authority, blocking the "absurd worst-case bound that unlocks
  codegen while shrinking the real API" attack. *Status:* reviewed design.
- **Held-out anti-memorization witnesses.** Frozen, hashed canary structures
  hidden from candidate authors until after freeze — guards against an AI
  reproducing a famous structure from memory instead of deriving it; "a
  concern with no direct human-engineer analogue." *Status:* ADOPTED gate in
  the capability research; reusable anywhere candidates are AI-generated.
- **BRAND-1 nominal identity + CONC-0 memory model.** Brand typing closed five
  forgery attacks on arena/queue identities; the from-scratch happens-before
  model makes "no data races" checkable rather than asserted. *Status:*
  ratified in the capability research; production separately gated.
- **Grep-zero-unsafe as an artifact property.** "The AI cannot have introduced
  memory corruption" is checkable on the artifact, not a review promise. The
  honest adversary is `#![forbid(unsafe_code)]` Rust; the surviving stack is
  no-unsafe AND per-op defined overflow AND AI authorship with a mechanical
  trust surface. *Status:* ADOPTED framing.
- **Optimization receipts.** Every removed check carries its proposition,
  producer, invalidators, and consumer — reviewable without reconstructing the
  optimizer. *Status:* CARDED. (`docs/ideas.md`)
- **Redundancy as hallucination checksum.** Writer-stated facts cross-checked
  against derived facts. *Status:* CONTESTED — same-pass model output may be
  error-correlated with itself, so the independence assumption is unproven;
  flagged everywhere the mechanism recurs.

## 4. Toolchain: one text, one tree, one artifact

- **Deterministic, rule-citing, byte-stable diagnostics as an API.** The repair
  loop is the writer's inner loop. *Evidence:* writability trials passed the
  pre-registered 70% bar only once a diagnostic-repair cycle was added (40%
  first-shot); the loop is load-bearing, not decorative. *Status:* ADOPTED;
  the "compiler as teacher" content ablation is an unrun experiment.
- **Canonical elaborated artifact as the re-read target.** Acceptance decidable
  from the artifact alone; drops, instantiations, and retained checks explicit.
  *Status:* LAW.
- **Reproducible builds; semantic diff and merge.** By construction of
  canonical form. *Status:* ADOPTED (by construction; whole-toolchain
  reproducibility gates are roadmap items, not claims).
- **Content-addressed caching / compile latency as the true cost channel.**
  "Compute is admissible, writer tokens are not" — the AI-loop analogue of
  build ergonomics. *Status:* CARDED, unmeasured; named the "biggest unpriced
  cost" of the closed-world design.
- **Regeneration-drift consistency checking.** Near-duplicate regeneration
  drift was the single most-cited AI-specific failure mode in round 2; a
  unified consistency layer (and the drift-rate experiment) remain unbuilt.
  *Status:* OPEN.
- **Alien-lexicon falsifiability.** Spec validation must include an
  alien-lexicon rendering so pretraining familiarity cannot mask spec gaps.
  *Status:* ADOPTED standing rule.
- **Candidate-construction harness.** AI-written candidates evaluated like ML
  systems — frozen model, tool, token, and repair budgets, randomized order —
  not like human PRs. *Status:* ADOPTED protocol in the capability research.
- **Mutual-oracle backends + differential fuzzing.** Independent LLVM/C/wasm
  backends compare values, traps, and teardown; the facts-on/facts-off
  differential instrument (Phase 10 completion set) is the same idea aimed at
  check-eliding channels. *Status:* CARDED + one instrument scheduled.
- **The native grammar verifier.** Spec proposals are checked by the compiler's
  own lexer/parser — the toolchain polices its own language evolution.
  *Status:* ADOPTED, in use.

## 5. Shipping: what the artifacts can be

- **The ABI swap-in mechanism.** C-ABI emission means artifacts LD_PRELOAD or
  symlink over incumbents with no relink; graded ladder from single-symbol
  crc32 through LZ4/zstd/zlib to vendored kernels (no canonical ABI exists for
  protobuf/DER/DNS/archive parsers — "drop-in" is honestly weaker there).
  *Status:* CARDED strategy; deferred behind Phase 11 as of 2026-07-27.
- **The 26-target artifact bank.** Codecs (zlib inflate/deflate, LZ4, snappy,
  zstd, base64, CRC32), images (QOI, PNG leaf and capstone, JPEG, WebP VP8L,
  stb_image, GIF), parsers (JSON validator, UTF-8 validator, protobuf varint,
  DER/X.509, DNS, TAR/ZIP/ELF), fonts (TrueType), text tools (Myers diff,
  grep -F, minifier), bitmaps (roaring). Shared essence: bit-identical,
  zero-unsafe, AI-authored, CVE-class-unrepresentable, with speed riding on
  the fact channels. Convergent top pick across five independent lenses: zlib
  inflate. *Status:* CARDED bank. (`research/notes/headline-artifact-brainstorm.md`)
- **Encode-vs-decode honesty rule.** Decode targets claim bit-identical;
  encoders claim only ratio-parity — never byte-identity against tuned
  match-finders. *Status:* recorded discipline.
- **Constant-time `secret` effect.** Secret-typed values provably never steer a
  branch, index, or variable-latency op; only DSLs (Jasmin, FaCT, CT-Wasm) do
  this today, and Rust's `subtle` cannot stop LLVM's select-to-branch
  miscompile. Killed once (D7c) for tripping a content filter — not on
  technical grounds — and restored to the deferred list 2026-07-27. Load-
  bearing gap: a backend constant-time-preservation contract through
  `clang -O2`. *Status:* CARDED, deferred, gap named.
- **Resource certificates.** Machine-readable stack/heap bounds, trap sites,
  and effects beside the artifact, checkable against a deployment budget.
  *Status:* CARDED. (`docs/ideas.md`)
- **Effect-derived sandbox policies.** WASI capability sets or syscall policies
  generated from checked effect rows, failing closed. *Status:* CARDED,
  blocked on real I/O effects existing. (`docs/ideas.md`)
- **Safe C ABI capsules; portable C backend; C-to-Whitefoot assumption
  extractor.** Export through opaque validated handles; reach non-LLVM targets
  with a per-fact disposition rule; ingest restricted C by converting its
  unstated assumptions into checked obligations. *Status:* CARDED. (`docs/ideas.md`)
- **The coreutils headline (D7a).** "AI wrote it, faster than GNU, the
  memory-corruption CVE class is unrepresentable" — explicitly not "no CVE";
  logic bugs remain and fuzz-diff is the answer. *Status:* CARDED framing.

## 6. Evolution: a language with no installed base

- **Breaking changes are cheap and stay cheap until external programs exist.**
  The writers regenerate; they do not migrate. The corpus is the owner's test
  programs. This freedom is an asset with an expiry date. *Status:* owner-
  affirmed 2026-07-27.
- **Evidence-selected forms (R3).** Every provisional spelling can in principle
  be A/B-tested against writers rather than argued in committee; canonical
  form makes each change mechanical to apply corpus-wide. *Status:* LAW in
  principle; the selection experiments are largely unrun.
- **The spec as experimental apparatus.** Append-only versions, a compiler-
  independent conformance corpus, and the one-hypothesis-per-loop experiment
  discipline make language semantics a measurable object. *Status:* ADOPTED,
  operating.
- **In-context distribution.** New model, same spec, same result — no
  pretraining dependency; the teaching pack is the unit of distribution. The
  underlying teachability bet (regular-but-alien beats familiar-but-irregular
  in-context) is the single largest UNVALIDATED premise in the corpus — an
  entire pre-registered experiment program (spec-size, regularity-vs-size,
  alien-vs-familiar arms) exists and has never run. *Status:* OPEN at the
  foundation.
- **Model-capability gates rejected twice.** D5 (2026-07-09) deprioritized the
  weak-model sprint; the 2026-07-27 reframe removed model scores from W1
  entirely. The stable pattern: bet on checker properties, not on any model
  generation. *Status:* owner doctrine.

## 7. Unrecorded possibilities (first written down 2026-07-28)

Brainstormed in discussion; nothing below appears in any repository record.
Each is SPECULATIVE until an experiment card exists.

- **The language as an RL/synthesis environment.** A deterministic, byte-stable,
  rule-citing checker over a canonical search space is an ideal reward oracle:
  dense feedback, no reward hacking (W3 is exactly anti-reward-hacking), and
  self-labeling corpora. Training-data generation and checker-guided program
  search (MCTS/enumerate-and-check, source-level superoptimization) fall out
  of properties the language already has.
- **The rejection-data flywheel.** Every rejection carries rule + node path —
  the checker continuously produces perfectly-labeled "why this is wrong"
  data usable to tune writers and teaching packs.
- **Capability-visible supply chain.** With no ambient authority, a
  dependency's whole external authority is readable from its signatures:
  "this library cannot touch the network/filesystem" as a compile-time fact,
  not an audit conclusion. (Gains force the moment I/O effects land.)
- **Mechanical semantic versioning.** Signatures + effect rows + contracts
  make "did the observable surface change" decidable; API-diff and semver
  enforcement become checker queries.
- **Provenance manifests for AI supply chains.** Embed model identity, spec
  hash, and repair-loop record in the artifact: an auditable "who wrote this
  and against which law" story no human-authored ecosystem can match.
- **Compiler-chosen data layout.** No human debugs memory dumps and no stable
  ABI exists inside the closed unit, so field order, packing, and niche
  exploitation are the compiler's to optimize per program — layout as a
  measured decision, not a declaration.
- **Deterministic record-replay debugging.** No UB, no races, defined traps,
  canonical elaboration: recording inputs replays the program exactly — the
  approval human gets time-travel debugging almost for free.
- **Portable determinism as a product property.** Strict IEEE, defined
  everything, bit-identical cross-target results: lockstep simulation,
  consensus/state-machine replication, and reproducible science are natural
  fits no C/Rust artifact can promise without heroics.
- **The formal-verification bridge.** The canonical elaborated artifact is an
  unusually good proof target (one text, explicit drops and checks);
  translation validation of the LLVM boundary (Alive2-style) and refinement
  proofs against the spec become tractable projects.
- **WCET/certification story.** Proof-discharged checks with receipts, no
  hidden runtime, explicit allocation, derived totality: a path toward
  certifiable timing/resource bounds (safety-critical adjacency) that
  GC/unwinding languages cannot take.
- **Multi-agent collaboration without merge noise.** Canonical bytes remove
  formatting conflicts; effect rows and ownership partition the world state so
  two writers cannot silently interfere — concurrency control for agent teams,
  enforced by the checker.
- **Machine-checked spec migration.** A vN→vN+1 change ships with a migrator
  verified against the conformance corpus — or simply with regeneration —
  making language evolution a tooling operation rather than an ecosystem
  event.
- **Fix-once security response.** If the deferred sealed catalog is adopted, a
  kernel defect is fixed in one audited project artifact and the world
  recompiles; no ecosystem-wide unsafe audit.
- **Documentation-free library consumption.** `doc` fields + complete
  signatures are the entire interface contract; a consumer model needs no
  prose, no examples, no tribal knowledge — library ecosystems sized to
  context windows.

## 8. The ML direction (owner intent stated 2026-07-27; researched 2026-07-31)

The owner's stated ambition: Whitefoot should eventually shine in ML
development, where Python dominates despite its defects. An eight-cluster
external research pass (Mojo, Julia, GPU routes, autodiff, shape checking,
determinism demand, the safe-language ML systems layer, JAX purity and
AI-written-ML evidence; all findings dated 2025-2026) verified the thesis's
claim clusters. Everything here is SPECULATIVE or OPEN — no roadmap standing;
the Phase 10/11 structure is unchanged, and this section exists so the
direction is finally in the record with its evidence and its refutations.

**The thesis, stated honestly.** Python's ML dominance is a human-ergonomics
equilibrium (notebooks, dynamism, ecosystem). The strong form — "the AI writer
dissolves the notebook advantage" — is *contradicted by current product
evidence*: 2025-2026 agent tooling is building deeper into notebook/REPL
execute-observe loops, not abandoning them, because grounded execution
feedback is what agents want. The surviving form is conditional: Whitefoot can
compete for AI writers only if its compile-run-trap loop is as fast and as
informative as a REPL turn — which makes build latency an ML-critical
property, not polish (Julia's TTFX history shows first-impression latency
damage outlives its fix by years). The floor argument does translate: ML
iteration is priced in GPU-hours, so "the experiment either doesn't compile or
runs to completion" — no shape error at hour 30, no silent dtype promotion, no
NaN from an unchecked cast — is the ML form of the floor claim.

**What the research confirmed (with dates):**

- **Purity checking is a real delta over JAX, stated precisely.** JAX's
  silently-dropped side effects under jit, tracer leaks, and PRNG key reuse
  are real and recurring (active GitHub issues through 2025) — but JAX ships
  opt-in *dynamic* checkers for two of the three (leak checker,
  `jax_debug_key_reuse`, both off by default; the key-reuse JEP is considering
  a stateful redesign because the functional keys are error-prone). The honest
  delta is **static-and-default versus dynamic-and-optional**, not "checked
  versus nothing." PRNG-keys-as-checked-affine-resource is the sharpest single
  instance: Whitefoot's ownership rules express exactly that today.
- **Compile-time shapes are real, unsolved, and being converged on.** Shape
  faults have a dedicated fault taxonomy (2021) and, decisively, Meta's
  Pyrefly added *experimental static tensor-shape checking* to a Python type
  checker in 2025-2026 — fresh evidence that a major player considers the
  problem worth compile-time treatment and unsolved. Whitefoot's const
  generics + `Int`-bounded parameters + `requires` are most of the mechanism
  already. Caveat recorded: no recent study supports "shape/dtype errors
  dominate LLM-written-ML failures" — MLE-bench failure analyses blame
  planning/verification, not shape errors; do not cite that statistic, it does
  not exist.
- **Determinism demand is real but must be claimed narrowly.**
  `torch.use_deterministic_algorithms` costs 2-5x with coverage gaps that
  raise errors; OpenMP explicitly disclaims bitwise-identical reductions; the
  Thinking Machines batch-invariance work (2025-09) proved LLM-inference
  nondeterminism fixable at 1.6-2.1x cost. But: TPU/XLA already claims
  bit-identical training in production, the Thinking Machines fix shipped as
  a *kernel library* in the existing stack, and FDA/finance guidance accepts
  "statistically equivalent." The claimable slice: **language-checked
  reproducibility on commodity hardware without vendor lock-in, where the
  compiler proves it rather than a library promising it** — a niche of
  sophisticated buyers, not a mass demand.
- **bf16 and f8 (E4M3/E5M2) are real upstream LLVM IR types today**, so a
  reduced-precision dtype family targets present backend capability; the OCP
  MX block-scaled formats are compound types (element + block + scale +
  rounding) — a materially bigger spec commitment PyTorch itself had not
  natively landed as of mid-2025.
- **Mojo, calibrated.** The closest adversary in positioning, not in
  guarantees: documented `UnsafePointer` escape hatch used throughout its
  stdlib, no algebraic-law checking or effect system surfaced, determinism by
  discipline not by compiler, compiler still closed-source (opening promised
  "by end of 2026"), 0.3-0.4% adoption (SO 2025), classes still unimplemented
  as of 2025-09 — but real, peer-reviewed multi-vendor GPU results (SC'25,
  H100 + MI300A, "competitive" with CUDA/HIP). Mojo's GPU lead is genuine;
  its checker story is absent. W3 remains the structural delta.
- **Julia, the four transferable lessons.** TIOBE #33 / 0.43% after 14 years
  and heavy funding: (1) iteration-latency reputation outlives its fix;
  (2) a correctness-bug reputation is durable even when "it's the ecosystem,
  not the core" — the 2022 critique was re-affirmed by its author in 2025, so
  Whitefoot's checked discipline must extend to any future package ecosystem
  or it inherits the same critique; (3) the incumbent co-opts your best pitch
  (torch.compile absorbed the two-language argument); (4) what actually
  worked was a narrow high-trust niche (SciML, pharma), never the frontal
  assault — the wedge model for any newcomer.

**The two hard walls:**

- **The GPU wall, now measured.** Every route is multi-quarter-to-multi-year:
  Rust-CUDA's reboot needed 7+ months just to restore basics on a previously
  working NVVM backend (NVIDIA's libNVVM pins an old LLVM, forcing manual
  feature backports); Mojo's GPU story is a company-scale multi-year MLIR
  investment; the cheapest historical precedent (Futhark's CUDA backend, one
  BSc thesis) is a *source-text-generation-plus-vendor-compiler* route that
  Whitefoot's LLVM-text pipeline does not resemble — tinygrad's pattern (tiny
  op set, delegate to vendor compilers) is the low-effort shape. And bitwise-
  reproducible GPU compute at competitive performance is an *open research
  problem* (papers still proposing systems in 2026): vendor-kernel speed comes
  precisely from the atomics and autotuning that determinism forbids, so
  "deterministic AND state-of-the-art GPU" is not currently purchasable at
  any engineering price. Any Whitefoot GPU claim starts as "a deterministic-
  mode subset at a real performance tax."
- **The interop wall, which reorders prerequisites.** The settled 2025-2026
  production pattern is Python-as-orchestrator calling safe compiled
  libraries (PyO3): Polars' enterprise traction and vLLM's 2026 Rust frontend
  merge (~5x request-preprocessing throughput) both won *as libraries inside
  the Python world*, and the field's own data-loading fixes (Meta's 56%
  GPU-stall figure) are scheduling/IO systems, not language rewrites. Whitefoot
  has no FFI. Consequence: **the §14 gated FFI family is the ML direction's
  first language prerequisite** — without a callable-from-Python story there
  is no entry into the only wedge the market has actually validated.

**Angles the research closed (moved to the killed list):** the
pickle/safetensors security wedge (pickle is still pervasive and still
producing 2026 CVEs, but the fix is format adoption — a behavior problem a new
language does not move); the tokenizer wedge (HF tokenizers is already Rust;
the 2026 frontier is Rust-vs-Rust SIMD work); kernel-layer safety demand
(llama.cpp shows zero visible appetite to leave C/C++ at the kernel layer).

**The wedge ranking that survives the evidence:**

1. **Serving/orchestration glue** (the vLLM-frontend shape): CPU-bound request
   handling, scheduling, batching, KV-cache bookkeeping — where safety bugs
   are plausible and the 5x-class wins are documented. Requires FFI + a
   Python-interop story first.
2. **The checked-correctness niche** (the Julia-SciML adoption shape):
   reproducibility- and correctness-sensitive numerical work where a
   compiler-proven guarantee is the buying criterion — checked purity, checked
   laws, reproducible reductions, checked shapes. Small, sophisticated,
   defensible.
3. **Data pipeline components** (the Polars shape) — again library-first.
4. **Kernels and GPU, last** — after the language survives 1-3, and entered
   through the tinygrad-style delegation pattern, not raw PTX codegen.

**The probe ladder (all cheap, none roadmap-authorized):** make the Phase 10
high-intensity kernel a GEMM or quantized attention block (already permitted
by the completion set — the first ML probe is free); a shape-contract MLP
forward pass exercising const generics + `requires` as tensor types; the
reproducible-reduction demo when Phase 11's reduction forms land; the Enzyme
spike — *rescoped by the research*: Enzyme's tested range trails current LLVM
majors, Rust's `std::autodiff` is still nightly-only after 1.5+ years of
toolchain plumbing, and Enzyme.jl's dominant failure mode is uncatchable LLVM
assertions (in direct tension with the no-crash trust posture) — so the spike
pins the LLVM major Enzyme actually tracks, expects toolchain pain, and
treats crash behavior as the primary observable; "restricted mutation
simplifies reverse-mode AD" is an engineering hypothesis consistent with
Enzyme's alias-analysis-driven design, not settled literature.

## 9. The embedded direction (owner intent stated 2026-07-31; researched same day)

The owner's stated view: real-time requirements are today fulfilled by testing
rather than the toolchain, Whitefoot's timing analyzability could change that,
and Whitefoot is in general a better choice than C for embedded development. A
seven-cluster research pass (embedded Rust/Ferrocene, SPARK/Ada, WCET state of
the art, RTOS certification economics, the regulatory tailwind, bare-metal
effort calibration, checked ISR models) verified the clusters. Everything here
is SPECULATIVE or OPEN — no roadmap standing.

**Why the fit is structurally better than ML's.** The runtime surface
(write/abort/malloc/free, zero dependencies, no GC, no unwinding) is already
embedded-shaped, and a no-heap subset is signature-visible today through
`allocates(heap)` absence. The closed compilation unit matches what firmware
is. And the deepest claim survived scrutiny with literature behind it: the
WCET field attributes C's static-timing intractability to exactly three
compounding problems — indirect-call resolution, loop-bound inference, and
aliasing — and **Whitefoot removes the first entirely by construction** (no
function pointers, closures, or dynamic dispatch anywhere) and structurally
constrains the other two. GNATstack, the shipping prior art for compile-time
stack bounds, names recursion and indirect calls as its own inaccuracy
sources; Whitefoot forbids one and can bound the other, so tighter-than-SPARK
static bounds are plausible. STOR-6's frame computation plus the fully static
call graph is most of a stack-bound certificate already.

**The premise-level finding that outranks the direction itself.** arXiv
2607.14340 (2026-07-15, "The Prover Is the Judge") had AI agents build
verified security software in Ada/SPARK: supervision cost fell **20-40x**
versus hand verification — the first third-party measurement of this
project's founding bet — while agents were observed **attempting to bypass
weak or incomplete checks rather than write correct code**, and a 2025 study
(Marmaragan, arXiv 2502.07728) found GPT-4o re-deriving already-proven SPARK
annotations only 50.7% of the time. Together: the AI+prover economics are
real and large; the burden does not dissolve but relocates to specification
strength and gaming resistance. This is external validation that W3
cheat-proofness and GATE-1-style gaming defenses are the load-bearing design
choices, and it belongs to the whole ledger, not just this section.

**What the research corrected (recorded, not dropped):**

- **"Move WCET from testing into the toolchain" overstates the market.**
  Recent DO-178C-aligned guidance trends toward *more* on-target measurement,
  not less; static bounds are structurally pessimistic; and AbsInt's aiT
  already does static WCET on compiled C binaries across the Cortex-M/R/A
  range. The defensible claim is narrower: *analyzable-by-construction inputs
  and tighter static bounds, as a complement to measurement* — not its
  replacement. SCADE+aiT proves the toolchain-as-timing-evidence model
  commercially at DAL-A, but via a synchronous dataflow language with a
  TQL-1-qualified generator — reaching that bar also means constraining
  concurrency Ravenscar-style, not just control flow.
- **"Cache-less MCU determinism" is core-class-dependent.** True for
  M0/M0+/M3-class parts; false for M7 (6-stage dual-issue, branch prediction,
  caches — vendor guidance says pin critical code to TCM). The timing pitch
  targets small cores or mandates TCM discipline; and the strongest academic
  analyzable-by-construction results pair the language with time-predictable
  *processors* (Patmos/PRET), a ceiling no language design removes on
  commodity silicon.
- **`clang -O2` is a named enemy for the third time.** Optimization is a
  documented, unsolved tension with WCET bounds (as it is with constant-time
  preservation and with check-motion trap attribution). "The optimizer
  contract" is now a recurring architectural theme across three separate
  ambitions; whatever form it takes, it is one problem, not three.
- **The regulatory-tailwind pillar collapses on inspection.** None of EU CRA
  (substantive obligations 2027-12), CISA guidance (voluntary, politically in
  limbo, explicitly accepts hardware mitigations as an alternative to language
  change), FDA 524B, or UK PSTI names a memory-safe language; all are
  process/outcome regulations a disciplined MISRA-C shop satisfies. And Rust
  adoption itself is reported stalling in 2026 (TIOBE #13→#16, talent-pool
  commentary). Regulation is background context, never a pillar.
- **Bare-metal is more work than "modest."** No bundled libc for
  `*-none-eabi`; linker script, vector table, startup, and a reimplemented
  runtime (static/arena allocation for `buffer`, trap-to-UART/HardFault
  instead of abort-to-stderr) are all real items; Rust's bare-metal Thumb
  targets reached a Tier-2 *proposal* only in 2026-04 after a decade of
  investment. Monomorphization bloat is a documented embedded-Rust problem
  class (LCTES'22), unquantified for Whitefoot — the 16-64KB M0 flash tier is
  unproven territory. A blinky-class spike is reachable; "general embedded" is
  a size-budget program of work.
- **Volatile is not a detail.** LLVM's volatile guarantee orders volatile
  against volatile only; the miscompilation history (Eide & Regehr) is real.
  MMIO must enter as a checked operation family with its own rules — which is
  exactly what the operation-table design is for, but it is spec work, not a
  convention.
- **Certification economics are brutal and the escape hatches matter more.**
  CompCert — with a full correctness proof — took ~20 years to its first real
  avionics qualification credit (early 2026); Zephyr, well-funded, is still
  mid-process on IEC 61508; Ferrocene needed a dedicated company and now holds
  ASIL D/SIL 3/Class C with a certified core subset and shipping automotive
  users. A research compiler does not chase that. The underused opening: DO-330
  lets an unqualified tool's output be independently verified downstream, and
  ISO 26262 TCL1 needs no qualification at all — **Whitefoot's
  checked-runtime, no-unsafe design is itself an argument for landing in the
  low-tool-confidence bucket**, which no one has written down as a strategy.
- **The adversary calibration.** Ferrocene + embedded Rust is the real
  competitor and it ships in cars — but its practitioners' top pains are
  ecosystem churn, unqualified async, and wireless stacks, *not* "too much
  unsafe," so a pitch centered on HAL-layer unsafe answers a problem that
  community ranks low. Two counterweights: safety-critical Rust teams
  themselves avoid third-party crates and hand-roll at high ASIL (so
  Whitefoot's no-ecosystem position is less disqualifying here than in ML),
  and no clean unsafe-density statistic exists (treat it as a measurement to
  make, not a fact to cite). RTIC proves the checked-ISR direction (SRP:
  race-freedom, deadlock-freedom, bounded blocking, compile-time) — but its
  core is an `unsafe` trusted kernel, so Whitefoot's version is either a
  compiler-verified primitive or, if D16's deferred boundary is adopted, an
  explicit trusted project kernel; D17 is the long-term proof path out of that
  boundary. And
  the minimal sound single-core model is small: Cortex-M does not reorder
  memory, so priority masking plus compiler-fence discipline suffices — do
  not build a C11 atomics vocabulary for a problem single-core silicon does
  not have. seL4's history warns that formal rigor alone does not drive
  adoption; no documented incident-prevention track record exists for any
  checked embedded model — the benefit argument stays design-time.

**The wedge ranking that survives:**

1. **The unregulated long tail** (IoT, industrial, the firmware nobody wants
   to write): no certification gate, C-dominated, quality notoriously poor —
   and the AI-writer floor story is the differentiator Rust does not have
   (its stall is talent economics; Whitefoot's writer is not hired).
2. **Analyzable-by-construction instruments**: the stack-bound certificate
   (near-term assembly of existing machinery) and later tighter WCET inputs
   on small cores — sold as evidence generators beside measurement, in the
   resource-certificate mold.
3. **The checked ISR mini-model** — plausibly Phase 11's simplest real
   consumer (masking + compiler-fence, priority ceilings as checked facts;
   the ISSTA 2023 interrupt-concurrency bug study is the peer-reviewed pain
   evidence).
4. **Certified verticals** — a decade-scale horizon entered, if ever, through
   the TCL1/DO-330 independent-verification argument, never head-on.

**The probe ladder (none roadmap-authorized):** a bare-metal blinky spike
(thumbv7em triple, linker script, static-allocation runtime, trap-to-UART) to
measure the real toolchain cost; a monomorphization size measurement of the
existing corpus at `-Oz`/gc-sections (cheap, uses current programs, answers
the M0-tier question); the MMIO operation-family design note; the stack-bound
certificate assembled from STOR-6 plus the static call graph (valuable on
hosted targets too); the ISR model riding Phase 11. FFI surfaces again —
vendor SDK and RTOS interop — making it three ambitions now queued behind the
§14 gated family.

## 10. The cost column

What the trade spends, with the honest numbers where they exist.

- **Human writability and readability** — spent deliberately (D0); trusted-base
  auditability is the sole exception (D0a/R5).
- **The spec/teaching token budget is the real scarce resource.** Measured
  crisis: catalog drafts projected ~65.5k tokens against a ≤40k target — every
  design stance undercounted its own spec mass at first pass; resolved by a
  tiered teaching layer to ~40–46k.
- **First-shot correctness has never cleared the bar without the repair
  loop.** Writability trials: 45%, 26%, 26% first-shot across rounds; 70%
  (14/20) only with one diagnostic-repair revision — and the trial's feedback
  text was "likely higher quality than real compiler diagnostics will be."
- **Over-rejection tax:** 5.5–7.2% of would-compile programs rejected by the
  conservative checker (measured; no tolerance threshold ever set).
- **Verbosity:** base64 is ~90 lines where C writes 15; the 1.35–1.48x token
  premium is an unvalidated regex-proxy number; the true generation-cost
  magnitude has never been measured.
- **The FFI wall carries all displaced boundary pressure.** The risk `unsafe`
  used to absorb migrates entirely to one gated chokepoint.
- **Gate-gaming replaces unsafe-abuse.** A stuck writer's cheat path moves to
  over-restrictive `requires` bounds, checked-op spam, vacuous rationale at
  approval gates; "weak-invariant equilibrium under lint pressure" is a named
  risk. The defenses (GATE-1, refuter, floor audit) exist but the arms race is
  permanent.
- **Same-pass error correlation** undermines every redundancy-as-checksum
  mechanism; unresolved.
- **Repair-loop economics under trap=abort** (process death per runtime
  diagnostic) — unmeasured; explicitly allowed to reopen design if too slow.
- **Determinism's bill:** early-exit search pays (SISAL Loop 16: 60% behind
  Fortran under a determinacy guarantee).
- **Trusted-surface size resists shrinking:** ~15–24k trusted LOC across ten
  kernels after the D16 cut — "still rivals the 90-rule kernel spec."
- **Missing SIMD story** blocks the speed half of several artifact claims;
  scalar Whitefoot loses to hand-vectorized incumbents until vector types or
  reliable autovectorization land.
- **Two premise arguments collide:** "verbosity is free" (maximal
  proof-carrying annotation, the SPARK reprice) versus "spec mass is the
  budget" (every annotation vocabulary costs teaching tokens). The collision
  forced the retreat from mandatory proof-carrying source; it will recur.
- **Anti-human cuts can harm the writer too:** removing comments may remove
  the model's own reasoning scaffold at generation time — flagged, never
  measured.
- **Eager boolean operators** are a live W1 trap: the natural translation of
  `i < len && arr[i]` traps at runtime under eager `band`.
- **"AI stability" is a named comparison axis with zero evidence anywhere** —
  ruled OPEN for every capability candidate; no AI-generation evidence was
  ever produced for any of them.
- *(Closed since recorded:)* the positional-construction transposition hole —
  flagged as a live violation of "errs toward rejection" — was closed by
  GRAM-8/10/11 named-in-declared-order forms in the current spec.

## 11. The killed list

Do not re-propose without naming the reason that lapsed.

- **Regulation as an embedded-adoption pillar** — no regime (CRA, CISA, FDA
  524B, PSTI) mandates or names memory-safe languages; all are process
  regulations a MISRA-C shop satisfies; CISA's push is voluntary and in
  political limbo (researched 2026-07-31). Background context only.
- **Near-term certified-vertical targeting** — CompCert's ~20-year path to a
  first avionics credit and Zephyr's still-incomplete IEC 61508 effort set the
  scale; entry, if ever, is via the TCL1/DO-330 independent-verification
  argument (researched 2026-07-31).
- **Stale evidence, do not cite:** the NVIDIA/SPARK firmware "win" (a 2019
  announcement of intent, no confirmed completion) and the "40% SPARK adoption
  growth" figure (traces to a content-farm article, no primary source).

- **The pickle/safetensors security wedge** — safetensors already exists, is
  PyTorch-Foundation-governed, and the residual 2026 CVEs are an adoption
  problem a new language cannot move (researched 2026-07-31).
- **The tokenizer wedge** — already cashed in by Rust; the 2026 frontier is
  Rust-vs-Rust SIMD/caching work, so the marginal language win is near zero
  (researched 2026-07-31).
- **Kernel-layer safety as an ML selling point** — llama.cpp, the dominant
  edge-inference engine, shows no visible appetite to leave C/C++ at the
  kernel layer; the validated safety wedge is the orchestration layer
  (researched 2026-07-31).

- **Writer-stated performance hints** (`likely`/`cold`) — unverifiable
  data-habit assertions; measured beats asserted (R1).
- **Mandatory SPARK-grade proof-carrying source** — non-canonicalizable proof
  surface breaks one-spelling; economics falsified by the spec-mass budget.
  What survives is narrowly scoped proof evidence for a named authority; D17
  commits specifically to the long-term representation-invariant lane, not to
  a mandatory general-purpose proof language.
- **Lazy/citation-database teaching** — removes the cost pressure that keeps
  the spec small; is itself uncounted spec mass.
- **Rust-shaped grammar as default** — D3 guard; familiarity may live in the
  lexicon only, and even that requires the alien-rendering falsifier.
- **Any writer-emittable unsafe/assume/trust construct** — categorical; no
  advocate ever defended one.
- **N-copy instead of generics** — each rewrite is a fresh hallucination
  opportunity.
- **Auto-discovered parallelism** — four decades of prior art extracted
  parallelism correctly and lost to granularity; survives only as the
  declared, verified form (Phase 11).
- **Candidate A** (proof-indexed resource calculus) — proof burden unsuitable
  for AI-written first-green work; largest forgeable-authority surface.
- **Candidate C as sole route** — mirror-image AI cost: locally easy APIs,
  globally costly family choices; witness-fitting vulnerability.
- **B-Graphs** — owner veto; local protocol descriptions shade into arbitrary
  writer invariants (smuggled proof authority).
- **Whole-compiler resource profiles, artifact replay, second verifiers,
  release machinery** — the four-step retreat recorded in the toolchain tree;
  research-compiler scope instead.
- **Generational pool for the compiler's own AST** — per-access tax
  legitimizing the recycle idiom; region arena instead.
- **Raw-init privileges for the sealed stdlib** (`MaybeUninit`-class) — the
  stdlib gets no privilege an ordinary checked library couldn't earn by proof.
- **Unguarded `take(index)`** leaving a writer-readable hole — rejected in
  general form; the §5 take/replace design must not reintroduce it.
- **Crypto/checksum interposition headlines (D7c)** — killed by a content-
  filter false positive, not on merits; constant-time is back on the deferred
  list, and this history is the reason its framing needs care.

## 12. Standing tensions — reread these every visit

1. **The teachability bet is unvalidated.** The foundational claim — an alien
   but regular in-context language beats a familiar one for machine writers —
   has a designed, unrun experiment program behind it and first-shot trial
   numbers below the bar. The floor reframe reduces exposure (the checker, not
   the model, carries the claim) but does not close it.
2. **B-Strata never got its verdict.** D14 mandated STRATA-YES/NO; D15 withdrew
   the mandate a day later; the successor three-tier catalog reused one
   ingredient. The question "is there a small project-independent semantic
   basis?" is open, not answered.
3. **Take/replace vs sealed kernels** — the deepest pending language decision
   (§5 affine mutation), deliberately deferred to the moment of maximum
   evidence; Hylo (mutable value semantics) and Austral (linear types) are the
   closest living answers and should be read before drafting.
4. **Interior mutability has no story.** No `UnsafeCell` analog exists or is
   planned; memo caches and shared counters route through `&uniq` parameters
   or patterns. Whether that holds at scale is undecided.
5. **Drop authorship** (writer-visible vs compiler-derived free) was left
   explicitly undecided in the GC-replacement verdict; current practice is
   compiler-derived; unrevisited.
6. **The redundancy-checksum independence problem** (§8) touches every
   mechanism that cross-checks writer statements against derivations.
7. **Sealed-kernel trust vs the no-trust brand.** The archived estimate of ten
   kernels at 15–24k LOC would be a real trusted base if that catalog is
   adopted; D17's proof lane is the announced exit, and its feasibility is
   unproven.
8. **The spec-mass budget governs everything.** Every new fact vocabulary
   (laws, refinements, secrecy, parallelism, budgets) pays teaching tokens;
   the 40k ceiling is the quiet veto over most of §7.
9. **W1's floor reading needs its constitution amendment honored in
   practice:** model runs are mistake generators and calibration only — any
   future gate that quietly reintroduces a model score violates the 2026-07-27
   ruling.
10. **The interactivity question (added 2026-07-31).** Agents currently exploit
   notebook/REPL execute-observe loops rather than bypassing them, and Julia
   shows iteration-latency reputations outlive their fixes. If Whitefoot is
   ever to carry the ML direction (§8), compile-run-trap latency competitive
   with a REPL turn becomes a first-class requirement — which also re-weights
   the content-addressed-caching entry in §4 from "unpriced cost" toward
   "ML-critical instrument."
11. **Checker-gaming now has published third-party evidence (added
   2026-07-31).** The July 2026 Ada/SPARK agent study documented agents
   bypassing weak checks rather than solving problems — the exact behavior W3
   and GATE-1 were designed against, observed in the wild with a 20-40x
   supervision-cost win alongside it. The design consequence: the checker
   must be uniformly strong, because agents route through the weakest gate;
   every new capability's verification story should be reviewed against this
   published failure mode, not just against honest-writer assumptions.
12. **"The optimizer contract" recurs (added 2026-07-31).** Three ambitions
   now collide with the same enemy: constant-time preservation, check-motion
   trap attribution, and WCET analyzability all need a stated contract about
   what `clang -O2` may do to emitted structure. One problem, three
   costumes — whichever ambition first forces a solution should solve it in
   the general form.

## Sources

Founding directives and debates: `archive/governance/directives.md`,
`archive/governance/decisions/` (v0.0–v0.6 volumes), round-2/3/4 debate
records under `archive/research/`. Capability era:
`archive/research/systems-performance-coverage/`, the parked
minimal-systems-capability line. Live: `docs/constitution.md`,
`docs/why-whitefoot.md`, `docs/patterns.md`, `docs/ideas.md`,
`docs/roadmap.md` (Phases 10–11), `mcts_mem/`. Idea banks:
`research/notes/headline-artifact-brainstorm.md` (= the 39-idea JSON),
`research/notes/missing-research-backlog.jsonl`. Section 7 is original to this
file, 2026-07-28. Section 8 rests on an eight-cluster external web-research
pass run 2026-07-31 (Mojo, Julia, GPU routes, Enzyme/autodiff, shape checking,
determinism demand, safe-language ML systems layer, JAX purity + AI-written-ML
evidence); Section 9 on a seven-cluster pass the same day (embedded
Rust/Ferrocene, SPARK/Ada, WCET, RTOS certification economics, regulation,
bare-metal effort, checked ISR models). Dated citations are inline, and each
pass's refuted briefing claims are recorded rather than silently dropped.
