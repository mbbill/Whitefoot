# 0025 — Attribution-divergence investigations

Frozen coordination history. This record reports how the task was carried
out; it is not authority.

- **Status:** `DONE` — landed on main, 2026-08-06
- **Authority (historical):** `ACTIVE` `docs/current-plan.md` Work item 3

## Outcome

All 15 divergences terminal: 4 compiler defects fixed with regressions
(lane Pass 306 → 310, zero new failures) — one parser defect (DIAG-1's
program-leftover row applied without its stated first-token precondition;
two FORM-3 cases now cite FORM-3) and two TYPE-7 implicit-read exclusivity
gaps at the match scrutinee and indexed-place root (same shape as the
propagate gap closed at 0019); 10 protected expectations returned to the
owner unedited with exact rule-text citations; 1 spec gap recorded
(TYPE-7-vs-OWN-1 ordering among simultaneously established semantic
rejections is implementation-defined; the compiler conforms, the case
over-specifies). The priority-1 question REVERSED the working hypothesis:
GRAM-10's text requires rejecting a binder spelled like its paired field in
three independent sentences, so the two "positive" programs are wrong as
written (all five GRAM-10 cases share that root), and the inventory check
stands untouched.

## Owner asks carried forward (nothing edited)

1. Five GRAM-10 sources: one-token binder rename each.
2. Three OP-family verdicts: correct to OP-1 (resolution precedes
   semantics; OP-2/7/8 unreachable by construction) or retire.
3. `x-eff-dup-reads-effect`: add region parameters so EFF-1 is reachable.
4. `own1-pos-return-affine-contextual-move`: write `return move item;` or a
   future OWN-1 return-position carve-in (a spec change).
5. The TYPE-7/OWN-1 ordering gap: future spec-batch candidate.

## Evidence and validation

- Landed commits: four on the branch, ff-merged. Both gates green by
  unpiped exit codes; a non-blocking `SELECT_ATOMS` observation recorded
  for the next table regeneration.
