# Whitefoot Direction Outline

Status: CANONICAL DIRECTION OUTLINE
Revision: 14

The active language authority is
[`spec/kernel-spec-v0.19.md`](../spec/kernel-spec-v0.19.md), SHA-256
`01fb10d2d61cc87cce72cc98071eda98c7411fdc95af4ef29b79ac9a49cb5398`.
Released numbered specifications are immutable. The current execution proposal
is [`docs/current-plan.md`](current-plan.md), project law is the
[`Constitution`](constitution.md), and the operational process is
[`WORKFLOW.md`](WORKFLOW.md).

## How to read this outline

This file is the owner-facing map of Whitefoot's live directions. It answers:

- what the project can already do;
- which ideas, requirements, and open questions belong together;
- what evidence exists and whether it is current or historical; and
- what is missing before a direction can advance.

It does **not** choose the current execution order. Candidate projects determine
when a direction matters. `current-plan.md` contains the only execution
proposal; after owner selection, it becomes the rolling plan for that one
milestone. A project can expose a missing capability, but cannot by itself
change the language or justify a project-shaped compiler special case.

Each item has one canonical home. Other directions link to its ID instead of
copying its status. Tags are scanning aids. The `Current` sentence is this
outline's canonical status summary, but the linked specification, code/tests,
RESULTS, project law, or design memory remains authority for the underlying
fact:

- `[current: ...]` — some stated part exists now; the suffix names its factual
  basis or exact boundary, and never means the whole item is complete;
- `[seeded]` — a live design or writer form exists but is not normative;
- `[historical: measured]` — a RESULTS record exists, but not for the current
  compiler or current project boundary;
- `[research-only]` — investigation exists without a production selection;
- `[speculative]` — an idea has not yet passed a bounded investigation; and
- `[candidate]` or `[candidate: later]` — a validation-project class, with the
  latter carrying substantial prerequisites; and
- `[next: ...]` or `[parked]` — the next evidence gate, not implementation
  authorization.

The file is updated in place. Increment `Revision` when an item's goal,
evidence-backed current state, next gate, or candidate-project disposition
changes. Git is the version history; do not create versioned copies of this
file. Detailed semantics, measurements, design rationale, and implementation
inventories remain in their canonical owners and are linked rather than copied.

## Current baseline

`[current: spec v0.19]` `[current: safe-Rust compiler]`

Whitefoot has one normal path from canonical source through resolution,
semantic and ownership checking, checked program, typed CFG IR, target
qualification, LLVM, and host execution on supported aarch64 and x86-64
macOS/Linux targets. Valid language the compiler has not implemented stops as
unsupported rather than invalid source.

The compiler implements enough scalar, nominal, generic, storage, borrow,
contract, cleanup, and program-level behavior to begin external validation, but
not the entire active language. The exact implementation inventory and gaps
belong in the [compiler README](../compiler/README.md); the
[v0.19 specification](../spec/kernel-spec-v0.19.md) remains semantic authority.
The first-slice system interface compiles and runs end-to-end on the native
macOS/Linux command target; the §9.1 cost and §12.2 hostile gates (task 0016)
remain ahead of any performance claim.
Which gap matters next is selected by a project, never by checklist length.

## Dependency rules

- CAND-1 records the completed flagship selection. CAND-8 supplies the current
  ripgrep project pressure; its mapped direction items inform the plan, but
  none authorizes work by itself.
- PERF-1 establishes ordinary code quality before a new optimizer fact or
  strategy is blamed or credited.
- Every production fact consumer in PROOF-1 through PROOF-4 and PROOF-7 depends
  on VERIFY-3. PROOF-2 depends on PROOF-5 only for a `willreturn`-class claim,
  not for memory-effect attributes.
- PAR-1 selects a source construct only after CAND-8 profiling exposes concrete
  parallel work; PAR-2 through PAR-4 cannot preselect proof rules, reductions,
  or a runtime before that evidence.
- STORE-2 must expose a concrete unsolved representation privilege before
  PROOF-6 can enter a plan.
- TARGET-2 through TARGET-4 depend on BOUND-1 whenever their authentic milestone
  crosses the closed compilation-unit boundary. PAR-4 and BOUND-1 must agree on
  resource transfer, waiting, cancellation, and runtime thread authority.
- APP-1 depends on BOUND-2 when its selected component requires opaque foreign
  code rather than a Whitefoot or compiler-owned system provider.

## Proof and optimizer facts

Serves Constitution P0, W3, T1, and T2: useful facts must improve code without
creating writer trust or weakening the checked safety envelope.

### PROOF-1 — Relational bounds proofs and check elision

`[current: compiler]` `[historical: measured]` `[next: project pressure]`

- **Goal:** remove a required bounds check only when a deterministic proof
  establishes the exact proposition that makes the operation safe.
- **Current:** the compiler executes concrete `requires` prologues but creates
  no `llvm.assume` and removes no downstream check. A historical base64 study
  measured a bounded proof consumer; it is not current compiler capability.
- **Missing / next:** a selected workload must first show retained-check
  pressure; then build one finite proof family with exact producers,
  invalidators, negative canaries, facts-off identity, and attribution.
- **Facts:** [compiler `requires` boundary](../compiler/README.md) ·
  [historical base64 result](../research/experiments/port-study/base64/RESULTS.md).

### PROOF-2 — Effect-derived optimizer facts

`[current: spec]` `[current: compiler]` `[historical: measured]` `[next: research]`

- **Goal:** safely project exact effect rows into backend facts at opaque call
  boundaries.
- **Current:** v0.17 and the compiler check reads, writes, allocation, and traps
  in both directions and project effects through storage origins. The backend
  deliberately emits no `willreturn`; historical attribute results used the
  retired compiler and ABI.
- **Missing / next:** re-derive a sound mapping for the active ABI against a
  real opaque-boundary workload. `pure` is not totality, and trapping behavior
  must remain observable.
- **Facts:** [historical effect result](../research/experiments/effect-attrs-channel/RESULTS.md) ·
  [current design memory](../mcts_mem/whitefoot/effects.md).

### PROOF-3 — Borrow-derived alias facts

`[current: compiler]` `[historical: measured]` `[next: project pressure]`

- **Goal:** turn checked uniqueness and provenance into useful backend alias
  information without runtime guards or writer promises.
- **Current:** the checker owns `&uniq`, resolved places, and finite slice-origin
  facts; the backend does not claim production alias-scope or `noalias`
  emission. Historical short-trip kernels measured a win and long-trip parity.
- **Missing / next:** first demonstrate alias pressure that LLVM cannot recover
  in a selected project, then add one active-backend consumer with hostile
  overlap tests and current-compiler attribution.
- **Facts:** [historical alias result](../research/experiments/scoped-alias-channel/RESULTS.md) ·
  [directional frequency study](../research/experiments/frequency-study/RESULTS.md).

### PROOF-4 — Checked laws as transformation authority

`[current: spec]` `[current: compiler]` `[historical: measured]` `[next: project pressure]`

- **Goal:** let a discharged algebraic law authorize one exact transformation
  while false laws fail closed.
- **Current:** v0.17 and the compiler discharge the closed FN-4 law table for
  source acceptance. Lowering ignores that metadata. A historical reassociation
  experiment measured 3.3x and refuted a false signed-saturating law.
- **Missing / next:** choose a real reduction workload and separately define the
  optimizer proposition and consequence; acceptance evidence does not
  automatically become optimization evidence.
- **Facts:** [historical checked-law result](../research/experiments/checked-law-channel/RESULTS.md) ·
  [current contract implementation](../compiler/README.md).

### PROOF-5 — Derived totality

`[current: spec]` `[research-only]` `[parked]`

- **Goal:** prove termination for a useful decidable fragment when a consumer
  needs a `willreturn`-class fact or a finite execution bound.
- **Current:** v0.17 explicitly has no termination checker; `pure` says nothing
  about return. Pure-row totality is a rejected design.
- **Missing / next:** reopen only for a selected effect optimization, embedded
  bound, or other concrete consumer; define the smallest fragment and its
  rejection boundary before implementation.
- **Facts:** v0.17 `EFF-3` · [totality design decision](../mcts_mem/whitefoot/effects/derived-totality.md).

### PROOF-6 — Proof-gated representation authority (D17)

`[current: project law]` `[research-only]` `[parked]`

- **Goal:** grant one narrow representation privilege only when a deterministic
  checker verifies the exact implementation invariants and obligations that
  make it safe, with no writer-accessible trust escape.
- **Current:** the Constitution selects this long-term lane. v0.17 has no proof
  language, privilege vocabulary, partial-initialization path, or production
  checker.
- **Missing / next:** a real project must first expose a representation blocker
  that ordinary checked mechanisms cannot solve; then select one minimal
  privilege/invariant pair rather than a general proof system.
- **Facts:** [Constitution D17](constitution.md) ·
  [archive-promotion placement](../research/archive-promotion-audit.md#1-d17-placement-completed).

### PROOF-7 — Verified strategy-selecting lowering

`[historical: measured]` `[speculative]` `[next: project pressure]`

- **Goal:** use a checked structural fact to select a faster algorithmic machine
  shape, not merely remove a guard from the writer's literal loop.
- **Current:** historical DEFLATE prototypes explored periodic overlap expansion
  and a guarded bit window; threaded interpreter dispatch and proof-guided
  autotuning remain idea-stage. None is a current compiler fact consumer.
- **Missing / next:** a selected workload must defeat ordinary lowering first;
  then bind one verified predicate to one consequence, portable fallback,
  target condition, differential oracle, and stop condition.
- **Facts:** [DEFLATE design handoff](../research/experiments/zlib-core-kernels/DESIGN-HANDOFF.md) ·
  [proof-guided autotuning](ideas.md#proof-guided-autotuning).

## Verification and compiler trust

Serves W3, T1, and T2: current claims must survive independent, hostile, and
facts-off evidence rather than trust in the compiler or writer.

### VERIFY-1 — Checked safety envelope in real programs

`[current: spec]` `[current: compiler]` `[next: project validation]`

- **Goal:** make memory corruption, data races, uninitialized reads, and silent
  overflow unrepresentable across success, failure, and cleanup paths.
- **Current:** the active language forbids writer trust; the compiler retains
  hazardous-operation checks, exact trap records, affine cleanup, checked
  indexing, and allocation-domain guards on its implemented path.
- **Missing / next:** validate malformed input and language-level failures
  separately from target or allocator resource failure, then exercise partial
  results, transfer, and teardown in the first selected medium project.
- **Facts:** [Constitution](constitution.md) · [compiler README](../compiler/README.md).

### VERIFY-2 — Execute the conformance corpus against the compiler

`[current: conformance corpus]` `[next: implement when selected]`

- **Goal:** compare compiler behavior with compiler-independent v0.17
  expectations through the normal command path.
- **Current:** the native execution adapter is wired (task 0014):
  `make conformance-run` compiles and runs every case through the real
  compiler. First full run: 242 pass / 123 pre-existing failures in four
  recorded buckets (45 pre-semantic rejections without rule ids; 41
  protected sources that are incomplete units; 35 runnable overclaims; 2
  real divergences) awaiting owner rulings; the corpus gate lane is
  ignored-with-reason until they land.
- **Missing / next:** distinguish correct execution, valid-but-unsupported,
  wrong rejection, crash, and trap mismatch. Any existing expectation or
  status weakening remains owner-protected.
- **Facts:** [conformance corpus](../tests/conformance) · [workflow](WORKFLOW.md).

### VERIFY-3 — Facts-on/facts-off differential trust

`[historical: measured]` `[next: with first fact consumer]`

- **Goal:** prove that an optional optimizer fact changes only justified code
  shape, never acceptance, outputs, required traps, or cleanup.
- **Current:** historical experiments have local controls; the current compiler
  has no production check-elision fact family and therefore no global claim.
- **Missing / next:** the first fact consumer must ship with legal-program
  differential generation, hostile premise mutation, output and trap identity,
  and attribution before timing.
- **Facts:** [experiment index](../research/experiments/README.md) ·
  [fact-channel design memory](../mcts_mem/whitefoot/fact-channels.md).

### VERIFY-4 — Deterministic and reproducible artifacts

`[current: bounded determinism]` `[next: real consumer]`

- **Goal:** make source, diagnostics, checked state, and emitted outputs
  reproducible at the boundary a real build or audit consumer needs.
- **Current:** source form, parsing, diagnostics, and selected output tests are
  deterministic; no complete cross-machine or whole-compiler object
  reproducibility claim exists.
- **Missing / next:** name a concrete caching, distribution, audit, or
  multi-backend consumer before adding stable artifacts, receipts, or replay.
- **Facts:** [compiler tests and boundary](../compiler/README.md) ·
  [rejected premature artifact architecture](../mcts_mem/whitefoot/toolchain.alt/product-scale-checked-artifact-toolchain.md).

## Performance floor and writer shape

Serves P0 and W1: ordinary accepted source should be forced toward a fast shape,
and every slower-but-accepted divergence becomes a measured finding.

### PERF-1 — Ordinary lowering and baseline code quality

`[current: credited fused-scan win 0.65→0.75]` `[current: attributed latency floor]`
`[next: check-aware wide-scan lowering question]`

- **Goal:** make ordinary checked source competitive before relying on a new
  proof channel, special writer trick, or project-specific lowering.
- **Current:** the compiler has one conservative LLVM path and executable
  program witnesses, but no current medium-project comparison of scalar code
  shape, retained checks, vectorization, and target output. `RG-BASE` completed
  one correctness-green upstream selection attempt; host cache-position noise
  defeated its precision gate, so it selected no comparator and made no
  performance claim. Its medians remain a development-cost table, not a
  baseline.
- **Missing / next:** require every newly runnable `wfgrep` slice to pass its
  correctness oracle and scoped cost-shape or performance gate before adding
  downstream behavior. Attribute each material loss to algorithm, required
  check, source shape, compiler lowering, LLVM recovery, runtime, I/O, output,
  or target, resolve its owning layer generally, and rerun the same slice. The
  full paired suite is reserved for a later public-claim candidate rather than
  the edit loop.
- **Facts:** [compiler backend boundary](../compiler/README.md) ·
  [historical DEFLATE result](../research/experiments/zlib-core-kernels/RESULTS.md) ·
  [ripgrep flagship frame](../research/notes/ripgrep-flagship-frame.md).

### FLOOR-1 — Canonical source and constrained control shape

`[current: spec]` `[current: compiler]` `[next: project validation]`

- **Goal:** remove accidental slow alternatives so the ordinary accepted shape
  is a strong default for an AI writer.
- **Current:** v0.17 fixes canonical bytes, flat ANF, one `loop` form, and closed
  statement/value branching; the compiler implements those forms. This alone
  is not a performance guarantee.
- **Missing / next:** compare current AI-written project code with a measured
  expert reference shape and identify accepted but materially slower forms.
- **Facts:** v0.17 `FORM-*` and `GRAM-*` · [floor rationale](why-whitefoot.md).

### FLOOR-2 — Closed, taught pattern catalog

`[seeded]` `[next: project validation]`

- **Goal:** teach a small set of patterns that are both expressive enough for
  real systems work and aligned with fast machine shapes.
- **Current:** `docs/patterns.md` contains ten entries of mixed maturity plus a
  known-gaps list. Some have measurements or current witnesses; P5 is deferred,
  P6 is validation-only, and the catalog is not normative language doctrine.
- **Missing / next:** validate individual patterns in candidate projects;
  promote a new card or rejection proposal only after observing a recurring
  slower-but-accepted or currently inexpressible shape.
- **Facts:** [pattern catalog](patterns.md) · [pattern design memory](../mcts_mem/whitefoot/pattern-doctrine.md).

### FLOOR-3 — Project floor audit

`[historical: measured]` `[next: selected project]`

- **Goal:** classify the difference between an AI's first correct implementation
  and the best known project shape as equal, lowering gap, pattern gap, or
  language gap.
- **Current:** historical first-green studies measured two narrow wins against
  shipped Rust libraries, but used the retired compiler and do not establish a
  general or current floor.
- **Missing / next:** run the audit on the current compiler and writer material
  inside a medium project; the checker outcome, not a model score, is the gate.
- **Facts:** [historical default-floor results](../research/experiments/default-floor/RESULTS.md).

### FLOOR-4 — Diagnostic repair loop

`[current: deterministic diagnostics]` `[next: measure]`

- **Goal:** give the AI deterministic, actionable failures that shorten the
  path from rejected source to a correct, efficient program.
- **Current:** v0.17 requires deterministic rule/location diagnostics and exact
  trap records; single-shot writability and repair effectiveness are not
  established.
- **Missing / next:** measure repair-to-green on real project failures and turn
  repeated confusion into a diagnostic or teaching defect.
- **Facts:** v0.17 `DIAG-*` · [honest limitation](why-whitefoot.md#part-vi-what-it-does-not-beat-and-what-is-not-yet-known).

## Storage, ownership, and representation

Serves P0, W1, W3, T1, and T2: useful data structures must retain safety and
optimizer facts without a writer-accessible escape or hidden pathological cost.

### STORE-1 — Borrow and provenance completeness

`[current: compiler]` `[next: project-selected gap]`

- **Goal:** express useful views and mutations while retaining exact ownership,
  origin, overlap, and effect information.
- **Current:** the compiler supports buffer/struct borrows, scoped child
  reborrows, direct slices, direct own returned slices, and — since task
  0024 — borrow-mode parameters, let-borrows, deref reads/writes, and
  matching through borrowed enums for scalar and enum content on one
  generalized address machinery. Returned reborrows are a recorded v0.19
  gap (OWN-6 defines child reborrows only in call-argument position);
  branch-produced loans and holder-derived slices remain absent.
- **Missing / next:** choose the smallest missing rule only after a real
  project cannot express its required access pattern. The 31-rule loan/freeze
  review candidate and older M1 model are parked evidence, not language
  authority or a ready implementation package; it vacated the v0.18 candidate
  slot for the BOUND-1 system-interface batch on 2026-08-05 (predates the
  wfgrep goal framing and would need re-derivation from a real blocker).
- **Facts:** [compiler borrow boundary](../compiler/README.md) ·
  [parked loan/freeze candidate](../governance/spec-evolution/parked-loan-freeze-candidate.md) ·
  [M1 placement](../research/archive-promotion-audit.md#2-keep-the-m1-loanfreeze-work-as-a-parked-candidate-not-a-rule-set).

### STORE-2 — Growth, replacement, occupancy, and identity

`[current: fixed storage only]` `[research-only]` `[parked]`

- **Goal:** support the storage transitions a real general-purpose structure
  needs without writer trust, leaks, stale identity, or hidden asymptotic cost.
- **Current:** arrays, fixed-length buffers, boxes, and an append-only SoA
  pattern exist. General growth, affine backing replacement, partial
  initialization, sparse occupancy, recyclable identity, and generational
  reuse are not selected production mechanisms.
- **Missing / next:** a reopening project must account separately for growth
  and replacement; move-out/failure/cleanup/destruction; partial initialization
  and occupancy; stable versus recyclable identity; invalidation and stale
  handles; and multi-place access/iteration/relocation under loans. First
  concrete reopening witness (2026-08-06): sequential `wfgrep` cannot grow its
  line buffer — `buffer<T>` has no in-place growth and STOR-1 rejects
  rebinding an affine place — so it carries a fixed maximum line length where
  real grep grows; see `docs/done/0015-sequential-wfgrep.md`.
- **Facts:** [promotion checklist](../research/archive-promotion-audit.md#4-storage-checklist-retained-as-direction-outline-reopening-input) ·
  [rejected owning-sequence experiment](../research/experiments/data-layout-owning-sequence/RESULTS.md).

### STORE-3 — Refined domains and automatic niches

`[speculative]` `[parked]`

- **Goal:** make valid-value domains machine-checked so layout can use genuine
  invalid bit patterns without writer-chosen sentinels.
- **Current:** existing narrow integer types are implemented; a general refined
  integer domain and automatic niche selection are idea-stage only.
- **Missing / next:** select a real bounded identifier or offset whose layout
  matters, then compare semantics, ABI, size, and runtime cost against the
  ordinary representation.
- **Facts:** [supporting idea](ideas.md#narrow-semantic-domains-and-automatic-niches).

## Parallelism and concurrency

Serves P0, T1, and T2: concurrency is useful only when checked non-interference
and failure semantics survive the runtime implementation.

### PAR-1 — Writer-declared, compiler-verified parallelism

`[research-only]` `[next: candidate project]`

- **Goal:** let the writer request a parallel construct while the compiler
  proves non-interference from checked places, effects, loans, and origins.
- **Current:** v0.17 has no thread construct or runtime. Research rejects the
  stronger claim that the compiler should discover profitable parallelism the
  writer did not express; the declared form remains plausible.
- **Missing / next:** choose a real parallel workload and define its source
  request, safety judgment, cost boundary, determinism posture, and baseline
  before language or runtime implementation.
- **Facts:** [auto-parallelism feasibility result](../research/experiments/auto-parallelism-feasibility/RESULTS.md).

### PAR-2 — Intra-object disjointness

`[research-only]` `[parked]`

- **Goal:** prove disjoint subranges or injective indexed writes when
  region-level separation is too coarse.
- **Current:** effects can separate storage origins but do not prove arbitrary
  element-level injectivity; there is no production `split_uniq` capability.
- **Missing / next:** a selected sequential or parallel project must first
  require this exact access pattern; then choose the smallest judgment and its
  lifetime across calls or recursion.
- **Facts:** [parallelism feasibility result](../research/experiments/auto-parallelism-feasibility/RESULTS.md) ·
  [pattern gaps](patterns.md#known-gaps-findings-not-yet-patterns).

### PAR-3 — Reductions, algebra, and trap selection

`[current: spec]` `[historical: measured]` `[next: project pressure]`

- **Goal:** parallelize only an exact algebraic domain whose result and failure
  semantics survive regrouping and concurrent eligibility.
- **Current:** FN-4 law discharge exists for acceptance; parallel reduction,
  floating reproducibility, and concurrent trap selection do not. Historical
  chunk-summary work found no Whitefoot-over-Rust delta.
- **Missing / next:** a real reduction workload must choose the integer/float
  domain, deterministic result, and trap rule before any transform.
- **Facts:** [historical chunk-summary result](../research/experiments/port-study/wc-chunk-summary/RESULTS.md).

### PAR-4 — Runtime, allocation, and dynamic fan-out

`[research-only]` `[parked]`

- **Goal:** execute selected parallel forms without hiding serialization,
  unbounded overhead, or an unexplained trusted runtime.
- **Current:** no runtime architecture is selected. The archive audit preserves
  one required witness: a runtime-count worker set whose workers share-read
  outer state; the old fixed-spawn answer has no authority.
- **Missing / next:** the PAR-1 project must exercise or deliberately reject
  that witness and measure allocation, scheduling, determinism, and absolute
  wall time; any OWN-11 change needs hostile soundness review.
- **Facts:** [dynamic fan-out placement](../research/archive-promotion-audit.md#3-dynamic-fan-out-retained-as-a-parallel-design-witness).

## Boundaries, targets, and deployment

Serves P0, W3, T1, T2, and R6: external usefulness and target reach may not
become alternate unchecked semantics or prematurely bind the whole toolchain.

### BOUND-1 — System capabilities and host integration

`[current: first slice complete end-to-end with standing cost gates]`
`[next: helper decomposition (system-type borrow parameters); later families on project pressure]`

- **Goal:** give command, service, and embedded program instances a
  coherent capability-based host boundary covering process context,
  filesystems, data streams, clocks, randomness, networking, waiting and
  cancellation, and future threads or tasks without ambient mutable authority
  or writer-defined trust.
- **Current:** the owner selected the dossier architecture on 2026-08-05 after
  a 31-issue adversarial review: exact typed entry inputs under a declared
  program kind, immutable values / shared capabilities / unique stateful
  resources over ordinary `own`/`&`/`&uniq`, exact `external` and `blocks`
  effects with conservative source ordering, operation-specific one-attempt
  I/O with portable error classes, lossless target paths, compiler-owned
  resource contracts with three completion policies, a Route C
  system-declaration domain (with the recorded fallback to a prelude
  extension if the syntactic conditional-visibility mechanism is declined),
  and static target qualification. v0.17 itself still has a fixed no-argument
  `main`, memory-only effect rows, and no compiler system path; the only
  ordinary external-I/O call is a private trap diagnostic.
- **Missing / next:** the first command slice is complete: v0.18/v0.19
  active, the compiler implements the whole surface end-to-end on the native
  macOS/Linux command target, sequential `wfgrep` passes its oracle, and the
  dossier §9.1 rows are standing machine gates (task 0016). Named
  implementation follow-up: borrow-mode parameters of system nominal types
  are an unsupported specified capability (task-0015 finding) forcing
  helper-free programs. Later families (clocks, network, async/wait,
  threads, child processes) remain additive true subsets on their own
  project pressure. Historically: v0.18 activated 2026-08-06 (`9768bae`)
  with the exact
  first-command-slice batch: 25 new rules, 13 modified, the Route C
  declaration domain, and the promoted native grammar. The compiler parses
  the whole surface and reports it as explicit unsupported capability;
  implementation proceeds through the eleven planned tasks 0006-0016 (front
  end, effects, checked IR, qualification and native lowering, deterministic
  test target, conformance execution, sequential `wfgrep`, cost and hostile
  gates), then returns to the frozen sequential `wfgrep` checkpoint. Later
  families (clocks, network, async/wait, threads, child processes) remain
  additive true subsets of the selected model and wait for their own project
  pressure.
- **Facts:** v0.17 `FN-7`, `EFF-1/2`, `PROG-1/2`, `CAP-1`, `GATE-1`, and
  `LEDGER-1` · [architecture dossier](../research/investigations/system-capability-architecture/DOSSIER.md) ·
  [review decision record](../research/investigations/system-capability-architecture/decisions.json) ·
  [WASI capability model](https://github.com/WebAssembly/WASI/blob/main/docs/Capabilities.md) ·
  [WASI 0.1–0.3 release lessons](https://wasi.dev/releases).

### BOUND-2 — Foreign ABI and opaque binaries

`[current: spec skeleton; compiler absent]` `[parked]`

- **Goal:** call an opaque foreign binary only through a gated signature whose
  ownership, layout, lifetime, effects, callbacks, failure, and trust
  obligations are complete, without making FFI an alternate unchecked
  language.
- **Current:** `GATE-1` and `LEDGER-1` reserve an opaque approved boundary, but
  no public ABI, import, callback, foreign-thread entry, loader, or compiler
  implementation exists. Project law prefers rewriting source-available code in
  Whitefoot.
- **Missing / next:** reopen only for an opaque binary that a selected project
  cannot credibly replace. Keep this distinct from BOUND-1's compiler-owned
  system provider; system calls do not by themselves justify general FFI.
- **Facts:** v0.17 `GATE-1` and `LEDGER-1` ·
  [safe capsule idea](ideas.md#safe-c-abi-capsules).

Migration tooling is supporting work under BOUND-2, not an independent language
authority; see the [C-to-Whitefoot assumption extractor](ideas.md#a-c-to-whitefoot-assumption-extractor).

### TARGET-1 — Portable and mutually checking backends

`[current: host LLVM only]` `[speculative]` `[parked]`

- **Goal:** reach a target or independent lowering oracle that the selected
  LLVM-host path cannot provide while preserving defined checked behavior.
- **Current:** the compiler emits selected-host LLVM. Portable C and
  multi-backend differential checking are supporting ideas, not current
  capability.
- **Missing / next:** an embedded, portability, or verification project must
  first show why current LLVM is insufficient; then lower a bounded corpus and
  compare values, traps, cleanup, code shape, and undefined-behavior risk.
- **Facts:** [portable C backend](ideas.md#a-portable-c-backend) ·
  [multiple backends](ideas.md#multiple-backends-as-mutual-oracles).

### TARGET-2 — Embedded and resource-bounded systems

`[research-only]` `[next: candidate research]`

- **Goal:** test whether Whitefoot's closed world, explicit effects, checked
  memory, and deterministic failures form a useful embedded/real-time basis.
- **Current:** there is no bare-metal target, MMIO family, ISR model, WCET
  checker, or resource certificate. The direction has a research synthesis,
  not an implementation selection.
- **Missing / next:** compare a bounded embedded exemplar with the compiler and
  runtime prerequisites it would force; start with target/runtime feasibility,
  not “general embedded support.”
- **Facts:** [embedded direction synthesis](bargain.md#9-the-embedded-direction-owner-intent-stated-2026-07-31-researched-same-day) ·
  [resource certificate idea](ideas.md#resource-certificates).

### TARGET-3 — Deployment evidence and policy

`[speculative]` `[parked]`

- **Goal:** let a concrete deployment or audit consumer inspect resource,
  effect, or check-elision evidence without granting that evidence authority it
  has not earned.
- **Current:** resource certificates, effect-derived sandbox policies, and
  optimization receipts are idea-stage; no stable artifact product exists.
- **Missing / next:** reopen only for a selected embedded, regulated, sandbox,
  or proof-consumer project and define one exact consumer before any schema.
- **Facts:** [resource certificates](ideas.md#resource-certificates) ·
  [sandbox policies](ideas.md#effect-derived-sandbox-policies) ·
  [optimization receipts](ideas.md#optimization-receipts).

### TARGET-4 — Constant-time secret-dependent behavior

`[speculative]` `[parked]`

- **Goal:** make a bounded class of secret-dependent branches, indices, and
  variable-latency operations rejectable, while preserving that property
  through the backend.
- **Current:** no `secret` type/effect, leakage model, backend contract, or
  validation path exists. The direction was deferred, not rejected on technical
  grounds.
- **Missing / next:** a selected cryptographic or embedded component must name
  the attacker observables, admitted operation set, target, optimizer contract,
  and independent leakage test before language design.
- **Facts:** [dated direction synthesis](bargain.md#5-shipping-what-the-artifacts-can-be).

### APP-1 — ML systems components

`[research-only]` `[next: candidate research]`

- **Goal:** test Whitefoot on a bounded ML systems component where checked
  shapes, effects, or proof-backed lowering provide a real advantage.
- **Current:** there is no Python boundary, tensor language, GPU backend, or ML
  runtime. Existing research identifies FFI and GPU integration as hard walls.
- **Missing / next:** select a component with a strong oracle and independent
  value; do not choose “rewrite an ML stack” or assume integration work away.
- **Facts:** [ML direction synthesis](bargain.md#8-the-ml-direction-owner-intent-stated-2026-07-27-researched-2026-07-31).

## Candidate validation projects

Ripgrep is the owner-selected umbrella project. Selection fixes the external
pressure source and the headline objective, **2x ripgrep**; it does not
authorize a favorable subset, a monolithic rewrite, or any particular
language, compiler, proof, runtime, or optimizer change. The rolling
`current-plan.md` still proposes one bounded evidence-producing step at a time,
and every step must state how it returns to the unchanged product comparison.

Owner framing (2026-08-05): the project's deliverable is what `wfgrep` proves
about the language's functional and performance ceiling — resolved general
capabilities, attributed wins, and honest negative results. The 2x claim
remains the pressure source and honesty anchor; shipping a finished tool is
not the completion condition. Every specification amendment on this path is
sourced from a need the frozen `wfgrep` slice actually exposed, then designed
as a complete capability rather than a wfgrep-minimal one. A language gap
exposed by a frozen slice is therefore a finding first; the owner-override
loop in `WORKFLOW.md` is its expected path, and the gap-accumulation rule
remains as the owner check-in, not as a presumption against the goal.

### CAND-1 — Select the first external validation project

`[current: completed — ripgrep selected]`

- **Goal:** choose a project with an immediately legible public result, low
  user trial cost, a strong oracle, and enough end-to-end pressure to expose
  general language, compiler, runtime, and machine-code needs.
- **Current:** the initial N1 screen advanced yyjson and LZ4 under a
  near-term-reachability gate. The owner subsequently made comparative
  performance and immediate tool adoption the primary public test and selected
  pinned ripgrep 15.2.0.
- **Missing / next:** none; `CAND-8` owns the live project sequence.
- **Facts:** [historical N1 shortlist](../research/notes/headline-artifact-shortlist.md) ·
  [ripgrep flagship frame](../research/notes/ripgrep-flagship-frame.md) ·
  [current executable programs](../compiler/README.md).

### CAND-8 — Ripgrep-compatible command-line search

`[selected flagship]` `[current: frame plus inconclusive upstream attempt]`
`[next: BOUND-1 system-capability architecture]`

- **Goal:** build a Whitefoot-written command-line search tool credible for
  ripgrep's primary line-oriented recursive regex use and reach at least
  2.00x pinned ripgrep 15.2.0 on a preregistered representative end-to-end
  suite.
- **Current:** ripgrep 15.2.0 at commit
  `e89fff89ac9af12e8d4ce9d5fd07beb408ca730f` is pinned. Its source path has
  been audited from CLI and ignore-aware traversal through regex/byte search,
  result construction, parallel publication, and exit status. The frozen
  nine-case suite completed one correctness-green official/native selection
  attempt, but all cases failed the 3% precision gate; no comparator, selected
  baseline, profile, Whitefoot result, or performance claim exists.
- **Claim boundary:** the suite must cover real source trees and large text;
  one and many files; several matcher families; ignore/filter work; and normal
  result production. A win on one file, `--sort`, fixed strings, a discarded
  output path, or a microbenchmark neither renames nor completes the flagship.
- **Missing / next:** the attributed-cause slice (task 0023, preregistered)
  credited the fused single-pass scan+match shape: 0.65 → 0.753/0.762 vs
  grep on scan cases, a 1.160 win on match-dense, landed with gates held.
  Counter re-attribution names the next floor: both improved shapes
  saturate at ~3.8 cycles/byte — the serial per-byte step's latency bound
  against memchr's 16-byte SIMD stride, with the SWAR shape's failure as
  the minimal witness that no legal source form lowers wide. The next
  performance question is therefore a check-aware wide-scan LOWERING
  capability (or the honest finding that it needs a proof mechanism);
  bounds traps remain secondary (~18% ceiling). Once the next slice passes
  its
  project-independent controls and same-slice correctness and cost gate, return
  to that exact `wfgrep` checkpoint. Each later slice stops on either the next
  semantic blocker or the first attributed material performance blocker; the
  full 2x objective remains unchanged.
- **Directions tested:** PERF-1 owns the baseline and attribution; BOUND-1 and
  VERIFY-1 enter with the real CLI/filesystem path; PAR-1 through PAR-4 enter
  only for measured parallel work and its proof/runtime contract; FLOOR-1
  through FLOOR-4 audit accepted source shape; STORE-1 and STORE-2 answer
  concrete matcher, queue, buffer, or result-representation blockers; PROOF-1,
  PROOF-2, PROOF-3, PROOF-7, and VERIFY-3 enter only for an observed fact
  consumer.
- **Facts:** [project frame, source audit, and comparison rules](../research/notes/ripgrep-flagship-frame.md) ·
  [inconclusive RG-BASE attempt](../research/experiments/ripgrep/RESULTS.md) ·
  [pinned upstream release](https://github.com/BurntSushi/ripgrep/releases/tag/15.2.0) ·
  [pinned upstream repository](https://github.com/BurntSushi/ripgrep/tree/15.2.0).

The other candidate classes remain comparison evidence or optional separately
approved probes. They are not phases or prerequisites in front of ripgrep.

| Candidate | Primary outline items | Current disposition | Reopening condition |
|---|---|---|---|
| `CAND-2` Compression / binary format | PERF-1, PROOF-1, PROOF-7, VERIFY-1, BOUND-1 | LZ4 and the raw-DEFLATE/zlib evidence are parked; they remain useful binary-transform controls. | A separately approved binary-transform question has independent decision value, including as a bounded cross-check for a live general mechanism. |
| `CAND-3` Parser / text validation | FLOOR-1, FLOOR-3, FLOOR-4, VERIFY-1, BOUND-1 | The yyjson strict-reader frame and current text witnesses are parked. | A separately approved parser or storage question has independent decision value that the current plan does not answer. |
| `CAND-5` Embedded / signal processing | TARGET-2, TARGET-3, PROOF-5, BOUND-1 | CMSIS-DSP remains parked; signal and image programs are internal evidence only. | A separately approved target/runtime question has an authentic Cortex boundary. |
| `CAND-6` Declared parallelism (`later`) | PAR-1 through PAR-4 | BLAKE3 remains a recognizable anchor; ripgrep now supplies the live project pressure, while automatic profitable discovery remains rejected. | A separately approved explicit-parallel question has independent value beyond the ripgrep plan. |
| `CAND-7` ML systems component (`later`) | APP-1, BOUND-2, TARGET-1 | Llama inference remains a possible attention probe, not a ripgrep prerequisite. | A separately approved ML question has independent value and a bounded real-model boundary. |

## Settled exclusions and history

- Automatic profitable parallelism discovery is rejected; only a
  writer-declared, compiler-verified direction remains live. See PAR-1.
- Product-scale artifact replay, capability overlays, whole-compiler resource
  profiles, and stable protocol machinery are not prerequisites for the
  research compiler. Reopen only for a real consumer. See
  [toolchain design memory](../mcts_mem/whitefoot/toolchain.md).
- Source-, function-, corpus-, project-, or test-shaped semantic dispatch is
  prohibited. Every selected project capability must use the normal general
  compiler path.
- The former phase-by-phase roadmap is preserved in Git history, not copied to
  `archive/`. Released specifications, approvals, results, and design memory
  retain the durable facts that still matter.
