# Attribution inventory — OWN-6 and OP-5 after the v0.32 flips

Batch 0072, work item W4. Report only: this record adds no conformance case
and changes no citation. It states what the corpus asserts today, what an
honest citation would assert, and the smallest case set that would restore
real coverage. The lead sequences any of it.

Measured at `4d6470d4` against the active `spec/kernel-spec.md` v0.32
(`5ea3927a…6bf5`, 135 rules). Every count below is a grep over
`tests/conformance/manifest.jsonl` and `compiler/src/`, reproducible in place.
Case ids are written as they stand after the W4 renames at `c09c1526`, except
where a pre-batch-0071 row is quoted as history.

The two questions are one question. A rule's declared coverage is a `rules`
list, and a `rules` list is writer-chosen prose. When a specification delta
moves the *reported* rule at a site, the corpus follows the report and the
vacated rule silently loses its evidence while `coverage` stays 135/135 —
because some other case still names it in a list. Nothing mechanical sees
this: the adapter compares a case's actual verdict against its declared one,
and on a citation flip both sides move together.

---

## 1. OWN-6 — zero reject-citing cases over an unpinned rejection surface

### What exists today

Four manifest rows cite `OWN-6`. None is a rejection.

| id | expect | why it cites OWN-6 |
| --- | --- | --- |
| `own6-pos-callscoped-temp` | `accept` | the call-scoped-temporary clause |
| `x-child-reborrow-run` | `run` exit 0 | statement-scoped unique child reborrows, among thirteen other rules |
| `own6-pos-callresult-borrow-chain` | `run` exit 0 | bound call-result holder resolution |
| `fn1-pos-result-provenance-distinct-regions` | `run` exit 0 | one candidate per region |

Before batch 0071 there was exactly one reject-citing row:
`own6-neg-callresult-no-provenance-candidate`, declared
`{"kind":"reject","rule":"OWN-6"}`. The [FN-1] declaration-site provenance
judgment moved that rejection to the callee's `rtype`, so batch 0071 correctly
restated the row as `{"kind":"reject","rule":"FN-1"}` — and OWN-6's reject
column went to zero without a single case being deleted.

The compiler does have an OWN-6 rejection surface, and it is not small.
`SemanticIssueKind::InvalidChildReborrow` is raised under `SemanticRule::Own6`
at three distinct conditions in `compiler/src/semantic/check/borrows.rs`:

- **:737** — a written child reborrow in a **non-candidate argument position**
  of a borrow-returning call. The v0.32 reborrow extension admits a child only
  in the call's single provenance-candidate position; every other
  borrow-returning receiver keeps OWN-6's own/unit-result condition.
- **:770** — a **non-candidate argument child whose region is not a
  statement-scoped local region**. In the candidate position that condition is
  replaced by the parent's permanent suspension; everywhere else the child
  stays statement-scoped.
- **:793** — a `CallArgument` child whose **holder is neither a parameter nor a
  `let` binding**, or a `uniq` child of a **non-`uniq` parent**, or a **parent
  region that does not outlive** the child's.

All three are pinned only by library tests:
`compiler/src/semantic/tests/borrows.rs:452` and `:473` (inside
`child_reborrow_shape_and_sibling_exclusivity_follow_own6`) and `:1140`
(`extension_keeps_non_candidate_children_rejected`). No corpus case reaches
any of them. The compiler-independent corpus therefore states nothing at all
about how a Whitefoot implementation must reject an inadmissible child
reborrow — a rule about aliasing, which is the property the language exists to
guarantee.

One further fact makes the gap look intentional when it is not.
`own6-pos-callscoped-temp` carries a `reason` field asserting that OWN-6 "is
never separately cited, so only a positive that exercises the
call-scoped-temporary clause is authored." That premise is false at three
compiler sites. A reader auditing coverage meets an annotation explaining why
no negative exists, and stops.

### What an honest citation would look like

A row declaring `{"kind":"reject","rule":"OWN-6"}` whose violated obligation is
OWN-6's own child-reborrow admission, not a neighbouring rule's. The three
conditions above are the whole surface, and each must be separable: a case for
one must not be rejectable by another rule first, or [DIAG-1] hands the
citation to whichever rule is defined earlier.

### Smallest case set restoring real coverage

Three cases, one per condition. Every shape below is already verified — it is
transcribed from a green library test that asserts exactly `SemanticRule::Own6`
plus `InvalidChildReborrow`, so no compiler behavior is in question and no
existing verdict moves.

1. **`own6-neg-non-candidate-position-child`** — condition **:737**. Shape at
   `borrows.rs:1136-1145`: `mix['p2, 'q2](p: &uniq 'p2 i32, q: &'q2 i32) ->
   &'q2 i32`, with the child `p: &uniq 'a deref(hx)` in the non-candidate
   position. `reject` / `OWN-6`.
2. **`own6-neg-child-region-spans-two-statements`** — condition **:770**.
   Shape at `borrows.rs:457-478`: two `take<'child>(out: &uniq 'child
   deref(out))` calls inside one `region 'child`, so the child region is not
   statement-scoped. `reject` / `OWN-6`.
3. **`own6-neg-uniq-child-of-shared-holder`** — condition **:793**'s kind
   clause. Shape at `borrows.rs:437-455`: `&uniq 'child deref(out)` where the
   holder `out` is `&'r`. `reject` / `OWN-6`.

If only one lands, take **(1)**. It is the clause the v0.32 reborrow extension
newly carved out, and it is the clause whose neighbouring rule (FN-1) now owns
the case that used to stand in this column — so it is the one a reader is most
likely to believe is already covered.

Alongside the cases, `own6-pos-callscoped-temp`'s `reason` needs correcting:
it is the sentence that would otherwise explain the restored negatives away.

---

## 2. OP-5 — nine positive citations to zero, over sixteen cases that still exercise it

### What exists today

Exactly one case row cites `OP-5`:

| id | expect | subject |
| --- | --- | --- |
| `fn8-neg-requires-non-bool-check` | `reject` / `OP-5` | an `i32` parameter as a `requires` final's condition |

Plus one non-case annotation row (`{"rule": "DIAG-3", "covered_by":
"compiler-trap-tests"}`) that mentions OP-5 in prose.

Before batch 0071, ten case rows cited OP-5 — nine of them non-reject:

```
trap-op5-check-fails                          trap
op5-pos-check-pass                            run 0
op5-neg-check-fails                           trap
x-arith-iadd-wrap-overflow-to-negative        run 0
x-arith-check-catches-wrapped-overflow-traps  trap
x-arith-isub-wrap-min-roundtrip-runs          run 0
x-arith-iadd-checked-overflow-err-arm-runs    run 0
x-requires-output-capacity-run                run 0
x-base64-rfc-vectors-run                      run 0
```

Check dissolution retired the body `check_stmt` from [GRAM-4], so every one of
those nine correctly moved its citation to `CLM-1`: the statement they
exercised no longer exists. Nine to zero, with no case deleted and no verdict
moved.

But [OP-5] did not go away with the statement it used to name. In v0.32 OP-5
**is** the condition judgment, reached from four places in the specification:
the `if` condition (§255), a loop's false result (§1270), the [CLM-1] claim
predicate (§2683), and the final `check_stmt` of a `requires`/`ensures` block
(§885).

That surface is exercised heavily and cited nowhere. Measured over the
corpus's 490 cases:

- **42** cases carry a contract-final `check_stmt` — 35 in a `requires` block,
  7 in an `ensures` block.
- Of those 42: **26 reject**, **6 accept**, **9 run**, **1 trap**. The
  **16 non-reject** cases each pass the OP-5 condition judgment on an accepted
  program.
- **None of the 16 cites OP-5.** Two of them —
  `x-requires-output-capacity-run` and `x-base64-rfc-vectors-run` — cited it
  until batch 0071 and now cite `CLM-1` in its place, which is wrong for the
  contract final: their contract finals are not claims.

The trap case is the sharpest instance. `fn8-trap-requires-false` executes a
failing contract final, and the compiler stamps that trap site
`rule_id: "OP-5"` (`compiler/src/semantic/check/control.rs:296`). The [DIAG-3]
record the case asserts therefore literally names OP-5, while the case's
manifest row cites `["FN-8", "SCOPE-4"]`.

Compiler-side, OP-5 is emitted at exactly one site — `control.rs:284`,
`InvalidCheckCondition` — pinned by one library test
(`compiler/src/semantic/tests/requires.rs:99-102`) which itself
`include_bytes!`s the single conformance case above. The corpus and the library
test are not independent evidence; they are the same case read twice.

### What an honest citation would look like

- **Positive:** a contract-final case whose accepted condition judgment is part
  of its subject cites OP-5 beside FN-8 or FN-9 — exactly as
  `x-requires-output-capacity-run` and `x-base64-rfc-vectors-run` did before
  the rewrite. That is a `rules`-list restatement, not a new source.
- **Trap:** `fn8-trap-requires-false` cites OP-5, because the record it asserts
  is stamped OP-5.
- **Reject:** `fn8-neg-requires-non-bool-check` is already honest and stays.
  Its in-source `doc` is not: it says "[FN-8/TYPE-5]" where the manifest row
  and the compiler both say OP-5.

### Smallest case set restoring real coverage

Two new cases plus three citation restatements.

New sources — each names a clause of OP-5 the corpus reaches nowhere:

1. **`op5-neg-ensures-non-bool-check`** — the `ensures` half of the condition
   judgment. Only the `requires` half is pinned today, and the halves are not
   interchangeable: §885 gives the `ensures` final the same condition judgment
   but hands ownership to [FN-9] as a proof obligation that never executes, so
   the `requires` case cannot carry it. `reject` / `OP-5`.
2. **`op5-neg-contract-final-borrowed-bool`** — the TYPE-7 exclusivity clause
   OP-5 states explicitly: a condition that uses a borrow-mode or box/arena
   binding where its referent `Bool` would be required "is rejected citing
   TYPE-7 and OP-5 forms no candidate." The corpus pins neither half of that
   exclusivity for a contract final. Its required verdict is `reject` /
   `TYPE-7`; it belongs in this inventory because it is the one clause that
   says where OP-5 does **not** fire.

Citation restatements — no new source, no verdict change:

3. `x-requires-output-capacity-run`: add `OP-5` (its contract final is not a
   claim).
4. `x-base64-rfc-vectors-run`: add `OP-5`, same reason.
5. `fn8-trap-requires-false`: add `OP-5`, because the trap record it asserts
   carries `rule_id: "OP-5"`.

If only one thing lands, take **(5)**. It costs a single token, and it closes
the one divergence where the artifact a case asserts and the row that declares
it name different rules.

---

## 3. Collateral residue met while measuring

Not part of either question; recorded so it is not re-discovered.

- **Three corpus cases are the same program.** `scope4-pos-check-traps`,
  `clm1-trap-false-claim-aborts`, and `clm1-trap-false-claim-not-refutable`
  are each `fn main() -> own unit traps { doc …; claim <name>: False() because
  "…"; return unit; }`, differing only in id, `doc`, and cited rules
  (`["SCOPE-4","CLM-1"]`, `["CLM-1","SCOPE-4"]`, `["CLM-1","CLM-2"]`). Three
  compilations and three process launches assert one behavior. Only the third
  has a distinct stated subject — CLM-2 declining to refute a constructed
  predicate — and it asserts that subject only by its verdict being a trap
  rather than a rejection, which the second case's verdict asserts equally.
- **`scope4-pos-check-traps` is misnamed twice:** `-pos-` on a `trap` verdict,
  and "check" for what is a claim. It was outside W4's deferred list and is
  untouched.
- **`fn8-neg-requires-non-bool-check.wf`'s in-source `doc` cites
  "[FN-8/TYPE-5]"** where the rule is OP-5.
