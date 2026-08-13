# 0052 — Stage 8a counted append proof probe

- **Status:** `IN PROGRESS`
- **Authority:** the `ACTIVE` Current Plan selected 2026-08-12, Workstream 8a
  `Local facts` and `Caller audit`, derived from Direction Outline revision 33
  item `PROOF-8` with `PROOF-1`, `VERIFY-1`, and `VERIFY-2` constraints
- **Owner / workspace:** Codex executor /
  `/Users/bytedance/do_not_scan/whitefoot-0052-stage8a-counted-append`, branch
  `codex/0052-stage8a-counted-append`
- **Base revision:**
  `c2c40924b5b7a4ac4fbcb54a3b88b9d025285e7d`

## Goal

Establish or falsify that the existing counted range can express a truthful,
behavior-equivalent `append_slice` on the exact admitted domain
`filled <= capacity`, while every normal return derives
`result <= capacity` without induction, a post-loop binder equality, or a
variable-subtraction fact.

This is a removable production-path experiment. It leaves no source-language,
compiler, real-program, specification, conformance, generated, or MCTS change.

## Direction and invariants

- The scratch helper uses the Current Plan's exact current-language
  requirement and the existing form `for @append at in filled..capacity`.
- The early return uses S11's body fact `at < capacity`; exhausted execution
  returns `capacity` directly. The counted binder never escapes its scope.
- `taken = at -wrap filled` creates no fact. The false `done` edge alone must
  prove the text index; S11 alone must prove the destination index.
- Equivalence is claimed only for `filled <= capacity`. The current body's
  exact `capacity=3, filled=4, len(text)=0` counterexample remains recorded and
  is not normalized away.
- Every output byte, return value, error, cleanup edge, effect, and required
  runtime check of the admitted cases is preserved.

## Method

1. Recompute all frozen identities and run `make -C compiler check` with a
   `TMPDIR` below `/Users/bytedance/do_not_scan`.
2. Reproduce the unmodified ordinary-loop postcondition goal as `unproved`
   and the exact invalid-domain counterexample as result `4` with an unchanged
   three-byte destination.
3. Build a scratch current-language counted variant with the exact requirement:

   ```whitefoot
   requires {
     let capacity = len(deref(destination));
     let admitted = ile(filled, capacity);
     check admitted else trap "append filled exceeds destination";
   }
   ```

   Its body computes `taken`, returns `at` on the true `done` edge, otherwise
   reads `text[taken]`, writes `destination[at]`, and returns `capacity` after
   exhaustion.
4. Use current-language proof-helper calls to show the early return and
   exhaustion return each discharge the same instantiated result bound.
5. Execute one differential harness over capacity and text length `0..=8`,
   every admitted `filled`, destination fills `0x00` and `0xA5`, and all-zero,
   all-maximum, and ascending text. Compare result and every destination byte
   in all 2,430 cases.
6. Run proof and behavioral controls, preserving the distinction between
   them:
   - removing the requirement still correctly proves the local result bound,
     but the invalid-domain witness must expose that the counted body returns
     `capacity` rather than the current body's `filled`; this is a behavioral
     non-equivalence control, not a failed proof;
   - returning `at +wrap 1` still correctly proves `result <= capacity` from
     S11's `at < capacity`, but must fail the differential behavior oracle;
   - returning `at +wrap 2` must fail the result-bound proof and behavior
     oracle;
   - restoring the ordinary loop, returning an actually independent
     parameter, and attempting to consume a post-loop binder fact must not
     produce the desired proof.
7. Replay the complete wfgrep `9/9` and raw-DEFLATE `3/3` program oracles on
   the unchanged real sources.
8. Remove every scratch/compiler/program change, restore all hashes, rerun the
   gates, and append the exact commands, matrix, and result to the existing
   obligation-discharge acceptance record.

## Scope and expected touch set

- Temporary only: scratch Whitefoot sources and harness data under
  `/Users/bytedance/do_not_scan`, focused semantic tests as needed, and local
  runtime variants of `append_slice` in the isolated worktree.
- Persistent: this task record and
  `research/investigations/obligation-discharge/ACCEPTANCE.md` only.
- The real `tests/programs/wfgrep.wf` and
  `tests/programs/raw_deflate_boundary.wf` remain byte-identical.

## Dependencies and integration order

- The plan activation at `c2c4092` is the sole premise. This task may execute
  in parallel with tasks 0051 and 0054.
- Because 0051 and 0052 share the acceptance record, 0052 refreshes/rebases
  after 0051 and integrates second. Task 0053 follows both positive results.
- If DIAG-2 changes the entailment engine before this task closes, refresh and
  rerun every proof and hostile case before integration.

## Validation

- The invalid-domain witness is reproduced before the candidate; no result is
  generalized beyond the admitted domain.
- Both candidate return shapes discharge independently; no post-loop binder
  fact or subtraction relation is observed.
- All 2,430 admitted differential cases match result and every byte.
- The two mathematically valid bounded variants (`no requirement` and
  `at +wrap 1`) retain their proof but fail the exact behavioral oracle; the
  genuinely out-of-bound and unrelated-value variants remain unproved. No
  control gains a fallback check or hidden premise.
- Unchanged real programs pass wfgrep `9/9` and raw-DEFLATE `3/3`.
- After restoration, only the acceptance record and task lifecycle differ;
  exact consumer and spec digests match the Current Plan.
- `env TMPDIR=/Users/bytedance/do_not_scan/whitefoot-cargo-tmp make -C compiler check`
  and `env TMPDIR=/Users/bytedance/do_not_scan/whitefoot-cargo-tmp make check`
  pass.

## Stop condition

Stop on any admitted result or byte mismatch; any return goal that remains
unproved; any need for induction, a loop fixed point, a post-loop binder
equality, a variable-subtraction fact, new syntax, a hidden premise, or
invalid-domain behavior change; or any real-program oracle drift. A stopped
result forbids task 0053 and Stage 8b while DIAG-2 continues independently.

## Lead correction after the first controls

The executor stopped correctly when the original task text required the two
mathematically valid variants `no requirement` and `at +wrap 1` to be
unproved. The lead corrected only the falsifier classification above: both
variants are required to keep the valid bound proof and fail the separate
behavior oracle, while `at +wrap 2` is the true proof-negative arithmetic
control. This correction does not change the Current Plan's fixed candidate,
admitted domain, behavioral oracle, acceptance boundary, or stop conditions.

## Progress and closure

- **Completed:** plan activation, exact task registration, frozen-identity and
  compiler pre-gates, the ordinary-loop `unproved` witness, the exact
  `capacity=3, filled=4` behavioral counterexample, both positive return
  proofs, and the executor's honest stop on the two misclassified controls.
- **Current:** refresh the isolated worktree onto the lead's falsifier-only
  task correction and rerun the corrected proof/behavior controls.
- **Next:** run the 2,430-case differential matrix, real-program oracles,
  restoration audit, and complete gates.

Close by moving this record to `docs/done/` in the lead-reviewed integration
change after durable evidence is complete and every temporary byte is absent.
