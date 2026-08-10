# 0050 — activate the bounded provenance gate

- **Status:** `IN PROGRESS`
- **Authority:** the ACTIVE `Current step — stage 5b provenance-gate
  activation` in `docs/current-plan.md`, owner-selected on 2026-08-10 and
  derived from Direction Outline revision 29 item `PROOF-8`, with `BOUND-1`,
  `VERIFY-1`, and `VERIFY-2` as boundary and evidence constraints
- **Owner / workspace:** Codex lead / `/Users/bytedance/code/Whitefoot`, branch
  `codex/0047-counted-range-impl`
- **Base revision:**
  `63e3407b997cce0716266ce6d7f6dc6039df92ab`

## Goal

Activate the already bounded PRV-1/PRV-2/PRV-3 explicit-dataflow gate against
active v0.26 and its requirement-to-protected-leaf bridge. Reject
assertion-backed discharge only when the constrained protected subject is
external, migrate exactly the eleven named real claims to the plan-selected
value paths, including the one `Result` propagation, and atomically activate
v0.27 after exact owner approval while preserving every frozen protected case,
oracle, unaffected effect judgment, and facts-on/facts-off behavior.

## Direction and invariants

- Re-derive the judgment from active v0.26, task 0046's held review, and the
  v0.26 requirement bridge. The held v0.24-era candidate remains evidence and
  is never fuzzy-patched into the stable specification.
- PRV-1 is exactly the plan's finite two-point explicit-dataflow
  classification and closed system-component table. It has direct payload
  projections, per-binding/per-root flow-insensitive monotone storage,
  root-plus-explicit-offset place reads, internal `len`, and one finite least
  fixed point. It adds no implicit-flow, write-address, path-sensitive,
  recursive-payload, or general theorem machinery.
- PRV-2 retains finite parameter-datum, result, write, and concrete
  protected-leaf identities and selects deterministic witnesses only after
  convergence. It composes with exact v0.26 requirement occurrences and
  subject-only bridges; it introduces no recognizer or second goal language.
- PRV-3 gates only the constrained subject. Internal subjects keep existing
  entailment; external subjects must discharge with S2/S3 removed. Real value
  branches may prove the goal, while unrelated external operands do not
  trigger rejection. Call gating follows the exact v0.26 bridge fixed point.
- Command inputs are external. The retained S4-blinded entry rewalk prevents
  the compiler wrapper or body S4 axiom from laundering a bridged protected
  leaf. Unrelated entry requirements retain exactly-once checking; no foreign
  adapter is added.
- Implement one ordinary safe-Rust semantic path over existing checked
  metadata. Facts-on and facts-off have identical acceptance and required
  runtime behavior. No project-, function-, claim-, source-, or test-shaped
  special case is permitted.
- Preserve the four frozen source identities, existing conformance manifest
  and 407 cases, 30 coverage annotations, all protected behavior, and every
  successful/error oracle. New PRV cases are additive only. Apply exactly the
  eleven error mappings and one `store_dynamic_length -> Result` change named
  by the ACTIVE plan.
- Prepare the v0.27 stable candidate and byte-identical outgoing v0.26 archive
  uncommitted, review and hash them independently, give the required complete
  Chinese explanation, and hard-wait for exact owner approval. Only then may
  atomic activation, approval-chain changes, and live MCTS updates land.

## Method

1. Freeze the exact source, manifest, case, coverage, effect, runtime, and
   active-spec identities named by the plan; reproduce every pre-migration
   provenance matrix and hostile control before changing semantics.
2. Consult the relevant live design nodes and rejected alternatives through
   the `mcts-mem-use` workflow.
3. Draft and independently review the smallest complete PRV-1/2/3 v0.27 delta
   plus the byte-identical outgoing v0.26 archive. Any spec-byte change
   restarts digest and impact review.
4. Add focused regressions, then implement the finite classification, witness
   fixed point, subject-only gating, diagnostics, call bridges, entry rewalk,
   and facts-off equivalence through the normal semantic path.
5. Migrate only the eleven named claims to their exact existing value paths;
   change only `store_dynamic_length` and its three propagation call sites as
   directed, and check all cleanup and effect edges.
6. Add only additive PRV conformance cases and coverage; update generated and
   derived specification data, compiler/writer documentation, acceptance
   evidence, Direction Outline, and relevant design memory.
7. Complete independent normative, implementation, protected-impact,
   real-program, derivation, archive, and pin reviews; present the exact digest
   and Chinese explanation, then stop for owner approval.
8. After approval, land one atomic activation, rerun every installed matrix,
   oracle, adapter, compiler, repository, and MCTS gate, then close the task in
   a separate canonical closure change.

## Progress

- **Completed:** task 0048 is terminal at `b882484`; v0.26 activation
  `441cd5b`, task 0046 held evidence `5683c85`, ACTIVE plan selection
  `df55e7c`, and the current ACTIVE plan establish the Stage 5b premises. The
  refreshed integration base also contains terminal independent research task
  0049 at `63e3407`; it supplies no Stage 5b authority. Registration landed at
  `ecfa57e`. On that exact clean tree an independent scratch rewalk reproduced
  `33/23`, `19/6/13-under-11/14`, canonical `3/3`, and the symbolic diagnostic
  projection `14/24`; every frozen boundary control and input identity also
  matched. The rejected contextual interpretation would double-report two
  parameter-dependent calls as `16/28` and is not the finite symbolic
  [PRV-2] judgment. An uncommitted v0.27 candidate and byte-identical outgoing
  v0.26 archive are frozen for review at SHA-256
  `6fa5fcf374f75145ae58005a6ca54f8c62ec87058557a4d3893a9bdcaf8bdcdf`
  and `18aa00e307642e608f2a3406642db9980dd3620291a7e434985e20a65eb0e476`;
  they remain non-authoritative and unreviewed pending the stop-condition
  disposition below.
- **Current:** stopped before compiler implementation. Two independent
  read-only consumer migrations reproduced the same exact [EFF-2] cascade:
  after the eleven selected claims become value branches and
  `store_dynamic_length` loses its claim-derived `traps`, `decode_length`, then
  `copy_distance`, then `decode_fixed` each has no remaining trapping source
  and must also lose `traps`; after those three removals all four sources pass
  semantic checking and no fourth effect row changes. The ACTIVE plan instead
  requires every other effect judgment to remain unchanged, so an executor
  cannot honestly continue without owner disposition.
- **Next:** ask the owner whether to amend the ACTIVE plan by authorizing those
  three forced effect-row removals in addition to the already selected
  `store_dynamic_length` change. If selected, record the exact amendment and
  restart independent normative review before implementation; otherwise
  terminally dispose the task without activating the frozen candidate.

## Scope and expected touch set

- Specification/governance: `spec/kernel-spec.md`, new
  `spec/kernel-spec-v0.26.md`, derivation and active-spec identity data,
  grammar/generated data where required, and `governance/APPROVALS.md` only
  after exact approval.
- Compiler: provenance and requirement-bridge checked metadata, finite semantic
  fixed points, diagnostics, entry rewalk, facts equivalence, and focused
  semantic/lowering/backend tests, including
  `compiler/src/semantic/provenance.rs` and its tests.
- Consumer: the four frozen `tests/programs/raw_deflate*.wf` compilation-unit
  files and their real-program tests, limited to the eleven prescribed value
  repairs and one prescribed result propagation.
- Conformance/evidence: additive PRV cases and coverage under
  `tests/conformance/`, unchanged existing identities and manifest fields,
  frozen obligation-discharge acceptance, adapter evidence, and exact runtime
  oracles.
- Documentation/memory: `compiler/README.md`, writer guidance,
  `docs/roadmap.md`, the successor proposal in `docs/current-plan.md`, this
  record, and only the relevant MCTS nodes selected through the skill workflow.
  The held provenance candidate and task-0046 evidence are inputs, not active
  specification text or implementation authority.

## Dependencies and integration order

- Task 0046 closure `5683c85` supplies held design evidence; v0.26 activation
  `441cd5b` and task 0048 closure `b882484` supply the bridge and active-language
  premise; plan selection `df55e7c` supplies execution authority.
- Task 0049 is terminal at the refreshed base `63e3407` and is independent of
  this task's Stage 5b language and compiler scope. It creates no dependency or
  integration-order constraint for this task.
- Normative derivation and review precede implementation; general semantic
  gating precedes consumer migration; exact approval precedes atomic
  activation; installed acceptance precedes terminal closure. No parallel task
  may change PRV semantics, requirement-bridge identity, the stable
  specification, or frozen protected material without an explicit cross-link
  and integration order.

## Validation

- Reproduce pre-migration `33/23`, external `19`, unasserted discharge `6`,
  rejection `13` under `11` claims, internal `14`, Huffman `3/3`, and diagnostic
  projection `14` calls / `24` atoms.
- Preserve negative controls: wfgrep `0/8`, `run-sysfile-multichunk` `0/4`, and
  every too-small/invalid-copy control `0/1`.
- After migration reproduce UTF-8 `33/22/11/0`, SHA-256 `9/9/0/0`, complete
  DEFLATE `29/24/5/0`, and dynamic DEFLATE `24/19/5/0`; retain twelve claims
  and establish all thirteen former claim-supported sites by real branches.
- Cover external+branch accept, external+check/claim reject, internal+claim
  accept, external-only-bound accept, exact/nonexact call goals, direct,
  two-hop, recursive, mutual and seedless bridges, payload sibling isolation,
  offset propagation, retained implicit-flow boundaries, and entry bridged
  rejection.
- Preserve every existing conformance byte and manifest field; retain existing
  adapter `Pass=393 Fail=1 Skip=13` with only OWN-3 unsupported.
- Verify both grammar paths, generated tables, specification/archive integrity,
  exact diagnostics, focused semantic/lowering/backend tests,
  facts-on/facts-off equivalence, the complete adapter, frozen four-source
  consumer, `make -C compiler check`, `make check`, and MCTS lint.

## Stop condition

Stop with the smallest reproducer if the current v0.26 sources do not reproduce
the frozen 19/6/13-under-11/14, 3/3, and 14/24 matrices; if process-entry gating
needs a new source surface or error protocol; if correct classification
requires control-flow taint, write-address taint, path-sensitive storage,
recursive payload paths, Boolean decomposition, general induction, or a new
theorem prover; if the eleven real repairs cannot use the named existing value
paths and one `Result` propagation; or if an existing protected case, verdict,
rule list, status, documentation field, or runtime behavior must change. Return
that evidence for owner disposition rather than expanding the gate, weakening a
test, retaining a hidden assertion, or skipping ahead.

## Closure

Move this record to `docs/done/` only after exact owner-approved atomic v0.27
activation; installed reruns of every frozen matrix, runtime oracle, adapter,
compiler and repository gate; and canonical updates to specification identity,
approval, derivation, acceptance evidence, Direction Outline, documentation,
and design memory. Positive closure must make Stage 5b terminal before Stage 8a
and must not authorize Stage 8a without a separately ACTIVE plan. A reproduced
stop condition returns evidence for owner disposition and the appropriate
terminal status instead of expanding the task or skipping ahead.
