# 0048 — make `requires` one atomic call-site goal

- **Status:** `IN PROGRESS`
- **Authority:** the ACTIVE stage-7 step in `docs/current-plan.md`, derived
  from Direction Outline revision 27
- **Owner / workspace:** Codex lead / `/Users/bytedance/code/Whitefoot`, branch
  `codex/0047-counted-range-impl`
- **Base revision:** `7eb78ab7ba36bafbb68f1b041104596f1a2d8b21`

## Goal

Replace the executable ordinary-callee `requires` prologue with one finite
atomic call-site proof goal while preserving the complete admitted FN-8
pure/total Bool surface. Give the callee body the verified goal as S4, preserve
the same checked failure at both real process entries, and retain the finite
subject-only bridge that closes provenance bypass O3 without activating the
held PRV gate.

## Direction and invariants

- A clause's lets alpha-expand into one typed structural predicate. The whole
  Bool DAG is atomic: no commutation, algebraic normalization, decomposition,
  Boolean composition, or O11 proof rule. Predicate equality excludes callee
  and NodePath identity; requirement and protected-leaf occurrences retain
  those identities for diagnostics and provenance.
- Every ordinary call proves the instantiated goal in its incoming full state
  after actual-expression obligations and borrow feasibility, but before
  consume/borrow commits and callee-effect kills. Unproved or refuted goals are
  call-site errors. No ordinary fallback runtime check exists.
- Signed opaque facts use ordinary support, kill, join, and loop rules. A
  combined L0/opaque contradiction is all-derivable and suppresses refutation
  exactly like current unreachable ENT-4 state. An exact existing L0 relation
  may prove the atomic root; no second theorem language is introduced.
- The body receives S4 as an axiom; a `requires` declaration contributes no
  callee effect. Explicit caller checks and claims retain their runtime traps.
  No proved goal becomes `llvm.assume` or an optimizer-only fact.
- Unlabelled and command process wrappers evaluate the goal once, directly and
  non-consumingly, before transferring any source owner to the body. There is no
  owner-taking helper thunk, duplicate body, duplicate release, fake foreign
  adapter, or new FFI surface. The existing future gated-boundary promise stays
  checked and the currently unimplemented path stays unsupported.
- O3 metadata is a finite least fixed point over protected ENT-6 leaves,
  requirement occurrences, ordinary calls, and parameter datums. Direct local
  and inherited call bridges are created only when S4-present succeeds and the
  same S4-blinded unasserted state fails. Subject datums come only from the
  protected obligation's constrained subject, never from every goal operand.
  Stage 7 retains and rewalks this bridge but emits no PRV rejection.
- Active specification bytes remain at `spec/kernel-spec.md`. The branch may
  prepare v0.26 and an exact outgoing v0.25 archive, but activation requires a
  Chinese owner-facing explanation followed by a hard wait for explicit exact
  approval. While v0.26 is active there must be no versioned v0.26 file.

## Method

1. Freeze v0.25 identity, all FN-8 declarations/calls/effect rows, both real
   entry wrappers, protected cases, real base64/deflate consumers, and the held
   O3 measurement. Consult the live requires/proof/effect design nodes and real
   alternatives through the MCTS-Mem skill.
2. Draft and independently review the smallest complete stable-file v0.26
   delta and exact v0.25 archive. Pin goal identity, call order, opaque-state
   contradiction, S4/effect changes, process-entry ownership, diagnostics,
   protected impact, O3 bridge generators, and finite convergence before
   implementation. Any spec-byte change restarts digest review.
3. Implement one typed goal representation and instance substitution path in
   checked semantics. Add signed goal facts, support/kills/joins, L0 projection,
   call discharge, body S4, deterministic diagnostics, effects, and bridge
   metadata through ordinary compiler structures rather than a general graph
   or theorem framework.
4. Remove the callee prologue from lowering. Inline the pure goal evaluation in
   each real process wrapper without taking ownership, then transfer each
   successful command owner once to the body. Preserve OP-5 failure bytes and
   EFF-4 trap cleanup behavior.
5. Apply only the protected dispositions preselected by the ACTIVE plan. Keep
   equality and base64 requirements exact with explicit caller evidence;
   repurpose the stale missing-traps and false-requires cases; make
   requires-only bodies pure; keep the FN-3 and noncopy-local cases focused;
   update output-capacity assertions from prologue to caller proof/body axiom.
   Any additional protected drift stops for review.
6. Rewalk the O3 helper, multi-hop/local-transform/recursive controls, and all
   three real `store_dynamic_length` calls. Update derived material and design
   memory only when the new language and implementation activate. Present the
   complete Chinese owner explanation and exact digest, then stop and wait.
7. After explicit approval, atomically install the stable spec, outgoing
   archive, compiler, protected deltas, active pins, approval chain, derivation,
   docs, and MCTS re-decision. Rerun installed acceptance and close this task in
   a separate canonical closure change before stage 5b begins.

## Progress

- **Completed:** task 0047 closed at `7eb78ab`; Direction Outline revision 27
  and the ACTIVE plan fix the Stage-7 semantics, protected dispositions,
  stable-file workflow, and stop boundary. Three independent plan reviews found
  no remaining P1/P2 after the O3 fixed point, contradiction, predicate
  identity, entry ownership, and MCTS activation scope were closed.
- **Current:** freeze the v0.25 FN-8 surface and prepare the exact v0.26
  normative candidate before substantive compiler changes.
- **Next:** independently review the complete candidate digest and impact, then
  implement regression-first through the ordinary semantic and backend paths.

## Scope and expected touch set

- Specification/governance: stable `spec/kernel-spec.md`, new outgoing
  `spec/kernel-spec-v0.25.md`, derivation ledger, `governance/APPROVALS.md` only
  after owner approval, active spec pins, grammar verification if bytes affect
  frontend contracts, and the stable archive/chain gates.
- Checked semantics: FN-8 clause checking and typed goal model; concrete
  instantiation/substitution; call ordering; entailment terms/state/flow,
  opaque support/kills/joins/S4/L0 projection; diagnostics; effects; and finite
  O3 bridge/PRV-2 metadata with focused tests.
- Lowering/backend: remove ordinary prologue operations; direct process-wrapper
  goal evaluation; OP-5 record; command input ownership, body transfer, and
  release controls; no new public IR or foreign entry surface.
- Consumers/evidence: the exact protected cases named in the ACTIVE plan,
  base64 and raw-deflate programs/tests, frozen obligation acceptance,
  conformance manifest and adapter, writer docs, roadmap/current plan, this
  record, and activation-time MCTS updates to `whitefoot`,
  `checks-and-proofs`, `requires-entry-contract`, `obligation-discharge`, and
  `effects` under the skill workflow.

## Dependencies and integration order

- Terminal task 0047 and activation `3e2e823` are the premise; closure
  `7eb78ab` is this task's base. No parallel task may activate a specification
  or change FN-8, ENT goal flow, entry-wrapper ownership, or PRV bridge identity.
- Exact normative semantics and impact review precede compiler implementation.
  General semantic discharge precedes protected migration and entry lowering.
  Exact approval precedes the one atomic activation commit; installed
  acceptance precedes task closure. Stage 5b remains unauthorized until this
  task is terminal.

## Validation

- Goal identity: alpha/local-sharing positives; operand/order/operation/const/
  substitution negatives; opaque `band`/`bor`/`bnot`; exact L0 projection;
  true, false, missing, killed, joined, contradictory, and unreachable states.
- Calls: actual obligations and borrow feasibility before proof; proof before
  transfer/effect kills; repeated call after kill; forward, generic, direct
  recursive, and mutually recursive order independence; exact diagnostics.
- Effects/body: no ordinary callee check, trap branch, body clone, fallback, or
  `llvm.assume`; S4 still discharges body obligations; exact pure/trapping rows.
- Entries: both real forms; true/false goal; one external `@main`; startup
  ordering; `Args`, `DirectoryRead`, stdout, and stderr owners consumed/released
  at most once; body zero/one execution; no foreign stub.
- O3: direct, two-hop, local-transform, recursive/mutual, and seedless-cycle
  bridge controls; subject-only bound/base negatives; claim/check full-only,
  branch unasserted positive; three deflate calls unasserted-positive with no
  source change and retained distance claim.
- Integration: exact stable/archive identities and chain; both native grammar
  paths; generated/derivation identity; protected before/after matrix; focused
  semantic/lowering/backend tests; `make -C compiler check`; `make check`;
  ignored adapter tally/rules; frozen acceptance; facts-off equivalence; MCTS
  lint and paired re-decision.

## Stop condition

Stop with the smallest reproducer if one atomic typed goal cannot preserve the
existing FN-8 surface, call ordering or owner identity requires a duplicate
runtime/body path, O3 needs whole-goal support or Boolean decomposition,
process entry requires duplicate affine ownership, an unlisted protected
verdict or behavior changes, or exact spec/compiler/archive activation cannot
satisfy the stable workflow. Do not narrow contracts, retain a hidden prologue,
invent an adapter, weaken a verdict, or smuggle in O11/general theorem proving.

## Closure

Move this record to `docs/done/` only after exact v0.26 activation, installed
acceptance, real-program and adapter results, O3 bridge evidence, protected
impact, and design re-decision are in their canonical homes. Positive closure
must replace the ACTIVE plan with stage 5b; a reproduced blocker returns for
owner disposition instead of skipping ahead.
