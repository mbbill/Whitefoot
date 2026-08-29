# TERRAIN — the claim / provenance / entailment area as it actually stands

Researcher deliverable for batch 0106. **This file contains no design of my
own.** It is a map: what the rules say, what programs actually wrote, what the
prover actually does, where the five holes are, and what the outside world has
said about the same questions. Every judgement of mine that is not a quotation
or a code reading is marked *[observation]*; every prior-art statement is marked
**[from-memory]** and was written without network access.

Tree read: `integration/2026-08-28c`, tip `b1367c82`, spec v0.39 ACTIVE.
Line numbers are `spec/kernel-spec.md` at that tip unless another file is named.

Contents:

1. [The rule texts](#1-the-rule-texts)
2. [The evidence corpus](#2-the-evidence-corpus)
3. [The prover today](#3-the-prover-today)
4. [The five holes with their artifacts](#4-the-five-holes-with-their-artifacts)
5. [Prior art, from memory](#5-prior-art-from-memory)
6. [Open facts a designer must not get wrong](#6-open-facts-a-designer-must-not-get-wrong)

---

## 1. The rule texts

The whole area lives in §18 "Obligation discharge: claims, entailment, and
provenance (normative)", lines 2694–3412, plus five satellites: [OP-4] (874–886)
and [ENT-6]'s repair sentence, [GIVE-1] (273–293), [DIAG-1]'s claim schedule
(1846–1860), [DIAG-3] (1960–1986), [PAR-3]'s erroneous-execution clauses
(2074–2079), and [SYS-8] (2537–2577).

The section is **13 rules in one closed pipeline**. Read in dependency order the
pipeline is: ENT-2 fixes the vocabulary → ENT-3 says what establishes a fact →
ENT-5 says what a fact depends on and when it dies → ENT-4 closes → ENT-6
attaches obligations and computes both provenance dependency and claim authority
→ CLM-1 admits a claim's shape and authority → CLM-2 decides whether the claim
earns its keep → CLM-3 partitions claim-free roots → PRV-1/2/3 gate external
data out of protected subjects.

### 1.1 [CLM-1] — what a claim *is* (2696–2757)

> **2696–2700:** `claim name: e because "text";` is the sole writer-spelled
> runtime boundary for a proof residual which the normative checker cannot
> derive. It is not an assertion, abort, conditional, test oracle, debug check,
> or general invariant facility. The author asserts that `e` is true on every
> execution reaching the statement; if it may legitimately be false, source must
> instead use ordinary `if`, `match`, loop transfer, typed result, return, or
> command status. `e` must have exact value mode and type `own Bool` under the
> [OP-5] condition judgment, including TYPE-7 implicit-read exclusivity, and
> must additionally satisfy the claim-proof-predicate judgment below.

*Question answered:* what a claim means and who is responsible for its truth —
the author, always, on every reaching execution. *Deliberately excluded:*
everything conditional. A claim is never a test; a predicate that may be false is
a control-flow question, not a claim. *Leaned on by:* [DIAG-3] (the record it
emits), constitution T3 (whose whole derivation is "a claim is a reviewed
always-true lemma, therefore a correct program cannot reach the trap"), and
[PAR-3] 2074–2079.

> **2702–2705:** A claim proof predicate is one finite direct goal [ENT-2] whose
> evaluation is total, deterministic, observational, non-consuming, and
> ownership-neutral. It may contain typed literals, named constants,
> non-consuming reads of live copy places, fixed-length observations, and
> compiler-known total non-trapping integer, float, Boolean, conversion,
> reinterpretation, enum-equality, and allocation-fit predicate rows, recursively
> under the same restriction. It may not contain a user or system call,
> subscript, proof-required exact operation, checked-result operation,
> allocation, construction, write, move, borrow or reborrow, consuming
> projection, residual drop or cleanup, release, block, external operation,
> nested claim or trap, or any other partial, effectful, ownership-changing, or
> potentially nonterminating computation. A rejected shape cites CLM-1 at the
> predicate `expr`; the checker never accepts it merely because its inferred
> effect row is `pure`.

*Question answered:* what may appear inside `e`. The predicate must be evaluable
with no side effect and no possibility of its own failure, because it executes at
runtime on every reach. *Deliberately excluded:* self-reference (a claim inside a
claim), and any operation whose own domain would need proof — you cannot bootstrap
a claim out of an operation that itself needs a claim. *Leaned on by:* CLM-2's
lifecycle (it needs `e` to be a total goal it can query both signs of) and the
lowering path (a retained check must be cheap and effect-free).

> **2707–2721 (abridged to its load-bearing sentences):** The decoded `because`
> STRING is exactly five LF-separated lines … `premises:` / `derivation:` /
> `conclusion:` / `checker gap:` / `consumers:` … The five fields are retained
> review data [DIAG-2]. This structural check does not prove their prose true:
> owner approval of the checker-accepted source requires human, AI-assisted, or
> offline-proof review to validate the stated premises, derivation, exact
> conclusion, checker gap, and one or more authentic terminal consumers. Such
> review may use only facts valid before this claim, including explicitly named
> earlier reviewed claims; it may not use this claim's own successful execution,
> a later fact, an unstated caller or environment promise, a user callee's body
> or unstated system behavior in place of a verified or specification-fixed
> callable-boundary fact, or a circular occurrence of the same dynamic claim.

*Question answered:* what a human reviewer is handed, and what counts as a legal
proof for them. *Deliberately excluded:* machine checking of the prose — the
compiler checks only that five labelled nonempty lines exist. The `checker gap`
field is the writer's own statement of *why the prover cannot do this*; the
`consumers` field is the writer's statement of *what breaks without it*. Both
duplicate, in prose, what CLM-2 then decides mechanically (lifecycle = gap;
residuality = consumers). *[observation]* That duplication is the only place in
the spec where the writer and the checker are asked the same question twice —
one in prose, one mechanically — and the corpus in §2 shows the prose field is
where the real language pressure is visible. *Leaned on by:* nothing mechanical;
by the whole review model socially.

> **2723–2740 (canonical formation, the D/S/F machinery):** After its type,
> proof-shape, per-function name, and five-field formation judgments succeed,
> CLM-1 performs one fact-free canonical-formation subjudgment before any claim
> truth, contradiction, provenance, or residuality query. Let `D(P)`, `S(P)`, and
> `F(P)` be three exact typed goal images of predicate P. `D(P)` is the direct
> GoalExpression of the evaluated written predicate. `F(P)` is the unique complete
> still-valid ordinary-let origin expansion of `D(P)` … `S(P)` is the
> support-canonical snapshot-frontier expansion: … when the current subtree
> already has an exact L0 projection or one fixed normalization, retain that
> subtree unchanged and do not expand any datum below it … Thus S preserves the
> support of each checker fact the claim actually reads, while F records the fully
> structural origin. … Canonical formation constructs the unique ordered
> `Contrib(P)` from positive S in the finite [ENT-2] fact vocabulary. … `bxor`,
> Boolean equivalence, a normalization with alternative positive clauses such as
> signed division/remainder, an ambiguous origin, or any shape for which
> normalization, support, component negation, S reconstruction, or D
> materialization is not unique is not an admitted claim predicate in this version
> and rejects under CLM-1 before a component ordinal is published.

*Question answered:* how one written Bool predicate becomes a finite ordered list
of *components*, so that CLM-2 can ask "is each conjunct individually necessary"
rather than "is the whole sentence necessary". Three images exist because three
different questions need three different expansions: `D` is what the runtime
check evaluates and what must be re-materializable; `S` is what the claim's
*support* (and hence its locality and its kills) is measured on; `F` is the
fully-expanded structural origin used only for lifecycle. *Deliberately
excluded:* any algebraic rewriting, any search, and any predicate whose
decomposition is not unique (hence the `bxor` and signed-`%` exclusions).
*Leaned on by:* ENT-3.S3 (which establishes exactly `Contrib(P)`, never `P`),
CLM-2's per-component residuality, ENT-5's per-component support, and CLM-1's own
per-component locality query.

> **2742–2746 (the locality gate — the heart of H1/H2/H3):** At the claim point,
> CLM-1 then queries every component's ordinary S-derived support [ENT-5] in
> component-ordinal order against [ENT-6]'s frozen claim-authority state. Every
> runtime value component and holder read by that support must be `Local` to the
> current function. If any support member is `BoundaryResult`, the whole claim is
> non-local and rejects under CLM-1 using [DIAG-1]'s least component, earliest
> boundary witness, and first canonical support carrier that observes that
> witness; no S3 source, lifecycle query, `Eligible` member, counterfactual run,
> ClaimLedger record, or lowering authority is formed for that occurrence. A
> verified `ensures` and its S12 publication never make the returned value local:
> the caller consumes the verified relation directly and cannot restate or
> strengthen it with a claim. This authority admission is independent of truth and
> of [PRV-1] provenance; a `Local` component is not thereby true or internal, and a
> PRV-internal call result is still `BoundaryResult`.

*Question answered:* **W3 cheat-proofness across the callable boundary.** A claim
is a lemma about *this function's own* values; the writer must not be able to
smuggle in an unverified promise about what a callee or the host returned.
*Deliberately excluded:* truth (a `Local` component is not thereby true) and
provenance (an internal call result is still `BoundaryResult`). These are three
independent axes and the spec says so twice. *Leaned on by:* the constitution's
W3 clause verbatim ("a caller may rely on a user callee's result behavior only
through its machine-verified callable boundary"), and by [PRV-3] 3406 ("a `claim`
may not authorize an external constrained subject").

> **2748–2757 (retention, name, and the non-repair sentence):** Every claim
> accepted by [CLM-2] is retained as one runtime check in every build mode, is
> never elided, and evaluates `e` exactly once at every dynamic reach. False
> evaluation emits the required record [DIAG-3] and aborts [SCOPE-4, EFF-4] before
> S3 can authorize a later operation; true evaluation continues and establishes
> only [ENT-3]'s canonical claim contribution. A `claim_stmt` syntactically
> exhibits `traps` [EFF-2] and does not count as delivery or must-divergence
> [GIVE-1]. … Operand provenance does not by itself prove a claim true: [PRV-2] and
> [PRV-3] still reject claim-only authorization of an unconditionally external
> constrained subject, while CLM-1 independently requires local authority and
> [CLM-2] independently requires a genuine admission consumer.

*Question answered:* the runtime contract, and the fact that **three independent
gates** (CLM-1 locality, CLM-2 residuality, PRV-2/3 external-subject) can each
refuse the same claim for different reasons. *Deliberately excluded:* any build
mode in which the check disappears. *Leaned on by:* [DIAG-2] 1884 "Every
writer-reachable source-language runtime check is one [CLM-1] claim with
disposition `retained`" — the claim is the *only* runtime check in the language.

### 1.2 [CLM-2] — whether a claim earns its keep (2759–2799)

> **2759–2766:** One FN-1-reachable concrete claim occurrence c is judged only
> after CLM-1 has admitted its predicate … and after evaluating its predicate but
> before its own S3 source. If the pre-S3 combined state is contradictory, c is
> vacuous and rejects; contradiction is tested first … Otherwise query both signs
> of every image in that ordered inventory. … Otherwise deriving any positive sign
> rejects c as redundant and deriving any negative sign rejects it as refuted.
> Thus `claim True()` is redundant and `claim False()` is refuted on every
> reachable path. **Checker strengthening may and must turn a formerly unknown
> claim into this source-upgrade error; the author removes or restructures the
> source and recompiles, and no compiler or optimizer silently elides the written
> check.**

*Question answered:* is this claim at the proof frontier? Four verdicts —
*vacuous* (unreachable-by-contradiction), *redundant* (prover already knows it),
*refuted* (prover knows the negation), *unknown* (proceed). *Deliberately
excluded:* acceptance-as-documentation. The bolded sentence is the whole of H4:
the spec makes the flip mandatory and calls it a "source-upgrade error".
*Leaned on by:* [ENT-1] 2853–2855, which repeats it as a version-monotonicity
rule.

> **2768–2780 (component lifecycle):** For a remaining unknown predicate P,
> component lifecycle consumes CLM-1's already formed `Contrib(P)` and walks S and
> F in lockstep through the same signed conjunctive Boolean structure. … A
> pre-proved component rejects c for overlap even when no complete predicate image
> was derivable; a pre-refuted component rejects it as inconsistent. Tentatively
> adding every component must remain non-contradictory, derive positive S, and not
> derive negative S through [ENT-4]'s retained ordinary derivation.

*Question answered:* the same four verdicts again, but per conjunct, so a claim
whose *left half* the prover already knows is refused as *overlapping*.
*Deliberately excluded:* partial admission. There is no "keep the half I still
need" — the writer restructures. *[observation]* This makes the H4 instability
finer-grained than it first looks: a prover strengthening that learns one conjunct
of a two-conjunct claim also flips that program.

> **2782–2798 (residuality):** Let `Eligible` be the fixed source-ordered set of
> concrete occurrences that passed … For each c and each component a,
> `Full-minus(c,a)` repeats the same whole-program proof analysis with every other
> Eligible S3 source and c's other components unchanged, while c still evaluates,
> exhibits the same effects, and retains its runtime statement, but a's one
> component-specific S3 source event is withheld. … **The terminal admission roots
> are exactly the four proof-required operation families [ENT-6], ordinary [FN-8]
> call requirements, and mandatory complete [FN-9] selected-return aggregate
> proofs.** Optimizer or observational S7/S11/S12 metadata, effect exhibition,
> CLM-3 structure, another claim's lifecycle, a test oracle, or a fact with no such
> terminal root is not a consumer. Every component a must have at least one
> terminal root that succeeds in Full and fails in `Full-minus(c,a)`, and at least
> one terminal root must likewise fail in `Full-minus(c)`. … Residuality is one
> simultaneous classification over fixed Eligible, never a fixed point selecting a
> survivor among alternatives.

*Question answered:* is this claim *load-bearing*? A counterfactual re-run of the
whole-program analysis with exactly this component's fact withheld must break at
least one of three named admission roots. *Deliberately excluded:* claims that
help only an optimizer, only another claim, or only a test. The closed list of
terminal roots is the language's answer to "what is a claim *for*": subscript
bounds, integer domain, allocation fit, system range, callee `requires`, and
`ensures` proof. *[observation]* This closed list is the single most important
structural fact in the area — the claim mechanism has *exactly six* legitimate
customers, and every one of them is a **partial-operation domain obligation**.
*Leaned on by:* ENT-5 3050–3054 (the mask changes nothing but S3), and
[DIAG-2] 1886.

### 1.3 [CLM-3] — `deny_claims` (2801–2829)

> **2801–2806:** Any source `fn_decl`, generic or nongeneric, may carry the one
> optional fixed terminal `deny_claims` before its optional `program_kind`. … Each
> marked concrete [FN-2] instance is one strict root. The marker is compile-time
> policy only: it adds no effect, trap, runtime check, fact, type, mode, region,
> call convention, body, or lowering, and it neither removes nor changes any
> [CLM-1] claim.
>
> **2815–2816:** A demanded component succeeds strictly exactly when its
> `MayClaims` set is empty, every ordinary user-call requirement owned by the
> component discharges at that call in caller U [FN-8], and every strictly
> outgoing demanded callee component has a successful strict summary. … an empty
> `MayClaims` set therefore already implies that every obligation this component's
> complete-state judgment discharged also discharges in U.

*Question answered:* how a writer says "this subtree must be trap-free", i.e. how
the *absence* of claims becomes a checkable property with an SCC-closed
transitive reading. *Deliberately excluded:* any per-claim policy; it is
all-or-nothing per reachable component. *Leaned on by:* nothing else — it is the
only leaf of §18. *[observation]* CLM-3 is the only place the language admits
that "how many claims are in the closure" is a thing a writer might care about;
it is also the only rule in the area whose criterion is a *count*, not a
property, of claims.

### 1.4 [ENT-1] — the fragment's constitution (2831–2864)

> **2831–2835:** The entailment fragment is a closed, deterministic, search-free
> derivation system fixed completely by this specification. Its state is the L0
> relation state plus [ENT-2]'s finite signed opaque goals. … [SCOPE-2] is
> unchanged: **every fact source [ENT-3] is an executed control condition, an
> executed retained residual claim, a requirement statically proved by every
> ordinary caller before S4 admits it to a body, a declared allocation or type
> property, a constant, S11's compiler-owned structural consequence, or S12's
> machine-verified normal-result publication.** No source postcondition is
> trusted…

*Question answered:* the closed list of things that may become a fact. Seven
kinds. Note the shape of the list: **five of the seven are things the compiler
itself observed; one is a caller's proof; exactly one is a writer's word.**
*Deliberately excluded:* search, solvers, inferred invariants, struct
invariants, unverified summaries. *Leaned on by:* everything; it is the area's
axiom.

> **2853–2858 (version monotonicity):** Version monotonicity of fact-source and
> closure strengthening preserves every already-discharged operation, call goal,
> or selected-return relation, **but claims deliberately sit at the proof
> frontier.** A later normative checker may newly derive a claim predicate, its
> negation, or one contribution component, or may make its S3 contribution
> unnecessary; CLM-2 must then reject that source as redundant, refuted,
> overlapping, or non-residual so the author removes or restructures it. This is
> an explicit source-upgrade rule, never authority for compiler or optimizer
> elision. Activating [PRV-2] or [PRV-3] for an already attached protected family,
> attaching a new protected family, changing a [SYS-2] component from internal to
> external, adding or removing a `BoundaryResult` seed or declassification, or
> adding a callable publication surface is an amendment-level accepted-set change,
> not implementation strengthening.

*Question answered:* what a compiler may and may not do between spec versions.
Ordinary programs are protected by monotonicity; **claim-bearing programs are
explicitly not**. And the last sentence is the one that governs H3: touching the
`BoundaryResult` seed set is amendment-level, never an implementation choice.
*Leaned on by:* CLM-2 2766, and the whole H4 question.

### 1.5 [ENT-2] — the vocabulary (2866–2905)

> **2870:** A term is exactly one of: (a) a tracked place … whose final selected
> type is one fragment type; (b) a length term `len(P)` … (c) a constant … (d) one
> of the two compiler-owned u64 capture terms belonging to an admitted `for_stmt`
> … (e) the one compiler-owned symbolic result datum of an admitted FN-9 clause …
> or (f) the distinguished zero term Z…
>
> **2872–2873:** Two places are the same term exactly when their roots resolve to
> the same declaration event [TYPE-6, DIAG-1] and their canonical source spellings
> [FORM-2] are byte-identical … **Term identity thus under-approximates aliasing,
> while kills [ENT-5] use [OWN-7]'s resolved-place overlap relation and
> over-approximate it.**
>
> **2901–2905:** An atomic fact is one difference bound `t1 - t2 <= c` … or one
> disequality `t1 != t2`. … Implicit facts hold at every program point: every term
> t carries the reflexive bound `t - t <= 0`; every term t of fragment type T
> carries `t - Z <= max(T)` and `Z - t <= -min(T)`; every length term over a place
> of type `array<T, N>` carries the equality `len(P) = N`…

*Question answered:* the algebra. It is **difference-bound matrices (DBM) plus
disequalities plus opaque signed Booleans**, over integer fragment types only.
*Deliberately excluded:* multiplication of two variables, floats, general linear
arithmetic, aliasing, and any relation between more than two terms.
*[observation]* The DBM choice is the single largest determinant of what the
writer must claim: every corpus claim in §2 is a fact that is *expressible* in
DBM but not *derivable* in it. Nothing in the corpus wants a richer logic; the
corpus wants more *sources* feeding the same logic. *Leaned on by:* ENT-4's three
closure rules, ENT-6's normalizations, and every obligation family.

### 1.6 [ENT-3] — the eleven fact sources (2907–3009)

> **2907:** The fact state is defined constructively over the conservative
> structural normal-control graph [FN-1]: each source below establishes its L0 and
> signed-goal facts at its stated point; facts flow forward along normal edges;
> kill events apply on the edges where [ENT-5] places them, with scope-exit kills
> applied before any join; merge points take the [ENT-5] join and loop heads the
> [ENT-5] loop rule; and the state queried at any point is the [ENT-4] closure of
> that flow.
>
> **2910:** Nothing else is a fact: an `ensures_clause` is only an FN-9 proof
> obligation, never a trusted source; no struct invariant, writer-stated or
> inferred loop induction, inferred summary, or unverified user-function result
> exists.
>
> **2912:** Provenance [PRV-1] is a separate judgment over finite value and
> storage components, not a fact: it establishes and kills no relation or signed
> goal, and no [ENT-4] answer depends on it.

The complete source list, with one line each on what it can and cannot see:

| id | source | what it gives | what it deliberately withholds |
| --- | --- | --- | --- |
| S1 (2945–2948) | branch facts | `+G` on then, `-G` on else, plus the exact L0 comparison projection and its exact negation | nothing from a non-comparison condition root beyond the signed goal itself |
| S3 (2950–2955) | **claim facts** | each ordered component of `Contrib(e)` on the normal continuation | D, S and F themselves are not established first; F never |
| S4 (2957–2961) | `requires` facts | `+G` at body entry, plus the projection when G's root is one admitted comparison | any child of G outside its signed decomposition set |
| S5 (2963–2964) | copy/conversion | `x = lit`, `x = p`, `y = cvt(p)` | anything computed |
| S6 (2966–2969) | length facts | `len(b) = n` at allocation, `m = len(P)`, `len(s) = len(P)` for a slice | element content |
| S7 (2971–2978) | constant-offset arithmetic | `s = p ± k` for constant k (range-guarded for wrap, unconditional after a discharged exact site); `+checked` arm equality; unsigned `iand` upper bounds; `ishl.wrap(1, n) != 0` | **`%`, `/`, `*`, `imin`, `imax`, and every two-variable operation** |
| S9 (2980–2983) | const-array element ranges | `vlo <= x <= vhi` for `c[i]` on a named const array | deeper const shapes |
| S10 (2985–2990) | boundary endpoint facts | on the success arm of the five range-bearing ops: `s <= w` and `w <= e` | **every other [SYS-8] relation**, including `next = start + required`, `entries`, and the buffer-disposition contracts (2990 says so explicitly) |
| S11 (2992–2997) | counted-range structural facts | capture equalities in the preheader; `lower_capture <= binder < upper_capture` on every true header edge | **no `binder = upper_capture` at the continuation** (2997) |
| S12 (2998–3007) | verified user normal results | an FN-9-verified `ensures` relation, substituted, on four named destinations only | a named/pending outcome, a stored whole outcome, a same-SCC callee |
| — (3009) | S8 retired | — | its midpoint family "may return as a later version's monotone addition the day a corpus program writes the shape" |

*Question the list answers:* where facts come from. *Deliberately excluded, in
one word:* **induction.** There is no ordinary-loop induction anywhere; S11 is a
*structural* recurrence the compiler owns for `for` only.

*[observation] The retired-S8 sentence at 3009 is the spec's own statement of the
project's evidence rule for this area — a fact source is added the day a corpus
program writes the shape, not before.*

### 1.7 [ENT-4] — closure (3011–3034)

> **3011:** The L0 component of the closed fact state is the least set containing
> its established and implicit facts and closed under exactly: (1) from
> `t1 - t2 <= c1` and `t2 - t3 <= c2`, derive `t1 - t3 <= c1 + c2`; (2) from
> `t1 - t2 <= 0` and a disequality between t1 and t2 in either orientation, derive
> `t1 - t2 <= -1`; (3) of two bounds on one ordered pair, the smaller constant
> subsumes.
>
> **3026–3032:** The combined state is contradictory when L0 derives `t - t <= -1`
> for any t or when both signs of one exact goal are derivable. At a contradictory
> point every L0 relation and both signs of every goal in the finite universe are
> derivable and every ordinary obligation, call goal, and FN-9 selected-return
> relation is discharged. CLM-2 checks contradiction before signs and therefore
> classifies no claim by this explosion. … an instantiated goal G is `discharged`
> when `+G` is derivable, `refuted` when `+G` is absent and `-G` is derivable, and
> `unproved` otherwise.

*Question answered:* what "the checker knows X" means — exactly three DBM rules
plus a small truth-functional Boolean reconstruction (3014–3018). *Deliberately
excluded:* everything else, and in particular (3023) "Derivability never
decomposes a merely derived parent". *Leaned on by:* CLM-2's three-way verdict,
ENT-6's obligation dispositions, FN-8, FN-9.

### 1.8 [ENT-5] — support, kills, joins, loops, delivery (3036–3128)

Five distinct jobs live in this rule and a designer should treat them
separately.

**(a) Support (3036–3049).**

> The support of an L0 fact is every tracked place occurring in its terms; every
> compiler-owned counted capture term occurring in its terms; for each length term
> len(P), the root binding of P but not P's element storage — an element write
> never kills a length fact … and every borrow or box/arena holder binding any of
> its places reads through by `deref`, a bound call-result holder included …
> **3048:** One CLM-2 contribution component has the ordinary support of its exact
> S-derived signed goal or relation.

*Answers:* what a fact depends on. This same support set is what CLM-1's locality
query walks (2742, and ENT-6 3248 "For one component, CLM-1 queries exactly
[ENT-5]'s ordinary S-derived relation or opaque-goal support, including each root
and holder"). **The locality judgment and the kill judgment read the same support
set** — that identity is load-bearing and easy to break.

**(b) Kills (3066–3067).**

> A fact dies at the earliest of: (a) a `set`/`replace` commit whose resolved
> target overlaps … the resolved place of any support member, or the
> compiler-owned update of a `for_stmt` binder …; (b) a call — user function,
> table operation, or system operation — one of whose [EFF-2] boundary-projected
> `writes` occurrences projects onto a caller place … so a callee writing only
> through one `&uniq` actual kills exactly the facts whose support overlaps that
> actual's resolved place, and a call whose row carries no `writes` kills nothing;
> (c) a consuming use of any support member's root; (d) an edge leaving the region
> of any borrow holder in its support, leaving the lexical scope of any support
> binding, or leaving the owning counted construct of any capture term…
> Scope exits are edge events: kills (c) and (d) apply on every edge leaving the
> scope, before any join at that edge's target is taken…

*[observation] Note the asymmetry that H3 lives in: **kill (b) does see the
callee's `&uniq` write** — a projected callee write kills caller facts — while
[CLM-1]'s claim-authority explicitly does **not** ([CLM-1] 3242, quoted in §1.10).
Facts and authority disagree about the same event.*

**(c) The ordinary join (3095–3103).**

> at the continuation of a `match_stmt` or `value_match`, the fact state is the
> join of the states on every arm exit edge reaching that continuation …, each
> taken after that edge's scope-exit kills and then closed [ENT-4]; an arm every
> path of which leaves by `return`, `break` … or `propagate`'s error edge
> contributes nothing there. In any nonempty join with at least one
> non-contradictory input, a contradictory all-derivable input imposes no
> constraint. **Over the non-contradictory inputs, the L0 join keeps for each
> ordered term pair the weakest (largest-constant) bound held by all and each
> disequality held by all; the opaque join keeps one signed fact exactly when that
> identical goal and sign are held by all.** The join of closed states is closed.
> A nonempty join whose every input is contradictory, and an empty join with no
> reaching edge, are each the contradictory all-derivable state.

*Answers:* the merge. It is **exactly a DBM convex hull (per-pair max of the
bound), with intersection semantics for disequalities and opaque goals.** No
widening exists because no loop iterates the state (see (e)). *Deliberately
excluded:* disjunction. Two arms with `x<8` and `x<128` give `x<128`
(explicitly, 3090); two arms with `x=0` and `x=1` give **nothing at all**, because
neither directed bound of either equality is held by both — this is the exact
mechanism behind H2's second half.

**(d) `value_if` bounded relation delivery (3078–3093).**

> Bounded relation delivery is an additional edge transfer only for the `value_if`
> carrier admitted by [GIVE-1]. On one reaching eligible `give d;` edge, evaluate
> the bare atom's value first. From the closed state at that point, take exactly
> each L0 bound or disequality whose normalized terms contain d; facts that do not
> contain d and opaque signed goals are not delivery candidates. Replace every
> occurrence of d with the receiving binding x … A non-bare, projected, consuming,
> computed, constructed, call, subscripted, **literal**, named-const,
> const-generic, capture, Z, contract-symbolic, wrong-mode, or wrong-type delivery
> forms no image; the value still follows ordinary GIVE-1 semantics. A
> `value_match` forms no delivery image under any source shape. … When at least
> one image is non-contradictory, contradictory images are neutral and the
> non-contradictory images retain for each ordered term pair the weakest
> (largest-constant) bound held by all and each disequality held by all; **a
> relation missing from one such image is not delivered.**

*Answers:* how a fact crosses a `value_if`. *Deliberately excluded, in the
spec's own word list:* **literals.** `give 0_u64;` carries nothing. That is H2's
first half, spelled out in the rule.

**(e) Loops (3105–3128).**

> **3110:** Ordinary loops carry no induction in this version: the fact state at
> the head of each iteration of `loop @l { … }` is the state before the loop minus
> every fact having a support member that a continuing kill event of `@l` may kill.
> … **3116:** Loop induction is a later version's [ENT-1]-monotone extension.
> **3118–3128:** A counted `for_stmt` uses one compiler-owned structural
> recurrence, not writer-supplied induction. First its preheader establishes the
> S11 capture equalities … Second, its head state is that one closed post-capture
> state minus every fact having a support member that a continuing kill event of
> this counted loop may kill. … Third, on each true header edge, S11 adds the two
> structural body-entry bounds…

*Answers:* how a loop head is computed — **subtraction, once, not a fixed point.**
There is no widening operator and no iteration to a fixed point in the fact
state at all. *Deliberately excluded:* every relation the loop maintains. This is
the source of ~29% of the corpus's claims (§2).

### 1.9 [ENT-6] part 1 — the obligations (3130–3181)

> **3130–3131:** An obligation is one normalized relation attached by a numbered
> rule to one source node, instantiated with that node's exact operands read as
> terms or constants; an operand that is not a term or constant leaves the relation
> underivable, never ill-formed. **This version attaches exactly four obligation
> families.**

The four, with their constrained subjects (3250–3256):

| family | goal | protected? | subject(s) |
| --- | --- | --- | --- |
| SubscriptBounds (3132) | `i < len(P)` at every `psuffix` | yes | `i` |
| IntegerDomain (3133–3153) | the exact `.defined` goal of every proof-required exact integer op | **no** | — |
| AllocationFit (3156–3161) | `buffer_fits<T>(n)` | yes | `n` |
| SystemRange (3163–3167) | `start <= end`, then `end <= len(buffer)` | yes | goal 0: start, end; goal 1: end |

> **3169–3173 (the writer-facing repair):** The mechanical fix for any unproved
> family is one dominating branch establishing its canonical goal, or a
> CLM-2-admissible residual claim only when the predicate is a universally true
> current-function-local theorem the normative checker cannot derive — for a
> subscript in canonical ANF, one `let` binding `len(P)` followed by one such local
> claim on, or `if` over, the admitted comparison [CLM-1, ENT-3]. After
> complete-state success for a protected family, a [PRV-2] or [PRV-3] rejection
> makes the assertion-only route unavailable… With at most that one rebinding step
> per nested offset, the fallback makes the goal writable, **at a per-site cost
> from zero where facts already prove the bound to one retained claim where the
> missing theorem is CLM-1-local**; rebinding a user-call or system-call result
> never makes it local, and cross-function behavior instead requires a verified
> FN-9/S12 relation or ordinary control.

*Question answered:* the writer's two-route menu — **branch, or claim** — and the
promise that one of the two is always available. *[observation] That promise is
exactly what H2 falsifies for the delivered-constant shape: there the branch
route is what produced the value in the first place, and the claim route is
refused by CLM-1.*

### 1.10 [ENT-6] part 2 — claim authority (3215–3248), the v0.39 paragraph

> **3215–3220 (the frame):** For CLM-1 only, ENT-6 also computes one independent
> finite forward **claim-authority** state over the structural normal-control
> graph. Claim authority is not an entailment fact, optimizer fact, callee summary,
> or [PRV-1] provenance pair, and it grants no operation authority. Each component
> is `Local` or `BoundaryResult(witness)`; component join retains `BoundaryResult`
> when either input has it and retains the earliest witness in stable source order.
> The component tree is structural and finite: a scalar or opaque value has one
> plain component; a struct has its recursively selected declaration-order fields;
> an enum has its tag and recursively selected declaration-order payload fields; an
> array, slice, or buffer has its length and one conservative all-elements
> component; and a box, arena, or borrow holder retains both the holder path and
> every selected dereference path used by a claim support. … Construction and
> projection are component-sensitive: a boundary field or payload does not taint an
> independent local sibling…
>
> **3222–3225 (the seed):** Every source parameter component, command-entry
> parameter component, literal, named const, and otherwise untainted local
> initializer begins `Local`; this judgment does not classify external input
> provenance. **Every result component of every ordinary user call and every system
> call begins `BoundaryResult`**, including a scalar, tag, payload, aggregate
> field, length, element, box or arena content, borrow holder, and value read
> through that returned holder. **This seed is unconditional:** it does not inspect
> or substitute the callee body, arguments, effect row, [PRV-1] class, a system
> component's external/internal/dependent class, or an FN-9/S12 relation. An
> `ensures` relation remains an independently verified fact for direct caller
> consumption and never declassifies any component of its returned value.
>
> **3227–3231 (transfer):** Ordinary copy or move, conversion, reinterpretation,
> arithmetic, float, Boolean and enum operations, `imin`, wrapping identity
> operations, allocation-fit operations, and every other total value operation join
> the authority of the value components they read into the result components they
> produce. … A place read obtains the reaching authority of its selected storage
> component and joins the authority of any value used to select a conservative
> element; an explicit `set` or `replace` transfers its right-hand side and selector
> authority into the written component. An unconditional explicit write to one
> statically exact whole value or exact field component is a strong replacement and
> may clear an older boundary marker when its right-hand side is Local … Control-flow
> joins combine corresponding components and never subtract a boundary witness.

*Question answered:* which of the current function's values are *its own*.
*Deliberately excluded:* truth, provenance, and any inspection of the callee.
The unconditional seed is the cheat-proofness core: a claim cannot be made local
by any property of the callee, however verified.

**The v0.39 control-dependence paragraph, verbatim (3233–3239) — the text batch
0102 wrote and the text 0106 is asked to redo:**

> Claim authority deliberately includes control dependence although [PRV-1]
> provenance does not, and it includes exactly the control dependence a selection
> carries.
> A `BoundaryResult` condition, match scrutinee or tag, counted endpoint, or other
> selector chooses an edge; its witness joins each matching binder that edge's arm
> introduces, each value `value_if` or `value_match` delivers along it, and, at
> each ordinary match reconvergence, loop head, and loop exit the selector reaches,
> exactly those components whose reaching definition on one incoming edge is a
> different definition occurrence from their reaching definition on another.
> Two reaching definitions are the same occurrence when they are one definition of
> that component, not when two separate definitions compute equal values;
> `value_if` and `value_match` deliver a selected value in every case, so selecting
> constants on the two arms or selecting the same local value on both arms does not
> declassify the delivered value.
> Standing on a boundary-selected edge is not itself a selection.
> An ordinary binding, a computed value, or a storage write whose own operands are
> every one Local — a literal, a named const, an ordinary parameter, or another
> Local value — is Local, and stays Local across a reconvergence, loop head, or
> loop exit whose every incoming edge reaches it through that one definition,
> whether it stands inside the selected arm or in post-join state.
> Thus writing a local constant on one arm and joining it with the other arm's
> older definition, and updating loop-carried state under a boundary-selected loop,
> each retain the selector's witness at the join, while a definition formed after
> the join from literals, named consts, parameters, and other Local values is Local
> although a boundary result selected the edge that reaches it.
> So a `match` on a system-call result whose `Err` arm returns leaves a following
> `let seed = 3209_u64;` and `let offset = seed % 64_u64;` Local, and `claim guard:
> ilt(offset, 64_u64)` is admitted; the same claim over a value that reads the
> delivered payload, a binder joined from two arms, storage the selected edge wrote
> and the other edge did not, or state a boundary-selected loop updates remains
> non-local and is refused.

*Question answered:* when does *choosing the path* count as *reading the value*.
*Deliberately excluded:* pure position (the repealed v0.38 clause). *Leaned on
by:* nothing else in the spec — this paragraph has no downstream consumer other
than CLM-1 2742. *[observation] It is also the only paragraph in §18 whose
criterion is an identity relation between analysis artifacts ("the same definition
occurrence") rather than a property of values or of the program text. Every other
rule in the section is stated over terms, goals, components, or source nodes.*

**The H3 carve-out, verbatim (3241–3243):**

> A call's result seed is the only call event added by this first locality version.
> A user or system call's possible write through an `&uniq` actual does not by
> itself change claim authority for that caller storage; ordinary effect, kill,
> provenance, and obligation judgments remain unchanged, and an explicit later
> write of a boundary-derived right-hand side still transfers normally.
> **Extending locality to call-written storage is an amendment-level accepted-set
> change rather than an implementation choice.**

*Question answered:* none — it is an explicit deferral. *Deliberately excluded:*
soundness reasoning. The paragraph states a scope boundary and names the class of
change needed to move it. *Leaned on by:* ENT-1 2856, which lists "adding or
removing a `BoundaryResult` seed" among amendment-level changes.

**Witness and tie-break (3245–3248):**

> One boundary witness contains the introducing call's NodePath and kind, plus the
> user callee's source declaration origin and source name or the system operation's
> `system_declaration_ordinal` and spelling. When more than one witness reaches a
> component, the least call NodePath wins, with boundary kind and the stable callee
> identity used only as a deterministic tie-break at one path; no scratch or dense
> identity is publishable. The authority analysis is computed once before S3, U, B,
> `Eligible`, or any `Full-minus` mask and is reused unchanged by every claim
> component query. For one component, CLM-1 queries exactly [ENT-5]'s ordinary
> S-derived relation or opaque-goal support, including each root and holder;
> canonical normalization may add a fact identity but never subtract authority
> support read by the retained S expression.

*[observation] The *witness* tie-break is fully pinned here. The *carrier* tie-break
— which support place is named in the diagnostic when two supports observe the
same earliest witness — is not; see H5.*

### 1.11 [PRV-1..3] — the other, older locality judgment (3305–3412)

> **[PRV-1] 3305–3314:** Provenance is a derived two-class explicit-dataflow
> judgment over exactly the finite components whose dependency transfer [ENT-6]
> retains. … Every component carries the pair `(unconditionally external,
> parameter datums)` … Each labelled `command` entry parameter instead begins
> unconditionally external … A [SYS-2] result or writable component begins with
> exactly its table-fixed unconditional bit … **These entry and system components
> are the only unconditional external origins.**
>
> **3327–3330:** No branch, match arm, loop guard, variant tag, or other control
> choice contributes provenance merely by selecting a path. No target-address
> operand contributes merely by selecting a write. **There is no path-sensitive
> storage, recursive payload path, implicit-flow analysis, integrity judgment,
> writer-spelled provenance annotation, trusted assertion, or optimizer
> assumption.** An external value used only as a bound, base, target address, or
> unrelated goal operand therefore does not become a constrained subject by
> association.
>
> **[PRV-3] 3404–3407:** The unasserted state removes exactly S3 claim
> establishment. S1 branches, every S4 requirement source, S5, S6, S7, S9, S10,
> S11, every kill and join, and [ENT-4] closure remain unchanged; B additionally
> removes every S4 source. **Thus a `claim` may not authorize an external
> constrained subject**, while an internal subject may use one only when CLM-2 also
> proves that exact occurrence and every contribution individually necessary for an
> allowed terminal root.
>
> **[PRV-2] 3386:** A `claim` is not a repair for an unconditionally external
> constrained subject.

*Question answered:* may a claim be the *only* thing standing between attacker
data and a partial operation? No. *Deliberately excluded:* control dependence —
stated twice and emphatically (3327–3329). *[observation] This is the single
sharpest comparison available inside the repository: **two locality judgments,
computed over the same program, disagree on exactly one axis — control
dependence — and each says explicitly that it made that choice on purpose**
(CLM-1 3233: "deliberately includes … although [PRV-1] provenance does not";
PRV-1 3327: "No branch … contributes provenance merely by selecting a path").
Whatever the redesign concludes, the terrain already contains a matched pair of
answers to the same question, and the reason they differ is that they defend
different things: PRV defends against *external data reaching a domain
obligation*, CLM defends against *an unverified cross-function promise becoming a
lemma*.*

### 1.12 The satellites

**[GIVE-1] 286–293** — only a `value_if` whose derived mode is `own` and whose
type is an ENT-2 fragment integer may carry a relation, and only through "one bare
tracked own-value binding of the exact receiving type… A literal, named const,
const-generic constant, Z, counted capture, contract definition, symbolic result
datum, projected place, consuming atom, or any other atom may still be admitted in
its own grammar role but carries no relation through a value initializer."
*Answers:* the exact syntactic carrier for delivery. *Excludes:* literals, again.

**[OP-4] 877–878** — a subscript whose bound is not discharged is an OP-4
rejection carrying the residual, whose "mechanical fix is a dominating branch
establishing the residual [ENT-3], or, only when the residual is an independently
true theorem outside the normative checker, a CLM-2-admissible residual `claim`
with a complete exact `because` record".

**[DIAG-1] 1846–1860 — the fixed claim diagnostic schedule.** FN-1 reachability →
CLM-1 type/shape/name/five-field → CLM-1 fact-free formation → CLM-1 component
authority (source-schema before concrete; least non-local component wins) →
contradiction-first lifecycle → ordinary OP/FN/PRV judgments → CLM-2 residuality.
1854: "This schedule prevents an invalid candidate from supplying another
candidate's baseline and prevents a premature unused-claim error from hiding an
ordinary proof or provenance defect." The locality payload (1857) retains "the
claim name and NodePath, least failing component ordinal, earliest boundary-call
NodePath, boundary kind, **the first source-ordered support carrier that observes
that same earliest witness** with its canonical source spelling, and the callee's
stable identity". Its two fixed restructurings (1859) are the only per-rule
teaching text in the area:
> for a user result its restructuring is `publish the required cross-function
> relation as an exact verified ensures clause on the callee and remove this caller
> claim`; for a system result it is `use the system operation's specified fact or
> typed outcome, or branch on the returned value; do not claim an unstated
> system-result property`.

**[DIAG-3] 1960–1986** — the trap record is
`{"rule_id":"CLM-1","message":<claim IDENT>,"function":<fn IDENT>,"node_path":[…]}`
plus one LF. 1977: "`message` is the claim's exact IDENT spelling; the
justification STRING is compile-time data and does not appear in the record."
1978: no other construct produces a DIAG-3 record. *[observation] The runtime
report therefore carries the claim's **name** and nothing else — the name is the
whole of the writer's runtime-facing channel, which is why 2752–2754 goes to the
trouble of taking claim names out of every declaration domain so `len` and `wrap`
are legal names.*

**[PAR-3] 2074–2079** — the erroneous-execution clauses: exactly one DIAG-3
record, schedule may pick which false claim it names and which already-permitted
transitions became visible, and "No permission, submission, completion, or fast
path reads a trap latch or pays any other cost whose purpose is to stabilize this
erroneous execution. A correct program executes no false `claim`, so the
impossible branch cannot narrow or surcharge its execution [SCOPE-4]." In
`compiler/src/semantic/staged_permission.rs` this is implemented by treating
`CheckedStatement::Claim` as an ordinary straight-line statement whose read
footprint is its condition (lines 608, 1136, 1669) and whose trap edge is
explicitly *not* a control edge (818–826). **[PAR-3] does not consume claims as
proof at all** — it consumes them only as a footprint and a non-edge.

**[SYS-8] 2537–2577** — the world's answers. Two static obligations per call
(`start <= end`, `end <= len(buffer)`), no operation-internal range check, and a
long list of exact outcome contracts: `start <= next <= end` on success
(2564, imported by S10); `ReadBytes(next)` only for `next > start`; exactly
`[start, next)` may have changed; `ListBytes(next, entries)` where `[start,next)`
holds exactly `entries` complete records; `host_copy_bytes` success returns
`Ok(next)` with `next = start + required`. **ENT-3.S10 imports two of these
relations and 2990 says the rest "are retained checked-program facts and are not
L0 fact sources in this version".**

---

## 2. The evidence corpus

Every `claim` written anywhere in the tree, counted mechanically at
`b1367c82`.

**Totals.** 135 claim occurrences in 101 `.wf` files under `tests/`, out of 517
conformance cases and 25 real programs. Split by home:

| home | files | claims | what they are |
| --- | --- | --- | --- |
| `tests/conformance/cases/` | 81 | 102 | normative verdicts; the rule families are 25 `*clm*`, 25 `ent*`, 18 `prv*` cases |
| `tests/programs/` | 7 of 25 | **18** | real programs — the only "someone wanted to write this" evidence |
| `tests/codegen/cases/bounds/` | 13 | 15 | bounds fixtures; 12 are one-line `ieq(value, N)` drift oracles and 3 are `False()` preemption markers, i.e. **fixture instrumentation, not writer evidence** |

**The real-program column, in full** (claims / lines):

```
wfgrep.wf                      5 / 1417
percent_decode.wf              4 /  168
fir_filter.wf                  3 /   73
ipv4_checksum.wf               2 /   95
utf8parse.wf                   2 /  276
raw_deflate_dynamic_decode.wf  1 /  371
par_layout.wf                  1 /  268
--- and eighteen programs with zero, including ---
raw_deflate_vectors.wf         0 /  863
dir_walk.wf                    0 /  569
byte_string.wf                 0 /  407
raw_deflate.wf                 0 /  364
sha256_abc.wf                  0 /  130
```

18 claims in ~6,600 lines of real program. The blind-writer trials add
**zero claims in 1,694 lines** (`docs/done/0098-blind-writer.md:40`: "Zero
`claim` statements in 1,694 lines. Every subscript, every `%` and `/`, every
system range call was discharged by ordinary `if` branches and `len()`
rebinding. The proof obligations — the part of this language everyone expects to
be the wall — were not the wall."). `docs/done/0100-writer-defaults-2.md`
records no written claim either.

*[observation] Per the owner's standing ruling, none of these counts is evidence
that a language need does or does not exist. They are useful for exactly one
thing: showing **which shapes** a writer reached for a claim over, and what
happened. The distribution below is that, and nothing more.*

### 2.1 Classified by what the claim's operands read

The `checker gap:` field is the writer's own statement of why the prover failed,
so it classifies the corpus more honestly than the predicate text does. Tally
over all 135 (some claims are counted in one bucket only, by their dominant
reason):

| bucket | count | representative gap text |
| --- | --- | --- |
| **arithmetic over locals — remainder result range** | **46** | "ENT proves the remainder operation domain but does not publish its result range" (35 verbatim, + 5 "through the wrapping offset", + 6 variants) |
| **loop-carried state — induction / strict loop range** | **39** | "ENT does not derive the strict loop range from the equality exit and loop recurrence"; "ENT carries no induction fact across the bytes-loop backedge"; "ENT does not derive this coupled ordinary-loop invariant" |
| **payloads and nominal fields** | 12 | "ENT does not publish a nominal payload value through borrow-mode match"; "ENT does not retain constructor values through nested nominal field projections" |
| **cross-function values (deliberate refusals + FN-8 shapes)** | 12 | "ENT does not publish an uncontracted user-call argument value into apply"; "a caller claim has no authority to substitute an unverified callee body for a published boundary fact" |
| **`imin`/`imax` result range** | 5 | "ENT does not publish the result range of imin" |
| **lengths and extents from reads** | (inside the above) | "ENT does not publish the borrowed buffer length through the child call" (2); "ENT does not correlate the two borrowed column lengths" (4) |
| **delivered values** | 2 | "a value_if delivers the value its selector chose … CLM-1 must refuse the claim whatever the arms deliver"; "the entailment state carries no residue for a literal remainder" |
| **joined definitions / arm-written storage** | 2 | "the two incoming reaching definitions differ, so the call result chose which one arrives" |
| **literals only** | 2 | "the entailment state carries no residue for a literal remainder, so the bound is left to this executed check" (the two `accept-clm1-local-claim-after-boundary-*` cases) |
| **callee-written `&uniq` storage** | **0** | *no claim in the tree has this shape; the carve-out (§1.10, 3241–3243) has no corpus case at all* |
| deliberate `False()` / name-collision / redundancy fixtures | ~7 | "there is no checker gap for a direct False() predicate" |

Consumers named by the 135 claims (a claim may name several): subscript 48,
`requires` 41+4, IntegerDomain 20, exact addition 14, a call 12, buffer/allocation 9,
division 4, system range 1. **Every named consumer is one of the six terminal
roots CLM-2 admits.** *[observation] No writer in the tree has ever tried to use
a claim for anything outside the closed root list — which is weak evidence that
the root list is right, and strong evidence that it is not the pinch point.*

### 2.2 The pinch point, stated plainly

Two shapes account for **85 of 135 claims (63%)**:

1. `let r = x % C;` then `claim: ilt(r, C)`. The checker *proves the `%` operation's
   own domain* (IntegerDomain: `C != 0`) and then discards the result range. 46 claims.
2. A loop whose induction variable or accumulator satisfies a relation the loop
   maintains. 39 claims.

Neither is a cross-function question, an aliasing question, or a locality
question. Both are **missing ENT-3 sources for facts already expressible in ENT-2's
DBM vocabulary.**

### 2.3 What happened to each class under v0.38 → v0.39

- **The 63/63 fuzzer rejections** (`docs/done/0097-differential-fuzz.md:205–265`):
  all `NonLocalClaim`, over 2004 accepted programs, on one minimized shape — a
  literal-only claim standing after a `match` on a system call whose `Err` arm
  returns. Zero true positives in that campaign. Under v0.39 the shape is
  admitted; the 0102 re-run measured **5/203 refusals at v0.38 vs 0/203 at
  v0.39**, all five `NonLocalClaim`, no program moving the other way
  (`docs/done/0102-clm1-narrow.md:281–300`).
- **Seven conformance cases added by 0102** and their measured before/after
  (0102:196–204): three accepts moved (`after-boundary-exit` was refused with
  carrier `offset`, `inside-selected-arm` with carrier `position`), four rejects
  are unchanged: `claim-on-selected-payload`, `claim-on-delivered-selection`,
  `claim-on-storage-written-under-selection`, `claim-on-loop-carried-update`.
- **Ten pre-existing compiler tests** asserted the repealed clause; nine became
  acceptance tests, and one —
  `control_authority_rejects_a_component_without_binding_supports`, claiming
  `ieq(four, 4_u64)` — became
  `a_local_named_const_component_reaches_the_redundancy_judgment`, because with
  locality granted the occurrence reaches CLM-2 and is refused as **redundant**
  (0102:220–226). *[observation] That single test is H1 and H4 shaking hands: the
  narrowing did not make the program compile, it moved the refusal from one rule
  to another.*
- **No pre-existing conformance verdict changed**; 510 cases byte-identical
  across the two compilers, 26 program sources byte-identical IR modulo the
  version banner (0102:240–251).
- **Historic churn from prover strengthening** (`docs/roadmap.md` PROOF-8):
  v0.25's counted range moved SHA-256 from `0/9` to `9/9` claim-independent
  obligations **"after deleting four claims"**; a later migration "replaces
  eleven DEFLATE claims with value branches"; DEFLATE at one point carried
  "sixteen claims, five redundancy advisories" and later "twelve claims … seven
  load-bearing and five redundant". Every prover improvement in this project's
  history has been accompanied by a corpus edit deleting claims.

---

## 3. The prover today

Read from `compiler/src/semantic/entailment/{flow.rs, state.rs, term.rs}`
(8670 + 4780 + 185 lines) and `compiler/src/semantic/claim_locality.rs` (2122).

### 3.1 What the state actually holds

`FactState` (state.rs) is, per proof view:

- `bounds: HashMap<(TermId, TermId), i128>` — the DBM. One constant per ordered
  term pair, meaning `left - right <= bound`.
- `distinct: Set<(TermId, TermId)>` — unordered disequalities.
- `opaque: Set<(GoalId, GoalSign)>` — signed opaque goals.
- `origins`, `goal_origins`, `ambiguous_goal_origins`, `outcomes` — per-binding
  maps recording *which comparison / goal / outcome a Bool binding came from*
  (ENT-3's comparison-origin and integer-domain-origin machinery), plus the
  checked-arithmetic/system-outcome origin used by S7 and S10.
- Parallel `*_proofs` and `*_candidates` maps carrying `DerivationId`s: every
  fact keeps its derivation, and each pair keeps a *list* of candidate proofs
  from which `select_bound_candidate` (state.rs:2435) picks the strongest bound
  and, at equal bound, the "better" (shallower) derivation.
- `all_derivable: bool` + `contradiction: Option<DerivationId>` — the explosion
  flag, checked before everything.

Three such states travel together as `ViewStates { complete, unasserted,
s4_blinded, entry_images }` (flow.rs). Every source, kill and join is applied to
all three; the views differ only in that `unasserted` suppresses S3 and
`s4_blinded` also suppresses S4.

### 3.2 What the merge at a join actually does

`Analyzer::join_views` (flow.rs:6228) fans out to `join_at` (state.rs:4037) once
per view with a shared `FlowEventId`, and `entry_images` takes the **element-wise
`min`** over the incoming edges' images (6242–6249). `join_at` calls
`join_at_once` (state.rs:4071), which is the real merge:

1. **Close every input first** (4086–4089), *before* filtering — the comment
   states why: "a contradiction established immediately before an edge is already
   the absorbing all-derivable state even when no kill had occasion to
   materialize its flag."
2. **Drop contradictory inputs** (4090–4094). If *none* survives, intern a
   `JoinContradiction` node over every edge's contradiction proof and return the
   all-derivable state (4095–4114). This is ENT-5 3099 exactly.
3. **Bounds** (4116–4162): iterate the *first surviving input's* pair keys in
   sorted order; for each pair, walk the other surviving inputs and take the
   **maximum (weakest) constant**, but only if **every** other input has that pair
   at all. Missing in one input ⇒ pair dropped. Then intern one `JoinBound` node
   whose parents are, per edge, either that edge's contradiction proof or that
   edge's own proof of the *joined* (weakened) bound.
4. **Disequalities** (4163–4196): plain set intersection across surviving inputs,
   one `JoinDistinct` node each.
5. **Opaque signed goals** (4199–4232): plain set intersection on `(goal, sign)`,
   one `JoinGoal` node each. The comment at 4197 is the design statement:
   *"Comparison and outcome origins are path conditions, not facts; one survives a
   join only when every contributing path carries the same one."*
6. **Origin maps** (4233–4259): `origins`, `outcomes`, `goal_origins` are
   retained only where **every** surviving input maps the binding to the
   *identical* value; `ambiguous_goal_origins` is intersected.
7. The result's `bound_candidates` / `distinct_candidates` are reseeded to exactly
   the one joined proof per pair (4260–4267) — a join discards the incoming
   candidate lists.

Then `join_at` runs a **second pass that the specification does not mention by
name** (4045–4068): if the joined state is non-contradictory *and* any selected
relation's proof `depends_on_postcondition_call`, it rebuilds every input with
postcondition-dependent proof candidates removed (`retain_non_postcondition_candidates`,
2555), re-joins, and merges the resulting *candidates* back
(`merge_relation_candidates_from`, 2561). Because `add_bound` →
`select_bound_candidate` keeps the strongest constant, this **cannot change which
relations the joined state derives**; it adds an S12-independent derivation for
each relation that also holds without one, so that later `depends_on_postcondition_call`
/ non-explosive-ancestry queries and the PRV view partition see the
postcondition-free route when one exists. *[observation] This is real behaviour
with no sentence in §18 attached to it; a redesign that reworks the join must
carry it or explain why it is unnecessary.*

**So, precisely: the merge is a per-pair convex hull on the DBM, intersection on
everything else, plus the requirement that a pair be present on every surviving
edge.** There is no disjunctive state, no path condition, no arm-indexed
alternative, and no widening — because nothing iterates.

### 3.3 Loops

There is **no fixed point over fact states**. `ENT-5` 3110/3120 is implemented as
a *subtraction*: `collect_continuing_loop_kills` (flow.rs:7908) walks the body
once, collecting every kill event on an edge from which the loop's own body entry
is reachable without leaving the body (`loop_block_reaches` /
`loop_statement_reaches`, 7824/7837), and `apply_loop_kills` (8132) removes every
fact whose support any of them may kill. The head state is then *entry state
minus those*, once. For `for`, S11 re-adds the two body-entry bounds on each true
header edge.

Consequence: a relation that holds at the head of every iteration but whose
support is written in the body is simply gone. That is the 39 loop-induction
claims of §2.

### 3.4 `value_if` delivery, in code

`eligible_delivery_terms` (flow.rs:6279) returns `None` unless the given value is
a `CheckedExpression::Binding` with `consume_root: false` whose summary has
`delivery_carrier` set and no holder and no implicit deref and whose type equals
the receiver's. **A `CheckedExpression::Constant` never matches** — so
`give 0_u64;` produces `ViewStates::default()`, an empty image. `delivery_edge_state`
(6343) then keeps only relations whose terms contain the carrier and whose proof
`depends_on_explicit_relation`, substitutes carrier→receiver, and interns a
`PostconditionGive` node; the per-edge images are joined by
`establish_delivery_join_view` (6473) under the same weakest-bound-held-by-all
rule. An empty image on any non-contradictory edge therefore deletes everything.

### 3.5 Claim authority, in code

`ClaimAuthorityAnalysis::analyze` (claim_locality.rs:758) short-circuits to an
empty analysis when the function contains no claim (765–770), then walks the body
once, with an inner fixed point per loop.

- `AuthorityValue` (243–259) is a sparse component tree: `ty`, `definition:
  DefinitionId`, `identity: Option<BoundaryWitness>` (survives a strong write
  through an owned deref, so a returned box cannot be laundered), `uniform:
  Option<BoundaryWitness>` (applies to all descendants while `children` is empty),
  and `children: Vec<(AuthorityStep, AuthorityValue)>` with steps
  `Field | EnumTag | EnumPayload | Element | Length | Deref` (234–241).
- `DefinitionId` (104–145) is `{ site: usize, kind }` where `site` is
  **`std::ptr::from_ref(stmt).addr()`**, the address of the checked statement, and
  `kind ∈ {Entry, Written, Taken, Binder, Merge, Fused}`. `Entry` and `Fused`
  reserve site 0.
- `ControlAuthority` (163–228) is a `Vec<ControlFrame { site, witness }>` sorted
  by site. `with_added` pushes/strengthens a frame; `remove` discharges one;
  `acquired(edges)` returns the earliest witness among frames the *edges* hold and
  `self` does not — "the boundary decisions taken between the merge's dominator
  and the merge, and nothing else" (0102:114–117).
- `AuthorityValue::merge` (584–630) is the v0.39 rule: **if the two incoming
  `DefinitionId`s differ**, `join` the two values, union the selection witness,
  and stamp the merge's own id; **if they agree**, recurse and take only the
  ordinary authority union — no selection witness at all.
- Call seeding (1370–1394): `UserCall` and `SystemCall` both produce
  `AuthorityValue::uniform(result_ty, Some(BoundaryWitness{kind, call}))`,
  unconditionally, after evaluating (and discarding) the arguments' own witness.
- Three sites keep an unconditional selector: the match binder (1105), the
  `value_if`/`value_match` delivered value (1165–1166), and the counted binder
  (1300).
- Frame discharge: at an exhaustive match reconvergence with no escaping edge
  (1187–1194), at a fully-delivering value initializer (1158–1160), and at a loop
  exit whose body neither returns, gives, nor breaks outward (1266–1272).
- The query, `ClaimAuthorityAnalysis::witness` (800–884), walks the claim's
  snapshot with the support's projections, following holder chains for `Deref` and
  appending `Length`; it returns the earliest witness on that path.
- The rejection, `Analyzer::claim_locality_failure` (flow.rs:6802–6848), iterates
  `Contrib(P)`'s components in ordinal order, and for each iterates
  `claim_component_supports` (6757), keeping a support only if
  `witness.source_cmp(current).is_lt()` — **strictly** earlier. The first
  component with any witness rejects.

### 3.6 What the prover cannot see, and why

| gap | deliberate or accidental | citation |
| --- | --- | --- |
| remainder / division result range | **deliberate**: ENT-6 3139 excludes "a normalization with alternative positive clauses such as signed division/remainder" from admitted claim predicates, and S7 simply has no `%` row. The exclusion is about *uniqueness of decomposition*, not about difficulty. | 2739, 2971–2978 |
| `imin`/`imax` result range | accidental-by-omission: `imin` is listed in ENT-6 3227 as a *total value operation* for authority transfer but has no ENT-3 source | 3227 vs §1.6 table |
| loop induction | **deliberate**, and dated: "Loop induction is a later version's [ENT-1]-monotone extension" | 3116 |
| two-variable arithmetic | **deliberate**: "Two nonconstant add, subtract, or multiply operands have no L0 normalization route" | 3146 |
| multiplication, general linear arithmetic | **deliberate**: DBM only | 2901 |
| any relation among 3+ terms | **deliberate**: DBM only | 2901, 3011 |
| aliasing precision | **deliberate and asymmetric**: term identity under-approximates aliasing, kills over-approximate it | 2873 |
| most [SYS-8] outcome relations | **deliberate, dated**: "The remaining [SYS-9] relations are retained checked-program facts and are not L0 fact sources in this version" | 2990 |
| `value_match` delivery | **deliberate**: "A `value_match` forms no delivery image under any source shape" | 3085 |
| delivered literals | **deliberate**: literals are in GIVE-1's excluded carrier list | 292, 3084 |
| struct invariants, ensures-as-source, inferred summaries | **deliberate** | 2910 |
| S8 midpoint family | **deliberate, struck, and revivable** | 3009 |
| the join losing `x=0 ⊔ x=1` | **structural**, not stated as a decision anywhere: it falls out of "weakest bound held by all" over a DBM with no disjunction | 3097 |

---

## 4. The five holes with their artifacts

### H1 — control dependence: a blanket, then a narrowing, neither derived

**What v0.38 said** (quoted at `docs/done/0102-clm1-narrow.md:38–40` and
`docs/done/0097-differential-fuzz.md:254–260`):

> When a `BoundaryResult` condition, match scrutinee or tag, counted endpoint, or
> other selector chooses an edge, its witness joins every binder, delivered value,
> or storage write whose reaching definition is selected by that edge, including
> `value_if`, `value_match`, ordinary match, `give`, loop-carried updates, and
> **post-join state**.

**Measured consequence:** 63 of 63 rejections over 2004 generated programs, all
`NonLocalClaim`, zero true positives (0097:205–216 for the tally, 289–305 for the
campaign block `rejections by cited rule / CLM-1 63 3.0% of attempts`; the spec's
own META header at line 7 states it as "Every one of that campaign's 63 rejections
over 2004 generated programs was this shape"). The minimized pair
differs by one line and the rejected member's claim is `ilt(offset, 64_u64)` where
`offset = 3209_u64 % 64_u64` — a fact whose truth reads nothing at all.

**What v0.39 replaced it with:** §1.10 above. The criterion is *definition
occurrence identity*.

**How that criterion is implemented:** `DefinitionId.site` is the **address of the
checked statement** (`claim_locality.rs:99–107, 142–145`). 0102 records four
unenforced soundness preconditions for that choice (0102:379–387):

> The identity is a checked-statement address, sound only because the analysis is
> per-function, the checked AST outlives it, identities are compared solely between
> two reaching definitions of one component, and site 0 is reserved. Nothing in the
> type system or a test enforces those preconditions; a refactor that cached or
> compared identities across allocations could silently equate two distinct
> definitions and drop a selector's witness.

0102's own "Not done" section lists three further open items: no probe recorded,
no analysis-time measurement on a large function (frames now live longer than
under v0.38 and `acquired` scans the frame vector once per edge frame), and the
carrier tie-break (H5).

**The shape of the hole.** *[observation]* The v0.38 rule and the v0.39 rule are
both **positional descriptions of where a witness spreads**, not derivations from
what a claim is for. The spec sentence at 3233 says control dependence is included
"deliberately", and the sentence at PRV-1 3327 says it is excluded "deliberately",
and neither states the premise from which either follows. The one derivable
premise in the repository is the constitution's W3 clause: *a claim may not "use a
claim to restate an unstated or stronger result property"* of a callee or system
result. What a redesign has to derive is: **which values can carry an unstated
callee property into a claim's truth, and does an edge choice carry one?** The
fuzzer's minimized pair is the sharpest available observation on that question —
the claim's truth is identical on every path, and 0097 says so ("although the
claim's truth reads nothing the system returned") — and it separates
"the selector's *value* is in the predicate's support" from "the selector's *choice*
put me here."

### H2 — the delivered-value squeeze

Two rules, each individually reasonable, close on the writer from both sides.

**Side one, the entailment:** delivery is a `value_if`-only, bare-binding-only,
L0-relations-only transfer. Literals carry nothing — GIVE-1 292 lists "literal"
among the atoms that "carr[y] no relation through a value initializer", and
`eligible_delivery_terms` (flow.rs:6284) enforces it by matching only
`CheckedExpression::Binding`. And even for bindings, "a relation missing from one
such image is not delivered" (3091), so two arms delivering *different constants*
deliver nothing: the ordinary DBM join keeps `x ≤ 1` (weakest bound held by all)
but loses `x = 0` and `x = 1` and every disequality that differs.

**Side two, CLM-1:** `value_if` "deliver[s] a selected value in every case", so a
boundary selector's witness joins the delivered value **unconditionally** (3234;
`claim_locality.rs:1165`), whatever the arms deliver.

**The artifact:** `tests/conformance/cases/reject-clm1-claim-on-delivered-selection.wf`,
a normative reject, in full:

```whitefoot
fn hidden_true() -> result: own Bool pure { return True(); }

fn read(values: own array<u8, 4>, unused: own u64) -> result: own u8 traps {
  let condition = hidden_true();
  let picked = if condition { give 0_u64; } else { give 1_u64; }
  claim reviewed_delivery: ilt(picked, 4_u64) because "…checker gap: a value_if
    delivers the value its selector chose, and this selector is a call result, so
    CLM-1 must refuse the claim whatever the arms deliver…";
  return values[picked];
}
```

`picked` is 0 or 1. `values` has length 4. The predicate is true on every
execution and its truth reads nothing the callee returned. Under v0.39 this
program **does not compile**, and the writer's two documented routes (ENT-6 3169:
"one dominating branch establishing its canonical goal, or a CLM-2-admissible
residual claim") are both closed — the dominating branch is the `value_if` itself.

The sibling artifact is `reject-clm1-claim-on-loop-carried-update.wf`, where
`cursor` is literal `0_u64` before a loop and literal `1_u64` inside it and the
loop's endpoint is a call result: same squeeze, different construct.

And the third, closing the triangle: the *accepted*
`accept-clm1-local-claim-after-boundary-exit.wf` states its own checker gap as
> "the entailment state carries no residue for a literal remainder, so the bound
> is left to this executed check"

— i.e. even the case the narrowing was written to admit is admitted only because
the prover cannot do `3209 % 64 < 64`, a ground arithmetic fact. *[observation]
Both halves of H2 are the same missing capability seen from two rules: the
entailment has no notion of "the set of values this expression can produce", so it
cannot deliver a constant, cannot join two constants, and cannot fold a ground
remainder — and CLM-1, having no such notion either, must treat every delivered
value as chosen.*

**Why the obvious fix is forbidden.** A union fact for delivered constants
answers this program and not the next one: constant-plus-one, a product of two
constants, `imin(x, 4)`, a length read from a file. The owner's charter names
exactly this as the case-by-case exception the redesign must not open. The
question the design has to answer once is **where proving power lives and what
its general shape is** — which is the same question the retired-S8 sentence (3009)
and the "checker gap" field (2713) are each a partial answer to.

### H3 — the callee-written `&uniq` carve-out

**The text** (3241–3243, quoted in full in §1.10): a call's result seed is the
only call event; a call's write through an `&uniq` actual "does not by itself
change claim authority for that caller storage"; extending it is amendment-level.

**The asymmetry that makes it a hole.** The *same* callee write **does** kill
entailment facts (ENT-5 3066 kill (b): "a callee writing only through one `&uniq`
actual kills exactly the facts whose support overlaps that actual's resolved
place") and **does** transfer PRV-1 provenance (ENT-6 3207: "A write component
unions every right-hand side written to a root overlapping that formal, together
with each callee write component whose [EFF-2] projection reaches it"). Only claim
authority ignores it.

**The exposure.** A callee may write a value into caller storage that its own
`match` on a call result chose. Under [CLM-1] as written, the caller's later claim
over that storage is `Local` — the storage began Local, no caller-visible call
*result* reached it, and the write is invisible to the authority state. So the
caller can claim a property of a value a boundary chose, which is exactly what the
locality gate exists to prevent. *[observation] Whether that is unsound depends
entirely on what the gate is defending, and §1.11's comparison is the reason:
against **external data reaching a domain obligation** it is not a hole at all
(PRV-1 does see the write, and PRV-2/3 still fire); against **an unverified
cross-function promise becoming a lemma** it is a hole with no fence. The spec
never says which of the two CLM-1 defends, and the two answers differ here.*

**Corpus status: zero.** No claim anywhere in the tree has this shape. The prompt
names 0102 skeptic programs `s36`/`s37` as the artifacts that raised it; **those
programs are not in this tree** — no file, doc, or record at `b1367c82` mentions
`s36` or `s37`, and 0102's own record mentions the carve-out only in "Not done"
(363–366). A designer must treat the exposure as reasoned, not measured, and
should reconstruct the two programs rather than cite them.

### H4 — the redundancy interplay

**The text.** CLM-2 2764: "deriving any positive sign rejects c as redundant".
CLM-2 2766: "Checker strengthening may and must turn a formerly unknown claim into
this source-upgrade error; the author removes or restructures the source and
recompiles". ENT-1 2853–2855 repeats it and adds: "claims deliberately sit at the
proof frontier … This is an explicit source-upgrade rule, never authority for
compiler or optimizer elision." CLM-2 2776 extends it per component: "A pre-proved
component rejects c for overlap even when no complete predicate image was
derivable."

**The artifacts.**

- `tests/conformance/cases/clm2-pos-redundant-claim-advisory.wf` — the file name
  still says *advisory*, the manifest says
  `"expect": {"kind": "reject", "rule": "CLM-2"}`. The doc line inside the case:
  "the second repeats the same fact after the first claim establishes it, so CLM-2
  rejects the source instead of issuing an advisory or silently retaining an
  assertion." *[observation] The name is the fossil of an earlier answer to this
  exact question.*
- `docs/roadmap.md` PROOF-8 records "five non-rejecting redundancy advisories" at
  one point and "five redundancy advisories" for DEFLATE at another — so
  *advisory* was once the shipped behaviour.
- 0102 flipped a compiler test from a CLM-1 rejection to a **CLM-2 redundancy**
  rejection because the narrowing made `ieq(four, 4_u64)` local (0102:220–226,
  340–345).
- Historic churn: SHA-256 lost four claims when S11 shipped; DEFLATE lost eleven
  to value branches; DEFLATE's claim count moved 16 → 12 (roadmap PROOF-8).

**The shape of the hole.** *[observation]* Three candidate semantics are all
consistent with the constitution — hard reject (today), a note, or
acceptance-as-documentation — and the constitution does not select among them,
because T3's derivation only needs the claim to be *true*, not to be *necessary*.
What selects is W3 ("a claim is admitted only as an independently true,
checker-unknown, **load-bearing** theorem") plus R4's shift-left ordering. The
observation that separates them is not a count: it is whether a claim the prover
has learned still carries information a *reviewer* needs — the five-field record
is retained review data (2718), and if a redundant claim's `because` record is
worth keeping, refusal destroys evidence; if it is not, refusal is free. Note also
that today's answer makes **every prover improvement a breaking change for
claim-bearing source**, which is the only place in the language where that is
true — ENT-1 2853 grants ordinary programs monotonicity explicitly and withdraws
it from claims in the same sentence.

### H5 — the rendered diagnostics

**Carrier tie-break.** Spec 2744 says the rejection uses "[DIAG-1]'s least
component, earliest boundary witness, and **first canonical support carrier** that
observes that witness"; DIAG-1 1857 says "the **first source-ordered** support
carrier". The implementation (`flow.rs:6816–6832`) keeps a support only when
`witness.source_cmp(current).is_lt()` — strictly earlier — so on a tie the winner
is the first entry of `claim_component_supports` (6757), which is
`GoalTable::support(goal)` order for an opaque component and literal
`[left, right]` term order for a relation, deduplicated by first occurrence. That
is neither obviously "canonical" nor "source-ordered", and 0102 flags it
(388–392): "no test pins which carrier wins a tie, so a change to support ordering
would silently move diagnostic text."

**The teaching channel.** There is no uniform one. What exists:

- [DIAG-1] 1871: "A mechanical fix or restructuring is included exactly where the
  owning rule requires one." So the channel is **per-rule, by construction**.
- [CLM-1] locality has two fixed restructurings (1859), selected by *boundary
  kind*: `publish the required cross-function relation as an exact verified
  ensures clause on the callee and remove this caller claim` for a user result;
  `use the system operation's specified fact or typed outcome, or branch on the
  returned value; do not claim an unstated system-result property` for a system
  result.
- [OP-4] 878 and [ENT-6] 3169 give the two-route menu (branch, or claim).
- [FN-8]'s undischarged-requirement fix, quoted in `docs/patterns.md` P16, is a
  third phrasing of the same menu.
- [PRV-2] 3383–3386 and [PRV-3] 3409 give the two provenance repairs and the flat
  refusal "A `claim` is not a repair for an unconditionally external constrained
  subject."
- CLM-2's rejections (redundant / refuted / overlapping / vacuous / non-residual)
  carry **no mechanical fix at all** — 1855–1856 lists their payload as "name,
  exact predicate, classification, and the deterministic concrete instance,
  component when applicable, and terminal-root witness when one exists."

*[observation] So a writer who is refused today gets: a precise fix for a
non-local claim, a two-route menu for an unproved obligation, a flat "not a
repair" for an external subject, and **nothing** for the redundancy family — which
is the family every prover improvement will grow. The runtime channel is
[DIAG-3] and carries only the claim's IDENT (1977).*

---

## 5. Prior art, from memory

Everything in this section is **[from-memory]**: written without network access,
from my own knowledge as of the training cutoff. Names, dates and system details
may be imprecise; treat every claim here as a lead to verify, never as a citation.
Where I am unsure I say so inline.

### 5.1 Information-flow type systems, and why most of them taint reachability

**[from-memory]** The canonical frame is Denning and Denning's lattice model
(1976–77) and its type-system descendants: Volpano–Irvine–Smith's soundness result
for a simple imperative language (mid-1990s), the JFlow / Jif line (Myers, late
1990s onward), FlowCaml (Pottier and Simonet), and the later dependency-core
calculi (Abadi, Banerjee, Heintze, Riecke's DCC). The shared apparatus is a
security lattice, a typing judgment `pc ⊢ e : τ` where `pc` is the *program-counter
label*, and the rule that assignments inside a conditional must target locations
at least as high as the label of the guard.

That `pc` rule **is** the taint-the-reachability decision, and it is not an
accident or a conservatism — it is forced by the property those systems prove.
Their property is **noninterference**: for two initial stores agreeing on low
inputs, the two executions agree on low outputs. Implicit flow is a genuine
counterexample generator: `if (secret) { x := 1 } else { x := 0 }` leaks a bit into
`x` with no explicit assignment from `secret`, and any system without a `pc` label
admits it. So the line is drawn at reachability because the *observable* they
protect is the final store, and the store is reachable-dependent.

Three refinements from that literature are directly relevant here:

- **The `pc` is scoped, and where it is discharged matters.** In Jif-style
  systems the `pc` is raised at a branch and *lowered at the join* — the taint
  applies to writes performed under the branch, not to the continuation's fresh
  computations. A variable assigned *after* the join from constants is low. That
  is structurally the v0.39 narrowing, arrived at from the other direction.
  **[from-memory, moderately confident]** the usual formulation is that the `pc`
  labels the *command* being typed, and sequencing after the conditional returns to
  the enclosing `pc`.
- **Termination- and progress-sensitivity is a separate axis.** Most practical
  systems are *termination-insensitive*: a loop whose trip count depends on a
  secret is not treated as leaking, because closing that channel costs far more
  than it buys. The literature is explicit that this is a deliberate weakening with
  a known leak rate (**[from-memory]** Askarov, Hunt, Sabelfeld and Sands have a
  result bounding the leakage of termination-insensitive noninterference). The
  lesson for Whitefoot is not the result but the *method*: the systems that
  survived named the channel they were declining to close and bounded what it
  costs.
- **Declassification is a first-class construct, not an escape hatch.** Sabelfeld
  and Sands's "Dimensions and Principles of Declassification" (mid-2000s) organizes
  it along *what / who / where / when* and states principles — semantic consistency,
  conservativity, non-occlusion, monotonicity of release — that any declassifier
  must satisfy. **[from-memory]** the key one for this terrain is *non-occlusion*:
  a declassification must not mask other leaks introduced elsewhere.

**What this literature would say about Whitefoot's H1.** *[observation on
from-memory material]* It would say the argument is being conducted with the wrong
property. Noninterference needs the `pc` because the protected observable is
data-dependent on reachability. Whitefoot's protected observable is **not** a store
— it is "did an unverified cross-function promise become a proof premise". A
selector's *choice* cannot convey a callee's unstated property into a claim's
*truth* unless a value chosen by that selector is in the predicate's support;
whereas in noninterference the choice conveys a bit into the store by construction.
So the two systems' answers differ **because their observables differ**, and the
right question to derive from is: *what does this system's claim actually
protect?* — precisely the question §1.11 says the spec never answers.

The information-flow answer to the *diagnostic* side is also relevant: Jif's
practical complaint, well documented, is label creep — labels grow monotonically
through a program until everything is high, and the error messages point at the
join rather than at the origin. That is the 63/63 failure mode.

### 5.2 Proof-carrying and refinement systems: where prover strength stops and user lemmas start

**[from-memory]** Four families, each with an explicit and different answer to the
boundary Whitefoot calls "the checker gap".

**(a) Verification-condition generation with an SMT backend — Dafny, Boogie, VCC,
Why3, Spec#, Frama-C/WP.** The architecture is: the tool generates verification
conditions from source plus annotations, discharges them with an SMT solver
(Z3 for Dafny/Boogie), and reports which VC failed. The user's tools for closing a
gap are graded and explicitly named: `assert` (a checked lemma — *proved*, then
available as a fact downstream), `assume` (an *unchecked* fact — Dafny historically
restricted or flagged it), `invariant` on loops, `decreases` for termination,
ghost variables and ghost functions, `calc` blocks for equational reasoning, and
lemma methods. **The crucial structural point for Whitefoot: in Dafny an `assert`
is proved by the solver, and it is *not* a runtime check; the user's contribution
is the *proof hint*, i.e. the intermediate step that makes the solver's search
succeed, not the truth of the statement.** Whitefoot's `claim` occupies a
genuinely different slot: the truth is contributed by human review and the
statement is retained as a runtime check. **[from-memory, confident]** Dafny's own
guidance is that a failing `assert` means either the property is false or the
solver needs more hints, and that the fix is more intermediate asserts, not
`assume`.
- **Loop invariants are the dominant annotation burden in every one of these
  systems.** Every Dafny tutorial's second chapter is loop invariants, and the
  empirical literature on Verified Software Competition entries reports invariants
  as the bulk of user-written annotation. Whitefoot's corpus (39/135 claims,
  §2.1) is reproducing that distribution exactly, with claims standing in for
  invariants.

**(b) Refinement types — Liquid Haskell, LiquidTypes (Rondon, Kawaguchi, Jhala),
F*, Stainless/Leon, Flux for Rust.** Here the boundary is drawn *by the
qualifier language*: types are refined by predicates drawn from a fixed
decidable fragment (typically quantifier-free linear integer arithmetic plus
uninterpreted functions), and **inference** finds the strongest refinement
expressible in that fragment by fixpoint over Horn constraints. **[from-memory]**
The Liquid line's central design claim is exactly that: give up expressiveness for
*predictable inference*, so the user writes types, not proofs. When inference
fails, the user's move is to *strengthen a signature* (a refinement on a function's
result), not to assert a fact at a program point.
- **F\*** sits at the other end: it has a full dependent type theory, SMT
  discharge for the decidable parts, and explicit `Lemma` declarations plus
  tactics (Meta-F*) for the rest. Its `assume` and `admit` are marked and tracked
  as part of the trusted base.
- The relevant lesson: **all of these systems locate proving power in a *fixed
  fragment* and locate user contribution in *boundary annotations* (signatures,
  invariants, lemmas), not at arbitrary program points.** Whitefoot already has the
  fixed fragment (ENT-2's DBM) and already has a boundary annotation (`requires` /
  `ensures` with FN-9 verification and S12 publication). What it *also* has, and
  these systems mostly do not, is a point-local user fact that survives to runtime.

**(c) Proof-carrying code — Necula and Lee, mid-1990s, and Foundational PCC
(Appel).** The relevant idea is the *certificate*: the producer ships a machine-
checkable proof, the consumer runs a small trusted checker. The boundary between
prover and user is not the issue; the issue is that **the trusted base is a
checker, not a prover**, and that proof *search* lives outside the trusted base
entirely. **[from-memory]** Whitefoot's ENT-1 makes the same move for a different
reason ("a solver result never participates … no implementation may strengthen,
weaken, time-bound, or randomize the derivable set") — but where PCC pushes search
outside and *accepts its certificate*, Whitefoot pushes search outside and accepts
*nothing back except a human review record*. That is the single largest structural
difference between this design and every system in this section, and it is exactly
where the `because` field sits.

**(d) SPARK/Ada and MISRA-style industrial verification.** **[from-memory]**
SPARK's model is closest to Whitefoot's in one respect: proof obligations are
generated per partial operation (overflow, range, division, index), and the
engineer's escape hatches are `pragma Assert` (proved or a runtime check depending
on mode) and `pragma Assume` (trusted, and required to be justified in a review
record). SPARK explicitly supports **justification annotations** —
`pragma Annotate (GNATprove, False_Positive, "...", "reason")` — which is a
five-field `because` record by another name, reviewed by a human, tracked by the
toolchain, and counted in the qualification evidence. It also has the notion of
mixed proof/test assurance levels, where an unproved VC may be discharged by a
retained runtime check plus a test argument. **This is the closest existing analogue
to a Whitefoot `claim` I can name, and it, too, keeps the runtime check.**

### 5.3 Abstract interpretation: joins, widening, and the price of path-sensitivity

**[from-memory]** Cousot and Cousot's framework (1977): an abstract domain with a
partial order, a join `⊔` at merges, and a **widening operator `∇`** to force
termination of fixpoint iteration over loops, optionally followed by narrowing to
recover precision.

Relevant domains, in increasing cost:

- **Intervals** (non-relational). Join is per-variable interval hull. `x=0 ⊔ x=1`
  gives `x ∈ [0,1]` — note that an interval domain **would** solve Whitefoot's H2
  join case, precisely because it tracks a *value set* per variable rather than
  pairwise differences.
- **Difference-bound matrices / zones** (Miné). Relational, `x - y ≤ c`, cubic
  closure. **This is Whitefoot's ENT-2/ENT-4 exactly.** DBMs cannot express
  `x ∈ {0,1}` as a fact stronger than `0 ≤ x ≤ 1`; they *can* express `0 ≤ x` and
  `x ≤ 1` — so a DBM join of `x=0` and `x=1` should yield `0 ≤ x ≤ 1`, not
  nothing. **[observation, and important]** Whitefoot's join loses it for a
  different reason than the domain: `join_at_once` requires a pair to be *present
  in every input* (state.rs:4123–4130), and `x = 0` is stored as the pair
  `(x, Z) ≤ 0` and `(Z, x) ≤ 0` while `x = 1` is `(x,Z) ≤ 1` and `(Z,x) ≤ -1`;
  both pairs *are* present in both inputs, so the join should give
  `(x,Z) ≤ 1, (Z,x) ≤ 0`, i.e. `0 ≤ x ≤ 1`. The loss in the corpus cases is
  therefore **not** the join's fault — it is that `give 0_u64;` establishes no
  fact at all on the delivery edge (§3.4), and that a loop head *subtracts* the
  pre-loop `cursor = 0` fact rather than joining it (§3.3). A designer must not
  mis-attribute H2 to the join.
- **Octagons** (Miné, 2001): `±x ± y ≤ c`, still polynomial.
- **Polyhedra** (Cousot–Halbwachs, 1978): general linear inequalities,
  exponential in the worst case, and the classic place where widening is
  unavoidable.

**Widening.** The standard widening on intervals keeps a bound that is stable and
jumps unstable bounds to ±∞; on polyhedra it keeps the constraints of the first
iterate that the second still satisfies. Widening is what buys termination when a
loop's abstract state is *iterated to a fixpoint*. **Whitefoot does not iterate**
(§3.3): it subtracts everything a continuing kill may touch and re-establishes
only S11's structural bounds. That is a legitimate design point — it is the
cheapest possible sound loop rule and it is deterministic, search-free and
trivially confluent, which ENT-1 requires — but it means the project has *never
paid for*, and therefore never had to specify, a widening operator. Adding loop
induction later (3116 promises it) is where that bill arrives.

**Path-sensitivity, and what it costs.** The standard techniques and their prices:
- **Trace partitioning** (Rival and Mauborgne, ~2005, in ASTRÉE): keep a *set* of
  abstract states indexed by a partition token (which branch was taken, which
  iteration range), merge lazily. Cost is a multiplicative blow-up bounded by the
  partition, plus a heuristic policy about when to merge — and the heuristic is
  where determinism goes.
- **Disjunctive completion / powerset domains**: exact, and non-terminating
  without a bound on the number of disjuncts.
- **Path-sensitive dataflow with predicate abstraction** (SLAM/BLAST-style
  CEGAR): a refinement loop driven by counterexamples, i.e. *search*.
- **Guarded/conditional facts** ("`p ⇒ x ≤ 4`"): keeps one state but attaches
  guards to facts; cheap in space, but the guard language is a second logic that
  must itself be closed under join, and query becomes implication checking.

**[observation]** Each of the four costs something ENT-1 forbids outright:
trace partitioning needs a merge *policy*, powerset needs a *bound*, CEGAR needs
*search*, and guarded facts need a *second logic*. ENT-1 2831 ("closed,
deterministic, search-free … fixed completely by this specification") and 2836
(two conforming implementations derive identical states) are the real constraints
on any redesign of the join, and they rule out three of the four before precision
is even discussed.

### 5.4 What these traditions would say about H2 and H3

**On H2 (the delivered-value squeeze) —**

- *Information flow* **[from-memory]**: would say the `value_if` case is a
  textbook implicit flow and the `pc` must taint `picked` — *if* the property is
  noninterference. It would then point out that the arms deliver constants
  independent of the guard, which is the standard **"the branches agree"**
  observation, and that several systems handle it: `if (h) x:=1 else x:=1` is
  accepted by any flow-sensitive analysis that compares the two post-states, and
  is the motivating example for *relational* rather than *label-based* reasoning
  (self-composition, product programs, RHLE/Relational Hoare Logic). The
  information-flow answer to H2 is therefore: **look at what the two arms
  actually establish about the delivered value, not at who chose the arm.** That
  generalizes — it is not a constants exception — and it is exactly what a DBM
  join over per-edge delivery images would compute if the images were nonempty.
- *Refinement types* **[from-memory]**: would say the whole question is misposed
  at the program point. `picked : {v: u64 | v < 4}` is the refinement, it is
  *inferred* by the fixpoint over the two arms (each arm's constant refines to
  `v = 0` and `v = 1`, whose join in the qualifier lattice is `v ≤ 1`), and no
  user annotation is needed at all. Liquid-style inference would close 46 of the
  corpus's 135 claims (the `%` family) for free, because `x % C` has a standard
  measure-based refinement.
- *Abstract interpretation* **[from-memory]**: would say this is a
  **transfer-function gap, not a domain gap** — see the correction in §5.3. The
  domain can hold `0 ≤ picked ≤ 1`; nothing establishes it.
- *Dafny/SPARK* **[from-memory]**: would prove it without any user input, because
  the VC is `0 < 4 ∧ 1 < 4` after the branch is expanded into the verification
  condition; VC generation is path-sensitive *for free* because the VC is a
  formula over the whole path structure, not a merged abstract state. That is the
  single sharpest contrast available: **the systems that need no annotation here
  are the ones that never merge.**

**On H3 (callee-written `&uniq` storage) —**

- *Information flow* **[from-memory]**: uninteresting as posed. Jif-style systems
  label *locations*, and a callee writing through a reference writes the location;
  the callee's signature carries the labels (`begin_label`/`end_label` in Jif) and
  the write's label is the join of the callee's `pc` and the value's. There is no
  carve-out to make: a write is a write, and a function's effect on labels is part
  of its type. The literature's whole answer is **put it in the signature**.
- *Refinement types / Dafny / SPARK* **[from-memory]**: identical answer, and it
  is the loudest agreement across the whole section. A callee that writes through a
  mutable parameter must have a **postcondition about that parameter**
  (`ensures`/`modifies` in Dafny, `Post` and `Global`/`Depends` in SPARK, a
  refinement on the output store in F*). The caller learns *exactly* what the
  signature says and nothing else. Under that discipline, the caller's knowledge
  about callee-written storage is neither "local" nor "boundary" — it is
  *whatever the verified contract published*, which in Whitefoot is precisely the
  FN-9/S12 route that already exists and that CLM-1 2745 already says cannot be
  restated by a claim.
- *Abstract interpretation* **[from-memory]**: would say the carve-out is unsound
  as an abstraction only if the analysis claims soundness about *values*; since
  claim authority is not a value analysis, the question is whether the abstraction
  is *conservative in the right direction*. The framework's standard answer to an
  unmodelled effect is `⊤` — i.e. the conservative choice is to mark
  callee-written storage `BoundaryResult`, and the burden is on the design to
  justify the *non*-conservative one.

*[observation]* The one thing all four traditions agree on, across H2 and H3
alike, is: **the callable boundary is where cross-function knowledge is declared,
and inside a function the question is what the code establishes, not who chose
the path.** Whitefoot already has both halves of that — FN-9/S12 for the boundary,
ENT-3/ENT-4 for the inside — which suggests the redesign's leverage is in the
relationship between them rather than in a new mechanism.

---

## 6. Open facts a designer must not get wrong

Ordered by how expensive the mistake would be.

1. **The claim mechanism has exactly six customers, and all six are
   partial-operation domain obligations.** CLM-2 2789: the terminal admission
   roots are the four ENT-6 families, ordinary FN-8 call requirements, and
   mandatory FN-9 selected-return aggregate proofs. Nothing else is a consumer —
   not an optimizer fact, not another claim, not a test. Any design that gives a
   claim a new job is changing that closed list, and every corpus claim's
   `consumers:` field names one of the six (§2.1).

2. **`Local` is not `true`, and it is not `internal`.** Three orthogonal axes,
   stated twice in the spec (2746, 2757) and again at PRV-1 3310. A claim can be
   refused by CLM-1 for authority, by CLM-2 for lifecycle or residuality, and by
   PRV-2/PRV-3 for external subjects, on the same occurrence, for three unrelated
   reasons. Collapsing any two is the fastest way to break the area.

3. **H2's second half is NOT a join defect.** §5.3 and §3.4: the DBM join of
   `x = 0` and `x = 1` yields `0 ≤ x ≤ 1` correctly, because both directed pairs
   are present in both inputs. The corpus's delivered-constant case loses the
   fact because `give 0_u64;` establishes **no delivery image at all** (GIVE-1 292
   excludes literal carriers; `eligible_delivery_terms` matches only
   `CheckedExpression::Binding`), and the loop-carried case loses it because the
   loop *head subtracts* the pre-loop fact rather than joining it (ENT-5 3110/3120,
   `apply_loop_kills`). Attributing H2 to the merge would fix the wrong thing.

4. **Nothing in the fact state iterates.** There is no fixed point over
   `FactState` and therefore no widening operator anywhere in the spec or the
   compiler. The loop rule is one subtraction. ENT-1 2831/2836 (closed,
   deterministic, search-free; two implementations derive identical states) is what
   forces that, and it rules out trace partitioning (needs a policy), powerset
   domains (needs a bound), and CEGAR (needs search) before precision is discussed.
   The one fixed point that *does* exist in the area is PRV-1/ENT-6's
   **two-stratum dependency fixpoint** (3210–3212, 3289–3291), which is a monotone
   set-union lattice, not a fact state.

5. **The claim-authority analysis is computed once, before everything.** ENT-6
   3247: "computed once before S3, U, B, `Eligible`, or any `Full-minus` mask and
   is reused unchanged by every claim component query", and the compiler honours it
   (`claim_locality.rs:758`, and `claim_locality_failure` returns `None` whenever a
   claim mask is active, flow.rs:6808). A design in which locality depends on facts
   would make CLM-2's counterfactual runs circular.

6. **Locality reads exactly the same support set the kills read.** ENT-5 3048 +
   ENT-6 3248, and "canonical normalization may add a fact identity but never
   subtract authority support read by the retained S expression". `S`, not `D` and
   not `F`, is the image whose support is queried. Changing which image carries
   support changes both what dies and what is local.

7. **`Full-minus` must produce exactly the same PRV failure set as `Full`.**
   ENT-5 3050–3054: the mask suppresses only S3 source events and changes no
   evaluation, effect, ownership, cleanup, scope, join, loop or runtime statement;
   "any new or removed PRV-2/PRV-3 event is a compiler consistency failure, never a
   residual witness". Residuality is not free to disturb provenance.

8. **CLM-2's redundancy flip is mandatory, and it is the *only* place the
   language withdraws version monotonicity.** ENT-1 2853–2855 grants monotonicity
   to every discharged operation, call goal and selected-return relation in the
   same breath as it withdraws it from claims. Whatever H4's answer is, it must be
   stated as a deliberate exception to ENT-1's own monotonicity clause, not
   alongside it. The redundancy family is also the one rejection family with **no
   mechanical fix** in DIAG-1's payload (1855–1856).

9. **Moving the `BoundaryResult` seed set is amendment-level by the spec's own
   words.** ENT-1 2856 and CLM-1 3243 both say so. H3's resolution is therefore a
   spec amendment in any direction, including "leave it as it is and say why".

10. **`deny_claims` counts claims; nothing else in the language does.** CLM-3
    2815: strict success requires `MayClaims` to be **empty**. If a redesign makes
    claims cheaper or more numerous, CLM-3's meaning shifts under it; if a
    redesign introduces a second claim-like construct, CLM-3's `DirectClaims`
    identity `(instance, NodePath, name)` (2809) must be re-derived.

11. **A claim's name is its entire runtime identity.** DIAG-3 1977: the record
    carries `rule_id`, the claim IDENT as `message`, the function IDENT, and the
    NodePath. The five `because` fields never appear at runtime. Claim names are
    deliberately outside every declaration domain (2752–2754) so that `len` and
    `wrap` are legal names — an owner ruling of 2026-08-07, not an accident.

12. **[PAR-3] does not consume claims as proof.** It treats a `Claim` statement as
    an ordinary straight-line statement with the condition as its read footprint
    and explicitly does **not** treat the trap edge as a control edge
    (`staged_permission.rs:818–826`). Its only claim content is the
    erroneous-execution clause (2074–2079), which is T3 restated. A design that
    made claims carry proof into the permission judgment would re-open T3's
    derivation.

13. **T3 stands on the claim's definition, and says so.** `docs/constitution.md`
    T3: "NOT an axiom — derived from W3's claim discipline, which is the
    load-bearing premise: a claim is admitted only as a reviewed, independently
    true, always-true lemma … **The theorem stands while that premise stands: a
    future construct admitting claim-like predicates that are not reviewed
    always-true lemmas — assertions, expected failures, unreviewed conditions — is
    outside this theorem until the derivation is redone against it.**" Any design
    that softens "always true on every reaching execution" owes T3 a new
    derivation, and T3's history section warns that the claim-free eligibility
    gate has already been re-proposed once by fresh reviewers.

14. **The evidence rule for a new fact source is already written into the spec.**
    ENT-3 3009, on the retired S8 midpoint family: it "may return as a later
    version's monotone addition **the day a corpus program writes the shape**".
    That is the same standard the owner's charter states, in the spec's own voice.

15. **Two shapes are 63% of all written claims, and neither is about locality.**
    46 remainder-result-range and 39 loop-induction claims out of 135 (§2.2). The
    entire H1 controversy — 63 fuzzer rejections and seven conformance cases —
    concerns a class with **two** real-program members and **zero** blind-writer
    members. A design that perfects the control-dependence line and leaves the
    other two shapes alone will not move the corpus.

16. **The nearest neighbour to H3 that *does* exist and *does* work:** claims over
    `len(deref(buffer))` where `buffer` is a `&uniq` parameter are Local (the
    parameter component seeds Local, ENT-6 3222) and appear in the corpus with the
    gap "ENT does not publish the borrowed buffer length through the child call".
    So the terrain already contains a working case in which a caller claims a
    property of storage reachable through a unique borrow — it differs from H3 only
    in who wrote the value. That pair is the observation that separates "authority
    follows the storage root" from "authority follows the last writer".

17. **`s36`/`s37` are not in this tree.** The 0102 skeptic programs the charter
    cites for H3 do not exist at `b1367c82` (the only match in the repository is an
    unrelated identifier in `archive/toolchains/…/llvm_scalar.wf`, and `archive/`
    is off-limits to active work). H3 must be re-grounded on a reconstructed
    program, and 0102's record supports only the "Not done" statement, not a
    measured exposure.

18. **The compiler's join does one thing the spec never mentions.** The
    postcondition-candidate second pass in `join_at` (state.rs:4045–4068). It
    cannot change the derivable set, but it exists so a postcondition-free
    derivation is retained where one exists. Any rewrite of the join must carry it
    or explain its removal (§3.2).

19. **Every prior-art tradition surveyed agrees on one thing** (§5.4): the callable
    boundary is where cross-function knowledge is declared, and inside a function
    the question is what the code *establishes*, not who chose the path. Whitefoot
    has both halves already — FN-9/S12 and ENT-3/ENT-4. Nothing in §5 suggests a
    missing mechanism; the suggestions all concern the relationship between the two.

20. **Migration cost is not a design criterion here, but the migration *surface* is
    knowable:** 135 claim occurrences in 101 files, 25 `*clm*`,
    25 `ent*` and 18 `prv*` conformance cases (of 517), ten rewritten tests from 0102, the six digest anchors
    and two transcribed literals any spec activation touches (0102:66–90), and
    `compiler/src/backend/qualification.rs`'s `REVIEWED_FOR`.
