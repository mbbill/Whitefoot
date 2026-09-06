# 0019 — Conformance rulings execution

Frozen coordination history. This record reports how the task was carried
out; it is not authority.

- **Status:** `DONE` — landed on main, 2026-08-06
- **Authority (historical):** `ACTIVE` `docs/current-plan.md` Work item 4
  and the owner's 2026-08-06 protected-surface approval

## Outcome

Both rulings executed exactly within the approved boundary: the 41
incomplete-unit sources completed with one uniform trivial entry (no variant
needed; verified 41 files / 164 insertions / 0 deletions), and the 35
overclaiming statuses corrected to pending (byte-verified: only
status/reason moved, authoring rationales preserved). Bucket 4 classified:
`gram5-pos-recursive-place-projection` is a protected-evidence mismatch (the
case source omits a required region argument; one-token amendment needs an
owner ruling), and `type7-neg-propagate-box-holder` was a compiler defect —
the propagate path lacked ERR-3's holder-without-deref TYPE-7 judgment —
fixed at landing by the lead under the plan's Work item 4 authority, with a
lib regression mirroring the return-site guard. Surprise finding: 13 of the
41 masked a second cause (9 more borrow-capability stops outside the
approved 35 enumeration — they resolve when 0021 lands — and 4 further
attribution divergences). Combined corpus truth after 0019+0020+the fix:
Pass 242 → 305, Fail 123 → 25 (9 awaiting 0021, 15 attribution divergences
under investigation, 1 owner ruling pending).

## Evidence and validation

- Landed commits: `9c2033e`/`32227fb`/`2904471`/`f43417a` (rulings), plus
  the lead's propagate-fix commit. Both gates green by unpiped exit codes;
  coverage 119/119 unmoved.

## Follow-ups

- gram5's one-token protected amendment rides the next owner ask.
- The 15 attribution divergences (0020's 11 + 0019's 4) are the named
  investigation stream for the checkpoint.
- The 9 borrow-stragglers flip with 0021.
