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

## Addendum — executor's per-case record (preserved at landing)

## Progress

All 15 investigated to a terminal classification. Corpus lane on this branch:
Pass 306 → 310, Fail 24 → 20 (the 4 defects below; set-diffed, zero new
failures). The other 9 baseline failures are 0024's borrow stragglers and are
untouched. Both gates green by unpiped exit codes.

### Disposition

| case | classification | action | evidence |
|---|---|---|---|
| `form3-neg-typeid-fn-name` | compiler defect | fixed | DIAG-1 row 5 fired without its "matches no consuming `item` row" precondition; `fn` does match one. Now FORM-3 at `Main` |
| `x-form-form3-enum-name-ident` | compiler defect | fixed | same row-5 defect; now FORM-3 at `sign` |
| `type7-neg-match-box-holder` | compiler defect | fixed | TYPE-7 implicit-read exclusivity; scrutinee cited TYPE-5 |
| `type7-neg-index-box-holder` | compiler defect | fixed | same exclusivity at an OP-2 operand (spec line 368); cited TYPE-5 |
| `own13-pos-uniq-match-payloads` | wrong expectation | owner-return | writes `Data(value: value)`; GRAM-10 forbids binder == paired field |
| `own1-pos-match-copy-payload-reuse` | wrong expectation | owner-return | writes `Wrapped(state: state)`; same |
| `own1-neg-match-move-through-borrow` | wrong expectation | owner-return | `Data(value: value)` masks the intended OWN-1 |
| `own5-neg-match-borrow-affine-payload-move` | wrong expectation | owner-return | `Data(item: item)` masks the intended OWN-5 |
| `x-enum-option-context-free-constructor` | wrong expectation | owner-return | `Some(value: value)` masks the intended TYPE-5 |
| `op2-neg-div-wrap` | wrong expectation | owner-return | `idiv.wrap` is a well-formed OPNAME resolving to no family; DIAG-1 selects OP-1 |
| `op7-neg-missing-prefix` | wrong expectation | owner-return | `add.wrap`; same chain |
| `op8-neg-rotate-trap` | wrong expectation | owner-return | `irotl.trap`; same chain |
| `x-eff-dup-reads-effect` | wrong expectation | owner-return | source declares no region params, so `'r` is an unresolved REGIONID use → OWN-3 before EFF-1 |
| `own1-pos-return-affine-contextual-move` | wrong expectation | owner-return | OWN-1's consumption list is closed and excludes return position |
| `type7-neg-return-box-as-referent` | spec gap | recorded, no change | TYPE-7 and OWN-1 are simultaneously established; DIAG-1 leaves the order implementation-defined |

Lane tally: 4 compiler defects fixed with regressions, 10 protected
expectations returned to the owner unedited, 1 spec gap recorded.

### The two rejected positive programs (priority 1)

GRAM-10's text requires the rejection, so this is **not** an over-rejection.
Three independent sentences say so — the binder "is a fresh IDENT chosen by
the writer and distinct from the field name" (rule text); a binder becomes
visible "only after GRAM-10 has established that it differs from its paired
field label" (scoping); and "a second `fieldbind` IDENT equal to its paired
field label ... is rejected citing GRAM-10 at that later/offending binder".
DIAG-1 ranks it 3 in declaration inventory, and inventory outranks resolution
and every semantic rule, which is why it also masks the intended rule in the
three negative GRAM-10 cases. The compiler's payload matches DIAG-1's
specified GRAM-10 shape exactly. Both case sources write `field: field`.

### Owner ask

Ten protected sources/expectations need an owner ruling; no byte was changed.

1. **Five GRAM-10 cases** — amend each case source to spell the binder
   differently from its field (e.g. `Data(value: payload)`). Two are positive
   programs that then run; three then reach their intended OWN-1 / OWN-5 /
   TYPE-5. This is the same one-token-amendment shape as 0019's
   `gram5-pos-recursive-place-projection`.
2. **Three OP-family cases** — the expectations name OP-2 / OP-7 / OP-8, but
   all three spellings are well-formed OPNAME tokens that resolve to no
   operation family, and DIAG-1's lexical-use role table maps an `OPNAME
   callee` with an empty admissible visible subset to **OP-1**. Resolution
   precedes semantics, so OP-2/OP-7/OP-8 are never reached. Either correct the
   three verdicts to OP-1, or retire them as unreachable-by-construction.
3. **`x-eff-dup-reads-effect`** — the source writes `reads('r), reads('s)` on
   a function with no `region_params`, so `'r` is an unresolved REGIONID use
   (DIAG-1 role table → OWN-3) before EFF-1's duplicate-category judgment can
   run. Adding the region parameters would unmask the intended EFF-1.
4. **`own1-pos-return-affine-contextual-move`** — expects Accept, but OWN-1
   consumes an affine place exactly once by `move p`, an own-place match
   scrutinee (OWN-13), or a `propagate` operand (ERR-3), and "every other bare
   `place` expression of affine type is a hard error". Return position is not
   in that closed list, so `return item;` on an `own Pair` is an OWN-1
   rejection. Either the case must write `return move item;` and stay
   positive, or OWN-1 needs a return-position carve-in (a spec change, not
   ours to make).

### Recorded gap (no self-authorized change)

`type7-neg-return-box-as-referent` (`return holder;` on `own box<i32>` with
`rtype own i32`) establishes TYPE-7 and OWN-1 at once. FN-1 states the TYPE-7
implicit-read exclusivity against *itself* ("FN-1 forms no candidate") but
nothing orders TYPE-7 against OWN-1; ERR-3 lists "their TYPE-7, OWN-1, and
OWN-11 judgments" side by side without ranking them; and DIAG-1 says the order
among simultaneously established post-resolution semantic rejections is
implementation-defined. The compiler citing OWN-1 is therefore conforming and
the case is over-specified. Closing this needs a spec decision (extend the
exclusivity to OWN-1, or relax the expectation), not a compiler change.

### Observation (not fixed; no observable effect)

The generated `SELECT_ATOMS` carry `transparent_name` on the `item` choice
decision's name positions but not on the `program`-`item*` decision's, though
both reference the same grammar-node provenance. With the row-5 precondition
in place the frontier descends into `item` and reaches the same FORM-3 with
the same coordinate and expected set, so the diagnostic is already correct;
the data asymmetry is noted for whoever next regenerates the grammar tables.

## Goal

Investigate each of the 15 recorded rule-attribution divergences (0020's
11: three OP-family, two FORM-3-vs-GRAM-2, one EFF-1-vs-OWN-3, five GRAM-10
including two rejected positive programs; 0019's 4: two TYPE-5, two OWN-1)
to a terminal classification per the blocker routing: a compiler defect is
fixed with the smallest regression; a wrong protected expectation is
returned to the owner with reproduction; a spec ambiguity is recorded as a
gap with no self-authorized change. The two rejected positive programs are
first priority (possible over-rejection of valid source).

## Validation, stop, and closure

Every fix carries a regression; no expectation edited without an owner
ruling; unpiped gates. Close to done with the per-case disposition table
and the final lane tally.
