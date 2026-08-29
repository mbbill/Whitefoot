# The claim model, redesigned: premise ownership

Design deliverable for batch 0106. Tree read: `integration/2026-08-28c`, tip
`b1367c82`, spec v0.39 ACTIVE. Bare line numbers are `spec/kernel-spec.md` at
that tip; every other citation names its file.

This is the single design the project should implement. It is a synthesis: three
independent designs were written from three angles (prover-first, world-first,
writer-first) and reviewed by two adversarial judges who landed twenty
counterexample programs between them. The map they all worked from is
`TERRAIN.md`, beside this file. §10 records which idea came from where, which
were rejected, and by which counterexample.

Nothing here is implemented. Every verdict change in §7 is a proposal, and the
specification text in §3 is draft text for a work branch, not an amendment.

**Census headlines.** `CENSUS.md`, beside this file, is batch 0106's B0
measurement of the corpus this design reasons about, and three of its numbers
should be read before §5 and §7. (1) Of the 114 gap-stating claims in the tree,
**108 are forward publisher gaps, 4 are backward transfers over a `±wrap` row,
and 2 are true residue** (§2.3 there). (2) **No claim anywhere in 682 source
files has selector width m ≥ 2**, and only three have m = 1 (§3.3 there). (3)
**All 18 real-program claims discharge SubscriptBounds**, and there is **not one
`claim` statement anywhere in the blind-writer corpus** (§1.4, §2.1 there). The
first and third strengthen the principle — a claim in this tree names a missing
publisher, not a missing fact language — and the second and third strengthen
§11's U1: the laundering family needs a boundary-derived selector, and in real
code that family is empty, because every function holding a real-program claim
contains no call at all, so review is being asked to fence a shape nobody has
yet written. Two things narrow. The refuter §10.2 rejected on correctness
grounds (A2–A4) is also moot on evidence: nothing anyone has written would
exercise it, and a feasibility study would have to construct its programs rather
than find them. And of the families this design still argues about, the
loop/`flow` family is the one live syntax-versus-prover choice left — the true
residue is two claims, and the backward-transfer gap is one row rule in the
existing vocabulary (§5.3's erratum).

## Contents

1. [The principle](#1-the-principle)
2. [The complete case walk](#2-the-complete-case-walk)
3. [The specification replacement text](#3-the-specification-replacement-text)
4. [The judgment architecture](#4-the-judgment-architecture)
5. [The prover ceiling](#5-the-prover-ceiling)
6. [Non-duplication and residuality](#6-non-duplication-and-residuality)
7. [Conformance migration](#7-conformance-migration)
8. [Implementation plan](#8-implementation-plan)
9. [The attacks the judges landed](#9-the-attacks-the-judges-landed)
10. [Provenance of the ideas](#10-provenance-of-the-ideas)
11. [Unsolved problems, in red ink](#11-unsolved-problems-in-red-ink)
12. [Open questions for the owner](#12-open-questions-for-the-owner)

---

## 1. The principle

### 1.1 Stated once

> **Premise ownership.** Every premise a Whitefoot proof may use is published by
> exactly one of three publishers, and each publisher's output is fixed by this
> specification.
>
> 1. **The entailment** publishes what the operation table and the control graph
>    entail about values this function's own text produced.
> 2. **The callable boundary** publishes what a machine-verified contract
>    (`requires`, an FN-9-verified `ensures`) or a specification-fixed operation
>    contract states about values a callee or the world produced.
> 3. **The reviewed claim** publishes one always-true function-local lemma about
>    values this function's own text produced, standing above the entailment's
>    published ceiling.
>
> A premise no publisher publishes is available to no one: not to the checker,
> not to the writer, not to the reviewer.

Everything below is a consequence. The area has had all three publishers since
v0.20-something; what it has never had is the statement that they are three, that
their subject matter is disjoint, and that each one's output is *published* —
enumerable, fixed by the specification, and citable.

### 1.2 The four consequences

**(C-I) The claim gate is a subject-matter rule, not a position rule.**
Publisher 3 may speak only about publisher 1's subject matter: values this
function's own text produced. A value a callee returned, or wrote into caller
storage, is publisher 2's subject matter, and publisher 3 may not restate it —
that is constitution W3 and [CLM-1] 2720 word for word. *Choosing an edge
publishes no value*, so a selector transfers no subject matter. H1 and H3 are
both decided here, in opposite directions, by one question: **did a boundary
produce this value?**

**(C-II) Publisher 1's output must be a closure, and its complement must be
published too.** "Published" is not satisfiable by a list chosen one construct at
a time. Today [ENT-3] is eleven numbered sources — S5 for copies, S6 for lengths,
S7 for `p ± k`, S9 for named const arrays, S10 for two of [SYS-8]'s seven
relations, S11 for counted loops — and S8 was struck with the note that it "may
return as a later version's monotone addition the day a corpus program writes the
shape" (3009). That sentence *is* the defect, in the specification's own voice: a
publisher whose output grows by petition has no published output. Replace the
list with one closure indexed by the operation table and the control graph — both
already closed, both already normative — and publish its complement as a finite
set of gap kinds. H2's prover half is decided here.

**(C-III) Two publishers may not publish the same premise.** A claim whose
content publisher 1 already publishes is a duplicate publication: it costs a
retained runtime check on every reach (2748) and buys nothing. That is H4's
verdict, derived rather than inherited, and it also derives *why* the verdict must
flip when the ceiling rises — a ceiling raise moves content from publisher 3 to
publisher 1, and the writer's edit is to stop publishing it twice.

**(C-IV) Every refusal names the publisher who owns the premise the writer
needs.** A writer refused today is told which rule said no. Under the principle
they are told which of the three publishers owns what they are missing and how to
get it published: raise it from the boundary (a contract), from the text (a
branch or a different spelling), or from review (a claim). H5's uniform teaching
channel is that sentence, mechanised.

### 1.3 Why premise ownership, and not one of the three angles

Each of the three input designs found one true thing and mislocated it.

- *world-first* derived the gate from what a boundary can inject, and got C-I
  right, but then asserted its judgment computes *settlability of truth* when it
  computes *enumerability of the value set*, and both judges refuted the
  equivalence with three-line programs. Ownership separates the two cleanly: the
  gate decides **subject matter**, review decides **truth**.
- *writer-first* derived the gate from what a reviewer may cite and got the same
  answer, more honestly — it stated its condition as necessary and not
  sufficient — but declined to move publisher 1 at all, leaving the ceiling as
  the eleven-source list and the writer paying a retained runtime check where a
  proof was available.
- *prover-first* built publisher 1's closure, which is the best construction in
  the batch, but then tried to recover the review residue with a machine (a
  witness state and a `refuted-on-a-path` verdict) that both judges broke, and
  shipped a loop rule that derives a false fact.

Premise ownership is what the three were each half-stating. It is also the frame
in which each of the five charter holes is the *same* defect rather than five:

| hole | the defect, in one sentence |
| --- | --- |
| H1 | Edge choice was treated as a publication, so publisher 2's subject matter grew to cover values it never produced. |
| H2 | `0 ≤ picked ≤ 1` had **no publisher**: publisher 1's list had no entry for a delivered literal, and publisher 3 was refused by H1's error. A premise with zero publishers is a program with no legal spelling. |
| H3 | A value a callee produced had **no owner**: publisher 2 produced it, publisher 1 did not model it, and publisher 3 was permitted to speak about it. |
| H4 | Publishers 1 and 3 publishing the same premise, with no rule saying which one must stop. |
| H5 | The writer was never told which publisher owns the premise they lack. |

That table is the design's central argument. If it is right, one principle covers
the space and no case-by-case exception is needed. §11 records the one place
where it does not reach, in red ink, as an unsolved problem rather than a rule.

### 1.4 The one thing the principle does *not* decide

Ownership decides who may speak about a value. It does not decide whether what
they said is true. A claim over values the function's own text produced whose
truth nevertheless depends on which arm a boundary selected — the *laundering
family* — passes the gate and is refused, if at all, by review.

```whitefoot
fn hidden() -> result: own u64 pure { return 9_u64; }

let n = hidden();
let big = ige(n, 4_u64);
let y = if big { give 5_u64; } else { give 1_u64; }
claim laundered: ilt(y, 4_u64) because "…";   // admitted. FALSE when big.
```

`y`'s support reads two literals; no boundary produced it; the gate admits. The
predicate is materially `n < 4`.

This is not an exception carved into the rule — the rule is about subject matter
and it answers correctly that `y` is publisher 1's subject. It is the **limit of
what a mechanical subject-matter test can do**, and three independent facts say
no setting of the dial improves it:

1. To refuse this program while admitting `claim ilt(y, 4_u64)` over arms
   `{0, 1}`, a checker must compute the joined value set. A checker that computes
   it derives `y < 4` for the good program outright, and the good claim is then a
   duplicate publication (C-III) and is refused anyway. **The checker is on the
   wrong side of both halves.** (writer-first §1.6, and neither judge disputed
   it.)
2. Every path-sensitive technique that would separate them — trace partitioning,
   powerset domains, CEGAR, guarded facts — needs a merge policy, a disjunct
   bound, search, or a second logic, and [ENT-1] 2831/2836 forbid all four.
3. The one mechanical defence proposed in the batch, prover-first's
   `refuted-on-a-path`, was refuted twice: it rejects true text-reviewable claims
   (judge-1 §1.2) and its tag is a path *suffix*, so two bounds sharing a tag
   need not share an execution (judge-2 F-2). §9 gives both programs.

So the design states the limit in the specification instead of pretending to
close it, compensates it with published review evidence (§4.5's case list and
§3.8's review note), and proposes the measurement that would move the line
(§8's review trial). T3 already lives with fallible reviews and says so: "an
execution that reaches it is a defective program **whose review was wrong**"
(`docs/constitution.md` T3). What T3 needs is that the review be *possible*, and
premise ownership is exactly the property that makes it possible.

### 1.5 The reviewer's rule, made precise by the principle

The compensation is not hand-waving, because ownership sharpens 2720 into a
checkable procedure. [CLM-1] 2720 already says review "may not use … an unstated
caller or environment promise, a user callee's body or unstated system behavior
in place of a verified or specification-fixed callable-boundary fact". Under the
principle that reads:

> **A `derivation:` line may cite only published premises.** Each step cites a
> fact the entailment publishes at that point, a contract the callable boundary
> publishes, an earlier reviewed claim named in `premises:`, or the function's own
> text. A step that needs an unpublished premise makes the record invalid and the
> reviewer refuses.

In the laundering program the reviewer's premise set contains "y is `5_u64` or
`1_u64`" (text) and *nothing whatever about `hidden`*, because `hidden` publishes
no contract. To conclude `y < 4` the derivation must decide which arm ran, which
needs an unpublished premise. **The record is invalid and the review refuses.**
In the sibling with arms `{0, 1}` the derivation covers both cases from the text
and is valid.

That is why "settlable" is the right word and "true" is not: the reviewer's job is
to decide whether the record's derivation stands on published premises, which is
a finite, mechanical-in-shape task once the case list is in front of them.

### 1.6 What the principle does to the constitution

- **T3 is untouched and needs no re-derivation.** A claim still means what 2696
  says it means: true on every execution reaching the statement, reviewed,
  retained, never elided. No construct admitting non-always-true predicates is
  introduced.
- **W3 is strengthened in one place and clarified in another.** Strengthened:
  H3's carve-out was the one route by which an unverified cross-function promise
  could become a lemma, and §3.4 closes it. Clarified: W3's phrase "over the
  current function's own value **and control** authority" was written to license
  the CLM-1 clause this design deletes. §12 Q1 puts the reading to the owner with
  a recommended amendment.
- **R4 improves.** Every verdict this design moves either removes a runtime check
  (a claim becomes a proof) or converts a compile-time refusal with no legal
  repair into a compiling program.

---

## 2. The complete case walk

Every row is decided by two questions and nothing else:

- **admission (publisher 3's subject matter):** does the predicate's support read
  a value a callable boundary produced?
- **necessity (C-III):** does publisher 1 already publish this content?

No row consults a construct kind, a source shape, a selector, a position, a
nesting depth, or a count. "v0.39" is the verdict at `b1367c82`; "here" is this
design's verdict; the delta column marks a move.

| # | shape | v0.39 | here | delta |
| --- | --- | --- | --- | --- |
| 1 | ground expression (`3209 % 64`) | accept | **CLM-2 duplicate** | ✔ |
| 2 | named consts | accept | **CLM-2 duplicate** | ✔ |
| 3 | parameter through a total row (`cursor % 4`) | accept | **CLM-2 duplicate** | ✔ |
| 4 | `%`, `/`, `imin`, `imax`, `iand`, shifts over locals | accept (46 corpus claims) | **CLM-2 duplicate** | ✔ |
| 5a | `value_if` delivering literals | **reject CLM-1** | **CLM-2 duplicate** | ✔ |
| 5b | `value_if`/`value_match` delivering a call payload | reject CLM-1 | reject CLM-1 | — |
| 6a | arm-written storage, both definitions below the bound | **reject CLM-1** | **CLM-2 duplicate** | ✔ |
| 6b | arm-written storage, one definition above the bound | reject CLM-1 | **accept**, retained claim + review note | ✔ |
| 7 | loop-carried literal update under a boundary endpoint | **reject CLM-1** | **accept**, retained claim (`flow` gap) | ✔ |
| 8 | loop accumulator (`acc = acc + 1`) | accept (39 corpus claims) | accept (`flow` gap) | — |
| 9a | storage a callee wrote through `&uniq` | **accept** (carve-out 3242) | **reject CLM-1**, gap `boundary` | ✔ |
| 9b | length of a buffer a callee whole-place replaced | accept | **reject CLM-1** | ✔ |
| 10 | `len(deref buf)` bound before a callee element write | accept | accept | — |
| 11 | payload of a matched call result | reject CLM-1 | reject CLM-1 | — |
| 12 | restatement of a verified `ensures` result | reject CLM-1 | reject CLM-1 | — |
| 13 | claim inside a boundary-selected arm | accept | **CLM-2 duplicate** or accept, by facts | ✔ (reason) |
| 14 | claim after a typed exit (the 0097 pair) | accept | **CLM-2 duplicate** | ✔ |
| 15 | nested boundary selections over literals | **reject CLM-1** | **CLM-2 duplicate** | ✔ |
| 16 | a [SYS-8] outcome relation outside today's S10 | reject CLM-1 | reject CLM-1; the obligation now discharges with no claim | ✔ (route) |
| 17 | a value of an array element | accept | accept (`content` gap) | — |
| 18 | **laundering** (literal arms, boundary-decided truth) | reject CLM-1 | **accept; review refuses** | ✔ ⚠ |
| 19 | two-conjunct claim, one conjunct derivable | reject CLM-2 overlap | **CLM-2 duplicate (component)**, fix names the ordinal | ✔ (fix) |
| 20 | claim over a bare parameter, false in general | accept; review refuses | unchanged | — |

Row 18 carries a warning mark and is the subject of §11.

### 2.1 Rows 1–4 — the ground case and the remainder family

```whitefoot
let seed = 3209_u64;
let offset = seed % 64_u64;
claim guard: ilt(offset, 64_u64) because "…checker gap: the entailment state
  carries no residue for a literal remainder, so the bound is left to this
  executed check…";
```

That is `accept-clm1-local-claim-after-boundary-exit.wf`, the accepted member of
the differential-fuzz pair batch 0102 was written to admit, and its own `checker
gap:` field is the confession: **v0.39's flagship accepted claim is a workaround
for a checker that cannot evaluate a closed ground expression.** Under §5's image
closure the `%` row publishes `Z ≤ r`, `r ≤ d − 1` and `r ≤ a`; with both operands
pinned the result is pinned; the predicate is derivable. **Duplicate publication;
the claim is deleted and the program compiles with no runtime check.**

There is no constant rule and no ground-folding rule. `3209 % 64` is folded by
the same image rule that serves `index % 19_u64` with `index` a parameter (row 3)
and `a % d` with both variable (row 4): the image is a function of the operand
bounds, and a literal's bounds pin it. That is the whole of C-II — a closure has
no cases.

Row 4 is 46 of the corpus's 135 claims, whose gap text is written 35 times
verbatim: *"ENT proves the remainder operation domain but does not publish its
result range"*. It is a precise bug report against the eleven-source list: the
checker proves `%`'s **domain** (IntegerDomain, `d ≠ Z`, 3148) and discards its
**range**, because S7 has a row for `p ± k` and none for `%`. All 46 retire.

### 2.2 Row 5 — delivered values, and why this is not a constants exception

```whitefoot
let condition = hidden_true();
let picked = if condition { give 0_u64; } else { give 1_u64; }
claim reviewed_delivery: ilt(picked, 4_u64) because "…";
return values[picked];
```

`reject-clm1-claim-on-delivered-selection.wf`, a normative reject at `b1367c82`,
and the sharpest artifact in the charter: the predicate is true on every
execution, its truth reads nothing the callee returned, and **both** documented
writer routes are closed. [ENT-6] 3169 promises "one dominating branch
establishing its canonical goal, or a CLM-2-admissible residual claim"; here the
dominating branch *is* the `value_if`, and the claim is refused by CLM-1. The
program has no legal spelling.

Both sides move, independently and generally.

*Admission:* `picked`'s support reads two literals. No boundary produced them.
`condition` chose an edge, and choosing publishes no value. **CLM-1 admits.**

*Necessity:* under §5's flow closure a `value_if` receiver is an ordinary merge
point, and each delivering edge establishes exactly the image an ordinary
`let x = a;` would establish for its delivered atom at that point. `give 0_u64;`
therefore establishes `picked = 0`, `give 1_u64;` establishes `picked = 1`, and
the merge — the same merge as everywhere else, weakest bound held by all (3097) —
gives `0 ≤ picked ≤ 1`. The predicate is derivable. **CLM-2 duplicate; the claim
is deleted; the program compiles with no runtime check.**

The charter warned that a union fact for delivered constants is exactly the
case-by-case exception to avoid, because next comes constant-plus-one, a product
of two constants, a length read from a file. The design opens none of them as
cases, because there is no delivery rule at all any more:

| next case | who publishes it, and why |
| --- | --- |
| `give 0_u64` | publisher 1: the literal row's image, reached because a delivery edge is a merge input like any other. |
| `give p +wrap k` | publisher 1: the `+wrap` row's image. Under the closure this needs no new sentence; today it fails only because GIVE-1 292 restricted the *carrier* by spelling. |
| `give imul.wrap(3_u64, 4_u64)` | publisher 1: the multiply row's image, which pins a product of two pinned operands. |
| a length read from a file | **publisher 2.** It is a value the world produced. No image, no claim: a contract or a branch. |
| `sum ≤ i * stride` across a loop | publisher 3, gap kind `vocabulary`: a product of two non-constant terms is not a difference bound. |

Four different answers, one rule each, none of them a case.

Note also that TERRAIN #3 is right and must not be mis-read: the DBM join of
`x = 0` and `x = 1` already yields `0 ≤ x ≤ 1` correctly, because both directed
pairs are present in both inputs. H2's loss was never the join. It was that
`give 0_u64;` established **no image at all**.

Row 5b is unchanged and is decided by the same sentence: `give w;` where `w` is a
`ReadBytes` payload puts a boundary-produced value in the support, so CLM-1
refuses, with the fixed restructuring at [DIAG-1] 1859.

### 2.3 Rows 6a/6b — arm-written storage, and the third spelling defect

```whitefoot
let cursor = 3_u64;
if condition { set cursor = 0_u64; } else { let untouched = 0_u8; }
claim reviewed_written: ilt(cursor, 4_u64) because "premises: cursor is either
  the literal 3_u64 this function wrote before the branch or the literal 0_u64
  the selected arm wrote…";
```

`reject-clm1-claim-on-storage-written-under-selection.wf`. Its own `derivation:`
line — *"both literals are below 4_u64, so cursor is below 4_u64 whichever
definition reached this point"* — is a sentence a reviewer settles in eight lines
and the checker can now check.

*Admission:* both reaching definitions are literals. **CLM-1 admits.**

*Necessity:* here the design closes a defect nobody put on the charter's list.
Today `let x = 0_u64;` establishes `x = 0` under S5 (2963–2964) and
`set x = 0_u64;` establishes **nothing** — `set` appears in [ENT-3] and [ENT-5]
only as a kill event (3036, 3066(a)) — although the two have identical value
semantics. That is a distinction by spelling, which `CLAUDE.md` forbids
("by grammar and semantic rule, never by shape"). Under the closure it cannot
arise: **the image closure is indexed by the operation and its destination, not
by the binding form.** A `set` commits a value produced by a row into a place;
the row's image is established on that place after the `set`'s own kill, in the
edge order [ENT-5] already fixes. The then edge carries `cursor = 0`, the else
edge `cursor = 3`, the merge gives `0 ≤ cursor ≤ 3`, and the predicate is
derivable. **CLM-2 duplicate.**

Row 6b is the same program with `9_u64` in place of `3_u64`. Admission is
identical; the merged state is `0 ≤ cursor ≤ 9`; the predicate is neither derived
nor refuted. **Accepted, retained claim**, with the review note of §3.8:

```text
review note [CLM-1] `reviewed_written`: at this statement the entailment does not
exclude cursor = 9_u64, whose definition is `let cursor = 9_u64;` at f.wf:3:7 and
which reaches here through the else edge of the match at f.wf:4:3. The reviewed
derivation must cover that case.
```

That is prover-first's best writer-facing text, recovered from the reviewer's
case list and the certain state, and emitted as **review evidence, never as a
verdict**. §9 shows the two programs that killed it as a verdict.

### 2.4 Rows 7–8 — loops, and why there is no loop clause

```whitefoot
let upper = endpoint(value: input);          // a call result
let cursor = 0_u64;
for @steps step in 0_u64..upper { set cursor = 1_u64; }
claim reviewed_carried: ilt(cursor, 4_u64) because "…";
```

`reject-clm1-claim-on-loop-carried-update.wf`. *Admission:* `cursor`'s support
reads two literals; `upper` decided how many times the loop ran and produced no
value into `cursor`. **CLM-1 admits.** *Necessity:* v0.40's loop head is still
[ENT-5] 3110/3120's subtraction (§5.4 explains why, and publishes it as a `flow`
ceiling clause), so the pre-loop fact is subtracted and nothing reaches the
continuation. **Accepted with a retained claim**, gap kind `flow`. The program
compiles; today it does not.

**A loop clause was proposed and is rejected.** world-first's clause 3 —
a component whose back-edge definition transitively reads something the body
redefines joins the loop's "repetition class" — was judge-1's favourite idea in
that document. It does not survive. Judge-2's program:

```whitefoot
let n = endpoint(value: input);
let flag = 0_u64;
for @steps step in 0_u64..n { set flag = 1_u64; }
claim never_ran: ieq(flag, 0_u64) because "…";   // admitted by clause 3
```

`flag`'s back-edge definition is a literal, so it is not trip-count-dependent by
the clause's own test, yet `flag == 0` is true exactly when `endpoint` returned
zero. The clause catches self-composition and misses "whether a definition
executed at all". Repairing it means taxing every component a loop body may
redefine whenever the repetition is boundary-decided — which is v0.38's blanket
restricted to loops, and which refuses row 7, the case the clause was introduced
alongside. And a rule that could separate row 7 from `never_ran` must compute the
value set, which makes row 7's claim a duplicate publication and gives the rule
nothing to do (§1.4, fact 1).

**Therefore there is no loop clause.** A loop head is a merge whose inputs are the
preheader and the back edges; a merge publishes no value; loop-carried components
are decided exactly as row 6 is. Row 8's accumulator and judge-1's
`claim ilt(acc, 4_u64)` under a boundary-decided endpoint are members of the
laundering family (§11), decided by review, with the case list rendering the
back-edge definition **as a recurrence** so the reviewer knows an induction is
owed and does not mistake it for a two-case enumeration.

### 2.5 Rows 9a/9b/10 — callee-written storage, and the separating pair

```whitefoot
fn fill['s](slot: &uniq 's u64) -> result: own unit writes(slot) { … }

fn read(values: own array<u8, 4>) -> result: own u8 traps {
  let cursor = 0_u64;
  region 'w { let done = fill<'w>(slot: &uniq 'w cursor); }
  claim written_by_callee: ilt(cursor, 4_u64) because "premises: fill writes a
    value below four into slot…";              // ← the forbidden citation
  return values[cursor];
}
```

Under v0.39 this compiles: `cursor` seeds `Local` (3222), no call *result* reaches
it, and 3242 makes the call's `&uniq` write invisible to claim authority. The
claim is true only because of `fill`'s body — the citation 2720 forbids. **Here
it is a CLM-1 refusal**, gap kind `boundary`.

The seed is defined without a new projection: **a call joins its witness into
exactly the components whose facts [ENT-5]'s kill (b) would kill at that call**,
under the same [EFF-2] projection and the same [OWN-7] overlap. Three consumers,
one relation — kills (3066), [PRV-1] write components (3207), and now admission.
Today the three disagree about one event: kills see the `&uniq` write, provenance
sees it, and only claim authority looks away (TERRAIN §1.8).

Two consequences fall out of the existing kill rule and are **not legislated**:

- *Row 10 survives.* [ENT-5] 3036 puts a length term's support at the root
  binding of the viewed place and excludes element storage, and 3066(a) says an
  element write kills no length fact. So a callee that writes elements through
  `&uniq buf` taints the all-elements component and not the length, and
  `claim c: ilt(i, room)` over `room = len(deref buf)` bound before the call stays
  admissible. That is `docs/patterns.md` P16, whose whole content is that the
  length fact survives a callee write, and which 34 of 41 length bindings in the
  blind-writer trial existed because writers disbelieved.
- *Row 9b refuses.* 3066(a) also says "a whole-place replace of a buffer or of any
  prefix of it **kills** that buffer's length facts". So a callee that
  whole-place-replaces a `&uniq buffer<u8>` with one sized by a call result taints
  the length, and `claim ilt(len(room), 4_u64)` after it is refused. A design that
  *legislates* a blanket length exemption instead of deriving it from the kill
  projection admits that program; judge-2 wrote it as F-4 against world-first.

Rows 9a and 10 are TERRAIN #16's separating pair — "authority follows the storage
root" versus "authority follows the last writer" — and the answer is neither:
**authority follows the components the declared write projection reaches**, which
is what the fact state already computes.

*One ambiguity to close before B2 writes a line of text.* 3036 excludes element
storage from a length term's support and says so twice, and its last clause
already covers this case — "so a `set` commit **or projected callee write**
through the chain kills exactly the facts supported by that storage". But
3066(b)'s prose reads "a callee writing only through one `&uniq` actual kills
exactly the facts whose support overlaps **that actual's resolved place**", and a
whole-buffer actual's resolved place does overlap a length term's root binding.
The two sentences are reconciled only by [EFF-2]'s projection carrying element
granularity, which 2068 permits ("those the callee's own summary fixes … after the
[EFF-2] boundary projection, and the exact subscripted position for a direct
element access") and which the compiler implements — `KillEvent::Write` carries an
`element: bool` whose doc comment is the rule
(`compiler/src/semantic/entailment/flow.rs:60–66`). **Row 10 and P16 depend on
that reading.** B2 must confirm it against [EFF-2] and, if the projection is
coarser than the compiler assumes, the defect is in the fact state first and in
admission only as a consequence — which is §4.4's point about a shared blast
radius, arriving earlier than expected.

The writer's repair is the one every surveyed tradition names (TERRAIN §5.4):
publish it on the boundary. Whitefoot's FN-9/S12 publishes relations about
*results* and not about `&uniq`-written parameters, so for the write channel the
repair today is "branch on the written value". That is an honest ceiling, and
§12 Q3 opens write-postconditions as its own question — a hole is not justified
by the inconvenience of closing it.

**The exposure is reasoned, not measured.** No claim in the tree has this shape,
and the `s36`/`s37` skeptic programs the charter cites do not exist at `b1367c82`
(TERRAIN #17). §8's first batch builds the reconstruction and runs it under v0.39
before a line of spec text is written, because this is the design's only
*tightening* and both judges said the same thing about it.

### 2.6 Rows 11–12, 16 — the cases the gate exists for

`reject-clm1-claim-on-selected-payload.wf` (a matched `Ok` payload) and
`clm1-neg-user-result-claim-locality.wf` (a call result read into a Bool) are
unchanged and are now the canonical illustrations of the principle: their support
reads a value publisher 2 produced, publisher 2 published no contract about it,
and publisher 3 may not invent one. Their own `checker gap:` prose already says
so — *"a caller claim has no authority to substitute an unverified callee body for
a published boundary fact"*.

2745 survives verbatim: a verified `ensures` and its S12 publication never make
the returned value text-produced. The caller consumes publisher 2's publication
directly and may not restate or strengthen it.

Row 16 changes the *route* rather than the verdict. Today S10 imports two of
[SYS-8]'s relations (`start ≤ next`, `next ≤ end`) and 2990 leaves the other five
— `ReadBytes(next)` only for `next > start`, `next = start + required` for
`host_copy_bytes`, the `entries` relation, the buffer-disposition contracts —
outside the state as "retained checked-program facts". A writer who needs one of
them has **no route at all**: the value is boundary-produced so a claim is
refused, and the fact the specification itself fixes at 2537–2577 is not
published. §5.3 imports all of them. The same accreting-list defect as row 4, on
the world side.

### 2.7 Rows 13–15 — the 0097 shapes, decided by not being a question

The 0097 campaign generated programs with a claim standing after a boundary
selection and measured **63 rejections out of 63, all `NonLocalClaim`, with zero
true positives** (`docs/done/0097-differential-fuzz.md:205–216`). v0.38 refused
them by position; v0.39 admits them by comparing definition occurrences; here
they are not a question at all, because a selector contributes nothing to
admission. A claim inside an arm, a claim after an `Err` arm returns, and a claim
under three nested matches are admitted or refused by exactly what their support
reads, and then decided by facts.

The reason changes even where the verdict does not, and that is the whole of H1's
complaint. v0.39 admits `accept-clm1-local-claim-inside-selected-arm.wf` because
`position`'s reaching definition is the same *definition occurrence* on every
incoming edge — a property of analysis artifacts, implemented as
`DefinitionId { site: std::ptr::from_ref(stmt).addr() }`
(`compiler/src/semantic/claim_locality.rs:104–145`) with four soundness
preconditions nothing enforces (`docs/done/0102-clm1-narrow.md:379–387`). Here it
is admitted because `position`'s support reads a parameter and a literal — a
property of the program text. There is no identity to compare, so nothing can
silently equate two definitions when someone refactors an allocation.

Row 15's three-deep nesting needs no rule: a join of joins of text-produced
components is text-produced, by induction on the merge structure. There is no
frame stack, no acquire, no discharge, and no nesting depth anywhere in the
judgment.

### 2.8 Rows 17, 19, 20 — the residual claim, and what it looks like now

Row 17 is the `content` gap: the term vocabulary has no per-element term (2870(a)
excludes subscript suffixes) and the component tree has one conservative
all-elements component (3218), so a fact about `arr[0]`'s value has no publisher
but publisher 3. That is a permanent ceiling clause, not an omission, and the
writer is told the token.

Row 19 is a two-conjunct claim one conjunct of which publisher 1 derives. It is
refused — two publishers may not publish one premise — and the diagnostic names
the component ordinal and the derivation, so the edit is located. §6.3 derives
why this stays a refusal and what changes about it.

Row 20 (`claim ilt(cursor, 4_u64)` over a bare parameter, false for most `u64`) is
admitted by the gate and refused by review, exactly as 2746 already says
("a `Local` component is not thereby true"). No new rule: a rule here would be a
shape rule ("a claim whose predicate is a bare parameter comparison"), which is
the case-by-case exception the charter forbids. The case list renders
"reached by: parameter `cursor`", and `requires` is the named instrument.

---

## 3. The specification replacement text

Draft text for a work branch. Line numbers are `spec/kernel-spec.md` at
`b1367c82`. Everything not listed survives byte-identical except for the class
rename of §3.4.

### 3.0 What survives untouched

| rule | lines | why |
| --- | --- | --- |
| [ENT-2] | 2866–2905 | the vocabulary is not the problem; every corpus claim is DBM-expressible. Terms, difference bounds, disequalities, signed goals, the implicit facts, and term identity under-approximating aliasing while kills over-approximate it |
| [ENT-4] | 3011–3034 | three closure rules, the Boolean reconstruction, the contradiction rule |
| [ENT-5] support and kills | 3036–3067 | admission, kills and provenance must read one relation (§4.4); kill (b) is the model the new seed copies |
| [ENT-5] ordinary join | 3095–3103 | the merge operator is right; this design makes strictly more points use it |
| [ENT-6] obligations | 3130–3181 | four families, four goals, the closed constrained-subject list |
| [CLM-1] meaning, shape, five fields, D/S/F, `Contrib(P)` | 2696–2740 | untouched |
| [CLM-1] retention | 2748–2757 | one retained runtime check in every build mode, never elided |
| [CLM-3] | 2801–2829 | `deny_claims` unchanged in mechanism; `MayClaims` empties in strictly more closures |
| [PRV-1..3] | 3305–3412 | untouched. After §3.4 the two judgments agree that control choice contributes nothing and differ only in what seeds them |
| [DIAG-3] | 1960–1986 | the trap record still carries only the claim's IDENT; the five fields are compile-time review data and the trap is not a debugging channel |
| [PAR-3] | 2074–2079 | claims are not consumed as proof by the permission judgment |

### 3.1 [ENT-1] — the published ceiling and the direction guarantee

**Add after 2852:**

> The derivable set of this fragment is its **published ceiling**: [ENT-2]'s
> vocabulary, [ENT-3]'s sources — every operation row's image, every
> specification-fixed operation contract, and the axioms — [ENT-4]'s closure, and
> [ENT-5]'s flow, kills, merges and loop rule. The ceiling's complement is
> published as exactly four gap kinds — `vocabulary`, `image`, `flow` and
> `content` [CLM-1] — together with the `boundary` class, which is the callable
> boundary's subject matter and which no claim may bridge. An implementation
> derives exactly the ceiling: neither less, which is a defect, nor more, which
> would make [CLM-2]'s duplication verdict implementation-dependent and is
> forbidden by 2835.

**Replace 2853–2855 with:**

> Version monotonicity of fact-source and closure strengthening preserves every
> already-discharged operation, call goal, or selected-return relation, but claims
> deliberately sit at the proof frontier. Raising the published ceiling —
> widening the vocabulary, publishing an image a row withheld, strengthening a
> flow point's transfer, or admitting a contract as a source — is an
> amendment-level accepted-set change. Such an amendment may newly derive a claim
> predicate, its negation, or one contribution component; [CLM-2] must then reject
> that source, and the author removes the duplicated content. **Every source edit
> a ceiling raise forces removes claim content: it never adds an obligation, never
> requires a new claim, and never narrows an ordinary program's accepted set.**
> The rejection names the derivation the checker now has and the exact content to
> remove [DIAG-1], so the upgrade class is bounded and mechanically located. This
> is an explicit source-upgrade rule, never authority for compiler or optimizer
> elision.

2856's amendment classes survive, with "adding or removing a `BoundaryResult`
seed" reading "an admission seed" and covering §3.4's call-write seed.

*Why the guarantee is worded "removes claim content" and not "is a deletion".*
world-first proposed the stronger sentence while keeping the per-component
verdict that falsifies it: a strengthening that learns one conjunct of a
two-conjunct claim forces a predicate edit and a rewrite of the record's
`derivation:` and `conclusion:` lines, which is a restructuring. Judge-1 caught
that. The wording here is the strongest sentence that is true of both the
whole-occurrence and the component case.

### 3.2 [ENT-3] — eleven sources become three

**Replaced:** S5 (2962–2964), S6 (2965–2969), S7 (2970–2978), S9 (2979–2983).
**Generalised:** S10 (2984–2990). **Deleted:** 3009, the retired-S8 sentence,
whose content — a source "may return … the day a corpus program writes the shape"
— is the accreting-list method this design replaces. **Unchanged:** S1 (control
conditions), S3 (claims), S4 (`requires`), S11 (counted structure), S12 (verified
results).

**Draft, one source in place of four:**

> **[ENT-3.S5] (operation image).** Every operation-table row usable in a total,
> non-consuming position carries one image rule fixed by [OP-2] alongside its type
> and effect rows: the unique strongest finite set of [ENT-2] facts over the row's
> result terms entailed by that row's exact semantics from the closed state's
> facts over its operand terms.

> At every **value-commit event** whose value is a direct application of such a
> row and whose operands are each a term or a constant — an ordinary `let`
> initializer, a `set` or `replace` commit, a `give` delivery edge, or an FN-9
> selected-return binding — establish that row's image over the committed
> destination place, on the normal continuation, after that event's own [ENT-5]
> kill. The binding form is not read: one operation committed to one destination
> establishes one image however it is spelled. Allocation length, `len(P)`
> binding, slice creation, copy, conversion, and constant introduction are image
> rules of their rows and are not separate sources.
>
> A row whose exact image is not unique in the [ENT-2] vocabulary publishes the
> **empty image**, and that publication is itself a clause of the ceiling
> [ENT-1] with gap kind `image`. Uniqueness is a property of the row, not of the
> state: a row publishes one image or none.

Three things this clause does that the four it replaces could not.

- **It removes the `let`/`set` distinction by spelling.** world-first found this
  defect and filed it as an open question; here it is not a rule but an absence
  of one, because the closure is indexed by operation and destination.
- **It removes the delivery carrier grammar.** A `give` edge is a value-commit
  event, so `give 0_u64;` establishes what `let x = 0_u64;` establishes. §3.3
  deletes [ENT-5]'s delivery block and [GIVE-1]'s carrier judgment outright.
- **It makes an unpublished row impossible.** `imin` is listed in [ENT-6] 3227 as
  a total value operation for authority transfer and appears in no [ENT-3]
  source; under the closure that state does not exist, because a row without an
  image entry is a hole in a closed table.

**Draft generalisation of S10:**

> **[ENT-3.S10] (specification-fixed operation contracts).** For a `match_stmt` or
> `value_match` whose scrutinee is a call to a [SYS-2] operation, or a bare IDENT
> naming a `let` binding of that call's outcome type under the existing no-kill,
> no-`set` path discipline: at each arm's entry establish **every** relation
> [SYS-8] fixes for that arm's outcome, with each declared parameter read as its
> exact actual term or constant. A relation whose substitution leaves a
> non-[ENT-2] operand establishes nothing. These facts carry the trust class of a
> declared operation contract, never a writer statement.

This deletes 2990. Keeping two of [SYS-8]'s relations and withholding five is not
a principle; it is the accreting list on the world side.

### 3.3 [ENT-5] — one merge operator, delivery deleted, the loop rule published

**Deleted whole:** 3078–3093, the sixteen-line bounded-relation-delivery block,
with 3084's carrier exclusion list and 3085's "A `value_match` forms no delivery
image under any source shape". **Also deleted:** [GIVE-1] 286–293's additional
fact-carrier judgment; GIVE-1 keeps delivery completeness and exact mode/type
agreement (273–285) and loses the carrier grammar, because there is no carrier.

**Draft replacement, three sentences inside the joins paragraph at 3095:**

> The continuation of a `value_if` or `value_match` is an ordinary merge point.
> Its inputs are the states on every delivering `give` edge, each taken after that
> edge's value-commit image over the receiving binding [ENT-3.S5], then after that
> edge's scope-exit kills, and then closed [ENT-4]; the join is the join of 3097,
> unchanged. No separate delivery image, carrier admission, per-relation delivery
> rule, or term substitution exists.

*Why "establish the image over the receiving binding" and not "substitute the
delivered atom's term".* prover-first proposed the substitution and judge-2
refuted it: a literal is an [ENT-2] term, `0_u64` normalizes onto the
distinguished zero term Z (2870(f)), and 2905 puts Z in every implicit fact
`t − Z ≤ max(T)`, so substituting every occurrence of the delivered term rewrites
the arm's entire numeric frame into relations against the receiver. Establishing
an image over a destination has no such failure mode, because an image is a
finite set of facts about one result place.

**Replaced:** 3110–3128, restated so the ceiling is honest about them.

> The fact state at the head of a `loop_stmt` is the state before the loop minus
> every fact having a support member that a continuing kill event of that loop may
> kill. A counted `for_stmt` head is the closed post-capture state minus the same,
> with S11's structural body-entry bounds added on each true header edge. **A loop
> head is a merge point whose transfer this version publishes as this
> subtraction**; a relation the loop maintains and the subtraction removes is a
> ceiling clause with gap kind `flow` [ENT-1]. Replacing the subtraction by the
> [ENT-5] join over the preheader and back edges, computed to a verified
> post-fixed point, is a ceiling raise and an amendment-level accepted-set change.

That is deliberately weaker than what two of the three input designs wanted. §5.4
gives the reason and §9 gives the two programs that decided it.

**Add one sentence to the joins paragraph**, for the behaviour the compiler
already has and the specification never named (TERRAIN #18,
`compiler/src/semantic/entailment/state.rs:4045–4068`):

> When a joined relation's selected derivation depends on an S12 publication and
> the same relation is also derivable without one, the join retains both
> derivations. This adds no relation to the joined state and removes none; it
> exists so the [PRV-1] view partition and every non-explosive-ancestry query see
> a publication-free route where one exists.

[ENT-1] 2836 requires two conforming implementations to derive the same states; a
second implementation cannot satisfy that against undocumented behaviour.

### 3.4 [ENT-6] — claim authority becomes the admission state

**Replaced:** 3215–3248. **Repealed outright:** 3233–3239, the v0.39
control-dependence paragraph including its worked `3209_u64 % 64_u64` example, and
3241–3243, the `&uniq` carve-out. **Renamed throughout:** `Local` →
`TextProduced`, `BoundaryResult` → `BoundaryProduced`. The old names invited every
reader to reason about *position*; the new ones name the question the judgment
asks.

**Draft:**

> For [CLM-1] only, ENT-6 also computes one independent finite forward
> **admission** state over the structural normal-control graph. Admission is not
> an entailment fact, optimizer fact, callee summary, or [PRV-1] provenance pair,
> and it grants no operation authority. Each component is `TextProduced` or
> `BoundaryProduced(witness)`; component join retains `BoundaryProduced` when
> either input has it and retains the earliest witness in stable source order. The
> component tree is structural and finite, exactly as 3218–3220 fixes it.
>
> **Seeds.** Every typed literal, named const, const-generic constant, source
> parameter component, command-entry parameter component, compiler-owned counted
> capture, and otherwise untainted local initializer begins `TextProduced`. Every
> result component of every ordinary user call and every system call begins
> `BoundaryProduced`, unconditionally, as 3223–3225 fixes it — the seed inspects
> no callee body, argument, effect row, [PRV-1] class, [SYS-2] component class, or
> FN-9/S12 relation. **Every caller storage component whose facts [ENT-5]'s kill
> event (b) would kill at a call likewise begins `BoundaryProduced` at that call,
> with that call as witness**, under the same [EFF-2] boundary-projected `writes`
> occurrences and the same [OWN-7] resolved-place overlap. The two judgments
> therefore respond to one call write identically.

> **Transfer** is 3227–3230 unchanged: every total value operation joins the
> admission classes it reads into the results it produces; construction and
> projection are component-sensitive, so a boundary payload does not taint an
> independent local sibling; a place read obtains the reaching class of its
> selected storage component joined with any conservative element selector; an
> unconditional explicit write to one statically exact whole value or exact field
> component is a strong replacement and may clear an older witness when its
> right-hand side is `TextProduced`.
>
> **Control contributes nothing.** A condition, match scrutinee or tag, counted
> endpoint, or other selector chooses an edge and produces no value. It
> contributes no admission class to any component: not to a binder its arm
> introduces, not to a value a `value_if` or `value_match` delivers along it, not
> to a component at a reconvergence, loop head or loop exit, and not to storage
> the selected edge wrote. Two reaching definitions are never compared for
> identity; a merge takes the componentwise lattice join and nothing else.
> A claim is a statement about values, and only what produced a value bears on
> whether this function may state a lemma about it. [PRV-1] 3327–3330 excludes
> control dependence for the same reason, and the two judgments now agree.
>
> **Witness and ordering.** One boundary witness contains the introducing call's
> NodePath and kind, plus the user callee's source declaration origin and source
> name or the system operation's `system_declaration_ordinal` and spelling; the
> least call NodePath wins, with boundary kind and stable callee identity as a
> deterministic tie-break at one path. The admission state is computed once before
> S3, U, B, `Eligible`, or any `Full-minus` mask and is reused unchanged by every
> claim component query. For one component, CLM-1 queries exactly [ENT-5]'s
> ordinary S-derived relation or opaque-goal support, including each root and
> holder; canonical normalization may add a fact identity but never subtract
> support read by the retained S expression.

Net: 34 lines to about 24, and the two hardest paragraphs in §18 replaced by one
lattice with three seeds and one join. Nothing in the judgment names an analysis
artifact, so `DefinitionId`, `DefinitionKind`, `ControlFrame` and
`ControlAuthority` have no referent in the specification and are deleted from the
compiler (§4.3).

### 3.5 [CLM-1] — the gate, its limit, and the gap token

**Replaced:** 2742–2746.

> At the claim point, CLM-1 queries every component's ordinary S-derived support
> [ENT-5] in component-ordinal order against [ENT-6]'s frozen admission state.
> Every runtime value component and holder read by that support must be
> `TextProduced`. If any support member is `BoundaryProduced`, this claim's gap is
> `boundary`; the whole claim rejects under CLM-1 using [DIAG-1]'s least component,
> earliest boundary witness, and the support carrier whose canonical source
> spelling [FORM-2] occurs first in the predicate's own source order among those
> observing that witness; and no S3 source, lifecycle query, `Eligible` member,
> counterfactual run, ClaimLedger record, or lowering authority is formed for that
> occurrence.
>
> A verified `ensures` and its S12 publication never make the returned value
> `TextProduced`: the caller consumes the verified relation directly and cannot
> restate or strengthen it with a claim. This admission is independent of truth
> and of [PRV-1] provenance; a `TextProduced` component is not thereby true or
> internal, and a PRV-internal call result is still `BoundaryProduced`.

**Add after 2720** — the sentence all three input designs needed and only one
wrote, and which judge-2 listed as undone by all three:

> A claim's five fields must be settlable against this function's source text, the
> semantics of this specification, the facts valid at the claim point, and the
> machine-verified or specification-fixed callable-boundary facts named above, and
> against nothing else. A `derivation:` line may cite only such published
> premises; a step requiring an unpublished premise makes the record invalid and
> the review refuses it. **The admission judgment below is the mechanical
> necessary condition for that settlability and is not a truth screen.** A
> predicate whose support reads a value a callable boundary produced cannot be
> settled from that evidence whatever its truth, and is refused. A predicate whose
> support reads only values this function's own text produced is settlable, and
> whether it is *settled* — whether the derivation covers every case the text
> admits — is the review's decision, for which [DIAG-2] retains the reaching
> definitions of each support component.

**Amend the five-field record at 2707–2721:**

> The `checker gap:` value begins with one of the fixed tokens `vocabulary`,
> `image`, `flow`, or `content`, followed by one ASCII space and nonempty prose.
> Each token names one published clause of the entailment ceiling [ENT-1]:
> `vocabulary`, the fact is not a difference bound or disequality over two terms;
> `image`, the operation row's exact image is not unique in that vocabulary and
> the row publishes the empty image; `flow`, no single reaching edge carries the
> fact and the flow point's published transfer does not derive it; `content`, the
> fact is about an element of an array, slice or buffer. The token `boundary` is
> not admissible: a claim never bridges a callable-boundary or
> specification-fixed-contract gap, and a claim naming it rejects under CLM-1 with
> that rule's fixed restructuring. The token is checked; the prose is retained
> review data and is not.

Four spec lines and one lexical check. What they buy:

- the writer states *which published limit* they are standing above, which is a
  claim a reviewer can check, in place of free prose 35 of whose instances are
  byte-identical today;
- every future ceiling amendment is a mechanical migration — grep the token, read
  the prose — and can be **announced** before it ships;
- H5's uniform teaching channel exists, because every claim-family diagnostic can
  now name a token and its clause;
- the review question becomes *"is this token right?"* instead of *"is this prose
  true?"*.

The tokens name **kinds, not instances**. A per-instance ceiling identifier
(`ENT-CEIL-REM-RANGE`) was the alternative; it needs a new identifier per
amendment and a registry to keep them live, and it buys only a finer grep.

### 3.6 [CLM-2] — one non-duplication rule, residuality unchanged in unit

**Replace 2764–2766 and 2775–2776 with one rule:**

> Otherwise, query both signs of every image in the ordered inventory. Deriving
> any negative sign rejects c as **refuted**. Deriving the positive sign of the
> whole predicate, or the positive sign of any one contribution component, rejects
> c as a **duplicate publication**: the entailment already publishes that content
> at this point, and one premise has one publisher. The rejection names whether
> the whole predicate or one component ordinal was derived, and the ordered fact
> sources of the derivation that produced it. Thus `claim True()` is a duplicate
> publication and `claim False()` is refuted on every reachable path.

The four verdicts of v0.39 — vacuous, redundant, refuted, overlapping — become
three: **vacuous** (contradiction, tested first, unchanged at 2761), **refuted**,
and **duplicate publication**, which absorbs redundant and overlapping into one
rule with one reason and one shape of fix. Residuality (2782–2799) is unchanged,
including its per-component `Full-minus(c,a)` masks: S3 establishes components
(2951), so the component is the unit of publication throughout, and the same unit
carries non-duplication, residuality, lifecycle and diagnostics. §6.2 derives
that choice and §6.4 records what it costs.

### 3.7 [ENT-6] 3169–3173 — the route menu, stated as total

**Replace 3169:**

> Exactly one of three routes discharges any unproved family, selected by which
> publisher owns the missing premise. When the residual reads only values this
> function's own text produced, the route is one dominating branch establishing
> the canonical goal, or one CLM-2-admissible residual claim. When it reads a
> value a callable boundary produced, the route is an exact verified `ensures` on
> the callee and its S12 publication, a specification-fixed [SYS-8] fact, or an
> ordinary branch on the returned value; a claim is not available and [CLM-1]
> refuses it. When the constrained subject is unconditionally external, the route
> is the dominating value branch of [PRV-2] and [PRV-3] alone. **At least one
> route is always open, and the diagnostic names it** [DIAG-1].

3170–3173 survive. The last sentence is normative on purpose: H2 is precisely the
failure of that promise, and a rule the specification does not state is a rule
nothing checks. §8's first falsifier makes it a conformance obligation.

### 3.8 [DIAG-1] and [DIAG-2] — one payload, one cause, one fix, one case list

**Replace 1855–1859:**

> Every claim rejection carries the same payload: the claim name and `claim_stmt`
> NodePath, the exact predicate, one `cause` drawn from the closed set
> { `predicate-shape`, `formation`, `duplicate-name`, `non-local`, `refuted`,
> `duplicate-publication`, `vacuous`, `non-residual`, `inadmissible-gap-token` },
> the deterministic concrete instance, the component ordinal where the cause
> selects one, the gap token where the claim carries one, and one mechanical fix
> fixed by the cause.

| cause | mechanical fix |
| --- | --- |
| `non-local`, user-call result witness | *publish the required cross-function relation as an exact verified ensures clause on the callee and remove this caller claim* (1859, kept) |
| `non-local`, system-call result witness | *use the system operation's specified fact or typed outcome, or branch on the returned value; do not claim an unstated system-result property* (1859, kept) |
| `non-local`, call-written-storage witness | *publish the required relation as an exact verified ensures clause on the callee that writes this storage, or branch on the written value* (new) |
| `duplicate-publication`, whole predicate | *the entailment derives this predicate here from `<sources>`; delete this claim statement* (new) |
| `duplicate-publication`, one component | *the entailment derives component `<ordinal>` here from `<sources>`; remove it from the predicate and its lines from the record* (new) |
| `non-residual` | *no admission root consumes this content; delete this claim statement* (new) |
| `refuted` | *the entailment derives the negation here from `<sources>`; the reviewed derivation is wrong* (new) |
| `vacuous` | *this statement is not reachable with a consistent state* (new) |
| `inadmissible-gap-token` | *`boundary` is not a claim gap; see the two restructurings above* (new) |
| shape and formation causes | existing per-rule text |

Today the whole redundancy family carries **no mechanical fix at all**
(1855–1856), and it is the family every ceiling raise will grow. That is H5's and
H4's shared defect and it is one table.

**Add to [DIAG-2] 1884–1886, the reviewer's case list:**

> For every accepted claim occurrence, the checked program retains, per
> contribution component and per support component, the ordered NodePaths of the
> value-commit events whose definitions reach that support at the claim point,
> each marked *forward* or *recurrent* when it reaches through a back edge. This
> is published review data and never an acceptance criterion: it is the evidence
> the review duty at 2719 requires, and a defect in it is a diagnostic defect.

Two properties make this safe where v0.39's definition identity was not: its
identities are NodePaths, which [DIAG-1] 1858 already requires to be publishable
and stable, and nothing reads it to decide a verdict, so 0102's four unenforced
soundness preconditions have no analogue.

**Add the review note.** An accepted claim whose predicate the certain state
neither derives nor refutes, and one of whose support components has more than
one reaching value-commit event, emits one non-blocking review note naming the
component, the reaching definitions the certain state does not exclude, and the
edge each arrives on:

```text
review note [CLM-1] `reviewed_written`: the entailment does not exclude
  cursor = 9_u64 here. Reaching definitions of `cursor`: `let cursor = 9_u64;`
  at f.wf:3:7 (forward, else edge of the match at f.wf:4:3) and
  `set cursor = 0_u64;` at f.wf:5:5 (forward, then edge). The reviewed
  derivation must cover every listed case.
```

It is a note, not a verdict, and the distinction is load-bearing: as a verdict
the same computation rejects claims that are true and text-reviewable (§9, A2 and
A3). As review evidence it is the best writer-facing text proposed anywhere in
this batch, and it costs no new analysis state — the case list is already
retained and the certain state is already computed.

### 3.9 Net specification shape

| rule | replaced | added | direction |
| --- | --- | --- | --- |
| [ENT-1] ceiling + direction guarantee | 3 | ~18 | +15 |
| [ENT-3] S5/S6/S7/S9 → one image source | 22 | ~14 | −8 |
| [ENT-3] S10 generalised; 3009 deleted | 8 | ~6 | −2 |
| [ENT-5] delivery block + [GIVE-1] carrier | 24 | ~3 | −21 |
| [ENT-5] loop rule restated; join sentence | 19 | ~13 | −6 |
| [ENT-6] claim authority → admission | 34 | ~24 | −10 |
| [CLM-1] gate, settlability, gap token | 5 | ~22 | +17 |
| [CLM-2] four verdicts → three | 8 | ~7 | −1 |
| [ENT-6] route menu | 1 | ~8 | +7 |
| [DIAG-1] payload + fix table; [DIAG-2] case list, note | 5 | ~26 | +21 |
| **net** | **129** | **~141** | **≈ +12 lines** |

Twelve lines longer, with **three concepts removed from the language** —
definition-occurrence identity, the delivery carrier grammar, and the
`redundant`/`overlapping` verdict split — and the operation table gaining an image
column, which is the real specification work and is counted in §8 rather than
here.

---

## 4. The judgment architecture

### 4.1 What is computed, and by whom

One forward walk over the [FN-1] structural normal-control graph produces two
products that share a graph, a kill relation and a merge operator and are
otherwise independent.

| product | lattice | join | consumed by |
| --- | --- | --- | --- |
| **certain state** | DBM bounds + disequalities + signed opaque goals | weakest bound / intersection held by all (3097) | [ENT-6] obligations, [FN-8], [FN-9], [CLM-2] |
| **admission state** | two points, `TextProduced` ⊑ `BoundaryProduced(witness)`, per component | `BoundaryProduced` if either; earliest witness | [CLM-1] only |

There is **no third state.** prover-first proposed one — a per-pair strongest
bound tagged with a merge input edge — to recover the laundering family
mechanically, and §9 gives the two programs that refuted it. The case list of
§3.8 is retained *data*, not a lattice, and no judgment reads it.

Ordering is unchanged and non-circular: the admission state reads no fact, so it
is computed once before S3, `Eligible` and every `Full-minus` mask (3247), and
CLM-2's counterfactual reruns cannot change what CLM-1 admitted. A locality
judgment that depended on facts would make residuality circular (TERRAIN #5).

### 4.2 What identity the admission state tracks — and why it cannot rot

Two, both already published and already ordered:

- the **component path** — `Field | EnumTag | EnumPayload | Element | Length |
  Deref` steps over a structural tree derived from types (3218);
- the **boundary witness** — the call's NodePath and kind plus the callee's stable
  source identity (3245), with the least-NodePath tie-break (3246).

It tracks **no definition identity at all**, and that is the direct answer to H1's
real complaint. H1 is not that v0.39's rule is wrong; it is that the rule's
criterion is an equality test between analysis artifacts. The compiler answers
"are these two reaching definitions the same occurrence?" with
`DefinitionId { site: std::ptr::from_ref(stmt).addr(), kind }`
(`claim_locality.rs:104–145`), sound only under four preconditions that neither
the type system nor any test enforces, whose failure mode 0102 states plainly:
"a refactor that cached or compared identities across allocations could silently
equate two distinct definitions and drop a selector's witness"
(`docs/done/0102-clm1-narrow.md:379–387`). All four vanish, because the question
they answer is not asked.

The general property, and the sentence that belongs in the specification rather
than only in a design document:

> **A judgment whose criterion is a lattice point can be wrong only by being
> imprecise; a judgment whose criterion is an artifact identity can be wrong by
> being unsound.**

v0.39 chose the second. This design chooses the first, and states the imprecision
it buys: a call that writes through `&uniq` taints every component its effect row
projects onto, whether or not that particular call actually wrote them.

### 4.3 Termination, determinism, cost

**Loops.** The admission state is the least fixed point of the same componentwise
join. It terminates because the lattice is two-point, the component tree of one
function body is finite (3218), and every transfer is monotone: each component
can rise at most once. This is the same kind of fixed point [ENT-6] 3210–3211
already runs for [PRV-1] dependency, so no new mechanism class enters the
language, and the current implementation already runs an inner fixed point per
loop (`claim_locality.rs:758`).

**Determinism.** [ENT-1] 2836 requires two conforming implementations to derive
the same claim-authority component tree, reaching state, support carrier and
witness. Today that rests on both implementations agreeing about
definition-occurrence identity, which neither the spec nor a test pins. Here it
follows from the lattice and the published witness order, and §3.5 pins the
carrier tie-break that 2744 and 1857 today word differently and that
`flow.rs:6816–6832` resolves by whatever `claim_component_supports` yields first.

**Cost.** `ControlAuthority::acquired` scans the held frame vector once per frame
an edge carries, "so its cost grows with the product of the two", and 0102 took
no measurement (`docs/done/0102-clm1-narrow.md:363–375`). That term is deleted
outright. The per-edge work becomes one componentwise join.

### 4.4 One relation, three consumers

The call-write seed is defined as *exactly* what [ENT-5]'s kill (b) kills, and
that is the design decision that keeps the area from re-growing a private notion
of what a call touched. The compiler already carries the projection in the shape
needed: `KillEvent::Write { place, element, source }`
(`compiler/src/semantic/entailment/flow.rs:62–70`), whose `element` flag is what
keeps P16 alive. Kills (3066), [PRV-1] write components (3207) and admission
become three consumers of one relation. If it is wrong, all three are wrong
together and one fix repairs all three — the opposite of today, where two see the
`&uniq` write and one looks away. §8's falsifier F4 makes the shared blast radius
the test.

### 4.5 What leaves the compiler, and what arrives

**Deleted.** `DefinitionId`, `DefinitionKind` (`claim_locality.rs:93–145`);
`ControlFrame`, `ControlAuthority` with `with_added`/`remove`/`acquired`/`join`
(`:147–228`); `AuthorityState::control` (`:637–639`); the definition-comparing
half of `AuthorityValue::merge` (`:584–630`), which becomes a plain lattice join;
the three unconditional selector stamps — match binder (`:1105`), delivered value
(`:1165–1166`), counted binder (`:1300`); the three frame-discharge sites
(`:1158–1160`, `:1187–1194`, `:1266–1272`). In `flow.rs`: `eligible_delivery_terms`
(`:6279`), `delivery_edge_state` (`:6343`), `establish_delivery_join_view`
(`:6473`) — a delivery edge is a value-commit event and a merge input.

**Added.** The call-write seed at the call sites that already evaluate arguments
(`claim_locality.rs:1370–1394`), reading `KillEvent::Write`'s existing projection;
a new `semantic/entailment/image.rs` holding one image rule per operation row,
table-driven; the case-list retention in the checked-program metadata; the gap
token's lexical check in claim formation.

**Kept unchanged.** The postcondition-candidate second pass in `join_at`
(`state.rs:4045–4068`), which now acquires the specification sentence of §3.3;
`staged_permission.rs`, since [PAR-3] does not consume claims as proof
(`:818–826`, `:608`, `:1136`, `:1669`).

### 4.6 The invariants this design must not break

- **Six terminal roots, all partial-operation obligations** (2789). Untouched.
  This design gives claims no new job; it takes jobs away.
- **`TextProduced` is not `true` and is not `internal`** (2746, 2757, PRV-1 3310).
  Preserved and reinforced by the rename. CLM-1, CLM-2 and PRV-2/3 can still each
  refuse one occurrence for three unrelated reasons.
- **Admission is computed once, before S3 and every mask** (3247). Preserved; it
  reads no fact.
- **Admission reads exactly the support the kills read** (3048, 3248). Preserved
  verbatim; `S`, not `D` and not `F`, remains the image whose support is queried.
- **`Full-minus` produces exactly the same PRV failure set as `Full`**
  (3050–3054). Preserved: the mask still suppresses only S3 events, and neither
  the image closure nor the admission state depends on S3.
- **[CLM-3] counts claims and nothing else does** (2815). Mechanism untouched;
  `DirectClaims`'s `(instance, NodePath, name)` identity (2809) unchanged; no
  second claim-like construct is introduced. Its *meaning* shifts in the intended
  direction — a `MayClaims`-empty subtree becomes reachable for the third of the
  corpus that two missing images were blocking from it.

---

## 5. The prover ceiling

### 5.1 The charter's hardest question, answered once

> *Where does proving power live, and what is its general shape?*

It lives in publisher 1, and its shape is **a closure indexed by two closed sets
the language already owns: the operation table and the control graph.** A writer
can only write operations from the table and control from the grammar. Therefore:

- adding a language operation adds an image rule to its row **in the same
  change**, or publishes the empty image and thereby names a ceiling clause.
  There is no state in which an operation exists and its proof behaviour has not
  been decided. `imin` is exactly that state today.
- adding a control construct adds its merge points, and a merge point is a merge
  point. There is no state in which a construct delivers values and its fact
  behaviour has not been decided. `value_match` is exactly that state today
  (3085).
- adding a system operation adds its contract to the source list. Five of
  [SYS-8]'s relations are exactly that state today (2990).

**Three constructs are in that state at `b1367c82`, and each is one of the
charter's holes.** That is the argument that the list-shaped design fails
structurally rather than by oversight, and it is a structural argument rather
than a corpus count.

### 5.2 The complement, published

A closure has a complement, and the complement is what a claim is for. Four
kinds, each a stated clause and each a token the `checker gap:` field must name.

| token | the gap | why it is out, and for how long |
| --- | --- | --- |
| `vocabulary` | the fact is not a difference bound or a disequality over two terms: a three-term relation, a product or quotient of two non-constant terms, a congruence, anything nonlinear | permanent while the fact language is [ENT-2]'s. Widening it is a different design with a different cost curve (octagons, polyhedra), and [ENT-1] 2835–2836 is why the language must be fixed rather than best-effort |
| `image` | the row's exact image is not unique in that vocabulary — two non-constant `+`/`-`/`*` operands (3146 already says so), `bxor`, a normalization with alternative positive clauses such as signed division/remainder (2739 already says so) | permanent per row while the row's image is non-unique. This is the one token that can be retired for a specific row by amendment, and the writer should know it |
| `flow` | no single reaching edge carries the fact and the flow point's published transfer does not derive it: a correlation the convex join loses, a relation a loop head's subtraction removes | permanent for the convex-join case while the domain has no disjunction; **retirable for the loop case** by the amendment §3.3 names |
| `content` | the fact is about an element of an array, slice or buffer | permanent while the element component is one conservative all-elements component (3218) and terms exclude subscript suffixes (2870(a)) |

And the one that is not a gap: `boundary`, the callable boundary's subject
matter, whose two repairs are already written at [DIAG-1] 1859.

### 5.3 What the ceiling gains at v0.40, concretely

The vocabulary is unchanged; the redesign is entirely on the establishment side.
Rows whose image is nonempty and which have no source today include:

- **`%` and `/`.** Unsigned `a % d` with `d ≥ 1` derivable: `Z ≤ r`, `r ≤ d − 1`,
  `r ≤ a`. Signed `a % c` for constant `c`: `r ≤ |c| − 1`, `Z − r ≤ |c| − 1`.
  Unsigned `a / d` with `d ≥ 1`: `q ≤ a`. All C0-expressible and unique. 46 of the
  corpus's 135 claims name the missing one.
- **`imin` / `imax`.** `imin(a,b) ≤ a`, `imin(a,b) ≤ b`, dual for `imax`. Five
  corpus claims. Listed at 3227 for authority transfer and in no source today.
- **The ground case of every arithmetic row**, as the degenerate case where the
  operand bounds pin the operands. There is no constant folder; there is one
  image rule per row.
- **Bit rows that were never reached for.** `ipopcount(x) ≤ width(T)`,
  `iclz(x) ≤ width(T)`, `ictz(x) ≤ width(T)` are C0-expressible and free.
- **All seven [SYS-8] outcome contracts** rather than two.

Rows that publish the **empty image**, explicitly, as ceiling clauses: every
float row, every `f`-domain compare, `bxor`, signed division and remainder's
non-unique normalizations, and any `+`/`-`/`*` with two non-constant operands.

The magnitude of the image column is 60–100 rows by the shape of [OP-6]'s `cvt`
pairs, [OP-7]'s domain prefixes and [OP-8]'s bit, shift, rotate, saturating and
min/max rows. Most are one line; many are the explicit empty image. **This was
not enumerated row by row by any of the three designs, and enumerating it is
batch B1's first task and its own falsifier** (§8, F3) — an enumeration that
must decide each row's *direction* as well as its image, per the erratum below.

> **Erratum (B0 census).** The list above, and §5.1's shape argument, take the
> image column to be **forward**: operand facts in, result facts out. Every
> example here is forward, and §3.2's draft [ENT-3.S5] is worded that way too
> ("entailed by that row's exact semantics from the closed state's facts over its
> operand terms"). `CENSUS.md` §2.3 and §2.4 measure what that reaches. It
> reaches 108 of the tree's 114 gap-stating claims — and **four real-program
> claims it closes none of**: `percent_decode.wf:28`, `percent_decode.wf:31`,
> `wfgrep.wf:434` and `wfgrep.wf:553` (rows T1–T4 of `CENSUS.md` §2.4). Their
> missing fact runs **backward across a `±wrap` row**: from a fact the state
> already carries about the row's *result* to a fact about its *operands*, across
> `r = a ± b` with `r`, `a` and `b` distinct. Neither a DBM nor an octagon can
> hold that equality — but nothing has to hold it. Given the no-wrap side
> condition the state already carries (`b <= a` for a subtraction), **one backward
> row rule, entirely inside the existing [ENT-2] vocabulary, closes all four**:
> publish `a - b >= k` when the state carries `r >= k`, and `a != b` when it
> carries `r != 0`. Two corrections follow, and both are applied below. First,
> whether a row's image may run backwards is not a detail of B1's enumeration but
> a decision it must take, so it is added to B1's scope (§8). Second, **F3 audits
> the column for uniqueness and does not ask about direction at all** (§8.1),
> which is amended there. What does not change: these four are a *publisher* gap,
> not a vocabulary gap, so the ceiling argument of §5.1–§5.2 stands and the true
> residue remains the two claims `CENSUS.md` §2.4 names. This backward rule is
> the only fact-language work the whole corpus asks for, and three real programs
> ask for it.

### 5.4 What the ceiling deliberately does *not* gain, and why

**Loop induction stays out of v0.40**, and the loop head is published as the
subtraction it is (§3.3). Two of the three input designs wanted it and one wrote
the rule; both judges independently broke that rule with programs in which the
declared head state asserts a false bound and a `SubscriptBounds` obligation
discharges against it — an out-of-bounds read in a program with no `claim` in it
(§9, A1). The error was one clause: widening once and stopping, with a stated
termination argument ("stable across two consecutive iterates") that the
counterexamples violate.

The repair is standard — widen at every unstable iterate and iterate until
`B(X) ⊑ X` is *verified* — but it is not a one-line edit to this design. It
introduces the first fixed point over `FactState` and therefore the language's
first widening operator (TERRAIN #4: "the project has never paid for, and
therefore never had to specify, a widening operator"); it makes the cost
unbounded by any published pass count; and it needs a soundness gate no proposed
falsifier supplied — prover-first's F4 only checks that the new head state is *at
least as strong* as the subtraction, which the broken rule satisfies while being
wrong.

So the sequencing is a soundness decision, not a churn decision. The ceiling is
honest about it: the loop head's transfer is published, the 39 loop-induction
claims carry the token `flow`, and §8's batch B6 raises that clause with three
gates — the post-fixed-point check as a compiler assertion on every loop, a
differential corpus run against the subtraction, and a measured cost on the two
largest programs. That is the ceiling machinery doing exactly the job it was
designed for, on its own first customer.

### 5.5 The honest weakness of "every row publishes an image"

Judge-2 stated it precisely and it must not be buried: **the column can be total
over rows and still weak on a row, and nothing detects that.** Totality is
mechanically checkable — a gate test that every row in the operation table has an
image entry — but *strength* ("the unique strongest set of [ENT-2] facts the row's
semantics entail") is a per-row proof obligation discharged by review, exactly as
the `because` prose is.

Three things reduce the exposure and none eliminates it:

1. one conformance case per nonempty image row, asserting the facts it publishes;
2. the empty image being an explicit, reviewable publication rather than an
   absence, so "no image" is a decision someone signed;
3. the gap token, which turns "the checker is weak here" from an unspoken
   condition into a citation a reviewer can dispute.

A row whose published image is weaker than its semantics entail is a *defect*
under [ENT-1]'s "derives exactly the ceiling: neither less, which is a defect",
and it shows up as a writer's claim carrying the token `image` where the token is
wrong. That is a better failure mode than today's — where the same weakness is
indistinguishable from a deliberate limit — but it is not a proof.

### 5.6 What each publisher owns, in one sentence for the writer

- **The entailment:** *the checker knows the range and relations of every value
  your function computes, as far as two-term difference bounds can express them,
  through every branch, delivery and merge, but not across a loop back edge.*
- **The callable boundary:** *the checker knows about a callee's result and about
  storage a callee wrote exactly what that callee's verified `ensures` or the
  specification's own operation contract states, and nothing else.*
- **The reviewed claim:** *you may state a lemma about values your own function
  produced, above the published ceiling, naming which of the four ceiling clauses
  you are standing over; the check is retained and never removed.*

That is a promise a writer can hold in their head, and it is what
`docs/done/0098-blind-writer.md:40` found in practice under a *weaker* ceiling:
"Zero `claim` statements in 1,694 lines. Every subscript, every `%` and `/`, every
system range call was discharged by ordinary `if` branches and `len()` rebinding.
The proof obligations — the part of this language everyone expects to be the wall
— were not the wall." Under today's ceiling that outcome required the writer to
route around `%`'s missing range with branches. Under this one it is the default.

---

## 6. Non-duplication and residuality

### 6.1 The verdict, derived

Three semantics for a checker-provable claim are consistent with T3, because T3's
derivation needs a claim to be *true*, not *necessary*: hard rejection, a note, or
acceptance-as-documentation. The file name
`tests/conformance/cases/clm2-pos-redundant-claim-advisory.wf` is the fossil of a
fourth answer that once shipped (`docs/roadmap.md` PROOF-8 records "five
non-rejecting redundancy advisories"), and its manifest today expects a CLM-2
reject.

Under premise ownership the question is not a preference. **Two publishers may
not publish one premise**, and the three candidates are ranked by three pieces of
project law:

- **T3 / P0.** A retained claim is one runtime check retained in every build mode,
  never elided, evaluated once at every dynamic reach (2748). A check no admission
  root needs taxes *correct* programs and buys nothing. Acceptance-as-documentation
  is refuted by T3's own premise, not by taste.
- **W3.** Accepted claims are never removed, so the compiler may not silently drop
  the check.
- **R4.** Unrepresentable > check-time rejection with rule-citing diagnostics >
  runtime trap. A note is a rejection the writer may ignore, which under an
  AI-writer model is silence with extra steps.

The compiler may not drop the check, may not keep it silently, and may not merely
mention it. **It must refuse the source.** There is one more argument, found by
world-first and worth keeping because it is architectural rather than
axiological: an accepted occurrence contributes S3 (2950–2951), which changes the
fact state, which changes **every other claim's** residuality — `Full-minus(c,a)`
re-runs the whole-program analysis "with every other Eligible S3 source …
unchanged" (2784). Admitting an unnecessary occurrence perturbs the counterfactual
every other occurrence is measured against, and 2796's "one simultaneous
classification over fixed `Eligible`" stops being well-founded. Acceptance of a
duplicate is not neutral; it destabilises the judgment that decides the others.

### 6.2 Why the unit is the component, and what that costs

writer-first proposed a real simplification: prove that `redundant` is a strict
special case of `non-residual` and delete the verdict, then drop per-component
residuality so every forced edit is a whole-statement deletion. The proof is the
most substantive reasoning in the batch and is worth stating:

> If every component of `Contrib(P)` is derivable in the pre-S3 state at the claim
> point, the occurrence is non-residual. S3 establishes exactly those components
> (2951), each carrying the ordinary support of its S-derived relation or goal
> (3048). `Full-minus(c)` withholds exactly those source events and changes no
> evaluation, effect, ownership, cleanup, scope, join, loop or runtime statement
> (3050–3051). By hypothesis each component is derivable from unmasked sources at
> that point; support is a property of the fact and not of its derivation
> (3036, 3040), so the independently derived fact dies on identical edges (3066)
> and survives identical joins (3097). Every queried state is equal in Full and
> `Full-minus(c)`, so no terminal root can differ. ∎

It has **one unclosed step**, which judge-2 found: it needs "no downstream
judgment distinguishes an established fact from a derived one", and [ENT-4] 3023
("Derivability never decomposes a merely derived parent") is evidence the
specification distinguishes them somewhere. §8's batch B3 closes it against
3014–3023's Boolean reconstruction or keeps both tests; either way the writer's
experience is identical, because the diagnostic and the fix are the same.

This design keeps the **component** as the unit throughout, for a reason that is
not a preference either: S3 establishes components, so the component is the unit
of publication, and one unit should carry non-duplication, residuality, lifecycle
and diagnostics alike. Dropping per-component residuality would be a real
performance win — one whole-program re-analysis per component per claim
disappears — but nobody has measured it, and "fix measured performance problems
instead of designing for imagined scale" cuts against buying it with a
non-uniformity. §12 Q5 puts it to the owner.

### 6.3 What keeps the writer stable as the prover grows

Nothing keeps it *stable*, and promising stability would be dishonest: [ENT-1]
2853 deliberately places claims at the proof frontier and withdraws version
monotonicity from them in the same sentence that grants it to everything else.
The history is consistent — SHA-256 moved to claim-independent obligations "after
deleting four claims" when S11 shipped; a later migration "replaces eleven DEFLATE
claims with value branches"; DEFLATE moved 16 claims to 12 (`docs/roadmap.md`
PROOF-8). Every prover improvement in this project has been accompanied by a
corpus edit deleting claims, and this design's own amendment retires roughly half
the corpus's claims.

What can be guaranteed is four things, and together they are the answer:

1. **Within a version there is no churn at all**, and there never was. [ENT-1]
   2835 forbids any implementation from strengthening the derivable set and 2836
   requires two conforming implementations to derive identical states. Today's
   instability is *entirely* a specification-version phenomenon that 2766
   describes as if it were implementation drift ("Checker strengthening"). Naming
   the ceiling (§3.1) fixes that confusion, and it is half the complaint.
2. **The direction is guaranteed** (§3.1): every forced edit removes claim
   content. Never a new obligation, never a new claim, never a narrowed ordinary
   accepted set, and each removal deletes one retained runtime check. The writer's
   experience is not stable; it is monotonically improving, and the language
   should say which.
3. **The edit is located and mechanical** (§3.8): the rejection names whether the
   whole predicate or which component was derived, renders the derivation the
   checker now has, and gives one fix. Today this family carries **no mechanical
   fix at all** (1855–1856), which is the actual defect behind H4 — the writer is
   told their program is now wrong and not told what to do.
4. **The migration is announceable** (§3.5): a ceiling raise names the clause it
   closes, and every claim standing over that clause carries its token. Raising
   the `%` row's image retires exactly the `image`-token claims whose prose names
   remainder. That is a grep before the amendment ships, not archaeology after.

The writer's own instrument is the token. Before this design a writer had no way
to know whether their claim stood above a permanent limit or a temporary one.
After it, `vocabulary` and `content` are permanent by construction, `flow` is
permanent for the convex-join case and retirable for the loop case, and `image` is
the token that says *this row could publish more one day*. A writer who wants
stability writes claims whose token is not `image`, and knows why.

### 6.4 What this design's own ceiling raise costs, honestly

By its own rule this design is an amendment that raises the ceiling a long way,
and it therefore retires a large fraction of every claim in the tree: the 46
remainder claims, three ground-remainder claims, three payload-construction
claims, the delivered-value and arm-written conformance cases, and both members
of the differential-fuzz pair. The 39 loop-induction claims survive, with the
token `flow`, until batch B6.

That churn is not a cost the design pays reluctantly. Every claim it retires was a
retained runtime check that correct programs were paying, on every reach, for a
fact the checker should have known. It is the deliverable.

---

## 7. Conformance migration

Every item below is conformance evidence under repository rule 4, so the merge
recording it names the exact added, modified, deleted or renamed content and its
before/after boundary in `governance/APPROVALS.md`. Paths are
`tests/conformance/cases/` unless named. Batch letters refer to §8.

### 7.1 Verdicts that move

| case | v0.39 | here | batch |
| --- | --- | --- | --- |
| `accept-clm1-local-claim-after-boundary-exit.wf` | accept | **reject CLM-2 duplicate-publication**, + a new accept sibling with the claim deleted | B1 |
| `accept-clm1-local-claim-after-boundary-join.wf` | accept | same | B1 |
| `accept-clm1-local-claim-inside-selected-arm.wf` | accept | same (`cursor % 4_u64` is now derivable) | B1 |
| `reject-clm1-claim-on-delivered-selection.wf` | reject CLM-1 | **reject CLM-2 duplicate-publication**, renamed `clm2-neg-…`, + accept sibling with the claim deleted | B2 |
| `reject-clm1-claim-on-storage-written-under-selection.wf` | reject CLM-1 | **reject CLM-2 duplicate-publication**, renamed, + accept sibling | B2 |
| `reject-clm1-claim-on-loop-carried-update.wf` | reject CLM-1 | **accept**, claim retained, renamed `accept-clm1-…`, gap token `flow` | B2 |
| `ent5-neg-value-match-no-delivery.wf` | negative | **positive**: a `value_match` receiver is an ordinary merge | B1 |
| `ent5-pos-value-if-delivery-join.wf` | accept, two claims load-bearing | **rewritten**: both `%` claims become duplicates; the delivery join is shown without claims, which is a better fixture | B1 |
| `clm1-pos-passing-claim-establishes-fact.wf` | accept (`seed = 3; index = seed % 8`) | **rewritten**: the predicate is now derivable; needs a genuine `vocabulary` residual | B1 |
| `clm2-pos-redundant-claim-advisory.wf` | reject CLM-2 | verdict unchanged; classification renamed to `duplicate-publication`; file renamed `clm2-neg-repeated-claim-duplicate-publication.wf` so the "advisory" fossil goes | B3 |
| `ent3-neg-stage8b-local-one.wf`, `ent3-pos-stage8b-bit-sources.wf` | verdicts hold | derivations move from S7 rows to row images | B1 |

### 7.2 Verdicts that hold, with their prose re-narrated

`reject-clm1-claim-on-selected-payload.wf`,
`clm1-neg-user-result-claim-locality.wf`,
`clm1-neg-system-result-claim-locality.wf`, every `ent2-*`, `ent4-*`, `prv*`,
`clm3-*`, `clm1-trap-*`, and `clm2-neg-refuted-claim`. No manifest change. The
`checker gap:` prose of every retained claim gains its token, and prose that
appeals to definition occurrences is rewritten to state the reason that is now
true. `ent5-neg-loop-rule-drops-preloop-fact.wf` stays negative and is now the
case that *documents* the published `flow` clause rather than an accident.

### 7.3 New cases required

| case | pins |
| --- | --- |
| `reject-clm1-claim-on-callee-written-storage.wf` | §2.5's program. **The first corpus member of H3, ever** |
| `reject-clm1-claim-on-callee-written-literal.wf` | boundary opacity: the callee writes only `0_u8` and the claim is still refused, so a later reader cannot "improve" the seed by inspecting the callee |
| `accept-clm1-length-survives-a-callee-element-write.wf` | P16 and the element half of kill (b); the guard that the seed did not over-reach |
| `reject-clm1-claim-on-callee-replaced-buffer-length.wf` | the whole-place-replace half of kill (a); the program that a legislated length exemption would wrongly admit |
| `accept-clm1-nested-selection-local.wf` | three literal definitions under two nested boundary selectors; catches a reintroduced frame stack |
| `accept-clm1-claim-on-loop-carried-literal.wf` | row 7 with a `loop_stmt` rather than a `for_stmt`, so both repetition forms are covered |
| `accept-ent5-delivery-of-literal-discharges-a-subscript.wf` | §3.3: two arms deliver `0_u64` and `1_u64` and an ordinary subscript discharges with no claim |
| `accept-ent3-set-commit-establishes-image.wf` | the `let`/`set` spelling defect, closed |
| `accept-ent3-value-match-delivery.wf` | 3085's deletion |
| one case per admissible gap token, plus one reject naming `boundary` | §3.5's lexical check and the token's teaching |
| `reject-clm1-carrier-tiebreak.wf` | the tie-break at a genuine tie, which 0102 left unpinned |
| `clm2-neg-duplicate-component-names-its-sources.wf` | §3.8's payload: the component ordinal, the derivation, and the fix |
| one case per nonempty image row family (`%`, `/`, `imin`/`imax`, `iand`, shifts, popcount family) | §5.5's partial mitigation |
| one case per newly imported [SYS-8] contract | §3.2's S10 |

### 7.4 The claim corpus, by the writers' own gap text

Tallied mechanically at `b1367c82` over `tests/conformance/cases/*.wf` and
`tests/programs/*.wf`.

| gap text (verbatim, deduplicated) | n | token | fate |
| --- | --- | --- | --- |
| "ENT proves the remainder operation domain but does not publish its result range" and its three variants | **43** | — | **retired** by the `%` image |
| "the entailment state carries no residue for a literal remainder / for a remainder by a literal" | **3** | — | **retired**, ground case of the same image |
| "ENT does not publish the result range of imin" | 5 | — | **retired** by the `imin` image |
| "ENT does not publish a nominal payload value through borrow-mode match" and the constructor-field variant | 3 | — | **retired**: construction is a row and its image is `field = operand` |
| "ENT does not derive the strict loop range from the equality exit and loop recurrence" and its four rewordings | 12 | `flow` | **kept** until B6 |
| "ENT does not derive this ordinary-loop induction invariant / across the backedge" | 5 | `flow` | **kept** until B6 |
| "ENT does not correlate the two borrowed column lengths / the two nominal field lengths" | 8 | `flow` | **kept** until B6: the loop range blocks them; the length correlation itself is two-term and needs only a publisher (S6, or a `requires`) — see the erratum below |
| "ENT does not publish the borrowed buffer length through the child call" | 2 | `flow` | **kept** |
| "ENT does not normalize the remaining-length guard into this offset bound" | 2 | `vocabulary` | **kept**, pending review of the token |
| "ENT does not publish an uncontracted user-call argument value into apply" | 2 | `boundary` | **verdict unchanged**: deliberate-refusal fixtures |
| "there is no checker gap for a direct False() predicate" and siblings | 6 | — | unchanged negative fixtures |

Roughly **54 of 135 claims retire**; 39 mention a loop and survive on `flow`
until B6 — the eight two-length claims among them, per the erratum below, which
is where `CENSUS.md` §2.2 independently places them; the rest are deliberate
`boundary` refusals, one `content` claim, and a `vocabulary` residue the census
counts as three claims tree-wide.

> **Erratum (B0 census).** The eight "two borrowed column lengths / two nominal
> field lengths" claims were tagged `vocabulary` in the table above, with the
> reason "a relation between two independent lengths is not two-term". **That
> reason is false**, and `CENSUS.md` §6.3 shows why: [ENT-2] 2870(b) makes
> `len(P)` a term and 2901 makes `t1 - t2 <= c` an atomic fact, so
> `len(a) - len(b) <= 0` is a well-formed [ENT-2] fact, and conjoined with
> `len(b) - len(a) <= 0` it is exactly the equality those claims want — in the
> vocabulary the language already has. What is missing is a **publisher**: in
> `x-struct-of-buffers-checksum-run.wf` both lengths come from
> `buffer_new(6_u64, …)` and S6 could publish both, and in
> `x-buffer-borrowed-columns-run.wf` the equality is a caller fact a `requires`
> clause would publish. All eight are publisher gaps (`CENSUS.md`'s bucket P),
> and their token is `flow` — the loop range is what actually blocks them, and
> each gap text says so first. The row is corrected accordingly. Their fate is
> unchanged (kept until B6), but the reason for keeping them is a missing
> transfer, not a missing fact language, and they must not be counted toward
> §5.2's `vocabulary` ceiling clause — which, on the census's count, holds three
> claims in the whole tree rather than the ten this table assigned it.

### 7.5 Other derived material

- **`tests/programs/`**: 18 claims in 7 files. The remainder and `imin` ones are
  deleted; `wfgrep.wf` (5), `percent_decode.wf` (4), `fir_filter.wf` (3),
  `ipv4_checksum.wf` (2), `utf8parse.wf` (2), `raw_deflate_dynamic_decode.wf` (1)
  and `par_layout.wf` (1) each need re-reading against the new ceiling. The 18
  claim-free programs are unaffected, and their IR must stay byte-identical modulo
  the version banner — that is a falsifier, not a hope (§8, F7).
- **`tests/codegen/cases/bounds/`**: 15 claims in 13 fixtures, of which 12 are
  one-line `ieq(value, N)` drift oracles. **All thirteen fixtures are rejected by
  the current compiler**, so this item is not an adjustment to a working fixture
  set: it is a rewrite of thirteen files that do not compile, each needing a
  claim with a legal subject before any question about the new ceiling arises, or
  replacement by a non-claim oracle. This is real work and is easy to
  under-estimate.

  > **Erratum (B0 census).** This bullet previously read "Under the image closure
  > most become duplicates and must be re-cut to a predicate above the new
  > ceiling", which assumes the thirteen fixtures compile at `b1367c82`. They do
  > not. `CENSUS.md` §6.1 compiled every one: all thirteen are rejected, **all
  > thirteen citing [CLM-1]** — the twelve masked-index drift oracles claim
  > `ieq(value, N)` over a *direct user-call result*, publisher 2's subject
  > matter, refused by [CLM-1] since long before v0.39, and
  > `output-capacity-lockstep/p08` claims `False()`, which fails CLM-1 fact-free
  > formation. Two things change. **Size**: the work is a rewrite rather than a
  > re-cut. Each fixture's claim must first be given a subject the fixture's own
  > text produced — under v0.39's gate and under this design's alike, since a
  > direct call result stays `boundary` subject matter — and only then can the
  > image closure make it a duplicate. Nothing in B1's ceiling raise moves these
  > verdicts on its own, because [CLM-1] is judged before [CLM-2] and refuses them
  > first. **Verification**: they cannot serve as before/after oracles for the
  > verdict moves of §7.1, nor for F7's IR identity, because there is no compiling
  > "before" to differ from; a rewritten fixture's first accepted compile is its
  > own new baseline, and its drift-oracle role has to be re-established rather
  > than preserved. B5's line "re-cut the drift oracles" should be read with
  > that. The count is otherwise inert: `CENSUS.md` §1.2 D and §5 limitation 6
  > exclude these 15 claims from every discharge population, so no number in §7.4
  > or §5.3 rests on them.

- **Compiler tests.** `compiler/src/semantic/tests/claim_locality.rs` holds ~130
  tests. Every test naming `DefinitionId`, `ControlAuthority`, `acquired` or a
  selector stamp is deleted **with its subject**, and the honest technical
  explanation is that the mechanism ceased to exist. Six move from non-local to
  local: `a_call_result_used_as_if_control_taints_written_values` (:637),
  `a_write_on_one_arm_only_is_selected_at_the_join` (:1716),
  `a_counted_endpoint_selects_loop_carried_state` (:1783),
  `an_ordinary_loop_selects_state_its_iterations_wrote` (:1816), and the two
  laundering tests below. `a_local_named_const_component_reaches_the_redundancy_judgment`
  (:1187) keeps its rejection and is renamed for the new cause name.
- **The two laundering tests.** `a_result_tag_cannot_be_laundered_through_a_value_match`
  (:332) and `a_call_result_used_as_value_if_control_taints_the_delivery` (:598)
  assert that the *checker* refuses `claim ieq(picked, 0_u64)` over arms `{0,1}`.
  Under this design the checker admits them and review refuses them. They are
  **rewritten as acceptance tests carrying §3.5's reason in their doc comment**,
  not deleted — judge-2 listed this as undone by all three input designs, and the
  spec sentence that justifies it is the one added at 2720.
- **Load-bearing and unchanged**: `a_matching_binder_is_selected_by_its_own_tag`
  (:1853, the binder *is* the payload), `a_direct_result_payload_keeps_the_call_boundary`
  (:298), `the_length_of_a_returned_buffer_keeps_the_call_boundary` (:485),
  `a_returned_box_cannot_hide_its_dereferenced_payload` (:513),
  `an_exact_ensures_does_not_authorize_a_caller_restatement_claim` (:857).
- **Anchors and docs**: `compiler/src/backend/qualification.rs`'s `REVIEWED_FOR`,
  the six digest anchors and two transcribed literals any spec activation touches
  (`docs/done/0102-clm1-narrow.md:66–90`), `docs/patterns.md` P15–P18, and the
  roadmap's PROOF-8 entry.

---

## 8. Implementation plan

Design only; no code is written here. Sizes are estimates and are the numbers I
would most expect to be wrong. Each batch is one merge to `main` and must pass
canonical `make check` at the exact revision merged; each carries its own derived
material, because bringing everything derived to the newest version in the same
work is not optional.

**B0 — measure before deciding. Small (1–2 days, no spec text).**
Build the H3 reconstruction (§2.5's `fill`/`read` pair, plus a variant where the
callee's write depends on its own `match` on a call result) and compile it at
`b1367c82`. This design's only *tightening* rests on the prediction that v0.39
accepts it, and no one has seen it compile — the `s36`/`s37` programs the charter
cites do not exist in this tree. Build the laundering programs of §1.4 and §9 in
the same pass, and stand up the review-trial harness (F2). Deliverable: a
`docs/done/` record with the measured verdicts. **If v0.39 already rejects the
reconstruction, §3.4's write seed is unnecessary and this design changes at Q3.**

**B1 — the ceiling. Large (the biggest batch; ~2 weeks).**
Enumerate the operation table's image column row by row **and decide each row's
direction** (§5.3 and its erratum, §5.5) — per `CENSUS.md` §2.4 the enumeration
must settle whether a row may publish a *backward* rule from result facts to
operand facts, and specifically must decide the `±wrap` backward rule, which is
the corpus's only fact-language request and closes four claims in three real
programs; write
[ENT-3.S5], the generalised S10, [ENT-1]'s ceiling and direction guarantee, and
[ENT-5]'s value-commit/merge text; delete the delivery block, [GIVE-1]'s carrier
judgment and 3009. Compiler: new `semantic/entailment/image.rs` (~700 lines,
table-driven), `flow.rs` loses three delivery functions and gains the
value-commit dispatch, `state.rs` gains image application. Migration: the ~54
retiring claims, the three moved accepts, the two `ent5` delivery cases, the
`ent3` stage-8b derivations, the codegen bounds fixtures. Gate additions: a test
that every operation-table row has an image entry; one conformance case per
nonempty image row family.

*Why the ceiling lands before the gate.* Under [DIAG-1]'s schedule CLM-1
authority is judged before CLM-2, so while the gate is unchanged the
delivered-value and arm-written cases stay CLM-1 rejects and their verdicts do
not move. Landing the gate first would move them to accept in B2 and to CLM-2
reject in B1', costing two approval records per case for one net change.

**B2 — the gate. Medium (~1 week).**
Write [ENT-6]'s admission state, repeal 3233–3239 and 3241–3243, rename the
classes, write [CLM-1]'s gate and the settlability sentence at 2720, and write
[ENT-6]'s route menu. Compiler: `claim_locality.rs` loses `DefinitionId`,
`DefinitionKind`, `ControlFrame`, `ControlAuthority`, the definition-comparing
merge and the six selector/discharge sites (−300 to −400 lines), and gains the
call-write seed reading `KillEvent::Write`'s projection (+60 to +90). Migration:
the three moved rejects and their accept siblings, the six moved compiler tests,
the two rewritten laundering tests, and §7.3's five admission cases. Estimated
final size of `claim_locality.rs`: 1,700–1,850 lines from 2,122.

**B3 — non-duplication and residuality. Small (~3 days).**
Write [CLM-2]'s single rule; close or refute §6.2's unproved step against
3014–3023 and record which; rename the verdicts in [DIAG-1]'s cause set. If the
step does not close, the two tests stay separate and only the *names* and the
fixes unify — the writer sees no difference.

**B4 — the teaching channel. Medium (~1 week).**
Write [DIAG-1]'s payload and fix table, [DIAG-2]'s case list, the gap token and
its lexical check, the review note, and the carrier tie-break. Migration: all
~86 surviving claims gain a token; the diagnostic goldens are re-cut. This is the
batch with the widest, shallowest diff and it should not be merged with another.

**B5 — the corpus pass. Medium (~1 week).**
Re-read the seven claim-bearing real programs and the 13 codegen bounds fixtures
against the settled ceiling; re-cut the drift oracles; run F7 (byte-identical IR
on the claim-free programs); update `docs/patterns.md` P15–P18 and PROOF-8.

**B6 — the loop head, later and gated. Large, and separate.**
Raise the `flow` clause: replace the subtraction by the [ENT-5] join over the
preheader and back edges with widening at every unstable iterate, iterated until
`B(X) ⊑ X` is verified. Three gates, all mandatory: a compiler assertion that the
declared head state is a post-fixed point on every loop in the corpus; a
differential run showing the head state dominates the subtraction on every
conformance case and program; and a measured compile time on `wfgrep.wf` (1,417
lines) and `raw_deflate_vectors.wf` (863 lines). Retires most of the 39 `flow`
claims. **This is the only batch with a soundness bill and it must not be folded
into B1.**

### 8.1 Falsifiers

Each names the element it would refute, and each is an experiment.

**F1 — the route menu, mechanized.** §3.7 states normatively that at least one
route is always open and the diagnostic names it. Generate programs with unproved
obligations, apply the compiler's own named fix mechanically, recompile. Any
program where the named route does not compile refutes the theorem. H2 is exactly
one such program today. The 0097 harness already generates the corpus; only
apply-and-recompile is new. **Run this first after B2.**

**F2 — the review trial.** §1.4 rests on the assertion that a laundering claim
carries its own counterexample in the function text. Hand reviewers — human, and
separately AI-assisted, since 2719 admits both — a blinded set of joined-literal
claims, half true on every arm and half true on one arm only, with §3.8's case
list attached and, as a control, without it. If reviewers approve the false ones
at a material rate *with* the list, the gate's line is in the wrong place and the
checker must do more than this design asks. If they catch them with the list and
miss them without it, the case list is not optional. **This is the measurement
the whole design's weakest premise rests on** and it is the only proposed
experiment in the batch that measures reviewers instead of asserting about them.

**F3 — the image-uniqueness audit.** Walk every operation-table row and write its
image. Refutes §3.2 if any row's exact image is expressible but not unique in
[ENT-2] and the non-uniqueness is *not* already covered by 2739's
`bxor`/signed-normalization exclusions or 3146's two-non-constant-operand
exclusion — i.e. if `image` turns out to be an open-ended list rather than a small
closed one. **Direction (erratum, B0 census).** The audit as first written asked
only about uniqueness. It also asks, per row, whether the row publishes a
*backward* rule — a fact about the result establishing a fact about the operands
across `r = a ± b` — and in particular whether `+wrap`/`-wrap` do, under the
no-wrap side condition the state already carries. A forward-only column closes
none of `CENSUS.md` §2.4's T1–T4, so a forward-only answer here is not a neutral
outcome but a decision to leave four real-program claims standing. The direction
half of F3 is refuted if the backward rule cannot be stated per row from side
conditions already in the state — that is, if publishing it would require holding
the three-term equality itself, which [ENT-2] cannot.

**F4 — a write the kill relation misses.** Exhibit a program in which a callee's
write reaches caller storage that [ENT-5] kill (b) does not kill. It refutes the
admission seed **and** the fact state **and** [PRV-1] 3207 simultaneously. That
shared blast radius is the point: a private authority projection would have hidden
it.

**F5 — a program someone wants that the write seed refuses.** A legitimate claim
over storage a callee wrote whose relation cannot be published as an `ensures`.
If it exists, the seed is too strong given today's FN-9 and Q3 becomes urgent.
The corpus contains zero such programs, which is not evidence either way — only a
program someone wants and cannot write is.

**F6 — churn that is not a removal.** §3.1 guarantees every forced edit removes
claim content. Take B1's own ceiling raise and check that all 54 affected claims
delete or lose a conjunct cleanly with no invented restructuring. If any needs a
rewrite beyond the record lines the removed content owned, the granularity is
still wrong.

**F7 — corpus IR identity.** The 18 claim-free real programs must compile to
byte-identical IR modulo the version banner across B2, exactly as 0102 verified
for 26 sources. A single moved program means the write seed's projection is wider
than the kill projection it claims to reuse.

**F8 — the blind-writer trial, repeated.** `docs/done/0098-blind-writer.md`
reports zero claims in 1,694 lines under today's ceiling. Repeat it after B1.
*Prediction, recorded so it can fail:* the walls will be loop induction and
two-term-vocabulary relations, and **not** claim locality. A campaign that reports
locality as the wall contradicts both that record and this design.

**F9 — the one that would refute the principle.** A program someone wants to
write whose only correct spelling requires a claim over a value a callee or the
world produced, and for which neither a verified `ensures`, a
specification-fixed contract, nor a branch on the returned value is available.
That would show the `boundary` class is not always repairable and that the gate is
too strong. Four independent traditions agree the callable boundary is the right
place (TERRAIN §5.4) — which is why this is worth actively hunting rather than
waiting for.

---

## 9. The attacks the judges landed

Two judges wrote twenty counterexample programs against the three input designs.
Every one is answered here, either because the design element it broke is not in
this synthesis or because the synthesis changed the element. Nothing is inherited
unexamined.

### A1 — the loop rule derives a false fact *(both judges, two independent programs)*

```whitefoot
let a = 0_u64; let b = 0_u64;
loop @l {
  if igt(a, 5_u64) { set b = b +wrap 1_u64; }
  set a = a +wrap 1_u64;
  if igt(a, 100_u64) { break @l; }
}
return small[b];                      // small: own array<u8, 2>
```

Under prover-first's rule `H1 = {0≤a≤1, b=0}`, `H2 = {0≤a≤2, b=0}`; only `(a,Z)`
changed, so widening relaxes `a` and **keeps `b = 0`**; `B(W)` then unlocks the
inner branch and gives `0≤b≤1`; the head is `W ⊔ B(W)` and the rule **stops**.
`b` actually reaches ~94, so a `SubscriptBounds` obligation discharges against a
false bound with **no claim in the program**. Judge-2's `i/g/h/x` gate chain is the
same defect through a three-deep dependency chain.

**Dissolved by not shipping it.** §3.3 publishes the loop head as the subtraction
it is today, and §5.4 states the repair (widen at every unstable iterate, iterate
until `B(X) ⊑ X` is *verified*) as batch B6 with three mandatory gates, one of
which is the post-fixed-point assertion these programs violate. The 39 loop claims
keep the token `flow` until then, which is honest rather than convenient.

### A2 — `refuted-on-a-path` rejects true, text-reviewable claims *(judge-1)*

```whitefoot
let flag = hidden_true();
let cursor = 9_u64;
if flag { set cursor = 0_u64; }
if flag { claim c: ilt(cursor, 4_u64) because "…"; return values[cursor]; }
return 0_u8;
```

A tag names a merge *input edge*, not an executable path. The else edge's witness
bound `cursor ≥ 9` refutes the predicate under one tag, yet that path cannot reach
the claim. The claim is true and settlable from the text, and it is precisely a
`flow`-gap claim — the category the design reserves for claims. A new
false-rejection engine of v0.38's own class.

### A3 — the witness tag is a path suffix *(judge-2)*

```whitefoot
let t = calls_bool();
let a = 0_u64; let b = 0_u64;
if t { set a = 5_u64; }             // merge A
if t { set b = 5_u64; }             // merge B
claim paired: ieq(a, b) because "…";   // TRUE on every execution
```

A bound that merely *flowed through* a merge is retagged with the last merge's
input edge, so two bounds sharing a tag need not share an execution: `a ≥ 5` from
merge A and `b ≤ 0` from merge B's false edge derive `a − b ≥ 1` under one tag and
reject a claim true on every execution. The alternative reading — never retag —
makes the mechanism unable to fire at a loop head, one of the three cases it was
promised for. The third option, a path token surviving merges, is trace
partitioning, which [ENT-1] 2831 forbids.

### A4 — the refuter is blind exactly where claims live *(judge-1)*

```whitefoot
let arr = buffer_new(8_u64, 0_u8);
match g(x: input) { Ok(value: v) => { set arr[0_u64] = 1_u8; } Err(error: e) => { } }
let e = arr[0_u64];
claim c: ieq(e, 1_u8) because "premises: g never fails for our inputs …";
```

Elements are a `content` gap, so nothing is tracked and nothing can refute. The
claim is admitted and justified only by `g`'s body.

**A2, A3 and A4 are dissolved together.** The witness state and the
`refuted-on-a-path` verdict are **not in this design** (§4.1). §3.8 keeps the one
thing they were worth — the rendered message, which is the best writer-facing text
proposed anywhere in this batch — as a **review note** computed from the retained
case list and the certain state. As a note, A2's program gets a note and compiles;
A3's program gets a note and compiles; A4's program gets no note, which is honest,
because the checker genuinely knows nothing there. A4 is also a member of the
laundering family and is §11's subject.

### A5 — a design's stated fence property is false *(judge-1, judge-2)*

```whitefoot
let cursor = 3_u64;
if condition { set cursor = 0_u64; }     // condition = a call result
claim c: ieq(cursor, 0_u64) because "…"; // admitted; true iff the call said True
```

world-first claimed its judgment "refuses every claim whose predicate may take a
premise from the world" and licensed only predicates true of *every* member of the
enumerable union, while its computed gate never checks that side condition. Both
judges refuted the sentence with three-line programs.

**Dissolved by not asserting it.** §1.1 makes the gate a **subject-matter** rule
and §3.5 states in the specification that it is a *necessary condition for
settlability and not a truth screen*. This program is admitted, the reviewer
enumerates `{3, 0}`, finds the derivation needs an unpublished premise about
`condition`, and refuses. §11 marks the residue in red ink.

### A6 — the trip-count clause misses the trip count *(judge-2)* and ignores `break` *(judge-1)*

```whitefoot
let n = endpoint(value: input);
let flag = 0_u64;
for @steps step in 0_u64..n { set flag = 1_u64; }
claim never_ran: ieq(flag, 0_u64) because "…";   // admitted by clause 3
```

```whitefoot
let acc = 0_u64;
for @steps step in 0_u64..64_u64 {              // endpoints are literals
  set acc = acc +wrap 1_u64;
  match g(x: input) { Ok(value: v) => { break @steps; } Err(error: e) => { } }
}
claim c: ilt(acc, 4_u64) because "…";           // admitted; g decides the count
```

The clause tests for self-composition, so it misses "whether a definition executed
at all"; and a counted loop's repetition class as drafted reads only its endpoint
captures, so a `break` on a call result escapes it.

**Dissolved by deleting the clause** (§2.4), with a derivation rather than a
patch: any repair that catches `never_ran` while admitting row 7 must compute the
joined value set, and a checker that computes it derives row 7's predicate and
makes that claim a duplicate publication. There is no setting of the dial with
work to do. Both programs are laundering-family members and go to review, and the
case list renders `acc`'s back-edge definition **as a recurrence** so the reviewer
knows an induction is owed — which answers judge-1's separate complaint that a
flat definition list would conceal it.

### A7 — a normative direction guarantee falsified by a retained verdict *(judge-1)*

world-first proposed "every source edit this rule forces is a deletion" while
keeping the per-component overlap verdict, under which a strengthening that learns
one conjunct forces a predicate edit **and** a rewrite of the record's
`derivation:` and `conclusion:` lines.

**Dissolved by wording the guarantee to what is true** (§3.1): every forced edit
*removes claim content*, never adds an obligation, never requires a new claim,
never narrows an ordinary program's accepted set. §3.8 gives the component case
its own fix line naming the ordinal, so the edit is located even though it is not
a whole-statement deletion.

### A8 — "there is exactly one way for a value set to be invisible" is false *(judge-1)*

```whitefoot
let n = endpoint(value: input);
let acc = 0_u64;
for @steps step in 0_u64..n { set acc = acc +wrap 1_u64; }
claim bounded_acc: ilt(acc, 4_u64) because "…";
```

**Dissolved by not making that claim.** §1.1 says the gate decides subject matter,
and §1.4 states the second way — a boundary-decided *choice among* text-produced
values — as the design's stated limit, with three reasons no mechanism closes it
and one measurement (F2) that would move the line.

### A9 — a shape rule survives: `give a;` carries a fact, `give 0_u64;` does not *(judge-1)*

**Dissolved by §3.2.** A `give` edge is a value-commit event and the image is
indexed by the operation and its destination, not by the atom's spelling. The
carrier grammar at [GIVE-1] 291–293 and [ENT-5] 3084 is deleted, not narrowed.
The same deletion closes world-first's `let`/`set` spelling defect and prover-first's
`value_match` exclusion in one clause.

### A10 — the delivery substitution corrupts the state on its own headline case *(judge-2)*

prover-first replaced delivery with "substitute every occurrence of the delivered
atom's term by the receiving binding". A literal is an [ENT-2] term, `0_u64`
normalizes onto Z (2870(f)), and 2905 puts Z in every implicit fact
`t − Z ≤ max(T)`; the substitution rewrites the arm's whole numeric frame into
relations against `picked`.

**Dissolved by §3.3.** There is no substitution. The delivery edge *establishes
the delivered atom's row image over the receiving binding* — a finite set of facts
about one destination place — which is world-first's correct *form* without
world-first's self-contradicting carrier list.

### A11 — the length exemption, legislated, admits a world-determined length *(judge-2)*

```whitefoot
fn refill['b](slot: &uniq 'b buffer<u8>) -> result: own unit writes(slot) {
  let n = measure();
  set deref(slot) = buffer_new(n, 0_u8);          // whole-place replace
  return unit();
}
… region 'b { let done = refill<'b>(slot: &uniq 'b room); }
let extent = len(room);
claim bounded: ilt(extent, 4_u64) because "…";    // admitted by a blanket exemption
```

**Dissolved by deriving instead of legislating** (§2.5, §3.4). The seed is exactly
what kill (b) kills, and [ENT-5] 3066(a) already says a whole-place replace kills
that buffer's length facts while an element write kills none. So this program is
refused and P16's program is admitted, from one rule, with no length clause
anywhere. §7.3 makes both a conformance case.

### A12 — the H3 seed's length behaviour was asserted, not written *(judge-1)*

prover-first's prose said the projection reaches elements and not the length and
wrote no clause. **Dissolved by §3.4**, which names kill (b) as the seed's
definition, so the element/length split is not this rule's business at all.

### A13 — a demand-driven ceiling keeps a distinction by spelling *(judge-2)*

```whitefoot
let a = 3_u64 +wrap 4_u64;         claim x: ilt(a, 13_u64) …   // a source exists
let b = imul.wrap(3_u64, 4_u64);   claim y: ilt(b, 13_u64) …   // "not a source until written"
```

world-first's law made a source exist "when a program someone wants to write is
blocked without it", which is the accreting list with an admission test. Two
adjacent ground expressions get opposite treatment for no semantic reason.

**Dissolved by §3.2's totality.** Every row publishes an image or the empty image.
`imul.wrap` of two pinned operands is pinned by the multiply row's image; there is
no petition and no waiting list, and 3009 — the specification's own statement of
the petition method — is deleted.

### A14 — the redundancy⊂non-residuality proof has an unclosed step *(judge-2)*

It needs "no downstream judgment distinguishes an established fact from a derived
one", and [ENT-4] 3023 is evidence the specification distinguishes them somewhere.
**Dissolved by not depending on it** (§6.2): §3.6's rule states both tests, batch
B3 closes or refutes the step against 3014–3023, and the writer's diagnostic and
fix are identical either way.

### A15 — admitting a partly-derived conjunct contradicts the P0 argument *(judge-2)*

writer-first refused acceptance-as-documentation because a retained check costs
P0 forever, then admitted a two-conjunct claim one conjunct of which the checker
derives — whose retained check evaluates that conjunct forever.
**Dissolved by keeping the component as the unit** (§3.6, §6.2): a derived
component is a duplicate publication and is refused, with a fix naming the ordinal.

### A16 — deferring the ceiling channel on churn grounds *(judge-2)*

writer-first deferred its ceiling identifiers "as its own batch … it touches all
135 existing claims", which is migration-cost reasoning the charter excludes.
**Dissolved by shipping the token in B4** (§3.5), inside this design, with the
migration of all surviving claims as part of it. The one thing this design *does*
sequence separately, the loop head, is sequenced on a soundness bill and two
counterexample programs (§5.4), not on churn.

### A17 — the three items all three input designs left undone *(judge-2)*

- **The laundering family needs a decision in the specification, not a design
  document.** Written, at 2720, in §3.5.
- **The postcondition-candidate second pass has no sentence.** Written, in §3.3.
- **Nobody measured anything.** Batch B0 exists for exactly that and gates the
  only tightening (§8).

---

## 10. Provenance of the ideas

### 10.1 Taken

| idea | from | why it survived |
| --- | --- | --- |
| The ceiling as a **closure indexed by the operation table and the control graph**; every row publishes an image or an explicit empty image; every merge is the same merge; every specification-fixed contract is a source | prover-first §1.3 | the only construction in the batch that makes "a hole nobody enumerated" structurally impossible. Its own evidence is structural, not a count: `imin` has an authority row and no fact source, `value_match` delivers values and forms no image, five [SYS-8] contracts sit outside the state — three constructs in that state today, each one of the charter's holes. Both judges ranked it first among ideas |
| The **four gap tokens plus the inadmissible `boundary` token**, checked at the head of `checker gap:` | prover-first §1.4/§3.5 | four spec lines and one lexical check; gives H5 a uniform channel, makes every ceiling amendment greppable and announceable, and converts the review question from "is this prose true" to "is this token right". Preferred over per-instance ceiling identifiers, which need a registry and a new identifier per amendment |
| **One kill projection, three consumers**: the call-write seed is exactly what [ENT-5] kill (b) kills, with the shared blast radius as its falsifier | writer-first §4.4/D3 | judge-2 called it the best paragraph in the batch. It is what stops the area re-growing a private notion of what a call touched, and it is why the length behaviour needs no clause |
| The mechanical condition stated as **necessary and not sufficient**, in the specification, with the laundering family named | writer-first §1.8 | the difference between a design that has a limit and one that has an unexamined hole. Both judges said so |
| The **reviewer's case list** as published review data, never an acceptance criterion | writer-first §7.3 | the one safe place a definition inventory can exist after `DefinitionId` dies: NodePath identities, diagnostic-defect blast radius, and it makes 2719's review duty executable |
| The **theorem** that redundancy is a special case of non-residuality | writer-first §6.1 | kept as the reason the two tests coincide and as a B3 obligation, not as a load-bearing premise (A14) |
| **Delivery equals what an ordinary `let` establishes** (the form) | world-first §3.4 | immune to the "next case" objection by construction. Its instance was self-contradicting; the closure of §3.2 supplies the correct one |
| **`set` should establish what `let` establishes** | world-first §2.6/Q9 | a real distinction by spelling nobody else found. Under §3.2 it is not a rule but the absence of one |
| The **[ENT-1] direction guarantee** | world-first §3.6 | the honest replacement for a stability promise 2853 cannot make — reworded to what is true (A7) |
| The **channel enumeration** (C4 injects nothing; C5 is C2) | world-first §0 | the cleanest statement of why v0.38's clause was a category error rather than conservatism; used as the derivation of "choosing an edge publishes no value" |
| The design rule **"a lattice point can be wrong only by being imprecise; an artifact identity can be wrong by being unsound"** | prover-first §4.2 | H1's real lesson; §4.2 puts it where the specification can hold it |
| The **`refuted-on-a-path` rendered message** | prover-first §1.7 | the best writer-facing text in the batch. Kept as a note; the verdict is not (A2, A3) |
| The **closed cause→fix table** and the **carrier tie-break by predicate source order** | writer-first D10/D11, prover-first §3.8 | the carrier is the value the writer must change, so it should be the one their eye reaches first |

### 10.2 Rejected, with the counterexample that decided it

| idea | from | rejected because |
| --- | --- | --- |
| The **loop-head fixed point as drafted** (widen once, one confirming pass, "at most four passes") | prover-first §1.5/§3.3 | A1. Two independent judge programs in which the declared head state asserts a false bound and a `SubscriptBounds` obligation discharges against it, with no claim in the program. Kept as batch B6 with a verified post-fixed point and three gates |
| The **witness state and the `refuted-on-a-path` verdict** | prover-first §1.7 | A2 (rejects true `flow`-gap claims, the category claims exist for), A3 (the tag is a path suffix, so bounds sharing a tag need not share an execution; and the "never retag" reading cannot fire at a loop head), A4 (blind wherever a claim is legitimate). The message survives as a note |
| The **delivery-by-term-substitution** rule | prover-first §3.3 | A10. A literal normalizes onto Z and 2905 puts Z in every implicit fact |
| **Trip-count dependence (clause 3)** | world-first §1.3 | A6. It catches self-composition and misses "whether a definition executed at all"; a counted loop's repetition class ignores `break`; and any repair either becomes v0.38's blanket over loops or needs the value analysis that makes the good claim a duplicate |
| The **legislated length exemption (clause 4)** | world-first §1.3 | A11. It contradicts [ENT-5] kill (a) verbatim and admits a claim over a length a system call sized |
| The **demand-driven fact-source law** ("a program someone wants to write is blocked without it") | world-first §5.1 | A13. It is the accreting list with an admission test, and it keeps two adjacent ground expressions on opposite sides |
| **Dropping per-component residuality** | writer-first D7 | not a counterexample but a uniformity argument (§6.2): S3 establishes components, so the component is the unit of publication, and the performance win it buys is unmeasured. Put to the owner at Q5 |
| **Dropping the overlap verdict** | writer-first D6 | A15. A derived conjunct's retained check costs P0 forever, which is the same argument that refuses acceptance-as-documentation |
| **Deferring the ceiling channel on churn grounds** | writer-first Q2 | A16. Migration cost is not a design criterion |
| **Advisory / accept-with-a-note redundancy** | the `clm2-pos-redundant-claim-advisory.wf` fossil | §6.1. Refuted by T3/P0 (a check no root needs taxes correct programs), by R4 (a rejection the writer may ignore is silence), and by the architectural argument that an accepted duplicate perturbs every other claim's `Full-minus` counterfactual |

### 10.3 Where the two judges disagreed, and how it was decided

Judge-1 ranked prover-first first on the charter's own terms and would have
grafted world-first's clause 3; judge-2 ranked writer-first first and said clause
3 and clause 4 "should not survive into a draft". The disagreement resolves on
evidence rather than by splitting: judge-2's `never_ran` program refutes clause 3
directly, and §2.4 adds the derivation that no repair to it has work to do.
Judge-1's own ranking note says the ranking inverts "if the owner weights
soundness-as-written above ceiling"; this design takes the ceiling *and* declines
the one clause that carried the soundness bill, which is the position neither
judge was asked to score.

---

## 11. Unsolved problems, in red ink

The charter says: *if the synthesis needs an exception, say so in red ink as an
unsolved problem rather than hiding it as a rule.* Three things qualify. None of
them is an exception carved into the principle; all three are places where the
principle is right and the mechanism is incomplete, and each is stated here rather
than smoothed into a clause.

### RED INK — U1 — the laundering family is admitted, and review is the only fence

A claim whose support reads only values this function's text produced, but whose
truth depends on which of those values a boundary selected, passes the gate.

```whitefoot
let n = hidden();
let big = ige(n, 4_u64);
let y = if big { give 5_u64; } else { give 1_u64; }
claim laundered: ilt(y, 4_u64) because "…";   // admitted. FALSE when big.
```

**This is the design's one genuine soundness residue and it must not be
under-sold.** T3 rests one notch more heavily on the review than it did under
v0.39, which refused this shape (along with 63 of 63 true programs of the same
shape). What the design offers in exchange:

- the specification *states* the limit at 2720 rather than implying a fence that
  does not exist (§3.5);
- the reviewer's premise rule is made precise — a `derivation:` may cite only
  published premises, and this one cannot (§1.5);
- the case list and the review note put the counterexample in front of the
  reviewer (§3.8);
- **F2 measures whether that works**, on humans and on AI-assisted reviewers, with
  and without the case list (§8).

What the design does *not* offer is a mechanism, and §1.4 gives three independent
reasons why: the checker is on the wrong side of both halves of the family, every
path-sensitive technique is forbidden by [ENT-1] 2831/2836, and the one mechanism
proposed in this batch was refuted twice (A2, A3). **If F2 reports that reviewers
approve false laundering claims at a material rate even with the case list, this
design's central concession is wrong and the area needs a fourth idea nobody in
this batch had.** That is the observation that would separate the two hypotheses,
and it is worth running before B2 ships rather than after.

### RED INK — U2 — the image column is total by test and strong only by review

§3.2 says every row publishes "the unique strongest finite set of [ENT-2] facts
entailed by that row's exact semantics". Totality is machine-checkable; strength
is not. A row whose published image is weaker than its semantics entail is a
defect by [ENT-1]'s own words and **nothing detects it**. §5.5 lists the three
partial mitigations and says plainly that none is a proof. The failure mode is
better than today's — a weak image shows up as a writer's claim carrying a token
that is wrong, which is disputable — but "the list can never again be short in a
place nobody thought of" is an overclaim, and judge-2 was right to say so. The
honest statement is: **the list can never again be short in a place nobody
*enumerated*; it can still be shallow in a place someone enumerated badly.**

### RED INK — U3 — the loop ceiling is published, and it is low

Thirty-nine of the corpus's remaining claims — the second-largest family — exist
because a loop head subtracts rather than merges, and this design ships that
subtraction unchanged. It is *published* rather than accidental, which is the
improvement; it is still the largest single thing the checker does not know, and
the writer pays a retained runtime check for each one. Batch B6 is where that bill
comes due, and B6 is the batch that introduces the language's first widening
operator and its first fixed point over `FactState`. Two judges broke the only
version of that rule anyone has written. **Nobody should read this design as
having solved loop induction; it has scheduled it and stated the gates.**

### What is *not* on this list

Three things that look like exceptions and are not, recorded so a later reader
does not mistake them:

- **Parameters seed `TextProduced` while [PRV-1] 3312 seeds a command-entry
  parameter unconditionally external.** Not an exception: three orthogonal axes
  (2746), and [PRV-3] 3406 independently refuses claim authorization of an
  external constrained subject. Judge-1 attacked this seam and found no break.
- **A verified `ensures` does not declassify its returned value.** Not an
  exception but the principle: publisher 2's publication is consumed directly, and
  publisher 3 may not restate it (2745, retained verbatim).
- **The `content` gap.** Not an exception but a published ceiling clause: the
  vocabulary has no per-element term and the component tree has one conservative
  all-elements component.

---

## 12. Open questions for the owner

Each has a recommendation. None is a stopping point for branch work; they are the
decisions I cannot make.

**Q1 — `docs/constitution.md` W3 says a claim is a theorem "over the current
function's own value **and control** authority". This design removes control
authority. Amend the phrase, or read it as already permitting this?**
*Recommendation: read it as permitting, and amend for clarity in the same
approval.* The defended object W3 names in the same sentence is a **result
property** — "a caller may rely on a user callee's result behavior only through
its machine-verified callable boundary … never by using a claim to restate an
unstated or stronger result property" — and the admission state defends exactly
that, in one more place than v0.39 does (call-written storage). A merge over
definitions the function itself wrote *is* the function's own control authority.
I would amend the phrase to "over values this function's own text produced" and
add the sentence that the admission judgment is a necessary condition for
reviewability rather than a truth screen. **If the owner reads "control
authority" as independently load-bearing, §1.1's C-I fails and this design should
be stopped there rather than patched.**

**Q2 — U1: is review an acceptable fence for the laundering family?**
*Recommendation: yes, provisionally, and run F2 before B2 merges.* T3 already
names a wrong review as its failure mode, and the alternatives are a rule that
rejects 63 of 63 true programs (v0.38) or a mechanism two judges broke. But the
concession rests on a claim about reviewers, and a claim about reviewers should be
measured. This is the question I would most want the owner's eye on.

**Q3 — the H3 write seed is amendment-level by the specification's own words
([ENT-1] 2856, [CLM-1] 3243). Approve the amendment?**
*Recommendation: yes, and run B0 first.* The argument is a premise argument, not a
corpus argument: a value a callee produced is a value a callee produced, whether
it arrived as a result or through a `&uniq` actual, and kills and [PRV-1] already
agree. But the exposure is reasoned rather than measured and the corpus contains
zero programs of the shape. A related sub-question the owner should see: closing
the carve-out leaves the writer without the repair every surveyed tradition names,
because FN-9/S12 publishes relations about *results* and not about
`&uniq`-written parameters. Should write-postconditions be opened as their own
design question? *Recommendation: yes, separately; a hole is not justified by the
inconvenience of closing it.*

**Q4 — the three conformance accepts batch 0102 added three days before this
design.** `accept-clm1-local-claim-after-boundary-exit.wf`,
`…-after-boundary-join.wf` and `…-inside-selected-arm.wf` become CLM-2
duplicate-publication rejects in B1, because their predicates are ground or
near-ground remainders the image closure now folds.
*Recommendation: keep the programs, move the verdicts, and add a claim-free accept
sibling for each.* The pair is the corpus's memory of the 63/63 campaign and is
worth preserving as evidence that the shape compiles — which under this design it
does, without a claim. I flag the recency because reversing a three-day-old
approved verdict deserves an explicit eye, not a migration-table row.

**Q5 — the unit of non-duplication and residuality: component or occurrence?**
This design keeps the component, because S3 establishes components and one unit
should carry every judgment. Dropping to the occurrence removes one whole-program
re-analysis per component per claim — the largest single performance item in the
area — and makes every forced edit a whole-statement deletion, at the cost of
admitting a claim one conjunct of which the checker derives, whose retained check
costs P0 forever.
*Recommendation: keep the component, measure the counterfactual cost in B3, and
revisit with a number.* "Fix measured performance problems instead of designing
for imagined scale" cuts against buying the win with a non-uniformity, and nobody
has measured it.

**Q6 — the class rename `Local`/`BoundaryResult` → `TextProduced`/`BoundaryProduced`.**
*Recommendation: rename.* "Local" is the word that made the confusion possible — a
value can be lexically local and boundary-produced at once — and every reader who
reasoned about this area from position rather than from premises was invited to by
that name. Roughly twenty occurrences in the specification, mechanically.

**Q7 — should the ceiling amendment channel be normative, i.e. must an amendment
that raises a ceiling clause *name* the clause and *list* the claims it retires
before it ships?**
*Recommendation: normative for naming the clause, tooling for listing the claims.*
Naming the clause is what makes the token useful and costs one sentence. Producing
the list is a compiler pass over the corpus and belongs in tooling on top of
§3.8's payload, not in the language.

**Q8 — the loop head (U3): schedule B6 now, or after a blind-writer campaign?**
*Recommendation: after F8.* The ceiling is honest either way, and F8's repeated
blind-writer trial is the cheapest available evidence about whether loop induction
or the two-term vocabulary is the real wall once the image closure lands. B6 is
the batch with the soundness bill; it should be aimed by measurement.

---

## Appendix A — the five holes, dissolved

| hole | what it was | what dissolves it |
| --- | --- | --- |
| **H1** control dependence | v0.38 taxed reachability (63/63 false rejections, zero true positives); v0.39 narrowed it to definition-occurrence identity implemented as checked-statement addresses with four unenforced soundness preconditions | Choosing an edge publishes no value, so a selector transfers no subject matter (§1.1 C-I, §3.4). The judgment tracks no artifact identity, so the preconditions have nothing to protect (§4.2) |
| **H2** the delivered-value squeeze | CLM-1 refused by shape; the entailment had no image for a literal carrier; the writer was squeezed between two systems whose boundary nobody derived | Both sides, independently and generally: the gate stops reading the selector, and the entailment's ceiling becomes a closure in which a `give` edge is a value-commit event like any other (§2.2, §3.2, §3.3). No constants exception; §2.2's table shows the next four cases decided by the same rules |
| **H3** the `&uniq` carve-out | a caller could claim a property of bytes a callee's own match chose; kills and provenance saw the write and only claim authority looked away | A callee write produces a value, so it seeds the boundary class — defined as *exactly* what [ENT-5] kill (b) kills, so the element/length behaviour needs no clause and one relation has three consumers (§2.5, §3.4, §4.4) |
| **H4** the redundancy interplay | hard reject with no mechanical fix; every prover gain a breaking change; the finer per-component half unnamed | Two publishers may not publish one premise, so the verdict is derived rather than inherited; the four verdicts become three; the family gains a fix that names the derivation and the content to remove; and the language guarantees the *direction* rather than a stability it cannot promise (§3.1, §3.6, §6) |
| **H5** the rendered diagnostics | unpinned carrier tie-break; a per-rule teaching channel; the redundancy family with no fix at all | One payload, one closed cause set, one fix per cause, each naming which publisher owns the missing premise; the carrier pinned to predicate source order; the gap token; the reviewer's case list; the review note (§3.5, §3.8) |

## Appendix B — the one-paragraph version

Every premise a Whitefoot proof may use is published by one of three publishers —
the entailment, the callable boundary, the reviewed claim — and a premise no
publisher publishes is available to no one. The area's five holes are all one
defect: premises with two owners or none. The entailment's output was an accreting
list of eleven sources, so `give 0_u64` carried nothing, `%` published no range,
`imin` had no source at all and five [SYS-8] contracts sat outside the state;
replace the list with a closure indexed by the operation table and the control
graph, and publish its four-kind complement as the language's stated ceiling.
A boundary publishes values, not edge choices, so a selector contributes nothing
to claim admission and the whole control-dependence apparatus — with the heap
addresses it compares — is deleted; but a callee's write through `&uniq` *is* a
publication, so it seeds the boundary class, defined as exactly what the fact
state's kill already computes. A claim publishes what the ceiling does not, over
values the function's own text produced, naming its gap token; publishing what the
entailment already publishes is refused, and every ceiling raise removes claim
content, names the clause it closed, and can be announced before it ships. What
remains unsolved, and is written in red ink rather than smoothed away, is that a
boundary can still *choose among* values the text produced, and no mechanism this
project can afford separates a true such claim from a false one — so the
specification says so, hands the reviewer the case list, and proposes the
experiment that would move the line.
