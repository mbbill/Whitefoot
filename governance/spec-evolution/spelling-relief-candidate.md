# FLOOR-5 spelling relief — batch candidate (SWEEP A + C)

Status: CANDIDATE, DRAFT (2026-08-07; owner overnight standing instruction —
draft-and-review lane only, activation sequenced later). Non-authoritative.
This document is the complete spelling-relief delta against the exact text
of the active `spec/kernel-spec-v0.22.md` (installed 817a8a7). Authority:
`research/investigations/spelling-relief/SWEEP.md`, implementing its A and
C verdicts exactly as scoped by the lead: A1 (value-op type-argument
deletion, per-op uniform), A3 (body-let mode/type annotation deletion), A4
(Bool-scrutinee match becomes `if`/`else` with the SWEEP-pinned layout), C1
(infix respellings for the hottest table operations, modes as suffixed
operators, semantics unchanged), and C3 (check-to-claim unification) which
is assessed below and found NOT pure spelling under v0.22 — listed open
(O2), not implemented. An adversarial pass with the residue-hunt axis is
expected before approval.

Two structural findings are surfaced up front rather than discovered late:

- The bare `<` and `>` comparisons collide with generic-call type
  arguments at exactly strong-LL(2)'s two-token horizon — `(IDENT, "<")`
  begins both `a < b` and `f<T>(…)` — and both bytes sit in FORM-2's
  attachment sets (they would render `a<b`). C1 is therefore drafted with
  `==`, `!=`, `<=`, `>=` infix and `ilt`/`igt` remaining named calls,
  and the fork is O1 with three resolution options.
- A3 on a value-producing initializer removes GIVE-1's declared-type
  anchor, so GIVE-1 gains a derived-common-delivery-type rule — a new T3
  uniqueness/normalization rule, flagged O4 as the batch's one genuinely
  new judgment.

## 1. Proposed version-header paragraph

> Status: REVIEW CANDIDATE vNEXT (2026-08-07; FLOOR-5 spelling relief:
> value-op type-argument deletion, body-let annotation deletion,
> `if`/`else` for Bool conditionals, infix arithmetic and comparison
> spellings). Deletes the type argument from every value-typed table
> operation — the operand atoms are typed, so the selected type is
> uniquely reconstructed per [OP-2]'s rewritten derivation, and only the
> type-choosing structural operations (`cvt`, `reinterpret`, `array_new`)
> keep type arguments, everywhere and mandatorily. Deletes the `: mode
> type` annotation from every `let` binder — the binder's mode and type
> are exactly what its right-hand side produces, statement-locally
> ([TYPE-5] rewritten; literals keep mandatory suffixes [FORM-5], so
> every right-hand side stays self-typed); a value initializer's type is
> the common exact mode and type of its delivering `give`s ([GIVE-1]
> rewritten). Replaces the Bool-scrutinee `match` with `if`/`else`: a
> Bool condition takes `if`, an enum scrutinee takes `match`, each the
> sole form for its class; an `else` with an empty block is rejected
> (spell the else-free form), an `else` whose block is exactly one `if`
> must flatten to `else if`, and the canonical layout is multi-line with
> the `} else {` join line, no one-line form. Respells the hottest
> integer table operations as infix with modes as operator suffixes —
> bare `+` `-` `*` `/` `%` carry the trapping-mode semantics unchanged,
> `+wrap`-class suffixed operators carry wrap/checked/sat, and `==` `!=`
> `<=` `>=` respell the four nonstrict comparisons — one constant
> spelling per operation as today, no precedence table because [GRAM-9]
> ANF admits exactly one operation per expression. `ilt` and `igt`
> remain named calls (O1). Specification delta: numbered rules +0/-0;
> twenty-two existing rules modified at thirty-two verbatim-anchored
> modification sites (a site is one contiguous verbatim-anchored
> replacement): FORM-2 (if layout; block-bearing
> list; value-if prefix line), GRAM-1 (operator token formation: `-`
> disambiguation, four new compound tokens, suffixed-operator munch),
> GRAM-4 (annotation-free `let_stmt`; `if_stmt` and `value_if`
> productions), GRAM-5 (`infix`/`infix_op` productions; call targs
> retained for the callee classes that keep them), GRAM-6 (rewritten:
> type-driven conditional forms; one-operation infix, no precedence),
> GRAM-7 (rewritten for the two `if` node kinds beside the two `match`
> kinds), GIVE-1 (derived common delivery type; if-arm delivery),
> GRAM-9 (infix operands are atoms), TYPE-5 (rewritten:
> statement-local derivation replaces the universal annotation mandate;
> boundary surfaces stay fully explicit), OP-1 (op-column respells;
> infix resolution by exact operator token), OP-2 (operand-derived
> selected type replaces the explicit-type-argument judgment), OP-7
> (naming convention gains the infix column), OP-8 (respelled lowering
> mentions), ERR-2 (Bool exhaustiveness moves to `if`), ERR-3
> (annotation-free propagate spelling), FN-8 (annotation-free clause
> lets; infix conditions admitted; example respelled), EFF-2 (traps
> contribution names bare infix arithmetic), DIAG-1 (infix-operand
> attribution; typed-call location rows), DIAG-3 (bare-operator overflow
> record), ENT-3 (S1 via `if`; S5/S6/S7 respelled), ENT-6 (fallback
> respelled), EX-1 (complete worked-example rewrite) — plus three
> R3-PROVISIONAL register settlements (match-only conditionals and
> no-if; prefix arithmetic surface; the interior annotation mandate's
> body half; each settled by this batch's SWEEP evidence, with the
> boundary half of the annotation mandate remaining). Tokens: +2 exact
> fixed lowercase atoms (`if`, `else`); +20 operator terminal spellings
> (`+ - * / %`, `== != <= >=`, and eleven suffixed operator forms);
> operation-table op-column respells 20 spellings (the four modes of
> add/subtract/multiply, the two of divide/remainder, and `ieq` `ine`
> `ile` `ige`), shrinking `DotlessOperationNames` and
> `ReservedLowerNames` by the four dotless comparisons; grammar
> productions +4 (`if_stmt`, `value_if`, `infix`, `infix_op`), with
> `stmt`, `let_stmt`, and `expr` modified; exception clauses +0/-0;
> sections +0. The accepted-program set changes as one canonical
> respelling plus two deliberate class changes: it widens where deleted
> annotations and type arguments made mismatches expressible (those
> error classes die with their bytes), and it narrows by the
> Bool-scrutinee `match` (rejected: spell `if`) — semantics of every
> operation, check, discharge judgment, and trap are otherwise
> unchanged. Selection ground: evidence-selected under the four-test
> spelling rule — SWEEP rows A1/A3/A4/C1 with their T1 unique-
> reconstruction arguments, the retained-ANF precedence-free property,
> and the owner rulings of record. These bytes are non-authoritative
> until the grammar check, derived-material review, full-document hash,
> exact owner approval, and active-target installation complete.

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

with every other GRAM-4 line byte-identical (`check_stmt`, `claim_stmt`,
`match_stmt`, `value_match`, `arm`, and the rest unchanged). [GRAM-5]'s
expression head becomes:

```
expr           := atom | call | construct | infix
infix          := atom infix_op atom
infix_op       := "+" | "+wrap" | "+checked" | "+sat"
                | "-" | "-wrap" | "-checked" | "-sat"
                | "*" | "*wrap" | "*checked" | "*sat"
                | "/" | "/checked" | "%" | "%checked"
                | "==" | "!=" | "<=" | ">="
```

with `call := callee targs? …` retained byte-identical — `targs` remains
grammatical on every call and the checker enforces the per-callee-class
policy [TYPE-5, OP-2]: required for user-generic calls, system-operation
region arguments, and the type-choosing operations; forbidden elsewhere.

Token formation [GRAM-1]: `if` and `else` are exact fixed lowercase atoms
(auto-excluded from IDENT by [FORM-3], no text change there). The
compound-token set grows from two to six: `->`, `=>`, `==`, `!=`, `<=`,
`>=`; a lone `!` remains a raw lexical defect (it exists only inside
`!=`). An operator form starts with `+`, `*`, `/`, or `%`, or with a `-`
that is immediately followed by neither a decimal digit (which starts a
numeric form, unchanged) nor `>` (which forms `->`, unchanged), and
continues through the maximal `[a-z]*` suffix; the suffix must be empty
or one of `wrap`, `checked`, `sat` per the closed `infix_op` list, and
any other suffix is a terminal-membership rejection. Canonical spacing:
none of the operator tokens joins either FORM-2 attachment set, so every
infix operator renders with one space on each side (`a + b`,
`a +wrap b`), which also keeps `a - 1_u64` (operator) and `-1_u64`
(literal) lexically distinct in canonical bytes.

Strong-LL(2): the `stmt` decision gains the `if` arm on its unique first
token; the `expr` decision distinguishes `infix` from a bare `atom` at
the second token (an operator token follows the first atom; no other
continuation of a complete atom begins with an operator token); `else`
follows a closed then-block, competing with nothing. The deliberate
exclusion: `(IDENT, "<")` cannot select between a comparison and a
generic call at two tokens, which is why bare `<` and `>` are not in
`infix_op` (O1). Nested infix does not exist ([GRAM-9]: operands are
atoms), so there is no precedence, associativity, or parenthesization
surface at all.

Verifier expectations: fail-closed against the v0.22 tables (grammar-
extending), recorded at proposal; the grammar-path task extends the
lexer/parser first. Post-extension: productions 65 + 4 = 69; terminal
predicates gain the two keywords and the operator forms and lose the
respelled OPNAME/dotless spellings; exact counts are established by that
task.

## 3. Modified rules (complete replacement deltas, verbatim anchors)

**[FORM-2]** Three sites. The block-bearing list "the body of `fn_decl`,
`requires_block`, `loop_stmt`, `region_stmt`, `match_stmt`,
`value_match`, and `arm`" gains `if_stmt` and `value_if` and their
then/else blocks. New rendering sentence appended to that paragraph: "An
`if` renders its introducer through `{` on one line; an `else` renders as
the join line `} else {`, and an else-if chain as the join line
`} else if` through that `if`'s `{`; the final close is `}` on its own
line. No one-line `if` form exists." The value-match prefix sentence "A
value-match let places its complete let prefix and the `match` introducer
through `{` on one line." becomes "A value-match or value-if let places
its complete let prefix and the `match` or `if` introducer through `{`
on one line."

**[GRAM-1]** As §2's token-formation paragraph: the compound-token
sentence "`->` and `=>` are the two compound punctuation tokens."
becomes "`->`, `=>`, `==`, `!=`, `<=`, and `>=` are the six compound
punctuation tokens.", and the operator-form clause is added after the
numeric-form clause — a `-` immediately followed by a decimal digit
still starts a numeric form, `->` still wins its compound, and every
other `+ - * / %` occurrence forms one operator token with its maximal
`[a-z]*` suffix, membership decided by the closed `infix_op` list at
terminal membership.

**[GRAM-4]** As §2: `let_stmt` loses `":" mode type` and gains the
`value_if` alternative; `if_stmt` and `value_if` are added; `stmt` gains
`if_stmt`.

**[GRAM-5]** As §2: `expr` gains `infix`; `infix` and `infix_op` are
added; everything else byte-identical.

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
> condition takes `if`/`else`, an enum scrutinee takes `match`, and each
> is the sole legal form for its class — a `match` whose scrutinee has
> type `Bool` is a hard error citing GRAM-6 at the scrutinee `expr` node
> (spell `if`). An `if` condition must have exact value mode and type
> `own Bool` under exactly the [OP-5] condition judgment, TYPE-7
> exclusivity included; every other condition failure cites GRAM-6 at
> the condition `expr` node. An `else` whose block is empty is a hard
> error citing GRAM-6 (spell the else-free `if`); an `else` whose block
> contains exactly one `if_stmt` and nothing else is a hard error citing
> GRAM-6 (spell `else if`). A conditional value is a `let`-initializer
> `match` or `if` [GRAM-7, GIVE-1]; iteration is `loop` + `break`.

(The retained subscript sentence stays byte-identical at the rule's
end.)

**[GRAM-7]** Complete replacement, extending the two-node-kind discipline
to `if`:

> [GRAM-7] `match` and `if` each have one source body shape and two
> distinct core-tree node kinds: `match_stmt`/`if_stmt` for statements,
> `value_match`/`value_if` for a `let` initializer. The pairs never
> compete at one grammar decision: the statement forms begin at the
> statement boundary, the value forms only after the complete
> `let IDENT =` prefix, so the parser decides from source position
> alone, without type, name-resolution, or checker context. A value form
> is value-producing, and every arm or branch body must satisfy the
> complete [GIVE-1] delivery judgment for its binding; a `value_if`'s
> `else` is grammatically mandatory [GRAM-4] because a missing branch
> could not deliver. Statement forms produce no value; their bodies act
> by effect. `return`-position conditionals deliver by returning from
> branches; there is no helper-function conditional-initialization
> idiom, and value-production is confined to the `let` initializer, so
> neither construct ever occupies an arbitrary expression position.

**[GIVE-1]** Two sites. The declared-type anchor "`e` must have that
`let`'s declared `mode type` (stated at the binder [TYPE-5], never
inferred from arms)." becomes "every delivering `give` of one value
initializer must have one identical exact mode and type, which is the
binding's derived mode and type [TYPE-5]; a delivering `give` whose
exact mode or type differs from an earlier delivering `give` of the same
initializer is a hard error citing GIVE-1 at the later `give_stmt` node
— derivation is agreement over the closed delivery set, never a join,
widening, or common-supertype rule." The delivery recursion sentence
"or a `match_stmt` every arm of which delivers relative to that same
value match" becomes "or a `match_stmt` every arm of which, or an
`if_stmt` with `else` both branches of which, delivers relative to that
same value initializer" (an else-free `if_stmt` has a continuing false
edge and never delivers). Every "value match" occurrence generalizes to
the value initializer; the give-legality, single-give, and borrow-region
sentences are unchanged.

**[GRAM-9]** "Every call argument, construct field value, and subscript
offset is an `atom` [GRAM-5]" becomes "Every call argument, construct
field value, infix operand, and subscript offset is an `atom` [GRAM-5]".

**[TYPE-5]** Complete replacement:

> [TYPE-5] Statement-local typing; boundary-explicit facts. A `let`
> binder's mode and type are derived, never written: exactly the mode
> and type its selected right-hand side produces — an `ordinary_let_rhs`
> from its expression, which is always self-typed (operands are typed
> atoms, calls are typed by their [FN-1]/[OP-1]/[SYS-2] signatures,
> literals carry mandatory suffixes [FORM-5], constructions name their
> nominal); a `propagate_let_rhs` from the propagated Ok payload
> [ERR-3]; a `value_match` or `value_if` from the derived common
> delivery type [GIVE-1]. This is unique reconstruction within one
> statement, not inference: no binder's type depends on a later
> statement, an expected type, or any use site, and no two derivations
> can disagree [FORM-1]. Call sites state explicitly exactly what their
> callee class requires: type, region, and const arguments for user
> generics [FN-2], region arguments for system operations [SYS-2], and
> type arguments for the type-choosing operations `cvt`, `reinterpret`,
> and `array_new` [OP-6, CONST-1] — required there, forbidden on every
> value-typed table operation, whose selected type is operand-derived
> [OP-2]. Argument types match declared parameter types exactly. After
> [SET-1] derives a writable target place of type T, the right-hand
> side of `set p = e;` must produce exactly `own T`; there is no mode
> coercion, type conversion, or target-selected operation overload.
> After the TYPE-7 implicit-read exclusivity below, a different
> right-hand-side mode or type is a hard error citing TYPE-5 at the
> complete `expr` child of the `set_stmt`, carrying expected `own T`
> and the actual mode and type. Redundant-explicit facts remain
> mandatory at every trust boundary — signatures with full modes,
> types, effect rows, and regions [FN-1], construction field names
> [GRAM-8], match binders [GRAM-10], call argument names [GRAM-11] —
> and are deleted exactly where reconstruction is unique and no
> transposition risk exists.

**[OP-1]** Two sites. The table's op column respells twenty spellings in
place, rows otherwise unchanged: `iadd.wrap isub.wrap imul.wrap` become
`+wrap -wrap *wrap`; `iadd.trap isub.trap imul.trap` become `+ - *`;
`iadd.checked isub.checked imul.checked` become `+checked -checked
*checked`; `idiv.trap irem.trap` become `/ %`; `idiv.checked
irem.checked` become `/checked %checked`; `iadd.sat isub.sat imul.sat`
become `+sat -sat *sat`; `ieq ine ile ige` become `== != <= >=`; `ilt`
and `igt` keep their spellings (O1). Resolution paragraph addition after
the operation-family sentence: "An `infix_op` token resolves to its
exactly spelled operation by the operator table row; infix resolution
consults no name domain, and an operator token is never a declaration,
callee IDENT, or OPNAME." Derived-set consequence: `ieq`, `ine`, `ile`,
`ige` leave `DotlessOperationNames` and therefore `ReservedLowerNames`;
`ilt`/`igt` remain members.

**[OP-2]** One paragraph replaced. "Each operation in the preceding
paragraphs has exactly one explicit type argument. In a concrete call it
is one member of the closed integer-type set. In a symbolic generic body
it may instead be one live type parameter whose bound resolves to PRE-1
`Int`; every concrete FN-2 instantiation substitutes one member of the
closed set and uses the corresponding mathematical semantics above. No
unbound or differently bounded generic type is admitted. Every call has
exactly two positional atom operands, both of the exact selected type.
Absence of the required explicit type argument cites [FN-2]. A different
type-argument count, a region or const argument in that position, a
concrete type outside the closed integer set, an inadmissible generic
type, or a wrong operand count cites [OP-1]." becomes:

> Each operation in the preceding paragraphs carries no type argument:
> its selected type is derived from its operands. Both operands must
> have one identical exact type — a member of the closed integer-type
> set or, in a symbolic generic body, one live type parameter whose
> bound resolves to PRE-1 `Int`, with every concrete FN-2 instantiation
> substituting one closed-set member and the corresponding mathematical
> semantics above. That common exact type is the selected type; the
> derivation is agreement, never widening, conversion, or preference.
> Operands of two different exact types are a hard error citing TYPE-5
> at the second operand atom in source order. A written type argument
> on one of these operations, a region or const argument, a concrete
> operand type outside the closed integer set, an inadmissible generic
> type, or a wrong operand count cites [OP-1].

The negation paragraph's parallel sentences take the same derivation
(one operand; its exact type is the selected type; signed subset
restriction unchanged), and the TYPE-7 exclusivity, exact-table-result,
and consuming-construct sentences are unchanged.

**[OP-7]** One sentence appended: "A respelled operation's operator
token is its one constant spelling — bare operators carry the
trapping-overflow mode, suffixed operators carry `wrap`, `checked`, and
`sat`, and the four nonstrict comparisons are `==` `!=` `<=` `>=` —
under exactly the same one-spelling-per-operation discipline; the
`i`-prefix convention continues to govern the operations that keep
named spellings."

**[OP-8]** Spelling mentions only: `iadd.sat`/`isub.sat` read
`+sat`/`-sat`, `imul.sat` reads `*sat`, and the shift/rotate/abs/float
sentences are unchanged (those operations keep named spellings).

**[ERR-2]** "Every `match` is exhaustive over declared variants; there
are no wildcard arms." gains: "Bool exhaustiveness is carried by `if`:
an else-free `if` is the empty-alternative form, an `if` with `else`
covers both, and a Bool-scrutinee `match` is rejected at GRAM-6, so no
match arm ever spells `True()` or `False()` as a scrutinee test."

**[ERR-3]** "Propagation: `let x: own T = propagate e;` requires
`e : own Result<T, E>`" becomes "Propagation: `let x = propagate e;`
requires `e : own Result<T, E>`, and x's derived mode and type are
`own T` [TYPE-5]"; the rest byte-identical.

**[FN-8]** Three sites. "Every computation in the block must be an ANF
[GRAM-9] call to a non-trapping, total operation-table row with effect
`pure`; the final check condition is either a Bool clause atom or one
such call returning Bool." becomes "Every computation in the block must
be an ANF [GRAM-9] call to, or infix spelling of, a non-trapping, total
operation-table row with effect `pure`; the final check condition is
either a Bool clause atom or one such operation returning Bool." The
example "(for example `len<u8>(deref(out))`)" becomes "(for example
`len(deref(out))`)". The structural-pass sentence naming
"`let_stmt` nodes whose selected right-hand side is `ordinary_let_rhs`"
is unchanged — clause lets are annotation-free like every let [GRAM-4],
and the open question O3 records the boundary-reading alternative.

**[EFF-2]** "exhibit `traps` iff either contains any `.trap` op,
`check`, `claim`, or a call" becomes "exhibit `traps` iff either
contains any trapping-mode operation — a bare infix arithmetic operator
(`+`, `-`, `*`, `/`, `%`) or a `.trap` OPNAME — `check`, `claim`, or a
call".

**[DIAG-1]** Two sites. Attribution row 2's "an `atom` occurrence in
`atom_list`, `fieldinit`, or the subscript offset" becomes "an `atom`
occurrence in `atom_list`, `fieldinit`, an `infix` operand, or the
subscript offset". The typed-call location sentence "a missing explicit
type argument uses `SourceNode` at the `call` node and that node's
complete source extent" is scoped to the callee classes that still carry
type arguments; for a value-typed table operation the class is
unreachable and the operand-type error follows OP-2's rewritten
second-operand attribution.

**[DIAG-3]** "For an executed `iadd.trap`, `isub.trap`, or `imul.trap`
overflow, `rule_id` is `OP-2`, `message` is `integer overflow`, and
`node_path` is the trapping `call` node." becomes "For an executed bare
`+`, `-`, or `*` overflow, `rule_id` is `OP-2`, `message` is `integer
overflow`, and `node_path` is the trapping `infix` node; a bare `/` or
`%` contract violation is a table-operation contract check at its
`infix` node."

**[ENT-3]** Four sites. S1's origin clause (a) "it is a call to one of
`ieq`, `ine`, `ilt`, `ile`, `igt`, `ige` [OP-2] whose two operands are
each a term or constant" becomes "it is an infix comparison `==`, `!=`,
`<=`, `>=`, or a call to `ilt` or `igt` [OP-2], whose two operands are
each a term or constant". S1's establishment sentence "For a
`match_stmt` or `value_match` whose scrutinee has comparison origin R, R
is established at the `True()` arm's entry and R's exact negation at the
`False()` arm's entry." becomes "For an `if_stmt` or `value_if` whose
condition has comparison origin R, R is established at the then-block's
entry and R's exact negation at the else-block's entry; for an else-free
`if_stmt`, the negation is established on the false edge, which joins
the then exit at the continuation [ENT-5]." S6's forms respell:
"`let b: own buffer<T> = buffer_new<T>(n, v);`" becomes
"`let b = buffer_new(n, v);`", "`let m: own u64 = len<T>(P);`" becomes
"`let m = len(P);`", and the slice_of form loses its annotation
likewise; S5's cvt form keeps its type arguments (type-choosing) and
loses only the binder annotation. S7's shapes respell:
"`let s: own T = iadd.wrap<T>(p, k);`" becomes "`let s = p +wrap k;`"
(and symmetrically `-wrap`), the trap forms become bare "`p + k`" and
"`p - k`", and the checked-origin scrutinee becomes the infix
"`p +checked k`" / "`p -checked k`"; every side condition, range
premise, and kill discipline is byte-identical.

**[ENT-6]** The fallback "in canonical ANF, one `let` binding
`len<T>(P)` followed by one `claim` on, or `match` over, the admitted
comparison [CLM-1, ENT-3]" becomes "in canonical ANF, one `let` binding
`len(P)` followed by one `claim` on, or `if` over, the admitted
comparison [CLM-1, ENT-3]".

**[EX-1]** Complete replacement of the worked example's program bytes
(canonical under every rule of this batch: derived binders, infix,
if/else with the pinned layout, else-if flattening, named `ilt` under
O1):

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
leave the register — settled by this batch's SWEEP four-test evidence
and rulings; "interior annotation mandate (TYPE-5 — round-2 verdict
still needs_evidence)" reduces to its surviving boundary half
("boundary annotation surface (TYPE-5)"). "statement-only match
(GRAM-7)" remains (value forms existed before this batch; the register
entry's question is untouched).

## 4. Acceptance-set delta

One canonical respelling plus two deliberate class changes. Respelling:
every existing program's canonical bytes change (annotations, type
arguments, Bool matches, respelled operations), and old bytes reject
under FORM-1 — the ordinary consequence, migrated mechanically (§5).
Widening: the error classes that lived only in deleted bytes die with
them (a wrong let annotation, a wrong or missing value-op type argument
— unwritable states now). Narrowing: a Bool-scrutinee `match` is
rejected (spell `if`) — the type-driven one-form-per-class rule that T3
requires; and a `value_match`/`value_if` whose delivering `give`s
disagree in exact type, previously a per-give mismatch against the
declared annotation, is now a GIVE-1 agreement error at the second
divergent give — same programs rejected, different citation. Every
operation's semantics, every trap, every discharge judgment, and the
claim lifecycle are unchanged; bare infix arithmetic is byte-for-byte
today's `.trap` mode under a shorter constant.

## 5. Corpus migration (mechanical, printer-driven; measured 2026-08-07
against the respelled v0.22 corpus, canonical `.wf` sources only)

- Value-op type arguments deleted: 1260 call sites across
  `tests/programs/` and `tests/conformance/cases/`.
- Let annotations deleted: 1748 binders.
- Bool matches to `if`/`else`: 257 `True()`-arm matches, including the
  else-if flattening of the corpus's Bool ladders (SWEEP's stated
  target).
- Infix respells: ~384 add/sub/mul/div/rem sites led by 229 `iadd.wrap`
  and 47 `iadd.trap`, plus the `== != <= >=` comparison sites; 56
  `ilt`/`igt` sites keep named calls under O1 (they lose only their
  type arguments).
- `check` statements: 404 — untouched (C3 open, O2).
- All migration is printer-driven per SWEEP's A/C batch rule (the
  canonical printer computes the new spelling from the old tree; zero
  semantic judgment); conformance sources and spelling-bearing manifest
  expectations respell in the same change under the standing
  derived-material rule; the derivation ledger, `docs/patterns.md`
  writer forms, and the register lines update in the same change.

## 6. Ruled and open list

Ruled (owner standing instruction, 2026-08-07): the batch itself — A1,
A3, A4, C1 as drafted; C3 assessed and deferred (O2).

Open (owner ruling needed; drafted with the recommended option):

- O1 — bare `<`/`>` infix: excluded as drafted. Two independent
  obstacles: `(IDENT, "<")` is strong-LL(2)-ambiguous against
  generic-call type arguments, and `<`/`>` sit in FORM-2's attachment
  sets (they would render attached). Options: (a) as drafted — `ilt`
  and `igt` stay named calls (56 corpus sites; the asymmetry against
  `<=`/`>=` is visible in EX-1's first branch); (b) respell the
  remaining targs introducer so `<` frees up — a breaking canonical
  change on every generic call and type; (c) new compound spellings for
  the strict comparisons. Recommendation: (a) now, revisit with (b)
  evidence if the asymmetry measurably hurts writers.
- O2 — C3 (check-to-claim) is NOT pure spelling under v0.22, on three
  grounds, and is deferred to its own batch: a claim requires a name
  (new semantic identity — per-function uniqueness, and the DIAG-3
  record carries the name where check carries the STRING, so trap
  bytes change); CLM-2 refutation would convert some accepted `check`s
  into rejections (an acceptance change no respelling may smuggle);
  and FN-8's structural pass requires a final `check_stmt`, so the
  unification forces an FN-8 semantics decision. The R2 residue
  finding (two spellings of one trap-check concern) stands tracked
  against that future batch.
- O3 — requires-clause lets: drafted annotation-free like every
  `let_stmt` (one grammar class, T4-uniform). Alternative: SWEEP's B
  row reads the requires block as signature surface ("the interface is
  the trust boundary"), arguing a split production that keeps clause
  annotations. The uniform reading is recommended — the clause lets
  are prologue scaffolding whose RHS is table-op-typed, and the final
  check is the boundary fact — but the fork is the owner's.
- O4 — the derived common delivery type is this batch's one new
  judgment (GIVE-1 agreement rule): T3-unique by construction
  (agreement over a closed set, no join), but it is a new
  normalization rule, not a deletion. Confirm.
- O5 — the `=[` cvalue attachment (v0.22 O1 ripple, owner-accepted
  as-ruled pending this batch): this batch proposes no change — no new
  evidence emerged against it, and infix introduces no new `=`
  adjacency (`=` keeps default spacing). Recommend closing it as
  standing.
- O6 — another-batch items restated from SWEEP D and C4, unchanged in
  scope: GRAM-9 nesting relaxation (deferred indefinitely by default),
  literal-class redesign, the counted range loop (obligation-discharge
  item 6), `.trap`/`.checked` OPNAME dissolution into goal-carrying
  bare ops (cross-track: when it lands, bare `+` is already the
  spelling it needs), and float/enum/bitwise infix (needs
  collision-free spellings; named ops meanwhile).
- O7 — an `if` with an empty then-block and non-empty `else` is
  admitted (mirrors the empty match arm it respells); the inverted
  spelling is a different checked program under ANF negation cost, so
  no T3 rule is drafted. Confirm.

No contradiction between the batch and v0.22 was found beyond the two
findings surfaced in the preamble (the `<`/`>` collision, resolved by
O1's exclusion; the GIVE-1 anchor, resolved by O4's derivation rule):
every other collision — TYPE-5's mandate, GRAM-6's no-if sentence,
GRAM-7's two-kind discipline, OP-2's explicit-type-argument judgment,
FN-8's clause subset, ERR-2's exhaustiveness wording, the register
entries, and EX-1's bytes — is an enumerated modification above.
