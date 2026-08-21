# Claim residual canonicality

Status: RESEARCH ONLY. This document records the 2026-08-21 owner direction,
the feasibility argument, the current implementation and corpus evidence, and
the boundary of a possible implementation plan. It is not a specification and
authorizes no compiler, protected-conformance, or gate change. The active
language remains v0.33 at `spec/kernel-spec.md`.

## 1. Conclusion

The direction is coherent, but its strongest honest statement has two parts.

Whitefoot can mechanically make the *structural role* and *safety boundary* of
every accepted claim exact:

- the normative checker neither proves nor refutes the predicate;
- the predicate is a total, deterministic, observational proof expression;
- the predicate has one versioned canonical contribution normal form whose
  every component is checker-unknown and individually load-bearing;
- the claim occurrence is also individually necessary for at least one fixed
  source-admission proof;
- the written `because` record has the required review structure; and
- the accepted claim is never elided and is evaluated at every dynamic reach in
  every build mode.

Whitefoot cannot have the same fast checker decide perfectly that an arbitrary
human derivation is true. If that checker can validate the derivation, it has
proved the predicate and the claim must instead be rejected as redundant. The
truth of a surviving residual is therefore an explicit human/AI/offline-prover
review obligation. This is not a safety hole: a mistaken claim traps before its
fact can authorize the following partial operation. It is the epistemic
boundary that makes a claim a claim.

The design is viable if it says this boundary plainly. It is not viable if it
pretends that a nonempty prose string, an SMT timeout, or occurrence in one
selected proof establishes human derivability.

## 2. Project law

The proposed author law is:

> A claim is neither an assertion, an abort, a test oracle, nor a conditional.
> It is admissible only when its predicate is universally true at that program
> point, every fact in its canonical contribution normal form is unknown to the
> normative checker and individually necessary, the author records the
> derivation in `because`, and the resulting claim is necessary to
> discharge at least one specific source-admission proof obligation. If the
> predicate may legitimately be false, the program must express that possibility
> with ordinary control flow: a branch, match, or loop transfer, or a typed
> result, return, or exit status at the appropriate boundary.

This law has immediate consequences:

- `claim False()` is never an intentional-abort form. It is false on every
  reachable execution and must be replaced by explicit program behavior.
- `claim True()` is redundant and must be removed.
- a check of input validity, an expected error, a test expectation, an
  impossible-arm oracle, or a debug invariant is not a claim unless the
  condition is in fact universally true and satisfies every rule below;
- a true but unused theorem is not a claim, because it fills no checker gap;
  and
- a claim that becomes unnecessary after checker strengthening is a source
  upgrade point. The author or AI edits the source and rechecks it. No compiler
  pass silently removes the accepted runtime check.

## 3. Three distinct authorities

The design needs three names because conflating these authorities recreates
`assert`, `expect`, or solver-dependent compilation.

### 3.1 Normative fast checker

The ordinary checker is the closed, deterministic source-acceptance authority.
It classifies the exact predicate before the claim contributes its own S3 fact,
checks the proof-expression restriction, and tests individual downstream
necessity. Optional solvers, profiles, targets, optimizer facts, timeouts, and
runtime resources cannot change this result.

"Checker cannot derive" means the versioned normative derivation system lacks
the rule. If the specification already requires the derivation and one compiler
implementation misses it, that is a compiler defect, not a legal claim gap.

### 3.2 Executed claim boundary

An accepted claim is never elided and evaluates and checks its predicate at
every dynamic reach in every build mode. Only its successful continuation
establishes S3. A failed claim emits the one mandatory trap record and aborts
before any proof that consumes S3 can execute. There is no accepted
`Eliminated` disposition.

### 3.3 Offline proof audit

A human, AI-assisted review, or slow prover checks the mathematical derivation
recorded by `because`. Its result is bound to the exact source and checker
identity. It does not change ordinary compilation and does not remove runtime
execution. If an audited proof pattern later becomes part of the normative
checker, the original source claim becomes a redundancy error and the author
removes it.

## 4. Exact source judgment

For one concrete inhabited function instance and one claim occurrence `c` at
location `l`, let:

- `P` be the exact typed predicate;
- `D(P)` be its direct evaluated GoalExpression snapshot, `S(P)` its
  support-canonical snapshot-frontier expansion, and `F(P)` its fully
  structural still-valid ordinary-let expansion;
- `Contrib(P)` be the versioned, ordered canonical contribution normal form
  derived only from S;
- `C_l` be the normative state after safe evaluation of `P`, but before `c`
  establishes S3;
- `Full` be the ordinary complete proof view;
- `Eligible` be the fixed set of occurrences that have passed every
  pre-residual machine check: proof-predicate shape/effects, FN-1 reachability and
  D/S/F exact lifecycle, component lifecycle, contribution consistency, S
  reconstruction and D materialization, and the five-field `because` schema;
- `Full-minus(c)` be the same flow and the same executed source, with every
  other occurrence in `Eligible` contributing S3 and only `c`'s S3
  establishment suppressed;
- `Full-minus(c,a)` additionally mean the same flow with c's runtime predicate
  and effects retained but its S3 source event emitting the basis without
  component-specific source event `a`; ordinary closure is
  rerun from all unmasked sources, so facts depending only on a disappear and
  independently rederived facts remain; every run, whether or not a terminal
  root changes, requires exactly the same PRV-2/PRV-3 failure set, so any provenance
  delta is a compiler consistency failure rather than a residual witness;
  and
- `AdmissionRoots` be the closed set of mandatory source-acceptance proofs.

For a one-component claim, the component and whole masks suppress the same
single S3 event. They are therefore one definitionally identical fresh run with
two evidence roles, not two semantically distinct worlds.

The fast lifecycle matrix is:

| Pre-claim result | Source result |
|---|---|
| `C_l` is contradictory | vacuous classification; never a proof of `P` |
| one exact D/S/F image proves both signs | compiler consistency failure, not a source classification |
| distinct equivalent D/S/F images prove opposite signs | vacuous source classification |
| non-contradictory `C_l` proves `P` | hard source error: redundant claim |
| non-contradictory `C_l` proves `not P` | hard source error: refuted claim |
| neither sign is derivable | `Unknown`; necessary but not sufficient for admission |

An `Unknown` claim is a mechanically admissible residual candidate only when
conditions 1, 2, 3, 5, and 6 hold; condition 4 is the additional approval rule:

1. `P` passes the proof-expression judgment in section 6 and has one unique
   `Contrib(P)` under the rules below.
2. Every signed component in `Contrib(P)` and its sound S/F lifecycle
   manifestations is unknown on both signs in `C_l`; adding all components is
   consistent, reconstructs S, and materializes D through retained normative
   derivations, while F remains lifecycle-only.
3. `because` has exactly the five labelled, nonempty fields specified in
   section 7; this is a structural machine check only.
4. The author asserts, and offline review accepts, a derivation of `P` from
   premises valid at `l` without using `c` itself or a later fact; review
   validates the semantic truth of all five fields, confirms that `P` is not
   semantically stronger than the missing lemma, and confirms that its
   consumers are authentic program obligations rather than artifacts inserted
   to justify the claim.
5. For every `a` in `Contrib(P)`, there is an admission root `R_a` after `c`
   for which `Full(R_a)` succeeds and `Full-minus(c,a)(R_a)` does not; at least
   one root also fails under whole-occurrence `Full-minus(c)`, and at least one
   c-dependent predecessor lineage for each selected root has c in its dynamic
   prefix.
6. Every selected complete derivation reaches the exact component/claim S3
   event, its root state is non-contradictory, and its c-dependent ancestry has
   no contradictory/ex-falso predecessor. At a join, every reachable
   predecessor contributing to the root independently has non-contradictory,
   non-explosive legal support; c need dominate only its c-dependent lineage,
   not every mutually exclusive predecessor.

Conditions 2, 5, and 6 make every canonical contribution and the occurrence
mechanically residual. Condition 4 is
the deliberately external truth review. The compiler can publish a
structurally qualified candidate without pretending it has decided condition
4; a human-approved Whitefoot program may contain only claims satisfying all
six conditions. Every compiler-accepted candidate remains retained at runtime.
Structurally unreachable statements are already FN-1 source errors and never
enter this lifecycle.

If any occurrence fails a pre-residual machine check, the unit reports the
deterministic earlier error and does not run residuality. There is no
counterfactual baseline in which an early-invalid claim supplies S3 to another
claim.

A contradiction created only after adding `P` is not a clever proof of every
later obligation. It means `P` is incompatible with the checker's established
state and is a source error, even if the current negative-goal query is too
narrow to name the exact refutation.

`Contrib(P)` is not a syntactic-leaf rule. It recursively extracts only sound
conjunctive information in the signed normative goal vocabulary: positive
`band`, negative `bor`, and `bnot` sign inversion are the first required cases.
A positive disjunction whose truth does not imply either child may remain one
exact root component. Every admitted Bool operator needs an operator-specific
canonical conjunctive basis: xor and equivalence cannot default to singleton
merely because the implementation lacks their partial-known rules.
Comparison/equality and `.defined` use one specified normative relation/domain
basis. S supplies the support-correct contribution identities; F supplies only
equivalent or positive-only lifecycle manifestations, and D is a reconstruction
target rather than another permission. Masking suppresses only the selected S3
source event, after which dependent reconstructions disappear naturally.
The sequence is ordered and deduplicated by normative fact identity so claim,
component, instance, and root witness selection is deterministic. Directional
bounds retain ordered endpoints. Disequality identity is unordered, so
`a != b` and `b != a` deduplicate, while the first left-to-right source
occurrence fixes rendering and component ordinal.

The same finite checker closure must reconstruct S from `C_l` plus all
components and materialize D from that proof. It may introduce only exact parent trees already present in the
finite goal universe, using fixed rule rank and minimum non-cyclic derivation
depth. It does not synthesize arbitrary formulas. If normalization is ambiguous,
support/kill cannot be preserved, S cannot be reconstructed, or D cannot be materialized, that
predicate shape is not yet admissible for claim; the compiler may not fall back
to publishing an unchecked bundle.

The runtime-success S3 event establishes the canonical S-derived contribution basis
directly; it does not first establish parent P and then decompose it. S
reconstruction and D materialization are ordinary normative ENT closure rules used identically in
complete, U, B, claim-free, and facts-off checking. This avoids a claim-specific
proof authority and prevents a retained parent-to-child-to-parent ancestry
cycle.

### 4.1 Closed admission-root set

The initial closed set should contain only proofs that can decide source
acceptance:

- proof-required operation, allocation, bounds, and system-range obligations;
- ordinary call requirements;
- mandatory complete-view proofs of selected function postconditions;
- the corresponding successful provenance gates where a protected obligation
  or call is involved; and
- any later proof family explicitly added to this closed list by the
  specification.

Optimizer facts, S7/S11/S12 observation alone, derivation metrics, claim
lifecycle roots, CLM-3's structural claim inventory, effect exhibition, runtime
test oracles, and facts used only to prove another claim do not count. A claim's
own `traps` effect can never justify the existence of that claim.

### 4.2 Why the existing U view is not enough

Whitefoot already computes U, the all-claims-blinded view, and its ClaimLedger
can report that a claim's S3 event occurs in the one deterministic complete
derivation. Together they establish only this weaker property:

```text
the selected complete proof uses c, and the checker with all claims removed
cannot prove the root
```

They do not establish that `c` itself is necessary. Another claim may provide
an alternative proof that the canonical DAG did not retain. Unsatisfiable cores
have the analogous limitation: membership in one core does not prove minimality.

The normative rule must therefore use `Full-minus(c)` plus every
`Full-minus(c,a)`, not just U plus ancestry. The smallest obviously correct
implementation is one counterfactual flow per claim/component mask, reusing the
existing flow engine. The corpus has hundreds, not millions, of claims;
correctness and wall-time measurement come before a dependency-bitset
optimization. A later implementation may compute the same fixed judgment more
efficiently, but it may not weaken it to canonical use.

This rule also handles alternatives cleanly. If either of two claims alone can
discharge the same root, neither is individually necessary while both are
present. Source repair chooses one, removes the other, and rechecks; the
remaining one can then become necessary. Claims in mutually exclusive branches
can both be necessary because suppressing either leaves its reaching branch
without a proof at the join.

This is a checker-relative component- and occurrence-irredundancy rule, not a
theorem that one unique or semantically weakest source basis exists.
After two alternative claims are rejected together, either singleton repair may
be valid. "Canonicality" here means a fixed judgment and deterministic
diagnostic over the written source, not a compiler-selected unique proof basis.

The canonical contribution normal form removes checker-visible bundled overlap.
For example, positive `band(known, residual)` contains one pre-proved component
and is rejected; the author claims the residual component, and finite Boolean
introduction reconstructs the exact parent consumer from both sources. This
must not ban a justified singleton compound: if `bor(A,B)` is universally true
while neither disjunct is, its exact positive root is a singleton component and
can be a residual.

Even one canonical component can be semantically stronger than its consumer:
for example, `i == 0` may be used only to derive `i < n`. Deciding the weakest
human proposition in a general logic is not a fast-checker judgment. The
remaining proposition-minimality and consumer-authenticity questions are
mandatory audit duties.

### 4.3 Judgment order

Pre-S3 shape, `because` structure, contradiction-first D/S/F exact lifecycle,
canonical-component lifecycle, consistency, S reconstruction, and D materialization can be
recorded while walking the function. Non-residuality cannot own a diagnostic
until the ordinary OP/FN judgments and provenance gates have produced an
otherwise successful candidate: only then is the terminal-root population complete.
Claim-specific counterfactuals run on that candidate, before CLM-3 and before
checked-program publication. CLM-3 therefore sees only valid retained claims,
and a later unrelated operation or provenance error is not hidden by a
premature unused-claim diagnosis.

Residuality is a simultaneous one-shot classification over `Eligible`, not a
fixed point that silently chooses one of several claims. Every
`Full-minus(c)` run retains S3 from all other eligible occurrences even when a
different counterfactual will later classify one of them non-residual. The
deterministic first machine-invalid occurrence owns the source diagnostic, and
authored repair reruns the whole judgment.

An admission root counts only when its exact complete query state is
non-contradictory and the c-dependent proof ancestry contains no
contradictory/ex-falso predecessor. At a join, every reachable predecessor
lineage contributing to the root must independently have non-contradictory,
non-explosive legal support. At least one c-dependent lineage must retain c in
its dynamic prefix, but c need not dominate a mutually exclusive sibling
lineage. This is what lets distinct branch claims jointly establish a fact at a
join without allowing one explosive predecessor to hide inside an apparently
non-contradictory root state. A claim cannot manufacture usefulness by making a
later branch impossible and then using ex-falso to discharge a partial
operation in that branch.

## 5. What a claim is not

| Situation | Correct Whitefoot form | Why claim is forbidden |
|---|---|---|
| malformed input or expected environment failure | branch or typed `Result` | the predicate can legitimately be false |
| deliberate program termination | return or `ExitStatus` | `claim False()` has no true derivation |
| unit-test expectation | compare and return a failing test status | a test oracle is an observation, not a proof premise |
| debug assertion or telemetry | explicit value/control-flow facility, if selected later | it need not authorize a proof-required operation |
| checker already proves the predicate | remove the claim | checker and claim proof contributions overlap |
| theorem is true but no admission proof needs it | remove the claim | it fills no checker incompleteness |
| external promise not stated by a verified contract | validate and branch | the environment is not a mathematical premise |
| human proof of an invariant the checker misses and a later bound needs | claim, with full `because` derivation | this is the intended residual |

The runtime trap remains useful, but only as protection against an erroneous
residual proof. The availability of that protection does not broaden the
source situations in which a claim is legal.

## 6. Proof-expression boundary

The current CLM-1 accepts any exact `own Bool` expression. That is too broad for
a construct whose sole purpose is proof discharge: the predicate can currently
carry calls, effects, consumption, cleanup, or nontermination and thereby hide
ordinary program behavior inside a claim.

The initial claim-predicate judgment should be deliberately small. Evaluation
must be:

- total by the unconditional semantics of its operations, with no
  proof-required partial operation or subscript hidden inside the predicate;
- deterministic;
- non-consuming and ownership-neutral;
- free of writes, allocation, external communication, blocking, nested traps,
  release, and cleanup; and
- limited to compiler-known total operations and observational reads of live,
  stable local or owned state.

`pure` effect-row spelling alone is insufficient: Whitefoot deliberately does
not infer totality from purity, and a user call can recurse, consume an affine
value, or trigger cleanup. The smallest candidate should therefore reject user
and system calls in a direct claim predicate until a separate machine-proved
total-and-observational judgment exists. Existing contract-goal/direct-goal
admission is a plausible implementation starting point, but the exact source
delta must be measured before selecting it.

A preceding `let` can still hide computation behind an identifier, so this
restriction cannot prove author intent. The offline audit must follow the
predicate's live origin and reject an empirical test disguised as a theorem.
The mechanical restriction nevertheless prevents the claim statement itself
from becoming a carrier of ordinary behavior.

The S3 support and later kill set must correspond to the same values observed
by the predicate. If parallel execution is added later, those reads need an
immutable/exclusive snapshot or an explicit happens-before relation; proof
purity does not by itself solve concurrent stability.

## 7. `because` is a derivation record

The current grammar stores an arbitrary STRING and the compiler checks none of
its content. Under the new doctrine, labels such as `"expected Ok"`,
`"unreachable"`, `"drift"`, or `"the server guarantees it"` are not
justifications.

Every `because` record must contain:

1. **Premises:** program facts, verified contracts, earlier passed claims, and
   mathematical facts available at the exact pre-S3 point.
2. **Derivation:** the intermediate steps and rules that lead from those
   premises to every canonical contribution and reconstruct the exact
   predicate.
3. **Conclusion:** the exact written predicate, not a nearby informal property.
4. **Checker gap:** the missing induction, algebraic rule, data-structure
   invariant, injectivity fact, or other specific incompleteness.
5. **Consumers:** one or more terminal source-admission obligations that need
   the fact; the machine inventory remains the complete list even when prose
   highlights one primary witness.

For a loop, the derivation supplies a base and step over the exact iteration
state. For recursion, it supplies a well-founded measure or a valid induction
principle for the claim fact. For a generic declaration, it states the
quantified parameter domain and the instantiation step.

The initial candidate can keep STRING but require exactly five labelled,
nonempty fields in the order above. The compiler can parse that shape, retain
it, and attach its own complete machine-derived list of changed admission roots
to the claim. It cannot decide whether the English fields state real premises,
valid steps, the exact semantic conclusion, the actual checker gap, or authentic
consumers without becoming the proof engine that would make the claim
redundant. Those meanings remain review requirements. Introducing formal
premise references or proof syntax is a separate language decision.

A good record explains a derivation. It does not merely restate the conclusion:

```text
premises: the loop has consumed k input bytes and emitted at most one output
byte per consumed byte; output capacity equals input length
derivation: by induction on completed iterations, emitted <= k <= input length
conclusion: output_index < output_length at this write
checker gap: no inductive loop-summary rule for emitted-byte count
consumers: the following output subscript obligation
```

Earlier verified claims may be premises, creating an acyclic source-ordered
audit dependency graph. A claim may not rely on the runtime success of itself,
the same static claim at a recursive call, a later claim, a loop's next dynamic
iteration, a postcondition whose proof ultimately depends on it, or an unstated
caller or environment convention. A logical induction hypothesis is different:
it is valid only with explicit base and step cases and a strictly decreasing
well-founded measure, and it may not assume that the runtime claim has already
passed. When an ordinary caller promise is expressible as `requires`, it belongs
in the contract; importing it through prose is not a checker gap.

## 8. Generic, reachability, and control-flow cases

- Lifecycle and individual necessity are checked per inhabited concrete
  instance. If any inhabited instance proves, refutes, contradicts, or does not
  need the shared source claim, the closed source is rejected at a deterministic
  concrete witness. Repair may split a helper or move the residual into a
  narrower branch; there is no instance-local check omission.
- An entry-uninhabited concrete instance supplies neither a report nor a
  residual witness. A contradictory local path in a schema or concrete flow is
  vacuous and cannot prove a claim by ex-falso.
- Residuality is a source-body admission property rather than observed
  reachability from `main`: a dead nongeneric function can retain a claim when
  its body obligation needs it. By parity, every generic body receives one
  source-canonical symbolic claim-schema judgment even with zero concrete
  instantiations, followed by each inhabited concrete recheck.
- The schema installs source-canonical call requirements and uses GenericInt or
  GenericFloat only as copy datums in exact opaque goals, never as invented L0
  terms. A symbolic FN-9 consumer exists only when its result, selected return,
  and relation already use concrete integer fragment types; generic-T FN-9 is
  concrete-instance-only.
- A stable schema report is owned by declaration and source NodePaths, contains
  no scratch FunctionId/GoalId/TermId/DerivationId or monomorph display identity,
  and links inhabited concrete reports in stable order.
- A structurally unreachable claim is an FN-1 error and does not enter CLM-2.
- A second claim whose predicate is established by an earlier S3 is redundant.
  Claims can depend on earlier claims in their human derivation, but cannot
  exist merely to recheck or rename an already available fact.
- CLM-2 validity and residual necessity precede CLM-3. Only a valid retained
  claim can reach a `deny_claims` rejection.

Mixed generic instances remain the largest foreseeable usability risk. A source
claim may be necessary for one instance but statically redundant for another,
and current language expressiveness may offer only helper duplication as the
repair. That is not a safety inconsistency: the selected rule rejects at the
first stable concrete witness and never omits a runtime claim for selected
instances. Hostile tests compare an uninstantiated generic with an equally dead
nongeneric function and lock the same source-body admission policy for both.

## 9. Offline audit model

An audit input should be derived from canonical checked source, not assembled
from the prose alone. A future claim packet would need at least:

- exact source, specification, checker, and packet hashes;
- claim identity, concrete instance, path, typed predicate, and exact D/S/F images;
- the versioned `Contrib(P)` identities and their support/kill manifestations;
- the proof-pure predicate snapshot and relevant ownership/kill state;
- the pre-S3 fast facts without the current claim;
- the `because` premises, steps, conclusion, gap, and consumer;
- every `Full-minus(c,a)` and `Full-minus(c)` admission root that changed, with
  its stable masked disposition and attached successful provenance evidence; and
- dependencies on earlier audited claims.

This is a research shape, not authority to build a generalized serialized
proof system now. The current private ClaimLedger already has source identity,
predicate, justification, derivation ancestry, uses, and provenance, but lacks
the pre-state, per-claim counterfactual, stable packet identity, and audit
status.

Suggested audit results are:

- `HumanVerified`;
- `CertificateVerified`;
- `CounterexampleCandidate`;
- `RefutedAfterReplay`;
- `Unknown`;
- `Vacuous`;
- `UnsupportedEncoding`; and
- `Stale`.

`Unknown`, timeout, out-of-memory, and unsupported encoding are absence of an
automated result. They neither validate a claim nor invalidate a sound human
proof. A claim cannot receive human approval while its derivation remains
unreviewed.

### 9.1 Slow SMT query

The usual validity query asks whether `Context and not P` is unsatisfiable. It
must be paired with an independent context-consistency/reachability check;
otherwise a contradictory context proves every predicate vacuously. A
satisfiable model from an over-approximated VC is initially only a
counterexample candidate and should be replayed against Whitefoot semantics
before being called a refutation.

The translation is part of the trusted boundary. It must encode Whitefoot's
fixed-width integer domains, checked-overflow preconditions, evaluation order,
aliasing, snapshots, ownership kills, and trap boundary. Replacing all machine
integers with mathematical integers, or all exact operations with wrapping
bit-vectors, proves the wrong program. Heap mutation, arrays, quantifiers,
recursion, nonlinear arithmetic, and loop induction will also leave real
claims as `Unknown`; a slow solver is not a completeness theorem.

### 9.2 Certificates and promotion

An SMT `unsat` answer is not automatically a small trusted proof. Certificate
formats cover different theory subsets, and some emitted proofs contain trust
steps. `CertificateVerified` requires complete theory coverage, no hole or
trust step, independent replay, and an explicit account of the
Whitefoot-to-logic translation TCB.

There are only two sound ways for offline success eventually to remove a
runtime residual:

1. promote the discovered proof rule into the versioned normative checker, at
   which point the source claim becomes redundant and the author deletes it;
   or
2. design a separate explicit static proof/certificate construct checked
   deterministically by the normative compiler, and replace the source claim
   with that construct.

An optional solver that happens to succeed on one run may never silently omit
an accepted claim or change ordinary source acceptance.

## 10. Pitfalls and their dispositions

| Pitfall | Required disposition |
|---|---|
| checker can prove P | source error; author removes claim |
| checker can prove not P | source error; use control flow if failure is possible |
| literal `False()` | source error, never intentional abort |
| contradictory context | vacuous, never positive proof |
| structurally unreachable statement | FN-1 rejection before claim lifecycle |
| adding P contradicts the pre-state | inconsistent-claim source error |
| claim fact makes a downstream root contradictory | root is ineligible; never discharge by explosion |
| `band(known, residual)` or the same shape behind an origin | reject overlapping component; claim the residual and reconstruct parent |
| positive disjunction is true but neither child is a theorem | retain exact root as one component; do not require a false child theorem |
| xor/equivalence repeats a pre-proved conjunct | operator-specific basis exposes and rejects the overlap |
| contribution normalization/reconstruction is ambiguous | predicate shape is not yet claim-admissible |
| one canonical component is stronger than the missing lemma | mechanical normal form is insufficient; review rejects over-strength |
| all-claims U fails but current claim is replaceable by another | reject under `Full-minus(c)` |
| selected canonical proof mentions c | insufficient by itself; run individual counterfactual |
| author manufactures a dead partial operation as consumer | structural check may pass; authenticity review rejects it |
| claim appears only in optimizer or observational metadata | reject as non-load-bearing |
| predicate performs ordinary behavior or may diverge | reject proof-expression judgment |
| five-field `because` shape is missing | machine source error |
| fields are vague, circular, or semantically false | author/audit failure; never an approved theorem |
| external promise is not a verified premise | branch on the external value |
| compiler implementation misses a normative derivation | fix the compiler; do not add a claim |
| offline solver says `Unknown` | no automated conclusion |
| SMT model is not replayed | counterexample candidate only |
| proof is stale after source/spec/checker change | re-audit exact packet |
| certificate contains a trust step | not `CertificateVerified` |
| generic instances disagree | reject shared shape at deterministic witness |
| uninstantiated generic and dead nongeneric use different reachability tests | implementation defect; both use source-body admission |
| accepted claim omitted by lowering or optimization | compiler correctness failure |

## 11. Baseline and branch implementation evidence

The v0.33 baseline compiler has several useful foundations:

- lifecycle is already judged after predicate effects and before the claim's
  own S3 establishment;
- complete, all-claims-blinded U, and S4-blinded views share one structural
  flow;
- the derivation ledger can identify the exact S3 events used by one canonical
  proof root;
- the ClaimLedger retains source identity, predicate, justification, uses, and
  provenance; and
- accepted claims use one ordinary lowering and backend path.

It also exposes the exact baseline gaps:

- CLM-1 currently accepts any `own Bool`, including effectful or nonterminating
  predicates;
- S3 currently publishes a Boolean root and its signed decomposition members,
  so one compound claim can hide a checker-known or unused component;
- CLM-2 does not yet query both signs of every allowed exact Bool/origin shape;
- the checker has no finite child-to-parent Boolean introduction, so a smaller
  residual cannot currently reconstruct an exact compound consumer;
- `because` is an arbitrary unchecked STRING;
- U blinds all claims rather than one claim;
- the canonical DAG does not retain every alternative proof;
- ledger construction is observational and occurs only after acceptance; and
- the active language explicitly accepts unused claims and constructed
  `False()` claims.

The branch candidate implements the bounded extension rather than a solver:
canonical contribution normalization, finite exact-parent reconstruction,
claim/component-blinded counterfactuals, a closed terminal-root test, a
proof-expression judgment, stable concrete/schema audit evidence, and source
errors, while keeping accepted-claim lowering unchanged. The remaining evidence
work is corpus migration, protected before/after audit, full-gate verification,
performance measurement, and adversarial review.

## 12. Current corpus impact

The strengthened doctrine is much larger than the old seven/36 redundancy
advisory migration.

At baseline `4f01bab6a7bf158fff19dd54b062b748d20086d1`:

| Corpus | Claims | Files | direct `False()` | direct `True()` |
|---|---:|---:|---:|---:|
| `tests/programs/*.wf` | 241 | 23 | 59 | 3 |
| protected conformance | 410 | 197 | 20 | 2 |
| selected in-scope external corpus | 651 | 220 | 79 | 5 |
| standalone `tests/codegen/**/*.wf` | 15 | 13 | 3 | 0 |
| historical research probes | 14 | 13 | 0 | 0 |
| all repository `.wf` source | 680 | 246 | 82 | 5 |

The current checker separately reports seven real-program and 36 protected
redundancies. In the selected 651, the 79 false literals, five true literals,
and 43 already-proved occurrences are disjoint, giving a hard lower bound of
127 migrations before individual necessity or `because` quality is considered.
The 15 codegen claims are dormant and the 14 research claims are historical
probes. Their aggregate presence bounds repository-wide scans, but they remain
excluded unchanged and create no per-item disposition or migration work unless
a live caller or gate is discovered. Compiler inline Rust fixtures contain at
least 43 direct false and 35 direct true claim spellings.

A syntactic shape census of the 651 program/protected predicates found 235 direct
identifiers, 386 known table-operation calls that appear pure/total, and 30
`.defined` or older typed-comparison forms; no direct user/system/effectful
call was found. This suggests the direct proof-expression migration may be
bounded, but identifier origins still require audit.

The in-scope review-data migration is material: 153 of 651 `because` strings
contain at most two whitespace-delimited words, 413 at most four, and 174
contain words
such as `drift`, `wrong`, `unexpected`, `expected`, `failed`, `accepted`,
or `rejected`. These are heuristic risk indicators, not semantic verdicts.
Every claim must be audited individually, and every protected change requires
an exact before/after inventory in the owner merge packet.

## 13. Implementation feasibility and sequence

This research document is not execution authority. The owner-chartered branch
uses the Current Plan and batch 0075 to proceed in this order:

1. freeze the complete 651-claim in-scope and inline-fixture inventory; record
   only the aggregate presence and explicit unchanged exclusion of the 29
   dormant/historical `.wf` occurrences unless a live caller or gate is found;
   for the in-scope set, record predicate effects/origins, concrete instances,
   current ledger uses, admission roots, first diagnostics, effect owners, and
   reverse-call closure;
2. prototype `Full-minus(c,a)` and `Full-minus(c)` by reusing the existing flow
   with component/claim S3 masks and measure absolute compiler wall time on real
   bundles;
3. define the closed admission-root set and prove hostile cases for alternative
   claims, branch joins with per-predecessor support and c-lineage-only
   domination, killed facts, loops, postconditions, provenance,
   inconsistent continuations and downstream explosion, manufactured
   consumers, per-component parent/descendant invalidation versus independent
   rederivation, and generic instances;
4. define and census the smallest proof-expression judgment and versioned
   `Contrib(P)` normal form; prove D/S/F support and manifestation mapping, equality and
   `.defined` treatment, finite non-cyclic S reconstruction and D materialization,
   `band(known, residual)`, positive-disjunction singleton, and partial-known
   xor/equivalence cases;
5. produce the exact specification and ordinary-test candidate for the
   lifecycle matrix, individual necessity, `because` author law, and accepted
   runtime retention;
6. migrate real programs and ordinary fixtures by semantic purpose: explicit
   branch/result/exit for possible failure, deletion for redundancy, and
   rewritten derivation for genuine residuals;
7. land candidate protected changes on the work branch with exact audits and
   carry them in the single owner merge packet;
8. on the branch chartered by the recorded owner direction, update project law,
   writer patterns, AI-agent instructions, and the paired MCTS decisions before
   the exact merge packet is presented; and
9. rebase the complete branch candidate onto then-current `main`, rerun complete
   gates, artifact checks, and adversarial audit, then present it for owner
   approval and perform only predetermined activation mechanics afterward.

If N counterfactual flows are too expensive, measure and implement an exactly
equivalent dependency analysis. Performance cannot justify accepting a weaker
canonical-use rule. If no tractable equivalent is found, stop and return to the
owner with evidence rather than silently narrowing “individually necessary.”

## 14. Sources and prior-art boundary

These sources support implementation cautions, not Whitefoot language
authority:

- Dafny's proof-dependency analysis distinguishes contradictory assumptions,
  vacuous proof, and unused assumptions, and notes that unsatisfiable cores are
  not guaranteed minimal:
  <https://dafny.org/blog/2023/10/27/proof-dependencies/>.
- SMT-LIB permits `sat`, `unsat`, and `unknown` results:
  <https://smt-lib.org/papers/smt-lib-reference-v2.7-r2025-02-05.pdf>.
- Why3 distinguishes `Unknown`, timeout, resource failure, invalid results, and
  stale/obsolete proof sessions:
  <https://why3.org/doc/manpages.html>.
- cvc5 documents that Alethe currently covers a theory subset and that LFSC
  output can contain arbitrary-formula trust steps:
  <https://cvc5.github.io/docs/cvc5-1.3.1/proofs/output_alethe.html> and
  <https://cvc5.github.io/docs/cvc5-1.3.1/proofs/output_lfsc.html>.
- SMTCoq illustrates a certified checker over an already-stated formula and
  certificate; it does not certify a future Whitefoot-to-formula translation:
  <https://smtcoq.github.io/capi.html>.

## 15. Final decision criterion

The direction passes first-principles review with one explicit limitation:

```text
machine-exact checker-relative structural residual + machine-safe runtime boundary
                                plus
human/AI/offline validation of theorem truth, minimality, and authentic use
```

It fails if presented as a single machine-perfect decision about arbitrary
human mathematics. With the two-layer statement, it gives Whitefoot a sharper
and more auditable construct than a general assertion: every claim has a
specific reason to exist, one or more terminal proof consumers, one written
derivation, and one retained runtime boundary.
