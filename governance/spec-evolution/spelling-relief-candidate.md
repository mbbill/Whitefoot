# FLOOR-5 spelling relief — batch candidate (SWEEP A + C)

Status: CANDIDATE, DRAFT (2026-08-07; owner overnight standing instruction —
draft-and-review lane only, activation sequenced later; adversarial-review
fixes F1–F11 and residue findings R1–R3 of the FLOOR-5 review applied per
lead direction, see `research/investigations/obligation-discharge/
CANDIDATE-REVIEW.md` at 710f4b3). Non-authoritative.
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
> twenty-four existing rules modified at forty-six verbatim-anchored
> modification sites (a site is one contiguous verbatim-anchored
> replacement; every site in this candidate is anchored — no prose
> sweeps): FORM-2 (block-bearing list; the sole-governance `if`
> rendering sentence; value-if prefix line), GRAM-1 (six compound
> tokens; operator-form munch with the minus/arrow/literal
> disambiguation; the `infix` node-kind sentence), GRAM-4
> (annotation-free `let_stmt`; `if_stmt` and `value_if`), GRAM-5
> (left-factored `expr` with `infix_tail` and `infix_op`; call
> targs retained for the classes that keep them), GRAM-6 (rewritten:
> type-driven conditional forms; the universal flattening mandate with
> [GIVE-1] ownership of the undeliverable `value_if` case), GRAM-7
> (rewritten: two `if` node kinds beside the two
> `match` kinds), GIVE-1 (complete replacement: the derived-delivery
> rule), GRAM-9 (two sites: infix operands; the forwarding-let
> parenthetical), TYPE-5 (rewritten: statement-local derivation; the
> total retained-argument class; boundaries stay fully explicit), OP-1
> (four sites: row selection reworded off the written argument;
> op-column respells; infix resolution by exact operator
> token; `ModeWords` derived from both suffix carriers), OP-2 (two
> sites: operand-derived selected type replaces the explicit-argument
> judgment, binary and negation paragraphs), OP-7 (three sites: infix
> convention; two keyed-on-the-selected-type rewrites), OP-8 (two
> sites plus one retention: sat respells; the contiguous `eeq`/`ene`
> operand-derived identity; `fneg(finf<T>())` retained), ERR-2
> (Bool exhaustiveness via `if`, with the empty-then/empty-else
> asymmetry stated), ERR-3 (full-sentence re-anchor with the derived
> type), FN-8 (three sites plus one retention: infix conditions;
> example; `value_if` named
> in the exclusion list; clause lets annotation-free retained), EFF-2
> (traps
> contribution names bare infix arithmetic), DIAG-1 (three sites:
> attribution row 2's position guard and token list gain the infix
> operand; the
> typed-call location paragraph replaced for the retained-argument and
> infix classes),
> DIAG-3 (bare-operator overflow record), ENT-2 (value_if joins the
> term-root forms), ENT-3 (seven sites: S1 origin and establishment;
> S4/S5/S6/S7/S9 respells), ENT-5 (the branch-continuation join in the
> CFG idiom, with the empty join defined for `match` and `if` alike),
> ENT-6
> (fallback respelled), EX-1 (complete worked-example rewrite) — plus
> three R3-PROVISIONAL register settlements (match-only conditionals
> and no-if; prefix arithmetic surface; the interior annotation
> mandate's body half, precedent question O8). Tokens: +2 exact fixed
> lowercase atoms (`if`, `else`); +20 operator terminal spellings
> (`+ - * / %`, `== != <= >=`, and eleven suffixed operator forms);
> operation-table op-column respells 20 spellings, shrinking
> `DotlessOperationNames` and `ReservedLowerNames` by the four dotless
> comparisons `ieq` `ine` `ile` `ige`; grammar productions +4
> (`if_stmt`, `value_if`, `infix_tail`, `infix_op`; total 69 — the
> `infix` node kind is carried by `infix_tail`'s 1:1 mapping, not a
> phantom production),
> with `stmt`, `let_stmt`, and `expr` modified; exception clauses
> +0/-0; sections +0. The accepted-program set changes as one canonical
> respelling plus two deliberate narrowings: the Bool-scrutinee
> `match` is rejected (spell `if`), and a value initializer with an
> empty
> delivery set is rejected (spell the statement form and drop the
> binding). Delivery-type disagreement is a re-citation, not a
> narrowing: a v0.22-accepted program's `give`s each matched the one
> written annotation and therefore agree with each other, so none is
> newly rejected there, while `give`s that agreed with each other
> against a wrong annotation join the widening below. The error
> classes that lived only in deleted
> bytes die with their bytes. Every operation's semantics, every trap,
> every discharge judgment, and the claim lifecycle are unchanged; bare
> infix arithmetic is byte-for-byte today's `.trap` mode under a
> shorter constant. Selection ground: evidence-selected under the
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
a terminal-membership rejection. Canonical spacing: no operator token
joins either FORM-2 attachment set, so every infix operator renders with
one space on each side, keeping `a - 1_u64` (operator) and `-1_u64`
(literal) lexically distinct in canonical bytes.

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

Mechanical check (process fix per the review): the decision rows above
were checked for pairwise SELECT-set disjointness by a scratch script
(do_not_scan, deleted after use) on 2026-08-07 — all seven decisions
pass, and the two expr-critical positions of the rewritten [EX-1]
(`match deref(p) +checked 2_i32 {` and `if ilt(x, 0_i32) {`) trace
through the factored productions and parse. Re-run after the NEW-1
production change: the dropped `infix` production appeared on no
right-hand side, so every decision row is identical, and all seven
pass again (same-day re-run, script recreated and deleted). Verifier
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

**[GRAM-1]** As §2's token-formation paragraph: "`->` and `=>` are the
two compound punctuation tokens." becomes "`->`, `=>`, `==`, `!=`,
`<=`, and `>=` are the six compound punctuation tokens."; the
operator-form clause is added after the numeric-form clause exactly as
§2 states it; and one node-mapping sentence is added: "`infix_tail` maps to the
`infix` node kind: a selected tail forms one `infix` node spanning the
complete `expr` — the atom and the tail — so the 1:1
production-to-node mapping is preserved by the factored recognition."

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

**[OP-1]** Four sites. (0) The row-selection sentence "Later typed
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
`ilt`/`igt` remain members.

**[OP-2]** Two sites. The binary judgment paragraph "Each operation in
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

The negation paragraph's opening "Each negation call has exactly one
explicit type argument and exactly one positional atom operand of that
exact selected type." becomes "Each negation call carries no written
type argument and has exactly one positional atom operand, whose exact
type is the selected type — restricted to the signed subset exactly as
stated below." (its remaining sentences unchanged; TYPE-7 exclusivity,
exact table result, and consuming-construct ownership are untouched).

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
> error, the second operand atom. The cited rule is the rule selected
> by [OP-2]: FN-2, OP-1, or TYPE-5.

Attribution row 2's "are `(IDENT, "(")`, `(IDENT, "<")`, `(OPNAME,
"(")`, `(OPNAME, "<")`, `(TYPEID, "(")`, or `(TYPEID, "<")`, the
rejection cites [GRAM-9]" gains: "; in an infix-operand occurrence, a
two-token start whose second token is an operator token — the
forbidden nested-infix start — likewise cites [GRAM-9]".

**[DIAG-3]** "For an executed `iadd.trap`, `isub.trap`, or `imul.trap`
overflow, `rule_id` is `OP-2`, `message` is `integer overflow`, and
`node_path` is the trapping `call` node." becomes "For an executed
bare `+`, `-`, or `*` overflow, `rule_id` is `OP-2`, `message` is
`integer overflow`, and `node_path` is the trapping `infix` node; a
bare `/` or `%` contract violation is a table-operation contract check
at its `infix` node."

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
S4's "or a call `len<T>(P)` over such a place — read as the length
term len(P)" becomes "or a call `len(P)` over such a place — read as
the length term len(P)". S5's "for `let x: own T = lit;`, x =
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

One canonical respelling plus two deliberate narrowings. Respelling:
every existing program's canonical bytes change and old bytes reject
under FORM-1 — migrated mechanically (§5). Widening: the error classes
that lived only in deleted bytes die with them (a wrong let annotation,
a wrong or missing value-op type argument — unwritable states now,
including `give`s that agreed with each other against a wrong
annotation). Narrowings: (1) a Bool-scrutinee `match` is rejected
(spell `if`) — the
type-driven one-form-per-class rule T3 requires; (2) a value
initializer with an empty delivery set — every arm or branch leaves by
`return` or `break` — is rejected at the
`let_stmt` node (review F4; v0.22 accepts it with the annotation
supplying the never-read binding's type; migration: spell the
statement form and drop the binding). Delivery-type disagreement is a
re-citation, not a narrowing (review NEW-4): a v0.22-accepted
program's `give`s each matched the one written annotation and
therefore agree with each other, so the agreement rule newly rejects
none — only the citation and location change (GIVE-1 at the second
divergent `give`, where TYPE-5 at each mismatching `give` stood
before). Every operation's semantics, every
trap, every discharge judgment, and the claim lifecycle are unchanged;
the `if` continuation joins facts exactly as the `match` continuation
it respells [ENT-5].

## 5. Corpus migration (mechanical, printer-driven; measured 2026-08-07
against the respelled v0.22 corpus, canonical `.wf` sources only, 399
files excluding the worktree mirror)

- Deleted-class type arguments: **1353** occurrences (settled by the
  reviewer's final per-callee enumeration across the 399 files; the
  earlier 1357/1356 figures both admitted invalid-spelling fixtures.
  The first draft's 1260 under-counted — it omitted the float family
  and `box_new`; this count is the complete deleted class: every table
  operation except the six retained). Separately: five
  deliberately-invalid OPNAME spellings in negative conformance
  fixtures (`irotl.trap`, `idiv.wrap`, `fneg.strict`, `iadd.bogus`,
  `add.wrap`) are NOT deleted-class sites — they never parse to a
  table-op call, and the migration never touches them. Retained-class sites, untouched: **101**
  (`cvt`/`reinterpret`/`array_new`/`arena_new`/`finf`/`fnan` — the F3
  orphans now have their one legal spelling, unchanged).
- Let annotations deleted: 1748 binders (reviewer-reproduced).
- Bool matches to `if`/`else`: 257 `True()`-arm matches
  (reviewer-reproduced), including the else-if flattening of the
  corpus's Bool ladders.
- Infix respells: ~384 add/sub/mul/div/rem sites led by 229 `iadd.wrap`
  and 47 `iadd.trap` (reviewer-reproduced), plus the `== != <= >=`
  sites; 56 `ilt`/`igt` sites keep named calls under O1, losing only
  their type arguments.
- `check` statements: **389** — untouched (C3 open, O2). Measured as
  line-leading `check` statements, which [FORM-2]'s line-bearing rule
  makes exact, and confirmed by the `else trap` count, unique to
  `check_stmt`; the twenty additional loose-grep occurrences live
  inside `doc` strings and trap messages (both earlier figures — the
  draft's 404 and the review's 409 — were loose counts).
- Empty-delivery-set value initializers (narrowing 2): any corpus
  instance is respelled to the statement form in the same migration;
  none is expected (the shape is pointless), and the migration pass
  reports each one it rewrites.
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
`=` is in neither attachment set, `==` cannot arise by attachment); O7
the empty then-block admitted with the asymmetry stated in ERR-2.

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
  is recorded as proportionate to A3's 1748 deleted annotations.

No other contradiction between the batch and v0.22 remains: every
collision — TYPE-5's mandate, GRAM-6's no-if sentence, GRAM-7's
two-kind discipline, OP-2's explicit-argument judgments, OP-7/OP-8's
keyed-on-the-argument sentences, GRAM-9's forwarding parenthetical,
DIAG-1's call-node locations, ENT-2/ENT-3/ENT-5/ENT-6's fact machinery,
FN-8's clause subset, ERR-2/ERR-3, the register entries, and EX-1's
bytes — is an enumerated modification above.
