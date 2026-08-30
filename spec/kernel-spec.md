# Kernel Specification v0.40

Status: CANDIDATE v0.40 supersedes v0.39 b4d8e01eecd81bdda9c632093873d604ddfbd64d979a4884472907e456d69516
Prior versions: the immutable `spec/kernel-spec-vN.md` archives and the `ACTIVE-SPEC:` chain in `governance/APPROVALS.md`.

META-5 delta declaration: numbered rules +0/-0 (138 remain); grammar productions +0/-0 (75 remain); unique fixed lowercase grammar atoms +0/-0; writer operation spellings +0/-0; opaque system nominal spellings +0/-0; runtime-trap families +0/-0; entry forms +0/-0; contract block forms +0/-0; system operations and declaration records +0/-0 (203 remain); exception clauses +0/-0. This candidate begins the proof-carrying safety replacement without adding a proof surface yet. [ENT-3.S5] now gives a successful direct-place [SET-1] value commit the same finite literal, term-copy, and total-conversion image as an ordinary binding, strictly after the old target value has been killed. [ENT-5] now closes a reaching state before lexical support is forgotten, so a conclusion whose own support is entirely outside the exiting scope survives even when an exiting local was an intermediate proof term; any conclusion still naming the local or one of its holders dies normally. Both changes only add machine-derived facts, add no search, runtime operation, check, trap, branch, or optimizer authority, and cannot turn a formerly discharged obligation into an undischarged one. No grammar production, keyword, operation, type, outcome, diagnostic family, or writer-visible surface is added or removed.
Selection ground for the [CLM-1] narrowing: evidence-selected, by the differential-fuzz campaign recorded in `docs/done/0097-differential-fuzz.md`. Every one of that campaign's 63 rejections over 2004 generated programs was this shape, and the minimized pair differs by one line: with an early `return` in the `Err` arm of a `match` on a system-call result, a following `let seed = 3209_u64; let offset = seed % 64_u64;` was post-join state selected by the `Err` edge and `claim guard: ilt(offset, 64_u64)` was refused as non-local with carrier `offset`, although the claim's truth reads nothing the system returned; with an empty `Err` arm the identical claim was admitted. The refused member is now accepted. The rejected wider alternative was to drop control dependence from claim authority entirely, which would admit a claim over a binder a boundary selector chose between two arms; the narrowing keeps that selection and removes only the part that selects nothing. Prior selection ground for [PAR-3]: the staged-pipeline design derived against the completion model and the io-completion benchmark's own measurements; first-principles derivation recorded in `research/investigations/io-model/FIRST-PRINCIPLES.md`, followed by the implementation audit in `research/investigations/io-model/IMPLEMENTATION-AUDIT.md`.

Rule IDs are stable; diagnostics cite rule IDs. Sections marked DEFERRED record obligations with spec deltas per META-5, not normative content.

R3-PROVISIONAL REGISTER (constitution audit 2026-07-05; these forms were minimality-selected, not evidence-selected, and require validation before ratification; their derivation status and open evidence are recorded in `spec/derivation/derivation-ledger.md` and relevant live `mcts_mem/` decisions): ordinary loop form (GRAM-4/6; the counted `for_stmt` is evidence-selected in v0.25 and is not this register item), statement-only match (GRAM-7), boundary annotation surface (TYPE-5), no-shadowing (TYPE-6), env-struct closures replacement (FN-5), contracts/conform as interfaces replacement (FN-3 — round-2 verdict still needs_evidence), byte-format choices and reject-vs-canonicalize (FORM-1/2), no-comments (FORM-4), decimal-only literals (FORM-5), checker completeness levers (OWN-3/8/11 — rejection-rate unmeasured), and deref prefix places (GRAM-5).

## 1. Scope and conformance

[SCOPE-1] This document defines the writer-facing kernel plus the writer-visible stubs of the gated family (§14).
The gated family's members (unsafe regions, FFI extern frames, trusted primitive imports) are not writable by the steady-state writer; a kernel program contains no gated constructs.

[SCOPE-2] A program is checker-accepted iff it parses under the canonical grammar and satisfies every machine judgment in this document.
An owner-approved program additionally has every retained claim's review record validated under [CLM-1]; that approval status is an external review judgment over the exact checker-accepted source and claim inventory, not another compiler fact source or a way to admit checker-rejected source.
Every proof-required hazardous operation is statically discharged by the deterministic checker before lowering; a writer may establish a missing fact with executed control flow or with one retained named [CLM-1]-local claim, but no operation receives an implicit runtime fallback.
There is no writer-emittable unchecked state: nothing writer-stated is trusted without either machine derivation or the executed claim boundary.
The sole trusted-assertion class is toolchain-gated ledger entries (§14), which the writer cannot author or edit.

[SCOPE-3] Accepted programs have no undefined behavior, conditional on: (a) the declared trusted computing base (compiler, checker, runtime, allocator, OS), and (b) when a program links gated FFI frames, ABI-well-behaved foreign code.
This is the Layer-4 envelope statement; violations of (a)/(b) are outside the language's guarantee.

[SCOPE-4] A false executed `claim` is the sole writer-reachable language runtime contract violation and traps [CLM-1].
Before aborting, the runtime attempts to write the exact [DIAG-3] trap record for that claim to standard error.
If control returns from that attempt, it then aborts without unwinding and without performing language cleanup after the violation.
Failure of the external output sink may terminate the process before that explicit abort, but it never permits execution to continue.

## 2. Canonical form

[FORM-1] There is exactly one spelling per semantic construct and one legal byte-level formatting.
Non-canonical input is a hard error; the toolchain never auto-formats.
Unknown constructs are hard errors (conservative extension).

[FORM-2] Each source file is UTF-8.
Once every source has passed raw lexical formation and the complete compilation unit has one derivation, each source owns one ordered derivation forest: exactly the top-level `item` subtrees under the single compilation-unit `program` root whose terminals belong to that source, in source-local item order.
A source forest is not a second `program` node, and a source with no items owns an empty forest.
That source's canonical bytes are exactly the result of rendering its forest by the following rules.
The input bytes must equal that rendering byte for byte; the toolchain does not normalize or rewrite input.
A source that has no complete `item*` derivation is rejected by its owning lexical or grammar rule before this forest-format comparison, and no tree or forest is fabricated [DIAG-1].

Outside terminal interiors, lines end only with LF and formatting bytes are only ASCII space and LF.
There is no CR, tab, trailing horizontal whitespace, leading blank line, or blank line inside a top-level item.
A nonempty source has exactly one empty line between consecutive top-level `item` nodes and no trailing blank line; its final nonempty line ends with exactly one LF.
A source containing zero items is exactly one LF.
Terminal interiors retain their exact bytes and are checked by their owning FORM rule.

The left-attachment set contains `(`, `[`, `<`, `&`, `.`, and `..`.
The right-attachment set contains `)`, `]`, `>`, `,`, `;`, `.`, `:`, `(`, `<`, `[`, and `..`.
Between two consecutive terminals on the same line, emit zero bytes when the left terminal is in the left-attachment set or the right terminal is in the right-attachment set; otherwise emit exactly one ASCII space.
Thus function headers are `fn f()`, `fn f<T>()`, and `fn f['r]()`; subscripts are `p[i]`; a counted range is `lower..upper`; generic and square-bracket interiors are compact; `](` and `>(` are attached; and commas and colons attach to their left operand and have one space before the grammar-required following element.
Examples include `Result<i32, Overflow>`, `f(x: a, y: b)`, `conform i32: Zeroed`, `['r, 's]`, and `[10_u8, 20_u8]`.

Every nonempty physical line begins with exactly two ASCII spaces for each enclosing brace block.
A closing brace is rendered after reducing the depth for the block it closes.
A match-arm header is therefore one level inside its match, and statements in the arm body are two levels inside it.

The line-bearing simple productions are `field`, `variant`, `fn_sig`, `law`, `fn_bind`, `const_decl`, `doc`, `contract_define`, `requires_clause`, `ensures_clause`, `set_stmt`, `expr_stmt`, `return_stmt`, `break_stmt`, `claim_stmt`, and `give_stmt`, plus a `let_stmt` whose selected right-hand side is `ordinary_let_rhs`, `propagate_let_rhs`, or `replace_let_rhs`.
Each renders completely on one line, including its final semicolon.

The block-bearing productions are `struct_decl`, `enum_decl`, `contract_decl`, `conform_decl`, the body of `fn_decl`, `contract_block`, `loop_stmt`, `for_stmt`, `region_stmt`, `match_stmt`, `value_match`, `if_stmt`, `value_if`, and `arm`.
Their introducer through `{` is one line; their children render on following lines at depth plus one; and `}` renders on its own line at the original depth.
Empty blocks still use an opening line followed by a closing-brace line.
An `if_stmt` or `value_if` is rendered solely by this sentence, the generic block-bearing rendering notwithstanding: its introducer through the then-block `{` is one line; then-children render at depth plus one; an `else` renders as the join line `} else {` at the original depth, and a chained `else if` as the join line `} else if` through that `if`'s `{` at the original depth, never as a nested introducer line; else-children render at depth plus one; and the final `}` renders on its own line at the original depth.
No one-line `if` form exists.
A value-match or value-if let places its complete let prefix and the `match` or `if` introducer through `{` on one line.

A function without a `contract_block` puts its complete header through the body `{` on one line.
A function with a `contract_block` puts its header through `contract {` on one line.
After that block, render its close and the body open as the single line `} {`.
Then render the body children and closing brace.
Every production not listed as line-bearing or block-bearing introduces no formatting boundary of its own.
Its terminals stay on the current line unless a descendant line-bearing or block-bearing production introduces one of the boundaries prescribed above.
No other LF or blank line is emitted.

[FORM-3] Lexical classes: IDENT `[a-z][a-z0-9_]*` excluding every lowercase token spelling produced by exact fixed grammar atoms in the complete grammar and the retired spelling `trap`; TYPEID `[A-Z][A-Za-z0-9]*`; REGIONID `'[a-z][a-z0-9_]*` (apostrophe-prefixed, the only region spelling); LABEL `@[a-z][a-z0-9_]*`; OPNAME `[a-z][a-z0-9_]*\.(wrap|defined|checked|sat|strict)` (single token; the base has the raw lowercase-word shape used by IDENT and the mode suffix is a closed word set, so an OPNAME can never maximal-munch a valid field-access place `p.field`: all five suffix words are reserved from field binding [OP-1, GRAM-5]; e.g. `ineg.checked`).

[FORM-4] There are no comments.
Documentation is the `doc` field of declarations [GRAM-2].
Provenance lives in toolchain records.

[FORM-5] Literals, exhaustively: integers `-?[0-9]+_TYPE` (decimal only, mandatory suffix; a leading `-` is legal for signed TYPE, and the signed value must lie in TYPE's range [FORM-7]; e.g. `42_i32`, `-2147483648_i32`); finite floats use the grammar `-?(0|[1-9][0-9]*)\.[0-9]+(e-?(0|[1-9][0-9]*))?_TYPE`, where TYPE is `f32` (IEEE 754 binary32) or `f64` (IEEE 754 binary64), positive exponents carry no sign, negative exponents carry one `-`, and only the integer and exponent components have the stated no-leading-zero form.
Let C be the nonnegative integer formed by concatenating the integer and fraction digits, let F be the number of fraction digits, and let E be the signed integer formed by the exponent digits and their optional `-`; when the exponent is absent E is zero, and `e-0` also gives E zero.
A matching decimal whose C is zero denotes signed decimal zero: a leading literal `-` selects negative zero and its absence selects positive zero, independently of E.
Every other matching decimal denotes the exact nonzero rational whose magnitude is C × 10^(E − F), with the leading literal sign applied.
For one finite bit pattern of TYPE, consider every matching decimal that rounds from that signed zero or exact nonzero rational to the bit pattern under IEEE 754 round-to-nearest, ties-to-even.
Its canonical spelling is the candidate with the fewest ASCII bytes before `_TYPE`; a tie is resolved by lexicographically least unsigned ASCII bytes.
This selection is total, host-independent, and unique; in particular `0.0` and `-0.0` remain distinct.
Other examples are `1.5_f64` and `6.022e23_f64`.
`unit`; STRING `"..."` whose interior is a sequence of items, each one raw ASCII-printable byte in U+0020..U+007E other than `"` and `\`, or one of exactly three escapes `\\ \" \n`; no other byte is legal, and each character has exactly one spelling (the escape where one is defined, the raw byte otherwise).
STRING appears only in `doc` entries and `claim` justifications; non-ASCII diagnostic text is DEFERRED.
There are no boolean literals: `Bool` is a prelude enum (§15).
Generic-numeric literals `0_T` and `1_T` are legal where `T` is a gparam bound by a numeric contract (`Int` or `Float`, §15), denoting T's additive and multiplicative identity; a concrete type uses `0_i32` and the like, so there is no dual spelling.
NaN and the infinities are not literals; they are the nullary ops `fnan` and `finf` [OP-1].

[FORM-6] The token `unit` names the unit type in type position and the unit value in expression position; the grammar positions are disjoint productions, so resolution is production-local, not contextual.
The lowercase spelling follows the primitive-type convention (TYPE-1: primitives are lowercase keywords, not TYPEIDs); the single-token value spelling is the R3 one-spelling choice for the type's sole inhabitant.

[FORM-7] Numeric-literal well-formedness (R4 check-reject).
An integer literal `-?d_T` is legal where its signed value lies in the closed range of T (signed `[-2^(K-1), 2^(K-1)-1]`, unsigned `[0, 2^K-1]`) and it has no leading zeros: the single digit `0` is its own form, a leading `-` is legal for signed T, and `-0` is written `0`.
A float literal is legal only when it has the unique canonical spelling selected by [FORM-5] and denotes a finite value of its stated TYPE.
An out-of-range integer, a leading-zero integer, a noncanonical float spelling, or a float decimal that rounds to a non-finite value is a hard error at check time [SCOPE-2]; a literal never denotes a wrapped, truncated, saturated, or undefined value.

[LEX-1] Lexicon policy: surface names label checked invariants, stated in this document self-containedly.
Names are never borrowed from backend IR vocabulary (e.g. `noalias`), which names lowering consequences, not source invariants; and a name is borrowed from another language's convention only where a divergence census shows the semantics genuinely match.
Ruling of record: the exclusive borrow mode is `uniq` (uniqueness-type lineage), not `mut` (Rust divergence: exclusivity is the invariant; mutation is only its permission, and the name breaks under a future explicitly bounded interior-mutation form).
DEFERRED with recorded delta: the two-axis mode vocabulary (exclusivity x write-permission, adding frozen/exclusive-read and an explicitly bounded shared-write form).

## 3. Grammar

[GRAM-1] The grammar is deterministic and unambiguous.
Raw lexical formation scans each source independently from byte offset zero and partitions it into tokens and trivia without normalization, decoding a value, or consulting grammar position, name lookup, the operation table, or another source.
At each cursor it takes exactly the following maximal form; no token or trivia crosses a source boundary.

- One or more ASCII space bytes form one trivia item.
One LF byte forms one trivia item.
- A lower word starts with `[a-z]` and continues through the maximal `[a-z0-9_]*` suffix.
If that complete base is followed immediately by `.` and exactly one of `wrap`, `defined`, `checked`, `sat`, or `strict`, and the suffix is not followed by an ASCII letter, ASCII digit, or `_`, the base, dot, and suffix instead form one operation-name token.
Otherwise the lower word ends before the dot.
- An upper word starts with `[A-Z]` and continues through the maximal `[A-Za-z0-9]*` suffix.
- A region form starts with `'` and a label form starts with `@`; the sigil must be followed by `[a-z]`, after which the token continues through the maximal `[a-z0-9_]*` suffix.
- A numeric form starts with a decimal digit, or with `-` immediately followed by a decimal digit.
It then consumes the maximal sequence of ASCII letters, ASCII digits, `_`, and `.`, plus a `+` or `-` only when that sign byte immediately follows `e` or `E`, except that when the next two bytes are `..` the numeric form ends immediately before the first dot.
A single dot and every other numeric candidate retain the preceding maximal rule unchanged.
Raw formation deliberately retains broad candidates such as `1e+`, `1.00_f64`, and `1.0E2_f64`; [FORM-5] and [FORM-7] decide membership and canonicality without rescanning or splitting them.
- An operator form starts with `+`, `*`, `/`, or `%`, or with a `-` that is immediately followed by neither a decimal digit (numeric form, unchanged) nor `>` (the `->` compound, unchanged), and continues through the maximal `[a-z]*` suffix; the suffix must be empty or one of `wrap`, `defined`, `checked`, `sat` per the closed `infix_op` list, and any other suffix is a terminal-membership rejection.
- A STRING form starts with `"` and ends at the first unescaped `"`.
Its interior consists only of raw bytes `0x20` through `0x7e` other than `"` and `\`, or the two-byte escapes `\\`, `\"`, and `\n`.
An escape consumes its backslash and follower together.
- `->`, `=>`, and `..` are the three compound punctuation tokens.
Otherwise each byte in `(`, `)`, `{`, `}`, `[`, `]`, `<`, `>`, `,`, `:`, `;`, `.`, `=`, and `&` is one exact punctuation token.

In source EBNF, each quoted fixed atom denotes the unique sequence of raw formed tokens whose concatenated bytes equal that atom.
In particular, `"&uniq"` expands to the punctuation token `&` followed by the fixed lower-word token `uniq`, while `"->"`, `"=>"`, and `".."` each denote one compound punctuation token.
The quoted `"[0-9]+"` atom in the `const` production is the sole pattern atom: it denotes one numeric-form token whose complete bytes match `[0-9]+`, and it is not a fixed atom.
`SELECT_2` and the two-token parser bound count the expanded raw formed tokens, not quoted-atom occurrences.
An external terminal denotes one predicate over one formed token.

Anything that cannot take one of those forms is a raw lexical defect with the attribution and exact span in [DIAG-1].
Raw formation gives every token exactly one context-free shape kind: lower word, upper word, region form, label form, operation-name form, operator form, numeric form, STRING form, or one exact punctuation form.
Terminal membership then visits every formed token in source-ordinal and token order.
For each token independently, and without consulting grammar position, name lookup, the operation table, or another token, it evaluates the complete approved set of exact fixed-terminal predicates and external-terminal predicates in this specification and retains every matching predicate.
It rejects the token exactly when that retained set is empty; it never selects one preferred predicate and never tests only the predicates expected at a parser position.
Grammar derivation later tests the retained predicate sets against its `SELECT_2` rows.

A grammar terminal is therefore a predicate over a token's shape kind and exact bytes, not a priority-selected replacement token kind.
Exact-spelling and union predicates may overlap only when they do not compete at one grammar decision; every choice, optional, and repetition decision has pairwise-disjoint strong-LL(2) `SELECT_2` languages, so a parser selects exactly one arm with at most two tokens.
In particular, a noncompeting overlap such as fixed `unit` with the `literal` union does not create an ambiguous parse, but no decision may use predicate priority to hide an overlap.
Every production maps 1:1 to one core-tree node kind; there is no desugaring.
`infix_tail` maps to the `infix` node kind: a selected tail forms one `infix` node spanning the complete `expr` — the atom and the tail — so the 1:1 production-to-node mapping is preserved by the factored recognition.

[GRAM-2] Items:

```wf-ebnf GRAM-2
program      := item*
item         := fn_decl | struct_decl | enum_decl | contract_decl | conform_decl | const_decl
struct_decl  := "struct" TYPEID generics? "{" doc? field* "}"
field        := IDENT ":" type ";"
enum_decl    := "enum" TYPEID generics? "{" doc? variant* "}"
variant      := TYPEID "(" vfield_list? ")" ";"
vfield_list  := vfield ("," vfield)*
vfield       := IDENT ":" type
fn_decl      := "deny_claims"? program_kind? "fn" IDENT generics? region_params? "(" param_list? ")"
                "->" result_binding effects contract_block? "{" doc? stmt* "}"
program_kind := "command"
result_binding:= IDENT ":" rtype
contract_block:= "contract" "{" contract_define* requires_clause* ensures_clause* "}"
contract_define:= "define" IDENT "=" expr ";"
requires_clause:= "requires" expr ";"
ensures_clause:= "ensures" ("when" result_route ":")? expr ";"
result_route:= TYPEID "(" fieldbind ")"
contract_decl:= "contract" TYPEID generics? "{" doc? fn_sig* law* "}"
fn_sig       := "fn" IDENT region_params? "(" param_list? ")" "->" result_binding effects ";"
law          := "law" IDENT "(" (law_arg ("," law_arg)*)? ")" ";"
law_arg      := IDENT | literal
conform_decl := "conform" type ":" TYPEID targs? "{" doc? fn_bind* "}"
const_decl   := "const" IDENT ":" type "=" cvalue ";"
fn_bind      := IDENT "=" IDENT ";"
doc          := "doc" STRING ";"
generics     := "<" gparam ("," gparam)* ">"
gparam       := TYPEID (":" TYPEID)? | "const" IDENT ":" type
region_params:= "[" REGIONID ("," REGIONID)* "]"
param_list   := param ("," param)*
param        := input_label? IDENT ":" mode type
input_label  := "command" "." IDENT "as"
```

[GRAM-3] Types and modes:

```wf-ebnf GRAM-3
type   := "i8"|"i16"|"i32"|"i64"|"u8"|"u16"|"u32"|"u64"|"f32"|"f64"|"unit"
        | TYPEID targs? | "array" "<" type "," const ">"
        | "slice" "<" REGIONID "," type ">" | "box" "<" type ">"
        | "arena" "<" REGIONID "," type ">" | "buffer" "<" type ">"
rtype  := mode type
mode   := "own" | "&" REGIONID | "&uniq" REGIONID
targs  := "<" targ ("," targ)* ">"
targ   := type | REGIONID | const
```

[GRAM-4] Statements:

```wf-ebnf GRAM-4
stmt        := let_stmt | set_stmt | expr_stmt | return_stmt | loop_stmt
             | for_stmt | break_stmt | region_stmt | claim_stmt
             | if_stmt | match_stmt | give_stmt
let_stmt    := "let" IDENT "="
               ( ordinary_let_rhs | propagate_let_rhs | replace_let_rhs
               | value_match | value_if )
if_stmt     := "if" expr "{" stmt* "}" ("else" (if_stmt | "{" stmt* "}"))?
value_if    := "if" expr "{" stmt* "}" "else" (value_if | "{" stmt* "}")
ordinary_let_rhs:= expr ";"
propagate_let_rhs := "propagate" expr ";"
replace_let_rhs := "replace" place "=" expr ";"
set_stmt    := "set" place "=" expr ";"
expr_stmt   := call ";"
return_stmt := "return" expr ";"
loop_stmt   := "loop" LABEL "{" stmt* "}"
for_stmt    := "for" LABEL IDENT "in" atom ".." atom "{" stmt* "}"
break_stmt  := "break" LABEL ";"
region_stmt := "region" REGIONID "{" stmt* "}"
claim_stmt  := "claim" IDENT ":" expr "because" STRING ";"
give_stmt   := "give" expr ";"
match_stmt  := "match" expr "{" arm+ "}"
value_match := "match" expr "{" arm+ "}"
arm            := TYPEID "(" fieldbind_list? ")" "=>" "{" stmt* "}"
fieldbind_list := fieldbind ("," fieldbind)*
fieldbind      := IDENT ":" IDENT
```

[GRAM-5] Expressions and places:

```wf-ebnf GRAM-5
expr           := atom infix_tail? | call | construct
infix_tail     := infix_op atom
infix_op       := "+" | "+wrap" | "+defined" | "+checked" | "+sat"
                | "-" | "-wrap" | "-defined" | "-checked" | "-sat"
                | "*" | "*wrap" | "*defined" | "*checked" | "*sat"
                | "/" | "/defined" | "/checked"
                | "%" | "%defined" | "%checked"
atom           := literal | "move" place | place | borrow_expr
call           := callee targs? "(" ( atom_list | fieldinit_list )? ")"
callee         := IDENT | OPNAME
construct      := TYPEID targs? "(" fieldinit_list? ")"
fieldinit_list := fieldinit ("," fieldinit)*
fieldinit      := IDENT ":" atom
borrow_expr    := "&" REGIONID place | "&uniq" REGIONID place
atom_list      := atom ("," atom)*
place          := pbase psuffix*
pbase          := IDENT | "deref" "(" place ")"
psuffix        := "." IDENT | "[" atom "]"
```

[GRAM-6] There is no general operator syntax and no precedence: an `infix` expression is exactly one operation over two atoms [GRAM-5, GRAM-9], composition is by `let`, and no precedence, associativity, or parenthesization surface exists.
There is no `while`.
Conditional control is type-driven with one form per class: a Bool condition takes `if`/`else`, an enum scrutinee takes `match`, and each is the sole legal form for its class — a `match` whose scrutinee has type `Bool` is a hard error citing GRAM-6 at the scrutinee `expr` node (spell `if`).
An `if` condition must have exact value mode and type `own Bool` under exactly the [OP-5] condition judgment, TYPE-7 exclusivity included; every other condition failure cites GRAM-6 at the condition `expr` node.
An `if_stmt` `else` whose block is empty is a hard error citing GRAM-6 at that `if_stmt` node (spell the else-free `if`; a `value_if`'s undelivering else is [GIVE-1]'s rejection, not this one).
An `else` whose block contains exactly one `if_stmt` and nothing else is a hard error citing GRAM-6 at that nested `if_stmt` node (spell `else if`); in a `value_if` whose else block is exactly one else-free `if_stmt`, the branch cannot deliver, [GIVE-1] owns the rejection, and GRAM-6 forms no candidate there, so the flattening fix is never demanded where the chain form could not be spelled.
A conditional value is a `let`-initializer `match` or `if` [GRAM-7, GIVE-1].
The only iteration forms are the ordinary `loop` plus `break`, and the counted ascending half-open `for` form whose complete semantics are [TYPE-5, TYPE-6, OWN-11, FN-1, ENT-2, ENT-3, ENT-5]; there is no step, reverse, iterator, or `continue` form.
The subscript suffix is a place form (its sole home); bounds semantics are [OP-4].

[GRAM-7] `match` and `if` each have one source body shape and two distinct core-tree node kinds: `match_stmt`/`if_stmt` for statements, `value_match`/`value_if` for a `let` initializer.
The pairs never compete at one grammar decision: the statement forms begin at the statement boundary, the value forms only after the complete `let IDENT =` prefix, so the parser decides from source position alone, without type, name-resolution, or checker context.
A value form is value-producing, and every arm or branch must satisfy the complete [GIVE-1] delivery judgment for its binding; a `value_if`'s `else` is grammatically mandatory [GRAM-4] because a missing branch could not deliver.
Statement forms produce no value; their bodies act by effect.
`return`-position conditionals deliver by returning from branches; there is no helper-function conditional-initialization idiom, and value-production is confined to the `let` initializer, so neither construct ever occupies an arbitrary expression position.

[GIVE-1] `give e;` delivers `e` as the value of the nearest enclosing value initializer — a `value_match` or `value_if`.
An else-position `value_if` of a chain is part of the chain, not a nested initializer: its `give`s deliver to the chain's binding.
A value initializer bound by its own inner `let` delivers only to that inner binding and never makes an outer arm or branch deliver.
`give` is legal only inside a value initializer's arm or branch — a checker-scoped restriction exactly as `break`'s enclosing-loop rule [TYPE-6]: the grammar admits `give_stmt` and the checker restricts it, which is META-2-clean by the `break` precedent.
The binding's mode and type are derived from the delivery set [TYPE-5]: every delivering `give` of one value initializer must have one identical exact mode and type, and that is the binding's derived mode and type; a delivering `give` whose exact mode or type differs from an earlier delivering `give` of the same initializer is a hard error citing GIVE-1 at the later `give_stmt` node — derivation is agreement over the closed delivery set, never a join, widening, or common-supertype rule.
A value initializer whose delivery set is empty — every arm or branch leaves by `return` or by `break` to an enclosing loop — is a hard error citing GIVE-1 at the `let_stmt` node; the mechanical fix is the statement form (`match_stmt` or `if_stmt`) with the binding dropped.
On every control path an arm or branch terminates in exactly one `give e;` or cannot reach the initializer's continuation; a give-free continuing path, a statement following a `give` in the same block, and a second `give` on one path are each a hard error citing GIVE-1 — the value analog of match exhaustiveness [ERR-2].
Give-completeness is a structural last-statement recursion: an arm or branch delivers when its final statement is a `give_stmt`, a `return_stmt`, a `break_stmt` whose resolved target loop lexically encloses the same value initializer, a `match_stmt` every arm of which delivers, or an `if_stmt` with `else` both branches of which deliver, relative to that same value initializer; an else-free `if_stmt` has a continuing false edge and never delivers.
A final nested value initializer bound by its own `let` delivers only to its own inner let and therefore does not make the outer arm or branch deliver.
A `claim` or call that may trap also has a normally continuing edge and does not count as delivery or must-divergence.
No `loop_stmt` or `for_stmt` is assumed to diverge.
This recursion is strictly simpler than the ownership checker.
`give e;` moves or copies `e` per [OWN-1]; a borrow-typed `e` is judged for regions exactly as a returned borrow of the same mode [OWN-4].
Only when the enclosing initializer is a `value_if`, its derived delivery mode is `own`, and its type is one [ENT-2] fragment integer may a direct non-consuming bare-atom `give` additionally participate in [ENT-5]'s bounded relation delivery.
The same spelling inside `value_match` carries no relation.
This adds no typing premise and never makes a move, borrow, call, construction, subscript, projection, or computed expression into a fact carrier.
GIVE-1 still owns delivery completeness and exact mode/type agreement; only after those judgments succeed may ENT-5 substitute the atom's already evaluated value into the receiving binding.

For that additional fact-carrier judgment, the direct atom must be one bare tracked own-value binding of the exact receiving type: its root resolves to a body `let_stmt` binding, `for_stmt` binder, parameter, or match binder, and it carries no suffix.
A literal, named const, const-generic constant, Z, counted capture, contract definition, symbolic result datum, projected place, consuming atom, or any other atom may still be admitted in its own grammar role but carries no relation through a value initializer.
Replace every occurrence of the delivered binding d with the receiver x (`d ↦ x`); no receiver fact is read and no inverse substitution is formed.

[GRAM-8] Named construction.
A `construct` of struct or enum-variant type K writes every declared field of K exactly once as `IDENT ":" atom`, the IDENTs equal to K's declared field names in declared order.
A missing, extra, repeated, misspelled, or out-of-order field name is a hard error citing GRAM-8 and K's declared field list.
There is no positional construction form; a nullary K is written `K()`.
Field names are redundant-explicit facts (the TYPE-5 class): checked, never chosen, never a reordering option (declared order is the one legal byte sequence).
The name-only-when-two-same-typed-fields alternative is a context-dependent spelling and is rejected [META-2].

[GRAM-9] Flat (three-address) computation.
Every call argument, construct field value, infix operand, subscript offset, and lower or upper endpoint of a `for_stmt` is an `atom` [GRAM-5]; a `call` or `construct` in an atom position does not derive under the grammar and is a hard error citing GRAM-9.
A computed value is forwarded to another operation only by binding it with a preceding `let` (whose mode and type are derived [TYPE-5]) and referencing the binding.
Nesting and let-splitting are not two spellings of one computation; there is no expression-nesting alternative [FORM-1].
`borrow_expr` is an `atom`, so borrows passed as arguments need no binding and OWN-6 is untouched.

[GRAM-10] Named match binders.
An `arm` for variant K writes every declared field of K exactly once as `IDENT ":" IDENT` (the declared field name, then a fresh binder), in declared order; a missing, extra, repeated, misspelled, or out-of-order field name is a hard error citing GRAM-10 and K's declared field list.
The binder is a fresh IDENT chosen by the writer and distinct from the field name, so TYPE-6 no-shadowing is never engaged by two arms binding fields of the same name.
Binder modes remain derived by OWN-13 (not written).
A nullary variant is written `K()`.

The `result_route` owns exactly one `fieldbind`, so zero-field and multi-field route shapes do not derive.
FN-9, not GRAM-10, owns that route after its leading TYPEID resolves: it admits exactly `Ok(value: IDENT)` for a concrete `Result<T, E>` whose T is one entailment-fragment integer type.
A misspelled field is therefore an FN-9 rejection at the `fieldbind`, as [DIAG-1] fixes; no match arm or runtime binder is formed.
Every other successfully resolved variant, payload type, nested projection, or route is outside the postcondition boundary and is rejected by FN-9 rather than generalized through this rule.

[GRAM-11] Named call arguments.
A `call` whose callee resolves to a user `fn` or to an admitted system operation [SYS-1] writes its arguments as `fieldinit_list` [GRAM-5] — each `IDENT ":" atom` equal to the callee's declared parameter names in declared order, fixed by [FN-1] for a user `fn` and by [SYS-2] for a system operation, the GRAM-8 discipline applied to calls.
A missing, extra, repeated, misspelled, or out-of-order parameter name is a hard error citing GRAM-11 and the callee's parameter list.
A `call` whose callee resolves to a table operation [OP-1] writes positional `atom_list` operands (operands are order-intrinsic and unnamed).
Argument reordering is not a spelling option: declared order is the one legal byte sequence [FORM-1], so parameter names are redundant checked facts (R4 anti-transposition), never a reordering license.
Callee kind is resolved by name lookup [OP-1], the same partition that already selects the callee.

## 4. Types

[TYPE-1] Primitive types: `i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 unit`.
(`Bool` is a prelude enum, §15, not a primitive.)

[TYPE-2] Composite types: `struct`, `enum`, `array<T, N>` (N a constant-expression, [CONST-1]), `slice<'r, T>` (region-carrying view), `box<T>` (heap-owned unique), `arena<'r, T>` (region-bounded owned), `buffer<T>` (heap-owned, runtime-length, flat contiguous {data-pointer, u64 length} value; affine single-owner; length fixed at allocation, no in-place growth).
The opaque system types [SYS-2] are a distinct class: they are nominal, have no writer-visible component, and are constructed only by system operations and standard entry bindings. v0 `array` element type T must be copy (a primitive or tag-only enum, per the OWN-1 copy amendment).
A `buffer` element type T must be copy or a region-free [STOR-5] affine type; construction is gated per operation — `buffer_new` fills only copy elements, and `buffer_vacant` constructs `Option`-element buffers [OP-1, OP-9] — so an affine-element buffer type outside those constructors is well-formed but has no v0 construction route, exactly the formation/construction distinction this rule already draws for its element domains.
Affine elements leave and enter their slots only through [SET-2] element replacement and are read in place through borrowed `match` [OWN-13]; the element exchange never changes the buffer's length [ENT-5].

[TYPE-3] Nameability: every constructible type/mode/effect has a canonical, finite, writable name requiring no compiler execution.

[TYPE-4] There are no implicit conversions.
Representation change is the single explicit op `cvt<Src, Dst>(x)`.
Totality is decided by value-preservation, not bit-width: `cvt` returns `own Dst` where every value of Src is exactly representable in Dst, and `own Result<Dst, NarrowError>` for every other distinct numeric pair; it never rounds, truncates, or saturates.
The exact partition and per-value semantics are [OP-6].
Deliberate rounding is a separate DEFERRED float-round op family, never `cvt`.

[TYPE-5] Statement-local typing; boundary-explicit facts.
A `let` binder's mode and type are derived, never written: exactly the mode and type its selected right-hand side produces — an `ordinary_let_rhs` from its expression, which is always self-typed (operands are typed atoms, calls are typed by their [FN-1]/[OP-1]/[SYS-2] signatures, literals carry mandatory suffixes [FORM-5], constructions name their nominal and, when that nominal is generic, write its arguments); a `propagate_let_rhs` from the propagated Ok payload [ERR-3]; a `replace_let_rhs` at mode `own` from its target place's final selected type [SET-2]; a `value_match` or `value_if` from the derived common delivery type [GIVE-1], whose delivering `give`s are inside the same `let_stmt`, so the derivation stays statement-local.
This is unique reconstruction, not inference: no binder's type depends on a later statement, an expected type, or any use site, and no two derivations can disagree [FORM-1].
One form is excluded rather than reconstructed: a body `let` may not annotate a borrow with a region its right-hand side did not name, stating a destination the right-hand side satisfies by outlives [OWN-4] rather than equals, and a derived type is always the region the right-hand side itself produces.
Call sites state explicitly exactly what their callee class requires: type, region, and const arguments for user generics [FN-2]; region arguments for system operations [SYS-2]; and, for exactly the retained-argument table operations — `cvt` and `reinterpret` (type pairs [OP-6, OP-8]), `array_new` (element type and const length [CONST-1]), `arena_new` (region and element type), `buffer_fits` and `buffer_vacant` (element type [OP-1, OP-9]), and `finf`/`fnan` (result type) — the written arguments their rows fix, because no operand can supply them.
A `construct` of a generic nominal states that nominal's type and const arguments on the same ground and in every position, mandatorily: the source nominals under [FN-2], and the prelude generic nominals `Option<T>` and `Result<T, E>` through their variant constructors `None`, `Some`, `Ok`, and `Err`.
A nullary `None()` has no operand to supply anything, and construction never consults an expected nominal type [TYPE-6], so the written arguments are the only supply there is; their absence, or a count other than the named nominal's parameter list, is a hard error citing TYPE-5 at the complete `construct`.
The non-generic prelude nominals — `Bool`, `Overflow`, `DivError`, `NarrowError` — have no parameters and write nothing.
Every other table operation carries no written argument and derives its selected type from its operands [OP-2]; a written argument there is a hard error citing OP-1.
Argument types match declared parameter types exactly.
After [SET-1] derives a writable target place of type T, the right-hand side of `set p = e;` must produce exactly `own T`; there is no mode coercion, type conversion, or target-selected operation overload.
After the TYPE-7 implicit-read exclusivity below, a different right-hand-side mode or type is a hard error citing TYPE-5 at the complete `expr` child of the `set_stmt`, carrying expected `own T` and the actual mode and type.
After [SET-2] derives a writable affine target place of type T, the right-hand side of `let x = replace p = e;` receives this same exact-`own T` judgment, located at the complete `expr` child of the `replace_let_rhs`.
Redundant-explicit facts remain mandatory at every trust boundary — signatures with full modes, types, effect rows, and regions [FN-1], construction field names [GRAM-8], match binders [GRAM-10], call argument names [GRAM-11] — and are deleted exactly where reconstruction is unique and no transposition risk exists.

Every `fn_decl` and `fn_sig` has one mandatory `result_binding` whose written `rtype` fixes the callable result mode and type.
The result name is a proof-only boundary spelling: it denotes no runtime slot, does not enter callable signature equality, and is unavailable in a function body.
An unrouted [FN-9] postcondition may admit it as that clause's symbolic whole-result datum; a routed postcondition instead derives its payload binder's type from the admitted `Result.Ok` payload and makes the whole-result name unavailable in that clause.

A `contract_define` derives exactly the own copy mode and type of its right-hand-side expression.
It is an erased, declaration-before-use abbreviation rather than a statement, evaluation, snapshot, or storage allocation.
Its initializer must satisfy [FN-8]'s pure, total, non-consuming contract-expression judgment; every clause use is recursively alpha-expanded before a proof template is formed.

Each lower and upper endpoint atom of a `for_stmt` must produce exactly `own u64`; after [TYPE-7]'s implicit-read exclusivity, every other mode or type is a hard error citing TYPE-5 at that endpoint's `atom` node, with `SourceCoordinate` equal to its complete checked half-open source extent.
The counted binder has the fixed compiler-derived mode and type `own u64`; it carries no source annotation and does not infer from either endpoint.

[TYPE-6] Name resolution uses the following closed declaration domains.
The grammar role, never an inferred type or expected result, selects the domain and admissible declaration class.

| domain | declarations | admitted uses |
|---|---|---|
| lexical IDENT | top-level `fn_decl`; top-level `const_decl`; const `gparam`; `param`; `let_stmt`; `for_stmt` binder; arm `fieldbind` binders; `contract_define`; FN-9-owned result and route candidates; admitted system operations [SYS-1] | a `callee` IDENT admits a top-level function or an admitted system operation; a `fn_bind` right IDENT admits only a top-level function; `const` IDENT admits only an in-scope const generic or earlier named const; `cvalue` IDENT admits only an earlier named const; `pbase` admits only an in-scope runtime value binding, contract definition, admitted symbolic result datum, or named const |
| nominal-type TYPEID | source `struct_decl` and `enum_decl` names; PRE-1 nominal types; admitted system nominal types [SYS-1]; lexical type `gparam`s overlay this domain while live | `type` TYPEID and the TYPEID suffix of a FORM-5 generic numeric literal admit a live type generic where that form requires one, otherwise a nominal type |
| constructor TYPEID | each source struct constructor under its struct TYPEID; every source enum `variant`; PRE-1 variants, classified as struct-constructor or enum-variant; admitted system constructors [SYS-1], classified as struct-constructor or enum-variant | the leading TYPEID of `construct` admits either class; the leading TYPEID of `arm` or `result_route` admits only enum-variant |
| contract TYPEID | source `contract_decl` names and PRE-1 contract names, including `Int` and `Float` | the optional bound TYPEID of a type `gparam` and the contract TYPEID of `conform_decl` |
| REGIONID | `region_params` and `region_stmt` | every REGIONID in `type`, `mode`, `targ`, arena-allocation effects, and `borrow_expr` |
| LABEL | `loop_stmt`; `for_stmt` | `break_stmt` |

A source struct contributes one declaration event that adds one nominal-type entry and one constructor entry with the same spelling.
Those entries do not collide because the grammar distinguishes a `type` role from a `construct` or `arm` role.
An enum declaration adds only its nominal type; each variant adds its constructor.
Entries must be unique within, but not across, the nominal-type, constructor, and contract domains.
Constructor uniqueness is whole-unit and context-free, so construction and matching never consult an expected nominal type.

PRE-1 contributes exactly twenty-four declaration records in this preorder: each enum nominal, then its type parameters in list order, then each variant and that variant's fields in list order, followed by the contracts in declaration order.
They are six nominal enums, ten enum-variant constructors, three owner-local type parameters (`Option.T`, `Result.T`, and `Result.E`), three owner-table fields, and two contracts.
Exactly the six nominals, ten constructors, and two contracts enter the source resolver's whole-unit lookup inventory and are visible throughout the closed unit.
The three type parameters resolve only within their owning compiled PRE-1 declaration, the three fields enter only their owning variant table, and none of those six owner-local records is visible to source lookup.
PRE-1 records have no source event or source node.
Every top-level function signature is visible throughout the closed compilation unit after unit formation and before any semantic use is resolved [FN-1].
A source nominal type or contract becomes visible immediately after its declaring TYPEID terminal.
A source struct constructor becomes visible at that same terminal; an enum-variant constructor becomes visible immediately after its variant TYPEID terminal.
Each remains visible through the end of the unit.
Whole-unit inventory checks uniqueness but grants no earlier visibility; a use before one of these declaration points is rejected even though inventory knows the later declaration exists.

A generic TYPEID parameter becomes visible after its declaring terminal through the remainder of its declaration's generic, header, and body scope.
It may not redeclare another parameter in the same generic list or shadow a live nominal type or enclosing generic type.
Constructor and contract spellings are separate grammar-selected domains and do not participate in that comparison.
A const generic becomes visible after its complete `gparam`.
A region parameter becomes visible after its terminal through the remainder of its signature and body; for `fn_sig`, that scope ends at the signature terminator.
Independently of visibility, OWN-3 requires every REGIONID declaration to be unique throughout its owning function declaration or contract-member signature, parameters included: a later region parameter or local region may not reuse an earlier region spelling even after the earlier region's lexical scope has ended.
A `fn_decl` parameter becomes visible after its complete `param` through the function's optional `contract_block` and body.
A `fn_sig` parameter becomes visible after its complete `param` through that signature's terminator; duplicate parameters in that signature are same-scope redeclarations even though there is no lexical value-use role in the remaining suffix.
A `let_stmt` binder becomes visible only after its complete initializer statement through the end of its lexical block.
A `contract_define` binder becomes visible after its complete initializer through later definitions and every following requires or ensures clause of that one block, and nowhere in the body.
Its initializer may therefore use parameters, named consts, live type or const parameters, and earlier definitions, never itself or a later definition.
The IDENT of every `result_binding` is an FN-9-owned proof candidate rather than a runtime TYPE-6 value declaration.
It participates in FORM-3 reservation and must differ from every parameter and definition in its function; in a `fn_sig` it has no use scope.
In a `fn_decl`, FN-9 admits it as the one symbolic datum visible only in each unrouted ensures expression whose result shape is eligible.
The second IDENT of a `result_route` fieldbind is an FN-9-owned payload candidate.
After the route's leading constructor and field are admitted, that binder is visible only in the same `ensures_clause` expression and the header result binder is not visible there.
It must differ from its paired field, every parameter, the result binder, and every live definition.
Different ensures clauses have disjoint result-datum scopes and may reuse one route-binder spelling.
Neither kind of result datum has runtime storage or ownership state, and neither is visible in the function body.
A match binder becomes visible in its arm body only after the complete fieldbind list and only after GRAM-10 has established that it differs from its paired field label, every earlier binder in that arm list, and every lexical-IDENT declaration live on arm entry.
A `for_stmt` binder becomes visible only after its complete header, including both endpoint atoms, and only within that counted body.
An ordinary or counted loop label and a local region are visible only in their respective bodies; neither a counted label nor its binder is visible in either endpoint.
A named const becomes visible only after its complete `const_decl`, preserving CONST-2's explicitly-earlier rule.

Within one domain, two declarations in the compilation-unit root or in the same lexical scope are a redeclaration attributed to the later declaration event.
Declarations in unrelated function or declaration owners are not duplicates merely because their spellings match.
A nested lexical declaration may not shadow an entry live at that declaration.
OWN-3's function-wide REGIONID uniqueness is stricter than either rule and is reported at the later region declaration with the conflicting region origin.
GRAM-10 exclusively owns arm match-binder distinctness and freshness: a second IDENT of an arm `fieldbind` equal to its paired field label, an earlier binder in the same arm list, or any lexical-IDENT declaration live on arm entry is rejected citing GRAM-10 at that later/offending binder before it becomes a declaration, rather than also being reported as TYPE-6 shadowing.
FN-9 exclusively owns the analogous result-datum checks described above; failure creates no TYPE-6 declaration or duplicate event.
Because every top-level function is live throughout the unit, any other parameter, local, or const generic in a nested scope may not use a top-level function spelling even when that function's source item occurs later; the nested declaration is the offending shadow event.
Disjoint expired lexical scopes may reuse an ordinary value or label spelling; REGIONID reuse remains forbidden throughout one function by OWN-3.
Logical paths and record boundaries never create a namespace, scope, or lookup key [PROG-2].

The owner-dependent and table-checked declaration and use roles are exactly the carriers classified by [DIAG-1].
They do not enter or query a lexical name domain.
DIAG-1 retains each owner-dependent carrier for later typed owner/member checking and each table-checked carrier for later [FN-7] table checking.
Deferral is neither acceptance nor rejection of its later owner/member or table relation.

[TYPE-7] Reading through a reference is explicit.
`deref(place)` where place has type `&'r T`, `&uniq 'r T`, `box<T>`, or `arena<'r, T>` denotes a place of referent type T [GRAM-5]; a use of that place copies it when T is copy and requires `move` when T is affine [OWN-1].
A borrow-mode or box/arena binding used where a value of its referent type T is expected is a hard error citing TYPE-7, with the mechanical fix `deref(.)`.
A bare borrow holder is not rebound by `set`; SET-1 requires explicit `deref(holder)` to select its referent, and only a live usable `&uniq` holder can make that referent writable [OWN-5].
There is no implicit read-through-borrow [TYPE-4, META-2].

[SET-1] Copy-place assignment.
For `set p = e;`, target evaluation first resolves and evaluates the complete `p` without reading or consuming the value stored there.
A nested place is evaluated from its base outward; at each subscript, the base place is evaluated before its offset atom, and the subscript's [OP-4] discharge obligation is judged at that target place exactly as in read position, so accepted target evaluation executes no runtime check and cannot trap.
Field suffixes introduce no runtime evaluation.

The target's final selected type is T.
The target is writable exactly when it is rooted in a live own-mode value binding whose storage is frame-resident, box-owned, arena-owned, or buffer-owned [STOR-1], or reaches a referent through an explicit `deref` of a live usable `&uniq` holder.
Fields and indices inherit the writability of their selected base except that no writable target path may traverse a `slice<'r, U>` value.
A slice is an alias-bearing shared view created by `slice_of`; borrowing the slice descriptor uniquely does not grant unique access to the viewed storage.
Therefore a slice-rooted target is not writable whether the slice binding is own-mode or is reached through another holder.
A named const is never writable [CONST-2].
A `for_stmt` binder is compiler-updated state and is never source-writable; a target rooted there is a SET-1 rejection at the complete target `place`.
A shared-borrow referent, suspended `&uniq` holder, or place conflicting with another live loan is not writable [OWN-5].
A bare borrow holder selects the holder rather than its referent and is not writable; when the holder is live usable `&uniq`, the mechanical fix is `deref(.)`.
A dead root is never writable and is not revived [OWN-1].
These specific rules own their stated violations; every other failure of this closed writability relation cites SET-1 at the complete target `place` child of the `set_stmt`, carrying the resolved root class and the required writable classes.

T must be copy under [OWN-1].
Setting a place whose final selected type is affine remains a hard error under [STOR-1]; `set` does not mean take, replace, reinitialize, or implicit destruction.
The right-hand side is then checked under [TYPE-5] and evaluated under its ordinary expression, ownership, effect, and trap rules.
The checker analyzes the normal continuation of `e` and re-establishes there that the same target root is live and that the resolved target remains writable under the resulting loan state.
If the right-hand side moved the target root, the commit is a later write of a dead root under OWN-1.
If it created or changed a loan that conflicts with the commit, OWN-5 rejects the commit.
This is a static acceptance check: at runtime every target component is evaluated exactly once before `e`, and lowering carries the resulting target address and offset values across `e` rather than evaluating source again.
No root-liveness or writability fact from before the right-hand side bypasses the post-state check.

On successful revalidation, assignment performs exactly one write of the resulting copy value into `p`.
The previous copy value ceases to occupy `p` and requires no drop, release, finalizer, or cleanup edge [STOR-3].
The new value occupies the same place and the target root remains live.
If right-hand-side evaluation traps, no store occurs; preceding right-hand-side effects are not rolled back, and trap-abort behavior remains [EFF-4].
The checked program retains the exact target path, each required target check, the right-hand-side value, the post-right-hand-side liveness and writability judgments, and the single store before lowering [DIAG-2].

[SET-2] Affine-place replacement.
`let x = replace p = e;` atomically exchanges the affine value stored at a writable place with a same-typed replacement, binding the previous value as the fresh `own` binding x [TYPE-5].
Target formation, evaluation order, subscript discharge in target position, the closed writability relation, the loan-state judgment, and the post-right-hand-side revalidation are exactly [SET-1]'s, including its specific rule attributions; every other failure of the writability relation cites SET-2 at the complete target `place`.
The target's final selected type T must be affine under [OWN-1] and region-free under [STOR-5]'s relation.
A copy-typed target is a hard error citing SET-2 at the complete target `place`, carrying T and the restructuring `use set for a copy place; read the previous value bare`.
A region-bearing target type — `slice<'r, U>` or `arena<'r, U>` at any depth of T — is a hard error citing SET-2 at the complete target `place`, carrying T and the restructuring `a slice's static origin set and an arena's confinement are fixed at initialization; bind a new slice or arena under a new let`; region-bearing types cannot occur in stored content [STOR-5], so this judgment bites only a direct binding or dereference target.
The right-hand side must produce exactly `own T` under the [TYPE-5] judgment stated there.
On successful revalidation, the commit performs one read of the previous value into x's storage and one write of the replacement value into resolved(p), with no writer-observable program point between them: at every program point the place holds exactly one valid owner, and no temporary uninitialized hole, vacancy state, or move-from-target residue exists.
The commit is not a consuming use of the target root under [OWN-1]: the root binding remains live, no partial-move death occurs, and the moved-out value's sole owner is x, an ordinary `own T` binding thereafter with the ordinary [OWN-1] and [STOR-3] lifecycle.
Through a live usable `&uniq` holder the commit is the sole exception to [OWN-5]'s prohibition on moving content reached through a borrow: the exchange leaves the far-side owner owning exactly one valid T in that place at every program point, and exclusivity already excludes every other observer for the statement's duration.
A commit through a shared holder is never admitted, and a suspended holder is not usable [OWN-5].
Under [EFF-2]'s attribution the commit is one read and one write of the target's ultimate storage origin.
A successful commit derives no drop, release, finalizer, or cleanup edge [STOR-3]: nothing is destroyed, and the previous value's later release, if x is abandoned, is x's ordinary compiler-derived scope-exit action.
If right-hand-side evaluation traps, no commit occurs and x is never initialized; the place still holds the previous value, preceding right-hand-side effects are not rolled back, and trap-abort behavior remains [EFF-4].
The commit is an [ENT-5] kill event exactly as stated there; it establishes no fact.
The checked program retains the exact target path, each discharged target check, the right-hand-side value, the post-right-hand-side liveness and writability judgments, the read-out, the write-in, and the binding initialization before lowering [DIAG-2].

[CONST-1] The grammar production `const` of the fence below is usable at `array<T, N>` sizes and `const` targs.

```wf-ebnf CONST-1
const := ("[0-9]+" | IDENT) (infix_op ("[0-9]+" | IDENT))?
```

A decimal integer literal is bare and u64 by position; an IDENT names an in-scope integer-typed const-generic parameter [GRAM-2] or a top-level integer-typed named-const item [CONST-2].
A const-expression is at most one operation over two terms, exactly the shape [GRAM-6] fixes for expressions: composition is by a named const or a forwarded const parameter, and no precedence, associativity, or parenthesization surface exists.
The tail reuses `infix_op`, and its spelling must be one of the five bare operators `+`, `-`, `*`, `/`, `%`; a mode-suffixed spelling is a hard error citing CONST-1 at the `infix_op` node, because const evaluation has no runtime overflow mode — the grammar admits and the checker restricts, META-2-clean by the `break` precedent [GIVE-1].
Constant-expressions are evaluated at monomorphization [FN-2].
An IDENT resolving to a non-integer or array-typed const is a compile-time rejection [DIAG-1].
Const evaluation is exact in the unsigned 64-bit domain under the const-eval overflow policy named `const-reject`: an operation whose mathematical result lies outside that domain, or whose divisor is zero, is a compile-time rejection citing CONST-1 at the complete `const` node.
`const-reject` is disjoint from runtime proof-required exact arithmetic: it never creates an [ENT-6] operation obligation or admits a `.defined` spelling, an accepted const-expression executes no runtime check and cannot trap, and a const-expression never enters EFF-2's exhibits-traps relation.
Inside a generic template an unevaluated const-expression is symbolic; two symbolic const-expressions are identical exactly when their operation and ordered terms are identical, with no commutation, constant folding, or reassociation, exactly as [FN-8] fixes goal identity.
This keeps the const-generic forwarding path closed under the one operation: `const N` is usable as an `array<T, N>` size, and a derived expression such as `N * 2` is usable there and forwardable as a `const` targ, with each concrete instantiation evaluating it to one u64 value.

[CONST-2] A `const IDENT: type = cvalue;` item declares an immutable, program-lifetime, read-only static value, with the `cvalue` production of the fence below.

```wf-ebnf CONST-2
cvalue := literal | IDENT | "[" cvalue ("," cvalue)* "]" | TYPEID targs? "(" (IDENT ":" cvalue ("," IDENT ":" cvalue)*)? ")"
```

`type` must be const-eligible: a primitive [TYPE-1], `array<T, N>` of const-eligible T, or a source `struct` whose every field type is const-eligible; enums, `box`, `buffer`, `arena`, and `slice` are not const-eligible (a const is pure static rodata: no allocation, no region, no drop).
The `cvalue` totally defines the value (T1): a primitive-typed const takes a FORM-5 numeric or unit literal or an IDENT naming an earlier const of that exact type; an `array<T, N>`-typed const takes `[cvalue, ..., cvalue]` with exactly N entries, each of type T, and a struct-typed const takes the construction form `TYPEID(field: cvalue, ...)` naming its exact struct and writing every declared field in declared order [GRAM-8], each field value a cvalue of the declared field type.
The const-dependency graph is acyclic and declaration-before-use [TYPE-6]; evaluation is substitution and layout only.
A const item is never `move`d, `set`, or `&uniq`-borrowed.
It is read via subscript/`len` (copy-out for copy elements) or shared-borrowed `&'r p` in any region [OWN-10], so a const table may be `slice_of`-viewed and passed to a consumer.
A struct-typed const is additionally read via its field suffixes exactly as subscript reads: a copy-scalar selection copies out, and a composite selection keeps the whole-composite read rules.
A struct-typed const is laid out as one read-only static aggregate in the nominal's ordinary representation.
Enum-typed consts and written generic construction arguments in const position are DEFERRED with recorded delta: a payload-enum const has no non-consuming read path (a `match` scrutinee is an own place [OWN-13]), and a tag-only-enum const additionally needs a constant-value family no current program demands.

## 5. Ownership, regions, borrows (PROVISIONAL pending formal-calculus reconciliation)

[OWN-1] Every value has exactly one owner.
Values are classified copy or affine: primitives (TYPE-1), shared borrows, and tag-only enums (every variant nullary; `Bool` is the canonical case) copy on use; all other values (owned composites, `box`, `arena`, `slice` as `&uniq`, uniq borrows) are affine.
An affine place rooted in a live own-mode binding is consumed exactly once by an explicit `move p`, by use as an own-place match scrutinee under [OWN-13], or by use as the direct bare affine `Result<T, E>` place operand of `propagate` under [ERR-3].
Every other bare `place` expression of affine type is a hard error, and `move p` on a copy value is a hard error (copy values are used bare — one spelling per meaning, FORM-1).
The bare-affine mechanical fix is position-conditional: in a function body it is write `move p`, while in a `contract_block`, where [FN-8] rejects `move` itself, it is restate the definition or clause over copy operands or non-consuming admitted reads, so the repair never instructs a spelling FN-8 forbids.
Resolving and evaluating the target of SET-1 or SET-2 does not by itself read, copy, or move the selected value or its affine owner.
After any consuming use, the whole binding rooting `p` is dead (partial moves kill the whole binding); any later use, write, or `set` of a dead binding is an error at the later use or target place — reinitialization requires a new `let`.
The [SET-2] replace commit is not a consuming use: it exchanges the stored value, leaves the target root live, and initializes its fresh binding as the moved-out value's sole owner.
SET-1 and SET-2 recheck the live-root premise after their right-hand sides and never revive a dead binding.

[OWN-2] Modes: `own` (owned), `&'r` (shared borrow in region `'r`), `&uniq 'r` (exclusive borrow in region `'r`).
Modes are always written.

[OWN-3] Regions are lexical.
`region 'r { ... }` introduces `'r`; `region_params` introduce caller-supplied regions.
Region identifiers are unique within a function (parameters included).
Outlives-or-equals is the total reflexive relation: `'a` outlives-or-equals `'b` iff `'a = 'b`, or `'a`'s block strictly encloses `'b`'s block, or `'a` is caller-supplied and `'b` is local.
Distinct caller-supplied regions are incomparable: any rule requiring an order between them fails closed (reject).

[OWN-4] A borrow `&'a p` / `&uniq 'a p` is live exactly until the end of `'a`'s block (named-region liveness).
It may be stored into a destination of declared region `'b`, passed to a parameter of region `'b`, or returned as `rtype` region `'b`, only if `'a` outlives-or-equals `'b`.

[OWN-5] Resolved-place exclusivity.
While `&uniq 'a p` is live and its holder is not suspended [OWN-6]: no place overlapping resolved(`p`) may be read, written, moved, or borrowed, except reads/writes through that borrow's holder and except the creation of a statement-scoped child reborrow, an arm-scoped child reborrow, a candidate-position child reborrow, or a returned reborrow of that holder [OWN-6, OWN-13, OWN-14].
While a holder is suspended (a live statement-scoped child, arm-scoped child, candidate-position child, or returned reborrow of it exists), its own read/write allowance is withdrawn: no read, write, move, copy, `set` commit, or call-transfer through it is admitted until its last child ends; a `&uniq` holder suspended by candidate-position child creation does not resume — its claim may survive in the bound call result [OWN-6].
While any `&'a p` is live: no place overlapping resolved(`p`) may be written, moved, uniq-borrowed, or committed by `set`; reads are permitted.
A SET-1 commit is one write to resolved(target), and a [SET-2] commit is one read and one write to resolved(target); each is judged against the complete loan state after right-hand-side evaluation.
A commit through a live usable `&uniq` holder is a write through that holder; a commit through a shared holder is never admitted.
Content reached through any borrow may never be moved: `move` requires a place rooted at an own-mode binding, and the [SET-2] replace commit is the sole exception, sound because the exchange leaves no program point at which the referent place lacks exactly one valid owner.
Exclusivity invariant, checked unconditionally: no two live usable `&uniq` borrows have overlapping resolved places; a suspended holder is not usable, so the only overlapping pairs — a suspended parent with its statement-scoped child, arm-scoped child, candidate-position child, bound call-result holder, or returned reborrow — are never both-usable by construction.

Every `slice<'r, T>` value carries a finite set of possible ultimate storage origins.
While one function body is checked, an origin is one resolved source place, the distinguished `immutable-const` origin, or a formal-slice origin naming one of that function's parameters whose direct written type is `slice<'r, T>`.
Each incoming parameter of direct slice type starts with the singleton containing its own formal-slice origin, whether its written mode is `own`, `&'d`, or `&uniq 'd`; that term stands for the actual slice's complete set and is substituted only at a call boundary [FN-1, EFF-2].
Borrowing the descriptor and resolving a place through the descriptor holder preserve this set rather than replacing it with the descriptor binding's place.
`slice_of` creates a singleton: a named const maps to `immutable-const`, and every other admitted source retains its complete resolved place, including a place reached in arena content.
Binding, moving, passing, and returning a slice preserve the complete set.

This specification defines no slice-valued control-flow join.
A value initializer whose derived delivery type [GIVE-1, TYPE-5] is `slice<'r, T>` is a hard error citing OWN-5 at the complete `value_match` or `value_if`, with `SourceCoordinate` equal to that production's complete checked half-open source extent and the restructuring `use a match or if statement whose arms or branches return the slice directly, or call helpers with direct slice results`.
Alternative direct returns are checked independently and the caller uses their common signature ceiling [FN-1].

A slice access is judged as one shared access through every resolved-place origin in its set.
A write, move, or unique borrow of an ordinary place conflicts when that place overlaps any such origin.
`immutable-const` creates no conflicting access because named const storage is permanently read-only [CONST-2].
A formal-slice origin has no directly writable storage path inside its callee; overlap with the caller's other actual arguments is checked after substitution under [OWN-12].
No traversal order or chosen runtime arm may narrow the static set.
Each runtime slice still has exactly one actual storage origin, and that origin is always a member of the static set after complete call substitution.
Under this specification's named-region liveness, moving or returning a descriptor neither shortens nor extends the shared claim established by its source.

[OWN-6] Holder, resolution, and statement-scoped child reborrow.
The holder of a borrow is the binding its `borrow_expr` initializes (a borrow not bound by `let` is a call-scoped temporary, live until the end of the enclosing statement). resolved(place) rewrites a place rooted at a holder binding to the borrowed place plus the appended suffix, recursively.
All OWN-5/OWN-7 judgments use resolved places.
A statement-scoped child reborrow is the written form `&uniq 'c` or `&'c` over `deref(h)` followed by any written suffix chain, occurring as an argument atom of a `call` expression [GRAM-9], admitted only when: the receiving call's result mode is `own` or `unit`, never a borrow — except in the receiving call's provenance-candidate position, where a borrow result is admitted; `'c` is a locally-introduced region [OWN-3] whose block does not extend beyond the enclosing statement, and a caller-supplied region parameter is not admitted — except in the provenance-candidate position, where `'c` is any live region that resolved(`h`)'s region outlives-or-equals, caller-supplied included; the eligible holder `h` is a function parameter or a `let`-bound borrow, never a `match` binder; and a `uniq` child has a `uniq` parent, while a `shared` child is admitted from either [OWN-5]. resolved(child) = resolved(`h`) ++ suffix.
Creating a child suspends `h` for the enclosing statement [OWN-5]; while a holder is suspended by this statement-scoped creation, the sole operation admitted through a place overlapping resolved(`h`) is creating a further sibling child, siblings judged by OWN-7 with any overlapping pair containing a `uniq` child an error, and `h` resumes at the end of the statement after its last child ends.
Creating a candidate-position child through a `&uniq` holder suspends that holder for the remainder of its life; there is no statement-end resumption, because the child's claim may survive in the bound call result.
A shared holder needs no suspension: it admits no write through itself.
A child is never bound, returned, `give`n, stored, or the whole call result, and its `'c` cannot outlive the statement, so no borrow derived from a child outlives its statement; with borrow-free storage [STOR-5] the child is non-escaping.

A `let` whose ordinary right-hand side is a user call with borrow-mode result is a borrow holder rooted at the callee's provenance candidate [FN-1], and every accepted callee has one or has none.
resolved(result holder) = the candidate actual's complete resolved place, even when the callee delivered a narrower suffix of it; the holder's borrow is otherwise ordinary — OWN-4 liveness in the substituted result region, OWN-5 exclusivity, OWN-6 child admission, OWN-14 returned reborrow.
Nothing here narrows FN-1: the caller still judges the call by the signature alone.
A borrow-mode call result with no candidate is rooted in named `const` storage [FN-1, CONST-2], which no accepted write or unique borrow reaches [OWN-5, OWN-7]; its holder claims no caller place and conflicts with nothing.

Bound children, result-carrying children (reference-result provenance), `uniq`-to-`shared` downgrade, `match`-binder parents, and written grandchild chains through a bound direct reborrow are DEFERRED with recorded delta; every written reborrow form outside this argument-atom position is dispositioned by [OWN-14], and the derived match-payload binder is [OWN-13]'s arm-scoped child reborrow.

[OWN-7] Overlap: resolved `p` overlaps resolved `q` iff one is a prefix of the other.
Two subscripted places with the same resolved base overlap iff their offsets are not both literals with unequal values.
Two slice values in a fully substituted caller context overlap conservatively iff at least one pair of their resolved-place [OWN-5] origins overlaps.
`immutable-const` needs no overlap claim because no accepted write or unique borrow of const storage exists.
Formal-slice origins are substituted before caller overlap checking [FN-1, OWN-12]; they never establish that two actual sources are disjoint.

[OWN-8] Reject-when-unsure: the checker rejects any program it cannot prove conformant.
Rejection of a sound-but-unprovable program is not a defect; the diagnostic names the rule and a restructuring.

[OWN-9] Non-normative consequence for the optimizer: a live, usable `&uniq` borrow's resolved place is unaliased by any other usable access path (a suspended holder [OWN-6, OWN-13, OWN-14] is not usable; a statement-scoped child, arm-scoped child, candidate-position child, bound call-result holder, or returned reborrow and its suspended ancestor, though both live, are never mutually noalias — the guarantee is one usable mutable path per place [OWN-5]); shared borrows are read-only for their duration; owned values are unaliased except by their own live shared borrows.

[OWN-10] Borrow-storage duration: `&'a p` is legal only if `p`'s storage outlives `'a`.
For `p` rooted at an own-mode binding b: `'a` must be introduced within b's scope (never a caller-supplied region, for locals and own parameters alike).
For `p` rooted at a borrow of region `'b`: `'b` must outlive-or-equals `'a`.
For `p` rooted in `arena<'r, T>` content: `'r` must outlive-or-equals `'a`.
For `p` rooted at a named `const` item [CONST-2]: any region `'a` is legal; immutable static storage has program lifetime and outlives every region.

[OWN-11] Loops: inside the body of an ordinary `loop_stmt` or a counted `for_stmt`, a `borrow_expr` may name only regions introduced inside that same loop body, and a binding declared outside that body may not be moved inside it (copies exempt).
A counted binder may be copied and may be shared-borrowed only into a region introduced inside its body, but it may not be moved, uniquely borrowed, or otherwise transferred to a callee as a writable place; source writes are independently forbidden by [SET-1].
These restrictions are checked for each enclosing loop, so nesting never grants an outer binding or region to an inner body.

[OWN-12] Calls (OWN-CALL cluster): at a call, declared region parameters are substituted with the caller's region arguments, which must be live; argument borrows are live accesses of their resolved places for the duration of the call and are judged under OWN-5 (two `&uniq` arguments whose resolved places overlap are an error); the callee's effect paths are projected through the corresponding actual places under [EFF-2] and checked against the caller's live borrows under OWN-5. Region substitution controls loan liveness and type equality only; it never supplies effect identity.
When an argument is a statement-scoped or candidate-position child reborrow [OWN-6], its suspended ancestor holder is excluded from this effect-row overlap check, since the child, not the ancestor, holds the claim for the call; every non-ancestor live borrow is still checked.

[OWN-13] Match ownership: a non-place expression scrutinee is an owned temporary (moved into the match).
Matching a place of own mode moves it (the binding dies; binders receive `own` payloads); matching through `&'r` / `&uniq 'r` leaves the scrutinee live and binds payloads as `&'r` / `&uniq 'r` respectively.
Binder modes are derived by this rule, stated once; they are not written.
A borrow-mode payload binder is an arm-scoped child reborrow of the scrutinee place's root binding: resolved(binder-rooted place) = resolved(scrutinee place) ++ that payload's field suffix ++ any written suffix [OWN-6], sibling binders are judged by OWN-7 with any overlapping pair containing a `uniq` binder an error [OWN-6], and creating the taken arm's binders from a `uniq`-mode root suspends that root binding [OWN-5].
Binder borrows are live until the end of their derived region's block [OWN-4], so a matched-through `uniq` root does not resume within that region; each binder is usable within its arm, and a binder borrow moved onward retains its ordinary [OWN-4]/[OWN-5]/[GIVE-1] judgments inside that same window.
Binders of a shared-mode root are overlapping shared borrows admitted by [OWN-5] without suspension.
Arm-end resumption of a matched-through `uniq` root is DEFERRED with recorded delta [META-5].
A value initializer — a `let`-initializer `match` or `if` — binds its value from its arm or branch `give`s [GIVE-1]; scrutinee treatment and binder-mode derivation are unchanged, and each delivering arm or branch delivers a value of the binding's derived mode and type [GIVE-1, TYPE-5], so on the taken arm or branch an `own` result is moved exactly once (no double-move; T1 preserved).
A `give e;` whose `e` is a borrow reaching through a binder or an outer borrow obeys [OWN-4]/[OWN-5] exactly as a returned borrow of the same mode.
This arm-result region join is an additive reuse of the return-of-borrow judgment and is PROVISIONAL pending confirmation against the formalized calculus before section-5 ratification (D1a).

[OWN-14] Non-argument reborrow disposition and the returned reborrow.
A reborrow form is a `borrow_expr` [GRAM-5] whose `place` is rooted at a binding of borrow mode — a borrow-mode function parameter, a `let`-bound borrow holder [OWN-6], or a `match` binder of derived borrow mode [OWN-13].
A reborrow form occurring as an argument atom of a `call` expression is judged by [OWN-6] alone.
A returned reborrow is the written form `&'b` or `&uniq 'b` over `deref(h)` followed by any written suffix chain, occurring as the complete `expr` of a `return_stmt` [GRAM-4], admitted only when the eligible holder `h` is a function parameter or a `let`-bound borrow, never a `match` binder, and a `uniq` returned reborrow has a `uniq` holder while a shared returned reborrow has a shared holder. resolved(returned reborrow) = resolved(`h`) ++ suffix [OWN-6].
Its region obligations are the existing borrow-rooted judgments, stated once elsewhere: creation obeys [OWN-10]'s borrow-rooted case, and the created borrow is an ordinary returned borrow judged by [OWN-4] against the written `rtype` region and by [FN-1] against the written `rtype`, so the caller judges the call result by the signature alone, exactly as for `return h;` — the callee body never narrows or widens that judgment.
Creating a returned reborrow is judged under [OWN-5] and suspends `h` exactly as child creation does [OWN-6]; control leaves the function before the enclosing statement ends, so `h` never resumes and no program point observes `h` and the returned reborrow both usable.
Every other occurrence of a reborrow form, and a `return`-position reborrow failing this admission, is a hard error citing OWN-14 with the restructuring `pass the reborrow as a statement-scoped child in argument position, return it as the complete return expression from a parameter or let-bound holder, or return the holder itself`.
Bound reborrows, `give`-position and stored reborrows, `uniq`-to-`shared` downgrade, and `match`-binder parents (the derived payload binder itself is [OWN-13]'s arm-scoped child, not a written reborrow form) remain DEFERRED with recorded delta [META-5]; return position is the sole non-argument position admitted because its creating statement is the function's last program point.

## 6. Storage

[STOR-1] Storage class is a function of type, stated once: `box<T>` is heap-owned; `arena<'r, T>` is arena-owned, bounded by `'r`; `buffer<T>` is heap-owned (one compiler-derived heap allocation, released by one compiler-derived free at owner scope-exit [STOR-3]); a `const` item [CONST-2] is immutable static storage (program-lifetime, read-only, never dropped); every other owned value is frame-resident (inline in its owner or the stack frame).
There is no per-binding storage annotation and no default clause.
The reserved storage-contract field `foreign_shared` exists in the vocabulary but is legal only in programs containing gated FFI frames (§14); compiler-inferred demotion of an allocation to foreign-shared is a floor violation.
SET-1 may overwrite only a copy-typed final place, and [SET-2] may replace only a region-free affine final place; the two forms partition writable final places by [OWN-1] class with one spelling each.
Setting an affine-typed final place with `set` is a hard error citing STOR-1 at the complete target `place`, carrying its exact affine type and the restructuring `use replace: let old = replace p = e; binds the previous owner`.
This specification defines no bare take operation, temporary uninitialized hole, vacancy type state, or implicit destruction of the old affine value: [SET-2]'s atomic exchange is the sole affine replacement form, and the previous value always leaves through its fresh binding.
Growable or keyed collections (dynamic vector, hash map, set, byte-string, text) are neither storage classes nor kernel constructs: they are library structures over `buffer<T>` plus struct/enum and generics (a byte-string is `buffer<u8>`; a growable vector pairs a `buffer<T>` with a length, growing by allocate-new, move, [SET-2] field replace, and ordinary release of the superseded buffer).
The arena-index-pool ownership pattern remains rejected as a collection basis (it resurrects use-after-free as well-typed slot-recycling); keyed collections additionally remain blocked on their own occupancy and identity designs.
Char and Unicode text are out-of-v0, recorded.

[STOR-2] Creation: `box_new(v)` returns `own box<T>` for `v`'s exact type T [OP-2]; `arena_new<'r, T>(v)` returns `own arena<'r, T>`; both are ordinary calls in the operation table.
Content access is through `deref`.

[STOR-3] Deallocation and resource release are compiler-derived and explicit in the checked program [DIAG-2]: every drop and every release is represented before lowering.
Every control-flow edge leaving a region block (fallthrough, `break`, `return`) carries that region's releases and drops in reverse declaration order.
Release actions run only on normal control-flow edges; a trap runs none [EFF-4].
No reference counting.

Every edge that leaves one entered `for_stmt` body normally — its fallthrough, a `break` naming that counted loop or an enclosing loop, a `return`, or a `propagate` error edge — carries exactly once every compiler-derived drop and release for the body scopes that edge leaves, innermost scope first and in reverse declaration order within each scope.
On body fallthrough those actions complete before the hidden counted update [FN-1].
The header's false edge never enters the body and therefore carries no body-scope cleanup.
A trap retains [EFF-4]'s no-release behavior.
No exit duplicates an action already carried by an inner scope edge.

The release action of a type is compiler-owned semantic data selected by that type, not a fixed enumeration of memory-reclamation actions.
A `box<T>` drop is one compiler-derived heap free.
A `buffer<T>` drop with copy-typed elements is one compiler-derived heap free on every owner-scope exit, ordered like a `box<T>` drop.
A `buffer<T>` drop with affine-typed elements [TYPE-2] is each element's compiler-derived drop in ascending index order followed by that same one heap free; for an element type whose own drop derives no action, the composite action remains exactly the heap free.
An `arena<'r, T>` value's storage is released with its region [STOR-4].
A `const` item [CONST-2] is never dropped.
Every other frame-resident owned value [STOR-1] has no release action.
Each of these memory-reclamation actions carries the empty effect row.

A compiler-owned system resource type additionally fixes exactly one release action in its normative type contract.
That action carries one ordinary state-effect row and one compiler-owned target contract.
The state-effect row is [EFF-2]'s release contribution; the target contract states whether the action may suspend and which completion milestone releases each retained loan.
The release row uses the table-local subject `owner`. When release consumes state supplied by one formal parameter path, [EFF-2] substitutes that path for `owner`; when it consumes a fresh local owner, the action remains local and contributes no enclosing effect.
No source construct selects, replaces, supplies, suppresses, reorders, duplicates, or observes a release action, and no release action is conditional on a source declaration.

There are no finalizers in the writer-registered sense: no source declaration, annotation, attribute, contract, conformance, or binding attaches a writer-defined action to a value's release, and this specification defines no construct that could.
This clause does not forbid the compiler-owned release action above, which is fixed by the language and its system type contracts rather than registered by a writer.

A successful SET-1 assignment replaces one copy value and therefore derives no drop, release, finalizer, or cleanup edge; an affine `set` target is rejected before checked-program construction [STOR-1].
A successful [SET-2] commit likewise derives no drop, release, finalizer, or cleanup edge: the previous value is not destroyed, and its later release, if its binding is abandoned, is that binding's ordinary scope-exit action under this rule.

[STOR-4] Arena confinement: a value of type `arena<'r, T>` may not be returned, stored into a field, or moved to a destination outside `'r`'s block; borrows of its content obey OWN-10 with source region `'r`.

[STOR-5] Storage is borrow-free and region-free.
A type is region-bearing when its complete type after generic substitution contains `slice<'r, T>` or `arena<'r, T>` at any depth.
No struct field, enum variant payload, `array`/`buffer` element, or `box`/`arena` content may be a borrow or region-bearing type.
The `field`/`vfield` grammar admits only `type`, and `type` has no borrow (`&` / `&uniq`) production [GRAM-3]; the semantic check is recursive after substitution and therefore also closes indirect forms such as `box<slice<'r, T>>`, `arena<'a, slice<'r, T>>`, and a generic field instantiated with a region-bearing type.
A violation is a hard error citing STOR-5 at the complete contained `type` whose placement would make storage region-bearing, with the restructuring `keep the slice or arena as a direct local, parameter, or result; do not store it inside another value`.
A direct `slice<'r, T>` or `arena<'r, T>` remains a legal complete parameter, local, or result type where its owning rules admit it.
Substituting a region-bearing T into `box_new` or `arena_new<'a, T>` places T in an enumerated `box` or `arena` content position and therefore rejects under STOR-5: for `arena_new`, at the complete `type` child of that operation call's `targ`; for `box_new`, whose content type is derived from its operand [STOR-2, OP-2], at that operand `atom` node and its complete checked half-open source extent.
A `slice` element is not one of this rule's enumerated stored-content positions: writing `slice<'s, slice<'r, T>>` does not by itself violate STOR-5 and retains its v0.16 type-formation status.
The ordinary `slice_of` source path cannot construct that value because its array or buffer source would place the region-bearing inner slice in an element position prohibited above. [FN-2] separately rejects every region-bearing generic type argument at its source `targ`.

Consequently borrow and slice provenance cannot hide in a stored or generic payload.
An ordinary borrow can leave a callee only through its direct return value.
A slice can cross a function boundary as one direct parameter or one direct `own` result whose [OWN-5] origins are checked by [FN-1]; a borrow-mode result of direct slice type is rejected by FN-1 because it would carry two provenance relations.
Per-leaf provenance inside stored values, `Result`, `Option`, user nominals, boxes, arenas, and other generic instances is a DEFERRED specification addition; a compiler limitation does not select that boundary.

[STOR-6] Concrete target layout is a target-stage obligation after complete source-semantic acceptance and monomorphization.
It neither supplies nor replaces a source type-formation, ownership, recursion, or monomorphization judgment; an unavailable earlier semantic judgment remains an unsupported compiler capability rather than becoming a target-layout failure.
For this rule, the selected target is the exact backend target and ABI fixed by the compiler executable together with its invocation options; it is not a source declaration, inferred source fact, or component of the [PROG-2] compilation-unit identity.
A target-independent checked program and target-independent IR may precede this obligation.
Before emitting any form whose object layout, allocator ABI, or address arithmetic depends on a target, the compiler must have selected that exact target and ABI.

For every concrete representation and compiler-generated target object in the ordinary facts-off target-stage materialization set after monomorphization and before optional optimization, the compiler computes the representation's size, alignment, field or payload offsets, element stride, and padding under that target's ABI using checked mathematical arithmetic.
The target-object calculation includes the complete size, alignment, and offsets of statics, complete stack frames and their slots or temporaries, and call/return ABI objects.
A semantics-preserving omission made by ordinary facts-off lowering creates no target materialization to check; optional optimizer facts and facts-on dead-code elimination may not shrink the established set or change target-layout success.
Each result is checked against the actual allocation, ABI, and address-index domain in which that object or value will be used.
If a required result is not representable, target compilation stops before emitting target-dependent output that contains the materialization.
It must not wrap, truncate, underallocate, reduce alignment, change [STOR-1] storage class, emit an unrepresentable materialization on the assumption that an optimizer will erase it, or continue into target address formation.
This stop is a target-layout failure under [DIAG-1], not a source-language rejection, and cites no language rule.

For a runtime-sized allocation, the concrete descriptor and element layout are checked statically as above.
For every type materialized by `buffer_new` or `buffer_vacant`, target qualification additionally verifies the actual size, alignment, and element stride against [OP-9]'s language ceilings before lowering the operation.
The source `buffer_fits` proof and this target qualification jointly establish that the u64 byte-count multiplication cannot overflow; neither alone authorizes emission.
Before the allocator is called, generated code checks or otherwise establishes that the complete runtime byte count has an exact value-preserving representation in the target allocator-parameter domain, and the allocator receives exactly that value.
Every emitted target address computation must likewise be valid for every runtime value that reaches it: generated code checks or otherwise establishes that each runtime index and each mathematically scaled byte offset actually used by the computation has an exact value-preserving representation in the applicable target address-index domain, and that scaling and offset addition do not wrap.
An [OP-4] bounds judgment together with an established complete-object-layout or successful-allocation invariant may discharge these obligations; a backend's implicit narrowing does not.
A failed dynamic target-domain guard follows a non-continuing TCB/resource-failure path before allocator invocation or address formation.
It is not a source rejection, [OP-4] bounds failure, new language trap, or [DIAG-3] event.

Complete generated frames remain subject to the mandatory checked-representability judgment above.
That judgment does not predict available stack capacity: available capacity depends on dynamic call depth, recursion, the caller, and the execution environment.
The language therefore defines no numeric per-array, per-object, or per-function frame ceiling.
A tool or selected target may stop compilation for its own conservative frame-capacity or resource limit as a non-language target/resource failure [DIAG-1], but that optional limit does not replace the mandatory representability judgment.
Exhaustion during execution is inside the compiler/runtime/OS TCB boundary [SCOPE-3], not a language trap: it adds no source effect, produces no mandatory [DIAG-3] record, and authorizes no hidden heap promotion.

## 7. Operations

[OP-1] Every computation is a call naming one operation from the operation table; one operation per (semantic operation × mode); nothing is overloaded.
The table below is the normative inventory (columns: op, type domain, signature, effects).

```wf-ops
| op | domain | signature | effects |
|---|---|---|---|
| `+wrap` `-wrap` `*wrap` | all int T | `(T, T) -> own T` | pure |
| `+` `-` `*` | all int T | `(T, T) -> own T` | pure |
| `+defined` `-defined` `*defined` | all int T | `(T, T) -> own Bool` | pure |
| `+checked` `-checked` `*checked` | all int T | `(T, T) -> own Result<T, Overflow>` | pure |
| `/` `%` | all int T | `(T, T) -> own T` | pure |
| `/defined` `%defined` | all int T | `(T, T) -> own Bool` | pure |
| `/checked` `%checked` | all int T | `(T, T) -> own Result<T, DivError>` | pure |
| `ineg.wrap` | signed int T | `(T) -> own T` | pure |
| `ineg` | signed int T | `(T) -> own T` | pure |
| `ineg.defined` | signed int T | `(T) -> own Bool` | pure |
| `ineg.checked` | signed int T | `(T) -> own Result<T, Overflow>` | pure |
| `ieq` `ine` `ilt` `ile` `igt` `ige` | all int T | `(T, T) -> own Bool` | pure |
| `eeq` `ene` | one exact nominal tag-only enum T (every variant nullary), including `Bool` | `(T, T) -> own Bool` | pure |
| `fadd.strict` `fsub.strict` `fmul.strict` `fdiv.strict` | f32 f64 | `(T, T) -> own T` | pure |
| `feq` `flt` `fle` `fgt` `fge` `fne` | f32 f64 | `(T, T) -> own Bool` | pure |
| `band` `bor` `bxor` | Bool | `(Bool, Bool) -> own Bool` | pure |
| `bnot` | Bool | `(Bool) -> own Bool` | pure |
| `cvt` | value-preserving pairs [OP-6] | `(Src) -> own Dst` | pure |
| `cvt` | all other distinct numeric pairs [OP-6] | `(Src) -> own Result<Dst, NarrowError>` | pure |
| `len` | `slice<'r, T>`, `array<T, N>`, `buffer<T>` | `-> own u64` | pure |
| `slice_of` | `array<T, N>`, `buffer<T>` | `&'r place -> own slice<'r, T>` (a borrow of the whole array/buffer place) | pure |
| `box_new` | any T | `(own T) -> own box<T>` | allocates(heap) |
| `arena_new` | any T | `(own T) -> own arena<'r, T>` | allocates(arena 'r) |
| `array_new` | `T` copy (v0: primitive), `N` a constant-expression [CONST-1] | `(T) -> own array<T, N>` (fills all N elements with the argument; T1) | pure |
| `buffer_fits` | `T` a concrete region-free buffer-storable type [TYPE-2, OP-9] | `(u64) -> own Bool` | pure |
| `buffer_new` | `T` copy (v0: primitive) | `(u64, T) -> own buffer<T>` (allocates a flat buffer of the u64 length and fills every element; T1) | allocates(heap) |
| `buffer_vacant` | `T` region-free [STOR-5] | `(u64) -> own buffer<Option<T>>` (allocates a flat buffer of the u64 length; every element is `None()` of `Option<T>`, compiler-minted, no source value duplicated; T1) | allocates(heap) |
| `iand` `ior` `ixor` | all int T | `(T, T) -> own T` | pure |
| `inot` | all int T | `(T) -> own T` | pure |
| `ishl.wrap` `ishr.wrap` | all int T | `(T, u32) -> own T` | pure |
| `ishl` `ishr` | all int T | `(T, u32) -> own T` | pure |
| `ishl.defined` `ishr.defined` | all int T | `(T, u32) -> own Bool` | pure |
| `irotl` `irotr` | all int T | `(T, u32) -> own T` | pure |
| `ipopcount` `iclz` `ictz` | all int T | `(T) -> own u32` | pure |
| `ibswap` | int T, width>=16 | `(T) -> own T` | pure |
| `imulhi` | all int T | `(T, T) -> own T` | pure |
| `+sat` `-sat` `*sat` | all int T | `(T, T) -> own T` | pure |
| `imin` `imax` | all int T | `(T, T) -> own T` | pure |
| `iabs.wrap` | signed int T | `(T) -> own T` | pure |
| `iabs` | signed int T | `(T) -> own T` | pure |
| `iabs.defined` | signed int T | `(T) -> own Bool` | pure |
| `iabs.checked` | signed int T | `(T) -> own Result<T, Overflow>` | pure |
| `reinterpret` | equal-width primitive pairs: i8<->u8, i16<->u16, i32<->u32, i64<->u64, {i32,u32}<->f32, {i64,u64}<->f64 | `(Src) -> own Dst` | pure |
| `fneg` `fabs` | f32 f64 | `(T) -> own T` | pure |
| `fcopysign` | f32 f64 | `(T, T) -> own T` | pure |
| `fmin` `fmax` | f32 f64 | `(T, T) -> own T` | pure |
| `ffloor` `fceil` `ftrunc` `froundeven` | f32 f64 | `(T) -> own T` | pure |
| `frem` | f32 f64 | `(T, T) -> own T` | pure |
| `fsqrt.strict` | f32 f64 | `(T) -> own T` | pure |
| `ffma.strict` | f32 f64 | `(T, T, T) -> own T` | pure |
| `finf` `fnan` | f32 f64 | `() -> own T` | pure |
```

Let `DotlessOperationNames` be exactly the set of distinct individual operation spellings enumerated in this rule's normative `op` column whose complete spelling satisfies IDENT and contains no dot.
Let `ModeWords` be exactly the suffix alternatives in FORM-3's active OPNAME formation rule together with the operator-form suffixes of [GRAM-1]; in this version the two carriers share one closed set, `{wrap, defined, checked, sat, strict}`.
`ReservedLowerNames` is exactly `DotlessOperationNames` union `ModeWords`.
A printed review list is non-authoritative and, when present, must equal the corresponding derived set.

Each distinct complete spelling in the operation table declares one operation-family identity, even when more than one row carries that spelling; the two `cvt` rows therefore belong to one `cvt` family.
An OPNAME callee resolves to its exactly spelled operation family.
An `infix_op` token resolves to its exactly spelled operation by the operator table row; infix resolution consults no name domain, and an operator token is never a declaration, callee IDENT, or OPNAME.
An IDENT callee whose spelling belongs to `DotlessOperationNames` resolves to that operation family; every other IDENT callee admits a top-level source `fn_decl` or an admitted system operation [SYS-1].
Absence from the selected operation-family, function, or system-operation inventory is a hard error citing OP-1.
Later typed operation checking uses the operand domains and, for the retained-argument operations [TYPE-5], the written arguments, to select the applicable row within the resolved family.
Operand types never select between an operation family, a system operation, and a function.
A bare `place` operand that a table-operation row reads without consuming — the `len` operand, the place viewed by `slice_of` through its explicit borrow, and the base place of a subscript — is a non-consuming read: it neither moves nor partially consumes an affine root [OWN-1], exactly the reading [FN-8] already states for a place used as a non-consuming operand of an admitted table operation.

No source declaration or FN-9 result-datum candidate in this closed list may use a member of `ReservedLowerNames`: the IDENT of `fn_decl`; the IDENT of `const_decl`; every `param` and `result_binding` IDENT; every `let_stmt` IDENT, including ordinary, propagate, value-match, and value-if lets; every `contract_define` IDENT; the second IDENT of any `fieldbind`, including a `result_route` payload binder; every `field` and `vfield` IDENT; and the IDENT-shaped interior of `region_params` and `region_stmt`.
Such a reserved spelling is rejected citing exactly FORM-3 before freshness ownership is considered.
Dependent field declarations participate in this pre-resolution reservation inventory even though their owner/member duplicates remain deferred.
No other declaration role is covered: type-generic TYPEIDs, const-generic IDENTs, LABELs, and contract-member `fn_sig` IDENTs remain outside this prohibition.
Dotted OPNAMEs cannot be declarations under the grammar.
This reservation keeps operation-versus-function resolution context-free [META-2] and keeps a field-access place from maximal-munching as OPNAME [FORM-3].

[OP-2] Integer value semantics are defined over mathematical integers and fixed-width bit strings, never host-language overflow or undefined behavior.
The closed integer-type set is `i8 i16 i32 i64 u8 u16 u32 u64`.
For `iK`, where K is 8, 16, 32, or 64, the value set is `[-2^(K-1), 2^(K-1)-1]`; for `uK` it is `[0, 2^K-1]`.
Let `M = 2^K`.
For any mathematical integer z, let u be the unique integer satisfying `0 <= u < M` and `u ≡ z (mod M)`.
Define `wrap_uK(z) = u`; define `wrap_iK(z) = u` when `u < 2^(K-1)`, and `wrap_iK(z) = u - M` otherwise.

The bare infix spellings `+ - * / %` and dotless spellings `ineg iabs ishl ishr` are the sole proof-required exact integer operations.
Every occurrence carries one canonical [ENT-6] integer-domain obligation equal to the corresponding total domain-query expression over the same selected type and exact operand-expression identities.
The checker accepts the occurrence only when the complete state discharges that goal; a refuted or unproved goal is a compile-time OP-2 rejection at the `infix` or `call` node.
A contradictory state discharges it under [ENT-4].
No runtime test, fallback check, trap site, checked-result conversion, or optimizer assumption is synthesized.
After discharge, the exact operation executes without a guard and returns the result fixed below.

For a common selected type T, the domain queries have these exact total Bool values:

```text
a +defined b  iff mathematical(a + b) belongs to T
a -defined b  iff mathematical(a - b) belongs to T
a *defined b  iff mathematical(a * b) belongs to T
n /defined d  iff d != 0 and (T is unsigned or n != MIN(T) or d != -1)
n %defined d  iff d != 0 and (T is unsigned or n != MIN(T) or d != -1)
ineg.defined(x) iff x != MIN(T)
iabs.defined(x) iff x != MIN(T)
ishl.defined(x, k) iff k < K
ishr.defined(x, k) iff k < K
```

Each domain query is pure, total, and returns `own Bool`; it does not execute the corresponding exact operation.
An executed branch condition, requirement, or retained claim continuation may establish its canonical goal through [ENT-3].
Merely computing the Bool value without an admitted fact source establishes nothing.
The former `.trap` spellings and hidden named aliases such as `iadd.trap` do not derive and are not compatibility names.

After discharge, exact add, subtract, and multiply return their mathematical result, which the obligation proves belongs to T.
Exact division is truncating toward zero and exact remainder satisfies `n = (n / d) * d + (n % d)` with the remainder having the dividend's sign or being zero; their obligation excludes both zero divisor and the signed `MIN(T), -1` pair for division and remainder alike.
Exact `ineg` returns mathematical `-x`, and exact `iabs` returns the nonnegative mathematical absolute value; their obligation excludes the signed minimum.
Exact `ishl` is the fixed-width left shift and may discard high bits, exact signed `ishr` is arithmetic, and exact unsigned `ishr` is logical; their sole domain condition is `k < K`.

The wrap forms return `wrap_T(z)` for add, subtract, multiply, and negation; `iabs.wrap(MIN(T))` returns `MIN(T)`.
`ishl.wrap` and `ishr.wrap` mask the amount to `k & (K-1)`.
The checked add, subtract, multiply, negation, and absolute forms return `Ok(value: z)` when their exact mathematical result belongs to T and otherwise `Err(error: Overflow())`.
`/checked` and `%checked` return `Err(error: DivideByZero())` for zero divisor, `Err(error: DivOverflow())` for the signed minimum/-1 pair, and `Ok(value: result)` otherwise.
The existing saturating forms clamp exactly as [OP-8] fixes.
There is no wrap division or remainder because divisor zero has no modular quotient or remainder.

For `ieq`, `ine`, `ilt`, `ile`, `igt`, and `ige`, both operands denote their mathematical values in T and the result is respectively `True()` exactly when `a=b`, `a!=b`, `a<b`, `a<=b`, `a>b`, or `a>=b`.
Ordering on signed T is signed mathematical ordering and on unsigned T is unsigned ordering.
All six comparisons are pure, total, Bool-valued operations.

Every operation above derives its selected type from its operands and carries no written type argument.
Operands that are specified as common-T must have one identical exact closed integer type or one live `Int`-bound generic type later concretized by [FN-2]; agreement never widens, converts, or consults an expected result.
The unary exact, defined, wrap, and checked negation and absolute families require the signed subset.
Shift values have selected type T and their amounts have exact type `own u32`.
A wrong argument kind or count, written argument, invalid concrete or generic domain, or unsigned negation/absolute cites OP-1; after TYPE-7 exclusivity, an exact-type mismatch cites TYPE-5 at the offending operand.
The table result type is exact, and the containing construct owns any later mode or result mismatch.

Mode membership is table data: add/subtract/multiply have exact, defined, wrap, checked, and sat; divide/remainder have exact, defined, and checked; negate/absolute have exact, defined, wrap, and checked; shifts have exact, defined, and wrap.
All these rows are pure.

[OP-3] Float ops that ROUND carry `.strict` (IEEE 754, no reassociation, no contraction) and are the family a future fast-math mode would relax: `fadd.strict` `fsub.strict` `fmul.strict` `fdiv.strict` `fsqrt.strict` `ffma.strict`.
Float ops that are EXACT or exact-selection are dotless: `fneg` `fabs` `fcopysign` `fmin` `fmax` `ffloor` `fceil` `ftrunc` `froundeven` `frem` and the six comparisons.
Approximation/fast-math modes remain an OPEN numeric-semantics question; a relaxed float op would be introduced as a distinct OPNAME (FORM-1-additive).

[OP-4] A subscript `p[i]` selects one element place of an indexable base: the base place `p`'s final selected type must be `array<T, N>`, `slice<'r, T>`, or `buffer<T>`, and the subscripted place's selected type is exactly that element type T — derived from the base place's already-fixed type [TYPE-5] — written where the binding carries an annotation, derived at a body `let` — by the same declared-type selection that types a field suffix, never from expected type or cross-statement inference; a subscript whose base's final selected type is not one of the three indexable types is a hard error citing OP-4 at that subscript's `psuffix` node.
The subscript carries the bounds obligation `i < len(p)` [ENT-6].
A discharged subscript reads or writes with no runtime bounds check in every build mode, and its checked-program disposition records the discharging derivation [DIAG-2].
Base discharge is judged before provenance: a subscript whose obligation the complete fact state does not discharge is a compile-time rejection citing OP-4 at that subscript's `psuffix` node, carrying the residual obligation rendered exactly per [ENT-6]; it forms no [PRV-2] or [PRV-3] candidate and publishes no checked program.
Its mechanical fix is a dominating branch establishing the residual [ENT-3], or, only when the residual is an independently true theorem outside the normative checker, a CLM-2-admissible residual `claim` with a complete exact `because` record [CLM-1].
Only after complete-state discharge succeeds may the constrained-subject gate replace that success with a [PRV-3] local-leaf rejection or retain a downstream demand for [PRV-2].
Discharge is a deterministic checker derivation [ENT-1]; a solver result never participates.
A `buffer<T>` obligation is over the runtime length term.
The offset atom has exact value mode and type `own u64`; after the [TYPE-7] implicit-read exclusivity, any other offset mode or type is a hard error citing OP-4 at the offset `atom` node, with `SourceCoordinate` equal to that atom's complete checked half-open source extent.
A subscript in a [SET-1] target forms the selected place without reading its stored value; its base and offset are evaluated during target evaluation, and its discharge judgment is identical in target position.
A successful bounds judgment neither narrows nor authorizes narrowing the offset or its scaled byte offset; target address formation additionally obeys [STOR-6].
System range calls carry their own static [SYS-8] obligations through the same [ENT-6] framework; no operation-internal range check is retained.
The [CLM-3] judgment reads successful U-view obligation metadata for consistency and postcondition premises but adds no separate strict subscript judgment or repair after the ordinary and provenance judgments have succeeded.

[OP-5] Every source condition and contract predicate requires its selected expression to have exact value mode and type `own Bool`, where `Bool` is the PRE-1 nominal type.
No integer, other enum, borrowed `Bool`, or implicit truthiness conversion is admitted [TYPE-4].
The implicit-read case already owned by [TYPE-7] is exclusive: when `e` uses a borrow-mode or box/arena binding where its referent `Bool` value would be required, that use is rejected citing TYPE-7 and OP-5 forms no candidate.
Every other exact-mode or exact-type failure is a hard error citing OP-5 at the selected `expr` node, with `SourceCoordinate` equal to that expression node's complete checked half-open source extent.
An `if` condition is executed control flow [GRAM-6], a `claim` condition is the one writer-authored runtime checked site [CLM-1], and a contract predicate is erased proof syntax [FN-8, FN-9].
This judgment alone creates no runtime check or effect.

[OP-6] cvt partition and semantics (cross-reference TYPE-4).
`cvt<Src, Dst>` is defined for every ordered pair of distinct numeric primitives; `cvt<T, T>` is not an operation. cvt is EXACT: it yields `Ok(y)` when the Src value is exactly representable in Dst (y the unique such Dst value) and `Err(NarrowError())` otherwise, and it never rounds, truncates, or saturates.
A non-integral float-to-int, an out-of-range value, a value not exactly representable in a narrower float, and any NaN or infinity targeting an integer all yield `Err`; for float-to-float, an infinity maps to the same infinity and NaN maps to the target canonical quiet NaN (value-preserving).
A pair is TOTAL — signature `(Src) -> own Dst`, no Result — where every Src value is exactly representable in Dst; the total pairs are exactly these 29: `iN->iM` and `uN->uM` for N<M; `uN->iM` for N<M; `{i8,i16,u8,u16}->f32`; `{i8,i16,i32,u8,u16,u32}->f64`; `f32->f64`.
Every other distinct numeric pair returns `(Src) -> own Result<Dst, NarrowError>`.

[OP-7] Operation-name convention (regularity, W1-predictable).
An arithmetic, logic, bit, or compare op carries a domain prefix — `i` (integer), `f` (float), `b` (Bool logic), or `e` (tag-only enum comparison, including `Bool`) — whether or not a cross-domain twin exists; the structural ops (`cvt`, `reinterpret`, `len`, `slice_of`, `box_new`, `arena_new`) carry no prefix.
`Bool` participates in the `b` family for boolean logic and the `e` family for tag-only equality; the operation name, not operand inference, selects the family.
A respelled operation's token is its one constant spelling under the same one-spelling-per-operation discipline.
Bare infix and dotless named integer spellings are proof-required exact operations; `.defined` is the distinct total Bool-valued domain query, not a result mode and not an execution of the partial primitive.
The total value-result policies remain `.wrap`, `.checked`, and `.sat` where [OP-1] lists them, and float `.strict` is unchanged.
Signedness-parametric lowering keyed on the operand-derived selected type [OP-2] (`ishr` is `ashr` for signed T and `lshr` for unsigned T; `imin` is `smin` or `umin`) is the same discipline as the `ilt` = `slt`/`ult` row, not overloading.
Nominal enum identity is likewise checked from the operand-derived selected type before `eeq`/`ene` lowering; equal representation width never makes distinct enum types interchangeable.

[OP-8] Edge semantics and confirmed lowerings for the operations added in this revision; every totality edge is closed here as table data, so no added row is writer-reachable poison (per T2 and W3).
`iand`/`ior`/`ixor` lower to `and`/`or`/`xor` and `inot` to `xor x, -1` (total).
A shift or rotate amount is `u32`; `ishl.wrap`/`ishr.wrap` mask the amount to `amt & (width-1)` and are total, exact `ishl`/`ishr` execute an ordinary shift only after [OP-2] proves the amount smaller than the width, `ishr` is `ashr` for signed T and `lshr` for unsigned T, and `irotl`/`irotr` lower to `llvm.fshl`/`llvm.fshr` whose amount is taken modulo width, so rotates are total.
`ipopcount` is `llvm.ctpop`; `iclz`/`ictz` are `llvm.ctlz`/`llvm.cttz` with is-zero-poison false, so a zero input returns the bit width (the zero-input fix); counts return `u32`.
`ibswap` is `llvm.bswap` (width a multiple of 16).
`imulhi` is the high half of the full double-width product.
`+sat`/`-sat` are `llvm.sadd.sat`/`uadd.sat` or `ssub.sat`/`usub.sat` clamping to T's range; `*sat` widens, multiplies, and clamps, which avoids the signed-saturation miscompile in `llvm.smul.fix.sat`.
`imin`/`imax` are `llvm.smin`/`umin` or `smax`/`umax`.
`iabs.wrap`, exact `iabs`, and `iabs.checked` use `llvm.abs` with is-int-min-poison false; `.wrap` returns `iK::MIN` on that edge, exact `iabs` is emitted only after its domain proof excludes the edge, and `.checked` returns `Err(Overflow())` there.
Every `.defined` query computes only its total comparison or overflow predicate and never executes the corresponding exact primitive.
`reinterpret` is the LLVM bitcast instruction for cross-domain pairs (int<->float; bit-preserving, all NaN payloads and sign bits preserved) and an identity bit-relabel for same-width int<->int resign (i8<->u8, i16<->u16, i32<->u32, i64<->u64); it is the bit-preserving counterpart of value-preserving `cvt`, giving bit-level resign a home distinct from cvt's value-preserving resign.
`fneg` is the LLVM fneg instruction (a sign-bit flip, not `fsub(0.0, x)`); `fabs` is `llvm.fabs`; `fcopysign` is `llvm.copysign`.
`fmin`/`fmax` are `llvm.minimum`/`llvm.maximum` (IEEE-2019, NaN-propagating, negative zero ordered below positive zero, deterministic); `llvm.minnum`/`maxnum` are not used, because their signed-zero tie result is unspecified and breaks the reproducibility FORM-1 requires.
`ffloor`/`fceil`/`ftrunc` are `llvm.floor`/`ceil`/`trunc` (roundToIntegral, staying in the float type); `froundeven` is `llvm.roundeven` (ties-to-even, matching `fadd.strict`).
`frem` is the LLVM frem instruction (the C `fmod`: remainder with the dividend's sign, truncated quotient, exact), a distinct operation from IEEE `remainder`.
`fsqrt.strict` is `llvm.sqrt` and `ffma.strict` is `llvm.fma` (single-rounding fused, distinct from the contraction [OP-3] forbids; a correctly-rounded libcall on hardware without an FMA unit).
The comparisons `feq`/`flt`/`fle`/`fgt`/`fge` are ordered (`fcmp o*`, false when either operand is NaN) and `fne` is unordered (`fcmp une`), so `fne` equals `bnot(feq)` on every input and `fne(x, x)` is true exactly when x is NaN.
`finf` is the positive-infinity value (negative infinity is `fneg(finf<T>())`) and `fnan` is the canonical quiet NaN; other NaN payloads are reachable through `reinterpret`.
For a tag-only enum T — the operand-derived selected type [OP-2] — `eeq(a, b)` is `True()` exactly when `a` and `b` denote the same declared variant of that nominal T, and `ene(a, b)` is its exact boolean complement.
Both operands must have that exact T, derived by [OP-2]'s agreement rule; representation equality never permits cross-enum comparison.
`Bool` is admitted by the same tag-only rule.
Both operations lower directly to equality or inequality of the validated discriminants in T's already-selected representation.
They are pure and total: after normal operand evaluation, the primitive does not inspect a payload, access memory, trap, convert a value, or introduce a new optimizer fact channel; an operand read still exhibits its ordinary effect before the primitive executes.
Payload-carrying enums, enum ordering, and enum/integer conversion remain outside the operation table.

[OP-9] `buffer_fits<T>(n)` is the pure, total, target-independent allocation-domain predicate
`n <= floor((2^64 - 1) / stride_ceiling(T))`, where `stride_ceiling(T) >= 1` is the language layout ceiling fixed below.
It returns `own Bool`, exposes no target ABI value, and has the same result for one source type and n on every qualified target.

`buffer_new(n, v)` over fill type T carries the one canonical obligation `buffer_fits<T>(n)`.
`buffer_vacant<T>(n)` carries `buffer_fits<Option<T>>(n)`.
Each is accepted only when [ENT-6] discharges that exact goal; its sole normalized component is the defining comparison above, which may supply an alternate L0 derivation of the same root.
The root does not project a new general L0 fact in the other direction.
A refuted or unproved goal is a static OP-9 rejection; a contradictory state discharges it under [ENT-4].
The length n is a protected subject under [PRV-2, PRV-3], so a claim about an unconditionally external n cannot launder it past the required real branch.
No runtime multiplication check, trap site, or fallback is retained.

All layout-ceiling arithmetic is over unbounded mathematical integers.
Let `round_up(x,a) = ceil(x/a) * a`.
For a sequence of `(size, alignment)` pairs, start at offset zero, round each current offset up to the next field's alignment, add that field's size, take aggregate alignment as the maximum of one and the field alignments, and round the final offset to that aggregate alignment.
The primitive `(size_ceiling, align_ceiling)` pairs are: `unit`, `Bool`, `i8`, and `u8` `(1,1)`; `i16` and `u16` `(2,2)`; `i32`, `u32`, and `f32` `(4,4)`; `i64`, `u64`, and `f64` `(8,8)`; `box<T>` `(16,16)`; `buffer<T>` `(32,16)`; and every opaque system type `(32,16)`.
A struct applies the sequence rule to fields in declaration order; an array repeats its element pair N times.
A tag-only enum with at most two variants has `(1,1)`, and every other tag-only enum `(4,4)`.
A payload enum, including `Option`, `Result`, and system enums, sequences a `(4,4)` tag followed conservatively by every variant payload field in variant and field declaration order.
Region-bearing slice and arena types are outside `buffer_fits`'s domain; existing recursive-type rejection remains, while box and buffer ceilings do not recursively expand their content.
`stride_ceiling(T)` is `max(1, size_ceiling(T))` after the aggregate rule.

Before emitting a stored type S, target qualification verifies that its actual size, alignment, and stride do not exceed the three language ceilings.
Only with both that qualification and the source obligation disposition may lowering emit `n * actual_stride(S)` as non-overflowing arithmetic.
Qualification failure is a target failure and may not become a runtime guard.
The [STOR-6] rule separately governs allocator and address-index representability; allocation failure remains a TCB/resource failure [SCOPE-3], never a language trap.
`array<T, N>` performs no runtime size computation: N is fixed at monomorphization and concrete target representability is checked under [STOR-6].
The language defines no numeric frame limit, and `array_new` remains pure because target-layout and resource failure are not program execution.

## 8. Functions, generics, contracts

[FN-1] A concrete function's callable boundary states everything ordinary callers need: parameter modes and types, the named result's mode and type, one formal-path state-effect row, region parameters, the ordered [FN-8] requirement GoalTemplates, the ordered verified [FN-9] normal-result RelationTemplates with their complete/unasserted/S4-blinded dispositions, the derived [PRV-2] provenance column, one compiler-derived result-state routing summary, and one compiler-derived target summary.
The result binder's spelling is mandatory but ignored by callable-signature equality and denotes no runtime storage.
The written templates are checked interface claims rather than trusted declarations; a caller consults only their verified finite summaries and never a callee body.
The provenance column is derived from the checked body and closed-unit fixed point, never written.
The written effect paths state which parameter-supplied state the function observes or changes. The checker derives the exact same set from body accesses, direct system contracts, releases, and calls and checks it in both directions under [EFF-2].
The result-state routing summary records, for each ordinary owned state leaf the result may carry, whether that value is fresh or is the same value supplied by one or more formal parameter leaves. It is derived from existing move, construction, match, return, and ownership flow; it adds no source syntax, identity, parent relation, permission, or runtime field. A caller uses it only to preserve those existing value identities when a later effect or compiler-derived release acts through the returned owner. A result with no state leaf has the empty summary.
The target summary states `never-suspends` or `may-suspend` and, for each reachable suspending action, the applicable `result-ready` components, `loan-released(formal path)` facts, and `terminal`; a release with no writer result has no `result-ready` milestone.
That summary is derived from exact system contracts and the finite concrete call graph, never written, inferred from a spelling, or weakened by a declaration. It describes suspension and ownership handoff only; it grants no access and supplies no concurrency or alias judgment.
Adding a protected parameter datum or payload projection to that column is a caller-visible interface change, exactly as strengthening the requirement GoalTemplate or RelationTemplate is.
A generic function carries the same boundary with its written type and const parameters, and each concrete [FN-2] instance substitutes them before its calls and body are re-checked.
A `fn_sig` has neither kind of template.
Function-signature visibility is the [TYPE-6] table.
Every explicit `return e;` must produce exactly the enclosing function's `result_binding` `rtype`; there is no result-mode or result-type conversion [TYPE-4].
The implicit-read case already owned by [TYPE-7] is exclusive: when `e` uses a borrow-mode or box/arena binding where its referent value would be required by the written `rtype`, that use is rejected citing TYPE-7 and FN-1 forms no candidate.
Every other return mode or type mismatch is a hard error citing FN-1 at the `return_stmt` node, with `SourceCoordinate` equal to the complete checked half-open source extent of its selected `expr` child.
FN-9 adds a stricter result and return-expression shape only for a function that declares an `ensures_clause`; a function with none retains every return form admitted here.

For a function whose written result is `own slice<'r, T>`, the written signature also determines one return-origin ceiling without additional syntax.
The ceiling contains `immutable-const` and the formal-slice origin of every parameter whose written mode and type are exactly `own slice<'r, T>` using that same formal region declaration and element type.
No parameter with a different mode, type, element type, or formal region is a supplier.
In particular a borrow-mode parameter and an `arena<'r, U>` parameter are not implicit slice suppliers.
Every explicit `return e;` producing that written result must have an [OWN-5] origin set contained in the ceiling.
Failure is a hard error citing FN-1 at the `return_stmt` node, with `SourceCoordinate` equal to the complete checked half-open source extent of its selected `expr` child and the restructuring `accept an exact direct input slice in the result region or keep the newly formed view in its caller; do not return a view of raw callee storage`.
OWN-10 independently rejects a returned origin whose storage is too short-lived.

A function whose written result mode is `&'d` or `&uniq 'd` and whose direct result type is `slice<'r, T>` is a hard error citing FN-1 at the complete `rtype`, with `SourceCoordinate` equal to that production's complete checked half-open source extent and the restructuring `return the direct own slice descriptor under its data region; do not return a borrow of a slice descriptor`.
This specification has no signature summary that carries both the returned descriptor's source-place provenance and the underlying slice value's complete origin set.
This rejection does not change any other returned-borrow judgment.
A function whose written result mode is `&'b` or `&uniq 'b` determines the result's provenance from its written parameters alone: a parameter is a provenance candidate iff its written mode is a borrow of the result's kind in the result's formal region `'b` [OWN-6].
Exactly one candidate is the result's debtor, and zero candidates is legal — OWN-10 admits no `'b`-region borrow rooted in callee-local storage, so the only remaining source is named `const` storage, whose immutable program-lifetime extent needs no claim [CONST-2].
The provenance judgment applies to a result whose written type is region-free; a region-bearing result type is rejected before it — a direct slice by this rule's slice sentence and an arena, in either result mode, by [STOR-4].
Two or more candidates, a same-region parameter of the other borrow kind, or any parameter whose written type names `'b` leaves the source undetermined and is a hard error citing FN-1 at the complete `rtype`, with `SourceCoordinate` equal to that production's complete checked half-open source extent and the restructuring `give the source parameter its own region so exactly one parameter shares the result's region and kind, or return the decision as a value and let the caller borrow from the source it names`.
The declaration is the error and no call is required to reach it: [GRAM-9] admits a computed value only through a preceding `let`, so a result no caller can bind is unusable by construction.

The signature-formation parts of these two slice-result judgments and of the borrow-result provenance judgment apply equally to a top-level `fn_decl` and a contract-member `fn_sig`: an `own slice` member has the same parameter-derived ceiling, a borrow-mode direct-slice member is rejected at that member's complete `rtype`, and a borrow-result member whose source its own parameters leave undetermined is rejected there too.
A `fn_sig` has no body returns to validate; any [FN-3] binding still requires its bound `fn_decl` to satisfy the complete body judgment independently.

At a call, an `own slice` result's origin set is computed only from the callee's written signature.
For every formal-slice origin in the ceiling, substitute the corresponding actual slice argument's complete origin set after ordinary argument checking, then take the deduplicated union and include `immutable-const`.
Substitution is simultaneous and recursive only over the finite set already attached to each actual value; it never opens the callee body.
A wrapper call therefore preserves its input terms, and a recursive call uses the same finite written ceiling without a body fixed point.
Distinct formal regions remain distinct even when one call writes the same actual region argument for both.
Conversely, every exact same-region, same-element slice parameter remains a possible origin; the caller does not remove an unused supplier by inspecting the body.
Thus a one-source pass-through result stays singleton apart from the nonconflicting const marker, while a genuine same-region choice conservatively retains every possible input.
This is signature checking, not inferred lifetime, body-derived interprocedural analysis, or an optimizer fact.

On entry to a `for_stmt`, the lower endpoint atom is evaluated exactly once and then the upper endpoint atom is evaluated exactly once, each under its ordinary atom, ownership, and source-check judgments; the compiler copies their mathematical u64 values into distinct immutable hidden lower and upper captures in that left-to-right order.
No header test or body operation occurs before both captures exist.
The compiler then initializes the fixed `own u64` binder to the lower capture.
At each header it performs one pure mathematical comparison of the binder with the upper capture.
A false result reaches the counted continuation without entering the body; a true result enters the body.
Thus lower greater than or equal to upper executes zero iterations after still evaluating both endpoints.
On normal body fallthrough, body-scope cleanup completes first [STOR-3], then the compiler updates the binder exactly once to its mathematical value plus one and returns to the header.
The true guard proves the old binder is less than the u64 upper capture, so that increment is representable; it is a pure compiler operation with no hidden trap, wrap, saturation, operation-table call, or effect, including when the upper capture is max(u64).
An edge leaving the counted body by `break` to that loop or an enclosing loop, `return`, or `propagate`'s `Err` path performs its ordinary cleanup exactly once and performs no hidden update.
The counted header's carried-identity set is exactly the bindings carried into the construct plus both captures and the binder; the continuation interface and a break naming the counted label carry only the incoming identities after their path-specific ownership, cleanup, and effect judgments, with the counted label, binder, and captures all out of scope.
Ordinary `loop_stmt` execution is unchanged.

Function completion and statement reachability use one conservative structural normal-control graph over the resolved function body.
For any statement s, `normal_successor(s)` is the entry of s's next sibling statement in the same block when one exists, and otherwise that containing block's normal exit.
A block entry reaches its first statement, or its normal block exit when it contains no statement.
An ordinary `let`, a `let` selecting `replace_let_rhs`, `set`, an expression statement, and a passed `claim` have a normal edge to `normal_successor(s)`.
A call or operation with a trapping effect also retains that normal edge; a possible trap never proves divergence.
A `return_stmt` has an edge only to the function-return sink.
A `region_stmt` enters its body, and that body's normal exit reaches `normal_successor(region_stmt)`.
A `match_stmt` enters every arm body, using an arm's normal exit when that body contains no statement, and each arm's normal exit reaches `normal_successor(match_stmt)`.
A `let_stmt` selecting `value_match` enters every arm body the same way and follows [GIVE-1]: each `give` edge reaches `normal_successor` of that enclosing `let_stmt`, each return edge reaches the function-return sink, and each resolved break edge reaches `normal_successor` of its target loop.
An `if_stmt` enters its then-block, and its else-block when it has one, using a block's normal exit when that block contains no statement; each block's normal exit reaches `normal_successor(if_stmt)`, and an else-free `if_stmt` also has its false edge directly to `normal_successor(if_stmt)`.
A `let_stmt` selecting `value_if` enters both branch blocks the same way and follows [GIVE-1] exactly as the `value_match` sentence above does; an else-position `value_if` of a chain contributes its own branch edges to the same enclosing `let_stmt` [GIVE-1], not to a nested one.
A `let_stmt` selecting `propagate_let_rhs` has an `Ok` edge to `normal_successor` of that enclosing `let_stmt` and an `Err` edge to the function-return sink [ERR-3].
A `break_stmt` reaches `normal_successor` of its resolved target loop, ordinary or counted.
A `loop_stmt` reaches its body entry, or its body's normal exit when the body contains no statement; the loop-body normal exit reaches the body entry again, or itself when the body contains no statement.
For this conservative judgment every `loop_stmt` also has an edge to `normal_successor(loop_stmt)`; no ordinary loop is assumed to diverge [GIVE-1].
A `for_stmt` reaches its compiler-owned preheader, the preheader reaches its header after both endpoint evaluations and binder initialization, and the header has both a true edge to its body entry (or the body's normal exit when empty) and a false edge to `normal_successor(for_stmt)`.
Its body normal exit reaches the compiler-owned update, and that update reaches the header.
A `break` naming the counted label reaches `normal_successor(for_stmt)` without the update.
Every counted header retains both structural edges even when its captured endpoints are constant, so no counted loop is assumed to execute or to diverge [GIVE-1].
The function-body normal exit has no successor.
These edges are structural and are not removed by constant evaluation, a proof, or backend reachability.

Each statement not reachable from function-body entry establishes an FN-1 rejection premise using `SourceNode` at the selected concrete statement production beneath its `stmt` wrapper and a `SourceCoordinate` equal to that production's complete checked half-open source extent.
When more than one statement establishes that premise, the reported one follows DIAG-1's implementation-defined deterministic traversal. [GIVE-1] remains the more specific owner of a statement following `give` in the same block, so that statement establishes no additional FN-1 reachability rejection.
The function body's normal exit must be unreachable.
If it is reachable, the function falls through and is rejected citing FN-1 at the `fn_decl` node, with `SourceCoordinate` equal to the complete source interval of the body-closing `}` token.
This requirement applies to `own unit` as well as every other result: successful completion is written `return unit;`; there is no implicit return.
A possible trap, a call with no termination proof, or a loop does not satisfy the return requirement.
This complete structural graph, its statement reachability, and every source call and claim identity are retained for source audit even when [FN-8] later proves one concrete instance uninhabited.
That proof changes only its checked body disposition and lowering authority; it never erases a source node or narrows the written effect row.

The optional `deny_claims` terminal is one caller-visible compile-time policy on each concrete function boundary.
It changes no parameter mode or type, return, effect, region, requirement, postcondition, provenance column, runtime body, or lowering, and it is absent from `fn_sig` and [FN-3] signature equality.
One concrete body may serve both ordinary and strict callers.
The derived strict summary reuses the finite concrete ordinary-call graph and SCC condensation already required by [FN-9], retains no foreign function-local derivation identity, and never creates a second graph or body.
A call consults this policy only where [FN-8] and [CLM-3] require the existing U judgment.

[FN-2] Function and nominal generics are monomorphization-only; instantiation arguments are always explicit; expansion is compiler-side, pre-IR; instantiations are re-checked as concrete code.
Every contract definition and requirement or postcondition template is substituted separately for each concrete function instance.
The [FN-8] uninhabited judgment is likewise instance-local and never propagates from one concrete substitution to its generic template or another instance.
Every explicit type argument supplied to a function, source nominal, or PRE-1 nominal generic parameter must be region-free under [STOR-5].
A region-bearing argument is a hard error citing FN-2 at that complete `targ`, with the restructuring `make the slice or arena a direct written parameter or result instead of a generic argument`; there is no generic substitution, storage, result, or call-summary rule for a hidden slice or arena leaf.
Region-free arguments remain governed by the ordinary bound and substitution rules.
The optional `generics` child admitted syntactically on a source `contract_decl` receives FN-3's explicit rejection and creates no contract template or contract monomorphization.
A generic type parameter's written contract bound is admitted only when it resolves to the prelude `Int` or `Float` marker; a source-contract bound receives FN-3's explicit rejection and creates no bound-satisfaction or behavior-selection judgment.

[FN-3] A source `contract` is a compile-time signature-and-law bundle.
It has no `generics`; a contract declaration carrying that optional grammar child is a hard error citing FN-3.
Its `fn_sig` members are ordered by declaration order and their names are unique within the contract; the later same-name member is an FN-3 rejection.
Each member signature is checked under the ordinary mode, type, region, and effect rules.
The optional `doc` contributes no member, law, or semantic authority.
A contract with zero members and zero laws is a legal marker contract.

A source `conform D : C { ... }` requires D to be one concrete type after ordinary constant evaluation and nominal instantiation.
C resolves under [TYPE-6] to one nongeneric source contract and carries no `targs`; a conformance that names either prelude marker contract, carries contract arguments, or lacks a concrete subject is a hard error citing FN-3.
Its key is `(D, source contract declaration)`.
Concrete type identity in this key is exact: primitive and `unit` types equal only the same primitive or `unit`; compound types equal only when they use the same type constructor and their region, type, and evaluated-constant arguments equal recursively in position; and a nominal instance equals only an instance of the same nominal declaration identity—the same source nominal declaration or the same PRE-1 nominal declaration—with equal concrete arguments.
Layout equality, member equality, spelling equality across distinct declarations, and implicit conversion never establish type identity.
Across the complete closed compilation unit, at most one source conformance may have one key; source-record paths and item order do not create another coherence domain.

For each contract member in declaration order, the conformance contains exactly one `fn_bind` in that same position.
The binding's left IDENT equals that member name, and its right IDENT resolves under [TYPE-6] to one top-level source function.
A contract with no members therefore has an empty conformance body apart from its optional leading `doc`.
A missing, extra, repeated, unknown, or out-of-order binding is an FN-3 rejection; no member is inferred from a function name or signature.

The bound function has no `generics` child or `contract_block`.
Region parameters are permitted and are not a `generics` child.
Its callable signature equals the member signature exactly: the two signatures have the same number of region parameters and value parameters; corresponding parameter modes and types, result mode and type, and normalized effect rows are equal after replacing every occurrence of the member's first, second, and later declared region parameters with the bound function's region parameters at those same zero-based ordinals.
The two mandatory result-binder spellings are ignored by that equality.
This replacement applies inside modes, types, and arena-allocation payloads; type components then use the preceding exact concrete-type identity recursively.
After each signature's independently applicable EFF-1 judgment and the bound function declaration's EFF-2 judgment succeed, an effect row normalizes to four components: the set of declared read paths, the set of declared write paths, the allocation set whose members are `heap` and each alpha-mapped `arena` region, and the presence or absence of `traps`; `pure` is four empty components.
An effect path uses its root parameter's zero-based ordinal followed by its static source-struct field ordinals. Parameter and field spellings do not create signature identity.
Equality requires all four components to be equal.
A `fn_sig` has no body and no compiler-derived release, so it declares these components without an EFF-2 judgment of its own; the bound `fn_decl` must exhibit exactly the member's declared row under [EFF-2], including a path the bound function contributes only through release.
The compiler-derived target summary is not a writer declaration and does not participate in source contract equality. After a conformance selects a concrete bound function, ordinary closed-world propagation uses that function's derived summary at every call.
Source occurrence order and repeated occurrences do not affect this equality, but no path, allocation, or trap component may be omitted or added; there is no effect subtyping or semantic implication.
Parameter identifiers and region identifiers themselves need not have equal spellings.
There is no parameter or result variance, mode coercion, effect subtyping, omitted effect, default, receiver, implicit subject parameter, or `Self` substitution.
Any mismatch is an FN-3 rejection.
A valid conformance is one complete source-ordered binding vector; no partial conformance or member result is published.

The prelude marker contracts `Int` and `Float` [PRE-1] retain their built-in closed conformer sets (`Int`: i8 i16 i32 i64 u8 u16 u32 u64; `Float`: f32 f64), not user `conform` declarations.
A generic type parameter bound by `Int` or `Float` admits exactly its built-in set and makes the corresponding operation-table rows [OP-1] and identity literals `0_T`/`1_T` [FORM-5] available, monomorphized to the concrete type's operations.
A generic type parameter naming a source contract as its bound is a hard error citing FN-3 after that bound has resolved successfully.
This specification defines no source-contract bound-satisfaction, implication, inheritance, blanket conformance, structural conformance, inference, overlap choice, specialization, negative conformance, or conformance supplied by a function signature.

Contracts, conformances, binding vectors, and the base law records required by [FN-4] are compile-time checked-program metadata.
They have no source-visible runtime value, storage, address, ABI component, initialization, destruction, effect, vtable, dictionary, function pointer, indirect call, target object, or lowering operation.
A bound function remains one ordinary directly named top-level function and is compiled only through its normal function path.
Every contract law is first checked as a declaration under [FN-4]; a contract without a conformance is a legal outstanding obligation.
For every validated conformance of a law-bearing contract, every law must then obtain FN-4's complete mandatory discharge before semantic success.
A law-free conformance needs no law record.

[FN-4] A law of a source conformance is admitted only through the mandatory closed discharge below.
A successful discharge is source-acceptance evidence, not optimizer authority.
For a domain D, totality means every application to values in D terminates without trapping and returns a value in D.
A law-table row also defines its result-equivalence relation `≡D`.
The checked equations are `f(f(x, y), z) ≡D f(x, f(y, z))` for `associative`, `f(x, y) ≡D f(y, x)` for `commutative`, and both `f(e, x) ≡D x` and `f(x, e) ≡D x` for `identity`, universally quantified over D.
`pure` alone proves neither totality nor an equation [EFF-3].

For FN-4 only, the following law-discharge relation is complete.
Source-law discharge starts from one whole conformance already validated under [FN-3], where D is one concrete integer type and the conformance names exactly the enclosing source contract that owns the law.
The law's `f` role equals the name of exactly one `fn_sig` in that contract, and the conformance's complete binding vector selects that member's one bound top-level function.
D, both `fn_sig` parameter types and its return type, and both `fn_decl` parameter types and its return type are the same concrete integer type.
Both signatures have exactly two `own D` parameters in corresponding ordinal positions, `own D` return, no region parameters, and effect `pure`; their parameter identifiers need not be equal.
The bound function is nongeneric and has no `contract_block`.
A law/member/domain/signature premise missing from this stricter relation is a hard error citing FN-4 and publishes no accepted law.
FN-3 already owns whole-conformance identity, completeness, binding, and signature validity; FN-4 neither weakens nor repeats that authority.

After an optional leading `doc`, the bound function's body must contain exactly one statement, `return p0 +sat p1;`.
In this metanotation, `p0` and `p1` mean the exact identifiers declared by the bound function's first and second parameters; they are not required source spellings.
Each is used as one bare place, once and in declaration order, and the operation's operand-derived selected type equals D.
No alias, field, dereference, `move`, reordered argument, extra statement, second operation, user call, or semantically equivalent body matches this discharge shape.

The law table is closed:

| resolved table operation | complete domain D and `≡D` | total | associative | commutative | identity |
|---|---|---|---|---|---|
| `+sat` for T in `u8 u16 u32 u64` | every integer in `[0, 2^K-1]`; same integer value | yes | holds | holds | zero of T |
| `+sat` for T in `i8 i16 i32 i64` | every integer in `[-2^(K-1), 2^(K-1)-1]`; same integer value | yes | refuted | holds | zero of T |

Here K is T's bit width.
An `identity` argument matches the row exactly when it is a same-typed [FORM-5] literal denoting T's zero or an IDENT naming an earlier same-typed [CONST-2] value whose substitution result is that zero.
Unsigned saturating addition is `min(2^K-1, x+y)`, which makes the three `holds` cells valid over the complete unsigned domain.
The signed associativity cell is refuted for every listed width by taking x as `MAX`, y as `1`, and z as `-1`: the left association is `MAX-1` and the right association is `MAX`.
Every operation, domain, law, or identity absent from a `holds` cell is unavailable for source discharge; a `refuted` or unavailable requested law, or a member function outside the exact discharge shape, is a hard error citing FN-4.
This deliberately bounded calculus is part of language acceptance and is identical in every conforming compiler; a compiler's optional prover strength cannot accept more source.
Each successfully discharged `(contract-law node, concrete-conformance node)` pair contributes exactly one base derivation record to the checked program [DIAG-2].
The record references its conformance, contract law, bound function, operation row, concrete domain, law, and optional identity; no pair is omitted, shared, or deduplicated.
This record participates in source acceptance only and grants no lowering or optimization consequence.
The originating semantic check constructs it directly; no serialization or replay step is required.

A law may affect optimization only through a separately approved optional fact family whose verifier binds the exact checked-program instance, target, backend, proposition, and authorized consequence.
For a source law, that verifier independently rederives the complete contract/member/body/table/identity relation from the bound canonical syntax and checked semantic state; it does not trust the stored base derivation record.
For a gated law, it validates the exact ledger-entry identity and scope in addition to the proposition.
Absence, rejection, or resource failure in that optional path leaves source acceptance, semantic identity, explicit checks, and facts-off lowering unchanged.
A pre-approved opaque gated-family signature may separately carry a candidate law proposition through its soundness-obligation ledger [LEDGER-1], but that proposition is not a source `conform` discharge and reaches no optimizer without the same independently verified optional-fact boundary.
General source proof artifacts, additional operation rows, and other complete-domain proof calculi are DEFERRED specification additions.
Sampling, bounded testing, runtime enumeration, and the non-normative law-test harness may prioritize a future gate review but never license optimizer use.

The grammar accepts an IDENT law name and zero or more `law_arg` nodes so that syntax formation does not encode a semantic name, arity, or argument-role table.
The checker then requires the name, arity, and argument roles to equal exactly one row of this closed declaration table: `associative(f)`, `commutative(f)`, or `identity(f, e)`.
An `f` role is an IDENT resolving to one `fn_sig` declared in the enclosing contract; that signature has effect row `pure`, has exactly two parameters, and gives both parameters and its return the same mode and type.
An `e` role is a literal of that type or an IDENT resolving under the ordinary declaration-before-use rule to a named const of that type, and it must be usable at the operation's mode under the ordinary typing and ownership rules.
An unknown law name, wrong arity, wrong argument kind, unresolved role, a non-pure signature, a signature that lacks this exact same-mode/same-type binary shape, or role type/mode mismatch is a hard error citing FN-4.
A well-formed law in a contract with no concrete conformance is a legal stated obligation and emits no accepted-law evidence.
For each validated FN-3 conformance, the resolved member must obtain the complete mandatory discharge above before its law obligation is accepted or its accepted-law record is emitted.

[FN-5] There are no function values and no dynamic dispatch in the kernel.
Closed-set dispatch is `match`.
This specification has no source grammar or semantic operation that calls a contract member, selects a bound function through a generic parameter, or turns a validated conformance into callable behavior.
In particular, a contract member is not a `callee`, source-contract bounds are not admitted, and FN-3 metadata cannot select an ordinary user call.
Env-struct behavior parameterization, its explicit member-call form, source-contract bounds, substitution and direct-call proof, and their exact diagnostics are DEFERRED constructs with recorded deltas.
When such a form is proposed, monomorphized direct calls rather than function-pointer or dictionary dispatch remain the performance candidate to test; this specification does not claim that unavailable mechanism.

[FN-6] Recursion is permitted.
Polymorphic recursion is rejected by a syntactic rule: in any call cycle among generic functions, every call instantiates the callee at exactly the caller's own type parameters.
This criterion is DELIBERATELY stronger than finiteness requires (it rejects some finite permutation cycles): predictable, locally explainable rejection per OWN-8's reject-and-restructure posture; the diagnostic must name the cycle and the restructuring.
Rejection-rate measurement is a registered experiment.

[FN-7] Exactly one top-level `fn_decl` named `main` must exist in the compilation unit.
That declaration is the unit's sole entry and must carry the exact fixed `command` program-kind marker.
It is nongeneric, declares no region parameters, and has no `contract_block`.
Its mandatory result binder is writer-named and its written result is exactly `own ExitStatus`.
Its written effect row is any subset of `reads` and `writes` paths rooted in its own labelled inputs, `allocates(heap)`, and `traps`, in [EFF-1] canonical order; `pure` is the empty subset and no arena allocation is admitted.
The entry is invoked exactly once by program start [PROG-3].
A source `call` whose callee resolves to it is a hard FN-7 rejection: its standard inputs are supplied only at start and source has no second entry route.
No other `fn_decl` may carry `program_kind` or `input_label`, and a main without `command` is not an alternate entry form.

The closed standard-input table for kind `command` is:

| ordinal | label | written mode and type | supplied value |
|---|---|---|---|
| 0 | `command.args` | `own Args` | the immutable invocation-argument snapshot |
| 1 | `command.cwd` | `own DirectoryRead` | the initial working-directory state |
| 2 | `command.stdout` | `own Output` | the standard output sink |
| 3 | `command.stderr` | `own Output` | the standard error sink |
| 4 | `command.files` | `own FileFactory` | the source of one-shot file-open permits |

Every value parameter of main carries an `input_label` and selects one row of that table.
Its label tail equals that row's tail and the written mode and type equal the row exactly; the `command` prefix is fixed by grammar and there is no conversion, default, or inferred mode [TYPE-4, TYPE-5].
Every row is optional and each may be selected at most once: an unused standard input is omitted, and the selected parameters appear in strictly increasing table-ordinal order, so declared order is the one legal byte sequence [FORM-1, GRAM-8].
A `command` entry that selects no row is admitted and receives no standard input.
The binder IDENT written after `as` is chosen by the writer and is an ordinary `param` declaration in the lexical IDENT domain [TYPE-6].
Ordinal identity, never type identity, selects the supplied value: `command.stdout` and `command.stderr` share one type and remain two distinct inputs.
An unknown, repeated, or out-of-order label, a mode or type differing from its row, an unlabelled main parameter, an `input_label` on another `fn_decl`, and an `input_label` in a `fn_sig` are each a hard FN-7 rejection.
No label tail is a member of [OP-1]'s `ModeWords`, because [GRAM-1] would form `command.` plus that tail as one operation-name token.
The system declaration domain is admitted to every compilation unit under [SYS-3]; entry validation therefore never changes which system names exist or lets an invalid entry steal an earlier undeclared-name diagnostic.

The one canonical byte sequence for a complete five-input entry header whose body immediately returns is `command fn main(command.args as args: own Args, command.cwd as cwd: own DirectoryRead, command.stdout as out: own Output, command.stderr as err: own Output, command.files as files: own FileFactory) -> status: own ExitStatus writes(cwd) {` because the normal return edge performs the directory state's compiler-derived close while the file factory's logical consume has an empty row.
The [FORM-2] rule renders it without amendment; `program_kind`, `input_label`, and `result_binding` introduce no formatting boundary.

The entry states a program's complete standard-input access in its own signature, so no system value reaches another function except as a written parameter [FN-1]: there is no ambient system state, and no entry-supplied aggregate that source can own, name, or pass.
There is no global state and no `'static` region in v0: ambient mutable globals would (a) erode the noalias fact base every function otherwise gets from parameter-only reachability (P0; carding backlog: GlobalsAA-class evidence), (b) create hidden inter-function channels invisible in signatures (W3, FN-1 signatures-as-trust-unit), and (c) pre-seed shared state for the future concurrency layer (T1).
Immutable `const` items [CONST-2] are permitted and are not global mutable state: being read-only they never erode the noalias fact base (reads of frozen rodata add no aliasing hazard), create no hidden inter-function channel (the value is source-determined in the closed unit), and may be shared under ordinary immutable borrows [CAP-1]; no `'static` region is introduced (borrows of const-rooted places obey the OWN-10 const clause), and there remains no writer-mutable global and no `static mut` analog.
A standard input is not global state: it is one written parameter of one function, owned and moved under the ordinary rules.

A missing entry is an FN-7 rejection at `BundleRoot` [DIAG-1].
A duplicate `main` spelling remains the later-source [TYPE-6] duplicate.
Every other FN-7 rejection uses `SourceNode` with the complete checked extent of the named node: a missing command marker or contract-bearing main at the `fn_decl`; a `program_kind` on another declaration at that node; an unknown, repeated, or out-of-order label or one outside main at the `input_label`; a wrong mode/type or unlabelled main parameter at the `param`; a wrong result at the `rtype`; a wrong effect row at `effects`; generic or region parameters at their child; and a source call to main at the `call`.

The optional [CLM-3] marker may prefix the entry as `deny_claims command fn main(...)`.
It creates no second form or invocation path; a marked entry is one strict root while every FN-7 judgment remains in force.

[FN-8] Every non-entry source `fn_decl`, generic or nongeneric, may carry one optional `contract_block`; [FN-7] forbids it on main and `fn_sig` has no such production.
A present block must contain at least one `requires_clause` or `ensures_clause`; an empty or define-only block is an FN-8 rejection at `contract_block`.
Grammar fixes all definitions before all requirements and all requirements before all postconditions.

The definition scope initially contains the function parameters, named consts, and live type and const parameters, then each earlier definition after its complete initializer.
Every definition and clause expression must consist only of non-consuming datums and operation-table forms that are pure and total for every value in their selected operand domain.
User and system calls, construction, move, borrow, subscript, mutation, control flow, allocation, and every proof-required exact or otherwise partial operation are inadmissible even when another clause states their domain.
The corresponding `.defined` queries are total and admissible.
Each definition produces an own copy value, follows ordinary typing and no-shadowing, and is erased by recursive alpha-expansion into every later clause; no definition is evaluated, snapshotted, lowered, or visible in the body.

Each requires expression has exact mode and type `own Bool` under [OP-5] and independently forms one finite typed GoalTemplate after definition expansion.
A formal datum keeps its zero-based parameter ordinal and field or `deref` projections; named consts, literals, selected operation rows, written arguments after substitution, result types, and operand order retain their existing identities.
Definition spelling, sharing, and NodePaths are absent after expansion.
The requirement occurrence is `(concrete function instance, requires_clause NodePath)` and is outside predicate equality.
Two predicates are equal only by exact typed-tree equality: there is no commutation, folding, reassociation, inversion, De Morgan rewrite, or composition of child proofs into a parent.
Signed decomposition and an exact comparison-root L0 projection remain exactly [ENT-3, ENT-4].

At an ordinary source call, resolution, concrete instantiation, named arguments, exact types, borrow feasibility, and all actual-expression obligations complete first.
For every GoalTemplate in requires-clause source order, substitute each formal with that actual's value image in the same pre-transfer fact state: a borrow formal uses its resolved referent and an own actual its value before transfer.
A literal, named const, or place with field and `deref` projections remains an ordinary datum.
A subscripted actual uses the existing ephemeral identity `(concrete caller instance, call NodePath, argument ordinal, exact checked type)` and cannot be named as a source fact.
Every instantiated goal is judged independently in that unchanged state; a discharged clause adds no fact for a later clause.
The first refuted or unproved clause is the FN-8 call-site rejection and forms no provenance target or checked program.
Only total success reaches [PRV-2], then ordinary transfer, effects, and normal return; no call receives a runtime fallback, alternate entry, or body clone.
Main is not source-callable [FN-7].

At concrete body entry, every requirement goal is established independently as an [ENT-3] S4 source, in source order, with its own signed decomposition and exact L0 projection.
The clauses are never banded together.
There is no executable callee prologue, `llvm.assume`, optimizer license, or alternate lowering; later kills apply normally.
Direct and mutual recursion, forward calls, and every concrete generic instance use the same finite rule.

After all S4 sources and implicit parameter/type facts are closed under [ENT-4], a contradictory entry state makes that concrete instance legally uninhabited.
The checked body disposition is `Uninhabited { contradiction: DerivationId }`; it is success metadata, not a source rejection, and the derivation survives final identity remapping.
Syntax, resolution, type, ownership, effect, return-shape, statement reachability, call, and claim audits still inspect the complete source body, while proof obligations discharge under the contradictory state.
An uninhabited instance publishes no postcondition summary.
Lowering must preserve its ordinary ABI and symbol but emit exactly one empty entry block terminated by `unreachable`, without traversing or lowering any source statement.
A source call must still prove every contradictory requirement, which no reachable non-contradictory caller state can do.

When provenance retains an S4-dependent bridge, its requirement identity is the complete ordered set of that instance's requires-clause NodePaths, never one arbitrarily selected clause.
U keeps every S4 source and B removes them all; an upstream call rejudges every member of that ordered set in one unchanged pre-transfer view.

After ordinary complete-state and provenance success, [CLM-3] judges every required call in a demanded strict component, and a call from outside directly into a marked strict root, against all instantiated clauses in caller U at that same pre-transfer point.
Imported claims are tested first; otherwise the first refuted or unproved clause in source order owns the FN-8 rejection.
There is no marked-program-entry requirement query because main has no contract.

[FN-9] Each `ensures_clause` in a [FN-8] `contract_block` declares one independent verified normal-return relation.
It is neither an executable epilogue nor a trusted assertion; no contract definition or clause contributes an effect, runtime operation, storage slot, or [DIAG-3] record.

An unrouted clause is admitted only when the written result is `own T` and T is one [ENT-2] fragment integer after concrete [FN-2] substitution.
Its symbolic whole-result datum is the header `result_binding`.
A routed clause is admitted only as exact `when Ok(value: r):` for written result `own Result<T,E>`, where T is a fragment integer and r is that clause's fresh symbolic payload datum; `Ok` and `value` retain their PRE-1 identities.
Route owner, variant, field, and freshness admission precedes resolution of that clause expression [GRAM-10, TYPE-6].
The header whole-Result binder is unavailable in a routed clause.
Borrow-mode, unit, float, aggregate, nested-payload, whole-Result, non-Ok, and every other shape remains a legal ordinary result but cannot supply a relation datum in this version.
Omitting Err routes means Err exits are unselected, not unreachable.

After recursively alpha-expanding every shared `contract_define`, the clause expression must have exact type `own Bool` and its root must be exactly one of `ieq`, `ine`, `ilt`, `ile`, `igt`, or `ige`.
Both operands must be the clause's symbolic result datum, a parameter datum with field and `deref` projections, a named const, a typed integer literal, or `len(P)` for an admitted formal place P; at least one operand contains the result datum.
No proof-required exact operation, computed arithmetic result, subscript, ephemeral actual, Boolean connective, nested result projection, or body local becomes a relation term.
The comparison normalizes to one finite L0 RelationTemplate; equality's two bounds remain one relation occurrence.
Parameters denote function-entry images.
The template retains parameter ordinals and projections, route declarations, named-const identity, literals, substitutions, comparison row, operand order, and normalized relation, while excluding result/route/definition spellings, definition sharing, and callee identity.
Its occurrence is `(concrete function instance, ensures_clause NodePath)`.

An unrouted clause selects every explicit return.
A routed Ok clause selects exactly a direct canonical `Ok<T,E>(value: atom)` return and uses that payload atom as its result datum; direct Err and propagated error exits are unselected.
Every other Result return in a function with an Ok clause is an FN-9 rejection rather than an inferred route.
At a selected return, the result datum evaluates to one [ENT-2] term or constant.
For an ordinary inhabited instance, each clause's selected-return set is independently nonempty; an empty set rejects at that `ensures_clause`.
An [FN-8] uninhabited instance still checks route, type, expression, and return-shape source judgments, but is exempt from nonempty and proof requirements and publishes no relation.

Each referenced parameter entry image creates no snapshot term.
Its stability begins live at body entry and becomes permanently unavailable on the first structural edge whose [ENT-5] kill overlaps the datum, a holder used by it, or its support; join is intersection and contradiction never restores it.
An element write does not invalidate `len(P)`, while killing P's root or holder does.

For each clause in source order and then each selected return in NodePath order, the checker first completes ordinary return typing, obligations, calls, effects, and pre-return kills.
If an entry image is unavailable, the relation is unproved; otherwise substitute the result datum and query immediately before return transfer and edge cleanup.
Complete, U, then B is the fixed view order at that return.
Every complete query must discharge; the first clause/return failure rejects with no runtime fallback.
For each clause, its U and B aggregates discharge exactly when all selected returns discharge in that view; their failure is metadata, not declaration rejection.
A complete-only relation may depend on a claim, and U-but-not-B on one or more requirements.

Postcondition verification has no summary fixed point.
Form the concrete ordinary-call graph, its SCCs, and the callee-before-caller condensation.
While verifying a component, all same-component S12 summaries are unavailable; previously completed callee components remain available.
Only after every relation of every inhabited instance in the component has mandatory complete success are all its relation summaries published atomically; any failure publishes none.
Uninhabited instances contribute no summary.
Declaration or worklist order and iteration cannot change the result.

For one ordinary call c, `A0(c)` means resolution, concrete instantiation, named arguments, exact types, borrow feasibility, every actual-expression obligation, exact formal substitution, and complete success of every FN-8 requirement have all occurred in that order at the same pre-transfer point.
Failure forms no postcondition candidate and this predicate remains independent of the later provenance verdict.
For one relation q, `M(c,q)` holds only when q's route matches that exact establishment event, result and referenced formals substitute independently to live [ENT-2] terms or constants after ordinary kills, and no referenced actual is FN-8-ephemeral.
A discarded or nested result, stored or propagated whole outcome, unsupported or unselected route, killed support, or nonterm actual makes only that M false.

Let Cq, Uq, and Bq name q's per-view aggregates.
`Gv(c)` means every actual obligation and every instantiated requirement clause discharges in caller view v at the unchanged pre-transfer point; an empty set succeeds trivially.
Subject to A0 and M, failure-atomic scratch establishes q after transfer, consumes, borrow commits, callee-effect kills, and target kills: in complete iff Cq; in U first when Bq, otherwise only when `Uq and GU(c)`; and in B first when Bq, otherwise only when `Uq and GB(c)`.
Every establishment retains all complete actual-obligation and requirement parents; the Bq branch adds its B aggregate, while the Uq branch adds its U aggregate and exact same-view Gv parents.
No view borrows evidence from another.

All matching verified relations are established together on the admitted result route.
An unrouted fragment result establishes onto the fresh binding of a direct ordinary-let call.
A selected Ok payload establishes only when the ordinary call is the direct scrutinee of a `match_stmt` or `value_match`, at entry to its exact direct `Ok(value: payload)` arm.
A named, stored, aliased, propagated, discarded, or otherwise indirect whole outcome carries no pending summary token.

The existing narrow receiver routes remain per relation.
For `set x = user_call(...)`, x is a live bare own fragment of the exact result type and exactly one argument is direct non-consuming x; after transfer, effects, commit, and kill, a relation may substitute result with post-write x only when it omits the formal supplied by x and all other supports remain live and disjoint [OWN-7].
For a selected payload, the first arm statement may be exactly `set outer = payload;`; after the ordinary commit and kill, replace only result-payload occurrences in each established relation with post-write outer when every other support remains live.
These routes establish no equality and every projected, consuming, repeated, aliased, nonfirst, wrong-type, wrong-binder, or unsupported form establishes nothing.

All candidate S12 and delivery facts remain in one complete/U/B scratch while PRV-1 converges and PRV-2/PRV-3 finalize.
Any provenance event publishes neither a candidate fact nor a checked program.
Otherwise the full batch remains unpublished until [CLM-3] total success; any CLM-3 or strict FN-8 rejection discards it, and only total success commits once.
No candidate is individually committed or retracted and no second flow walk or negative fixed point exists.

Every successful selected-return proof and caller establishment extends [DIAG-2]'s one derivation DAG.
Postconditions add no runtime operation, hidden check, assume, optimizer license, serialized certificate, portable identity, alternate lowering path, or ABI field.

## 9. Effects (candidate unified-state revision)

[EFF-1] Row grammar: the `effects` and `effect` productions of the fence below, in exactly this canonical order (reads, writes, allocates, traps).

```wf-ebnf EFF-1
effects := "pure" | effect ("," effect)*
effect := "reads" "(" effect_path ("," effect_path)* ")"
        | "writes" "(" effect_path ("," effect_path)* ")"
        | "allocates" "(" ("heap" | "arena" REGIONID)+ ")" | "traps"
effect_path := IDENT ("." IDENT)*
```

A category appears at most once in one row.
`pure` is the unique spelling of the empty row.
Frame residency (STOR-1) is not an allocation by definition.
The spellings `external`, `blocks`, `memory`, `world`, and `capability` are not grammar atoms, effects, retired spellings, or reserved words. They satisfy IDENT wherever any other lowercase identifier does.

Every `effect_path` is rooted at one formal value parameter of the same callable. Each suffix selects one statically known source-struct field from the preceding type. A root resolving to a local, result binder, unrelated declaration, or non-parameter declaration is an EFF-1 rejection. An unknown field, an enum payload, a dynamic subscript, a dereference spelling, and every other place form are outside this candidate grammar. A bare parameter names the complete state that parameter supplies; a field path names only that structural substate.

For a borrow parameter, its effect path names the borrowed referent rather than the local reference representation. For a direct `slice<'r, T>` parameter, it names the viewed backing state rather than the descriptor. For an `own` parameter, the path names the incoming owned state. Merely moving, returning, or structurally repacking that value does not observe or change it; an operation which reads or changes its contents exhibits the corresponding path. A REGIONID never names effect identity: regions state loan liveness and outlives relations only.

The row describes observations and changes of ordinary Whitefoot state, allocation, and the sole writer-reachable trap. It does not distinguish memory from outside state and does not describe a host scheduling mechanism. Opaque system resources, buffers, aggregates, factories, permits, clocks, and Sources all use the same path, exactness, call-substitution, and ownership rules. No type or path carries a writer-visible capability category.
`reads(path)` means the operation observes that state. `writes(path)` means the operation replaces or advances that state. They remain independent exact facts: an operation which observes prior state while changing it names the path in both categories, while a complete overwrite need only write it.
Whether a target uses a native completion queue, readiness, polling, a bounded blocking helper, an interrupt, or inline completion is target data [QUAL-1], not a source effect.

[EFF-2] A concrete function declaration exhibits the union of exactly two contributions: its body-syntactic contribution and its release contribution.
The body-syntactic contribution is syntactic over the complete function body: it exhibits `traps` iff the body contains a `claim` or a call to an operation or function whose selected row includes `traps`; it exhibits reads, writes, and allocations from the resolved accesses, calls, and allocation operations the body uses.
Proof-required exact integer operations, integer domain queries, proved allocation operations, and proved system-range operations contribute no `traps`, because source acceptance precedes lowering and admits no runtime fallback.
A bare operator inside a `const` [CONST-1] is const evaluation under `const-reject`, not a trapping-mode operation, and contributes nothing to any effect row.
An optional `contract_block` consists only of erased definitions and proof clauses [FN-8, FN-9]; it contributes no read, write, allocation, or trapping category.
An [FN-8] uninhabited instance still derives and checks this contribution from its complete source body; the unreachable lowering stub never narrows its written callable row.
The release contribution is defined below and has no syntactic occurrence anywhere in the declaration.
A `for_stmt` endpoint and body contribute their ordinary source occurrences under these same clauses, and its body-exit cleanup contributes under the release rule below.
Its compiler-owned captures, binder initialization, header comparison, and representable hidden update contribute no read, write, allocation, or trapping effect.
Function-body attribution and call-boundary projection are separate judgments.

While one function body is checked, every exhibited read or write is attributed after ordinary place resolution, holder resolution, and [OWN-5] slice-view provenance.
An access to a borrowed referent, direct slice backing, or incoming owned state contributes the most precise formal-rooted static struct path [EFF-1] admits for that state. A dynamic element or range access maps to its nearest statically nameable enclosing path because [EFF-1] admits no dynamic selector.
A read through a shared, exclusive, or owned parameter may exhibit `reads(path)`. A write may exhibit `writes(path)` only when ordinary ownership already grants exclusive or owned access to that state. The effect path grants no permission, changes no loan extent, and cannot narrow a borrow of a whole aggregate to one field.
A named const root and `immutable-const` contribute no read effect because their state is permanently fixed [CONST-2]. Moving, returning, or repacking an incoming owner without inspecting or changing it contributes no effect. A fresh local own binding contributes no enclosing read or write effect, even when reached through a local borrow, local slice, or later local move.

A direct `slice<'r, T>` parameter names its viewed backing state rather than its descriptor. Reading through it contributes `reads(parameter)`; a slice derived from an incoming buffer or slice parameter retains that formal-rooted origin, and a multi-origin slice contributes the deduplicated union of every formal-rooted origin. The descriptor's own mode region still governs its loan, but no lifetime spelling enters an effect row.
Binding, moving, passing, returning, borrowing, reborrowing, and slicing preserve the existing resolved place identity. This is the same identity tracking already required by ownership and move checking; EFF-2 adds no parent link, result ancestry, resource root, or second provenance system.

At a user or system call, each callee effect path selects its root formal's actual argument and appends its static field suffix to that actual's resolved place. Holder resolution then reaches the borrowed referent, and a slice actual projects through its complete [OWN-5] origin set. A projection rooted in one of the current function's formals contributes the corresponding current-function path. A projection rooted only in fresh local state contributes no enclosing effect.
Thus a callee write through a child reborrow of incoming `&uniq` storage reaches the incoming formal path, while the same callee write through fresh local storage frames out. Equal lifetime arguments never merge two suppliers because lifetimes do not participate in this substitution.

Resource-producing calls follow the same rule. For example, `reserve_file<'r>(factory: &uniq 'r factory)` exhibits the callee's `writes(factory)` on the caller's `factory`; an open with `permit: move permit` exhibits `writes(permit)` only on that local permit; and later operations on the returned fresh local resource remain local. Creating the permit or resource establishes no hidden child-to-factory ancestry. Any externally visible change to the factory or namespace is the direct effect of the operation that changes that parameter and must appear in that operation's own row [EFF-5].
Framing an action on fresh local state out of the enclosing signature means only that it contributes no formal-rooted boundary path. The checked call still retains its instantiated nonempty effect on that local place. Eliminating it requires the ordinary closed-state, escape, result, release, and observer proof which justifies deleting any stateful call; absence from the enclosing row alone proves none of those facts. A target operation is lowered with its qualified physical side effects intact. The mandatory direct write on the creating factory, namespace, allocator, or permit prevents the enclosing call from becoming `pure` merely because the produced owner stayed local [EFF-5].

The release contribution collects the effects of compiler-derived release.
Under [STOR-3] each type fixes one compiler-derived release action together with that action's state-effect row.
For the function being checked, the release contribution is the union of the effect rows of every release action that may run on any edge of the conservative structural normal-control graph defined in [FN-1].
A release contributes when it may run on at least one such edge; running on only some paths never weakens it, and no path condition, constant evaluation, discharged law, optimizer fact, or backend reachability judgment removes an edge from that graph.
An owner moved or returned on one `match` arm and released on another therefore contributes its release row to the enclosing function, and so does a release derived on only one arm of any other branch, one `give` edge, one propagation edge, or one loop exit.
On each normal edge every owner has exactly one disposition — moved or returned, consumed by an explicit consuming operation, or released by exactly one compiler-derived release action — so one owner contributes at most one release per edge, and an owner consumed on that edge contributes no release there.
Release actions run only on normal edges; a trap runs none and contributes nothing [EFF-4].
A release action substitutes its released owner's resolved identity for the type contract's table-local `owner` path. Releasing an incoming owner, including one first moved through local bindings, therefore reaches that incoming formal path; releasing a fresh local owner frames out. A release-derived effect inside a callee belongs to that callee's row and reaches the caller only through the ordinary call-boundary projection of the callee's declared row; it is never attributed to two functions. The release's suspension and milestone summary propagates separately under [FN-1].

This attribution reads only the release rows [STOR-3] fixes, and it does not retrofit memory reclamation into effect rows.
A `box<T>` drop, a `buffer<T>` drop, an `arena<'r, T>` region release, and the absent drop of a `const` item [CONST-2] each carry the empty release row and therefore contribute nothing to any function's exhibited row; only a system resource type whose contract fixes a nonempty release row contributes one.

A [SET-1] commit is one write under this attribution, and a [SET-2] commit is one read and one write of the same target origin.
A shared-holder commit is rejected [OWN-5] and contributes no accepted effect judgment.
Effects exhibited while evaluating the target and right-hand side contribute normally; an accepted target subscript is discharged [OP-4] and contributes no `traps`.
Rows are checked both ways against the exhibited row defined above: undeclared-but-exhibited and declared-but-unexhibited are both errors, and an entry contributed only by the release contribution is checked exactly like one written in the body.
A mismatch involving the release contribution has no offending source occurrence, so it is a hard error citing EFF-2 using `SourceNode` at that function's `effects` node, with `SourceCoordinate` equal to that node's complete checked half-open source extent; the diagnostic additionally renders the parameter or binding whose release contributed the category, and the restructuring `declare the release effects of every resource this function may release, or move the owner out`.
When more than one owner establishes that premise, the reported one follows DIAG-1's implementation-defined deterministic traversal.
A function whose body and release contribution are empty may therefore declare `pure` while carrying an erased contract.
An explicit body `claim` still contributes `traps` to that caller.

Canonically, a nongeneric function whose only parameter is `own ReadFile` and whose complete body is exactly `return unit;` declares `writes(file)`.
Its compiler-derived release contributes that state write and a `may-suspend` target action on the function-return edge. Declaring `pure` is an undeclared-but-exhibited EFF-2 rejection.
This shape cannot be reduced further: [FN-1] requires the body's normal exit to be unreachable, so a function with an empty body is separately rejected and is not the canonical case.

[EFF-3] A call whose row is `pure` and whose derived target summary is `never-suspends` licenses deduplication and reordering with equal arguments.
Elimination of an unused such call additionally requires a termination proof; v0 provides no termination checker, so unused calls are not eliminated.
The source spelling `pure` excludes traps, state reads, state writes, and allocations; it does not promise termination.
A call that exhibits `writes(path)` may remain observable even when its result is unused. A call on fresh local state retains that instantiated effect even though it frames out of the enclosing signature. No optimization may erase, duplicate, speculate, or reorder either call unless ordinary effect-path overlap, closed-state, escape, ownership, control, result, release, and surviving-observer proofs establish the exact transformation; system state receives no separate effect category or observability tag.

[EFF-4] Trap is abort: there is no unwinding and no post-violation language cleanup.
The exact [DIAG-3] trap record is the sole mandatory post-violation language output.

[EFF-5] Every Whitefoot-observable system interaction is one ordinary state access under [EFF-1] and ordinary ownership under [OWN-1] through [OWN-12]. There is no second outside-state permission, root, fragment, coexistence relation, or global world object.

A system operation whose behavior or result can depend on evolving state carries the occurrence through at least one `own` or `&uniq` state parameter and exhibits `writes(path)` for that transition. Other inputs whose own Whitefoot-visible state stays stable for the complete loan may be shared and read normally; no shared object receives an interior-mutation exception. In particular, a file open consumes and writes a one-shot `FilePermit`, while the `DirectoryRead` and path or component bytes are stable selector inputs borrowed through `&`. An operation which creates a system resource or consumes finite quota must receive the factory, allocator, permit, or other changed state as an ordinary parameter and exhibit its write directly. No source operation obtains mutable system state from ambient process context.

The returned owner of a successful resource-producing operation is a fresh ordinary value. It carries no hidden ancestry to the parameter that produced it. The producing operation's direct parameter effect records the factory or namespace transition, while later actions on the returned local value frame out in the enclosing function exactly as actions on fresh local memory do [EFF-2]. Moves and borrows use their existing ownership identity and add no language feature.

The trusted target adapter, including any C implementation, must honor the same ownership boundary. Submission may retain only the loans recorded for that call; it may neither copy an affine owner nor access a retained referent after the target contract publishes `loan-released(path)` for that referent. Native completion rings, readiness tables, helper mailboxes, and device queues are target-private protocol state and are never exposed as ordinary shared Whitefoot storage [QUAL-1].

Two actions may overlap only when [PAR-1] or [PAR-2] proves that their ordinary places, loans, value dependencies, and exits permit it. Separate owned values are separate language places even when host paths, hard links, duplicated descriptors, redirected streams, or another process make their native implementations contact the same physical object. Those environment aliases neither merge nor separate Whitefoot places.

Reordering, deduplication, coalescing, hoisting, sinking, speculation, and elimination are licensed only when the complete state effects, loans, target milestones, result dependencies, and control flow preserve the same program observations. Facts-off compilation remains correct and may choose the sequential submission schedule without changing source acceptance.

## 10. Errors

[ERR-1] Recoverable errors are values: prelude `Result<T, E>` and `Option<T>` (§15), dispatched by `match`.
No exceptions, no unwinding, no panic values.

[ERR-2] Every `match` is exhaustive over declared variants; there are no wildcard arms.
Bool exhaustiveness is carried by `if`: an else-free `if` is the empty-alternative form, an `if` with `else` covers both, and a Bool-scrutinee `match` is rejected at GRAM-6.
The asymmetry is deliberate and content-driven: the empty then-block is admitted while the empty else is not, because the else-free form is the one spelling of the empty alternative.
Variant addition surfaces site-enumerated edit lists (toolchain contract).

[ERR-3] Propagation: `let x = propagate e;` requires `e : own Result<T, E>` and the enclosing function's return type `own Result<U, E>` (same E — no conversions, TYPE-4); x's derived mode and type are `own T` [TYPE-5].
The propagation operand is a consuming context.
A non-place Result expression is its owned temporary.
When `e` is a direct bare place of affine `Result<T, E>` type rooted in a live own-mode binding, propagation consumes that place exactly once under [OWN-1] without requiring a written `move`; a partial place consumes its whole root and retains the ordinary residual cleanup.
An explicitly written `move p` retains its ordinary OWN-1 meaning.
A place rooted through a borrow, a borrow or box holder used without `deref`, a dead root, and an outer affine root consumed inside a loop retain their TYPE-7, OWN-1, and OWN-11 judgments; ERR-3 grants no read-through, move-through-borrow, revival, copy, or loop escape.
The operand is consumed before the result tag is dispatched.
On `Ok(v)` propagation binds v; on `Err(err)` the function returns `Err(err)`, and the checked program attaches an auto-derived context record `(function, node_path)` to the propagation edge — zero hand-written tokens per site.
For an enclosing FN-9 `Ok` route, that automatic error return is unselected and publishes no normal-result relation.
This is Result propagation, not an exception construct or a region in which an exception may be thrown.
Derivation: R4 (keeps recoverable errors shift-left; manual re-match boilerplate invites silent context loss), W1 (one mechanical pattern), W3 (propagation cannot drop the error).

[ERR-4] Classification: expected environment and input failures are values (`Result`); unproved function, operation-domain, allocation-fit, bounds, and system-range obligations are source rejections; a false executed `claim` traps [SCOPE-4].
An operation's classification is fixed by its table row and attached static obligations, never by call-site preference.

## 11. Programs, closed world

[PROG-1] One closed compilation unit formed by [PROG-2]; every language name is defined within it, by the prelude (§15), or by the system declaration domain admitted to that unit (§16).
There is no include, import, module, separate compilation, incremental semantic cache, internal ABI, dynamic loading, reflection, or source-path lookup in the language.
A logical source path contributes identity only and never a namespace or lookup key.
The only external boundary for foreign code is the gated FFI wall (§14); compiler-owned system operations [QUAL-1] are implemented by an approved target entry rather than by foreign code, and are not such a boundary [GATE-2].

[PROG-2] One compilation unit is one ordered nonempty sequence of logical source records.
Each record contains one logical path and one exact source-byte sequence.
A logical path is an ASCII relative path made from one or more nonempty components separated by exactly one `/` byte, with no leading, trailing, or repeated `/`; each component contains only ASCII letters, ASCII digits, `.`, `_`, or `-`, and no component is `.` or `..`.
Path spelling is preserved exactly and compared case-sensitively.
An empty record sequence, an invalid logical path, or two records with the same logical path is an input-envelope failure, not a source-language rejection.
Record order is exactly the order in the bound invocation; no path sort, host enumeration order, or other reordering is applied.
Within that bound unit, a source record is identified by its zero-based ordinal, exact logical path, and exact source bytes.

[PROG-3] A conforming implementation starts one program instance by supplying exactly the standard inputs the entry declares [FN-7] and completing their ordinary target-side setup.
No source body statement executes during that setup, and no source construct observes, names, or reconstructs the private start-time aggregate through which a target may deliver those inputs.
After successful setup, the implementation transfers each declared standard-input owner exactly once into one invocation of the entry body.
There is no source contract on main [FN-7], entry-goal evaluation, runtime wrapper condition, helper function, duplicate body, or second external entry.
The optional `deny_claims` marker changes only the compile-time [CLM-3] closure and creates no start-time operation.

Supplying each declared standard input is a start-time obligation of the selected target.
When the selected target cannot supply one, start fails before the body is invoked: no source statement executes, no owner comes into existence, no language cleanup runs, and no `ExitStatus` is produced.
A start failure is a target or environment failure.
It is not a source-language rejection [DIAG-1], not a trap [SCOPE-4], and never rewrites a source acceptance judgment.

A `command` entry that completes normally returns exactly one `own ExitStatus` [FN-1].
Compiler-derived release for every owner live on that return edge runs before the instance terminates [STOR-3].
The selected target then maps that returned value to the process status exactly.
No other source value, written output, effect, release result, or target condition contributes to that status, and the language defines no second normal status channel.

A failing executed claim terminates the instance abnormally [SCOPE-4]: the entry's return edge is not taken, no release action runs, and no `ExitStatus` is produced or mapped.
Start failure and traps are therefore both outside the returned status, and `ExitStatus` carries normal command status only.

Every record is parsed as an independent [GRAM-2] `item*` sequence and audited as an independent [FORM-2] source.
A zero-byte source is a valid input record whose empty `item*` derivation fails [FORM-2]; the sole canonical zero-item source is exactly one LF byte.
No token, trivia item, grammar production below the compilation-unit `program` root, or source span crosses a record boundary.
The toolchain inserts no token, whitespace, delimiter, declaration, or separator between records.

The `program` root defined by [GRAM-2] owns the flattened sequence of all item nodes, ordered first by source ordinal and then by source-local item order.
Its location extent is `BundleRootExtent`, the ordered sequence `(source_ordinal, 0, source_byte_length)` for every record, including records with no item nodes.
It is not a fabricated source-local span.
Every descendant is source-local, and source records are not grammar-tree nodes.
Canonical formatting is checked separately for every record.
A record boundary and an empty record remain part of the bound source identity even when they contribute no item; repartitioning or reordering the same item bytes therefore changes that identity.

Top-level declaration order is source ordinal followed by source-local item order.
Name visibility is exactly the [TYPE-6] table.
Global uniqueness, the prelude, `main`, conformances, call graphs, strongly connected components, concrete instances, and reports range over the entire closed compilation unit.
Logical paths and record boundaries introduce no namespace, scope, import, or lookup key.

## 12. Diagnostics and checked compilation (toolchain floor)

[DIAG-1] Every source-language rejection cites exactly one numbered language rule and exactly one location from this closed sum:

1.
`SourceBytes(SourceCoordinate)` when no offending canonical-tree node exists or the defect belongs only to a source boundary;
2.
`SourceNode(NodePath, SourceCoordinate)` when one source-backed canonical-tree node is the offending node; or
3.
`BundleRoot(NodePath, BundleRootExtent)` for a whole-unit defect with no offending source declaration.
This form requires the empty root `NodePath` and carries no source-local byte interval.

`SourceCoordinate` is `(source_ordinal, byte_start, byte_end)` in the bound [PROG-2] unit.
Its byte interval is checked, half-open, and contained in that exact source.
End of source is the zero-width interval whose two offsets equal the source byte length.
`NodePath` is the sequence of zero-based child ordinals from the finalized compilation-unit root; the root path is the empty sequence.
Every source-backed node has one checked source-local extent.
In `SourceNode`, the rule-selected coordinate lies within that extent (`node_start <= byte_start <= byte_end <= node_end`) but need not equal the complete extent; the path identifies the existing offending or owning node while the coordinate identifies its exact offending subinterval or boundary.
`BundleRootExtent` is the exact ordered byte-extent sequence defined by [PROG-2], not a cross-source byte span.
A diagnostic never fabricates a node or node path.
A nested-place rejection additionally renders the offending access-path segment.

The frontend selects defects stage by stage.
Each stage scans every source in source-ordinal order and byte order, stops at its first defect, and the next stage begins only if the preceding stage succeeds for every source.
The stage order is: raw lexical formation; terminal membership; grammar derivation; then canonical [FORM-2] rendering.
Within one grammar decision, production definitions rank by their first appearance in this specification, and alternatives rank left to right as written.
Numbered rules rank by their first appearance in this specification.

Raw lexical scanning is quote-aware and reports the first defect at its cursor.
If the actual byte sequence beginning at the cursor does not begin one complete well-formed UTF-8 encoding of a Unicode scalar value, the first byte always cites [FORM-2] and spans that one byte, including when the cursor is inside a STRING candidate.
Outside a STRING candidate, a byte in `0x00..0x1f` other than LF, or byte `0x7f`, cites [FORM-2] and spans that byte.
An exact `//` or `/*` prefix outside a STRING candidate cites [FORM-4] and spans those two bytes.
A `'` or `@` not followed by `[a-z]` cites [FORM-3] and spans only the sigil.
Any other ASCII byte that cannot begin a specified token cites [FORM-1] and spans that byte.
Any valid non-ASCII scalar outside a STRING candidate cites [FORM-1] and spans its complete UTF-8 encoding.

After an opening `"`, `//` and `/*` are ordinary raw STRING bytes and never comment prefixes.
A final backslash cites [FORM-5] and spans only that backslash.
A backslash followed by an ASCII byte other than `\`, `"`, or `n` cites [FORM-5] and spans both bytes.
If the actual byte sequence beginning at a backslash's follower does not begin one complete well-formed UTF-8 encoding of a Unicode scalar value, that follower instead cites [FORM-2] and spans only its first byte; if the follower begins a valid non-ASCII scalar, [FORM-5] spans the backslash and that scalar's complete UTF-8 encoding.
A raw ASCII byte outside the permitted STRING interior set cites [FORM-5] and spans that byte.
At any other STRING cursor, if the actual byte sequence beginning there does not begin one complete well-formed UTF-8 encoding of a Unicode scalar value, [FORM-2] spans its first byte; a valid non-ASCII scalar instead cites [FORM-5] and spans its complete UTF-8 encoding.
If no unescaped closing quote occurs and no earlier defect applies, the unterminated STRING cites [FORM-5] and spans from its opening quote through end of source.
Terminal membership uses the complete context-free predicate set required by [GRAM-1]; a token with no matching predicate cites [FORM-3] or [FORM-5], whichever rule owns the rejected spelling.
Every lexical, terminal-membership, or grammar rejection uses `SourceBytes`; its coordinate is the exact interval above, the exact offending token interval, or the zero-width end-of-source interval defined above.

Every grammar production and external terminal predicate is owned by the numbered rule containing its unique definition.
A source-EBNF decision is a `|`, `?`, `*`, or the continuation decision of `+`.
Its stable identity is the zero-based ordinal of its production by first definition in this specification followed by the zero-based EBNF child-index path from that production's root.
Its arms retain source order; a consuming arm precedes an exit arm.
The strong-LL(2) analysis required by [GRAM-1] supplies every arm's `SELECT_2` rows.
Every predicate in a row retains its source-EBNF provenance and whether it came from inside that arm or from the arm's caller continuation.
Lookahead is padded to two positions with `SOURCE_END`.

Recognition selects an arm only when that arm has a full two-position row match.
Two matching arms are a [GRAM-1] specification defect, not a precedence rule.
Whenever no row matches, the diagnostic machine computes each arm's score: the greatest proper-prefix length, zero or one, by which any of that arm's rows accepts the actual two-position lookahead.
Let `m` be the greatest score at that frontier.
The failure boundary is the actual lookahead token at position `m`, or the zero-width end-of-source coordinate when that position is `SOURCE_END`.
The maximal-prefix rows are every row with score `m`.
The expected-terminal set is the distinct predicates at position `m` in those rows, ordered by their first terminal occurrence in the approved grammar; written terminals precede `SOURCE_END`.
A direct terminal mismatch is the same calculation with one row and has a singleton expected set.

At every no-row frontier, the following closed attribution rows are tested in order before diagnostic traversal descends.
The first matching row stops traversal.
A row retains the frontier expected-terminal set and coordinate unless that row names a replacement.

1.
If the boundary token is one member of four consecutive actual tokens `IDENT "." IDENT ("("|"<")`, that dotted call-or-targs spelling cites [FORM-3].
Its coordinate is the complete interval from the first IDENT through the second IDENT.
An allowed suffix would already be one maximal OPNAME token, while a field place cannot be called or given targs.
This bounded diagnostic window may include already recognized tokens, performs no operation-table or name lookup, consumes nothing, and does not enlarge recognition's two-token lookahead.
2.
If source-EBNF provenance reaches or would next enter an `atom` occurrence in `atom_list`, `fieldinit`, an `infix` operand, the subscript offset, or either endpoint of a `for_stmt`, and the two actual tokens at the start of that occurrence are `(IDENT, "(")`, `(IDENT, "<")`, `(OPNAME, "(")`, `(OPNAME, "<")`, `(TYPEID, "(")`, or `(TYPEID, "<")`, the rejection cites [GRAM-9]; in an infix-operand occurrence, a two-token start whose second token is an operator token — the forbidden nested-infix start — likewise cites [GRAM-9].
These are exactly the `call` and `construct` starts forbidden in an atom-only position; no name lookup participates.
Its coordinate is the complete interval from the first through the second token of that forbidden call or construct start.
3.
If the boundary token has the raw shape admitted by an expected external predicate before that predicate's explicit spelling restrictions, and fails only those restrictions, the rejection cites that predicate's owner.
This includes an exact fixed lowercase grammar word in an IDENT slot and a numeric-form token missing FORM-5 membership.
For the rest of this row, a boundary-name candidate is one of `IDENT`, `TYPEID`, `REGIONID`, `LABEL`, or `OPNAME` when the boundary token satisfies a different predicate in that five-member set.
A transparent mandatory-name path begins at a position-m predicate occurrence in one of the current frontier's maximal-prefix `SELECT_2` rows, using that occurrence's source-EBNF provenance; it never restarts at the failed decision's head.
The path ends at a boundary-name candidate which is its first nonnullable unconsumed terminal.
It may traverse a group; a sequence whose preceding children are completely matched; or a production reference whose expansion before that terminal contains no source `|` decision.
At a `?`, `*`, or `+` continuation it examines both the consuming direction and the exit/caller-continuation direction recursively.
The nullable decision is transparent only when no direction's first nonnullable unconsumed predicate accepts the boundary token.
Every direction that recursively reaches a boundary-name candidate contributes a path; a direction that instead reaches a different nonmatching predicate contributes none.
A path stops at any source `|` or at any nonnullable terminal before its candidate.
A name-slot mismatch exists only when at least one transparent path exists and every transparent path ends in the same name predicate.
It cites [FORM-3].
Thus traversal reaches a direct name, a name inside a consuming list arm, or a name after one or more skipped nullable prefixes such as `doc?`, but cannot tunnel through structural choices such as `item`, `stmt`, `expr`, `atom`, `callee`, `pbase`, `targ`, `law_arg`, `contract_define`, `requires_clause`, `ensures_clause`, `result_route`, or `atom_list | fieldinit_list`.
If several external predicates qualify under the first sentence, their owners rank by first rule occurrence in this specification.
4.
At the `program` `item*` or `item` entry, any `stmt*` or `stmt` entry, or any of the three repeated-entry frontiers inside `contract_block`, an IDENT-headed lookahead accepted by no complete construct row cites [FORM-1] as an unknown construct.
Its coordinate is the exact interval of that first IDENT token.
A lookahead that selects a defined construct is not covered by this clause.
5.
At `program`'s `item*`, after any complete item prefix, if the first actual token predicate matches no consuming `item` row, the token is an unexpected leftover, the expected-terminal set is replaced by only `SOURCE_END`, and the rejection cites the owner of `program`.

If no attribution row applies and exactly one arm has a score strictly greater than every other arm, diagnostic traversal descends into that arm only when every next expected predicate in that arm's maximal-prefix rows came from inside the arm rather than from its caller continuation.
Otherwise the current frontier is the stopping point.
A tie is never guessed.
Traversal through the selected arm follows the same source EBNF and repeats this procedure.
It cannot cross from a completed arm into its continuation, make a failed row valid, insert, delete, recover, or skip a token, or create a derivation or node.
It is used only after recognition has failed; reaching a successful end instead of a stopping point is a compiler-invariant failure.

At a stopping decision the total fallback cites the owner of the production containing that source-EBNF decision; a direct terminal mismatch cites the owner of its containing production.
A recursive-descent or table-driven implementation must report the result of this same source-EBNF diagnostic machine.

A source-local trivia gap is the complete interval between two adjacent terminal leaves after excluding those terminal bytes, between source start and the first terminal, or between the last terminal and source end.
It contains every intervening trivia item and may be zero-width; for a source with no terminal leaves, the whole source is its single gap.
The forest renderer defines the required bytes for each corresponding boundary.
A [FORM-2] mismatch is selected by the first byte offset at which the source and its complete forest rendering differ, treating the end of either byte sequence as a boundary.
Because terminal bytes have already passed lexical formation, terminal membership, and grammar derivation, that offset selects exactly one such actual-or-required gap.
Its coordinate is the complete actual gap interval, or the zero-width terminal boundary when required trivia is missing.
For a gap between two adjacent terminal leaves in the same top-level item, the location is `SourceNode` for their deepest common production-node ancestor in the finalized compilation-unit tree.
A source-leading, source-final, inter-item, or zero-item-source gap uses `SourceBytes`.
No renderer-authored owner, parser stack position, or implementation emission order participates in this choice.

An input-envelope failure, resource failure, target-layout failure [STOR-6], target-qualification failure [QUAL-1], compiler-invariant failure, unsupported compiler capability, backend failure, or external-tool failure is not a source-language rejection, cites no language rule, and carries no expected-terminal set.

After canonical FORM-2 succeeds for every source, semantic diagnostic selection first runs [FN-8]'s contract-presence judgment over every `contract_block`.
An empty or define-only block uses `SourceNode` at that complete `contract_block`; no declaration, route reservation, or use role inside such a rejected block is classified or counted.
Grammar already fixes each admitted block's definitions-before-requirements-before-postconditions structure and excludes every statement form, so no second structural-entry filter exists.
Only complete unit-wide FN-8 admission permits ordinary role classification, declaration inventory, and lexical resolution in their existing order.
Within an admitted routed `ensures_clause`, the route's leading lookup and [FN-9] route-admission subjudgment occur before lexical resolution of that clause expression; every unrelated block and event retains the ordinary global ordering.
Poison declarations and partial resolution are forbidden.
An early FN-8 rejection outranks every inventory or resolution rejection; inventory still outranks resolution even when the later-stage event has an earlier source coordinate.

A semantic role is owned by the lowest production node whose selected right-hand side directly contains the terminal that carries the role; a role reached only through a referenced child production is owned by that child.
A referenced child production means a child production node, not an external terminal predicate such as `literal`.
A semantic role may occupy a complete name terminal, a complete literal terminal, or the exact TYPEID suffix of a FORM-5 generic numeric literal `0_T` or `1_T`.
The suffix role's spelling excludes `_`, and its coordinate is exactly the suffix byte interval.
One token may carry more than one role: for example, a law argument `0_T` has one deferred law-argument role on the complete literal and one lexical generic-type use on `T`.
A struct TYPEID remains one declaration event producing two domain entries, not two events.

Within one owner node, distinct direct grammar-role carriers are ordered left to right by their complete carrier coordinates; distinct carriers with identical complete coordinates use the closed class order declaration, result-reservation, lexical-use, deferred-use.
The zero-based carrier index is `role_ordinal`.
`subtoken_ordinal` is zero for a role covering its complete carrier; embedded semantic name roles are numbered from one in byte order.
The only multi-role carrier is X09/U18, where the class tie does not reorder the embedded role: a law-argument `0_T` gives its complete deferred argument `(role_ordinal, 0)` and its embedded generic-type use `(role_ordinal, 1)`.
Every role has exactly one owner, class, role ordinal, and subtoken ordinal.
Every declaration event, FN-9 result-reservation event, lexical-use event, and deferred-use event has canonical key `(source_ordinal, byte_start, byte_end, NodePath, role_ordinal, subtoken_ordinal)`.
Numeric fields compare ascending.
NodePath compares lexicographically by production-child ordinal, with a proper prefix first.
Role and subtoken ordinals are consulted only after the complete path is equal.
For a complete IDENT, TYPEID, OPNAME, REGIONID, LABEL, or literal role, the coordinate is the complete token interval, including a sigil; only the generic-numeric suffix uses a subtoken coordinate.
The event's `SourceNode` names its owner production.
Traversal order, allocation identity, map order, logical path, and inferred type never participate.

Declaration inventory and FN-9 result reservation create candidates under this closed rank:

1. a FORM-3 reserved-name violation defined by OP-1's derived set;
2. an OWN-3 repeated REGIONID declaration within one function declaration or contract-member signature, parameters included;
3. a GRAM-10 match-binder freshness violation;
4. a TYPE-6 collision with PRE-1;
5. a TYPE-6 collision with an admitted system declaration [SYS-1];
6. a TYPE-6 compilation-root duplicate or same-lexical-scope redeclaration; and
7. a TYPE-6 nested declaration shadowing a live declaration.

Each declaration or result-reservation event forms an inventory candidate only for an applicable rank above; an event for which no rank applies forms no candidate.
The stage selects the minimum canonical event key among events with at least one candidate and then the first applicable rank at that event.
A FORM-3 reservation payload is `(spelling, carrier_role, reserved_class, inventory_ordinal)`.
Its `spelling` is the complete declaration or result-candidate spelling.
A REGIONID payload uses its unsigiled IDENT-shaped interior while the rejection coordinate retains the complete sigiled token.
Its closed carrier roles are function, named-const, parameter, contract-definition, let, for-binder, match-binder, result-binding, route-result, field, variant-field, region-parameter, and local-region.
`reserved_class` is dotless-operation or mode-word.
A dotless-operation ordinal is the zero-based first occurrence among distinct operation-family spellings, scanning OP-1 rows top to bottom and each `op` cell left to right and skipping every later occurrence of the same spelling; both `cvt` rows therefore name one family and one ordinal.
A mode-word ordinal is the zero-based FORM-3 alternative order `wrap`, `defined`, `checked`, `sat`, `strict`.
Those two reserved sets are disjoint in this version.
An OWN-3 repeated-region payload is `(spelling, conflicting_region_origin)` and points to the later region declaration; OWN-3 precedes GRAM-10 in the rank even though no grammar carrier can be both a region declaration and a match binder.
For the GRAM-10 violation defined by TYPE-6, the payload is `(binder_spelling, paired_field_spelling, optional_earlier_binder_origin, ordered_arm_entry_live_lexical_ident_origins)`.
Earlier binders and arm-entry origins are ordered by declaration-event key.
That binder does not also create a TYPE-6 duplicate or shadow candidate.

A TYPE-6 collision payload is `(spelling, ordered_nonempty_conflicts)`.
Conflict domains use the fixed order lexical-IDENT, nominal-type, constructor, contract, REGIONID, LABEL.
Each conflict contains its domain, declaration class, and `conflicting_origin`; conflicts within one domain use PRE-1 declaration ordinal first, then system declaration ordinal, then source declaration-event key.
A source origin is `(NodePath, SourceCoordinate, role_ordinal, subtoken_ordinal)`; a PRE-1 origin is `(PRE-1, declaration_ordinal)`, where `declaration_ordinal` is the zero-based twenty-four-record preorder fixed by TYPE-6; a system origin is `(System, system_declaration_ordinal)`, where `system_declaration_ordinal` is the zero-based preorder fixed by [SYS-2] and appears in every unit [SYS-3].
A struct event may report both nominal-type and constructor conflicts in that order.
Rank 4 reports only PRE-1 conflicts when the same event also conflicts with an admitted system declaration or with source.
Rank 5 reports only system conflicts when the same event also conflicts with source, and is selected for a colliding declaration event at the compilation root and in a nested scope alike, ahead of ranks 6 and 7 at that event.
A PRE-1 collision and a system collision each point to the source declaration.
Rank 6 points to the later source declaration event.
Rank 7 points to the nested declaration, including one shadowing a source-later but whole-unit-visible function.
Every declaration-inventory rejection uses `SourceNode` at the declaration role and has no expected-terminal set.
An FN-9 result-datum reservation instead uses `SourceNode` at the owning `result_binding` or `fieldbind`, a coordinate equal to the candidate IDENT token, and the FORM-3 payload above; it creates no TYPE-6 runtime declaration or duplicate event.

If inventory succeeds, every lexical use admitted by TYPE-6 or OP-1 creates one lexical-use event.
The generic-numeric suffix admits a live generic TYPEID parameter; FN-3 and FORM-5, not lexical resolution, later require its numeric bound.
Lexical resolution fixes only the declaration or operation-family target.

The closed declaration-class order is function, named-const, const-generic, value, generic-type, nominal-type, struct-constructor, enum-variant, contract, region, label, operation-family.
TYPE-6 and OP-1 fix each lexical role's ordered admissible subset.
A use's exact-spelling candidate universe contains all compilation-root entries in its grammar-selected domain and, for non-root declarations, only entries belonging to its declaration-owner chain.
All sibling or expired lexical scopes within the same `fn_decl` owner participate so that an out-of-scope same-function declaration can be distinguished from absence.
A contract-member signature admits declarations of that signature and its enclosing contract ancestry but not declarations owned only by a sibling member signature.
A struct, enum, contract, or function generic belongs only to that declaration and its descendants.
No local, generic, parameter, region, or label owned solely by an unrelated top-level declaration or function participates.
PRE-1 owner-local type parameters and fields never participate in source lookup.
LABEL uses instead follow the separate current-function rule below.

For one lexical-use event the closed lookup rank is:

1. the candidate universe has at least one declaration in an admissible class but its admissible visible subset is empty; cite the role-attribution table below and carry every invisible admissible origin in declaration-event order;
2. for LABEL only, the current function has at least one exact-spelling label but none declares a loop lexically enclosing the `break`; cite TYPE-6 and carry every such current-function label origin in declaration-event order; and
3. the visible admissible subset is empty and neither rank 1 nor rank 2 applies; cite the role-attribution table below.

| lexical-use role | rule cited by rank 1 or rank 3 |
|---|---|
| `type` TYPEID | TYPE-5 |
| contract bound or `conform_decl` contract TYPEID | FN-3 |
| `construct` constructor TYPEID, enum-variant-only `arm` TYPEID, or `result_route` TYPEID | TYPE-6 |
| REGIONID use | OWN-3 |
| LABEL use | TYPE-6 |
| `const` IDENT | CONST-1 |
| `cvalue` IDENT | CONST-2 |
| `pbase` IDENT | TYPE-5 |
| IDENT or OPNAME `callee` | OP-1 |
| `fn_bind` right IDENT | FN-3 |
| FORM-5 generic-numeric TYPEID suffix | FORM-5 |

A successful non-LABEL lookup has exactly one visible admissible target; a successful LABEL lookup has exactly one enclosing target.
A rank-1 payload is `(spelling, lexical_use_role, ordered_admissible_classes, ordered_nonempty_invisible_origins)`.
A rank-2 payload is `(spelling, lexical_use_role, ordered_nonempty_label_origins)`.
A rank-3 payload is `(spelling, lexical_use_role, ordered_admissible_classes, ordered_available_classes)`, where available classes are visible exact-spelling entries in that use's candidate universe, listed once in the closed class order and possibly empty.
Complete IDENT, TYPEID, OPNAME, REGIONID, and LABEL use spellings include any sigil; only the generic-numeric suffix spelling is bare `T`.
This is declaration-kind resolution, not type checking.
Across use events the minimum event key wins.
Every resolution rejection uses `SourceNode` at the use role and has no expected-terminal set.

The dependent-declaration carriers are exactly the `field` and `vfield` declarations and the member declaration of `fn_sig`.
Each is a declaration-class carrier that produces one dependent-declaration record and one declaration event for later typed owner/member checking, but none enters a resolver lookup inventory.
The two field carriers participate in FORM-3's reservation inventory; the contract-member carrier does not.
The deferred-use carriers are exactly the `law` name and each complete law argument, the left IDENT of `fn_bind`, the first IDENT of an arm `fieldbind`, each `fieldinit` IDENT, and each `psuffix` IDENT.
Each produces one deferred-use record for later typed owner/member checking.
The `result_binding` IDENT and the second IDENT of a `result_route` fieldbind are FN-9-owned result-datum carriers.
The header candidate produces one reservation event with `role_ordinal` zero in its `result_binding`.
The route candidate produces one reservation event with `role_ordinal` one in its `fieldbind`, after the first IDENT's FN-9 field-owner carrier.
A result-datum reservation uses the canonical key above but is not a runtime declaration, lexical use, dependent declaration, or deferred use.
Candidates participate in FORM-3 reservation checking but provide no pbase target before FN-9 admission; they enter no owner/member lookup and no TYPE-6 duplicate or shadow inventory.
The header candidate is available only to an admitted unrouted ensures clause; after the leading route TYPEID's ordinary constructor lookup, FN-9 admits the route payload candidate only within that one routed clause.
No result datum is visible in a contract definition, requirement, function body, or different ensures clause.
The table-checked carriers are exactly the `program_kind` IDENT and both IDENTs of an `input_label`.
Each produces one record for later [FN-7] table checking; none produces a declaration, lexical-use, dependent-declaration, or deferred-use record, none enters or queries a lexical name domain, and none participates in FORM-3's reservation inventory.
The claim-name carrier is exactly the IDENT of a `claim_stmt` [CLM-1].
It produces one record for CLM-1's per-function uniqueness judgment; it produces no declaration, lexical-use, dependent-declaration, deferred-use, or table-checked record, enters and queries no lexical name domain, and does not participate in FORM-3's reservation inventory.
The lexical generic suffix inside a deferred literal law argument additionally receives its ordinary lexical-use record; this X09/U18 pair is the only same-token overlap and produces two distinct role records.
In an `arm` or `result_route`, the leading TYPEID first resolves globally to an enum variant.
Later typed checking compares that variant's owner with the scrutinee enum for an arm; a foreign arm variant cites TYPE-6.
FN-9 separately requires the route's successfully resolved variant and owner to be exactly PRE-1 `Result.Ok`.
The resolver does not otherwise accept or reject a dependent role's owner/member relation.

A missing whole-unit requirement is not fabricated as an inventory or lookup event.
Missing `main` remains an FN-7 rejection at `BundleRoot`.
Duplicate `main` names are the later-source TYPE-6 duplicate; one unique but wrong-signature `main` is a later FN-7 rejection at its source declaration.
Missing or duplicate contract members, field labels, conform bindings, and law roles remain typed-dependent rejections.

Apart from FN-9's explicitly interleaved selector-admission subjudgment above, after complete lexical resolution succeeds, source semantic checking covers the complete closed unit and precedes every target-dependent check or lowering action.
A source-semantic rejection cites one numbered rule whose rejection premise the checker has established.
A required child or referenced-declaration premise that has not obtained its judgment is never replaced by a guessed parent rejection.
An unsupported compiler capability, unavailable semantic judgment, semantic-checker invariant failure, or resource failure establishes no source violation and remains a non-language failure under this rule.

A semantic rejection uses the exact location stated by its cited numbered rule or by a more specific row below.
When neither states one, it uses `SourceNode` at one existing canonical node that directly supplied an immediate source premise of the failed judgment, with a `SourceCoordinate` equal to that node's complete checked half-open source extent.
Which such participating node is selected is implementation-defined.
A whole-unit requirement with no offending source declaration uses `BundleRoot` exactly as already defined.
Post-resolution semantic rejection never uses `SourceBytes`, fabricates a node, or carries an expected-terminal set.

Two or more simultaneously established post-resolution semantic rejections whose immediate offending source premise is the same use of the same canonical node are one rejection event, and that event cites the established rule whose definition appears first in this specification; a rule whose own text states that it forms no candidate in that situation [FN-1] is not among the established rules, and the event's location follows the cited rule.
The order among rejection events at distinct nodes is implementation-defined.
One compiler executable invoked on the identical bound unit with identical options and sufficient resources must use one stable deterministic traversal and produce the same first rejection event.
Selection may not depend on allocation identity, unordered-container iteration, worker scheduling, or backend traversal.
Different conforming implementations may report different first established rejection events at distinct nodes; the same-node citation above is fixed for every conforming implementation, and neither freedom changes the accepted-program set or checked-program authority.

Semantic success is failure-atomic at its publication boundary.
Private scratch judgments may be constructed in any deterministic order, but no checked program, lowering input, optimization fact, partially accepted declaration, or other semantic authority is published unless every applicable source-side numbered-rule judgment through complete-unit semantic checking succeeds.
Target-layout checking under [STOR-6] occurs only after that publication and produces no source rejection, rule citation, or semantic authority.
Target-stage failures are outside source-language rejection ordering.
Backend, linker, runtime-environment, and external-tool failures remain non-language failures [DIAG-1].

After complete lexical resolution succeeds, FN-3 validates the complete source-ordered contract table before any FN-4 discharge.
A source contract carrying a `generics` child rejects with `SourceNode` at that child and its complete extent; unresolved names inside that child instead retain their earlier resolver-owned rejection because FN-3 is not reached.
A repeated contract member rejects at the later `fn_sig` node and its complete extent.
A nonconcrete conformance subject rejects at the selected `type` child and its complete extent.
A successfully resolved prelude contract reference rejects with `SourceNode` at the `conform_decl` and a coordinate equal to that contract TYPEID token.
Contract arguments reject at the `targs` child and its complete extent.
A duplicate exact conformance key rejects at the later `conform_decl` and its complete extent.
An unresolved name inside the subject or contract arguments retains its earlier resolver-owned rejection.
An unknown, repeated, extra, out-of-order, function-`generics`, contract-bearing, or signature-incompatible binding rejects at the offending `fn_bind` and its complete extent.
A missing binding rejects at the `conform_decl` node with the coordinate of its closing `}` token.
A generic type-parameter bound that successfully resolves to a source contract rejects with `SourceNode` at its owning `gparam` and a coordinate equal to that bound TYPEID token; an unresolved bound retains its earlier resolver-owned FN-3 rejection at the same lexical-use role.
The post-resolution order among independent FN-3 candidates remains the implementation-defined deterministic order above.
No FN-4 law discharge runs for an invalid conformance, and an FN-3 failure publishes neither partial contract metadata nor a partial binding vector.

For a call to a callee class that carries written arguments — a user-generic `fn` [FN-2], a system operation's region arguments [SYS-2], or a retained-argument table operation [TYPE-5] — a missing, wrong-kind, wrong-count, or wrong-domain argument, or a missing operand, uses `SourceNode` at the `call` node and that node's complete source extent.
For an operation spelled infix, a wrong operand domain or a missing operand uses `SourceNode` at the `infix` node and its complete extent.
An extra operand or every wrong exact operand type other than the TYPE-7 implicit-read case uses `SourceNode` at the first offending `atom` node in source order and that atom's complete extent — for [OP-2]'s operand-agreement error, the second operand atom.
The cited rule is the rule selected by the callee's class: [FN-2] for a user-generic call, [SYS-2] for a system operation's region arguments, and, for a table operation, the rule [OP-2] selects — OP-1 or TYPE-5.
The TYPE-7 case follows that rule and the general participating-node location above.
A table-operation call written with a `fieldinit_list` instead of positional operands cites GRAM-11 using `SourceNode` at the `call` node and its complete extent.
A result mismatch is located and attributed only by the consuming construct as stated in OP-2.

An [FN-8] ordinary-call requirement judgment begins only after every earlier callee, concrete-instantiation, argument, type, borrow-feasibility, and actual-expression-obligation judgment named by FN-8 succeeds.
An unproved or refuted instantiated goal is one hard rejection citing FN-8 with `SourceNode` at that existing `call` node and `SourceCoordinate` equal to the call node's complete checked half-open source extent.
Its deterministic payload contains the concrete callee instance, the failing `requires_clause` NodePath, the complete instantiated typed goal, and exactly one disposition, `unproved` or `refuted`.
The required restructuring is `establish the complete callee requirement with one dominating branch, or with one CLM-2-admissible residual candidate whose theorem is separately validated before owner approval, before the call`.
When the payload contains an ephemeral actual-value datum, it additionally renders that datum as `argument #N pre-transfer value`, with N the zero-based argument ordinal, and replaces the restructuring with `bind that argument or referent value with one preceding ordinary let, establish the complete requirement over that binding, and pass the binding, borrowing it when the parameter mode requires a borrow`.
A concrete generic instance that changes a substituted type, const, or datum changes the payload goal and is judged independently.
This rejection is never replaced with a runtime fallback or reported at the callee declaration.

An [FN-9] result-datum admission subjudgment begins only after [FN-8] contract admission, FORM-3 result reservation, the route's ordinary leading-variant lookup when present, and concrete [FN-2] signature substitution.
Admission through freshness precedes lexical resolution or semantic checking of the owning `ensures_clause` expression; the remaining clause, selected-return, and proof judgments begin only after that expression resolves and the surrounding function's ordinary semantic judgments required by the failed premise succeed.
For an unrouted clause, test in this fixed order: written result mode/type and fragment class; header result-candidate freshness against every declaration live in the clause.
For a routed clause, test in this fixed order: written whole-result mode/type and `Result` class; resolved variant owner and exact `Ok` identity; the written field against the variant's sole declaration-order field; route-candidate freshness against that field, the header result candidate, and every declaration live in the clause.
A result, class, owner, variant, or missing-field failure uses `SourceNode` at the complete `ensures_clause` or its `result_route` when present.
An extra, misspelled, or out-of-order field uses `SourceNode` at the complete `fieldbind`.
A candidate equal to its paired field or another live candidate or declaration uses `SourceNode` at its owning `result_binding` or `fieldbind`, with coordinate equal to the candidate IDENT token.
Those are FN-9 events, not GRAM-10 or TYPE-6 duplicates.
An unresolved leading route TYPEID remains the earlier TYPE-6 lexical-use rejection and forms no FN-9 candidate.

After result-datum admission, an inadmissible clause computation uses its offending expression node; a condition that is not one exact output-bearing L0 relation uses that `ensures_clause`'s `expr`.
A concrete instance with no selected normal exit uses the `ensures_clause` and residual exactly `no selected normal exit`.
An unsupported selected return expression, unavailable entry image, or complete relation failure uses `SourceNode` at that existing `return_stmt` and its complete checked extent.
The deterministic relation payload is `(concrete function instance, postcondition occurrence, route identity or unrouted, instantiated normalized relation, disposition)`, with disposition exactly `unproved` or `refuted`; entry-image unavailability fixes `unproved`.
Instances use DIAG-1's stable concrete-instance order, selected returns use NodePath order, and the first complete-view failure wins.
U and B are then computed in that order as metadata and form no rejection event.
A later PRV rejection is owned only by PRV-2 or PRV-3; it does not relocate to the FN-9 declaration or publication event.
No FN-9 failure fabricates an executable epilogue, runtime fallback, optimizer assumption, pending named-outcome fact, or caller-side rejection.
An excluded caller route, including a named or pending outcome, is not itself a rejection: it establishes no S12 fact or metadata, and any later query that needed that absent relation is diagnosed only at that later node by its ordinary owning rule.

Claim diagnostics use this fixed semantic schedule.
FN-1 first rejects every structurally unreachable statement; only a claim occurrence with a structurally reachable normal entry enters the CLM-1/CLM-2 schedule below.
CLM-1 predicate type, proof-predicate shape, per-function name uniqueness, and five-field structure are checked first.
CLM-1 next performs fact-free D/S/F and `Contrib(P)` formation; an ambiguous or unsupported origin, normalization, support, component negation, reconstruction, or materialization reports its existing formation error before locality, and no component ordinal is fabricated when formation fails.
For every successfully formed source schema and concrete instance, CLM-1 then checks component authority in source-occurrence then stable-instance order; source-schema reports precede reports from the same occurrence's concrete instances, and the least non-local component ordinal wins.
Only the resulting admitted inventory records contradiction-first D/S/F exact lifecycle, component lifecycle, consistency, and S-to-D reconstruction; the first invalid claim in source occurrence then stable instance order rejects before any counterfactual run.
Complete OP-2/OP-4/OP-9/SYS-8, FN-8, FN-9, PRV-2, and PRV-3 judgments then select their ordinary errors.
Only an otherwise-successful unit freezes Eligible and runs CLM-2 component and whole-occurrence residuality; the first non-residual occurrence owns that rejection, component failures precede a whole-occurrence failure, and component failures use the least component ordinal.
This schedule prevents an invalid candidate from supplying another candidate's baseline and prevents a premature unused-claim error from hiding an ordinary proof or provenance defect.
All claim source errors cite CLM-1 for predicate, justification, canonical, or locality admission and CLM-2 for vacuous, redundant, refuted, overlapping, inconsistent, reconstruction, or non-residual lifecycle, using `SourceNode` at the `claim_stmt` and its complete extent unless CLM-1 already selects its `expr`.
Their payload retains name, exact predicate, classification, and the deterministic concrete instance, component when applicable, and terminal-root witness when one exists; a whole-occurrence failure has no component ordinal.
One locality payload instead retains the claim name and NodePath, least failing component ordinal, earliest boundary-call NodePath, boundary kind, the first source-ordered support carrier that observes that same earliest witness with its canonical source spelling, and the callee's stable identity: source declaration origin and source name for a user call, or [SYS-2]'s `system_declaration_ordinal` and operation spelling for a system call.
It never publishes a scratch `FunctionId`, `NominalId`, dense instance number, `$instance$N` spelling, or traversal-order identity.
For a user result its restructuring is `publish the required cross-function relation as an exact verified ensures clause on the callee and remove this caller claim`; for a system result it is `use the system operation's specified fact or typed outcome, or branch on the returned value; do not claim an unstated system-result property`.
The both-sign case, an unavailable required source-schema judgment, or an inconsistent counterfactual result is a compiler failure or explicit unsupported capability rather than a guessed source rejection.

The [CLM-3] stage begins only after every ordinary source, provenance, and CLM-2 residual judgment has succeeded.
Validate marked roots in the stable concrete-instance order.
If the root SCC has a direct claim, cite CLM-3 at the first claim in stable member-instance then claim-NodePath order and retain `(strict root, concrete claim owner, claim NodePath, name, predicate, justification, retained disposition)`.
Otherwise cite CLM-3 at the first call in stable caller-instance then call-NodePath order within the root SCC whose strictly outgoing callee component has a nonempty `MayClaims` set, retaining `(strict root, concrete caller, call NodePath, concrete callee, least downstream claim identity)`.
A component summary is silent: a claim reached only below that boundary is reported at the importing call, not duplicated at its declaration.
If the root shares an SCC with that claim, the claim is direct and the claim node wins.
At one importing call, CLM-3 is selected before a strict FN-8 U failure; a non-claim strict FN-8 failure is emitted only at its actual call, and no caller-side summary failure is fabricated.
All these candidates use existing source nodes and the complete extents stated by their owning rules, obey semantic failure atomicity, and publish no checked program or derived ClaimLedger.

A mechanical fix or restructuring is included exactly where the owning rule requires one.
Every published static diagnostic is deterministic for one compiler executable under the conditions above.
Cross-implementation byte identity is required only where this specification explicitly fixes both selection and encoding; the runtime trap record [DIAG-3] is such a case.

[DIAG-2] Successful semantic checking produces one private checked-program value bound to the exact canonical compilation unit.
It is the only input that may grant lowering authority.

The checked program explicitly represents every source operation and every compiler-derived operation required for execution, including drops, arena releases, monomorphized instances, propagation edges, retained runtime checks, every direct slice value's finite origin set, every `own slice` result's FN-1 formal return-origin ceiling and call-site substitution, and one abstract target-domain representability obligation at every runtime-sized allocation and element-address operation governed by [STOR-6].
It additionally retains every [FN-8] GoalTemplate, its requirement occurrence `(concrete callee instance, requires_clause NodePath)`, every concrete call substitution and independently discharged-goal derivation, every S4 body-entry axiom, and each inhabited or `Uninhabited { contradiction: DerivationId }` body disposition.
It retains every proof-required integer-domain, allocation-fit, subscript-bounds, and system-range obligation occurrence, its complete/U/B dispositions where applicable, and the exact derivation authorizing its accepted source node.
It also retains the converged [PRV-1] component summaries, every [PRV-2] result, write, direct-demand, and bridge column, the complete/U/B outcomes and successful no-event disposition of each accepted call argument, every [PRV-3] local-leaf disposition, and the post-convergence deterministic predecessor choices.
A rejecting PRV-2 target set or PRV-3 witness exists only in failure-atomic diagnostic scratch and is never published as checked-program or lowering authority.
Target lowering must discharge each target-domain obligation from the selected target plus already-checked layout, allocation, and bounds facts, or materialize its exact non-continuing guard before the governed allocation or address operation.
Every writer-reachable source-language runtime check is one [CLM-1] claim with disposition `retained`.
No accepted proof-required operation carries an implicit runtime check or elimination disposition: a subscript, exact integer operation, buffer allocation, or system range is `discharged` at its owning source node, and the checked program retains its exact [ENT-4] or [ENT-6] derivation there.
Every accepted [CLM-1] claim is `retained`; the checked program retains its source occurrence and concrete-instance identities, name, exact D, S, and F predicate images, five parsed justification fields, ordered structured `Contrib(P)`, each component S3 source derivation, the retained S reconstruction and D materialization derivations, every successful component and whole counterfactual witness, the closed terminal-root inventory, and each witness's non-contradictory and non-explosive ancestry disposition.
A concrete terminal-root identity uses the owning function instance plus the operation NodePath/family/conjunct, the call NodePath/callee/requirement NodePath, or the complete-postcondition block/relation ordinal; display symbols are never identity.
A generic source occurrence additionally retains one source-stable schema report owned by its declaration, with rendered D, S, and F predicate images, ordered contribution descriptions, local-authority success, S-to-D reconstruction success, stable counterfactual witness summaries, and ordered inhabited concrete-instance report links.
The schema report contains no monomorphized display symbol or symbolic-scratch `FunctionId`, `GoalId`, `TermId`, or `DerivationId`; only concrete reports may retain finalized function-local proof identifiers.
Malformed, non-local, redundant, refuted, vacuous, overlapping, inconsistent, reconstruction-failed, or non-residual candidates publish no claim metadata or checked program.
A `requires_clause` is represented only by its GoalTemplate, call-site derivations, and S4 source; an `ensures_clause` only by its verified RelationTemplate, selected-exit judgments, and derivations.
Neither contract clause has executable checked-program form.
In facts-off compilation every claim remains `retained`, and all [ENT-1] source-acceptance and call-goal judgments are identical in facts-on and facts-off compilation.
Neither a discharged call goal nor S4 authorizes `llvm.assume`, an optimizer fact, or a second lowering path.
STOR-6 target-domain obligations instead follow the target-stage discharge-or-guard judgment above identically in facts-on and facts-off compilation; an optional optimizer fact supplies no target-layout discharge.

The complete, U, and B analyses of one concrete function extend one function-local derivation DAG and one event stream; a view tag distinguishes their nodes, and every S3 event additionally retains its source claim and component ordinal.
This is the same authority that already proves accepted obligations, discharged call goals, and S11 facts.
Every parent precedes its child, every retained node is reachable from a required root, and finalization performs one reachability traversal and one identity remap.
An implementation may choose its private Rust layout, but it may not build a postcondition-only proof graph, merge separately authoritative view ledgers, reconstruct a missing root after publication, or consult another checker.
CLM-2's specified pre-publication `Full-minus` analyses are fresh runs of this same flow over failure-atomic scratch and are the sole permitted acceptance-bearing rewalks; each repeats entailment and verifies the provenance-invariance condition below, then publishes only its stable witness summary after every claim succeeds.
A masked witness records its component-or-whole mask, stable terminal-root identity, and exactly one masked disposition: missing or ordinarily undischarged operation root, refuted or unproved call root, or failed complete postcondition.
Masked derivation identifiers are never published; a masked contradictory, all-derivable, or explosive result is an inconsistent counterfactual and therefore a compiler failure rather than a residual witness.
A callee summary is referenced by checked-program-private `(concrete callee instance, postcondition occurrence, view)` identity; a caller never imports a callee's local node identity.

Every new S7 fact is retained even when no later query consumes it.
`BitAndBound` roots the exact direct `iand` result relation at its binding and carries the selected unsigned operation row, result binding, operand ordinal, admitted operand term or constant, and source event.
`ShiftOneNonzero` roots the exact direct `ishl.wrap` result disequality against the mathematical-zero endpoint Z and carries the selected unsigned row, result binding, count atom, and the checked mathematical-one constant identity.
A signed row, non-direct result, nonterm operand, or non-one source forms no such root.
These are ordinary source roots in the same DAG, not trusted optimizer facts.

For every concrete FN-9 declaration, `PostconditionExit` roots each discharged selected-return relation in each view on the exact local [ENT-4] derivation after result substitution and before return transfer or cleanup.
It also retains the view-independent entry-image-stability disposition and ordered invalidating event when unavailable; successful absence of an invalidating event is validation metadata, not a fabricated positive parent.
`PostconditionAggregate` has the nonempty selected-exit roots as parents in return-NodePath order.
A non-discharged view retains its ordered dispositions and residual but no success aggregate root.
Component summaries become referenceable together only after the SCC schedule validates that every summary reference points strictly from a caller component to an earlier callee component.

Every S12 fact actually established in accepted semantic flow is a required caller-local root even when no later query consumes it.
`PostconditionCall` carries q, the caller proof view, the checked aggregate summary reference selected as C in the complete view, otherwise B when Bq holds, and otherwise U only under the exact Uq-and-Gv branch, exact per-formal pre-transfer substitution, A0's complete actual-obligation and FN-8 goal roots in every view, and the ordered transfer/consume/borrow/effect/kill event prefix.
A Bq branch additionally carries the B aggregate parent and no same-view Gv parent; a Uq branch additionally carries the U aggregate and every exact same-view Gv actual-obligation and FN-8 goal parent.
`PostconditionDirectResult` adds the fresh ordinary-let binding substitution.
`PostconditionDirectMatch` adds the direct-call scrutinee, selected `Ok` variant and `value` field identities, and payload substitution at arm entry.
`PostconditionDirectReceiver` adds the direct-set target kill and result-only post-write substitution.
`PostconditionSelectedReceiver` adds the selected arm's immediate payload read, target kill, and result-payload-only outer substitution.
A named or pending outcome, false `M(c,q)`, unavailable view, rejected call, killed support, or excluded receiver creates no fact root or pending metadata.

For bounded `value_if` delivery, `PostconditionGive` records one eligible reaching edge, the already evaluated source value and relation root, then the forward `d ↦ x` substitution, then that edge's ordinary scope and event kills applied to every other support in that order.
`PostconditionDeliveryJoin` orders all non-contradictory reaching delivery images by edge NodePath and applies exactly the ordinary [ENT-5] L0 delivery join.
Its parents therefore need not state byte-identical relations; an `x < 8` image and an `x < 128` image may parent the joined `x < 128` root.
Contradictory inputs use the existing contradiction root and are neutral when a non-contradictory input reaches.
Missing edge evidence, a `value_match`, or no common joined relation creates no delivery root.
Kill events never become invented positive evidence.

Candidate S12 and delivery nodes live only in failure-atomic semantic scratch while FN-9's one PRV validation batch and [CLM-3] run.
On any PRV-2 or PRV-3 event the whole candidate root set is discarded with the unpublished checked program.
A PRV no-event verdict leaves the identical candidate S12 and delivery root set unfinalized; CLM-3 adds none of those roots.
After total strict success, that root set and the strict metadata below enter the sole reachability walk and identity remap, with no candidate S12 or delivery root added, removed, or rebuilt between the PRV verdict and finalization.
A CLM-3 or strict FN-8 event instead discards them.
This preserves A0/A atomicity and makes the derivation DAG evidence for the accepted semantic flow rather than a second acceptance path.

For [CLM-3], the successful checked program additionally retains each declaration marker, each concrete strict root, its outgoing SCC membership, each component `DirectClaims` and `MayClaims` set, every source-ordered call occurrence used by the summary, the successful component and root disposition, the marked command-entry disposition, and the exact existing U derivation root for every demanded protected obligation and call requirement.
The claim and call graph is formed in private semantic scratch from checked claim occurrences and the ordinary concrete call inventory; the checked-program `ClaimLedger` is derived only after success and is never read back as acceptance authority.
Strict roots extend the same function-local view-tagged derivation DAG and event stream; they are registered before the sole reachability walk and identity remap, import no foreign `DerivationId`, and create no copied graph or second semantic flow.
On any CLM-3 or strict FN-8 event, all strict metadata, candidate S12 and delivery roots, the prospective checked program, and every checked-program-derived tool projection are discarded together.
On success, lowering consumes the same checked functions and executable operations as the unmarked unit and reads none of this metadata.

For every `for_stmt`, the checked program additionally represents its source label and binder, the two source endpoint atoms in evaluation order, the two immutable compiler-owned captures with their identities, binder initialization, the pure header comparison, both header edges, the exact normal-body cleanup and update order, every no-update exit edge, the distinct header and continuation carried-binding sets, every hidden scope kill, and the complete [ENT-4] derivation of each S11 fact.
These are checked semantic operations and facts, not a source desugaring or an optimizer reconstruction.

The checked program also retains one complete source-ordered contract table, one validated conformance record per source `conform_decl`, each conformance's member-order binding vector, and every FN-4 base law derivation.
Those records are semantic evidence, not executable operations.
Ordinary lowering consumes the same checked functions and operations it would consume if those metadata tables were empty; it emits no contract or conformance object and obtains no dispatch target, check elimination, reassociation, or other optimization consequence from either a written law or a base derivation.
Any future optional law-fact family remains subject to FN-4's independent rederivation boundary and facts-off identity.

The checked-program representation is private compiler state.
Its Rust layout, allocation strategy, dense identities, instruction grouping, and internal ordering where this specification defines no semantic order are implementation-defined.
This specification defines no checked-program byte encoding, portable identity, cache format, deserialization authority, artifact hash, or replay step.
Emitting diagnostic, debug, or experimental files from it grants no authority to reload those files for lowering.

[DIAG-3] The sole mandatory language runtime report is one trap record for a failing executed [CLM-1] claim.
Its exact UTF-8 bytes are:

```text
{"rule_id":RULE,"message":MESSAGE,"function":FUNCTION,"node_path":[COMPONENTS]}
```

The displayed line excludes its Markdown line ending.
The record bytes are the displayed JSON object followed by exactly one byte `0x0A`; no `0x5C 0x6E` suffix is present.

`RULE`, `MESSAGE`, and `FUNCTION` are JSON strings.
Fields occur in exactly the written order with no extra whitespace or fields.
`COMPONENTS` is the source node's zero-based `NodePath`, written as comma-separated unsigned decimal integers with no leading zeros; the empty path is `[]`.

`rule_id` is the JSON string `CLM-1`.
`function` is the exact enclosing source function IDENT.
`node_path` identifies that `claim_stmt` production.
`message` is the claim's exact IDENT spelling; the justification STRING is compile-time data and does not appear in the record.
No contract clause, exact operation, allocation, subscript, system range, typed expected failure, resource failure, target-qualification failure, or compiler/runtime invariant failure produces a DIAG-3 record.

JSON string encoding is canonical for the complete character set defined here: `"` becomes `\"`, `\` becomes `\\`, LF becomes `\n`, and every other permitted ASCII byte is emitted unchanged.
A final single LF terminates the record.

Identical bound source bytes reaching the same failing claim site produce byte-identical report bytes in every conforming implementation.
Dynamic call-stack attribution, artifact identity, successful-check reports, lifetime reports, check-density reports, and optimizer-development reports are not normative outputs.
An implementation may provide additional developer output only on a separately selected channel that cannot alter, prefix, suffix, or replace the mandatory trap record.

## 13. Execution overlap

[CAP-1] The kernel defines no writer-visible capability category and no system-specific concurrency permission. `own`, `&`, `&uniq`, place overlap, and the ordinary effect row are the complete authority and interference vocabulary available to [PAR-1] and [PAR-2].
This version defines no thread construct. A later thread construct must derive transfer and sharing permission from these same ownership rules and the represented type; it may not add hidden shared mutation to an opaque system type. Data-race impossibility is D1 law; general race conditions are out of scope (C004 amended scope).

[PAR-1] An implementation may execute two statements of one block with overlapping execution only when the permission this rule defines holds for that ordered pair.
Permission holds for the ordered pair (s1, s2), where s1 precedes s2 in one block, exactly when all of the following hold.
Each of s1 and s2 is a `let_stmt` whose selected `ordinary_let_rhs` is one call of a declared function [FN-1] or one system operation [SYS-2]; a recursive or mutually recursive user callee is admitted on the same terms as any other.
No argument of s2 reads a binding s1 defines.
The two calls have disjoint footprints under [OWN-7]: one call's written footprint is the places its callee row's `writes` paths reach through its actual arguments under the [EFF-2] call-boundary projection, together with the places its consumed `own` arguments name and the caller region each `allocates(arena 'r)` entry names after region substitution, and its read footprint is the places that row's `reads` paths reach under the same projection; the written footprint of s1 overlaps neither footprint of s2, and the written footprint of s2 overlaps neither footprint of s1.
Evaluating a statement's own argument expressions is part of that statement and therefore part of the overlap, so each call's written footprint also overlaps no place the other statement's argument expressions read; taking the address of a place is not reading it, and both directions are required because which statement's argument evaluation an overlap moves is the implementation's choice.
Each statement additionally holds, for the duration of its call, a loan on the resolved place of every argument written as a borrow [OWN-12]: a `&uniq 'r` argument holds an exclusive loan and a `&'r` argument a shared loan, whatever that argument's parameter region does or does not carry in the callee's row.
A loan of one statement denies permission against an overlapping loan or footprint element of the other exactly where [OWN-5] denies the corresponding pair of a live loan and an overlapping access: an exclusive loan denies against every overlapping loan, written or read footprint element, and argument-expression read of the other statement, and a shared loan denies against every overlapping exclusive loan and written footprint element; two overlapping shared loans deny nothing.
The reason is [OWN-5] itself: every borrow this rule judges is live and usable across the whole of its statement's call [OWN-12], so an implementation that overlaps two statements makes both statements' borrows simultaneously live and usable, and permission therefore requires of the resulting loan state exactly what [OWN-5] requires of one statement holding all of those loans at once.
A footprint element whose caller place the implementation does not resolve overlaps every place, and so does a place read by an argument expression whose caller place the implementation does not resolve, and so does the loan of a borrow-written argument whose caller place the implementation does not resolve, so an unresolved element denies permission rather than granting it.
Every statement written between s1 and s2 is part of the permitted window and is judged by these same conditions: it is an ordinary value `let_stmt`, a `let_stmt` whose selected right-hand side is one call judged exactly as a member is, or a `set` or `replace` statement; its written, read, and argument-expression footprints and its loans are formed exactly as a member's are; and no written footprint, read footprint, or loan of a member may stand against any other window statement, nor any intervening statement's against a member, where the conditions above deny the pair; two intervening statements owe each other nothing, because both admitted schedules run the intervening statements in source order on the thread that did not take the hand-out, so no two of them ever overlap — with one exception on the member side, owed to the same pair of admitted schedules as the argument-evaluation sentence above: no obligation runs between an intervening statement's written footprint or loan and s1's own argument-expression reads, because either admitted schedule completes s1's argument evaluation before any intervening statement runs.
A statement of any other form between the members denies permission, a statement carrying an exit edge denies permission, and a non-call statement that forms a borrow denies permission, so every loan live inside a permitted window is one an argument of a judged call holds.
Every call and compiler-derived release in the window has a complete target summary [FN-1, SYS-2]. The summary does not grant or deny permission; effects and loans already decide interference. A `may-suspend` member selects completion lowering when the implementation actualizes the window, and each retained argument loan remains live until that member's corresponding `loan-released(path)` milestone. A target that cannot actualize that complete lowering executes the otherwise valid window sequentially or reports an unsupported target lowering, never a source rejection.
Every normal continuation of s1 reaches s2, so no edge out of s1 leaves the enclosing block or function without first reaching s2, and every normal continuation of each intervening statement likewise reaches s2.
Permission for a chain of three or more statements is exactly permission for every ordered pair the chain contains.

Under a permitted overlap, bindings and every Whitefoot state place equal the source-order result. Writes to distinct places have no additional cross-place order merely because their target implementations are externally observable [EFF-5]. Target completion order, the lane that harvests a completion, and writer resumption order are not observations unless an ordinary dependency exposes their results in that order.
That identity is conditional on contract compliance, exactly as [SCOPE-3]'s freedom from undefined behavior is conditional on its trusted computing base.
For an execution in which no executed `claim` is false it holds in every execution, not in a typical execution or in some execution.
An execution in which some executed `claim` is false is erroneous: the program has violated the sole writer-reachable language runtime contract [SCOPE-4], and this rule then requires exactly the following of that execution.
The process writes exactly one complete [DIAG-3] record, naming one `claim` whose predicate evaluated false, and then aborts the whole process without unwinding and without language cleanup [TRAP-1].
No second record, and no partial or interleaved record, is written.
Which such `claim` that record names may depend on the schedule. The schedule may also select which already permitted state transitions became visible before process termination; those transitions retain their system operation contracts and are not rolled back.
Nothing else narrows for an erroneous execution: it has no undefined behavior [SCOPE-3], and no overlapped pair reaches one state place or violates one ordinary loan except as the conditions above admit.
No permission, submission, completion, or fast path reads a trap latch or pays any other cost whose purpose is to stabilize this erroneous execution. A correct program executes no false `claim`, so the impossible branch cannot narrow or surcharge its execution [SCOPE-4].
The number of workers, the identity of the host thread that executes a statement, the schedule, and whether an overlap was performed at all are not observable, and no rule of this specification is stated in terms of them.
An implementation that overlaps nothing therefore conforms: this permission is never an obligation, and no program depends on it being taken.
When an execution of s1 or s2 reaches a false-claim trap instead of its continuation, what survives is exactly what the erroneous-execution clauses above fix: one complete [DIAG-3] record, abort without unwinding or cleanup, unconditional memory safety, and only system-contract-valid state transitions. This rule does not promise source-order intermediate values or a deterministic partial outside trace for the defective execution.
Exhaustion of the execution resources an implementation spends on overlapping is a resource condition under [SCOPE-3] and is not an observable of this rule.
This rule depends on [DIAG-3]'s record shape: that record names no worker, host thread, or dynamic call stack, so a permitted overlap cannot change its bytes.
Every construct of this specification defines one total sequential order over its operand evaluations, and this rule is a consumer of that order rather than a relaxation of it.
This rule uses [CAP-1]'s ordinary ownership boundary directly; it introduces no additional sharing classification.
The counted permission [PAR-2] and the staged permission [PAR-3] each form every footprint and every loan of a statement exactly as this rule forms one.

[PAR-2] An implementation may execute two iterations of one `for_stmt` body with overlapping execution, and may recombine that loop's accumulator across them, only when the permission this rule defines holds for that counted loop.
Permission holds for a `for_stmt` L exactly when all of the following hold, writing B for L's body and forming every written, read, and operand-read footprint of a statement of B exactly as [PAR-1] forms one.
A footprint of B writes at most one place rooted in a binding declared outside L; that binding is L's accumulator, and every occurrence of it in B is one operand of one `set` statement whose target is that whole binding and whose right-hand side is one operation applied to that operand and to a second operand reaching the accumulator nowhere.
That operation is one operation fixed for the accumulator across the whole of B, and is exactly one of `+wrap`, `*wrap`, `iand`, `ior`, `ixor`, `imin`, `imax`, `band`, `bor`, and `bxor` [OP-1].
Every place a footprint of B writes is either that accumulator's whole place or is rooted in a binding B itself introduces, so no two iterations write one place except through that accumulator.
Every place a footprint of B holds an exclusive loan on — its statements' argument borrows holding loans exactly as [PAR-1]'s do — is rooted in a binding B itself introduces, so no two iterations hold exclusive loans on one place; a shared loan needs no condition of its own, because the written condition above leaves the accumulator as the one enclosing place any iteration writes and an accumulator any borrow reaches is refused by the accumulator condition; and a non-call statement of B that forms a borrow denies permission, exactly as one denies a [PAR-1] window.
A footprint element whose caller place the implementation does not resolve overlaps every place, so an unresolved element denies permission rather than granting it.
Every call and compiler-derived release in B has a complete target summary. Effects and ordinary loans decide interference between iterations exactly as they do between [PAR-1] calls. A `may-suspend` action requires a completion lowering that retains every iteration's argument loans until its declared milestones; an implementation without that lowering executes the loop sequentially.
Every normal continuation of every statement of B reaches L's compiler-owned binder update, so no statement of B is a `return_stmt`, a `give_stmt`, a `break_stmt` naming L or a loop enclosing L, or a `let_stmt` selecting `propagate_let_rhs` [FN-1, GIVE-1, ERR-3].

Under a permitted overlap every state-place observable is the one produced by executing L's iterations in index order. Distinct places gain no extra cross-place order from the target mechanism [EFF-5].
Write a0 for the accumulator's value on the true header edge entering the first executed iteration, and t0 through tm for the values the second operand of its writes evaluates to, in the order those writes execute across L's iterations taken in index order.
Source order computes the accumulator's value at L's continuation as the left-nested application of that operation to a0 then t0 through tm where its writes place the accumulator in the first operand position, and as the right-nested application to t0 through tm then a0 where they place it in the second.
An implementation may instead apply that operation over any binary tree whose leaves are a0 and t0 through tm, each occurring exactly once and in any order, together with any number of leaves holding that operation's identity element.
Every admitted operation is a total function on the complete value set of its type, carries no domain obligation, and is associative and commutative on that set with a two-sided identity element — `+wrap` and `*wrap` are the ring operations of the integers modulo two to the width, with identities zero and one; `iand`, `ior`, and `ixor` are the meet, join, and group operations of the bit vector, with identities the all-ones vector, zero, and zero; `imin` and `imax` are the meet and join of that type's total order, with identities the type's greatest and least values; and `band`, `bor`, and `bxor` are the two-element cases of the same three, with identities `true`, `false`, and `false` — so every such tree denotes one value of that type and the accumulator's value at L's continuation is that one value in every execution.
No further operation is admitted: `+`, `+defined`, and `+checked` each attach a domain obligation or a `Result` route to every application, `+sat` is not associative, and no float operation of [OP-1] is associative, so recombining a `fadd.strict` or `fmul.strict` fold could change published bytes.
This rule uses associativity, commutativity, and the identity together: commutativity is what admits any leaf order and the fold of the second operand position, and the identity is what lets an implementation seed a subrange of iterations before knowing whether that subrange writes, so a range of iterations that writes the accumulator not at all contributes either nothing or identity leaves that change nothing.
That identity is conditional on contract compliance exactly as [PAR-1]'s is, and an erroneous execution of L — one in which some executed `claim` is false — receives exactly the guarantee [PAR-1] states for one, with the `claim` the single [DIAG-3] record names selected from among those whose predicates evaluated false.
Both endpoint atoms are still evaluated exactly once each in [FN-1]'s order before any iteration begins, and the binder still takes each value of the half-open range exactly once; this rule relaxes only the order in which iterations execute and the shape of the accumulator's combination, never the set of iterations, the values the binder takes, or either endpoint evaluation.
The number of workers, the identity of the host thread that executes an iteration, the schedule, how the index range is divided, and whether any overlap or recombination was performed at all are not observable, and no rule of this specification is stated in terms of them.
An implementation that overlaps nothing therefore conforms: this permission is never an obligation, and no program depends on it being taken.
When an execution of one iteration does not reach its continuation, the overlapped execution produces exactly the observables the index-order execution produces before that point and produces none after it.
Exhaustion of the execution resources an implementation spends on overlapping is a resource condition under [SCOPE-3] and is not an observable of this rule.
Permission over the iterations of a `for_stmt` written inside B is exactly this rule applied to that loop; no rule of this specification joins two index ranges into one iteration space.
This rule uses [CAP-1]'s ordinary ownership boundary directly; it introduces no additional sharing classification for the accumulator or any other place.
A separate staged permission [PAR-3] cuts the body of any `for_stmt` or `loop_stmt` at its first `may-suspend` submission and admits a different overlap; it is not this permission, and neither implies the other.

[PAR-3] An implementation may execute the segments of two iterations of one `for_stmt` or `loop_stmt` body with overlapping execution only when the staged permission this rule defines holds for that loop.
This permission is distinct from [PAR-2]'s counted permission, requires none of its accumulator, combination, or index conditions, and grants a different overlap; a loop may hold either, both, or neither.
Permission holds for a loop L exactly when all of the following hold, writing B for L's body and forming every written, read, and operand-read footprint and every loan of a statement of B exactly as [PAR-1] forms one.
There is one program point c of B such that every statement of B either executes before c on every path through B or is reached only through c, and c is the argument evaluation and submission of the first `may-suspend` action of B in program order. Write P for the statements up to and including c and E for the rest.
Every edge that leaves B — a `return_stmt`, a `give_stmt` delivering outside B, a `break_stmt` naming L or a loop enclosing L, and a `let_stmt` selecting `propagate_let_rhs` [FN-1, GIVE-1, ERR-3] — occurs in P.
An edge the statement performing c takes on the outcome of that submission, which is the edge a `let_stmt` selecting `propagate_let_rhs` at c takes, is an edge of E and not of P.
Every borrow a `may-suspend` call of B retains past its own submission is on a place rooted in a binding B itself introduces, on a place this rule replicates, or on a place no footprint of B writes. Every exclusive loan a call of E holds is on a place rooted in a binding B itself introduces or on a place this rule replicates.
Every place rooted in a binding declared outside L that a footprint of B reaches satisfies one of exactly three conditions, and a place satisfying none denies permission. Either no footprint of B writes it and every loan on it is shared; or every footprint element and every loan touching it belongs to one of P and E alone and no loan on it is retained past c; or this rule replicates it.
Every call and compiler-derived release in B has a complete target summary [FN-1, SYS-2]; a footprint element, loan, extent, or statement form the implementation does not resolve denies permission rather than granting it.
Under the staged permission an implementation may execute the segment E of one iteration with overlapping execution against the segment P of any later iteration, and against the segment E of any other iteration.
The executions of P for the iterations taken in index order do not overlap one another, and no execution of P begins before the execution of P of every earlier iteration has completed.
Every write E performs to a place rooted outside B occurs in the order of the iterations that perform it.
Every read E performs of a place rooted outside B that a footprint of B writes likewise occurs in the order of the iterations that perform it.
Under a permitted staged overlap, bindings and every Whitefoot state place equal the source-order result, on exactly the terms [PAR-2] states for its own permitted overlap, including its erroneous-execution clauses.
An implementation may replicate a place, giving each concurrently executing iteration its own storage of the same length, only when that place's element type is copy [OWN-1], when no statement L's continuation reaches reads it, and when on every path through B every byte of it a footprint of B reads was written by an earlier footprint of B on that path.
The bytes one footprint reads, and the bytes it may write, are exactly those its operation contract fixes for a system operation [SYS-8], those the callee's own summary fixes for a user call after the [EFF-2] boundary projection, and the exact subscripted position for a direct element access; observing a place's length reads no byte of it.
A byte counts as written by a footprint, for the coverage condition above, only where that contract fixes that the footprint changes it: a contract stating only which bytes of a buffer may have changed [SYS-8] establishes no written byte, and a range the coverage condition needs must come from a contract that states the change exactly.
An extent the implementation does not resolve is the whole place for a read and empty for a write, and an underivable containment denies replication rather than granting it.
When an execution of one iteration leaves L through an edge of P, the overlapped execution produces exactly the observables the source-order execution produces before that point and produces none after it; every operation of an earlier iteration still outstanding is completed and its segment E performed before that edge is taken.
The host resources a system operation of L creates are not execution resources an implementation spends on overlapping. An overlapped execution delivers for each operation of L an outcome that operation could deliver in the source-order execution at that point, so an implementation whose overlap holds more such resources at once than the source-order execution holds completes the earlier iterations and performs the operation again at the source-order resource footprint before delivering any outcome.
Exhaustion of the execution resources an implementation spends on overlapping is a resource condition under [SCOPE-3] and is not an observable of this rule.
An execution in which some executed `claim` is false is erroneous: the program has violated the sole writer-reachable language runtime contract [SCOPE-4], and this rule then requires exactly the following of that execution.
The process writes exactly one complete [DIAG-3] record, naming one `claim` whose predicate evaluated false, and then aborts the whole process without unwinding and without language cleanup [TRAP-1].
No second record, and no partial or interleaved record, is written.
Which such `claim` that record names may depend on the schedule. The schedule may also select which already permitted state transitions became visible before process termination; those transitions retain their system operation contracts and are not rolled back.
No permission, submission, completion, or fast path reads a trap latch or pays any other cost whose purpose is to stabilize this erroneous execution. A correct program executes no false `claim`, so the impossible branch cannot narrow or surcharge its execution [SCOPE-4].
The number of operations an implementation keeps outstanding, the identity of the host thread that executes a segment, whether any overlap was performed at all, the storage an implementation gives a replicated place, and the storage an implementation reuses across iterations for a construction whose value the body releases without observing it, are not observable, and no rule of this specification is stated in terms of them.
An implementation that overlaps nothing therefore conforms: this permission is never an obligation, and no program depends on it being taken.
Permission over the iterations of a loop written inside B is exactly this rule applied to that loop; no rule of this specification joins two iteration spaces into one.
This rule uses [CAP-1]'s ordinary ownership boundary directly; it introduces no additional sharing classification for a replicated place or any other place.

## 14. Gated family (writer-visible stub)

[GATE-1] Editing any declared contract, signature, law bundle, storage contract, or gated-family member is one privileged, gated toolchain operation with one audit trail, outside steady-state writer capability.

This version defines no callable FFI import, export, inbound callback, foreign-thread entry, or generated foreign adapter.
A later amendment that adds one must define a new proof boundary explicitly; it may not silently reinterpret an internal [FN-8] requirement as an executable prologue or recover the deleted check-and-trap adapter model.
Until such a path is specified and implemented it remains unsupported compiler capability [DIAG-1].

[LEDGER-1] There is exactly one boundary-construct family (unsafe regions, FFI extern frames, trusted primitive imports), sharing one per-fact soundness-obligation ledger; manifest-free members are unrepresentable; members are AI-authored and human-approved through the gate (owner ruling D0a).
A kernel writer sees these constructs only as opaque, pre-approved library signatures.

[GATE-2] The system domain is not this family.
A system operation is
compiler-owned: this specification fixes its complete contract [QUAL-1] and an
approved target entry supplies its implementation, so it is neither an unsafe
region, an FFI extern frame, nor a trusted primitive import; it holds no
per-fact soundness-obligation ledger entry [LEDGER-1] and is not a
writer-authored, writer-approved, or gate-edited declaration [GATE-1].
A
program that calls system operations therefore contains no gated construct and
remains a kernel program [SCOPE-1], and [SCOPE-3]'s foreign-code condition is
not engaged by a system operation.
The converse separation is equally exact:
the system domain admits exactly the operations this specification names, while
general FFI, arbitrary imported or exported foreign calls, raw host-ABI calls,
and writer-declared external signatures remain reserved to this family and are
unreachable through the system domain.
Adding a system operation is a
specification amendment [META-5]; it is never a gate approval, a ledger entry,
or a target-implementation act.

## 15. Prelude (normative, counted)

[PRE-1] The prelude is exactly:

```
enum Bool {
  True();
  False();
}

enum Option<T> {
  None();
  Some(value: T);
}

enum Result<T, E> {
  Ok(value: T);
  Err(error: E);
}

enum Overflow {
  Overflow();
}

enum DivError {
  DivideByZero();
  DivOverflow();
}

enum NarrowError {
  NarrowError();
}

contract Int {
}

contract Float {
}
```

## 16. System declaration domain (normative, counted)

[SYS-1] There is exactly one compiler-owned system declaration domain.
It is a third admitted declaration source alongside source declarations and the prelude [PRE-1]; it is not the prelude and not a member of the gated boundary family [GATE-1, LEDGER-1].
Its complete membership is the system inventory [SYS-2], admitted by every compilation unit [SYS-3].

A system declaration is compiler-owned data of this specification.
It is not a source record, include, import, module, separate compilation, dynamic loading, or source-path lookup [PROG-1].
It has no source record, source node, source coordinate, role, or declaration event, and no source construct declares, redeclares, extends, reopens, or overrides it.

The inventory contributes exactly three declaration classes, each already a member of the closed declaration-class order [DIAG-1].
A system nominal type takes the nominal-type class and is an entry of the nominal-type TYPEID domain [TYPE-6].
A system constructor takes the struct-constructor or enum-variant class fixed for it by [SYS-2] and is an entry of the constructor TYPEID domain [TYPE-6].
A system operation takes the function class and is an entry of the lexical IDENT domain [TYPE-6].
The domain contributes no contract, region, label, const-generic, generic-type, value, or operation-family entry, and introduces no declaration class.

In every unit every inventory entry is visible throughout the closed unit, is a compilation-root entry of its domain in every lexical use's candidate universe [DIAG-1], and participates in that domain's whole-unit uniqueness [TYPE-6].
That visibility depends on neither the position of the entry declaration, nor record order [PROG-2], nor any source declaration point.
The owner-local field and parameter records fixed by [SYS-2] are visible only within their owning system declaration and never enter source lookup.

A source declaration whose spelling equals an inventory entry's spelling in the same domain is a collision, rejected under [DIAG-1], at the compilation root and in every nested scope alike.
No source declaration displaces, replaces, overrides, reopens, or shadows an inventory entry, and no inventory entry displaces a source declaration: the unit is rejected and neither declaration resolves.
No use of a colliding spelling is decided by proximity, declaration order, scope depth, or expected type.

A system nominal type, constructor, or operation exists only as an inventory entry [SYS-2].
No source construct becomes one by spelling, signature, parameter shape, result type, effect row, or any other source property.

Each inventory entry has one zero-based `system_declaration_ordinal` assigned by the [SYS-2] preorder.
That ordinal is the entry's identity in a diagnostic origin [DIAG-1].

[SYS-2] The system inventory is exactly:

The notation here is normative record notation and is not writable source.

Ten opaque nominal types: `Args`, `HostString`, `RelativePath`, `DirectoryRead`, `ReadFile`, `Output`, `ExitStatus`, `DirectorySource`, `FileFactory`, and `FilePermit`.
Each contributes one nominal-type entry and no constructor entry.
An opaque type has no writer-visible field, variant, literal, size, alignment, or representation.
It is a complete written `type` under [GRAM-3] as a bare TYPEID with no `targs`, carries no region and no type parameter, and is therefore region-free under [STOR-5].
It is not const-eligible [CONST-2], is not a `cvt` or `reinterpret` domain [OP-6, OP-8], is not an integer, float, `Bool`, or tag-only enum operand domain [OP-1], and has no equality, ordering, or conversion operation.
Its values are produced only by the operations in this rule and by the command entry's standard input bindings.
Every value of an opaque type is affine under [OWN-1].

Eight enum nominal types with forty variant constructors:

```
enum ArgError {
  InvalidIndex();
}

enum Utf8Error {
  Utf8Invalid();
}

enum CopyError {
  CopyTooSmall(required: u64);
}

enum Utf8CopyError {
  Utf8CopyTooSmall(required: u64);
  Utf8CopyInvalid();
}

enum PathError {
  PathInvalid();
}

enum ReadOutcome {
  ReadBytes(next: u64);
  ReadEnd();
  ReadFailed(error: IoError);
}

enum IoError {
  NotFound(code: u32, origin: u8);
  PermissionDenied(code: u32, origin: u8);
  AlreadyExists(code: u32, origin: u8);
  NotDirectory(code: u32, origin: u8);
  IsDirectory(code: u32, origin: u8);
  DirectoryNotEmpty(code: u32, origin: u8);
  ReadOnly(code: u32, origin: u8);
  ResourceBusy(code: u32, origin: u8);
  InvalidInput(code: u32, origin: u8);
  InvalidPath(code: u32, origin: u8);
  Unsupported(code: u32, origin: u8);
  TimedOut(code: u32, origin: u8);
  BrokenPipe(code: u32, origin: u8);
  WriteZero(code: u32, origin: u8);
  UnexpectedEnd(code: u32, origin: u8);
  ConnectionRefused(code: u32, origin: u8);
  ConnectionReset(code: u32, origin: u8);
  ConnectionAborted(code: u32, origin: u8);
  NotConnected(code: u32, origin: u8);
  AddressInUse(code: u32, origin: u8);
  AddressUnavailable(code: u32, origin: u8);
  ResourceExhausted(code: u32, origin: u8);
  FileTooLarge(code: u32, origin: u8);
  NoSpace(code: u32, origin: u8);
  QuotaExceeded(code: u32, origin: u8);
  CrossDevice(code: u32, origin: u8);
  DeviceFailure(code: u32, origin: u8);
  Other(code: u32, origin: u8);
}
enum ListOutcome {
  ListBytes(next: u64, entries: u64);
  ListEnd();
  ListFailed(error: IoError);
}
```

Sixteen operations, each one complete signature record in the [GRAM-2] `fn_sig` shape:

```
fn args_count['a](args: &'a Args) -> result: own u64 reads(args);
fn arg_get['a](args: &'a Args, position: own u64) -> result: own Result<HostString, ArgError> reads(args);
fn host_bytes_len['v](value: &'v HostString) -> result: own u64 reads(value);
fn host_copy_bytes['v, 'd](value: &'v HostString, destination: &uniq 'd buffer<u8>, start: own u64, end: own u64) -> result: own Result<u64, CopyError> reads(value, destination), writes(destination);
fn host_utf8_len['v](value: &'v HostString) -> result: own Result<u64, Utf8Error> reads(value);
fn host_copy_utf8['v, 'd](value: &'v HostString, destination: &uniq 'd buffer<u8>, start: own u64, end: own u64) -> result: own Result<u64, Utf8CopyError> reads(value, destination), writes(destination);
fn relative_path(value: own HostString) -> result: own Result<RelativePath, PathError> pure;
fn open_read['c, 'p](permit: own FilePermit, root: &'c DirectoryRead, path: &'p RelativePath) -> result: own Result<ReadFile, IoError> reads(permit, root, path), writes(permit);
fn read_at['f, 'd](file: &'f ReadFile, destination: &uniq 'd buffer<u8>, file_offset: own u64, start: own u64, end: own u64) -> result: own ReadOutcome reads(file, destination), writes(destination);
fn write_once['o, 's](output: &uniq 'o Output, source: &'s buffer<u8>, start: own u64, end: own u64) -> result: own Result<u64, IoError> reads(output, source), writes(output);
fn exit_status(code: own u8) -> result: own ExitStatus pure;
fn open_directory['c, 'n](permit: own FilePermit, root: &'c DirectoryRead, name: &'n buffer<u8>, start: own u64, end: own u64) -> result: own Result<DirectoryRead, IoError> reads(permit, root, name), writes(permit);
fn open_directory_source['c](permit: own FilePermit, directory: &'c DirectoryRead) -> result: own Result<DirectorySource, IoError> reads(permit, directory), writes(permit);
fn directory_next['l, 'd](source: &uniq 'l DirectorySource, destination: &uniq 'd buffer<u8>, start: own u64, end: own u64) -> result: own ListOutcome reads(source, destination), writes(source, destination);
fn open_file['c, 'n](permit: own FilePermit, root: &'c DirectoryRead, name: &'n buffer<u8>, start: own u64, end: own u64) -> result: own Result<ReadFile, IoError> reads(permit, root, name), writes(permit);
fn reserve_file['f](factory: &uniq 'f FileFactory) -> result: own FilePermit reads(factory), writes(factory);
```

The inventory is therefore exactly eighteen nominal types, forty enum-variant constructors, sixty-three variant fields, sixteen operations, twenty-two operation region parameters, and forty-four operation value parameters.

Each operation's state access is fixed by its own signature. Immutable invocation and host-string state is observed through shared parameters and contributes `reads(parameter)`. Every buffer or system resource the operation changes is supplied through `&uniq` and contributes `writes(parameter)`. The lifetime on each borrow states only how long that loan lives and never appears in the row.
The rows above are exactly those state accesses; a system operation's row is declaration data and is never derived from a body, narrowed by a proof, or selected by a call site [ERR-4].
Each operation's result components and writable `&uniq` parameter components additionally carry the following closed [PRV-1] provenance classes as declaration data; these classes do not add to or modify an operation signature or effect row.
No system operation allocates.

Each operation additionally carries one compiler-owned target contract. This record is not source syntax and does not change contract equality.
The target contract is `never-suspends` for `args_count`, `arg_get`, `host_bytes_len`, `host_copy_bytes`, `host_utf8_len`, `host_copy_utf8`, `relative_path`, `exit_status`, and `reserve_file`. It is `may-suspend` for `open_read`, `read_at`, `write_once`, `open_directory`, `open_directory_source`, `directory_next`, and `open_file`.
A `may-suspend` operation is a finite one-shot operation. Its logical record exists before target handoff and carries separate `result-ready`, one `loan-released(path)` fact for every retained borrow, and `terminal` milestones. In this first system slice the `loan-released(path)` fact for the name an open borrows — `open_read`'s `path`, `open_file`'s and `open_directory`'s `name` — is published before target transfer, because forming the request copies the admitted code-unit range into compiler-owned storage and that copy is the operation's last access to the caller's buffer; every other applicable fact is published by the same exactly-once terminal transition. Keeping them distinct is required contract structure, not a promise that later operations publish them together.
The operation result becomes an ordinary usable source value only when its `result-ready` fact holds, and each borrow held for the call remains live until its own `loan-released(path)` fact holds. The call's ownership-complete requirement is the conjunction of its result fact and every loan-release fact the caller regains.

Every borrow keeps its ordinary [OWN-5] loan until the target contract releases it. `reserve_file` returns one fresh ordinary `FilePermit`; a successful open which returns `ReadFile`, `DirectoryRead`, or `DirectorySource` likewise produces one fresh ordinary owned value. Each result has its own binding identity and no compiler-retained parent relation to the parameter that produced it [EFF-2].

Submission has exactly three internal outcomes. `inline-terminal` publishes every promised milestone and guarantees that no later completion can arrive. `target-owned` transfers the complete operation bundle to the qualified adapter. `wait-capacity` transfers nothing to the target and retains the bundle in the runtime so another ready frame can run until bounded target capacity is available. No source value observes which outcome occurred.
A qualified target may implement this contract with native completion, readiness plus a nonblocking attempt, polling, interrupts, or a bounded blocking helper. A helper executes only the typed target adapter and publishes milestones; it never executes a writer function or writer continuation. Target completion publication may make a stackless writer frame runnable, but target code never invokes that frame directly.
No submission, completion, or target path tests a trap latch or records trap-specific state [SCOPE-4].

```wf-prov
| operation | result component class | written parameter component class |
|---|---|---|
| `args_count` | plain result external | — |
| `arg_get` | `Ok(value:)` external; `Err(error:)` external | — |
| `host_bytes_len` | plain result external | — |
| `host_copy_bytes` | `Ok(value:)` dependent; `Err(error:)` external | `destination` external |
| `host_utf8_len` | `Ok(value:)` external; `Err(error:)` external | — |
| `host_copy_utf8` | `Ok(value:)` dependent; `Err(error:)` external | `destination` external |
| `relative_path` | `Ok(value:)` external; `Err(error:)` external | — |
| `open_read` | `Ok(value:)` external; `Err(error:)` external | — |
| `read_at` | `ReadBytes(next:)` dependent; `ReadFailed(error:)` external; `ReadEnd()` carries no result component | `destination` external |
| `write_once` | `Ok(value:)` dependent; `Err(error:)` external | `output` external |
| `exit_status` | plain result internal | — |
| `open_directory` | `Ok(value:)` external; `Err(error:)` external | — |
| `open_directory_source` | `Ok(value:)` external; `Err(error:)` external | — |
| `directory_next` | `ListBytes(next:)` dependent; `ListBytes(entries:)` internal; `ListFailed(error:)` external; `ListEnd()` carries no result component | `destination` external |
| `open_file` | `Ok(value:)` external; `Err(error:)` external | — |
| `reserve_file` | plain result internal | `factory` internal |
```

A plain-result cell fixes that result's sole aggregate component.
A named payload cell fixes exactly that direct variant-field projection; a payload-carrying result's aggregate is the join of its projections, while a nullary variant and the control choice of a variant carry no component of their own.
An external class seeds the unconditional-external bit; an internal class seeds no bit.
A dependent class denotes the join of the concrete call's `start` actual and one sanitized internal host count, as fixed by [SYS-9]; it seeds exactly that instantiated join rather than a declaration-global bit.
No unlisted result, projection, parameter, field, or component inherits an external class by association.
The third column is provenance only. The boundary action gives that formally written component an unconditional external origin for PRV-1 composition; the same ordinary effect paths and loans already describe interference [EFF-2, EFF-5].
The internal success components are exactly the program-bounded entry counts fixed by [SYS-9].

Every system operation is nongeneric: it declares no type parameter and no const parameter, so no `targ` in a system-operation call is a `type` or a `const`.
A call whose callee resolves to a system operation writes its region arguments as `targs` in declared region-parameter order and its value arguments as a `fieldinit_list` [GRAM-5] whose IDENTs equal the declared parameter names in declared order, under the same discipline [GRAM-11] applies to a user function.
Positional operands are not admitted.
A system operation is not a contract member, is not the right IDENT of an [FN-3] `fn_bind`, and never satisfies [FN-4]'s bound-function premise; a conformance binds only a top-level source function.

The inventory contributes exactly two hundred and three declaration records in this preorder: each nominal type in table order; then each constructor in table order, and within one constructor each of its fields in declared order; then each operation in table order, and within one operation each of its region parameters in declared order followed by each of its value parameters in declared order.
Exactly the nominal types, the constructors, and the operations enter the source resolver's whole-unit lookup inventory of a system-admitted unit [SYS-1].
The field and parameter records are owner-local: a field record enters only its owning constructor's table and a parameter record only its owning operation signature, and neither is visible to source lookup.

Each nominal-type row fixes one TYPEID spelling.
Each constructor row fixes one TYPEID spelling, its class as struct-constructor or enum-variant, its owning system nominal type, and its fields in declared order, each with a field name and a type.
Each operation row fixes one IDENT spelling; its region parameters in declared order, each one REGIONID spelling; its value parameters in declared order, each with a parameter name, a mode, and a type; its result mode and type; and its written effect row.

The table satisfies the following properties.
Each is a property of this specification's data, established once for this document, and is not a source-language check.
Every nominal-type and constructor spelling satisfies TYPEID and every operation spelling satisfies IDENT and contains no dot [FORM-3].
No operation spelling is a member of `ReservedLowerNames` [OP-1].
Spellings are unique within each of the nominal-type, constructor, and lexical IDENT domains, and are disjoint from the PRE-1 spellings of the same domain.
Every field name is unique within its constructor and every parameter name is unique within its operation.
Every type written in the table is a nameable type [TYPE-3] fixed by this document or by this table.

[SYS-3] Every compilation unit is system-admitted.
The complete system inventory [SYS-2] enters every unit's declaration inventory as fixed by [SYS-1], independently of the unit's item sequence, command entry inputs, or uses.
There is no source form that disables, replaces, shadows, or conditionally admits that inventory; every collision and use is decided by the ordinary declaration and lexical-use ranks [DIAG-1].

## 17. System interface (normative, counted)

[HOST-1] A host string is a lossless target-indexed sequence of code units, not text.
The complete set of code-unit families is exactly two.
On a Unix-family target a host-string code unit is one 8-bit value in `0x01..0xff`, so every arbitrary non-NUL byte sequence is representable and preserved exactly.
On a Windows-family target a host-string code unit is one native 16-bit value, so every arbitrary 16-bit unit sequence is representable and preserved exactly, including sequences that are not a valid Unicode scalar sequence.
Every argument, environment entry, and native path value that reaches source through a system operation retains its complete original code-unit sequence: no operation normalizes, case-folds, reorders, substitutes, truncates, or refuses a code-unit sequence because it is not valid text.
Code-unit width and family are properties of the selected target [STOR-6], not a portable source layout: the opaque type carries them, no source construct observes them, and no operation yields them as source values of a fixed width.
A target whose native representation belongs to neither family qualifies for the host-string and path semantic IDs only under a specification amendment [META-5] that gives that target its own lossless family; without that amendment it fails qualification for exactly those semantic IDs [QUAL-1], and no implementation narrows their semantics to what its own string domain can carry.

[HOST-2] Conversion between host-string code units and UTF-8 text is explicit and fallible.
There is no implicit conversion [TYPE-4]: no rule admits a host string where text is required or text where a host string is required, and a host string reaches source content only through an operation that names the route it takes.
Exactly two routes exist.
The lossless route reports the exact length of, and copies, the target's own code units with no validation and no Unicode restriction; its only recoverable failure is a destination too small for that exact length.
The text route validates the complete code-unit sequence as text and either reports the exact encoded length and copies the complete encoding or reports an explicit invalid-text outcome; it never emits a replacement code point, drops a code unit, produces a truncated encoding, or copies part of an encoding.
Escaped, quoted, and lossy display of a host string are a DEFERRED separate presentation family with their own delta [META-5], not a mode of either route.
The exact operation names, signatures, buffer and range preconditions, and outcome types of both routes are [SYS-2] inventory data, with their transfer semantics in [SYS-8] and their outcome types in [SYS-6].

[HOST-3] The first system slice defines exactly one host-string type.
Its value is an opaque inline lease — a private code-unit address and length carried in the value itself — over immutable backing supplied by the command invocation, and a relative path constructed from one retains that same inline representation [PATH-1].
A lease owns no code-unit storage, several live leases may denote the same backing code units, and its compiler-derived release is a logical consume with no host call and no state effect [STOR-3].
Its backing is the command-lifetime argument snapshot that [QUAL-2] requires of every qualified target.
Because that backing strictly outlives every value derived from it, a lease denotes valid code units however it is bound, moved, matched, returned, passed, or stored, and no source-level rule relates a lease to its backing: a lease is neither a borrow nor a region-bearing type, so [STOR-5] places no restriction on storing one and [OWN-5] provenance does not describe it.
That guarantee is a property of the target, enforced at qualification, and is not a judgment over source.
A producer whose backing is not command-lifetime yields no value of this type: it introduces a distinct owned-backing string resource with its own release action and its own type contract, because storage class is a function of type [STOR-1] and one type carries exactly one release action.
Conversion between the two types is an explicit later operation with its own delta [META-5]; no implicit retype, coercion, or representation change relates them [TYPE-4].
Retention of lease identity in the checked program [DIAG-2] serves auditing and lowering; it is not a source-acceptance judgment and refuses no program.

[PATH-1] A relative path is an opaque value whose code units are admitted by construction from one host string and are never assembled, split, or concatenated as source text.
Construction consumes its input host string on success and on failure.
It succeeds exactly when the complete code-unit sequence contains no NUL code unit and begins with no target-root prefix, where a target-root prefix is a code-unit sequence the selected target resolves against a filesystem root, drive, device, or other namespace root rather than against a supplied directory state.
The exact target-root prefix set is target data fixed by that target's qualification record [QUAL-1]; a Unix-family leading separator and a Windows-family drive or UNC prefix are members of their targets' sets.
Success retypes the same inline lease [HOST-3] with no allocation, no copy, and no code-unit change; failure yields no path value.
Construction preserves every admitted code unit exactly — including `.` and `..` components and every separator — and performs no normalization, canonicalization, case folding, prefix stripping, or component collapse.
A path component type, an absolute path type, and every operation that decomposes, enumerates, joins, or displays a path are DEFERRED additions with their own deltas [META-5].
The first slice constructs one relative path from one host string and supplies it to a directory-relative operation [PATH-2].

[PATH-2] A `DirectoryRead` state value names one directory object, and a directory-relative operation resolves a relative path against it through the target's own directory-relative resolution.
A directory-relative operation resolves either one relative path value or one caller-supplied single path component [SYS-14]; both are resolved through the target's own directory-relative facility and neither is concatenated onto a prefix.
The value bound to the command's working-directory entry input is process-equivalent: resolution follows `.` and `..` components, symbolic links, reparse points, and mount transitions exactly as the surrounding process namespace does, so a resolved object may lie outside the directory that value names.
That is the complete promise this type makes, and it is not a confinement claim.
An implementation presents no stronger one: a target implements directory-relative resolution with its own directory-relative facility, never by concatenating a prefix onto a path and resolving the result against an ambient working directory, and a target with no directory-relative facility fails qualification for the directory-relative semantic IDs [QUAL-1] rather than emulating them.
A confined directory state type, one guaranteeing that lexical traversal, links, mount transitions, and rename races cannot escape a granted root, is a DEFERRED addition with its own distinct contract [META-5]; a value's confinement promise is fixed by its type and never changes at runtime.
Absolute paths, cross-root operations, and target-root prefixes require their own inputs and operations, and `DirectoryRead` admits none of them [PATH-1].

[QUAL-1] Every system operation has exactly one target-independent semantic ID owned by this specification.
That ID's record binds the operation's signature, complete outcome set, ownership transitions, state effects [EFF-1], capacity behavior, completion milestones, compiler-derived cleanup [STOR-3], target action, and required target guarantees [QUAL-2].
The checked program carries only the semantic ID [DIAG-2]: an operation's identity comes from resolution in the system declaration domain, and no source function name or spelling, logical path [PROG-2], project, corpus, test, or signature lookalike ever selects, adds, or removes one.
A separate target-qualification table maps each `(specification version, semantic ID, target, program kind)` to exactly one approved implementation version and one private ABI symbol.
The compiler consults that table after selecting the exact target and ABI [STOR-6] and before emitting any use of the operation.
Compilation stops when the mapping is absent, when the approved implementation is incompatible with the selected target or program kind, or when a required target guarantee is unmet.
That stop is a target-qualification failure under [DIAG-1]; like a target-layout failure it is not a source-language rejection and cites no language rule.
Qualification never narrows a semantic ID to what a target can supply, and no implementation substitutes a different or weaker operation for an unqualified one.
An approved implementation may be replaced only within one semantic identity: a change to any element the record binds is a different semantic ID under a new specification version [META-5] and a compatibility review, never a target-code update.
The table is compiler-internal data; the language defines no registry, negotiation protocol, dynamic loading, or plugin interface [PROG-1].

[QUAL-2] A target qualifies for a semantic ID exactly when it supplies every target guarantee that ID's record requires; when it cannot supply one, it fails qualification for that ID and compilation stops [QUAL-1] rather than admitting the operation under a weaker guarantee.
Four guarantees are stated here because each is a property of the target with nothing in a program to check.
The first is command-lifetime argument backing: a target qualified for the command entry and for argument access supplies immutable backing for every argument code unit that is valid from before entry until the command invocation ends, either as stable native argument backing or as one complete snapshot taken before any Whitefoot code runs.
A target that can supply neither fails qualification for both IDs; a qualified target that cannot establish the backing for one invocation refuses startup before entry rather than entering with backing that does not meet this guarantee.
The second is a lossless host-string code-unit family [HOST-1] for the host-string and path semantic IDs.
The third is the target's own directory-relative resolution facility [PATH-2] for every semantic ID that resolves a relative path or one caller-supplied component against a `DirectoryRead`.
A target with no such facility fails qualification for those IDs rather than concatenating a prefix or resolving against an ambient working directory.
The fourth is a directory-enumeration facility for the enumeration semantic IDs [SYS-14]: one host call that reports a bounded batch of the entries of an open directory and advances that directory's own enumeration position.
A target with no such facility fails qualification for those IDs rather than emulating them, and in particular never substitutes a scan built out of other operations.
A target that has such a facility but for which the table [QUAL-1] holds no approved implementation is a different stop with the same effect: compilation stops for an absent mapping, the target is not thereby declared unqualified, and no implementation is improvised for it in either case.
Qualification failure and startup refusal both occur before entry [PROG-3], so neither is a source-returned status, a recoverable outcome, or a trap [TRAP-1].

[QUAL-3] For a natively compiled command, selection is static for the whole build: [QUAL-1] fixes the approved implementation of each semantic ID at compile time, and the emitted program contains no runtime operation-ID switch, target tag, per-call dispatch table, instance handle table, or handle lookup that selects among implementations.
An `inline-terminal` transfer lowers to its required source and target checks [STOR-6], at most one direct host attempt, one count or outcome check, and a cold outcome mapper reached only on failure; it remains one completion-contract outcome rather than a second blocking source mode.
That path performs no heap allocation, no copy of the transferred data, no global system lock acquisition, and no per-call signal-disposition operation.
The compiler wrapper is inlined, or any remaining call is shown to be immaterial, as a condition of qualification.
One-time per-invocation normalization belongs to the command bootstrap before entry rather than to any transfer: on the first native command targets that bootstrap owns the process and installs the ignored disposition for the write-to-closed-pipe signal, so a closed output destination reaches source as a recoverable outcome [ERR-4].
This rule fixes the required emitted shape; the evidence establishing it is inspection of emitted code and symbols, not a machine-checked language judgment.

[TRAP-1] A failing executed claim in a program holding system resources retains [SCOPE-4] and [EFF-4] exactly: the runtime attempts the mandatory [DIAG-3] record and then aborts the whole process without unwinding and without running language cleanup; no status is produced [PROG-3].
No release, close, flush, detach, or completion action fixed by a system resource contract [STOR-3] runs after a failed claim, and no source-visible cleanup, handler, or recovery point exists.
Process-local memory, native descriptors, and every other process-local system object held at that moment are reclaimed by operating-system process teardown, which is a property of the host inside the [SCOPE-3] trusted computing base rather than a language cleanup guarantee.
State transitions already performed are not rolled back: bytes already written remain written, an object already created remains created, and a persistent object or already-started target operation retains its own system contract.
A host that requires a Whitefoot instance to fail without ending its process runs that instance in a separate process.
Because a trap ends the owning process, no instance resource table, per-instance reaper, or pending-operation transfer is required, and none appears on an `inline-terminal` transfer path [QUAL-3].
Host-surviving in-process trap containment is a DEFERRED language amendment with its own delta [META-5].

[SYS-4] System types introduce no state-kind or capability classification.
Every operation signature directly states its ordinary parameter modes and exact state effects, which are the complete source facts used by ownership, effect checking, and overlap. A stable observation may use `&` plus `reads(path)`; an operation which advances, consumes, acknowledges, clears, or otherwise changes state uses `&uniq` or `own` and declares `writes(path)`. These are ordinary [OWN-2, EFF-1] judgments rather than properties inferred from the nominal type name.

The system catalog may keep target-only representation and release data for an opaque type [SYS-2, SYS-5]. That data selects construction, target lowering, and compiler-derived release, but it grants no source access and forms no second type hierarchy.
A later operation that duplicates or splits a resource exists only when it returns ordinary owned values whose independence is part of that operation's complete semantic contract. The first slice declares none, so no system value is duplicated, split, attenuated, or converted, and no integer right mask is exposed to source.

[SYS-5] Every system resource type declares one completion policy.
This specification defines exactly one: release-complete.
Under it, compiler-derived release is the complete language obligation for the type, and a source program needs no terminal operation to discharge ownership.
`Args`, `HostString`, `RelativePath`, `DirectoryRead`, `ReadFile`, `Output`, `ExitStatus`, `DirectorySource`, `FileFactory`, and `FilePermit` are all release-complete, so this specification defines no exact-use checking obligation.

Two further policy classes are named and reserved without machinery.
Explicitly-abandonable means the type exposes a consuming abandon operation whose contract permits loss of unfinished external work, so abandonment is a source action rather than an accidental affine discard.
Completion-required means every normal or recoverable exit must consume the owner through a terminal transition.
This specification declares no type under either class and defines no operation, checker obligation, or diagnostic for either; naming them fixes the vocabulary a later buffered output, atomic replacement, pending operation, or child process must use rather than silently inheriting release-complete [SYS-12].

The consuming release action of each system type is fixed by the following table. The table-local path `owner` is substituted under [EFF-2] and is not source syntax:

```wf-sys
| type | release action | state row | target contract |
|---|---|---|---|
| `Args` | logical consume | none | never-suspends |
| `HostString` | logical consume of an inline lease | none | never-suspends |
| `RelativePath` | logical consume of an inline lease | none | never-suspends |
| `DirectoryRead` | at most one native close attempt | `writes(owner)` | may-suspend; terminal |
| `ReadFile` | at most one native close attempt | `writes(owner)` | may-suspend; terminal |
| `Output` | logical source detach | none | never-suspends |
| `ExitStatus` | logical consume | none | never-suspends |
| `DirectorySource` | at most one native close attempt | `writes(owner)` | may-suspend; terminal |
| `FileFactory` | logical consume | none | never-suspends |
| `FilePermit` | logical consume | none | never-suspends |
```

A logical consume performs no host call, no target call, no handle lookup, no byte copy, and no state effect.
A native close attempt discards only the close diagnostic and never retries an ambiguous close: a consuming close invalidates the source handle on success and on error, because the native descriptor may already be closed and reusable.
Its one-shot terminal transition consumes the moved owner and carries no writer result. A consuming caller cannot continue past the release until `terminal`, but a scheduler lane need not remain occupied while the target owns the close.
`Output`'s logical source detach neither closes nor flushes the host descriptor [SYS-12].
Release of an outcome value is release of its components: `ArgError`, `Utf8Error`, `CopyError`, `Utf8CopyError`, `PathError`, `IoError`, `ReadOutcome`, and `ListOutcome` have no release action and take no row above, and a `ReadOutcome`, `ListOutcome`, or `Result` carrying a system value releases that value by this table.

A release action is compiler-derived and explicit in the checked program [STOR-3, DIAG-2].
`flush`, `sync`, directory sync, atomic commit, and final handle release are different semantic operations; this specification declares none of them, and release is never a substitute for one.
Whole-process abort performs no release: a trap runs no language cleanup and returns no status [PROG-3, EFF-4, SCOPE-4], and the operating system reclaims process-local memory and handles while completed system-state writes are not rolled back.

[SYS-6] Each system operation declares its own outcome type; there is no shared outcome union.
An operation with exactly two outcomes returns a [PRE-1] `Result<T, E>` instantiation and declares no new constructor spelling.
Each operation with more than two outcomes declares its own enum whose variant spellings carry that operation's prefix, so no two operations compete for a constructor name in the whole-unit constructor domain [TYPE-6].
The complete inventory is:

```wf-sys
| operation | outcome type |
|---|---|
| `args_count` | `own u64`; total, no failure outcome |
| `arg_get` | `own Result<HostString, ArgError>` |
| `host_bytes_len` | `own u64`; total, no failure outcome |
| `host_utf8_len` | `own Result<u64, Utf8Error>` |
| `host_copy_bytes` | `own Result<u64, CopyError>` |
| `host_copy_utf8` | `own Result<u64, Utf8CopyError>` |
| `relative_path` | `own Result<RelativePath, PathError>` |
| `open_read` | `own Result<ReadFile, IoError>` |
| `read_at` | `own ReadOutcome` |
| `write_once` | `own Result<u64, IoError>` |
| `exit_status` | `own ExitStatus`; total, no failure outcome |
| `open_directory` | `own Result<DirectoryRead, IoError>` |
| `open_directory_source` | `own Result<DirectorySource, IoError>` |
| `directory_next` | `own ListOutcome` |
| `open_file` | `own Result<ReadFile, IoError>` |
| `reserve_file` | `own FilePermit`; total, no failure outcome |
```

`InvalidIndex` states that the requested argument index is not present and returns no value.
`Utf8Invalid` states that the host string is not valid UTF-8.
`CopyTooSmall(required)` and `Utf8CopyTooSmall(required)` state the exact length the destination range must have for the same call to succeed.
`Utf8CopyInvalid` states that the host string is not valid UTF-8.
`PathInvalid` states that the consumed host string is not a valid relative path and returns no value.
`ReadBytes(next)`, `ReadEnd`, and `ReadFailed(error)` are [SYS-8]'s three read outcomes.
`ListBytes(next, entries)`, `ListEnd`, and `ListFailed(error)` are [SYS-8]'s three enumeration outcomes; `next - start` is the exact byte length of the portable entry-record prefix written into the requested range and `entries` is the exact number of complete records that prefix holds.
On a successful `arg_get` the `Ok` payload is the requested `HostString`; on a successful copy or write the `Ok` payload is the absolute first endpoint after the transferred range.

These error types are distinct nominal types and do not convert into one another [TYPE-4].
`propagate` [ERR-3] therefore chains only across operations that already share one error type: that is exactly `open_read`, `write_once`, `open_directory`, `open_directory_source`, and `open_file` inside a function whose written result is `own Result<U, IoError>`.
`PathError`'s `PathInvalid` and `IoError`'s `InvalidPath` are deliberately different failures and never substitute for each other.

[SYS-7] `IoError` is the closed portable class set declared by [SYS-2].
Its twenty-eight classes are the complete portable failure vocabulary of every system operation that can fail against a host, and the class is the sole portable semantic discriminator: exhaustive portable control flow branches on the class [ERR-2].
A target maps every native failure it can produce onto exactly one class.
A native error with no portable distinction in this set maps to `Other`; a new native error likewise maps to `Other` until a later numbered specification deliberately adds a portable distinction.
A target that cannot uphold a stated guarantee returns `Unsupported` rather than silently weakening it.

Every class carries the same fixed-size inline target detail: `code` is the target's native error code for that failure, mapped value-preservingly into `u32`, and `origin` is the target-owned discriminator selecting which native facility produced `code`.
Each field is zero when the target supplies no value for it.
A target that cannot represent its detail in these two fields maps the class to `Other` and reports the remainder through its own diagnostics.
The detail is diagnostic data, not a portable discriminator: source code may read and report it, and no portable semantics is defined in terms of it.

The detail is copy data in the transfer sense: it allocates nothing, owns nothing, and has no release action, so `IoError` takes no row in [SYS-5]'s release table and no operation row in [SYS-2] carries `allocates`.
A payload-carrying variant is affine under [OWN-1], so an `IoError` value, like a `ReadOutcome` value, is moved or matched rather than copied; that affinity is a consequence of the declared source form and is not a cleanup obligation.
No class carries a message, a buffer, or any heap-backed payload.

[SYS-8] `read_at`, `write_once`, `directory_next`, `host_copy_bytes`, `host_copy_utf8`, `open_directory`, and `open_file` are the complete range-bearing system-operation set.
Each accesses one caller-owned initialized `buffer<u8>` through a call-scoped borrow and names a half-open range `[start, end)` in that buffer; every resource and buffer owner remains with the caller on every outcome.

Every call to a member of this family carries exactly two independent [ENT-6] obligations in this order: `start <= end`, then `end <= len(deref(buffer))`, where `buffer` is that operation's declared buffer parameter.
Both obligations are queried in the caller's pre-transfer state and must be derived independently; neither is a premise for the other.
A refuted or unproved obligation rejects the call under [ERR-4].
There is no operation-internal range check, runtime fallback, or range trap.
Only after both obligations succeed may lowering form the exact extent `end - start`, whose absence of underflow follows from the first obligation, and pass the range to a target.
The target is never asked to validate a source pointer or source range.

For an empty range, `read_at` returns `ReadBytes(start)`, `write_once` returns `Ok(start)`, and `directory_next` returns `ListBytes(start, 0)`, each without a host transfer; an empty read or enumeration is never reported as `ReadEnd` or `ListEnd`.
For an empty copy range, a zero-length source succeeds with `Ok(start)` and a nonempty source returns its ordinary too-small outcome without writing.
For `open_directory` and `open_file`, an empty range is ordinary invalid component content and returns `Err(InvalidPath(code: 0_u32, origin: 0_u8))` before any host call.

For a nonempty range, `read_at`, `write_once`, and `directory_next` make at most one progress-producing host transfer attempt for the admitted call.
If a host interruption reports no progress, the target adapter resumes the same operation without publishing a writer outcome; if an attempt reports progress, the operation returns that progress immediately and never hides a later failure by attempting again.
Host readiness or nonblocking refusal is target scheduling state and produces no `WouldBlock` writer outcome. A readiness adapter waits for the next readiness fact and retries the same admitted operation while retaining its operation record and argument loans.
`read_at` performs a positioned read beginning at `file_offset` and never observes or changes an implicit file cursor. It returns `ReadBytes(next)` only for `next > start`. A `file_offset` which the qualified target cannot represent returns `ReadFailed(InvalidInput(...))` before target handoff.
`write_once` never reports an unchanged endpoint after a nonempty host attempt: a host zero-length write is `Err(WriteZero())`.
A short success is not end of input; only `ReadEnd` states that no byte was available at the observed end.
Repetition, accumulation, and retry policy are ordinary source loops over these operations; this specification defines no read-exact, write-all, positioned, or vectored operation.
`directory_next` returns `ListBytes(next, entries)` for the records one admitted batch reported, `ListEnd` exactly when the source reported that the directory holds no further entry, and `ListFailed(error)` otherwise.
A batch ending before `end` is not the end of the directory; only `ListEnd` states that.
A range too small for the target's own next record is reported as a recoverable failure in that target's class rather than as a truncated or partial entry, and the cursor does not advance, so the same handle with a larger range reports the same entries.
No entry is ever split across two attempts and no record is ever reported without its complete name.

Buffer and cursor disposition is exact.
Every successful transfer payload is an absolute endpoint `next`, not a count, and satisfies `start <= next <= end`; the checked program establishes both relations on the matching successful edge [ENT-3.S10].
On `ReadBytes(next)` exactly `[start, next)` may have changed and every other byte of the buffer is unchanged; `ReadFile` has no implicit byte cursor or observation counter to advance.
On `ReadEnd` and on `ReadFailed` no byte of the buffer changes, because an attempt that made progress reports `ReadBytes` instead.
On every recoverable failure of `write_once` and of both copy operations the whole buffer is unchanged.
On `ListBytes(next, entries)` any byte in `[start, end)` may have changed, every byte of the buffer outside that range is unchanged, `[start, next)` is the portable entry-record prefix holding exactly `entries` complete records, and the enumeration cursor advances past exactly the entries those records name.
On `ListEnd` and on `ListFailed` no byte of the buffer changes and the cursor does not advance.
A qualified target binding guarantees that its internal host count is no greater than `end - start`. For `read_at` and `write_once`, only that compiler-owned sanitized count may form `next = start + count`; for `directory_next`, that count bounds the native batch and only the compiler-derived length of the completely validated portable prefix may form `next = start + length`.
A violation is a target/runtime TCB defect [SCOPE-3, QUAL-1], never a source-visible outcome, language trap, or permission to continue with an out-of-range endpoint.

The two copy operations differ only after their two call obligations succeed.
`host_copy_bytes` performs the lossless transfer defined by [SYS-9] and has no failure mode beyond `CopyTooSmall(required)`.
`host_copy_utf8` first validates and measures the encoding and returns `Utf8CopyInvalid()` or `Utf8CopyTooSmall(required)` without writing any byte, and only then copies the complete encoding.
A successful copy returns `Ok(next)` where `next = start + required`, changes exactly `[start, next)`, and leaves the rest of the buffer unchanged.

[SYS-9] `Args` is an immutable entry value.
`args_count` and `arg_get` borrow it and leave it live, and no operation changes its source-visible state.
`arg_get` returns one inline opaque `HostString` lease with no allocation and no byte copy; several leases may refer to the same immutable bytes, and `InvalidIndex()` returns no value.
`args_count` is total.
`arg_get` returns `Ok` exactly when `position` is less than the count `args_count` returns for the same `Args`, and the checked program retains that relation [DIAG-2].

`HostString` refers to immutable target-native code units and owns a lease over them rather than the storage itself.
Source code cannot expose, index, or mix those native code units.
It reads them by exactly the two routes [HOST-2] fixes: the lossless route is `host_bytes_len` and `host_copy_bytes`, and the text route is `host_utf8_len` and `host_copy_utf8` [SYS-8].
`host_bytes_len` is total; the text route is fallible because conversion to text is fallible.

The lossless route's contract is [HOST-2]; it is defined here over a target family whose native code unit is exactly one byte.
On such a family `host_bytes_len` returns the exact count of the host string's native bytes, and `host_copy_bytes` transfers exactly those bytes with no validation and no Unicode restriction, so a host string that is not valid UTF-8 reaches source exactly as given.
That count is exactly the `required` length a `host_copy_bytes` on the same host string reports, so a `host_copy_bytes` whose `end - start` is at least that count returns `Ok(start + required)`, and the checked program retains that relation [DIAG-2].
For a target family whose native code unit is wider than one byte, what these two operations count and transfer is fixed by that family's target qualification; this specification defines it for no such family.
A qualification that narrows the result to what one string domain can carry, or that transcodes it silently, does not satisfy the lossless contract these two operations state.
The text route is defined on every qualified family.
On `Ok(length)`, a `host_copy_utf8` on the same host string neither returns `Utf8CopyInvalid()` nor, for an `end - start` of at least `length`, returns `Utf8CopyTooSmall(required)`, and the checked program retains that relation [DIAG-2].

`relative_path`'s construction, consumption, and retyping semantics are [PATH-1]; `PathInvalid()` returns no value and returns no `HostString`, and neither outcome allocates or copies a byte.

The one-host-string-type rule, the command-lifetime backing, the distinct owned-backing type for any other producer, and the no-implicit-retype consequence are [HOST-3]; release is a logical consume with no target call [SYS-5].
No system value stores an ordinary source borrow or needs a runtime handle-table lookup.

[SYS-10] `FileFactory`, `FilePermit`, and `DirectoryRead` are ordinary affine opaque values; none is a writer-visible capability category.
Program start supplies one `FileFactory` only when the entry selects `command.files`. `reserve_file` takes a call-scoped `&uniq FileFactory`, exhibits `reads(factory), writes(factory)`, and returns one fresh `FilePermit`. The factory loan ends when that inline operation returns, so a caller may reserve several permits through short sequential loans and then move those permits into independent long-running opens.

A `FilePermit` authorizes exactly one attempt by `open_read`, `open_file`, `open_directory`, or `open_directory_source`. Each operation takes `permit: own FilePermit`, exhibits `reads(permit), writes(permit)`, and consumes it on every success or recoverable-failure outcome. This first slice never returns or recycles the permit. Reserving it promises no native descriptor, handle-table entry, kernel memory, or host quota: host exhaustion remains the ordinary `ResourceExhausted` member of the open operation's typed `IoError` result.

`DirectoryRead` is a stable directory-selector resource with one live state. It is live from its entry binding or from the `open_directory` that created it until its release. This specification declares no duplicate, split, or explicit close operation for it.

`open_read`, `open_file`, and `open_directory` borrow `DirectoryRead` through `&` as `root`; `open_directory_source` borrows it through `&` as `directory`. They exhibit `reads(root)` or `reads(directory)` and do not change that value. The changing observation occurrence belongs to the consumed `FilePermit`, not to hidden mutation behind the shared directory borrow [EFF-5]. The directory owner remains live on every outcome.

On success, `open_read` and `open_file` each return one fresh `ReadFile`, `open_directory` returns one fresh `DirectoryRead`, and `open_directory_source` returns one fresh `DirectorySource`. The returned owner has its own ordinary binding identity, carries no parent relation to the directory argument, and can be released without changing that argument. Two values may still contact the same host directory or file; nothing infers host separateness from a native handle or separate open [EFF-5].

Its completion policy is release-complete [SYS-5], on the same ground as `ReadFile` [SYS-11]: losing a close diagnostic on a read-only directory state cannot invalidate an already opened file and cannot promise durability.

Two opens using two distinct owned permits may overlap through shared loans of the same `DirectoryRead` when their other ordinary loans, data dependencies, and exits permit it. Reserving those permits does not keep a long factory loan alive. Operations on distinct directory values may likewise overlap even when the host environment later makes those values contact one physical object.

`FileFactory` and `FilePermit` have proof-only target representation. `reserve_file` performs no host call and returns a harmless opaque value; a qualified open wrapper consumes that value in the checked program but passes no extra argument to the native open facility. Thus explicit authority changes source ownership and effect facts without adding a native open hot-path ABI argument [QUAL-3]. Both types release by logical consume with the empty row [SYS-5].

Resolution, process-equivalence, the no-emulation qualification rule, and the deferred confined form are [PATH-2].

A `DirectoryRead` value returned by `open_directory` names the object the target's own directory-relative resolution reached for that component, with the process equivalence and the deferred confinement [PATH-2] already fixes.
Two `DirectoryRead` values may denote the same directory object however they were produced, and a program that descends must exclude the self and parent components itself: nothing in this specification detects a cycle.

[SYS-11] `ReadFile` is a random-access state resource with one live state.
`open_read` and `open_file` each return one fresh ordinary owner. Separate owners remain distinct Whitefoot places even when the host environment makes them contact the same filesystem object [EFF-5].
`read_at` is call-scoped, takes `&ReadFile` and `&uniq buffer<u8>`, exhibits `reads(file, destination), writes(destination)`, and leaves both owners live on every outcome. The explicit offset removes an implicit byte cursor, so the operation observes but does not advance the `ReadFile` state. Its positioned transfer and buffer semantics are [SYS-8].
Several reads through the same `ReadFile` may overlap when their destination loans and other ordinary dependencies permit it. Sequential access is written by advancing an explicit offset from typed outcomes; a persistent read-ahead Source is a separate system type rather than hidden state in this type. Environment-created changes to the same physical file do not merge or mutate Whitefoot places [EFF-5].

`ReadFile` is release-complete [SYS-5].
Compiler-derived release consumes the resource and may discard only a close diagnostic, which carries no guarantee about bytes already observed and no durability guarantee.
This specification declares no separate explicit-close operation.
A later consuming close may expose that diagnostic, but it must consume the owner on every outcome and may not change derived-release semantics.
Whole-process abort relies on operating-system teardown [SYS-5].

[SYS-12] `Output` is a state resource with one live state.
The standard output and standard error entry bindings supply separate affine owners. Host redirection may make them contact one physical sink, but environment aliasing does not merge their Whitefoot places or introduce a cross-value order [EFF-5].
`write_once` takes `&uniq Output`, reads the current output state and supplied payload, and exhibits `reads(output, source), writes(output)`. The exclusive loan remains live until `loan-released(output)`, so another write through the same `Output` begins only after the first operation has completed its access. Source order on one value therefore follows ordinary ownership without an ordered queue or a second attribution mechanism. An `Ok(next)` means exactly that the local output facility accepted the prefix `[start, next)`: it promises neither line atomicity, peer acknowledgement, nor storage durability.

Writes through distinct `Output` values may overlap under [PAR-1]. Every failure reported for one call reaches that call's typed outcome; target capacity pressure remains internal `wait-capacity`, not `WouldBlock`.
`Output` is release-complete [SYS-5]: compiler-derived release only detaches the source state and reports nothing.
It does not close the host descriptor, it does not flush, and it makes no target call; operating-system process teardown closes the native descriptors afterwards.

That policy has one stated limitation.
A failure a host surfaces only at descriptor close or at writeback — delayed allocation, a network filesystem, a late out-of-space condition — is outside this specification's error model and can be lost, so a redirected command may return a successful `ExitStatus` after a failed writeback.
This is a stated limitation of the type contract, not a silently weakened guarantee.
Strengthening it is a later buffered or durable output type, which is completion-required [SYS-5] and must expose its own flush or finish operation; it does not inherit this policy.

A broken pipe reaches `write_once` as `BrokenPipe` through the bootstrap signal normalization [QUAL-3] fixes; a deployment the bootstrap does not own obtains an equivalent guarantee under its own qualification [QUAL-3].

Terminal control, color, and console mode require separate system state types that this specification does not declare.
The mandatory trap record uses its own runtime channel [SCOPE-4, DIAG-3]; it never flushes an `Output` and source code cannot reach it.

[SYS-13] `ExitStatus` is an opaque immutable value carrying one portable command code.
`exit_status(code)` is its one constructor: it is total and pure, every `u8` is a valid command code, so the closed code range is 0 through 255 and there is no failure outcome, no allocation, no host call, and no state effect.
`ExitStatus` is release-complete and its release is a logical consume [SYS-5].

The type is opaque rather than an alias for `u8`.
There are no implicit conversions [TYPE-4] and every value's type is exactly what its producer fixes [TYPE-5], so without a stated constructor the command entry's returned value would be unwritable; keeping the type distinct also keeps an arbitrary integer from being returned as a command status, and matches how every other system type is fixed [SYS-2].

The target maps the returned code exactly onto the host process status.
Startup failure before entry and a trap are outside this mapping [PROG-3]: a trap performs no language cleanup and returns no status [EFF-4, SCOPE-4].

[SYS-14] `DirectorySource` is a state resource with one live enumeration state.
`open_directory_source` consumes one `FilePermit`, takes `&DirectoryRead`, exhibits `reads(permit, directory), writes(permit)`, and on success returns one fresh `DirectorySource` over the directory object the supplied value names. A separate call with a separate permit returns a separate ordinary owner. Environment aliasing may make two Sources enumerate the same physical directory, but it does not merge their Whitefoot places [EFF-5].

`directory_next` is call-scoped, takes `&uniq DirectorySource` and `&uniq buffer<u8>`, exhibits `reads(source, destination), writes(source, destination)`, and leaves both owners live on every outcome; its transfer, cursor, and buffer semantics are [SYS-8]. Only one call through one Source may be pending because the exclusive source loan remains live until `loan-released(source)`. Calls through distinct Sources may overlap under [PAR-1].
It reports the entries the host reported, in the host's own order: this specification fixes no enumeration order, promises no stability across two enumerations of the same directory, and states no relationship to a concurrent change of that directory's content.
A program that needs a deterministic order sorts what it collected.

The reported entries are exactly what the target's directory holds, including the self and parent entries when the target's directory holds them.
They are not filtered, because filtering them would cost a second host call in the batch that held only them [QUAL-3], and a program that descends must exclude them anyway to terminate.

The target shim may rewrite the transferred records in place within the caller's validated range into the portable form; that rewrite is part of the one transfer, not a copy of the transferred data [QUAL-3].
One entry record is one `kind: u8`, one `name_length: u16` encoded in little-endian byte order, and exactly `name_length` name bytes.
The closed kind set is `0` unknown, `1` regular file, `2` directory, `3` symbolic link, and `4` other; `0` states that the target classified the entry at enumeration time as nothing more specific, not that the entry is absent or unreadable.
A name is one path component: it is never empty, never longer than the target's component limit, and contains no NUL and no target separator, so no record a program reads can name more than one component.
The component limit used by this version's Darwin-family approved implementations is 1023 bytes, and the limit used by its Linux-family approved implementations is 255 bytes [QUAL-1].

An entry name reaches source only as those bytes.
This specification declares no operation turning an enumerated name into a `HostString` or a `RelativePath`, because a name's backing is not the command-lifetime argument snapshot [HOST-3] and a path value is an inline lease over that snapshot [PATH-1].
`open_directory` and `open_file` therefore take a caller-owned name range rather than a path value, and path composition remains the DEFERRED addition [PATH-1] states.
Each call first discharges [SYS-8]'s two static range obligations; neither operation has a runtime range check or `traps` effect.
Each then validates `[start, end)` as one component before any host call: a component that is empty, longer than the target's component limit, or containing a NUL or a target separator yields `Err(InvalidPath(code: 0_u32, origin: 0_u8))`, no host call, and no resource value.
A valid range for which the directory-relative open itself fails yields the target-mapped [SYS-7] error, as `open_read` does.
After `open_file` obtains a provisional descriptor, descriptor-status inspection is required before publication: inspection failure returns its target-mapped [SYS-7] error, a successfully inspected directory returns `Err(IsDirectory(code: 0_u32, origin: 0_u8))`, and every other successfully inspected non-regular object returns `Err(Other(code: 0_u32, origin: 0_u8))`.
Before returning any of those post-open errors, `open_file` makes exactly one native close attempt, discards its close diagnostic without retry as [SYS-5] requires, and returns the inspection or synthetic classification error unchanged.
On success `open_directory` returns an independent `DirectoryRead` for the named directory and `open_file` returns an independent `ReadFile` for the named regular file; a symbolic link is not followed by either operation.

`DirectorySource` is release-complete [SYS-5].
Compiler-derived release consumes the resource and may discard only a close diagnostic, which carries no guarantee about entries already observed.
This specification declares no separate explicit-close operation, and a deep traversal therefore holds one descriptor per live level.
Whole-process abort relies on operating-system teardown [SYS-5].

## 18. Obligation discharge: claims, entailment, and provenance (normative)

[CLM-1] `claim name: e because "text";` is the sole writer-spelled runtime boundary for a proof residual which the normative checker cannot derive.
It is not an assertion, abort, conditional, test oracle, debug check, or general invariant facility.
The author asserts that `e` is true on every execution reaching the statement; if it may legitimately be false, source must instead use ordinary `if`, `match`, loop transfer, typed result, return, or command status.
`e` must have exact value mode and type `own Bool` under the [OP-5] condition judgment, including TYPE-7 implicit-read exclusivity, and must additionally satisfy the claim-proof-predicate judgment below.
An exact-mode or exact-type failure cites CLM-1 at the selected `expr` node and its complete checked half-open extent.

A claim proof predicate is one finite direct goal [ENT-2] whose evaluation is total, deterministic, observational, non-consuming, and ownership-neutral.
It may contain typed literals, named constants, non-consuming reads of live copy places, fixed-length observations, and compiler-known total non-trapping integer, float, Boolean, conversion, reinterpretation, enum-equality, and allocation-fit predicate rows, recursively under the same restriction.
It may not contain a user or system call, subscript, proof-required exact operation, checked-result operation, allocation, construction, write, move, borrow or reborrow, consuming projection, residual drop or cleanup, release, block, external operation, nested claim or trap, or any other partial, effectful, ownership-changing, or potentially nonterminating computation.
A rejected shape cites CLM-1 at the predicate `expr`; the checker never accepts it merely because its inferred effect row is `pure`.

The decoded `because` STRING is exactly five LF-separated lines, with no leading or trailing extra line, in this order:

```
premises: nonempty text
derivation: nonempty text
conclusion: nonempty text
checker gap: nonempty text
consumers: nonempty text
```

Each fixed label and following ASCII space is exact and each value remains nonempty after removing leading and trailing ASCII spaces.
The five fields are retained review data [DIAG-2].
This structural check does not prove their prose true: owner approval of the checker-accepted source requires human, AI-assisted, or offline-proof review to validate the stated premises, derivation, exact conclusion, checker gap, and one or more authentic terminal consumers.
Such review may use only facts valid before this claim, including explicitly named earlier reviewed claims; it may not use this claim's own successful execution, a later fact, an unstated caller or environment promise, a user callee's body or unstated system behavior in place of a verified or specification-fixed callable-boundary fact, or a circular occurrence of the same dynamic claim.
An optional solver result never changes ordinary source acceptance or runtime execution [ENT-1].

After its type, proof-shape, per-function name, and five-field formation judgments succeed, CLM-1 performs one fact-free canonical-formation subjudgment before any claim truth, contradiction, provenance, or residuality query.
Let `D(P)`, `S(P)`, and `F(P)` be three exact typed goal images of predicate P.
`D(P)` is the direct GoalExpression of the evaluated written predicate.
`F(P)` is the unique complete still-valid ordinary-let origin expansion of `D(P)` under the ordinary goal-origin rule below.
`S(P)` is the support-canonical snapshot-frontier expansion: starting from `D(P)`, perform the same unique still-valid ordinary-let replacement as `F(P)`, except that when the current subtree already has an exact L0 projection or one fixed normalization, retain that subtree unchanged and do not expand any datum below it; expansion continues everywhere else.
Thus S preserves the support of each checker fact the claim actually reads, while F records the fully structural origin.
The ordered exact-image inventory keeps the first occurrence of each distinct identity in D, S, F order; equal images collapse and do not create duplicate queries.

Canonical formation constructs the unique ordered `Contrib(P)` from positive S in the finite [ENT-2] fact vocabulary.
The recursive walk visits operation arguments left to right, fixed normalizations visit components in their rule-defined order, and the result keeps only the first occurrence of each exact signed-goal or normalized-relation identity; duplicate identities never create duplicate component ordinals.
Difference-bound identities retain ordered endpoints; disequality identities are unordered, so `a != b` and `b != a` deduplicate, while the first left-to-right source occurrence fixes the retained rendering and ordinal.
Positive `band`, negative `bor`, and either sign of `bnot` recursively contribute their sound signed conjuncts.
Positive `bor` and negative `band`, whose truth is disjunctive, remain one exact signed-goal component.
An integer inequality contributes its one normalized bound; positive integer equality contributes its two directed zero bounds, negative equality its disequality, with `ine` dual; a representable positive `.defined` or allocation-fit predicate contributes the relations of its one fixed conjunction normalization.
D is a reconstruction target, not a contribution basis; F is a lifecycle image, not an S3 contribution or reconstruction target.
An otherwise proof-pure exact goal for which the checker has no finer conjunctive fact vocabulary is one opaque component.
`bxor`, Boolean equivalence, a normalization with alternative positive clauses such as signed division/remainder, an ambiguous origin, or any shape for which normalization, support, component negation, S reconstruction, or D materialization is not unique is not an admitted claim predicate in this version and rejects under CLM-1 before a component ordinal is published.
Formation alone assigns the finite component ordinals; it establishes no fact and makes no lifecycle classification.

At the claim point, CLM-1 then queries every component's ordinary S-derived support [ENT-5] in component-ordinal order against [ENT-6]'s frozen claim-authority state.
Every runtime value component and holder read by that support must be `Local` to the current function.
If any support member is `BoundaryResult`, the whole claim is non-local and rejects under CLM-1 using [DIAG-1]'s least component, earliest boundary witness, and first canonical support carrier that observes that witness; no S3 source, lifecycle query, `Eligible` member, counterfactual run, ClaimLedger record, or lowering authority is formed for that occurrence.
A verified `ensures` and its S12 publication never make the returned value local: the caller consumes the verified relation directly and cannot restate or strengthen it with a claim.
This authority admission is independent of truth and of [PRV-1] provenance; a `Local` component is not thereby true or internal, and a PRV-internal call result is still `BoundaryResult`.

Every claim accepted by [CLM-2] is retained as one runtime check in every build mode, is never elided, and evaluates `e` exactly once at every dynamic reach.
False evaluation emits the required record [DIAG-3] and aborts [SCOPE-4, EFF-4] before S3 can authorize a later operation; true evaluation continues and establishes only [ENT-3]'s canonical claim contribution.
A `claim_stmt` syntactically exhibits `traps` [EFF-2] and does not count as delivery or must-divergence [GIVE-1].

The claim name is one IDENT and is not a declaration: it enters no [TYPE-6] domain, no [OP-1] reservation inventory, and no lexical lookup, and no source construct references it; its [DIAG-1] carrier classification is the claim-name carrier.
Because the name is outside the reservation inventory, a claim may be named `len` or `wrap`, while the retired spelling `trap`, `claim`, and every exact fixed lowercase grammar atom remain unwritable as IDENT [FORM-3] — a chosen asymmetry (owner ruling 2026-08-07), not an accident.
Within one `fn_decl` every claim name is unique; a repeated spelling is a hard error citing CLM-1 at the later `claim_stmt` node.
The required labels and nonempty values select source formation, but the fields are absent from runtime behavior and their prose truth establishes no checker fact.
A claim is legal in exactly the statement positions [GRAM-4] admits; a `contract_block` contains only `contract_define`, `requires_clause`, and `ensures_clause` productions, so no claim or other statement can appear there.
Operand provenance does not by itself prove a claim true: [PRV-2] and [PRV-3] still reject claim-only authorization of an unconditionally external constrained subject, while CLM-1 independently requires local authority and [CLM-2] independently requires a genuine admission consumer.

[CLM-2] One FN-1-reachable concrete claim occurrence c is judged only after CLM-1 has admitted its predicate, canonical D/S/F images, ordered `Contrib(P)`, and local authority, and after evaluating its predicate but before its own S3 source.
If the pre-S3 combined state is contradictory, c is vacuous and rejects; contradiction is tested first among lifecycle truth queries, after CLM-1 admission and before either predicate sign, and ex-falso never proves claim truth or erases a locality failure.
Otherwise query both signs of every image in that ordered inventory.
Both signs at one image are a compiler consistency failure, not a source classification.
A positive sign at one image and a negative sign at a distinct image, with no one image deriving both, make the source vacuous because its support-correct exact images conflict.
Otherwise deriving any positive sign rejects c as redundant and deriving any negative sign rejects it as refuted.
Thus `claim True()` is redundant and `claim False()` is refuted on every reachable path.
Checker strengthening may and must turn a formerly unknown claim into this source-upgrade error; the author removes or restructures the source and recompiles, and no compiler or optimizer silently elides the written check.

For a remaining unknown predicate P, component lifecycle consumes CLM-1's already formed `Contrib(P)` and walks S and F in lockstep through the same signed conjunctive Boolean structure.
At one S snapshot-frontier leaf, the ordered components extracted from S are the contribution identities.
When F has the same ordered component cardinality, the corresponding F component is an equivalent manifestation of the S component.
When the S leaf contributes exactly one component, the exact signed F leaf is also an equivalent manifestation.
When the S leaf contributes more than one component, positive proof of the exact signed F leaf is a positive-only manifestation of each component: it proves every conjunct, but the opposite sign of the whole F leaf refutes no particular conjunct.
For one component, either sign of an equivalent manifestation participates in lifecycle classification, while a positive-only manifestation participates only positively.
Both signs at one manifestation are a compiler consistency failure; positive and negative results obtained only across distinct equivalent manifestations make the source vacuous.
Every component must otherwise be unknown on both signs in the non-contradictory pre-S3 state.
A pre-proved component rejects c for overlap even when no complete predicate image was derivable; a pre-refuted component rejects it as inconsistent.
Tentatively adding every component must remain non-contradictory, derive positive S, and not derive negative S through [ENT-4]'s retained ordinary derivation.
Retain that positive S proof; when D differs from S, materialize positive D from it under D's own ordinary support and require the resulting trial to remain non-contradictory.
CLM-2 neither requires nor performs reconstruction of F.
S3 establishes the contribution components directly, not P followed by a decomposition.

Let `Eligible` be the fixed source-ordered set of concrete occurrences that passed FN-1 reachability, every CLM-1 formation and locality judgment, exact lifecycle, component lifecycle, consistency, and reconstruction.
If any occurrence fails one of those earlier judgments, the unit reports the deterministic earlier error and residuality does not run; an invalid occurrence never supplies another candidate's baseline S3.
For each c and each component a, `Full-minus(c,a)` repeats the same whole-program proof analysis with every other Eligible S3 source and c's other components unchanged, while c still evaluates, exhibits the same effects, and retains its runtime statement, but a's one component-specific S3 source event is withheld.
Closure is recomputed from unmasked sources, so every fact depending only on a disappears and an independently rederived identical fact remains.
`Full-minus(c)` analogously withholds all of c's S3 contribution.
When `Contrib(P)` has exactly one component, `Full-minus(c,a)` and `Full-minus(c)` suppress the identical one-event source set and are definitionally the same fresh analysis; an implementation performs that analysis once and records the same stable result under both the component and whole-occurrence evidence roles.

The terminal admission roots are exactly the four proof-required operation families [ENT-6], ordinary [FN-8] call requirements, and mandatory complete [FN-9] selected-return aggregate proofs.
A protected operation or call root is eligible only after its attached ordinary provenance gate succeeds in Full; that gate is retained audit data, not an independent counterfactual root.
Optimizer or observational S7/S11/S12 metadata, effect exhibition, CLM-3 structure, another claim's lifecycle, a test oracle, or a fact with no such terminal root is not a consumer.
Every component a must have at least one terminal root that succeeds in Full and fails in `Full-minus(c,a)`, and at least one terminal root must likewise fail in `Full-minus(c)`.
The Full proof must reach a's exact S3 event and contain no contradictory or ex-falso predecessor; its query state must be non-contradictory.
At a join, every reachable predecessor contributing proof support must independently be legal and non-explosive, while c need dominate only the c-dependent lineage rather than a mutually exclusive sibling route.

Residuality is one simultaneous classification over fixed Eligible, never a fixed point selecting a survivor among alternatives.
It proves checker-relative component and occurrence irredundancy, not a unique proof basis, minimum claim count, mathematical weakest theorem, or authentic author intent; the latter two remain mandatory five-field review duties [CLM-1].
A component or occurrence with no qualifying root rejects as non-residual.
Only occurrences passing every judgment have disposition `retained`, contribute S3, enter CLM-3 and the ClaimLedger, and lower through the unchanged runtime path.

[CLM-3] Any source `fn_decl`, generic or nongeneric, may carry the one optional fixed terminal `deny_claims` before its optional `program_kind`.
That terminal is ineligible for IDENT under [FORM-3].
Each marked concrete [FN-2] instance is one strict root.
The marker is compile-time policy only: it adds no effect, trap, runtime check, fact, type, mode, region, call convention, body, or lowering, and it neither removes nor changes any [CLM-1] claim.
A declaration without the marker is no strict root.

After every ordinary semantic, provenance, and CLM-2 residual judgment succeeds, form the finite concrete ordinary-user-call graph already used by [FN-9], retaining every checked call occurrence in source NodePath order, including calls and retained claims in structurally checked arms irrespective of optimization.
Take the same SCCs and callee-before-caller condensation.
One direct claim identity is exactly `(concrete function instance, claim_stmt NodePath, claim name)` for a CLM-2-retained residual occurrence.
`DirectClaims(K)` is the union of all such identities in component K.
In callee-before-caller order, `MayClaims(K)` is `DirectClaims(K)` union the `MayClaims` set of every strictly outgoing callee component.
Sets are ordered by stable concrete-instance order, then NodePath, then name.
The closure of one strict root is exactly its root component plus every component reachable along outgoing edges; it includes the whole of each reached SCC and never follows an incoming edge into an unrelated caller.

A demanded component succeeds strictly exactly when its `MayClaims` set is empty, every ordinary user-call requirement owned by the component discharges at that call in caller U [FN-8], and every strictly outgoing demanded callee component has a successful strict summary.
No separate operation-obligation query is a source judgment: U differs from the complete state only by S3 [ENT-6], a complete-only callee summary is exactly a claim-dependent one [FN-9], and an empty `MayClaims` set therefore already implies that every obligation this component's complete-state judgment discharged also discharges in U.
Calls inside one SCC consume no same-SCC summary; all members succeed or fail atomically.
These are finite queries over the already-produced view and DAG, not a body rewalk or another fixed point.
Component summaries are silent.
For one marked root, a nonempty `DirectClaims` set in its own SCC rejects at the least direct claim node; otherwise the first call in stable caller-instance then call-NodePath order within that SCC whose strictly outgoing callee component has nonempty `MayClaims` rejects there as an imported-claim event.
A direct claim in a reached unmarked component is therefore reported at the root-facing importing call, while a claim in the marked root SCC is reported at its own node.
When a downstream component instead fails a non-claim U judgment, only the actual FN-8 call is reported; no caller-side summary event is created.
Multiple roots use the stable concrete-instance order fixed by [DIAG-1].

An ordinary caller outside the closure that calls a marked root remains ordinary but must also discharge every requirement of that root in its own U state [FN-8].
A marked command entry has no contract [FN-7], so its strict judgment consists only of its own and its reachable callees' claim and call-requirement judgments; this specification defines no foreign adapter.
Claim import is tested before a strict FN-8 judgment at the same call.
All strict roots and candidate S12 or delivery facts remain unpublished in one failure-atomic batch; any CLM-3 or strict FN-8 event discards that batch and the prospective checked program.
Strict acceptance reads valid retained claim occurrences and call metadata directly from semantic scratch, never the checked-program `ClaimLedger` [DIAG-2], which is constructed only after successful finalization.

[ENT-1] The entailment fragment is a closed, deterministic, search-free derivation system fixed completely by this specification.
Its state is the L0 relation state plus [ENT-2]'s finite signed opaque goals.
The fixed judgments in this section are source-acceptance judgments: complete-state obligation discharge [ENT-6], claim proof-predicate and canonical formation, claim-authority admission, exact and component lifecycle, contribution reconstruction, and individual residuality [CLM-1, CLM-2], ordinary-call requirement discharge [FN-8], verified normal-return proof and view classification [FN-9], provenance classification [PRV-1], the call-argument gate [PRV-2], and the local constrained-subject gate [PRV-3] are post-resolution semantic judgments under [DIAG-1], identical in facts-on and facts-off compilation, and are not an optional optimizer-fact family. [SCOPE-2] is unchanged: every fact source [ENT-3] is an executed control condition, an executed retained residual claim, a requirement statically proved by every ordinary caller before S4 admits it to a body, a declared allocation or type property, a constant, S11's compiler-owned structural consequence, or S12's machine-verified normal-result publication.
No source postcondition is trusted: FN-9 proves every selected exit, requires a nonempty selected-exit set, withholds same-SCC summaries, and subjects every candidate caller fact to the ordinary FN-8 and PRV gates before atomic publication.
The fragment is the deterministic checker derivation of [OP-4], [FN-8], [FN-9], and [DIAG-2] for the judgments this version attaches; a solver result never participates, and no implementation may strengthen, weaken, time-bound, or randomize the derivable set.
Two conforming implementations derive the same complete, unasserted, and S4-blinded fact states at every applicable point; the same FN-9 selected exits, aggregate dispositions, concrete-SCC order, and S12 establishment set; the same claim-authority component tree, reaching state, support carrier, and boundary witness at every claim; the same [PRV-1] class and symbolic dependency for every component; the same [PRV-2] result, write, demand, target, and event sets; and the same disposition for every obligation, claim, call goal, postcondition relation, local leaf, and call argument.
Every nongeneric source body receives this judgment whether or not `main` reaches it.
Every generic source body additionally receives one source-schema judgment under the one source-canonical symbolic substitution formed during generic-body validation, even when it has no concrete instantiation.
That symbolic inventory includes the declaration's complete source body and recursively installed source-canonical call requirements.
Before residuality, it performs every ordinary OP-2/OP-4/OP-9/SYS-8, FN-8, and expressible FN-9 admission judgment that the schema vocabulary can represent; an unproved ordinary source-body operation is not accepted merely because no concrete instance is reachable from `main`.
It constructs and deduplicates D, S, and F by the same rules, derives `Contrib(P)` only from S, and performs the same component-authority admission before it queries either predicate sign; only an admitted schema then queries D-S-F in order, applies the same equivalent and positive-only F manifestation rules, and reconstructs S then D exactly as a concrete inventory does.
It freezes its own schema-Eligible set after the same CLM-1 formation and locality, contradiction-first lifecycle, component lifecycle, consistency, reconstruction, ordinary-admission, and provenance judgments, then runs the same simultaneous `Full-minus(c,a)` and `Full-minus(c)` analysis against source-schema terminal roots expressible in that symbolic goal vocabulary, with the same component ordering, ancestry, non-explosion, provenance-invariance, and individual-necessity requirements as a concrete inventory.
Its only vocabulary differences are the explicitly specified symbolic datum restrictions; claim authority uses resolved source components and therefore has no generic L0-type exception, and the schema may not collapse the three-image judgment, use F as contribution authority, approximate an unavailable symbolic relation, omit locality, or omit an ordinary judgment that its vocabulary can form.
Generic integer and float type parameters are copy datums only for exact opaque goals in this schema; they are not [ENT-2] L0 fragment types, while a const-generic parameter whose written type is one concrete integer fragment remains the symbolic constant term fixed below.
An FN-8 source-call requirement over those exact datums is a schema terminal root.
An FN-9 source-schema terminal root exists only when its result datum, selected return, and normalized relation are already expressible over concrete integer fragment types; a postcondition whose result or relation depends on a generic integer or float type parameter is rechecked only in inhabited concrete instances and is not silently approximated by an opaque schema root.
The schema judgment publishes no executable function, ordinary summary, or lowering authority, and its stable source-only report is ordered before the same source occurrence's inhabited concrete reports and contains no generated instance, scratch function, or scratch nominal identity.
Every inhabited concrete [FN-2] instance is then rechecked independently after substitution; an entry-uninhabited concrete instance still performs CLM-1 formation and locality but produces no CLM-2 residual report or terminal witness.
A contradictory local path in either schema or concrete flow never supplies a residual witness and never bypasses CLM-1 locality.
If concrete instances disagree, the first invalid concrete instance in stable instance order rejects the shared source occurrence; no instance-specific claim elision exists.
A type or const generic in an ordinary requirement or postcondition is otherwise substituted as its owning rules require, and concrete const-generic terms retain their concrete values.
The fragment joins the trusted computing base exactly as the type and ownership checkers do [SCOPE-3]; a wrong derivation is a compiler defect class, owned by testing, not a language hedge.
Version monotonicity of fact-source and closure strengthening preserves every already-discharged operation, call goal, or selected-return relation, but claims deliberately sit at the proof frontier.
A later normative checker may newly derive a claim predicate, its negation, or one contribution component, or may make its S3 contribution unnecessary; CLM-2 must then reject that source as redundant, refuted, overlapping, or non-residual so the author removes or restructures it.
This is an explicit source-upgrade rule, never authority for compiler or optimizer elision.
Activating [PRV-2] or [PRV-3] for an already attached protected family, attaching a new protected family, changing a [SYS-2] component from internal to external, adding or removing a `BoundaryResult` seed or declassification, or adding a callable publication surface is an amendment-level accepted-set change, not implementation strengthening.
Beyond those classes, this specification adds only FN-9/S12, the two stated unsigned S7 relations, [ENT-6]'s exact integer-domain, allocation-fit, subscript-bounds, and system-range obligation families, and [ENT-5]'s value-if-only delivery, and retains the provenance gate.
No implementation may activate, expand, or reclassify any such judgment independently, and apart from an explicit specification amendment of those kinds no other entailment-fragment judgment may tighten acceptance across versions.

The [CLM-3] strict partition is one additional fixed source-acceptance judgment over the same finite semantic result, not a fact source or optimizer family.
Conforming implementations compute the same concrete ordinary-call occurrences, SCC condensation and callee-before-caller order, declaration markers, strict-root outgoing closures, direct claim identities, `DirectClaims` and `MayClaims` sets, demanded components, existing-U protected-obligation and call-goal dispositions, marked command-entry disposition, and strict success summaries.
The judgment is per concrete [FN-2] instance, uses the already-produced U view and the same function-local derivation DAG, and introduces no solver, flow rewalk, second closure, negative fixed point, or new relation.
Facts-on and facts-off acceptance remains identical.
This partition is an explicit opt-in accepted-set tightening only for marker-bearing roots, plus the FORM-3 reservation of `deny_claims`; it changes no unmarked judgment and is the sole additional tightening authorized beyond the amendment classes enumerated above.

[ENT-2] The fragment constructs one flow state for one concrete function body and proof view at a time.
No caller fact is copied into a callee: an ordinary call judges its instantiated [FN-8] goal in the caller's entering state, the callee body begins with its own proved requirement as [ENT-3] source S4, and only a separately FN-9-verified earlier-SCC summary may establish its instantiated normal-result relation back in the caller.
A fragment type is one member of the closed integer set [OP-2]; relations are over mathematical values, so relations between terms of different fragment types are well-formed and are created only by the sources and flow transports [ENT-3, ENT-5] admit.

A term is exactly one of: (a) a tracked place — a `place` [GRAM-5] whose root `pbase` IDENT resolves to any `let_stmt` binding, a `for_stmt` binder, a `param`, any match binder regardless of its [OWN-13]-derived mode, or a named const [CONST-2], formed with any number of field-selection `psuffix`es and `deref` wrappings and no subscript suffix, whose final selected type is one fragment type; (b) a length term `len(P)`, of fragment type u64, where P is a place formed under the same restriction whose final selected type is `array<T, N>`, `slice<'r, T>`, or `buffer<T>`; (c) a constant — the mathematical value of an integer literal or of an integer-typed named const, or symbolically an in-scope integer-typed const-generic parameter; (d) one of the two compiler-owned u64 capture terms belonging to an admitted `for_stmt`, identified exactly by `(that for_stmt's NodePath, lower)` or `(that for_stmt's NodePath, upper)`; (e) the one compiler-owned symbolic result datum of an admitted FN-9 clause while its RelationTemplate is formed, identified by that `ensures_clause`, its route or unrouted class, and fragment type; or (f) the distinguished zero term Z, used only to carry constant bounds, S7's exact mathematical-zero disequality, and [ENT-6]'s normalized integer-domain components.
The FN-9 result datum occurs only in its template: every selected-return or caller query substitutes it with one ordinary term or constant before flow, so it never enters a body state, survives a return, or creates runtime storage.
Two places are the same term exactly when their roots resolve to the same declaration event [TYPE-6, DIAG-1] and their canonical source spellings [FORM-2] are byte-identical; a fresh binding legally reusing an expired spelling is a distinct term, and distinct spellings are distinct terms even when they resolve to overlapping storage.
Term identity thus under-approximates aliasing, while kills [ENT-5] use [OWN-7]'s resolved-place overlap relation and over-approximate it.

After TYPE-5 succeeds, each `for_stmt` endpoint atom is admitted only when its evaluated value is itself one preceding term or constant.
Any other atom is a hard error citing ENT-2 at that endpoint's `atom` node, with `SourceCoordinate` equal to its complete checked half-open source extent and the restructuring `bind the computed u64 value with one preceding ordinary let and use that term as the endpoint`.
In particular a subscripted place is not made a term by endpoint position.
The two capture terms are finite, immutable, compiler-owned, and not source bindings or source places: source cannot name, write, borrow, move, or shadow them.
Their scope begins after their respective once-only endpoint captures and ends on every edge leaving the counted construct.
The counted binder's compiler fact scope begins at its initialization and ends on every edge leaving the counted construct, even though [TYPE-6] makes its source name visible only in the body.

An FN-9 parameter datum denotes its function-entry image in the RelationTemplate but creates no snapshot term.
Local proof may reuse the ordinary parameter term only while FN-9's view-independent entry-image stability remains live; caller publication substitutes the corresponding pre-transfer actual image independently for each referenced formal.

A concrete goal is one finite typed expression tree with exact result `own Bool` formed under [FN-8]'s structural identity, either by concrete substitution of a GoalTemplate or by [ENT-3]'s goal-origin judgment in the current function.
A concrete place datum retains the resolved root declaration event and its ordered field and `deref` projections; an actual substituted for a borrow formal uses the resolved referent datum, while an own actual uses its pre-transfer datum.
Named consts and typed literals retain the identities FN-8 fixes.
The compiler-owned ephemeral actual-value datum of FN-8 may occur only in the instantiated goal of its one ordinary call.
It has the finite structural identity fixed there, is neither a place nor an L0 term, has no direct or complete ordinary source goal origin, and therefore cannot be established by naming the original subscript again.
Goal equality is exact tree equality and therefore may hold across two requirement occurrences or concrete callee instances only when their substituted typed trees are identical.
The finite goal universe of one concrete function is exactly the goals formed from its written Bool conditions, claims, requirement S4 sources, proof-required operation obligations, ordinary-call requirements, and every canonical CLM-2 contribution child and reconstruction parent after the finite expansions [ENT-3] admits.
Contribution construction may intern only subexpressions, exact signed disjunctive roots already present in a claim predicate, and the claim's D/S/F origin images and manifestations; it synthesizes no arbitrary formula or unbounded algebraic search.

A signed opaque fact is exactly `+G` or `-G` for one concrete goal G, meaning that exact whole expression evaluated respectively true or false.
It carries no child facts merely by existing; [ENT-3] fact sources establish their selected signed contribution and [ENT-4] alone performs the finite parent reconstruction below.
If G's complete root is exactly one comparison origin relation R under [ENT-3], `+G` has the exact L0 projection R and `-G` has R's exact negation; a non-comparison root has no L0 projection.
The signed fact and its projection are distinct manifestations in one combined state and have the supports [ENT-5] fixes.
For CLM-2, all lifecycle manifestations mapped to one canonical S-derived component share one `(claim occurrence, component ordinal)` contribution identity for evidence and ancestry.
Only the component's S3 source event is maskable; F is never sourced, and masking never suppresses an independently established equal relation or goal.

An atomic fact is one difference bound `t1 - t2 <= c` (t1, t2 terms, c a mathematical integer) or one disequality `t1 != t2`.
Difference-bound identity preserves the ordered term pair; disequality identity is the unordered endpoint pair, although the first source-normalization encounter preserves its written orientation for rendering and component order.
Source relations normalize exactly: `a <= b` is `a - b <= 0`; `a < b` is `a - b <= -1`; `a = b` is the bound pair `a - b <= 0` and `b - a <= 0`; `a >= b` and `a > b` swap operands; `a != b` is one disequality.
A constant operand folds through Z: `a <= 7` is `a - Z <= 7`.
Implicit facts hold at every program point: every term t carries the reflexive bound `t - t <= 0`; every term t of fragment type T carries `t - Z <= max(T)` and `Z - t <= -min(T)`; every length term over a place of type `array<T, N>` carries the equality `len(P) = N` (both bounds), with concrete N a constant and const-generic N a symbolic constant term.

[ENT-3] The fact state is defined constructively over the conservative structural normal-control graph [FN-1]: each source below establishes its L0 and signed-goal facts at its stated point; facts flow forward along normal edges; kill events apply on the edges where [ENT-5] places them, with scope-exit kills applied before any join; merge points take the [ENT-5] join and loop heads the [ENT-5] loop rule; and the state queried at any point is the [ENT-4] closure of that flow.
retired: S8
Dominated straight-line establishment is a consequence of this construction, not a second definition.
Nothing else is a fact: an `ensures_clause` is only an FN-9 proof obligation, never a trusted source; no struct invariant, writer-stated or inferred loop induction, inferred summary, or unverified user-function result exists.
S11 is only the compiler-owned consequence of the counted operations [FN-1] actually executes, and S12 exists only from a separately verified earlier-SCC summary under the publication formula below.
Provenance [PRV-1] is a separate judgment over finite value and storage components, not a fact: it establishes and kills no relation or signed goal, and no [ENT-4] answer depends on it.

A comparison origin is defined first.
An expression has comparison origin R when (a) it is a call to one of `ieq`, `ine`, `ilt`, `ile`, `igt`, `ige` [OP-2] whose two operands are each a term or constant, R the corresponding relation over them; or (b) it is a bare IDENT naming a `let` binding of type `own Bool` whose initializer right-hand side satisfies (a) with relation R, no [ENT-5] kill event (a)–(d) applies to a fact supported by an operand term of R on any path from that initializer to the use, and the binding is the target of no `set` on any such path.
No other shape has one: `band`, `bor`, `bxor`, `bnot`, `eeq`, `ene`, user-function results, and deeper indirection chains contribute no L0 comparison origin in this version; an established Boolean goal contributes relations only through the members of its signed decomposition set.

An expression has integer-domain-predicate origin G when (a) it is one total `+defined`, `-defined`, `*defined`, `/defined`, `%defined`, `ineg.defined`, `iabs.defined`, `ishl.defined`, or `ishr.defined` operation with its selected concrete operand type and complete ordered operand-expression identities, G that exact typed GoalExpression; or (b) it is a bare IDENT naming an own-Bool ordinary-let binding whose initializer satisfies (a), no [ENT-5] kill event applies to G's support on any path from that initializer to the use, and the binding is the target of no `set` on any such path.
This origin is one ordinary exact goal, not a second fact channel.
Its support, expansion, kills, scope exit, joins, and signed establishment are the ordinary goal rules below.

A Bool expression has a direct goal origin G when its completely typed expression consists only of non-consuming place datums, typed literals, named const datums, and calls to or infix spellings of pure, total, non-trapping operation-table rows, with exact tree identity as [FN-8] fixes.
Construction, a user-function or system call, a subscript, a move or borrow, a trapping or partial operation, and any other expression shape has no goal origin.
Starting from a direct goal, its complete origin expansion recursively replaces an ordinary-let datum by that binding's unique defining right-hand side exactly when the right-hand side itself has such a typed pure/total origin, the binding is no `set` target on any path from that initializer to this use, and no [ENT-5] kill event applies to the replacement's support on any such path.
Expansion continues to a fixed point and is all-or-nothing for every eligible leaf; it never performs an algebraic rewrite.
The goal-origin set is the direct goal plus that one complete valid expansion when it differs.
Thus a condition binding's own Bool value and its still-valid computation origin are both retained: a later write to an origin place kills the expanded goal but not the already-computed binding goal, while a write to the binding kills the latter normally.
Definition expansion in FN-8 is unconditional because every `contract_define` is erased pure proof syntax and the admitted block contains no mutation.

Signed Boolean decomposition applies at every ordinary non-claim establishment of a signed goal fact by the sources below.
The decomposition set of `+G` whose complete root is `band(A, B)` is `+A` and `+B` together with each member's own decomposition set; the decomposition set of `-G` whose complete root is `bor(A, B)` is `-A` and `-B` together with each member's own decomposition set; the decomposition set of either sign of `bnot(A)` is the opposite sign of A and its recursive decomposition; `-band` and `+bor` remain exact disjunctive roots and have no child member.
Every admitted member whose root has an exact comparison projection also establishes that signed projection.
Each member has its own [ENT-5] support, kills, joins, and loop treatment.
This is a finite structural walk with no algebraic rewrite.

CLM-1's `Contrib(P)` formation applies the same signed walk to S, but is a source-admission basis rather than an automatic parent establishment.
It replaces a projected positive equality with its two directed bounds and a projected negative equality with its disequality; it similarly expands one available fixed positive conjunction normalization into its ordered relations.
An exact disjunctive or otherwise opaque residual remains one signed-goal component only when CLM-1 admits that root class.
S3 establishes these components directly under separate contribution identities.
ENT-4 reconstructs S from its components and then materializes D when distinct; F remains lifecycle-only, so ordinary proof consumers receive no claim-specific shortcut.

The sources are:

[ENT-3.S1]
- S1 (branch facts).
At an `if_stmt` or `value_if`, each goal G in the condition's goal-origin set is established as `+G` at the then-block's entry and `-G` at the else-block's entry; for an else-free `if_stmt`, `-G` is established on the false edge, which joins the then exit at the continuation [ENT-5].
Independently, when the condition has comparison origin R, R is established at the then entry and R's exact negation at the else entry or false edge.
L0 negation is exact over mathematical integers: the negation of `a - b <= c` is `b - a <= -c - 1`; the negation of `a = b` is `a != b` and conversely.
[ENT-3.S3]
- S3 (claim facts).
After one CLM-2-retained `claim n: e because "…";` evaluates true [CLM-1], establish each ordered component of `Contrib(e)` directly on the normal continuation, with its exact signed-goal or L0 relation manifestation and one component-specific S3 event.
Do not establish D, S, or F before the components.
After all components are established, close under [ENT-4], retain the reconstructed positive S proof, and when D differs from S materialize positive D from that proof under D's own support.
F remains lifecycle-only and receives no S3 source or reconstruction event.
No S3 source exists for an early-invalid or non-residual claim.
[ENT-3.S4]
- S4 (requires facts).
At a concrete function-body entry, its complete instantiated [FN-8] goal G is established as `+G`.
When and only when G's complete root is one comparison call admitted by comparison-origin shape (a), whose operands after template and call substitution are each an admitted term, constant, or `len(P)` length term, that exact relation R is also established.
Beyond that projection, only the members of G's signed decomposition set and their projections are established; no other child of any goal is established.
S4 is the admitted-body axiom justified by every ordinary caller's static discharge; no callee-entry prologue or boundary check executes.
[ENT-3.S5]
- S5 (copy and conversion equalities).
An `ordinary_let_rhs` establishes at its binding: for `let x = lit;`, x = value(lit); for `let x = p;` with p a term of type T, x = p; for `let y = cvt<Src, Dst>(p);` with (Src, Dst) a total pair [OP-6] and p a term or constant, y = p — `cvt` keeps its written type pair [TYPE-5].
A successful [SET-1] commit to a direct fragment-typed place applies the same three-row image to the post-write destination: `set x = lit;` establishes x = value(lit), `set x = p;` establishes x = p, and `set x = cvt<Src, Dst>(p);` for a total pair and term or constant p establishes x = p.
The right-hand side is evaluated first, then [ENT-5] kills every fact supported by the old target value, and only then is this post-write equality established; the equality therefore cannot carry an old target fact through the write.
An array- or buffer-index target, a non-fragment target, a narrowing conversion, and every other computed right-hand side establish no S5 commit image in this version.
[ENT-3.S6]
- S6 (length facts).
`let b = buffer_new(n, v);` and `let b = buffer_vacant<T>(n);` each establish len(b) = n on the normal continuation [OP-9], n read as term or constant.
`let m = len(P);` for a tracked P establishes m = len(P).
`let s = slice_of…(&'r P);` for a tracked P establishes len(s) = len(P).
[ENT-3.S7]
- S7 (constant-offset arithmetic).
For `let s = p +wrap k;` with p a term of type T and k a constant in either operand position, when the closed state at that point derives `min(T) <= p + k` and `p + k <= max(T)` (as bounds on p through Z), s = p + k is established; `p -wrap k` with constant k establishes s = p - k under the dual range condition.
For proof-required exact `p + k` and `p - k` with constant k, s = p ± k is established on the normal continuation unconditionally after source acceptance: that exact site's discharged IntegerDomain obligation is the proof [OP-2, ENT-6].
For a `match` whose scrutinee is directly `p +checked k` or `p -checked k` with constant k, or a bare IDENT let-bound to one where no [ENT-5] kill event applies to a fact supported by p between the initializer and the match and that binding is no `set` target on that path, the `Ok(value: w)` arm establishes w = p ± k at arm entry; the `Err` arm establishes nothing.
Additionally, for a direct ordinary binding `let r = iand(a, b);` at unsigned integer type T, establish `r <= a` when a is an admitted term or constant and independently establish `r <= b` when b is one, in operand order; signed `iand`, every other bit operation, a nonterm operand, and a result not introduced by that direct binding establish no such relation.
For a direct ordinary binding `let r = ishl.wrap(one, count);` at unsigned integer type T, establish `r != Z` exactly when `one` is directly a checked typed literal or directly an earlier named const whose mathematical value is one.
A local binding merely proved equal to one, a const-generic value equal to one, a signed result, any other left operand, a non-direct result, and every other shift mode establish no nonzero fact.
The latter is sound because [OP-8] masks count modulo T's width, so shifting the one bit never clears it.
[ENT-3.S9]
- S9 (const-array element ranges).
For `let x = c[i];` where c is the bare IDENT of a named const of type `array<T, N>` [CONST-2] and T a fragment type, with vlo and vhi the minimum and maximum of its N declared element values, vlo <= x and x <= vhi are established at the binding.
The index's own bounds obligation [ENT-6] is judged separately and is unaffected.
Deeper const shapes establish nothing in this version.
[ENT-3.S10]
- S10 (boundary endpoint facts).
For a `match_stmt` or `value_match` whose scrutinee is directly a call to `read_at`, `write_once`, `directory_next`, `host_copy_bytes`, or `host_copy_utf8` [SYS-2, SYS-8], or a bare IDENT naming a `let` binding of that call's outcome type under the same no-kill, no-`set` path discipline as S7's checked-arithmetic origin: let s and e be the exact actuals bound to `start` and `end`, each read as a term or constant and still live at the match.
The `ReadBytes(next: w)` arm of `read_at`, the `ListBytes(next: w, entries: n)` arm of `directory_next`, and the `Ok(value: w)` arm of the other three independently establish `s <= w` and `w <= e` at arm entry; every other arm establishes neither endpoint fact.
Each result endpoint's [PRV-1] dependency additionally includes the concrete start actual, so this relation never launders an external start into an internal result.
These facts carry the same trust class as S6's allocation-length equality — a declared operation contract, never a writer statement.
The remaining [SYS-9] relations are retained checked-program facts and are not L0 fact sources in this version.
[ENT-3.S11]
- S11 (counted-range structural facts).
In a `for_stmt` preheader, immediately after the lower and upper endpoint values have been captured once in [FN-1]'s order, establish `lower_capture = lower_endpoint` and `upper_capture = upper_endpoint`, reading each admitted endpoint as its exact term or constant, and establish `binder = lower_capture` at the compiler-owned initialization.
Close that post-capture state under [ENT-4] before [ENT-5] forms the counted head state.
On every true header edge that actually enters the body, establish `lower_capture <= binder` and `binder < upper_capture`; the first follows from initialization plus the exact representable compiler updates, and the second from the header comparison just executed.
The capture-to-endpoint equalities are established once in the preheader only: no later header or body entry rereads an endpoint or reasserts a capture equal to the current value of a mutable endpoint source.
The false header edge, every `break` edge, and the counted continuation establish no S11 fact and in particular no `binder = upper_capture` postcondition.
[ENT-3.S12]
- S12 (verified user normal results).
For one call c and verified relation q, use exactly FN-9's `A0(c)`, per-relation `M(c,q)`, nonempty aggregate booleans Cq/Uq/Bq, and same-view call premise Gv(c).
Candidate scratch establishes q in each proof view exactly as [FN-9] fixes, after ordinary transfer and every applicable consume, borrow, callee-effect, and target kill.
This is FN-9's Bq-first evidence order, not a nondeterministic Boolean-parent choice.
Each substituted formal is independent: a referenced actual that has no ENT-2 image makes only that q unavailable, while an unreferenced non-ENT-2 actual has no effect on q.
FN-8 ephemeral actual-value datums never enter q.
The only destinations are the fresh direct ordinary-let binding, direct-call selected `Ok` payload, narrow direct-set receiver, and narrow selected-payload outer receiver [FN-9, ENT-5].
A named or pending outcome, stored or propagated whole outcome, false matching predicate, unavailable view, killed support, or rejected call establishes nothing.
After the one PRV batch succeeds, A(c) retains exactly these candidate facts unchanged in failure-atomic scratch; on any PRV event none is published, and only total [CLM-3] success makes them authoritative atomically.

The label S8 is retired, not reused: its midpoint family was struck as an owner-approved version amendment and may return as a later version's monotone addition the day a corpus program writes the shape.

[ENT-4] The L0 component of the closed fact state is the least set containing its established and implicit facts and closed under exactly: (1) from `t1 - t2 <= c1` and `t2 - t3 <= c2`, derive `t1 - t3 <= c1 + c2`; (2) from `t1 - t2 <= 0` and a disequality between t1 and t2 in either orientation, derive `t1 - t2 <= -1`; (3) of two bounds on one ordered pair, the smaller constant subsumes.
L0 derivability is exact: `a - b <= c` is derivable when the closed state contains `a - b <= c'` with c' <= c; `a = b` when both `a - b <= 0` and `b - a <= 0` are derivable; `a != b` when a disequality is present or `a - b <= -1` or `b - a <= -1` is derivable.

The opaque component retains established signed facts and the following finite truth-functional parent reconstruction over exact parent goals already interned in [ENT-2]'s universe.
`+band(A,B)` derives from both `+A` and `+B`; `-band(A,B)` derives from either `-A` or `-B`; `+bor(A,B)` derives from either `+A` or `+B`; `-bor(A,B)` derives from both `-A` and `-B`; and either sign of `bnot(A)` derives from the opposite sign of A.
Literal `True()` has an implicit positive proof and literal `False()` an implicit negative proof.
No `bxor` or Boolean-equivalence introduction is admitted in this version.
The closure considers only already-interned exact parent trees, uses the written rule order and minimum non-cyclic derivation depth, and creates no new formula.
Exact signed-goal identity includes every selected operation-table row, concrete selected operand type, and complete ordered operand GoalExpression.
`+G` is derivable when that exact positive fact is present, when G has an exact comparison projection R and L0 derives R, or when G is an integer-domain predicate whose fixed [ENT-6] component normalization proves true.
`-G` is derivable when that exact negative fact is present, when G has a comparison projection and L0 derives R's exact negation, or when G is an integer-domain predicate whose fixed normalization proves false.
Integer-domain component relations are only an alternate derivation route into that same exact signed goal; they establish no second source goal and receive no source-obligation identity of their own.
Derivability never decomposes a merely derived parent: [ENT-3] decomposes only source establishments, and S3 establishes its basis directly.
One retained proof never uses a parent-to-child source derivation and then that child solely to reconstruct the same parent; deterministic minimum-depth selection therefore contains no parent-child-parent cycle.

The combined state is contradictory when L0 derives `t - t <= -1` for any t or when both signs of one exact goal are derivable.
At a contradictory point every L0 relation and both signs of every goal in the finite universe are derivable and every ordinary obligation, call goal, and FN-9 selected-return relation is discharged.
CLM-2 checks contradiction before signs and therefore classifies no claim by this explosion.
At a non-contradictory query point, an instantiated goal G is `discharged` when `+G` is derivable, `refuted` when `+G` is absent and `-G` is derivable, and `unproved` otherwise.
An instantiated L0 relation R is `discharged` when every normalized conjunct of R is derivable, `refuted` when R is not discharged and R's exact negation is derivable, and `unproved` otherwise.
A one-bound negation is S1's reversed strict bound, an equality relation's negation is its disequality, and a disequality's negation is the equality's two-bound relation.
These three dispositions are complete and exclusive [FN-8, FN-9].
The least closure is unique and finite up to L0 subsumption because only the finite terms and goals [ENT-2] participate and the rules are monotone.
Implementations may compute lazily or incrementally, but every derivability and disposition answer must equal this least-closure answer.

[ENT-5] The support of an L0 fact is every tracked place occurring in its terms; every compiler-owned counted capture term occurring in its terms; for each length term len(P), the root binding of P but not P's element storage — an element write never kills a length fact, because a `buffer<T>` length is fixed at allocation and an `array<T, N>` or `slice<'r, T>` length is fixed by its type or creation [TYPE-2, OP-1]; and every borrow or box/arena holder binding any of its places reads through by `deref`, a bound call-result holder included — its resolved place is the candidate actual's complete resolved place [OWN-6], so a `set` commit or projected callee write through the chain kills exactly the facts supported by that storage.
Z, literals, and named const values have empty support and never die.
A counted capture is immutable and can die only on an edge leaving its compiler-owned construct scope.

The support of either sign of an opaque goal is the union of the resolved places whose values its complete typed expression reads.
A direct binding goal therefore depends on that binding, while its separately established complete origin expansion depends on the places read by the expansion.
For a `len(P)` node, support includes P's root and every holder used to reach it but not P's element storage, under the same fixed-length boundary as an L0 length term.
Literals and named const values add no support.
An ephemeral actual-value datum adds no support: it denotes an already evaluated captured value, is queried only at that immediate call judgment, and never causes the original subscript to be reread.
Every borrow or box/arena holder used by a goal's resolved place is also a support member.
The two signs of one goal have identical support.

One CLM-2 contribution component has the ordinary support of its exact S-derived signed goal or relation.
An F manifestation is queried with its own fully structural support but is never established by S3; a materialized D snapshot has D's direct support.
`Full-minus(c,a)` and `Full-minus(c)` change no evaluation, effect, ownership, cleanup, scope, join, loop, or runtime statement.
They suppress only the selected component-specific S3 source event or all S3 source events of c, then rerun the same source, kill, join, closure, and FN-9 publication from scratch.
No descendant, S reconstruction, D materialization, or cached proof depending solely on a masked source survives; an independently established equal D, S, F, relation, or goal remains available.
U already suppresses every S3 source, B differs from U only by suppressing S4, and claims change no PRV-1 value or storage flow.
Therefore every Full-minus run must produce exactly the same provenance-failure set as Full, independently of whether a terminal entailment root changes; any new or removed PRV-2/PRV-3 event is a compiler consistency failure, never a residual witness.

An S12 relation, a narrow-receiver relation, and a relation transported through `value_if` have exactly the ordinary L0 support of their terms after the route's stated substitutions.
The callee summary reference, proof view, call or delivery edge, pre-transfer substitution record, and a result or payload binder already replaced by its receiver are checked metadata, not additional support.
A route whose substitution leaves a non-[ENT-2] operand never creates an L0 fact.

Independently of relation flow, FN-9 entry-image stability begins live for each referenced parameter datum at function-body entry.
The same overlap, holder, consume, effect, scope-exit, and counted-continuing-kill classifications below permanently invalidate it; for a `len(P)` datum the element-storage exception is the same as for ordinary length support.
A structural merge retains stability only when every reaching input retains it, and a loop head removes stability for every datum a continuing kill may invalidate.
Neither contradiction, re-establishment of a fact, assignment of an equal value, nor a later iteration restores stability.
This metadata creates no snapshot, term, relation, signed goal, or runtime action.

A fact dies at the earliest of: (a) a [SET-1] `set` or [SET-2] `replace` commit whose resolved target [SET-1, SET-2, OWN-5] overlaps, under [OWN-7]'s overlap relation, the resolved place of any support member, or the compiler-owned update of a `for_stmt` binder when that binder is a support member — because a length term's support is its viewed place's non-element root path, a whole-place replace of a buffer or of any prefix of it kills that buffer's length facts, while an element-position replace, like an element write, kills none; after a SET-1 target kill, exactly [ENT-3.S5]'s applicable post-write image is established; (b) a call — user function, table operation, or system operation — one of whose [EFF-2] boundary-projected `writes` occurrences projects onto a caller place or origin set containing a place that overlaps [OWN-7] the resolved place of any support member; the projection is exactly [EFF-2]'s, so a callee writing only through one `&uniq` actual kills exactly the facts whose support overlaps that actual's resolved place, and a call whose row carries no `writes` kills nothing; (c) a consuming use [OWN-1] of any support member's root; (d) an edge leaving the region of any borrow holder in its support, leaving the lexical scope of any support binding, or leaving the owning counted construct of any capture term in its support, region exit [OWN-3] included.
Scope exits are edge events. After every earlier event and its stated post-event image on that edge, first close the reaching state under [ENT-4] while all lexical terms remain available; then apply kills (c) and (d); then close the surviving state before any query or join at the target.
A materialized conclusion survives exactly when its own support survives. Thus an arm-local term may be an intermediate vertex proving a relation among outer values, but no fact or goal whose conclusion still names that local, its holder, or its storage survives the scope into a join.

An ordinary user-call boundary has one order in every proof view.
First, at the pre-transfer point, complete the A0 judgments, retain each referenced formal's exact pre-transfer substitution, and judge that view's actual obligations and FN-8 goal.
Second, apply argument consumes and borrow commits, the callee's projected effect and write kills, and any route-specific target commit and kill.
Third, and only when `M(c,q)` still holds after those events, establish an eligible S12 relation with its result destination substituted.
A fresh direct ordinary-let result is introduced only after the call kills; a direct-call match result is introduced only after dispatch enters the exact selected `Ok(value:)` arm; no relation or pending token exists on the intervening whole outcome.
For the narrow direct-set route, the target kill precedes the result-to-post-write receiver substitution.
For the narrow selected-payload route, the payload relation already exists at arm entry, the right-hand side is evaluated, the outer target kill occurs, and only then is the result-payload term replaced by the post-write outer receiver.
No pre-transfer substitution carries an old fact through a kill, no later substitution reverses a kill, and every non-result support must still be live at establishment.

Bounded relation delivery is an additional edge transfer only for the `value_if` carrier admitted by [GIVE-1].
On one reaching eligible `give d;` edge, evaluate the bare atom's value first.
From the closed state at that point, take exactly each L0 bound or disequality whose normalized terms contain d; facts that do not contain d and opaque signed goals are not delivery candidates.
Replace every occurrence of d with the receiving binding x before applying the give edge's ordinary scope-exit and other event kills to every remaining support.
Thus d's own branch-scope exit cannot delete the already delivered relation, while the death of any other support deletes that relation normally.
Close the surviving substituted relations under [ENT-4] to form that edge's delivery image.
A non-bare, projected, consuming, computed, constructed, call, subscripted, literal, named-const, const-generic, capture, Z, contract-symbolic, wrong-mode, or wrong-type delivery forms no image; the value still follows ordinary GIVE-1 semantics.
A `value_match` forms no delivery image under any source shape.

At the receiving `let` continuation, ordinary fact flow and its ordinary branch join remain unchanged.
Separately join one delivery image from every reaching `give` edge of the `value_if`, in edge NodePath order, after the substitutions and kills above.
When at least one image is non-contradictory, contradictory images are neutral and the non-contradictory images retain for each ordered term pair the weakest (largest-constant) bound held by all and each disequality held by all; a relation missing from one such image is not delivered.
Hence images containing `x < 8` and `x < 128` establish `x < 128`, not nothing and not `x < 8`.
An all-contradictory image set is contradictory; an absent eligible relation on a non-contradictory edge contributes an empty image and prevents delivery of that relation.
Add exactly the joined L0 relations to the receiver's ordinary continuation state and close once.
This transport reads no pre-existing fact on x, forms no inverse `x ↦ d`, copies no unrelated relation, and creates no runtime operation.

Joins: at the continuation of a `match_stmt` or `value_match`, the fact state is the join of the states on every arm exit edge reaching that continuation on the conservative structural graph [FN-1], each taken after that edge's pre-exit closure, scope-exit kills, and surviving-state closure above; an arm every path of which leaves by `return`, `break` to an enclosing loop, or `propagate`'s error edge contributes nothing there.
In any nonempty join with at least one non-contradictory input, a contradictory all-derivable input imposes no constraint.
Over the non-contradictory inputs, the L0 join keeps for each ordered term pair the weakest (largest-constant) bound held by all and each disequality held by all; the opaque join keeps one signed fact exactly when that identical goal and sign are held by all.
The join of closed states is closed.
A nonempty join whose every input is contradictory, and an empty join with no reaching edge, are each the contradictory all-derivable state.
At the continuation of an `if_stmt` or `value_if`, this same join is taken over every branch exit edge reaching that continuation — for an else-free `if_stmt`, the false edge is such an edge — each after its pre-exit closure, scope-exit kills, and surviving-state closure; a branch every path of which leaves by `return`, `break` to an enclosing loop, or `propagate`'s error edge contributes nothing there.
The continuation of a `loop_stmt` uses the same join over its `break` edges.
A `loop_stmt` with no `break` naming its label has an empty join and therefore the contradictory state, consistent with that continuation being unreachable in truth while the conservative graph keeps it reachable.
A `propagate` right-hand side's `Err` edge leaves the function; its normal continuation keeps the preceding state subject to the initializer call's own kill events (b) and (c), and its binder gains no fact.

The continuation of a `for_stmt` is the join of its structural false-header edge and every `break` edge naming that counted label, each taken after the applicable pre-exit closure, all binder, capture, and body-scope exit kills, and surviving-state closure.
The false edge always exists in the conservative graph [FN-1], so this join is never empty.
A `break` naming an enclosing loop, a `return`, or a `propagate` error edge contributes nothing there.
Because the counted binder and both captures are out of scope before the join, no S11 body fact, capture fact, or claimed `binder = upper_capture` fact reaches the continuation.

Ordinary loops carry no induction in this version: the fact state at the head of each iteration of `loop @l { … }` is the state before the loop minus every fact having a support member that a continuing kill event of `@l` may kill.
A kill event (a)–(d) placed inside `@l`'s body, at any nesting depth, is continuing for `@l` exactly when some path of the conservative structural normal-control graph [FN-1] leads from the edge carrying that event to `@l`'s body entry without leaving `@l`'s body — that is, exactly when an execution taking that edge can reach a later iteration head of the same loop.
Every other kill event inside the body is not continuing and is not scanned: an event on or reachable only through a `break` edge naming `@l` or any enclosing loop, a `return` edge, or a `propagate` error edge leaves `@l` for the loop's continuation or the function-return sink [FN-1, ERR-3], and no iteration head of `@l` is reached from it without first re-entering `@l` from outside, where the enclosing flow supplies the state.
A kill inside a nested ordinary or counted loop whose continuation lies inside `@l`'s body is continuing for `@l`, including the kills carried on that nested loop's own `break` edges, because `@l`'s body entry is reached from that nested loop's continuation without leaving `@l`.
The surviving facts hold at every iteration head; establishment and kills then proceed ordinarily within the iteration, and no fact established inside an iteration survives to the next iteration's head.
A fact a non-continuing edge kills is still removed on that edge: the continuation join above takes each `break` edge after that edge's scope-exit kills, and an edge to the function-return sink reaches no queried program point, so narrowing this scan opens no path on which a dead fact is read.
Loop induction is a later version's [ENT-1]-monotone extension.

A counted `for_stmt` uses one compiler-owned structural recurrence, not writer-supplied induction.
First its preheader establishes the S11 capture equalities and binder initialization and closes that complete post-capture state under [ENT-4].
Second, its head state is that one closed post-capture state minus every fact having a support member that a continuing kill event of this counted loop may kill.
An event in the body, including the hidden normal-fallthrough binder update and body-scope cleanup, is continuing exactly when some path of the conservative structural normal-control graph [FN-1] leads from its edge through the counted header to a later entry of that same body without leaving the counted body; an event on or reachable only through a `break` naming that counted loop or an enclosing loop, a `return`, or a `propagate` error edge is not continuing.
Kills inside a nested ordinary or counted loop are classified by that same positive reachability predicate.
Third, on each true header edge, S11 adds the two structural body-entry bounds to that head state.
The hidden binder update kills every fact supported by the binder before a later header, while S11 re-establishes only its two stated bounds after the next true guard.
This order is fixed: preheader establishment and closure, then the continuing-kill subtraction, then S11 body-entry establishment.
Neither endpoint is evaluated again and neither capture-to-endpoint equality is re-established after the preheader.
Therefore a continuing write to a mutable endpoint source kills the direct capture-to-source equality, while a consequence already closed in the preheader whose support contains only immutable captures and other still-live terms may soundly survive.
No other fact established inside one counted iteration survives to a later counted head.

[ENT-6] An obligation is one normalized relation attached by a numbered rule to one source node, instantiated with that node's exact operands read as terms or constants; an operand that is not a term or constant leaves the relation underivable, never ill-formed.
This version attaches exactly four obligation families.
The first family: for every source subscript `P[i]` — read, write, and [SET-1] target position alike — the bounds obligation `i < len(P)`, normalized `i - len(P) <= -1`, at that subscript's `psuffix` node, one obligation per subscript in a chain, where `i` is the offset atom whose exact type [OP-4] fixes as `own u64`, so both sides are u64-typed and the relation is over their mathematical values.
The second family is IntegerDomain.
Every proof-required exact integer occurrence [OP-2] has exactly one obligation at its `infix` or `call` node.
Its canonical goal is the corresponding total `.defined` operation with the same selected concrete type and complete ordered operand-expression identities.
The disposition order in each complete, U, or B view is fixed: a contradictory state discharges; an exact positive canonical goal discharges; a complete fixed normalization proving true discharges; an exact negative canonical goal or fixed normalization proving false refutes; otherwise the goal is unproved.
One `IntegerDomainProof` root aggregates the selected route's parents in the fixed component order below.
Components are derivation-internal and are not separate obligations or protected-leaf identities.

For exact add, subtract, and multiply with at least one constant operand, the fixed normalization is the existing two-bound proof over mathematical integers, upper component then lower component.
For `t + c` and `c + t`: `t - Z <= max(T) - c`, then `Z - t <= c - min(T)`.
For `t - c`: `t - Z <= max(T) + c`, then `Z - t <= -min(T) - c`.
For `c - t`: `t - Z <= c - min(T)`, then `Z - t <= max(T) - c`.
For `t * c` with c > 0: `t - Z <= floor(max(T)/c)`, then `Z - t <= -ceil(min(T)/c)`; with c = 0 both components are `Z - Z <= 0`; with c < 0: `t - Z <= floor(min(T)/c)`, then `Z - t <= -ceil(max(T)/c)`.
For two constants, normalization is ground true exactly when the mathematical result belongs to T and ground false otherwise.
Two nonconstant add, subtract, or multiply operands have no L0 normalization route; their exact canonical goal remains writable and sufficient.

For exact division and remainder, normalization first requires `d != Z`.
For unsigned T the second component is ground true.
For signed T it succeeds when either `n != min(T)` or `d != -1` is derived, testing the dividend witness before the divisor witness; it is false exactly when `n = min(T)` and `d = -1` are both derived.
For exact negate and absolute, normalization is `x != min(T)`.
For exact shift, normalization is `k < K`, or `k - Z <= K - 1`, with K the selected value type's bit width.
For every IntegerDomain occurrence, failure is an OP-2 rejection carrying its canonical `.defined` goal and `unproved` or `refuted` disposition.
The family creates no provenance demand and no runtime operation.

The third family is AllocationFit.
Every `buffer_new(n, v)` has one canonical `buffer_fits<T>(n)` goal, and every `buffer_vacant<T>(n)` one `buffer_fits<Option<T>>(n)` goal, at that `call` node [OP-9].
An exact positive goal discharges first; otherwise its sole normalization component is `n <= floor((2^64 - 1) / stride_ceiling(S))` for the selected stored type S.
An exact negative goal or a derived false comparison refutes; absence of either proof is unproved.
The occurrence is protected with constrained subject n [PRV-2, PRV-3].
Failure is an OP-9 rejection and creates no allocation or runtime operation.

The fourth family is SystemRange.
Every call to one of [SYS-8]'s seven range-bearing operations has two independent goals in declared order at that `call`: ordinal zero is the exact `ile(start, end)` goal, with constrained subjects start and end; ordinal one is the exact `ile(end, len(buffer))` goal, with constrained subject end, where buffer is that operation's declared buffer parameter.
Each goal may discharge through its exact positive signed fact or its canonical L0 comparison projection; each exact negative fact or derived opposite comparison refutes; neither goal supplies a premise for the other.
Both complete-state goals must succeed before their protected-subject judgments [PRV-2, PRV-3].
The first refuted or unproved goal is a SYS-8 rejection and creates no host call, runtime condition, effect, or trap.
Failure of the first family's base judgment is the [OP-4] rejection, forms no provenance demand or event, and publishes no checked program; its diagnostic renders the residual as exactly: the offset atom's canonical source bytes, then ` < len(`, then the base place's canonical source bytes, then `)`.
The mechanical fix for any unproved family is one dominating branch establishing its canonical goal, or a CLM-2-admissible residual claim only when the predicate is a universally true current-function-local theorem the normative checker cannot derive — for a subscript in canonical ANF, one `let` binding `len(P)` followed by one such local claim on, or `if` over, the admitted comparison [CLM-1, ENT-3].
After complete-state success for a protected family, a [PRV-2] or [PRV-3] rejection makes the assertion-only route unavailable: the writer uses a dominating value branch whose false edge takes the domain outcome, or restructures so the external value no longer occupies the constrained-subject position.
For an offset atom that is itself a subscripted place — legal under [GRAM-5]'s place grammar but no term under [ENT-2] — the base fix first rebinds that inner read through one ordinary `let` (and, where the element type is narrower than u64, one total `cvt` [OP-6], both S5-tracked), making the offset a term whose own inner obligation is discharged the same way.
With at most that one rebinding step per nested offset, the fallback makes the goal writable, at a per-site cost from zero where facts already prove the bound to one retained claim where the missing theorem is CLM-1-local; rebinding a user-call or system-call result never makes it local, and cross-function behavior instead requires a verified FN-9/S12 relation or ordinary control.
The fallback does not by itself satisfy the provenance gate.

For checked metadata, each concrete obligation has identity `(concrete function instance, exact obligation-occurrence NodePath, family ordinal)`.
SubscriptBounds, IntegerDomain, and AllocationFit each use family ordinal zero; SystemRange uses zero for `start <= end` and one for `end <= len(buffer)`.
Only the protected families' identities are protected-leaf identities.
A requirement occurrence has identity `(the same form of concrete function instance, requires_clause NodePath)` [DIAG-2].
These occurrence identities do not participate in goal equality [FN-8].
The finite requirement-to-leaf bridge retained here is consumed by the active [PRV-2] and [PRV-3] judgments.
Those judgments attach derived provenance classes and source-acceptance dispositions to the same identities without changing goal equality, adding an optimizer consequence, or adding a runtime operation.

A parameter datum is `(zero-based value-parameter ordinal, selector)`.
A non-payload value has the sole selector `plain`.
For an enum carrying payload, the selectors are its exact `(variant declaration ordinal, payload-field declaration ordinal)` projections; the enum aggregate denotes the union of those projections and is not another selector.
Dependency is retained per value binding and per whole resolved storage root, with direct enum-payload projections only for value bindings and no field, element, variant, or path projection for storage.
It is flow-insensitive: one binding or root receives the union of its initializer and every `set` or call write whose resolved target overlaps that root, independent of branch order; joins do not subtract an edge.
Each ordinary parameter component is seeded with its own parameter datum.
Every other finite dependency is the least union generated below.

A literal or named const has empty dependency.
Ordinary copy, `move`, borrow, `deref`, and `reinterpret` preserve dependency.
A place read unions its whole storage root with every subscript-offset atom used to select the value; a write unions its right-hand-side dependency into the whole written root, while its target address contributes none.
A plain table-operation result unions its value operands, except that `len(P)` is empty.
For checked integer arithmetic and `cvt`, the `Ok(value:)` projection unions the value operands and the tag-only `Err(error:)` projection is empty; a total `cvt` preserves its operand.
A non-enum construction unions its field atoms.
An enum construction carries each field atom only into its exact direct payload projection and derives its aggregate by union; a nullary variant adds no payload edge.
A matching binder receives only its selected direct projection.
Whenever a payload-enum value binding is initialized or delivered, a source already carrying corresponding direct projections transfers them componentwise; a source carrying only one aggregate dependency — including a whole-storage read or one outer payload — conservatively seeds every direct projection of that binding rather than forming a recursive selector path.
This rule applies identically to ordinary initialization, match binding, propagation, `give`, return, and a user-call result.
`value_match` and `value_if` union corresponding `give` components, while their scrutinee or condition contributes no dependency merely by selecting a path.
`propagate` carries only `Ok(value:)` to its binding and the selected `Err(error:)` component to the enclosing result.
A counted binder receives its lower endpoint's dependency at initialization and the compiler-owned unit increment preserves that dependency; the upper guard contributes none merely by controlling iteration.

Each concrete function retains a result component and one write component per `&uniq` parameter in addition to the requirement-to-leaf relation below.
Explicit returns union their plain or direct payload dependencies componentwise into the result, and a propagation error contributes its selected error component.
A write component unions every right-hand side written to a root overlapping that formal, together with each callee write component whose [EFF-2] projection reaches it; a system write adds no parameter datum and seeds the destination component's unconditional-external bit exactly when [SYS-2]'s closed table classifies that writable parameter external.
An ordinary user call substitutes the actual component dependencies into the callee's result and write components.
A system result adds no formal parameter datum and seeds each plain or direct payload component's unconditional-external bit exactly as [SYS-2]'s closed table fixes; an internal component seeds no bit, while a dependent endpoint component additionally unions the concrete call's `start` actual dependency.
Storage, result, write, and user-call component propagation are solved first to a least fixed point over the finite concrete instances and then frozen.
Recursion and mutual recursion in this component stratum therefore use that fixed point, not traversal order or a stored witness path.
Only after the component pairs freeze does the direct-demand and requirement-bridge stratum below inspect them; that second stratum never feeds a bit or datum back into a component pair.
The ephemeral actual-value datum of FN-8 is separate from this dependency judgment: its originating checked actual still carries the ordinary root, offset, and operand dependencies defined here.

For CLM-1 only, ENT-6 also computes one independent finite forward **claim-authority** state over the structural normal-control graph.
Claim authority is not an entailment fact, optimizer fact, callee summary, or [PRV-1] provenance pair, and it grants no operation authority.
Each component is `Local` or `BoundaryResult(witness)`; component join retains `BoundaryResult` when either input has it and retains the earliest witness in stable source order.
The component tree is structural and finite: a scalar or opaque value has one plain component; a struct has its recursively selected declaration-order fields; an enum has its tag and recursively selected declaration-order payload fields; an array, slice, or buffer has its length and one conservative all-elements component; and a box, arena, or borrow holder retains both the holder path and every selected dereference path used by a claim support.
A uniform authority on a whole value applies to every existing or later-materialized descendant.
Construction and projection are component-sensitive: a boundary field or payload does not taint an independent local sibling, while reading or operating on the whole aggregate joins all selected components.

Every source parameter component, command-entry parameter component, literal, named const, and otherwise untainted local initializer begins `Local`; this judgment does not classify external input provenance.
Every result component of every ordinary user call and every system call begins `BoundaryResult`, including a scalar, tag, payload, aggregate field, length, element, box or arena content, borrow holder, and value read through that returned holder.
This seed is unconditional: it does not inspect or substitute the callee body, arguments, effect row, [PRV-1] class, a system component's external/internal/dependent class, or an FN-9/S12 relation.
An `ensures` relation remains an independently verified fact for direct caller consumption and never declassifies any component of its returned value.

Ordinary copy or move, conversion, reinterpretation, arithmetic, float, Boolean and enum operations, `imin`, wrapping identity operations, allocation-fit operations, and every other total value operation join the authority of the value components they read into the result components they produce.
Struct, enum, array, buffer, slice, box, and arena construction transfers each operand to its exact component where that component is known; projection, matching, propagation, `give`, return delivery, and dereference preserve the selected component, and an aggregate-to-unresolved-payload transfer conservatively joins the aggregate into every possible direct payload.
A place read obtains the reaching authority of its selected storage component and joins the authority of any value used to select a conservative element; an explicit `set` or `replace` transfers its right-hand side and selector authority into the written component.
An unconditional explicit write to one statically exact whole value or exact field component is a strong replacement and may clear an older boundary marker when its right-hand side is Local; an element write, partial write, possible-overlap write, or other write represented by a shared conservative component joins and never clears.
Control-flow joins combine corresponding components and never subtract a boundary witness.

Claim authority deliberately includes control dependence although [PRV-1] provenance does not, and it includes exactly the control dependence a selection carries.
A `BoundaryResult` condition, match scrutinee or tag, counted endpoint, or other selector chooses an edge; its witness joins each matching binder that edge's arm introduces, each value `value_if` or `value_match` delivers along it, and, at each ordinary match reconvergence, loop head, and loop exit the selector reaches, exactly those components whose reaching definition on one incoming edge is a different definition occurrence from their reaching definition on another.
Two reaching definitions are the same occurrence when they are one definition of that component, not when two separate definitions compute equal values; `value_if` and `value_match` deliver a selected value in every case, so selecting constants on the two arms or selecting the same local value on both arms does not declassify the delivered value.
Standing on a boundary-selected edge is not itself a selection.
An ordinary binding, a computed value, or a storage write whose own operands are every one Local — a literal, a named const, an ordinary parameter, or another Local value — is Local, and stays Local across a reconvergence, loop head, or loop exit whose every incoming edge reaches it through that one definition, whether it stands inside the selected arm or in post-join state.
Thus writing a local constant on one arm and joining it with the other arm's older definition, and updating loop-carried state under a boundary-selected iteration, each retain the selector's witness at the join, while a definition formed after the join from literals, named consts, parameters, and other Local values is Local although a boundary result selected the edge that reaches it.
So a `match` on a system-call result whose `Err` arm returns leaves a following `let seed = 3209_u64;` and `let offset = seed % 64_u64;` Local, and `claim guard: ilt(offset, 64_u64)` is admitted; the same claim over a value that reads the delivered payload, a binder joined from two arms, storage the selected edge wrote and the other edge did not, or state a boundary-selected loop updates remains non-local and is refused.

A call's result seed is the only call event added by this first locality version.
A user or system call's possible write through an `&uniq` actual does not by itself change claim authority for that caller storage; ordinary effect, kill, provenance, and obligation judgments remain unchanged, and an explicit later write of a boundary-derived right-hand side still transfers normally.
Extending locality to call-written storage is an amendment-level accepted-set change rather than an implementation choice.

One boundary witness contains the introducing call's NodePath and kind, plus the user callee's source declaration origin and source name or the system operation's `system_declaration_ordinal` and spelling.
When more than one witness reaches a component, the least call NodePath wins, with boundary kind and the stable callee identity used only as a deterministic tie-break at one path; no scratch or dense identity is publishable.
The authority analysis is computed once before S3, U, B, `Eligible`, or any `Full-minus` mask and is reused unchanged by every claim component query.
For one component, CLM-1 queries exactly [ENT-5]'s ordinary S-derived relation or opaque-goal support, including each root and holder; canonical normalization may add a fact identity but never subtract authority support read by the retained S expression.

The protected families and constrained subjects are closed.
For SubscriptBounds the sole subject is offset i in `i < len(P)`.
For AllocationFit the sole subject is n.
For SystemRange goal zero the subjects are start then end; for goal one the sole subject is end.
IntegerDomain is not protected.
Each subject's parameter datums are exactly that value's parameter-dependency set at the obligation.
A buffer base, `len(buffer)`, comparison bound, write target, type/layout constant, and every other goal operand contributes no subject datum merely by being mentioned.
A protected leaf with no subject parameter datum still has its structural bridge identity; an implementation must not manufacture a datum from a bound, base, type, or another goal operand.

For the active gate, the **complete state** is the ordinary [ENT-3] flow and closure used by the base [OP-4] and [FN-8] judgments.
The **unasserted state** U is that flow recomputed with S3 establishment disabled and every other source, kill, join, loop rule, and closure unchanged.
The **S4-blinded state** B is U with every positive S4 goal and each exact L0 projection omitted at body entry.
Only a leaf whose complete-state base judgment succeeds reaches the local demand generator.
If B discharges the leaf, add no demand.
Otherwise, inspect each constrained subject in its fixed order.
A subject component whose unconditional-external bit is true creates the local [PRV-3] candidate regardless of U and regardless of whether the component also carries parameter datums; retain those datums only as explanations and add no direct demand or bridge tuple for that subject.
Only a component whose unconditional bit is false reaches the remaining partition: if U does not discharge the leaf, add one direct `(subject parameter datum, leaf)` demand for each subject datum; if U discharges it while B does not, add the structural pair from the complete ordered requirement-occurrence set to the leaf and add one `(requirement occurrence set, subject parameter datum, leaf)` bridge tuple for each subject datum.
A false bit with no subject datum is internal and creates no rejection or caller-visible target.
A complete-state failure remains the obligation family's owning rejection, forms none of these members, and publishes no checked program.
A function with no requirements cannot distinguish U from B.

S12 establishment and bounded `value_if` delivery are computed independently in each of these three views and enter one fixed optimistic semantic batch before provenance finalization.
A complete-only S12 relation is absent from U and B; a U-but-not-B relation enters U only through that caller's exact U call premises and enters B only when FN-9's B alternative independently permits it; a B relation may enter all views.
A delivered relation remains in only the view of its source relation.
These facts may discharge a protected leaf in that same view, but they add no parameter datum, component predecessor, unconditional-external bit, protected family, constrained subject, demand kind, bridge kind, or callable component.
PRV-1 therefore converges and freezes exactly the ordinary dependency components first.
The fixed candidate fact batch then supplies complete/U/B outcomes to the existing PRV-2/PRV-3 demand and event stratum.
A resulting event discards the whole candidate fact batch with the unpublished checked program; absence of every event leaves the unchanged batch in failure-atomic scratch until [CLM-3] succeeds, vacuously for a unit with no marker, and only that success finalizes it atomically.
No per-fact retraction, negative lattice edge, second provenance class, or second flow pass exists.

At an ordinary call, full-state [FN-8] acceptance is first; a refuted or unproved complete instantiated goal forms no [PRV-2] target.
After full success, a callee direct demand always composes through the selected actual component: a true unconditional-external bit creates the local call-argument candidate and retains any parameter datums only as explanations, while only a false bit permits each caller parameter datum in that component to add the corresponding direct demand to the caller.
A callee bridge target instead rejudges the complete instantiated call goal in the caller's U and B states.
If B discharges it, the chain ends at that evidence.
Otherwise, if the selected actual component's unconditional bit is true, create the local call-argument candidate regardless of U and retain any parameter datums only as explanations; the bit and those datums do not propagate as a direct demand or another bridge.
Only a selected component whose unconditional bit is false reaches the remaining bridge partition: if U does not discharge the goal, each caller parameter datum in that component becomes a direct caller demand; if U discharges it while B does not, an ordinary caller adds the structural bridge from its requirement occurrence to the inherited leaf, composes those parameter datums into bridge tuples, and retains the call and downstream requirement occurrence as witness predecessors.
A `command` entry has no caller to continue the latter bridge, so a false-bit selected actual creates no local gate event there.
No synthetic external parameter datum is introduced.

After the component stratum converges and freezes, take direct-demand, bridge, call-target, and event composition together to a second least fixed point over the finite concrete function instances, frozen component pairs, requirement occurrences, full-state-accepted calls, parameter datums, and protected leaves.
Complete/U/B outcomes and every tested unconditional bit are fixed inputs to this stratum; its transfers only add set members, so a false-bit premise never later becomes true and requires no retraction.
Direct, recursive, and mutually recursive demand paths therefore converge independently of traversal order, while a recursive component with no local protected-leaf seed remains empty.
Multiple datum or leaf explanations for one actual argument remain distinct targets, but [PRV-2] coalesces them into one event per call and argument.
Witness paths and tie-breaking predecessors are selected only after this second convergence and cannot change either lattice.
The checked program retains the converged components, direct demands, structural pairs, bridge tuples, call links, complete/U/B outcomes, target sets, and deterministic finite witness predecessors [DIAG-2].
An unconditional-external bit is never replaced by or propagated as parameter-only demand metadata: it terminates at its local leaf under [PRV-3], or at its call argument under [PRV-2] for a direct demand or a B-failing bridge, retaining any parameter explanations only for diagnostics.
At a `command` entry, each labelled input is unconditionally external [PRV-1]; a B-failing direct local leaf whose subject carries that bit is owned by PRV-3, while a B-failing inherited bridge whose selected actual carries that bit is owned by PRV-2 at that call's argument.
This active bridge and gate add no runtime operation, fallback check, trusted assertion, or optimizer consequence.

For [CLM-3], the unasserted U state is exactly the unasserted state U above, retaining every S4 source after its independently proved incoming boundary.
Each demanded ordinary-call goal queries its existing instantiated goal in caller U, and each demanded protected leaf retains its already-produced U derivation root as checked metadata without a separate [CLM-3] rejection.
These strict queries introduce no new obligation family, protected subject, provenance class, component dependency, direct demand, bridge, call target, fact source, or repair.
After PRV-1 freezes and PRV-2 or PRV-3 produces no event, the candidate S12 and delivery batch remains unchanged in failure-atomic scratch until every applicable CLM-3 query and summary succeeds; only then does the sole finalization publish that batch and the checked program.
Any strict event discards both, with no per-fact retraction or second pass.

[PRV-1] Provenance is a derived two-class explicit-dataflow judgment over exactly the finite components whose dependency transfer [ENT-6] retains.
A plain value binding and a whole resolved storage root each have one aggregate component.
A payload-carrying enum value binding instead has one component for each direct `(variant declaration ordinal, payload-field declaration ordinal)` projection and an aggregate defined as their join; storage has no payload, field, element, variant, or path projection.
Every component carries the pair `(unconditionally external, parameter datums)`, where the first member is one Boolean and the second is the finite [ENT-6] set.
Join is Boolean disjunction and set union.
Under a concrete assignment of classes to ordinary parameter datums, the component is **external** exactly when its Boolean is true or at least one member of its parameter set is external; otherwise it is **internal**.
An ordinary source parameter component begins with only its identity parameter datum.
Each labelled `command` entry parameter instead begins unconditionally external and creates no caller-substitutable sentinel.
A [SYS-2] result or writable component begins with exactly its table-fixed unconditional bit and no formal parameter datum; a dependent endpoint result then joins the concrete call's `start` actual pair.
These entry and system components are the only unconditional external origins.

The complete transfer is [ENT-6]'s positive dependency transfer applied componentwise to that pair.
In particular, storage is flow-insensitive per binding and whole root; every initializer, overlapping `set`, and projected call write joins, and no later flow subtracts an edge.
A selected place read joins its root and every explicit subscript-offset atom in the resolved place, field selection preserves the accumulated pair, and `len(P)` is internal.
A `set` target's address contributes nothing to the stored value or root; only its right-hand side does.
Literals and named consts are internal.
Ordinary table-operation results join their value operands, checked arithmetic and `cvt` carry those operands only into `Ok(value:)` while a tag-only `Err(error:)` is internal, total `cvt` and `reinterpret` preserve, system dependent endpoints join their concrete `start` actual, and construction, matching, `give`, propagation, return, and counted-binder initialization follow the exact component transfers already enumerated by ENT-6.
When a payload-enum target receives only one aggregate component — including from whole storage or one outer payload — that aggregate conservatively seeds every direct projection; selectors never recurse.
A user-call result and write substitute the callee's current result or write pair through the exact actual component.
Result, write, storage, and user-call pair propagation form the first finite least fixed point.
Once converged, every pair is frozen and retained in the [PRV-2] boundary column before any direct-demand, bridge, target, or event generator runs.

No branch, match arm, loop guard, variant tag, or other control choice contributes provenance merely by selecting a path.
No target-address operand contributes merely by selecting a write.
There is no path-sensitive storage, recursive payload path, implicit-flow analysis, integrity judgment, writer-spelled provenance annotation, trusted assertion, or optimizer assumption.
An external value used only as a bound, base, target address, or unrelated goal operand therefore does not become a constrained subject by association.

Every positive PRV-1 predecessor edge has exactly one carrier NodePath, fixed by this exhaustive mapping rather than implementation choice.
An ordinary or labelled-entry seed uses its complete `param` node.
A system result or system write uses that system-operation `call` node.
A user-call result, projected write, or substituted call-component edge uses that user `call` node.
Within an explicit source expression, a place read, copy, move, borrow, dereference, operation result, infix result, or construction edge uses the smallest selected `atom`, `call`, `infix`, or `construct` node that produces the receiving value.
Delivery from an expression into an ordinary or propagated binding uses the owning `let_stmt`; delivery into storage uses the owning `set_stmt`; delivery into a function result uses the owning `return_stmt`, or the owning propagated `let_stmt` on its automatic error edge; delivery through `give` uses the owning `give_stmt`; and delivery into a match binder uses that complete `fieldbind` node.
The componentwise delivery from a `value_match` or `value_if` into its binding uses the owning `let_stmt`, after its contributing `give_stmt` edges.
A counted binder's initialization edge uses the lower endpoint's `atom` node, and its compiler-owned dependency-preserving increment uses the owning `for_stmt` node.
These cases cover every positive transfer enumerated by [ENT-6]; no control-only or address-only non-edge receives a carrier.
External origins are the labelled-entry `param` carrier or the system-operation `call` carrier and, for a system write, its exact writable parameter and caller actual.
Every predecessor tie, rendered payload, and origin coordinate in [PRV-2] and [PRV-3] uses this carrier mapping and the carrier node's complete checked half-open extent.

[PRV-2] Every concrete [FN-2] function instance derives one caller-visible provenance column.
Its result and per-`&uniq` write components are the converged PRV-1 pairs retained by [ENT-6].
Its protected-demand part retains each direct `(parameter datum, protected leaf)` demand and each `(requirement occurrence, parameter datum, protected leaf)` bridge tuple produced by ENT-6's complete/U/B partition.
A parameter datum is exactly ENT-6's zero-based parameter ordinal and selector; selector order is `plain` when present, then direct enum projections in variant-declaration and payload-field declaration order.
An unconditional-external bit is never represented by a synthetic parameter datum.
No mention-all-parameters, whole-goal-support, recognizer, or second goal language adds a member.

At a full-state-accepted call `c: F -> G`, ENT-6 composes every direct demand and every still-live bridge target through the selected plain or payload component of the corresponding actual.
For zero-based argument ordinal `q`, `Targets(c, q)` is the finite set of all direct-demand records whose selected actual component has a true unconditional-external bit, together with every bridge record for which the caller's B state fails and that selected component has a true unconditional bit.
A bridge that B discharges contributes no target even when the bit is true; after B fails, a true bit contributes a target regardless of U, while parameter datums beside that bit remain explanations and never replace or propagate it.
The same partition applies inside a `command` entry, which has no ordinary caller to continue a false-bit bridge.
Each record retains the callee parameter datum, demand kind, exact protected leaf, every bridge predecessor, and the nonpropagated companion parameter datums in the parameter order above.
If `Targets(c, q)` is nonempty, the compiler emits exactly one hard rejection event for that `(call, q)` pair, citing PRV-2 with `SourceNode` at the existing argument `atom` node and `SourceCoordinate` equal to that atom's complete checked half-open extent.
A second datum, leaf, or route at the same argument enlarges the retained target set but creates no second event.
Events at different argument ordinals remain distinct.
A call whose full goal is refuted or unproved is only [FN-8], creates no target, and never reaches this judgment.
The command entry remains uncallable [FN-7].

With every [PRV-1] component pair converged and frozen, direct-demand, bridge, call-target, and event composition form the second finite monotone product fixed point over the closed unit.
There are finitely many concrete instances, frozen components, full-state-accepted calls, parameter datums, requirement occurrences, and protected leaves, and every transfer in this stratum only adds a set member, so its least fixed point exists, is unique, terminates, and is independent of visit order.
No component bit or dependency set changes here; every false-bit branch is therefore a fixed premise rather than a negative dependency on a growing lattice.
Direct, recursive, and mutually recursive routes use this second fixed point.
A recursive component with no local protected-leaf seed remains empty.
Result and write pairs are outputs of the first stratum and fixed inputs here; call paths and witness choices are absent from both lattices.

Only after both strata converge does one PRV-2 event select its diagnostic witness.
A complete demand-state identity is either `direct(concrete function, parameter datum, protected leaf)` or `bridge(concrete function, requirement occurrence, parameter datum, protected leaf)`; the fixed tag order is `direct < bridge`, and a bridge retains its exact requirement occurrence rather than collapsing to the same function, datum, and leaf.
The witness first minimizes call boundaries.
Ties compare lexicographically the complete sequence of boundary states.
Each boundary contains the call NodePath, argument NodePath, complete callee demand-state tag and bridge occurrence when applicable, callee parameter datum, and one optional caller continuation.
The optional continuation uses the fixed tags `absent < present`; `absent` is the real terminal-boundary case and contains no caller parameter datum or synthetic sentinel, while `present` contains the resulting caller demand-state tag and bridge occurrence when applicable plus its caller parameter datum.
Demand-state tags, requirement-occurrence NodePaths, parameter ordinals, and selectors compare in that written order; concrete-instance-only ties use the stable order below.
At a true-bit rejecting boundary the caller continuation is absent even when the frozen selected component also carries parameter datums; those companion datums remain a separately ordered diagnostic explanation list and do not enter the propagated route.
The protected leaf's concrete instance, obligation NodePath, and family ordinal follow that route key.
A remaining tie between concrete instances uses one implementation-defined but stable deterministic instance order fixed for that compiler executable and independent of hash iteration, worklist order, allocation identity, or worker scheduling.
At the terminal boundary, the PRV-1 predecessor suffix follows only edges carrying the true unconditional bit to its labelled-entry or [SYS-2] origin, minimizes component edges, and then compares the complete sequence of carrier NodePaths fixed by PRV-1 and their selectors lexicographically; a companion parameter path is not eligible as that suffix.
Reconstruction records visited complete demand-state identities, so `direct(F, d, L)` and `bridge(F, R, d, L)` remain distinct even at the same function, datum, and leaf, while revisiting one identical full state cuts the cycle and recursion never appears as an infinite witness.

The event payload retains the complete ordered `Targets(c, q)` set, the selected leaf's [ENT-6] residual, and one rendered chain from that leaf backward through requirement occurrences, callee datums, and call boundaries to the rejecting actual, then through that actual's PRV-1 predecessors to its labelled-entry or [SYS-2] origin.
For a direct demand, its legal repair is a real branch in the protected leaf's owning body that establishes the residual and takes the domain outcome on the false edge.
For a requirement-bridge target, the branch instead establishes the complete bridged call goal in the rejecting caller's unasserted state before that call.
Either kind may also be repaired by restructuring the route so the external value no longer reaches the protected constrained subject.
A `claim` is not a repair for an unconditionally external constrained subject.

[PRV-3] This rule owns only a local protected leaf, including a leaf local to a command entry.
The [ENT-6] complete-state family judgment runs first.
If it fails, that family's owning rule is the sole rejection and no PRV-3 candidate exists.
After success, inspect the closed constrained-subject list [ENT-6] in order.
If B discharges the leaf, no provenance demand remains.
If B does not discharge it and any subject's PRV-1 pair has a true unconditional-external bit, the leaf is one hard rejection citing PRV-3 with `SourceNode` at its existing obligation-owning `psuffix` or `call` node and `SourceCoordinate` equal to that node's complete checked half-open extent, regardless of U and regardless of whether the pair also carries parameter datums.
All external subjects and companion datums remain ordered diagnostic explanations but create no direct demand, bridge tuple, or second event.
Only false-bit subject pairs reach the remaining partition.
An empty parameter set is internal and creates no rejection or caller-visible target.
With a nonempty parameter set, failure in U retains the direct demand for [PRV-2], while success in U followed by failure in B retains the complete ordered S4 requirement bridge for PRV-2; neither case is a local PRV-3 rejection.

A `command` entry follows the same partition and has no contract, S4 source, or ordinary caller.
Every labelled input has a true unconditional-external bit, so a direct entry-local leaf whose constrained subject carries that bit must be justified by a real source branch present in U and B; a claim-only proof is the local PRV-3 rejection above.
An inherited leaf reached through an entry-body call remains a call-argument judgment and is owned only by [PRV-2].
This entry disposition adds no wrapper, foreign adapter, alternate error protocol, source surface, or second body.

The unasserted state removes exactly S3 claim establishment.
S1 branches, every S4 requirement source, S5, S6, S7, S9, S10, S11, every kill and join, and [ENT-4] closure remain unchanged; B additionally removes every S4 source.
Thus a `claim` may not authorize an external constrained subject, while an internal subject may use one only when CLM-2 also proves that exact occurrence and every contribution individually necessary for an allowed terminal root.
Provenance of a buffer base, `len(buffer)`, a comparison bound, a target address, a type/layout constant, or another non-subject goal operand does not gate the obligation.

A PRV-3 payload contains the exact ENT-6 residual, the shortest post-convergence PRV-1 chain from the subject component to its labelled-entry or [SYS-2] origin, and the two legal repairs: a dominating real branch whose false edge takes the domain outcome, or a restructure in which the external value no longer occupies the constrained-subject position.
An entry-bridge payload additionally retains the requirement occurrence and deterministic bridge predecessor.
Component-edge count is minimized first; ties use complete predecessor NodePaths and selector order exactly as PRV-2.
A local PRV-3 rejection never relocates to an upstream claim and never becomes a call-argument event.

## 19. Worked example (normative bytes)

[EX-1] The following complete program is byte-exact canonical form:

```
enum Sign {
  Neg();
  Zero();
  Pos();
}

fn sign_of(x: own i32) -> result: own Sign pure {
  doc "Conditional value produced by returning from branches (canonical for return position).";
  if ilt(x, 0_i32) {
    return Neg();
  } else if ieq(x, 0_i32) {
    return Zero();
  } else {
    return Pos();
  }
}

command fn main() -> status: own ExitStatus pure {
  doc "let-initializer match with give: a conditional value bound, then reused.";
  let a = 40_i32;
  region 'r {
    let p = &'r a;
    let v = match deref(p) +checked 2_i32 {
      Ok(value: w) => {
        give w;
      }
      Err(error: e) => {
        let failed = exit_status(code: 1_u8);
        return move failed;
      }
    }
    let expected = ieq(v, 42_i32);
    if expected {
    } else {
      let failed = exit_status(code: 1_u8);
      return move failed;
    }
  }
  let success = exit_status(code: 0_u8);
  return move success;
}
```

## 20. Spec meta-rules (CI-checked)

[META-1] Spec-CI enforces the regularity invariants defined elsewhere: one spelling per construct [FORM-1] and a 1:1 production-to-core-tree-node mapping [GRAM-1].
Its unique machine-checked content is that no rule ID is defined twice and every cross-reference resolves [META-4, META-6].
[META-2] No context-dependent spellings or rule variants: no rule's meaning depends on surrounding context; defaulting rules do not exist.
[META-3] No rule carries an exception clause; conditional structure is expressed as total positive rules or table data.
[META-4] Every normative fact is stated once; other mentions are rule-ID cross-references.
[META-5] Every change to this artifact declares its spec delta (rules ±, tokens ±, spellings ±, exceptions ±) and its SELECTION GROUND (evidence-selected vs minimality-selected) in this document's status header.
`WORKFLOW.md` defines the candidate and approval loop, and exact approvals are recorded in `governance/APPROVALS.md`; DEFERRED markers are tracked delta obligations.
[META-6] Every rule carries an entry in `spec/derivation/derivation-ledger.md` tracing it to `docs/constitution.md`; a rule whose chain is refuted or orphaned (evidence card dies, constitutional premise amended) is flagged for re-grounding, and underived rules may not ratify.
The native `whitefoot-spec` gate checks that every active rule ID has a ledger row.
