# FLOOR-5 spelling relief — batch candidate (SWEEP A + C)

Status: CANDIDATE, DRAFT (2026-08-07; owner overnight standing instruction —
draft-and-review lane only, activation sequenced later; adversarial-review
fixes F1–F11 and residue findings R1–R3 of the FLOOR-5 review applied per
lead direction, see `research/investigations/obligation-discharge/
CANDIDATE-REVIEW.md` at 710f4b3). Non-authoritative.

Sweep completeness (2026-08-07, prose-sweep repair). The 46-site revision
covered the rules this batch *modifies* but not every rule whose normative
prose *uses* a respelled operation, so fifteen sites were missing. The
whole active spec has now been re-swept mechanically for the miss class —
every lowercase operation name followed by `<`, every dotted OPNAME
spelling, every `index<`, every annotated `let` form, every enumeration of
`let`-RHS or conditional node kinds, and every arm-scoped or
control-graph prose site — across all 128 rules rather than the modified
ones (method and cleared near-misses in §7). Two of the fifteen were
outside the reported list and are load-bearing: [FN-1]'s conservative
structural control graph gives `if_stmt` and a `value_if` `let` no edges
at all, and [OWN-5]'s slice-join prohibition is keyed on a declared type
A3 deletes and reaches only `value_match`.

Seventh sweep pattern (2026-08-07). The six patterns above are lexical:
each keys on a spelling, a production form, or a node-kind enumeration.
Normative prose that names a deleted construct *in words* — "explicit
type argument", "every binding states its exact type", "the base place's
already-stated type", "one comparison call" — carries none of those
bytes and is unreachable from any of them. That class was swept
separately (§7), against all four constructs this batch deletes: the
written type argument, the `let` annotation, the Bool-scrutinee `match`,
and — the one an annotation-focused probe forgets — the *call spelling*
of the ten families C1 respells as infix.

It found three further sites, [OP-4], [SYS-13], and [OWN-13], and two
call-spelling anchor extensions that add no site, [ENT-3] S4 and
[DIAG-3]. One of those, [ENT-3] S4, is an unlisted narrowing on the
[FN-1] scale rather than a characterization defect: S4 fires only on
"one comparison call", so after C1 no `requires` prologue establishes
its entry fact and every dependent obligation is rejected under [OP-4]
(**22** files carry `requires` blocks on the migration basis —
`tests/conformance` and `tests/programs`, the corpora maintained at the
active spec version — and 40 more sit in the two frozen corpora,
`tests/codegen/` 37 and `research/experiments/` 3, for 62
repository-wide. Task 0031 left both frozen unrespelled and both still
spell the `index<T>(p, i)` form v0.22 deleted, so neither migrates with
this batch; they reconcile per case at promotion, and their exposure
surfaces there rather than at migration. Two withdrawn figures are
recorded in §3: a 122 that counted every source twice through the
worktree mirror, and a 25 that wrongly counted the frozen research
sources into the migration basis). It also found the one
defect in this candidate's own text: [OP-2] site (g) retained a mandate
that the FN-2 missing-type-argument judgment applies, after site (f)
deleted the only sentence defining it — with a third such FN-2 naming
carried into [DIAG-1]'s own replacement text. It cleared, by inspection
of the active file, three sites reported to exist and absent from
v0.22 — a [TYPE-4] `let` judgment, an [OWN-1] declared type, and a
[STOR-1] declared binding type (§7).

This document is the complete spelling-relief delta against the exact text
of the active `spec/kernel-spec-v0.22.md` (installed 817a8a7). Authority:
`research/investigations/spelling-relief/SWEEP.md`, implementing its A and
C verdicts as scoped by the lead: A1 (value-op type-argument deletion with
a total retained-argument class), A3 (body-let annotation deletion), A4
(Bool-scrutinee match becomes `if`/`else` with the SWEEP-pinned layout), C1
(infix respellings, modes as operator suffixes, semantics unchanged), and
C3 (check-to-claim unification) which is assessed NOT pure spelling under
v0.22 and deferred open (O2), reviewer-confirmed. The drafted production
set passed a mechanical SELECT-set disjointness check before this revision
was reported (§2); the review's process note is honored, not deferred.

Two structural findings remain surfaced up front:

- Bare `<` and `>` comparisons collide with call type arguments at
  strong-LL(2)'s two-token horizon (`(IDENT, "<")` begins both `a < b`
  and `f<T>(…)`), and both bytes sit in FORM-2's attachment sets. C1
  ships with `==` `!=` `<=` `>=` infix and `ilt`/`igt` as named calls;
  the fork is O1, with the rejected alternative's cost now stated
  accurately per the review (call-targs-only introducer — small, not
  "every generic call and type").
- A3 removes GIVE-1's declared-type anchor. Per review R3 and ruling O4,
  the replacement — the derived common delivery type — ships as a
  fully-worked rule (complete GIVE-1 replacement in §3), covering the
  agreement judgment, the empty-delivery-set rejection (F4), and the
  else-if chain's delivery semantics (F5).

## 1. Proposed version-header paragraph

> Status: REVIEW CANDIDATE vNEXT (2026-08-07; FLOOR-5 spelling relief:
> value-op type-argument deletion, body-let annotation deletion,
> `if`/`else` for Bool conditionals, infix arithmetic and comparison
> spellings). Deletes the written type argument from every table
> operation outside the closed retained-argument class — the operand
> atoms are typed, so the selected type is uniquely reconstructed per
> [OP-2]'s rewritten derivation; exactly `cvt`, `reinterpret`,
> `array_new` (type and const), `arena_new` (region and type), and
> `finf`/`fnan` (type) keep written arguments, everywhere and
> mandatorily, because no operand can supply them. Deletes the `: mode
> type` annotation from every `let` binder — the binder's mode and type
> are exactly what its right-hand side produces, statement-locally
> ([TYPE-5] rewritten; literals keep mandatory suffixes [FORM-5], so
> every right-hand side stays self-typed); a value initializer's type is
> the derived common delivery type ([GIVE-1] rewritten in full: exact
> agreement over the closed delivery set, an empty delivery set
> rejected, and an else-if chain delivering to the chain's binding).
> Replaces the Bool-scrutinee `match` with `if`/`else`: a Bool condition
> takes `if`, an enum scrutinee takes `match`, each the sole form for
> its class; an `if_stmt` `else` with an empty block is rejected (spell
> the else-free form; the empty then-block is admitted — both follow
> from the else-free form being the one spelling of the empty
> alternative), an `else` whose block is exactly one `if_stmt` must
> flatten to `else if` (universal; the undeliverable `value_if` case is
> [GIVE-1]'s rejection, on which GRAM-6 forms no candidate), and the
> canonical layout is
> multi-line with `} else {` and `} else if … {` join lines, governed
> solely by [FORM-2]'s dedicated `if` sentence, no one-line form.
> Respells the hottest integer table operations as infix with modes as
> operator suffixes — bare `+` `-` `*` `/` `%` carry the trapping-mode
> semantics unchanged, `+wrap`-class suffixed operators carry
> wrap/checked/sat, and `==` `!=` `<=` `>=` respell the four nonstrict
> comparisons — one constant spelling per operation as today, no
> precedence table because [GRAM-9] ANF admits exactly one operation
> per expression, and the `expr` grammar left-factored so the decision
> is strong-LL at one token after the shared atom prefix. `ilt` and
> `igt` remain named calls (O1). The `if` continuation is an enumerated
> [ENT-5] merge point in the same CFG idiom as `match`, with the empty
> join defined as the contradictory state for both, so branch facts
> join exactly as the match
> spelling joins today. Specification delta: numbered rules +0/-0;
> thirty-four existing rules modified at sixty-four verbatim-anchored
> modification sites (a site is one contiguous verbatim-anchored
> replacement; every site in this candidate is anchored — no prose
> sweeps. The count is of anchor/replacement pairs in §3, not of delta
> line items: a reviewer counting the pairs reaches sixty-four, and a
> site whose replacement edits several sentences of one contiguous
> paragraph — [OP-2] (b) two, [OP-2] (g) three, [ENT-3] S4 two,
> [DIAG-3] two — is still one site):
> FORM-2 (block-bearing list; the sole-governance `if`
> rendering sentence; value-if prefix line), FORM-3 (the OPNAME
> example respelled to a surviving OPNAME), GRAM-1 (four sites: six
> compound
> tokens; operator-form munch with the minus/arrow/literal
> disambiguation; the shape-kind enumeration gains the operator form;
> the `infix` node-kind sentence), GRAM-4
> (annotation-free `let_stmt`; `if_stmt` and `value_if`), GRAM-5
> (left-factored `expr` with `infix_tail` and `infix_op`; call
> targs retained for the classes that keep them), GRAM-6 (rewritten:
> type-driven conditional forms; the universal flattening mandate with
> [GIVE-1] ownership of the undeliverable `value_if` case), GRAM-7
> (rewritten: two `if` node kinds beside the two
> `match` kinds), GIVE-1 (complete replacement: the derived-delivery
> rule), GRAM-9 (two sites: infix operands; the forwarding-let
> parenthetical), TYPE-5 (rewritten: statement-local derivation; the
> total retained-argument class; boundaries stay fully explicit),
> OWN-5 (the slice-valued-join prohibition rekeyed to the derived
> delivery type and extended to `value_if`), OWN-13 (the value-initializer
> ownership sentence rekeyed off the deleted `let` annotation and
> extended to `value_if`, keeping its T1 single-move argument),
> STOR-2 (`box_new` loses
> its argument), STOR-5 (the region-bearing-content substitution
> sentence, whose `box_new` half loses the `targ` its diagnostic was
> anchored at), OP-1
> (five sites: row selection reworded off the written argument;
> op-column respells; infix resolution by exact operator
> token; `ModeWords` derived from both suffix carriers; the
> reservation rule's `let_stmt` kind list gains the value-if let),
> OP-2 (seven
> sites: the three arithmetic and one comparison semantics paragraphs
> respelled infix; the negation paragraph de-argumented; the div/rem
> mode clause named off the deleted `.trap`/`.checked` spellings;
> operand-derived selected type replaces the explicit-argument
> judgment, binary and negation paragraphs), OP-4 (the subscript
> element-type derivation re-keyed off the base place's "already-stated"
> type), OP-7 (three sites: infix
> convention; two keyed-on-the-selected-type rewrites), OP-8 (two
> sites plus one retention: sat respells; the contiguous `eeq`/`ene`
> operand-derived identity; `fneg(finf<T>())` retained), OP-9
> (`buffer_new` loses its argument), ERR-2
> (Bool exhaustiveness via `if`, with the empty-then/empty-else
> asymmetry stated), ERR-3 (full-sentence re-anchor with the derived
> type), FN-1 (the conservative structural normal-control graph gains
> the `if_stmt` and `value_if` edges every downstream join, delivery,
> and reachability judgment reads), FN-4 (two sites: the law-discharge
> body shape becomes the infix form with its premise re-keyed to the
> operand-derived selected type; the two law-table rows respell),
> FN-8 (three sites plus one retention: infix conditions;
> example; `value_if` named
> in the exclusion list; clause lets annotation-free retained), EFF-2
> (traps
> contribution names bare infix arithmetic), DIAG-1 (three sites:
> attribution row 2's position guard and token list gain the infix
> operand; the
> typed-call location paragraph replaced for the retained-argument and
> infix classes, its closing citation re-keyed to the callee's class
> because [OP-2] no longer selects FN-2),
> DIAG-3 (bare-operator overflow record, with the `node_path`
> enumeration above it admitting the `infix` node), ENT-2 (value_if
> joins the
> term-root forms), ENT-3 (seven sites: S1 origin and establishment;
> S4 re-keyed so a requires prologue spelled infix still establishes
> its entry fact; S5/S6/S7/S9 respells), ENT-5 (the branch-continuation join in the
> CFG idiom, with the empty join defined for `match` and `if` alike),
> ENT-6
> (fallback respelled), EX-1 (complete worked-example rewrite) — plus
> three R3-PROVISIONAL register settlements (match-only conditionals
> and no-if; prefix arithmetic surface; the interior annotation
> mandate's body half, precedent question O8). Tokens: +1 exact fixed
> lowercase atom (`if`) — `else` is already an exact fixed atom in
> v0.22's `check_stmt` and gains a second grammatical position, not a
> new token or terminal spelling; +20 operator terminal spellings
> (`+ - * / %`, `== != <= >=`, and eleven suffixed operator forms);
> operation-table op-column respells 20 spellings, shrinking
> `DotlessOperationNames` and `ReservedLowerNames` by the four dotless
> comparisons `ieq` `ine` `ile` `ige`; grammar productions +4
> (`if_stmt`, `value_if`, `infix_tail`, `infix_op`; total 69 — the
> `infix` node kind is carried by `infix_tail`'s 1:1 mapping, not a
> phantom production),
> with `stmt`, `let_stmt`, and `expr` modified; exception clauses
> +0/-0; sections +0. The accepted-program set changes as one canonical
> respelling plus three deliberate narrowings: the Bool-scrutinee
> `match` is rejected (spell `if`), a value initializer with an
> empty
> delivery set is rejected (spell the statement form and drop the
> binding), and `if` leaves IDENT eligibility [FORM-3] so a program
> declaring `if` as a function, const, parameter, let, binder, or
> field name is rejected — a measured-empty class (zero declarations
> across the 610-file corpus; the only two `if` tokens are inside
> `doc` strings), recorded on the same footing as v0.22's
> measured-empty S8 narrowing. Delivery-type disagreement is a re-citation, not a
> narrowing: a v0.22-accepted program's `give`s each matched the one
> written annotation and therefore agree with each other, so none is
> newly rejected there, while `give`s that agreed with each other
> against a wrong annotation join the widening below. The error
> classes that lived only in deleted
> bytes die with their bytes. Every operation's semantics, every trap,
> and the claim lifecycle are unchanged; bare
> infix arithmetic is byte-for-byte today's `.trap` mode under a
> shorter constant. [FN-4]'s source-law discharge is re-keyed, not
> changed: the mandated body becomes the infix form and the written
> type argument's premise becomes the operand-derived selected type,
> so exactly the same conformances discharge exactly the same laws
> over exactly the same domains, and every other FN-4 premise, the
> closed law table, and the base derivation record are untouched.
> Selection ground: evidence-selected under the
> four-test spelling rule — SWEEP rows A1/A3/A4/C1 with their T1
> unique-reconstruction arguments, the retained-ANF precedence-free
> property, and the owner rulings of record. These bytes are
> non-authoritative until the grammar check, derived-material review,
> full-document hash, exact owner approval, and active-target
> installation complete.

## 2. Grammar delta

[GRAM-4]'s statement block becomes:

```
stmt        := let_stmt | set_stmt | expr_stmt | return_stmt | loop_stmt
             | break_stmt | region_stmt | check_stmt | claim_stmt | if_stmt
             | match_stmt | give_stmt
let_stmt    := "let" IDENT "="
               ( ordinary_let_rhs | propagate_let_rhs | value_match
               | value_if )
if_stmt     := "if" expr "{" stmt* "}" ("else" (if_stmt | "{" stmt* "}"))?
value_if    := "if" expr "{" stmt* "}" "else" (value_if | "{" stmt* "}")
```

with every other GRAM-4 line byte-identical. [GRAM-5]'s expression head
becomes (left-factored per review F2 — the unfactored
`expr := atom | call | construct | infix` form is NOT strong-LL(2),
because an atom is not one token and `(IDENT, ".")`, `(IDENT, "[")`,
`("deref", "(")`, `("move", IDENT)`, and `("&", REGIONID)` each begin
both an atom expression and an infix expression):

```
expr           := atom infix_tail? | call | construct
infix_tail     := infix_op atom
infix_op       := "+" | "+wrap" | "+checked" | "+sat"
                | "-" | "-wrap" | "-checked" | "-sat"
                | "*" | "*wrap" | "*checked" | "*sat"
                | "/" | "/checked" | "%" | "%checked"
                | "==" | "!=" | "<=" | ">="
```

`infix_tail` maps 1:1 to the `infix` node kind: a selected tail forms
one `infix` node spanning the complete `expr` — the atom and the tail —
so [GRAM-1]'s production-to-node mapping stays 1:1 with no phantom
production and no exception clause [META-3] (stated as one GRAM-1
sentence, §3; review NEW-1 — the honest production total is 69, and the
verifier count is an owner-approved number, so it must be exact). `call := callee targs? …` is
retained byte-identical — `targs` remains grammatical on every call and
the checker enforces the per-callee-class policy [TYPE-5, OP-2]:
required for user-generic calls, system-operation region arguments, and
the retained-argument operations; forbidden elsewhere.

Token formation [GRAM-1]: `if` and `else` are exact fixed lowercase
atoms (auto-excluded from IDENT by [FORM-3]). The compound-token set
grows from two to six: `->`, `=>`, `==`, `!=`, `<=`, `>=`; a lone `!`
remains a raw lexical defect (it exists only inside `!=`). An operator
form starts with `+`, `*`, `/`, or `%`, or with a `-` that is
immediately followed by neither a decimal digit (numeric form,
unchanged) nor `>` (the `->` compound, unchanged), and continues through
the maximal `[a-z]*` suffix; the suffix must be empty or one of `wrap`,
`checked`, `sat` per the closed `infix_op` list, and any other suffix is
a terminal-membership rejection. That form is a new context-free shape
kind: GRAM-1's shape-kind enumeration — the enumeration terminal
membership and every downstream token classification are written
against — gains `operator form` beside the seven it lists, because a
suffixed operator is neither a lower word (it does not start `[a-z]`),
nor punctuation (`+`, `*`, `/`, `%` are not v0.22 punctuation bytes at
all, and no punctuation form carries a letter suffix), nor an
operation-name form (no dot). The four compound comparisons `==`, `!=`,
`<=`, `>=` are not operator forms; they take the existing exact
punctuation shape kind as compound punctuation tokens. Canonical spacing: no operator token
joins either FORM-2 attachment set, so every infix operator renders with
one space on each side, keeping `a - 1_u64` (operator) and `-1_u64`
(literal) lexically distinct in canonical bytes. The O5 attachment
closure extends to all four compound comparisons: `=` is in neither
attachment set (so `==` cannot arise by attachment), `!` is not a v0.22
byte at all (so `!=` cannot), and although `<` is in the left set and
`>` in the right set, no grammar position places `=` immediately after
either — every `>`-then-`=` position is separated by one space because
`=` is in neither set — so `<=` and `>=` cannot arise by attachment.

Metanotation convention (drafting note; O9 ruled, §6). C1 makes `+ - *
/ %` source operator tokens, and this specification already writes
mathematical notation in those same bytes. The two are told apart by
two facts the document already states normatively — no new convention
is introduced, and none is needed:

- Canonical source renders every infix operator with exactly one space
  on each side [FORM-2], so *unspaced* arithmetic or comparison is
  never canonical source: [OP-2]'s `a+b`, `a-b`, `a*b` and its result
  relations `a=b`, `a≠b`, `a<b`, `a<=b`, `a>b`, `a>=b`, and [FN-4]'s
  `x+y`, `2^K-1`, and `MAX-1` are all mathematical by their bytes.
- Canonical source admits exactly one operation per expression
  [GRAM-9], so a *spaced* relation carrying an arithmetic operator
  together with a relational one is never canonical source either,
  spacing notwithstanding. This is what covers the [ENT] fragment,
  whose relation notation is spaced throughout — `a - b <= c`,
  `t1 - t3 <= c1 + c2`, `p + k <= max(T)`, `b - a <= -c - 1`,
  `i - len(P) <= -1` — and which a spacing rule alone would misread.

Measured over the active document: those two facts discriminate every
backticked arithmetic occurrence except three, on two lines — [OP-2]'s
two `iK::MIN / -1` and [OP-9]'s `n * sizeof(T)`, each spaced,
single-operation, and relation-free. Those three are exactly the
occurrences this batch restates in words at sites [OP-2] (e) and
[OP-9]. The minimal edit set is therefore **complete, not merely
small**: it is precisely the residue the two standing facts cannot
discriminate, which is why no whole-document metanotation change is
owed and none is drafted.

Strong-LL analysis (decision by decision; the false claim of the first
draft — that the unfactored choice decides "at the second token" — is
withdrawn):

- `expr` three-way: the call arm's `SELECT_2` is `(IDENT, "(")`,
  `(IDENT, "<")`, `(OPNAME, "(")`, `(OPNAME, "<")`; the construct arm's
  is `(TYPEID, "(")`, `(TYPEID, "<")`; the atom-headed arm never
  produces those pairs, because after a bare-IDENT place the next token
  is a suffix (`.`, `[`), an operator token, or FOLLOW(`expr`) (`;`,
  `{`, `else`) — never `(` or `<` (`<` is deliberately not an
  `infix_op`). Pairwise disjoint at two tokens.
- `infix_tail?`: decided at one token after the complete atom — an
  operator token consumes, FOLLOW(`expr`) = {`;`, `{`, `else`} skips;
  disjoint at one token.
- let-RHS four-way after `let IDENT =`: `propagate` / `match` / `if` /
  FIRST(`expr`); the three keywords are fixed atoms excluded from
  IDENT; disjoint at one token.
- `psuffix*`: consume on `.`/`[`, exit on operator tokens and the rest
  of FOLLOW; disjoint.
- `if` `else?`: mandatory braces make the dangling-else unrepresentable
  — an inner `if` nested in a then-block is separated from any outer
  `else` by the block's `}`, and a chain's trailing `else` is consumed
  innermost-first by the grammar's rightward nesting; FOLLOW(`if_stmt`)
  at the decision is statement starts plus `}`, never `else`.
- `else` alternative: `if` vs `{`; disjoint.
- `stmt` twelve-way: every arm is introduced by a distinct exact fixed
  atom except `expr_stmt`, whose FIRST is `(IDENT, OPNAME)`; `if` is a
  fixed atom excluded from IDENT [FORM-3], so the added `if_stmt` arm
  competes with none of the eleven. Disjoint at one token.

Mechanical check (process fix per the review): the decision rows above
were checked for pairwise SELECT-set disjointness by a scratch script
(do_not_scan, deleted after use) on 2026-08-07 — all seven decisions
pass, and the two expr-critical positions of the rewritten [EX-1]
(`match deref(p) +checked 2_i32 {` and `if ilt(x, 0_i32) {`) trace
through the factored productions and parse. Re-run after the NEW-1
production change: the dropped `infix` production appeared on no
right-hand side, so every decision row is identical, and all seven
pass again (same-day re-run, script recreated and deleted). Third
re-run after the prose-sweep repair (same day, script recreated and
deleted): no production text moved — the repair edits normative prose
only — so the seven rows are unchanged and pass again, and the
previously unlisted `stmt` decision was added and passes, giving eight.
Two further traces were run because the repair puts an infix expression
in two positions [EX-1] did not exercise: [FN-4]'s repaired discharge
body `return p0 +sat p1;` (`return` takes `expr`; the deciding token
after the complete atom `p0` is the operator form `+sat`; FOLLOW is
`;`) and [EX-1]'s existing `check v == 42_i32 else trap "…";` (FOLLOW
is `else`, already in the drafted FOLLOW set). Both parse. Verifier
expectations: fail-closed against the v0.22 tables (grammar-extending),
recorded at proposal; post-extension, productions 65 + 4 = 69; exact
terminal counts established by the grammar-path task.

## 3. Modified rules (complete replacement deltas, verbatim anchors)

**[FORM-2]** Three sites. The block-bearing list "the body of `fn_decl`,
`requires_block`, `loop_stmt`, `region_stmt`, `match_stmt`,
`value_match`, and `arm`" gains `if_stmt` and `value_if` (productions
only — their brace blocks are inline occurrences, not productions), and
the same site appends the sole-governance sentence: "An `if_stmt` or
`value_if` is rendered solely by this sentence, the generic
block-bearing rendering notwithstanding: its introducer through the
then-block `{` is one line; then-children render at depth plus one; an
`else` renders as the join line `} else {` at the original depth, and a
chained `else if` as the join line `} else if` through that `if`'s `{`
at the original depth, never as a nested introducer line; else-children
render at depth plus one; and the final `}` renders on its own line at
the original depth. No one-line `if` form exists." Third site: "A
value-match let places its complete let prefix and the `match`
introducer through `{` on one line." becomes "A value-match or value-if
let places its complete let prefix and the `match` or `if` introducer
through `{` on one line."

**[FORM-3]** One site. The OPNAME lexical-class example "e.g.
`iadd.checked`" becomes "e.g. `ineg.checked`". `iadd.checked` is
respelled to `+checked` by this batch, so after it the example
illustrates the OPNAME token class with a spelling that forms an OPNAME
token but names no operation in the table — lexically still valid,
normatively misleading. `ineg.checked` is a surviving OPNAME with the
same base shape and the same mode word, so the example keeps its
teaching content and its FORM-3/OP-1 cross-references intact. (The
IDENT clause, the closed suffix set, and the maximal-munch
field-access argument are byte-identical; the suffix set is unchanged
because `ineg`, `iabs`, the shifts, and the float `.strict` family all
keep dotted spellings.)

**[GRAM-1]** Four sites. As §2's token-formation paragraph: "`->` and `=>` are the
two compound punctuation tokens." becomes "`->`, `=>`, `==`, `!=`,
`<=`, and `>=` are the six compound punctuation tokens."; the
operator-form clause is added after the numeric-form clause exactly as
§2 states it; the shape-kind enumeration "Raw formation gives every
token exactly one context-free shape kind: lower word, upper word,
region form, label form, operation-name form, numeric form, STRING
form, or one exact punctuation form." becomes "Raw formation gives
every token exactly one context-free shape kind: lower word, upper
word, region form, label form, operation-name form, operator form,
numeric form, STRING form, or one exact punctuation form."; and one
node-mapping sentence is added: "`infix_tail` maps to the
`infix` node kind: a selected tail forms one `infix` node spanning the
complete `expr` — the atom and the tail — so the 1:1
production-to-node mapping is preserved by the factored recognition."

The third site is not editorial. Adding an operator form to the maximal-
form list without adding it here leaves the enumeration false for every
operator token, and that enumeration is what the two sentences after it
are written against: terminal membership "visits every formed token"
and "A grammar terminal is therefore a predicate over a token's shape
kind and exact bytes". A token with no shape kind has no predicate
domain, so the approved operator bytes would be unclassifiable — the
lexer cannot be extended to them, and the strong-LL(2) `SELECT_2` rows
that name operator tokens have nothing to match. The enumeration is
closed and ordinal-free, so the addition is one word in one list.

**[GRAM-4]** As §2: `let_stmt` loses `":" mode type` and gains the
`value_if` alternative; `if_stmt` and `value_if` are added; `stmt`
gains `if_stmt`.

**[GRAM-5]** As §2: `expr` is left-factored; `infix`, `infix_tail`, and
`infix_op` are added; everything else byte-identical.

**[GRAM-6]** Complete replacement of "There is no operator syntax, no
precedence, no infix, no `if`, no `while`, no `for`. Conditional control
is `match` on prelude `Bool` [PRE-1]; a conditional value is a
`let`-initializer `match` [GRAM-7, GIVE-1]; iteration is `loop` +
`break`.":

> There is no general operator syntax and no precedence: an `infix`
> expression is exactly one operation over two atoms [GRAM-5, GRAM-9],
> composition is by `let`, and no precedence, associativity, or
> parenthesization surface exists. There is no `while` and no `for`.
> Conditional control is type-driven with one form per class: a Bool
> condition takes `if`/`else`, an enum scrutinee takes `match`, and
> each is the sole legal form for its class — a `match` whose scrutinee
> has type `Bool` is a hard error citing GRAM-6 at the scrutinee `expr`
> node (spell `if`). An `if` condition must have exact value mode and
> type `own Bool` under exactly the [OP-5] condition judgment, TYPE-7
> exclusivity included; every other condition failure cites GRAM-6 at
> the condition `expr` node. An `if_stmt` `else` whose block is empty
> is a hard error citing GRAM-6 at that `if_stmt` node (spell the
> else-free `if`; a `value_if`'s undelivering else is [GIVE-1]'s
> rejection, not this one). An `else` whose block contains exactly one
> `if_stmt` and nothing else is a hard error citing GRAM-6 at that
> nested `if_stmt` node (spell `else if`); in a `value_if` whose else
> block is exactly one else-free `if_stmt`, the branch cannot deliver,
> [GIVE-1] owns the rejection, and GRAM-6 forms no candidate there, so
> the flattening fix is never demanded where the chain form could not
> be spelled. A conditional value is
> a `let`-initializer `match` or `if` [GRAM-7, GIVE-1]; iteration is
> `loop` + `break`.

(The retained subscript sentence stays byte-identical at the rule's
end.)

**[GRAM-7]** Complete replacement:

> [GRAM-7] `match` and `if` each have one source body shape and two
> distinct core-tree node kinds: `match_stmt`/`if_stmt` for statements,
> `value_match`/`value_if` for a `let` initializer. The pairs never
> compete at one grammar decision: the statement forms begin at the
> statement boundary, the value forms only after the complete
> `let IDENT =` prefix, so the parser decides from source position
> alone, without type, name-resolution, or checker context. A value
> form is value-producing, and every arm or branch must satisfy the
> complete [GIVE-1] delivery judgment for its binding; a `value_if`'s
> `else` is grammatically mandatory [GRAM-4] because a missing branch
> could not deliver. Statement forms produce no value; their bodies act
> by effect. `return`-position conditionals deliver by returning from
> branches; there is no helper-function conditional-initialization
> idiom, and value-production is confined to the `let` initializer, so
> neither construct ever occupies an arbitrary expression position.

**[GIVE-1]** Complete replacement (the fully-worked derived-delivery
rule, per ruling O4 and review F4/F5):

> [GIVE-1] `give e;` delivers `e` as the value of the nearest enclosing
> value initializer — a `value_match` or `value_if`. An else-position
> `value_if` of a chain is part of the chain, not a nested initializer:
> its `give`s deliver to the chain's binding. A value initializer bound
> by its own inner `let` delivers only to that inner binding and never
> makes an outer arm or branch deliver. `give` is legal only inside a
> value initializer's arm or branch — a checker-scoped restriction
> exactly as `break`'s enclosing-loop rule [TYPE-6]: the grammar admits
> `give_stmt` and the checker restricts it, which is META-2-clean by
> the `break` precedent. The binding's mode and type are derived from
> the delivery set [TYPE-5]: every delivering `give` of one value
> initializer must have one identical exact mode and type, and that is
> the binding's derived mode and type; a delivering `give` whose exact
> mode or type differs from an earlier delivering `give` of the same
> initializer is a hard error citing GIVE-1 at the later `give_stmt`
> node — derivation is agreement over the closed delivery set, never a
> join, widening, or common-supertype rule. A value initializer whose
> delivery set is empty — every arm or branch leaves by `return` or by
> `break` to an enclosing loop — is a
> hard error citing GIVE-1 at the `let_stmt` node; the mechanical fix
> is the statement form (`match_stmt` or `if_stmt`) with the binding
> dropped. On every control path an arm or branch terminates in exactly
> one `give e;` or cannot reach the initializer's continuation; a
> give-free continuing path, a statement following a `give` in the same
> block, and a second `give` on one path are each a hard error citing
> GIVE-1 — the value analog of match exhaustiveness [ERR-2].
> Give-completeness is a structural last-statement recursion: an arm or
> branch delivers when its final statement is a `give_stmt`, a
> `return_stmt`, a `break_stmt` whose resolved target loop lexically
> encloses the same value initializer, a `match_stmt` every arm of
> which delivers, or an `if_stmt` with `else` both branches of which
> deliver, relative to that same value initializer; an else-free
> `if_stmt` has a continuing false edge and never delivers. A final
> nested value initializer bound by its own `let` delivers only to its
> own inner let and therefore does not make the outer arm or branch
> deliver. A `check`, `claim`, or call that may trap also has a
> normally continuing edge and does not count as delivery or
> must-divergence. No `loop_stmt` is assumed to diverge. This recursion
> is strictly simpler than the ownership checker. `give e;` moves or
> copies `e` per [OWN-1]; a borrow-typed `e` is judged for regions
> exactly as a returned borrow of the same mode [OWN-4].

**[GRAM-9]** Two sites. "Every call argument, construct field value, and
subscript offset is an `atom` [GRAM-5]" becomes "Every call argument,
construct field value, infix operand, and subscript offset is an `atom`
[GRAM-5]". "A computed value is forwarded to another operation only by
binding it with a preceding `let` (stating its explicit mode and type
[TYPE-5]) and referencing the binding." becomes "A computed value is
forwarded to another operation only by binding it with a preceding
`let` (whose mode and type are derived [TYPE-5]) and referencing the
binding."

**[TYPE-5]** Complete replacement:

> [TYPE-5] Statement-local typing; boundary-explicit facts. A `let`
> binder's mode and type are derived, never written: exactly the mode
> and type its selected right-hand side produces — an
> `ordinary_let_rhs` from its expression, which is always self-typed
> (operands are typed atoms, calls are typed by their
> [FN-1]/[OP-1]/[SYS-2] signatures, literals carry mandatory suffixes
> [FORM-5], constructions name their nominal); a `propagate_let_rhs`
> from the propagated Ok payload [ERR-3]; a `value_match` or `value_if`
> from the derived common delivery type [GIVE-1], whose delivering
> `give`s are inside the same `let_stmt`, so the derivation stays
> statement-local. This is unique reconstruction, not inference: no
> binder's type depends on a later statement, an expected type, or any
> use site, and no two derivations can disagree [FORM-1]. Call sites
> state explicitly exactly what their callee class requires: type,
> region, and const arguments for user generics [FN-2]; region
> arguments for system operations [SYS-2]; and, for exactly the
> retained-argument table operations — `cvt` and `reinterpret` (type
> pairs [OP-6, OP-8]), `array_new` (element type and const length
> [CONST-1]), `arena_new` (region and element type), and `finf`/`fnan`
> (result type) — the written arguments their rows fix, because no
> operand can supply them. Every other table operation carries no
> written argument and derives its selected type from its operands
> [OP-2]; a written argument there is a hard error citing OP-1.
> Argument types match declared parameter types exactly. After [SET-1]
> derives a writable target place of type T, the right-hand side of
> `set p = e;` must produce exactly `own T`; there is no mode coercion,
> type conversion, or target-selected operation overload. After the
> TYPE-7 implicit-read exclusivity below, a different right-hand-side
> mode or type is a hard error citing TYPE-5 at the complete `expr`
> child of the `set_stmt`, carrying expected `own T` and the actual
> mode and type. Redundant-explicit facts remain mandatory at every
> trust boundary — signatures with full modes, types, effect rows, and
> regions [FN-1], construction field names [GRAM-8], match binders
> [GRAM-10], call argument names [GRAM-11] — and are deleted exactly
> where reconstruction is unique and no transposition risk exists.

**[OWN-5]** One site. The slice-valued-join prohibition "A
`let`-initializer `match` whose declared result type is `slice<'r, T>`
is a hard error citing OWN-5 at the complete `value_match`, with
`SourceCoordinate` equal to that production's complete checked
half-open source extent and the restructuring `use a match statement
whose arms return the slice directly, or call helpers with direct
slice results`." is replaced:

> A value initializer whose derived delivery type [GIVE-1, TYPE-5] is
> `slice<'r, T>` is a hard error citing OWN-5 at the complete
> `value_match` or `value_if`, with `SourceCoordinate` equal to that
> production's complete checked half-open source extent and the
> restructuring `use a match or if statement whose arms or branches
> return the slice directly, or call helpers with direct slice
> results`.

Two independent defects, both of this batch's making. The rule is keyed
on "declared result type", which A3 deletes — the type is now derived
from the delivery set, and the judgment must read the derived type or
it reads nothing. And it names only `value_match`, so after A4 a writer
spells the same prohibited slice-valued join as a `value_if` and it is
admitted: an unlisted *widening*, contradicting §4's claim that the
acceptance-set change is one respelling plus three narrowings. Repaired,
the prohibition is respelling-neutral — exactly the programs v0.22
rejects in their `match` spelling are rejected in their `if` spelling,
and nothing else changes. Both node kinds are named because the
diagnostic anchors at the production, and GIVE-1's chain rule already
makes an else-position `value_if` part of its chain rather than a
separate initializer, so a chain reports once at the outermost
`value_if`.

**[OWN-13]** One site (seventh pattern, §7). The value-initializer
ownership sentence "A `let`-initializer `match` binds its value from arm
`give`s [GIVE-1]; scrutinee treatment and binder-mode derivation are
unchanged, and each arm delivers a value of the `let`'s declared mode and
type, so on the taken arm an `own` result is moved exactly once (no
double-move; T1 preserved)." is replaced:

> A value initializer — a `let`-initializer `match` or `if` — binds its
> value from its arm or branch `give`s [GIVE-1]; scrutinee treatment and
> binder-mode derivation are unchanged, and each delivering arm or branch
> delivers a value of the binding's derived mode and type [GIVE-1,
> TYPE-5], so on the taken arm or branch an `own` result is moved exactly
> once (no double-move; T1 preserved).

Two defects in one sentence, one from A3 and one from A4, in a rule that
appeared on no earlier list. "the `let`'s declared mode and type" is the
only occurrence of `declared mode` in the active document and names
exactly the annotation A3 deletes, so the T1 no-double-move argument —
the reason the sentence exists — rests on a construct this batch
removes. The replacement rests it on the derived delivery type instead,
which is the same value by construction: [GIVE-1] derives the binding's
mode and type as the one exact mode and type every delivering `give`
shares, so "a value of the binding's derived mode and type" is what each
delivering arm or branch produces, and the single-move conclusion is
unchanged. Second, exactly as at [OWN-5], the sentence names only the
`match` spelling, so after A4 a `value_if`'s delivered value has no
stated ownership treatment at all. Both halves are respelling-neutral:
the same programs move the same values exactly once in either spelling.

This rule escaped patterns 5 and 6 because it writes "`let`-initializer
`match`" in words rather than as the node kind `value_match`, so a
node-kind enumeration sweep cannot reach it, and it escaped pattern 4
because it carries no `let IDENT: mode` form. Every other [OWN-13]
sentence is byte-identical, the closing "arm-result region join"
PROVISIONAL note included: that phrase labels the judgment stated in the
sentence before it, which is already written node-kind-neutrally ("A
`give e;` whose `e` is a borrow reaching through a binder or an outer
borrow obeys [OWN-4]/[OWN-5] exactly as a returned borrow of the same
mode") and therefore already reaches a `value_if` branch, and [GIVE-1]'s
replacement restates the same borrow judgment generically. Relabeling it
would change no judgment.

**[STOR-2]** One site. "Creation: `box_new<T>(v)` returns `own box<T>`;
`arena_new<'r, T>(v)` returns `own arena<'r, T>`" becomes "Creation:
`box_new(v)` returns `own box<T>` for `v`'s exact type T [OP-2];
`arena_new<'r, T>(v)` returns `own arena<'r, T>`". `box_new` is in the
deleted class — its content operand supplies T — while `arena_new`
keeps both written arguments because its region cannot come from an
operand [TYPE-5]. The asymmetry is now visible in the rule that
introduces the pair, which is the point of naming the derivation here
rather than leaving `box_new`'s T unexplained. The remaining sentence
("both are ordinary calls in the operation table. Content access is
through `deref`.") is byte-identical.

**[STOR-5]** One site. "Substituting a region-bearing T into
`box_new<T>` or `arena_new<'a, T>` places T in an enumerated `box` or
`arena` content position and therefore rejects under STOR-5 at the
complete `type` child of that operation call's `targ`." is replaced:

> Substituting a region-bearing T into `box_new` or `arena_new<'a, T>`
> places T in an enumerated `box` or `arena` content position and
> therefore rejects under STOR-5: for `arena_new`, at the complete
> `type` child of that operation call's `targ`; for `box_new`, whose
> content type is derived from its operand [STOR-2, OP-2], at that
> operand `atom` node and its complete checked half-open source extent.

This site is a diagnostic-location repair, not a respelling. Deleting
`box_new`'s type argument deletes the node the rejection was anchored
at, so the sentence as written is unanchorable for exactly the half of
the pair that loses its argument, and `box_new(s)` for a region-bearing
`s` would have a rejection with no location. The operand atom is the
only node that carries the offending type after A1, and it is the same
node [OP-2]'s operand judgments already use, so the anchor moves to a
node the diagnostic machinery already visits. Every other STOR-5
sentence — the region-bearing definition, the enumerated positions, the
`field`/`vfield` argument, the `slice` element carve-out, and the FN-2
cross-reference — is byte-identical.

**[OP-1]** Five sites. (0) The row-selection sentence "Later typed
operation checking uses the written type arguments and operand domains
to select the applicable row within the resolved family." becomes
"Later typed operation checking uses the operand domains and, for the
retained-argument operations [TYPE-5], the written arguments, to
select the applicable row within the resolved family." (review NEW-2 —
the same keyed-on-the-deleted-argument defect class as F10, one rule
upstream of the OP-7/OP-8 fixes). (i) The table's op column respells twenty
spellings in place: `iadd.wrap isub.wrap imul.wrap` become
`+wrap -wrap *wrap`; `iadd.trap isub.trap imul.trap` become `+ - *`;
`iadd.checked isub.checked imul.checked` become `+checked -checked
*checked`; `idiv.trap irem.trap` become `/ %`; `idiv.checked
irem.checked` become `/checked %checked`; `iadd.sat isub.sat imul.sat`
become `+sat -sat *sat`; `ieq ine ile ige` become `== != <= >=`; `ilt`
and `igt` keep their spellings (O1). (ii) After the operation-family
resolution sentence: "An `infix_op` token resolves to its exactly
spelled operation by the operator table row; infix resolution consults
no name domain, and an operator token is never a declaration, callee
IDENT, or OPNAME." (iii) The `ModeWords` definition sentence "Let
`ModeWords` be exactly the suffix alternatives in FORM-3's active
OPNAME formation rule" becomes "Let `ModeWords` be exactly the suffix
alternatives in FORM-3's active OPNAME formation rule together with the
operator-form suffixes of [GRAM-1]; in this version the two carriers
share one closed set" — so the reservation set is derived from both
suffix carriers (review R1), not from whichever rows happen to be
respelled. Derived-set consequence: `ieq` `ine` `ile` `ige` leave
`DotlessOperationNames` and therefore `ReservedLowerNames`;
`ilt`/`igt` remain members. (iv) The reservation rule's declaration
list "every `let_stmt` IDENT, including ordinary, propagate,
value-match, and requires-block lets" becomes "every `let_stmt` IDENT,
including ordinary, propagate, value-match, value-if, and
requires-block lets" — the same staleness class as review F6, where an
enumeration of `let`-RHS kinds that does not gain `value_if` goes
quietly false. The governing word is "every", so no binding escapes
reservation either way; the list is the reader's inventory and the
source of DIAG-1's `let` declaration role, and a four-of-five
enumeration invites the reading that a value-if let is outside the
rule.

**[OP-2]** Seven sites. The four semantics paragraphs that name the
respelled operations are respelled first; the two judgment paragraphs
and the div/rem mode clause follow.

(a) The wrapping paragraph's opening "For `iadd.wrap<T>(a, b)`,
`isub.wrap<T>(a, b)`, and `imul.wrap<T>(a, b)`, let z be respectively
the mathematical result `a+b`, `a-b`, or `a*b`." becomes "For `a +wrap
b`, `a -wrap b`, and `a *wrap b` over a common selected type T, let z
be respectively the mathematical sum, difference, or product of a and
b." (its remaining sentences — `wrap_T(z)`, totality, purity, no
runtime overflow check — byte-identical).

(b) The trapping paragraph's opening "For `iadd.trap<T>(a, b)`,
`isub.trap<T>(a, b)`, and `imul.trap<T>(a, b)`, let z be the same
mathematical result." becomes "For `a + b`, `a - b`, and `a * b` over
a common selected type T, let z be the same mathematical result."; and
in the same paragraph "Integer overflow in one of these `.trap`
operations is a contract violation" becomes "Integer overflow in one of
these bare-operator trapping operations is a contract violation". Its
remaining sentences — the recoverable-`Overflow` exclusion list, the
constant-operand acceptance, and the [EFF-2] `traps` sentence — are
byte-identical. This is one contiguous site: the two edited sentences
are the first and third of the paragraph.

(c) The comparison paragraph's opening "For `ieq<T>`, `ine<T>`,
`ilt<T>`, `ile<T>`, `igt<T>`, and `ige<T>`, both operands denote their
mathematical values in the selected T." becomes "For `a == b`, `a !=
b`, `ilt(a, b)`, `a <= b`, `igt(a, b)`, and `a >= b`, both operands
denote their mathematical values in the selected T." — the operand
order in the existing result sentence ("`True()` exactly when `a=b`,
`a≠b`, `a<b`, `a<=b`, `a>b`, or `a>=b`") is positional, so the
respelled list must keep the same six positions, and the O1 asymmetry
(`ilt`/`igt` named, the four nonstrict comparisons infix) is visible
here exactly as it is in [EX-1]. Its remaining sentences — signed and
unsigned ordering, the same-exact-T requirement, and totality/purity
over `own Bool` — are byte-identical. The replacement does bind `a` and
`b`, which the v0.22 sentence left free even though the result sentence
below it uses them. This paragraph is where the metanotation question
[O9, ruled] is sharpest and, on the ruling, where it answers itself:
the byte-identical result sentence reads "`True()` exactly when `a=b`,
`a≠b`, `a<b`, `a<=b`, `a>b`, or `a>=b`" — mathematical relations one
sentence below the source spellings `a == b`, `a != b`, `a <= b`, `a
>= b` — and the two are distinguished by [FORM-2]'s mandatory
one-space-each-side rendering, which makes every unspaced form
non-source by construction. The result sentence is therefore left
byte-identical, and this paragraph is the convention's clearest
demonstration rather than a counterexample to it (§2 drafting note).

(d) The negation paragraph's opening "For `ineg.wrap<T>(a)`,
`ineg.trap<T>(a)`, and `ineg.checked<T>(a)`, T is one signed member of
the closed integer-type set and z is the mathematical integer `-a`."
becomes "For `ineg.wrap(a)`, `ineg.trap(a)`, and `ineg.checked(a)`, T
is the operand's exact type, one signed member of the closed
integer-type set, and z is the mathematical integer `-a`."; the three
following occurrences of `ineg.wrap<T>(a)`, `ineg.trap<T>(a)`, and
`ineg.checked<T>(a)` in the same paragraph lose their type arguments
identically. `ineg` is not respelled — it keeps its dotted OPNAME
spelling and only loses the written argument A1 deletes [TYPE-5], so
this is the deleted-class edit, not a C1 edit. One contiguous site.

(e) The div/rem mode clause "`.trap` traps on either, and `.checked`
returns `Err(DivideByZero())` for a zero divisor and
`Err(DivOverflow())` for signed `iK::MIN / -1`, else `Ok`." becomes
"the bare `/` and `%` operators trap on either, and `/checked` and
`%checked` return `Err(DivideByZero())` for a zero divisor and
`Err(DivOverflow())` for signed minimum divided by negative one, else
`Ok`." — the same defect class as the [EFF-2] site: prose naming a
`.trap`/`.checked` spelling that division and remainder no longer
carry. The mode-axis membership sentence later in the paragraph
("div/rem carry {trap, checked}") is byte-identical, because it names
modes as words, not spellings, and the modes are unchanged. The
`iK::MIN / -1` metanotation is spelled out in words in the replacement
because after C1 those exact bytes are a source operator; see O9.

(f) The binary judgment paragraph "Each operation in
the preceding paragraphs has exactly one explicit type argument. …
cites [OP-1]." is replaced:

> Each operation in the preceding paragraphs carries no written type
> argument: its selected type is derived from its operands. Both
> operands must have one identical exact type — a member of the closed
> integer-type set or, in a symbolic generic body, one live type
> parameter whose bound resolves to PRE-1 `Int`, with every concrete
> FN-2 instantiation substituting one closed-set member and the
> corresponding mathematical semantics above. That common exact type is
> the selected type; the derivation is agreement, never widening,
> conversion, or preference. Operands of two different exact types are
> a hard error citing TYPE-5 at the second operand atom in source
> order. A written type argument, a region or const argument, a
> concrete operand type outside the closed integer set, an inadmissible
> generic type, or a wrong operand count cites [OP-1].

(g) The negation judgment paragraph's first three sentences "Each negation
call has exactly one explicit type argument and exactly one positional atom
operand of that exact selected type. The explicit-type-argument and
symbolic-generic judgments are the same as in the earlier two-operand
operation judgment paragraph, except that every concrete selected type must
belong to the signed subset `i8 i16 i32 i64`; selecting an unsigned integer
or another domain cites [OP-1]. The same FN-2 missing-type-argument, OP-1
type-argument-kind/count and operand-count, TYPE-7 implicit-read
exclusivity, TYPE-5 wrong-operand-type, exact table-result, and
consuming-construct judgments apply." are replaced:

> Each negation call carries no written type argument and has exactly one
> positional atom operand; that operand's exact type is the selected type.
> The type-derivation and symbolic-generic judgments are the same as in the
> earlier two-operand operation judgment paragraph — the selected type is
> derived from the operand, never written — except that every concrete
> selected type must belong to the signed subset `i8 i16 i32 i64`;
> selecting an unsigned integer or another domain cites [OP-1]. The same
> OP-1 written-argument, argument-kind, and operand-count judgments, and
> the same TYPE-7 implicit-read exclusivity, exact table-result, and
> consuming-construct judgments, apply.

The paragraph's fourth and final sentence ("No operand value or expected
result type selects the negation mode, changes its signed domain, or
inserts a conversion.") is byte-identical and stays true: it quantifies
over the operand's *value* and over the expected result type, neither of
which this batch makes type-selecting, and the signed domain stays fixed
by this rule rather than by the operand. One contiguous site.

Why all three sentences and not only the opening one. Rewriting the
opening sentence alone — "its remaining sentences unchanged" — leaves two
dangling references in the same paragraph, both of the F10/NEW-2 class,
and the first of them is load-bearing:

- Sentence 3 mandates that "The same FN-2 missing-type-argument …
  judgments apply" while site (f) deletes the only sentence that defines
  that judgment ("Absence of the required explicit type argument cites
  [FN-2]."). Applied literally, the surviving mandate runs a
  missing-type-argument judgment against a call that can no longer carry
  a type argument, so every `ineg.wrap`, `ineg.trap`, and `ineg.checked`
  call is rejected for a missing argument — a total, unlisted narrowing
  of the negation family, the same shape as the [FN-4] discharge defect.
- Sentence 2 delegates "The explicit-type-argument and symbolic-generic
  judgments" to the two-operand paragraph, where after site (f) there is
  no explicit-type-argument judgment left to delegate to. The
  symbolic-generic half does survive (f) verbatim, and the replacement
  keeps delegating to it rather than restating it.

Two judgments leave sentence 3's list because their domains are empty
after A1, not because the batch narrows them. FN-2's
missing-type-argument judgment goes with the written argument it was
about, and a written argument now cites OP-1 under (f)'s replacement —
which is why the replacement list opens with the OP-1 written-argument
judgment. TYPE-5's wrong-operand-type judgment was the mismatch between a
written type argument and the operand; with the selected type derived
*from* that operand there is nothing left for it to disagree with, and
(f)'s operand-agreement error is explicitly a two-operand error reported
at the second operand atom, which a negation call does not have. TYPE-5
still reaches the negation *result* through the consuming-construct
judgment the list retains.

Sites (a)–(d) are the miss this repair exists to close: the 46-site
revision rewrote OP-2's two *judgment* paragraphs off the written type
argument while leaving the four *semantics* paragraphs above them
writing one. As assembled the rule was self-contradictory — its own
rewritten paragraph read "Each operation in the preceding paragraphs
carries no written type argument" directly beneath four paragraphs
whose every operation carried one.

**[OP-4]** One site (seventh pattern, §7). The subscript element-type
derivation clause "derived from the base place's already-stated type by
the same declared-type selection that types a field suffix, never from
expected type or cross-statement inference [TYPE-5]" is replaced:

> derived from the base place's already-fixed type [TYPE-5] — written
> where the binding carries an annotation, derived at a body `let` — by
> the same declared-type selection that types a field suffix, never from
> expected type or cross-statement inference

The clause names the deleted annotation in words rather than in its
spelling, so none of the six lexical patterns reaches it. After A3 a
base place rooted in a body `let` has no stated type, and the clause
cites [TYPE-5] — the rule this batch rewrites to say the opposite — for
a property it states as "already-stated". The judgment does not move:
the base place's selected type is fixed before the subscript is checked
in either spelling, and the property the clause exists to state, "never
from expected type or cross-statement inference", is exactly what the
rewritten TYPE-5 preserves. The three indexable base types, `array<T,
N>`'s const length, the field-suffix selection, the bounds obligation,
and the offset judgment are byte-identical. (The near-miss "already-
stated indexable type" in the v0.22 version header is frozen history,
cleared with the other `Prior:` paragraphs below.)

**[OP-7]** Three sites. One sentence appended: "A respelled operation's
operator token is its one constant spelling — bare operators carry the
trapping-overflow mode, suffixed operators carry `wrap`, `checked`,
and `sat`, and the four nonstrict comparisons are `==` `!=` `<=` `>=`
— under exactly the same one-spelling-per-operation discipline; the
`i`-prefix convention continues to govern the operations that keep
named spellings." "Signedness-parametric lowering keyed on the
explicit type argument (`ishr` is `ashr` for signed T and `lshr` for
unsigned T; `imin` is `smin` or `umin`)" becomes
"Signedness-parametric lowering keyed on the operand-derived selected
type [OP-2] (`ishr` is `ashr` for signed T and `lshr` for unsigned T;
`imin` is `smin` or `umin`)". "Nominal enum identity is likewise
checked from the explicit type argument before `eeq`/`ene` lowering"
becomes "Nominal enum identity is likewise checked from the
operand-derived selected type before `eeq`/`ene` lowering".

**[OP-8]** Two sites plus one retention. "`iadd.sat`/`isub.sat` are
`llvm.sadd.sat`/`uadd.sat` or `ssub.sat`/`usub.sat` clamping to T's
range; `imul.sat` widens, multiplies, and clamps" becomes
"`+sat`/`-sat` are `llvm.sadd.sat`/`uadd.sat` or `ssub.sat`/`usub.sat`
clamping to T's range; `*sat` widens, multiplies, and clamps". "For a
tag-only enum T, `eeq<T>(a, b)` is `True()` exactly when `a` and `b`
denote the same declared variant of the same nominal T, and
`ene<T>(a, b)` is its exact boolean complement." becomes "For a
tag-only enum T — the operand-derived selected type [OP-2] — `eeq(a,
b)` is `True()` exactly when `a` and `b` denote the same declared
variant of that nominal T, and `ene(a, b)` is its exact boolean
complement." "Both operands and the explicit type argument must have
that exact T; representation equality never permits cross-enum
comparison." becomes "Both operands must have that exact T, derived by
[OP-2]'s agreement rule; representation equality never permits
cross-enum comparison." The `eeq` and both-operands sentences are
adjacent in v0.22 and form one contiguous site. The `fneg(finf<T>())`
sentence is retained
byte-identical — `finf` keeps its type argument in the
retained-argument class [TYPE-5].

**[OP-9]** One site. "`buffer_new<T>(n, v)` computes its allocation
byte-size as `n * sizeof(T)` in u64" becomes "`buffer_new(n, v)`
computes its allocation byte-size as the u64 product of n and
sizeof(T)", with T the element type derived from the `v` operand
[OP-2]. `buffer_new` is in the deleted class. The rest of the rule is
byte-identical: every later mention spells `buffer_new`, `box_new`,
`arena_new`, `array_new`, and `array<T, N>` without a type argument
already, the trap judgment and its [SCOPE-4]/[STOR-6] boundary are
untouched, and `array_new` keeps its written element type and const
length in the retained class [TYPE-5]. The product is stated in words
rather than as `n * sizeof(T)` for the same reason as OP-2 site (e);
see O9.

**[ERR-2]** "Every `match` is exhaustive over declared variants; there
are no wildcard arms." gains: "Bool exhaustiveness is carried by `if`:
an else-free `if` is the empty-alternative form, an `if` with `else`
covers both, and a Bool-scrutinee `match` is rejected at GRAM-6.
The asymmetry is deliberate and content-driven: the empty then-block
is admitted while the empty else is not, because the else-free form is
the one spelling of the empty alternative."

**[ERR-3]** "Propagation: `let x: own T = propagate e;` requires
`e : own Result<T, E>` and the enclosing function's return type
`own Result<U, E>` (same E — no conversions, TYPE-4)." becomes
"Propagation: `let x = propagate e;` requires `e : own Result<T, E>`
and the enclosing function's return type `own Result<U, E>` (same E —
no conversions, TYPE-4); x's derived mode and type are `own T`
[TYPE-5]." The rest of the rule is byte-identical.

**[FN-1]** One site. In the conservative structural normal-control
graph, after "A `let_stmt` selecting `value_match` enters every arm
body the same way and follows [GIVE-1]: each `give` edge reaches
`normal_successor` of that enclosing `let_stmt`, each return edge
reaches the function-return sink, and each resolved break edge reaches
`normal_successor` of its target loop.", insert:

> An `if_stmt` enters its then-block, and its else-block when it has
> one, using a block's normal exit when that block contains no
> statement; each block's normal exit reaches
> `normal_successor(if_stmt)`, and an else-free `if_stmt` also has its
> false edge directly to `normal_successor(if_stmt)`. A `let_stmt`
> selecting `value_if` enters both branch blocks the same way and
> follows [GIVE-1] exactly as the `value_match` sentence above does;
> an else-position `value_if` of a chain contributes its own branch
> edges to the same enclosing `let_stmt` [GIVE-1], not to a nested
> one.

This site was outside the reported list and is the most load-bearing of
the fifteen. FN-1's graph is not a description — it is the sole
definition of `normal_successor` and of the reachability the rest of
the specification reads. Without these edges an `if_stmt` has no
successors at all, so on the graph as written every statement inside
both branch blocks is unreachable from function-body entry and
establishes an FN-1 rejection premise, and the function-body normal
exit's reachability is undefined; the batch's own [ENT-5] join sentence
cites "the conservative structural graph [FN-1]" for a set of `if`
branch exit edges that graph does not contain; [GIVE-1]'s
give-completeness recursion and [ENT-3]'s branch facts read the same
graph. In other words the A4 respelling, unrepaired, rejects every
program it is supposed to make writable. The inserted text adds no new
idiom: the `if_stmt` sentence is the `match_stmt` sentence with arms
replaced by branches, the `value_if` sentence delegates to the
`value_match` sentence verbatim, and the else-free false edge is the
same edge [ENT-5] already names.

**[FN-4]** Two sites, per the lead's ruling of record: the discharge
relation is re-keyed, not narrowed, and `iadd.sat` does **not** join
[TYPE-5]'s retained-argument class (the review certified that class
total against the complete operation table under F3; adding a member
whose operands can supply its type reopens F3).

(i) The mandated body-shape sentences "After an optional leading `doc`,
the bound function's body must contain exactly one statement, `return
iadd.sat<D>(p0, p1);`." and, in the same paragraph, "Each is used as
one bare place, once and in declaration order, and the explicit type
argument is D." become:

> After an optional leading `doc`, the bound function's body must
> contain exactly one statement, `return p0 +sat p1;`.

and

> Each is used as one bare place, once and in declaration order, and
> the operation's operand-derived selected type equals D.

with the paragraph's remaining sentences byte-identical — the `p0`/`p1`
metanotation disclaimer and the closed exclusion list ("No alias,
field, dereference, `move`, reordered argument, extra statement, second
operation, user call, or semantically equivalent body matches this
discharge shape."). One contiguous site.

Why re-keying rather than retention. The batch respells `iadd.sat` to
`+sat` and deletes its written type argument, so the mandated shape
`return iadd.sat<D>(p0, p1);` has no legal spelling afterwards and the
premise "the explicit type argument is D" quantifies over a node that
no longer exists. Both premises are mandatory and the relation is
declared complete, so every source-law discharge would fail — a total,
previously unlisted narrowing of a construct the batch does not
otherwise touch, live at four active corpus sites and four conformance
verdicts. Re-keying is the same move the batch already makes twice:
[OP-7] and [OP-8]'s signedness and enum-identity judgments were rekeyed
from "the explicit type argument" to "the operand-derived selected
type" under review finding F10, and [OP-1]'s row selection under NEW-2.
The premise is *equally strong* after the move, because [OP-2]'s
agreement rule makes the selected type the unique exact type both
operands share, and FN-4's surrounding relation independently pins both
`fn_decl` parameter types to D — so "operand-derived selected type
equals D" is derivable from premises FN-4 already states, and the
discharged set is provably identical. Nothing else about FN-4 moves:
the FN-3 delegation, the signature and domain premises, the closed law
table's cells, the `identity` argument rule, the base derivation record
[DIAG-2], and the optional-fact verifier boundary are untouched.
Confirming evidence that the re-key is minimal rather than
compensatory: the compiler's `discharge_domain` already matches the
checked tree's `IntegerOperation { operation: AddSaturating,
operand_type, … }` and compares `operand_type` against the subject — it
never reads a written type argument — so the repair leaves the
discharge premise exactly where the implementation already stands, and
confines the compiler's change to the spelling path.

(ii) The two law-table rows respell in the `resolved table operation`
column: "| `iadd.sat<T>` for T in `u8 u16 u32 u64` |" and "|
`iadd.sat<T>` for T in `i8 i16 i32 i64` |" become "| `+sat` for T in
`u8 u16 u32 u64` |" and "| `+sat` for T in `i8 i16 i32 i64` |". The two
rows are adjacent and form one contiguous site. Every other cell — the
domains, `≡D`, and the `yes`/`holds`/`refuted`/`zero of T` verdicts —
is byte-identical, and the paragraph below the table (K as bit width,
the unsigned `min` argument, the signed refutation witness, the
unavailability rule, and the base-derivation-record sentences) is
byte-identical, `min(2^K-1, x+y)` and `MAX-1` included: both are
unspaced and therefore mathematical by [FORM-2]'s rendering rule, so
[O9]'s ruling leaves them untouched.

**[FN-8]** Three sites plus one retention. "Every computation in the block must be an ANF
[GRAM-9] call to a non-trapping, total operation-table row with effect
`pure`; the final check condition is either a Bool clause atom or one
such call returning Bool." becomes "Every computation in the block must
be an ANF [GRAM-9] call to, or infix spelling of, a non-trapping,
total operation-table row with effect `pure`; the final check
condition is either a Bool clause atom or one such operation returning
Bool." "(for example `len<u8>(deref(out))`)" becomes "(for example
`len(deref(out))`)". "a `propagate_let_rhs`, a `value_match`, or any
other direct statement shape is a hard error citing FN-8" becomes "a
`propagate_let_rhs`, a `value_match`, a `value_if`, or any other
direct statement shape is a hard error citing FN-8". The
structural-pass sentence naming "`let_stmt` nodes whose selected
right-hand side is `ordinary_let_rhs`" is retained byte-identical —
clause lets are annotation-free like every let [GRAM-4], the
reviewer-confirmed uniform reading (O3).

**[EFF-2]** "exhibit `traps` iff either contains any `.trap` op,
`check`, `claim`, or a call" becomes "exhibit `traps` iff either
contains any trapping-mode operation — a bare infix arithmetic
operator (`+`, `-`, `*`, `/`, `%`) or a `.trap` OPNAME — `check`,
`claim`, or a call".

**[DIAG-1]** Three sites. Attribution row 2's position guard "an `atom`
occurrence in `atom_list`, `fieldinit`, or the subscript offset"
becomes "an `atom` occurrence in `atom_list`, `fieldinit`, an `infix`
operand, or the subscript offset" — without this the token-list
addendum below is unreachable (review F8b: the guard admits the
occurrence, the token list then fires, and `let x = a + b * c;`
attributes at `*`). The typed-call location paragraph "For a typed
call to an [OP-2] operation, a missing explicit type argument uses
`SourceNode` at the `call` node and that node's complete source
extent. A wrong type-argument kind, count, or domain, or a missing
operand, uses the same call location. An extra operand or every wrong
exact operand type other than the TYPE-7 implicit-read case uses
`SourceNode` at the first offending `atom` node in source order and
that atom's complete extent. The cited rule is the rule selected by
[OP-2]: FN-2, OP-1, or TYPE-5." is replaced:

> For a call to a callee class that carries written arguments — a
> user-generic `fn` [FN-2], a system operation's region arguments
> [SYS-2], or a retained-argument table operation [TYPE-5] — a
> missing, wrong-kind, wrong-count, or wrong-domain argument, or a
> missing operand, uses `SourceNode` at the `call` node and that
> node's complete source extent. For an operation spelled infix, a
> wrong operand domain or a missing operand uses `SourceNode` at the
> `infix` node and its complete extent. An extra operand or every
> wrong exact operand type other than the TYPE-7 implicit-read case
> uses `SourceNode` at the first offending `atom` node in source order
> and that atom's complete extent — for [OP-2]'s operand-agreement
> error, the second operand atom. The cited rule is the rule selected by
> the callee's class: [FN-2] for a user-generic call, [SYS-2] for a
> system operation's region arguments, and, for a table operation, the
> rule [OP-2] selects — OP-1 or TYPE-5.

The closing sentence is the third [FN-2] bridge, and unlike the other
two it lives in text this batch itself writes. The previous revision
carried v0.22's "The cited rule is the rule selected by [OP-2]: FN-2,
OP-1, or TYPE-5." into the replacement verbatim, but after sites (f) and
(g) [OP-2] selects FN-2 for nothing: the only two sentences that made a
table operation cite FN-2 are deleted, and every remaining FN-2 citation
in this paragraph belongs to the user-generic half the replacement's own
first sentence added. Attributing that citation to OP-2's selection is
therefore wrong in the new paragraph's widened scope, and dropping FN-2
outright would strand the user-generic case. Keying the selection to the
callee's class states what each half actually does and leaves [FN-2]'s
own bytes untouched (§7).

Attribution row 2's "are `(IDENT, "(")`, `(IDENT, "<")`, `(OPNAME,
"(")`, `(OPNAME, "<")`, `(TYPEID, "(")`, or `(TYPEID, "<")`, the
rejection cites [GRAM-9]" gains: "; in an infix-operand occurrence, a
two-token start whose second token is an operator token — the
forbidden nested-infix start — likewise cites [GRAM-9]".

**[DIAG-3]** One site, its anchor extended upward by one sentence to the
`node_path` enumeration (seventh pattern, §7). "and the operation `call`
for a table-operation contract check and for the [SYS-8] range
validation judged under [OP-4]'s retained operation-internal semantics.
For an executed `iadd.trap`, `isub.trap`, or `imul.trap` overflow,
`rule_id` is `OP-2`, `message` is `integer overflow`, and `node_path` is
the trapping `call` node." becomes "and the operation `call` — or, for
an operation spelled infix, the `infix` node — for a table-operation
contract check and for the [SYS-8] range validation judged under
[OP-4]'s retained operation-internal semantics. For an executed bare
`+`, `-`, or `*` overflow, `rule_id` is `OP-2`, `message` is `integer
overflow`, and `node_path` is the trapping `infix` node; a bare `/` or
`%` contract violation is a table-operation contract check at its
`infix` node."

The enumeration sentence is the seventh pattern's find; the sentence
below it was already a site. Left alone, [DIAG-3] would define
`node_path` for a table-operation contract check as "the operation
`call`" two sentences above this batch's own clause placing a bare `/`
or `%` contract violation "at its `infix` node" — the general rule
naming a node kind the infix spelling does not produce, contradicted by
the specific one. The parenthetical is vacuous for the [SYS-8] half and
deliberately so: system operations are never spelled infix, their range
validation keeps its `call` node, and [OP-4]'s closing cross-reference
to "the operation `call` node [DIAG-3]" for exactly that judgment stays
true and byte-identical.

**[SYS-13]** One site (seventh pattern, §7). The opacity argument "There
are no implicit conversions [TYPE-4] and every binding states its exact
type [TYPE-5], so without a stated constructor the command entry's
returned value would be unwritable" becomes "There are no implicit
conversions [TYPE-4] and every value's type is exactly what its producer
fixes [TYPE-5], so without a stated constructor the command entry's
returned value would be unwritable".

The same class as the [OP-4] site: the premise is the deleted annotation
stated in words. The argument is unchanged and in fact simpler after A3.
What made an `ExitStatus` unwritable without `exit_status(code)` was
never the annotation — it was that no conversion, literal, or
construction produces the type, and the replacement states exactly that
under the rewritten TYPE-5, where a binding's type is what its
right-hand side produces. `exit_status(code)` remains the one
constructor, the closed 0–255 code range and the totality, purity, and
no-failure properties are untouched, and the rest of [SYS-13] is
byte-identical.

**[ENT-2]** "(whichever of the three right-hand forms — ordinary,
`propagate`, or `value_match` — the statement selects)" becomes
"(whichever of the four right-hand forms — ordinary, `propagate`,
`value_match`, or `value_if` — the statement selects)".

**[ENT-3]** Seven sites. S1's origin clause "it is a call to one of
`ieq`, `ine`, `ilt`, `ile`, `igt`, `ige` [OP-2] whose two operands are
each a term or constant" becomes "it is an infix comparison `==`,
`!=`, `<=`, `>=`, or a call to `ilt` or `igt` [OP-2], whose two
operands are each a term or constant". S1's establishment sentence
"For a `match_stmt` or `value_match` whose scrutinee has comparison
origin R, R is established at the `True()` arm's entry and R's exact
negation at the `False()` arm's entry." becomes "For an `if_stmt` or
`value_if` whose condition has comparison origin R, R is established
at the then-block's entry and R's exact negation at the else-block's
entry; for an else-free `if_stmt`, the negation is established on the
false edge, which joins the then exit at the continuation [ENT-5]."
S4's normalization and firing sentences "replace every requires-clause
local by its unique defining right-hand side, repeatedly, until only
parameters, named consts, literals, and admitted table-operation calls
remain. When the result is one comparison call admitted by the
comparison-origin shape (a) whose operands after substitution are each a
term over parameters or named consts, a constant, or a call `len<T>(P)`
over such a place — read as the length term len(P) — that relation is
established at body entry." become "replace every requires-clause local
by its unique defining right-hand side, repeatedly, until only
parameters, named consts, literals, and admitted table-operation calls
and infix spellings remain. When the result is one comparison admitted
by the comparison-origin shape (a) whose operands after substitution are
each a term over parameters or named consts, a constant, or a call
`len(P)` over such a place — read as the length term len(P) — that
relation is established at body entry." (one contiguous site; it
subsumes the `len<T>(P)` respell the previous revision anchored alone).
S5's "for `let x: own T = lit;`, x =
value(lit); for `let x: own T = p;` with p a term of type T, x = p;
for `let y: own Dst = cvt<Src, Dst>(p);` with (Src, Dst) a total pair
[OP-6] and p a term or constant, y = p." becomes "for `let x = lit;`,
x = value(lit); for `let x = p;` with p a term of type T, x = p; for
`let y = cvt<Src, Dst>(p);` with (Src, Dst) a total pair [OP-6] and p
a term or constant, y = p — `cvt` keeps its written type pair
[TYPE-5]." S6's forms respell: "`let b: own buffer<T> =
buffer_new<T>(n, v);`" becomes "`let b = buffer_new(n, v);`",
"`let m: own u64 = len<T>(P);`" becomes "`let m = len(P);`", and the
slice_of form loses its annotation likewise. S7's shapes respell:
"`let s: own T = iadd.wrap<T>(p, k);`" becomes "`let s = p +wrap k;`"
(and symmetrically `-wrap`), the trap forms become bare "`p + k`" /
"`p - k`", and the checked-origin scrutinee becomes "`p +checked k`" /
"`p -checked k`"; every side condition, range premise, and kill
discipline is byte-identical. S9's "For `let x: own T = c[i];`"
becomes "For `let x = c[i];`" (x's derived type is the element type).

S4 is the load-bearing member of the seventh pattern and the one site in
this repair whose omission would have been an unlisted *narrowing*, in
the [FN-1] class rather than the [OP-4]/[SYS-13] characterization class.
Both of its defects name the deleted construct in words: the
normalization terminates on "admitted table-operation **calls**", and
the rule fires only "When the result is one comparison **call**". C1
respells `ieq` `ine` `ile` `ige` as `==` `!=` `<=` `>=`, and this
batch's own [FN-8] site deliberately admits "an ANF [GRAM-9] call to, or
infix spelling of" an operation-table row inside a `requires` block. So
after the batch a requires prologue normally reduces to a form S4 can
neither terminate on nor recognize; S4 establishes nothing at body
entry; every obligation that depended on the prologue fact goes
undischarged; and [OP-4] makes an undischarged subscript obligation "a
compile-time rejection". The unrepaired batch would therefore reject the
`requires`-carrying programs it compiles today, exactly as the
unrepaired [FN-1] graph would have rejected every `if` program.

Measured on the active corpus (2026-08-07; `.wf` sources excluding
`archive/` and the worktree mirror). Two bases are stated because they
differ sharply and only the first sizes this batch's migration.

*Migration basis* — the corpora maintained at the active spec version,
`tests/conformance` and `tests/programs`, which is §5's basis and the
scope this batch migrates. On that basis **22**
files carry a `requires` block, one block each, and 20 of those blocks
end in a final `check`. Two of the 20 conditions are a direct
comparison call (both `ile`). Seventeen are Bool clause atoms whose
defining right-hand sides are `ige` 8, `eeq` 3, `igt` 3,
`ile` 1, `ieq` 1, and `ilt` 1; the twentieth is a bare parameter in
the negative case `fn8-neg-requires-non-bool-check.wf`, which exists to
be rejected and establishes nothing either way. So 12 of the 20 route
through a comparison C1 respells and are exactly the shape S4 stops
recognizing, 4 keep a call spelling under O1 (`igt`, `ilt`), and the
`eeq` clauses are neither respelled nor S1 origins and are
untouched. The canonical broken shape is `let permitted: own Bool =
ieq<i32>(copied, x); check permitted …`, migrating to `let permitted =
copied == x; check permitted …`
(`tests/conformance/cases/fn8-neg-requires-user-call.wf`, verified).
Independently of the final condition, 3 arithmetic lets inside these
blocks (`imul.wrap`, `iadd.wrap`, `iadd.trap`, one each) respell to
infix and break the normalization's termination clause on their own,
while `ishr.wrap`, `band`, and `len` keep call spellings and do not.

```
git ls-files 'tests/conformance/*.wf' 'tests/programs/*.wf' \
  | xargs grep -l 'requires {' | wc -l          # 22
```

*Outside the migration basis*, the two frozen corpora carry 40 more
`requires` blocks — 37 in `tests/codegen/` and 3 in
`research/experiments/` — for 62 repository-wide. The
`tests/codegen/` share is the sharper shape: 36 of its 37 conditions
are a direct `ile`/`ilt` comparison call, precisely the form S4 stops
recognizing. Those files do not migrate with this batch and are
reconciled with the active spec per case at promotion, so the S4
breakage surfaces there rather than here. The 22 sizes this batch's
work; the 62 sizes the eventual exposure. (Two earlier revisions of
this paragraph are withdrawn: one reported 122 files, counting every
source twice through the worktree mirror, with an 82/76/4/2/46
breakdown that did not reconcile against itself; the other reported 25
on a basis that wrongly included `research/experiments`, which is
frozen evidence and not on the active spec — see §5.) The repair is
keyed forward, not widened: "one comparison admitted by the
comparison-origin shape (a)" delegates the spelling question to (a),
which this batch's S1 site already rewrites to admit exactly the four
infix comparisons plus `ilt`/`igt`, so the admitted set is unchanged and
no new origin becomes derivable.

**[ENT-5]** One site. Before "The continuation of a `loop_stmt` is the
join over the states on its `break` edges", insert: "At the
continuation of an `if_stmt` or `value_if`, the fact state is the join
of the states on every branch exit edge reaching that continuation on
the conservative structural graph [FN-1] — for an else-free `if_stmt`,
the false edge is such an edge — each taken after that edge's
scope-exit kills and then closed [ENT-4]; a branch every path of which
leaves by `return`, `break` to an enclosing loop, or `propagate`'s
error edge contributes nothing there. An empty join — no arm or branch
exit edge reaches the continuation — is the contradictory
all-derivable state [ENT-4], as for a break-free loop; this empty-join
clause governs `match_stmt` and `value_match` continuations
identically." (Review F1 and its residues: without the join, the `if`
continuation is either an undefined merge point or a fact-flow-through
that deletes a bounds check; the CFG idiom covers chains, else-free
forms, and nesting without enumeration — the else-position `if` of a
chain contributes its branch exit edges directly; and the empty-join
clause closes the hole v0.22 inherited for all-arms-return `match`
continuations rather than duplicating it for `if`.)

**[ENT-6]** The fallback "in canonical ANF, one `let` binding
`len<T>(P)` followed by one `claim` on, or `match` over, the admitted
comparison [CLM-1, ENT-3]" becomes "in canonical ANF, one `let`
binding `len(P)` followed by one `claim` on, or `if` over, the
admitted comparison [CLM-1, ENT-3]".

**[EX-1]** Complete replacement of the worked-example program bytes
(canonical under every rule of this batch; the O1 asymmetry is
deliberately visible in `sign_of`'s first branch):

```
enum Sign {
  Neg();
  Zero();
  Pos();
}

fn sign_of(x: own i32) -> own Sign pure {
  doc "Conditional value produced by returning from branches (canonical for return position).";
  if ilt(x, 0_i32) {
    return Neg();
  } else if x == 0_i32 {
    return Zero();
  } else {
    return Pos();
  }
}

fn main() -> own unit traps {
  doc "let-initializer match with give: a conditional value bound, then reused.";
  let a = 40_i32;
  region 'r {
    let p = &'r a;
    let v = match deref(p) +checked 2_i32 {
      Ok(value: w) => {
        give w;
      }
      Err(error: e) => {
        return unit;
      }
    }
    check v == 42_i32 else trap "arithmetic drift";
  }
  return unit;
}
```

**R3-PROVISIONAL register** (header material): "match-only conditionals
and no-if (GRAM-6/PRE-1)" and "prefix arithmetic surface (OP-1/GRAM-6)"
leave the register — the first on the redundancy proof (the two arm
labels are always exactly `True()`/`False()` in fixed order), the
second on SWEEP's objective tiebreaks, which is R3's stated currency;
"interior annotation mandate (TYPE-5 — round-2 verdict still
needs_evidence)" reduces to its surviving boundary half ("boundary
annotation surface (TYPE-5)") — whether a T1 argument discharges a
`needs_evidence` entry is O8, the precedent question. "statement-only
match (GRAM-7)" remains.

## 4. Acceptance-set delta

One canonical respelling, three deliberate narrowings, and one named
consequence. Respelling:
every existing program's canonical bytes change and old bytes reject
under FORM-1 — migrated mechanically (§5). Widening: the error classes
that lived only in deleted bytes die with them (a wrong let annotation,
a wrong or missing value-op type argument — unwritable states now,
including `give`s that agreed with each other against a wrong
annotation); separately, `ieq` `ine` `ile` `ige` leave
`ReservedLowerNames` and become writer-reusable declaration spellings
(R2). Narrowings: (1) a Bool-scrutinee `match` is rejected
(spell `if`) — the
type-driven one-form-per-class rule T3 requires; (2) a value
initializer with an empty delivery set — every arm or branch leaves by
`return` or `break` — is rejected at the
`let_stmt` node (review F4; v0.22 accepts it with the annotation
supplying the never-read binding's type; migration: spell the
statement form and drop the binding); (3) `if` becomes an exact fixed
grammar atom and therefore leaves IDENT [FORM-3], so it is no longer
legal as a function, const, parameter, let, match-binder, field,
variant-field, or region name — measured empty across the 610-file
corpus (the only two `if` tokens are inside `doc` strings, which are
STRING interiors and not tokens of this class), and recorded on the
same footing as v0.22's measured-empty S8 narrowing. Note that `else`
adds no such narrowing: it is already an exact fixed atom in v0.22's
`check_stmt` and is already excluded from IDENT.

**[TYPE-5]'s body-let mismatch rejection becomes unspellable, not
accepted.** Under v0.22 a body `let` whose written annotation disagreed
with its initializer was a [TYPE-5] rejection. Under this batch that
program is not rejected but unspellable, because the annotation it
disagreed with no longer exists. The check that vanished is precisely
the body-position redundancy the T2 test classifies as rot and this
batch deletes by rule — the same trade approved with A3, not a new one.
This is a consequence of the respelling, not a fourth acceptance change:
no program that v0.22 accepts is rejected, and no program v0.22 rejects
is accepted. (Rule id verified against the active file rather than
assumed: v0.22 locates that rejection at [OP-2]'s "TYPE-5 owns
let-binding and call-argument exactness", which occurs once and which
site (f) retains byte-identical below its replaced span. [TYPE-4]
supplies the no-conversion law the rejection rests on but states no
`let` judgment at all — it is about `cvt` and implicit conversion — so
it is not the citation and not a site, §7.)

Five further classes are *not* acceptance-set changes only because the
prose and pattern-7 sweeps key them forward; each would have been an
unlisted change had an earlier revision shipped. [FN-4]'s source-law discharge
is re-keyed to the operand-derived selected type, so exactly the same
conformances discharge exactly the same laws — unrepaired it was a
total narrowing, because the mandated body had no legal spelling.
[FN-1]'s control graph gains the `if_stmt` and `value_if` edges, so
`if` programs are reachable and complete exactly as their `match`
spellings are — unrepaired, every statement inside every branch was
unreachable and every `if` program rejected. [OWN-5]'s slice-valued
join prohibition is rekeyed to the derived delivery type and extended
to `value_if`, so it rejects exactly the programs it rejects today —
unrepaired it was a *widening*, admitting in the `if` spelling a join
v0.22 rejects in the `match` spelling. [ENT-3] S4 is re-keyed so a
`requires` prologue whose comparison is spelled infix still establishes
its entry fact, so exactly the same obligations discharge — unrepaired
it was a narrowing on the [FN-1] scale, because S4 fires only on "one
comparison call" while C1 makes the four nonstrict comparisons infix
and [FN-8] admits them there, leaving every dependent subscript
obligation undischarged and rejected under [OP-4] across the 22
files this batch migrates, and across the 40 more in the two frozen
corpora when those are promoted (§3). [OP-2]'s negation judgment
(site (g)) drops the FN-2 missing-type-argument mandate along with the
argument it quantified over — unrepaired it was a total narrowing of
the negation family, rejecting every `ineg.wrap`, `ineg.trap`, and
`ineg.checked` call for an argument A1 forbids it to carry.
Delivery-type disagreement is a
re-citation, not a narrowing (review NEW-4): a v0.22-accepted
program's `give`s each matched the one written annotation and
therefore agree with each other, so the agreement rule newly rejects
none — only the citation and location change (GIVE-1 at the second
divergent `give`, where TYPE-5 at each mismatching `give` stood
before). Every operation's semantics, every
trap, every discharge judgment, and the claim lifecycle are unchanged;
the `if` continuation joins facts exactly as the `match` continuation
it respells [ENT-5].

## 5. Corpus migration (mechanical, printer-driven; re-measured
2026-08-07 at this revision against the respelled v0.22 corpus, on the
**migration basis**: the corpora maintained at the active spec version —
`tests/conformance` 401 and `tests/programs` 19, **420** files)

**Standing requirement for this section: every figure carries the exact
command that reproduces it — directory scope, pattern, and exclusions —
so a reader re-runs the number instead of trusting this report.** These
are the largest inputs the migration task sizes its work against and
they go to the owner, and the staleness recorded below survived several
reviews precisely because the figures travelled without their commands.
A figure stated here without a runnable command is incomplete.

Why the basis is these two corpora and not every `.wf` file. Both
`tests/codegen/` and `research/experiments/` are frozen at older spec
versions and were excluded from the v0.22 migration by task 0031
("`tests/codegen/` deliberately untouched (preserved holding corpus per
its README; no harness consumes it); research experiment bundles stay
frozen evidence"). That is not only a record but a measurable fact: both
still spell the indexed place as the prefix `index<T>(place, offset)`
that v0.22 deleted, 134 occurrences in `tests/codegen/` and 128 in
`research/experiments/`, while the two basis corpora contain zero and
use the `place[offset]` subscript throughout. Counting a corpus that is
not on the active spec into a migration figure inflates it with work
this batch does not do.

```
for d in tests/conformance tests/programs research/experiments tests/codegen; do
  printf '%s index< = ' "$d"
  git ls-files "$d/*.wf" | xargs cat | sed 's/^/ /' \
    | grep -oE '[^A-Za-z0-9_.]index<' | wc -l
done
```

Two definitions are shared by every command; paste them first. Every
command below was run at this revision and returned the number it is
attached to.

```
BASIS() { git ls-files 'tests/conformance/*.wf' 'tests/programs/*.wf'; }
PREP()  { xargs cat | sed -E 's/"([^"\\]|\\.)*"/""/g' | sed 's/^/ /'; }
```

`PREP` does two things. It blanks double-quoted string literals, which
matters by exactly 3 occurrences across the wider corpus — all `doc`
text quoting a rule (`op6-neg-cvt-identity`, `op9-pos-buffer-new`,
`stor2-pos-box-new`). Then it prefixes every line with a space so that a
leading `[^A-Za-z0-9_.]` class suffices and no pattern needs `^` inside
an alternation.

Three method points, each load-bearing. The leading `[^A-Za-z0-9_.]` is
what makes a count a per-callee enumeration rather than a name grep:
without it, `name<` also matches `buffer<`, `array<`, `slice<`,
`Result<`, every user-generic call, and every field access `x.len<` — on
this basis a bare `identifier<` sweep returns 3494 against the 1588
below. An operation alternation must enumerate the table's actual rows,
never a cross-product of stems and mode words: the tempting
`(iadd|isub|imul|idiv|irem)\.(wrap|trap|checked|sat)<` matches
`idiv.wrap<i32>` in `tests/conformance/cases/op2-neg-div-wrap.wf`, one
of the five deliberately-invalid fixtures and a row the table does not
contain. And the space-prefix idiom is not cosmetic: the `grep` on this
machine is ugrep 7.5.0, which silently returns **0** for an
`-oE '(^|[^A-Za-z0-9_.])name<'` pattern instead of erroring, so a
command written that way reports a confident, wrong zero. Every figure
below was cross-checked against an independent Python `re` count using
a true lookbehind, and the two agree exactly.

- Basis: **420** files.

  ```
  BASIS | wc -l
  ```
- Deleted-class type arguments: **1588** occurrences. The op alternation
  is derived from the active spec's own table, so the command re-derives
  the deleted class rather than hard-coding 77 spellings that could
  drift from it:

  ```
  OPS=$(sed -n '/^| op | domain/,/^$/p' spec/kernel-spec-v0.22.md \
    | sed -E 's/^\| *//; s/ *\|.*//' | grep -o '`[^`]*`' | tr -d '`' \
    | sort -u \
    | grep -vE '^(cvt|reinterpret|array_new|arena_new|finf|fnan)$' \
    | sed 's/\./\\./g' | paste -sd'|' -)
  BASIS | PREP | grep -oE "[^A-Za-z0-9_.]($OPS)<" | wc -l
  ```

  The complete deleted class — every table
  operation except the six retained — led by `iadd.wrap`, `ieq`,
  `len`, `ilt`, and `isub.wrap`. Separately: five
  deliberately-invalid OPNAME spellings in negative conformance
  fixtures (`irotl.trap`, `idiv.wrap`, `fneg.strict`, `iadd.bogus`,
  `add.wrap`) are NOT deleted-class sites — they never parse to a
  table-op call, the migration never touches them, and a per-callee
  enumeration excludes them automatically because no such row exists.
  Retained-class sites, untouched: **102**
  (`cvt`/`reinterpret`/`array_new`/`arena_new`/`finf`/`fnan`
  — the F3 orphans now have their one legal spelling, unchanged).

  ```
  BASIS | PREP \
    | grep -oE '[^A-Za-z0-9_.](cvt|reinterpret|array_new|arena_new|finf|fnan)<' \
    | wc -l
  ```
- Staleness correction of record, applying to **every** figure in this
  section rather than to one of them. The previously settled numbers —
  1353 deleted-class, 101 retained-class, 1748 let annotations, 257
  `True()` arms, 56 `ilt`/`igt`, 389 `check` statements, all "across
  399 files" — reproduce exactly at the revision that settled them: at
  4530716 the corpus held `tests/conformance` 380 plus `tests/programs`
  19, which is precisely the 399, and counting those two directories at
  that revision *without* blanking string literals yields 1353, 101,
  1748, 257, 56, and 389 to the digit. That exact reproduction settles
  what "399 files" was, a question this section previously left to
  inference. **The basis was correct; the figures were stale.** Twenty-one
  conformance cases have landed since — 71 `.wf` files were added
  corpus-wide in the three days to 2026-08-07 — and re-measuring the
  same two corpora at this revision moves deleted-class 1353 → **1588**,
  retained 101 → **102**, let annotations 1748 → **2003**, `True()` arms
  257 → **262**, `ilt`/`igt` 56 → **207**, and `check` 389 → **406**.
  The migration task sizes its work against 1588.
- Let annotations deleted: **2003** binders (1748 when settled).

  ```
  BASIS | PREP \
    | grep -oE '[^A-Za-z0-9_]let +[A-Za-z_][A-Za-z0-9_]* *:' | wc -l
  ```
- Bool matches to `if`/`else`: **262** `True()`-arm matches (257 when
  settled), including the else-if
  flattening of the corpus's Bool ladders.

  ```
  BASIS | PREP | grep -oE 'True\(\) *=>' | wc -l
  ```
- Infix respells: **378** add/sub/mul/div/rem sites, plus **519**
  `== != <= >=` sites; **207**
  `ilt`/`igt` sites keep named calls under O1, losing only their type
  arguments.

  ```
  BASIS | PREP | grep -oE \
    '[^A-Za-z0-9_.]((iadd|isub|imul)\.(wrap|trap|checked|sat)|(idiv|irem)\.(trap|checked))<' \
    | wc -l                                            # 378
  BASIS | PREP | grep -oE '[^A-Za-z0-9_.](ieq|ine|ile|ige)<' | wc -l  # 519
  BASIS | PREP | grep -oE '[^A-Za-z0-9_.](ilt|igt)<'   | wc -l        # 207
  ```

  The arithmetic total reproduces the settled `~384` as an exact 378 by
  this method; the settled per-op 229 `iadd.wrap` and 47 `iadd.trap`
  come out 227 and 45, a residual 2 each that is *not* string handling
  (both spellings have zero string-literal occurrences at that
  revision). Six other figures reproduce to the digit, so the basis is
  certain; the residual is recorded rather than smoothed over. The
  `ilt`/`igt` figure moves most, **56 to 207**, because the added
  conformance cases exercise the two retained comparisons heavily —
  O1's cost is roughly four times what the settled figure implied,
  which is the one place this correction bears on an open ruling.
- `check` statements: **406** — untouched (C3 open, O2); 389 when
  settled.

  ```
  BASIS | xargs grep -hE '^[[:space:]]*check ' | wc -l
  ```

  Measured as
  line-leading `check` statements, which [FORM-2]'s line-bearing rule
  makes exact, and confirmed by the `else trap` count, unique to
  `check_stmt`; the additional loose-grep occurrences live
  inside `doc` strings and trap messages (both earlier figures — the
  draft's 404 and the review's 409 — were loose counts).
- Empty-delivery-set value initializers (narrowing 2): any corpus
  instance is respelled to the statement form in the same migration;
  none is expected (the shape is pointless), and the migration pass
  reports each one it rewrites.
- `if` as a declaration spelling (narrowing 3): **0** sites across all
  **610** `.wf` files. This one figure is deliberately measured on the
  widest scope rather than the migration basis — `archive/` and both
  frozen corpora included — because it supports a "nowhere in the
  repository" claim, which a narrow basis could not. Re-verified at
  this revision: 0 `if` tokens outside string literals, 2 in total, and
  both are the `doc`-string occurrences already named
  (`gram6-pos-no-operators.wf`,
  `x-arith-iadd-checked-overflow-err-arm-runs.wf`), which migrate by no
  rule.

  ```
  git ls-files '*.wf' | wc -l                                    # 610
  git ls-files '*.wf' | xargs cat | sed -E 's/"([^"\\]|\\.)*"/""/g' \
    | sed 's/^/ /' | grep -oE '[^A-Za-z0-9_.]if[^A-Za-z0-9_]' | wc -l   # 0
  git ls-files '*.wf' | xargs grep -lE '[^A-Za-z0-9_.]if[^A-Za-z0-9_]'  # the 2
  ```
- [FN-4] law-discharge bodies (the re-keyed shape): **2** migrating sites
  carry the exact discharge body `return iadd.sat<D>(p0, p1);` —
  `research/experiments/checked-law-channel/kernel.wf` and
  `kernel_lib.wf`, and conformance cases `fn4-pos-law-discharged.wf`
  and `fn4-neg-law-refuted-signedness.wf` — each migrating to `return
  p0 +sat p1;` by the ordinary printer pass, with no verdict change.
  A fifth site under `archive/` is out of scope by the standing
  no-active-dependency rule. **Scope settled (lead, 2026-08-07):** the
  migrating count is **2** — the two conformance cases. The two sites
  under `research/experiments/` do NOT migrate with this batch: task
  0031 froze that bundle as evidence, it is demonstrably not on the
  active spec (it still writes the `index<T>(p, i)` form v0.22 deleted,
  128 occurrences), and editing it here would both break that freeze
  and require migrating its v0.22 debt first. They reconcile with the
  active specification at promotion, exactly as the `tests/codegen`
  holding corpus does. The re-key argument is unaffected — it is a rule
  judgment, not a count. Four conformance verdicts read the FN-4
  discharge relation (`fn4-pos-law-discharged`, `fn4-pos-law-in-contract`,
  `fn4-neg-law-undischarged`, `fn4-neg-law-refuted-signedness`); all
  four are preserved by the re-key, and `fn4-neg-law-undischarged`
  keeps its rejection because its body remains multi-statement after
  its own `iadd.wrap<u64>(t, 1_u64)` respells to `t +wrap 1_u64`.
  `fn4-neg-bad-lawname` is independent of the discharge shape.
- All migration is printer-driven per SWEEP's A/C batch rule;
  conformance sources and spelling-bearing manifest expectations
  respell in the same change under the standing derived-material rule;
  the derivation ledger, `docs/patterns.md` writer forms, and the
  register lines update in the same change.

## 6. Ruled and open list

Ruled (owner standing instruction plus reviewer confirmations,
2026-08-07): the batch itself — A1 (with F3's total retained class), A3,
A4, C1 as revised; C3 deferred (O2, reviewer-verified on all three
grounds); O3 uniform annotation-free requires lets (reviewer-recommended
on the T2 ground: the boundary fact is the final check, not the
scaffolding); O5 the `=[` attachment stands closed (reviewer-verified:
`=` is in neither attachment set, `==` cannot arise by attachment,
extended by this repair to `<=`, `>=`, and `!=`); O7
the empty then-block admitted with the asymmetry stated in ERR-2.

O9 — arithmetic metanotation versus the new operator tokens — is ruled
(lead, 2026-08-07): the minimal option stands, so the batch restates in
words only the spots it already edits and leaves every other backticked
arithmetic occurrence byte-identical; no whole-document metanotation
change is owed. The reason is recorded as a drafting note in §2, and
the record differs in one respect from the ruling as first stated. The
ruling's ground was that mathematical relations are written unspaced
with single symbols while source spellings are spaced — true of
[OP-2]'s comparison paragraph, but **false of the [ENT] fragment**,
which writes its mathematical relations spaced throughout (`a - b <=
c`, `p + k <= max(T)`, `b - a <= -c - 1`). A spacing rule alone would
therefore have misread the largest body of collision-prone notation in
the document, so the note states the discriminator as the two standing
facts that actually hold: [FORM-2]'s mandatory one-space-each-side
rendering (which rules out unspaced notation as source) together with
[GRAM-9]'s one-operation-per-expression rule (which rules out any
relation carrying both an arithmetic and a relational operator). That
pair discriminates every backticked arithmetic occurrence in the active
document except three, on two lines, and those three are exactly the
ones this batch already restates in words. The ruling's conclusion is
thus not merely upheld but strengthened: the minimal edit set is
provably complete rather than a pragmatic trim.

Open (owner ruling needed; drafted with the recommended option):

- O1 — bare `<`/`>` infix: excluded as drafted; `ilt`/`igt` stay named
  (56 sites; the asymmetry is visible in EX-1). Corrected record of the
  rejected alternative (review): after A1, the entire collision surface
  is `(IDENT, "<")` on user-generic calls and SYS-2 region arguments —
  type-position targs never compete (TYPEID is not an atom), so option
  (b) is a call-targs-only introducer (turbofish-shaped compound
  token), far cheaper than the first draft claimed, though still a
  canonical change on every generic call. Revisit condition: R2's
  predictability cost measured, or the next batch touching call
  syntax.
- O4 — the derived common delivery type ships as the fully-worked
  GIVE-1 replacement (§3): agreement judgment, empty-set rejection,
  chain delivery, ENT-2 term roots, ENT-5 join. Confirm the complete
  rule.
- O6 — another-batch items restated from SWEEP D and C4: GRAM-9
  nesting relaxation (deferred indefinitely), literal-class redesign,
  the counted range loop (obligation-discharge item 6),
  `.trap`/`.checked` OPNAME dissolution — now carrying R1 as its named
  discharge condition — and float/enum/bitwise infix (needs
  collision-free spellings).
- O8 — register-settlement precedent (review F11f): the TYPE-5
  settlement rests on SWEEP's T1 argument, while the register entry
  says `needs_evidence` (writer/codegen comparison currency). The
  other two settlements carry the register's own currency. Owner
  ruling requested on whether a T1 argument discharges a
  `needs_evidence` entry — this batch sets the precedent for the
  remaining eleven.

Residue findings of record (review Part 2, tracked here so the debts
have owners):

- R1 — two mode-suffix carriers (`FORM-3` OPNAME dots; `GRAM-1`
  operator suffixes) now spell the same three words. Named, dated debt
  (2026-08-07): temporary by construction; discharge condition is
  O6's `.trap`/`.checked` dissolution batch; the OP-1 `ModeWords`
  sentence (§3) keeps the reservation set derived from both carriers
  meanwhile.
- R2 — the comparison family is split by a lexer constraint (`== != <=
  >=` infix; `ilt`/`igt` named), the batch's one piece of
  implementation-shaped surface; discharge path is O1's revisit
  condition. The reservation consequence is recorded: `ieq`-class
  names become writer-reusable while `ilt`/`igt` stay reserved.
- R3 — the derived delivery type is the batch's only new machinery;
  it ships fully worked (GIVE-1, ENT-2, ENT-5, F4, F5) and its cost
  is recorded as proportionate to A3's 2003 deleted annotations (§5).

No other contradiction between the batch and v0.22 remains: every
collision — TYPE-5's mandate, GRAM-6's no-if sentence, GRAM-7's
two-kind discipline, OP-2's semantics and judgment paragraphs,
OP-7/OP-8's keyed-on-the-argument sentences, GRAM-9's forwarding
parenthetical, FN-1's control graph, FN-4's discharge shape, OWN-5's
slice join, STOR-2/STOR-5's `box_new`, OP-9's `buffer_new`, FORM-3's
OPNAME example, GRAM-1's shape-kind enumeration,
DIAG-1's call-node locations and FN-2 citation, DIAG-3's `node_path`
enumeration, OP-4's "already-stated" base-place type,
SYS-13's unwritability argument, OWN-13's declared-mode-and-type
delivery sentence, ENT-2/ENT-3/ENT-5/ENT-6's fact
machinery including ENT-3 S4's requires-prologue normalization,
FN-8's clause subset, ERR-2/ERR-3, the register entries, and EX-1's
bytes — is an enumerated modification above. That claim now rests on
the mechanical sweep of §7 — seven patterns, the seventh non-lexical
and run against all four deleted constructs — rather than on the
modified-rule list.

## 7. Sweep method and site ledger

The 46-site revision was assembled by working through the rules this
batch *modifies*. That is the wrong closure: a respelling also
invalidates every rule whose normative prose *uses* the respelled
construct, and those rules are not reachable from the modification
list. Fifteen sites were missed, two of them load-bearing enough to
reject or admit whole program classes ([FN-1], [OWN-5]). The sweep
below is over the whole active `spec/kernel-spec-v0.22.md` — all 128
rules — and is the closure this candidate now claims.

Patterns swept, each over the complete file:

1. Every lowercase name immediately followed by `<`, enumerated by
   name and counted, then classified against the deleted class (every
   table operation except the six retained) versus type constructors
   (`slice` `buffer` `array` `arena` `box`), bounds (`int`), and
   metavariables. Hits: `len` `ineg.wrap` `ineg.trap` `ineg.checked`
   `ieq` `ine` `ilt` `ile` `igt` `ige` `iadd.sat` `iadd.wrap`
   `iadd.trap` `iadd.checked` `isub.wrap` `isub.trap` `isub.checked`
   `imul.wrap` `imul.trap` `eeq` `ene` `buffer_new` `box_new`.
   Retained and correctly untouched: `cvt` `arena_new` `finf`.
2. Every dotted OPNAME spelling, and separately every bare backticked
   mode word (`` `.trap` `` `` `.checked` `` `` `.wrap` `` ``
   `.sat` `` `` `.strict` ``), classified by whether this batch
   respells the family.
3. `index<` — zero occurrences; v0.22 already removed the construct.
4. Every annotated `let` form (`let IDENT: mode …`) in rule prose and
   normative examples.
5. Every enumeration of `let`-RHS kinds ("ordinary, propagate,
   value-match"), of conditional node kinds (`match_stmt`,
   `value_match`), and of `True()`/`False()` arm positions.
6. Every arm-scoped, join, control-graph, and reachability prose site,
   reached from the node-kind enumeration of pattern 5.
7. Every normative sentence naming a deleted construct *in words*
   rather than in its spelling. Patterns 1–6 are lexical: each keys on
   a spelling, a production form, or a node-kind enumeration, and a
   sentence like "every binding states its exact type" contains none of
   those bytes, so no earlier pattern can reach it. Swept as a phrase
   sweep over the complete file for each way a deleted construct is
   named in prose. The batch deletes **four** constructs, not three,
   and the fourth is the one an annotation-focused probe forgets:

   - the written type argument — "explicit type argument", "written
     type argument", and the hyphenated forms a space-separated grep
     cannot see ("explicit-type-argument", "missing-type-argument",
     "type-argument-kind"), which is how [OP-2] site (g)'s surviving
     FN-2 mandate hid;
   - the `let` annotation — "states its … type", "stating its explicit
     mode and type", "declared `mode type`", "declared mode and type",
     "at the binder", "No inference", "full mode", "already-stated",
     "declared-type", "annotation";
   - the Bool-scrutinee conditional — "conditional control is `match`
     on prelude `Bool`", and the node kinds written in words rather
     than as productions ("a `let`-initializer `match`", "each arm
     delivers");
   - **the call spelling itself.** C1 respells ten operation families
     as infix, so every sentence that says an operation is reached by
     a "call" *in words* — "admitted table-operation calls", "one
     comparison call", "the operation `call` for a table-operation
     contract check" — names a form those operations no longer have.
     This class is invisible to patterns 1 and 2, which look for
     operation *spellings*, and it is where the sweep's one remaining
     narrowing lived ([ENT-3] S4).

   Every hit was classified against the site list. Two probe failures
   are recorded so the next reviewer does not repeat them: searching
   "declared type" returns zero hits in the active file while the live
   defect is spelled "declared mode and type" ([OWN-13] — the one
   "declared mode" in the document), and a phrase list built from the
   three *deletions* rather than the four *constructs* never asks about
   the call spelling at all.

Fifteen new sites, all verbatim-anchored and each anchor verified to
occur exactly once in the active file: FORM-3 1, GRAM-1 1 (its fourth),
OWN-5 1, STOR-2 1, STOR-5 1, OP-1 1 (its fifth), OP-2 5, OP-9 1, FN-1
1, FN-4 2. Of these, thirteen were reported by the assembling executor
and two — FN-1's control graph and OWN-5's slice join — were found only
by patterns 5 and 6.

Pattern 7 found three further sites and extended two existing anchors,
each anchor verified to occur exactly once in the active file.

The three sites. [OP-4]'s "already-stated" base-place type and
[SYS-13]'s "every binding states its exact type" opacity argument are
characterization defects of the same class as [GRAM-9]'s forwarding
parenthetical — a rule stating in words where a type comes from, keyed
on the annotation A3 deletes and citing the very rule this batch
rewrites to say the opposite — and neither rejects nor admits a program
class. [OWN-13]'s "each arm delivers a value of the `let`'s declared
mode and type" is the third and the only one that adds a rule to the
batch: it carries both an A3 defect (the sole "declared mode" in the
document) and an A4 defect (it names only the `match` spelling, the
[OWN-5] gap class), and it evaded patterns 4, 5, and 6 by writing
"`let`-initializer `match`" in words rather than as `value_match`.

The two anchor extensions, both in the call-spelling class and neither
adding a site. [ENT-3] S4 is the one pattern-7 find that *is* an
unlisted narrowing, in the [FN-1] class: it normalizes to "admitted
table-operation calls" and fires only on "one comparison call", so
after C1 the `requires` prologue this batch's own [FN-8] site admits
reduces to a form S4 cannot recognize, no entry fact is established,
and [OP-4] turns the resulting undischarged obligations into
compile-time rejections across the 22 files carrying
`requires` blocks that this batch migrates — §5's basis — and across
the 40 more in the two frozen corpora at their
promotion (§3). [DIAG-3]'s `node_path` enumeration names "the operation
`call` for a table-operation contract check" two sentences above this
batch's own clause placing a bare `/` or `%` violation at its `infix`
node. A third call-spelling correction, [DIAG-1]'s "the rule selected
by [OP-2]: FN-2, OP-1, or TYPE-5.", lies inside an existing site's
replacement text and likewise adds no site.

Pattern 7 also found the one defect in this candidate's own text rather
than in v0.22: [OP-2] site (g) retained a mandate that the FN-2
missing-type-argument judgment applies after site (f) deleted the only
sentence defining it, repaired in §3. Every other pattern-7 hit was
already a site — [TYPE-5], [GRAM-4]'s production, [GRAM-6]'s
`match`-on-`Bool` sentence, [GRAM-7]'s `let IDENT : mode type =`
prefix, [GIVE-1]'s "declared `mode type` (stated at the binder)",
[GRAM-9]'s forwarding parenthetical, [OP-1]'s written-argument row
selection, [OP-2] (f) and (g), [OP-7], [OP-8], [FN-4] (i), [DIAG-1]'s
typed-call paragraph, and the R3-PROVISIONAL register's "interior
annotation mandate".

Recount, independently: FORM-2 3, FORM-3 1, GRAM-1 4, GRAM-4 1, GRAM-5
1, GRAM-6 1, GRAM-7 1, GIVE-1 1, GRAM-9 2, TYPE-5 1, OWN-5 1, OWN-13 1,
STOR-2 1,
STOR-5 1, OP-1 5, OP-2 7, OP-4 1, OP-7 3, OP-8 2, OP-9 1, FN-1 1, FN-4
2, FN-8
3, EFF-2 1, ERR-2 1, ERR-3 1, DIAG-1 3, DIAG-3 1, SYS-13 1, ENT-2 1,
ENT-3 7,
ENT-5 1, ENT-6 1, EX-1 1 = **64 sites across 34 rules**, reconciling
against the previous 61/31 as exactly the three pattern-7 sites
([OP-4], [SYS-13], [OWN-13]) and their three rules, and against the
original 46/24 as those plus the fifteen sites and seven rules of the
prose sweep. The count is of
anchor/replacement pairs in §3, not of delta line items: a site is one
contiguous verbatim-anchored replacement however many sentences it
edits, so [OP-2] (b) — two edited sentences of one paragraph — [OP-2]
(g) — three — [ENT-3] S4 — two — and [DIAG-3] — two — are one site
each, and a reviewer counting pairs reaches 64. Three pattern-7
corrections deliberately do *not* raise the count, because each widens
an anchor or edits replacement text inside a site the batch already
had: [ENT-3] S4 (ENT-3 stays 7), [DIAG-3]'s `node_path` enumeration
(DIAG-3 stays 1), and [DIAG-1]'s FN-2 citation sentence (DIAG-1 stays
3). Productions remain 65 + 4 = 69 and no production text moved:
§2's two EBNF blocks are byte-identical through this repair, MD5
00f6095415ba43440367b87d94f06a3e (`stmt`/`let_stmt`/`if_stmt`/
`value_if`) and cfd44788e1b76e4017271f8e639f2308 (`expr`/`infix_tail`/
`infix_op`), re-verified after this revision's edits and recorded so the
next revision can re-check the property mechanically. [EX-1]'s program
block is likewise unmoved (MD5 814fdabade0cea99e3879bd5fdc6f892). The
four retentions (FN-8's clause lets, OP-8's
`fneg(finf<T>())`, GRAM-5's `call := callee targs? …`, and GRAM-6's
subscript sentence) are still excluded from the total.

Inspected and cleared — near-misses that are *not* sites, recorded so
the next reviewer does not re-derive them:

- The `Prior:` version-header paragraphs, notably v0.14's, which
  restate `ineg.wrap<T>(a)` and its siblings. Frozen history describing
  what a past revision defined in that revision's spelling; respelling
  them would falsify the record. The v0.22 header at the top becomes a
  `Prior:` entry by the ordinary version procedure.
- [CONST-1]'s "does not overload the runtime `.trap` OPNAMEs". The
  class stays nonempty and correct after the batch — `ineg.trap`,
  `iabs.trap`, `ishl.trap`, `ishr.trap` keep dotted spellings — it
  merely no longer covers add/sub/mul/div/rem.
- [OP-8]'s `ishl.wrap`, `ishr.trap`, `iabs.wrap`/`.trap`/`.checked`,
  and the whole float `.strict` family. Not respelled by C1.
- [FN-2]'s explicit-type-argument prose. Scoped to function, source
  nominal, and PRE-1 nominal generics; table operations are none of
  those. v0.22 bridges FN-2 to the table operations twice, not once, so
  the clearance rests on the pair: site (f) removes the first bridge,
  the binary judgment's "Absence of the required explicit type argument
  cites [FN-2].", and site (g) removes the second, the negation
  judgment's "The same FN-2 missing-type-argument … judgments apply".
  Either bridge left standing would keep a missing-type-argument
  judgment alive over an argument no call can write, rejecting the
  family it governs; with both removed, FN-2 reaches no table
  operation. The earlier revision cleared this entry on the first
  bridge alone and called it "the only bridge", which the pattern-7
  sweep falsified. A third naming, in [DIAG-1]'s "The cited rule is the
  rule selected by [OP-2]: FN-2, OP-1, or TYPE-5.", is a consequence of
  the other two rather than an independent bridge — and it sits in text
  this batch itself writes, having been carried into the DIAG-1
  replacement verbatim. It is corrected inside that existing site (§3),
  which re-keys the citation to the callee's class, so [FN-2]'s own
  bytes remain untouched by this batch and this entry stays a clearance
  rather than becoming a site.
- [TYPE-4], [OWN-1], and [STOR-1], each reported to this drafting lane
  as an unlisted pattern-7 site and each verified absent from the
  active file. [TYPE-4] states the no-implicit-conversion law and the
  `cvt` totality partition and never mentions `let`; it carries no
  annotation/initializer mismatch judgment to split, and the body-let
  mismatch rejection v0.22 actually states lives at [OP-2]'s "TYPE-5
  owns let-binding and call-argument exactness" (occurs once, retained
  byte-identical by site (f); §4). "declared type" occurs zero times in
  the whole active file, so neither [OWN-1] — copy/affine
  classification and consumption — nor [STOR-1] keys any judgment on a
  binding's declared type. [STOR-1]'s only neighbouring phrase is
  "There is no per-binding storage annotation", which is about storage
  class, is not a type annotation, and is untouched by A3. Recorded by
  name so the next reviewer does not re-derive three absent sites.
- The `param`, `rtype`, and entry-point `input_label IDENT ":" mode
  type` productions. Signature positions keep their annotations by
  design — A3 deletes the binder annotation only — so a `mode type`
  match there is a pattern-7 near-miss, not a site.
- [CLM-1]'s `own Bool` condition. Judged through [OP-5] exactly as
  `check` is, so it follows the `if` condition judgment without an
  edit, the same reasoning as the [OWN-13] entry below.
- [ENT-3] S10's `match_stmt`/`value_match` over `read_once` and the
  transfer operations. The scrutinee is a payload enum, so it keeps
  `match` under GRAM-6's type-driven rule; S1 is the Bool-scrutinee
  source and is already a site.
- [OWN-13]'s *scrutinee* ownership, as distinct from its delivery
  sentence, which is a site (§3). `Bool` is copy under [OWN-1]
  ("tag-only enums … copy on use"), so a Bool scrutinee is a copy use
  in both spellings and the `if` condition needs no ownership
  analogue; it inherits [OP-5]'s condition judgment exactly as `check`
  does. This half of the rule is why [OWN-13] appeared on the cleared
  list of the prose sweep and stayed off the site list until pattern 7
  read the other half: the scrutinee question really is closed, and
  the defect is one sentence further down, in what the arms *deliver*.
- [TYPE-6] lexical scoping. Branch blocks are ordinary lexical scopes
  under "A nested lexical declaration may not shadow an entry live at
  that declaration"; no node-kind enumeration to extend.
- [DIAG-1]'s `IDENT "." IDENT ("("|"<")` boundary rule. Still a raw
  lexical defect citing FORM-3; after the batch no dotted spelling
  takes `<` at all, which strengthens rather than falsifies it.
- [DIAG-1]'s production-ordinal identity. Inserting productions shifts
  later ordinals, but the identity is derived per version and
  cross-version stability is deferred wholesale by owner ruling
  (v0.22 header), so it is not a defect.
- The SYS operation-table properties (`Every … operation spelling
  satisfies IDENT and contains no dot`). Scoped to the system table,
  whose spellings are untouched dotless IDENTs.
- [FORM-2]'s "A match-arm header is therefore one level inside its
  match". Still true of `match`; `if` rendering is governed solely by
  the batch's dedicated FORM-2 sentence.
- `expr_stmt := call ";"` is unchanged, so infix does not become a
  statement; `set`, `return`, `give`, and `check` take `expr` in both
  versions, so infix reaches exactly the positions the named-call
  spelling reached.
