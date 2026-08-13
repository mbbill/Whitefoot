# 0051 — Stage 8a bit-bound proof probe

- **Status:** `WAITING`
- **Authority:** the `ACTIVE` Current Plan selected 2026-08-12, Workstream 8a
  `Local facts` and `Caller audit`, derived from Direction Outline revision 32;
  current Direction Outline revision 33 records the same `PROOF-8` direction
  with `PROOF-1`, `VERIFY-1`, and `VERIFY-2` constraints
- **Owner / workspace:** Codex executor /
  `/Users/bytedance/do_not_scan/whitefoot-0051-stage8a-bit-bound`, branch
  `codex/0051-stage8a-bit-bound`
- **Base revision:**
  `c2c40924b5b7a4ac4fbcb54a3b88b9d025285e7d`

## Goal

Establish or falsify that exactly two bounded unsigned fact sources are enough
for the real `read_bits` normal-result path to derive `value < mask_high`:

- `result = iand(left, right)` derives `result <= left` and
  `result <= right`; and
- `high = ishl.wrap(one, count)` derives `high != 0` only when `one` is a
  checked unsigned constant whose mathematical value is one.

This is a removable production-path experiment. It selects no language rule
and must leave no compiler, program, specification, conformance, generated, or
MCTS bytes behind.

## Direction and invariants

- Use the production checked tree, term table, fact state, closure, support,
  kill, and call-goal judgment. A source-, function-, test-, or corpus-shaped
  recognizer is forbidden.
- The result must follow through the existing unsigned type range, ENT-4
  disequality strengthening, and existing S7 `mask = high -wrap 1`; no third
  fact family, arithmetic-expression term, Boolean decomposition, induction,
  fixed point, or solver is admitted.
- Each relation retains its exact normal support. A later write to
  `state.hold` must kill `value <= old_hold` while preserving
  `value <= mask`; mutation of either operand or the result kills only the
  facts that name it.
- Signed operands and operation near misses establish nothing. `Err` outcomes
  establish no result fact.
- Temporary source and test changes stay in the isolated worktree and are
  restored before integration. Persistent evidence is observational only.

## Method

1. Recompute the exact v0.27, four-source raw-DEFLATE, and wfgrep identities
   from the Current Plan; run the unmodified negative witness and
   `make -C compiler check` with `TMPDIR` under
   `/Users/bytedance/do_not_scan`.
2. Add only temporary in-crate source recognition in
   `compiler/src/semantic/entailment/flow/sources.rs` and focused harnesses in
   `compiler/src/semantic/tests/entailment.rs`.
3. On a scratch clone of the real `read_bits` body, call a current-language
   proof helper immediately before each normal `Ok(value:)` return. Without
   the two sources the instantiated call goal must be `unproved`; with both
   sources the same goal must be `discharged`. The `Err` path must publish no
   normal-result witness.
4. Cover `u8`, `u16`, `u32`, and `u64`; both `iand` operands and operand order;
   term and checked-constant operands; and counts `0`, `1`, `W-2`, `W-1`,
   `W`, and `W+1` where representable.
5. Cover signed integer types, `ior`, `ixor`, right shift, trapping shift,
   left operands `0` and `2`, a nonconstant left operand, and constant one in
   the wrong operand position. Every near miss must remain underived.
6. Exercise per-support writes, whole-root and element-write distinctions,
   scope exit, and the real `state.hold` mutation. Record the exact surviving
   path through `mask`.
7. Remove every temporary change, prove the host identities are restored, and
   rerun focused and complete gates. Prepare the complete append-only section
   in scratch with its commands, matrix, result, limitations, exact revision,
   SHA-256, and byte/line counts. Do not modify the installed acceptance
   record until the fixed combined 0051/0052 candidate receives explicit owner
   approval; any changed byte returns to the hard wait.

## Scope and expected touch set

- Temporary only: `compiler/src/semantic/entailment/flow/sources.rs`,
  `compiler/src/semantic/tests/entailment.rs`, and scratch inputs below
  `/Users/bytedance/do_not_scan`.
- Persistent after exact approval: this task record,
  `research/investigations/obligation-discharge/ACCEPTANCE.md`, and the one
  combined approval entry in `governance/APPROVALS.md` only.
- Read-only: active specification, frozen real consumers, compiler model and
  goal machinery, and the consulted proof design memory.

## Dependencies and integration order

- The original probe ran from the plan-activation premise at `c2c4092` in
  parallel with tasks 0052 and 0054. Task 0054 subsequently changed the
  canonical entailment engine, and task 0055 extends the same retained root
  path; this task must wait for 0055's terminal closure, refresh onto that exact
  revision, and rerun the complete proof, near-miss, support/kill, real-body,
  determinism, restoration, and gate matrix once.
- Tasks 0051 and 0052 share their first protected equivalent-compliance batch.
  Its fixed append order is installed base, refreshed 0051 section, then
  refreshed 0052 section, with one exact before/after audit and
  approval-ledger entry. Neither may install or close until that exact combined
  candidate receives explicit owner approval.
- Task 0053 is claimable only after 0051 and 0052 are terminal positive on that
  combined evidence revision. It may then execute in parallel with task 0056;
  on that positive path their later evidence belongs to a second independent
  protected evidence packet that must land before any Stage 8b candidate work
  begins. If this task or 0052 stops, 0053 is not claimed and 0056 closes its
  independent DIAG-2 evidence separately.

## Validation

- Baseline negative witness and positive witness use the same checked source
  except for the temporary general fact sources.
- Positive, signed, near-miss, operand-order, outcome, and support-kill cases
  all have explicit expected dispositions and deterministic repeats.
- The real `state.hold` write kills only its supported relation and the
  `value <= mask < high` route remains live.
- Before approval, restoration leaves the repository worktree clean and only
  the exact scratch section differs. After approved combined integration, the
  diff is limited to the fixed acceptance append, approval entry, and task
  lifecycle changes; all frozen source and spec digests match the Current Plan.
- `env TMPDIR=/Users/bytedance/do_not_scan/whitefoot-cargo-tmp make -C compiler check`
  and `env TMPDIR=/Users/bytedance/do_not_scan/whitefoot-cargo-tmp make check`
  pass from the persistent result tree.

## Stop condition

Stop with the smallest reproducer if the goal needs a third source family,
general arithmetic or Boolean reasoning, loop induction, a solver, a
recognizer, an unproved premise, or a relation whose support has already died;
if any signed or operation near miss gains a fact; or if restoration cannot
return every host identity. A stopped result is valid evidence and forbids
task 0053 and Stage 8b while the independent DIAG-2 work continues.

## Progress and closure

- **Completed:** the original removable probe produced positive local,
  near-miss, support/kill, and restoration results with full gates green.
- **Current:** the former scratch section and combined candidate ending in
  SHA-256 `78ce0073244e810c1acb1b094c86d58d0522800ce025fc1f197c369fb84d53d5`
  are withdrawn and must not be installed because later DIAG-2 entailment
  changes make their revision identity stale; wait for task 0055 terminal.
- **Next:** refresh once onto task 0055's terminal closure, rerun the complete
  matrix, and produce a new exact scratch section for the combined protected
  candidate.

Close by moving this record to `docs/done/` in the lead-reviewed combined
integration change after the exact owner-approved canonical acceptance bytes
and approval entry land and all temporary bytes are absent.
