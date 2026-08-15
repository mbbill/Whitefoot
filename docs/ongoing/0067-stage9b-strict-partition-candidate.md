# 0067 — Stage 9b strict no-claim candidate

- **Status:** `IN PROGRESS`
- **Owner:** lead agent `/root`
- **Workspace / branch:** `/Users/bytedance/code/Whitefoot`, held branch
  `codex/0067`
- **Base revision:**
  `61b91f095b7b4942b5fa570eb62929a603a93b7c`
- **Authority:** the `ACTIVE` Current Plan, Workstream 9b `opt-in strict
  no-claim partition`, under Direction Outline `PROOF-8`

## Goal and direction

Freeze one exact, non-authoritative v0.29 specification and protected-evidence
candidate for the smallest opt-in strict partition. A declaration prefixed by
`deny_claims` is a strict root. Its finite outgoing concrete-call/SCC closure
must contain no checked claim, and every protected obligation, ordinary call
requirement, and current program-start requirement in that closure must be
discharged in the existing unasserted proof view. The closure never flows
upward into unrelated callers and creates no second runtime body, lowering,
proof engine, fact source, or serialized authority.

This task prepares exact bytes and the owner packet only. Per the current
handoff, it does not implement or activate Stage 9b before the owner approves
the frozen specification and protected corpus. It therefore records the native
grammar verifier's honest active-v0.28 frontend mismatch rather than weakening
the verifier or silently installing candidate frontend behavior.

## Scope and invariants

- Add one optional fixed `deny_claims` prefix directly to `fn_decl` and one
  numbered rule, `CLM-3`; add no production, AST attribute family, effect-row
  bit, runtime check, ABI surface, or future foreign adapter.
- Define claim identity, outgoing reachability, SCC-atomic `DirectClaims` and
  `MayClaims`, stable direct/import diagnostics, existing-U judgments,
  program-start pre-check judgment, failure-atomic publication, and exact
  DIAG-2 retention without consulting the Stage 9a observational
  `ClaimLedger` as acceptance authority.
- Preserve all ordinary v0.28 judgments and diagnostics first. Strict local
  bounds failures remain `OP-4`; strict call and marked-entry requirement
  failures remain `FN-8`; only a direct or imported claim is `CLM-3`.
- Preserve every runtime claim/check, effect, error, cleanup, body, lowering,
  and facts-on/off result. A function may serve ordinary and strict callers
  through the same checked body. No marker means byte-for-byte ordinary
  behavior and acceptance.
- Prepare exactly nine additive runnable conformance cases: two positive and
  seven negative, covering direct unreachable claims, concrete generics,
  upward closure near miss, mutual SCC import, generated entry, local OP-4,
  local FN-8, transitive strict failure, and the value-branch repair.
- Mark only the authentic wfgrep `report_failure` function as the real strict
  root and preserve its frozen output, error, cleanup, status, runtime-check,
  and facts-off oracles. Claims in callers outside its outgoing closure remain
  ordinary.
- The outgoing `spec/kernel-spec-v0.28.md` archive is an exact copy of the
  installed v0.28 bytes and is never edited after activation. Candidate bytes,
  protected rows, and the archive remain held and non-authoritative until
  exact owner approval.
- Do not edit `governance/APPROVALS.md`, the `ACTIVE-SPEC` chain, compiler or
  generated frontend bytes, canonical runner logic/pin, adapter, gate targets,
  collection/invocation wiring, lowering, backend, runtime, or active docs in
  this preapproval task.

## Method and expected touch set

1. Consult the live obligation-discharge MCTS node and rejected alternative;
   freeze the finite outgoing-closure design and exact diagnostic ownership.
2. Draft `spec/kernel-spec.md` as candidate v0.29, add the byte-identical
   `spec/kernel-spec-v0.28.md`, and update only
   `spec/derivation/derivation-ledger.md` for 133 rules and 83/50 provenance.
3. Add exactly nine new files under `tests/conformance/cases/` and append nine
   exact rows to `tests/conformance/manifest.jsonl`; modify no existing case,
   row, annotation, verdict, status, rule citation, runner, or adapter.
4. Add the marker to the one existing `report_failure` declaration in
   `tests/programs/wfgrep.wf`; change no body byte or caller.
5. Freeze full-file and suffix SHA-256 identities, exact before/after counts,
   accepted-set and identifier census, impact inventory, verifier output,
   source-location/payload expectations, and the proposed archive and
   activation-chain action. Obtain independent read-only review.
6. Set this record to `WAITING` and stop for owner approval. Any changed byte
   or scope restarts the exact review and approval packet.

Expected candidate paths are limited to:

- `spec/kernel-spec.md`
- `spec/kernel-spec-v0.28.md`
- `spec/derivation/derivation-ledger.md`
- `tests/conformance/manifest.jsonl`
- nine new `tests/conformance/cases/clm3-*.wf` files
- `tests/programs/wfgrep.wf`
- this coordination record

## Dependencies and integration order

Stage 9a is terminal at implementation commit
`e04d3acad80e1260c4f1aee24d8f45cba5140d84` and closure commit
`61b91f095b7b4942b5fa570eb62929a603a93b7c`. This candidate branches only from
that closure. It must not enter `main` independently. After exact approval, a
separate reviewed activation/implementation sequence will synchronize the
approved spec, archive, frontend, semantic implementation, generated data,
real source, protected corpus, identities, approval chain, documentation, and
task lifecycle atomically; a changed candidate returns here first.

## Validation and done criteria

- Installed v0.28 spec/archive/manifest/case/annotation identities and counts
  are captured before editing; candidate spec, archive, ledger, manifest,
  suffix, real source, and all nine case SHA-256 values are captured after.
- Grammar arithmetic is exactly 73 productions, 91 decisions, 90 fixed
  terminals, and 98 terminal predicates; rule coverage is projected from
  132/132 to 133/133 without changing the existing 132 rows.
- Protected accounting is exactly 437 to 446 cases, 30 annotations unchanged,
  424 to 433 runnable plus 13 pending, and projected adapter
  `Pass=432 Fail=1 Skip=13` with only the unchanged OWN-3 boundary.
- The production grammar verifier is run against the outgoing archive and its
  exact structural mismatch is reported as the expected consequence of the
  owner-directed no-preapproval-implementation boundary. It may not be called
  green. Archive-to-installed v0.28 remains green.
- Each proposed source with only the unapproved marker removed resolves and
  checks under installed v0.28; exact Stage 9b verdict, rule, node, coordinate,
  and payload expectations are frozen for later ordinary compiler tests.
- `git diff --check`, manifest/schema parsing, exact path-set audit, MCTS lint,
  identifier census, independent semantic/spec/protected review, and candidate
  hash recomputation all pass.
- Completion for this preapproval phase is `WAITING`, not activation: one
  exact owner-facing explanation names every byte, impact, limitation, red
  verifier boundary, and later atomic action, then stops in the same turn.

## Stop condition

Stop without weakening the candidate if the partition requires a second fact
flow, solver, negative fixed point, effect/proof framework, copied DAG,
portable proof identity, Stage 9a ledger authority, body specialization,
alternate lowering, future adapter invention, upward closure, same-SCC
non-atomic summary, assertion-backed wrapper self-authorization, ordinary-code
acceptance/runtime drift, facts-on/off drift, or any protected/gate change
outside the exact owner packet. A verifier-green packet that would require
preapproval implementation also stops and is reported honestly rather than
crossing the owner's stated boundary.
