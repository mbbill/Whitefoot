# Obligation-discharge batch 1 — specification-change candidate

Status: CANDIDATE, DRAFT (2026-08-06). Non-authoritative. This document is the
complete batch-1 delta of the obligation-discharge design against the exact
text of `spec/kernel-spec-v0.20.md`. It authorizes nothing: per
`docs/WORKFLOW.md`, activation requires owner approval of exact bytes, and per
`compiler/README.md` a grammar-extending candidate additionally requires the
native grammar path to be extended first (verifier evidence in §3). Scope is
exactly items 1–3 of `research/investigations/obligation-discharge/DOSSIER.md`
§8: the claim construct, the normative L0 entailment fragment, and caller-side
discharge for OP-4 index bounds only. Everything else in that dossier —
arithmetic-mode dissolution, requires-as-goal, ensures, the taint gate,
boundary-op count postconditions, counted range loops, partitions and ledger
tooling — is deliberately out of this batch (§10).

Base: `spec/kernel-spec-v0.20.md` (REVIEW CANDIDATE v0.20 bytes as of
2026-08-06). Target version number and canonical candidate path are an owner
choice (open question O1); this file deliberately does not occupy
`kernel-spec-v0.21-candidate.md`.

## 1. Proposed version-header paragraph

The following paragraph is drafted for the eventual numbered candidate's
status header, in the v0.20 header conventions.

> Status: REVIEW CANDIDATE vNEXT (2026-08-06; obligation-discharge batch 1:
> the claim statement, the normative L0 entailment fragment, and caller-side
> discharge of index bounds). Adds one named runtime-check statement —
> `claim name: e because "text";` — whose semantics are exactly [OP-5]'s
> check-else-trap plus a name carried by the [DIAG-3] trap record and a
> version-monotone lifecycle: a claim whose predicate the checker already
> proves is a non-rejecting redundancy advisory, a claim whose predicate the
> checker refutes is a hard error, and a fired claim is surfaced for
> reclassification as a toolchain contract [CLM-1, CLM-2]. Adds the closed
> deterministic L0 entailment fragment as normative acceptance machinery
> [ENT-1..ENT-6]: difference-bound facts over tracked places, length terms,
> and constants; fact sources exactly enumerated (dominating branch and match
> facts, check and claim facts, FN-8 requires facts by clause-local
> substitution, copy/conversion equalities, allocation-length equalities,
> constant-offset wrap/trap/checked arithmetic, the unsigned midpoint family,
> const-array element ranges, implicit type ranges); shortest-path closure
> with disequality strengthening; and kill, join, and no-induction loop rules
> driven by resolved-place overlap and [EFF-2] effect-row projection.
> Rewrites [OP-4]: a source `index` compiles with no runtime bounds check
> exactly when the fragment discharges its bounds obligation at that node,
> and an undischarged index is a compile-time rejection whose diagnostic
> prints the residual obligation; the total read form `index_get` returning
> `Option<T>` is added to the operation table; the [SYS-8] range-validation
> trap is unchanged as an operation-internal contract check. [FN-8]'s
> foreign-entry execution is restated as one toolchain-synthesized boundary
> adapter at gated entries, trap semantics unchanged. Specification delta:
> numbered rules +8/-0 (CLM-1, CLM-2, ENT-1, ENT-2, ENT-3, ENT-4, ENT-5,
> ENT-6); eleven existing rules modified: FORM-2 (claim_stmt is line-bearing),
> FORM-5 (STRING homes), GRAM-4 (claim_stmt production; stmt gains one
> alternative), GIVE-1 (claim is non-delivering), OP-1 (index_get row;
> derived reserved sets grow by one), OP-4 (rewritten to discharge-or-reject),
> FN-8 (passed fact feeds ENT-3; synthesized boundary adapter), EFF-2 (traps
> contribution: bounds-checked index out, claim in), SET-1 (no runtime check
> in target evaluation), DIAG-2 (discharged disposition; claims always
> retained), DIAG-3 (claim trap record; index-place row removed). Tokens
> +2/-0 (`claim`, `because` as exact fixed lowercase grammar atoms; terminal
> predicates 75 -> 77); terminal spellings +2/-0; grammar productions +1/-0
> (`claim_stmt`); exception clauses +0/-0; source constructs +1 (the claim
> statement); operation table +1 row (`index_get`), growing the derived
> `DotlessOperationNames` and `ReservedLowerNames` sets by one member each;
> sections +1 (new §18, worked example and meta-rules renumbered; no existing
> cross-reference names either renumbered section). The accepted-program set
> changes in both directions and this transition is deliberately non-monotone
> (§7): it grows by claim statements and `index_get` calls, and it shrinks by
> exactly four classes — an `index` whose bounds obligation the fragment does
> not discharge, an effect row whose `traps` was exhibited only by
> bounds-checked indexes, a declaration spelled `index_get`, and identifiers
> spelled `claim` or `because`. From this version forward the [ENT-1]
> monotonicity law governs: checker strengthening may only convert claims to
> advisories and undischarged obligations to discharged ones. Selection
> ground: evidence-selected — SIMULATION.md (L0 discharges 57–59% of
> non-test bounds sites outright on three real programs, every residual one
> line, threading depth ≤ 3), PROBE-W1 rounds 1–2 (16/16 honest writer shapes
> steered by the residual-printing error), PROBE-TAINT (one structural claim
> in 723 lines at a real gated boundary), PROBE-CODEGEN (claim shape equals
> today's fused check shape by construction). These bytes are
> non-authoritative until the grammar check, derived-material review,
> full-document hash, exact owner approval, and active-target installation
> complete.

## 2. Grammar delta

[GRAM-4]'s statement block becomes (complete replacement of the two changed
lines plus one added production; every other line byte-identical):

```
stmt        := let_stmt | set_stmt | expr_stmt | return_stmt | loop_stmt
             | break_stmt | region_stmt | check_stmt | claim_stmt | match_stmt
             | give_stmt
check_stmt  := "check" expr "else" "trap" STRING ";"
claim_stmt  := "claim" IDENT ":" expr "because" STRING ";"
```

`claim` and `because` become exact fixed lowercase grammar atoms and are
therefore excluded from IDENT by [FORM-3]'s existing exclusion clause with no
FORM-3 text change. Raw lexical formation [GRAM-1] is untouched: both spell as
ordinary lower words; terminal membership gains two exact fixed-terminal
predicates. The `stmt` decision stays strong-LL(2): the new arm is selected by
first token `claim`, which begins no other statement arm; inside `claim_stmt`,
the `psuffix*` and call continuations of `expr` are exited by `because`, which
no expr-interior decision consumes.

## 3. Native grammar-verifier evidence

Control run (approved v0.20 candidate, unmodified):

```sh
cargo run --manifest-path compiler/Cargo.toml --bin whitefoot-grammar -- \
  governance/spec-evolution/kernel-spec-v0.20-candidate.md
# -> grammar-preserving candidate verified by the active compiler:
#    64 productions, 74 decisions, 75 terminal predicates
# exit code 0
```

Batch-1 probe (byte-exact copy of `spec/kernel-spec-v0.20.md` with exactly
the §2 grammar delta and the FORM-2 line-bearing-list edit applied; scratch
file, not part of this candidate):

```sh
cargo run --manifest-path compiler/Cargo.toml --bin whitefoot-grammar -- \
  /Users/bytedance/do_not_scan/wf-batch1/batch1-grammar-probe.md
# -> whitefoot-grammar: candidate changes the lexer or source grammar of the
#    active specification; a structural change must first extend the native
#    grammar path
# exit code 1
```

This is the verifier's specified fail-closed behavior for a
grammar-extending proposal (`compiler/README.md`): the production set cannot
be verified grammar-preserving because it is deliberately grammar-extending.
Consequence for activation sequencing: the compiler's lexer/parser and
committed grammar tables must be extended with `claim_stmt` and the two new
terminals first, after which the verifier reruns against the full numbered
candidate and must report exactly 65 productions and 77 terminal predicates.
No unintended grammar drift exists: the probe differs from v0.20 by the three
quoted lines only.

## 4. New rules — claims

These rules, with ENT-1..ENT-6, form a new section
"## 18. Obligation discharge: claims and the entailment fragment (normative)"
inserted before the worked example; the worked example and meta-rules
renumber to §19 and §20 (no existing text cross-references either section by
number). Rule rank under [DIAG-1] follows first appearance as always.

[CLM-1] `claim name: e because "text";` is a named runtime check. `e` must
have exact value mode and type `own Bool` under exactly the [OP-5] condition
judgment, including the TYPE-7 implicit-read exclusivity: when `e` uses a
borrow-mode or box/arena binding where its referent `Bool` value would be
required, that use is rejected citing TYPE-7 and CLM-1 forms no candidate.
Every other exact-mode or exact-type failure is a hard error citing CLM-1 at
the selected `expr` node, with `SourceCoordinate` equal to that node's
complete checked half-open source extent. A conforming claim is a runtime
check in all build modes and is never elided; its checked-program disposition
is always `retained` [DIAG-2]. If `e` is `False()` it emits the required trap
record naming this claim [DIAG-3] and aborts [SCOPE-4, EFF-4]; if `e` is
`True()` execution continues, and the passed fact enters the dominated
continuation's fact state exactly as [ENT-3] admits it. A `claim_stmt`
syntactically exhibits `traps` [EFF-2] and does not count as delivery or
must-divergence [GIVE-1].

The claim name is one IDENT and is not a declaration: it enters no [TYPE-6]
domain, no [OP-1] reservation inventory, and no lexical lookup, and no source
construct references it. Within one `fn_decl` every claim name is unique; a
repeated spelling is a hard error citing CLM-1 at the later `claim_stmt`
node. The `because` STRING is the claim's justification: mandatory
compile-time review data retained by the checked program [DIAG-2], absent
from runtime behavior, and never semantics-selecting. A claim is legal in
exactly the statement positions [GRAM-4] admits; [FN-8]'s structural pass
continues to admit only ordinary lets and one final check, so a claim cannot
appear in a `requires` block. This version defines no taint judgment: no
predicate is illegal by operand provenance (the subject-position gate is a
later batch with its own delta).

[CLM-2] Claim lifecycle judgments are fixed by the entailment fragment and
are version-monotone [ENT-1]. When the closed fact state dominating a
`claim_stmt` derives its predicate [ENT-4], the claim is redundant: the
program remains accepted, the check still executes [CLM-1], and a conforming
implementation reports one non-rejecting redundancy advisory naming the claim
— an advisory is not a [DIAG-1] rejection, and a later specification version
that proves more predicates therefore rejects no previously accepted program
on that ground. When the fact state is non-contradictory [ENT-4] and derives
the predicate's exact negation, the program is rejected with a hard error
citing CLM-2 at the `claim_stmt` node, carrying the claim name, the
predicate, and the derived negation: a refuted claim is a defect found at
compile time. A claim whose trap record any execution produces is thereby
demonstrated not to be a necessary truth; surfacing fired claims for
reclassification is a toolchain contract in the [ERR-2] edit-list sense, not
a language judgment. Advisory channel and encoding are implementation-owned
in this version (open question O5).

## 5. New rules — the L0 entailment fragment

[ENT-1] The L0 entailment fragment is a closed, deterministic, search-free
derivation system fixed completely by this specification. Its judgments are
source-acceptance judgments: obligation discharge [ENT-6], claim redundancy,
and claim refutation [CLM-2] are post-resolution semantic judgments under
[DIAG-1], identical in facts-on and facts-off compilation, and are not an
optional optimizer-fact family. [SCOPE-2] is unchanged: every fact source
[ENT-3] is an executed runtime check, an executed entry prologue, a declared
allocation or type property, or a constant — no construct introduces a fact
without a proof or an executed check, and nothing writer-stated is trusted
unchecked. The fragment is the "deterministic checker derivation" of [OP-4]
and [DIAG-2] for the obligations this version attaches; a solver result never
participates, and no implementation may strengthen, weaken, time-bound, or
randomize the derivable set: two conforming implementations derive the same
closed fact state at every program point and the same disposition for every
obligation and claim. The fragment joins the trusted computing base exactly
as the type and ownership checkers do [SCOPE-3]; a wrong derivation is a
compiler defect class, owned by testing, not a language hedge. Version
monotonicity is law: a later specification version may add fact sources and
closure rules and may remove none, so checker strengthening converts claims
into [CLM-2] advisories and undischarged obligations into discharged ones,
and never the reverse.

[ENT-2] The fragment judges one function body at a time; no fact crosses a
call boundary except as [ENT-3] source S4 fixes for the body's own `requires`
prologue. A fragment type is one member of the closed integer set [OP-2];
relations are over mathematical values, so relations between terms of
different fragment types are well-formed and are created only by the sources
[ENT-3] admits.

A term is exactly one of: (a) a tracked place — a `place` [GRAM-5] whose root
`pbase` IDENT resolves to a `param`, ordinary `let`, requires-clause local,
or match-binder value binding or to a named const [CONST-2], formed with any
number of `psuffix` field selections and `deref` wrappings and no `index`
segment, whose final selected type is one fragment type; (b) a length term
`len(P)`, of fragment type u64, where P is a place formed under the same
restriction whose final selected type is `array<T, N>`, `slice<'r, T>`, or
`buffer<T>`; (c) a constant — the mathematical value of an integer literal or
of an integer-typed named const, or symbolically an in-scope integer-typed
const-generic parameter; or (d) the distinguished zero term Z, used only to
carry constant bounds. Two places are the same term exactly when their
canonical source spellings [FORM-2] are byte-identical; distinct spellings
are distinct terms even when they resolve to overlapping storage. Term
identity thus under-approximates aliasing, which is sound for derivation,
while kills [ENT-5] use resolved-place overlap [OWN-5], which
over-approximates it.

An atomic fact is one difference bound `t1 - t2 <= c` (t1, t2 terms, c a
mathematical integer) or one disequality `t1 != t2`. Source relations
normalize exactly: `a <= b` is `a - b <= 0`; `a < b` is `a - b <= -1`;
`a = b` is the bound pair `a - b <= 0` and `b - a <= 0`; `a >= b` and
`a > b` swap operands; `a != b` is one disequality. A constant operand folds
through Z: `a <= 7` is `a - Z <= 7`. Implicit facts hold at every program
point: every term t of fragment type T carries `t - Z <= max(T)` and
`Z - t <= -min(T)`; every length term over a place of type `array<T, N>`
carries the equality `len(P) = N` (both bounds), with concrete N a constant
and const-generic N a symbolic constant term.

[ENT-3] The fact state at a program point contains exactly the facts below
whose establishing event dominates the point on the conservative structural
normal-control graph [FN-1], subject to the kills, joins, and loop rule of
[ENT-5], closed under [ENT-4]. Nothing else is a fact: no ensures, struct
invariant, loop induction, user-function postcondition, or taint judgment
exists in this version.

A comparison origin is defined first. An expression has comparison origin R
when (a) it is a call to one of `ieq`, `ine`, `ilt`, `ile`, `igt`, `ige`
[OP-2] whose two operands are each a term or constant, R the corresponding
relation over them; or (b) it is a bare IDENT naming a `let` binding of type
`own Bool` whose initializer right-hand side satisfies (a) with relation R,
no operand term of R is killed [ENT-5] on any path from that initializer to
the use, and the binding is the target of no `set` on any such path. No other
shape has one: `band`, `bor`, `bnot`, `eeq`, `ene`, user-function results,
and deeper indirection chains contribute nothing in this version.

The sources are:

- S1 (branch and match facts). For a `match_stmt` or `value_match` whose
  scrutinee has comparison origin R, R is established at the `True()` arm's
  entry and R's exact negation at the `False()` arm's entry. Negation is
  exact over mathematical integers: the negation of `a - b <= c` is
  `b - a <= -c - 1`; the negation of `a = b` is `a != b` and conversely.
- S2 (check facts). After `check e else trap "…";` [OP-5] whose `e` has
  comparison origin R, R is established on the normal continuation.
- S3 (claim facts). After `claim n: e because "…";` [CLM-1] whose `e` has
  comparison origin R, R is established on the normal continuation.
- S4 (requires facts). At function-body entry, take the final `check`
  condition of the `requires` block [FN-8] and replace every requires-clause
  local by its unique defining right-hand side, repeatedly, until only
  parameters, named consts, literals, and admitted table-operation calls
  remain. When the result is one comparison call admitted by the
  comparison-origin shape (a) whose operands after substitution are each a
  term over parameters or named consts, a constant, or a call `len<T>(P)`
  over such a place — read as the length term len(P) — that relation is
  established at body entry. Any other substituted shape establishes nothing;
  the prologue still executes [FN-8].
- S5 (copy and conversion equalities). An `ordinary_let_rhs` establishes at
  its binding: for `let x: own T = lit;`, x = value(lit); for
  `let x: own T = p;` with p a term of type T, x = p; for
  `let y: own Dst = cvt<Src, Dst>(p);` with (Src, Dst) a total pair [OP-6]
  and p a term or constant, y = p.
- S6 (length facts). `let b: own buffer<T> = buffer_new<T>(n, v);`
  establishes len(b) = n on the normal continuation [OP-9], n read as term or
  constant. `let m: own u64 = len<T>(P);` for a tracked P establishes
  m = len(P). `let s: own slice<'r, T> = slice_of…(&'r P);` establishes
  len(s) = len(P).
- S7 (constant-offset arithmetic). For `let s: own T = iadd.wrap<T>(p, k);`
  with p a term of type T and k a constant in either operand position, when
  the closed state at that point derives `min(T) <= p + k` and
  `p + k <= max(T)` (as bounds on p through Z), s = p + k is established;
  `isub.wrap<T>(p, k)` with constant k establishes s = p - k under the dual
  range condition. For `iadd.trap<T>(p, k)` and `isub.trap<T>(p, k)` with
  constant k, s = p ± k is established on the normal continuation
  unconditionally: the executed contract check is the proof [OP-2]. For a
  `match` whose scrutinee is directly a call `iadd.checked<T>(p, k)` or
  `isub.checked<T>(p, k)` with constant k, or a bare IDENT let-bound to one
  with no kill of p between the initializer and the match and no `set` of
  that binding, the `Ok(value: w)` arm establishes w = p ± k at arm entry;
  the `Err` arm establishes nothing.
- S8 (the midpoint family). Where a body contains, in this definitional
  shape with T unsigned, lo and hi terms of type T,

  ```
  let d: own T = isub.wrap<T>(hi, lo);
  let h: own T = ishr.wrap<T>(d, 1_u32);
  let m: own T = iadd.wrap<T>(lo, h);
  ```

  (the three lets need not be adjacent; `idiv.trap<T>(d, 2_T)` is admitted as
  the alternative defining shape of h), and no member of {lo, hi, d, h} is
  killed or `set` between its definition and m's definition, then: when the
  closed state at m's definition derives lo <= hi, the facts lo <= m and
  m <= hi are established at m; when it derives lo < hi, additionally
  m <= hi - 1. This is the whole family; no other multi-variable arithmetic
  composes in this version.
- S9 (const-array element ranges). For `let x: own T = index<T>(c, i);`
  where c is the bare IDENT of a named const of type `array<T, N>` [CONST-2]
  and T a fragment type, with vlo and vhi the minimum and maximum of its N
  declared element values, vlo <= x and x <= vhi are established at the
  binding. The index's own bounds obligation [ENT-6] is judged separately
  and is unaffected. Deeper const shapes establish nothing in this version.

[ENT-4] The closed fact state at a point is the least set containing its
established and implicit facts and closed under exactly: (1) from
`t1 - t2 <= c1` and `t2 - t3 <= c2`, derive `t1 - t3 <= c1 + c2`; (2) from
`t1 - t2 <= 0` and a disequality between t1 and t2 in either orientation,
derive `t1 - t2 <= -1`; (3) of two bounds on one ordered pair, the smaller
constant subsumes. The closure is unique and finite up to subsumption: only
terms written in the function participate, and it equals the all-pairs
shortest-path bounds of the difference graph with disequality strengthening
iterated to its unique fixed point. Derivability is exact: `a - b <= c` is
derivable when the closed state contains `a - b <= c'` with c' <= c;
`a = b` when both `a - b <= 0` and `b - a <= 0` are derivable; `a != b` when
a disequality is present or `a - b <= -1` or `b - a <= -1` is derivable. A
state is contradictory when `t - t <= -1` is derivable for any t; at a
contradictory point every relation is derivable, every obligation is
discharged, and no claim is refuted — [CLM-2] refutation requires a
non-contradictory state, so a claim there is redundant, never rejected.
Implementations may compute lazily or incrementally, but every derivability
answer must equal the least-closure answer.

[ENT-5] The support of a fact is: every tracked place occurring in its
terms; for each length term len(P), the root binding of P but not P's element
storage — an element write never kills a length fact, because a `buffer<T>`
length is fixed at allocation and an `array<T, N>` or `slice<'r, T>` length
is fixed by its type or creation [TYPE-2, OP-1]; and every borrow or
box/arena holder binding any of its places reads through by `deref`. Z and
constants have empty support and never die.

A fact dies at the earliest of: (a) a `set p = e;` commit whose resolved
target [SET-1, OWN-5] overlaps the resolved place of any support member;
(b) a call — user function, table operation, or system operation — one of
whose [EFF-2] boundary-projected `writes` occurrences projects onto a caller
place or origin set whose storage may overlap the resolved place of any
support member; the projection is exactly [EFF-2]'s, so a callee writing only
through one `&uniq` actual kills exactly the facts whose support overlaps
that actual's resolved place, and a call whose row carries no `writes` kills
nothing; (c) a consuming use [OWN-1] of any support member's root; (d) the
end of the region of any borrow holder in its support and the end of the
lexical scope of any support binding, including region exit [OWN-3].

Joins: at the continuation of a `match_stmt` or `value_match`, the fact
state is the join of the closed states at every arm exit edge reaching that
continuation on the conservative structural graph [FN-1]; an arm every path
of which leaves by `return`, `break` to an enclosing loop, or `propagate`'s
error edge contributes nothing there. The join keeps, for each ordered term
pair, the weakest (largest-constant) bound held by all joined states, and
each disequality held by all of them; the join of closed states is closed.
The continuation of a `loop_stmt` is the join over the closed states at its
`break` statements. A `propagate` right-hand side's `Err` edge leaves the
function; its normal continuation keeps the preceding state and its binder
gains no fact.

Loops carry no induction in this version: the fact state at the head of each
iteration of `loop @l { … }` is the state before the loop minus every fact
having a support member that any kill event (a)–(d) occurring anywhere
inside the loop body, at any nesting depth, may kill. The surviving facts
hold at every iteration head; establishment and kills then proceed ordinarily
within the iteration, and no fact established inside an iteration survives to
the next iteration's head. Loop induction is a later version's
[ENT-1]-monotone extension.

[ENT-6] An obligation is one normalized relation attached by a numbered rule
to one source node, instantiated with that node's exact operands read as
terms or constants; an operand that is not a term or constant leaves the
relation underivable, never ill-formed. This version attaches exactly one
obligation family: for every source `index<T>(P, i)` place — read, write,
and [SET-1] target position alike — the bounds obligation `i < len(P)`,
normalized `i - len(P) <= -1`, at the `index` node. The obligation is
discharged exactly when the closed fact state at that node derives it
[ENT-4, ENT-5]. An undischarged obligation is the [OP-4] rejection; its
diagnostic renders the residual as exactly: the offset atom's canonical
source bytes, then ` < len(`, then the base place's canonical source bytes,
then `)`. The mechanical fix is one dominating claim or branch establishing
the relation — in canonical ANF, one `let` binding `len<T>(P)` followed by
one `claim` on, or `match` over, the admitted comparison [CLM-1, ENT-3];
that fallback always closes discharge and reproduces the pre-revision
runtime check per site, from zero cost where facts already prove the bound
to full price where none do.

## 6. Modified rules (complete replacement deltas)

Each entry states the exact edit against v0.20; unquoted text is unchanged.

**[FORM-2]** In the line-bearing list, `…`break_stmt`, `check_stmt`, and
`give_stmt`…` becomes `…`break_stmt`, `check_stmt`, `claim_stmt`, and
`give_stmt`…`.

**[FORM-5]** "STRING appears only in `doc` and `check` messages" becomes
"STRING appears only in `doc` entries, `check` messages, and `claim`
justifications".

**[GRAM-4]** As §2 above: `stmt` gains the `claim_stmt` alternative between
`check_stmt` and `match_stmt`; the production
`claim_stmt := "claim" IDENT ":" expr "because" STRING ";"` is added after
`check_stmt`.

**[GIVE-1]** "A `check` or call that may trap also has a normally continuing
edge…" becomes "A `check`, `claim`, or call that may trap also has a
normally continuing edge…".

**[OP-1]** One table row is added:

| op | domain | signature | effects |
|---|---|---|---|
| `index_get` | `array<T, N>`, `slice<'r, T>`, `buffer<T>`, copy element T | `(place, u64) -> own Option<T>` | pure |

No text change: `DotlessOperationNames`, and therefore `ReservedLowerNames`,
grow by the derived member `index_get`. `index_get` is a table operation with
positional operands [GRAM-11].

**[OP-4]** Complete replacement:

> [OP-4] `index<T>(p, i)` carries the bounds obligation `i < len(p)`
> [ENT-6]. A discharged index reads or writes with no runtime bounds check in
> every build mode, and its checked-program disposition records the
> discharging derivation [DIAG-2]. An index whose obligation is not
> discharged is a compile-time rejection citing OP-4 at the `index` place
> node, carrying the residual obligation rendered exactly per [ENT-6]; the
> mechanical fix is a dominating `claim` of the residual [CLM-1] or a
> dominating branch establishing it [ENT-3]. Discharge is a deterministic
> checker derivation [ENT-1]; a solver result never participates. `index`
> applies to `array<T, N>`, `slice<'r, T>`, and `buffer<T>` places; a
> `buffer<T>` obligation is over the runtime length term. An `index` in a
> [SET-1] target forms the selected place without reading its stored value;
> its base and offset are evaluated during target evaluation, and its
> discharge judgment is identical in target position. A successful bounds
> judgment neither narrows nor authorizes narrowing the index or its scaled
> byte offset; target address formation additionally obeys [STOR-6]. The
> range validation of the system transfer operations [SYS-8] is unchanged:
> it is an operation-internal contract check with table-fixed trap semantics
> [ERR-4] whose trap record uses the operation `call` node [DIAG-3]; the
> discharge judgment does not apply to it. `index_get<T>(p, i)` [OP-1] is
> the total read form over the same three domains with copy element T: it
> returns `Some(value: v)` with a copy of the selected element when
> `i < len(p)` and `None()` otherwise; the length comparison is its own
> semantics, not a contract check, so it is pure and total, carries no
> obligation, and is an expression call, never a place; it does not admit
> writes.

**[FN-8]** Two sentences change. "…every invocation executes it once after
parameter binding and before the function body, including an invocation
entering through a gated foreign boundary" gains: "; a gated foreign entry
executes it through one toolchain-synthesized boundary adapter [GATE-1,
LEDGER-1] — compiler-owned, not writer-visible source, executing the
complete prologue before the body on every foreign entry, with trap
semantics unchanged in this version; a boundary error protocol replacing
that trap is a later amendment with its own delta". The final sentence "The
final statement has exactly [OP-5] semantics; a deterministic proof from its
passed fact may eliminate only downstream implicit checks such as [OP-4]
bounds checks." becomes "The final statement has exactly [OP-5] semantics;
its passed fact enters the function body's entry fact state exactly as
[ENT-3] source S4 fixes."

**[EFF-2]** Two edits. In the syntactic contribution: "exhibit `traps` iff
either contains any `.trap` op, `check`, a bounds-checked `index`, or a call
to any operation or function whose effect row includes `traps` (even if
later proven away)" becomes "exhibit `traps` iff either contains any `.trap`
op, `check`, `claim`, or a call to any operation or function whose effect
row includes `traps` (even if later proven away)". In the SET-1 paragraph:
"Effects exhibited while evaluating the target and right-hand side
contribute normally, and every bounds-checked target `index` syntactically
exhibits `traps` even when its runtime check is later proof-eliminated."
becomes "Effects exhibited while evaluating the target and right-hand side
contribute normally; an accepted target `index` is discharged [OP-4] and
contributes no `traps`."

**[SET-1]** "…at each `index<T>(base, offset)`, `base` is evaluated before
`offset`, and its retained [OP-4] bounds check executes before evaluation
continues. Field suffixes introduce no runtime evaluation. If target
evaluation traps, `e` is not evaluated and no store occurs." becomes "…at
each `index<T>(base, offset)`, `base` is evaluated before `offset`, and the
index's [OP-4] discharge obligation is judged at that target place exactly
as in read position, so accepted target evaluation executes no runtime check
and cannot trap. Field suffixes introduce no runtime evaluation."

**[DIAG-2]** The disposition sentences become: "Every potentially removable
implicit source-language check has exactly one disposition: `retained`, or
`eliminated` with a deterministic checker derivation or separately verified
proof that authorizes that exact elimination. A source `index` place carries
no implicit check and no such disposition: an accepted index is
`discharged`, and the checked program retains its exact [ENT-4] derivation
for that node. An explicit [OP-5] check and every [CLM-1] claim are always
`retained`; the checked program retains each claim's name, predicate, and
justification STRING. In facts-off compilation every required
source-language check is `retained`, and the [ENT-1] discharge judgment is
identical in facts-on and facts-off compilation."

**[DIAG-3]** The node_path sentence becomes: "`node_path` identifies the
source production that introduced the failing checked condition: the
`check_stmt` for [OP-5], the `claim_stmt` for [CLM-1], and the operation
`call` for a table-operation contract check and for the [SYS-8] range
validation judged under [OP-4]'s retained operation-internal semantics."
(The "checked `index` place for [OP-4]" clause is removed: an accepted
source index has no runtime check and produces no trap record.) The message
paragraph gains: "For a [CLM-1] claim, `rule_id` is `CLM-1` and `message` is
the claim's exact IDENT spelling; the justification STRING is compile-time
data and does not appear in the record." (Alternative encoding is open
question O3.)

## 7. Acceptance-set delta and monotonicity

Grows: programs containing `claim_stmt` statements; programs calling
`index_get`. Shrinks: (1) any program with an `index` whose bounds obligation
L0 does not discharge — the dominant migration class; (2) any program whose
declared `traps` was exhibited only by bounds-checked indexes — now a
declared-but-unexhibited [EFF-2] rejection until the row is corrected or a
claim restores the category; (3) any declaration spelled `index_get`
([FORM-3] via the derived reserved set); (4) any identifier spelled `claim`
or `because`. The transition is deliberately non-monotone — it is the
design's one-time dissolution of implicit bounds checks into explicit,
named, reviewable discharge — and the [ENT-1]/[CLM-2] monotonicity law
governs every subsequent version. The current `tests/programs/` corpus
contains no identifier spelled `claim`, `because`, or `index_get` (checked
2026-08-06), so class (3)/(4) migration is empty today.

## 8. Migration note (SIMULATION.md buckets)

Source: `research/investigations/obligation-discharge/SIMULATION.md`
(hand-simulation at exactly this candidate's L0 strength; single-analyst
caveat recorded there). Under batch 1, `requires` stays callee-executed, so
the simulation's "threading tax" clauses are ordinary executed prologue
additions (cost parity with today), and its branch-bucket sites are already
branch-shaped in the corpus (no edit).

- `tests/programs/utf8parse.wf` (44 sites, 8 of them test assertions): 25
  L0-proven; add 2 loop-head claims (the `i <= source_length` /
  `count <= i` pair), which with the existing requires axiom cover the 11
  remaining body sites by transitivity; the 8 test assertions remain `check`
  statements unchanged. Hot loop: 2 checks/iteration today -> 2 claim checks
  (parity at L0).
- deflate-dynamic unit (`raw_deflate.wf` helpers + `raw_deflate_dynamic.wf`
  + `raw_deflate_dynamic_decode.wf`, dynamic path, 30 sites): 17 L0-proven
  (6 by pre-existing Err guards, i.e. branch discharge with zero edits); add
  ~8 claims; the 4–5 guard regions stay as the branches they already are;
  add 5–7 requires clauses to helpers (`store_dynamic_length`,
  `build_huffman_table`, `copy_distance` depth-3 specimen) so their bodies
  gain entry facts.
- `tests/programs/sha256_abc.wf` (9 sites, 1 test assertion): 0 L0-proven;
  add 3 claims, one being the loop-head claim
  `16 <= extend_index < 64` covering all five schedule accesses (hottest
  loop: 5 checks/iteration -> 1 claim check at L0).
- Effect rows: every function whose `traps` came only from bounds-checked
  indexes and which ends up fully proven must drop `traps`; functions
  gaining claims keep it. This row churn is part of the same migration
  change.
- The remaining corpus programs (`wfgrep.wf`, `raw_deflate_vectors.wf`,
  `percent_decode.wf`, and the rest) were not classified by the simulation;
  migration requires a per-program pass, and this candidate makes no bucket
  claim about them.

Conformance corpus: bounds-trap expectations for source `index` sites become
either discharged-accept expectations or CLM-1 claim-trap expectations;
changing any protected verdict requires owner agreement and an
approval-ledger entry per standing law (no such edit is made or implied by
this candidate).

## 9. Acceptance criterion

Batch 1 is implemented correctly only when the real checker, run at exactly
this candidate's L0 strength on the three simulation programs after the §8
migration edits, reproduces the simulation's classification: per-site
proven / claim / branch buckets matching SIMULATION.md's table for
`utf8parse` (25 proven, 2 claims covering 11 sites, 0 forced branches),
the deflate-dynamic unit (17 proven, ~8 claims, 4–5 branch regions), and
`sha256_abc` (0 proven, 3 claims covering 8 sites), with every rejection at
an undischarged site printing a one-line residual per [ENT-6]. Divergence in
either direction is investigated before activation: a site the checker
proves that the simulation did not is re-derived by hand against ENT-2..5
(the simulation may be wrong; the borderline judgments it flagged are
path-sensitive match-join facts and kill granularity through `&uniq` calls),
and a site it fails that the simulation proved is a defect in this text or
the implementation. `make -C compiler check` and `make check` gate the
activation change as always.

## 10. Explicitly out of batch 1

Per DOSSIER §8 ordering: arithmetic `.trap`/`.checked` mode dissolution
(§2.9 — OP-2 rows unchanged here), requires-as-goal (FN-8 entry execution
unchanged), `ensures`, the taint/subject-position gate and signature
provenance column, boundary-op count postconditions (SYS-8 unchanged),
counted range loops, loop induction, struct/witness invariants, partition
policies and ledger tooling, and any warning-bytes normalization. Each needs
its own candidate and evidence.

## 11. Open questions requiring owner ruling

- O1. Version number and path: does this batch become
  `kernel-spec-v0.21-candidate.md` (path currently unoccupied), and does the
  owner want the full-document candidate produced from this delta before or
  after the compiler grammar-path extension (§3 sequencing)?
- O2. Total-form spelling: `index_get` (proposed: dotless IDENT-domain table
  op, zero token changes, OP-7-consistent since totality is not a mode axis)
  vs `index.get` (requires adding `get` to FORM-3/GRAM-1's closed OPNAME
  suffix set: token change, reserved mode-word `get`, wider blast radius) vs
  a different spelling.
- O3. Claim trap-record bytes: proposed `rule_id` `"CLM-1"` and `message` =
  claim name, keeping the four-field DIAG-3 schema byte-stable. Alternative:
  a fifth `"claim"` field carrying the name with `message` carrying the
  justification. The dossier's "operand values in the record" (§2.3) is not
  representable in the current schema at all; carrying operand values is a
  larger DIAG-3 amendment this candidate does not draft.
- O4. Rule-family naming and placement: CLM/ENT names, one new §18 with the
  worked example and meta-rules renumbered to §19/§20 (proposed) vs folding
  claim rules into the OP family beside OP-5.
- O5. Advisory (warning) status: this is the language's first non-rejecting
  required diagnostic. Proposed: required to exist, channel and encoding
  implementation-owned until ledger tooling lands. Alternatives: fully
  normative bytes now, or downgrade to recommended.
- O6. Claim-name discipline: per-function uniqueness outside every TYPE-6
  domain and outside OP-1's reservation inventory (proposed) vs unit-wide
  uniqueness or reservation coverage.
- O7. `check` beside `claim`: batch 1 keeps OP-5 and FN-8's final check
  unchanged; the dossier's end-state has claim as the sole trap source. Is a
  deprecation trajectory for bare `check` wanted, and should test assertions
  migrate to claims now or later?
- O8. Contradictory (unreachable-in-truth) fact states: proposed
  discharge-everything / refute-nothing / redundancy-advisory [ENT-4].
  Alternative: a dedicated unreachable-code diagnostic.
- O9. Index-offset operand typing: this candidate expresses the obligation
  over the offset's mathematical value and changes no operand-typing rule; I
  found no v0.20 rule fixing the offset's exact required type (the `index`
  production types only the element). If a rule fixes it elsewhere, [ENT-6]
  should cite it; if not, is fixing it (presumably u64) wanted in this batch?
- O10. Midpoint-family scope: exactly the two defining shapes of S8
  (`ishr.wrap` by literal 1; `idiv.trap` by literal 2), unsigned T only.
  Confirm this closed list, or name additional shapes now.
- O11. Comparison-origin indirection: exactly one `let` step, no
  `band`/`bor`/`bnot` composition (proposed L0 cut). Confirm.
- O12. Boundary adapters: batch 1 preserves FN-8 trap semantics at gated
  entries; the dossier's §2.8 error-protocol return for foreign callers is
  deferred to the requires-as-goal batch. Confirm the deferral.
- O13. EFF-2 row churn (§7 class 2, §8) is a deliberate consequence of
  removing index `traps` in this batch rather than deferring that clause
  until arithmetic dissolution. Confirm.
- O14. Residual rendering: the fixed whole-obligation schema of [ENT-6]
  (proposed) vs a reduced-frontier residual (rendering the nearest missing
  difference bound after transitive reduction), which is more informative
  but requires a canonical-frontier selection rule this candidate does not
  draft.

No genuine contradiction between DOSSIER §8 items 1–3 and v0.20 was found
during drafting: every collision (OP-4's implicit checks, EFF-2's index
clause, DIAG-3's index row, SET-1's target-trap sentence, FN-8's
check-elision sentence) is a deliberate, enumerated modification above
rather than an ambiguity. The nearest tension is O3 (dossier wants operand
values in the trap record; DIAG-3's fixed schema has no home for them) and
O9 (offset typing unstated in the studied text).
