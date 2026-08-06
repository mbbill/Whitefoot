# 0019 — Conformance rulings execution

This is a temporary live coordination record, not execution authority. Move
this same numbered record to `docs/done/` at terminal disposition.

- **Status:** `IN PROGRESS`
- **Authority:** `ACTIVE` `docs/current-plan.md` Work item 4 and the owner's
  2026-08-06 protected-surface approval (`governance/APPROVALS.md`)
- **Owner / workspace:** executor agent / worktree
  `.claude/worktrees/agent-a873aed21e5e3e431`, branch
  `worktree-agent-a873aed21e5e3e431`, lead-reviewed
- **Base revision:** `d84643b`

## Goal

Execute the two approved rulings and classify the two divergences: append
the minimal admissible entry to the 41 incomplete-unit case sources (every
other byte unchanged; each case then reproduces its declared verdict);
correct the 35 overclaiming `runnable` statuses to `pending` with reasons
naming the borrow-parameter capability (task 0021); investigate
`gram5-pos-recursive-place-projection` (expects run 0, gets TYPE-5) and
`type7-neg-propagate-box-holder` (expects TYPE-7, gets ERR-3) to a
classification per the blocker routing — report, do not fix here.

## Progress

Both rulings are executed and both divergences are classified. The lane is
still red and stays `#[ignore]`d, with its reason rewritten to the remaining
truth.

- **Baseline reproduced.** `Pass=242 Fail=123 Skip=14`, identical to task
  0014, and the four buckets partition the 123 exactly: 45 uncited
  rejections, 41 `MissingMain`, 35 `RegionsAndBorrows`, 2 divergences.
- **Ruling 1 (41 sources).** `fn main() -> own unit pure { return unit; }`
  appended to each, preceded by one blank line — the form
  `fn8-neg-requires-non-bool-check.wf` already uses. No case needed a
  variant: none of the 41 mentions `main` and every one ended in a single
  newline. The diff is 41 files, 164 insertions, 0 deletions, every added
  line one of those four, 41 times each. 28 of the 41 now reach their
  declared verdict; the other 13 are findings below.
- **Ruling 2 (35 statuses).** `runnable` → `pending`, one reason naming the
  borrow capability task 0021 implements. The 4 cases already carrying an
  authoring rationale keep it and have the cause appended, so no protected
  text is deleted. Verified line by line against `HEAD`: 380 lines byte
  identical, 35 changed, and on each of those every key but `status` and
  `reason` is unchanged, with field order preserved.
- **Lane after both rulings.** `Pass=270 Fail=60 Skip=49` — `Skip` +35 and
  `Pass` +28 exactly, no other case moved.

## Findings (reported, not edited)

- **13 of the 41 still diverge**, all because FN-7 was masking a second
  cause. 9 stop as unsupported on the same borrow capability as ruling 2 but
  are outside the enumeration the owner approved; 4 cite a different rule
  than declared (`type7-neg-match-box-holder`, `type7-neg-index-box-holder`
  → TYPE-5; `type7-neg-return-box-as-referent`,
  `own1-pos-return-affine-contextual-move` → OWN-1).
- **`gram5-pos-recursive-place-projection` — protected-evidence mismatch.**
  Line 18 calls the region-generic `read_projection` with no explicit region
  argument. FN-2 requires instantiation arguments to be explicit, and
  `type5-neg-wrong-region-arg-count` records that a call-site region-argument
  count mismatch is TYPE-5, so the compiler's rejection is correct and the
  `run 0` expectation is unsatisfiable. Adding `<'projection>` compiles and
  runs to exit 0. The source is outside the approved 41, so it is untouched.
- **`type7-neg-propagate-box-holder` — compiler defect.** ERR-3 states that a
  box holder used without `deref` retains its TYPE-7 judgment, so the case's
  declared TYPE-7 is what the specification requires. The compiler cites
  ERR-3 instead: `check_propagate_let` falls through to `invalid_propagation`
  when the operand type is not a `Result` nominal, with no holder check
  first. `propagate deref(holder)` clears that stop, which isolates the
  missing `deref` as the whole cause. The equivalent guard already exists at
  the return site. The three box-holder cases unmasked by ruling 1 are the
  same attribution gap. Acceptance is correct in every instance; only the
  cited rule is wrong. Not fixed here — no regression added, no rule moved.

## Validation, stop, and closure

Per-case verdict evidence after each source completion; the corpus lane
tally must improve exactly by the predicted buckets; any case whose verdict
still diverges after completion is a finding, not an edit target. Unpiped
gates. Close to done with the lane tally before/after.
