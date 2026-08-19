# Strict-in-U clause retirement — v0.33 candidate delta

Status: DELTA TEXT FOR LEAD INTEGRATION (batch 0072, W3, executor G2). This
file is delta input to one v0.33 candidate; nothing here changes
`spec/kernel-spec.md`, and every byte lands only through the owner's
exact-byte approval. Basis revision: ACTIVE v0.32 at `spec/kernel-spec.md`,
SHA-256 `5ea3927aef20d08e1c9c80a50242628f2c469974261b68c696ee2db3934e6bf5`
(activation commit `f8c81dfc`).

Scope: retire the three [CLM-3] strict *obligation* rejection clauses — the
strict [OP-4] bounds clause, the strict [OP-2] overflow clause, and the
strict [OP-2] division clause — because no source program can reach them.
Nothing else about `deny_claims` changes. The unasserted state U itself is
**not** retired: it keeps its [PRV-1]/[PRV-2]/[PRV-3] demand-partition role
and its [CLM-3] call-requirement role unchanged.

## 1. The claim, stated exactly

Under v0.32, a rejection citing OP-4 or OP-2 with `unasserted` view can never
be emitted for any source program. The clauses are dead text.

The reachable strict path that survives retirement is [FN-8]'s
call-requirement rejection in U, in both of its forms: a call inside a
demanded strict component, and a call from outside the closure directly into
a marked strict root. The conformance corpus already pins the second form
(`fn8-neg-strict-outside-caller-unproved-requirement`), and the strict
program-start requirement query is the same FN-8 rejection at the entry's
final `check_stmt`.

## 2. Why they are unreachable — the rule arithmetic

Three spec facts compose into the result. Each is quoted from v0.32 at its
line number.

**(a) U differs from the complete state only by S3.** `spec/kernel-spec.md:3056`:

> The **unasserted state** U is that flow recomputed with S3 establishment
> disabled and every other source, kill, join, loop rule, and closure
> unchanged.

S3 is [ENT-3.S3], claim facts, and it is the only claim-fed source: S2 no
longer exists in v0.32 (it was retired into a self-contained S3 by the
batch-0071 check dissolution). So for a function containing no `claim_stmt`,
U and the complete state are the same closed state at every point, and every
obligation discharged in the complete state discharges in U.

**(b) The one cross-function route into U carries the same restriction.**
`spec/kernel-spec.md:3068`:

> A complete-only S12 relation is absent from U and B; …

and `spec/kernel-spec.md:1340`:

> A complete-only summary may therefore depend on a claim, while a
> U-but-not-B summary may depend on the proved function requirement.

A callee summary can therefore be present in the complete state and absent
from U only when the callee's own [FN-9] proof used a claim — that is, only
when the callee itself contains a `claim_stmt`. A U-but-not-B summary
(S4-dependent) is present in U, so it never creates the gap. No other fact
crosses a function boundary: [ENT-2] fixes that "No caller fact is copied
into a callee", and S4 is established in the complete state *and* in U.

**(c) A claim anywhere in the demanded closure is rejected first.** [CLM-3]
at `spec/kernel-spec.md:2722–2724`:

> For one marked root, a nonempty `DirectClaims` set in its own SCC rejects
> at the least direct claim node; otherwise the first call in stable
> caller-instance then call-NodePath order within that SCC whose strictly
> outgoing callee component has nonempty `MayClaims` rejects there as an
> imported-claim event.

`MayClaims(K)` is `DirectClaims(K)` unioned with the `MayClaims` of every
strictly outgoing callee component, and the closure of a root is "its root
component plus every component reachable along outgoing edges". So every
component in the closure is either the root component itself (covered by
`DirectClaims`) or reached by a chain whose first hop leaves the root
component (covered by that hop's `MayClaims`). Either way, one claim
anywhere in the closure produces a CLM-3 event.

**Composition.** Take any protected obligation owned by a component of a
demanded closure whose complete-state judgment succeeded — a failing
complete-state judgment is the ordinary OP-4/OP-2 rejection and the program
never reaches CLM-3. For the strict clause to fire, that obligation must fail
in U, so some fact present in the complete state must be absent from U. By
(a) and (b) that fact is a claim fact of the obligation's own function, or a
complete-only summary whose callee holds a claim; the callee is in the
closure because the call edge that imports the summary is exactly an outgoing
closure edge. Either way the closure holds a claim, and by (c) CLM-3 rejects
before the U obligation query. The strict obligation arms are therefore
unreachable. ∎

**The implementation agrees, at the same three points.** `S3` is established
at exactly one site, into the complete state alone
(`compiler/src/semantic/entailment/flow.rs:5776`), under a comment that says
so: "`claim` [CLM-1] is the sole writer-stated source, at S3"
(`flow.rs:5701`) — a `Check` statement, which after check dissolution is only
a contract final, establishes nothing. S4 is established into the complete
state *and* U (`flow.rs:517–518`). Every other source, kill, join, arm-fact,
counted-preheader, and goal-origin operation is applied to all three views,
either through `ViewStates::for_each_mut` or through three explicit sibling
calls. And `check_strict_partition` raises the direct-claim and
imported-claim events before it ever calls `strict_closure_failures`
(`compiler/src/semantic/check/strict.rs`), matching §(c)'s order.

## 3. Refutation attempts

Nine programs were written against the v0.32 compiler at
`762fb016`, deliberately targeting the shapes most likely to break the
argument. None reached a strict OP-4/OP-2 rejection. Full sources and
verbatim diagnostics: `research/investigations/strict-clause-retirement/probes/`.

| probe | shape aimed at the strict arm | observed |
| --- | --- | --- |
| `r1.wf` | complete-and-U-but-not-B summary (callee postcondition proved from its own S4) feeding a bounds obligation inside a marked root | **accepted** — U carries the S4-dependent summary |
| `r2.wf` | complete-only summary: callee postcondition proved only by a claim, bounds obligation in the marked root | `[CLM-3] StrictImportedClaim` at the root's call |
| `r3.wf` | mutual-recursion SCC: claim in the root's SCC sibling, subscript in the root | `[OP-4] UndischargedBoundsObligation` — the ordinary complete-state rejection; facts do not cross the function boundary at all |
| `r4.wf` | claim two components below the root, relayed through a claim-free middle | `[CLM-3] StrictImportedClaim`, least downstream claim named as `leaf` |
| `r5.wf` | claim inside a generic body, one instance inside the closure and one outside | `[CLM-3] StrictImportedClaim` on `bounded$instance$3` |
| `r6.wf` | marked `command` program entry, bounds obligation discharged only by a claim | `[CLM-3] StrictDirectClaim` (lifecycle `Redundant`) |
| `r7.wf` | constant-operand **overflow** obligation discharged only by a claim, in a marked root | `[CLM-3] StrictDirectClaim` |
| `r8.wf` | divisor-class **division** obligation discharged only by a claim, in a marked root | `[CLM-3] StrictDirectClaim` |
| `r9.wf` | claim-free strict closures discharging by S1 branch, S11 counted range, divisor-class branch, and constant-operand-overflow branch | **accepted** — all four survive the U query |

`r7` and `r8` are the direct tests of the two OP-2 arms and `r6`/`r2` of the
OP-4 arm: in each case the claim event fires first, exactly as §2(c) says.

## 4. Exact edits

Every edit deletes text; none adds a rule, changes a rule id, or alters the
135-rule inventory. Line numbers are v0.32.

### 4.1 [OP-2] — delete the strict overflow clause

Delete `spec/kernel-spec.md:806` in full:

```
For a class call in a [CLM-3] demanded strict component, the same normalized obligation must additionally discharge in that function's already-computed unasserted U state [ENT-6]; a refuted or unproved strict judgment is a hard rejection citing OP-2 at the same `infix` node, carrying the same exact residual plus the strict root, concrete function instance, and `unasserted` view, and its mechanical repair is [OP-4]'s strict repair.
```

The surrounding constant-operand-class paragraph is otherwise unchanged: line
805 (ground false conjunct) is followed directly by line 807 (the retained
two-non-constant trapping judgment).

### 4.2 [OP-2] — delete the strict division clause

Delete `spec/kernel-spec.md:852`, which is byte-identical to 806. Line 851
(ground false conjunct) is followed directly by line 853 (the retained signed
two-non-constant trapping judgment).

### 4.3 [OP-4] — delete the strict subscript paragraph, keep its last sentence's content

Delete `spec/kernel-spec.md:879–882`:

```
For a protected subscript in a [CLM-3] demanded strict component, the complete-state base judgment and every applicable [PRV-2] or [PRV-3] judgment above still run first.
After those succeed, the same normalized obligation must additionally discharge in that function's already-computed unasserted U state [ENT-6].
A refuted or unproved strict judgment is a hard rejection citing OP-4 at the same `psuffix` node, carrying the same exact residual plus the strict root, concrete function instance, and `unasserted` view; it creates no new runtime bounds check, provenance event, fallback, fact source, or caller-side duplicate.
Its mechanical repair is a dominating real branch or another non-assertion fact source admitted by [ENT-3]; a claim is not a strict repair.
```

Line 883 —

```
An unmarked function outside every demanded strict closure keeps exactly the preceding ordinary judgment.
```

— must be **retained but reworded**, because after 879–882 go it no longer has
an antecedent inside OP-4. Replacement (one sentence, replacing line 883, and
the preceding blank line 878 is deleted with the paragraph so the sentence
joins the base paragraph):

```
Every function, marked or unmarked, uses exactly this judgment: [CLM-3] adds no second subscript judgment and no strict subscript repair.
```

This is the load-bearing consequence of the retirement and is worth stating
positively rather than leaving as an absence.

### 4.4 [CLM-3] — narrow the strict success condition

`spec/kernel-spec.md:2718`, current:

```
A demanded component succeeds strictly exactly when its `MayClaims` set is empty, every protected obligation owned by the component discharges in its owning function's existing unasserted U state [OP-4, OP-2, ENT-6], every ordinary user-call requirement owned by the component discharges at that call in caller U [FN-8], and every strictly outgoing demanded callee component has a successful strict summary.
```

Replacement:

```
A demanded component succeeds strictly exactly when its `MayClaims` set is empty, every ordinary user-call requirement owned by the component discharges at that call in caller U [FN-8], and every strictly outgoing demanded callee component has a successful strict summary.
```

Add, immediately after that sentence, the one sentence that records why the
obligation conjunct is gone — the fact a reader would otherwise have to
rediscover:

```
No separate protected-obligation query is stated: U differs from the complete state only by S3 [ENT-6], a complete-only callee summary is exactly a claim-dependent one [FN-9], and an empty `MayClaims` set therefore already implies that every obligation this component's complete-state judgment discharged discharges in U.
```

### 4.5 [CLM-3] — the two reporting sentences

`spec/kernel-spec.md:2724`, current:

```
When a downstream component instead fails a non-claim U judgment, only the actual OP-4 leaf or FN-8 call is reported; no caller-side summary event is created.
```

Replacement:

```
When a downstream component instead fails a non-claim U judgment, only the actual FN-8 call is reported; no caller-side summary event is created.
```

`spec/kernel-spec.md:2730`, current:

```
All strict roots and candidate S12 or delivery facts remain unpublished in one failure-atomic batch; any CLM-3, strict OP-4, or strict FN-8 event discards that batch and the prospective checked program.
```

Replacement:

```
All strict roots and candidate S12 or delivery facts remain unpublished in one failure-atomic batch; any CLM-3 or strict FN-8 event discards that batch and the prospective checked program.
```

### 4.6 [FN-9] and [DIAG-2] — the same three-way phrase, four occurrences

Each of these reads "CLM-3, strict OP-4, or strict FN-8"; each becomes
"CLM-3 or strict FN-8".

- `spec/kernel-spec.md:1400`: `Any CLM-3, strict OP-4, or strict FN-8 rejection discards the whole unpublished batch; …` → `Any CLM-3 or strict FN-8 rejection discards the whole unpublished batch; …`
- `spec/kernel-spec.md:2007`: `A CLM-3, strict OP-4, or strict FN-8 event instead discards them.` → `A CLM-3 or strict FN-8 event instead discards them.`
- `spec/kernel-spec.md:2013`: `On any CLM-3, strict OP-4, or strict FN-8 event, all strict metadata, …` → `On any CLM-3 or strict FN-8 event, all strict metadata, …`
- `spec/kernel-spec.md:1944`: current

  ```
  At one importing call, CLM-3 is selected before a strict FN-8 U failure; a non-claim strict OP-4 or FN-8 failure is emitted only at its actual leaf or call, and no caller-side summary failure is fabricated.
  ```

  replacement

  ```
  At one importing call, CLM-3 is selected before a strict FN-8 U failure; a non-claim strict FN-8 failure is emitted only at its actual call, and no caller-side summary failure is fabricated.
  ```

### 4.7 [DIAG-2] — the retained U derivation roots

`spec/kernel-spec.md:2010` ends:

```
… and the exact existing U derivation root for every demanded protected obligation and call requirement.
```

**Keep this unchanged.** Retirement removes the *rejection*, not the retained
metadata: [ENT-6]'s U obligation dispositions are still computed for every
view, still consumed by the [PRV-2]/[PRV-3] demand partition at
`spec/kernel-spec.md:3061`, and their successful U derivation roots are still
the honest record of what the strict closure proved. Deleting this phrase
would silently narrow the checked program.

### 4.8 [ENT-6] — the CLM-3 query paragraph

`spec/kernel-spec.md:3096`, current:

```
Each demanded protected leaf queries its existing normalized relation in U and each demanded ordinary-call goal queries its existing instantiated goal in caller U; successful queries retain their already-produced U derivation roots.
```

Replacement:

```
Each demanded ordinary-call goal queries its existing instantiated goal in caller U, and each demanded protected leaf retains its already-produced U derivation root as checked metadata without a separate rejection [CLM-3].
```

Line 3095 (`For [CLM-3], the unasserted U state is exactly the unasserted
state U above…`), line 3097 (the marked program-start query), and line 3098
(`These strict queries introduce no new obligation family…`) are unchanged.

### 4.9 [FN-1]

`spec/kernel-spec.md:1028` (`A call consults this policy only where [FN-8]
and [CLM-3] require the existing U judgment.`) is unchanged: it is about
calls, which keep their U judgment.

## 5. What survives

- The `deny_claims` marker, its closure, `DirectClaims`/`MayClaims`, the
  direct-claim and imported-claim events, and every claim-lifecycle
  disposition — untouched.
- [FN-8]'s strict call-requirement rejection in U, inside the closure and
  from an outside caller into a marked root, with its exact diagnostic
  payload (strict root, concrete caller, concrete callee, final-check
  NodePath, instantiated goal, disposition, `unasserted` view) and its
  mechanical repair sentence at `spec/kernel-spec.md:1279`. This is the
  clause `fn8-neg-strict-outside-caller-unproved-requirement` pins.
- The strict program-start requirement query and its FN-8 rejection at the
  requirement final `check_stmt`.
- The three proof views and their whole [PRV-1]/[PRV-2]/[PRV-3] role: U's
  demand partition at `spec/kernel-spec.md:3061` is unaffected, and B's
  bridge partition with it.
- Every ordinary complete-state OP-4 and OP-2 judgment, residual rendering,
  and mechanical fix.

The `deny_claims` guarantee is therefore unchanged in substance: a demanded
closure still contains no claim, and every obligation it owns is still proved
without one. What is removed is a second query that could never disagree with
the first.

## 6. Impact inventory

**Accepted set: unchanged.** The retired clauses can only reject; deleting a
rejection can only widen the accepted set, and §2 establishes the rejection
is never emitted, so nothing widens either. Verified rather than assumed by:

1. the rule arithmetic in §2, whose three inputs are quoted spec sentences;
2. the nine probes in §3, which found no reaching program;
3. the compiler-side check in §7 — the arm's construction sites are
   removed and replaced by an internal-consistency guard, and the full gate
   (`make -C compiler check`, 526-case conformance corpus included) stays
   green, so no existing accepted or rejected program changes verdict.

**Rejected set: unchanged.** No conformance case expects a rejection citing
OP-4 or OP-2 through the strict path. The two cases that list OP-4 among
their exercised rules alongside CLM-3 —
`clm3-neg-body-check-bounds` and `clm3-neg-transitive-check-summary` — both
declare `{"kind": "reject", "rule": "CLM-3"}`, which is exactly §2(c)'s
ordering.

**Diagnostics:** three `SemanticIssueKind` variants and one detail struct
cease to exist. No diagnostic that any program can produce changes.

**Runtime, lowering, checked program:** no change. The retired clauses
already stated that they created "no new runtime bounds check, provenance
event, fallback, fact source, or caller-side duplicate".

**Rule inventory:** 135, unchanged. No rule id is added, removed, or
renumbered.

**Grammar:** untouched; the native grammar verifier is not implicated.

## 7. Compiler change (landed in this batch)

`compiler/src/semantic/check/strict.rs` and
`compiler/src/semantic/mod.rs`.

Deleted: the `StrictFailure::Bounds` variant and its rule mapping; the
obligation half of `strict_closure_failures`; the `StrictUndischargedBounds`
/ `StrictUndischargedOverflow` / `StrictUndischargedDivision` construction in
`reject_strict_failure`; the three `SemanticIssueKind` variants; and the
`StrictUndischargedBoundsDetail` struct. `ObligationFamily` is no longer
needed by `strict.rs`.

Retained as an internal-consistency guard rather than deleted outright: the
scan over `entailment.unasserted.obligations` for a demanded component now
returns `SemanticCompilerFailure::InvalidResolution` — a compiler defect,
never a source-language rejection [DIAG-1] — if it ever finds an undischarged
U obligation in a demanded closure. §2 is an invariant of the checker, not a
hope; encoding it as an internal failure keeps it machine-checked, and
misclassifying a compiler defect as invalid source is exactly what the
project rules forbid.

**Not switch-guarded, deliberately.** A switch exists to let two behaviors be
compared. These two behaviors are indistinguishable by construction: no
program reaches the deleted arm, so a switch would select between identical
observable results at the cost of a permanent branch in the checker. The
change is also observationally conformant with ACTIVE v0.32 for that same
reason, so the compiler does not drift from the active specification while
this delta waits for approval.

## 8. PROPOSED conformance dispositions

Every item is a proposal; the conformance corpus is protected and no byte of
it is edited by this task.

| case | current | proposed disposition |
| --- | --- | --- |
| `fn8-neg-strict-outside-caller-unproved-requirement` | reject / FN-8 | **keep unchanged.** This is the surviving strict-in-U path and the only one. Its doc string already says "the one strict-in-U path an ordinary caller reaches"; under this delta that sentence becomes exactly true rather than nearly true. |
| `clm3-neg-body-check-bounds` | reject / CLM-3 | keep the verdict. Independently of this delta it is already on the batch-0071 re-authoring list (a body `check` no longer parses under v0.32); if it is re-authored on a claim, the CLM-3 verdict is unchanged and this delta adds no further requirement. |
| `clm3-neg-transitive-check-summary` | reject / CLM-3 | same as above. |
| `clm3-neg-body-check-requires`, `clm3-neg-direct-unreachable-claim`, `clm3-neg-generic-first-import`, `clm3-neg-mutual-scc-import`, `clm3-neg-generated-wrapper-check`, `clm3-pos-upward-near-miss`, `clm3-pos-transitive-value-branch` | unchanged | **keep unchanged.** None exercises a strict obligation query. |

**One proposed addition**, to keep the surviving surface pinned from both
sides rather than only the negative side:

- `clm3-pos-strict-obligation-in-u` — accept. A `deny_claims` root whose
  closure carries a bounds obligation, a constant-operand overflow
  obligation, and a divisor-class division obligation, each discharged by a
  dominating branch or an S11 counted range and none by a claim, plus one
  callee whose FN-9 summary is available in U through its own proved
  requirement. Source: `probes/r9.wf` merged with `probes/r1.wf`. It runs
  today and must keep running after retirement; it is the positive
  counterpart of `fn8-neg-strict-outside-caller-unproved-requirement`.

No case is deleted, weakened, or re-verdicted by this delta.

## 9. Residual risk

One. The argument in §2 is an argument about v0.32's fact sources. If a later
version adds a second complete-only source — a fact established in the
complete state and not in U — the strict obligation arms would become
reachable again, and this delta would have removed the clause that handled
them. Two things bound that risk:

- the retirement is stated in [CLM-3] as a derived consequence (§4.4's added
  sentence) rather than silently, so the next author who adds such a source
  reads why the conjunct is missing; and
- the compiler keeps the invariant as an internal-consistency guard (§7),
  so a future complete-only source would surface as a loud compiler failure
  on the first program that reaches it, not as a silently accepted program.
