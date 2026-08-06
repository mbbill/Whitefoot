# 0024 — General borrow-mode parameters and let-borrows

This is a temporary live coordination record, not execution authority.

- **Status:** `IN PROGRESS`
- **Authority:** `ACTIVE` `docs/current-plan.md` Work item 2 (unsupported
  specified capability, task-0021 finding)
- **Owner / workspace:** executor agent / `worktree-agent-a1d0ac9d018482e6f`,
  lead-reviewed
- **Base revision:** `7240f84`

## Progress

- Claimed at `7240f84`; authorities read (v0.19 OWN-2/4/5/6/10/11/13, TYPE-7,
  GRAM-10, EFF-2 §9.1; task-0021 finding; `docs/WORKFLOW.md` execution loop).
- Baseline before any change: corpus adapter Pass=306 Fail=24 Skip=49. The 44
  waiting cases are the 35 manifest `pending` entries naming this capability
  plus the 9 runnable `RegionsAndBorrows` failures (`reject-own10-dangle`,
  `gram3-pos-modes`, `own4-pos-return-caller-borrow`,
  `own4-neg-return-local-borrow`, `own10-neg-dangle-caller`,
  `own13-pos-borrow-match-live`, `x-borrow-return-uniq-local-region`,
  `type7-neg-match-reference-call`, `type7-neg-return-reference-holder`).
- **Done — capability.** Borrow-mode parameters, `let` borrows, deref reads
  and writes, borrowed match, borrow-mode results, lowering, and backend for
  scalar and enum content. Sites: `check/types.rs` and `check/generics.rs`
  admit any borrowable type by one predicate pair in `check/borrows.rs`
  (`borrowable_type`, `borrow_addresses_storage`); `check_borrow` and
  `check_child_reborrow` gained the addressed-storage arm; `TYPE-7` gained a
  `reference_value` marker so a bare holder, a `borrow_expr`, and a
  reference-returning call are rejected as `match` scrutinees; `return`
  splits FN-1 (mode) from OWN-4 (region). Representation follows the existing
  struct-borrow machinery, generalized rather than duplicated:
  `CheckedExpression::BorrowStruct`/`ReborrowStruct` became
  `BorrowAddressed`/`ReborrowAddressed` carrying a type, a new
  `DerefAddressed` node distinguishes reading through a holder from passing
  the holder on, and `IrType::NominalAddress(id)` became
  `IrType::Address(IrAddressed)` with `AddressOf`/`Load`/`Store`.
- **Attribution correction.** `SET-1` hands the shared-borrow referent to
  `[OWN-5]` in its own text; `check_dereferenced_set_target` cited SET-1. Now
  OWN-5, matching the protected case `x-borrow-write-through-shared-borrow`.
  The lib test asserting the old citation was updated (rejection unchanged),
  and the test asserting `x-enum-borrow-payload-live` is unsupported now
  asserts it checks.
- **Result: 32 of the 44 reach their declared verdict; 12 do not.** Corpus
  Pass 306 → 338, Fail 24 → 27, Skip 49 → 14. Lib tests 442 → 447 (three
  semantic, two backend regressions added; none weakened). The 12 fall in
  three classes, each verified by a probe that differs from the case source
  only in the named respect:
  - **EFF-2, authored row (3):** `gram3-pos-modes`,
    `own13-pos-borrow-match-live`, `x-fn-own-arg-for-ref-param` declare
    `pure` while reading through a `&'r` parameter, which §9.1 attributes as
    `reads('r)`. Identical sources with `reads('r)` compile and run.
  - **TYPE-5, missing region argument (6):** `own6-pos-callscoped-temp`,
    `own12-pos-distinct-uniq-args`, `own12-neg-alias-uniq-args`,
    `x-borrow-uniq-shared-call-args-overlap`, `own1-neg-bare-uniq-copy`,
    `type7-neg-match-reference-call` call a region-parameterized function
    without stating its region argument, which TYPE-5 requires ("call sites
    state all type/region/const arguments explicitly"). With `<'r>` written,
    each reaches its declared verdict — verified for OWN-12 and TYPE-7.
  - **OWN-6, returned reborrow (3) — STOP, unresolved by v0.19:**
    `own4-pos-return-caller-borrow`, `own4-neg-return-local-borrow`,
    `x-borrow-return-uniq-local-region` write `&'a deref(h)` in return
    position. OWN-6 defines the child reborrow only as "the written form
    ... occurring as an argument atom of a `call` expression", says "a child
    is never bound, returned, `give`n, stored", and defers bound and
    result-carrying children; OWN-10's second clause gives a region rule for
    a place rooted at a borrow without saying where that form may be
    written. Nothing in v0.19 names the return-position case, so the
    executor did not invent a rule: the compiler's pre-existing OWN-6
    rejection now applies, which contradicts `own4-pos`'s `accept` and
    displaces `own4-neg`'s and `x-borrow-return-uniq`'s OWN-4 citation. The
    plainly specified form `return x;` (OWN-4 on the incoming borrow) is
    implemented and carries a regression both ways.
- **`own13-pos-borrow-affine-payload` is not in reach and was left alone.**
  Its authored reason stands. It stops at GRAM-10 binder freshness
  (`Data(item: item)`), the pre-semantic class task 0025 owns; the same
  source with a fresh binder and the §9.1 row compiles, so the borrowed
  match over an affine struct payload works.
- **Manifest:** the 35 flipped to `runnable` with their pre-task-0019
  authoring rationales restored; verified line by line that only `status`
  and `reason` moved on those 35 and no other row changed. Coverage unmoved
  at 119/119.
- **Rebased onto main at `3d41f9b`** (was `7240f84`), no conflicts. Task
  0025's `57a0dad` had added TYPE-7 exclusivity for a `box` holder at the
  match scrutinee and the indexed-place root, overlapping this task's
  `reference_value` rule at the scrutinee.
- **Done — one TYPE-7 mechanism.** Both assert the same rule, so they became
  one predicate, `reads_implicitly_through_holder` in `check/borrows.rs`
  beside the existing `borrow_for_destination` TYPE-7 judgment. Neither
  representation "won", because the two holder shapes are genuinely
  different here and the predicate takes both facts: a borrow-mode value
  already carries its referent's checked type, so only provenance separates
  holder from referent; a `box` binding carries the holder's type, so its
  referent is the question. What is unified is the rule — one predicate, one
  `RequiredReferent` vocabulary (`Enum` for a scrutinee, `IndexableStorage`
  for an index root), one citation and fix, asked once per position.
  `enum_referent_of_holder` and the box arm inside `match_descriptor` are
  gone; the index root's own unconditional borrow test now applies the same
  requirement, so a borrow of something no `index` could reach cites TYPE-5
  rather than TYPE-7 — matching how the box clause already behaved and how
  TYPE-7 words its exclusivity. No case moved. Both regression sets green:
  0025's `match_and_index_of_a_box_holder_are_type7_missing_dereferences`
  and this task's four borrow-holder scrutinee assertions.
- **wfgrep claim re-confirmed post-rebase.** This branch changes no byte of
  `tests/programs/` or `backend/tests/cost_shape.rs` against main's tip; task
  0023's rewritten fused scan+match program and re-derived gates run green
  under this change — all nine wfgrep oracle cases and all ten cost-shape
  gates pass.

## Goal

Implement borrow-mode parameters and let-borrows of scalar and enum types
on the normal path (v0.19 admits the modes; the compiler stops at
`check/types.rs` parse_parameters_with and `check/borrows.rs` check_borrow's
trailing arm — task 0021's reproductions), including match payloads read
through borrows with deref, per the existing buffer/struct/box machinery's
patterns. The 44 waiting conformance cases (35+9, per-case list at task
0021's record) flip runnable and must pass; wfgrep is untouched.

## Validation, stop, and closure

Per-case run evidence for every flip; no existing test weakened; any
semantics question v0.19 does not settle stops the task; unpiped gates.
Close to done with the lane delta.

Validation run (post-rebase, on `b854ec8`): `make check` green by unpiped
exit code — compiler gate 449 lib tests, 27 program tests including all nine
wfgrep oracle cases and ten cost-shape gates, corpus structure, coverage
119/119, spec append-only. `make conformance-run` Pass=342 Fail=23 Skip=14,
measured against main at `a0a3491` Pass=310 Fail=20 Skip=49 (the `3d41f9b`
difference is docs-only): 35 cases leave `skip`, 29 of them pass, 6 fail,
and 3 of main's existing failures are fixed. Per-case evidence recorded
above for all 44 and unchanged by the rebase. The "44 cases green"
closure condition is **not met**: 32 are, and the remaining 12 need owner or
lead decisions on the three classes named above — two protected-source
classes and one v0.19 semantics question this task stopped on rather than
inventing.
