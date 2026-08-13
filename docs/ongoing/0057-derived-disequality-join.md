# 0057 — derived disequality closure and join repair

- **Status:** `IN PROGRESS`
- **Authority:** the `ACTIVE` Current Plan selected 2026-08-12, `Trust
  prerequisite — bounded existing-DIAG-2 repair`, plus the workflow's bounded
  compiler-defect routing; derived from Direction Outline revision 33 item
  `PROOF-8` and active v0.27 ENT-4/ENT-5
- **Owner / workspace:** Codex executor /
  `/Users/bytedance/do_not_scan/whitefoot-0057-derived-disequality-join`, branch
  `codex/0057-derived-disequality-join`
- **Base revision:**
  `c3d56e81e23373782dea16f941029769a3981273`

## Goal

Repair one existing discrepancy between the canonical entailment engine and
active ENT-4/ENT-5. A strict bound in either orientation must make the two
terms disequal throughout the least closure, that derived disequality must be
available to ENT-4 strengthening, and a join must retain the disequality when
every non-contradictory predecessor derives it even when their strict
orientations or grounds differ.

This is the smallest semantic prerequisite for task 0054's exact join parents.
It implements already-active v0.27 behavior. It changes no specification,
runtime operation, lowering rule, protected conformance artifact, or proof
ledger representation.

## Reproduction and fixed implementation decision

- In the minimal witness, one branch establishes `left < right`, the other
  establishes `right < left`, and the continuation calls a function requiring
  `left != right`. Active ENT-4 derives the same disequality on both branches;
  ENT-5 therefore requires the join to retain it. Frozen revision `20c0e55`
  instead reports the concrete call goal as `Unproved`.
- The cause is local to `state.rs`: `ClosedState::derives` answers a strict
  disequality query from bounds, but `close` retains only explicitly
  established entries in `ClosedState::distinct`, and `join` intersects that
  incomplete set.
- Keep one closure algorithm. Start its working disequality set from the live
  explicit facts. During the existing fixed point, insert the normalized pair
  whenever either closed directional bound is strict (`<= -1`), and use that
  complete working set for rule (2) strengthening of any held weak zero bound.
  Continue until neither bounds nor disequalities change. Publish that complete
  set in `ClosedState`; the existing join intersection then retains exactly
  the disequalities held by every non-contradictory predecessor.
- Do not preserve an orientation-specific strict bound unless that ordered
  bound is independently held by all inputs. The join may materialize the
  common disequality alone. Existing kill support remains the two terms, so a
  later overlapping write, consume, or scope exit removes the joined fact in
  the normal path.
- Reuse current ordinary collections and deterministic semantic answers. Do
  not add another closure, relation family, solver, flow graph, certificate,
  or special case for the reproducer.

## Scope and expected touch set

- `compiler/src/semantic/entailment/state.rs`
- focused ordinary regressions in
  `compiler/src/semantic/tests/entailment.rs`
- this task lifecycle record only

If the repair requires another production file, a new representation family,
or any specification/protected-evidence byte, stop for lead review.

## Dependencies and integration order

- The exact starting revision is `c3d56e8`; the defect itself is present at
  task 0054's frozen `20c0e55` base and is independent of Stage 8a.
- Task 0054 is waiting with an uncommitted six-file derivation-ledger change.
  Land 0057 first. Then 0054 refreshes/rebases and extends the repaired closure
  and join with exact parents; no last-writer-wins resolution is allowed.
- Tasks 0055 and 0056 remain downstream of 0054. Stage 8b remains downstream
  of both the DIAG-2 chain and Stage 8a terminal evidence.

## Validation

- First retain the smallest failing ordinary regression: opposite strict
  orientations on two reachable inputs must discharge only the common
  disequality after the join.
- Cover same-orientation strict inputs, explicit disequality on both inputs,
  and mixed explicit/strict grounds. Cover at least three predecessors with
  mixed grounds and a contradictory-neutral predecessor.
- Show the joined state does not invent either directional strict bound:
  post-join `ine(left, right)` is derivable while `ilt(left, right)` and
  `ilt(right, left)` remain unproved when orientations differ.
- Show a predecessor with only equality or no relation prevents the
  disequality from surviving, and a write that kills one branch's support
  before the join prevents survival.
- Exercise the closure interaction directly: a strict bound supplies the
  disequality used to strengthen an available reverse weak bound, including
  the resulting contradiction when both strict directions follow. Preserve
  empty/all-contradictory and ordinary-loop non-induction behavior.
- Run the exact focused test at least twice, then
  `env TMPDIR=/Users/bytedance/do_not_scan/whitefoot-cargo-tmp make -C compiler check`
  before and after the production edit and
  `env TMPDIR=/Users/bytedance/do_not_scan/whitefoot-cargo-tmp make check`
  before integration.

## Done

The active ENT-4 least closure contains every disequality implied by a strict
bound and permits it to participate in strengthening; ENT-5 joins retain a
common derived disequality across different grounds without inventing a common
ordered bound; hostile kill/contradiction/join tests and complete gates pass;
the touch set stays closed.

## Stop condition

Stop with the smallest reproduction if the active specification is ambiguous;
if the repair needs a second closure, general solver, new fact family, flow or
AST redesign, proof-ledger machinery, or source/test identity recognition; if
it changes runtime/lowering/diagnostics beyond the exact newly correct
ENT-4/ENT-5 dispositions; or if any specification, protected conformance,
canonical gate wiring, or real-program source would need to change.

## Progress and closure

- **Completed:** task 0054 isolated the exact failing join, active-spec lines,
  and frozen implementation cause without attempting an out-of-scope repair.
- **Current:** run the clean pre-gate and add the minimal ordinary regression
  in this isolated worktree.
- **Next:** implement the one-fixed-point disequality completion, run hostile
  tests and complete gates, and submit the two-file compiler change for lead
  review.

Close only through lead review by moving this record to `docs/done/` with the
landed commit, exact regression results, and gate totals.
