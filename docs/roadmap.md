# Whitefoot Direction Outline

Status: CANONICAL DIRECTION OUTLINE
Revision: 56 (v0.39 narrows [CLM-1]'s claim-authority control dependence to
the definitions a boundary selector actually chooses)

The active language authority is v0.39, SHA-256
`4be4830fa87a534879de17524599b0919aef4dfab072dad823bf2f9b54d32d58`, carried by
the stable path [`spec/kernel-spec.md`](../spec/kernel-spec.md). It supersedes
v0.38 at `5a43c7638bd5839d77829836518374f9a169eb953d9c1edbd66b87815aedfb2d`,
archived at [`spec/kernel-spec-v0.38.md`](../spec/kernel-spec-v0.38.md). The
merge-time approval record is in
[`governance/APPROVALS.md`](../governance/APPROVALS.md) and becomes effective
with the owner's merge approval of the exact revision containing it; the batch
record is [batch 0091](done/0091-par3-judgment.md). The execution plan is
[`docs/current-plan.md`](current-plan.md).
Project law is the [`Constitution`](constitution.md), and the operational
process is [`WORKFLOW.md`](WORKFLOW.md).

## How to read this outline

This file is the owner-facing map of Whitefoot's live directions. It answers:

- what the project can already do;
- which ideas, requirements, and open questions belong together;
- what evidence exists and whether it is current or historical; and
- what is missing before a direction can advance.

This outline records the landscape; `current-plan.md` records technical
sequencing. Neither file grants or withholds permission to work on a branch or
adds a merge condition beyond the four rules in `WORKFLOW.md`. A project can
expose a missing capability, but cannot by itself change the language or
justify a project-shaped compiler special case.

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
- `[next: ...]` or `[parked]` — the next useful evidence or sequencing note,
  not an approval boundary.

The file is updated in place. Increment `Revision` when an item's goal,
evidence-backed current state, next gate, or candidate-project disposition
changes. Git is the version history; do not create versioned copies of this
file. Detailed semantics, measurements, design rationale, and implementation
inventories remain in their canonical owners and are linked rather than copied.

## Current baseline

`[current: spec v0.39]` `[current: safe-Rust compiler]`

Whitefoot has one normal path from canonical source through resolution,
semantic and ownership checking, checked program, typed CFG IR, target
qualification, LLVM, and host execution on supported aarch64 and x86-64
macOS/Linux targets. Valid language the compiler has not implemented stops as
unsupported rather than invalid source.

The compiler implements enough scalar, nominal, generic, storage, borrow,
contract, cleanup, and program-level behavior to begin external validation, but
not the entire active language. The exact implementation inventory and gaps
belong in the [compiler README](../compiler/README.md). v0.39 is the active
semantic authority. It rebuilds the system interface around formal state paths,
ordinary ownership, and completion-only lowering, with no separate world
region, capability class, blocking-call family, or `Ordered` relation.
Which gap matters next is selected by a project, never by checklist length.

## Dependency rules

- outline:CAND-1 records the completed flagship selection. outline:CAND-8 remains the selected
  flagship and pressure source. outline:PROOF-8's selected obligation-discharge
  sequence is now complete, so outline:CAND-8 is unparked; when its next
  bounded slice is selected, `current-plan.md` records that sequence. Mapped
  direction items inform the choice but do not grant or withhold branch
  permission.
- outline:PROOF-9 is the owner-selected successor to terminal outline:PROOF-8.
  It takes the explicit-obligation model language-wide, makes `claim` the only
  writer-reachable runtime rejection point, and replaces the historical entry
  contract exception with one closed-world command entry. v0.33 installs this
  successor together with batch 0072's completed outline:CAND-8 language
  deltas; the direction and its plan are terminal.
- outline:PROOF-10 is the owner-selected lifecycle correction after terminal
  outline:PROOF-9. It preserves `claim` as the sole writer-reachable runtime
  trap and preserves runtime execution for every accepted claim. It narrows the
  source construct to one human-proved, checker-unknown, individually necessary
  proof residual. Mechanically detectable redundant, contradictory, malformed,
  and unused forms become source errors; assertion, abort, test-oracle, and
  possible-failure intent is barred from positive programs and repaired by the
  author even when only semantic review can recognize its disguise. Its
  branch revision becomes mainline language authority only when the exact
  revision is tested, owner-approved, and merged under the four rules.
- outline:PERF-1 establishes ordinary code quality before a new optimizer fact or
  strategy is blamed or credited.
- Every production fact consumer in outline:PROOF-1 through outline:PROOF-4 and outline:PROOF-7 depends
  on outline:VERIFY-3. outline:PROOF-2 depends on outline:PROOF-5 only for a `willreturn`-class claim,
  not for memory-effect attributes.
- outline:PAR-1 still selects a source construct only after profiling exposes concrete
  parallel work; batch 0074 deliberately built permission without one. outline:PAR-4's
  runtime is now selected for the compute lane and measured there, so it is
  evidence for exactly that lane and preselects nothing for the I/O lane;
  outline:PAR-2 and outline:PAR-3 cannot preselect proof rules or reductions before their
  own workload evidence. For outline:PAR-3 the owner's chartering direction of
  2026-08-23 supplied that evidence and overtook this caution: the branch
  carries a counted-loop reduction candidate, measured on the `grid` family.
  outline:PAR-2's half is still parked and still bound by this sentence.
- outline:STORE-2 must expose a concrete unsolved representation privilege before
  outline:PROOF-6 can enter a plan.
- outline:TARGET-2 through outline:TARGET-4 depend on outline:BOUND-1 whenever their authentic milestone
  crosses the closed compilation-unit boundary. outline:PAR-4 and outline:BOUND-1 must agree on
  resource transfer, waiting, cancellation, and runtime thread authority.
- outline:APP-1 depends on outline:BOUND-2 when its selected component requires opaque foreign
  code rather than a Whitefoot or compiler-owned system provider.

## Proof and optimizer facts

Serves Constitution P0, W3, T1, and T2: useful facts must improve code without
creating writer trust or weakening the checked safety envelope.

### outline:PROOF-1 — Relational bounds proofs and static discharge

`[current: compiler]` `[historical: measured]` `[next: project pressure]`

- **Goal:** admit a proof-required bounds operation only when a deterministic
  derivation establishes the exact proposition that makes it safe.
- **Current:** the compiler discharges exact L0 bounds obligations and admits a
  proved `requires` goal as the callee-body S4 axiom. Ordinary callers prove
  that complete goal before transfer; no callee prologue or `llvm.assume` is
  emitted. Opaque Boolean goal identity adds no Boolean decomposition or new
  optimizer authority.
- **Missing / next:** a selected workload must first show a concrete proof gap
  or hot retained-claim pressure; then build one finite proof family with exact
  producers, invalidators, negative canaries, facts-off identity, and
  attribution. O11
  Boolean-goal composition stays an open question with four recorded findings
  and a de-pairing ruling (`governance/APPROVALS.md`); its trigger is a real
  program whose discharge needs a composed Boolean goal, and it re-enters only
  through the specification workflow.
- **Facts:** [compiler `requires` boundary](../compiler/README.md) ·
  [historical base64 result](../research/experiments/port-study/base64/RESULTS.md).

### outline:PROOF-2 — Effect-derived optimizer facts

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

### outline:PROOF-3 — Borrow-derived alias facts

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

### outline:PROOF-4 — Checked laws as transformation authority

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

### outline:PROOF-5 — Derived totality

`[current: spec]` `[research-only]` `[parked]`

- **Goal:** prove termination for a useful decidable fragment when a consumer
  needs a `willreturn`-class fact or a finite execution bound.
- **Current:** v0.17 explicitly has no termination checker; `pure` says nothing
  about return. Pure-row totality is a rejected design.
- **Missing / next:** reopen only for a selected effect optimization, embedded
  bound, or other concrete consumer; define the smallest fragment and its
  rejection boundary before implementation.
- **Facts:** v0.17 `EFF-3` · [totality design decision](../mcts_mem/whitefoot/effects/derived-totality.md).

### outline:PROOF-6 — Proof-gated representation authority (D17)

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

### outline:PROOF-7 — Verified strategy-selecting lowering

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

### outline:PROOF-8 — Obligation-discharge semantics: claims, caller-side discharge, trap as checker backstop

`[current: items 1–4, counted range, atomic requires goals, bounded provenance gate, complete DIAG-2 retention, installed v0.28 verified postconditions, deterministic claim ledger, and installed v0.29 strict partition]`
`[terminal]`

- **Goal:** replace each selected implicit trap family with explicit
  machine-tracked obligations. A migrated partial operation or `requires`
  becomes a proof goal; a call site discharges it statically, by a named and
  justification-carrying `claim`, or by a value branch where policy requires
  one. Result and trap decouple: Result models expected outcomes, while a claim
  is the checker's named runtime backstop at its provability frontier. This
  direction does not yet dissolve bare trapping arithmetic or ordinary
  explicit checks, so it makes no language-wide sole-trap-source claim.
- **Current:** v0.21 and v0.22 shipped the claim construct, normative L0
  entailment fragment, caller-side OP-4 index discharge, and the SYS-8 transfer
  count bounds introduced into the fact state by ENT-3 S10. v0.24 installs the
  corrected ENT-5 continuing-kill rule at `spec/kernel-spec.md`. The frozen
  installed-authority rerun proves 22/33 UTF-8 obligations, 0/9 SHA-256
  obligations, and 11/29 deflate obligations without claim support (11/24 on
  the dynamic path), with no proven-site regression and five non-rejecting
  redundancy advisories. The real boundary path produces its S10 relation; all
  four focused producer-family consumers and the kill control pass, while the
  driver itself honestly has no natural current obligation that consumes it.
  Task 0041 then measured the held provenance rule on the four-file
  boundary-fed DEFLATE unit: 18/33 obligation subjects are external, six prove
  without S2/S3, and 12 obligations under ten distinct claims would be rejected.
  The rule catches `order_slot_in_offsets` and `ordered_in_symbols` but launders
  `destination_in_symbols` through internal RHS values and an internal
  `offsets` root, so the required canonical result is only 2/3. The same walk
  records zero formal misclassifications under the drafted whole-root rule,
  five site-local stored-block precision false positives, and eight
  noncanonical positive declarations as the broader precision-spill count.
  Task 0046 repaired the held design without adding implicit-flow analysis:
  a place read now joins every explicit subscript offset, and PRV-2 relates a
  finite parameter datum to a concrete protected leaf with a terminating,
  deterministic witness. The frozen rewalk becomes 19/33 external subjects,
  six unasserted-state discharges, 13 rejected obligation nodes under eleven
  claims, and 14 internal subjects; the canonical result is 3/3 and the prior
  fifteen boundary-program claims remain ungated. The diagnostic projection is
  fourteen rejecting calls and 24 external actual atoms. Direct enum payload
  projections preserve success/error provenance while nested payloads expand
  conservatively one level. The rule remains held design evidence: its explicit
  write-address/control-flow limitation is recorded and O3 still blocks
  activation. v0.25 adds the evidence-selected counted `u64` half-open range
  and its finite S11 structural source without general induction. The ordinary
  compiler path captures endpoints once, preserves labelled cleanup and the
  maximum-u64 edge, and the real SHA-256 program discharges all nine index
  obligations after deleting four claims while its unrelated ordinary loop
  remains ordinary. The installed frozen confirmation reproduces UTF-8
  `22/33`, deflate `11/29`, and dynamic deflate `11/24`, while SHA moves from
  `0/9` to `9/9` claim-independent obligations; the worker is pure, contains
  no `wf_trap`, and retains both runtime oracles. Task 0047 is terminal and the
  complete repository gate is green. v0.26 preserves the complete FN-8
  declaration surface as one finite typed goal, requires every ordinary caller
  to prove the instantiated goal before transfer, supplies it to the body as
  S4, removes the callee prologue and its effect contribution, and retains the
  original dynamic failure behavior at both real process entries. Signed opaque
  facts stay atomic. The checked program retains subject-only local and
  transitive requirement-to-protected-leaf bridges plus full, unasserted, and
  S4-blinded rewalks; recursive and mutually recursive wrappers converge, a
  seedless cycle stays empty, and the three real DEFLATE calls discharge from
  allocation facts. This closes O3 structurally without emitting a provenance
  rejection. Installed confirmation at activation `441cd5b8` reproduces UTF-8
  `33/22/11/0`, SHA-256 `9/9/0/0`, complete DEFLATE `29/11/18/0`, and dynamic
  DEFLATE `24/11/13/0`; DEFLATE retains sixteen claims, five redundancy
  advisories, and no refuted claim. The complete repository gate is green, and
  the separately invoked adapter remains `Pass=393 Fail=1 Skip=13`, with only
  the pre-existing OWN-3 unsupported boundary. Commit `d495d8c` records the
  paired requirement-enforcement re-decision and passes MCTS lint. Task 0048 is
  terminal. Exact-approved v0.27 became active at
  `bbd7250084123bbce3267f741f30f6c12efc73c341ff8d361dd1b19d9502090f`,
  with byte-identical outgoing v0.26 archived at
  `spec/kernel-spec-v0.26.md`. It activates the bounded PRV-1/PRV-2/PRV-3
  explicit-dataflow gate over the retained requirement bridge. The approved
  real-program migration replaces eleven DEFLATE claims with value branches,
  adds the one `store_dynamic_length -> Result` change and its three
  propagations, removes only the four now-unexhibited `traps` categories, and
  adds sixteen conformance cases without changing any existing case or manifest
  row. Activation commit `5ab45aa7` installed those exact bytes and the
  byte-identical archive. Its installed rerun reproduces UTF-8 `33/22/11/0`,
  SHA-256 `9/9/0/0`, complete DEFLATE `29/24/5/0`, and dynamic DEFLATE
  `24/19/5/0`; twelve claims remain, seven load-bearing and five redundant,
  with no refuted claim, and all thirteen migrated sites discharge through
  real branches. Focused provenance is 41/41 and the frozen raw-DEFLATE oracle
  is 3/3. The complete gate is green with 698/698 library tests, 30/30 real
  programs, 131/131 rule coverage, and all 19 activation-chain entries. The
  installed adapter is `Pass=409 Fail=1 Skip=13`, with only the retained OWN-3
  unsupported boundary. Commit `74512309` makes the live design memory
  truthful and passes MCTS lint; task 0050 is terminal. Tasks 0054 and 0055
  install one function-local DIAG-2 derivation DAG with complete bounds, call,
  and counted-range roots. Task 0056 independently confirms root completeness,
  `O(S + P + R + C)` ledger-owned storage, preserved behavior, and measured
  release cost. Stage 8a tasks 0051 and 0052 establish the two removable local
  witnesses with hostile controls and restored green gates; none of their
  temporary fact sources or program variants is installed. Task 0053 freezes
  the exact fourteen `read_bits` and twenty `append_slice` callers: the read
  side has fourteen mutable-delivery gaps, and the append map is staged
  `19 discharged / 1 unproved`, with two distinct declarations and only the
  wfgrep separator needing a caller repair. Task 0058's hostile source replay
  falsified the former value-match repair and froze the narrow selected-payload
  receiver now installed by v0.28. Tasks 0059–0065 then installed FN-9,
  complete/U/B callee proofs, callee-before-caller summaries, the four closed
  call-result routes, `value_if`-only delivery, failure-atomic provenance
  publication, and the five real consumers without a second proof authority or
  runtime fallback. All fourteen `read_bits` and twenty `append_slice` routes
  now discharge through the ordinary compiler path; A10 alone uses the bounded
  `value_if` repair. The owner-approved additive protected matrix brings the
  corpus to 437 cases and 132/132 rule coverage, with the previous 423 cases
  and 30 annotations unchanged. Task 0066 then installs one checked-program-only
  read-only claim ledger over the existing finalized derivation DAG, with no
  second semantic walk or closure, copied proof graph, acceptance consumer, or
  lowering consumer. Commit `e04d3ac` enumerates the complete installed claim
  populations as UTF-8 `2`, four-source raw-DEFLATE `12`, and wfgrep `8`; the
  frozen-real owning test is `414.00s` against commit
  `5fd017b46973e5cbf990fe3fc92a2cc20a76f91c` at `412.36s` (about `0.4%`),
  and the complete compiler and repository gates are green while the separate
  adapter preserves the installed `Pass=423 Fail=1 Skip=13` boundary.
  The owner then approved the exact Stage 9b candidate bound by commit
  `4e47073` and approval record `137ef4d`. Held frontend checkpoint `ec95b70`
  defined the reviewed v0.29 declaration marker and generated tables; held
  semantic checkpoint `a927f2c` defined CLM-3's finite direct/import may-claim SCC summaries, existing-U
  OP-4/FN-8 judgments, marked-entry pre-wrapper check, and failure-atomic
  checked publication. Ordinary unmarked acceptance, runtime bodies, lowering,
  and the Stage 9a observational ledger remain outside that authority path.
  Focused strict semantics pass 17/17, the non-heavy semantic selection passes
  481/481, the separately selected heavy entailment owning test passes 1/1 in
  711.37s, and the separately selected heavy provenance test passes 1/1 in
  207.28s. Those selections cover the original 483 semantic tests but were not
  one 483/483 run. The 711.37s owning test combines multiple real sources,
  production checking, and test-only derivation validation, so it cannot be
  attributed separately to wfgrep compilation or `validate_derivations`. The
  sole wfgrep source change is `deny_claims ` on `report_failure`; the installed
  source hashes to `fb2f3b44160a947d...` and preserves its semantic and
  non-upward oracles. The additive protected matrix is 446 cases, 30 unchanged
  annotations, and 133/133 rule coverage; the adapter is
  `Pass=432 Fail=1 Skip=13` in 202.22s. `make conformance-run` exits 2 only
  because `own3-pos-outlives-store` still expects `Run(0)` and reaches
  `Unsupported(RegionsAndBorrows)`; no other protected case fails. An exact
  wfgrep preactivation integration test ran 139.63s and then failed only at the
  expected still-v0.28 `CommandEntry` target mapping pin; it was neither an
  activation-era passing wfgrep gate nor a semantic failure. The final
  repository-root `make check` exits 0 with specification append-only checks,
  archive identities 30, runner 23/23, coverage 133/133, compiler library
  833/833 in 679.39s, real programs 32/32 in 2170.33s, active v0.29 at its
  exact SHA with 133 rules and 21 activation links, rustdoc warnings denied,
  and both compiler and repository green markers; the deliberately ignored
  conformance target remains the known OWN-3 integration. The approved
  derivation ledger at SHA-256 `7f2b277c...`
  intentionally retains its candidate-time historical prose. The active
  specification is sole language authority; the v0.29 activation chain records
  its installed identity, and this outline records terminal `outline:PROOF-8` status.
- **Missing / next:** none inside outline:PROOF-8. The direction is terminal and
  `outline:CAND-8` is unparked. Record its next bounded technical sequence in
  `current-plan.md` when selected.
- **Facts:** [design dossier](../research/investigations/obligation-discharge/DOSSIER.md) ·
  [simulation](../research/investigations/obligation-discharge/SIMULATION.md) ·
  [native acceptance](../research/investigations/obligation-discharge/ACCEPTANCE.md) ·
  [Stage 8b semantic freeze](done/0058-stage8b-normal-return-postconditions.md) ·
  [Stage 9a deterministic claim ledger](done/0066-stage9a-deterministic-claim-ledger.md) ·
  [Stage 9b strict partition](done/0067-stage9b-strict-partition-candidate.md) ·
  [W1 probes](../research/investigations/obligation-discharge/PROBE-W1.md) ·
  [taint](../research/investigations/obligation-discharge/PROBE-TAINT.md) ·
  [codegen](../research/investigations/obligation-discharge/PROBE-CODEGEN.md) ·
  [stable specification model](../governance/spec-evolution/stable-spec-filename-proposal.md).

### outline:PROOF-9 — Claim-only runtime trap surface and static contracts

`[historical: active v0.33 closure]` `[terminal]`

- **Goal:** make `claim` the only writer-reachable source of a language-level
  runtime rejection. Every hazardous operation is either total, returns a
  typed expected outcome, or carries a deterministic proof obligation that is
  discharged by machine facts, a real branch, or an executed named claim.
  Function contracts are erased proof structures rather than statement-shaped
  pseudo-runtime blocks.
- **Selection:** every function result is explicitly named; one unified
  `contract` block contains erased symbolic definitions and independent
  `requires` and `ensures` clauses, including selected `Result` routes.
  Requirements are internal-call obligations only. The sole entry is an
  uncallable `command fn main` with a named `ExitStatus` result and no contract.
  A checker-detected contradictory requirement set denotes an uninhabited
  function and lowers to an ABI-preserving unreachable stub, never to an
  unchecked version of its source body.
- **v0.33 closure:** v0.33 and the compiler implemented named results, the unified
  erased contract surface, plural static proofs, command-only entry, exact
  operation/allocation/system obligations, and claim-only DIAG-3 lowering.
  The installed 499-case protected corpus is in exact manifest bijection;
  canonical conformance is `Pass=498 Skip=1 Fail=0`. Target review and the
  independent final audits are complete.
- **Required closure:** met. The installed grammar contains no contract-local
  statement or trap spelling; accepted integer, allocation, and affected
  system-operation IR carries no runtime trap site; all requirement calls are
  proved before transfer; every retained language trap record is owned by an
  executed claim; exact checked/wrapping/saturating arithmetic, typed host and
  content failures, target qualification, and resource-exhaustion boundaries
  retain their distinct semantics.
- **Facts:** [contract surface design space](../research/investigations/contract-surface/DESIGN-SPACE.md) ·
  [terminal obligation-discharge direction](#outlineproof-8) ·
  [unified host-boundary architecture](#outlinebound-1) ·
  [batch 0072 closure](done/0072-searching-wfgrep.md) ·
  [batch 0073 closure](done/0073-claim-only-contracts.md).

### outline:PROOF-10 — Claim residual source canonicality

`[current: active v0.39; introduced by v0.34]` `[terminal]`

- **Goal:** make every claim in a successful checker result one mechanically
  qualified, individually necessary runtime-residual candidate, and every
  claim in a human-reviewed positive program an actually proved residual. A claim is
  neither an assertion, abort, test oracle, nor conditional. Its predicate is
  universally true at that point, unknown to the normative checker, total and
  observational to evaluate, and has a versioned canonical contribution normal
  form whose every component is checker-unknown and load-bearing rather than a
  bundle with known or unused members. It is explained by an actual derivation in
  `because` and needed by a fixed source-admission proof. Every
  compiler-accepted claim remains retained, contributes its exact source
  effects including `traps`, lowers normally, and is evaluated at every dynamic
  reach in every build mode.
- **Selection:** the compiler mechanically enforces the residual shape; human,
  AI-assisted, or offline-prover review owns theorem truth, proposition
  minimality, and consumer authenticity. The compiler checks a deterministic
  five-field `because` shape but does not pretend to validate its English.
  Exact positive and negative lifecycle queries cover the unique D/S/F images:
  direct snapshot D, support-canonical contribution frontier S, and fully
  structural lifecycle image F. `Contrib(P)` is derived only from S and uses an operator-specific sound signed
  conjunctive basis and leaves only justified roots such as positive
  disjunction as singletons; xor/equivalence may not default to singleton;
  finite deterministic parent introduction must reconstruct S and then
  materialize D from checker facts plus its contributions through one shared,
  non-cyclic ENT rule; F is never an S3 source or reconstruction target
  in every proof view. Proved, refuted, contradictory,
  inconsistent, overlapping, and
  non-load-bearing candidates are source errors. `claim True()` is redundant
  and `claim False()` is illegal, never an intentional-abort form. Individual
  necessity is simultaneous `Full-minus(c,a)` for each canonical component plus
  whole-occurrence `Full-minus(c)` over a fixed otherwise-valid candidate set.
  Every component and occurrence must make at least one non-contradictory,
  non-explosive closed-list admission root stop discharging through the exact
  S3 event. At a join, c need dominate only its c-dependent predecessor
  lineage, while every reachable predecessor contributing to the root must
  independently have non-contradictory, non-explosive legal support.
  All-claims U plus occurrence in one canonical proof is deliberately
  insufficient. This establishes checker-relative component/occurrence
  irredundancy, not a unique proof basis or globally weakest proposition.
  Accepted-claim runtime execution is unchanged, and no optional solver
  participates in ordinary source acceptance.
- **Current:** v0.34 introduced this direction, and active v0.39 preserves it.
  The exact activation identities and conformance boundaries remain in
  `governance/APPROVALS.md`. The residual lifecycle, contribution basis,
  reconstruction, fixed eligible set, component/occurrence necessity, stable
  terminal evidence, generic
  schema and concrete-instance checks, and unchanged retained lowering path
  are implemented. The locality correction classifies every user-call or
  system-call result component as `BoundaryResult` and propagates that
  authority through value, control, holder, and storage flow. Neither S12 nor
  a PRV-internal result declassifies it; caller code consumes verified boundary
  facts directly instead of restating them in a claim. Locality is analyzed
  once per applicable function inventory, claim-free functions take a fast
  path, and PRV-1 is frozen once rather than rerun for each mask. Residual
  component and whole-occurrence masks still rerun all function inventories
  and the program-level PRV-2/PRV-3 scans; that inherited baseline path remains
  a high-claim-count scaling risk even though this change removes its repeated
  PRV-1 fixed point. The real,
  backend, and protected migrations are complete through `99df5579`. The
  protected source census at activation was 95 claim statements in 74 `.wf`
  files using `^[[:space:]]*claim `; raw word counts that include documentation
  and strings are not source-statement counts. Warm probes measured locality at
  about 226 microseconds once per inventory, `utf8parse` compilation about 25%
  faster, and optimized `wfgrep` compilation about 10–13% faster than the
  compared baseline; these are bounded development measurements, not a general
  speed guarantee or a closure of the inherited per-mask scaling risk.
- **Missing / next:** none inside this direction. Later specification versions
  must preserve its source and runtime invariants unless a new owner-approved
  specification change explicitly supersedes them.
- **Facts:** [claim residual canonicality investigation](../research/investigations/obligation-discharge/CLAIM-RESIDUAL-CANONICALITY.md) ·
  [batch 0073 claim-only closure](done/0073-claim-only-contracts.md).

## Verification and compiler trust

Serves W3, T1, and T2: current claims must survive independent, hostile, and
facts-off evidence rather than trust in the compiler or writer.

### outline:VERIFY-1 — Checked safety envelope in real programs

`[current: spec]` `[current: compiler]` `[next: project validation]`

- **Goal:** make memory corruption, data races, uninitialized reads, and silent
  overflow unrepresentable across success, failure, and cleanup paths.
- **Current:** the active language forbids writer trust; the compiler
  admits partial operations only after static discharge, retains exact
  claim-owned DIAG-3 records, affine cleanup, proof-required indexing, and
  distinct target/resource guards on its implemented path.
- **Missing / next:** validate malformed input and language-level failures
  separately from target or allocator resource failure, then exercise partial
  results, transfer, and teardown in the first selected medium project.
- **Facts:** [Constitution](constitution.md) · [compiler README](../compiler/README.md).

### outline:VERIFY-2 — Execute the conformance corpus against the compiler

`[current: native adapter installed]`
`[next: publish an exact-revision report for the activated v0.39 revision]`

- **Goal:** compare compiler behavior with compiler-independent active-spec
  expectations through the normal command path.
- **Current:** the native execution adapter is wired (task 0014):
  `make conformance-run` compiles and runs every case through the real
  compiler. Canonical root `make check` invokes the complete adapter
  explicitly even though the Cargo integration remains marked `#[ignore]` for
  ordinary test runs. Historical exact-revision results remain in their batch
  and activation records; this outline carries no floating count of its own.
- **Missing / next:** publish the next independent pass/fail/skip report for
  the exact activated v0.39 revision. Any
  expectation, source, status, collection, or invocation change is conformance
  evidence whose exact before/after content is recorded under merge rule 4.
- **Facts:** [conformance corpus](../tests/conformance) · [workflow](WORKFLOW.md).

### outline:VERIFY-3 — Facts-on/facts-off differential trust

`[historical: measured]` `[next: with first fact consumer]`

- **Goal:** prove that an optional optimizer fact changes only justified code
  shape, never acceptance, outputs, written-claim execution, or cleanup.
- **Current:** historical experiments have local controls; the current compiler
  has no production check-elision fact family and therefore no global claim.
- **Missing / next:** the first fact consumer must ship with legal-program
  differential generation, hostile premise mutation, output and claim identity,
  and attribution before timing.
- **Facts:** [experiment index](../research/experiments/README.md) ·
  [fact-channel design memory](../mcts_mem/whitefoot/fact-channels.md).

### outline:VERIFY-4 — Deterministic and reproducible artifacts

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

### outline:PERF-1 — Ordinary lowering and baseline code quality

`[current: check-aware wide probe landed]` `[current: wfgrep beats pinned grep 1.07/1.35]`
`[next: owner-selected next attributed cause or capability]`

- **Goal:** make ordinary checked source competitive before relying on a new
  proof channel, special writer trick, or project-specific lowering.
- **Current:** the compiler has one conservative LLVM path and executable
  program witnesses, but no current medium-project comparison of scalar code
  shape, proof gaps, retained claims, vectorization, and target output. `RG-BASE` completed
  one correctness-green upstream selection attempt; host cache-position noise
  defeated its precision gate, so it selected no comparator and made no
  performance claim. Its medians remain a development-cost table, not a
  baseline.
- **Missing / next:** require every newly runnable `wfgrep` slice to pass its
  correctness oracle and scoped cost-shape or performance gate before adding
  downstream behavior. Attribute each material loss to algorithm, static proof
  gap, retained claim, source shape, compiler lowering, LLVM recovery, runtime,
  I/O, output, or target, resolve its owning layer generally, and rerun the same
  slice. The
  full paired suite is reserved for a later public-claim candidate rather than
  the edit loop.
- **Facts:** [compiler backend boundary](../compiler/README.md) ·
  [historical DEFLATE result](../research/experiments/zlib-core-kernels/RESULTS.md) ·
  [ripgrep flagship frame](../research/notes/ripgrep-flagship-frame.md).

### outline:FLOOR-1 — Canonical source and constrained control shape

`[current: spec]` `[current: compiler]` `[next: project validation]`

- **Goal:** remove accidental slow alternatives so the ordinary accepted shape
  is a strong default for an AI writer.
- **Current:** v0.17 fixes canonical bytes, flat ANF, one `loop` form, and closed
  statement/value branching; the compiler implements those forms. This alone
  is not a performance guarantee.
- **Missing / next:** compare current AI-written project code with a measured
  expert reference shape and identify accepted but materially slower forms.
- **Facts:** v0.17 `FORM-*` and `GRAM-*` · [floor rationale](why-whitefoot.md).

### outline:FLOOR-2 — Closed, taught pattern catalog

`[seeded]` `[next: project validation]`

- **Goal:** teach a small set of patterns that are both expressive enough for
  real systems work and aligned with fast machine shapes.
- **Current:** `docs/patterns.md` contains twelve entries of mixed maturity plus a
  known-gaps list. Some have measurements or current witnesses; P5 is deferred,
  P6 is validation-only, and the catalog is not normative language doctrine.
- **Missing / next:** validate individual patterns in candidate projects;
  promote a new card or rejection proposal only after observing a recurring
  slower-but-accepted or currently inexpressible shape.
- **Facts:** [pattern catalog](patterns.md) · [pattern design memory](../mcts_mem/whitefoot/pattern-doctrine.md).

### outline:FLOOR-3 — Project floor audit

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

### outline:FLOOR-4 — Diagnostic repair loop

`[current: deterministic diagnostics]` `[next: measure]`

- **Goal:** give the AI deterministic, actionable failures that shorten the
  path from rejected source to a correct, efficient program.
- **Current:** the candidate requires deterministic rule/location diagnostics
  and exact named-claim records; single-shot writability and repair
  effectiveness are not established.
- **Missing / next:** measure repair-to-green on real project failures and turn
  repeated confusion into a diagnostic or teaching defect.
- **Facts:** v0.17 `DIAG-*` · [honest limitation](why-whitefoot.md#part-vi-what-it-does-not-beat-and-what-is-not-yet-known).

### outline:FLOOR-5 — Spelling rule and surface relief

`[research: complete]` `[next: spec batch after outline:PROOF-8 slice 1]`

- **Goal:** every surface byte carries a decision the checker cannot
  reconstruct (tests T1 decision / T2 boundary / T3 uniqueness / T4
  globality, plus no-optionality); relieve ceremony strictly per grammar
  class while boundaries stay fully explicit.
- **Current:** rule agreed and the full v0.20 surface sweep recorded:
  whole-class deletions (value-op type args, `index` type, body-let
  annotations, Bool-match arm ceremony → `if`/mandatory `else if`
  flattening, all auto-migratable by the canonical printer), whole-class
  keeps (literal suffixes, loop labels, the three named-argument
  disciplines, signatures), per-operation respellings (precedence-free
  infix — safe exactly because GRAM-9 ANF is retained), ANF relaxation
  deferred indefinitely.
- **Missing / next:** one spelling batch (deletions + respellings), native
  grammar-verifier pass, mechanical corpus migration in the same change.
- **Facts:** [sweep](../research/investigations/spelling-relief/SWEEP.md).

## Storage, ownership, and representation

Serves P0, W1, W3, T1, and T2: useful data structures must retain safety and
optimizer facts without a writer-accessible escape or hidden pathological cost.

### outline:STORE-1 — Borrow and provenance completeness

`[current: compiler]` `[next: project-selected gap]`

- **Goal:** express useful views and mutations while retaining exact ownership,
  origin, overlap, and effect information.
- **Current:** the compiler supports buffer/struct borrows, scoped child
  reborrows, direct slices, direct own returned slices, and — since task
  0024 — borrow-mode parameters, let-borrows, deref reads/writes, and
  matching through borrowed enums for scalar and enum content on one
  generalized address machinery. v0.20 (activated 2026-08-07, task 0029)
  adds mode-preserving returned reborrows (OWN-14) and defines borrow-mode
  match payload binders as arm-scoped child reborrows with region-remainder
  suspension of a uniq root (OWN-13), closing the recorded OWN-6 gap and
  the OWN-13/OWN-5 contradiction. Uniq non-copy payload binders and written
  uniq nested chains remain explicit capability gaps; branch-produced loans
  and holder-derived slices remain absent.
- **Missing / next:** choose the smallest missing rule only after a real
  project cannot express its required access pattern. The 31-rule loan/freeze
  review candidate and older M1 model are parked evidence, not language
  authority or a ready implementation package; it vacated the v0.18 candidate
  slot for the outline:BOUND-1 system-interface batch on 2026-08-05 (predates the
  wfgrep goal framing and would need re-derivation from a real blocker).
- **Facts:** [compiler borrow boundary](../compiler/README.md) ·
  [parked loan/freeze candidate](../governance/spec-evolution/parked-loan-freeze-candidate.md) ·
  [M1 placement](../research/archive-promotion-audit.md#2-keep-the-m1-loanfreeze-work-as-a-parked-candidate-not-a-rule-set).

### outline:STORE-2 — Growth, replacement, occupancy, and identity

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

### outline:STORE-3 — Refined domains and automatic niches

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

### outline:PAR-1 — Proof-derived permission, writer-declared surface

`[current: branch candidate]` `[current: measured]` `[next: owner merge decision]`

- **Goal:** overlap execution only where the compiler's existing acceptance
  proofs already show it cannot be observed, and give the writer a
  non-authoritative way to say what it expects.
- **Current:** batch 0074 built the permission half on branch
  `par/proof-derived-parallelism` and left the declared surface for its own
  packet. The compiler judges sibling call pairs against four conditions read
  off resolved places, effect rows, the [OWN-7] overlap relation, the [EFF-2]
  projection, and the call graph; `--par-ledger` prints the verdict for every
  analyzed site, and one hint line per counted loop a recursive index split
  would make eligible; a pthread lane pool actualizes eligible chains, sized by
  `WF_WORKERS`, which on the branch defaults to the machine's logical CPUs and
  is opted out of with `0` or `1`. Spec CANDIDATE v0.35 states the law as one
  rule, [PAR-1]. The direction that stays rejected is unchanged and worth
  keeping straight: the compiler decides *legality* from proofs and never
  guesses *profitability*, which a runtime lane budget decides at the moment of
  the offer.
- **Missing / next:** the owner's merge decision, including two rulings the
  batch record asks for; then the I/O concurrency lane, which is where the
  measured profit is, ahead of the `pal` marker and any permission widening.
- **Facts:** [design contract](../research/investigations/proof-derived-parallelism/DESIGN.md) ·
  [measured results](../research/investigations/proof-derived-parallelism/RESULTS.md) ·
  [deciding probes](../research/investigations/proof-derived-parallelism/probes/README.md) ·
  [permission and layer model](../research/investigations/proof-derived-parallelism/PAL.md) ·
  [batch record 0074](done/0074-proof-derived-parallelism.md) ·
  [auto-parallelism feasibility result](../research/experiments/auto-parallelism-feasibility/RESULTS.md).

### outline:PAR-2 — Intra-object disjointness

`[research-only]` `[parked]`

- **Goal:** prove disjoint subranges or injective indexed writes when
  region-level separation is too coarse.
- **Current:** effects can separate storage origins but do not prove arbitrary
  element-level injectivity; there is no production `split_uniq` capability.
  **This is exactly what batch 0078 deferred**: its counted-loop permission is
  the reduction and not the map, because a resolved place carries no index
  segment, so `dst[i]` and `dst[j]` are one place and every element write
  denies under its condition 2. The probe carrying that fact is
  `loop/probes/x1_same_buffer.wf`.
- **Missing / next:** a selected sequential or parallel project must first
  require this exact access pattern; then choose the smallest judgment and its
  lifetime across calls or recursion. The 0078 re-entry condition is the
  concrete form of that: a real program with a compute-heavy
  single-destination map, or places gaining index granularity — whichever
  arrives first reopens the map half of [PAR-2].
- **Facts:** [parallelism feasibility result](../research/experiments/auto-parallelism-feasibility/RESULTS.md) ·
  [pattern gaps](patterns.md#known-gaps-findings-not-yet-patterns) ·
  [loop-permission design ruling](../research/investigations/proof-derived-parallelism/loop/DESIGN.md) ·
  [batch record 0078](done/0078-loop-permission.md).

Note the name collision, which the `outline:` prefix keeps formally distinct:
this direction is `outline:PAR-2`, and the spec rule `[PAR-2]` the branch
carries is counted-loop reduction permission, not intra-object disjointness.

### outline:PAR-3 — Reductions, algebra, and trap selection

`[current: spec]` `[historical: measured]` `[next: project pressure]`

- **Goal:** parallelize only an exact algebraic domain whose result and failure
  semantics survive regrouping and concurrent eligibility.
- **Current:** FN-4 law discharge exists for acceptance. Parallel reduction,
  deterministic result, and concurrent trap selection **now exist as branch
  candidates**, where this entry previously said they do not. Batch 0078 on
  `par/loop-permission` carries a CANDIDATE rule [PAR-2] granting counted-loop
  reduction over a normatively enumerated exactly-associative combine set
  (float excluded by rule, not by hedge), a lowering that publishes the
  sequential fold's bytes at every worker count, and a trap rule: the claim
  redirect makes the observable-identity guarantee conditional on contract
  compliance, and `wf_trap` takes a first-trap-wins latch, so an erroneous
  execution writes exactly one record and which claim it names is the
  schedule's to choose. None of it is active; all of it activates only at
  merge. Floating reproducibility is still absent and is excluded by the rule.
  Historical chunk-summary work found no Whitefoot-over-Rust delta.
- **Missing / next:** the workload evidence this entry asked for is partly in —
  the `grid` family measures 6.5x for the loop form against its own sequential
  build, and the corpus census reports the rule fires on none of the twelve
  counted loops the project has already written. The float domain, the map
  (element-write) half, and any non-commutative associative operation are open,
  each with a named re-entry condition in the design ruling.
- **Facts:** [historical chunk-summary result](../research/experiments/port-study/wc-chunk-summary/RESULTS.md) ·
  [loop-permission design ruling](../research/investigations/proof-derived-parallelism/loop/DESIGN.md) ·
  [batch record 0078](done/0078-loop-permission.md).

### outline:PAR-4 — Runtime, allocation, and dynamic fan-out

`[current: branch candidate]` `[current: measured]`

- **Goal:** execute selected parallel forms without hiding serialization,
  unbounded overhead, or an unexplained trusted runtime.
- **Current:** batch 0074's compute lane pool remains the measured baseline.
  The I/O rebuild separately retains bounded generation-checked completion
  records, exact result and loan-release milestones, one
  compute/target/completion wake decision, typed target-only helpers, real
  Linux io_uring positioned I/O, and the Windows IOCP foundation. The rejected
  root/family/Ordered group layer has been removed. Completion drain still
  precedes dependent-frame readiness, the first tail-wrapper stackless slice
  can resume on any scheduler lane, and pure compute links no completion
  runtime.
- **Missing / next:** generalize selective stackless continuation lowering to
  branches, loops, multiple suspension points, and non-tail children;
  measure cold/high-latency and native target workloads; and execute the
  Windows probe before qualification. Any widening keeps the same bounded
  ownership and hostile soundness gates.
- **Facts:** [dynamic fan-out placement](../research/archive-promotion-audit.md#3-dynamic-fan-out-retained-as-a-parallel-design-witness) ·
  [measured lane grants and wall time](../research/investigations/proof-derived-parallelism/RESULTS.md).

## Boundaries, targets, and deployment

Serves P0, W3, T1, T2, and R6: external usefulness and target reach may not
become alternate unchecked semantics or prematurely bind the whole toolchain.

### outline:BOUND-1 — Unified state and host integration

`[current: v0.39 active; unified-state model implemented and validated]`
`[next: wider APIs and target measurements]`

- **Goal:** give command, service, and embedded program instances a coherent
  host boundary covering process context, filesystems, data streams, clocks,
  randomness, networking, waiting, and cancellation without ambient mutable
  authority, writer-defined trust, or a second I/O type system.
- **Current:** v0.39 is active and carries this model. It uses
  ordinary opaque affine values and the existing `own`, `move`, `&`, and
  `&uniq` rules for all resources. `reads` and `writes` name formal parameters
  or static struct fields rather than lifetimes. Lifetimes state loan duration
  only. There is one state/effect system and
  no `world`, `external`, `blocks`, `capability-root`, `family-fragment`, or
  `Ordered` permission layer. Changing clocks, Outputs, Sources, cursors, and
  factories use `own` or `&uniq`; genuinely independent work uses distinct
  ordinary owned places or borrows. File opens consume proof-only one-shot
  `FilePermit` values produced by total `reserve_file(&uniq FileFactory)`;
  `DirectoryRead` is a shared selector, host exhaustion remains a typed open
  result, and the permit is erased before the native ABI. Completion remains the sole language-level
  I/O model. The generation-safe runtime core, target-only helpers, Linux
  io_uring work, Windows IOCP foundation, selective stackless slice, and
  component measurements were retained while the rejected group machinery was
  removed. The activated revision passes compiler, program, conformance,
  sanitizer, native helper, stress, and cross-link gates. Whole programs have
  now been measured on both macOS and Linux with io_uring
  ([batch 0084](done/0084-io-performance.md)): the shipped build is about
  twice its own sequential build on a many-independent-files workload, and
  within 3.4 percent of a hand-written io_uring pipeline at the same queue
  depth the source can ask for. Measuring also found and fixed a join
  busy-wait, a default helper count pinned at the worst value, and an
  identifier collision that had made every Linux link of a completion program
  fail to compile. [SYS-14] directory enumeration is now qualified on both
  families ([batch 0094](done/0094-linux-directory-row.md)): the Darwin row
  reads the name length its `struct dirent` states, the Linux row derives it
  by one scan bounded by the extent `d_reclen` reports, because
  `struct linux_dirent64` states none — one normalizer text, one target-selected
  block. That closed the one gap batch 0090 left open on the Linux gate: six
  conformance cases now pass at their existing verdicts, both
  directory-walking corpus programs compile and run there, and every host limit
  0090 declared for the missing row is removed.
- **Missing / next:** widen stackless lowering beyond single-instruction tail
  chains; execute and qualify Windows; add a clock reading, keyed directory
  places, namespace mutation, and network, timer, cancellation, deadline, and
  finish-required output APIs only with complete ordinary ownership and target
  contracts. The performance question that remains is width, not protocol:
  overlap groups are runs of consecutive calls in one basic block, so a loop
  with one I/O call per iteration overlaps nothing, and whether the language,
  the lowering, or neither should widen that is undecided. The open items are
  enumerated in [batch 0082](done/0082-unified-state-completion-io.md) and
  [batch 0084](done/0084-io-performance.md).
- **Facts:** [batch record 0082](done/0082-unified-state-completion-io.md) ·
  [batch record 0084](done/0084-io-performance.md) ·
  [batch record 0089](done/0089-loop-pipeline-batch0.md) ·
  [batch record 0094](done/0094-linux-directory-row.md) ·
  [program-level measurement bundle](../research/experiments/io-completion-bench/README.md) ·
  [first-principles derivation](../research/investigations/io-model/FIRST-PRINCIPLES.md) ·
  [concrete API and lowering design](../research/investigations/io-model/DESIGN.md) ·
  [staged loop pipeline design](../research/investigations/io-model/LOOP-PIPELINE.md) ·
  [experimental implementation audit](../research/investigations/io-model/IMPLEMENTATION-AUDIT.md) ·
  [program-level and clean-core measurements](../research/investigations/io-model/RESULTS.md) ·
  [historical architecture dossier](../research/investigations/system-capability-architecture/DOSSIER.md) ·
  [historical review decision record](../research/investigations/system-capability-architecture/decisions.json) ·
  [WASI capability model](https://github.com/WebAssembly/WASI/blob/main/docs/Capabilities.md) ·
  [WASI 0.1–0.3 release lessons](https://wasi.dev/releases).

### outline:BOUND-2 — Foreign ABI and opaque binaries

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
  cannot credibly replace. Keep this distinct from outline:BOUND-1's compiler-owned
  system provider; system calls do not by themselves justify general FFI.
- **Facts:** v0.17 `GATE-1` and `LEDGER-1` ·
  [safe capsule idea](ideas.md#safe-c-abi-capsules).

Migration tooling is supporting work under outline:BOUND-2, not an independent language
authority; see the [C-to-Whitefoot assumption extractor](ideas.md#a-c-to-whitefoot-assumption-extractor).

### outline:TARGET-1 — Portable and mutually checking backends

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

### outline:TARGET-2 — Embedded and resource-bounded systems

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

### outline:TARGET-3 — Deployment evidence and policy

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

### outline:TARGET-4 — Constant-time secret-dependent behavior

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

### outline:APP-1 — ML systems components

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
justify a favorable subset, a monolithic rewrite, or any particular
language, compiler, proof, runtime, or optimizer change. Completed plans
through batch 0070 closed the bounded outline:PROOF-8 undertaking and the
specified-gap/take-replace undertaking (v0.31 activated). When the next
outline:CAND-8 slice is selected, `current-plan.md` records how it returns to
the unchanged product comparison; that sequencing record is not branch-work
permission.

Owner framing (2026-08-05): the project's deliverable is what `wfgrep` proves
about the language's functional and performance ceiling — resolved general
capabilities, attributed wins, and honest negative results. The 2x claim
remains the pressure source and honesty anchor; shipping a finished tool is
not the completion condition. Every specification amendment on this path is
sourced from a need the frozen `wfgrep` slice actually exposed, then designed
as a complete capability rather than a wfgrep-minimal one. A language gap
exposed by a frozen slice is therefore a finding first; the technical
language-gap discipline remains its expected path, not a presumption against
the goal or an additional approval step.

### outline:CAND-1 — Select the first external validation project

`[current: completed — ripgrep selected]`

- **Goal:** choose a project with an immediately legible public result, low
  user trial cost, a strong oracle, and enough end-to-end pressure to expose
  general language, compiler, runtime, and machine-code needs.
- **Current:** the initial N1 screen advanced yyjson and LZ4 under a
  near-term-reachability gate. The owner subsequently made comparative
  performance and immediate tool adoption the primary public test and selected
  pinned ripgrep 15.2.0.
- **Missing / next:** none; `outline:CAND-8` owns the preserved flagship sequence and
  is unparked now that outline:PROOF-8 is terminal. Record its next selected
  slice in `current-plan.md`.
- **Facts:** [historical N1 shortlist](../research/notes/headline-artifact-shortlist.md) ·
  [ripgrep flagship frame](../research/notes/ripgrep-flagship-frame.md) ·
  [current executable programs](../compiler/README.md).

### outline:CAND-8 — Ripgrep-compatible command-line search

`[selected flagship]` `[current: credited compute-bound win]`
`[unparked: new high-level plan required]`

- **Goal:** build a Whitefoot-written command-line search tool credible for
  ripgrep's primary line-oriented recursive regex use and reach at least
  2.00x pinned ripgrep 15.2.0 on a preregistered representative end-to-end
  suite.
- **Current:** ripgrep 15.2.0 at commit
  `e89fff89ac9af12e8d4ce9d5fd07beb408ca730f` is pinned. Its source path has
  been audited from CLI and ignore-aware traversal through regex/byte search,
  result construction, parallel publication, and exit status. The frozen
  nine-case suite completed one correctness-green official/native selection
  attempt, but all cases failed the 3% precision gate, so it established no
  full-suite comparison. Separately, sequential wfgrep now compiles and runs
  through the normal command path and has one credited compute-bound win over
  the pinned system grep after the check-aware probe. That exact checkpoint is
  preserved evidence, not the 2x ripgrep flagship claim.
- **Claim boundary:** the suite must cover real source trees and large text;
  one and many files; several matcher families; ignore/filter work; and normal
  result production. A win on one file, `--sort`, fixed strings, a discarded
  output path, or a microbenchmark neither renames nor completes the flagship.
- **Missing / next:** the latency-floor question is answered (task 0026,
  preregistered): the lowering emits a check-aware 16-byte probe at
  recognized byte-walk loop headers — every observable byte, including
  every trap, still executes the unchanged scalar body — moving no-match
  instructions/byte 17.68 to 3.10 and lifting wfgrep vs the pinned grep
  from 0.753/0.762 to 1.069/1.071 (1.346 match-dense): the first full
  compute-bound win, with trap identity oracle-pinned on hostile bounds.
  The probe covers only the recognized byte-walk class; the verify subloop
  and copy loops stay scalar, and bounds traps remain secondary (~18%
  ceiling). On 2026-08-09 the owner parked every further wfgrep slice until the
  complete outline:PROOF-8 obligation-discharge sequence selected in the Current Plan
  was implemented and verified. That sequence is now terminal, so the exact
  credited checkpoint and full 2x objective are unparked rather than replaced.
  The next bounded wfgrep slice should be recorded in `current-plan.md` when
  selected; neither this outline nor that plan grants or withholds branch
  permission.
- **Directions tested:** outline:PERF-1 owns the baseline and attribution; outline:BOUND-1 and
  outline:VERIFY-1 enter with the real CLI/filesystem path; outline:PAR-1 through outline:PAR-4 enter
  only for measured parallel work and its proof/runtime contract; outline:FLOOR-1
  through outline:FLOOR-4 audit accepted source shape; outline:STORE-1 and outline:STORE-2 answer
  concrete matcher, queue, buffer, or result-representation blockers; outline:PROOF-1,
  outline:PROOF-2, outline:PROOF-3, outline:PROOF-7, and outline:VERIFY-3 enter only for an observed fact
  consumer.
- **Facts:** [project frame, source audit, and comparison rules](../research/notes/ripgrep-flagship-frame.md) ·
  [inconclusive RG-BASE attempt](../research/experiments/ripgrep/RESULTS.md) ·
  [pinned upstream release](https://github.com/BurntSushi/ripgrep/releases/tag/15.2.0) ·
  [pinned upstream repository](https://github.com/BurntSushi/ripgrep/tree/15.2.0).

The other candidate classes remain comparison evidence or optional separately
selected probes. They are not phases or prerequisites in front of ripgrep.

| Candidate | Primary outline items | Current disposition | Reopening condition |
|---|---|---|---|
| `outline:CAND-2` Compression / binary format | outline:PERF-1, outline:PROOF-1, outline:PROOF-7, outline:VERIFY-1, outline:BOUND-1 | LZ4 and the raw-DEFLATE/zlib evidence are parked; they remain useful binary-transform controls. | A separately selected binary-transform question has independent decision value, including as a bounded cross-check for a live general mechanism. |
| `outline:CAND-3` Parser / text validation | outline:FLOOR-1, outline:FLOOR-3, outline:FLOOR-4, outline:VERIFY-1, outline:BOUND-1 | The yyjson strict-reader frame and current text witnesses are parked. | A separately selected parser or storage question has independent decision value that the current plan does not answer. |
| `outline:CAND-5` Embedded / signal processing | outline:TARGET-2, outline:TARGET-3, outline:PROOF-5, outline:BOUND-1 | CMSIS-DSP remains parked; signal and image programs are internal evidence only. | A separately selected target/runtime question has an authentic Cortex boundary. |
| `outline:CAND-6` Declared parallelism (`later`) | outline:PAR-1 through outline:PAR-4 | BLAKE3 remains a recognizable anchor; ripgrep now supplies the live project pressure, while automatic profitable discovery remains rejected. | A separately selected explicit-parallel question has independent value beyond the ripgrep plan. |
| `outline:CAND-7` ML systems component (`later`) | outline:APP-1, outline:BOUND-2, outline:TARGET-1 | Llama inference remains a possible attention probe, not a ripgrep prerequisite. | A separately selected ML question has independent value and a bounded real-model boundary. |

## Settled exclusions and history

- Automatic profitable parallelism discovery is rejected: no compiler pass
  decides that overlapping is worth it. Deriving *permission* from proofs the
  compiler already computes is a different question and is live, with the
  writer-declared surface still the intended form for saying what a program
  expects. See outline:PAR-1.
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
