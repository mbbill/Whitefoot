# Current Plan — claim residual canonicality

Status: PROPOSED (branch execution chartered by the 2026-08-21 owner direction
quoted in batch 0075). This plan does not itself expand that direction, and only
an owner-approved merge can make it authoritative on `main`. If chartered
branch execution exhausts the plan before that merge,
activation and completion occur atomically at the merge and the final recorded
status is `COMPLETE` rather than an intermediate persisted `ACTIVE` state.

Derived from Direction Outline revision 47 and main at
`4f01bab6a7bf158fff19dd54b062b748d20086d1`. Supersedes the completed
claim-only runtime-trap plan in place; its landed history remains in batch 0073
and git. Active language authority remains v0.33 at
`spec/kernel-spec.md`, SHA-256
`fc6b5a109e56b4bcd93d30ef934d3c78eca9bddafd640d30c10649e9ba62d08f`.

## Objective

Make every claim in a successful checker result one mechanically qualified,
individually necessary runtime-residual candidate at the normative checker's
proof frontier, and make every claim in a human-approved program an actually
proved residual.

A claim is neither an assertion, abort, test oracle, nor conditional. Its
predicate must be universally true at that point, unknown to the normative
checker, total and observational to evaluate, supported by a written human
derivation, and individually necessary for a fixed source-admission proof. If
the predicate can legitimately be false, source must express that possibility
with ordinary control flow: a branch, match, or loop transfer, or a typed
result, return, or exit status at the appropriate boundary.

The compiler mechanically enforces the residual shape and retains every
accepted claim at runtime. Human, AI-assisted, or offline-prover review owns the
truth of the remaining derivation. This plan does not claim that a fast
incomplete checker can decide arbitrary human mathematics.

## Selected semantic boundary

For one FN-1-reachable concrete inhabited claim occurrence `c`, before its own S3 fact, the
checker forms three exact typed images for every admitted predicate shape:
`D(P)`, the direct evaluated snapshot; `S(P)`, the support-canonical expansion
that stops below an exact projection or fixed normalization; and `F(P)`, the
fully structural still-valid ordinary-let expansion. It queries the unique
images in D-S-F order. This is not
limited to literals, comparisons, or `.defined` goals:

- if the pre-state is contradictory, reject it as vacuous rather than proving
  it by ex-falso; structurally unreachable statements are already FN-1 errors
  and never enter this lifecycle;
- otherwise, if one image has both signs, report a compiler consistency
  failure; if distinct equivalent images have opposite signs, reject the
  vacuous path rather than misreporting an internal failure;
- otherwise, if the checker proves its exact predicate, reject it as redundant;
- otherwise, if the checker proves its exact negation, reject it as refuted;
- if tentatively adding the predicate contradicts the established pre-state,
  reject it as inconsistent;
- otherwise `Unknown` is only a residual candidate, not permission by itself.

A candidate is mechanically accepted only when rules 1, 2, and 4 through 6
hold.
Rule 3 is the separate approval condition:

1. its condition satisfies a new total, deterministic, non-consuming,
   observational proof-predicate judgment and has one unique, versioned
   canonical contribution normal form `Contrib(P)`;
2. its `because` string passes a deterministic five-field structural schema for
   premises, derivation steps, exact conclusion, checker gap, and consumers;
3. human/AI/offline review validates those fields, confirms that the predicate
   is no stronger than the missing lemma and its consumers are authentic, and
   rules out the current claim, later facts, unstated environment promises, or
   circular proof;
4. `Contrib(P)` is derived only from S; every component `a` and its sound F
   lifecycle manifestations are unknown on both signs before S3; adding the
   whole contribution is consistent, reconstructs S, and materializes D with
   retained normative derivations, while F remains lifecycle-only;
5. for every component `a`, at least one closed-list source-admission root
   succeeds in the complete view and fails in `Full-minus(c,a)`, which suppresses
   only that component while retaining c's other components and every other
   member of the fixed `Eligible` set; at least one c-dependent predecessor
   lineage has c in its dynamic prefix, and the whole occurrence also passes
   `Full-minus(c)`; and
6. every selected root has a non-contradictory complete query state and its
   c-dependent derivation reaches the exact component/claim event without any
   contradictory or explosive predecessor; at a join, every reachable
   predecessor contributing to the root must independently have
   non-contradictory, non-explosive legal support, while c need dominate only
   the c-dependent lineage rather than every mutually exclusive route.

The fifth rule is the normative component- and occurrence-necessity judgment.
For a one-component claim the component and whole masks remove the identical
single S3 event, so one fresh counterfactual supplies both separately labelled
evidence roles without changing the judgment.
The existing all-claims-blinded U view plus one canonical
derivation is insufficient because another claim may be an unretained
alternative proof. Residuality is a simultaneous one-shot classification over
that fixed candidate set, not a fixed point that silently selects a survivor.
It proves checker-relative component and occurrence irredundancy, not a unique
proof basis, minimum claim count, or globally weakest proposition; proposition
minimality and consumer authenticity remain review obligations.

`Eligible` is fixed before any counterfactual: it contains exactly the
occurrences that passed every earlier machine check—predicate shape/effects,
   FN-1 reachability and D/S/F exact lifecycle, canonical-component lifecycle,
contribution consistency and S-to-D reconstruction, and the five-field `because` schema. If
any occurrence fails an earlier check, the unit reports that deterministic
earlier error and never starts residuality; an invalid claim never supplies S3
to another claim's baseline.

`Contrib(P)` is a proof-interface normal form, not a syntactic-leaf restriction.
Sound conjunctive information such as positive `band`, negative `bor`, and
`bnot` is normalized into signed components; a positive disjunction whose truth
does not imply either child remains one exact root component. Every admitted
Bool operator needs an operator-specific canonical conjunctive basis: xor and
equivalence may not default to singleton merely because the implementation
lacks their partial-known rules. S supplies the canonical component identities;
F supplies only equivalent or positive-only lifecycle manifestations, and D is
materialized only after S reconstruction. Comparisons, equality, and `.defined` follow a fixed
normative basis. The checker candidate adds finite, deterministic
truth-functional introduction over exact parent goals already present in the
goal universe. Thus known `A` plus residual `B` can reconstruct an exact
`band(A, B)` consumer, while `claim band(A, B)` is rejected when its `A`
component overlaps the checker. An irreducible true `bor(A, B)` can still be a
singleton residual. W3 must freeze this normalization and closure; ambiguity,
cyclic ancestry, missing reconstruction, or inability to express these cases is
a stop condition.

`claim False()` is therefore illegal, not an intentional trap form.
`claim True()` is redundant. A non-load-bearing claim is a source error, not
an optional unused-claim lint. Every claim that survives these rules remains
`retained`, contributes its exact source effects including `traps`, lowers
through the one ordinary path, and is evaluated at every dynamic reach in every
build mode.

Optional solvers never participate in ordinary source acceptance. An offline
proof success neither removes nor weakens a claim. A later proof rule or
explicit certificate becomes relevant only after it is installed as
deterministic normative authority; the author then replaces or deletes the
source claim and reruns the checker.

## Workstreams

- **W1 — freeze the complete claim surface and feasibility baseline.** Reproduce
  the current 241 claims in 23 real-program files, 410 claims in 197 protected
  files: 651 in-scope external `.wf` occurrences including 79 direct `False()`
  and five direct `True()`. Include the seven real and 36 protected current
  redundancy advisories and every inline Rust fixture. Record the aggregate
  presence of 15 claims in 13 dormant `tests/codegen` files and 14 claims in 13
  historical research probes, but exclude those files unchanged unless a live
  caller or gate is found; they create no per-item migration work. For every
  in-scope occurrence inventory
  predicate shape and origin,
  condition effects and ownership behavior, concrete instances and
  reachability, current lifecycle and ClaimLedger uses, terminal admission
  roots, `because` quality, effect owner and reverse caller closure, runtime
  oracle purpose, and first diagnostic. Treat word-count and oracle-word scans
  only as triage, never semantic verdicts.

- **W2 — prototype the exact individual counterfactual.** Reuse the existing
  flow with claim/component S3 masks to compute `Full-minus(c,a)` for every
  canonical contribution and `Full-minus(c)` for every whole occurrence,
  simultaneously over the fixed `Eligible` set after all pre-residual machine
  checks. Each mask leaves predicate evaluation and source effects intact and
  makes c's source event emit its basis minus the selected contribution identity
  and all of that identity's manifestations, then reruns the ordinary
  provenance-carrying closure from sources. No cached reconstructed parent or
  descendant survives unless an independent source rederives it. Define
  the closed terminal-root set as
  proof-required operation/allocation/bounds/system obligations, ordinary call
  requirements and mandatory complete FN-9 postcondition proofs. A protected
  root is eligible only after its ordinary provenance gate succeeds; the gate
  is attached audit evidence, not a separate mask witness. Exclude observational S7/S11/S12 roots, effect
  exhibition, CLM-3 structure, claim lifecycle, another claim, optimizer data,
  and test oracles. A root counts only when its exact complete query state is
  non-contradictory and its c-dependent derivation ancestry contains no
  contradictory/ex-falso predecessor. At a join, inspect every reachable
  predecessor lineage that contributes to the root: each needs independent
  non-contradictory, non-explosive legal support, and at least one c-dependent
  lineage must retain c in its dynamic prefix. The claim need not dominate a
  mutually exclusive sibling route.
  Measure absolute compiler wall time and retained memory on
  real bundles. If N counterfactual flows are materially too expensive, design
  and prove an exactly equivalent dependency analysis; never substitute U plus
  canonical ancestry.

- **W3 — prepare the normative source candidate and ordinary falsifiers.**
  Specify the lifecycle matrix, inconsistent continuation, individual
  necessity, closed terminal roots, generic schema and FN-1 reachability behavior,
  deterministic diagnostic ordering, and accepted runtime retention. Define
  the smallest proof-predicate form over compiler-known total/non-trapping
  primitives and stable observational reads, with no user/system call, write,
  allocation, external, block, nested trap, affine consume, release, cleanup,
  or hidden partial operation. Define `Contrib(P)` for every permitted shape:
  operator-specific sound signed conjunctive decomposition, justified singleton
  roots such as positive disjunction, and support-correct ordinary-let
  D/S/F lifecycle and manifestation mapping, equality, `.defined`, and `bnot`. Define the
  finite truth-functional introduction needed
  to reconstruct exact compound consumers from checker facts plus contributions
  without synthesizing new formulas. Keep
  `because` semantics as review data, but define a deterministic five-field
  STRING schema whose labels and nonempty shape are mechanically checked; the
  compiler's terminal-consumer inventory remains authoritative. Add ordinary
  hostile tests before implementation, including alternative claims, jointly
  necessary claims, mutually exclusive branch claims with per-predecessor join
  support, killed facts,
  contradiction after S3 and at a downstream root, a contradictory predecessor
  hidden by a non-contradictory join, exact-parent reconstruction from
  `known + residual`, positive-disjunction roots, partial-known xor/equivalence,
  per-component masks that remove solely dependent parents/descendants while
  preserving independent rederivation, claim-root versus child consumers,
  hidden compound origins and distinct D/S/F identity,
  equality and multi-condition `.defined`, an over-strong predicate, a
  manufactured dead consumer, OP/FN-8/FN-9 consumers, provenance, runtime
  oracles, parent-child-parent proof cycles, complete/U/B/claim-free/facts-off
  parity, generic disagreements, and CLM-3 ordering.

- **W4 — implement one deterministic compiler path.** Reuse pre-S3 lifecycle
  analysis and the existing flow/derivation infrastructure. Add proof-predicate
  checking; exact positive and negative lifecycle queries for every admitted
  canonical predicate/origin image rather than only literals or the current
  relation subset; canonical contribution construction, per-component
  D/S/F lifecycle and S3 masking, S reconstruction and D materialization, and the bounded
  truth-functional parent closure with deterministic non-cyclic ancestry. The
  claim event establishes the contribution basis directly—never parent then
  children—and parent introduction is one ordinary ENT rule shared by complete,
  U, B, claim-free, and facts-off checking, not a claim-only shortcut. Add the
  five-field `because` structure check; redundant,
  refuted, vacuous, inconsistent, and non-residual source issues; the
  `Full-minus(c,a)` and `Full-minus(c)` candidate stage over complete entailment
  reruns plus an unconditional U/B/PRV-1 provenance-invariance check; explicit rejection of any c-dependent proof
  ancestry containing contradiction/explosion; deterministic schema-first,
  source-occurrence-then-concrete-instance claim diagnostics, stable
  claim-source, canonical-component, terminal-root, masked-disposition, and
  provenance witnesses with no scratch IDs; and
  failure-atomic publication. Run early shape and
  lifecycle checks during flow, ordinary OP/FN/PRV judgments next,
  non-residuality only after their candidate succeeds, CLM-3 after claim
  validity, and checked-program publication last. Successful checked data and
  the ClaimLedger contain retained claims only. Accepted-claim IR, backend,
  trap records, and evaluation on every dynamic reach in every build remain
  unchanged.

- **W5 — migrate ordinary source by semantic purpose and audit every survivor.**
  Replace possible-failure, abort, impossible-arm, and test-oracle claims with
  an explicit branch, match, loop transfer, typed error, return, exit status, or
  purpose-appropriate test
  observation. Remove checker-proved and non-load-bearing claims. Restructure
  mixed generic instances rather than omitting checks per instance. For every
  genuine residual, rewrite `because` with premises, steps, exact conclusion,
  checker gap, the derivation of every canonical contribution, S reconstruction,
  D materialization, and terminal consumers, then perform and record human review. Recheck each
  whole compilation unit and repair the removed condition effects plus `traps`
  and reverse callers to a fixed point. Preserve intended non-claim values,
  observations, evaluation order, cleanup, and ownership; inventory every
  intentional trap/effect/code-shape delta. Only surviving genuine claims must
  preserve exact CLM-1 identity and lowering.

- **W6 — build and verify the exact branch candidate and merge packet.** Audit all
  protected cases, including cases currently rejected before lifecycle or
  counterfactual analysis can publish. Rewrite each according to its evidence
  purpose and flag each protected-class change in the batch record when it
  lands on the work branch. Keep specification bytes in CANDIDATE status.
  Before the packet, update the compiler README, derivation/claim-ledger
  description, Constitution, writer patterns, and AI-agent law on the branch;
  update `AGENTS.md` and `CLAUDE.md` together while preserving their shared
  rules and tool-specific differences; and use `mcts-mem-use` for the paired
  redecision that installs residual-only authority while retaining the old
  defensive/advisory choices as rejected history. Rebase the completed
  candidate onto then-current `main`, rerun all gates and audits, and produce
  the candidate SHA-256 and complete diff, complete specification impact
  inventory, native grammar-verifier output, accepted-set and first-diagnostic
  differentials, exact protected
  before/after audit, all-claim migration ledger, proof-predicate and
  contribution-normal-form census,
  generic simulation, effect closure, real-program results, runtime and
  code-shape canaries, counterfactual performance measurements, branch-tip
  full gates, audit dispositions, and unresolved review findings in one merge
  packet. The packet must also freeze the complete prospective
  activation/closure diff and every precomputable final byte, plus the exact
  mechanical input/output rule for approval-dependent ledger evidence.
  Materialize a canonical-placeholder form of that prospective tree without
  advancing the branch, record its template-tree identity, and run the
  prospective-tree gates. The packet must define the sole permitted final-tree
  function as that template plus the bounded owner-approval evidence bytes;
  approval authorizes only the output of that frozen function.

- **W7 — request merge and activate atomically.** Present the exact rebased
  branch packet for owner approval. Any substantive byte or scope change after
  presentation, including a rebase because `main` moved, re-enters branch audit
  and owner review. On approval, perform only the predetermined mechanical
  activation/closure transformation frozen in the packet. Fill only the
  bounded approval-evidence input, derive and record the final tree identity,
  then prepare one exact commit. The approval makes the plan
  authoritative and that same commit records `COMPLETE`, because the chartered
  branch already exhausted W1-W7; update the outline from branch-candidate to
  installed status;
  archive the outgoing specification; flip the candidate to its `ACTIVE`
  version; append the chained digest; regenerate the identity module and grammar
  tables; record the approval ledger; and apply the predetermined batch-record
  finalization/move. Commit those bytes once, independently verify the final
  tree identity, run the activation-side gates and artifact/target canaries,
  and fast-forward the already closed linear branch to `main`. Any mismatch
  from the packet's frozen transformation re-enters owner review. No new design,
  project-law, MCTS, specification-candidate,
  or protected-evidence bytes are authored after packet presentation.

## Judgment and diagnostic ordering

The selected ordering is part of correctness:

1. resolve and type-check the proof predicate;
2. enforce proof-predicate shape/ownership and the five-field `because` schema;
3. in a contradiction-first guard, query both signs of every unique D/S/F
   exact image in order and
   classify pre-S3 redundancy, refutation, pre-state contradiction, and
   post-S3 inconsistency in ordinary failure scratch;
4. construct `Contrib(P)` from S, classify each component through its S/F
   manifestations, and verify contribution consistency plus retained S
   reconstruction and D materialization;
5. complete ordinary operation, call, postcondition, effect, and provenance
   judgments;
6. on an otherwise successful fixed `Eligible` set, run simultaneous component
   `Full-minus(c,a)` and whole-occurrence `Full-minus(c)` residuality, discard
   roots with contradictory or explosive ancestry, and select the deterministic
   first machine-invalid claim;
7. run CLM-3 over valid retained claims only; and
8. publish checked data and lower every accepted claim unchanged.

This prevents a speculative unused-claim diagnosis from hiding a real later OP
or provenance error. A machine-invalid claim never reaches CLM-3, checked data,
IR, LLVM, object code, or an executable. A structurally qualified but
semantically false or dishonestly explained claim can still compile as an audit
candidate and trap at runtime; it can never enter an approved positive or
production program.

## Selected generic and reachability policy

Residual use is a source-admission property, not evidence that a runtime path
was observed from `main`. A dead nongeneric function can therefore contain a
residual when its body obligation needs it. The same standard means absence of
a concrete generic instantiation cannot by itself make a source claim
non-residual: an uninstantiated generic needs a parametric body-obligation and
audit judgment, while every inhabited concrete instance is additionally
rechecked. Uninhabited instances supply neither proof by ex-falso nor evidence
of use.

The implemented candidate retains the source-canonical symbolic inventory
formed during generic validation, installs its source-call requirements, and
runs the same lifecycle, contribution, component/whole counterfactual, terminal
root, non-explosion, and provenance judgments as a concrete inventory wherever
the symbolic vocabulary is normative. Generic integer/float type parameters
remain exact opaque Bool datums rather than invented L0 terms. Symbolic FN-9 is
a schema terminal only when its result and relation already use concrete
integer fragment types; generic-T FN-9 is concrete-instance-only. The stable
schema report owns identities by source declaration and NodePath, contains no
discarded scratch IDs or monomorph display-symbol authority, and links every
inhabited concrete report in stable order. Entry-uninhabited concrete instances
supply neither a report nor a witness.

A shared source claim whose inhabited instances disagree on
proved/refuted/residual status is rejected at the first stable concrete witness;
helper duplication may be the only repair. Structurally unreachable statements
remain FN-1 errors. A contradictory schema or concrete path is CLM-2-vacuous and
never proves a residual by ex-falso. There is no instance-local elision,
actual-instantiation-only reachability rule, or unaudited generic.

## Offline audit boundary

This plan authorizes review of claim derivations, not a general SMT,
proof-certificate, or serialized-artifact framework. A future audit packet may
bind exact source/spec/checker hashes, instance, typed predicate, pre-S3 state,
canonical contributions and support/kill manifestations, ownership/kill
snapshot, `because` steps, changed terminal roots, and earlier claim
dependencies.

A slow solver must first distinguish a contradictory/vacuous context from
proof of `P`. `Unknown`, timeout, resource exhaustion, unsupported
translation, and an unreplayed SAT model grant no approval. An independently
checked certificate must cover every theory used, contain no hole or trust
step, and still relies on the Whitefoot-to-logic translation. None of these
offline outcomes changes ordinary compilation or runtime execution.

## Acceptance

- Every successful checked-program claim is structurally a residual candidate:
  proof-predicate-valid; every member of its canonical `Contrib(P)` unknown and
  consistent pre-S3; S reconstructible and D materializable; every component individually
  necessary under `Full-minus(c,a)`; the occurrence necessary under
  `Full-minus(c)`; backed by non-explosive exact derivation ancestry; and
  retained at runtime.
- Every claim in the owner-approved candidate has a structurally valid and
  semantically reviewed `because` record with premises, derivation, exact
  conclusion, checker gap, and one or more terminal consumers. Review confirms
  proposition minimality and consumer authenticity. Dependencies are
  source-dominated and acyclic; an unreviewed compiler candidate is never
  represented as an approved theorem.
- `claim True()`, `claim False()`, a pre-proved or refuted predicate, a
  contradictory context, a predicate that contradicts the pre-state, an
  unused theorem, and an effectful or possibly nonterminating predicate each
  exercise deterministic machine rejection. Obvious runtime/test-oracle forms
  are ordinary falsifiers; disguised intent or a false checker-unknown theorem
  is an audit rejection, not a claimed compiler decision.
- U failure plus selected-proof ancestry is demonstrated insufficient by a
  two-alternative-claim test. Per-claim suppression rejects both alternatives
  while both exist; after authored removal, the one necessary residual is
  accepted. Mutually exclusive branch claims and jointly required claims remain
  expressible.
- A valid claim demonstrably supports each of an operation obligation, FN-8
  requirement, and FN-9 complete postcondition. A claim used only by
  observational metadata, an effect row, another claim, or CLM-3 is rejected.
- Generic all-valid, mixed redundant/retained, mixed
  residual/non-residual, uninhabited, and uninstantiated cases have explicit,
  deterministic results with no instance-local omission.
- An accepted claim enters successful checked data and the ClaimLedger, lowers
  to the ordinary claim IR, is never elided, is evaluated at every dynamic
  reach in every build mode, and emits the exact CLM-1/DIAG-3 record on a
  deliberately fault-injected proof failure.
- Any machine-invalid claim publishes no checked program, strict metadata,
  ledger, IR, LLVM, object, or executable. Any review-invalid candidate is
  barred from approved positive/production programs even if structural
  compilation produced temporary audit artifacts. Explicitly classified
  negative and backend/IR fault-injection fixtures may remain to test rejection
  and CLM-1/DIAG-3 runtime behavior.
- Source migrations preserve intended non-claim values, observations,
  evaluation order, cleanup, and ownership; every intentional effect, trap, and
  code-shape delta is inventoried. Exact effect-row repair proceeds from edited
  source, never compiler check elimination, and every surviving claim preserves
  its CLM-1 identity and lowering.
- The complete 651-claim in-scope inventory, explicit exclusion of unchanged
  dormant/historical `.wf` sources unless a live caller or gate is discovered,
  inline fixtures, protected
  before/after audit, first-diagnostic differential, generic simulation, and
  real-program wall-time measurements reconcile with the exact candidate.
- Native spec verification, compiler checks, real programs, canonical
  conformance, full repository gates, artifact canaries, MCTS lint, and the
  adversarial audit are green under the exact approved bytes.

## Stop conditions

- Any accepted claim is absent from runtime lowering or execution.
- Any optional solver, optimizer fact, target, profile, timeout, or runtime
  resource choice changes ordinary claim acceptance.
- The implementation uses all-claims U plus a selected derivation as proof of
  individual necessity, or an unsat core as proof of minimality.
- A claim contribution includes a checker-proved, refuted, or non-load-bearing
  component; canonical D/S/F, comparison, `bnot`, or `.defined`
  normalization is ambiguous; S cannot be reconstructed or D materialized; the bounded
  parent closure creates cyclic/explosive ancestry; or `Full-minus(c)` is
  represented as proof that a proposition is globally weakest or uniquely
  canonical.
- Boolean parent introduction differs across complete/U/B, claim-free, or
  facts-off checking, or a claim derivation is formed as a parent-to-child-to-
  parent proof cycle rather than one S3 source event establishing its basis.
- A component counterfactual deletes an independently supported fact, retains a
  cached fact whose only provenance used the masked component, or changes
  predicate evaluation, effects, ownership, or cleanup rather than S3 alone.
- The terminal-root set admits observational metadata, effect exhibition, a
  test oracle, another claim, or CLM-3 as the sole consumer.
- Proof-predicate admission allows ordinary behavior, affine consumption,
  cleanup, or possible nontermination.
- Contradiction or unreachable code proves a claim by ex-falso, or an added
  claim creates contradiction and then discharges later obligations by
  explosion.
- A semantically over-strong predicate or a manufactured dead consumer passes
  human/AI/offline review merely because occurrence-level residuality succeeded.
- A generic implementation omits a claim only for selected instances, exempts
  an unaudited generic silently, applies actual-instantiation reachability only
  to generics, or cannot offer an acceptable source repair for the measured
  mixed-instance cases.
- A migrated possible-failure condition remains a claim instead of explicit
  control flow.
- A compiler implementation omission is presented as checker incompleteness
  even though the active normative fragment already requires the derivation.
- `because` is treated as trusted semantics, or a vague label is presented as
  a derivation.
- An offline `Unknown`, timeout, stale result, unsupported encoding,
  unreplayed model, or certificate trust step is reported as proof.
- A protected or first-diagnostic delta escapes the exact reviewed inventory,
  an effect closure reaches an unreviewed interface, or candidate bytes or
  scope change after merge-packet presentation without renewed review.
- Exact counterfactual implementation is not tractable on real bundles and no
  equivalent fixed judgment has been demonstrated. Stop for direction rather
  than weaken residuality.

## Exclusions

No `pal` or parallelism implementation, FFI, export adapter, dynamic dispatch,
general SMT/Boolean solver, proof-certificate language, generalized serialized
proof packet, writer `assert`/`expect`/`assume`, intentional-abort claim,
optimizer claim elimination, instance-local claim omission, compatibility
mode, unrelated wfgrep performance work, or generic-container project is
authorized by this plan.
