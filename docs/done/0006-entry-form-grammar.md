# 0006 — Entry-form grammar closure

Frozen coordination history. This record reports how the task was carried
out; it is not authority.

- **Status:** `DONE` — landed on main at `5cd1eef`, 2026-08-06
- **Authority (historical):** `ACTIVE` `docs/current-plan.md` Work item 2,
  wave 1

## Outcome

Audited the entry-form grammar layer the v0.18 activation (`9768bae`) landed
and closed its residue. Already complete and untouched: grammar tables, the
parse/finalize path over both entry shapes, FORM-2 rendering (verified — the
canonical FN-7 header round-trips byte-for-byte with no attachment-set
change), table-checked carrier classification, and parser diagnostics at the
two new LL(2) decisions. One real defect found and fixed: the [SYS-3]
admission decision ran before FN-8 requires-block admission, against DIAG-1's
fixed stage order — moved after `check_requires_blocks` with a two-sided
regression. Gap-fills were evidence: canonical-byte acceptance plus
trivia-mutation reject sweep, five malformed `input_label` shapes, complete
expected sets at both decisions, non-entry `program_kind`/`input_label`
cases. The FN-7 kind-declaring judgment is now published as
`crate::syntax::unit_program_kind(topology) -> Option<NodeId>`
(`compiler/src/syntax/entry_form.rs`) — `Some` carries the DIAG-1
`SourceNode`; tasks 0007/0008 consume it instead of rederiving the judgment.

## Evidence and validation

- Landed commits: `cb778c1` (claim), `5cd1eef` (implementation).
- Gates green on main after landing; lib tests 330 → 339; no existing test
  changed; no conformance verdict touched; native verifier 64/74/75.

## Follow-ups

- Task 0007 (live, cross-linked) rebases onto this landing and adopts the
  accessor as its admission trigger.
- Task 0008 owns: converting the conservative unsupported gates to real FN-7
  judgments, and the flagged ordering between the semantic-side
  `check_system_surface_support` gate and `check_main_header` (an
  `external`/`blocks` row can currently mask an FN-7 source error).
