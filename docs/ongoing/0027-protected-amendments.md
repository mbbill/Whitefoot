# 0027 — Protected-source amendment bundle execution

This is a temporary live coordination record, not execution authority.

- **Status:** `IN PROGRESS`
- **Authority:** `ACTIVE` `docs/current-plan.md` Work item 2 and the
  owner's 2026-08-06 19-item approval (`governance/APPROVALS.md`)
- **Owner / workspace:** executor agent / isolated worktree, lead-reviewed
- **Base revision:** `23bb7b0`

## Progress

All 19 amendments applied byte-minimally and verified per case; the diff
against the base moves only the enumerated bytes (16 case sources, 3
manifest verdicts). Adapter: Pass 342 → 356, Fail 23 → 9, Skip 14
unchanged. Both gates green by unpiped exit codes; coverage 119/119.

14 of the 19 reach their declared verdict. **5 are reported OPEN, not
repaired**, because applying the approved bytes exactly did not close them
and no approval covers what lies behind:

- `own1-pos-match-copy-payload-reuse`, `own1-neg-match-move-through-borrow`,
  `own5-neg-match-borrow-affine-payload-move` — the GRAM-10 rename lifted a
  pre-semantic mask and each source stops at `FN-7 MissingMain`. All three
  are incomplete compilation units, the same class as the 2026-08-06
  41-source completion but outside its enumerated boundary (git confirms
  none was touched by `9c2033e`/`32227fb`/`2904471`/`f43417a`).
- `own13-pos-uniq-match-payloads` — after the rename it reaches `OWN-5`
  BorrowConflict at `deref(payload)`, contradicting the case's own declared
  intent that the derived &uniq payload projection be readable while the
  parent holder stays frozen. A semantic question, not a source typo.
- `own1-pos-return-affine-contextual-move` — `return move item;` repaired
  `pass_pair`; the OWN-1 stop moved to a second bare affine return,
  `return holder;` on an `own box<i32>` in `pass_box`, which the approved
  one-token amendment does not name. Coupled to the recorded
  TYPE-7/OWN-1 gap case, which is the same `own box<i32>` return shape.

The remaining Fail set is therefore 9, not the anticipated 4: the three
OWN-6-gap cases, the TYPE-7/OWN-1 ordering case, and these 5.

Also recorded: correcting the three OP verdicts to OP-1 leaves total
coverage at 119/119 (the original rule stays in each case's `rules` list,
which the manifest validator requires the reject rule to join), but the
negative-coverage count moves 44 → 41 — no case now rejects citing OP-2,
OP-7, or OP-8. Task 0025 anticipated this when it offered "correct the
three verdicts to OP-1, or retire them as unreachable-by-construction".

## Goal

Execute the 19 enumerated amendments exactly as approved (per-case lists at
`docs/done/0024-general-borrow-parameters.md` and
`docs/done/0025-attribution-divergences.md`), with per-case verdict
verification after each; nothing beyond the enumeration. The lane's
remaining red afterwards must be exactly the three OWN-6-gap cases
(task 0028's territory).

## Validation, stop, and closure

Programmatic before/after diff showing only the enumerated bytes moved;
per-case adapter evidence; unpiped gates. Close to done with the final
tally.
