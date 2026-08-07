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
