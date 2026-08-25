# 0048 — make `requires` one atomic call-site goal

This is frozen coordination history, not execution authority.

- **Status:** `DONE` (2026-08-10)
- **Authority:** the ACTIVE stage-7 plan derived from Direction Outline
  revision 28 and the owner's exact approval of v0.26 digest
  `18aa00e307642e608f2a3406642db9980dd3620291a7e434985e20a65eb0e476`,
  including the nine protected source changes, eight manifest documentation
  changes, and outgoing v0.25 archive
- **Owner / workspace:** Codex lead / `<repository-root>`, branch
  `codex/0047-counted-range-impl`
- **Base revision:** `7eb78ab7ba36bafbb68f1b041104596f1a2d8b21`

## Outcome

v0.26 is active at the stable `spec/kernel-spec.md` path and v0.25 is frozen at
`spec/kernel-spec-v0.25.md`. Every admitted function requirement is one finite
typed atomic goal. Ordinary calls prove the exact instantiated goal after
actual obligations and borrow feasibility but before transfer and callee-effect
kills; refuted and unproved calls reject. The body receives the proved goal as
S4 without an executable ordinary-callee prologue or effect contribution.

The two real process wrappers retain one checked boundary and preserve exact
failure, ownership, cleanup, and body-call behavior. Signed goal evidence keeps
exact support, kill, join, loop, and contradiction semantics; only an exact
comparison root projects to L0. The checked program retains finite subject-only
requirement bridges, counterfactual rewalks, and deterministic predecessors.
Direct, two-hop, local-transform, recursive, and mutually recursive bridges
converge while a seedless cycle stays empty. This closes the O3 helper bypass
without activating provenance rejection.

## Landed work

- `b11e22f` — registered task 0048 after terminal task 0047.
- `441cd5b8` — one atomic v0.26 activation: exact stable bytes and outgoing
  v0.25 archive, eighteenth approval-chain link, general compiler
  implementation, active pins, derivation, writer documentation, approved
  protected migration, real-program updates, and activation-state authority.
- `d495d8c` — recorded the paired requirement-enforcement re-decision and froze
  the superseded unconditional callee-entry prologue in design history.
- This closure change — installed acceptance, Direction Outline revision 29,
  the proposed stage-5b replacement plan, and this move from `docs/ongoing/`
  to `docs/done/`.

## Canonical evidence

- `spec/kernel-spec.md`, `governance/APPROVALS.md`, and
  `spec/derivation/derivation-ledger.md` own the language identity, approval
  boundary, and derivation.
- `research/investigations/obligation-discharge/ACCEPTANCE.md` owns the reviewed
  and installed frozen buckets.
- `compiler/README.md`, focused requirement/entailment/provenance/lowering/
  backend tests, and the real program suite own compiler and executable
  evidence.
- `mcts_mem/whitefoot/checks-and-proofs/requires-entry-contract.md` and its
  `requirement-enforcement` child own the live design and paired predecessor.
- Direction Outline revision 29 owns current status. The replacement Current
  Plan is a proposal awaiting owner selection, not execution authority.

## Validation

- Installed SHA-256 identities are
  `18aa00e307642e608f2a3406642db9980dd3620291a7e434985e20a65eb0e476`
  for active v0.26 and
  `c0b3c279f4c20d06da17ef7ac0e4ec882c8a76c560f62cce47d5b4fd4ac6beab`
  for the byte-identical outgoing v0.25 archive. No v0.26 archive exists.
- `whitefoot-spec` reports v0.26, 128 rules, and 18 unbroken activations;
  archive integrity reports 27 recorded identities. Native grammar is 70
  productions, 85 decisions, and 96 terminal predicates, with committed tables
  matching exactly.
- The installed complete gate is green: 675 library tests, 30 real-program
  tests, 23 conformance-tool tests, canonical corpus, formatting, clippy,
  rustdoc, 128/128 rule coverage, repository invariants, and exact spec-chain
  validation pass.
- The separately invoked adapter reports `Pass=393 Fail=1 Skip=13`. Its sole
  divergence remains `own3-pos-outlives-store` at the pre-existing
  `RegionsAndBorrows` unsupported boundary.
- Installed frozen acceptance is UTF-8 `33/22/11/0`, SHA-256 `9/9/0/0`,
  complete DEFLATE `29/11/18/0`, and dynamic DEFLATE `24/11/13/0` in
  total/proven/claim-supported/baseline-undischarged order. DEFLATE retains
  sixteen claims, five non-rejecting redundancy advisories, and zero refuted
  claims; no proven site regresses.
- The canonical O3 result is 3/3. All three real `store_dynamic_length` calls
  discharge in both unasserted and S4-blinded rewalks. Requirement, entailment,
  provenance, entry, protected base64, and runtime-oracle controls pass.
- MCTS lint reports 77 nodes and zero fact files after the paired re-decision.

## Follow-up

Direction Outline revision 29 names provenance activation as the next gate, and
the replacement Current Plan contains a complete stage-5b proposal. It is not
ACTIVE: owner selection must precede execution. No successor task is registered
by this closure. After plan approval lands, the next free task number after
refreshing the integration branch must be registered in a separate lifecycle
commit before substantive work begins.

The held v0.24-era provenance prose must be rederived against active v0.26. Any
v0.27 candidate still follows the stable-file workflow and requires a complete
Chinese owner explanation followed by a hard wait for exact approval. Stages
8a, 8b, 9a, and 9b remain later dependencies; wfgrep stays parked.
