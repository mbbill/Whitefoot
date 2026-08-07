# Obligation-discharge batch 1 — specification-change candidate

Status: CANDIDATE, APPROVED AT SITTING (drafted 2026-08-06; owner rulings
of 2026-08-07 applied, §12; adversarial-review fixes F2–F11 applied
2026-08-07, see
`research/investigations/obligation-discharge/CANDIDATE-REVIEW.md` and the
revision-pass notes in §12; owner approval at the 2026-08-07 sitting
adopted the second [OP-1] modification and admitted this candidate to the
activation pipeline). Non-authoritative: language authority arises only
from the `docs/WORKFLOW.md` step-4 exact-byte approval of the full v0.21
candidate generated from this delta, and its activation. This document is the
complete batch-1 delta of the obligation-discharge design against the exact
text of `spec/kernel-spec-v0.20.md`. It authorizes nothing: per
`docs/WORKFLOW.md`, activation requires owner approval of exact bytes, and per
`compiler/README.md` a grammar-extending candidate additionally requires the
native grammar path to be extended first (verifier evidence in §3). Scope is
items 1–3 of `research/investigations/obligation-discharge/DOSSIER.md` §8 —
the claim construct, the normative L0 entailment fragment, and caller-side
discharge for OP-4 index bounds — plus, folded in on lead direction
2026-08-06, the count-bound core of item 4 as Section D (§7): the SYS
count-bound postconditions surveyed by `SYS-POSTCONDITIONS.md` (commit
a926f13), riding in this batch per ruling O15 with final wording settled at
the approval sitting with the BOUND-1 owner. Everything else
in that dossier — arithmetic-mode dissolution, requires-as-goal, ensures, the
taint gate, counted range loops, partitions and ledger tooling — is
deliberately out of this batch (§11).

Base: `spec/kernel-spec-v0.20.md` (REVIEW CANDIDATE v0.20 bytes as of
2026-08-06). Per ruling O1 the target version is v0.21, and the
full-document candidate at `kernel-spec-v0.21-candidate.md` is generated
from this delta after the compiler grammar-path extension (§3 sequencing).
The independent adversarial review landed 2026-08-07 (commit ccffbd0); its
mandatory soundness fix F2, acceptance fixes F3–F8, and editorial batch are
applied in this revision.

## 1. Proposed version-header paragraph

The following paragraph is drafted for the eventual numbered candidate's
status header, in the v0.20 header conventions.

> Status: REVIEW CANDIDATE v0.21 (2026-08-07; obligation-discharge batch 1:
> the claim statement, the normative L0 entailment fragment, and caller-side
> discharge of index bounds). Adds one named runtime-check statement —
> `claim name: e because "text";` — whose semantics are exactly [OP-5]'s
> check-else-trap plus a name carried by the [DIAG-3] trap record and a
> lifecycle version-monotone in the redundancy direction: a claim whose
> predicate the checker already
> proves is a non-rejecting redundancy advisory, a claim whose predicate the
> checker refutes is a hard error — the lifecycle's one deliberate
> non-monotone edge [ENT-1] — and a fired claim is surfaced for
> reclassification as a toolchain contract [CLM-1, CLM-2]. Adds the closed
> deterministic L0 entailment fragment as normative acceptance machinery
> [ENT-1..ENT-6]: difference-bound facts over tracked places, length terms,
> and constants; fact sources exactly enumerated (dominating branch and match
> facts, check and claim facts, FN-8 requires facts by clause-local
> substitution, copy/conversion equalities, allocation-length equalities,
> constant-offset wrap/trap/checked arithmetic, the unsigned midpoint family,
> const-array element ranges, implicit type ranges); one least-fixed-point
> difference-bound closure
> with disequality strengthening; and kill, join, and no-induction loop rules
> driven by [OWN-7] resolved-place overlap and [EFF-2] effect-row
> projection, with scope-exit kills ordered before joins.
> Rewrites [OP-4]: a source `index` compiles with no runtime bounds check
> exactly when the fragment discharges its bounds obligation at that node,
> and an undischarged index is a compile-time rejection whose diagnostic
> prints the residual obligation; the index offset atom's exact type is
> fixed as `own u64`, stating for the first time a rule prior text left
> unstated; the total read form `index_get` returning
> `Option<T>` is added to the operation table; the [SYS-8] range-validation
> trap is unchanged as an operation-internal contract check. [FN-8]'s
> foreign-entry execution is restated as one toolchain-synthesized boundary
> adapter at gated entries, trap semantics unchanged. Makes the [SYS-8]
> one-attempt count bounds and three [SYS-9] length and index relations
> normative checker-visible postconditions in the [SYS-12] retained-fact
> form, and admits the four count bounds as [ENT-3] boundary fact source
> S10 in the same trust class as allocation-length equality; the three
> [SYS-9] relations are retained facts with no L0 consumer in this version.
> Specification delta:
> numbered rules +8/-0 (CLM-1, CLM-2, ENT-1, ENT-2, ENT-3, ENT-4, ENT-5,
> ENT-6); sixteen existing rules modified: FORM-2 (claim_stmt is
> line-bearing),
> FORM-5 (STRING homes), GRAM-4 (claim_stmt production; stmt gains one
> alternative), GIVE-1 (claim is non-delivering), OP-1 (index_get row;
> derived reserved sets grow by one; non-consuming place-operand reads
> stated for `len`, `slice_of`, and the `index`/`index_get` base), OP-4
> (rewritten to discharge-or-reject;
> offset atom fixed as `own u64`), FN-1 (a passed `claim` gains its normal
> edge in the conservative structural graph),
> FN-8 (passed fact feeds ENT-3; synthesized boundary adapter), EFF-2 (traps
> contribution: bounds-checked index out, claim in), SET-1 (no runtime check
> in target evaluation), DIAG-1 (claim-name carrier class added to the
> closed taxonomy), DIAG-2 (discharged disposition; claims always
> retained), DIAG-3 (claim trap record; index-place row removed), SYS-8
> (successful-count bounds stated as postconditions, replacing the
> target-facing sentence in place, and the range-validation cross-reference
> repointed), SYS-9 (the arg_get index relation and
> the two host-string length relations stated for the first time). Tokens
> +2/-0 (`claim`, `because` as exact fixed lowercase grammar atoms; terminal
> predicates 75 -> 77); terminal spellings +2/-0; grammar productions +1/-0
> (`claim_stmt`); exception clauses +0/-0; source constructs +1 (the claim
> statement); operation table +1 row (`index_get`), growing the derived
> `DotlessOperationNames` and `ReservedLowerNames` sets by one member each;
> sections +1 (new §18, worked example and meta-rules renumbered; no existing
> cross-reference names either renumbered section). The accepted-program set
> changes in both directions and this transition is deliberately non-monotone
> (§8): it grows by claim statements and `index_get` calls, and it shrinks by
> five classes — an `index` whose bounds obligation the fragment does
> not discharge, an effect row whose `traps` was exhibited only by
> bounds-checked indexes, a declaration spelled `index_get`, identifiers
> spelled `claim` or `because`, and an `index` whose offset atom is not
> exact `own u64` (the newly stated typing rule). From this version forward
> the [ENT-1]
> monotonicity law governs: checker strengthening may only convert claims to
> advisories and undischarged obligations to discharged ones, with one
> enumerated exception — a strengthened fragment may newly refute a claim
> under [CLM-2], rejecting a program thereby proven to trap on every
> execution reaching it. Selection
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

[GRAM-4]'s statement block becomes (one changed line — the `stmt`
continuation line — plus one added production; `check_stmt` is quoted
unchanged for context, and every other line is byte-identical):

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
construct references it; its [DIAG-1] carrier classification is the
claim-name carrier that rule's modification adds (§6). Because the name is
outside the reservation inventory, a claim may be named `len` or `wrap`,
while `trap`, `claim`, and every other exact fixed lowercase grammar atom
remain unwritable as IDENT [FORM-3] — a chosen asymmetry (ruling O6), not
an accident. Within one `fn_decl` every claim name is unique; a
repeated spelling is a hard error citing CLM-1 at the later `claim_stmt`
node. The `because` STRING is the claim's justification: mandatory
compile-time review data retained by the checked program [DIAG-2], absent
from runtime behavior, and never semantics-selecting. A claim is legal in
exactly the statement positions [GRAM-4] admits; [FN-8]'s structural pass
continues to admit only ordinary lets and one final check, so a claim cannot
appear in a `requires` block. This version defines no taint judgment: no
predicate is illegal by operand provenance (the subject-position gate is a
later batch with its own delta).

[CLM-2] Claim lifecycle judgments are fixed by the entailment fragment
under [ENT-1]'s monotonicity law, whose one enumerated non-monotone edge is
this rule's refutation. Redundancy and refutation are judged only for a
predicate with comparison origin [ENT-3]; a conforming claim whose
predicate has none — a constructed `True()`, a `band` result — is neither
redundant nor refutable, is accepted, and traps whenever it evaluates false
at runtime, exactly as today's `check` on the same expression. When the
closed fact state at a
`claim_stmt` [ENT-3] derives its predicate [ENT-4], the claim is redundant:
the
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
in this version until ledger tooling lands (ruling O5, §12); the advisory
itself is required to exist.

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
obligation and claim. In a generic function, discharge, redundancy, and
refutation are judged per concrete [FN-2] instantiation, exactly as
instantiations are re-checked as concrete code; a const-generic constant
term is judged at its instantiated value, never symbolically. The fragment
joins the trusted computing base exactly
as the type and ownership checkers do [SCOPE-3]; a wrong derivation is a
compiler defect class, owned by testing, not a language hedge. Version
monotonicity is law with one enumerated exception: a later specification
version may add fact sources and closure rules and may remove none, so
checker strengthening never converts a discharged obligation into an
undischarged one and never converts a claim into a redundancy-ground
rejection — a claim the stronger fragment proves becomes a [CLM-2]
advisory in every later version, never an error. The one exception is
refutation: a strengthened fragment may newly derive a claim predicate's
exact negation and reject under [CLM-2], rejecting a program thereby
proven to trap on every execution reaching that claim. Refutation is the
lifecycle's one deliberate non-monotone edge; no other judgment of this
family may tighten acceptance across versions.

[ENT-2] The fragment judges one function body at a time; no fact crosses a
call boundary except as [ENT-3] source S4 fixes for the body's own `requires`
prologue. A fragment type is one member of the closed integer set [OP-2];
relations are over mathematical values, so relations between terms of
different fragment types are well-formed and are created only by the sources
[ENT-3] admits.

A term is exactly one of: (a) a tracked place — a `place` [GRAM-5] whose root
`pbase` IDENT resolves to any `let_stmt` binding (whichever of the three
right-hand forms — ordinary, `propagate`, or `value_match` — the statement
selects), a `param`, a requires-clause local, any match binder regardless
of its [OWN-13]-derived mode, or a named const [CONST-2], formed with any
number of `psuffix` field selections and `deref` wrappings and no `index`
segment, whose final selected type is one fragment type; (b) a length term
`len(P)`, of fragment type u64, where P is a place formed under the same
restriction whose final selected type is `array<T, N>`, `slice<'r, T>`, or
`buffer<T>`; (c) a constant — the mathematical value of an integer literal or
of an integer-typed named const, or symbolically an in-scope integer-typed
const-generic parameter; or (d) the distinguished zero term Z, used only to
carry constant bounds. Two places are the same term exactly when their root
`pbase` IDENTs resolve to the same declaration event [TYPE-6, DIAG-1] and
their canonical source spellings [FORM-2] are byte-identical; a fresh
binding legally reusing an expired spelling [TYPE-6] is therefore a
distinct term, and distinct spellings are distinct terms even when they
resolve to overlapping storage. Term
identity thus under-approximates aliasing, which is sound for derivation,
while kills [ENT-5] use the resolved-place overlap relation [OWN-7] over
[OWN-5] resolved places, which
over-approximates it.

An atomic fact is one difference bound `t1 - t2 <= c` (t1, t2 terms, c a
mathematical integer) or one disequality `t1 != t2`. Source relations
normalize exactly: `a <= b` is `a - b <= 0`; `a < b` is `a - b <= -1`;
`a = b` is the bound pair `a - b <= 0` and `b - a <= 0`; `a >= b` and
`a > b` swap operands; `a != b` is one disequality. A constant operand folds
through Z: `a <= 7` is `a - Z <= 7`. Implicit facts hold at every program
point: every term t carries the reflexive bound `t - t <= 0`; every term t
of fragment type T carries `t - Z <= max(T)` and
`Z - t <= -min(T)`; every length term over a place of type `array<T, N>`
carries the equality `len(P) = N` (both bounds), with concrete N a constant
and const-generic N a symbolic constant term.

[ENT-3] The fact state is defined constructively over the conservative
structural normal-control graph [FN-1]: each source below establishes its
facts at its stated point; facts flow forward along normal edges; kill
events apply on the edges where [ENT-5] places them, with scope-exit kills
applied before any join; merge points take the [ENT-5] join and loop heads
the [ENT-5] loop rule; and the state queried at any point is the [ENT-4]
closure of that flow. Dominated straight-line establishment is a
consequence of this construction, not a second definition. Nothing else is
a fact: no ensures, struct
invariant, loop induction, user-function postcondition, or taint judgment
exists in this version.

A comparison origin is defined first. An expression has comparison origin R
when (a) it is a call to one of `ieq`, `ine`, `ilt`, `ile`, `igt`, `ige`
[OP-2] whose two operands are each a term or constant, R the corresponding
relation over them; or (b) it is a bare IDENT naming a `let` binding of type
`own Bool` whose initializer right-hand side satisfies (a) with relation R,
no [ENT-5] kill event (a)–(d) applies to a fact supported by an operand
term of R on any path from that initializer to
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
  m = len(P). `let s: own slice<'r, T> = slice_of…(&'r P);` for a tracked P
  establishes len(s) = len(P).
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
  where no [ENT-5] kill event applies to a fact supported by p between the
  initializer and the match and that binding is no `set` target on that
  path, the `Ok(value: w)` arm establishes w = p ± k at arm entry;
  the `Err` arm establishes nothing.
- S8 (the midpoint family). Where a body contains, in this definitional
  shape with T unsigned, lo and hi terms of type T,

  ```
  let d: own T = isub.wrap<T>(hi, lo);
  let h: own T = ishr.wrap<T>(d, 1_u32);
  let m: own T = iadd.wrap<T>(lo, h);
  ```

  (the three lets need not be adjacent; `idiv.trap<T>` of d and the literal
  two of the concrete type T is admitted as
  the alternative defining shape of h), and, between each member's
  definition and m's definition, no [ENT-5] kill event applies to a fact
  supported by any member of {lo, hi, d, h} and no member is a `set`
  target, then: when the
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
- S10 (boundary count facts; Section D, §7). For a
  `match_stmt` or `value_match` whose scrutinee is directly a call to
  `read_once`, `write_once`, `host_copy_bytes`, or `host_copy_utf8`
  [SYS-2, SYS-8], or a bare IDENT naming a `let` binding of the call's
  outcome type initialized by such a call under the same no-kill, no-`set`
  path discipline as S7's checked-arithmetic origin: with k the actual bound
  to the call's bounding parameter — `capacity` for `read_once`,
  `host_copy_bytes`, and `host_copy_utf8`; `count` for `write_once` — read
  as a term or constant, where no [ENT-5] kill event applies to a fact
  supported by k on the path to the match, the
  `ReadBytes(count: w)` arm of a `read_once` match and the `Ok(value: w)`
  arm of the other three establish w <= k at arm entry; every other arm
  establishes nothing. These facts carry the same trust class as S6's
  allocation-length equality — a declared operation contract, never a
  writer statement. The three [SYS-9] relations of Section D are retained
  checked-program facts and are not L0 fact sources in this version (§7).

[ENT-4] The closed fact state at a point is the least set containing its
established and implicit facts and closed under exactly: (1) from
`t1 - t2 <= c1` and `t2 - t3 <= c2`, derive `t1 - t3 <= c1 + c2`; (2) from
`t1 - t2 <= 0` and a disequality between t1 and t2 in either orientation,
derive `t1 - t2 <= -1`; (3) of two bounds on one ordered pair, the smaller
constant subsumes. This least closure is the one definition: it is unique
and finite up to subsumption because only terms written in the function
participate, the rules are monotone, and the least fixed point is reached
in finitely many steps. Derivability is exact: `a - b <= c` is
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
target [SET-1, OWN-5] overlaps, under [OWN-7]'s overlap relation, the
resolved place of any support member;
(b) a call — user function, table operation, or system operation — one of
whose [EFF-2] boundary-projected `writes` occurrences projects onto a caller
place or origin set containing a place that overlaps [OWN-7] the resolved
place of any
support member; the projection is exactly [EFF-2]'s, so a callee writing only
through one `&uniq` actual kills exactly the facts whose support overlaps
that actual's resolved place, and a call whose row carries no `writes` kills
nothing; (c) a consuming use [OWN-1] of any support member's root; (d) an
edge leaving the region of any borrow holder in its support or leaving the
lexical scope of any support binding, region exit [OWN-3] included. Scope
exits are edge events: kills (c) and (d) apply on every edge leaving the
scope, before any join at that edge's target is taken — mirroring
[STOR-3]'s edge-carried releases — so no arm-local or block-local fact
survives its scope into a join under any reading.

Joins: at the continuation of a `match_stmt` or `value_match`, the fact
state is the join of the states on every arm exit edge reaching that
continuation on the conservative structural graph [FN-1], each taken after
that edge's scope-exit kills and then closed [ENT-4]; an arm every path
of which leaves by `return`, `break` to an enclosing loop, or `propagate`'s
error edge contributes nothing there. The join keeps, for each ordered term
pair, the weakest (largest-constant) bound held by all joined states, and
each disequality held by all of them; the join of closed states is closed.
The continuation of a `loop_stmt` is the join over the states on its
`break` edges, each likewise taken after its scope-exit kills and closed. A
`loop_stmt` with no `break` naming its label has an empty join: its
continuation state is the contradictory all-derivable state [ENT-4],
consistent with that continuation being unreachable in truth while the
conservative graph keeps it reachable. A `propagate` right-hand side's
`Err` edge leaves the
function; its normal continuation keeps the preceding state subject to the
initializer call's own kill events (b) and (c), and its binder
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
normalized `i - len(P) <= -1`, at the `index` node, where `i` is the offset
atom whose exact type [OP-4] fixes as `own u64`, so both sides are
u64-typed and the relation is over their mathematical values. The
obligation is
discharged exactly when the closed fact state at that node derives it
[ENT-4, ENT-5]. An undischarged obligation is the [OP-4] rejection; its
diagnostic renders the residual as exactly: the offset atom's canonical
source bytes, then ` < len(`, then the base place's canonical source bytes,
then `)`. The mechanical fix is one dominating claim or branch establishing
the relation — in canonical ANF, one `let` binding `len<T>(P)` followed by
one `claim` on, or `match` over, the admitted comparison [CLM-1, ENT-3].
For an offset atom that is itself an index-bearing place — legal under
[GRAM-5]'s place grammar but no term under [ENT-2] — the fix first rebinds
that inner read through one ordinary `let` (and, where the element type is
narrower than u64, one total `cvt` [OP-6], both S5-tracked), making the
offset a term whose own inner obligation is discharged the same way. With
at most that one rebinding step per nested offset,
the fallback always closes discharge and reproduces the pre-revision
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

The row is appended as the last row of the OP-1 table, which fixes
`index_get`'s derived dotless-family ordinal as the last in [DIAG-1]'s
reservation payloads; the `(place, u64)` signature notation follows
`slice_of`'s existing place-operand row style. No text change:
`DotlessOperationNames`, and therefore `ReservedLowerNames`,
grow by the derived member `index_get`. `index_get` is a table operation with
positional operands [GRAM-11].

**[OP-1] — second modification (reviewer recommendation F11.7; adopted by
the owner at the 2026-08-07 sitting).** One
sentence is added immediately after the operation-family resolution
paragraph (the paragraph ending "Operand types never select between an
operation family, a system operation, and a function."): "A bare `place`
operand that a table-operation row reads without consuming — the `len`
operand, the place viewed by `slice_of` through its explicit borrow, and
the base place of `index` and `index_get` — is a non-consuming read: it
neither moves nor partially consumes an affine root [OWN-1], exactly the
reading [FN-8] already states for a place used as a non-consuming operand
of an admitted table operation." This makes [ENT-6]'s fallback
(`let n: own u64 = len<T>(P);`) well-formed for every affine base by
stated rule rather than by v0.20's latent reading, and makes the [OP-4]
index_get non-consuming sentence a restatement rather than a special
case. With this adoption the candidate total is sixteen modified rules,
counted in the §1 header and in §7's accounting.

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
> `buffer<T>` obligation is over the runtime length term. The offset atom
> has exact value mode and type `own u64`; after the [TYPE-7] implicit-read
> exclusivity, any other offset mode or type is a hard error citing OP-4 at
> the offset `atom` node, with `SourceCoordinate` equal to that atom's
> complete checked half-open source extent. An `index` in a
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
> obligation, and is an expression call, never a place; its place operand is
> a non-consuming read of the base — the affine base is neither moved nor
> partially consumed by the call, exactly as an `index` base or `len`
> operand; it does not admit
> writes.

**[FN-1]** One edge-enumeration sentence of the conservative structural
normal-control graph changes: "An ordinary `let`, `set`, expression
statement, and a passed `check` have a normal edge to
`normal_successor(s)`." becomes "An ordinary `let`, `set`, expression
statement, and a passed `check` or `claim` have a normal edge to
`normal_successor(s)`." — giving `claim_stmt` the normal continuation edge
that [CLM-1], [ENT-3] S3, [EFF-2], and [GIVE-1] rely on (review finding
F6.1).

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

**[EFF-2]** Three edits. In the syntactic contribution: "exhibit `traps` iff
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
contributes no `traps`." In the canonical-example paragraph (the paragraph
beginning "Canonically, a nongeneric function whose only parameter is
`own ReadFile`…"): "Its declaration contains no call, no `check`, no
bounds-checked `index`, and no other syntactic effect occurrence" becomes
"Its declaration contains no call, no `check`, no `claim`, and no other
syntactic effect occurrence" — an assembly-discovered consequential edit
(D1, 2026-08-07): the example's contributor enumeration must match the
amended contributor list, and neither the original delta nor the review
enumerated it.

**[SET-1]** "…at each `index<T>(base, offset)`, `base` is evaluated before
`offset`, and its retained [OP-4] bounds check executes before evaluation
continues. Field suffixes introduce no runtime evaluation. If target
evaluation traps, `e` is not evaluated and no store occurs." becomes "…at
each `index<T>(base, offset)`, `base` is evaluated before `offset`, and the
index's [OP-4] discharge obligation is judged at that target place exactly
as in read position, so accepted target evaluation executes no runtime check
and cannot trap. Field suffixes introduce no runtime evaluation."

**[DIAG-1]** The closed carrier taxonomy gains one class (review finding
F6.2). Inserted into the carrier-taxonomy paragraph, immediately after the
table-checked-carriers sentences (after "… and none participates in
FORM-3's reservation inventory.") and before the X09/U18 sentence: "The
claim-name carrier is exactly the IDENT of a `claim_stmt`
[CLM-1]. It produces one record for CLM-1's per-function uniqueness
judgment; it produces no declaration, lexical-use, dependent-declaration,
deferred-use, or table-checked record, enters and queries no lexical name
domain, and does not participate in FORM-3's reservation inventory." The
X09/U18 only-same-token-overlap sentence is unaffected: the claim-name
carrier is single-role.

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
data and does not appear in the record." (Ruled O3: the four-field schema
is kept; an operand-value DIAG-3 amendment is queued with ledger tooling,
not this batch.)

**[SYS-8], [SYS-9]** — modified by Section D; exact deltas in §7.

## 7. Section D — SYS count-bound postconditions (rides in this batch per ruling O15)

Basis: `research/investigations/obligation-discharge/SYS-POSTCONDITIONS.md`
(commit a926f13; survey of the count-returning [SYS-2] operations in v0.20).
Finding of record: zero of the seven count-returning system operations state
a checkable bound today — the four one-attempt transfers (`read_once`,
`write_once`, `host_copy_bytes`, `host_copy_utf8`) imply `count <= bound` in
prose only, and `args_count`, `host_bytes_len`, and `host_utf8_len` state no
relation at all. These bounds are load-bearing for this candidate's
fragment: PROBE-TAINT.md's wfgrep result (one structural claim in 723 lines,
zero forced branches) assumed exactly `read_once`'s `count <= capacity` and
`host_copy_bytes`'s `copied <= capacity`; without them, cursor arithmetic
downstream of every boundary read floods with environment magnitudes and
[ENT] cannot discharge it. The mechanism adds no new construct: [QUAL-1]
already binds an operation's complete outcome set to its semantic ID, and
[SYS-12]'s retained-redirection fact is the exact stylistic precedent for a
checker-visible fact stated as prose plus [DIAG-2] retention.

Ruling O15 (§12): Section D rides in this batch, and the [SYS-8] sentence is
replaced in place. This section's wording remains drafting material in the
survey's proposed sentences: final text is settled at the approval sitting
in coordination with the owner of the in-flight BOUND-1 system-capability
work, which owns the SYS-family surface. Two rules are modified.

**[SYS-8] modification.** In the "Buffer and cursor disposition is exact."
paragraph, the sentence "A target returning a count outside the validated
range violates its compiler-owned contract; source code does not defend
against it." is replaced in place by the survey's consolidated paragraph:

> Every successful count is bounded by the caller's validated range, and the
> checked program retains that bound as a fact about the returned value
> [DIAG-2]. On `ReadBytes(count)` the count is at most the requested
> `capacity`; on a successful `write_once` the accepted length is at most
> the requested `count`; on a successful `host_copy_bytes` or
> `host_copy_utf8` the copied length is at most the requested `capacity`.
> These are postconditions of the operations, not defensive obligations on
> source: a target returning a larger count violates its compiler-owned
> contract [QUAL-1], and source code neither checks nor branches on that
> possibility.

The existing conditional lower bound is deliberately not flattened: [SYS-8]'s
stated facts — a zero-length range reports a count of zero, and for a
nonempty range `ReadBytes(count)` implies `count > 0` — remain exactly as
written, and this candidate adds no lower-bound fact source.

A second [SYS-8] edit repairs the cross-reference the rewritten [OP-4]
would leave dangling (review finding F9): in the range-validation
paragraph, "traps under the bounds semantics of [OP-4]" becomes "traps as
the operation-internal contract check retained by [OP-4] [ERR-4]".

**[SYS-9] modification.** Three relation sentences are added in the survey's
proposed wording, one per operation, each in the [SYS-12] retained-fact
form:

- `args_count`/`arg_get`: "`arg_get` returns `Ok` exactly when `position` is
  less than the count `args_count` returns for the same `Args`, and the
  checked program retains that relation [DIAG-2]."
- `host_bytes_len`: "That count is exactly the `required` length a
  `host_copy_bytes` on the same host string reports, so a `host_copy_bytes`
  whose `capacity` is at least that count returns `Ok` with exactly that
  count, and the checked program retains that relation [DIAG-2]."
- `host_utf8_len`: "On `Ok(length)`, a `host_copy_utf8` on the same host
  string neither returns `Utf8CopyInvalid()` nor, for a `capacity` of at
  least `length`, returns `Utf8CopyTooSmall(required)`, and the checked
  program retains that relation [DIAG-2]."

**Checker consumption is deliberately split.** The four count bounds are L0
fact sources in this batch: [ENT-3] source S10 admits each at the match arm
observing the successful outcome, in the same trust class as S6's
allocation-length equality — a declared operation contract, never a writer
statement, so [SCOPE-2] and the W3 keystone are untouched. The three
[SYS-9] relations become normative retained facts but are not L0 fact
sources: consuming them requires relating two calls on the same `Args` or
`HostString` value (cross-call congruence), which is outside this batch's
fragment. Until a later version adds that machinery their effect is
[SYS-12]-style fail-closed retention plus review value, and the `Err` arms
they could someday prove dead remain written — match exhaustiveness [ERR-2]
is unchanged. The failing-call companion bound `required > capacity` is
recorded by the survey and proposed by neither it nor this candidate.

Accounting: this section adds no rule, token, or production; it modifies
[SYS-8] (two edits) and [SYS-9] (candidate total: sixteen modified rules
with [FN-1], [DIAG-1], and the adopted second [OP-1] modification, §6) and
extends
[ENT-3] with source S10.

## 8. Acceptance-set delta and monotonicity

Grows: programs containing `claim_stmt` statements; programs calling
`index_get`. Shrinks: (1) any program with an `index` whose bounds obligation
L0 does not discharge — the dominant migration class; (2) any program whose
declared `traps` was exhibited only by bounds-checked indexes — now a
declared-but-unexhibited [EFF-2] rejection until the row is corrected or a
claim restores the category; (3) any declaration spelled `index_get`
([FORM-3] via the derived reserved set); (4) any identifier spelled `claim`
or `because`; (5) any program whose index offset atom is not exact
`own u64` — the ruling-O9 sentence states a rule v0.20 left unstated, and
is expected to formalize implemented behavior (verified at implementation
time). The transition is deliberately non-monotone — it is the
design's one-time dissolution of implicit bounds checks into explicit,
named, reviewable discharge — and the [ENT-1]/[CLM-2] monotonicity law
governs every subsequent version. The current `tests/programs/` corpus
contains no identifier spelled `claim`, `because`, or `index_get` (checked
2026-08-06), so class (3)/(4) migration is empty today.

## 9. Migration note (SIMULATION.md buckets)

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
  add 4 claims — the loop-head pair `16 <= extend_index` and
  `extend_index < 64`, since the simulation's single conjoined claim splits
  into two single-comparison claims under ruling O11, covering all five
  schedule accesses, plus two others (hottest
  loop: 5 checks/iteration -> 2 claim checks at L0).
- Effect rows: every function whose `traps` came only from bounds-checked
  indexes and which ends up fully proven must drop `traps`; functions
  gaining claims keep it. This row churn is part of the same migration
  change.
- The remaining corpus programs (`wfgrep.wf`, `raw_deflate_vectors.wf`,
  `percent_decode.wf`, and the rest) were not classified by the simulation;
  migration requires a per-program pass, and this candidate makes no bucket
  claim about them.
- The [OP-4] offset-typing sentence (ruling O9) is expected to require no
  corpus edit: it states the operand type the implemented compiler already
  uses, which the implementation pass confirms.

Conformance corpus: bounds-trap expectations for source `index` sites become
either discharged-accept expectations or CLM-1 claim-trap expectations;
changing any protected verdict requires owner agreement and an
approval-ledger entry per standing law (no such edit is made or implied by
this candidate).

## 10. Acceptance criterion

Batch 1 is implemented correctly only when the real checker, run at exactly
this candidate's L0 strength on the three simulation programs after the §9
migration edits, reproduces the simulation's classification: per-site
proven / claim / branch buckets matching SIMULATION.md's table for
`utf8parse` (25 proven, 2 claims covering 11 sites, 0 forced branches),
the deflate-dynamic unit (17 proven, ~8 claims, 4–5 branch regions), and
`sha256_abc` (0 proven, 4 claims covering 8 sites — SIMULATION.md's frozen
"3 claims" row counts its conjoined loop-head claim once, where ruling
O11's single-comparison rule requires two, and is read accordingly), with
every rejection at
an undischarged site printing a one-line residual per [ENT-6]. Divergence in
either direction is investigated before activation: a site the checker
proves that the simulation did not is re-derived by hand against ENT-2..5
(the simulation may be wrong; the borderline judgments it flagged are
path-sensitive match-join facts and kill granularity through `&uniq` calls),
and a site it fails that the simulation proved is a defect in this text or
the implementation. `make -C compiler check` and `make check` gate the
activation change as always.

## 11. Explicitly out of batch 1

Per DOSSIER §8 ordering: arithmetic `.trap`/`.checked` mode dissolution
(§2.9 — OP-2 rows unchanged here), requires-as-goal (FN-8 entry execution
unchanged), `ensures`, the taint/subject-position gate and signature
provenance column, counted range loops, loop induction, struct/witness
invariants, partition policies and ledger tooling, and any warning-bytes
normalization. Also out per ruling O7: any `check` deprecation and any
test-assertion migration to claims — deferred to the FLOOR-5 spelling
batch. Within Section D's own area, also out: cross-call-congruence
consumption of the three [SYS-9] relations, any lower-bound fact source,
and the failing-call `required > capacity` companion bound. The operand-
value DIAG-3 amendment is queued with ledger tooling (ruling O3). Each
needs its own candidate and evidence.

## 12. Resolved rulings (owner, 2026-08-07)

All sixteen questions raised by the 2026-08-06 draft were ruled on
2026-08-07; every ruling selects the draft's proposed option unless noted.
The rulings are applied throughout this document.

- O1 — Ruled: target version v0.21; the full-document candidate at
  `kernel-spec-v0.21-candidate.md` is generated from this delta after the
  compiler grammar-path extension (§3 sequencing).
- O2 — Ruled: `index_get`, dotless IDENT-domain table operation; no OPNAME
  suffix change.
- O3 — Ruled: four-field DIAG-3 schema, `rule_id` `"CLM-1"`, `message` =
  claim name; the operand-value DIAG-3 amendment is queued with ledger
  tooling, not this batch.
- O4 — Ruled: CLM/ENT family names and the new §18 placement with §19/§20
  renumbering, as drafted.
- O5 — Ruled: the redundancy advisory is required to exist; channel and
  encoding are implementation-owned until ledger tooling lands.
- O6 — Ruled: per-function claim-name uniqueness, outside every TYPE-6
  domain and outside OP-1's reservation inventory, as drafted.
- O7 — Ruled: `check` kept unchanged in this batch; deprecation and
  test-assertion migration are deferred to the FLOOR-5 spelling batch.
- O8 — Ruled: contradictory states discharge every obligation and refute no
  claim, as drafted [ENT-4].
- O9 — Ruled: the offset-typing fix lands in this batch — the offset atom
  is exact `own u64`, stated in the [OP-4] replacement text (§6) and cited
  from [ENT-6].
- O10 — Ruled: the S8 midpoint family is exactly the two defining shapes
  (`ishr.wrap` by literal 1; `idiv.trap` by literal 2), unsigned T only.
- O11 — Ruled: comparison-origin indirection is exactly one `let` step; no
  `band`/`bor`/`bnot` composition at L0.
- O12 — Ruled: boundary adapters preserve FN-8 trap semantics; the dossier
  §2.8 error protocol waits for the requires-as-goal batch.
- O13 — Ruled: the EFF-2 row churn (§8 class 2, §9) is accepted as part of
  this batch's migration.
- O14 — Ruled: the fixed whole-obligation residual schema of [ENT-6], as
  drafted.
- O15 — Ruled: Section D rides in this batch; the [SYS-8] sentence is
  replaced in place; final wording is settled at the approval sitting with
  the BOUND-1 owner.
- O16 — Ruled: the three [SYS-9] relations land now as retained facts with
  no L0 consumer; no lower-bound fact source in this batch.

No genuine contradiction between DOSSIER §8 items 1–3 plus Section D and
v0.20 remains: every collision (OP-4's implicit checks, EFF-2's index
clause, DIAG-3's index row, SET-1's target-trap sentence, FN-8's
check-elision sentence, SYS-8's target-facing sentence and range-validation
cross-reference, FN-1's edge enumeration, DIAG-1's closed carrier taxonomy)
is a deliberate, enumerated modification above rather than an ambiguity.
The first draft missed the last three; the adversarial review caught them
(findings F6 and F9) and they are now enumerated. The two tensions the
draft flagged are both closed by ruling: O3's operand-value record
amendment is queued with ledger tooling, and O9's previously unstated
offset typing is now stated in [OP-4].

### Revision-pass notes (2026-08-07, post-review)

Fixes F2–F8 and the editorial batch of
`research/investigations/obligation-discharge/CANDIDATE-REVIEW.md` (commit
ccffbd0) are applied throughout. Choices the review left open are adopted
here explicitly rather than silently:

1. The empty-join value for a break-less `loop_stmt` continuation is the
   contradictory all-derivable state [ENT-4, ENT-5]. The review proposed
   this with "presumably"; it is now normative candidate text and should be
   confirmed at the approval sitting.
2. `index_get`'s place operand is stated non-consuming explicitly (§6
   OP-4), while `len`, `slice_of`, and the `index` base inherit v0.20's
   latent unstated non-consuming reading that [ENT-6]'s fallback
   (`let n: own u64 = len<T>(P);`) relies on. The one-sentence
   clarification was drafted as the second [OP-1] modification (§6) per the
   reviewer's recommendation and adopted by the owner at the 2026-08-07
   sitting, closing the latent reading and making sixteen modified rules.
3. F2 is repaired by both of the review's independent repairs — scope-exit
   kills ordered before joins as edge events, and declaration-anchored term
   identity — not only the mandatory first.
4. The sha256 acceptance bucket is restated as 4 claims and the hot-loop
   static count as 2 claim checks per iteration at L0 (F8, ruling O11).
   SIMULATION.md is frozen research and is deliberately not edited; the
   divergence is explained at the two places the buckets are used (§9,
   §10).
5. F4 is worded per lead direction: redundancy monotonicity is absolute,
   and CLM-2 refutation is the one enumerated acceptance-tightening edge
   [ENT-1]; the review's alternative (cross-version refutation as advisory)
   was not taken.
6. Re-verification (2026-08-07, commit 9a4ff9f) confirmed every finding
   closed, found no new soundness or acceptance issue, and concurred with
   the five choices above. The subsequent polish pass scoped the §1
   header's monotonicity sentence with the refutation exception, named
   [ENT-6]'s rebinding step for nested-index offsets, gave the [DIAG-1]
   carrier insertion its exact anchor, and drafted the second [OP-1]
   modification (§6), which the owner adopted at the sitting.
