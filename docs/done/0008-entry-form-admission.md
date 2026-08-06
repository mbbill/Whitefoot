# 0008 — Entry-form admission

Frozen coordination history. This record reports how the task was carried
out; it is not authority.

- **Status:** `DONE` — landed on main at `8e89d52` (ff), 2026-08-06
- **Authority (historical):** `ACTIVE` `docs/current-plan.md` Work item 2,
  wave 3; lead-authorized overlap with 0009 (this task lands first)

## Outcome

FN-7's real semantic admission replaced the conservative gates on its
territory: the closed kind table, the closed standard-input label table
(spelling, at-most-once, strictly increasing ordinal order, exact mode and
type, binder as ordinary declaration) with FN-7's exact per-node rejection
locations, result and admitted-effect-row per kind, the rejected source call
to a kind-declaring entry, and GRAM-11 named-argument admission over SYS-2
calls. `UnsupportedSemanticFeature::{LabelledEntryInput, KindDeclaringEntry}`
became real judgments; `SystemEffectCategory` and `SystemDeclarationUse`
remain for 0009/0010, with source judgments now ordered before capability
stops (the 0006-flagged masking fixed and pinned both ways). Two reviewed
departures: FN-7's per-node locations were repaired for both entry forms
(v0.18 fixes one location table for every FN-7 rejection; routed as a
compiler defect with regressions), and no emitter change landed (every
admitted command entry stops at the system-use boundary before lowering;
`CheckedEntryForm` carries the fact forward for 0009/0011). Nine conformance
cases flipped pending → runnable, each verified and pinned by a compiler
regression; the 0007-owned collision case was flipped by the lead at landing.

## Evidence and validation

- Landed commits: `92d59fe` (claim), `aeb6d94` (implementation), `255dd5a`
  (record), `8e89d52` (rebase/cross-link), lead flip in the closure change.
- Gates green on main; lib tests 353 → 370 (total 398), none removed or
  weakened; coverage 119/119; no `expect`/`rules`/case-source byte changed
  (verified programmatically).

## Follow-ups

- 0009 rereads `check_program`'s new stage sequence (`item_declarations` →
  `check_entry_form` → `check_system_call_arguments` →
  `check_system_surface_support` → …) and removes the `SystemEffectCategory`
  stop.
- The positive `arg_get` call case stays blocked on the v0.19 rename (task
  0018); the coupled assertion is cross-linked there.
- The command form's inadmissible-effect-row set is currently unreachable
  (needs region parameters FN-7 rejects first); implemented as written,
  reachable-failure set empty.
