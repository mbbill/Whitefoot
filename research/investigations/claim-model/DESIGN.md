# Proof replaces claim: the trap-free language

Design deliverable for batch 0108, superseding batch 0106's *"The claim model,
redesigned: premise ownership"* **in place**. Tree read:
`batch/0106-claim-model-design`, spec **v0.39 ACTIVE**; bare four-digit line
numbers are `spec/kernel-spec.md` at that tip, and every other citation names its
file. Evidence beside this file: `TERRAIN.md` (the rule and prover map),
`CENSUS.md` (the B0 measurement), `F2-REVIEW-TRIAL.md` (the review trial), and
`CLAIM-DISSOLUTION-AUDIT.md` (the 0107 adjudication, whose verdict table,
proposal list and irreducible list I1-I7 this design implements).

Nothing here is implemented and no compiler code was written for it. The
specification text in section 3 is draft text for a work branch, not an
amendment.

**The two pre-implementation falsifiers this file named have been run, each
repair has been re-executed and failed in turn, and this revision is the fifth
repair - the first written as invariants rather than as another repaired
sentence.** 3.8 now opens with the **four laws of the `[IND]` pipeline** - L1
production-time measurement, L2 prover decisions as content-only, L3 a derived
figure for every computation the check performs, L4 the statement's admission
rules staying on the statement - and `[IND-3]`'s scope, `[IND-4]`'s substitution,
`[IND-6]` clause (i), `[IND-7]`'s caps, steps, `RELAX` and acceptance, 3.9.1's
regenerated twenty-four-row sweep and its five-part monotonicity argument are
derived from them in one pass instead of repaired one at a time. F-D4 rewrote the three flagship sources claim-free in full (**MIXED**:
all three compile and are byte-identical over 1,195 differential cases, and five
of 4.5.2's rows were wrong or understated). F-I1 hand-executed `[IND-7]`'s
certificate check (**FAIL**: every drafted derivation reproduced and the rule
around them did not - two soundness breaks, the `[ENT-1]` monotonicity theorem
still false through the caps, and three determinism holes). F-I1b re-executed
everything F-I1 repaired and attacked it with two fresh programs (**FAIL**: both
soundness repairs and every trace hold by execution, and the `[ENT-1]` theorem
was still false, because the first repair made the slot list syntactic at a visit
set the prover still chose). `[IND-4]`'s **shape rule** - a hypothesis's terms
and its slot are present whatever the prover derives, only its content is the
prover's - closed the visit set, and F-I1c confirmed it by execution and by three
fresh attacks (**FAIL** all the same: the theorem was false a fourth time in the
certificate's own **arithmetic**, where `[IND-7]` had imported `[IND-3]`'s
per-step hard error into a space in which the prover decides which steps
execute). The third repair closed the **class** instead of the instance: 3.9.1's
**reachable-error sweep** enumerates every hard error and spec-fixed limit
reachable while a `bound_stmt` is admitted and checked, keeps as hard errors only
those on counts the path's text fixes, and gives every content-reachable one
`[IND-8]`'s discard treatment, so **no hard error is reachable from inside the
certificate check**. F-I1d attacked that class directly and **could not move
it** - the visit set, both caps, the step arithmetic and all five parts of the
monotonicity argument held, and `N3` and every drafted trace re-executed
unchanged - and returned **FAIL** one level *out* instead: the sweep had removed
`[IND-3]`'s limit from the certificate step in exchange for a syntactic bound,
and scoped that rule so narrowly that the shapes the rule itself writes were left
unlimited, so the bound was false on an ordinary program (`N4`) and the
determinism sentence resting on it went with it. The fourth repair widened
`[IND-3]` to every polynomial the shape rule produces, which restored the bound's
premise - and **F-I1e broke it again in the join between two rules** (**FAIL**):
the widened scope was delimited by *carried into the head frame*, and a dropped
path condition is not carried into the head frame, so the pipeline's single
prover-flippable decision decided whether an over-limit shape was measured and an
ordinary program went from verified to hard error under a strengthening (`N5`);
and the `[IND-4]` backward substitution turned out to have no bound anywhere in
the text, with early abort forbidden by cancellation, so an implementation's
capability decided a program exactly as it had on `N4` (`N6`). This revision
answers the pattern rather than the instance: the four laws are stated first and
the rules are derived from them, `[IND-3]`'s limits are tested on every shape the
pass **produces** and **after every substitution step**, the sweep is regenerated
in pipeline order with the pass's own row added, and `N5` and `N6` are priced away
uniformly. 3.9.1 works the six programs underneath the theorem. Section 8's B0
records all six verdicts; the rule text they moved is marked where it moved.

## 0. What changed, and the result in one page

### 0.1 The supersession, stated

The 0106 design kept `claim` and rebuilt it. The owner's charter of 2026-08-29
discards the construct. This file is the same design with its third publisher
replaced, not a new one:

| 0106 | 0108 |
| --- | --- |
| **the principle** - premise ownership, three publishers, disjoint subject matter | **kept verbatim**, with publisher 3 replaced (section 1) |
| publisher 3 = the reviewed claim | publisher 3 = **executed control flow, a verified contract, and a verified induction statement** - all three machine-checked, none trusted |
| `[ENT-3.S5]` the value-commit image closure; the generalised `[ENT-3.S10]`; the image column and its complement | **kept and completed** with every row the 0107 audit requires (3.4, 3.11) |
| `[CLM-1]` gate, `[CLM-2]` non-duplication, `[CLM-3]` partition, the `because` record, the gap token the *writer* spells | **all deleted**; the gap token is **computed by the compiler** (section 6) |
| the loop head published as a subtraction; induction deferred to a gated batch B6 | **`[ENT-5.R]` retention plus `[IND]` the induction statement**, both drafted here (3.6, 3.8) |
| T3 untouched (design section 1.6) | **T3 re-derived**; its premise becomes a property of the accepted set (section 5) |
| U1 red ink: the laundering family is admitted and review is the only fence | **U1 is gone.** With no claim there is no laundering route and no review obligation; `F2-REVIEW-TRIAL.md` becomes the record of why (11.4) |
| 12 open questions | 9 flagged decisions (section 9) and 11 open questions (section 10), of which Q1 is now closed by measurement |
| the two pre-implementation falsifiers were pending | **both have been run, and each repair has been re-run against itself, four times.** F-D4 is MIXED, F-I1 is FAIL, F-I1b is FAIL, F-I1c is FAIL, F-I1d is FAIL - that one on the *determinism* half, with monotonicity and every syntactic row surviving direct attack - and F-I1e is FAIL, in the **join** between the fourth repair's two rules rather than inside either (`N5`, `N6`); 3.8, 3.8.2, 3.8.4, 3.9, 2.4, 4.4, 4.5.2, 6.2, 7.2, 8, 9, 10, 11 and 12.4 carry the repairs. `[IND-4]`'s shape rule is the second, 3.9.1's **reachable-error sweep** the third, `[IND-3]`'s widened scope the fourth, and **3.8's four laws - stated up front and with every rule derived from them - are the fifth**, which is the first repair aimed at the *pattern* the four before it kept reproducing. 11.5 is the red ink they leave |

Section by section, so nothing is lost silently:

| 0106 section | disposition here |
| --- | --- |
| 1 The principle | **carried forward** (1.1-1.5), publisher 3 replaced |
| 1.4, 1.5 the laundering limit and the reviewer's rule | **dropped**; no construct is left for a laundering argument to inhabit, and there is no review record to have a rule about (11.4) |
| 2 The complete case walk (20 rows over claim admission) | **superseded** by section 4, which walks the 50 audit scenarios, the 7 irreducibles, the 6 customers and the corpus instead |
| 2.8 The residual claim | **dropped**; no subject |
| 3 The specification replacement text | **carried forward and completed** into section 3 |
| 3.4-3.6 the claim gate, `[CLM-1]`, `[CLM-2]` | **dropped**; the rules are deleted (3.1) |
| 4 The judgment architecture | **carried forward** into 3.10.7 and 3.14 |
| 5 The prover ceiling | **carried forward** into 3.4, 3.5 and 4.4, with the audit's row corrections applied |
| 6 Non-duplication and residuality | **dropped**; consequence C-III dies with the runtime cost that motivated it (1.3, 2.10) |
| 7 Conformance migration | **carried forward** into section 7 |
| 8 Implementation plan | **carried forward** into section 8, re-batched |
| 9 The attacks the judges landed (on the 0106 drafts) | **superseded** by section 2, which adjudicates the attacks landed on the 0108 drafts |
| 10 Provenance of the ideas | **dropped**; section 2 records provenance where it decides something, and the audit records the rest |
| 11 Unsolved problems, in red ink | **carried forward** into section 11; U1 and U3 leave the list (11.4) and four new entries join it, the last (11.5) written after the falsifiers ran |
| 12 Open questions for the owner | **carried forward**, split into section 9's decisions and section 10's questions |
| Appendix A, the five holes | **carried forward** into 1.4 |
| Appendix B, the one-paragraph version | **dropped**; section 0.2 replaces it |

### 0.2 The result

Delete `claim` and the language loses **no discharge route any accepted program
uses**, because each of the six things a claim could discharge is dischargeable
by a guard, a contract, a published operation image, a retained loop fact, or a
verified induction statement - and every one of those is a fact the checker
itself derived or verified. Section 4's case walk is the demonstration: 50 audit
scenarios, 7 irreducibles, 6 customers, 135 corpus claims, 20 corpus sites walked
one at a time.

Four things become true that were not:

1. **`[ENT-1]` version monotonicity becomes unconditional.** Today the
   specification grants monotonicity to every discharged operation, call goal and
   selected-return relation and withdraws it from claims in the same sentence
   (2853-2855). `claim` is the only construct in the language whose acceptance
   *shrinks* when the prover gets stronger. After the deletion no construct's
   admission depends on a fact **not** being derivable - and section 2.4 is the
   adjudication that had to be won to keep that sentence true.
2. **`[SCOPE-4]` becomes a trap-freedom statement rather than a trap
   definition.** An accepted program has no writer-reachable runtime contract
   violation at all.
3. **`[PAR-1]`, `[PAR-2]` and `[PAR-3]`'s identity guarantees become
   unconditional** (3.13), and the staged permission judgment loses a read
   footprint and a non-continuing edge per former claim site. That is the
   parallelism the charter asks for, and it is a deletion rather than a feature.
4. **Deterministic verification removes branches from the emitted code.** Every
   worked loop in section 3 loses one comparison or one never-taken arm relative
   to the spelling that compiles today.

The cost is one honest bill, stated in red ink in 11.1: where a fact is true and
underivable, the writer writes a branch whose false edge cannot be taken, and in
value position invents a value for it.

### 0.3 What is compiled and what is read

Every verdict marked *(compiled)* was run against the gate-profile `whitefootc`
built from this tree at spec v0.39. The design draws on 160 audit probes, 16
`t*` probes, 26 `L*` probes, 25 `c*` probes, 7 judge probes and my own 4; all of
them were re-run for this file and all reproduce. Section 12 is the consolidated
ledger and says, row by row, what compiles and what is a prediction about a rule
that does not exist.

The four probes that are mine are the ones that changed a decision:
`y1`/`y2` (a compiled separating pair refuting the drafted P-LOOP candidate
definition), `y3` (the loop it breaks), and `y4` (the one claim customer no part
of the batch had witnessed).

## Contents

1. [The principle](#1-the-principle)
2. [The adjudication](#2-the-adjudication)
3. [The construct catalog](#3-the-construct-catalog)
4. [The complete case walk](#4-the-complete-case-walk)
5. [T3 and W3, re-derived](#5-t3-and-w3-re-derived)
6. [Diagnostics and teaching](#6-diagnostics-and-teaching)
7. [Conformance and activation](#7-conformance-and-activation)
8. [The implementation plan](#8-the-implementation-plan)
9. [Flagged decisions for the owner](#9-flagged-decisions-for-the-owner)
10. [Open questions, each with a recommendation](#10-open-questions-each-with-a-recommendation)
11. [Honest limits, in red ink](#11-honest-limits-in-red-ink)
12. [Probe ledger](#12-probe-ledger)

---

## 1. The principle

### 1.1 Stated once, with the third publisher replaced

> **Premise ownership.** Every premise a Whitefoot proof may use is published by
> exactly one of three publishers, and each publisher's output is fixed by this
> specification.
>
> 1. **The entailment** publishes what the operation table and the control graph
>    entail about values this function's own text produced.
> 2. **The callable boundary** publishes what a machine-verified contract
>    (`requires`, an `[FN-9]`-verified `ensures`, an `[FN-10]`-verified write
>    postcondition) or a specification-fixed operation contract states about
>    values a callee or the world produced.
> 3. **The writer's verified statement** publishes what the writer said about
>    this function's own values *and the checker then proved*: an executed
>    branch condition, and a verified induction statement over a loop or a local
>    computation.
>
> A premise no publisher publishes is available to no one: not to the checker,
> not to the writer, not to a reviewer.

The 0106 file's publisher 3 was *the reviewed claim*: one always-true lemma,
taken on a human's word, backed by a retained runtime trap. That is the only
sentence of the principle that moves, and moving it is the whole batch.

### 1.2 What the replacement buys, stated as the publication law

After the deletion the fact-source list `[ENT-1]` 2831-2836 governs reads, with
nothing removed but the writer's *word*:

> Every fact source `[ENT-3]` is an executed control condition, a requirement
> statically proved by every ordinary caller before S4 admits it to a body, a
> declared allocation or type property, a constant, an operation row's published
> image, S11's compiler-owned structural consequence, S12's machine-verified
> normal-result publication, or a machine-verified induction statement.

One kind left the list and one joined it, and the *shape* of the list changed
qualitatively: **every source is now something the compiler itself observed or
verified.** Before the deletion, exactly one entry was a writer's word taken on
trust; after it, the entry in that position is a statement the checker proves
before it is a fact.
That sentence is the whole content of section 5's W3 amendment, and it is why
T3's derivation gets stronger rather than weaker.

### 1.3 The four consequences, re-derived

**(C-I) Subject matter, not position.** Unchanged in force and simpler in
statement. Publisher 3 may speak only about publisher 1's subject matter -
values this function's own text produced - but with the claim gone the rule no
longer needs a *gate* to enforce it. A verified statement cannot smuggle a
premise across a boundary: if the base and step do not derive, it is rejected,
whatever its support reads. **`[ENT-6]`'s entire claim-authority analysis
(3215-3248, and `compiler/src/semantic/claim_locality.rs`, 2122 lines) is
deleted with no replacement**, and that is the single largest simplification the
deletion buys.

**(C-II) Publisher 1's output is a closure and its complement is published.**
Unchanged, and now load-bearing for a second reason: with no claim to stand over
the ceiling, a ceiling clause is a program the writer must *restructure*, so the
complement has to be honest. Section 3.4 carries the closure forward; section 6
computes its tokens instead of asking the writer to spell them.

**(C-III) Two publishers may not publish the same premise.** This consequence
**dies with the claim, and its death is a result rather than a loss.** The rule
existed because a duplicate publication cost a retained runtime check on every
reach (2748). An induction statement costs nothing at runtime, so a statement
that publishes what publisher 1 already publishes costs nothing, and
`[IND-9]` says so normatively. Section 2.10 records the owner's ruling and the
asymmetry that derives it.

**(C-IV) Every refusal names the publisher who owns the premise the writer
needs.** Unchanged, strengthened, and now mechanical: the writer no longer
spells the gap token in a `because` record, because there is no record - the
compiler computes it (section 6).

### 1.4 The five holes, and the one that closes itself

The 0106 file's central table said the five charter holes were one defect.
Under the deletion four of them stay closed by the same mechanism and the fifth
stops existing:

| hole | disposition at 0108 |
| --- | --- |
| H1 - edge choice treated as a publication | closed by the value-commit image closure (3.4): one operation committed to one destination establishes one image however it is spelled |
| H2 - a delivered literal had no publisher | closed by the same clause; a `give` edge is a value-commit event |
| H3 - a callee-written value had no owner | closed by `[FN-10]`, the write postcondition (3.10), which is publisher 2 finally able to speak about what a callee wrote |
| H4 - two publishers publishing one premise | **dissolved.** With no runtime cost to duplication there is nothing to arbitrate |
| H5 - the writer was never told which publisher owns what they lack | closed by the computed gap token and its seven-row fix table (section 6) |

### 1.5 The one thing the principle still does not decide

Ownership decides who may speak about a value. It does not decide how *much* the
entailment can say about it. The complement of publisher 1's closure - the
prover ceiling - is a real ceiling, and the 0106 file's four gap kinds survive
as four of the seven tokens in section 6.1. What has changed is the writer's
recourse when they hit it: under 0106 they wrote a claim and a human validated
it; under 0108 they write a branch, a contract, or a statement the machine
validates, and if none of the three reaches the fact they **restructure the
program**. Section 11.1 states that bill in red ink and prices it in three
tiers.

The 0106 file's *laundering family* - a claim whose truth depends on which arm a
boundary selected, admitted by the subject-matter gate and refusable only by
review - has no successor and needs none. There is no construct left that a
laundering argument could inhabit. `F2-REVIEW-TRIAL.md`, which measured whether
reviewers catch such a claim, is kept beside this file as the evidence for why
the model that needed it is gone (11.4).

---

## 2. The adjudication

Two adversarial judges read the three 0108 part-designs and landed nineteen
distinct flaws, nine of them named fatal. I adjudicated every one against the
quoted rule text and, where a program could arbitrate, against the compiler.
**Every judged fatal is upheld except one, which is upheld in part.** Four are
memory-unsafety admissions; one falsifies the batch's headline theorem; five are
seams between the parts. The repairs are in section 3 with the rules they
repair; this section is the verdict, the evidence, and - for the four that
needed a design decision rather than a side condition - the reasoning.

I also record three findings of my own that neither judge made, one of which is
a compiled separating pair that changes a central rule.

### 2.1 The verdict table

| id | flaw, in one line | verdict | evidence | repair, and where |
| --- | --- | --- | --- | --- |
| **A1** | `[IND-4]`'s wrap/sat proviso is vacuous: a `wrap` row has no `[ENT-6]` domain obligation, so `a -wrap b` substitutes to `a - b` unconditionally | **upheld** | `j01_wrap_total.wf` **accepts** (compiled): `let room = a -wrap b;` with no guard and no obligation. `j02` (compiled) rejects the shape the drafted rule would admit | 3.8, `[IND-4]` clause (b): the *corresponding exact row's* `[OP-2]` no-wrap side condition must be derivable at that commit before the commit's exact value is available at all |
| **A2** | `[IND-4]`'s "substitution applies identically to every hypothesis" makes the step self-cancelling, admitting a false invariant and a silent unsigned overflow | **upheld** | hand arithmetic, reproduced independently: `p := 255*p0 - 255*H1 = -65025`, verified, for a loop whose accumulator grows by 1000 | 3.8, `[IND-6]` the frame rule: statement polynomials are head-frame facts and are never substituted |
| **A3** | P-OFF's `x` row publishes `r - a >= al*(bl-1)` with no `bl >= 1` condition, so `a in [2,10]`, `b = 0` publishes `a >= 10` | **upheld** | `j03_poff_mul_unsound.wf` (compiled) rejects exactly the obligation the false fact would discharge; `harm(a: 2, b: 0)` satisfies every `requires` and underflows | 3.5, `[ENT-3.S5.O]`: publish the `x` rows only when the state derives `Z - b <= -1` |
| **A4** | `[IND-7]`'s greedy elimination is non-monotone under prover strengthening, falsifying `[ENT-1]`'s rewritten monotonicity sentence | **upheld** | hand construction over three terms; a new row image inserts a hypothesis earlier in a fixed order and displaces the useful one | 3.9, `[IND-7]` replaced by a **certificate** form; 2.4 is the reasoning |
| **A5** | the batch names P-COMMIT, P-DOM, P-ROW's `%`, `/`, `imin`/`imax`, P-MONO, the corrected `*wrap` box image and the `ior`/`maxor` image as hard prerequisites and drafts none of them | **upheld in part** | correct about the three part-designs; **wrong about the batch**: `[ENT-3.S5]`, the generalised S10, the image column and its `%`, `/`, `imin`/`imax` and bit rows are drafted in the 0106 file this one supersedes (its 3.2 and 5.3) | 3.4 and 3.5 carry them forward and complete them; 2.5 records the correction |
| **A6** | three parts, three edge-ordering texts, one of them v0.39's order, which both siblings call mandatory to reverse | **upheld** | spec 3095 quoted: "each taken after that edge's scope-exit kills and then closed"; `r7_closure_vs_kill.wf` (compiled) rejects `best < len(data)` | 3.3, one rule id `[ENT-5.P0]`; `[ENT-5.X1]` is deleted as a duplicate and 3.14's pipeline is rewritten |
| **A7** | `[FN-10]`'s flagship clause `ensures wrote(deref(at): next): ige(next, deref(at))` cannot be established at the caller: the entry image of the caller's place is dead after the callee-effect kill, and `[ENT-2]` gives no snapshot term | **upheld** | argument over quoted `[FN-10.E]`/`[FN-10.K]`/`[ENT-2]` text; the same defect as D7 in the other direction | 3.10, `[FN-10.A]`: a write clause is admitted only over operands the call does not disturb. The snapshot term is flagged decision D6 |
| **A8** | the unlabelled induction statement has no rule text: no path, no start, no stop, no depth bound | **upheld** | the sibling file designing the same route fixes a depth of 8 "because `[ENT-1]` forbids an implementation-chosen limit"; the other bounds nothing | 3.9, the **local** form, drafted in full with a straight-line region restriction |
| **A9** | `[IND]` has no proof-view discipline, so a statement resting on S4 publishes in the S4-blinded view and `[PRV-3]`'s external fence is defeated | **upheld** | `[PRV-3]` 3392 and `[FN-9]`'s per-view aggregates quoted; the drafted `[IND-8]` has no counterpart | 3.9, `[IND-8.V]`, mirroring `[FN-9]`'s `Cq`/`Bq` |
| **A10** | the generalised `[ENT-3.S10]` mandates establishing `next = start + required`, a three-term relation `[ENT-2]` 2901 cannot hold | **upheld** | spec 2577 and 2901 quoted; the escape clause reaches a bad *operand*, not a bad *arity* | 3.11, `[ENT-3.S10]` gains an explicit arity projection |
| **A11** | the contract part still queries the **U** view the core part deletes | **upheld** | `[PRV-3]` 3404 defines U as the complete state minus S3; with S3 deleted `U == complete` pointwise | 3.10 and 3.14: two views everywhere |
| **A12** | `[FN-10]`'s disjointness tie-break is a silent-loss rule its own author recommends against | **upheld** | the drafted rule and its own Q3 contradict each other | 3.10, `[FN-10.A]` clause (e): reject at admission |
| **A13** | the redundancy-note channel is homed in `[DIAG-3] 1985`, a rule the same file deletes | **upheld** | quoted from both sections of the same file | 6.4: homed in `[DIAG-1]`, with a sentence forbidding it from ever becoming a verdict |
| **A14** | deleting `[TRAP-1]` wholesale drops a lowering-shape guarantee `[QUAL-3]` cites, and silently retires a deferred amendment | **upheld** | `[TRAP-1]`'s "no instance resource table, per-instance reaper, or pending-operation transfer ... on an `inline-terminal` transfer path `[QUAL-3]`" is not about the claim's own execution | 3.13: re-derived into `[QUAL-3]` from the new `[SCOPE-4]`; the deferral's withdrawal is a META-5 line |
| **A15** | the `gap` token table never names the batch's own new construct, and the one token that fires on loops teaches a fix that cannot work | **upheld** | clause 4 fires exactly on the candidates P-LOOP **deleted** - facts that are not inductive - and tells the writer to state them as an induction statement, whose step will fail on the very path that killed the candidate | 6.1: a seventh token `induction`, and clause 4 rewritten |
| **A16** | `[IND-6]` checks the step in the **body-exit** state while `[IND-4]`'s substitution expresses the polynomial over **head** values, verifying a false invariant and admitting an out-of-bounds write | **upheld** | `j3_ind6_checkpoint_break.wf` (compiled) rejects `x < len(out)`, the single obligation the false publication discharges; `j3b_ind6_consumer.wf` (compiled) **accepts** with that fact supplied. The pair is the break | 3.8, `[IND-6]` the frame rule; and 2.3 |
| **A17** | `[IND-4]`'s "until no destination written on the path remains" forces re-substitution, a second independent unsoundness and a determinism defect | **upheld** | on the same program it rewrites `p = cursor` to `p = 0` by a different route; and two readings of "until" accept different programs | 3.8, `[IND-4]`'s backward-pass sentence: **one pass, no revisiting** |
| **A18** | `[FN-10]` verifies a write clause on `Ok` returns and establishes it on **every** normal continuation, so an `Err` return leaves the place unconstrained and the caller reads past the end | **upheld** | `j2_fn10_err_hole_shape.wf` (compiled) is that program minus the clause, rejecting on exactly `cursor < len(values)` | 3.10, `[FN-10.V]`: a write clause selects **every** return |
| **A19** | P-LOOP's candidate set is the **tightest** entry bound per ordered pair, so it can never retain a weakening - and the inductive bound of an ordinary counting loop is almost always a weakening | **upheld**, and I compiled the separating pair | `[ENT-2]` 2901 and `[ENT-4]` rule (3) quoted: a closed state holds one bound per ordered pair. **`y1_entry_tight_step.wf` rejects** and **`y2_relaxed_step.wf` accepts** (both compiled) on the same step obligation, the only difference being whether the head fact is the entry-tight atom or its weakening. `y3_const_extent_loop.wf` (compiled) is the loop | 3.6, `[ENT-5.R2]`: the **constant ladder**; and 2.6 |
| **A20** | `[ENT-5.R3]` freezes the candidate universe in the no-retention flow, so an inner loop can never receive a fact only an outer loop's retention makes available | **upheld** | `fir_filter.wf:45` is that program and is in the same file's own migration table; the audit's own P-LOOP text does not freeze the universe | 3.6, `[ENT-5.R3]`: an ascending universe iteration, which **terminates because the ladder makes the universe a subset of a syntactically fixed finite set** |
| **A21** | the generalised `[ENT-3.S10]` silently drops spec 2988's `[PRV-1]` dependency sentence, defeating the world-value fence the whole design leans on | **upheld** | spec 2988 quoted: "Each result endpoint's `[PRV-1]` dependency additionally includes the concrete start actual, so this relation never launders an external start into an internal result." The drafted replacement has four sentences and that is not one | 3.11: restored and generalised to every actual the source names |
| **A22** | two notations and two verification procedures for I1 - `bound` with a substitution-and-elimination check, and `prove` with an unfold-to-depth-8 check and a new 60-100 row denotation column | **upheld** | the two cost accounts are an order of magnitude apart in one deliverable | 3.9: one construct, `bound`; `prove` and the denotation column are **not adopted**; 2.7 is the reasoning and 3.9's finding on data-dependent probes is kept |
| **A23** | the four bucket-B corpus claims get three mutually exclusive dispositions, and two of the three rest on a factual claim a compiled sibling probe refutes | **upheld** | `t4`, `t8`, `t10` all **accept** (re-run, compiled): T1, T2, T3 and R1 dissolve today with a guard and no new rule | 4.5: one disposition per claim; the statement and the backward wrap rows both lose their necessity argument, and 2.8 states what that costs |
| **A24** | seven determinism gaps: circular term/hypothesis order; `RELAX` undefined on a mixed-sign degree-2 monomial; `[IND-8]`'s `m` ambiguous; cross-type difference bounds have no `[ENT-2]` producer; `[ENT-3.S5.B]`'s reading point pinned only in prose; `[ENT-5.R6]`'s monotonicity argument omits `B`; the META-5 delta is never summed | **upheld, all seven** | `j4_mixed_type_compare.wf` (compiled) shows `ile(a_u32, b_u64)` is `[TYPE-5] TypeMismatch` today, so `[IND-8]` would be the first producer of a fact class `[ENT-2]` has never carried | closed at 3.9 (order dissolved by the certificate form; `RELAX` corners; `m`; the cap), 3.9's `[IND-8.T]` (the `[ENT-2]` amendment), 3.5 (the reading point nailed to the event), 3.6 (`B` monotone in `R`), and 7.3 (the summed delta) |
| **A25** | smaller, all upheld: `[ENT-5.R4]`'s definition is circular against its own algorithm; `[IND-8]`'s constants escape the `2^127` cap; `[IND-3]`'s `ine`-as-hypothesis clause is dead text; I4's closure is narrower than its headline; `[ENT-2]` 2891's amendment puts a statement in the *goal* universe; `[SYS-8]`'s SystemRange customer is unwitnessed | **upheld** | the last one I closed: **`y4_systemrange_guard.wf` accepts** (compiled) | 3.6, 3.9, 3.12, and 12 |

### 2.2 My own findings, which neither judge made

**S1 - `[IND-4]` as drafted refuses its own flagship example.** The
substitution walks backwards "replacing, at each `let` or `set` commit whose
destination occurs in the polynomial, that destination by the polynomial of the
commit's right-hand side, until no destination written on the path remains", and
"a subscript ... **refuses** the statement". The I2 worked example is

```whitefoot
let w = deref(weights)[i];
let wide = cvt<u8, u32>(w);
set sum = sum + wide;
bound @weigh per_byte: ile(sum, 255_u32 * i);
```

and its trace keeps `wide` in the polynomial and relaxes it with `cu(wide) =
255`. But `wide` **is** a destination written on the path, so the rule replaces
it by its `cvt` operand `w`; `w` is also a destination written on the path, and
its right-hand side is a **subscript**, so the rule refuses. **The rule text and
the worked example contradict each other, and the rule text refuses the
example.** The repair is not to weaken the refusal: it is to say what a
path-local value *is*. Section 3.8's `[IND-4]` clause (e) gives an unsubstitutable
`let` commit a **fresh opaque witness term** with its commit-point bounds as
hypotheses - the device `[IND-4]` already uses for `a / k` - and reserves refusal
for an unsubstitutable `set`. With it, the I2 trace runs unchanged with `w`'s
witness in place of `wide`.

**S2 - A5 is overstated, and the correction matters for scheduling.** All three
part-designs assign the value-flow publishers to a fourth designer and both
judges conclude the batch is unmergeable. The publishers are drafted: the 0106
file this one supersedes contains `[ENT-3.S5]` (the value-commit image closure -
the audit's P-COMMIT plus its frame), the generalised `[ENT-3.S10]`, the
`[ENT-1]` ceiling clause and complement, and the row list naming `%`, `/`,
`imin`/`imax` and the bit rows with their images. What is genuinely missing is
narrower and I draft it here: **P-DOM** (3.5), **P-MONO/P-OFF corrected**
(3.5), the **corrected `*wrap` box image** and the **`ior`/`maxor` image** (3.5),
and the enumeration discipline that makes the column total (3.4). The batch is
one merge, as the core part says; it is not missing a designer.

**S3 - `[ENT-3.S10]`'s widening is two changes wearing one sentence.** The
drafted generalisation changes *which operations* the source covers (five named
operations become every `[SYS-2]` operation) **and** *how much* it imports (two
endpoint facts become every `[SYS-8]` relation). The second is argued; the first
is not argued anywhere. I keep both, because the accreting-list objection applies
to the operation list exactly as it applies to the relation list, but I add the
obligation that makes the widening safe: each `[SYS-8]` contract's admitted
projection is **enumerated in the same change**, exactly as an operation row's
image is (3.11). Without that, the widening replaces one accreting list with an
unenumerated one.

**S4 - the compiled separating pair for A19.** Both judges reasoned about
P-LOOP's candidate definition from rule text. The question is decidable today,
because the step obligation of a candidate can be posed as a contract:

```whitefoot
fn step_tight(tap: own u64, cap: own u64) -> result: own u64 pure contract {
  requires ile(tap, 0_u64);      // the ENTRY-TIGHT atom: tap = 0, cap = 8
  requires ine(tap, cap);        // the loop's own exit test, negated
  requires ieq(cap, 8_u64);
  ensures ile(result, 0_u64);
} { let next = tap +wrap 1_u64; return next; }
```

`y1_entry_tight_step.wf` **rejects** (compiled). The same function with the
head fact weakened to `ile(tap, cap)` and the conclusion to `ile(result, cap)` -
`y2_relaxed_step.wf` - **accepts** (compiled). The pair says, with the compiler
rather than by reading, that **the entry-tight atom is not inductive and its
weakening is**, and `y3_const_extent_loop.wf` (compiled, rejects `tap <
len(taps)`) is the ordinary constant-extent walk that needs it. This is the
evidence that turns A19 from a plausible reading into a rule change.

### 2.3 A16 and A2: the frame, which is one defect wearing two costumes

The step check has to answer one question: *for every execution that enters the
body at a head satisfying the hypothesis and follows path p to the back edge,
does the statement hold of the values at the next head?* The goal is naturally
about **exit** values and the hypothesis about **head** values, so exactly one
translation is needed and every quantity in the check must end up on the same
side of it.

`[IND-4]`'s backward substitution translates the goal into the head frame. That
is the right choice and it is correct as an algorithm - a single backward pass
leaves every occurrence denoting its path-entry value. Two drafted sentences then
read quantities in the wrong frame:

- `[IND-6]` reads the state at the **body exit**, so a term the body writes later
  on the path is bounded by its exit value while the polynomial means its head
  value. That is A16, and `j3`/`j3b` compile the harm.
- `[IND-4]`'s closing sentence substitutes **every** hypothesis, including the
  statement's own polynomial - which is a head-frame fact and needs no
  translation. Substituting it makes the hypothesis equal to the goal modulo the
  binder shift, and the body's effect cancels. That is A2. (The repaired rule
  draws the line where A2 requires and F-I1's F2 requires: the backward pass
  rewrites the hypotheses **the pass itself introduced**, which are path-local
  and must be carried into the head frame, and never a statement polynomial,
  which is already in it.)

So the repair is one rule and it fixes both: **name the frame, put everything in
it, and say what happens to a term that has no value in it.** Section 3.8's
`[IND-6]` does that in four clauses, and section 3.8's `[IND-4]` clause (e)
handles the last part (S1) by turning a path-local into a witness term rather
than by refusing.

### 2.4 A4: the monotonicity break, and the repair that keeps both laws

This is the adjudication that decides whether the batch may keep its headline
sentence, so it gets the reasoning in full.

**The break.** `[IND-7]` eliminates one term at a time against "the **first**
unused hypothesis whose coefficient `b` on `t` satisfies `a*b > 0`", where the
hypothesis order enumerates, "for each ordered pair of terms in term order, the
tightest difference bound derivable at the check point, **if any**". A stronger
checker derives a bound on a pair where the earlier one derived none; that bound
is inserted at its fixed position, which may **precede** the one the earlier
checker selected, and it displaces it. Terms `a`, `b`, `c` and goal `a - c + 1
<= 0`: version 1 has no bound on `(a,b)`, selects `a - c <= -1`, residual 0,
accepted; version 2 derives `a - b <= 100`, `(a,b)` precedes `(a,c)`, residual
becomes `b - c + 101` and then 101, rejected. **A program that compiled under
v0.40 does not compile under v0.41.** That is the exception `[CLM-2]` had,
reintroduced by the construct that replaces the claim, and worse: `[CLM-2]` fired
on a predicate the writer could delete, while here the writer must re-derive a
statement against an elimination order they cannot see.

**The three repairs on the table, and why two fail.**

- *Freeze the fragment.* Declare `[IND-7]` an amendment-level surface, so
  changing it is an accepted-set change like changing the grammar. This does not
  work, because the break is driven by **ambient prover strengthening** - adding
  a row image changes which difference bounds are derivable at the check point -
  which is exactly the "implementation strengthening" `[ENT-1]` promises is safe.
  The theorem would have to be weakened to "strengthening preserves acceptance
  provided it does not change what is derivable at an `[IND-7]` check point",
  which is no theorem at all.
- *Make the hypothesis list syntactically total*, so a strengthening tightens an
  existing slot but never inserts one - for instance by filling an underivable
  slot with the type-implied bound `cu(t1) - cl(t2)`. Monotone, and it **breaks
  I1**: the midpoint derivation's second elimination consumes `lo - hi <= -1`
  and depends on the slot `(q, hi)` being *absent*; filled with `max(u64)` it is
  selected instead and the derivation dies. The repair costs the one irreducible
  the owner asked to be examined. (F-I1 records that the certificate form
  dissolves this objection as a side effect: with the `(q, hi)` slot present the
  midpoint certificate still exists, because nothing forces `sigma` to name it.
  The option stays unadopted - it is simply no longer the one that costs I1 -
  and 3.9.1's repaired slot list is syntactically total in exactly this sense,
  with the slots present and possibly empty rather than filled with a
  type-implied bound.)

**The repair I adopt: make the check a certificate rather than a sequence.**
`[IND-7]` stops prescribing *which* hypothesis eliminates each term and instead
asks whether **some** assignment of hypotheses to terms, drawn from a
syntactically fixed finite list and bounded by a spec-fixed cap, drives the
relaxed residual to zero. Section 3.9 gives the text. Four properties decide it:

1. **Monotone, and provably so - but only after F-I1's repair.** Tightening a
   hypothesis constant makes `-lambda*h` smaller, so the residual is smaller and
   a succeeding certificate still succeeds. Adding a hypothesis only enlarges the
   certificate space and every prior certificate survives with that hypothesis
   unused. **That argument was not enough as drafted**, and F-I1 is where it
   broke: the two caps that keep the search affordable were hard errors on counts
   a strengthening can grow - a thirteenth filled ordered-pair slot, or a `wrap`
   commit whose exact route replaces one witness term with two operand terms - so
   a v0.40 program could still fail to compile on v0.41. A4 had moved from the
   elimination order to the caps rather than being closed. 3.9.1 repairs it by
   making both counts **syntactic**: every ordered-pair slot is present whether or
   not a bound is derivable, every non-exact commit contributes exactly one
   witness term whether or not the no-wrap side condition derives, and `[IND-4]`
   clause (a) is unconditional. **That was still not enough, and the second
   falsifier run is where it broke again**: fixing the slot list at a fixed visit
   set left the *visit set* under the prover's control, because clause (b)'s
   equality pair entered the pass's reachable terms only when its side condition
   derived, so a strengthening could drive the backward pass into a commit the
   weaker checker never reached - into clause (e)'s `set` refusal, or into four
   more slots. `[IND-4]`'s **shape rule** is the close: a hypothesis's terms and
   its slot are present whatever the prover derives, and only its content is the
   prover's, so the visit set, the elimination-term list and the slot list are
   all computed before a fact is consulted. **That was still not enough, and the
   third falsifier run is where it broke again**: the space was the same set at
   both versions but the *work done inside it* was not, because which certificate
   steps execute is decided by which slots the prover filled, and `[IND-7]` had
   imported `[IND-3]`'s per-step magnitude limit - a hard error - into exactly
   that arithmetic (`N3`). The close is a **sweep** rather than a fourth patch:
   3.9.1 enumerates every hard error and every spec-fixed limit reachable while
   a `bound_stmt` is admitted and checked, keeps as hard errors only those on
   counts the path's text fixes, and gives every content-reachable one
   `[IND-8]`'s discard treatment, so **no hard error is reachable from inside the
   certificate check at all**. With that, *no prover strengthening can reject a
   statement an earlier checker verified*, and `[ENT-1]`'s new sentence holds
   without exception - which is the theorem this batch is for, so it is stated
   there in five parts with the case analysis written out and with the three
   programs (`N1`, `N2`, `N3`) that broke it worked underneath. **The fourth
   falsifier run attacked all five parts directly, including the covering lemma
   three ways, and broke none of them**; what it broke was property 4 below.
   **And that repair broke the theorem a fourth time, in the join between two
   rules rather than inside either**: the widened `[IND-3]` was delimited by the
   phrase *carried into the head frame*, and a **dropped** path condition is not
   carried into the head frame, so whether an over-limit shape was measured
   turned on clause (b)'s `set` refusal - the one decision a strengthening can
   flip (`N5`). The fifth repair stops writing scopes and writes **invariants**:
   3.8's four laws, from which `[IND-3]`, `[IND-4]`, `[IND-6]`, `[IND-7]`, the
   sweep and this argument are all derived. L1 is the one that closes `N5` -
   what is measured is what the pass *produces*, so a drop, a fill and a refusal
   are content moves and none of them reaches a limit.
2. **It accepts a superset of the greedy rule, over a fixed hypothesis list.**
   Every greedy elimination sequence *is* a certificate, and F-I1 confirms the
   inclusion and could construct no counterexample. **The superset is over one
   hypothesis list, not over the two rules' accepted sets**: `[IND-5]`'s and
   `[IND-10]`'s hypothesis sets are strictly smaller than the drafted ones, so
   the accepted set is *not* a superset of the drafted rule's, and `B1`'s vacuous
   bases - every labelled statement proving itself - are exactly what is
   deliberately lost (3.9.7). So none of **the derivations this file actually
   drafts** - the **seven** of 3.9 (I2's base and step, I3's base and step, I4's
   base and its step, and I1's midpoint; I4's step is one derivation over two
   body paths, which is how 12.4's ledger counts it; F-I1 executed six of the
   seven, and I3's base is written down here by the same repair) and the two
   refusal traces of 3.8.3 - has to be re-derived for the change of rule. **The
   drafted version of this sentence named two more that do not exist**: there is no `[IND-7]`
   derivation of the four bucket-B statements anywhere in this file, because 2.8
   routes all four to guard rewrites, and none of the counted ipv4 restructure,
   which 4.4 now prices as unreachable and 2.8 routes to the pair guard. The
   superset argument covers the traces that exist and nothing else.
3. **It dissolves three other defects.** The circular term/hypothesis order
   (A24's D-1 and D-8) has nothing left to be circular about, because no order
   fixes the accepted set. The loop file's own Q4 - "a different order accepts a
   different, incomparable set of statements, and F-L4 is the experiment that
   would move this" - stops being an open question.
4. **It is deterministic.** Acceptance is an **existential over a spec-fixed
   finite space**: the obligation verifies when at least one certificate in the
   space succeeds. The caps and their hard errors are on the *size* of that
   space, and nothing inside it raises - every way a certificate can fail is a
   failure or a discard confined to that certificate - so **enumeration order
   cannot change the predicate**, and an implementation may stop at the first
   success or evaluate all of them. Two conforming implementations **that derive
   the same facts at the check point** compute the same answer on the same
   inputs, and property 1 is what covers the case where they do not: the *visit
   set* and the slot *positions* are fixed by this rule, the slot *contents* are
   whatever the ambient prover derives, and a slot that fills or tightens never
   loses a certificate. F-I1 is why that sentence carries its qualifier; the
   unqualified version was false, since group 3 and `RELAX` both read the ambient
   prover. The second run is why the visit set is named beside the slot
   positions, and the third is why the order sentence is here at all - with a
   hard error live inside the space, whether it fired depended on which
   certificate an implementation tried first. **The fourth run refuted this
   property outright, and it is repaired here**: order was not the only
   implementation-chosen freedom the rule contained. `[IND-7]` also said a
   certificate an implementation *cannot evaluate* does not succeed, which is a
   **capability** choice, and the resource bound that was supposed to make it
   dead letter rested on a scope `[IND-3]` had narrowed past its own hypothesis
   shapes. On `N4` (3.9.1) an exact implementation verified and one sized to the
   published bound raised a hard error, both conforming. So determinism rests on
   **two** legs, and the file now says so: the order argument, and a resource
   bound whose premise is `[IND-3]`'s widened scope. `[ENT-1]` 2835-2836 forbids
   an implementation-chosen *strategy*, not only an order, and `AGENTS.md` says
   the same from the other side - compiler capability is not a source-language
   rejection. **The fifth run then found the same defect in a computation that
   had no figure at all**: `[IND-3]`'s limits were on the shape rule's *output*,
   nothing bounded the backward substitution's intermediates, and an
   implementation may not abort early because cancellation can restore legality,
   so on `N6` one implementation cannot finish and one that stops early rejects
   what the first accepts. Determinism therefore rests on **L3 as a law** - every
   computation carries a derived figure, the pass included - and the pass's
   figure is supplied by construction: `[IND-3]`'s limits are tested after every
   substitution step, so nothing over-limit is ever materialized.

**The one thing to flag.** `[ENT-1]`'s law is quoted as *closed, deterministic,
search-free*, and an existential over a finite space is, literally read, a
search. I hold that it is legal - the prohibition is on **implementation-chosen**
strategy, and every conforming implementation computes the identical predicate,
which is the property 2836 actually demands - but the owner should rule.
**Flagged decision D1** (section 9) states both readings, prices the fallback
(freeze the fragment and weaken the theorem), and records that I adopted the
certificate form.

### 2.5 A5: what the batch was actually missing

Recorded so the scheduling conclusion is not carried forward wrong. Both judges
conclude, correctly from the three files in front of them, that "the batch as it
stands cannot be merged". The premise is that seven publishers are drafted
nowhere. Five of the seven are drafted in the 0106 design being superseded, and
this file carries them forward with their corrections (3.4, 3.5, 3.11). What was
genuinely undrafted anywhere in the project is:

| item | why it was missing | drafted at |
| --- | --- | --- |
| **P-DOM**, the two-nonconstant `.defined` route | the audit named it as a `[ENT-6]` normalization route and no design wrote the sentence; `L08`/`L09` (compiled) reject with the projection already supplied | 3.5, `[ENT-6.D]` |
| **P-MONO / P-OFF**, corrected | drafted by the loop part and disowned by it; unsound as drafted (A3) | 3.5, `[ENT-3.S5.O]` |
| the corrected **`*wrap` box image** | the audit required the correction; no part took it | 3.5 |
| the **`ior` / `maxor`** image | same | 3.5 |
| the **enumeration discipline** that makes the column total and reviewable | 0106 stated it; nothing carried it into a gate | 3.4, 7.4 |

The sequencing conclusion is unchanged and it is the core part's: **the deletion
narrows the accepted set and the publishers widen it, so this is one merge.**
Section 8 orders the work inside that merge.

### 2.6 A19 and A20: what P-LOOP may retain

The drafted candidate set is "the atomic facts of `E(@l)` that are not in
`H0(@l)`". `[ENT-2]` 2901 makes an atomic fact one difference bound, and
`[ENT-4]` rule (3) makes the closed state hold exactly one - the tightest - per
ordered pair. So per pair the only candidate is the constant that holds at
entry, and P-LOOP retains that constant or nothing.

The inductive bound of an ordinary counting loop is almost never that constant.
`for`-shaped or `loop`-shaped, a walk over a container whose length folds to a
literal has `tap = 0` and `tap_count = 8` at entry, so the closed entry state
holds `tap - tap_count <= -8`; the body derives `-7` on the back edge; `-8` is
deleted in round one and the head learns nothing. The fact the loop maintains is
`tap - tap_count <= 0`, which is **true at entry** and **never a candidate**.
`y1`/`y2`/`y3` compile all three halves of that sentence.

Two consequences follow, and both matter more than the defect.

**The family splits on whether the extent folds to a constant.** A walk over
`&buffer<u8>` with a symbolic `source_length` works, because the entry-tight atom
on `(scan, source_length)` *is* `<= 0`; the byte-identical walk over
`array<u8, 8>` does not. That is the audit's own indictment of I7 - "the family
splits on whether the count is a compile-time constant, which is not a
distinction any writer would predict" - turned inward on the batch's own
headline rule.

**The repair must not become a widening.** Closing the candidate set downward in
the constant makes it infinite and destroys the deletion argument. My repair
(3.6, `[ENT-5.R2]`) is a **spec-fixed finite ladder**: for each ordered pair, the
candidates are the bounds whose constant is drawn from `K`, the finite set of
integers the function's own text determines - every bound constant in the closed
entry states, every constant of a normalized `[ENT-2]` goal of the function, and
0 and -1 - restricted to constants at least as large as the entry-tight one. Four
properties:

- **A1-immunity survives verbatim.** Every candidate is *entailed at entry*,
  because its constant is at least the entry-tight constant. P-LOOP still never
  establishes at a head a fact that was not already true before the loop, and
  `[ENT-5.R9]`'s two counterexample programs are refused by construction exactly
  as drafted.
- **The deletion still terminates and is still unique.** `K` is syntactically
  determined and finite; the candidate set is a subset of `pairs x K`; deletion
  only shrinks it.
- **It closes the constant-extent family.** For `(tap, tap_count)`, `0` is in `K`
  (the loop's initializer puts `tap - Z <= 0` in the entry state); the candidate
  `tap - tap_count <= 0` is true at entry and re-derived on the back edge, so it
  is retained, and `y2` compiles that step.
- **It pays for itself in complexity by a factor of `|K|`,** which is the number
  of distinct constants one function mentions. Section 3.6 states the cost.

**A20 then becomes free.** The drafted `[ENT-5.R3]` freezes the candidate
universe in the no-retention flow to make termination trivial, at the cost that
an inner loop can never receive a fact only an outer loop's retention makes
available - which `fir_filter.wf:45` wants, in the same design's own migration
table. With the ladder, the universe is a subset of a **syntactically fixed
finite set** (`pairs x K`), so the outer ascending iteration - recompute the
universes under the converged family, converge again, repeat - terminates by
finiteness rather than by fiat. Section 3.6 draws the two-level fixed point and
its monotonicity argument. The audit's own P-LOOP text never froze the universe;
the freeze was an artifact of drafting and the repair returns to the audit.

### 2.7 A22: one notation for I1, and what the second one contributed

The owner's I1 ruling asked for one thing: examine the declared-statement route,
state exactly what normalizer power it needs, and price the residue. Two parts
did it separately and produced two keywords, two procedures, and two cost
accounts an order of magnitude apart. I land one.

**`bound`, with the substitution-and-certificate check, is the construct.** It is
closed, its soundness argument is written and checks out, it needs **no new
column on the operation table**, and it is the same construct that answers I2,
I3 and I4 - so the language gains one statement form rather than two. The three
normalizer powers I1 needs, stated exactly as the owner asked, are in 3.9.3:
the division witness for a literal divisor, elimination against ambient
difference bounds over the statement's own terms, and integer tightening
(`floor(C/s)`). All three are inside `[IND-7]` as this file drafts it.

**`prove` and its exact algebraic denotation column are not adopted.** The
column is 60-100 rows carrying a per-row correctness obligation that review must
discharge, which roughly doubles the review load of the image column - and the
image column is being enumerated in this same change. Its own author recommends
buying the experiment before the column. The route it describes reaches nothing
`[IND-4]` does not reach, and it reaches it by unfolding through eight commit
events with no path restriction, which is strictly harder to fence.

**What the second design contributed, and I keep as a finding.** The split
between algebraic and data-dependent probes is the best idea in that section and
it survives the deletion of its notation: *the algebraic probes (binary,
ternary, galloping) go to the declared statement; the data-dependent probe
(interpolation search) goes to an `ensures` on the function that computes the
scaled offset, whose `ensures ilt(result, span)` under `requires ilt(lo, hi)`
gives the caller `mid < hi` by the same cancellation.* **Nothing needs a shape
rule.** That is the sentence that closes the door on `[ENT-3.S8]`'s restoration
for good, and it is in 3.9.4.

### 2.8 A23: the four bucket-B claims, and what losing them costs

`CENSUS.md` names four claims whose fact is two-term but whose only bridge is a
three-term equality, calls them "the only fact-language work the whole corpus
asks for", and every part of the batch built an argument on them. Three parts
gave them three incompatible routes. The compiler settles it:

| site | the three routes offered | verdict |
| --- | --- | --- |
| `percent_decode.wf:28`, `:31` | a guard rewrite; an unlabelled statement; the backward `+-wrap` rows | **the guard rewrite.** `t4_percent_escape_free.wf` **accepts** (re-run, compiled) with no new rule: two `+checked` arm equalities and one guard `ilt(last_index, source_length)` discharge both subscripts |
| `wfgrep.wf:434` | a guard respelling; an unlabelled statement; the backward rows | **the guard respelling.** `t10_residue_repaired.wf` **accepts** (compiled): `ine(room, 0_u64)` written `ilt(carry, input_room)` |
| `wfgrep.wf:553` | restructure; an unlabelled statement; the backward rows | **restructure**, guarding `source_index` directly against `input_room`. F-D4 compiles it: `probes/p6_shift_restructured.wf` in isolation and `rw/wfgrep.wf` in the whole source. What has no compiled route is the *arithmetic* one (`probes/p1,p2,p4,p5`) |
| `ipv4_checksum.wf:22` (the congruence residue) | a pair guard; a counted restructure plus a statement | **the pair guard.** `t8_ipv4_parity_free.wf` **accepts** (compiled), and its false edge is the odd-tail case the RFC actually specifies |

**F-D4 confirms all four rows against the whole sources**, not against reduced
probes: `rw/percent_decode.wf`, `rw/ipv4_checksum.wf` and `rw/wfgrep.wf` compile
claim-free at v0.39 and are byte-identical to the originals over 1,195
differential cases. The ipv4 row gains an execution result it did not have: the
pair guard's false edge, given the RFC 1071 fold rather than t8's plain add,
agrees with an independent reference on 160 odd-length headers, so "the odd-tail
case the RFC actually specifies" is now measured rather than read.

Two arguments lose their support and both must be re-priced honestly rather than
papered over.

**The local induction statement loses three of its four stated customers.** Its
strongest argument was "these four have no `if`/`else` route: their `else` arms
are unreachable", and `t4`'s two `Err` arms are empty, legal, and the honest
reading of an index within two of `max(u64)`. What survives is **I1's midpoint**,
where the `else` arm genuinely is a lie about the program - *what does a binary
search do when its own midpoint is outside its own window?* One customer is a
thin case for a statement form, and section 9's **flagged decision D2** is
exactly that question, with my adopted recommendation (ship it, restricted) and
the recorded dissent (hold it) stated side by side.

Note also that A1's repair removes T3's derivation independently: with the wrap
gate in place, `room = input_room -wrap carry` contributes `input_room - carry`
only when `carry <= input_room` is already derivable, which is the fact the
statement was trying to prove.

**The backward `+-wrap` rows lose their necessity argument.** The contract part
promotes them from the audit's *demoted* status to required, on the ground that
"with `claim` deleted these four programs have no other route". Three of the four
have a compiled route. I keep the rows - they are sound, they are cheap, and
`wfgrep.wf:553` and the general shape still want them - but I keep them as an
**ordinary row-image direction decision inside the enumeration** (3.5), not as a
rule the batch is required to buy, and I record that the audit's demotion stands
on its own reasoning.

### 2.9 A6 and A11: the seams, closed by deletion rather than by choice

Three parts wrote three edge-ordering texts. Two are the same rule under two
ids; the third is v0.39's order. There is nothing to choose between: `[ENT-5.P0]`
and `[ENT-5.X1]` say the same thing, `[ENT-5.P0]` says more of it (it also fixes
where an image lands relative to its own kill), and the third text is simply
wrong - `r7_closure_vs_kill.wf` (compiled) rejects `best < len(data)` under it,
so "every remember-this-index pattern loses its bound". Section 3.3 keeps
`[ENT-5.P0]`, deletes `[ENT-5.X1]`, and section 3.14 rewrites the acceptance
pipeline to match. The same disposal applies to the U view (A11): `[PRV-3]` 3404
*defines* U as the complete state minus S3, so deleting S3 makes `U == complete`
by the definition rather than by argument, and every text that still names three
views is corrected rather than reconciled.

Two seams held under attack and are recorded as such, because a review that only
lists breaks misprices the work:

- **The loop/contract seam at heads.** `[FN-9.L1]` deliberately asserts a
  *non*-rule - a contract fact is an ordinary P-LOOP candidate with no longer and
  no shorter a life than an operation image of the same support - and the loop
  design makes no exception for contract facts. The joint is anchored by
  `c23` **accepting** and `c09` **rejecting** on byte-identical statement
  sequences whose only difference is a `loop`. Two designers who could have
  invented parallel mechanisms deliberately did not. Kept verbatim at 3.10.4.
- **The core/loop interface obligations L1-L6.** All six are satisfied by the
  induction statement as this file drafts it, and 3.9.6 checks them one at a
  time.

### 2.10 The redundancy ruling, and why it is the load-bearing asymmetry

The 0107 audit rejected a loop invariant clause partly on redundancy grounds. The
owner's ruling of 2026-08-29 supersedes that reading: **a written induction
statement is never an error merely for being redundant.** The reason belongs in
the design notes and it is an asymmetry rather than a preference:

> A `claim` **trapped at runtime**. A redundant claim was therefore a live trap
> site retained in every build mode, for a predicate the checker already knew - a
> permanent cost, and a site whose review record asserted a checker gap that did
> not exist. `[CLM-2]`'s non-duplication rule had to ban it. An induction
> statement **costs nothing at runtime**: it is erased proof syntax, like
> `requires` and `ensures`. Redundancy therefore costs nothing. And the asymmetry
> runs forward as well as backward: because a redundant statement is legal, a
> later version's stronger prover may quietly make old statements redundant
> without breaking, churning, or forcing an edit to a single program. Under
> `[CLM-2]` the same strengthening was a **hard error** on every program carrying
> the newly-derivable claim - `[ENT-1]` 2853-2858 says so explicitly - which is
> why the claim model could not let its own prover grow.

Two sentences that look like they are in tension are both true and are the two
halves of the ruling. *A statement is never required where retention suffices*
(`[ENT-5.R]` keeps the easy loops annotation-free, which is the audit's rejection
of a mandatory invariant clause surviving in exactly the form it survives in).
*A statement covering what retention reaches is not an error* (`[IND-9]`). The
first keeps the notation from becoming proof plumbing; the second keeps the
prover free to grow.

This ruling is also what makes `[ENT-1]`'s new unconditional monotonicity
sentence true of the *whole* language rather than of the claim-free fragment. If
a redundant statement were an error, the batch would have removed one
monotonicity exception and immediately added another - and section 2.4 is the
adjudication that stopped it from adding a second one by a different route.

---

## 3. The construct catalog

Eight constructs, in the order a fact travels: the deletion, the guard, the edge
order, the operation image, the row column, loop retention, the loop exit, the
induction statement, the contract system, and the boundary. Each carries draft
specification text with rule ids and exact sentences, a worked example, the
judgment as an algorithm where one is needed, and the falsifier that would refute
it.

### 3.1 The deletion of `claim`

#### 3.1.1 What goes, exhaustively

| kind | deleted | after |
| --- | --- | --- |
| numbered rules | `[CLM-1]` 2696-2757, `[CLM-2]` 2759-2799, `[CLM-3]` 2801-2829, `[DIAG-3]` 1960-1986, `[TRAP-1]` 2432-2434 | 138 become **133** |
| grammar productions | `claim_stmt` (225), its `stmt` alternative (208), `"deny_claims"?` in `fn_decl` (166) | 75 become **73** |
| fixed lowercase atoms | `claim`, `because`, `deny_claims` | moved to the `[FORM-3]` retired-spelling reservation list beside `trap` |
| effect categories | `traps` (`[EFF-1]` 1348/1354, `[EFF-2]` 1372/1415, the four-component normalization at 1094, the `pure` gloss at 1423) | 4 become **3** |
| fact sources | `[ENT-3.S3]` 2950-2955; the ordinal is **retired, not reused**, exactly as S8 is | 11 become 10, and `[IND]` adds one back (1.2) |
| judgments | `[ENT-6]`'s claim-authority state 3215-3248 in full - the `Local`/`BoundaryResult` tree, the seeds, the transfer rules, the v0.39 control-dependence paragraph, the H3 `&uniq` carve-out, the witness identity and its tie-break | in the compiler, `claim_locality.rs` entire (2122 lines) plus its `flow.rs` call sites |
| proof views | the *unasserted* state U; `ViewStates` becomes `{ complete, s4_blinded, entry_images }`, `[FN-9]`'s per-relation `Cq/Uq/Bq` becomes `Cq/Bq`, `[FN-8]` 1308's three-way relation classification becomes two-way | 3 become **2** |
| demand kinds | `[PRV-2]`'s `direct(F, d, L)` and the tag order `direct < bridge` (3374); derived, not chosen - see 3.1.3 | 2 become **1** |
| acceptance-bearing re-analysis | `[CLM-2]` 2782-2798's `Full-minus(c, a)` and `Full-minus(c)` counterfactuals and `[DIAG-2]` 1901's "sole permitted acceptance-bearing rewalks" | `1 + k + sum(m)` whole-program walks become **one forward walk per function per view** |
| runtime surface | the retained check (`[CLM-1]` 2748), the trap record, the abort path (`[SCOPE-4]` 28-30, `[TRAP-1]`, `[PROG-3]` 1497-1498), and the erroneous-execution clauses (`[PAR-1]` 2010-2019, `[PAR-2]` 2043, `[PAR-3]` 2074-2079) | **runtime-trap families: 1 becomes 0** |

`[DIAG-3]` is deleted rather than kept because its own text (1978) says no
construct other than a failing claim produces its record: retaining it would be
the specification describing an output no conforming program can cause. `[EFF-2]`
1372 sources `traps` from exactly "a `claim` or a call to an operation or
function whose selected row includes `traps`", and 1373/1410/2682 confirm no
operation row carries it, so the category is **inductively empty** the moment the
construct goes; `t5` (compiled) shows the compiler already reports
`extra: ["traps"]` for a claim-free body that declares it.

#### 3.1.2 The load-bearing replacement text

Purely English uses of the word "claim" that are not the construct (574, 588,
595, 614, 968, 2397) are untouched.

**`[SCOPE-2]` 19** - DELETE (the retained-claim review-record sentence). Owner
approval is a repository act over exact source bytes and stops being a
specification judgment.

**`[SCOPE-2]` 20-21** - replace both sentences with:

> Every proof-required hazardous operation is statically discharged by the
> deterministic checker before lowering; a writer establishes a missing fact with
> executed control flow, with a machine-verified contract, or with a
> machine-verified induction statement, and no operation receives an implicit
> runtime fallback or a writer-spelled runtime boundary.
> There is no writer-emittable unchecked state and no writer-emittable trusted
> assertion: nothing writer-stated is a fact until this specification's own
> derivation establishes it.

**`[SCOPE-4]` 27-30** - replace the whole rule with:

> `[SCOPE-4]` An accepted program has no writer-reachable language runtime
> contract violation. No source construct traps, aborts, or otherwise fails at
> runtime by its own language contract, and no source construct can request such
> a failure. Every partial operation is admitted only by static discharge of its
> domain `[ENT-6]`, every writer-stated proof obligation is verified at compile
> time `[FN-8, FN-9, FN-10, IND]`, and no accepted operation carries a retained
> runtime check.
> A resource failure, a target-layout or target-qualification failure
> `[STOR-6, QUAL-1]`, and a trusted-computing-base failure remain outside this
> guarantee exactly as `[SCOPE-3]` fixes; none of them is a language trap, none
> cites a language rule, and no rule of this specification is stated in terms of
> them.

That last paragraph is the honest fence and must be in the text: after the
deletion the only non-continuing runtime path a target may still materialize is
`[STOR-6]`'s address-domain guard, which `[DIAG-2]` already requires to be
discharged or guarded and which `[DIAG-3]` 1978 already excluded from the record.
It is invisible to the source language and belongs to the trusted base.

**`[ENT-1]` 2853-2858** - the replacement the whole batch rests on:

> Version monotonicity: fact-source and closure strengthening preserves every
> already-discharged operation, call goal, selected-return relation and verified
> induction statement, and **preserves the acceptance of every accepted program
> without exception**. A later normative checker derives at least what an earlier
> one derived; because no construct's admission depends on a fact *not* being
> derivable, a program accepted by an earlier conforming checker is accepted by a
> later one. No source edit is ever forced by a checker strengthening.
> Activating `[PRV-2]` or `[PRV-3]` for an already attached protected family,
> attaching a new protected family, changing a `[SYS-2]` component from internal
> to external, or adding a callable publication surface remains an
> amendment-level accepted-set change rather than implementation strengthening.

The second sentence is the whole point: *"no construct's admission depends on a
fact not being derivable"* is exactly what `[CLM-2]` violated, it is the reason
the exception existed, and it is now a checkable property of the rule set - which
section 2.4 had to defend against the batch's own new construct, and section
9's D1 flags.

**"Depends on a fact not being derivable" has to be read to include a limit whose
count moves with the prover, and F-I1 is why that sentence is written down.** A
hard error at "more than 16 hypotheses" is not a negative admission condition on
its face, and it is one in effect: the sixteenth slot filled by a v0.41 row image
rejects a program v0.40 compiled. Every spec-fixed limit in `[IND]` is therefore
a limit on a count fixed by the program's own text - body paths, and the
monomials, degree and magnitude of every polynomial the shape rule produces, and
the elimination terms and hypothesis slots - and none on a count of what a prover
happens to derive. **The stronger form of the same sentence is F-I1c's, and it is
what 3.9.1's sweep enforces**: a hard error must not be reachable from anywhere a
prover-supplied *value* can reach either, because "which steps execute" is as
much the prover's as "how many slots are filled". That is the property `[ENT-1]`
needs and the one F-D1's mechanical grep should be extended to look for.
**F-I1d adds the third form and it is not a grep**: a limit *removed* in exchange
for a resource bound is only as good as the bound's premise, so the same reading
has to be applied to every claim that a quantity is "bounded by the caps alone".

The remaining edits are mechanical and are given in spec order so the amendment
can be executed from this table:

| line | edit |
| --- | --- |
| `[FORM-1]` 94 | "STRING appears only in `doc` entries and `claim` justifications" becomes "STRING appears only in `doc` entries" |
| `[GRAM-4]` 166, 208, 225 | delete the three productions and the marker |
| `[GIVE-1]` 282 | "A `claim` or call that may trap also has a normally continuing edge..." becomes "A call also has a normally continuing edge and does not count as delivery or must-divergence" |
| `[OP-2]` 840 | "An executed branch condition, requirement, or retained claim continuation" becomes "An executed branch condition, a requirement, or a verified induction statement" |
| `[OP-4]` 878 | replaced by 3.12's four-route menu |
| `[OP-5]` 892 | "a `claim` condition is the one writer-authored runtime checked site" is deleted; the sentence becomes "An `if` condition is executed control flow `[GRAM-6]`; a contract predicate and an induction statement's relation are erased proof syntax `[FN-8, FN-9, FN-10, IND-1]` and create no runtime site" |
| `[OP-9]` 944 | "so a claim about an unconditionally external n cannot launder it past the required real branch" becomes "so an unconditionally external n reaches an allocation only through a real branch that establishes the fit predicate" |
| `[FN-1]` 1027, 1052, 1055 | drop "and a passed `claim`"; "every source call and claim identity are retained" becomes "every source call identity is retained"; delete the `deny_claims` sentence |
| `[FN-2]` 1225 | delete the `deny_claims command fn main` prefix sentence |
| `[FN-8]` 1261, 1270, 1308 | drop "claim audits"; "Imported claims are tested first; otherwise..." becomes "The first refuted or unproved clause in source order owns the FN-8 rejection"; the three-way classification becomes two-way |
| `[EFF-1]` 1094, 1348, 1354, 1423 | three components, not four; delete the `traps` alternative; amend the `pure` gloss |
| `[EFF-2]` 1372, 1373, 1410, 1415 | the body-syntactic contribution exhibits reads, writes and allocations only; delete the three claim sentences |
| `[ERR-4]` 1463 | "Classification: expected environment and input failures are values (`Result`); unproved function, operation-domain, allocation-fit, bounds, and system-range obligations are source rejections. **There is no third class: no source construct defers a proof obligation to runtime `[SCOPE-4]`.**" |
| `[PROG-3]` 1485, 1497-1498 | delete the `deny_claims` start-time sentence; a start failure becomes the only way an instance terminates without mapping a returned `ExitStatus`, and it is a target or environment condition |
| `[DIAG-1]` 1760, 1846-1869 | delete the claim-name carrier and the whole claim-diagnostic schedule and `[CLM-3]` stage; section 6 is their replacement |
| `[DIAG-2]` 1884-1893, 1897, 1901-1903, 1941-1942 | "No writer-reachable source-language runtime check exists..."; one facts-off sentence; delete the S3 view tag, the `Full-minus` sentences, the retention paragraph and the ClaimLedger |
| `[ENT-1]` 2833, 2836, 2843, 2850, 2861, 2864 | the fixed-judgment list becomes complete-state discharge `[ENT-6]`, call-goal discharge `[FN-8]`, selected-return verification `[FN-9, FN-10]`, and induction verification `[IND]`; two fact states, not three; delete the claim-authority vocabulary and partition sentences |
| `[ENT-2]` 2891, 2892, 2898 | drop claims from the goal universe; keep "exact signed disjunctive roots already present in a written condition"; delete the CLM-2 contribution identity. **An induction statement's relation is not a goal** - 3.9's `[IND-3]` gives it its own clause (A25) |
| `[ENT-3]` 2910, 2930, 2936-2940, 2950-2955 | 2910's "no ... writer-stated or inferred loop induction ... exists" becomes "no **unverified** writer-stated or inferred loop induction exists"; "at every ordinary non-claim establishment" becomes "at every establishment"; delete the `Contrib(P)` paragraph and S3; add the retirement sentence beside S8's |
| `[ENT-4]` 3028 | delete the CLM-2 contradiction clause; the contradiction rule is untouched |
| `[ENT-5]` 3053, 3108 | "B differs from the complete state only by suppressing every S4 source"; "no S11 body fact or capture fact reaches the join" |
| `[ENT-6]` 3169-3173, 3215-3248 | 3.12's route menu; delete the claim-authority block |
| `[PRV-2]` 3346, 3352, 3355, 3370, 3380, 3386 | 3.1.3's collapse; delete "A `claim` is not a repair" |
| `[PRV-3]` 3404-3406, 3412 | two views; "A local PRV-3 rejection never becomes a call-argument event" |
| section 18 heading 2694 | "Obligation discharge: entailment and provenance (normative)" |

**The S3/S8 retirement sentence**, placed beside S8's at 3009:

> The labels S3 and S8 are retired, not reused. S3 was the executed-claim source;
> the `claim` construct is deleted and no *unverified* writer statement is a fact
> source.

#### 3.1.3 The three derived collapses

**Three proof views become two.** `[PRV-3]` 3404 defines U as the complete state
with "exactly S3 claim establishment" removed, so deleting S3 makes `U` and the
complete state the same state at every program point *by the definition*. Every
source, kill, join and closure is applied twice instead of three times; `[ENT-1]`
2836's cross-implementation obligation names two states; `[FN-8]` 1308's
three-way split becomes two-way. Nothing about provenance's meaning changes -
`[PRV-2]` and `[PRV-3]` defend against an external value reaching a protected
subject and the state that decides it, B, is untouched. The middle view existed
only to answer *"would this still discharge if the writer's word were
withdrawn?"*, and after the deletion the writer has no word to withdraw.

**`[PRV-2]`'s `direct` demand kind becomes unreachable.** The argument, as a
proof obligation a reviewer can check against quoted text: (1) `[PRV-3]` 3389 -
the `[ENT-6]` complete-state judgment runs first, and if it fails no PRV-3
candidate exists - so every leaf reaching the partition has discharged in the
complete state; (2) 3397 retains a direct demand only on *failure in U*; (3)
`U == complete`, so failure in U is unsatisfiable. Therefore `Targets(c, q)` is
built entirely from bridge records and the tag order has one element. The reading
this confirms is the right one: a *direct* demand was precisely "the caller's
actual reached a protected subject and only the callee's **claim** stood between
them". **The claim was the only non-bridge route from external data to a
protected operation, and `[PRV-2]`'s two demand kinds were exactly the two
routes.** Section 3.9's `[IND-8.V]` is what keeps the induction statement from
becoming a second one, and A9 is the adjudication that caught it.

**The counterfactual re-walks disappear.** `[CLM-2]` 2782-2798 required, per
admitted claim `c` and per contribution component `a`, a repeat of the whole
program analysis with `a`'s S3 event withheld, plus one more with all of `c`
withheld. Deleting the claim deletes the only construct in the language whose
*acceptance* is defined by a counterfactual, and with it the failure-atomic
scratch discipline, the masked-witness inventory, and the
"inconsistent-counterfactual is a compiler failure" clause.

#### 3.1.4 Falsifiers for the deletion

**F-D1 (the monotonicity theorem).** *Refutation:* exhibit any surviving rule
whose acceptance condition is negative in the fact state - a rule that rejects
when something *is* derivable. The audit is mechanical: grep the amended
specification for "redundant", "refuted", "vacuous", "already derives",
"unnecessary". This experiment found `[IND-7]` (section 2.4) and it must be
re-run after every repair. **Extend it twice**: F-I1 showed the second shape of
the same defect, a hard error on a count a strengthening can grow, which no
negative-fact grep finds; F-I1b showed the third, a count that is syntactic given
a traversal the prover chooses, which no count-by-count audit finds either. Grep
for every "hard error" and ask, of each, whether its count is fixed by the
program's text or by what the prover derives - and then ask the same of every
*input* to that count, down to the set of program points the rule reads.

**F-D2 (`traps` is really empty).** *Refutation:* find any operation-table row,
system operation, or compiler-derived operation whose selected row includes
`traps`. Five specification sites were checked and `t5` compiles the consequence;
the operation table is 203 system records long and was not read row by row.

**F-D3 (`direct` really is unreachable).** *Refutation:* construct a program,
under the amended rules, producing a protected leaf that discharges in the
complete state, fails in B, has a nonempty parameter set, and is not discharged
in U. Since `U == complete` this requires the complete state both to discharge
and not to discharge one leaf, so a refutation is an inconsistency in the reading
of 3389 and 3397.

**F-D4 (the corpus disposition). RUN; verdict MIXED.** The refutation condition
was that the reduced probes fail to survive contact with the whole sources - an
interaction with an effect row, a contract, or another claim in the same
function. All three sources were rewritten claim-free in full, compile at v0.39
with no diagnostic, and are byte-identical to the originals over 1,195
differential cases; eleven claims deleted, no rule added, no behaviour changed.
**The refutation condition fired once, on an effect row**: guarding a `&uniq`
output parameter against its own `len` reads that parameter and `[EFF-2]` widens
the function's declared row (11.1, and 6.2's `guard` row now says so). Two of
4.5.2's route assignments were wrong and two more understated what v0.39 already
reaches, and a fifth row (`wfgrep:553`) is rewritten because Q1 closed on the
restructure - **five rows corrected, covering six of the eleven sites**, since
the `:469`/`:495` row carries two. Nothing found refutes the deletion, the
principle, or any of the four routes named as compiled - `t4`, `t8`, `t10` and
Q1's option (b) all held under contact with the whole sources.

### 3.2 Guard publication - the successor's load-bearing rule

The finding is that **it needs no amendment**. `[ENT-3.S1]` (2945-2948) already
publishes everything the successor story needs; what follows is the rule restated
as the specification will read it, because after the deletion it is the first
rule a writer meets.

> **`[ENT-3.S1]` (guard facts).** At an `if_stmt` or `value_if`, each goal G in
> the condition's goal-origin set is established as `+G` at the then-block's
> entry and `-G` at the else-block's entry; for an else-free `if_stmt`, `-G` is
> established on the false edge, which joins the then exit at the continuation
> `[ENT-5]`. Independently, when the condition has comparison origin R, R is
> established at the then entry and R's exact negation at the else entry or false
> edge. L0 negation is exact over mathematical integers: the negation of
> `a - b <= c` is `b - a <= -c - 1`; the negation of `a = b` is `a != b` and
> conversely.

Read off the four edge kinds, exhaustively:

| edge | what arrives |
| --- | --- |
| **then entry** of `if c { ... }` | `+G` for every G in c's goal-origin set; the comparison relation R when c has one; every member of `+G`'s signed decomposition set (so `band(A,B)` publishes `+A` and `+B`, recursively) and each member's own comparison projection |
| **else entry** of `if c { ... } else { ... }` | `-G` for every such G; R's exact negation; every member of `-G`'s decomposition set (so `bor(A,B)` publishes `-A` and `-B`, recursively) |
| **false edge** of an else-free `if` | exactly the else-entry facts, on the edge that joins the then exit at the continuation |
| **continuation** of `if c { ... }` whose then arm cannot reach it | exactly the false edge's facts, because `[ENT-5]` 3095 gives an arm all of whose paths leave by `return`, `break`, `give` or `propagate`'s error edge no contribution to the join |

**The fourth row carries the corpus.** It is the guard-and-exit idiom -
`if past { break @l; }` - and it is why a loop whose exit test is
`ige(at, length)` republishes `ilt(at, length)` at every continuing point,
whatever the stride, with no induction at all. `t2` (compiled) is that program: a
claim-free byte walk over a runtime-length buffer.

`else if` needs no clause: it is a nested `if_stmt` in the else block
`[GRAM-6]`, so its then entry carries the outer negation and the inner assertion
by composition, and the chain's last else carries every negation. An ordinary
`match_stmt` or `value_match` over a user enum publishes nothing about a
payload's *value* on any arm and this does not change - `[ENT-2]` 2870 has no tag
term, so there is no fact to publish. The two exceptions are already written and
are the ones the corpus uses: `[ENT-3.S7]`'s checked-arithmetic arm equality and
`[ENT-3.S10]`'s system-range endpoint facts.

**The transfer function**, stated so a second implementation reproduces it:

```
G  := goal_origin_set(c)                    # ENT-3 2922-2926, unchanged
R  := comparison_origin(c)                  # ENT-3 2915, unchanged; may be absent
then_edge_facts := { +g } U signed_decomposition(+g) for g in G
                 U { projection(m) : m in those members with one }
                 U { R }                                  if R present
else_edge_facts := { -g } U signed_decomposition(-g) for g in G
                 U { projection(m) : ... }
                 U { exact_negation(R) }                  if R present
exact_negation(a - b <= c) = (b - a <= -c - 1)     # mathematical integers
exact_negation(a = b)      = (a != b);  and conversely
```

**Worked example, as it compiles today.** `t14_else_free_guard.wf` (compiled,
**accepts**) is the zero-cost shape:

```whitefoot
fn peek['d](data: &'d buffer<u8>, index: own u64) -> result: own u8 reads(data) {
  let acc = 0_u8;
  let room = len(deref(data));
  let inside = ilt(index, room);
  if inside {
    let byte = deref(data)[index];
    set acc = byte;
  }
  return acc;
}
```

`let room = len(deref(data));` establishes `room = len(deref(data))` by S6;
`inside` has comparison origin `index - room <= -1`, so S1 establishes exactly
that at the then entry; `[OP-4]` attaches `index < len(deref(data))`, `[ENT-6]`
normalizes it, and `[ENT-4]` closure derives it in one transitivity step.
`[GRAM-6]` forbids an empty else and prescribes the else-free form, whose false
edge joins the continuation. **Nothing is invented and nothing is reviewed.**

### 3.3 `[ENT-5.P0]` - the edge order, one rule

The single ordering text for the whole language, replacing `[ENT-5]` 3095's
kill-then-close ordering and absorbing the audit's P-ORDER and P-CLOSE. The
sibling id `[ENT-5.X1]` is **not** written; A6 is the adjudication.

> **`[ENT-5.P0]` (edge order).** On every edge, in this fixed order: (1)
> establish every `[ENT-3]` image and every `[ENT-3.S12]` or `[FN-10.E]` relation
> that edge's events establish, **each after that event's own `[ENT-5]` kill and
> before the next event of the same edge is processed, in `[FN-1]` order**; (2)
> take the `[ENT-4]` closure of the resulting state; (3) apply that edge's
> scope-exit kills (c) and (d); (4) close again. A consequence whose own terms
> are still live after (3) survives; a fact supported by a dying term dies. The
> continuation of a `value_if` or `value_match` is an ordinary merge point whose
> inputs are the states on every delivering `give` edge, each taken through this
> same order, and the join is the join of 3097, unchanged.

**Why it is mandatory rather than desirable.** Without step (2) before step (3),
v0.40 as drafted **rejects** `p_vif_both_bare.wf`, a program v0.39 accepts - a
redesign that loses an accepted program to an ordering accident is not a
redesign. And `r7_closure_vs_kill.wf` (compiled) **rejects** `best < len(data)`
under today's order, so **every** "remember this index" pattern loses its bound,
inside loops and outside them. It is also a precondition of loop retention: the
binder-derived equality `best = i` is killed on the break edge before
`i < extent` can be combined with it, so without P0 every candidate whose
derivation runs through the compiler-owned binder update dies on the very edge
where retention tests it.

**Determinism.** Steps (1)-(4) are a fixed sequence over operations the checker
already performs, and `[ENT-5]` 3120-3127 already fixes exactly this order at the
counted preheader, so this generalises the specification's own device. The
inner clause of (1) - *each after its own kill, in `[FN-1]` order, before the
next event* - is the sentence A24's D-5 asks for: it nails the backward row
images' reading point to the **event**, not to the edge, so an edge carrying two
commits has no ambiguity about how many closures precede the second reading.

### 3.4 `[ENT-3.S5]` - the value-commit image closure

Carried forward from the 0106 design (its 3.2), which is where the audit's
P-COMMIT and the operation-image frame were first drafted. Unchanged in content;
the ordering clause moves out into `[ENT-5.P0]`.

> **`[ENT-3.S5]` (operation image).** Every operation-table row usable in a
> total, non-consuming position carries one image rule fixed by `[OP-2]`
> alongside its type and effect rows: the unique strongest finite set of
> `[ENT-2]` facts over the row's result terms entailed by that row's exact
> semantics from the closed state's facts over its operand terms.
>
> At every **value-commit event** whose value is a direct application of such a
> row and whose operands are each a term or a constant, establish that row's
> image over the committed destination place, on the normal continuation,
> under `[ENT-5.P0]`. **The binding form is not read: one operation committed to
> one destination establishes one image however it is spelled.** Allocation
> length, `len(P)` binding, slice creation, copy, conversion and constant
> introduction are image rules of their rows and are not separate sources.
>
> A row whose exact image is not unique in the `[ENT-2]` vocabulary publishes the
> **empty image**, and that publication is itself a clause of the ceiling
> `[ENT-1]` with gap kind `image`. Uniqueness is a property of the row, not of
> the state: a row publishes one image or none.

**The closed list of value-commit events**, from the audit's P-COMMIT, is: an
ordinary `let` initializer; a `set` or `replace` commit; a `give` delivery edge
of a `value_if` or `value_match`; a `propagate` initializer's normal continuation
binding; an `[FN-9]` selected-return binding; and a direct `[GRAM-8]`
construction, which establishes `x.f = a` for each field whose atom is a term or
constant of fragment type and `len(x.f) = len(P)` for each field initialized by
`P` or `move P` at array, slice or buffer type.

Three things this clause does that the four sources it replaces could not: it
removes the `let`/`set` distinction by spelling; it removes the delivery carrier
grammar (`give 0_u64;` establishes what `let x = 0_u64;` establishes, so
`[ENT-5]` 3078-3093's sixteen-line delivery block and `[GIVE-1]` 286-293's
carrier judgment are deleted); and **it makes an unpublished row impossible**,
because a row without an image entry is a hole in a closed table.

**This clause is a hard prerequisite of everything downstream.** The audit's
`b15`/`b15b` pair is its evidence, and loop retention reduces to today's
subtraction exactly without it: with `set` mute, every candidate dies on round
one.

**The enumeration discipline** - the sentence that makes the column reviewable,
which 0106 stated in prose and which becomes a gate here:

> A conforming implementation's operation table has an image entry for every row.
> A row's entry is either a finite set of `[ENT-2]` facts or the explicit empty
> image. The gate test is totality over rows; per-row **strength** is a review
> obligation, and a row whose published image is weaker than its semantics entail
> is a defect under `[ENT-1]`'s "derives exactly the ceiling: neither less, which
> is a defect".

**The honest weakness, restated because it must not be buried.** The column can
be total over rows and still weak on a row, and nothing detects that. Three
things reduce the exposure and none eliminates it: one conformance case per
nonempty image row family; the empty image being an explicit, reviewable
publication rather than an absence; and the computed gap token (section 6), which
turns "the checker is weak here" from an unspoken condition into a citation a
reviewer can dispute. Section 11.2 keeps this in red ink.

### 3.5 The rows the audit requires

Ten items. Five are carried forward from the 0106 design; five are drafted here
for the first time (S2). Everything below is one row of the image column of 3.4,
except `[ENT-6.D]`, which is a normalization route.

#### 3.5.1 Carried forward, with the corrections the audit required

For unsigned T unless stated. `Z` is `[ENT-2]`'s distinguished zero term.

| row | image | note |
| --- | --- | --- |
| `a % d`, state derives `d >= 1` | `Z <= r`, `r <= d - 1`, `r <= a` | **46 of the 108 bucket-P corpus claims** - the largest single family. Scoped to unsigned; the signed rows need their own statement (`r <= abs(c) - 1`, `Z - r <= abs(c) - 1` for a constant `c`) |
| `a / d`, state derives `d >= 1` | `q <= a` | as above |
| `imin(a,b)` / `imax(a,b)` | `r <= a`, `r <= b` / `r >= a`, `r >= b` | listed at `[ENT-6]` 3227 as a total value operation and appearing in **no** `[ENT-3]` source today; that state is exactly what 3.4 makes impossible |
| `ipopcount(x)`, `iclz(x)`, `ictz(x)` | `r <= width(T)` | free, and never reached for |
| `ishr.wrap(x, k)`, k a literal | `r - x <= 0`; and from `x - Z <= hx`, `r - Z <= floor(hx / 2^k)` | - |
| `ishl.wrap(a, k)`, k a literal | from `a - Z <= ha`, `r - Z <= M` where **M is the attained maximum of `(v * 2^k) mod 2^w` over the integers v in `[0, ha]`** | the audit's required correction. The drafted `min(ha*2^k, max(T))` gives 255 for u8, `ha = 200`, `k = 1`, and the attained maximum is **254**, because `v*2` is always even. A weaker-than-unique image is a defect under `[ENT-1]`. M is computed as `ha * 2^k` when that is below `2^w`, and `2^w - 2^k` otherwise. A non-literal shift count publishes the empty image |
| `a -sat b` | `r - a <= 0`, `Z - r <= 0`; and from `b - Z >= k`, `r - a <= -k` | the strict-decrease clause is what makes a saturating back-off loop's measure visible; without it the row publishes only `r <= a`, which is true and useless |
| `a +sat b` | `a - r <= 0`, `b - r <= 0`, `r - Z <= max(T)` | this is where the monotonicity fact lives **free**: no side condition to discharge. `c19b` and `c24` (compiled) are the witnesses that neither the exact nor the saturating row publishes it today |
| `a *sat b` | `r - Z <= max(T)`; and when b is a constant `c >= 1`, `a - r <= 0` | - |
| every float row, every f-domain compare, `bxor`, signed division and remainder's non-unique normalizations | **the explicit empty image** | the publication is a ceiling clause with token `image`, not an absence |

#### 3.5.2 The corrected `*wrap` box image (drafted here)

The audit required this row to "fix the arithmetic and state what it publishes
when `ha*hb` leaves the type", because three defensible readings accept different
programs, which `[ENT-1]` 2835-2836 forbids.

> **`[ENT-3.S5.M]` (unsigned multiply box).** At `a *wrap b` or a discharged
> exact `a * b` at unsigned T of width w, from `a - Z <= ha` and `b - Z <= hb`
> both derivable, let `p = ha * hb` **over the mathematical integers**, as
> `[ENT-6]` 3143 already requires for normalization. Publish `r - Z <= p` when
> `p <= max(T)`, and **publish nothing** otherwise. When either operand bound is
> absent, publish nothing.

The "publish nothing" branch is the whole content of the correction: a wrapping
product whose box leaves the type has no sound constant bound below `max(T)`, and
`r - Z <= max(T)` is already an implicit fact of 2905, so publishing it would be
publishing nothing while looking like something.

#### 3.5.3 The `ior` / `maxor` image (drafted here)

> **`[ENT-3.S5.B1]` (bitwise or).** At `ior(a, b)` at unsigned T, publish
> `a - r <= 0` and `b - r <= 0` unconditionally. From `a - Z <= ha` and
> `b - Z <= hb` both derivable, publish `r - Z <= maxor(ha, hb)`, where
> `maxor(x, y)` is the **attained** maximum of `u | v` over `0 <= u <= x`,
> `0 <= v <= y`, computed as: if `x = 0` then `y`; if `y = 0` then `x`;
> otherwise let `m` be the highest bit set in `x | y`, and
> `maxor(x, y) = 2^(m+1) - 1`.

The audit records that the weaker `< 2^k` sketch is withdrawn and `maxor` is the
attained maximum, for the same `[ENT-1]` reason as the shift row.

#### 3.5.4 `[ENT-3.S5.O]` - the relaxed-operand image, with A3's repair

The audit's P-MONO published `r >= a` at an exact `+` or `*` under a discharged
no-wrap side condition. The loop part generalised it to a two-sided relaxation
and, in doing so, published a false fact. This is the repaired rule.

> **`[ENT-3.S5.O]` (relaxed-operand image).** At `r = a op b` for `op` in
> `{+, -, *}` at unsigned T, where the state discharges that operation's no-wrap
> side condition - which the `Ok` arm of `+checked` / `-checked` / `*checked`
> establishes by construction and an exact `+` / `-` / `*` establishes by its own
> discharged `[OP-2]` obligation - let `[bl, bh]` and `[al, ah]` be the tightest
> constant intervals for `b` and `a` derivable in the state. Publish:
>
> | op | published | side condition |
> | --- | --- | --- |
> | `+` | `r - a >= bl`, `r - a <= bh`, `r - b >= al`, `r - b <= ah` | none beyond the no-wrap condition |
> | `-` | `a - r >= bl`, `a - r <= bh` | none beyond the no-wrap condition |
> | `*` | `r - a >= al * (bl - 1)`, `r - a <= ah * (bh - 1)` | **only when the state derives `Z - b <= -1`** |
>
> Where an endpoint is unbounded the corresponding fact is not published.

**The `*` side condition is A3's repair and it is load-bearing twice.** Without
it, `a` in `[2,10]` and `b = 0` publishes `r - a <= 10*(0-1) = -10`, that is
`a >= 10`, false for every admitted `a` except 10; `j03_poff_mul_unsound.wf`
(compiled) rejects exactly the obligation that false fact discharges, and the
call `harm(a: 2_u64, b: 0_u64)` satisfies every `requires` and underflows. It is
also needed for **determinism**: with `bl = 0`, a *tighter* `al` publishes a
*weaker* lower bound on `r - a`, so the body transfer is not monotone in the head
state, and loop retention's "the union of two inductive families is inductive"
fails with it. One missing side condition breaks soundness and determinism
together. The rule's own justifying customer - `capacity *checked 2_u64` with
`capacity >= 1`, so `bl = bh = 2` and `r - a >= 1` - is unaffected, and the
`L21`/`L22` separation (compiled: `igt` accepts, `ige` rejects) survives intact.

**This row strictly subsumes P-MONO**, whose `r >= a` is the `+` row with `bl`
relaxed to 0, which unsigned typing always gives.

#### 3.5.5 `[ENT-6.D]` - the two-nonconstant `.defined` route (drafted here)

The audit's P-DOM. Nothing in the batch drafted it, and two of the induction
statement's three flagship examples reject without it with the published
projection already supplied (`L08`, `L09`, compiled).

> **`[ENT-6.D]`** `[ENT-6]` gains one normalization route. At unsigned T, the
> goal `a -defined b` is discharged when the closed state derives `b - a <= 0`.
> The goal `a +defined b` is discharged when the closed state derives
> `a - Z <= c1` and `b - Z <= c2` with `c1 + c2 <= max(T)` over the mathematical
> integers. The goal `a *defined b` is discharged when it derives
> `a - Z <= c1` and `b - Z <= c2` with `c1 * c2 <= max(T)`. This decides a goal
> from bounds already in the closed state and introduces no term.

**Intent.** None written, and that is the point: today the writer's own sentence
*"I never take more than I have left"*, spelled `ile(want, remaining)`, is
refused while `remaining -defined want` is accepted. That is the proof residual
promoted into source, and it is the calibration failure the audit identified.
The constant-subtrahend case is **already decided this way today** (`s11`,
compiled, accepts exact `remaining - 7_u64` under `ile(7_u64, remaining)`), so
this generalises an existing route rather than inventing one.

**Determinism.** A lookup of two existing bounds and one comparison of
mathematical integers.

#### 3.5.6 `[ENT-3.S5.B]` - the backward `+-wrap` rows

Kept, with its necessity argument corrected (A23, 2.8): three of its four claimed
customers dissolve today with a guard. It is a **direction decision inside the
enumeration**, taken because it is sound, cheap, and the general shape wants it -
not because the batch is required to buy it.

> **`[ENT-3.S5.B]` (backward wrap image).** At a value-commit event whose value
> is `a +wrap b` or `a -wrap b` at unsigned T with operand terms a and b,
> committed to place r, and where the closed state derives that row's no-wrap
> side condition - for `-wrap`, `b - a <= 0`; for `+wrap`, `a - Z <= c1` and
> `b - Z <= c2` with `c1 + c2 <= max(T)` - establish additionally, from the facts
> the state carries about r at the reading point `[ENT-5.P0]` clause (1) fixes:
>
> | row | fact about r | published about the operands |
> | --- | --- | --- |
> | `a -wrap b` | `r - Z >= k` | `a - b >= k` |
> | `a -wrap b` | `r - Z <= h` | `a - b <= h` |
> | `a -wrap b` | `r != Z` | `a != b` |
> | `a +wrap b` | `r - Z <= h` | `a - Z <= h` and `b - Z <= h` |
> | `a +wrap b` | `r - Z >= k`, b a constant c | `a - Z >= k - c` |
>
> Where the state holds several facts about r on one pair, the **tightest** is
> read. The backward rows are applied **once** per commit event, at the reading
> point of `[ENT-5.P0]` clause (1) - after that event's own kill, after its
> forward image, after one closure - and their output names only a and b, so no
> backward fact enables another backward fact at the same event and there is no
> iteration here. Where the commit destination r is also an operand place - the
> cursor shape `set at = at -wrap 1_u64` - the operand term is the destination's
> **pre-commit** value, which `[ENT-5]`'s kill has removed, so no backward fact
> is published. No other backward fact is published; in particular `a +wrap b`
> publishes nothing backward from `r != Z`, and nothing backward from `r >= k`
> with both operands non-constant, because neither is a difference bound in this
> vocabulary.

Two notes the rule's reader needs, and one A24 asked for. First, `a - b >= k` is
not a restatement of `r >= k`: it is a bound on the **operand pair**, which the
domain can hold, while `r = a - b` is a three-term equality it cannot. Second,
the addition rows are deliberately lopsided, and the asymmetry is arithmetic
rather than an oversight: under no-wrap both unsigned operands are below the sum,
so an upper bound on the sum bounds each operand, while a lower bound on a sum
constrains only their total. Third - the sentence A24's D-5 asked for - the
cursor clause: without it, `set at = at -wrap 1_u64` names an `a` that no live
term denotes, and the row is the common shape in the corpus.

**The warning the rule must carry.** These are the only images that are not a
pure function of the operand bounds; they read facts about r. `[ENT-1]` 2836 still
holds because the reading point is pinned to the letter by `[ENT-5.P0]`, and
loosening any of those words makes two conforming implementations diverge. Its
conformance case must assert the **reading point**, not only the published facts
(7.2, case 12).

#### 3.5.7 The falsifier for the whole column

**F-R1 (the image does not leak).** *Refutation:* implementing the rows changes
the acceptance of any program that does not contain a site of the row in
question. The `[ENT-5.P0]` reading point and the `[ENT-3.S5.O]` `*` side
condition are the two places this is most likely to fail, and `j03` is the
program that finds the second one.

**F-R2 (uniqueness, per row).** *Refutation:* for any nonempty image row,
exhibit operand bounds and an execution attaining a value the image excludes
(unsound), or a strictly tighter `[ENT-2]` fact the row's semantics entail
(a defect under `[ENT-1]`). The shift row and the `ior` row were repaired by
exactly this experiment; **it must be run row by row and it is batch B1's own
falsifier.**

### 3.6 `[ENT-5.R]` - loop-head retention

The audit's P-LOOP, drafted as an executable rule with its three mandatory
corrections in the rule text, and with A19's and A20's repairs. This replaces
`[ENT-5]` 3110-3116 and the second sentence of 3120-3128, and supersedes the 0106
design's published `flow` ceiling and its deferral of induction to a gated batch.

#### 3.6.1 The draft sentences

> **`[ENT-5.R1]` (back edges).** A **back edge** of a loop `@l` is the single
> edge of the conservative structural normal-control graph `[FN-1]` that leaves
> the last statement of `@l`'s body and re-enters `@l`: for a `loop_stmt`, the
> edge to the body entry; for a `for_stmt`, the edge carrying the compiler-owned
> binder update to the header. Because the language has no `continue` form
> `[GRAM-6]`, every loop has exactly one back edge, and the state on it is the
> `[ENT-5]` join of every path through the body that reaches it. A `break` edge
> naming `@l` or an enclosing loop, a `return` edge, and a `propagate` error edge
> are not back edges of `@l`.
>
> **`[ENT-5.R2]` (candidates, and the constant ladder).** For each loop `@l`, its
> **subtracted head** `H0(@l)` is the state this version already computes: for a
> `loop_stmt`, the state before the loop minus every fact having a support member
> that a continuing kill event of `@l` may kill; for a `for_stmt`, the closed
> post-capture state minus the same. Its **entry state** `E(@l)` is the closed
> state at its preheader.
>
> The function's **constant ladder** `K` is the finite set of integers consisting
> of: every bound constant appearing in any `E(@l)` of the function; every
> constant of the normalized form of any `[ENT-2]` goal of the function or of the
> exact negation of one; and `0` and `-1`. `K` is determined by the function's
> syntax and its entry states and is computed once.
>
> The **candidate set** `C(@l)` is the set of atomic facts `f` such that: `f` is
> derivable in `E(@l)`; `f` is not derivable in `H0(@l)`; and either `f` is a
> disequality, or `f` is a difference bound `t1 - t2 <= c` whose constant `c` is
> in `K`.
>
> **`[ENT-5.R3]` (universes and families).** A **retention family** is a choice,
> for every loop `@l` of the function, of a subset `R(@l)` of `C(@l)`. The **head
> state** of `@l` under a family is the `[ENT-4]` closure of
> `H0(@l) U R(@l) U B(@l)`, where `B(@l)` is the set of facts published by `@l`'s
> verified `bound_stmt`s `[IND-8]`. The fixed order of 3126 becomes: preheader
> establishment and closure; continuing-kill subtraction; retention; for a
> `for_stmt`, S11's two body-entry bounds on the true header edge and
> `[ENT-5.X]`'s bounds on the false one; then bound publication; then closure.
> **`B(@l)`'s projection is computed in the pre-publication state**, so two
> `bound_stmt`s on one loop publish simultaneously and the order of statements on
> one loop is not observable.
>
> **`[ENT-5.R4]` (inductive).** A family is **inductive** when, for every loop
> `@l` and every fact `f` in `R(@l)`, `f` is derivable in the state on `@l`'s
> back edge obtained by the ordinary forward flow of the whole function under
> that same family.
>
> **`[ENT-5.R5]` (the family this version uses).** Let `U0` be the family
> universe computed from the flow in which every loop head is `H0` - that is,
> with no retention anywhere - and `R0` the greatest inductive family within
> `U0`. For `i >= 1`, let `Ui` be the family universe recomputed from the flow
> under `R(i-1)`, and `Ri` the greatest inductive family within `Ui`. The
> sequence is ascending in both components; the family this version uses is its
> limit, which is reached in at most `|pairs(F) x K| + 1` outer rounds, where
> `pairs(F)` is the set of ordered pairs of terms live in the function.
>
> **`[ENT-5.R6]` (existence and uniqueness).** Within one universe the greatest
> inductive family exists and is unique. The body transfer is monotone in the
> head state - every `[ENT-3]` source, `[ENT-5]` kill, `[ENT-5]` join and
> `[ENT-4]` closure rule derives at least as much from a larger input state - and
> `B(@l)` is monotone in `R(@l)`, because `[IND-8]`'s projection relaxes the
> statement's other terms to constants the state derives and a larger state
> derives tighter constants. Therefore the union of two inductive families is
> inductive, and the union of all of them is the greatest one. The outer sequence
> of `[ENT-5.R5]` is ascending for the same reason: a larger family makes each
> preheader state larger, hence each universe larger, hence each greatest
> inductive family within it at least as large.
>
> **`[ENT-5.R7]` (computation).** Within one universe the greatest inductive
> family is computed by deletion: start from `R(@l) = C(@l)` for every loop;
> recompute the function's forward flow under the current family; delete from
> `R(@l)` every fact not derivable on `@l`'s back edge in that flow; repeat until
> a round deletes nothing. **Deletion is simultaneous over all loops of the
> function**: a round deletes from every loop before the next round recomputes.
> Candidate sets are finite and never grow, so at most `|C| + 1` inner rounds
> occur.
>
> **`[ENT-5.R8]` (soundness, normative note).** Every candidate is derivable in
> `E(@l)`, hence true on every entry to `@l` under the family that produced that
> universe. Order the visits to all loop heads of a function in any execution by
> trace position and induct: at a first visit to `@l`'s head the facts of `R(@l)`
> hold because they held at entry; at a later visit they hold because the
> previous visit's head satisfied the whole family, the body transfer derived
> each of them on the back edge `[ENT-5.R4]`, and every fact the derivation used
> was either in `H0` - sound by 3110 - or in the family, which the induction
> hypothesis supplies. The published bound facts `B(@l)` are assumed in the same
> induction and are discharged by `[IND-5]` and `[IND-6]` at the limit family; a
> function with an unverified `bound_stmt` is rejected, so no accepted program
> rests on an unproved member of the assumption set.
>
> **`[ENT-5.R9]` (A1 immunity, normative note).** Retention never establishes at
> a loop head a fact that was not already true before the loop. It is a
> **deletion** from a set of facts the entry state already derives; the analysis
> has no widening operator, no join over an iterated head state, and no way to
> write a constant that no source produced - the ladder `K` is drawn entirely
> from constants the function's own text and entry states contain, and every
> candidate is required to be derivable in `E(@l)`. In particular
>
> ```whitefoot
> let n = endpoint(value: input);
> let flag = 0_u64;
> for @steps step in 0_u64..n { set flag = 1_u64; }
> ```
>
> has `flag - Z <= 0` among the candidates of `@steps`, and the body's `set`
> derives `flag - Z <= 1` and `Z - flag <= -1` on the back edge, from which
> `flag - Z <= 0` is not derivable; the candidate is deleted in round one and the
> head learns nothing. Correspondingly, `ilt(cursor, 4_u64)` after
> `for @steps step in 0_u64..upper { set cursor = 1_u64; }` is not derived at the
> continuation, because a retained fact is a *head* fact and the continuation
> join is unchanged by this rule.

#### 3.6.2 The judgment, as an algorithm

A second implementation reproduces this byte-identically by executing exactly:

```
 1  K       := ladder(F)                       # ENT-5.R2, syntactic, computed once
 2  family  := empty for every loop
 3  repeat                                     # the outer universe iteration
 4      flowU := forward flow of F under family
 5      for each loop @l:  E, H0 from flowU;  C(@l) := ladder-restricted candidates
 6      R(@l) := C(@l) for every loop
 7      repeat                                 # the inner deletion
 8          flow := forward flow with head(@l) = close(H0(@l) U R(@l) U B(@l))
 9                                             (+ S11 body-entry bounds for counted)
10          D := { (@l,f) : f in R(@l), f not derivable on backedge(@l) in flow }
11          R := R \ D
12      until D is empty
13      grew := (R strictly contains family)
14      family := R
15  until not grew
16  verify every bound_stmt against the final flow            [IND-5], [IND-6]
```

Every step is a total function of the syntax tree and of already-specified
operations. Line 8's "forward flow" is `[ENT-3]` 2907 unchanged; line 10's
"derivable" is `[ENT-4]` 3011 unchanged; line 11 removes a set and no order
inside a round is observable, because the round's tests all read `flow`, which
line 8 fixed before any deletion. **There is no widening operator, no fixed point
over `FactState`, no choice point, no backtracking and no implementation-chosen
threshold**, so `[ENT-1]` 2835-2836's byte-identical-derivation requirement
holds.

**Why the inner deletion must be simultaneous.** `r_p1_nested.wf` is an ordinary
nested walk in which the inner loop's candidate `y - Z <= 0` is re-derived on the
inner back edge *only while the outer loop's candidate `x - Z <= 0` survives*. A
per-loop procedure with no stated order lets an implementation solve the inner
loop first, retain `y <= 0`, then delete `x <= 0` at the outer loop and never
revisit the inner one - retaining a fact that is false on the second outer round
and admitting an out-of-bounds read. This is the only *soundness* break the audit
found against any proposal, and it belongs in the rule text rather than in a
note. The separating pair is machine-checked: `L10a` (compiled) **rejects** the
inner step's `ensures ile(result, 0_u64)` with `"x - 0 <= 0", Unproved`, and
`L10b` (compiled) **accepts** the same step once the outer candidate is supplied
as a `requires`. Lines 6-12 have no order to fix because a round's tests are all
taken against one flow.

**Why the outer universe iteration is needed** (A20). `fir_filter.wf:45` is the
witness and it is in the loop design's own migration table: the inner `@taps`
loop needs `read_cursor - Z <= 7`, which is derivable at its preheader only if
the **outer** loop retained `filter.delay.cursor <= 7`. Freezing the universe in
the no-retention flow makes that candidate unreachable in every round. The
audit's own P-LOOP text does not freeze the universe; the freeze was drafting for
easy termination, and with the ladder the universe lives in the fixed finite set
`pairs(F) x K` so termination is by finiteness instead.

#### 3.6.3 Breaks, typed exits, nesting, regions

- **`break`.** A break edge is not a back edge, so no candidate has to be
  re-derived on it. A retained head fact flows through the body to the break edge
  under ordinary kills and reaches the continuation join of 3097 unchanged, which
  is what makes the break-carried witness of 3.7.3 work.
- **`return` and `propagate`'s error edge.** Not back edges; 3110's existing
  sentence that a fact a non-continuing edge kills is still removed on that edge
  is unchanged.
- **Nested loops.** An inner loop's candidates are still computed from the state
  at *its* preheader, so facts established inside an outer iteration - a row base
  address, a per-record limit - are candidates of the inner loop exactly as
  before; facts carried from outside the outer loop are shared through line 8's
  single flow, and facts the outer loop *retains* enter the inner universe on the
  next outer round.
- **The counted head.** Candidates are drawn from the closed post-capture state,
  so capture terms may appear in them (`upper_capture = n` is a candidate and
  survives unless `n` is written in the body). **The binder is not live before the
  loop, so no candidate mentions the binder**, and the hidden binder update on the
  back edge kills every fact supported by it. That is not an accident of
  drafting: it is exactly why the trip-count family cannot be reached by
  retention and needs `[IND]` instead (3.9.5).
- **Regions and borrows.** A candidate whose support contains a borrow holder
  released by a region exit inside the body cannot be re-derived on the back
  edge, because its support member is dead there; it is deleted in round one. No
  special sentence is needed.

#### 3.6.4 Complexity

Let `T` be the number of `[ENT-2]` terms live in the function, `n` its
statements, `L` its loops, `D` the disequalities of the entry states. Then
`|C| <= L * (2T^2 * |K| + D)`, one inner round is one forward pass at
`O(n * T^3)` with the standard DBM closure, the inner loop runs at most `|C| + 1`
rounds, and the outer loop at most `|pairs(F) x K| + 1` times. The honest reading:
`T` is the number of terms live in **one function**, not in the program; `|K|` is
the number of distinct constants one function mentions; and a round that deletes
nothing ends the loop, so in practice the count is the length of the longest
chain of candidates that depend on one another, which is at most three in every
program in `tests/programs/`. **A spec-fixed round cap is legal under `[ENT-1]`
and none is written**, because no measurement asks for one; if one is ever needed
it is a number in the specification, never an implementation choice.

The ladder multiplies the candidate count by `|K|` and that is the price of A19.
It is the one place in this design where a repair costs measurable compile time,
and F-L2 below is the measurement.

#### 3.6.5 Worked examples

**(a) The descending walk (ITER-03, ITER-28).** `r13_descend_guarded.wf`
(compiled) rejects today with residual `cursor < len(data)`.

```whitefoot
let length = len(data);
let cursor = length;
let sum = 0_u8;
loop @down {
  let live = igt(cursor, 0_u64);
  if live { } else { break @down; }
  set cursor = cursor -wrap 1_u64;
  let value = data[cursor];
  set sum = sum +wrap value;
}
```

`E(@down)` derives `cursor - length <= 0` and `length - cursor <= 0`; the
continuing `set cursor` kills both, so both are candidates (with constant 0,
which is in `K`). Round one, on the back edge: the guard gives `cursor >= 1`,
S7's range-guarded wrap subtraction plus the value-commit image give
`cursor' = cursor - 1` over the `set` destination, and from the head's
`cursor - length <= 0` the flow derives `cursor' - length <= -1`, hence
`<= 0`. The first candidate is re-derived and retained; the second is not and is
deleted. `R(@down) = { cursor - length <= 0 }`, converged in one round, and the
subscript discharges from `cursor' - length <= -1`. **No writer text.** The step
obligation is machine-checked today in isolation: `r13b_descend_step.wf`
(compiled) **accepts** exactly this induction supplied as `requires`/`ensures`.

**(b) The constant-extent walk, which the drafted rule lost (A19).**

```whitefoot
let taps = buffer_new(8_u64, 1_u8);
let tap_count = len(taps);
let tap = 0_u64;
loop @taps_loop {
  let finished = ieq(tap, tap_count);
  if finished { break @taps_loop; }
  let coefficient = taps[tap];
  set tap = tap +wrap 1_u64;
}
```

`y3_const_extent_loop.wf` (compiled) rejects with `[OP-4] residual: tap <
len(taps)`. `E` derives `tap_count = 8` and `tap = 0`, so the *tightest* atom on
the pair is `tap - tap_count <= -8`, which the back edge does not re-derive. The
ladder puts `tap - tap_count <= 0` in `C(@taps_loop)` because `0` is in `K` and
the bound is derivable at entry; the negated exit test gives `tap != tap_count`,
`[ENT-4]` rule (2) tightens the head's `<= 0` to `<= -1` inside the body, the
`+wrap` image gives `tap' = tap + 1`, and `tap' - tap_count <= 0` is re-derived.
Retained. The subscript then discharges from the in-body `<= -1`.
**`y1_entry_tight_step.wf` rejects and `y2_relaxed_step.wf` accepts** (both
compiled): those two functions are the two candidates' step obligations posed as
contracts, and they are the evidence that the ladder is necessary and sufficient
here.

**(c) The chunked carried cursor (ITER-19, ITER-20).** `L12_chunk_loop_carry.wf`
(compiled) rejects with `[FN-8] UndischargedCallRequirement ile(carry, room)` -
which is the scenario's own sentence, that the *next* iteration's read cannot
re-prove that the carry fits.

```whitefoot
loop @chunks {
  let filled = refill(start: carry, end: room);   // requires ile(start, end)
  let done = ieq(filled, carry);
  if done { break @chunks; }
  let short = ile(filled, 15_u64);
  if short { set carry = filled; } else { set carry = 15_u64; }
}
```

`carry - room <= 0` is derivable at the preheader (`carry = 0`, `Z <= room`) and
the continuing `set carry` kills it, so it is a candidate. On the back edge - the
join of both arms - the short arm derives it from the modelled `ensures`
`filled <= room` through the value-commit image, and the long arm from the
negated guard `15 < filled <= room`; the `[ENT-5]` join keeps the bound both arms
hold, so it is retained and the next round's `requires` discharges at the head,
**with no writer text at all**. `L14_chunk_step.wf` (compiled) **accepts**
exactly this induction posed as a contract across the modelled read and both
arms. The price today is `L13_chunk_loop_guarded.wf` (compiled, accepts): four
added lines that re-test `ile(carry, room)` at every head and `break` when it
fails, whose `else` arm is unreachable - the fabricated arm the audit's intent
test refuses.

**(d) The capacity-checked worklist (ITER-23).** `L17_worklist_uncapped.wf`
(compiled) rejects with `head < len(queue)` and **deserves to**: `pending` grows
by a number read out of the data, so `pending <= len(queue)` is false and the
program is wrong. **Any loop rule that rescues it is refuted by this probe**, and
recording that is how an iteration rule is bounded. The correct program checks
capacity on the **push** side, where the check is real, and
`L18_worklist_capped.wf` (compiled) does exactly that and still rejects today;
under retention, `pending - len(queue) <= 0` is a candidate re-derived on the
only continuing path (the `ok` arm, where the just-executed test gives it
directly through the value-commit image), while the `Err` arm and the `else` arm
leave by `break` and are not back edges. `L20_worklist_step.wf` (compiled)
**accepts** that induction posed as a contract. The potential-function reading
the scenario file argues for is not needed: it is a *termination* argument,
Whitefoot has no termination obligation, and the *safety* obligation is the
subscript, which the capped invariant closes without counting anything.

#### 3.6.6 Falsifiers

**F-L1 (reach).** Eleven audit scenarios and nine corpus claims are predicted to
dissolve under retention. *The experiment:* implement `[ENT-5.R]` over the
value-commit image and `[ENT-5.P0]`, and delete the claims from `fir_filter.wf`,
`utf8parse.wf`, `percent_decode.wf`, `wfgrep.wf` and `par_layout.wf`. *Refuted
if* fewer than six of the nine predicted sites compile, or if any program needs a
statement the table does not predict. **This is the single highest-value
experiment in the design**, and note that its threshold must be evaluated against
this file's `[ENT-5.R2]`: under the drafted candidate definition the three
`fir_filter` sites fail on rule text alone (A19), so a run against the unrepaired
rule would refute the wrong thing.

**F-L2 (cost).** The ladder multiplies the candidate count by `|K|` and the outer
universe iteration adds rounds. *Refuted if* any program in `tests/programs/`
needs more than five inner rounds or more than two outer rounds, or if compile
time on `wfgrep.wf` (1,417 lines) or `raw_deflate_vectors.wf` (863 lines) moves
by more than a small constant factor. Instrument the round counters; no new
programs needed.

**F-L3 (simultaneity).** `r_p1_nested.wf` must remain a rejection under
`[ENT-5.R7]`, and an inner-first per-loop procedure must accept it. *Refuted if*
a conforming implementation of R7 accepts it, or if no per-loop ordering accepts
it - the second would mean the audit's soundness break was not one.

**F-L4 (A1 immunity is a property, not a hope).** *Refuted if* any program exists
in which the head state under the limit family derives a fact false on some entry
to that loop. The 0106 design's section 9 counterexamples are the seed set and
`[ENT-5.R9]` predicts they are refused by construction.

### 3.7 `[ENT-5.X]` - the counted false edge

The audit withdrew the counted exit postcondition `binder = upper_capture` for
want of a customer. Section 3.9.5 supplies the customer. But the withdrawn form
is also **false**, and that has to be said before it is restored.

`L23_for_descending_range.wf` (compiled) is the counterexample:

```whitefoot
let seen = 0_u64;
for @scan i in 5_u64..3_u64 { set seen = i; }
```

The loop runs zero times and the binder is 5 at the false header edge, while
`upper_capture` is 3. `binder = upper_capture` is simply not a theorem of the
counted form. The repair is one side condition.

> **`[ENT-5.X]` (counted false edge).** On the false header edge of an admitted
> `for_stmt`, establish `upper_capture - binder <= 0`. Establish additionally
> `binder - upper_capture <= 0` exactly when the closed post-capture state of
> 3120 derives `lower_capture - upper_capture <= 0`. Both are established under
> `[ENT-5.P0]`, so they are closed with the still-live capture-to-endpoint
> equalities of S11 **before** the binder and the captures leave scope. 3116's
> sentence that the false header edge establishes no S11 fact is replaced by this
> rule; the `break` edges and the continuation join are unchanged.

**Soundness.** The first conjunct is the negation of the header comparison just
executed and needs no side condition. The second: the binder is initialized to
`lower_capture`, is updated only by the compiler-owned `+1` on an edge from a
state where `binder < upper_capture`, and both captures are immutable after the
preheader, so `binder <= upper_capture` is invariant *given*
`lower_capture <= upper_capture` at initialization - and without that premise the
binder never moves, which is exactly what `L23` refutes.

**What it does not do.** It publishes nothing about a `break` edge and nothing
about any body-written quantity. On its own it closes nothing: its whole value is
that a `bound_stmt`'s head fact, which relates a body-written quantity to the
binder, can be composed with it under `[ENT-5.P0]` before the binder dies. That
composition is 3.9.5.

**Worked example - the break-carried witness (ITER-22, ITER-33, ITER-34).**
`p7_witness.wf` (compiled) rejects today with residual `found < len(data)`.

```whitefoot
let length = len(data);
let found = 0_u64;
let hit = False();
for @scan i in 0_u64..length {
  let value = data[i];
  let is_target = ieq(value, 7_u8);
  if is_target { set found = i; set hit = True(); break @scan; }
}
if hit { let again = data[found]; }
```

`found - Z <= 0` is in `E(@scan)` and the continuing `set found` kills it, so it
is a candidate. On the back edge the only path reaching it is the non-matching
path, on which `found` is unchanged, so it is re-derived and retained; the
matching path leaves by `break`, which is not a back edge, so it imposes no
obligation on the candidate - `[ENT-5.R1]` doing the work. On the break edge,
S11 gives `i < upper_capture` and the preheader gives `upper_capture = length`;
the value-commit image gives `found = i`; `[ENT-5.P0]` closes
`found - length <= -1` **before** `i` and `upper_capture` leave scope; the
surviving fact's support is `{found, length}` and it reaches the continuation
join. The `if hit` arm then discharges with no writer text.

Note that the *empty-buffer* case needs no disjunction and no `hit` reasoning: on
the false header edge `found` is still 0 but `length` may be 0, so the join of
the false edge and the break edge gives only `found - length <= 0`. The audit's
ITER-34 route - an early return for the empty buffer, after which
`found < extent` is unconditional - is therefore the right program, and the 0106
design's convex-join ceiling clause still has **no witness here**. Section 11.3
records that clause's disposition.

### 3.8 `[IND]` part one - the statement, its substitution, and its frame

For facts retention cannot reach because they are **not true before the loop**:
the audit's I2 (`sum <= 255 * i`), I3 (`acc <= i * factor`) and I4
(`hits <= i`). Retention is a deletion from what was already true; these facts
become true one iteration at a time, so no deletion procedure can produce them
and no stronger row image can either.

**The four laws of the `[IND]` pipeline.** 3.8 and 3.9 are one derivation, and it
has now been written down wrong six times - each round repairing the instance in
front of it and leaving the argument one step behind the text. So this revision
states the invariants **first**, as the pipeline's own laws, and every rule below
is an instance of one of them rather than a fresh local promise. `[IND-3]`'s
scope, `[IND-4]`'s shape rule and substitution, `[IND-6]` clause (i), `[IND-7]`'s
caps, steps, `RELAX` and acceptance, 3.9.1's *reachable-error sweep* and its
five-part monotonicity argument are all derived from these four.

**L1 - production-time measurement.** Every quantity `[IND]` limits or counts is
computed from the path's text at the moment `[IND-4]`'s shape rule **produces**
it, before and independent of every prover decision. The pass produces the same
sequence of shapes at every version, and a shape is measured whether or not its
slot fills and whether or not `[IND-6]` clause (i) later drops the condition it
belongs to. Dropping, filling and tightening touch **content** only - never the
measured set, never a limit, never a count.

**L2 - a prover decision fills or tightens content, and does nothing else.** The
single place where a prover decision can move acceptance is `[IND-4]` clause
(b)'s refusal at a `set` destination, and it moves it only from **refusing to
admitting**. That enumeration is complete, it is priced in 3.8.4, and it is safe
because by L1 admitting submits nothing new to a limit. Every other consultation
of the state fills an empty slot, tightens a filled one, or tightens a `RELAX`
interval.

**L3 - every computation the check performs has a bound derived from L1's
counts.** The backward pass **and each intermediate inside it**, the certificate
steps, `RELAX` and `[IND-8]`'s projection each carry a figure derived below from
the spec-fixed limits and the path's own counts. A cost that is assumed rather
than derived is how an implementation's capability becomes a source-language
rejection, which `AGENTS.md` forbids outright.

**L4 - typing and vocabulary stay on the statement.** `[IND-3]`'s *Typing* rule
and *Vocabulary fence* are admission conditions on a `bound_stmt`'s own
`rel_term`. They do not travel to a branch condition: an ordinary `if` condition
that is not a fragment-integer `rel_term`, or that names an element of an
indexable place, gives `[IND-3]` no polynomial, contributes one always-empty
slot, and is never an error.

#### 3.8.1 The spelling, and why it passes the intent test

```whitefoot
for @weigh i in 0_u64..count {
  bound @weigh per_byte: ile(sum, 255_u32 * i);
  let w = deref(weights)[i];
  let wide = cvt<u8, u32>(w);
  set sum = sum + wide;
}
```

Read it as English: *bound, for the loop `@weigh`, named `per_byte`: `sum` is at
most 255 times `i`.* This is what the writer of an accumulator loop already
wanted to say and already writes in a comment - the loop's running bound is the
one thing a reader of a fold wants documented - and deleting the line deletes a
statement about the program, not a hint to a prover. It names no term the
compiler chose, no residual, no lemma and no proof step: it is the **conclusion**,
in the writer's own arithmetic.

Contrast the owner's calibration point. Proof plumbing pinned on `let`s -
`let bound_term = ...; let step_ok = ...;` and a statement over them - is rejected
precisely because the writer is made to name the prover's intermediate values. A
`bound_stmt` names none.

**Position.** A `bound_stmt` carrying a label is admitted only as a **leading**
statement of that label's loop body: it may be preceded only by other
`bound_stmt`s and by `doc`. That is where a reader looks for what a loop
maintains, it makes "this holds whenever we arrive here" unambiguous, and it
gives the rule a single syntactic place to check. The **local** form, which
carries no label, is 3.9.4 and is restricted differently.

**Grammar** (`[GRAM-4]`, one production; `[GRAM-5]` gains a relation grammar used
only here):

```wf-ebnf
stmt        := ... | bound_stmt
bound_stmt  := "bound" LABEL? IDENT ":" rel_term ";"
rel_term    := IDENT "(" affine "," affine ")"
affine      := product (("+" | "-") product)*
product     := factor ("*" factor)*
factor      := place | literal | "len" "(" place ")"
```

`rel_term`'s head IDENT is one of the `[OP-5]` comparison names (`ile`, `ilt`,
`ige`, `igt`, `ieq`, `ine`), read as a relation and **never evaluated**. `affine`
and `product` have exactly two precedence levels, fixed here. `[GRAM-6]`'s
no-precedence rule is about *evaluated* expressions, where evaluation order, trap
points and ANF are the reason for the restriction; a `bound_stmt` is never
evaluated, has no trap point and produces no value, so the same reason does not
apply - the argument that already lets `[FN-9]` clauses use an operand grammar
the statement grammar does not have. **Division is not admitted in `affine`**; it
reaches the check only through substitution, so that no surface statement can
name a rounding the writer did not write. Within one `fn_decl` every
`bound_stmt` name is unique, exactly as claim names were (`[CLM-1]` 2754's rule,
retained with the claim gone).

#### 3.8.2 The draft sentences

> **`[IND-1]` (no runtime behaviour).** A `bound_stmt` establishes no runtime
> behaviour. It compiles to no instruction, tests nothing, and can neither trap
> nor diverge; it exhibits the empty `[EFF-2]` row and does not count as delivery
> or must-divergence `[GIVE-1]`. A `bound_stmt` whose base or step obligation
> `[IND-5, IND-6]` is not discharged **in the complete state** is a **hard
> error** at that statement's node. **That qualifier is `[IND-8.V]`'s, and it is
> stated here rather than left to be inferred**: a `bound_stmt` is verified per
> proof view, exactly as `[FN-9]` verifies a relation per view, so a statement
> that discharges in the complete state and fails in the S4-blinded state `B` is
> **not** an error - it simply does not publish in `B`. Read without the
> qualifier, every statement resting on a `requires` would be a hard error.
> There is no runtime fallback and no retained check: the language has none.
>
> **`[IND-2]` (position).** A labelled `bound_stmt` is admitted only in the
> leading position of the body of the `loop_stmt` or `for_stmt` carrying that
> label, preceded only by `doc` and other `bound_stmt`s of the same label; any
> other position is a hard error citing `[IND-2]`. An unlabelled `bound_stmt` is
> a **local statement** and is admitted under `[IND-10]`.
>
> **`[IND-3]` (the statement polynomial).** The statement polynomial of a
> `bound_stmt` is obtained from its `rel_term` by moving both sides to the left
> in the normal form `p <= 0` (`ilt` and `igt` contributing the integer `+1`,
> `ieq` producing the two polynomials `p <= 0` and `-p <= 0`), and by expanding
> `affine` and `product` into the canonical multivariate polynomial with integer
> coefficients over `[ENT-2]` terms, monomials sorted by the `[FORM-2]` canonical
> spelling of their factors and then by degree. `ine` is not admitted as a
> statement relation. All arithmetic in `[IND-3]` through `[IND-8]` is over the
> mathematical integers, as `[ENT-6]` 3143 already requires for normalization.
> Three spec-fixed limits apply, **by L1**, to every polynomial `[IND-4]`'s shape
> rule **produces, at the moment it produces it**, and each violation is a hard
> error naming the statement: coefficient or constant magnitude at most `2^127`,
> degree at most 4, and at most 256 monomials. These are spec-fixed limits, not
> implementation choices. **The scope is every shape the pass produces**, which
> is four things: the statement polynomial as written; the group 1 statement
> hypotheses, **as written, at their own admission** - they are head-frame facts
> and are *never* substituted (`[IND-6]`), so no other measurement of them
> exists; the substituted obligation, **after every substitution step and not
> only at the end of the pass**; and clause (b)'s no-wrap equality pair, clause
> (d)'s witness pair and the `[IND-3]` polynomial of each `[IND-6]` clause (i)
> path condition, each likewise after **every step** of the same backward
> substitution.
>
> **A shape is measured whether or not its slot fills, whether or not the
> substitution that produces it later refuses, and whether or not clause (i)
> drops the condition it belongs to.** Measurement precedes every prover
> decision, and no refusal, drop or fill removes a produced shape from the
> measured set or adds one to it. The one constant in any shape that a prover
> decision selects is clause (d)'s `k - 1` against `0`; `k` is a literal of a
> fragment integer type, so `|k - 1| < 2^64 < 2^127` and the magnitude test
> answers identically whichever member applies. No ambient prover contributes a
> coefficient, a degree or a monomial to any polynomial in this scope, and every
> one of these limits is a count the path's text fixes.
>
> They do **not** apply to a fact the prover *offers* for a slot - a derivable
> constant bound, a derivable difference bound. That is content, not shape, and a
> fact that does not normalize inside these limits simply **does not fill the
> slot** (`[IND-7]`, sweep row 19). At the present fact vocabulary that discard is
> a safety net rather than a live route: a constant bound and a difference bound
> are degree 1 with at most three monomials and unit coefficients, and every
> endpoint of every fragment integer type is inside `[-2^63, 2^64)`, so every
> offerable fact already normalizes inside all three limits. The rule is kept
> because it is what keeps L3's premise true under any later widening of the
> fact language. Nor do the limits apply to `[IND-7]`'s certificate arithmetic,
> to `RELAX`, or to `[IND-8]`'s projection arithmetic, none of which is a
> normalization and each of which reads quantities the prover supplies - **each
> instead carries a derived figure, by L3**. The line is **shape against
> content**, and it is the line the whole sweep is drawn on. 3.9.1's
> *reachable-error sweep* is the enumeration that fixes this scope, row by row,
> and states for each limit why it sits on the side it sits on.
>
> **The two rules below are admission conditions on a `bound_stmt`'s own
> `rel_term`, and by L4 they stay there.** When `[IND-6]` clause (i) asks this
> rule to normalize an *ordinary `if` condition*, only the normal form and the
> expansion above apply: a condition that is not a `rel_term`, or whose operands
> do not all have fragment integer types, or that names an element of an
> indexable place, **yields no polynomial**, and `[IND-7]` group (2) gives it one
> slot that is always empty. It is never an error, and no loop is rejected for
> carrying a `bound_stmt` beside an ordinary branch on a boolean, a handle or a
> subscript.
>
> *Typing.* A `rel_term`'s operands may mix fragment integer types, and a
> `product` may multiply terms of different types. `[OP-5]`'s equal-type
> requirement is a rule about an *evaluated* comparison, whose result must have a
> representation; a `bound_stmt` is never evaluated, its relation is read over the
> mathematical integers, and each term contributes the interval of its own type.
> Every operand must still have a fragment integer type, and a signed term
> contributes its own signed interval.
>
> *Vocabulary fence.* **No operand of a `bound_stmt` names an element of an
> indexable place, and no `bound_stmt` quantifies.** `[ENT-2]` 2870(a) is
> unchanged: `factor` admits a place, a literal, or `len` of a place, and a
> subscript suffix is not a place for this purpose. A `bound_stmt` therefore
> cannot state a per-element property (irreducible I5) or a data-structure
> invariant (irreducible I6), and no rule of this design lets an iteration
> notation swallow either.
>
> **`[IND-4]` (body paths and substitution).** A **body path** of a loop is a
> maximal path of the conservative structural normal-control graph `[FN-1]` from
> the loop's body entry to its back edge `[ENT-5.R1]`; a path leaving by `break`,
> `return` or a `propagate` error edge is not a body path and carries no step
> obligation, because it does not reach a later head. A `bound_stmt` whose loop
> has more than **64** body paths is a hard error naming that statement and the
> limit; this is a spec-fixed limit, not an implementation choice.
>
> Along one body path, the **substituted statement polynomial** is obtained by
> **one backward pass** over the path, from its end to its entry, replacing, at
> each `let` or `set` commit whose destination is in the pass's **term set** at
> the moment the pass reaches that commit, that destination by the polynomial of
> the commit's right-hand side. **The pass visits each commit exactly once and
> never revisits one**; after it, every occurrence of every term, in the
> polynomial and in every hypothesis alike, denotes that term's value at the
> path's entry.
>
> **Each substitution step is one measured step.** `[IND-3]`'s three limits are
> tested on each polynomial the pass carries **after every step**, not only at
> the end; a violation is `[IND-3]`'s hard error, naming the statement, the
> commit and the limit. This is L1 and L3 together, and it is the rule that
> bounds the pass's own work: an implementation never materializes a polynomial
> larger than the limits plus one step's expansion, and it never has to decide
> whether an over-limit intermediate might cancel back inside them later.
> **A shape that crosses a limit at an intermediate step is refused even if the
> pass would have ended inside the limits**; 3.9.1's `N6` is that program and
> the price is stated there.
>
> **A refusal never truncates the pass.** Clause (b)'s refusal leaves the witness
> and the pair in place, so the substitution continues past that commit exactly
> as it would have without the refusal, and the produced shape sequence is
> identical at every version. The only clause that *ends* a substitution is
> clause (e) at a `set` destination, where there is no admitted right-hand side
> to substitute - and that test is the grammar of the right-hand side and the
> destination's binder kind, both fixed by the path's text. So *which* shapes the
> pass produces, and therefore everything L1 measures, is a function of the
> path's text alone.
>
> **The term set, and the shape rule.** Every hypothesis this pass introduces has
> a **shape** - the polynomial expression the clause below writes for it - and
> **the shape is present whatever the ambient prover derives**. A slot whose fact
> is not derivable at that commit is *empty*, but its shape's terms are in the
> term set and the commit that would fill them is visited all the same. The term
> set starts as the terms of the polynomial the rule invoking `[IND-4]` hands the
> pass, and gains the terms of each `[IND-6]` clause (i) branch condition as the
> pass reaches that branch. At each visited commit, **in this order**: the
> destination is substituted away, everywhere it occurs, and leaves the term set;
> then the terms of what the clause substitutes for it, and the terms of **every
> hypothesis shape the clause introduces there**, enter the term set. So a clause
> (b) commit whose `P` names its own destination - `set hits = hits +wrap 1` -
> puts that destination back, and an earlier commit to it on the same path is
> reached, which is what the rewritten hypotheses mean.
>
> **The term set, the set of visited commits, the polynomial and the slot list of
> `[IND-7]` are therefore computed from the path's text and the statement alone,
> before any fact is derived.** That is what makes both of `[IND-7]`'s caps
> legal, and it is the sentence the second falsifier run had to be repaired to
> make true: it is not enough that the slot list be syntactic at a fixed visit
> set, because a prover-dependent visit set moves the slot list and moves clause
> (e)'s refusal with it. The admitted right-hand sides and their polynomials are:
>
> (a) `a + b`, `a - b`, `a * b`, and the `Ok`-arm binding of `+checked`,
> `-checked` or `*checked`: the exact polynomial, **unconditionally**. The three
> exact operations carry their own `[ENT-6]` domain obligation at that
> occurrence, which 3.14 step 5 discharges in every accepted program, so on
> every program this rule can affect the exact polynomial *is* the operation's
> value; the `Ok`-arm bindings have **no** domain obligation to discharge and
> `[ENT-6]` 3143 already fixes the arm's value exactly;
>
> (b) `a op b` at a `wrap` or `sat` operation: a **fresh opaque witness term**
> `o` for the destination, together with, as hypotheses, every constant bound on
> that destination derivable in the state immediately after that commit on that
> path, and the **no-wrap equality pair** `o - P <= 0` and `P - o <= 0`, where
> `P` is the exact polynomial of `a op b`. **The pair is introduced in shape
> unconditionally** - its two slots are present, and `P`'s terms enter the term
> set, at every clause (b) commit the pass visits - and the two slots are
> **filled when the state at that commit derives the corresponding exact row's
> `[OP-2]` no-wrap side condition** and **empty when it does not**.
> The side conditions are: for `-wrap` and `-sat`, `b - a <= 0`; for `+wrap` and
> `+sat`, `a - Z <= c1` and `b - Z <= c2` with `c1 + c2 <= max(T)`; for `*wrap`
> and `*sat`, `c1 * c2 <= max(T)` for the corresponding bounds. When the side
> condition does not derive **and** the destination is a `set` destination, the
> substitution **refuses** the statement, with a diagnostic naming that commit
> and the checked spelling that would admit it. **A refusal never withdraws the
> witness or the slots.** The witness term `o`, the two constant-bound slots and
> the pair's two slots are present at every visited clause (b) commit, refusing
> or not; a refusal met while substituting the statement's own polynomial ends
> the check at `[IND-1]`, and the one route on which a refusing commit reaches
> `[IND-7]` at all is `[IND-6]` clause (i)'s **dropped path condition**, where
> the commit is visited, its witness term exists, and its four slots are present
> and empty. **Clause (e)'s `set` refusal is read by the same two sentences**,
> and 3.9.1 works the branch-condition witness;
>
> (c) a `cvt` at a widening conversion: its operand. A copy or literal: that
> atom;
>
> (d) `a / k` for a literal `k >= 1`: a fresh opaque term `q`, together with,
> as hypotheses, every constant bound on the destination derivable in the state
> immediately after that commit, and the **witness pair**, which depends on the
> dividend's sign because 845 fixes exact division as **truncating toward
> zero**. **Both members of the pair name `q`, `k` and `a` in either case, so the
> pair's shape - its two slots and the terms it puts in the term set - is the
> same whichever member applies**, and the sign test below chooses only the
> constant each slot is filled with:
>
> > (d1) when `Z - a <= 0` is derivable in the state at that commit - which every
> > unsigned `a` satisfies from `[ENT-2]` 2905's implicit per-term bounds alone -
> > the pair is `k*q - a <= 0` and `a - k*q <= k - 1`;
> >
> > (d2) otherwise the pair is `k*q - a <= k - 1` and `a - k*q <= k - 1`, which
> > is true of truncation toward zero for either sign of `a`.
>
> (e) **any other right-hand side** - a call, a subscript, a construction, a
> `replace`, a `propagate` or a delivered value. If the destination is a `let`
> binder committed exactly once on the path, the commit contributes a **fresh
> opaque witness term** `o` for that binder, together with, as hypotheses, every
> constant bound on that destination derivable in the state immediately after
> that commit on that path. If the destination is a `set` destination, the
> substitution **refuses**, with a diagnostic naming that commit - and, exactly
> as in clause (b), **a refusal never withdraws the slots, and what it ends
> depends on what was being substituted**: a refusal met while substituting the
> statement's own polynomial, or a witness hypothesis carried into the head
> frame, ends the check at `[IND-1]`; a refusal met while substituting an
> `[IND-6]` clause (i) path condition **drops that condition** and the check
> proceeds, the commit visited and its slots present and empty. Which of the two
> it is turns on where the refusal is met, never on which clause raised it.

Clause (b) is A1's repair; the "one backward pass, never revisits" sentence is
A17's; clause (e)'s witness half is S1's, and it is what lets the I2 example's
`let w = deref(weights)[i];` stand as an opaque term with `w <= 255` rather than
refusing the statement the design was built for.

**Four things in that block are F-I1's repairs, and the last of them is what the
`[ENT-1]` theorem now rests on.** The hypothesis rewriting in the backward-pass
sentence is F-I1's F2: I1's midpoint needs `span` rewritten inside the two
division witnesses, and the sentence as drafted rewrote only "the polynomial", so
`span` survived into `H`, could not be eliminated, and the midpoint was refused
(F-I1 `worksheets/T4_i1_midpoint.md`, reading B). **Clause (a)'s proviso is
deleted**, which is F-I1's F3: a `+checked` has no `[ENT-6]` domain obligation, so
"provided the state discharges it" had a vacuous reading under which `span` and
`mid` both fell to clause (e) and I1 died - A1's own vacuity, re-created one
clause away from where it was repaired. Deleting it rather than splitting it is
sound because the three exact operations carry the obligation themselves and the
`Ok`-arm bindings have none, so the proviso was never doing work on a program that
compiles; and it removes the last prover-dependence from clause (a). Clause (d)'s
split is F-I1's B2, a **soundness** break: with no restriction on the dividend's
sign, `a = -5, k = 2` makes `k*q - a <= 0` false (the language's `q` is `-2`), the
certificate proves `h <= -3` where the truth is `h <= -2`, and
`probes/f2_sdiv_consumer.wf` (compiled) is the accepted program that divides by
zero on the strength of it. 3.9.4 re-derives the midpoint under the repaired
text; 3.9.7 re-derives B2 in both directions, refusing the false bound and keeping
the true one.

**And clause (b) no longer chooses between two term shapes.** As drafted it
substituted the exact polynomial when the no-wrap side condition derived and fell
to clause (e)'s witness when it did not, so the **number of elimination terms**
`[IND-7]` sees was a function of prover strength: one witness term became two
operand terms the day a row image made the side condition derivable, and
`[IND-7]`'s four-term cap is a hard error (F-I1's F7b). Under the text above every
non-exact commit contributes exactly **one** witness term whatever the prover can
derive, and the no-wrap side condition adds two *hypotheses* instead of changing
the polynomial's shape. The elimination-term list is therefore a function of the
path's text and the statement alone. The same move closes F-I1's F7c, the hazard
it named without a witness: the witness bounds are read at the commit, where the
path conditions are in force, and they are now read there on **every** route, so
no strengthening can trade a tight commit-point bound for a loose check-point one.

**The shape rule is the second repair, and it is what closes the visit set.**
The first repair moved the operands out of the polynomial
and into the hypotheses, and at the same time made the pass follow the
hypotheses - the F2 rewriting I1's midpoint genuinely needs, since `span` is
rewritten only because it stands inside the two division witnesses. Individually
right; jointly they re-created the same prover-dependence one level down. With
clause (b)'s equality pair introduced only *when its side condition derived*, the
operands `P` names entered the pass's reachable terms only on a prover that could
derive it, so the **visit set** of the backward pass was a function of the ambient
prover: a strengthening could drag a commit into the pass that the weaker checker
never reached, and there meet clause (e)'s `set` refusal, or spend four more
slots. 3.9.1's `N1` and `N2` are the two programs that did it, and they are
written out there under the repaired text. The repair is the "in shape" sentence:
**a hypothesis's terms and its slot are present whatever the prover derives, and
only its *content* is the prover's.** Clause (d)'s pair already had that property
- `d1` and `d2` name the same three things and differ in one constant - and
clause (b) now has it too, so the visit set is fixed before a single fact is
consulted. `[IND-6]` clause (i) carries the same treatment for the one remaining
route that introduced witness terms of its own.

**It was not the last one, and the third repair is why `[IND-3]`'s limits are
scoped above.** Closing the visit set fixed *which* commits are visited and
*which* slots exist, and left *which certificate steps execute* under the prover
- an empty slot is skipped, a filled one is not - with `[IND-7]` importing
`[IND-3]`'s per-step magnitude limit, a hard error, into exactly that arithmetic.
3.9.1's `N3` is the program that did it. The lesson generalized is the one this
revision acts on: **each repair had been correct about the break in front of it
and had moved the prover-dependence one level down**, so the third is written
against the class rather than the instance. 3.9.1's *reachable-error sweep*
enumerates every hard error and spec-fixed limit reachable while a `bound_stmt`
is admitted and checked, and classifies each; the ones on counts the
path's text fixes stay hard errors, and every one a prover-supplied value can
touch becomes a discard. The `[IND-3]` paragraph above is the first row of that
work: its three limits are limits on **shapes**, and they stop at content.

**The fourth round is why that paragraph says *every* shape.** The third repair
wrote the scope as "exactly those two polynomials" - the statement polynomial and
the substituted obligation - which was too narrow, because clause (b)'s and
clause (d)'s pairs and clause (i)'s path conditions are polynomials **the rule
itself writes** and the backward pass then rewrites. The prover decides only
whether their slots fill; their content is as syntactic as the two the narrow
scope kept, so limiting them is a legal hard error and *not* limiting them left
`[IND-7]`'s resource bound with no premise. `N4` (3.9.1) is the ordinary program
that carried a degree-8 hypothesis into a certificate step, and the price of the
widened scope - `N4` is refused, uniformly and at admission of that shape - is
stated there.

**The fifth round is why the scope is stated as a law rather than as a list, and
it is the round that stopped patching.** The fourth repair widened `[IND-3]` but
delimited the widened scope with the phrase *carried into the head frame*, and a
dropped path condition is not carried into the head frame. That put a hard error
behind clause (b)'s `set` refusal - the one decision a strengthening can flip and
the one the file had already cleared, under the *old* scope, as safe because it
moved only from refusing to admitting. After the widening, admitting also
submitted a new polynomial to a hard-error limit, so a strengthening turned an
ordinary verified statement into a hard error (`N5`, 3.9.1). Three readings of
the same paragraph gave three verdicts on the same program, which is the
determinism defect the fourth round was convened to close. **L1 is the repair,
and it is written as an invariant over the whole pipeline instead of as a scope
list**: what is measured is what the pass *produces*, so a drop, a fill and a
refusal are all content moves and none of them can reach a limit. The
per-step form of the same law is what closes `N6`, the one computation the fourth
round left with no bound at all - the backward substitution's own intermediates,
which nothing limited and which an implementation could not abort early because
cancellation can bring the result back inside the limits. L3 requires the figure;
the per-step test supplies it by construction, and 3.9.1 derives it.

> **`[IND-5]` (base).** For a labelled `bound_stmt` on loop `@l`, the base
> obligation is the statement polynomial checked by `[IND-7]` in the closed state
> at `@l`'s preheader. For a `for_stmt`, it is checked in the closed post-capture
> state of 3120, in which `binder = lower_capture` holds. No `[IND-4]`
> substitution is performed: the preheader state is already the state the
> polynomial's terms denote.
>
> The **statement hypotheses** `[IND-7]` receives for a base obligation are the
> statement polynomials, as written, of the `bound_stmt`s of `@l` that **precede
> this one in textual order** - never this statement itself. The loop's
> statements are checked in textual order and each preceding base has already
> discharged in the same preheader state, so the set is well founded. **No
> `[IND-8]` projection of any statement of `@l` is in the preheader state**:
> `[IND-8]` publishes on the header edges and, for a `loop_stmt`, on the
> body-entry edge, never on the preheader edge.
>
> This is the F-I1 repair B1, and it is a soundness repair. The drafted rule sent
> the base through `[IND-7]` with `[IND-7]`'s own unrestricted hypothesis list,
> whose first group was "the statement polynomials of that loop's `bound_stmt`s",
> so `sigma(t) = H1` with `H1` the statement itself gave `p := |b|*p - |a|*h = 0`
> and **every labelled `bound_stmt` had a vacuous base**. 3.9.7 works the witness:
> `bound @spin lie: ile(idx, 0_u64);` with `idx = 9`, whose consumer
> `j3b_ind6_consumer.wf` (compiled) accepts, writing one byte past a one-byte
> buffer.
>
> **`[IND-6]` (step, and the frame).** The step obligation of a labelled
> `bound_stmt` is one obligation per body path `[IND-4]`. On each body path, take
> the statement polynomial, replace every occurrence of that loop's `for_stmt`
> binder by `binder + 1`, and substitute the path's commits `[IND-4]`. The result
> is a **head-frame polynomial**: every term in it denotes its value at the
> path's entry, which is the loop head.
>
> The obligation is checked by `[IND-7]` **in the head state** - the closed state
> at the loop head under `[ENT-5.R3]`'s order - extended by exactly two sets of
> hypotheses:
>
> (i) the **path conditions**, each branch condition governing that path,
> expressed in the head frame by the same `[IND-4]` backward substitution taken
> from that branch back to the path entry; a path condition whose substitution
> refuses - **by clause (b)'s `set` refusal or by clause (e)'s, the two are
> treated alike** - is dropped rather than refusing the statement. **Dropping is
> a decision about a slot's content, never about the slot list and never about
> the measured set.** By L1 every shape the pass produced while substituting that
> condition - the condition's own polynomial after every step, and every pair the
> commits it reached introduced - was measured against `[IND-3]` at the moment it
> was produced, so a drop removes content and nothing else. Without that
> sentence a `set` refusal, which is the one decision a strengthening can flip,
> would decide whether an over-limit shape is measured, and a strengthening would
> turn a verified statement into a hard error: 3.9.1's `N5` is that program, and
> under L1 it is a hard error at every conforming checker. A branch
> condition's terms
> enter `[IND-4]`'s term set as written when the pass reaches that branch, and
> every commit it thereby reaches is visited and contributes its slots in shape,
> whether or not the substitution refuses and whether or not the condition is
> dropped; a dropped path condition's slots are **present and empty**. Which
> commits a path condition reaches, and how many slots they cost, is therefore
> fixed by the path's text, exactly as for the statement polynomial's own chain -
> without this sentence the same prover-dependence the shape rule closes in
> clause (b) returns through clause (i), because a path-condition substitution
> introduces witness terms of its own and clause (b)'s `set` refusal decides
> whether it refuses; and
>
> (ii) the **witness hypotheses** `[IND-4]` clauses (b), (d) and (e) introduced,
> each carried into the head frame by the same backward pass.
>
> No fact read at the body-exit state enters the check, and no state fact is
> read at any point other than the head. Path-sensitivity is confined to this
> check: **no fact it derives is published anywhere.** Base and step are both
> checked in the flow at the limit retention family `[ENT-5.R5]`, once, after
> deletion has ended.
>
> The **statement hypotheses** `[IND-7]` receives for a step obligation are the
> statement polynomials, as written, of **every** `bound_stmt` of that loop,
> including the statement under check. That is the induction hypothesis, and for
> several statements on one loop it is ordinary mutual induction over iterations:
> each statement is assumed at the head and each step is proved. They **are
> head-frame facts and are never substituted**; they enter `[IND-7]` exactly as
> written, while every other polynomial entering the check is in the head frame
> by the paragraph above. **`[IND-3]` measures them once, as written, at their own
> admission, and there is no second measurement**: the earlier reading, which
> listed group 1 among the shapes limited "after the same backward substitution",
> contradicted this sentence and is withdrawn. Substituting a statement
> hypothesis is exactly what made A2's program verify falsely (3.8.3), so the two
> rules have to agree and this is the sentence they agree on.
>
> **The check state is the state 3.14 step 3 built, and it already carries this
> loop's `[IND-8]` projections.** Step 3's walk applies `[IND-8]` projections as
> an `[ENT-3]` source; step 4 then verifies every `bound_stmt`. So the head state
> in which a step obligation is checked, and every state along the body path in
> which an `[IND-4]` clause (b) side condition, a clause (d1) sign test
> `Z - a <= 0`, or a clause (b)/(d)/(e) witness bound is read, is a state in
> which this loop's statements are **assumed**. That
> is sound and it is assume-guarantee, not circularity: the assumption is the
> induction hypothesis, discharged at the head by `[IND-5]`'s base, which by the
> paragraph above is checked in a preheader state carrying **no** projection of
> the statement under check.

`[IND-6]` is A16's and A2's repair together, and its two operative sentences are
the ones the drafted rules got wrong in opposite directions. Reading the state at
the body exit bounds a head-frame term by its exit value - the out-of-bounds
write `j3`/`j3b` compile. Substituting the statement's own polynomial makes the
hypothesis equal the goal modulo the binder shift, and the body's effect on the
accumulator cancels - the silent unsigned overflow of A2's program. **One frame,
named, with everything translated into it, is the whole content of both
repairs.**

The closing paragraph is F-I1's F5. The drafted rule said the head state was
"extended by exactly two sets of hypotheses" and said nothing about the
projections, which reads *against* the pipeline; and I4's own worked step says
out loud that its `+wrap` side condition comes "through the published
`hits - i <= 0`". The dependency is real - `s22`, `s23` and `L08` all reject
without the projection - so the rule has to say the thing 3.14 was already doing.

#### 3.8.3 The frame, worked against the two breaking programs

**A16's program.** `bound @l s: ile(x, 0_u64);` in a loop whose body is
`set out[x] = 1_u8; set x = cursor; set cursor = 0_u64;` with `cursor = 7`
before the loop. One backward pass over the body path: at `set cursor = 0_u64`
the polynomial is `x` and `cursor` does not occur, so nothing happens; at
`set x = cursor` the destination occurs and `x` is replaced by `cursor`. The pass
ends with `p = cursor`, denoting the **path-entry** value of `cursor`. Under the
drafted rule `[IND-7]` relaxed it in the body-exit state, where `cursor = 0`,
`C = 0`, verified - and `x = 7` at the second head, one byte past a one-byte
buffer. Under `[IND-6]` the check reads the **head** state, where the entry value
`cursor - Z <= 7` is what is derivable, `RELAX` gives 7, and the statement is
**refused**. `j3_ind6_checkpoint_break.wf` (compiled) rejects on exactly the
obligation the false publication would have discharged, and
`j3b_ind6_consumer.wf` (compiled) **accepts** with the fact supplied - the pair
is the break, and the repair turns it back into a rejection.

**A2's program.** `bound @weigh per_byte: ile(sum, 255_u32 * i);` in a loop whose
body is `set sum = sum + 1000_u32;`. Step: `p0 = sum + 1000 - 255*i - 255`.
Under the drafted closing sentence the hypothesis is `subst(P) = sum + 1000 -
255*i`, so `p := 255*p0 - 255*H1 = -65025` and the statement verifies although it
is false at `i = 1`. Under `[IND-6]` the hypothesis is the statement polynomial
**as written**, `H1 = sum - 255*i`, so
`p := 255*p0 - 255*H1 = 255*1000 - 255*255 = 255*745 = 189975`, `RELAX` cannot
reduce a positive constant, and the statement is **refused**. The published fact that
would have discharged the exact `+`'s `[OP-2]` obligation never exists.

**Both traces were re-executed against the repaired text and neither moves.**
A16's path substitutes only by clause (c) - a copy and a literal - which
introduces no witness and no hypothesis, so `p0 = cursor` stands and `RELAX`
still reads `cu(cursor) = 7` at the head. A2's `set sum = sum + 1000_u32` is
clause (a), now unconditional, so `p0` is unchanged and the certificate still
lands on 189975 - `255*(1000 - 255)`, the figure both earlier revisions of this
file and F-I1's `worksheets/T7-T8_refusals.md` printed as 190125, corrected here
and positive either way, so the refusal never depended on it; the exhaustive
search does **not** resurrect FF2, because the
only hypothesis that would have worked is the substituted polynomial and
`[IND-6]` removes that from the space entirely - F-I1 enumerates every
certificate of A2's obligation in `worksheets/T7-T8_refusals.md` and all four
fail. Clause (a) contributes no carried bounds (it introduces no witness term),
so the repair adds nothing to either space. **The shape rule leaves both traces
alone for the same reason**: it puts terms in the term set only where a clause
introduces a hypothesis, and neither path has a clause (b), (d) or (e) commit, so
neither visit set nor slot list moves.

#### 3.8.4 What `[IND-4]`'s refusals cost, stated

Refusing at an unsubstitutable `set` is the conservative half, and it is where a
writer will meet the rule. A body that writes an accumulator through a call
result (`set acc = fold(acc: acc, x: v);`) has no statement route and goes to the
callee's `ensures` instead - which is the right home, because the callee is what
knows the step. A body whose accumulator is written by a `replace` of a whole
place has neither route and goes to `if`/`else` with the price of 3.12. Those are
honest limits and section 11 keeps them.

The third refusal is clause (b)'s: a `set` written by a `wrap` or `sat` operation
whose no-wrap side condition does not derive at that commit. That one has a named
repair - the checked spelling, which the diagnostic prints - and it is the only
`[IND-4]` refusal a prover strengthening can lift. Lifting a refusal moves a
program from rejected to accepted, which is the direction `[ENT-1]` permits.

**That clearance is L2's whole content, and it is only sound with L1 beside it.**
The fifth falsifier run broke it in the gap between the two: with the measured
set delimited by what is *carried into the head frame*, admitting no longer moved
only from refusing to admitting - it also submitted a polynomial to a hard-error
limit that a refusal would have kept out of the measured set, so the one
prover-flippable decision in the pipeline gated a class-(a) hard error and `N5`
went from verified to hard error under a strengthening. Under L1 there is no such
gate: the shape was measured when the pass produced it, at both versions, so
admitting adds content and nothing else. **The clearance below is therefore
stated as *refusing and admitting measure the same set*, not as *admitting only
adds a hypothesis*,** and that is the sentence the next round should attack.

**And it is the only such refusal because of the shape rule, not by inspection of
the clauses** - it is row 14 of 3.9.1's reachable-error sweep, the one row there
that is neither syntactic nor a discard, admitted because its only prover-driven
move is from refusing to admitting and because by L1 that move changes no
measurement. Clause (e)'s refusal tests two syntactic things -
the right-hand side's grammar and whether the destination is a `set` destination
- so it cannot flip under a strengthening *provided the pass reaches the same
commits*. That
proviso is the whole content of the second repair: with a prover-dependent visit
set, a strengthening that made a clause (b) side condition derivable dragged the
operands into the pass's reach and drove it into a clause (e) `set` commit the
weaker checker never saw, turning an accepted program into a hard error without
any clause changing its mind. 3.9.1's `N1` is that program, and under the shape
rule it is refused identically by every conforming checker. **The price is
stated where it is paid**: the reach of the pass is now the widest it can be for
the path's text, so a `set` written through a call anywhere in the operand chain
of a substituted `wrap` refuses the statement even on a checker that would never
have looked. The route is unchanged and it is the right one - the callee's
`ensures` is what knows the step.

### 3.9 `[IND]` part two - the check, the publication, and the local form

#### 3.9.1 `[IND-7]` - the check, as a certificate

This is A4's repair and it is the one place where I replaced a drafted rule
rather than adding a side condition to it; section 2.4 is the reasoning. The
drafted rule prescribed *which* hypothesis eliminates each term and was therefore
non-monotone under prover strengthening, falsifying `[ENT-1]`'s new sentence on
the day the construct lands. This rule asks instead whether a certificate
exists.

**Everything from here to the end of this subsection is one derivation from
3.8's four laws**, and it is written that way because five rounds of repairing it
locally left the argument one step behind the text every time. L1 fixes what is
measured and counted; L2 fixes what a prover may move; L3 requires a derived
figure for every computation the check performs; L4 keeps the statement's
admission rules off ordinary branch conditions. The caps and the step rule below
are L1's instances, the resource figures are L3's, the *reachable-error sweep* is
the row-by-row audit that the four laws actually cover every error the pipeline
can raise, and the five-part *Monotonicity* argument is derived from the laws
rather than re-inspected clause by clause. `N1` through `N6` are the six programs
the derivation is answerable to, and each is worked under this text.

> **`[IND-7]` (the check).** Let `p <= 0` be the obligation after `[IND-4]` and
> `[IND-6]`. Its **elimination terms** are the degree-1 monomials of `p` with
> nonzero coefficient, in `[FORM-2]` canonical spelling order with `[IND-4]`'s
> witness terms first in order of introduction; there are at most **4**, and more
> is a hard error naming the statement. **That count is a function of the path's
> text and the statement alone, and so is the visit set that produces it**:
> `[IND-4]`'s shape rule fixes which commits the pass visits before any fact is
> derived, clauses (a) and (c) substitute unconditionally, and clauses (b), (d)
> and (e) each contribute exactly one witness term whatever the ambient prover
> derives, so no strengthening can cross the cap.
>
> **The elimination terms are not `[IND-4]`'s term set, and the two rules read
> different sets.** `[IND-4]`'s **term set** is a set of `[ENT-2]` *terms* - the
> atoms a monomial is a product of, which is what `[IND-3]` means by "over
> `[ENT-2]` terms" - and **the term set is the set the visit rule reads**: a
> commit is visited when its destination is in the term set, whatever the degree
> of the monomial that destination occurs in. `[IND-7]`'s **elimination terms**
> are the degree-1 *monomials* of `p`, and they are what the elimination list,
> the 4-term cap and group 3's ordered pairs read. The second is a subset of the
> first, and a proper subset whenever a term occurs only inside a monomial of
> degree 2 or more.
>
> The **hypothesis list** `H` is an ordered list of **slots**, each either filled
> with one polynomial `h` known to satisfy `h <= 0`, or **empty**. Its slots are,
> in this order:
>
> (1) one slot for each **statement hypothesis the invoking rule supplies** -
> `[IND-6]` supplies every `bound_stmt` of that loop, `[IND-5]` supplies the ones
> textually preceding the statement under check, and `[IND-10]` supplies none -
> filled with that statement polynomial as written;
>
> (2) **one slot for each `[IND-3]` polynomial of each path condition of
> `[IND-6]` clause (i)** - two for an `ieq` condition, one for any other
> `rel_term`, and one slot for a branch condition `[IND-3]` gives no polynomial
> at all (a disequality, or a condition that is not a `rel_term`), that last slot
> being always empty; and, **at each `[IND-4]` commit the pass visits whose
> governing clause is (b), (d) or (e)**, two slots for that destination's
> derivable constant bounds, plus, **at every such commit governed by clause (b)
> or clause (d)**, the two slots of that clause's no-wrap equality pair or
> witness pair. **Which clause governs a commit is fixed by the commit's
> right-hand side alone, and whether the pass visits it is fixed by `[IND-4]`'s
> shape rule**, so all four counts are syntactic; in particular the pair's two
> slots are present at every visited clause (b) and clause (d) commit whatever
> the prover derives. A slot whose fact is not derivable at that commit is
> **empty**, and so is a slot at a commit reached only through a path condition
> `[IND-6]` clause (i) dropped - **the commit is visited and its witness term
> exists all the same**, by clause (b)'s own sentence, so no slot in this group
> ever names a term that does not exist;
>
> (3) for each **ordered pair of distinct elimination terms** - twelve slots at
> four terms, six at three, two at two, none at one - the tightest difference
> bound `t1 - t2 <= c` derivable at the check point `[ENT-4]`, the slot being
> **empty** when none is.
>
> `H` has at most **32 slots**, and more is a hard error naming the statement.
> **Both caps are on counts `[IND-4]`'s shape rule produces**: the visited
> commits, the elimination terms and the slot positions are all computed from the
> path's text and the statement before any fact is consulted. Which slots are
> **filled** depends on the ambient prover; which commits are visited, how many
> slots there are, and where each sits does not. **The slot count being syntactic
> at a fixed visit set is not enough and was the second falsifier's break**: the
> visit set has to be syntactic too, or a strengthening moves the count.
>
> A **certificate** is a partial injection `sigma` from the elimination terms to
> the slots of `H`, together with the derived multipliers. Processing the
> elimination terms in order, for a term `t` with current coefficient `a` in `p`
> and `sigma(t)` a slot filled with `h` whose coefficient on `t` is `b`: the step
> is **admitted** when `a*b > 0`, and sets `p := |b|*p - |a|*h` and `s := s*|b|`.
> If `sigma(t)`'s slot is empty, or `a` is zero, or `a*b <= 0`, **the term is
> skipped and `p` and `s` are unchanged**; a term outside `sigma`'s domain is
> likewise skipped. `s` starts at 1. The certificate **succeeds** when
> `floor(RELAX(p) / s) <= 0` at the end.
>
> **Nothing inside this check raises a hard error.** The step arithmetic is exact
> over the mathematical integers and carries **no limit of its own**: `[IND-3]`'s
> three limits scope to the polynomials the shape rule produces and stop there,
> so a step may produce a coefficient, a constant or a monomial count past any of
> them without consequence. That is not a licence for unbounded work, and the
> bound is derived here rather than asserted. Write `M = 2^127` for `[IND-3]`'s
> magnitude limit and `k` for the number of steps.
>
> *The pass, and its intermediates.* **L3 applies first to the computation that
> runs before the check**, `[IND-4]`'s backward substitution, and the figure is
> supplied by construction rather than by analysis: `[IND-3]`'s limits are tested
> after **every** substitution step, so no polynomial the pass carries is ever
> over-limit when a step begins. What one step may materialize before its result
> is tested is then bounded by the clause list. **Every right-hand side any clause
> substitutes is at most 2 monomials of degree at most 2 with coefficients under
> `2^64`** - clause (a)'s `a + b`, `a - b`, `a * b` and their `Ok`-arm forms,
> where an operand may be a literal of a fragment integer type; clause (c)'s atom;
> and clauses (b), (d) and (e)'s single witness term. Replacing a degree-1 term
> `t` in a polynomial of at most 256 monomials of degree at most 4: a monomial
> carrying `t^e` has `e <= 4`, expands to at most `2^e <= 16` monomials of degree
> at most `4 + e <= 8`, and its coefficient grows by at most
> `6 * (2^64)^4 < 2^259`. So **one step materializes at most `256*16 = 4096`
> monomials of degree at most 8 with coefficients under `2^127 * 2^259 = 2^386`,
> and under `2^398` after collection**, and then the limits are tested. The
> **number of steps** is at most the number of commits on the body path, each
> visited once, times the number of shapes the pass carries - the statement
> polynomial, at most two per branch condition, and at most four per visited
> commit. Every one of those is a count of the path's text, so by L1 no prover
> contributes to the pass's cost. `N6` below is the program this paragraph exists
> for, and its price is stated there.
>
> *The premise.* Every polynomial a step reads is inside `[IND-3]`'s limits when
> the check starts. The obligation is, by `[IND-3]`. **So is every filled slot's
> own polynomial**, and for the two disjoint reasons the widened scope supplies:
> a slot filled with a **shape** - group 1's statement hypothesis, clause (b)'s
> or clause (d)'s pair, a clause (i) path condition - is a polynomial `[IND-3]`
> limits, its limits being hard errors that fire before any certificate is tried;
> and a slot filled with a **fact the prover offered** is filled only if that
> fact is `[IND-3]`-normalizable, by the *discard* paragraph below, and is empty
> otherwise. There is no third kind of filled slot. **This premise is exactly
> what the fourth falsifier run destroyed and L1 restores**: with `[IND-3]`
> scoped to two polynomials, a rule-written pair could carry any degree at all
> into a step, and every figure below was false (`N4`). It survives the fifth run
> unchanged for a reason worth stating - a **dropped** condition never fills a
> slot, so neither reading of the drop question could touch this premise, and
> that is why `N5` is a monotonicity break and not a resource one.
>
> *The bound.* There are at most 4 elimination terms, so `k <= 4`. **Degree never
> rises**, because `|b|*p - |a|*h` has the degree of its operands and both are at
> most 4. The **monomial count** is at most `256*(k+1) = 5*256`, since each step
> adds the monomials of one `h` to those of `p`. A **coefficient or constant** is
> at most `(2M)^(k+1) = (2*2^127)^5 = 2^640`: a step replaces a bound `C` by at
> most `|b|*C + |a|*M <= M*C + C*M = 2M*C`, so **each step multiplies the bound
> by at most `2M = 2^128`** - it does not double it - and `k` steps from `C = M`
> give `(2M)^k * M <= (2M)^(k+1)`. The multiplier `s`, a product of at most four
> `|b|`s, is at most `M^4 = 2^508`. **`RELAX` on the residual is bounded by the
> same data, though by no limit** (row 20): a factor's interval endpoint is at
> most the wider of its type's bound and an `[IND-3]`-normalizable constant,
> `M` - the integer type set is closed at `i8..u64` (812), so every type endpoint
> is at most `max(u64) < M` - so a degree-4 monomial's corner product is at most
> `2^640 * M^4 = 2^1148` and the sum of at most `5*256` of them is under
> `2^1159`. **`[IND-8]`'s corner *minimum* takes the same derivation on smaller
> data and gets its own figure** (row 23): it is computed from the statement
> polynomial **as written**, whose coefficients `[IND-3]` limits to `M`, so a
> monomial's corner minimum is at least `-(M * M^4) = -2^635` and the sum of at
> most 256 of them is above `-2^643`. **Those bounds are computed from the caps,
> `[IND-3]`'s limits and the type widths alone, and no prover contributes to any
> of them**, which is
> what makes exact arbitrary-precision arithmetic here affordable and is why it
> needs no limit, and so no error, of its own. **With the pass's figure above,
> every computation `[IND]` performs now has one**, which is L3 discharged and is
> the claim the next falsifier run should attack first.
>
> **Where a quantity the prover supplies would violate a limit, the affected slot
> or certificate is DISCARDED, never raised.** A fact offered for a slot that is
> not an `[IND-3]`-normalizable polynomial does not fill it; the slot is
> **empty**. A certificate a conforming implementation cannot evaluate does not
> succeed; the check moves to the next one - **and the bound above is what makes
> that sentence dead letter, which is the only thing that makes it legal**. Every
> certificate in the space is evaluable within four steps, degree 4, `5*256`
> monomials, coefficients under `2^640` and a residual `RELAX` under `2^1159`,
> and exact evaluation inside those bounds is required of
> a conforming implementation, so no conforming implementation has a certificate
> it cannot evaluate and the sentence can never decide a program. An
> implementation that declines work inside those bounds is not conforming:
> **compiler capability is not a source-language rejection**, and with the bound
> false - as it was before `[IND-3]`'s scope was widened - this sentence let two
> conforming implementations decide different predicates on `N4`. **The same
> conformance requirement covers the pass**, whose figure is the paragraph above:
> exact evaluation of one substitution step inside 4096 monomials, degree 8 and
> `2^398` is required, and an implementation that declines it is not conforming
> either. That is the half the fifth run found missing - a certificate-scoped
> dead-letter argument said nothing about a computation that runs before any
> certificate exists (`N6`). This is
> otherwise exactly `[IND-8]`'s treatment
> of an over-magnitude published constant - *not published, and publication never
> raises a hard error* - stated here for the check, and the *reachable-error
> sweep* below is the enumeration that holds the two rules to it.
>
> The obligation is **discharged exactly when some certificate succeeds.** The
> certificate space is the set of partial injections from at most 4 terms into at
> most 32 slots - `sum_k C(4,k)*P(32,k) = 988,161` at the caps, and 1,021 on the
> largest space this design works (I1's midpoint: three terms, ten slots; the
> largest slot count is I4's matching step path at eleven slots over two terms,
> whose space is 133).
> It is finite and fixed by this rule. **Acceptance is an existential over that
> fixed space**: the statement's obligation verifies when **at least one**
> certificate in the space succeeds, and is refused when none does. **Enumeration
> order cannot change that predicate.** Every way a certificate can fail to
> succeed is a *failure or a discard* confined to that certificate - an empty
> slot, a wrong-signed coefficient, a residual that stays positive, a slot no
> admissible fact filled - and by the paragraph above no certificate raises a
> hard error, publishes a fact, or leaves any trace an implementation could carry
> into the next one. So an implementation may enumerate the space in any order,
> stop at the first success, prune, or evaluate every member, and it computes the
> same value; **there is no order in which a program is accepted and another in
> which it is a hard error.** Because a skipped step leaves `p` and `s`
> unchanged, **the certificate that simply omits that term reaches the same
> residual**, so an implementation may drop any assignment whose first step is
> skipped without changing the predicate; that, not the raw space, is what makes
> the enumeration cheap. **Two conforming implementations that derive the same
> facts at the check point therefore decide the same predicate on the same
> inputs**, and the *Monotonicity* paragraph below is what covers the case where
> they do not. **That sentence rests on two things, not one**: on the order
> argument just given, and on the resource bound above, because the *discard*
> paragraph's capability sentence is a **second** implementation-chosen degree of
> freedom and only the bound makes it dead letter. `[ENT-1]` 2835-2836 forbids an
> implementation-chosen **strategy**, not only an implementation-chosen order,
> and the fourth falsifier run refuted this sentence through the second freedom
> while the first held (`N4`).
>
> `RELAX(p)` is the sum, over the monomials of `p`, of the maximum of that
> monomial's interval, where a monomial's interval is the product of its factors'
> intervals **taken as the maximum over the corner products** - for a degree-d
> monomial the maximum over the `2^d` products of endpoints, the monomial's
> integer coefficient included as a factor - and the empty product being 1. A
> factor `t`'s interval is `[cl, cu]` for the tightest constant bounds
> `Z - t <= -cl` and `t - Z <= cu` **derivable at the check point in the state
> the invoking rule checks in - `[IND-5]`'s preheader or post-capture state,
> `[IND-6]`'s head state as that rule extends it, or `[IND-10]`'s state - or
> standing in a filled slot of `H`**. **`[IND-5]` is named because every base
> obligation in this file is decided by `RELAX` in that state**; without it a
> base's factor intervals have no source. Where
> neither source supplies a bound, the interval is the interval of `t`'s own
> fragment integer type; a `[IND-4]` witness term takes the type of the
> destination it stands for. **`RELAX` is total, is defined on every term
> including every witness term, and raises no hard error.** Every quantity is a
> mathematical integer, and **no `[IND-3]` limit applies to `RELAX`**: a
> degree-4 monomial over four `u64` factors has corner products near `2^256`, so
> a magnitude limit here would fire on ordinary arithmetic rather than on an
> abuse. Where a factor's interval is wider than any limit, `RELAX` returns the
> larger number and the certificate simply fails; that is a discard, not an
> error.
>
> *Soundness.* Each admitted step replaces `p` by `|b|*p - |a|*h` with `|b| > 0`,
> `|a| > 0` and `h <= 0`, so the new polynomial is at least `|b|` times the old;
> a skipped step changes neither `p` nor `s`. The accumulated `s` therefore
> satisfies `p_final >= s * p_0`. `RELAX` over-approximates `p_final`, so
> `s * p_0 <= RELAX(p_final)`; `p_0` takes integer values, so
> `p_0 <= floor(RELAX(p_final) / s)`. The final test therefore proves `p_0 <= 0`.
>
> *Monotonicity.* Let a **strengthening** be any change to the ambient prover
> that makes more facts derivable at more points and none fewer. Then:
>
> (i) *the produced shapes, the visit set and the obligation are unchanged, and
> so is everything measured.* This is **L1**, and it is stated over the pass's
> output rather than over the visit set alone, because the fifth run broke the
> narrower version. `[IND-4]`'s term set, the set of commits the pass visits, the
> sequence of polynomials it carries, `p` and the elimination-term list are
> computed by the shape rule, which reads the path's text and the statement and
> never the prover: clauses (a) and (c) substitute unconditionally, clauses (b),
> (d) and (e) each contribute exactly one witness term, and **every hypothesis a
> visited commit introduces is introduced in shape**, so the terms it puts in the
> term set - the operands `P` names at a clause (b) commit, the dividend at a
> clause (d) commit, the branch condition's terms at a clause (i) path
> condition - are there whatever the prover derives. **A refusal does not
> truncate the pass** (`[IND-4]`), so the shape sequence does not shorten or
> lengthen under a strengthening either, and by L1 every shape in it is measured
> against `[IND-3]` whichever way every refusal, drop and fill goes. **So the set
> of `[IND-3]` tests performed, and their answers, are identical at both
> versions.** The only `[IND-4]` decision a strengthening can flip is clause (b)'s
> refusal on a `set` destination, which it flips **from refusing to admitting**,
> and by the sentence just given that move changes no measurement - which is
> exactly what was false before L1 and is what `N5` broke. Clause (e)'s refusal
> tests only the right-hand side's grammar and the destination's binder kind, at
> a commit the shape rule has already fixed as visited, so it cannot flip at all.
>
> (ii) *the slot list is unchanged, and no slot is emptied.* Every slot position
> is an output of the same shape computation: one per supplied statement
> hypothesis, one per branch condition governing the path - one per `[IND-3]`
> polynomial it yields, and one always-empty slot where by **L4** it yields none -
> two constant-bound slots at each visited clause (b), (d) or (e) commit and two
> more at each visited clause (b) or clause (d) commit, and one per ordered pair
> of elimination terms. None of those counts consults the prover, so **neither cap
> can be crossed by a strengthening**. A strengthening can only fill a slot that
> was empty or tighten a slot that was filled - **L2**, whose enumeration of
> prover-touched sites is the constant bounds, the difference bounds, the
> `RELAX` intervals, clause (d)'s `d2 -> d1` constant, and clause (b)'s refusal.
> There is no sixth site.
>
> (iii) *filling or tightening a slot never loses a certificate.* Tightening the
> constant of a filled slot makes that hypothesis polynomial larger, hence
> `-|a|*h` smaller, hence the residual smaller, so a certificate that named it
> still succeeds. For a slot that goes from **empty to filled** the argument is
> **not** that prior certificates are untouched - a certificate that named that
> slot skipped it before and may not skip it now, and its residual moves. It is
> the **omitting certificate** of the paragraph above: because a skipped step
> leaves `p` and `s` unchanged, the certificate that drops that term from
> `sigma`'s domain reaches exactly the residual the old certificate reached, and
> by (i) and (ii) it lies in the same space at both versions, since the space is
> the set of partial injections over the unchanged term list into the unchanged
> slot list. So whatever succeeded before still has a witness that succeeds.
> Clause (d)'s `d1` pair is exactly `d2`'s pair with the constant of one member
> tightened from `k - 1` to `0`, with the same coefficients, so the `d2` -> `d1`
> move is the tightening case.
>
> (iv) *tightening a `RELAX` interval never loses a certificate*, because
> `RELAX(p)` is monotone in each factor interval and the final test is monotone
> in `RELAX(p)`.
>
> (v) *no step a strengthening newly executes can lose a certificate, and no step
> can raise an error.* Parts (i) to (iv) are about the **space**; this one is
> about the **work done inside it**, which is where the theorem was false the
> third time. Filling an empty slot turns a step that was **skipped** into one
> that **executes**, and an executing step's arithmetic is `p := |b|*p - |a|*h`
> with `|b|` a coefficient the program's own text supplies, so a fill can make
> the arithmetic arbitrarily larger than it was. Two things close it. First, by
> the *no hard error* paragraph above **an executing step cannot raise anything**:
> the step carries no limit of its own, and every remaining way to fail - an
> unfillable slot, a residual that will not close - is a discard confined to that
> one certificate. Second, let `sigma` succeed at the weaker version and let
> `sigma'` be `sigma` with **every term whose slot was empty then and is filled
> now dropped from its domain**. By (i) and (ii) `sigma'` is in the same space at
> both versions; it skips exactly where `sigma` skipped by emptiness; and each
> step it does run uses a slot filled at both versions, whose content at the
> stronger version is the same or tighter - and a tightening moves a **constant**
> only, since a constant-bound slot and a difference-bound slot both carry fixed
> `+1`/`-1` coefficients and group 1, group 2's pair and the path conditions are
> fixed polynomials. So by (iii) `sigma'` reaches a residual no larger than
> `sigma` reached at the weaker version, which was already at most zero.
> **A previously succeeding certificate therefore either still succeeds, or the
> omitting certificate does**, and `N3` below is the pair this part is answerable
> to.
>
> **Therefore no fact-source or closure strengthening can refuse a statement an
> earlier conforming checker verified**, which is what `[ENT-1]` requires of every
> construct. The five parts compose in one direction, and the composition is the
> four laws in order. By **L1**, restated as (i) and (ii): the produced shapes,
> the measurements taken on them, the visit set, the elimination terms and the
> slot positions are one syntactic computation, so the certificate space is
> *literally the same set* at both versions and every `[IND-3]` test has the same
> answer at both. By **L4**: a branch condition outside the fragment contributes
> an always-empty slot at both versions rather than an error at either. By
> **L2**: the only prover-driven move on acceptance is clause (b)'s refusal, in
> the permitted direction, and every other prover move fills or tightens a slot.
> By **L3** and the *reachable-error sweep* below: every hard error still
> reachable anywhere in the pipeline is on a count L1 fixes, and every computation
> inside the space has a derived figure, so neither a limit nor an
> implementation's resources can decide a program. And by (iii), (iv) and (v)
> each fill and each tightening leaves the weaker version's succeeding
> certificate with a witness inside that same space. **The theorem is derived from
> the laws, not re-inspected clause by clause, and that is the difference between
> this statement of it and the four before it.**
>
> It is stated over the whole pipeline because the drafted version has been false
> **four** times inside the certificate form. F-I1 found the two caps were hard
> errors on counts a strengthening could grow, and A4 had merely moved from the
> elimination order to the caps; the repair of *that* left `[IND-4]`'s **visit
> set** prover-dependent, which put clause (e)'s refusal and the slot count
> straight back under the prover's control; the repair of *that* left the
> certificate's own **arithmetic** prover-dependent, because `[IND-3]`'s per-step
> limits were imported into a space in which which steps execute is decided by
> which slots the prover filled; and the repair of *that* - a scope widened but
> delimited by *carried into the head frame* - put a class-(a) hard error behind
> the one prover-flippable decision, so a **dropped** path condition's shape was
> measured on one checker and not on the other. `N1`, `N2`, `N3` and `N5` below
> are the four programs that did it, worked under the text above, and this
> argument is answerable to them. **A fourth run attacked all five parts
> directly, including the covering lemma of (v) three separate ways, and broke
> none of them**; what it broke was the *determinism* half of the same law,
> through the resource bound stated above (`N4`), and the fifth run broke
> monotonicity again at the seam between the two classes and found `N6`, the one
> computation with no bound at all. Both are worked below.
>
> *This is a fixed incomplete fragment, and it is meant to be.* `[ENT-1]`
> 2835-2836 forbids an implementation-chosen strategy, not an incomplete one. A
> statement the fragment cannot verify is a compile error naming the residue, and
> the writer's routes are to restate the bound, to strengthen a `requires`, or to
> write the `if` with an honest `else` arm.

**The reachable-error sweep.** The theorem has been written down false **six**
times counting A4's elimination order, and five times inside the certificate
form - four through monotonicity and one through determinism. Each repair moved
the prover-dependence one level down and each left the argument one step behind
the text, so this revision derives the table from the **four laws** instead of
re-enumerating it: every row below is licensed by L1, L2, L3 or L4, and the last
column names which. **The table is regenerated from the final rule text and
renumbered in pipeline order** - admission of the statement, production by the
backward pass, the check's entry counts, the work inside the check, publication,
and the verdict - because the previous round's append-only numbering had put the
newest rows five places away from the rows they belong with, and a table whose
order does not match the pipeline is how a stale clearance survives a repair.

The **scope** is every hard error and every spec-fixed limit reachable while a
`bound_stmt` is **admitted, produced and checked** - `[IND-1]` through `[IND-10]`,
3.8.1's naming rule, and `[IND-4]`'s substitution pass, whose per-step limits are
new this round and whose intermediates were the one computation the previous
round left unbounded. A row is **(a) syntactic** when its count is computable
from the path's text and the statement alone, before any prover decision, by
L1 - those stay hard errors, and the last column says why the count cannot move
with the prover. A row is **(b) content-reachable** when a value the prover
supplies touches it - a filled slot, a `RELAX` interval, a certificate step's
arithmetic - and **none of those raises**: the slot is simply empty, or that
certificate simply does not succeed. Row 18 is the one where even a discard would
move with the prover, since a discard on a step is a certificate lost to a
strengthening that filled a slot, so the limit is **removed** from that
arithmetic and replaced by a derived figure. Row 14 is neither class and says so;
by L2 it is the only such row, and by L1 it now gates no measurement. **A new row
is a break**, which is what makes this table the falsifier target (F-I2, 3.9.7)
rather than a summary - the fourth run found four rows missing, all admission-time
and all harmless, and the fifth found none missing but two rows *misclassified*:
the drop row was carrying a hard error behind it, and the pass had a computation
with no row at all.

| # | limit or hard error | raised where | class, law | why it stays, or what happens instead |
| --- | --- | --- | --- | --- |
| 1 | `[IND-2]` leading-position rule | admission | (a), L1 | reads the statement's position among its loop's body statements; no fact is consulted |
| 2 | `[IND-10]` same-region restriction | admission of a local statement | (a), L1 | the region is a syntactic run of statements ending at the `bound_stmt` |
| 3 | `[IND-3]` `ine` not admitted as a statement relation | admission | (a), L4 | reads the `rel_term`'s head name; no fact is consulted |
| 4 | `[IND-3]` *Typing*: every operand of the **statement's** `rel_term` has a fragment integer type | admission | (a), L4 | a type test on the operands as written. **By L4 it does not travel to a branch condition**: an ordinary `if` on a boolean, a handle, a call result or a subscripted place yields `[IND-3]` no polynomial and one always-empty slot (`[IND-7]` group 2), never an error on the loop that carries the statement |
| 5 | `[IND-3]` *Vocabulary fence*: no subscripted operand and no quantification, **in the statement** | admission | (a), L4 | a grammar test on the statement as written; `[ENT-2]` 2870(a) decides it. Row 4's L4 fence applies here identically, and the pair of them is what the fifth run read as two hard errors on ordinary branches |
| 6 | 3.8.1's per-`fn_decl` uniqueness of `bound_stmt` names | admission | (a), L1 | a name comparison over the function's text, `[CLM-1]` 2754's rule retained |
| 7 | `[IND-3]`'s three limits on the **statement polynomial as written**, and on each group 1 statement hypothesis as written | admission of that statement | (a), L1 | the polynomial the writer wrote. Group 1 is **never substituted** (`[IND-6]`), so this is its only measurement, and the earlier reading that measured it "after the same backward substitution" is withdrawn |
| 8 | `[IND-4]` 64 body paths | path enumeration | (a), L1 | a count over the `[FN-1]` conservative structural normal-control graph; no derived fact prunes an edge |
| 9 | `[IND-3]` magnitude `2^127` | **after every substitution step**, on every shape the pass produces: the substituted obligation, clause (b)'s and clause (d)'s pairs, and each clause (i) path condition | (a), L1 | every polynomial in that scope is the shape rule's output: clauses (a)-(e) substitute polynomials fixed by each commit's right-hand side, and the pairs and path conditions are written by the rule itself. **Measured when produced, so a drop, a fill or a refusal cannot move the measured set** - which is what `N5` broke and L1 fixes. The one prover-selected constant in any shape is clause (d)'s `k - 1` against `0`, and both are under `2^64` |
| 10 | `[IND-3]` degree `4` | the same steps and shapes | (a), L1 | a property of the same fixed monomials. `N4`'s degree-8 pair is what this row catches, and it catches it at the step that produces the degree rather than at the end of the pass |
| 11 | `[IND-3]` 256 monomials | the same steps and shapes | (a), L1 | a count of the same fixed monomials |
| 12 | `[IND-4]`'s substitution work, and one step's intermediate | substitution | (a), L3 | **no limit of its own and no error of its own**: rows 9 to 11 fire on each step's *result*, and the intermediate one step materializes before that test is derived in `[IND-7]`'s *pass* paragraph from the clause list - at most 4096 monomials, degree at most 8, coefficients under `2^398`. The step count is a count of the path's text. Exact evaluation inside that figure is a conformance requirement, not an implementation's choice. **This is `N6`'s row, and until this round the pass was the one computation in the pipeline with no row and no bound** |
| 13 | `[IND-4]` clause (e) refusal at a `set` destination | substitution | (a), L1 | tests the right-hand side's grammar and the destination's binder kind, at a commit the shape rule has already fixed as visited. It is the **only** refusal that ends a substitution, and both its inputs are text |
| 14 | `[IND-4]` clause (b) refusal at a `set` destination | substitution | **one-way**, L2 | content-reachable but not a limit and not inside the check: a strengthening moves it only from refusing to **admitting**, the direction `[ENT-1]` permits (3.8.4). **The only row in this table a prover decision moves at all**, and by L1 both directions measure the same shapes and visit the same commits - the clearance the fourth round left stale, restated on the property that now carries it |
| 15 | `[IND-6]` clause (i) dropping a path condition whose substitution refused | head-frame hypotheses | (b), L1 | **dropped**: the slot is present and empty, the visit set does not move, and **the condition's shapes were measured by rows 9 to 11 when the pass produced them**. Dropping is a content decision only. `N5` is the program that fails if this row is read any other way |
| 16 | `[IND-7]` 4 elimination terms | the check's entry | (a), L1 | the degree-1 monomials of a syntactic `p`; clauses (b), (d) and (e) each contribute exactly one witness term whatever derives |
| 17 | `[IND-7]` 32 slots | the check's entry | (a), L1 | one per supplied statement hypothesis, one per governing branch condition - by L4 one per `[IND-3]` polynomial it yields, or one always-empty slot where it yields none - two or four per visited commit by governing clause, one per ordered pair of elimination terms; every count an output of the shape rule |
| 18 | `[IND-3]`'s three limits **at a certificate step** | `p := \|b\|*p - \|a\|*h` | (b), L3 | **removed.** Which steps execute is decided by which slots the prover filled, so an error *or* a discard here moves with the prover. The step carries no limit; its size is derived instead - four steps, degree 4, `5*256` monomials, `2^640`, `s <= 2^508` - **and that derivation is only as good as rows 9 to 11's scope**, which is why they cover every shape the pass produces. This is `N3`'s row, and the figure is `N4`'s |
| 19 | a slot content that is not `[IND-3]`-normalizable | filling a slot | (b), L2 | **discard**: the slot is empty, which is already the (ii)/(iii) move. At the present fact vocabulary it is a **safety net rather than a live route** - every constant and difference bound is degree 1 with unit coefficients and endpoints inside `[-2^63, 2^64)`, so every offerable fact already normalizes. It is what keeps L3's premise true if the fact language ever widens |
| 20 | `RELAX` corner products and their sum | the final test | (b), L3 | **no limit**: intervals come from filled slots and from type widths, and a degree-4 `u64` monomial is near `2^256` in the ordinary case. `RELAX` is total and returns the number. **Unlimited is not unbounded**: its magnitude is under `2^1159`, derived from the caps, the type widths and `[IND-3]`'s limits, which is what row 21's capability half needs |
| 21 | a certificate that fails **on its residual**, or one an implementation cannot evaluate | the check | (b), L3 | **discard**, confined to that certificate; the existential moves to the next. The capability half is **dead letter** by rows 12, 18 and 20 - the pass, every certificate and `RELAX` are each evaluable inside a derived figure, and an implementation that declines work inside those figures is not conforming. That is load-bearing twice over: with the certificate figure false it let two conforming implementations decide `N4` differently, and with the pass unbounded it said nothing at all about `N6` |
| 22 | `[IND-8]` published constant over magnitude | projection | (b), L2 | **not published**, unchanged - the rule the rest of this table follows |
| 23 | `[IND-8]` corner-minimum arithmetic | projection | (b), L3 | **no limit**, for `RELAX`'s reason - and now with `RELAX`'s kind of figure rather than by analogy to it: the minimum is computed from the statement polynomial **as written**, whose coefficients row 7 limits to `M`, so a monomial's corner minimum is at least `-2^635` and the sum of at most 256 of them is above `-2^643` (`[IND-7]`, *the bound*) |
| 24 | `[IND-1]` undischarged base or step **in the complete state** | the verdict | -, L2 | the predicate's own value, not a limit; *Monotonicity* is what fixes its direction. **The view qualifier is part of the row**: verification is per view (`[IND-8.V]`, the `[FN-9]` parallel), so a statement undischarged only in the S4-blinded view `B` is not an error and merely does not publish there |

`[IND-5]`, `[IND-8.T]`, `[IND-8.V]` and `[IND-9]` raise nothing at all - `[IND-9]`
says so in as many words, and the other three add hypotheses, a typing amendment
and a view partition. **Rows 9 to 11 against row 18 are the whole content of the
last three rounds**: `[IND-3]` first said its three limits "apply at every step"
and `[IND-7]` imported that into the certificate, which put a hard error on a
value the prover decided whether to compute; the third round scoped them to the
two polynomials `[IND-3]` itself normalizes, which cut too deep, because the
pairs and the path conditions are shapes the rule writes and row 18's replacement
figure quietly assumed they were still limited; the fourth round widened the
scope and delimited it by *carried into the head frame*, which put the same hard
error behind row 14's prover-flippable refusal by way of row 15's drop. **The
line that actually separates the two sides is shape against content, and the
moment that fixes it is production**, and rows 9 to 11 are drawn on both: every
shape the pass produces is measured when it is produced, every prover-offered
fact is a discard, and every remaining (b) row already had, or now has,
`[IND-8]`'s discard treatment.

**Two structural readings of the table are worth stating, because a row-by-row
reading missed both.** First, the seam: a class-(b) decision may not gate a
class-(a) hard error. Row 14 is the only (b) decision in the pipeline and rows 9
to 11 are the (a) errors nearest it, and L1 is exactly the sentence that keeps
the two apart - `N5` is what lives in the seam when it does not. Second, the
gap: a computation with no row is worse than a row misclassified, because
nothing in the table points at it. Row 12 exists because `[IND-4]`'s pass had no
row until this round, and the check for the next run is not "is every row right"
but **"is every computation in a row"**.

**The term set and the elimination terms, worked.** The distinction the rule
draws above is load-bearing for row 13, and I3's statement is the witness. Take
`bound @step running: ile(acc, i * factor);` - I3's own statement - and put
`set factor = load(cfg, i);` in the body. `factor` occurs in `p` only inside the
degree-2 monomial `i*factor`, so it **is** an `[ENT-2]` term of `p` and **is** in
`[IND-4]`'s term set: the commit is visited, its right-hand side is a call, its
destination is a `set` destination, and clause (e) **refuses the statement**.
`i*factor` is not a degree-1 monomial, so it is **not** an elimination term,
which is why I3's own trace counts one elimination term and not two. Reading the
visit rule over the elimination terms instead would leave that commit unvisited,
leave in `p` a `factor` the body has overwritten, and verify a false head-frame
claim - two readings accepting different programs, one of them unsound, which is
A17's defect class. The term set is the set the visit rule reads.

**A refusing commit reached only through a branch condition, worked.** This is
row 15's route and the sentence clause (e), clause (b) and `[IND-6]` clause (i)
now share. Take `bound @l counted: ile(hits, i);` - I4's statement, which never
names `t` - over the body

```whitefoot
set t = load(buf, i);
let c = ilt(t, 5_u64);
if c { set hits = hits +wrap 1_u64; }
```

On the true path the step polynomial is `p = hits - i - 1`. The `set hits` commit
is clause (b): witness `o`, `p0 = o - i - 1`, two constant-bound slots and the
pair `E1 = o - hits - 1 <= 0` / `E2 = hits + 1 - o <= 0`. Clause (i) then takes
the governing condition `c`, whose defining comparison is what `[IND-3]`
normalizes - exactly as `N3`'s inline `ilt(g1, g2)` is - giving `t - 4 <= 0`,
so `t`
enters the term set and the pass reaches `set t = load(buf, i)`: a call into a
`set` destination, **clause (e) refuses**. Because that refusal is met while
substituting a *path condition*, the condition is **dropped** - its slot present
and empty, the commit visited, its slots present and empty - and the check
proceeds on the statement's own chain: `sigma(o) = E1` gives `p := hits - i`,
`sigma(i) = H1` with `H1 = hits - i` gives `p := 0`, and `RELAX(0) = 0`.
**Verified**, at every conforming checker. Had the statement itself named `t`,
the identical refusal would be met while substituting the statement's own
polynomial and would end the check at `[IND-1]`. The three sentences say one
thing: **where the refusal is met decides what it ends**, and the clause that
raised it decides nothing.

**And L1 adds the fourth sentence this example needs, which `N5` is the program
for.** The dropped condition's shape `t - 4 <= 0` was **measured** against rows 9
to 11 when the pass produced it - degree 1, two monomials, coefficient 1, so it
passes - and it would have been measured identically had the refusal not
happened. Here the refusal is clause (e)'s, which is syntactic, so nothing turned
on it either way; in `N5` the refusal is clause (b)'s, which a strengthening
flips, and only production-time measurement keeps the two checkers on the same
verdict.

**The corner-product sentence is A24's D-2** - the drafted `RELAX` was undefined
for a mixed-sign degree-2 monomial and two implementations would have differed on
the first signed accumulator. Its type-interval fallback is F-I1's F4/F9: `RELAX`
as drafted read "the tightest constant bounds derivable at the check point", and
a clause (d) witness `q` has **no** derivable constant bound at all, so the
drafted rule was undefined on a term the exhaustive enumeration is guaranteed to
reach (the empty certificate on I1's own obligation is one). Two readings - treat
it as unbounded, or raise `[IND-3]`'s hard error - accept different sets, and only
the first leaves the accepted set alone. The type interval is that first reading,
written down.

**The caps.** As drafted they were `16` hypotheses and `4` elimination terms,
both hard errors on counts a prover strengthening can grow, which is exactly the
`[ENT-1]` break A4 was supposed to have closed. Repaired, both are counts of
**slots at a visit set the shape rule fixes**: twelve ordered-pair slots at four
terms whether or not any bound is derivable, two constant-bound slots per visited
witness commit and two pair slots per visited clause (b) or clause (d) commit
whether or not any of them is derivable, and one witness term per non-exact
commit whether or not the no-wrap side condition derives. The slot cap had to
rise from 16 to **32** as a consequence, because twelve pair slots plus one
statement hypothesis plus three clause (b) commits at four slots each is already
twenty-five, and the drafted sixteen could not hold even one such commit beside
the pair slots; the corrected space is
`sum_k C(4,k)*P(32,k) = 988,161` partial injections at the caps. **The drafted
figure was wrong twice over**: `16*15*14*13` = 43,680 counts only the full-domain
injections, and the space at sixteen slots is `sum_k C(4,k)*P(16,k)` = **58,625**
partial injections. What keeps the work small is not the cap but the skip rule -
an assignment whose step is skipped is equivalent to the certificate that omits
the term - so an implementation evaluates only the assignments where a slot is
filled and carries a correctly-signed coefficient on its term. The largest
certificate space in this file is I1's midpoint - three elimination terms and
ten slots, so 1,021 partial injections, of which only six `(term, slot)` assignments are
admissible at all - two each for `q`, `hi` and `lo` - because no other filled slot
carries a correctly-signed coefficient on its term. I2's
step is nine slots and 748; **I4's matching step path is eleven slots over two
terms and 133** - the largest slot count in the file, and eleven rather than ten
because an `ieq` path condition now takes one slot per `[IND-3]` polynomial.

**The six programs the four laws are answerable to.** Each was built to break the
argument and each is worked here under the final text, because a monotonicity
argument with no adversary in front of it is the thing that has already been
wrong four times. `N1` and `N2` attack the visit set and parts (i) and (ii) -
**L1**; `N3` attacks the arithmetic and part (v). `N4` is a program of a
different kind - it leaves all five parts standing and attacks the **resource
figure** the arithmetic's removal is paid for with, and through it the
determinism half of the same law - **L3**. `N5` is the fifth run's, and it is the
one L1 exists for: it attacks the **seam** between the two classes, where the
single prover-flippable decision (row 14) gated a class-(a) hard error (rows 9 to
11) through row 15's drop. `N6` is the fifth run's second, and it attacks **L3**
where no row existed at all - the backward pass's own intermediates.

`N1` **- the visit set drives clause (e)'s refusal.**
`bound @l s: ile(x, 255_u16);` over a body `set acc = load(buf, i); let y = acc
+wrap 7_u8; let z = cvt<u8, u16>(y); set x = z;`, with `x` a `u16`, so the
statement is a real fact and not a type bound. The pass, end to entry:
`set x = z` and `let z = cvt(y)` are clause
(c), leaving `p = y - 255`; `let y = acc +wrap 7_u8` is clause (b) on a `let`
binder, so no refusal, witness `o`, `p = o - 255`, two constant-bound slots on
`y` and - **in shape, whatever the prover derives** - the pair `o - acc - 7 <= 0`
and `acc + 7 - o <= 0`, which puts `acc` in the term set. `set acc = load(buf,
i)` is therefore visited, its right-hand side is a call, its destination is a
`set` destination, and clause (e) **refuses the statement**, naming that commit.
That is a hard error at `[IND-1]` on **every** conforming checker, weak or
strong: whether the pair's two slots are filled decides nothing, because the
statement never reaches `[IND-7]`. Under the pre-repair text the weak checker
left the pair out, never reached `set acc`, verified the statement with five
slots, and a checker that could derive `acc - Z <= 100` reached it and hard-
errored - a v0.40 program refused at v0.41, which is precisely the sentence
`[ENT-1]` forbids. The repair costs `N1` its acceptance at every version, and
3.8.4 prices that.

`N2` **- the visit set drives the slot count.** Nine chained wrap commits on
`let` binders, `let a1 = seed +wrap 7_u8; ... let a9 = a8 +wrap 7_u8;` then
`let z = cvt<u8, u16>(a9); set x = z;` under the same statement, with `seed` a
loop-invariant parameter so the chain ends there. The two clause (c) commits give
`p = a9 - 255`; each `let a_j` is a visited clause (b) commit contributing four
slots - two constant bounds and the pair, whose `P = a_{j-1} + 7` puts the next
operand down into the term set unconditionally - so all nine are visited at every
version. Slot count `1 + 9*4 = 37`: one group-1 statement hypothesis, thirty-six
from the chain, and no group-3 pair, since `p`'s only degree-1 monomial is `o9`.
**37 > 32, so it is the same hard error naming the statement at every conforming
checker.** Under the pre-repair text the weak checker stopped at `a9` for five
slots and verified, and the strong one walked all nine for thirty-seven and hard-
errored. `N2` is now refused uniformly, which is what a cap is for; what it costs
is real and is stated in Q3 - a clause (b) chain costs four slots a link, so the
cap binds at eight links (`1 + 4*8 = 33`), and seven fit.

`N3` **- the certificate's arithmetic drives a per-step hard error.** With
`K = 15000000000000000000_u64`, a legal `u64` literal:

```whitefoot
let w1 = imin(k1, 1_u64);  let w2 = imin(k2, 1_u64);  let acc = 0_u64;
for @l i in 0_u64..n {
  bound @l s: ile(acc, i + w1 + w2);
  let g1 = w1 *wrap 15000000000000000000_u64;   // C1  clause (b), let binder
  let g2 = w2 *wrap 15000000000000000000_u64;   // C2  clause (b), let binder
  if ilt(g1, g2) { }                            // puts g1, g2 in the term set
  set acc = acc + 1;                            // C3  clause (a)
}
```

The strengthening is 2.4's own example: v0.40 lacks `imin`'s row image, v0.41 has
it, so v0.41 derives `w1 - Z <= 1` and `w2 - Z <= 1` in the body where v0.40 has
only `[ENT-2]` 2905's implicit `u64` bound. **The shape rule holds perfectly.**
The base is `p0 = acc - i - w1 - w2` with an empty group 1 and `RELAX = 0`. The
step visits C3; the branch condition puts `g1` and `g2` in the term set; C2 and
C1 are visited clause (b) commits whose pairs put `w2` and `w1` back. Four
elimination terms, at the cap, at both versions. Twenty-two slots at both - one
statement hypothesis, one path condition, four at each of C1 and C2, twelve
ordered pairs - so the space is *literally the same set*, and `sigma(acc) = H1`
gives `p := 0` and verifies at both. Parts (i) to (iv) are all satisfied.

What moves is the **work**. At v0.40 the `*wrap` side condition `c1 * c2 <=
max(u64)` needs a bound on `w1` better than `max(u64)`, so all four pair slots
are present and **empty**, and no step anywhere in the space grows a coefficient
past 1. At v0.41 `1 * K <= max(u64)` and the four pairs **fill**, so the space
now contains `sigma = {w1 |-> o1 - K*w1 <= 0, w2 |-> o2 - K*w2 <= 0}`, whose two
admitted steps run `p := K*p0 - 1*Q1a` and then `p := K*p - K*Q2a`, reaching
`s = K^2 = 225000000000000000000000000000000000000` against
`2^127 = 170141183460469231731687303715884105728`. Under the pre-repair text that
is `[IND-3]`'s magnitude limit crossed at a step - *a hard error naming the
statement* - **on a program the weaker checker compiled**. Read the other way,
with an implementation stopping at the first success, whether the error fires at
all depends on the order the space is enumerated, which breaks the determinism
half of `[ENT-1]` instead of the monotonicity half. Same text, same visit set,
same twenty-two slots, and two verdicts.

Under the text above neither reading survives, and part (v) is why. The step
carries no limit (sweep row 18), so `K^2` is an ordinary intermediate, that
certificate is evaluated to a residual like any other, and if it does not succeed
that is a discard confined to it. The certificate that decides the statement is
`sigma'`, which drops from `sigma`'s domain exactly the terms whose slots went
empty-to-filled - here both of them, leaving `sigma(acc) = H1` - performs the
arithmetic it performed at v0.40 step for step, and reaches `p := 0`. **`N3`
verifies at both versions**, no hard error is reachable from inside the check,
and no enumeration order changes the answer.

`N4` **- the fourth program, and it attacks the bound rather than the space.**
`N3`'s repair removed `[IND-3]`'s limits from the certificate step and paid for
that with a syntactic bound. `N4` is the program that showed the bound had no
premise, and it is an ordinary one - I4's own shape with the increment computed
instead of written as a literal:

```whitefoot
fn scan(x: own u64, n: own u64) contract { requires ile(x, 1_u64); } {
  let hits = 0_u64;
  for @scan i in 0_u64..n {
    bound @scan counted: ile(hits, i);
    let m1 = x * x;  let m2 = m1 * m1;  let m3 = m2 * m2;   // clause (a) x3
    set hits = hits +wrap m3;                               // clause (b)
  }
}
```

Every commit is legal and every domain obligation discharges, since `x <= 1`
gives `m1, m2, m3 <= 1`. The backward pass gives `set hits` a witness `o` and the
pair `E1 = o - hits - m3 <= 0` / `E2 = hits + m3 - o <= 0`, filled exactly as
I4's is, through the published `hits - i <= 0`; the three clause (a) commits then
rewrite the hypotheses - `m3 -> m2*m2 -> m1^4 -> x^8` - leaving
`E1 = o - hits - x^8 <= 0`, **degree 8**. Meanwhile `p0 = o - i - 1` is degree 1
with two elimination terms and seven slots, so **no cap and no other row of the
sweep fires**, and `sigma(o) = E1` is the only route to the witness: it runs
`p := p0 - E1 = hits - i - 1 + x^8`, then `sigma(i) = H1` gives `x^8 - 1`, and
`RELAX` closes it at `0`.

**Under the scope as the third round wrote it, that step is legal and the bound
is false.** "Degree never rises, because both are at most 4" needed `E1` to be
limited, and `E1` is a *hypothesis shape*, which the narrow scope did not cover.
The same construction breaks the other two figures: a four-term sum squared four
times puts `C(19,3) = 969 > 256` monomials in a filled pair, and seven
`let t = s * 1000000_u64` commits put a coefficient past `2^127`. Only "at most
four steps" survived. **And the determinism sentence went with it**, through the
*discard* paragraph's capability half: implementation **A**, evaluating exactly
over the mathematical integers as this rule says the arithmetic is, verifies
`scan`; implementation **B**, sized to the bound this file published, cannot
evaluate the degree-8 step, discards that certificate, finds no other, and raises
a **hard error at `[IND-1]`**. Both conformed, both derived the same facts, and
they decided different predicates - which is the defect `AGENTS.md` names in as
many words, *compiler capability is not a source-language rejection*. Not a
monotonicity break: the `sigma'` lemma covers the reverse direction, since the
omitting certificate runs only the weaker version's arithmetic and is by
construction as evaluable as it was.

**Under the widened scope `N4` is refused, and that is the price, stated.**
`E1`'s degree 8 crosses row 10 at the moment the shape rule produces it, so the
diagnostic names the statement and the limit, and it does so **identically at
every conforming checker** - the pair is in shape whatever the prover derives, so
this is a class-(a) hard error like `N2`'s cap, not a fact-dependent one. The
writer's route is the ordinary one: bind the increment above the loop, where
`m3` is loop-invariant, the pass never reaches its commit, and `E1` stays
`o - hits - m3 <= 0` at degree 1. What the language loses is a statement over an
accumulator whose per-iteration increment is a degree-8 expression **in the
statement's own terms**; nothing in this file needs one, and the alternative -
keeping the wide hypotheses and publishing a bound that is a function of the
path's text rather than a spec-fixed constant - buys that program at the cost of
every resource figure the check publishes.

`N5` **- the fifth program, and it lives in the seam between the two classes.**
`N4`'s repair widened `[IND-3]` but delimited the widened scope by *carried into
the head frame*. A **dropped** path condition is not carried into the head frame,
and whether it is dropped is decided by clause (b)'s `set` refusal - the one
decision a strengthening can flip. `N5` is the ordinary program that puts a
degree-8 shape on exactly that gate:

```whitefoot
fn tally(x: own u64, n: own u64) contract { requires ile(x, 1_u64); } {
  let hits = 0_u64;
  let t = 9_u64;
  for @l i in 0_u64..n {
    bound @l counted: ile(hits, i);
    let w = imin(t, 1_u64);                 // C0 clause (e), let binder
    set t = t -wrap w;                      // C1 clause (b), SET destination
    let m1 = x * x;  let m2 = m1 * m1;  let m3 = m2 * m2;   // clause (a) x3
    let c = ilt(m3, t);
    if c { set hits = hits +wrap 1_u64; }   // C6 clause (b)
  }
}
```

Legal, and every domain obligation discharges from `x <= 1`. On the true path the
statement's own chain is I4's: C6 gives witness `o`, `p0 = o - i - 1`, and the
pair `E1 = o - hits - 1 <= 0` fills through the published `hits - i <= 0`. Clause
(i) takes `ilt(m3, t)` to `m3 - t + 1 <= 0`; the three clause (a) commits rewrite
it to `x^8 - t + 1 <= 0`; the pass then reaches C1, whose `-wrap` side condition
is `w - t <= 0`. Two elimination terms, fourteen slots, no cap fires.

**Under the fourth round's text the two checkers disagreed.** A checker with no
`imin` row image cannot derive `w - t <= 0`, so clause (b) refuses at a `set`
destination, the condition is **dropped**, nothing is carried into the head
frame, no limit is consulted, and the statement **verifies** by
`sigma(o) = E1`, `sigma(i) = H1`, `RELAX(0) = 0`. A checker that derives it keeps
the condition, carries `x^8 - o_t + 1 <= 0` into the head frame, and **hard-errors
at `[IND-3]`**. Verified to hard error under a strengthening - `[ENT-1]`
monotonicity, broken by the previous round's own repair, on the single decision
the file had already isolated as prover-flippable and cleared under the *old*
scope.

**Under L1 there is no disagreement, and the price is the same shape as `N4`'s.**
The measurement happens when the pass **produces** `x^8 - t + 1 <= 0` - three
clause (a) steps before C1 is reached and before any side condition is
consulted - so both checkers raise the identical hard error at row 10, naming the
statement, the branch condition and the degree limit. Neither the refusal, nor
the drop, nor the fill can reach it. What the language loses is what `N4` lost,
one construct over: a **branch condition** whose substituted form is a degree-8
expression in the loop's own terms, on a loop that carries a `bound_stmt`. The
route is the same - hoist `m3` above the loop, where the pass never reaches its
commit and the condition stays `m3 - t + 1 <= 0` at degree 1 - and 3.8.4 already
prices the reach the shape rule buys. **`N5` is priced away, not argued away**,
and that is the honest form: the alternative is a measured set that moves with
the prover, which is the defect itself.

`N6` **- the sixth program, and it attacks the one computation that had no row.**
`[IND-3]`'s limits were limits on the shape rule's **output**. Nothing limited
the *intermediate* the backward pass materializes, and an implementation cannot
abort early on an over-limit intermediate, because cancellation can bring the
output back inside the limits. Clause (a) admits the `Ok`-arm binding of
`*checked` as the exact polynomial unconditionally and with no domain obligation
(the F3 deletion), and `[IND-10]` says outright that "the region bounds the
substitution, and no depth limit is needed". So, in one straight-line region:

```
v0 = Ok-arm of a +checked b                      // a + b
u1 = Ok-arm of v0 *checked v0 ; ... ; u40        // (a+b)^(2^40)
w1 = Ok-arm of v0 *checked v0 ; ... ; w40        // a parallel chain
z  = Ok-arm of u40 -checked w40
bound region_zero: ile(z, 0_u64);
```

`p = z` substitutes to `u40 - w40` and then, through both chains, to
`(a+b)^(2^40) - (a+b)^(2^40) = 0`. The **output** is `0 <= 0`: degree 0, one
monomial, verified by the empty certificate, inside every limit. The
**intermediate** is `(a+b)^(2^40)` - on the order of `10^12` monomials with
coefficients around `10^300000`. One implementation cannot finish it; one that
stops early rejects a program the first accepts. That is `N4`'s species with the
prover taken out of it, and the fourth round's dead-letter argument did not reach
it, because it was scoped to *certificates* and the substitution runs before any
certificate exists.

**Under L3's per-step form `N6` is refused within four substitution steps, at every conforming
checker.** The pass replaces `z` by `u40 - w40` (degree 1), then `u40` by
`u39*u39` (degree 2), then `u39` by `u38*u38` (degree 4), and the next step
produces degree 8 and raises row 10's hard error, naming the statement and the
commit. Nothing on the order of `10^12` is ever materialized: by the *pass*
paragraph above, no step begins from an over-limit polynomial and one step
expands by at most a factor of 16. **The price is stated and it is real**: a
program whose substituted obligation would have *cancelled* back inside the
limits is refused for the intermediate it passes through, and no writer will see
why from the final polynomial. The diagnostic therefore names the commit and the
step, not just the statement, and the route is the ordinary one - a chain of
exact `*` commits deeper than degree 4 has no statement route and goes to the
callee's `ensures` or to a hoisted binder. The alternative is a rule that admits
`N6` and publishes no figure for it, which is where `AGENTS.md`'s *compiler
capability is not a source-language rejection* stops being satisfiable.

**The certificate form accepts a superset of the drafted greedy rule**, because
every greedy elimination sequence is a certificate: "unused" makes greedy's
selection injective, the traversal is the same term order, and a term greedy
could not match is a term outside `sigma`'s domain. F-I1 confirms the inclusion
and could construct no counterexample. **That superset is over a fixed hypothesis
list, and it is not a claim that the repaired rule accepts everything the drafted
one did.** `[IND-5]`'s and `[IND-10]`'s hypothesis sets are strictly *smaller*
than the drafted ones, so the accepted set is not a superset of the drafted
rule's: `B1`'s vacuous bases are exactly what is lost, deliberately, and 3.9.7
works the program that shows why losing them is the point. **What the argument
also does not do is cover a trace that was never drafted**, and the drafted claim
that "none of the worked traces has to be re-derived" was used to cover two that
do not exist anywhere in this file (2.4, repaired). It covers the seven
derivations of 3.9 and the two refusal traces of 3.8.3, and those are the ones
F-I1 re-executed. The loop design's own Q4 - *"the order fixes the accepted set;
a different order accepts a different, incomparable set"* - stops being an open
question, because no order fixes anything.

#### 3.9.2 `[IND-8]` - what a verified statement publishes

> **`[IND-8]` (projection).** A verified `bound_stmt` publishes its
> **projection** on each edge where its facts hold - for a labelled statement,
> both header edges of its loop and, for a `loop_stmt`, its body-entry edge; for
> a local statement, its normal continuation. The projection is computed
> separately on each such edge, in that edge's own state, from the statement
> polynomial **as written**. It is exactly the following finite set of atomic
> facts and nothing else: for every term `t` occurring in exactly one degree-1
> monomial of the normalized `p <= 0` with coefficient `a` in `{+1, -1}`, let
> `r := p - a*t`; publish `t - Z <= -m` when `a = +1` and `Z - t <= -m` when
> `a = -1`, where **`m` is the sum over the monomials of `r` of that monomial's
> minimum, a monomial's minimum being the minimum over its corner products** -
> for a degree-d monomial the minimum over the `2^d` products of endpoints, the
> monomial's integer coefficient included as a factor, the empty product being 1,
> and each factor's interval taken exactly as `[IND-7]`'s `RELAX` takes it,
> **on that edge's own state**; and when `r` is exactly one degree-1 monomial
> `b*u` with `b = -a` plus a constant `k`, publish additionally the difference
> bound `a*t + b*u <= -k` in whichever orientation `[ENT-2]` 2901 admits.
>
> The direction matters and is the reason the minimum is named rather than the
> maximum: `p = a*t + r <= 0` gives `a*t <= -r <= -m`, so `m` must be a lower
> bound of `r` and the corner minimum is the tightest one this rule can compute.
> An `m` larger than `r`'s true minimum would publish a bound tighter than the
> statement entails.
>
> Published facts carry the ordinary `[ENT-5]` support of their own terms and are
> killed ordinarily. **The polynomial itself is never a fact**: no state anywhere
> holds a relation over more than two terms, and `[ENT-2]` 2870 and 2901 are
> unchanged. A published constant whose magnitude exceeds `[IND-3]`'s limit is
> not published, and the projection is otherwise unchanged; publication never
> raises a hard error.
>
> **`[IND-8.T]` (`[ENT-2]` amendment).** A difference bound `t1 - t2 <= c` may
> relate terms of different fragment integer types. Its meaning is over the
> mathematical integers, exactly as `[ENT-6]` 3143 already fixes for
> normalization; `[ENT-2]` 2905's implicit per-term bounds are unchanged, and
> `[ENT-4]`'s three closure rules apply unchanged. No other rule of this
> specification produces such a fact, and `[OP-5]`'s equal-type requirement on an
> *evaluated* comparison is untouched.
>
> **`[IND-8.V]` (views).** A `bound_stmt` is verified separately in the complete
> state and in the S4-blinded state B, exactly as `[FN-9]` verifies a relation
> per view. Its projection is published **only in the views in which its base and
> step obligations both discharge.** A statement whose derivation uses an S4
> requirement source therefore publishes in the complete state and not in B, and
> `[PRV-3]`'s external-subject partition is unaffected by its presence.

`[IND-8.V]` is A9's repair and it is load-bearing for the *deletion*: 3.1.3 argues
that the claim was the only non-bridge route from external data to a protected
operation, and without a view discipline the induction statement is a second one -
a statement resting on a `requires` would discharge the leaf in B, `[PRV-3]` 3392
would find no demand remaining, and a caller could pass an external value
straight through. The sum-of-minima reading of `m`, and the magnitude clause, are
A24's D-3 and A25.

**The corner-minimum sentence is F-I1's F8**, and it was A24's D-2 repaired in
`[IND-7]` and left open one rule later: `[IND-7]` defined "the maximum over the
corner products" and `[IND-8]` then said "that monomial's minimum" with no rule
defining it. **Nothing this repair publishes changes.** I2's projection is
`r = -255*i` with `i` in `[0, 999]` on the true header edge, corner products `0`
and `-254745`, minimum `-254745`, so `sum - Z <= 254745` stands, and on the false
edge `cu(i) = 1000` gives 255000 exactly as before; I3's is `r = -i*factor`,
corner minimum `-cu(i)*cu(factor)`, so `acc - Z <= cu(i)*cu(factor)` stands; I4's
`r = -i` is a single degree-1 monomial of opposite sign, so the difference-bound
clause fires and `hits - i <= 0` is published unchanged. The three consumers walk
identically: `s22`/`L08` still need `254745 + 255 = 255000 <= max(u32)` through
`[ENT-6.D]`, `s23` still needs the added `requires ile(n, 1000_u64)` to bring
`cu(i)*cu(factor)` down to 999000, and I4's exit closure still reaches
`hits - n <= 0`.

#### 3.9.3 `[IND-9]` redundancy, and the I2 and I3 traces

> **`[IND-9]` (redundancy).** A `bound_stmt` is **never** an error for being
> derivable. A statement whose projection the state already derives, at the head
> or anywhere else, is verified by `[IND-5]` and `[IND-6]` exactly as any other,
> is kept, and publishes exactly as any other. No rule anywhere compares a
> statement against what the checker could have proved without it.

That is the owner's ruling of 2026-08-29, and section 2.10 records the asymmetry
that derives it rather than merely quoting it.

**I2 - an accumulator bounded by its increments.** `s22_accum_const.wf`
(compiled) rejects today with `[OP-2] residual: sum +defined wide`.

```whitefoot
fn weigh['w](weights: &'w buffer<u8>, count: own u64) -> total: own u32
  reads(weights) contract {
  define room = len(deref(weights));
  requires ile(count, room);
  requires ile(count, 1000_u64);
} {
  let sum = 0_u32;
  for @weigh i in 0_u64..count {
    bound @weigh per_byte: ile(sum, 255_u32 * i);
    let w = deref(weights)[i];
    let wide = cvt<u8, u32>(w);
    set sum = sum + wide;
  }
  return sum;
}
```

*Base* `[IND-5]`: post-capture `sum = 0`, `binder = 0`; polynomial
`sum - 255*i <= 0`; no `[IND-4]` substitution; **the supplied statement
hypotheses are empty**, because `per_byte` is the loop's only `bound_stmt` and
`[IND-5]` excludes the statement under check. The empty certificate relaxes with
the monomial `sum` over `[0,0]` and the monomial `-255*i` over `[0,0]`, so
`C = 0`, `s = 1`, verified - **without touching group 1 at all**, which is the
point of the B1 repair: the base of this statement is exactly as strong as it was
and it is no longer able to prove itself.

*Step* `[IND-6]`: one body path. `[IND-4]` replaces `sum` by `sum + wide` under
clause (a), which is unconditional; `wide` by its `cvt` operand `w` under clause
(c); and `w`'s commit is a subscript, so clause (e) gives a witness `o` with the
constant-bound slots on `w` read at the commit, filling `o - Z <= 255` (`w`'s
type is u8) and `Z - o <= 0`. The binder becomes `i + 1`:

```
p0 = (sum + o) - 255*(i + 1)   =  sum + o - 255*i - 255
elimination terms: o (witness, first), i, sum          -- three, <= 4
H slots: (1) H1 = sum - 255*i        the statement AS WRITTEN, [IND-6]
         (2) o - 255 <= 0, -o <= 0   the clause (e) constant bounds on w
         (3) six ordered pairs over {o, i, sum}, filled or empty
         nine slots, <= 32
certificate: sigma(i) = H1
  t = o   : outside sigma's domain -> skipped
  a = -255 on i, b = -255 on i, a*b > 0 -> p := 255*p0 - 255*H1 = 255*o - 65025 ; s := 255
  t = sum : coefficient is now 0 -> skipped
RELAX: cu(o) = 255 from its filled slot  ->  C = 255*255 - 65025 = 0
floor(0 / 255) = 0 <= 0   ->  verified
```

*Publication* `[IND-8]`: on the true header edge the state has S11's
`binder < upper_capture`, `upper_capture = count` and the contract's
`count <= 1000`, so `cu(i) = 999`; `sum` is solitary with `a = +1`, `r = -255*i`,
`m = -254745`, so `sum - Z <= 254745` is published. On the false header edge
`[ENT-5.X]` gives `binder = upper_capture = count <= 1000`, so `cu(i) = 1000` and
the published constant is 255000 - a different constant on a different edge,
computed by the same rule.

*Consumer.* The obligation the program actually has is `sum +defined wide` at
u32, and it discharges from `sum <= 254745` and `wide <= 255` because
`254745 + 255 = 255000 <= max(u32)`. **That route is `[ENT-6.D]` and it does not
exist today**: `L08_i2_consumer.wf` (compiled) supplies exactly those two bounds
as a contract and still **rejects**. `[ENT-6.D]` (3.5.5) is a hard prerequisite
of this example, not an optimization, and `L09` says the same for I3.

*What the clause (a) repair removes from this trace.* As drafted, substituting
`sum + wide` was conditional on the state at that commit discharging the `+`'s
`[ENT-6]` obligation, and the only source of that fact is the statement's own
`[IND-8]` projection at the head - so the substitution read a state that assumed
the statement under check (F-I1's F5). With clause (a) unconditional the I2 and
I3 steps no longer read the projection at all: the `+`'s obligation is the
program's own and 3.14 step 5 discharges it or rejects the program. The
assume-then-verify ordering `[IND-6]` now states is still needed, but its
customer is narrower - it is clause (b)'s no-wrap side condition, which is
exactly where I4 uses it.

**I3 - an accumulator bounded by a parameter product.** `s23_accum_param.wf`
(compiled) rejects today with `[OP-2] residual: acc +defined factor`. The
statement is `bound @step running: ile(acc, i * factor);`; its base is the empty
certificate and its step is one cancellation:

```
BASE [IND-5], post-capture acc = 0, binder = 0, no supplied statement hypothesis
p0 = acc - i*factor
elimination terms: acc only  --  i*factor is degree 2, not a degree-1 monomial
empty certificate: monomial acc over [0,0] -> 0
                   monomial -i*factor, corner products over i in [0,0] -> 0
                   C = 0 ; floor(0/1) = 0 <= 0   ->  verified

STEP [IND-6], one body path, clause (a) unconditional on `set acc = acc + factor`
p0 = (acc + factor) - (i + 1)*factor  =  acc - i*factor
H1 =  acc - i*factor
certificate: sigma(acc) = H1;  a = +1, b = +1  ->  p := p0 - H1 = 0 ; s := 1
RELAX(0) = 0 ;  floor(0/1) = 0 <= 0   ->  verified
```

Note the monomial `i*factor` - a product of two non-constant terms, the thing
`[ENT-2]` deliberately excludes from the fact language. It appears here only as a
**column that cancels against itself**, and `[IND-8]` never publishes it: the
projection publishes `acc - Z <= cu(i)*cu(factor)`, one constant.

*The honest limit this example exposes.* `s23` as written has
`requires ile(n, room)` with `room = len(deref(out))` unbounded, so
`cu(i)*cu(factor)` is astronomical and the consumer still fails. **The statement
gives the running bound; a constant ceiling still has to come from somewhere**,
and here it must be a `requires ile(n, 1000_u64)` the writer adds. With it,
`acc <= 999000` and `acc + factor <= 1000000 <= max(u32)` discharges under
`[ENT-6.D]`. That extra `requires` is a real migration cost and it is also a real
improvement to the contract: the function genuinely does overflow without it.

**The separation that makes this construct small**, stated once because it is the
design's central structural idea: **the statement is verified in a polynomial
language and published in the difference-bound language.** The richer arithmetic
lives inside one statement's own check and never enters the ambient fact state;
`[ENT-2]` 2870 and 2901 are untouched; the head learns one difference bound. That
is the shape the owner's I1 ruling names - *verify the given statement by
substitute-and-compare over that statement's own terms, which never requires
holding a three-term relation in the ambient fact state* - generalized from I1 to
the whole family.

#### 3.9.4 `[IND-10]` - the local statement, and I1

The owner's I1 ruling shelves the two-law dilemma: buy no three-term ambient
domain, restore no shape source, and examine instead the route where the writer
**states** the fact and the checker verifies the given statement by
substitute-and-compare over that statement's own terms. This is that route,
drafted, with its normalizer power stated exactly and its residue priced.

> **`[IND-10]` (the local statement).** An unlabelled `bound_stmt` is admitted as
> an ordinary statement, subject to one restriction: **every term of its
> statement polynomial must be committed within the same straight-line region as
> the statement itself, by a commit `[IND-4]` admits, or be a place live and
> uncommitted throughout that region.** A straight-line region is the maximal run
> of statements ending at the `bound_stmt` and containing no branch, loop,
> `match`, call boundary or region boundary. Any other statement is a hard error
> citing `[IND-10]`.
>
> Its obligation is the statement polynomial with `[IND-4]`'s backward
> substitution taken over that region alone - so the region bounds the
> substitution, and no depth limit is needed - checked once by `[IND-7]` in the
> closed state at the region's entry, extended by `[IND-4]`'s witness
> hypotheses. There is no base and no step. Its projection `[IND-8]` is published
> on its normal continuation, and `[IND-8.V]` applies unchanged.
>
> **The statement hypotheses `[IND-10]` supplies to `[IND-7]` are none.** A local
> statement has no loop, so "that loop's `bound_stmt`s" names nothing; and with
> no base and no step, a rule that let the statement's own polynomial into its own
> hypothesis list would make every local statement true by construction. An
> enclosing loop's labelled statements are not supplied either: they hold at that
> loop's head, and the region entry is inside the body, where the body's commits
> may already have moved their terms. What such a statement contributes reaches
> this check the ordinary way, as the `[IND-8]` projection standing in the
> region-entry state.
>
> Two local statements in one region do not compose: the second is checked at the
> region's entry, which precedes the first, so the first's projection is not in
> that state. Widening this is monotone and can wait for a program that wants it.

**The statement, and the program it stands in:**

```whitefoot
let span = hi -checked lo;      // Ok arm
let half = span / 2_u64;
let mid  = lo +checked half;    // Ok arm
bound probe_inside: ilt(mid, hi);
let probe = deref(table)[mid];
```

**The trace.** One backward pass over the region, end to entry. At
`let mid = lo +checked half` clause (a) applies **unconditionally** - the `Ok`-arm
binding has no `[ENT-6]` obligation to discharge, which is the F3 repair - giving
`p = lo + half - hi + 1`. At `let half = span / 2_u64` clause (d) applies, and
`span` is `u64`, so `Z - span <= 0` holds from `[ENT-2]` 2905's implicit per-term
bound and the **d1** pair is the one drafted. At `let span = hi -checked lo`,
clause (a) again, and `span` no longer occurs in the polynomial but **does occur
in the two witness hypotheses**, which the backward pass now rewrites - the F2
repair, without which `span` survives into `H`, is not an elimination term, cannot
be eliminated, and relaxes at a region entry where it is not even in scope:

```
p0 = lo + q - hi + 1 <= 0
elimination terms: q (a witness term, first), then hi, lo   -- three, <= 4
H slots: (2) the clause (d) constant bounds on `half`, filled from u64's own
             implicit bounds: -q <= 0 and q - max(u64) <= 0, neither used
             the d1 witness pair, rewritten through `span <- hi - lo`:
               H1 = 2q - hi + lo <= 0 ,  H2 = hi - lo - 2q - 1 <= 0
         (3) six ordered pairs over {q, hi, lo}: (lo,hi) filled with
               H3 = lo - hi + 1 <= 0   (the loop's own settled test), rest empty
         ten slots, <= 32 ; group 1 is empty by [IND-10]
         (the shape rule moves nothing here: no clause (b) commit, no branch,
          and clause (d)'s pair was already unconditional in shape)
certificate: sigma(q) = H1, sigma(hi) = H3
  t = q  : a = +1, b = +2, a*b > 0 ->  p := 2*p0 - 1*H1 = lo - hi + 2 ; s := 2
  t = hi : a = -1, b = -1, a*b > 0 ->  p := 1*p - 1*H3 = 1 ; s := 2
  t = lo : outside sigma's domain -> skipped
RELAX(1) = 1 ;  floor(1 / 2) = 0 <= 0   ->  verified
```

Step two in full, because the coefficients are where a reader will want to check:
`2*p0 = 2lo + 2q - 2hi + 2`, minus `H1 = 2q - hi + lo` leaves `lo - hi + 2`; then
`(lo - hi + 2) - (lo - hi + 1) = 1`, and `floor(1/2) = 0` is the integer
tightening the whole derivation turns on.

**And 2.4's third repair option stays dissolved.** The objection to a
syntactically total hypothesis list was that filling the `(q, hi)` slot with the
type-implied bound would displace `lo - hi <= -1` and kill this derivation. Under
the certificate form that slot is present here and empty, and were it filled the
certificate above still exists with the filled slot outside `sigma`'s image. The
search, not the emptiness, is what saves I1.

**So the normalizer power I1 needs, exactly, is three things beyond what I2, I3
and I4 need**, and this is the answer the owner's ruling asked for:

1. **the division witness** - `[IND-4]` clause (d)'s `a / k` for a literal
   `k >= 1`, introducing one opaque term and two hypotheses, and specifically
   the **d1** pair, which needs the dividend provably nonnegative. That
   restriction is F-I1's B2 and it is a soundness repair, not a scruple: 845
   fixes exact division as truncating toward zero, so for `a = -5, k = 2` the
   language's `q` is `-2` and the drafted `k*q - a <= 0` is **false**;
   `probes/f3_sdiv_false_bound.wf` (compiled) shows today's checker refusing
   exactly the bound the drafted pair would have proved, and
   `probes/f2_sdiv_consumer.wf` (compiled) accepts the program that then divides
   by zero. A midpoint over an unsigned window is d1's case for free;
2. **elimination against ambient difference bounds**, not only against the
   statement's own polynomial - the second step consumes `lo - hi <= -1`, which
   is a bound over two terms *of the statement itself*, so no three-term ambient
   relation is ever held;
3. **integer tightening** - the final `floor(C / s)` with `s = 2`. Without it the
   derivation is off by exactly one half and the statement is refused. This is
   the whole content of *"an even window's midpoint is strictly inside it"*, and
   no relaxation recovers it.

**The cost.** All three are already inside `[IND-7]` as drafted for I2/I3/I4, so
**I1's cost is not new machinery**; it is that the fragment must carry (2) and
(3), which I2/I3/I4 alone would not have forced. In particular it costs **no new
column on the operation table**: the competing design's exact algebraic
denotation column (60-100 rows, each carrying a per-row correctness obligation
that review must discharge, roughly doubling the image column's review load) is
**not adopted**, and A22 is the adjudication.

**The residue the route cannot check, and its price.** Two families survive:

- **Interpolation search.** `mid = lo + (needle - lo_val)*(hi - lo)/(hi_val -
  lo_val)` divides by a **term**. `[IND-4]` admits `a / k` only for a literal
  `k`, because a term divisor's witness needs `k >= 1` from the state and its two
  hypotheses are then degree 2 in the unknowns, which `[IND-7]` only relaxes.
  **Refused by a stated rule with a diagnostic naming the divisor** - and it
  **dissolves through the contract system instead**: give the scaling function an
  `ensures ilt(result, span)` under `requires ilt(lo, hi)`, and the caller's
  `mid < hi` follows from `lo + scaled < lo + span = hi` by the same
  cancellation, now with the scaled term bounded by a published relation rather
  than by an unfold. **The algebraic probes go to the declared statement, the
  data-dependent probes go to an `ensures`, and nothing needs a shape rule.**
  That sentence is the one that closes the door on `[ENT-3.S8]`'s restoration for
  good, and it is the surviving contribution of the design A22 rejects.
- **Ternary search** `lo + d/3` verifies (literal divisor); **galloping search**
  needs no statement at all, because the honest program clamps the probe with
  `imin(lo +checked step, top)` and the `imin` image publishes `r <= top`
  directly.
- **A probe that is neither** - a window advanced by a value read from the
  buffer, with no helper function to carry a contract - has no route and goes to
  `if`/`else` with the price of 3.12.

**The price of routing I1 itself to `if`/`else`, measured.**
`L11_bsearch_ifelse_price.wf` (compiled) is the binary search with the midpoint
test written as an ordinary branch, and it **accepts today, with no rule at
all**. The price has three parts and only the third is real: (1) one compare and
one never-taken, perfectly-predicted branch per round - twenty extra branches on
a million-element search, on values already in registers; (2) nothing at all in
the fact state, since the branch is the *source* of the fact, so the checker gets
stronger; (3) **the writer must invent a behaviour the program does not have** -
what does a binary search do when its own midpoint is outside its own window?
`break` is the least-bad answer and it is a lie about the program. The third part
is the audit's own intent test, and it is the whole argument for `[IND-10]`.

**Flagged decision D2** (section 9) puts that argument to the owner with both
sides, because A23 removed three of `[IND-10]`'s four stated customers.

#### 3.9.5 I4 - the counter and the trip count, and what it does not close

The audit records I4 as irreducible and names it "induction of the kind P-LOOP is
defined not to do". That is right about retention and wrong about the language
this design proposes.

**The audit's own witness is a wrong program.** `s13_matchcount.wf` allocates
`data` and `out` at eight bytes each, counts the bytes equal to zero - all eight
of them - and writes `out[hits]` with `hits = 8`. It is not merely undischarged;
it is out of bounds, and no rule may accept it. The item survives its witness
being repaired: `L24_matchcount_correct.wf` (compiled) sizes `out` at nine and
still **rejects** with `[OP-4] residual: hits < len(out)`, and
`L25_matchcount_loop_form.wf` (compiled) is the same count written as an ordinary
`loop` and rejects identically.

```whitefoot
let data = buffer_new(8_u64, 0_u8);
let out  = buffer_new(9_u64, 0_u8);
let n = len(data);
let hits = 0_u64;
for @scan i in 0_u64..n {
  bound @scan counted: ile(hits, i);
  let value = data[i];
  let is_target = ieq(value, 0_u8);
  if is_target { set hits = hits +wrap 1_u64; }
}
set out[hits] = 1_u8;
```

*Base* `[IND-5]`. Post-capture `hits = 0`, `binder = 0`; no supplied statement
hypothesis, since `counted` is the loop's only statement and `[IND-5]` excludes
it. Empty certificate: monomial `hits` over `[0,0]` and monomial `-i` over
`[0,0]`, so `C = 0` and `floor(0/1) = 0 <= 0`. Verified, and - the B1 repair
again - verified from the preheader state rather than from itself.

*Step* `[IND-6]`. Two body paths. The matching path is where clause (b) does its
work, and the two repairs change the shape of the trace without changing its
outcome: a `wrap` commit contributes a **witness term** whatever the prover can
derive; the no-wrap side condition adds two equality hypotheses instead of
rewriting the polynomial; and those two hypotheses are now present **in shape**
whether or not the side condition derives, which is what fixes the visit set. I4
is the trace that shows why the second repair had to make the pair unconditional
in shape rather than delete it: `sigma(o) = E1` is the only step that eliminates
the witness, so a clause (b) that contributed no relational hypothesis would lose
this derivation outright.

```
matching path: `let value = data[i]; if ieq(value,0) { set hits = hits +wrap 1; }`
binder shift: P[i := i+1] = hits - i - 1
backward pass: `set hits = hits +wrap 1_u64`  ->  clause (b): hits <- o,
   constant-bound slots on `hits` after the commit, and the equality pair, whose
   two slots are present in shape and are here FILLED, because the head state
   derives the +wrap side condition `hits - Z <= c1`, `1 <= c2`,
   `c1 + c2 <= max(u64)` through the published `hits - i <= 0` with
   `i < n <= max(u64)`:
       E1 = o - hits - 1 <= 0 ,  E2 = hits + 1 - o <= 0
   `P = hits + 1` puts `hits` back in the term set; no earlier commit on this
   path writes `hits`, so the pass reaches none. The path condition
   `ieq(value, 0_u8)` puts `value` there, so `let value = data[i]` is visited:
   clause (e) on a `let` binder, witness `ov`, two constant bounds.
p0 = o - i - 1
elimination terms: o (witness, first), i        -- two, <= 4
H slots: (1) H1 = hits - i        the statement as written
         (2) the path condition `ieq(value, 0_u8)`, TWO slots by [IND-3]'s two
               polynomials, substituted to `ov <= 0` and `-ov <= 0`
             E1, E2 and the two constant bounds on `hits`   -- the clause (b)
               commit's four
             the two clause (e) witness bounds for `value`
         (3) two ordered pairs over {o, i}
         eleven slots, <= 32   (133 partial injections)
certificate: sigma(o) = E1, sigma(i) = H1
  t = o : a = +1, b = +1, a*b > 0 -> p := 1*p0 - 1*E1 = hits - i ; s := 1
  t = i : a = -1, b = -1, a*b > 0 -> p := 1*p - 1*H1 = 0 ; s := 1
RELAX(0) = 0 ; floor(0/1) = 0 <= 0   ->  verified

non-matching path: no commit touches `hits`; the governing branch condition is
  the false edge of `ieq(value, 0_u8)`, a disequality [IND-3] gives no polynomial,
  so its slot is present and always empty - and its terms still put `value` in
  the term set, so `let value = data[i]` is visited here too
p0 = hits - i - 1 ;  sigma(hits) = H1 ;  a = +1, b = +1
  p := p0 - H1 = -1 ;  RELAX(-1) = -1 ;  floor(-1/1) = -1 <= 0  ->  verified
H slots: 1 statement + 1 empty path condition + 2 witness bounds on `value`
         + 2 ordered pairs over {hits, i}  =  six slots
```

`hits` enters the matching path's residual through `E1` and leaves it through
`H1`; it is never an elimination term, because the elimination-term list is fixed
from `p0`. Both obligations are exactly the ones `L05_i4_step.wf` and
`L06_i4_step_skip.wf` (compiled) pose as contracts, and **today's checker accepts
both**.
*Publication.* `hits` is solitary with `a = +1` and `r = -i` is a single
degree-1 monomial of opposite sign, so `hits - i <= 0` is published on both
header edges.
*Exit.* On the false header edge `[ENT-5.X]` applies, because the post-capture
state derives `lower_capture - upper_capture <= 0` (`0 <= n`), and establishes
`binder - upper_capture <= 0` beside the negated guard. `[ENT-5.P0]` then closes,
**before** the binder and captures leave scope:

```
hits - binder          <= 0     [IND-8]
binder - upper_capture <= 0     [ENT-5.X]
upper_capture - n      <= 0     S11 preheader equality
                         =>  hits - n <= 0     support {hits, n}, both still live
```

`n = len(data) = 8` and `len(out) = 9`, so `hits < len(out)` discharges at the
continuation. **I4 is closed, with one written line, no runtime check, and no new
fact-language power.** Everything it needed was named in the audit; what the
audit lacked was the statement, and so `[ENT-5.X]` had no customer and looked
withdrawable.

**What is not closed, stated precisely** (A25). The headline must read: *I4 is
closed for a counter that leaves its loop by the false header edge, or by a
`break` taken before the counter's write on that path.* A `for` loop that
`break`s **after** writing the counter reaches the continuation on an edge where
`[IND-8]` published nothing - publication is on the two header edges and, for a
`loop_stmt`, the body-entry edge, never mid-body - and where the write has killed
the head fact. The `loop`-form variant needs no `[ENT-5.X]` at all, because its
exit test is a source `if` and S1 supplies `at >= n` on the break edge, but it
does need retention to carry the published bound across the head, which is
`[ENT-5.R3]`'s `B(@l)` term. And a count whose relation to the binder the writer
cannot state in `affine` - a count that advances by a value read from the data,
with no bound of its own - is still unreachable, **correctly**: there is no true
statement to write.

#### 3.9.6 The interface obligations, checked

The core part of the batch set six obligations any loop design must satisfy or it
re-opens something the deletion closed. All six hold:

| # | obligation | where it is satisfied |
| --- | --- | --- |
| L1 | the statement is verified, never trusted | `[IND-1]`: an undischarged obligation is a hard error. `[ENT-3]` 2910 becomes "no **unverified** writer-stated or inferred loop induction exists", so the trusted-assertion class stays empty and W3 stays literally true |
| L2 | the statement creates no runtime site | `[IND-1]`: no instruction, no retained check, empty `[EFF-2]` row, no `[GIVE-1]` delivery |
| L3 | the statement needs no locality gate | by omission, and it is the largest simplification the deletion buys. `[CLM-1]` 2742's apparatus existed because a claim was *trusted*; a verified statement cannot smuggle anything, so `[ENT-6]`'s claim-authority block stays deleted |
| L4 | a redundant statement is never an error | `[IND-9]`, and section 2.10's asymmetry |
| L5 | retention keeps the easy loops annotation-free | `[ENT-5.R]`; a statement is never *required* where retention suffices, and 3.6.5's four worked loops carry no statement |
| L6 | the statement is about the loop, not about a place | `[IND-2]`'s position rule and `[IND-3]`'s grammar: it states the loop's running bound, base plus step, both ordinary `[ENT-4]` queries under `[IND-7]` |

**And the sentence the batch was missing:** a `bound_stmt`'s status against T3's
warning clause. T3 warns that "a future construct admitting claim-like predicates
that are not reviewed always-true lemmas - assertions, expected failures,
unreviewed conditions - is outside this theorem until the derivation is redone
against it". **A `bound_stmt` is verified by `[IND-5]` and `[IND-6]` before it is
a fact, is erased before lowering, and can fail only at compile time; it is
therefore not a claim-like predicate and does not reopen the derivation.**
Section 5 carries that sentence into T3 itself.

#### 3.9.7 Falsifiers for `[IND]`

**F-I1 (the fragment verifies what it claims to). RUN, AND IT FIRED.** The
experiment was to hand-execute `[IND-7]`'s certificate check against the
derivations of 3.9.3, 3.9.4 and 3.9.5, refuted if any needs a hypothesis the rule
does not name, a fifth elimination term, or a certificate outside the space. Its
verdict is **FAIL**, and the rule text above is the repair. What it found, in the
order it matters:

- **Every derivation the file drafts reproduces, digit for digit** - the seven of
  3.9 and the two refusals of 3.8.3, including `255*o - 65025`, `floor(0/255)`,
  I3's `p := 0` cancellation and I1's `floor(1/2) = 0`. The certificate form does
  the arithmetic the file says it does.
- **Two soundness breaks**, B1 and B2, worked below. Both are repaired above.
- **`[ENT-1]` monotonicity was not restored**, because the two caps were hard
  errors on prover-dependent counts (3.9.1's cap paragraph).
- **Three determinism holes** - the backward pass over witness hypotheses (F2),
  clause (a)'s vacuous proviso (F3), and `RELAX` over a term with no derivable
  bound (F4/F9) - plus `[IND-8]`'s undefined monomial minimum (F8), the check
  state's assume-then-verify ordering (F5), the group-1 reading for a local
  statement (F10), and the zeroed-coefficient skip (F11).
- **A trace-set mismatch**: 2.4 and this falsifier both named traces the file
  never drafted. 2.4 is repaired and the ipv4 row is priced in 4.4.
- **What holds:** the superset claim over the greedy rule is correct and F-I1
  could construct no counterexample; FF2 and FATAL-1 are genuinely closed by
  `[IND-6]`'s frame sentences and the certificate search reopens neither; and
  2.4's rejected "syntactically total hypothesis list" repair is dissolved, since
  with the `(q, hi)` slot present the midpoint certificate still exists.

**B1, and the repair executed.** `[IND-5]` sent the base through `[IND-7]` with
`[IND-7]`'s own first hypothesis group, "the statement polynomials of that loop's
`bound_stmt`s as written". Taking `sigma(t) = H1` with `H1` the statement itself
gives `p := |b|*p - |a|*h = 0` for the leading term of **any** statement, so every
labelled `bound_stmt` had a vacuous base. The witness:

```whitefoot
let out = buffer_new(1_u64, 0_u8);
let idx = 9_u64;
for @spin t in 0_u64..4_u64 { bound @spin lie: ile(idx, 0_u64); }
set out[idx] = 1_u8;
```

`P = idx <= 0`. The step is honestly true - no commit touches `idx` - and
`[IND-8]` publishes `idx - Z <= 0`, which discharges `idx < len(out)` at the
continuation while `idx` is 9 and `out` is one byte. Under the drafted base:
`sigma(idx) = H1 = idx`, `a = +1`, `b = +1`, `p := 1*p0 - 1*H1 = 0`, `s = 1`,
verified. **Under the repaired base it is refused**: `[IND-5]` supplies no
statement hypothesis, so group 1 is empty; `idx` is the only elimination term, so
group 3 has no ordered pair of distinct terms; every slot that could carry a
coefficient on `idx` is absent, so every certificate is the empty one;
`RELAX(idx) = cu(idx) = 9` at the preheader and `floor(9/1) = 9 > 0`. The
statement is a hard error at its own node `[IND-1]`, nothing is published, and
`j3_ind6_checkpoint_break.wf` (compiled) is the rejection that stands -
`[OP-4] residual: x < len(out)`. Its twin `j3b_ind6_consumer.wf` (compiled)
**accepts** with that fact supplied as a contract, which is what makes the pair an
arbitration rather than an assertion: the false publication buys exactly the
out-of-bounds write and nothing else.

**B2, and the repair executed.** With no restriction on the dividend's sign,
clause (d)'s pair `k*q - a <= 0`, `a - k*q <= k - 1` is **false** under 845's
truncation toward zero: `a = -5`, `k = 2` gives `q = -2` and `k*q - a = 1 > 0`.
The certificate then proved `h <= -3` where the truth is `h <= -2`, and
`floor(C/s)` - the same integer tightening I1 depends on - is what turned the
half-integer into an accepted bound:

```
bound tight: ile(h, -3_i64);   over  let h = a / 2_i64;  with a = -5
DRAFTED (d):  p0 = q + 3 ; sigma(q) = (2q - a <= 0), a_coef = +1, b = +2
              p := 2*(q+3) - 1*(2q - a) = a + 6 ; s = 2
              RELAX = cu(a) + 6 = 1 ; floor(1/2) = 0 <= 0  ->  VERIFIED, and false
REPAIRED:     `Z - a <= 0` is not derivable, so (d2) applies:
              H1 = 2q - a - 1 <= 0 ,  H2 = a - 2q - 1 <= 0
              sigma(q) = H1 : p := 2*(q+3) - 1*(2q - a - 1) = a + 7 ; s = 2
              RELAX = cu(a) + 7 = 2 ; floor(2/2) = 1 > 0     ->  REFUSED
              sigma(q) = H2 : b = -2 on q, a_coef = +1, a*b < 0 -> skipped
              sigma = {} : q has no derivable bound, so RELAX takes i64's
                           interval and the residual is positive  ->  REFUSED
```

The honest companion still verifies, which is the point of splitting the clause
rather than weakening it: `bound low: ige(h, -5_i64)` gives `p0 = -q - 5`, and
`sigma(q) = H2` with `a_coef = -1`, `b = -2` gives `p := 2*(-q-5) - 1*(a - 2q - 1)
= -a - 9`, `RELAX = -cl(a) - 9 = -4`, `floor(-4/2) = -2 <= 0`. **Verified.**
`probes/f3_sdiv_false_bound.wf` (compiled) is the arbitration: today's checker
refuses precisely `ile(h, -3_i64)`, the bound the drafted pair would have proved.
`probes/f2_sdiv_consumer.wf` (compiled) **accepts** the consumer that divides by
`h + 2`, so the false bound is what buys the nonzero divisor, and
`probes/f4_sdiv_interval.wf` (compiled) rejects `[OP-2] residual: h +defined
2_i64`, so today's `/` image supplies nothing here and the witness pair was the
only source. With `k = 1` the two pairs coincide and no repair is needed.

**F-I1b (the repair itself). RUN, AND IT FIRED TOO.** F-I1's repair was
re-executed against the same traces and attacked with two new programs. Its
verdict is **FAIL**, and `[IND-4]`'s shape rule is the second repair.

- **The `[ENT-1]` theorem was still false.** Making the slot list syntactic at a
  fixed visit set left the *visit set* prover-dependent: clause (b)'s equality
  pair entered the pass's reachable terms only when its side condition derived,
  so a strengthening could drive the backward pass into a commit the weaker
  checker never reached - into clause (e)'s `set` refusal (`N1`) or into four
  more slots and past the cap (`N2`). Both programs are worked in 3.9.1 under the
  repaired text, where each is refused identically at every version. `[IND-6]`
  clause (i)'s dropped path condition was the same mechanism unflagged and now
  carries the same treatment.
- **Ten wording items had drifted from the rule**, each corrected in this
  revision: the `255*745` residual printed as `190125` in 3.8.3 and in F-I1's own
  `worksheets/T7-T8_refusals.md`; `[IND-5]`'s cross-reference to 3.9.5 for a
  witness worked in 3.9.7; 2.4's "seven" against eight enumerated lines; 4.5.2's
  "four corrected rows"; the two under-specifications in the slot list (an `ieq`
  path condition's slot count, and "where that clause applies"); `[IND-6]`'s
  closing enumeration omitting clause (d1)'s sign test; 4.4's and 11.5's
  "publishes nothing"; 2.4's and 3.9.1's superset paragraphs reading as if
  nothing verified is lost; and *Monotonicity* (iii)'s justification, which
  argued that prior certificates are untouched when a slot fills and now argues
  from the omitting certificate instead. **That tenth item was missing from this
  list in the previous revision**, which was the same count-versus-enumeration
  slip as its own third entry; F-I1c caught it.
- **What it confirmed by execution.** Both soundness repairs hold - `B1` refuses,
  and `B2` refuses the false bound while keeping the true one, in both directions
  and for both signs; every drafted trace reproduces; and every F-D4 fold
  recompiles with the verdicts 12.4 records.

**F-I1c (the second repair). RUN, AND IT FIRED TOO.** The shape rule was
re-executed against every trace and attacked with three fresh programs. `N1` and
`N2` were re-derived and hold; three new attacks on the visit set - a destination
reached only through an *empty* slot's terms, the `[IND-6]` clause (i) route with
a witness-introducing path condition, and a destination occurring only in a
degree-2 monomial - all failed to break it, so **the visit set is closed**. Its
verdict is **FAIL** all the same, one level further in.

- **The `[ENT-1]` theorem was still false, in the arithmetic.** `[IND-7]`
  imported `[IND-3]`'s per-step magnitude limit - a hard error naming the
  statement - into the certificate steps, while a step against an empty slot is
  *skipped*. Which steps execute is therefore the prover's, and `p := |b|*p -
  |a|*h` multiplies by a `|b|` a clause (b) pair on a `wrap` by a literal
  supplies. `N3` (3.9.1) is the program: same text, same visit set, same
  twenty-two slots, verified under the weaker prover and a hard error under the
  stronger. Read as an enumeration question it is a **determinism** hole instead,
  since whether the error fires depends on which certificate is tried first.
- **The repair is a sweep, not a patch.** The class is *every hard error
  reachable from inside the certificate check*, and each round had moved the
  prover-dependence one level down within it. 3.9.1's **reachable-error sweep**
  enumerates every reachable error and limit - eighteen rows then, twenty-four
  now - keeps as hard errors only those on counts the path's text fixes, and
  gives every content-reachable one `[IND-8]`'s discard treatment. `[IND-3]`'s
  three limits were scoped to the two polynomials that rule normalizes, which
  F-I1d then found too narrow to carry the replacement bound; the certificate
  step carries no limit and is bounded syntactically instead.
- **Four wording items**, each corrected here: the term set versus the degree-1
  monomials of `p`, never stated to be different sets though I3's own trace turns
  on it; `[IND-7]` group 2's "left no witness term" against `[IND-4]` clause (b)'s
  "in shape unconditionally"; 2.4's dangling "reconciles seven with the eight
  lines above"; and F-I1b's "ten wording items" enumerating nine.
- **What it confirmed by execution.** Every drafted trace was re-executed step by
  step under the discard semantics and **none changed**, which is stated here
  from the execution and not assumed: the largest intermediate any of them
  reaches is T8's `255*745 = 189,975`, then I2's step at `65,025` and its
  multiplier `s = 255`; every other trace stays under a hundred (I1's midpoint
  runs `2*p0 - H1` to `lo - hi + 2` and then to `1` with `s = 2`; I3's step and
  I2's and I4's bases reach `0`; I4's non-matching path reaches `-1`; T7 refuses
  at `7`; B2's two directions reach `2` and `-4`). No trace exceeds degree 2 or
  five monomials, and `2^127` is thirty-three orders of magnitude above the
  largest of them, so no drafted trace was ever near the limit that broke it -
  which is exactly why an inspection of the traces could not have found `N3`, and
  why the sweep is over the rule's reachable errors rather than over its
  examples. Every count in the file recomputes, including `988,161`, `58,625`,
  `1,021`, `748`, `133`, `189,975`, `254,745`, `255,000`, `999,000` and `N2`'s
  `37`.

**F-I1d (the sweep). RUN. The class held; the round fired somewhere else.** The
table was re-enumerated independently *before* being read, every row classified
(a) was attacked for a prover-dependent contribution, every discard row was
checked in its operative sentence rather than in the table, all five parts of
*Monotonicity* were attacked directly - including the covering lemma of (v) three
ways: move a coefficient rather than a constant, empty a filled slot, make
`RELAX` worse - and `N1`, `N2`, `N3` and every drafted trace were re-executed.
**All of it held. The visit set, both caps, the step arithmetic and part (v) are
closed, and `N3` verifies at both versions digit for digit.** Verdict **FAIL** all
the same, on the two other claims the sweep makes.

- **The resource bound had no premise.** `[IND-3]`'s scope had been narrowed to
  "exactly those two polynomials", but clause (b)'s and clause (d)'s pairs and
  clause (i)'s path conditions are shapes **the rule itself writes** and the
  backward pass then rewrites, so nothing limited them. `N4` (3.9.1) is an
  ordinary program - `requires ile(x, 1_u64)` and a `*`-chained increment - whose
  filled pair reaches **degree 8**, falsifying "degree never rises"; a four-term
  sum squared four times reaches `969 > 256` monomials and seven `*` commits pass
  `2^127`. The repair is the **widened scope**: `[IND-3]` limits every polynomial
  the shape rule produces, all of which are prover-independent in content, so the
  sweep gains rows rather than losing its argument and `[IND-7]`'s premise is
  restored verbatim. `N4` is refused as the price, uniformly and at every checker.
- **The determinism sentence went with it.** With the bound false, `[IND-7]`'s
  "a certificate a conforming implementation cannot evaluate does not succeed"
  was a live **capability** discard: implementation A evaluates exactly and
  verifies `scan`; implementation B, sized to the published bound, raises a hard
  error at `[IND-1]`. Both conform. That refuted `[IND-7]`'s determinism sentence,
  2.4 property 4 and 9's D1, and it is the defect `AGENTS.md` names outright.
  `[ENT-1]` 2835-2836 forbids an implementation-chosen **strategy**, not only an
  order. Restoring the bound makes the sentence dead letter, and the rule now
  says so instead of leaving it to be noticed.
- **Four rows were missing and the stated scope was narrower than the table.**
  `[IND-3]`'s `ine` refusal, its *Typing* requirement, its *Vocabulary fence* and
  3.8.1's per-`fn_decl` name uniqueness are now rows 3 to 6; all four are
  admission-time, syntactic, and carry no `[ENT-1]` consequence. The scope now
  reads "admitted and checked" rather than "`[IND-4]` through `[IND-10]`", which
  is what the table always covered.
- **Three under-specifications**, each closed: `[IND-4]` clause (e) said flatly
  that a refusal refuses the statement while `[IND-6]` clause (i) said such a
  substitution is dropped, so a `set` commit reached only through a branch
  condition had two readings - now one, worked in 3.9.1; `[IND-7]`'s `RELAX`
  named `[IND-6]` and `[IND-10]` as the state and omitted `[IND-5]`, leaving
  every base obligation's factor intervals sourceless; and `[IND-1]` and row 24
  stated the undischarged hard error without `[IND-8.V]`'s per-view qualifier,
  under which every statement resting on a `requires` reads as an error.
- **One wording item**: `[IND-7]`'s "each step at most **doubles** the bound"
  described multiplying by `2M = 2^128`. The figure `(2M)^(k+1) = 2^640` was
  right; the sentence was not, and the derivation is now written out.
- **What it confirmed by execution.** Every drafted trace and every drafted
  hypothesis **shape** was re-checked against the widened scope and **none
  moves**: the largest coefficient in any shape anywhere is `N3`'s
  `K = 1.5e19` in `o1 - K*w1 <= 0`, nineteen orders below `2^127`; the largest
  degree is I3's `2` (`i*factor`); the largest monomial count is `4` (I1's `H2`,
  `N3`'s statement). I1's `H1 = 2q - hi + lo`, I2's `H1 = sum - 255*i`, I4's
  `E1 = o - hits - 1`, `N1`'s and `N2`'s pairs and B2's `2q - a - 1` are all far
  inside. The widened scope is therefore free on everything this file draws, and
  costs exactly `N4`.

**F-I1e (the widened scope). RUN, AND IT FIRED - in the seam, and in a
computation with no row.** The fifth run took the fourth repair's own diagnosis
as its charter: four rounds of instance repairs had each left the argument one
step behind the text, so it attacked the *interaction* between the sweep's two
classes rather than either class alone. Verdict **FAIL**, on two findings, and
this revision is the fifth repair - the first one written as **invariants** (3.8's
four laws) rather than as another scope list.

- **The monotonicity break, and it is the same row the fourth round cleared.**
  `[IND-3]`'s widened scope was delimited by *carried into the head frame*, and a
  dropped path condition is not carried into the head frame. So whether a
  degree-8 shape was measured turned on clause (b)'s `set` refusal, which is the
  one `[IND-4]` decision a strengthening flips - and row 7's clearance ("a
  strengthening moves it only from refusing to admitting") had been written when
  admitting could only *add a hypothesis*. After the widening, admitting also
  submitted a polynomial to a hard-error limit. `N5` (3.9.1) is the ordinary
  program: verified on a checker with no `imin` row image, hard error on one that
  has it. **Three readings of one paragraph gave three verdicts on `N5`** - "every
  polynomial the shape rule produces" (safe), "carried into the head frame" (the
  break), and "an over-limit shape is row 11's discard" (a third) - which is the
  determinism defect the fourth round was convened to close, one clause away.
- **The capability finding: the `[IND-4]` backward substitution had no bound
  anywhere.** `[IND-3]`'s limits were on the shape rule's *output*; nothing
  limited the intermediate, and an implementation cannot abort early because
  cancellation restores legality. The fourth round's dead-letter argument was
  scoped to certificates and the substitution runs before any certificate exists,
  so it did not reach the one computation in the pipeline with no figure at all.
  `N6` (3.9.1) is the witness. This is the finding that made **L3** a law rather
  than a paragraph: *every* computation, not every certificate.
- **Three consequential under-specifications, each closed by a law.** `[IND-3]`'s
  *Typing* and *Vocabulary fence* read, literally, as hard errors on any loop
  whose `if` condition is not a fragment-integer `rel_term` while a `bound_stmt`
  is present - **L4**. `[IND-3]` listed group 1's statement hypotheses among the
  shapes limited "after the same backward substitution" while `[IND-6]` said they
  are never substituted - resolved for `[IND-6]`, since substituting them is
  exactly A2's soundness break. And sweep row 15's corner minimum published no
  figure while row 12's `RELAX` had just gained one - now derived, `-2^643`.
- **What held, and it is most of the file.** `[IND-7]`'s resource premise is
  genuinely restored and its bound is correct arithmetic under it; `N4` gets one
  answer from the text at every implementation; the four appended rows are
  correctly classified; and every drafted trace re-executes. The run could not
  move the visit set, either cap, the step arithmetic, or parts (iii), (iv) and
  (v) of the monotonicity argument.
- **What this revision re-executed against the final text.** All nine drafted
  traces - the seven derivations of 3.9 and the two refusals of 3.8.3 - land
  where the file says: I1's midpoint
  `2*p0 - H1 = lo - hi + 2`, then `- H3 = 1`, `s = 2`, `floor(1/2) = 0`,
  **verified**, ten slots, 1,021 injections; I2's step `255*p0 - 255*H1` to
  `255o - 65025` and `255*255 - 65025 = 0`, **verified**; I3's base and step to
  `p := 0`, **verified**, degree 2 at `i*factor`; I4's matching path
  `p0 - E1 = hits - i`, `- H1 = 0`, **verified**, eleven slots and 133
  injections, and its non-matching path `p0 - H1 = -1`, `floor(-1/1) = -1`,
  **verified**, six slots; A16's `p = cursor` **refused** at `RELAX = 7` and A2's
  at `255*745 = 189975`. Every shape in every one of them is degree at most 2
  with at most four monomials and coefficients at most 255, and every
  intermediate of every substitution step in them is inside rows 9 to 11 by
  orders of magnitude, so **the per-step form of the limits is free on
  everything this file draws**. `N1` is still refused at clause (e) at every
  version, `N2` still crosses the 32-slot cap at 37, `N3` still verifies at both
  versions by `sigma(acc) = H1` with no step raising, and `N4` still raises row
  10's hard error at every version - now at the third clause (a) step rather than
  at the end of the pass, which is the same verdict reached earlier. `N5` and
  `N6` are the two that move, and both move **to a uniform hard error**: the
  file's price list gains two programs and loses no implementation-dependent
  verdict. `B1`'s vacuous base is still refused at `floor(9/1) = 9`, and B2's two
  directions still reach `2` and `-4`.

**F-I2 (monotonicity is real).** Take any verified statement, add a row image to
the checker, and re-verify. *Refuted if* any statement moves from verified to
refused. F-I1 refuted this for the drafted text by the caps, not by the
elimination order; F-I1b refuted it again for the first repair, through the visit
set; F-I1c refuted it a third time for the second repair, through the certificate
arithmetic; F-I1d attacked the sweep and could not move monotonicity at all, and
refuted the **determinism** half instead; F-I1e refuted monotonicity a fourth
time, in the seam where a class-(b) decision gated a class-(a) hard error, and
found the pipeline's one unbounded computation beside it.

**The charter is now the four laws, one experiment each, and it is deliberately
not a row hunt.** Four rounds of attacking rows found instance breaks and left
the pattern; the fifth found the pattern by attacking the *joins* between rules.
So: **(L1)** find a quantity `[IND]` limits or counts whose value differs between
two checkers - and the sharpest form is a decision that happens *after* a
measurement but changes what was measured, which is precisely what `N5` was.
**(L2)** find a sixth prover-touched site, or show that clause (b)'s refusal
moves something other than admission - the enumeration in part (ii) is the
target. **(L3)** find a computation the pipeline performs with no figure in the
text, or a figure whose derivation has a false premise; the pass, `RELAX`, the
corner minimum and the certificate steps each now claim one, and `N6` is the
model of what a missing one looks like. **(L4)** find a rule stated for the
statement that the pipeline applies to something else - a branch condition, a
witness term, a published fact.

Only after those: find a twenty-fifth sweep row, or a row classified (a) whose
count a strengthening can move. And the seeds the earlier repairs had to close
still stand as material: a step that crosses twelve filled ordered-pair slots; a
`wrap` commit whose no-wrap side condition becomes derivable between versions;
`N1`'s shape, a `set` destination written by a call standing one operand upstream
of such a commit; `N3`'s shape, a filled pair carrying a large literal
coefficient into a step the weaker checker skipped; and now `N5`'s shape, a
prover-flippable refusal standing anywhere upstream of a measurement.

**F-I3 (the frame repair is complete).** Re-run A16's and A2's programs under
`[IND-6]`. *Refuted if* either verifies. Then generalize: for every accepted
program with a `bound_stmt`, check that the published projection is true at every
head visit, as a compiler assertion in a debug mode. **This is the only assertion
in the design that would catch a frame error I have not thought of, and it should
be built.**

**F-I4 (the path cap is not binding).** *Refuted if* any program in
`tests/programs/` or the blind-writer corpus has a loop body with more than 64
paths carrying a bound the writer would state. Count paths statically.

**F-I5 (the notation survives contact with a writer).** Give the labelled
`bound_stmt` and the three-tier redirect of 3.12 to a blind writer under the
`0098`/`0100` protocol, with no explanation of the prover. *Refuted if* the
writer states bounds the checker refuses more often than they hit any other wall,
or if they reach for the statement where retention would have sufficed and find
the experience worse than writing nothing. **This is the only falsifier in the
design that tests the notation rather than a rule, and it is the one the intent
test actually turns on.**

### 3.10 The contract system

#### 3.10.1 The diagnosis, and the mechanism it says is not needed

Two statement sequences, byte for byte the same, one inside a loop and one not:

```whitefoot
set at = step(at: at, room: room);   // step: requires ilt(at, room)
let byte = data[at];                 //       ensures ilt(result, room)
```

Outside a loop this compiles, twice in a row (`c23`, compiled). Inside a `loop`,
the identical pair is refused - `[FN-8] UndischargedCallRequirement ilt(at,
room)` (`c09`, compiled) - and if the call is deleted the subscript is refused on
its own (`c08`, compiled). **The boundary already publishes the inductive step,
on the exact commit event a loop body uses, by a route the specification already
wrote** (`[FN-9]` 1334's narrow set-receiver route). What fails is not the
contract; it is that a loop head subtracts the fact before the body can re-derive
it. So the contract-side answer to the audit's iteration-20 is **no new contract
mechanism at all**, and the interface is written instead of a second machine:

> **`[FN-9.L1]` (contract facts are ordinary retention candidates).** An
> `[ENT-3.S12]` or `[FN-10.E]` relation established on a value-commit edge is an
> ordinary fact of the state at that edge. It is subtracted at a loop head by the
> same rule as any other fact, is a retention candidate under the same condition,
> and is re-derived at a back edge exactly when its establishing call occurs on
> that edge and every one of its supports is live there. No rule gives a contract
> fact a longer life at a loop head than an operation image with the same
> support, and none gives it a shorter one. A verified `bound_stmt`'s projection
> is subject to the same sentence, in every proof view `[IND-8.V]`.

The last clause is A9's twin, added here so the two publishers cannot drift
apart at the seam. The rest is deliberately a *non*-rule, and two consequences
are worth writing down: a contract fact is never invented at a head, because
retention never invents a bound that did not hold before the loop, so an
`ensures` whose fact is false before the first iteration is not rescued; and
`[ENT-5.P0]` is load-bearing for contracts specifically, because a contract
fact's support routinely includes a `region`-scoped borrow actual and today's
checker kills before it closes.

#### 3.10.2 The requires side, kept verbatim

`[FN-8]`'s requires machinery is the healthiest thing in this area and this
design keeps it. Four properties earn that: the clause is an arbitrary total pure
`own Bool` rather than a comparison root, so `.defined` queries, named-const
predicates and any pure total row are already admitted (1233-1234); substitution
is caller-side and pre-transfer, each formal replaced by that actual's value image
in the same unchanged state, every goal judged independently, so there is no state
accumulation to make order matter (1244-1247); entry is S4 and nothing else, with
no executable prologue and no optimizer license (1252-1254); and a contradictory
entry state is legal and is metadata (1257-1263).

**All 131 `requires` clauses in the tree keep their exact meaning and need no
edit.** One compiler defect is filed rather than specified: `define here = r.at;`
is refused `[FN-8] InvalidRequires` (`c10`, compiled) while the identical
projection written inline as `requires ile(want, r.at);` is admitted (`c11`,
compiled). 1233 admits non-consuming datums, a field selection on a parameter is
one, and 1238 explicitly retains field and `deref` projections on a formal datum.
Nothing in `[FN-8]` excludes it. **File it against the compiler; write no spec
text.**

**`requires` is the language's disjunction, and that becomes teaching.** The
audit dissolves the convex-join family by factoring a correlated body into a
function whose `requires` *is* the correlation, so each call site instantiates
its own concrete pair (`s20`, `s21`, compiled, body written once). With `claim`
gone this is the writer's only route for a two-armed correlation, so it is taught
rather than discovered: one line in `docs/patterns.md` and one clause in
`[DIAG-1]`'s `[OP-4]` fix text (6.2). No rule; the machinery exists. It passes
the intent test because the writer wanted to say *"this body works whenever the
budget fits the buffer"*, and the body was going to be a function the moment it
appeared under two arms.

#### 3.10.3 The `ensures` overhaul

Three refusals, all compiled, describe the largest single hole on the boundary:

```whitefoot
ensures ieq(len(result), n);            // c01: [GRAM-9] parse error at `len(`
ensures ile(result.stop, room);         // c05: [FN-9] InvalidPostconditionSelector
ensures when Ok(value: made): ieq(made, want);   // c06: same, routed
```

The cause is `[FN-9]` 1276: "An unrouted clause is admitted only when the written
result is `own T` and T is one `[ENT-2]` fragment integer", and its routed twin
at 1278. **Every factory, every allocator, every parser that returns a span, and
every fallible version of any of them is outside the contract language.**

> **`[GRAM-9.C1]` (contract length atom).** Inside a `contract_block`, the atom
> production additionally admits `len` applied to one admitted contract place - a
> formal datum with field and `deref` projections, an earlier `contract_define`
> binder, or an `[FN-9]` result or `[FN-10]` written datum so projected - at
> array, slice or buffer type. No other operation-table row and no user or system
> call becomes an atom there; the existing mechanical fix continues to name
> `contract_define` for every other inner call.

This is **required**, not optional: `result` is not in the definition scope
(1231), so no `contract_define` can bind `len(result)`, and `c01` and `c03` die
at the same parse offset. The widened datum is unspellable without it. `len` is
not an ordinary call - spec 802 already carves its operand out by name - so the
carve-out is narrow and precedented.

> **`[FN-9.E1]` (admitted result datum).** An unrouted `ensures_clause` is
> admitted when the written result is `own T` for T an `[ENT-2]` fragment
> integer, `own K` for a struct nominal K, `own buffer<T>`, `own array<T, N>`, or
> `own slice<'r, T>`, after concrete `[FN-2]` substitution. Its symbolic
> whole-result datum is the header `result_binding`.
>
> A routed clause is admitted as exact `when Ok(value: r):` for written result
> `own Result<T, E>` where **T is any type this paragraph admits unrouted**, and
> r is that clause's fresh symbolic payload datum. Route owner, variant, field
> and freshness admission are unchanged.
>
> A **contract datum** is the clause's result datum, a formal parameter datum, or
> an `[FN-10]` written datum, in each case carrying zero or more field-selection
> and `deref` projections. A contract datum is an admitted clause **operand**
> when its finally selected type is an `[ENT-2]` fragment integer, or when it is
> written `len(D)` for a contract datum D whose finally selected type is an
> array, slice or buffer type. Borrow-mode results, float results, whole-`Result`
> datums, non-`Ok` routes and nested-payload routes remain legal ordinary results
> and supply no relation datum in this version.

Three things this does that the audit's N2 does not. It admits `array` and
`slice` as well as struct and buffer, because a rule that admits `len(D)` for a
buffer field and refuses it for an array one makes a reader memorise which
containers may be measured. It makes result, parameter and written datums **one
notion** with one projection rule and one `len` rule, which removes a case table
instead of mirroring one, and which is what makes
`ensures ile(result.start, result.stop)` fall out with no extra sentence. And it
routes the payload through the same admission - *T is any type this paragraph
admits unrouted* - which makes the audit's amendment mechanical rather than a
promise.

> **`[FN-9.E2]` (clause relation).** After recursively alpha-expanding every
> shared `contract_define`, the clause expression must have exact type `own Bool`
> and its root must be exactly one of `ieq`, `ine`, `ilt`, `ile`, `igt`, `ige`.
> Both operands must be an admitted clause operand `[FN-9.E1]`, a named const, or
> a typed integer literal, and at least one operand must contain that clause's
> result datum or, in an `[FN-10]` clause, its written datum. No proof-required
> exact operation, computed arithmetic result, subscript, ephemeral actual,
> Boolean connective, or body local becomes a relation term. The comparison
> normalizes to one finite L0 RelationTemplate; equality's two bounds remain one
> relation occurrence.
>
> *Note on origin.* A specification-fixed operation contract `[SYS-8]` may state
> a relation this fragment's vocabulary cannot verify, because it is an axiom
> about a trusted implementation. A user `ensures` states only what `[FN-9]`
> verifies from the callee's own body in this fragment. The two are the same kind
> of published fact held to different standards of origin, and a user contract is
> never widened to match an axiom.

**No Boolean connective**, deliberately and permanently recorded so it is not
re-proposed: a conjunction is two clauses (`c21`, compiled, both published), and
1287's "one finite L0 RelationTemplate" is what makes `[FN-9]`'s per-clause
independence, per-view aggregate and diagnostic `conjunct` ordinal work. A
*disjunction* is what the language genuinely lacks, and 3.10.2's per-call-site
instantiation answers it without a connective the fact state cannot hold.

**Entry images are kept and their diagnostic is repaired.** `c22` (compiled)
shows `ensures ile(result, n);` over a body that writes `n` reported as
`relation: "n - n <= 0", Unproved` - a tautology reported as unproved, because
the entry image and the live place share a spelling. The rule is right; an
`ensures` that silently switched to the exit value of a written parameter would
change what existing contracts mean. The **rendering** is the defect:

> **`[DIAG-2.E1]`** When an `[FN-9]` or `[FN-10]` relation is unproved because a
> referenced parameter entry image is unavailable, the payload names that
> parameter, its ordinal, and the NodePath of the first structural edge whose
> `[ENT-5]` kill overlapped it, and renders the entry image with the fixed prefix
> `entry ` in the relation text. The mechanical fix is: bind the entry value
> before the first write, or restate the clause over a value the body does not
> write.

**No `entry(...)` spelling is added.** A bare parameter datum already means the
entry image everywhere, and an explicit synonym would immediately raise the
question of what a bare datum means in a clause that also has an exit datum - the
exact ambiguity `[FN-10]` is built to avoid. A24's attack on this rule failed:
`j1_uniq_len_entry_image.wf` (compiled) **rejects in the callee** when a function
publishes `ilt(result, len(deref(data)))` and then shrinks the buffer through
`&uniq`, because 1300 kills the entry image on the first overlapping kill.
`[FN-9.E1]`'s widening does not reopen it, because `len(D)` for a parameter datum
obeys the same rule.

#### 3.10.4 The three establishment routes, made consistent

A verified clause is worth nothing until it lands in the caller's state, and
1330-1339 fixes four routes to do that. Three are broken and each repair is one
list item.

> **`[FN-9.E4.a]`** Strike `propagated` from the exclusion list at 1331. A
> `propagate` over an ordinary user call establishes that call's matching
> verified relations on the normal continuation binding of its
> `propagate_let_rhs`, with the `[ERR-3]` `Ok` payload as the result datum. The
> `Err` edge leaves the function and establishes nothing.
>
> **`[FN-9.E4.b]`** 1335's first-statement `set outer = payload;` route applies
> under exactly 1334's conditions on the destination: `outer` is a live bare own
> place of the exact payload type, and a relation may substitute the payload with
> post-write `outer` only when it omits the formal supplied by `outer`, if any,
> and all other supports remain live and disjoint `[OWN-7]`. **Whether `outer`
> also appeared as an actual of that call is not read.**
>
> **`[FN-9.E4.c]`** Delete `projected` from 1336's exclusion list. P may be any
> live `[ENT-2]` term place of the exact result type formed with field-selection
> and `deref` projections and no subscript suffix. Every other condition of 1334
> is unchanged, including `[OWN-7]` disjointness, which is what keeps
> `set r.at = f(..., room: len(r.data))` sound.

Each has a compiled separating pair: `c17` (rejects, `propagate` drops the
summary) against `c18` (accepts, the same call spelled `match`); the audit's
`r12b`/`r12d` for the arm-set route; and `r13a` for P-PROJ, whose aliasing attack
failed in the callee because a function cannot prove an `ensures` over a
parameter length it has just invalidated. Together they make one sentence true
that is false today: **a verified relation is delivered on every commit event
that commits the call's value, whatever the spelling of the commit** - which is
3.4's rule for operation images applied to the boundary. `[FN-9.E4.a]` is the
difference between a growable-vector module compiling and not:
`tests/programs/growable_vec.wf` propagates on lines 16 and 28.

#### 3.10.5 `[FN-10]` - the write postcondition

The hole, compiled twice. A callee that writes through `&uniq` kills the caller's
fact and publishes nothing in its place (`c14`, compiled, rejects
`cursor < len(values)`), and the obvious repair is not available either, because
`own unit` is not a fragment integer so `[FN-9]` will not look at the clause
(`c20`, compiled, `InvalidPostconditionSelector` at the *result binding*). Under
`claim` this program was accepted through `[ENT-6]` 3241-3243's `&uniq`
carve-out, which the deletion removes; with the carve-out gone the program simply
fails. **Every surveyed tradition names publishing on the boundary as the answer
here, and Whitefoot is the one that cannot spell it.**

Three spellings were considered and two rejected for the same reason - both make
one spelling mean two things depending on context. A bare `deref(slot)` meaning
the exit image flips a spelling that means the entry image everywhere else, and a
second implementation would have to index the meaning by the `[EFF-2]` `writes`
row. A `final(P)` datum former puts two datum vocabularies in one clause and
invites `ieq(final(a), final(b))`, a relation between two exit images that no
`[ENT-5]` kill ordering makes well-defined. The chosen spelling is a **clause
route**, mirroring `when Ok(value: r):` - the same production with a place where
the variant was.

> **`[FN-10]` (write postcondition).** An `ensures_clause` may carry a **write
> route** in place of a `when` result route:
>
> ```wf-ebnf
> ensures_clause := "ensures" ( "when" result_route ":" | write_route ":" )? expr ";"
> write_route    := "wrote" "(" place ":" IDENT ")"
> ```
>
> `place` is the ordinary place production, so a write place is spelled exactly
> as in a body or in an `[EFF-2]` `writes` row - `deref(slot)`, `deref(r).start`,
> `s.buf` - with no new syntax and no subscript suffix.
>
> **`[FN-10.A]` (admission).** The clause is admitted only when all of: (a) the
> place resolves to one declared formal parameter of `&uniq` mode carrying
> field-selection and `deref` projections; (b) its complete projected occurrence
> appears in this function's declared `[EFF-2]` `writes` row - a clause about a
> place the function does not write is either dead or a lie about `[EFF-2]`, and
> both deserve a diagnostic at the declaration; (c) its finally selected type is
> an `[ENT-2]` fragment integer or an array, slice or buffer type; (d) **every
> operand of the clause other than the written datum is a term the call cannot
> disturb** - a formal parameter datum whose place is not in the `writes` row, a
> `len` of such a datum, a named const, or a literal; and (e) no two write-routed
> clauses of one function have `[OWN-7]`-overlapping places. The IDENT is that
> clause's fresh symbolic **written datum**, denoting the value of that place on
> the function's normal return; it obeys the `ReservedLowerNames` discipline of
> spec line 804 exactly as a result-route payload binder does. The header
> `result_binding` is unavailable in a write-routed clause.
>
> **`[FN-10.V]` (verification).** A write-routed clause selects **every** return
> of the function - every `return` of any variant whose edge is a normal return,
> and the fallthrough when the result type admits one - and every propagated
> error exit. At each selected return, after ordinary return typing, obligations,
> calls, effects and pre-return kills, the written datum evaluates to the
> `[ENT-2]` term or constant naming that place's current value, and the relation
> is queried immediately before return transfer and edge cleanup, in the fixed
> complete-then-B view order. If the place's term is not live at that return it
> was moved out, or its root was killed and not rewritten - the relation is
> unproved at that return, and the payload names the killing edge `[DIAG-2.E1]`.
>
> **`[FN-10.E]` (establishment).** At an ordinary call c satisfying `A0(c)`, and
> after transfer, consumes, borrow commits, callee-effect kills and target kills,
> each verified write-routed relation of the callee is established with its
> written datum replaced by the caller's resolved place for that actual's
> write-place projection, and each referenced formal replaced by its ordinary
> pre-transfer actual image. Establishment is subject to the `M(c,q)` conditions
> 1327 already fixes. A relation whose substitution leaves a non-`[ENT-2]`
> operand establishes nothing, and only that relation is lost.
>
> **`[FN-10.K]` (the kill/publish seam).** The callee-effect kill of `[ENT-5]`
> 3066(b) runs first and unchanged; `[FN-10.E]` then establishes onto the killed
> places, under `[ENT-5.P0]` clause (1). The published relation therefore never
> coexists with a stale fact about the same place, and a place the callee's
> `[EFF-2]` projection does not reach is neither killed nor published about.

**Clause (a)-(c) and `[FN-10.E]`/`[FN-10.K]` are the drafted rule. Clauses (d)
and (e) and the word "every" in `[FN-10.V]` are this design's repairs**, and each
closes a break:

- **(d) is A7.** The drafted flagship clause
  `ensures wrote(deref(at): next): ige(next, deref(at));` - *"I never move the
  cursor backwards"* - cannot be established at the caller, because the formal
  `deref(at)` and the written datum resolve to the **same caller place**, whose
  pre-transfer image is dead by the time `[FN-10.K]` establishes, and `[ENT-2]`
  gives the caller no snapshot term. The relation either fails `M(c,q)` or
  substitutes to `cursor - cursor <= 0`, the exact tautology `[DIAG-2.E1]`
  diagnoses. Clause (d) refuses that clause **at admission**, where the writer can
  see why. The sentence it costs the language is real and it is flagged decision
  D6, whose answer is a caller-side snapshot term with its own kill rule.
- **(e) is A12**, and it takes the drafted rule's own author's recommendation:
  two write clauses whose places overlap are a writer error, not a checker limit,
  so they are refused at declaration rather than silently losing establishment at
  every call. The drafted silent-loss tie-break is the one place in that design
  where an ordinary program loses a fact it looks like it should have, which is
  exactly the failure mode section 6's token apparatus exists to make legible.
- **"every return" is A18**, and it is a memory-unsafety repair. The drafted rule
  selected only normal `Ok`-shaped returns while `[FN-10.E]` establishes on the
  call's single normal continuation edge, whatever variant the callee returned;
  a callee that returns `Err` having left the place at 99 then publishes
  `cursor < 4` at a caller who reads 95 bytes past a four-byte buffer.
  `j2_fn10_err_hole_shape.wf` (compiled) is that program minus the clause,
  rejecting on exactly `cursor < len(values)`. Selecting every return costs one
  word and rejects the bad callee, which is right, because it really does leave
  the place at 99. **Composing the write route with an outcome route
  (`ensures when Ok(...) wrote(P: w): ...`) is the recorded widening**, not the
  v0.40 rule; it is more expressive and costs a second route production.

**Worked example - the H3 program, repaired** *(reading; `[FN-10]` does not
exist)*:

```whitefoot
fn fill['s](slot: &uniq 's u64) -> result: own unit writes(slot) contract {
  ensures wrote(deref(slot): written): ilt(written, 4_u64);
} {
  set deref(slot) = 2_u64;
  return unit;                      // written = 2, `2 < 4` discharges
}

fn read(values: own array<u8, 4>) -> result: own u8 pure {
  let cursor = 0_u64;
  region 'w { let done = fill<'w>(slot: &uniq 'w cursor); }
  //  kill (b) removes `cursor = 0`; FN-10.E then establishes `cursor < 4`
  return values[cursor];            // discharged: len(values) = 4 from its type
}
```

Compare `c14` (the same program without the clause, refused) and `c15` (the same
program repaired by a runtime branch, accepted). `[FN-10]` is the third answer:
no branch, no claim, and the callee is the one that had to prove it. A callee
that must write two correlated places writes two clauses over disjoint places;
the audit's alternative - return one struct and commit its fields, which
`[FN-9.E1]` and `[FN-9.E4.c]` now make possible - still exists and is often
better, and `[FN-10]` covers the case where the writer does not want to change
the signature, which is the ordinary systems shape and the one
`tests/programs/byte_string.wf` writes eleven times.

**What `[FN-10]` cannot do, deliberately.** It states no element property - a
write place's finally selected type may be a container, but only so `len(written)`
is expressible; there is no term for an element and `[FN-10]` adds none (that is
I5). It states nothing about a place the `writes` row does not reach, so
`[EFF-2]` remains the single description of what a callee touches. It never
speaks about an unchecked world value: `[FN-10]` publishes a relation the callee
**proved**, and a callee cannot prove a relation about a byte the host chose. And
it grants no aliasing licence: `[FN-10.E]`'s `[OWN-7]` conditions are 1327's,
unchanged, and the callee's own `[FN-10.V]` query runs after its own kills.

**The soundness bill, restated correctly.** `[FN-10]` is the one construct in
this design that adds a new establishment event, so it carries the only real
soundness obligation here, and the drafted bill mis-stated it. Three properties
discharge it: the relation was verified in the callee **at every return that
reaches the call's normal continuation** (A18's repair is exactly the gap between
*selected* and *reached*, and no pre-existing rule closed it because no
pre-existing rule publishes on an ungated edge); the set of places it can publish
about is exactly the set `[EFF-2]` declares and `[ENT-5]` 3066(b) kills, so no
stale fact survives; and the substitution is the pre-transfer actual resolution
`[FN-8]` 1244 already performs, run in the opposite direction. The residual risk
is the `[FN-10.K]` seam and its falsifier is F-C3.

#### 3.10.6 `[FN-8]` / `[FN-9]` / `[FN-10]` unification

Three rules now describe one construct and would duplicate machinery: both
`[FN-9]` and `[FN-10]` need the datum grammar, the entry-image rule, the
selected-return walk, the per-view aggregate and the SCC publication, and
`[FN-8]` and `[FN-9]` both need the contract-expression judgment and
`contract_define`'s alpha-expansion.

**Keep the numbers; factor the shared text once.** `[FN-8]` gains a leading
**contract surface** paragraph fixing, in one place, the definition scope and
`contract_define` erasure (1231, 1235), the pure/total/non-consuming
contract-expression judgment (1232-1234), the contract-datum grammar
(`[FN-9.E1]`, plus 1285's operand list), the entry-image rule (1298-1301), and the
reserved-name discipline (804). `[FN-9]` keeps routes, selected returns and S12
establishment; `[FN-8]` keeps GoalTemplates, call-site substitution and S4 entry;
`[FN-10]` is the write route, its selected returns and its establishment - about
eighteen lines instead of the fifty a self-contained copy would need.

Collapsing all three into one rule id is rejected, and the reason is the
repository's own: *never relocate a load-bearing path merely for tidiness*, and a
rule id is the most load-bearing path in this project - it appears in diagnostics
a writer reads, in conformance case names, and in append-only approval records.
`[FN-9]` is cited at 20-plus specification lines and in every `fn9-*` case.

#### 3.10.7 The judgment, as an algorithm

```
A1  parse: contract_define* requires_clause* ensures_clause*      [GRAM-9, +C1]
A2  for each contract_define in source order: resolve in the definition scope;
      reject unless every datum is non-consuming and every row pure and total;
      record it; it is erased by alpha-expansion, never evaluated
A3  for each requires_clause: alpha-expand; require exact own Bool;
      form one GoalTemplate; occurrence = (instance, clause NodePath)
A4  for each ensures_clause: classify route BEFORE lexical resolution of the
      clause expression, as 1641 already fixes for `when`;
      admit the datum (E1 for result and payload, FN-10.A for the write place);
      admit the relation (E2); form one RelationTemplate
A5  reject an empty block, and a block with defines only                 [1229]

V1  establish every requires GoalTemplate at body entry, independently,
      in source order, as [ENT-3] S4                                     [1252]
V2  close; a contradiction makes the instance Uninhabited (metadata)     [1257]
V3  for each ensures clause in source order:
      selected-return set:  unrouted -> every explicit return            [1289]
                            when Ok  -> every direct canonical Ok return [1290]
                            wrote    -> EVERY return                     [FN-10.V]
      reject an empty set for an inhabited instance                      [1293]
      for each selected return in NodePath order: ordinary typing, obligations,
        calls, effects, kills; bind the datum; query complete, then B
V4  publish the instance's summaries atomically, per SCC, only on total
      complete success of every clause of every inhabited instance       [1310]

C1  A0(c): resolution, instantiation, named arguments, exact types, borrow
      feasibility, every actual-expression obligation, exact formal
      substitution - in that order, at one pre-transfer point            [1319]
C2  for each requires GoalTemplate in source order: substitute each formal with
      that actual's value image in the SAME unchanged state; judge
      independently; the first refuted or unproved clause owns the rejection
C3  ordinary transfer, consumes, borrow commits, callee-effect kills, target kills
C4  for each verified relation q, test M(c,q); establish each q that passes,
      on the routes of 3.10.4 and [FN-10.E]
C5  the edge order of [ENT-5.P0]
```

**Two views, not three** (A11). C2 judges each goal in one state no earlier goal
modified, so clause order changes nothing; C4's tests are per-relation and
independent; `[FN-10.A]` clause (e) removed the last order-sensitivity by
rejecting overlapping write clauses at declaration. Two implementations that
agree on the fact state before C1 agree on it after C5.

#### 3.10.8 Falsifiers

**F-C1 (the interface claim).** *Claim:* retention alone closes the
`ensures`-as-inductive-step family with no contract change. *Experiment:*
implement `[ENT-5.R]` and run `c08`/`c09` unmodified. *Refuted if* either still
rejects. **The cheapest and most important falsifier in this section**, because
`c23` already compiles the step in straight-line code, so the prediction is
narrow.

**F-C2 (the `[FN-9.E1]` reach claim).** Write the `make`/`use_it` factory pair
and compile. *Refuted if* the `ensures` verifies in the callee but does not
establish `len(data) = n` on the caller's `Ok` arm, which would mean 1332's
payload route does not carry a `len` term.

**F-C3 (the `[FN-10]` ordering claim).** Give every `writes`-carrying function in
`tests/programs` a trivially true write clause, recompile, and diff the accepted
set. *Refuted if* any program's acceptance changes on a statement whose
obligation is not about the written place. Eleven `byte_string.wf` sites plus the
deflate module.

**F-C4 (zero forced edits).** *Refuted if* any of the tree's 131 `requires` or 27
`ensures` clauses moves verdict. **Free - it is `make check`.**

### 3.11 The boundary and the world

#### 3.11.1 `[ENT-3.S10]`, generalised - with both restorations

Today S10 imports two of `[SYS-8]`'s seven relations for five named operations
and leaves the rest as "retained checked-program facts" (2990). Keeping two and
withholding five is the accreting list on the world side. The generalisation is
carried forward from the 0106 design with **three sentences the drafted version
lost or never had** (A10, A21, S3).

> **`[ENT-3.S10]` (specification-fixed operation contracts).** For a `match_stmt`
> or `value_match` whose scrutinee is a call to a `[SYS-2]` operation, or a bare
> IDENT naming a `let` binding of that call's outcome type under the existing
> no-kill, no-`set` path discipline: at each arm's entry establish every relation
> `[SYS-8]` fixes for that arm's outcome, with each declared parameter read as
> its exact actual term or constant.
>
> **Projection.** A relation is established exactly when its normalized form is a
> finite set of `[ENT-2]` atomic facts under 2901. A relation of higher arity
> establishes exactly the atomic facts obtained by folding every operand that is
> a constant, and nothing else; where no such fact remains it establishes
> nothing. In particular `next = start + required` establishes
> `next - start <= c` and `start - next <= -c` when `required` is a constant `c`,
> and the empty set otherwise. A relation whose substitution leaves a
> non-`[ENT-2]` operand establishes nothing.
>
> **Provenance.** Every relation this source establishes carries, in each result
> term's `[PRV-1]` dependency, the dependencies of every actual the relation
> names, so this source never launders an external actual into an internal
> result.
>
> **Enumeration.** Each `[SYS-8]` contract's admitted projection is enumerated
> beside that contract, in the same change that writes it, exactly as an
> operation row's image is `[ENT-3.S5]`; a contract with no admitted projection
> says so explicitly. These facts carry the trust class of a declared operation
> contract, never a writer statement.

**The projection sentence is A10.** Spec 2577 fixes `next = start + required` for
`host_copy_bytes`, a three-term relation `[ENT-2]` 2901 cannot hold, and the
escape clause does not reach it: its operands are all perfectly good terms - it
is the **arity** that is wrong. Without this sentence the rule instructs a
conforming implementation to establish a fact its own fact language cannot hold,
and each implementation invents its own projection.

**The provenance sentence is A21**, and it is a restoration. Spec 2988 reads
"Each result endpoint's `[PRV-1]` dependency additionally includes the concrete
start actual, so this relation never launders an external start into an internal
result", and the drafted replacement has four sentences and that is not one -
because the check was made against `[SYS-8]`'s contract list rather than against
the sentence being replaced. With it gone, the endpoint the **host** chose no
longer inherits the external provenance of `start`, `[PRV-3]` partitions it as an
*internal* subject, `[PRV-2]`'s bridge demand never fires, and an external value
reaches a protected leaf with no real branch. That is precisely the fence the
world-value story leans on (`t13a`/`t13b`, compiled) - and the same edit
**triples the surface it protects**, from five operations to every `[SYS-2]`
operation and from two relations to seven. Restoring it generalized, to every
actual the relation names, is the minimum.

**The enumeration sentence is S3.** The widening from five named operations to
every `[SYS-2]` operation is a second change wearing the same sentence and is
argued nowhere; the enumeration obligation is what makes it safe, because
otherwise one accreting list is replaced by an unenumerated one.

#### 3.11.2 `[ENT-3.S10.W]` - the world-values refusal, as a rule

Three of `[SYS-8]`'s sentences are about **bytes**, not endpoints: "On
`ReadBytes(next)` exactly `[start, next)` may have changed", "`[start, next)` is
the portable entry-record prefix holding exactly `entries` complete records", and
the unchanged-buffer clauses. Those are permission and content facts, and the
generalisation must not be read as importing them.

> **`[ENT-3.S10.W]`** A `[SYS-8]` sentence about which bytes of a buffer may have
> changed, or about the content of a byte range, establishes no `[ENT-2]` fact.
> It constrains the operation's permitted behaviour and is consumed by `[EFF-2]`
> and `[OWN-7]`, not by the fact state. **No construct states a relation about a
> value a host produced except by executing a comparison on it.**

That last sentence makes the owner's world-value ruling a rule rather than a
discipline, and it is the only place in the design where the ruling needs text.
Everything else enforces it structurally: `[FN-9]` verifies from the callee's own
body, `[FN-10.V]` verifies in the callee against its own kills, `[IND]` verifies
base and step, and `[PRV-3]` refuses an external subject that reaches a protected
operation without a real branch.

**Compiled, both halves.** `t13a_world_branch.wf` (= the tree's
`prv3-pos-external-branch.wf`, **accepts**) discharges a subscript whose index is
an external `args_count` result by an ordinary `ilt` guard with an honest exit on
the false edge. `t13b_world_claim.wf` (= `prv3-neg-external-claim.wf`, **rejects
`[PRV-3]` ExternalProtectedSubject**) is the same program written with a claim.
After the deletion the second program does not exist and the first is the only
spelling. And `y4_systemrange_guard.wf` (**accepts**, compiled) is the same story
on the range obligations themselves: an external `args_count` endpoint passed to
`read_at`, with `[SYS-8]`'s `start <= end` and `end <= len(destination)`
discharged by two ordinary guards over the exact terms the obligations name.
**That probe is claim customer 4, which no part of the batch had witnessed.**

#### 3.11.3 `contract_block` on a system operation: not proposed

`[SYS-2]` operations have no source `contract_block` and this design does not
give them one. Their contracts are specification text, not writer text, and
moving them into a `contract_block` would make the specification's own axioms
look like verified user clauses - the exact confusion `[FN-9.E2]`'s
note-on-origin exists to prevent.

### 3.12 The if/else residue

#### 3.12.1 `[ENT-6]`'s route menu, restated as total

**Replace 3169-3170 with:**

> Exactly one of four routes discharges any unproved family, selected by where
> the missing premise must come from. When the residual's terms are all values
> this function's own body computes and the fact holds on some but not all paths
> reaching the operation, the route is one dominating branch establishing the
> canonical goal, whose false edge does not reach the operation. When the
> residual reads a value a callable boundary produced, the route is an exact
> verified `ensures` or write postcondition on the callee and its `[ENT-3.S12]`
> or `[FN-10.E]` publication, a specification-fixed `[SYS-8]` fact, or an
> ordinary branch on the returned value. When the fact holds at every iteration
> of an enclosing loop but the head state does not carry it, the route is loop
> retention `[ENT-5.R]` or one verified induction statement `[IND]`. When the
> residual is a correlation two values acquired together on one branch, the route
> is to factor the dependent statements into a function whose `requires` states
> the correlation and to call it from each branch. **At least one route is always
> open, and `[DIAG-1]` names which one the residual selects; where the only open
> route re-establishes a fact an earlier pass established, `[DIAG-1]` names that
> earlier pass.**
>
> No route defers the obligation to runtime. The language supplies no value, no
> abort, and no fallback on the false edge of a guard: the writer's own program
> says what happens there `[GRAM-6, FN-1, ERR-4]`.

3171-3173 survive verbatim, with "makes the assertion-only route unavailable"
rewritten as "leaves only the dominating value branch" and the per-site cost
clause becoming "at a per-site cost from zero where facts already prove the bound
to one branch where they do not". The totality sentence is normative on purpose;
its qualifier is section 10's Q2, and section 11.1 prices what it does not say.

#### 3.12.2 The honest-else rule - there is no new judgment

This is the finding. The language **already** makes a dishonest else
unrepresentable, by three rules written for other reasons that compose exactly
right:

1. `[GRAM-6]` 261 - an `if_stmt` `else` whose block is empty is a hard error
   citing GRAM-6 (spell the else-free `if`). The writer cannot write a silent
   do-nothing arm and pretend it handled the case.
2. `[FN-1]` - every path of a function whose result type is not `unit` must reach
   a `return`, so in value position the else arm must produce a value and the
   writer's choice is visible in the source.
3. `[ERR-4]` as redrafted (3.1.2) - expected failures are `Result` values and
   there is no third class. So the honest choices on a false edge are exactly:
   return a typed error, return a defined ordinary value, `break` or `return` out
   of the region, or let control continue where no value is required. **Never an
   implicit trap, because the language has none.**

The one sentence added is teaching and it lives in the route menu: *the language
supplies nothing on a false edge*. Under v0.39 that sentence would have been
false, because `[CLM-1]` supplied an abort. It is the exact sentence the deletion
buys.

#### 3.12.3 The impossible-else problem, and the three reliefs refused

> **The impossible-else problem.** Let F be a fact true on every execution
> reaching an operation O that the checker cannot derive. The successor of a
> claim is `if F' { O } else { E }`, where F' is F spelled as a guard. Because F
> is always true, **E is code no execution takes**. The language will not write
> E: it has no `unreachable`, no `assume`, and no trap. So the writer invents E,
> the reviewer reads E, and the compiler emits a branch it cannot fold.

Three tiers, very different sizes:

- **Statement position: zero.** The guard is the else-free `if` and nothing is
  invented (`t14`, compiled). This is the common case, because a guarded read or
  write is usually a statement.
- **Loop position: zero to one `break`.** The guard-and-exit idiom is the loop's
  own exit test rewritten to name the obligation's term - `ige(at, length)` in
  place of `ieq(at, length)` - so the "invented" arm is the loop exit the program
  already had (`t2`, `t4`, `t8`, compiled).
- **Value position: one invented value, or one widened signature.** `t1`
  (compiled): the result type becomes `Result<T, E>` and the false edge returns
  `Err`. The signature change is honest - the function *is* partial there as far
  as any checker can tell - but it is real churn and it propagates a `match` to
  every caller. **This is the one place where the deletion visibly costs the
  writer something**, and section 11.1 keeps it in red ink.

The dishonest variants are named so a reviewer can refuse them. An `else` that
returns a plausible-looking wrong value (`return 0_u8;`) is *accepted* by the
language and is worse than a claim was, because a claim at least announced that
the writer believed the case impossible. A rewrite that adds a dominating branch
no execution can take, purely to satisfy the checker, is the audit's own
intent-test break and is the thing to look for in review.

**Three reliefs considered and refused, each with its reason:**

- **An `unreachable` or `never` construct.** Refused. It is either a runtime
  abort - the exact writer-reachable trap this design deletes, reintroduced under
  a different keyword and *without* even a review record - or it is an `assume`,
  which `[ENT-3]` 2910 and W3 forbid outright. It would also re-open section 5's
  T3 derivation on the day it closed. There is no third reading.
- **Relaxing `[GRAM-6]` so an `else` block may be empty when the checker derives
  a contradiction at its entry.** Refused as useless. If the else edge's state is
  contradictory then the guard's condition is derivable at the `if`, in which
  case the writer should have no `if`; and if they keep it, the else-free form
  already serves. It buys nothing and adds a rule whose premise is a fact query -
  the one thing `[DIAG-1]`'s schedule works to keep out of grammar-level
  judgments.
- **A "reviewed unreachable" carrying a `because` record and no runtime check.**
  Refused. With no runtime check the record is unfalsifiable by execution, so it
  is a pure trusted assertion - strictly worse than the claim, which at least
  trapped. W3's "cheating is made unrepresentable, not detected later" settles
  it.

**The relief that costs nothing and is not vocabulary at all** is the audit's own
rule and it is teaching: *branch on the term the obligation names*. Section 4.5
shows it dissolving four of the six hardest claims in the tree with no rule
change.

#### 3.12.4 Worked example - the hardest corpus shape, as it compiles today

`tests/programs/percent_decode.wf` carries four claims, two of them the census's
hardest value-flow rows (`input_index + 1 < source_length` and
`+ 2 <`, whose bridge is a three-term equality no difference-bound state can
hold). `t4_percent_escape_free.wf` (compiled, **accepts**) is that program's
escape, claim-free, with **no new rule**:

```whitefoot
loop @decode_loop {
  let more = ilt(input_index, source_length);
  if more { } else { break @decode_loop; }
  let byte = src[input_index];
  let is_percent = ieq(byte, 37_u8);
  let step = 1_u64;
  if is_percent {
    match input_index +checked 1_u64 {
      Ok(value: next_index) => {
        match input_index +checked 2_u64 {
          Ok(value: last_index) => {
            let last_ok = ilt(last_index, source_length);
            if last_ok {
              let high_byte = src[next_index];
              let low_byte = src[last_index];
              set step = 3_u64;
            }
          }
          Err(error: over_last) => { }
        }
      }
      Err(error: over_next) => { }
    }
  }
  set output_count = output_count +wrap 1_u64;
  set input_index = input_index +wrap step;
}
```

Three separate rules already in the specification make it work. The loop's exit
test names the obligation's term, so `[ENT-5]` 3095 leaves
`ilt(input_index, source_length)` on the continuation and `src[input_index]`
discharges. `[ENT-3.S7]` publishes `next_index = input_index + 1` and
`last_index = input_index + 2` on the `Ok` arms. The single guard
`ilt(last_index, source_length)` then discharges **both** subscripts, because
closure gives `next_index - last_index <= -1` from the two equalities and
composes it with `last_index - source_length <= -1`.

Nothing is invented. `has_pair = ige(remaining, 3_u64)` and `last_ok =
ilt(last_index, source_length)` are the same condition; only the second is the
obligation spelled as a guard. The two `Err` arms are empty, which is legal for a
match arm and is the honest reading - an index within one or two of `max(u64)` is
not the start of an escape. **The source's own doc sentence, "at least three
remaining bytes place the second successor strictly before source_length", stops
being a `because` record a human must validate and becomes a branch the checker
validates.** That is the successor of the `because` text, exactly as the charter
says, and it is why A23 rules the guard route ahead of two proposed rules.

**The boundary of the `+checked` respelling, compiled.** The audit's headline is
that spelling a cursor advance `+checked` rather than `+wrap` publishes the
offset equality the loop head cannot carry. That is right and it is teaching
material rather than new mechanism - but it has a boundary the audit did not
draw:

| spelling | publishes | why |
| --- | --- | --- |
| `at +wrap d` | **nothing**, unless the state range-guards the wrap | S7 2971 |
| `at + d` (exact), `at +defined d` | `s = at + k` **when `d` is a constant `k`** | S7's constant-offset equality, after the discharged `[OP-2]` site |
| `at +checked d`, `Ok(next)`, `d` constant | `next = at + k` | S7's checked-arm equality |
| `at +checked d`, `Ok(next)`, `d` a **term** | **nothing** | `next = at + d` is a relation over three terms; `[ENT-2]` 2901 has no such fact |

The last row is compiled: `L03_checked_term_publishes.wf` **rejects** with
`[OP-4] residual: at < len(data)` on statements byte-identical to
`s16_u32_checked.wf`'s except that the offset is a parameter `stride` instead of
the literal `4_u64`, with `requires ige(stride, 1_u64)` supplying everything else
the derivation needs, and `L15_varstride_loop.wf` is the same separation inside a
loop. **So the audit's nineteen-scenario respelling covers fixed-width strides,
and the variable-width chunk - a length-prefixed record reader, a UTF-8 decoder,
a percent-decoder - is not covered by it.** That family is covered by
`[ENT-3.S5.O]`'s `+` row instead: from `stride >= 1`, `next - at >= 1`, which
with `next <= length` from the frame test gives `at < length`. Without it the
writer pays `L16_varstride_guarded.wf` (compiled, accepts): one extra
`ilt(at, next)` test per chunk with an unreachable `else`. **That is the same
never-taken-arm price as I1's, in a family far more common than binary search**,
and it is the strongest argument in the design for buying the relaxed-operand
image.

### 3.13 `[PAR]`, `[TRAP-1]` and `[QUAL-3]`

Three rules carry the same erroneous-execution block: `[PAR-1]` 2010-2019,
`[PAR-2]` 2043 and `[PAR-3]` 2074-2079. Each opens with *"An execution in which
some executed `claim` is false is erroneous"*, so after the deletion the
antecedent is unsatisfiable and the whole block is vacuous.

Read carefully the block does two jobs. The first is a **promise about the
defective execution** - one complete `[DIAG-3]` record, abort without unwinding,
no second or interleaved record, only system-contract-valid transitions, no
undefined behaviour. The second is a **prohibition on the correct execution**:
"No permission, submission, completion, or fast path reads a trap latch or pays
any other cost whose purpose is to stabilize this erroneous execution." That
second sentence is T3, written into the permission rules. **Deleting the claim
deletes the first job entirely and discharges the second by construction.**

> **`[PAR-1]`** - replace 2010-2019 with: *Under a permitted overlap, bindings
> and every Whitefoot state place equal the source-order result, in every
> execution. That identity is conditional only on `[SCOPE-3]`'s trusted computing
> base and, for a program linking gated FFI frames, on ABI-well-behaved foreign
> code; an accepted program has no language runtime contract violation for this
> rule to except `[SCOPE-4]`.*
>
> **`[PAR-2]`** - 2043's second clause becomes *"That identity is conditional
> exactly as `[PAR-1]`'s is."*
>
> **`[PAR-3]`** - replace 2074-2079 with: *The identity above holds in every
> execution of L, conditional only as `[PAR-1]` fixes. No permission,
> submission, completion, or fast path pays any cost whose purpose is to
> stabilize or reproduce a defective execution, and none is available to pay: an
> accepted program has none `[SCOPE-4, T3]`.*

Everything else in all three rules is untouched - the observability sentences,
the conforming-implementation-that-overlaps-nothing sentence, the
resource-exhaustion sentence, 2080's observability sentence and the host-resources
sentence. They are about **normal** execution and were always independent of the
claim. `TERRAIN.md` section 6.12 is the check that nothing else is lost: the
permission judgment treats a `Claim` statement as an ordinary straight-line
statement with the condition as its read footprint and explicitly does not treat
the trap edge as a control edge, so **`[PAR-3]` never consumed a claim as
proof** and no permission decision changes.

**What this buys, precisely.** Three exception clauses leave the META-5 count.
The staged pipeline's permission judgment loses a **read footprint and a
non-continuing edge per former claim site**, so an overlap the claim previously
blocked is now permitted - a real, if small, P0 gain, and the direction the
charter names when it asks that parallelism become natural. The
"schedule may select which claim the record names" indeterminacy is gone; it was
the one place in the specification where an observable was schedule-dependent by
design. And `[EFF-4]` and `[SCOPE-4]` lose their fault-injection test lane,
because there is no fault to inject.

**`[TRAP-1]` is deleted, and one sentence of it is re-derived rather than lost**
(A14). Two of its sentences are not about the claim's own execution:

> Because a trap ends the owning process, **no instance resource table,
> per-instance reaper, or pending-operation transfer is required, and none
> appears on an `inline-terminal` transfer path `[QUAL-3]`**.
> Host-surviving in-process trap containment is a DEFERRED language amendment
> with its own delta `[META-5]`.

The first is a **shape guarantee about emitted code**, cross-referenced by
`[QUAL-3]` and stated as a consequence of the trap. Deleting `[TRAP-1]` leaves it
without a basis while `[QUAL-3]` still points at it. **Re-derive it into
`[QUAL-3]` from the new `[SCOPE-4]`:** an accepted program has no non-continuing
language edge at all, so no instance resource table, per-instance reaper or
pending-operation transfer is required and none appears on an `inline-terminal`
transfer path. The second sentence is a deferred-amendment record, and deleting
it silently retires a deferral; **its withdrawal is a line in the META-5 delta**
(7.3), not an omission. The general whole-process-abort sentence at 2481 survives
with its cross-reference amended from `[SCOPE-4]` to `[SCOPE-3]`, since after
the deletion whole-process abort is reachable only through the trusted base.

### 3.14 The acceptance pipeline, after the change

Stated so a second implementation reproduces it byte-identically, which is
`[ENT-1]` 2836's requirement. This replaces the drafted pipeline whose merge
order was v0.39's (A6).

```
for each concrete function instance F, in stable instance order:
  1. build FN-1's structural normal-control graph; reject unreachable statements
  2. compute the ladder K and the retention family of [ENT-5.R5]:
       the outer universe iteration around the inner deletion of [ENT-5.R7],
       each flow being step 3 run with the current family
  3. for each proof view V in [complete, s4_blinded]:        # TWO, not three
       walk F's graph once, in FN-1 order:
         at each node apply ENT-3's sources for V
                       (S1 guard, S4 requires (complete only),
                        S5..S12 images and structure, IND-8 projections
                        subject to IND-8.V)
         on each edge apply [ENT-5.P0]: image after its own kill, close,
                       scope-exit kills, close again
         at each merge take the ENT-5 join of the arm-exit states, each taken
                       through that same edge order
         at each loop head take [ENT-5.R3]'s head state
         close under ENT-4 wherever the state is queried
  4. verify every bound_stmt: base [IND-5], step [IND-6], per view [IND-8.V]
  5. for each obligation occurrence in NodePath order:
       query the complete state; if not discharged -> reject (section 6)
  6. for each call in NodePath order: discharge FN-8's goals or reject
  7. for each selected return: verify FN-9's and FN-10's relations or reject
  8. run PRV-1's two-stratum fixed point; then PRV-3 local leaves,
     then PRV-2 call targets, using complete for admission and
     s4_blinded for the external-subject partition
  9. publish the checked program
```

**Step 3 before step 4 is normative, not an implementation convenience.** The
walk applies every `[IND-8]` projection as an `[ENT-3]` source, and step 4 then
verifies the statements; so a statement's base and step are checked in a state
that *assumes* the loop's statements. That is the induction hypothesis, and
`[IND-6]` now says so in rule text rather than leaving it to this diagram, because
`[IND-4]` clause (b)'s no-wrap side condition depends on it - I4's `+wrap` step is
licensed by I4's own published `hits - i <= 0`. What keeps it from being circular
is `[IND-5]`: the base is checked at the preheader, where no projection of the
statement under check has been published, and with the statement excluded from its
own supplied hypotheses.

There is no step 10. Under v0.39 there were four more: freeze `Eligible`, run
`[CLM-2]` component residuality by re-running the walk with one S3 event masked,
run whole-occurrence residuality the same way, and run `[CLM-3]`'s `MayClaims`
closure. **The pipeline is now one forward walk per function per view, plus one
bounded fixed point over a finite candidate set, and no re-walk of anything
acceptance-bearing.**

**What a second implementation must reproduce:** the two proof views and the same
closed state at every point; the edge order of `[ENT-5.P0]`, including the
per-event reading point; the ladder `K`, the universe iteration and the limit
retention family; the ordering of step 3's projections before step 4's
verification; the certificate predicate of `[IND-7]`, including its slot list
with its empty slots in place, and the projection of `[IND-8]`; the same
rejection at the same node with the same rendered residual;
and the same `gap` token and fix string (section 6, and flagged decision D3).
**Nothing about claims, authority, residuality, counterfactuals, ledgers or
`MayClaims`, because none of it exists.**

---

## 4. The complete case walk

### 4.1 The fifty audit scenarios

Walked in the audit's order. "**Homed**" means a rule drafted in section 3
reaches it; the route column names that rule.

| scenarios | route under this design | status |
| --- | --- | --- |
| ITER-01, -02, -04, -07, -09, -11, -12, -15, -19, -25, -27; VF-04/04a/04b, -06, -06a, -11, -12, -20, -21/21a, -22/22a (the 19 DISSOLVED-EXISTING primaries and their variants) | already compile: 3.2's guard restatement, the guard-and-exit idiom, the `+checked` respelling, the factored-`requires` route | **homed, and compiled** |
| ITER-03, -06, -08, -20, -23, -28, -29, -33, -34 | `[ENT-5.R]` + `[ENT-5.P0]`, over `[ENT-3.S5]` | **homed**; worked at 3.6.5 (a), (c), (d) and 3.7 |
| ITER-05, -14; VF-14, -16, -17 | `[ENT-3.S5]`'s `%`, `/`, `imin`/`imax` and saturating rows (3.5.1) | **homed** |
| ITER-16, ITER-36 | `[ENT-6.D]` (3.5.5) | **homed** |
| ITER-24, -30, -35 | `[ENT-3.S5.O]` with A3's side condition (3.5.4) + `[ENT-5.R]` | **homed**; the `L21`/`L22` strictness separation is why the relaxed image and not P-MONO |
| ITER-13, -32 (**I1**) | `[IND-10]` (3.9.4), one construct, one procedure | **homed**, subject to flagged decision D2 |
| ITER-17, -18 (**I2, I3**) | `bound @l` + `[ENT-6.D]` | **homed**; 3.9.3 |
| ITER-21, -31, VF-07 (**I4**) | `bound @l` + `[ENT-5.X]` + `[ENT-5.P0]` | **homed**, with 3.9.5's narrowed statement |
| ITER-22 | routed `ensures` from inside the loop; `[ENT-5.P0]` | **homed** |
| ITER-26 (**I6**) | none, deliberately; `[IND-3]`'s vocabulary fence says so normatively | **correctly refused** |
| ITER-37, -38 | `[ENT-5.R7]`'s simultaneity; `[IND-9]`'s redundancy ruling | **homed**, both in rule text as the audit demanded |
| VF-01, -02, -03, -05, -05a, -08, -24 | `[ENT-3.S5]`'s value-commit closure + `[ENT-5.P0]` | **homed** (3.4) |
| VF-08a | `[FN-9.E4.a]` | **homed** |
| VF-09, -09a, -10 | `[FN-9.E1]` + `[GRAM-9.C1]` | **homed**, and wider than the audit's N2 |
| VF-12a | `[FN-9.E4.b]` | **homed** |
| VF-13 | `[FN-9.E4.c]` | **homed** |
| VF-15, -18, -19 | the shift rows with the attained-maximum correction; `[ENT-3.S5.B1]` (`ior`/`maxor`); `[ENT-3.S5.M]` (the corrected `*wrap` box) | **homed**; the last two are drafted here for the first time |
| VF-19a (**I7**) | refused: no division by a term (`[IND-4]` clause (d)); the adjacent shape that works - compute `cells` first and allocate `buffer_new(cells, ...)` - is named | **no home, honestly** |
| VF-22b, VF-23 (**I5**) | guard each use, or size the table to the value's type (`s6`, compiled) | **no home, honestly**; and it is the likely falsifier of the totality sentence (Q2) |

**Scenarios with no home: I5's two and I7's one.** The audit predicted exactly
those. **Scenarios whose route has no drafted rule: none.** That is the
difference between this file and the three part-designs it synthesises, and S2 is
why: five of the seven "missing" publishers were drafted in the file being
superseded.

### 4.2 The seven irreducibles

| id | the knowledge | disposition here | what it cost |
| --- | --- | --- | --- |
| **I1** - a probe inside a carried window | `lo <= mid < hi` with `mid = lo + (hi-lo)/2` | **closed by `[IND-10]`**, subject to D2. The window half is `[ENT-5.R]`; the midpoint is one local statement whose three normalizer powers are named exactly at 3.9.4 | no new table column, no ambient three-term relation, no shape source. The two-law dilemma stays shelved, as the owner ruled |
| **I2** - an accumulator bounded by its increments | `sum <= 255 * i` | **closed** by `bound @l` + `[ENT-6.D]` (3.9.3) | one written line, and `[ENT-6.D]`, which the corpus wants anyway |
| **I3** - an accumulator bounded by a parameter product | `acc <= i * factor` | **closed** by the same, **plus one added `requires` ceiling** the writer must supply | the extra `requires` is real migration cost and a real improvement: the function genuinely overflows without it |
| **I4** - a counter bounded by the trip count | `count <= n` after the loop | **closed**, narrowed: for a counter leaving by the false header edge, or by a `break` taken before the counter's write on that path (3.9.5) | one written line, `[ENT-5.X]`'s repair, and `[ENT-5.P0]`. The audit's own witness `s13` is itself out of bounds; `L24` is the repaired one |
| **I5** - a validated element property narrower than the element type | "every element of this buffer is below K" | **open.** The two answers are: size the table to the value's type, which genuinely dissolves the byte-wide case (`s6`, compiled, and it is the better program) and is impossible at `2^32` entries; or guard each use, which re-runs a validation the program already performed | priced at 4.4's vocabulary ledger row V5/V6 and declined. `[IND-3]`'s vocabulary fence says in rule text that no iteration notation swallows it |
| **I6** - a quantified data-structure invariant | "every `next` field in this arena is a valid index or the sentinel" | **open, and deliberately unreachable.** Both audit sides recommended rejecting any iteration notation that would swallow it, and the audit affirmed that without qualification. **`[IND-3]`'s vocabulary fence is that affirmation, written as a rule** rather than left for a reader to derive from a grammar | nothing; its home, if the language grows one, is a type invariant or a contract over the aggregate |
| **I7** - a runtime-strided walk | `k * stride < len(records)` with the stride read from a header | **open.** `[IND-4]` refuses division by a term with a stated rule and a diagnostic naming the divisor, because a term divisor's witnesses are degree 2 in the unknowns | the audit's sharpest complaint about I7 - *the family splits on whether the count is a compile-time constant, which no writer would predict* - is **not** answered, and 11.3 keeps it in red ink. The corrected `*wrap` box image (3.5.2) answers the constant half |

**I4 and I1 are the two the audit called irreducible that this design closes**, and
both are closed by the same structural idea: verify in a polynomial language,
publish in the difference-bound language.

### 4.3 The six claim customers

`[CLM-2]` 2789 fixes the closed list of what a claim could discharge, and
`TERRAIN.md` records that all 135 corpus claims name one of them.

| # | customer | what discharged it with a claim | what discharges it now | corpus claims | compiled witness |
| --- | --- | --- | --- | --- | --- |
| 1 | **SubscriptBounds** `i < len(P)` | an S3 component on the normal continuation | an S1 guard on the same comparison; or a row image bounding `i`; or a retained loop fact; or a verified statement | **55 of 83** in accepting sources; **all 18** real-program claims | `t14`, `t1`, `t2`, `t4`, `t8`, `t10` |
| 2 | **IntegerDomain**, the `.defined` goal | S3 | an S1 guard on each operand's bound; the `Ok` arm of `+checked`; **`[ENT-6.D]`**'s two-bound route | **25 of 83**, every one in a case written to exercise the rule | `t4`'s `+checked` arms, `t5`; `L08`/`L09` are the compiled evidence that `[ENT-6.D]` is required |
| 3 | **AllocationFit** `buffer_fits<T>(n)` | S3 | an S1 guard on `buffer_fits<T>(n)`, a pure total predicate the writer may branch on | **0** | `t11` |
| 4 | **SystemRange** `start <= end`, `end <= len(buffer)` | S3 | S1 guards on the two endpoint comparisons; `[ENT-3.S10]` for the returned endpoints | **0** | **`y4`** - written for this file, and the row no part of the batch had witnessed |
| 5 | **`[FN-8]` call requirement** | S3 in the caller | an S1 guard in the caller establishing the instantiated goal | **2 of 83** | `t11` |
| 6 | **`[FN-9]` selected-return proof** | S3 in the callee | an S1 guard in the callee; the boundary-export idiom (return from inside the loop where the binder is live) | **1 of 83** | `t12` |

**Two readings matter.** In programs someone wanted, the claim construct was a
**subscript-bounds construct**: all 18 real-program claims and 55 of 83 discharge
customer 1, zero discharge customers 3 and 4, one discharges customer 6. The
successor story therefore stands or falls on whether a subscript bound can be
guarded, and the ledger says it can, in every shape the corpus writes except the
six of 4.5. And **customers 3 and 4 never needed the construct at all**:
`buffer_fits<T>(n)` is a pure total predicate and the two `[SYS-8]` range goals
are ordinary comparisons, each directly branchable, and zero claims in 682 source
files ever discharged either. **Every customer now has a compiled witness**,
which was the one gap left in the batch.

### 4.4 The vocabulary ledger - the price list

Every place this design or its neighbours wanted a term `[ENT-2]` does not have.
`[ENT-2]`'s current vocabulary is difference bounds over two terms,
disequalities, and signed goals; cost is measured against that.

| id | the term | buys | closure cost | soundness bill | verdict |
| --- | --- | --- | --- | --- | --- |
| V1 | `t <= c * b`, c a literal, b a term | I2 as an *ambient* fact; the constant half of I7 | zone becomes coefficient-aware; O(n^3) stays but each step carries a rational | moderate: every row image producing such a bound must be re-verified, and `c * b`'s overflow side-conditioned | **not bought.** `[IND]` reaches I2 without it, by holding the product inside one statement's check |
| V2 | `t <= u * v`, both non-constant | I3 ambiently; I7 in full | nonlinear; no complete decision procedure, so any rule is a spec-fixed incomplete fragment | high: the fragment must be in the specification, not chosen by the implementation | **not bought**, same reason |
| V3 | three-term relations (octagon, then polyhedra) | I1 as an ambient fact | octagon is O(n^3) over 2n variables with a documented normal form; polyhedra is exponential | large: every existing image restated in the wider domain and re-reviewed | **not bought.** The owner shelved the dilemma and `[IND-10]` is the route examined instead |
| V3' | I1 by **declared statement** | I1's algebraic probes (binary, ternary, galloping) | **none in the ambient domain** | `[IND-7]`'s fragment, already needed for I2/I3/I4 | **bought** (3.9.4), subject to D2 |
| V3'' | I1 by an **exact algebraic denotation column** on the operation table | the same programs | none ambient | 60-100 rows, each a per-row correctness obligation review must discharge - roughly doubling the image column's review load | **not bought** (A22). Its own author recommends buying the experiment first, and the experiment's expected answer (six rows, not sixty) converges on `[IND-4]`'s admitted-RHS list, which is free |
| V4 | mod-k congruence `t = r (mod k)` | nothing in this audit | small: one residue per term | small | **not bought - no customer.** The two-byte fold dissolves with `+checked` (`s17`, compiled), `by k` is withdrawn, and the ipv4 congruence dissolves with a pair guard (`t8`, compiled). Recorded as priced and unbought so it is not re-proposed |
| V5 | quantified element facts, `for all e in P: e < K` | I5, I6 | instantiation is a search unless the instantiation set is spec-fixed | very large: every mutation needs re-instantiation | **not bought**, and both audit sides plus the owner asked that no iteration notation swallow I6 |
| V6 | one conservative value interval per indexable place | *part* of I5 | small: one interval per place, interval join at merges | moderate | **not bought**, and the ceiling text must say **which half of I5 it is**: `[ENT-3.S9]` 2981 **already** publishes the declared element range of a named const array, which is why `p_constarr.wf` compiles and `p_content.wf` does not. V6 generalises S9 to runtime-built tables and still does not close I5, because the fill loop's element write is a continuing kill and the component is top at the head - so V6 needs I4's machinery to be worth anything, and buying it alone buys a program nobody writes |

**V7, the halved-extent relation, is priced here because F-I1 tried to buy it and
could not.** The counted ipv4 restructure - `let half = length / 2; for @words k
in 0..half { let offset = k *wrap 2; ... }` - is the shape earlier drafts of this
file listed among `[IND]`'s worked traces. It has **no `[IND-7]` derivation**, and
F-I1 constructed and refused four attempts. The fact every route needs is
`2*half - length <= 0`: a two-term relation with a coefficient of 2. `[IND-4]`
cannot put it at the check point, because the division is committed in the
preheader and `[IND-5]` performs no substitution there. `[IND-7]`'s group 3
cannot, because it enumerates **difference** bounds `t1 - t2 <= c`. And `[IND-8]`
cannot publish it even when a local statement before the loop proves it - which
one does, `bound halved: ile(2_u64 * half, len(deref(header)));` verifying by
`sigma(q) = (2q - length <= 0)` to `p := 0`, `s = 2` - because the projection
publishes only terms whose coefficient is `+1` or `-1` and difference bounds whose
two coefficients are `+a` and `-a`. What it does publish is **the vacuous
`Z - length <= 0`**: `length` is solitary with coefficient `-1`, so the
projection fires on it with `r = 2*half`, whose corner minimum over `u64` is `0`.
Nothing else - `half` carries coefficient `2`, so no bound on `half` is
published, and the difference-bound clause needs `b = -a` where `b` is `+2`.
**The proved fact dies at the statement boundary**, and what survives is a
restatement of `u64`. So the counted restructure is routed away explicitly, and the cost is
three things worth naming: `ipv4_checksum.wf:22` is closed by the pair guard
instead (`t8` compiled, and F-D4's `rw/ipv4_checksum.wf` in the whole source), so
no corpus site is lost; the *general* "walk a halved extent by twos" shape has no
route in this design at all, and a writer who reaches for it meets a residual
naming `offset < len(deref(header))` (`L26_ipv4_counted.wf`, compiled) with no
statement that helps; and `[IND-8]`'s unit-coefficient projection is the reason,
which is a stated ceiling of the publication language rather than of the check.

**The counted family is declined twice, and both declines are stronger than the
audit's.** `rev` passes the intent test but no scenario requires it: the
descending walk dissolves under retention (3.6.5 (a), machine-checked step), and
a descending walk whose carried bound is *not* true at entry is now reachable by
a `bound_stmt`, which works for **every** monotone measure rather than for the
one shape a grammar would name. Were it ever written, the terminal decrement
`lower_capture - 1` is unrepresentable at the type's minimum, so the exit test
must be `lower_capture < binder` evaluated *before* the decrement, and the claim
that "S11 is unchanged verbatim and no other rule moves" does not survive that
repair. `by k` is withdrawn on *existing machinery first* - its sole justifying
customer compiles today with `+checked` (`s17`), and a block walk compiles with a
**variable** width `by 16` could not express (`r6`, compiled) - with a
determinism defect as a second reason (a strength-reduced `binder <= upper - k`
underflows for `0..3 by 4` and publishes a **false** fact) and a third added
here: `by k` would fork `[ENT-5.X]`, `[IND-6]`'s substitution and `[IND-8]`'s
projection into `k`-parameterized twins, four rule texts forked to express
something `s17` already compiles.

### 4.5 The corpus, dispositioned

Migration cost is not a design criterion and nothing below argues for or against
any rule; corpus counts are **existence evidence and seed samples, never scope
bounds**. This is the work list.

#### 4.5.1 The 135 claims by home

| home | files | claims | disposition |
| --- | --- | --- | --- |
| `tests/programs/` | 7 of 25 | **18** | rewritten claim-free, function by function (4.5.2) |
| `tests/conformance/cases/` | 81 | **102** | 39 `run` and 9 `accept` cases rewritten claim-free or retired with a stated reason; 33 `reject` cases deleted with their rules (7.2) |
| `tests/codegen/cases/bounds/` | 13 | **15** | all 13 fixtures rewritten; **none compiles today** (4.5.3) |

By customer, over the 83 claims in sources the current compiler accepts:
SubscriptBounds 55, IntegerDomain 25, `[FN-8]` 2, `[FN-9]` 1, AllocationFit 0,
SystemRange 0. By what reaches the fact after the deletion, over the 114
gap-stating claims: **108 bucket P** - a forward publisher over the two-term
state, which is 3.4 and 3.5 - **4 bucket B**, and **2 bucket R**, plus 21
fixtures with no gap field.

Those are census classifications of the gap a claim states, not route
assignments, and one of them was read as a route assignment and was wrong:
`wfgrep.wf:556` is bucket P by its gap and is **not** a customer of the
unsigned-subtraction image, because at v0.39 it needs the same restructure
`:553` forces (4.5.2, F-D4 `probes/w_nowrite.wf`). A bucket count says what the
claim was about; it does not say which publisher pays for it.

#### 4.5.2 The twenty named sites, walked

| site | fact | route under this design | evidence |
| --- | --- | --- | --- |
| `fir_filter.wf:31` | ring cursor in the delay line | `[ENT-5.R]` **with the ladder** + the `%` image | the drafted candidate rule loses this (A19); `y1`/`y2`/`y3` are the separating evidence |
| `fir_filter.wf:42` | tap index in taps | `[ENT-5.R]` with the ladder | same; the extent folds to 8, which is exactly the case the ladder recovers |
| `fir_filter.wf:45` | read cursor in the delay line | `[ENT-5.R]` with the ladder **and the universe iteration** | needs the outer loop's retained fact at the inner preheader (A20) |
| `utf8parse.wf:18` | `scan < len(source)` across a variable-width step | `[ENT-5.R]` + `[ENT-3.S5.O]`'s `+` row | `L03`/`L15` compile the gap; `L16` compiles the price |
| `utf8parse.wf:20` | `events <= scan`, coupled counters | `[ENT-5.R]` alone: true at entry (`0 <= 0`), re-derived on every arm | the entry-tight atom **is** the inductive one here |
| `percent_decode.wf:16` | variable-stride cursor bound | **the loop-exit respelling**: `ieq(input_index, source_length)` written `ilt(...)` with the break on the false edge, which republishes the source bound at every head. **No rule** | F-D4 `rw/percent_decode.wf` compiles it; `t4` is the same shape. The design read this as `[ENT-5.R]` + `[ENT-3.S5.O]`, which **understates** what v0.39 reaches. Does *not* transfer to `utf8parse:18` untested |
| `percent_decode.wf:18` | `writes <= scan` | **a guard on `source_length`** plus the existing `requires ige(output_length, source_length)`, at the price of one `break`. **No rule** | F-D4 `rw/percent_decode.wf`. The design read this as `[ENT-5.R]` alone, which **understates** it. See 11.1: guarding the *output buffer's own* length instead widens the `[EFF-2]` row |
| `percent_decode.wf:28`, `:31` | `input_index + 1 < source_length`, `+2 <` | **the guard rewrite** | `t4` **accepts**, compiled, no new rule (A23) |
| `wfgrep.wf:434` | `carry < input_room` through a disequality | **the guard respelling** | `t10` **accepts**, compiled |
| `wfgrep.wf:469`, `:495` | probe index inside the input | **the guard respelling**: `imin(available, input_room)` followed by `ieq(available, bounded_available)` is the long spelling of `ile(available, input_room)`, so the admission test *is* a published guard and both reads discharge by transitivity through the existing loop guards. **No rule, no `imin` image, no added guard, no added branch** | F-D4 `rw/wfgrep.wf`. The design's route - `[ENT-5.R]` + the `imin` image - is **wrong**: the site is easier than it says, and these two claims are simply deleted with their `probe_ok`/`spot_ok` bindings |
| `wfgrep.wf:553` | `source_index < input_room` through two sums | **restructure**: guard `source_index` directly against `input_room` before the read, on both edges setting `produced = moved` and breaking | **compiled**, in `probes/p6_shift_restructured.wf` and in `rw/wfgrep.wf`. `[ENT-3.S5.B]` also reaches it; Q1 is now closed on the restructure, by proof and by 228 differential cases |
| `wfgrep.wf:556` | `tail <= bounded_available` | **the same restructure as `:553`**, which `:553` forces anyway: guard `moved` before the write | F-D4 `probes/w_nowrite.wf` is the full rewrite with only that guard removed and it fails with residual `moved < len(deref(input))`, so the respelling does not reach it. The design's route - the unsigned-subtraction image, bucket P - is **wrong at v0.39**, and this site must stop being counted as a bucket-P customer |
| `ipv4_checksum.wf:19`, `:22` | even-stride offsets in an even-length header | **the pair guard** | `t8` **accepts**, compiled; its false edge is the odd-tail case the RFC specifies, and the artificial even-length `requires` is deleted with the claim |
| `par_layout.wf:90` | band index in range | `[ENT-5.R]` + the `imin` image | 3.5.1 |
| `raw_deflate_dynamic_decode.wf:32` | `bounded < 19` where `bounded = index % 19` | the `%` image | **omitted from every part-design's migration table**; recorded here |
| `x-base64-rfc-vectors-run.wf:16` | `bounded < limit` published onward by an `ensures when Ok` | the `%` image; the `ensures` then verifies from it | the only site in the tree whose `consumers:` line says it feeds a contract, and after this design the function says it with no claim in it |
| `prv3-neg-read-offset-taint.wf:44` | element content | **case deleted** with `[PRV-3]`'s claim clause | a negative provenance fixture whose subject is the refusal, not the program |
| the two conformance `[FN-8]` sites | a caller-side residual for an undischarged requirement | the image closure; else `if`/`else` | 4.3 row 5 |

**Score, after F-D4.** The three flagship sources were rewritten claim-free in
full, compiled at v0.39 and run differentially, so the eleven named sites in them
are no longer a prediction: **all eleven are compiled claim-free today** -
`percent_decode:16,18,28,31`, `wfgrep:434,469,495,553,556`,
`ipv4_checksum:19,22` - eleven claims deleted, zero rules added, zero behaviour
changes over 1,195 differential cases (691 for `percent_decode`, 276 even-length
plus 160 odd-length headers for `ipv4_checksum`, 228 for `wfgrep`). Six of those
eleven sites the design's own table understated or mis-routed, and the **five**
corrected rows above are that finding - `:16`, `:18`, the `:469`/`:495` pair,
`:553` and `:556`, five rows covering six sites, since the third row carries two;
F-D4's own score table folds them into eight rows.

Of the remaining nine named sites, eight are reached by a rule drafted in section
3 and one (`prv3-neg-read-offset-taint.wf:44`) is a case deleted with its rule;
three of the eight depend on the ladder and the universe iteration this file adds
(A19, A20) and would have been lost without them.

**What survives verbatim from the drafted score is the narrower true statement**:
`wfgrep.wf:553` has no **arithmetic** route at v0.39. `probes/p1` (the respelling
alone), `p2` (`+checked` on the offset), `p4` (`-checked` on the tail as well) and
`p5` (both checked, with the whole loop inside the `Ok` arm so no mutation stands
between the subtraction and the use) all fail identically with
`[OP-4] UndischargedBoundsObligation residual: "source_index < len(deref(input))"`.
The chain needs two sums and v0.39 does not compose them even when every term is
exact. What the site *does* have is a compiled route - Q1's option (b), in
`probes/p6_shift_restructured.wf` and shipped in `rw/wfgrep.wf` - so it is no
longer "the corpus's one true residue"; it is a site whose only route is a
restructure.

#### 4.5.3 The thirteen codegen bounds fixtures - the clearest vindication in the tree

Twelve in `tests/codegen/cases/bounds/masked-index/` and one in
`output-capacity-lockstep/`; fifteen claims. `CENSUS.md` measured that **none of
the thirteen compiles today** - every one is rejected citing `[CLM-1]` - so this
is a rewrite of stale fixtures rather than an adjustment of a working set, and
the estimate must say so.

What the claims are doing there decides the rewrite. In
`masked-index/p01-mask3-table4.wf` the whole claim is:

```whitefoot
command fn main() -> status: own ExitStatus traps {
  let value = lookup(x: 7_u64);
  claim masked_lookup_drift: ieq(value, 40_u8) because "masked lookup drift";
  return exit_status(code: 0_u8);
}
```

That is a **drift oracle** - an executable assertion that `lookup` still returns
what the fixture expects, placed in `main` so a wrong answer aborts the run. It
is not a proof residual at all, and `cases.json`'s own note says so. It also
fails `[CLM-1]` on locality, because `value` is a user-call result, which is why
none of them compiles. All fifteen claims are deleted and the oracle is respelled
as ordinary control flow, which is what it always should have been:

```whitefoot
command fn main() -> status: own ExitStatus pure {
  let value = lookup(x: 7_u64);
  let expected = ieq(value, 40_u8);
  if expected { } else { return exit_status(code: 1_u8); }
  return exit_status(code: 0_u8);
}
```

Same measurement (`proof.sites` counted in `lookup`, untouched), same failure
signal (nonzero exit instead of abort), one fewer effect category, and - unlike
today - the fixtures compile. Each `cases.json` `measurement.note` referring to
the claim check is updated in the same change; the three `False()` preemption
markers in `output-capacity-lockstep/p08` become an `if False() { ... }` guard or
are deleted with their case if the preemption they mark no longer exists, and
that judgment must be written down either way.

**This is the clearest single vindication of the deletion in the corpus.** The
only place in the tree where a writer used the claim construct as an *assertion*
rather than as a proof residual is a fixture set that has not compiled for
months, because `[CLM-1]` correctly refused it. `[ERR-4]` said assertions are not
a class; the fixtures did not believe it; the compiler enforced it; nobody
noticed for months. **After the deletion the mistake is unspellable.**

---

## 5. T3 and W3, re-derived

### 5.1 Why T3 must be re-derived at all

`docs/constitution.md` T3 is explicit that it is not an axiom and names its
premise: *"derived from W3's claim discipline, which is the load-bearing premise:
a claim is admitted only as a reviewed, independently true, always-true lemma, so
the retained trap detects a violated approved theorem, an execution that reaches
it is a defective program whose review was wrong, and a correct program cannot
reach the trap path at all."* Delete the claim and that premise does not become
false - it becomes **vacuous**. A theorem resting on a vacuous premise is not
thereby wrong, but its statement and its derivation both change, and T3's own
closing sentence ("The theorem stands while that premise stands") requires the
re-derivation to be written rather than assumed.

The change is a **strengthening**. Under v0.39 T3 said *a correct program cannot
reach the trap path*. After the deletion **no program can, because there is no
trap path.**

### 5.2 The new statement

> **T3 - Correct programs are never taxed for defects** (owner rulings
> 2026-08-23, 2026-08-25; re-derived 0108 on the deletion of `claim`): no
> permission, optimization, or fast path is withheld from any accepted program to
> stabilize, reproduce, or make observable the behaviour of an execution that
> violates a language contract; the trusted base spends nothing on such a path.
> NOT an axiom - derived from the trap-free core rather than from a review
> discipline. Its premise is now a property of the accepted set: **an accepted
> program has no writer-reachable language runtime contract violation**
> `[SCOPE-4]`. Every partial operation is admitted only by static discharge of
> its domain before lowering `[ENT-6]`; every writer-stated proof obligation -
> a `requires`, an `ensures`, a write postcondition, an induction statement - is
> verified at compile time and never deferred `[FN-8, FN-9, FN-10, IND, ENT-3]`;
> every proof-bearing construct is erased before lowering and contributes no
> instruction, no branch, no observable and no schedule-visible event; and no
> accepted operation carries a retained runtime check `[DIAG-2]`. There is
> therefore no execution of an accepted program that the language itself deems
> defective, hence no observable of such an execution for a rule to fix, hence
> nothing for a permission or fast path to buy by narrowing.
> What lies outside the guarantee is exactly `[SCOPE-3]`'s trusted computing base
> - compiler, checker, runtime, allocator, operating system, and, for a program
> linking gated FFI frames, ABI-well-behaved foreign code - together with
> resource conditions and `[STOR-6]`/`[QUAL-1]` target failures. No rule of this
> specification is stated in terms of any of them, and none of them is a language
> trap.
> **The condition under which this derivation must be redone** is now sharper
> than before: any future construct that admits a writer-spelled predicate which
> is *trusted rather than verified*, or that can fail at runtime on a
> writer-stated condition - an assertion, an expected failure, an unreviewed
> condition, an `unreachable`, or a restored `claim` - reinstates the old premise
> and puts itself outside this theorem until the derivation is redone against it.
> **A verified induction statement `[IND]` and a write postcondition `[FN-10]`
> are expressly not such constructs**: each is verified by the checker's own
> derivation before it is a fact, each is erased before lowering, and each can
> fail only at compile time.
> History: the claim-free eligibility gate was removed under the earlier
> derivation (batch 0078) and independently re-proposed during the I/O design
> round of 2026-08-25; under the new derivation a claim-free eligibility gate is
> not merely unnecessary but **unstatable**, because every program is claim-free.

### 5.3 The argument, in five steps

1. **Nothing writer-reachable traps.** By 3.1.1 the only construct that could was
   `claim_stmt` and it is deleted; by `[EFF-2]` 1372 no operation row carries
   `traps`; by `[DIAG-3]` 1978 no other construct produced the record; and by
   `[OP-4]` 876, `[OP-2]`, `[OP-9]` 945 and `[SYS-8]` no accepted proof-required
   operation retains a runtime check. The set of writer-reachable
   language-contract violations is empty.
2. **Therefore the erroneous-execution clauses are vacuously satisfied.**
   `[PAR-1]` 2011, `[PAR-2]` 2043 and `[PAR-3]` 2074 each open with "An execution
   in which some executed `claim` is false"; that antecedent is unsatisfiable, so
   each rule's identity guarantee holds in every execution (3.13).
3. **Therefore no permission is conditional on a defect's observables.** T3's
   operative content is a prohibition on narrowing; with nothing to narrow *for*,
   the prohibition is discharged by construction rather than by rule.
4. **Therefore the trusted base spends nothing on a trap path.** There is no trap
   latch to read, no record to write, no abort to sequence, no `[TRAP-1]`
   resource-teardown ordering to honour, and no `inline-terminal` reaper to
   provide (3.13).
5. **The remaining conditionality is `[SCOPE-3]`'s and is unchanged.** T3 was
   never about the trusted base and still is not; a resource failure or a
   target-layout failure terminates a process without any language rule being
   stated in terms of it, exactly as `[PROG-3]` 1488-1490 already says of a start
   failure.

The contract half of step 1 is worth its own citation, because it is what makes
"every proof-bearing construct is erased" a quotation rather than an assertion:
`[FN-8]` 1254 forbids an executable callee prologue; `[FN-9]` 1274 says a
postcondition "is neither an executable epilogue nor a trusted assertion" and
1354 forbids it any "runtime operation, hidden check, assume, optimizer license,
serialized certificate, portable identity, alternate lowering path, or ABI
field"; 1375 gives a `contract_block` an empty effect contribution; and `[IND-1]`
says the same of an induction statement in the same words.

### 5.4 W3's clause, and the one sentence to change in it

`docs/constitution.md` W3 currently reads, in part: *"partial operations are
admitted only by proof; a claim is admitted only as an independently true,
checker-unknown, load-bearing theorem over the current function's own value and
control authority, with an exact derivation record; ... accepted claims are never
removed; expected failures are typed outcomes or ordinary value/control paths
rather than false claims; ... The retained runtime trap detects a violated
approved theorem, but it cannot turn a hidden cross-function promise into
legitimate proof authority, and source cannot intentionally request that abort."*

Replace the claim clauses with one sentence and keep everything else:

> partial operations are admitted only by proof, and **every fact the checker
> uses is one the checker itself derived or verified - there is no writer-spelled
> trusted assertion and no writer-reachable runtime boundary**; a cross-function
> promise becomes proof authority only through a machine-verified callable
> boundary; expected failures are typed outcomes or ordinary value and control
> paths; and canonical bytes leave nowhere to hide edits.

That sentence is **stronger** than what it replaces, and 1.2 is its proof: after
the deletion every fact source `[ENT-3]` admits is something the compiler
observed or verified. The old W3 had to carve out one exception and then spend
four clauses fencing it.

---

## 6. Diagnostics and teaching

### 6.1 The rejection payload and the computed gap token

Today, deleting a claim from an accepting program produces exactly one
diagnostic, and `t9_residue_rejection.wf` (compiled) is it, verbatim:

```text
whitefootc: Semantics/Source [OP-4]: UndischargedBoundsObligation {
  residual: "carry < len(deref(input))",
  mechanical_fix: "establish the residual with a dominating branch, or, only
    when it is an independently true theorem outside checker rules, add a
    CLM-2-admissible residual `claim` with a complete exact `because` record" }
```

After the deletion the residual and the location are unchanged and the
`mechanical_fix` is replaced. **That one string is the entire writer-facing
migration channel**, so it is worth drafting exactly.

> **`[DIAG-1]` obligation-rejection payload.** Every rejection of an unproved
> `[ENT-6]` family, `[FN-8]` call requirement, `[FN-9]` or `[FN-10]`
> selected-return relation, or `[IND-5]`/`[IND-6]` obligation carries: the owning
> rule id and source node; the residual rendered exactly as that rule fixes; one
> `gap` token drawn from the closed set { `guard`, `image`, `flow`, `induction`,
> `boundary`, `content`, `vocabulary` }; and the one mechanical fix that token
> selects. The token is a deterministic function of the residual and the closed
> state, computed by the procedure below. It is published review and teaching
> data; **no acceptance judgment reads it.**

The 0106 design made the *writer* spell this token in the `because` record's
`checker gap:` field. With no record there is nobody to write it - so the
compiler computes it, which is better on every axis: it cannot be wrong about the
checker, it costs the writer nothing, and it turns every future ceiling raise
into a grep over diagnostics rather than a grep over source.

**The procedure.** Given an undischarged obligation whose normalized residual is
one atomic fact over terms `t1`, `t2` (or one signed goal), take the first
matching clause in this fixed order:

```
classify(O):                           # first matching clause wins
  ops := operand_terms(O)
  1. some a in ops is not an [ENT-2] term because it is or reads an element of
     an array, slice or buffer                    -> ("content", base place)
  2. some a in ops has a reaching value-commit whose row publishes the empty
     image, least NodePath first                  -> ("image", row, binding)
  3. some a in ops has a reaching definition that is a user- or system-call
     result binding, and no verified relation of that call mentions a,
     least call NodePath first                    -> ("boundary", call, callee)
  4. O's relation names a quantity the innermost enclosing loop's body commits
     together with that loop's binder or extent, and is NOT derivable in the
     closed preheader state of that loop
                                                  -> ("induction", loop label)
  5. O's relation IS derivable in that loop's closed preheader state and is not
     in that loop's head state under [ENT-5.R]    -> ("flow", loop label, kill)
  6. O's relation is derivable at some point that dominates O, least such point
     in FN-1 order                                -> ("guard", that point)
  7. otherwise                                    -> ("vocabulary",)
```

**Clause 4 is A15's repair and it is the token the batch was missing.** In the
drafted procedure the `flow` clause fired on exactly the candidates retention
**deleted** - the facts that are *not* inductive - and told the writer to state
them as an induction statement, whose step would fail on the very path that
killed the candidate; while the real `bound_stmt` customers (I2, I3, I4, and
every accumulator whose bound is false at entry) fell through to `vocabulary` and
were told to restructure. Splitting the clause on **whether the residual is
derivable before the loop** puts each family on the right side: a fact true
before the loop and lost at the head is a retention question (`flow`), and a fact
false before the loop and true one iteration at a time is a statement question
(`induction`).

Clauses 4, 5 and 6 are decidable with state the checker already computes - the
preheader state exists at `[ENT-5]` 3120 and dominator queries are ordinary
`[FN-1]` graph facts; clause 2 needs the image column's "publishes nothing"
entry, which 3.4 creates anyway. **Because no acceptance judgment reads the
token, an imprecise clause is a diagnostic defect and never an `[ENT-1]`
problem** - which is what makes it affordable at all, and why it lives in
`[DIAG-1]` rather than in `[ENT-6]`.

### 6.2 The mechanical fix channel - one table

One fix per token, fixed by the specification. Today the whole redundancy family
carries **no mechanical fix at all** (`[DIAG-1]` 1855-1856) and the obligation
families share one two-clause string; this replaces both with seven.

| token | mechanical fix, as rendered |
| --- | --- |
| `guard` | *the residual holds at `<point>` but not here; add a dominating branch on `<residual>` whose false edge does not reach this operation - and where `<residual>` names `len` of a parameter this function does not otherwise read, branch on a term the contract already relates to that length instead, because reading the parameter widens the function's `[EFF-2]` row* |
| `image` | *`<row>` publishes no fact about its result in this version; branch on `<residual>`, or compute the value by a row whose image bounds it* |
| `flow` | *the residual holds before loop `<label>` and is removed at its head by `<kill>`; re-establish it inside the body by a branch, or state it as `bound @<label>` and let the checker verify base and step* |
| `induction` | *the residual relates `<term>`, which loop `<label>`'s body writes, to that loop's binder or extent, and is not true before the loop; state the loop's running bound as `bound @<label> <name>: <relation>` - the checker verifies the base case at the preheader and the step on every body path* |
| `boundary` | *`<callee>` publishes no verified relation about this result; add an exact `ensures` or write postcondition to `<callee>`, use the system operation's specified fact or typed outcome, or branch on the returned value* |
| `content` | *no term names an element of `<base>`; bound the index instead, or size the indexed object to the value's type* |
| `vocabulary` | *the residual is not derivable from any two-term fact this version admits; restructure so the operation is guarded by a comparison over the terms it names, or - when the residual is a correlation two values acquired together on one branch - factor the dependent statements into a function whose `requires` states the correlation and call it from each branch* |

Two of the seven are `[DIAG-1]` 1859's existing restructurings, kept verbatim in
the `boundary` row. The `content` row's second clause is the `s6` finding - sizing
the fold table to the value's type dissolves the byte-wide half of I5 and is the
better program. The `vocabulary` row's second clause is 3.10.2's disjunction
teaching, folded in here rather than drafted as a second selection procedure -
which is the seam A24 flagged when two parts wrote two `[DIAG-1]` fix-selection
rules for the same payloads. **There is one procedure and one table.**

**The `guard` row's second clause is F-D4's, and without it the channel points a
writer at a signature change.** F-D4's first rewrite of `percent_decode.wf:18`
took the obvious route the drafted row hands out - `let output_room =
len(deref(out)); let room_left = ilt(output_index, output_room);` - and it is
rejected, not for the bound, which it discharges, but by `[EFF-2]`:

```text
[EFF-2] EffectMismatch expected_row: "reads(out, src), writes(out)"
                       found_row:    "reads(src), writes(out)"
                       missing:      ["reads(out)"]
```

**Reading `len` of a `&uniq` parameter is a read of it.** The original reached that
length only through the contract's `define output_length = len(deref(out))`, so
the guard route on a write-only output parameter widens the published effect row,
and the widening propagates to every caller that declares one. A mechanical fix
that changes a signature is not a mechanical fix. The escape is the row's second
clause - guard `output_index` against `source_length` and let the existing
`requires ige(output_length, source_length)` carry it - and 11.1 records that the
escape exists only for a function that already has such a contract.

A worked rendering, for `t9`'s residual after the deletion:

```text
whitefootc: Semantics/Source [OP-4]: UndischargedBoundsObligation {
  residual: "carry < len(deref(input))", gap: "vocabulary",
  mechanical_fix: "the residual is not derivable from any two-term fact this
    version admits; restructure so the operation is guarded by a comparison
    over the terms it names" }
```

and the writer's edit is `t10_residue_repaired.wf` (compiled, **accepts**):
`ine(room, 0_u64)` where `room = input_room -wrap carry` becomes
`ilt(carry, input_room)`. Same condition, same behaviour, one guard, no rule.

### 6.3 `docs/patterns.md`

**P8 "Claims to the boundary" - DELETE.** Its whole subject is where to place a
claim. Its one surviving sentence - *a fact about a callee result belongs in that
callee's verified `ensures` and reaches the caller through S12* - moves into P14.
Its historical speed evidence, the retired `wc` experiment in which "a
trap-per-increment form produced no vector operations while the semantically
valid wrapping-counter form reached full SIMD and roughly 2x throughput", moves
into P19, where it is the *general* argument rather than an argument about claim
placement. **That number is historical and about a different comparison; it must
not be re-used as this design's own measured gain.**

**P14 "Claim only the proof residual" - REPLACE ENTIRELY** with:

> **P14. Branch on the term the obligation names.** *Problem:* a partial
> operation needs a fact the checker does not derive, and the writer already
> computes a Boolean equivalent to it - `ige(remaining, 3_u64)`,
> `ine(room, 0_u64)`, `ieq(at, length)`. *Pattern:* rewrite that Boolean so its
> two operands are exactly the terms the obligation names, and guard on it.
> `ilt(last_index, source_length)` in place of `ige(remaining, 3_u64)`;
> `ilt(carry, input_room)` in place of `ine(room, 0_u64)`; `ige(at, length)` as a
> loop exit test in place of `ieq(at, length)`. The condition is the same
> condition, the behaviour is the same behaviour, and only the second spelling is
> a fact `[ENT-3.S1]`. *Why it works:* the entailment state holds two-term
> difference bounds; a guard whose operands are the obligation's own terms lands
> as exactly the relation the obligation wants, while a guard over a derived
> quantity needs the bridge equation - `remaining = source_length - input_index`,
> three terms - which no difference-bound state can hold in either direction.
> *Current value:* this single rewrite dissolves the two hardest value-flow
> claims in the tree and two of the hardest iteration claims with **no rule
> change** (`t4`, `t8`, `t10`, compiled).

**New P19. Guard in statement position; widen the signature in value position.**
The else-free `if` is free; a false edge that must produce a value is not. When a
guard's false edge cannot be taken but a value is required there, prefer widening
the result to `Result<T, E>` and returning `Err` over inventing a plausible
ordinary value: the first is honest about the function's domain, the second hides
a defect in code no test reaches.

**New P20. The loop exit test is the loop's guard.** The exit test of an ordinary
`loop` is the only fact the head carries about the cursor. Spell it as the
comparison the body's subscripts need and the body needs no other guard, whatever
the stride (`t2`, `t4`, `t8`).

**New P21. State the loop's running bound, not the prover's intermediate.** When
a fact becomes true one iteration at a time - an accumulator's ceiling, a
counter's relation to the trip count - write it as `bound @l name: relation;` at
the top of the body, in the arithmetic the fact actually has. Do not introduce
`let`s to name the prover's intermediate values: the statement is the conclusion,
and the checker finds the rest. When the fact is *already* true before the loop,
write nothing - retention carries it.

`docs/patterns.md`'s header paragraph and its "Known gaps" section both need one
pass for claim references, and P12 ("External constrained subject takes a value
path") is unchanged and becomes the *only* rule for world values rather than one
of two.

### 6.4 The redundancy note, and the fence it needs

As the prover grows, guards written against an older ceiling become dead code the
writer would like to delete. A **verdict** here would reintroduce exactly the
monotonicity exception this design removes - `t3_redundant_guard.wf` (compiled,
accepts today) would stop compiling - so it must never be one.

> **`[DIAG-1]` review note (non-blocking).** When the entailment derives an
> `if_stmt` or `value_if` condition's comparison relation at that node, the
> compiler may emit, on a separately selected review channel, the note *"the
> entailment derives `<residual>` here from `<sources>`; this guard's false edge
> is unreachable and the guard may be deleted."* **No acceptance judgment reads
> this note, and no configuration of a conforming implementation converts it into
> a verdict, a warning that fails a build, or any other acceptance-bearing
> output.** The same applies to a verified `bound_stmt` whose projection the
> state already derives `[IND-9]`.

The last sentence is A13's fence, and it is deliberately stronger than "gate it
behind a falsifier": the drafted recommendation homed this note in `[DIAG-3]`
1985, a rule the same design deletes, and left the "may become a verdict"
question to a build flag. **A note that can become a verdict by configuration is
the monotonicity exception with an extra step**, so the prohibition belongs in
the rule text. This is the only mechanism found that gives `[CLM-2]`'s redundancy
rule its benefit without any of its cost.

---

## 7. Conformance and activation

### 7.1 What the conformance corpus does

517 cases; **81 have a claim in their source** and **55 cite a `CLM-*` rule** (53
of those 55 are claim-bearing; `clm3-pos-transitive-value-branch` and
`clm3-neg-static-conjunction-unproved` cite CLM-3 without writing a claim, which
is the point of both).

| expect | cases | disposition |
| --- | --- | --- |
| `run` (exit 0) | 39 | **rewrite claim-free and keep.** These are executable behaviour cases whose claims are incidental scaffolding for a subscript. Each keeps its id, its `rules` list loses `CLM-*`, and its source gains a guard. Any case that cannot be rewritten without changing what it executes is **retired with an honest technical reason in the same change, never silently** |
| `accept` | 9 | **rewrite claim-free and keep**, same rule |
| `reject CLM-1` | 7 | **delete.** Every one asserts a claim-admission refusal; with no claim there is no judgment to test. Two of the seven are the trap cases and go with `[DIAG-3]` |
| `reject CLM-2` | 4 | **delete**, same reason |
| `reject CLM-3` | 6 | **delete**, plus the two non-claim-bearing CLM-3 cases: 8 in total |
| `reject PRV-2` | 6 | **rewrite, do not delete.** These test that external data cannot reach a protected subject; each currently exercises the claim route and must be re-cut against the surviving route so the rule keeps its coverage |
| `reject PRV-3` | 3 | **rewrite**, same reason; `prv3-neg-read-offset-taint` is deleted, its subject being the claim-only route over element content |
| `reject EFF-2` | 5 | **rewrite or delete** depending on whether the case is about `traps` (delete with the category) or another row (rewrite) |
| `reject FN-8`, `reject FN-3` | 1 each | rewrite |

Net: **20 cases deleted, 63 rewritten**, and the manifest's `rules` coverage for
`CLM-1`, `CLM-2` and `CLM-3` disappears with the rules. The manifest's non-case
policy rows for `DIAG-3`, `SCOPE-4`, `EFF-4`, `TRAP-1`, `PAR-1`, `PAR-2` and
`PAR-3` all carry claim-specific `reason` prose today and must be rewritten or
deleted in the same change; the `SCOPE-4` and `EFF-4` rows lose their
fault-injection coverage entirely, because there is no fault to inject.

**Two verdict families hold with their prose re-narrated.**
`fn9-neg-named-outcome-no-publication` holds - a named stored outcome still
carries no pending summary, and only `propagated` leaves the exclusion list -
with its `doc` restated to the new list. `fn9-neg-entry-image-kill` keeps verdict
`reject FN-9` and gains the `entry ` prefix and a killing-edge coordinate; since
the manifest pins only `{"kind":"reject","rule":"FN-9"}`, the case is unaffected
mechanically and only its `doc` gains a sentence. All 15 `fn8-neg-*`, all 3
`fn8-pos-*` and all 3 `fn9-pos-*` are unchanged.

### 7.2 New cases required

Conformance evidence is what `AGENTS.md` rule 4 makes the owner record, so this
list is the one to get right. Each row pins one sentence of new normative text;
rows 40 to 42 carry F-I1's four cases, row 43 F-I1b's, row 44 F-I1c's, rows 45
and 46 F-I1d's and rows 47 to 49 F-I1e's, each added after the run that asked for
it. **Rows 47 to 49 are one per law that this round moved**: 47 is L1, 48 is L3,
49 is L4. L2 has no case of its own because its content is an enumeration -
row 43 already pins the shape rule's half of it, and the rest is checked by
reading, not by running.

| # | case | expect | what it pins |
| --- | --- | --- | --- |
| 1 | `scope4-pos-no-writer-trap` | accept | a program exercising every proof-required family, with an empty effect row where v0.39 would have required `traps` |
| 2 | `eff1-neg-traps-spelling` | reject EFF-1 | `traps` in an effect row is now a syntax error, not an over-declaration |
| 3 | `gram4-neg-claim-spelling` | reject GRAM-4 | `claim x: c because "...";` no longer parses; the word is retired, not free |
| 4 | `gram4-neg-deny-claims-spelling` | reject GRAM-4 | same for the `fn_decl` marker |
| 5 | `ent1-pos-monotone-guard` | accept | the `t3` shape: an `if` on a fact the checker already derives is accepted, so a strengthening cannot reject it |
| 6 | `ent3-pos-guard-negation-continuation` | accept | the `t2` shape |
| 7 | `ent3-pos-checked-arm-pair-guard` | accept | the `t4` shape |
| 8 | `prv3-pos-external-branch-only` | accept | replaces `prv3-pos-internal-claim` |
| 9 | `prv2-neg-bridge-only` | reject PRV-2 | 3.1.3: the surviving demand kind is `bridge` and a direct demand cannot be constructed |
| 10 | `sys8-pos-range-guarded-external` | accept | **the `y4` shape**: `[SYS-8]`'s two range goals discharged by guards over an external endpoint |
| 11 | `op4-neg-residue-gap-token` | reject OP-4 | the `t9` shape, pinning the rendered residual **and** the `gap` token and fix string |
| 12 | `op4-neg-induction-gap-token` | reject OP-4 | 6.1 clause 4: an accumulator residual is classified `induction`, not `vocabulary` |
| 13 | `ent5-pos-close-before-kill` | accept | `[ENT-5.P0]`: `r7_closure_vs_kill.wf`'s shape, which rejects under the old order |
| 14 | `ent3-pos-backward-wrap-reading-point` | accept | `[ENT-3.S5.B]` reads the state after the kill, after the forward image, after one closure, once |
| 15 | `ent3-neg-backward-wrap-self-destination` | reject OP-4 | `set at = at -wrap 1_u64` publishes no backward fact |
| 16 | `ent3-pos-shl-attained-maximum` | accept | u8, `ha = 200`, `k = 1`: the image is 254, and a program relying on 255 is refused |
| 17 | `ent3-neg-mul-box-leaves-type` | reject OP-2 | `[ENT-3.S5.M]` publishes nothing when `ha*hb` exceeds the type |
| 18 | `ent3-neg-relaxed-mul-needs-positive-b` | reject OP-2 | **`j03`'s shape**: the `*` row is silent without `Z - b <= -1`, so the false fact is not published |
| 19 | `ent6-pos-defined-two-nonconstant` | accept | `[ENT-6.D]` |
| 20 | `ent5-pos-retention-constant-ladder` | accept | **`y3`'s shape**: a constant-extent walk compiles because the ladder offers the weakening |
| 21 | `ent5-neg-retention-nested-simultaneity` | reject OP-4 | `r_p1_nested.wf` must remain a rejection |
| 22 | `ent5-pos-retention-nested-universe` | accept | `fir_filter.wf:45`'s shape: the inner loop receives the outer's retained fact |
| 23 | `ent5-pos-counted-false-edge` | accept | `[ENT-5.X]`'s second conjunct under `lower <= upper` |
| 24 | `ent5-neg-counted-false-edge-descending` | reject OP-4 | **`L23`'s shape**: `for i in 5..3` leaves the binder at 5, so the conjunct is withheld |
| 25 | `ind-pos-counter-trip-count` | accept | I4 closed (3.9.5) |
| 26 | `ind-neg-step-frame` | reject IND-6 | **`j3`'s shape**: the statement is refused because the check reads the head state, not the exit state |
| 27 | `ind-neg-wrap-without-side-condition` | reject IND-4 | **`j01`/`j02`'s shape**: without `b - a <= 0` a `-wrap` commit contributes only an opaque witness and its constant bounds, and on a `set` destination it refuses |
| 28 | `ind-pos-redundant-statement` | accept | `[IND-9]`: a statement the checker could prove without it is verified and kept |
| 29 | `ind-neg-path-cap`, `ind-neg-magnitude-cap` | reject IND-4 / IND-3 | the two spec-fixed limits and their hard errors, both on counts the path's text fixes (3.9.1's sweep, rows 8 and 9) |
| 30 | `fn9-pos-result-len-unrouted`, `-result-field-projection`, `-result-to-result`, `-routed-buffer-payload` | accept | `[FN-9.E1]`'s four widenings |
| 31 | `gram9-pos-contract-len-atom` | accept | `len(P)` inline in a clause, no `contract_define` |
| 32 | `fn9-pos-propagate-publishes`, `-arm-set-same-binding`, `-projected-set-receiver` | accept | `[FN-9.E4.a/b/c]`, each a compiled separating pair today |
| 33 | `fn10-pos-write-postcondition` | accept | 3.10.5's worked example |
| 34 | `fn10-neg-write-place-not-in-writes-row` | reject FN-10 | admission clause (b) |
| 35 | `fn10-neg-entry-image-operand` | reject FN-10 | **admission clause (d)**: A7's flagship clause is refused where the writer can see why |
| 36 | `fn10-neg-overlapping-write-relations` | reject FN-10 | admission clause (e) |
| 37 | `fn10-neg-err-return-unconstrained` | reject FN-10 | **`j2`'s shape**: a callee leaving the place at 99 on its `Err` return is refused, because every return is selected |
| 38 | `ent3-pos-sys8-projection-constant`, `ent3-neg-sys8-projection-symbolic` | accept / reject OP-4 | `[ENT-3.S10]`'s arity projection: `next = start + required` publishes two bounds for a constant `required` and nothing otherwise |
| 39 | `prv3-neg-sys8-external-start` | reject PRV-3 | **the restored provenance sentence**: an external `start` is not laundered into an internal endpoint |
| 40 | `ind-neg-base-self-discharge` | reject IND-5 | **F-I1's B1**: `bound @spin lie: ile(idx, 0_u64);` with `idx = 9`; the base may not use its own statement polynomial, and the consumer would write one byte past a one-byte buffer |
| 41 | `ind-neg-signed-division-witness`, `ind-pos-signed-division-witness` | reject IND-10 / accept | **F-I1's B2**: `ile(h, -3_i64)` over `let h = a / 2_i64;` with `a = -5` is refused under clause (d2), and the true `ige(h, -5_i64)` still verifies. 845 truncates toward zero |
| 42 | `ind-pos-local-hypothesis-rewrite` | accept | **F-I1's F2**: I1's midpoint, whose certificate exists only because the backward pass rewrites `span` inside the division witnesses |
| 43 | `ind-pos-visit-set-through-empty-slot` | accept | **F-I1b's `N1` shape, positive half**: a clause (b) commit whose pair is empty still puts `P`'s terms in the term set, so the earlier commit is visited and the slot count is the same on a checker that fills the pair. Pins `[IND-4]`'s shape rule as a count, not as a fact |
| 44 | `ind-pos-step-overflows-no-error` | accept | **F-I1c's `N3` shape**: a filled clause (b) pair carries a large literal coefficient into a certificate step whose intermediate passes `2^127`. The step raises nothing, that certificate is discarded, and the statement verifies on the certificate that omits the term. This is the case that separates a discard from a hard error, and the only executable check on 3.9.1's sweep row 18 |
| 45 | `ind-neg-hypothesis-shape-degree` | reject IND-3 | **F-I1d's `N4`**: a clause (b) pair rewritten by three clause (a) commits into `o - hits - x^8 <= 0`. The limit is on the **shape**, so it fires identically whether or not the pair's side condition derives - which is what makes `[IND-7]`'s resource bound true. The one executable check on the widened scope of rows 9 to 11 |
| 46 | `ind-pos-refusing-commit-through-branch-condition` | accept | **F-I1d's clause (e)/(i) witness**: `set t = load(buf, i);` reached only through the branch condition governing the path. The condition is dropped, its slot and the commit's are present and empty, and the statement verifies. Pins the one reading of the three sentences that used to disagree |
| 47 | `ind-neg-dropped-condition-shape-degree` | reject IND-3 | **F-I1e's `N5`**: a branch condition rewritten by three clause (a) commits into `x^8 - t + 1 <= 0`, over a `-wrap` commit whose side condition a stronger checker derives and a weaker one does not. The limit is measured **when the shape is produced**, so the case rejects identically whether the condition is later dropped or kept - the executable check on L1, and the one case in the corpus that would have caught `N5` |
| 48 | `ind-neg-substitution-intermediate-degree` | reject IND-3 | **F-I1e's `N6`**: a chain of `*checked` `Ok`-arm bindings whose substituted obligation cancels to `0 <= 0` but whose intermediate passes degree 4 at the third step. Rejects at the step that produces the degree, before anything large is materialized. The executable check on L3's per-step limits and the only case that pins the pass's own figure |
| 49 | `ind-pos-branch-condition-outside-fragment` | accept | **F-I1e's L4 case**: a loop carrying a `bound_stmt` whose body branches on a boolean returned by a call, and a second whose branch names an element of an indexable place. Neither condition yields an `[IND-3]` polynomial; each contributes one always-empty slot and the statement verifies. Pins that `[IND-3]`'s *Typing* and *Vocabulary fence* are admission rules on the statement and do not travel to branch conditions |

Cases 18, 26, 27, 35, 37, 39, 40 and 41 are the ones that would have been
forgotten: each pins a repair that turns an admitted memory-unsafe or laundering
program back into a rejection, and each has a compiled premise or a compiled
arbitration in section 12's ledger. Cases 40 and 41 are F-I1's two soundness
findings, and 41's accepting half is what keeps the repair from being a blanket
weakening of the division witness.

### 7.3 The META-5 delta, summed

A24's D-7: three parts declared three half-deltas and nobody summed them. The
merged declaration, in the specification's own form, counting `[IND]` as **one**
numbered rule with ten clauses and `[ENT-5.R]`/`[ENT-5.X]`/`[ENT-5.P0]` as
clauses of `[ENT-5]` rather than as new rules:

> numbered rules **+2/-5** (135 remain: `[CLM-1]`, `[CLM-2]`, `[CLM-3]`,
> `[DIAG-3]`, `[TRAP-1]` deleted; `[IND]` and `[FN-10]` added);
> grammar productions **+3/-2** (76 remain: `bound_stmt`, `rel_term` with its
> `affine`/`product`/`factor` levels counted as one, and `write_route` added;
> `claim_stmt` and its `stmt` alternative deleted, and the `deny_claims`
> terminal removed from `fn_decl`);
> unique fixed lowercase grammar atoms **+2/-3** (`bound` and `wrote` added;
> `claim`, `because` and `deny_claims` moved to the retired-spelling reservation
> list);
> writer operation spellings **+0/-0**; opaque system nominal spellings
> **+0/-0**; **runtime-trap families +0/-1 (0 remain)**;
> entry forms **+0/-0** (the `deny_claims command fn main` prefix is deleted,
> leaving one form);
> contract block forms **+0/-0** (the write route is a clause route, not a block
> form); system operations and declaration records **+0/-0** (203 remain);
> exception clauses **+0/-3** (`[PAR-1]`, `[PAR-2]` and `[PAR-3]`'s
> erroneous-execution clauses);
> effect categories **+0/-1** (`traps`; three remain);
> fact sources **+1/-1** (eleven remain: S3 retired beside S8, `[IND-8]`'s
> projection added);
> proof views **+0/-1** (two remain);
> PRV demand kinds **+0/-1** (one remains);
> **deferred amendments +0/-1**: `[TRAP-1]`'s host-surviving in-process trap
> containment is withdrawn, because the construct it would have contained does
> not exist.

**The selection ground is owner-directed, not evidence-selected**: the charter of
2026-08-29 discards the construct. The header must say so in those words rather
than citing a campaign, because no measurement selected it and pretending
otherwise is the one thing `META-5` exists to prevent.

**This is a narrowing of the accepted set** - every claim-bearing program is
rejected - and it is the first such narrowing the project has made deliberately.
The `[ENT-1]` monotonicity sentence of 3.1.2 is written to hold **from v0.40
forward**, not across the v0.39-to-v0.40 boundary, and the amendment record in
`governance/APPROVALS.md` must say so in those words.

### 7.4 Activation mechanics

The mechanism is built and needs no change; this is the exact sequence, so nobody
has to reverse-engineer the Makefile.

1. **On the work branch the active file becomes a candidate.** Set the header to
   `Status: CANDIDATE v0.40 supersedes v0.39 <v0.39's recorded digest>`, keep the
   title token at `v0.40`, and edit `spec/kernel-spec.md` in place.
   `make spec-candidate-integrity` passes exactly when v0.39 is the one recorded
   version without an archive, the declared supersedes digest is v0.39's recorded
   digest, and v0.40 is v0.39's successor. The candidate's own digest is
   deliberately unchecked, which is what lets several halves land on the same
   branch independently and be merged before activation.
2. **Write the META-5 delta (7.3) and the selection ground.**
3. **At merge time the outgoing bytes are archived.** Copy the v0.39 bytes to
   `spec/kernel-spec-v0.39.md`, set the active header to `Status: ACTIVE v0.40`,
   and append to `governance/APPROVALS.md` the `ARCHIVE-SPEC` and `ACTIVE-SPEC`
   records. `make check`'s `spec-archive-integrity` stage rejects `CANDIDATE`
   status, so a merge-ready revision must have done exactly this;
   `spec-append-only` and `spec-digest-sync` then hold the chain.
4. **Record the conformance content** as `AGENTS.md` rule 4 requires: the exact
   added, modified, deleted and renamed conformance content and its before/after
   boundary. For this amendment that is 7.1's table plus 7.2's list, and it is
   unusually large - **20 deleted cases is the biggest single conformance
   deletion the project has made.** The record must state, in the owner-approved
   text, that the deletions are of cases whose subject rules no longer exist, and
   name each one. Deleting a failing test to go green is a governance breach; the
   only thing distinguishing this from that is the record.
5. **Derived material in the same change**, as `AGENTS.md`'s consistency rule
   requires: the lexer/parser and generated syntax data lose three atoms and two
   productions and gain two atoms and three productions; `docs/patterns.md` takes
   6.3; `docs/constitution.md` takes section 5's T3 and W3;
   `docs/roadmap.md`'s PROOF-* entries lose their claim framing; the compiler
   deletes `claim_locality.rs` and the CLM-1/2/3 passes and the `Full-minus`
   scratch machinery; `tests/codegen/cases/bounds/*/cases.json` takes 4.5.3.
6. **New gate tests wired in the same change**: (a) every operation-table row has
   an image entry (3.4's totality test); (b) every `[SYS-8]` contract has an
   admitted projection (3.11.1's enumeration sentence); (c) the `[IND-8]`
   head-truth assertion of F-I3, in a debug mode.

**The sequencing note that decides the shape of the merge.** The deletion
narrows the accepted set and the publishers widen it. Landing the deletion first
leaves every claim-bearing program rejected with no replacement route for the 108
bucket-P facts, so `make check` cannot be green at that point - which by
`AGENTS.md` rule 3 means the deletion cannot be merged alone. **The deletion, the
image closure and its rows, retention, the exit rule and the induction statement
are one merge.** This is a scheduling fact, not an approval requirement.

---

## 8. The implementation plan

Design only; no code is written here. Sizes are estimates and are the numbers I
would most expect to be wrong. **Batches B1 through B5 are one merge to `main`**
by 7.4's sequencing note; they are ordered so that each is independently
reviewable and each leaves the tree buildable, not so that each is separately
mergeable. Each carries its own derived material.

**B0 - the two cheap experiments. DONE, and both moved rule text - F-I1 twice.**
They were the first work of this batch precisely because neither needs a line of
compiler code and both would move rule text if they failed. Both did, and F-I1's
own repair was then re-executed and failed in turn (F-I1b).

F-D4 rewrote `percent_decode.wf`, `ipv4_checksum.wf` and `wfgrep.wf` claim-free
in full: **MIXED**. All three compile at v0.39 and are byte-identical to the
originals over 1,195 differential cases, eleven claims deleted with no new rule;
the impossible-else bill came out 8 / 3 / 0 with no value-position invented
return, which is 11.1's load-bearing prediction and it held. Against that, five of
4.5.2's rows were wrong or understated - six of the eleven sites - one `[EFF-2]`
interaction was missed, and Q1 resolved. 4.5.2, 6.2, 10 and 11.1 carry the
corrections.

F-I1 hand-executed `[IND-7]`'s certificate check against the seven derivations of
3.9 and the two refusals of 3.8.3: **FAIL**. Every drafted derivation reproduced
digit for digit, and the rule around them did not survive: two soundness breaks
(the self-discharging base, the signed division witness), the `[ENT-1]`
monotonicity theorem still false through the two caps, three determinism holes,
`[IND-8]`'s undefined minimum, and two traces named in 2.4 that the file never
drafted. 3.8.2, 3.9.1, 3.9.2, 3.9.4, 3.9.5, 3.9.7, 2.4 and 4.4 carry the repairs.

F-I1b re-executed the same traces against **F-I1's repair** and attacked it with
two new programs: **FAIL** again. Both soundness repairs held and every trace
reproduced, but the `[ENT-1]` theorem was still false - the first repair fixed
the slot list at a fixed visit set and left the visit set itself prover-dependent
(3.9.1's `N1` and `N2`). `[IND-4]`'s **shape rule** is the second repair; 3.8.2,
3.8.4, 3.9.1, 3.9.5, 3.9.7, 2.4, 4.4, 4.5.2, 9 (D1), 10 (Q3) and 11.5 carry it
and the ten wording corrections that came with it.

F-I1c re-executed everything again against **the shape rule** and attacked it
with three new programs: **FAIL**, one level further in. The visit set held under
all three attacks and `N1`/`N2` re-derived unchanged, but `[IND-7]` had imported
`[IND-3]`'s per-step magnitude hard error into certificate arithmetic while
*which steps execute* is decided by which slots the prover filled (3.9.1's `N3`),
which is a monotonicity break on one reading and a determinism hole on another.
The third repair is a **sweep**, not a patch: 3.9.1's reachable-error table
enumerates every hard error and spec-fixed limit reachable while a `bound_stmt`
is admitted and checked and classifies each syntactic or content-reachable, so no
hard error remains inside the check. 3.8.2, 3.9.1, 3.9.7, 2.4, 9 (D1), 10 (Q3),
11.5 and 12.4 carry it and its four wording corrections.

F-I1d re-enumerated the sweep independently before reading it, attacked every
syntactic row and all five parts of *Monotonicity*, and **moved none of them** -
the class the third repair aimed at is closed. **FAIL** all the same, one level
*out*: the sweep had paid for removing `[IND-3]`'s per-step limit with a
syntactic bound, and `[IND-3]`'s scope was too narrow to carry it, because the
pairs and path conditions are shapes the rule itself writes. `N4` is the ordinary
program with a degree-8 hypothesis, and with the bound false the rule's
capability sentence let two conforming implementations decide it differently -
`[ENT-1]`'s determinism half, and `AGENTS.md`'s "compiler capability is not a
source-language rejection". The fourth repair is the **widened scope**: `[IND-3]`
limits every polynomial the shape rule produces. 3.8.2, 3.9.1, 3.9.7, 2.4, 7.2,
9 (D1), 10 (Q3), 11.5 and 12.4 carry it, with four added sweep rows, three
under-specifications and one wording item.

F-I1e took the fourth run's own diagnosis as its charter - four rounds of
instance repairs, each leaving the argument one step behind the text - and
attacked the **interaction** between the sweep's two classes instead of either
class alone. **FAIL**, twice. The widened scope was delimited by *carried into
the head frame*, so a **dropped** path condition's shape was measured on one
checker and not on another, and the deciding drop is clause (b)'s `set` refusal -
the single prover-flippable decision in the pipeline, cleared under the *old*
scope and left stale. `N5` is the ordinary program that goes from verified to
hard error under a strengthening. And the `[IND-4]` backward substitution turned
out to be the one computation in the pipeline with **no bound anywhere**: its
intermediates were unlimited, early abort is forbidden because cancellation
restores legality, and the fourth round's dead-letter argument was scoped to
certificates that do not yet exist when the pass runs (`N6`). The fifth repair
stops repairing sentences: 3.8 now states the pipeline's **four laws** -
production-time measurement, prover decisions as content-only, a derived figure
for every computation, and the statement's admission rules staying on the
statement - and `[IND-3]`, `[IND-4]`, `[IND-6]`, `[IND-7]`, the regenerated
twenty-four-row sweep and the five-part monotonicity argument are derived from
them. 3.8, 3.8.2, 3.8.4, 3.9.1, 3.9.2, 3.9.7, 2.4, 7.2, 9 (D1), 10 (Q3), 11.5 and
12.4 carry it.

What remains of B0 is the deliverable, not the experiment: a `docs/done/` record
carrying all six verdicts, the rewrites and probes, and the worked certificate
sheets. **The experiments have already paid for themselves in rule text.** F-D4
does not need re-running; F-I1 does, once, against the four laws - it has now
fired on every version of `[IND-7]` it has been pointed at, **five for five**, and
this design should not assume the sixth one is the fixed point. What is different
is that the target has changed shape. The four previous runs each attacked a rule
and found the break one rule over; the fifth attacked the *joins* and found both
of its breaks there. So the run's job is now **one experiment per law** (3.9.7's
F-I2), and the first of them is L1's: find a decision that happens after a
measurement and changes what was measured. Row hunting comes second, not first.

**B1 - the image column and its rows. Large (the biggest batch, ~2 weeks).**
Enumerate the operation table's image column row by row **and decide each row's
direction**; write `[ENT-3.S5]`, the corrected shift, `ior`/`maxor` and `*wrap`
box rows, the relaxed-operand image with its `*` side condition, the saturating
and `%`/`/`/`imin`/`imax` rows, `[ENT-6.D]`, the generalised `[ENT-3.S10]` with
its projection, provenance and enumeration sentences, and `[ENT-5.P0]`; delete
the delivery block, `[GIVE-1]`'s carrier judgment and 3009. Compiler: a new
`semantic/entailment/image.rs` (~700 lines, table-driven), `flow.rs` loses three
delivery functions and gains the value-commit dispatch, `state.rs` gains image
application and the `[ENT-5.P0]` order. Gate additions: 7.4's tests (a) and (b);
one conformance case per nonempty image row family. **Its own falsifier is F-R2
and it must be run row by row.**

**B2 - retention and the exit rule. Medium-large (~1.5 weeks).**
Write `[ENT-5.R1]`-`[ENT-5.R9]` including the ladder and the universe iteration,
and `[ENT-5.X]`. Compiler: a fixed-point driver around the existing transfer with
a candidate set - **the transfer itself does not change** - plus the ladder
computation. Migration: the nine corpus sites 4.5.2 routes through retention.
Gate additions: cases 20-24 of 7.2. **Its falsifiers are F-L1 through F-L4, and
F-L2's cost measurement gates whether the ladder needs a spec-fixed round cap.**

**B3 - the induction statement. Medium (~1 week).**
Write `[IND-1]` through `[IND-10]`, the grammar production and the `[ENT-2]`
cross-type amendment. Compiler: a new module - a polynomial type, a normalizer, a
path enumerator, the substitution with its frame discipline, and the certificate
check. **Neither touches the DBM.** Gate additions: cases 25-29, plus F-I3's
head-truth assertion in a debug mode. **This is the batch with a soundness bill**
- `[IND-6]`'s frame and `[IND-4]`'s substitution are where two independent
memory-unsafety admissions lived - and F-I3 is the assertion that catches a frame
error nobody thought of.

**B4 - the contract system. Medium (~1 week).**
Write `[FN-8]`'s contract surface, `[FN-9.E1]`, `[FN-9.E2]`, `[FN-9.E4.a/b/c]`,
`[GRAM-9.C1]`, `[FN-10]` with its five admission clauses, `[DIAG-2.E1]`, and
`[FN-9.L1]`. Compiler: `[FN-10]`'s establishment needs a resolved place for a
write actual's projection at a call site - machinery `[OWN-7]` already has - and a
second RelationTemplate kind; the selected-return walk, the per-view aggregates
and the SCC publication are reused unchanged. Three deletions offset it:
`propagated` from one exclusion list, `projected` from another, and the arm-set
route's destination-identity test. File the two compiler defects (the projected
`contract_define`, the `n - n <= 0` rendering) separately and now, because both
are wrong today and neither needs a spec change.

**B5 - the deletion, the diagnostics, and the corpus pass. Medium (~1.5 weeks).**
Delete `[CLM-1..3]`, `[DIAG-3]`, `[TRAP-1]`, the `traps` category, S3, the U
view, `[PRV-2]`'s direct demand, and the claim-authority block; write the
`[SCOPE-4]`, `[ENT-1]`, `[ERR-4]`, `[SCOPE-2]`, `[PAR-*]` and `[QUAL-3]`
replacements; write `[DIAG-1]`'s payload, the seven-token classifier, the fix
table and the review note; take `docs/constitution.md`'s T3 and W3 and
`docs/patterns.md`'s five entries; rewrite the 135 corpus claims, the 63
conformance cases and the 13 codegen fixtures; delete the 20 cases with their
record. Compiler: `claim_locality.rs` (2122 lines) and the CLM passes are
deleted, `ViewStates` loses a view, and the `Full-minus` scratch discipline goes.
**This batch is the widest, shallowest diff in the project and it is where the
approval record is written.**

**B6 - measurement, after the merge. Small.**
Run F-I5 (the blind-writer trial on the `bound` notation and the three-tier
redirect) and the P0 measurement the deletion predicts: the staged permission
judgment loses a read footprint and a non-continuing edge per former claim site
(3.13), and `docs/patterns.md` P8's retired `wc` number is the only comparable
figure in the tree and is **historical**. A fresh measurement on one real loop is
what would let this design claim a P0 gain in its own right.

**What is deliberately not in the plan.** No octagon, no polyhedra, no widening
operator, no quantifier, no denotation column, no `rev`, no `by k`, no
congruence, and no restoration of a shape source. Each is priced in 4.4 and each
is declined with a reason.

---

## 9. Flagged decisions for the owner

Nine. Each is a place where I made a decision the design needed and the owner may
reverse. **Every recommendation below is ADOPTED IN THIS DESIGN AND FLAGGED FOR
THE OWNER. None of them is owner-approved, and nothing here records approval of
anything.** Each states the alternative and what reversing costs.

**D1 - is `[IND-7]`'s certificate check "search-free"?**
`[ENT-1]`'s law is quoted as *closed, deterministic, search-free*. `[IND-7]`
(3.9.1) decides an obligation by asking whether **some** assignment of
hypotheses to elimination terms, drawn from a syntactically fixed list of at most
32 slots and applied to at most 4 terms, drives the relaxed residual to zero -
`sum_k C(4,k)*P(32,k) = 988,161` partial injections at the caps. Literally read,
an existential over a finite space is a search.
*The case for it:* every conforming implementation computes the **identical
predicate** on identical inputs, which is the property 2836 actually demands; the
space is spec-fixed, its bounds are counts of slots and terms that `[IND-4]`'s
shape rule computes from the program's own text before any fact is derived rather
than counts of what a prover can derive, and there is no implementation-chosen
strategy anywhere in it. **And nothing inside the space raises**: by 3.9.1's
reachable-error sweep every hard error still live during the check is on a count
of the space, never on a value in it, so enumeration order cannot change the
answer and an implementation may stop at the first success - **and every
certificate is evaluable inside a bound the caps and `[IND-3]` compute, so an
implementation's resources decide nothing either**. Four falsifier runs
sharpened this: as first drafted the caps *were* prover-dependent, so the space
was not spec-fixed; the first repair made the slot list syntactic at a fixed
visit set and left the **visit set** prover-dependent, so the space still was
not; the shape rule made both counts outputs of one syntactic computation and
left a hard error on the arithmetic *inside* the space, which is a search whose
answer depended on the order it was walked; and the sweep removed that hard error
but published a bound whose premise `[IND-3]`'s scope did not carry, which let an
implementation's **capability** decide a program (`N4`). The case for it holds
against the widened text and did not hold against any of the four earlier ones.
*The case against:* it is the first place in the language where acceptance is
defined by an existential rather than by a procedure.
*The alternative and its price:* keep the drafted greedy elimination and accept
that `[ENT-1]`'s monotonicity sentence becomes **conditional** - "strengthening
preserves acceptance provided it does not change what is derivable at an
`[IND-7]` check point" - which is not a theorem, and which reintroduces the
`[CLM-2]` exception in a worse form, because the writer must re-derive a
statement against an elimination order they cannot see (2.4).
**And it was not enough a fifth time, in the join between two rules rather than
inside either.** F-I1e found that the widened scope was delimited by *carried
into the head frame*, which put a class-(a) hard error behind the pipeline's one
prover-flippable decision (`N5`), and that the `[IND-4]` substitution itself had
no bound at all, so an implementation's capability decided `N6` exactly as it had
decided `N4`. Both are fixed by **stating the pipeline's four laws and deriving
the rules from them** (3.8), and the owner should read this decision's
determinism case as resting on all four: L1 makes every measurement
prover-independent, L2 confines prover decisions to content, L3 gives every
computation - including the pass, which had none - a figure that makes the
capability sentence dead letter, and L4 keeps the statement's admission rules off
ordinary branch conditions.

**ADOPTED AND FLAGGED: the certificate form.** It is the only repair found that
keeps both laws, and it accepts a superset of the greedy rule at a fixed
hypothesis list so no worked trace is lost. The owner should know that the form
alone was not enough, three times: F-I1 found the theorem still false through the
caps, F-I1b found the syntactic-slot repair still false through the visit set,
and F-I1c found the shape rule still false through the certificate's arithmetic.
`[IND-4]`'s shape rule, 3.9.1's reachable-error sweep **and 3.8's four laws** are
together what make the "both laws" claim true, and 3.9.1 works the six programs
it is answerable to. The reversal price is unchanged, and the
owner should know two more things. It was not enough a third time: `N3`
showed a hard error surviving *inside* the space, which broke the theorem on one
reading and this decision's own determinism argument on the other, and the close
is the sweep - no hard error is reachable from inside the check at all. **And it
was not enough a fourth time, on this decision's own argument rather than on the
theorem.** F-I1d could not move monotonicity or a single syntactic row, but the
sweep had bought the removal of `[IND-3]`'s per-step limit with a resource bound
whose premise `[IND-3]`'s narrowed scope did not supply, and `[IND-7]` also let a
certificate an implementation *cannot evaluate* fail - so on `N4` two conforming
implementations decided differently. **The determinism case therefore rests on a
resource bound as well as on the order argument**, and the owner should read it
that way: `[IND-3]` now limits every shape the rule produces, the bound is
derived in the rule text rather than asserted, and exact evaluation inside it is
a conformance requirement rather than an implementation's choice.

**D2 - ship the local statement `[IND-10]`, or hold it?**
A23 removed three of its four stated customers: `percent_decode.wf:28,31` and
`wfgrep.wf:434` all dissolve today with a guard (`t4`, `t10`, compiled), so the
argument *"these have no `if`/`else` route, their `else` arms are unreachable"* is
refuted by a compiled sibling probe.
*What survives:* **I1's midpoint**, where the `else` arm genuinely is a lie about
the program - *what does a binary search do when its own midpoint is outside its
own window?* - and `L11_bsearch_ifelse_price.wf` (compiled) prices the branch
route at one never-taken, perfectly-predicted compare per round plus one invented
behaviour.
*The recorded dissent:* one customer is a thin basis for a statement form, and
stripped of its label and its base/step obligation the construct is a
writer-spelled predicate at an arbitrary statement position - the deleted claim
with the trap and the record removed. The owner shelved the I1 dilemma rather
than asking for it to be closed at any price.
*The restriction that answers the dissent:* `[IND-10]`'s straight-line-region
rule. Every term must be committed in the same region or live and uncommitted
across it, which is exactly what distinguishes I1's locally computed midpoint
from a claim about loop-carried values, and it also removes the need for any
depth bound on the substitution.
**ADOPTED AND FLAGGED: ship it, restricted, in the same change as the labelled
form.** *If the owner holds it:* I1 goes to `if`/`else` at `L11`'s measured price
and stays on the irreducible list; nothing else in the design moves, because the
four bucket-B claims have compiled routes and `[IND-1]`-`[IND-9]` do not depend
on it.

**D3 - is the `gap` token cross-implementation normative?**
`[DIAG-1]` 1873 fixes byte identity "only where this specification explicitly
fixes both selection and encoding", and 6.1's procedure is fully deterministic,
so it *can* be fixed.
**ADOPTED AND FLAGGED: fix it.** The token is the whole teaching channel after
the deletion, and a uniform token is what makes a future ceiling raise a grep
over diagnostics. The cost is that a second implementation must compute
dominators and preheader states **on the rejection path**, which it already has,
and which is the one path where the compiler has already decided to do no more
work.

**D4 - delete `[PRV-2]`'s `direct` demand kind, or keep it unreachable?**
**ADOPTED AND FLAGGED: delete it** (3.1.3). Keeping an unreachable case "for a
future construct" is the accreting-list method the audit struck a proposal for,
and the derivation that resurrects it is three sentences long if a future
construct ever needs one. The risk is that someone later re-adds a non-bridge
route and does not notice the demand kind is gone; case 9 of 7.2
(`prv2-neg-bridge-only`) is the guard against that, and `[IND-8.V]` is the reason
the induction statement is not itself such a route.

**D5 - keep `claim`, `because` and `deny_claims` reserved, or free them?**
**ADOPTED AND FLAGGED: keep them reserved**, on the `[FORM-3]` retired-spelling
list that already holds `trap`. Freeing them lets a program bind
`let claim = ...;` and lets an AI writer trained on v0.39 produce source that
parses into something else entirely. Reserving costs three identifiers.

**D6 - a caller-side snapshot term for `[FN-10]`?**
`[FN-10.A]` clause (d) refuses a write clause whose other operand is a place the
call disturbs, which costs the language the sentence *"I never move the cursor
backwards"* - `ensures wrote(deref(at): next): ige(next, deref(at));` - because
at the caller the formal and the written datum resolve to the same place and its
pre-transfer image is dead (A7).
*The alternative:* let `[FN-10.E]` introduce **one fresh compiler-owned snapshot
term per established relation**, bound at the call before the callee-effect kill,
in the same way `[ENT-2]` (d) already admits two compiler-owned capture terms for
a `for_stmt`, with its own kill rule.
**ADOPTED AND FLAGGED: refuse at admission for v0.40; record the snapshot term as
the widening.** The refusal is honest and diagnosable, the widening is a real new
term kind with its own kill discipline, and it should be bought when a program
wants it rather than on the strength of one example that does not work.

**D7 - buy the backward `+-wrap` rows, or leave the direction column forward?**
The audit **demoted** them ("neither necessary nor sufficient"), one part
re-promoted them to required on a factual claim `t4`/`t8`/`t10` refute, and this
design keeps them as an ordinary direction decision (3.5.6).
**ADOPTED AND FLAGGED: keep them, as part of B1's direction enumeration rather
than as a rule the batch must buy.** They are sound, they are O(1) per commit,
they reach `wfgrep.wf:553`, and the enumeration has to decide direction for every
row anyway. *If the owner declines them:* `wfgrep.wf:553` takes the restructure
Q1 recommends, and nothing else in the design moves.

**D8 - `[ENT-3.S10]`'s widening from five operations to every `[SYS-2]`
operation.**
The 0106 design widened both the operation set and the relation set in one
sentence and argued only the second (S3).
**ADOPTED AND FLAGGED: keep both widenings, with the enumeration sentence.** The
accreting-list objection applies to the operation list exactly as it applies to
the relation list, and the enumeration obligation - each `[SYS-8]` contract's
admitted projection written beside it, in the same change - is what keeps one
accreting list from becoming an unenumerated one. *If the owner declines the
operation widening:* keep the five named operations and the projection sentence;
the world-value fence and every scenario are unaffected.

**D9 - one contract rule id, or three?**
**ADOPTED AND FLAGGED: three, with the shared text factored into `[FN-8]`'s
contract surface** (3.10.6). Collapsing into one id buys legibility at the price
of every cross-reference in the specification, every `fn9-*` conformance case
name, every diagnostic a writer reads, and the append-only approval records - and
the repository's own rule is never to relocate a load-bearing path merely for
tidiness.

---

## 10. Open questions, each with a recommendation

These are questions the design does not need to answer to be coherent, unlike
section 9's decisions, which it does.

**Q1 - `wfgrep.wf:553`. CLOSED on option (b).** Its chain is
`bounded_scan + moved < bounded_scan + tail = bounded_available <= input_room`,
needing two sums. The options were (a) the backward `+-wrap` rows (D7), which do
reach this shape, and (b) restructure the source to guard `source_index` directly
against `input_room` before the read, which is P14's rewrite one more time. The
recommendation was (b), with the flag that *if the rewrite turns out to change
what `wfgrep` does, that is a finding worth more than the rule*. **It does not,
and the finding does not collect.** F-D4 closes it on two independent grounds.

*By proof.* On the guarded path `available <= input_room` and `scan <=
available`, and the loop guard gives `moved < tail` with `tail = available - scan`
and no wrap. So `source_index = scan + moved < scan + (available - scan) =
available <= input_room`. The guard's false edge is unreachable and `produced`
therefore always equals `tail`; the same argument gives `moved < input_room`.

*By execution.* 228 differential cases against the unmodified build - the test
suite's own search tree under seven patterns, the 300-level tree, every
read-boundary offset from 4056 to 4135, lines of 4095, 4096, 4097, 8192 and 12295
bytes each also unterminated, the empty and error cases, 120 randomised trees, and
an 80-entry directory past the truncation bound - byte-identical on stdout, stderr
and exit status, including a 40-buffer file with 42 matches whose output equals
host `grep -rn`. Those are the shapes that drive `shift_input_tail` at all.

Option (b) compiles in isolation (`probes/p6_shift_restructured.wf`) and in the
whole source (`rw/wfgrep.wf`). The row rule is not bought, and `:556` rides in on
the same two lines rather than on bucket P (4.5.2).

**Q2 - is the route menu's totality sentence true, or true-but-expensive?**
3.12.1 says at least one route is always open. For **I5** the guard route exists
but re-runs a validation the program already performed, and `[ENT-6]` 3172's
promise it descends from was only ever *"the goal is writable"*, not "cheaply".
*Recommendation: keep the sentence with the clause it already carries* - "where
the only open route re-establishes a fact an earlier pass established,
`[DIAG-1]` names that earlier pass" - and let the experiment test it. Weakening
it to nothing is worse: an unstated totality promise is a promise nothing checks.

**Q3 - are `[IND-3]`'s and `[IND-7]`'s caps the right ones?** Coefficient
magnitude `2^127`, degree 4, 256 monomials, plus `[IND-4]`'s 64 body paths and
`[IND-7]`'s 4 elimination terms and **32 hypothesis slots**. *Recommendation:
keep all six and measure*, **with `[IND-3]`'s three scoped to every polynomial
the shape rule produces** - the statement, the substituted obligation, and every
hypothesis shape - which is the fourth run's repair and is what `[IND-7]`'s
resource bound stands on. Every one is a spec-fixed limit with a named hard
error, which
is the legal form, and after F-I1, F-I1b and F-I1c every surviving hard error is
on a **syntactic** count, which is what makes a hard error legal at all: a limit
a prover strengthening can cross is an `[ENT-1]` break wearing a limit's clothes,
and that is exactly what the drafted `16` was, and what the first repair's slot
count still was since it counted syntactically over a visit set the prover chose,
and what `[IND-3]`'s per-step magnitude limit was once `[IND-7]` imported it into
arithmetic the prover decides whether to perform (`N3`). That last one is not a
count at all, which is why the answer was to **scope it out of the check** rather
than to make it syntactic; 3.9.1's sweep is the row-by-row statement of where
each of the six now applies, and the certificate step carries no limit and is
bounded syntactically instead. **The fourth run then showed that scoping it out
of the check is not the same as scoping it off the shapes**: with the limits
covering only the two polynomials `[IND-3]` normalizes, a rule-written pair could
carry any degree into a step and the syntactic bound was false (`N4`), so the
limits now cover every shape and stop at prover-offered content. **The fifth run
then moved the answer twice more, and both moves widen what is measured rather
than what is limited**: the three limits are tested **after every substitution
step**, not only at the end of the pass, which is what gives `[IND-4]`'s
substitution a figure at all (`N6`); and they are tested on every shape the pass
**produces**, whether or not the condition it belongs to is later dropped, which
is what keeps the one prover-flippable decision from gating a hard error (`N5`).
Neither adds a seventh limit, and neither changes a number. The measurement
question is unchanged; what changed is which polynomials, and at which moments,
the answer is about.
The slot cap is the one with the least headroom
and the one to measure first: twelve ordered-pair slots at four elimination terms leave twenty
for the statements, the path conditions and the witness commits, and the largest
obligation in this file uses eleven slots in total (I4's matching step path).
**The measurement to take is the length of a clause (b) chain.** Under the shape
rule each visited `wrap` or `sat` commit costs four slots and drags its operands
into the next commit, so a chain of `k` such commits under one statement costs
`1 + 4k` and the cap binds at `k = 8`; seven links fit and eight do not, and
3.9.1's `N2` is the nine-link program that meets it. Whether eight is enough for
a real program is what a corpus count would tell us, and nothing here does.

**Q4 - does `[ENT-5.R]`'s ladder need a spec-fixed round cap?** 3.6.4 argues not
and F-L2 is the measurement. *Recommendation: write none until F-L2 asks for
one*, and if one is ever needed make it a number in the specification, never an
implementation choice.

**Q5 - `[IND-10]`'s straight-line region: should a `match` arm count as one?**
As drafted a region ends at any branch, so a statement inside a `match` arm sees
only that arm's statements. That is right for the midpoint and might be too tight
for a decoder that computes a probe inside an arm. *Recommendation: leave it
tight and widen on a program*, because widening is monotone and narrowing is not.

**Q6 - should `[ENT-3.S9]`'s const-array element range be generalised to a
runtime-built table (V6)?** *Recommendation: no, and say why in the ceiling text*
(4.4). The ceiling clause today reads as one gap and is two: S9 already publishes
half of it, which is why `p_constarr.wf` compiles and `p_content.wf` does not. A
writer holding those two probes can falsify a ceiling clause that does not say
which half it means.

**Q7 - does `[ERR-4]` need a fourth class for the `[STOR-6]` target guard?**
*Recommendation: no.* `[PROG-3]` 1488-1490 already classifies a target failure as
"a target or environment failure ... not a source-language rejection, not a
trap", and the new `[SCOPE-4]` names it explicitly. Adding a class would put a
trusted-base condition into a source-facing vocabulary for the first time.

**Q8 - delete `[DIAG-3]`, or repurpose it for target and resource failures?**
*Recommendation: delete.* A machine-readable record for a trusted-base failure is
a different artifact with different consumers, and inheriting the claim record's
JSON shape would be the specification pretending a continuity that does not
exist. If a target-failure record is wanted it is a new rule with its own
derivation.

**Q9 - should the corpus's 132 `contract_define`s that exist only to name a
`len(...)` be inlined after `[GRAM-9.C1]`?** *Recommendation: change none of
them.* The define is still the more legible spelling when the length is used
twice, and inlining 132 of them is exactly the large structural churn no current
work needs. New contracts written after the change may inline.

**Q10 - does `requires` now need a review discipline, since it is the only place
a writer can state a premise the callee's own text cannot derive?**
*Recommendation: no.* A `requires` is discharged by the caller before entry
(`[FN-8]` 1244-1248), so it is never trusted; it **moves** an obligation rather
than creating one. The one place this could go wrong is a `requires` no caller
can satisfy, and 1257's uninhabited disposition already makes that a success with
metadata rather than a hidden hole. Worth one sentence in `docs/patterns.md`, not
a rule.

**Q11 - the staged-permission interaction with retention.** `[PAR-3]` constrains
how a drain loop's body may be arranged, so a loop rule must survive a body the
writer cannot freely restructure. Retention imposes no arrangement - it is a fact
rule over whatever graph `[FN-1]` gives - and a `bound_stmt`'s leading position
is compatible with any body. *Recommendation: no rule change, and add the
staged-write dual to F-L1's program set*, because it was not compiled and "the
arrangement does not matter" is a reading.

---

## 11. Honest limits, in red ink

### 11.1 RED INK - the impossible-else bill

**Where a fact is true and underivable, the writer writes a branch no execution
takes, and in value position invents a value for its false edge.** That is
3.12.3's problem and it is the only genuine cost of the deletion. Its three tiers
are zero (statement position, the common case), zero-to-one `break` (loop
position), and one invented value or one widened signature (value position).

The corpus bounds how often the third tier bites: of 83 claims in accepting
sources, all 18 real-program claims and 55 of 83 discharge a subscript, and a
guarded subscript is a statement in every one of them. **No corpus claim was
found whose successor is a value-position invented return.**

**F-D4 has now run that experiment on the three flagships, and the prediction
held.** Over the eleven claims in `percent_decode.wf`, `ipv4_checksum.wf` and
`wfgrep.wf`, rewritten claim-free in full and compiled at v0.39, the bill is
**8 / 3 / 0**: eight tier-zero sites (`percent_decode:16,28,31`,
`wfgrep:434,469,495`, `ipv4_checksum:19,22`), of which three cost *nothing at
all* because they are respellings that delete a test rather than add one; three
tier-one sites, each one `break` (`percent_decode:18`, `wfgrep:553,556`); and
**zero tier-three sites**. Three false edges are unreachable, and none is a dead
guard in the dishonest sense this section warns about: each has a true total
meaning on its own terms - "the output is full, stop", "only this many bytes
moved, report that" - and none returns a plausible-looking wrong value. That is
the single most load-bearing prediction in this file and it is a pass.

**And a fourth cost the tier list did not have: a guard on a `&uniq` output
parameter reads that parameter, so `[EFF-2]` widens the function's declared row,
and the widening reaches every caller that declares one.** F-D4's first rewrite
of `percent_decode:18` guarded `output_index` against `len(deref(out))` and was
rejected `[EFF-2] missing: ["reads(out)"]`, because the original reached that
length only through a `define` in the contract. The escape is to guard a term the
contract already relates to the parameter's length - here `source_length`, with
`requires ige(output_length, source_length)` carrying the rest - and **that escape
exists only for a function that already has such a contract.** A function guarding
a `&uniq` output parameter with no length relation in its contract has none, and
its successor is a signature change: tier three by another door. 6.2's `guard` row
now says so, because a mechanical fix that changes a signature is not one.

**The interaction also runs the other way, and this file did not count that
either.** A claim puts `traps` in its function's `[EFF-2]` row and `traps`
propagates to every transitive caller. Deleting the eleven claims deleted `traps`
from `decode` and `main` in `percent_decode`, from `checksum` and `main` in
`ipv4_checksum`, and from `search_file`, `walk` and `main` in `wfgrep` - three
functions whose own claim count is zero. The deletion is cheaper at the boundary
than the tier list makes it look.

Two things must be said beside it and neither is comfortable. **The dishonest
successor is accepted by the language**: an `else` that returns a
plausible-looking wrong value compiles, and it is worse than a claim was, because
a claim at least announced that the writer believed the case impossible. And the
third tier's cost is understated by "one widened signature": a result type that
becomes `Result<T, E>` **propagates a `match` to every caller**, and the drafted
claim that "there is no case where the deletion adds a runtime check that was not
there before" is true of the guarded operation and not of the callers.

### 11.2 RED INK - the image column is total by test and strong only by review

3.4's gate test checks that every operation-table row **has** an image entry.
Nothing checks that a row's entry is **the unique strongest set of `[ENT-2]`
facts its semantics entail**. A row whose published image is weaker than its
semantics entail is a defect under `[ENT-1]`, and it shows up as a writer's
`image` token where the token is wrong - a better failure mode than today's,
where the same weakness is indistinguishable from a deliberate limit, but not a
proof.

This design **made that exposure larger, not smaller**, and the honesty demands
saying so: `[ENT-3.S10]`'s enumeration sentence adds a second per-contract
review obligation on the `[SYS-8]` list, and `[ENT-3.S5.B]`'s backward rows are
the only images that are not a pure function of their operands. Two of the rows
this batch inherited were **wrong when reviewed** - the shift image published 255
where 254 is attained, and the relaxed-operand `*` row published a false fact
that a compiled program (`j03`) turns into an unsigned underflow. **Two defects
in ten rows is the measured error rate of this kind of review**, and B1's F-R2
must be run row by row rather than sampled.

### 11.3 RED INK - three things this design does not reach, and one it makes worse

**I5, the validated element property.** No term names an element, so no fact can
be about one, and neither a contract nor an induction statement can state the
property. The two answers - size the table to the value's type, or guard each use
- are a real program and a real cost respectively, and Q2 says the totality
sentence survives only with its qualifier.

**I6, the quantified structure invariant.** Deliberately unreachable, and
`[IND-3]`'s vocabulary fence says so in rule text rather than leaving a reader to
derive it from a grammar. This is the one entry on the audit's irreducible list
that rests on reading alone, and nothing here changes that.

**I7, the runtime-strided walk - and the complaint that is not answered.** The
audit's sharpest sentence about I7 is that *the family splits on whether the
count is a compile-time constant, which is not a distinction any writer would
predict*. `[ENT-3.S5.M]` answers the constant half; `[IND-4]`'s refusal of a term
divisor leaves the other half where it was; and **the unpredictable split
remains**. A writer whose stride is a literal compiles and a writer whose stride
comes from a header does not, with no rule text they can read that explains why.
That is the least defensible ceiling in the design.

**And one thing this design makes worse.** `[ENT-5.R]`'s ladder has the same
shape of unpredictability at a smaller scale: whether a loop's inductive bound is
in `K` depends on which constants the function's own text happens to mention.
`K` is syntactic and deterministic, so `[ENT-1]` is satisfied - but a writer
cannot predict from the loop alone whether retention will carry its bound, and
the answer can change when an unrelated literal is added elsewhere in the
function. The alternative (a widening ladder over the type's constants) is worse
on determinism; the honest statement is that **the ladder buys a large family at
the price of a rule the writer cannot fully predict**, and F-L1's measurement is
what would tell us whether the price is felt.

### 11.4 What left the red-ink list, and why that is a result

The 0106 design's U1 - *the laundering family is admitted and review is the only
fence* - **is gone.** There is no construct a laundering argument can inhabit:
every fact source is something the compiler observed or verified, `[IND-8.V]`
keeps a statement resting on S4 out of the blinded view, and `[ENT-3.S10]`'s
restored provenance sentence keeps an external actual from being laundered into
an internal endpoint. `F2-REVIEW-TRIAL.md` measured whether reviewers catch a
false laundering claim; it is kept beside this file as the evidence for why the
model that needed that measurement is the one being deleted.

The 0106 design's U3 - *the loop ceiling is published, and it is low* - is also
gone, replaced by `[ENT-5.R]` and `[IND]`. What replaced it has its own red ink
above and below, which is the honest trade rather than a clean win.

### 11.5 RED INK - what the falsifier runs left on the list after the repairs

The headline is that **the `[ENT-1]` theorem was false every time it was written
down**, six times: the elimination order (A4, repaired by the certificate form),
then the two caps (F-I1's F7, repaired by the syntactic slot list), then the
backward pass's visit set (F-I1b's `N1`/`N2`, repaired by `[IND-4]`'s shape
rule), then the certificate's own arithmetic (F-I1c's `N3`, where `[IND-7]` had
imported `[IND-3]`'s per-step hard error into a space in which the prover decides
which steps execute), and then the **resource bound** that removal was paid for
with (F-I1d's `N4`, where `[IND-3]`'s scope had been narrowed past the shapes the
rule itself writes - a *determinism* break rather than a monotonicity one, since
it let two conforming implementations decide one program differently), and then
the **seam between the two classes that repair created** (F-I1e's `N5`, where the
widened scope was delimited by *carried into the head frame* so that the one
prover-flippable decision decided whether an over-limit shape was measured at
all). Each repair was correct about the break in front of it and each pushed the
prover-dependence one level down - and that pattern, not any one break, was the
actual defect.

**The standing observation that closed the last four rounds is now discharged,
and this is what discharging it looked like.** Every round through the fifth
ended with the same sentence: *each repair is correct about the break in front of
it and leaves the argument one step behind the text*. Restating it a sixth time
would have been the same defect wearing a different hat, so this revision stops
repairing instances and states the pipeline's **four laws** up front (3.8), then
derives `[IND-3]`'s scope, `[IND-4]`'s substitution, `[IND-6]` clause (i),
`[IND-7]`'s caps, steps, `RELAX` and acceptance, the sweep and the five-part
monotonicity argument from them. **L1** production-time measurement, **L2** the
one prover-driven acceptance flip, **L3** a derived figure for every computation,
**L4** the statement's admission rules stay on the statement. That is a stronger
claim than any previous round made and it is falsifiable in a sharper way: F-I2's
charter is now one experiment per law rather than a hunt for the next row. It is
not a proof that the sixth break does not exist. It is the first version of this
argument in which finding one requires breaking a stated invariant rather than
noticing a sentence nobody re-read.

**The fourth repair was the first one aimed at the class instead of the instance,
and the fourth falsifier run says it worked - within its class.** The class the
pattern names is *every hard error reachable from inside the certificate check*,
and 3.9.1's
**reachable-error sweep** enumerates it: twenty-four rows covering every hard error
and spec-fixed limit reachable while a `bound_stmt` is admitted, produced and
checked, each
classified **syntactic** - a count the path's text fixes, with the reason it
cannot move written in the row - or **content-reachable**, in which case it is a
discard and not an error. F-I1d enumerated the same errors independently before
reading the table, attacked every syntactic row and all five parts of
*Monotonicity*, and **moved none of them**: the class is closed, the visit set
and both caps and the step arithmetic hold, and the four rows it added are
admission-time bookkeeping. **What it moved was one level out, not one level
down.** The sweep removed a limit and offered a syntactic bound in exchange, and
the bound rested on a scope too narrow to carry it; the removals lean on
**unstated resource and capability assumptions**, and that is where the fifth
break was. **And the fifth run showed that a class closed one rule at a time is
not the same as a pipeline closed once**: the fourth repair widened `[IND-3]` and
delimited the widened scope with a phrase - *carried into the head frame* - that
put a class-(a) hard error behind the sweep's single class-(b) decision, and left
the pass's own intermediates outside every row (`N5`, `N6`). That is the sixth
break, and it is why this revision states 3.8's four laws and derives the rules
from them rather than repairing another sentence. **A theorem six times false in
one file's lifetime is one to distrust**, and the honest reading is that a
seventh attack should be assumed to exist - but it has a name before it is found,
and this is the third round in a row where it did: **F-I2's charter is now one
experiment per law**, and the sharpest of the four is L1's - a decision that
happens after a measurement and changes what was measured. Four things around it
are worth red ink.

**The shape rule bought monotonicity with reach, and reach costs programs.** The
pass now visits every commit the path's text can reach through a hypothesis
shape, whether or not any of those hypotheses is derivable. Two programs pay for
that here: `N1` is refused at every version rather than accepted at the weak one,
because a `set` written by a call now sits inside the pass's reach, and `N2` is a
hard error at every version because nine chained `wrap` commits cost 37 slots.
Both were previously *accepted by the weakest conforming checker*, and the repair
takes that away in exchange for taking it away uniformly. That is the right
trade - a program whose acceptance depends on which checker you have is worse
than one nobody accepts - but it is a real narrowing and the slot cap is now
reachable by an ordinary-looking chain of eight `+wrap`s (Q3). **`N4` is the
third program to pay, and the newest**: with `[IND-3]`'s limits widened to every
shape the rule produces, an accumulator whose per-iteration increment is a
degree-8 expression in the statement's own terms is a hard error at admission of
that shape rather than a verified statement. The route is to hoist the increment
above the loop, and the alternative was publishing a resource bound that is a
function of the path's text rather than a spec-fixed constant. **`N5` and `N6`
are the fourth and fifth programs to pay, and between them they are the price of
L1 and L3.** `N5` is `N4` one construct over - a *branch condition* whose
substituted form reaches degree 8 on a loop carrying a `bound_stmt` - and it was
previously accepted by the weakest conforming checker, so what the round takes
away it takes away uniformly, which is the same trade `N1` and `N2` made. `N6` is
a harder bill to read: a chain of exact `*` commits whose substituted obligation
would have **cancelled** back inside the limits is refused for an intermediate it
passes through, and the writer sees a rejection whose reason is not visible in
the final polynomial. The diagnostic names the commit and the step for exactly
that reason, and the honest statement is that this is the cost of having a figure
for the pass at all: the alternative is a computation the specification declines
to bound, which is where an implementation's capability starts deciding
programs.

**The publication language is narrower than the check language, and that gap is
now a named lost program.** `[IND-7]` verifies over polynomials; `[IND-8]`
publishes unit-coefficient difference bounds. A statement can therefore be
*proved* and unable to *say* what it proved: `bound halved: ile(2_u64 * half,
len(deref(header)));` verifies by a two-line certificate and publishes only the
vacuous `Z - length <= 0`, because `half` carries coefficient 2 and `length` is
the one term the projection can reach (4.4 works it). That is why the counted
ipv4 restructure has no route (4.4), and it is a ceiling of the projection, not
of the check - which makes it the cheapest of these four to lift and the one most
likely to be asked for first.

**Two local statements in one region do not compose** (3.9.4), because the second
is checked at a region entry that precedes the first. Widening is monotone and can
wait, but a writer who states two facts about one computation and finds only the
first usable will not read that as a design.

**The certificate space is bounded and the work is not bounded by the same
argument.** 988,161 partial injections at the caps is a bound on the space; what
makes the check cheap is the skip rule, which lets an implementation evaluate only
assignments whose step is admissible. That is sound - a skipped step changes
nothing, so the omitting certificate reaches the same residual - but it means the
affordability claim rests on a pruning argument rather than on the cap, and no
measurement stands behind it. Q3 says measure the slot cap first. **The *size* of
one certificate is now derived rather than asserted** - four steps, degree 4,
`5*256` monomials, `2^640`, `s <= 2^508`, all from the caps and `[IND-3]`'s
limits - and that derivation is exactly the kind of claim F-I1d broke once
already, so it is written with its premise named rather than assumed. **The
backward pass now has a figure too, and it did not before** - one step at most
4096 monomials, degree 8, coefficients under `2^398`, over a step count the
path's text fixes - which is L3 discharged for the last computation that was
running on an assumption (`N6`). The count of certificates evaluated is still a
pruning argument, and it is now the **only** unmeasured cost claim left in the
check. That is the one to point a measurement at, and Q3 says so.

---

## 12. Probe ledger

Every row was run against the gate-profile `whitefootc` built from this tree at
spec **v0.39 ACTIVE**. Sources live under
`wf-0108-design/{trap-free-core-probes,loop-system-probes,contract-system-probes,judge1,judge2-probes,synth-probes}/`
and the audit's own probes under `wf-0107-audit/synth/probe/`. **All 232
reproduce.** Only the rows that decide something in this file are listed
individually; the rest are ledgered in the files beside this one.

### 12.1 My own probes, written for this design

| probe | serves | verdict |
| --- | --- | --- |
| `y1_entry_tight_step.wf` | **A19** | **rejects** `[FN-9]` Unproved - the step obligation of the entry-tight candidate `tap <= 0` under `tap != cap`, `cap = 8`. The entry-tight atom is **not** inductive |
| `y2_relaxed_step.wf` | **A19** | **accepts** - the identical function with the head fact weakened to `tap <= cap` and the conclusion to `result <= cap`. **The weakening is inductive**, and the pair is the compiled evidence for `[ENT-5.R2]`'s ladder |
| `y3_const_extent_loop.wf` | **A19** | **rejects** `[OP-4] residual: tap < len(taps)` - the ordinary constant-extent walk the drafted candidate rule loses |
| `y4_systemrange_guard.wf` | **A25, claim customer 4** | **accepts** - `[SYS-8]`'s `start <= end` and `end <= len(destination)` discharged by two ordinary guards over an **external** `args_count` endpoint, with `[PRV-3]` satisfied by the real branch. The one customer no part of the batch had witnessed |

### 12.2 The judge probes, re-run

| probe | serves | verdict |
| --- | --- | --- |
| `j01_wrap_total.wf` | **A1** | **accepts** - `let room = a -wrap b;` with no guard, no contract and no obligation, so a `wrap` row's `[ENT-6]` proviso is vacuous |
| `j02_wrap_nonzero_is_false.wf` | **A1** | **rejects** `[OP-4] carry < len(deref(input))` - the shape the drafted substitution would have admitted, with `carry: 9_u64` over a four-byte buffer as the out-of-bounds execution |
| `j03_poff_mul_unsound.wf` | **A3** | **rejects** `[OP-2] a -defined 10_u64` - exactly the obligation the false `a >= 10` would discharge; `harm(a: 2, b: 0)` satisfies every `requires` |
| `j1_uniq_len_entry_image.wf` | 3.10.3 | **rejects** `[FN-9]` in the callee - an `ensures` cannot cross the world boundary through an entry-image `len`, so the entry-image rule is right and `[FN-9.E1]` does not reopen it |
| `j2_fn10_err_hole_shape.wf` | **A18** | **rejects** `[OP-4] cursor < len(values)` - the obligation an `Err`-return write clause would have discharged |
| `j3_ind6_checkpoint_break.wf` | **A16** | **rejects** `[OP-4] x < len(out)` - the single obligation the false publication would discharge |
| `j3b_ind6_consumer.wf` | **A16** | **accepts** with that fact supplied as a contract. `j3`/`j3b` together are the break |
| `j4_mixed_type_compare.wf` | **A24 D-4** | **rejects** `[TYPE-5] TypeMismatch` on `ile(a_u32, b_u64)` - so `[IND-8]` would be the first producer of a cross-type difference bound, which is why `[IND-8.T]` is drafted |

### 12.3 The sibling probes this design's verdicts rest on

| probe | serves | verdict |
| --- | --- | --- |
| `t4_percent_escape_free.wf` | **A23**, 3.12.4 | **accepts** - `percent_decode`'s escape claim-free, two `+checked` equalities and one guard, no new rule |
| `t8_ipv4_parity_free.wf` | **A23**, 4.5.2 | **accepts** - the congruence residue dissolved by one pair guard whose false edge is the odd-tail case |
| `t9_residue_rejection.wf` | 6.1 | **rejects** `[OP-4] carry < len(deref(input))` - the exact diagnostic a former claim-writer sees |
| `t10_residue_repaired.wf` | **A23**, 6.2 | **accepts** - `ine(room, 0_u64)` respelled `ilt(carry, input_room)` |
| `t2`, `t14`, `t1` | 3.2, 3.12.3 | **accept** - the guard-and-exit idiom, the else-free guard, and the value-position widened signature: the three tiers of the impossible-else bill |
| `t3` / `t3b` | 3.1.2, 6.4 | **accepts** / **rejects `[CLM-2]` "redundant"** - the monotonicity asymmetry, compiled: the same derivable fact is fine as a guard and a hard error as a claim |
| `t5` | 3.1.1 | **rejects `[EFF-2]` extra: ["traps"]** - a claim-free body may not declare the category |
| `t11`, `t12`, `t13a`, `t13b` | 4.3, 3.11.2 | AllocationFit and an `[FN-8]` requirement by branches; the `ensures` route; the world-value branch and its claim-written twin |
| `c08`/`c09` vs `c23` | 3.10.1 | **reject** / **accept** on byte-identical statement sequences whose only difference is a `loop` - the diagnosis that the contract system needs no new loop mechanism |
| `c14`, `c15`, `c20` | 3.10.5 | the `&uniq` write hole, its branch repair, and the refusal at the result binding that makes `[FN-10]` a route rather than a clause |
| `c17` / `c18` | 3.10.4 | **rejects** / **accepts** - `propagate` drops the summary that `match` keeps |
| `c01`, `c03` | 3.10.3 | **reject `[GRAM-9]`** at the same parse offset - `len(result)` is unspellable and unbindable, so `[GRAM-9.C1]` is a hard prerequisite |
| `c19b`, `c24` | 3.5.1 | **reject `[FN-9]`** on `a - s <= 0` for exact `+` and for `+sat` alike - neither row publishes monotonicity today |
| `L03` vs `s16` | 3.12.4 | **rejects** / **accepts** on the same shape with a parameter offset versus a literal - `+checked` publishes its arm equality only for a **constant** offset |
| `L08`, `L09` | 3.9.3, 4.3 | **reject `[OP-2]`** with the published projection already supplied - `[ENT-6.D]` is a hard prerequisite |
| `L10a` / `L10b` | 3.6.2 | **rejects** / **accepts** - the simultaneity separating pair |
| `L11` | 3.9.4, D2 | **accepts** - I1 routed to `if`/`else`, compiling today with no rule; the price of the alternative |
| `L21` / `L22` | 3.5.4 | **accepts** / **rejects** on `igt` versus `ige` - the strictness the relaxed-operand image supplies and P-MONO does not |
| `L23` | 3.7 | **rejects**, and shows `for i in 5..3` leaves the binder at 5, refuting `binder = upper_capture` |
| `L24`, `L25` | 3.9.5 | **reject `[OP-4] hits < len(out)`** - I4 with the audit's own out-of-bounds witness repaired |
| `L12`/`L13`/`L14`, `L17`-`L20`, `L15`/`L16` | 3.6.5, 3.12.4 | the chunked cursor, the capacity-checked worklist and the variable-stride walk: each a rejecting program, its guarded price, and its induction step machine-checked as a contract |
| `r7_closure_vs_kill.wf` | 3.3 | **rejects** `best < len(data)` under today's kill-then-close order |
| `s6`, `s7` | 4.2 I5 | **accepts** / **rejects** - the fold table sized to the value's type dissolves the byte-wide half; u32 points validated `< 300` do not |
| `s11`, `s20`, `s21` | 3.5.5, 3.10.2 | the constant-subtrahend `.defined` route that already works; the factored-`requires` disjunction in both directions |
| `s22`, `s23` | 3.9.3 | **reject `[OP-2]`** - I2's and I3's consumers, the two constructed witnesses |
| `b15` / `b15b` | 3.4 | **accepts** / **rejects** on identical arithmetic - the value-commit closure is a hard prerequisite of retention |

### 12.4 The falsifier runs, and the artifacts they leave

B0's experiments produced evidence of their own, and the repairs in 3.8.2, 3.9,
4.4, 4.5.2, 6.2, 10 and 11 cite it by filename. It lives beside the run records,
not in this file. There are six runs, not two: F-D4, F-I1, F-I1b - which
re-executed F-I1's repair, confirmed both soundness repairs and every trace by
execution, and refuted the `[ENT-1]` theorem a second time - F-I1c, which
re-executed the shape rule, confirmed the visit set closed against three fresh
attacks, and refuted the theorem a third time inside the certificate's
arithmetic - F-I1d, which re-enumerated the sweep independently, **could not
move monotonicity or any syntactic row**, and refuted the theorem a fourth time
through the *determinism* half, on the resource bound row 18 offers (`N4`) - and
F-I1e, which attacked the *interaction* between the sweep's two classes rather
than either class alone, refuted monotonicity a fourth time in the seam where the
widened scope met the one prover-flippable decision (`N5`), and found the
backward substitution running with no bound anywhere in the text (`N6`).

**How this ledger counts a derivation.** One `bound_stmt` obligation is one
derivation, even where `[IND-6]` poses it on more than one body path: I4's step
is **one** derivation over two body paths, which is why 3.9 has seven and not
eight. 2.4's bullet 2 enumerates the same seven.

| artifact | serves | verdict |
| --- | --- | --- |
| `rw/percent_decode.wf` | F-D4, 4.5.2, 11.1 | **compiles**; `pd_diff.py` 691 differential cases, 0 divergences; 4 claims deleted |
| `rw/ipv4_checksum.wf` | F-D4, 2.8, 4.5.2 | **compiles**; 276 even-length cases, 0 divergences and 0 reference mismatches; 160 odd-length headers all match the RFC 1071 reference; 2 claims deleted |
| `rw/wfgrep.wf` | F-D4, 4.5.2, Q1 | **compiles**; `wfgrep_diff.py` 228 cases, 0 divergences; 5 claims deleted in three functions |
| `probes/p1`, `p2`, `p4`, `p5` | 4.5.2's score, Q1 | **reject** identically `[OP-4] residual: source_index < len(deref(input))` - there is no arithmetic route to `wfgrep:553` at v0.39 |
| `probes/p6_shift_restructured.wf` | Q1 option (b), 4.5.2 | **accepts** - the restructure in isolation |
| `probes/w_nowrite.wf`, `w_noread.wf` | 4.5.2's `:556` row | **reject** `moved < len(deref(input))` and its read twin - both `shift_input_tail` guards are load-bearing, so `:556` is not a bucket-P customer |
| `rw/percent_decode.wf`, first draft | 6.2, 11.1 | **rejects** `[EFF-2] missing: ["reads(out)"]` - the effect-row widening the mechanical fix channel would have handed the writer |
| `probes/f3_sdiv_false_bound.wf` | B2, 3.9.4, 3.9.7 | **rejects** `[FN-8] instantiated_goal: "ile(h, -3_i64)"` - today's checker refuses exactly the bound the drafted clause (d) would have proved |
| `probes/f2_sdiv_consumer.wf` | B2, 3.9.7 | **accepts** - the consumer that divides by `h + 2`, so the false bound is what buys the nonzero divisor |
| `probes/f4_sdiv_interval.wf` | B2, 3.9.7 | **rejects** `[OP-2] residual: h +defined 2_i64` - today's `/` image supplies nothing, so the witness pair was the only source |
| `L26_ipv4_counted.wf` | 4.4's V7 | **rejects** `[OP-4] residual: offset < len(deref(header))` - the counted restructure, with no statement that reaches it |
| the certificate worksheets | 3.8.3, 3.9, 3.9.7 | the six derivations of 3.9 the file then drafted, and the two refusals of 3.8.3, hand-executed digit by digit; all eight reproduce. I3's base is the seventh, added by this repair and derived in 3.9.3 rather than by F-I1. `T7-T8_refusals.md` prints A2's residual as `190125`; the figure is `255*745 = 189975`, corrected in 3.8.3, and both are positive so the refusal is unaffected |
| `N1`, `N2` | F-I1b, 3.9.1's *Monotonicity*, 11.5 | hand-derived, not compiled - the construct they attack does not exist at v0.39. Under the pre-repair text each is verified by a checker deriving fewer facts and hard-errors on one deriving more; under `[IND-4]`'s shape rule each is refused identically at both, `N1` at clause (e) and `N2` at the 32-slot cap. Both re-derived unchanged by F-I1c |
| `N3` | F-I1c, 3.9.1's *Monotonicity* (v), 11.5 | hand-derived, not compiled, same reason. Same text, same visit set, same twenty-two slots at both versions; under the shape-rule text alone it verifies on the weaker prover and crosses `[IND-3]`'s per-step magnitude limit on the stronger (`K^2 = 2.25e38 > 2^127 = 1.70e38`), or, read as an enumeration question, fires or not by the order the space is walked. Under the sweep it verifies at both by `sigma(acc) = H1` and no step raises |
| `N4` | F-I1d, 3.9.1's `N4` block, `[IND-3]`, 11.5 | hand-derived, not compiled, same reason. An ordinary program: `requires ile(x, 1_u64)` and a `*`-chained increment under `ile(hits, i)`. Its clause (b) pair rewrites to `o - hits - x^8 <= 0`, **degree 8**, which the third round's narrow `[IND-3]` scope did not limit - so `[IND-7]`'s "degree never rises ... both are at most 4" was false, and with it `5*256` and `2^640`. Under the widened scope it is a class-(a) hard error naming the statement and the degree limit, identical at every checker. Its second half is the determinism witness: on the pre-repair text implementation A verifies it and implementation B, sized to the published bound, hard-errors at `[IND-1]` |
| `N5` | F-I1e, 3.9.1's `N5` block, 3.8's L1, 11.5 | hand-derived, not compiled, same reason. `N4`'s construction moved one clause over: a **branch condition** `ilt(m3, t)` whose three clause (a) commits rewrite it to `x^8 - t + 1 <= 0`, over a `set t = t -wrap w;` whose `-wrap` side condition `w - t <= 0` needs `imin`'s row image. Under the fourth round's scope, delimited by *carried into the head frame*, the weaker checker refuses at clause (b), **drops** the condition, measures nothing and verifies by `sigma(o) = E1`, `sigma(i) = H1`, `RELAX(0) = 0`; the stronger keeps it, carries degree 8 into the frame and hard-errors. Verified to hard error under a strengthening. Under **L1** the shape is measured when the pass produces it - three clause (a) steps before the refusal is even reached - so both checkers raise row 10's hard error and the program is priced away uniformly |
| `N6` | F-I1e, 3.9.1's `N6` block, 3.8's L3, 11.5 | hand-derived, not compiled, same reason. Two parallel forty-deep `*checked` `Ok`-arm chains over `a + b`, subtracted at the end under `bound region_zero: ile(z, 0_u64);`. The **output** is `0 <= 0`, inside every limit and verified by the empty certificate; the **intermediate** is `(a+b)^(2^40)`, about `10^12` monomials with coefficients around `10^300000`. Nothing in the fourth round's text bounded it and no implementation may abort early, because cancellation restores legality - so one implementation cannot finish and one that stops early rejects what the first accepts. Under **L3**'s per-step limits it is refused within four substitution steps at every checker, and the pass's own figure - 4096 monomials, degree 8, `2^398` per step - is derived in `[IND-7]` |
| the F-I1d attacks that failed | F-I1d, 3.9.1, 11.5 | hand-derived: every sweep row classified (a) attacked for a prover-dependent count, every discard row checked in its operative sentence, and part (v)'s covering lemma attacked three ways (move a coefficient, empty a filled slot, make `RELAX` worse). **None moved.** `N1`, `N2`, `N3` and all nine drafted traces re-executed unchanged, `N3` digit for digit at both versions |
| the F-I1e re-execution | F-I1e, 3.9.7, 3.9.1 | hand-derived: all nine drafted traces - the seven derivations of 3.9 and the two refusals of 3.8.3 - and `N1` through `N4` re-executed against the final text with the per-step limits in force, and **none moves** - I1's midpoint to `floor(1/2) = 0` over ten slots, I2's step to `floor(0/255) = 0`, I3's base and step to `0`, I4's two paths to `0` and `-1` over eleven and six slots, A16 refused at `RELAX = 7`, A2 at `189975`, `B1` at `floor(9/1) = 9`. Every intermediate of every substitution step in them is inside rows 9 to 11 by orders of magnitude, so the per-step form is free on everything the file draws and costs exactly `N5` and `N6` |
| the three F-I1c attacks on the visit set | F-I1c, 3.9.1 | hand-derived: a destination reached only through an empty slot's terms, the `[IND-6]` clause (i) route with a witness-introducing path condition, and a destination occurring only in a degree-2 monomial. The first two produce identical visit sets, slot counts and verdicts at both provers; the third is the term-set/elimination-term witness worked in 3.9.1 and is a reading defect, now stated, not a monotonicity break |

`probes/f1_sdiv_trunc_break.wf` rejects earlier than intended, on the `+`
obligation, and is kept only as the negative control.

### 12.5 What the ledger establishes, and what it does not

**Established by compilation.** Every successor route in section 3.2, 3.12 and
4.3 works **at v0.39, before any rule lands**: the guard, the guard-and-exit
idiom, the `+checked` respelling, the `ensures` route, the boundary branch, the
factored-`requires` disjunction, and now the system-range guard. Every repair in
section 2 has a compiled premise or a compiled harm. The four claims section 4.5
disposes of by rewriting are compiled claim-free. And the two separating pairs
that changed rules in this file - `y1`/`y2` for the retention ladder, `j3`/`j3b`
for the induction frame - are compiled on both sides. **The three flagship
sources are compiled claim-free in full and byte-identical to their originals
over 1,195 differential cases** (12.4), which is a stronger statement than the
reduced probes it replaces.

**Not established by compilation, and offered as such.** Every DISSOLVED-PROPOSED
verdict is by construction a prediction about a rule that does not exist. The
strongest ones, marked so they can be attacked first: **retention's reach**
(eleven scenarios, nine corpus sites) rests on reading, and what is compiled is
that its *step obligations are answerable by today's checker* (`r13b`, `L14`,
`L20`, `b16`, `b17`, `y2` all accept); **`[IND-7]`'s certificate check** is hand
executed on the seven derivations of 3.9 and the two refusals of 3.8.3 and
implemented nowhere - and the hand execution is the reason the rule around those
derivations changed, since every derivation reproduced and the rule did not
survive; **`[FN-10]`** does not exist, so
its worked example is a reading; **`[ENT-3.S10]`'s projection and provenance
sentences** are arguments over quoted rule text; and **3.1.3's `[PRV-2]`
collapse** is a reading of 3389 and 3397 offered as a proof obligation a reviewer
can check.
