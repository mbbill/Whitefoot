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
