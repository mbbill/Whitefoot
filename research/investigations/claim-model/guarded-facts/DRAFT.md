> **STATUS: superseded evidence.** The normative content of this round-3
> draft was integrated into `../DESIGN.md` section 3.7 (commit 19440621) and
> that section is the authority for `[ENT-5.G]`. This file is kept as the
> evidence record for the three design rounds: the full hand-execution walks,
> the probe ledger, and the per-clause derivations DESIGN.md compresses.

# `[ENT-5.G]` — guarded facts: the flag-correlation rule (round 3)

Draft specification text plus its argument, its price and its falsifiers, for
integration into `research/investigations/claim-model/DESIGN.md` as a new
subsection of §3 and a bought row in §4.4's vocabulary ledger.

Baseline: the 0108 design as it stands — `[ENT-3.S1]` guard publication (§3.2),
`[ENT-3.S5]` the value-commit image closure (§3.4), `[ENT-5.P0]` the edge order
(§3.3), `[ENT-5.R]` loop retention (§3.6), `[IND]`, and the deletion of `claim`.
This round moves four of them, in the small and stated ways §4.7 records.

---

## 0. Round-3 status

### 0.1 What refuted round 2, in one paragraph each

**The soundness refutation — `c01`.** `[ENT-5.G2]`(a) and (b) formed an entry
keyed on the branch goal `G` without checking that the arms left `G` alone.
`audit2/probes/c01_arm_writes_flag.wf` (compiled, **REJECT** today with exactly
`[OP-4] residual: x < len(deref(data))`) has an `else` arm that sets the flag
true — *the fast path did not apply, so force the general path* — and round 2
accepts it and reads index 50 of a 4-byte buffer. `c01c_joined_control.wf`
(**ACCEPT**) is the same program with one literal changed 50→3, so the discharge
arithmetic is machine-checked. `c02_negative_key_arm_writes.wf` is the same break
through clause (b). `[ENT-5.G6]`'s premise — *"its complement holds on the other
reaching edge — for (a) and (b) by `[ENT-3.S1]`"* — is a category error: S1 is an
**entry** source and the premise is an **exit** property, and only clause (a′)
stated it.

**The reading defect — `b01`.** Round 2's own headline repair,
`[ENT-5.G3]`(d)'s *"put through every reaching edge's kill events"*, has two
readings that differ on memory safety. `audit2/probes/b01_carry_out_interior.wf`
(compiled, **REJECT**, same residual) is `r09` with the flag write moved off the
arm's last statement: under the literal reading the arm-exit edge carries only
scope-exit kills and the entry is carried and index 50 is read; under the
whole-arm reading it is not. Nothing in round 2 chose.

### 0.2 Why this round states laws first

Three rounds have now shipped a clause whose soundness paragraph was **restated
instead of re-derived**, and each time the result was an out-of-bounds read:
round 1's projection route (`a01`), round 1's key extension (`r09`, found by
round 2), round 2's formation clause (`c01`, found by the round-2 audit). The
pattern is the one `DESIGN.md` §3.8 hit six times in the `[IND]` cluster and
closed by *stating the invariants first, as the pipeline's own laws, with every
rule below an instance of one of them rather than a fresh local promise.*

This round does that. §2 states four laws. §3's specification text derives every
clause from them, and §2.5's map says which law each clause instances. A clause
that cannot name its law is not admitted.

### 0.3 The flagged decisions

Rounds 1 and 2 put three decisions on the owner's table (D1–D3); the round-2
audit confirmed all three and put five more (P1–P5). All eight are **taken here
per the auditor's recommendation and recorded as adopted-and-flagged**. None is
owner-approved; each is a decision this draft has taken so the design can be
worked end to end, and each is a place the owner may overrule.

| flag | source | decision taken | what overruling it would change |
| --- | --- | --- | --- |
| **D1** | audit-1 O1 | **Delete the projection release route.** Satisfaction is derivability of the key member itself | keeping it needs a `support(R_i)` clause; a memory-safety property would rest on a condition with no syntax (`a01`) |
| **D2** | audit-1 O2 | **The key is the direct goal and its support is the direct goal's support** — the binding for a binding goal, never the origin expansion's places | the expansion reading loses `layout.wf` line 139 and buys nothing once D1 is taken |
| **D3** | audit-1 O3 | **Buy the committed-flag key** `[ENT-5.G2]`(a′), with `P-S5B` carried as a prerequisite row | without it the rule handles one idiom, not the family, and flagship B discharges zero |
| **D4** | audit-2 P1 | **The arm-write test** — clauses (a) and (b) form only when neither arm disturbs `support(G)` — in its **syntactic** form, not the transplant of (a′)'s derivability condition | the transplant is equally sound and more permissive on one shape (`s03`), but it would put a derivability question into the presence of *three* keys instead of one, weakening L-G1 further than it must be |
| **D5** | audit-2 P2 | **Weaken the key-set law to monotonicity** (L-G1). "The key set is syntactic" is false as drafted — (a′) reads derivability — and determinism does not need it | claiming syntacticity again requires a syntactic (a′), which does not exist: only the arm-exit states know which arm a committed flag records |
| **D6** | audit-2 P3 | **`[ENT-5.R]`'s ladder and retention family are computed per proof view**, in this batch (`§3.14` step 2), rather than deferred to Q6 | the alternative is to qualify §4.7.3's per-view rule, which would leave a stated rule contradicted by a stated pipeline. Cost: step 2's flows run once per view — today two |
| **D7** | round 3 | **`ladder(F)`'s constant set `K` is the union of the constants of the retention-free flow with step (2′) and of the one without it** | without the union, release can *tighten* a preheader bound and delete the looser constant from `K`, removing a retention candidate that compiled before — an `[ENT-1]` 2853 break at the seam (§4.7.1) |
| **D8** | audit-2 P4 | **`[ENT-5.G8]` gains the not-formed, not-carried and same-place strings**, and the killed-entry history is **reconstructed on demand, not carried** | without them the rule's principal loss modes — refusal at formation (D4) and refusal at carry-out (L-G4) — are the least legible thing in it |

### 0.4 Everything the round-2 audit found, and where it is answered

| audit-2 finding | answer |
| --- | --- |
| **soundness (refuting):** (a)/(b) never check that the arm left the flag alone (`c01`, `c02`) | **L-G3** and the **arm-write test** in `[ENT-5.G2]`. Re-executed in §8: `c01` and `c02` are refusals, `c01c` still accepts |
| **soundness/determinism:** (d)'s "every reaching edge's kill events" has two readings (`b01`) | **L-G4**, stated over *events on paths through an arm*, never *events of an edge*, and quoted verbatim at the three sites that use it. `r09` and `b01` are re-derived side by side in §8.2 |
| **determinism/law:** the domain law is falsified by (a′), and `[ENT-5.G7]` leans on it | **L-G1**, weakened to monotone-in-the-entering-state (D5). §3.7's determinism paragraph is rewritten to rest on `[ENT-4]` 3033–3035's least closure instead |
| **proof gap:** round 2's justification of union assumes what it proves | **L-G2**, the continuous-liveness lemma, stated once and cited from union (§6.5), from carry-out (§3.3) and from the bottom argument (§3.6) |
| **specification gap:** Union can produce an inconsistent delta that is not `⊥` | `[ENT-5.G2]`'s Union gains a consistency step: **an inconsistent union is `⊥`**, with L-G2 supplying both its soundness and its monotonicity (§3.2) |
| **seam (unowned):** §3.14 step 2 computes the ladder and family once, before the per-view loop | **D6**: per view, in this batch, with the cost stated (§4.7.3) |
| **seam:** the ladder's two short disclosures | §4.7.1: a contradictory `E(@l)` contributes no constant; "enlarges", not "does not introduce"; and **D7**, a third exposure round 3 found while making F-G5 well-posed |
| **seam:** `[ENT-5.R3]`'s slot is justified by a false reason | §4.7.2: the slot is **deleted**. The head subtraction is part of the continuing-kill step, which is true by construction and adds nothing to the order |
| **auditability:** `[ENT-5.G8]` is silent in four cases | **D8**: `[ENT-5.G8]` gains three strings and answers the dead-entry question (§3.8) |
| **cost/bound:** a `⊥` delta and the killed-entry history are outside the bound | §3.7: `⊥` costs one bit; dead entries are not in the map and the diagnostic reconstructs them (§3.8) |
| **precision (unpriced):** goal identity by typed tree against support by resolved place | §9.1's new row, priced and **machine-pinned** by `s01`/`s01c` — which show the asymmetry is sharper than the audit could see without the probe (§5.2) |
| **measurement:** two mislabelled figures | §6.7 and §9.2. The formation-time sum and the point measurement are now separate numbers, and the zone cost is per point |
| **`[ENT-2]` dependency unnamed** | `[ENT-5.G1]`'s closing sentence names it (`b03`/`b03c`) |
| **P5:** run F-G1 before or after the repair | §10 and §11's Q9: **before** — implement `[ENT-5.P0]` step (2′) *and* clause (a)/(b) formation with the arm-write test, which is the smallest machine check that would have caught `c01` |

---

## 1. The rule in one sentence, and why it is not a new publisher

> At the continuation of an `if_stmt` or `value_if`, the facts an arm derived
> and the join is about to discard are not discarded: they are retained,
> attached to a signed goal **that the branch itself decides** — true at the
> continuation exactly of the executions that came by one arm — and re-admitted
> to the ordinary state on any later edge at which that signed goal is derivable
> again.

The load-bearing observation is unchanged and has survived two audits: **this
establishes no fact any rule did not already establish.** Every retained fact was
derivable in the arm-exit state under the existing sources; the ordinary
`[ENT-5]` join throws it away on *all* paths because it is false on *some*.
`[ENT-5.G]` throws it away only on the paths where the guard was false. §1's
premise-ownership law is untouched: the fact is publisher 1's (the entailment's
own output on the arm edge) and the guard is publisher 3's executed branch,
already published by `[ENT-3.S1]`. `[ENT-3]`'s "Nothing else is a fact" sentence
survives verbatim, which is why this is a retention rule — a sibling of
`[ENT-5.R]` — rather than a source.

The three emphasised words are round 3's. Round 1 said *"the signed goal
`[ENT-3.S1]` published at that arm's entry"* and round 2 said *"a signed goal
that is true exactly on that arm"* — and neither sentence was checked by the
clause underneath it. `c01` is what the gap between the sentence and the clause
costs: a goal S1 published at the arm's entry need not still record that arm at
the arm's exit, because the arm may have written it. **The property in the
sentence is now a law (L-G3) and every formation clause discharges it
explicitly.**

---

## 2. The four laws

`[ENT-5.G]`'s clauses, its proof, its determinism and monotonicity arguments,
its diagnostic and its cost bound are all instances of the four laws below.
They are stated first, and once, because the alternative has now failed three
times: a clause repaired in place leaves the argument one step behind the text,
and the argument is where the memory-safety property lives.

### 2.1 L-G1 — the key-set law

> **Which keys carry an entry at a program point is monotone in the entering
> state.** A key's presence may be decided by the syntax tree, by `[ENT-5]`'s
> kill events, and by conditions that are conjunctions of **positive
> derivability** claims about the closed states — nothing else. In particular no
> rule may make a key's presence depend on a fact's **absence**, on the
> emptiness of a delta, on a contradiction not being visible, or on any other
> quantity a stronger prover can make smaller. Only the **deltas** may depend
> freely on what the prover derives.

*What it delivers.* If `F0` and `F1` are two flows of one function and `F1`'s
entering state at every point derives everything `F0`'s does, then
`keys(F1) ⊇ keys(F0)`: syntax and kill events are identical in the two flows,
and a conjunction of positive derivability claims that fires in `F0` fires in
`F1`. That is the premise `[ENT-1]` 2853's *fact-source and closure
strengthening* class needs, and it is the whole of what monotonicity requires of
the key set.

*What it does not deliver, and what replaces it.* Rounds 1 and 2 stated this law
as *"the key set is **syntactic**"*, and `[ENT-5.G7]`'s determinism paragraph
leaned on that word. It is false: `[ENT-5.G2]`(a′) reads *"when `A` derives `+S`
and `B` derives `-S`"*, which is a derivability question, and it cannot be made
syntactic — a committed flag is recorded by an arm only if the arm-exit states
decide it oppositely, and only the states know that. **Determinism therefore
comes from `[ENT-4]`, not from the law.** `[ENT-4]` 3033–3035 makes derivability
the unique answer of a least closure over a fixed input, so *"`A` derives `+S`"*
is a total function of the syntax tree and the closed states `[ENT-3]`,
`[ENT-4]` and `[ENT-5]` already fix; two conforming implementations that agree
on those states agree on the key set. Syntacticity was a stronger claim than the
design can keep and a weaker guarantee than the design needs.

*The shape the law preserves.* Presence is decided by **syntax plus kill events
plus monotone-up derivability conditions**, and today exactly one clause uses the
third kind. D4 is chosen to keep it that way: the arm-write test for (a) and (b)
is syntactic, where the transplant of (a′)'s condition would have put a
derivability question into the presence of every key in the rule.

### 2.2 L-G2 — the continuous-liveness lemma

> Let `ε` be an entry with key `K = [s_1 … s_k]`, let `π` be a path from `P1` to
> `P2`, and suppose `ε` is present at every point of `π` — inherited at every
> merge on it by `[ENT-5.G3]`(c) or (d), and killed at no point of it. Then no
> `[ENT-5]` kill event on `π` applies to `support(s_i)` for any `i`, and
> therefore, for every execution that reaches `P2` along `π`, each `s_i`'s own
> typed expression has the same value at `P2` as at `P1`.

*Proof.* `[ENT-5.G4]` removes the whole entry on the first edge carrying an event
that applies to any key member's support, so presence throughout `π` gives the
first half. `[OWN-7]`'s overlap relation over-approximates, so an absent kill
means no place `s_i`'s own direct typed expression reads was written on `π`; by
D2 those places are exactly `support(s_i)`. An expression over unwritten places
has an unchanged value. ∎

*Three consumers, each of which the previous rounds argued without it.*

1. **Union** (`[ENT-5.G2]`). Two deltas are unioned under one key only when the
   row was inherited from the earlier formation site to the later one, which is
   exactly L-G2's premise. So the key's truth value is constant across the span,
   and a point where the key is satisfied is a point at which *both* recorded
   arms ran. §6.4 works it on `layout.wf`'s E11/E11′ pair. Round 2's
   justification — *"each was established under the same key on a path reaching
   here, so each holds wherever the key is satisfied"* — assumed its conclusion.
2. **Carry-out** (`[ENT-5.G3]`(d)). L-G4's condition (ii) is what makes the
   premise true across a branch an entry leaves on one edge; without it the
   entry is present at `P2` having survived a span on which another reaching
   edge's commit *did* touch a key member's support, and the lemma does not
   apply. `r09` and `b01` are the two witnesses.
3. **The bottom element.** An entry whose delta is `⊥`, or whose union is
   inconsistent, releases a contradiction wherever its key is satisfied. That
   point is unreachable, and here is the complete argument, which round 2 gave
   only for the formation site: let `P` be a point where the row is live and `K`
   satisfied, and let the `⊥` have entered the row at continuation `C`. The row
   was present throughout `C`→`P` (otherwise the `⊥` is not there), so by L-G2
   each key member's value is the same at `P` as at `C`. If an execution reaches
   `P`, the state at `P` is sound, so a member derivable at `P` is true at `P`,
   hence true at `C`, hence that execution came by the arm the member records
   (L-G3) — and that arm's exit state was contradictory, so no execution leaves
   it. No execution reaches `P`, whichever site the `⊥` came from. ∎

### 2.3 L-G3 — formation hygiene, and the arm-recording property

> **A clause may form an entry whose last key member is `s` only if the clause
> itself establishes that `s`'s truth *at the continuation* is decided by which
> arm the execution took.** No clause may take this property from
> `[ENT-3.S1]`: S1 establishes the branch goal at an arm's **entry**, and the
> property is about the arm's **exit**, with every kill event on the arm in
> between.

Two discharges are admitted, and each clause below names the one it uses.

- **The arm-write test (syntactic).** For an arm `X` of the branch whose
  continuation is `C` and a set of places `p`, say `X` **disturbs** `p` when some
  `[ENT-5]` kill event (a)–(d) at a node on some path from the branch node
  through `X` to `C` applies to `p`: a value commit `[SET-1, SET-2]` whose
  resolved target overlaps `p`, a call whose `[EFF-2]` projected write row
  overlaps `p`, or a scope exit killing a place of `p`. If neither arm disturbs
  `support(G)`, then `+G` reaches the then-exit and `-G` reaches the else-exit
  under `[ENT-5]`'s own transport, so `A ⊢ +G` and `B ⊢ -G` and the property
  holds. The test is decided from the arm's statement list and the `[EFF-2]` rows
  of the calls in it, without consulting the prover, which is why L-G1 keeps its
  syntax-plus-kills shape for clauses (a) and (b).
- **The arm-exit derivability condition.** `A ⊢ +S` and `B ⊢ -S` (or the
  reverse), read directly off the two closed arm-exit states. This is the only
  discharge available to clause (a′), whose whole point is that the arm *does*
  write `S`; it is a conjunction of positive derivability claims, so L-G1 admits
  it.

*The syntactic test implies the derivability one*, so `[ENT-5.G6]`'s proof needs
only the property, not the discharge, and one soundness argument covers all three
clauses. The converse fails, and that gap is D4's price: an arm that writes the
flag **without changing it on that edge** satisfies the derivability condition and
fails the syntactic test.

**The gap is narrower than it looks, and the reason is that the two discharges
interlock.** When the condition is a `Bool` **place datum** — a binding, a
parameter, a field — an arm that writes it is an arm that commits a `Bool` place,
so clause (a′) considers that very place as its `S`, and (a′) fires exactly when
the arm-exit states decide it oppositely, which is exactly the case the syntactic
test refused wrongly. `s02_redundant_flag_write.wf` is that program:
`if f { … } else { set f = no; … }` loses its `[±f]` branch keys to the arm-write
test and gets the identical `[+f] ↦ {x - Z <= 3}` back from (a′). What (a′) cannot
reach is a condition that is **not** a `Bool` place — a comparison or a compound —
whose arm writes an **operand** while preserving the condition's truth on that
edge. `s03_operand_write_preserving.wf` (compiled, REJECT) is that program, it is
the whole of what D4 costs, and §9.1 prices it. The price is paid in a
prerequisite as well as in coverage: (a)/(b) need no `[ENT-3.S5]` extension, while
(a′) needs P-S5B item 3, so `s02` moves from a prerequisite-free route to a
prerequisite-carrying one.

### 2.4 L-G4 — carry discipline

> An entry present at a branch continuation on **fewer than all** reaching edges
> is carried only when both hold:
>
> (i) some member of its key is a signed goal `[ENT-3.S1]` establishes on the
> edge `e` it survives on — the key already records that this arm ran; and
>
> (ii) it survives **every kill event on every path from the branch node through
> every other reaching arm to the continuation** — the arm-write test of L-G3,
> applied to a carried entry instead of a formed one. An entry a path through
> another arm would have disturbed at a key member's support is not carried; a
> delta fact such a path would have killed is dropped from the carried delta.
>
> Condition (ii) is over **events on paths**, never over **events of the reaching
> edge**. A commit in the middle of an arm disturbs the entry exactly as a commit
> on the arm's last statement does.

*Why the last sentence is normative and not a gloss.* `[ENT-5]` places a commit's
kill on that commit's own edge and makes scope exits edge events, so under a
literal "the reaching edge's kill events" the arm-exit edge of an `else` arm
carries only its scope-exit kills. `audit2/probes/b01_carry_out_interior.wf` is
`r09` with the flag write made interior, and the two readings give different
accepted sets — one of them reading index 50 of a 4-byte buffer. Two conforming
implementations must not be able to differ here, which is `[ENT-1]` 2835–2836,
so the disambiguating phrase is quoted verbatim at `[ENT-5.G2]`, `[ENT-5.G3]`(d)
and `[ENT-5.G6]` rather than paraphrased. This is the same instrument the round-1
audit's O2 finding required for `support(+G)`, applied to the sentence that is
round 2's own soundness repair.

*And (i) does not repeat round 2's category error, though it reads as if it
might.* `[ENT-3.S1]` establishes the arm's goal at the arm's **entry**, while `e`
is its **exit** — the exact gap L-G3 exists to close for formation. Here it is
closed by the transport rule instead of by a condition: an entry whose key member
the arm has since falsified is already dead, because falsifying `s_i` requires
writing `support(s_i)`, and `[ENT-5.G4]` removes the whole entry at that write. So
an entry that is *present* on `e` at all has had no such write, and the member S1
established at the arm's entry still holds at its exit. This is worth stating
rather than leaving to the reader: it is the same question that was answered
wrongly for (a) and (b) in round 2, and here the answer happens to be right for a
reason that is written down somewhere else.

*Why (i) alone is not enough, and why (ii) alone is not either.* Without (i) an
entry leaves an arm carrying nothing that records the arm, and the release
condition no longer implies the execution took it — round 1's key extension,
`r09`. Without (ii) the entry leaves an arm the execution never took while
another arm's commit has since made the key satisfiable — `r09` again, one level
down, and `b01`. The two conditions are the same condition at two altitudes and
the proof (§3.6) uses them in one induction step.

### 2.5 The map: which law each clause instances

| clause or argument | law | how |
| --- | --- | --- |
| `[ENT-5.G1]` domain sentence | **L-G1** | states the law and names its one derivability-conditioned clause |
| `[ENT-5.G2]`(a), (b) formation | **L-G3** | discharged by the arm-write test (D4) |
| `[ENT-5.G2]`(a′) formation | **L-G3**, **L-G1** | discharged by the arm-exit condition; the one monotone derivability condition the law admits |
| `[ENT-5.G2]` Union, and its `⊥` case | **L-G2** | soundness and the inconsistency rule |
| `[ENT-5.G3]`(c) inherited component | **L-G2** | presence on every contributing edge is what keeps the lemma's premise true through merges |
| `[ENT-5.G3]`(d) carry-out | **L-G4** | both conditions, verbatim |
| `[ENT-5.G3]` head subtraction | **L-G2** | an entry that survives a head has had no key-member kill, so its key is constant across the loop |
| `[ENT-5.G4]` support and kills | **L-G2** | the kill discipline is what makes the lemma's first half true |
| `[ENT-5.G5]` release, single route | **L-G3** | satisfaction is only meaningful because the member records an arm |
| `[ENT-5.G6]` soundness | all four | one induction: L-G3 at the formation site, L-G4 across each arm boundary, L-G2 along each span, L-G1 nowhere (it is about flows, not executions) |
| `[ENT-5.G7]` determinism | **L-G1** | monotone presence + `[ENT-4]`'s least closure |
| `[ENT-5.G7]` size | **L-G1** | keys are `chain(C) ++ [m]`, so the count is syntactic even though one presence condition is not |
| `[ENT-5.G8]` diagnostic | **L-G3**, **L-G4** | its two new strings report exactly the two refusals those laws create |

---

## 3. The drafted specification text

The family is sited in `[ENT-5]` because formation is a join rule, transport is
the kill rule, and release is a step of `[ENT-5.P0]`'s edge order. `[ENT-3]`
gains one cross-reference sentence and no new source.

### 3.1 `[ENT-5.G1]` — the guarded component

> The fact state gains one component, the **guarded component**: a finite
> partial map from **keys** to **deltas**, held per proof view `[ENT-3]` §3.14.
>
> A **key** is a nonempty ordered list of distinct signed goals `s_1 … s_k`,
> listed outermost-first. A **delta** is an L0 state over the terms live at the
> point: at most one difference bound per ordered term pair, together with a set
> of disequalities; or the bottom element `⊥`. The map holds at most one delta
> per key at any point, and holds **only live entries**: an entry a kill event
> removes is gone from the map, and nothing in the state records that it existed.
>
> Guarded entries are not L0 facts and not signed goals. `[ENT-4]` does not see
> them, no obligation, call goal or selected-return relation is judged against
> them, and no `[IND]` check reads them. They are visible to exactly four rules:
> `[ENT-5.G2]` forms them, `[ENT-5.G3]` merges and subtracts them, `[ENT-5.G4]`
> kills them, `[ENT-5.G5]` releases them.
>
> **Key-set law.** *Which keys carry an entry at a program point is monotone in
> the entering state: a flow whose closed states derive everything another's do
> carries at least the keys the other does.* A key's presence may be decided by
> the syntax tree, by `[ENT-5]`'s kill events, and by conditions that are
> conjunctions of positive derivability claims — and by nothing else. No rule
> below may make a key's presence depend on a fact's **absence**, on an empty
> delta, on a contradiction not being visible, or on any quantity a stronger
> prover can make smaller. Exactly one clause below uses a derivability
> condition: `[ENT-5.G2]`(a′), which needs it (L-G3).
>
> **Dependency.** This rule's separation of two struct instances sharing a field
> spelling, of a shadowed rebinding from its predecessor, and of two bindings of
> one comparison is entirely `[ENT-2]`'s term identity **by resolved place**
> together with `[FN-8]`'s exact-tree goal identity. `[ENT-5.G]` adds nothing to
> it and would become unsound without it.

The key-set law is the general repair for the class of hazard the round-1 audit
found in the omission clause and the round-2 audit found in the law's own
over-statement. Round 1 let three prover-dependent quantities decide whether a
key existed — an empty delta, a contradictory arm, and *"a live entry already
holds it"* — and each was a place where a stronger prover could produce a
*smaller* guarded component and therefore a program that stops compiling. Round 2
forbade the class with a word ("syntactic") its own next clause contradicted.
L-G1 forbids the class with the property that is actually needed and actually
held.

The **dependency** sentence is the round-2 audit's R7. `b03` (**REJECT**
`[FN-8]`) and `b03c` (**ACCEPT**) are its evidence: `if h2.flag` does not
discharge a requirement on `h1.flag`, and the same program tested on `h1` itself
compiles. If a future change interned place datums by field path rather than by
resolved place, this rule would silently become unsound; the sentence is there so
that whoever next edits `[ENT-2]` sees the customer.

### 3.2 `[ENT-5.G2]` — formation at a branch continuation

> Let `C` be the continuation of an `if_stmt` or `value_if` **both of whose edges
> reach it** `[ENT-5]` 3097. Let `A` and `B` be the closed states on the then and
> else (or false) edges as `[ENT-5.P0]` step (4) leaves them, and `J` the ordinary
> `[ENT-5]` join of the reaching edges.
>
> **The enclosing chain** `chain(C)` is the ordered list, outermost-first, of the
> signed goals of the branch arms syntactically enclosing `C`: `+G_i` when `C`
> lies in the then arm of an enclosing `if_stmt` or `value_if` whose condition has
> direct goal `G_i`, `-G_i` when it lies in the else arm. An enclosing branch
> whose condition has no goal origin contributes no member. A loop, a `match` arm
> and a block contribute no member.
>
> **Disturbance.** For an arm `X` of this branch and a set of places `p`, `X`
> **disturbs** `p` when some `[ENT-5]` kill event (a)–(d) at a node on some path
> from the branch node through `X` to `C` applies to `p`: a value commit
> `[SET-1, SET-2]` whose resolved target overlaps `p`, an ordinary call whose
> `[EFF-2]` projected write row overlaps `p`, or a scope exit killing a place of
> `p`. **This is every kill event on every path through the arm, and not the kill
> events of the arm's reaching edge**; a commit in the middle of an arm disturbs
> exactly as a commit on its last statement does.
>
> **The difference** `diff(X, J)`, for `X` one of `A` and `B`, is: for each
> ordered pair `(t1, t2)` of terms live at `C`, the single tightest constant `c`
> such that `X` derives `t1 - t2 <= c` and `J` derives no `t1 - t2 <= c'` with
> `c' <= c`; together with each disequality `X` derives and `J` does not. When `X`
> is contradictory, `diff(X, J)` is `⊥`. For a `value_if`, that edge's `[ENT-5]`
> delivery image after substitution is a set of relations over terms live at `C`
> and is included by the same clause; it adds no fact kind.
>
> The guarded component at `C` is `[ENT-5.G3]`(c) and (d)'s inherited component
> with the following entries **unioned** into it, in any order:
>
> (a) *branch key, positive.* When the condition has a goal origin with direct
> goal `G` **and neither arm disturbs `support(G)`**: the entry
> `chain(C) ++ [+G]` ↦ `diff(A, J)`.
>
> (b) *branch key, negative.* Under the identical condition: the entry
> `chain(C) ++ [-G]` ↦ `diff(B, J)`.
>
> (a′) *committed-flag key.* For each `Bool` place datum `S` that a value commit
> `[SET-1, SET-2]` inside either arm writes: when `A` derives `+S` and `B` derives
> `-S`, the two entries `chain(C) ++ [+S]` ↦ `diff(A, J)` and
> `chain(C) ++ [-S]` ↦ `diff(B, J)`; when `A` derives `-S` and `B` derives `+S`,
> the same two entries with the signs exchanged. In every other case, including
> when neither edge decides `S`, no entry is formed for `S`.
>
> **Union** of two deltas under one key is: per ordered pair the tighter
> constant, and the union of the disequalities; `⊥` absorbs; **and if the
> resulting set of atomic facts is contradictory — its `[ENT-4]` closure derives
> a bound and a disequality or two bounds that cannot both hold — the union is
> `⊥`.** Union is commutative, associative and idempotent, so the order of (a),
> (b), (a′) and the inherited component is not observable.
>
> When only one edge reaches `C`, no entry is formed here and `C` is an ordinary
> merge over that edge `[ENT-5.G3]`(c). An arm all of whose paths leave by
> `return`, `break`, `give` or `propagate`'s error edge does not reach `C`; that
> is `[ENT-5]` 3097's sentence, unchanged.

Four things in that text are round-3 repairs or round-2 repairs kept, and each is
load-bearing.

**Clauses (a) and (b) now discharge L-G3, and that is the refutation repair.**
The arm-write test is a conjunct of the formation condition, not a remark about
it. `c01`'s `else` arm commits `f`, so it disturbs `support(+f) = {f}`, so
neither `[+f]` nor `[-f]` forms and the later `if f` releases nothing; `c02` is
the same through (b). Note that the condition governs the **pair**: (a) and (b)
share `support(G)`, so a write in either arm refuses both. That is not
over-conservatism dressed up — for each sign, a write to `support(G)` in one arm
makes one of the two entries **unsound** (the arm that wrote is not the one the
member records) and the other **vacuous** (the member's value is now the same on
both edges, and re-establishing it later requires a write to `support(G)`, which
`[ENT-5.G4]` makes kill the entry first). The single exception is a write that
does not change the value on its own edge; where the condition is a `Bool` place
that write is a commit and clause (a′) picks the entry back up (`s02`), and where
it is not, the correlation is lost and that is D4's stated price (`s03`, §9.1).

**The key is fixed whole at formation** (round 2, kept). Round 1 formed a
one-member key and prepended members at each enclosing branch continuation. That
is deleted, and `[ENT-5.G3]`(d) carries a key out of an arm unchanged. The two
formulations agree on every release, and what the new one buys is that a key is
`chain(C) ++ [member]` for its formation continuation — so the entry count does
not depend on nesting depth, two entries share a key exactly when they share
both, and the E11/E11′ collision is the intended union (§6.4).

**Formation is unconditional in everything L-G1 forbids conditioning on.** No
"and is not contradictory", no "an empty delta is not formed", no omission
clause. A contradictory arm gives `⊥`; an empty delta is an entry that releases
nothing. Both are free: the entry bound counts keys and the fact bound counts
facts, and `⊥` is one bit, not a set of facts. The arm-write test is not an
exception to this — it reads syntax and kill events, which L-G1 admits, and it is
*monotone in the wrong direction only if the prover could remove a kill event*,
which no prover can.

**Union has a consistency step, and L-G2 is why it is both sound and monotone.**
The round-2 audit's `b04` is the shape where an inconsistent union would arise:
`h.at - Z <= 3` from one site and `Z - h.at <= -50` from another, tighter in both
directions, contradictory, and not the syntactic `⊥`. Under round 3 that program
never reaches the question — and it is L-G4's carry condition rather than
formation that stops it, because (a′) legitimately forms a second `[+h.flag]` row
there (§8.3) — but the operator must still be total. Mapping
an inconsistent union to `⊥` is **sound** by L-G2's third consumer — the point
where both were established under one live key is unreachable — and **monotone**
because `⊥` is the top of the delta order, so the flow that sees the
contradiction derives at least what the flow that does not derives. The
consistency test is a derivability question about a *delta*, which L-G1 permits
in as many words.

### 3.3 `[ENT-5.G3]` — merges, loop heads, and the head subtraction

> (c) *inherited component.* At every merge — a branch continuation, a
> `match_stmt` or `value_match` continuation, a loop exit, the delivery join of a
> `value_if` or `value_match` — the inherited guarded component holds, for each
> key present on **every** contributing edge, the **join** of that key's deltas:
> per ordered pair the weakest bound held on all of them, and the disequalities
> held on all of them, with `⊥` the identity of the join. No key is formed at a
> merge that is not a branch continuation, because no signed goal there is true
> exactly on one input.
>
> (d) *carry-out of an entry formed inside an arm.* At a **branch continuation**
> only, a key present on **exactly one** reaching edge `e` is carried unchanged
> when both:
>
> > (i) some member of the key is a signed goal `[ENT-3.S1]` establishes on `e` —
> > the key already records that this arm ran; and
> >
> > (ii) no other reaching arm **disturbs** `support(s_i)` for any key member
> > `s_i`, where *disturbs* is `[ENT-5.G2]`'s predicate: **every kill event on
> > every path from the branch node through that arm to the continuation, and not
> > the kill events of that arm's reaching edge**. A delta fact such a path would
> > have killed is dropped from the carried delta.
>
> A key present on exactly one reaching edge that fails (i) is **not** inherited;
> a key present on exactly one reaching edge that fails (ii) is **not**
> inherited; and at every other merge a key absent on some contributing edge is
> **not** inherited. Clause (d) is the one place in the rule that reads an arm on
> which an entry is absent, and it must, because a carried entry is a claim about
> the executions that took that arm too.
>
> *Loop heads.* At an ordinary or counted loop head the guarded component is the
> guarded component of the state the head rule already subtracts from — the state
> before the loop, or the closed post-capture state — with (i) every entry
> **whose key has a member whose support** a continuing kill event of that loop
> may kill removed entirely, and (ii) from each surviving entry, **every delta
> fact whose own support** such an event may kill removed. The predicate is
> `[ENT-5]`'s unchanged continuing-kill predicate; only the granularity is
> stated, and it is the granularity the ordinary loop rule already uses — a key
> is what makes a delta mean anything, so it takes the whole entry with it; a
> delta fact is an ordinary fact and dies alone.
>
> A guarded **entry** is not an `[ENT-5.R]` retention candidate: `C(@l)` ranges
> over atomic facts and is not widened by this rule. A **released** fact is an
> ordinary atomic fact and is a candidate like any other; see `[ENT-5.G5]`.

**Clause (d) is L-G4 written out, and the wording of (ii) is normative.** Round 1
key-extended a singly-present entry and applied only that edge's kills;
`probes/r09_carry_out_witness.wf` (compiled, **REJECT** today with exactly
`[OP-4] x < len(deref(data))`) is the program that accepts under and reads index
50 of a 4-byte buffer. Round 2 repaired it with *"every reaching edge's kill
events"*, and `audit2/probes/b01_carry_out_interior.wf` (compiled, **REJECT**,
same residual) is the same program with the flag write made interior, which that
phrase decides differently under its two readings. §8.2 derives `r09` and `b01`
side by side under the one reading; they land identically, which is the point.

**The head subtraction's granularity** is round 2's and is kept: a loop that
writes one delta term should cost that fact, not the correlation. L-G2 is what
makes the surviving entry meaningful across the loop — an entry that survives the
head has had no kill of a key member's support, so its key's truth is constant
over every iteration.

### 3.4 `[ENT-5.G4]` — support and kills

> **The support of a key member is the support `[ENT-5]` 3040–3041 already gives
> that signed goal: the resolved places its own direct typed expression reads —
> for a binding goal, the binding — and never the places of its origin
> expansion.**
>
> The support of a delta fact is its ordinary `[ENT-5]` L0 support. The support of
> an entry is the union of both.
>
> Kill events (a)–(d) apply to guarded entries with no exception and on the same
> edges: a delta fact dies exactly when the same event would kill it as an
> ordinary fact, and the whole entry dies when an event kills the support of any
> key member. Scope-exit kills (c) and (d) apply on every edge leaving the scope,
> before any join at that edge's target. A killed entry leaves the map and leaves
> no trace in it. An entry whose delta has become empty is **not** removed;
> emptiness is prover-dependent and the key-set law forbids it from deciding which
> keys exist.
>
> No sentence about rebinding or shadowing is needed. `[ENT-2]` makes a fresh
> binding reusing an expired spelling a distinct term and a distinct goal, so a
> shadowed flag cannot match a stored key; and kill (d) removes the entry on the
> edge leaving the old binding's scope in any case.

The support sentence is D2, quoted rather than paraphrased on purpose, at each of
the five sites that depend on it (here, §3.6, §4.4, §6.5 and §7.4). Round 1
paraphrased it as *"the union of the resolved places its complete typed
expression reads"*, which reads as the *expansion*; the round-1 audit showed the
two readings accept different programs. The specification already settles it at
`[ENT-5]` 3041 — *"A direct binding goal therefore depends on that binding, while
its separately established complete origin expansion depends on the places read
by the expansion"* — so this clause adds no definition. It says which of the two
objects a key holds: the direct goal, hence the first clause of that sentence.

This clause is also L-G2's first half. The lemma is true because a kill event
that reaches a key member's support removes the whole entry here, so an entry
that is still present has had no such event on its path.

### 3.5 `[ENT-5.G5]` — release, and `[ENT-5.P0]`'s amended edge order

> A key member `s_i` is **satisfied** in a closed state when `s_i` is derivable
> there `[ENT-4]`. **There is no second route.** A guarded entry is satisfied
> when every one of its key members is.
>
> `[ENT-5.P0]` gains one step, and the four existing steps do not move:
>
> > On every edge, in this fixed order: (1) establish every `[ENT-3]` image and
> > every `[ENT-3.S12]` or `[FN-10.E]` relation that edge's events establish,
> > each after that event's own `[ENT-5]` kill and before the next event of the
> > same edge is processed, in `[FN-1]` order; (2) take the `[ENT-4]` closure of
> > the resulting state; **(2′) guarded release: repeat — add to the state, as
> > ordinary L0 facts with their own ordinary support, the delta of every
> > satisfied guarded entry, and take the `[ENT-4]` closure — until a round adds
> > no fact the state did not already derive. Release does not remove the entry:
> > the entry must still be present on this edge for `[ENT-5.G3]`(c) to carry it
> > through the next merge;** (3) apply that edge's scope-exit kills (c) and (d);
> > (4) close again.
>
> Step (2′) runs on every edge and at no other point. It is not run at a merge or
> at a loop head, because a join and a head subtraction each derive a subset of
> every input, so neither can satisfy a key no input satisfied.
>
> **A released fact is an ordinary fact in every respect**, carrying only its own
> support. It is visible to `[ENT-4]`, to every obligation, to `[FN-8]` and
> `[FN-9]`, to `E(@l)`, to `[ENT-5.R2]`'s candidate set, and to `[IND-7]`'s
> certificate slots — exactly as any other L0 fact. Only the *entry* is invisible
> to those consumers.
>
> Step (2′) terminates in at most one round per live entry plus one, and its
> result does not depend on the order entries are visited: releasing one entry
> only enlarges the state, and enlarging the state can only satisfy more entries.

**Why the single route is sound and the projection route was not** (D1, kept). A
key member is a signed goal `+S`. Its truth at a point is the truth of `S`'s own
typed expression *at that point*, over the places `[ENT-5]` 3041 makes its
support. If those places are unwritten since the branch, `S`'s value there equals
its value at the branch — L-G2 — and by L-G3 the branch's value selected the arm.
A projection `R` is a proposition about *other* places, the ones the flag's
initializer read, and nothing in the entry protected them. `a01` is that gap:
`fits` is a snapshot of `ige(m, 8)`, `set m = 64` moves `m` without touching
`fits`, and a fresh `ige(m, 8)` at the use republishes `R` while `fits` is still
false. `[ENT-3]` 2924 already makes the origin expansion valid only while *"the
binding is no `set` target on any path from that initializer to this use, and no
`[ENT-5]` kill event applies to the replacement's support on any such path"*, and
2915(b) admits the origin of a bare `own Bool` IDENT under the same restriction —
so even an implementation that kept a projection list would find `R` unavailable
at `a01`'s use. `g13`/`g14` (compiled, REJECT) against `g15` (ACCEPT) pin that
the checker makes no such back-derivation today.

### 3.6 `[ENT-5.G6]` — soundness, normative note

> Let `ε` be an entry with key `K = [s_1 … s_k]` and delta `δ`, live and
> satisfied at a point `P`. Then every fact of `δ` still live at `P` holds of
> every execution reaching `P`; and if `δ` is `⊥`, no execution reaches `P`.
>
> *(1) The last member records an arm.* `s_k` is `±G` for the branch's own direct
> goal (clauses (a), (b)) or `±S` for a `Bool` place an arm commits (clause
> (a′)). In both cases **the forming clause established, as a condition of
> forming, that `s_k` holds at `C` exactly of the executions that came by the
> edge whose difference `δ` is**: for (a) and (b) because neither arm disturbs
> `support(G)`, so `[ENT-3.S1]`'s `+G` at the then entry and `-G` at the else
> entry survive `[ENT-5]`'s transport to the two arm exits; for (a′) because the
> clause reads `A ⊢ +S` and `B ⊢ -S` directly. This paragraph cites no rule that
> speaks about an arm's **entry** as if it spoke about its **exit**, which is the
> error round 2 shipped.
>
> *(2) The member's value is unchanged from `C` to `P`.* **The support of a key
> member is the support `[ENT-5]` 3040–3041 already gives that signed goal: the
> resolved places its own direct typed expression reads — for a binding goal, the
> binding — and never the places of its origin expansion.** `ε` live at `P` means
> no kill event on `C`→`P` reached that support, so by `[OWN-7]`'s
> over-approximating overlap no place `s_k`'s own direct typed expression reads
> was written. This is L-G2.
>
> *(3) Every path to `P` passes through a formation of `φ`.* Fix a fact `φ` of
> `δ` live at `P` and an execution reaching `P`. `φ` entered the `K`-row at some
> continuation as `diff(X, J)` for an arm edge, and thereafter the row only moves
> forward: `[ENT-5.G3]`(c) keeps a key only when it is present on **every**
> contributing edge, and (d) keeps a singly-present one only under L-G4's two
> conditions — a member recording that arm, and survival of **every kill event on
> every path from the branch node through every other reaching arm to the
> continuation**. By induction over `[FN-1]` order the execution's own path
> passes through a continuation `C'` whose arm edge `E'` derived `φ`, with no kill
> of `φ`'s support or of any key member's support on `C'`→`P` along it.
>
> *(4) Satisfaction.* `ε` satisfied at `P` means `s_k` is derivable at `P`, hence
> true at `P` (`[ENT-3]`), hence — by (2) applied to `C'` — true at `C'`, hence
> by (1) this execution came by `E'`, hence `φ` held on `E'` (`[ENT-5]`,
> unchanged). `φ` was carried from there under the ordinary kill discipline for
> its own support — the same transport an unconditionally joined fact receives.
>
> *(5) The outer members.* `s_1 … s_{k-1}` are `chain(C')`. They are not needed
> for (4), which `s_k` alone anchors; they are needed for (3), because they are
> what licenses (d) to carry the entry out of each enclosing arm. Without them an
> entry formed inside an arm is present on one edge with nothing in its key
> recording that arm, and (d) refuses it — the conservative outcome, not an
> unsound one.
>
> *(6) The bottom element.* If `δ` is `⊥` — from a contradictory arm-exit state
> or from an inconsistent union — then by (2) the key members' values are
> constant from the site that contributed the `⊥` to `P`, and by (1) a member
> derivable at `P` means the execution came by an arm whose exit state was
> contradictory, which no execution leaves. So no execution reaches `P`, and a
> contradiction released there constrains nothing.
>
> Nothing here is a new soundness obligation. The release admits no fact the
> arm-exit state did not derive; it declines a discard the join would have made.

Each of the six paragraphs names the law it uses: (1) is L-G3, (2) is L-G2, (3)
is L-G4, (4) is (1)+(2), (5) is L-G4 again, (6) is L-G2's third consumer. Clause
(a′) needs no separate proof, and that is the argument for buying it: the only
property (1) asks of `s_k` is *"true at `C` exactly of the executions that came by
this edge"*, and a committed flag whose complement the other edge derives has it
for the same reason a branch goal does.

### 3.7 `[ENT-5.G7]` — determinism, monotonicity and size

> **Determinism.** The guarded component at every point is a total function of
> the syntax tree and of the closed states `[ENT-3]`, `[ENT-4]` and `[ENT-5]`
> already fix. The key set is decided by syntax, by kill events and — for
> `[ENT-5.G2]`(a′) alone — by derivability of two signed goals in the two
> arm-exit states, which `[ENT-4]` 3033–3035 makes the unique answer of a least
> closure over a fixed input. The delta is the exact difference of two closed
> states over the ordered pairs of terms live at the continuation; union and join
> are per-pair extrema with `⊥` absorbing and a closure-decided consistency step;
> release is a monotone fixed point whose result is order-independent. There is
> no threshold, no widening, no search, no backtracking, no implementation choice
> and no spec-fixed constant. Two conforming implementations that agree on the
> closed states hold the same guarded component at every point, which is
> `[ENT-1]` 2835–2836's requirement.
>
> **Monotonicity.** For any two flows of one function in which the second's
> entering state at every point derives everything the first's does, the second
> derives everything the first does at every point after `[ENT-5.G]` as well.
> *Keys:* by the key-set law, `keys(F1) ⊇ keys(F0)` — syntax and kill events are
> identical and (a′)'s condition is a conjunction of positive derivability
> claims. *Deltas:* `diff(X, J)` together with `J` is `closure(X)` over the live
> terms, and `X` is monotone in the entering state, so a fact released in the
> weaker flow is, in the stronger flow, either in `δ` again under the same key,
> or absorbed into `J` — and a `J`-held fact is transported **strictly better**
> than a `δ`-held one, because a delta fact dies under exactly the events that
> would kill it as an ordinary fact *and* additionally when its key's support
> dies or when `[ENT-5.G3]`(c)/(d) decline to inherit. *Operators:* union and
> join are monotone in both arguments; `⊥` is the top of the delta order and is
> reached only by the stronger flow, whether by a contradictory arm or by a
> detected inconsistency; release is a monotone fixed point. Hence
> `[ENT-5.R6]`'s *"every source, kill, join and closure derives at least as much
> from a larger input state"* extends to guarded formation and release, and
> `[ENT-5.R]`'s greatest inductive family, `[ENT-5.R7]`'s deletion and
> `[IND-7]`'s certificate check are unaffected.
>
> **Size.** With `T` the terms live at a point, `N_if` the `if_stmt` and
> `value_if` nodes of the function, and `S_B` the value commits to `Bool` places
> in its arms: at most `2(N_if + S_B)` keys carry an entry at any point, and each
> delta holds at most `T(T-1)` bounds and `T(T-1)/2` disequalities — the
> `value_if` delivery relations are relations over live terms and are inside that
> count — so the guarded component holds at most `3(N_if + S_B)·T(T-1)` atomic
> facts, plus at most one key-member slot per entry per enclosing branch arm. A
> `⊥` delta is not a set of facts and contributes nothing to that count. The map
> holds no dead entries, so nothing outside it is retained; `[ENT-5.G8]`'s
> killed-entry branch is served by reconstruction (§3.8), not by state. Release
> costs at most one closure per live entry per edge.

The entry bound is a theorem, not an estimate: every key is `chain(C) ++ [m]` for
a formation continuation `C` and a member `m` drawn from `{+G, -G}` for `C`'s own
goal or `{+S, -S}` for a `Bool` place `C`'s arms commit, so summing over
continuations gives at most `2N_if + 2S_B` pairs, and coinciding keys share one
row. If one arm commits `S` twice, that is two commits and one key pair, so
counting commits over-counts and the bound still holds. **Note what the arm-write
test does to the count: nothing.** It can only refuse entries, and the bound is
an upper bound.

The determinism paragraph is the one round 2 got wrong, and the correction is
worth stating plainly. Round 2 wrote *"the key set is syntactic
(`[ENT-5.G1]`'s domain law)"* while `[ENT-5.G2]`(a′) two pages earlier read
*"when `A` derives `+S` and `B` derives `-S`"*. Determinism never needed
syntacticity: it needs the key set to be a **function** of quantities the
specification already fixes, and derivability is such a function. Monotonicity
needs something else again — that the function be **monotone** — and that is
L-G1. Conflating the two is what let a false sentence sit in the rule for a round.

### 3.8 `[ENT-5.G8]` — the correlation diagnostic

> `[DIAG-1]`'s computed gap token gains one clause, inserted between the existing
> clauses 5 and 6:
>
> ```
>   5b. O's relation is, or is over the terms of, the delta of a guarded entry
>       that (i) was killed at a point dominating O, (ii) is live at O and
>       unsatisfied, (iii) was refused at formation by [ENT-5.G2]'s arm-write
>       test, or (iv) was refused inheritance by [ENT-5.G3](c) or (d)
>                           -> ("correlation", the forming or would-be forming
>                               continuation, the responsible key member, the
>                               responsible event or arm, and the goals tested
>                               at O)
> ```
>
> The mechanical fix for `correlation` is one of six strings, chosen by which
> branch fired:
>
> - *killed*: `the branch at <node> established this relation under <key>, and
>   the write at <node> to <place> removed it; re-establish it under the same
>   condition after that write, or move the write.`
> - *unsatisfied*: `the branch at <node> established this relation under <key>;
>   test <key member> at this point to admit it.`
> - *different proposition*: `the branch at <node> established this relation
>   under <key member>; the test at this point establishes <tested goal>, which
>   is a different <ENT-2> goal — a second binding of the same comparison, a
>   place holding the flag's value, or a call result — and neither is derived
>   from the other. Test <key member> itself, or bind the condition once and test
>   that one binding at both sites.`
> - *same place, different goal*: `the branch at <node> established this relation
>   under <key member>, which names place <p>; the test at this point names the
>   same place through <written expression> and is a different <ENT-2> goal.
>   Write the test the way the branch wrote it, or bind the place once and test
>   that binding at both sites.`
> - *not formed*: `the branch at <node> would carry this relation under <key>,
>   but its <then|else> arm writes <place>, which is the condition's own support,
>   so the flag no longer records which arm ran. Move the write after the branch,
>   or test a flag the arms do not write.`
> - *not carried*: `the branch at <node> established this relation under <key>,
>   but it was not carried past <continuation>: <the key records no arm of that
>   branch | the <other> arm's write at <node> to <place> would have removed it |
>   <key> carries nothing on the edge from <node>>. Re-establish the correlation
>   after that continuation under the same condition.`
>
> *Triggers.* The *different proposition* string fires when the state at `O`
> derives a signed goal whose typed expression equals a key member's after
> replacing each ordinary-let datum by its `[ENT-3]` 2924 expansion. The *same
> place, different goal* string fires when it derives a signed goal whose
> resolved place datum equals a key member's while its typed tree does not —
> `[ENT-2]` 2886 already performs that resolution for call actuals. The *not
> formed* string fires when `O`'s relation is over a term some arm of a branch
> with a goal origin commits, and that branch's arms disturb its own
> `support(G)`; this is a syntactic search over the function and needs no
> hypothetical delta. All three are tree or place comparisons over already
> interned objects; they read no acceptance judgment and cannot change one.
>
> *The killed branch carries no state.* `[ENT-5.G1]`'s map holds only live
> entries. When `[DIAG-1]` computes this token it **re-runs the guarded flow for
> the enclosing function**, recording for each key its forming continuation and
> the event that removed it. That is one extra forward flow over one function,
> paid only where a gap token is produced, and it keeps `[ENT-5.G7]`'s size bound
> true as written.

**This ships as part of the rule, not as polish**, and the round-2 audit's
strongest process point is why: round 2 moved the rule's principal loss mode from
*killed* to *not carried*, and round 3 adds a second, *not formed* — the refusal
that makes `c01` safe. Both are invisible in the program text: `c01` differs from
an accepted program by one literal, and the reason it is refused is a `set` in an
arm the writer thinks of as the fallback. Because no acceptance judgment reads
the token (`[DIAG-1]` §6.1), an imprecise clause here is a diagnostic defect and
never an `[ENT-1]` problem, so shipping it in the same batch costs the soundness
argument nothing.

The *same place, different goal* string is new in round 3 and it has a compiled
customer: `s01`/`s01c` (§5.2). One case remains silent by construction and is
disclosed rather than repaired: Fallback N1's excluded shape (§9.2), where no
entry is formed because no arm commits.

---

## 4. The questions, answered against the drafted text

### 4.1 What may guard

**There is no syntactic stable-flag class, and writing one would still be the
design's main mistake.** The property a guard needs is not "single-assignment
let-bound comparison", it is *"the same proposition at both sites"* — and
`[ENT-5]`'s support-and-kill discipline already decides exactly that question for
every signed goal. Round 3 adds one column: what the arm-write test (L-G3) does
to each shape, which is nothing except where the arm writes the flag itself.

| flag shape | guards? | why |
| --- | --- | --- |
| `let f = ilt(i, n);` | **yes** | direct goal `f`, support `{f}`. The one release route opens at any later `if f` |
| a bare `own Bool` **parameter** | **yes** (`g06`) | a place datum, so a direct goal |
| a `Bool` **struct field**, `style.justify` | **yes** (`g07`) | a place with a field suffix is a place datum. A write to `style` or any overlapping place kills the entry, correctly; a write to a sibling field does not (`r06`) |
| a `Bool` **returned from a function** with no `ensures` | **yes** (`g10`) | its *truth* is opaque, but a guard never needs the truth — only the identity of the proposition |
| a **copy**, `let g = f; … if g` | **yes, in one direction only** (`g11` ACCEPT, **`r05` REJECT**) | testing the copy releases an `f`-keyed entry, because `[ENT-3]` 2924's expansion replaces the let datum `g` by its initializer `f`. Testing the **original** does **not** release a `g`-keyed entry: expansion never replaces a binding by something computed from it |
| a **`Bool` place the arm commits**, `set h.has_body = yes;` | **yes** (a′), **subject to §4.3.3's prerequisite** | the committed-flag key, and the one clause whose presence condition reads derivability |
| **the branch's own flag, written inside one of its arms** | **no** as a branch key (`c01`, `c02`); **yes** as a committed-flag key when the arm-exit states still decide it oppositely (`s02`) | L-G3. The arms must leave `support(G)` alone or the branch goal stops recording which arm ran — but a `Bool` place an arm commits is exactly (a′)'s subject, so the sound half comes back through the other clause. `[ENT-5.G8]`'s *not formed* string is what the writer sees when neither reaches |
| the same comparison, **two bindings** | **no** (`g13`, `g14`) | the goals differ and neither is derived from the other. `[ENT-5.G8]`'s *different proposition* string |
| the same place, **written through a borrow at one site** | **no** (`s01` REJECT, `s01c` ACCEPT) | goal identity is by exact typed tree; `deref(p).flag` and `h.flag` are one proposition over one resolved place and two goals. `[ENT-5.G8]`'s *same place, different goal* string. Priced in §9.1 |
| a `Bool` **element of a buffer or array**, `flags[i]` | **no** | a subscripted place is not a place datum, so there is no goal origin. Priced in §9.1 |
| a condition built from a **call to a partial or trapping row**, or a construction | **no** | no goal origin |
| a `match` arm on a user enum | **no** | `[ENT-2]` has no tag term and `[ENT-3]` publishes nothing about a payload's value on an arm. Priced in §9.1 |

**Conjunction and nesting.** Two different mechanisms, both free.

- *A compound condition is one goal.* `if band(f08, f06) { … }` publishes the
  single direct goal `band(f08, f06)`, so the key has **one** member. It releases
  at the identical compound test by goal identity, at `if f08 { if f06 { … } }`
  because `[ENT-4]`'s parent reconstruction closes `+f08` and `+f06` into
  `+band(f08, f06)` over the already-interned parent tree (`g08`), and from
  inside a compound test by S1's signed decomposition (`g09`). Its support is the
  union of the conjuncts' supports, so the arm-write test refuses formation if an
  arm writes **either** conjunct.
- *Nesting builds a multi-member key at formation.* An entry formed inside two
  enclosing arms is keyed `[+a, +b, +own]` from the start (`chain(C)`), and
  releases only where all three are satisfied. Nothing extends a key later.

**No cap.** Round 1 set `C_G = 3` to bound key length under incremental
extension. With keys fixed at formation the entry count is `2(N_if + S_B)`
whatever the nesting depth, so the cap bounded nothing and excluded a real shape.
Key length is bounded by the grammar's finite branch-nesting depth. Removing a
cap is a strict widening and free under `[ENT-1]`.

### 4.2 The join rule, and where the deltas come from

**Both arms, always.** The negative side carries
`if multicol { … } else { col_w = n; }`, one of flagship A's nine sites, and it
is where the *largest* deltas on that program live.

**The difference against the join, not the whole arm state.** An entry records
only what the join is about to lose.

**Which fact kinds.** Atomic L0 facts — difference bounds and disequalities —
including, for a `value_if`, the arm's delivery image relations, which after
substitution are relations over live terms and so need no separate count. Signed
goals are excluded from the delta in this draft; §9.1 prices the exclusion and
§11's Q2 recommends buying it in the same batch.

**From the closed arm state, not the raw one.** If the delta were taken before
closure, a consequence of one guarded fact and one joined fact would be lost
whenever the joined fact's support died between the join and the release, because
guarded facts are held out of `[ENT-4]` until released. Taking `δ` from
`[ENT-5.P0]` step (4)'s output stores those consequences explicitly. Flagship A's
`head_end` entry is exactly this case: the arm commits `head_end = 3` and the
useful member is the *derived* `head_end - n <= -1`, which closure with the
guard's own `n >= 4` produces.

**And after step (2′).** Because release runs on every edge including an arm's
own entry edge, a fact released *inside* an arm is in `A` and may be in the new
entry's delta. Flagship A does this at line 74 (§6.2). This is sound — the deeper
key implies the outer one — and it is why §6's trace lists released facts among
the deltas.

**And after step (2), before step (3).** The arm-local `let t = n -wrap 8_u64;
set tail_at = t;` spelling puts the image on a binding that dies at the arm's
scope exit. `[ENT-5.P0]`'s order closes before the scope kill, so
`tail_at - n <= -8` is in the closed state when `t` dies and is in `δ`. 0108 §3.3
is a hard prerequisite, not a neighbour.

### 4.3 The record of flags

This is D3, and the shape is: a function computes a comparison, **stores the
result into a `Bool` field of a record**, stores the correlated offset into a
sibling field, and later **re-tests the field**. `audit/probes/a05_record_inline.wf`
is that program; round 1 discharged none of its four residuals, and `a07`
(REJECT) against `a08` (ACCEPT) isolates why: `if h.flag` publishes the opaque
signed goal `+h.flag` and nothing numeric, so a key on the *comparison* is
satisfied by neither route.

#### 4.3.1 What the store publishes, and what the re-test releases

Nothing at the store is a guarded fact. The store publishes an ordinary
`[ENT-3.S5]` image, and the **continuation** forms the key.

```whitefoot
  let h = Head(has_body: no, body_at: 0_u64, …);   // S5 field images: -h.has_body, h.body_at = 0
  let room4 = ige(n, 4_u64);
  if room4 {
    set h.has_body = yes;                          // S5 Bool commit image: +h.has_body
    let b = n -wrap 4_u64;
    set h.body_at = b;                             // S5 field image:       h.body_at = n - 4
  }
```

At that continuation `[ENT-5.G2]` forms four entries and one of them is the
useful one:

| clause | key | delta | what it is for | L-G3 discharge |
| --- | --- | --- | --- | --- |
| (a) | `[+room4]` | `diff(A, J)` | the comparison key. Releases at a later `if room4` | arm-write test: the arm writes `h.has_body` and `h.body_at`, not `room4` ✓ |
| (b) | `[-room4]` | `diff(B, J)` | the negative comparison key | same test, same pair ✓ |
| **(a′)** | **`[+h.has_body]`** | **`diff(A, J)`** | **the committed-flag key**, which the use phase tests | arm-exit condition: `A ⊢ +h.has_body` from the commit image, `B ⊢ -h.has_body` from the construction image, unwritten on that edge ✓ |
| (a′) | `[-h.has_body]` | `diff(B, J)` | its complement | same ✓ |

The two discharges in one table are the clearest statement of why L-G3 needs
both forms. The **branch** key is protected by a syntactic test the arm passes
because it writes something else. The **committed-flag** key is protected by a
derivability condition, because the arm writes exactly the place the key names —
which is the whole idiom. A rule with one discharge would either lose flagship B
(if only the syntactic test existed) or put a derivability question into every
key's presence (if only the arm-exit condition did). D4 buys the first at the
price of `s03` and keeps L-G1 as tight as the design can hold it.

The re-test releases by the ordinary single route: `if h.has_body` publishes
`+h.has_body` by `[ENT-3.S1]` — a field place datum is a direct goal, which `g07`
(ACCEPT) pins — the key is satisfied by derivability of the key member itself,
and step (2′) adds `h.body_at - n <= -4`. No new release route, no new
satisfaction condition, and `[ENT-5.G6]` applies verbatim.

#### 4.3.2 Aliasing through borrows of the struct

Four routes, each decided by rules already written and each now pinned by a
compiled probe.

| route | outcome | mechanism |
| --- | --- | --- |
| a callee takes `&uniq` of the **whole** struct and declares `writes(h.seen)` | the entry survives, correctly | `[EFF-2]` projects onto the resolved place `h.seen`, which does not overlap `h.has_body` under `[OWN-7]`. `b02` (**ACCEPT**) pins the **goal** half, `r06` (**ACCEPT**) the L0 half |
| the same callee declares `writes(h.has_body)` | kills the entry, correctly | kill (b) through the `&uniq` actual. `b02c` (**REJECT** `[FN-8]`) |
| `set deref(p).flag = …` through a borrow | kills | `[SET-1]`/`[OWN-5]`/`[OWN-6]` resolve the target to `h.flag`; kill (a) |
| the borrow **itself** | kills nothing | a borrow is neither a commit nor a projected write; the region exit kills only facts whose support contains the *holder*, and the key member's resolved place is reached without `deref` |

`b02`/`b02c` are the round-2 audit's contribution and they close the one step
round 2 asserted without a probe: the **signed-goal** half of the transport is
field-granular, not just the L0 half. §7.3's "every key survives `bump`" is now
confirmed rather than asserted.

**The snapshot case is where D2 earns its keep.** With `let flag = h.has_body;`
and key `[+flag]`, a later `set h.has_body = no;` does **not** kill the entry, and
that is right: `flag` is a snapshot of the value at the copy, and `+flag` at `P`
still says the arm ran. The converse crossing is blocked twice over — 2924's
expansion of `flag` to `h.has_body` is invalid the moment `h.has_body` is a `set`
target on the path, and `r05` (REJECT) pins that the expansion is
one-directional anyway.

**And one precision loss, now measured rather than described** (§5.2): an entry
formed under `h.flag` and a use written `deref(p).flag` through a borrow of the
same `h` are the same proposition over the same resolved place and two different
`[ENT-2]` goals, so the key does not release. `s01` (**REJECT**) against `s01c`
(**ACCEPT**) shows the asymmetry is sharper than a precision note: today's
checker resolves a call actual rooted at a borrow holder **to its referent**
(`[ENT-2]` 2886) when it instantiates a requirement, while `[ENT-3.S1]`'s branch
publication keeps the written tree. So `if deref(p).flag { … }` establishes a goal
that a requirement over `h.flag` cannot consume *today*, before `[ENT-5.G]` is in
the picture at all. §9.1 prices the row; §11's Q8 says what to do about the
asymmetry, which is not this rule's to fix.

#### 4.3.3 The `[ENT-3.S5]` prerequisite, named precisely

This route does not work on today's checker, and the missing piece is **not**
`[ENT-5.G]`. Nine probes pin the gap.

| probe | verdict | what it pins |
| --- | --- | --- |
| `a09_field_kill.wf` | REJECT | a struct **construction** publishes no per-field numeric image |
| `a10_local_literal.wf` | ACCEPT | the same literal in a plain local does carry an L0 fact — so a09 isolates the construction, not the fact |
| `a11_field_is_term.wf` | ACCEPT | a struct field **is** an `[ENT-2]` term carrying L0 bounds |
| `r01_field_false_image.wf` | **REJECT** | a `Bool` field constructed `False()` does **not** make the then edge of `if h.flag` contradictory: no per-field `Bool` image |
| `r02_local_false_image.wf` | **ACCEPT** | the same question on a **local**: `let no = False(); if no { … }` **is** contradictory today |
| `r08_local_true_control.wf` | **REJECT** | the control that makes r02 mean something: with `True()` the arm is satisfiable |
| `r04_bool_commit_image.wf` | **REJECT** | `set h.flag = yes;` publishes no signed goal for the destination |
| `r03_true_literal_goal.wf` | **REJECT** | `let yes = True();` does not make `+yes` an `[FN-8]`-dischargeable goal, even on a local |
| `g05_set_commit.wf` | REJECT | `set x = t;` on a **local** publishes no numeric image either — 0108 §3.4's own subject |

> **Prerequisite row P-S5B (`[ENT-3.S5]`, `Bool` and field destinations).**
> `[ENT-5.G2]`(a′) is admissible only over an `[ENT-3.S5]` that publishes:
>
> 1. **a per-field image from a construction** — `Head(has_body: no, body_at: 0_u64, …)`
>    establishes `-h.has_body` and `h.body_at = 0` at the construction, each with
>    the ordinary support of the constructed place's field suffix;
> 2. **a per-field image from a value commit** — `set h.body_at = b;` establishes
>    `h.body_at = b`, and `set h.has_body = yes;` establishes `+h.has_body` when
>    `+yes` is derivable at the commit and `-h.has_body` when `-yes` is;
> 3. **a signed goal for the `Bool` literal constructions** — `True()` and
>    `False()` establish `+` and `-` of the destination they initialize.
>
> Items 1 and 2's numeric half are 0108 §3.4's stated subject extended from local
> to field destinations. Item 2's `Bool` half and item 3 are new and are the
> smallest addition that makes (a′) reachable. Item 3 is separately necessary:
> `r02`/`r08` show the checker already distinguishes a local `False()` from a
> local `True()` somewhere in its reachability judgment, but `r03` shows that
> knowledge is not available as a signed goal, and (a′) needs the goal.

**P-S5B is also what makes L-G3's second discharge checkable.** Without item 1,
`B ⊢ -h.has_body` is unavailable and (a′) forms nothing at all — so the
prerequisite is not only a precision dependency but the condition under which the
clause's soundness discharge can be evaluated. That is worth saying because it
means an implementation that ships (a′) *before* P-S5B does not get a permissive
approximation; it gets no entries.

#### 4.3.4 The honest boundary: what the record route still does not reach

| stays out | why | route today |
| --- | --- | --- |
| **a flag round-tripped through an opaque call with no `ensures`** — `let h2 = classify(h);` then `if h2.wide` | no branch in *this* function established `h2.wide`, so no clause forms a key on it. Crossing a callable boundary is publisher 2's subject matter | an `[FN-9]`-verified `ensures` on the callee, or recompute in the caller |
| **a flag whose pre-branch value is not decided** — a `Bool` field of a parameter struct, set true in the arm and left alone otherwise | (a′) needs the *other* edge to derive `-S`. An unknown incoming value decides nothing | initialize it in this function, or add `else { set h.flag = no; }` — one line, and it is what the shape means |
| **a flag written on both arms to the same value** | no discrimination; (a′) forms nothing, correctly | none needed |
| **a flag held in a buffer or array element**, `if flags[i]` | a subscripted place is not a place datum | bind the element first — `let f = flags[i];` |
| **a flag or offset field written between the store and the re-test** | the entry or the fact dies with its support, correctly | re-establish under the same condition after the write, as flagship A does at 110–113 |
| **the branch flag written inside the branch's own arm** | L-G3, and `c01` is why | move the write after the continuation, or branch on a flag the arms do not write |

The first row is the real ceiling and it is the one round 1 named: a `measure()`
helper returning a record and a `render()` consuming it will not carry a
correlation across the call. That is the boundary §1's principle draws, not an
accident. The second row is the one a writer will actually hit, and it belongs in
`docs/patterns.md`: *a record flag that guards must be initialized in the function
that sets it.*

### 4.4 Kills, aliasing, and the shadowing question

The kill rule is `[ENT-5]`'s, unchanged, applied to one more kind of object.

- **A write to a delta term kills that delta fact only.** An entry holding four
  facts about three terms loses one member when one term is written.
- **A write to a key member's support kills the whole entry**, because the key is
  what makes the delta mean anything — and because L-G2 stops being true of it
  the moment such a write happens.
- **A borrow-reachable write kills correctly with no new sentence**, because a
  delta fact carries its *ordinary* `[ENT-5]` support, which already includes
  every borrow or box/arena holder its places are reached through, and kill (b)
  uses `[EFF-2]`'s boundary projection. `r06` (ACCEPT) is the field-granular
  witness and `b02c` (REJECT) its negative control.
- **Shadowing and rebinding need no rule.** `[ENT-2]` makes a fresh binding that
  reuses an expired spelling a distinct term and a distinct goal; and the edge
  leaving the old binding's scope carries kill (d).

**The aliasing case worth calling out as sound rather than lucky.** A key whose
member is a *binding* goal — flagship A's `f10_canvasfit`, defined as
`ige(room, 32_u64)` — is a snapshot proposition. **The support of a key member is
the support `[ENT-5]` 3040–3041 already gives that signed goal: the resolved
places its own direct typed expression reads — for a binding goal, the binding —
and never the places of its origin expansion.** So when the sink call kills
`room = len(deref(canvas))`, the key `+f10_canvasfit` survives, and it should: it
still names *"the room measured before the call was at least 32"*, which is
exactly the condition under which the delta was derived. Note that the same
sentence decides an L-G3 question at flagship A's line 103: the `ink` call sits
**inside** that branch's then arm, and it disturbs `deref(canvas)` — but
`support(+f10_canvasfit)` is the binding, so the arm does not disturb it and the
entry forms (§6.3).

### 4.5 `[IND-7]`, and what the entries are invisible to

- A guarded **entry** is invisible to `[IND-7]`: not an atomic fact, fills no
  certificate slot, no visit set ranges over it.
- A **released** fact is an ordinary L0 fact and therefore **does** fill and
  tighten `[IND-7]`'s certificate slots.

That is safe, and the reason is 0108 §2.4's fifth repair rather than anything in
this rule: it made the slot list, the visit set and the elimination-term list
**syntactic**, with contents supplied by the ambient prover and no hard error
reachable from inside the check — *"a slot that fills or tightens never loses a
certificate."* `[ENT-5.G]` is exactly an ambient strengthening. The N1–N6 family
of `[IND-7]` breaks does not recur, and `[IND-9]`'s ruling that an induction
statement made redundant by a stronger prover is legal keeps a newly-released
fact from being an error where `[CLM-2]` would have made one.

### 4.6 Loops — the seam, owned

**(a) Do entries reach a loop head?** Yes, by `[ENT-5.G3]`: the head's guarded
component is the pre-loop component with entries whose *key* support a continuing
kill may touch removed whole, and delta facts whose *own* support such a kill may
touch removed individually. An entry whose flag and whose delta terms the body
never writes survives to every iteration head and releases inside the body. L-G2
is what that survival means: no kill of a key member's support, so the key's truth
is the same at every head as before the loop.

**(b) Are entries `[ENT-5.R]` candidates? No — but released facts are.** `C(@l)`
ranges over atomic facts, so a guarded **entry** is not a retention candidate; a
candidate notion over (key, delta) pairs would multiply `|C|` by the entry count,
need a ladder per delta, and has no customer. But step (2′) runs on the preheader
edge like every other edge, so a **released** fact is in `E(@l)`, is an
`[ENT-5.R2]` candidate, and can be retained across the loop like any other
entry-state fact. §4.7.1 is the consequence.

What is excluded is narrow: a correlation **first established inside the body**
and needed at the *next* iteration's head. The head rule resets the component to
the pre-loop one, so such an entry is gone at the next head. It can still be
consumed inside its own iteration, and it can leave on a `break` edge — where
`[ENT-5.G3]`(c)'s presence-on-every-contributing-edge test is what makes that
sound.

**(c) Does release inside a body break `[ENT-5.R]`'s deletion argument?** No.
`[ENT-5.R8]` inducts over head visits: a retained fact must be re-derived on the
back edge using only `H0` facts and family facts. Under this design a body
derivation may additionally use a **released** fact. Every released fact comes
from an entry that is either (i) formed inside the body of the same iteration —
sound by `[ENT-5.G6]` with no cross-iteration assumption — or (ii) survived the
head subtraction, which by the head rule means no continuing kill touches its key
support, so by L-G2 its key holds at *every* head for the same reason an ordinary
surviving fact does. Neither route consults the induction hypothesis, so
`[ENT-5.R8]`'s induction and `[ENT-5.R9]`'s A1 immunity are unchanged: release
adds no constant the arm-exit state did not already hold.

One clause is added to `[ENT-5.R6]`'s monotonicity list and `[ENT-5.G7]` supplies
it. The subtle point is unchanged: the delta alone is *not* monotone (a bigger
join can absorb a delta member), but `J ∪ δ = closure(A)` is, and a member
absorbed into `J` is transported strictly better than one held in `δ`.

### 4.7 The seams, each decided

#### 4.7.1 `[ENT-5.R2]`'s constant ladder under release

`K` is *"every bound constant appearing in any `E(@l)` of the function"* plus the
goal constants plus `0` and `-1`, and 3.6.2's algorithm computes it at line 1,
once. Step (2′) runs on the preheader edge, so `E(@l)` now contains released facts
and their constants. Three things must be said, and round 2 said one of them.

**Decision (kept).** `ladder(F)` is computed from the retention-free flow **in
which step (2′) runs**, and remains computed **once** per view. `[ENT-5.R5]`'s
termination bound is `|pairs(F) × K| + 1` outer rounds and needs `K` to be one
fixed finite set; recomputing `K` per round would put a moving set inside that
product and force the termination argument to be rewritten, for a gain no program
has asked for. Walked against 3.6.2's fourteen lines with a released candidate:
line 1 computes `K` from the family-empty flow; lines 4 and 8 carry the guarded
component under §4.7.2's edit; a fact released on a preheader edge is in `E(@l)`
at line 5, its constant is in `K` from line 1, and it becomes an ordinary
`[ENT-5.R2]` candidate at line 6, deletable at line 10 like any other. **The
algorithm needs no fifteenth line.**

**Disclosure 1 — a contradictory `E(@l)` contributes no constant.** A released
`⊥` delta makes the preheader state contradictory, and a contradictory state
derives *every* bound, so *"every bound constant appearing in `E(@l)`"* would be
either the implementation's chosen representation — visible, hence an `[ENT-1]`
2835 problem — or infinite. The sentence: **a contradictory `E(@l)` contributes
no constant to `K`.** It is well-posed and it costs nothing, because a
contradictory preheader means the head is unreachable and retention there
constrains no execution (`[ENT-5.G6]` paragraph (6)).

**Disclosure 2 — release *enlarges* the round-`i` class.** Round 2 wrote that a
candidate whose constant appears only in a round-`i` state *"is `[ENT-5.R2]`'s
existing property, not something release introduces"*. Half right: release makes
the class strictly **larger**, because a delta is `diff(A, J)` over states that
themselves move between rounds, so retention at round `i` can create a delta
member with a constant no round-0 state held. The direction is safe and
deterministic; the sentence should say *enlarges*.

**Disclosure 3 (round 3, and it needed a decision) — release can also *remove* a
constant, and that direction is not safe.** `E(@l)` is a closed state, so it
holds the **tightest** bound per pair. If release tightens a preheader bound from
`t1 - t2 <= 7` to `t1 - t2 <= 3`, then `7` no longer appears in that state, and if
it appears in no other `E(@l)` it leaves `K`. The candidate `t1 - t2 <= 7` is
still derivable at the preheader, but `[ENT-5.R2]` cannot form it, so a loop that
retained `<= 7` — inductive where `<= 3` is not — retains nothing. **That is a
program that compiles with `[ENT-5.G]` off and fails with it on**: precisely the
`[ENT-1]` 2853 break F-G3 exists to catch, arriving through a seam rather than
through the rule. **D7 is the repair:** `K` is the **union** of the constants of
the retention-free flow with step (2′) and of the retention-free flow without it.
Both flows are already defined objects, the second is exactly the flow
`[ENT-5.R2]` computes today, and the union is one fixed finite set, so
`[ENT-5.R5]`'s bound is untouched. Cost: the ladder pass runs twice per view —
two forward flows, not two universe iterations (§9.2). With D7 the superset
property holds **by construction**, which is what makes F-G5's second condition
statable at all.

#### 4.7.2 The per-round recomputation, and the deleted slot

`[ENT-5.G3]` derives a loop head's guarded component from "the state before the
loop", which changes between rounds of `[ENT-5.R5]`'s universe iteration. Two
edits, and round 2's third is withdrawn.

**Edit 1 — `[ENT-5.R3]`'s fixed head order gains no slot.** Round 2 inserted the
guarded-component head subtraction between the continuing-kill subtraction and
retention, justified by *"retention must see the head's guarded component in
order for a released fact to be a candidate."* **That reason is false.**
`[ENT-5.G5]` says step (2′) is not run at a loop head, so the head's guarded
component releases nothing there, and retention's candidates come from `E(@l)` —
the closed **preheader** state, where (2′) already ran — and from `H0(@l)`.
Retention never reads the head's guarded component. The placement was
unobservable, but a fixed order justified by a false reason invites an
implementation to reason from it, and this is a rule whose determinism case is
that the order is fixed and every slot's reason is stated.

> The slot is **deleted**. `[ENT-5.R3]`'s continuing-kill subtraction step applies
> to **both** components of the state — the ordinary facts, and the guarded
> component at `[ENT-5.G3]`'s two granularities. The order gains nothing, and the
> reason is true by construction: it is one predicate applied to one state.

**Edit 2 — 3.6.2's algorithm** needs no new line; one sentence after the listing
does it:

> "Forward flow" at lines 1, 4 and 8 carries the guarded component and runs
> `[ENT-5.P0]` step (2′). The guarded component is therefore recomputed in every
> round of the outer universe iteration and of the inner deletion, from that
> round's own states, with no separate step.

Adding a fifteenth line would invite an implementation to compute the component
out of order; making it part of "forward flow" — which lines 4 and 8 already
recompute per round — is fewer words and stricter.

**Edit 3 — §3.14's pipeline step 3** gains a clause in the walk:

> ```
>          on each edge apply [ENT-5.P0]: image after its own kill, close,
>                        guarded release (2'), scope-exit kills, close again
>          at each merge take the ENT-5 join of the arm-exit states and the
>                        [ENT-5.G3](c) join of their guarded components
> ```

#### 4.7.3 One guarded component per proof view — and step 2 with it (D6)

> **The guarded component is a component of the fact state, and the fact state is
> per proof view.** §3.14 step 3 walks each function once per view (`complete`
> and `s4_blinded`); formation reads that view's closed states, release writes
> into that view's state, and **no entry, delta or release crosses views.**

Worked on flagship B, whose contract carries `requires ige(room, 1_u64)`:

- In the **complete** view the else-edge entry `[-h.wide]` carries
  `h.wide_at - Z <= 0`, and `h.wide_at < len(deref(data))` closes from that plus
  S4's `room >= 1`.
- In the **s4_blinded** view S4 publishes nothing. The entry still forms —
  `diff(B, J)` is computed in the blinded states and still holds
  `h.wide_at - Z <= 0`, which does not depend on the requirement — and it still
  releases, and the obligation is still unproved, because the *other* premise is
  missing. The blinded view reports exactly what it is for.

A shared component would not be a precision loss but a **wrong partition**: a
delta computed in the complete view holds facts derived from S4, and releasing
those into the blinded view would let it discharge an obligation the caller's
requirement is carrying, while step 8 uses `s4_blinded` for `[PRV]`'s
external-subject partition.

**And that rule is contradicted by §3.14's own step 2 unless step 2 moves.**
Step 2 computes the ladder and the `[ENT-5.R5]` retention family **once**, before
step 3's per-view loop; under Edit 2 those flows run step (2′); so one family
derived from one view's guarded releases would be installed at every loop head in
**both** views. Round 2 deferred this to Q6 as *"a seam this design inherits
rather than creates"*. That is not available: before `[ENT-5.G]` step 2's
view-ambiguity was a **precision** under-specification (S4 facts in `E(@l)`);
after it, a **stated rule** is contradicted by a **stated pipeline**, which is the
one thing `[ENT-1]` cannot absorb.

> **D6.** `§3.14` step 2 computes `ladder(F)` and the `[ENT-5.R5]` retention
> family **once per proof view**, from that view's own flows, and step 3 installs
> the family of the view it is walking.

**The price, stated.** Step 2's work multiplies by the number of views — today
two. With D7 each view computes two ladder flows, so step 2 runs four forward
flows and two universe iterations per function instead of one and one. The
universe iteration dominates, so the honest figure is **step 2 costs twice what
it did**, not four times. Nothing else in the pipeline changes: `[ENT-5.R5]`'s
termination bound applies per view with that view's own fixed `K`; determinism is
per view and each view's inputs are already fixed; and `[PRV]`'s partition
argument is *simplified*, because the blinded view's family is now derived only
from blinded facts. This is an `[ENT-5.R]` change made in a `[ENT-5.G]` batch,
which is why it is flagged (D6) rather than assumed.

### 4.8 `[ENT-1]`, monotonicity and determinism

**Acceptance only widens.** `[ENT-5.G]` adds facts to states and removes none:
formation reads two states and writes a new component, release writes into L0,
and no rule deletes an L0 fact that would otherwise have been present. By
induction over `[FN-1]` order and the monotonicity of every transfer, the closed
state at every point derives a superset of what it derives with `[ENT-5.G]`
switched off. This is the *fact-source and closure strengthening* class `[ENT-1]`
2853 already promises is safe.

Five ways that promise could break, and where each stands:

1. *A redundancy error.* Under `[CLM-2]` a newly derivable predicate was a hard
   error on every program carrying the claim. 0108 §2.10's ruling removes exactly
   that. **Without it this rule would be a monotonicity break the day it landed.**
2. *A cap that is a hard error on a prover-dependent count.* There is no cap.
   `C_G` is removed and no count in this design is prover-dependent.
3. *A displaced choice.* `[IND-7]`'s A4 break came from a *selection* whose
   result a strengthening could change. `[ENT-5.G]` selects nothing.
4. **A prover-dependent key set.** Round 1 got this wrong in three places
   (empty-delta, contradictory arm, omission clause); round 2 forbade the class
   with a word its own clause contradicted. **L-G1** is the repair that is both
   true and sufficient: presence is monotone, and the one clause that reads the
   prover reads it monotonically upward.
5. **A seam that shrinks a candidate set.** New in round 3: §4.7.1's disclosure 3
   is such a break, arriving through `[ENT-5.R2]`'s ladder rather than through
   this rule's own text. **D7** closes it. This is the one class the earlier
   rounds' monotonicity arguments could not have found, because they checked the
   rule and not the pipeline the rule feeds.

**Determinism.** Every quantity — the key set, each key's order, the delta, the
union with its consistency step, the join, the release rounds — is a total
function of the syntax tree and the closed states. No heuristic, no threshold, no
time bound, no randomisation, no spec-fixed constant.

**Facts-off compilation.** `[ENT-5.G]` is a source-acceptance judgment, identical
in facts-on and facts-off compilation. It emits no code, changes no lowering, and
there are no claims for it to elide.

---

## 5. What today's checker actually does

Every row is compiled against the `batch/0106-claim-model-design` worktree
compiler, unmodified. `./run.sh probes/*.wf`, `./audit/run.sh` and
`./audit2/run.sh` reproduce them; all three were re-run for this round and every
round-1 and round-2 verdict stands.

### 5.1 The findings the design rests on

**The isolating pair.** `g03_pure_join.wf` REJECTs with
`[OP-4] start < len(deref(data))`; `g04_fused_let.wf`, the identical arithmetic
and read fused into one arm, ACCEPTs. So S7's range-guarded `-wrap` image, the
projection of a let-bound flag and the `[ENT-4]` step are **all present today**,
and the join is the entire loss. `g04` uses `value_if` delivery, so the pair
isolates `[ENT-5.G]` with no `[ENT-3.S5]` prerequisite in the way.

**The guardable class is wider than a syntactic flag.** `g06`, `g07`, `g10`,
`g11` all ACCEPT: parameters, struct fields, call results and copies all publish
a usable signed goal today. `[ENT-5.G]` keys on whatever `[ENT-3.S1]` publishes.

**The expansion is one-directional** (`g11` ACCEPT, `r05` REJECT), **field kills
are granular on both halves** (`r06` ACCEPT for L0, `b02`/`b02c` for the goal),
**two instances are two goals** (`b03` REJECT, `b03c` ACCEPT), and **the size
maximiser is real in both directions** (`a12`, `r07` ACCEPT).

### 5.2 Round 3's three additions

**1. `s01_borrow_goal_identity.wf` REJECTs and `s01c_borrow_same_expression.wf`
ACCEPTs**, and the pair says something the round-2 audit could describe but not
measure. Both programs hold a shared borrow `let p = &'m h;` live across the same
region; they differ only in which expression names the flag where.

| program | branch condition | call actual | verdict |
| --- | --- | --- | --- |
| `s01` | `if deref(p).flag` | `needs_flag(f: h.flag, …)` | **REJECT** `[FN-8]`, `instantiated_goal: "h.flag"` |
| `s01c` | `if h.flag` | `needs_flag(f: deref(p).flag, …)` | **ACCEPT** |

The instantiated goal is `h.flag` in **both** programs: `[ENT-2]` 2886 resolves a
call actual rooted at a borrow holder to its referent datum. `[ENT-3.S1]`'s branch
publication does not. So the asymmetry is on the **publication** side, and a
condition written through a borrow establishes a goal that a requirement over the
same resolved place cannot consume — **today, before `[ENT-5.G]` exists**. For
this rule the consequence is a precision row (§9.1) and a diagnostic string
(§3.8); for `[ENT-2]`/`[ENT-3]` it is a defect report (§11's Q8).

**2. `s02_redundant_flag_write.wf` REJECTs and `s02c_joined_control.wf` ACCEPTs**,
and the pair is where D4's price *looked* like it lived. `s02` is `c01` with the
`else` arm setting the flag to `False()` — the value that edge already carries —
so the arm-exit states still decide `f` oppositely and the entry `[+f]` is sound
while the syntactic arm-write test refuses it. **Executing the whole rule rather
than the one clause changes the answer**: `f` is a `Bool` place datum an arm
commits, so clause (a′) considers it, finds `A ⊢ +f` and `B ⊢ -f`, and forms the
identical entry. `s02` **compiles** under the drafted text — through the other
clause, and at the cost of P-S5B item 3, which (a)/(b) would not have needed.
This is the round's own execution correcting the round's own first reading of its
own decision, and it is why §8 executes every witness against §3's whole text
rather than against the clause the witness was written for.

**3. `s03_operand_write_preserving.wf` REJECTs and `s03c_joined_control.wf`
ACCEPTs**, and *this* pair prices D4. The condition is the comparison
`ige(m, 8_u64)`, not a `Bool` place, so (a′) has no `S` to work with; the then arm
writes `set m = 100_u64;`, an operand of the condition, **preserving its truth on
that edge**, so `A ⊢ +ige(m, 8_u64)` and `B ⊢ -ige(m, 8_u64)` and the entry would
be sound. The syntactic test looks at the write and not at the value, and refuses.
`s03c` is the same program with the else arm delivering `3`, so the fact is joined
and it compiles. **This is the whole family D4 gives up**: a non-place condition
whose arm writes an operand without changing the comparison. §9.1 carries the row
and F-G7 is the falsifier that says how much of the corpus it costs.

---

## 6. Flagship A — `probes/layout.wf`, hand-executed end to end

186 lines: one line of a layout engine. Ten style flags — three from struct
fields, one from a bare `Bool` parameter, six from content and geometry
comparisons — nine offsets committed under flags, a middle phase with a bounded
accumulation walk and a sink call, and a render phase that re-tests the flags
singly, in conjunction, three deep and inside a loop. **No workaround appears in
it**: no offset is re-tested at its use, no `else` arm exists that no execution
takes, no enum encodes the correlation, and no helper carries it in a `requires`.

### 6.0 What today's checker says

Compiled, the program rejects; neutralising each rejecting site in turn and
recompiling enumerates the whole rejection surface
(`probes/layout_neutralised.wf` is the fixed point and **ACCEPTs**):

```
  tail_at  < len(deref(glyphs))    let b = deref(glyphs)[tail_at];
  mark_at  < len(deref(glyphs))    let b = deref(glyphs)[mark_at];
  col_at   < len(deref(glyphs))    let b = deref(glyphs)[col_at];
  hyph_at  < len(deref(glyphs))    let b = deref(glyphs)[hyph_at];
  ind_at   < len(deref(glyphs))    let b = deref(glyphs)[ind_at];
  rtl_at   < len(deref(glyphs))    let b = deref(glyphs)[rtl_at];
  just_at  < len(deref(glyphs))    let b = deref(glyphs)[just_at];
  ile(n, col_w)                    let s = fill_span(width: col_w, count: n);
  head_end < len(deref(glyphs))    let b = deref(glyphs)[head_end];
```

Nine residuals, eight `[OP-4]` and one `[FN-8]`, one per flag correlation — and
nothing else. The ten flags, the effect rows, the ownership and region structure,
the accumulation loop and the sink call are all already legal. These nine also
need `[ENT-3.S5]`'s local value-commit image (0108 §3.4), because `set x = t;`
publishes nothing today (`g05`); two of the nine deltas mention `style.columns`
and so additionally need P-S5B item 1.

### 6.1 The counting rule the deltas follow

A delta is **what the join is about to lose**, not what the guard publishes and
not what the arm commits. On a difference-bound state that means: a guard of the
form `n >= c` improves the bound of **every ordered pair `(x, v)` with `x` known
at or below zero and `v` known equal to `n`** — not just the pairs the arm wrote.
Writing `Z_f(C)` for the terms pinned to `Z` at `C` and `U(C)` for the terms known
equal to `n`, a positive arm contributes on the order of `|Z_f| · |U|` members
before it commits anything, and a **negative** arm — `n <= c - 1` — contributes on
the order of `(free terms below n) · |Z_f|`, which on this program is larger.
`a12_dbm_collapse.wf` (ACCEPT) pins the positive half and `r07_upper_collapse.wf`
(ACCEPT) the negative half.

### 6.2 Formation, checked clause by clause under L-G3

This is the round-3 pass the earlier rounds did not make: **at every branch
continuation of the flagship, does the arm-write test admit the entry?** The
answer is yes at every one of them, and the reason is worth reading, because it
is what makes D4 free here.

| continuation | condition `G` | `support(G)` (D2) | what the arms write | forms? |
| --- | --- | --- | --- | --- |
| @51 | `f06_tail` | `{f06_tail}` | `tail_at`, arm-local `t` | **yes** |
| @55 | `f05_wide` | `{f05_wide}` | `mark_at`, `t` | **yes** |
| @58 | `f07_head` | `{f07_head}` | `head_end` | **yes** |
| @62 | `band(f08_multicol, f06_tail)` | `{f08_multicol, f06_tail}` | `col_at`, `t` | **yes** |
| @66 | `band(f02_hyphenate, f06_tail)` | `{f02_hyphenate, f06_tail}` | `hyph_at`, `t` | **yes** |
| @70 | `band(f09_indented, f07_head)` | `{f09_indented, f07_head}` | `ind_at`, `t` | **yes** |
| @78 | `f10_canvasfit` | `{f10_canvasfit}` | `rtl_at`, `t` (two levels in) | **yes** |
| @77 | `f03_rtl` | `{f03_rtl}` | `rtl_at`, `t` | **yes** |
| @76 | `f06_tail` | `{f06_tail}` | `rtl_at`, `t` | **yes** |
| @84 | `f06_tail` | `{f06_tail}` | `just_at`, `t` | **yes** |
| @83 | `band(f04_strict, f01_justify)` | `{f04_strict, f01_justify}` | `just_at`, `t` | **yes** |
| @90 | `f08_multicol` | `{f08_multicol}` | `col_w` in both arms, `t` | **yes** |
| @98 | `more` | `{more}` | — | **no entry: one reaching edge** (the else arm `break`s) |
| @107 | `f10_canvasfit` | `{f10_canvasfit}` | the `ink` call, `writes(canvas)` | **yes** — see below |
| @113 | `band(f09_indented, f07_head)` | `{f09_indented, f07_head}` | `ind_at`, `t` | **yes** |
| @118…@157 | the thirteen render-phase branches | the flags | `out`, `b`, `s` | **yes** |
| @167 | `f07_head` | `{f07_head}` | `out`, `b` | **yes** |
| @161 | `done` | `{done}` | — | **no entry: one reaching edge** (`break`) |

**Not one arm of this program writes a flag**, which is what the round-2 audit
found by enumerating the `set` targets (`tail_at`, `mark_at`, `head_end`,
`col_at`, `hyph_at`, `ind_at`, `rtl_at`, `just_at`, `col_w`, `acc`, `k`, `out`,
`pass`) and what this table confirms per continuation. **D4 costs flagship A
nothing.**

Two rows deserve a sentence.

- **@107 is the interesting one.** The arm contains
  `region 'sink { let w = ink<'sink>(canvas: &uniq 'sink deref(canvas), …); }`,
  and `ink` declares `writes(canvas)`, so a kill (b) event *does* occur on the
  path through that arm — and a region exit kills (c)/(d) with it. The test asks
  whether any of them applies to `support(G)`, and by D2 that is the **binding**
  `{f10_canvasfit}`, not the `room = len(deref(canvas))` its initializer read. It
  does not. The entry forms. Had D2 gone the other way, this branch would have
  formed nothing and the flagship would have lost a row — the same sentence,
  doing work at a third site.
- **@98 and @161 form nothing at all**, and round 2's §5.2 was wrong to list
  `[±more]` as an entry killed later by kill (d). The `else` arm `break`s, so only
  one edge reaches the continuation and `[ENT-5.G2]`'s single-reaching-edge
  sentence applies — the same sentence that makes flagship B's early `return`
  free. The entry count is unaffected because round 2's own line-114 count did not
  include it.

### 6.3 Every formed entry

`chain(C)` is empty at every top-level continuation, `[+f10_canvasfit]` at @77,
`[+f10_canvasfit, +f03_rtl]` at @76, and `[+f06_tail]` at @83. Entries are listed
at their forming continuation; keys that coincide share one row (§6.5). `|δ|` is a
**hand count of the delta as formed** and should be read as ±20%; the
load-bearing member is named exactly. §6.7 states separately what survives to
line 114, which is a different number.

| formed at | key | \|δ\| | the member that discharges, or why the row exists |
| --- | --- | --- | --- |
| @51 | `[+f06_tail]` | 18 | **`tail_at - n <= -8`**; the other 17 are `x - v <= -8` for the eight zero-family terms against `n` and `just_at` |
| @51 | `[-f06_tail]` | 18 | `n - x <= 7` and `just_at - x <= 7` over the nine-term zero family. Releases nothing on this program |
| @55 | `[+f05_wide]` | 16 | **`mark_at - n <= -64`** |
| @55 | `[-f05_wide]` | 24 | `n <= 63` bounds `tail_at` too, which is now free |
| @58 | `[+f07_head]` | 20 | **`head_end - n <= -1`**, a *derived* member: the arm commits `head_end = 3` and closure against the guard's own `n >= 4` produces it |
| @58 | `[-f07_head]` | 28 | the largest row in the function: `n <= 3` bounds `tail_at`, `mark_at` and the zero family |
| @62 | `[+band(f08_multicol, f06_tail)]` | 19 | **`col_at - n <= -2`**, plus `Z - style.columns <= -2` |
| @62 | `[-band(f08_multicol, f06_tail)]` | 5 | `col_at - x <= 0`: the false edge keeps the constructor default while `A` puts `col_at` at `n - 2` with `n` unbounded above, so the join loses the upper bound |
| @66 | `[+band(f02_hyphenate, f06_tail)]` | 14 | **`hyph_at - n <= -3`** |
| @66 | `[-band(f02_hyphenate, f06_tail)]` | 4 | as @62's negative row |
| @70 | `[+band(f09_indented, f07_head)]` | 15 | **`ind_at - n <= -4`**, plus `style.indent != Z` |
| @70 | `[-band(f09_indented, f07_head)]` | 3 | as @62's negative row |
| @78 | `[+f10_canvasfit]` | 4 | `x - room <= -32` over the three-term zero family, plus `head_end - room <= -29` |
| @78 | `[-f10_canvasfit]` | 5 | `room - x <= 31`, plus `rtl_at - x <= 0` |
| @77 | `[+f10_canvasfit, +f03_rtl]` | 0 | `f03_rtl` is a `Bool` field with no comparison origin and the arm commits nothing at this level. **An empty delta is still an entry** (L-G1) |
| @77 | `[+f10_canvasfit, -f03_rtl]` | 2 | `rtl_at - x <= 0` |
| @76 | `[+f10_canvasfit, +f03_rtl, +f06_tail]` | 16 | **`rtl_at - n <= -7`**. Four of the sixteen are facts **released on this edge** by the `[+f06_tail]` entry and then re-recorded under the deeper key (§4.2) |
| @76 | `[+f10_canvasfit, +f03_rtl, -f06_tail]` | 24 | `n <= 7` bounds six free terms |
| @84 | `[+f06_tail]` | 7 | unions into @51's row |
| @84 | `[-f06_tail]` | 16 | unions into @51's row |
| @83 | `[+f06_tail, +band(f04_strict, f01_justify)]` | 5 | **`just_at - n <= -5`** |
| @83 | `[+f06_tail, -band(f04_strict, f01_justify)]` | 5 | `n - just_at <= 0`: the sentinel default `just_at = n` is what the join loses |
| @90 | `[+f08_multicol]` | 3 | `Z - style.columns <= -2`, `col_w - n <= -2` |
| @90 | `[-f08_multicol]` | 3 | **`n - col_w <= 0`**, plus `style.columns - Z <= 1` |
| @107 | `[±f10_canvasfit]` | ~2 | unions into @78's rows |
| @113 | `[+band(f09_indented, f07_head)]` | ~15 | unions into @70's row and **re-establishes what @109's write killed** (§6.5) |
| @113 | `[-band(f09_indented, f07_head)]` | ~3 | unions into @70's negative row |
| @118…@157 | thirteen render-phase branch continuations | ~65 total | mostly facts about `out`; they union into the rows above where the keys coincide |
| @167 | `[±f07_head]` | ~4 | inside `@paint`; unions into @58's rows |

*The negative rows are not empty and not small.* Round 1 wrote that *"a negated
conjunction publishes no conjunct and no numeric content, so the false-edge delta
is empty and no entry is formed."* The first clause is true and the conclusion
does not follow: a delta is what the join loses, and the false edge of
`if band(f08, f06)` keeps `col_at` at its constructor default while the then edge
sends it to `n - 2` with `n` unbounded above. @51's, @55's, @58's and @76's
negative rows are among the largest in the function.

### 6.4 The nesting, and the carry-out that leaves the arms

Round 2 forms `[+f10_canvasfit, +f03_rtl, +f06_tail]` directly at @76, because
`chain(@76)` is `[+f10_canvasfit, +f03_rtl]` by syntax, and then **carries it out
unchanged** at @77 and @78 under `[ENT-5.G3]`(d). The same at @83:
`chain(@83) = [+f06_tail]`, so the key is
`[+f06_tail, +band(f04_strict, f01_justify)]` from the start and is carried out at
@84. Round 3 re-checks the carry-out at each of the three continuations under
L-G4's two conditions, the second stated over **paths through the other arm**:

| at | present on | (i) member records the arm | (ii) does the other arm disturb any key member's support? | carried? |
| --- | --- | --- | --- | --- |
| @77 | then edge of `if f03_rtl` only | `+f03_rtl` ✓ | the false edge is the empty implicit `else`: no commit, no call, no scope exit reaching a key member. **No** | **yes** |
| @78 | then edge of `if f10_canvasfit` only | `+f10_canvasfit` ✓ | same, empty implicit `else`. **No** | **yes** |
| @84 | then edge of `if f06_tail` only | `+f06_tail` ✓ | same. **No** | **yes** |

The three "empty implicit `else`" cells are the whole cost of L-G4 on this
program: **zero**. It is worth noticing that under the *literal* reading round 2
left open, these three rows would read the same — the difference between the
readings appears only when an arm writes a flag away from its last statement,
which is `b01` and which this program does not contain. A flagship cannot settle
a reading; only a witness can, which is why §8.2 derives `r09` and `b01` under the
one wording.

### 6.5 The middle phase, and the E11/E11′ union worked under L-G2

**`@scan` (93–102).** Its continuing kill events are `set acc` and `set k`. No
live entry's key support contains `acc` or `k`, and no live entry's delta mentions
them, so `[ENT-5.G3]`'s head subtraction removes nothing at either granularity.
Every entry flows through the body to the `break` edge and reaches the
continuation. Mandatory work between the phases costs the guarded sets nothing.

**The sink call (105).** `ink` declares `writes(canvas)`, so kill (b) projects
onto the `deref(canvas)` actual and removes `room = len(deref(canvas))` and every
fact supported by that storage. It removes **no guarded entry**: the delta facts of
the `f10_canvasfit`-keyed rows are over the local place `room`, not the canvas
storage, and — **the support of a key member is the support `[ENT-5]` 3040–3041
already gives that signed goal: the resolved places its own direct typed
expression reads — for a binding goal, the binding — and never the places of its
origin expansion** — so the key `+f10_canvasfit` survives a write to `room`'s
origin. The `x - room <= -32` members of the `[+f10_canvasfit]` row **do** die,
because their own support contains `room`; that row is left holding
`head_end - room <= -29` and nothing this program consumes.

**`set ind_at = widened;` (109), and the union at @113.** At @70 the entry
`[+band(f09_indented, f07_head)]` holds `ind_at - n <= -4` among 15 facts. Line
109 kills every fact whose support contains `ind_at` — kill (a) — so that member
and its companions die. **The entry does not die**: its key member's support is
`{f09_indented, f07_head}`, untouched. Lines 110–113 re-establish the correlation
under the same condition, and at @113 clause (a) forms an entry with key
`chain(@113) ++ [+band(f09_indented, f07_head)]` — **the identical key**. One row
per key, **unioned** at formation: the row's delta becomes the per-pair tighter of
what survived and what @113 derived, `ind_at - n <= -4` is back, and site 133
discharges.

**Why the union is sound, stated where the operator is stated** (L-G2, and the
round-2 audit's R4). The row was present at every point from @70 to @113 — nothing
between them touches `{f09_indented, f07_head}`, and every merge on the path is
either an ordinary merge where the key is present on all contributing edges or a
branch continuation where it is. So by L-G2 the key's truth value is **constant**
across that span; a point where the key is satisfied is a point at which the
`band(f09_indented, f07_head)` arm ran at **both** sites; so both deltas hold
there and the tighter constant is sound. Round 2's justification — *"each was
established under the same key on a path reaching here, so each holds wherever the
key is satisfied"* — is the conclusion, not the argument. And the union is
consistent here, as `[ENT-5.G2]`'s consistency step requires: `ind_at - n <= -4`
from @113 against the surviving members of @70, which are the same facts.

- **Union at a formation point** combines deltas live **on the same path** under
  L-G2's premise. Sound.
- **Join at a merge** combines deltas arriving on **different paths**. A fact
  established under key `K` on one incoming path need not hold on another path
  where `K` is also satisfied. Join is required, and unioning at a merge would be
  a soundness bug.

`[ENT-5.G2]` says "unioned into" and `[ENT-5.G3]`(c) says "the join of", and this
paragraph is why the two words differ.

### 6.6 Every release

Step (2′) of `[ENT-5.P0]`, on the edge named. "identity" means the key member is
the same signed goal `[ENT-3.S1]` publishes there; "reconstruction" means
`[ENT-4]`'s parent closure (`g08`); "decomposition" means S1's signed
decomposition (`g09`). **There is no projection column, because there is no
projection route.**

| site | edge facts | entry satisfied, how | released | obligation |
| --- | --- | --- | --- | --- |
| 115 `if f06_tail` | `+f06_tail` | `[+f06_tail]` by identity | `tail_at - n <= -8` | `tail_at < len(deref(glyphs))` **discharged** |
| 119 `if f05_wide` | `+f05_wide` | `[+f05_wide]` by identity | `mark_at - n <= -64` | `mark_at < …` **discharged** |
| 123→124 | `+f08_multicol`, `+f06_tail` | `[+band(f08_multicol, f06_tail)]` by **reconstruction** over the tree interned at line 59 | `col_at - n <= -2` | `col_at < …` **discharged** |
| 129 | `+band(…)` and its decomposition | by identity | `hyph_at - n <= -3` | `hyph_at < …` **discharged** |
| 133 | `+band(…)` | by identity, on the row §6.5's union repaired | `ind_at - n <= -4` | `ind_at < …` **discharged** |
| 137→138→139 | `+f10_canvasfit`, `+f03_rtl`, `+f06_tail` | the depth-3 key, **all three members by identity**; the key survived the sink call by D2 | `rtl_at - n <= -7` | `rtl_at < …` **discharged** |
| 145→146→147 | `+f06_tail`; `+f04_strict`; `+f01_justify` | member 1 by identity, member 2 by **reconstruction** of `+band(f04_strict, f01_justify)` interned at line 80 | `just_at - n <= -5` | `just_at < …` **discharged** |
| 153 else edge | `-f08_multicol` | by identity on a **negative** key | `n - col_w <= 0` | `[FN-8] ile(n, col_w)` **discharged** |
| 164, inside `@paint` | `+f07_head` | `[+f07_head]`, which survived the `@paint` head subtraction | `head_end - n <= -1` | `head_end < …` **discharged** |

**Nine sites, nine discharges, and §6.0's residual list is empty** — unchanged by
round 3, because L-G3 refuses nothing here and L-G4 carries everything here.
Every release route the design defines is exercised: identity, reconstruction
from nested singles, decomposition inside a compound, a negative key, a
three-member key, and a release from inside a loop body.

**The loop seam, at row nine.** `@paint`'s continuing kills are `set out` and
`set pass`. The `[+f07_head]` row's key support is `{f07_head}` and its delta
mentions `head_end`, `n`, `just_at` and the zero family — none written in the body
— so the head subtraction removes the entry at neither granularity, and step (2′)
releases on the then edge of `if f07_head` on every iteration.

### 6.7 The measurement, with its two figures labelled

Round 2 reported *"22 live entries holding about 257 atomic facts at line 114"*.
The entry count is right; the fact count was a **formation-time sum presented as a
point measurement**, and the two are now separate numbers.

- **Live entries at line 114: 22.** The rows from @51, @55, @58, @62, @66, @70,
  @78, @77, @76, @83 and @90 — eleven continuations, two entries each — with @84,
  @107 and @113 unioning into earlier rows and @98 forming nothing (§6.2).
  Against the entry bound `2(N_if + S_B) = 60`, a factor of **2.7**.
- **Delta members as formed: 251.** The `|δ|` column of §6.3 summed over those 22
  rows. This is a *formation-time* figure: it is what the rows held when they were
  made, at eleven different program points.
- **Delta members live at line 114: ≈245.** §6.5's own kills are deducted: the
  sink call removes the three `x - room <= -32` members of the `[+f10_canvasfit]`
  row and the `room - x <= 31` members of its negative twin. Line 109's kill is
  restored by @113's union before 114.
- **Against the ambient state:** with `T = 21` the closed L0 state at that point
  holds at most 420 ordered-pair bounds, so the guarded component is about
  **0.6×** the state it rides beside — an order of magnitude inside F-G2's
  threshold of ten times.
- **Against the bound:** 251 against `3(N_if + S_B)·T(T-1) = 3 × 30 × 21 × 20 =
  37,800` is a factor of 150 — **two** orders of magnitude, not three.

All five figures are hand counts at ±20% except the entry count, which is exact
and which §3.7's bound makes a theorem.

### 6.8 What the writer did not have to do

| route the writer would otherwise take | cost on this program |
| --- | --- |
| re-test the offset at its use (`g16`, compiles) | 9 extra comparisons and 9 extra branches, each with a false arm no execution takes; the render phase doubles in branch count and the reader can no longer tell which tests are real |
| guard the value with an impossible `else` in value position | the `fill_span` site becomes `Result`-typed or invents a width; 0108 §11.1's third tier |
| restructure into an enum carrying the correlated value | ten flags means up to 2^10 states to encode, or one enum per correlation — nine new nominal types on one function |
| factor each dependent statement into a function whose `requires` states the correlation | 9 helper functions on a 60-statement render phase, each called from inside the arm that already tested the flag |

---

## 7. Flagship B — the record of flags, hand-executed end to end

`audit/probes/a05_record_inline.wf`, the round-1 audit's own second flagship,
chosen because round 1 discharged **none** of it. It is structurally different
from flagship A on every axis that matters: the flags **and** the offsets live in
one struct the function itself writes; an **early `return`** sits between the
measure and use phases; the struct is handed to a `&uniq` mutator in between; one
site is under a **negative** key; and one is **inside a loop**.

### 7.0 What today's checker says

Neutralising each rejecting site in turn to its fixed point
(`a05_record_inline_neutralised.wf`, **ACCEPT**) enumerates the whole surface:

```
  h.body_at < len(deref(data))     if h.has_body { deref(data)[h.body_at] }
  h.wide_at < len(deref(data))     if h.wide    { deref(data)[h.wide_at] }
  h.wide_at < len(deref(data))     else         { deref(data)[h.wide_at] }
  h.body_at < len(deref(data))     inside @paint, if h.has_body
```

Four residuals, all flag correlation, nothing else — the same result flagship A
produces on a different structure. **The family is real, and it is not one
idiom.**

### 7.1 Formation, checked clause by clause under L-G3

`parse_record`'s `set` targets are `h.has_body`, `h.body_at`, `h.wide`,
`h.wide_at`, `out`, `pass`, and `deref(h).seen` inside `bump`. Its branch
conditions are `room4`, `room64`, `bad`, `h.has_body`, `h.wide`, `done` and
`h.has_body` again inside `@paint`.

| continuation | clause | key member | discharge | forms? |
| --- | --- | --- | --- | --- |
| @30 | (a), (b) | `±room4` | arm-write test: the arm writes `h.has_body`, `h.body_at`, `b` — not `room4` | **yes** |
| @30 | (a′) | `±h.has_body` | arm-exit condition: `A ⊢ +h.has_body` (P-S5B items 2 and 3, from `+yes`), `B ⊢ -h.has_body` (item 1, unwritten on that edge) | **yes** |
| @38 | (a), (b) | `±room64` | arm-write test: the arms write `h.wide`, `h.wide_at`, `w` | **yes** |
| @38 | (a′) | `±h.wide` | arm-exit condition, symmetrically | **yes** |
| @42 | — | — | `if bad { return 0_u8; }`: **one reaching edge**, so nothing forms and every entry passes through | **no entry, free** |
| @50, @57 | (a), (b) | `±h.has_body`, `±h.wide` | arm-write test: the arms write `out`, `g`, `g0` | **yes** |
| @67 | (a), (b) | `±h.has_body` | inside `@paint`; the arm writes `out`, `g` | **yes** |
| @62 | — | — | `if done { break @paint; }`: one reaching edge | **no entry** |

**No arm of flagship B writes the flag its own branch tests either**, so D4 costs
this program nothing as well. The one place an arm writes a `Bool` place is @30
and @38 — and there the place is `h.has_body` / `h.wide`, which is a *different*
goal from the branch condition `room4` / `room64`, so the branch keys are
untouched and the committed-flag keys are formed by the clause that was written
for exactly this. **This is the structural reason the refutation does not reach
flagship B**, and it is worth stating because `c01` and flagship B look alike from
a distance: both have an arm that writes a `Bool`. The difference is whether the
`Bool` the arm writes is the one the branch tested.

### 7.2 The measure phase: what forms

Terms live at the measure phase: `n`, `room` (both `len(deref(data))`),
`h.has_body`, `h.wide`, `h.body_at`, `h.wide_at`, `h.seen`, `tag`, `Z`. The zero
family at line 26 is `{Z, h.body_at, h.wide_at, h.seen}`; `U = {n, room}`.

**@30, the continuation of `if room4 { set h.has_body = yes; set h.body_at = b; }`.**
`A` derives `n >= 4`, `h.body_at = n - 4`, `+h.has_body`; `B` derives `n <= 3`,
`h.body_at = 0` (unwritten since the construction), `-h.has_body`. Four entries:

| clause | key | \|δ\| | contains |
| --- | --- | --- | --- |
| (a) | `[+room4]` | 8 | `h.body_at - n <= -4`, `x - v <= -4` over the zero family |
| (b) | `[-room4]` | 8 | `v - x <= 3` |
| **(a′)** | **`[+h.has_body]`** | **8** | **`h.body_at - n <= -4`** — the same delta, under the key the use phase will test |
| (a′) | `[-h.has_body]` | 8 | `diff(B, J)` |

**@38, the continuation of `if room64 { set h.wide = yes; set h.wide_at = w; }
else { set h.wide_at = 0_u64; }`.** The `Bool` place committed in an arm is
`h.wide`; `h.wide_at` is committed in both arms and is not a `Bool`, so it is a
delta term, not a key candidate. Four entries again, and **two of them
discharge**:

| clause | key | contains |
| --- | --- | --- |
| (a) / (b) | `[±room64]` | the comparison keys; nothing on this program consumes them |
| **(a′)** | **`[+h.wide]`** | **`h.wide_at - n <= -64`** |
| **(a′)** | **`[-h.wide]`** | **`h.wide_at - Z <= 0`** and `Z - h.wide_at <= 0`. The else arm pins `h.wide_at` to zero while the then arm sends it to `n - 64` with `n` unbounded above, so the **upper** bound is what the join loses |

**@42, the early `return`.** One reaching continuation edge, so `[ENT-5.G2]`
forms nothing and the continuation is an ordinary merge over that edge: every
entry passes through unchanged. An early exit between the phases is free.

**The `&uniq` mutator at 44.** `bump` declares `reads(h.seen), writes(h.seen)`.
Kill (b) projects under `[EFF-2]` onto the resolved place `h.seen`, which does not
overlap `h.has_body`, `h.wide`, `h.body_at` or `h.wide_at` under `[OWN-7]`. Every
key survives; the only casualties are delta members mentioning `h.seen`, which no
site consumes. **This is now probed on both halves**: `r06` (ACCEPT) for the L0
fact and `b02` (ACCEPT) against `b02c` (REJECT) for the signed goal — the step
round 2 asserted and could not probe.

### 7.3 Every release

| site | edge facts | entry satisfied, how | released | obligation |
| --- | --- | --- | --- | --- |
| 47 `if h.has_body` | `+h.has_body` | `[+h.has_body]` by **identity** — a field place datum is a direct goal (`g07`) | `h.body_at - n <= -4` | `h.body_at < len(deref(data))` **discharged** |
| 51 `if h.wide` then | `+h.wide` | `[+h.wide]` by identity | `h.wide_at - n <= -64` | `h.wide_at < …` **discharged** |
| 54 else edge | `-h.wide` | `[-h.wide]` by identity, on a **negative committed-flag key** | `h.wide_at - Z <= 0` | `h.wide_at < …` **discharged**, with S4's `room >= 1` supplying the other premise |
| 64, inside `@paint` | `+h.has_body` | `[+h.has_body]`, which survived the head subtraction | `h.body_at - n <= -4` | `h.body_at < …` **discharged** |

**Four residuals, four discharges.** `@paint`'s continuing kills are `set out` and
`set pass`; the `[+h.has_body]` row's key support is `{h.has_body}` and its
surviving delta is over `h.body_at`, `n` and `Z`, so the head subtraction removes
the entry at neither granularity and the release repeats every iteration.

Row three is worth reading twice: it is the release round 1 could not have made
under any repair, because the **negative** side of a committed flag is not a
branch condition at all. The writer wrote `if h.wide { … } else { … }` and
subscripted with `h.wide_at` in both arms; the else arm's correlation is *"the
flag is false, so the offset is still the sentinel"*, and (a′)'s symmetric clause
carries it.

### 7.4 The rewrite this replaces, and what flagship B needs

`a06_record_localflag.wf` is the same program with every use site rewritten to
test the local comparison flag instead of the record field. It rejects with the
identical four residuals, and even round 1's rule discharges all four. So round 1
did not fail on this program's *difficulty*; it failed on the writer's choice of
which name to test — and the rewrite is exactly the plumbing §6.8 is proud that
flagship A does not contain: a shadow local for every flag already in the record,
a second name for one predicate carried across two phases, and a binding kept
alive only to satisfy the checker.

**P-S5B, all three items** (§4.3.3). Flagship A needs only 0108 §3.4's local
value-commit image plus item 1 for two deltas mentioning `style.columns`.
Flagship B needs the construction image (item 1) to derive `-h.has_body` on the
false edge, the field commit image (item 2) for both the `Bool` and the numeric
destination, and the `Bool` literal goal (item 3) so that `+yes` is available at
the commit. `r01`, `r03` and `r04` (all REJECT) are the three gaps, one per item.

**Nothing else.** No new release route, no new satisfaction condition, no new
support rule, no new kill. **The support of a key member is the support `[ENT-5]`
3040–3041 already gives that signed goal: the resolved places its own direct typed
expression reads — for a binding goal, the binding — and never the places of its
origin expansion** — and for `+h.has_body` that is the field place, which is the
right answer: a write to `h.has_body`, to `h`, or through a `&uniq` borrow by a
callee declaring `writes(h)` kills the entry, and each is a point after which the
flag no longer records which arm ran.

---

## 8. Every witness, hand-executed under the final text

Each of these is executed against §3's rule text as written above, not against a
summary of it, and each verdict is stated from the execution rather than
asserted. The compiled verdicts in the "today" column are reproduced by
`./run.sh`, `./audit/run.sh` and `./audit2/run.sh`.

### 8.1 The formation family — `c01`, `c02`, `c01c`, `s02`, `s02c`, `s03`, `s03c`

**`c01_arm_writes_flag.wf` — REFUSED at formation.** At the `value_if`
continuation `C`, `chain(C)` is empty and the condition `f` is a place datum,
hence a direct goal. Clause (a) asks first whether either arm **disturbs**
`support(+f) = {f}` (D2: the binding). The `else` arm contains
`set f = yes;` — a `[SET-1]` commit whose resolved target is `f` — at a node on
the path from the branch through that arm to `C`. It disturbs. **No entry is
formed, for either sign.** Nothing is inherited (this is the first branch on `f`),
so the guarded component at `C` is empty on this key. At `if f` step (2′) has
nothing to release, `x < len(deref(data))` stays unproved, and the program
**rejects exactly as it does today** with `[OP-4] residual: x < len(deref(data))`.
The writer sees `[ENT-5.G8]`'s *not formed* string naming line 66's `set f = yes;`.

**And clause (a′) does not pick it up**, which has to be checked because the arm
commits a `Bool` place and (a′) is the clause for `Bool` places arms commit. Its
condition is that the two arm-exit states decide `S` **oppositely**. Here
`S = f`: `A ⊢ +f` (the then arm was entered because `f` was true and does not
write it) and `B ⊢ +f` (the else arm has just set it true). Both edges derive
`+f`, so (a′) is in its *"every other case"* branch and forms nothing. That is
the same fact the refutation turns on, seen from the other clause: after this
branch `f` no longer tells anyone which arm ran.

**`c02_negative_key_arm_writes.wf` — REFUSED at formation**, by the identical
step through clause (b): the **then** arm commits `f`, the pair test is over
`support(G)` and refuses both signs, and the `[-f]` entry the else edge would
have carried never exists. (a′) again forms nothing — `A ⊢ -f` after the write
and `B ⊢ -f` on the else edge, the same sign on both. **Rejects.**

**`c01c_joined_control.wf` — still ACCEPTED.** The `else` arm still writes `f`,
so round 3 forms nothing here either — and it does not matter: both arms deliver
`3`, so `x <= 3` survives the ordinary `[ENT-5]` join and no guarded release is
needed. This is the row that shows the refusal removes no acceptance: the sole
difference between `c01` and `c01c` is a literal, and round 3 refuses the release
in both while the join carries `c01c` on its own.

**`s02_redundant_flag_write.wf` — REFUSED by (a) and (b), and ADMITTED by (a′).**
The `else` arm's `set f = no;` writes `f` to the value that edge already carries.
Clauses (a) and (b) refuse: the arm disturbs `support(+f) = {f}` and the test does
not look at values. Clause (a′) then asks its own question of the same write —
`f` is a `Bool` place datum an arm commits — and finds `A ⊢ +f` and `B ⊢ -f`, the
opposite-decision case, so it forms `[+f] ↦ diff(A, J) ∋ x - Z <= 3` and
`[-f] ↦ diff(B, J)`. At `if f` the `[+f]` entry is satisfied by identity, `x <= 3`
is released, and with `room >= 4` the subscript discharges. **The program
compiles**, and it is safe: called with `f0 = True` the then arm runs and `x` is 3.

Two things follow, and both are round 3 correcting its own first reading. First,
**the two discharges of L-G3 interlock**: where the condition is a `Bool` place,
what the syntactic test refuses wrongly the derivability discharge admits rightly,
so D4's price is not "an arm that writes its flag" at all. Second, the price is
not zero either — `s02` now needs **P-S5B item 3** to derive `-f` from
`set f = no;`, where the branch-key route needed no `[ENT-3.S5]` extension at all.
D4 moves this program from a prerequisite-free route to a prerequisite-carrying
one. `s02c_joined_control.wf` (ACCEPT) pins the discharge arithmetic either way.

**`s03_operand_write_preserving.wf` — REFUSED, and this is D4's real price.** The
condition is `ige(m, 8_u64)`, a comparison and not a place, so `support(G) = {m}`
and the then arm's `set m = 100_u64;` disturbs it: (a) and (b) refuse. (a′) has
nothing to work with, because no arm commits a `Bool` place. And the entry would
have been sound: `A ⊢ +ige(m, 8_u64)` (100 ≥ 8) and `B ⊢ -ige(m, 8_u64)`, so the
arm-exit derivability discharge is satisfied and the transplant form of the repair
would form `[+ige(m, 8_u64)] ↦ {x - Z <= 3}`, release it at the later
`if ige(m, 8_u64)`, and compile. Under D4 the program **rejects**, as it does
today. `s03c_joined_control.wf` (ACCEPT) pins that the arithmetic is otherwise
present. **This shape — a non-place condition whose arm writes an operand while
preserving the comparison — is the whole of what D4 costs**, §9.1 carries it, and
F-G7 measures it against the corpus.

### 8.2 The carry-out pair, derived side by side — `r09` and `b01`

Both are `if f { if g { set x = 3_u64; } } else { … set f = yes; … }` followed by
`if f { if g { deref(data)[x] } }`, and they differ only in **where** the flag
write sits in the `else` arm. The point of deriving them together is that under
round 3's wording they must land identically, and they do.

| step | `r09` (write is the arm's last statement) | `b01` (write is interior; two statements follow) |
| --- | --- | --- |
| formation at the inner continuation | `chain = [+f]`, condition `g`, `support(+g) = {g}`; the inner arm writes `x`, not `g`, so the arm-write test passes and `[+f, +g] ↦ {x - Z <= 3}` forms | identical |
| formation at the outer continuation | condition `f`, `support(+f) = {f}`; the `else` arm commits `f`, so it **disturbs**, so `[±f]` **form nothing** here | identical — the write is interior, but *disturbance is over every kill event on every path through the arm*, so it disturbs just the same |
| carry-out of `[+f, +g]` at the outer continuation | present on the then edge only; (i) `+f` is the goal S1 establishes there ✓; (ii) the other arm disturbs `support(+f)` → **not carried** | identical, and this is the row round 2's wording decided differently: the arm-**exit edge** carries only scope-exit kills for `yes` and `pad`, so a reading over edges would have **carried** the entry |
| release at `if f { if g { … } }` | nothing to release | nothing to release |
| verdict | **rejects**, as it does today | **rejects**, as it does today |

Under round 2's literal reading, `b01` reads index 50 of a 4-byte buffer while
`r09` does not — two conforming implementations, one sentence, different accepted
sets, one unsound. L-G4's *events on paths, not events of edges* is what removes
the choice, and it is quoted verbatim at `[ENT-5.G2]`, `[ENT-5.G3]`(d) and
`[ENT-5.G6]`(3) so that no site can paraphrase it back into ambiguity.

**Clause (a′) is checked at the outer continuation of both and forms nothing.**
The `else` arm commits the `Bool` place `f`, so (a′) considers it — and finds
`A ⊢ +f` and `B ⊢ +f`, the same sign on both edges, because the arm sets the flag
to the value the *other* arm was entered on. (a′) is in its *"every other case"*
branch. This is the check `s02` shows is not a formality (§8.1): where the arm's
write leaves the two edges deciding the flag oppositely, (a′) does form the entry
and the program compiles.

Note also that under round 3 **two** independent conditions refuse each program:
formation refuses the `[±f]` keys and carry-out refuses the inherited `[+f, +g]`
entry. That is not redundancy to be trimmed — they are different keys, and each
condition is the only one that reaches its own key.

### 8.3 The flag cleared between store and test — `b04`

`if h.flag { set h.at = 3_u64; }` at `@C1`; `if h.flag { set h.at = 50_u64; }
else { set h.flag = no; … }` at `@C2`; `if h.flag { deref(data)[h.at] }` at the
use. Round 2 refused this program **by accident** — the round-2 audit's words —
because (d) carried the entry as it stood on `e` and the then arm's own write to
`h.at` had already killed the stale fact. Round 3's execution, taken clause by
clause rather than clause by intention:

1. **`@C1` forms `[+h.flag] ↦ {h.at <= 3, h.at >= 3}`.** The condition is
   `h.flag`, `support(+h.flag) = {h.flag}`, and the arm writes `h.at` only. The
   arm-write test passes.
2. **`@C2`'s branch keys are refused.** The `else` arm commits `h.flag`, which is
   the condition's own support, so (a) and (b) form nothing. Round 2 formed
   `[+h.flag] ↦ {h.at <= 50, h.at >= 50}` here as a branch key.
3. **`@C2`'s committed-flag key is admitted.** `h.flag` is a `Bool` place datum
   the `else` arm commits, and the arm-exit states decide it oppositely —
   `A ⊢ +h.flag` (the then arm does not write it), `B ⊢ -h.flag` — so (a′) forms
   `[+h.flag] ↦ diff(A, J)`, which holds `h.at = 50`, and `[-h.flag] ↦ diff(B, J)`.
   Sound: after `@C2`, `+h.flag` does record the then arm, because the else arm
   made it false and nothing writes it afterwards.
4. **`@C1`'s entry is not carried past `@C2`.** It is live entering the branch; on
   the then edge `set h.at = 50_u64;` kills both delta members and leaves the row
   with an empty delta; on the else edge `set h.flag = no;` kills the row
   outright. So the key is present on exactly one reaching edge, L-G4(i) holds
   (`+h.flag` is S1's goal there), and **L-G4(ii) fails** — the other arm
   disturbs `support(+h.flag)`. Not carried.
5. **So no union happens**, and the inconsistent delta the round-2 audit
   constructed — `h.at - Z <= 3` from `@C1` unioned with `Z - h.at <= -50` from
   `@C2` — never forms. It is **L-G4 that prevents it, not formation**: (a′)
   legitimately forms a `[+h.flag]` row at `@C2`, and what keeps the stale row
   from meeting it is the carry rule.
6. **The program rejects.** At the use, `[+h.flag]` is satisfied and releases
   `h.at = 50` — a true fact about the executions that took `@C2`'s then arm — and
   `h.at < len(deref(data))` with `room >= 4` does not follow from it. `[OP-4]
   h.at < len(deref(data))`, exactly as today.

**The rejection is now a derivation rather than an accident**, and its shape is
worth keeping: the correlation the program actually has is *"if the flag is still
set, the offset is 50"*, which is true and useless. `[ENT-5.G2]`'s consistency
step is still required — the operator must be total, and "no program we tried
reaches it" is what round 2 said about three other things — but on this witness it
is not what does the work.

### 8.4 The round-1 refutation — `a01` and `a04`

**`a01_projection_retest.wf` — REFUSED at release.** Formation is admitted: at the
`value_if` continuation, neither arm commits anything at all, so nothing disturbs
`support(+fits) = {fits}` and clause (a) forms `[+fits] ↦ {idx - Z <= 3, …}` from
the delivery images; clause (a′) forms nothing, because no `Bool` place is
committed in either arm. Transport is admitted: `set m = 64_u64;` kills neither
the delta (support `{idx, Z}`) nor the key member (support `{fits}`, by D2).
**Release is where it stops.** `[ENT-5.G5]` has one route: `+fits` must be
derivable on the then edge of `if fits_again`. `[ENT-3.S1]` publishes
`+fits_again` there and, by 2924's expansion, `+ige(m, 8_u64)` — but **not**
`+fits`, and `[ENT-4]` does not back-derive it; `g13`/`g14` (REJECT) against `g15`
(ACCEPT) pin that. 2915(b) independently voids `fits`'s comparison origin at the
use, because `set m = 64_u64;` is a kill event on the replacement's support. The
entry is not satisfied, nothing is released, and the program **rejects exactly as
it does today**. The writer sees the *different proposition* string, and following
it makes the program **still** reject — which is the honest answer, since the
correlation the writer wants is false.

**`a04_loop_projection.wf` — REFUSED at release**, and not by the head rule. The
entry survives `@scan`'s head subtraction (its key support `{fits}` and delta
support `{idx, Z}` contain no continuing-kill target), reaches the body, and fails
to release there for `a01`'s reason: `again` is a third binding of the comparison.
The head rule needed no repair; the release route did, and D1 is that repair.

### 8.5 The remaining witnesses

| witness | today | under round 3 | the step that decides it |
| --- | --- | --- | --- |
| `a05_record_inline.wf` | REJECT ×4 | **all four discharge** | §7: (a′) at @30 and @38 under the arm-exit condition; single-edge continuation at @42; field-granular kill at 44; head subtraction at `@paint`. Needs P-S5B |
| `a06_record_localflag.wf` | REJECT ×4 | all four discharge, and **it is the rewrite the rule exists to make unnecessary** | §7.4 |
| `a07_opaque_field_key.wf` | REJECT `[FN-8]` | still a rejection **before** P-S5B; with P-S5B it is `a05`'s shape | the *different proposition* string is what the writer sees: the entry is keyed on `room4`, the test names `h.flag` |
| `a08_control_localflag.wf` | ACCEPT | unchanged | the control that isolates `a07` |
| `a09`, `a10`, `a11` | REJECT / ACCEPT / ACCEPT | unchanged | they pin P-S5B.1 and field-termhood, not this rule |
| `a12_dbm_collapse.wf` | ACCEPT | unchanged | pins the positive half of the size maximiser; §9.2's bound is derived against it |
| `r01`, `r03`, `r04` | REJECT | unchanged | P-S5B items 1, 3, 2 |
| `r02` / `r08` | ACCEPT / REJECT | unchanged | the pair that shows the checker computes something equivalent to item 3 without exposing it |
| `r05_copy_reverse.wf` | REJECT | unchanged, and it is **why §4.1's copy row is one-directional** | 2924's expansion never replaces a binding by something computed from it |
| `r06_field_kill_granular.wf` | ACCEPT | unchanged | the L0 half of the sibling-field transport |
| `r07_upper_collapse.wf` | ACCEPT | unchanged | the negative half of the size maximiser |
| `r09_carry_out_witness.wf` | REJECT | **refused twice** (§8.2) | L-G3 at the outer continuation, L-G4 at the carry-out |
| `b01_carry_out_interior.wf` | REJECT | **refused, identically to `r09`** (§8.2) | L-G4's *events on paths* |
| `b02` / `b02c` | ACCEPT / REJECT | unchanged | the signed-goal half of the `&uniq` transport, and its control |
| `b03` / `b03c` | REJECT / ACCEPT | unchanged | `[ENT-2]`'s term identity by resolved place — the dependency `[ENT-5.G1]` now names |
| `b04_flag_cleared_between.wf` | REJECT | **refused by derivation** (§8.3): (a)/(b) refuse at `@C2`, (a′) forms a sound row there, L-G4 declines to carry `@C1`'s, and the released `h.at = 50` does not discharge the subscript | L-G4 at the carry-out |
| `c01` / `c01c` / `c02` | REJECT / ACCEPT / REJECT | **refused, refused nothing, refused** (§8.1) | L-G3's arm-write test |
| `s01` / `s01c` | REJECT / ACCEPT | unchanged; **priced** in §9.1 | goal identity by typed tree against resolution by referent |
| `s02` / `s02c` | REJECT / ACCEPT | **`s02` compiles** — (a)/(b) refuse and (a′) admits the same entry, at the cost of P-S5B item 3 (§8.1) | the interlock of L-G3's two discharges |
| `s03` / `s03c` | REJECT / ACCEPT | **`s03` is refused, and this one costs a real program** (§8.1) | D4's syntactic test where (a′) cannot reach: a comparison condition whose arm writes an operand |
| `g01`–`g16`, `layout.wf` | as §5 | unchanged | the isolating pair, the guardable class, the compound routes, the price |

**Every memory-safety witness the three rounds have produced is now a refusal,
and each is refused by a clause that states its own reason:** `a01` and `a04` by
the single release route (D1), `r09` and `b01` by L-G4, `c01`, `c02` and `b04` by
L-G3. **Two programs are refused that need not be** — `s03`, by D4's choice of
discharge, and the `s01` shape, by goal identity — and both are priced. `s02`,
which the first pass of this round recorded as a third, is not: executing the
whole rule rather than the clause the witness was written for showed (a′)
admitting it.

---

## 9. Prices, limits, and the ledger entry

### 9.1 What stays out, each with its cheapest route

| stays out | why | cheapest route today |
| --- | --- | --- |
| **two bindings of one comparison** — established under `if fits`, used under a freshly written `if ilt(i, n)` | D1. The goals differ and neither is derived from the other; releasing across them is `a01` | test the same binding, or bind the condition once. The *different proposition* string says so at the site |
| **the branch's own flag written inside one of its arms** — `if fits { … } else { set fits = yes; … }` | L-G3. The goal stops recording which arm ran, and `c01` is an out-of-bounds read if it does not | move the write after the continuation, or branch on a flag the arms do not write. The *not formed* string names the write |
| **a non-place condition whose arm writes an operand while preserving its truth** — `if ige(m, 8) { set m = 100; … }` | **D4's price, and all of it.** The arm-exit states still decide the condition oppositely, so the entry would be sound; the syntactic test looks at the write and not at the value, and (a′) cannot help because no `Bool` place is committed. `s03` is the program | move the write after the continuation, bind the comparison (`let big = ige(m, 8_u64);`) so that (a′)'s route or a stable key applies, or overrule D4 and take the derivability transplant |
| **the same place named through a borrow at one site** — entry under `h.flag`, use under `deref(p).flag` | goal identity is exact-tree; `[ENT-2]` 2886 resolves a call actual to its referent while `[ENT-3.S1]` keeps the written tree, so the two do not meet (`s01` REJECT, `s01c` ACCEPT). **This is not created by `[ENT-5.G]`** | write the test the way the branch wrote it, or bind the flag once — `let flag = deref(p).flag;` — and test that binding. The *same place, different goal* string says which. §11's Q8 is the real fix and it belongs to `[ENT-2]`/`[ENT-3]` |
| **a flag held in a buffer or array element** — `if flags[i]` | a subscripted place is not a place datum, so the condition has no goal origin and `S` is not a key candidate. `[ENT-2]`'s term vocabulary, not this rule's | bind the element first — `let f = flags[i];`. One line, no branch |
| **a correlation established under a `match` arm and used under a later arm on the same scrutinee** | `[ENT-2]` has no tag term, so an enum arm publishes no goal to key on | bind a `Bool` in each arm and branch on it, or merge the two matches |
| **a correlation carried through a function return with no `ensures`** | crossing a callable boundary is publisher 2's subject matter and no retention rule may speak across it | return the pair and re-derive, or state the correlation as an `[FN-9]`-verified `ensures`. A returned *flag* guards fine (`g10`); a returned *correlation* does not |
| **a record flag whose pre-branch value is undecided** | (a′) needs the other edge to derive the complement | initialize the flag in this function, or write the `else` arm |
| **a signed goal in the delta** — "under `+wide`, `+ok` also held" | the drafted delta is L0-only (§4.2) | re-test the derived flag, or bind it before the branch. §11's Q2 recommends buying this |
| **a correlation first established inside a loop body and needed at the next iteration's head** | entries are not `[ENT-5.R]` candidates (§4.6); released facts are | ordinary retention if the fact is unconditional, `[IND]` if it is not, or hoist the flag test out of the loop |
| **an entry live before a branch and killed on exactly one arm** | L-G4(i): nothing in such an entry's key records the arm it survives on | re-establish under the same condition after the branch, as flagship A does at 110–113. The *not carried* string |
| **a flag reassigned between the two sites** | the entry dies with its key's support, correctly | do not reassign it; or recompute the offset after the reassignment |

Three deserve red ink. **The returned-correlation row is the real ceiling** and it
is the boundary §1's principle draws. **The undecided-record-flag row is the
papercut a writer will actually hit**, and it belongs in `docs/patterns.md`.
**The `s03` row is the one this round chose to pay**, and it is the only price in
the table that a different owner decision would remove.

### 9.2 The size story and the fallback

**The bound.** `2(N_if + S_B)` entries; each delta at most `1.5·T(T-1)` atomic
facts; total `3(N_if + S_B)·T(T-1)`. On `render_line`: `N_if = 30`, `S_B = 0`,
`T = 21`, so 60 entries and 37,800 facts. A `⊥` delta costs one bit and no facts.
Dead entries are not in the map (§3.7, §3.8).

**The measurement** (§6.7): 22 live entries — a factor of 2.7 below the entry
bound — holding 251 delta members as formed and about 245 live at line 114, two
orders of magnitude below the fact bound and about **0.6×** the ambient closed L0
state at that point.

**The attribution.** Round 1 said the gap is because *"an arm changes two or three
terms and the delta is a difference"*. That is refuted: the maximiser is a chain
`t1 <= … <= tT` under one guard `if ile(tT, t1)`, whose then-edge closure gives
`T(T-1)/2` delta members **from an arm that commits nothing** — `a12` (ACCEPT)
pins the positive half, `r07` (ACCEPT) the negative. The correct attribution is
*a delta is large exactly when the guard makes many term pairs comparable at once,
and ordinary programs do not collapse their zones.* **The bound is tight up to a
constant, not loose.**

**The zone figure, corrected.** Round 2 wrote that the guarded component costs at
most `2(N_if + S_B)` extra zones *"over the whole function — a constant number per
branch node"*. That is wrong: the component is carried along the flow, so it costs
that many extra zones **at every program point** — on `layout.wf` a **60×**
state-size ceiling per point. The *measured* factor is 0.6×, so the conclusion
(this is affordable) holds and the reassurance does not.

**D6 and D7's pipeline cost, stated in the same units.** Step 2 of §3.14 runs
once per proof view instead of once (D6), and its ladder pass runs twice per view
instead of once (D7). The universe iteration dominates the ladder pass, so the
honest figure is **step 2 costs about twice what it did**; steps 3–8 are
unchanged, because they already ran per view.

**The fallback, re-specified.** Round 1's *"restrict the delta to ordered pairs at
least one of whose terms the arm commits"* is **retracted as refuted**: on the
collapse shape it drops 100% of the delta, and it is lossless precisely when the
entry is useless. The replacement, if F-G2 fires, is a **priced ceiling with a
named excluded shape**:

> **Fallback N1 (a formation restriction, not a delta restriction).** Form
> `[ENT-5.G2]`(a), (b) and (a′) entries only at continuations at least one of
> whose arms contains a value commit `[SET-1, SET-2]` or a `value_if` delivery.
>
> - *Prover-independent*: "the arm contains a commit" is syntactic, so L-G1 holds.
> - *It neutralises the maximiser exactly*: the collapse shape's arm commits
>   nothing, so no entry forms and the quadratic delta never exists.
> - *It costs both flagships nothing*: every load-bearing entry in §6.3 and §7.2
>   is formed at a continuation whose arm commits.
> - **Its excluded shape, stated**: a guard that only *narrows terms that already
>   exist* — a zone collapse with no commit — whose premises die before the use.
>   Such a correlation is silently unavailable, and no `[ENT-5.G8]` string fires,
>   because no entry was ever formed and the *not formed* string's trigger
>   requires an arm that commits the term.
>
> A cap on **delta size** is not available at any price: the count is
> prover-dependent, so a stronger prover could trip it and lose a program that
> compiled, which is the break `[ENT-1]` forbids and L-G1 states.

### 9.3 The `§4.4` ledger row

| id | the term | buys | closure cost | soundness bill | verdict |
| --- | --- | --- | --- | --- | --- |
| **V8** | **a guarded fact — one atomic fact attached to a signed goal the branch itself decides, admitted only where that goal is derivable again** | **two idioms, not one**: the let-bound comparison flag re-tested in the same function (flagship A, nine sites) and the flag computed as a comparison, **stored in a record** and re-tested from the record (flagship B, four sites). Beyond them: `par_layout.wf`'s banded measure, the style-flag shape in `wfgrep.wf`'s line scan, and every audit scenario whose route menu currently reads "factor into a function whose `requires` states the correlation" | none in the ambient L0 domain — no new term kind, no new relation shape, no change to `[ENT-4]`. One new state component; one monotone fixed-point step per edge, bounded by the live entry count; and a doubled `§3.14` step 2 (D6, D7) | **small, and now argued rather than asserted.** Four laws, one six-paragraph proof, each paragraph naming its law; the delta facts are the arm's own `[ENT-3]` output, the key is a signed goal `[ENT-3]` already publishes, transport and kills are `[ENT-5]`'s unchanged rules | **bought**, with **P-S5B** carried as a prerequisite row and D4–D8 flagged |

Where it sits against the ledger's other rows: V1–V3 and V5–V6 were refused
because they widen `[ENT-2]`'s *vocabulary*, and each widening carries a per-row
re-verification bill over the operation table. V8 widens nothing. It changes
**when** a fact is available, not **what** can be said.

**The prerequisite is part of the price and is not this row's to hide.** P-S5B is
three additions to `[ENT-3.S5]`, one of which (item 3, `Bool` literal goals) is a
new fact source rather than an extension of one. It is small, it is independently
useful, and `r02`/`r08` show the checker already computes something equivalent
without exposing it as a goal.

### 9.4 The route menu, amended

`[ENT-6]`'s route menu (§3.12.1) currently reads, for this family: *"When the
residual is a correlation two values acquired together on one branch, the route is
to factor the dependent statements into a function whose `requires` states the
correlation and to call it from each branch."* That sentence is replaced by:

> When the residual is a correlation two values acquired together on one branch,
> and the condition of that branch has a goal origin, the route is **to test the
> same proposition again at the use**. "The same proposition" means the same
> `[ENT-2]` goal, not the same spelling: the same binding, the same place, the
> same field, or a compound whose decomposition or reconstruction reaches it —
> not a second binding of the same comparison, whose operands may have moved
> since, and not the same place named through a borrow at one site and directly
> at the other. **The branch's arms must also leave that proposition alone**: an
> arm that assigns the flag it branched on destroys the correlation, and the
> route there is to move the assignment after the continuation. When the branch
> commits a `Bool` place whose other edge leaves at the opposite value, testing
> **that place** is equally a route. When the correlation crosses a callable
> boundary, or the branch is a `match` arm over a user enum, the route is a
> verified `ensures` or a `requires` on a factored function.

Round 1's version said *"to test that same condition again at the use"*, and the
round-1 audit was right that `a01` is a writer doing exactly that. Round 2 fixed
that half and the round-2 audit was right that the fixed sentence *"tells the
writer to write exactly the second `if f`"* of `c01` — the menu walked the writer
into round 2's own refutation. The third sentence above is the missing half, and
`[ENT-5.G8]`'s *not formed* string reports it at the site. **The route menu and
the diagnostic have to agree, and this is the second round in which they did not.**

### 9.5 The prerequisite rows, collected

| row | what it is | who needs it | pinned by |
| --- | --- | --- | --- |
| **P-S5L** | 0108 §3.4's `[ENT-3.S5]` value-commit image on **local** destinations | flagship A's nine sites; every `set x = t;` correlation | `g05`, `g01`, `g02` (REJECT) |
| **P-S5B.1** | a per-field image from a **construction** | flagship A's two `style.columns` deltas; flagship B's `-h.has_body`, hence (a′)'s discharge | `a09` REJECT against `a10` ACCEPT; `r01` REJECT |
| **P-S5B.2** | a per-field image from a **value commit**, numeric and `Bool` | flagship B's `h.body_at` delta and its `+h.has_body` key | `r04` REJECT |
| **P-S5B.3** | a signed goal for the `Bool` literal constructions `True()` / `False()` | flagship B's commit image, which needs `+yes` at the commit | `r03` REJECT, with `r02`/`r08` showing an equivalent is computed but not exposed |
| **P-P0** | `[ENT-5.P0]`'s close-before-scope-kill order (0108 §3.3) | every arm-local `let t = …; set x = t;` spelling, which is both flagships | 0108's own `r7_closure_vs_kill.wf` |

None of these is `[ENT-5.G]`'s to write, and none is optional for the program it
carries. The honest statement of what this design delivers on its own is
`g03`-against-`g04`: the join is the entire loss, and `[ENT-5.G]` is what stops
losing it.

---

## 10. Falsifiers

**F-G1 (reach).** `[ENT-5.G]` over P-S5L, P-S5B and `[ENT-5.P0]` is predicted to
discharge all nine residuals of `probes/layout.wf` **and all four of
`audit/probes/a05_record_inline.wf`**, and to leave `tests/programs/` unchanged.
*Refuted if* fewer than seven of the nine or fewer than three of the four compile,
or if any program in `tests/programs/` that compiles today stops compiling, or if
any site needs a re-test the traces in §6.6 and §7.3 do not predict.

**F-G1′ (the refusals are refusals).** `c01`, `c02`, `b04`, `r09`, `b01`, `a01`
and `a04` must **stay rejections** under an implementation of the rule, and
`c01c`, `s02c`, `s03c`, `a08`, `b03c`, `s01c`, `g15` must stay acceptances, and
`s02` must **become** one (§8.1). *Refuted if*
any of the seven compiles or any of the six stops compiling. This falsifier is new
in round 3 and it is the one that would have caught all three shipped defects:
each of them was a program the round's own table predicted would reject.

**F-G2 (cost).** The bound is two orders of magnitude above the measured fact
figure on `layout.wf`; the entry count is within a factor of three of its bound;
the component is 0.6× the ambient L0 state at the measured point. *Refuted if* the
total guarded fact count at any point in `wfgrep.wf` (1,417 lines) or
`raw_deflate_vectors.wf` (863 lines) exceeds ten times the ambient closed L0 state
size at that point, if the live entry count anywhere exceeds `2(N_if + S_B)` —
which would be a bug, since §3.7 proves it cannot — or if compile time on either
moves by more than a small constant factor. **`§3.14` step 2's doubling (D6, D7)
is inside "a small constant factor" and is measured separately.** If F-G2 is
refuted the repair is **Fallback N1** (§9.2), not round 1's commit-pair narrowing.

**F-G3 (monotonicity is a property, not a hope).** *Refuted if* any program exists
that compiles with `[ENT-5.G]` switched off and fails with it switched on. Seeds:
the `[ENT-5.R]` fixed point, every `[IND]` certificate check in the corpus, any
`[ENT-5]` contradiction-sensitive judgment, and — **new in round 3, and the one
seed that is a live hazard rather than a residue** — a loop whose preheader bound
release *tightens*, testing D7 (§4.7.1, disclosure 3). Round 1 was refuted here by
the omission clause; under L-G1 the construction should not build.

**F-G4 (the release is not too generous).** Construct a program in which a flag is
reassigned, shadowed, aliased through a struct, written through a borrow, or
**written inside the branch's own arm**, and in which releasing would admit a
false fact. *Refuted if* any such program is accepted. The seed set is: `set f = …`
between the sites; a shadowed rebinding reusing the spelling; a `&uniq` callee
writing the struct holding the flag; a copy by a route the origin expansion does
not reach (`r05`); a delta term written by a callee's projected `writes` row; the
round-1 refutation (`a01`, `a04`); round 2's own finding (`r09`); the reading
defect (`b01`); and **the round-2 refutation — an arm that writes the flag its own
branch tests (`c01`, `c02`, and `b04`'s second branch), which is the seed to
generalise first**, because it needs no operand to move, no second binding and no
nesting: it is an ordinary `else` arm assigning a flag.

**F-G5 (the loop seam).** *Refuted if* a guarded entry that survives a loop-head
subtraction is ever released at a point where its delta is false on some
iteration, or if adding release to the body flow changes `[ENT-5.R]`'s limit
family on any program in `tests/programs/`. `r_p1_nested.wf` must remain a
rejection. Round 2 added a second condition — *"`ladder(F)` computed with step
(2′) is a superset of `ladder(F)` computed without it"* — which is **ill-posed as
stated**: a contradictory `E(@l)` makes the constant set either
implementation-visible or infinite, and release can *tighten* a bound and delete
the looser constant (§4.7.1). Round 3 replaces it with a condition over the
**observable**, which is well-posed under D7 and measurable without inspecting an
implementation's state representation:

> *Refuted if* the `[ENT-5.R5]` retention family computed **with** step (2′)
> omits, at any loop head of any program in `tests/programs/`, a fact the family
> computed **without** it contains.

That is the property F-G3 needs at this seam, it is what D7 buys, and it does not
depend on what "a constant appears in a state" means.

**F-G6 (the record route reaches the idiom, not just the example).**
`[ENT-5.G2]`(a′) is predicted to discharge the record-of-flags shape wherever the
flag is a `Bool` place the function itself decides and neither arm writes the
branch's own condition. *Refuted if* a corpus site of that shape needs a shadow
local anyway — that is, if `a06`-style plumbing survives the rule. Seeds:
`wfgrep.wf`'s scan-state struct and `par_layout.wf`'s band record.

**F-G7 (the price of D4 is the price we think it is).** After the interlock of
§8.1, the arm-write test is predicted to lose **no** corpus site except the `s03`
shape: a branch whose condition is not a `Bool` place, one of whose arms writes an
operand of the condition while leaving the condition's truth on that edge
unchanged. Everything else it refuses is either unsound (`c01`, `c02`), vacuous,
or recovered by clause (a′) (`s02`). *Refuted if* a corpus site loses a discharge
at formation that the arm-exit derivability condition would have admitted and that
(a′) does not recover. Two or more such sites is the signal to overrule D4 and
take the transplant; **zero** would say D4 is free and the flag should be
retired.

---

## 11. Open questions, each with a recommendation

**Q1 — Should there be any cap at all?** *Recommendation:* **no cap.** With keys
fixed at formation the entry count no longer depends on nesting depth, so `C_G`
bounded nothing and excluded a real shape. Removing one is a strict widening and
free under `[ENT-1]`; re-adding one later, if key-member storage shows up in a
measurement, is a separate and narrower decision.

**Q2 — Should signed goals be in the delta, not just L0 facts?** The excluded
shape is *"under `+wide`, the derived flag `+ok` also held"*, whose consumers are
`[FN-8]` call goals rather than subscripts. *Recommendation:* **buy it, in the
same batch, as a second sentence in `[ENT-5.G2]`** — "together with each signed
goal the arm state derives and the join does not". `[ENT-5.G6]` does not change by
a word (a signed goal has the same `[ENT-5]` support discipline, and L-G2 applies
to it unchanged), the size bound grows by the goal universe, which `[ENT-2]`
already bounds finitely, and the monotonicity argument is identical. Note what the
audits established: **Q2 does not fix the record idiom** — it would put
`+h.has_body` in the `[+room4]` entry's delta, which derives the flag *given* the
branch, and the program needs the converse. (a′) is the converse.

**Q3 — Should the key be the direct goal, or the whole goal-origin set?**
*Recommendation:* **keep the direct goal.** `r05` (REJECT) shows the expansion is
asymmetric, so a two-member key would be permissive in one direction only, and a
disjunctive key member would make the release condition a two-level structure.
Revisit if F-G1 turns up a corpus site that needs it.

**Q4 — Where does the family belong: `[ENT-5.G]` or a new `[ENT-3.S13]`?**
*Recommendation:* **keep `[ENT-5.G]`.** Formation is a join rule, transport is the
kill rule, release is a step of `[ENT-5.P0]`; siting it in `[ENT-3]` would make it
look like a fourth publisher, which §1 argues it is not. `[ENT-3]` gains one
cross-reference sentence: *"`[ENT-5.G]` re-admits facts this section's sources
already established; it establishes none of its own."*

**Q5 — Does the `value_if` delivery clause belong in `[ENT-5.G2]`?**
*Recommendation:* **keep it.** It is the clause that makes `g03`/`g04` — the
cleanest isolating witness — meaningful, and after round 2 it needs no separate
size accounting. It does need its own conformance case rather than riding on the
`if_stmt` cases, because it is the one place `[ENT-5.G]` reads a rule outside the
ordinary state. Note that it is also the clause that makes `c01` a *refutation
rather than a hypothesis*: `c01`'s delta comes from delivery, so it needs no
P-S5 prerequisite and the witness stands on today's checker.

**Q6 — Is `[ENT-5.R]`'s retention family per proof view?** **Taken, not deferred**
(D6, §4.7.3). The round-2 audit is right that deferring leaves a stated rule
contradicted by a stated pipeline. This is an `[ENT-5.R]` change made in an
`[ENT-5.G]` batch, and whoever owns `[ENT-5.R]` should see it as such; the
recommendation stands because the alternative is worse and because the change is
one sentence in step 2.

**Q7 — Should (a′)'s candidate set be "any `Bool` place datum" rather than "a
`Bool` place a commit in an arm writes"?** *Recommendation:* **keep the commit
restriction**, which is syntactic and keeps the entry bound at `2(N_if + S_B)`.
Widening it later is a strict widening and therefore free. Revisit if F-G6 finds a
corpus site where the flag is *delivered* rather than committed.

**Q8 — What about the borrow/publication asymmetry `s01` exposes?**
`[ENT-2]` 2886 resolves a call actual rooted at a borrow holder to its referent
datum; `[ENT-3.S1]` publishes the written tree. So `if deref(p).flag { … }`
establishes a goal that a requirement over `h.flag` cannot consume, today, with no
`[ENT-5.G]` in the picture (`s01` REJECT against `s01c` ACCEPT).
*Recommendation:* **report it as an `[ENT-2]`/`[ENT-3]` defect and do not fix it
here.** `[ENT-5.G]` inherits the asymmetry as a precision row (§9.1) and reports
it with the *same place, different goal* string; making S1 publish the resolved
form is a change to what a branch establishes, which is publisher 3's subject and
would need its own soundness argument about reborrows and shadowed holders.

**Q9 — Is F-G1 run before or after the repair lands?** **Before, and not as
written.** The draft has been honest for two rounds that no line of `[ENT-5.G]` is
implemented and that every acceptance in the flagship tables is a hand-execution
over a state the compiler prints no view of. Round 1 shipped a memory-safety bug
in a table of nine confident discharges; round 2 shipped a second one in the same
table. *Recommendation:* implement **`[ENT-5.P0]` step (2′) together with clause
(a)/(b) formation and the arm-write test** — the smallest fragment that makes
`c01` a machine verdict rather than an argument — and run **F-G1′** (§10) before
F-G1. It is the cheapest thing that would stop this recurring, and it is smaller
than the fragment the round-1 audit suggested, because the arm-write test is
syntactic and needs no delta at all to refuse `c01`.

---

## 12. Probe ledger

Fifty-five files across three directories, all compiled against the
`batch/0106-claim-model-design` worktree compiler, unmodified. `./run.sh
probes/*.wf`, `./audit/run.sh` and `./audit2/run.sh` reproduce every verdict, and
all three were re-run for this round with no change.

**`probes/` — the design's own, thirty-three files.**

| group | files | what they establish |
| --- | --- | --- |
| the isolating pair | `g03` (REJECT), `g04` (ACCEPT) | the join is the entire loss; every other premise the read needs exists today |
| the commit-image separation | `g01`, `g02`, `g05` (REJECT) | `set` publishes nothing today, so the flagship's residuals need P-S5L **and** `[ENT-5.G]` |
| the guardable class | `g06`, `g07`, `g10`, `g11` (ACCEPT) | parameters, struct fields, call results and copies all publish a usable signed goal |
| the compound routes | `g08`, `g09`, `g12` (ACCEPT) | reconstruction, decomposition and negation work today |
| the projection gap | `g13`, `g14` (REJECT) against `g15` (ACCEPT) | L0 does not back-derive a signed goal for a second binding — **the reason `a01` is refused** |
| the price | `g16` (ACCEPT) | the re-test workaround compiles, so the bill is branches and readability |
| the flagship | `layout.wf` (REJECT ×9), `layout_neutralised.wf` (ACCEPT) | the whole rejection surface of a layout-shaped program is flag correlation |
| round 2's nine | `r01`–`r09` | P-S5B's three items and their controls; the one-directional expansion; field-granular kills; both halves of the size maximiser; and `r09`, the carry-out witness |
| **round 3's six** | `s01`, `s01c`, `s02`, `s02c`, `s03`, `s03c` | the borrow/publication asymmetry; the interlock of L-G3's two discharges; and D4's price |

**`audit/probes/` — the round-1 audit's thirteen**, not superseded: `a01`/`a04`
are the round-1 refutation and are re-executed as rejections in §8.4; `a05`–`a08`
are flagship B and its isolating pair; `a09`–`a12` pin P-S5B.1, its contrast,
field-termhood and the size maximiser.

**`audit2/probes/` — the round-2 audit's nine**, likewise: `c01`/`c01c`/`c02` are
the round-2 refutation and are re-executed as refusals in §8.1; `b01` is the
reading witness; `b04` the flag-cleared-between attack; `b02`/`b02c` the
signed-goal half of the `&uniq` transport; `b03`/`b03c` the two-instance control.

**Round 3's four, in detail.**

| probe | verdict | what it establishes, and what it changed |
| --- | --- | --- |
| `s01_borrow_goal_identity.wf` | **REJECT** `[FN-8]`, `instantiated_goal: "h.flag"` | **added §9.1's row and `[ENT-5.G8]`'s fourth string.** A branch written through a borrow establishes a goal a requirement over the same resolved place cannot consume |
| `s01c_borrow_same_expression.wf` | **ACCEPT** | its isolating control: the borrow is live at the same point and the *argument* is written through it, so the asymmetry is on the **publication** side |
| `s02_redundant_flag_write.wf` | **REJECT** `[OP-4] x < len(deref(data))` | **corrected this round's own reading of D4.** An arm that writes its branch flag without changing it is refused by (a)/(b) and **admitted by (a′)**, so it compiles under the drafted text — at the cost of P-S5B item 3 |
| `s02c_joined_control.wf` | **ACCEPT** | its control: with the fact joined the program compiles, so `s02`'s rejection today is the release and nothing else |
| `s03_operand_write_preserving.wf` | **REJECT** `[OP-4] x < len(deref(data))` | **priced D4.** A comparison condition whose arm writes an operand while preserving its truth: sound under the derivability discharge, refused by the syntactic one, and out of (a′)'s reach |
| `s03c_joined_control.wf` | **ACCEPT** | its control, with the fact joined |

**What the ledger does not establish.** No line of `[ENT-5.G]` is implemented, so
every acceptance claimed in §6.6 and §7.3 is a hand-execution of the drafted rule
over a state the compiler prints no view of. That gap let round 1 ship a
memory-safety bug in a table of nine confident discharges and let round 2 ship a
second one in the same table. Every *refusal* claimed in §8, by contrast, is a
program that rejects today and whose premises are separately machine-checked —
which is why the refutations were constructible and the acceptances were not
checkable. §11's Q9 is the recommendation that changes this, and F-G1′ is the
falsifier that runs against it.

---

## 13. The round-3 ledger, appended to the decisions

| # | change | driver | evidence |
| --- | --- | --- | --- |
| 1 | **the four laws (§2)** are stated first and every clause is derived as an instance of one; §2.5 maps clause to law | three rounds, three restated-instead-of-re-derived soundness paragraphs, three out-of-bounds reads; `DESIGN.md` §3.8's precedent | `c01`, `r09`, `a01` |
| 2 | **L-G3 and the arm-write test**: `[ENT-5.G2]`(a)/(b) form only when neither arm disturbs `support(G)`. **The refutation repair** (**D4**) | audit-2 §1, P1 | `c01`, `c02` REJECT; `c01c` ACCEPT; cost verified per continuation on both flagships (§6.2, §7.1) |
| 3 | **L-G4**: (d)'s condition is over *every kill event on every path through the arm*, never *the reaching edge's events*, quoted verbatim at three sites | audit-2 §2 | `b01` and `r09` derived side by side (§8.2) |
| 4 | **L-G1 weakened to monotonicity** (**D5**); `[ENT-5.G7]`'s determinism paragraph rests on `[ENT-4]` 3033–3035 instead of on "syntactic" | audit-2 §4.4, P2 | (a′)'s own text; no `[CLM-2]` divergence is constructible either way |
| 5 | **L-G2**, the continuous-liveness lemma, stated once and cited from union, carry-out and the `⊥` argument | audit-2 §3.2, R4 | §6.5's E11/E11′ union; `[ENT-5.G6]`(6) |
| 6 | **Union gains a consistency step**: an inconsistent union is `⊥`, sound and monotone by L-G2 | audit-2 §4.2 | `b04`, which round 3 refuses twice before the question arises (§8.3) |
| 7 | **§3.14 step 2 goes per proof view** (**D6**), with its cost stated | audit-2 §7.3, P3 | §4.7.3; the `s4_blinded` walk of flagship B |
| 8 | **`ladder(F)`'s `K` is the union of the with-(2′) and without-(2′) flows** (**D7**); a contradictory `E(@l)` contributes no constant; "enlarges" replaces "does not introduce" | audit-2 §7.1 plus round 3's own third disclosure | §4.7.1; F-G5 re-posed over the retention family |
| 9 | **`[ENT-5.R3]`'s guarded slot is deleted**; the head subtraction is part of the continuing-kill step, which is true by construction | audit-2 §7.2 | `[ENT-5.G5]`: (2′) does not run at a head, so retention never reads the head's component |
| 10 | **`[ENT-5.G8]` gains three strings** — *not formed*, *not carried*, *same place, different goal* — and the killed-entry history is **reconstructed, not carried** (**D8**) | audit-2 §8, P4; the cost/bound finding | `s01`/`s01c`; the bound in §3.7 is now true as written |
| 11 | **`[ENT-5.G1]` names its `[ENT-2]` dependency** | audit-2 §4.3, R7 | `b03` REJECT, `b03c` ACCEPT |
| 12 | **two measurement labels fixed**: 251 as formed against ≈245 live at line 114; the zone cost is per **point**, not per function | audit-2 §5.2 | §6.7, §9.2 |
| 13 | **§6.2 corrects round 2's `@98` row**: `if more { } else { break @scan; }` has one reaching edge, so it forms nothing — it is not an entry killed later | round 3's own execution of the flagship | `layout.wf` 95–98; the same sentence that makes flagship B's @42 free |
| 14 | **the route menu gains its third sentence** — the arms must leave the proposition alone | audit-2 §1.5: round 2's menu walked the writer into `c01` | §9.4 |
| 15 | **F-G1′ and F-G7 are added**; F-G5's second condition is re-posed over the retention family; F-G3 and F-G4 gain the round-3 seeds | the pattern: each round's refutation was a program its own tables predicted would reject | §10 |
| 16 | **six probes appended** (`s01`, `s01c`, `s02`, `s02c`, `s03`, `s03c`) | the two unpriced items the round-2 audit named — and `s03`, which this round needed after executing `s02` against the whole rule instead of one clause | §5.2, §8.1, §12 |

**What is unchanged from round 2 and why it survived a second audit:** D1 (one
release route), D2 (the direct goal's support, quoted at five sites), the
key-indexed map with union at formation, keys fixed whole at formation, the
deletion of the omission clause and of `C_G`, the head subtraction's two
granularities, D3 and (a′), P-S5B, the per-view component, `[IND-7]`'s split, and
the retraction of the commit-pair narrowing. The round-2 audit attacked all of
them and refused its own attacks; §8.5 records the outcome per witness.

**The one process sentence worth keeping.** Round 2 wrote that the lesson is *"a
rule this invisible must be sound by construction, because the writer has no way
to audit it and the reviewer has nothing to review"* — and then shipped a
formation clause whose soundness paragraph asserted a premise the clause did not
check. The narrower lesson, which is what §2 encodes: **every clause that names a
signed goal as a key must state, in the clause, what makes that goal record the
arm.** Under round 3 all four clauses do, and L-G3 is the law that makes it
impossible to add a fifth that does not.
