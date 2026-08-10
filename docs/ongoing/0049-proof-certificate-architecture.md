# 0049 — proof-certificate architecture research

- **Status:** `IN PROGRESS`
- **Authority:** separate owner-approved bounded research, 2026-08-10; the
  temporary `research/investigations/proof-certificate-architecture/HANDOFF.md`
  and the owner's follow-up prioritize architectural clarity, then correctness,
  then compile-time performance. This authority permits research and a
  decision-ready packet only.
- **Owner / workspace:** Codex lead /
  `/Users/bytedance/do_not_scan/whitefoot-proof-certificate-research`, branch
  `codex/0049-proof-certificate-research`
- **Base revision:** `b11e22f1901dc9e59cac79a9250d709e4a2082a8`, plus a
  read-only snapshot of task 0048's uncommitted v0.26 candidate taken on
  2026-08-10. The snapshot is evidence under review, not landed authority.

## Goal

Decide whether Whitefoot should retain one deterministic entailment engine or
separate proof discovery from a small independent certificate verifier, and, if
the split is justified, identify the smallest behavior-preserving pipeline that
makes safety correctness directly auditable without disproportionate compiler
complexity or compile-time cost.

The decision bears on Direction Outline items `PROOF-1`, `PROOF-8`, and
`VERIFY-3`. The hypothesis is that an untrusted producer plus a materially
smaller verifier can clarify the check-elision trust boundary while preserving
the exact normative derivability relation. The research must reject that
hypothesis if the verifier duplicates the checker, deterministic acceptance
cannot be preserved, or the machinery costs more than the capability it serves.

## Direction and invariants

- Architectural clarity is the first selection criterion: proof discovery,
  normative validation, diagnostics, and lowering authority must have explicit
  responsibilities and one-way dataflow rather than shared implicit state.
- Soundness is non-negotiable. A bad or missing certificate fails closed and
  never authorizes removal of a required runtime check.
- An implementation-only proposal must preserve source acceptance, claim
  retained/redundant/refuted outcomes, canonical residuals, deterministic first
  rejection, runtime behavior, and facts-on/facts-off identity.
- Explicit `claim` remains an executed named runtime check even when redundant.
  A stronger checker may expose redundancy or newly refute a reachable claim;
  the compiler does not silently rewrite source.
- Certificates remain internal checked-program data. External proof inputs,
  source proof terms, new proof rules, or acceptance changes are language or
  toolchain proposals and stop this research for owner disposition.
- Compile-time, memory, and artifact costs are measured or honestly bounded.
  No caching, portable identity framework, replay protocol, or generalized
  theorem infrastructure is introduced without current evidence.
- This task does not modify compiler behavior, the active specification,
  conformance verdicts, the Current Plan, the Direction Outline, approvals, or
  MCTS-Mem.

## Method

1. Refresh and distinguish the landed active authority from task 0048's
   candidate snapshot; before final synthesis, rebase onto task 0048's terminal
   revision and reread every changed authority and implementation boundary.
2. Consult the live `whitefoot`, `checks-and-proofs`,
   `obligation-discharge`, `requires-entry-contract`, `fact-channels`, and
   related proof/effect decisions and their real rejected alternatives through
   the `mcts-mem-use` workflow, without editing the tree.
3. Run three bounded read-only investigations: current TCB and metadata flow;
   minimal certificate/verifier models with worked derivations; and hostile
   review of determinism, completeness, recursion, identity, diagnostics, and
   compile cost. Each finding must cite primary evidence or be marked unknown.
4. Compare the status quo, derived-witness-only, producer/verifier,
   proof-carrying checked IR, untrusted SMT producer, language proof objects,
   and implicit runtime fallback against the same criteria.
5. Synthesize one
   `research/investigations/proof-certificate-architecture/PACKET.md` containing
   the authorization, current-state inventory, worked certificates, adversarial
   results, cost evidence, migration path, owner decisions, and one explicit
   implement/defer/status-quo recommendation.

## Progress

- **Completed:** owner research boundary understood; workflow, current outline,
  Current Plan, task 0048, and the relevant live design-memory decisions and
  real alternatives read; isolated worktree created; the complete uncommitted
  task-0048 snapshot copied without changing its source workspace; three
  independent read-only investigations reconciled; current TCB, lowering
  authority, identity, hostile cases, seven alternatives, and migration gates
  drafted in `PACKET.md`; candidate release timings and two sampling profiles
  recorded.
- **Current:** task 0048's v0.26 activation commit `441cd5b` has landed and its
  implementation blobs match the captured research snapshot, while its
  one-shot installed-authority acceptance probe and terminal task disposition
  are still in flight. The packet therefore remains explicitly draft.
- **Next:** refresh onto task 0048's terminal revision, re-read its final
  authority and evidence, rerun affected release profiles and repository gates,
  obtain the final adversarial challenge, and only then finalize the packet and
  close this record.

## Scope and expected touch set

- Durable output: this task record and one
  `research/investigations/proof-certificate-architecture/PACKET.md`.
- Read-only evidence: active and candidate specification, semantic entailment,
  goal/provenance/checking metadata, lowering/backend consumers, focused tests,
  compiler README, obligation-discharge evidence, and relevant MCTS-Mem nodes.
- Scratch measurements and agent notes stay under
  `/Users/bytedance/do_not_scan`; no overlapping repository reports are added.

## Dependencies and integration order

- Task 0048 may continue concurrently and owns all v0.26 language, compiler,
  protected-evidence, plan, outline, approval, and design-memory changes.
- Task 0049 may perform read-only research over its captured candidate snapshot,
  but task 0048 must become terminal first. Task 0049 then refreshes/rebases,
  replaces snapshot observations with landed evidence, reruns affected probes,
  and only afterward finalizes or integrates its packet.
- Any task-number collision is resolved by renumbering this record before
  integration. No last-writer-wins resolution is permitted for semantic or
  design-memory overlap.

## Validation

- Every central current-state claim cites active specification, landed code,
  test/output, or exact revision; candidate-only observations are labelled.
- One complete certificate proves `i < len(values)` from `i < n` and
  `n = len(values)`, with write, projected callee-write, consume, and scope-exit
  invalidations.
- The hostile inventory covers claims, joins, contradiction, ordinary and
  counted loops, recursion/mutual recursion, generic/const substitution,
  named-constant and borrow identities, facts-off/on behavior, and backend
  no-check authorization.
- All seven alternatives receive the same architecture, soundness, TCB,
  completeness, determinism, diagnostic, migration, and cost review.
- Measurements state command, scope, input revision, result, and limitation;
  unmeasured quantities are labelled `not measured`.
- An independent adversarial review challenges the final recommendation before
  closure.

## Stop condition

Stop with an exact finding if deterministic acceptance cannot be preserved, the
verifier is not materially smaller or easier to audit than the producer, the
certificate cannot express an already landed acceptance-bearing case, task
0048's representation remains unstable, the work expands into a language or
accepted-set change, or certificate infrastructure becomes disproportionate.
Retaining the unified entailment engine is a successful result.

## Closure

After task 0048 is terminal, refresh and validate the packet against its landed
revision. Close by moving this same record to `docs/done/` with the packet and
validation as canonical evidence. Research alone authorizes no implementation;
any selected implementation requires a separately approved task. The temporary
handoff is deleted once this record carries its authority and the packet carries
the complete brief.
