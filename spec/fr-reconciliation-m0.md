# M0: Featherweight-Rust reconciliation memo (spec §5)

Basis: K003 (Pearce, TOPLAS 2021 — soundness-proven lightweight calculus: copy/move,
mutable borrowing, reborrowing, partial moves, lifetimes; Java reference impl;
500B-program model checking; rustc fuzz found 1 confirmed bug). Status: reconciliation
argued from the verified card + FR's published calculus structure; a verbatim
rule-by-rule check against the paper text is the remaining OBLIGATION-0.

## Structural mapping

| Ours | FR mechanism | Relation | Obligation |
|---|---|---|---|
| OWN-1 copy/affine + whole-binding kill | FR type-classed copy vs move; partial moves tracked per sub-path | STRICTER (we kill whole binding) | none — restriction preserves soundness |
| OWN-2 own/&'r/&uniq 'r | FR box T / &(paths) / &mut(paths) | DIFFERENT-SOUND: FR types borrows by the SET of places they may reference; ours by one place + named region | see T-A below |
| OWN-3 lexical named regions, total outlives | FR block-scoped lifetimes l with nesting order | EQUIVALENT on lexical fragment; ours adds caller-region incomparability (fail-closed) | none |
| OWN-4 store/return outlives direction | FR write-compatibility: assigned value's lifetime >= destination slot's | EQUIVALENT (direction verified — this is the rule our critique found inverted in v0; FR agrees with the fix) | none |
| OWN-5 resolved-place exclusivity | FR read/write-prohibited side conditions computed from borrow types in env | EQUIVALENT intent; ours eager per-place, FR per-judgment | OBL-1: model-check equivalence on shared fragment |
| OWN-6 holder resolution; stmt-end temporaries | FR reborrowing via *w typing | EQUIVALENT mechanism (eager resolution vs type-level paths) | covered by OBL-1 |
| OWN-7 prefix overlap, conservative index | FR path disjointness on strict prefixes | STRICTER (we conservatively overlap non-constant indices) | none |
| OWN-8 reject-when-unsure | FR is complete for its calculus | POSTURE DIFFERENCE: we may reject FR-typable programs | acceptable by design (D1a) |
| OWN-10 borrow-storage duration | FR: borrows of block-local slots cannot outlive the block (env lifetimes) | EQUIVALENT; ours states it explicitly incl. own params vs caller regions | none |
| OWN-11 loop region confinement | FR models loops via recursion on blocks; borrow re-entry excluded by env reset | STRICTER-SOUND (region-per-iteration by construction) | none |
| OWN-12 call-argument conflicts | FR function-free core (calls not in minimal calculus) | EXTENSION beyond FR | OBL-2: prove via OWN-5 reduction (args = simultaneous borrows) |
| OWN-13 match binder modes | not in FR (no sums) | EXTENSION | OBL-3: env-merge rule when checker implements arms as branches (see T-B) |
| STOR-4 arena confinement | FR boxes + no-dangling theorem | STRICTER (no escape at all) | none |

## Two structural theorems that simplify our obligation set

**T-A (singleton provenance).** FR must type borrows with *sets* of target places because
FR permits rebinding borrow variables at control-flow merges. Our kernel has NO borrow
rebinding: `set` on a ref-mode binding writes THROUGH it (OWN-5/6); bindings bind once
(TYPE-6, OWN-1 reinit-requires-new-let). Therefore every borrow's provenance is a
singleton for its entire life, FR's path-sets degenerate to our one-place model, and
merge-point soundness is trivial. This is the load-bearing simplification of the whole
D1a "frontend-scale checker" bet — and it is a LANGUAGE choice, not a checker shortcut.

**T-B (arm isolation).** Match arms are alternative blocks with binders scoped per arm;
no borrow born in an arm can outlive the match without naming an outer region, which
OWN-10/OWN-4 already police. So the checker may verify arms from a common snapshot
(no cross-arm state) once it stops approximating arms as sequential — OBL-3 names the
implementation task, not a spec change.

## Adopted from FR wholesale
- M2 validation method: generative model checking (bounded program enumeration) +
  differential oracle (interpreter); target parity goal scaled to our budget.
- The precedent that borrow checkers harbor bugs (FR's fuzz found one in rustc):
  OWN-8's reject-when-unsure + the oracle are the mitigations.

## Obligations before §5 ratification
- OBL-0: verbatim check of this memo against the FR paper text (fetch blocked so far).
- OBL-1: model-check OWN-5/6 vs FR prohibitions on the shared fragment (M2 harness).
- OBL-2: OWN-12-as-OWN-5-reduction proof note.
- OBL-3: checker arm-isolation implementation (replaces sequential approximation).

## OBL-2 discharged: OWN-12 reduces to OWN-5

Claim: OWN-12 (call-argument conflicts) adds no new soundness content beyond OWN-5
under simultaneity. Proof sketch: let a call create argument borrows b1..bn, all live
for the call's duration (OWN-12's premise). Fix any order and check each bi under
OWN-5 against the already-live b1..bi-1: OWN-5 rejects iff there exists j<i with
overlap(resolved(bi), resolved(bj)) and (uniq(bi) or uniq(bj)). Taking the union over
all i gives exactly the pairwise condition "some overlapping pair with a uniq among
them" — which is OWN-12's rule verbatim. So OWN-12 = the closure of OWN-5 over
simultaneous creation; its soundness is inherited, and its separate rule ID exists
only for DIAG-1 rule-precision at call sites. QED (modulo OBL-0's verbatim pass).

## OBL-1 first discharge (2026-07-07): model-check run

prototype/checker/modelcheck.py (FR-inherited method: generative programs + independent
dynamic oracle, seed 20260707): 20,000 programs over the covered fragment (lets, borrows,
moves, sets, nested regions) — 14,905 accepted, ALL oracle-clean: 0 soundness violations.
Over-rejection measured for the first time: 149/5,095 rejections (2.9%) are oracle-clean —
the empirical cost of OWN-8 reject-when-unsure on this fragment (the number the
constitution audit listed as "unmeasured" for the D1a checker levers).

Caveats: fragment excludes calls/match/borrow-in-loop generation; oracle detects
dangling/use-after-move only (exclusivity is not dynamically observable single-threaded);
oracle is code-independent but same-author; loops bounded to two iterations. OBL-1 status:
partially discharged; full discharge = widen generator to calls/match + third-party oracle review.

## OBL-3 discharged (2026-07-07): arm isolation implemented

checker.py match nodes now implement T-B: each arm checked from a snapshot (isolated),
arm-locals scoped out, then a conservative join (moved-in-any-arm => moved after; union
of surviving holder-borrows). democ emits real match nodes (sequential approximation
removed). Regression pair proves the semantics both ways: uniq-borrow-in-both-arms now
ACCEPTED (was false-rejected sequentially); move-in-one-arm-then-use REJECTED [OWN-1]
via the join. Oracle upgraded to bounded all-paths exploration (up to 256 paths), so a
join bug cannot hide behind path choice. Model check rerun WITH match generation:
20,000 programs, 15,436 accepted, 0 soundness violations; over-rejection now 1.1%.
Remaining for section-5 ratification: OBL-0 only (verbatim FR paper pass).

## OBL-1 hardened (2026-07-07): calls, params, caller-horizon oracle

Generator widened to calls (borrow arguments; stresses OWN-12/through-borrow) and function
parameters with caller regions (stresses OWN-10 both arms). Oracle extended past return:
frame dies, surviving caller-region borrows touched — post-return dangles (OWN-10 target
class) now dynamically observable, so OWN-10 soundness is genuinely tested. 30,000
programs: 13,335 accepted, 0 soundness violations. Over-rejection decomposed per rule:
static-only rules (OWN-5/OWN-12 exclusivity, OWN-1 move-through-borrow) are dynamically
invisible BY DESIGN; TRUE over-rejection = 7.2%, all OWN-10 on never-escaping statement
temporaries — measured conservatism (refinement candidate: OWN-10 at store/return instead
of creation; deferred under reject-when-unsure).

## OBL-0 DISCHARGED (2026-07-07): verbatim pass against the FR paper
Source: Pea21_TOPLAS_preprint.pdf (fetched from whileydave.com; archived in
optimizer-language-research/sources/). Page-anchored confirmations:

1. **Borrow types carry lval SETS** — CONFIRMED. Type syntax `&[mut] w̄` over lval
   sequences; W-Bor subtyping `ū ⊆ w̄`; type join Def 3.8: "&x ⊔ &y gives the type
   &x,y which represents an immutable borrow to either x or y" (p.20). Sets arise
   at control-flow joins — exactly T-A's premise.
2. **Assignment containment** — CONFIRMED verbatim. T-Assign requires shape
   compatibility AND `Γ ⊢ T ≥ m` (Def 3.21 Well-Formed Type): "every borrowed
   reference contained in T lives AT LEAST AS LONG as lifetime l" (p.24) — our
   OWN-4 direction, letter-for-letter.
3. **Partial moves** — CONFIRMED (Def 3.18 Move, strike() producing ⌊T⌋ shadows,
   p.22). FR tracks the exact moved sub-path; we kill the whole binding —
   STRICTER as claimed.
4. **Progress & preservation** — CONFIRMED: "small-step operational semantics
   which lends itself naturally to proofs of progress and preservation" (p.14).
5. **Core omits functions** — CONFIRMED: method invocation listed under
   Extensions (p.3).

Bonus exact alignments found:
- **Copy classification IDENTICAL**: Def 3.6: "T has copy semantics when T = int
  or T = &w̄" — primitives and shared borrows copy, all else moves = OWN-1
  verbatim.
- **No shadowing is FR's own choice**: "FR does not support variable shadowing
  which, although not ideal, provides important simplifications" (p.14) = TYPE-6.
  (Rust allows shadowing; FR and we both cut it, same reason.)
- T-Move ¬writeProhibited / T-Copy ¬readProhibited / T-MutBorrow mut()∧¬wP /
  T-ImmBorrow ¬rP = our OWN-5 clause set, rule for rule.
- FR path conflict is COARSER than ours ("any paths involving the same variable
  conflict", p.21, with field-disjointness noted as the sanctioned extension) —
  our OWN-7 field-prefix precision is along FR's own extension direction.

**The one true permissiveness delta, now precisely located**: FR supports borrow
REASSIGNMENT with retyping (T-Assign write⁰ example 17: z retypes &mut y → &mut x,
p.25) — the exact feature whose absence T-A exploits. FR needs lval-set types and
retyping machinery to support it; we ban it (R1: serves writer flexibility only)
and the whole set/retype apparatus collapses to singletons. T-A is thereby
confirmed as sound-by-subsetting: our checker handles a strict subset of FR's
state space.

**Status: §5 formally reconciled. All obligations OBL-0..3 discharged**
(OBL-1 at fragment scope). Recommendation to owner: ratify §5.
