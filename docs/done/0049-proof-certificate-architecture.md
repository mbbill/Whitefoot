# 0049 — proof-certificate architecture research

This is frozen coordination history, not execution authority.

- **Status:** `DONE` (2026-08-10)
- **Authority:** separate owner-approved bounded research, 2026-08-10, limited
  to a decision-ready architecture packet; architectural clarity, correctness,
  and compile-time performance were the ordered selection criteria
- **Owner / workspace:** Codex lead /
  `<scratch-root>/whitefoot-proof-certificate-final`, branch
  `codex/proof-certificate-architecture-final`
- **Base revision:**
  `df55e7ca60e14fd32983738c92cef4528f0eda0c`, the separately landed Stage 5b
  plan-selection commit after terminal task 0048

## Outcome

The research rejects an immediate replacement of the canonical entailment
engine by an untrusted complete producer plus a fully independent exact
verifier. A small positive verifier can validate a used derivation, but exact
acceptance, `refuted`/`unproved`, `redundant`/`retained`, residual, and first
diagnostic behavior also depend on non-derivability and complete flow state.
Rechecking those properties independently would reproduce the analyzer and its
measured closure hot path.

The selected near-term architecture retains one deterministic completeness
engine and first repairs the complete DIAG-2 derivation ledger: every accepted
subscript, every discharged ordinary-call goal, and every S11 fact for every
`for_stmt`, including facts not later queried. After an independently walked
complete authorization inventory is stable, the same engine may atomically
seal one private lifetime-bound `EntailmentApprovedProgram` consumed by
lowering. That local capability clarifies and exhaustively binds no-check
authority without claiming to reduce the TCB.

A later bounded experiment may extract one trusted syntax-directed ProofFlow
and add a sparse positive verifier. It may replace the issuer of that same
capability only as the joint extractor/verifier/failure-atomic-publisher
boundary, only after exact equivalence and explicit cost gates pass. The
canonical engine remains the completeness authority unless a compact complete
non-derivability method is demonstrated. Research alone authorizes no stage.

## Landed work

- `99c6ab4` — registered the owner-approved bounded research as task 0049.
- From `c905891` through `2c8c6c5` — developed, adversarially reviewed,
  corrected, measured, and held the packet for task-0048 terminal authority.
- This terminal change — rebased the complete research history onto
  `df55e7c`, refreshed landed authority, finalized the packet, and moved this
  record from `docs/ongoing/` to `docs/done/`.

## Canonical evidence

- `research/investigations/proof-certificate-architecture/PACKET.md` owns the
  decision, current TCB inventory, worked certificate, hostile cases,
  performance evidence, seven-alternative comparison, migration boundaries,
  promotion gates, and owner decisions.
- Active v0.26 `spec/kernel-spec.md`, the current compiler implementation and
  focused tests, terminal task 0048, and
  `research/investigations/obligation-discharge/ACCEPTANCE.md` are the primary
  landed inputs.
- Relevant live MCTS nodes and their real rejected alternatives were consulted
  read-only through the required workflow. This task changed no MCTS state.

## Validation

- Task 0049 is a linear descendant of exact base `df55e7c`. Compiler tree
  object `2933aea434069209ad47e7f6d20b11ddd67b9442` and active-spec blob
  `82e3357c79ed9cc0bb5fc4dd2d0eecd909e9be69` are identical at `441cd5b8`
  and `df55e7c`; the active SHA-256 remains
  `18aa00e307642e608f2a3406642db9980dd3620291a7e434985e20a65eb0e476`.
  The MCTS tree object is unchanged from `d495d8c` through `df55e7c`.
- The final complete repository gate passes: 675 compiler library tests, 30
  real-program tests, 23 conformance-tool tests, canonical corpus, formatting,
  clippy, rustdoc, 128/128 rule coverage, repository invariants, and exact
  specification-chain validation are green. The known OWN-3 adapter boundary
  remains separately reported and unchanged.
- Release whole-compiler timings were 0.62 s for SHA-256, 2.30 s for UTF-8,
  and 1.47 s for the four-file boundary-fed DEFLATE unit. Sampling attributes
  99.29% and 96.79% of samples respectively below `entailment::state::close`;
  certificate construction and verifier overhead remain explicitly unmeasured.
- Two independent final adversarial reviews found no remaining P1/P2 issue.
  MCTS lint reports 77 nodes and zero fact files.

## Follow-up

The mandatory DIAG-2 repair, complete shadow inventory, canonical lowering
capability, optional ProofFlow/verifier experiment, and closure-performance
work each require separate owner selection, authority, and task registration.
The packet's promotion and stop gates are the handoff; this task changes no
compiler behavior, specification, Current Plan, Direction Outline, approval,
conformance verdict, or design memory. The temporary HANDOFF has been removed.
