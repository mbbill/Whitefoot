# 0051 — Stage 8a bit-bound proof probe

- **Status:** `IN PROGRESS`
- **Authority:** the `ACTIVE` Current Plan selected 2026-08-12, Workstream 8a
  `Local facts` and `Caller audit`, derived from Direction Outline revision 33
  item `PROOF-8` with `PROOF-1`, `VERIFY-1`, and `VERIFY-2` constraints
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
   rerun focused and complete gates. Append the commands, matrix, result,
   limitations, and exact revision to the existing obligation-discharge
   acceptance record.

## Scope and expected touch set

- Temporary only: `compiler/src/semantic/entailment/flow/sources.rs`,
  `compiler/src/semantic/tests/entailment.rs`, and scratch inputs below
  `/Users/bytedance/do_not_scan`.
- Persistent: this task record and
  `research/investigations/obligation-discharge/ACCEPTANCE.md` only.
- Read-only: active specification, frozen real consumers, compiler model and
  goal machinery, and the consulted proof design memory.

## Dependencies and integration order

- The plan activation at `c2c4092` is the sole premise. This task may execute
  in parallel with tasks 0052 and 0054.
- Tasks 0051 and 0052 both append to `ACCEPTANCE.md`; canonical integration is
  fixed as 0051, then a refreshed/rebased 0052, then task 0053.
- If task 0054 lands first, refresh onto its canonical entailment changes and
  rerun the complete matrix before integrating this result.
- Task 0053 is claimable only after both 0051 and 0052 have terminal positive
  results. Stage 8b additionally requires the complete DIAG-2 prerequisite.

## Validation

- Baseline negative witness and positive witness use the same checked source
  except for the temporary general fact sources.
- Positive, signed, near-miss, operand-order, outcome, and support-kill cases
  all have explicit expected dispositions and deterministic repeats.
- The real `state.hold` write kills only its supported relation and the
  `value <= mask < high` route remains live.
- After restoration, `git diff` contains only the acceptance record and task
  lifecycle change; all frozen source and spec digests match the Current Plan.
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

- **Completed:** plan activation and exact task registration.
- **Current:** create the isolated worktree, refresh it through this
  registration commit, and reproduce the unmodified negative witness.
- **Next:** run the two-source matrix and restore all temporary changes.

Close by moving this record to `docs/done/` in the lead-reviewed integration
change after the canonical acceptance evidence is complete and all temporary
bytes are absent.
