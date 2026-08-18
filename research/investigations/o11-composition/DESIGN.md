# O11 Boolean-goal composition — candidate design

> Superseded 2026-08-18: applied at v0.31 activation (eb8e8634). Historical design evidence; the active spec owns the normative text.

Batch 0070, workstream W5 item "O11 boolean composition (from its four
recorded findings)". Authorized by the ACTIVE `docs/current-plan.md`
(owner expansion, 2026-08-17). This document is design evidence for the
v0.31 candidate; it changes no active-specification byte and no protected
conformance material.

## Verdict

**Closable as a v0.31 candidate; not activatable in this batch.** The four
recorded findings (`governance/APPROVALS.md`, 2026-08-09) are sequencing,
approval, and blast-radius constraints, not open soundness questions. The
sound decomposition rule is stated below, its classic sign asymmetry is
exact, and the goal-origin machinery implements it tonight as
acceptance-dark candidate metadata: computed and retained on the live
path, established as facts by nothing. v0.30 acceptance is byte-identical;
the gate stays green. Activation requires the owner's exact-byte v0.31
approval carrying the one ruled conformance verdict flip in the same
sitting (finding 2).

## The recorded constraints, and how each is honored

1. **O11 is not an approved specification change** (ruling 2026-08-09
   (1)). It is drafted here from nothing, as its own delta
   (`SPEC-DELTA.md`), and takes its own owner approval. Nothing in this
   batch establishes a decomposed fact.
2. **O11 flips a declared conformance verdict** (ruling (2)):
   `tests/conformance/cases/ent3-neg-bor-no-comparison-origin.wf`,
   declared `reject` on OP-4, accepts under the correction. The ruled
   disposition stands: rewrite as a positive case whose subject is that
   the conjunctive read discharges the guard, presented to the owner with
   the byte approval in one sitting. This batch touches neither the case
   nor the manifest; the flip set re-verified below is exactly this one
   case.
3. **The de-pairing ruling** (2026-08-09, "O11 comes out of the ENT-5
   activation") kept O11 off a vehicle chosen for smallness. The owner's
   2026-08-17 expansion routes every deferred item into the single v0.31
   candidate, superseding that sequencing for this vehicle. Flagged for
   the morning packet so the owner re-affirms the stacking knowingly.
4. **Blast radius includes CLM-2 and must be re-measured at drafting
   time** (ruling (4)). Re-measured against the active v0.30 bytes below:
   **six carrier sentences change** (FN-8 x3, ENT-3.S4 x1, ENT-4 x2) and
   **two sentences stay literally true but gain a clause** (CLM-2's
   worked example, ENT-3's origin exclusion list) — four rules touched
   in all. Neither previously carried figure (5 or 6) was forwarded.

The roadmap's recorded trigger — "a real program whose discharge needs a
composed Boolean goal" — has recorded evidence:
`research/investigations/obligation-discharge/ACCEPTANCE.md` driver 2:
`decode_length` in `raw_deflate_dynamic.wf` guards with
`bor<Bool>(ilt(symbol, 257), ige(symbol, 286))`, the `False()` arm
establishes nothing, and both table subscripts need the
`length_symbol_in_tables` claim; micro-controls D2a/D2b reproduce it in
four lines. Driver 3 (D3a/D3b) shows the same cap on FN-8's one final
check: a `band`-folded two-sided requirement threads nothing. The
acceptance review already named the candidate monotone fix ("admit
`band`/`bor` of comparisons as a conjunction/disjunction of relations").
The frozen sha256 acceptance row (4 claims against the simulated 3,
"because a two-sided bound is not expressible as one claim") is the same
pressure measured.

## The rule (candidate, exact)

**Signed Boolean decomposition.** For one concrete goal G and sign s,
define decomp(sG) structurally over G's typed expression tree:

- decomp(+G), root `band(A, B)`: {+A, +B} ∪ decomp(+A) ∪ decomp(+B)
- decomp(−G), root `bor(A, B)`: {−A, −B} ∪ decomp(−A) ∪ decomp(−B)
- decomp(+G), root `bnot(A)`: {−A} ∪ decomp(−A)
- decomp(−G), root `bnot(A)`: {+A} ∪ decomp(+A)
- every other root (`bxor`, `eeq`, `ene`, a comparison call, a datum, any
  non-Boolean operation): decomp(sG) = ∅

Whenever [ENT-3] establishes a signed goal fact ±G — S1 at either arm
entry or edge, S2 and S3 on the normal continuation, S4 at body entry —
it also establishes every member of decomp(±G) at the same point and
event. Each member is itself one concrete goal (a subtree of G, already
inside FN-8's structural identity); each member whose complete root is
one comparison call of comparison-origin shape (a) over admitted operands
independently establishes its exact relation under `+` and the relation's
exact L0 negation under `−`. Each member's ENT-5 support is the ordinary
signed-goal support of its own expression — the resolved places its
subtree reads — so kills, scope exits, joins, and the loop rule apply to
members with no new lifetime machinery. Decomposition is a finite
structural walk: no algebraic rewrite, no De Morgan rewriting (the
`bnot`-over-`bor` case falls out of recursion, not rewriting), no search,
and no new term forms.

**What is deliberately not added:**

- **No composition.** Children never derive a parent. `+A` and `+B` never
  yield `+band(A, B)`; a conjunctive FN-8 goal is still discharged only
  by whole-tree evidence. This keeps
  `clm3-neg-generated-wrapper-check.wf` (reject FN-8: "U does not compose
  that atomic goal") at its declared verdict — adding composition would
  flip a second protected verdict beyond the one ruling (2) already
  disposed, and would need pair-search the fragment's deterministic
  no-search doctrine excludes.
- **No comparison-origin extension.** The single-comparison definition
  (shapes (a)/(b)) and its exclusion sentence stay exact. Decomposed
  members project through the existing per-goal projection, not through a
  widened origin.
- **No ENT-4 closure rule.** Decomposition happens at establishment.
  ENT-4's opaque component still receives no closure: for a Boolean-root
  parent, derivable implies established (band/bor/bnot have no
  projection, so ±parent is derivable only when present), so
  establishment-side decomposition loses nothing a query-time rule would
  find — and it keeps derivability non-recursive.
- **No CLM-2 judgment extension.** A `band`-rooted claim predicate still
  has no comparison origin and remains neither redundant nor refutable in
  this version. Refuting a composed claim from one refuted conjunct is
  sound (a false conjunct forces the conjunction false at every
  evaluation the support discipline admits) but is a child-to-parent
  step; it is recorded as a weighed-out follow-up, not silently included.

## Soundness against the S-rule negation semantics — the exact asymmetry

`band`, `bor`, `bnot` are pure, total, non-trapping `(Bool,…) -> own
Bool` operation rows inside a goal expression that is wholly pure and
total, so for the one evaluation a signed fact records, operand values
are well defined and:

- `band(A, B) = true  ⟺ A = true and B = true` — the **positive** edge
  decomposes; the negative edge yields only `¬A ∨ ¬B`.
- `bor(A, B) = false ⟺ A = false and B = false` — the **negative** (else /
  `False()`) edge decomposes; the positive edge yields only `A ∨ B`.
- `bnot(A) = s ⟺ A = ¬s` — both edges flip.
- `bxor` constrains neither child pointwise on either edge.

The classic asymmetry, stated precisely: each lattice connective
decomposes on exactly one sign, and the other sign's content is genuinely
disjunctive — inexpressible per-child in a conjunctive fact state.
Granting either child there is unsound by two-line counterexample
(`A=true, B=false` gives `−band` with `−A` false, and `+bor` with `+B`
false). The disjunctive information is not dropped unsoundly: it is
retained as the opaque ±G exactly as today, available to exact-tree
rediscovery, and the fragment adds no case split to exploit it. Ruling
(2)'s worked example is the sound direction: the else edge of
`bor(below, above)` establishes `−below` and `−above`, whose projections
negate to `symbol >= 0` and `symbol < 4`, and `len(table) = 4` is the
ENT-2 implicit fact that closes the OP-4 obligation.

Residual soundness points:

- **Support and kills.** A member's reads are a subset of the parent's;
  each member carries its own ENT-5 support, so a write to one conjunct's
  operand kills that member and the parent while correctly retaining the
  sibling. This is the existing signed-goal discipline, unchanged.
- **Joins.** Members join by the existing intersection. A member can
  survive a join where its parents differ (both edges establish `+A`
  under different conjunctions) — sound, since each edge established it
  soundly, and strictly more precise than parent-only retention.
- **Contradictory members.** `+band(A, bnot(A))` establishes `+A` and
  `−A` on an edge unreachable in truth; the fragment's existing
  contradiction posture (the ruled O8 treatment of
  unreachable-in-truth states) already governs this. No new rule.
- **Views.** Decomposition rides each establishment event in whatever
  view it fires: blinding a claim (unasserted) or S4 (S4-blinded) blinds
  that establishment's members with it. No view-specific text.
- **Finiteness and determinism.** Members are subtrees of written
  conditions after the existing finite expansion, so the finite goal
  universe stays finite and the walk is deterministic and monotone.

## Acceptance impact inventory (for the owner packet)

O11 is *not* purely widening:

- **Widens:** OP-4/ENT-6 obligations and FN-8 requirement goals discharge
  from decomposed members and their projections (rejection → acceptance
  for guard shapes like the `bor` early-exit and `band`-folded checks).
  An FN-8 goal moving unproved → refuted keeps the same rejection verdict
  with a better diagnostic.
- **Narrows:** a claim whose comparison predicate is contradicted by a
  newly established member's negated projection moves unproved → refuted,
  and CLM-2 makes a refuted claim a hard error; a claim newly derivable
  from members draws a new non-rejecting redundancy advisory. Both are
  the fragment working as specified on programs whose claims contradict
  or restate their own guards. Corpus sweep (2026-08-17, v0.30 corpus,
  483 manifest rows): exactly seven cases use `band`/`bor`/`bnot`, none
  of them contains a `claim` statement, five are accept/run cases whose
  discharge only widens, and `clm3-neg-generated-wrapper-check.wf`'s
  subject is the preserved no-composition direction — so the corpus flip
  set is exactly the one case ruling (2) disposed,
  `ent3-neg-bor-no-comparison-origin.wf`.

## Specification blast radius — re-measured 2026-08-17 against v0.30

Carrier sentences that change (line numbers in the active
`spec/kernel-spec.md`, SHA
`5ed210190737b2aa53a91dc901f07d02344669eeb6d6660224602872331204d1`):

1. **[FN-8] L1184** "…one indivisible goal: evidence for its children
   establishes nothing about the whole, and evidence for the whole
   establishes nothing about a child." — second half becomes false;
   first half (no composition) remains normative and is kept.
2. **[FN-8] L1185** "no Boolean subtree receives such a projection." —
   false under O11; decomposed subgoals project.
3. **[FN-8] L1199** body-entry sentence — extended: the body starts with
   the S4 goal, its decomposition members, and their projections.
4. **[ENT-3] L2706 (S4)** "No child of any other goal is established." —
   replaced by the decomposition statement.
5. **[ENT-4] L2758** "…receives no closure, decomposition, composition,
   or implication rule." — reworded: decomposition is an establishment
   rule; the derivation stratum still has none.
6. **[ENT-4] L2760** "Deriving the two children of a Boolean operation
   never derives its parent, and deriving the parent never derives either
   child." — first half kept verbatim (no composition); second half
   reworded to the establishment/derivation split.

Sentences that stay true but gain a clause so they cannot be read against
the new rule:

7. **[CLM-2] L2578** — "a `band` result" still has no comparison origin
   and the claim is still neither redundant nor refutable; the sentence
   gains the clarification that S3 nevertheless establishes the
   predicate's decomposition members (finding (4)'s named CLM-2 site).
8. **[ENT-3] L2679** — the origin exclusion list stays exact; gains a
   forward reference to establishment-side decomposition.

One new paragraph in ENT-3 (after the goal-origin-set paragraph, before
"The sources are:") states the rule once for every signed-goal
establishment, so S1/S2/S3/S4 need no per-source restatement. Exact
candidate text: `SPEC-DELTA.md` beside this file. No grammar production
changes, so the native grammar verifier has nothing new to verify.

Knock-ons the activation batch must carry (supersede-in-place, same
change):

- Conformance: the ruling-(2) rewrite of
  `ent3-neg-bor-no-comparison-origin.wf` (protected; owner sitting).
- `mcts_mem` paired moves at activation, not before:
  `checks-and-proofs/requires-entry-contract` Item 3 ("Goal evidence is
  atomic: no … decomposition …") and
  `checks-and-proofs/obligation-discharge` Item 13 ("…without
  decomposing the goal") become false the commit the rule goes live.
- The obligation-discharge candidate-review sha256 bucket arithmetic
  ("4 claims") reverts to the conjoined 3-claim shape for any *future*
  accounting; frozen research files are not edited.

## Implementation staged tonight (acceptance-dark)

The recorded one-path doctrine (`check_semantics_dark`: "acceptance
behavior has exactly one path") forbids a second acceptance mode, even
test-only, and `UnsupportedSemanticFeature` is for spec-valid programs
the compiler cannot handle — the opposite of O11, whose programs v0.30
genuinely rejects. The honest boundary is therefore: the goal-origin
machinery computes each establishment's decomposition members and their
projections on the live path and retains them as candidate metadata on
`FunctionEntailment`; nothing establishes them as facts. Tests pin both
directions — the members and projections are exactly the rule above, and
every v0.30 verdict, including the ruled flip case's `reject`, is
unchanged. At activation the diff is establishing the already-computed
members at the already-identified four `establish_goal` sites.

Evidence in this directory, run 2026-08-17 with `whitefootc` at the
candidate branch tip:

- `probe-bor-guard.wf` (the ruled flip shape) rejects
  `[OP-4] UndischargedBoundsObligation { residual: "symbol < len(table)" }`.
- `probe-band-check.wf` (D3b-shaped conjoined check) rejects
  `[OP-4] UndischargedBoundsObligation { residual: "low < len(table)" }`.

Both become accepted, runnable programs once decomposition activates;
each probe's `doc` states its expected post-activation disposition.
Ordinary regression tests in both directions live in
`compiler/src/semantic/tests/boolean_composition.rs`.

One sharp edge observed while building the caller-discharge test, spec-
correct and pre-existing, recorded for the writer doctrine rather than
O11: a caller that literal-binds the guarded operands (`let low = 1_u64;
… check band(…)`) cannot discharge the conjunctive requirement, because
goal-origin expansion runs to its literal fixed point (`low` → `1_u64`)
while FN-8 substitution retains the place datum for the actual; the
established tree and the instantiated goal differ exactly at those
leaves. Parameter-shaped operands do not expand and discharge normally.
O11 neither causes nor fixes this; it is the two-member origin-set
design working as specified.

## Rejected alternatives (weighed tonight)

- **Extend comparison origin to Boolean trees** — rejected: it would
  overload one definition with sign-dependent meaning, force CLM-2's
  redundancy/refutation judgment open in the same change, and touch more
  carrier sentences than establishment-side decomposition; the
  single-comparison cut stays.
- **Composition (children → parent)** — rejected above; flips a second
  protected verdict, needs pair-search.
- **ENT-4 query-time decomposition closure** — rejected: equal power for
  Boolean roots (derivable ⟹ established), but makes derivability
  recursive and support attribution retroactive.
- **A test-only candidate acceptance mode** — rejected against the
  recorded one-path doctrine.
- **Extending CLM-2 refutation to composed claims** — deferred, recorded
  above; sound but child-to-parent and acceptance-narrowing beyond the
  measured flip set.

## Removal condition

This directory is superseded by the v0.31 activation packet: at
activation the delta text moves into the approved candidate, the probes
graduate into ordinary compiler tests or the rewritten conformance case,
and this dossier is retired to the investigation's archive disposition in
the same change.
