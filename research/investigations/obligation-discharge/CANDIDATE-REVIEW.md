# Adversarial review: obligation-discharge batch-1 candidate

Status: review of record (2026-08-07). Target:
`governance/spec-evolution/obligation-discharge-batch1-candidate.md` at
commit bf75fba, checked against `spec/kernel-spec-v0.20.md`,
`DOSSIER.md` (§2, §8), `SIMULATION.md`, `PROBE-TAINT.md`, and
`SYS-POSTCONDITIONS.md`. The working copy has since gained the 2026-08-07
owner-rulings pass (O1–O16 applied); each finding below states whether that
revision already resolves it. Reviewer did not author the candidate. This
file records findings only; it changes no candidate text.

Severity legend: **S** soundness-breaking (an accepted program can go out of
bounds), **A** acceptance-changing (two conforming implementations diverge,
a version-law is violated, or a program class is rejected/accepted contrary
to the design), **E** editorial.

Summary: 2 S (one already fixed in the working copy), 6 A, ~10 E. Worst
open finding: F2.

---

## F1 (S) — Missing offset typing lets a provably negative index discharge

**Candidate text (bf75fba §5 [ENT-6]):** "This version attaches exactly one
obligation family: for every source `index<T>(P, i)` place — read, write,
and [SET-1] target position alike — the bounds obligation `i < len(P)`,
normalized `i - len(P) <= -1`", combined with [ENT-2]: "relations are over
mathematical values, so relations between terms of different fragment types
are well-formed", and the [OP-4] replacement: "A discharged index reads or
writes with no runtime bounds check in every build mode."

**Attack.** No v0.20 rule fixes the index offset's type (the candidate's own
O9 concedes this: "I found no v0.20 rule fixing the offset's exact required
type"). The obligation has no lower bound, and only a u64-typed offset gets
one implicitly (`Z - t <= -min(T)` is `i >= 0` only when `min(T) = 0`).
Counterexample with a signed offset:

```
let i: own i64 = -1_i64;            // S5: i = -1  (i - Z <= -1)
let x: own u8 = index<u8>(buf, i);  // obligation i - len(buf) <= -1
```

Closure: `i - Z <= -1` plus the implicit `Z - len(buf) <= 0` gives
`i - len(buf) <= -1`. Discharged; the rewritten OP-4 emits **no runtime
check**; execution reads `buf[-1]`. Under v0.20 the retained bounds check
caught this at runtime; the candidate deletes the check on discharge, so the
unstated typing gap becomes memory unsafety. Drafting an open question (O9)
does not make the normative bytes safe: as written at bf75fba they license
the access.

**Fix.** Fix the offset atom's exact type to `own u64` in the OP-4
replacement and cite it from ENT-6 (equivalently: add the second obligation
`0 <= i`).

**Status: RESOLVED in the 2026-08-07 working copy** (ruling O9: "The offset
atom has exact value mode and type `own u64`; ... any other offset mode or
type is a hard error citing OP-4", with ENT-6 restated over u64 and the new
shrink class (5) enumerated). The fix is sound and complete for this attack.

---

## F2 (S) — Spelling-identified terms plus unordered scope-exit kills let a
stale fact attach to a fresh binding

**Candidate text (§5 [ENT-2]):** "Two places are the same term exactly when
their canonical source spellings [FORM-2] are byte-identical". **[ENT-5]
kill (d):** "the end of the region of any borrow holder in its support and
the end of the lexical scope of any support binding, including region exit".
**[ENT-5] joins:** "at the continuation of a `match_stmt` or `value_match`,
the fact state is the join of the closed states at every arm exit edge
reaching that continuation".

**Attack.** TYPE-6 permits spelling reuse across disjoint expired scopes
("Disjoint expired lexical scopes may reuse an ordinary value or label
spelling"). The arm-exit edge and the arm-scope end are the same program
point, and the text never orders kill (d) against the join's sampling of
"the closed states at every arm exit edge". Under the admissible reading in
which the exit-edge state is the state after the arm's last statement
(kills applied "at the end of the scope" being an event the edge has not
yet crossed), arm-local facts survive into the join keyed by their
spelling:

```
match c {
  True()  => { let x: own u64 = 0_u64; ... }   // S5: x - Z <= 0
  False() => { let x: own u64 = 0_u64; ... }   // S5: x - Z <= 0
}
// join keeps x - Z <= 0 (same spelling, both arms)
let x: own u64 = n;                  // legal reuse; NEW binding, SAME term
let e: own u8 = index<u8>(buf, x);   // stale x <= 0 + len(buf) >= 1 => discharged
```

At runtime `x = n` is arbitrary: out-of-bounds with no check. Worse, if the
new binding's S5 fact contradicts the stale bound, the state becomes
contradictory and [ENT-4] discharges *everything* at a perfectly reachable
point. Under the other reading (kills applied on the exit edge) the program
is rejected — so at minimum the two readings split acceptance, violating
ENT-1's own two-implementations law; under the permissive reading the
fragment is unsound. Note region exit is handled explicitly ("including
region exit") but arm and block exit are not tied to edges.

**Fix.** Two independent repairs; do at least the first:
(1) state that kill events (c) and (d) apply **on every edge leaving the
scope, before any join is taken**, mirroring STOR-3's edge-carried release
phrasing; (2) close the root cause by making term identity
declaration-anchored: "two places are the same term exactly when their
roots resolve to the same declaration event and their spellings are
byte-identical" — spelling reuse then never collides.

**Status: OPEN in the working copy** (ENT-2/ENT-5 unchanged by the rulings
pass). This is the worst open finding.

---

## F3 (A) — ENT-3's dominance clause contradicts ENT-5's join

**Candidate text (§5 [ENT-3]):** "The fact state at a program point contains
exactly the facts below whose establishing event dominates the point on the
conservative structural normal-control graph [FN-1], subject to the kills,
joins, and loop rule of [ENT-5]". **[ENT-5]:** "the fact state is the join
of the closed states at every arm exit edge".

**Attack.** A fact established separately in *every* arm (e.g. each arm
executes `check ilt<u64>(i, n) else trap "…";`, or one arm has the S1
branch fact and the other re-checks) has **no single dominating establishing
event** at the continuation, yet the join keeps it ("the weakest
(largest-constant) bound held by all joined states"). The two sentences
select different states: a dominance-reading implementation drops the fact
and rejects a following `index<u8>(buf, i)`; a join-reading implementation
keeps it and accepts. That is exactly the two-implementation acceptance
divergence ENT-1 forbids ("two conforming implementations derive the same
closed fact state at every program point"). SIMULATION.md flagged
"path-sensitive match-join facts" as a borderline judgment "which must be
specced" — the candidate specced both answers at once.

**Fix.** Delete the dominance phrasing and define the state constructively:
established facts flow forward along the conservative structural graph,
kills apply per [ENT-5] on the edges where their events occur, merge points
take the [ENT-5] join, and loop heads take the loop rule. (Dominance is
then a theorem for straight-line establishment, not a definition.)

**Status: OPEN in the working copy.**

---

## F4 (A) — CLM-2 refutation contradicts the stated version-monotonicity law

**Candidate text (§5 [ENT-1]):** "Version monotonicity is law: a later
specification version may add fact sources and closure rules and may remove
none, so checker strengthening converts claims into [CLM-2] advisories and
undischarged obligations into discharged ones, **and never the reverse**."
Header: "a version-monotone lifecycle". **[CLM-2]:** "When the fact state is
non-contradictory [ENT-4] and derives the predicate's exact negation, the
program is rejected with a hard error citing CLM-2".

**Attack.** Adding fact sources derives more *negations* too. A v0.21
program containing `claim c: ilt<u64>(x, y) because "…";` at a point where
the v0.21 state derives neither the predicate nor its negation is accepted.
A v0.22 fragment with one more fact source (any of the deferred items:
ensures, cross-call congruence, loop induction) can derive `y <= x` there —
CLM-2 then rejects the previously accepted program. So a v0.21-legal
program becomes illegal under a strictly stronger later fragment, through a
path the candidate's own rules define. The absolute law in ENT-1 and the
header's "version-monotone lifecycle" are false as stated. (CLM-2's
redundancy sentence is carefully scoped — "rejects no previously accepted
program **on that ground**" — but ENT-1's "never the reverse" is not.)
DOSSIER §2.7 chose this deliberately ("Checker upgrade refutes a claim →
hard error") and scoped its monotonicity claim to the redundancy direction
("acceptance monotonicity depends on redundant-claim being a warning"); the
candidate overstates the dossier.

**Fix.** Scope the law and name the exception: "checker strengthening never
converts a discharged obligation to undischarged or a conforming claim to a
rejection, with one enumerated exception: a strengthened fragment may newly
refute a claim under [CLM-2], rejecting a program thereby proven to trap on
every execution reaching that claim. Refutation is the deliberate
non-monotone edge of the lifecycle." Alternatively make cross-version
refutation an advisory — an owner ruling either way; the current text
asserts a false theorem about its own system.

**Status: OPEN in the working copy.**

---

## F5 (A) — Term roots exclude propagate-bound lets, value-match lets, and
borrow-mode match binders; the "fallback always closes discharge" claim is
false there

**Candidate text (§5 [ENT-2]):** a tracked place's "root `pbase` IDENT
resolves to a `param`, **ordinary `let`**, requires-clause local, or
**match-binder value binding** or to a named const". **[ENT-6]:** "that
fallback always closes discharge". **S6:** "`let m: own u64 = len<T>(P);`
for a **tracked** P establishes m = len(P)".

**Attack.** "Ordinary let" is the spec's fixed term for a `let_stmt`
selecting `ordinary_let_rhs` (GRAM-4, FN-8, and the candidate's own S5 use
it that way). A binding introduced by `propagate` or by a value-match `let`
is therefore not a term root; nor is a match binder of derived borrow mode
(only "value binding" binders qualify). Any place rooted there is
untracked, so `len(P)` is not a length term and the obligation is
permanently underivable — and the advertised fallback cannot help, because
a claim over untracked operands establishes nothing (comparison-origin
operands must be terms) and S6 requires a tracked P:

```
let buf: own buffer<u8> = propagate make_buffer(cap: n);
let x: own u8 = index<u8>(buf, 0_u64);   // rejected; no claim or branch can fix it
```

The only repair is `let buf2: own buffer<u8> = move buf;` and re-rooting
every use — a whole-shape rewrite, not the dossier's "paste the printed
residual as a claim ... always closes discharge in one step" (§2.2). Every
buffer/array/slice obtained through `propagate` — the standard fallible
constructor shape — falls in this class; SIMULATION's hand entailment did
not model this restriction, so the §10 buckets are also at risk. This is a
silent narrowing of the dossier's design, not among §8's enumerated shrink
classes.

**Fix.** Widen term roots to every `let_stmt` binding (all three right-hand
forms), every param, requires-clause local, every match binder regardless
of derived mode, and named consts. Nothing in the fragment's soundness
argument depends on the excluded classes: identity is spelling(+declaration
per F2), and mutation/consumption/aliasing are already handled by the kill
rules. If any exclusion is deliberate, enumerate it as a shrink class and
correct ENT-6's "always closes" sentence.

**Status: OPEN in the working copy.**

---

## F6 (A) — Missed knock-on modifications: FN-1's structural graph and
DIAG-1's closed carrier taxonomy

**Candidate text (§1):** "thirteen existing rules modified: FORM-2 …
SYS-9". **§12:** "No genuine contradiction … every collision … is a
deliberate, enumerated modification above rather than an ambiguity."

**Attack.** Two unlisted v0.20 rules collide with the new statement:

1. **[FN-1]:** "An ordinary `let`, `set`, expression statement, and a
   passed `check` have a normal edge to `normal_successor(s)`." The
   enumeration is closed; `claim_stmt` gets no edge. Strictly read, every
   statement after a claim fails to be "reachable from function-body
   entry" (an FN-1 rejection), and every candidate reference to a claim's
   "normal continuation" (CLM-1, ENT-3 S3, GIVE-1 delta, EFF-2) points at
   an edge the graph does not define. One sentence of FN-1 must change
   ("a passed `check` or `claim`"), and FN-1 belongs in the modified list.
2. **[DIAG-1]:** the carrier taxonomy is closed ("The deferred-use carriers
   are exactly …"; "The table-checked carriers are exactly the
   `program_kind` IDENT and both IDENTs of an `input_label`"). The
   claim-name IDENT — a carrier that is deliberately no declaration, no
   lexical use, no deferred use, and no table-checked record — fits no
   class. Either DIAG-1's lists gain the claim-name carrier or CLM-1 must
   state its classification explicitly.

Both falsify the §12 completeness claim and the header's "thirteen"
(→ fifteen, or fourteen if the DIAG-1 fact is folded into CLM-1).

**Fix.** As above; recount the header.

**Status: OPEN in the working copy** (modified-rules list unchanged).

---

## F7 (A) — The two closure definitions in ENT-4 disagree on reflexive bounds

**Candidate text (§5 [ENT-4]):** "the least set containing its established
and implicit facts and closed under exactly: (1) … (2) … (3) …" and, in the
same rule, "it equals the all-pairs shortest-path bounds of the difference
graph with disequality strengthening iterated to its unique fixed point."

**Attack.** All-pairs shortest path assigns every node self-distance 0 (the
empty path), i.e. `t - t <= 0` for every term. The inductive closure never
produces `t - t <= 0`: rule (1) only composes two existing bounds, and no
implicit fact supplies reflexivity (the implicit type-range pair composes
to `t - t <= max(T) - min(T)`, not 0). The two definitions the rule declares
equal are not equal. Where it bites: any derivation needing `a <= a` with
`a` one term — S8's midpoint instantiated with `lo` and `hi` the same term
(the state must "derive lo <= hi"), and derived equality `a = a`. A
shortest-path implementation establishes S8's facts there; a
least-closure implementation does not; downstream discharge then splits
acceptance between two conforming implementations.

**Fix.** Add reflexivity to the implicit facts ("every term t carries
`t - t <= 0`") or strike the shortest-path sentence; state one definition.

**Status: OPEN in the working copy.**

---

## F8 (A) — §9/§10's sha256 bucket is unrepresentable at this candidate's L0

**Candidate text (§9):** "add 3 claims, one being the loop-head claim
`16 <= extend_index < 64` covering all five schedule accesses". **§10:**
implementation must reproduce "`sha256_abc` (0 proven, 3 claims covering 8
sites)". **O11 (ruled):** comparison origin admits "no `band`/`bor`/`bnot`
composition".

**Attack.** A claim's predicate contributes a fact only through comparison
origin — one single comparison call (or a Bool let bound to one). The
conjunction `16 <= extend_index < 64` cannot be one claim: written as
`band(...)` it establishes **nothing** (S3 via origin (a)/(b) only), so the
loop-head fact requires two claims. A conforming L0 checker therefore needs
4 claims for sha256's 8 sites, and §10's acceptance criterion — match the
simulation's "3 claims" — is unsatisfiable by a correct implementation of
this very text. (SIMULATION.md predates the single-comparison cut; the
other buckets survive: utf8parse's pair is written as two claims.)

**Fix.** Restate the sha256 bucket as 4 claims (two loop-head comparisons
plus two others) in §9 and §10, or explicitly note in §10 that the
simulation's conjunction counts one claim where the fragment requires two
and adjust the matching rule.

**Status: OPEN in the working copy.**

---

## F9 (E) — Dangling SYS-8 cross-reference to OP-4's deleted bounds semantics

Unmodified [SYS-8] (range-validation paragraph): "traps under the bounds
semantics of [OP-4], before any host transfer". The rewritten OP-4 no
longer defines general runtime bounds-trap semantics; the reference now
lands on OP-4's one retained sentence about SYS-8 itself (circular). Not
acceptance-relevant, but §12's completeness claim misses this sentence —
only SYS-8's target-facing sentence is enumerated. Fix: repoint the phrase
at OP-4's "retained operation-internal contract check" clause (or [ERR-4])
in the same Section D edit.

## F10 (E) — §2 line accounting

"complete replacement of the two changed lines plus one added production;
every other line byte-identical" — only **one** existing line changes (the
`stmt` continuation line); `check_stmt` is quoted byte-identical for
context. §3's "differs from v0.20 by the three quoted lines only" is
correct (1 changed + 1 added + the FORM-2 line). Say "one changed line".

## F11 (E) — Editorial batch

1. **Overlap citation.** ENT-2 ("kills [ENT-5] use resolved-place overlap
   [OWN-5]") and kill (a)/(b): the overlap relation is *defined* by
   [OWN-7]; kill (b)'s "whose storage may overlap" should cite OWN-7's
   exact relation — "may" invites a nonconservative reading.
2. **Break-less loops.** "The continuation of a `loop_stmt` is the join
   over the closed states at its `break` statements" — the empty join is
   undefined. FN-1's conservative graph makes the continuation reachable,
   so state the empty-join value (presumably the contradictory
   all-derivable state, consistent with [ENT-4]'s unreachability posture).
3. **Propagate continuation.** "its normal continuation keeps the preceding
   state and its binder gains no fact" — add "subject to the kill events of
   the initializer call" so kill (b) is not read as skipped.
4. **S6 slice_of.** The `len(s) = len(P)` form omits the "for a tracked P"
   qualifier that the `len` form carries; add it.
5. **S8 metanotation.** `idiv.trap<T>(d, 2_T)` — `2_T` is not a writable
   literal (FORM-5 defines only `0_T`/`1_T` generically); say "the literal
   two of the concrete type T".
6. **Generic bodies.** State that discharge, redundancy, and refutation are
   judged per FN-2 instantiation ("instantiations are re-checked as
   concrete code") — it matters for const-generic length terms
   (`i <= 7` vs `len = N` discharges at N = 100, not symbolically), and
   nothing currently fixes the judgment point.
7. **index_get row.** The row's table position is unstated (it fixes the
   derived dotless-family ordinal in DIAG-1 reservation payloads); the
   `(place, u64)` signature notation is novel next to `len`'s bare
   `-> own u64`; and the place operand should be stated non-consuming in
   body position — FN-8's non-consuming-place-operand sentence is
   requires-scoped, and OWN-1 otherwise makes a bare affine place operand
   an error (`len` inherits the same latent v0.20 ambiguity; ENT-6's
   fallback `let n = len<T>(P);` relies on it).
8. **Non-relational predicates.** CLM-2's redundancy/refutation are only
   meaningful for predicates with comparison origin; state that any other
   conforming predicate (`True()`, a `band` result) is neither redundant
   nor refutable, so `claim c: False() …` is accepted and traps — parallel
   to today's `check False()`.
9. **Claim-name spellings.** Since the name enters no reservation
   inventory, `claim len: …` and `claim wrap: …` are legal while
   `claim trap: …` is not (`trap` is a fixed atom). Intended per O6, but
   worth one sentence so the asymmetry reads as chosen.
10. **S1 origin "killed" wording.** "no operand term of R is killed" —
    kills are defined on facts, not terms; say "no kill event (a)–(d)
    applies to a fact supported by that term on any such path".

---

## Verified — axes and attacks that survived

- **Quoted originals.** All ten original-sentence quotes in §6/§7 (FORM-2,
  FORM-5, GIVE-1, EFF-2 ×2, SET-1, FN-8 ×2, DIAG-3, SYS-8) were checked
  verbatim against v0.20 by exact string search: every one matches.
- **Verifier evidence.** The §3 control run was re-executed: the active
  compiler reports exactly "64 productions, 74 decisions, 75 terminal
  predicates", exit 0, matching the candidate; 64+1=65 and 75+2=77 are
  consistent. `compiler/README.md` does specify the fail-closed
  grammar-extension behavior the probe exhibits.
- **Renumbering claim.** v0.20 contains no cross-reference to §18 or §19;
  the "no existing cross-reference names either renumbered section" claim
  holds.
- **Corpus claim.** `tests/programs/` contains no occurrence of `claim`,
  `because`, or `index_get` (word grep over `*.wf`): the §8 empty-migration
  claim holds.
- **Rule-count accounting.** 8 added (CLM-1..2, ENT-1..6) and thirteen
  modified are internally consistent across §1, §6, §7 — modulo F6's two
  missing rules.
- **Grammar/LL(2).** `because` exits every expr-interior decision exactly
  as `check`'s `else` does; `claim` uniquely selects the new stmt arm;
  FORM-3's derived IDENT exclusion needs no text change. No ambiguity
  found.
- **S8 midpoint arithmetic.** Verified over unsigned T with lo <= hi:
  d = hi−lo exact (no wrap), h = ⌊d/2⌋ under both defining shapes
  (`ishr.wrap` masks 1 to 1; `idiv.trap` by 2 cannot trap), lo+h <= hi
  <= max(T) (no wrap), and m ∈ [lo, hi] with m <= hi−1 when lo < hi. The
  underflow attack (lo > hi) is blocked because facts are established only
  when the state derives lo <= hi. Sound.
- **S10 boundary facts.** The upper bounds match SYS-8's new normative
  paragraph (itself verbatim from SYS-POSTCONDITIONS.md's consolidated
  alternative); the conditional lower bound (`capacity > 0 ∧ ReadBytes ⇒
  count > 0`) is correctly *not* admitted, avoiding the flattening trap the
  survey warns about; the three SYS-9 sentences match the survey's proposed
  wording verbatim; binder/field names (`ReadBytes(count:)`, `Ok(value:)`)
  match SYS-2/PRE-1. Sound given the [QUAL-1] contract trust class, which
  is the same class as S6's allocation length.
- **Kill-rule attacks that failed.** Buffer reassignment (STOR-1 rejects
  `set` of an affine place; `move` re-rooting kills via (c)); element
  writes vs length facts (lengths are allocation-fixed; supports exclude
  element storage); aliasing through borrows (holder resolution folds
  `deref(h)` onto the borrowed place, so kill (a)/(b) fire on either
  spelling); two-uniq-param aliasing inside a callee (excluded caller-side
  by OWN-5/OWN-12, so S4 entry facts about one param survive writes through
  another soundly); writes hidden from effect rows (no globals; slices
  unwritable; consuming own arguments is kill (c)); requires substitution
  (fail-closed on any non-comparison substituted shape; pure/total ops make
  textual duplication semantically harmless); comparison-origin staleness
  (the no-kill-on-any-path condition over the conservative graph, including
  loop back edges, blocks the stale-Bool replay attack); S7 wrap facts
  (established only under derived no-overflow range conditions; trap forms
  use the executed check). No soundness hole found beyond F1/F2/F5.
- **Section D fidelity.** The split (four count bounds as L0 source S10;
  three SYS-9 relations retained-only) matches the survey's (b)/(c)
  classification exactly; the `required > capacity` companion bound is
  correctly recorded-not-proposed on both sides.

## Severity counts

| severity | count | findings |
|---|---|---|
| soundness-breaking | 2 | F1 (fixed in working copy), F2 (open) |
| acceptance-changing | 6 | F3, F4, F5, F6, F7, F8 |
| editorial | ~10 | F9, F10, F11.1–.10 |

Single worst finding at the reviewed commit: **F1** (discharged negative
index compiles with no bounds check) — already closed by ruling O9 in the
working copy. Single worst finding still open: **F2** — spelling-identified
terms plus the unordered scope-exit/join interaction let a stale fact
attach to a fresh same-spelled binding and discharge an out-of-bounds
index; fix by ordering kills before joins on scope-exit edges and anchoring
term identity to declaration events.

---

# Re-verify (2026-08-07, targeted; candidate at commit f38a99e, 944 lines)

Scope: targeted re-verification of the applied fixes for F2–F8 (plus F9 and
the F11 batch as encountered), per lead direction. Not a full re-review.

## F2 — CLOSED (re-attacked, both repairs verified independent)

The revision applies both proposed repairs. ENT-5 now reads "Scope exits
are edge events: kills (c) and (d) apply on every edge leaving the scope,
before any join at that edge's target is taken — mirroring [STOR-3]'s
edge-carried releases — so no arm-local or block-local fact survives its
scope into a join under any reading", and the join clause samples each arm
state "taken after that edge's scope-exit kills and then closed". ENT-2 now
anchors term identity: "Two places are the same term exactly when their
root `pbase` IDENTs resolve to the same declaration event [TYPE-6, DIAG-1]
and their canonical source spellings [FORM-2] are byte-identical; a fresh
binding legally reusing an expired spelling [TYPE-6] is therefore a
distinct term".

Re-attack attempts, all blocked:

- **Original counterexample** (arm-local `let x` facts + fresh `x` after
  the match): blocked twice over. Kill (d) fires on each arm exit edge
  before the join, so the fact never reaches the continuation; and even if
  it did, the fresh `x` is a distinct declaration event, hence a distinct
  term, and no fact links the new term to the dead one (S5 at
  `let x = n;` relates only new-x and n), so no transitive chain reaches
  the stale bound.
- **Break edges**: a fact about a loop-local binding flowing on a `break`
  edge to the loop-continuation join — the break edge leaves the binding's
  scope, kill (d) is an edge event on it, applied before that join. Dead.
- **Give edges**: a value_match arm's `give` edge leaves the arm scope;
  the "every edge leaving the scope" phrasing covers it identically.
- **Propagate early-return edges**: the `Err` edge leaves the function (no
  continuation state to poison); the normal continuation is now explicitly
  "subject to the initializer call's own kill events (b) and (c)".
- **Nested regions / multi-scope exits**: "region exit [OWN-3] included",
  and the per-scope phrasing applies once per scope a single edge leaves.
- **Legitimate-derivation regression check**: facts about outer-rooted
  terms established inside an arm (support scopes not exited) correctly
  survive the arm exit edge into the join — the both-arms case still
  works, so the repair does not over-kill.

Verdict: the counterexample and every variant tried fail against either
repair alone; the revision lands both. CLOSED.

## F3 — CLOSED (constructive flow verified single-outcome)

ENT-3 now defines the state constructively (establishment, forward flow,
edge-placed kills with scope-exit kills before joins, ENT-5 joins and loop
rule, ENT-4 closure of the result) and demotes dominance: "Dominated
straight-line establishment is a consequence of this construction, not a
second definition." A fact established separately in every arm now has one
outcome: each arm exit edge carries it (its support was not scope-killed),
the join keeps the weakest common bound, and it is present at the
continuation in every conforming implementation. Direction check: the
constructive state is a superset of the old dominance-only reading
(straight-line dominated facts flow trivially; merge points only add), and
every added fact holds on all incoming paths from executed sources, so the
resolution is sound and matches the join reading SIMULATION.md assumed —
no acceptance covered by the original finding changes in an unintended
direction. I also verified the "join of closed states is closed" claim the
join rule relies on: the pointwise-weakest of two difference-bound-closed
states satisfies the triangle rule (max distributes over the sums), and
rule (2) adds nothing new post-join because any pair with a common
disequality and a common zero bound already carried the strengthened bound
in each component state. Residual informal uses of "dominating/dominated"
remain at four spots (header source list, CLM-1, ENT-6, OP-4) — all are
now consequence-language over straight-line establishment, not a second
definition; acceptable as prose. CLOSED.

## F7 — CLOSED (one definition; reflexivity restored)

ENT-4 now states "This least closure is the one definition" and the
shortest-path sentence is gone (grep for "shortest" over the candidate: no
hits; the §1 header now says "one least-fixed-point difference-bound
closure"). ENT-2's implicit facts gain "every term t carries the reflexive
bound `t - t <= 0`". The S8 same-term derivation is restored: with lo and
hi the same term t, `lo <= hi` is the implicit `t - t <= 0`, so the
midpoint facts establish. Side-effect check: reflexive bounds compose to
no-ops under rule (1) and do not mask contradiction detection (rule (3)
keeps the smaller constant, so a derivable `t - t <= -1` still wins over
the implicit 0). CLOSED.

## F4 — CLOSED, one editorial residue

ENT-1 now states the law scoped, with refutation as "the lifecycle's one
deliberate non-monotone edge; no other judgment of this family may tighten
acceptance across versions", and CLM-2 opens by naming that edge. The §1
header describes the lifecycle as "version-monotone in the redundancy
direction" and names the refutation edge. **Residue (editorial, fix before
approval):** the header's closing sentence (lines 106–107) survived
unscoped — "checker strengthening may only convert claims to advisories
and undischarged obligations to discharged ones" — still asserting the
absolute the same paragraph disclaims earlier. These bytes become the
v0.21 status header; append "with [CLM-2] refutation as the one enumerated
exception" (or equivalent). Semantics correct; one sentence of bytes is
not.

## F5 — CLOSED for the finding's classes, one adjacent residue

ENT-2's term roots are widened exactly as proposed: "any `let_stmt`
binding (whichever of the three right-hand forms — ordinary, `propagate`,
or `value_match` — the statement selects), a `param`, a requires-clause
local, any match binder regardless of its [OWN-13]-derived mode, or a
named const". The counterexample now discharges: a propagate-bound buffer
is tracked, `len(buf)` is a length term, and the ENT-6 fallback closes the
site. I checked the widened roots against the kill rules: binder-rooted
places resolve through OWN-13's child-reborrow resolution so kills (a)/(b)
fire on aliasing writes, and binder facts die at arm exit under the F2
edge rule — no soundness regression. With v0's flat-element rule (TYPE-2:
no array-of-array elements) every legal index base is now term-formable.
**Adjacent residue (editorial):** ENT-6's "that fallback always closes
discharge" still overreaches for an *offset* atom that is itself an inline
`index` place (legal: GRAM-9 offsets are atoms, atoms include places,
places may be index-rooted). A non-term offset leaves the relation
underivable and no claim can state it; the actual fix is rebinding the
offset through an ordinary let and indexing with the binding. One sentence
("an offset that is not a term is first rebound through an ordinary
`let`") makes the sentence true.

## F6 — CLOSED (both knock-ons landed; accounting verified)

FN-1 is now a §6 entry quoting the exact v0.20 edge-enumeration sentence
(verified verbatim against the base) and adding "or `claim`"; DIAG-1 gains
the claim-name carrier class, cross-referenced from CLM-1. Accounting
recounted independently: §6 lists thirteen entries (FORM-2, FORM-5,
GRAM-4, GIVE-1, OP-1, OP-4, FN-1, FN-8, EFF-2, SET-1, DIAG-1, DIAG-2,
DIAG-3) plus §7's SYS-8 and SYS-9 = **fifteen**, matching the header's
"fifteen existing rules modified" and its itemized list, and §7's
"candidate total: fifteen modified rules" line. §12's completeness claim
now enumerates the three caught collisions. **Nit (editorial):** the
DIAG-1 entry states the added sentence but not its insertion point within
DIAG-1's carrier paragraphs; every other §6 entry gives a before/after
anchor, and the full-document generation will need one.

## F8 — CLOSED (buckets restated consistently)

§9 sha256 now reads "add 4 claims — the loop-head pair `16 <=
extend_index` and `extend_index < 64` ... plus two others (hottest loop: 5
checks/iteration -> 2 claim checks at L0)", and §10 requires "4 claims
covering 8 sites" with an explicit note that SIMULATION.md's frozen "3
claims" row counts the conjoined claim once where ruling O11 requires two.
Arithmetic checked: 1 conjoined + 2 others = 3 becomes 2 + 2 = 4; hot-loop
5 -> 2 is the split pair. Leaving SIMULATION.md unedited is correct
(frozen research; editing evidence to match a later rule is the exact
regenerate-to-go-green breach standing law forbids). CLOSED.

## Bonus checks

- **F9 — CLOSED**: a second SYS-8 edit repoints the range-validation
  cross-reference ("traps as the operation-internal contract check
  retained by [OP-4] [ERR-4]"), and the header records it.
- **F11 batch — all ten applied**: OWN-7 cited for overlap in ENT-2 and
  kills (a)/(b); empty break-join defined (see choice 1 below); propagate
  continuation subject to kills (b)/(c); S6 slice_of gains "for a tracked
  P"; S8's `2_T` replaced by "the literal two of the concrete type T";
  per-instantiation judgment stated in ENT-1; index_get row position fixed
  (appended last) with the place operand stated non-consuming; CLM-2
  defines non-comparison predicates as neither redundant nor refutable;
  CLM-1 states the reserved-name asymmetry as chosen; S1/S7/S8/S10 origin
  conditions reworded to "no [ENT-5] kill event applies to a fact
  supported by ...". "coordination pending" phrasing is gone.

## Revision-pass notes — position on the five recorded choices

1. **Empty-join value = contradictory all-derivable state** (owner
   sitting). **AGREE — recommend adopting.** The continuation of a
   break-less loop is unreachable in truth (v0 loops exit only via `break`
   naming their label; FN-1's extra edge is deliberately conservative for
   reachability only), and this is exactly the already-ruled O8 posture
   for unreachable-in-truth states: discharge everything, refute nothing.
   The alternative (empty fact state) would demand claims after the loop
   that can never execute — dead checks as review noise. It is also the
   algebraically forced choice: the join over zero states is the join
   identity, so anything else breaks join associativity on edge cases.
2. **len/slice_of/index-base non-consuming clarification as a sixteenth
   modified rule** (owner sitting). **AGREE — recommend landing it in this
   batch.** ENT-6's fallback (`let n: own u64 = len<T>(P);`) is now
   load-bearing normative machinery at every migration site, and it rests
   on a reading v0.20 never states: OWN-1 literally makes a bare affine
   place operand a hard error, and FN-8's non-consuming sentence is
   requires-scoped. Strictly read, the batch's own mechanical fix is
   ill-formed for exactly the affine bases (buffer, array) it exists to
   serve. One sentence in OP-1 (place operands of `len`, `slice_of`,
   `index_get`, and the `index` base are non-consuming reads [OWN-1])
   closes it cheaply and pre-empts the next review or a second
   implementation finding the same hole.
3. **F2 fixed by both repairs, not only the mandatory one.** AGREE — each
   repair independently blocks the counterexample (verified above), and
   belt-and-braces is proportionate for machinery whose wrong derivation
   compiles a raw out-of-bounds access into the TCB's blind spot.
4. **sha256 restated to 4 claims; SIMULATION.md left frozen.** AGREE —
   editing frozen evidence to match a later rule is the
   regenerate-to-go-green breach; explaining the divergence at the two
   consumption points (§9, §10) is the correct mechanism.
5. **F4 worded as absolute redundancy-monotonicity plus refutation as the
   enumerated tightening edge; advisory alternative not taken.** AGREE
   with the direction — it matches DOSSIER §2.7's deliberate lifecycle
   ("checker upgrade refutes a claim → hard error") and the shift-left
   goal; a claim proven false on every reaching execution is a defect, and
   an advisory would knowingly ship a guaranteed trap. The only remaining
   defect is the unscoped header sentence recorded under F4 above.

## Re-verify verdict

| finding | verdict |
|---|---|
| F2 | CLOSED (both repairs verified; re-attack variants all blocked) |
| F3 | CLOSED (single-outcome verified; join-closure claim checked) |
| F4 | CLOSED — one editorial residue: unscoped header sentence (lines 106–107) |
| F5 | CLOSED for the finding's classes — one editorial residue: "always closes" vs inline-index offsets |
| F6 | CLOSED (fifteen-count verified independently) |
| F7 | CLOSED (one definition; reflexivity restored; no side effects) |
| F8 | CLOSED (4-claim restatement arithmetically consistent) |
| F9 | CLOSED (cross-reference repointed) |
| F11 | all ten applied |

No new soundness or acceptance-changing finding surfaced during re-attack.
Remaining work is three editorial items (F4 header sentence, F5 offset
caveat, F6 DIAG-1 insertion anchor) plus the two owner-sitting choices,
on both of which this review recommends adoption as recorded above.

---

# v0.22 review (2026-08-07; index-surface candidate at a5e1cdb, against installed spec/kernel-spec-v0.21.md)

Part 1 is the targeted adversarial pass on
`governance/spec-evolution/index-surface-v022-candidate.md`; Part 2 is the
first application of the standing residue-hunt axis over the v0.21 surface
plus this candidate's additions. Finding IDs continue as V* (v0.22) and
R* (residue).

## Part 1 — targeted adversarial pass

### Anchors and accounting — verified, one definitional wobble

All **21 quoted anchors** were checked verbatim against
`spec/kernel-spec-v0.21.md` by exact string search: every one matches
exactly once (FORM-2 ×2, GRAM-5 ×2, GRAM-6, GRAM-9, SET-1 ×2, CONST-2,
OWN-7, OP-1 ×2, OP-4, FN-8, EFF-2, DIAG-1, DIAG-2, ENT-2, ENT-3, ENT-6
×2). The **sixteen modified rules** are exactly the sixteen the header
itemizes. The **nineteen sites** reproduce only as the header itemizes
them per rule (SET-1, OP-1, ENT-6 at two each, the rest at one); no single
definition of "site" yields 19 — contiguous paragraphs give 18 (ENT-6's
two edits share one paragraph), edited sentences give 20 (FORM-2 edits
two), verbatim anchors give 21. Editorial: define "site" or replace the
number with the stable anchor count.

**Completeness** (the axis that caught F6 last time): an exhaustive sweep
of every construct-level `index`/`index_get` occurrence in v0.21 maps each
one to the sixteen rules or the register note — lines 179 (GRAM-5), 183
(GRAM-6), 191 (GRAM-9), 232/236 (SET-1), 242 (CONST-2), 264 (OWN-7), 360/
364 (OP-1), 386 (OP-4), 486 (FN-8), 506 (EFF-2), 573 (DIAG-1), 652
(DIAG-2), 1012 (ENT-2), 1038 (S9), 1051 (ENT-6 ×2), line 34 (the
R3-PROVISIONAL register entry, covered by the candidate's register
reduction), line 3 (the frozen v0.21 status header, correctly untouched),
and English-usage-only lines 282 ("arena-index-pool") and 962 ("expose,
index, or mix"). The §19 worked example contains no index spelling and no
bracket, so no EX-1 respell is owed. No missed rule.

### V1 (acceptance-changing spec-craft, must fix — via O3): the drafted
rejection/obligation anchor is undefined for non-final subscripts

**Candidate text:** "at the place node formed by that subscript suffix"
(OP-4 rewrite, twice, and ENT-6), glossed in O3 as "the innermost place
node whose final suffix is the subscript".

**Attack.** Under the new grammar a place is one flat production —
`place := pbase psuffix*` — so `a[i][j]` derives **one** place node with
three children (pbase `a`, psuffix `[i]`, psuffix `[j]`); there is no
intermediate place node for `a[i]` (GRAM-1's 1:1 production-to-node
mapping; the offset atoms `i`, `j` are their own nodes, the prefix chain
is not). For the inner subscript `[i]`, no place node has `[i]` as its
final suffix — the drafted anchor selects nothing; for the chain's two
bounds obligations, both would collapse onto the single place node,
breaking DIAG-2's "retains its exact [ENT-4] derivation **for that
node**", leaving the residual choice at a two-rejection node ambiguous,
and giving the wrong-base error at an inner subscript no defined
location. Under the old grammar each nested `index` was its own pbase
node, so v0.21 had per-subscript identity for free; the respelling loses
it exactly where the drafted anchor pretends to keep it.

**Fix.** Rule O3 to its listed alternative: anchor at the subscript's
`psuffix` node (a real node with its own extent, one per subscript), and
align the three dependents — the undischarged rejection, the wrong-base
error, and ENT-6's obligation attachment ("at that subscript `psuffix`
node"). DIAG-2's per-node derivation retention is then well-defined for
chains.

### V2 — LL(2) argument attacked and survived

What was tried, per the lead's list plus variants:

- **Follow set of a complete place.** Enumerated every context a `place`
  can occupy in v0.21: atom positions (followed by `,` `)` `;` `{`
  `else` `because` and, newly, `]`), set targets (`=`), deref interiors
  (`)`), borrow operands, scrutinees, claim/check conditions. No
  production places `[` after a complete place — region_params follow a
  declaring `fn` IDENT or a generics `>` (declaration positions, no place
  decision live), and cvalue `[` follows `=`, `,`, or `[` inside
  `const_decl` (no place decision live). So the `psuffix*` decision's
  consuming rows (`.`-IDENT, `[`-first(atom)) are disjoint from every
  exit-continuation row. 
- **Generic argument lists**: at an expr decision, (IDENT, `[`) selects
  atom→place-with-subscript while call rows are (IDENT, `<`) and (IDENT,
  `(`) — two-token disjoint; targs interiors contain no `[`.
- **Nested subscripts**: `a[b[i]]` and `a[b][i]` are distinct token
  streams, each parsed deterministically (inside an offset atom the inner
  place's own `psuffix*` consumes `[` greedily; `]` selects neither arm
  and exits) — matching the candidate's O4 posture.
- **Attachment (O1) interaction**: attachment sets are byte-format rules
  applied after parsing; token streams are unchanged, so the O1 choice
  cannot create a parse ambiguity in any direction.

No ambiguity found; the candidate's §2 argument is correct (its
parenthetical omits the `fn f<T> ['r](` generics case, but the conclusion
covers it — cosmetic).

### V3 — spelling-transport verified; exactly two semantic deltas, both
flagged

Side-by-side of the OP-4 rewrite against v0.21's paragraph: every
sentence is byte-transported except (a) the element-type derivation
sentence, which is the ruled A2 item itself (the deleted targ's
information moves to a stated derivation; "declared-type selection that
types a field suffix" is the right analogy and imports no inference), and
(b) the wrong-base hard error (O7, assessed below). The `index_get`
sentences leave with the operation. The ENT anchor moves (ENT-2, S9,
ENT-6) carry no drift — S9's `own T` annotation still states the element
type at the binding, so the fact family is unchanged. SET-1's
restatement is equivalent: "base place evaluated before its offset atom"
plus the retained "from its base outward" reproduces the old nested
order (a, i, j for `a[i][j]`). The claim "narrows semantically nowhere"
holds for the respelled program set; programs whose old spelling wrote a
*wrong* element type simply lose the wrong byte along with the right
one, which is the deletion working as ruled.

### V4 (editorial): §6's measured footprint is wrong in both ripple counts

Re-measured 2026-08-07: subscript sites **266** (`tests/programs/*.wf`)
and **138** (`tests/conformance/`) match, `index_get` nowhere matches. But
region-parameter headers are **84**, not 88 (` ['` over both corpora's
`.wf`, and the unfiltered count is also 84); and cvalue arrays are
**31**, not 40 — the 40 comes from an unfiltered grep counting nine
`= [` lines in `tests/conformance/runner.py` and `test_runner.py`, which
are Python list literals that do not reprint. A governance candidate's
"Measured footprint" must measure canonical sources only.

### V5 (editorial): two unenumerated attachment-ripple classes

(1) The installed spec's own bytes: the [SYS-2] signature block spells
nine ` ['` region-parameter headers (`fn args_count ['a](…` …), and under
O1 the canonical `fn_sig` rendering changes; the full-document v0.22
candidate must either respell that block or state that record notation is
exempt from FORM-2 attachment — §6 enumerates neither. (2) The `, [`
boundary: with `[` right-attaching, a nested cvalue array renders
`],[` after a comma; the corpus has zero such sites today (measured), but
the class is real and the ripple enumeration should name it so the
printer change is reviewed against it.

### V6 (editorial): OWN-6/OWN-13/OWN-14 bracket metanotation now collides
with live syntax

v0.21 writes reborrow forms as `&uniq 'c deref(h)[suffix]` (and
`resolved(child) = resolved(h) ++ suffix`), where `[suffix]` is
metanotation for an appended suffix chain. Once `p[i]` is real syntax,
`deref(h)[suffix]` reads as a subscript whose offset is a binding named
`suffix`. No semantic effect, but the same batch should restate the
metanotation ("`deref(h)` followed by any written suffix chain") in
OWN-6/OWN-14 (and OWN-13's `++` gloss) or the v0.22 full document
inherits a genuinely misreadable normative sentence.

### O1–O7 recommendations

- **O1 (attachment fork): adopt `[` into the right-attachment set.** A
  mandatory `p [i]` defeats the readability ground the respelling stands
  on, META-2 rightly forbids a per-context set, and the ripple is
  printer-mechanical — but land it with the corrected counts (84 + 31)
  and the V5 enumeration.
- **O2 (register): adopt the settlement reading.** SWEEP's four-test
  verdict is exactly the validation the R3-PROVISIONAL register demands;
  re-entering the new spelling as fresh-provisional would make every
  respelling self-perpetuating in the register. The surviving "deref
  prefix places" half stays provisional, which the candidate gets right.
- **O3 (rejection-node identity): reject the drafted anchor, adopt the
  `psuffix`-node alternative** — see V1; the drafted option is undefined
  for chains.
- **O4 (nested subscripts): confirm no tightening.** Mirrors v0.21;
  ENT-6's rebinding sentence already prices the no-term offset;
  tightening grammar here would be semantics riding a spelling batch.
- **O5 (released spellings): confirm no soft-reservation.** A soft
  reservation would be a third naming state the language has nowhere
  else; the derived sets exist so reservation follows the table
  mechanically. Add one conformance case using `index` as an ordinary
  binding to pin the release.
- **O6 (verifier sequencing): confirm the batch-1 shape.** §3 correctly
  records fail-closed expectations and defers exact counts to the
  grammar-path task.
- **O7 (wrong-base attribution): keep the sentence.** It is a legitimate
  statement of existing behavior's attribution, not new semantics: a
  non-indexable base was never accepted under v0.21 (OP-4's applies-to
  list gave it no meaning; SCOPE-2 accepts only what every rule
  satisfies), so the sentence changes no program's acceptance — it pins
  which rule and node an already-mandatory rejection cites, exactly the
  class of the ruled batch-1 O9 offset-typing sentence, and the batch
  deleting the redundant element type is the right moment to pin it
  (wrong-base confusion becomes likelier at exactly these sites). One
  adjustment: its drafted location inherits V1; re-anchor per O3, and
  consider the base place node rather than the subscript node — the
  defect is the base's type, and the offset-typing error already anchors
  at the offset atom, so base-side symmetry reads best.

## Part 2 — residue hunt (first application of the standing axis)

Method: for each construct on the v0.21 surface plus this candidate's
additions, the four questions — (1) re-derivable from the kernel today,
(2) native need vs imported habit, (3) what family the justification
licenses and whether it is bounded, (4) one mechanism per concern.

### R1 (residue) — ENT-3 S8, the midpoint family

**Fails question 1 and strains 2 and 3.** Corpus evidence (measured
2026-08-07): zero sites in `tests/programs/` and `tests/conformance/`
match the three-let shape — the only shift-by-one sites are a CRC bit
fold (`crc >> 1`) and a pool-tree loop bound (`half = width >> 1` with no
`isub.wrap(hi, lo)` and no `iadd.wrap(lo, half)`), and no binary search
exists in either corpus; the `idiv.trap(_, 2)` alternative shape appears
nowhere. SIMULATION.md lists "halving" in its L0 capability description
but names no site that demanded it. So the kernel, asked today, answers
the midpoint with one claim per site (the same relocate-and-price posture
the dossier assigns to loop facts) — we would not invent a bespoke
three-statement shape matcher for a shape no program writes. On question
2, the overflow-safe midpoint is cross-language folklore (the canonical
Java binary-search bug), imported in anticipation rather than demanded by
this corpus. On question 3, shape-keyed fact sources invite accretion,
and the asymmetry shows it: the manifestly more-demanded shape for this
corpus's future (the `irem` remainder bound `r < n` — ring buffers, hash
tables) is absent while midpoint is present, i.e. selection tracked
anticipation, not need. It also grazes the project's own taste rule
("by grammar and semantic rule, never by … source shape") — S8 is
normative source-shape-keyed semantics, the only ENT-3 source keyed to a
multi-statement pattern rather than one node plus a path condition.
Soundness is untouched (the family was verified exact in the original
review); this is a residue verdict. **Recommendation:** strike S8 from
the next fragment revision or park it until a corpus program writes the
shape; ENT-1's monotone versioning exists precisely so it can be re-added
for free the day binary search lands. (Precedent: `index_get` survived
four passes on soundness grounds and fell to exactly this question.)

### R2 (residue, tracked) — S2 check-facts beside S3 claim-facts

Fails question 4: `check` and `claim` are two spellings of one
trap-check concern, both fact sources, differing only in name and
lifecycle. Already owner-acknowledged (batch-1 ruling O7 defers `check`
deprecation to the FLOOR-5 spelling batch); recorded here so FLOOR-5
inherits it as the standing duplicate rather than rediscovering it.

### R3 (residue, low) — the three SYS-9 relations with no L0 consumer

Fail question 1 as of today: normative retained facts whose only stated
effect is "fail-closed retention plus review value" until cross-call
congruence exists. The SYS-12 precedent is weaker than it looks — its
retained redirection fact names the exact future consumer it fails
closed against (a cross-resource reordering fact), while these three
name none. Owner-ruled (O16) and cheap, so keep — but the pattern to
watch is "normative facts landed ahead of any consumer"; the residue
axis should re-ask question 1 of these rows when cross-call congruence
is next scoped.

### Constructs examined and passed

- **`because` justification string** — passes 1 (the untrusted-writer
  authorship factoring demands auditable prose at the assertion site;
  today's consumer is the human reviewer, which is real), 2 (SPARK's
  workflow is prior art but the mandate derives from W3's own frame), 3
  (branches need no string — the written else is their justification;
  bounded), 4 (per-assertion data, distinct from declaration `doc`).
- **Claim name discipline** — passes; the dual identity with `node_path`
  is two stability classes (edit-stable vs positional), not duplication.
- **CLM-2's three verdicts** — all forced (redundancy-advisory is the
  monotonicity keystone; refutation is the ruled shift-left edge;
  fired-claim escalation is toolchain, not language). Question 3 note:
  the advisory family will grow by the dossier's dead-else lint
  (§4.1c); enumerable, bounded.
- **ENT-3 S1, S4, S5, S6, S7, S9, S10 and implicit type ranges** — each
  is either structurally forced (S1/S4/S5/S6) or anchored to a named
  probe demand (S7 cursor arithmetic and S9 const tables in
  SIMULATION.md; S10 in PROBE-TAINT.md; type ranges in the u8-table
  prediction). Only S8 lacks a demand (R1).
- **Boundary adapters** — forced by the kernel (foreign entries have no
  call site, so someone must run the prologue); trap semantics
  unchanged; noted that gated FFI is a stubbed path today, but the
  sentence restates an existing FN-8 obligation rather than adding
  machinery.
- **SYS count postconditions (the four S10 bounds)** — pass;
  PROBE-TAINT promoted them to load-bearing before they were drafted.
- **v0.22's subscript respelling** — passes: imported idiom on its face,
  but the R3 register held the old prefix form as explicitly
  provisional pending exactly this validation, and SWEEP's four tests
  are the evidence form the register demands.
- **v0.22's element-type deletion** — passes, with the doctrine line
  worth recording: redundancy **with transposition risk** stays
  (GRAM-8/GRAM-11 names guard reorderable same-typed slots); redundancy
  with a unique reconstruction carries zero check value and goes (one
  possible T per site). Future deletion candidates should be tested on
  that line, not on "redundant" alone.
- **v0.22's index_get removal** — the precedent this axis is built on;
  the rationale of record ("the washing branch is the kernel's total
  access") is the question-1 answer verbatim.

## v0.22 review verdict

Part 1: the candidate is in good shape — all 21 anchors verbatim, rule
accounting correct, completeness sweep clean, LL(2) solid, transport
faithful. One must-fix before approval: **V1/O3** (per-subscript node
identity; adopt the psuffix-node alternative and re-anchor OP-4's two
errors and ENT-6's attachment). Three editorial: V4 (correct the
84/31 footprint), V5 (SYS-2 block and `,[` ripple classes), V6
(reborrow metanotation), plus the site-count definition nit. O1, O2,
O4, O5, O6 recommended as drafted; O7 keep with the V1 re-anchor.

Part 2: three residue findings — **R1 (S8 midpoint family, recommend
strike-or-park)**, R2 (check-beside-claim, tracked to FLOOR-5), R3
(SYS-9 no-consumer rows, keep but named as the pattern to watch). The
axis's first sweep found exactly one construct in the S8 position —
machinery justified by an anticipated program rather than a written
one — which is the `index_get` failure mode recurring one layer deeper.

---

# FLOOR-5 review (2026-08-07; spelling-relief candidate at 75dd5f6, against installed `spec/kernel-spec-v0.22.md`)

Target: `governance/spec-evolution/spelling-relief-candidate.md` (592
lines). Base: `spec/kernel-spec-v0.22.md` (installed 817a8a7). Context
authorities: `research/investigations/spelling-relief/SWEEP.md` (the
T1–T4 rule and the A/C verdicts this batch implements). Reviewer did not
author the candidate. This section records findings only; it changes no
candidate text. Severity legend as above: **S** soundness-breaking, **A**
acceptance-changing, **E** editorial.

Summary: 1 S, 9 A, 7 E, 3 residue findings. Worst by severity: **F1**
(no ENT-5 join rule for `if` continuations; the permissive completion
deletes a bounds check). Worst for the batch's viability: **F2** — the
drafted `expr` production is not strong-LL(2), and the candidate's own
EX-1 cannot be parsed by the grammar the candidate drafts. F1, F2, F3,
and F4 are must-fix before approval.

Anchors: all eight verbatim quotations I spot-checked against v0.22 are
byte-exact (GRAM-6's no-if sentence, GIVE-1's declared-type anchor and
delivery recursion, GRAM-9's atom-position list, OP-2's explicit-type-
argument paragraph, ERR-2's exhaustiveness sentence, GRAM-1's compound-
token sentence, FORM-2's value-match prefix sentence), as are FN-8's two
and DIAG-1's, DIAG-3's, and EFF-2's. The anchoring craft is good where
the candidate anchors at all; see F11(a) for where it does not.

---

## F1 (S) — No ENT-5 join rule for `if` continuations; the reading ENT-3 itself offers deletes a bounds check

**Candidate text (§3 [ENT-3]):** S1's establishment sentence becomes
"For an `if_stmt` or `value_if` whose condition has comparison origin R,
R is established at the then-block's entry and R's exact negation at the
else-block's entry; for an else-free `if_stmt`, the negation is
established on the false edge, **which joins the then exit at the
continuation [ENT-5]**."

**v0.22 [ENT-5], complete join text:** "Joins: at the continuation of a
`match_stmt` or `value_match`, the fact state is the join of the states
on every arm exit edge reaching that continuation… The continuation of a
`loop_stmt` is the join over the states on its `break` edges…" There is
no third sentence. **ENT-5 is not in the candidate's twenty-two-rule
modification list**, so after this batch the fragment defines joins for
`match`, `value_match`, and `loop` only — and the batch has just made
`if` the sole legal spelling of every Bool conditional, i.e. the
overwhelmingly common merge point.

**Attack.** [ENT-3]'s constructive definition reads: "each source below
establishes its facts at its stated point; **facts flow forward along
normal edges**; kill events apply on the edges where [ENT-5] places
them, with scope-exit kills applied before any join; **merge points take
the [ENT-5] join**". If an `if` continuation is a merge point, ENT-5
supplies no join and the state is undefined — ENT-1's law ("two
conforming implementations derive the same closed fact state at every
program point") is violated outright. If it is *not* a merge point —
textually available, because ENT-5 enumerates which continuations are
joins and this is not one of them — then the first clause governs and
facts flow forward along the normal edges:

```
fn f(b: own buffer<u8>, i: own u64) -> own u8 traps {
  let n = len(b);
  if ilt(i, n) {
    let seen = b[i];
  }
  let x = b[i];
  return x;
}
```

The then-block establishes `i < n` (S1). Its support is `{i, n}`, both
bound outside the block, so no [ENT-5] scope-exit kill touches it on the
block's exit edge. Under the flow-forward reading that fact reaches
`let x = b[i];`, where it discharges `i - len(b) <= -1`. [OP-4]: "A
discharged subscript reads or writes with **no runtime bounds check in
every build mode**." Calling `f` with `i >= len(b)` reads out of bounds
with no check. The correct completion — join the then exit against the
false edge, which carries `n <= i` — leaves the obligation underivable
and rejects, which is what v0.22 does today for the `match` spelling of
the same program.

This is the exact shape of F1/F2 in the batch-1 review: the normative
bytes as drafted license the access, and no open question is drafted
against it (the candidate does not list ENT-5 as touched at all).

**Fix.** Add ENT-5 to the modification list with a fourth join sentence:
at the continuation of an `if_stmt`, the state is the join of the
then-block exit edge and the else-block exit edge (or, for an else-free
`if_stmt`, the false edge), each taken after that edge's scope-exit
kills and then closed — word-for-word parallel to the `match_stmt`
sentence, including the "an arm every path of which leaves by `return`,
`break`, or `propagate`'s error edge contributes nothing there" clause,
which an `if` needs identically. A `value_if` needs the same treatment
as `value_match`.

---

## F2 (A) — The drafted `expr` production is not strong-LL(2); EX-1 does not parse

**Candidate text (§2):** "`expr := atom | call | construct | infix`",
"`infix := atom infix_op atom`", and the justification: "the `expr`
decision distinguishes `infix` from a bare `atom` at the second token
(an operator token follows the first atom; no other continuation of a
complete atom begins with an operator token)".

**v0.22 [GRAM-1]:** "every choice, optional, and repetition decision has
pairwise-disjoint strong-LL(2) `SELECT_2` languages, so a parser selects
exactly one arm with at most two tokens", and "`SELECT_2` and the
two-token parser bound count the expanded raw formed tokens".

**Attack.** The justification confuses "the token following the complete
atom" with "the second token of the phrase". `SELECT_2` is the language
of the first *two tokens*. An `atom` is one token only when it is a
literal; every other atom is longer:

| atom shape | first two tokens |
|---|---|
| `a.f` | `(IDENT, ".")` |
| `a[i]` | `(IDENT, "[")` |
| `deref(p)` | `("deref", "(")` |
| `move p` | `("move", IDENT)` |
| `&'r p` | `("&", REGIONID)` |

Each pair belongs to `SELECT_2(atom)` **and** to `SELECT_2(infix)`, so
the four-way `expr` choice is not pairwise disjoint and the parser
cannot select an arm within its two-token bound. Minimal witness pair:

```
let x = a.f;            // expr -> atom
let x = a.f + 1_i32;    // expr -> infix
```

Both begin `a` `.`. The decisive token sits at an unbounded distance
(`a.f.g.h.i + 1_i32`).

The candidate's own [EX-1] replacement contains the failure:

```
let v = match deref(p) +checked 2_i32 {
```

`value_match`'s scrutinee is an `expr`; the two tokens at that decision
are `deref` and `(`, which is exactly `SELECT_2` of a bare `place`
atom. The normative worked example is unparseable under the normative
grammar of the same document.

This is not a drafting slip in one sentence — it is the reason C1 looked
cheap. SWEEP's key fact ("with GRAM-9 (ANF) retained, an expression
contains exactly one operation, so no precedence table exists and the T3
uniqueness argument is trivial") is true and survives; what does not
survive is the inference that *no precedence* implies *no left
factoring*. Precedence and factoring are independent problems.

**Fix.** Left-factor at the shared prefix:

```
expr        := atom infix_tail? | call | construct
infix_tail  := infix_op atom
```

The optional's decision point is after the complete atom, where
lookahead is one token: an operator token selects the tail, and
FOLLOW(`expr`) — `;` (ordinary/propagate let, `return_stmt`,
`expr_stmt`), `{` (`if` condition, `match` scrutinee), `else`
(`check_stmt`) — selects the skip. Those sets are disjoint at one token,
so the factored form is strong-LL(1) at that decision and needs no new
argument.

Cost the candidate must then re-state: productions become 65 + 5 = 70,
not 69; `infix` as a named node kind is either dropped in favour of
`infix_tail` carrying the node (preserving GRAM-1's 1:1 production-to-
node mapping) or GRAM-1 gains a sentence saying the `expr` node kind is
`infix` exactly when the tail is present. §2's strong-LL(2) paragraph
must be rewritten, not patched: the sentence quoted above is false as
written and should not survive into the spec.

**Process note.** `CLAUDE.md` requires the new grammar to be verified
with the native grammar verifier *before* a spec change is proposed. The
candidate defers it ("Verifier expectations: fail-closed against the
v0.22 tables (grammar-extending), recorded at proposal; the grammar-path
task extends the lexer/parser first"), following the v0.21/v0.22
precedent recorded in `docs/done/0030-grammar-path-extension.md`. This
is the case where the deferral cost something: the verifier's
`SELECT_2` check is precisely what catches F2, and it would have caught
it in minutes. Independently confirmed while checking accounting: the
v0.22 baseline really is 65 productions (61 in fenced blocks plus the
four inline `const`, `cvalue`, `effect`, `effects`), matching the
verifier's 65 in task 0030.

---

## F3 (A) — Three table-operation classes lose a type argument that no operand can supply; the corpus cannot be migrated

**Candidate text (§3 [TYPE-5] replacement):** "Call sites state
explicitly exactly what their callee class requires: type, region, and
const arguments for user generics [FN-2], region arguments for system
operations [SYS-2], and **type arguments for the type-choosing
operations `cvt`, `reinterpret`, and `array_new`** [OP-6, CONST-1] —
required there, **forbidden on every value-typed table operation, whose
selected type is operand-derived** [OP-2]."

The class rule is stated over the whole table but the derivation is only
ever argued for [OP-2]'s two-operand and one-operand integer paragraphs.
Three classes fall outside it.

**(a) Nullary operations — no operand exists.** [OP-1] row: "`finf`
`fnan` | f32 f64 | `() -> own T` | pure". Zero operands, so nothing to
derive from, and the targ is forbidden. Corpus witnesses, both live:

```
tests/programs/telemetry_packet.wf:68:  let nan: own f32 = fnan<f32>();
tests/programs/telemetry_packet.wf:89:    let infinity: own f32 = finf<f32>();
```

Under the candidate these two lines have no legal spelling at all. The
printer cannot compute one, which directly refutes §5's "All migration
is printer-driven … zero semantic judgment". [OP-8] compounds it: the
retained sentence "negative infinity is `fneg(finf<T>())`" is normative
text spelling a construct the batch forbids.

**(b) `arena_new` — the region is not operand-derived.** [OP-1] row:
"`arena_new` | any T | `(own T) -> own arena<'r, T>`". `'r` appears only
in the result. `arena_new` is a table operation, not a [SYS-2] system
operation — §17 is explicit that "Every system operation is nongeneric…
A call whose callee resolves to a system operation writes its region
arguments as `targs`", and `arena_new` is not in that inventory — so the
"region arguments for system operations" clause does not cover it and
the "forbidden on every value-typed table operation" clause does. Three
corpus sites, e.g.

```
tests/conformance/cases/stor4-pos-arena-confined.wf:4:    let a: own arena<'r, i32> = arena_new<'r, i32>(4_i32);
```

Deleting `<'r, i32>` leaves the arena's region unrecoverable, and A3
simultaneously deletes the `let` annotation that was the only other
place `'r` appeared. This one is worse than (a): it is not merely
unwritable, it is a region-inference demand, which is exactly what
TYPE-5's own replacement text swears off ("This is unique
reconstruction within one statement, not inference").

**(c) `array_new`'s second argument is a const, not a type.** The
candidate licenses "**type** arguments for the type-choosing operations
… `array_new`", but [GRAM-3] `targ := type | REGIONID | const` and the
corpus writes `array_new<i32, 4>`, `array_new<T, n>`,
`array_new<i32, name>` (12 sites). [CONST-1]'s const-generic forwarding
path — "`const N` is usable as an `array<T, N>` size and forwardable as
a `const` targ" — depends on that second argument. As worded, N is not
licensed.

**Fix.** State the retained-targ class by enumeration over the operation
table rather than by the phrase "type-choosing", and make it total: the
operations that keep written arguments are exactly `cvt`, `reinterpret`,
`array_new` (type *and* const), `arena_new` (region and type), and
`finf`/`fnan` (type); every other table operation's selected type is
operand-derived and its targ is forbidden. Then re-count: §1 claims
"only the type-choosing structural operations (`cvt`, `reinterpret`,
`array_new`) keep type arguments, everywhere and mandatorily", which
becomes six spellings, not three, and §5's "1260 call sites" needs
re-measuring against the corrected class (I could not reproduce 1260
with a simple regex; my upper bound over all lowercase-callee targ call
sites in the two named directories is 1666, of which 96 are
cvt/reinterpret/array_new, the remainder including user-generic calls
that keep theirs — so 1260 is plausible but unverified, unlike the five
counts in F11(g) which reproduce exactly).

---

## F4 (A) — A value initializer with an empty delivery set has no derived type

**Candidate text (§3 [GIVE-1]):** "every delivering `give` of one value
initializer must have one identical exact mode and type, which is the
binding's derived mode and type [TYPE-5]". **[TYPE-5] replacement:** "a
`value_match` or `value_if` from the derived common delivery type
[GIVE-1]."

**Attack.** v0.22's [GIVE-1] gives delivery four structural forms, only
one of which is a `give`: "an arm delivers when its final statement is a
`give_stmt`, a `return_stmt`, a `break_stmt` whose resolved target loop
lexically encloses the same value match, or a `match_stmt` every arm of
which delivers". So a legal value initializer can have **zero**
delivering `give`s:

```
let v = match r {
  Ok(value: w) => { return w; }
  Err(error: e) => { return unit; }
}
```

Every arm delivers (both end in `return_stmt`), so [GIVE-1]'s
give-completeness recursion is satisfied and the construct is accepted
today. Under v0.22, `v`'s mode and type came from the binder annotation.
Under the candidate the delivery set is empty, "the common exact mode
and type of its delivering `give`s" is undefined, and TYPE-5's guarantee
"no two derivations can disagree" is satisfied only vacuously — there is
no derivation. `v` is a binding of no type. The same holds for the
`break` form inside a loop and for a `value_if` both of whose branches
return.

The continuation is unreachable, so nothing *reads* `v` — but [TYPE-6]
still enters `v` as a lexical declaration, [ENT-2] still asks whether it
is a tracked place of a fragment type, and [DIAG-1] still has to
classify it. "Unreachable, so don't care" is not a rule; one
implementation will reject and another will accept.

**Fix.** Make the empty delivery set a hard error citing GIVE-1 at the
`let_stmt` node (mechanical fix: spell the construct as a `match_stmt`
or `if_stmt` and drop the binding). This is a **third** acceptance-set
narrowing and must be listed in §4, which today enumerates exactly two.
Note it is also a *new* rejection of a program v0.22 accepts, so it is
not covered by §4's "the error classes that lived only in deleted bytes
die with them" — it is the opposite direction.

---

## F5 (A) — GIVE-1's blanket generalization contradicts the chained `value_if`

**Candidate text (§3 [GIVE-1]):** "Every 'value match' occurrence
generalizes to the value initializer".

**v0.22 [GIVE-1], one of the occurrences that sweeps up:** "A final
nested `value_match` delivers only to its own inner let and therefore
does not make the outer arm deliver."

**Attack.** Generalized as instructed, that sentence reads "A final
nested value initializer delivers only to its own inner let and
therefore does not make the outer arm deliver." But the candidate's own
`value_if := "if" expr "{" stmt* "}" "else" (value_if | "{" stmt* "}")`
makes the else-position `value_if` of a chain *exactly* a nested value
form that must deliver to the **outer** let — that is what an `else if`
chain is:

```
let grade = if a {
  give 1_i32;
} else if b {
  give 2_i32;
} else {
  give 3_i32;
}
```

The `else if b { … } else { … }` node is a `value_if` nested inside a
`value_if`, and its `give`s deliver to `grade`. The generalized sentence
says they cannot. Meanwhile [GRAM-6]'s flattening mandate makes the
chained form the *only* legal spelling, so the rule as generalized
rejects the only legal spelling of every else-if value ladder — and
SWEEP names the corpus's Bool ladders as A4's stated target.

The root cause is that "every occurrence generalizes" is a sweep
instruction, not a verbatim-anchored replacement (see F11(a)); it cannot
be applied without reading each occurrence for whether the
generalization is intended.

**Fix.** Anchor that sentence individually and restate it as: a final
nested value form bound by its **own** `let` delivers only to that inner
let; an else-position `value_if` of the same chain delivers to the
chain's binding. Equivalently, define the chain as one construct with
n+1 branches rather than as nesting.

---

## F6 (A) — ENT-2 and three ENT-3 sources are outside the modification list and go stale

The candidate lists [ENT-3] as having "Four sites" (S1's origin clause,
S1's establishment sentence, S6's forms, S7's shapes) and does not list
[ENT-2] at all. Both are wrong.

**(a) [ENT-2]'s term-root enumeration is closed and now short by one.**
"a `place` … whose root `pbase` IDENT resolves to any `let_stmt` binding
(**whichever of the three right-hand forms — ordinary, `propagate`, or
`value_match` — the statement selects**)". The batch adds a fourth,
`value_if`. Left unamended, a `value_if`-bound binding is not a term, so
no fact about it can form. This is not merely stale prose — it changes
acceptance under mechanical migration:

```
// v0.22 — accepted, n is a term, facts about n discharge downstream
let n: own u64 = match ilt<u64>(i, m) { True() => { give 1_u64; } False() => { give 0_u64; } }

// printer output under A4 — n is no longer a term; every obligation over n goes underivable
let n = if ilt(i, m) { give 1_u64; } else { give 0_u64; }
```

The respelling is exactly what §5 promises is "zero semantic judgment",
and it silently deletes discharge facts.

**(b) [ENT-3] S5 and S9 quote annotated `let`s.** S5: "for
`let x: own T = lit;`… for `let x: own T = p;`… for
`let y: own Dst = cvt<Src, Dst>(p);`". S9: "For `let x: own T = c[i];`".
A3 deletes those annotations; the source shapes as written match nothing
after migration. S5's third form additionally needs no change to its
`cvt<Src, Dst>` targ (correctly retained), which is worth stating
explicitly since every neighbouring targ dies.

**(c) [ENT-3] S4 quotes a targ'd `len`.** "or a call `len<T>(P)` over
such a place — read as the length term len(P)". The candidate respells
`len<T>(P)` in S6 and in ENT-6 but not in S4.

**Fix.** Add ENT-2 to the modification list (one anchored replacement:
three forms become four) and raise ENT-3's site count from four to
seven. Then re-audit: a grep for `<T>`, `<i32>`, `: own`, and `: &` in
§18 is the whole job.

---

## F7 (A) — GRAM-9 still mandates the annotation A3 deletes

**Candidate text (§3 [GRAM-9]):** one site only — "'Every call argument,
construct field value, and subscript offset is an `atom` [GRAM-5]'
becomes 'Every call argument, construct field value, infix operand, and
subscript offset is an `atom` [GRAM-5]'."

**v0.22 [GRAM-9], the sentence the candidate leaves alone:** "A computed
value is forwarded to another operation only by binding it with a
preceding `let` (**stating its explicit mode and type [TYPE-5]**) and
referencing the binding."

After the batch, [GRAM-4] has no annotation slot and [TYPE-5] says the
binder's mode and type "are derived, never written", while [GRAM-9] —
the rule that *creates* almost every let in an ANF language — still
requires the writer to state them. Two normative rules, direct
contradiction, both cited constantly. A conforming implementation can
cite GRAM-9 to reject every annotation-free forwarding let, which is to
say every let in the migrated corpus.

**Fix.** Second GRAM-9 site: delete the parenthetical, or replace it
with "(whose mode and type are derived [TYPE-5])". The rule count stays
22; the site count rises by one.

---

## F8 (A) — DIAG-1's OP-2 diagnostic locations are all `call` nodes that infix operations no longer have

**Candidate text (§3 [DIAG-1]):** "The typed-call location sentence 'a
missing explicit type argument uses `SourceNode` at the `call` node and
that node's complete source extent' **is scoped to the callee classes
that still carry type arguments**; for a value-typed table operation the
class is unreachable and the operand-type error follows OP-2's rewritten
second-operand attribution."

**v0.22 [DIAG-1], the full paragraph:** "For a typed call to an [OP-2]
operation, a missing explicit type argument uses `SourceNode` at the
`call` node and that node's complete source extent. **A wrong
type-argument kind, count, or domain, or a missing operand, uses the
same call location.** An extra operand or every wrong exact operand type
other than the TYPE-7 implicit-read case uses `SourceNode` at the first
offending `atom` node in source order and that atom's complete extent.
The cited rule is the rule selected by [OP-2]: FN-2, OP-1, or TYPE-5."

**Attack (a).** The paragraph is scoped "for a typed call to an [OP-2]
operation", and after A1 **no** [OP-2] operation carries a type
argument. "Scoped to the callee classes that still carry type arguments"
therefore scopes it to the empty set: the first sentence becomes dead
text, and the *second* sentence — which locates OP-1's wrong-domain and
missing-operand errors — is silently orphaned along with it, because
"the same call location" no longer exists for an infix operation. Write
`a + b` with `a : own f64`: [OP-2]'s replacement says this "cites
[OP-1]", and DIAG-1 has no `SourceNode` to offer. Every operand-domain
diagnostic on the twenty respelled operations loses its location.

**Attack (b).** Attribution row 2 gains "an `infix` operand" but not the
operator tokens: "the two actual tokens at the start of that occurrence
are `(IDENT, "(")`, `(IDENT, "<")`, `(OPNAME, "(")`, `(OPNAME, "<")`,
`(TYPEID, "(")`, or `(TYPEID, "<")`, the rejection cites [GRAM-9]". The
single most likely new writer error is nested infix:

```
let x = a + b * c;
```

`b * c` is not an `atom`, so this must be rejected under GRAM-9 — but
the second operand *starts* at `(IDENT, "*")`, which is not in row 2's
token list, so the parse fails at `*` with no attributed rule. Adding an
infix surface without teaching the ANF-attribution row about operator
tokens leaves the new construct's characteristic error unattributed.

**Fix.** (a) Replace the paragraph, do not scope it: keep the call-node
locations for the callee classes that genuinely retain arguments (user
generics [FN-2], SYS-2 region arguments, and F3's corrected retained-targ
set) and add a parallel sentence locating operand-domain and
operand-count errors at the `infix` node, with wrong-operand-type at the
second operand atom per OP-2. (b) Extend row 2's token list with the
twenty operator tokens for occurrences in infix-operand position.

---

## F9 (A) — FORM-2 double-governs `if` rendering, so canonical bytes are not uniquely determined

**Candidate text (§3 [FORM-2]):** the block-bearing list "gains
`if_stmt` and `value_if` **and their then/else blocks**", plus a new
appended sentence: "An `if` renders its introducer through `{` on one
line; an `else` renders as the join line `} else {`…".

**v0.22 [FORM-2], the rule that still applies to every list member:**
"The block-bearing productions are … Their introducer through `{` is one
line; their children render on following lines at depth plus one; **and
`}` renders on its own line at the original depth**."

**Attack (a) — two rules, two answers.** With `if_stmt` in the
block-bearing list, the generic sentence renders its `}` on its own line
at the original depth; the new sentence renders it as `} else {`. Both
are normative, neither is stated to override the other, and [FORM-1]
requires input bytes to equal *the* rendering. There is no unique
rendering, so either no `if`/`else` program is canonical or two
implementations disagree. The precedent the candidate is reaching for —
`requires {` … `} {` — works because FORM-2 states it in a dedicated
paragraph *and* because `fn_decl`'s body and `requires_block` are two
separate productions; here one production owns two or more brace blocks,
which is new in the grammar.

**Attack (b) — the list holds productions, and then/else blocks are not
productions.** The block-bearing list names `struct_decl`, `enum_decl`,
… `arm`, plus the phrase "the body of `fn_decl`". `if_stmt`'s branches
are inline `"{" stmt* "}"` occurrences with no production name; "their
then/else blocks" cannot be added to a list of productions without
inventing two productions (cost: +2, contradicting the +4 accounting).

**Attack (c) — the chain nesting.** In `} else if c {`, the
else-position `if_stmt` is itself block-bearing, so the generic sentence
puts its "introducer through `{`" on its own line at its own depth,
which is the deep-nesting layout A4 exists to kill.

**Fix.** State the override explicitly: `if_stmt` and `value_if` are
block-bearing *and* their rendering is governed solely by the new
sentence, which must then be complete — introducer line, then-children
at depth+1, join line at the original depth, else-children at depth+1,
final `}` at the original depth, and the else-if chain rendering the
nested `if` on the join line at the original depth. Drop "and their
then/else blocks".

---

## F10 (A) — OP-7 and OP-8 keep normative sentences about the explicit type argument A1 deletes

**Candidate text:** "**[OP-7]** One sentence appended…"; "**[OP-8]**
Spelling mentions only: `iadd.sat`/`isub.sat` read `+sat`/`-sat`,
`imul.sat` reads `*sat`, and the shift/rotate/abs/float sentences are
unchanged (those operations keep named spellings)."

Both understate the damage. Sentences left standing:

- [OP-7]: "Signedness-parametric lowering **keyed on the explicit type
  argument** (`ishr` is `ashr` for signed T and `lshr` for unsigned T;
  `imin` is `smin` or `umin`)…" and "Nominal enum identity is likewise
  **checked from the explicit type argument** before `eeq`/`ene`
  lowering".
- [OP-8]: "`eeq<T>(a, b)` is `True()` exactly when…", "`ene<T>(a, b)` is
  its exact boolean complement", "Both operands **and the explicit type
  argument** must have that exact T", and "negative infinity is
  `fneg(finf<T>())`".

`ishr`, `imin`, `eeq`, and `ene` all keep their *names* but all lose
their *type arguments* under A1's "every value-typed table operation"
(22 `eeq<` and 15 `ene<` sites in the corpus), so these are not spelling
mentions — they are lowering and identity rules whose stated input no
longer exists. The `eeq`/`ene` selected type is genuinely operand-derived
and the rules survive with rewording; the point is that the candidate
declares them untouched.

**Fix.** Add OP-7 (two more sites) and OP-8 (three more sites) properly:
"keyed on the explicit type argument" becomes "keyed on the
operand-derived selected type", and OP-8's `eeq<T>`/`ene<T>`/`finf<T>`
spellings resolve per F3's corrected retained-targ set.

---

## F11 (E) — Editorial batch

**(a) The "thirty-two verbatim-anchored modification sites" count is not
reproducible, and at least four "sites" are not anchored.** The
candidate defines "a site is one contiguous verbatim-anchored
replacement", then uses prose sweeps for [GIVE-1] ("Every 'value match'
occurrence generalizes to the value initializer" — the direct cause of
F5), [OP-8] ("Spelling mentions only" — F10), [DIAG-1] ("is scoped to
the callee classes that still carry type arguments" — F8), and [OP-2]'s
negation paragraph ("takes the same derivation"). Summing the
per-rule counts the document itself states gives 33, not 32, and the
prose sweeps have no determinate site count at all. A batch destined for
exact-byte owner approval must be applicable mechanically from its own
deltas; this one is not.

**(b) The [ERR-3] replacement garbles the sentence.** v0.22 reads
"Propagation: `let x: own T = propagate e;` requires `e : own
Result<T, E>` **and the enclosing function's return type `own
Result<U, E>` (same E — no conversions, TYPE-4)**." The candidate's
replacement ends "…requires `e : own Result<T, E>`, and x's derived mode
and type are `own T` [TYPE-5]", with "the rest byte-identical" — so the
return-type requirement now dangles off "x's derived mode and type are…
and the enclosing function's return type…". Re-anchor on the full
sentence and append the derived-type clause after it.

**(c) [GRAM-6]'s flattening mandate orders an ungrammatical fix inside a
`value_if`.** "an `else` whose block contains exactly one `if_stmt` and
nothing else is a hard error citing GRAM-6 (spell `else if`)". In a
`value_if` whose else block is exactly one *else-free* `if_stmt`, the
prescribed fix is unspellable: `value_if`'s else alternative is
`value_if | "{" stmt* "}"` and `value_if` requires its own `else`. The
program is rejected either way (GIVE-1 also refuses, since an else-free
`if_stmt` never delivers), so acceptance is unchanged — but the
mechanical fix is unfollowable. Scope the flattening mandate to a nested
`if_stmt` that has an `else`, or to `if_stmt` else-blocks only.

**(d) Empty-`else` is double-cited.** `let v = if c { give 1_i32; }
else { }` is rejected by GRAM-6 (empty else) and by GIVE-1 (the else
does not deliver). [META-4] wants one statement of a fact; say which
rule owns it, or scope GRAM-6's empty-else rule to `if_stmt`, leaving
`value_if`'s to GIVE-1.

**(e) [FN-8]'s exclusion list is now incomplete.** "a `propagate_let_rhs`,
a `value_match`, or any other direct statement shape is a hard error
citing FN-8" — the catch-all covers `value_if` and `if_stmt` correctly,
but the enumeration should name `value_if` beside `value_match` since it
names the sibling.

**(f) The R3-register settlement for TYPE-5 is thinner than the register
asks for.** The register (v0.22 line 36) says its entries "were
minimality-selected, not evidence-selected, and require validation
before ratification", and flags this one specifically: "interior
annotation mandate (TYPE-5 — **round-2 verdict still needs_evidence**)".
The candidate settles it with SWEEP's T1 argument. That argument is
sound, but it is a design rule, not the writer/codegen comparison the
register's own wording demands — the same currency the register asks for
in the FN-8 entry ("semantics selected, spelling not yet compared"). The
other two settlements are better grounded: "match-only conditionals and
no-if" rests on a redundancy proof (the two arm labels are always
exactly `True()`/`False()` in fixed order), and "prefix arithmetic
surface" rests on SWEEP's objective tiebreaks, which is exactly R3's
stated currency. Recommend the owner rule explicitly on whether a T1
argument discharges a `needs_evidence` register entry, since this batch
sets the precedent for the remaining eleven.

**(g) Migration counts verify; one does not.** Measured independently
over `tests/programs` + `tests/conformance/cases` (399 `.wf` files,
excluding the `.claude/worktrees/` mirror, which doubles every naive
count): let annotations **1748** (claimed 1748 ✓), `True()` arms
**257** (claimed 257 ✓), `ilt`/`igt` sites **56** (claimed 56 ✓),
`iadd.wrap` **229** (claimed 229 ✓), `iadd.trap` **47** (claimed 47 ✓),
`check` statements 409 by a loose regex against a claimed 404 (the
difference is `check` inside `doc`/message strings — consistent). Only
the 1260 value-op targ deletions did not reproduce; see F3.

---

## Verified — axes and attacks that survived

**T3 uniqueness — the hard axis. Survived every attack except through
F4/F5.**

- **Else-if flattening vs. block nesting.** `else { if d { … } else { … } }`
  and `else if d { … } else { … }` are the same program; GRAM-6's
  flattening mandate kills the first. The mandate reaches inside a
  `value_if`'s else too (it is an `else` with a block), so both the
  statement and the value ladder have exactly one spelling. Checked the
  else-free nested case, the "if plus one more statement" case (block
  form correctly required, and the flattened alternative would change
  semantics, so no dual spelling), and deep chains.
- **`value_if` vs `value_match`.** They never compete: GRAM-6 makes the
  scrutinee/condition type decide, one form per class, and no type has
  both. Parser-side they are disjoint at one token after `let IDENT =`
  (`propagate` / `match` / `if` / else `expr`), and `if` is an exact
  fixed atom so [FORM-3] excludes it from IDENT — no `expr` can begin
  with it.
- **Infix vs named call for the four respelled comparisons.** Confirmed
  bare `<`/`>` were **not** drafted, so no dual spelling exists there.
  Confirmed the respelled four cannot be written both ways: once `ieq`
  `ine` `ile` `ige` leave the [OP-1] op column they leave the derived
  `DotlessOperationNames` automatically (the set is derived as "spellings
  … whose complete spelling satisfies IDENT and contains no dot", and
  `==` does not satisfy IDENT), so `ieq(a, b)` falls through to
  "every other IDENT callee admits a top-level source `fn_decl` or an
  admitted system operation" and, absent such a declaration, is a hard
  error citing OP-1. One spelling each. The same derivation makes the
  `DotlessOperationNames`/`ReservedLowerNames` shrink self-maintaining —
  no extra site needed, and the candidate is right to claim it.
- **Operator maximal munch.** Worked through the adversarial cases: `a -
  -1_i32` (operator then negative literal — round-trips, both tokens
  distinct, canonical bytes stable); `a - 1_u64` vs `-1_u64` (the
  candidate's own claim, confirmed against GRAM-1's unchanged numeric-form
  clause); `a+1_i32` and `a +-1_i32` (both lex and parse, both rejected
  as non-canonical under FORM-1 — a rejection, not a second spelling);
  `a-1_i32` (lexes as two atoms and fails to parse — see below); `->`
  preserved by the explicit `-`-followed-by-`>` carve-out; `/` safe
  because [FORM-4] has no comments; `!` correctly left as a raw lexical
  defect outside `!=`, since GRAM-1's single-byte punctuation list does
  not contain it and the candidate only extends the compound sentence.
- **Suffix words cannot collide with identifiers.** `a -wrap b` versus
  `a - wrap b` is not a hazard: `wrap`, `checked`, and `sat` are in
  `ModeWords` and therefore `ReservedLowerNames`, so [OP-1]'s reservation
  makes them unwritable as any binding, parameter, field, or function
  name. No place expression can ever spell them.
- **FORM-2 attachment.** Verified directly against the sets: `=` is in
  neither attachment set, so `= =` never renders adjacent and `==` cannot
  arise by attachment; `>` and `<` are in the sets but as *single-byte
  terminals*, and `<=`/`>=` are distinct compound terminals that are not
  members, so they always render spaced; `const c: array<u8, 4> = […]`
  renders `> =`, not `>=`, because `=` is in neither set. Also checked
  `a[i] + b`, `a.f + b`, `a + b[i]`, `&'r p + q` — all render with the
  operator spaced on both sides, as the candidate claims.
- **Annotation-free lets.** Walked every right-hand form for a
  second reading: literal (own T, suffix mandatory), generic numeric
  `0_T`, bare copy place, `move place`, `borrow_expr`, `deref(p)` (copy
  of the referent under TYPE-7), call (FN-1/SYS-2 `rtype`), construct
  (`own K`), `propagate` (own T from `Result<T, E>`). Each produces one
  mode and one type; none admits two readings. TYPE-5's "no binder's
  type depends on a later statement" survives the value forms on a
  technicality worth naming: the delivering `give`s are inside the *same*
  `let_stmt`, so the derivation stays statement-local even though it now
  reads an arbitrarily deep subtree.
- **`a-1_i32` diagnostic.** Lexes as `a` then numeric `-1_i32` (the
  numeric-form clause is unchanged and wins on `-`+digit), giving two
  adjacent atoms and a parse failure rather than "spell it `a - 1_i32`".
  Correct behaviour, poor message; worth a DIAG-1 hint row eventually,
  not a batch blocker.

**T4 globality — survived.** The retained-targ rule keys on callee class,
not use site (subject to F3 making the class total). The empty-`else` and
flattening rules key on block content, which is syntactic structure, not
context or inference. The one rule that looked like a violation is
GRAM-6's type-driven conditional class: the scrutinee's type is not a
grammar class, and T4's letter says "grammar class … never on use-site
context". I attacked it through generics — if a `match` scrutinee could
have symbolic type T, legality would flip per [FN-2] instantiation — and
it is unreachable: `arm := TYPEID "(" fieldbind_list? ")" "=>"` names
variant constructors, which no symbolic T supplies, so a `match` on a
generic-typed scrutinee cannot be written. Type-driven here is a total
function of a fact the checker always has, and it does not depend on
inference succeeding. It passes, but the owner should note the precedent
being set: this is the first class rule in the language keyed on a
checked type rather than a grammar shape.

**Semantic drift — survived.** Bare `+ - * / %` map row-for-row onto
`iadd.trap` `isub.trap` `imul.trap` `idiv.trap` `irem.trap`, whose
[OP-2] semantics, [ERR-4] contract-violation classification, and
[EFF-2] `traps` contribution are untouched; the EFF-2 replacement
correctly keeps both the bare-operator and the surviving `.trap` OPNAME
carriers (`ineg.trap`, `iabs.trap`, `ishl.trap`, `ishr.trap`), and
`buffer_new`'s `traps` still arrives through "or a call". [OP-2]'s
operand-derived selected type cannot change which row resolves: the
family is chosen by the operator token alone ("infix resolution consults
no name domain"), and within the family the row is chosen by the
selected type, which under both v0.22 and the candidate must equal both
operands' exact type — the written targ was already required to match
them, so the derived value is the same integer type at every site where
the program was accepted before. Where they would differ, the program
was already rejected. [DIAG-3]'s trap record changes `node_path` from
`call` to `infix` and nothing else, which is a location change, not a
semantics change. Accounting checks: the 20 respelled op-column
spellings (3 ops × 4 modes, 2 ops × 2 modes, 4 comparisons) equal the 20
new terminal spellings (5 bare + 11 suffixed + 4 compound) and equal the
20 alternatives in the drafted `infix_op` list — three independent
counts, all consistent. Baseline 65 productions verified independently
against both the document and task 0030's verifier figure.

**Deferral reasoning for C3 (O2) — sound.** All three grounds check out
against v0.22: [CLM-1] does require a per-`fn_decl`-unique name and
[DIAG-3]'s record carries it where `check`'s carries the STRING, so trap
bytes genuinely change; [CLM-2] refutation genuinely converts some
accepted `check`s into hard errors, which no respelling may smuggle; and
[FN-8]'s structural pass genuinely requires "exactly one final
`check_stmt`", so unifying forces an FN-8 decision. Keep C3 out.

---

## Residue hunt (standing axis)

**R1 — two mode-suffix mechanisms now carry the same three words.**
After C1, `wrap`/`checked`/`sat` are spelled two ways: dot-separated on
the operations that keep names (`ineg.wrap`, `ishl.trap`, `iabs.checked`)
and glued to an operator on the ones that do not (`+wrap`, `/checked`,
`*sat`). Two lexical rules form them ([FORM-3]'s OPNAME clause and the
new operator-form clause), and the reservation story runs through only
one of them (`ModeWords` is defined as "the suffix alternatives in
FORM-3's active OPNAME formation rule", which is now a proper subset of
where mode words appear). Re-derivable from kernel principles? Only
half — the *concept* is [OP-7]'s mode axis, which is principled; the
*duplication* is an artifact of respelling some rows and not others.
Proliferation-bounded? Yes, at two. One mechanism per concern? No. The
honest reading is that this residue is temporary by construction: O6
already tracks `.trap`/`.checked` dissolution, and the candidate is
right that "when it lands, bare `+` is already the spelling it needs".
Recommend recording it as a named, dated debt with O6 as its discharge
condition rather than leaving it implicit — and adding one sentence to
[OP-1] making `ModeWords` derived from both carriers, so the reservation
set does not quietly depend on which rows happen to be respelled.

**R2 — the comparison family is split by a lexer accident, and the split
is writer-visible.** `==` `!=` `<=` `>=` are infix; `ilt` `igt` are named
calls. The boundary carries no semantic content: all six are one [OP-1]
row, one signature, one effect. [OP-7] exists to make operation names
"W1-predictable", and a writer now cannot predict from the operation
which spelling it takes. The candidate sees this ("the asymmetry against
`<=`/`>=` is visible in EX-1's first branch" — and it is: EX-1's
`sign_of` reads `if ilt(x, 0_i32) { … } else if x == 0_i32 {`, two
spellings of comparison in one function). The split also propagates into
the reservation set: `ieq`/`ine`/`ile`/`ige` become writer-reusable
names while `ilt`/`igt` stay reserved, so a program may now declare
`fn ieq(…)` but not `fn ilt(…)`. Imported habit? No — it is the opposite,
a native constraint. But it is the batch's one piece of surface whose
shape is explained by the implementation rather than the language, which
is exactly what the axis hunts for. See O1.

**R3 — the derived common delivery type is real machinery, and it is the
batch's only addition.** The candidate flags this itself (O4) and is
right to: A1/A3/A4 are deletions, C1 is a constant swap, and this one
rule is new normalization. It earns its place — deleting the binder
annotation forces *something* to type the binding — but it is worth
recording that its cost is not the one-line agreement rule as drafted.
It is the rule plus its completeness obligation (F4's empty set), plus
[ENT-2]'s term enumeration (F6a), plus the nested-delivery interaction
(F5). Three of this review's nine A findings trace to it. Proportionate
to A3's 1748 deleted annotations — yes, comfortably. But it should ship
as a fully-worked rule, not a sentence.

---

## O1–O7 — recommendations

**O1 (bare `<`/`>` infix) — recommend (a) as drafted for this batch,
with a correction to the record and a named revisit condition.** The
exclusion is forced: `(IDENT, "<")` cannot select between `a < b` and a
generic call at two tokens, and this is not speculative — [DIAG-1]'s
attribution row 2 already lists `(IDENT, "<")` as a GRAM-9 signal, so
the pair is load-bearing in the current design. But **the candidate
mis-scopes option (b)**. It calls (b) "a breaking canonical change on
every generic call **and type**"; in fact type-level targs never
compete with a comparison, because `type := TYPEID targs?` and
`construct := TYPEID targs?` both begin with TYPEID, and TYPEID is not
an `atom` (`atom := literal | "move" place | place | borrow_expr`,
`pbase := IDENT | "deref" …`). Nor does `(OPNAME, "<")` compete. After
A1 deletes value-op targs, the *entire* remaining collision surface is
`(IDENT, "<")` on user-generic `fn` calls and SYS-2 region arguments —
a far smaller change than "every generic call and type", and one a
call-targs-only introducer (a turbofish-shaped compound token) would
close without touching a single type. That does not make (b) right for
this batch, but the owner should rule on (a) knowing (b) costs a
fraction of what the candidate states. Revisit condition: R2's
predictability cost, measured, or the next batch that touches call
syntax for another reason.

**O2 (C3 deferral) — confirm as drafted.** Reasoning verified above
against all three grounds; nothing to add.

**O3 (requires-clause lets) — recommend the uniform annotation-free
reading, as drafted.** T4 is decisive here: a split production would
make annotation legality depend on which block a `let` sits in, which is
a grammar class, so it would technically pass T4 — but SWEEP's B-row
argument is T2 ("the interface is the trust boundary"), and the
boundary fact in a requires block is the final `check` condition, not
the scaffolding lets that compute it. [FN-8] already treats those lets
as a mechanical prologue subset ("zero or more `let_stmt` nodes whose
selected right-hand side is `ordinary_let_rhs`, followed by exactly one
final `check_stmt`"), and their right-hand sides are table-op-typed and
therefore self-typed. One `let_stmt` production, one rule. Also cheaper:
the alternative adds a production and a second `let` node kind.

**O4 (derived common delivery type) — confirm the rule, but not as
drafted.** The agreement construction is right and is genuinely T3-unique
(agreement over a closed set is not a join and admits no widening). It
must ship with the empty-delivery-set case closed (F4), the nested-value
interaction restated (F5), and [ENT-2]'s term roots extended (F6a).

**O5 (`=[` cvalue attachment) — recommend closing it as standing, as
drafted.** Verified independently: `=` is in neither FORM-2 attachment
set, so it always renders spaced and no infix operator can become
adjacent to it; `==` cannot arise from two `=` terminals; and the only
`=`-adjacency in the language remains `=[` from `[` being in the
right-attachment set, which C1 does not touch. The candidate's reasoning
is correct and the conclusion holds.

**O6 (another-batch items) — confirm as restated.** One addition:
R1 should be attached to the `.trap`/`.checked` dissolution item as its
explicit discharge condition, so the two-mode-suffix-mechanism debt has
a named owner rather than living only in this review.

**O7 (empty then-block with non-empty `else`) — confirm the admission,
and state it in ERR-2.** The reasoning holds: the inverted spelling
`if bnot(c) { B }` is a different checked program (it contains an extra
operation, and under ANF that operation is a real node with a real
effect row), so there is no dual spelling to kill and no T3 rule is
owed. Note the asymmetry is already stated for the other direction
(empty `else` rejected, spell the else-free form), so the pair of rules
is complete and content-driven. Worth one clause in the [ERR-2] addition
so a reader does not infer symmetry: the empty *then* is admitted, the
empty *else* is not, and both follow from "the else-free form is the
one spelling of the empty alternative".

---

## Severity counts

| severity | count | ids |
|---|---|---|
| S | 1 | F1 |
| A | 9 | F2, F3, F4, F5, F6, F7, F8, F9, F10 |
| E | 7 | F11(a)–(g) |
| residue | 3 | R1, R2, R3 |

Must-fix before approval: **F1** (ENT-5 join), **F2** (left-factor
`expr`; the current draft's own EX-1 does not parse), **F3** (retained-targ
class is not total — two `finf`/`fnan`, three `arena_new`, and twelve
`array_new` corpus sites have no legal spelling), **F4** (empty delivery
set). F5–F10 are corrections to rules the candidate declares untouched
or scopes by prose; each is a one-anchor fix but each is a live
contradiction if it ships. F2 additionally changes the accounting the
owner would be approving: 70 productions, not 69.

What the batch got right, since the finding list is long: the migration
measurements are honest (five of six reproduce exactly), the anchor
craft is byte-exact where anchors exist, the token and op-column
accounting is internally consistent across three independent counts, the
production baseline of 65 is correct, the two structural findings the
candidate surfaces up front are both real and both correctly diagnosed,
C3's deferral is correctly reasoned, and the T3 uniqueness argument for
the else-if chain — the part most likely to hide a second spelling —
holds under every attack I could construct.

---

## Re-verify (2026-08-07, targeted; revised candidate at f078247, 787 lines, 24 rules / 44 sites)

Scope: the five axes the lead named, not a full re-review. Every anchor
newly quoted by the revision was checked verbatim against
`spec/kernel-spec-v0.22.md` — all eight I spot-checked resolve
(ENT-5's loop-join sentence, ENT-2's three-form parenthetical, GRAM-9's
forwarding parenthetical, OP-1's `ModeWords` definition, OP-1's
row-selection sentence, OP-7's signedness sentence, FN-8's exclusion
list, DIAG-1 attribution row 2). Corpus re-measured over the same 399
files.

Result: **8 of 11 findings CLOSED, 1 STILL-OPEN, 1 CLOSED-with-regression,
1 CLOSED-with-nit; 4 new findings (2 A, 2 E).** Recommendation at the
end: one more short round.

### F1 (S) — CLOSED

The new [ENT-5] site inserts, before the loop-join sentence: "At the
continuation of an `if_stmt` or `value_if`, the fact state is the join
of the then-block exit edge and the else-block exit edge — or, for an
else-free `if_stmt`, the false edge — each taken after that edge's
scope-exit kills and then closed [ENT-4]; a branch every path of which
leaves by `return`, `break` to an enclosing loop, or `propagate`'s error
edge contributes nothing there."

Re-attacked, all six variants, and the outcome is single and sound in
each:

- **Original attack** (`let n = len(b); if ilt(i, n) { let seen = b[i]; }
  let x = b[i];`). The then exit carries `i - n <= -1`; the false edge
  carries `n - i <= 0` and, for the pair (i, n), only the trivial
  implicit bound `i - Z <= max(u64)` composed through Z. The join keeps
  "the weakest (largest-constant) bound held by all joined states", so
  the useful bound is dropped, `i - len(b) <= -1` is underivable, and
  the second subscript is rejected. Exactly what the `match` spelling
  does today. The deletion path is closed.
- **Kill inside one branch.** `claim g: ilt(i, n) …;` before the `if`,
  then `set i = 99_u64;` in the then-block: kill (a) removes the fact on
  the then exit, the false edge still holds it, the join drops it,
  `b[i]` rejects. Sound — the join cannot be bypassed by killing on one
  side only.
- **Both branches establish.** A `claim` of the same relation in each
  branch: both exits hold `i - n <= -1`, the join keeps it, `b[i]`
  discharges. Correct and not over-conservative — both paths executed a
  runtime check.
- **One branch leaves.** `if ilt(i, n) { … } else { return 0_u8; }`: the
  else contributes nothing, the join is over the then exit alone, and
  `i < n` survives to the continuation. Sound (the continuation is
  reachable only through the branch that established it), and identical
  to today's match behaviour. The inverse — else-free `if` whose then
  returns — leaves only the false edge, carrying the negation, and
  correctly rejects.
- **`if` inside a loop body.** [ENT-5]'s loop rule already quantifies
  over kills "occurring anywhere inside the loop body, at any nesting
  depth", so branch kills are covered, and "no fact established inside
  an iteration survives to the next iteration's head" covers branch
  establishment. No new gap.
- **`value_if` arms.** Covered by the sentence's `value_if` mention. The
  bound value itself gains no [ENT-3] fact (S5 covers `ordinary_let_rhs`
  only), exactly as for `value_match` today — not a regression.

Two editorial residues, neither soundness-bearing:

**(i) The chain case is phrased over a construct a chain does not have.**
In `if A { } else if B { } else { }` the outer `if`'s else alternative is
an `if_stmt`, not a block, so "the else-block exit edge" names nothing.
Both available readings — compose the nested joins, or treat every branch
exit of the chain as one join — give the same state, because the join is
pointwise max over bound constants plus intersection of disequalities,
hence associative, commutative, and idempotent, and [ENT-5] already
guarantees "the join of closed states is closed", so no re-closure step
can perturb the composition. No divergence, so not a finding. But the
existing match sentence is phrased over the CFG ("the join of the states
on every arm exit edge reaching that continuation on the conservative
structural graph [FN-1]"), and borrowing that idiom — "every branch exit
edge reaching that continuation" — would cover chains, else-free `if`s,
and nesting with no enumeration and no case analysis. Recommend the
reword.

**(ii) The empty join is undefined — inherited, not introduced.** For an
`if` both of whose branches `return`, every branch contributes nothing
and the join is over zero edges. [ENT-5] states the empty case only for
loops ("A `loop_stmt` with no `break` naming its label has an empty join:
its continuation state is the contradictory all-derivable state").
Checked v0.22: the `match_stmt` sentence has exactly the same hole (a
match all of whose arms return), so this is pre-existing and the batch
does not widen it. It is a free fix while the sentence is open — one
clause, "an empty join is the contradictory all-derivable state, as for
a break-free loop" — and closing it for `if` and `match` together would
retire the gap rather than duplicate it.

### F2 (A) — CLOSED, with one new finding

The grammar is now `expr := atom infix_tail? | call | construct`,
`infix := atom infix_tail`, `infix_tail := infix_op atom`. I re-derived
the drafter's six recorded decisions independently rather than accepting
them:

- **`expr` three-way.** The atom-headed arm's IDENT-headed two-token
  pairs are exactly (IDENT, X) for X in {`.`, `[`} ∪ the twenty operator
  tokens ∪ FOLLOW(`expr`). I computed FOLLOW(`expr`) from every
  occurrence of `expr` in the grammar — `ordinary_let_rhs`,
  `propagate_let_rhs`, `return_stmt`, `set_stmt` (all `;`), `check_stmt`
  (`else`), `match_stmt`/`value_match`/`if_stmt`/`value_if` (`{`) —
  giving exactly {`;`, `{`, `else`}, which is the drafter's set and is
  complete. Neither `(` nor `<` is in it, `<` is deliberately not an
  `infix_op`, and the non-IDENT atom heads are (`move`, IDENT),
  (`deref`, `(`), (`&`, REGIONID), and literal tokens — `move` and
  `deref` are exact fixed atoms and therefore excluded from IDENT by
  [FORM-3]. Disjoint from the call arm {(IDENT,`(`), (IDENT,`<`),
  (OPNAME,`(`), (OPNAME,`<`)} and the construct arm {(TYPEID,`(`),
  (TYPEID,`<`)}. **Verified.**
- **`infix_tail?`.** Take-set is FIRST(`infix_tail`) = the twenty
  operator tokens; skip-set is FOLLOW(`expr`) = {`;`, `{`, `else`}.
  Disjoint at one token. **Verified.**
- **`psuffix*`.** Consume on `.`/`[`; I checked global FOLLOW(`place`)
  for both bytes and neither can follow a complete place anywhere in the
  grammar (`.` appears only in `psuffix`, `input_label`, and inside the
  single OPNAME token; `[` only in `psuffix` and `region_params`, which
  never follows a place). Strong-LL(1)-clean. **Verified.**
- **let-RHS four-way.** `propagate` / `match` / `if` are exact fixed
  atoms excluded from IDENT, and none is in FIRST(`expr`) = {literals,
  `move`, `deref`, `&`, IDENT, OPNAME, TYPEID}. **Verified.**
- **`else?` and the dangling else.** FOLLOW(`if_stmt`) is statement
  starts ∪ {`}`} — `else` occurs only inside the `if_stmt` production, so
  it is not a statement start and not in FOLLOW. The optional is
  therefore decidable at one token and always takes. Mandatory braces do
  the rest: a nested `if` in a then-block reaches its own `else?`
  decision facing `}`. A chain's trailing `else` binds innermost, which
  is the intended chain semantics. **Verified.**
- **`else` alternative** (`if` vs `{`). Trivially disjoint. **Verified.**

My two minimal witnesses now parse with one decision each: `let x = a.f;`
takes `atom infix_tail?` with the tail skipped at `;`, and
`let x = a.f + 1_i32;` takes the same arm with the tail consumed at `+`
— the shared prefix is decided once, after the atom, not at token two.
Both critical [EX-1] positions also trace: `match deref(p) +checked
2_i32 {` selects the atom arm on (`deref`, `(`), exits `psuffix*` at
`+checked`, consumes the tail, and returns to `value_match` at `{`;
`if ilt(x, 0_i32) {` selects the call arm on (IDENT, `(`), skips
`targs?` at `(`, and skips `infix_tail?` at `{`. The LL(2) defect is
closed and the withdrawn claim is correctly withdrawn in §2.

**NEW-1 (E) — the node-kind repair leaves one unreachable production and
one node-less production, and the stated total of 70 is at risk.**
`infix := atom infix_tail` appears on no right-hand side and is not the
start symbol, so no derivation ever produces it; meanwhile `infix_tail`
is reachable but, by the added [GRAM-1] sentence ("a selected
`infix_tail` reduces the atom and tail to one `infix` node"), maps to no
node kind. That inverts [GRAM-1]'s "Every production maps 1:1 to one
core-tree node kind" rather than preserving it. In practice the existing
grammar already treats `expr` as a production whose node kind is the
selected arm's, so the honest formulation is that the `expr` arm with a
present tail *is* the `infix` node and `infix_tail` is a factoring
artifact — in which case the grammar the verifier will check has **69**
productions, not the 70 the version-header paragraph states. Since task
0030 recorded the verifier count as an explicit success criterion ("the
native grammar verifier accepts the candidate grammar with 65
productions"), a 70-vs-69 mismatch will surface at the grammar-path task
against a number the owner has already approved byte-exactly. Pin it
before then: either drop the phantom `infix` production and state the
node mapping in [GRAM-1] prose (total 69), or keep it and say
explicitly that `infix_tail` introduces no node — noting [META-3]
forbids exception clauses, which argues for the former.

### F3 (A) — CLOSED

The retained-argument class is now enumerated: `cvt` and `reinterpret`
(type pairs), `array_new` (element type and const length), `arena_new`
(region and element type), `finf`/`fnan` (result type), with "Every
other table operation carries no written argument and derives its
selected type from its operands [OP-2]; a written argument there is a
hard error citing OP-1."

I walked the complete [OP-1] table row by row rather than trusting the
enumeration — all forty-odd row groups, asking of each whether every
component of the result type is fixed by the operands. Everything else
derives: the integer and float arithmetic/comparison families and
`imin`/`imax`/`imulhi`/bitwise/`iabs` from operand 1 (or the agreement
of both); the shifts from operand 1 with the `u32` amount fixed by the
row; `eeq`/`ene` from the operands' nominal type; `band`/`bor`/`bxor`/
`bnot` monomorphic in `Bool`; `len`, `box_new`, `buffer_new` from their
operands; and `slice_of` from both its operand's type **and** its
region, because the operand is a `borrow_expr` that carries the
REGIONID syntactically (`slice_of(&'r P)`), which is the one row where I
expected a second `arena_new`-shaped hole and did not find one. The six
retained are exactly the six that cannot derive. **The enumeration is
total.**

Migration spot-check: retained-class sites reproduce **exactly** —
`cvt<` 78, `reinterpret<` 6, `array_new<` 12, `arena_new<` 3, `finf<` 1,
`fnan<` 1 = **101**, the candidate's figure. Deleted-class sites: my
regex over every non-retained table-operation spelling gives **1356**
against the claimed 1357; a one-site gap, almost certainly a
line-wrapped call my line-oriented pattern misses. Worth the drafter
reconciling, not worth blocking.

**NEW-2 (A) — [OP-1]'s row-selection sentence still keys on the deleted
written argument.** v0.22 [OP-1], verbatim and untouched by all three of
the candidate's OP-1 sites: "Later typed operation checking uses **the
written type arguments** and operand domains to select the applicable
row within the resolved family." This is the rule that actually performs
row selection — which `cvt` row applies, whether `ishr` lowers to `ashr`
or `lshr`, which integer width an `iadd` family member takes. After A1
the written type arguments exist for exactly six operations, so for
every other family the selection procedure names an input that is no
longer there. This is the same defect class the drafter correctly fixed
in [OP-7] and [OP-8] under F10; it survives one rule upstream, in the
rule those two cross-reference. Fix: a fourth OP-1 site — "Later typed
operation checking uses the operand domains and, for the
retained-argument operations [TYPE-5], the written arguments, to select
the applicable row within the resolved family."

### F4 / F5 / O4 — CLOSED

The complete [GIVE-1] replacement was re-attacked on the delivery
construction:

- **Empty set.** Now "A value initializer whose delivery set is empty …
  is a hard error citing GIVE-1 at the `let_stmt` node", with the
  statement-form fix. My witness (`let v = match r { Ok => { return w; }
  Err => { return unit; } }`) is rejected, and §4 lists it as narrowing
  (2). Decidable structurally before any type derivation, so no
  circularity with the agreement rule. **Closed.**
- **Mixed arms.** Give-plus-return and give-plus-break both yield a
  nonempty singleton delivery set and derive from it; [EX-1]'s own
  `main` is the first shape and types correctly.
- **Chain delivery.** The former contradiction is gone: the sentence now
  reads "A final nested value initializer **bound by its own `let`**
  delivers only to its own inner let", and the qualifier is exactly what
  exempts an else-position `value_if`. Combined with "An else-position
  `value_if` of a chain is part of the chain, not a nested initializer",
  the partition rule reduces to *deliver to the nearest enclosing value
  initializer that is bound by a `let`* — total and unambiguous. **F5
  closed.**
- **Nested chains delivering to different bindings.** `let outer = if c
  { let inner = if d { give 1; } else { give 2; } give inner; } else {
  give 3; }` partitions correctly into two delivery sets by that rule.
- **Delivery through a nested `match_stmt`.** A `match_stmt` is not a
  value initializer, so gives inside one in a branch deliver to the
  enclosing value form and join its delivery set — checked, and the
  agreement rule then covers them uniformly.
- **Disagreement location.** Determinate: the error lands on the later
  `give_stmt` in source order, and with a chain all branches share one
  set, so the location is stable.

One nit: the empty-set parenthetical lists "by `propagate`'s error edge"
as a way a branch leaves, but `propagate` is not one of the five
delivery forms in the structural recursion — a `let x = propagate e;` as
a branch's final statement makes the branch *fail* to deliver rather
than deliver without a `give`, so that disjunct describes an
unreachable case. Harmless, borrowed from [ENT-5]'s join wording; drop
it or the reader will hunt for the missing recursion clause.

O4: **confirm the complete rule.**

### F6, F7, F9, F10 — CLOSED

- **F6.** [ENT-2] now reads four right-hand forms including `value_if`
  (anchor verbatim); [ENT-3] is seven sites with S4's `len<T>(P)`, S5's
  three annotated forms, and S9's annotated form all respelled, and S5
  correctly notes `cvt` keeps its written type pair. My migration attack
  — a Bool `value_match` delivering an integer, respelled to `value_if`
  and silently losing term status — no longer applies.
- **F7.** [GRAM-9]'s second site lands: "(stating its explicit mode and
  type [TYPE-5])" becomes "(whose mode and type are derived [TYPE-5])".
  The contradiction with the rewritten TYPE-5 is gone.
- **F9.** [FORM-2] now carries an explicit sole-governance clause ("is
  rendered solely by this sentence, the generic block-bearing rendering
  notwithstanding"), the block-bearing list adds only the two
  productions with "their brace blocks are inline occurrences, not
  productions", and the chain renders "never as a nested introducer
  line". All three of my attacks (double governance, non-productions in
  a production list, chain nesting) are answered, and the sentence is
  complete enough to render `value_if` too, consistently with the third
  site's let-prefix rule.
- **F10.** Both [OP-7] keyed-on-the-argument sentences and both [OP-8]
  `eeq`/`ene` sentences are re-anchored onto the operand-derived
  selected type, and `fneg(finf<T>())` is correctly retained
  byte-identical — `finf` is in the retained class, so that spelling is
  still the legal one. The reasoning is right, not just the edit.

### F8 — (a) CLOSED, (b) STILL-OPEN

**(a) closed.** The typed-call location paragraph is fully replaced,
locating retained-argument and user-generic errors at the `call` node,
infix operand-domain and operand-count errors at the `infix` node, and
operand-agreement at the second operand atom. The orphaned "uses the
same call location" sentence is gone with the paragraph.

**(b) still open — the addendum is unreachable.** The revision appends
to attribution row 2's *token list*: "; in an infix-operand occurrence,
a two-token start whose second token is an operator token — the
forbidden nested-infix start — likewise cites [GRAM-9]". But row 2's
*position guard* is untouched and still reads (verbatim in v0.22, and
verified unmodified by the candidate): "If source-EBNF provenance
reaches or would next enter an `atom` occurrence in `atom_list`,
`fieldinit`, **or the subscript offset**". An infix operand is not in
that list, so the guard never admits the occurrence the new clause
describes and the clause can never fire. The first draft had the
position list right and the token list missing; the revision has it the
other way round. `let x = a + b * c;` still fails at `*` with no cited
rule. Fix: a third DIAG-1 site adding "an `infix` operand" to the
position guard — one anchored replacement, and then the two halves meet.

Residual once fixed, worth recording rather than chasing: the addendum
detects a nested infix only when the offending operand's operator sits
at token two, so `a + b.f * c` still falls through unattributed. That is
the same bounded-lookahead character row 2's existing call/construct
detection has, so it is a known limit of the row, not a new defect.

### F11 — (a) closed with a labelling nit, (b) (d) (e) (f) closed, (c) REGRESSED, (g) one figure wrong

**(a) closed.** I re-read every §3 entry: each now quotes a verbatim
anchor and supplies replacement text, and the four former prose sweeps
(GIVE-1's "every occurrence generalizes", OP-8's "spelling mentions
only", DIAG-1's "is scoped to", OP-2's negation paragraph) are all
properly anchored. The claim "no prose sweeps" holds. Recounted the
total independently: FORM-2 3, GRAM-1 3, GRAM-4 1, GRAM-5 1, GRAM-6 1,
GRAM-7 1, GIVE-1 1, GRAM-9 2, TYPE-5 1, OP-1 3, OP-2 2, OP-7 3, OP-8 2,
ERR-2 1, ERR-3 1, FN-8 3, EFF-2 1, DIAG-1 2, DIAG-3 1, ENT-2 1, ENT-3 7,
ENT-5 1, ENT-6 1, EX-1 1 = **44**, matching. Twenty-four rules,
matching. The nit: §3 labels FN-8 "Four sites" and OP-8 "Three sites",
but each includes one item that is explicitly *retained byte-identical*
(FN-8's structural-pass sentence; OP-8's `fneg(finf<T>())` sentence). A
retention is not "one contiguous verbatim-anchored replacement" under
the candidate's own definition, and the total of 44 is only reachable by
excluding them — so the two per-rule headers should read "three sites
plus one retention" and "two sites plus one retention". The total is
right; the labels double-count.

**(b), (d), (e), (f) closed.** ERR-3 is re-anchored on the complete
original sentence with the derived-type clause appended as a new
sentence, so nothing dangles. Empty-`else` ownership is assigned
explicitly ("a `value_if`'s undelivering else is [GIVE-1]'s rejection,
not this one"). FN-8's exclusion list names `value_if`. The register
precedent is raised as O8 with the currency question stated fairly —
the two well-grounded settlements are distinguished from the TYPE-5 one,
which is exactly the distinction the register's own wording supports.

**NEW-3 (A) — (c) traded an editorial defect for a T3 defect.** The
revised [GRAM-6] reads: "An `else` whose block contains exactly one
**`else`-bearing** `if_stmt` and nothing else is a hard error citing
GRAM-6 (spell `else if`); an else-free nested `if_stmt` stays in block
form, whose flattening would change which paths continue."

Three problems, and the first is the batch's killer axis:

1. **The stated justification is false.** `if a { P } else { if b { X } }`
   and `if a { P } else if b { X }` have identical control-flow graphs:
   the inner `if` completes and control continues in both. I also
   checked the fact states — the [ENT-5] composition
   join(then, join(inner-then, inner-false)) is the same in both, since
   the join is associative. Nothing about which paths continue changes.
2. **Under the exemption reading, T3 breaks.** If "stays in block form"
   merely exempts the else-free case from the mandate, then both spellings
   above are legal, and they are one program with two spellings — a
   direct [FORM-1] violation, in the construct this batch exists to
   introduce.
3. **Under the mandate reading, the rule is under-drafted.** If "stays in
   block form" is meant as a requirement, it is the only prohibition in
   that paragraph with no "is a hard error citing GRAM-6", no node, and
   no [DIAG-1] attribution, while every sibling clause has all three —
   and it would contradict the grammar, which admits an else-free
   `if_stmt` as an `else` alternative.

The ambiguity itself means two implementations diverge, so this is A
either way. **My original F11(c) recommendation offered two scopings and
the drafter took the one that breaks; that is on the review as much as
the draft.** The correct fix is the other one, and the drafter already
has the device: restore the universal mandate ("exactly one `if_stmt`",
dropping "`else`-bearing", with the hard-error form its siblings have)
and handle the `value_if` interaction by ownership, exactly as the
empty-`else` sentence already does — in a `value_if` whose else block is
one else-free `if_stmt`, the branch cannot deliver, so it is [GIVE-1]'s
rejection and GRAM-6's unspellable fix message is never reached. Under
that scoping every one of the four cases has exactly one legal spelling
or is rejected on both: (`if_stmt`, else-free) flatten; (`if_stmt`,
else-bearing) flatten; (`value_if`, else-bearing) flatten as a chained
`value_if`; (`value_if`, else-free) rejected by GIVE-1 in either form.

**(g) — one figure is wrong, and it is the one presented as a correction
of mine.** §5 says "`check` statements: 404 — untouched (C3 open, O2;
the reviewer's 409 includes five `check` occurrences inside strings)".
Measured two independent ways over the same 399 files, the true count is
**389**: 389 lines match `^ *check ` (and [FORM-2] makes `check_stmt`
line-bearing, so every one starts its line), and 389 occurrences of
`else trap`, which is unique to `check_stmt`. My 409 was the loose
count; the twenty non-statement occurrences are all inside `doc` strings
and trap messages, not five. Neither 409 nor 404 is right. The class is
untouched by this batch, so nothing depends on it — but it sits in the
migration record and was offered as a correction, so it should be 389.
The other four figures in §5 (1748, 257, 229/47, 101) all reproduce
exactly.

### R1, R2, R3, O8 — properly recorded

R1 is a named, dated debt with O6's `.trap`/`.checked` dissolution as
its explicit discharge condition, and the [OP-1] `ModeWords` fix is
sound: the union of FORM-3's OPNAME suffixes {wrap, trap, checked, sat,
strict} with GRAM-1's operator suffixes {wrap, checked, sat} is the
former set, so `ReservedLowerNames` is unchanged in this version while
the definition stops depending on which rows happen to be respelled.
R2 records the reservation consequence explicitly. R3 records the
machinery cost. O8 raises the precedent question. All four are recorded
as findings of record rather than resolved by assertion, which is the
right disposition.

**NEW-4 (E) — "three deliberate narrowings" is two.** §1 and §4 count
delivery-type disagreement as narrowing (3), described as "same programs
rejected, different citation and location". That is right about the
citation and therefore wrong about the classification: a program v0.22
accepts must have every `give` matching one written annotation, hence
all `give`s agreeing, hence accepted under the candidate's agreement
rule. No program is newly rejected on this ground, so it is not a
narrowing — it is a re-citation, plus a widening (gives that agree with
each other but differed from the annotation) already covered by "the
error classes that lived only in deleted bytes die with them". The
batch has exactly two narrowings: the Bool-scrutinee `match` and the
empty delivery set. This sits in the version-header paragraph the owner
approves byte-exactly.

### Re-verify verdict

| finding | disposition |
|---|---|
| F1 (S) | CLOSED — join added; two editorial residues (chain phrasing, inherited empty join) |
| F2 (A) | CLOSED — factoring verified independently; NEW-1 raised |
| F3 (A) | CLOSED — enumeration verified total row by row; 101 reproduces exactly; NEW-2 raised |
| F4 (A) | CLOSED |
| F5 (A) | CLOSED |
| F6 (A) | CLOSED |
| F7 (A) | CLOSED |
| F8 (A) | (a) CLOSED, (b) **STILL-OPEN** — addendum unreachable behind an unextended position guard |
| F9 (A) | CLOSED |
| F10 (A) | CLOSED |
| F11 (E) | (a) closed w/ labelling nit, (b)(d)(e)(f) closed, (c) **REGRESSED → NEW-3**, (g) 389 not 404 |

New: **NEW-2 (A)** OP-1's row-selection sentence still keys on written
type arguments; **NEW-3 (A)** the flattening-rule rescoping is ambiguous
and, on one reading, a T3 dual spelling; **NEW-1 (E)** phantom `infix`
production and the 70-vs-69 count; **NEW-4 (E)** narrowing miscount.

**Recommendation: one more short round, not ready-for-owner.** The
revision is a large and genuinely good-faith pass — eight findings
closed outright, the two hardest (the ENT-5 join and the LL(2)
factoring) closed correctly and for the right reasons, the retained-
argument enumeration verified total against the whole operation table,
and the corpus figures now reproducing to the site. What remains is
three anchored sentences and two numbers:

1. **NEW-3** — restore the universal flattening mandate in its siblings'
   hard-error form and delegate the `value_if` case by ownership. This
   is the only finding that touches T3, and T3 is the axis this batch
   lives or dies on.
2. **NEW-2** — one OP-1 site rewording row selection off the written
   argument.
3. **F8(b)** — one DIAG-1 site adding "an `infix` operand" to
   attribution row 2's position guard.
4. **NEW-1** — pin the production total at 69 or 70 and make [GRAM-1]'s
   node-mapping sentence consistent with it, before the grammar-path
   task discovers the mismatch against an already-approved header.
5. **NEW-4** and the §5 `check` figure (389) — two number corrections in
   owner-approved bytes.

None requires new design; all five are edits the drafter can make
without a further review round beyond confirming the five landed. Given
that the last revision closed eight of eleven cleanly, I would expect
the next pass to be verifiable in a single targeted read rather than a
full re-attack.

---

### Final pass (2026-08-07; candidate at 0e81dd8, 848 lines, 24 rules / 46 sites, 69 productions)

Single read against the five items named at the end of the re-verify.
All five landed; every newly quoted anchor resolves verbatim and
uniquely in v0.22 (DIAG-1's row-2 position guard, ENT-5's loop-join
sentence, OP-1's row-selection sentence, ERR-2's exhaustiveness
sentence — each exactly one occurrence, so every replacement is
unambiguous).

**1. F8(b) — guard and token list now meet.** DIAG-1 is three sites, and
the new one extends the position guard to "an `atom` occurrence in
`atom_list`, `fieldinit`, an `infix` operand, or the subscript offset".
Traced end to end on the witness: for `let x = a + b * c;` provenance
would next enter the infix-operand occurrence at `b`, the guard now
admits it, the two-token start is (IDENT, `*`), the token-list addendum
fires, and the rejection cites GRAM-9 at `*`. **Closed.** The bounded-
lookahead residual I recorded stands unchanged and unfixed by design —
`a + b.f * c` puts the operator past token two and still falls through
— which is the same limit row 2's existing call/construct detection
has, correctly left alone.

**2. NEW-3 — universal mandate, hard-error form, ownership delegation.**
GRAM-6 now reads "An `else` whose block contains exactly one `if_stmt`
and nothing else is a hard error citing GRAM-6 at that nested `if_stmt`
node (spell `else if`); in a `value_if` whose else block is exactly one
else-free `if_stmt`, the branch cannot deliver, [GIVE-1] owns the
rejection, and GRAM-6 forms no candidate there". The `else`-bearing
qualifier is gone, the clause has the citation and node its siblings
have, the false path-continuation justification is deleted, and the
delegation uses v0.22's own "forms no candidate" idiom. Re-walked all
four cases: (`if_stmt`, else-free) now flattens, so
`if a { P } else { if b { X } }` is rejected and only `else if` remains
— **the T3 dual spelling is closed**; (`if_stmt`, else-bearing)
flattens; (`value_if`, else-bearing) flattens into a chained `value_if`
and is correctly outside the carve-out, which is scoped to the else-free
case only; (`value_if`, else-free) is GIVE-1's, and the carve-out is
total because "an else-free `if_stmt` has a continuing false edge and
never delivers" is categorical. Each case has exactly one legal spelling
or is rejected in both forms, with a followable diagnostic in every one.
The empty-`else` clause also gained the node it was missing. **Closed.**

**3. NEW-2 — OP-1 fourth site.** Row selection is reworded off the
deleted input: "uses the operand domains and, for the retained-argument
operations [TYPE-5], the written arguments, to select the applicable
row". Checked it still does its two jobs: the two `cvt` rows are
selected by the written pair (`cvt` is retained), and `ishr`'s
`ashr`/`lshr` and `imin`'s `smin`/`umin` are selected by the operand
domain. **Closed.**

**4. NEW-1 — clean 69 with an honest 1:1 mapping.** The phantom
`infix := atom infix_tail` production is deleted from the block; the
mapping is now "`infix_tail` maps 1:1 to the `infix` node kind: a
selected tail forms one `infix` node spanning the complete `expr` — the
atom and the tail", with no phantom production and no exception clause
[META-3]. Productions are +4 (`if_stmt`, `value_if`, `infix_tail`,
`infix_op`), total **69**, and §1, §2, and the GRAM-1 sentence all
agree. The node spans more than its own production, which is unusual but
stated, and it is what DIAG-1's "at the `infix` node and its complete
extent" and DIAG-3's "the trapping `infix` node" both need — checked
both, and each still resolves. The LL re-run is reasoned rather than
re-asserted ("the dropped `infix` production appeared on no right-hand
side, so every decision row is identical"), which is correct: an
unreachable production contributes to no decision, so all seven rows
carry over unchanged. **Closed.**

**5. Numeric corrections — three of four exact, one still wrong.**

- **389 ✓.** §5 now carries 389 with both derivations stated (line-
  leading `check`, exact under FORM-2's line-bearing rule; confirmed by
  `else trap`, unique to `check_stmt`) and correctly characterizes the
  draft's 404 and my 409 as loose counts. Matches my measurement.
- **Two narrowings ✓.** §1 and §4 both drop delivery-type disagreement
  to a re-citation and place the widening explicitly ("`give`s that
  agreed with each other against a wrong annotation"), which is a
  cleaner statement than the one I proposed.
- **ENT-5 CFG idiom and empty join ✓.** The sentence now reads "the join
  of the states on every branch exit edge reaching that continuation on
  the conservative structural graph [FN-1]", matching the `match`
  sentence word for word in idiom, with the else-free false edge named
  as such an edge. Chains need no enumeration: an else-position `if`
  contributes its branch exit edges directly to the shared continuation.
  The empty join is defined once and governs "`match_stmt` and
  `value_match` continuations identically", so the hole v0.22 inherited
  is retired rather than duplicated. Re-ran my three sharpest F1 attacks
  against the reworded sentence — original bounds attack, kill-in-one-
  branch, one-branch-returns — all three give the same sound outcome as
  before. Checked that the contradictory empty-join state cannot leak:
  it derives every bound, so a later join taking "the weakest bound held
  by all" is never restricted by it and yields the other state's bounds.
- **1357 ✗ — the figure and its reconciliation are both wrong.** §5 now
  says "**1357** occurrences on 1356 source lines — exactly one line
  carries two deleted-class calls, which reconciles the reviewer's
  line-oriented 1356 with this occurrence count". My 1356 was not
  line-oriented: it came from `grep -oE`, which emits one line per
  *match*, so it was already an occurrence count and the stated
  reconciliation does not hold. I settled the difference by enumerating
  every distinct lowercase callee taking a `<` across the 399 files and
  summing by hand. The genuine deleted class — every table operation
  except the six retained — totals **1353** occurrences. My 1356 was
  1353 plus three deliberately-invalid OPNAME spellings my pattern
  admitted: `irotl.trap` (rotates are dotless-total, [OP-8]),
  `idiv.wrap` ("There are no wrap modes for division/remainder",
  [OP-2]), and `fneg.strict` (`fneg` is dotless, [OP-3]). Two more
  invalid spellings sit in the same corpus and neither pattern caught,
  `iadd.bogus` and `add.wrap`. All five are rejection fixtures in
  negative conformance cases; none is a table operation, so none is a
  member of the deleted class, and the printer will never delete a type
  argument from them because they never parse to a table-op call.
  **The figure should be 1353**, with a separate line noting the five
  invalid-spelling fixtures that migrate by no rule. This is confined to
  §5's migration record — §1's version-header paragraph does not cite
  it, and no rule, acceptance class, token count, or production count
  depends on it — so it is a record correction, not a defect in the
  batch.

**Accounting recounted independently.** Sites: FORM-2 3, GRAM-1 3,
GRAM-4 1, GRAM-5 1, GRAM-6 1, GRAM-7 1, GIVE-1 1, GRAM-9 2, TYPE-5 1,
OP-1 4, OP-2 2, OP-7 3, OP-8 2, ERR-2 1, ERR-3 1, FN-8 3, EFF-2 1,
DIAG-1 3, DIAG-3 1, ENT-2 1, ENT-3 7, ENT-5 1, ENT-6 1, EX-1 1 = **46**,
across **24** rules — both matching, and both reconciling against the
previous 44 as exactly the two new sites (OP-1's row selection, DIAG-1's
position guard). The two retention relabels ("plus one retention" on
FN-8 and OP-8) fix the labelling nit without moving the total, since
retentions were already excluded from 44. OP-8's claim that the `eeq`
and both-operands sentences "are adjacent in v0.22 and form one
contiguous site" is correct — they are consecutive sentences.
Productions 65 + 4 = 69 ✓.

**One carry-forward note for the grammar-path task, not a finding.** The
production is named `infix_tail` while its node kind is named `infix`;
every other production/node pair in this grammar shares a name. If the
generated syntax data derives node kinds from production names, that
asymmetry is the one place this batch will surprise the extension task —
which is precisely the surprise NEW-1 existed to prevent, so it is worth
one line in the task's success criteria rather than discovering it at
verifier time.

### Final verdict

**READY-FOR-OWNER.** All five items landed as specified; the two
findings that touched the batch's load-bearing axes — NEW-3 on T3
uniqueness and F8(b) on GRAM-9 attribution — are closed with the
mechanisms I would have written, and NEW-3 in particular is closed using
the candidate's own ownership idiom rather than a special case. The
grammar is strong-LL with an honest 69-production count and a 1:1 node
mapping; the ENT-5 join is now stated in the spec's own CFG idiom and
retires an inherited hole instead of duplicating it; the retained-
argument class is total against the complete operation table; GIVE-1
ships as a fully-worked rule; and every corpus figure in §5 except one
reproduces exactly.

Residual, named and non-blocking: **§5's deleted-class figure should be
1353, not 1357, and the "1356 lines" reconciliation is incorrect** — the
gap is five invalid-OPNAME rejection fixtures, not a line-counting
artifact. One line in the migration record, outside the version-header
bytes, affecting no rule or count. Recommend correcting it in the commit
that goes to the owner; it does not warrant another review round.
