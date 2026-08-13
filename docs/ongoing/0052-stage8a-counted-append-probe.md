# 0052 — Stage 8a counted append proof probe

- **Status:** `WAITING`
- **Authority:** the `ACTIVE` Current Plan selected 2026-08-12, Workstream 8a
  `Local facts` and `Caller audit`, derived from Direction Outline revision 32;
  current Direction Outline revision 33 records the same `PROOF-8` direction
  with `PROOF-1`, `VERIFY-1`, and `VERIFY-2` constraints
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
8. Remove every scratch/compiler/program change, restore all hashes, and rerun
   the gates. Prepare the complete append-only section in scratch with its
   exact commands, matrix, result, revision, SHA-256, and byte/line counts. Do
   not modify the installed acceptance record until the fixed combined
   0051/0052 candidate receives explicit owner approval; any changed byte
   returns to the hard wait.

## Scope and expected touch set

- Temporary only: scratch Whitefoot sources and harness data under
  `/Users/bytedance/do_not_scan`, focused semantic tests as needed, and local
  runtime variants of `append_slice` in the isolated worktree.
- Persistent after exact approval: this task record,
  `research/investigations/obligation-discharge/ACCEPTANCE.md`, and the one
  combined approval entry in `governance/APPROVALS.md` only.
- The real `tests/programs/wfgrep.wf` and
  `tests/programs/raw_deflate_boundary.wf` remain byte-identical.

## Dependencies and integration order

- The original probe ran from the plan-activation premise at `c2c4092` in
  parallel with tasks 0051 and 0054. Tasks 0054 and 0055 change the canonical
  entailment/root path; this task must wait for 0055's terminal closure,
  refresh onto that exact revision, and rerun every proof, hostile control,
  2,430-case differential, real-program oracle, restoration check, and gate.
- Tasks 0051 and 0052 share their first protected equivalent-compliance batch.
  Its fixed append order is installed base, refreshed 0051 section, then
  refreshed 0052 section, with one exact before/after audit and
  approval-ledger entry. Neither may install or close until that exact combined
  candidate receives explicit owner approval.
- Task 0053 is claimable only after 0051 and 0052 are terminal positive on that
  combined evidence revision. It may then execute in parallel with task 0056;
  on that positive path their later evidence belongs to a second independent
  protected evidence packet that must land before any Stage 8b candidate work
  begins. If this task or 0051 stops, 0053 is not claimed and 0056 closes its
  independent DIAG-2 evidence separately.

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
- Before approval, restoration leaves the repository worktree clean and only
  the exact scratch section differs. After approved combined integration, the
  diff is limited to the fixed acceptance append, approval entry, and task
  lifecycle changes; exact consumer and spec digests match the Current Plan.
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

- **Completed:** the corrected original probe produced both positive return
  proofs, all proof/behavior controls, 2,430/2,430 admitted differential cases,
  unchanged real-program oracles, exact restoration, and full green gates.
- **Current:** the former scratch section and combined candidate ending in
  SHA-256 `78ce0073244e810c1acb1b094c86d58d0522800ce025fc1f197c369fb84d53d5`
  are withdrawn and must not be installed because later DIAG-2 entailment
  changes make their revision identity stale; wait for task 0055 terminal.
- **Next:** refresh once onto task 0055's terminal closure, rerun the complete
  matrix, and produce a new exact scratch section for the combined protected
  candidate.

Close by moving this record to `docs/done/` in the lead-reviewed combined
integration change after the exact owner-approved canonical acceptance bytes
and approval entry land and every temporary byte is absent.
